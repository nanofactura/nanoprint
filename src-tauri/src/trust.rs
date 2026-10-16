//! Adds this installation's certificate authority to the current user's trust store, so browsers
//! accept `wss://localhost`. Uses each system's own tool, with an argument vector (never a shell),
//! and the system asks the user to confirm:
//!
//! - Windows: `certutil -user` on the user's Root store (security warning dialog).
//! - macOS: `security` on the login keychain (password prompt).
//! - Linux: NSS `certutil` on `~/.pki/nssdb`, the store used by Chrome and Chromium.
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
    use std::process::Command;

    use super::{Error, check, run};
    use crate::tls::{Certificate, Meta};

    fn nss_db() -> String {
        let home = std::env::var("HOME").unwrap_or_default();
        format!("sql:{home}/.pki/nssdb")
    }

    pub fn is_trusted(certificate: &Certificate) -> bool {
        contains(&certificate.meta)
    }

    pub fn contains(meta: &Meta) -> bool {
        run(Command::new("certutil").args(["-d", &nss_db(), "-L", "-n", &meta.ca_name]))
            .is_ok_and(|output| output.status.success())
    }

    pub fn install(certificate: &Certificate) -> Result<(), Error> {
        // Chrome creates the database on first run; create it if the user never opened Chrome.
        let home = std::env::var("HOME").unwrap_or_default();
        let dir = std::path::Path::new(&home).join(".pki/nssdb");
        if !dir.join("cert9.db").exists() {
            std::fs::create_dir_all(&dir).map_err(|error| Error::Failed(error.to_string()))?;
            check(&run(Command::new("certutil").args([
                "-d",
                &nss_db(),
                "-N",
                "--empty-password",
            ]))?)?;
        }
        let mut command = Command::new("certutil");
        command
            .args([
                "-d",
                &nss_db(),
                "-A",
                "-t",
                "C,,",
                "-n",
                &certificate.meta.ca_name,
                "-i",
            ])
            .arg(&certificate.ca_path);
        check(&run(&mut command)?)
    }

    pub fn remove(meta: &Meta) -> Result<(), Error> {
        check(&run(Command::new("certutil").args([
            "-d",
            &nss_db(),
            "-D",
            "-n",
            &meta.ca_name,
        ]))?)
    }
}
