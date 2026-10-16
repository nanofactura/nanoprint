//! The diagnostic log: what helps diagnose a problem the user reports (Preferences → About opens
//! its folder). It stays on this computer and never holds message contents: print data, site
//! names and logos never reach it, because only NanoPrint's own messages and the libraries'
//! warnings are written.
//!
//! A new file starts with each session; the previous session's file is kept and anything older is
//! deleted. Each file is capped at 128 KB, and archived files go after 30 days like the history.

use std::{
    fs,
    path::Path,
    time::{Duration, SystemTime},
};

use log::LevelFilter;
use tauri::{Runtime, plugin::TauriPlugin};
use tauri_plugin_log::{Builder, FileOpenStrategy, RotationStrategy};

/// Libraries whose debug and trace output carries message contents (WebSocket frames, TLS
/// records) or only noise. They log warnings and errors only, in every build.
const QUIET: &[&str] = &[
    "tungstenite",
    "tokio_tungstenite",
    "rustls",
    "tokio_rustls",
    "keyring_core",
    "tao",
    "wry",
];
const MAX_FILE_SIZE: u128 = 128 * 1024;
/// Archived files start with this; the current one is `NanoPrint.log`.
const ARCHIVE_PREFIX: &str = "NanoPrint_";

pub fn plugin<R: Runtime>() -> TauriPlugin<R> {
    let level = if cfg!(debug_assertions) {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    let builder = Builder::new()
        .level(level)
        .max_file_size(MAX_FILE_SIZE)
        .file_open_strategy(FileOpenStrategy::Rotate)
        .rotation_strategy(RotationStrategy::KeepSome(1));
    QUIET
        .iter()
        .fold(builder, |builder, target| {
            builder.level_for(*target, LevelFilter::Warn)
        })
        .build()
}

/// Deletes archived log files last written more than `retention` ago.
pub fn purge(dir: &Path, retention: Duration) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let cutoff = SystemTime::now()
        .checked_sub(retention)
        .unwrap_or(SystemTime::UNIX_EPOCH);
    for entry in entries.flatten() {
        let path = entry.path();
        let archived = path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with(ARCHIVE_PREFIX))
            && path
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("log"));
        let old = entry
            .metadata()
            .and_then(|metadata| metadata.modified())
            .is_ok_and(|modified| modified < cutoff);
        if archived
            && old
            && let Err(error) = fs::remove_file(&path)
        {
            log::warn!("could not delete an old log file: {error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deletes_only_old_archived_logs() {
        let dir = std::env::temp_dir().join(format!("nanoprint-logs-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        for name in [
            "NanoPrint.log",
            "NanoPrint_2026-01-01_00-00-00.log",
            "other.log",
        ] {
            fs::write(dir.join(name), "x").unwrap();
        }
        purge(&dir, Duration::from_secs(3600));
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 3);
        // Everything counts as old with no retention; only the archive goes.
        purge(&dir, Duration::ZERO);
        let mut left: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect();
        left.sort();
        assert_eq!(left, ["NanoPrint.log", "other.log"]);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn websocket_and_tls_libraries_never_log_contents() {
        for target in ["tungstenite", "tokio_tungstenite", "rustls", "tokio_rustls"] {
            assert!(QUIET.contains(&target), "{target} must be quiet");
        }
    }
}
