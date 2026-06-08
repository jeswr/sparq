//! sparq-core: dictionary-encoded RDF storage with six permutation indexes.
//!
//! This is the storage substrate for the query engine: a [`Graph`] holds the
//! term [`Dict`]ionary and the [`TripleStore`] (six sorted permutations), and
//! is built from an RDF document via the bulk loader.

pub mod dict;
pub mod store;

use dict::{Dict, Id};
use oxrdf::vocab::xsd;
use oxrdf::{Literal, NamedNode, Term};
use oxttl::{NQuadsParser, NTriplesParser, TriGParser, TurtleParser};
use store::{Pattern, TripleStore};

/// An immutable, dictionary-encoded RDF graph ready for querying.
pub struct Graph {
    pub dict: Dict,
    pub store: TripleStore,
    /// Parallel to the dictionary: the f64 value of each numeric literal (NaN for
    /// non-numeric terms). Lets the engine evaluate numeric filters / comparisons
    /// / ORDER BY without materialising the term and parsing its string each time
    /// — a lightweight, u32-id-preserving stand-in for QLever's inline ValueIds.
    numerics: Vec<f64>,
}

/// The f64 value of a term if it is a numeric XSD literal, else NaN.
fn numeric_of(term: &Term) -> f64 {
    match term {
        Term::Literal(l) if is_numeric_dt(l) => l.value().parse::<f64>().unwrap_or(f64::NAN),
        _ => f64::NAN,
    }
}

fn is_numeric_dt(l: &Literal) -> bool {
    let dt = l.datatype().as_str();
    dt == xsd::INTEGER.as_str()
        || dt == xsd::DECIMAL.as_str()
        || dt == xsd::DOUBLE.as_str()
        || dt == xsd::FLOAT.as_str()
        || dt == xsd::LONG.as_str()
        || dt == xsd::INT.as_str()
        || dt == xsd::SHORT.as_str()
        || dt == xsd::BYTE.as_str()
        || dt == xsd::NON_NEGATIVE_INTEGER.as_str()
        || dt == xsd::POSITIVE_INTEGER.as_str()
        || dt == xsd::NON_POSITIVE_INTEGER.as_str()
        || dt == xsd::NEGATIVE_INTEGER.as_str()
        || dt == xsd::UNSIGNED_INT.as_str()
        || dt == xsd::UNSIGNED_LONG.as_str()
        || dt == xsd::UNSIGNED_SHORT.as_str()
        || dt == xsd::UNSIGNED_BYTE.as_str()
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

        Ok(Self::build(dict, triples))
    }

    /// Streaming loader: parses an RDF document incrementally from a reader (so a
    /// gzip / bzip2 decompression stream can be ingested without holding the whole
    /// document in memory). Same formats as [`load_str`](Self::load_str). The
    /// dictionary and triple buffer still grow in memory, so the full store only
    /// fits for datasets that fit in RAM; the streaming ingest *throughput*,
    /// however, is measurable on arbitrarily large inputs (see `sparq-cli ingest`).
    pub fn load_reader<R: std::io::Read>(reader: R, format: &str) -> Result<Graph, String> {
        let mut dict = Dict::new();
        let mut triples: Vec<[Id; 3]> = Vec::new();

        macro_rules! push_triple {
            ($s:expr, $p:expr, $o:expr) => {{
                let s = dict.intern(&subject_term($s));
                let p = dict.intern(&Term::NamedNode($p.clone()));
                let o = dict.intern($o);
                triples.push([s, p, o]);
            }};
        }

        match format {
            "nquads" | "n-quads" => {
                for q in NQuadsParser::new().for_reader(reader) {
                    let q = q.map_err(|e| e.to_string())?;
                    push_triple!(&q.subject, &q.predicate, &q.object);
                }
            }
            "trig" | "application/trig" => {
                for q in TriGParser::new().for_reader(reader) {
                    let q = q.map_err(|e| e.to_string())?;
                    push_triple!(&q.subject, &q.predicate, &q.object);
                }
            }
            "turtle" | "ttl" => {
                for t in TurtleParser::new().for_reader(reader) {
                    let t = t.map_err(|e| e.to_string())?;
                    push_triple!(&t.subject, &t.predicate, &t.object);
                }
            }
            _ => {
                for t in NTriplesParser::new().for_reader(reader) {
                    let t = t.map_err(|e| e.to_string())?;
                    push_triple!(&t.subject, &t.predicate, &t.object);
                }
            }
        }

        Ok(Self::build(dict, triples))
    }

    /// Builds the store + numeric cache from interned triples (shared by the
    /// string and streaming loaders).
    fn build(dict: Dict, triples: Vec<[Id; 3]>) -> Graph {
        let store = TripleStore::from_triples(triples);

        // Precompute the numeric value of every dictionary term (one parse each).
        let n = dict.len();
        #[cfg(feature = "parallel")]
        let numerics: Vec<f64> = {
            use rayon::prelude::*;
            (0..n).into_par_iter().map(|i| numeric_of(&dict.term(i as Id + 1))).collect()
        };
        #[cfg(not(feature = "parallel"))]
        let numerics: Vec<f64> = (0..n).map(|i| numeric_of(&dict.term(i as Id + 1))).collect();

        Graph { dict, store, numerics }
    }

    /// The numeric value of a term id, or `None` if it is not a numeric literal.
    /// O(1), no allocation — the engine's fast path for numeric filters. An inline
    /// integer id carries its value directly (no lookup); other ids use the cache.
    #[inline]
    pub fn numeric_value(&self, id: Id) -> Option<f64> {
        if dict::is_inline(id) {
            return Some((id - dict::INLINE_BASE) as f64);
        }
        let v = *self.numerics.get((id - 1) as usize)?;
        if v.is_nan() {
            None
        } else {
            Some(v)
        }
    }

    pub fn len(&self) -> usize {
        self.store.len()
    }

    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }

    /// A rough estimate of the graph's in-memory footprint in bytes (dictionary +
    /// the six permutation indexes), for benchmarking.
    pub fn heap_bytes(&self) -> usize {
        self.dict.heap_bytes() + self.store.heap_bytes() + self.numerics.capacity() * std::mem::size_of::<f64>()
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
