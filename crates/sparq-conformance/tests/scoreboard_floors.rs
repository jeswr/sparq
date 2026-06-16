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
        // Match e.g. `const BASELINE_PASS: usize = 98;` (ignoring any trailing comment).
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
];

#[test]
fn central_floors_match_crate_local_sources() {
    for (label, src, const_name) in CRATE_LOCAL_FLOORS {
        let suite = SUITES
            .iter()
            .find(|s| s.label == *label)
            .unwrap_or_else(|| panic!("scoreboard registry missing suite {label:?}"));
        // It must be a crate-test suite (not a binary one).
        assert!(
            matches!(suite.runner, Runner::CrateTest { .. }),
            "{label} should be a CrateTest suite in the registry"
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
        if matches!(suite.runner, Runner::CrateTest { .. }) {
            assert!(
                CRATE_LOCAL_FLOORS.iter().any(|(label, _, _)| *label == suite.label),
                "registry CrateTest suite {:?} has no floor-sync guard in CRATE_LOCAL_FLOORS",
                suite.label
            );
        }
    }
}

/// The scoreboard renders and lists all five suites — smoke check on the binary's
/// output shape.
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
    // The consolidation claim: SHACL + GeoSPARQL now appear in this central report.
    assert!(md.contains("W3C SHACL core"));
    assert!(md.contains("OGC GeoSPARQL"));
}
