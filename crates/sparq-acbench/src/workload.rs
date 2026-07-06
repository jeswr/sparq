//! Workload engine stubs (W1–W4) — bead `sq-i6du2.6`.
//!
//! This module provides the typed interfaces for the four benchmark workloads defined
//! in §2.1 of `research/ac-query-benchmark.md`. The full implementation lives in
//! bead `sq-i6du2.6` (oracle + workload engine). The scaffold defines the types and
//! `todo!()` stubs so beads `.2`–`.5` can import them without touching `lib.rs`.
//!
//! # Fail-closed harness contract (B6's responsibility)
//! - Any decision mismatch → nonzero exit; no timing is reported for failed lanes.
//! - Any result-set mismatch → nonzero exit.
//! - Any post-churn stale grant → nonzero exit.
//! - W4 query sub-lane emits `RunOutcome::Skipped { reason }` until the `&self`
//!   read-side (#1569) lands.
//!
//! # File ownership
//! **Only bead `sq-i6du2.6` edits this file.**

use crate::{AcModel, Decision, GenParams, Request};

/// The outcome of one workload lane run.
#[derive(Debug, Clone)]
pub enum RunOutcome {
    /// The lane passed all oracle checks.
    Passed {
        /// Number of decisions evaluated.
        decisions: usize,
        /// Hint at wall-clock; NON-CANONICAL on a work-box (labels required by harness).
        wall_us_indicative: u64,
    },
    /// The lane failed an oracle check.
    Failed {
        /// Human-readable description of the first mismatch found.
        mismatch: String,
    },
    /// The lane was explicitly skipped (blocked on a dependency).
    Skipped {
        /// Reason for skip (e.g. `"blocked: #1569"`).
        reason: String,
    },
}

impl RunOutcome {
    /// Returns `true` iff the outcome is a pass. Skipped is neither pass nor fail.
    #[must_use]
    pub fn is_pass(&self) -> bool {
        matches!(self, RunOutcome::Passed { .. })
    }

    /// Returns `true` iff the outcome is a failure.
    #[must_use]
    pub fn is_failure(&self) -> bool {
        matches!(self, RunOutcome::Failed { .. })
    }
}

/// A W1 decision-batch workload: batches of `(agent, client, resource, mode)` tuples
/// through the access-control evaluator.
pub struct W1DecisionBatch {
    /// The requests to evaluate.
    pub requests: Vec<Request>,
    /// The expected decision for each request (parallel to `requests`).
    pub expected: Vec<Decision>,
    /// The model to evaluate against.
    pub model: AcModel,
}

impl W1DecisionBatch {
    /// Run this batch through the by-construction oracle and return a `RunOutcome`.
    ///
    /// The oracle is consulted against the generator's intent table (passed via
    /// the generator's `expected_decisions` field, not re-derived here).
    ///
    /// # Fail-closed
    /// Any mismatch returns `RunOutcome::Failed`.
    #[must_use]
    pub fn run_oracle(&self) -> RunOutcome {
        todo!("sq-i6du2.6: implement W1 decision-batch oracle runner")
    }
}

/// Configuration for a W4 concurrent-reader run.
pub struct W4Config {
    /// Number of parallel reader threads.
    pub n_threads: usize,
    /// Number of W1 batches per thread.
    pub batches_per_thread: usize,
    /// The model to evaluate against.
    pub model: AcModel,
    /// Scale factor (from `GenParams::sf`).
    pub sf: u32,
}

impl W4Config {
    /// Run the W4 concurrent-reader workload.
    ///
    /// # Skipped (until #1569 lands)
    /// The W4 query sub-lane (Q-point via `query_as`) emits
    /// `RunOutcome::Skipped { reason: "blocked: #1569" }` until the `&self`
    /// read-side in `sparq-solid` lands. The decision sub-lane (W1 via `decide_batch`)
    /// is available today and runs normally.
    #[must_use]
    pub fn run(&self, _params: &GenParams) -> RunOutcome {
        todo!("sq-i6du2.6: implement W4 concurrent-reader runner")
    }
}
