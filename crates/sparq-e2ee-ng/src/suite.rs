//! **Algorithm agility** (§8.1). Every capability, commit, and session binds a
//! single **suite identifier**, and an implementation MUST NOT silently
//! substitute algorithms. v0 ships exactly one concrete suite and rejects any
//! other id.
//!
//! > **Honesty / audit boundary.** The concrete v0 primitives below are
//! > well-known vetted building blocks (XChaCha20-Poly1305, HKDF-SHA-256,
//! > Ed25519, X25519), but their *composition into this profile* has **not**
//! > received external cryptographic review. The v0 suite name is a placeholder
//! > pending that review; production suite selection is gated by **sq-qhy4**
//! > (research doc §8.1, §9). Nothing here is claimed sound or private.

use crate::error::{Error, Result};
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    XChaCha20Poly1305, XNonce,
};

/// The v0 suite identifier. A deployment MUST bind exactly this string (until a
/// reviewed suite replaces it) into every capability, commit, and session; a
/// mismatch is [`Error::UnknownSuite`]. The name encodes the concrete choices so
/// two peers cannot disagree about them silently.
pub const SUITE_V0: &str =
    "urn:jeswr:w3id:e2ee-ng:draft:2026-07#suite-v0-x25519-xchacha20poly1305-hkdf-sha256-ed25519";

/// AEAD nonce length for the v0 suite (XChaCha20-Poly1305: 192-bit / 24-byte).
/// A 24-byte nonce is large enough to be drawn at random per block/wrap without
/// a stateful counter, which is what the randomized-envelope design needs.
pub const AEAD_NONCE_LEN: usize = 24;

/// AEAD symmetric key length (256-bit).
pub const AEAD_KEY_LEN: usize = 32;

/// Reject any suite id other than the bound v0 suite. Call this on every decode
/// of a suite-carrying object so an unknown/substituted suite fails closed.
pub fn check_suite(id: &str) -> Result<()> {
    if id == SUITE_V0 {
        Ok(())
    } else {
        Err(Error::UnknownSuite)
    }
}

/// AEAD-seal (v0 = XChaCha20-Poly1305). `aad` is authenticated but not
/// encrypted; it is how the envelope binds version/suite/repo/branch/epoch/
/// object/block/chunk (§8.3). Returns ciphertext-and-tag.
pub fn aead_seal(
    key: &[u8; AEAD_KEY_LEN],
    nonce: &[u8; AEAD_NONCE_LEN],
    aad: &[u8],
    plaintext: &[u8],
) -> Vec<u8> {
    let cipher = XChaCha20Poly1305::new(key.into());
    cipher
        .encrypt(XNonce::from_slice(nonce), Payload { msg: plaintext, aad })
        // XChaCha20-Poly1305 encryption is infallible for in-bounds inputs.
        .expect("aead seal")
}

/// AEAD-open (v0 = XChaCha20-Poly1305). Fails closed on a wrong key, tampered
/// ciphertext, or wrong associated data.
pub fn aead_open(
    key: &[u8; AEAD_KEY_LEN],
    nonce: &[u8; AEAD_NONCE_LEN],
    aad: &[u8],
    ciphertext: &[u8],
) -> Result<Vec<u8>> {
    let cipher = XChaCha20Poly1305::new(key.into());
    cipher
        .decrypt(XNonce::from_slice(nonce), Payload { msg: ciphertext, aad })
        .map_err(|_| Error::Decrypt)
}
