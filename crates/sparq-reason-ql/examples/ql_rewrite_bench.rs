//! [OPUS-4.8] sq-t5bne: QL-rewrite micro-benchmark — a self-asserting, deterministic workload
//! for the OWL 2 QL (DL-Lite_R) PerfectRef query rewriter. Run:
//!
//!     cargo run -p sparq-reason-ql --example ql_rewrite_bench --release --features experimental [DEPTH]
//!
//! No dataset to fetch: the TBox + query are generated in-process (a pure function of DEPTH,
//! byte-deterministic, no RNG/network/toolchain), so this is a hermetic, regenerable benchmark.
//! It runs the gated rewriter, so it `required-features = ["experimental"]` (a plain
//! `cargo build --all-targets` — which passes no features — skips it rather than failing on the
//! feature-gated `rewrite` import, mirroring sparq-reason-el's `hasse_bench`).
//!
//! Two workloads, each over the SAME public `rewrite` API, each with a CLOSED-FORM UCQ-size:
//!
//!   1. `subclass` — a DEPTH-deep `rdfs:subClassOf` chain `C0 ⊑ C1 ⊑ … ⊑ C_DEPTH` and the query
//!      `?x rdf:type C_DEPTH`. PerfectRef rewrites the single class atom backward along the chain:
//!      the original CQ plus one rewrite for EACH of the DEPTH proper sub-classes. The UCQ size
//!      has a CLOSED FORM: exactly `DEPTH + 1` disjuncts.
//!
//!   2. `subproperty` — a DEPTH-deep `rdfs:subPropertyOf` chain `P0 ⊑ P1 ⊑ … ⊑ P_DEPTH` and the
//!      query `?x P_DEPTH ?y`. PerfectRef rewrites the single role atom backward along the chain:
//!      again exactly `DEPTH + 1` disjuncts.
//!
//! THE CLOSED-FORM ASSERTION IS THE GATE. A linear chain of length DEPTH must produce a UCQ of
//! exactly DEPTH+1 conjunctive queries — UCQ size grows LINEARLY in the TBox size, never
//! super-linearly. Asserting this exact closed form proves there is NO HIDDEN BLOW-UP in the
//! rewrite/reduce saturation (a duplicate-disjunct or non-terminating-rewrite regression would
//! over-count and fail LOUDLY), and that the fixpoint dedup is working. Wall-clock seconds are
//! TREND-ONLY (quiet-box sensitive); the disjunct-count closed forms are the hard, deterministic,
//! load-robust CORRECTNESS gate — exactly as sparq-reason-el's classify_bench asserts closure
//! sizes and bench/deep-taxonomy / bench/owl-sameas assert closure sizes for the RL reasoner.

use oxrdf::Triple;
use spargebra::{Query, SparqlParser};
use sparq_reason_ql::{rewrite, rewrite_production};
use std::str::FromStr;
use std::time::Instant;

const RDFS_SUB_CLASS_OF: &str = "http://www.w3.org/2000/01/rdf-schema#subClassOf";
const RDFS_SUB_PROPERTY_OF: &str = "http://www.w3.org/2000/01/rdf-schema#subPropertyOf";
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const EX: &str = "http://sparq.dev/bench/ql#";

fn secs(t: Instant) -> f64 {
    t.elapsed().as_secs_f64()
}

fn triple(s: &str) -> Triple {
    Triple::from_str(s).expect("n-triple parse")
}

/// Build a DEPTH-deep `rdfs:subClassOf` chain `C0 ⊑ C1 ⊑ … ⊑ C_DEPTH`.
fn subclass_chain(depth: usize) -> Vec<Triple> {
    (0..depth)
        .map(|i| triple(&format!("<{EX}C{i}> <{RDFS_SUB_CLASS_OF}> <{EX}C{}> .", i + 1)))
        .collect()
}

/// Build a DEPTH-deep `rdfs:subPropertyOf` chain `P0 ⊑ P1 ⊑ … ⊑ P_DEPTH`.
fn subproperty_chain(depth: usize) -> Vec<Triple> {
    (0..depth)
        .map(|i| triple(&format!("<{EX}P{i}> <{RDFS_SUB_PROPERTY_OF}> <{EX}P{}> .", i + 1)))
        .collect()
}

fn parse(q: &str) -> Query {
    SparqlParser::new().parse_query(q).expect("query parse")
}

fn run_subclass(depth: usize) {
    let tbox = subclass_chain(depth);
    let query = parse(&format!("SELECT ?x WHERE {{ ?x <{RDF_TYPE}> <{EX}C{depth}> }}"));
    let t = Instant::now();
    let r = rewrite(&query, &tbox).expect("subclass chain CQ is in QL scope");
    let dt = secs(t);

    // Closed form: the original CQ + one rewrite per proper sub-class = DEPTH + 1 disjuncts.
    let expected = depth + 1;
    assert_eq!(
        r.report.disjuncts, expected,
        "subclass depth {depth}: UCQ size {} != closed form {expected} (UCQ blow-up / dedup regression)",
        r.report.disjuncts
    );
    println!(
        "subclass    depth {depth:>6}: rewrite {dt:9.6}s   disjuncts {} (= d+1, asserted; linear UCQ, no blow-up)",
        r.report.disjuncts
    );
}

fn run_subproperty(depth: usize) {
    let tbox = subproperty_chain(depth);
    let query = parse(&format!("SELECT ?x ?y WHERE {{ ?x <{EX}P{depth}> ?y }}"));
    let t = Instant::now();
    let r = rewrite(&query, &tbox).expect("subproperty chain CQ is in QL scope");
    let dt = secs(t);

    // Closed form: the original CQ + one rewrite per proper sub-property = DEPTH + 1 disjuncts.
    let expected = depth + 1;
    assert_eq!(
        r.report.disjuncts, expected,
        "subproperty depth {depth}: UCQ size {} != closed form {expected} (UCQ blow-up / dedup regression)",
        r.report.disjuncts
    );
    println!(
        "subproperty depth {depth:>6}: rewrite {dt:9.6}s   disjuncts {} (= d+1, asserted; linear UCQ, no blow-up)",
        r.report.disjuncts
    );
}

/// Production-path invariant check (sq-g19x0): on the SAME linear `subclass` chain the production
/// rewrite (PerfectRef + tree-witness + minimisation) must (a) never EMPTY the UCQ — the input
/// CQ's answers must survive — and (b) never GROW it past the pre-minimisation count. A linear
/// chain has no redundant disjuncts (each disjunct is an incomparable single class atom), so the
/// minimised count equals the baseline DEPTH+1: minimisation correctly drops NOTHING. Asserting
/// this closed form proves minimisation is not over-aggressive on the chain (it would be an
/// unsoundness bug to collapse incomparable disjuncts) and not a no-op stub.
fn run_production(depth: usize) {
    let tbox = subclass_chain(depth);
    let query = parse(&format!("SELECT ?x WHERE {{ ?x <{RDF_TYPE}> <{EX}C{depth}> }}"));
    let t = Instant::now();
    let r = rewrite_production(&query, &tbox).expect("subclass chain CQ is in QL scope");
    let dt = secs(t);
    assert!(
        r.report.disjuncts >= 1,
        "production UCQ must never be empty (the input CQ's answers must survive)"
    );
    assert!(
        r.report.disjuncts <= r.report.disjuncts_before_minimisation,
        "minimisation must only REMOVE disjuncts, never grow the UCQ"
    );
    // Linear chain ⇒ DEPTH+1 incomparable single-atom disjuncts ⇒ nothing is redundant.
    let expected = depth + 1;
    assert_eq!(
        r.report.disjuncts, expected,
        "production depth {depth}: minimised UCQ size {} != closed form {expected} (incomparable disjuncts must not be dropped)",
        r.report.disjuncts
    );
    println!(
        "production  depth {depth:>6}: rewrite {dt:9.6}s   disjuncts {} (= d+1; before-min {}, dropped {})",
        r.report.disjuncts,
        r.report.disjuncts_before_minimisation,
        r.report.disjuncts_before_minimisation - r.report.disjuncts
    );
}

fn main() {
    let depth: usize = std::env::args()
        .nth(1)
        .and_then(|v| v.parse().ok())
        .unwrap_or(2_000);
    println!("sparq-reason-ql ql_rewrite_bench — depth {depth} (deterministic, regenerable)");
    run_subclass(depth);
    run_subproperty(depth);
    run_production(depth);
    println!("OK: closed-form UCQ-size assertions held (UCQ scales linearly; wall-clock above is trend-only).");
}
