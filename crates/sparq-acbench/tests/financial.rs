//! Tests for the U3 financial-services generator (bead `sq-i6du2.4`).
//!
//! Scaffold-tier tests confirm module types compile; generator-body tests are
//! `#[ignore]`-tagged pending bead `sq-i6du2.4`.
//!
//! Bead `sq-i6du2.4` ONLY adds tests here and fills in `src/financial.rs::generate`.
//! It must NOT edit any other file.

use sparq_acbench::financial::FinancialDataset;

/// Compile-time import check.
#[test]
fn financial_types_accessible() {
    // FinancialDataset is a struct; we can only confirm it's importable at this tier.
    let _: fn(&sparq_acbench::GenParams) -> FinancialDataset = sparq_acbench::financial::generate;
    // If this compiles, the type and function signature are correct.
}

/// Placeholder: bead `sq-i6du2.4` fills this in.
#[test]
#[ignore = "sq-i6du2.4: implement U3 generator body first"]
fn generate_u3_determinism() {
    use sparq_acbench::{GenParams, financial};
    let params = GenParams::smoke();
    let ds1 = financial::generate(&params);
    let ds2 = financial::generate(&params);
    assert_eq!(ds1.data_nquads, ds2.data_nquads);
}

/// Placeholder: bead `sq-i6du2.4` verifies constraint-bearing intents are ODRL-only.
#[test]
#[ignore = "sq-i6du2.4: implement U3 generator body first"]
fn generate_u3_constraint_intents_odrl_only() {
    use sparq_acbench::{Condition, Expressibility, GenParams, financial, compile_wac, compile_acp};
    let params = GenParams::smoke();
    let ds = financial::generate(&params);
    let constrained: Vec<_> = ds
        .intents
        .iter()
        .filter(|row| !matches!(row.condition, Condition::None))
        .collect();
    assert!(!constrained.is_empty(), "U3 must produce constraint-bearing intents");
    for row in constrained {
        let wac = compile_wac(row);
        let acp = compile_acp(row, &[]);
        assert_eq!(
            wac.expressibility,
            Expressibility::Unsupported,
            "WAC must not express constraint intents"
        );
        assert_eq!(
            acp.expressibility,
            Expressibility::Unsupported,
            "ACP must not express constraint intents"
        );
    }
}
