//! [OPUS-4.8] Multi-window joins (sq-9u1, part 2): an RSP-QL query opening more
//! than one named window and JOINing across them, evaluated correctly over the
//! per-window contents at each synchronized evaluation tick.

use oxrdf::{Literal, NamedNode, Term};
use sparq_rsp::{ContinuousMultiQuery, WindowResult};

fn nn(s: &str) -> NamedNode {
    NamedNode::new_unchecked(format!("http://ex/{s}"))
}

fn iri(s: &str) -> Term {
    nn(s).into()
}

/// `<http://ex/{s}> <http://ex/{p}> {o}`
fn t(s: &str, p: &str, o: Term) -> [Term; 3] {
    [iri(s), iri(p), o]
}

const Q_JOIN: &str = "\
REGISTER STREAM <http://ex/out> AS
SELECT ?room ?v WHERE {
  WINDOW <http://ex/w1> { ?s <http://ex/value> ?v }
  WINDOW <http://ex/w2> { ?s <http://ex/in> ?room }
}
FROM NAMED WINDOW <http://ex/w1> ON <http://ex/temp> RANGE 10 STEP 10
FROM NAMED WINDOW <http://ex/w2> ON <http://ex/meta> RANGE 10 STEP 10";

/// A row `(room, v)` as the join would emit it.
fn row(room: &str, v: i32) -> Vec<Option<Term>> {
    vec![Some(iri(room)), Some(Literal::from(v).into())]
}

/// THE multi-window join: a sensor's READINGS on stream :temp (window w1) joined
/// with the ROOM that sensor is in on stream :meta (window w2), per synchronized
/// tumbling window. Pins correct results over a synthetic 2-stream input.
#[test]
fn multi_window_join_across_two_streams() {
    let mut q = ContinuousMultiQuery::register(Q_JOIN).unwrap();
    assert_eq!(q.window_iris().len(), 2);
    assert_eq!(q.output_stream().unwrap().as_str(), "http://ex/out");

    let temp = nn("temp");
    let meta = nn("meta");
    let mut results: Vec<WindowResult> = Vec::new();

    // --- window [0,10) ---
    // meta: s1 is in the kitchen, s2 is in the hall.
    q.push(&meta, t("s1", "in", iri("kitchen")), 1, |r| results.push(r)).unwrap();
    q.push(&meta, t("s2", "in", iri("hall")), 2, |r| results.push(r)).unwrap();
    // temp: s1 reads 21 and 23; s2 reads 30.
    q.push(&temp, t("s1", "value", Literal::from(21).into()), 3, |r| results.push(r)).unwrap();
    q.push(&temp, t("s1", "value", Literal::from(23).into()), 4, |r| results.push(r)).unwrap();
    q.push(&temp, t("s2", "value", Literal::from(30).into()), 5, |r| results.push(r)).unwrap();

    // A temp reading at ts 12 advances the SHARED clock to 12, closing [0,10) on
    // BOTH windows even though no meta arrived after ts 2.
    q.push(&temp, t("s1", "value", Literal::from(99).into()), 12, |r| results.push(r)).unwrap();

    assert_eq!(results.len(), 1, "one synchronized tick closed [0,10)");
    let mut got: Vec<_> = results[0].rows.clone();
    got.sort_by_key(|r| format!("{r:?}"));
    let mut want = vec![row("kitchen", 21), row("kitchen", 23), row("hall", 30)];
    want.sort_by_key(|r| format!("{r:?}"));
    assert_eq!(
        got, want,
        "each reading joins with its sensor's room ACROSS the two windows"
    );
    assert_eq!(
        results[0].vars.iter().map(|v| v.as_str()).collect::<Vec<_>>(),
        ["room", "v"]
    );

    // --- flush: window [10,20) holds only the ts-12 reading; no meta in it, so
    // the join produces NOTHING (w2's window is empty → GRAPH yields no rows). ---
    results.clear();
    q.flush(|r| results.push(r)).unwrap();
    let total_rows: usize = results.iter().map(|r| r.rows.len()).sum();
    assert_eq!(total_rows, 0, "no meta in [10,20): the join is empty");
}

/// Sanity: the join is genuinely a CROSS-WINDOW join, not a per-window union.
/// A reading whose sensor has NO room in the meta window does not join; a room
/// whose sensor has NO reading does not join.
#[test]
fn multi_window_join_only_matches_shared_subject() {
    let mut q = ContinuousMultiQuery::register(Q_JOIN).unwrap();
    let temp = nn("temp");
    let meta = nn("meta");
    let mut results: Vec<WindowResult> = Vec::new();

    // s1 has a room but no reading; s2 has a reading but no room.
    q.push(&meta, t("s1", "in", iri("kitchen")), 1, |r| results.push(r)).unwrap();
    q.push(&temp, t("s2", "value", Literal::from(30).into()), 2, |r| results.push(r)).unwrap();
    q.flush(|r| results.push(r)).unwrap();

    let total_rows: usize = results.iter().map(|r| r.rows.len()).sum();
    assert_eq!(total_rows, 0, "no shared subject → empty join");
}

/// Two windows of DIFFERENT step over the same conceptual time: a slow window's
/// content is held steady across the fast window's ticks (the snapshot rule).
#[test]
fn multi_window_join_with_differing_steps() {
    // w1 (readings) tumbles every 10; w2 (rooms) tumbles every 20, so w2's
    // content spans two of w1's ticks.
    let q_text = "\
REGISTER STREAM <http://ex/out> AS
SELECT ?room ?v WHERE {
  WINDOW <http://ex/w1> { ?s <http://ex/value> ?v }
  WINDOW <http://ex/w2> { ?s <http://ex/in> ?room }
}
FROM NAMED WINDOW <http://ex/w1> ON <http://ex/temp> RANGE 10 STEP 10
FROM NAMED WINDOW <http://ex/w2> ON <http://ex/meta> RANGE 20 STEP 20";
    let mut q = ContinuousMultiQuery::register(q_text).unwrap();
    let temp = nn("temp");
    let meta = nn("meta");
    let mut results: Vec<WindowResult> = Vec::new();

    // meta: s1 in kitchen at ts 1 (lands in w2's [0,20)).
    q.push(&meta, t("s1", "in", iri("kitchen")), 1, |r| results.push(r)).unwrap();
    // temp: s1 reads 21 at ts 2 (w1's [0,10)) and 23 at ts 13 (w1's [10,20)).
    q.push(&temp, t("s1", "value", Literal::from(21).into()), 2, |r| results.push(r)).unwrap();
    q.push(&temp, t("s1", "value", Literal::from(23).into()), 13, |r| results.push(r)).unwrap();
    // Advance the shared clock to 22: closes w1's [0,10) AND [10,20), and w2's [0,20).
    q.push(&temp, t("s1", "value", Literal::from(99).into()), 22, |r| results.push(r)).unwrap();

    // Collect all emitted rows tagged with their tick end.
    let mut by_tick: Vec<(u64, Vec<Vec<Option<Term>>>)> =
        results.iter().map(|r| (r.end, r.rows.clone())).collect();
    by_tick.sort_by_key(|(e, _)| *e);

    // Tick at end=10: w1 has reading 21; w2 (slow, not yet closed at boundary 10)
    // has NO closed content → empty named graph → no join rows.
    // Tick at end=20: w1 has reading 23 ([10,20)); w2's [0,20) closed with the
    // kitchen fact → join: (kitchen, 23).
    let tick20 = by_tick.iter().find(|(e, _)| *e == 20).map(|(_, r)| r.clone());
    assert_eq!(tick20, Some(vec![row("kitchen", 23)]), "w2 content joins w1's [10,20)");

    // The end=10 tick must NOT have a join row (w2 had no closed content yet).
    let tick10 = by_tick.iter().find(|(e, _)| *e == 10).map(|(_, r)| r.clone());
    assert_eq!(tick10, Some(vec![]), "slow window not closed at boundary 10 → empty join");
}

/// Registration errors: a single-window RSP-QL query (use ContinuousQuery), a
/// WHERE WINDOW with no declaration, and a non-SELECT body.
#[test]
fn multi_query_registration_validates() {
    // Only one window declared.
    let one = "SELECT * WHERE { WINDOW <http://ex/w> { ?s ?p ?o } }\n\
               FROM NAMED WINDOW <http://ex/w> ON <http://ex/s> RANGE 10";
    assert!(ContinuousMultiQuery::register(one).is_err());

    // WHERE references an undeclared window.
    let undeclared = "SELECT * WHERE {\n\
                        WINDOW <http://ex/w1> { ?s ?p ?o }\n\
                        WINDOW <http://ex/wX> { ?s ?q ?r }\n\
                      }\n\
                      FROM NAMED WINDOW <http://ex/w1> ON <http://ex/s1> RANGE 10\n\
                      FROM NAMED WINDOW <http://ex/w2> ON <http://ex/s2> RANGE 10";
    assert!(ContinuousMultiQuery::register(undeclared).is_err());

    // Non-SELECT.
    let ask = "ASK { WINDOW <http://ex/w1> { ?s ?p ?o } WINDOW <http://ex/w2> { ?s ?q ?r } }\n\
               FROM NAMED WINDOW <http://ex/w1> ON <http://ex/s1> RANGE 10\n\
               FROM NAMED WINDOW <http://ex/w2> ON <http://ex/s2> RANGE 10";
    assert!(ContinuousMultiQuery::register(ask).is_err());
}
