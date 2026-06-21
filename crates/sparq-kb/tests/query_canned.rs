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
use sparq_kb::query::{ask_pkg, load_pkg};

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
