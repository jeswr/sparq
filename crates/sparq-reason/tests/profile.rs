//! [SONNET-4.6] sq-6tykl.5 — the reasoning profiler, end to end: per-rule-group cost, the
//! per-round materialization progress monitor, and the top-N offender summary.
#![cfg(feature = "profile")]
use sparq_core::dict::{Dict, Id};
use sparq_reason::profile::{rules, Profiler, Progress, TickKind};
use sparq_reason::{materialize, materialize_profiled, materialize_profiled_with, Profile};
use std::sync::{Arc, Mutex};

/// A transitive-property chain: the classic closure blow-up (an N-chain closes to O(N²)).
fn fixture() -> (Dict, Vec<[Id; 3]>) {
    let mut d = Dict::new();
    let ty = d.intern_iri(oxrdf::vocab::rdf::TYPE.as_str());
    let transitive = d.intern_iri("http://www.w3.org/2002/07/owl#TransitiveProperty");
    let p = d.intern_iri("http://example.com/path");
    let ns: Vec<_> = (0..12)
        .map(|i| d.intern_iri(&format!("http://example.com/n{}", i)))
        .collect();
    let mut ts = vec![[p, ty, transitive]];
    ts.extend(ns.windows(2).map(|w| [w[0], p, w[1]]));
    (d, ts)
}

/// An RDFS subclass hierarchy over some typed individuals (single-pass materializer).
fn rdfs_fixture() -> (Dict, Vec<[Id; 3]>) {
    let mut d = Dict::new();
    let ty = d.intern_iri(oxrdf::vocab::rdf::TYPE.as_str());
    let sc = d.intern_iri(oxrdf::vocab::rdfs::SUB_CLASS_OF.as_str());
    let cs: Vec<_> = (0..6)
        .map(|i| d.intern_iri(&format!("http://example.com/C{}", i)))
        .collect();
    let mut ts: Vec<[Id; 3]> = cs.windows(2).map(|w| [w[0], sc, w[1]]).collect();
    for i in 0..8 {
        let x = d.intern_iri(&format!("http://example.com/x{}", i));
        ts.push([x, ty, cs[0]]);
    }
    (d, ts)
}

/// THE headline guard: profiling neither changes the closure nor the count, and the rule group
/// that generated the blow-up is the top offender, with the fixpoint's rounds attributed.
#[test]
fn explosive_rule_is_top_without_changing_closure() {
    let (mut da, mut a) = fixture();
    let (mut db, mut b) = fixture();
    let plain = materialize(Profile::OwlRl, &mut da, &mut a);
    let (profiled, report) = materialize_profiled(Profile::OwlRl, &mut db, &mut b);

    assert_eq!(plain, profiled, "instrumentation must not change the fact count");
    assert_eq!(
        a, b,
        "instrumentation must not change the closure — element for element, not just as a set"
    );

    // prp-trp fires inside the fused semi-naive delta emitter, so that is the group that
    // emits the O(N²) candidate mass on a transitive chain.
    let top = report.top(1);
    assert_eq!(top.len(), 1);
    assert_eq!(
        top[0].rule,
        rules::DELTA_SWEEP,
        "the transitive chain's blow-up belongs to the delta sweep; got {:?}",
        report.stats()
    );
    assert!(
        top[0].derived_count > 40,
        "an 11-edge transitive chain emits far more than 40 candidates: {:?}",
        top[0]
    );
    // The fixpoint really iterated, and every round is attributed.
    assert!(report.rounds() > 1, "a transitive chain needs several rounds");
    assert_eq!(
        top[0].fired_count,
        report.rounds(),
        "the delta sweep runs exactly once per round"
    );
    assert_eq!(
        report.total_derived(),
        plain,
        "total_derived is the materializer's return value"
    );
    // The commit phase is separately attributed, so a slow commit is distinguishable from a
    // rule that emits too much.
    let commit = report
        .stats()
        .iter()
        .find(|s| s.rule == rules::COMMIT)
        .expect("the commit phase is measured");
    assert_eq!(commit.fired_count, report.rounds());
}

/// The progress monitor: one notification per round, monotone round numbers, and a running
/// total that lands exactly on the materializer's return value.
#[test]
fn progress_monitor_reports_every_round_of_a_materialization() {
    let (mut dict, mut triples) = fixture();
    let seen: Arc<Mutex<Vec<Progress>>> = Arc::default();
    let sink = Arc::clone(&seen);
    let (added, report) = materialize_profiled_with(
        Profiler::with_progress(move |p| sink.lock().unwrap().push(p)),
        Profile::OwlRl,
        &mut dict,
        &mut triples,
    );

    let seen = seen.lock().unwrap();
    assert_eq!(seen.len(), report.rounds(), "one notification per round");
    assert!(seen.len() > 1, "the fixture must exercise a real fixpoint");
    assert_eq!(
        seen.iter().map(|p| p.round).collect::<Vec<_>>(),
        (1..=seen.len()).collect::<Vec<_>>(),
        "round numbers are 1-based and gapless"
    );
    assert!(
        seen.windows(2).all(|w| w[0].total_derived <= w[1].total_derived),
        "the running total never decreases"
    );
    assert_eq!(
        seen.last().unwrap().total_derived,
        added,
        "the ticks account for the whole closure — this fixture has no owl:sameAs, so nothing \
         is committed outside a round (contrast `sameas_expansion_is_counted_in_total_derived`)"
    );
    // The final round of a fixpoint is the one that derives nothing — that is how it stops.
    assert_eq!(seen.last().unwrap().derived_count, 0);
}

/// The single-pass RDFS materializer is instrumented too, with its own phases.
#[test]
fn rdfs_materialization_attributes_its_phases() {
    let (mut da, mut a) = rdfs_fixture();
    let (mut db, mut b) = rdfs_fixture();
    let plain = materialize(Profile::Rdfs, &mut da, &mut a);
    let (profiled, report) = materialize_profiled(Profile::Rdfs, &mut db, &mut b);

    assert_eq!(plain, profiled);
    assert_eq!(a, b, "the RDFS closure is unchanged element for element");
    let names: Vec<_> = report.stats().iter().map(|s| s.rule).collect();
    for expected in [rules::SCHEMA_SATURATE, rules::ABOX_SWEEP, rules::FINALIZE] {
        assert!(names.contains(&expected), "{} missing from {:?}", expected, names);
    }
    assert_eq!(report.rounds(), 1, "the RDFS materializer is single-pass");
    assert_eq!(report.total_derived(), plain);
    let sweep = report.stats().iter().find(|s| s.rule == rules::ABOX_SWEEP).unwrap();
    assert!(
        sweep.derived_count >= plain,
        "the sweep emits at least the facts that survive dedup: {:?}",
        sweep
    );
}

/// Incremental maintenance is instrumented by the same profiler — this is the half of the bead
/// that batch materialization does not cover.
#[test]
fn incremental_maintenance_is_profiled() {
    use sparq_reason::MaterializedGraph;
    let mut dict = Dict::new();
    let ty = dict.intern_iri(oxrdf::vocab::rdf::TYPE.as_str());
    let sc = dict.intern_iri(oxrdf::vocab::rdfs::SUB_CLASS_OF.as_str());
    let dog = dict.intern_iri("http://example.com/Dog");
    let mammal = dict.intern_iri("http://example.com/Mammal");
    let animal = dict.intern_iri("http://example.com/Animal");
    let rex = dict.intern_iri("http://example.com/rex");
    let base = [[dog, sc, mammal], [mammal, sc, animal]];

    let mut graph = MaterializedGraph::new(&mut dict, &base);
    let ticks: Arc<Mutex<Vec<Progress>>> = Arc::default();
    let sink = Arc::clone(&ticks);
    // ONE asserted triple with SEVERAL consequences, then its deletion: the case that tells a
    // base-mutation count apart from a derived-fact count in both directions.
    let ((before, after_insert, after_delete), report) = sparq_reason::profile::with_profiler(
        Profiler::with_progress(move |p| sink.lock().unwrap().push(p)),
        || {
            let before = graph.closure().len();
            graph.insert(&[[rex, ty, dog]]);
            let after_insert = graph.closure().len();
            graph.delete(&[[rex, ty, dog]]);
            (before, after_insert, graph.closure().len())
        },
    );

    assert_eq!(before, 3, "2 asserted subClassOf + the entailed Dog ⊑ Animal");
    assert_eq!(
        after_insert - before,
        3,
        "the one asserted (rex type Dog) makes 3 closure facts visible: itself plus the \
         entailed Mammal and Animal types"
    );
    assert_eq!(
        after_delete, 3,
        "deleting the only ABox triple leaves the asserted + entailed TBox"
    );
    let names: Vec<_> = report.stats().iter().map(|s| s.rule).collect();
    assert!(names.contains(&rules::INCREMENTAL_INSERT), "{:?}", names);
    assert!(names.contains(&rules::INCREMENTAL_DELETE), "{:?}", names);
    let insert = report
        .stats()
        .iter()
        .find(|s| s.rule == rules::INCREMENTAL_INSERT)
        .unwrap();
    assert_eq!(insert.fired_count, 1);
    assert!(
        insert.derived_count >= 2,
        "(rex type Dog) entails Mammal and Animal: {:?}",
        insert
    );

    // The tick contract: an incremental tick reports the BASE MUTATION it processed, and says
    // nothing about derived facts. Pinned per tick so the two quantities cannot be conflated
    // again — the insert moved the closure by 3 while reporting 1, and the delete moved it by
    // -3 while also reporting 1, so neither number is a signed count of committed facts.
    let ticks = ticks.lock().unwrap();
    assert_eq!(ticks.len(), 2, "one tick per maintenance op");
    assert_eq!(ticks[0].kind, TickKind::Insert);
    assert_eq!(ticks[1].kind, TickKind::Delete);
    for t in ticks.iter() {
        assert_eq!(t.base_mutations, 1, "one base triple processed: {:?}", t);
        assert_eq!(
            t.derived_count, 0,
            "an incremental tick claims no committed facts: {:?}",
            t
        );
        assert_eq!(t.total_derived, 0, "and never moves the running total: {:?}", t);
    }
    assert_eq!(
        report.total_derived(),
        0,
        "the mutations cancelled out; total_derived must not end at 2 by counting the two \
         input triples as derived facts"
    );
    assert_eq!(report.rounds(), 2, "rounds() counts ticks, incremental ones included");
}

/// A TBox change falls back to a full re-materialization — the profiler names that, so a
/// workload that is secretly rebuilding on every mutation is visible rather than mysterious.
#[test]
fn a_tbox_insert_is_attributed_to_the_rebuild_fallback() {
    use sparq_reason::MaterializedGraph;
    let mut dict = Dict::new();
    let sc = dict.intern_iri(oxrdf::vocab::rdfs::SUB_CLASS_OF.as_str());
    let (a, b, c) = (
        dict.intern_iri("http://example.com/A"),
        dict.intern_iri("http://example.com/B"),
        dict.intern_iri("http://example.com/C"),
    );
    let mut graph = MaterializedGraph::new(&mut dict, &[[a, sc, b]]);
    let ticks: Arc<Mutex<Vec<Progress>>> = Arc::default();
    let sink = Arc::clone(&ticks);
    let (_, report) = sparq_reason::profile::with_profiler(
        Profiler::with_progress(move |p| sink.lock().unwrap().push(p)),
        || {
            graph.insert(&[[b, sc, c]]); // a TBox triple: forces the full rebuild path
        },
    );
    let rebuild = report
        .stats()
        .iter()
        .find(|s| s.rule == rules::INCREMENTAL_REBUILD)
        .unwrap_or_else(|| {
            panic!("the rebuild fallback must be attributed: {:?}", report.stats())
        });
    assert_eq!(rebuild.fired_count, 1);
    assert!(
        !report.stats().iter().any(|s| s.rule == rules::INCREMENTAL_INSERT),
        "a rebuilding insert must NOT also be counted as an incremental sweep"
    );
    // The documented progress promise for this path: a `MaterializedGraph` rebuild
    // re-materializes DIRECTLY (it replicates `rdfs_closure` rather than calling it), so it
    // emits no tick of its own and no round — `incremental-rebuild` above is the only signal
    // that anything happened. Pinned so the docs cannot drift back to claiming delegation.
    assert_eq!(
        ticks.lock().unwrap().len(),
        0,
        "a direct rebuild must not fabricate progress ticks"
    );
    assert_eq!(report.rounds(), 0, "and it contributes no rounds");
}

/// The one rebuild path that DOES tick: a `MaterializedOwlGraph` that has fallen back to
/// whole-graph OWL-RL delegates to the batch materializer, so that run's rounds are reported.
#[test]
fn an_owl_fallback_rebuild_ticks_the_batch_run_it_delegates_to() {
    use sparq_reason::MaterializedOwlGraph;
    let mut dict = Dict::new();
    let ty = dict.intern_iri(oxrdf::vocab::rdf::TYPE.as_str());
    let sc = dict.intern_iri(oxrdf::vocab::rdfs::SUB_CLASS_OF.as_str());
    let inter = dict.intern_iri("http://www.w3.org/2002/07/owl#intersectionOf");
    let first = dict.intern_iri(oxrdf::vocab::rdf::FIRST.as_str());
    let rest = dict.intern_iri(oxrdf::vocab::rdf::REST.as_str());
    let nil = dict.intern_iri(oxrdf::vocab::rdf::NIL.as_str());
    let iri = |d: &mut Dict, s: &str| d.intern_iri(s);
    let (ca, c1, c2) = (
        iri(&mut dict, "http://example.com/CA"),
        iri(&mut dict, "http://example.com/C1"),
        iri(&mut dict, "http://example.com/C2"),
    );
    let (l1, l2, x) = (
        iri(&mut dict, "http://example.com/l1"),
        iri(&mut dict, "http://example.com/l2"),
        iri(&mut dict, "http://example.com/x"),
    );
    let (d, e) = (
        iri(&mut dict, "http://example.com/D"),
        iri(&mut dict, "http://example.com/E"),
    );
    // owl:intersectionOf puts the graph in `Fallback` mode (no counting maintenance for it).
    let base = [
        [ca, inter, l1],
        [l1, first, c1],
        [l1, rest, l2],
        [l2, first, c2],
        [l2, rest, nil],
        [x, ty, c1],
        [x, ty, c2],
    ];
    let mut graph = MaterializedOwlGraph::new(&mut dict, &base);
    let ticks: Arc<Mutex<Vec<Progress>>> = Arc::default();
    let sink = Arc::clone(&ticks);
    let (_, report) = sparq_reason::profile::with_profiler(
        Profiler::with_progress(move |p| sink.lock().unwrap().push(p)),
        || {
            graph.insert(&mut dict, &[[d, sc, e]]); // TBox: forces the rebuild
        },
    );
    let names: Vec<_> = report.stats().iter().map(|s| s.rule).collect();
    assert!(names.contains(&rules::INCREMENTAL_REBUILD), "{:?}", names);
    assert!(
        names.contains(&rules::DELTA_SWEEP) && names.contains(&rules::COMMIT),
        "the rebuild encloses a whole nested batch run's groups: {:?}",
        names
    );
    let ticks = ticks.lock().unwrap();
    assert!(!ticks.is_empty(), "the delegated batch run reports its rounds");
    assert_eq!(ticks.len(), report.rounds(), "one notification per round");
}

/// `Report::total_derived` means the run's NET NEW FACTS — the materializer's return value —
/// even when most of them are created after the fixpoint closes. The OWL-RL fixpoint commits
/// only *canonical* facts; the owl:sameAs equivalence classes are expanded back over the
/// closure afterwards, so a round sum alone would report far too few (here: none at all).
#[test]
fn sameas_expansion_is_counted_in_total_derived() {
    let mut dict = Dict::new();
    let iri = |d: &mut Dict, s: &str| d.intern_iri(s);
    let same = iri(&mut dict, "http://www.w3.org/2002/07/owl#sameAs");
    let (p, q) = (
        iri(&mut dict, "http://example.com/p"),
        iri(&mut dict, "http://example.com/q"),
    );
    let (a, b, c) = (
        iri(&mut dict, "http://example.com/a"),
        iri(&mut dict, "http://example.com/b"),
        iri(&mut dict, "http://example.com/c"),
    );
    let z = iri(&mut dict, "http://example.com/z");
    // A three-member equivalence class {a, b, c} that two assertions actually mention, so the
    // expansion has real work: every triple over a class member is re-emitted for each member.
    let mut triples = vec![[a, same, b], [b, same, c], [a, p, z], [z, q, c]];

    let mut plain_dict = dict.clone();
    let mut plain_triples = triples.clone();
    let plain = materialize(Profile::OwlRl, &mut plain_dict, &mut plain_triples);

    let ticks: Arc<Mutex<Vec<Progress>>> = Arc::default();
    let sink = Arc::clone(&ticks);
    let (added, report) = materialize_profiled_with(
        Profiler::with_progress(move |t| sink.lock().unwrap().push(t)),
        Profile::OwlRl,
        &mut dict,
        &mut triples,
    );

    assert_eq!(plain, added, "instrumentation must not change the fact count");
    assert_eq!(plain_triples, triples, "nor the closure, element for element");
    assert!(added > 0, "the fixture must actually derive something");
    assert_eq!(
        report.total_derived(),
        added,
        "total_derived is the run's net-new-fact count, expansion included: {:?}",
        report.stats()
    );
    // The equality expansion is a phase, not a round — it must not inflate `rounds()`, and its
    // facts reach the report without a fabricated tick.
    let ticks = ticks.lock().unwrap();
    assert_eq!(ticks.len(), report.rounds(), "one notification per round");
    let round_sum: usize = ticks.iter().map(|t| t.derived_count).sum();
    assert_eq!(
        round_sum,
        ticks.last().map_or(0, |t| t.total_derived),
        "each tick's running total is the sum of the round counts before it"
    );
    assert!(
        round_sum < added,
        "this fixture's facts come from the expansion, so the rounds alone under-report \
         ({} of {}) — that gap is exactly what total_derived must close",
        round_sum,
        added
    );
    let expand = report
        .stats()
        .iter()
        .find(|s| s.rule == rules::SAMEAS_EXPAND)
        .expect("the expansion phase is measured");
    assert_eq!(expand.fired_count, 1, "expansion runs once, after the fixpoint");
    // The renderable surfaces carry the reconciled number, not the round sum.
    assert!(
        report.to_text_summary(3).contains(&format!("{} fact(s) derived", added)),
        "{}",
        report.to_text_summary(3)
    );
    assert!(
        report.to_json().contains(&format!("\"total_derived\":{}", added)),
        "{}",
        report.to_json()
    );
}

/// Every batch profile reports the same thing: `total_derived` is what `materialize` returned.
/// D-entailment is single-pass like RDFS, so it is one round.
#[cfg(feature = "d-entail")]
#[test]
fn d_entailment_reports_its_derived_count() {
    let mut dict = Dict::new();
    let p = dict.intern_iri("http://example.com/p");
    let s = dict.intern_iri("http://example.com/s");
    let lit = dict.intern_lit("42", "http://www.w3.org/2001/XMLSchema#integer", None);
    let mut triples = vec![[s, p, lit]];
    let (added, report) = materialize_profiled(Profile::D, &mut dict, &mut triples);
    assert!(added > 0, "rdfD1 types the recognized literal");
    assert_eq!(report.total_derived(), added);
    assert_eq!(report.rounds(), 1, "rdfD1 is single-pass");
}

/// The renderable surfaces a CLI / server / GUI consumes.
#[test]
fn summary_surfaces_render_the_measured_report() {
    let (mut dict, mut triples) = fixture();
    let (added, report) = materialize_profiled(Profile::OwlRl, &mut dict, &mut triples);

    let text = report.to_text_summary(3);
    assert!(text.starts_with("reasoning profile: "), "{}", text);
    assert!(
        text.contains(&format!("{} fact(s) derived", added)),
        "the summary states the run's real derived count: {}",
        text
    );
    assert_eq!(
        text.lines().count(),
        2 + 3,
        "header + column head + exactly top(3): {}",
        text
    );

    let json = report.to_json();
    assert!(
        json.contains(&format!("\"total_derived\":{}", added)),
        "{}",
        json
    );
    assert!(json.contains("\"rule\":\"delta-sweep\""), "{}", json);
    assert_eq!(
        json.matches("\"fired_count\"").count(),
        report.stats().len(),
        "every measured rule group reaches the machine-readable surface"
    );
}
