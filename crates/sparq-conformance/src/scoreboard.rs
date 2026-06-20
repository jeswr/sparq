//! [OPUS-4.8] sq-ncvq.16 — the CENTRAL conformance SCOREBOARD registry.
//!
//! Before this module the project's conformance ratchets were reported in
//! several disconnected places: the W3C SPARQL suites (this crate's
//! `sparq-conformance` binary), the inference suites (`sparq-inference-conformance`),
//! the W3C SHACL core + SHACL-SPARQL suites (crate-local `cargo test` runners in
//! `sparq-shacl`), the OGC GeoSPARQL topology ratchet (a crate-local `cargo test`
//! in `sparq-geo`), the Solid WAC + ACP library-level decision-parity suites
//! (crate-local `cargo test` runners in `sparq-solid`), and the SolidLab ODRL Test
//! Suite (a crate-local `cargo test` runner in `sparq-policy`, sq-tmsd6). The
//! drift-scanner
//! (`scripts/drift-scan.py` §5.E `conformance-split`) flagged the SHACL + geo
//! ratchets (sq-ncvq.16), then the Solid WAC/ACP ratchets (sq-j174), as living
//! OUTSIDE the central scoreboard, so no single artifact answered "what conformance
//! does sparq claim, and at what floor?".
//!
//! This registry is that single source of truth. It enumerates EVERY conformance
//! suite the project ratchets — across crates — with, for each: the spec family,
//! the runner that executes it (this crate's binaries, or the named crate-local
//! `cargo test`), the CI job that gates it, and its RATCHET FLOOR (a pass /
//! pass+divergence count that may only RISE). The `sparq-conformance-scoreboard`
//! binary renders it as ONE markdown table so CI (and humans) see all suites at
//! a glance.
//!
//! ## Why a registry, not a merged runner
//!
//! The SHACL + geo + Solid runners are NOT re-implemented here. They depend on
//! `sparq-shacl` / `sparq-geo` / `sparq-solid`, and this crate must stay free of
//! those deps — exactly the constraint `crates/sparq-shacl/tests/w3c_core.rs`
//! records in its own header ("Manifest-walking helpers are modelled on
//! `sparq-conformance`'s (copied, not shared — that crate is dev-only and must not
//! become a dependency)"). Pulling sparq-shacl/sparq-geo/sparq-solid into the
//! conformance crate would invert that and couple the SPARQL/inference scoreboard
//! to the SHACL/geo/Solid build.
//! So consolidation happens at the REPORTING layer: the floors live here as the
//! authoritative list, the runners stay where their dependencies are, and a guard
//! test (`tests/scoreboard_floors.rs`) hermetically reads the crate-local
//! sources to prove the centrally-declared floors still match the runner-enforced
//! ones (so the two can never silently diverge).

/// How a suite is executed — the command CI runs to enforce its ratchet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Runner {
    /// This crate's `sparq-conformance` binary (W3C SPARQL query/update/syntax).
    SparqlBinary,
    /// This crate's `sparq-inference-conformance` binary (rdf-mt / OWL 2 RL / N3 /
    /// entailment / rdf-turtle).
    InferenceBinary,
    /// A crate-local `cargo test` runner: `(crate, test target)`. The runner
    /// asserts its own pinned floor; CI re-checks the printed scoreboard.
    CrateTest {
        /// Crate the test lives in, e.g. `sparq-shacl`.
        krate: &'static str,
        /// `--test <target>` name, e.g. `w3c_core`.
        target: &'static str,
    },
    /// [OPUS-4.8] sq-oy1f.2 — a crate-local `cargo test` runner that is OFF by
    /// default and only runs under an opt-in cargo `feature` (e.g. the JSON-LD
    /// lane behind `jsonld-suite`). Same as [`CrateTest`](Runner::CrateTest) but
    /// the rendered command names the required feature, so the scoreboard's
    /// "how to run" column is correct for a feature-gated ratchet (the lane
    /// compiles out / self-skips when the feature is off — the lean-core posture).
    FeatureGatedCrateTest {
        /// Crate the test lives in, e.g. `sparq-conformance`.
        krate: &'static str,
        /// `--test <target>` name, e.g. `jsonld_suite`.
        target: &'static str,
        /// The cargo `--features` flag the lane requires, e.g. `jsonld-suite`.
        feature: &'static str,
    },
}

impl Runner {
    /// The shell command a maintainer (or CI) runs to enforce this suite's
    /// ratchet, for the report's "how to run" column.
    pub fn command(&self) -> String {
        match self {
            Runner::SparqlBinary => {
                "cargo run --release -p sparq-conformance --bin sparq-conformance".into()
            }
            Runner::InferenceBinary => {
                "cargo run --release -p sparq-conformance --bin sparq-inference-conformance".into()
            }
            Runner::CrateTest { krate, target } => {
                format!("cargo test -p {krate} --test {target}")
            }
            Runner::FeatureGatedCrateTest { krate, target, feature } => {
                format!("cargo test -p {krate} --features {feature} --test {target}")
            }
        }
    }
}

/// One conformance suite the project ratchets.
#[derive(Clone, Copy, Debug)]
pub struct Suite {
    /// Human label, e.g. "W3C SHACL core".
    pub label: &'static str,
    /// Spec family / governing body, e.g. "W3C SHACL", "OGC GeoSPARQL".
    pub family: &'static str,
    /// How it is run.
    pub runner: Runner,
    /// The CI job (in `.github/workflows/ci.yml`) that gates it.
    pub ci_job: &'static str,
    /// The RATCHET FLOOR — the minimum pass (or pass+documented-divergence) count
    /// CI enforces. May only RISE. For the binary-runner suites this is the count
    /// parsed from the report's Overall line; for the crate-test suites it is the
    /// `const` floor the runner asserts (kept in sync by the guard test).
    pub ratchet_floor: usize,
    /// What "floor" counts (for the report footnote) — pass, or pass+divergence.
    pub floor_basis: &'static str,
    /// One-line scope note.
    pub note: &'static str,
}

/// The CENTRAL registry: every conformance suite, in report order. Adding a new
/// ratchet anywhere in the workspace means adding a row HERE (and the drift
/// scanner's `conformance-split` exemption references this list).
///
/// FLOORS — keep each in lock-step with its enforcer (the guard test checks the
/// crate-test ones against source):
/// * SPARQL  1229  — `ci.yml` job `conformance` (`RATCHET=1229`).
/// * inference 1967 — `ci.yml` job `inference-conformance` (`RATCHET=1967`).
/// * SHACL core 98 — `sparq-shacl` `w3c_core.rs` `BASELINE_PASS = 98`.
/// * SHACL-SPARQL 5 — `sparq-shacl` `w3c_sparql.rs` `SHACL_SPARQL_FLOOR = 5`.
/// * OGC GeoSPARQL 119 — `sparq-geo` `ogc_compliance_ratchet.rs`
///   `OGC_RATCHET_FLOOR = 119`.
/// * Solid WAC 12 — `sparq-solid` `tests/common/mod.rs` `WAC_SCENARIO_FLOOR = 12`
///   (sq-j174; floor const moved to the shared parity-corpus module in sq-t58w.6).
/// * Solid ACP 12 — `sparq-solid` `tests/common/mod.rs` `ACP_SCENARIO_FLOOR = 12`
///   (sq-j174; floor const moved to the shared parity-corpus module in sq-t58w.6).
/// * JSON-LD toRdf 413 — `sparq-conformance` `tests/jsonld_suite.rs`
///   `TORDF_FLOOR = 413` (sq-oy1f.2; opt-in `jsonld-suite` feature).
/// * JSON-LD fromRdf 51 — `sparq-conformance` `tests/jsonld_suite.rs`
///   `FROMRDF_FLOOR = 51` (sq-oy1f.2; opt-in `jsonld-suite` feature).
/// * JSON-LD compact 186 — `sparq-conformance` `tests/jsonld_suite.rs`
///   `COMPACT_FLOOR = 186` (sq-3uos5; RAISED 163→186 by sq-oy1f.16 after #978's
///   faithfulness fixes; opt-in `jsonld-suite` feature; RDF → compacted JSON-LD via
///   the native Compaction Algorithm, lossless round-trip).
/// * JSON-LD frame 61 — `sparq-conformance` `tests/jsonld_suite.rs`
///   `FRAME_FLOOR = 61` (sq-oy1f.19; opt-in `jsonld-suite` feature; RDF → framed
///   JSON-LD via the native Framing Algorithm over the SEPARATE w3c/json-ld-framing
///   suite, compared by re-parse RDF-equivalence to the normative expected output).
/// * Solid WAC differential 0 — `sparq-solid` `tests/differential_oracle.rs`
///   `DIVERGENCE_FLOOR = 0` (sq-t58w.8; a divergence-count floor, hard 0 — the WAC
///   and ACP differential rows share this one const).
/// * Solid ACP differential 0 — same `DIVERGENCE_FLOOR = 0` (sq-t58w.8).
/// * SolidLab ODRL 67 — `sparq-policy` `tests/odrl_test_suite.rs`
///   `ODRL_SUITE_FLOOR = 67` (sq-tmsd6 wired it at 59; the constraint-matching batch
///   sq-euhr3/sq-k7itg/sq-a0zef raised it to 67 of 68 cases pass, 1 in a documented
///   not-implemented bucket).
pub const SUITES: &[Suite] = &[
    Suite {
        label: "W3C SPARQL (1.0 / 1.1 / 1.2, query+update+syntax)",
        family: "W3C SPARQL",
        runner: Runner::SparqlBinary,
        ci_job: "conformance",
        ratchet_floor: 1229,
        floor_basis: "pass + documented divergence",
        note: "query/update evaluation + the four mf:*SyntaxTest* kinds",
    },
    Suite {
        label: "Inference (rdf-mt / OWL 2 RL / N3 / entailment / rdf-turtle)",
        family: "W3C RDF Semantics + OWL + N3",
        runner: Runner::InferenceBinary,
        ci_job: "inference-conformance",
        ratchet_floor: 1967,
        floor_basis: "pass + documented divergence",
        note: "reasoning suites run against sparq-reason, plus the Turtle oracle",
    },
    Suite {
        label: "W3C SHACL core",
        family: "W3C SHACL",
        runner: Runner::CrateTest { krate: "sparq-shacl", target: "w3c_core" },
        ci_job: "shacl-conformance",
        ratchet_floor: 98,
        floor_basis: "pass",
        note: "data-shapes sht:Validate core suite (w3c/data-shapes)",
    },
    Suite {
        label: "W3C SHACL-SPARQL",
        family: "W3C SHACL",
        runner: Runner::CrateTest { krate: "sparq-shacl", target: "w3c_sparql" },
        ci_job: "shacl-conformance",
        ratchet_floor: 5,
        floor_basis: "pass",
        note: "sh:sparql node + property constraint sub-suites",
    },
    Suite {
        label: "OGC GeoSPARQL topology compliance",
        family: "OGC GeoSPARQL",
        runner: Runner::CrateTest { krate: "sparq-geo", target: "ogc_compliance_ratchet" },
        ci_job: "geo-conformance",
        ratchet_floor: 119,
        floor_basis: "pass",
        note: "hand-curated sf/eh/rcc8 topology + WKT/GML equivalence assertions",
    },
    // [OPUS-4.8] sq-j174 — the Solid WAC + ACP library-level decision-parity suites
    // (harness landed under sq-3jtd.8). The runners stay crate-local in sparq-solid
    // (the dev-only conformance crate must not take sparq-solid as a dep — same
    // constraint as SHACL/geo); the central scoreboard REPORTS them with their
    // ratchet floors, kept in lock-step by `tests/scoreboard_floors.rs`. The floor
    // is the scenario COUNT each corpus must cover (a corpus that silently shrinks is
    // a coverage regression), exactly the in-source ratchet shape geo uses.
    Suite {
        label: "Solid WAC decision parity",
        family: "Solid WAC",
        runner: Runner::CrateTest { krate: "sparq-solid", target: "conformance_wac" },
        ci_job: "solid-conformance",
        ratchet_floor: 12,
        floor_basis: "scenario",
        note: "library-level allow/deny parity over minimal per-construct WAC .acl scenarios",
    },
    Suite {
        label: "Solid ACP decision parity",
        family: "Solid ACP",
        runner: Runner::CrateTest { krate: "sparq-solid", target: "conformance_acp" },
        ci_job: "solid-conformance",
        ratchet_floor: 12,
        floor_basis: "scenario",
        note: "library-level allow/deny parity over minimal per-construct ACP ACR scenarios",
    },
    // [OPUS-4.8] sq-t58w.8 — the Solid WAC + ACP DIFFERENTIAL ORACLE (harness landed
    // under sq-t58w.7, `crates/sparq-solid/tests/differential_oracle.rs`). It runs the
    // SAME shared parity corpus through THREE independent deciders (the engine, a
    // from-scratch procedural reference evaluator, and the hand `Expect` table) and
    // asserts they never disagree. The ratchet is a DIVERGENCE count whose ONLY
    // acceptable value is 0 — so unlike the scenario-COUNT floors above (which rise as
    // the corpus grows), this floor is the hard `DIVERGENCE_FLOOR = 0` the runner
    // asserts. Both rows share that ONE source const; the floor-sync guard
    // (`tests/scoreboard_floors.rs`) keeps them locked to it. The runner stays
    // crate-local in sparq-solid (this dev-only crate must not take sparq-solid as a
    // dep — same constraint as SHACL/geo/the conformance suites above). `ci_job` is the
    // Solid conformance lane; the dedicated `<WAC|ACP> differential … divergences N`
    // grep-ratchet is sq-t58w.3.
    Suite {
        label: "Solid WAC differential oracle",
        family: "Solid WAC",
        runner: Runner::CrateTest { krate: "sparq-solid", target: "differential_oracle" },
        ci_job: "solid-conformance",
        ratchet_floor: 0,
        floor_basis: "0 divergences",
        note: "engine vs an independent reference evaluator vs the hand Expect table, \
               over the WAC parity corpus (zero divergence)",
    },
    Suite {
        label: "Solid ACP differential oracle",
        family: "Solid ACP",
        runner: Runner::CrateTest { krate: "sparq-solid", target: "differential_oracle" },
        ci_job: "solid-conformance",
        ratchet_floor: 0,
        floor_basis: "0 divergences",
        note: "engine vs an independent reference evaluator vs the hand Expect table, \
               over the ACP parity corpus (zero divergence)",
    },
    // [OPUS-4.8] sq-oy1f.2 / sq-3uos5 / sq-oy1f.19 — the W3C JSON-LD 1.1 conformance
    // ratchets. The runner is crate-local here (`tests/jsonld_suite.rs`) but behind
    // the OPT-IN `jsonld-suite` feature (forwards to sparq-core/jsonld + sparq-engine/
    // serialize-rdf) so the default + `--workspace` builds neither link oxjsonld
    // nor go red — the lean-core posture. Four gated categories: toRdf (JSON-LD →
    // RDF through the real oxjsonld parse path), fromRdf (RDF → JSON-LD through the
    // native serialize-rdf writer, re-parse round-trip), compact (RDF → compacted
    // JSON-LD via the native Compaction Algorithm, lossless round-trip), and frame
    // (RDF → framed JSON-LD via the native Framing Algorithm over the SEPARATE
    // w3c/json-ld-framing suite, RDF-equivalence to the normative expected output).
    // The floors are the MEASURED pass counts at the pinned suite revisions (NOT
    // 100% — remote-context/option divergences are honest, recorded gaps); they may
    // only RISE. expand-out/flatten-out remain the documented NOT-IMPLEMENTED
    // buckets the runner reports separately (never failed). Floors kept in lock-step
    // by `tests/scoreboard_floors.rs`.
    Suite {
        label: "W3C JSON-LD 1.1 toRdf",
        family: "W3C JSON-LD",
        runner: Runner::FeatureGatedCrateTest {
            krate: "sparq-conformance",
            target: "jsonld_suite",
            feature: "jsonld-suite",
        },
        ci_job: "jsonld-conformance",
        ratchet_floor: 413,
        floor_basis: "pass",
        note: "JSON-LD → RDF through the real oxjsonld parse path (jsonld feature); \
               compact + frame are now gated; expand/flatten remain not-implemented buckets",
    },
    Suite {
        label: "W3C JSON-LD 1.1 fromRdf",
        family: "W3C JSON-LD",
        runner: Runner::FeatureGatedCrateTest {
            krate: "sparq-conformance",
            target: "jsonld_suite",
            feature: "jsonld-suite",
        },
        ci_job: "jsonld-conformance",
        ratchet_floor: 51,
        floor_basis: "pass",
        note: "RDF → JSON-LD through the native serialize-rdf writer, compared by a \
               re-parse RDF-dataset round-trip (expanded + prefix-@context forms)",
    },
    // [OPUS-4.8] sq-3uos5 — the W3C JSON-LD 1.1 `compact` ratchet (extends sq-oy1f.2,
    // epic sq-oy1f). Each `jld:CompactTest` input is parsed to RDF (the real oxjsonld
    // path), compacted against the case `@context` through the native hand-rolled
    // Compaction Algorithm (`graph_to_jsonld_compact`, serialize-rdf), then the
    // compacted document is re-parsed and required to reconstruct the SAME RDF dataset
    // (`reparse(compact(D, ctx)) ≡ D` — the lossless-compaction invariant, the same
    // oxjsonld self-reparse oracle toRdf/fromRdf use). The floor is the MEASURED pass
    // count at the pinned revision; the remaining cases are honest compaction
    // divergences (below the floor, to RISE) or documented SKIP buckets (negatives
    // sparq does not raise, JSON-LD-1.0-only, non-inline/remote @context, empty RDF).
    // Floor kept in lock-step by `tests/scoreboard_floors.rs`.
    Suite {
        label: "W3C JSON-LD 1.1 compact",
        family: "W3C JSON-LD",
        runner: Runner::FeatureGatedCrateTest {
            krate: "sparq-conformance",
            target: "jsonld_suite",
            feature: "jsonld-suite",
        },
        ci_job: "jsonld-conformance",
        // [OPUS-4.8] sq-oy1f.16 — RAISED 163 → 186 after the #978 compaction
        // faithfulness fixes landed (re-measured on current main: 186 pass).
        ratchet_floor: 186,
        floor_basis: "pass",
        note: "RDF → compacted JSON-LD through the native Compaction Algorithm \
               (serialize-rdf), compared by a re-parse RDF-dataset round-trip \
               (lossless-compaction invariant)",
    },
    // [OPUS-4.8] sq-oy1f.19 — the W3C JSON-LD 1.1 `frame` ratchet (epic sq-oy1f),
    // over the SEPARATE w3c/json-ld-framing suite (fetch-jsonld-framing-tests.sh).
    // Each `jld:FrameTest` input (an arbitrary EXPANDED JSON-LD document) is parsed
    // to RDF (the real oxjsonld path), framed against the case frame document
    // through the native hand-rolled Framing Algorithm (`graph_to_jsonld_framed`,
    // serialize-rdf), then the framed output is re-parsed and required to
    // reconstruct the SAME RDF dataset as the suite's NORMATIVE expected output
    // (`reparse(frame(D, F)) ≡ reparse(expected)` — framing is a SELECT+RESHAPE, so
    // the oracle anchors on the expected document, NOT the input). The floor is the
    // MEASURED pass count; the remaining cases are honest framer divergences (below
    // the floor, to RISE) or documented SKIP (the 3 frame-validation negatives
    // sparq's TOTAL framer does not raise). Floor kept in lock-step by
    // `tests/scoreboard_floors.rs`.
    Suite {
        label: "W3C JSON-LD 1.1 frame",
        family: "W3C JSON-LD",
        runner: Runner::FeatureGatedCrateTest {
            krate: "sparq-conformance",
            target: "jsonld_suite",
            feature: "jsonld-suite",
        },
        ci_job: "jsonld-conformance",
        ratchet_floor: 61,
        floor_basis: "pass",
        note: "RDF → framed JSON-LD through the native Framing Algorithm \
               (serialize-rdf) over the w3c/json-ld-framing suite, compared by a \
               re-parse RDF-equivalence to the normative expected output",
    },
    // [OPUS-4.8] sq-tmsd6 — the SolidLab ODRL Test Suite, wired as a crate-local
    // decision-parity ratchet in sparq-policy (mirrors the Solid WAC/ACP pattern:
    // the dev-only conformance crate must not take sparq-policy as a dep). Each of
    // the 68 self-describing Turtle cases is driven through the REAL
    // `parse_policy_str` + `evaluate` path; the oracle is the case's expected
    // compliance report (`report:activationState` ⇒ ALLOW/DENY). The floor is the
    // pass COUNT (67 at the pinned revision after the constraint-matching batch
    // sq-euhr3/sq-k7itg/sq-a0zef — `odrl:LogicalConstraint`, party/asset collection
    // membership, and the `odrl:use` action hierarchy now PASS); the 1 remaining
    // case is a documented NOT-IMPLEMENTED divergence (a duty whose discharge state
    // is unknown/`report:NonSet` — sparq is fail-closed) that does not fail the gate.
    // Floor kept in lock-step by `tests/scoreboard_floors.rs`.
    Suite {
        label: "SolidLab ODRL Test Suite",
        family: "SolidLab ODRL",
        runner: Runner::CrateTest { krate: "sparq-policy", target: "odrl_test_suite" },
        ci_job: "odrl-conformance",
        ratchet_floor: 67,
        floor_basis: "scenario",
        note: "library-level allow/deny parity over the SolidLab self-describing ODRL \
               cases through sparq-policy's real evaluate() path",
    },
];

/// Render the registry as one markdown scoreboard. This is a STATIC view of what
/// the project ratchets and where each suite is enforced — it does not re-run the
/// suites (each runner owns its own data fetch + execution), so it is hermetic and
/// fast, and serves as the single index CI surfaces alongside the per-suite
/// reports.
pub fn render_scoreboard() -> String {
    use std::fmt::Write;
    let mut md = String::new();
    let _ = writeln!(md, "# sparq conformance scoreboard\n");
    let _ = writeln!(
        md,
        "The single index of EVERY conformance suite sparq ratchets, across crates. \
         Each suite has a FLOOR that CI enforces: a pass-count (or pass+divergence) \
         floor that may only RISE, or — for the differential oracles — a hard \
         divergence-count floor of 0. The per-suite detail reports are produced by the \
         runners in the *run* column; this table is the consolidated map (sq-ncvq.16 \
         brought the SHACL + GeoSPARQL ratchets in; sq-j174 the Solid WAC + ACP \
         decision-parity ones; sq-t58w.8 the Solid WAC + ACP differential oracles — \
         all previously lived outside this scoreboard).\n"
    );
    let _ = writeln!(
        md,
        "| suite | family | floor | basis | CI job | run |"
    );
    let _ = writeln!(md, "|---|---|---:|---|---|---|");
    for s in SUITES {
        let _ = writeln!(
            md,
            "| {} | {} | {} | {} | `{}` | `{}` |",
            s.label,
            s.family,
            s.ratchet_floor,
            s.floor_basis,
            s.ci_job,
            s.runner.command(),
        );
    }
    let total: usize = SUITES.iter().map(|s| s.ratchet_floor).sum();
    let _ = writeln!(
        md,
        "| **total ({} suites)** | | **{}** | | | |\n",
        SUITES.len(),
        total
    );
    let _ = writeln!(
        md,
        "Notes:\n\n{}",
        SUITES
            .iter()
            .map(|s| format!("- **{}**: {}", s.label, s.note))
            .collect::<Vec<_>>()
            .join("\n")
    );
    md
}
