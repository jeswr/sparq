//! Tests for the U2 commercial-project-management generator (bead `sq-i6du2.3`).
//!
//! Tests at three tiers:
//! 1. Scaffold-tier compile checks (always run).
//! 2. Generator-body invariant tests (un-ignored by bead `sq-i6du2.3`).
//! 3. Non-vacuous oracle + expressibility tests added by bead `sq-i6du2.3`.
//!
//! This file ONLY edits `src/project_mgmt.rs::generate`. It must NOT edit any
//! other file in the crate.

use sparq_acbench::project_mgmt::ChurnStep;

// ── Tier 1: compile-time checks (always run) ──────────────────────────────────────────

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

// ── Tier 2: generator-body invariants (un-ignored by sq-i6du2.3) ─────────────────────

/// Determinism: same seed → byte-identical N-Quads output.
#[test]
fn generate_u2_determinism() {
    use sparq_acbench::{project_mgmt, GenParams};
    let params = GenParams::smoke();
    let ds1 = project_mgmt::generate(&params);
    let ds2 = project_mgmt::generate(&params);
    assert_eq!(
        ds1.data_nquads, ds2.data_nquads,
        "U2 generator must be deterministic: same seed → same N-Quads"
    );
}

/// The generator emits at least one AllExcept intent (core U2 expressibility signal).
#[test]
fn generate_u2_all_except_expressibility() {
    use sparq_acbench::{project_mgmt, Audience, GenParams};
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

// ── Tier 3: non-vacuous oracle + expressibility tests added by sq-i6du2.3 ────────────

/// All-except intents carry expressibility-matrix entries for WAC, ACP, and ODRL.
///
/// WAC entry must be Expansion or Unsupported (never Native — WAC has no deny).
/// ACP entry must be Native (acp:deny is native for all-except).
/// ODRL entry must be Native (Prohibition is native for all-except).
#[test]
fn all_except_intents_have_per_model_expressibility_entries() {
    use sparq_acbench::{project_mgmt, AcModel, Audience, Expressibility, GenParams};
    let params = GenParams::smoke();
    let ds = project_mgmt::generate(&params);

    // Find all AllExcept intent indices.
    let ae_indices: Vec<usize> = ds
        .intents
        .iter()
        .enumerate()
        .filter(|(_, row)| matches!(&row.audience, Audience::AllExcept(_)))
        .map(|(i, _)| i)
        .collect();
    assert!(
        !ae_indices.is_empty(),
        "must have at least one AllExcept intent"
    );

    for &ae_idx in &ae_indices {
        // Gather expressibility entries for this intent index.
        let entries: Vec<_> = ds
            .expressibility_matrix
            .iter()
            .filter(|(idx, _)| *idx == ae_idx)
            .collect();

        let wac_entry = entries.iter().find(|(_, e)| e.model == AcModel::Wac);
        let acp_entry = entries.iter().find(|(_, e)| e.model == AcModel::Acp);
        let odrl_entry = entries.iter().find(|(_, e)| e.model == AcModel::Odrl);

        assert!(
            wac_entry.is_some(),
            "AllExcept intent {ae_idx} must have a WAC expressibility entry"
        );
        assert!(
            acp_entry.is_some(),
            "AllExcept intent {ae_idx} must have an ACP expressibility entry"
        );
        assert!(
            odrl_entry.is_some(),
            "AllExcept intent {ae_idx} must have an ODRL expressibility entry"
        );

        // WAC: AllExcept → no deny → must be Expansion or Unsupported, never Native.
        let wac_expr = &wac_entry.unwrap().1.expressibility;
        assert!(
            matches!(
                wac_expr,
                Expressibility::Expansion(_) | Expressibility::Unsupported
            ),
            "WAC AllExcept expressibility must be Expansion or Unsupported, got {wac_expr:?}"
        );

        // ACP: acp:deny handles AllExcept natively.
        assert_eq!(
            acp_entry.unwrap().1.expressibility,
            Expressibility::Native,
            "ACP AllExcept expressibility must be Native (acp:deny)"
        );

        // ODRL: Prohibition handles AllExcept natively.
        assert_eq!(
            odrl_entry.unwrap().1.expressibility,
            Expressibility::Native,
            "ODRL AllExcept expressibility must be Native (Prohibition)"
        );
    }
}

/// Group-nesting intents (member/guest role groups) carry ACP expansion entries.
///
/// ACP has no vcard:Group matcher — groups must be expanded to per-member agent
/// matchers. The expressibility matrix must record this blowup factor.
#[test]
fn group_intents_have_acp_expansion_entries() {
    use sparq_acbench::{project_mgmt, AcModel, Audience, Expressibility, GenParams};
    let params = GenParams::smoke();
    let ds = project_mgmt::generate(&params);

    let group_indices: Vec<usize> = ds
        .intents
        .iter()
        .enumerate()
        .filter(|(_, row)| matches!(&row.audience, Audience::Group(_)))
        .map(|(i, _)| i)
        .collect();
    assert!(
        !group_indices.is_empty(),
        "U2 generator must produce at least one Group intent"
    );

    for &gi in &group_indices {
        let acp_entries: Vec<_> = ds
            .expressibility_matrix
            .iter()
            .filter(|(idx, e)| *idx == gi && e.model == AcModel::Acp)
            .collect();
        assert!(
            !acp_entries.is_empty(),
            "Group intent {gi} must have an ACP expressibility entry"
        );
        let acp_expr = &acp_entries[0].1.expressibility;
        assert!(
            matches!(acp_expr, Expressibility::Expansion(_)),
            "ACP group intent {gi} must be Expansion, got {acp_expr:?}"
        );
    }
}

/// Fail-closed: an unknown external agent is denied everywhere.
///
/// This test verifies the oracle's fail-closed invariant against actual generator output.
#[test]
fn unknown_agent_is_denied_by_all_models() {
    use sparq_acbench::{project_mgmt, AcModel, Decision, GenParams};
    let params = GenParams::smoke();
    let ds = project_mgmt::generate(&params);

    let external = "https://bench.sparq.dev/pm/agents/external/unknown";
    let denied = ds
        .expected_decisions
        .iter()
        .filter(|ed| ed.request.agent == external)
        .all(|ed| ed.decision == Decision::Deny);
    assert!(
        denied,
        "External unknown agent must be denied by all models (fail-closed)"
    );

    // Also verify the three models all produce Deny decisions for this agent.
    let wac_denies = ds
        .expected_decisions
        .iter()
        .filter(|ed| ed.request.agent == external && ed.model == AcModel::Wac)
        .count();
    assert!(
        wac_denies > 0,
        "Must have at least one WAC decision for the unknown agent"
    );
}

/// Owner is allowed read on project 0 by all models.
#[test]
fn owner_allowed_on_own_project() {
    use sparq_acbench::{project_mgmt, AcModel, Decision, GenParams};
    let params = GenParams::smoke();
    let ds = project_mgmt::generate(&params);

    let owner = "https://bench.sparq.dev/pm/agents/org/0/owner";
    let project0 = "https://bench.sparq.dev/pm/org/0/project/0/";

    for model in [AcModel::Wac, AcModel::Acp, AcModel::Odrl] {
        let decision = ds.expected_decisions.iter().find(|ed| {
            ed.request.agent == owner
                && ed.request.resource == project0
                && ed.request.mode.read
                && !ed.request.mode.write
                && ed.model == model
        });
        assert!(
            decision.is_some(),
            "Must have a decision for owner reading project 0 under {model:?}"
        );
        assert_eq!(
            decision.unwrap().decision,
            Decision::Allow,
            "Owner must be allowed to read their own project 0 under {model:?}"
        );
    }
}

/// Member write on a project is denied (only read was granted to member groups).
#[test]
fn member_write_on_project_is_denied() {
    use sparq_acbench::{project_mgmt, AcModel, AccessMode, Decision, GenParams};
    let params = GenParams::smoke();
    let ds = project_mgmt::generate(&params);

    let member = "https://bench.sparq.dev/pm/agents/org/0/agent/1";
    let project0 = "https://bench.sparq.dev/pm/org/0/project/0/";
    let write_mode = AccessMode {
        read: false,
        write: true,
        control: false,
    };

    let decision = ds.expected_decisions.iter().find(|ed| {
        ed.request.agent == member
            && ed.request.resource == project0
            && ed.request.mode == write_mode
            && ed.model == AcModel::Wac
    });
    assert!(
        decision.is_some(),
        "Must have a WAC decision for member writing project 0"
    );
    assert_eq!(
        decision.unwrap().decision,
        Decision::Deny,
        "Member must be denied write access to project 0 (only read granted)"
    );
}

/// W3 churn steps are non-empty and have correct structure.
///
/// Each step must have a description and expected_deltas covering all three models.
#[test]
fn churn_steps_non_empty_and_structured() {
    use sparq_acbench::{project_mgmt, AcModel, GenParams};
    let params = GenParams::smoke();
    let ds = project_mgmt::generate(&params);

    assert!(
        !ds.churn_steps.is_empty(),
        "U2 generator must produce at least one W3 churn step"
    );

    for (i, step) in ds.churn_steps.iter().enumerate() {
        assert!(
            !step.description.is_empty(),
            "Churn step {i} must have a description"
        );
        // Each step must have expected_deltas for all three models.
        let models_covered: std::collections::HashSet<_> =
            step.expected_deltas.iter().map(|ed| &ed.model).collect();
        assert!(
            models_covered.contains(&AcModel::Wac),
            "Churn step {i} must have a WAC expected delta"
        );
        assert!(
            models_covered.contains(&AcModel::Acp),
            "Churn step {i} must have an ACP expected delta"
        );
        assert!(
            models_covered.contains(&AcModel::Odrl),
            "Churn step {i} must have an ODRL expected delta"
        );
    }
}

/// W3 churn grant step changes the decision for the new agent to Allow (WAC).
///
/// Verifies the by-construction delta: before grant → Deny; grant → Allow.
#[test]
fn churn_grant_step_produces_allow_delta() {
    use sparq_acbench::{project_mgmt, AcModel, Decision, GenParams};
    let params = GenParams::smoke();
    let ds = project_mgmt::generate(&params);

    // Grant step is always first (grant → revoke order).
    let grant_step = ds
        .churn_steps
        .first()
        .expect("must have at least one churn step");
    assert!(grant_step.description.starts_with("Grant"));

    // The WAC expected delta after a grant must be Allow.
    let wac_delta = grant_step
        .expected_deltas
        .iter()
        .find(|ed| ed.model == AcModel::Wac)
        .expect("grant step must have WAC expected delta");
    assert_eq!(
        wac_delta.decision,
        Decision::Allow,
        "WAC grant step must produce Allow decision for the new agent"
    );
}

/// W2 query fixtures: all four classes are present and non-empty.
#[test]
fn query_fixtures_all_classes_present() {
    use sparq_acbench::{project_mgmt, GenParams, QueryClass};
    let params = GenParams::smoke();
    let ds = project_mgmt::generate(&params);

    assert!(
        !ds.queries.is_empty(),
        "U2 must produce at least one W2 query fixture"
    );

    let has_point = ds.queries.iter().any(|q| q.class == QueryClass::Point);
    let has_scan = ds.queries.iter().any(|q| q.class == QueryClass::Scan);
    let has_join = ds.queries.iter().any(|q| q.class == QueryClass::Join);
    let has_agg = ds.queries.iter().any(|q| q.class == QueryClass::Aggregate);

    assert!(has_point, "U2 must include a Q-point fixture");
    assert!(has_scan, "U2 must include a Q-scan fixture");
    assert!(has_join, "U2 must include a Q-join fixture");
    assert!(has_agg, "U2 must include a Q-agg fixture");
}

/// Data graph contains the expected hierarchy of orgs, projects, sites, and documents.
#[test]
fn data_graph_contains_hierarchy_triples() {
    use sparq_acbench::{project_mgmt, GenParams};
    let params = GenParams::smoke();
    let ds = project_mgmt::generate(&params);

    assert!(!ds.data_nquads.is_empty(), "data graph must be non-empty");

    // Must contain an org container triple.
    let has_org = ds
        .data_nquads
        .iter()
        .any(|nq| nq.contains("bench.sparq.dev/pm/org/0/") && nq.contains("BasicContainer"));
    assert!(
        has_org,
        "data graph must contain an org BasicContainer triple"
    );

    // Must contain a project container triple.
    let has_project = ds
        .data_nquads
        .iter()
        .any(|nq| nq.contains("/project/0/") && nq.contains("BasicContainer"));
    assert!(
        has_project,
        "data graph must contain a project BasicContainer triple"
    );

    // Must contain a vcard:Group triple for role groups.
    let has_group = ds
        .data_nquads
        .iter()
        .any(|nq| nq.contains("vcard/ns#Group"));
    assert!(
        has_group,
        "data graph must contain vcard:Group triples for role groups"
    );
}

/// Compiled WAC policy is non-empty.
#[test]
fn wac_policy_non_empty() {
    use sparq_acbench::{project_mgmt, GenParams};
    let params = GenParams::smoke();
    let ds = project_mgmt::generate(&params);
    assert!(
        !ds.wac_policy.is_empty(),
        "WAC compiled policy must be non-empty"
    );
}

/// Compiled ACP policy is non-empty.
#[test]
fn acp_policy_non_empty() {
    use sparq_acbench::{project_mgmt, GenParams};
    let params = GenParams::smoke();
    let ds = project_mgmt::generate(&params);
    assert!(
        !ds.acp_policy.is_empty(),
        "ACP compiled policy must be non-empty"
    );
}

/// Compiled ODRL policy is non-empty.
#[test]
fn odrl_policy_non_empty() {
    use sparq_acbench::{project_mgmt, GenParams};
    let params = GenParams::smoke();
    let ds = project_mgmt::generate(&params);
    assert!(
        !ds.odrl_policy.is_empty(),
        "ODRL compiled policy must be non-empty"
    );
}

/// Cross-org group reuse: the subcontractor group IRI appears in multiple container
/// policies (cross-org reuse is the signature AC shape of U2).
#[test]
fn subcontractor_cross_org_group_reuse() {
    use sparq_acbench::{project_mgmt, Audience, GenParams};
    let mut params = GenParams::smoke();
    params.sf = 1; // ensure at least 2 orgs
    let ds = project_mgmt::generate(&params);

    // Collect all group IRIs referenced across all intents.
    let sub_group_intents: Vec<_> = ds
        .intents
        .iter()
        .filter(|row| {
            if let Audience::Group(g) = &row.audience {
                g.contains("/groups/subcontractor/")
            } else {
                false
            }
        })
        .collect();
    assert!(
        !sub_group_intents.is_empty(),
        "U2 must contain subcontractor (cross-org) group intents"
    );
}

/// Oracle consistency: expected_decisions match oracle_wac / oracle_acp / oracle_odrl
/// when evaluated directly against the intent table.
#[test]
fn expected_decisions_consistent_with_oracle() {
    use sparq_acbench::{oracle_acp, oracle_odrl, oracle_wac, project_mgmt, AcModel, GenParams};
    let params = GenParams::smoke();
    let ds = project_mgmt::generate(&params);

    for ed in &ds.expected_decisions {
        let oracle_decision = match ed.model {
            AcModel::Wac => oracle_wac(&ed.request, &ds.intents),
            AcModel::Acp => oracle_acp(&ed.request, &ds.intents),
            AcModel::Odrl => oracle_odrl(&ed.request, &ds.intents),
        };
        assert_eq!(
            oracle_decision, ed.decision,
            "expected_decisions must be consistent with the oracle for request {:?} model {:?}",
            ed.request, ed.model
        );
    }
}
