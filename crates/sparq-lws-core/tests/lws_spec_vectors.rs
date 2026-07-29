// AUTHORED-BY Claude Sonnet 4.6
#![cfg(feature = "access-profile-odrl1")]
//! sq-gg0qq.6 — the crate's LWS conformance gate: the vendored `jeswr/lws-spec` test-vectors,
//! wired as a data-driven suite.
//!
//! The contract is [`jeswr/lws-spec`](https://github.com/jeswr/lws-spec), vendored at a pinned
//! commit under `lws-spec/` (provenance + the vendored-not-submodule decision are in
//! `lws-spec/README.md`). **Where the crate and the spec disagree, the SPEC WINS** — a failing
//! vector is fixed in `src/`, or challenged upstream on `lws-spec`. Editing a vendored vector to
//! make this suite green is a spec change made in the wrong repository.
//!
//! Two things are asserted, and the second is the one that keeps the first honest:
//!
//! 1. **Verdict reproduction.** Every case whose `operation` this crate implements is run and its
//!    derived verdict diffed against the vector's `expected`. A divergence names the suite (the
//!    document class) and the vector id.
//! 2. **The coverage ledger.** Every case lands in exactly one bucket of
//!    `lws-spec/coverage-baseline.json` — `evaluated` or `pending` — and the suite fails if the
//!    per-operation tallies drift in EITHER direction. So an unimplemented operation is counted
//!    rather than silently skipped, and growing coverage forces a visible baseline edit.
//!
//! Today only `evaluate-access` (19 cases, the `access-grants` class) is reproduced, by
//! [`sparq_lws_core::authz::access_profile`]. Re-deriving those verdicts from the normative N3 rule
//! set under an N3 reasoner needs Node + EYE and stays an opt-in lane (`lws-spec/README.md`).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use sparq_lws_core::authz::access_profile::evaluate_access_json;

/// The one operation this crate reproduces today. Kept as a list so adding a class is a one-line
/// change here plus a baseline move.
const EVALUATED_OPERATIONS: [&str; 1] = ["evaluate-access"];

/// The spec revision the vendored corpus was generated against, as recorded in the vendored
/// `manifest.json`. Pinned here so a corpus swap cannot pass unnoticed.
const EXPECTED_SPEC_SOURCE: &str = "lws-spec@59da847";

fn vendored_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("lws-spec")
}

fn read_json(path: &Path) -> Value {
    let bytes = fs::read(path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    serde_json::from_slice(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", path.display(), e))
}

/// One vendored case, with the suite it belongs to (the spec's document class).
struct Case {
    suite: String,
    id: String,
    operation: String,
    body: Value,
}

/// Walk the vendored manifests and load every case, asserting the manifests agree with each other
/// and with what is actually on disk (a truncated vendoring must not read as a passing suite).
fn load_cases() -> Vec<Case> {
    let root = vendored_root().join("test-vectors");
    let top = read_json(&root.join("manifest.json"));

    assert_eq!(
        top["specSource"].as_str(),
        Some(EXPECTED_SPEC_SOURCE),
        "vendored manifest specSource changed — refresh the pin in lws-spec/README.md and this test"
    );
    let declared_cases = top["caseCount"].as_u64().expect("manifest caseCount");
    let suites = top["suites"].as_array().expect("manifest suites");
    assert_eq!(
        suites.len() as u64,
        top["suiteCount"].as_u64().expect("manifest suiteCount"),
        "manifest suiteCount disagrees with the suites it lists"
    );

    let mut cases = Vec::new();
    for suite in suites {
        let name = suite["suite"].as_str().expect("suite name").to_owned();
        let manifest_path = root.join(suite["path"].as_str().expect("suite path"));
        let manifest = read_json(&manifest_path);
        let listed = manifest["cases"].as_array().expect("suite cases");
        // Three independently-authored counts of the same suite must agree.
        assert_eq!(
            listed.len() as u64,
            suite["caseCount"].as_u64().expect("suite caseCount"),
            "{}: top-level caseCount disagrees with the suite manifest",
            name
        );
        assert_eq!(
            listed.len() as u64,
            manifest["caseCount"].as_u64().expect("suite caseCount"),
            "{}: suite manifest caseCount disagrees with the cases it lists",
            name
        );
        let suite_dir = manifest_path.parent().expect("suite manifest has a parent");
        for entry in listed {
            let id = entry["id"].as_str().expect("case id").to_owned();
            let body = read_json(&suite_dir.join(entry["path"].as_str().expect("case path")));
            assert_eq!(
                body["id"].as_str(),
                Some(id.as_str()),
                "{}: case file id disagrees with the manifest entry",
                id
            );
            let operation = body["operation"]
                .as_str()
                .unwrap_or_else(|| panic!("{}: case has no operation", id))
                .to_owned();
            cases.push(Case {
                suite: name.clone(),
                id,
                operation,
                body,
            });
        }
    }
    assert_eq!(
        cases.len() as u64,
        declared_cases,
        "loaded {} cases but the manifest declares {}",
        cases.len(),
        declared_cases
    );
    cases
}

/// Reproduce every vector whose operation this crate implements. A divergence is reported as
/// `suite/vector-id`, with the derived and expected verdicts, so a failure says WHICH vector and
/// WHICH class broke rather than just "conformance failed".
#[test]
fn evaluated_vectors_reproduce_the_spec_verdicts() {
    let cases = load_cases();
    let mut failures = Vec::new();
    let mut reproduced = 0usize;

    for case in &cases {
        if case.operation != "evaluate-access" {
            continue;
        }
        let expected = case.expected_decision();
        let derived = match evaluate_access_json(&case.body["input"]) {
            Ok(decision) => decision.as_str().to_owned(),
            // A decode error is a divergence, not a deny: the vectors are all inside the profile's
            // document shape, so failing to read one means the port's decoder is wrong.
            Err(error) => format!("ERROR ({})", error),
        };
        if derived == expected {
            reproduced += 1;
        } else {
            failures.push(format!(
                "  {} [class {}] derived={} expected={}",
                case.id, case.suite, derived, expected
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "{} of {} evaluate-access vectors diverge from lws-spec:\n{}\n\
         The SPEC WINS: fix crates/sparq-lws-core/src/authz/access_profile.rs, or raise a change \
         proposal on jeswr/lws-spec. Do NOT edit a vendored vector.",
        failures.len(),
        reproduced + failures.len(),
        failures.join("\n")
    );
    assert!(
        reproduced > 0,
        "no evaluate-access vectors ran — the vendored corpus is missing or mis-filtered"
    );
}

/// The coverage ledger: every vendored case is either evaluated or explicitly pending, and the
/// tallies match `lws-spec/coverage-baseline.json` EXACTLY. This is what makes the suite above
/// meaningful — without it, deleting the dispatch would still leave a green "no failures" run.
#[test]
fn coverage_ledger_matches_the_pinned_baseline() {
    let cases = load_cases();
    let baseline = read_json(&vendored_root().join("coverage-baseline.json"));

    let mut evaluated: BTreeMap<String, u64> = BTreeMap::new();
    let mut pending: BTreeMap<String, u64> = BTreeMap::new();
    for case in &cases {
        let bucket = if EVALUATED_OPERATIONS.contains(&case.operation.as_str()) {
            &mut evaluated
        } else {
            &mut pending
        };
        *bucket.entry(case.operation.clone()).or_default() += 1;
    }

    assert_eq!(
        cases.len() as u64,
        baseline["caseCount"].as_u64().expect("baseline caseCount"),
        "the vendored corpus size changed — update lws-spec/coverage-baseline.json"
    );
    assert_eq!(
        evaluated,
        tally(&baseline["evaluated"]),
        "the EVALUATED coverage ledger drifted. If this is new coverage, move the operation from \
         `pending` to `evaluated` in lws-spec/coverage-baseline.json in the same change."
    );
    assert_eq!(
        pending,
        tally(&baseline["pending"]),
        "the PENDING coverage ledger drifted. Either coverage grew (move the operation to \
         `evaluated`) or the vendored pin was refreshed (re-derive both buckets)."
    );
    // Both buckets are disjoint by construction; assert they also account for everything.
    let ledgered: u64 = evaluated.values().chain(pending.values()).sum();
    assert_eq!(
        ledgered,
        cases.len() as u64,
        "the ledger accounts for {} cases but {} were loaded",
        ledgered,
        cases.len()
    );
}

/// The normative rule set this crate's port cites must actually be vendored alongside the vectors —
/// it is the authority a reviewer checks the port against, and `src/authz/access_profile.rs` points
/// at it by path.
#[test]
fn the_normative_rule_set_is_vendored() {
    for name in ["access-decision.n3", "access-decision.query.n3"] {
        let path = vendored_root().join("semantics").join(name);
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
        assert!(
            text.contains("ax:permittedBy"),
            "{} does not derive/project ax:permittedBy — wrong or truncated vendoring",
            path.display()
        );
    }
    // The permit derivation keys on the exact profile IRI the Rust port checks for.
    let rules = fs::read_to_string(vendored_root().join("semantics/access-decision.n3"))
        .expect("rule set readable");
    assert!(
        rules.contains(sparq_lws_core::authz::access_profile::PROFILE_ODRL1),
        "the vendored rule set does not mention the profile IRI the port keys on"
    );
}

impl Case {
    /// The vector's expected verdict. Absent is a fault in the vendored case, not a pass.
    fn expected_decision(&self) -> String {
        self.body["expected"]["decision"]
            .as_str()
            .unwrap_or_else(|| panic!("{}: case has no expected.decision", self.id))
            .to_owned()
    }
}

fn tally(value: &Value) -> BTreeMap<String, u64> {
    value
        .as_object()
        .expect("baseline bucket is an object")
        .iter()
        .map(|(operation, count)| {
            (
                operation.clone(),
                count.as_u64().unwrap_or_else(|| {
                    panic!("baseline count for {} is not a number", operation)
                }),
            )
        })
        .collect()
}
