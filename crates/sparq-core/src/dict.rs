//! Term dictionary: bijection between RDF terms and dense `u32` ids.
//!
//! Dictionary encoding is the foundation of every fast triplestore (RDF-3X,
//! QLever, RDFox): triples are stored and joined as fixed-width integers, and
//! the (large, string-heavy) terms live once in the dictionary. M1 keeps the
//! dictionary fully in memory; later milestones add front-coding / on-disk
//! vocabularies and inline-encoded numeric ids (QLever's value ids).

use oxrdf::Term;
use rustc_hash::FxHashMap;

/// A dense term id. `u32` (≤ 4.29 B distinct terms) keeps index entries small
/// and cache-friendly; the id space is widened to `u64` only if a dataset needs
/// it. Id 0 is reserved as a sentinel ("no such term").
pub type Id = u32;

pub const NO_ID: Id = 0;

#[derive(Default)]
pub struct Dict {
    // id (1-based) -> term
    terms: Vec<Term>,
    // term lexical key -> id
    ids: FxHashMap<String, Id>,
}

impl Dict {
    pub fn new() -> Self {
        Dict {
            terms: Vec::new(),
            ids: FxHashMap::default(),
        }
    }

    pub fn with_capacity(n: usize) -> Self {
        Dict {
            terms: Vec::with_capacity(n),
            ids: FxHashMap::with_capacity_and_hasher(n, Default::default()),
        }
    }

    /// Canonical lexical key for a term (its N-Triples form), used as the
    /// dictionary key so equal terms map to one id.
    #[inline]
    fn key(term: &Term) -> String {
        term.to_string()
    }

    /// Interns a term, returning its id (creating it if new).
    #[inline]
    pub fn intern(&mut self, term: &Term) -> Id {
        let key = Self::key(term);
        if let Some(&id) = self.ids.get(&key) {
            return id;
        }
        let id = (self.terms.len() as Id) + 1; // 1-based
        self.terms.push(term.clone());
        self.ids.insert(key, id);
        id
    }

    /// Returns the id for a term if present, else `NO_ID`.
    #[inline]
    pub fn lookup(&self, term: &Term) -> Id {
        self.ids.get(&Self::key(term)).copied().unwrap_or(NO_ID)
    }

    /// Returns the term for an id (panics on an invalid id — ids come from the
    /// store, which only holds interned ids).
    #[inline]
    pub fn term(&self, id: Id) -> &Term {
        &self.terms[(id - 1) as usize]
    }

    pub fn len(&self) -> usize {
        self.terms.len()
    }

    pub fn is_empty(&self) -> bool {
        self.terms.is_empty()
    }

    /// A rough estimate of the dictionary's heap footprint in bytes (for
    /// benchmarking). Counts the `terms` vector, the lexical-key strings stored in
    /// the hash map (the dominant cost), and the map's bucket array.
    pub fn heap_bytes(&self) -> usize {
        let term_slots = self.terms.capacity() * std::mem::size_of::<Term>();
        // Each interned term keeps a String key (its N-Triples form) plus an id.
        let key_bytes: usize = self.ids.keys().map(|k| k.len() + std::mem::size_of::<String>()).sum();
        // FxHashMap bucket overhead (hashbrown): ~ (capacity) * (entry + control byte).
        let buckets = self.ids.capacity() * (std::mem::size_of::<(String, Id)>() + 1);
        term_slots + key_bytes + buckets
    }
}
