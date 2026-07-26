//! The canned-query helper must keep loading the PKG and returning the expected rows —
//! a rot-guard for the `pkg-query` skill (sq-2m6zm.3). The answers are computed by
//! sparq's own SPARQL engine over the ingested graph; the engine bounds the result set,
//! so within the store the answer cannot be fabricated.
//!
//! [OPUS-4.8] sq-2m6zm.3 (epic sq-2m6zm). 🤖 SPARQ agent — query-the-PKG skill PoC.
//!
//! Run: `cargo test -p sparq-kb --features query --test query_canned -- --nocapture`
//! (and, to exercise the optional closure path:
//!  `cargo test -p sparq-kb --features close --test query_canned -- --nocapture`).
#![cfg(feature = "query")]

use oxrdf::Term;
use sparq_engine::QueryResult;
use sparq_kb::query::canned::{self, CannedQuery};
use sparq_kb::query::{ask_pkg, load_pkg, load_pkg_with_extra};

/// Render + run a canned query (with an optional argument) over the loaded PKG.
fn run(q: &CannedQuery, arg: Option<&str>) -> QueryResult {
    let g = load_pkg().expect("PKG loads");
    let sparql = q.render(arg);
    ask_pkg(&g, &sparql).unwrap_or_else(|e| panic!("`{}` query runs: {e}", q.name))
}

/// The literal string value of cell `col` in `row`, lower-cased (empty if unbound/IRI).
fn lit(r: &QueryResult, row: usize, col: usize) -> String {
    match r.rows.get(row).and_then(|x| x.get(col)) {
        Some(Some(Term::Literal(l))) => l.value().to_lowercase(),
        _ => String::new(),
    }
}

/// The local name of an IRI cell, or the literal value, lower-cased.
fn term_str(t: &Option<Term>) -> String {
    match t {
        Some(Term::NamedNode(n)) => n
            .as_str()
            .rsplit(['#', '/'])
            .next()
            .unwrap_or("")
            .to_lowercase(),
        Some(Term::Literal(l)) => l.value().to_lowercase(),
        _ => String::new(),
    }
}

#[test]
fn registry_is_complete_and_named() {
    // Every canned query is reachable by name, and the parameterised ones have a
    // working default that round-trips through `render`.
    assert!(
        !canned::ALL.is_empty(),
        "the canned registry must not be empty"
    );
    for q in canned::ALL {
        assert_eq!(canned::by_name(q.name).map(|x| x.name), Some(q.name));
        let rendered = q.render(None);
        assert!(
            !rendered.contains("{ARG}"),
            "`{}` still has an unsubstituted {{ARG}} after rendering its default",
            q.name
        );
    }
    // The §6 worked queries the skill documents must all be present.
    for name in [
        "schema-classes",
        "schema-properties",
        "findings-about",
        "finding-provenance",
        "finding-quality-dqv",
        "unexplored-sources",
        "task-depends-on",
        "task-blocks",
        "high-followup-priority",
        "ready-frontier",
    ] {
        assert!(
            canned::by_name(name).is_some(),
            "canned query `{name}` is missing"
        );
    }
}

#[test]
fn introspect_schema_classes() {
    // The schema card must surface the PKG classes actually present, with Task the
    // largest (the bd projection dominates the ingest).
    let r = run(&canned::SCHEMA_CLASSES, None);
    let classes: Vec<String> = r.rows.iter().map(|row| term_str(&row[0])).collect();
    for expect in ["task", "finding", "source", "technique"] {
        assert!(
            classes.contains(&expect.to_string()),
            "schema card missing pkg:{expect}; got {classes:?}"
        );
    }
    assert_eq!(
        classes.first().map(String::as_str),
        Some("task"),
        "Task must be the largest class"
    );
}

#[test]
fn ground_findings_about_merge_discipline() {
    // The merge-discipline topic must surface the ci-summary base gate, each with a
    // section anchor (provenance) and a confidence.
    let topic = "https://sparq.dev/ns/pkg/kb#topic-merge-discipline";
    let r = run(&canned::FINDINGS_ABOUT, Some(topic));
    assert!(
        !r.rows.is_empty(),
        "findings-about returned no rows for the merge-discipline topic"
    );
    let labels: Vec<String> = (0..r.rows.len()).map(|i| lit(&r, i, 0)).collect();
    assert!(
        labels.iter().any(|l| l.contains("ci-summary")),
        "merge-discipline findings must surface the ci-summary gate; got {labels:?}"
    );
    // Every row carries a source section anchor (col 1) — the queryable citation.
    assert!(
        r.rows
            .iter()
            .all(|row| matches!(&row[1], Some(Term::Literal(_)))),
        "every finding must carry its dcterms:source section anchor"
    );
}

#[test]
fn ground_finding_provenance_is_sourced_and_confident() {
    // Every Finding has a derived-from source, an assurance basis, and a confidence.
    let r = run(&canned::FINDING_PROVENANCE, None);
    assert!(
        r.rows.len() >= 6,
        "expected the AGENTS.md finding slice; got {} rows",
        r.rows.len()
    );
    for (i, row) in r.rows.iter().enumerate() {
        assert!(
            matches!(&row[1], Some(Term::NamedNode(_))),
            "row {i}: provenance source must be an IRI"
        );
        let assurance = term_str(&row[3]);
        assert!(
            matches!(assurance.as_str(), "proven" | "claimed" | "conjectured"),
            "row {i}: assurance must be a secx: basis; got `{assurance}`"
        );
        let conf = lit(&r, i, 4).parse::<f64>().unwrap_or(-1.0);
        assert!(
            (0.0..=1.0).contains(&conf),
            "row {i}: confidence must be in 0..1; got {conf}"
        );
    }
}

/// The DQV-modelled quality axis (sq-2489d.3) answers AT LEAST what the `pkg:confidence`
/// shorthand answers. Over a small DQV-bearing fixture (a Finding with both the
/// `pkg:confidence` shorthand AND a reified `dqv:QualityMeasurement`), the
/// `finding-quality-dqv` query surfaces the measurement and its `dqv:value` MATCHES the
/// shorthand — proving the two never contradict and the modelled axis is queryable.
/// The fixture is loaded via the `load_pkg_with_extra` seam so the canned query runs the
/// REAL engine path over real DQV triples, not a mock.
#[test]
fn ground_finding_quality_dqv_agrees_with_confidence_shorthand() {
    const DQV_FIXTURE: &str = r#"
@prefix pkg:     <https://sparq.dev/ns/pkg#> .
@prefix dqv:     <http://www.w3.org/ns/dqv#> .
@prefix sigimpl: <https://w3id.org/zkp-sparql/sig-impl#> .
@prefix secx:    <https://w3id.org/zkp-sparql/sec-prop#> .
@prefix prov:    <http://www.w3.org/ns/prov#> .
@prefix rdfs:    <http://www.w3.org/2000/01/rdf-schema#> .
@prefix exq:     <https://sparq.dev/ns/pkg/dqv-fixture#> .

exq:src a pkg:Source ; rdfs:label "fixture source"@en-GB ; pkg:confidence 0.7 .
exq:find a pkg:Finding ;
  rdfs:label "fixture finding"@en-GB ;
  sigimpl:justification "a sufficiently long non-filler justification for the fixture"@en-GB ;
  pkg:assurance secx:Claimed ;
  prov:wasDerivedFrom exq:src ;
  pkg:confidence 0.42 ;
  dqv:hasQualityMeasurement exq:meas .
exq:meas a dqv:QualityMeasurement ;
  dqv:isMeasurementOf pkg:ConfidenceMeasurement ;
  dqv:computedOn exq:find ;
  dqv:value 0.42 .
"#;

    let g = load_pkg_with_extra(&[DQV_FIXTURE]).expect("PKG + DQV fixture loads");
    let r = ask_pkg(&g, &canned::FINDING_QUALITY_DQV.render(None)).expect("dqv query runs");
    // The measurement is surfaced with its subject + metric + value.
    let mut found = false;
    for row in &r.rows {
        let subject = term_str(&row[0]);
        let metric = term_str(&row[1]);
        let value = match &row[2] {
            Some(Term::Literal(l)) => l.value().to_string(),
            _ => String::new(),
        };
        if subject == "find" {
            assert_eq!(metric, "confidencemeasurement", "metric must be the DQV metric");
            assert_eq!(
                value.parse::<f64>().unwrap_or(-1.0),
                0.42,
                "the dqv:value must MATCH the pkg:confidence shorthand"
            );
            found = true;
        }
    }
    assert!(
        found,
        "the DQV quality query must surface the fixture Finding's measurement; got {} rows",
        r.rows.len()
    );
}

/// The DQV axis is queryable over the **REAL ingest**, not only over a fixture
/// (sq-2489d.7). Phase 3 modelled DQV in `pkg.ttl` and demonstrated it in the example
/// file, but `ingest/pkg-instances.ttl` carried the `pkg:confidence` shorthand alone, so
/// `finding-quality-dqv` matched nothing over the graph agents actually query. The
/// ingest projectors now reify every Finding/Source confidence, so this asserts:
/// non-empty rows, BOTH named metrics present, and every value a well-formed 0..1
/// decimal. No fixture is loaded — this is `load_pkg()`, the shipped graph.
#[test]
fn ground_finding_quality_dqv_is_non_empty_over_the_real_ingest() {
    let r = run(&canned::FINDING_QUALITY_DQV, None);
    assert!(
        !r.rows.is_empty(),
        "the real ingest must carry reified dqv:QualityMeasurements (sq-2489d.7), but \
         finding-quality-dqv returned 0 rows"
    );

    let mut metrics: Vec<String> = Vec::new();
    for (i, row) in r.rows.iter().enumerate() {
        assert!(
            matches!(&row[0], Some(Term::NamedNode(_))),
            "row {i}: the dqv:computedOn subject must be an IRI"
        );
        let value = lit(&r, i, 2).parse::<f64>().unwrap_or(-1.0);
        assert!(
            (0.0..=1.0).contains(&value),
            "row {i}: dqv:value must be in 0..1; got {value}"
        );
        metrics.push(term_str(&row[1]));
    }
    // Both named metrics pkg.ttl declares are exercised: a Finding's epistemic weight and
    // a Source's reliability. (The ingest carries Findings AND Sources with confidence.)
    for metric in ["confidencemeasurement", "sourcereliabilitymeasurement"] {
        assert!(
            metrics.iter().any(|m| m == metric),
            "the real ingest must project pkg:{metric}; got {metrics:?}"
        );
    }
}

/// The DQV projection over the real ingest is COMPLETE and AGREES with the shorthand
/// (sq-2489d.7): NO `pkg:confidence` in the shipped graph is left without a reified
/// measurement, and none disagrees with its measurement's `dqv:value`. This is the
/// assertion that would go red if any one ingest tier (`pkg-instances.ttl`, the compiled
/// agents-findings tier, the trust-graph tier) regressed to the bare shorthand — a
/// per-tier "is it non-empty" check would not, since the other tiers would still answer.
#[test]
fn ground_every_confidence_in_the_real_ingest_has_an_agreeing_dqv_measurement() {
    const UNREIFIED: &str = r#"
PREFIX pkg: <https://sparq.dev/ns/pkg#>
PREFIX dqv: <http://www.w3.org/ns/dqv#>
SELECT ?s ?conf WHERE {
  ?s pkg:confidence ?conf .
  FILTER NOT EXISTS {
    ?m dqv:computedOn ?s ; dqv:value ?conf .
  }
}"#;
    let g = load_pkg().expect("PKG loads");
    let r = ask_pkg(&g, UNREIFIED).expect("completeness query runs");
    let offenders: Vec<String> = r
        .rows
        .iter()
        .map(|row| format!("{} (confidence {})", term_str(&row[0]), term_str(&row[1])))
        .collect();
    assert!(
        offenders.is_empty(),
        "every pkg:confidence in the ingest must be reified as an AGREEING \
         dqv:QualityMeasurement (sq-2489d.7); {} subject(s) are not: {:?}",
        offenders.len(),
        offenders
    );
}

#[test]
fn ground_unexplored_sources_is_the_honest_none_answer() {
    // Over the Phase-1 ingest every source is pkg:Explored, so the targeted-follow-up
    // list is EMPTY — the honest "none outstanding" answer, computed by the engine. This
    // is the negative/out-of-KG-existence stratum the design wants represented.
    let r = run(&canned::UNEXPLORED_SOURCES, None);
    assert!(
        r.rows.is_empty(),
        "every ingested source is pkg:Explored, so unexplored-sources must be empty; got {} rows",
        r.rows.len()
    );
}

#[test]
fn ground_task_blocks_returns_downstream_dependents() {
    // sq-8thu is the most-depended-on blocker in the ingest; finishing it unblocks many.
    let r = run(&canned::TASK_BLOCKS, Some("sq-8thu"));
    assert!(
        r.rows.len() >= 10,
        "sq-8thu should have many downstream dependents; got {} rows",
        r.rows.len()
    );
}

#[test]
fn ground_task_depends_on_returns_dependencies() {
    // The default arg (sq-0po6) depends on two tasks; each comes back with a status.
    let r = run(&canned::TASK_DEPENDS_ON, None);
    assert!(
        !r.rows.is_empty(),
        "task-depends-on default (sq-0po6) should have dependencies"
    );
    let dep_ids: Vec<String> = r.rows.iter().map(|row| term_str(&row[0])).collect();
    assert!(
        dep_ids.iter().any(|d| d == "sq-8thu"),
        "sq-0po6 must depend on sq-8thu; got {dep_ids:?}"
    );
}

#[test]
fn ground_ready_frontier_is_non_empty() {
    // The §4.1 ready-frontier (dependency half) over the real backlog is non-empty.
    let r = run(&canned::READY_FRONTIER, None);
    let n = match r.rows.first().and_then(|row| row.first()) {
        Some(Some(Term::Literal(l))) => l.value().parse::<i64>().unwrap_or(0),
        _ => 0,
    };
    assert!(
        n > 0,
        "the §4.1 ready-frontier must be non-empty over the bd backlog; got {n}"
    );
}

/// The optional closure step must materialise the `pkg:dependsOn owl:inverseOf
/// pkg:blockedBy` pair, so a `pkg:blockedBy` query (the inverse direction, never
/// asserted in the data) returns the same downstream set as the `pkg:dependsOn` query.
/// Only built with `--features close`.
#[cfg(feature = "close")]
#[test]
fn closure_materialises_the_blockedby_inverse() {
    use sparq_kb::query::close::{load_pkg_closed, Profile};

    let (g, entailed) = load_pkg_closed(Profile::OwlRl).expect("closed PKG loads");
    assert!(entailed > 0, "OWL-RL closure must add entailed triples");

    // The inverse direction, asked only via pkg:blockedBy (never asserted as such):
    let inverse = r#"
PREFIX pkg:     <https://sparq.dev/ns/pkg#>
PREFIX dcterms: <http://purl.org/dc/terms/>
SELECT (COUNT(*) AS ?n) WHERE {
  ?blocker dcterms:identifier "sq-8thu" . ?blocker pkg:blockedBy ?d
}"#;
    let r = ask_pkg(&g, inverse).expect("inverse query runs");
    let n = match r.rows.first().and_then(|row| row.first()) {
        Some(Some(Term::Literal(l))) => l.value().parse::<i64>().unwrap_or(0),
        _ => 0,
    };
    // Must match the asserted forward (dependsOn) count for the same blocker.
    let forward = run(&canned::TASK_BLOCKS, Some("sq-8thu")).rows.len() as i64;
    assert_eq!(
        n, forward,
        "blockedBy (entailed inverse) must match dependsOn (asserted) count"
    );
    assert!(n > 0, "the entailed inverse must be non-empty for sq-8thu");
}
