//! Site decisions and the 30-day history, in SQLite. Every site row is signed with an HMAC whose
//! key lives in the operating system's credential store (see `keys.rs`), so a row written by
//! another program is detected and ignored.

use std::{
    path::Path,
    sync::{Mutex, MutexGuard, PoisonError},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use hmac::{Hmac, KeyInit, Mac};
use rusqlite::{Connection, OptionalExtension, functions::FunctionFlags, params};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use tokio::sync::watch;

use crate::identity::{self, Identity};

pub type Key = [u8; 32];

/// How long history rows are kept.
pub const RETENTION: Duration = Duration::from_hours(30 * 24);

/// How long after startup decisions wait for the signing key (see [`Store::key_ready`]).
const KEY_GRACE: Duration = Duration::from_secs(15);

const SCHEMA: &str = "
    PRAGMA journal_mode = WAL;
    CREATE TABLE IF NOT EXISTS sites (
        origin       TEXT PRIMARY KEY,
        decision     TEXT NOT NULL,
        granted_at   INTEGER NOT NULL,
        granted_via  TEXT NOT NULL,
        last_used_at INTEGER,
        job_count    INTEGER NOT NULL DEFAULT 0,
        signature    BLOB NOT NULL
    );
    CREATE TABLE IF NOT EXISTS events (
        id          INTEGER PRIMARY KEY,
        at          INTEGER NOT NULL,
        kind        TEXT NOT NULL,
        origin      TEXT,
        printer     TEXT,
        job_name    TEXT,
        bytes       INTEGER,
        duration_ms INTEGER,
        error       TEXT
    );
    CREATE INDEX IF NOT EXISTS events_at ON events (at);
";

/// Changes to [`SCHEMA`], applied in order; `PRAGMA user_version` counts those already applied.
const MIGRATIONS: &[&str] = &[
    // How sites present themselves (see `identity.rs`). Cosmetic, so not signed.
    "ALTER TABLE sites ADD COLUMN name TEXT;
     ALTER TABLE sites ADD COLUMN icon BLOB;",
];

const SITE_COLUMNS: &str =
    "origin, decision, granted_at, granted_via, last_used_at, job_count, signature, name, icon";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Decision {
    Allow,
    Block,
}

impl Decision {
    fn as_str(self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Block => "block",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "allow" => Some(Self::Allow),
            "block" => Some(Self::Block),
            _ => None,
        }
    }
}

/// A stored site decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Site {
    pub origin: String,
    /// `None` when the row is not trustworthy (bad signature or unknown value).
    pub decision: Option<Decision>,
    /// Unix seconds.
    pub granted_at: i64,
    pub last_used_at: Option<i64>,
    pub job_count: i64,
    /// The name the site gave itself, if any.
    pub name: Option<String>,
    /// The site's logo as a `data:image/png` URL, if any.
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    ConnectionRejected,
    PermissionGranted,
    PermissionDenied,
    PermissionRevoked,
    JobPrinted,
    JobFailed,
    RateLimited,
}

impl EventKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::ConnectionRejected => "connection_rejected",
            Self::PermissionGranted => "permission_granted",
            Self::PermissionDenied => "permission_denied",
            Self::PermissionRevoked => "permission_revoked",
            Self::JobPrinted => "job_printed",
            Self::JobFailed => "job_failed",
            Self::RateLimited => "rate_limited",
        }
    }
}

/// Groups of events the panel filters the history by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HistoryCategory {
    Prints,
    Errors,
    Permissions,
}

impl HistoryCategory {
    /// At most three kinds: [`Store::history`] has room for that many.
    fn kinds(self) -> &'static [EventKind] {
        match self {
            Self::Prints => &[EventKind::JobPrinted],
            Self::Errors => &[
                EventKind::JobFailed,
                EventKind::RateLimited,
                EventKind::ConnectionRejected,
            ],
            Self::Permissions => &[
                EventKind::PermissionGranted,
                EventKind::PermissionDenied,
                EventKind::PermissionRevoked,
            ],
        }
    }
}

#[derive(Debug, Default)]
pub struct HistoryFilter<'a> {
    /// Text to find in the site's origin or name, ignoring case.
    pub search: Option<&'a str>,
    pub category: Option<HistoryCategory>,
}

/// A history row as shown in the panel. Times are Unix seconds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub at: i64,
    pub kind: String,
    pub origin: Option<String>,
    /// The name the site gave itself, while it is in the site list.
    pub site_name: Option<String>,
    pub printer: Option<String>,
    pub job_name: Option<String>,
    pub bytes: Option<i64>,
    pub duration_ms: Option<i64>,
    pub error: Option<String>,
}

/// Details of a history row. Job contents are never stored: receipts carry personal data.
#[derive(Debug, Default)]
pub struct Event<'a> {
    pub origin: Option<&'a str>,
    pub printer: Option<&'a str>,
    pub job_name: Option<&'a str>,
    pub bytes: Option<usize>,
    pub duration: Option<Duration>,
    pub error: Option<&'a str>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("could not create the data directory: {0}")]
    Io(#[from] std::io::Error),
    #[error("the signing key is not available yet")]
    KeyUnavailable,
}

pub struct Store {
    connection: Mutex<Connection>,
    /// `None` until the credential store hands the key over (see `keys.rs`): meanwhile no
    /// decision verifies and none can be stored, so sites are asked for this session only.
    key: watch::Sender<Option<Key>>,
    opened_at: Instant,
}

impl std::fmt::Debug for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Store").finish_non_exhaustive()
    }
}

impl Store {
    pub fn open(path: &Path, key: Option<Key>) -> Result<Self, Error> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
            restrict(dir, 0o700)?;
        }
        let connection = Connection::open(path)?;
        restrict(path, 0o600)?;
        Self::init(connection, key)
    }

    #[cfg(test)]
    pub fn in_memory(key: Key) -> Self {
        Self::init(Connection::open_in_memory().unwrap(), Some(key)).unwrap()
    }

    fn init(mut connection: Connection, key: Option<Key>) -> Result<Self, Error> {
        connection.execute_batch(SCHEMA)?;
        migrate(&mut connection)?;
        // SQLite's own `lower()` only folds ASCII; names are searched in any language.
        connection.create_scalar_function(
            "unicode_lower",
            1,
            FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
            |context| {
                Ok(context
                    .get::<Option<String>>(0)?
                    .map(|text| text.to_lowercase()))
            },
        )?;
        Ok(Self {
            connection: Mutex::new(connection),
            key: watch::Sender::new(key),
            opened_at: Instant::now(),
        })
    }

    /// Provides the signing key once the credential store is reachable.
    pub fn set_key(&self, key: Key) {
        self.key.send_replace(Some(key));
    }

    /// Waits, during the first seconds after startup, for the signing key: the credential store is
    /// asked in the background, and until it answers no stored decision can be verified. Later
    /// calls (a store that stays unavailable) return at once.
    pub async fn key_ready(&self) {
        let remaining = KEY_GRACE.saturating_sub(self.opened_at.elapsed());
        let mut key = self.key.subscribe();
        let _ = tokio::time::timeout(remaining, key.wait_for(Option::is_some)).await;
    }

    /// The decision for `origin`, if a trustworthy one is stored.
    pub fn decision(&self, origin: &str) -> Result<Option<Decision>, Error> {
        Ok(self.site(origin)?.and_then(|site| site.decision))
    }

    pub fn site(&self, origin: &str) -> Result<Option<Site>, Error> {
        self.lock()
            .query_row(
                &format!("SELECT {SITE_COLUMNS} FROM sites WHERE origin = ?1"),
                params![origin],
                |row| self.site_from_row(row),
            )
            .optional()
            .map_err(Error::from)
    }

    /// Every stored site, most recently granted first.
    pub fn sites(&self) -> Result<Vec<Site>, Error> {
        let connection = self.lock();
        let mut statement = connection.prepare(&format!(
            "SELECT {SITE_COLUMNS} FROM sites ORDER BY granted_at DESC"
        ))?;
        let sites = statement
            .query_map([], |row| self.site_from_row(row))?
            .collect::<Result<_, _>>()?;
        Ok(sites)
    }

    /// Stores (or replaces) the user's decision for `origin`.
    pub fn set_decision(&self, origin: &str, decision: Decision) -> Result<(), Error> {
        let granted_at = now();
        let signature = self
            .sign(origin, decision.as_str(), granted_at, "prompt")
            .ok_or(Error::KeyUnavailable)?;
        self.lock().execute(
            "INSERT INTO sites (origin, decision, granted_at, granted_via, signature)
             VALUES (?1, ?2, ?3, 'prompt', ?4)
             ON CONFLICT (origin) DO UPDATE SET decision = ?2, granted_at = ?3,
                 granted_via = 'prompt', signature = ?4, last_used_at = NULL, job_count = 0",
            params![origin, decision.as_str(), granted_at, signature],
        )?;
        Ok(())
    }

    /// Updates how a stored site presents itself. Sites without a stored decision are skipped.
    pub fn set_identity(&self, origin: &str, identity: &Identity) -> Result<(), Error> {
        self.lock().execute(
            "UPDATE sites SET name = ?2, icon = ?3
             WHERE origin = ?1 AND (name IS NOT ?2 OR icon IS NOT ?3)",
            params![origin, identity.name, identity.icon],
        )?;
        Ok(())
    }

    pub fn remove(&self, origin: &str) -> Result<bool, Error> {
        Ok(self
            .lock()
            .execute("DELETE FROM sites WHERE origin = ?1", params![origin])?
            > 0)
    }

    /// Records a printed job for the site's usage statistics.
    pub fn touch(&self, origin: &str) -> Result<(), Error> {
        self.lock().execute(
            "UPDATE sites SET last_used_at = ?2, job_count = job_count + 1 WHERE origin = ?1",
            params![origin, now()],
        )?;
        Ok(())
    }

    pub fn record(&self, kind: EventKind, event: &Event<'_>) -> Result<(), Error> {
        let clip = |value: Option<&str>, max: usize| -> Option<String> {
            value.map(|text| text.chars().take(max).collect())
        };
        self.lock().execute(
            "INSERT INTO events (at, kind, origin, printer, job_name, bytes, duration_ms, error)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                now(),
                kind.as_str(),
                clip(event.origin, 512),
                clip(event.printer, 256),
                clip(event.job_name, 128),
                event.bytes.and_then(|bytes| i64::try_from(bytes).ok()),
                event
                    .duration
                    .and_then(|duration| i64::try_from(duration.as_millis()).ok()),
                clip(event.error, 512),
            ],
        )?;
        Ok(())
    }

    /// The most recent history rows that match `filter`, newest first.
    pub fn history(
        &self,
        filter: &HistoryFilter<'_>,
        limit: u32,
    ) -> Result<Vec<HistoryEntry>, Error> {
        // Origins are lowercase ASCII already; names go through `unicode_lower`.
        let search = filter
            .search
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(str::to_lowercase);
        let kinds = filter.category.map_or(&[][..], HistoryCategory::kinds);
        let kind = |index: usize| kinds.get(index).map(|kind| kind.as_str());
        let connection = self.lock();
        let mut statement = connection.prepare(
            "SELECT e.at, e.kind, e.origin, s.name, e.printer, e.job_name, e.bytes, e.duration_ms,
                    e.error
             FROM events e LEFT JOIN sites s ON s.origin = e.origin
             WHERE (?1 IS NULL OR instr(e.origin, ?1) > 0 OR instr(unicode_lower(s.name), ?1) > 0)
               AND (?3 IS NULL OR e.kind IN (?3, ?4, ?5))
             ORDER BY e.id DESC LIMIT ?2",
        )?;
        let entries = statement
            .query_map(params![search, limit, kind(0), kind(1), kind(2)], |row| {
                Ok(HistoryEntry {
                    at: row.get(0)?,
                    kind: row.get(1)?,
                    origin: row.get(2)?,
                    site_name: row.get(3)?,
                    printer: row.get(4)?,
                    job_name: row.get(5)?,
                    bytes: row.get(6)?,
                    duration_ms: row.get(7)?,
                    error: row.get(8)?,
                })
            })?
            .collect::<Result<_, _>>()?;
        Ok(entries)
    }

    /// Deletes history rows older than [`RETENTION`].
    pub fn purge_history(&self) -> Result<usize, Error> {
        let cutoff = now() - i64::try_from(RETENTION.as_secs()).unwrap_or(i64::MAX);
        Ok(self
            .lock()
            .execute("DELETE FROM events WHERE at < ?1", params![cutoff])?)
    }

    fn site_from_row(&self, row: &rusqlite::Row<'_>) -> rusqlite::Result<Site> {
        let origin: String = row.get(0)?;
        let decision: String = row.get(1)?;
        let granted_at: i64 = row.get(2)?;
        let granted_via: String = row.get(3)?;
        let signature: Vec<u8> = row.get(6)?;
        let trusted = self.verify(&origin, &decision, granted_at, &granted_via, &signature);
        let icon: Option<Vec<u8>> = row.get(8)?;
        Ok(Site {
            decision: Decision::parse(&decision).filter(|_| trusted),
            origin,
            granted_at,
            last_used_at: row.get(4)?,
            job_count: row.get(5)?,
            name: row.get(7)?,
            icon: icon.as_deref().map(identity::icon_data_url),
        })
    }

    /// `None` while the signing key is unavailable.
    fn mac(
        &self,
        origin: &str,
        decision: &str,
        granted_at: i64,
        granted_via: &str,
    ) -> Option<Hmac<Sha256>> {
        let key = (*self.key.borrow())?;
        let mut mac =
            Hmac::<Sha256>::new_from_slice(&key).expect("HMAC accepts keys of any length");
        mac.update(
            format!("nanoprint/site/v1\n{origin}\n{decision}\n{granted_at}\n{granted_via}")
                .as_bytes(),
        );
        Some(mac)
    }

    fn sign(
        &self,
        origin: &str,
        decision: &str,
        granted_at: i64,
        granted_via: &str,
    ) -> Option<Vec<u8>> {
        self.mac(origin, decision, granted_at, granted_via)
            .map(|mac| mac.finalize().into_bytes().to_vec())
    }

    fn verify(
        &self,
        origin: &str,
        decision: &str,
        granted_at: i64,
        granted_via: &str,
        signature: &[u8],
    ) -> bool {
        self.mac(origin, decision, granted_at, granted_via)
            .is_some_and(|mac| mac.verify_slice(signature).is_ok())
    }

    fn lock(&self) -> MutexGuard<'_, Connection> {
        // A panic mid-statement leaves SQLite consistent (statements are atomic).
        self.connection
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

fn migrate(connection: &mut Connection) -> Result<(), Error> {
    let applied: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    for (version, migration) in (1..)
        .zip(MIGRATIONS)
        .skip_while(|(version, _)| *version <= applied)
    {
        let transaction = connection.transaction()?;
        transaction.execute_batch(migration)?;
        transaction.pragma_update(None, "user_version", version)?;
        transaction.commit()?;
    }
    Ok(())
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|elapsed| i64::try_from(elapsed.as_secs()).ok())
        .unwrap_or_default()
}

/// Limits a path to the current user. On Windows the profile directory's ACLs already do.
#[cfg(unix)]
fn restrict(path: &Path, mode: u32) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
}

#[cfg(not(unix))]
#[allow(clippy::unnecessary_wraps)] // Same signature as the Unix version.
fn restrict(_path: &Path, _mode: u32) -> std::io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: Key = [7; 32];

    #[test]
    fn stores_and_reads_decisions() {
        let store = Store::in_memory(KEY);
        assert_eq!(store.decision("https://shop.example").unwrap(), None);
        store
            .set_decision("https://shop.example", Decision::Allow)
            .unwrap();
        store
            .set_decision("https://evil.example", Decision::Block)
            .unwrap();
        assert_eq!(
            store.decision("https://shop.example").unwrap(),
            Some(Decision::Allow)
        );
        assert_eq!(
            store.decision("https://evil.example").unwrap(),
            Some(Decision::Block)
        );
        assert!(store.remove("https://shop.example").unwrap());
        assert_eq!(store.decision("https://shop.example").unwrap(), None);
    }

    #[test]
    fn ignores_rows_written_outside_nanoprint() {
        let store = Store::in_memory(KEY);
        // Another program inserts an "allow" row without the key...
        store
            .lock()
            .execute(
                "INSERT INTO sites (origin, decision, granted_at, granted_via, signature)
                 VALUES ('https://attacker.example', 'allow', 1, 'prompt', x'00')",
                [],
            )
            .unwrap();
        // ...or flips a "block" into an "allow".
        store
            .set_decision("https://blocked.example", Decision::Block)
            .unwrap();
        store
            .lock()
            .execute(
                "UPDATE sites SET decision = 'allow' WHERE origin = 'https://blocked.example'",
                [],
            )
            .unwrap();
        assert_eq!(store.decision("https://attacker.example").unwrap(), None);
        assert_eq!(store.decision("https://blocked.example").unwrap(), None);
    }

    #[test]
    fn signatures_only_verify_with_the_same_key() {
        let signature = Store::in_memory(KEY)
            .sign("https://shop.example", "allow", 1, "prompt")
            .unwrap();
        let verify = |key| {
            Store::in_memory(key).verify("https://shop.example", "allow", 1, "prompt", &signature)
        };
        assert!(verify(KEY));
        assert!(!verify([8; 32]));
    }

    #[test]
    fn tracks_usage_and_purges_old_history() {
        let store = Store::in_memory(KEY);
        store
            .set_decision("https://shop.example", Decision::Allow)
            .unwrap();
        store.touch("https://shop.example").unwrap();
        store.touch("https://shop.example").unwrap();
        let site = store.site("https://shop.example").unwrap().unwrap();
        assert_eq!(site.job_count, 2);
        assert!(site.last_used_at.is_some());

        let event = Event {
            origin: Some("https://shop.example"),
            printer: Some("POS-58"),
            bytes: Some(130),
            ..Event::default()
        };
        store.record(EventKind::JobPrinted, &event).unwrap();
        store
            .lock()
            .execute("UPDATE events SET at = at - 31 * 24 * 60 * 60", [])
            .unwrap();
        store.record(EventKind::JobPrinted, &event).unwrap();
        assert_eq!(store.purge_history().unwrap(), 1);
    }

    #[test]
    fn filters_history_by_site_and_category() {
        let store = Store::in_memory(KEY);
        store
            .set_decision("https://a.example", Decision::Allow)
            .unwrap();
        let kiosco = Identity {
            name: Some("ÑANDÚ Kiosco".into()),
            icon: None,
        };
        store.set_identity("https://a.example", &kiosco).unwrap();
        for (kind, origin) in [
            (EventKind::JobPrinted, "https://a.example"),
            (EventKind::JobPrinted, "https://b.example"),
            (EventKind::JobFailed, "https://a.example"),
            (EventKind::PermissionRevoked, "https://b.example"),
        ] {
            let event = Event {
                origin: Some(origin),
                ..Event::default()
            };
            store.record(kind, &event).unwrap();
        }
        store
            .record(EventKind::ConnectionRejected, &Event::default())
            .unwrap();

        let history = |search, category| {
            store
                .history(&HistoryFilter { search, category }, 10)
                .unwrap()
        };
        let all = history(None, None);
        assert_eq!(all.len(), 5);
        assert_eq!(all[0].kind, "connection_rejected");
        assert_eq!(all[2].site_name.as_deref(), Some("ÑANDÚ Kiosco"));
        // By origin or by name, ignoring case (beyond ASCII too) and surrounding spaces.
        assert_eq!(history(Some("B.EXAMPLE"), None).len(), 2);
        assert_eq!(history(Some(" ñandú "), None).len(), 2);
        assert_eq!(history(Some("ÑANDÚ"), None).len(), 2);
        assert_eq!(history(Some(""), None).len(), 5);
        assert_eq!(history(None, Some(HistoryCategory::Prints)).len(), 2);
        assert_eq!(history(None, Some(HistoryCategory::Errors)).len(), 2);
        assert_eq!(history(None, Some(HistoryCategory::Permissions)).len(), 1);
        assert_eq!(
            history(Some("a.example"), Some(HistoryCategory::Errors)).len(),
            1
        );
        assert_eq!(
            store.history(&HistoryFilter::default(), 1).unwrap().len(),
            1
        );
    }

    #[test]
    fn waits_for_the_signing_key() {
        let store = Store::in_memory(KEY);
        store
            .set_decision("https://shop.example", Decision::Allow)
            .unwrap();
        // The same database while the credential store is still locked...
        let connection =
            std::mem::replace(&mut *store.lock(), Connection::open_in_memory().unwrap());
        let locked = Store::init(connection, None).unwrap();
        assert_eq!(locked.decision("https://shop.example").unwrap(), None);
        assert!(matches!(
            locked.set_decision("https://other.example", Decision::Allow),
            Err(Error::KeyUnavailable)
        ));
        // ...and once it hands the key over.
        locked.set_key(KEY);
        assert_eq!(
            locked.decision("https://shop.example").unwrap(),
            Some(Decision::Allow)
        );
    }

    #[test]
    fn keeps_how_stored_sites_present_themselves() {
        let store = Store::in_memory(KEY);
        let identity = Identity {
            name: Some("Kiosco".into()),
            icon: Some(vec![1, 2, 3]),
        };
        // Sites without a decision are not kept.
        store
            .set_identity("https://shop.example", &identity)
            .unwrap();
        assert_eq!(store.sites().unwrap(), []);

        store
            .set_decision("https://shop.example", Decision::Allow)
            .unwrap();
        store
            .set_identity("https://shop.example", &identity)
            .unwrap();
        let site = store.site("https://shop.example").unwrap().unwrap();
        assert_eq!(site.name.as_deref(), Some("Kiosco"));
        assert_eq!(site.icon.as_deref(), Some("data:image/png;base64,AQID"));
        assert_eq!(site.decision, Some(Decision::Allow));
    }

    #[test]
    fn upgrades_databases_from_earlier_versions() {
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch(SCHEMA).unwrap();
        let store = Store::init(connection, Some(KEY)).unwrap();
        store
            .set_decision("https://shop.example", Decision::Allow)
            .unwrap();
        let version: i64 = store
            .lock()
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, i64::try_from(MIGRATIONS.len()).unwrap());
        // Opening again applies nothing twice.
        let reopened = Store::init(
            std::mem::replace(&mut *store.lock(), Connection::open_in_memory().unwrap()),
            Some(KEY),
        )
        .unwrap();
        assert_eq!(
            reopened.decision("https://shop.example").unwrap(),
            Some(Decision::Allow)
        );
    }
}
