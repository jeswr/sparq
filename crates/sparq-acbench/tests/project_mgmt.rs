//! Tests for the U2 commercial-project-management generator (bead `sq-i6du2.3`).
//!
//! Scaffold-tier tests confirm the module types are importable; generator-body tests
//! are `#[ignore]`-tagged pending bead `sq-i6du2.3`.
//!
//! Bead `sq-i6du2.3` ONLY adds tests here and fills in `src/project_mgmt.rs::generate`.
//! It must NOT edit any other file.

use sparq_acbench::project_mgmt::ChurnStep;

/// Compile-time check: `ChurnStep` is importable.
#[test]
fn project_mgmt_types_accessible() {
    let step = ChurnStep {
        description: "scaffold compile check".to_string(),
        delta_add: vec![],
        delta_remove: vec![],
        expected_deltas: vec![],
    };
    assert!(step.delta_add.is_empty());
}

/// Placeholder: bead `sq-i6du2.3` fills this in.
#[test]
#[ignore = "sq-i6du2.3: implement U2 generator body first"]
fn generate_u2_determinism() {
    use sparq_acbench::{GenParams, project_mgmt};
    let params = GenParams::smoke();
    let ds1 = project_mgmt::generate(&params);
    let ds2 = project_mgmt::generate(&params);
    assert_eq!(ds1.data_nquads, ds2.data_nquads);
}

/// Placeholder: bead `sq-i6du2.3` verifies all-except intents carry expressibility.
#[test]
#[ignore = "sq-i6du2.3: implement U2 generator body first"]
fn generate_u2_all_except_expressibility() {
    use sparq_acbench::{Audience, GenParams, project_mgmt};
    let params = GenParams::smoke();
    let ds = project_mgmt::generate(&params);
    let all_except_intents: Vec<_> = ds
        .intents
        .iter()
        .filter(|row| matches!(&row.audience, Audience::AllExcept(_)))
        .collect();
    assert!(
        !all_except_intents.is_empty(),
        "U2 generator must produce at least one AllExcept intent"
    );
}
