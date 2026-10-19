//! Connects the print server to the rest of the app: permissions, printers and history.

use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex, PoisonError},
    time::{Duration, Instant},
};

use futures_util::future::BoxFuture;
use tauri::{AppHandle, Manager};
use tokio::sync::broadcast;

use crate::{
    identity::Identity,
    onboarding,
    permissions::Permissions,
    printers::{
        self,
        network::{Links, NetworkPrinter},
    },
    protocol::{PrintRequest, PrinterTarget},
    server::{Agent, Printer, Rejection},
    settings::{Settings, SettingsStore},
    store::{Decision, Event, EventKind, Store},
    updates::Updates,
};

/// Print jobs a single site may send per minute.
const JOBS_PER_MINUTE: usize = 30;
/// Refused connections recorded in the history per minute. Each origin and reason is recorded
/// once a minute at most, so a page retrying in a loop cannot flood the history.
const REJECTIONS_PER_MINUTE: usize = 20;
/// How long the system's printer list is reused (see [`PrinterCache`]).
const PRINTER_LIST_TTL: Duration = Duration::from_secs(30);
const MINUTE: Duration = Duration::from_secs(60);

pub struct AppAgent {
    store: Arc<Store>,
    permissions: Arc<Permissions>,
    printers: Arc<PrinterCache>,
    recent_jobs: Mutex<HashMap<String, RecentJobs>>,
    recent_rejections: Mutex<VecDeque<(Instant, Option<String>, Rejection)>>,
    app: AppHandle,
}

#[derive(Default)]
struct RecentJobs {
    sent: VecDeque<Instant>,
    /// When the history last recorded this site hitting the limit.
    reported_limit: Option<Instant>,
}

enum Admission {
    Allowed,
    /// Over the limit; `report` is true once a minute, so the history gets one entry per minute
    /// of refused jobs instead of one per job.
    Limited {
        report: bool,
    },
}

impl AppAgent {
    pub fn new(
        store: Arc<Store>,
        permissions: Arc<Permissions>,
        printers: Arc<PrinterCache>,
        app: AppHandle,
    ) -> Self {
        Self {
            store,
            permissions,
            printers,
            recent_jobs: Mutex::default(),
            recent_rejections: Mutex::default(),
            app,
        }
    }

    /// The current preferences, which say the printers web pages may use (Printers tab).
    fn settings(&self) -> Settings {
        self.app.state::<SettingsStore>().get()
    }

    /// Whether a refused connection should go into the history (see [`REJECTIONS_PER_MINUTE`]).
    fn should_record_rejection(&self, origin: Option<&str>, reason: Rejection) -> bool {
        let now = Instant::now();
        let mut recent = self
            .recent_rejections
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        while recent
            .front()
            .is_some_and(|(at, ..)| now.duration_since(*at) > MINUTE)
        {
            recent.pop_front();
        }
        let repeated = recent
            .iter()
            .any(|(_, known, code)| known.as_deref() == origin && *code == reason);
        if repeated || recent.len() >= REJECTIONS_PER_MINUTE {
            return false;
        }
        recent.push_back((now, origin.map(str::to_owned), reason));
        true
    }

    /// Sliding one-minute window per origin.
    fn admit(&self, origin: &str) -> Admission {
        let now = Instant::now();
        let mut recent = self
            .recent_jobs
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let jobs = recent.entry(origin.to_owned()).or_default();
        while jobs
            .sent
            .front()
            .is_some_and(|sent| now.duration_since(*sent) > MINUTE)
        {
            jobs.sent.pop_front();
        }
        if jobs.sent.len() < JOBS_PER_MINUTE {
            jobs.sent.push_back(now);
            return Admission::Allowed;
        }
        let report = jobs
            .reported_limit
            .is_none_or(|reported| now.duration_since(reported) > MINUTE);
        if report {
            jobs.reported_limit = Some(now);
        }
        Admission::Limited { report }
    }
}

impl Agent for AppAgent {
    fn authorize(&self, origin: &str, certificate: Option<String>) -> BoxFuture<'static, bool> {
        let permissions = Arc::clone(&self.permissions);
        let origin = origin.to_owned();
        Box::pin(async move {
            // The site's name and logo are only read when they will be shown: never for a blocked
            // site.
            match permissions.decision_when_ready(&origin).await {
                Some(Decision::Block) => false,
                Some(Decision::Allow) => {
                    if let Some(identity) = identity(certificate).await {
                        permissions.update_identity(&origin, identity);
                    }
                    true
                }
                None => {
                    let identity = identity(certificate).await;
                    permissions.authorize(origin, identity).await
                }
            }
        })
    }

    fn revocations(&self) -> broadcast::Receiver<String> {
        self.permissions.subscribe()
    }

    /// The printers web pages may see: those the system reports or the user added, and the user
    /// offers them.
    fn printers(&self) -> BoxFuture<'static, Result<Vec<Printer>, String>> {
        let cache = Arc::clone(&self.printers);
        let settings = self.settings();
        Box::pin(async move {
            let mut printers: Vec<_> = all_printers(&cache, &settings)
                .await?
                .iter()
                .map(Destination::printer)
                .collect();
            printers.retain(|printer| settings.shares(printer));
            Ok(printers)
        })
    }

    fn print(&self, origin: &str, request: PrintRequest) -> BoxFuture<'static, Result<(), String>> {
        // An update waits until printing has been quiet for a while.
        self.app.state::<Updates>().printed();
        let store = Arc::clone(&self.store);
        let cache = Arc::clone(&self.printers);
        let links = Arc::clone(&self.app.state::<Arc<Links>>());
        let settings = self.settings();
        let origin = origin.to_owned();
        let admission = self.admit(&origin);
        Box::pin(async move {
            if let Admission::Limited { report } = admission {
                if report {
                    let event = Event {
                        origin: Some(&origin),
                        ..Event::default()
                    };
                    record(&store, EventKind::RateLimited, &event);
                }
                return Err("Too many print jobs; try again in a minute".into());
            }
            let job_name = request
                .job_name
                .clone()
                .unwrap_or_else(|| "NanoPrint".into());
            let target = match &request.printer {
                PrinterTarget::Name(name) => name.clone(),
                PrinterTarget::Host { host, port } => format!("{host}:{port}"),
            };
            let bytes = request.data.len() * usize::try_from(request.copies).unwrap_or(1);
            let started = Instant::now();
            let result = deliver(&cache, &links, &settings, request, &job_name).await;
            let event = Event {
                origin: Some(&origin),
                printer: Some(&target),
                job_name: Some(&job_name),
                bytes: Some(bytes),
                duration: Some(started.elapsed()),
                error: result
                    .as_ref()
                    .err()
                    .map(|failure| failure.history.as_str()),
            };
            if result.is_ok() {
                record(&store, EventKind::JobPrinted, &event);
                if let Err(error) = store.touch(&origin) {
                    log::warn!("could not update usage for {origin}: {error}");
                }
            } else {
                record(&store, EventKind::JobFailed, &event);
            }
            result.map_err(|failure| failure.message)
        })
    }

    fn rejected(&self, origin: Option<&str>, reason: Rejection) {
        if !self.should_record_rejection(origin, reason) {
            log::debug!("refused a connection from {origin:?}: {}", reason.code());
            return;
        }
        log::info!("refused a connection from {origin:?}: {}", reason.code());
        record(
            &self.store,
            EventKind::ConnectionRejected,
            &Event {
                origin,
                error: Some(reason.code()),
                ..Event::default()
            },
        );
    }

    fn certificate_refused(&self) {
        onboarding::certificate_refused(&self.app);
    }
}

/// Why a job did not print: the message for the page, and what the history keeps (a code the
/// panel translates, or the system's own message).
struct Failure {
    message: String,
    history: String,
}

impl Failure {
    fn coded(code: &str, message: &str) -> Self {
        Self {
            message: message.to_owned(),
            history: code.to_owned(),
        }
    }

    fn system(message: String) -> Self {
        Self {
            history: message.clone(),
            message,
        }
    }
}

/// Sends the job to the printer queue, or straight to a network printer.
async fn deliver(
    cache: &PrinterCache,
    links: &Links,
    settings: &Settings,
    request: PrintRequest,
    job_name: &str,
) -> Result<(), Failure> {
    // Only printers the operating system reports or the user added can be used: the name never
    // reaches the spooler unless it matches one exactly, and a page's address must be one the
    // user added.
    let destination = match &request.printer {
        PrinterTarget::Name(name) => find_printer(cache, settings, name)
            .await
            .map_err(Failure::system)?
            .ok_or_else(|| {
                Failure::coded("printer_not_found", "Specified printer could not be found.")
            })?,
        PrinterTarget::Host { host, port } => settings
            .network_printers
            .iter()
            .find(|printer| printer.is_at(host, *port))
            .cloned()
            .map(Destination::Network)
            .ok_or_else(|| {
                Failure::coded(
                    "network_printer",
                    "Network printers must be added in NanoPrint first",
                )
            })?,
    };
    // A printer kept from web pages does not exist as far as the page knows.
    if !settings.shares(&destination.printer()) {
        return Err(Failure::coded(
            "printer_hidden",
            "Specified printer could not be found.",
        ));
    }
    let data = request
        .data
        .repeat(usize::try_from(request.copies).unwrap_or(1));
    send(links, destination, job_name.to_owned(), data)
        .await
        .map_err(Failure::system)
}

/// A printer jobs can go to.
#[derive(Debug, Clone)]
pub enum Destination {
    /// Reported by the operating system: jobs go through its queue.
    System(Printer),
    /// Added by the user: jobs go over the network to its address.
    Network(NetworkPrinter),
}

impl Destination {
    pub fn printer(&self) -> Printer {
        match self {
            Self::System(printer) => printer.clone(),
            Self::Network(printer) => printer.printer(),
        }
    }
}

/// The printers the system reports, then the network printers the user added. A system printer
/// with the name of an added one is left out, so every name leads to one printer.
pub async fn all_printers(
    cache: &PrinterCache,
    settings: &Settings,
) -> Result<Vec<Destination>, String> {
    let added = &settings.network_printers;
    Ok(cache
        .list(false)
        .await?
        .into_iter()
        .filter(|printer| added.iter().all(|known| known.name != printer.name))
        .map(Destination::System)
        .chain(added.iter().cloned().map(Destination::Network))
        .collect())
}

/// The printer named exactly `name`: a network printer the user added, or one the system reports.
pub async fn find_printer(
    cache: &PrinterCache,
    settings: &Settings,
    name: &str,
) -> Result<Option<Destination>, String> {
    if let Some(printer) = settings
        .network_printers
        .iter()
        .find(|printer| printer.name == name)
    {
        return Ok(Some(Destination::Network(printer.clone())));
    }
    Ok(cache.find(name).await?.map(Destination::System))
}

/// Prints `data` as one raw job. The error is the system's message.
pub async fn send(
    links: &Links,
    destination: Destination,
    job_name: String,
    data: Vec<u8>,
) -> Result<(), String> {
    match destination {
        Destination::Network(printer) => links
            .send(&printer, &data)
            .await
            .map_err(|error| format!("{}: {error}", printer.address())),
        Destination::System(printer) => tauri::async_runtime::spawn_blocking(move || {
            printers::print_raw(&printer.name, &job_name, &data)
        })
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string()),
    }
}

/// The system's printer list. Listing can take seconds (an offline network printer on Windows),
/// so it is reused for [`PRINTER_LIST_TTL`] and refreshed early only when a name is missing.
#[derive(Default)]
pub struct PrinterCache(tokio::sync::Mutex<Option<(Instant, Vec<Printer>)>>);

impl PrinterCache {
    pub async fn list(&self, refresh: bool) -> Result<Vec<Printer>, String> {
        let mut cached = self.0.lock().await;
        if !refresh
            && let Some((listed_at, printers)) = cached.as_ref()
            && listed_at.elapsed() < PRINTER_LIST_TTL
        {
            return Ok(printers.clone());
        }
        let printers = tauri::async_runtime::spawn_blocking(printers::list)
            .await
            .map_err(|error| error.to_string())?
            .map_err(|error| error.to_string())?;
        *cached = Some((Instant::now(), printers.clone()));
        Ok(printers)
    }

    /// The printer the system reports with exactly this `name`. A printer added or renamed since
    /// the last listing is found by listing again.
    pub async fn find(&self, name: &str) -> Result<Option<Printer>, String> {
        let named = |printers: Vec<Printer>| printers.into_iter().find(|known| known.name == name);
        match named(self.list(false).await?) {
            Some(printer) => Ok(Some(printer)),
            None => Ok(named(self.list(true).await?)),
        }
    }
}

/// How the site presents itself, from the setup message. Decoding a logo is CPU work, kept off the
/// network threads.
async fn identity(certificate: Option<String>) -> Option<Identity> {
    let certificate = certificate?;
    tauri::async_runtime::spawn_blocking(move || Identity::from_certificate(&certificate))
        .await
        .ok()
        .flatten()
}

pub fn record(store: &Store, kind: EventKind, event: &Event<'_>) {
    if let Err(error) = store.record(kind, event) {
        log::warn!("could not record history: {error}");
    }
}

#[cfg(test)]
mod tests {
    use tokio::{io::AsyncReadExt, net::TcpListener, task::JoinHandle};

    use super::*;
    use crate::printers::network;

    /// A network printer on this computer that hands back the bytes of the first job it gets.
    async fn fake_printer() -> (u16, JoinHandle<Vec<u8>>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let job = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut data = Vec::new();
            socket.read_to_end(&mut data).await.unwrap();
            data
        });
        (port, job)
    }

    fn added(name: &str, port: u16) -> Settings {
        Settings {
            network_printers: vec![network::check(name, "127.0.0.1", port).unwrap()],
            ..Settings::default()
        }
    }

    fn job(printer: PrinterTarget, copies: u32) -> PrintRequest {
        PrintRequest {
            printer,
            job_name: None,
            copies,
            data: b"\x1b@ticket\n".to_vec(),
        }
    }

    async fn deliver_to(settings: &Settings, printer: PrinterTarget) -> Result<(), Failure> {
        // Network printers are found without listing the system's printers.
        let cache = PrinterCache::default();
        deliver(&cache, &Links::default(), settings, job(printer, 2), "Test").await
    }

    #[tokio::test]
    async fn prints_to_an_added_network_printer_by_name() {
        let (port, received) = fake_printer().await;
        let settings = added("Kitchen", port);
        let printed = deliver_to(&settings, PrinterTarget::Name("Kitchen".into())).await;
        assert!(printed.is_ok());
        assert_eq!(received.await.unwrap(), b"\x1b@ticket\n\x1b@ticket\n");
    }

    #[tokio::test]
    async fn prints_to_the_address_of_an_added_printer() {
        let (port, received) = fake_printer().await;
        let settings = added("Kitchen", port);
        let target = PrinterTarget::Host {
            host: "127.0.0.1".into(),
            port,
        };
        assert!(deliver_to(&settings, target).await.is_ok());
        assert_eq!(received.await.unwrap().len(), 2 * b"\x1b@ticket\n".len());
    }

    #[tokio::test]
    async fn refuses_an_address_the_user_did_not_add() {
        let (port, _received) = fake_printer().await;
        let settings = added("Kitchen", port);
        for (host, port) in [("127.0.0.1", port ^ 1), ("127.0.0.2", port)] {
            let target = PrinterTarget::Host {
                host: host.into(),
                port,
            };
            let failure = deliver_to(&settings, target).await.err().unwrap();
            assert_eq!(failure.history, "network_printer", "{host}:{port}");
        }
    }

    #[tokio::test]
    async fn refuses_a_network_printer_kept_from_sites() {
        let (port, _received) = fake_printer().await;
        let mut settings = added("Kitchen", port);
        settings.shared_printers.insert("Kitchen".into(), false);
        let target = PrinterTarget::Host {
            host: "127.0.0.1".into(),
            port,
        };
        let failure = deliver_to(&settings, target).await.err().unwrap();
        assert_eq!(failure.history, "printer_hidden");
    }

    #[tokio::test]
    async fn reports_a_network_printer_that_does_not_answer() {
        let (port, received) = fake_printer().await;
        received.abort();
        let _ = received.await;
        let settings = added("Kitchen", port);
        let failure = deliver_to(&settings, PrinterTarget::Name("Kitchen".into()))
            .await
            .err()
            .unwrap();
        assert!(failure.history.starts_with(&format!("127.0.0.1:{port}: ")));
    }
}
