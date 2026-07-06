//! [OPUS-4.8] sq-e5atd (epic sq-pbz04) — the D-entailment (datatype / value-space)
//! conformance lane, wired as a RATCHETED gate that mirrors the SPARQL / SHACL /
//! GeoSPARQL / Solid / JSON-LD ratchets in this crate (crate-local `cargo test` +
//! a pinned pass-count FLOOR that may only RISE, registered in the central
//! `scoreboard::SUITES` and guarded by `tests/scoreboard_floors.rs`).
//!
//! ## What is gated
//!
//! The genuinely D-ONLY tests of the W3C `sparql11/entailment` suite — those whose
//! `sd:entailmentRegime` is `ent:D` WITHOUT any stronger RDFS/RDF/OWL-RDF-Based
//! regime (those are exercised under their own regime by the inference binary). Each
//! is driven through the REAL D path: the premise data is materialized through
//! `sparq_reason::Profile::D` (the rdfD1 datatype-typing closure, with the typed
//! value-space comparator — `"1"^^xsd:integer` ≡ `"1.0"^^xsd:decimal`, NOT an f64
//! fast path), then the query is run over the closure through the SAME
//! evaluation/comparison machinery as the gating SPARQL harness, with the SPARQL
//! 1.1 Entailment-Regimes answer restriction applied (literal-subject + surrogate
//! blank-node bindings are never answers).
//!
//! These tests previously sat in the inference scoreboard's OutOfScope bucket
//! ("entailment regime(s) D not supported"); this lane GRADUATES them to Pass.
//!
//! ## Honest floor + the tracked-not-asserted remainder
//!
//! `D_ENTAIL_FLOOR` is the MEASURED D-only pass count at the pinned rdf-tests
//! revision, NOT an aspirational target. At the current pin the suite contains a
//! single D-only test (`d-ent-01`), which passes — so the floor is 1. The broader
//! D-entailment surface (D-inconsistency / ill-typed-literal clashes, value-space
//! subset reasoning across the integer/decimal/temporal spaces, the rdf-mt
//! recognized-datatype tests that ride RDFS rather than D-only) is exercised
//! elsewhere — the inference binary's rdf-mt section, and `sparq-reason`'s own
//! `dtype` unit tests for the value-space invariant — and the rest is honestly
//! tracked-not-asserted (a child of the D-entailment epic), never skip-laundered to
//! inflate this count. The floor may only RISE as the D-only corpus grows.
//!
//! ## Feature gating (both states)
//!
//! The whole lane is behind this crate's opt-in `d-entail` feature (forwards to
//! `sparq-reason/d-entail`). With the feature OFF this file compiles to a single
//! self-SKIP `#[test]` (no `Profile::D` code links, and the inference BINARY keeps
//! these tests OutOfScope, so its ratchet floor is byte-for-byte unchanged — the
//! lean opt-in posture). With it ON the runner executes the D-only tests and
//! asserts the pinned floor. The rdf-tests fixtures are fetched by
//! `scripts/fetch-inference-suites.sh` into the gitignored `tests/w3c/rdf-tests/`;
//! when absent the runner SKIPS so a fresh offline checkout stays green.

// [OPUS-4.8] When the lane feature is OFF the runner is a single self-SKIP test so
// the default + `--workspace` builds neither link the D materializer nor go red on
// a fresh checkout. (cfg gate, not a runtime branch, so zero D-entailment code
// compiles in the default state — the lean-core invariant.)
#[cfg(not(feature = "d-entail"))]
#[test]
fn d_entail_suite_skipped_without_feature() {
    eprintln!(
        "SKIP: D-entailment conformance lane is OFF — build with \
         `--features d-entail` (and run scripts/fetch-inference-suites.sh) to run it."
    );
}

#[cfg(feature = "d-entail")]
mod gated {
    use sparq_conformance::inference::report::{Outcome, TestResult};
    use sparq_conformance::inference::sparql_entail;
    use std::path::PathBuf;

    /// The D-entailment pass floor (the RATCHET). MEASURED D-only pass count at the
    /// pinned rdf-tests revision; MIRRORED in the central scoreboard
    /// (`scoreboard::SUITES`) and read textually by the guard test
    /// `tests/scoreboard_floors.rs`. It may only RISE — never lower it (raise as the
    /// D-only corpus / the materializer's value-space coverage grow). This is the
    /// ACTUAL current pass count, not an aspirational target.
    pub const D_ENTAIL_FLOOR: usize = 1;

    fn rdf_tests_root() -> PathBuf {
        // CARGO_MANIFEST_DIR is crates/sparq-conformance; the data lives at the
        // workspace-root `tests/w3c/rdf-tests` (the same clone the SPARQL +
        // inference harnesses use, fetched by scripts/fetch-inference-suites.sh).
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/w3c/rdf-tests")
    }

    #[test]
    fn d_entailment_ratchet() {
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
        sparql_entail::run_d_regime_suite(&root, &mut results)
            .unwrap_or_else(|e| panic!("D-regime suite error: {e}"));

        let mut pass = 0usize;
        let mut fail = 0usize;
        let mut oos = 0usize;
        let mut fails: Vec<String> = Vec::new();
        for r in &results {
            match &r.outcome {
                Outcome::Pass | Outcome::Divergence(_, _) => pass += 1,
                Outcome::Fail(why) => {
                    fail += 1;
                    fails.push(format!("{}: {}", r.name, why));
                }
                Outcome::OutOfScope(_) => oos += 1,
            }
        }

        // [OPUS-4.8] The CI ratchet greps this exact `TOTAL d-entail` line (pass
        // count in field $3), mirroring the JSON-LD lane's `TOTAL <cat>` shape.
        // Positional `format!`/`println!` args (not inline `{pass}`) per the CodeQL
        // `rust/unused-variable` false-positive guard in the shared agent contract.
        println!("\nW3C SPARQL 1.1 D-entailment conformance (pinned rdf-tests)");
        println!(
            "TOTAL d-entail {} {} {} (floor {})",
            pass, fail, oos, D_ENTAIL_FLOOR
        );
        if !fails.is_empty() {
            println!("D-entailment FAILS:");
            for f in &fails {
                println!("  - {}", f);
            }
        }

        assert_eq!(fail, 0, "D-entailment lane has failing tests: {:?}", fails);
        assert!(
            pass >= D_ENTAIL_FLOOR,
            "D-entailment pass count {} regressed below the ratchet floor {}",
            pass,
            D_ENTAIL_FLOOR
        );
    }
}
