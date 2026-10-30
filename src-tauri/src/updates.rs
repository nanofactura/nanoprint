//! Automatic updates. A minute after it starts and every six hours, NanoPrint asks GitHub Releases
//! for a newer version (`latest.json`, see `plugins.updater` in `tauri.conf.json`), downloads it in
//! the background and verifies its signature against the public key built into the app; a bad
//! signature aborts the update. It installs the update once no job has printed for five minutes,
//! then starts again, quietly in the tray.
//!
//! The user can turn automatic updates off in Preferences: NanoPrint then connects to nothing, and
//! a check happens only when the user asks for one.
//!
//! If an update did not install (an antivirus blocked the installer, for example), the installer
//! starts the previous version again (`src-tauri/windows/hooks.nsh`), which waits six hours before
//! it tries again instead of a minute.

use std::{
    sync::{Mutex, MutexGuard, PoisonError},
    time::{Duration, Instant},
};

use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::{settings::SettingsStore, ui_error::UiError};

const FIRST_CHECK: Duration = Duration::from_secs(60);
const CHECK_EVERY: Duration = Duration::from_hours(6);
/// An update waits until no job has printed for this long, so it never cuts one off.
const IDLE_BEFORE_INSTALL: Duration = Duration::from_mins(5);
/// How often the background task wakes up to decide what to do.
const TICK: Duration = Duration::from_mins(1);

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
    /// The verified download, ready to install.
    pending: Mutex<Option<(Update, Vec<u8>)>>,
    /// One check at a time, whether automatic or asked for.
    checking: tokio::sync::Mutex<()>,
    /// When a job last printed (see [`Updates::printed`]).
    last_print: Mutex<Option<Instant>>,
}

impl Updates {
    fn new() -> Self {
        let status = unsupported().map_or(Status::Idle, |reason| Status::Unsupported { reason });
        Self {
            status: Mutex::new(status),
            pending: Mutex::default(),
            checking: tokio::sync::Mutex::new(()),
            last_print: Mutex::default(),
        }
    }

    fn status(&self) -> MutexGuard<'_, Status> {
        self.status.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn set(&self, status: Status) {
        *self.status() = status;
    }

    fn pending(&self) -> MutexGuard<'_, Option<(Update, Vec<u8>)>> {
        self.pending.lock().unwrap_or_else(PoisonError::into_inner)
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
    app.manage(Updates::new());
    let first_check = first_check(app);
    if unsupported().is_some() {
        return;
    }
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(first_check).await;
        let mut next_check = Instant::now();
        loop {
            if automatic(&app) {
                if Instant::now() >= next_check {
                    next_check = Instant::now() + CHECK_EVERY;
                    check(&app).await;
                }
                let updates = app.state::<Updates>();
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

/// When to check first: soon, unless the update this start follows did not install.
fn first_check(app: &AppHandle) -> Duration {
    let settings = app.state::<SettingsStore>();
    let Some(attempted) = settings.get().installing_update else {
        return FIRST_CHECK;
    };
    if let Err(error) = settings.update(|settings| settings.installing_update = None) {
        log::warn!("could not save the settings: {error}");
    }
    let running = app.package_info().version.to_string();
    if installed(&attempted, &running) {
        log::info!("updated to NanoPrint {running}");
        FIRST_CHECK
    } else {
        log::warn!(
            "NanoPrint {attempted} did not install; still on {running}, trying again in six hours"
        );
        CHECK_EVERY
    }
}

fn installed(attempted: &str, running: &str) -> bool {
    attempted == running
}

fn automatic(app: &AppHandle) -> bool {
    !app.state::<SettingsStore>().get().manual_updates
}

/// Checks for a newer version and downloads it. Leaves the outcome in the status.
async fn check(app: &AppHandle) {
    let updates = app.state::<Updates>();
    let _one_at_a_time = updates.checking.lock().await;
    if updates.pending().is_some() {
        return;
    }
    updates.set(Status::Checking);
    let found = match app.updater() {
        Ok(client) => client.check().await,
        Err(error) => Err(error),
    };
    match found {
        Ok(Some(update)) => {
            let version = update.version.clone();
            log::info!("NanoPrint {version} is available; downloading it");
            updates.set(Status::Downloading {
                version: version.clone(),
            });
            // The plugin verifies the signature before returning the bytes.
            match update.download(|_, _| {}, || {}).await {
                Ok(bytes) => {
                    *updates.pending() = Some((update, bytes));
                    updates.set(Status::Ready { version });
                }
                Err(error) => {
                    log::warn!("could not download NanoPrint {version}: {error}");
                    updates.set(Status::Failed {
                        message: error.to_string(),
                    });
                }
            }
        }
        Ok(None) => updates.set(Status::UpToDate {
            checked_at: time::OffsetDateTime::now_utc().unix_timestamp(),
        }),
        Err(error) => {
            log::warn!("could not check for updates: {error}");
            updates.set(Status::Failed {
                message: error.to_string(),
            });
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
    log::info!("installing NanoPrint {}", update.version);
    // The installer starts NanoPrint again with the arguments it has now; the next start stays
    // in the tray whatever they are.
    app.state::<SettingsStore>()
        .update(|settings| {
            settings.quiet_next_start = true;
            settings.installing_update = Some(update.version.clone());
        })
        .map_err(|error| UiError::with_detail("settings", &error))?;
    if let Err(error) = update.install(bytes) {
        log::warn!("could not install NanoPrint {}: {error}", update.version);
        updates.set(Status::Failed {
            message: error.to_string(),
        });
        return Err(UiError::with_detail("update_failed", &error));
    }
    app.restart();
}

/// What the Preferences tab shows. Mirrors `UpdateView` in `src/lib/agent.ts`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateView {
    status: Status,
    automatic: bool,
}

fn view(app: &AppHandle) -> UpdateView {
    UpdateView {
        status: app.state::<Updates>().status().clone(),
        automatic: automatic(app),
    }
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri passes command arguments by value.
pub fn get_update_status(app: AppHandle) -> UpdateView {
    view(&app)
}

/// Checks now, whether or not updates are automatic, and downloads what it finds.
#[tauri::command]
pub async fn check_for_updates(app: AppHandle) -> UpdateView {
    if unsupported().is_none() {
        check(&app).await;
    }
    view(&app)
}

/// Installs the downloaded update now, even while printing.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri passes command arguments by value.
pub fn install_update(app: AppHandle) -> Result<(), UiError> {
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
    fn reports_the_status_to_the_panel_in_camel_case() {
        let json = serde_json::to_string(&Status::UpToDate { checked_at: 1 }).unwrap();
        assert_eq!(json, r#"{"state":"upToDate","checkedAt":1}"#);
    }
}
