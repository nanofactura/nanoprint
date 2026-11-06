//! Updates. A minute after it starts and every six hours, NanoPrint asks GitHub Releases for a newer
//! version (`latest.json`, see `plugins.updater` in `tauri.conf.json`).
//!
//! With automatic updates (the default), it downloads the new version in the background and
//! verifies its signature against the public key built into the app; a bad signature aborts the
//! update. It installs the update once no job has printed for five minutes, then starts again,
//! quietly in the tray.
//!
//! With automatic updates off, it only lets the user know: a dot on the tray icon and a banner in
//! the settings window. It stops asking GitHub until the user installs that version, which
//! downloads and verifies it first, or turns automatic updates back on.
//!
//! If an update did not install (an antivirus blocked the installer, for example), the installer
//! starts the previous version again (`src-tauri/windows/hooks.nsh`). NanoPrint does not try that
//! version again by itself, which would only meet the antivirus again: it waits for the user, as
//! with automatic updates off, and the banner says it could not be installed. A newer version
//! installs by itself again.

use std::{
    sync::{Mutex, MutexGuard, PoisonError},
    time::{Duration, Instant},
};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::{settings::SettingsStore, tray, ui_error::UiError};

const FIRST_CHECK: Duration = Duration::from_secs(60);
const CHECK_EVERY: Duration = Duration::from_hours(6);
/// An update waits until no job has printed for this long, so it never cuts one off.
const IDLE_BEFORE_INSTALL: Duration = Duration::from_mins(5);
/// How often the background task wakes up to decide what to do.
const TICK: Duration = Duration::from_mins(1);
/// Sent to the windows with an [`UpdateView`] whenever the status changes. Mirrors
/// `UPDATE_STATUS_CHANGED` in `src/lib/agent.ts`.
pub const STATUS_CHANGED: &str = "update-status-changed";

/// What the Preferences tab shows. Mirrors `UpdateStatus` in `src/lib/agent.ts`.
#[derive(Debug, Clone, Serialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "state"
)]
pub enum Status {
    /// Development builds, and Linux packages that their package manager updates.
    Unsupported {
        reason: Unsupported,
    },
    Idle,
    Checking,
    /// Unix seconds of the last successful check.
    UpToDate {
        checked_at: i64,
    },
    /// Found but not downloaded: waits for the user to install it (see [`waiting`]).
    Available {
        version: String,
    },
    Downloading {
        version: String,
    },
    /// Downloaded and verified, waiting to be installed.
    Ready {
        version: String,
    },
    Failed {
        message: String,
    },
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Unsupported {
    Development,
    Package,
}

/// Why this installation cannot update itself, if it cannot.
fn unsupported() -> Option<Unsupported> {
    if cfg!(debug_assertions) {
        // A development build would download and run the release installer over itself.
        return Some(Unsupported::Development);
    }
    // Only the AppImage can replace itself on Linux; .deb and .rpm belong to the package manager.
    if cfg!(target_os = "linux") && std::env::var_os("APPIMAGE").is_none() {
        return Some(Unsupported::Package);
    }
    None
}

pub struct Updates {
    status: Mutex<Status>,
    /// A newer version that waits for the user, not downloaded yet.
    available: Mutex<Option<Update>>,
    /// The verified download, ready to install.
    pending: Mutex<Option<(Update, Vec<u8>)>>,
    /// The version that could not be installed (`Settings::failed_update`). Kept here too, so a
    /// failure to save the settings never makes NanoPrint try it again and again.
    failed: Mutex<Option<String>>,
    /// One check or download at a time, whether automatic or asked for.
    checking: tokio::sync::Mutex<()>,
    /// When a job last printed (see [`Updates::printed`]).
    last_print: Mutex<Option<Instant>>,
}

impl Updates {
    fn new() -> Self {
        let status = unsupported().map_or(Status::Idle, |reason| Status::Unsupported { reason });
        Self {
            status: Mutex::new(status),
            available: Mutex::default(),
            pending: Mutex::default(),
            failed: Mutex::default(),
            checking: tokio::sync::Mutex::new(()),
            last_print: Mutex::default(),
        }
    }

    fn status(&self) -> MutexGuard<'_, Status> {
        self.status.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn available(&self) -> MutexGuard<'_, Option<Update>> {
        self.available
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    fn pending(&self) -> MutexGuard<'_, Option<(Update, Vec<u8>)>> {
        self.pending.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn failed(&self) -> MutexGuard<'_, Option<String>> {
        self.failed.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Whether a newer version is known, downloaded or not.
    fn found(&self) -> bool {
        self.available().is_some() || self.pending().is_some()
    }

    /// Called by the print server for every job, so an update never interrupts printing.
    pub fn printed(&self) {
        *self
            .last_print
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = Some(Instant::now());
    }

    fn idle(&self) -> bool {
        self.last_print
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .is_none_or(|at| at.elapsed() >= IDLE_BEFORE_INSTALL)
    }
}

pub fn start(app: &AppHandle) {
    let updates = Updates::new();
    *updates.failed() = after_update(app);
    app.manage(updates);
    if unsupported().is_some() {
        return;
    }
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(FIRST_CHECK).await;
        let mut next_check = Instant::now();
        loop {
            let automatic = automatic(&app);
            let updates = app.state::<Updates>();
            if Instant::now() >= next_check && keeps_checking(automatic, updates.found()) {
                next_check = Instant::now() + CHECK_EVERY;
                check(&app).await;
            }
            // A version that could not be installed waits for the user even here.
            if automatic && waiting_for_user(&app).is_none() {
                // Found while automatic updates were off. A failure is logged and shown in the
                // status, and the next check finds the version again.
                if updates.available().is_some() {
                    let _ = download(&app).await;
                }
                if updates.pending().is_some()
                    && updates.idle()
                    && let Err(error) = install(&app)
                {
                    log::warn!("could not install the update: {error:?}");
                }
            }
            tokio::time::sleep(TICK).await;
        }
    });
}

/// Whether to ask GitHub again: always with automatic updates, otherwise until a version is found,
/// which then waits for the user.
fn keeps_checking(automatic: bool, found: bool) -> bool {
    automatic || !found
}

/// Records whether the update this start follows installed. Returns the version that could not be
/// installed, now or before, if any.
fn after_update(app: &AppHandle) -> Option<String> {
    let store = app.state::<SettingsStore>();
    let settings = store.get();
    let Some(attempted) = settings.installing_update else {
        return settings.failed_update;
    };
    let running = app.package_info().version.to_string();
    let failed = if installed(&attempted, &running) {
        log::info!("updated to NanoPrint {running}");
        None
    } else {
        log::warn!(
            "NanoPrint {attempted} did not install; still on {running}, waiting for the user to try again"
        );
        Some(attempted)
    };
    if let Err(error) = store.update(|settings| {
        settings.installing_update = None;
        settings.failed_update.clone_from(&failed);
    }) {
        log::warn!("could not save the settings: {error}");
    }
    failed
}

fn installed(attempted: &str, running: &str) -> bool {
    attempted == running
}

fn automatic(app: &AppHandle) -> bool {
    !app.state::<SettingsStore>().get().manual_updates
}

/// Sets the status, tells the windows and shows on the tray whether an update waits for the user.
fn publish(app: &AppHandle, status: Status) {
    *app.state::<Updates>().status() = status;
    changed(app);
}

/// Tells the windows and the tray about the current status and setting.
fn changed(app: &AppHandle) {
    if let Err(error) = app.emit(STATUS_CHANGED, view(app)) {
        log::warn!("could not send the update status: {error}");
    }
    tray::show_update(app, waiting_for_user(app).as_deref());
}

/// The version that waits for the user to install it, if any.
pub fn waiting_for_user(app: &AppHandle) -> Option<String> {
    let updates = app.try_state::<Updates>()?;
    let failed = updates.failed().clone();
    waiting(automatic(app), failed.as_deref(), &updates.status()).map(str::to_owned)
}

/// Automatic updates install by themselves: a version waits for the user when they are off, or
/// when it is the version that `failed` to install.
fn waiting<'a>(automatic: bool, failed: Option<&str>, status: &'a Status) -> Option<&'a str> {
    match status {
        Status::Available { version }
        | Status::Downloading { version }
        | Status::Ready { version }
            if !automatic || failed == Some(version.as_str()) =>
        {
            Some(version)
        }
        _ => None,
    }
}

/// Checks for a newer version. With automatic updates it downloads it, unless it could not be
/// installed before; otherwise it waits for the user. Leaves the outcome in the status.
async fn check(app: &AppHandle) {
    let updates = app.state::<Updates>();
    let _one_at_a_time = updates.checking.lock().await;
    let automatic = automatic(app);
    // A version that waits for the user stays. With automatic updates, a newer one replaces it.
    let waiting = updates.available().is_some();
    if updates.pending().is_some() || (waiting && !automatic) {
        return;
    }
    if !waiting {
        publish(app, Status::Checking);
    }
    let found = match app.updater() {
        Ok(client) => client.check().await,
        Err(error) => Err(error),
    };
    match found {
        Ok(Some(update))
            if automatic && updates.failed().as_deref() != Some(update.version.as_str()) =>
        {
            log::info!("NanoPrint {} is available; downloading it", update.version);
            *updates.available() = None;
            // A failure is logged and left in the status.
            let _ = fetch(app, update).await;
        }
        Ok(Some(update)) => {
            let version = update.version.clone();
            if !waiting {
                log::info!("NanoPrint {version} is available; waiting for the user to install it");
            }
            *updates.available() = Some(update);
            publish(app, Status::Available { version });
        }
        Ok(None) => {
            *updates.available() = None;
            publish(
                app,
                Status::UpToDate {
                    checked_at: time::OffsetDateTime::now_utc().unix_timestamp(),
                },
            );
        }
        Err(error) => {
            log::warn!("could not check for updates: {error}");
            // The banner keeps showing a version that waits for the user.
            if !waiting {
                publish(
                    app,
                    Status::Failed {
                        message: error.to_string(),
                    },
                );
            }
        }
    }
}

/// Downloads the version that waits for the user. Returns why it failed, if it did.
async fn download(app: &AppHandle) -> Result<(), String> {
    let updates = app.state::<Updates>();
    let _one_at_a_time = updates.checking.lock().await;
    let Some(update) = updates.available().take() else {
        return Ok(());
    };
    fetch(app, update).await
}

/// Downloads `update`; the plugin verifies its signature before returning the bytes. A failed
/// download forgets the version, so the next check finds it again.
async fn fetch(app: &AppHandle, update: Update) -> Result<(), String> {
    let version = update.version.clone();
    publish(
        app,
        Status::Downloading {
            version: version.clone(),
        },
    );
    match update.download(|_, _| {}, || {}).await {
        Ok(bytes) => {
            *app.state::<Updates>().pending() = Some((update, bytes));
            publish(app, Status::Ready { version });
            Ok(())
        }
        Err(error) => {
            log::warn!("could not download NanoPrint {version}: {error}");
            let message = error.to_string();
            publish(
                app,
                Status::Failed {
                    message: message.clone(),
                },
            );
            Err(message)
        }
    }
}

/// Installs the downloaded update and starts NanoPrint again. On Windows the installer takes over
/// and this process exits inside `install`.
fn install(app: &AppHandle) -> Result<(), UiError> {
    let updates = app.state::<Updates>();
    let Some((update, bytes)) = updates.pending().take() else {
        return Err(UiError::new("update_not_ready"));
    };
    let version = update.version.clone();
    log::info!("installing NanoPrint {version}");
    // The installer starts NanoPrint again with the arguments it has now; the next start stays
    // in the tray whatever they are, and tells whether the update installed (`after_update`).
    let settings = app.state::<SettingsStore>();
    settings
        .update(|settings| {
            settings.quiet_next_start = true;
            settings.installing_update = Some(version.clone());
        })
        .map_err(|error| UiError::with_detail("settings", &error))?;
    if let Err(error) = update.install(bytes) {
        // The installer did not start (an antivirus removing it, for example) and NanoPrint keeps
        // running. As after an installation that failed, the version waits for the user. If the
        // settings cannot be saved, `installing_update` still tells the next start.
        log::warn!("could not install NanoPrint {version}: {error}");
        *updates.failed() = Some(version.clone());
        if let Err(error) = settings.update(|settings| {
            settings.quiet_next_start = false;
            settings.installing_update = None;
            settings.failed_update = Some(version.clone());
        }) {
            log::warn!("could not save the settings: {error}");
        }
        *updates.available() = Some(update);
        publish(app, Status::Available { version });
        return Err(UiError::with_detail("update_failed", &error));
    }
    app.restart();
}

/// What the Preferences tab shows. Mirrors `UpdateView` in `src/lib/agent.ts`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateView {
    status: Status,
    automatic: bool,
    /// The version that could not be installed: it waits for the user even with automatic updates.
    failed: Option<String>,
}

fn view(app: &AppHandle) -> UpdateView {
    let updates = app.state::<Updates>();
    let failed = updates.failed().clone();
    let status = updates.status().clone();
    UpdateView {
        status,
        automatic: automatic(app),
        failed,
    }
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri passes command arguments by value.
pub fn get_update_status(app: AppHandle) -> UpdateView {
    view(&app)
}

/// Checks now, whether or not updates are automatic; downloads what it finds only if they are.
#[tauri::command]
pub async fn check_for_updates(app: AppHandle) -> UpdateView {
    if unsupported().is_none() {
        check(&app).await;
    }
    view(&app)
}

/// Installs the update now, even while printing, downloading it first if it was only found.
#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), UiError> {
    if let Err(message) = download(&app).await {
        return Err(UiError::with_detail("update_download_failed", &message));
    }
    install(&app)
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri passes command arguments by value.
pub fn set_automatic_updates(
    app: AppHandle,
    store: State<'_, SettingsStore>,
    automatic: bool,
) -> Result<UpdateView, UiError> {
    store
        .update(|settings| settings.manual_updates = !automatic)
        .map_err(|error| UiError::with_detail("settings", &error))?;
    changed(&app);
    Ok(view(&app))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waits_until_printing_has_been_quiet() {
        let updates = Updates::new();
        assert!(updates.idle(), "nothing has printed yet");
        updates.printed();
        assert!(!updates.idle(), "a job just printed");
        *updates.last_print.lock().unwrap() = Instant::now().checked_sub(IDLE_BEFORE_INSTALL);
        assert!(updates.idle(), "the last job is old enough");
    }

    #[test]
    fn tells_whether_the_update_installed() {
        assert!(installed("0.3.0", "0.3.0"));
        assert!(
            !installed("0.3.0", "0.2.1"),
            "still on the previous version"
        );
    }

    #[test]
    fn stops_checking_once_a_version_waits_for_the_user() {
        assert!(keeps_checking(true, false));
        assert!(
            keeps_checking(true, true),
            "automatic updates keep checking"
        );
        assert!(keeps_checking(false, false), "nothing found yet");
        assert!(!keeps_checking(false, true), "a version waits for the user");
    }

    #[test]
    fn a_version_waits_for_the_user_without_automatic_updates_or_after_failing() {
        let available = Status::Available {
            version: "0.4.0".to_owned(),
        };
        let downloading = Status::Downloading {
            version: "0.4.0".to_owned(),
        };
        let ready = Status::Ready {
            version: "0.4.0".to_owned(),
        };
        for status in [&available, &downloading, &ready] {
            assert_eq!(waiting(false, None, status), Some("0.4.0"), "{status:?}");
            assert_eq!(
                waiting(true, None, status),
                None,
                "installs by itself: {status:?}"
            );
            assert_eq!(
                waiting(true, Some("0.4.0"), status),
                Some("0.4.0"),
                "could not be installed: {status:?}"
            );
            assert_eq!(
                waiting(true, Some("0.3.9"), status),
                None,
                "a newer version than the one that failed installs by itself: {status:?}"
            );
        }
        assert_eq!(waiting(false, None, &Status::Checking), None);
        assert_eq!(
            waiting(false, None, &Status::UpToDate { checked_at: 1 }),
            None
        );
    }

    #[test]
    fn reports_the_status_to_the_panel_in_camel_case() {
        let json = serde_json::to_string(&Status::UpToDate { checked_at: 1 }).unwrap();
        assert_eq!(json, r#"{"state":"upToDate","checkedAt":1}"#);
    }
}
