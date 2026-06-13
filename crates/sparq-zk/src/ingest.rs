//! Trusted-ingest path: globally-unique per-graph salts (sq-610) and (Stage 2)
//! per-named-graph commitment (sq-cn8). See the type-level docs.
//!
//! This commit (sq-610) lands the **salt mint**: every named graph gets a salt
//! drawn from [`SaltMint`], which fills 32 bytes from the OS CSPRNG
//! ([`getrandom`]) and enforces **global uniqueness** across the whole ingest —
//! every issued salt is tracked, a CSPRNG collision is redrawn, and any
//! externally-supplied salt that collides with one already issued is *rejected*
//! ([`SaltMint::register`]). The salt is the per-graph RDFC10 bnode salt
//! (`zk:rdfc10Salt`): the Q6 "bnodes from different graphs are distinct by
//! construction" guarantee rests on it being globally unique, so this is the
//! trusted-ingest enforcement the soundness audit (#9) requires instead of an
//! unenforced convention.
//!
//! # Soundness note (honest scope)
//! Salt uniqueness is enforced *at ingest*; the verifier-side / in-circuit
//! binding of the salt (so a salt-reusing prover cannot present a graph the
//! trusted ingest never minted) lives in the issuer signature
//! ([`crate::sig::commitment_message_with_salt`], audit #9 fix (a)) and is NOT
//! re-implemented here — this module is the *mint* side. A salt minted here is
//! globally unique within one [`SaltMint`] session; uniqueness across separate
//! processes/sessions rests on 248-bit CSPRNG entropy (collision probability
//! negligible), and a persistent ingester that must guarantee uniqueness across
//! restarts should seed the mint's seen-set from the existing registry (see
//! [`SaltMint::from_registry`]). Deliberately deferred: in-circuit salt binding
//! (audit #9 fix (b)) and cross-process durable uniqueness state.

use crate::encode::salt_from_bytes;
use crate::field::Fr;
use crate::registry::RegistryEntry;
use std::collections::HashSet;

/// How many CSPRNG redraws before a mint gives up on finding a fresh salt.
/// A collision needs two of the same 248-bit draw, so a single redraw already
/// makes failure astronomically unlikely; the bound only exists so a broken
/// entropy source fails loud instead of looping forever.
const MAX_MINT_REDRAWS: usize = 64;

/// Ingest-path failures.
#[derive(Debug)]
pub enum IngestError {
    /// The salt mint could not produce a fresh salt (broken entropy source, or
    /// a saturated seen-set — both unreachable in practice).
    SaltExhausted,
    /// An externally-supplied salt collides with one already issued by the same
    /// mint — the uniqueness invariant the trusted ingest enforces (sq-610).
    SaltCollision,
}

impl std::fmt::Display for IngestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IngestError::SaltExhausted => write!(
                f,
                "salt mint exhausted: could not draw a globally-unique salt (check the OS CSPRNG)"
            ),
            IngestError::SaltCollision => write!(
                f,
                "salt collision rejected: the supplied salt was already issued in this ingest \
                 (globally-unique per-graph salts are required — Q6 / audit #9)"
            ),
        }
    }
}

impl std::error::Error for IngestError {}

/// A trusted-ingest salt mint: draws globally-unique per-graph RDFC10 bnode
/// salts and enforces that no two graphs in the same ingest share a salt
/// (sq-610 / audit #9).
///
/// The salt is a 248-bit BN254 field element ([`salt_from_bytes`] over 32 OS
/// CSPRNG bytes). Uniqueness is structural: every issued salt is recorded, a
/// (vanishingly improbable) CSPRNG collision triggers a redraw, and a caller
/// who supplies its own salt has it checked against the seen-set.
#[derive(Debug, Default)]
pub struct SaltMint {
    /// Every salt this mint has issued / accepted — the uniqueness oracle.
    issued: HashSet<Fr>,
}

impl SaltMint {
    /// A fresh mint with an empty seen-set.
    pub fn new() -> Self {
        SaltMint { issued: HashSet::new() }
    }

    /// Seeds the seen-set from an existing registry's salts so a persistent
    /// ingester guarantees uniqueness against previously-minted graphs (cross
    /// the durability gap noted in the module docs). Salts already in the
    /// registry are treated as issued.
    pub fn from_registry(entries: &[RegistryEntry]) -> Self {
        SaltMint { issued: entries.iter().map(|e| e.salt).collect() }
    }

    /// The number of salts issued so far (for tests / introspection).
    pub fn issued_count(&self) -> usize {
        self.issued.len()
    }

    /// `true` if `salt` has already been issued by this mint.
    pub fn contains(&self, salt: &Fr) -> bool {
        self.issued.contains(salt)
    }

    /// Draws a globally-unique salt from the OS CSPRNG. Fills 32 random bytes,
    /// folds them into a field element, and redraws on the (negligible) chance
    /// the same salt was already issued. Fails closed ([`IngestError::SaltExhausted`])
    /// only if the entropy source is broken (no fresh value in
    /// [`MAX_MINT_REDRAWS`] tries).
    pub fn mint(&mut self) -> Result<Fr, IngestError> {
        for _ in 0..MAX_MINT_REDRAWS {
            let mut bytes = [0u8; 32];
            // getrandom is the OS CSPRNG; an error here means no entropy source.
            if getrandom::getrandom(&mut bytes).is_err() {
                return Err(IngestError::SaltExhausted);
            }
            let salt = salt_from_bytes(&bytes);
            if self.issued.insert(salt) {
                return Ok(salt);
            }
        }
        Err(IngestError::SaltExhausted)
    }

    /// Registers an externally-supplied salt (e.g. one drawn by an issuer tool
    /// out of band), enforcing global uniqueness. Returns
    /// [`IngestError::SaltCollision`] if the salt was already issued — this is
    /// the *detection/prevention* of a forced collision (sq-610): a duplicate
    /// salt can never enter the ingest.
    pub fn register(&mut self, salt: Fr) -> Result<Fr, IngestError> {
        if self.issued.insert(salt) {
            Ok(salt)
        } else {
            Err(IngestError::SaltCollision)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    // ---- sq-610: globally-unique salts ----------------------------------------

    /// A mint issues all-distinct salts over many draws.
    #[test]
    fn mint_draws_are_all_distinct() {
        let mut mint = SaltMint::new();
        let mut seen = HashSet::new();
        for _ in 0..1000 {
            let s = mint.mint().unwrap();
            assert!(seen.insert(s), "minted salt repeated: {s:?}");
        }
        assert_eq!(mint.issued_count(), 1000);
    }

    /// A forced collision (re-registering an already-issued salt) is detected
    /// and prevented — the duplicate cannot enter the ingest.
    #[test]
    fn forced_salt_collision_is_rejected() {
        let mut mint = SaltMint::new();
        let s = mint.mint().unwrap();
        // Re-registering the same salt collides and is refused.
        assert!(matches!(mint.register(s), Err(IngestError::SaltCollision)));
        // A genuinely fresh external salt is accepted.
        let fresh = salt_from_bytes(&[0xABu8; 32]);
        assert!(mint.register(fresh).is_ok());
        // ...and re-registering THAT one now also collides.
        assert!(matches!(mint.register(fresh), Err(IngestError::SaltCollision)));
    }
}
