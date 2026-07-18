//! [SONNET-4.6] sq-hmd7l.9 (epic sq-hmd7l): the sparq leg of the OWL 2 QL rewriting comparison
//! vs Ontop on the NPD + Requiem suites (`scripts/bench/reason-ql-same-box.sh`). Run:
//!
//!     cargo run -p sparq-reason-ql --example ql_npd_requiem_bench --release --features experimental -- --smoke
//!     cargo run -p sparq-reason-ql --example ql_npd_requiem_bench --release --features experimental -- \
//!         <suite> <tbox.nt|.ttl> <queries-dir> [--abox <data.nt|.ttl>]
//!
//! TWO metrics, both required by the bead: rewrite WALL TIME and output UCQ SIZE (disjunct
//! count). REGIME LABELS (printed in the `#`-prefixed banner, echoed into the harness
//! envelope): the `*_rewrite_ms` columns are REWRITER-PHASE ONLY (in-process `rewrite` /
//! `rewrite_production`, no execution); `exec_ms`/`e2e_ms` (only with `--abox`) are the
//! execution of the minimised UCQ over the loaded ABox and rewrite+execute respectively —
//! the column comparable to Ontop's end-to-end CLI regime.
//!
//! THE UCQ-EQUIVALENCE SANITY CHECK IS THE GATE (the sq-hmd7l.9 INVARIANT). Before ANY
//! timing row is printed for a query, the RAW PerfectRef UCQ (`rewrite`) and the MINIMISED
//! production UCQ (`rewrite_production`) are BOTH executed over the same data and their
//! result SETS must agree — containment minimisation must never change answers. In gather
//! mode without `--abox` the data is a deterministic PER-QUERY WITNESS ABox: the frozen
//! canonical instance of every CQ disjunct of the original query, the raw UCQ, and the
//! minimised UCQ (variables/blank nodes frozen to fresh per-disjunct IRIs; IRI and literal
//! constants kept as-is). Every disjunct matches at least its own frozen instance via the
//! identity homomorphism — including query-only predicates absent from the TBox and
//! multi-atom shared-variable joins — and because frozen terms are disjoint across
//! disjuncts, a minimisation that drops a NON-subsumed disjunct changes the result set
//! (the classical canonical-database containment argument for UCQs). CAVEAT: a disjunct
//! under a re-applied FILTER/VALUES modifier (the B3/B4 pass-through) is populated but its
//! frozen bindings may still fail the modifier, so `--abox` real data remains the stronger
//! regime. A disagreement aborts the run (exit 1) — it is an unsoundness signal, never
//! papered over with a plausible timing.
//!
//! `--smoke` is the hermetic acceptance path (no network, no files): NPD/Requiem-shaped
//! fixtures with HAND-VERIFIED closed-form minimised-UCQ sizes and pinned certain-answer
//! counts, each asserted before its timing row — same discipline as `ql_rewrite_bench`'s
//! closed forms (which this example does NOT touch).
//!
//! Out-of-scope queries (OPTIONAL/FILTER/aggregation/… — the fail-closed CQ gate) are
//! REPORTED per-row as `out-of-scope`, not errors: on the real NPD query mix that count is
//! itself an honest datum about rewriter coverage.

use oxrdf::Triple;
use spargebra::algebra::GraphPattern;
use spargebra::term::{NamedNodePattern, TermPattern, TriplePattern};
use spargebra::{Query, SparqlParser};
use sparq_core::Graph;
use sparq_reason_ql::{rewrite, rewrite_production};
use std::time::Instant;

/// Witness-ABox namespace — disjoint from any real suite vocabulary.
const WITNESS: &str = "http://sparq.dev/bench/ql-witness#";

fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1_000.0
}

/// Execute a query and return its result SET as (vars, sorted row strings). All fixture and
/// suite queries are `SELECT DISTINCT`, so the sorted rows are a canonical set encoding.
fn eval_rows(graph: &Graph, query: &Query) -> (Vec<String>, Vec<String>) {
    let res = sparq_engine::query(graph, &query.to_string())
        .unwrap_or_else(|e| panic!("rewritten query must execute: {}", e));
    let vars = res.vars.iter().map(|v| v.to_string()).collect();
    let mut rows: Vec<String> = res
        .rows
        .iter()
        .map(|row| {
            row.iter()
                .map(|cell| cell.as_ref().map(|t| t.to_string()).unwrap_or_default())
                .collect::<Vec<_>>()
                .join("\t")
        })
        .collect();
    rows.sort();
    rows.dedup(); // DISTINCT already dedupes; belt-and-braces so the comparison is set-vs-set
    (vars, rows)
}

/// The UCQ-equivalence sanity check: raw PerfectRef UCQ vs minimised production UCQ must
/// return the SAME result set over the same data. Returns the agreed row count.
fn assert_ucq_equivalence(graph: &Graph, raw: &Query, minimised: &Query, id: &str) -> usize {
    let (raw_vars, raw_rows) = eval_rows(graph, raw);
    let (min_vars, min_rows) = eval_rows(graph, minimised);
    assert_eq!(
        raw_vars, min_vars,
        "{}: raw and minimised UCQ project different variables",
        id
    );
    assert_eq!(
        raw_rows, min_rows,
        "{}: UCQ-EQUIVALENCE FAILURE — minimisation changed the result set (unsoundness signal; timing suppressed)",
        id
    );
    min_rows.len()
}

// =========================================================================================
// --smoke: hermetic NPD/Requiem-shaped fixtures with closed-form UCQ sizes + pinned answers
// =========================================================================================

const TURTLE_PREFIXES: &str = "@prefix : <http://ex/> . \
    @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> . \
    @prefix owl: <http://www.w3.org/2002/07/owl#> . ";
const SPARQL_PREFIXES: &str = "PREFIX : <http://ex/> \
    PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#> \
    PREFIX owl: <http://www.w3.org/2002/07/owl#> ";

struct SmokeCase {
    id: &'static str,
    tbox_ttl: &'static str,
    abox_ttl: &'static str,
    query: &'static str,
    /// Hand-verified closed-form MINIMISED UCQ size (the size axis of the comparison).
    expected_min_disjuncts: usize,
    /// Pinned certain-answer count over the fixture ABox.
    expected_answers: usize,
}

/// Isolated per-case fixtures (no shared TBox — every closed form is hand-verifiable in
/// isolation). Cases 1–2 mirror NPD shapes (facility/platform/wellbore hierarchy, an
/// operatedBy domain axiom); cases 3–6 mirror Requiem shapes (inverse role, role chain,
/// existential generator, redundant conjunction that minimises).
const SMOKE_CASES: &[SmokeCase] = &[
    SmokeCase {
        id: "npd-class-hierarchy",
        tbox_ttl: ":Platform rdfs:subClassOf :Facility . :Facility rdfs:subClassOf :Asset . \
                   :Wellbore rdfs:subClassOf :Asset .",
        abox_ttl: ":a1 a :Asset . :f1 a :Facility . :p1 a :Platform . :w1 a :Wellbore . :o1 a :Other .",
        query: "SELECT DISTINCT ?x WHERE { ?x a :Asset }",
        // Asset + Facility + Platform + Wellbore — 2-deep chain plus a sibling.
        expected_min_disjuncts: 4,
        expected_answers: 4,
    },
    SmokeCase {
        id: "npd-domain-role",
        tbox_ttl: ":operatedBy rdfs:domain :Facility . :Platform rdfs:subClassOf :Facility .",
        abox_ttl: ":f1 a :Facility . :p1 a :Platform . :x1 :operatedBy :op1 .",
        query: "SELECT DISTINCT ?x WHERE { ?x a :Facility }",
        // Facility + Platform + the ∃operatedBy disjunct the domain axiom introduces.
        expected_min_disjuncts: 3,
        expected_answers: 3,
    },
    SmokeCase {
        id: "requiem-inverse-role",
        tbox_ttl: ":employs owl:inverseOf :worksFor .",
        abox_ttl: ":acme :employs :alice . :bob :worksFor :initech .",
        query: "SELECT DISTINCT ?c ?w WHERE { ?c :employs ?w }",
        // employs(c,w) + worksFor(w,c).
        expected_min_disjuncts: 2,
        expected_answers: 2,
    },
    SmokeCase {
        id: "requiem-role-chain",
        tbox_ttl: ":manages rdfs:subPropertyOf :supervises . :supervises rdfs:subPropertyOf :worksFor .",
        abox_ttl: ":a :worksFor :b . :c :supervises :d . :e :manages :f .",
        query: "SELECT DISTINCT ?x ?y WHERE { ?x :worksFor ?y }",
        // worksFor + supervises + manages.
        expected_min_disjuncts: 3,
        expected_answers: 3,
    },
    SmokeCase {
        id: "requiem-exists-generator",
        tbox_ttl: ":Employee rdfs:subClassOf [ owl:onProperty :worksFor ; owl:someValuesFrom owl:Thing ] .",
        abox_ttl: ":bob :worksFor :initech . :erin a :Employee .",
        query: "SELECT DISTINCT ?x WHERE { ?x :worksFor ?y }",
        // worksFor(x,y) with ?y non-distinguished + the Employee generator.
        expected_min_disjuncts: 2,
        expected_answers: 2,
    },
    SmokeCase {
        id: "requiem-join-minimises",
        tbox_ttl: ":Manager rdfs:subClassOf :Employee .",
        abox_ttl: ":dave a :Manager . :erin a :Employee .",
        query: "SELECT DISTINCT ?x WHERE { ?x a :Manager . ?x a :Employee }",
        // Manager(x) ∧ Employee(x) collapses to Manager(x): minimised UCQ is a single CQ.
        expected_min_disjuncts: 1,
        expected_answers: 1,
    },
];

fn parse_tbox_ttl(ttl: &str) -> Vec<Triple> {
    let full = format!("{}{}", TURTLE_PREFIXES, ttl);
    oxttl::TurtleParser::new()
        .for_reader(full.as_bytes())
        .collect::<Result<Vec<_>, _>>()
        .expect("fixture TBox turtle must parse")
}

fn parse_query(body: &str) -> Query {
    SparqlParser::new()
        .parse_query(&format!("{}{}", SPARQL_PREFIXES, body))
        .expect("fixture query must parse")
}

fn run_smoke() {
    println!("ql_npd_requiem_bench --smoke: hermetic NPD/Requiem-shaped fixtures (no network)");
    println!("# regime: *_rewrite_ms = rewriter-phase only (in-process); answers = certain answers over the fixture ABox");
    println!("case\tstatus\traw_disjuncts\tmin_disjuncts\traw_rewrite_ms\tmin_rewrite_ms\tanswers\tequivalence");
    for case in SMOKE_CASES {
        let tbox = parse_tbox_ttl(case.tbox_ttl);
        let abox = Graph::load_str(&format!("{}{}", TURTLE_PREFIXES, case.abox_ttl), "turtle")
            .expect("fixture ABox must load");
        let query = parse_query(case.query);

        let t_raw = Instant::now();
        let raw = rewrite(&query, &tbox)
            .unwrap_or_else(|e| panic!("{} must be in QL scope: {}", case.id, e));
        let raw_ms = ms(t_raw);
        let t_min = Instant::now();
        let minimised = rewrite_production(&query, &tbox)
            .unwrap_or_else(|e| panic!("{} must be in QL scope: {}", case.id, e));
        let min_ms = ms(t_min);

        // --- ALL correctness gates BEFORE the timing row ---
        assert!(
            minimised.report.disjuncts <= minimised.report.disjuncts_before_minimisation,
            "{}: minimisation must only REMOVE disjuncts",
            case.id
        );
        assert!(
            minimised.report.disjuncts <= raw.report.disjuncts,
            "{}: minimised UCQ ({}) larger than raw UCQ ({})",
            case.id,
            minimised.report.disjuncts,
            raw.report.disjuncts
        );
        assert_eq!(
            minimised.report.disjuncts, case.expected_min_disjuncts,
            "{}: minimised UCQ size {} != hand-verified closed form {}",
            case.id, minimised.report.disjuncts, case.expected_min_disjuncts
        );
        let answers = assert_ucq_equivalence(&abox, &raw.query, &minimised.query, case.id);
        assert_eq!(
            answers, case.expected_answers,
            "{}: certain-answer count {} != pinned {}",
            case.id, answers, case.expected_answers
        );

        println!(
            "{}\tok\t{}\t{}\t{:.6}\t{:.6}\t{}\tagree(fixture-abox)",
            case.id, raw.report.disjuncts, minimised.report.disjuncts, raw_ms, min_ms, answers
        );
    }
    println!(
        "SMOKE OK: {} cases — closed-form minimised-UCQ sizes held, raw-vs-minimised result sets agree, answer counts pinned (timings trend-only).",
        SMOKE_CASES.len()
    );
}

// =========================================================================================
// gather mode: <suite> <tbox.nt|.ttl> <queries-dir> [--abox <data.nt|.ttl>]
// =========================================================================================

fn load_triples(path: &str) -> Vec<Triple> {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("cannot read TBox {}: {}", path, e));
    if path.ends_with(".nt") || path.ends_with(".ntriples") {
        oxttl::NTriplesParser::new()
            .for_reader(text.as_bytes())
            .collect::<Result<Vec<_>, _>>()
            .unwrap_or_else(|e| panic!("TBox {} N-Triples parse: {}", path, e))
    } else {
        oxttl::TurtleParser::new()
            .for_reader(text.as_bytes())
            .collect::<Result<Vec<_>, _>>()
            .unwrap_or_else(|e| panic!("TBox {} Turtle parse: {}", path, e))
    }
}

/// Collect every BGP (CQ disjunct body) in a query's pattern tree. The shapes that can
/// occur here are exactly what the fail-closed CQ gate admits and what the emitter
/// produces: BGPs under a `Union` fold, wrapped in `Project`/`Distinct`/`Reduced`/`Slice`,
/// with the B3/B4 pass-through re-applying `Filter` and `Join`-with-`Values` around a
/// branch. Anything else is fail-closed: PANIC rather than synthesise a witness that
/// silently under-populates the data (use `--abox` for such suites).
fn collect_bgps<'a>(pattern: &'a GraphPattern, out: &mut Vec<&'a [TriplePattern]>) {
    match pattern {
        GraphPattern::Bgp { patterns } => out.push(patterns),
        GraphPattern::Union { left, right } | GraphPattern::Join { left, right } => {
            collect_bgps(left, out);
            collect_bgps(right, out);
        }
        GraphPattern::Filter { inner, .. }
        | GraphPattern::Project { inner, .. }
        | GraphPattern::Distinct { inner }
        | GraphPattern::Reduced { inner }
        | GraphPattern::Slice { inner, .. } => collect_bgps(inner, out),
        GraphPattern::Values { .. } => {} // constant bindings — contributes no triples
        other => panic!(
            "witness ABox synthesis: unsupported pattern shape {:?} — run with --abox",
            other
        ),
    }
}

fn query_pattern(q: &Query) -> &GraphPattern {
    match q {
        Query::Select { pattern, .. } | Query::Ask { pattern, .. } => pattern,
        _ => panic!("witness ABox synthesis: expected a SELECT/ASK query"),
    }
}

/// Synthesise the deterministic PER-QUERY WITNESS ABox for one equivalence check: the
/// union of the FROZEN CANONICAL INSTANCES of every CQ disjunct of every given query
/// (original, raw UCQ, minimised UCQ). Freezing maps each variable / blank node to a
/// fresh per-disjunct witness IRI and keeps IRI/literal constants as-is, so every
/// disjunct — query-only predicates and multi-atom shared-variable joins included —
/// matches at least its own instance (the identity homomorphism). Frozen terms are
/// disjoint across disjuncts, so a kept disjunct produces a dropped disjunct D's frozen
/// head tuple only via a homomorphism into D's instance, i.e. only when D was genuinely
/// subsumed: a minimisation that drops a non-subsumed disjunct changes the result set
/// (the canonical-database containment argument). Regression-tested below.
fn witness_abox(queries: &[&Query]) -> Graph {
    let mut nt = String::new();
    for (qi, q) in queries.iter().enumerate() {
        let mut bgps: Vec<&[TriplePattern]> = Vec::new();
        collect_bgps(query_pattern(q), &mut bgps);
        for (di, bgp) in bgps.iter().enumerate() {
            // One fresh constant per (query, disjunct, kind, name) — shared variables
            // within a disjunct freeze to the SAME constant, so joins are satisfied.
            let frozen = |kind: &str, name: &str| {
                format!("<{}q{}d{}{}{}>", WITNESS, qi, di, kind, name)
            };
            for tp in bgp.iter() {
                let s = match &tp.subject {
                    TermPattern::NamedNode(n) => format!("<{}>", n.as_str()),
                    TermPattern::BlankNode(b) => frozen("b", b.as_str()),
                    TermPattern::Variable(v) => frozen("v", v.as_str()),
                    other => panic!("witness ABox: unsupported subject pattern {:?}", other),
                };
                let p = match &tp.predicate {
                    NamedNodePattern::NamedNode(n) => format!("<{}>", n.as_str()),
                    NamedNodePattern::Variable(v) => frozen("p", v.as_str()),
                };
                let o = match &tp.object {
                    TermPattern::NamedNode(n) => format!("<{}>", n.as_str()),
                    TermPattern::Literal(l) => l.to_string(), // canonical N-Triples form
                    TermPattern::BlankNode(b) => frozen("b", b.as_str()),
                    TermPattern::Variable(v) => frozen("v", v.as_str()),
                    other => panic!("witness ABox: unsupported object pattern {:?}", other),
                };
                nt.push_str(&format!("{} {} {} .\n", s, p, o));
            }
        }
    }
    Graph::load_str(&nt, "ntriples").expect("witness ABox must load")
}

fn run_gather(suite: &str, tbox_path: &str, queries_dir: &str, abox_path: Option<&str>) {
    let tbox = load_triples(tbox_path);
    let real_abox: Option<Graph> = abox_path.map(|p| {
        let text = std::fs::read_to_string(p)
            .unwrap_or_else(|e| panic!("cannot read ABox {}: {}", p, e));
        let fmt = if p.ends_with(".nt") || p.ends_with(".ntriples") { "ntriples" } else { "turtle" };
        Graph::load_str(&text, fmt).unwrap_or_else(|e| panic!("ABox {} load: {}", p, e))
    });
    let data_label = if real_abox.is_some() { "abox" } else { "witness-abox" };

    let mut query_files: Vec<std::path::PathBuf> = std::fs::read_dir(queries_dir)
        .unwrap_or_else(|e| panic!("cannot read queries dir {}: {}", queries_dir, e))
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|ext| ext == "rq" || ext == "sparql"))
        .collect();
    query_files.sort();
    assert!(
        !query_files.is_empty(),
        "no .rq/.sparql query files in {}",
        queries_dir
    );

    println!(
        "# suite {}: TBox {} ({} triples), {} queries, equivalence data = {} ({})",
        suite,
        tbox_path,
        tbox.len(),
        query_files.len(),
        data_label,
        abox_path
            .unwrap_or("per-query frozen canonical instances of the original/raw/minimised UCQ disjuncts")
    );
    println!("# regime: *_rewrite_ms = rewriter-phase only (in-process rewrite/rewrite_production);");
    println!("#         exec_ms = minimised-UCQ execution over the loaded data; e2e_ms = rewrite+execute (comparable to an end-to-end competitor column)");
    println!("suite\tcase\tstatus\traw_disjuncts\tmin_disjuncts\tskipped_axioms\traw_rewrite_ms\tmin_rewrite_ms\tequivalence\tanswers\texec_ms\te2e_ms");

    let (mut in_scope, mut out_of_scope, mut parse_errors) = (0usize, 0usize, 0usize);
    for path in &query_files {
        let id = path.file_stem().and_then(|s| s.to_str()).unwrap_or("query");
        let text = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("cannot read query {}: {}", path.display(), e));
        let query = match SparqlParser::new().parse_query(&text) {
            Ok(q) => q,
            Err(e) => {
                parse_errors += 1;
                println!(
                    "{}\t{}\tparse-error: {}\tNA\tNA\tNA\tNA\tNA\tNA\tNA\tNA\tNA",
                    suite,
                    id,
                    e.to_string().replace(['\t', '\n'], " ")
                );
                continue;
            }
        };

        let t_raw = Instant::now();
        let raw = match rewrite(&query, &tbox) {
            Ok(r) => r,
            Err(e) => {
                out_of_scope += 1;
                println!(
                    "{}\t{}\tout-of-scope: {}\tNA\tNA\tNA\tNA\tNA\tNA\tNA\tNA\tNA",
                    suite,
                    id,
                    e.to_string().replace(['\t', '\n'], " ")
                );
                continue;
            }
        };
        let raw_ms = ms(t_raw);
        let t_min = Instant::now();
        let minimised = rewrite_production(&query, &tbox)
            .unwrap_or_else(|e| panic!("{}: raw rewrite succeeded but production failed: {}", id, e));
        let min_ms = ms(t_min);

        // UCQ-equivalence sanity check BEFORE the timing row (the sq-hmd7l.9 invariant).
        // Without --abox the data is the PER-QUERY witness: the frozen canonical instances
        // of every disjunct of the original, raw, and minimised queries, so the differential
        // is per-disjunct non-vacuous even for predicates that never occur in the TBox.
        let witness;
        let data: &Graph = match &real_abox {
            Some(g) => g,
            None => {
                witness = witness_abox(&[&query, &raw.query, &minimised.query]);
                &witness
            }
        };
        let answers = assert_ucq_equivalence(data, &raw.query, &minimised.query, id);

        // End-to-end leg (rewrite + execute), labelled; execution-only measured separately.
        let t_exec = Instant::now();
        let _ = sparq_engine::count(data, &minimised.query.to_string())
            .unwrap_or_else(|e| panic!("{}: minimised UCQ execution failed: {}", id, e));
        let exec_ms = ms(t_exec);
        let t_e2e = Instant::now();
        let e2e = rewrite_production(&query, &tbox).expect("second rewrite must succeed");
        let _ = sparq_engine::count(data, &e2e.query.to_string())
            .unwrap_or_else(|e| panic!("{}: e2e execution failed: {}", id, e));
        let e2e_ms = ms(t_e2e);

        in_scope += 1;
        println!(
            "{}\t{}\tok\t{}\t{}\t{}\t{:.6}\t{:.6}\tagree({})\t{}\t{:.6}\t{:.6}",
            suite,
            id,
            raw.report.disjuncts,
            minimised.report.disjuncts,
            minimised.report.skipped_axioms,
            raw_ms,
            min_ms,
            data_label,
            answers,
            exec_ms,
            e2e_ms
        );
    }
    println!(
        "# summary suite {}: {} in-scope, {} out-of-scope (fail-closed CQ gate), {} parse-errors of {} queries",
        suite,
        in_scope,
        out_of_scope,
        parse_errors,
        query_files.len()
    );
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--smoke") => run_smoke(),
        Some(suite) if args.len() >= 3 => {
            let abox = args
                .iter()
                .position(|a| a == "--abox")
                .and_then(|i| args.get(i + 1))
                .map(String::as_str);
            run_gather(suite, &args[1], &args[2], abox);
        }
        _ => {
            eprintln!(
                "usage: ql_npd_requiem_bench --smoke | <suite> <tbox.nt|.ttl> <queries-dir> [--abox <data.nt|.ttl>]"
            );
            std::process::exit(2);
        }
    }
}

// =========================================================================================
// Witness-gate regression tests (run by `cargo test -p sparq-reason-ql --features
// experimental` — the [[example]] entry sets `test = true`). They pin the vacuity the old
// TBox-vocabulary witness had for query-only predicates + multi-atom joins, and prove the
// frozen-instance witness both populates every disjunct and DETECTS a wrongly-dropped one.
// =========================================================================================
#[cfg(test)]
mod tests {
    use super::*;

    /// A raw UCQ whose predicates occur ONLY in the query (never in any TBox): a multi-atom
    /// shared-variable join disjunct, plus a literal-object disjunct.
    const RAW_UCQ: &str = "SELECT DISTINCT ?x WHERE { \
        { ?x :headOf ?d . ?d :partOf ?u } UNION { ?x :label \"w\" } }";
    /// A deliberately WRONG minimisation of [`RAW_UCQ`]: the non-subsumed join disjunct is
    /// dropped, so the two queries are semantically different.
    const WRONG_MIN_UCQ: &str = "SELECT DISTINCT ?x WHERE { ?x :label \"w\" }";
    /// A sound minimisation of [`RAW_UCQ`] (same UCQ, disjuncts reordered).
    const EQUIV_UCQ: &str = "SELECT DISTINCT ?x WHERE { \
        { ?x :label \"w\" } UNION { ?x :headOf ?d . ?d :partOf ?u } }";

    /// The PRE-FIX witness construction, reproduced literally for the TBox
    /// `:Manager rdfs:subClassOf :Employee .`: one rdf:type assertion per TBox class. The
    /// query-only predicates above receive no data from it, so BOTH the raw and the wrongly
    /// minimised UCQ evaluate to the empty set and the old gate agreed VACUOUSLY — the
    /// defect this regression pins.
    #[test]
    fn tbox_vocabulary_witness_was_vacuous_for_query_only_predicates() {
        let data = Graph::load_str(
            "<http://sparq.dev/bench/ql-witness#ci0> \
             <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://ex/Employee> .\n\
             <http://sparq.dev/bench/ql-witness#ci1> \
             <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://ex/Manager> .\n",
            "ntriples",
        )
        .expect("old-style witness must load");
        let (_, raw_rows) = eval_rows(&data, &parse_query(RAW_UCQ));
        let (_, wrong_rows) = eval_rows(&data, &parse_query(WRONG_MIN_UCQ));
        assert!(
            raw_rows.is_empty() && wrong_rows.is_empty(),
            "TBox-vocabulary witness leaves query-only predicates empty on both sides — \
             the vacuous agreement the per-query frozen-instance witness replaces"
        );
    }

    /// The frozen-instance witness populates the join disjunct (query-only predicates,
    /// shared-variable structure) AND the literal-object disjunct, and the equivalence gate
    /// DETECTS the dropped non-subsumed disjunct as a result-set mismatch.
    #[test]
    fn frozen_instance_witness_detects_wrongly_dropped_join_disjunct() {
        let raw = parse_query(RAW_UCQ);
        let wrong = parse_query(WRONG_MIN_UCQ);
        let data = witness_abox(&[&raw, &wrong]);
        let (_, raw_rows) = eval_rows(&data, &raw);
        let (_, wrong_rows) = eval_rows(&data, &wrong);
        assert!(
            !raw_rows.is_empty(),
            "every raw disjunct must match its own frozen instance"
        );
        assert!(
            !wrong_rows.is_empty(),
            "the literal-object disjunct must match its own frozen instance"
        );
        assert_ne!(
            raw_rows, wrong_rows,
            "dropping a non-subsumed disjunct must surface as a result-set mismatch"
        );
    }

    /// A sound minimisation (equivalent UCQ, reordered disjuncts) still agrees over the
    /// frozen-instance witness, with a non-empty (non-vacuous) agreed row set.
    #[test]
    fn frozen_instance_witness_agrees_for_equivalent_ucqs() {
        let raw = parse_query(RAW_UCQ);
        let equiv = parse_query(EQUIV_UCQ);
        let data = witness_abox(&[&raw, &equiv]);
        let answers = assert_ucq_equivalence(&data, &raw, &equiv, "equivalent-ucqs");
        assert!(answers > 0, "the agreement must be over a non-empty row set");
    }
}
