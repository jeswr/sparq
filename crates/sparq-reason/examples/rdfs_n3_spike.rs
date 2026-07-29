//! [OPUS-5] sq-zgbso.6 (epic sq-zgbso, issue #1582, Phase 4) — the MEASURED SPIKE that
//! decides whether RDFS (and then OWL-RL) should be represented as Notation3 rule text
//! compiled through the `compiled-rules` evaluator instead of the hand-optimized
//! id-level materializer in `src/rdfs.rs`.
//!
//! # The bar, stated up front
//!
//! `sparq-reason` ALREADY has a hand-optimized id-level RDFS materializer
//! ([`sparq_reason::materialize`] with [`Profile::Rdfs`]) which saturates the schema
//! once and then sweeps the ABox in parallel, driving the shared
//! `sparq_substrate::join` kernels under the `substrate-join` feature. A compiled-N3
//! path has to MATCH or BEAT it to justify a swap; the plausible win is a
//! single-source-of-truth rule text and less bespoke Rust, **not** speed. This example
//! measures, it does not assume, and it asserts result-equivalence BEFORE it reports a
//! single timing.
//!
//! # What it does
//!
//! 1. Compiles `rules/rdfs.n3` — the same six rules (rdfs2/3/5/7/9/11) `src/rdfs.rs`
//!    implements — through [`sparq_reason::n3::compiled::compile`].
//! 2. **Entailment-conformance corpus** (§A): one fixture per rule plus the rule
//!    interactions the RDFS conformance shapes exercise. Both closures must be
//!    SET-EQUAL — a hard assert, so the example cannot report a ratio for a path that
//!    computes a different answer.
//! 3. **Scale corpora** (§B): a deep-taxonomy shape (rdfs9/rdfs11 heavy) and a
//!    schema+ABox shape (rdfs2/3/7 heavy). Set-equality is asserted first, then each
//!    path is timed best-of-N and the ratio reported.
//! 4. **Envelope probes** (§C): meta-level shapes where a rule DERIVES a schema triple
//!    (or retypes through `rdf:type`). The materializer saturates its schema from the
//!    ASSERTED triples only and routes `rdf:type` down an exclusive branch, so it is a
//!    single pass, not a fixpoint, over these. Reported, never asserted — the point of
//!    a spike is to state the envelope the equivalence in §A/§B actually holds over.
//!
//! **All wall-clock output is NON-CANONICAL** — work-box measurements for the bead/PR
//! comment only. Nothing here feeds docs, tests, or `bench/perf-baseline.json`, and this
//! example changes no shipping path: `rules/rdfs.n3` is read by nothing else.
//!
//! Run: `cargo run -p sparq-reason --example rdfs_n3_spike --features compiled-rules
//! --release` (add `--features compiled-rules,substrate-join` for the materializer's
//! shared-join feature state).

use sparq_core::dict::{Dict, Id};
use sparq_reason::n3::compiled::{compile, intern_facts, CompiledRuleSet};
use sparq_reason::{materialize, Profile};
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::time::Instant;

/// The rule text under test, embedded at build time (the epic's compiled-in posture).
const RDFS_N3: &str = include_str!("../rules/rdfs.n3");

const ITERS: usize = 5;

// ---------------------------------------------------------------------------------
// Closure helpers — both paths start from the SAME id-level facts in the SAME `Dict`,
// which is the situation a real swap would face.
// ---------------------------------------------------------------------------------

/// Render a closure as dictionary-independent term strings, so the two paths can be
/// compared as sets even though each interns its own vocabulary into its own clone.
fn as_strings(dict: &Dict, ids: &[[Id; 3]]) -> BTreeSet<String> {
    ids.iter()
        .map(|t| format!("{} {} {}", dict.term(t[0]), dict.term(t[1]), dict.term(t[2])))
        .collect()
}

/// The SHIPPING path: `materialize(Profile::Rdfs, …)` over a copy of the facts.
fn hand_closure(dict: &Dict, facts: &[[Id; 3]]) -> BTreeSet<String> {
    let mut d = dict.clone();
    let mut t = facts.to_vec();
    materialize(Profile::Rdfs, &mut d, &mut t);
    as_strings(&d, &t)
}

/// The COMPILED-N3 path: bind the rule symbols into the dictionary, run the fixpoint.
fn compiled_closure(dict: &Dict, facts: &[[Id; 3]], rules: &CompiledRuleSet) -> BTreeSet<String> {
    let mut d = dict.clone();
    let out = rules.bind(&mut d).eval(&mut d, facts);
    as_strings(&d, &out)
}

/// Set-difference report used by both the hard asserts (§A/§B) and the probes (§C).
fn diff(hand: &BTreeSet<String>, comp: &BTreeSet<String>) -> (Vec<String>, Vec<String>) {
    (
        hand.difference(comp).cloned().collect(),
        comp.difference(hand).cloned().collect(),
    )
}

fn assert_equivalent(what: &str, hand: &BTreeSet<String>, comp: &BTreeSet<String>) {
    let (hand_only, comp_only) = diff(hand, comp);
    assert!(
        hand_only.is_empty() && comp_only.is_empty(),
        "{what}: compiled-N3 closure diverges from materialize(Profile::Rdfs)\n  \
         materializer-only ({} of {}): {:#?}\n  compiled-only ({} of {}): {:#?}",
        hand_only.len(),
        hand.len(),
        &hand_only[..hand_only.len().min(20)],
        comp_only.len(),
        comp.len(),
        &comp_only[..comp_only.len().min(20)],
    );
}

// ---------------------------------------------------------------------------------
// Timing — the state each iteration mutates is rebuilt OUTSIDE the timed region.
// ---------------------------------------------------------------------------------

fn time_hand(dict: &Dict, facts: &[[Id; 3]]) -> (f64, usize) {
    let (mut best, mut n) = (f64::INFINITY, 0);
    for _ in 0..ITERS {
        let mut d = dict.clone();
        let mut t = facts.to_vec();
        let t0 = Instant::now();
        materialize(Profile::Rdfs, &mut d, &mut t);
        best = best.min(t0.elapsed().as_secs_f64() * 1e3);
        n = t.len();
    }
    (best, n)
}

fn time_compiled(dict: &Dict, facts: &[[Id; 3]], rules: &CompiledRuleSet) -> (f64, usize) {
    let (mut best, mut n) = (f64::INFINITY, 0);
    for _ in 0..ITERS {
        // `bind` stays INSIDE the timed region: a swapped-in compiled path pays it per
        // call, exactly as `compiled_rules_bench` charges it.
        let mut d = dict.clone();
        let t0 = Instant::now();
        let out = rules.bind(&mut d).eval(&mut d, facts);
        best = best.min(t0.elapsed().as_secs_f64() * 1e3);
        n = out.len();
    }
    (best, n)
}

// ---------------------------------------------------------------------------------
// §A — entailment-conformance corpus: one fixture per rule + the interactions.
// ---------------------------------------------------------------------------------

const PREFIXES: &str = "@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
                        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
                        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
                        @prefix ex: <http://ex/> .\n";

/// `(name, facts)` — each fixture is a complete N3 fact document.
fn conformance_corpus() -> Vec<(&'static str, String)> {
    let f = |body: &str| format!("{PREFIXES}{body}");
    vec![
        (
            "rdfs2 domain",
            f("ex:p rdfs:domain ex:C . ex:a ex:p ex:b ."),
        ),
        ("rdfs3 range", f("ex:p rdfs:range ex:C . ex:a ex:p ex:b .")),
        (
            "rdfs5 subPropertyOf transitivity",
            f("ex:p rdfs:subPropertyOf ex:q . ex:q rdfs:subPropertyOf ex:r .\n\
               ex:r rdfs:subPropertyOf ex:s ."),
        ),
        (
            "rdfs7 subPropertyOf rewrite",
            f("ex:p rdfs:subPropertyOf ex:q . ex:a ex:p ex:b ."),
        ),
        (
            "rdfs9 subClassOf typing",
            f("ex:C rdfs:subClassOf ex:D . ex:a rdf:type ex:C ."),
        ),
        (
            "rdfs11 subClassOf transitivity",
            f("ex:C rdfs:subClassOf ex:D . ex:D rdfs:subClassOf ex:E .\n\
               ex:E rdfs:subClassOf ex:F ."),
        ),
        (
            "rdfs9 x rdfs11 (transitive typing)",
            f("ex:Dog rdfs:subClassOf ex:Mammal . ex:Mammal rdfs:subClassOf ex:Animal .\n\
               ex:rex rdf:type ex:Dog ."),
        ),
        (
            "rdfs7 x rdfs2 (rewrite then domain typing)",
            f("ex:p rdfs:subPropertyOf ex:q . ex:q rdfs:domain ex:C . ex:s ex:p ex:o ."),
        ),
        (
            "rdfs7 x rdfs3 x rdfs9 (rewrite, range typing, subclass)",
            f("ex:p rdfs:subPropertyOf ex:q . ex:q rdfs:range ex:C .\n\
               ex:C rdfs:subClassOf ex:D . ex:s ex:p ex:o ."),
        ),
        (
            "rdfs2+rdfs3 through a subPropertyOf chain",
            f("ex:p rdfs:subPropertyOf ex:q . ex:q rdfs:subPropertyOf ex:r .\n\
               ex:r rdfs:domain ex:C . ex:r rdfs:range ex:D . ex:a ex:p ex:b ."),
        ),
        (
            "shared subject/object (rdfs2 and rdfs3 on one term)",
            f("ex:p rdfs:domain ex:C . ex:p rdfs:range ex:D . ex:a ex:p ex:a ."),
        ),
        (
            "literal object typed by rdfs3",
            f("ex:age rdfs:range xsd:integer . ex:a ex:age \"42\"^^xsd:integer ."),
        ),
        (
            "diamond subclass hierarchy",
            f("ex:A rdfs:subClassOf ex:B . ex:A rdfs:subClassOf ex:C .\n\
               ex:B rdfs:subClassOf ex:D . ex:C rdfs:subClassOf ex:D .\n\
               ex:x rdf:type ex:A ."),
        ),
        (
            "cyclic subClassOf (equivalence by mutual subsumption)",
            f("ex:A rdfs:subClassOf ex:B . ex:B rdfs:subClassOf ex:A . ex:x rdf:type ex:A ."),
        ),
        (
            "cyclic subPropertyOf",
            f("ex:p rdfs:subPropertyOf ex:q . ex:q rdfs:subPropertyOf ex:p . ex:a ex:p ex:b ."),
        ),
        (
            "no entailment (nothing to fire)",
            f("ex:a ex:p ex:b . ex:c ex:q ex:d ."),
        ),
        (
            "schema-only (no ABox)",
            f("ex:C rdfs:subClassOf ex:D . ex:p rdfs:subPropertyOf ex:q .\n\
               ex:p rdfs:domain ex:C . ex:q rdfs:range ex:D ."),
        ),
    ]
}

// ---------------------------------------------------------------------------------
// §B — scale corpora.
// ---------------------------------------------------------------------------------

/// Deep-taxonomy shape: a `depth`-long `rdfs:subClassOf` chain with `individuals`
/// members at the bottom. Stresses rdfs11 (quadratic schema closure) and rdfs9
/// (every individual typed at every level).
fn taxonomy_facts(depth: usize, individuals: usize) -> String {
    let mut s = String::from(PREFIXES);
    for i in 0..depth {
        let _ = writeln!(s, "ex:C{i} rdfs:subClassOf ex:C{}.", i + 1);
    }
    for k in 0..individuals {
        let _ = writeln!(s, "ex:i{k} rdf:type ex:C0.");
    }
    s
}

/// Schema+ABox shape: a three-level class tree, properties with domain/range and a
/// two-level subPropertyOf chain, and a wide ABox of property assertions. Stresses the
/// rdfs2/3/7 predicate join — the branch the materializer batches through the shared
/// substrate join kernels.
fn schema_abox_facts(
    tops: usize,
    mids: usize,
    leaves: usize,
    props: usize,
    assertions: usize,
) -> String {
    let mut s = String::from(PREFIXES);
    let mut leaf_classes = Vec::new();
    for t in 0..tops {
        let _ = writeln!(s, "ex:T{t} rdfs:subClassOf ex:Top.");
        for m in 0..mids {
            let _ = writeln!(s, "ex:M{t}_{m} rdfs:subClassOf ex:T{t}.");
            for l in 0..leaves {
                let _ = writeln!(s, "ex:L{t}_{m}_{l} rdfs:subClassOf ex:M{t}_{m}.");
                leaf_classes.push(format!("ex:L{t}_{m}_{l}"));
            }
        }
    }
    for p in 0..props {
        let dom = &leaf_classes[p % leaf_classes.len()];
        let rng = &leaf_classes[(p * 7 + 3) % leaf_classes.len()];
        let _ = writeln!(s, "ex:base{p} rdfs:domain {dom}.");
        let _ = writeln!(s, "ex:base{p} rdfs:range {rng}.");
        let _ = writeln!(s, "ex:p{p} rdfs:subPropertyOf ex:base{p}.");
        let _ = writeln!(s, "ex:sub{p} rdfs:subPropertyOf ex:p{p}.");
    }
    for a in 0..assertions {
        let p = a % props;
        let _ = writeln!(s, "ex:x{a} ex:sub{p} ex:y{a}.");
    }
    // A slice of directly-typed individuals so rdfs9 has asserted work too.
    for a in 0..assertions / 4 {
        let c = &leaf_classes[a % leaf_classes.len()];
        let _ = writeln!(s, "ex:x{a} rdf:type {c}.");
    }
    s
}

/// The shape most FAVOURABLE to a rule engine: a minimal schema (one subClassOf level,
/// no subPropertyOf chain) with an ABox-dominated fact set, so almost all the work is a
/// single flat join per rule and neither path has a deep fixpoint to grind. Included so
/// the verdict does not rest on hierarchy depth, which the materializer closes with a
/// BFS the rule engine has to reach round by round.
fn shallow_abox_facts(classes: usize, props: usize, assertions: usize) -> String {
    let mut s = String::from(PREFIXES);
    for c in 0..classes {
        let _ = writeln!(s, "ex:C{c} rdfs:subClassOf ex:Top.");
    }
    for p in 0..props {
        let _ = writeln!(s, "ex:p{p} rdfs:domain ex:C{}.", p % classes);
        let _ = writeln!(s, "ex:p{p} rdfs:range ex:C{}.", (p + 1) % classes);
    }
    for a in 0..assertions {
        let _ = writeln!(s, "ex:x{a} ex:p{} ex:y{a}.", a % props);
    }
    s
}

// ---------------------------------------------------------------------------------
// §C — envelope probes (REPORTED, never asserted).
// ---------------------------------------------------------------------------------

fn probe_corpus() -> Vec<(&'static str, String, &'static str)> {
    let f = |body: &str| format!("{PREFIXES}{body}");
    vec![
        (
            "derived schema edge (rdfs7 mints a subClassOf triple)",
            f("ex:sub rdfs:subPropertyOf rdfs:subClassOf .\n\
               ex:A ex:sub ex:B . ex:B rdfs:subClassOf ex:C . ex:x rdf:type ex:A ."),
            "rdfs7 derives (A subClassOf B); a fixpoint then closes it with rdfs11/rdfs9",
        ),
        (
            "derived rdf:type edge (rdfs7 rewrites INTO rdf:type)",
            f("ex:kind rdfs:subPropertyOf rdf:type .\n\
               ex:x ex:kind ex:C . ex:C rdfs:subClassOf ex:D ."),
            "rdfs7 derives (x type C); a fixpoint then applies rdfs9 to it",
        ),
        (
            "rdf:type carrying a domain (rdfs2 over a type assertion)",
            f("rdf:type rdfs:domain ex:Thing . ex:x rdf:type ex:C ."),
            "rdfs2 fires on the (x type C) assertion itself",
        ),
        (
            "rdfs:subClassOf carrying a domain",
            f("rdfs:subClassOf rdfs:domain ex:Class . ex:A rdfs:subClassOf ex:B ."),
            "rdfs2 fires on a schema triple",
        ),
        (
            "derived subPropertyOf edge (meta subPropertyOf)",
            f("ex:spo rdfs:subPropertyOf rdfs:subPropertyOf .\n\
               ex:p ex:spo ex:q . ex:a ex:p ex:b ."),
            "rdfs7 derives (p subPropertyOf q); a fixpoint then rewrites (a p b)",
        ),
    ]
}

// ---------------------------------------------------------------------------------

fn features() -> String {
    let mut on: Vec<&str> = vec!["compiled-rules"];
    if cfg!(feature = "parallel") {
        on.push("parallel");
    }
    if cfg!(feature = "substrate-join") {
        on.push("substrate-join");
    }
    on.join(",")
}

fn main() {
    println!(
        "rdfs_n3_spike (sq-zgbso.6) — compiled `rules/rdfs.n3` vs materialize(Profile::Rdfs).\n\
         features: {}  |  timings are NON-CANONICAL work-box measurements (best of {ITERS}).\n",
        features()
    );

    // ---- compile the ruleset ----------------------------------------------------
    let t0 = Instant::now();
    let rules = compile(RDFS_N3).expect("rules/rdfs.n3 must be inside the compiled subset");
    let compile_ms = t0.elapsed().as_secs_f64() * 1e3;
    println!(
        "compiled rules/rdfs.n3: {} rules, {} rule-document facts, {compile_ms:.3} ms \
         (once per process; a build-time compile would pay 0)\n",
        rules.n_rules(),
        rules.n_facts()
    );
    assert_eq!(rules.n_rules(), 6, "the RDFS ruleset is rdfs2/3/5/7/9/11");

    // ---- §A equivalence on the entailment-conformance corpus --------------------
    println!("§A entailment-conformance corpus (equivalence is a HARD assert):");
    let corpus = conformance_corpus();
    let mut total = 0usize;
    for (name, facts) in &corpus {
        let mut dict = Dict::new();
        let ids = intern_facts(&mut dict, facts).expect("fixture facts must parse");
        let hand = hand_closure(&dict, &ids);
        let comp = compiled_closure(&dict, &ids, &rules);
        assert_equivalent(name, &hand, &comp);
        total += hand.len();
        println!("  ok  {name} — {} closure triples", hand.len());
    }
    println!(
        "  => {} fixtures equivalent, {total} closure triples compared\n",
        corpus.len()
    );

    // ---- §B scale: equivalence first, then the ratio -----------------------------
    println!("§B scale corpora (equivalence asserted BEFORE any timing):");
    let scales: Vec<(String, String)> = vec![
        ("deep taxonomy (chain 120, 200 individuals)".into(), taxonomy_facts(120, 200)),
        (
            "schema+ABox (84 leaf classes, 24 properties, 20k assertions)".into(),
            schema_abox_facts(4, 4, 4, 24, 20_000),
        ),
        (
            "shallow schema, ABox-dominated (16 classes, 16 properties, 40k assertions)".into(),
            shallow_abox_facts(16, 16, 40_000),
        ),
    ];
    let mut verdict_rows: Vec<(String, f64, f64)> = Vec::new();
    for (name, facts) in &scales {
        let mut dict = Dict::new();
        let ids = intern_facts(&mut dict, facts).expect("scale facts must parse");
        let hand = hand_closure(&dict, &ids);
        let comp = compiled_closure(&dict, &ids, &rules);
        assert_equivalent(name, &hand, &comp);
        println!(
            "  ok  {name}: {} input triples -> {} closure triples, closures identical",
            ids.len(),
            hand.len()
        );

        let (hand_ms, hand_n) = time_hand(&dict, &ids);
        let (comp_ms, comp_n) = time_compiled(&dict, &ids, &rules);
        assert_eq!(hand_n, comp_n, "{name}: closure sizes must agree");
        println!(
            "      materializer {hand_ms:.2} ms | compiled-N3 {comp_ms:.2} ms | \
             compiled/materializer {:.2}x",
            comp_ms / hand_ms
        );
        verdict_rows.push((name.clone(), hand_ms, comp_ms));
    }
    println!();

    // ---- §C envelope probes (report only) ----------------------------------------
    println!(
        "§C envelope probes — meta-level shapes where a rule DERIVES a schema triple.\n\
         The materializer saturates its schema from the ASSERTED triples once (src/rdfs.rs\n\
         `rdfs_closure` step 1-2) and routes rdf:type down an exclusive branch, so it is a\n\
         single pass, not a fixpoint, over these. REPORTED, not asserted:"
    );
    let probes = probe_corpus();
    let mut divergences = 0usize;
    for (name, facts, why) in &probes {
        let mut dict = Dict::new();
        let ids = intern_facts(&mut dict, facts).expect("probe facts must parse");
        let hand = hand_closure(&dict, &ids);
        let comp = compiled_closure(&dict, &ids, &rules);
        let (hand_only, comp_only) = diff(&hand, &comp);
        if hand_only.is_empty() && comp_only.is_empty() {
            println!("  same  {name}");
        } else {
            divergences += 1;
            println!("  DIFF  {name} — {why}");
            for t in &comp_only {
                println!("          compiled-N3 derives, materializer does not: {t}");
            }
            for t in &hand_only {
                println!("          materializer derives, compiled-N3 does not: {t}");
            }
        }
    }
    println!(
        "  => {divergences} of {} probes diverge (outside the §A/§B equivalence envelope)\n",
        probes.len()
    );

    // ---- verdict inputs ----------------------------------------------------------
    let worst = verdict_rows
        .iter()
        .map(|(_, h, c)| c / h)
        .fold(f64::NEG_INFINITY, f64::max);
    println!(
        "VERDICT INPUTS (perf rule: GO only if compiled-N3 matches or beats the materializer)"
    );
    for (name, h, c) in &verdict_rows {
        println!("  {name}: {:.2}x", c / h);
    }
    println!(
        "  worst compiled/materializer ratio: {worst:.2}x -> perf axis {}",
        if worst <= 1.0 { "GO" } else { "NO-GO" }
    );
    println!(
        "  semantic note: {divergences} meta-level probe(s) where the two paths differ — \
         a compiled-N3 swap would CHANGE the closure on those shapes."
    );
    println!("\nNON-CANONICAL: work-box measurements, for the bead/PR comment only.");
}
