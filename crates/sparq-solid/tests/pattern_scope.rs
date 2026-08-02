//! [FABLE-5] sq-lrtc3.3 — the pattern-scope differential acceptance test.
//!
//! THE soundness property of masked-subgraph materialization (design record
//! `research/odrl-pattern-scoped-targets-2026-07.md` §0/§3): for every query in a
//! battery covering the known leakage channels — SELECT rows, OPTIONAL, EXISTS,
//! NOT EXISTS, MINUS, COUNT/GROUP BY aggregates, ASK, `GRAPH ?g` enumeration, and a
//! property path — the scoped dataset answers **identically** to an ORACLE store
//! loaded from the source dataset with the masked lines physically deleted, and
//! **differently** from the unmasked store (non-vacuity: a no-op mask flips the
//! differ-assertions red). Plus the fail-closed properties: empty-allow ⇒ graph
//! indistinguishable from absent; a scope on a non-accessible graph never widens.

#![cfg(feature = "pattern-scope")]

use oxrdf::{NamedNode, Term};
use rustc_hash::FxHashMap;
use sparq_solid::{GraphScope, Mode, PodStore, ScopePattern, Session};

const NS: &str = "https://ex.dev/ns#";
const CONTACTS: &str = "https://pod.ex/contacts";
const NOTES: &str = "https://pod.ex/notes";

/// The pod's .acl: alice may Read everything under the pod root (subtree default).
fn acl_quads() -> String {
    let acl = "https://pod.ex/.acl";
    [
        "type <http://www.w3.org/ns/auth/acl#Authorization>",
        "default <https://pod.ex/>",
        "agent <https://alice.ex/card#me>",
        "mode <http://www.w3.org/ns/auth/acl#Read>",
    ]
    .iter()
    .map(|po| {
        let (p, o) = po.split_once(' ').unwrap();
        let p = if p == "type" {
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_owned()
        } else {
            format!("http://www.w3.org/ns/auth/acl#{p}")
        };
        format!("<{acl}#r> <{p}> {o} <{acl}> .\n")
    })
    .collect()
}

/// Data quads: contacts (names + phones + a knows-chain) and an unscoped notes graph.
/// `with_phones=false` is the ORACLE variant — the masked triples physically absent.
fn data_quads(with_phones: bool) -> String {
    let mut q = String::new();
    for (person, name, phone) in
        [("c1", "Ada", "555-0100"), ("c2", "Bo", "555-0101"), ("c3", "Cy", "555-0102")]
    {
        q += &format!("<{CONTACTS}#{person}> <{NS}name> \"{name}\" <{CONTACTS}> .\n");
        if with_phones {
            q += &format!("<{CONTACTS}#{person}> <{NS}phone> \"{phone}\" <{CONTACTS}> .\n");
        }
    }
    // knows-chain c1 -> c2 -> c3; the c2 -> c3 edge is ALSO phone-independent, while the
    // c1 -> c2 edge is masked in the masked variant (property-path leak probe).
    if with_phones {
        q += &format!("<{CONTACTS}#c1> <{NS}knows> <{CONTACTS}#c2> <{CONTACTS}> .\n");
    }
    q += &format!("<{CONTACTS}#c2> <{NS}knows> <{CONTACTS}#c3> <{CONTACTS}> .\n");
    q += &format!("<{NOTES}#n1> <{NS}title> \"hello\" <{NOTES}> .\n");
    q
}

fn store(with_masked_triples: bool) -> PodStore {
    let nq = data_quads(with_masked_triples) + &acl_quads();
    let mut s = PodStore::new(sparq_core::Graph::load_dataset(&nq, "nquads").unwrap());
    s.materialize_wac().unwrap();
    s
}

fn alice() -> Session<'static> {
    Session { agent: Some("https://alice.ex/card#me"), client: None, issuer: None, now: None }
}

fn iri(s: &str) -> Term {
    Term::NamedNode(NamedNode::new(s).unwrap())
}

/// The scope under test: alice sees the contacts graph EXCEPT phone triples and
/// EXCEPT the c1->knows->c2 edge (an ODRL prohibition carving holes out of a grant).
fn masking_scopes() -> FxHashMap<Term, GraphScope> {
    let mut scopes = FxHashMap::default();
    scopes.insert(
        iri(CONTACTS),
        GraphScope::deny_within(vec![
            ScopePattern::new(None, Some(iri(&format!("{NS}phone"))), None),
            ScopePattern::new(
                Some(iri(&format!("{CONTACTS}#c1"))),
                Some(iri(&format!("{NS}knows"))),
                None,
            ),
        ]),
    );
    scopes
}

/// The leakage-channel battery (all deterministic: ORDER BY on every SELECT).
const BATTERY: &[(&str, &str)] = &[
    ("select-all", "SELECT ?s ?p ?o WHERE { GRAPH ?g { ?s ?p ?o } } ORDER BY ?s ?p ?o"),
    (
        "optional",
        "SELECT ?s ?ph WHERE { GRAPH ?g { ?s <https://ex.dev/ns#name> ?n \
         OPTIONAL { ?s <https://ex.dev/ns#phone> ?ph } } } ORDER BY ?s ?ph",
    ),
    (
        "exists",
        "SELECT ?s WHERE { GRAPH ?g { ?s <https://ex.dev/ns#name> ?n \
         FILTER EXISTS { ?s <https://ex.dev/ns#phone> ?ph } } } ORDER BY ?s",
    ),
    (
        "not-exists",
        "SELECT ?s WHERE { GRAPH ?g { ?s <https://ex.dev/ns#name> ?n \
         FILTER NOT EXISTS { ?s <https://ex.dev/ns#phone> ?ph } } } ORDER BY ?s",
    ),
    (
        "minus",
        "SELECT ?s WHERE { GRAPH ?g { { ?s <https://ex.dev/ns#name> ?n } \
         MINUS { ?s <https://ex.dev/ns#phone> ?ph } } } ORDER BY ?s",
    ),
    ("count", "SELECT (COUNT(*) AS ?c) WHERE { GRAPH ?g { ?s ?p ?o } }"),
    (
        "group-count",
        "SELECT ?s (COUNT(?p) AS ?c) WHERE { GRAPH ?g { ?s ?p ?o } } GROUP BY ?s ORDER BY ?s",
    ),
    ("graphs", "SELECT DISTINCT ?g WHERE { GRAPH ?g { ?s ?p ?o } } ORDER BY ?g"),
    (
        "path",
        "SELECT ?x ?y WHERE { GRAPH ?g { ?x <https://ex.dev/ns#knows>+ ?y } } ORDER BY ?x ?y",
    ),
];

const ASK_PHONE: &str = "ASK { GRAPH ?g { ?s <https://ex.dev/ns#phone> ?o } }";
const ASK_PATH: &str =
    "ASK { GRAPH ?g { <https://pod.ex/contacts#c1> <https://ex.dev/ns#knows>+ <https://pod.ex/contacts#c3> } }";

/// Masked triples are unobservable through EVERY battery channel: the scoped dataset
/// answers exactly as the oracle (triples physically absent) — and NOT as the
/// unmasked store (non-vacuity).
#[test]
fn differential_battery_masked_equals_oracle_and_differs_from_unmasked() {
    let full = store(true);
    let oracle = store(false);
    let scoped = full.scoped_dataset(&alice(), Mode::Read, &masking_scopes());

    for (label, q) in BATTERY {
        let got = scoped.query_json(q).unwrap();
        let want = oracle.query_json_as(&alice(), Mode::Read, q).unwrap();
        assert_eq!(got, want, "leak via {label}: scoped != oracle");
    }
    // Non-vacuity: a no-op mask makes these agree and flips the test red.
    for label in ["select-all", "count", "path"] {
        let q = BATTERY.iter().find(|(l, _)| l == &label).unwrap().1;
        let scoped_json = scoped.query_json(q).unwrap();
        let unmasked = full.query_json_as(&alice(), Mode::Read, q).unwrap();
        assert_ne!(scoped_json, unmasked, "vacuous mask: {label} unchanged by masking");
    }
    // ASK: satisfiability matches the oracle, differs from the unmasked store.
    assert!(!scoped.ask(ASK_PHONE).unwrap());
    assert!(!oracle.ask_as(&alice(), Mode::Read, ASK_PHONE).unwrap());
    assert!(full.ask_as(&alice(), Mode::Read, ASK_PHONE).unwrap());
    // Property-path transitivity through a masked edge: c1 ->+ c3 must be false
    // (the c1->c2 hop is masked) even though c2->c3 stays visible.
    assert!(!scoped.ask(ASK_PATH).unwrap());
    assert!(!oracle.ask_as(&alice(), Mode::Read, ASK_PATH).unwrap());
    assert!(full.ask_as(&alice(), Mode::Read, ASK_PATH).unwrap());
}

/// Fail-closed: an empty-allow scope masks the WHOLE graph, and the fully-masked
/// graph is indistinguishable from an absent one (name enumeration + direct ASK).
#[test]
fn fully_masked_graph_is_indistinguishable_from_absent() {
    let full = store(true);
    let mut scopes = FxHashMap::default();
    scopes.insert(iri(CONTACTS), GraphScope::allow_only(Vec::new()));
    let scoped = full.scoped_dataset(&alice(), Mode::Read, &scopes);

    let graphs = scoped
        .query("SELECT DISTINCT ?g WHERE { GRAPH ?g { ?s ?p ?o } } ORDER BY ?g")
        .unwrap();
    assert_eq!(graphs.rows.len(), 1, "only the notes graph remains enumerable");
    assert!(!scoped
        .ask("ASK { GRAPH <https://pod.ex/contacts> { ?s ?p ?o } }")
        .unwrap());
    // The unscoped notes graph is untouched.
    assert!(scoped.ask("ASK { GRAPH <https://pod.ex/notes> { ?s ?p ?o } }").unwrap());
}

/// Never widens: a pattern scope for a graph the session has NO graph-level grant on
/// contributes nothing (a grant-less session stays at zero rows).
#[test]
fn scope_on_non_accessible_graph_does_not_widen() {
    let full = store(true);
    let nobody = Session::default();
    let mut scopes = FxHashMap::default();
    scopes.insert(iri(CONTACTS), GraphScope::deny_within(Vec::new())); // "allow all"
    let scoped = full.scoped_dataset(&nobody, Mode::Read, &scopes);
    let rows = scoped.query("SELECT ?s WHERE { GRAPH ?g { ?s ?p ?o } }").unwrap();
    assert_eq!(rows.rows.len(), 0, "pattern scope widened a grant-less session");
    assert!(!scoped.ask("ASK { GRAPH ?g { ?s ?p ?o } }").unwrap());
}

/// The scoped replica honours the same read-path rewrite as `query_as`: a bare
/// default-graph pattern matches nothing without the union-default opt-in.
#[cfg(not(feature = "legacy-union-default-graph"))]
#[test]
fn scoped_dataset_keeps_empty_default_graph_semantics() {
    let full = store(true);
    let scoped = full.scoped_dataset(&alice(), Mode::Read, &masking_scopes());
    let bare = scoped.query("SELECT ?s WHERE { ?s ?p ?o }").unwrap();
    assert_eq!(bare.rows.len(), 0, "bare pattern must see the empty default graph");
    let opt_in = scoped
        .query(
            "SELECT ?s FROM <http://www.w3.org/ns/solid/sparql#union-default-graph> \
             WHERE { ?s ?p ?o }",
        )
        .unwrap();
    assert!(!opt_in.rows.is_empty(), "union-default opt-in must range over the replica");
}

// --- [SONNET-4.6] sq-nc3c6: replica cache + write-path invalidation (design record §6) ---

/// The pod's .acl granting alice Read AND Write over the subtree, so the write path can
/// be driven through the real `update_as` enforcement (a Read-only session cannot write
/// at all, which would make the invalidation test vacuous).
fn rw_acl_quads() -> String {
    let acl = "https://pod.ex/.acl";
    let rdf_type = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
    let acl_ns = "http://www.w3.org/ns/auth/acl#";
    format!(
        "<{acl}#r> <{rdf_type}> <{acl_ns}Authorization> <{acl}> .\n\
         <{acl}#r> <{acl_ns}default> <https://pod.ex/> <{acl}> .\n\
         <{acl}#r> <{acl_ns}agent> <https://alice.ex/card#me> <{acl}> .\n\
         <{acl}#r> <{acl_ns}mode> <{acl_ns}Read> <{acl}> .\n\
         <{acl}#r> <{acl_ns}mode> <{acl_ns}Write> <{acl}> .\n"
    )
}

fn rw_store() -> PodStore {
    let nq = data_quads(true) + &rw_acl_quads();
    let mut s = PodStore::new(sparq_core::Graph::load_dataset(&nq, "nquads").unwrap());
    s.materialize_wac().unwrap();
    s
}

const NAMES: &str =
    "SELECT ?n WHERE { GRAPH ?g { ?s <https://ex.dev/ns#name> ?n } } ORDER BY ?n";

/// A DATA write invalidates the masked replicas: the scoped dataset built AFTER the
/// write reflects it, both for an inserted triple and for a deleted one. This is the
/// stale-read differential — without the invalidation seam in `update_inner` the
/// post-write `scoped_dataset` answers from the pre-write replica.
///
/// The writes deliberately target a pod DATA document, not a `.acl`, so
/// `Permit::rematerialize` is false and no re-materialization fires: this exercises the
/// write-path seam specifically, not `reindex_with`'s.
#[test]
fn data_write_invalidates_the_scoped_replica() {
    let mut store = rw_store();
    let before = store.scoped_dataset(&alice(), Mode::Read, &masking_scopes());
    let before_names = before.query_json(NAMES).unwrap();
    assert!(before_names.contains("Ada") && !before_names.contains("Di"));

    store
        .update_as(
            &alice(),
            &format!(
                "INSERT DATA {{ GRAPH <{CONTACTS}> {{ <{CONTACTS}#c4> <{NS}name> \"Di\" }} }}"
            ),
        )
        .unwrap();
    let after_insert = store.scoped_dataset(&alice(), Mode::Read, &masking_scopes());
    assert!(
        after_insert.query_json(NAMES).unwrap().contains("Di"),
        "stale replica: the scoped dataset did not see the inserted triple"
    );

    // Redaction-by-DELETE is the case staleness would actually defeat.
    store
        .update_as(
            &alice(),
            &format!("DELETE DATA {{ GRAPH <{CONTACTS}> {{ <{CONTACTS}#c1> <{NS}name> \"Ada\" }} }}"),
        )
        .unwrap();
    let after_delete = store.scoped_dataset(&alice(), Mode::Read, &masking_scopes());
    let names = after_delete.query_json(NAMES).unwrap();
    assert!(!names.contains("Ada"), "stale replica: a deleted triple stayed visible");
    assert!(names.contains("Di") && names.contains("Bo"), "the rest of the graph survives");
}

/// Invalidation must not weaken the mask: a phone triple written AFTER the replica was
/// cached is masked by the same scope, exactly as the pre-existing ones are.
#[test]
fn a_replica_rebuilt_after_a_write_still_masks() {
    let mut store = rw_store();
    let scoped = store.scoped_dataset(&alice(), Mode::Read, &masking_scopes());
    assert!(!scoped.ask(ASK_PHONE).unwrap());

    store
        .update_as(
            &alice(),
            &format!(
                "INSERT DATA {{ GRAPH <{CONTACTS}> {{ <{CONTACTS}#c4> <{NS}phone> \"555-0103\" ; \
                 <{NS}name> \"Di\" }} }}"
            ),
        )
        .unwrap();
    let rebuilt = store.scoped_dataset(&alice(), Mode::Read, &masking_scopes());
    assert!(rebuilt.query_json(NAMES).unwrap().contains("Di"), "the write landed");
    assert!(!rebuilt.ask(ASK_PHONE).unwrap(), "the newly written phone triple leaked");
    assert!(
        !rebuilt.query_json("SELECT ?o WHERE { GRAPH ?g { ?s ?p ?o } }").unwrap().contains("555-0103"),
        "the newly written phone value leaked"
    );
}

/// Reusing a cached replica changes NOTHING observable: a scoped dataset built from a
/// warm cache answers the whole leakage battery exactly as the first (cold) one, and as
/// the oracle store with the masked triples physically deleted.
#[test]
fn a_warm_replica_answers_identically_to_a_cold_one() {
    let full = store(true);
    let oracle = store(false);
    let cold = full.scoped_dataset(&alice(), Mode::Read, &masking_scopes());
    let warm = full.scoped_dataset(&alice(), Mode::Read, &masking_scopes());
    for (label, q) in BATTERY {
        let cold_json = cold.query_json(q).unwrap();
        assert_eq!(cold_json, warm.query_json(q).unwrap(), "warm != cold via {label}");
        assert_eq!(
            cold_json,
            oracle.query_json_as(&alice(), Mode::Read, q).unwrap(),
            "warm/cold != oracle via {label}"
        );
    }
}

/// Two DIFFERENT scopes over the SAME graph, interleaved so both are cached at once,
/// must each get their own mask. This is the fail-open shape the replica cache could
/// introduce: keying on the graph name alone (or on a colliding fingerprint) would serve
/// one requester's replica to the other and over-disclose.
#[test]
fn two_scopes_on_one_graph_never_cross_serve() {
    let full = store(true);
    // Scope A hides phones (the shared `masking_scopes` mask); scope B hides names.
    let mut b = FxHashMap::default();
    b.insert(
        iri(CONTACTS),
        GraphScope::deny_within(vec![ScopePattern::new(None, Some(iri(&format!("{NS}name"))), None)]),
    );

    let a_first = full.scoped_dataset(&alice(), Mode::Read, &masking_scopes());
    let b_first = full.scoped_dataset(&alice(), Mode::Read, &b);
    // …and again, now that BOTH replicas are live in the cache.
    let a_warm = full.scoped_dataset(&alice(), Mode::Read, &masking_scopes());
    let b_warm = full.scoped_dataset(&alice(), Mode::Read, &b);

    for a in [&a_first, &a_warm] {
        assert!(!a.ask(ASK_PHONE).unwrap(), "scope A leaked a phone triple");
        assert!(a.query_json(NAMES).unwrap().contains("Ada"), "scope A lost a name triple");
    }
    for bb in [&b_first, &b_warm] {
        assert!(bb.ask(ASK_PHONE).unwrap(), "scope B lost its phone triples");
        assert_eq!(bb.query(NAMES).unwrap().rows.len(), 0, "scope B leaked a name triple");
    }
}
