//! Per-origin consent. The first request from an unknown site opens a NanoPrint window; the answer
//! is stored (signed) for HTTPS sites or kept for this session only. Both kinds are listed in the
//! panel and can be revoked.

use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex, MutexGuard, PoisonError},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde::Serialize;
use tauri::{AppHandle, State};
use tokio::sync::{broadcast, oneshot};
use url::Url;

use crate::{
    identity::{self, Identity},
    store::{
        Decision, Event, EventKind, HistoryCategory, HistoryEntry, HistoryFilter, Site, Store,
    },
    ui_error::UiError,
    window,
};

/// A prompt left unanswered this long, counted from when it is shown, blocks the site for the
/// session.
const PROMPT_TIMEOUT: Duration = Duration::from_secs(120);
/// Sites that may wait for an answer at once. More are refused without a prompt, so one page with
/// many origins (subdomains) can neither flood the user nor hold every connection while waiting.
const MAX_WAITING_SITES: usize = 4;

/// Event telling the permission window to show the next request.
pub const REQUEST_CHANGED: &str = "permission-request-changed";

#[derive(Default)]
struct Pending {
    /// Origins waiting for the user, in order. The window shows the first one.
    queue: VecDeque<String>,
    waiters: HashMap<String, Vec<oneshot::Sender<Decision>>>,
    /// How each waiting site presents itself, when it says.
    identities: HashMap<String, Identity>,
    /// Changes whenever another request is shown, so the timer of an earlier one does nothing.
    shown: u64,
}

/// A decision kept until NanoPrint closes.
struct SessionDecision {
    decision: Decision,
    /// Unix seconds.
    decided_at: i64,
    identity: Option<Identity>,
}

pub struct Permissions {
    store: Arc<Store>,
    /// Decisions that are not stored (non-HTTPS sites, "remember" unchecked, unanswered prompts).
    session: Mutex<HashMap<String, SessionDecision>>,
    pending: Mutex<Pending>,
    revoked: broadcast::Sender<String>,
    app: AppHandle,
}

impl Permissions {
    pub fn new(store: Arc<Store>, app: AppHandle) -> Self {
        Self {
            store,
            session: Mutex::default(),
            pending: Mutex::default(),
            revoked: broadcast::channel(64).0,
            app,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.revoked.subscribe()
    }

    /// The current decision for `origin`: this session's, or a trustworthy stored one.
    pub fn decision(&self, origin: &str) -> Option<Decision> {
        if let Some(entry) = lock(&self.session).get(origin) {
            return Some(entry.decision);
        }
        self.store.decision(origin).unwrap_or_else(|error| {
            log::error!("could not read the decision for {origin}: {error}");
            None
        })
    }

    /// Like [`Self::decision`], but right after startup it first waits for the signing key, so a
    /// remembered site is not asked again just because the credential store was still answering.
    pub async fn decision_when_ready(&self, origin: &str) -> Option<Decision> {
        self.store.key_ready().await;
        self.decision(origin)
    }

    /// Whether `origin` may print. Asks the user when nothing is decided yet; `identity` is how
    /// the site presents itself in the prompt.
    pub async fn authorize(self: &Arc<Self>, origin: String, identity: Option<Identity>) -> bool {
        if let Some(decision) = self.decision_when_ready(&origin).await {
            return decision == Decision::Allow;
        }
        let (sender, receiver) = oneshot::channel();
        let shown = {
            let mut pending = lock(&self.pending);
            if !pending.waiters.contains_key(&origin) && pending.queue.len() >= MAX_WAITING_SITES {
                log::debug!("{origin} asked while {MAX_WAITING_SITES} sites wait; refused");
                return false;
            }
            let waiters = pending.waiters.entry(origin.clone()).or_default();
            waiters.push(sender);
            if waiters.len() == 1 {
                pending.queue.push_back(origin.clone());
            }
            if let Some(identity) = identity {
                pending.identities.insert(origin.clone(), identity);
            }
            // The prompt shows this request now only if nothing was waiting before it.
            (pending.queue.len() == 1 && pending.waiters[&origin].len() == 1).then(|| {
                pending.shown += 1;
                pending.shown
            })
        };
        if let Some(shown) = shown {
            self.expire_later(origin, shown);
            window::show_permission(&self.app);
        }
        receiver
            .await
            .is_ok_and(|decision| decision == Decision::Allow)
    }

    /// The request the permission window should show.
    pub fn current(&self) -> Option<PermissionRequest> {
        let pending = lock(&self.pending);
        let origin = pending.queue.front()?;
        let identity = pending.identities.get(origin);
        Some(PermissionRequest {
            rememberable: rememberable(origin),
            origin: origin.clone(),
            name: identity.and_then(|identity| identity.name.clone()),
            icon: identity
                .and_then(|identity| identity.icon.as_deref())
                .map(identity::icon_data_url),
        })
    }

    pub fn has_pending(&self) -> bool {
        !lock(&self.pending).queue.is_empty()
    }

    /// The user's answer to the request on screen. Answers for any other origin are ignored: the
    /// window can only decide what it was asked.
    pub fn answer(self: &Arc<Self>, origin: &str, decision: Decision, remember: bool) {
        if lock(&self.pending).queue.front().map(String::as_str) != Some(origin) {
            log::warn!("ignoring an answer for {origin}, which is not being asked about");
            return;
        }
        self.decide(origin, decision, remember, true);
    }

    /// The user closed the prompt: the request on screen is blocked for this session. Returns
    /// whether more requests are waiting, so the window should stay open for them.
    pub fn dismiss_shown(self: &Arc<Self>) -> bool {
        let shown = lock(&self.pending).queue.front().cloned();
        if let Some(origin) = shown {
            self.decide(&origin, Decision::Block, false, false);
        }
        self.has_pending()
    }

    /// Updates how an allowed site presents itself, for the site list.
    pub fn update_identity(&self, origin: &str, identity: Identity) {
        if let Some(entry) = lock(&self.session).get_mut(origin) {
            entry.identity = Some(identity);
            return;
        }
        if let Err(error) = self.store.set_identity(origin, &identity) {
            log::warn!("could not update how {origin} presents itself: {error}");
        }
    }

    /// Forgets the decision for `origin` and closes its open connections.
    pub fn revoke(&self, origin: &str) {
        if let Err(error) = self.store.remove(origin) {
            log::error!("could not remove {origin}: {error}");
        }
        lock(&self.session).remove(origin);
        let _ = self.revoked.send(origin.to_owned());
        self.record(EventKind::PermissionRevoked, origin);
    }

    /// Stored and session decisions, most recent first.
    pub fn sites(&self) -> Result<Vec<SiteView>, crate::store::Error> {
        let mut sites: Vec<SiteView> = self
            .store
            .sites()?
            .into_iter()
            .map(|site| SiteView {
                site,
                session_only: false,
            })
            .collect();
        for (origin, entry) in lock(&self.session).iter() {
            let identity = entry.identity.as_ref();
            let view = SiteView {
                site: Site {
                    origin: origin.clone(),
                    decision: Some(entry.decision),
                    granted_at: entry.decided_at,
                    last_used_at: None,
                    job_count: 0,
                    name: identity.and_then(|identity| identity.name.clone()),
                    icon: identity
                        .and_then(|identity| identity.icon.as_deref())
                        .map(identity::icon_data_url),
                },
                session_only: true,
            };
            // A session decision is the one in force, even next to a stored row.
            match sites.iter_mut().find(|known| known.site.origin == *origin) {
                Some(known) => *known = view,
                None => sites.push(view),
            }
        }
        sites.sort_by_key(|view| std::cmp::Reverse(view.site.granted_at));
        Ok(sites)
    }

    /// Applies a decision, wakes every connection waiting for it and moves the prompt on to the
    /// next request (or closes it, when `update_window`).
    fn decide(
        self: &Arc<Self>,
        origin: &str,
        decision: Decision,
        remember: bool,
        update_window: bool,
    ) {
        let identity = lock(&self.pending).identities.remove(origin);
        let stored = remember && rememberable(origin) && {
            match self.store.set_decision(origin, decision) {
                Ok(()) => true,
                Err(error) => {
                    log::error!("could not store the decision for {origin}: {error}");
                    false
                }
            }
        };
        if stored {
            // The stored decision replaces any earlier one for this session.
            lock(&self.session).remove(origin);
            if let Some(identity) = &identity
                && let Err(error) = self.store.set_identity(origin, identity)
            {
                log::warn!("could not store how {origin} presents itself: {error}");
            }
        } else {
            lock(&self.session).insert(
                origin.to_owned(),
                SessionDecision {
                    decision,
                    decided_at: now(),
                    identity,
                },
            );
        }
        let kind = match decision {
            Decision::Allow => EventKind::PermissionGranted,
            Decision::Block => EventKind::PermissionDenied,
        };
        self.record(kind, origin);

        let (waiters, next) = {
            let mut pending = lock(&self.pending);
            let was_shown = pending.queue.front().map(String::as_str) == Some(origin);
            pending.queue.retain(|queued| queued != origin);
            let waiters = pending.waiters.remove(origin).unwrap_or_default();
            let next = if was_shown {
                pending.shown += 1;
                Some(
                    pending
                        .queue
                        .front()
                        .cloned()
                        .map(|next| (next, pending.shown)),
                )
            } else {
                None
            };
            (waiters, next)
        };
        for waiter in waiters {
            let _ = waiter.send(decision);
        }
        match next {
            Some(Some((next, shown))) => {
                self.expire_later(next, shown);
                if update_window {
                    window::refresh_permission(&self.app);
                }
            }
            Some(None) if update_window => window::close_permission(&self.app),
            _ => {}
        }
    }

    /// Blocks `origin` for the session if its request is still on screen after
    /// [`PROMPT_TIMEOUT`].
    fn expire_later(self: &Arc<Self>, origin: String, shown: u64) {
        let permissions = Arc::clone(self);
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(PROMPT_TIMEOUT).await;
            let still_shown = {
                let pending = lock(&permissions.pending);
                pending.shown == shown && pending.queue.front() == Some(&origin)
            };
            if still_shown {
                log::info!(
                    "the prompt for {origin} was not answered; blocking it for this session"
                );
                permissions.decide(&origin, Decision::Block, false, true);
            }
        });
    }

    fn record(&self, kind: EventKind, origin: &str) {
        let event = Event {
            origin: Some(origin),
            ..Event::default()
        };
        if let Err(error) = self.store.record(kind, &event) {
            log::warn!("could not record history: {error}");
        }
    }
}

/// Only HTTPS origins (and loopback, for development) are remembered: on plain HTTP a network
/// attacker could impersonate the site.
fn rememberable(origin: &str) -> bool {
    let Ok(url) = Url::parse(origin) else {
        return false;
    };
    url.scheme() == "https" || matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"))
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    // The guarded maps stay consistent across a panic: every update is a single operation.
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|elapsed| i64::try_from(elapsed.as_secs()).ok())
        .unwrap_or_default()
}

#[derive(Debug, Clone, Serialize)]
pub struct PermissionRequest {
    origin: String,
    rememberable: bool,
    /// The name the site gave itself, if any.
    name: Option<String>,
    /// The site's logo as a `data:image/png` URL, if any.
    icon: Option<String>,
}

/// A site as the panel lists it. Mirrors `Site` in `src/lib/agent.ts`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteView {
    #[serde(flatten)]
    site: Site,
    /// The decision lasts until NanoPrint closes; it is not stored.
    session_only: bool,
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri passes command arguments by value.
pub fn get_permission_request(
    permissions: State<'_, Arc<Permissions>>,
) -> Option<PermissionRequest> {
    permissions.current()
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri passes command arguments by value.
pub fn answer_permission_request(
    permissions: State<'_, Arc<Permissions>>,
    origin: String,
    allow: bool,
    remember: bool,
) {
    let decision = if allow {
        Decision::Allow
    } else {
        Decision::Block
    };
    permissions.answer(&origin, decision, remember);
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri passes command arguments by value.
pub fn list_sites(permissions: State<'_, Arc<Permissions>>) -> Result<Vec<SiteView>, UiError> {
    permissions
        .sites()
        .map_err(|error| UiError::with_detail("storage", &error))
}

/// The last 500 history rows that match the panel's filters.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri passes command arguments by value.
pub fn list_history(
    store: State<'_, Arc<Store>>,
    search: Option<String>,
    category: Option<HistoryCategory>,
) -> Result<Vec<HistoryEntry>, UiError> {
    let filter = HistoryFilter {
        search: search.as_deref(),
        category,
    };
    store
        .history(&filter, 500)
        .map_err(|error| UiError::with_detail("storage", &error))
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri passes command arguments by value.
pub fn revoke_site(permissions: State<'_, Arc<Permissions>>, origin: String) {
    permissions.revoke(&origin);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remembers_only_https_and_loopback_origins() {
        assert!(rememberable("https://shop.example"));
        assert!(rememberable("http://localhost:3000"));
        assert!(rememberable("http://127.0.0.1:5173"));
        assert!(!rememberable("http://shop.example"));
        assert!(!rememberable("http://192.168.1.10"));
    }
}
