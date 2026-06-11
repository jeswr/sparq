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

/// Tagged-ValueId base: ids in `[INLINE_BASE, INLINE_BASE + 2^30)` encode an
/// `xsd:integer` whose value is `id - INLINE_BASE` *inline* — no dictionary entry,
/// no string parse. Numeric FILTER / comparison / ORDER BY read the value straight
/// from the id (QLever's value-id idea, kept in `u32` so the index stays compact).
/// Inline integers also sort by value in the permutations (enabling range pruning).
///
/// The `u32` id space is partitioned: dictionary ids `[1, INLINE_BASE)` (≈2.1 billion
/// distinct terms — enough for e.g. full-Wikidata's term count without widening to `u64`,
/// which would double the index), inline integers `[INLINE_BASE, INLINE_BASE + 2^30)`, and
/// the engine's local-vocab ids `[INLINE_BASE + 2^30, 2^32)`. `0` is `NO_ID`.
pub const INLINE_BASE: Id = 1 << 31;
/// The largest value encodable inline (the inline range stays 2^30 wide; bigger integers
/// fall back to the dictionary).
const INLINE_MAX: u32 = (1 << 30) - 1;

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

/// Whether an id encodes an inline integer value. (`INLINE_BASE << 1` would overflow `u32`,
/// so the upper bound is expressed via the inline width.)
#[inline]
pub fn is_inline(id: Id) -> bool {
    id >= INLINE_BASE && id - INLINE_BASE <= INLINE_MAX
}

/// The inline id of an integer VALUE in the inline range — the id the canonical
/// `xsd:integer` literal of that value interns/looks up to. The engine's fast path for
/// resolving COMPUTED (BIND/aggregate) integer values to ids without constructing a
/// term, or even its lexical form.
#[inline]
pub fn inline_id_of_int(v: i64) -> Option<Id> {
    if (0..=INLINE_MAX as i64).contains(&v) {
        Some(INLINE_BASE + v as u32)
    } else {
        None
    }
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
    terms: Vec<Stored>,                       // id-1 -> compact term (empty once compacted)
    table: HashTable<Id>,                     // hash(term) -> id (bare ids, compared via the arena)
    // When `Some` (after `into_blob`), id->term is served from a single concatenated term
    // BLOB + per-term offsets instead of `Vec<Stored>` — no per-term `Box<str>` allocation
    // overhead, ~half the resident dict bytes. `table` is kept (lookup verifies via the
    // blob); `terms` is empty. The memory-bound (browser) storage mode for the dictionary.
    blob: Option<(Vec<u8>, Vec<u32>)>,
    // When `Some` (after `open_mmap`), term(id)/term_parts/lookup are served from mmap'd
    // files and `terms`/`table` are empty — the out-of-core, minimal-RAM dictionary.
    #[cfg(feature = "mmap")]
    mapped: Option<MappedDict>,
    // APPEND-ONLY growth over the compacted storage modes (delta-overlay updates, T17):
    // ids `1..=base` are served by the blob / mmap'd record store; freshly interned terms
    // go to the `terms` arena with ids `base + i + 1`. Always 0 in the plain arena mode,
    // so the arena hot paths are unchanged (one predictable comparison).
    base: usize,
}

/// The compact per-term storage. An IRI is `(prefix id, suffix)`; a literal is
/// `(value, datatype id, optional language)`; a blank node is its label. An RDF 1.2
/// TRIPLE TERM (`<<( s p o )>>`) is stored STRUCTURALLY as the ids of its three
/// components (which are interned first, so a child id is always lower than — or an
/// inline id distinct from — the triple's own id; nesting recurses through the object).
enum Stored {
    Iri { prefix: u32, suffix: Box<str> },
    Lit { value: Box<str>, datatype: u32, lang: Option<Box<str>> },
    Blank(Box<str>),
    Triple([Id; 3]),
}

/// A borrowed view of a dictionary term's string components (no allocation), for
/// serialising results directly from ids. An IRI is its namespace prefix + local
/// suffix; a literal its lexical value, datatype IRI and optional language.
pub enum TermParts<'a> {
    Iri { prefix: &'a str, suffix: &'a str },
    Lit { value: &'a str, datatype: &'a str, lang: Option<&'a str> },
    Blank(&'a str),
    /// An RDF 1.2 triple term: the ids of its subject/predicate/object (resolve each via
    /// `term_parts`/`term` recursively — a child may also be an inline-integer id).
    Triple([Id; 3]),
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

/// Content hash of a structurally stored triple term — over the (already-interned) ids of
/// its components, NOT strings. Unlike the other term kinds this hash is dict-relative
/// (the ids are), so a `Term::Triple` is hashed only AFTER its children are resolved in
/// the target dict (`intern`/`lookup` handle this; `hash_term` cannot).
#[inline]
fn hash_triple_ids(ids: [Id; 3]) -> u64 {
    let mut h = rustc_hash::FxHasher::default();
    h.write_u8(3);
    h.write_u32(ids[0]);
    h.write_u32(ids[1]);
    h.write_u32(ids[2]);
    h.finish()
}

/// NOTE: returns a placeholder for `Term::Triple` — a triple term's hash is over its
/// component IDS (dict-relative), so callers must resolve those first (see
/// `Dict::intern` / `Dict::lookup`, which intercept `Term::Triple` before hashing).
/// The STORED language slot of a literal: the BCP47 tag, with an RDF 1.2 base direction
/// appended as `lang--dir` (the SPARQL/Turtle surface syntax — `--` can never occur inside
/// a valid language tag, so the encoding is unambiguous). One string field keeps the
/// storage layout, hashing and equality unchanged; [`reconstruct_ref`] splits it back out.
#[inline]
fn lang_with_dir(l: &Literal) -> Option<std::borrow::Cow<'_, str>> {
    match l.direction() {
        Some(d) => Some(std::borrow::Cow::Owned(format!("{}--{d}", l.language().unwrap_or("")))),
        None => l.language().map(std::borrow::Cow::Borrowed),
    }
}

#[inline]
fn hash_term(t: &Term) -> u64 {
    match t {
        Term::NamedNode(n) => hash_iri(n.as_str()),
        Term::Literal(l) => hash_lit(l.value(), l.datatype().as_str(), lang_with_dir(l).as_deref()),
        Term::BlankNode(b) => hash_blank(b.as_str()),
        _ => 0,
    }
}

/// Content hash of a borrowed term — same value as `hash_term` for the same term, computed
/// from already-split components (no `Term`, no IRI concat). Used to route the sharded
/// interner without reconstructing a `Term` per occurrence.
fn hash_termparts(tp: &TermParts) -> u64 {
    match tp {
        TermParts::Iri { prefix, suffix } => hash_iri_parts(prefix, suffix),
        TermParts::Lit { value, datatype, lang } => hash_lit(value, datatype, *lang),
        TermParts::Blank(b) => hash_blank(b),
        // Dict-relative (ids of the SOURCE dict): only meaningful within one dict.
        // The cross-dict parts paths (`intern_partials`) reject triple terms explicitly.
        TermParts::Triple(ids) => hash_triple_ids(*ids),
    }
}

#[inline]
fn hash_stored(s: &Stored, prefixes: &[Box<str>], datatypes: &[NamedNode]) -> u64 {
    match s {
        Stored::Iri { prefix, suffix } => hash_iri_parts(&prefixes[*prefix as usize], suffix),
        Stored::Lit { value, datatype, lang } => hash_lit(value, datatypes[*datatype as usize].as_str(), lang.as_deref()),
        Stored::Blank(b) => hash_blank(b),
        Stored::Triple(ids) => hash_triple_ids(*ids),
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

/// NOTE: `Term::Triple` always misses here — triple terms are matched by their component
/// IDS (see `Dict::lookup`, which resolves the children and intercepts before this path).
#[inline]
fn stored_eq_term(s: &Stored, q: &Term, prefixes: &[Box<str>], datatypes: &[NamedNode]) -> bool {
    match q {
        Term::NamedNode(n) => stored_is_iri(s, n.as_str(), prefixes),
        Term::Literal(l) => stored_is_lit(s, l.value(), l.datatype().as_str(), lang_with_dir(l).as_deref(), datatypes),
        Term::BlankNode(b) => matches!(s, Stored::Blank(x) if **x == *b.as_str()),
        _ => false,
    }
}

/// Rebuilds an RDF 1.2 triple term from its stored component ids. Interning only ever
/// stores a valid (IRI|blank, IRI, any-term) shape, so the inner matches cannot fail on
/// ids produced by this dict.
fn reconstruct_triple(d: &Dict, ids: [Id; 3]) -> Term {
    let subject: oxrdf::NamedOrBlankNode = match d.term(ids[0]) {
        Term::NamedNode(n) => n.into(),
        Term::BlankNode(b) => b.into(),
        other => unreachable!("stored triple-term subject must be an IRI or blank node, got {other}"),
    };
    let predicate = match d.term(ids[1]) {
        Term::NamedNode(n) => n,
        other => unreachable!("stored triple-term predicate must be an IRI, got {other}"),
    };
    Term::Triple(Box::new(oxrdf::Triple::new(subject, predicate, d.term(ids[2]))))
}

// ---- Memory-mapped (out-of-core) dictionary --------------------------------------
// The in-memory `Dict` holds `terms: Vec<Stored>` (~1 GB+ at 100M terms) plus the
// lookup `HashTable`, and rebuilds both on open. For querying a dataset whose indexes
// are already mmap'd, that resident RAM (and the rebuild time) is the last big cost.
// `MappedDict` instead serves term(id)/term_parts/lookup straight from mmap'd files —
// nothing big is resident, and open is just `mmap` (no parse, no table rebuild).

/// A borrowed view of a compact stored term, parsed zero-copy from a term BLOB (the
/// concatenated record format shared by the in-memory compacted dict and the mmap'd
/// out-of-core dict). Mirrors [`Stored`] but with `&str` slices into the blob.
enum StoredRef<'a> {
    Iri { prefix: u32, suffix: &'a str },
    Lit { value: &'a str, datatype: u32, lang: Option<&'a str> },
    Blank(&'a str),
    Triple([Id; 3]),
}

#[inline]
fn rd_u32(b: &[u8], p: &mut usize) -> u32 {
    let v = u32::from_le_bytes([b[*p], b[*p + 1], b[*p + 2], b[*p + 3]]);
    *p += 4;
    v
}

#[inline]
fn rd_str<'a>(b: &'a [u8], p: &mut usize) -> &'a str {
    let n = rd_u32(b, p) as usize;
    let s = &b[*p..*p + n];
    *p += n;
    // SAFETY: written from a `&str` in `save_mmap`; the blob is immutable.
    unsafe { std::str::from_utf8_unchecked(s) }
}

/// Parses one term record (the blob record format) at `b[0..]`.
#[inline]
fn parse_stored_ref(b: &[u8]) -> StoredRef<'_> {
    let mut p = 1;
    match b[0] {
        0 => {
            let prefix = rd_u32(b, &mut p);
            StoredRef::Iri { prefix, suffix: rd_str(b, &mut p) }
        }
        1 => {
            let value = rd_str(b, &mut p);
            let datatype = rd_u32(b, &mut p);
            let lang = if b[p] == 1 {
                p += 1;
                Some(rd_str(b, &mut p))
            } else {
                None
            };
            StoredRef::Lit { value, datatype, lang }
        }
        3 => StoredRef::Triple([rd_u32(b, &mut p), rd_u32(b, &mut p), rd_u32(b, &mut p)]),
        _ => StoredRef::Blank(rd_str(b, &mut p)),
    }
}

fn reconstruct_ref(d: &Dict, s: &StoredRef) -> Term {
    match *s {
        StoredRef::Iri { prefix, suffix } => {
            let p = &d.prefixes[prefix as usize];
            let mut iri = String::with_capacity(p.len() + suffix.len());
            iri.push_str(p);
            iri.push_str(suffix);
            Term::NamedNode(NamedNode::new_unchecked(iri))
        }
        StoredRef::Lit { value, datatype, lang } => Term::Literal(match lang {
            // A stored `lang--dir` slot is an RDF 1.2 directional language-tagged string
            // (see `lang_with_dir`); a plain tag is an ordinary one.
            Some(l) => match l.split_once("--") {
                Some((tag, dir)) => Literal::new_directional_language_tagged_literal_unchecked(
                    value.to_string(),
                    tag.to_string(),
                    if dir == "rtl" { oxrdf::BaseDirection::Rtl } else { oxrdf::BaseDirection::Ltr },
                ),
                None => Literal::new_language_tagged_literal_unchecked(value.to_string(), l.to_string()),
            },
            None => Literal::new_typed_literal(value.to_string(), d.datatypes[datatype as usize].clone()),
        }),
        StoredRef::Blank(b) => Term::BlankNode(oxrdf::BlankNode::new_unchecked(b.to_string())),
        StoredRef::Triple(ids) => reconstruct_triple(d, ids),
    }
}

/// NOTE: like `stored_eq_term`, `Term::Triple` always misses here — `Dict::lookup`
/// resolves a triple term's component ids first and matches `StoredRef::Triple` by id.
fn stored_ref_eq_term(s: &StoredRef, q: &Term, prefixes: &[Box<str>], datatypes: &[NamedNode]) -> bool {
    match (s, q) {
        (StoredRef::Iri { prefix, suffix }, Term::NamedNode(n)) => {
            let p = &prefixes[*prefix as usize];
            let iri = n.as_str();
            iri.len() == p.len() + suffix.len() && iri.starts_with(p.as_ref()) && iri[p.len()..] == **suffix
        }
        (StoredRef::Lit { value, datatype, lang }, Term::Literal(l)) => {
            *value == l.value()
                && lang.as_deref() == lang_with_dir(l).as_deref()
                && datatypes[*datatype as usize].as_str() == l.datatype().as_str()
        }
        (StoredRef::Blank(x), Term::BlankNode(b)) => *x == b.as_str(),
        _ => false,
    }
}

/// A zero-copy `StoredRef` view of an arena `Stored` — so the appended-term (arena-over-
/// blob/mmap) paths can share the `StoredRef`-based comparison/serialisation code.
#[inline]
fn stored_as_ref(s: &Stored) -> StoredRef<'_> {
    match s {
        Stored::Iri { prefix, suffix } => StoredRef::Iri { prefix: *prefix, suffix },
        Stored::Lit { value, datatype, lang } => {
            StoredRef::Lit { value, datatype: *datatype, lang: lang.as_deref() }
        }
        Stored::Blank(b) => StoredRef::Blank(b),
        Stored::Triple(ids) => StoredRef::Triple(*ids),
    }
}

/// Content hash of a `StoredRef` record — same value as `hash_stored` for the same term.
#[inline]
fn hash_stored_ref(s: &StoredRef, prefixes: &[Box<str>], datatypes: &[NamedNode]) -> u64 {
    match s {
        StoredRef::Iri { prefix, suffix } => hash_iri_parts(&prefixes[*prefix as usize], suffix),
        StoredRef::Lit { value, datatype, lang } => hash_lit(value, datatypes[*datatype as usize].as_str(), *lang),
        StoredRef::Blank(b) => hash_blank(b),
        StoredRef::Triple(ids) => hash_triple_ids(*ids),
    }
}

// `StoredRef` counterparts of the stored-vs-query-component comparisons, for the
// blob/mmap'd base records consulted by the append-capable intern paths.

#[inline]
fn stored_ref_is_iri(s: &StoredRef, iri: &str, prefixes: &[Box<str>]) -> bool {
    match s {
        StoredRef::Iri { prefix, suffix } => {
            let p = &prefixes[*prefix as usize];
            iri.len() == p.len() + suffix.len() && iri.as_bytes().starts_with(p.as_bytes()) && iri[p.len()..] == **suffix
        }
        _ => false,
    }
}

#[inline]
fn stored_ref_is_iri_parts(s: &StoredRef, prefix: &str, suffix: &str, prefixes: &[Box<str>]) -> bool {
    match s {
        StoredRef::Iri { prefix: pid, suffix: suf } => prefixes[*pid as usize].as_ref() == prefix && *suf == suffix,
        _ => false,
    }
}

#[inline]
fn stored_ref_is_lit(s: &StoredRef, value: &str, datatype: &str, lang: Option<&str>, datatypes: &[NamedNode]) -> bool {
    match s {
        StoredRef::Lit { value: v, datatype: dt, lang: lg } => {
            *v == value && *lg == lang && datatypes[*dt as usize].as_str() == datatype
        }
        _ => false,
    }
}

/// Content hash of any id present in the lookup `table` (blob-base or appended-arena —
/// mapped-base ids are never in the table; they are found via the mmap'd sorted index).
/// Free function so it can serve hashbrown's rehash closures while `table` is borrowed.
#[inline]
fn hash_tabled(
    id: Id,
    base: usize,
    terms: &[Stored],
    blob: &Option<(Vec<u8>, Vec<u32>)>,
    prefixes: &[Box<str>],
    datatypes: &[NamedNode],
) -> u64 {
    let i = (id - 1) as usize;
    if i >= base {
        return hash_stored(&terms[i - base], prefixes, datatypes);
    }
    let (b, offs) = blob.as_ref().expect("a tabled id below `base` must be a blob id");
    hash_stored_ref(&parse_stored_ref(&b[offs[i] as usize..]), prefixes, datatypes)
}

/// The mmap-backed term store + lookup index (out-of-core dictionary).
#[cfg(feature = "mmap")]
struct MappedDict {
    blob: memmap2::Mmap,    // concatenated term records (the format `save_mmap` writes)
    offsets: memmap2::Mmap, // [u64; n] byte offset of each term record in `blob`
    hashes: memmap2::Mmap,  // [u64; n] content hashes, SORTED (for lookup)
    hashids: memmap2::Mmap, // [u32; n] term ids parallel to `hashes`
}

#[cfg(feature = "mmap")]
impl MappedDict {
    #[inline]
    fn slice_u64(m: &memmap2::Mmap) -> &[u64] {
        // SAFETY: written as little-endian u64; mmap base is page-aligned (>= 8).
        unsafe { std::slice::from_raw_parts(m.as_ptr().cast::<u64>(), m.len() / 8) }
    }
    #[inline]
    fn offsets(&self) -> &[u64] {
        Self::slice_u64(&self.offsets)
    }
    #[inline]
    fn hashes(&self) -> &[u64] {
        Self::slice_u64(&self.hashes)
    }
    #[inline]
    fn hashids(&self) -> &[u32] {
        // SAFETY: written as little-endian u32; mmap base is page-aligned (>= 4).
        unsafe { std::slice::from_raw_parts(self.hashids.as_ptr().cast::<u32>(), self.hashids.len() / 4) }
    }
    /// The parsed term record for a 1-based id.
    #[inline]
    fn stored(&self, id: Id) -> StoredRef<'_> {
        let off = self.offsets()[(id - 1) as usize] as usize;
        parse_stored_ref(&self.blob[off..])
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

    /// Assigns the next id to a freshly built `Stored` and indexes it. With a compacted
    /// base (blob/mmap), the new term is APPENDED to the arena above `base` — the dict's
    /// append-only growth path for delta-overlay updates.
    #[inline]
    fn push(&mut self, hash: u64, stored: Stored) -> Id {
        let id = (self.base + self.terms.len()) as Id + 1; // 1-based
        // Enforced in release too: once the (non-inline) dictionary reaches INLINE_BASE
        // distinct terms, new ids would collide with the inline-integer range and decode
        // as integers — silent corruption. 2^31 ≈ 2.1B distinct non-integer terms is the
        // hard capacity limit of the u32 scheme; fail loudly (widen Id to u64).
        assert!(id < INLINE_BASE, "dictionary exceeded the id capacity (2^31 distinct non-integer terms); widen Id to u64");
        self.terms.push(stored);
        let (base, terms, blob, prefixes, datatypes) =
            (self.base, &self.terms, &self.blob, &self.prefixes, &self.datatypes);
        self.table.insert_unique(hash, id, |&i| hash_tabled(i, base, terms, blob, prefixes, datatypes));
        id
    }

    /// Finds a term in the MAPPED base by content hash (binary search of the mmap'd
    /// sorted-hash index, verifying candidates with `eq`). `None` when not mapped or
    /// absent — the caller then consults the table (blob base + appended arena terms).
    #[cfg(feature = "mmap")]
    #[inline]
    fn mapped_find(&self, hash: u64, eq: impl Fn(&StoredRef) -> bool) -> Option<Id> {
        let m = self.mapped.as_ref()?;
        let hashes = m.hashes();
        let mut i = hashes.partition_point(|&h| h < hash);
        let ids = m.hashids();
        while i < hashes.len() && hashes[i] == hash {
            let id = ids[i];
            if eq(&m.stored(id)) {
                return Some(id);
            }
            i += 1;
        }
        None
    }

    /// The record of a TABLED id (blob base, or appended arena) as needed by the intern
    /// comparison closures. Pure-arena dicts (`base == 0`) always take the arena branch,
    /// so the bulk-load hot path is unchanged beyond one predictable comparison.
    #[inline]
    fn tabled_base_ref(&self, id: Id) -> StoredRef<'_> {
        let (blob, offs) = self.blob.as_ref().expect("a tabled id below `base` requires the blob");
        parse_stored_ref(&blob[offs[(id - 1) as usize] as usize..])
    }

    #[inline]
    fn tabled_is_iri(&self, id: Id, iri: &str) -> bool {
        let i = (id - 1) as usize;
        if i >= self.base {
            stored_is_iri(&self.terms[i - self.base], iri, &self.prefixes)
        } else {
            stored_ref_is_iri(&self.tabled_base_ref(id), iri, &self.prefixes)
        }
    }

    #[inline]
    fn tabled_is_iri_parts(&self, id: Id, prefix: &str, suffix: &str) -> bool {
        let i = (id - 1) as usize;
        if i >= self.base {
            stored_is_iri_parts(&self.terms[i - self.base], prefix, suffix, &self.prefixes)
        } else {
            stored_ref_is_iri_parts(&self.tabled_base_ref(id), prefix, suffix, &self.prefixes)
        }
    }

    #[inline]
    fn tabled_is_lit(&self, id: Id, value: &str, datatype: &str, lang: Option<&str>) -> bool {
        let i = (id - 1) as usize;
        if i >= self.base {
            stored_is_lit(&self.terms[i - self.base], value, datatype, lang, &self.datatypes)
        } else {
            stored_ref_is_lit(&self.tabled_base_ref(id), value, datatype, lang, &self.datatypes)
        }
    }

    #[inline]
    fn tabled_is_blank(&self, id: Id, label: &str) -> bool {
        let i = (id - 1) as usize;
        if i >= self.base {
            matches!(&self.terms[i - self.base], Stored::Blank(b) if **b == *label)
        } else {
            matches!(self.tabled_base_ref(id), StoredRef::Blank(b) if b == label)
        }
    }

    #[inline]
    fn tabled_is_triple(&self, id: Id, ids: [Id; 3]) -> bool {
        let i = (id - 1) as usize;
        if i >= self.base {
            matches!(&self.terms[i - self.base], Stored::Triple(x) if *x == ids)
        } else {
            matches!(self.tabled_base_ref(id), StoredRef::Triple(x) if x == ids)
        }
    }

    #[inline]
    fn tabled_eq_term(&self, id: Id, term: &Term) -> bool {
        let i = (id - 1) as usize;
        if i >= self.base {
            stored_eq_term(&self.terms[i - self.base], term, &self.prefixes, &self.datatypes)
        } else {
            stored_ref_eq_term(&self.tabled_base_ref(id), term, &self.prefixes, &self.datatypes)
        }
    }

    /// Interns an IRI term from its string, returning its id.
    #[inline]
    pub fn intern_iri(&mut self, iri: &str) -> Id {
        let hash = hash_iri(iri);
        // Mapped base terms live in the mmap'd sorted index, not the table.
        #[cfg(feature = "mmap")]
        if let Some(id) = self.mapped_find(hash, |s| stored_ref_is_iri(s, iri, &self.prefixes)) {
            return id;
        }
        if let Some(&id) = self.table.find(hash, |&id| self.tabled_is_iri(id, iri)) {
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
        #[cfg(feature = "mmap")]
        if let Some(id) = self.mapped_find(hash, |s| stored_ref_is_iri_parts(s, prefix, suffix, &self.prefixes)) {
            return id;
        }
        if let Some(&id) = self.table.find(hash, |&id| self.tabled_is_iri_parts(id, prefix, suffix)) {
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
        #[cfg(feature = "mmap")]
        if let Some(id) = self.mapped_find(hash, |s| stored_ref_is_lit(s, value, datatype, lang, &self.datatypes)) {
            return id;
        }
        if let Some(&id) = self.table.find(hash, |&id| self.tabled_is_lit(id, value, datatype, lang)) {
            return id;
        }
        let datatype = self.intern_datatype(datatype);
        self.push(hash, Stored::Lit { value: value.into(), datatype, lang: lang.map(Into::into) })
    }

    /// Interns a blank node from its label, returning its id.
    #[inline]
    pub fn intern_blank(&mut self, label: &str) -> Id {
        let hash = hash_blank(label);
        #[cfg(feature = "mmap")]
        if let Some(id) = self.mapped_find(hash, |s| matches!(s, StoredRef::Blank(b) if *b == label)) {
            return id;
        }
        if let Some(&id) = self.table.find(hash, |&id| self.tabled_is_blank(id, label)) {
            return id;
        }
        self.push(hash, Stored::Blank(label.into()))
    }

    /// Interns an RDF 1.2 triple term whose components are ALREADY interned in this dict
    /// (ids may include inline-integer ids). Content-addressed by the component ids, so
    /// separately-interned identical triples share one id.
    fn intern_triple_ids(&mut self, ids: [Id; 3]) -> Id {
        let hash = hash_triple_ids(ids);
        #[cfg(feature = "mmap")]
        if let Some(id) = self.mapped_find(hash, |s| matches!(s, StoredRef::Triple(x) if *x == ids)) {
            return id;
        }
        if let Some(&id) = self.table.find(hash, |&id| self.tabled_is_triple(id, ids)) {
            return id;
        }
        self.push(hash, Stored::Triple(ids))
    }

    /// Interns a term, returning its id (creating it if new). Dispatches to the
    /// component interners so the `Term` and byte-slice paths share one code path.
    /// An RDF 1.2 triple term is stored STRUCTURALLY: its s/p/o are interned first
    /// (recursing through a nested triple-term object) and the triple holds their ids.
    #[inline]
    pub fn intern(&mut self, term: &Term) -> Id {
        match term {
            Term::NamedNode(n) => self.intern_iri(n.as_str()),
            Term::Literal(l) => self.intern_lit(l.value(), l.datatype().as_str(), lang_with_dir(l).as_deref()),
            Term::BlankNode(b) => self.intern_blank(b.as_str()),
            Term::Triple(t) => {
                let s = match t.subject {
                    oxrdf::NamedOrBlankNode::NamedNode(ref n) => self.intern_iri(n.as_str()),
                    oxrdf::NamedOrBlankNode::BlankNode(ref b) => self.intern_blank(b.as_str()),
                };
                let p = self.intern_iri(t.predicate.as_str());
                let o = self.intern(&t.object);
                self.intern_triple_ids([s, p, o])
            }
        }
    }

    /// Interns a borrowed term (already-split components) without building a `Term` or
    /// concatenating the IRI — the fast path used by the sharded parallel merge.
    ///
    /// Triple terms are NOT supported here: `TermParts::Triple` carries ids of the
    /// SOURCE dict, which are meaningless in another dict. The callers that feed this
    /// (the sharded N-Triples bulk loaders) can never produce one — the byte-level
    /// N-Triples parser rejects RDF-star syntax — so this is a loud guard, not a path.
    #[inline]
    fn intern_parts(&mut self, tp: &TermParts) -> Id {
        match tp {
            TermParts::Iri { prefix, suffix } => self.intern_iri_parts(prefix, suffix),
            TermParts::Lit { value, datatype, lang } => self.intern_lit(value, datatype, *lang),
            TermParts::Blank(b) => self.intern_blank(b),
            TermParts::Triple(_) => {
                panic!("RDF-star triple terms are not supported by the sharded (parts-based) bulk interner; use the serial loader")
            }
        }
    }

    /// Returns the id for a term if present, else `NO_ID`.
    #[inline]
    pub fn lookup(&self, term: &Term) -> Id {
        // An RDF 1.2 triple term is matched STRUCTURALLY: resolve its components first
        // (a missing component means the triple term cannot be present either), then
        // find the triple by its component ids.
        if let Term::Triple(t) = term {
            let s = match t.subject {
                oxrdf::NamedOrBlankNode::NamedNode(ref n) => self.lookup(&Term::NamedNode(n.clone())),
                oxrdf::NamedOrBlankNode::BlankNode(ref b) => self.lookup(&Term::BlankNode(b.clone())),
            };
            let p = self.lookup(&Term::NamedNode(t.predicate.clone()));
            let o = self.lookup(&t.object);
            if s == NO_ID || p == NO_ID || o == NO_ID {
                return NO_ID;
            }
            return self.lookup_triple_ids([s, p, o]);
        }
        if let Some(id) = try_inline(term) {
            return id;
        }
        let hash = hash_term(term);
        // Mapped base first (binary search of the sorted hash index, verifying each
        // equal-hash candidate — content hashes can collide), then the table, which
        // covers the blob base and any APPENDED terms (delta-overlay growth).
        #[cfg(feature = "mmap")]
        if let Some(id) = self.mapped_find(hash, |s| stored_ref_eq_term(s, term, &self.prefixes, &self.datatypes)) {
            return id;
        }
        self.table.find(hash, |&id| self.tabled_eq_term(id, term)).copied().unwrap_or(NO_ID)
    }

    /// Returns the id for a literal given its components, else `NO_ID` — [`lookup`]
    /// without constructing an `oxrdf::Term` (the fast path for resolving computed
    /// BIND/aggregate values against the dictionary). Canonical small `xsd:integer`s
    /// resolve to their inline id, exactly like `lookup`/`intern_lit`.
    #[inline]
    pub fn lookup_lit(&self, value: &str, datatype: &str, lang: Option<&str>) -> Id {
        if let Some(id) = try_inline_lit(value, datatype) {
            return id;
        }
        let hash = hash_lit(value, datatype, lang);
        #[cfg(feature = "mmap")]
        if let Some(id) = self.mapped_find(hash, |s| stored_ref_is_lit(s, value, datatype, lang, &self.datatypes)) {
            return id;
        }
        self.table.find(hash, |&id| self.tabled_is_lit(id, value, datatype, lang)).copied().unwrap_or(NO_ID)
    }

    /// Returns the id of a triple term whose components resolved to `ids`, else `NO_ID`.
    /// Serves all three storage modes (arena, blob, mmap).
    fn lookup_triple_ids(&self, ids: [Id; 3]) -> Id {
        let hash = hash_triple_ids(ids);
        #[cfg(feature = "mmap")]
        if let Some(id) = self.mapped_find(hash, |s| matches!(s, StoredRef::Triple(x) if *x == ids)) {
            return id;
        }
        self.table.find(hash, |&id| self.tabled_is_triple(id, ids)).copied().unwrap_or(NO_ID)
    }

    /// Borrows a dictionary term's components WITHOUT reconstructing an `oxrdf::Term`
    /// (no allocation) — for serialising results straight from ids. Only valid for a
    /// real dictionary id (`1..INLINE_BASE`); the caller handles inline / local ids.
    #[inline]
    pub fn term_parts(&self, id: Id) -> TermParts<'_> {
        // The &str slices borrow the mmap/blob record store (or the arena), plus the
        // in-RAM prefix+datatype tables, all owned by `self` for `'_` — zero-copy.
        match self.record(id) {
            StoredRef::Iri { prefix, suffix } => TermParts::Iri { prefix: &self.prefixes[prefix as usize], suffix },
            StoredRef::Lit { value, datatype, lang } => {
                TermParts::Lit { value, datatype: self.datatypes[datatype as usize].as_str(), lang }
            }
            StoredRef::Blank(b) => TermParts::Blank(b),
            StoredRef::Triple(ids) => TermParts::Triple(ids),
        }
    }

    /// Returns the term for an id. Inline-integer ids are decoded directly; others are
    /// reconstructed from the compact record store (panics on an invalid index — ids
    /// come from the store).
    #[inline]
    pub fn term(&self, id: Id) -> Term {
        if is_inline(id) {
            return Term::Literal(Literal::new_typed_literal((id - INLINE_BASE).to_string(), xsd::INTEGER));
        }
        reconstruct_ref(self, &self.record(id))
    }

    /// The record for ANY 1-based dictionary id, across all storage modes: ids above
    /// `base` come from the (appended) arena; ids at or below it from the mmap'd files
    /// or the in-memory blob. Pure-arena dicts always take the first branch (base = 0).
    #[inline]
    fn record(&self, id: Id) -> StoredRef<'_> {
        let i = (id - 1) as usize;
        if i >= self.base {
            return stored_as_ref(&self.terms[i - self.base]);
        }
        #[cfg(feature = "mmap")]
        if let Some(m) = &self.mapped {
            return m.stored(id);
        }
        let (blob, offs) = self.blob.as_ref().expect("an id below `base` requires the blob or mmap store");
        parse_stored_ref(&blob[offs[i] as usize..])
    }

    /// Compacts the id→term storage into a single concatenated BLOB + per-term `u32`
    /// offsets, freeing the `Vec<Stored>` (with its per-term `Box<str>` allocations). The
    /// hash table is kept (lookup verifies candidates against the blob). Roughly halves
    /// the resident dictionary bytes — the memory-bound (browser) storage mode. A no-op if
    /// already compacted, memory-mapped, or empty.
    pub fn into_blob(mut self) -> Dict {
        #[cfg(feature = "mmap")]
        if self.mapped.is_some() {
            return self;
        }
        if self.blob.is_some() || self.terms.is_empty() {
            return self;
        }
        let mut blob: Vec<u8> = Vec::with_capacity(self.terms.len() * 8);
        let mut offsets: Vec<u32> = Vec::with_capacity(self.terms.len());
        for t in &self.terms {
            assert!(
                blob.len() <= u32::MAX as usize,
                "dictionary term blob exceeds 4 GiB; use the mmap dict (u64 offsets) at this scale"
            );
            offsets.push(blob.len() as u32);
            // Writing to a `Vec<u8>` is infallible.
            let _ = write_record(&mut blob, &stored_as_ref(t));
        }
        blob.shrink_to_fit();
        self.base = offsets.len(); // appended (delta-overlay) terms continue above the blob
        self.blob = Some((blob, offsets));
        self.terms = Vec::new(); // free the Stored arena (the win)
        self
    }

    /// Merges another (partial) dictionary into this one, returning the remap from the
    /// other's local ids to this dictionary's global ids: `remap[local - 1]` is the
    /// global id for the other's 1-based local id `local`. Used by the parallel bulk
    /// loader. Interns each of the other's terms directly from its compact components —
    /// no `Term` reconstruction, no IRI concatenation.
    pub fn merge_remap(&mut self, other: &Dict) -> Vec<Id> {
        self.terms.reserve(other.terms.len());
        {
            let (base, terms, blob, prefixes, datatypes) =
                (self.base, &self.terms, &self.blob, &self.prefixes, &self.datatypes);
            self.table
                .reserve(other.terms.len(), |&i| hash_tabled(i, base, terms, blob, prefixes, datatypes));
        }
        let mut remap: Vec<Id> = Vec::with_capacity(other.terms.len());
        for s in &other.terms {
            let id = match s {
                Stored::Iri { prefix, suffix } => self.intern_iri_parts(&other.prefixes[*prefix as usize], suffix),
                Stored::Lit { value, datatype, lang } => self.intern_lit(value, other.datatypes[*datatype as usize].as_str(), lang.as_deref()),
                Stored::Blank(b) => self.intern_blank(b),
                // A triple term's components always precede it in `other` (interning
                // pushes children first), so their remapped ids are already known.
                Stored::Triple(ids) => {
                    let m = |id: Id| if is_inline(id) { id } else { remap[(id - 1) as usize] };
                    self.intern_triple_ids([m(ids[0]), m(ids[1]), m(ids[2])])
                }
            };
            remap.push(id);
        }
        remap
    }

    /// (Re)builds the content-hash lookup table from the term arena — for dictionaries
    /// assembled WITHOUT incremental table inserts (`ShardedDict::into_merged`), so the
    /// result supports `lookup`/`intern` like any serially-built dict. The per-term
    /// hashing (the dominant cost) runs in parallel; the inserts are a fast serial pass
    /// (the table is pre-sized, so no resize re-hashing). Arena-mode only.
    pub fn build_table(&mut self) {
        debug_assert!(self.base == 0 && self.blob.is_none(), "build_table is arena-mode only");
        let hashes: Vec<u64> = {
            #[cfg(feature = "parallel")]
            {
                use rayon::prelude::*;
                self.terms.par_iter().map(|t| hash_stored(t, &self.prefixes, &self.datatypes)).collect()
            }
            #[cfg(not(feature = "parallel"))]
            self.terms.iter().map(|t| hash_stored(t, &self.prefixes, &self.datatypes)).collect()
        };
        let mut table = HashTable::with_capacity(self.terms.len());
        for (i, &h) in hashes.iter().enumerate() {
            table.insert_unique(h, (i as Id) + 1, |&j| hashes[(j - 1) as usize]);
        }
        self.table = table;
    }

    /// Serialises the dictionary (prefixes, datatypes, compact terms) to `path` in a
    /// compact binary format. The hash table is NOT written — it is rebuilt on `open`.
    /// Arena-mode only (it serialises `terms`); the compacted/mmap'd modes persist via
    /// [`save_mmap`](Self::save_mmap), which handles appended terms too.
    pub fn save(&self, path: &std::path::Path) -> std::io::Result<()> {
        use std::io::Write;
        assert_eq!(self.base, 0, "Dict::save is arena-mode only; use save_mmap for blob/mmap'd (or grown) dicts");
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
                Stored::Triple(ids) => {
                    w.write_all(&[3])?;
                    for id in ids {
                        w.write_all(&id.to_le_bytes())?;
                    }
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
        let timing = std::env::var("SPARQ_DICT_TIMING").is_ok();
        let t0 = std::time::Instant::now();
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
                3 => Stored::Triple([read_u32(&mut r)?, read_u32(&mut r)?, read_u32(&mut r)?]),
                other => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("bad term tag {other}"))),
            });
        }
        let t_parse = t0.elapsed();
        // Rebuild the hash table from the arena.
        let t1 = std::time::Instant::now();
        let mut table = HashTable::with_capacity(nt);
        for (i, t) in terms.iter().enumerate() {
            let id = (i as Id) + 1;
            let hash = hash_stored(t, &prefixes, &datatypes);
            table.insert_unique(hash, id, |&j| hash_stored(&terms[(j - 1) as usize], &prefixes, &datatypes));
        }
        if timing {
            eprintln!(
                "[dict open] {nt} terms: read+parse {:.2}s, hashtable rebuild {:.2}s",
                t_parse.as_secs_f64(),
                t1.elapsed().as_secs_f64(),
            );
        }
        Ok(Dict {
            prefixes,
            prefix_ids,
            datatypes,
            datatype_ids,
            terms,
            table,
            blob: None,
            #[cfg(feature = "mmap")]
            mapped: None,
            base: 0,
        })
    }

    /// Serialises the dictionary for the MEMORY-MAPPED, out-of-core open path: a small
    /// meta file (prefixes + datatypes) plus four mmap-friendly blobs — the term records,
    /// their byte offsets, and a hash-sorted `(hash, id)` lookup index. Opened by
    /// [`open_mmap`](Self::open_mmap) with NOTHING large resident and no table rebuild.
    #[cfg(feature = "mmap")]
    pub fn save_mmap(&self, dir: &std::path::Path) -> std::io::Result<()> {
        use std::io::Write;
        std::fs::create_dir_all(dir)?;
        // ALL ids — the blob/mmap'd base plus any APPENDED (delta-overlay) terms — so a
        // grown dictionary persists totally (the compaction path relies on this).
        let n = self.len();

        // meta: prefixes, datatypes, term count.
        let mut meta = std::io::BufWriter::new(std::fs::File::create(dir.join("dict-meta.bin"))?);
        meta.write_all(&(self.prefixes.len() as u32).to_le_bytes())?;
        for p in &self.prefixes {
            write_str(&mut meta, p)?;
        }
        meta.write_all(&(self.datatypes.len() as u32).to_le_bytes())?;
        for d in &self.datatypes {
            write_str(&mut meta, d.as_str())?;
        }
        meta.write_all(&(n as u64).to_le_bytes())?;
        meta.flush()?;

        // term blob + per-term byte offsets; collect (hash, id) for the lookup index.
        let mut blob = std::io::BufWriter::new(std::fs::File::create(dir.join("dict-terms.bin"))?);
        let mut offsets: Vec<u64> = Vec::with_capacity(n);
        let mut pairs: Vec<(u64, u32)> = Vec::with_capacity(n);
        let mut pos: u64 = 0;
        for id in 1..=n as Id {
            offsets.push(pos);
            let r = self.record(id);
            pos += write_record(&mut blob, &r)?;
            pairs.push((hash_stored_ref(&r, &self.prefixes, &self.datatypes), id));
        }
        blob.flush()?;
        write_pod_slice(&dir.join("dict-offs.bin"), &offsets)?;

        // hash-sorted parallel arrays (binary-searchable lookup).
        pairs.sort_unstable_by_key(|&(h, _)| h);
        let hashes: Vec<u64> = pairs.iter().map(|&(h, _)| h).collect();
        let ids: Vec<u32> = pairs.iter().map(|&(_, id)| id).collect();
        write_pod_slice(&dir.join("dict-hash.bin"), &hashes)?;
        write_pod_slice(&dir.join("dict-hid.bin"), &ids)?;
        Ok(())
    }

    /// Opens a dictionary written by [`save_mmap`](Self::save_mmap) with the term store +
    /// lookup index MEMORY-MAPPED: open is just `mmap` (no term parse, no table rebuild),
    /// and the large data stays off-heap. Only the small prefix/datatype tables are read
    /// into RAM. Read-only (no further interning).
    #[cfg(feature = "mmap")]
    pub fn open_mmap(dir: &std::path::Path) -> std::io::Result<Dict> {
        use std::io::Read;
        let mut r = std::io::BufReader::new(std::fs::File::open(dir.join("dict-meta.bin"))?);
        let np = read_u32(&mut r)? as usize;
        let mut prefixes = Vec::with_capacity(np);
        for _ in 0..np {
            prefixes.push(read_str(&mut r)?);
        }
        let nd = read_u32(&mut r)? as usize;
        let mut datatypes = Vec::with_capacity(nd);
        for _ in 0..nd {
            datatypes.push(NamedNode::new_unchecked(String::from(read_str(&mut r)?)));
        }
        let mut nbuf = [0u8; 8];
        r.read_exact(&mut nbuf)?;
        let len = u64::from_le_bytes(nbuf) as usize;

        let map = |name: &str| -> std::io::Result<memmap2::Mmap> {
            let f = std::fs::File::open(dir.join(name))?;
            // SAFETY: read-only mapping of a file owned by this dict for its lifetime.
            unsafe { memmap2::Mmap::map(&f) }
        };
        let mapped = MappedDict {
            blob: map("dict-terms.bin")?,
            offsets: map("dict-offs.bin")?,
            hashes: map("dict-hash.bin")?,
            hashids: map("dict-hid.bin")?,
        };
        Ok(Dict { prefixes, datatypes, mapped: Some(mapped), base: len, ..Default::default() })
    }

    pub fn len(&self) -> usize {
        // `base` is the blob/mmap'd record count (0 for a plain arena dict); appended
        // (delta-overlay) terms live in the arena above it.
        self.base + self.terms.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// A rough estimate of the dictionary's heap footprint in bytes (for
    /// benchmarking). Counts the compact `terms` arena (slots + suffix/value/lang
    /// bytes), the shared prefix + datatype tables, and the hash table (bare ids).
    pub fn heap_bytes(&self) -> usize {
        // A memory-mapped dictionary keeps only the small prefix/datatype tables resident;
        // the term blob + offsets + lookup index are mmap'd (OS page cache, not the heap).
        // Appended (delta-overlay) terms live in the in-RAM arena in every mode.
        let appended: usize = self.terms.capacity() * std::mem::size_of::<Stored>()
            + self.terms.iter().map(stored_owned_bytes).sum::<usize>();
        #[cfg(feature = "mmap")]
        if self.mapped.is_some() {
            let prefix_bytes: usize = self.prefixes.iter().map(|p| p.len() + std::mem::size_of::<Box<str>>()).sum();
            let dt_bytes: usize = self.datatypes.iter().map(|d| d.as_str().len() + 32).sum();
            let table = self.table.capacity() * (std::mem::size_of::<Id>() + 1);
            return appended + table + prefix_bytes + dt_bytes;
        }
        // Compacted (blob) mode: the term blob + u32 offsets + the kept hash table, plus
        // the small prefix/datatype tables — no per-`Stored` slot or per-`Box<str>` term.
        if let Some((blob, offs)) = &self.blob {
            let prefix_bytes: usize = self.prefixes.iter().map(|p| p.len() + std::mem::size_of::<Box<str>>()).sum::<usize>() * 2;
            let dt_bytes: usize = self.datatypes.iter().map(|d| d.as_str().len() + 32).sum();
            let table = self.table.capacity() * (std::mem::size_of::<Id>() + 1);
            return appended + blob.capacity() + offs.capacity() * 4 + table + prefix_bytes + dt_bytes;
        }
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

/// Writes one compact term record (the format `parse_stored_ref` reads) and returns the
/// number of bytes written — for building the mmap term blob + its offset index.
fn write_record(w: &mut impl std::io::Write, t: &StoredRef) -> std::io::Result<u64> {
    Ok(match t {
        StoredRef::Iri { prefix, suffix } => {
            w.write_all(&[0])?;
            w.write_all(&prefix.to_le_bytes())?;
            write_str(w, suffix)?;
            1 + 4 + 4 + suffix.len() as u64
        }
        StoredRef::Lit { value, datatype, lang } => {
            w.write_all(&[1])?;
            write_str(w, value)?;
            w.write_all(&datatype.to_le_bytes())?;
            let lang_bytes = match lang {
                Some(l) => {
                    w.write_all(&[1])?;
                    write_str(w, l)?;
                    1 + 4 + l.len() as u64
                }
                None => {
                    w.write_all(&[0])?;
                    1
                }
            };
            1 + (4 + value.len() as u64) + 4 + lang_bytes
        }
        StoredRef::Blank(b) => {
            w.write_all(&[2])?;
            write_str(w, b)?;
            1 + 4 + b.len() as u64
        }
        StoredRef::Triple(ids) => {
            w.write_all(&[3])?;
            for id in ids {
                w.write_all(&id.to_le_bytes())?;
            }
            1 + 12
        }
    })
}

/// Writes a slice of plain-old-data (u32/u64) as raw little-endian bytes (for the mmap
/// offset / hash / id arrays). Little-endian is assumed on read (all target platforms).
#[cfg(feature = "mmap")]
fn write_pod_slice<T: Copy>(path: &std::path::Path, data: &[T]) -> std::io::Result<()> {
    // SAFETY: T is u32/u64 (POD); we only read its bytes.
    let bytes = unsafe { std::slice::from_raw_parts(data.as_ptr().cast::<u8>(), std::mem::size_of_val(data)) };
    std::fs::write(path, bytes)
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
        Stored::Triple(_) => 0, // the three ids live inline in the slot
    }
}

// ---- Hash-sharded dictionary for PARALLEL bulk interning --------------------------
// The external-memory build's serial `merge_remap` (re-interning every parsed term into one
// global dict) is the ingest bottleneck — it's latency-bound on the single growing global
// hash table. `ShardedDict` splits it into N independent shards (a term routes to
// `hash % N`), so the interning parallelises with NO cross-shard contention. A term gets a
// TEMPORARY id `shard*STRIDE + shard_local_id` (STRIDE = INLINE_BASE/N ≥ max shard size);
// these temp ids sort in the SAME order as the final dense ids `base[shard] + local`
// (base = prefix-sum of shard sizes, monotonic), so the externally-sorted permutations need
// only an order-preserving remap — no re-sort. `into_merged` consumes the shards into one
// regular `Dict` (MOVING the term strings; only the small prefix/datatype id fields are
// remapped to unified tables), reusing the existing `save_mmap`/`numerics_of` path. Inline
// integer ids (≥ INLINE_BASE) are global and never enter a shard.

/// A hash-sharded interner — see the module comment above.
pub struct ShardedDict {
    shards: Vec<Dict>,
    stride: u32,
}

impl ShardedDict {
    pub fn new(n: usize) -> ShardedDict {
        let n = n.max(1);
        ShardedDict { shards: (0..n).map(|_| Dict::new()).collect(), stride: INLINE_BASE / n as u32 }
    }

    pub fn n(&self) -> usize {
        self.shards.len()
    }

    /// Route + intern a batch of `(tag, idx, term)` items (the terms must be NON-inline —
    /// inline integers are handled by the caller), returning `(tag, idx, temp-id)`. The
    /// per-shard interning runs in parallel (each shard is single-writer → no contention).
    ///
    /// RDF-star triple terms are NOT supported by the sharded interner (a triple's
    /// component terms would be interned both in their hash-routed shard and alongside
    /// the triple, breaking the term↔id bijection on merge); the N-Triples bulk loaders
    /// that feed it reject RDF-star syntax before it could reach here.
    pub fn intern_terms(&mut self, items: Vec<(u32, Id, Term)>) -> Vec<(u32, Id, Id)> {
        let n = self.shards.len();
        let stride = self.stride;
        let mut buckets: Vec<Vec<(u32, Id, Term)>> = (0..n).map(|_| Vec::new()).collect();
        for (tag, idx, t) in items {
            assert!(
                !matches!(t, Term::Triple(_)),
                "RDF-star triple terms are not supported by the sharded bulk interner; use the serial loader"
            );
            let s = (hash_term(&t) % n as u64) as usize;
            buckets[s].push((tag, idx, t));
        }
        let intern_bucket = |s: usize, shard: &mut Dict, bucket: Vec<(u32, Id, Term)>| -> Vec<(u32, Id, Id)> {
            bucket
                .into_iter()
                .map(|(tag, idx, t)| {
                    let lid = shard.intern(&t);
                    debug_assert!(lid < stride, "shard {s} exceeded STRIDE — raise shard count / widen Id");
                    (tag, idx, (s as u32) * stride + lid)
                })
                .collect()
        };
        #[cfg(feature = "parallel")]
        {
            use rayon::prelude::*;
            self.shards
                .par_iter_mut()
                .zip(buckets)
                .enumerate()
                .map(|(s, (shard, bucket))| intern_bucket(s, shard, bucket))
                .flatten()
                .collect()
        }
        #[cfg(not(feature = "parallel"))]
        {
            self.shards
                .iter_mut()
                .zip(buckets)
                .enumerate()
                .flat_map(|(s, (shard, bucket))| intern_bucket(s, shard, bucket))
                .collect()
        }
    }

    /// The fast bulk-merge path: route every (non-inline) term of each parsed partial dict to
    /// its shard and intern it IN PARALLEL across shards — interning straight from the
    /// partial's borrowed components (`term_parts`), so there is no `Term` allocation, no IRI
    /// concat, and the cheaper part-based hashing is reused (mirrors the serial `merge_remap`
    /// per-term cost, but parallel and contention-free). Returns a per-partial remap:
    /// `remap[p][local_id] = temp_id` (`remap[p][0]` unused).
    ///
    /// FULLY parallel (measured: the old serial hash-routing scan + serial remap-vec
    /// scatter were ~half the "merge" bucket): the routing runs per-partial in parallel
    /// (each partial fills its own per-shard sub-buckets — no shared state), the interning
    /// runs per-shard in parallel (each shard walks the partials IN ORDER, so per-shard id
    /// assignment — and therefore the merged dict and every downstream byte — is identical
    /// to the old serial-routed version), and each shard scatters its temp ids straight
    /// into the shared remap table (disjoint writes: a term instance `(partial, local-id)`
    /// is hash-routed to exactly one shard).
    pub fn intern_partials(&mut self, partials: &[(Dict, Vec<[Id; 3]>)]) -> Vec<Vec<Id>> {
        let n = self.shards.len();
        let stride = self.stride;
        // Route each partial's terms to per-shard sub-buckets (parallel over partials).
        fn route<'a>(pd: &'a Dict, n: usize) -> Vec<Vec<(Id, TermParts<'a>)>> {
            let mut b: Vec<Vec<(Id, TermParts<'a>)>> = (0..n).map(|_| Vec::with_capacity(pd.len() / n + 1)).collect();
            for i in 1..=pd.len() as Id {
                let tp = pd.term_parts(i);
                debug_assert!(
                    !matches!(tp, TermParts::Triple(_)),
                    "RDF-star triple terms are not supported by the sharded bulk interner"
                );
                let s = (hash_termparts(&tp) % n as u64) as usize;
                b[s].push((i, tp));
            }
            b
        }
        #[cfg(feature = "parallel")]
        let routed: Vec<Vec<Vec<(Id, TermParts)>>> = {
            use rayon::prelude::*;
            partials.par_iter().map(|(pd, _)| route(pd, n)).collect()
        };
        #[cfg(not(feature = "parallel"))]
        let routed: Vec<Vec<Vec<(Id, TermParts)>>> = partials.iter().map(|(pd, _)| route(pd, n)).collect();

        // Pre-size the remap table; shards scatter into it with DISJOINT writes.
        let mut remaps: Vec<Vec<Id>> = partials.iter().map(|(pd, _)| vec![0 as Id; pd.len() + 1]).collect();
        // Raw view of `remaps` so each shard can write its own (partial, local-id) slots
        // from a parallel context. SAFETY (disjointness): the hash routing above assigns
        // every (pidx, i) slot to exactly ONE shard, so no two shards write the same slot,
        // and nobody reads until the parallel scope ends.
        #[derive(Clone, Copy)]
        struct SlotPtr(*mut Id, usize);
        unsafe impl Send for SlotPtr {}
        unsafe impl Sync for SlotPtr {}
        let scatter: Vec<SlotPtr> = remaps.iter_mut().map(|v| (SlotPtr(v.as_mut_ptr(), v.len()))).collect();

        // Intern each shard's terms in parallel (single-writer per shard → no contention),
        // walking partials in order so per-shard id assignment matches the serial routing.
        let intern_shard = |s: usize, shard: &mut Dict| {
            for (pidx, per_partial) in routed.iter().enumerate() {
                let SlotPtr(ptr, len) = scatter[pidx];
                for (i, tp) in &per_partial[s] {
                    let lid = shard.intern_parts(tp);
                    debug_assert!(lid < stride, "shard {s} exceeded STRIDE — raise shard count / widen Id");
                    debug_assert!((*i as usize) < len);
                    // SAFETY: i < len (local ids are 1..=pd.len() < pd.len()+1) and this
                    // (pidx, i) slot is written by exactly this shard — see RemapScatter.
                    unsafe { ptr.add(*i as usize).write((s as u32) * stride + lid) };
                }
            }
        };
        #[cfg(feature = "parallel")]
        {
            use rayon::prelude::*;
            self.shards.par_iter_mut().enumerate().for_each(|(s, shard)| intern_shard(s, shard));
        }
        #[cfg(not(feature = "parallel"))]
        self.shards.iter_mut().enumerate().for_each(|(s, shard)| intern_shard(s, shard));
        remaps
    }

    /// `base[s]` = number of terms in shards `< s` (the final id of shard `s`'s local 1 is
    /// `base[s] + 1`).
    fn bases(&self) -> Vec<u64> {
        let mut base = Vec::with_capacity(self.shards.len() + 1);
        let mut acc = 0u64;
        base.push(0);
        for sh in &self.shards {
            acc += sh.terms.len() as u64;
            base.push(acc);
        }
        base
    }

    /// Consume the shards into one merged `Dict` (final dense ids in shard order) + the
    /// `(base, stride)` to remap temp ids → final ids. Moves term strings; only the small
    /// prefix/datatype id fields are remapped to the unified tables.
    pub fn into_merged(self) -> (Dict, Vec<u64>, u32) {
        let base = self.bases();
        let stride = self.stride;
        // Unify prefix + datatype tables; build per-shard local-id -> unified-id remaps.
        let (mut uni_prefixes, mut pidx): (Vec<Box<str>>, FxHashMap<Box<str>, u32>) = Default::default();
        let (mut uni_dts, mut didx): (Vec<NamedNode>, FxHashMap<Box<str>, u32>) = Default::default();
        let mut prefix_remap: Vec<Vec<u32>> = Vec::with_capacity(self.shards.len());
        let mut dt_remap: Vec<Vec<u32>> = Vec::with_capacity(self.shards.len());
        for sh in &self.shards {
            let pr = sh
                .prefixes
                .iter()
                .map(|p| {
                    *pidx.entry(p.clone()).or_insert_with(|| {
                        uni_prefixes.push(p.clone());
                        (uni_prefixes.len() - 1) as u32
                    })
                })
                .collect();
            prefix_remap.push(pr);
            let dr = sh
                .datatypes
                .iter()
                .map(|d| {
                    *didx.entry(d.as_str().into()).or_insert_with(|| {
                        uni_dts.push(d.clone());
                        (uni_dts.len() - 1) as u32
                    })
                })
                .collect();
            dt_remap.push(dr);
        }
        // Build the merged arena by MOVING each shard's Stored (only the id field remaps).
        // The move runs PER-SHARD IN PARALLEL into disjoint slices of the target arena
        // (offsets = the same `base` prefix sums) — at 1 B-triple scale this single pass
        // over every distinct term was a measurable serial tail of the consolidation.
        let total: usize = self.shards.iter().map(|s| s.terms.len()).sum();
        let mut terms: Vec<Stored> = Vec::with_capacity(total);
        {
            let remap_one = |stored: Stored, pr: &[u32], dr: &[u32]| -> Stored {
                match stored {
                    Stored::Iri { prefix, suffix } => Stored::Iri { prefix: pr[prefix as usize], suffix },
                    Stored::Lit { value, datatype, lang } => Stored::Lit { value, datatype: dr[datatype as usize], lang },
                    Stored::Blank(b) => Stored::Blank(b),
                    // Guarded out at `intern_terms` / `intern_parts` — a shard can never
                    // hold a triple term (see the `intern_terms` doc comment).
                    Stored::Triple(_) => unreachable!("triple terms cannot enter the sharded interner"),
                }
            };
            #[cfg(feature = "parallel")]
            {
                use rayon::prelude::*;
                // Split the spare capacity into one disjoint slice per shard.
                let mut spare: &mut [std::mem::MaybeUninit<Stored>] = &mut terms.spare_capacity_mut()[..total];
                let mut slices: Vec<&mut [std::mem::MaybeUninit<Stored>]> = Vec::with_capacity(self.shards.len());
                for sh in &self.shards {
                    let (head, tail) = spare.split_at_mut(sh.terms.len());
                    slices.push(head);
                    spare = tail;
                }
                self.shards
                    .into_par_iter()
                    .zip(slices)
                    .enumerate()
                    .for_each(|(s, (sh, out))| {
                        let (pr, dr) = (&prefix_remap[s], &dt_remap[s]);
                        for (slot, stored) in out.iter_mut().zip(sh.terms) {
                            slot.write(remap_one(stored, pr, dr));
                        }
                    });
                // SAFETY: every one of the `total` slots was initialised exactly once above
                // (the slices partition [0, total) and each shard fills its slice fully).
                unsafe { terms.set_len(total) };
            }
            #[cfg(not(feature = "parallel"))]
            for (s, sh) in self.shards.into_iter().enumerate() {
                let (pr, dr) = (&prefix_remap[s], &dt_remap[s]);
                for stored in sh.terms {
                    terms.push(remap_one(stored, pr, dr));
                }
            }
        }
        let prefix_ids = uni_prefixes.iter().enumerate().map(|(i, p)| (p.clone(), i as u32)).collect();
        let datatype_ids = uni_dts.iter().enumerate().map(|(i, d)| (d.as_str().into(), i as u32)).collect();
        let merged = Dict { prefixes: uni_prefixes, prefix_ids, datatypes: uni_dts, datatype_ids, terms, ..Default::default() };
        (merged, base, stride)
    }
}

/// Remap a temporary sharded id to its final dense id (inline ids pass through unchanged).
#[inline]
pub fn remap_sharded(id: Id, base: &[u64], stride: u32) -> Id {
    if id >= INLINE_BASE {
        return id; // inline integer — global, never sharded
    }
    let shard = (id / stride) as usize;
    let local = (id % stride) as u64;
    (base[shard] + local) as Id
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
    fn into_blob_roundtrips() {
        // Build a dict spanning all term kinds (prefixed IRIs, typed/lang/plain literals,
        // a blank node, an inline integer), then compact it and verify every term still
        // round-trips: term(id), term_parts(id), and lookup(term)->id are unchanged, and
        // a non-present term still misses.
        let mut d = Dict::new();
        let terms = vec![
            Term::NamedNode(NamedNode::new_unchecked("http://ex/alice")),
            Term::NamedNode(NamedNode::new_unchecked("http://ex/bob")),
            Term::NamedNode(NamedNode::new_unchecked("http://other.org/x#y")),
            Term::Literal(Literal::new_simple_literal("a plain string")),
            Term::Literal(Literal::new_language_tagged_literal_unchecked("café", "fr")),
            Term::Literal(Literal::new_typed_literal("1.5", xsd::DECIMAL)),
            Term::BlankNode(oxrdf::BlankNode::new_unchecked("b0")),
            int("42"),       // inline
            int("99999999"), // also inline (within range)
        ];
        let ids: Vec<Id> = terms.iter().map(|t| d.intern(t)).collect();
        let before_len = d.len();

        let d = d.into_blob();
        assert_eq!(d.len(), before_len, "len unchanged after compaction");
        for (t, &id) in terms.iter().zip(&ids) {
            assert_eq!(d.term(id), *t, "term({id}) round-trips");
            assert_eq!(d.lookup(t), id, "lookup round-trips");
        }
        // A term that was never interned must still miss.
        assert_eq!(d.lookup(&Term::NamedNode(NamedNode::new_unchecked("http://ex/absent"))), NO_ID);
        // Compaction is idempotent.
        let d = d.into_blob();
        assert_eq!(d.term(ids[0]), terms[0]);
    }

    #[test]
    fn sharded_dict_roundtrips_and_preserves_order() {
        // Intern terms (with cross-shard duplicates) through the sharded interner, merge,
        // and verify: distinct count, every term round-trips via remap→term, duplicates get
        // the same temp id, and temp-id order == final-id order (the no-re-sort invariant).
        let mut sd = ShardedDict::new(4);
        let mk = |i: usize| -> Term {
            let v = (i / 3) % 30; // value decorrelated from the type selector (i % 3)
            match i % 3 {
                0 => Term::NamedNode(NamedNode::new_unchecked(format!("http://ex/n{v}"))),
                1 => Term::Literal(Literal::new_simple_literal(format!("lit{v}"))),
                _ => Term::BlankNode(oxrdf::BlankNode::new_unchecked(format!("b{v}"))),
            }
        };
        let terms: Vec<Term> = (0..300).map(mk).collect();
        let items: Vec<(u32, Id, Term)> = terms.iter().enumerate().map(|(j, t)| (0, (j + 1) as Id, t.clone())).collect();
        let resolved = sd.intern_terms(items);
        let mut temp_of: std::collections::HashMap<Id, Id> = std::collections::HashMap::new();
        for (_, j, temp) in &resolved {
            temp_of.insert(*j, *temp);
        }
        let (merged, base, stride) = sd.into_merged();
        // 90 distinct terms: 30 IRIs + 30 literals + 30 blanks.
        assert_eq!(merged.len(), 90, "distinct term count");
        // Every term round-trips through remap→term.
        for (j, t) in terms.iter().enumerate() {
            let temp = temp_of[&((j + 1) as Id)];
            let fin = remap_sharded(temp, &base, stride);
            assert_eq!(merged.term(fin), *t, "term {j} round-trips");
        }
        // Duplicates (same content) share a temp id.
        assert_eq!(temp_of[&1], temp_of[&91], "j=0 and j=90 are the same term (n0)");
        // Order preservation: sorting distinct temp ids == sorting their final ids.
        let mut temps: Vec<Id> = temp_of.values().copied().collect();
        temps.sort_unstable();
        temps.dedup();
        let mut prev = 0;
        for &temp in &temps {
            let fin = remap_sharded(temp, &base, stride);
            assert!(fin > prev, "final ids strictly increase with temp ids (no re-sort needed)");
            prev = fin;
        }
        // Final ids are dense [1, 90].
        assert_eq!(prev, 90, "final ids are dense and 1-based");
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

    fn triple(s: &str, p: &str, o: Term) -> Term {
        Term::Triple(Box::new(oxrdf::Triple::new(
            NamedNode::new_unchecked(s),
            NamedNode::new_unchecked(p),
            o,
        )))
    }

    #[test]
    fn triple_terms_intern_structurally_and_roundtrip() {
        let mut d = Dict::new();
        let tt = triple("http://ex/alice", "http://ex/age", int("30"));
        let id = d.intern(&tt);
        assert!(id >= 1 && id < INLINE_BASE);
        // Children were interned (subject + predicate; the object is inline).
        assert_eq!(d.len(), 3, "subject + predicate + the triple itself (inline object not stored)");
        // term() rebuilds a structural Term::Triple, not a literal.
        assert_eq!(d.term(id), tt);
        // lookup agrees with intern; a separately-built identical triple shares the id.
        assert_eq!(d.lookup(&tt), id);
        assert_eq!(d.intern(&triple("http://ex/alice", "http://ex/age", int("30"))), id);
        // A different triple gets a different id; lookup with absent components misses.
        let other = triple("http://ex/bob", "http://ex/age", int("30"));
        let oid = d.intern(&other);
        assert_ne!(oid, id);
        assert_eq!(d.lookup(&triple("http://ex/carol", "http://ex/age", int("30"))), NO_ID);
        // A triple whose components exist but that was never interned also misses.
        assert_eq!(d.lookup(&triple("http://ex/alice", "http://ex/age", int("25"))), NO_ID);
    }

    #[test]
    fn nested_triple_terms_roundtrip() {
        // RDF 1.2 nests triple terms through the OBJECT position.
        let mut d = Dict::new();
        let inner = triple("http://ex/a", "http://ex/b", Term::NamedNode(NamedNode::new_unchecked("http://ex/c")));
        let outer = triple("http://ex/x", "http://ex/p", inner.clone());
        let oid = d.intern(&outer);
        let iid = d.intern(&inner);
        assert_ne!(oid, iid);
        assert_eq!(d.term(oid), outer);
        assert_eq!(d.term(iid), inner);
        assert_eq!(d.lookup(&outer), oid);
        // Blank-node subject round-trips too.
        let bsubj = Term::Triple(Box::new(oxrdf::Triple::new(
            oxrdf::BlankNode::new_unchecked("r0"),
            NamedNode::new_unchecked("http://ex/p"),
            Term::Literal(Literal::new_simple_literal("v")),
        )));
        let bid = d.intern(&bsubj);
        assert_eq!(d.term(bid), bsubj);
        assert_eq!(d.lookup(&bsubj), bid);
    }

    #[test]
    fn triple_terms_survive_blob_compaction() {
        let mut d = Dict::new();
        let inner = triple("http://ex/a", "http://ex/b", int("7"));
        let outer = triple("http://ex/x", "http://ex/p", inner.clone());
        let plain = Term::NamedNode(NamedNode::new_unchecked("http://ex/other"));
        let ids = [d.intern(&outer), d.intern(&inner), d.intern(&plain)];
        let d = d.into_blob();
        assert_eq!(d.term(ids[0]), outer);
        assert_eq!(d.term(ids[1]), inner);
        assert_eq!(d.term(ids[2]), plain);
        assert_eq!(d.lookup(&outer), ids[0]);
        assert_eq!(d.lookup(&inner), ids[1]);
        assert_eq!(d.lookup(&triple("http://ex/x", "http://ex/p", int("7"))), NO_ID);
    }

    #[test]
    fn triple_terms_survive_save_open() {
        let mut d = Dict::new();
        let inner = triple("http://ex/a", "http://ex/b", int("7"));
        let outer = triple("http://ex/x", "http://ex/p", inner.clone());
        let (oid, iid) = (d.intern(&outer), d.intern(&inner));
        let dir = std::env::temp_dir().join(format!("sparq-dict-star-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("dict.bin");
        d.save(&path).unwrap();
        let d2 = Dict::open(&path).unwrap();
        assert_eq!(d2.term(oid), outer);
        assert_eq!(d2.term(iid), inner);
        assert_eq!(d2.lookup(&outer), oid);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[cfg(feature = "mmap")]
    #[test]
    fn triple_terms_survive_save_open_mmap() {
        let mut d = Dict::new();
        let inner = triple("http://ex/a", "http://ex/b", int("7"));
        let outer = triple("http://ex/x", "http://ex/p", inner.clone());
        let (oid, iid) = (d.intern(&outer), d.intern(&inner));
        let dir = std::env::temp_dir().join(format!("sparq-dict-star-mmap-{}", std::process::id()));
        d.save_mmap(&dir).unwrap();
        let d2 = Dict::open_mmap(&dir).unwrap();
        assert_eq!(d2.term(oid), outer);
        assert_eq!(d2.term(iid), inner);
        assert_eq!(d2.lookup(&outer), oid);
        assert_eq!(d2.lookup(&triple("http://ex/x", "http://ex/p", int("8"))), NO_ID);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn merge_remap_translates_triple_term_components() {
        // A partial dict's triple term must be re-interned against the GLOBAL ids of its
        // components (which differ from the partial's local ids).
        let mut global = Dict::new();
        // Pre-populate the global dict so ids diverge from the partial's.
        for i in 0..5 {
            global.intern(&Term::NamedNode(NamedNode::new_unchecked(format!("http://pre/{i}"))));
        }
        let mut partial = Dict::new();
        let inner = triple("http://ex/a", "http://ex/b", int("7"));
        let outer = triple("http://ex/x", "http://ex/p", inner.clone());
        let local_outer = partial.intern(&outer);
        let remap = global.merge_remap(&partial);
        let gid = remap[(local_outer - 1) as usize];
        assert_eq!(global.term(gid), outer);
        assert_eq!(global.lookup(&outer), gid);
        assert_eq!(global.lookup(&inner), remap[(partial.lookup(&inner) - 1) as usize]);
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
