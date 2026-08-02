//! [SONNET-4.6] sq-gg0qq.6 (issue #2746) — the `jeswr/lws-spec` conformance vectors, wired as a
//! data-driven suite whose oracle is the spec's OWN normative N3 rule set, executed in-process by
//! sparq's N3 reasoner.
//!
//! The spec — not this crate — is the contract: `conformance/lws-spec/semantics/access-decision.n3`
//! IS the definition of the strict ODRL access profile's `evaluate-access` decision function, and
//! `access-decision.query.n3` is its decision-extraction query (an identity projection of
//! `ax:permittedBy`). Both are vendored verbatim; the decision is permit iff at least one
//! `ax:permittedBy` justification is derived, deny iff none is (the closed-world absence).
//!
//! The rule set is STRATIFIED and says so — its header declares the strata
//! `K -> M -> N, O -> D` — and its only negation is `log:collectAllIn` over predicates an
//! EARLIER stratum derives. A single-pass closure is therefore NOT a sound driver for it: rules N
//! and O (prohibition / obligation matching) and rule D (the permit) become eligible in the same
//! fixpoint round, so D can fire against a fact set that does not yet carry the prohibition, and
//! a matching prohibition fails OPEN. Measured on the vendored file: a single-pass run of
//! `access-grants/prohibition-denies-despite-permission` derives `ax:prohibitedIn` AND
//! `ax:permittedBy` together. So the rule set is fed to [`sparq_reason::n3::reason_n3_stratified`]
//! in the strata it declares — see [`declared_strata`].
//!
//! Coverage is deliberately partial and is asserted, not asserted-away: see
//! `conformance/lws-spec/README.md` for which of the corpus's ten suites are wired, which are not,
//! and why.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

/// The `jeswr/lws-spec` commit the fixtures under `conformance/lws-spec/` were copied from.
/// Re-vendoring means moving this constant and the files in the same change.
const VENDORED_REV: &str = "ffaea0497de41cd709a742e0c4a90831a500fd97";

/// The one corpus suite whose vectors this crate reproduces today.
const WIRED_SUITE: &str = "access-grants";
/// The one vector operation the N3 oracle answers.
const WIRED_OPERATION: &str = "evaluate-access";

/// Corpus-level facts of the pinned revision, pinned so a re-vendor cannot silently shrink the
/// inventory this suite reports itself against.
const CORPUS_CASES: u64 = 157;
const CORPUS_SUITES: usize = 10;
/// Of `access-grants`' 24 cases: 19 are `evaluate-access` (executed below) and 5 are
/// `validate-access-document` (a SHACL-shape operation, not wired — see the README).
const WIRED_SUITE_CASES: usize = 24;
const EXECUTED_CASES: usize = 19;

fn spec_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("conformance").join("lws-spec")
}

fn read_json(path: &Path) -> Value {
    let raw = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("vendored fixture {} is unreadable: {}", path.display(), e));
    serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("vendored fixture {} is not JSON: {}", path.display(), e))
}

// ---------------------------------------------------------------------------------------------
// Inventory — what the pinned corpus contains, and how much of it this crate answers.
// ---------------------------------------------------------------------------------------------

#[test]
fn corpus_manifest_pins_the_full_vector_inventory() {
    let manifest = read_json(&spec_dir().join("vectors").join("manifest.json"));

    assert_eq!(
        manifest["caseCount"].as_u64(),
        Some(CORPUS_CASES),
        "lws-spec@{} declares a different corpus size than this suite reports against",
        VENDORED_REV
    );
    let suites = manifest["suites"].as_array().expect("the corpus manifest must list its suites");
    assert_eq!(suites.len(), CORPUS_SUITES, "corpus suite count moved");
    assert_eq!(
        manifest["suiteCount"].as_u64(),
        Some(CORPUS_SUITES as u64),
        "the manifest's declared suiteCount disagrees with its own suite list"
    );

    let summed: u64 = suites.iter().map(|s| s["caseCount"].as_u64().unwrap_or(0)).sum();
    assert_eq!(summed, CORPUS_CASES, "the per-suite case counts do not add up to the corpus total");

    let wired = suites
        .iter()
        .find(|s| s["suite"].as_str() == Some(WIRED_SUITE))
        .unwrap_or_else(|| panic!("the corpus no longer carries the {} suite", WIRED_SUITE));
    assert_eq!(
        wired["caseCount"].as_u64(),
        Some(WIRED_SUITE_CASES as u64),
        "the {} suite changed size; re-derive this suite's coverage before moving the constant",
        WIRED_SUITE
    );
}

#[test]
fn wired_suite_manifest_matches_the_vendored_cases() {
    let cases = wired_suite_cases();
    assert_eq!(cases.len(), WIRED_SUITE_CASES, "vendored case count differs from the manifest");

    let mut by_operation: BTreeMap<String, usize> = BTreeMap::new();
    for (id, case) in &cases {
        let op = case["operation"]
            .as_str()
            .unwrap_or_else(|| panic!("vector {} has no `operation`", id))
            .to_string();
        *by_operation.entry(op).or_default() += 1;
    }

    assert_eq!(
        by_operation.get(WIRED_OPERATION).copied(),
        Some(EXECUTED_CASES),
        "the {} suite's {} population moved (operations: {:?}); the oracle below asserts the same \
         number, so update both together",
        WIRED_SUITE,
        WIRED_OPERATION,
        by_operation
    );
}

/// `(vector id, case)` for every case the wired suite's manifest lists, read through the manifest
/// rather than by globbing so a case the manifest forgets is a failure, not a silent skip.
fn wired_suite_cases() -> Vec<(String, Value)> {
    let suite_dir = spec_dir().join("vectors").join(WIRED_SUITE);
    let manifest = read_json(&suite_dir.join("manifest.json"));
    assert_eq!(manifest["suite"].as_str(), Some(WIRED_SUITE), "vendored suite manifest mismatch");

    manifest["cases"]
        .as_array()
        .expect("a suite manifest must list its cases")
        .iter()
        .map(|entry| {
            let id = entry["id"].as_str().expect("a manifest case entry must carry an id");
            let path = entry["path"].as_str().expect("a manifest case entry must carry a path");
            let case = read_json(&suite_dir.join(path));
            assert_eq!(case["id"].as_str(), Some(id), "case.json id disagrees with the manifest");
            (id.to_string(), case)
        })
        .collect()
}

// ---------------------------------------------------------------------------------------------
// The oracle — the spec's own N3, run by sparq's own reasoner.
// ---------------------------------------------------------------------------------------------

/// The IRI whose derivation IS the permit decision. Taken from the rule set's `ax:` prefix and
/// cross-checked against the vendored decision-extraction query below, so the decision predicate
/// is the spec's, not this harness's.
const PERMITTED_BY: &str = "https://w3id.org/jeswr/lws/authz#permittedBy";

/// The section banners that open each stratum AFTER the first, in the order the rule set's header
/// declares them (`K -> M -> N, O -> D`). Splitting on the banners leaves the vendored bytes
/// untouched; [`strata_split_matches_the_rule_sets_declared_stratification`] fails loudly if a
/// re-vendor reshapes the file so that this split stops describing it.
const STRATUM_BANNERS: [&str; 3] =
    ["## M. Rule matching", "## N. Prohibition composition", "## D. The permit derivation"];

/// The rule set, cut into the strata it declares. Stratum 0 carries the `@prefix` block; every
/// later stratum is prefixed with a copy of it so it parses standalone.
fn declared_strata(rules: &str) -> Result<Vec<String>, String> {
    let mut cuts = Vec::with_capacity(STRATUM_BANNERS.len());
    for banner in STRATUM_BANNERS {
        let at = rules
            .find(banner)
            .ok_or_else(|| format!("the rule set no longer carries the section {:?}", banner))?;
        if rules[at + banner.len()..].contains(banner) {
            return Err(format!("the section banner {:?} is ambiguous", banner));
        }
        // Cut at the comment rule that opens the section, not mid-banner.
        let start = rules[..at]
            .rfind("## ---")
            .ok_or_else(|| format!("the section {:?} has no opening banner", banner))?;
        if cuts.last().is_some_and(|prev| *prev >= start) {
            return Err("the rule set's sections are no longer in stratum order".to_string());
        }
        cuts.push(start);
    }

    let prefixes: String = rules[..cuts[0]]
        .lines()
        .filter(|l| l.starts_with("@prefix"))
        .collect::<Vec<_>>()
        .join("\n");
    if prefixes.is_empty() {
        return Err("the rule set's `@prefix` block is not where the split expects it".to_string());
    }

    let mut strata = vec![rules[..cuts[0]].to_string()];
    for (i, start) in cuts.iter().enumerate() {
        let end = cuts.get(i + 1).copied().unwrap_or(rules.len());
        strata.push(format!("{}\n{}", prefixes, &rules[*start..end]));
    }
    Ok(strata)
}

/// Whole-line N3 comments dropped. Deliberately not a general comment stripper: an inline `#`
/// would also open a comment, but it would equally be inside an IRI (`jlws:` resolves to a `#`
/// fragment), and this is only used for the structural assertions below.
fn without_comments(n3: &str) -> String {
    n3.lines().filter(|l| !l.trim_start().starts_with('#')).collect::<Vec<_>>().join("\n")
}

/// The vendored rule set plus its decision query, read once per run.
fn semantics() -> (Vec<String>, String) {
    let dir = spec_dir().join("semantics");
    let rules = fs::read_to_string(dir.join("access-decision.n3"))
        .expect("the vendored access-decision.n3 must be readable");
    let query = fs::read_to_string(dir.join("access-decision.query.n3"))
        .expect("the vendored access-decision.query.n3 must be readable");
    (declared_strata(&rules).expect("the rule set must split into its declared strata"), query)
}

/// Run the normative decision function over one encoded input.
///
/// Returns `true` for permit (at least one `ax:permittedBy` justification is derived) and `false`
/// for deny (none is — the decision-time closed-world absence).
fn evaluate_access(strata: &[String], encoded_input: &str) -> Result<bool, String> {
    let mut documents: Vec<&str> = strata.iter().map(String::as_str).collect();
    // The encoded request + recorded grants join the FIRST stratum; every later stratum reads them
    // through the carried closure.
    let first = format!("{}\n{}", strata[0], encoded_input);
    documents[0] = &first;

    let mut dict = sparq_core::dict::Dict::default();
    let closure = sparq_reason::n3::reason_n3_stratified(&mut dict, &documents)?;
    // Interning (rather than looking up) is deliberate: an input that derives no permit at all
    // never puts the predicate in the dictionary, and a fresh id matches no fact — deny.
    let permitted_by = dict.intern_iri(PERMITTED_BY);
    Ok(closure.facts.iter().any(|t| t[1] == permitted_by))
}

#[test]
fn strata_split_matches_the_rule_sets_declared_stratification() {
    let (strata, query) = semantics();
    assert_eq!(strata.len(), STRATUM_BANNERS.len() + 1, "unexpected stratum count");

    // Over the RULES only — the file's prose header names every predicate, so a text check that
    // counted comments would always pass.
    let rules: Vec<String> = strata.iter().map(|s| without_comments(s)).collect();

    // The decision predicate is derived in the LAST stratum and nowhere before it — the property
    // that makes rule D's negation of N/O sound under a stratified driver.
    for (i, stratum) in rules.iter().enumerate() {
        assert_eq!(
            stratum.contains("ax:permittedBy"),
            i + 1 == rules.len(),
            "stratum {} disagrees with the declared stratification over `ax:permittedBy`",
            i
        );
    }
    // ... and the predicates it negates are derived strictly earlier (stratum N, O).
    for predicate in ["ax:prohibitedIn", "ax:obligationUnmetIn"] {
        assert!(rules[2].contains(predicate), "stratum 2 no longer derives {}", predicate);
        assert!(rules[3].contains(predicate), "stratum 3 no longer negates {}", predicate);
        assert!(
            !rules[0].contains(predicate) && !rules[1].contains(predicate),
            "{} escaped into a stratum before the one that derives it",
            predicate
        );
    }

    // The spec's decision-extraction query is an identity projection of the same predicate; this
    // is what licenses reading the decision off the closure instead of running the query.
    assert!(
        query.contains("ax:permittedBy"),
        "the vendored decision query no longer projects the permit predicate"
    );
}

#[test]
fn access_decision_vectors_reproduce_the_spec_verdicts() {
    let (strata, _query) = semantics();
    let mut executed = 0usize;
    let mut failures: Vec<String> = Vec::new();

    for (id, case) in wired_suite_cases() {
        if case["operation"].as_str() != Some(WIRED_OPERATION) {
            continue;
        }
        executed += 1;

        let expected = match case["expected"]["decision"].as_str() {
            Some("permit") => true,
            Some("deny") => false,
            other => {
                failures.push(format!(
                    "suite={} vector={} — unusable expected decision {:?}",
                    WIRED_SUITE, id, other
                ));
                continue;
            }
        };

        let encoded = match encode_decision_input(&case["input"]) {
            Ok(n3) => n3,
            Err(e) => {
                failures.push(format!(
                    "suite={} vector={} — encode error: {}",
                    WIRED_SUITE, id, e
                ));
                continue;
            }
        };

        match evaluate_access(&strata, &encoded) {
            Ok(actual) if actual == expected => {}
            Ok(actual) => failures.push(format!(
                "suite={} vector={} — expected {}, oracle derived {} ({})",
                WIRED_SUITE,
                id,
                decision_word(expected),
                decision_word(actual),
                case["title"].as_str().unwrap_or("")
            )),
            Err(e) => failures.push(format!(
                "suite={} vector={} — oracle error: {}",
                WIRED_SUITE, id, e
            )),
        }
    }

    assert_eq!(
        executed, EXECUTED_CASES,
        "expected {} {} vectors from lws-spec@{}, ran {}",
        EXECUTED_CASES, WIRED_OPERATION, VENDORED_REV, executed
    );
    assert!(
        failures.is_empty(),
        "{} of {} lws-spec@{} {} vectors do not reproduce:\n  {}",
        failures.len(),
        executed,
        VENDORED_REV,
        WIRED_OPERATION,
        failures.join("\n  ")
    );
}

fn decision_word(permit: bool) -> &'static str {
    if permit {
        "permit"
    } else {
        "deny"
    }
}

// ---------------------------------------------------------------------------------------------
// The input encoding (semantics/README.md § "Input encoding").
//
// Security invariant: this mapping emits triples ONLY for the fields the spec's table names. The
// profile facts — `ax:KnownAction` membership and the `odrl:includedIn` lattice — live in the rule
// set and are never read from an evaluated document, so a hostile grant cannot inject widening.
// Everything unrecognised is an encode error (fail loud, never silently dropped).
// ---------------------------------------------------------------------------------------------

const ODRL: &str = "http://www.w3.org/ns/odrl/2/";
const JLWS: &str = "https://w3id.org/jeswr/lws#";

const PREFIXES: &str = "\
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
@prefix jlws: <https://w3id.org/jeswr/lws#> .
@prefix ax:   <https://w3id.org/jeswr/lws/authz#> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
";

/// ODRL 2.2 operator names. Anything outside this set (and outside an absolute `http(s)` IRI) is an
/// encode error; the rule set separately fails closed on any pair it cannot evaluate (rule K4).
const ODRL_OPERATORS: [&str; 12] = [
    "eq", "neq", "lt", "lteq", "gt", "gteq", "isA", "isAllOf", "isAnyOf", "isNoneOf", "isPartOf",
    "hasPart",
];

fn encode_decision_input(input: &Value) -> Result<String, String> {
    let mut n3 = String::from(PREFIXES);

    let grants = input["grants"].as_array().ok_or("vector input carries no `grants` array")?;
    for grant in grants {
        encode_grant(grant, &mut n3)?;
    }
    encode_request(&input["request"], &mut n3)?;
    Ok(n3)
}

fn encode_grant(grant: &Value, out: &mut String) -> Result<(), String> {
    // A record with a missing or unknown `@type` is NEVER fabricated into an `odrl:Offer`.
    match grant["@type"].as_str() {
        Some("Offer") => {}
        other => return Err(format!("unrecognised grant @type {:?}", other)),
    }
    let uid = absolute_iri(&grant["uid"]).ok_or("grant has no absolute `uid` IRI")?;
    let profile = absolute_iri(&grant["profile"]).ok_or("grant has no absolute `profile` IRI")?;

    write!(out, "<{}> a odrl:Offer ;\n  odrl:profile <{}> ;\n  ax:recordedIn ax:GrantStore", uid, profile)
        .expect("writing to a String cannot fail");

    for (json_key, predicate) in
        [("permission", "odrl:permission"), ("prohibition", "odrl:prohibition"), ("obligation", "odrl:obligation")]
    {
        let Some(rules) = grant.get(json_key) else { continue };
        let rules = rules
            .as_array()
            .ok_or_else(|| format!("grant `{}` is not an array", json_key))?;
        for rule in rules {
            write!(out, " ;\n  {} {}", predicate, encode_rule(rule)?)
                .expect("writing to a String cannot fail");
        }
    }
    out.push_str(" .\n\n");
    Ok(())
}

/// A permission, prohibition and obligation rule carry the identical shape (rule M of the rule
/// set matches all three the same way), so one encoder serves all three.
fn encode_rule(rule: &Value) -> Result<String, String> {
    let assignee = absolute_iri(&rule["assignee"]).ok_or("rule has no absolute `assignee` IRI")?;
    let action = action_iri(rule["action"].as_str().ok_or("rule has no `action`")?)?;

    let mut enc = format!("[\n    odrl:assignee <{}> ;\n    odrl:action <{}> ;\n    odrl:target {}", assignee, action, encode_target(&rule["target"])?);

    if let Some(constraints) = rule.get("constraint") {
        let constraints =
            constraints.as_array().ok_or("rule `constraint` is not an array")?;
        let encoded: Result<Vec<String>, String> =
            constraints.iter().map(encode_constraint).collect();
        let encoded = encoded?;
        if !encoded.is_empty() {
            write!(enc, " ;\n    odrl:constraint {}", encoded.join(" , "))
                .expect("writing to a String cannot fail");
        }
    }
    enc.push_str("\n  ]");
    Ok(enc)
}

fn encode_target(target: &Value) -> Result<String, String> {
    let ty = match target["@type"].as_str() {
        Some(t @ ("DataResource" | "Container" | "StorageResource")) => t,
        other => return Err(format!("unrecognised target @type {:?}", other)),
    };
    let uid = absolute_iri(&target["uid"]).ok_or("target has no absolute `uid` IRI")?;

    let mut enc = format!("[ a jlws:{} ; odrl:uid <{}>", ty, uid);
    if target["recursive"] == Value::Bool(true) {
        if ty != "Container" {
            return Err(format!("`recursive` is a Container-only target field, found on {}", ty));
        }
        enc.push_str(" ; jlws:recursive true");
    }
    enc.push_str(" ]");
    Ok(enc)
}

fn encode_constraint(constraint: &Value) -> Result<String, String> {
    // A constraint missing any of the three parts is malformed; the rule set fails it closed
    // (K5-K7), so it is encoded as-is rather than rejected here.
    let mut parts: Vec<String> = Vec::new();
    if let Some(left) = constraint.get("leftOperand") {
        let left = left_operand_iri(left.as_str().ok_or("`leftOperand` is not a string")?)?;
        parts.push(format!("odrl:leftOperand <{}>", left));
    }
    if let Some(operator) = constraint.get("operator") {
        let operator = operator_iri(operator.as_str().ok_or("`operator` is not a string")?)?;
        parts.push(format!("odrl:operator <{}>", operator));
    }
    if let Some(right) = constraint.get("rightOperand") {
        let is_date_time =
            constraint["leftOperand"].as_str() == Some("dateTime") || constraint["leftOperand"].as_str() == Some("http://www.w3.org/ns/odrl/2/dateTime");
        parts.push(format!("odrl:rightOperand {}", encode_right_operand(right, is_date_time)?));
    }
    Ok(format!("[ {} ]", parts.join(" ; ")))
}

/// `{ "@value": v, "@type": t }` and bare strings both land as the plain literal `"v"` — the
/// canonical lexical form the rule set compares lexicographically; an absolute `http(s)` IRI
/// string lands as an IRI.
///
/// Calendar validity of a `dateTime` bound is deliberately NOT re-checked here: the rule set
/// itself derives `ax:unsatisfiedFor` on any non-canonical or nonexistent instant (K3b/K3c), which
/// is the spec's own defence-in-depth for embedders feeding the reasoner unvalidated data — so
/// delegating keeps this encoder from being a second, drifting copy of that predicate. A
/// FOREIGN-datatyped bound is still rejected, because that one the rules cannot see.
fn encode_right_operand(right: &Value, is_date_time: bool) -> Result<String, String> {
    let (lexical, datatype) = match right {
        Value::String(s) => (s.as_str(), None),
        Value::Object(_) => (
            right["@value"].as_str().ok_or("`rightOperand` object has no string `@value`")?,
            right["@type"].as_str(),
        ),
        other => return Err(format!("unusable `rightOperand` {}", other)),
    };

    if is_date_time {
        match datatype {
            None | Some("xsd:dateTime") | Some("http://www.w3.org/2001/XMLSchema#dateTime") => {}
            Some(other) => {
                return Err(format!("a dateTime bound must be xsd:dateTime-typed, found {:?}", other))
            }
        }
        return Ok(format!("\"{}\"", escape_literal(lexical)));
    }

    match absolute_iri(right) {
        Some(iri) if datatype.is_none() => Ok(format!("<{}>", iri)),
        _ => Ok(format!("\"{}\"", escape_literal(lexical))),
    }
}

fn encode_request(request: &Value, out: &mut String) -> Result<(), String> {
    let agent = absolute_iri(&request["agent"]).ok_or("request has no absolute `agent` IRI")?;
    let action = action_iri(request["action"].as_str().ok_or("request has no `action`")?)?;
    let target = absolute_iri(&request["target"]).ok_or("request has no absolute `target` IRI")?;

    // The profile's left-operand IRIs ARE the context keys.
    let mut context: Vec<String> = Vec::new();
    if let Some(entries) = request.get("context") {
        let entries = entries.as_object().ok_or("request `context` is not an object")?;
        for (key, value) in entries {
            let left = left_operand_iri(key)?;
            let is_date_time = left == format!("{}dateTime", ODRL);
            context.push(format!("<{}> {}", left, encode_right_operand(value, is_date_time)?));
        }
    }

    write!(
        out,
        "[] a ax:Request ;\n  ax:agent <{}> ;\n  ax:action <{}> ;\n  ax:target <{}> ;\n  ax:context [ {} ] .\n",
        agent,
        action,
        target,
        context.join(" ; ")
    )
    .expect("writing to a String cannot fail");
    Ok(())
}

fn action_iri(name: &str) -> Result<String, String> {
    match name {
        "read" | "modify" | "delete" => Ok(format!("{}{}", ODRL, name)),
        "create" | "append" => Ok(format!("{}{}", JLWS, name)),
        _ => absolute_iri(&Value::String(name.to_string()))
            .ok_or_else(|| format!("unrecognised action {:?}", name)),
    }
}

fn left_operand_iri(name: &str) -> Result<String, String> {
    match name {
        "purpose" | "dateTime" => Ok(format!("{}{}", ODRL, name)),
        "client" | "mediaType" | "resourceType" => Ok(format!("{}{}", JLWS, name)),
        _ => absolute_iri(&Value::String(name.to_string()))
            .ok_or_else(|| format!("unrecognised left operand / context key {:?}", name)),
    }
}

fn operator_iri(name: &str) -> Result<String, String> {
    if ODRL_OPERATORS.contains(&name) {
        return Ok(format!("{}{}", ODRL, name));
    }
    absolute_iri(&Value::String(name.to_string()))
        .ok_or_else(|| format!("unrecognised operator {:?}", name))
}

/// An absolute `http(s)` IRI passes through everywhere. The `<`/`>`/whitespace rejection keeps a
/// hostile fixture from closing the IRI and injecting further triples.
fn absolute_iri(value: &Value) -> Option<String> {
    let s = value.as_str()?;
    let absolute = s.starts_with("http://") || s.starts_with("https://");
    let clean = !s.contains(['<', '>', '"', '{', '}', '|', '^', '`', '\\'])
        && !s.chars().any(char::is_whitespace);
    (absolute && clean).then(|| s.to_string())
}

fn escape_literal(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r")
}

// ---------------------------------------------------------------------------------------------
// Direct unit coverage of the encoder's fail-loud edges — the vectors above exercise the happy
// paths, but "never silently dropped" is a security property of the mapping and needs its own
// witness.
// ---------------------------------------------------------------------------------------------

#[test]
fn encoder_refuses_to_fabricate_an_offer_from_an_unknown_record_type() {
    let grant = serde_json::json!({
        "@type": "Agreement",
        "uid": "https://storage.example/alice/.grants/x",
        "profile": "https://w3id.org/jeswr/lws/access-profile/odrl-1",
    });
    let err = encode_grant(&grant, &mut String::new()).unwrap_err();
    assert!(err.contains("unrecognised grant @type"), "{}", err);

    let mut missing = serde_json::json!({ "uid": "https://storage.example/alice/.grants/x" });
    missing["profile"] = Value::String("https://w3id.org/jeswr/lws/access-profile/odrl-1".into());
    assert!(encode_grant(&missing, &mut String::new()).is_err());
}

#[test]
fn encoder_rejects_relative_and_iri_closing_terms() {
    assert!(absolute_iri(&Value::String("/alice/notes/a.txt".into())).is_none());
    assert!(absolute_iri(&Value::String("urn:uuid:1234".into())).is_none());
    // The injection shape: an IRI that closes itself and asserts a profile fact.
    assert!(absolute_iri(&Value::String(
        "https://e.example/a> . jlws:create odrl:includedIn <https://e.example/b".into()
    ))
    .is_none());
}

#[test]
fn encoder_rejects_unrecognised_profile_terms() {
    assert!(action_iri("administer").is_err());
    assert!(action_iri("https://extension.example/administer").is_ok());
    assert!(left_operand_iri("elevation").is_err());
    assert!(operator_iri("approximatelyEq").is_err());
    assert!(operator_iri("lt").is_ok());
}

#[test]
fn encoder_rejects_a_foreign_datatyped_datetime_bound() {
    let bound = serde_json::json!({ "@type": "xsd:string", "@value": "2026-06-01T00:00:00Z" });
    assert!(encode_right_operand(&bound, true).is_err());
    let ok = serde_json::json!({ "@type": "xsd:dateTime", "@value": "2026-06-01T00:00:00Z" });
    assert_eq!(encode_right_operand(&ok, true).unwrap(), "\"2026-06-01T00:00:00Z\"");
}

#[test]
fn encoder_rejects_recursive_on_a_non_container_target() {
    let target = serde_json::json!({
        "@type": "DataResource",
        "uid": "https://storage.example/alice/notes/a.txt",
        "recursive": true,
    });
    assert!(encode_target(&target).unwrap_err().contains("Container-only"));
}
