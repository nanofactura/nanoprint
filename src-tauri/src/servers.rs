//! Starts the `ws://` and `wss://` print servers, renews the local certificate while they run, and
//! reports their state, and the certificate's, to the panel.

use std::{
    path::PathBuf,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
    time::Duration,
};

use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::{
    agent::AppAgent,
    onboarding, server,
    settings::SettingsStore,
    tls::{self, Certificate, CertificateResolver, Meta},
    trust,
    ui_error::UiError,
};

/// How often a running NanoPrint checks whether the certificate is due for renewal.
const RENEWAL_CHECK: Duration = Duration::from_hours(12);

/// State of one listener.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase", tag = "state")]
pub enum Listener {
    #[default]
    Starting,
    Listening {
        port: u16,
    },
    Failed {
        message: String,
    },
}

#[derive(Debug, Default)]
struct State {
    insecure: Listener,
    secure: Listener,
    certificate: Option<Certificate>,
    /// Serves `certificate` to the `wss://` server; a renewal swaps it.
    resolver: Option<Arc<CertificateResolver>>,
    /// Where the certificate files live.
    tls_dir: Option<PathBuf>,
    /// Authorities replaced by a renewal, removed from the trust store with the next install.
    replaced: Vec<Meta>,
}

#[derive(Debug, Default)]
pub struct Servers(Mutex<State>);

impl Servers {
    fn lock(&self) -> MutexGuard<'_, State> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The local certificate, once the `wss://` server has prepared it.
    pub fn certificate(&self) -> Option<Certificate> {
        self.lock().certificate.clone()
    }
}

pub fn start(app: &AppHandle, agent: &Arc<AppAgent>) {
    app.manage(Servers::default());
    tauri::async_runtime::spawn(run_insecure(app.clone(), Arc::clone(agent)));
    tauri::async_runtime::spawn(run_secure(app.clone(), Arc::clone(agent)));
}

/// The port chosen in Preferences, or the ones unmodified qz-tray.js clients try.
fn ports(chosen: Option<u16>, defaults: &[u16]) -> Vec<u16> {
    chosen.map_or_else(|| defaults.to_vec(), |port| vec![port])
}

async fn run_insecure(app: AppHandle, agent: Arc<AppAgent>) {
    let chosen = app.state::<SettingsStore>().get().insecure_port;
    match server::bind(&ports(chosen, &server::INSECURE_PORTS)).await {
        Ok(listening) => {
            log::info!("ws:// print server on port {}", listening.port);
            app.state::<Servers>().lock().insecure = Listener::Listening {
                port: listening.port,
            };
            server::serve(listening, agent, None).await;
        }
        Err(error) => {
            log::error!("ws:// print server could not start: {error}");
            app.state::<Servers>().lock().insecure = Listener::Failed {
                message: error.to_string(),
            };
        }
    }
}

async fn run_secure(app: AppHandle, agent: Arc<AppAgent>) {
    match prepare_secure(&app).await {
        Ok((listening, config)) => {
            log::info!("wss:// print server on port {}", listening.port);
            app.state::<Servers>().lock().secure = Listener::Listening {
                port: listening.port,
            };
            tauri::async_runtime::spawn(renew_when_due(app.clone()));
            server::serve(listening, agent, Some(config)).await;
        }
        Err(message) => {
            log::error!("wss:// print server could not start: {message}");
            app.state::<Servers>().lock().secure = Listener::Failed { message };
        }
    }
}

async fn prepare_secure(
    app: &AppHandle,
) -> Result<(server::Listening, Arc<rustls::ServerConfig>), String> {
    let dir = app
        .path()
        .app_local_data_dir()
        .map_err(|error| error.to_string())?
        .join("tls");
    let (certificate, replaced) = {
        let dir = dir.clone();
        tauri::async_runtime::spawn_blocking(move || tls::load_or_create(&dir))
            .await
            .map_err(|error| error.to_string())?
            .map_err(|error| error.to_string())?
    };
    let resolver = CertificateResolver::new(&certificate).map_err(|error| error.to_string())?;
    let config = tls::server_config(Arc::clone(&resolver)).map_err(|error| error.to_string())?;
    {
        let servers = app.state::<Servers>();
        let mut state = servers.lock();
        state.certificate = Some(certificate);
        state.resolver = Some(resolver);
        state.tls_dir = Some(dir);
        state.replaced = replaced;
    }
    let chosen = app.state::<SettingsStore>().get().secure_port;
    let listening = server::bind(&ports(chosen, &server::SECURE_PORTS))
        .await
        .map_err(|error| error.to_string())?;
    Ok((listening, config))
}

/// Renews the certificate before it expires while NanoPrint keeps running: a POS computer may go
/// months without restarting it. The new authority is not trusted yet, so the next browser that
/// refuses it brings up the reminder to enable secure sites again.
async fn renew_when_due(app: AppHandle) {
    let mut interval = tokio::time::interval(RENEWAL_CHECK);
    // The first tick is immediate, and startup has just checked.
    interval.tick().await;
    loop {
        interval.tick().await;
        let (due, dir, resolver) = {
            let servers = app.state::<Servers>();
            let state = servers.lock();
            (
                state
                    .certificate
                    .as_ref()
                    .is_some_and(|certificate| tls::needs_renewal(&certificate.meta)),
                state.tls_dir.clone(),
                state.resolver.clone(),
            )
        };
        let (true, Some(dir), Some(resolver)) = (due, dir, resolver) else {
            continue;
        };
        let renewed = tauri::async_runtime::spawn_blocking(move || tls::load_or_create(&dir)).await;
        match renewed {
            Ok(Ok((certificate, replaced))) => {
                if let Err(error) = resolver.replace(&certificate) {
                    log::error!("could not serve the renewed certificate: {error}");
                    continue;
                }
                {
                    let servers = app.state::<Servers>();
                    let mut state = servers.lock();
                    state.certificate = Some(certificate);
                    state.replaced = replaced;
                }
                log::info!("renewed the local certificate");
                onboarding::certificate_renewed(&app);
            }
            Ok(Err(error)) => log::error!("could not renew the local certificate: {error}"),
            Err(error) => log::error!("could not renew the local certificate: {error}"),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusView {
    insecure: Listener,
    secure: Listener,
    certificate: Option<CertificateView>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CertificateView {
    /// Whether this computer trusts the local certificate (browsers accept `wss://`).
    trusted: bool,
    /// Unix seconds; NanoPrint renews the certificate before this date.
    expires_at: i64,
}

#[tauri::command]
pub async fn get_server_status(app: AppHandle) -> StatusView {
    let (insecure, secure, certificate) = {
        let servers = app.state::<Servers>();
        let state = servers.lock();
        (
            state.insecure.clone(),
            state.secure.clone(),
            state.certificate.clone(),
        )
    };
    let certificate = match certificate {
        Some(certificate) => {
            let expires_at = certificate.meta.expires_at;
            let trusted =
                tauri::async_runtime::spawn_blocking(move || trust::is_trusted(&certificate))
                    .await
                    .unwrap_or(false);
            Some(CertificateView {
                trusted,
                expires_at,
            })
        }
        None => None,
    };
    StatusView {
        insecure,
        secure,
        certificate,
    }
}

/// Adds the local authority to the trust store. The system asks the user to confirm.
#[tauri::command]
pub async fn install_certificate(app: AppHandle) -> Result<(), UiError> {
    let (certificate, dir, replaced) = {
        let servers = app.state::<Servers>();
        let state = servers.lock();
        let certificate = state
            .certificate
            .clone()
            .ok_or(UiError::new("certificate_not_ready"))?;
        (certificate, state.tls_dir.clone(), state.replaced.clone())
    };
    let (removed, installed) = tauri::async_runtime::spawn_blocking(move || {
        let removed: Vec<Meta> = replaced
            .into_iter()
            .filter(
                |old| match trust::contains(old).then(|| trust::remove(old)) {
                    None | Some(Ok(())) => true,
                    Some(Err(error)) => {
                        log::warn!("could not remove a replaced certificate authority: {error}");
                        false
                    }
                },
            )
            .collect();
        if let Some(dir) = dir
            && !removed.is_empty()
            && let Err(error) = tls::forget_replaced(&dir, &removed)
        {
            log::warn!("could not update the list of replaced authorities: {error}");
        }
        (removed, trust::install(&certificate))
    })
    .await
    .map_err(|error| UiError::with_detail("unexpected", &error))?;
    app.state::<Servers>()
        .lock()
        .replaced
        .retain(|meta| !removed.contains(meta));
    installed.map_err(trust_error)
}

/// Removes the local authority from the trust store.
#[tauri::command]
pub async fn remove_certificate(app: AppHandle) -> Result<(), UiError> {
    let meta = app
        .state::<Servers>()
        .lock()
        .certificate
        .as_ref()
        .map(|certificate| certificate.meta.clone())
        .ok_or(UiError::new("certificate_not_ready"))?;
    tauri::async_runtime::spawn_blocking(move || trust::remove(&meta))
        .await
        .map_err(|error| UiError::with_detail("unexpected", &error))?
        .map_err(trust_error)
}

fn trust_error(error: trust::Error) -> UiError {
    log::warn!("trust store change failed: {error}");
    match error {
        trust::Error::MissingTool => UiError::new("trust_tool_missing"),
        trust::Error::Failed(detail) => UiError::with_detail("trust_failed", &detail),
    }
}
