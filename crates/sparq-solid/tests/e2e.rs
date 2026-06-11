//! End-to-end: the SAME SPARQL query, run as different sessions, returns different
//! results — enforced purely by the rewrite + dataset clause on today's engine APIs.

use sparq_core::Graph;
use sparq_solid::fixture::{wac_fixture, ALICE, BOB, CAROL};
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

#[test]
fn rewrite_shape() {
    let allowed = [oxrdf::NamedNode::new("https://pod.ex/pub1/c0/g0/d0.ttl").unwrap()];
    let out = rewrite_for(TITLES, &allowed).unwrap();
    assert!(out.contains("FROM NAMED <https://pod.ex/pub1/c0/g0/d0.ttl>"), "{out}");
    assert!(out.contains("GRAPH"), "default-graph pattern wrapped: {out}");
}
