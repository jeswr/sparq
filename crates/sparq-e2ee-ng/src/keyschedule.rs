//! The **domain-separated key schedule** (§8.3). Every derived key is bound to a
//! distinct domain-separation label plus the exact context that scopes it, so a
//! key derived for one purpose can never be confused with, or substituted for, a
//! key for another. The KDF is HKDF-SHA-256 (RFC 5869): a fixed `extract` over
//! the label as salt, then an `expand` over the length-delimited context.
//!
//! The context fields are concatenated with an unambiguous 2-byte big-endian
//! length prefix per field, so `("ab","c")` and `("a","bc")` never collide.

use crate::ids::{BlockId, BranchId, Epoch, ObjectId, RepoId, Secret32};
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

// --- domain-separation labels (used as the HKDF salt) -----------------------
const LABEL_OBJECT_KEY: &[u8] = b"urn:jeswr:w3id:e2ee-ng:draft:2026-07 object-key v0";
const LABEL_BLOCK_KEY: &[u8] = b"urn:jeswr:w3id:e2ee-ng:draft:2026-07 block-key v0";
const LABEL_WRAP_KEY: &[u8] = b"urn:jeswr:w3id:e2ee-ng:draft:2026-07 recipient-wrap v0";
// Profile SE (`se` feature) value-key label. Note the *different* URN stem: an SE
// literal key belongs to the `e2ee-sparql` draft, not to the `e2ee-ng` block
// profile, so an SE value key and an E2EE-NG object/block key can never coincide
// even if a deployment feeds the same input keying material to both.
#[cfg(feature = "se")]
const LABEL_VALUE_KEY: &[u8] = b"urn:jeswr:w3id:e2ee-sparql:draft:2026-07 value-key v0";

/// HKDF-Extract (RFC 5869 §2.2): `PRK = HMAC(salt, ikm)`.
fn extract(salt: &[u8], ikm: &[u8]) -> [u8; 32] {
    let mut mac = <HmacSha256 as Mac>::new_from_slice(salt).expect("hmac any key len");
    mac.update(ikm);
    mac.finalize().into_bytes().into()
}

/// HKDF-Expand (RFC 5869 §2.3), specialized to a single 32-byte output block
/// (`L = 32 <= 255*HashLen`), which is all the profile derives.
fn expand32(prk: &[u8; 32], info: &[u8]) -> [u8; 32] {
    let mut mac = <HmacSha256 as Mac>::new_from_slice(prk).expect("hmac 32-byte key");
    mac.update(info);
    mac.update(&[0x01]); // T(1) counter
    mac.finalize().into_bytes().into()
}

/// Encode `parts` as a length-delimited, unambiguous byte string used as the
/// HKDF `info`. Each part is `len(u16-be) || part`.
fn info(parts: &[&[u8]]) -> Vec<u8> {
    let mut out = Vec::new();
    for p in parts {
        let len = u16::try_from(p.len()).expect("context field < 64 KiB");
        out.extend_from_slice(&len.to_be_bytes());
        out.extend_from_slice(p);
    }
    out
}

/// Derive a 32-byte key from an input keying material and a labelled context.
fn derive(label: &[u8], ikm: &[u8], parts: &[&[u8]]) -> [u8; 32] {
    let prk = extract(label, ikm);
    expand32(&prk, &info(parts))
}

/// Per-object key `= HKDF(K_read; repo || branch || epoch || object_id)` (§8.3).
/// Binds the read secret to the exact (repo, branch, epoch, object) scope.
pub fn object_key(
    k_read: &Secret32,
    repo: &RepoId,
    branch: &BranchId,
    epoch: Epoch,
    object: &ObjectId,
) -> [u8; 32] {
    derive(
        LABEL_OBJECT_KEY,
        k_read.expose(),
        &[
            repo.as_bytes(),
            branch.as_bytes(),
            &epoch.0.to_be_bytes(),
            object.as_bytes(),
        ],
    )
}

/// Domain-separated per-block key `= HKDF(object_key; block_id || chunk_index)`
/// (§8.3). Each block within an object gets its own AEAD key.
pub fn block_key(object_key: &[u8; 32], block: &BlockId, chunk_index: u64) -> [u8; 32] {
    derive(
        LABEL_BLOCK_KEY,
        object_key,
        &[block.as_bytes(), &chunk_index.to_be_bytes()],
    )
}

/// Profile SE per-position value key
/// `= HKDF(dek; predicate || graph_present || graph)` — the AEAD key for one
/// encrypted **literal value** (see [`crate::literal`]).
///
/// Binding the per-predicate DEK to the predicate (and, when the value sits in a
/// named graph, to that graph IRI) means a DEK leaked for one predicate cannot
/// open another predicate's values, and a value cannot be replayed from one
/// named graph into another. `graph = None` is the default graph and is encoded
/// distinguishably from `Some("")` via an explicit presence byte, so the two can
/// never derive the same key.
///
/// The *subject* is deliberately NOT an input here — it is bound in the AEAD
/// associated data instead (optionally, per
/// [`ValueContext::subject`](crate::literal::ValueContext::subject)), because
/// [`crate::literal::equality_tag`] must be comparable across subjects.
#[cfg(feature = "se")]
#[cfg_attr(docsrs, doc(cfg(feature = "se")))]
pub fn value_key(dek: &Secret32, predicate: &str, graph: Option<&str>) -> [u8; 32] {
    let present = [u8::from(graph.is_some())];
    derive(
        LABEL_VALUE_KEY,
        dek.expose(),
        &[
            predicate.as_bytes(),
            &present,
            graph.unwrap_or("").as_bytes(),
        ],
    )
}

/// Recipient-wrap key `= HKDF(ecdh_shared; recipient_pub || ephemeral_pub)`
/// (used by [`crate::wrap`]). Binding both public keys thwarts key-reuse /
/// unknown-key-share confusion.
pub fn wrap_key(shared: &[u8; 32], recipient_pub: &[u8; 32], ephemeral_pub: &[u8; 32]) -> [u8; 32] {
    derive(LABEL_WRAP_KEY, shared, &[recipient_pub, ephemeral_pub])
}
