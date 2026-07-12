//! Differential test for SPARQL Updates over STRUCTURALLY FORKED generations (the
//! sparq-serve writer's path): `fork → update_in_place → publish` chained across
//! generations must be observationally identical to the engine's rebuild path
//! (`update`) applied to a flat graph — across a query battery covering scans,
//! merge/bind joins (including joins THROUGH terms first interned in a fork),
//! filters (numeric — exercising the forked numeric cache), aggregates, OPTIONAL,
//! deleted-triple non-visibility, re-inserted triples, and named-graph updates.
//! Earlier generations are held alive and re-checked (snapshot isolation).

use sparq_core::Graph;

fn count(g: &Graph, q: &str) -> usize {
    sparq_engine::count(g, q).unwrap_or_else(|e| panic!("query failed: {e}\n{q}"))
}

/// Serialised SELECT * result (sorted rows) — order-insensitive comparison.
fn rows(g: &Graph, q: &str) -> Vec<String> {
    let r = sparq_engine::query(g, q).unwrap_or_else(|e| panic!("query failed: {e}\n{q}"));
    let mut out: Vec<String> = r
        .rows
        .iter()
        .map(|row| {
            row.iter()
                .map(|t| t.as_ref().map_or("UNDEF".to_string(), |t| t.to_string()))
                .collect::<Vec<_>>()
                .join("\t")
        })
        .collect();
    out.sort();
    out
}

const BATTERY: &[&str] = &[
    // plain scans
    "PREFIX : <http://ex/> SELECT * WHERE { ?s :p ?o }",
    "PREFIX : <http://ex/> SELECT * WHERE { ?s ?p ?o }",
    // merge/bind join, incl. through fork-interned terms
    "PREFIX : <http://ex/> SELECT * WHERE { ?s :p ?m . ?m :p ?o }",
    "PREFIX : <http://ex/> SELECT * WHERE { ?s :linked ?m . ?m :p ?o }",
    // numeric filter (forked numeric cache) + comparison
    "PREFIX : <http://ex/> SELECT * WHERE { ?s :age ?a . FILTER(?a > 20) }",
    "PREFIX : <http://ex/> SELECT * WHERE { ?s ?p ?o . FILTER(isLiteral(?o)) }",
    // aggregate
    "PREFIX : <http://ex/> SELECT (COUNT(*) AS ?c) WHERE { ?s ?p ?o }",
    "PREFIX : <http://ex/> SELECT ?p (COUNT(?s) AS ?c) WHERE { ?s ?p ?o } GROUP BY ?p",
    // OPTIONAL
    "PREFIX : <http://ex/> SELECT * WHERE { ?s :p ?o OPTIONAL { ?s :age ?a } }",
    // named graphs
    "PREFIX : <http://ex/> SELECT * WHERE { GRAPH ?g { ?s ?p ?o } }",
];

fn assert_battery(forked: &Graph, flat: &Graph, ctx: &str) {
    for q in BATTERY {
        assert_eq!(rows(forked, q), rows(flat, q), "{ctx}: query diverged: {q}");
    }
}

#[test]
fn forked_generations_match_rebuild() {
    let src = "@prefix : <http://ex/> . \
               :a :p :b . :b :p :c . :c :p :a . \
               :a :age 30 . :b :age 25 . :c :age 17 .";
    let g0 = Graph::load_str(src, "turtle").unwrap();
    let mut flat = Graph::load_str(src, "turtle").unwrap();

    // The generational update sequence — each op runs once per generation step.
    // Deliberately includes: inserts with brand-new terms (:n*, :linked), a join
    // THROUGH those new terms, deletes of base triples, a re-insert of a deleted
    // triple, numeric literals (filter cache growth), a DELETE/INSERT WHERE
    // rewrite, and named-graph operations.
    let ops: &[&str] = &[
        "PREFIX : <http://ex/> INSERT DATA { :n0 :linked :a . :n0 :age 41 . :n1 :linked :b . :n1 :age 8 }",
        "PREFIX : <http://ex/> DELETE DATA { :a :p :b }",
        "PREFIX : <http://ex/> INSERT DATA { :a :p :b . :n2 :p :n0 }", // re-insert + new-term object link
        "PREFIX : <http://ex/> DELETE { ?s :age ?a } INSERT { ?s :years ?a } WHERE { ?s :age ?a . FILTER(?a < 10) }",
        "PREFIX : <http://ex/> INSERT DATA { GRAPH :g1 { :x :q :y . :x :q :z } }",
        "PREFIX : <http://ex/> DELETE DATA { GRAPH :g1 { :x :q :z } }",
        "PREFIX : <http://ex/> INSERT { GRAPH :g2 { ?s ?p ?o } } WHERE { GRAPH :g1 { ?s ?p ?o } }",
        "PREFIX : <http://ex/> DELETE WHERE { ?s :linked ?o . ?o :p ?x }",
        "PREFIX : <http://ex/> INSERT DATA { :m0 :p \"lit\" . :m0 :age 99 }",
    ];

    // The published-generation chain (every generation stays alive, like the ring).
    let mut generations: Vec<Graph> = vec![g0];
    let mut snapshots: Vec<Vec<String>> = vec![rows(&generations[0], BATTERY[1])];

    for (i, op) in ops.iter().enumerate() {
        let base = generations.last().unwrap();
        let mut working = base.fork(); // the writer's cheap fork
        sparq_engine::update_in_place(&mut working, op).unwrap();
        flat = sparq_engine::update(&flat, op).unwrap(); // reference: full rebuild

        assert_battery(&working, &flat, &format!("op {i}"));

        // Snapshot isolation: every retained generation still answers its own state.
        for (k, old) in generations.iter().enumerate() {
            assert_eq!(
                rows(old, BATTERY[1]),
                snapshots[k],
                "generation {k} drifted after op {i}"
            );
        }

        snapshots.push(rows(&working, BATTERY[1]));
        generations.push(working);
    }

    // Compact the head generation: nothing observable may change, and further
    // fork+update cycles continue working from the compacted base.
    let mut head = generations.last().unwrap().fork();
    head.compact().unwrap();
    assert_eq!(head.pending_delta_len(), 0);
    assert_battery(&head, &flat, "post-compact");

    let mut next = head.fork();
    sparq_engine::update_in_place(
        &mut next,
        "PREFIX : <http://ex/> INSERT DATA { :post :p :compact }",
    )
    .unwrap();
    flat = sparq_engine::update(
        &flat,
        "PREFIX : <http://ex/> INSERT DATA { :post :p :compact }",
    )
    .unwrap();
    assert_battery(&next, &flat, "post-compact generation");
    assert_eq!(
        count(&head, BATTERY[1]),
        count(&flat, BATTERY[1]) - 1,
        "compacted base isolated"
    );
}
