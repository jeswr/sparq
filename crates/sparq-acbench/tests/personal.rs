//! Tests for the U1 personal-data-storage generator (bead `sq-i6du2.2`).
//!
//! All tests that exercise the generator body are `#[ignore]`-tagged with a note
//! pointing to bead `sq-i6du2.2`. The scaffold-tier test below confirms the stub
//! compiles and that the module is importable from an integration test.
//!
//! Bead `sq-i6du2.2` ONLY adds tests here and fills in `src/personal.rs::generate`.
//! It must NOT edit any other file.

use sparq_acbench::QueryClass;

/// Compile-time check: the `QueryClass` type is importable from the crate root.
#[test]
fn query_class_accessible() {
    // QueryClass derives PartialEq; verify it compiles.
    assert_eq!(QueryClass::Point, QueryClass::Point);
    assert_ne!(QueryClass::Point, QueryClass::Scan);
}

/// Compile-time check: `PersonalDataset` is importable from `personal`.
#[test]
fn personal_dataset_type_accessible() {
    let _: fn(&sparq_acbench::GenParams) -> sparq_acbench::personal::PersonalDataset =
        sparq_acbench::personal::generate;
}

/// Placeholder: bead `sq-i6du2.2` fills this in with real generator tests.
#[test]
#[ignore = "sq-i6du2.2: implement U1 generator body first"]
fn generate_u1_determinism() {
    use sparq_acbench::{GenParams, personal};
    let params = GenParams::smoke();
    let ds1 = personal::generate(&params);
    let ds2 = personal::generate(&params);
    assert_eq!(
        ds1.data_nquads, ds2.data_nquads,
        "U1 generator must be deterministic: same seed → same N-Quads"
    );
}

/// Placeholder: bead `sq-i6du2.2` verifies oracle fail-closed for generated corpus.
#[test]
#[ignore = "sq-i6du2.2: implement U1 generator body first"]
fn generate_u1_oracle_fail_closed() {
    use sparq_acbench::{GenParams, personal, oracle_wac};
    let params = GenParams::smoke();
    let ds = personal::generate(&params);
    // Every expected Deny in the WAC lane must be produced by the oracle with no
    // matching Allow intent.
    for ed in &ds.expected_decisions {
        let oracle_result = oracle_wac(&ed.request, &ds.intents);
        assert_eq!(
            oracle_result, ed.decision,
            "U1 oracle mismatch for request {:?}",
            ed.request
        );
    }
}
