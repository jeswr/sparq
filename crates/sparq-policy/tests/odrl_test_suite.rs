//! [OPUS-4.8] sq-tmsd6 — the **SolidLab ODRL-Test-Suite** wired as a conformance
//! RATCHET, mirroring the crate-local Solid WAC/ACP decision-parity ratchets in
//! `sparq-solid` (a `cargo test` runner + a pinned pass-count FLOOR that may only
//! RISE, registered in the central `sparq_conformance::scoreboard::SUITES` and
//! guarded by `sparq-conformance`'s `tests/scoreboard_floors.rs`).
//!
//! ## The suite
//!
//! `github.com/SolidLabResearch/ODRL-Test-Suite` (MIT, `LICENSE.md`), pinned in
//! `scripts/fetch-odrl-suite.sh`. The suite ships 68 self-describing Turtle test
//! cases under `data/test_cases/*.ttl`, each of which references — BY UUID — a
//! policy (`data/policies/*.ttl`), a request (`data/requests/*.ttl`), a
//! state-of-the-world (`data/sotw/*.ttl`) and an *expected compliance report*.
//! The data is FETCHED, never committed (gitignored under `/tests/odrl-test-suite/`,
//! like the W3C suites under `tests/w3c/`). When the fetched dir is ABSENT this
//! runner SELF-SKIPS cleanly so `cargo test` is green offline; the CI lane
//! (`odrl-conformance`) does the fetch and enforces the floor.
//!
//! ## What fragment is exercised
//!
//! Every policy in the suite is an `odrl:Set` of `odrl:permission` /
//! `odrl:prohibition` rules carrying `odrl:action` / `odrl:target` /
//! `odrl:assignee` and optional `odrl:constraint` blocks over `odrl:dateTime`
//! (`odrl:eq|lt|lteq|gt|gteq`) plus `odrl:duty` obligations. That is EXACTLY the
//! ODRL fragment `sparq-policy`'s `parse_policy_str` + `evaluate` already support
//! (the single-node base case). The runner loads each policy FILE through the
//! REAL parse path (`sparq_policy::parse_policy_str`), derives a real
//! `sparq_policy::Request` from the request + sotw files, and evaluates it
//! through the REAL `sparq_policy::evaluate` — this is a genuine differential
//! oracle, not a mock.
//!
//! ## The oracle (load-bearing invariant)
//!
//! Each test case's expected report names ONE rule report (`report:ruleReport`):
//!
//! * `report:PermissionReport` + `report:activationState report:Active` ⇒ ALLOW.
//! * `report:ProhibitionReport` + `report:activationState report:Active` ⇒ DENY
//!   (the prohibition fires).
//! * `report:activationState report:Inactive` (either kind) ⇒ DENY.
//!
//! A case PASSES iff `sparq_policy::evaluate(&policy, &request).allow` equals that
//! expected boolean. The floor `ODRL_SUITE_FLOOR` is the ACTUAL current pass count
//! at the pinned revision; it may only RISE.
//!
//! ## Feature states
//!
//! `sparq-policy`'s ODRL evaluator is the crate's DEFAULT surface (not
//! feature-gated), so this lane needs NO new cargo feature. The crate's only
//! feature is `count-enforcement` (stateful `odrl:count`), which the ODRL suite
//! does not exercise — so this runner passes IDENTICALLY in both states:
//! `cargo test -p sparq-policy --test odrl_test_suite` (default) and
//! `… --features count-enforcement`.

use sparq_core::Graph;
use sparq_policy::{evaluate, parse_policy_str, Request};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

// ---- The RATCHET floor. The ACTUAL current pass count at the pinned suite
// revision (scripts/fetch-odrl-suite.sh). MIRRORED in the central scoreboard
// (sparq_conformance::scoreboard::SUITES) and read TEXTUALLY by the floor-sync
// guard (crates/sparq-conformance/tests/scoreboard_floors.rs); it may only RISE.
// Declared `pub const … : usize = N;` at TOP LEVEL so the guard's `const_floor_in`
// helper (which matches `pub const NAME: usize = N;`) can read it. [OPUS-4.8]
//
/// Pass-count floor for the SolidLab ODRL Test Suite. RATCHET: may only RISE.
/// At the pinned revision 59 of the 68 self-describing cases pass through the
/// real `parse_policy_str` + `evaluate` path; the remaining 9 are in the
/// documented NOT-IMPLEMENTED bucket — constructs outside sparq-policy's
/// supported single-node base fragment: `odrl:LogicalConstraint`/`odrl:and`
/// compound constraints, `odrl:PartyCollection`/`odrl:AssetCollection`
/// membership, the `odrl:use` umbrella-action divergence (sparq treats `use` as
/// a super-action; SolidLab requires exact match), and a duty whose discharge
/// state is unknown/NonSet (sparq is fail-closed). Raise this as those land.
pub const ODRL_SUITE_FLOOR: usize = 59;

/// The size of the documented not-implemented bucket at the pinned revision. The
/// runner pins it so the bucket cannot silently GROW (a new regression hidden as
/// "not-implemented") — it must SHRINK as the fragment grows.
const NOT_IMPLEMENTED_CEILING: usize = 9;

const ODRL_NS: &str = "http://www.w3.org/ns/odrl/2/";
const REPORT_NS: &str = "https://w3id.org/force/compliance-report#";
const DCT_NS: &str = "http://purl.org/dc/terms/";

/// Workspace-root-relative location of the fetched suite (gitignored), discovered
/// from this crate's manifest dir (`crates/sparq-policy`; workspace root is two up).
fn suite_data_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/odrl-test-suite/data")
}

/// The expected decision an expected-report rule-report encodes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Expected {
    Allow,
    Deny,
}

#[derive(Default)]
struct Score {
    pass: usize,
    fail: usize,
    /// Cases whose ODRL construct is outside sparq-policy's supported fragment
    /// — recorded honestly, NOT counted as a failure and NOT gating.
    not_implemented: usize,
    failures: Vec<String>,
}

/// Read every `*.ttl` in `dir`, returning a `subject-UUID → file-text` index.
/// Each policy/request/sotw file declares ITS subject (the UUID a test case
/// references) via the typed subject `?s a <wanted_type>`; we index by that
/// subject IRI so a test case's UUID resolves to the right file regardless of
/// filename. [OPUS-4.8]
fn index_by_subject(dir: &Path, wanted_type_iri: &str) -> BTreeMap<String, String> {
    let mut idx = BTreeMap::new();
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return idx,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("ttl") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(graph) = Graph::load_str(&text, "turtle") else {
            continue;
        };
        // The typed subject is the file's "primary" node (the policy/request/sotw).
        let q = format!("SELECT ?s WHERE {{ ?s a <{}> }}", wanted_type_iri);
        if let Ok(res) = sparq_engine::query(&graph, &q) {
            for row in res.rows {
                if let Some(Some(oxrdf::Term::NamedNode(n))) = row.into_iter().next() {
                    idx.insert(n.into_string(), text.clone());
                    break;
                }
            }
        }
    }
    idx
}

/// First IRI object of `?subject <predicate> ?o` in `graph`.
fn first_iri_object(graph: &Graph, subject: &str, predicate: &str) -> Option<String> {
    let q = format!(
        "SELECT ?o WHERE {{ <{}> <{}> ?o }}",
        subject.trim_start_matches('<').trim_end_matches('>'),
        predicate
    );
    let res = sparq_engine::query(graph, &q).ok()?;
    for row in res.rows {
        if let Some(Some(oxrdf::Term::NamedNode(n))) = row.into_iter().next() {
            return Some(n.into_string());
        }
    }
    None
}

/// First literal lexical value of `?subject <predicate> ?o`.
fn first_literal_object(graph: &Graph, subject: &str, predicate: &str) -> Option<String> {
    let q = format!(
        "SELECT ?o WHERE {{ <{}> <{}> ?o }}",
        subject.trim_start_matches('<').trim_end_matches('>'),
        predicate
    );
    let res = sparq_engine::query(graph, &q).ok()?;
    for row in res.rows {
        if let Some(Some(oxrdf::Term::Literal(l))) = row.into_iter().next() {
            return Some(l.value().to_owned());
        }
    }
    None
}

/// Derive the EXPECTED decision from a test-case graph's expected report.
///
/// Returns `Err` if the report shape is not the
/// PermissionReport/ProhibitionReport + activationState form the oracle covers
/// (so an unsupported report shape lands in the not-implemented bucket rather
/// than being silently scored).
fn expected_decision(graph: &Graph, expected_report: &str) -> Result<Expected, String> {
    let rule_report = first_iri_object(graph, expected_report, &format!("{REPORT_NS}ruleReport"))
        .ok_or("expected report has no report:ruleReport")?;
    let kind = first_iri_object(
        graph,
        &rule_report,
        "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
    )
    .ok_or("rule report has no rdf:type")?;
    let activation = first_iri_object(graph, &rule_report, &format!("{REPORT_NS}activationState"))
        .ok_or("rule report has no report:activationState")?;

    let active = activation == format!("{REPORT_NS}Active");
    let inactive = activation == format!("{REPORT_NS}Inactive");
    if !active && !inactive {
        return Err(format!("unrecognized activationState {activation}"));
    }
    if !active {
        // Inactive (either kind) ⇒ DENY.
        return Ok(Expected::Deny);
    }
    if kind == format!("{REPORT_NS}PermissionReport") {
        Ok(Expected::Allow)
    } else if kind == format!("{REPORT_NS}ProhibitionReport") {
        // An ACTIVE prohibition fires ⇒ DENY.
        Ok(Expected::Deny)
    } else {
        Err(format!("unrecognized rule report kind {kind}"))
    }
}

/// Build a `sparq_policy::Request` from the request file + the sotw file.
///
/// Request file: `a odrl:Request ; odrl:permission [ odrl:assignee P ;
/// odrl:action A ; odrl:target T ]`. sotw file: `temp:currentTime dct:issued
/// "…"^^xsd:dateTime` (the evaluation instant) and, for duty cases, a
/// `report:deonticState report:Fulfilled` marking duties discharged.
fn build_request(
    request_ttl: &str,
    sotw_ttl: &str,
    policy: &sparq_policy::Policy,
) -> Result<Request, String> {
    let rgraph = Graph::load_str(request_ttl, "turtle").map_err(|e| e.to_string())?;
    // The request's inner permission node (object of odrl:permission on the Request).
    let req_subj = {
        let q = format!("SELECT ?s WHERE {{ ?s a <{ODRL_NS}Request> }}");
        let res = sparq_engine::query(&rgraph, &q).map_err(|e| e.to_string())?;
        res.rows
            .into_iter()
            .next()
            .and_then(|r| r.into_iter().next().flatten())
            .and_then(|t| match t {
                oxrdf::Term::NamedNode(n) => Some(n.into_string()),
                _ => None,
            })
            .ok_or("request file has no odrl:Request subject")?
    };
    let perm = first_iri_object(&rgraph, &req_subj, &format!("{ODRL_NS}permission"))
        .ok_or("request has no odrl:permission")?;
    let action = first_iri_object(&rgraph, &perm, &format!("{ODRL_NS}action"))
        .ok_or("request permission has no odrl:action")?;
    let mut req = Request::new(action);
    if let Some(target) = first_iri_object(&rgraph, &perm, &format!("{ODRL_NS}target")) {
        req = req.on(target);
    }
    if let Some(assignee) = first_iri_object(&rgraph, &perm, &format!("{ODRL_NS}assignee")) {
        req = req.by(assignee);
    }

    // sotw: evaluation instant (temp:currentTime dct:issued "…") + duty discharge.
    let sgraph = Graph::load_str(sotw_ttl, "turtle").map_err(|e| e.to_string())?;
    if let Some(instant) = first_literal_object(
        &sgraph,
        "http://example.com/request/currentTime",
        &format!("{DCT_NS}issued"),
    ) {
        req = req.at(instant);
    }
    // If the sotw asserts ANY duty fulfilled, discharge every duty action named
    // in the policy's permissions (the suite's duty cases have a single duty).
    let fulfilled = {
        // ASK is returned as zero variables + one (empty) row iff the pattern
        // matches (see sparq-engine), so a non-empty result row set ⇒ true.
        let q = format!("ASK {{ ?s <{REPORT_NS}deonticState> <{REPORT_NS}Fulfilled> }}");
        sparq_engine::query(&sgraph, &q)
            .map(|r| !r.rows.is_empty())
            .unwrap_or(false)
    };
    if fulfilled {
        for rule in &policy.permissions {
            for duty in &rule.duties {
                req = req.discharge(duty.action.0.clone());
            }
        }
    }
    Ok(req)
}

/// The principled reason a test case's ODRL construct is OUTSIDE sparq-policy's
/// supported single-node base fragment, if any. Detected from the policy graph
/// (and the request action / sotw duty state). A case routes to the
/// not-implemented bucket ONLY when it both mismatches the oracle AND carries one
/// of these — so an UNEXPLAINED mismatch is a genuine FAIL that gates (no silent
/// "mismatch ⇒ not-implemented"). [OPUS-4.8]
fn unsupported_construct(
    policy_graph: &Graph,
    request_action: &str,
    duty_state_unknown: bool,
) -> Option<&'static str> {
    let ask = |q: &str| {
        sparq_engine::query(policy_graph, q)
            .map(|r| !r.rows.is_empty())
            .unwrap_or(false)
    };
    // (a) Compound/logical constraints — `odrl:LogicalConstraint` + `odrl:and`.
    // The parser models a direct `odrl:constraint [ leftOperand … ]` block, not
    // the LogicalConstraint indirection, so the compound constraint is not
    // evaluated. (sq-tmsd6 follow-up.)
    if ask(&format!("ASK {{ ?s a <{ODRL_NS}LogicalConstraint> }}"))
        || ask(&format!("ASK {{ ?s <{ODRL_NS}and> ?o }}"))
    {
        return Some("odrl:LogicalConstraint / odrl:and compound constraint (unsupported)");
    }
    // (b) Party/asset COLLECTION membership — assignee/target is a collection and
    // the requester/resource is a member (`odrl:partOf` / `odrl:source`). The base
    // case matches assignee/target by IRI equality, not collection membership.
    if ask(&format!("ASK {{ ?s a <{ODRL_NS}PartyCollection> }}"))
        || ask(&format!("ASK {{ ?s a <{ODRL_NS}AssetCollection> }}"))
    {
        return Some("odrl:PartyCollection / odrl:AssetCollection membership (unsupported)");
    }
    // (c) The `odrl:use` umbrella-action SEMANTIC DIVERGENCE. sparq-policy treats
    // `odrl:use` as the ODRL super-action that permits any action; the SolidLab
    // oracle requires an exact action match (a `use` permission is Inactive for a
    // `sell`/`read`/… request). Documented divergence, not a bug in either.
    if request_action != format!("{ODRL_NS}use")
        && ask(&format!("ASK {{ ?perm <{ODRL_NS}action> <{ODRL_NS}use> }}"))
    {
        return Some("odrl:use umbrella-action vs exact-match divergence (documented)");
    }
    // (d) A duty whose discharge state is UNKNOWN/NonSet. sparq-policy is
    // fail-closed (a duty must be explicitly discharged); the SolidLab oracle
    // treats a not-VIOLATED (unknown) duty as active. Documented divergence.
    if duty_state_unknown && ask(&format!("ASK {{ ?perm <{ODRL_NS}duty> ?d }}")) {
        return Some("duty discharge unknown/NonSet (fail-closed vs not-violated divergence)");
    }
    None
}

/// The outcome of running one test case.
enum Outcome {
    Pass,
    /// A genuine FAILURE (mismatch with no unsupported-construct explanation) —
    /// this GATES.
    Fail,
    /// Outside the supported fragment (a documented divergence / unsupported
    /// construct) — recorded honestly, does NOT count as a failure, does NOT gate.
    NotImplemented(&'static str),
}

/// Run one test-case file through the real `parse_policy_str` + `evaluate` path
/// and compare against the expected-report oracle.
fn run_case(
    path: &Path,
    policies: &BTreeMap<String, String>,
    requests: &BTreeMap<String, String>,
    sotws: &BTreeMap<String, String>,
) -> Result<Outcome, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let graph = Graph::load_str(&text, "turtle").map_err(|e| e.to_string())?;
    let case_subj = {
        let q = "SELECT ?s WHERE { ?s a <http://example.org/TestCase> }".to_string();
        let res = sparq_engine::query(&graph, &q).map_err(|e| e.to_string())?;
        res.rows
            .into_iter()
            .next()
            .and_then(|r| r.into_iter().next().flatten())
            .and_then(|t| match t {
                oxrdf::Term::NamedNode(n) => Some(n.into_string()),
                _ => None,
            })
            .ok_or("test case file has no ex:TestCase subject")?
    };
    let policy_uuid = first_iri_object(&graph, &case_subj, "http://example.org/policy")
        .ok_or("test case has no ex:policy")?;
    let request_uuid = first_iri_object(&graph, &case_subj, "http://example.org/request")
        .ok_or("test case has no ex:request")?;
    let sotw_uuid = first_iri_object(&graph, &case_subj, "http://example.org/sotw")
        .ok_or("test case has no ex:sotw")?;
    let expected_report = first_iri_object(&graph, &case_subj, "http://example.org/expectedReport")
        .ok_or("test case has no ex:expectedReport")?;

    let policy_ttl = policies
        .get(&policy_uuid)
        .ok_or_else(|| format!("no policy file for {policy_uuid}"))?;
    let request_ttl = requests
        .get(&request_uuid)
        .ok_or_else(|| format!("no request file for {request_uuid}"))?;
    let sotw_ttl = sotws
        .get(&sotw_uuid)
        .ok_or_else(|| format!("no sotw file for {sotw_uuid}"))?;

    let expected = expected_decision(&graph, &expected_report)?;
    let policy = parse_policy_str(policy_ttl, "turtle")?;
    let request = build_request(request_ttl, sotw_ttl, &policy)?;
    let decision = evaluate(&policy, &request);

    let ours = if decision.allow {
        Expected::Allow
    } else {
        Expected::Deny
    };
    if ours == expected {
        return Ok(Outcome::Pass);
    }

    // Mismatch: route to the not-implemented bucket ONLY when the case carries a
    // construct outside the supported fragment; otherwise it is a genuine FAIL.
    let policy_graph = Graph::load_str(policy_ttl, "turtle").map_err(|e| e.to_string())?;
    let request_action = request.action.clone();
    let duty_state_unknown = {
        let sgraph = Graph::load_str(sotw_ttl, "turtle").map_err(|e| e.to_string())?;
        // The sotw asserts a duty whose deonticState is NOT Fulfilled (Unknown /
        // NonSet) — sparq-policy is fail-closed; SolidLab treats it as active.
        let has_duty_report =
            sparq_engine::query(&sgraph, &format!("ASK {{ ?s a <{REPORT_NS}DutyReport> }}"))
                .map(|r| !r.rows.is_empty())
                .unwrap_or(false);
        let fulfilled = sparq_engine::query(
            &sgraph,
            &format!("ASK {{ ?s <{REPORT_NS}deonticState> <{REPORT_NS}Fulfilled> }}"),
        )
        .map(|r| !r.rows.is_empty())
        .unwrap_or(false);
        has_duty_report && !fulfilled
    };
    match unsupported_construct(&policy_graph, &request_action, duty_state_unknown) {
        Some(reason) => Ok(Outcome::NotImplemented(reason)),
        None => Ok(Outcome::Fail),
    }
}

#[test]
fn odrl_test_suite_decision_parity() {
    let data = suite_data_root();
    if !data.join("test_cases").is_dir() {
        // [OPUS-4.8] Off/no-fixture state: SELF-SKIP cleanly so `cargo test` is
        // green without the fetch (the off state is the lean default). The CI
        // lane runs scripts/fetch-odrl-suite.sh first, then enforces the floor.
        eprintln!(
            "SKIP: SolidLab ODRL Test Suite not present at {} — run scripts/fetch-odrl-suite.sh",
            data.display()
        );
        return;
    }

    let policies = index_by_subject(&data.join("policies"), &format!("{ODRL_NS}Set"));
    let requests = index_by_subject(&data.join("requests"), &format!("{ODRL_NS}Request"));
    let sotws = index_by_subject(&data.join("sotw"), "http://example.org/Sotw");

    let mut case_files: Vec<PathBuf> = std::fs::read_dir(data.join("test_cases"))
        .expect("read test_cases dir")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("ttl"))
        .collect();
    case_files.sort();

    let mut score = Score::default();
    for path in &case_files {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_owned();
        match run_case(path, &policies, &requests, &sotws) {
            Ok(Outcome::Pass) => score.pass += 1,
            // A GENUINE failure: a decision mismatch with NO unsupported-construct
            // explanation. This counts as a fail and GATES (a real regression).
            Ok(Outcome::Fail) => {
                score.fail += 1;
                score
                    .failures
                    .push(format!("{name}: decision mismatch vs expected report"));
            }
            // Outside the supported fragment → not-implemented bucket (does NOT
            // count as fail, does NOT gate). Each entry names the principled
            // reason (compound constraint / collection membership / documented
            // semantic divergence), so the bucket stays honest and shrinks as the
            // fragment grows.
            Ok(Outcome::NotImplemented(reason)) => {
                score.not_implemented += 1;
                score
                    .failures
                    .push(format!("{name}: NOT-IMPLEMENTED ({reason})"));
            }
            // A harness error (a malformed case / missing referenced file) is a
            // real problem with the suite wiring — surface it as a failure.
            Err(why) => {
                score.fail += 1;
                score
                    .failures
                    .push(format!("{name}: HARNESS ERROR ({why})"));
            }
        }
    }

    // CI-greppable summary line (mirrors the Solid `WAC scenarios pass N / fail M`
    // shape). KEEP this exact format — the CI grep gate matches
    // `ODRL suite scenarios pass [0-9]+`. [OPUS-4.8]
    println!(
        "ODRL suite scenarios pass {} / fail {} / not-implemented {} (floor {})",
        score.pass, score.fail, score.not_implemented, ODRL_SUITE_FLOOR
    );
    if !score.failures.is_empty() {
        for f in &score.failures {
            println!("  - {}", f);
        }
    }

    assert_eq!(
        case_files.len(),
        68,
        "expected 68 ODRL test cases at the pinned revision, got {}",
        case_files.len()
    );
    assert_eq!(
        score.fail, 0,
        "ODRL conformance has genuine (unexplained) failures — these must be 0; \
         a mismatch is only tolerated when it carries a documented unsupported \
         construct (the not-implemented bucket)"
    );
    assert!(
        score.pass >= ODRL_SUITE_FLOOR,
        "ODRL conformance regressed: pass {} < floor {ODRL_SUITE_FLOOR} \
         (fail {}, not-implemented {})",
        score.pass,
        score.fail,
        score.not_implemented
    );
    assert!(
        score.not_implemented <= NOT_IMPLEMENTED_CEILING,
        "the not-implemented bucket GREW ({} > ceiling {NOT_IMPLEMENTED_CEILING}) — \
         a real regression must not hide as not-implemented; investigate before \
         raising the ceiling",
        score.not_implemented
    );
}
