//! U2 — Commercial project-management generator (bead `sq-i6du2.3`).
//!
//! Generates a Graphmetrix-inspired dataset with org → project → site → document-set
//! hierarchy, cross-org subcontractor access, role/team group reuse, and all-except
//! (deny-shaped) access intents.
//!
//! # AC shape stressed
//! - Role/team groups with **cross-org group reuse** (same group IRI referenced by
//!   multiple container policies).
//! - Wide flat containers (many documents per project).
//! - "All-except" intents: ACP/ODRL native; WAC inexpressible → enum-allow blowup.
//! - Handover/revocation churn (W3 workload): grant-then-revoke sequences.
//! - `policies_per_resource` drives accreted policy fan-in.
//!
//! # File ownership
//! **Only bead `sq-i6du2.3` edits this file.**
//!
//! # Status
//! Scaffold stub.

use crate::{CompiledPolicy, ExpectedDecision, GenParams, IntentRow, QueryFixture};

/// Output of the U2 commercial-project-management generator.
pub struct ProjectMgmtDataset {
    /// N-Quads lines forming the data graph.
    pub data_nquads: Vec<String>,
    /// Compiled WAC policy graph.
    pub wac_policy: Vec<CompiledPolicy>,
    /// Compiled ACP policy graph.
    pub acp_policy: Vec<CompiledPolicy>,
    /// Compiled ODRL policy graph.
    pub odrl_policy: Vec<CompiledPolicy>,
    /// Model-agnostic intent table.
    pub intents: Vec<IntentRow>,
    /// Request tuples with expected decisions.
    pub expected_decisions: Vec<ExpectedDecision>,
    /// W2 SPARQL query fixtures.
    pub queries: Vec<QueryFixture>,
    /// W3 churn steps: interleaved grant/revoke writes with expected decision deltas.
    pub churn_steps: Vec<ChurnStep>,
}

/// A W3 churn step: one ACL write + the expected decision delta it produces.
#[derive(Debug, Clone)]
pub struct ChurnStep {
    /// Description of the write (for diagnostics).
    pub description: String,
    /// The grant or revoke operation (as N-Quads to add or remove).
    /// N-Quads triples to add (policy graph update).
    pub delta_add: Vec<String>,
    /// N-Quads triples to remove (policy graph update).
    pub delta_remove: Vec<String>,
    /// Expected decision changes: `(request, model, new_decision)`.
    pub expected_deltas: Vec<ExpectedDecision>,
}

/// Generate a U2 commercial-project-management dataset.
///
/// # Invariants
/// - Determinism: same `params` → same output.
/// - All-except intents in `intents` carry per-model [`crate::ExpressibilityEntry`]
///   entries in the expressibility matrix.
/// - W3 churn steps have exact expected decision deltas by construction.
///
/// # Panics
/// Panics if `params.validate()` fails.
#[must_use]
pub fn generate(_params: &GenParams) -> ProjectMgmtDataset {
    todo!("sq-i6du2.3: implement U2 project-management generator")
}
