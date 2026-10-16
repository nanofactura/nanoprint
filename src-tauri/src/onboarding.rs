//! The welcome window on first run, and the reminder shown when a browser refuses the local
//! certificate: HTTPS pages can only connect once this computer trusts it.

use std::sync::{
    Mutex, PoisonError,
    atomic::{AtomicBool, Ordering},
};

use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::{servers::Servers, settings::SettingsStore, trust, window};

/// Why the onboarding window is open.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Reason {
    /// First run: what NanoPrint does, and secure connections.
    #[default]
    Welcome,
    /// A browser just refused the certificate.
    CertificateRefused,
}

#[derive(Debug, Default)]
pub struct Onboarding {
    reason: Mutex<Reason>,
    /// The reminder is shown at most once per run, however many connections fail.
    reminded: AtomicBool,
}

/// Opens the welcome window the first time NanoPrint runs. Returns whether it did.
pub fn welcome(app: &AppHandle) -> bool {
    let settings = app.state::<SettingsStore>();
    if settings.get().onboarded {
        return false;
    }
    if let Err(error) = settings.update(|settings| settings.onboarded = true) {
        log::warn!("could not save the welcome as seen: {error}");
    }
    show(app, Reason::Welcome);
    true
}

/// A browser refused the local certificate. Offers to install it unless this computer already
/// trusts it (then the browser keeps its own list, which the panel mentions).
pub fn certificate_refused(app: &AppHandle) {
    if app
        .state::<Onboarding>()
        .reminded
        .swap(true, Ordering::SeqCst)
    {
        return;
    }
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let Some(certificate) = app.state::<Servers>().certificate() else {
            return;
        };
        let trusted = tauri::async_runtime::spawn_blocking(move || trust::is_trusted(&certificate))
            .await
            .unwrap_or(false);
        if trusted {
            log::info!("a browser refused the certificate although this computer trusts it");
        } else {
            show(&app, Reason::CertificateRefused);
        }
    });
}

/// The certificate was renewed while NanoPrint runs: its new authority is not trusted yet, so the
/// reminder may show again.
pub fn certificate_renewed(app: &AppHandle) {
    app.state::<Onboarding>()
        .reminded
        .store(false, Ordering::SeqCst);
}

fn show(app: &AppHandle, reason: Reason) {
    // An open window keeps its reason: the welcome already covers the reminder.
    if app.get_webview_window(window::ONBOARDING).is_none() {
        *app.state::<Onboarding>()
            .reason
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = reason;
    }
    window::show_onboarding(app, reason == Reason::CertificateRefused);
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri passes command arguments by value.
pub fn get_onboarding_reason(onboarding: State<'_, Onboarding>) -> Reason {
    *onboarding
        .reason
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}
