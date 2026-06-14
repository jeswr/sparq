// [OPUS-4.8] Shared value types crossing the MPC module boundaries (M0).
//! Shared, crypto-free value types used across the MPC module boundaries.
//!
//! Architecture refs: §4.1 (parties — holders are identified), §4.3 step 3
//! (disclosure minimisation — a partial carries only disclosed bindings), and
//! the empirical-honesty discipline (every fork-dependent path surfaces a
//! single, named [`MpcError::NotYetImplemented`] rather than fake crypto).

use oxrdf::{Term, Variable};

/// Identifies one holder (data source / MPC compute party) within a federation.
///
/// Architecture §4.1: holders are the four Pods — mutually distrusting,
/// possibly malicious, acting as MPC compute parties and collaborative provers.
/// At M0 this is just a stable label; the cryptographic identity (the holder's
/// signing key / proof-of-possession) is gated on the ZK foundation and lives
/// behind [`crate::proof`], not here.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct HolderId(pub String);

impl HolderId {
    pub fn new(id: impl Into<String>) -> Self {
        HolderId(id.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for HolderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// One holder's *local partial result*: the disclosed bindings that holder
/// computed over ITS OWN graphs, ready to be joined on global IRIs.
///
/// Architecture §4.2 / §4.3 step 1: this is the unit of inter-node sharing, and
/// the whole point of the design is to MINIMISE it — a holder ships these
/// partials, **never** its raw named graphs. The row shape mirrors
/// `sparq_engine::QueryResult` (`vars` + nullable `rows`) so a holder never has
/// to re-encode its own engine output to produce a partial.
///
/// What is and is NOT disclosed in a partial is the RQ2a question (§4.3 step 3,
/// §5.2 Q3). At M0 the local evaluation discloses its full local SELECT result
/// (the "disclosed-property" regime of convention #4); the *hidden*-intermediate
/// regime (values that must never leave a source, fed into the MPC core) is
/// future work behind [`crate::backend`] and is NOT represented here yet.
#[derive(Debug, Clone, PartialEq)]
pub struct PartialResult {
    /// The holder that produced this partial.
    pub holder: HolderId,
    /// The projected variables, in column order (matches `rows`).
    pub vars: Vec<Variable>,
    /// Disclosed solution rows; one entry per `vars` column, `None` = unbound.
    pub rows: Vec<Vec<Option<Term>>>,
}

impl PartialResult {
    /// Number of disclosed rows in this partial.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// `true` if this holder contributed no rows for the fragment.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// The crate's error type. Crucially, [`MpcError::NotYetImplemented`] is the
/// SINGLE honest channel for everything that is fork-dependent or gated on the
/// ZK foundation: no method in this crate fakes crypto — it either does real
/// work (local eval) or returns this with the gating milestone/issue named.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MpcError {
    /// A holder's local SPARQL sub-evaluation failed (parse / eval error from
    /// `sparq-engine`, surfaced verbatim). This is a REAL error path — local
    /// evaluation is the one piece implemented at M0.
    LocalEval { holder: HolderId, message: String },
    /// A federation/protocol precondition was violated before any crypto would
    /// run (e.g. holders disagree on the projected variable list of a fragment,
    /// or an empty federation). Real, M0-checkable.
    Protocol(String),
    /// The single honest stub channel: this path is gated on a milestone and/or
    /// an open design fork. `what` names the capability; `gated_on` names the
    /// milestone(s), ZK-remediation issue(s), and/or open question(s) that must
    /// land first. Carrying the gate in the value (not just a doc-comment) keeps
    /// the deferral auditable at the call site.
    NotYetImplemented { what: String, gated_on: String },
    /// **Active tampering detected** during a consistency-checked / robust
    /// reconstruction (guarantee (D), malicious-honest-majority abort — sq-m34i).
    /// Returned when redundant Shamir shares (`n > t+1`) do NOT all lie on one
    /// degree-`t` polynomial and the inconsistency exceeds the correctable error
    /// budget `e = ⌊(n−t−1)/2⌋`. This is **not** a precondition bug
    /// ([`MpcError::Protocol`]) nor a deferred stub ([`MpcError::NotYetImplemented`]):
    /// it means a party fed an inconsistent share value, so the result is
    /// REFUSED rather than silently corrupted. `cheaters` lists the evaluation
    /// points (`Share::x`) identified as off-curve where attribution is possible
    /// (best-effort when correction itself is impossible — detection is sound,
    /// attribution is heuristic). NOTE: at exactly `t+1` shares (no redundancy)
    /// tampering is information-theoretically undetectable and this is NEVER
    /// returned there — see [`crate::robust::reconstruct_robust`].
    Tampered { detail: String, cheaters: Vec<u64> },
}

impl MpcError {
    /// Construct a [`MpcError::NotYetImplemented`] from static descriptions.
    /// Used by every crypto-deferred trait method so the gate is uniform and
    /// greppable.
    pub fn not_yet(what: &str, gated_on: &str) -> Self {
        MpcError::NotYetImplemented { what: what.to_string(), gated_on: gated_on.to_string() }
    }
}

impl std::fmt::Display for MpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MpcError::LocalEval { holder, message } => {
                write!(f, "holder {holder} local sub-evaluation failed: {message}")
            }
            MpcError::Protocol(m) => write!(f, "MPC protocol precondition violated: {m}"),
            MpcError::NotYetImplemented { what, gated_on } => {
                write!(f, "not yet implemented: {what} (gated on {gated_on})")
            }
            MpcError::Tampered { detail, cheaters } => {
                write!(
                    f,
                    "tampered share(s) detected: {detail} (suspect evaluation points: {cheaters:?})"
                )
            }
        }
    }
}

impl std::error::Error for MpcError {}
