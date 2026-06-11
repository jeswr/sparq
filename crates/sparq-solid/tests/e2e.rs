//! End-to-end: the SAME SPARQL query, run as different sessions, returns different
//! results — enforced by the engine's zero-copy dataset view (the default path),
//! with the v1 FROM-NAMED rewrite path kept as a differential oracle: both paths
//! must return byte-identical JSON for every fixture session.

use sparq_core::Graph;
use sparq_solid::fixture::{wac_fixture, ALICE, APP, BOB, CAROL};
use sparq_solid::{rewrite_for, Mode, PodStore, Session};

fn store() -> PodStore {
    let g = Graph::load_dataset(&wac_fixture(), "nquads").expect("fixture loads");
    let mut s = PodStore::new(g);
    s.materialize_wac().expect("materialize");
    s
}

/// Hand-computed readable-document counts for the WAC fixture (4 docs per leaf
/// container, 144 per depth-1 subtree; see tests/wac.rs for the per-subtree logic):
///   alice  = priv0 144 + pub1 144 + friends3 48* + mixed4 (144-24-1) + origin5 144 = 599
///           (*friends3's own ACL gives alice acl:accessTo ONLY — no acl:default — so
///            she reads just the two restated subtrees c0/c1; nearest-ACL fail-closed)
///   carol  = pub1 144 + team2 144 + mixed4 (144-24-24-1 = 95)** + mixed4/c0 24     = 407
///           (**minus c0 (carol reads it via the DEEP acl instead), minus c2 — its
///             own ACL is owner-only, so AuthenticatedAgent loses it — minus the
///             bob-only doc override)
///   anon   = pub1 144
const TITLES: &str = "SELECT ?title WHERE { ?s <https://ex.dev/ns#title> ?title }";

#[test]
fn same_query_different_agents_different_results() {
    let mut s = store();
    let alice = s.query_as(&Session { agent: Some(ALICE), client: None }, Mode::Read, TITLES).unwrap();
    let carol = s.query_as(&Session { agent: Some(CAROL), client: None }, Mode::Read, TITLES).unwrap();
    let anon = s.query_as(&Session::default(), Mode::Read, TITLES).unwrap();
    assert_eq!(alice.rows.len(), 599, "alice sees her documents");
    assert_eq!(carol.rows.len(), 407, "carol sees group + public + deep-override docs");
    assert_eq!(anon.rows.len(), 144, "anonymous sees only the public subtree");
    // the kept v1 portability path (FROM NAMED rewrite) returns the same counts
    let v1 = s.query_as_rewrite(&Session { agent: Some(ALICE), client: None }, Mode::Read, TITLES).unwrap();
    assert_eq!(v1.rows.len(), 599);
}

#[test]
fn graph_patterns_and_cross_document_joins_stay_inside_the_sandbox() {
    let mut s = store();
    // explicit GRAPH ?g: ranges over authorized graphs only (FROM NAMED injection)
    let q = "SELECT ?g WHERE { GRAPH ?g { ?s <https://ex.dev/ns#title> ?title } }";
    let anon = s.query_as(&Session::default(), Mode::Read, q).unwrap();
    assert_eq!(anon.rows.len(), 144);
    // a cross-document join: doc graph (inSubtree) ⋈ container graph (ldp:contains);
    // carol can read team2 documents AND the team2 container itself (acl:accessTo) —
    // 144 docs × 6 children listed in the container graph
    let join = "SELECT ?s ?child WHERE { \
                  ?s <https://ex.dev/ns#inSubtree> <https://pod.ex/team2/> . \
                  <https://pod.ex/team2/> <http://www.w3.org/ns/ldp#contains> ?child }";
    let carol = s.query_as(&Session { agent: Some(CAROL), client: None }, Mode::Read, join).unwrap();
    assert_eq!(carol.rows.len(), 144 * 6, "join across document + container graphs");
    let bob_no_container =
        s.query_as(&Session { agent: Some(BOB), client: None }, Mode::Read, join).unwrap();
    // bob reads team2 docs too (group member) and the container via… also group
    assert_eq!(bob_no_container.rows.len(), 144 * 6);
    let alice = s.query_as(&Session { agent: Some(ALICE), client: None }, Mode::Read, join).unwrap();
    assert_eq!(alice.rows.len(), 0, "alice is shadowed out of team2 entirely");
}

#[test]
fn explicit_named_graph_query_cannot_escape() {
    let mut s = store();
    // anonymous explicitly asks for a private graph: absent from FROM NAMED ⇒ empty
    let q = "SELECT ?o WHERE { GRAPH <https://pod.ex/priv0/c4/g0/d0.ttl> { ?s ?p ?o } }";
    let anon = s.query_as(&Session::default(), Mode::Read, q).unwrap();
    assert_eq!(anon.rows.len(), 0, "unauthorized graph behaves as absent");
    let alice = s.query_as(&Session { agent: Some(ALICE), client: None }, Mode::Read, q).unwrap();
    assert!(!alice.rows.is_empty());
    // …and a pre-existing FROM NAMED is intersected, never widened
    let widened = "SELECT ?o FROM NAMED <https://pod.ex/priv0/c4/g0/d0.ttl> \
                   WHERE { GRAPH ?g { ?s ?p ?o } }";
    let anon2 = s.query_as(&Session::default(), Mode::Read, widened).unwrap();
    assert_eq!(anon2.rows.len(), 0);
}

/// The view path (default, zero-copy) and the v1 rewrite path (FROM NAMED + copy)
/// must be observationally EQUAL: for every e2e fixture session, the same query
/// returns byte-identical SPARQL-JSON through both. Queries carry a total ORDER BY
/// over the projected variables so row order is fully determined by the semantics
/// (both paths produce the same duplicate-row multiset; identical rows serialize
/// identically regardless of their relative order — a full sort makes the whole
/// serialization canonical).
#[test]
fn view_path_and_rewrite_path_return_identical_json() {
    let mut s = store();
    let queries = [
        // plain default-graph pattern (union-default emulation on both paths)
        "SELECT ?s ?title WHERE { ?s <https://ex.dev/ns#title> ?title } ORDER BY ?title ?s",
        // explicit GRAPH ?g enumeration
        "SELECT ?g ?s WHERE { GRAPH ?g { ?s <https://ex.dev/ns#title> ?title } } ORDER BY ?g ?s",
        // explicit named graph (authorized for alice, absent for everyone else)
        "SELECT ?p ?o WHERE { GRAPH <https://pod.ex/priv0/c4/g0/d0.ttl> { ?s ?p ?o } } ORDER BY ?p ?o",
        // cross-document join
        "SELECT ?s ?child WHERE { \
           ?s <https://ex.dev/ns#inSubtree> <https://pod.ex/team2/> . \
           <https://pod.ex/team2/> <http://www.w3.org/ns/ldp#contains> ?child } ORDER BY ?s ?child",
        // attacker-supplied FROM NAMED: intersected (view) / intersected (rewrite)
        "SELECT ?g ?o FROM NAMED <https://pod.ex/priv0/c4/g0/d0.ttl> \
           WHERE { GRAPH ?g { ?s ?p ?o } } ORDER BY ?g ?o",
        // aggregate over the visible set
        "SELECT (COUNT(?s) AS ?n) WHERE { ?s <https://ex.dev/ns#title> ?title }",
    ];
    let sessions = [
        ("alice", Session { agent: Some(ALICE), client: None }),
        ("carol", Session { agent: Some(CAROL), client: None }),
        ("bob+app (origin pair)", Session { agent: Some(BOB), client: Some(APP) }),
        ("anonymous", Session::default()),
    ];
    for (label, session) in sessions {
        for q in queries {
            let view_json = s.query_json_as(&session, Mode::Read, q).unwrap();
            let allowed = s.accessible(&session, Mode::Read);
            let rewritten = rewrite_for(q, &allowed).unwrap();
            let v1_json = sparq_engine::query_json(&s.graph, &rewritten).unwrap();
            assert_eq!(view_json, v1_json, "paths diverge for {label}: {q}");
        }
    }
}

#[test]
fn rewrite_shape() {
    let allowed = [oxrdf::NamedNode::new("https://pod.ex/pub1/c0/g0/d0.ttl").unwrap()];
    let out = rewrite_for(TITLES, &allowed).unwrap();
    assert!(out.contains("FROM NAMED <https://pod.ex/pub1/c0/g0/d0.ttl>"), "{out}");
    assert!(out.contains("GRAPH"), "default-graph pattern wrapped: {out}");
}
