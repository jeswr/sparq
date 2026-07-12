//! [OPUS-4.8] sq-ncvq.16 — guard that the CENTRAL conformance scoreboard's
//! declared ratchet floors stay in lock-step with the floors the crate-local
//! SHACL + GeoSPARQL runners actually enforce.
//!
//! The scoreboard registry ([`sparq_conformance::scoreboard::SUITES`]) is the
//! single index of every conformance ratchet, but it does NOT depend on
//! `sparq-shacl` / `sparq-geo` (those crates must not become deps of this
//! dev-only harness — see `scoreboard.rs`). So the SHACL/geo floors are copied
//! into the registry as plain `usize`s. This test reads the crate-local source
//! files TEXTUALLY (no cargo build, no cross-crate dep) and asserts the copied
//! values still equal the `const` floors the runners assert — so the central
//! scoreboard can never silently fall out of sync with what CI enforces. If a
//! runner raises its floor, this test fails until the registry is updated too.

use sparq_conformance::scoreboard::{Runner, SUITES};
use std::path::PathBuf;

/// Read `const <NAME>: usize = <N>;` from a crate-local test source, hermetically.
fn const_floor_in(rel_from_workspace: &str, const_name: &str) -> usize {
    // CARGO_MANIFEST_DIR is crates/sparq-conformance; the workspace root is two up.
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel_from_workspace);
    let src =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    for line in src.lines() {
        let line = line.trim();
        // [OPUS-4.8] sq-t58w.6 — tolerate an optional `pub ` visibility prefix. The
        // Solid WAC/ACP floor consts moved into the shared `tests/common/mod.rs`
        // module where they are declared `pub const` (so both conformance test
        // binaries can read them); the SHACL/geo floors remain bare `const`. Match
        // e.g. `const BASELINE_PASS: usize = 98;` AND `pub const WAC_SCENARIO_FLOOR:
        // usize = 12;` (ignoring any trailing comment).
        let line = line.strip_prefix("pub ").unwrap_or(line);
        if let Some(rest) = line.strip_prefix("const ") {
            if let Some(after_name) = rest.strip_prefix(const_name) {
                // Guard against a longer const that merely starts with `const_name`
                // (the next char must be `:` or whitespace, i.e. the type ascription).
                if !after_name.starts_with(':') && !after_name.starts_with(char::is_whitespace) {
                    continue;
                }
                if let Some(eq) = after_name.split('=').nth(1) {
                    let digits: String = eq
                        .trim_start()
                        .chars()
                        .take_while(|c| c.is_ascii_digit())
                        .collect();
                    if !digits.is_empty() {
                        return digits.parse().expect("floor parses");
                    }
                }
            }
        }
    }
    panic!(
        "did not find `const {const_name}: usize = N;` in {}",
        path.display()
    );
}

/// (suite label, source file, const name) for each crate-local ratchet the
/// scoreboard mirrors. Keep this aligned with the `CrateTest` rows in `SUITES`.
const CRATE_LOCAL_FLOORS: &[(&str, &str, &str)] = &[
    (
        "W3C SHACL core",
        "crates/sparq-shacl/tests/w3c_core.rs",
        "BASELINE_PASS",
    ),
    (
        "W3C SHACL-SPARQL",
        "crates/sparq-shacl/tests/w3c_sparql.rs",
        "SHACL_SPARQL_FLOOR",
    ),
    (
        "OGC GeoSPARQL topology compliance",
        "crates/sparq-geo/tests/ogc_compliance_ratchet.rs",
        "OGC_RATCHET_FLOOR",
    ),
    // [OPUS-4.8] sq-wf9qg / [SONNET-4.6] sq-lk3aw.2 — the OGC GeoSPARQL
    // QUERY-REWRITE extension ratchet. The floor const
    // (`const OGC_QUERY_REWRITE_FLOOR: usize = 48;`) lives in `sparq-geo`'s
    // `tests/ogc_query_rewrite_ratchet.rs` (behind the opt-in `geosparql_rewrite`
    // feature); the guard reads it textually — exactly like the sibling OGC topology
    // floor — so the central scoreboard's `ratchet_floor` can never drift from what
    // the runner asserts.
    (
        "OGC GeoSPARQL query-rewrite extension",
        "crates/sparq-geo/tests/ogc_query_rewrite_ratchet.rs",
        "OGC_QUERY_REWRITE_FLOOR",
    ),
    // [OPUS-4.8] sq-j174 — the Solid WAC + ACP decision-parity ratchets.
    // [OPUS-4.8] sq-t58w.6 — the floor consts moved from `conformance_{wac,acp}.rs`
    // into the shared `tests/common/mod.rs` parity-corpus module (so both the
    // conformance suites AND the differential oracle read ONE floor); point the
    // floor-sync guard at the new single source. Floor value unchanged (12).
    (
        "Solid WAC decision parity",
        "crates/sparq-solid/tests/common/mod.rs",
        "WAC_SCENARIO_FLOOR",
    ),
    (
        "Solid ACP decision parity",
        "crates/sparq-solid/tests/common/mod.rs",
        "ACP_SCENARIO_FLOOR",
    ),
    // [OPUS-4.8] sq-t58w.8 — the Solid WAC + ACP DIFFERENTIAL ORACLE ratchets. Both
    // rows mirror the SAME hard divergence-count floor const (`DIVERGENCE_FLOOR = 0`)
    // declared in `tests/differential_oracle.rs` (the oracle landed under sq-t58w.7).
    // Unlike the scenario-count floors above (which rise as the corpus grows), the
    // only acceptable divergence count is 0, so this floor is the fixed 0 — and this
    // guard pins the central scoreboard's `ratchet_floor: 0` to that source const so
    // the two can never silently diverge.
    (
        "Solid WAC differential oracle",
        "crates/sparq-solid/tests/differential_oracle.rs",
        "DIVERGENCE_FLOOR",
    ),
    (
        "Solid ACP differential oracle",
        "crates/sparq-solid/tests/differential_oracle.rs",
        "DIVERGENCE_FLOOR",
    ),
    // [FABLE-5] sq-oy1f.40 — the SIX W3C JSON-LD 1.1 ratchets (toRdf, fromRdf,
    // compact, frame, expand, flatten) are NO LONGER listed here. Their floor consts
    // moved LIB-SIDE to `src/floors/<lane>.rs` (`floors::<lane>::FLOOR`) and are
    // IMPORTED directly into the `Suite` rows in `scoreboard::SUITES`
    // (`ratchet_floor: crate::floors::<lane>::FLOOR`) AND into the runner
    // (`tests/jsonld_suite/common.rs` re-exports them). Both therefore read the SAME
    // compile-time constant — they CANNOT drift, so a textual floor-sync row is
    // unnecessary (and would re-introduce the very hard-coded duplicate this bead
    // removed). These six are the `LIB_SOURCED_FLOORS` set below; the
    // `all_crate_test_suites_are_guarded` test exempts exactly them, so no OTHER
    // crate-test suite can silently escape the textual guard. See
    // `crates/sparq-conformance/src/floors/mod.rs`.
    // [OPUS-4.8] sq-tmsd6 — the SolidLab ODRL Test Suite decision-parity ratchet.
    // The floor const (`pub const ODRL_SUITE_FLOOR`) lives top-level in
    // `sparq-policy`'s `tests/odrl_test_suite.rs`; the guard reads it textually
    // (the `pub ` prefix is already tolerated) so the central scoreboard's
    // `ratchet_floor` can never drift from what the runner asserts.
    (
        "SolidLab ODRL Test Suite",
        "crates/sparq-policy/tests/odrl_test_suite.rs",
        "ODRL_SUITE_FLOOR",
    ),
    // [OPUS-4.8] sq-e5atd (epic sq-pbz04) — the W3C SPARQL 1.1 D-entailment ratchet.
    // `pub const D_ENTAIL_FLOOR` lives in this crate's `tests/d_entail_suite.rs`
    // (behind the opt-in `d-entail` feature, inside the `gated` module — the guard
    // reads it TEXTUALLY, so the `#[cfg]`/module nesting do not affect the match);
    // the guard pins the central scoreboard's `ratchet_floor` to it so the two can
    // never silently drift.
    (
        "W3C SPARQL 1.1 D-entailment",
        "crates/sparq-conformance/tests/d_entail_suite.rs",
        "D_ENTAIL_FLOOR",
    ),
    // [FABLE-5] sq-pbz04.5.5 (epic sq-pbz04.5) — the W3C RIF WG Core test-suite
    // CONFORMANCE ratchet. `pub const RIF_WG_CORE_FLOOR` lives in this crate's
    // `tests/rif_wg_core_suite.rs` (behind the opt-in `rif-wg-core` feature, inside the
    // `gated` module — the guard reads it TEXTUALLY, so the `#[cfg]`/module nesting do
    // not affect the match); the guard pins the central scoreboard's `ratchet_floor` to
    // it so the two can never silently drift. It is a STANDARDS-suite lane (family "W3C
    // RIF") over the real W3C RIF WG Core test cases — DISTINCT from the
    // sparq-EXTENSION `RIF_CORE_FLOOR` expressivity ratchet below.
    (
        "W3C RIF WG Core test suite",
        "crates/sparq-conformance/tests/rif_wg_core_suite.rs",
        "RIF_WG_CORE_FLOOR",
    ),
    // [FABLE-5] sq-pbz04.6.4 (epic sq-pbz04.6) — the sparq D VALUE-SPACE MATRIX arm's
    // EXTENSION ratchet. `pub const D_VALUE_MATRIX_FLOOR` lives in this crate's
    // `tests/d_entail_suite.rs` (behind the opt-in `d-entail` feature, inside the `gated`
    // module — the guard reads it TEXTUALLY, so the `#[cfg]`/module nesting do not affect
    // the match); the guard pins the central scoreboard's `ratchet_floor` to it so the two
    // can never silently drift. It is a sparq EXTENSION-shaped ratchet (value-space
    // assertions), tallied separately from the W3C D-entailment pass count above.
    (
        "D value-space matrix (integer/decimal/boolean/binary/temporal)",
        "crates/sparq-conformance/tests/d_entail_suite.rs",
        "D_VALUE_MATRIX_FLOOR",
    ),
    // [OPUS-4.8] sq-ddpgx (epic sq-my8wd) — the W3C SPARQL 1.1 sparql11/service
    // EVALUATION ratchet. `pub const SERVICE_EVAL_FLOOR` lives in this crate's
    // `tests/service_eval_suite.rs` (behind the opt-in `service` feature, inside the
    // `gated` module — the guard reads it TEXTUALLY, so the `#[cfg]`/module nesting
    // do not affect the match); the guard pins the central scoreboard's
    // `ratchet_floor` to it so the two can never silently drift.
    (
        "W3C SPARQL 1.1 sparql11/service evaluation",
        "crates/sparq-conformance/tests/service_eval_suite.rs",
        "SERVICE_EVAL_FLOOR",
    ),
    // [OPUS-4.8] sq-jaj38 (epic sq-my8wd) — the W3C SPARQL 1.1 PROTOCOL (HTTP layer) ratchet.
    // `pub const HTTP_PROTOCOL_FLOOR` lives in this crate's `tests/http_protocol_suite.rs`
    // (behind the opt-in `http-protocol` feature, inside the `gated` module — the guard reads
    // it TEXTUALLY, so the `#[cfg]`/module nesting do not affect the match); the guard pins the
    // central scoreboard's `ratchet_floor` to it so the two can never silently drift.
    (
        "W3C SPARQL 1.1 Protocol (HTTP)",
        "crates/sparq-conformance/tests/http_protocol_suite.rs",
        "HTTP_PROTOCOL_FLOOR",
    ),
    // [OPUS-4.8] sq-1uuxz (epic sq-my8wd) — the SPARQL 1.1 SERVICE-DESCRIPTION +
    // GRAPH-STORE PROTOCOL ratchet. `pub const SD_GSP_FLOOR` lives in this crate's
    // `tests/sd_gsp_suite.rs` (behind the opt-in `federation-descriptors` feature, inside
    // the `gated` module — the guard reads it TEXTUALLY, so the `#[cfg]`/module nesting do
    // not affect the match); the guard pins the central scoreboard's `ratchet_floor` to it
    // so the two can never silently drift.
    (
        "SPARQL 1.1 Service Description + Graph Store Protocol",
        "crates/sparq-conformance/tests/sd_gsp_suite.rs",
        "SD_GSP_FLOOR",
    ),
    // [OPUS-4.8] sq-ripcg (epic sq-lk3aw) — the sparq-text DIFFERENTIAL BM25 ORACLE,
    // a sparq EXTENSION ratchet (NOT standards conformance — no normative
    // full-text-over-RDF / BM25 suite exists). `const TEXT_ORACLE_FLOOR` lives
    // top-level in `sparq-text`'s `tests/bm25_oracle.rs`; the guard reads it
    // TEXTUALLY (same hermetic mechanism — the dev-only conformance crate takes NO
    // dependency edge on sparq-text) so the central scoreboard's `ratchet_floor`
    // can never silently drift from what the runner asserts.
    (
        "text-search differential oracle",
        "crates/sparq-text/tests/bm25_oracle.rs",
        "TEXT_ORACLE_FLOOR",
    ),
    // [OPUS-4.8] sq-mcb3q (epic sq-2n1q3) — the sparq-rsp RSP EXPRESSIVITY /
    // SRBench CORRECTNESS oracle, a sparq EXTENSION ratchet (NOT standards
    // conformance — no normative RDF-Stream-Processing / RSP conformance suite
    // exists; RSP-QL is a W3C-community spec and SRBench a benchmark). `const
    // RSP_EXPRESSIVITY_FLOOR` lives top-level in `sparq-rsp`'s
    // `tests/srbench_oracle.rs`; the guard reads it TEXTUALLY (same hermetic
    // mechanism — the dev-only conformance crate takes NO dependency edge on
    // sparq-rsp) so the central scoreboard's `ratchet_floor` can never silently
    // drift from what the runner asserts.
    (
        "RSP expressivity / SRBench correctness",
        "crates/sparq-rsp/tests/srbench_oracle.rs",
        "RSP_EXPRESSIVITY_FLOOR",
    ),
    // [OPUS-4.8] sq-rh4gu (epic sq-pbz04) — the RIF-Core EXPRESSIVITY ratchet. The
    // floor const (`pub const RIF_CORE_FLOOR`) lives in THIS crate's
    // `tests/rif_core_suite.rs` (behind the opt-in `rif-core` feature, inside the
    // `gated` module — the guard reads it TEXTUALLY, so the `#[cfg]`/module nesting
    // do not affect the match); the guard pins the central scoreboard's
    // `ratchet_floor` to it so the two can never silently drift. It is a sparq
    // EXTENSION-shaped ratchet over the RIF-Core (monotone Horn) subset, NOT the
    // normative W3C SPARQL-RIF conformance suite.
    (
        "RIF-Core expressivity (monotone Horn subset)",
        "crates/sparq-conformance/tests/rif_core_suite.rs",
        "RIF_CORE_FLOOR",
    ),
    // [OPUS-4.8] sq-qo1a9 (epic sq-pbz04, the LAST conformance bead) — the GRADUATED
    // OWL 2 QL (DL-Lite_R) CERTAIN-ANSWER oracle ratchet. The floor const
    // (`pub const QL_DLLITE_FLOOR`) lives in THIS crate's `tests/ql_dllite_suite.rs`
    // (behind the opt-in `ql-experimental` feature, inside the `gated` module — the
    // guard reads it TEXTUALLY, so the `#[cfg]`/module nesting do not affect the
    // match); the guard pins the central scoreboard's `ratchet_floor` to it so the
    // two can never silently drift. It is a sparq EXTENSION-shaped ratchet over the
    // hand-derived DL-Lite_R certain-answer oracle, NOT a full-OWL-2-QL-conformance
    // claim.
    (
        "OWL 2 QL (DL-Lite_R) certain-answer oracle",
        "crates/sparq-conformance/tests/ql_dllite_suite.rs",
        "QL_DLLITE_FLOOR",
    ),
    // [FABLE-5] sq-pbz04.3.4 (epic sq-pbz04.3) — the OWL 2 QL ENTAILMENT-REGIME
    // GRADUATED-SUBSET ratchet. The floor const (`pub const QL_ENTAILMENT_FLOOR`)
    // lives in THIS crate's `tests/ql_entailment_floor.rs` (behind the opt-in
    // `ql-experimental` feature, inside the `gated` module — the guard reads it
    // TEXTUALLY, so the `#[cfg]`/module nesting do not affect the match); the guard
    // pins the central scoreboard's `ratchet_floor` to it so the two can never
    // silently drift. The floor equals the length of the PINNED NAMED-CASE list
    // (asserted in-runner), a sparq EXTENSION-shaped ratchet over the six-condition
    // sound `pr:QL` subset, NOT an OWL 2 QL / entailment-regime conformance claim.
    (
        "OWL 2 QL entailment-regime graduated subset",
        "crates/sparq-conformance/tests/ql_entailment_floor.rs",
        "QL_ENTAILMENT_FLOOR",
    ),
    // [SONNET-4.6] sq-pbz04.2.4 (epic sq-pbz04) — the OWL 2 EL classification ratchet.
    // The floor const (`pub const EL_SUITE_FLOOR`) lives in THIS crate's
    // `tests/el_suite.rs` (behind the opt-in `el-suite` feature, inside the `gated`
    // module — the guard reads it TEXTUALLY, so the `#[cfg]`/module nesting do not
    // affect the match); the guard pins the central scoreboard's `ratchet_floor` to it
    // so the two can never silently drift. It is a sparq EXTENSION-shaped ratchet over
    // the EL fragment sparq-reason-el's classifier implements, NOT a full-OWL-2-EL-
    // conformance claim.
    (
        "OWL 2 EL classification (sparq-reason-el)",
        "crates/sparq-conformance/tests/el_suite.rs",
        "EL_SUITE_FLOOR",
    ),
    // [FABLE-5] sq-pbz04.4.5 (epic sq-pbz04.4) — the OWL 2 DIRECT-SEMANTICS arm's two
    // ratchets. The floor consts (`pub const DL_PROFILE_FLOOR` / `DL_DIRECT_FLOOR`)
    // live in THIS crate's `tests/dl_suite.rs` (behind the opt-in `dl-direct` feature,
    // inside the `gated` module — the guard reads them TEXTUALLY, so the
    // `#[cfg]`/module nesting do not affect the match); the guard pins the central
    // scoreboard's `ratchet_floor`s to them so they can never silently drift. Both are
    // sparq EXTENSION measurements over the SCOPED FRAGMENT the layered
    // `sparq-reason-dl` checker implements — NOT full OWL 2 DL — and are EXACT-pinned
    // in-runner (`==`, not `>=`: the tri-state invariant "an abstention is NEVER a
    // pass" needs the inflation direction caught too).
    (
        "OWL 2 DL profile identification (Direct arm)",
        "crates/sparq-conformance/tests/dl_suite.rs",
        "DL_PROFILE_FLOOR",
    ),
    (
        "OWL 2 Direct-Semantics consistency + entailment (scoped fragment)",
        "crates/sparq-conformance/tests/dl_suite.rs",
        "DL_DIRECT_FLOOR",
    ),
    // [FABLE-5] the UFO-SN3 finite-world expressibility ratchet. The floor const
    // (`pub const UFO_SN3_FLOOR`) lives top-level in THIS crate's
    // `tests/ufo_sn3_suite.rs` (UNGATED — the lane calls plain `reason_n3` and runs
    // in ordinary `cargo test --workspace`); the guard reads it TEXTUALLY so the
    // central scoreboard's `ratchet_floor` can never silently drift from what the
    // runner asserts. It is a sparq EXTENSION-shaped ratchet over the finite-world
    // UFO-SN3 reference profile, NOT a UFO/gUFO/OntoUML standards-conformance claim
    // (no normative UFO conformance suite exists).
    (
        "UFO-SN3 finite-world expressibility",
        "crates/sparq-conformance/tests/ufo_sn3_suite.rs",
        "UFO_SN3_FLOOR",
    ),
];

/// [FABLE-5] sq-oy1f.40 — the suite labels whose ratchet floor is sourced at
/// COMPILE TIME from the LIB-SIDE `sparq_conformance::floors::<lane>::FLOOR` const
/// (imported into BOTH `scoreboard::SUITES` and the `jsonld_suite` runner). These
/// six do NOT need a textual floor-sync row in `CRATE_LOCAL_FLOORS`: the registry
/// and the runner read the SAME `const`, so they cannot drift. The
/// `all_crate_test_suites_are_guarded` test exempts exactly this set (and asserts
/// nothing else escapes the textual guard).
const LIB_SOURCED_FLOORS: &[&str] = &[
    "W3C JSON-LD 1.1 toRdf",
    "W3C JSON-LD 1.1 fromRdf",
    "W3C JSON-LD 1.1 compact",
    "W3C JSON-LD 1.1 frame",
    "W3C JSON-LD 1.1 expand",
    "W3C JSON-LD 1.1 flatten",
];

/// [FABLE-5] sq-oy1f.40 — the pinned floor VALUES the six lib-sourced JSON-LD
/// registry rows must carry, as `(label, expected_floor)`. Because the registry
/// imports the lib-side `const` directly, this compile-time check catches a silent
/// LOWERING of any of the six floors (a ratchet may only RISE): if someone edits
/// `src/floors/<lane>::FLOOR` down, the registry `ratchet_floor` drops with it and
/// this assertion fires. RAISING a floor deliberately updates the value here in the
/// same commit (mirroring how a textual-guard floor is bumped). This is the
/// lib-sourced analogue of the `const_floor_in` textual re-read, giving the six
/// JSON-LD lanes the same "floor cannot silently move" protection without a
/// hard-coded duplicate of the number in two source files.
const LIB_SOURCED_EXPECTED: &[(&str, usize)] = &[
    ("W3C JSON-LD 1.1 toRdf", 413),
    // [FABLE-5] sq-oy1f.28 — raised 51 → 52: the lane flipped from the engine-writer
    // round-trip oracle to the native document-level `sparq_jsonld::from_rdf` (§8.1)
    // oracle (normative expected-doc comparison + scoped round-trip + REAL negative
    // error-code assertions). Bumped in the SAME commit as src/floors/from_rdf.rs::FLOOR
    // and the ci.yml job name (rise-only; side-by-side in src/floors/from_rdf.rs).
    ("W3C JSON-LD 1.1 fromRdf", 52),
    // [FABLE-5] sq-oy1f.27 — oracle-correction re-pin 186 → 228: the compact lane
    // moved from the RDF-writer self-reparse round-trip to the NATIVE document-level
    // Compaction Algorithm compared against the W3C EXPECTED document (see
    // src/floors/compact.rs for the side-by-side). Bumped in the SAME commit as the
    // lib const (rise-only).
    ("W3C JSON-LD 1.1 compact", 228),
    // [FABLE-5] sq-oy1f.29 — raised 61 → 92: the frame lane moved from the RDF-first
    // framer (`graph_to_jsonld_framed`) to the NATIVE Framing pipeline compared against
    // the W3C EXPECTED document under the stronger normative oracle (see
    // src/floors/frame.rs for the side-by-side). Bumped in the SAME commit as the lib
    // const src/floors/frame.rs::FLOOR and the ci.yml grep gate (rise-only).
    ("W3C JSON-LD 1.1 frame", 92),
    // [SONNET-4.6] sq-oy1f.45 — raised 259 → 276 (expand() correctness: FsLoader
    // wiring + @id-null retention + IRI-colon scheme check + @nest scoped ctx
    // propagation + @reverse @index + 1.0-mode round-trip guard). Bumped in the
    // SAME commit as src/floors/expand.rs::FLOOR (rise-only).
    ("W3C JSON-LD 1.1 expand", 276),
    // [FABLE-5] sq-oy1f.26 — oracle-change re-pin (RDF-writer 50 → native flatten() 53).
    // The native lane composes over expand() and inherits the sq-oy1f.37 expand raises,
    // so merging main flips its 7 inherited fails to passes and it now MEASURES 53 pass /
    // 0 fail on the merged tree — a net RISE above the old writer oracle's 50 (union of
    // the native oracle AND main's expand fixes — see src/floors/flatten.rs). Rise-only.
    ("W3C JSON-LD 1.1 flatten", 53),
];

/// [FABLE-5] sq-oy1f.40 — the registry's six lib-sourced JSON-LD floors carry the
/// pinned values (a ratchet may only RISE; a silent LOWERING of a
/// `src/floors/<lane>::FLOOR` const drops the registry value and trips this).
#[test]
fn lib_sourced_jsonld_floors_are_pinned() {
    for (label, expected) in LIB_SOURCED_EXPECTED {
        let suite = SUITES
            .iter()
            .find(|s| s.label == *label)
            .unwrap_or_else(|| panic!("scoreboard registry missing lib-sourced suite {label:?}"));
        assert_eq!(
            suite.ratchet_floor, *expected,
            "lib-sourced floor for {label} is {} but the pinned ratchet is {} — a floor may \
             only RISE; if this is a deliberate raise, bump LIB_SOURCED_EXPECTED in the same \
             commit that edits src/floors/<lane>::FLOOR (and the ci.yml grep gate)",
            suite.ratchet_floor, expected
        );
    }
}

#[test]
fn central_floors_match_crate_local_sources() {
    for (label, src, const_name) in CRATE_LOCAL_FLOORS {
        let suite = SUITES
            .iter()
            .find(|s| s.label == *label)
            .unwrap_or_else(|| panic!("scoreboard registry missing suite {label:?}"));
        // It must be a crate-test suite (not a binary one). [OPUS-4.8] sq-oy1f.2:
        // the feature-gated JSON-LD lane is a `FeatureGatedCrateTest` — still a
        // crate-local `cargo test` whose floor lives in source, so it is covered
        // by the same textual floor-sync guard.
        assert!(
            matches!(
                suite.runner,
                Runner::CrateTest { .. } | Runner::FeatureGatedCrateTest { .. }
            ),
            "{label} should be a (feature-gated) CrateTest suite in the registry"
        );
        let source_floor = const_floor_in(src, const_name);
        assert_eq!(
            suite.ratchet_floor, source_floor,
            "scoreboard floor for {label} ({}) is out of sync with {const_name} in {src} ({}) \
             — raise the registry value to match, or the central scoreboard under/over-reports \
             the enforced ratchet",
            suite.ratchet_floor, source_floor
        );
    }
}

/// Every crate-test suite in the registry is covered by a sync check above (so a
/// new crate-local ratchet added to `SUITES` cannot escape the guard).
///
/// [FABLE-5] sq-oy1f.40 — a crate-test suite is guarded either by a TEXTUAL
/// floor-sync row in `CRATE_LOCAL_FLOORS` (the runner's `const` is read out of
/// source and `assert_eq!`'d against the registry copy) OR by being LIB-SOURCED —
/// its floor lives in `src/floors/<lane>::FLOOR` and is imported into BOTH the
/// registry row and the runner, so the two read the same compile-time const and
/// cannot drift (the `LIB_SOURCED_FLOORS` set, covered by
/// `lib_sourced_jsonld_floors_are_pinned`). Every crate-test suite must be in
/// exactly one of those two buckets — nothing escapes the guard.
#[test]
fn all_crate_test_suites_are_guarded() {
    for suite in SUITES {
        // [OPUS-4.8] sq-oy1f.2: cover both the plain and the feature-gated
        // crate-test runners — neither may escape the floor-sync guard.
        if matches!(
            suite.runner,
            Runner::CrateTest { .. } | Runner::FeatureGatedCrateTest { .. }
        ) {
            let textually_guarded = CRATE_LOCAL_FLOORS
                .iter()
                .any(|(label, _, _)| *label == suite.label);
            let lib_sourced = LIB_SOURCED_FLOORS.contains(&suite.label);
            assert!(
                textually_guarded || lib_sourced,
                "registry CrateTest suite {:?} has neither a textual floor-sync guard in \
                 CRATE_LOCAL_FLOORS nor a lib-sourced floor in LIB_SOURCED_FLOORS",
                suite.label
            );
            // A suite must not be in BOTH buckets — a textual row for a lib-sourced
            // floor re-introduces the hard-coded duplicate sq-oy1f.40 removed.
            assert!(
                !(textually_guarded && lib_sourced),
                "registry CrateTest suite {:?} is BOTH textually guarded and lib-sourced — \
                 remove the CRATE_LOCAL_FLOORS row (the lib-side const is the single source)",
                suite.label
            );
        }
    }
}

/// The scoreboard renders and lists every registered suite — smoke check on the
/// binary's output shape.
#[test]
fn scoreboard_renders_all_suites() {
    let md = sparq_conformance::scoreboard::render_scoreboard();
    assert!(md.contains("conformance scoreboard"));
    for suite in SUITES {
        assert!(
            md.contains(suite.label),
            "rendered scoreboard omits suite {:?}",
            suite.label
        );
    }
    // The consolidation claim: SHACL + GeoSPARQL (sq-ncvq.16), Solid WAC/ACP
    // decision parity (sq-j174), and the Solid WAC/ACP differential oracles
    // (sq-t58w.8) now all appear in this central report.
    assert!(md.contains("W3C SHACL core"));
    assert!(md.contains("OGC GeoSPARQL topology compliance"));
    // [OPUS-4.8] sq-wf9qg — the OGC GeoSPARQL query-rewrite extension ratchet.
    assert!(md.contains("OGC GeoSPARQL query-rewrite extension"));
    assert!(md.contains("Solid WAC decision parity"));
    assert!(md.contains("Solid ACP decision parity"));
    assert!(md.contains("Solid WAC differential oracle"));
    assert!(md.contains("Solid ACP differential oracle"));
    // [OPUS-4.8] sq-oy1f.2 — the W3C JSON-LD 1.1 toRdf + fromRdf ratchets.
    assert!(md.contains("W3C JSON-LD 1.1 toRdf"));
    assert!(md.contains("W3C JSON-LD 1.1 fromRdf"));
    // [OPUS-4.8] sq-3uos5 — the W3C JSON-LD 1.1 compact ratchet.
    assert!(md.contains("W3C JSON-LD 1.1 compact"));
    // [OPUS-4.8] sq-oy1f — the W3C JSON-LD 1.1 expand + flatten ratchets.
    assert!(md.contains("W3C JSON-LD 1.1 expand"));
    assert!(md.contains("W3C JSON-LD 1.1 flatten"));
    // [OPUS-4.8] sq-tmsd6 — the SolidLab ODRL Test Suite decision-parity ratchet.
    assert!(md.contains("SolidLab ODRL Test Suite"));
    // [OPUS-4.8] sq-e5atd — the W3C SPARQL 1.1 D-entailment ratchet.
    assert!(md.contains("W3C SPARQL 1.1 D-entailment"));
    // [FABLE-5] sq-pbz04.5.5 — the W3C RIF WG Core test-suite conformance ratchet (a
    // STANDARDS lane, family "W3C RIF" — NOT the sparq-extension RIF-Core expressivity row).
    assert!(md.contains("W3C RIF WG Core test suite"));
    // [OPUS-4.8] sq-ddpgx — the W3C SPARQL 1.1 sparql11/service evaluation ratchet.
    assert!(md.contains("W3C SPARQL 1.1 sparql11/service evaluation"));
    // [OPUS-4.8] sq-jaj38 — the W3C SPARQL 1.1 Protocol (HTTP layer) ratchet.
    assert!(md.contains("W3C SPARQL 1.1 Protocol (HTTP)"));
    // [OPUS-4.8] sq-1uuxz — the SPARQL 1.1 Service Description + Graph Store Protocol ratchet.
    assert!(md.contains("SPARQL 1.1 Service Description + Graph Store Protocol"));
    // [OPUS-4.8] sq-ripcg — the sparq-text BM25 differential oracle, HONESTLY
    // rendered as a sparq EXTENSION (not a conformance claim): the row, its
    // `sparq extension` family, and the explicit "NOT conformance" total-line
    // disclaimer must all appear.
    assert!(md.contains("text-search differential oracle"));
    assert!(md.contains("sparq extension"));
    assert!(md.contains("NOT a standards-conformance claim") || md.contains("NOT conformance"));
    // [OPUS-4.8] sq-mcb3q — the sparq-rsp RSP expressivity / SRBench correctness
    // oracle, HONESTLY rendered as a sparq EXTENSION (not a conformance claim):
    // the row appears, and (now there are TWO extension rows) the total line
    // pluralises to "rows".
    assert!(md.contains("RSP expressivity / SRBench correctness"));
    // [OPUS-4.8] sq-rh4gu — the RIF-Core expressivity ratchet, HONESTLY rendered as
    // a sparq EXTENSION over the RIF-Core (monotone Horn) subset.
    assert!(md.contains("RIF-Core expressivity (monotone Horn subset)"));
    // [OPUS-4.8] sq-qo1a9 — the GRADUATED OWL 2 QL (DL-Lite_R) certain-answer oracle,
    // HONESTLY rendered as a sparq EXTENSION (NOT folded into the conformance total).
    assert!(md.contains("OWL 2 QL (DL-Lite_R) certain-answer oracle"));
    // [FABLE-5] sq-pbz04.3.4 — the OWL 2 QL entailment-regime graduated-subset
    // ratchet (the pinned named-case floor), HONESTLY rendered as a sparq EXTENSION
    // (NOT folded into the conformance total).
    assert!(md.contains("OWL 2 QL entailment-regime graduated subset"));
    // [SONNET-4.6] sq-pbz04.2.4 — the OWL 2 EL classification ratchet, HONESTLY rendered
    // as a sparq EXTENSION (the total line pluralises to "rows", and it is NOT folded
    // into the conformance total).
    assert!(md.contains("OWL 2 EL classification (sparq-reason-el)"));
    // [FABLE-5] sq-pbz04.4.5 — the two OWL 2 Direct-Semantics arm rows, HONESTLY
    // rendered as sparq EXTENSIONS over the scoped fragment (NOT full OWL 2 DL, NOT
    // folded into the conformance total).
    assert!(md.contains("OWL 2 DL profile identification (Direct arm)"));
    assert!(md.contains("OWL 2 Direct-Semantics consistency + entailment (scoped fragment)"));
    assert!(md.contains("NOT full OWL 2 DL"));
    // [FABLE-5] sq-pbz04.6.4 — the sparq D value-space matrix, HONESTLY rendered as a
    // sparq EXTENSION (tallied separately from the W3C D-entailment pass count).
    assert!(md.contains("D value-space matrix (integer/decimal/boolean/binary/temporal)"));
    // [FABLE-5] — the UFO-SN3 finite-world expressibility ratchet, HONESTLY rendered
    // as a sparq EXTENSION (no normative UFO/gUFO conformance suite exists; NOT
    // folded into the conformance total).
    assert!(md.contains("UFO-SN3 finite-world expressibility"));
    assert!(
        md.contains("sparq-extension (10 rows, NOT conformance)"),
        "ten extension rows should be tallied separately and pluralised"
    );
}
