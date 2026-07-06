//! [FABLE-5] sq-zgbso.2 — the CI-locked Rust-vs-N3 ODRL decision differential
//! (epic sq-zgbso, #1582; design record `research/odrl-n3-compiled-rules.md`).
//!
//! INVARIANT (fail-closed + decision-equivalence): on every corpus scenario the N3
//! rule strata (`rules/odrl-core-{a,b,c,d}.n3` via `materialize_policy_n3`) must
//! produce EXACTLY the auth-view triples the Rust path produces
//! (`odrl_bridge::materialize_policy` over `sparq_policy::evaluate`) — the same
//! allow grants, the same `auth:deny*` triples (deny-overrides), the same refusals,
//! and NOTHING on the fail-closed denies. Any divergence is a red test.
//!
//! The corpus = the sq-zgbso.1 spike's 7 scenarios (with their independently-stated
//! expected sets, so a both-wrong agreement cannot pass) + generated permutations:
//! the action×action hierarchy matrix, structural (target/assignee) permutations,
//! `odrl:dateTime` windows across all six operators and mixed-offset request times
//! (the finding-(a) normalization), dateTime-constrained PROHIBITIONS (the
//! finding-(c) stratification case), recipient/assignee/purpose constraints,
//! `and`/`or`/`xone` combinator status patterns, duties, conflict strategies, and
//! deny-overrides compositions. Out-of-subset constructs are separately asserted to
//! be LOUD errors that materialize nothing (never a silent divergence window).
//!
//! Runs under `--features odrl-bridge` (NOT `--all-features`; the pre-existing
//! sq-s5tkx count-enforcement failure is out of scope).
#![cfg(feature = "odrl-bridge")]

use sparq_core::Graph;
use sparq_policy::{parse_policy_str, Policy, Value};
use sparq_solid::odrl_bridge::{materialize_policy, materialize_policy_n3, N3Request};
use sparq_solid::AUTH_NS;

const ODRL: &str = "http://www.w3.org/ns/odrl/2/";
const T1: &str = "urn:t/1";
const T2: &str = "urn:t/2";
const ALICE: &str = "urn:alice";
const BOB: &str = "urn:bob";
const CAROL: &str = "urn:carol";
const PURPOSE: &str = "http://www.w3.org/ns/odrl/2/purpose";
const SPATIAL: &str = "http://www.w3.org/ns/odrl/2/spatial";
const RECIPIENT: &str = "http://www.w3.org/ns/odrl/2/recipient";

type AuthSet = Vec<(String, String, String)>;

fn act(local: &str) -> String {
    format!("{ODRL}{local}")
}

/// One ODRL rule body (the `[ … ]` block) from parts.
fn rule_body(action: &str, target: Option<&str>, assignee: Option<&str>, extra: &str) -> String {
    let mut b = format!("odrl:action odrl:{action}");
    if let Some(t) = target {
        b.push_str(&format!(" ; odrl:target <{t}>"));
    }
    if let Some(a) = assignee {
        b.push_str(&format!(" ; odrl:assignee <{a}>"));
    }
    if !extra.is_empty() {
        b.push_str(" ; ");
        b.push_str(extra);
    }
    b
}

/// A one-rule `odrl:Set` policy ("permission" or "prohibition").
fn policy_ttl(kind: &str, body: &str) -> String {
    format!(
        "@prefix odrl: <http://www.w3.org/ns/odrl/2/> .\n\
         @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
         <urn:pol/1> a odrl:Set ; odrl:{kind} [ {body} ] ."
    )
}

/// A permission + prohibition `odrl:Set` policy.
fn policy_ttl2(perm_body: &str, proh_body: &str) -> String {
    format!(
        "@prefix odrl: <http://www.w3.org/ns/odrl/2/> .\n\
         @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
         <urn:pol/1> a odrl:Set ; odrl:permission [ {perm_body} ] ; \
         odrl:prohibition [ {proh_body} ] ."
    )
}

/// An `odrl:dateTime <op>` constraint fragment.
fn dt_constraint(op: &str, bound: &str) -> String {
    format!(
        "odrl:constraint [ odrl:leftOperand odrl:dateTime ; odrl:operator odrl:{op} ; \
         odrl:rightOperand \"{bound}\"^^xsd:dateTime ]"
    )
}

/// A generic IRI-bound constraint fragment.
fn iri_constraint(left_local: &str, op: &str, right_iri: &str) -> String {
    format!(
        "odrl:constraint [ odrl:leftOperand odrl:{left_local} ; odrl:operator odrl:{op} ; \
         odrl:rightOperand <{right_iri}> ]"
    )
}

/// A string-bound constraint fragment.
fn str_constraint(left_local: &str, op: &str, right: &str) -> String {
    format!(
        "odrl:constraint [ odrl:leftOperand odrl:{left_local} ; odrl:operator odrl:{op} ; \
         odrl:rightOperand \"{right}\" ]"
    )
}

/// A compound `odrl:LogicalConstraint` fragment over atomic operand fragments.
fn logical(combinator: &str, operands: &[String]) -> String {
    let ops: Vec<String> = operands
        .iter()
        .map(|o| {
            // strip the leading "odrl:constraint " to get the bare [ … ] node
            o.strip_prefix("odrl:constraint ").expect("operand fragment").to_owned()
        })
        .collect();
    format!("odrl:constraint [ odrl:{combinator} {} ]", ops.join(" , "))
}

/// Collect the (grant, deny) auth triples of a bridge outcome, sorted.
fn outcome_set(granted: Option<(String, String, String)>, denied: Option<(String, String, String)>) -> AuthSet {
    let mut s: AuthSet = granted.into_iter().chain(denied).collect();
    s.sort();
    s
}

/// Run BOTH paths on the same (policy, request) and assert decision equality.
/// Returns the agreed set so spike-style cases can ALSO pin an independently-stated
/// expectation. Panics with the scenario label + policy text on any divergence.
fn assert_parity(label: &str, ttl: &str, req: &N3Request) -> AuthSet {
    let policy: Policy =
        parse_policy_str(ttl, "turtle").unwrap_or_else(|e| panic!("{label}: policy parse: {e}\n{ttl}"));
    let mut g_rust = Graph::new();
    let rust = materialize_policy(&mut g_rust, &policy, &req.to_request());
    let mut g_n3 = Graph::new();
    let n3 = materialize_policy_n3(&mut g_n3, &policy, req)
        .unwrap_or_else(|e| panic!("{label}: N3 path must stay in-subset on the corpus: {e}\n{ttl}"));

    assert_eq!(rust.refused, n3.refused, "{label}: refusal divergence\n{ttl}");
    assert_eq!(rust.granted, n3.granted, "{label}: granted divergence\n{ttl}");
    assert_eq!(rust.prohibited, n3.prohibited, "{label}: prohibited divergence\n{ttl}");
    let rust_set = outcome_set(rust.grant_triple, rust.deny_triple);
    let n3_set = outcome_set(n3.grant_triple, n3.deny_triple);
    assert_eq!(rust_set, n3_set, "{label}: auth-triple divergence\n{ttl}");
    rust_set
}

/// Pin an independently-stated expected set on top of parity (the spike discipline:
/// a both-wrong agreement cannot pass).
fn assert_case(label: &str, ttl: &str, req: &N3Request, want: &[(&str, &str, &str)]) {
    let got = assert_parity(label, ttl, req);
    let mut expected: AuthSet = want
        .iter()
        .map(|(a, m, t)| ((*a).to_owned(), format!("{AUTH_NS}{m}"), (*t).to_owned()))
        .collect();
    expected.sort();
    assert_eq!(got, expected, "{label}: agreed set != independently-stated expectation\n{ttl}");
}

// ── the spike's 7 scenarios, expectations restated independently ───────────────────────

#[test]
fn spike_scenarios_with_pinned_expectations() {
    let pol_a = policy_ttl(
        "permission",
        &rule_body("read", Some(T1), Some(ALICE), &dt_constraint("lteq", "2026-12-31T00:00:00Z")),
    );
    let pol_b = policy_ttl("permission", &rule_body("read", Some(T1), Some(ALICE), ""));
    let pol_c = policy_ttl2(
        &rule_body("read", Some(T1), Some(ALICE), ""),
        &rule_body("read", Some(T1), Some(ALICE), ""),
    );
    let base = || N3Request::new(act("read")).on(T1).by(ALICE);

    assert_case("A1 within window", &pol_a, &base().at("2026-07-05T00:00:00Z"), &[(ALICE, "read", T1)]);
    assert_case("A2 after window", &pol_a, &base().at("2027-01-01T00:00:00Z"), &[]);
    assert_case("A3 no time evidence", &pol_a, &base(), &[]);
    assert_case(
        "A4 wrong assignee",
        &pol_a,
        &N3Request::new(act("read")).on(T1).by(BOB).at("2026-07-05T00:00:00Z"),
        &[],
    );
    assert_case(
        "A5 action mismatch",
        &pol_a,
        &N3Request::new(act("write")).on(T1).by(ALICE).at("2026-07-05T00:00:00Z"),
        &[],
    );
    assert_case("B1 unconstrained", &pol_b, &base(), &[(ALICE, "read", T1)]);
    assert_case("C1 deny-overrides", &pol_c, &base(), &[(ALICE, "denyRead", T1)]);
}

// ── generated permutations ─────────────────────────────────────────────────────────────

#[test]
fn action_hierarchy_matrix() {
    // policy action × request action across the mode table, the `use` umbrella, the
    // transfer subtree, and an unmapped action.
    for pol_act in ["read", "use", "modify", "transfer", "aggregate"] {
        for req_act in ["read", "modify", "append", "use", "sell", "aggregate"] {
            let ttl = policy_ttl("permission", &rule_body(pol_act, Some(T1), Some(ALICE), ""));
            let req = N3Request::new(act(req_act)).on(T1).by(ALICE);
            assert_parity(&format!("action {pol_act}×{req_act}"), &ttl, &req);
        }
    }
    // pinned edges of the matrix (independent statement so both-wrong cannot pass):
    // `use` permits read (grant), does NOT permit sell (nothing), and a `use` REQUEST
    // is unmapped (nothing even though the evaluator permits).
    let use_pol = policy_ttl("permission", &rule_body("use", Some(T1), Some(ALICE), ""));
    assert_case("use permits read", &use_pol, &N3Request::new(act("read")).on(T1).by(ALICE), &[(ALICE, "read", T1)]);
    assert_case("use excludes sell", &use_pol, &N3Request::new(act("sell")).on(T1).by(ALICE), &[]);
    assert_case("use request unmapped", &use_pol, &N3Request::new(act("use")).on(T1).by(ALICE), &[]);
}

#[test]
fn structural_target_assignee_permutations() {
    let targets: [Option<&str>; 3] = [Some(T1), Some(T2), None];
    let assignees: [Option<&str>; 3] = [Some(ALICE), Some(BOB), None];
    for t in targets {
        for a in assignees {
            let ttl = policy_ttl("permission", &rule_body("read", t, a, ""));
            // request always on T1 by ALICE
            assert_parity(
                &format!("structural target={t:?} assignee={a:?}"),
                &ttl,
                &N3Request::new(act("read")).on(T1).by(ALICE),
            );
            // partyless request (nothing can materialize; unconstrained-assignee
            // rules still MATCH in the evaluator — concreteness gates the triple)
            assert_parity(
                &format!("structural partyless target={t:?} assignee={a:?}"),
                &ttl,
                &N3Request::new(act("read")).on(T1),
            );
            // targetless request
            assert_parity(
                &format!("structural targetless target={t:?} assignee={a:?}"),
                &ttl,
                &N3Request::new(act("read")).by(ALICE),
            );
        }
    }
}

/// The six dateTime operators × request times either side of the bound, INCLUDING
/// mixed-offset lexicals that denote the same or nearby instants — the spike's
/// finding-(a) normalization surface. Bound: 2026-07-01T12:00:00Z.
#[test]
fn datetime_operator_window_matrix() {
    let times: [Option<&str>; 6] = [
        Some("2026-07-01T11:00:00Z"),      // before
        Some("2026-07-01T12:00:00Z"),      // equal (canonical)
        Some("2026-07-01T13:00:00Z"),      // after
        Some("2026-07-01T14:00:00+02:00"), // equal INSTANT via offset
        Some("2026-07-01T09:30:00-01:00"), // before via offset (=10:30Z)
        None,                              // unprovable
    ];
    for op in ["lt", "lteq", "gt", "gteq", "eq", "neq"] {
        let ttl = policy_ttl(
            "permission",
            &rule_body("read", Some(T1), Some(ALICE), &dt_constraint(op, "2026-07-01T12:00:00Z")),
        );
        for at in times {
            let mut req = N3Request::new(act("read")).on(T1).by(ALICE);
            if let Some(at) = at {
                req = req.at(at);
            }
            assert_parity(&format!("dateTime perm {op} at={at:?}"), &ttl, &req);
        }
    }
    // pinned: the offset form that equals the bound instant must NOT satisfy `lt`
    // and MUST satisfy `lteq`/`eq` (lexical comparison would get all three wrong).
    let lteq = policy_ttl(
        "permission",
        &rule_body("read", Some(T1), Some(ALICE), &dt_constraint("lteq", "2026-07-01T12:00:00Z")),
    );
    let lt = policy_ttl(
        "permission",
        &rule_body("read", Some(T1), Some(ALICE), &dt_constraint("lt", "2026-07-01T12:00:00Z")),
    );
    let eq = policy_ttl(
        "permission",
        &rule_body("read", Some(T1), Some(ALICE), &dt_constraint("eq", "2026-07-01T12:00:00Z")),
    );
    let offset_equal = N3Request::new(act("read")).on(T1).by(ALICE).at("2026-07-01T14:00:00+02:00");
    assert_case("offset-equal lteq grants", &lteq, &offset_equal, &[(ALICE, "read", T1)]);
    assert_case("offset-equal lt denies", &lt, &offset_equal, &[]);
    assert_case("offset-equal eq grants", &eq, &offset_equal, &[(ALICE, "read", T1)]);
}

/// Constraint-BEARING prohibitions — the spike's finding (c): these need the fuller
/// stratification (a prohibition's constraint statuses must be complete before the
/// deny-overrides negation runs). An unconstrained permission sits alongside, so a
/// mis-stratified rule set would either over-deny or over-grant visibly.
#[test]
fn datetime_constrained_prohibition_matrix() {
    let times: [Option<&str>; 6] = [
        Some("2026-07-01T11:00:00Z"),
        Some("2026-07-01T12:00:00Z"),
        Some("2026-07-01T13:00:00Z"),
        Some("2026-07-01T14:00:00+02:00"),
        Some("2026-07-01T09:30:00-01:00"),
        None,
    ];
    for op in ["lt", "lteq", "gt", "gteq", "eq", "neq"] {
        let ttl = policy_ttl2(
            &rule_body("read", Some(T1), Some(ALICE), ""),
            &rule_body("read", Some(T1), Some(ALICE), &dt_constraint(op, "2026-07-01T12:00:00Z")),
        );
        for at in times {
            let mut req = N3Request::new(act("read")).on(T1).by(ALICE);
            if let Some(at) = at {
                req = req.at(at);
            }
            assert_parity(&format!("dateTime proh {op} at={at:?}"), &ttl, &req);
        }
    }
    // pinned: within the prohibition window → deny WINS (and no allow triple);
    // window definitely lapsed → the permission grants; NO time evidence → the
    // prohibition is unprovable, does not carve out, and the permission grants
    // (matching the evaluator: an unproven prohibition never blocks — the
    // fail-closed direction is on GRANTS, not on hypothetical denies).
    let ttl = policy_ttl2(
        &rule_body("read", Some(T1), Some(ALICE), ""),
        &rule_body("read", Some(T1), Some(ALICE), &dt_constraint("lt", "2026-07-01T12:00:00Z")),
    );
    assert_case(
        "proh window active",
        &ttl,
        &N3Request::new(act("read")).on(T1).by(ALICE).at("2026-07-01T11:00:00Z"),
        &[(ALICE, "denyRead", T1)],
    );
    assert_case(
        "proh window lapsed",
        &ttl,
        &N3Request::new(act("read")).on(T1).by(ALICE).at("2026-07-01T13:00:00Z"),
        &[(ALICE, "read", T1)],
    );
    assert_case(
        "proh window unprovable",
        &ttl,
        &N3Request::new(act("read")).on(T1).by(ALICE),
        &[(ALICE, "read", T1)],
    );
}

#[test]
fn recipient_and_assignee_constraints() {
    for op in ["eq", "neq"] {
        for bound in [ALICE, BOB] {
            let ttl = policy_ttl(
                "permission",
                &rule_body("read", Some(T1), Some(ALICE), &iri_constraint("recipient", op, bound)),
            );
            // recipient defaults to the requesting party
            assert_parity(
                &format!("recipient {op} {bound} default"),
                &ttl,
                &N3Request::new(act("read")).on(T1).by(ALICE),
            );
            // explicit recipient evidence takes precedence over the party
            assert_parity(
                &format!("recipient {op} {bound} explicit"),
                &ttl,
                &N3Request::new(act("read"))
                    .on(T1)
                    .by(ALICE)
                    .with(RECIPIENT, Value::Iri(CAROL.to_owned())),
            );
            // no party AND no explicit recipient → unprovable (fail-closed)
            assert_parity(
                &format!("recipient {op} {bound} no evidence"),
                &ttl,
                &N3Request::new(act("read")).on(T1),
            );
        }
    }
    // the "everyone-except-BOB" prohibition shape: carved out for ALICE, not for BOB
    let ttl = policy_ttl2(
        &rule_body("read", Some(T1), None, ""),
        &rule_body("read", Some(T1), None, &iri_constraint("recipient", "neq", BOB)),
    );
    assert_case(
        "except-bob denies alice",
        &ttl,
        &N3Request::new(act("read")).on(T1).by(ALICE),
        &[(ALICE, "denyRead", T1)],
    );
    assert_case(
        "except-bob spares bob",
        &ttl,
        &N3Request::new(act("read")).on(T1).by(BOB),
        &[(BOB, "read", T1)],
    );
    // assignee-as-CONSTRAINT (distinct from the rule attribute): no evidence
    // dimension is supplied for it → unprovable → no grant on either path.
    let ttl = policy_ttl(
        "permission",
        &rule_body("read", Some(T1), None, &iri_constraint("assignee", "eq", ALICE)),
    );
    assert_case("assignee-constraint unprovable", &ttl, &N3Request::new(act("read")).on(T1).by(ALICE), &[]);
}

#[test]
fn purpose_constraints_flat_base_case() {
    let eq_ttl = policy_ttl(
        "permission",
        &rule_body("read", Some(T1), Some(ALICE), &iri_constraint("purpose", "eq", "urn:purpose/research")),
    );
    for (label, ev) in [
        ("match", Some("urn:purpose/research")),
        ("mismatch", Some("urn:purpose/marketing")),
        ("missing", None),
    ] {
        let mut req = N3Request::new(act("read")).on(T1).by(ALICE);
        if let Some(p) = ev {
            req = req.with(PURPOSE, Value::Iri(p.to_owned()));
        }
        assert_parity(&format!("purpose eq {label}"), &eq_ttl, &req);
    }
    // isPartOf over the compact `|`/space/comma set encoding
    let set_ttl = policy_ttl(
        "permission",
        &rule_body("read", Some(T1), Some(ALICE), &str_constraint("purpose", "isPartOf", "a|b c")),
    );
    for ev in ["a", "b", "c", "d"] {
        assert_parity(
            &format!("purpose isPartOf ev={ev}"),
            &set_ttl,
            &N3Request::new(act("read")).on(T1).by(ALICE).with(PURPOSE, Value::Str(ev.to_owned())),
        );
    }
    assert_parity(
        "purpose isPartOf missing",
        &set_ttl,
        &N3Request::new(act("read")).on(T1).by(ALICE),
    );
}

/// `and` / `or` / `xone` over two atomic operands (purpose, spatial), sweeping the
/// operand statuses: Satisfied / DefinitelyUnsatisfied / Unprovable per operand.
#[test]
fn logical_combinator_status_matrix() {
    let operands = [
        iri_constraint("purpose", "eq", "urn:purpose/research"),
        iri_constraint("spatial", "eq", "urn:region/eu"),
    ];
    // evidence patterns → (purpose evidence, spatial evidence); None = Unprovable
    let patterns: [(&str, Option<&str>, Option<&str>); 7] = [
        ("SS", Some("urn:purpose/research"), Some("urn:region/eu")),
        ("SD", Some("urn:purpose/research"), Some("urn:region/us")),
        ("SU", Some("urn:purpose/research"), None),
        ("DD", Some("urn:purpose/marketing"), Some("urn:region/us")),
        ("DU", Some("urn:purpose/marketing"), None),
        ("UU", None, None),
        ("DS", Some("urn:purpose/marketing"), Some("urn:region/eu")),
    ];
    for combinator in ["and", "or", "xone"] {
        let ttl = policy_ttl(
            "permission",
            &rule_body("read", Some(T1), Some(ALICE), &logical(combinator, &operands)),
        );
        for (label, purpose, spatial) in patterns {
            let mut req = N3Request::new(act("read")).on(T1).by(ALICE);
            if let Some(p) = purpose {
                req = req.with(PURPOSE, Value::Iri(p.to_owned()));
            }
            if let Some(s) = spatial {
                req = req.with(SPATIAL, Value::Iri(s.to_owned()));
            }
            assert_parity(&format!("{combinator} {label}"), &ttl, &req);
        }
        // single-operand compound
        let ttl1 = policy_ttl(
            "permission",
            &rule_body("read", Some(T1), Some(ALICE), &logical(combinator, &operands[..1])),
        );
        assert_parity(
            &format!("{combinator} single sat"),
            &ttl1,
            &N3Request::new(act("read"))
                .on(T1)
                .by(ALICE)
                .with(PURPOSE, Value::Iri("urn:purpose/research".to_owned())),
        );
        // compound on a PROHIBITION (with an unconstrained permission alongside)
        let ttl_p = policy_ttl2(
            &rule_body("read", Some(T1), Some(ALICE), ""),
            &rule_body("read", Some(T1), Some(ALICE), &logical(combinator, &operands)),
        );
        for (label, purpose, spatial) in patterns {
            let mut req = N3Request::new(act("read")).on(T1).by(ALICE);
            if let Some(p) = purpose {
                req = req.with(PURPOSE, Value::Iri(p.to_owned()));
            }
            if let Some(s) = spatial {
                req = req.with(SPATIAL, Value::Iri(s.to_owned()));
            }
            assert_parity(&format!("proh {combinator} {label}"), &ttl_p, &req);
        }
    }
    // pinned xone semantics: exactly-one satisfied grants; two satisfied does not;
    // an unprovable sibling never lets xone claim "exactly one" (fail-closed).
    let xone = policy_ttl(
        "permission",
        &rule_body("read", Some(T1), Some(ALICE), &logical("xone", &operands)),
    );
    let one = N3Request::new(act("read"))
        .on(T1)
        .by(ALICE)
        .with(PURPOSE, Value::Iri("urn:purpose/research".to_owned()))
        .with(SPATIAL, Value::Iri("urn:region/us".to_owned()));
    assert_case("xone exactly-one grants", &xone, &one, &[(ALICE, "read", T1)]);
    let two = N3Request::new(act("read"))
        .on(T1)
        .by(ALICE)
        .with(PURPOSE, Value::Iri("urn:purpose/research".to_owned()))
        .with(SPATIAL, Value::Iri("urn:region/eu".to_owned()));
    assert_case("xone two-sat denies", &xone, &two, &[]);
    let one_unprovable = N3Request::new(act("read"))
        .on(T1)
        .by(ALICE)
        .with(PURPOSE, Value::Iri("urn:purpose/research".to_owned()));
    assert_case("xone unprovable sibling denies", &xone, &one_unprovable, &[]);
}

#[test]
fn duties_gate_grants_not_matching() {
    let anonymize = "odrl:duty [ odrl:action odrl:anonymize ]".to_owned();
    let ttl = policy_ttl("permission", &rule_body("read", Some(T1), Some(ALICE), &anonymize));
    assert_case(
        "duty discharged",
        &ttl,
        &N3Request::new(act("read")).on(T1).by(ALICE).discharge(act("anonymize")),
        &[(ALICE, "read", T1)],
    );
    assert_case("duty undischarged", &ttl, &N3Request::new(act("read")).on(T1).by(ALICE), &[]);
    // two duties, one discharged → still blocked
    let two = "odrl:duty [ odrl:action odrl:anonymize ] , [ odrl:action odrl:compensate ]".to_owned();
    let ttl2 = policy_ttl("permission", &rule_body("read", Some(T1), Some(ALICE), &two));
    assert_case(
        "one of two duties discharged",
        &ttl2,
        &N3Request::new(act("read")).on(T1).by(ALICE).discharge(act("anonymize")),
        &[],
    );
    // a second CLEAN permission grants even though the duty-bearing one is blocked
    let mixed = format!(
        "@prefix odrl: <http://www.w3.org/ns/odrl/2/> .\n\
         <urn:pol/1> a odrl:Set ; \
         odrl:permission [ {} ] ; odrl:permission [ {} ] .",
        rule_body("read", Some(T1), Some(ALICE), &anonymize),
        rule_body("read", Some(T1), Some(ALICE), "")
    );
    assert_case(
        "clean sibling permission grants",
        &mixed,
        &N3Request::new(act("read")).on(T1).by(ALICE),
        &[(ALICE, "read", T1)],
    );
}

#[test]
fn conflict_strategy_refusals_mirror() {
    let body = rule_body("read", Some(T1), Some(ALICE), "");
    let proh = rule_body("read", Some(T1), Some(ALICE), "");
    let req = N3Request::new(act("read")).on(T1).by(ALICE);
    // deny-overrides (odrl:prohibit) is the admissible strategy
    let ok = format!(
        "@prefix odrl: <http://www.w3.org/ns/odrl/2/> .\n\
         <urn:pol/1> a odrl:Set ; odrl:conflict odrl:prohibit ; odrl:permission [ {body} ] ."
    );
    assert_case("conflict prohibit admissible", &ok, &req, &[(ALICE, "read", T1)]);
    // odrl:perm / unknown → REFUSED on both paths (nothing materialized)
    for strategy in ["odrl:perm", "<urn:strategy/unknown>"] {
        let ttl = format!(
            "@prefix odrl: <http://www.w3.org/ns/odrl/2/> .\n\
             <urn:pol/1> a odrl:Set ; odrl:conflict {strategy} ; odrl:permission [ {body} ] ."
        );
        assert_case(&format!("conflict {strategy} refused"), &ttl, &req, &[]);
    }
    // odrl:invalid with a DETECTED conflict → refused; without one → admissible
    let invalid_conflicting = format!(
        "@prefix odrl: <http://www.w3.org/ns/odrl/2/> .\n\
         <urn:pol/1> a odrl:Set ; odrl:conflict odrl:invalid ; \
         odrl:permission [ {body} ] ; odrl:prohibition [ {proh} ] ."
    );
    assert_case("conflict invalid+conflict refused", &invalid_conflicting, &req, &[]);
    let invalid_clean = format!(
        "@prefix odrl: <http://www.w3.org/ns/odrl/2/> .\n\
         <urn:pol/1> a odrl:Set ; odrl:conflict odrl:invalid ; odrl:permission [ {body} ] ."
    );
    assert_parity("conflict invalid clean", &invalid_clean, &req);
}

#[test]
fn deny_overrides_compositions() {
    // prohibition on a DIFFERENT action does not block
    let ttl = policy_ttl2(
        &rule_body("read", Some(T1), Some(ALICE), ""),
        &rule_body("modify", Some(T1), Some(ALICE), ""),
    );
    assert_case(
        "unrelated prohibition",
        &ttl,
        &N3Request::new(act("read")).on(T1).by(ALICE),
        &[(ALICE, "read", T1)],
    );
    // `use`-umbrella prohibition carves out a read request (hierarchy on the DENY side)
    let ttl = policy_ttl2(
        &rule_body("read", Some(T1), Some(ALICE), ""),
        &rule_body("use", Some(T1), Some(ALICE), ""),
    );
    assert_case(
        "umbrella prohibition denies read",
        &ttl,
        &N3Request::new(act("read")).on(T1).by(ALICE),
        &[(ALICE, "denyRead", T1)],
    );
    // prohibition scoped to BOB does not carve out ALICE
    let ttl = policy_ttl2(
        &rule_body("read", Some(T1), Some(ALICE), ""),
        &rule_body("read", Some(T1), Some(BOB), ""),
    );
    assert_case(
        "other-party prohibition",
        &ttl,
        &N3Request::new(act("read")).on(T1).by(ALICE),
        &[(ALICE, "read", T1)],
    );
    // prohibition on another TARGET does not carve out T1
    let ttl = policy_ttl2(
        &rule_body("read", Some(T1), Some(ALICE), ""),
        &rule_body("read", Some(T2), Some(ALICE), ""),
    );
    assert_case(
        "other-target prohibition",
        &ttl,
        &N3Request::new(act("read")).on(T1).by(ALICE),
        &[(ALICE, "read", T1)],
    );
    // an UNMAPPED-action matching prohibition still blocks the grant (evaluate
    // denies) even though no deny triple can be materialized
    let ttl = policy_ttl2(
        &rule_body("aggregate", Some(T1), Some(ALICE), ""),
        &rule_body("aggregate", Some(T1), Some(ALICE), ""),
    );
    assert_case(
        "unmapped action both ways",
        &ttl,
        &N3Request::new(act("aggregate")).on(T1).by(ALICE),
        &[],
    );
    // append/write modes flow through the deny table
    let ttl = policy_ttl2(
        &rule_body("append", Some(T1), Some(ALICE), ""),
        &rule_body("append", Some(T1), Some(ALICE), ""),
    );
    assert_case(
        "append deny mode",
        &ttl,
        &N3Request::new(act("append")).on(T1).by(ALICE),
        &[(ALICE, "denyAppend", T1)],
    );
    // a PROHIBITION-ONLY policy: the matched carve-out materializes its deny with no
    // permission in sight; an unmatched one materializes nothing
    let ttl = policy_ttl("prohibition", &rule_body("modify", Some(T1), Some(ALICE), ""));
    assert_case(
        "prohibition-only matched",
        &ttl,
        &N3Request::new(act("modify")).on(T1).by(ALICE),
        &[(ALICE, "denyWrite", T1)],
    );
    assert_case(
        "prohibition-only unmatched",
        &ttl,
        &N3Request::new(act("modify")).on(T2).by(ALICE),
        &[],
    );
    // the `use` umbrella prohibition does NOT carve out the transfer subtree
    let ttl = policy_ttl("prohibition", &rule_body("use", Some(T1), Some(ALICE), ""));
    assert_case("umbrella spares sell", &ttl, &N3Request::new(act("sell")).on(T1).by(ALICE), &[]);
}

#[test]
fn malformed_constraints_fail_closed_on_both_paths() {
    // a structurally-incomplete constraint (no operator/right) parses to the
    // unsatisfiable guard on the Rust side; the N3 side must agree: NOTHING.
    let ttl = format!(
        "@prefix odrl: <http://www.w3.org/ns/odrl/2/> .\n\
         <urn:pol/1> a odrl:Set ; odrl:permission [ {} ; \
         odrl:constraint [ odrl:leftOperand odrl:purpose ] ] .",
        rule_body("read", Some(T1), Some(ALICE), "")
    );
    assert_case(
        "incomplete constraint",
        &ttl,
        &N3Request::new(act("read"))
            .on(T1)
            .by(ALICE)
            .with(PURPOSE, Value::Iri("urn:purpose/research".to_owned())),
        &[],
    );
    // an unknown operator likewise becomes the unsatisfiable guard
    let ttl = format!(
        "@prefix odrl: <http://www.w3.org/ns/odrl/2/> .\n\
         <urn:pol/1> a odrl:Set ; odrl:permission [ {} ; \
         odrl:constraint [ odrl:leftOperand odrl:purpose ; odrl:operator odrl:hasPart ; \
                           odrl:rightOperand <urn:purpose/research> ] ] .",
        rule_body("read", Some(T1), Some(ALICE), "")
    );
    assert_case(
        "unknown operator",
        &ttl,
        &N3Request::new(act("read"))
            .on(T1)
            .by(ALICE)
            .with(PURPOSE, Value::Iri("urn:purpose/research".to_owned())),
        &[],
    );
}

// ── out-of-subset constructs are LOUD errors, never silent divergence ─────────────────

#[test]
fn out_of_subset_is_a_loud_error_not_a_divergence() {
    let req = || N3Request::new(act("read")).on(T1).by(ALICE);
    let cases: [(&str, String); 5] = [
        (
            "fractional-second bound",
            policy_ttl(
                "permission",
                &rule_body("read", Some(T1), Some(ALICE), &dt_constraint("lteq", "2026-12-31T00:00:00.5Z")),
            ),
        ),
        (
            "negative-year bound",
            policy_ttl(
                "permission",
                &rule_body("read", Some(T1), Some(ALICE), &dt_constraint("lteq", "-0044-03-15T00:00:00Z")),
            ),
        ),
        (
            "order operator on purpose",
            policy_ttl(
                "permission",
                &rule_body("read", Some(T1), Some(ALICE), &str_constraint("purpose", "lteq", "5")),
            ),
        ),
        (
            "numeric count bound",
            policy_ttl(
                "permission",
                &rule_body(
                    "read",
                    Some(T1),
                    Some(ALICE),
                    "odrl:constraint [ odrl:leftOperand odrl:count ; odrl:operator odrl:lteq ; \
                     odrl:rightOperand 5 ]",
                ),
            ),
        ),
        (
            "nested logical constraint",
            policy_ttl(
                "permission",
                &rule_body(
                    "read",
                    Some(T1),
                    Some(ALICE),
                    "odrl:constraint [ odrl:and [ odrl:or [ odrl:leftOperand odrl:purpose ; \
                     odrl:operator odrl:eq ; odrl:rightOperand <urn:p/r> ] ] ]",
                ),
            ),
        ),
    ];
    for (label, ttl) in cases {
        let policy = parse_policy_str(&ttl, "turtle").expect("parses");
        let mut g = Graph::new();
        let err = materialize_policy_n3(&mut g, &policy, &req())
            .expect_err(&format!("{label}: must be a loud error"));
        assert!(!err.is_empty(), "{label}: error carries a reason");
        // and NOTHING was materialized (fail-closed)
        assert!(
            g.named.is_empty(),
            "{label}: an out-of-subset refusal must materialize nothing"
        );
    }
    // request-side: fractional-second evaluation time
    let ttl = policy_ttl("permission", &rule_body("read", Some(T1), Some(ALICE), ""));
    let policy = parse_policy_str(&ttl, "turtle").expect("parses");
    let mut g = Graph::new();
    let err = materialize_policy_n3(&mut g, &policy, &req().at("2026-07-01T00:00:00.123Z"))
        .expect_err("fractional-second request time must be a loud error");
    assert!(err.contains("lexical space"), "clear reason, got: {err}");
    assert!(g.named.is_empty());
}
