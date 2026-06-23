//! # `did.rs` — standard Ed25519 `did:key` / `did:web` resolution
//!
//! Resolves a W3C [`did:key`]/[`did:web`] DID (or a fragment-bearing
//! `verificationMethod` IRI) to a **standard Ed25519** [`VerifyingKey`] — the key
//! the `eddsa-rdfc-2022` cryptosuite checks signatures against.
//!
//! [`did:key`]: https://w3c-ccg.github.io/did-method-key/
//! [`did:web`]: https://w3c-ccg.github.io/did-method-web/
//!
//! ## How this differs from `sparq-trust::did` (deliberately)
//!
//! `sparq-trust::did` resolves to a **Baby-JubJub** key under a *sparq-private*
//! multicodec, because the ZK estate's issuer signature is Schnorr-over-Baby-JubJub
//! and it explicitly **rejects** the standard Ed25519 `z6Mk…` form. This module is
//! the opposite: it is the **W3C-interoperable** resolver, so it accepts exactly
//! the standard Ed25519 multicodec (`0xed01`, the `z6Mk…` form) and rejects
//! everything else. The two coexist; they serve the two distinct trust models
//! (vetted-interop here, custom-ZK there).
//!
//! ## Two methods, two trust models (be precise about which you get)
//!
//! - **`did:key`** ([`DidKeyResolver`]) — **self-certifying, offline, no network.**
//!   The Ed25519 public key is encoded IN the DID identifier (multibase `z`-base58btc
//!   over the `0xed01`-prefixed 32-byte point), so "resolving" is a local decode: the
//!   DID *is* the key. What it buys is a stable, round-trippable, method-tagged
//!   identifier — **not** a stronger root of trust (whoever can choose the DID can
//!   choose the key).
//! - **`did:web`** (`DidWebResolver`, `did-web` feature) — **document-fetched,
//!   host-rooted.** The DID maps to an `https://` URL serving a DID document; the key
//!   is read from a verification method. The root of trust is **whoever controls the
//!   host + its TLS**. The HTTP fetch is **not** performed here — the caller supplies
//!   a `DidDocumentFetcher` (a `did-web`-feature item), so this crate ships no HTTP
//!   client and stays
//!   offline-testable.

use ed25519_dalek::VerifyingKey;

/// The multicodec code for an Ed25519 public key (unsigned-varint `0xed01`). This is
/// the **standard** W3C `did:key` Ed25519 prefix — the `z6Mk…` form — i.e. byte
/// sequence `0xed 0x01` followed by the 32-byte compressed point.
pub const MULTICODEC_ED25519_PUB: u64 = 0xed;

/// A parsed DID: its method (`key` / `web`) and method-specific id, with any
/// `verificationMethod` fragment (`#…`) stripped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Did {
    /// The DID method, e.g. `key` or `web`.
    pub method: String,
    /// The method-specific identifier (no `did:<method>:` prefix, no `#fragment`).
    pub method_specific_id: String,
}

impl Did {
    /// Parse a DID string or a `verificationMethod` IRI. A trailing `#fragment`
    /// (the verification-method id) is stripped — the DID it refers to is what
    /// resolves. Returns `None` if the string is not `did:<method>:<id>`.
    pub fn parse(s: &str) -> Option<Did> {
        let rest = s.strip_prefix("did:")?;
        // Strip a verificationMethod fragment if present.
        let rest = rest.split('#').next().unwrap_or(rest);
        let (method, id) = rest.split_once(':')?;
        if method.is_empty() || id.is_empty() {
            return None;
        }
        Some(Did {
            method: method.to_string(),
            method_specific_id: id.to_string(),
        })
    }

    /// Reconstruct the bare DID string (`did:<method>:<id>`, no fragment).
    pub fn to_did_string(&self) -> String {
        // Positional args (CodeQL rust/unused-variable false-positive avoidance).
        format!("did:{}:{}", self.method, self.method_specific_id)
    }
}

/// DID resolution failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DidError {
    /// The string did not parse as `did:<method>:<id>`.
    Malformed(String),
    /// The DID method is not supported by this resolver.
    UnsupportedMethod(String),
    /// The `did:key` / document key material did not decode to a valid Ed25519 key
    /// (bad multibase, base58, multicodec, or point length).
    BadKeyMaterial(String),
    /// A `did:web` document fetch failed (the pluggable fetcher returned an error).
    FetchFailed(String),
    /// A `did:web` document carried no usable Ed25519 verification method.
    NoIssuerKey(String),
}

impl std::fmt::Display for DidError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DidError::Malformed(s) => write!(f, "malformed DID: {}", s),
            DidError::UnsupportedMethod(m) => write!(f, "unsupported DID method: {}", m),
            DidError::BadKeyMaterial(s) => write!(f, "bad Ed25519 key material: {}", s),
            DidError::FetchFailed(e) => write!(f, "did:web document fetch failed: {}", e),
            DidError::NoIssuerKey(s) => write!(f, "no usable Ed25519 verification method: {}", s),
        }
    }
}

impl std::error::Error for DidError {}

/// A pluggable DID resolver: map a DID to the Ed25519 [`VerifyingKey`] the
/// `eddsa-rdfc-2022` verify checks against.
pub trait DidResolver {
    /// Resolve a parsed [`Did`] to its Ed25519 verifying key.
    fn resolve(&self, did: &Did) -> Result<VerifyingKey, DidError>;

    /// Resolve a DID string or `verificationMethod` IRI directly.
    fn resolve_str(&self, s: &str) -> Result<VerifyingKey, DidError> {
        let did = Did::parse(s).ok_or_else(|| DidError::Malformed(s.to_string()))?;
        self.resolve(&did)
    }
}

/// The offline `did:key` resolver: decode the multibase/multicodec key material in
/// the DID identifier to an Ed25519 [`VerifyingKey`]. No network, no document — the
/// DID *is* the key. Only the standard Ed25519 multicodec (`0xed01`, the `z6Mk…`
/// form) is accepted; any other codec fails closed with [`DidError::BadKeyMaterial`].
pub struct DidKeyResolver;

impl DidResolver for DidKeyResolver {
    fn resolve(&self, did: &Did) -> Result<VerifyingKey, DidError> {
        if did.method != "key" {
            return Err(DidError::UnsupportedMethod(did.method.clone()));
        }
        decode_did_key_id(&did.method_specific_id)
    }
}

/// Mint the canonical `did:key` string for an Ed25519 [`VerifyingKey`]: multibase
/// (`z`-base58btc) over the `0xed01`-multicodec-prefixed 32-byte point. Round-trips
/// with [`DidKeyResolver`].
pub fn did_key_for(vk: &VerifyingKey) -> String {
    let mut payload = encode_varint(MULTICODEC_ED25519_PUB);
    payload.extend_from_slice(vk.as_bytes());
    // Positional args (CodeQL rust/unused-variable false-positive avoidance).
    format!("did:key:z{}", bs58::encode(payload).into_string())
}

/// Decode a `did:key` method-specific id (the `z6Mk…` string) to an Ed25519
/// [`VerifyingKey`]. Fail-closed on a wrong multibase prefix, bad base58, the wrong
/// multicodec, or a non-32-byte point.
pub(crate) fn decode_did_key_id(id: &str) -> Result<VerifyingKey, DidError> {
    let b58 = id
        .strip_prefix('z')
        .ok_or_else(|| DidError::BadKeyMaterial(format!("not multibase-z: {}", id)))?;
    let bytes = bs58::decode(b58)
        .into_vec()
        .map_err(|e| DidError::BadKeyMaterial(format!("base58: {}", e)))?;
    let (code, rest) = decode_varint(&bytes)
        .ok_or_else(|| DidError::BadKeyMaterial("truncated multicodec varint".to_string()))?;
    if code != MULTICODEC_ED25519_PUB {
        return Err(DidError::BadKeyMaterial(format!(
            "unsupported multicodec 0x{:x} (sparq-vc uses Ed25519 0x{:x})",
            code, MULTICODEC_ED25519_PUB
        )));
    }
    let arr: [u8; 32] = rest
        .try_into()
        .map_err(|_| DidError::BadKeyMaterial(format!("expected 32-byte Ed25519 point: {}", id)))?;
    VerifyingKey::from_bytes(&arr)
        .map_err(|e| DidError::BadKeyMaterial(format!("not a valid Ed25519 point: {}", e)))
}

// --- did:web (document-fetched, pluggable fetcher) --------------------------

/// A pluggable fetcher for a `did:web` DID document. The resolver hands it the
/// resolved `https://…/did.json` URL; the implementation returns the document bytes
/// (UTF-8 JSON) or an error. This is the **network seam** — the crate ships no HTTP
/// client of its own, so the build forces no `reqwest`/`ureq` onto anyone and the
/// resolver stays offline-testable (an in-memory map is a valid fetcher for tests /
/// a pre-cached registry).
#[cfg(feature = "did-web")]
#[cfg_attr(docsrs, doc(cfg(feature = "did-web")))]
pub trait DidDocumentFetcher {
    /// Fetch the DID document at `url` (always an `https://` URL ending in
    /// `did.json`). Return the raw UTF-8 JSON, or a human-readable error string
    /// (mapped to [`DidError::FetchFailed`]).
    fn fetch(&self, url: &str) -> Result<String, String>;
}

/// The `did:web` resolver over a pluggable [`DidDocumentFetcher`]. Maps the DID to
/// its `https://` document URL (W3C `did:web` method), fetches it, and extracts the
/// first standard Ed25519 verification method.
#[cfg(feature = "did-web")]
#[cfg_attr(docsrs, doc(cfg(feature = "did-web")))]
pub struct DidWebResolver<F: DidDocumentFetcher> {
    fetcher: F,
}

#[cfg(feature = "did-web")]
impl<F: DidDocumentFetcher> DidWebResolver<F> {
    /// Build a `did:web` resolver over the given fetcher.
    pub fn new(fetcher: F) -> Self {
        DidWebResolver { fetcher }
    }

    /// Map a `did:web` method-specific id to its document URL per the method spec:
    /// the authority (with an optional `%3A`-encoded port) and `:`-separated path
    /// segments become a `/`-separated path. A bare-host DID (`did:web:example.com`)
    /// resolves to `https://example.com/.well-known/did.json`; a pathful DID
    /// (`did:web:example.com:user:alice`) resolves to
    /// `https://example.com/user/alice/did.json`. Returns `None` for an
    /// empty/invalid id.
    pub fn document_url(id: &str) -> Option<String> {
        if id.is_empty() {
            return None;
        }
        let mut parts = id.split(':');
        let authority = parts.next()?;
        if authority.is_empty() {
            return None;
        }
        // `%3A` in the authority is a port separator per the spec.
        let authority = authority.replace("%3A", ":");
        let path_segments: Vec<&str> = parts.collect();
        if path_segments.iter().any(|s| s.is_empty()) {
            return None; // a `::` empty path segment is malformed
        }
        if path_segments.is_empty() {
            // Positional args (CodeQL rust/unused-variable false-positive avoidance).
            Some(format!("https://{}/.well-known/did.json", authority))
        } else {
            Some(format!(
                "https://{}/{}/did.json",
                authority,
                path_segments.join("/")
            ))
        }
    }
}

#[cfg(feature = "did-web")]
impl<F: DidDocumentFetcher> DidResolver for DidWebResolver<F> {
    fn resolve(&self, did: &Did) -> Result<VerifyingKey, DidError> {
        if did.method != "web" {
            return Err(DidError::UnsupportedMethod(did.method.clone()));
        }
        let url = Self::document_url(&did.method_specific_id)
            .ok_or_else(|| DidError::Malformed(did.to_did_string()))?;
        let doc = self.fetcher.fetch(&url).map_err(DidError::FetchFailed)?;
        extract_ed25519_key_from_doc(&doc, &did.to_did_string())
    }
}

/// Extract the first standard Ed25519 [`VerifyingKey`] from a `did:web` DID
/// document (JSON). The document must carry a verification method whose
/// `publicKeyMultibase` is the standard `did:key`-style `z`-base58btc/`0xed01`
/// encoding. The first verification method that decodes to a valid Ed25519 key
/// wins; if none do, this fails closed with [`DidError::NoIssuerKey`].
#[cfg(feature = "did-web")]
fn extract_ed25519_key_from_doc(doc: &str, did: &str) -> Result<VerifyingKey, DidError> {
    let value: serde_json::Value =
        serde_json::from_str(doc).map_err(|e| DidError::NoIssuerKey(format!("{}: {}", did, e)))?;
    for m in collect_verification_methods(&value) {
        if let Some(mb) = m.get("publicKeyMultibase").and_then(|v| v.as_str()) {
            if let Ok(vk) = decode_did_key_id(mb) {
                return Ok(vk);
            }
        }
    }
    Err(DidError::NoIssuerKey(did.to_string()))
}

/// Collect candidate verification-method JSON objects from a DID document: the
/// embedded objects under `verificationMethod` plus any embedded under
/// `assertionMethod`.
#[cfg(feature = "did-web")]
fn collect_verification_methods(doc: &serde_json::Value) -> Vec<&serde_json::Value> {
    let mut out = Vec::new();
    for key in ["verificationMethod", "assertionMethod"] {
        if let Some(arr) = doc.get(key).and_then(|v| v.as_array()) {
            for m in arr {
                if m.is_object() {
                    out.push(m);
                }
            }
        }
    }
    out
}

// --- minimal multicodec varint (no extra deps — lean-core rule) -------------

/// Encode an unsigned-LEB128 varint (the multicodec encoding).
fn encode_varint(mut v: u64) -> Vec<u8> {
    let mut out = Vec::new();
    loop {
        let mut byte = (v & 0x7f) as u8;
        v >>= 7;
        if v != 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if v == 0 {
            break;
        }
    }
    out
}

/// Decode an unsigned-LEB128 varint from the front of `bytes`; returns
/// `(value, rest)`. `None` on a truncated or over-long (> 64-bit) varint
/// (fail-closed).
fn decode_varint(bytes: &[u8]) -> Option<(u64, &[u8])> {
    let mut value: u64 = 0;
    let mut shift = 0u32;
    for (i, &b) in bytes.iter().enumerate() {
        if shift >= 64 {
            return None; // over-long
        }
        value |= ((b & 0x7f) as u64) << shift;
        if b & 0x80 == 0 {
            return Some((value, &bytes[i + 1..]));
        }
        shift += 7;
    }
    None // truncated
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey as DalekSigningKey;

    fn test_vk() -> VerifyingKey {
        // Deterministic key for a stable did:key string in the assertions below.
        let sk = DalekSigningKey::from_bytes(&[7u8; 32]);
        sk.verifying_key()
    }

    #[test]
    fn did_key_round_trip() {
        let vk = test_vk();
        let did = did_key_for(&vk);
        assert!(
            did.starts_with("did:key:z6Mk"),
            "Ed25519 did:key form: {did}"
        );
        let parsed = Did::parse(&did).unwrap();
        assert_eq!(parsed.method, "key");
        let resolved = DidKeyResolver.resolve(&parsed).unwrap();
        assert_eq!(resolved.as_bytes(), vk.as_bytes());
    }

    /// Interop sanity: a published W3C did:key Ed25519 example
    /// (did-method-key spec / DID test vectors) decodes to a valid Ed25519 key and
    /// re-encodes byte-identically — proving this crate's multibase/multicodec
    /// handling matches the standard `z6Mk…` form (NOT a sparq-private codec).
    #[test]
    fn decodes_published_w3c_did_key_vector() {
        let published = "z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
        let vk = decode_did_key_id(published).expect("published vector decodes");
        let reencoded = did_key_for(&vk);
        assert_eq!(reencoded, format!("did:key:{published}"));
    }

    #[test]
    fn resolve_str_strips_fragment() {
        let vk = test_vk();
        let did = did_key_for(&vk);
        let vm = format!("{did}#{}", did.strip_prefix("did:key:").unwrap());
        let resolved = DidKeyResolver.resolve_str(&vm).unwrap();
        assert_eq!(resolved.as_bytes(), vk.as_bytes());
    }

    #[test]
    fn rejects_non_ed25519_multicodec() {
        // 0xec01 is X25519 (a key-agreement key), not Ed25519 — must fail closed.
        let mut payload = encode_varint(0xec);
        payload.extend_from_slice(&[0u8; 32]);
        let id = format!("z{}", bs58::encode(payload).into_string());
        let err = decode_did_key_id(&id).unwrap_err();
        assert!(matches!(err, DidError::BadKeyMaterial(_)), "{err:?}");
    }

    #[test]
    fn rejects_malformed_did() {
        assert!(Did::parse("not-a-did").is_none());
        assert!(Did::parse("did:key:").is_none());
        assert!(Did::parse("did::z6Mk").is_none());
    }

    #[cfg(feature = "did-web")]
    #[test]
    fn did_web_document_url_mapping() {
        assert_eq!(
            DidWebResolver::<DummyFetcher>::document_url("example.com").unwrap(),
            "https://example.com/.well-known/did.json"
        );
        assert_eq!(
            DidWebResolver::<DummyFetcher>::document_url("example.com:user:alice").unwrap(),
            "https://example.com/user/alice/did.json"
        );
        assert_eq!(
            DidWebResolver::<DummyFetcher>::document_url("example.com%3A8443").unwrap(),
            "https://example.com:8443/.well-known/did.json"
        );
        assert!(DidWebResolver::<DummyFetcher>::document_url("example.com::x").is_none());
    }

    #[cfg(feature = "did-web")]
    struct DummyFetcher(String);
    #[cfg(feature = "did-web")]
    impl DidDocumentFetcher for DummyFetcher {
        fn fetch(&self, _url: &str) -> Result<String, String> {
            Ok(self.0.clone())
        }
    }

    #[cfg(feature = "did-web")]
    #[test]
    fn did_web_extracts_ed25519_multibase() {
        let vk = test_vk();
        let mb = did_key_for(&vk)
            .strip_prefix("did:key:")
            .unwrap()
            .to_string();
        let doc = format!(
            r#"{{"id":"did:web:example.com","verificationMethod":[
                {{"id":"did:web:example.com#k","type":"Ed25519VerificationKey2020",
                  "publicKeyMultibase":"{mb}"}}]}}"#
        );
        let resolver = DidWebResolver::new(DummyFetcher(doc));
        let did = Did::parse("did:web:example.com").unwrap();
        let resolved = resolver.resolve(&did).unwrap();
        assert_eq!(resolved.as_bytes(), vk.as_bytes());
    }

    #[cfg(feature = "did-web")]
    #[test]
    fn did_web_no_key_fails_closed() {
        let doc = r#"{"id":"did:web:example.com","verificationMethod":[]}"#.to_string();
        let resolver = DidWebResolver::new(DummyFetcher(doc));
        let did = Did::parse("did:web:example.com").unwrap();
        assert!(matches!(
            resolver.resolve(&did),
            Err(DidError::NoIssuerKey(_))
        ));
    }
}
