//! Starting NanoPrint from a web page. A page that cannot reach NanoPrint can open
//! `nanoprint://start`: the browser asks the user, and the system starts NanoPrint, which then
//! takes the page's connection as usual, permission prompt included. A link can do nothing else.
//! Whatever follows the scheme is ignored, and NanoPrint starts quietly in the tray, as it does
//! with the system.

use tauri::AppHandle;
#[cfg(target_os = "macos")]
use tauri::RunEvent;

use crate::window;

const PREFIX: &str = "nanoprint:";

/// Whether a command-line argument is a `nanoprint:` link. Windows and Linux start NanoPrint with
/// the link as its only argument.
pub fn is_link(arg: &str) -> bool {
    arg.get(..PREFIX.len())
        .is_some_and(|start| start.eq_ignore_ascii_case(PREFIX))
}

/// Whether a link started this process (Windows and Linux; see [`on_event`] for macOS).
pub fn started_by_link() -> bool {
    std::env::args().skip(1).any(|arg| is_link(&arg))
}

/// The installers register the scheme. An `AppImage` is never installed, and a development build
/// should answer links too, so those register it themselves (in the user's profile).
#[cfg(any(windows, target_os = "linux"))]
pub fn register(app: &AppHandle) {
    use tauri_plugin_deep_link::DeepLinkExt;

    if (cfg!(debug_assertions) || std::env::var_os("APPIMAGE").is_some())
        && let Err(error) = app.deep_link().register_all()
    {
        log::warn!("could not register nanoprint: links: {error}");
    }
}

/// macOS registers the scheme from the app bundle.
#[cfg(not(any(windows, target_os = "linux")))]
pub fn register(_app: &AppHandle) {}

/// Shows the panel to a user who opened NanoPrint, and nothing when a link started it.
#[cfg(not(target_os = "macos"))]
pub fn show_settings_unless_started_by_link(app: &AppHandle) {
    window::show_settings(app);
}

/// macOS reports the link that started NanoPrint only after start-up, as an event: the panel
/// waits a moment for it.
#[cfg(target_os = "macos")]
pub fn show_settings_unless_started_by_link(app: &AppHandle) {
    use std::{sync::atomic::Ordering, time::Duration};

    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_millis(800)).await;
        if !macos::OPENED_BY_LINK.load(Ordering::Relaxed) {
            window::show_settings(&app);
        }
    });
}

/// Notes a link opened on macOS, where it arrives as an event rather than an argument.
#[cfg(target_os = "macos")]
pub fn on_event(event: &RunEvent) {
    if let RunEvent::Opened { urls } = event
        && urls.iter().any(|url| url.scheme() == "nanoprint")
    {
        macos::OPENED_BY_LINK.store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use std::sync::atomic::AtomicBool;

    pub static OPENED_BY_LINK: AtomicBool = AtomicBool::new(false);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_only_nanoprint_links() {
        for link in [
            "nanoprint://start",
            "NanoPrint://",
            "nanoprint:anything at all",
        ] {
            assert!(is_link(link), "{link}");
        }
        for other in [
            "--autostart",
            "--uninstall",
            "nanoprint",
            "nanoprints://",
            "ñanoprint:",
            "",
        ] {
            assert!(!is_link(other), "{other}");
        }
    }
}
