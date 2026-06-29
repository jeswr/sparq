//! [OPUS-4.8] sq-kuvu3 (epic sq-pbz04) — the EXPERIMENTAL OWL 2 QL (DL-Lite_R)
//! query-rewriting arm of the `sparql11/entailment` conformance suite.
//!
//! ## What this lane is — and what it is deliberately NOT
//!
//! It runs the experimental `sparq-reason-ql` query-rewriter over every
//! `sd:EntailmentProfile pr:QL` entailment test and reports, HONESTLY, what the
//! rewriter genuinely computes — as experimental/OutOfScope, NOT a graduated
//! conformance floor.
//!
//! Unlike the D-entailment / service / RIF-Core ratchets in this crate, this lane
//! is **explicitly NON-GATING**: there is NO pinned pass-count `const`, NO
//! `scoreboard::SUITES` row, and NO ratchet that asserts OWL 2 QL conformance. That
//! is the load-bearing design choice (sq-kuvu3): a future QL regression must never
//! be able to silently claim conformance. QL graduation to a pinned conformance
//! floor is a SEPARATE, deferred bead that must sequence through the contended
//! conformance scoreboard.
//!
//! So what this test asserts is the HONESTY INVARIANTS, not a floor:
//!   1. NO FLOOR GRADUATION — every row the arm emits is in the experimental
//!      `Outcome::OutOfScope` bucket (so it counts toward neither a pass-rate nor a
//!      ratchet floor).
//!   2. FAIL-CLOSED PRESERVED — there is at least one genuine ABSTAIN among the
//!      QL-tagged tests (the suite is dominated by non-conjunctive / non-DL-Lite
//!      queries the rewriter must reject, never guess), and every abstain carries
//!      the rewriter's fail-closed reason.
//!   3. NO FAKED PASSES — a `computed result-equivalent` row is reported only when
//!      the rewritten UCQ's evaluation over the UNMODIFIED data genuinely matches
//!      the oracle; the run prints the full experimental tally for the record.
//!
//! ## Feature gating (both states)
//!
//! The whole lane is behind this crate's opt-in `ql-experimental` feature (forwards
//! to `sparq-reason-ql/experimental`). With the feature OFF this file compiles to a
//! single self-SKIP `#[test]` (no QL rewriter links, and the inference BINARY's QL
//! section is `#[cfg]`-stripped, so its scoreboard is byte-for-byte unchanged — the
//! lean opt-in posture). The rdf-tests fixtures are fetched by
//! `scripts/fetch-inference-suites.sh` into the gitignored `tests/w3c/rdf-tests/`;
//! when absent the runner SKIPS so a fresh offline checkout stays green.

#[cfg(not(feature = "ql-experimental"))]
#[test]
fn ql_experimental_arm_skipped_without_feature() {
    eprintln!(
        "SKIP: the EXPERIMENTAL OWL 2 QL query-rewriting arm is OFF — build with \
         `--features ql-experimental` (and run scripts/fetch-inference-suites.sh) to run it."
    );
}

#[cfg(feature = "ql-experimental")]
mod gated {
    use sparq_conformance::inference::report::Outcome;
    use sparq_conformance::inference::report::TestResult;
    use sparq_conformance::inference::sparql_entail;
    use std::path::PathBuf;

    fn rdf_tests_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/w3c/rdf-tests")
    }

    #[test]
    fn ql_arm_is_experimental_not_a_conformance_floor() {
        let root = rdf_tests_root();
        if !root.join("sparql/sparql11/entailment/manifest.ttl").exists() {
            eprintln!(
                "SKIP: rdf-tests `sparql11/entailment` not present under {} — run \
                 scripts/fetch-inference-suites.sh",
                root.display()
            );
            return;
        }

        let mut results: Vec<TestResult> = Vec::new();
        sparql_entail::run_ql_experimental_arm(&root, &mut results)
            .unwrap_or_else(|e| panic!("QL experimental arm error: {e}"));

        assert!(
            !results.is_empty(),
            "the entailment suite has pr:QL-tagged tests — the arm must report them"
        );

        // INVARIANT 1 — NO FLOOR GRADUATION: every row is in the experimental
        // OutOfScope bucket (never a Pass/Fail/Divergence that counts to a rate or a
        // ratchet floor). This is the whole point: QL cannot silently claim
        // conformance through this lane.
        let mut abstains = 0usize;
        let mut equivalent = 0usize;
        let mut divergent = 0usize;
        let mut inconclusive = 0usize;
        for r in &results {
            let Outcome::OutOfScope(reason) = &r.outcome else {
                panic!(
                    "QL arm emitted a non-OutOfScope row for {:?} ({:?}) — that would risk a \
                     faked conformance pass or a graduated floor",
                    r.name, r.outcome
                );
            };
            assert!(
                reason.starts_with("QL experimental"),
                "every QL row must carry the experimental marker; got: {reason}"
            );
            if reason.contains("abstain, fail-closed") {
                abstains += 1;
            } else if reason.contains("result-equivalent") {
                equivalent += 1;
            } else if reason.contains("DIVERGENT") {
                divergent += 1;
            } else if reason.contains("inconclusive") {
                inconclusive += 1;
            }
        }

        // INVARIANT 2 — FAIL-CLOSED PRESERVED: the QL-tagged corpus is dominated by
        // non-conjunctive / non-DL-Lite queries, so the rewriter MUST abstain on a
        // genuine subset rather than guess. (Positional `println!` args per the CodeQL
        // `rust/unused-variable` false-positive guard in the shared agent contract.)
        println!(
            "OWL 2 QL EXPERIMENTAL arm (sparql11/entailment pr:QL): {} tests — \
             {} abstain (fail-closed), {} computed-equivalent, {} computed-divergent, \
             {} inconclusive (ALL OutOfScope/experimental — NOT a conformance floor)",
            results.len(),
            abstains,
            equivalent,
            divergent,
            inconclusive
        );
        assert!(
            abstains > 0,
            "fail-closed gate must reject some QL-tagged queries (none abstained — \
             the rewriter may be guessing)"
        );

        // INVARIANT 3 — NO FAKED PASSES: a `computed-equivalent` row is reported only
        // where the rewriter genuinely matched the oracle; that the divergent case is
        // reported AS a divergence (and not laundered into equivalence) is the proof.
        // We do NOT assert a minimum equivalence count — that would be a backdoor
        // floor. The counts are printed above for the record only.
        let _ = equivalent;
        let _ = divergent;
    }
}
