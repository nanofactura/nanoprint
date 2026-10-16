//! CUPS (macOS and Linux) through its command-line tools. Arguments are passed as an argument
//! vector, never through a shell, and printer names come from `lpstat` itself.

use std::{
    io::Write,
    process::{Command, Stdio},
};

use super::Error;
use crate::server::Printer;

pub fn list() -> Result<Vec<Printer>, Error> {
    let names = match lpstat("-e") {
        Ok(names) => names,
        // `lpstat` exits with an error when no printer is configured.
        Err(Error::Command(_)) => String::new(),
        Err(error) => return Err(error),
    };
    let default = lpstat("-d").ok().and_then(|output| {
        output
            .split_once(": ")
            .map(|(_, name)| name.trim().to_owned())
    });
    Ok(names
        .lines()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(|name| Printer {
            default: default.as_deref() == Some(name),
            name: name.to_owned(),
            driver: None,
            connection: None,
        })
        .collect())
}

pub fn print_raw(printer: &str, job_name: &str, data: &[u8]) -> Result<(), Error> {
    let mut child = Command::new("lp")
        .args(["-d", printer, "-o", "raw", "-t", job_name])
        .env("LC_ALL", "C")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| Error::Os {
            operation: "lp",
            source,
        })?;
    // Dropping stdin after the write closes the pipe, so `lp` sees the end of the job.
    let written = child
        .stdin
        .take()
        .map_or(Ok(()), |mut stdin| stdin.write_all(data));
    // Always wait: `lp` may have stopped reading because it refused the job, and its own message
    // explains why better than the broken pipe; waiting also reaps the process.
    let output = child.wait_with_output().map_err(|source| Error::Os {
        operation: "lp",
        source,
    })?;
    if !output.status.success() {
        return Err(Error::Command(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }
    written.map_err(|source| Error::Os {
        operation: "lp",
        source,
    })
}

fn lpstat(flag: &str) -> Result<String, Error> {
    let output = Command::new("lpstat")
        .arg(flag)
        .env("LC_ALL", "C")
        .output()
        .map_err(|source| Error::Os {
            operation: "lpstat",
            source,
        })?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(Error::Command(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ))
    }
}
