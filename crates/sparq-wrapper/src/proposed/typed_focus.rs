// [SONNET-4.6] sq-1rg2q.2: typed focus kinds + bound node factories
// Source: rdfjs/wrapper PRs #83-#87 (proposed, not yet landed upstream).
//
// INVARIANT: `SubjectNode` and `PredicateNode` enforce RDF positional legality
// at compile time via a sealed `IntoSubject` / `IntoPredicate` trait —
// `Literal` does not implement either, so passing one to `BoundFactory::subject`
// or `BoundFactory::predicate` is a compile error rather than a runtime panic.

use crate::{Node, NodeSet, Store};
use oxrdf::{BlankNode, NamedNode, Term};
use sparq_core::Graph;
use std::fmt;

// ---------------------------------------------------------------------------
// Sealed trait scaffolding – positional legality at compile time
// ---------------------------------------------------------------------------

mod sealed {
    pub trait Sealed {}
    impl Sealed for oxrdf::NamedNode {}
    impl Sealed for oxrdf::BlankNode {}
}

/// A term that is valid in the RDF subject position (IRI or blank node).
///
/// This trait is sealed: `Literal` does not implement it, so supplying one
/// to [`BoundFactory::subject`] is a **compile error**, not a runtime panic.
pub trait IntoSubject: sealed::Sealed {
    #[doc(hidden)]
    fn into_subject_focus(self) -> SubjectFocus;
}

impl IntoSubject for NamedNode {
    fn into_subject_focus(self) -> SubjectFocus {
        SubjectFocus::Named(self)
    }
}

impl IntoSubject for BlankNode {
    fn into_subject_focus(self) -> SubjectFocus {
        SubjectFocus::Blank(self)
    }
}

// ---------------------------------------------------------------------------
// Focus-kind enums
// ---------------------------------------------------------------------------

/// A term in the RDF subject position: either a named node (IRI) or a blank node.
///
/// This type cannot be constructed from a `Literal`; see [`IntoSubject`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubjectFocus {
    /// An IRI subject.
    Named(NamedNode),
    /// A blank node subject.
    Blank(BlankNode),
}

impl SubjectFocus {
    /// Returns the underlying `NamedNode` if this is one.
    pub fn as_named_node(&self) -> Option<&NamedNode> {
        match self {
            Self::Named(n) => Some(n),
            Self::Blank(_) => None,
        }
    }

    /// Returns the underlying `BlankNode` if this is one.
    pub fn as_blank_node(&self) -> Option<&BlankNode> {
        match self {
            Self::Named(_) => None,
            Self::Blank(b) => Some(b),
        }
    }
}

impl From<SubjectFocus> for Term {
    fn from(focus: SubjectFocus) -> Term {
        match focus {
            SubjectFocus::Named(n) => Term::NamedNode(n),
            SubjectFocus::Blank(b) => Term::BlankNode(b),
        }
    }
}

impl fmt::Display for SubjectFocus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Named(n) => fmt::Display::fmt(n, f),
            Self::Blank(b) => fmt::Display::fmt(b, f),
        }
    }
}

// ---------------------------------------------------------------------------
// Typed node wrappers
// ---------------------------------------------------------------------------

/// A node whose focus is known to be valid in the RDF **subject** position.
///
/// Because [`BoundFactory::subject`] requires an [`IntoSubject`] argument,
/// it is impossible to construct a `SubjectNode` around a `Literal`.
///
/// # Compile-fail guarantee
///
/// ```compile_fail
/// # use oxrdf::Literal;
/// # use sparq_core::Graph;
/// # use sparq_wrapper::Store;
/// # use sparq_wrapper::proposed::typed_focus::BoundFactory;
/// # let graph = Graph::new();
/// # let store = Store::borrowed(&graph);
/// # let factory = BoundFactory::from_store(&store);
/// // Literal does not implement IntoSubject — compile error.
/// let _ = factory.subject(Literal::new_simple_literal("bad"));
/// ```
pub struct SubjectNode<'g> {
    graph: &'g Graph,
    focus: SubjectFocus,
}

impl fmt::Debug for SubjectNode<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SubjectNode")
            .field("focus", &self.focus)
            .finish_non_exhaustive()
    }
}

impl Clone for SubjectNode<'_> {
    fn clone(&self) -> Self {
        Self {
            graph: self.graph,
            focus: self.focus.clone(),
        }
    }
}

impl<'g> SubjectNode<'g> {
    /// Returns the typed subject focus for this node.
    pub fn focus(&self) -> &SubjectFocus {
        &self.focus
    }

    /// Returns the focus as a generic RDF `Term`.
    pub fn as_term(&self) -> Term {
        self.focus.clone().into()
    }

    /// Traverses `(focus, predicate, object)` and returns the matched objects.
    pub fn out(&self, predicate: &NamedNode) -> NodeSet<'g> {
        Node::from_raw(self.graph, self.as_term()).out(predicate)
    }

    /// Traverses `(subject, predicate, focus)` and returns the matched subjects.
    pub fn r#in(&self, predicate: &NamedNode) -> NodeSet<'g> {
        Node::from_raw(self.graph, self.as_term()).r#in(predicate)
    }

    /// Returns an untyped [`Node`] view of this subject, giving access to the
    /// full traversal and typed-accessor API.
    pub fn as_node(&self) -> Node<'g> {
        Node::from_raw(self.graph, self.as_term())
    }
}

/// A node whose focus is known to be valid in the RDF **predicate** position
/// (a named node; blank nodes and literals are excluded).
pub struct PredicateNode<'g> {
    graph: &'g Graph,
    focus: NamedNode,
}

impl fmt::Debug for PredicateNode<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PredicateNode")
            .field("focus", &self.focus)
            .finish_non_exhaustive()
    }
}

impl Clone for PredicateNode<'_> {
    fn clone(&self) -> Self {
        Self {
            graph: self.graph,
            focus: self.focus.clone(),
        }
    }
}

impl<'g> PredicateNode<'g> {
    /// Returns the underlying `NamedNode`.
    pub fn named_node(&self) -> &NamedNode {
        &self.focus
    }

    /// Returns the focus as a generic RDF `Term`.
    pub fn as_term(&self) -> Term {
        Term::NamedNode(self.focus.clone())
    }

    /// Returns an untyped [`Node`] view of this predicate.
    pub fn as_node(&self) -> Node<'g> {
        Node::from_raw(self.graph, self.as_term())
    }
}

/// A node in the RDF **object** position: any term (IRI, blank node, or literal).
pub struct ObjectNode<'g> {
    graph: &'g Graph,
    focus: Term,
}

impl fmt::Debug for ObjectNode<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ObjectNode")
            .field("focus", &self.focus)
            .finish_non_exhaustive()
    }
}

impl Clone for ObjectNode<'_> {
    fn clone(&self) -> Self {
        Self {
            graph: self.graph,
            focus: self.focus.clone(),
        }
    }
}

impl<'g> ObjectNode<'g> {
    /// Returns the focus term.
    pub fn term(&self) -> &Term {
        &self.focus
    }

    /// Traverses `(focus, predicate, object)` and returns the matched objects.
    ///
    /// Note: traversal from a `Literal` focus will always return an empty set
    /// because literals cannot appear in the subject position of stored triples.
    pub fn out(&self, predicate: &NamedNode) -> NodeSet<'g> {
        Node::from_raw(self.graph, self.focus.clone()).out(predicate)
    }

    /// Returns an untyped [`Node`] view of this object node.
    pub fn as_node(&self) -> Node<'g> {
        Node::from_raw(self.graph, self.focus.clone())
    }
}

// ---------------------------------------------------------------------------
// BoundFactory
// ---------------------------------------------------------------------------

/// A factory bound to a borrowed graph that manufactures typed focus nodes
/// without cloning the dataset.
///
/// A single factory can produce any number of [`SubjectNode`], [`PredicateNode`],
/// or [`ObjectNode`] values over the same underlying graph borrow, amortising
/// the store-bind cost across all nodes.
///
/// # Example
///
/// ```
/// use oxrdf::NamedNode;
/// use sparq_core::Graph;
/// use sparq_wrapper::Store;
/// use sparq_wrapper::proposed::typed_focus::BoundFactory;
///
/// let graph = Graph::load_str(
///     "@prefix ex: <http://example.org/> .
///      ex:alice ex:knows ex:bob .
///      ex:bob   ex:knows ex:carol .",
///     "turtle",
/// )
/// .unwrap();
/// let store = Store::borrowed(&graph);
/// let factory = BoundFactory::from_store(&store);
///
/// let alice = factory.subject(NamedNode::new("http://example.org/alice").unwrap());
/// let bob   = factory.subject(NamedNode::new("http://example.org/bob").unwrap());
/// let knows = NamedNode::new("http://example.org/knows").unwrap();
///
/// let alice_friends: Vec<_> = alice.out(&knows).collect();
/// let bob_friends:   Vec<_> = bob.out(&knows).collect();
/// assert_eq!(alice_friends.len(), 1);
/// assert_eq!(bob_friends.len(), 1);
/// ```
pub struct BoundFactory<'g> {
    graph: &'g Graph,
}

impl<'g> BoundFactory<'g> {
    /// Binds a factory to the graph inside `store`.
    ///
    /// The factory borrows the graph for its lifetime `'g`; it produces nodes
    /// that share that borrow without copying or materialising the dataset.
    pub fn from_store(store: &'g Store<'_>) -> Self {
        Self {
            graph: store.graph(),
        }
    }

    /// Creates a subject-position node for the given term.
    ///
    /// Only `NamedNode` and `BlankNode` implement [`IntoSubject`]; passing a
    /// `Literal` is a **compile error** — see [`SubjectNode`] for the doctest.
    pub fn subject<T: IntoSubject>(&self, term: T) -> SubjectNode<'g> {
        SubjectNode {
            graph: self.graph,
            focus: term.into_subject_focus(),
        }
    }

    /// Creates a predicate-position node.
    ///
    /// Only `NamedNode` is a valid RDF predicate; this method documents that
    /// constraint explicitly.
    pub fn predicate(&self, term: NamedNode) -> PredicateNode<'g> {
        PredicateNode {
            graph: self.graph,
            focus: term,
        }
    }

    /// Creates an object-position node for any RDF term.
    ///
    /// Object position accepts IRIs, blank nodes, and literals.
    pub fn object(&self, term: impl Into<Term>) -> ObjectNode<'g> {
        ObjectNode {
            graph: self.graph,
            focus: term.into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Store;
    use oxrdf::Term;
    use sparq_core::Graph;

    fn iri(local: &str) -> NamedNode {
        NamedNode::new(format!("http://example.org/{local}")).unwrap()
    }

    fn blank(id: &str) -> BlankNode {
        BlankNode::new(id).unwrap()
    }

    fn load_graph(ttl: &str) -> Graph {
        Graph::load_str(ttl, "turtle").unwrap()
    }

    // -----------------------------------------------------------------------
    // SubjectFocus classification
    // -----------------------------------------------------------------------

    #[test]
    fn subject_focus_named_accessor() {
        let n = iri("alice");
        let focus = n.clone().into_subject_focus();
        assert_eq!(focus.as_named_node(), Some(&n));
        assert_eq!(focus.as_blank_node(), None);
    }

    #[test]
    fn subject_focus_blank_accessor() {
        let b = blank("b0");
        let focus = b.clone().into_subject_focus();
        assert_eq!(focus.as_blank_node(), Some(&b));
        assert_eq!(focus.as_named_node(), None);
    }

    #[test]
    fn subject_focus_into_term_roundtrip_named() {
        let n = iri("x");
        let term: Term = n.clone().into_subject_focus().into();
        assert_eq!(term, Term::NamedNode(n));
    }

    #[test]
    fn subject_focus_into_term_roundtrip_blank() {
        let b = blank("b1");
        let term: Term = b.clone().into_subject_focus().into();
        assert_eq!(term, Term::BlankNode(b));
    }

    // -----------------------------------------------------------------------
    // BoundFactory: construction witnesses
    // -----------------------------------------------------------------------

    #[test]
    fn bound_factory_subject_named_holds_correct_focus() {
        let g = Graph::new();
        let store = Store::borrowed(&g);
        let factory = BoundFactory::from_store(&store);
        let node = factory.subject(iri("alice"));
        assert_eq!(node.focus().as_named_node(), Some(&iri("alice")));
    }

    #[test]
    fn bound_factory_subject_blank_holds_correct_focus() {
        let g = Graph::new();
        let store = Store::borrowed(&g);
        let factory = BoundFactory::from_store(&store);
        let node = factory.subject(blank("b0"));
        assert_eq!(node.focus().as_blank_node(), Some(&blank("b0")));
    }

    #[test]
    fn bound_factory_predicate_holds_named_node() {
        let g = Graph::new();
        let store = Store::borrowed(&g);
        let factory = BoundFactory::from_store(&store);
        let pn = factory.predicate(iri("knows"));
        assert_eq!(pn.named_node(), &iri("knows"));
        assert_eq!(pn.as_term(), Term::NamedNode(iri("knows")));
    }

    #[test]
    fn bound_factory_object_accepts_literal() {
        use oxrdf::Literal;
        let g = Graph::new();
        let store = Store::borrowed(&g);
        let factory = BoundFactory::from_store(&store);
        let obj = factory.object(Literal::new_simple_literal("hello"));
        assert!(matches!(obj.term(), Term::Literal(_)));
    }

    // -----------------------------------------------------------------------
    // Acceptance: one factory, multiple subjects, same borrowed store
    // -----------------------------------------------------------------------

    #[test]
    fn bound_factory_wraps_multiple_terms_over_same_store_and_traverses() {
        let ttl = "@prefix ex: <http://example.org/> .
                   ex:alice ex:knows ex:bob .
                   ex:alice ex:knows ex:carol .
                   ex:bob   ex:knows ex:dave .";
        let g = load_graph(ttl);
        let store = Store::borrowed(&g);
        let factory = BoundFactory::from_store(&store);

        // Both subjects created from the SAME factory / same graph borrow.
        let alice = factory.subject(iri("alice"));
        let bob = factory.subject(iri("bob"));
        let knows = iri("knows");

        let mut alice_friends: Vec<Term> = alice.out(&knows).map(|n| n.into_term()).collect();
        alice_friends.sort_by_key(|t| t.to_string());

        let bob_friends: Vec<Term> = bob.out(&knows).map(|n| n.into_term()).collect();

        // alice knows bob + carol (order unspecified)
        assert_eq!(alice_friends.len(), 2);
        assert!(alice_friends.contains(&Term::NamedNode(iri("bob"))));
        assert!(alice_friends.contains(&Term::NamedNode(iri("carol"))));

        // bob knows dave
        assert_eq!(bob_friends.len(), 1);
        assert_eq!(bob_friends[0], Term::NamedNode(iri("dave")));
    }

    // -----------------------------------------------------------------------
    // Reverse traversal from a subject node
    // -----------------------------------------------------------------------

    #[test]
    fn subject_node_in_traversal_finds_inbound_subjects() {
        let ttl = "@prefix ex: <http://example.org/> .
                   ex:alice ex:knows ex:bob .
                   ex:carol ex:knows ex:bob .";
        let g = load_graph(ttl);
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

    // -----------------------------------------------------------------------
    // ObjectNode: out-traversal follows IRI objects as subjects
    // -----------------------------------------------------------------------

    #[test]
    fn object_node_out_traversal_follows_iri_objects_as_subjects() {
        let ttl = "@prefix ex: <http://example.org/> .
                   ex:alice ex:knows ex:bob .
                   ex:bob   ex:name  \"Bob\" .";
        let g = load_graph(ttl);
        let store = Store::borrowed(&g);
        let factory = BoundFactory::from_store(&store);

        let alice = factory.subject(iri("alice"));
        let knows = iri("knows");
        let name = iri("name");

        // alice's knows-target is bob; re-wrap bob as an ObjectNode and traverse.
        let bob_term = alice.out(&knows).next().unwrap().into_term();
        let bob_obj = factory.object(bob_term);

        let names: Vec<_> = bob_obj.out(&name).collect();
        assert_eq!(names.len(), 1);
        assert_eq!(names[0].as_str().unwrap(), "Bob");
    }

    // -----------------------------------------------------------------------
    // ObjectNode: literal object out-traversal is always empty
    // -----------------------------------------------------------------------

    #[test]
    fn object_node_out_traversal_from_literal_is_empty() {
        use oxrdf::Literal;
        let g = Graph::new();
        let store = Store::borrowed(&g);
        let factory = BoundFactory::from_store(&store);
        let lit_obj = factory.object(Literal::new_simple_literal("leaf"));
        let results: Vec<_> = lit_obj.out(&iri("anything")).collect();
        assert!(results.is_empty(), "literal out-traversal must be empty");
    }

    // -----------------------------------------------------------------------
    // as_node escape hatch
    // -----------------------------------------------------------------------

    #[test]
    fn subject_node_as_node_returns_correct_focus() {
        let g = Graph::new();
        let store = Store::borrowed(&g);
        let factory = BoundFactory::from_store(&store);
        let alice = factory.subject(iri("alice"));
        assert_eq!(alice.as_node().focus(), &Term::NamedNode(iri("alice")));
    }

    #[test]
    fn predicate_node_as_node_returns_correct_focus() {
        let g = Graph::new();
        let store = Store::borrowed(&g);
        let factory = BoundFactory::from_store(&store);
        let pred = factory.predicate(iri("knows"));
        assert_eq!(pred.as_node().focus(), &Term::NamedNode(iri("knows")));
    }

    // -----------------------------------------------------------------------
    // Mutation witness: SubjectNode as_term reflects focus correctly
    // -----------------------------------------------------------------------

    #[test]
    fn subject_node_as_term_matches_original_named_node() {
        let n = iri("resource");
        let g = Graph::new();
        let store = Store::borrowed(&g);
        let factory = BoundFactory::from_store(&store);
        let node = factory.subject(n.clone());
        assert_eq!(node.as_term(), Term::NamedNode(n));
    }
}
