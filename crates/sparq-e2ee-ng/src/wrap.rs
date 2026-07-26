//! **Recipient-wrapped secrets** (§4.2, §8.2). Capability secret fields (the
//! read secret `K_read`, publisher/admin private keys) and per-object DEKs are
//! bearer secrets: they MUST NOT travel in RDF, broker requests, logs, URLs, or
//! topic messages, and when they *are* transferred they go inside a recipient
//! wrapping envelope or a separately protected channel.
//!
//! v0 wrapping (WRAP-256) is a sealed-box shape: an ephemeral X25519 key does
//! ECDH to the recipient's public key, the shared secret is run through the
//! domain-separated [`crate::keyschedule::wrap_key`] (binding both public keys),
//! and the payload is AEAD-sealed under that key. Only the recipient's private
//! key recovers it. Each wrap draws a fresh ephemeral sender key and nonce, so
//! wraps are independent — but this is ephemeral-*static* ECDH and does **not**
//! provide forward secrecy: a recorded [`WrappedSecret`] carries `ephemeral_pub`,
//! so if the recipient's long-term private key is later compromised an attacker
//! can recompute `X25519(recipient_sk, ephemeral_pub)`, re-derive the wrap key,
//! and decrypt the historical ciphertext.
//! Both seal and open reject a non-contributory (low-order-point) exchange
//! before key derivation, so the wrap key always depends on both keys.

use crate::cbor::{enc_bytes, enc_map, enc_uint, read_struct_map, Limits, Reader};
use crate::error::{Error, Result};
use crate::keyschedule::wrap_key;
use crate::suite::{aead_open, aead_seal, AEAD_NONCE_LEN};
use rand_core::{OsRng, RngCore};
use x25519_dalek::{PublicKey, StaticSecret};

/// Length of an X25519 public key.
pub const WRAP_PUBLIC_KEY_LEN: usize = 32;

/// A recipient's X25519 private key. Zeroed on drop by `StaticSecret`.
pub struct RecipientSecretKey {
    inner: StaticSecret,
}

impl RecipientSecretKey {
    /// Generate a fresh recipient key from the OS CSPRNG.
    pub fn generate() -> Self {
        RecipientSecretKey {
            inner: StaticSecret::random_from_rng(OsRng),
        }
    }

    /// Construct from 32 raw bytes (clamped; deterministic — used by test vectors).
    pub fn from_bytes(b: [u8; 32]) -> Self {
        RecipientSecretKey {
            inner: StaticSecret::from(b),
        }
    }

    /// The recipient's public key (safe to share with senders).
    pub fn public(&self) -> RecipientPublicKey {
        RecipientPublicKey(PublicKey::from(&self.inner).to_bytes())
    }
}

impl core::fmt::Debug for RecipientSecretKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("RecipientSecretKey(REDACTED)")
    }
}

/// A recipient's X25519 public key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecipientPublicKey(pub [u8; WRAP_PUBLIC_KEY_LEN]);

/// A wrapped secret: an ephemeral public key plus an AEAD ciphertext under the
/// derived wrap key. Deterministic CBOR so it can be embedded and hashed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrappedSecret {
    /// The sender's ephemeral X25519 public key.
    pub ephemeral_pub: [u8; WRAP_PUBLIC_KEY_LEN],
    /// AEAD nonce.
    pub nonce: [u8; AEAD_NONCE_LEN],
    /// AEAD ciphertext-and-tag over the wrapped secret.
    pub ciphertext: Vec<u8>,
}

impl WrappedSecret {
    // wire keys
    const K_VERSION: u64 = 1;
    const K_EPH: u64 = 2;
    const K_NONCE: u64 = 3;
    const K_CT: u64 = 4;
    const VERSION: u64 = 0;

    /// Canonical CBOR encoding.
    pub fn encode(&self) -> Vec<u8> {
        enc_map(vec![
            (Self::K_VERSION, enc_uint(Self::VERSION)),
            (Self::K_EPH, enc_bytes(&self.ephemeral_pub)),
            (Self::K_NONCE, enc_bytes(&self.nonce)),
            (Self::K_CT, enc_bytes(&self.ciphertext)),
        ])
    }

    /// Decode + validate a wrapped secret (fail-closed).
    pub fn decode(bytes: &[u8], limits: Limits) -> Result<Self> {
        let mut r = Reader::new(bytes, limits);
        let mut version: Option<u64> = None;
        let mut eph: Option<[u8; 32]> = None;
        let mut nonce: Option<[u8; AEAD_NONCE_LEN]> = None;
        let mut ct: Option<Vec<u8>> = None;
        read_struct_map(&mut r, |r, key| match key {
            Self::K_VERSION => {
                version = Some(r.uint()?);
                Ok(true)
            }
            Self::K_EPH => {
                eph = Some(r.bytes_fixed::<32>()?);
                Ok(true)
            }
            Self::K_NONCE => {
                nonce = Some(r.bytes_fixed::<AEAD_NONCE_LEN>()?);
                Ok(true)
            }
            Self::K_CT => {
                ct = Some(r.bytes()?.to_vec());
                Ok(true)
            }
            _ => Ok(false),
        })?;
        r.finish()?;
        if version != Some(Self::VERSION) {
            return Err(Error::Schema("wrapped-secret version"));
        }
        Ok(WrappedSecret {
            ephemeral_pub: eph.ok_or(Error::Schema("missing ephemeral_pub"))?,
            nonce: nonce.ok_or(Error::Schema("missing nonce"))?,
            ciphertext: ct.ok_or(Error::Schema("missing ciphertext"))?,
        })
    }
}

/// Wrap `plaintext` to `recipient`, binding `aad` (e.g. a purpose label). Draws
/// a fresh ephemeral key and nonce from the OS CSPRNG. Fails closed with
/// [`Error::BadKey`] if `recipient` is a low-order point (the exchange would be
/// non-contributory: the "shared" secret would not depend on our ephemeral key).
pub fn wrap(
    recipient: &RecipientPublicKey,
    plaintext: &[u8],
    aad: &[u8],
) -> Result<WrappedSecret> {
    let mut eph_seed = [0u8; 32];
    OsRng.fill_bytes(&mut eph_seed);
    let mut nonce = [0u8; AEAD_NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);
    wrap_with(eph_seed, nonce, recipient, plaintext, aad)
}

/// Wrap with caller-supplied ephemeral seed + nonce. Deterministic — the path
/// test vectors exercise. Production code uses [`wrap`]. Fails closed on a
/// low-order (non-contributory) recipient key.
pub fn wrap_with(
    eph_seed: [u8; 32],
    nonce: [u8; AEAD_NONCE_LEN],
    recipient: &RecipientPublicKey,
    plaintext: &[u8],
    aad: &[u8],
) -> Result<WrappedSecret> {
    let eph_sk = StaticSecret::from(eph_seed);
    let eph_pub = PublicKey::from(&eph_sk);
    let recipient_pk = PublicKey::from(recipient.0);
    let shared = eph_sk.diffie_hellman(&recipient_pk);
    if !shared.was_contributory() {
        return Err(Error::BadKey("non-contributory X25519 recipient key"));
    }
    let wk = wrap_key(shared.as_bytes(), &recipient.0, eph_pub.as_bytes());
    let ct = aead_seal(&wk, &nonce, aad, plaintext);
    Ok(WrappedSecret {
        ephemeral_pub: eph_pub.to_bytes(),
        nonce,
        ciphertext: ct,
    })
}

/// Unwrap a [`WrappedSecret`] with the recipient's private key. Fails closed on
/// a wrong key, tampered ciphertext, wrong `aad`, or a low-order ephemeral key
/// (a non-contributory exchange would derive the wrap key from public context
/// alone, so it is rejected before any key derivation).
pub fn unwrap(sk: &RecipientSecretKey, wrapped: &WrappedSecret, aad: &[u8]) -> Result<Vec<u8>> {
    let eph_pub = PublicKey::from(wrapped.ephemeral_pub);
    let recipient_pub = sk.public();
    let shared = sk.inner.diffie_hellman(&eph_pub);
    if !shared.was_contributory() {
        return Err(Error::Decrypt);
    }
    let wk = wrap_key(shared.as_bytes(), &recipient_pub.0, &wrapped.ephemeral_pub);
    aead_open(&wk, &wrapped.nonce, aad, &wrapped.ciphertext)
}
