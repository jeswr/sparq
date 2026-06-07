//! sparq-core: dictionary-encoded RDF storage with six permutation indexes.
//!
//! This is the storage substrate for the query engine: a [`Graph`] holds the
//! term [`Dict`]ionary and the [`TripleStore`] (six sorted permutations), and
//! is built from an RDF document via the bulk loader.

pub mod dict;
pub mod store;

use dict::{Dict, Id};
use oxrdf::{NamedNode, Term};
use oxttl::{NQuadsParser, NTriplesParser, TriGParser, TurtleParser};
use store::{Pattern, TripleStore};

/// An immutable, dictionary-encoded RDF graph ready for querying.
pub struct Graph {
    pub dict: Dict,
    pub store: TripleStore,
}

impl Graph {
    /// Loads triples from an RDF document (default graph only for M1; named
    /// graphs from TriG/N-Quads are folded into the default graph). Returns the
    /// built graph. `format`: "turtle" | "ntriples" | "nquads" | "trig".
    pub fn load_str(text: &str, format: &str) -> Result<Graph, String> {
        let mut dict = Dict::new();
        let mut triples: Vec<[Id; 3]> = Vec::new();
        let bytes = text.as_bytes();

        macro_rules! push_triple {
            ($s:expr, $p:expr, $o:expr) => {{
                let s = dict.intern(&subject_term($s));
                let p = dict.intern(&Term::NamedNode($p.clone()));
                let o = dict.intern($o);
                triples.push([s, p, o]);
            }};
        }

        match format {
            "ntriples" | "n-triples" => {
                for t in NTriplesParser::new().for_slice(bytes) {
                    let t = t.map_err(|e| e.to_string())?;
                    push_triple!(&t.subject, &t.predicate, &t.object);
                }
            }
            "nquads" | "n-quads" => {
                for q in NQuadsParser::new().for_slice(bytes) {
                    let q = q.map_err(|e| e.to_string())?;
                    push_triple!(&q.subject, &q.predicate, &q.object);
                }
            }
            "trig" | "application/trig" => {
                for q in TriGParser::new().for_slice(bytes) {
                    let q = q.map_err(|e| e.to_string())?;
                    push_triple!(&q.subject, &q.predicate, &q.object);
                }
            }
            _ => {
                for t in TurtleParser::new().for_slice(bytes) {
                    let t = t.map_err(|e| e.to_string())?;
                    push_triple!(&t.subject, &t.predicate, &t.object);
                }
            }
        }

        let store = TripleStore::from_triples(triples);
        Ok(Graph { dict, store })
    }

    pub fn len(&self) -> usize {
        self.store.len()
    }

    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }

    /// Resolves a term to its id, or `None` if the term is absent (so a pattern
    /// bound to it cannot match).
    pub fn id_of(&self, term: &Term) -> Option<Id> {
        let id = self.dict.lookup(term);
        if id == dict::NO_ID {
            None
        } else {
            Some(id)
        }
    }

    /// Builds an id pattern from optional terms; returns `None` if any bound
    /// term is absent from the dictionary.
    pub fn pattern(
        &self,
        s: Option<&Term>,
        p: Option<&NamedNode>,
        o: Option<&Term>,
    ) -> Option<Pattern> {
        let resolve = |t: Option<&Term>| -> Option<Option<Id>> {
            match t {
                None => Some(None),
                Some(t) => self.id_of(t).map(Some),
            }
        };
        let s = resolve(s)?;
        let p = match p {
            None => None,
            Some(n) => Some(self.id_of(&Term::NamedNode(n.clone()))?),
        };
        let o = resolve(o)?;
        Some([s, p, o])
    }
}

fn subject_term(s: &oxrdf::NamedOrBlankNode) -> Term {
    match s {
        oxrdf::NamedOrBlankNode::NamedNode(n) => Term::NamedNode(n.clone()),
        oxrdf::NamedOrBlankNode::BlankNode(b) => Term::BlankNode(b.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_and_scan() {
        let ttl = "@prefix ex: <http://ex/> . ex:a ex:p ex:b, ex:c . ex:d ex:p ex:b .";
        let g = Graph::load_str(ttl, "turtle").unwrap();
        assert_eq!(g.len(), 3);

        let p = NamedNode::new("http://ex/p").unwrap();
        let b = Term::NamedNode(NamedNode::new("http://ex/b").unwrap());
        // ?s ex:p ex:b  -> ex:a and ex:d
        let pat = g.pattern(None, Some(&p), Some(&b)).unwrap();
        let scan = g.store.scan(&pat);
        assert_eq!(scan.rows.len(), 2);
    }
}
