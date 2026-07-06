//! U3 — Financial services generator (bead `sq-i6du2.4`).
//!
//! Generates a dataset modelling a financial institution: clients, accounts,
//! transactions, advisory documents, auditors, and regulators with strict
//! compartmentalization, high policy fan-in per resource, ODRL duties/constraints
//! (retention windows, purpose-of-use, count-limited access), and audit-trail reads.
//!
//! # AC shape stressed
//! - Strict compartmentalization: low public-audience mix.
//! - High policy fan-in: `policies_per_resource` drives ODRL policy accrual.
//! - ODRL duties/constraints (levels 1–3 per `ConstraintComplexity`): constraint-bearing
//!   intents are ODRL-only in the expressibility matrix — WAC and ACP emit nothing for
//!   them.
//! - Audit-trail: read-only access for auditor/regulator agents with purpose constraints.
//!
//! # File ownership
//! **Only bead `sq-i6du2.4` edits this file.**
//!
//! # Status
//! Scaffold stub.

use crate::{CompiledPolicy, ExpectedDecision, GenParams, IntentRow, QueryFixture};

/// Output of the U3 financial-services generator.
pub struct FinancialDataset {
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
}

/// Generate a U3 financial-services dataset.
///
/// # Invariants
/// - Determinism: same `params` → same output.
/// - Constraint-bearing intents (`condition ≠ None`) appear in the intent table and have
///   [`crate::Expressibility::Unsupported`] for WAC and ACP in the expressibility matrix.
/// - ODRL-only intents: `compile_wac` and `compile_acp` return empty triples for them.
///
/// # Panics
/// Panics if `params.validate()` fails.
#[must_use]
pub fn generate(_params: &GenParams) -> FinancialDataset {
    todo!("sq-i6du2.4: implement U3 financial-services generator")
}
