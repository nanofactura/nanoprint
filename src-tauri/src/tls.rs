//! Local TLS for `wss://`: a certificate for `localhost`, `127.0.0.1` and `::1`, issued by a
//! certificate authority unique to this installation.
//!
//! The authority's private key only exists in memory while it signs the certificate and is then
//! dropped, so nobody can use the trusted authority to mint certificates for other sites. Its
//! certificate also carries name constraints limiting it to loopback names.

use std::{
    fs, io,
    path::Path,
    sync::{Arc, PoisonError, RwLock},
};

use rcgen::{
    BasicConstraints, CertificateParams, CidrSubnet, DnType, ExtendedKeyUsagePurpose,
    GeneralSubtree, IsCa, Issuer, KeyPair, KeyUsagePurpose, NameConstraints, SerialNumber,
};
use rustls::{
    crypto::CryptoProvider,
    pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
    server::{ClientHello, ResolvesServerCert},
    sign::CertifiedKey,
};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

/// Apple rejects TLS server certificates valid for more than 825 days.
const VALIDITY: Duration = Duration::days(800);
/// Certificates are replaced this long before they expire.
const RENEW_BEFORE: Duration = Duration::days(30);

const CA_FILE: &str = "ca.pem";
const CERT_FILE: &str = "localhost.pem";
const KEY_FILE: &str = "localhost-key.pem";
const META_FILE: &str = "certificate.json";
/// Authorities replaced by a renewal that may still be in the trust store.
const REPLACED_FILE: &str = "replaced.json";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not create the certificate: {0}")]
    Generate(#[from] rcgen::Error),
    #[error("could not read or write the certificate: {0}")]
    Io(#[from] io::Error),
    #[error("invalid certificate files: {0}")]
    Pem(#[from] rustls::pki_types::pem::Error),
    #[error("invalid TLS configuration: {0}")]
    Tls(#[from] rustls::Error),
    #[error("invalid certificate metadata: {0}")]
    Meta(#[from] serde_json::Error),
}

/// What other modules need to know about the installed certificate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    /// Common name of this installation's authority, used to find it in trust stores.
    pub ca_name: String,
    /// Serial number of the authority certificate, in hexadecimal.
    pub ca_serial: String,
    /// Unix seconds.
    pub expires_at: i64,
}

/// The certificate files in `dir`.
#[derive(Debug, Clone)]
pub struct Certificate {
    pub meta: Meta,
    pub ca_path: std::path::PathBuf,
    // Used by the macOS trust check.
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    pub cert_path: std::path::PathBuf,
    cert_pem: String,
    key_pem: String,
}

/// Loads the certificate from `dir`, creating or renewing it when needed. Also returns every
/// authority replaced by a renewal and not yet removed from the trust store (see
/// [`forget_replaced`]); they are recorded on disk before being replaced, so a restart or a failed
/// removal never leaves one behind for good.
pub fn load_or_create(dir: &Path) -> Result<(Certificate, Vec<Meta>), Error> {
    let existing = load(dir).ok();
    let certificate = match existing {
        Some(certificate) if !needs_renewal(&certificate.meta) => certificate,
        existing => {
            if let Some(old) = existing {
                let mut replaced = replaced(dir);
                replaced.push(old.meta);
                save_replaced(dir, &replaced)?;
            }
            save(dir, &generate()?)?;
            load(dir)?
        }
    };
    Ok((certificate, replaced(dir)))
}

/// Drops replaced authorities that left the trust store.
pub fn forget_replaced(dir: &Path, removed: &[Meta]) -> Result<(), Error> {
    let mut replaced = replaced(dir);
    replaced.retain(|meta| !removed.contains(meta));
    save_replaced(dir, &replaced)
}

/// Every authority of this installation that may be in a trust store: the current one and those
/// replaced by renewals. Reads only; for `nanoprint --uninstall`.
pub fn authorities(dir: &Path) -> Vec<Meta> {
    let mut authorities = replaced(dir);
    if let Ok(certificate) = load(dir) {
        authorities.push(certificate.meta);
    }
    authorities
}

/// Whether the certificate is within [`RENEW_BEFORE`] of expiring (or past it).
pub fn needs_renewal(meta: &Meta) -> bool {
    meta.expires_at - OffsetDateTime::now_utc().unix_timestamp() <= RENEW_BEFORE.whole_seconds()
}

/// Serves the current certificate; a renewal replaces it while the server keeps running (a POS
/// computer may never restart NanoPrint).
#[derive(Debug)]
pub struct CertificateResolver(RwLock<Arc<CertifiedKey>>);

impl CertificateResolver {
    pub fn new(certificate: &Certificate) -> Result<Arc<Self>, Error> {
        Ok(Arc::new(Self(RwLock::new(certified_key(certificate)?))))
    }

    pub fn replace(&self, certificate: &Certificate) -> Result<(), Error> {
        let key = certified_key(certificate)?;
        *self.0.write().unwrap_or_else(PoisonError::into_inner) = key;
        Ok(())
    }
}

impl ResolvesServerCert for CertificateResolver {
    fn resolve(&self, _client_hello: ClientHello<'_>) -> Option<Arc<CertifiedKey>> {
        Some(Arc::clone(
            &self.0.read().unwrap_or_else(PoisonError::into_inner),
        ))
    }
}

/// A rustls configuration serving the resolver's certificate (TLS 1.3 only, `ring` provider).
pub fn server_config(
    resolver: Arc<CertificateResolver>,
) -> Result<Arc<rustls::ServerConfig>, Error> {
    let config = rustls::ServerConfig::builder_with_provider(provider())
        .with_protocol_versions(&[&rustls::version::TLS13])?
        .with_no_client_auth()
        .with_cert_resolver(resolver);
    Ok(Arc::new(config))
}

fn provider() -> Arc<CryptoProvider> {
    Arc::new(rustls::crypto::ring::default_provider())
}

fn certified_key(certificate: &Certificate) -> Result<Arc<CertifiedKey>, Error> {
    let chain = CertificateDer::pem_slice_iter(certificate.cert_pem.as_bytes())
        .collect::<Result<Vec<_>, _>>()?;
    let key = PrivateKeyDer::from_pem_slice(certificate.key_pem.as_bytes())?;
    Ok(Arc::new(CertifiedKey::from_der(chain, key, &provider())?))
}

fn replaced(dir: &Path) -> Vec<Meta> {
    match fs::read_to_string(dir.join(REPLACED_FILE)) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_else(|error| {
            log::warn!("ignoring an unreadable list of replaced authorities: {error}");
            Vec::new()
        }),
        Err(_) => Vec::new(),
    }
}

fn save_replaced(dir: &Path, replaced: &[Meta]) -> Result<(), Error> {
    let path = dir.join(REPLACED_FILE);
    if replaced.is_empty() {
        return match fs::remove_file(path) {
            Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error.into()),
            _ => Ok(()),
        };
    }
    fs::create_dir_all(dir)?;
    write(&path, &serde_json::to_string_pretty(replaced)?, false)?;
    Ok(())
}

struct Generated {
    meta: Meta,
    ca_pem: String,
    cert_pem: String,
    key_pem: String,
}

fn generate() -> Result<Generated, Error> {
    let now = OffsetDateTime::now_utc();
    // A day of slack for computers whose clock runs a little behind.
    let not_before = now - Duration::days(1);
    let not_after = now + VALIDITY;
    let serial = random_serial();
    let ca_name = format!("NanoPrint Local CA {}", &hex::encode(serial)[..8]);

    let mut ca = CertificateParams::default();
    ca.distinguished_name
        .push(DnType::CommonName, ca_name.clone());
    ca.distinguished_name
        .push(DnType::OrganizationName, "NanoPrint");
    ca.is_ca = IsCa::Ca(BasicConstraints::Constrained(0));
    ca.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
    ca.name_constraints = Some(NameConstraints {
        permitted_subtrees: vec![
            GeneralSubtree::DnsName("localhost".into()),
            GeneralSubtree::IpAddress(CidrSubnet::V4([127, 0, 0, 1], [255; 4])),
            GeneralSubtree::IpAddress(CidrSubnet::V6(
                std::net::Ipv6Addr::LOCALHOST.octets(),
                [255; 16],
            )),
        ],
        excluded_subtrees: Vec::new(),
    });
    ca.serial_number = Some(SerialNumber::from_slice(&serial));
    ca.not_before = not_before;
    ca.not_after = not_after;
    let ca_key = KeyPair::generate()?;
    let ca_pem = ca.self_signed(&ca_key)?.pem();
    // Owns the authority's private key; dropped at the end of this function, never saved.
    let issuer = Issuer::new(ca, ca_key);

    let mut leaf = CertificateParams::new(vec![
        "localhost".to_owned(),
        "127.0.0.1".to_owned(),
        "::1".to_owned(),
    ])?;
    leaf.distinguished_name
        .push(DnType::CommonName, "localhost");
    leaf.is_ca = IsCa::ExplicitNoCa;
    leaf.key_usages = vec![KeyUsagePurpose::DigitalSignature];
    leaf.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    leaf.use_authority_key_identifier_extension = true;
    leaf.serial_number = Some(SerialNumber::from_slice(&random_serial()));
    leaf.not_before = not_before;
    leaf.not_after = not_after;
    let leaf_key = KeyPair::generate()?;
    let cert_pem = leaf.signed_by(&leaf_key, &issuer)?.pem();

    Ok(Generated {
        meta: Meta {
            ca_name,
            ca_serial: hex::encode(serial),
            expires_at: not_after.unix_timestamp(),
        },
        ca_pem,
        cert_pem,
        key_pem: leaf_key.serialize_pem(),
    })
}

fn save(dir: &Path, generated: &Generated) -> Result<(), Error> {
    fs::create_dir_all(dir)?;
    write(&dir.join(KEY_FILE), &generated.key_pem, true)?;
    write(&dir.join(CERT_FILE), &generated.cert_pem, false)?;
    write(&dir.join(CA_FILE), &generated.ca_pem, false)?;
    write(
        &dir.join(META_FILE),
        &serde_json::to_string_pretty(&generated.meta)?,
        false,
    )?;
    Ok(())
}

fn load(dir: &Path) -> Result<Certificate, Error> {
    let meta = serde_json::from_str(&fs::read_to_string(dir.join(META_FILE))?)?;
    Ok(Certificate {
        meta,
        ca_path: dir.join(CA_FILE),
        cert_path: dir.join(CERT_FILE),
        cert_pem: fs::read_to_string(dir.join(CERT_FILE))?,
        key_pem: fs::read_to_string(dir.join(KEY_FILE))?,
    })
}

/// Writes `contents`; private files are readable by the current user only on macOS and Linux
/// (the profile directory's ACLs cover Windows).
fn write(path: &Path, contents: &str, private: bool) -> io::Result<()> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    if private {
        std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    }
    #[cfg(not(unix))]
    let _ = private;
    io::Write::write_all(&mut options.open(path)?, contents.as_bytes())
}

fn random_serial() -> [u8; 16] {
    let mut serial = [0; 16];
    getrandom::fill(&mut serial).expect("the operating system's random generator must work");
    // Positive and without a leading zero byte, as X.509 serial numbers should be.
    serial[0] = (serial[0] & 0x7f) | 0x01;
    serial
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("nanoprint-tls-{name}-{}", std::process::id()))
    }

    #[test]
    fn creates_reuses_and_serves_a_certificate() {
        let dir = temp_dir("create");
        let (first, replaced) = load_or_create(&dir).unwrap();
        assert_eq!(replaced, []);
        assert!(first.meta.ca_name.starts_with("NanoPrint Local CA "));
        assert!(server_config(CertificateResolver::new(&first).unwrap()).is_ok());
        // The authority's private key is never written.
        let files: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect();
        assert_eq!(files.len(), 4);
        assert!(
            !fs::read_to_string(dir.join(CA_FILE))
                .unwrap()
                .contains("PRIVATE KEY")
        );

        let (again, replaced) = load_or_create(&dir).unwrap();
        assert_eq!(replaced, []);
        assert_eq!(again.meta, first.meta);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn renews_certificates_close_to_expiry() {
        let dir = temp_dir("renew");
        let (first, _) = load_or_create(&dir).unwrap();
        let mut meta = first.meta.clone();
        meta.expires_at = OffsetDateTime::now_utc().unix_timestamp() + 60;
        fs::write(dir.join(META_FILE), serde_json::to_string(&meta).unwrap()).unwrap();

        let (renewed, replaced) = load_or_create(&dir).unwrap();
        assert_eq!(replaced, [meta.clone()]);
        assert_ne!(renewed.meta.ca_serial, first.meta.ca_serial);
        // Still listed after a restart, until it is removed from the trust store.
        assert_eq!(load_or_create(&dir).unwrap().1, [meta.clone()]);
        forget_replaced(&dir, &[meta]).unwrap();
        assert_eq!(load_or_create(&dir).unwrap().1, []);
        assert!(!dir.join(REPLACED_FILE).exists());
        fs::remove_dir_all(dir).unwrap();
    }
}
