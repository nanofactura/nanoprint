//! The local print server: a WebSocket on loopback that speaks the qz-tray.js protocol.
//!
//! Every connection must come from a browser page on this computer: the `Host` header has to name
//! loopback (defeats DNS rebinding) and the `Origin` header must be an `http(s)` origin, which is
//! what NanoPrint asks the user about. Everything else is delegated to an [`Agent`], so the server
//! can be tested without windows or printers.

use std::{
    collections::HashMap,
    io,
    net::{Ipv4Addr, Ipv6Addr},
    sync::{Arc, Mutex, PoisonError},
    time::Duration,
};

use futures_util::{SinkExt, StreamExt, future::BoxFuture};
use serde::Serialize;
use serde_json::{Value, json};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpListener,
    sync::{Semaphore, broadcast},
    time::Instant,
};
use tokio_rustls::TlsAcceptor;
use tokio_tungstenite::{
    accept_hdr_async_with_config,
    tungstenite::{
        Message,
        handshake::server::{ErrorResponse, Request, Response},
        http::StatusCode,
        protocol::WebSocketConfig,
    },
};
use url::Url;

use crate::protocol::{self, Call, Incoming, PrintRequest};

/// Ports tried in order for `ws://` and `wss://`. Unmodified qz-tray.js clients look for a server
/// on these: the insecure list from `http://` pages, the secure one from `https://` pages.
pub const INSECURE_PORTS: [u16; 4] = [8182, 8283, 8384, 8485];
pub const SECURE_PORTS: [u16; 4] = [8181, 8282, 8383, 8484];

/// Largest accepted message. Raw receipts, even with raster logos, are far smaller.
const MAX_MESSAGE_SIZE: usize = 2 * 1024 * 1024;
/// Open connections in total, and per origin, so one site cannot take every slot.
const MAX_CONNECTIONS: usize = 256;
const MAX_CONNECTIONS_PER_ORIGIN: usize = 16;
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);
/// A connection must be allowed this soon after opening (qz-tray.js asks right away); the time
/// the user takes to answer the prompt does not count.
const AUTHORIZE_WITHIN: Duration = Duration::from_secs(if cfg!(test) { 1 } else { 30 });
/// Allowed connections that send nothing for this long are closed. qz-tray.js pings every 60 s.
const IDLE_TIMEOUT: Duration = Duration::from_secs(if cfg!(test) { 2 } else { 180 });
const MAX_ORIGIN_LEN: usize = 512;

/// The reply qz-tray.js clients expect when the user refuses a site.
pub const BLOCKED: &str = "Connection blocked by client";

/// A printer as reported to clients.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Printer {
    pub name: String,
    pub driver: Option<String>,
    pub connection: Option<String>,
    pub default: bool,
}

/// What the server needs from the rest of the application.
pub trait Agent: Send + Sync + 'static {
    /// Whether `origin` may print, asking the user when there is no stored decision.
    /// `certificate` is what the page sent in the setup message, if anything.
    fn authorize(&self, origin: &str, certificate: Option<String>) -> BoxFuture<'static, bool>;
    /// Origins whose permission was revoked; their open connections are closed.
    fn revocations(&self) -> broadcast::Receiver<String>;
    fn printers(&self) -> BoxFuture<'static, Result<Vec<Printer>, String>>;
    fn print(&self, origin: &str, request: PrintRequest) -> BoxFuture<'static, Result<(), String>>;
    /// A connection refused before the WebSocket handshake completed.
    fn rejected(&self, origin: Option<&str>, reason: Rejection);
    /// A browser refused the `wss://` certificate: this computer does not trust it yet.
    fn certificate_refused(&self);
}

/// Listening sockets on `127.0.0.1` and, when available, `::1`.
#[derive(Debug)]
pub struct Listening {
    pub port: u16,
    listeners: Vec<TcpListener>,
}

/// Binds the first port of `ports` that is free on loopback. Port 0 picks any free port.
pub async fn bind(ports: &[u16]) -> io::Result<Listening> {
    let mut last_error = io::Error::new(io::ErrorKind::InvalidInput, "no ports to try");
    for &port in ports {
        let v4 = match TcpListener::bind((Ipv4Addr::LOCALHOST, port)).await {
            Ok(listener) => listener,
            Err(error) => {
                last_error = error;
                continue;
            }
        };
        let port = v4.local_addr()?.port();
        match TcpListener::bind((Ipv6Addr::LOCALHOST, port)).await {
            Ok(v6) => {
                return Ok(Listening {
                    port,
                    listeners: vec![v4, v6],
                });
            }
            // Another program holds the port on IPv6 (QZ Tray listens on all addresses): browsers
            // resolving `localhost` to `::1` would reach it instead, so skip this port.
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::AddrInUse | io::ErrorKind::PermissionDenied
                ) =>
            {
                last_error = error;
            }
            // No IPv6 on this machine.
            Err(error) => {
                log::debug!("IPv6 loopback unavailable: {error}");
                return Ok(Listening {
                    port,
                    listeners: vec![v4],
                });
            }
        }
    }
    Err(last_error)
}

/// Accepts connections until the listeners fail. With `tls`, connections are `wss://`.
pub async fn serve(
    listening: Listening,
    agent: Arc<dyn Agent>,
    tls: Option<Arc<rustls::ServerConfig>>,
) {
    let limits = Limits {
        total: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
        origins: Arc::default(),
    };
    let tls = tls.map(TlsAcceptor::from);
    let tasks: Vec<_> = listening
        .listeners
        .into_iter()
        .map(|listener| {
            tokio::spawn(accept_loop(
                listener,
                listening.port,
                Arc::clone(&agent),
                limits.clone(),
                tls.clone(),
            ))
        })
        .collect();
    for task in tasks {
        if let Err(error) = task.await {
            log::error!("print server listener stopped: {error}");
        }
    }
}

/// Connection limits shared by the listeners of one server.
#[derive(Clone)]
struct Limits {
    total: Arc<Semaphore>,
    origins: Arc<Mutex<HashMap<String, usize>>>,
}

/// One open connection counted against its origin; released when dropped.
struct OriginSlot {
    origins: Arc<Mutex<HashMap<String, usize>>>,
    origin: String,
}

impl Limits {
    fn claim(&self, origin: &str) -> Option<OriginSlot> {
        let mut origins = self.origins.lock().unwrap_or_else(PoisonError::into_inner);
        let open = origins.entry(origin.to_owned()).or_default();
        if *open >= MAX_CONNECTIONS_PER_ORIGIN {
            return None;
        }
        *open += 1;
        Some(OriginSlot {
            origins: Arc::clone(&self.origins),
            origin: origin.to_owned(),
        })
    }
}

impl Drop for OriginSlot {
    fn drop(&mut self) {
        let mut origins = self.origins.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(open) = origins.get_mut(&self.origin) {
            *open -= 1;
            if *open == 0 {
                origins.remove(&self.origin);
            }
        }
    }
}

async fn accept_loop(
    listener: TcpListener,
    port: u16,
    agent: Arc<dyn Agent>,
    limits: Limits,
    tls: Option<TlsAcceptor>,
) {
    loop {
        let stream = match listener.accept().await {
            Ok((stream, _)) => stream,
            Err(error) => {
                log::warn!("could not accept a connection: {error}");
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            }
        };
        let Ok(permit) = Arc::clone(&limits.total).try_acquire_owned() else {
            log::debug!("too many open connections; refusing a new one");
            continue;
        };
        let agent = Arc::clone(&agent);
        let tls = tls.clone();
        let limits = limits.clone();
        tokio::spawn(async move {
            match tls {
                None => connection(stream, port, agent.as_ref(), &limits).await,
                Some(acceptor) => {
                    let accepted =
                        tokio::time::timeout(HANDSHAKE_TIMEOUT, acceptor.accept(stream)).await;
                    match accepted {
                        Ok(Ok(stream)) => connection(stream, port, agent.as_ref(), &limits).await,
                        Ok(Err(error)) => {
                            log::debug!("TLS handshake failed: {error}");
                            if refuses_certificate(&error) {
                                agent.certificate_refused();
                            }
                        }
                        Err(_) => log::debug!("TLS handshake timed out"),
                    }
                }
            }
            drop(permit);
        });
    }
}

/// Whether the client ended the TLS handshake because it does not trust the certificate.
/// Chromium-based browsers send `certificate_unknown`, Firefox `bad_certificate`, and OpenSSL or
/// rustls clients `unknown_ca`.
fn refuses_certificate(error: &io::Error) -> bool {
    use rustls::AlertDescription::{BadCertificate, CertificateUnknown, UnknownCA};
    matches!(
        error
            .get_ref()
            .and_then(|inner| inner.downcast_ref::<rustls::Error>()),
        Some(rustls::Error::AlertReceived(
            BadCertificate | CertificateUnknown | UnknownCA
        ))
    )
}

async fn connection<S>(stream: S, port: u16, agent: &dyn Agent, limits: &Limits)
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let mut origin = None;
    let mut rejection = None;
    #[allow(clippy::result_large_err)] // The error type is set by tungstenite's `Callback` trait.
    let callback = |request: &Request, response: Response| match check_request(request, port) {
        Ok(checked) => {
            origin = Some(checked);
            Ok(response)
        }
        Err(reason) => {
            let claimed = request
                .headers()
                .get("origin")
                .and_then(|value| value.to_str().ok())
                .map(|value| value.chars().take(MAX_ORIGIN_LEN).collect::<String>());
            rejection = Some((claimed, reason));
            let mut response = ErrorResponse::new(Some(reason.message().to_owned()));
            *response.status_mut() = StatusCode::FORBIDDEN;
            Err(response)
        }
    };
    let config = WebSocketConfig::default()
        .max_message_size(Some(MAX_MESSAGE_SIZE))
        .max_frame_size(Some(MAX_MESSAGE_SIZE));
    let handshake = tokio::time::timeout(
        HANDSHAKE_TIMEOUT,
        accept_hdr_async_with_config(stream, callback, Some(config)),
    )
    .await;
    if let Some((claimed, reason)) = rejection {
        agent.rejected(claimed.as_deref(), reason);
        return;
    }
    let (Ok(Ok(mut websocket)), Some(origin)) = (handshake, origin) else {
        return;
    };
    let Some(_slot) = limits.claim(&origin) else {
        log::debug!("too many open connections from {origin}; refusing a new one");
        let _ = websocket.close(None).await;
        return;
    };

    let (mut sink, mut stream) = websocket.split();
    let mut revocations = agent.revocations();
    let mut session = Session {
        origin,
        authorized: false,
    };
    let opened = Instant::now();
    let mut last_message = opened;
    loop {
        let deadline = if session.authorized {
            last_message + IDLE_TIMEOUT
        } else {
            opened + AUTHORIZE_WITHIN
        };
        // `biased`: a revocation that is already waiting wins over the next client frame.
        let reply = tokio::select! {
            biased;
            revoked = revocations.recv() => match revoked {
                Ok(revoked) if revoked != session.origin => Reply::Nothing,
                // Revoked, or revocations were missed: close and let the client reconnect, which
                // checks the permission again.
                Ok(_) | Err(_) => Reply::Close,
            },
            message = stream.next() => {
                last_message = Instant::now();
                match message {
                    Some(Ok(Message::Text(text))) => session.handle(text.as_str(), agent).await,
                    // Binary frames are not part of the protocol.
                    Some(Ok(Message::Binary(_) | Message::Close(_)) | Err(_)) | None => {
                        Reply::Close
                    }
                    Some(Ok(_)) => Reply::Nothing,
                }
            },
            () = tokio::time::sleep_until(deadline) => Reply::Close,
        };
        match reply {
            Reply::Nothing => {}
            Reply::Send(text) => {
                if sink.send(Message::text(text)).await.is_err() {
                    break;
                }
            }
            Reply::SendAndClose(text) => {
                let _ = sink.send(Message::text(text)).await;
                break;
            }
            Reply::Close => break,
        }
    }
    let _ = sink.close().await;
}

enum Reply {
    Nothing,
    Send(String),
    SendAndClose(String),
    Close,
}

struct Session {
    origin: String,
    authorized: bool,
}

impl Session {
    async fn handle(&mut self, text: &str, agent: &dyn Agent) -> Reply {
        let (uid, call) = match protocol::parse(text) {
            Ok(Incoming::Ignored) => return Reply::Nothing,
            Ok(Incoming::Setup { uid, certificate }) => {
                return if self.authorize(agent, certificate).await {
                    Reply::Send(protocol::result(&uid, &Value::Null))
                } else {
                    Reply::SendAndClose(protocol::error(&uid, BLOCKED))
                };
            }
            // The version is public: clients ask for it before connecting.
            Ok(Incoming::Call {
                uid,
                call: Call::GetVersion { details },
            }) => return Reply::Send(protocol::result(&uid, &version(details))),
            Ok(Incoming::Call { uid, call }) => (uid, Ok(call)),
            Err(protocol::Error::InvalidCall { uid, message }) => (uid, Err(message)),
            Err(protocol::Error::Malformed) => return Reply::Close,
        };
        // Some clients skip the setup message; every other call needs permission anyway.
        if !self.authorize(agent, None).await {
            return Reply::SendAndClose(protocol::error(&uid, BLOCKED));
        }
        let outcome = match call {
            Ok(call) => self.call(call, agent).await,
            Err(message) => Err(message),
        };
        Reply::Send(match outcome {
            Ok(value) => protocol::result(&uid, &value),
            Err(message) => protocol::error(&uid, &message),
        })
    }

    async fn authorize(&mut self, agent: &dyn Agent, certificate: Option<String>) -> bool {
        if !self.authorized {
            self.authorized = agent.authorize(&self.origin, certificate).await;
        }
        self.authorized
    }

    async fn call(&self, call: Call, agent: &dyn Agent) -> Result<Value, String> {
        match call {
            Call::FindPrinters { query } => {
                let printers = agent.printers().await?;
                match query {
                    None => Ok(json!(
                        printers.iter().map(|p| p.name.as_str()).collect::<Vec<_>>()
                    )),
                    Some(query) => find(&printers, &query)
                        .map(|printer| json!(printer.name))
                        .ok_or_else(|| "Specified printer could not be found.".to_owned()),
                }
            }
            Call::DefaultPrinter => Ok(json!(
                agent
                    .printers()
                    .await?
                    .into_iter()
                    .find(|printer| printer.default)
                    .map(|printer| printer.name)
            )),
            Call::PrinterDetails => Ok(json!(agent.printers().await?)),
            Call::Print(request) => agent
                .print(&self.origin, request)
                .await
                .map(|()| Value::Null),
            Call::Unsupported(name) => Err(format!("'{name}' is not supported by NanoPrint")),
            Call::GetVersion { details } => Ok(version(details)),
        }
    }
}

/// Matches like QZ Tray: case-insensitive, an exact name first, then the first partial match.
fn find<'a>(printers: &'a [Printer], query: &str) -> Option<&'a Printer> {
    let query = query.to_lowercase();
    printers
        .iter()
        .find(|printer| printer.name.to_lowercase() == query)
        .or_else(|| {
            printers
                .iter()
                .find(|printer| printer.name.to_lowercase().contains(&query))
        })
}

fn version(details: bool) -> Value {
    if details {
        json!({
            "title": "NanoPrint",
            "version": env!("CARGO_PKG_VERSION"),
            "protocol": protocol::PROTOCOL_VERSION,
            "url": env!("CARGO_PKG_REPOSITORY"),
        })
    } else {
        json!(protocol::PROTOCOL_VERSION)
    }
}

/// Why a connection was refused before the WebSocket handshake.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rejection {
    MissingHost,
    ForeignHost,
    MissingOrigin,
    InvalidOrigin,
}

impl Rejection {
    /// Stable code stored in the history; the panel translates it.
    pub fn code(self) -> &'static str {
        match self {
            Self::MissingHost => "missing_host",
            Self::ForeignHost => "foreign_host",
            Self::MissingOrigin => "missing_origin",
            Self::InvalidOrigin => "invalid_origin",
        }
    }

    /// Body of the HTTP 403 reply, for developers.
    fn message(self) -> &'static str {
        match self {
            Self::MissingHost => "missing Host header",
            Self::ForeignHost => "Host must be this computer",
            Self::MissingOrigin => "missing Origin header",
            Self::InvalidOrigin => "invalid Origin header",
        }
    }
}

/// Validates the upgrade request and returns the normalized origin.
fn check_request(request: &Request, port: u16) -> Result<String, Rejection> {
    let header = |name| {
        request
            .headers()
            .get(name)
            .and_then(|value| value.to_str().ok())
    };
    let host = header("host").ok_or(Rejection::MissingHost)?;
    if !is_loopback_host(host, port) {
        return Err(Rejection::ForeignHost);
    }
    let origin = header("origin").ok_or(Rejection::MissingOrigin)?;
    normalize_origin(origin).ok_or(Rejection::InvalidOrigin)
}

fn is_loopback_host(host: &str, port: u16) -> bool {
    let host = host.to_ascii_lowercase();
    ["localhost", "127.0.0.1", "[::1]"]
        .iter()
        .any(|name| host == format!("{name}:{port}"))
}

/// Returns the `scheme://host[:port]` form of an `http(s)` origin, with international domain
/// names in punycode so look-alike characters are visible. Opaque and other origins are refused.
pub fn normalize_origin(value: &str) -> Option<String> {
    if value.len() > MAX_ORIGIN_LEN {
        return None;
    }
    let url = Url::parse(value).ok()?;
    let plain = matches!(url.scheme(), "http" | "https")
        && url.username().is_empty()
        && url.password().is_none()
        && url.path() == "/"
        && url.query().is_none()
        && url.fragment().is_none();
    let origin = url.origin();
    (plain && origin.is_tuple()).then(|| origin.ascii_serialization())
}

#[cfg(test)]
mod tests;
