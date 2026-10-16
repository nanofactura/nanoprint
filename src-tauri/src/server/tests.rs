use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

use futures_util::{SinkExt, StreamExt, future::BoxFuture};
use serde_json::{Value, json};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
    sync::broadcast,
};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{self, Message, client::IntoClientRequest, http::HeaderValue},
};

use super::*;

const ORIGIN: &str = "https://shop.example";

struct FakeAgent {
    allow: bool,
    revoked: broadcast::Sender<String>,
    printed: Mutex<Vec<(String, PrintRequest)>>,
    rejected: Mutex<Vec<String>>,
    certificates: Mutex<Vec<Option<String>>>,
    certificate_refusals: AtomicUsize,
}

impl Agent for FakeAgent {
    fn authorize(&self, _origin: &str, certificate: Option<String>) -> BoxFuture<'static, bool> {
        self.certificates.lock().unwrap().push(certificate);
        let allow = self.allow;
        Box::pin(async move { allow })
    }

    fn revocations(&self) -> broadcast::Receiver<String> {
        self.revoked.subscribe()
    }

    fn printers(&self) -> BoxFuture<'static, Result<Vec<Printer>, String>> {
        let printer = |name: &str, default| Printer {
            name: name.into(),
            driver: Some("Generic".into()),
            connection: Some("USB002".into()),
            default,
        };
        let printers = vec![printer("Office Laser", false), printer("POS-58", true)];
        Box::pin(async move { Ok(printers) })
    }

    fn print(&self, origin: &str, request: PrintRequest) -> BoxFuture<'static, Result<(), String>> {
        self.printed
            .lock()
            .unwrap()
            .push((origin.to_owned(), request));
        Box::pin(async { Ok(()) })
    }

    fn rejected(&self, origin: Option<&str>, reason: Rejection) {
        self.rejected
            .lock()
            .unwrap()
            .push(format!("{origin:?}: {}", reason.code()));
    }

    fn certificate_refused(&self) {
        self.certificate_refusals.fetch_add(1, Ordering::SeqCst);
    }
}

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

async fn start(allow: bool) -> (u16, Arc<FakeAgent>) {
    start_with(allow, None).await
}

async fn start_with(allow: bool, tls: Option<Arc<rustls::ServerConfig>>) -> (u16, Arc<FakeAgent>) {
    let agent = Arc::new(FakeAgent {
        allow,
        revoked: broadcast::channel(4).0,
        printed: Mutex::default(),
        rejected: Mutex::default(),
        certificates: Mutex::default(),
        certificate_refusals: AtomicUsize::new(0),
    });
    let listening = bind(&[0]).await.unwrap();
    let port = listening.port;
    tokio::spawn(serve(listening, Arc::clone(&agent) as Arc<dyn Agent>, tls));
    (port, agent)
}

async fn connect(
    port: u16,
    origin: Option<&str>,
    host: Option<&str>,
) -> Result<Socket, tungstenite::Error> {
    let mut request = format!("ws://127.0.0.1:{port}/")
        .into_client_request()
        .unwrap();
    if let Some(origin) = origin {
        request
            .headers_mut()
            .insert("origin", HeaderValue::from_str(origin).unwrap());
    }
    if let Some(host) = host {
        request
            .headers_mut()
            .insert("host", HeaderValue::from_str(host).unwrap());
    }
    connect_async(request).await.map(|(socket, _)| socket)
}

async fn send<S>(socket: &mut WebSocketStream<S>, message: &Value) -> Option<Value>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    socket
        .send(Message::text(message.to_string()))
        .await
        .unwrap();
    next(socket).await
}

/// The next JSON reply, or `None` once the server closed the connection.
async fn next<S>(socket: &mut WebSocketStream<S>) -> Option<Value>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    loop {
        match socket.next().await {
            Some(Ok(Message::Text(text))) => {
                return Some(serde_json::from_str(text.as_str()).unwrap());
            }
            Some(Ok(Message::Close(_)) | Err(_)) | None => return None,
            Some(Ok(_)) => {}
        }
    }
}

fn assert_forbidden(result: Result<Socket, tungstenite::Error>) {
    match result {
        Err(tungstenite::Error::Http(response)) => assert_eq!(response.status(), 403),
        Err(other) => panic!("expected HTTP 403, got {other}"),
        Ok(_) => panic!("expected HTTP 403, got a connection"),
    }
}

#[tokio::test]
async fn refuses_pages_without_an_origin() {
    let (port, agent) = start(true).await;
    assert_forbidden(connect(port, None, None).await);
    assert_eq!(agent.rejected.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn refuses_hosts_other_than_loopback() {
    // DNS rebinding: a page on evil.example resolves its own name to 127.0.0.1.
    let (port, _) = start(true).await;
    let host = format!("evil.example:{port}");
    assert_forbidden(connect(port, Some("http://evil.example"), Some(&host)).await);
    assert_forbidden(connect(port, Some(ORIGIN), Some("localhost:1")).await);
}

#[tokio::test]
async fn refuses_origins_that_are_not_web_pages() {
    let (port, _) = start(true).await;
    for origin in [
        "null",
        "file://",
        "chrome-extension://abcdef",
        "https://user@shop.example",
    ] {
        assert_forbidden(connect(port, Some(origin), None).await);
    }
}

#[tokio::test]
async fn serves_an_allowed_site() {
    let (port, agent) = start(true).await;
    let mut socket = connect(port, Some(ORIGIN), None).await.unwrap();

    let version = send(&mut socket, &json!({"call": "getVersion", "uid": "v1"})).await;
    assert_eq!(version, Some(json!({"uid": "v1", "result": "2.3.0"})));
    let setup = send(&mut socket, &json!({"certificate": null, "uid": "s1"})).await;
    assert_eq!(setup, Some(json!({"uid": "s1", "result": null})));

    let all = send(
        &mut socket,
        &json!({"call": "printers.find", "params": {}, "uid": "f1"}),
    )
    .await;
    assert_eq!(all.unwrap()["result"], json!(["Office Laser", "POS-58"]));
    let one = send(
        &mut socket,
        &json!({"call": "printers.find", "params": {"query": "pos"}, "uid": "f2"}),
    )
    .await;
    assert_eq!(one.unwrap()["result"], json!("POS-58"));
    let missing = send(
        &mut socket,
        &json!({"call": "printers.find", "params": {"query": "zebra"}, "uid": "f3"}),
    )
    .await;
    assert_eq!(
        missing.unwrap()["error"],
        json!("Specified printer could not be found.")
    );
    let default = send(
        &mut socket,
        &json!({"call": "printers.getDefault", "params": null, "uid": "d1"}),
    )
    .await;
    assert_eq!(default.unwrap()["result"], json!("POS-58"));
    let details = send(
        &mut socket,
        &json!({"call": "printers.detail", "uid": "d2"}),
    )
    .await;
    assert_eq!(details.unwrap()["result"][1]["connection"], json!("USB002"));

    let print = json!({
        "call": "print",
        "params": {"printer": {"name": "POS-58"}, "options": {"jobName": "Ticket"}, "data": ["\u{1b}@"]},
        "uid": "p1",
    });
    assert_eq!(
        send(&mut socket, &print).await,
        Some(json!({"uid": "p1", "result": null}))
    );
    {
        let printed = agent.printed.lock().unwrap();
        assert_eq!(printed[0].0, ORIGIN);
        assert_eq!(printed[0].1.data, b"\x1b@");
    }

    let unsupported = send(
        &mut socket,
        &json!({"call": "file.read", "params": {"path": "C:/x"}, "uid": "u1"}),
    )
    .await;
    assert_eq!(
        unsupported,
        Some(json!({"uid": "u1", "error": "'file.read' is not supported by NanoPrint"}))
    );
}

#[tokio::test]
async fn blocks_and_disconnects_a_refused_site() {
    let (port, agent) = start(false).await;
    let mut socket = connect(port, Some(ORIGIN), None).await.unwrap();
    let setup = send(&mut socket, &json!({"certificate": null, "uid": "s1"})).await;
    assert_eq!(setup, Some(json!({"uid": "s1", "error": BLOCKED})));
    assert_eq!(next(&mut socket).await, None);
    assert!(agent.printed.lock().unwrap().is_empty());
}

#[tokio::test]
async fn every_call_needs_permission() {
    // A client that skips the setup message is asked about too.
    let (port, _) = start(false).await;
    let mut socket = connect(port, Some(ORIGIN), None).await.unwrap();
    let find = send(
        &mut socket,
        &json!({"call": "printers.find", "params": {}, "uid": "f1"}),
    )
    .await;
    assert_eq!(find, Some(json!({"uid": "f1", "error": BLOCKED})));
}

#[tokio::test]
async fn revoking_closes_open_connections() {
    let (port, agent) = start(true).await;
    let mut socket = connect(port, Some(ORIGIN), None).await.unwrap();
    send(&mut socket, &json!({"certificate": null, "uid": "s1"})).await;
    agent.revoked.send(ORIGIN.into()).unwrap();
    assert_eq!(next(&mut socket).await, None);
}

#[tokio::test]
async fn limits_connections_per_origin() {
    let (port, _) = start(true).await;
    let mut open = Vec::new();
    for _ in 0..MAX_CONNECTIONS_PER_ORIGIN {
        let mut socket = connect(port, Some(ORIGIN), None).await.unwrap();
        assert!(
            send(&mut socket, &json!({"certificate": null, "uid": "s1"}))
                .await
                .is_some()
        );
        open.push(socket);
    }
    let mut extra = connect(port, Some(ORIGIN), None).await.unwrap();
    assert_eq!(next(&mut extra).await, None);
    // Other sites still get in.
    let mut other = connect(port, Some("https://other.example"), None)
        .await
        .unwrap();
    assert!(
        send(&mut other, &json!({"certificate": null, "uid": "s1"}))
            .await
            .is_some()
    );
    // A closed connection frees its place.
    drop(open.pop());
    tokio::time::sleep(Duration::from_millis(100)).await;
    let mut again = connect(port, Some(ORIGIN), None).await.unwrap();
    assert!(
        send(&mut again, &json!({"certificate": null, "uid": "s1"}))
            .await
            .is_some()
    );
}

#[tokio::test]
async fn closes_connections_that_never_ask_for_permission() {
    let (port, _) = start(true).await;
    let mut socket = connect(port, Some(ORIGIN), None).await.unwrap();
    socket.send(Message::text("ping")).await.unwrap();
    let closed = tokio::time::timeout(AUTHORIZE_WITHIN * 3, next(&mut socket)).await;
    assert_eq!(closed, Ok(None));
}

#[tokio::test]
async fn closes_idle_connections_but_not_pinging_ones() {
    let (port, _) = start(true).await;
    let mut idle = connect(port, Some(ORIGIN), None).await.unwrap();
    let mut pinging = connect(port, Some(ORIGIN), None).await.unwrap();
    for socket in [&mut idle, &mut pinging] {
        send(socket, &json!({"certificate": null, "uid": "s1"})).await;
    }
    // qz-tray.js keeps its connection alive with "ping" frames.
    let started = tokio::time::Instant::now();
    while started.elapsed() < IDLE_TIMEOUT + Duration::from_millis(500) {
        pinging.send(Message::text("ping")).await.unwrap();
        tokio::time::sleep(IDLE_TIMEOUT / 4).await;
    }
    assert_eq!(
        tokio::time::timeout(Duration::from_secs(1), next(&mut idle)).await,
        Ok(None)
    );
    let version = send(&mut pinging, &json!({"call": "getVersion", "uid": "v1"})).await;
    assert_eq!(version, Some(json!({"uid": "v1", "result": "2.3.0"})));
}

#[tokio::test]
async fn closes_connections_that_send_oversized_messages() {
    let (port, _) = start(true).await;
    let mut socket = connect(port, Some(ORIGIN), None).await.unwrap();
    let huge = "x".repeat(MAX_MESSAGE_SIZE + 1);
    let _ = socket.send(Message::text(huge)).await;
    assert_eq!(next(&mut socket).await, None);
}

/// Runs the real qz-tray.js client (a development dependency) against the server, the way a web
/// page does. Requires `npm ci`.
#[tokio::test]
async fn works_with_the_qz_tray_js_client() {
    let (port, agent) = start(true).await;
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/qz-client.mjs");
    let output = tokio::process::Command::new("node")
        .arg(script)
        .arg(port.to_string())
        .output()
        .await
        .expect("Node.js must be installed to run the compatibility test");
    assert!(
        output.status.success(),
        "qz-tray.js client failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let found: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        found,
        json!({"printers": ["Office Laser", "POS-58"], "printer": "POS-58"})
    );

    // The site's name and logo travel in the setup message's certificate field.
    let certificates = agent.certificates.lock().unwrap();
    let presented: Value = serde_json::from_str(certificates[0].as_deref().unwrap()).unwrap();
    assert_eq!(presented["nanoprint"]["name"], json!("Shop Example"));

    let printed = agent.printed.lock().unwrap();
    let (origin, request) = &printed[0];
    assert_eq!(origin, ORIGIN);
    assert_eq!(
        request.printer,
        crate::protocol::PrinterTarget::Name("POS-58".into())
    );
    assert_eq!(request.job_name.as_deref(), Some("Ticket"));
    // "Año" in code page 850, then the base64 element.
    assert_eq!(
        request.data,
        [0x1b, b'@', b'A', 0xA4, b'o', b'\n', 0x1b, b'@']
    );
}

/// Runs web-escpos-printer's QZ Tray transport (a development dependency) against the server, the
/// way a page using that library does, including a site name set on the same qz-tray module.
/// Requires `npm ci`.
#[tokio::test]
async fn works_with_web_escpos_printer() {
    let (port, agent) = start(true).await;
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/web-escpos-client.mjs");
    let output = tokio::process::Command::new("node")
        .arg(script)
        .arg(port.to_string())
        .output()
        .await
        .expect("Node.js must be installed to run the compatibility test");
    assert!(
        output.status.success(),
        "web-escpos-printer client failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        result,
        json!({"printers": ["Office Laser", "POS-58"], "connected": true})
    );

    let certificates = agent.certificates.lock().unwrap();
    let presented: Value = serde_json::from_str(certificates[0].as_deref().unwrap()).unwrap();
    assert_eq!(presented["nanoprint"]["name"], json!("Web ESC/POS shop"));

    let printed = agent.printed.lock().unwrap();
    let (origin, request) = &printed[0];
    assert_eq!(origin, ORIGIN);
    assert_eq!(
        request.printer,
        crate::protocol::PrinterTarget::Name("POS-58".into())
    );
    assert_eq!(request.data, [0x1b, b'@', b'H', b'i', b'\n']);
}

/// A certificate in a temporary directory, removed when dropped.
struct TestCertificate {
    dir: std::path::PathBuf,
    certificate: crate::tls::Certificate,
}

impl TestCertificate {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("nanoprint-{name}-{}", std::process::id()));
        let (certificate, _) = crate::tls::load_or_create(&dir).unwrap();
        Self { dir, certificate }
    }
}

impl Drop for TestCertificate {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

impl TestCertificate {
    /// A root store trusting only this certificate's authority.
    fn roots(&self) -> rustls::RootCertStore {
        use rustls::pki_types::{CertificateDer, pem::PemObject};
        let mut roots = rustls::RootCertStore::empty();
        for certificate in CertificateDer::pem_file_iter(&self.certificate.ca_path).unwrap() {
            roots.add(certificate.unwrap()).unwrap();
        }
        roots
    }
}

fn tls_config(certificate: &crate::tls::Certificate) -> Arc<rustls::ServerConfig> {
    crate::tls::server_config(crate::tls::CertificateResolver::new(certificate).unwrap()).unwrap()
}

/// Opens a TLS connection to `localhost:port` as a client trusting only `roots`.
async fn tls_connect(
    port: u16,
    roots: rustls::RootCertStore,
) -> std::io::Result<tokio_rustls::client::TlsStream<TcpStream>> {
    let client = rustls::ClientConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .unwrap()
    .with_root_certificates(roots)
    .with_no_client_auth();
    let tcp = TcpStream::connect(("127.0.0.1", port)).await?;
    tokio_rustls::TlsConnector::from(Arc::new(client))
        .connect(
            rustls::pki_types::ServerName::try_from("localhost").unwrap(),
            tcp,
        )
        .await
}

/// `wss://` end to end: a TLS client that trusts only this installation's authority.
#[tokio::test]
async fn serves_wss_with_the_local_certificate() {
    let test = TestCertificate::new("wss");
    let config = tls_config(&test.certificate);
    let (port, agent) = start_with(true, Some(config)).await;

    let tls = tls_connect(port, test.roots()).await.unwrap();
    assert_eq!(agent.certificate_refusals.load(Ordering::SeqCst), 0);
    let mut request = format!("wss://localhost:{port}/")
        .into_client_request()
        .unwrap();
    request
        .headers_mut()
        .insert("origin", HeaderValue::from_static(ORIGIN));
    let (mut socket, _) = tokio_tungstenite::client_async(request, tls).await.unwrap();

    let version = send(&mut socket, &json!({"call": "getVersion", "uid": "v1"})).await;
    assert_eq!(version, Some(json!({"uid": "v1", "result": "2.3.0"})));
}

/// A renewal takes effect on the running server: new connections get the new certificate.
#[tokio::test]
async fn serves_a_renewed_certificate_without_restarting() {
    let old = TestCertificate::new("renew-old");
    let new = TestCertificate::new("renew-new");
    let resolver = crate::tls::CertificateResolver::new(&old.certificate).unwrap();
    let config = crate::tls::server_config(Arc::clone(&resolver)).unwrap();
    let (port, _) = start_with(true, Some(config)).await;

    assert!(tls_connect(port, old.roots()).await.is_ok());
    assert!(tls_connect(port, new.roots()).await.is_err());
    resolver.replace(&new.certificate).unwrap();
    assert!(tls_connect(port, new.roots()).await.is_ok());
    assert!(tls_connect(port, old.roots()).await.is_err());
}

/// A client that does not trust the authority ends the handshake with an alert. NanoPrint notices,
/// so it can offer to install the certificate.
#[tokio::test]
async fn notices_clients_that_refuse_the_certificate() {
    let test = TestCertificate::new("refused");
    let config = tls_config(&test.certificate);
    let (port, agent) = start_with(true, Some(config)).await;
    assert!(
        tls_connect(port, rustls::RootCertStore::empty())
            .await
            .is_err()
    );
    // The server reads the alert on its own task.
    for _ in 0..100 {
        if agent.certificate_refusals.load(Ordering::SeqCst) == 1 {
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("the refused certificate was not reported");
}

/// The real qz-tray.js client over `wss://localhost`, as from an HTTPS page that trusts the local
/// authority.
#[tokio::test]
async fn works_with_the_qz_tray_js_client_over_wss() {
    let test = TestCertificate::new("qz-wss");
    let config = tls_config(&test.certificate);
    let (port, agent) = start_with(true, Some(config)).await;
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/qz-client.mjs");
    let output = tokio::process::Command::new("node")
        .arg(script)
        .arg(port.to_string())
        .arg(&test.certificate.ca_path)
        .output()
        .await
        .expect("Node.js must be installed to run the compatibility test");
    assert!(
        output.status.success(),
        "qz-tray.js client failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(agent.printed.lock().unwrap().len(), 1);
}
