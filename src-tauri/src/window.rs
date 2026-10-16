//! Windows are created on demand and destroyed when closed, so no webview uses memory while
//! NanoPrint idles in the tray.

use std::sync::Arc;

use tauri::{
    AppHandle, Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent,
};

use crate::{
    permissions::{self, Permissions},
    ui_error::UiError,
};

const SETTINGS: &str = "settings";
pub const PERMISSION: &str = "permission";
pub const ONBOARDING: &str = "onboarding";
/// A static page; no capability names this window, so it cannot call any command.
const LICENSES: &str = "licenses";

/// Opens the third-party licenses page (generated for releases by `npm run notices`). Async on
/// purpose: synchronous commands run on the main thread, and a webview created there on Windows
/// never finishes loading.
#[tauri::command]
#[allow(clippy::needless_pass_by_value, clippy::unused_async)] // Tauri passes arguments by value.
pub async fn show_licenses(app: AppHandle) {
    let result = match app.get_webview_window(LICENSES) {
        Some(window) => focus(&window),
        None => WebviewWindowBuilder::new(
            &app,
            LICENSES,
            WebviewUrl::App("third-party-notices.html".into()),
        )
        .title("NanoPrint")
        .inner_size(640.0, 720.0)
        .center()
        .build()
        .map(|_| ()),
    };
    if let Err(error) = result {
        log::error!("could not open the licenses window: {error}");
    }
}

/// Shows the log folder in the system's file manager, so a user can attach the log to a bug
/// report. The folder is fixed: callers cannot choose what is opened.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri passes command arguments by value.
pub fn open_log_folder(app: AppHandle) -> Result<(), UiError> {
    let failed = |error: &dyn std::fmt::Display| UiError::with_detail("log_folder", error);
    let dir = app.path().app_log_dir().map_err(|error| failed(&error))?;
    std::fs::create_dir_all(&dir).map_err(|error| failed(&error))?;
    #[cfg(windows)]
    let program = "explorer";
    #[cfg(target_os = "macos")]
    let program = "open";
    #[cfg(all(unix, not(target_os = "macos")))]
    let program = "xdg-open";
    std::process::Command::new(program)
        .arg(dir)
        .spawn()
        .map(|_| ())
        .map_err(|error| failed(&error))
}

/// Opens the settings window, or brings it to the front if it is already open.
pub fn show_settings(app: &AppHandle) {
    let result = match app.get_webview_window(SETTINGS) {
        Some(window) => focus(&window),
        None => WebviewWindowBuilder::new(app, SETTINGS, WebviewUrl::App("index.html".into()))
            .title("NanoPrint")
            .inner_size(560.0, 640.0)
            .min_inner_size(480.0, 480.0)
            .center()
            .build()
            .map(|_| ()),
    };
    if let Err(error) = result {
        log::error!("could not open the settings window: {error}");
    }
}

/// Opens the permission prompt, or tells it to show the next request if it is already open.
pub fn show_permission(app: &AppHandle) {
    let result = match app.get_webview_window(PERMISSION) {
        Some(window) => window
            .emit(permissions::REQUEST_CHANGED, ())
            .and_then(|()| focus(&window)),
        None => {
            WebviewWindowBuilder::new(app, PERMISSION, WebviewUrl::App("permission.html".into()))
                .title("NanoPrint")
                .inner_size(460.0, 300.0)
                .resizable(false)
                .minimizable(false)
                .maximizable(false)
                .always_on_top(true)
                .center()
                .focused(true)
                .build()
                .map(|window| {
                    let app = app.clone();
                    window.on_window_event(move |event| match event {
                        // The user closed the prompt: the request on screen is blocked for this
                        // session, and the window stays for any request still waiting.
                        WindowEvent::CloseRequested { api, .. } => {
                            if app.state::<Arc<Permissions>>().dismiss_shown() {
                                api.prevent_close();
                                refresh_permission(&app);
                            }
                        }
                        // A request may have arrived while the window was going away; open a new
                        // one once this one is gone.
                        WindowEvent::Destroyed if app.state::<Arc<Permissions>>().has_pending() => {
                            let reopen = app.clone();
                            if let Err(error) =
                                app.run_on_main_thread(move || show_permission(&reopen))
                            {
                                log::error!("could not reopen the permission window: {error}");
                            }
                        }
                        _ => {}
                    });
                })
        }
    };
    if let Err(error) = result {
        log::error!("could not open the permission window: {error}");
    }
}

/// Tells the permission prompt to show the request now at the front.
pub fn refresh_permission(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(PERMISSION)
        && let Err(error) = window.emit(permissions::REQUEST_CHANGED, ())
    {
        log::warn!("could not update the permission window: {error}");
    }
}

/// Closes the permission prompt once nothing is waiting. `destroy` skips `CloseRequested`, which
/// is reserved for the user dismissing a request.
pub fn close_permission(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(PERMISSION)
        && let Err(error) = window.destroy()
    {
        log::warn!("could not close the permission window: {error}");
    }
}

/// Opens the welcome / secure connections window, or brings it to the front. `on_top` keeps it
/// above the browser, for the reminder shown while the user is trying to print.
pub fn show_onboarding(app: &AppHandle, on_top: bool) {
    let result = match app.get_webview_window(ONBOARDING) {
        Some(window) => focus(&window),
        None => {
            WebviewWindowBuilder::new(app, ONBOARDING, WebviewUrl::App("onboarding.html".into()))
                .title("NanoPrint")
                .inner_size(460.0, 480.0)
                .resizable(false)
                .minimizable(false)
                .maximizable(false)
                .always_on_top(on_top)
                .center()
                .focused(true)
                .build()
                .map(|_| ())
        }
    };
    if let Err(error) = result {
        log::error!("could not open the onboarding window: {error}");
    }
}

fn focus(window: &WebviewWindow) -> tauri::Result<()> {
    window.unminimize()?;
    window.show()?;
    window.set_focus()
}
