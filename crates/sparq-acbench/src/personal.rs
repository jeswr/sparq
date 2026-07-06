//! U1 — Personal data storage generator (bead `sq-i6du2.2`).
//!
//! Generates a Solid pod-shaped dataset with owner-centric access control,
//! deep container inheritance, friend/family group nesting, and app-restricted
//! access (ACP `acp:client`; WAC `acl:origin` approximation).
//!
//! # AC shape stressed
//! - Container inheritance depth (driven by `GenParams::container_depth`).
//! - Owner-centric: most resources are owned by one agent, with shared sub-trees.
//! - Friend/family groups: `vcard:Group` chains at depth ≤ `group_nesting_depth`.
//! - Public/private/shared mix driven by `GenParams::mix`.
//! - App-restricted: a fraction of shared resources carry `acp:client` / `acl:origin`.
//!
//! # File ownership
//! **Only bead `sq-i6du2.2` edits this file.**
//! All other beads in the `sq-i6du2` family leave it untouched.
//!
//! # Status
//! Scaffold stub — function signatures and doc-tests are in place so the
//! `sq-i6du2.2` implementor can fill in the body without touching `lib.rs`.

use crate::{
    AcModel, CompiledPolicy, Decision, ExpectedDecision, GenParams, IntentRow,
    QueryFixture, Request,
};

/// Output of the U1 personal-data-storage generator.
///
/// All fields are deterministic for a given [`GenParams`] (same seed, same output).
pub struct PersonalDataset {
    /// N-Quads lines forming the data graph (resources, containers, metadata).
    pub data_nquads: Vec<String>,
    /// Compiled WAC policy graph.
    pub wac_policy: Vec<CompiledPolicy>,
    /// Compiled ACP policy graph.
    pub acp_policy: Vec<CompiledPolicy>,
    /// Compiled ODRL policy graph.
    pub odrl_policy: Vec<CompiledPolicy>,
    /// The model-agnostic intent table (one row per access-control decision point).
    pub intents: Vec<IntentRow>,
    /// Request tuples with expected decisions for W1 / W3 oracle checking.
    pub expected_decisions: Vec<ExpectedDecision>,
    /// W2 SPARQL queries with expected result sets (closed-form, as N-Quads row sets).
    pub queries: Vec<QueryFixture>,
}

/// Generate a U1 personal-data-storage dataset.
///
/// # Invariants
/// - **Determinism**: same `params` → same `PersonalDataset` every call.
/// - **Fail-closed oracle**: every `ExpectedDecision` with no matching allow rule
///   has `decision = Deny`.
/// - **Independent oracle**: expected decisions are computed from the intent table
///   without calling any sparq evaluator.
///
/// # Panics
/// Panics if `params.validate()` fails.
#[must_use]
pub fn generate(_params: &GenParams) -> PersonalDataset {
    todo!("sq-i6du2.2: implement U1 personal-data-storage generator")
}

/// Smoke-test helper: generate U1 at `GenParams::smoke()` and return the first
/// `n` expected decisions (WAC model).
///
/// Used by the scaffold-level tests to confirm the function is callable
/// without actually running the generator body (which is a `todo!()`).
///
/// # Panics
/// Always panics (delegates to `generate` which is a `todo!()`).
#[doc(hidden)]
#[must_use]
pub fn smoke_decisions(n: usize) -> Vec<(Request, Decision)> {
    let params = GenParams::smoke();
    let ds = generate(&params);
    ds.expected_decisions
        .iter()
        .filter(|ed| ed.model == AcModel::Wac)
        .take(n)
        .map(|ed| (ed.request.clone(), ed.decision.clone()))
        .collect()
}
