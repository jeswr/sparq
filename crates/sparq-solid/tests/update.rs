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
    Session { agent, client: None, issuer: None, now: None }
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
fn var_graph_with_clause_resolves_precisely() {
    // [OPUS-4.8] sq-cnor: a `WITH`/`USING` re-scope on a variable-GRAPH op is now resolved
    // PRECISELY (no longer the conservative all-graphs fallback). The binding SELECT is handed
    // the same active dataset the apply's `build_using` builds — for `WITH` (which re-scopes
    // only the DEFAULT graph), `named: None` keeps all store named graphs, re-expressed as an
    // explicit `FROM NAMED` of every store named graph. Every quad here is `GRAPH ?g`-scoped,
    // so the `WITH` default graph never participates; `?g` resolves to exactly the team2
    // content graphs CAROL owns — so she is now PERMITTED, just as without the WITH clause
    // (var_graph_precise_allows_authorized_subset).
    let mut s = wac_store();
    let before = graph_len(&s, TEAM2_DOC);
    assert!(before > 0, "team2 doc has a title triple to delete");
    let upd = format!(
        "WITH <{TEAM2_DOC}> \
         DELETE {{ GRAPH ?g {{ ?s <{TITLE}> ?o }} }} \
         WHERE  {{ GRAPH ?g {{ ?s <{TITLE}> ?o }} FILTER(STRSTARTS(STR(?g), \"https://pod.ex/team2/\")) }}"
    );
    s.update_as(&sess(Some(CAROL)), &upd)
        .expect("precise WITH-clause resolution: carol may delete titles across team2 she owns");
    assert_eq!(graph_len(&s, TEAM2_DOC), before - 1, "title deleted from a bound team2 graph");
}

#[test]
fn var_graph_with_clause_precise_still_denies_unwritable_binding() {
    // The precise WITH/USING resolution must NOT confuse the re-scope with a free pass: CAROL
    // can READ mixed4 but WRITE none of it. A `WITH`-carrying variable-GRAPH delete whose `?g`
    // binds to the (readable, unwritable) mixed4 graphs is still DENIED, store untouched —
    // exactly as the no-WITH precise path denies (var_graph_precise_denies_readable_but_unwritable_binding).
    let mut s = wac_store();
    let before = graph_len(&s, MIXED4_DOC);
    assert!(before > 0, "mixed4 doc non-empty");
    let upd = format!(
        "WITH <{MIXED4_DOC}> \
         DELETE {{ GRAPH ?g {{ ?s <{TITLE}> ?o }} }} \
         WHERE  {{ GRAPH ?g {{ ?s <{TITLE}> ?o }} FILTER(STRSTARTS(STR(?g), \"https://pod.ex/mixed4/\")) }}"
    );
    let r = s.update_as(&sess(Some(CAROL)), &upd);
    assert!(r.is_err(), "WITH does not waive write-auth: carol may read but not write mixed4 -> denied: {r:?}");
    assert_eq!(graph_len(&s, MIXED4_DOC), before, "denied WITH variable-graph delete changed nothing");
}

#[test]
fn var_graph_with_clause_denies_binding_to_auth_view() {
    // [OPUS-4.8] sq-cnor — the AUTH_GRAPH under-count regression guard at the PRODUCTION
    // `check` boundary. `Dataset::build_using(named: None)` (the `WITH` re-scope) keeps EVERY
    // store named graph in the active dataset, INCLUDING the reserved `urn:sparq:auth` view.
    // So a `WITH … DELETE { GRAPH ?g { … } } WHERE { GRAPH ?g { ?s <auth#read> ?o } }` makes
    // `?g` bind to the auth view and the engine WOULD write it. The prior `rescope_dataset`
    // dropped the auth view from the materialized `FROM NAMED` set, so the precise resolver
    // MISSED that binding — the op could be (wrongly) PERMITTED and transiently mutate the
    // authorization view. With the auth view restored to the materialized set the binding is
    // resolved, and since no session is ever write-granted on the auth view the op is DENIED
    // fail-closed. The auth view must be untouched.
    let mut s = wac_store();
    let auth = "urn:sparq:auth";
    let before = graph_len(&s, auth);
    assert!(before > 0, "materialized auth view holds the WAC grant triples");
    // `auth#read` triples exist ONLY in the auth view, so `?g` binds exactly {urn:sparq:auth}.
    let upd = "WITH <https://pod.ex/team2/c3/g0/d0.ttl> \
               DELETE { GRAPH ?g { ?s ?p ?o } } \
               WHERE  { GRAPH ?g { ?s <https://sparq.dev/ns/auth#read> ?o . ?s ?p ?o } }";
    let r = s.update_as(&sess(Some(CAROL)), upd);
    assert!(
        r.is_err(),
        "a WITH var-graph op whose ?g binds to the auth view must be DENIED (no write grant on \
         the auth view); was: {r:?}"
    );
    assert_eq!(graph_len(&s, auth), before, "denied op left the auth view untouched");
}

// --- [OPUS-4.8] sq-3jtd.2: fail-closed-BEFORE-apply — a DENIED update mutates NOTHING ---
//
// The tests above assert per-graph triple COUNTS are unchanged on a deny. This block
// strengthens that to a WHOLE-STORE canonical-equality check: serialize EVERY graph
// (default + every named graph, including the team2 `.acl` docs) into a deterministic,
// sorted snapshot and assert it is identical before vs after a denied update. A count-only
// check could miss a delta that removes one triple and adds another (net-zero count); the
// canonical snapshot catches any mutation at all.
//
// The KEY case (`multi_op_one_unauthorized_*`) is a `;`-separated multi-operation body
// where ONE op is unauthorized: the invariant is that `update_inner` authorizes the WHOLE
// parsed update via `update::check` BEFORE ever calling `sparq_engine::update_in_place`, so
// the authorized op must NOT partially apply. Structurally `check` returns `Err` before any
// apply runs, so this is fail-closed-before-apply; these tests are the regression guard.

/// A single S/P/O triple as canonical strings.
type TripleStr = (String, String, String);
/// A whole-store snapshot: (graph-name -> sorted triples), default graph keyed by "".
type StoreSnapshot = Vec<(String, Vec<TripleStr>)>;

/// A deterministic snapshot of the ENTIRE store: the default graph (key "") plus every
/// named graph, each rendered as a sorted vector of canonical N-Triples-style S/P/O
/// strings. Equality of two snapshots => the store's quad set is unchanged (canonical
/// equality, independent of internal id assignment or row order).
fn store_snapshot(store: &PodStore) -> StoreSnapshot {
    fn dump_graph(g: &sparq_core::Graph) -> Vec<TripleStr> {
        let pat: sparq_core::store::Pattern = [None, None, None];
        let scan = g.store.scan(&pat);
        let mut v: Vec<TripleStr> = scan
            .rows
            .iter()
            .map(|r| {
                let spo = scan.to_spo(r);
                (
                    g.dict.term(spo[0]).to_string(),
                    g.dict.term(spo[1]).to_string(),
                    g.dict.term(spo[2]).to_string(),
                )
            })
            .collect();
        // Equality comparison only needs a deterministic order, not a stable one;
        // sort_unstable is faster for large snapshots. [OPUS-4.8]
        v.sort_unstable();
        v
    }
    let mut out: StoreSnapshot = vec![(String::new(), dump_graph(&store.graph))];
    for (name, sub) in &store.graph.named {
        out.push((name.to_string(), dump_graph(sub)));
    }
    // Snapshot is only compared for equality; an unstable sort is faster. [OPUS-4.8]
    out.sort_unstable();
    out
}

#[test]
fn denied_single_op_leaves_store_canonically_identical() {
    // BOB has no grant on priv0 -> a single-op INSERT is denied. The WHOLE store (every
    // graph) must be canonically identical before vs after.
    let mut s = wac_store();
    let doc = "https://pod.ex/priv0/c4/g0/d0.ttl";
    let before = store_snapshot(&s);
    let r = s.update_as(&sess(Some(BOB)), &insert_tag(doc, "bob"));
    assert!(r.is_err(), "bob denied on priv0: {r:?}");
    assert_eq!(
        store_snapshot(&s),
        before,
        "denied update mutated the store"
    );
}

/// A `;`-separated TWO-operation update body: an INSERT into `auth_graph` (which the actor
/// MAY write) followed by an INSERT into `denied_graph` (which the actor may NOT write).
fn multi_op_body(auth_graph: &str, denied_graph: &str) -> String {
    format!(
        "INSERT DATA {{ GRAPH <{auth_graph}> {{ <{auth_graph}#it> <{TAG}> \"authorized-op\" }} }} ; \
         INSERT DATA {{ GRAPH <{denied_graph}> {{ <{denied_graph}#it> <{TAG}> \"unauthorized-op\" }} }}"
    )
}

#[test]
fn multi_op_one_unauthorized_denies_whole_body_no_partial_apply() {
    // THE key fail-closed-before-apply case. CAROL may write team2 but NOT priv0. A two-op
    // `;`-separated body inserts into team2 (authorized) AND priv0 (unauthorized). The
    // WHOLE update must be refused and the AUTHORIZED op must NOT partially apply — the
    // store stays canonically identical.
    let mut s = wac_store();
    let auth_graph = "https://pod.ex/team2/c3/g0/d0.ttl"; // carol: write
    let denied_graph = "https://pod.ex/priv0/c4/g0/d0.ttl"; // carol: no grant
    let before = store_snapshot(&s);
    let body = multi_op_body(auth_graph, denied_graph);

    let r = s.update_as(&sess(Some(CAROL)), &body);
    assert!(
        r.is_err(),
        "multi-op body with one unauthorized op must be denied wholesale: {r:?}"
    );
    // Critically: the authorized first op did NOT slip through before the denial.
    assert_eq!(
        store_snapshot(&s),
        before,
        "fail-closed-before-apply violated: an authorized op in a denied multi-op body \
         partially applied — this would be a REAL BUG (sq-3jtd.2)"
    );
}

#[test]
fn multi_op_denied_when_second_op_is_a_delete_too() {
    // Mix operation kinds: an authorized INSERT into team2 followed by an unauthorized
    // DELETE DATA against priv0. Still one unauthorized target -> whole body refused,
    // nothing applied.
    let mut s = wac_store();
    let auth_graph = "https://pod.ex/team2/c3/g0/d0.ttl";
    let denied_graph = "https://pod.ex/priv0/c4/g0/d0.ttl";
    let before = store_snapshot(&s);
    let body = format!(
        "INSERT DATA {{ GRAPH <{auth_graph}> {{ <{auth_graph}#it> <{TAG}> \"x\" }} }} ; \
         DELETE DATA {{ GRAPH <{denied_graph}> {{ <{denied_graph}#it> <{TITLE}> \"doc 0-4-0-0\" }} }}"
    );
    let r = s.update_as(&sess(Some(CAROL)), &body);
    assert!(
        r.is_err(),
        "authorized INSERT + unauthorized DELETE denied wholesale: {r:?}"
    );
    assert_eq!(
        store_snapshot(&s),
        before,
        "no partial apply of the authorized INSERT"
    );
}

#[test]
fn positive_control_fully_authorized_multi_op_body_applies() {
    // Positive control: the SAME two-op `;`-separated shape, but BOTH targets are graphs
    // CAROL may write (two distinct team2 docs). This DOES apply — proving the denied
    // tests above exercise a real allow/deny boundary, not a path that no-ops regardless.
    let mut s = wac_store();
    let g1 = "https://pod.ex/team2/c3/g0/d0.ttl";
    let g2 = "https://pod.ex/team2/c3/g0/d1.ttl";
    let before1 = graph_len(&s, g1);
    let before2 = graph_len(&s, g2);
    let body = multi_op_body(g1, g2);

    s.update_as(&sess(Some(CAROL)), &body)
        .expect("fully-authorized multi-op body applies");
    assert_eq!(graph_len(&s, g1), before1 + 1, "first op applied");
    assert_eq!(graph_len(&s, g2), before2 + 1, "second op applied");
}

// ───────────────── budgeted write path (sq-yhlf0) ─────────────────
//
// [SONNET-4.6] sq-yhlf0 — `update_as_with_budget` bounds BOTH evaluations an update
// performs over the whole store: the authorization check's `GRAPH ?var` binding SELECT
// and the engine's template instantiation. A host that must bound every evaluation it
// issues (the sparq-mcp pod server, mcp-solid draft §9.4) depends on both being covered.

/// A budget with an already-elapsed deadline: any evaluation under it trips at its first
/// cooperative check site, so these tests carry no wall-clock race.
///
/// NATIVE ONLY, with the two tests that use it: `QueryBudget::deadline` is itself
/// `#[cfg(not(target_arch = "wasm32"))]` (`std::time::Instant` panics on
/// `wasm32-unknown-unknown`), so naming the field would not compile under the wasm lane's
/// `cargo clippy -p sparq-solid --target wasm32-unknown-unknown --all-targets`. This
/// mirrors how `sparq-engine` gates its own deadline tests. The budgeted write path keeps
/// full wasm compile coverage through the portable `max_rows` tests below.
#[cfg(not(target_arch = "wasm32"))]
fn expired_budget() -> sparq_engine::QueryBudget {
    let mut b = sparq_engine::QueryBudget::unlimited();
    b.deadline = Some(std::time::Instant::now() - std::time::Duration::from_secs(1));
    b
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn budgeted_update_bounds_the_engine_apply() {
    // STATIC (authorized) template target ⇒ the check resolves without evaluating
    // anything, so the only evaluation is the engine's — and the budget must reach it.
    let mut s = wac_store();
    let g = "https://pod.ex/team2/c3/g0/d0.ttl";
    let before = store_snapshot(&s);
    let body = format!(
        "INSERT {{ GRAPH <{g}> {{ <{g}#it> <{TAG}> ?o1 }} }} WHERE \
         {{ GRAPH ?g1 {{ ?s1 ?p1 ?o1 }} GRAPH ?g2 {{ ?s2 ?p2 ?o2 }} }}"
    );
    let err = s
        .update_as_with_budget(&sess(Some(CAROL)), &body, &expired_budget())
        .expect_err("an exhausted budget must refuse the apply");
    assert!(err.contains("query budget exceeded"), "{err}");
    assert_eq!(store_snapshot(&s), before, "nothing applied");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn budgeted_update_bounds_the_authorization_check() {
    // A `GRAPH ?var` template slot makes the CHECK evaluate the WHERE (the binding
    // SELECT). The budget must bound that too, and the trip must surface as the budget
    // error — NOT be swallowed into the conservative all-graphs wildcard, which would
    // report a misleading "update denied" for a resource failure.
    let mut s = wac_store();
    let before = store_snapshot(&s);
    let body = format!(
        "INSERT {{ GRAPH ?g1 {{ <urn:x> <{TAG}> \"z\" }} }} WHERE {{ GRAPH ?g1 {{ ?s1 ?p1 ?o1 }} }}"
    );
    let err = s
        .update_as_with_budget(&sess(Some(CAROL)), &body, &expired_budget())
        .expect_err("an exhausted budget must refuse the check");
    assert!(
        err.contains("query budget exceeded"),
        "a budget trip must not be reported as a deny: {err}"
    );
    assert_eq!(store_snapshot(&s), before, "nothing applied");
}

#[test]
fn an_unlimited_budget_is_plain_update_as() {
    // The equivalence the unbudgeted entry points rely on: same allow, same deny, same
    // resulting store. (Positive AND negative — a path that denied everything would pass
    // a positive-only check.)
    let g = "https://pod.ex/team2/c3/g0/d0.ttl";
    let denied = "https://pod.ex/priv0/c4/g0/d0.ttl";
    let unlimited = sparq_engine::QueryBudget::unlimited();

    for body in [insert_tag(g, "carol"), insert_tag(denied, "carol")] {
        let (mut plain, mut budgeted) = (wac_store(), wac_store());
        let a = plain.update_as(&sess(Some(CAROL)), &body);
        let b = budgeted.update_as_with_budget(&sess(Some(CAROL)), &body, &unlimited);
        assert_eq!(a.is_ok(), b.is_ok(), "verdicts diverge for `{body}`: {a:?} vs {b:?}");
        assert_eq!(store_snapshot(&plain), store_snapshot(&budgeted), "stores diverge");
    }
}

#[test]
fn partial_apply_that_retains_an_acl_change_still_refreshes_the_auth_view() {
    // The multi-operation security invariant behind the partial-apply contract: the
    // engine apply is NON-atomic, so an op that trips the budget keeps the ops BEFORE
    // it. If one of those retained ops rewrote an `.acl` graph, the materialized auth
    // view is now stale w.r.t. the store — and the next authorization decision would be
    // made against access-control rules that no longer exist. `update_inner` must
    // therefore re-materialize on the ERROR path too, not only after a clean apply.
    let mut s = wac_store();
    // priv0/c0/ has its OWN acl granting ALICE Read+Write+Control (it shadows the root),
    // so removing its `acl:mode acl:Write` triple revokes alice's write on that subtree.
    // She keeps Control, which is what lets her write the `.acl` graph in the first place.
    let acl = "https://pod.ex/priv0/c0/.acl";
    let revoked = "https://pod.ex/priv0/c0/g0/d0.ttl";
    // A graph alice can still write, so the whole request passes `check` before applying.
    let other = "https://pod.ex/priv0/c4/g0/d0.ttl";
    assert!(
        s.update_as(&sess(Some(ALICE)), &insert_tag(revoked, "before")).is_ok(),
        "precondition: alice may write priv0/c0 before the revocation"
    );
    let acl_before = graph_len(&s, acl);
    let other_before = graph_len(&s, other);
    let revoked_before = graph_len(&s, revoked);

    // op1 revokes alice's Write (an `.acl` write ⇒ `permit.rematerialize`); op2's WHERE
    // is a whole-store scan that blows the row budget. `INSERT DATA`/`DELETE DATA` do not
    // consult the budget, so op1 applies and op2 is the one that trips.
    let body = format!(
        "DELETE DATA {{ GRAPH <{acl}> {{ <{acl}#owner> \
         <http://www.w3.org/ns/auth/acl#mode> <http://www.w3.org/ns/auth/acl#Write> }} }} ; \
         INSERT {{ GRAPH <{other}> {{ <{other}#it> <{TAG}> ?o }} }} \
         WHERE {{ GRAPH ?g {{ ?s ?p ?o }} }}"
    );
    let mut budget = sparq_engine::QueryBudget::unlimited();
    budget.max_rows = Some(1);
    let err = s
        .update_as_with_budget(&sess(Some(ALICE)), &body, &budget)
        .expect_err("op2 must trip the row budget");
    assert!(err.contains("query budget exceeded"), "{err}");

    // The documented partial-apply contract: op1 (before the failing op) is RETAINED,
    // op2 is not applied.
    assert_eq!(graph_len(&s, acl), acl_before - 1, "op1 retained (acl:Write revoked)");
    assert_eq!(graph_len(&s, other), other_before, "op2 not applied");

    // THE INVARIANT: authorization now reflects the retained access-control graph. With
    // a stale view alice would still be allowed here, because the view still held the
    // deleted `acl:mode acl:Write`.
    let after = s.update_as(&sess(Some(ALICE)), &insert_tag(revoked, "after"));
    assert!(
        after.is_err(),
        "auth view is stale: the retained .acl revocation was not re-materialized: {after:?}"
    );
    assert_eq!(graph_len(&s, revoked), revoked_before, "denied write changed nothing");
}
