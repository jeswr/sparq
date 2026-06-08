//! Term dictionary: bijection between RDF terms and dense `u32` ids.
//!
//! Dictionary encoding is the foundation of every fast triplestore (RDF-3X,
//! QLever, RDFox): triples are stored and joined as fixed-width integers, and the
//! (large, string-heavy) terms live once in the dictionary. Terms are stored COMPACT
//! and prefix-factored (IRIs share a namespace table); the interner is single-storage
//! (the hash table holds only ids) and content-addressed by a hash reproducible from
//! either an `oxrdf::Term` or the parsed byte components — so a byte-level parser can
//! intern straight from slices with no intermediate `Term`.

use hashbrown::HashTable;
use oxrdf::vocab::xsd;
use oxrdf::{Literal, NamedNode, Term};
use rustc_hash::FxHashMap;
use std::hash::Hasher;

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

/// If a literal `value`/`datatype` is a canonical non-negative `xsd:integer` in
/// range, its inline id. Only the canonical lexical form (no leading zeros / sign)
/// inlines, so `"030"^^integer` stays a distinct dictionary term.
#[inline]
fn try_inline_lit(value: &str, datatype: &str) -> Option<Id> {
    if datatype == xsd::INTEGER.as_str() {
        if let Ok(v) = value.parse::<u32>() {
            if v <= INLINE_MAX && v.to_string() == value {
                return Some(INLINE_BASE + v);
            }
        }
    }
    None
}

/// If `term` is a canonical non-negative `xsd:integer` in range, its inline id.
fn try_inline(term: &Term) -> Option<Id> {
    match term {
        Term::Literal(l) => try_inline_lit(l.value(), l.datatype().as_str()),
        _ => None,
    }
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

/// A borrowed view of a dictionary term's string components (no allocation), for
/// serialising results directly from ids. An IRI is its namespace prefix + local
/// suffix; a literal its lexical value, datatype IRI and optional language.
pub enum TermParts<'a> {
    Iri { prefix: &'a str, suffix: &'a str },
    Lit { value: &'a str, datatype: &'a str, lang: Option<&'a str> },
    Blank(&'a str),
}

// ---- Content hashing ---------------------------------------------------------
// A term's hash must be identical whether computed from an `oxrdf::Term`, from parsed
// byte slices, or from the compact `Stored` arena form — so interning and rehashing
// never build a `Term` or concatenate an IRI. FxHasher is deterministic (no seed).

#[inline]
fn hash_iri_parts(prefix: &str, suffix: &str) -> u64 {
    // Length-prefix the prefix and make hash_iri route through here on the SAME split,
    // so the two paths issue an identical sequence of writes — FxHasher is word-chunked,
    // so `write(a); write(b)` does NOT equal `write(a++b)`; identical call sequences do.
    let mut h = rustc_hash::FxHasher::default();
    h.write_u8(0);
    h.write_usize(prefix.len());
    h.write(prefix.as_bytes());
    h.write(suffix.as_bytes());
    h.finish()
}

#[inline]
fn hash_iri(iri: &str) -> u64 {
    let (prefix, suffix) = split_iri(iri);
    hash_iri_parts(prefix, suffix)
}

#[inline]
fn hash_lit(value: &str, datatype: &str, lang: Option<&str>) -> u64 {
    let mut h = rustc_hash::FxHasher::default();
    h.write_u8(1);
    h.write_usize(value.len());
    h.write(value.as_bytes());
    h.write_usize(datatype.len());
    h.write(datatype.as_bytes());
    match lang {
        Some(l) => {
            h.write_u8(1);
            h.write(l.as_bytes());
        }
        None => h.write_u8(0),
    }
    h.finish()
}

#[inline]
fn hash_blank(label: &str) -> u64 {
    let mut h = rustc_hash::FxHasher::default();
    h.write_u8(2);
    h.write(label.as_bytes());
    h.finish()
}

#[inline]
fn hash_term(t: &Term) -> u64 {
    match t {
        Term::NamedNode(n) => hash_iri(n.as_str()),
        Term::Literal(l) => hash_lit(l.value(), l.datatype().as_str(), l.language()),
        Term::BlankNode(b) => hash_blank(b.as_str()),
        _ => 0,
    }
}

#[inline]
fn hash_stored(s: &Stored, prefixes: &[Box<str>], datatypes: &[NamedNode]) -> u64 {
    match s {
        Stored::Iri { prefix, suffix } => hash_iri_parts(&prefixes[*prefix as usize], suffix),
        Stored::Lit { value, datatype, lang } => hash_lit(value, datatypes[*datatype as usize].as_str(), lang.as_deref()),
        Stored::Blank(b) => hash_blank(b),
    }
}

/// Splits an IRI into (namespace prefix, local suffix) at the last `#` or `/`, the
/// boundary that captures the shared namespace. IRIs with neither get an empty prefix.
#[inline]
fn split_iri(iri: &str) -> (&str, &str) {
    let cut = iri.rfind(['#', '/']).map_or(0, |i| i + 1);
    iri.split_at(cut)
}

// ---- Stored vs query-component equality (no reconstruction on the hot path) ---

#[inline]
fn stored_is_iri(s: &Stored, iri: &str, prefixes: &[Box<str>]) -> bool {
    match s {
        Stored::Iri { prefix, suffix } => {
            let p = &prefixes[*prefix as usize];
            iri.len() == p.len() + suffix.len() && iri.as_bytes().starts_with(p.as_bytes()) && iri[p.len()..] == **suffix
        }
        _ => false,
    }
}

#[inline]
fn stored_is_iri_parts(s: &Stored, prefix: &str, suffix: &str, prefixes: &[Box<str>]) -> bool {
    match s {
        Stored::Iri { prefix: pid, suffix: suf } => prefixes[*pid as usize].as_ref() == prefix && **suf == *suffix,
        _ => false,
    }
}

#[inline]
fn stored_is_lit(s: &Stored, value: &str, datatype: &str, lang: Option<&str>, datatypes: &[NamedNode]) -> bool {
    match s {
        Stored::Lit { value: v, datatype: dt, lang: lg } => {
            **v == *value && lg.as_deref() == lang && datatypes[*dt as usize].as_str() == datatype
        }
        _ => false,
    }
}

#[inline]
fn stored_eq_term(s: &Stored, q: &Term, prefixes: &[Box<str>], datatypes: &[NamedNode]) -> bool {
    match q {
        Term::NamedNode(n) => stored_is_iri(s, n.as_str(), prefixes),
        Term::Literal(l) => stored_is_lit(s, l.value(), l.datatype().as_str(), l.language(), datatypes),
        Term::BlankNode(b) => matches!(s, Stored::Blank(x) if **x == *b.as_str()),
        _ => false,
    }
}

/// Rebuilds a full `Term` from its compact storage (for `term()`).
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

    #[inline]
    fn intern_datatype(&mut self, dt: &str) -> u32 {
        if let Some(&id) = self.datatype_ids.get(dt) {
            return id;
        }
        let id = self.datatypes.len() as u32;
        self.datatypes.push(NamedNode::new_unchecked(dt));
        self.datatype_ids.insert(dt.into(), id);
        id
    }

    /// Assigns the next id to a freshly built `Stored` and indexes it.
    #[inline]
    fn push(&mut self, hash: u64, stored: Stored) -> Id {
        let id = (self.terms.len() as Id) + 1; // 1-based
        // Enforced in release too: once the (non-inline) dictionary reaches INLINE_BASE
        // distinct terms, new ids would collide with the inline-integer range and decode
        // as integers — silent corruption. 2^30 ≈ 1.07B distinct non-integer terms is a
        // hard capacity limit of the u32 inline scheme; fail loudly (widen Id to u64).
        assert!(id < INLINE_BASE, "dictionary exceeded the inline-id capacity (2^30 distinct non-integer terms); widen Id to u64");
        self.terms.push(stored);
        let (terms, prefixes, datatypes) = (&self.terms, &self.prefixes, &self.datatypes);
        self.table.insert_unique(hash, id, |&i| hash_stored(&terms[(i - 1) as usize], prefixes, datatypes));
        id
    }

    /// Interns an IRI term from its string, returning its id.
    #[inline]
    pub fn intern_iri(&mut self, iri: &str) -> Id {
        let hash = hash_iri(iri);
        if let Some(&id) = self.table.find(hash, |&id| stored_is_iri(&self.terms[(id - 1) as usize], iri, &self.prefixes)) {
            return id;
        }
        let (p, suffix) = split_iri(iri);
        let prefix = self.intern_prefix(p);
        self.push(hash, Stored::Iri { prefix, suffix: suffix.into() })
    }

    /// Interns an IRI already split into (prefix, suffix) — used by the parallel-load
    /// merge, where the canonical split is already known (no re-split, no concat).
    #[inline]
    fn intern_iri_parts(&mut self, prefix: &str, suffix: &str) -> Id {
        let hash = hash_iri_parts(prefix, suffix);
        if let Some(&id) =
            self.table.find(hash, |&id| stored_is_iri_parts(&self.terms[(id - 1) as usize], prefix, suffix, &self.prefixes))
        {
            return id;
        }
        let prefix = self.intern_prefix(prefix);
        self.push(hash, Stored::Iri { prefix, suffix: suffix.into() })
    }

    /// Interns a literal from its components, returning its id. Canonical small
    /// `xsd:integer`s are encoded inline and never stored.
    #[inline]
    pub fn intern_lit(&mut self, value: &str, datatype: &str, lang: Option<&str>) -> Id {
        if let Some(id) = try_inline_lit(value, datatype) {
            return id;
        }
        let hash = hash_lit(value, datatype, lang);
        if let Some(&id) =
            self.table.find(hash, |&id| stored_is_lit(&self.terms[(id - 1) as usize], value, datatype, lang, &self.datatypes))
        {
            return id;
        }
        let datatype = self.intern_datatype(datatype);
        self.push(hash, Stored::Lit { value: value.into(), datatype, lang: lang.map(Into::into) })
    }

    /// Interns a blank node from its label, returning its id.
    #[inline]
    pub fn intern_blank(&mut self, label: &str) -> Id {
        let hash = hash_blank(label);
        if let Some(&id) = self.table.find(hash, |&id| matches!(&self.terms[(id - 1) as usize], Stored::Blank(b) if **b == *label)) {
            return id;
        }
        self.push(hash, Stored::Blank(label.into()))
    }

    /// Interns a term, returning its id (creating it if new). Dispatches to the
    /// component interners so the `Term` and byte-slice paths share one code path.
    #[inline]
    pub fn intern(&mut self, term: &Term) -> Id {
        match term {
            Term::NamedNode(n) => self.intern_iri(n.as_str()),
            Term::Literal(l) => self.intern_lit(l.value(), l.datatype().as_str(), l.language()),
            Term::BlankNode(b) => self.intern_blank(b.as_str()),
            other => unreachable!("non-triple term in dictionary: {other:?}"),
        }
    }

    /// Returns the id for a term if present, else `NO_ID`.
    #[inline]
    pub fn lookup(&self, term: &Term) -> Id {
        if let Some(id) = try_inline(term) {
            return id;
        }
        let hash = hash_term(term);
        self.table
            .find(hash, |&id| stored_eq_term(&self.terms[(id - 1) as usize], term, &self.prefixes, &self.datatypes))
            .copied()
            .unwrap_or(NO_ID)
    }

    /// Borrows a dictionary term's components WITHOUT reconstructing an `oxrdf::Term`
    /// (no allocation) — for serialising results straight from ids. Only valid for a
    /// real dictionary id (`1..INLINE_BASE`); the caller handles inline / local ids.
    #[inline]
    pub fn term_parts(&self, id: Id) -> TermParts<'_> {
        match &self.terms[(id - 1) as usize] {
            Stored::Iri { prefix, suffix } => TermParts::Iri { prefix: &self.prefixes[*prefix as usize], suffix },
            Stored::Lit { value, datatype, lang } => {
                TermParts::Lit { value, datatype: self.datatypes[*datatype as usize].as_str(), lang: lang.as_deref() }
            }
            Stored::Blank(b) => TermParts::Blank(b),
        }
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

    /// Merges another (partial) dictionary into this one, returning the remap from the
    /// other's local ids to this dictionary's global ids: `remap[local - 1]` is the
    /// global id for the other's 1-based local id `local`. Used by the parallel bulk
    /// loader. Interns each of the other's terms directly from its compact components —
    /// no `Term` reconstruction, no IRI concatenation.
    pub fn merge_remap(&mut self, other: &Dict) -> Vec<Id> {
        self.terms.reserve(other.terms.len());
        {
            let (terms, prefixes, datatypes) = (&self.terms, &self.prefixes, &self.datatypes);
            self.table
                .reserve(other.terms.len(), |&i| hash_stored(&terms[(i - 1) as usize], prefixes, datatypes));
        }
        other
            .terms
            .iter()
            .map(|s| match s {
                Stored::Iri { prefix, suffix } => self.intern_iri_parts(&other.prefixes[*prefix as usize], suffix),
                Stored::Lit { value, datatype, lang } => self.intern_lit(value, other.datatypes[*datatype as usize].as_str(), lang.as_deref()),
                Stored::Blank(b) => self.intern_blank(b),
            })
            .collect()
    }

    /// Serialises the dictionary (prefixes, datatypes, compact terms) to `path` in a
    /// compact binary format. The hash table is NOT written — it is rebuilt on `open`.
    pub fn save(&self, path: &std::path::Path) -> std::io::Result<()> {
        use std::io::Write;
        let mut w = std::io::BufWriter::new(std::fs::File::create(path)?);
        w.write_all(&(self.prefixes.len() as u32).to_le_bytes())?;
        for p in &self.prefixes {
            write_str(&mut w, p)?;
        }
        w.write_all(&(self.datatypes.len() as u32).to_le_bytes())?;
        for d in &self.datatypes {
            write_str(&mut w, d.as_str())?;
        }
        w.write_all(&(self.terms.len() as u32).to_le_bytes())?;
        for t in &self.terms {
            match t {
                Stored::Iri { prefix, suffix } => {
                    w.write_all(&[0])?;
                    w.write_all(&prefix.to_le_bytes())?;
                    write_str(&mut w, suffix)?;
                }
                Stored::Lit { value, datatype, lang } => {
                    w.write_all(&[1])?;
                    write_str(&mut w, value)?;
                    w.write_all(&datatype.to_le_bytes())?;
                    match lang {
                        Some(l) => {
                            w.write_all(&[1])?;
                            write_str(&mut w, l)?;
                        }
                        None => w.write_all(&[0])?,
                    }
                }
                Stored::Blank(b) => {
                    w.write_all(&[2])?;
                    write_str(&mut w, b)?;
                }
            }
        }
        w.flush()
    }

    /// Loads a dictionary written by [`save`](Self::save), rebuilding the hash table.
    pub fn open(path: &std::path::Path) -> std::io::Result<Dict> {
        let mut r = std::io::BufReader::new(std::fs::File::open(path)?);
        let np = read_u32(&mut r)? as usize;
        let mut prefixes = Vec::with_capacity(np);
        let mut prefix_ids = FxHashMap::default();
        for i in 0..np {
            let p = read_str(&mut r)?;
            prefix_ids.insert(p.clone(), i as u32);
            prefixes.push(p);
        }
        let nd = read_u32(&mut r)? as usize;
        let mut datatypes = Vec::with_capacity(nd);
        let mut datatype_ids = FxHashMap::default();
        for i in 0..nd {
            let d = read_str(&mut r)?;
            datatype_ids.insert(d.clone(), i as u32);
            datatypes.push(NamedNode::new_unchecked(String::from(d)));
        }
        let nt = read_u32(&mut r)? as usize;
        let mut terms = Vec::with_capacity(nt);
        for _ in 0..nt {
            terms.push(match read_u8(&mut r)? {
                0 => Stored::Iri { prefix: read_u32(&mut r)?, suffix: read_str(&mut r)? },
                1 => {
                    let value = read_str(&mut r)?;
                    let datatype = read_u32(&mut r)?;
                    let lang = if read_u8(&mut r)? == 1 { Some(read_str(&mut r)?) } else { None };
                    Stored::Lit { value, datatype, lang }
                }
                2 => Stored::Blank(read_str(&mut r)?),
                other => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("bad term tag {other}"))),
            });
        }
        // Rebuild the hash table from the arena.
        let mut table = HashTable::with_capacity(nt);
        for (i, t) in terms.iter().enumerate() {
            let id = (i as Id) + 1;
            let hash = hash_stored(t, &prefixes, &datatypes);
            table.insert_unique(hash, id, |&j| hash_stored(&terms[(j - 1) as usize], &prefixes, &datatypes));
        }
        Ok(Dict { prefixes, prefix_ids, datatypes, datatype_ids, terms, table })
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

// ---- (de)serialisation helpers for save/open --------------------------------

fn write_str(w: &mut impl std::io::Write, s: &str) -> std::io::Result<()> {
    w.write_all(&(s.len() as u32).to_le_bytes())?;
    w.write_all(s.as_bytes())
}

fn read_u32(r: &mut impl std::io::Read) -> std::io::Result<u32> {
    let mut b = [0u8; 4];
    r.read_exact(&mut b)?;
    Ok(u32::from_le_bytes(b))
}

fn read_u8(r: &mut impl std::io::Read) -> std::io::Result<u8> {
    let mut b = [0u8; 1];
    r.read_exact(&mut b)?;
    Ok(b[0])
}

fn read_str(r: &mut impl std::io::Read) -> std::io::Result<Box<str>> {
    let n = read_u32(r)? as usize;
    let mut buf = vec![0u8; n];
    r.read_exact(&mut buf)?;
    String::from_utf8(buf)
        .map(String::into_boxed_str)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid utf-8 in dictionary"))
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
        for s in ["05", "+5", "-3", "007"] {
            let id = d.intern(&int(s));
            assert!(!is_inline(id), "{s:?} must not inline");
        }
        assert_ne!(d.intern(&int("05")), d.intern(&int("5")));
        let typed = Term::Literal(Literal::new_typed_literal("5", xsd::INT));
        assert!(!is_inline(d.intern(&typed)));
        assert!(is_inline(d.intern(&int(&INLINE_MAX.to_string()))));
        assert!(!is_inline(d.intern(&int(&(INLINE_BASE).to_string()))));
    }

    #[test]
    fn iri_prefix_factoring_roundtrips() {
        // IRIs sharing a namespace dedupe the prefix; every term round-trips exactly,
        // and lookup agrees with intern (the content hash matches across paths).
        let mut d = Dict::new();
        let iris = ["http://ex/a", "http://ex/b", "http://www.w3.org/2001/XMLSchema#date", "urn:x"];
        let ids: Vec<Id> = iris.iter().map(|i| d.intern(&Term::NamedNode(NamedNode::new_unchecked(*i)))).collect();
        for (iri, id) in iris.iter().zip(&ids) {
            let t = Term::NamedNode(NamedNode::new_unchecked(*iri));
            assert_eq!(d.term(*id), t);
            assert_eq!(d.lookup(&t), *id);
        }
        // Distinct IRIs get distinct ids; re-interning is idempotent.
        assert_eq!(ids.iter().collect::<std::collections::HashSet<_>>().len(), iris.len());
        assert_eq!(d.intern(&Term::NamedNode(NamedNode::new_unchecked("http://ex/a"))), ids[0]);
        // A language-tagged literal and a typed literal round-trip with their components.
        let lang = Term::Literal(Literal::new_language_tagged_literal_unchecked("hi", "en"));
        let dec = Term::Literal(Literal::new_typed_literal("1.0", xsd::DECIMAL));
        let (lid, did) = (d.intern(&lang), d.intern(&dec));
        assert_eq!(d.term(lid), lang);
        assert_eq!(d.term(did), dec);
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
