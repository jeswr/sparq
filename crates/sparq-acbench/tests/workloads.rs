//! Tests for the workload + oracle engine (W1–W4) — bead `sq-i6du2.6`.
//!
//! Scaffold-tier tests confirm the `RunOutcome` API compiles and the
//! `is_pass`/`is_failure` helpers behave correctly. Full workload tests are
//! `#[ignore]`-tagged pending bead `sq-i6du2.6`.
//!
//! Bead `sq-i6du2.6` ONLY adds tests here and fills in `src/workload.rs` / `src/oracle.rs`.
//! It must NOT edit any other file.

use sparq_acbench::workload::RunOutcome;

#[test]
fn run_outcome_passed_is_pass() {
    let outcome = RunOutcome::Passed { decisions: 10, wall_us_indicative: 0 };
    assert!(outcome.is_pass());
    assert!(!outcome.is_failure());
}

#[test]
fn run_outcome_failed_is_failure() {
    let outcome = RunOutcome::Failed { mismatch: "deliberate test failure".to_string() };
    assert!(!outcome.is_pass());
    assert!(outcome.is_failure());
}

#[test]
fn run_outcome_skipped_is_neither() {
    let outcome = RunOutcome::Skipped { reason: "blocked: #1569".to_string() };
    assert!(!outcome.is_pass());
    assert!(!outcome.is_failure());
}

/// Placeholder: bead `sq-i6du2.6` fills in the anti-vacuity test — a deliberately
/// miscompiled policy fixture must cause the workload harness to return `Failed`,
/// not `Passed`.
#[test]
#[ignore = "sq-i6du2.6: implement workload engine body first"]
fn anti_vacuity_miscompiled_policy_causes_failure() {
    // B6 implements this as the "oracle's oracle": generate a corpus where one policy
    // is deliberately miscompiled (e.g. a Deny intent rendered as Allow), run W1,
    // and assert the harness returns RunOutcome::Failed, not Passed.
    todo!("sq-i6du2.6")
}

/// Placeholder: bead `sq-i6du2.6` verifies W4 query sub-lane emits Skipped.
#[test]
#[ignore = "sq-i6du2.6: implement workload engine body first"]
fn w4_query_sublane_skipped_until_1569() {
    use sparq_acbench::{AcModel, GenParams, workload::W4Config};
    let cfg = W4Config {
        n_threads: 2,
        batches_per_thread: 1,
        model: AcModel::Wac,
        sf: 1,
    };
    let params = GenParams::smoke();
    let outcome = cfg.run(&params);
    // Until #1569 lands, the query sub-lane must emit Skipped, not Passed or Failed.
    assert!(matches!(outcome, RunOutcome::Skipped { .. }), "W4 query sublane must be Skipped");
}
