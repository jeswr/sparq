//! Scaffold-level determinism and invariant tests (bead `sq-i6du2.1`).
//!
//! These tests exercise the parts of the scaffold that are fully implemented:
//! - [`GenParams`] construction and validation.
//! - Intent-table IR: `IntentRow` round-trip (construct → clone → assert-eq).
//! - Per-model compilers: smoke test (one row per model → non-empty or Unsupported
//!   as documented).
//! - Oracle fail-closed default: a request with NO matching intent → `Decision::Deny`.
//! - Anti-vacuity negative fixture: a deliberately miscompiled WAC policy (Deny intent)
//!   → the WAC oracle returns `Decision::Deny` (fail-closed, not a spurious Allow).
//! - [`AudienceMix`] validation.
//!
//! Tests that require the generator body (B2–B5) are `#[ignore]`-tagged with a note
//! pointing to the responsible bead.

use sparq_acbench::{
    compile_acp, compile_odrl, compile_wac, oracle_acp, oracle_odrl, oracle_wac, AcModel,
    AccessMode, Audience, AudienceMix, Condition, Decision, Effect, ExpectedDecision, GenParams,
    IntentRow, Request, Scope,
};

// ── GenParams validation ─────────────────────────────────────────────────────────────

#[test]
fn gen_params_smoke_validates() {
    GenParams::smoke()
        .validate()
        .expect("smoke params must be valid");
}

#[test]
fn gen_params_container_depth_max() {
    let mut p = GenParams::smoke();
    p.container_depth = 6;
    p.validate().expect("depth 6 is at max, must be valid");
}

#[test]
fn gen_params_container_depth_over_max_is_invalid() {
    let mut p = GenParams::smoke();
    p.container_depth = 7;
    assert!(p.validate().is_err(), "depth 7 exceeds max of 6");
}

#[test]
fn gen_params_group_nesting_depth_max() {
    let mut p = GenParams::smoke();
    p.group_nesting_depth = 4;
    p.validate()
        .expect("nesting depth 4 is at max, must be valid");
}

#[test]
fn gen_params_group_nesting_depth_over_max_is_invalid() {
    let mut p = GenParams::smoke();
    p.group_nesting_depth = 5;
    assert!(p.validate().is_err(), "nesting depth 5 exceeds max of 4");
}

#[test]
fn gen_params_acl_coverage_bounds() {
    let mut p = GenParams::smoke();
    p.acl_coverage = 0.0;
    p.validate().expect("0.0 is valid");
    p.acl_coverage = 1.0;
    p.validate().expect("1.0 is valid");
    p.acl_coverage = 1.001;
    assert!(p.validate().is_err(), "1.001 exceeds max");
    p.acl_coverage = -0.001;
    assert!(p.validate().is_err(), "negative is invalid");
}

// ── AudienceMix ──────────────────────────────────────────────────────────────────────

#[test]
fn audience_mix_valid() {
    let mix = AudienceMix {
        public: 0.3,
        private: 0.5,
        shared: 0.2,
    };
    mix.validate().expect("sums to 1.0, must be valid");
}

#[test]
fn audience_mix_does_not_sum_to_one_is_invalid() {
    let mix = AudienceMix {
        public: 0.3,
        private: 0.5,
        shared: 0.3,
    };
    assert!(mix.validate().is_err(), "1.1 sum must be invalid");
}

// ── IntentRow round-trip ──────────────────────────────────────────────────────────────

#[test]
fn intent_row_round_trip() {
    let row = IntentRow {
        audience: Audience::Agent("https://example.org/alice".to_string()),
        scope: Scope::Resource,
        mode: AccessMode::read_only(),
        condition: Condition::None,
        effect: Effect::Allow,
        resource_uri: "https://example.org/resource1".to_string(),
    };
    let cloned = row.clone();
    assert_eq!(row, cloned, "IntentRow must round-trip through clone");
}

#[test]
fn intent_row_temporal_condition_round_trip() {
    let row = IntentRow {
        audience: Audience::Public,
        scope: Scope::Subtree,
        mode: AccessMode::read_only(),
        condition: Condition::Temporal {
            start: "2026-01-01T00:00:00Z".to_string(),
            end: "2027-01-01T00:00:00Z".to_string(),
        },
        effect: Effect::Allow,
        resource_uri: "https://example.org/container/".to_string(),
    };
    assert_eq!(row, row.clone());
}

// ── Compiler smoke tests ──────────────────────────────────────────────────────────────

fn simple_allow_row() -> IntentRow {
    IntentRow {
        audience: Audience::Agent("https://example.org/alice".to_string()),
        scope: Scope::Resource,
        mode: AccessMode::read_only(),
        condition: Condition::None,
        effect: Effect::Allow,
        resource_uri: "https://example.org/res".to_string(),
    }
}

#[test]
fn compile_wac_allow_agent_produces_triples() {
    let row = simple_allow_row();
    let compiled = compile_wac(&row);
    assert_eq!(compiled.model, AcModel::Wac);
    assert!(
        !compiled.nquads.is_empty(),
        "WAC compiler must produce triples for a simple allow"
    );
}

#[test]
fn compile_wac_deny_is_unsupported() {
    use sparq_acbench::Expressibility;
    let row = IntentRow {
        effect: Effect::Deny,
        ..simple_allow_row()
    };
    let compiled = compile_wac(&row);
    assert_eq!(compiled.expressibility, Expressibility::Unsupported);
    assert!(compiled.nquads.is_empty(), "WAC deny must emit no triples");
}

#[test]
fn compile_wac_condition_is_unsupported() {
    use sparq_acbench::Expressibility;
    let row = IntentRow {
        condition: Condition::Temporal {
            start: "2026-01-01T00:00:00Z".to_string(),
            end: "2027-01-01T00:00:00Z".to_string(),
        },
        ..simple_allow_row()
    };
    let compiled = compile_wac(&row);
    assert_eq!(compiled.expressibility, Expressibility::Unsupported);
    assert!(compiled.nquads.is_empty());
}

#[test]
fn compile_acp_allow_agent_produces_triples() {
    let row = simple_allow_row();
    let compiled = compile_acp(&row, &[]);
    assert_eq!(compiled.model, AcModel::Acp);
    assert!(
        !compiled.nquads.is_empty(),
        "ACP compiler must produce triples for a simple allow"
    );
}

#[test]
fn compile_acp_deny_produces_triples() {
    let row = IntentRow {
        effect: Effect::Deny,
        ..simple_allow_row()
    };
    let compiled = compile_acp(&row, &[]);
    assert_eq!(compiled.model, AcModel::Acp);
    assert!(
        !compiled.nquads.is_empty(),
        "ACP deny should produce triples (acp:deny)"
    );
}

#[test]
fn compile_acp_condition_is_unsupported() {
    use sparq_acbench::Expressibility;
    let row = IntentRow {
        condition: Condition::Purpose("https://example.org/purpose/audit".to_string()),
        ..simple_allow_row()
    };
    let compiled = compile_acp(&row, &[]);
    assert_eq!(compiled.expressibility, Expressibility::Unsupported);
}

#[test]
fn compile_odrl_allow_produces_triples() {
    let row = simple_allow_row();
    let compiled = compile_odrl(&row);
    assert_eq!(compiled.model, AcModel::Odrl);
    assert!(
        !compiled.nquads.is_empty(),
        "ODRL compiler must produce triples for a simple allow"
    );
}

#[test]
fn compile_odrl_temporal_condition_produces_triples() {
    let row = IntentRow {
        condition: Condition::Temporal {
            start: "2026-01-01T00:00:00Z".to_string(),
            end: "2027-01-01T00:00:00Z".to_string(),
        },
        ..simple_allow_row()
    };
    let compiled = compile_odrl(&row);
    assert!(
        !compiled.nquads.is_empty(),
        "ODRL temporal condition must produce constraint triples"
    );
}

#[test]
fn compile_odrl_count_condition_produces_triples() {
    let row = IntentRow {
        condition: Condition::Count(5),
        ..simple_allow_row()
    };
    let compiled = compile_odrl(&row);
    assert!(!compiled.nquads.is_empty());
}

// ── Oracle fail-closed default ────────────────────────────────────────────────────────

/// Core invariant: with an EMPTY intent table, every oracle returns Deny.
#[test]
fn oracle_fail_closed_empty_intents_wac() {
    let request = Request {
        agent: "https://example.org/alice".to_string(),
        client: None,
        resource: "https://example.org/res".to_string(),
        mode: AccessMode::read_only(),
    };
    assert_eq!(
        oracle_wac(&request, &[]),
        Decision::Deny,
        "WAC oracle must deny with empty intent table (fail-closed)"
    );
}

#[test]
fn oracle_fail_closed_empty_intents_acp() {
    let request = Request {
        agent: "https://example.org/alice".to_string(),
        client: None,
        resource: "https://example.org/res".to_string(),
        mode: AccessMode::read_only(),
    };
    assert_eq!(
        oracle_acp(&request, &[]),
        Decision::Deny,
        "ACP oracle must deny with empty intent table (fail-closed)"
    );
}

#[test]
fn oracle_fail_closed_empty_intents_odrl() {
    let request = Request {
        agent: "https://example.org/alice".to_string(),
        client: None,
        resource: "https://example.org/res".to_string(),
        mode: AccessMode::read_only(),
    };
    assert_eq!(
        oracle_odrl(&request, &[]),
        Decision::Deny,
        "ODRL oracle must deny with empty intent table (fail-closed)"
    );
}

// ── Oracle positive match ─────────────────────────────────────────────────────────────

#[test]
fn oracle_wac_allows_matching_agent_intent() {
    let row = simple_allow_row();
    let request = Request {
        agent: "https://example.org/alice".to_string(),
        client: None,
        resource: "https://example.org/res".to_string(),
        mode: AccessMode::read_only(),
    };
    assert_eq!(
        oracle_wac(&request, &[row]),
        Decision::Allow,
        "WAC oracle must allow when a matching Allow intent exists"
    );
}

#[test]
fn oracle_acp_allows_matching_agent_intent() {
    let row = simple_allow_row();
    let request = Request {
        agent: "https://example.org/alice".to_string(),
        client: None,
        resource: "https://example.org/res".to_string(),
        mode: AccessMode::read_only(),
    };
    assert_eq!(
        oracle_acp(&request, &[row]),
        Decision::Allow,
        "ACP oracle must allow when a matching Allow intent exists"
    );
}

#[test]
fn oracle_odrl_allows_matching_agent_intent() {
    let row = simple_allow_row();
    let request = Request {
        agent: "https://example.org/alice".to_string(),
        client: None,
        resource: "https://example.org/res".to_string(),
        mode: AccessMode::read_only(),
    };
    assert_eq!(
        oracle_odrl(&request, &[row]),
        Decision::Allow,
        "ODRL oracle must allow when a matching Permission intent exists"
    );
}

// ── Anti-vacuity negative fixture ────────────────────────────────────────────────────
//
// "Deliberately miscompiled" policy: an intent that LOOKS like it might grant access
// but is structurally Deny. The oracle must return Deny regardless.

/// A deny intent that targets Alice's resource — the oracle must return Deny even
/// though the request matches on agent + resource + mode.
#[test]
fn oracle_anti_vacuity_deny_intent_returns_deny_wac() {
    // WAC has no deny → the oracle skips Deny intents, so the result is Deny (fail-closed).
    let row = IntentRow {
        effect: Effect::Deny,
        ..simple_allow_row()
    };
    let request = Request {
        agent: "https://example.org/alice".to_string(),
        client: None,
        resource: "https://example.org/res".to_string(),
        mode: AccessMode::read_only(),
    };
    assert_eq!(
        oracle_wac(&request, &[row]),
        Decision::Deny,
        "WAC oracle: Deny intent must not produce Allow (anti-vacuity)"
    );
}

/// ACP: explicit Deny overrides any Allow — the oracle must return Deny.
#[test]
fn oracle_anti_vacuity_deny_overrides_allow_acp() {
    let allow_row = simple_allow_row();
    let deny_row = IntentRow {
        effect: Effect::Deny,
        ..simple_allow_row()
    };
    let request = Request {
        agent: "https://example.org/alice".to_string(),
        client: None,
        resource: "https://example.org/res".to_string(),
        mode: AccessMode::read_only(),
    };
    assert_eq!(
        oracle_acp(&request, &[allow_row, deny_row]),
        Decision::Deny,
        "ACP oracle: Deny must override Allow (deny-wins, anti-vacuity)"
    );
}

/// ODRL: Prohibition without a matching Permission → Deny (fail-closed).
#[test]
fn oracle_anti_vacuity_prohibition_without_permission_is_deny_odrl() {
    let row = IntentRow {
        effect: Effect::Deny,
        ..simple_allow_row()
    };
    let request = Request {
        agent: "https://example.org/alice".to_string(),
        client: None,
        resource: "https://example.org/res".to_string(),
        mode: AccessMode::read_only(),
    };
    assert_eq!(
        oracle_odrl(&request, &[row]),
        Decision::Deny,
        "ODRL oracle: Prohibition without Permission must deny (anti-vacuity)"
    );
}

// ── Oracle: wrong resource → Deny ────────────────────────────────────────────────────

#[test]
fn oracle_wac_deny_on_wrong_resource() {
    let row = IntentRow {
        resource_uri: "https://example.org/other".to_string(),
        ..simple_allow_row()
    };
    let request = Request {
        agent: "https://example.org/alice".to_string(),
        client: None,
        resource: "https://example.org/res".to_string(),
        mode: AccessMode::read_only(),
    };
    assert_eq!(
        oracle_wac(&request, &[row]),
        Decision::Deny,
        "WAC oracle must not match wrong resource"
    );
}

// ── Decision::default() ───────────────────────────────────────────────────────────────

#[test]
fn decision_default_is_deny() {
    assert_eq!(
        Decision::default(),
        Decision::Deny,
        "Decision::default() must be Deny (fail-closed invariant)"
    );
}

// ── ExpectedDecision struct ───────────────────────────────────────────────────────────

#[test]
fn expected_decision_round_trip() {
    let ed = ExpectedDecision {
        request: Request {
            agent: "https://example.org/alice".to_string(),
            client: None,
            resource: "https://example.org/res".to_string(),
            mode: AccessMode::read_only(),
        },
        model: AcModel::Wac,
        decision: Decision::Allow,
    };
    let cloned = ed.clone();
    assert_eq!(ed.decision, cloned.decision);
    assert_eq!(ed.model, cloned.model);
}

// ── Subtree scope ─────────────────────────────────────────────────────────────────────

#[test]
fn oracle_wac_subtree_matches_child_resources() {
    let row = IntentRow {
        scope: Scope::Subtree,
        resource_uri: "https://example.org/container/".to_string(),
        ..simple_allow_row()
    };
    let request = Request {
        agent: "https://example.org/alice".to_string(),
        client: None,
        resource: "https://example.org/container/file.txt".to_string(),
        mode: AccessMode::read_only(),
    };
    assert_eq!(
        oracle_wac(&request, &[row]),
        Decision::Allow,
        "WAC subtree scope must match child resources"
    );
}
