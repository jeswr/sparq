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
//! mode without `--abox` the data is a deterministic WITNESS ABox synthesised from the
//! TBox vocabulary (one witness assertion per named class/property), so every predicate the
//! rewriting can touch is populated; with `--abox` the agreement runs over the real data
//! (stronger). A disagreement aborts the run (exit 1) — it is an unsoundness signal, never
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

use oxrdf::{NamedOrBlankNode, Term, Triple};
use spargebra::{Query, SparqlParser};
use sparq_core::Graph;
use sparq_reason_ql::{rewrite, rewrite_production};
use std::collections::BTreeSet;
use std::time::Instant;

const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const RDFS_SUB_CLASS_OF: &str = "http://www.w3.org/2000/01/rdf-schema#subClassOf";
const RDFS_SUB_PROPERTY_OF: &str = "http://www.w3.org/2000/01/rdf-schema#subPropertyOf";
const RDFS_DOMAIN: &str = "http://www.w3.org/2000/01/rdf-schema#domain";
const RDFS_RANGE: &str = "http://www.w3.org/2000/01/rdf-schema#range";
const OWL_INVERSE_OF: &str = "http://www.w3.org/2002/07/owl#inverseOf";
const OWL_ON_PROPERTY: &str = "http://www.w3.org/2002/07/owl#onProperty";
const OWL_CLASS: &str = "http://www.w3.org/2002/07/owl#Class";
const OWL_OBJECT_PROPERTY: &str = "http://www.w3.org/2002/07/owl#ObjectProperty";
const OWL_DATATYPE_PROPERTY: &str = "http://www.w3.org/2002/07/owl#DatatypeProperty";
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

fn named(iri: &str) -> bool {
    // Skip the built-in vocabularies — witnesses for owl:Thing etc. would be noise.
    !(iri.starts_with("http://www.w3.org/2002/07/owl#")
        || iri.starts_with("http://www.w3.org/2000/01/rdf-schema#")
        || iri.starts_with("http://www.w3.org/1999/02/22-rdf-syntax-ns#")
        || iri.starts_with("http://www.w3.org/2001/XMLSchema#"))
}

/// Synthesise the deterministic WITNESS ABox: one assertion per named class / property in
/// the TBox vocabulary, so every predicate PerfectRef can rewrite into is populated and the
/// raw-vs-minimised result-set agreement is a non-vacuous differential.
fn witness_abox(tbox: &[Triple]) -> Graph {
    let mut classes: BTreeSet<String> = BTreeSet::new();
    let mut props: BTreeSet<String> = BTreeSet::new();
    let mut dt_props: BTreeSet<String> = BTreeSet::new();
    for t in tbox {
        let s = match &t.subject {
            NamedOrBlankNode::NamedNode(n) => Some(n.as_str().to_owned()),
            _ => None,
        };
        let o = match &t.object {
            Term::NamedNode(n) => Some(n.as_str().to_owned()),
            _ => None,
        };
        match t.predicate.as_str() {
            RDFS_SUB_CLASS_OF => {
                classes.extend(s.into_iter().chain(o).filter(|i| named(i)));
            }
            RDFS_SUB_PROPERTY_OF | OWL_INVERSE_OF => {
                props.extend(s.into_iter().chain(o).filter(|i| named(i)));
            }
            RDFS_DOMAIN | RDFS_RANGE => {
                props.extend(s.into_iter().filter(|i| named(i)));
                classes.extend(o.into_iter().filter(|i| named(i)));
            }
            OWL_ON_PROPERTY => {
                props.extend(o.into_iter().filter(|i| named(i)));
            }
            RDF_TYPE => match o.as_deref() {
                Some(OWL_CLASS) => classes.extend(s.into_iter().filter(|i| named(i))),
                Some(OWL_OBJECT_PROPERTY) => props.extend(s.into_iter().filter(|i| named(i))),
                Some(OWL_DATATYPE_PROPERTY) => dt_props.extend(s.into_iter().filter(|i| named(i))),
                _ => {}
            },
            _ => {}
        }
    }
    let mut nt = String::new();
    for (i, c) in classes.iter().enumerate() {
        nt.push_str(&format!("<{}ci{}> <{}> <{}> .\n", WITNESS, i, RDF_TYPE, c));
    }
    for (i, p) in props.iter().enumerate() {
        if dt_props.contains(p) {
            continue;
        }
        nt.push_str(&format!("<{}ps{}> <{}> <{}po{}> .\n", WITNESS, i, p, WITNESS, i));
    }
    for (i, p) in dt_props.iter().enumerate() {
        nt.push_str(&format!("<{}ds{}> <{}> \"w{}\" .\n", WITNESS, i, p, i));
    }
    Graph::load_str(&nt, "ntriples").expect("witness ABox must load")
}

fn run_gather(suite: &str, tbox_path: &str, queries_dir: &str, abox_path: Option<&str>) {
    let tbox = load_triples(tbox_path);
    let (data, data_label) = match abox_path {
        Some(p) => {
            let text = std::fs::read_to_string(p)
                .unwrap_or_else(|e| panic!("cannot read ABox {}: {}", p, e));
            let fmt = if p.ends_with(".nt") || p.ends_with(".ntriples") { "ntriples" } else { "turtle" };
            (
                Graph::load_str(&text, fmt).unwrap_or_else(|e| panic!("ABox {} load: {}", p, e)),
                "abox",
            )
        }
        None => (witness_abox(&tbox), "witness-abox"),
    };

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
        abox_path.unwrap_or("synthesised from the TBox vocabulary")
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
        let answers = assert_ucq_equivalence(&data, &raw.query, &minimised.query, id);

        // End-to-end leg (rewrite + execute), labelled; execution-only measured separately.
        let t_exec = Instant::now();
        let _ = sparq_engine::count(&data, &minimised.query.to_string())
            .unwrap_or_else(|e| panic!("{}: minimised UCQ execution failed: {}", id, e));
        let exec_ms = ms(t_exec);
        let t_e2e = Instant::now();
        let e2e = rewrite_production(&query, &tbox).expect("second rewrite must succeed");
        let _ = sparq_engine::count(&data, &e2e.query.to_string())
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
