//! NanoPrint desktop agent: the tray, the windows and the local print server.

mod agent;
mod i18n;
mod identity;
mod keys;
mod logs;
mod onboarding;
mod permissions;
mod printer_panel;
mod printers;
mod server;
mod servers;
mod settings;
mod store;
mod tls;
mod tray;
mod trust;
mod ui_error;
mod uninstall;
mod updates;
mod url_launch;
mod window;

use std::{path::PathBuf, sync::Arc, time::Duration};

// What web pages send is parsed in its own crate, which the fuzz targets test on their own.
use nanoprint_protocol as protocol;

use tauri::{AppHandle, Manager, RunEvent};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

use crate::{
    agent::{AppAgent, PrinterCache},
    onboarding::Onboarding,
    permissions::Permissions,
    settings::SettingsStore,
    store::Store,
};

/// The app identifier, as in `tauri.conf.json`. Names the data folders and the credential entry.
pub const IDENTIFIER: &str = "com.nanofactura.nanoprint";

/// Passed by the start-with-the-system entry: NanoPrint then starts quietly in the tray. Opened by
/// the user (the installer's last page, the Start menu, Finder), it shows a window instead.
const AUTOSTART_FLAG: &str = "--autostart";

pub fn run() {
    // Before anything else, and before the single-instance check would hand the flag to a
    // running NanoPrint instead.
    if std::env::args().any(|arg| arg == uninstall::FLAG) {
        std::process::exit(i32::from(!uninstall::run()));
    }

    let app = tauri::Builder::default()
        // Must be the first plugin: a second launch focuses the running instance instead. A page
        // starting NanoPrint with a link finds it running already: nothing to show then.
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            if !args.iter().any(|arg| url_launch::is_link(arg)) {
                window::show_settings(app);
            }
        }))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(logs::plugin())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![AUTOSTART_FLAG]),
        ))
        .invoke_handler(tauri::generate_handler![
            settings::get_settings,
            settings::set_language,
            settings::set_ports,
            settings::restart,
            servers::get_server_status,
            servers::install_certificate,
            servers::remove_certificate,
            permissions::list_sites,
            permissions::list_history,
            permissions::revoke_site,
            permissions::get_permission_request,
            permissions::answer_permission_request,
            printer_panel::list_printers,
            printer_panel::set_printer_shared,
            printer_panel::print_test_page,
            onboarding::get_onboarding_reason,
            updates::get_update_status,
            updates::check_for_updates,
            updates::install_update,
            updates::set_automatic_updates,
            window::show_licenses,
            window::open_log_folder,
        ])
        .setup(setup)
        .build(tauri::generate_context!())
        .expect("failed to build the NanoPrint application");

    app.run(|_app, event| {
        #[cfg(target_os = "macos")]
        url_launch::on_event(&event);
        // NanoPrint lives in the tray: closing the last window must not quit it. An explicit exit
        // (tray menu "Quit") carries an exit code and goes through.
        if let RunEvent::ExitRequested {
            api, code: None, ..
        } = event
        {
            api.prevent_exit();
        }
    });
}

/// Opens the stores, starts the print servers, the tray and the updater, and shows the first window
/// when the user opened NanoPrint.
fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // Tray-only app: no Dock icon on macOS.
    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);

    let settings = SettingsStore::load(app.handle())?;
    if settings.is_first_run() {
        settings.save()?;
        enable_autostart(app.handle());
    } else {
        refresh_autostart(app.handle());
    }
    // The start right after installing an update stays in the tray, whatever the arguments.
    let after_update = settings.get().quiet_next_start;
    if after_update {
        settings.update(|settings| settings.quiet_next_start = false)?;
    }
    app.manage(settings);

    // Permissions and history stay on this machine: never in a roaming profile.
    let data_dir = app.path().app_local_data_dir()?;
    let store = Arc::new(Store::open(&data_dir.join("nanoprint.db"), None)?);
    // The credential store may wait for the user to unlock it: ask off the main thread,
    // so the tray and the print servers start meanwhile.
    tauri::async_runtime::spawn({
        let store = Arc::clone(&store);
        let dir = data_dir.clone();
        async move {
            let key = tauri::async_runtime::spawn_blocking(move || keys::integrity_key(&dir)).await;
            match key {
                Ok(Some(key)) => store.set_key(key),
                _ => store.set_key(keys::wait_for_key().await),
            }
        }
    });
    let permissions = Arc::new(Permissions::new(Arc::clone(&store), app.handle().clone()));
    app.manage(Arc::clone(&store));
    app.manage(Arc::clone(&permissions));

    app.manage(Onboarding::default());
    // Shared by the print servers and the Printers tab.
    let printers = Arc::new(PrinterCache::default());
    app.manage(Arc::clone(&printers));
    let agent = Arc::new(AppAgent::new(
        Arc::clone(&store),
        permissions,
        printers,
        app.handle().clone(),
    ));
    updates::start(app.handle());
    servers::start(app.handle(), &agent);
    tauri::async_runtime::spawn(purge_daily(store, app.path().app_log_dir()?));

    tray::create(app.handle())?;
    url_launch::register(app.handle());
    // Started with the system, by a page's link or after an update: stay in the tray.
    let quiet = after_update
        || std::env::args().any(|arg| arg == AUTOSTART_FLAG)
        || url_launch::started_by_link();
    if !onboarding::welcome(app.handle()) && !quiet {
        url_launch::show_settings_unless_started_by_link(app.handle());
    }
    Ok(())
}

/// Starts NanoPrint with the system by default. The user can turn it off in the panel.
fn enable_autostart(app: &AppHandle) {
    // A development build would register the debug binary to start at login.
    if cfg!(debug_assertions) {
        return;
    }
    if let Err(error) = app.autolaunch().enable() {
        log::warn!("could not enable start with the system: {error}");
    }
}

/// Rewrites an existing start-with-the-system entry, so one written by an earlier version (or
/// pointing to an earlier install) gets the current path and [`AUTOSTART_FLAG`].
fn refresh_autostart(app: &AppHandle) {
    if cfg!(debug_assertions) || !app.autolaunch().is_enabled().unwrap_or(false) {
        return;
    }
    if let Err(error) = app.autolaunch().enable() {
        log::warn!("could not update start with the system: {error}");
    }
}

/// Deletes history rows and archived logs older than 30 days, at startup and once a day.
async fn purge_daily(store: Arc<Store>, log_dir: PathBuf) {
    let mut interval = tokio::time::interval(Duration::from_hours(24));
    loop {
        interval.tick().await;
        match store.purge_history() {
            Ok(0) => {}
            Ok(deleted) => log::info!("deleted {deleted} history entries older than 30 days"),
            Err(error) => log::warn!("could not purge history: {error}"),
        }
        logs::purge(&log_dir, store::RETENTION);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn identifier_matches_the_tauri_configuration() {
        let config: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        assert_eq!(config["identifier"], super::IDENTIFIER);
    }
}
