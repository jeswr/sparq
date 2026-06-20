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
    let src = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
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
                    let digits: String =
                        eq.trim_start().chars().take_while(|c| c.is_ascii_digit()).collect();
                    if !digits.is_empty() {
                        return digits.parse().expect("floor parses");
                    }
                }
            }
        }
    }
    panic!("did not find `const {const_name}: usize = N;` in {}", path.display());
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
    // [OPUS-4.8] sq-oy1f.2 — the W3C JSON-LD 1.1 toRdf + fromRdf ratchets. The
    // floor consts (`pub const TORDF_FLOOR` / `FROMRDF_FLOOR`) live in this same
    // crate's `tests/jsonld_suite.rs` (behind the opt-in `jsonld-suite` feature);
    // the guard reads them textually — exactly like the SHACL/geo/Solid floors —
    // so the central scoreboard's `ratchet_floor` can never drift from what the
    // runner asserts. (`const_floor_in` already tolerates the `pub ` prefix.)
    (
        "W3C JSON-LD 1.1 toRdf",
        "crates/sparq-conformance/tests/jsonld_suite.rs",
        "TORDF_FLOOR",
    ),
    (
        "W3C JSON-LD 1.1 fromRdf",
        "crates/sparq-conformance/tests/jsonld_suite.rs",
        "FROMRDF_FLOOR",
    ),
    // [OPUS-4.8] sq-3uos5 — the W3C JSON-LD 1.1 `compact` ratchet (extends sq-oy1f.2).
    // `pub const COMPACT_FLOOR` lives in the same `tests/jsonld_suite.rs`; the guard
    // reads it textually so the central scoreboard's `ratchet_floor` can never drift
    // from what the runner asserts.
    (
        "W3C JSON-LD 1.1 compact",
        "crates/sparq-conformance/tests/jsonld_suite.rs",
        "COMPACT_FLOOR",
    ),
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
];

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
#[test]
fn all_crate_test_suites_are_guarded() {
    for suite in SUITES {
        // [OPUS-4.8] sq-oy1f.2: cover both the plain and the feature-gated
        // crate-test runners — neither may escape the floor-sync guard.
        if matches!(
            suite.runner,
            Runner::CrateTest { .. } | Runner::FeatureGatedCrateTest { .. }
        ) {
            assert!(
                CRATE_LOCAL_FLOORS.iter().any(|(label, _, _)| *label == suite.label),
                "registry CrateTest suite {:?} has no floor-sync guard in CRATE_LOCAL_FLOORS",
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
    assert!(md.contains("OGC GeoSPARQL"));
    assert!(md.contains("Solid WAC decision parity"));
    assert!(md.contains("Solid ACP decision parity"));
    assert!(md.contains("Solid WAC differential oracle"));
    assert!(md.contains("Solid ACP differential oracle"));
    // [OPUS-4.8] sq-oy1f.2 — the W3C JSON-LD 1.1 toRdf + fromRdf ratchets.
    assert!(md.contains("W3C JSON-LD 1.1 toRdf"));
    assert!(md.contains("W3C JSON-LD 1.1 fromRdf"));
    // [OPUS-4.8] sq-3uos5 — the W3C JSON-LD 1.1 compact ratchet.
    assert!(md.contains("W3C JSON-LD 1.1 compact"));
    // [OPUS-4.8] sq-tmsd6 — the SolidLab ODRL Test Suite decision-parity ratchet.
    assert!(md.contains("SolidLab ODRL Test Suite"));
}
