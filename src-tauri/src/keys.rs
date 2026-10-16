//! The key that signs stored permissions. It lives in the operating system's credential store
//! (Windows Credential Manager, macOS Keychain, Linux Secret Service), so another program cannot
//! simply read it next to the database. On systems without a credential store it falls back to a
//! file only the current user can read.
//!
//! Once the credential store has held the key, NanoPrint never switches to the file: a store that
//! does not answer yet (a keyring still locked right after login) is waited for, because
//! decisions signed with another key would stop verifying when the store comes back.

use std::{
    fs,
    io::{self, Write},
    path::Path,
    sync::{Mutex, PoisonError},
    time::Duration,
};

use crate::store::Key;

const SERVICE: &str = crate::IDENTIFIER;
const ACCOUNT: &str = "permissions-integrity-key";
/// Present once the key has been kept in the credential store.
const MARKER_FILE: &str = "integrity.keyring";
/// The key itself, on systems without a credential store.
const KEY_FILE: &str = "integrity.key";
/// How often an unavailable credential store is asked again.
const RETRY_EVERY: Duration = Duration::from_secs(5);

/// Loads the signing key from `dir`'s installation, creating it on first use. `None` means the
/// credential store holds the key but cannot be reached now: use [`wait_for_key`]. Blocking, and
/// the store may wait for the user to unlock it: never call it on the main thread.
pub fn integrity_key(dir: &Path) -> Option<Key> {
    match credential_store_key() {
        Ok(key) => {
            if let Err(error) = mark(dir) {
                log::warn!("could not record where the signing key lives: {error}");
            }
            Some(key)
        }
        Err(error) if dir.join(MARKER_FILE).exists() => {
            log::warn!("credential store unavailable ({error}); waiting for it");
            None
        }
        Err(error) => {
            let path = dir.join(KEY_FILE);
            log::warn!(
                "no credential store ({error}); keeping the signing key in {}",
                path.display()
            );
            Some(file_key(&path).unwrap_or_else(|error| {
                // Decisions signed with this key will not verify after a restart, so sites are
                // asked again: safe, just less convenient.
                log::error!("could not persist the signing key: {error}");
                random_key()
            }))
        }
    }
}

/// Asks the credential store again until it hands the key over.
pub async fn wait_for_key() -> Key {
    loop {
        tokio::time::sleep(RETRY_EVERY).await;
        if let Ok(Ok(key)) = tauri::async_runtime::spawn_blocking(credential_store_key).await {
            log::info!("credential store available again");
            return key;
        }
    }
}

/// Deletes the key wherever it is kept (for `nanoprint --uninstall`). Returns whether nothing is
/// left behind.
pub fn forget(dir: &Path) -> bool {
    let from_store = set_up_store().is_ok_and(|()| {
        keyring_core::Entry::new(SERVICE, ACCOUNT).is_ok_and(|entry| {
            matches!(
                entry.delete_credential(),
                Ok(()) | Err(keyring_core::Error::NoEntry)
            )
        })
    });
    let from_disk =
        [MARKER_FILE, KEY_FILE]
            .iter()
            .all(|file| match fs::remove_file(dir.join(file)) {
                Err(error) => error.kind() == io::ErrorKind::NotFound,
                Ok(()) => true,
            });
    from_store && from_disk
}

/// Connects to the credential store once it succeeds. A failure is not remembered: right after
/// login the store (the Secret Service on D-Bus) may not be running yet, and a later attempt works.
fn set_up_store() -> Result<(), String> {
    static READY: Mutex<bool> = Mutex::new(false);
    let mut ready = READY.lock().unwrap_or_else(PoisonError::into_inner);
    if !*ready {
        set_default_store()?;
        *ready = true;
    }
    Ok(())
}

fn credential_store_key() -> Result<Key, String> {
    set_up_store()?;
    let entry = keyring_core::Entry::new(SERVICE, ACCOUNT).map_err(|e| e.to_string())?;
    match entry.get_secret() {
        Ok(secret) if secret.len() == 32 => {
            let mut key = [0; 32];
            key.copy_from_slice(&secret);
            Ok(key)
        }
        // Missing or unusable: start over. Previously signed decisions stop verifying.
        Ok(_) | Err(keyring_core::Error::NoEntry) => {
            let key = random_key();
            entry.set_secret(&key).map_err(|e| e.to_string())?;
            Ok(key)
        }
        Err(error) => Err(error.to_string()),
    }
}

fn set_default_store() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let store = windows_native_keyring_store::Store::new();
    #[cfg(target_os = "macos")]
    let store = apple_native_keyring_store::keychain::Store::new();
    #[cfg(all(unix, not(target_os = "macos")))]
    let store = zbus_secret_service_keyring_store::Store::new();
    keyring_core::set_default_store(store.map_err(|e| e.to_string())?);
    Ok(())
}

fn mark(dir: &Path) -> io::Result<()> {
    let marker = dir.join(MARKER_FILE);
    if marker.exists() {
        return Ok(());
    }
    fs::create_dir_all(dir)?;
    fs::write(marker, b"")
}

fn file_key(path: &Path) -> io::Result<Key> {
    if let Ok(bytes) = fs::read(path)
        && let Ok(key) = Key::try_from(bytes.as_slice())
    {
        return Ok(key);
    }
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let key = random_key();
    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    options.open(path)?.write_all(&key)?;
    Ok(key)
}

fn random_key() -> Key {
    let mut key = [0; 32];
    getrandom::fill(&mut key).expect("the operating system's random generator must work");
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_key_is_created_once_and_reused() {
        let dir = std::env::temp_dir().join(format!("nanoprint-key-test-{}", std::process::id()));
        let path = dir.join(KEY_FILE);
        let first = file_key(&path).unwrap();
        assert_eq!(file_key(&path).unwrap(), first);
        assert_ne!(first, [0; 32]);
        fs::remove_dir_all(dir).unwrap();
    }
}
