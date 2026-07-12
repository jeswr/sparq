//! Prefix completion for RDF IRIs, local names, and common label predicates.
//! [GPT-5.6] sq-lsp7k.9.1.
//!
//! The index deliberately performs prefix matching only. Fuzzy matching is out of
//! scope: callers that need it should layer a separate matcher over the returned
//! candidates rather than making this compact, deterministic index more costly.

use std::collections::BTreeMap;

use oxrdf::NamedNode;
use rustc_hash::FxHashMap;
use sparq_core::{dict::Id, dict::TermParts, Graph};

const RDFS_LABEL: &str = "http://www.w3.org/2000/01/rdf-schema#label";
const SKOS_PREF_LABEL: &str = "http://www.w3.org/2004/02/skos/core#prefLabel";

/// The source of a completion candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CandidateKind {
    /// A complete IRI string.
    Iri,
    /// The IRI suffix after its final `#` or `/`.
    LocalName,
    /// A literal label, carrying the label predicate's dictionary id.
    Label(Id),
}

/// One prefix-completion result.
#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    /// The entity identified by this completion.
    pub id: Id,
    /// The case-folded key that matched the query.
    pub key: Box<str>,
    /// How the key was derived.
    pub kind: CandidateKind,
    /// An injected importance score, or zero when no score was supplied.
    pub score: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CompletionEntry {
    id: Id,
    kind: CandidateKind,
}

/// An owned prefix index over graph IRIs and `rdfs:label`/`skos:prefLabel` values.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct CompletionIndex {
    keys: BTreeMap<Box<str>, Vec<CompletionEntry>>,
    indexed_dict_len: usize,
}

impl CompletionIndex {
    /// Builds a completion index for `graph` without adding dependencies to a ranker.
    pub fn build(graph: &Graph) -> Self {
        let mut index = Self::default();
        for (id, parts) in graph.dict.iter() {
            if let TermParts::Iri { prefix, suffix } = parts {
                let iri = format!("{prefix}{suffix}");
                index.insert(id, &iri, CandidateKind::Iri);
                let local = iri.rsplit(['#', '/']).next().unwrap_or(&iri);
                if !local.is_empty() {
                    index.insert(id, local, CandidateKind::LocalName);
                }
            }
        }

        for predicate in [RDFS_LABEL, SKOS_PREF_LABEL] {
            let predicate = NamedNode::new_unchecked(predicate);
            let Some(pattern) = graph.pattern(None, Some(&predicate), None) else {
                continue;
            };
            let predicate_id = pattern[1].expect("predicate-bound pattern");
            let scan = graph.store.scan(&pattern);
            for row in scan.rows.iter() {
                let [subject, _, object] = scan.to_spo(row);
                if let TermParts::Lit { value, .. } = graph.dict.term_parts(object) {
                    index.insert(subject, value, CandidateKind::Label(predicate_id));
                }
            }
        }
        index.indexed_dict_len = graph.dict.len();
        index
    }

    fn insert(&mut self, id: Id, key: &str, kind: CandidateKind) {
        let key: Box<str> = key.to_lowercase().into_boxed_str();
        if !key.is_empty() {
            self.keys
                .entry(key)
                .or_default()
                .push(CompletionEntry { id, kind });
        }
    }

    /// Returns at most `k` case-insensitive prefix matches, ordered by descending
    /// injected score and then ascending key and entity id.
    pub fn complete(
        &self,
        prefix: &str,
        k: usize,
        scores: Option<&FxHashMap<Id, f64>>,
    ) -> Vec<Candidate> {
        if k == 0 {
            return Vec::new();
        }
        let prefix = prefix.to_lowercase();
        let mut candidates = Vec::new();
        for (key, entries) in self.keys.range(prefix.clone().into_boxed_str()..) {
            if !key.starts_with(&prefix) {
                break;
            }
            candidates.extend(entries.iter().map(|entry| {
                Candidate {
                    id: entry.id,
                    key: key.clone(),
                    kind: entry.kind,
                    score: scores
                        .and_then(|map| map.get(&entry.id))
                        .copied()
                        .unwrap_or(0.0),
                }
            }));
        }
        candidates.sort_by(|a, b| {
            b.score
                .total_cmp(&a.score)
                .then_with(|| a.key.cmp(&b.key))
                .then_with(|| a.id.cmp(&b.id))
                .then_with(|| a.kind.cmp(&b.kind))
        });
        candidates.truncate(k);
        candidates
    }

    /// Number of indexed key/entity/source entries.
    pub fn len(&self) -> usize {
        self.keys.values().map(Vec::len).sum()
    }

    /// Whether the index contains no completion entries.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Dictionary length recorded when this index was built.
    pub fn indexed_dict_len(&self) -> usize {
        self.indexed_dict_len
    }

    /// A rough estimate of the index's owned heap footprint in bytes.
    pub fn heap_bytes(&self) -> usize {
        self.keys
            .iter()
            .map(|(key, entries)| {
                key.len()
                    + std::mem::size_of::<Box<str>>()
                    + 24
                    + entries.capacity() * std::mem::size_of::<CompletionEntry>()
            })
            .sum()
    }
}
