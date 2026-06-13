// [OPUS-4.8] GlobalJoin trait — join holders' partials on GLOBAL IRIs.
//! The cross-holder join protocol (interface only; impl deferred to M2).
//!
//! Architecture refs: §2 convention #6 (GLOBAL IRIs as cross-credential join
//! keys — the distinguishing feature vs all prior graph-MPC), §4.3 step 4 (the
//! join model: key-on-key → (circuit-)PSI; non-key → oblivious join with
//! bounded intermediate size), §3.1 (PSI does NOT compose for free into a
//! multi-pattern BGP), and OPEN QUESTION **§5.2 Q3** (BGP-join obliviousness
//! cost).
//!
//! ## Why this is its own boundary
//!
//! GOOSE/SMPG/PPMQ all join on *node-local* (Cypher) identifiers, which
//! disqualifies them for federation: a node id is meaningful only inside one
//! database. The contribution here is joining on **global IRIs** that mean the
//! same thing across independent holders (architecture §2 convention #6). That
//! makes the join key a public, dereferenceable identifier — which interacts
//! directly with the no-proof-of-revealed-properties rule (#4): where the join
//! KEY is disclosed, the join can be checked OUTSIDE the cryptographic core (a
//! plaintext check over disclosed IRIs); only where joined VALUES must stay
//! hidden does the join go into the MPC. Q3 is exactly how much of the
//! per-pattern obliviousness padding that out-of-circuit handling collapses,
//! and for which SPARQL fragment (RQ2b).
//!
//! ## Status
//! Interface + the data describing a join only. NO protocol runs at M0; the
//! join itself (M2) is gated on the disclosure-minimisation analysis and, where
//! values are hidden, on a chosen [`crate::backend::MpcBackend`] (M3).

use crate::partial::{MpcError, PartialResult};
use oxrdf::Variable;

/// Describes ONE cross-holder join: the variable to join on and whether its
/// bound values are disclosed (so the join can be checked in the clear) or
/// hidden (so it must run inside the MPC). This is pure data — produced by the
/// (untrusted, §4.1) planner and consumed by a [`GlobalJoin`] impl. The planner
/// is untrusted: a [`GlobalJoin`] must not rely on this plan for *soundness*,
/// only as a hint for *which* protocol to run.
#[derive(Debug, Clone)]
pub struct JoinPlan {
    /// The shared variable the holders' partials are joined on.
    pub join_var: Variable,
    /// `true` if `join_var`'s bound values are disclosed global IRIs (key-on-key
    /// equi-join checkable in the clear, §4.3 step 4); `false` if they are
    /// hidden values requiring an oblivious / PSI join inside the MPC.
    pub key_disclosed: bool,
}

/// The protocol that joins multiple holders' [`PartialResult`]s on a global IRI.
///
/// Architecture §4.3 step 4: cross-source joins on global IRIs are the heart of
/// federated evaluation. Two regimes the eventual impl must distinguish:
///
/// - **Disclosed-key join** (`JoinPlan::key_disclosed == true`): a plaintext
///   equi-join over the disclosed IRIs, computed OUTSIDE the cryptographic core
///   per convention #4. This sub-case is *invariant to Q1/Q2* (no secret data)
///   and is the natural first target within M2.
/// - **Hidden-value join** (`key_disclosed == false`): requires circuit-PSI /
///   oblivious join inside a [`crate::backend::MpcBackend`]; gated on Q2/Q3 and
///   far heavier.
///
/// ## Status
/// No implementor at M0. [`Self::join`] returns [`MpcError::NotYetImplemented`].
/// The disclosed-key path is the first thing M2 will implement (it needs no
/// crypto); the hidden path waits on the backend and the Q3 cost analysis.
pub trait GlobalJoin {
    /// Join holders' partials according to `plan`, returning the combined
    /// disclosed result.
    ///
    /// Even the disclosed-key (crypto-free) path is deferred to M2 so that the
    /// disclosure-minimisation / soundness obligations (the join must not trust
    /// the untrusted planner for correctness) are designed before any code
    /// silently does a naive in-the-clear join that a later proof layer cannot
    /// back. Returns [`MpcError::NotYetImplemented`] at M0.
    fn join(&self, partials: &[PartialResult], plan: &JoinPlan) -> Result<PartialResult, MpcError>;
}
