//! User preferences, persisted as JSON in the app config directory.

use std::{
    collections::BTreeMap,
    fs, io,
    path::PathBuf,
    sync::{Mutex, MutexGuard, PoisonError},
};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

use crate::{
    i18n::{self, Locale},
    printers::{self, network::NetworkPrinter},
    server::Printer,
    tray,
    ui_error::UiError,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LanguagePreference {
    #[default]
    System,
    En,
    Es,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub language: LanguagePreference,
    /// Whether the welcome window was shown.
    pub onboarded: bool,
    /// The printers the user offered to web pages (`true`) or kept from them (`false`), by name.
    /// Printers not listed follow the default (see [`Settings::shares`]).
    pub shared_printers: BTreeMap<String, bool>,
    /// Network printers the user added by address (Printers tab).
    pub network_printers: Vec<NetworkPrinter>,
    /// Fixed ports, or `None` for the first free one of the ports qz-tray.js clients look for.
    pub insecure_port: Option<u16>,
    pub secure_port: Option<u16>,
    /// Updates only when the user asks (Preferences); otherwise NanoPrint checks and installs them.
    pub manual_updates: bool,
    /// Set before installing an update, so the start that follows stays quietly in the tray.
    pub quiet_next_start: bool,
}

impl Settings {
    /// Whether web pages may see and use `printer`: the user's choice, or else every printer that
    /// can print raw tickets (see [`printers::limitation`]).
    pub fn shares(&self, printer: &Printer) -> bool {
        self.shared_printers
            .get(&printer.name)
            .copied()
            .unwrap_or_else(|| printers::limitation(printer).is_none())
    }
}

/// What the settings window receives. Mirrors `SettingsView` in `src/lib/settings.ts`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    language: LanguagePreference,
    resolved_locale: Locale,
    system_locale: Locale,
    insecure_port: Option<u16>,
    secure_port: Option<u16>,
}

impl From<&Settings> for SettingsView {
    fn from(settings: &Settings) -> Self {
        Self {
            language: settings.language,
            resolved_locale: i18n::resolve(settings.language),
            system_locale: i18n::system_locale(),
            insecure_port: settings.insecure_port,
            secure_port: settings.secure_port,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not find the configuration directory: {0}")]
    ConfigDir(#[from] tauri::Error),
    #[error("could not access the settings file: {0}")]
    Io(#[from] io::Error),
    #[error("could not encode the settings: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug)]
pub struct SettingsStore {
    path: PathBuf,
    settings: Mutex<Settings>,
    first_run: bool,
}

impl SettingsStore {
    pub fn load(app: &AppHandle) -> Result<Self, Error> {
        let path = app.path().app_config_dir()?.join("settings.json");
        let (settings, first_run) = match fs::read_to_string(&path) {
            Ok(json) => {
                let settings = serde_json::from_str(&json).unwrap_or_else(|err| {
                    log::warn!(
                        "ignoring unreadable settings file {}: {err}",
                        path.display()
                    );
                    Settings::default()
                });
                (settings, false)
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => (Settings::default(), true),
            Err(err) => return Err(err.into()),
        };
        Ok(Self {
            path,
            settings: Mutex::new(settings),
            first_run,
        })
    }

    /// True when no settings file existed at startup.
    pub fn is_first_run(&self) -> bool {
        self.first_run
    }

    pub fn get(&self) -> Settings {
        self.lock().clone()
    }

    /// Writes the current settings to disk.
    pub fn save(&self) -> Result<(), Error> {
        let settings = self.lock();
        self.write(&settings)
    }

    /// Applies `change` and persists the result. Memory is only updated if the write succeeds.
    pub fn update(&self, change: impl FnOnce(&mut Settings)) -> Result<Settings, Error> {
        let mut settings = self.lock();
        let mut next = settings.clone();
        change(&mut next);
        self.write(&next)?;
        *settings = next.clone();
        Ok(next)
    }

    /// Writes to a temporary file and renames it, so a crash never leaves a truncated file.
    fn write(&self, settings: &Settings) -> Result<(), Error> {
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir)?;
        }
        let tmp = self.path.with_extension("json.tmp");
        fs::write(&tmp, serde_json::to_vec_pretty(settings)?)?;
        fs::rename(&tmp, &self.path)?;
        Ok(())
    }

    fn lock(&self) -> MutexGuard<'_, Settings> {
        // `Settings` is only ever replaced as a whole, so a poisoned lock still holds valid data.
        self.settings.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri passes command arguments by value.
pub fn get_settings(store: State<'_, SettingsStore>) -> SettingsView {
    SettingsView::from(&store.get())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri passes command arguments by value.
pub fn set_language(
    app: AppHandle,
    store: State<'_, SettingsStore>,
    language: LanguagePreference,
) -> Result<SettingsView, UiError> {
    let settings = store
        .update(|settings| settings.language = language)
        .map_err(|error| UiError::with_detail("settings", &error))?;
    tray::refresh(&app);
    Ok(SettingsView::from(&settings))
}

/// Fixes the ports the print servers listen on (`None`: automatic). Takes effect when NanoPrint
/// restarts (see [`restart`]).
#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri passes command arguments by value.
pub fn set_ports(
    store: State<'_, SettingsStore>,
    insecure_port: Option<u16>,
    secure_port: Option<u16>,
) -> Result<SettingsView, UiError> {
    check_ports(insecure_port, secure_port)?;
    let settings = store
        .update(|settings| {
            settings.insecure_port = insecure_port;
            settings.secure_port = secure_port;
        })
        .map_err(|error| UiError::with_detail("settings", &error))?;
    Ok(SettingsView::from(&settings))
}

/// Starts NanoPrint again, to listen on newly chosen ports.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri passes command arguments by value.
pub fn restart(app: AppHandle) {
    app.restart();
}

/// Ports below 1024 are reserved by the systems, and the two servers cannot share one.
fn check_ports(insecure: Option<u16>, secure: Option<u16>) -> Result<(), UiError> {
    if [insecure, secure]
        .into_iter()
        .flatten()
        .any(|port| port < 1024)
    {
        return Err(UiError::new("port_invalid"));
    }
    if insecure.is_some() && insecure == secure {
        return Err(UiError::new("ports_equal"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_language_in_lowercase() {
        let json = serde_json::to_string(&Settings {
            language: LanguagePreference::Es,
            onboarded: true,
            ..Settings::default()
        })
        .unwrap();
        assert_eq!(
            json,
            r#"{"language":"es","onboarded":true,"sharedPrinters":{},"networkPrinters":[],"insecurePort":null,"securePort":null,"manualUpdates":false,"quietNextStart":false}"#
        );
    }

    #[test]
    fn offers_physical_printers_unless_the_user_chose_otherwise() {
        let printer = |name: &str, port: &str| Printer {
            name: name.to_owned(),
            driver: None,
            connection: Some(port.to_owned()),
            default: false,
        };
        let (pos, pdf) = (
            printer("POS-58", "USB002"),
            printer("Microsoft Print to PDF", "PORTPROMPT:"),
        );
        let mut settings = Settings::default();
        assert!(settings.shares(&pos));
        assert!(!settings.shares(&pdf));
        settings.shared_printers.insert(pos.name.clone(), false);
        settings.shared_printers.insert(pdf.name.clone(), true);
        assert!(!settings.shares(&pos));
        assert!(settings.shares(&pdf));
    }

    #[test]
    fn accepts_only_usable_and_distinct_ports() {
        assert_eq!(check_ports(None, None), Ok(()));
        assert_eq!(check_ports(Some(9182), Some(9181)), Ok(()));
        assert_eq!(
            check_ports(Some(80), None),
            Err(UiError::new("port_invalid"))
        );
        assert_eq!(
            check_ports(Some(9000), Some(9000)),
            Err(UiError::new("ports_equal"))
        );
    }

    #[test]
    fn missing_fields_fall_back_to_defaults() {
        let settings: Settings = serde_json::from_str("{}").unwrap();
        assert_eq!(settings, Settings::default());
    }
}
