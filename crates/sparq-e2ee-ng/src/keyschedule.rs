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

/// Recipient-wrap key `= HKDF(ecdh_shared; recipient_pub || ephemeral_pub)`
/// (used by [`crate::wrap`]). Binding both public keys thwarts key-reuse /
/// unknown-key-share confusion.
pub fn wrap_key(shared: &[u8; 32], recipient_pub: &[u8; 32], ephemeral_pub: &[u8; 32]) -> [u8; 32] {
    derive(LABEL_WRAP_KEY, shared, &[recipient_pub, ephemeral_pub])
}
