//! A term-level read view over a dictionary-encoded [`sparq_core::Graph`].
//!
//! SHACL evaluation is term-oriented (focus nodes, value nodes, shape parameters
//! are RDF terms), so this wraps the id-level permutation scans of the store and
//! materialises `oxrdf::Term`s at the boundary. Every lookup is index-backed
//! (bound-prefix range scan on the best permutation), never a full-graph filter.

use oxrdf::{NamedNode, Term};
use sparq_core::Graph;

pub(crate) const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
pub(crate) const RDF_FIRST: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#first";
pub(crate) const RDF_REST: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#rest";
pub(crate) const RDF_NIL: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#nil";
pub(crate) const RDFS_SUBCLASS_OF: &str = "http://www.w3.org/2000/01/rdf-schema#subClassOf";

/// A borrowed, term-level view of a graph.
#[derive(Clone, Copy)]
pub struct GraphView<'g> {
    g: &'g Graph,
}

impl<'g> GraphView<'g> {
    pub fn new(g: &'g Graph) -> Self {
        GraphView { g }
    }

    /// All triples matching the optional-term pattern, as `[s, p, o]` terms.
    /// A bound term absent from the graph's dictionary matches nothing.
    pub fn triples(&self, s: Option<&Term>, p: Option<&str>, o: Option<&Term>) -> Vec<[Term; 3]> {
        let p_node = match p {
            None => None,
            Some(iri) => match NamedNode::new(iri) {
                Ok(n) => Some(n),
                Err(_) => return Vec::new(),
            },
        };
        let Some(pattern) = self.g.pattern(s, p_node.as_ref(), o) else {
            return Vec::new();
        };
        let scan = self.g.store.scan(&pattern);
        scan.rows
            .iter()
            .map(|r| {
                let [s, p, o] = scan.to_spo(r);
                [
                    self.g.dict.term(s),
                    self.g.dict.term(p),
                    self.g.dict.term(o),
                ]
            })
            .collect()
    }

    /// True iff the fully-ground triple is present.
    pub fn contains(&self, s: &Term, p: &str, o: &Term) -> bool {
        let Ok(p) = NamedNode::new(p) else {
            return false;
        };
        match self.g.pattern(Some(s), Some(&p), Some(o)) {
            Some([Some(s), Some(p), Some(o)]) => self.g.store.contains([s, p, o]),
            _ => false,
        }
    }

    pub fn objects(&self, s: &Term, p: &str) -> Vec<Term> {
        self.triples(Some(s), Some(p), None)
            .into_iter()
            .map(|[_, _, o]| o)
            .collect()
    }

    pub fn object(&self, s: &Term, p: &str) -> Option<Term> {
        self.objects(s, p).into_iter().next()
    }

    pub fn subjects(&self, p: &str, o: &Term) -> Vec<Term> {
        self.triples(None, Some(p), Some(o))
            .into_iter()
            .map(|[s, _, _]| s)
            .collect()
    }

    /// Distinct subjects of any triple with predicate `p`.
    pub fn subjects_of(&self, p: &str) -> Vec<Term> {
        dedup(
            self.triples(None, Some(p), None)
                .into_iter()
                .map(|[s, _, _]| s),
        )
    }

    /// Distinct objects of any triple with predicate `p`.
    pub fn objects_of(&self, p: &str) -> Vec<Term> {
        dedup(
            self.triples(None, Some(p), None)
                .into_iter()
                .map(|[_, _, o]| o),
        )
    }

    /// All `(predicate, object)` pairs of triples with subject `s` (sh:closed).
    pub fn predicate_objects(&self, s: &Term) -> Vec<(Term, Term)> {
        self.triples(Some(s), None, None)
            .into_iter()
            .map(|[_, p, o]| (p, o))
            .collect()
    }

    /// The lexical form of the first literal object, if any.
    pub fn str_object(&self, s: &Term, p: &str) -> Option<String> {
        match self.object(s, p) {
            Some(Term::Literal(l)) => Some(l.value().to_string()),
            _ => None,
        }
    }

    /// True iff `s` has `rdf:type o`.
    pub fn has_type(&self, s: &Term, class: &str) -> bool {
        match NamedNode::new(class) {
            Ok(c) => self.contains(s, RDF_TYPE, &Term::NamedNode(c)),
            Err(_) => false,
        }
    }

    /// Walks an `rdf:first`/`rdf:rest` list from `head` (cycle-safe).
    pub fn list(&self, head: &Term) -> Vec<Term> {
        let mut out = Vec::new();
        let mut seen: rustc_hash::FxHashSet<Term> = rustc_hash::FxHashSet::default();
        let mut cur = head.clone();
        loop {
            if matches!(&cur, Term::NamedNode(n) if n.as_str() == RDF_NIL) {
                break;
            }
            if !seen.insert(cur.clone()) {
                break; // malformed cyclic list
            }
            match self.object(&cur, RDF_FIRST) {
                Some(f) => out.push(f),
                None => break, // not a (well-formed) list node
            }
            match self.object(&cur, RDF_REST) {
                Some(r) => cur = r,
                None => break,
            }
        }
        out
    }

    /// The SHACL instances of `class`: nodes whose `rdf:type` is `class` or any
    /// `rdfs:subClassOf`-descendant of it (closure computed within this graph).
    pub fn instances_of(&self, class: &Term) -> Vec<Term> {
        let mut out = Vec::new();
        let mut seen: rustc_hash::FxHashSet<Term> = rustc_hash::FxHashSet::default();
        for c in self.subclass_closure(class) {
            for s in self.subjects(RDF_TYPE, &c) {
                if seen.insert(s.clone()) {
                    out.push(s);
                }
            }
        }
        out
    }

    /// `class` plus all its `rdfs:subClassOf`-descendants (reflexive-transitive).
    pub fn subclass_closure(&self, class: &Term) -> Vec<Term> {
        let mut out = vec![class.clone()];
        let mut seen: rustc_hash::FxHashSet<Term> = std::iter::once(class.clone()).collect();
        let mut i = 0;
        while i < out.len() {
            let c = out[i].clone();
            for sub in self.subjects(RDFS_SUBCLASS_OF, &c) {
                if seen.insert(sub.clone()) {
                    out.push(sub);
                }
            }
            i += 1;
        }
        out
    }

    /// True iff `node` is a SHACL instance of `class` in this graph.
    pub fn is_instance_of(&self, node: &Term, class: &Term) -> bool {
        if matches!(node, Term::Literal(_)) {
            return false;
        }
        for ty in self.objects(node, RDF_TYPE) {
            if &ty == class {
                return true;
            }
            // Walk ty's superclasses looking for `class`.
            let mut stack = vec![ty];
            let mut seen: rustc_hash::FxHashSet<Term> = rustc_hash::FxHashSet::default();
            while let Some(t) = stack.pop() {
                if !seen.insert(t.clone()) {
                    continue;
                }
                for sup in self.objects(&t, RDFS_SUBCLASS_OF) {
                    if &sup == class {
                        return true;
                    }
                    stack.push(sup);
                }
            }
        }
        false
    }
}

/// Order-preserving dedup.
pub(crate) fn dedup(items: impl IntoIterator<Item = Term>) -> Vec<Term> {
    let mut seen: rustc_hash::FxHashSet<Term> = rustc_hash::FxHashSet::default();
    let mut out = Vec::new();
    for t in items {
        if seen.insert(t.clone()) {
            out.push(t);
        }
    }
    out
}
