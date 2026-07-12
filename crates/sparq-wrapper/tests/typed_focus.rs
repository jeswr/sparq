// [SONNET-4.6] sq-1rg2q.2: integration tests for typed focus kinds + bound node factories.
// These run with `--features proposed-focus-kinds` only.

#![cfg(feature = "proposed-focus-kinds")]

use oxrdf::{BlankNode, Literal, NamedNode, Term};
use sparq_core::Graph;
use sparq_wrapper::proposed::typed_focus::{BoundFactory, SubjectFocus};
use sparq_wrapper::Store;

fn iri(local: &str) -> NamedNode {
    NamedNode::new(format!("http://example.org/{local}")).unwrap()
}

fn load(ttl: &str) -> Graph {
    Graph::load_str(ttl, "turtle").unwrap()
}

// ---------------------------------------------------------------------------
// Acceptance test: one factory, multiple subjects, same borrowed store
// ---------------------------------------------------------------------------

#[test]
fn acceptance_bound_factory_wraps_multiple_terms_and_traverses() {
    let g = load(
        "@prefix ex: <http://example.org/> .
         ex:alice ex:knows ex:bob .
         ex:alice ex:knows ex:carol .
         ex:bob   ex:knows ex:dave .",
    );
    let store = Store::borrowed(&g);
    let factory = BoundFactory::from_store(&store);

    let alice = factory.subject(iri("alice"));
    let bob = factory.subject(iri("bob"));
    let knows = iri("knows");

    let mut alice_friends: Vec<Term> = alice.out(&knows).map(|n| n.into_term()).collect();
    alice_friends.sort_by_key(|t| t.to_string());

    let bob_friends: Vec<Term> = bob.out(&knows).map(|n| n.into_term()).collect();

    assert_eq!(alice_friends.len(), 2, "alice knows two people");
    assert!(
        alice_friends.contains(&Term::NamedNode(iri("bob"))),
        "alice knows bob"
    );
    assert!(
        alice_friends.contains(&Term::NamedNode(iri("carol"))),
        "alice knows carol"
    );
    assert_eq!(bob_friends.len(), 1, "bob knows one person");
    assert_eq!(bob_friends[0], Term::NamedNode(iri("dave")));
}

// ---------------------------------------------------------------------------
// SubjectFocus classification
// ---------------------------------------------------------------------------

#[test]
fn subject_focus_named_node_classifies_correctly() {
    let n = iri("alice");
    // NamedNode implements IntoSubject; build the focus via the factory.
    let g = Graph::new();
    let store = Store::borrowed(&g);
    let factory = BoundFactory::from_store(&store);
    let node = factory.subject(n.clone());
    match node.focus() {
        SubjectFocus::Named(found) => assert_eq!(found, &n),
        SubjectFocus::Blank(_) => panic!("expected Named variant"),
    }
}

#[test]
fn subject_focus_blank_node_classifies_correctly() {
    let b = BlankNode::new("b0").unwrap();
    let g = Graph::new();
    let store = Store::borrowed(&g);
    let factory = BoundFactory::from_store(&store);
    let node = factory.subject(b.clone());
    match node.focus() {
        SubjectFocus::Blank(found) => assert_eq!(found, &b),
        SubjectFocus::Named(_) => panic!("expected Blank variant"),
    }
}

// ---------------------------------------------------------------------------
// Predicate node
// ---------------------------------------------------------------------------

#[test]
fn predicate_node_holds_named_node() {
    let g = Graph::new();
    let store = Store::borrowed(&g);
    let factory = BoundFactory::from_store(&store);
    let pred = factory.predicate(iri("knows"));
    assert_eq!(pred.named_node(), &iri("knows"));
    assert_eq!(pred.as_term(), Term::NamedNode(iri("knows")));
}

// ---------------------------------------------------------------------------
// Object node
// ---------------------------------------------------------------------------

#[test]
fn object_node_accepts_literal() {
    let g = Graph::new();
    let store = Store::borrowed(&g);
    let factory = BoundFactory::from_store(&store);
    let obj = factory.object(Literal::new_simple_literal("hello"));
    assert!(matches!(obj.term(), Term::Literal(_)));
}

#[test]
fn object_node_out_traversal_from_literal_is_empty() {
    let g = Graph::new();
    let store = Store::borrowed(&g);
    let factory = BoundFactory::from_store(&store);
    let lit = factory.object(Literal::new_simple_literal("leaf"));
    let results: Vec<_> = lit.out(&iri("anything")).collect();
    assert!(results.is_empty(), "literal out-traversal must always be empty");
}

#[test]
fn object_node_out_traversal_follows_iri_object_as_new_subject() {
    let g = load(
        "@prefix ex: <http://example.org/> .
         ex:alice ex:knows ex:bob .
         ex:bob   ex:name  \"Bob\" .",
    );
    let store = Store::borrowed(&g);
    let factory = BoundFactory::from_store(&store);

    let alice = factory.subject(iri("alice"));
    let knows = iri("knows");
    let name = iri("name");

    let bob_term = alice.out(&knows).next().unwrap().into_term();
    let bob_obj = factory.object(bob_term);
    let names: Vec<_> = bob_obj.out(&name).collect();
    assert_eq!(names.len(), 1);
    assert_eq!(names[0].as_str().unwrap(), "Bob");
}

// ---------------------------------------------------------------------------
// Reverse traversal
// ---------------------------------------------------------------------------

#[test]
fn subject_node_in_traversal_finds_all_inbound_subjects() {
    let g = load(
        "@prefix ex: <http://example.org/> .
         ex:alice ex:knows ex:bob .
         ex:carol ex:knows ex:bob .",
    );
    let store = Store::borrowed(&g);
    let factory = BoundFactory::from_store(&store);

    let bob = factory.subject(iri("bob"));
    let knows = iri("knows");
    let mut inbound: Vec<Term> = bob.r#in(&knows).map(|n| n.into_term()).collect();
    inbound.sort_by_key(|t| t.to_string());

    assert_eq!(inbound.len(), 2);
    assert!(inbound.contains(&Term::NamedNode(iri("alice"))));
    assert!(inbound.contains(&Term::NamedNode(iri("carol"))));
}

// ---------------------------------------------------------------------------
// as_node escape hatch
// ---------------------------------------------------------------------------

#[test]
fn subject_node_as_node_exposes_full_traversal_api() {
    let g = load(
        "@prefix ex: <http://example.org/> .
         ex:alice ex:name \"Alice\" .",
    );
    let store = Store::borrowed(&g);
    let factory = BoundFactory::from_store(&store);
    let alice = factory.subject(iri("alice"));
    let name = iri("name");

    // as_node gives the untyped Node with the full accessor API.
    let value = alice.as_node().out(&name).next().unwrap().as_str().unwrap().to_owned();
    assert_eq!(value, "Alice");
}

// ---------------------------------------------------------------------------
// Mutation-witnessed: values survive graph rebuild
// ---------------------------------------------------------------------------

#[test]
fn factory_on_owned_store_traverses_inserted_triples() {
    let mut store = Store::new();
    store
        .insert(iri("alice"), iri("knows"), iri("bob"))
        .unwrap();

    // Build a new borrowed store view so the borrow is fresh after mutation.
    let g_ref = store.graph();
    let view = Store::borrowed(g_ref);
    let factory = BoundFactory::from_store(&view);

    let alice = factory.subject(iri("alice"));
    let knows = iri("knows");
    let friends: Vec<Term> = alice.out(&knows).map(|n| n.into_term()).collect();

    assert_eq!(friends.len(), 1);
    assert_eq!(friends[0], Term::NamedNode(iri("bob")));
}
