//! [SONNET-4.6] sq-neovc (issue #1248 item 3 / #992 FR-4) — the **named-graph-per-document
//! loader contract** that the in-process embedding surface ([`PodStore::query_as`] /
//! [`PodStore::accessible`]) is stabilised against.
//!
//! The API-stability marker on those methods is only worth something if the *dataset shape*
//! they are defined over is pinned too: a host (an LDP resource server embedding this crate)
//! must know exactly how to lay its pod out before "which graphs may this session read?" has
//! a stable answer. Clauses 1–4 of the contract documented on `PodStore` are pinned here,
//! each exercising the REAL `PodStore::new` → `materialize_wac` (and, for clause 3's `.acr`
//! half, `materialize_acp`) → `accessible`/`query_as` path — no mock stands in for the
//! verdict.
//!
//! The last two clauses were already pinned and are not duplicated: clause 5 (the reserved
//! `urn:sparq:` space) by `hardening.rs` (`sentinel_graph_cannot_be_smuggled` /
//! `reserved_session_values_fail_closed`) plus the `PodStore::new` doctest, and clause 6
//! (trusted facts only via the typed channels) by `acp.rs`
//! (`acp_forged_*_in_acr_document_does_not_grant`).

use oxrdf::Term;
use sparq_core::Graph;
use sparq_solid::conformance::AcrBuilder;
use sparq_solid::wac_conformance::AclBuilder;
use sparq_solid::{Mode, PodStore, Session};
// Only the spec-conformant (default) read path exercises the union-default opt-in; the
// `legacy-union-default-graph` escape hatch predates the spec-minted IRI.
#[cfg(not(feature = "legacy-union-default-graph"))]
use sparq_solid::UNION_DEFAULT_GRAPH_IRI;
use std::collections::BTreeSet;

const ALICE: &str = "https://alice.ex/card#me";
const N1: &str = "https://pod.ex/notes/n1";
const N2: &str = "https://pod.ex/notes/n2";
const EX_PROP: &str = "https://ex.dev/ns#prop";

fn alice() -> Session<'static> {
    Session { agent: Some(ALICE), client: None, issuer: None, now: None }
}

fn materialized(nquads: &str) -> PodStore {
    let g = Graph::load_dataset(nquads, "nquads").expect("fixture loads");
    let mut s = PodStore::new(g);
    s.materialize_wac().expect("materializes");
    s
}

/// As [`materialized`], but through the ACP materializer (clause 3's `.acr` half).
fn materialized_acp(nquads: &str) -> PodStore {
    let g = Graph::load_dataset(nquads, "nquads").expect("fixture loads");
    let mut s = PodStore::new(g);
    s.materialize_acp().expect("materializes");
    s
}

/// [`readable`] over an ACP corpus.
fn readable_acp(nquads: &str) -> BTreeSet<String> {
    readable(&materialized_acp(nquads))
}

/// The session's readable graph names, as plain IRI strings.
fn readable(store: &PodStore) -> BTreeSet<String> {
    store
        .accessible(&alice(), Mode::Read)
        .iter()
        .map(|n| n.as_str().to_owned())
        .collect()
}

/// The lexical values bound to the single projected variable of `q`, run as alice.
fn objects(store: &PodStore, q: &str) -> BTreeSet<String> {
    store
        .query_as(&alice(), Mode::Read, q)
        .expect("query ok")
        .rows
        .iter()
        .filter_map(|row| row.first().and_then(|t| t.as_ref()))
        .map(|t| match t {
            Term::Literal(l) => l.value().to_owned(),
            other => other.to_string(),
        })
        .collect()
}

/// The IRIs bound to `?g` by a `GRAPH ?g { ?s ?p ?o }` scan run as alice — i.e. the graphs
/// `query_as` actually evaluates over, as opposed to the ones `accessible` advertises.
fn scanned(store: &PodStore) -> BTreeSet<String> {
    let r = store
        .query_as(&alice(), Mode::Read, "SELECT ?g WHERE { GRAPH ?g { ?s ?p ?o } }")
        .expect("query ok");
    let col = r.vars.iter().position(|v| v.as_str() == "g").expect("projects ?g");
    r.rows
        .iter()
        .filter_map(|row| row.get(col).and_then(|t| t.as_ref()))
        .filter_map(|t| match t {
            Term::NamedNode(n) => Some(n.as_str().to_owned()),
            _ => None,
        })
        .collect()
}

// ─── Clause 1: one document = one named graph, named by the document IRI ──────────────

/// A grant names a DOCUMENT, and the document IRI is verbatim the graph name — so a granted
/// document appears in the authorized set under its own IRI, and an ungranted sibling
/// document in the same container is absent from both `accessible` and `query_as`.
#[test]
fn graph_name_is_the_document_iri_and_the_unit_of_authorization() {
    let mut b = AclBuilder::new();
    b.document(N1);
    b.document(N2);
    b.access_to(N1, |a| a.agent(ALICE).mode(Mode::Read));
    let store = materialized(&b.into_nquads());

    let acc = readable(&store);
    assert!(acc.contains(N1), "the granted document IRI is the graph name: {:?}", acc);
    assert!(!acc.contains(N2), "an ungranted sibling document stays invisible: {:?}", acc);

    // `query_as` evaluates over exactly the advertised documents — no wider, no narrower.
    assert_eq!(scanned(&store), BTreeSet::from([N1.to_owned()]));
}

/// Authorization granularity is the whole named graph: a fragment-identified subject
/// (`<doc#it>`) is governed by the graph named by its FRAGMENT-LESS document IRI. There is
/// no per-fragment authorization, and `<doc#it>` is not itself a graph name.
#[test]
fn fragment_subjects_are_governed_by_their_document_graph() {
    let mut b = AclBuilder::new();
    b.document(N1); // emits <N1#it> <ex:prop> "x" <N1>
    b.access_to(N1, |a| a.agent(ALICE).mode(Mode::Read));
    let store = materialized(&b.into_nquads());

    let subjects = store
        .query_as(
            &alice(),
            Mode::Read,
            &format!("SELECT ?s WHERE {{ GRAPH ?g {{ ?s <{}> ?o }} }}", EX_PROP),
        )
        .expect("query ok");
    assert_eq!(subjects.rows.len(), 1, "the fragment subject rides its document's grant");

    // …and only its document's grant: an anonymous session sees nothing, so visibility is
    // the graph's verdict rather than a property of the subject.
    let anon = store
        .query_as(
            &Session::default(),
            Mode::Read,
            &format!("SELECT ?s WHERE {{ GRAPH ?g {{ ?s <{EX_PROP}> ?o }} }}"),
        )
        .expect("query ok");
    assert_eq!(anon.rows.len(), 0, "no grant, no fragment");

    let acc = readable(&store);
    assert!(!acc.contains(&format!("{N1}#it")), "a fragment is never a graph name: {:?}", acc);
}

// ─── Clause 2: the default graph carries no pod data ──────────────────────────────────

/// A triple the host loads into the DEFAULT graph is outside the contract: it is governed by
/// nothing and is invisible to the read path — including under the explicit union-default
/// opt-in, whose union is over the authorized NAMED graphs only.
#[test]
fn default_graph_pod_data_is_never_readable() {
    let mut b = AclBuilder::new();
    b.document(N1);
    b.access_to_and_default("https://pod.ex/", |a| a.agent(ALICE).mode(Mode::Read));
    let mut nq = b.into_nquads();
    // N-Triples line (no graph name) => the default graph.
    nq.push_str(&format!("<https://pod.ex/loose#it> <{EX_PROP}> \"loose\" .\n"));
    let store = materialized(&nq);

    // The union-default opt-in: the union is over the authorized NAMED graphs, so alice sees
    // the document ("x") — the positive control — and never the default-graph triple. Default
    // feature state only: the `legacy-union-default-graph` rewrite predates the spec-minted
    // IRI and treats it as an ordinary (absent) dataset reference, so the opt-in reads as a
    // restriction to nothing there.
    #[cfg(not(feature = "legacy-union-default-graph"))]
    {
        let opt_in = objects(
            &store,
            &format!("SELECT ?o FROM <{UNION_DEFAULT_GRAPH_IRI}> WHERE {{ ?s <{EX_PROP}> ?o }}"),
        );
        assert!(opt_in.contains("x"), "the granted named document is readable: {:?}", opt_in);
        assert!(!opt_in.contains("loose"), "default-graph data is not in the union: {:?}", opt_in);
    }

    // A bare default-graph pattern reaches the VIEW's default graph, which is empty
    // (`DefaultGraphMode::Empty`) — the clause that breaks if pod data is allowed to live in
    // the default graph. Holds in both feature states: with the `legacy-union-default-graph`
    // escape hatch a bare pattern ranges over the authorized union instead, which is still
    // named graphs only (and reads the document — the positive control for that state).
    let bare = objects(&store, &format!("SELECT ?o WHERE {{ ?s <{EX_PROP}> ?o }}"));
    #[cfg(feature = "legacy-union-default-graph")]
    assert!(bare.contains("x"), "legacy union-always still reads the document: {:?}", bare);
    assert!(!bare.contains("loose"), "default-graph data is never readable: {:?}", bare);
}

// ─── Clause 3: control documents are recognized by the `.acl` / `.acr` suffix ──────────

/// `<R> + ".acl"` is the WAC control document of `<R>` — by NAMING CONVENTION, not by any
/// triple in it. The same authorization triples in an identically-shaped graph under any
/// other name are inert pod content and grant nothing.
#[test]
fn control_document_suffix_is_load_bearing() {
    let mut b = AclBuilder::new();
    b.document(N1);
    b.access_to(N1, |a| a.agent(ALICE).mode(Mode::Read));
    let granting = b.into_nquads();
    assert!(readable(&materialized(&granting)).contains(N1), "the `.acl` graph governs N1");

    // Byte-identical corpus with the control graph renamed `.acl-backup`: same triples,
    // same subjects, no grant — the suffix is what makes a graph an ACL.
    let renamed = granting.replace(&format!("{N1}.acl"), &format!("{N1}.acl-backup"));
    assert_ne!(renamed, granting, "the rename actually rewrote the corpus");
    assert!(
        readable(&materialized(&renamed)).is_empty(),
        "authorization triples outside a `.acl`/`.acr` graph are inert pod content"
    );
}

/// A `.acl` graph is not ordinary content: reading it is gated by `acl:Control` on the
/// resource it governs, not by a `Read` grant on that resource. So a host must not name a
/// content document `*.acl` — its readability would follow the ACL rules, not its own.
#[test]
fn control_documents_are_gated_by_control_not_by_read() {
    let acl_of_n1 = format!("{N1}.acl");

    let mut read_only = AclBuilder::new();
    read_only.document(N1);
    read_only.access_to(N1, |a| a.agent(ALICE).mode(Mode::Read));
    let acc = readable(&materialized(&read_only.into_nquads()));
    assert!(acc.contains(N1), "Read on the document: {:?}", acc);
    assert!(!acc.contains(&acl_of_n1), "Read does not reach the governing ACL: {:?}", acc);

    let mut with_control = AclBuilder::new();
    with_control.document(N1);
    with_control.access_to(N1, |a| a.agent(ALICE).mode(Mode::Control));
    let acc = readable(&materialized(&with_control.into_nquads()));
    assert!(acc.contains(&acl_of_n1), "Control reaches the governing ACL: {:?}", acc);
}

/// The ACP half of clause 3, on the REAL `PodStore::new` → `materialize_acp` → `accessible`
/// path: `<R> + ".acr"` is the ACP control document of `<R>` by the same naming convention,
/// and the identical ACR triples under any other graph name are inert pod content.
#[test]
fn acr_control_document_suffix_is_load_bearing() {
    let mut acr = AcrBuilder::new();
    acr.access_control(N1, |p| p.allow(Mode::Read).any_of_agent(ALICE));
    acr.document(N1);
    let granting = acr.into_nquads();
    let acc = readable_acp(&granting);
    assert!(acc.contains(N1), "the `.acr` graph governs N1: {:?}", acc);

    // Byte-identical corpus with the ACR graph renamed `.acr-backup`: same policies, same
    // matchers, no grant — as with `.acl`, the suffix is what makes a graph a control doc.
    let renamed = granting.replace(&format!("{N1}.acr"), &format!("{N1}.acr-backup"));
    assert_ne!(renamed, granting, "the rename actually rewrote the corpus");
    let acc = readable_acp(&renamed);
    assert!(
        acc.is_empty(),
        "ACP policies outside a `.acr` graph are inert pod content: {:?}",
        acc
    );
}

// ─── Clause 4: containment comes from the graph-name IRI path ─────────────────────────

/// Container inheritance is derived from the SLASH STRUCTURE of the graph name, not from
/// containment triples: a deeply nested document inherits the root container's `acl:default`
/// even though none of its intermediate containers has a graph of its own — and those
/// synthesized containers show up in `accessible` as inheritance anchors. A document under a
/// different authority shares no ancestor and inherits nothing.
#[test]
fn containment_is_derived_from_the_graph_name_path() {
    let deep = "https://pod.ex/a/b/deep";
    let foreign = "https://other.ex/a/b/deep";
    let mut b = AclBuilder::new();
    b.document(deep);
    b.document(foreign);
    b.default_for("https://pod.ex/", |a| a.agent(ALICE).mode(Mode::Read));
    let store = materialized(&b.into_nquads());

    let acc = readable(&store);
    assert!(acc.contains(deep), "the nested document inherits acl:default: {:?}", acc);
    assert!(
        acc.contains("https://pod.ex/a/") && acc.contains("https://pod.ex/a/b/"),
        "containers with no graph of their own still exist as inheritance anchors: {:?}",
        acc
    );
    assert!(
        !acc.contains(foreign),
        "a document under another authority shares no ancestor container: {:?}",
        acc
    );

    // The advertised set is authorized RESOURCE names, NOT loaded documents — the two
    // container anchors above have no graph of their own. `view_for` consumes the set as a
    // named-graph visibility whitelist, so an authorized name with no graph matches nothing
    // and contributes no data: the scan sees the one document and neither anchor.
    assert_eq!(scanned(&store), BTreeSet::from([deep.to_owned()]));
}
