//! U4 — Research-data consortium generator (bead `sq-i6du2.5`).
//!
//! Generates a dataset modelling a research consortium: datasets, papers-in-progress,
//! instruments, and consortium membership rolls with temporal embargo constraints,
//! very large flat groups, public-after-embargo flips (churn workload), and
//! authenticated-agent-wide grants.
//!
//! # AC shape stressed
//! - Temporal ODRL constraints (`dateTime` embargo-until): W4's churn workload flips
//!   resources from embargoed to public after the embargo window.
//! - Very large flat groups: `members_per_group` drives member count (no nesting depth
//!   needed — flat consortium membership roll).
//! - Authenticated-agent-wide grants: the authenticated access class at scale.
//! - Embargo flips produce exact W3 decision deltas by construction.
//!
//! # File ownership
//! **Only bead `sq-i6du2.5` edits this file.**
//!
//! # Status
//! Scaffold stub.

use crate::{CompiledPolicy, ExpectedDecision, GenParams, IntentRow, QueryFixture};
use crate::project_mgmt::ChurnStep;

/// Output of the U4 research-data-consortium generator.
pub struct ConsortiumDataset {
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
    /// W3 embargo-flip churn steps with exact expected decision deltas.
    pub embargo_flips: Vec<ChurnStep>,
}

/// Generate a U4 research-data-consortium dataset.
///
/// # Invariants
/// - Determinism: same `params` → same output.
/// - Embargo flips in `embargo_flips` produce exact decision deltas by construction
///   (the oracle evaluates the post-flip intent table).
///
/// # Panics
/// Panics if `params.validate()` fails.
#[must_use]
pub fn generate(_params: &GenParams) -> ConsortiumDataset {
    todo!("sq-i6du2.5: implement U4 research-data-consortium generator")
}
