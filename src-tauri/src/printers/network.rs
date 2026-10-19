//! Network printers the user adds by address, printed to over a raw TCP connection (usually on
//! port 9100): the bytes go to the printer as they are, with no queue or driver in between. Web
//! pages can only reach the addresses the user added.

use std::{
    collections::HashMap,
    io,
    net::{IpAddr, SocketAddr},
    sync::{Arc, Mutex, PoisonError},
    time::Duration,
};

use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time::timeout,
};

use crate::server::Printer;

/// Longest printer name, in characters.
const MAX_NAME: usize = 64;
/// How long to wait for each address of the printer to accept the connection.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
/// How long the printer may take to take in a whole job (a slow printer with a small buffer reads
/// as it prints).
const WRITE_TIMEOUT: Duration = Duration::from_secs(60);
/// How long to wait, once the job is sent, for the printer to close the connection. Closing first
/// while the printer still has status bytes for us can reset the connection and lose the job's end.
const CLOSE_TIMEOUT: Duration = Duration::from_secs(2);

/// A printer the user added by address.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkPrinter {
    pub name: String,
    /// An IP address or a host name, normalized by [`check`].
    pub host: String,
    pub port: u16,
}

impl NetworkPrinter {
    /// As web pages and the Printers tab see it.
    pub fn printer(&self) -> Printer {
        Printer {
            name: self.name.clone(),
            driver: None,
            connection: Some(self.address()),
            default: false,
        }
    }

    /// `host:port`, with an IPv6 address in brackets.
    pub fn address(&self) -> String {
        if self.host.contains(':') {
            format!("[{}]:{}", self.host, self.port)
        } else {
            format!("{}:{}", self.host, self.port)
        }
    }

    /// Whether a page's `{host, port}` names this printer.
    pub fn is_at(&self, host: &str, port: u16) -> bool {
        port == self.port && normalize_host(host).as_deref() == Some(self.host.as_str())
    }
}

/// Why a printer cannot be added. Each is a `UiError` code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Invalid {
    Name,
    Host,
    Port,
}

impl Invalid {
    pub fn code(self) -> &'static str {
        match self {
            Self::Name => "network_name_invalid",
            Self::Host => "network_host_invalid",
            Self::Port => "network_port_invalid",
        }
    }
}

/// The printer to add, with its name trimmed and its host normalized.
pub fn check(name: &str, host: &str, port: u16) -> Result<NetworkPrinter, Invalid> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > MAX_NAME || name.chars().any(char::is_control) {
        return Err(Invalid::Name);
    }
    let host = normalize_host(host).ok_or(Invalid::Host)?;
    if port == 0 {
        return Err(Invalid::Port);
    }
    Ok(NetworkPrinter {
        name: name.to_owned(),
        host,
        port,
    })
}

/// An IP address in its canonical form (an IPv6 one may come in brackets), or a host name in
/// lowercase made of letters, digits, hyphens and dots. Anything else is refused, so a host can
/// never carry a path, a port or a URL.
fn normalize_host(host: &str) -> Option<String> {
    let host = host.trim();
    let bare = host
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
        .unwrap_or(host);
    if let Ok(ip) = bare.parse::<IpAddr>() {
        return Some(ip.to_string());
    }
    let name = host.strip_suffix('.').unwrap_or(host);
    let valid_label = |label: &str| {
        (1..=63).contains(&label.len())
            && label
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
            && !label.starts_with('-')
            && !label.ends_with('-')
    };
    (name.len() <= 253 && name.split('.').all(valid_label)).then(|| name.to_ascii_lowercase())
}

/// One connection at a time per printer: most network printers take a single client, and two jobs
/// at once would mix on paper or fail. Jobs to different printers do not wait for each other.
#[derive(Default)]
pub struct Links(Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>);

impl Links {
    /// Sends `data` to `printer` as one job, waiting for any job already going to it.
    pub async fn send(&self, printer: &NetworkPrinter, data: &[u8]) -> io::Result<()> {
        let address = printer.address();
        let link = Arc::clone(
            self.0
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .entry(address)
                .or_default(),
        );
        let _turn = link.lock().await;
        send(&printer.host, printer.port, data).await
    }
}

async fn send(host: &str, port: u16, data: &[u8]) -> io::Result<()> {
    let mut stream = connect(host, port).await?;
    timeout(WRITE_TIMEOUT, async {
        stream.write_all(data).await?;
        stream.shutdown().await
    })
    .await
    .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "the printer stopped taking data"))??;
    // Anything the printer answers (status bytes) is read and dropped until it closes. A printer
    // that keeps the connection open is fine: the job is already sent.
    let mut sink = [0_u8; 256];
    let _ = timeout(CLOSE_TIMEOUT, async {
        while stream.read(&mut sink).await? > 0 {}
        io::Result::Ok(())
    })
    .await;
    Ok(())
}

/// Connects to the first address of `host` that answers.
async fn connect(host: &str, port: u16) -> io::Result<TcpStream> {
    let addresses: Vec<SocketAddr> =
        timeout(CONNECT_TIMEOUT, tokio::net::lookup_host((host, port)))
            .await
            .map_err(|_| {
                io::Error::new(io::ErrorKind::TimedOut, "the name did not resolve in time")
            })??
            .collect();
    let mut last = io::Error::new(io::ErrorKind::NotFound, "the name has no address");
    for address in addresses {
        match timeout(CONNECT_TIMEOUT, TcpStream::connect(address)).await {
            Ok(Ok(stream)) => return Ok(stream),
            Ok(Err(error)) => last = error,
            Err(_) => {
                last = io::Error::new(io::ErrorKind::TimedOut, format!("{address} did not answer"));
            }
        }
    }
    Err(last)
}

#[cfg(test)]
mod tests {
    use tokio::net::TcpListener;

    use super::*;

    #[test]
    fn accepts_addresses_and_host_names() {
        let host = |host: &str| check("Kitchen", host, 9100).map(|printer| printer.host);
        assert_eq!(host(" 192.168.1.50 "), Ok("192.168.1.50".into()));
        assert_eq!(host("[FE80::1]"), Ok("fe80::1".into()));
        assert_eq!(host("fe80::1"), Ok("fe80::1".into()));
        assert_eq!(host("Printer-2.local."), Ok("printer-2.local".into()));
    }

    #[test]
    fn refuses_hosts_that_are_not_just_a_host() {
        for host in [
            "",
            "http://192.168.1.50",
            "192.168.1.50:9100",
            "printer/path",
            "user@printer",
            "-printer",
            "printer..local",
            "impresora-ñ",
            "a b",
        ] {
            assert_eq!(check("Kitchen", host, 9100), Err(Invalid::Host), "{host}");
        }
    }

    #[test]
    fn checks_name_and_port() {
        assert_eq!(check("  ", "10.0.0.2", 9100), Err(Invalid::Name));
        assert_eq!(
            check("POS\n", "10.0.0.2", 9100).map(|p| p.name),
            Ok("POS".into())
        );
        assert_eq!(check("PO\u{1b}S", "10.0.0.2", 9100), Err(Invalid::Name));
        assert_eq!(check(&"x".repeat(65), "10.0.0.2", 9100), Err(Invalid::Name));
        assert_eq!(check("POS", "10.0.0.2", 0), Err(Invalid::Port));
    }

    #[test]
    fn matches_the_address_a_page_names() {
        let printer = check("Bar", "fe80::1", 9100).unwrap();
        assert_eq!(printer.address(), "[fe80::1]:9100");
        assert!(printer.is_at("[FE80::1]", 9100));
        assert!(!printer.is_at("fe80::1", 9101));
        assert!(!printer.is_at("fe80::2", 9100));
    }

    #[tokio::test]
    async fn delivers_the_bytes_and_closes_the_connection() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let received = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut data = Vec::new();
            socket.read_to_end(&mut data).await.unwrap();
            data
        });
        let printer = check("Test", "127.0.0.1", port).unwrap();
        Links::default()
            .send(&printer, b"\x1b@ticket")
            .await
            .unwrap();
        assert_eq!(received.await.unwrap(), b"\x1b@ticket");
    }

    #[tokio::test]
    async fn reports_a_printer_that_is_not_listening() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let printer = check("Off", "127.0.0.1", port).unwrap();
        assert!(Links::default().send(&printer, b"x").await.is_err());
    }
}
