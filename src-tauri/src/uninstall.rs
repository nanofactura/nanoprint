//! `nanoprint --uninstall`: removes what NanoPrint keeps outside its own folders, which an
//! uninstaller cannot reach by deleting files:
//!
//! - this installation's certificate authorities, from the user's trust store;
//! - the key that signs permissions, from the credential store.
//!
//! The Windows uninstaller runs it when the user also deletes NanoPrint's data (keeping the data
//! keeps the certificate trusted for a reinstall). On macOS and Linux, run it before deleting the
//! app. It never starts the user interface or the print server.

use std::path::PathBuf;

use crate::{IDENTIFIER, keys, tls, trust};

/// The command-line flag.
pub const FLAG: &str = "--uninstall";

/// Removes the trust and credential store entries. Returns whether everything was removed.
pub fn run() -> bool {
    let Some(dir) = data_dir() else {
        return false;
    };
    let mut removed = true;
    for authority in tls::authorities(&dir.join("tls")) {
        if trust::contains(&authority) && trust::remove(&authority).is_err() {
            removed = false;
        }
    }
    keys::forget(&dir) && removed
}

/// Where Tauri keeps the app's local data (`app_local_data_dir`), without starting Tauri.
fn data_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    let base = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    #[cfg(target_os = "macos")]
    let base = std::env::var_os("HOME")
        .map(|home| PathBuf::from(home).join("Library/Application Support"));
    #[cfg(all(unix, not(target_os = "macos")))]
    let base = std::env::var_os("XDG_DATA_HOME")
        .filter(|dir| !dir.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")));
    base.map(|base| base.join(IDENTIFIER))
}
