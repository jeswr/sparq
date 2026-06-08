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
        // Enforced in release too: once the (non-inline) dictionary reaches
        // INLINE_BASE distinct terms, new ids would collide with the inline-integer
        // range and decode as integers — silent corruption. 2^30 ≈ 1.07B distinct
        // non-integer terms is a hard capacity limit of the u32 inline scheme; fail
        // loudly rather than corrupt (widen `Id` to u64 to lift it).
        assert!(id < INLINE_BASE, "dictionary exceeded the inline-id capacity (2^30 distinct non-integer terms); widen Id to u64");
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

#[cfg(test)]
mod tests {
    use super::*;

    fn int(s: &str) -> Term {
        Term::Literal(Literal::new_typed_literal(s, xsd::INTEGER))
    }

    #[test]
    fn inline_integer_roundtrip_and_boundaries() {
        let mut d = Dict::new();
        // Canonical non-negative integers inline; the id carries the value and never
        // touches the dictionary.
        for v in [0u32, 1, 42, INLINE_MAX] {
            let id = d.intern(&int(&v.to_string()));
            assert!(is_inline(id), "{v} should inline");
            assert_eq!(id, INLINE_BASE + v);
            assert_eq!(d.term(id), int(&v.to_string()), "round-trips to the canonical term");
            assert_eq!(d.lookup(&int(&v.to_string())), id, "lookup agrees with intern");
        }
        assert_eq!(d.len(), 0, "no inline integer is stored in the dictionary");
    }

    #[test]
    fn non_canonical_and_out_of_range_do_not_inline() {
        let mut d = Dict::new();
        // Leading zero, explicit sign, and negative are NOT the canonical form, so
        // they stay DISTINCT dictionary terms (term identity preserved).
        for s in ["05", "+5", "-3", "007"] {
            let id = d.intern(&int(s));
            assert!(!is_inline(id), "{s:?} must not inline");
        }
        // "05" and "5" are different terms with different ids (one inline, one not).
        assert_ne!(d.intern(&int("05")), d.intern(&int("5")));
        // A non-integer datatype with an integer lexical form does not inline.
        let typed = Term::Literal(Literal::new_typed_literal("5", xsd::INT));
        assert!(!is_inline(d.intern(&typed)));
        // INLINE_MAX inlines but INLINE_MAX + 1 (= INLINE_BASE) is out of range.
        assert!(is_inline(d.intern(&int(&INLINE_MAX.to_string()))));
        assert!(!is_inline(d.intern(&int(&(INLINE_BASE).to_string()))));
    }

    #[test]
    fn non_integer_terms_get_dictionary_ids_below_inline_base() {
        let mut d = Dict::new();
        let iri = Term::NamedNode(oxrdf::NamedNode::new("http://ex/x").unwrap());
        let id = d.intern(&iri);
        assert!(id >= 1 && id < INLINE_BASE);
        assert!(!is_inline(id));
        assert_eq!(d.term(id), iri);
    }
}
