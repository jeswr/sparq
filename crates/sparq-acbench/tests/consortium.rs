//! Tests for the U4 research-data-consortium generator (bead `sq-i6du2.5`).
//!
//! Scaffold-tier tests confirm module types compile; generator-body tests are
//! `#[ignore]`-tagged pending bead `sq-i6du2.5`.
//!
//! Bead `sq-i6du2.5` ONLY adds tests here and fills in `src/consortium.rs::generate`.
//! It must NOT edit any other file.

use sparq_acbench::consortium::ConsortiumDataset;

/// Compile-time import check.
#[test]
fn consortium_types_accessible() {
    let _: fn(&sparq_acbench::GenParams) -> ConsortiumDataset = sparq_acbench::consortium::generate;
}

/// Placeholder: bead `sq-i6du2.5` fills this in.
#[test]
#[ignore = "sq-i6du2.5: implement U4 generator body first"]
fn generate_u4_determinism() {
    use sparq_acbench::{GenParams, consortium};
    let params = GenParams::smoke();
    let ds1 = consortium::generate(&params);
    let ds2 = consortium::generate(&params);
    assert_eq!(ds1.data_nquads, ds2.data_nquads);
}

/// Placeholder: bead `sq-i6du2.5` verifies embargo flips produce exact decision deltas.
#[test]
#[ignore = "sq-i6du2.5: implement U4 generator body first"]
fn generate_u4_embargo_flips_have_deltas() {
    use sparq_acbench::{GenParams, consortium};
    let params = GenParams::smoke();
    let ds = consortium::generate(&params);
    assert!(
        !ds.embargo_flips.is_empty(),
        "U4 generator must produce at least one embargo-flip churn step"
    );
    for flip in &ds.embargo_flips {
        assert!(
            !flip.expected_deltas.is_empty(),
            "Each embargo flip must have at least one expected decision delta"
        );
    }
}
