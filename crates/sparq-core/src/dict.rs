//! Term dictionary: bijection between RDF terms and dense `u32` ids.
//!
//! Dictionary encoding is the foundation of every fast triplestore (RDF-3X,
//! QLever, RDFox): triples are stored and joined as fixed-width integers, and
//! the (large, string-heavy) terms live once in the dictionary. M1 keeps the
//! dictionary fully in memory; later milestones add front-coding / on-disk
//! vocabularies and inline-encoded numeric ids (QLever's value ids).

use oxrdf::vocab::xsd;
use oxrdf::{Literal, Term};
use rustc_hash::FxHashMap;

/// A dense term id. `u32` (≤ 4.29 B distinct terms) keeps index entries small
/// and cache-friendly; the id space is widened to `u64` only if a dataset needs
/// it. Id 0 is reserved as a sentinel ("no such term").
pub type Id = u32;

pub const NO_ID: Id = 0;

/// Tagged-ValueId base: ids in `[INLINE_BASE, INLINE_BASE*2)` encode an
/// `xsd:integer` whose value is `id - INLINE_BASE` *inline* — no dictionary entry,
/// no string parse. Numeric FILTER / comparison / ORDER BY read the value straight
/// from the id (QLever's value-id idea, kept in `u32` so the index stays compact).
/// Inline integers also sort by value in the permutations (enabling range pruning).
/// Dictionary ids occupy `[1, INLINE_BASE)`; the engine's local-vocab ids start at
/// `INLINE_BASE*2` (= 1<<31), so the three ranges never overlap.
pub const INLINE_BASE: Id = 1 << 30;
const INLINE_MAX: u32 = INLINE_BASE - 1;

/// If `term` is a canonical non-negative `xsd:integer` in range, its inline id.
fn try_inline(term: &Term) -> Option<Id> {
    if let Term::Literal(l) = term {
        if l.datatype() == xsd::INTEGER {
            if let Ok(v) = l.value().parse::<u32>() {
                // Only the canonical lexical form (no leading zeros / sign) inlines,
                // so "030"^^integer stays a distinct dictionary term.
                if v <= INLINE_MAX && v.to_string() == l.value() {
                    return Some(INLINE_BASE + v);
                }
            }
        }
    }
    None
}

/// Whether an id encodes an inline integer value.
#[inline]
pub fn is_inline(id: Id) -> bool {
    (INLINE_BASE..INLINE_BASE << 1).contains(&id)
}

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

    /// Interns a term, returning its id (creating it if new). Canonical small
    /// `xsd:integer`s are encoded inline and never stored.
    #[inline]
    pub fn intern(&mut self, term: &Term) -> Id {
        if let Some(id) = try_inline(term) {
            return id;
        }
        let key = Self::key(term);
        if let Some(&id) = self.ids.get(&key) {
            return id;
        }
        let id = (self.terms.len() as Id) + 1; // 1-based
        debug_assert!(id < INLINE_BASE, "dictionary exceeded the inline-id base");
        self.terms.push(term.clone());
        self.ids.insert(key, id);
        id
    }

    /// Returns the id for a term if present, else `NO_ID`.
    #[inline]
    pub fn lookup(&self, term: &Term) -> Id {
        if let Some(id) = try_inline(term) {
            return id;
        }
        self.ids.get(&Self::key(term)).copied().unwrap_or(NO_ID)
    }

    /// Returns the term for an id. Inline-integer ids are decoded directly; others
    /// index the dictionary (panics on an invalid index — ids come from the store).
    #[inline]
    pub fn term(&self, id: Id) -> Term {
        if is_inline(id) {
            Term::Literal(Literal::new_typed_literal((id - INLINE_BASE).to_string(), xsd::INTEGER))
        } else {
            self.terms[(id - 1) as usize].clone()
        }
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
