//! [SONNET-4.6] sq-6tykl.5 — the reasoning profiler, end to end: per-rule-group cost, the
//! per-round materialization progress monitor, and the top-N offender summary.
#![cfg(feature = "profile")]
use sparq_core::dict::{Dict, Id};
use sparq_reason::profile::{rules, Profiler, Progress};
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
        "the per-round committed counts must sum to the materializer's return value"
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
        "the last notification's total is the closure's new-fact count"
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
    let (closure_len, report) = sparq_reason::profile::with_profiler(
        Profiler::with_progress(move |p| sink.lock().unwrap().push(p)),
        || {
            graph.insert(&[[rex, ty, dog]]);
            graph.delete(&[[rex, ty, dog]]);
            graph.closure().len()
        },
    );

    assert_eq!(
        closure_len, 3,
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
    assert_eq!(ticks.lock().unwrap().len(), 2, "one tick per maintenance op");
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
    let (_, report) = sparq_reason::profile::with_profiler(Profiler::new(), || {
        graph.insert(&[[b, sc, c]]); // a TBox triple: forces the full rebuild path
    });
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
