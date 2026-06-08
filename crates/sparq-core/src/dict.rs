//! Term dictionary: bijection between RDF terms and dense `u32` ids.
//!
//! Dictionary encoding is the foundation of every fast triplestore (RDF-3X,
//! QLever, RDFox): triples are stored and joined as fixed-width integers, and
//! the (large, string-heavy) terms live once in the dictionary. M1 keeps the
//! dictionary fully in memory; later milestones add front-coding / on-disk
//! vocabularies and inline-encoded numeric ids (QLever's value ids).

use hashbrown::HashTable;
use oxrdf::vocab::xsd;
use oxrdf::{Literal, NamedNode, Term};
use rustc_hash::FxHashMap;
use std::hash::{Hash, Hasher};

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

/// A compact, single-storage term interner. Each (non-inline) term is stored once, in
/// the `terms` arena, and the hash table holds only its `Id`. IRIs are split into a
/// shared namespace prefix (deduplicated in `prefixes`, the redundancy SPARQL PREFIX
/// directives target) plus a per-term suffix, and literal datatypes are deduplicated in
/// `datatypes` — so a long repeated `http://…/` is stored once, not per term, and the
/// per-term slot (`Stored`) is far smaller than a full `oxrdf::Term`.
#[derive(Default)]
pub struct Dict {
    prefixes: Vec<Box<str>>,                  // id -> IRI namespace prefix
    prefix_ids: FxHashMap<Box<str>, u32>,     // prefix -> id
    datatypes: Vec<NamedNode>,                // id -> literal datatype (a small set)
    datatype_ids: FxHashMap<Box<str>, u32>,   // datatype IRI -> id
    terms: Vec<Stored>,                       // id-1 -> compact term
    table: HashTable<Id>,                     // hash(term) -> id (bare ids, compared via the arena)
}

/// The compact per-term storage. An IRI is `(prefix id, suffix)`; a literal is
/// `(value, datatype id, optional language)`; a blank node is its label.
enum Stored {
    Iri { prefix: u32, suffix: Box<str> },
    Lit { value: Box<str>, datatype: u32, lang: Option<Box<str>> },
    Blank(Box<str>),
}

/// A deterministic hash of a term (FxHasher has no random seed, so the same term
/// always hashes the same — required for the rehash-from-arena table).
#[inline]
fn hash_term(t: &Term) -> u64 {
    let mut h = rustc_hash::FxHasher::default();
    t.hash(&mut h);
    h.finish()
}

/// Splits an IRI into (namespace prefix, local suffix) at the last `#` or `/`, the
/// boundary that captures the shared namespace. IRIs with neither get an empty prefix.
#[inline]
fn split_iri(iri: &str) -> (&str, &str) {
    let cut = iri.rfind(['#', '/']).map_or(0, |i| i + 1);
    iri.split_at(cut)
}

/// Whether a stored term equals a query term, WITHOUT reconstructing it (the hot-path
/// comparison): compare the IRI prefix+suffix / literal components directly.
fn stored_eq(s: &Stored, q: &Term, prefixes: &[Box<str>], datatypes: &[NamedNode]) -> bool {
    match (s, q) {
        (Stored::Iri { prefix, suffix }, Term::NamedNode(n)) => {
            let p = &prefixes[*prefix as usize];
            let iri = n.as_str();
            iri.len() == p.len() + suffix.len()
                && iri.as_bytes().starts_with(p.as_bytes())
                && iri[p.len()..] == **suffix
        }
        (Stored::Lit { value, datatype, lang }, Term::Literal(l)) => {
            **value == *l.value()
                && lang.as_deref() == l.language()
                && datatypes[*datatype as usize].as_ref() == l.datatype()
        }
        (Stored::Blank(b), Term::BlankNode(bn)) => **b == *bn.as_str(),
        _ => false,
    }
}

/// Rebuilds a full `Term` from its compact storage (for `term()` and table rehashing).
fn reconstruct(s: &Stored, prefixes: &[Box<str>], datatypes: &[NamedNode]) -> Term {
    match s {
        Stored::Iri { prefix, suffix } => {
            let p = &prefixes[*prefix as usize];
            let mut iri = String::with_capacity(p.len() + suffix.len());
            iri.push_str(p);
            iri.push_str(suffix);
            Term::NamedNode(NamedNode::new_unchecked(iri))
        }
        Stored::Lit { value, datatype, lang } => Term::Literal(match lang {
            Some(l) => Literal::new_language_tagged_literal_unchecked(value.to_string(), l.to_string()),
            None => Literal::new_typed_literal(value.to_string(), datatypes[*datatype as usize].clone()),
        }),
        Stored::Blank(b) => Term::BlankNode(oxrdf::BlankNode::new_unchecked(b.to_string())),
    }
}

impl Dict {
    pub fn new() -> Self {
        Dict::default()
    }

    pub fn with_capacity(n: usize) -> Self {
        Dict {
            terms: Vec::with_capacity(n),
            table: HashTable::with_capacity(n),
            ..Default::default()
        }
    }

    /// Interns an IRI namespace prefix, returning its (small) id.
    #[inline]
    fn intern_prefix(&mut self, prefix: &str) -> u32 {
        if let Some(&id) = self.prefix_ids.get(prefix) {
            return id;
        }
        let id = self.prefixes.len() as u32;
        let boxed: Box<str> = prefix.into();
        self.prefixes.push(boxed.clone());
        self.prefix_ids.insert(boxed, id);
        id
    }

    /// Interns a literal datatype, returning its (small) id.
    #[inline]
    fn intern_datatype(&mut self, dt: oxrdf::NamedNodeRef<'_>) -> u32 {
        if let Some(&id) = self.datatype_ids.get(dt.as_str()) {
            return id;
        }
        let id = self.datatypes.len() as u32;
        self.datatypes.push(dt.into_owned());
        self.datatype_ids.insert(dt.as_str().into(), id);
        id
    }

    /// Builds the compact storage for a term, interning its prefix / datatype.
    #[inline]
    fn store_term(&mut self, term: &Term) -> Stored {
        match term {
            Term::NamedNode(n) => {
                let (p, suffix) = split_iri(n.as_str());
                let prefix = self.intern_prefix(p);
                Stored::Iri { prefix, suffix: suffix.into() }
            }
            Term::Literal(l) => {
                let datatype = self.intern_datatype(l.datatype());
                Stored::Lit { value: l.value().into(), datatype, lang: l.language().map(Into::into) }
            }
            Term::BlankNode(b) => Stored::Blank(b.as_str().into()),
            other => unreachable!("non-triple term in dictionary: {other:?}"),
        }
    }

    /// Interns a term, returning its id (creating it if new). Canonical small
    /// `xsd:integer`s are encoded inline and never stored.
    #[inline]
    pub fn intern(&mut self, term: &Term) -> Id {
        if let Some(id) = try_inline(term) {
            return id;
        }
        let hash = hash_term(term);
        if let Some(&id) =
            self.table.find(hash, |&id| stored_eq(&self.terms[(id - 1) as usize], term, &self.prefixes, &self.datatypes))
        {
            return id;
        }
        let id = (self.terms.len() as Id) + 1; // 1-based
        // Enforced in release too: once the (non-inline) dictionary reaches
        // INLINE_BASE distinct terms, new ids would collide with the inline-integer
        // range and decode as integers — silent corruption. 2^30 ≈ 1.07B distinct
        // non-integer terms is a hard capacity limit of the u32 inline scheme; fail
        // loudly rather than corrupt (widen `Id` to u64 to lift it).
        assert!(id < INLINE_BASE, "dictionary exceeded the inline-id capacity (2^30 distinct non-integer terms); widen Id to u64");
        let stored = self.store_term(term);
        self.terms.push(stored);
        let (terms, prefixes, datatypes) = (&self.terms, &self.prefixes, &self.datatypes);
        self.table
            .insert_unique(hash, id, |&i| hash_term(&reconstruct(&terms[(i - 1) as usize], prefixes, datatypes)));
        id
    }

    /// Returns the id for a term if present, else `NO_ID`.
    #[inline]
    pub fn lookup(&self, term: &Term) -> Id {
        if let Some(id) = try_inline(term) {
            return id;
        }
        let hash = hash_term(term);
        self.table
            .find(hash, |&id| stored_eq(&self.terms[(id - 1) as usize], term, &self.prefixes, &self.datatypes))
            .copied()
            .unwrap_or(NO_ID)
    }

    /// Returns the term for an id. Inline-integer ids are decoded directly; others are
    /// reconstructed from the compact arena (panics on an invalid index — ids come from
    /// the store).
    #[inline]
    pub fn term(&self, id: Id) -> Term {
        if is_inline(id) {
            Term::Literal(Literal::new_typed_literal((id - INLINE_BASE).to_string(), xsd::INTEGER))
        } else {
            reconstruct(&self.terms[(id - 1) as usize], &self.prefixes, &self.datatypes)
        }
    }

    /// Merges another (partial) dictionary into this one, returning the remap from
    /// the other's local ids to this dictionary's global ids: `remap[local - 1]` is
    /// the global id for the other's 1-based local id `local`. Used by the parallel
    /// bulk loader, where each thread builds a partial dictionary and the partials are
    /// merged into one. Inline-integer ids are NOT in any dictionary and pass through
    /// unchanged (the caller checks `is_inline`).
    pub fn merge_remap(&mut self, other: &Dict) -> Vec<Id> {
        self.terms.reserve(other.terms.len());
        {
            let (terms, prefixes, datatypes) = (&self.terms, &self.prefixes, &self.datatypes);
            self.table
                .reserve(other.terms.len(), |&i| hash_term(&reconstruct(&terms[(i - 1) as usize], prefixes, datatypes)));
        }
        // Reconstruct each of the other dictionary's terms and re-intern (which rebuilds
        // this dictionary's prefix / datatype tables), giving a local->global id remap.
        (0..other.terms.len())
            .map(|k| {
                let t = reconstruct(&other.terms[k], &other.prefixes, &other.datatypes);
                self.intern(&t)
            })
            .collect()
    }

    pub fn len(&self) -> usize {
        self.terms.len()
    }

    pub fn is_empty(&self) -> bool {
        self.terms.is_empty()
    }

    /// A rough estimate of the dictionary's heap footprint in bytes (for
    /// benchmarking). Counts the compact `terms` arena (slots + suffix/value/lang
    /// bytes), the shared prefix + datatype tables, and the hash table (bare ids).
    pub fn heap_bytes(&self) -> usize {
        let term_slots = self.terms.capacity() * std::mem::size_of::<Stored>();
        let owned: usize = self.terms.iter().map(stored_owned_bytes).sum();
        let prefix_bytes: usize =
            self.prefixes.iter().map(|p| p.len() + std::mem::size_of::<Box<str>>()).sum::<usize>() * 2; // Vec + map key
        let dt_bytes: usize = self.datatypes.iter().map(|d| d.as_str().len() + 32).sum();
        let table = self.table.capacity() * (std::mem::size_of::<Id>() + 1);
        term_slots + owned + prefix_bytes + dt_bytes + table
    }
}

/// The owned heap bytes of a compact stored term's string content (suffix / value /
/// language / blank label) — the part outside the fixed `Stored` slot. The IRI prefix
/// and datatype live once in the shared tables, not here.
fn stored_owned_bytes(s: &Stored) -> usize {
    match s {
        Stored::Iri { suffix, .. } => suffix.len(),
        Stored::Lit { value, lang, .. } => value.len() + lang.as_ref().map_or(0, |l| l.len()),
        Stored::Blank(b) => b.len(),
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
