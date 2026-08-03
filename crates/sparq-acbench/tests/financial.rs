//! Tests for the U3 financial-services generator (bead `sq-i6du2.4`).
//!
//! The scaffold-tier test (`financial_types_accessible`) confirms the stub compiles.
//! All generator-body tests are implemented by bead `sq-i6du2.4` (previously `#[ignore]`).
//!
//! Invariants verified here:
//! 1. **Determinism**: same `GenParams` → byte-identical output on every call.
//! 2. **Constraint intents are ODRL-only**: WAC and ACP compilers return
//!    `Expressibility::Unsupported` for every intent with `condition ≠ None`.
//! 3. **Non-empty constrained intents at ≥ Temporal**: the smoke params use
//!    `ConstraintComplexity::None`; a separate params set exercises each level.
//! 4. **Fail-closed oracle**: every expected decision is consistent with the
//!    by-construction oracle (no spurious Allows, no missed Denies).
//! 5. **Non-vacuous expected decisions**: the corpus contains at least one Allow
//!    and at least one Deny (the fail-closed proof point).
//! 6. **Data graph non-empty**: the N-Quads output is non-empty and deterministic.
//! 7. **W2 queries non-empty**: at least one query fixture is produced.
//! 8. **All four query classes present** at SF > 1 (requires ≥ 2 clients).
//! 9. **Unknown agent is always denied**: the fail-closed sentinel in expected_decisions.
//! 10. **Policy fan-in produces extra intents**: `policies_per_resource > 1` increases
//!     the intent count.

use sparq_acbench::financial::FinancialDataset;
use sparq_acbench::{
    AcModel, Condition, ConstraintComplexity, Decision, Expressibility, GenParams,
};

/// Compile-time import check.
#[test]
fn financial_types_accessible() {
    let _: fn(&GenParams) -> FinancialDataset = sparq_acbench::financial::generate;
}

// ── (1) Determinism ──────────────────────────────────────────────────────────────────

#[test]
fn generate_u3_determinism() {
    use sparq_acbench::financial;
    let params = GenParams::smoke();
    let ds1 = financial::generate(&params);
    let ds2 = financial::generate(&params);
    assert_eq!(
        ds1.data_nquads, ds2.data_nquads,
        "U3 data_nquads must be byte-identical for the same seed"
    );
    // Compare intent-table sizes (IntentRow doesn't derive PartialEq with Box, so use len).
    assert_eq!(
        ds1.intents.len(),
        ds2.intents.len(),
        "intent table length must be deterministic"
    );
    assert_eq!(
        ds1.expected_decisions.len(),
        ds2.expected_decisions.len(),
        "expected_decisions count must be deterministic"
    );
    assert_eq!(
        ds1.queries.len(),
        ds2.queries.len(),
        "query fixture count must be deterministic"
    );
    assert_eq!(
        ds1.wac_policy.len(),
        ds2.wac_policy.len(),
        "WAC policy count must be deterministic"
    );
    assert_eq!(
        ds1.odrl_policy.len(),
        ds2.odrl_policy.len(),
        "ODRL policy count must be deterministic"
    );
}

// ── (2) Constraint intents are ODRL-only ─────────────────────────────────────────────

#[test]
fn generate_u3_constraint_intents_odrl_only() {
    use sparq_acbench::{compile_acp, compile_wac, financial};
    // Use Temporal so the smoke params produce constrained intents.
    let mut params = GenParams::smoke();
    params.constraint_complexity = ConstraintComplexity::Temporal;
    let ds = financial::generate(&params);

    let constrained: Vec<_> = ds
        .intents
        .iter()
        .filter(|row| !matches!(row.condition, Condition::None))
        .collect();
    assert!(
        !constrained.is_empty(),
        "U3 must produce constraint-bearing intents when constraint_complexity = Temporal"
    );
    for row in constrained {
        let wac = compile_wac(row);
        let acp = compile_acp(row, &[]);
        assert_eq!(
            wac.expressibility,
            Expressibility::Unsupported,
            "WAC must return Unsupported for constraint intent: {:?}",
            row.condition
        );
        assert_eq!(
            acp.expressibility,
            Expressibility::Unsupported,
            "ACP must return Unsupported for constraint intent: {:?}",
            row.condition
        );
    }
}

// ── (3) Non-empty constrained intents at each level ─────────────────────────────────

#[test]
fn generate_u3_purpose_or_count_constraints_present() {
    use sparq_acbench::financial;
    let mut params = GenParams::smoke();
    params.constraint_complexity = ConstraintComplexity::PurposeOrCount;
    let ds = financial::generate(&params);
    let constrained_count = ds
        .intents
        .iter()
        .filter(|r| !matches!(r.condition, Condition::None))
        .count();
    assert!(
        constrained_count > 0,
        "PurposeOrCount level must produce at least one constraint intent"
    );
}

#[test]
fn generate_u3_compound_constraints_present() {
    use sparq_acbench::financial;
    let mut params = GenParams::smoke();
    params.constraint_complexity = ConstraintComplexity::Compound;
    let ds = financial::generate(&params);
    let compound_count = ds
        .intents
        .iter()
        .filter(|r| matches!(r.condition, Condition::And(_, _)))
        .count();
    assert!(
        compound_count > 0,
        "Compound level must produce at least one And(...) condition intent"
    );
}

// ── (4) Fail-closed oracle consistency ───────────────────────────────────────────────

/// Every ExpectedDecision must agree with the procedural oracle.
/// This is the load-bearing correctness test: any inconsistency means the
/// by-construction oracle was mis-populated.
#[test]
fn generate_u3_oracle_consistency() {
    use sparq_acbench::{financial, oracle_acp, oracle_odrl, oracle_wac};
    let params = GenParams::smoke();
    let ds = financial::generate(&params);
    for ed in &ds.expected_decisions {
        let oracle_result = match ed.model {
            AcModel::Wac => oracle_wac(&ed.request, &ds.intents),
            AcModel::Acp => oracle_acp(&ed.request, &ds.intents),
            AcModel::Odrl => oracle_odrl(&ed.request, &ds.intents),
        };
        assert_eq!(
            oracle_result, ed.decision,
            "Oracle mismatch for {:?} model, request agent={}, resource={}",
            ed.model, ed.request.agent, ed.request.resource,
        );
    }
}

/// Same oracle consistency test at Temporal constraint level.
#[test]
fn generate_u3_oracle_consistency_temporal() {
    use sparq_acbench::{financial, oracle_acp, oracle_odrl, oracle_wac};
    let mut params = GenParams::smoke();
    params.constraint_complexity = ConstraintComplexity::Temporal;
    let ds = financial::generate(&params);
    for ed in &ds.expected_decisions {
        let oracle_result = match ed.model {
            AcModel::Wac => oracle_wac(&ed.request, &ds.intents),
            AcModel::Acp => oracle_acp(&ed.request, &ds.intents),
            AcModel::Odrl => oracle_odrl(&ed.request, &ds.intents),
        };
        assert_eq!(
            oracle_result, ed.decision,
            "Oracle mismatch (Temporal) for {:?} model, agent={}, resource={}",
            ed.model, ed.request.agent, ed.request.resource,
        );
    }
}

// ── (5) Non-vacuous: both Allow and Deny present ─────────────────────────────────────

#[test]
fn generate_u3_expected_decisions_non_vacuous() {
    use sparq_acbench::financial;
    let params = GenParams::smoke();
    let ds = financial::generate(&params);
    let has_allow = ds
        .expected_decisions
        .iter()
        .any(|ed| ed.decision == Decision::Allow);
    let has_deny = ds
        .expected_decisions
        .iter()
        .any(|ed| ed.decision == Decision::Deny);
    assert!(
        has_allow,
        "Expected decisions must contain at least one Allow"
    );
    assert!(
        has_deny,
        "Expected decisions must contain at least one Deny (fail-closed proof)"
    );
}

// ── (6) Data graph non-empty ─────────────────────────────────────────────────────────

#[test]
fn generate_u3_data_nquads_non_empty() {
    use sparq_acbench::financial;
    let params = GenParams::smoke();
    let ds = financial::generate(&params);
    assert!(
        !ds.data_nquads.is_empty(),
        "U3 data_nquads must be non-empty"
    );
    // Every line should look like a valid N-Quads statement (starts with '<').
    for line in &ds.data_nquads {
        assert!(
            line.starts_with('<'),
            "N-Quads line should start with '<': {line}"
        );
    }
}

// ── (7) W2 queries non-empty ─────────────────────────────────────────────────────────

#[test]
fn generate_u3_queries_non_empty() {
    use sparq_acbench::financial;
    let params = GenParams::smoke();
    let ds = financial::generate(&params);
    assert!(
        !ds.queries.is_empty(),
        "U3 must produce at least one W2 query fixture"
    );
    for q in &ds.queries {
        assert!(
            !q.sparql.is_empty(),
            "Each query fixture must have a non-empty SPARQL string"
        );
    }
}

// ── (8) All four query classes at SF=2 ───────────────────────────────────────────────

#[test]
fn generate_u3_all_four_query_classes_at_sf2() {
    use sparq_acbench::{financial, QueryClass};
    let mut params = GenParams::smoke();
    params.sf = 2;
    let ds = financial::generate(&params);

    let classes: Vec<&QueryClass> = ds.queries.iter().map(|q| &q.class).collect();
    assert!(
        classes.iter().any(|c| **c == QueryClass::Point),
        "U3 SF=2 must include Q-point query"
    );
    assert!(
        classes.iter().any(|c| **c == QueryClass::Scan),
        "U3 SF=2 must include Q-scan query"
    );
    assert!(
        classes.iter().any(|c| **c == QueryClass::Join),
        "U3 SF=2 must include Q-join query (requires ≥2 clients)"
    );
    assert!(
        classes.iter().any(|c| **c == QueryClass::Aggregate),
        "U3 SF=2 must include Q-agg query"
    );
}

// ── (9) Unknown agent is always denied ───────────────────────────────────────────────

/// The corpus always contains a sentinel unknown-agent request that must be Denied
/// by all three models (fail-closed invariant proof point).
#[test]
fn generate_u3_unknown_agent_always_denied() {
    use sparq_acbench::financial;
    let params = GenParams::smoke();
    let ds = financial::generate(&params);

    let unknown_agent = "https://bench.sparq.dev/financial/agent/unknown/9999";
    let unknown_decisions: Vec<_> = ds
        .expected_decisions
        .iter()
        .filter(|ed| ed.request.agent == unknown_agent)
        .collect();

    assert!(
        !unknown_decisions.is_empty(),
        "Expected decisions must contain the unknown-agent sentinel"
    );
    for ed in unknown_decisions {
        assert_eq!(
            ed.decision,
            Decision::Deny,
            "Unknown agent must always be Denied by {:?} model (fail-closed)",
            ed.model
        );
    }
}

// ── (10) Policy fan-in increases intent count ────────────────────────────────────────

#[test]
fn generate_u3_policy_fan_in_increases_intents() {
    use sparq_acbench::financial;
    let mut params_base = GenParams::smoke();
    params_base.policies_per_resource = 1;
    let ds_base = financial::generate(&params_base);

    let mut params_fan = GenParams::smoke();
    params_fan.policies_per_resource = 3;
    let ds_fan = financial::generate(&params_fan);

    assert!(
        ds_fan.intents.len() > ds_base.intents.len(),
        "policies_per_resource=3 must produce more intents than policies_per_resource=1 \
         (got base={}, fan={})",
        ds_base.intents.len(),
        ds_fan.intents.len()
    );
}

// ── Scale-factor scaling ─────────────────────────────────────────────────────────────

#[test]
fn generate_u3_sf_scaling() {
    use sparq_acbench::financial;
    let mut params1 = GenParams::smoke();
    params1.sf = 1;
    let ds1 = financial::generate(&params1);

    let mut params2 = GenParams::smoke();
    params2.sf = 2;
    let ds2 = financial::generate(&params2);

    assert!(
        ds2.data_nquads.len() > ds1.data_nquads.len(),
        "SF=2 must produce more N-Quads than SF=1 (got sf1={}, sf2={})",
        ds1.data_nquads.len(),
        ds2.data_nquads.len()
    );
    assert!(
        ds2.intents.len() > ds1.intents.len(),
        "SF=2 must produce more intents than SF=1 (got sf1={}, sf2={})",
        ds1.intents.len(),
        ds2.intents.len()
    );
}

// ── ODRL policy non-empty ────────────────────────────────────────────────────────────

#[test]
fn generate_u3_odrl_policy_non_empty() {
    use sparq_acbench::financial;
    let params = GenParams::smoke();
    let ds = financial::generate(&params);
    assert!(
        !ds.odrl_policy.is_empty(),
        "ODRL policy must be non-empty for U3"
    );
    // Every ODRL compiled policy must have non-empty nquads.
    for cp in &ds.odrl_policy {
        assert!(
            !cp.nquads.is_empty(),
            "Each ODRL compiled policy must produce nquads"
        );
    }
}

// ── WAC/ACP unsupported for constraint intents (direct policy output check) ──────────

#[test]
fn generate_u3_wac_policy_empty_for_constraint_intents() {
    use sparq_acbench::financial;
    let mut params = GenParams::smoke();
    params.constraint_complexity = ConstraintComplexity::Temporal;
    let ds = financial::generate(&params);

    // Zip intents and wac_policy: constrained intents must have empty nquads.
    for (intent, wac_cp) in ds.intents.iter().zip(ds.wac_policy.iter()) {
        if !matches!(intent.condition, Condition::None) {
            assert!(
                wac_cp.nquads.is_empty(),
                "WAC policy must produce no triples for constraint intent \
                 (resource={}, condition={:?})",
                intent.resource_uri,
                intent.condition
            );
            assert_eq!(
                wac_cp.expressibility,
                Expressibility::Unsupported,
                "WAC expressibility must be Unsupported for constraint intent"
            );
        }
    }
}

#[test]
fn generate_u3_acp_policy_empty_for_constraint_intents() {
    use sparq_acbench::financial;
    let mut params = GenParams::smoke();
    params.constraint_complexity = ConstraintComplexity::Temporal;
    let ds = financial::generate(&params);

    for (intent, acp_cp) in ds.intents.iter().zip(ds.acp_policy.iter()) {
        if !matches!(intent.condition, Condition::None) {
            assert!(
                acp_cp.nquads.is_empty(),
                "ACP policy must produce no triples for constraint intent \
                 (resource={}, condition={:?})",
                intent.resource_uri,
                intent.condition
            );
            assert_eq!(
                acp_cp.expressibility,
                Expressibility::Unsupported,
                "ACP expressibility must be Unsupported for constraint intent"
            );
        }
    }
}
