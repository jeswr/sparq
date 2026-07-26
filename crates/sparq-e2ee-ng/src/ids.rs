//! Fixed-width public identifiers (§4.1). These are opaque 32-byte values. In
//! v0 every id that the profile calls "random" MUST be drawn from a CSPRNG and
//! is **not** derived from plaintext or ciphertext (this rules out the
//! confirmation / equality / graph-shape leakage that a convergent or
//! hash-derived id would introduce — see research doc §2 and §8.3).
//!
//! These identifiers are **not** RDF IRIs unless a local mapping chooses to make
//! them so (§4.1). They carry no secret material and are safe to expose to a
//! broker where the profile says so (§5).

use rand_core::{OsRng, RngCore};
use zeroize::Zeroize;

/// Declare a `#[repr(transparent)]` 32-byte public identifier newtype.
macro_rules! id32 {
    ($(#[$m:meta])* $name:ident) => {
        $(#[$m])*
        #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(pub [u8; 32]);

        impl $name {
            /// Draw a fresh random identifier from the OS CSPRNG.
            pub fn random() -> Self {
                let mut b = [0u8; 32];
                OsRng.fill_bytes(&mut b);
                $name(b)
            }
            /// The raw 32 bytes.
            pub fn as_bytes(&self) -> &[u8; 32] {
                &self.0
            }
            /// Construct from raw bytes.
            pub fn from_bytes(b: [u8; 32]) -> Self {
                $name(b)
            }
        }

        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                // Short hex prefix; ids are public but full-width dumps are noisy.
                write!(f, "{}({:02x}{:02x}{:02x}{:02x}…)", stringify!($name),
                       self.0[0], self.0[1], self.0[2], self.0[3])
            }
        }
    };
}

id32!(
    /// Random 256-bit public identifier for one repository (§4.1).
    RepoId
);
id32!(
    /// Random 256-bit stable identifier for one branch (§4.1).
    BranchId
);
id32!(
    /// Epoch-specific random public routing identifier for one branch (§4.1).
    TopicId
);
id32!(
    /// Random 256-bit identifier of an encrypted block; authenticated inside its
    /// encrypted parent, and — critically — **not** derived from plaintext or
    /// ciphertext in v0 (§4.1, §8.3).
    BlockId
);
id32!(
    /// Random 256-bit identifier for a chunked object (§8.3).
    ObjectId
);
id32!(
    /// `CommitId = SHA-256(canonical encrypted root envelope)` (§8.3).
    CommitId
);
id32!(
    /// `cap_id = SHA-256(canonical public grant)` — a local lookup handle that
    /// is never itself a secret (§4.2).
    CapId
);
id32!(
    /// An author's public-key identifier (§8.3 field 6). In v0 this is the raw
    /// Ed25519 public key bytes.
    AuthorKeyId
);

/// Monotonic membership/key-rotation counter (§4.1). Wrapped in a newtype so an
/// epoch is never confused with a logical clock or any other `u64`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Epoch(pub u64);

impl Epoch {
    /// The first epoch.
    pub const ZERO: Epoch = Epoch(0);
    /// The next epoch (saturating; the profile never wraps an epoch).
    pub fn next(self) -> Epoch {
        Epoch(self.0.saturating_add(1))
    }
}

/// Zero a 32-byte secret buffer wrapper on drop. Used for read secrets and raw
/// private-key bytes so they do not linger in freed memory.
#[derive(Clone, Zeroize)]
#[zeroize(drop)]
pub struct Secret32(pub [u8; 32]);

impl Secret32 {
    /// Draw a fresh 32-byte secret from the OS CSPRNG.
    pub fn random() -> Self {
        let mut b = [0u8; 32];
        OsRng.fill_bytes(&mut b);
        Secret32(b)
    }
    /// Borrow the raw bytes (handle with care).
    pub fn expose(&self) -> &[u8; 32] {
        &self.0
    }
}

impl core::fmt::Debug for Secret32 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // NEVER print secret bytes.
        f.write_str("Secret32(REDACTED)")
    }
}
