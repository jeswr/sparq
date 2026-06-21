//! End-to-end proof of the Phase-1 "query-the-PKG" recipe (introspect → ground → ask).
//!
//! [OPUS-4.8] sq-2m6zm.3 (epic sq-2m6zm). 🤖 SPARQ agent — query-the-PKG skill PoC.
//! These run the recipe against the ingested PKG and assert the three steps actually
//! fire: the closure materialises entailed triples, the introspection card is a real
//! token-budgeted schema card, the §4.1-class queries return the minimal answer, the
//! bound IRIs are grounded into ABSTAT-style minimal subgraphs, and the executed
//! SPARQL is surfaced for verification. The answers are computed by sparq's own engine
//! over the closed graph — within the store the answer cannot be fabricated.
//!
//! Run: `cargo test -p sparq-kb --features skill --test skill_recipe -- --nocapture`
#![cfg(feature = "skill")]

use sparq_kb::skill::{
    ask, introspect, load_closed_pkg, query_the_pkg, recipes, Closure, SkillConfig,
};

/// The closure step must run and (for the PKG, which declares `owl:inverseOf` +
/// `owl:SymmetricProperty` + RDFS subclass edges) add entailed triples under OWL-RL.
#[test]
fn closure_materialises_entailed_triples() {
    let (_g, entailed) = load_closed_pkg(Closure::OwlRl).expect("PKG loads + closes");
    eprintln!("OWL-RL closure added {entailed} entailed triples");
    assert!(
        entailed > 0,
        "OWL-RL closure must materialise entailed triples over the PKG ontology"
    );
}

/// The introspect step must produce a non-empty, budget-bounded schema card that
/// actually mentions the PKG classes (the grounding context for query construction).
#[test]
fn introspect_yields_schema_card() {
    let (g, _) = load_closed_pkg(Closure::OwlRl).expect("PKG loads");
    let budget = 4096;
    let card = introspect(&g, budget);
    eprintln!("=== schema card ({} chars) ===\n{card}", card.len());
    assert!(!card.is_empty(), "schema card must be non-empty");
    assert!(
        card.chars().count() <= budget,
        "schema card must respect its char budget"
    );
    assert!(
        card.contains("Task") || card.contains("Finding") || card.contains("Class"),
        "schema card must surface the PKG schema; got: {card}"
    );
    // budget 0 disables the introspect step.
    assert!(introspect(&g, 0).is_empty(), "budget 0 must skip the card");
}

/// The §4.1 ready-frontier count must be non-empty over the real bd backlog, and the
/// recipe must surface the executed SPARQL + the closure report.
#[test]
fn ready_frontier_count() {
    let ans = query_the_pkg(recipes::READY_FRONTIER_COUNT, &SkillConfig::default())
        .expect("frontier recipe runs");
    eprintln!("\n=== executed SPARQL ===\n{}", ans.sparql);
    eprintln!(
        "closure={:?} entailed={}",
        ans.closure, ans.entailed_triples
    );
    assert_eq!(
        ans.sparql,
        recipes::READY_FRONTIER_COUNT,
        "executed SPARQL must be surfaced verbatim"
    );
    assert!(
        ans.entailed_triples > 0,
        "the recipe must close the graph before asking"
    );
    assert!(
        !ans.schema_card.is_empty(),
        "the recipe must mine the schema card"
    );
    assert_eq!(ans.row_count(), 1, "COUNT returns exactly one row");
    let count: i64 = ans.rows[0][0]
        .as_ref()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    eprintln!("ready-frontier count = {count}");
    assert!(
        count > 0,
        "the ready-frontier must be non-empty over the bd backlog; got {count}"
    );
}

/// The ready-frontier IRI projection must bind task IRIs AND ground each into a minimal
/// subgraph (the "ground" half of the recipe) — verifiable facts, not prose.
#[test]
fn ready_frontier_grounds_tasks() {
    let cfg = SkillConfig {
        max_grounded_nodes: 5,
        ..Default::default()
    };
    let ans = query_the_pkg(recipes::READY_FRONTIER, &cfg).expect("frontier recipe runs");
    eprintln!("\n=== ready-frontier ({} rows) ===", ans.row_count());
    assert!(
        ans.row_count() > 0,
        "the ready-frontier must bind at least one task"
    );
    assert_eq!(ans.vars, vec!["t".to_string(), "prio".to_string()]);

    // Each grounded node is a real PKG Task IRI with a non-empty minimal subgraph.
    assert!(
        !ans.grounded.is_empty(),
        "the recipe must ground the bound task IRIs"
    );
    assert!(
        ans.grounded.len() <= cfg.max_grounded_nodes,
        "grounding must respect max_grounded_nodes"
    );
    for gn in &ans.grounded {
        eprintln!("\n{} — {} fact(s)", gn.iri, gn.facts.len());
        for f in &gn.facts {
            eprintln!("  {} -> {}", f.predicate, f.object);
        }
        assert!(gn.iri.starts_with("http"), "grounded node must be an IRI");
        assert!(
            !gn.facts.is_empty(),
            "a grounded task must carry a minimal subgraph"
        );
        // Minimal subgraph respects the per-node fact budget.
        assert!(
            gn.facts.len() <= cfg.max_facts_per_subgraph,
            "subgraph must respect max_facts_per_subgraph"
        );
        // The grounded facts include the task's status (the load-bearing predicate the
        // frontier selected on) — proof the subgraph is the relevant minimal pattern.
        assert!(
            gn.facts
                .iter()
                .any(|f| f.predicate.ends_with("#status") || f.predicate.ends_with("#priority")),
            "a grounded Task's minimal subgraph must carry its status/priority; got {:?}",
            gn.facts
        );
    }
}

/// A "Findings about a topic" lookup (built from the verified template) must return the
/// sourced, confidence-bearing Findings for a real PKG topic — the citation half of the
/// guardrail, queryable.
#[test]
fn findings_about_a_topic() {
    let topic = "https://sparq.dev/ns/pkg/kb#topic-merge-discipline";
    let sparql = recipes::findings_about(topic);
    let ans = query_the_pkg(&sparql, &SkillConfig::default()).expect("findings recipe runs");
    eprintln!(
        "\n=== findings about {topic} ({} rows) ===",
        ans.row_count()
    );
    for row in &ans.rows {
        eprintln!("  {row:?}");
    }
    assert!(
        ans.sparql.contains(topic),
        "the executed SPARQL must bind the requested topic"
    );
    assert!(
        ans.row_count() > 0,
        "merge-discipline topic must have Findings"
    );
    // Projection is ?f ?label ?section ?conf — every Finding carries a source section
    // (the citation) and a confidence.
    assert_eq!(ans.vars, vec!["f", "label", "section", "conf"]);
    for row in &ans.rows {
        assert!(
            row[2].is_some(),
            "every Finding must carry its dcterms:source section"
        );
        assert!(
            row[3].is_some(),
            "every Finding must carry its pkg:confidence"
        );
    }
    // The Finding IRIs (?f) are grounded into minimal subgraphs.
    assert!(
        !ans.grounded.is_empty(),
        "the recipe must ground the Finding IRIs"
    );
}

/// RDFS-only closure is also a valid grounding profile; the recipe must run under it
/// (completeness is then RDFS-profile-relative, per the design).
#[test]
fn rdfs_closure_also_runs() {
    let cfg = SkillConfig {
        closure: Closure::Rdfs,
        ..Default::default()
    };
    let ans = query_the_pkg(recipes::READY_FRONTIER_COUNT, &cfg).expect("recipe runs under RDFS");
    assert_eq!(ans.closure, Closure::Rdfs);
    assert_eq!(ans.row_count(), 1);
}

/// The low-level `ask` over a caller-closed graph surfaces the executed SPARQL and
/// grounds bound IRIs without re-closing (the composable building block).
#[test]
fn ask_over_caller_closed_graph() {
    let (g, _) = load_closed_pkg(Closure::OwlRl).expect("PKG loads");
    let q = r#"
PREFIX pkg: <https://sparq.dev/ns/pkg#>
SELECT ?f WHERE { ?f a pkg:Finding } LIMIT 3"#;
    let ans = ask(&g, q, &SkillConfig::default()).expect("ask runs");
    assert_eq!(ans.sparql, q, "ask must surface the executed SPARQL");
    assert!(ans.row_count() > 0 && ans.row_count() <= 3);
    assert!(
        !ans.grounded.is_empty(),
        "ask must ground the bound Finding IRIs"
    );
}
