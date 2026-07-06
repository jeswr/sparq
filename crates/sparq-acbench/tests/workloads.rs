//! Tests for the workload + oracle engine (W1–W4) — bead `sq-i6du2.6`.
//!
//! These tests exercise the fail-closed harness contract directly, with **fixtures built
//! in-test from the public intent-table IR** — they do NOT depend on any generator body
//! (beads `sq-i6du2.2`–`.5`), so this lane can land independently of the generators.
//!
//! The load-bearing test is [`anti_vacuity_wrong_expected_fails_harness`]: a deliberately
//! WRONG expected-decision set MUST make the harness report `Failed` and exit non-zero.
//! Without it, a harness that always passed would be indistinguishable from a correct one
//! — a benchmark an unsound implementation could win. This is the oracle's oracle.
//!
//! Bead `sq-i6du2.6` ONLY adds tests here and fills in `src/workload.rs` / `src/oracle.rs`.
//! It must NOT edit any other file.

use sparq_acbench::workload::{
    AclWrite, ChurnProbe, HarnessReport, RunOutcome, W1DecisionBatch, W2QueryCheck, W2QueryLane,
    W3ChurnScript, W4Config, W4_QUERY_SKIP_REASON,
};
use sparq_acbench::{
    AcModel, AccessMode, Audience, Condition, Decision, Effect, IntentRow, QueryClass, Request,
    Scope,
};

// ── Fixture builders (public IR only — no generator dependency) ───────────────────────

fn agent_allow(agent: &str, resource: &str) -> IntentRow {
    IntentRow {
        audience: Audience::Agent(agent.to_string()),
        scope: Scope::Resource,
        mode: AccessMode::read_only(),
        condition: Condition::None,
        effect: Effect::Allow,
        resource_uri: resource.to_string(),
    }
}

fn req(agent: &str, resource: &str) -> Request {
    Request {
        agent: agent.to_string(),
        client: None,
        resource: resource.to_string(),
        mode: AccessMode::read_only(),
    }
}

const ALICE: &str = "https://alice.example/card#me";
const BOB: &str = "https://bob.example/card#me";
const RES: &str = "https://alice.example/docs/notes.ttl";

// ── RunOutcome helpers (retained from scaffold) ───────────────────────────────────────

#[test]
fn run_outcome_passed_is_pass() {
    let outcome = RunOutcome::Passed { decisions: 10, wall_us_indicative: 0 };
    assert!(outcome.is_pass());
    assert!(!outcome.is_failure());
    assert!(!outcome.is_skipped());
}

#[test]
fn run_outcome_failed_is_failure() {
    let outcome = RunOutcome::Failed { mismatch: "deliberate test failure".to_string() };
    assert!(!outcome.is_pass());
    assert!(outcome.is_failure());
    assert_eq!(outcome.mismatch(), Some("deliberate test failure"));
}

#[test]
fn run_outcome_skipped_is_neither() {
    let outcome = RunOutcome::Skipped { reason: "blocked".to_string() };
    assert!(!outcome.is_pass());
    assert!(!outcome.is_failure());
    assert!(outcome.is_skipped());
}

// ── W1: decision micro-benchmark ──────────────────────────────────────────────────────

#[test]
fn w1_passes_when_expected_matches_oracle() {
    let intents = vec![agent_allow(ALICE, RES)];
    let batch = W1DecisionBatch {
        requests: vec![req(ALICE, RES), req(BOB, RES)],
        // Alice is granted; Bob is fail-closed Deny.
        expected: vec![Decision::Allow, Decision::Deny],
        model: AcModel::Wac,
        intents,
    };
    let outcome = batch.run_oracle();
    assert!(outcome.is_pass(), "W1 must pass when expected matches the oracle: {outcome:?}");
    if let RunOutcome::Passed { decisions, .. } = outcome {
        assert_eq!(decisions, 2);
    }
}

#[test]
fn w1_wiring_bug_length_mismatch_fails() {
    let batch = W1DecisionBatch {
        requests: vec![req(ALICE, RES)],
        expected: vec![], // deliberately wrong length
        model: AcModel::Wac,
        intents: vec![agent_allow(ALICE, RES)],
    };
    assert!(batch.run_oracle().is_failure(), "length mismatch must fail-closed");
}

/// ANTI-VACUITY (the oracle's oracle): a deliberately WRONG expected set — claiming Bob
/// (who has no grant) is Allowed — MUST make the lane fail. If this passed, the harness
/// would be vacuous and an unsound implementation could win the benchmark.
#[test]
fn anti_vacuity_wrong_expected_fails_harness() {
    let intents = vec![agent_allow(ALICE, RES)];
    let batch = W1DecisionBatch {
        requests: vec![req(BOB, RES)],
        // WRONG: Bob has no grant; fail-closed truth is Deny. Claim Allow.
        expected: vec![Decision::Allow],
        model: AcModel::Wac,
        intents,
    };
    let outcome = batch.run_oracle();
    assert!(
        outcome.is_failure(),
        "anti-vacuity: a wrong expected set MUST fail the harness, got {outcome:?}"
    );

    // …and it must propagate to a non-zero harness exit code with NO timing recorded.
    let mut report = HarnessReport::new();
    report.record("W1/anti-vacuity/WAC", outcome);
    assert_eq!(report.exit_code(), 1, "harness must exit non-zero on a failed lane");
    assert_eq!(report.failed_count(), 1);
    // No `Passed { wall_us_indicative }` may exist for the failed lane.
    assert!(
        report.lanes.iter().all(|l| !l.outcome.is_pass()),
        "a failed lane must never carry a timing number"
    );
}

/// Anti-vacuity, mirror direction: claiming an Allowed agent is Denied must ALSO fail.
#[test]
fn anti_vacuity_wrong_deny_fails_harness() {
    let intents = vec![agent_allow(ALICE, RES)];
    let batch = W1DecisionBatch {
        requests: vec![req(ALICE, RES)],
        expected: vec![Decision::Deny], // WRONG: Alice is granted.
        model: AcModel::Wac,
        intents,
    };
    assert!(batch.run_oracle().is_failure(), "wrong Deny expectation must fail");
}

/// A deliberately MISCOMPILED policy (a Deny intent that a buggy compiler renders as if it
/// grants) must not launder into an Allow: the WAC oracle skips Deny rows, so the truth
/// stays Deny and any "expected Allow" fails. Proves the compiler cannot fool the oracle.
#[test]
fn anti_vacuity_miscompiled_deny_intent_cannot_grant() {
    let deny = IntentRow { effect: Effect::Deny, ..agent_allow(ALICE, RES) };
    let batch = W1DecisionBatch {
        requests: vec![req(ALICE, RES)],
        // A miscompiled harness might "expect" the deny to have granted. It must not.
        expected: vec![Decision::Allow],
        model: AcModel::Wac,
        intents: vec![deny],
    };
    assert!(
        batch.run_oracle().is_failure(),
        "a Deny intent must never produce Allow under WAC (anti-vacuity)"
    );
}

// ── W2: access-controlled query result-set oracle ─────────────────────────────────────

#[test]
fn w2_authorized_intersection_passes() {
    // Candidate rows the query would return over ALL data; agent authorized to see only r1.
    let check = W2QueryCheck {
        class: QueryClass::Point,
        model: AcModel::Wac,
        candidate_rows: vec!["<r1> <p> <o1> .".to_string(), "<r2> <p> <o2> .".to_string()],
        authorized_rows: vec!["<r1> <p> <o1> .".to_string()],
        // Correct: the engine returned exactly the authorized ∩ candidate row.
        produced_rows: vec!["<r1> <p> <o1> .".to_string()],
    };
    assert_eq!(check.expected_rows(), vec!["<r1> <p> <o1> .".to_string()]);
    let lane = W2QueryLane { checks: vec![check] };
    assert!(lane.run_oracle().is_pass());
}

#[test]
fn w2_over_share_fails() {
    // Engine leaked an unauthorized row (r2) — an over-share MUST fail.
    let check = W2QueryCheck {
        class: QueryClass::Scan,
        model: AcModel::Acp,
        candidate_rows: vec!["<r1> <p> <o1> .".to_string(), "<r2> <p> <o2> .".to_string()],
        authorized_rows: vec!["<r1> <p> <o1> .".to_string()],
        produced_rows: vec!["<r1> <p> <o1> .".to_string(), "<r2> <p> <o2> .".to_string()],
    };
    let lane = W2QueryLane { checks: vec![check] };
    let outcome = lane.run_oracle();
    assert!(outcome.is_failure(), "over-share must fail: {outcome:?}");
    assert!(outcome.mismatch().unwrap().contains("over-shared"));
}

#[test]
fn w2_under_share_fails() {
    // Engine dropped an authorized row — an under-share MUST fail too.
    let check = W2QueryCheck {
        class: QueryClass::Aggregate,
        model: AcModel::Odrl,
        candidate_rows: vec!["<r1> <p> <o1> .".to_string(), "<r2> <p> <o2> .".to_string()],
        authorized_rows: vec!["<r1> <p> <o1> .".to_string(), "<r2> <p> <o2> .".to_string()],
        produced_rows: vec!["<r1> <p> <o1> .".to_string()], // dropped r2
    };
    let lane = W2QueryLane { checks: vec![check] };
    let outcome = lane.run_oracle();
    assert!(outcome.is_failure());
    assert!(outcome.mismatch().unwrap().contains("missing"));
}

// ── W3: ACL-write + invalidation churn ────────────────────────────────────────────────

#[test]
fn w3_revoke_produces_deny_after_step() {
    // Start: Alice granted. Step 1: revoke. Probe before revoke = Allow; after = Deny.
    let grant = agent_allow(ALICE, RES);
    let script = W3ChurnScript {
        initial_intents: vec![grant.clone()],
        writes: vec![AclWrite::Revoke(grant.clone())],
        probes: vec![
            ChurnProbe { request: req(ALICE, RES), after_step: 0, expected: Decision::Allow },
            ChurnProbe { request: req(ALICE, RES), after_step: 1, expected: Decision::Deny },
        ],
        model: AcModel::Wac,
    };
    let outcome = script.run_oracle();
    assert!(outcome.is_pass(), "W3 revoke churn must pass: {outcome:?}");
}

#[test]
fn w3_grant_then_probe_allows() {
    let grant = agent_allow(BOB, RES);
    let script = W3ChurnScript {
        initial_intents: vec![],
        writes: vec![AclWrite::Grant(grant)],
        probes: vec![
            ChurnProbe { request: req(BOB, RES), after_step: 0, expected: Decision::Deny },
            ChurnProbe { request: req(BOB, RES), after_step: 1, expected: Decision::Allow },
        ],
        model: AcModel::Acp,
    };
    assert!(script.run_oracle().is_pass());
}

/// A STALE grant (expecting Allow after a revoke) MUST fail — this is the invalidation
/// lane's whole reason to exist.
#[test]
fn w3_stale_grant_after_revoke_fails() {
    let grant = agent_allow(ALICE, RES);
    let script = W3ChurnScript {
        initial_intents: vec![grant.clone()],
        writes: vec![AclWrite::Revoke(grant)],
        // WRONG: after the revoke the truth is Deny, but claim the (stale) grant survives.
        probes: vec![ChurnProbe {
            request: req(ALICE, RES),
            after_step: 1,
            expected: Decision::Allow,
        }],
        model: AcModel::Wac,
    };
    let outcome = script.run_oracle();
    assert!(outcome.is_failure(), "a surviving stale grant MUST fail the churn lane");
    assert!(outcome.mismatch().unwrap().contains("stale"));
}

#[test]
fn w3_probe_past_end_is_wiring_failure() {
    let script = W3ChurnScript {
        initial_intents: vec![],
        writes: vec![],
        probes: vec![ChurnProbe {
            request: req(ALICE, RES),
            after_step: 5, // > 0 writes
            expected: Decision::Deny,
        }],
        model: AcModel::Wac,
    };
    assert!(script.run_oracle().is_failure());
}

// ── W4: concurrent readers ────────────────────────────────────────────────────────────

#[test]
fn w4_decision_lane_concurrent_passes() {
    let intents = vec![agent_allow(ALICE, RES)];
    let batch = W1DecisionBatch {
        requests: vec![req(ALICE, RES), req(BOB, RES)],
        expected: vec![Decision::Allow, Decision::Deny],
        model: AcModel::Wac,
        intents,
    };
    let cfg = W4Config { n_threads: 4, batches_per_thread: 8, model: AcModel::Wac, sf: 1 };
    let outcome = cfg.run(&batch);
    assert!(
        outcome.decision_lane.is_pass(),
        "W4 concurrent decision lane must pass: {:?}",
        outcome.decision_lane
    );
    // Concurrent readers must agree with the single-threaded oracle (determinism).
    if let RunOutcome::Passed { decisions, .. } = outcome.decision_lane {
        assert_eq!(decisions, 4 * 8 * 2, "every thread×batch×request must be checked");
    }
}

#[test]
fn w4_decision_lane_propagates_a_mismatch() {
    let intents = vec![agent_allow(ALICE, RES)];
    let batch = W1DecisionBatch {
        requests: vec![req(BOB, RES)],
        expected: vec![Decision::Allow], // WRONG: Bob is Deny.
        model: AcModel::Wac,
        intents,
    };
    let cfg = W4Config { n_threads: 3, batches_per_thread: 2, model: AcModel::Wac, sf: 1 };
    let outcome = cfg.run(&batch);
    assert!(
        outcome.decision_lane.is_failure(),
        "a concurrent mismatch must fail the decision lane"
    );
}

/// The W4 query sub-lane is Skipped in this oracle crate, with an ACCURATE recorded reason
/// (#1569 / PR #1612 has MERGED — the skip is architectural, not a false "unmerged dep").
#[test]
fn w4_query_sublane_skipped_with_accurate_reason() {
    let cfg = W4Config { n_threads: 2, batches_per_thread: 1, model: AcModel::Wac, sf: 1 };
    let batch = W1DecisionBatch {
        requests: vec![],
        expected: vec![],
        model: AcModel::Wac,
        intents: vec![],
    };
    let outcome = cfg.run(&batch);
    assert!(
        matches!(outcome.query_lane, RunOutcome::Skipped { .. }),
        "W4 query sub-lane must be Skipped"
    );
    if let RunOutcome::Skipped { reason } = &outcome.query_lane {
        assert_eq!(reason, W4_QUERY_SKIP_REASON);
        // The reason must NOT claim the dependency is still unmerged (it has merged).
        assert!(
            reason.contains("has merged"),
            "the skip reason must record that #1569/#1612 has merged"
        );
        assert!(
            !reason.contains("blocked: #1569"),
            "must not use the stale 'blocked: #1569' wording — that PR merged"
        );
    }
}

/// `run_params` (the scaffold `run(&params)` shape) still yields a Skipped query sub-lane.
#[test]
fn w4_run_params_query_sublane_skipped() {
    use sparq_acbench::GenParams;
    let cfg = W4Config { n_threads: 2, batches_per_thread: 1, model: AcModel::Wac, sf: 1 };
    let outcome = cfg.run_params(&GenParams::smoke());
    assert!(matches!(outcome.query_lane, RunOutcome::Skipped { .. }));
    assert!(outcome.decision_lane.is_pass(), "empty batch trivially passes");
}

// ── Harness aggregator: fail-closed exit code ─────────────────────────────────────────

#[test]
fn harness_all_pass_exits_zero() {
    let mut report = HarnessReport::new();
    report.record("W1/x/WAC", RunOutcome::Passed { decisions: 1, wall_us_indicative: 0 });
    report.record(
        "W4-query/x/WAC",
        RunOutcome::Skipped { reason: W4_QUERY_SKIP_REASON.to_string() },
    );
    assert_eq!(report.exit_code(), 0, "all-pass (+ skips) exits zero");
    assert_eq!(report.skipped_count(), 1);
    assert!(!report.any_failed());
}

#[test]
fn harness_any_fail_exits_nonzero() {
    let mut report = HarnessReport::new();
    report.record("W1/x/WAC", RunOutcome::Passed { decisions: 1, wall_us_indicative: 0 });
    report.record("W3/x/WAC", RunOutcome::Failed { mismatch: "stale grant".to_string() });
    assert_eq!(report.exit_code(), 1, "any failed lane exits non-zero");
    assert_eq!(report.failed_count(), 1);
}
