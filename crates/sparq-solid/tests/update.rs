//! [OPUS-4.8] sq-xor3: WRITE/update-path access-control enforcement.
//!
//! Mirrors the read-path matrices (tests/wac.rs, tests/acp.rs) for the write path:
//! an actor WITH write permission succeeds and the store changes; WITHOUT, the update
//! is denied and the store is unchanged; per-graph ACLs are respected (a write to one
//! graph cannot piggy-back another); `.acl`/`.acr` writes need Control; default-graph
//! writes are denied; variable-graph / CLEAR-ALL updates fail closed; and a permitted
//! control-doc write auto-re-materializes.
//!
//! Expected outcomes are hand-computed from the fixture (crates/sparq-solid/src/fixture.rs):
//! WAC write grants — ALICE (root owner Read/Write/Control, inherited but SHADOWED out
//! of team2/), the team group BOB+CAROL (Read+Write on team2/), and Control→.acl-write
//! for ALICE; the mixed4/c1/g0/d0.ttl resource ACL grants BOB Read ONLY (no Write).

use sparq_core::Graph;
use sparq_solid::fixture::{ALICE, BOB, CAROL, DAVE};
use sparq_solid::{wac_fixture, PodStore, Session};

fn wac_store() -> PodStore {
    let g = Graph::load_dataset(&wac_fixture(), "nquads").expect("fixture loads");
    let mut s = PodStore::new(g);
    s.materialize_wac().expect("wac materializes");
    s
}

fn sess(agent: Option<&str>) -> Session<'_> {
    Session { agent, client: None }
}

/// How many triples a graph currently holds (0 if absent).
fn graph_len(store: &PodStore, name: &str) -> usize {
    let term = oxrdf::Term::NamedNode(oxrdf::NamedNode::new_unchecked(name));
    store
        .graph
        .named
        .iter()
        .find(|(n, _)| *n == term)
        .map(|(_, g)| {
            let pat: sparq_core::store::Pattern = [None, None, None];
            g.store.scan(&pat).rows.len()
        })
        .unwrap_or(0)
}

const TAG: &str = "https://ex.dev/ns#tag";
/// Insert a UNIQUE triple (so two inserts by different actors don't collide under set
/// semantics) into `graph`, tagged with `who`.
fn insert_tag(graph: &str, who: &str) -> String {
    format!("INSERT DATA {{ GRAPH <{graph}> {{ <{graph}#it> <{TAG}> \"{who}\" }} }}")
}
/// Delete the fixture title triple of `graph` (the title's literal is
/// `"doc {i}-{j}-{k}-{d}"`, derived from the document's place in the tree).
fn delete_title(graph: &str, title: &str) -> String {
    format!(
        "DELETE DATA {{ GRAPH <{graph}> {{ <{graph}#it> <https://ex.dev/ns#title> \"{title}\" }} }}"
    )
}

#[test]
fn write_with_permission_succeeds_and_mutates() {
    let mut s = wac_store();
    // ALICE owns priv0 (root owner default, inherited to depth 4) -> Write.
    let doc = "https://pod.ex/priv0/c4/g0/d0.ttl";
    let before = graph_len(&s, doc);
    s.update_as(&sess(Some(ALICE)), &insert_tag(doc, "alice")).expect("alice may write priv0");
    assert_eq!(graph_len(&s, doc), before + 1, "insert applied");
}

#[test]
fn write_without_permission_is_denied_and_store_unchanged() {
    let mut s = wac_store();
    let doc = "https://pod.ex/priv0/c4/g0/d0.ttl";
    let before = graph_len(&s, doc);
    // BOB has no grant on priv0 -> denied, nothing changes.
    let r = s.update_as(&sess(Some(BOB)), &insert_tag(doc, "bob"));
    assert!(r.is_err(), "bob denied on priv0: {r:?}");
    assert_eq!(graph_len(&s, doc), before, "store unchanged on deny");
    // anonymous likewise denied
    assert!(s.update_as(&Session::default(), &insert_tag(doc, "anon")).is_err());
    assert_eq!(graph_len(&s, doc), before);
}

#[test]
fn group_write_grant_allows_carol_and_bob_not_dave() {
    let mut s = wac_store();
    // team2 grants the vcard group (bob, carol) Read+Write; dave is not a member.
    let doc = "https://pod.ex/team2/c3/g0/d0.ttl";
    let before = graph_len(&s, doc);
    s.update_as(&sess(Some(CAROL)), &insert_tag(doc, "carol")).expect("carol writes team2");
    assert_eq!(graph_len(&s, doc), before + 1);
    s.update_as(&sess(Some(BOB)), &insert_tag(doc, "bob")).expect("bob writes team2");
    assert_eq!(graph_len(&s, doc), before + 2);
    let dave = s.update_as(&sess(Some(DAVE)), &insert_tag(doc, "dave"));
    assert!(dave.is_err(), "dave (non-member) denied: {dave:?}");
    assert_eq!(graph_len(&s, doc), before + 2, "dave's denied write did nothing");
}

#[test]
fn nearest_acl_shadowing_denies_alice_in_team2() {
    let mut s = wac_store();
    // team2's own ACL re-grants ONLY the group, shadowing the root owner — alice, who
    // can write everywhere else, CANNOT write team2 (mirrors the read-path assertion
    // wac_expected_access_matrix #5).
    let doc = "https://pod.ex/team2/c3/g0/d0.ttl";
    let before = graph_len(&s, doc);
    let r = s.update_as(&sess(Some(ALICE)), &insert_tag(doc, "alice"));
    assert!(r.is_err(), "alice shadowed out of team2 for write: {r:?}");
    assert_eq!(graph_len(&s, doc), before);
}

#[test]
fn delete_needs_write_not_just_append() {
    // BOB has Read only on the resource-specific mixed4/c1/g0/d0.ttl ACL -> no Write,
    // no Append: a DELETE must be denied. (subtree mixed4 = index 4, c1, g0, d0.)
    let mut s = wac_store();
    let doc = "https://pod.ex/mixed4/c1/g0/d0.ttl";
    let before = graph_len(&s, doc);
    let r = s.update_as(&sess(Some(BOB)), &delete_title(doc, "doc 4-1-0-0"));
    assert!(r.is_err(), "bob has Read only, delete denied: {r:?}");
    assert_eq!(graph_len(&s, doc), before);
    // ALICE has no grant on this resource (the resource ACL replaced the container
    // default with bob-only) -> also denied.
    assert!(s.update_as(&sess(Some(ALICE)), &delete_title(doc, "doc 4-1-0-0")).is_err());
    assert_eq!(graph_len(&s, doc), before);
}

#[test]
fn alice_delete_with_write_succeeds() {
    let mut s = wac_store();
    // priv0 = subtree index 0, c4, g0, d0 -> title literal "doc 0-4-0-0".
    let doc = "https://pod.ex/priv0/c4/g0/d0.ttl";
    let before = graph_len(&s, doc);
    assert!(before > 0, "fixture doc non-empty");
    s.update_as(&sess(Some(ALICE)), &delete_title(doc, "doc 0-4-0-0")).expect("alice may delete priv0");
    assert_eq!(graph_len(&s, doc), before - 1, "delete applied");
}

#[test]
fn partial_graph_write_respects_per_graph_acls() {
    // A single update touching TWO graphs is permitted only if BOTH are writable.
    // carol may write team2 but NOT priv0 -> the combined update is denied wholesale and
    // NEITHER graph changes (the check runs entirely before apply).
    let mut s = wac_store();
    let ok_graph = "https://pod.ex/team2/c3/g0/d0.ttl"; // carol: write
    let bad_graph = "https://pod.ex/priv0/c4/g0/d0.ttl"; // carol: no grant
    let before_ok = graph_len(&s, ok_graph);
    let before_bad = graph_len(&s, bad_graph);
    let upd = format!(
        "INSERT DATA {{ GRAPH <{ok_graph}> {{ <{ok_graph}#it> <{TAG}> \"a\" }} \
                        GRAPH <{bad_graph}> {{ <{bad_graph}#it> <{TAG}> \"b\" }} }}"
    );
    let r = s.update_as(&sess(Some(CAROL)), &upd);
    assert!(r.is_err(), "mixed-graph update denied (carol lacks priv0): {r:?}");
    assert_eq!(graph_len(&s, ok_graph), before_ok, "writable graph untouched on deny");
    assert_eq!(graph_len(&s, bad_graph), before_bad, "unwritable graph untouched");

    // …and when carol writes ONLY the graph she may, it succeeds.
    s.update_as(&sess(Some(CAROL)), &insert_tag(ok_graph, "carol")).expect("carol writes team2 alone");
    assert_eq!(graph_len(&s, ok_graph), before_ok + 1);
}

#[test]
fn default_graph_write_denied() {
    let mut s = wac_store();
    // Even the owner cannot write the default graph (pod data is never there).
    let r = s.update_as(&sess(Some(ALICE)), "INSERT DATA { <urn:x:s> <urn:x:p> <urn:x:o> }");
    assert!(r.is_err(), "default-graph write denied even for owner: {r:?}");
}

#[test]
fn control_gates_acl_document_writes() {
    let mut s = wac_store();
    // The rules translate alice's acl:Control on the pod root into `auth:write` on the
    // .acl graph (design doc §3.3), so writing .acl needs Write — which ONLY a
    // Control-holder has. ALICE permitted; BOB (no Control) denied — Control gating with
    // no special branch.
    let acl = "https://pod.ex/.acl";
    let upd =
        format!("INSERT DATA {{ GRAPH <{acl}> {{ <{acl}#x> <https://ex.dev/ns#note> \"n\" }} }}");
    let before = graph_len(&s, acl);
    // bob denied
    let bob = s.update_as(&sess(Some(BOB)), &upd);
    assert!(bob.is_err(), "bob lacks Control on .acl: {bob:?}");
    assert_eq!(graph_len(&s, acl), before, "bob's denied .acl write did nothing");
    // alice permitted; because it touched an .acl graph the view AUTO-re-materializes
    // afterwards (alice can still read priv0).
    s.update_as(&sess(Some(ALICE)), &upd).expect("alice has Control on .acl");
    assert_eq!(graph_len(&s, acl), before + 1, "alice's .acl write applied");
    // re-materialization kept the view intact: alice still owns priv0.
    let doc = "https://pod.ex/priv0/c4/g0/d0.ttl";
    s.update_as(&sess(Some(ALICE)), &insert_tag(doc, "after-remat")).expect("view survived remat");
}

#[test]
fn variable_graph_target_fails_closed_for_non_owner() {
    let mut s = wac_store();
    // A DELETE/INSERT with a variable GRAPH slot could touch any graph -> requires write
    // on EVERY graph. Carol cannot write priv0/etc., so this is denied even though the
    // WHERE would only match graphs she can see.
    let upd = "DELETE { GRAPH ?g { ?s ?p ?o } } WHERE { GRAPH ?g { ?s ?p ?o } }";
    let r = s.update_as(&sess(Some(CAROL)), upd);
    assert!(r.is_err(), "variable-graph DELETE denied for non-omnipotent actor: {r:?}");
}

#[test]
fn clear_all_fails_closed() {
    let mut s = wac_store();
    // CLEAR ALL touches every graph -> needs write on all of them. Nobody in the fixture
    // can write the .acl docs except via Control AND every content graph -> denied.
    let r = s.update_as(&sess(Some(ALICE)), "CLEAR ALL");
    assert!(r.is_err(), "CLEAR ALL denied (no actor can write every graph + every .acl): {r:?}");
    // and the store is intact: alice can still read+write priv0.
    let doc = "https://pod.ex/priv0/c4/g0/d0.ttl";
    assert!(graph_len(&s, doc) > 0, "CLEAR ALL did not run");
}

#[test]
fn unparseable_update_is_an_error() {
    let mut s = wac_store();
    assert!(s.update_as(&sess(Some(ALICE)), "NONSENSE not sparql").is_err());
}

// --- [OPUS-4.8] sq-biss: PRECISE per-solution variable-GRAPH write checks ------------
//
// The conservative path (variable_graph_target_fails_closed_for_non_owner / clear_all_…)
// demands write on EVERY store graph for a `GRAPH ?var` slot. These tests cover the
// precise resolution: the operation's WHERE is evaluated to find the CONCRETE graphs
// `?var` actually binds to, and write is required only on those — strictly LESS
// restrictive for the authorized case, still fail-closed for the unauthorized one.
//
// Expected outcomes are hand-computed against the WAC fixture's CAROL grant: CAROL is a
// team-group member with Read+Write on the WHOLE `team2/` subtree (187 graphs, none of
// them `.acl`) and NOTHING else — but she can READ far more (e.g. all of `mixed4/`, which
// any authenticated agent reads). The 144 `team2/…` content documents carry an
// `<ex:title>` triple; the team2 `.acl` documents do not (they hold WAC Authorizations),
// so an `<ex:title>`-only WHERE restricted to the team2 prefix binds `?g` to exactly the
// 144 content graphs, every one of which CAROL may write.

const TITLE: &str = "https://ex.dev/ns#title";
const TEAM2_DOC: &str = "https://pod.ex/team2/c3/g0/d0.ttl"; // subtree 2, c3, g0, d0
const MIXED4_DOC: &str = "https://pod.ex/mixed4/c3/g0/d0.ttl";

/// A `DELETE { GRAPH ?g { … } } WHERE { GRAPH ?g { ?s <title> ?o } FILTER prefix }` —
/// the `?g` slot binds only to graphs under `prefix` that carry a title.
fn var_delete_title_under(prefix: &str) -> String {
    format!(
        "DELETE {{ GRAPH ?g {{ ?s <{TITLE}> ?o }} }} \
         WHERE  {{ GRAPH ?g {{ ?s <{TITLE}> ?o }} FILTER(STRSTARTS(STR(?g), \"{prefix}\")) }}"
    )
}

#[test]
fn var_graph_precise_allows_authorized_subset() {
    // CAROL, variable GRAPH, WHERE bound to the team2 subtree she fully owns.
    // OLD behaviour: the same actor's UNFILTERED variable-GRAPH delete is denied
    // (variable_graph_target_fails_closed_for_non_owner), because the all-graphs check
    // demands write on graphs outside team2 she cannot write. The precise check resolves
    // `?g` to the 144 writable team2 content graphs only -> PERMITTED.
    let mut s = wac_store();
    let before = graph_len(&s, TEAM2_DOC);
    assert!(before > 0, "team2 doc has a title triple to delete");

    s.update_as(&sess(Some(CAROL)), &var_delete_title_under("https://pod.ex/team2/"))
        .expect("precise check: carol may delete titles across the team2 subtree she owns");

    // the delete actually applied (the doc lost its single title triple)
    assert_eq!(graph_len(&s, TEAM2_DOC), before - 1, "title deleted from a bound team2 graph");
}

#[test]
fn var_graph_precise_denies_readable_but_unwritable_binding() {
    // CAROL can READ all of mixed4/ (authenticated-agent read) but can WRITE none of it.
    // The precise check must NOT confuse read with write: resolving `?g` to the mixed4
    // content graphs and finding CAROL lacks Write -> DENIED, store untouched.
    let mut s = wac_store();
    let before = graph_len(&s, MIXED4_DOC);
    assert!(before > 0, "mixed4 doc non-empty");

    let r = s.update_as(&sess(Some(CAROL)), &var_delete_title_under("https://pod.ex/mixed4/"));
    assert!(r.is_err(), "carol may read but not write mixed4 -> denied: {r:?}");
    assert_eq!(graph_len(&s, MIXED4_DOC), before, "denied variable-graph delete changed nothing");
}

#[test]
fn var_graph_precise_denies_when_a_bound_graph_is_unwritable_control_doc() {
    // A WHERE matching ANY predicate under team2 also binds `?g` to the team2 `.acl`
    // graphs (they hold triples). CAROL has Read+Write on team2 content but NO Control,
    // so she lacks Write on the `.acl` graphs -> the precise check denies (an `.acl`
    // target needs the Write grant only a Control-holder has), store untouched.
    let mut s = wac_store();
    let acl = "https://pod.ex/team2/.acl";
    let before_acl = graph_len(&s, acl);
    let before_doc = graph_len(&s, TEAM2_DOC);
    let upd = "DELETE { GRAPH ?g { ?s ?p ?o } } \
               WHERE  { GRAPH ?g { ?s ?p ?o } FILTER(STRSTARTS(STR(?g), \"https://pod.ex/team2/\")) }";
    let r = s.update_as(&sess(Some(CAROL)), upd);
    assert!(r.is_err(), "binding includes team2 .acl docs carol cannot control: {r:?}");
    assert_eq!(graph_len(&s, acl), before_acl, ".acl untouched on deny");
    assert_eq!(graph_len(&s, TEAM2_DOC), before_doc, "content untouched on deny (check is pre-apply)");
}

#[test]
fn var_graph_precise_insert_authorized_subset_succeeds() {
    // INSERT through a variable GRAPH needs Write-OR-Append per bound graph. CAROL copies
    // a tag onto every team2 content graph that has a title (bound `?g` = the 144 team2
    // content graphs, all writable). Old all-graphs check: denied. Precise check: ok.
    let mut s = wac_store();
    let before = graph_len(&s, TEAM2_DOC);
    let upd = format!(
        "INSERT {{ GRAPH ?g {{ ?s <{TAG}> \"v\" }} }} \
         WHERE  {{ GRAPH ?g {{ ?s <{TITLE}> ?o }} FILTER(STRSTARTS(STR(?g), \"https://pod.ex/team2/\")) }}"
    );
    s.update_as(&sess(Some(CAROL)), &upd).expect("precise insert into team2 graphs carol owns");
    assert_eq!(graph_len(&s, TEAM2_DOC), before + 1, "one tag inserted into a bound team2 graph");
}

#[test]
fn var_graph_precise_insert_into_unwritable_binding_denied() {
    // Same INSERT shape, but `?g` resolves to mixed4 (readable, not writable) -> denied.
    let mut s = wac_store();
    let before = graph_len(&s, MIXED4_DOC);
    let upd = format!(
        "INSERT {{ GRAPH ?g {{ ?s <{TAG}> \"v\" }} }} \
         WHERE  {{ GRAPH ?g {{ ?s <{TITLE}> ?o }} FILTER(STRSTARTS(STR(?g), \"https://pod.ex/mixed4/\")) }}"
    );
    let r = s.update_as(&sess(Some(CAROL)), &upd);
    assert!(r.is_err(), "insert into unwritable mixed4 graphs denied: {r:?}");
    assert_eq!(graph_len(&s, MIXED4_DOC), before, "store unchanged on deny");
}

#[test]
fn var_graph_empty_binding_is_a_permitted_noop() {
    // A WHERE that binds `?g` to NOTHING (no graph under this absent prefix) resolves to
    // an empty target set -> the precise check has nothing to authorize and permits the
    // (no-op) update for any session, even one with no write grants. Fail-closed is about
    // not WRITING without permission; writing nothing needs nothing.
    let mut s = wac_store();
    let upd = var_delete_title_under("https://pod.ex/no-such-subtree/");
    s.update_as(&Session::default(), &upd).expect("empty-binding variable-graph update is a no-op");
}

#[test]
fn var_graph_with_using_clause_falls_back_to_conservative() {
    // A USING/WITH re-scope on a variable-GRAPH op cannot be faithfully reproduced by a
    // plain SELECT (the apply's `build_using` keeps all store named graphs for `WITH`,
    // while a query's FROM-only dataset has an EMPTY named set), so precise resolution
    // would UNDER-count the bound graphs. The check fails closed to the all-graphs
    // wildcard: CAROL — who could write this exact team2-bounded delete WITHOUT the WITH
    // clause (var_graph_precise_allows_authorized_subset) — is now denied, because she
    // cannot write every store graph.
    let mut s = wac_store();
    let before = graph_len(&s, TEAM2_DOC);
    let upd = format!(
        "WITH <{TEAM2_DOC}> \
         DELETE {{ GRAPH ?g {{ ?s <{TITLE}> ?o }} }} \
         WHERE  {{ GRAPH ?g {{ ?s <{TITLE}> ?o }} FILTER(STRSTARTS(STR(?g), \"https://pod.ex/team2/\")) }}"
    );
    let r = s.update_as(&sess(Some(CAROL)), &upd);
    assert!(r.is_err(), "USING/WITH variable-graph op falls back to conservative deny: {r:?}");
    assert_eq!(graph_len(&s, TEAM2_DOC), before, "conservative fallback applied nothing");
}
