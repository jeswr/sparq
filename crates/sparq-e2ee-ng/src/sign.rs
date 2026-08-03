//! Signatures (§8.2, §8.3) — v0 = Ed25519 (RFC 8032). The profile uses three
//! *separate* signing authorities, and this module gives them one small typed
//! surface so they cannot be crossed accidentally:
//!
//! * an **author** signs a commit's plaintext fields (§8.3 field 11);
//! * a **publisher** is admitted by the broker via its registered public key
//!   (§4.2, §8.4) — a publisher is a distinct authority from a reader;
//! * an **admin** signs capability public grants and epoch-transition commits
//!   (§4.2, §8.3) — a distinct authority from both.
//!
//! Key *possession* is what grants power; a signature authenticates a public
//! grant/commit but is never itself a decryption key (§4.2).

use crate::error::{Error, Result};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use zeroize::Zeroize;

/// Length of an Ed25519 public key.
pub const PUBLIC_KEY_LEN: usize = 32;
/// Length of an Ed25519 signature.
pub const SIGNATURE_LEN: usize = 64;

/// A private Ed25519 signing authority (author / publisher / admin). The seed is
/// zeroed on drop. Which role a given key plays is enforced by *where it is
/// stored in a capability*, not by the key type itself (all three are Ed25519).
pub struct SecretSigningKey {
    inner: SigningKey,
}

impl SecretSigningKey {
    /// Generate a fresh key from the OS CSPRNG.
    pub fn generate() -> Self {
        SecretSigningKey {
            inner: SigningKey::generate(&mut rand_core::OsRng),
        }
    }

    /// Construct from a 32-byte seed (deterministic; used by test vectors).
    pub fn from_seed(seed: [u8; 32]) -> Self {
        let mut seed = seed;
        let inner = SigningKey::from_bytes(&seed);
        seed.zeroize();
        SecretSigningKey { inner }
    }

    /// The corresponding public verification key.
    pub fn public(&self) -> PublicVerifyingKey {
        PublicVerifyingKey {
            inner: self.inner.verifying_key(),
        }
    }

    /// Sign `msg`, returning the 64-byte detached signature.
    pub fn sign(&self, msg: &[u8]) -> [u8; SIGNATURE_LEN] {
        self.inner.sign(msg).to_bytes()
    }

    /// The raw 32-byte seed (handle with care; used to serialize a secret-bearing
    /// capability for out-of-band / recipient-wrapped transfer only).
    pub fn to_seed(&self) -> [u8; 32] {
        self.inner.to_bytes()
    }
}

impl core::fmt::Debug for SecretSigningKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("SecretSigningKey(REDACTED)")
    }
}

/// A public Ed25519 verification key. Safe to publish (it is registered with the
/// broker for publisher admission, and embedded in capability public grants).
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PublicVerifyingKey {
    inner: VerifyingKey,
}

impl PublicVerifyingKey {
    /// The raw 32-byte public key.
    pub fn to_bytes(&self) -> [u8; PUBLIC_KEY_LEN] {
        self.inner.to_bytes()
    }

    /// Parse a 32-byte public key, rejecting non-canonical / small-order points.
    pub fn from_bytes(b: &[u8; PUBLIC_KEY_LEN]) -> Result<Self> {
        VerifyingKey::from_bytes(b)
            .map(|inner| PublicVerifyingKey { inner })
            .map_err(|_| Error::BadKey("invalid ed25519 public key"))
    }

    /// Verify a detached signature over `msg`. Uses `verify_strict` to reject the
    /// small-order / malleable edge cases.
    pub fn verify(&self, msg: &[u8], sig: &[u8; SIGNATURE_LEN]) -> Result<()> {
        let signature = Signature::from_bytes(sig);
        self.inner
            .verify_strict(msg, &signature)
            .map_err(|_| Error::BadSignature)
    }
}

impl core::fmt::Debug for PublicVerifyingKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let b = self.to_bytes();
        write!(
            f,
            "PublicVerifyingKey({:02x}{:02x}{:02x}{:02x}…)",
            b[0], b[1], b[2], b[3]
        )
    }
}
