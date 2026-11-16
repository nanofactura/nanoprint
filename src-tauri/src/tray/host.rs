//! A login session may start the application before its `StatusNotifier` host.

use futures_util::StreamExt;
use tauri::AppHandle;
use zbus::{Connection, fdo, proxy::CacheProperties};

const WATCHER: &str = "org.kde.StatusNotifierWatcher";
const PATH: &str = "/StatusNotifierWatcher";

pub(super) fn create(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let ready = async {
            let connection = Connection::session().await?;
            wait_for_host(&connection).await
        }
        .await;
        if let Err(error) = ready {
            log::warn!(
                "could not wait for the system tray host; continuing without a tray: {error}"
            );
            return;
        }
        let tray_app = app.clone();
        if let Err(error) = app.run_on_main_thread(move || {
            if let Err(error) = super::build(&tray_app) {
                log::warn!("could not create the system tray; printing will continue: {error}");
            }
        }) {
            log::warn!("could not schedule system tray creation; printing will continue: {error}");
        }
    });
}

async fn wait_for_host(connection: &Connection) -> zbus::Result<()> {
    let bus = fdo::DBusProxy::new(connection).await?;
    let owners = bus
        .receive_name_owner_changed_with_args(&[(0, WATCHER)])
        .await?;
    let watcher: zbus::Proxy<'_> = zbus::proxy::Builder::new(connection)
        .destination(WATCHER)?
        .path(PATH)?
        .interface(WATCHER)?
        .cache_properties(CacheProperties::No)
        .build()
        .await?;
    let hosts = watcher
        .receive_signal("StatusNotifierHostRegistered")
        .await?;
    // Subscribe before checking readiness, so neither the name nor host can appear unnoticed.
    let mut changes = futures_util::stream::select(owners.map(|_| ()), hosts.map(|_| ()));
    let mut waiting = false;
    loop {
        match watcher
            .get_property::<bool>("IsStatusNotifierHostRegistered")
            .await
        {
            Ok(true) => return Ok(()),
            Ok(false) => {}
            Err(error) => match fdo::Error::from(error) {
                fdo::Error::ServiceUnknown(_) | fdo::Error::NameHasNoOwner(_) => {}
                error => return Err(error.into()),
            },
        }
        if !waiting {
            log::info!("waiting for the system tray host");
            waiting = true;
        }
        changes.next().await.ok_or_else(|| {
            zbus::Error::Failure("system tray D-Bus signal streams closed".into())
        })?;
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::{BufRead, BufReader},
        process::{Child, Command, Stdio},
        sync::atomic::{AtomicBool, Ordering},
        time::Duration,
    };

    use super::*;

    struct Bus {
        child: Child,
        address: String,
    }

    impl Bus {
        fn new() -> Self {
            let mut child = Command::new("dbus-daemon")
                .args(["--session", "--nofork", "--print-address=1"])
                .stdout(Stdio::piped())
                .spawn()
                .expect("tray host tests require dbus-daemon");
            let mut address = String::new();
            BufReader::new(child.stdout.take().unwrap())
                .read_line(&mut address)
                .unwrap();
            Self {
                child,
                address: address.trim().to_owned(),
            }
        }

        async fn connect(&self) -> Connection {
            zbus::connection::Builder::address(self.address.as_str())
                .unwrap()
                .build()
                .await
                .unwrap()
        }
    }

    impl Drop for Bus {
        fn drop(&mut self) {
            self.child.kill().unwrap();
            self.child.wait().unwrap();
        }
    }

    struct Watcher(AtomicBool);

    #[zbus::interface(name = "org.kde.StatusNotifierWatcher")]
    impl Watcher {
        #[zbus(property)]
        fn is_status_notifier_host_registered(&self) -> bool {
            self.0.load(Ordering::SeqCst)
        }

        #[zbus(signal)]
        async fn status_notifier_host_registered(
            emitter: &zbus::object_server::SignalEmitter<'_>,
        ) -> zbus::Result<()>;
    }

    async fn publish(connection: &Connection, ready: bool) {
        connection
            .object_server()
            .at(PATH, Watcher(AtomicBool::new(ready)))
            .await
            .unwrap();
        connection.request_name(WATCHER).await.unwrap();
    }

    #[tokio::test]
    async fn accepts_an_already_registered_host() {
        let bus = Bus::new();
        let host = bus.connect().await;
        publish(&host, true).await;
        let client = bus.connect().await;
        tokio::time::timeout(Duration::from_secs(5), wait_for_host(&client))
            .await
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn waits_for_both_the_watcher_and_its_host() {
        let bus = Bus::new();
        let client = bus.connect().await;
        let ready = wait_for_host(&client);
        tokio::pin!(ready);
        assert!(
            tokio::time::timeout(Duration::from_millis(100), &mut ready)
                .await
                .is_err()
        );
        let host = bus.connect().await;
        publish(&host, false).await;
        assert!(
            tokio::time::timeout(Duration::from_millis(100), &mut ready)
                .await
                .is_err()
        );
        let watcher = host
            .object_server()
            .interface::<_, Watcher>(PATH)
            .await
            .unwrap();
        watcher.get().await.0.store(true, Ordering::SeqCst);
        Watcher::status_notifier_host_registered(watcher.signal_emitter())
            .await
            .unwrap();
        tokio::time::timeout(Duration::from_secs(5), &mut ready)
            .await
            .unwrap()
            .unwrap();
    }

    struct DeniedWatcher(&'static str);

    #[zbus::interface(name = "org.kde.StatusNotifierWatcher")]
    impl DeniedWatcher {
        #[zbus(property)]
        fn is_status_notifier_host_registered(&self) -> fdo::Result<bool> {
            Err(fdo::Error::AccessDenied(self.0.into()))
        }
    }

    #[tokio::test]
    async fn propagates_errors_other_than_an_absent_watcher() {
        let bus = Bus::new();
        let host = bus.connect().await;
        host.object_server()
            .at(PATH, DeniedWatcher("denied by the test watcher"))
            .await
            .unwrap();
        host.request_name(WATCHER).await.unwrap();
        let client = bus.connect().await;
        let error = tokio::time::timeout(Duration::from_secs(5), wait_for_host(&client))
            .await
            .unwrap()
            .unwrap_err();
        assert!(matches!(
            fdo::Error::from(error),
            fdo::Error::AccessDenied(_)
        ));
    }
}
