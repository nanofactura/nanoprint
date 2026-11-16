//! Adds this installation's certificate authority to the current user's trust store, so browsers
//! accept `wss://localhost`. Uses each system's own tool, with an argument vector (never a shell),
//! and the system asks the user to confirm:
//!
//! - Windows: `certutil -user` on the user's Root store (security warning dialog).
//! - macOS: `security` on the login keychain (password prompt).
//! - Linux: NSS `certutil` on the shared store used by Chrome and Chromium. Since M146, Chromium
//!   prefers `${XDG_DATA_HOME:-~/.local/share}/pki/nssdb`, unless `~/.pki/nssdb` already exists;
//!   see <https://chromium.googlesource.com/chromium/src/+/main/docs/linux/cert_management.md>.
//!
//! Firefox keeps its own store and is not handled yet.

use std::{
    io,
    process::{Command, Output},
};

use crate::tls::{Certificate, Meta};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The system's tool is missing (NSS `certutil` is an optional package on Linux).
    #[error("the certificate tool is not installed")]
    MissingTool,
    /// The tool failed, or the user declined its confirmation; its own message.
    #[error("{0}")]
    Failed(String),
}

/// Whether browsers will trust the local certificate.
pub fn is_trusted(certificate: &Certificate) -> bool {
    imp::is_trusted(certificate)
}

/// Adds the authority. Blocks until the user answers the system's confirmation.
pub fn install(certificate: &Certificate) -> Result<(), Error> {
    imp::install(certificate)
}

/// Removes an authority (the current one, or one replaced by a renewal).
pub fn remove(meta: &Meta) -> Result<(), Error> {
    imp::remove(meta)
}

/// Whether the trust store still lists an authority.
pub fn contains(meta: &Meta) -> bool {
    imp::contains(meta)
}

fn run(command: &mut Command) -> Result<Output, Error> {
    #[cfg(windows)]
    {
        // No console window flashing over the desktop.
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    command.output().map_err(|error| match error.kind() {
        io::ErrorKind::NotFound => Error::MissingTool,
        _ => Error::Failed(error.to_string()),
    })
}

fn check(output: &Output) -> Result<(), Error> {
    if output.status.success() {
        Ok(())
    } else {
        let detail = String::from_utf8_lossy(&output.stderr);
        let detail = if detail.trim().is_empty() {
            String::from_utf8_lossy(&output.stdout)
        } else {
            detail
        };
        Err(Error::Failed(detail.trim().chars().take(300).collect()))
    }
}

#[cfg(windows)]
mod imp {
    use std::process::Command;

    use super::{Error, check, run};
    use crate::tls::{Certificate, Meta};

    pub fn is_trusted(certificate: &Certificate) -> bool {
        contains(&certificate.meta)
    }

    pub fn contains(meta: &Meta) -> bool {
        run(Command::new("certutil").args(["-user", "-store", "Root", &meta.ca_serial]))
            .is_ok_and(|output| output.status.success())
    }

    pub fn install(certificate: &Certificate) -> Result<(), Error> {
        let mut command = Command::new("certutil");
        command
            .args(["-user", "-addstore", "Root"])
            .arg(&certificate.ca_path);
        check(&run(&mut command)?)
    }

    pub fn remove(meta: &Meta) -> Result<(), Error> {
        check(&run(Command::new("certutil").args([
            "-user",
            "-delstore",
            "Root",
            &meta.ca_serial,
        ]))?)
    }
}

#[cfg(target_os = "macos")]
mod imp {
    use std::{path::PathBuf, process::Command};

    use super::{Error, check, run};
    use crate::tls::{Certificate, Meta};

    fn login_keychain() -> PathBuf {
        let home = std::env::var_os("HOME").unwrap_or_default();
        PathBuf::from(home).join("Library/Keychains/login.keychain-db")
    }

    pub fn is_trusted(certificate: &Certificate) -> bool {
        let mut command = Command::new("security");
        command
            .args(["verify-cert", "-p", "ssl", "-s", "localhost", "-c"])
            .arg(&certificate.cert_path);
        run(&mut command).is_ok_and(|output| output.status.success())
    }

    pub fn install(certificate: &Certificate) -> Result<(), Error> {
        let mut command = Command::new("security");
        command
            .args(["add-trusted-cert", "-r", "trustRoot", "-k"])
            .arg(login_keychain())
            .arg(&certificate.ca_path);
        check(&run(&mut command)?)
    }

    pub fn remove(meta: &Meta) -> Result<(), Error> {
        let mut command = Command::new("security");
        command
            .args(["delete-certificate", "-c", &meta.ca_name])
            .arg(login_keychain());
        check(&run(&mut command)?)
    }

    pub fn contains(meta: &Meta) -> bool {
        let mut command = Command::new("security");
        command
            .args(["find-certificate", "-c", &meta.ca_name])
            .arg(login_keychain());
        run(&mut command).is_ok_and(|output| output.status.success())
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
mod imp {
    use std::{
        ffi::OsString,
        path::{Path, PathBuf},
        process::Command,
    };

    use super::{Error, check, run};
    use crate::tls::{Certificate, Meta};

    fn nss_dirs(home: &Path, xdg_data_home: Option<&Path>) -> (PathBuf, PathBuf) {
        let legacy = home.join(".pki/nssdb");
        let data = xdg_data_home
            .filter(|path| path.is_absolute())
            .map_or_else(|| home.join(".local/share"), Path::to_path_buf);
        let modern = data.join("pki/nssdb");
        (legacy, modern)
    }

    fn nss_dir(home: &Path, xdg_data_home: Option<&Path>) -> PathBuf {
        let (legacy, modern) = nss_dirs(home, xdg_data_home);
        if legacy.is_dir() || !modern.is_dir() {
            legacy
        } else {
            modern
        }
    }

    fn paths() -> Result<(PathBuf, PathBuf, PathBuf), Error> {
        let home = std::env::var_os("HOME")
            .filter(|home| !home.is_empty())
            .ok_or_else(|| Error::Failed("HOME is not set".into()))?;
        let xdg_data_home = std::env::var_os("XDG_DATA_HOME");
        let home = Path::new(&home);
        let xdg_data_home = xdg_data_home.as_deref().map(Path::new);
        let (legacy, modern) = nss_dirs(home, xdg_data_home);
        Ok((nss_dir(home, xdg_data_home), legacy, modern))
    }

    fn nss_db(dir: &Path) -> OsString {
        let mut db = OsString::from("sql:");
        db.push(dir);
        db
    }

    fn contains_in(dir: &Path, meta: &Meta) -> Result<bool, Error> {
        let output = run(Command::new("certutil")
            .env("LC_ALL", "C")
            .arg("-d")
            .arg(nss_db(dir))
            .args(["-L", "-n", &meta.ca_name]))?;
        if output.status.success() {
            Ok(true)
        } else if String::from_utf8_lossy(&output.stderr).contains("Could not find cert:")
            && String::from_utf8_lossy(&output.stderr).contains("PR_FILE_NOT_FOUND_ERROR")
        {
            Ok(false)
        } else {
            check(&output)?;
            Ok(false)
        }
    }

    fn contains_or_log(dir: &Path, meta: &Meta) -> bool {
        if !dir.is_dir() {
            return false;
        }
        match contains_in(dir, meta) {
            Ok(found) => found,
            Err(error) => {
                log::warn!("could not check NSS database {}: {error}", dir.display());
                false
            }
        }
    }

    pub fn is_trusted(certificate: &Certificate) -> bool {
        match paths() {
            Ok((active, _, _)) => contains_or_log(&active, &certificate.meta),
            Err(error) => {
                log::warn!("could not locate NSS database: {error}");
                false
            }
        }
    }

    pub fn contains(meta: &Meta) -> bool {
        match paths() {
            Ok((_, legacy, modern)) => [legacy, modern]
                .iter()
                .any(|dir| contains_or_log(dir, meta)),
            Err(error) => {
                log::warn!("could not locate NSS database: {error}");
                false
            }
        }
    }

    pub fn install(certificate: &Certificate) -> Result<(), Error> {
        let (dir, _, _) = paths()?;
        install_in(&dir, certificate)
    }

    fn install_in(dir: &Path, certificate: &Certificate) -> Result<(), Error> {
        // Chrome creates the database on first run; create it if the user never opened Chrome.
        let db = nss_db(dir);
        if !dir.join("cert9.db").exists() {
            std::fs::create_dir_all(dir).map_err(|error| Error::Failed(error.to_string()))?;
            check(&run(Command::new("certutil")
                .arg("-d")
                .arg(&db)
                .args(["-N", "--empty-password"]))?)?;
        }
        let mut command = Command::new("certutil");
        command
            .arg("-d")
            .arg(db)
            .args(["-A", "-t", "C,,", "-n", &certificate.meta.ca_name, "-i"])
            .arg(&certificate.ca_path);
        check(&run(&mut command)?)
    }

    pub fn remove(meta: &Meta) -> Result<(), Error> {
        let (active, legacy, modern) = paths()?;
        remove_from(meta, &active, &[legacy, modern])
    }

    fn remove_from(meta: &Meta, active: &Path, dirs: &[PathBuf]) -> Result<(), Error> {
        let mut removed = false;
        for dir in dirs {
            if dir.is_dir() && contains_in(dir, meta)? {
                check(&run(Command::new("certutil")
                    .arg("-d")
                    .arg(nss_db(dir))
                    .args(["-D", "-n", &meta.ca_name]))?)?;
                removed = true;
            }
        }
        if removed {
            Ok(())
        } else {
            check(&run(Command::new("certutil")
                .arg("-d")
                .arg(nss_db(active))
                .args(["-D", "-n", &meta.ca_name]))?)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn selects_chromiums_existing_nss_database() {
            let home = std::env::temp_dir().join(format!("nanoprint-nss-{}", std::process::id()));
            let legacy = home.join(".pki/nssdb");
            let modern = home.join(".local/share/pki/nssdb");
            let custom_data = home.join("custom-data");
            let custom = custom_data.join("pki/nssdb");
            std::fs::create_dir_all(&home).unwrap();
            assert_eq!(nss_dir(&home, None), legacy);
            std::fs::create_dir_all(&modern).unwrap();
            assert_eq!(nss_dir(&home, None), modern);
            std::fs::create_dir_all(&custom).unwrap();
            assert_eq!(nss_dir(&home, Some(&custom_data)), custom);
            assert_eq!(nss_dir(&home, Some(Path::new("relative"))), modern);
            std::fs::create_dir_all(&legacy).unwrap();
            assert_eq!(nss_dir(&home, Some(&custom_data)), legacy);
            std::fs::remove_dir_all(home).unwrap();
        }

        #[test]
        fn removes_authority_from_both_databases() {
            Command::new("certutil")
                .arg("-H")
                .output()
                .expect("NSS integration tests require certutil (libnss3-tools or nss-tools)");
            let home =
                std::env::temp_dir().join(format!("nanoprint-nss-cert-{}", std::process::id()));
            let legacy = home.join("legacy");
            let modern = home.join("modern");
            let broken = home.join("broken");
            let (certificate, _) = crate::tls::load_or_create(&home.join("tls")).unwrap();
            install_in(&legacy, &certificate).unwrap();
            install_in(&modern, &certificate).unwrap();
            assert!(contains_in(&legacy, &certificate.meta).unwrap());
            assert!(contains_in(&modern, &certificate.meta).unwrap());
            std::fs::create_dir_all(&broken).unwrap();
            assert!(contains_in(&broken, &certificate.meta).is_err());
            assert!(remove_from(&certificate.meta, &legacy, &[broken]).is_err());
            remove_from(
                &certificate.meta,
                &legacy,
                &[legacy.clone(), modern.clone()],
            )
            .unwrap();
            assert!(!contains_in(&legacy, &certificate.meta).unwrap());
            assert!(!contains_in(&modern, &certificate.meta).unwrap());
            std::fs::remove_dir_all(home).unwrap();
        }
    }
}
