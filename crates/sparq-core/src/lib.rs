//! sparq-core: dictionary-encoded RDF storage with six permutation indexes.
//!
//! This is the storage substrate for the query engine: a [`Graph`] holds the
//! term [`Dict`]ionary and the [`TripleStore`] (six sorted permutations), and
//! is built from an RDF document via the bulk loader.

pub mod compress;
pub mod dict;
#[cfg(feature = "dict-spill")]
pub mod dictspill;
#[cfg(feature = "mmap")]
pub mod extsort;
mod nt;
pub mod store;
pub mod temporal;

use dict::{Dict, Id};
use temporal::Temporal;

/// Per-chunk parse output: a partial dictionary + that chunk's local-id triples, one
/// element per parallel chunk (the parallelizable half of ingest, merged downstream).
// [OPUS-4.8] Used only by `parse_block` (`#[cfg(feature = "parallel")]`); match that gating
// so neither the default non-parallel build nor the no-default-features (wasm) build flags
// the alias as dead code under the `-D warnings` clippy gate.
#[cfg(feature = "parallel")]
type ChunkPartials = Vec<(Dict, Vec<[Id; 3]>)>;
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
    numerics: NumData,
    /// Parallel to the dictionary: the precomputed comparison key of each
    /// `xsd:dateTime`/`xsd:dateTimeStamp`/`xsd:date` literal (see
    /// [`temporal::Temporal`]). The temporal twin of `numerics`: dateTime
    /// FILTER / ORDER BY / MIN/MAX read the timeline value O(1) from the cache
    /// instead of materialising the term and re-parsing its lexical per row.
    temporals: TempData,
    /// Named graphs (each a self-contained `Graph`), keyed by their name term. Empty for the
    /// usual single-default-graph load; populated by [`load_dataset`](Self::load_dataset) from
    /// N-Quads / TriG so the engine can evaluate `GRAPH <iri> { … }` / `GRAPH ?g { … }`.
    pub named: Vec<(Term, Graph)>,
    /// The write-ahead log for a DIRECTORY-BACKED graph (opened via [`open`](Self::open)):
    /// every [`apply_delta`](Self::apply_delta) batch is appended + fsync'd here BEFORE it
    /// is applied, and replayed into the delta-overlay on the next `open` — durability for
    /// incremental updates. `None` for in-memory graphs (updates are overlay-only).
    #[cfg(feature = "mmap")]
    wal: Option<Wal>,
}

/// [OPUS-4.8] (sq-5lf) An IMMUTABLE, logically-independent point-in-time view of a
/// [`Graph`], produced cheaply by [`Graph::snapshot`] (O(pending delta), never
/// O(triples)) via the structural fork — the immutable base storage is `Arc`-shared, so
/// a snapshot duplicates neither the permutation indexes nor the dictionary arena.
///
/// A `GraphSnapshot` reads exactly the triples present when it was taken and is
/// unaffected by any later mutation of the source graph; equally, the source can never
/// be reached through the snapshot. It [`Deref`](std::ops::Deref)s to `&Graph` (so every
/// graph READ method — `len`, `id_of`, `pattern`, `iter_ids`, `store`/`dict` access for
/// the query engine — is available and a `&GraphSnapshot` is usable anywhere a `&Graph`
/// is) but deliberately exposes NO `DerefMut` and no mutating method: the snapshot cannot
/// be mutated, so it cannot drift from its point-in-time state. It is [`Send`] + [`Sync`]
/// (a `Graph` is), so it can be published across threads — the production serving pattern
/// (one mutable master + a cheap immutable snapshot per commit / per closed RSP window).
///
/// If you need a snapshot you can then keep mutating, use [`Graph::fork`] (it returns a
/// `Graph`); `snapshot` is the read-only surface layered on top of it.
pub struct GraphSnapshot {
    graph: Graph,
}

impl GraphSnapshot {
    /// Consumes the snapshot, yielding the underlying point-in-time [`Graph`] as an
    /// independently mutable graph (it already carries its own `Arc`-shared base + delta —
    /// this is just dropping the read-only wrapper, O(1)). Use when a consumer that took an
    /// immutable snapshot now wants to apply further updates to that exact state, e.g.
    /// sparq-py's `Graph.copy()` returning a mutable copy.
    #[inline]
    pub fn into_graph(self) -> Graph {
        self.graph
    }

    /// Borrows the underlying [`Graph`] (also reachable via the [`Deref`](std::ops::Deref)
    /// impl; this is the explicit form for call sites that prefer it).
    #[inline]
    pub fn as_graph(&self) -> &Graph {
        &self.graph
    }
}

impl std::ops::Deref for GraphSnapshot {
    type Target = Graph;
    #[inline]
    fn deref(&self) -> &Graph {
        &self.graph
    }
}

/// Backing storage for the numeric-value cache (`numerics[id-1]` = f64 value of term
/// `id`, NaN for non-numeric): owned dense in RAM, mmap'd from disk (out-of-core), or
/// SPARSE — only the numeric terms in a hash map. Most RDF terms are IRIs/strings (NaN),
/// and small integers inline (carrying their own value, never cached), so the dense
/// cache is mostly — often entirely — NaN; the sparse form stores only the few real
/// numeric literals, the right shape for the memory-bound browser store.
enum NumData {
    Owned(Vec<f64>),
    /// The mmap'd dense cache, plus a small side map for terms APPENDED after open
    /// (delta-overlay updates) — the mmap'd file cannot grow, the dictionary can.
    #[cfg(feature = "mmap")]
    Mapped(memmap2::Mmap, rustc_hash::FxHashMap<Id, f64>),
    Sparse(rustc_hash::FxHashMap<Id, f64>),
    /// A FORKED graph's cache: the base graph's cache SHARED immutably (Arc) plus a
    /// small side map for terms interned after the fork — the in-RAM twin of
    /// `Mapped`'s grow-over-immutable-base shape. Lookup: base first, side map as the
    /// fallback (the side map only ever holds ids the base does not cover).
    /// INVARIANT: the inner cache is never itself `Forked` (fork flattens one level).
    Forked { base: std::sync::Arc<NumData>, extra: rustc_hash::FxHashMap<Id, f64> },
}

impl NumData {
    /// The cached numeric value of a 1-based dictionary id, or `None` if it is not a
    /// (cached) numeric literal. The engine's O(1) numeric fast path.
    #[inline]
    fn lookup(&self, id: Id) -> Option<f64> {
        match self {
            NumData::Sparse(m) => m.get(&id).copied(),
            #[cfg(feature = "mmap")]
            NumData::Mapped(_, extra) => match self.as_slice().get((id - 1) as usize) {
                Some(v) if !v.is_nan() => Some(*v),
                Some(_) => None,
                // Beyond the mmap'd dense cache: a term appended after open.
                None => extra.get(&id).copied(),
            },
            NumData::Owned(v) => {
                let v = *v.get((id - 1) as usize)?;
                if v.is_nan() {
                    None
                } else {
                    Some(v)
                }
            }
            NumData::Forked { base, extra } => {
                base.lookup(id).or_else(|| extra.get(&id).copied())
            }
        }
    }

    /// Appends cache entries for freshly interned dictionary ids `old_len+1 ..= dict.len()`
    /// (delta-overlay growth): the dense backing extends in place; the sparse / mmap'd
    /// backings record only the real numeric values in their side map.
    fn extend_for(&mut self, dict: &Dict, old_len: usize) {
        for i in old_len..dict.len() {
            let id = i as Id + 1;
            let v = numeric_of(&dict.term(id));
            match self {
                NumData::Owned(vec) => vec.push(v),
                NumData::Sparse(m) => {
                    if !v.is_nan() {
                        m.insert(id, v);
                    }
                }
                #[cfg(feature = "mmap")]
                NumData::Mapped(_, extra) => {
                    if !v.is_nan() {
                        extra.insert(id, v);
                    }
                }
                NumData::Forked { extra, .. } => {
                    if !v.is_nan() {
                        extra.insert(id, v);
                    }
                }
            }
        }
    }

    /// The cache for a structurally FORKED graph: the base cache shared immutably,
    /// new terms recorded in a per-fork side map. Forking a fork shares the same
    /// base Arc and copies the (small) side map; the FIRST fork of a flat cache
    /// pays a one-time O(n) copy to freeze the shareable base (the mmap'd backing
    /// is materialised — an `Mmap` cannot be cloned; forked graphs are in-memory).
    fn fork(&self) -> NumData {
        match self {
            NumData::Forked { base, extra } => NumData::Forked {
                base: std::sync::Arc::clone(base),
                extra: extra.clone(),
            },
            NumData::Owned(v) => NumData::Forked {
                base: std::sync::Arc::new(NumData::Owned(v.clone())),
                extra: rustc_hash::FxHashMap::default(),
            },
            NumData::Sparse(m) => NumData::Forked {
                base: std::sync::Arc::new(NumData::Sparse(m.clone())),
                extra: rustc_hash::FxHashMap::default(),
            },
            #[cfg(feature = "mmap")]
            NumData::Mapped(_, extra) => NumData::Forked {
                // Materialise the mmap'd dense part (an `Mmap` cannot be cloned);
                // ids the file does not cover stay in the carried side map.
                base: std::sync::Arc::new(NumData::Owned(self.as_slice().to_vec())),
                extra: extra.clone(),
            },
        }
    }

    /// Folds a forked cache's side map into a FRESH shared base (compaction): one
    /// dense/sparse backing covering the whole dictionary, kept in the SHAREABLE
    /// `Forked` shape (Arc base + empty side map) — so the side map stops growing
    /// AND the next fork is still an Arc bump, never an O(dict) re-freeze.
    /// Non-forked backings are returned as-is.
    fn fold(self, dict_len: usize) -> NumData {
        match self {
            NumData::Forked { base, extra } => {
                let folded = match &*base {
                    NumData::Owned(v) => {
                        let mut dense = v.clone();
                        dense.resize(dict_len, f64::NAN);
                        for (&id, &val) in &extra {
                            dense[(id - 1) as usize] = val;
                        }
                        NumData::Owned(dense)
                    }
                    NumData::Sparse(m) => {
                        let mut m = m.clone();
                        m.extend(extra);
                        NumData::Sparse(m)
                    }
                    // `fork` never freezes a Forked or Mapped base (invariant above).
                    _ => unreachable!("a forked numeric cache's base is Owned or Sparse"),
                };
                NumData::Forked { base: std::sync::Arc::new(folded), extra: rustc_hash::FxHashMap::default() }
            }
            other => other,
        }
    }

    /// The dense f64 slice — valid only for the Owned/Mapped backings (the ones `save`
    /// persists); the sparse backing is never written to disk. Only the mmap (save /
    /// mapped-lookup) paths need it.
    #[cfg(feature = "mmap")]
    #[inline]
    fn as_slice(&self) -> &[f64] {
        match self {
            NumData::Owned(v) => v,
            #[cfg(feature = "mmap")]
            NumData::Mapped(m, _) => {
                let n = m.len() / std::mem::size_of::<f64>();
                // SAFETY: numerics.bin is a whole number of f64; the mmap base is
                // page-aligned (>= the 8-byte f64 alignment).
                unsafe { std::slice::from_raw_parts(m.as_ptr().cast::<f64>(), n) }
            }
            NumData::Sparse(_) => unreachable!("as_slice on a sparse numeric cache"),
            NumData::Forked { .. } => unreachable!("as_slice on a forked numeric cache"),
        }
    }

    /// Resident heap bytes (a memory-mapped cache contributes 0 — it is page cache).
    #[inline]
    fn heap_bytes(&self) -> usize {
        match self {
            NumData::Owned(v) => v.capacity() * std::mem::size_of::<f64>(),
            #[cfg(feature = "mmap")]
            NumData::Mapped(_, extra) => extra.capacity() * 17,
            // hashbrown: ~(8-byte key + 8-byte f64 + 1 control byte) per slot.
            NumData::Sparse(m) => m.capacity() * 17,
            // Reachable footprint: the Arc-shared base is counted in full (like the
            // store's Arc-shared permutations), so a compacted graph — whose whole
            // cache lives in the folded base — still reports its memory. Forks that
            // share a base each report it (roborev 1945).
            NumData::Forked { base, extra } => base.heap_bytes() + extra.capacity() * 17,
        }
    }

    /// Converts a dense cache into the sparse form when it is mostly NaN (≥ 3/4 of terms
    /// non-numeric — almost always true), keeping only the real numeric literals. A no-op
    /// (kept dense) when numeric values are common enough that the map would not save.
    fn into_sparse_if_worthwhile(self) -> NumData {
        let dense = match &self {
            NumData::Owned(v) => v,
            _ => return self, // mmap'd/already-sparse: leave as is
        };
        let numeric = dense.iter().filter(|x| !x.is_nan()).count();
        if numeric * 4 > dense.len() {
            return self; // numeric-dense: the Vec is the better representation
        }
        let mut m: rustc_hash::FxHashMap<Id, f64> = rustc_hash::FxHashMap::default();
        m.reserve(numeric);
        for (i, &v) in dense.iter().enumerate() {
            if !v.is_nan() {
                m.insert(i as Id + 1, v);
            }
        }
        NumData::Sparse(m)
    }
}

/// Backing storage for the temporal-value cache, mirroring [`NumData`]: owned dense in
/// RAM (two parallel columns — a flag byte per term and an f64 instant per term), mmap'd
/// from `temporals.bin` (out-of-core), or SPARSE (only the temporal literals, the right
/// shape for the memory-bound browser store — most terms are not dates).
enum TempData {
    /// `cells[id-1]` — flag (see [`temp_flag`]; 0 = not temporal) + instant in ONE
    /// 16-byte cell, so a cache probe touches a single cache line (the probes are
    /// random-access from row order; split columns would double the misses).
    Owned(Vec<TempCell>),
    /// The mmap'd dense cache (`temporals.bin`: `n` little-endian f64 instants then `n`
    /// flag bytes), plus a side map for terms APPENDED after open (delta-overlay growth).
    #[cfg(feature = "mmap")]
    Mapped(memmap2::Mmap, rustc_hash::FxHashMap<Id, Temporal>),
    Sparse(rustc_hash::FxHashMap<Id, Temporal>),
    /// A FORKED graph's cache — see [`NumData::Forked`]: shared immutable base + a
    /// per-fork side map for terms interned after the fork.
    Forked { base: std::sync::Arc<TempData>, extra: rustc_hash::FxHashMap<Id, Temporal> },
}

/// Dense-cache flag byte of a temporal value: kind + timezone presence. 0 = not temporal.
fn temp_flag(t: Temporal) -> u8 {
    match (t.kind, t.has_tz) {
        (temporal::TemporalKind::DateTime, false) => 1,
        (temporal::TemporalKind::DateTime, true) => 2,
        (temporal::TemporalKind::Date, false) => 3,
        (temporal::TemporalKind::Date, true) => 4,
    }
}

/// Decodes a dense-cache cell back into the temporal value (`None` for flag 0).
fn temp_unflag(flag: u8, instant: f64) -> Option<Temporal> {
    let (kind, has_tz) = match flag {
        1 => (temporal::TemporalKind::DateTime, false),
        2 => (temporal::TemporalKind::DateTime, true),
        3 => (temporal::TemporalKind::Date, false),
        4 => (temporal::TemporalKind::Date, true),
        _ => return None,
    };
    Some(Temporal { instant, has_tz, kind })
}

impl TempData {
    /// The cached temporal value of a 1-based dictionary id, or `None` if it is not a
    /// (cached) dateTime/date literal. The engine's O(1) temporal fast path.
    #[inline]
    fn lookup(&self, id: Id) -> Option<Temporal> {
        let i = (id - 1) as usize;
        match self {
            TempData::Owned(cells) => cells.get(i).and_then(|c| temp_unflag(c.flag, c.instant)),
            TempData::Sparse(m) => m.get(&id).copied(),
            TempData::Forked { base, extra } => base.lookup(id).or_else(|| extra.get(&id).copied()),
            #[cfg(feature = "mmap")]
            TempData::Mapped(m, extra) => {
                let n = Self::mapped_len(m);
                if i < n {
                    // SAFETY: the instant section is `n` little-endian f64 at offset 0
                    // (page-aligned mmap base >= 8-byte alignment); flags follow at n*8.
                    let instant = unsafe { *m.as_ptr().cast::<f64>().add(i) };
                    temp_unflag(m[n * 8 + i], instant)
                } else {
                    extra.get(&id).copied() // appended after open
                }
            }
        }
    }

    /// Number of terms covered by a mapped `temporals.bin` (9 bytes per term).
    #[cfg(feature = "mmap")]
    #[inline]
    fn mapped_len(m: &memmap2::Mmap) -> usize {
        m.len() / 9
    }

    /// Appends cache entries for freshly interned dictionary ids `old_len+1 ..= dict.len()`
    /// (delta-overlay growth), mirroring [`NumData::extend_for`].
    fn extend_for(&mut self, dict: &Dict, old_len: usize) {
        for i in old_len..dict.len() {
            let id = i as Id + 1;
            let t = temporal_of_id(dict, id);
            match self {
                TempData::Owned(cells) => cells.push(match t {
                    Some(t) => TempCell { instant: t.instant, flag: temp_flag(t) },
                    None => TempCell { instant: f64::NAN, flag: 0 },
                }),
                TempData::Sparse(m) => {
                    if let Some(t) = t {
                        m.insert(id, t);
                    }
                }
                #[cfg(feature = "mmap")]
                TempData::Mapped(_, extra) => {
                    if let Some(t) = t {
                        extra.insert(id, t);
                    }
                }
                TempData::Forked { extra, .. } => {
                    if let Some(t) = t {
                        extra.insert(id, t);
                    }
                }
            }
        }
    }

    /// The cache for a structurally FORKED graph — mirror of [`NumData::fork`].
    fn fork(&self) -> TempData {
        match self {
            TempData::Forked { base, extra } => TempData::Forked {
                base: std::sync::Arc::clone(base),
                extra: extra.clone(),
            },
            TempData::Owned(cells) => TempData::Forked {
                base: std::sync::Arc::new(TempData::Owned(cells.clone())),
                extra: rustc_hash::FxHashMap::default(),
            },
            TempData::Sparse(m) => TempData::Forked {
                base: std::sync::Arc::new(TempData::Sparse(m.clone())),
                extra: rustc_hash::FxHashMap::default(),
            },
            #[cfg(feature = "mmap")]
            TempData::Mapped(m, extra) => {
                // Materialise the mmap'd cells (an `Mmap` cannot be cloned).
                let n = Self::mapped_len(m);
                // SAFETY: instants are `n` little-endian f64 at offset 0 (page-aligned).
                let instants = unsafe { std::slice::from_raw_parts(m.as_ptr().cast::<f64>(), n) };
                let cells: Vec<TempCell> = (0..n)
                    .map(|i| TempCell { instant: instants[i], flag: m[n * 8 + i] })
                    .collect();
                TempData::Forked {
                    base: std::sync::Arc::new(TempData::Owned(cells)),
                    extra: extra.clone(),
                }
            }
        }
    }

    /// Folds a forked cache's side map into a fresh shared base, keeping the
    /// shareable `Forked` shape — mirror of [`NumData::fold`].
    fn fold(self, dict_len: usize) -> TempData {
        match self {
            TempData::Forked { base, extra } => {
                let folded = match &*base {
                    TempData::Owned(cells) => {
                        let mut cells = cells.clone();
                        cells.resize(dict_len, TempCell { instant: f64::NAN, flag: 0 });
                        for (&id, &t) in &extra {
                            cells[(id - 1) as usize] = TempCell { instant: t.instant, flag: temp_flag(t) };
                        }
                        TempData::Owned(cells)
                    }
                    TempData::Sparse(m) => {
                        let mut m = m.clone();
                        m.extend(extra);
                        TempData::Sparse(m)
                    }
                    _ => unreachable!("a forked temporal cache's base is Owned or Sparse"),
                };
                TempData::Forked { base: std::sync::Arc::new(folded), extra: rustc_hash::FxHashMap::default() }
            }
            other => other,
        }
    }

    /// Resident heap bytes (a memory-mapped cache contributes 0 — it is page cache).
    #[inline]
    fn heap_bytes(&self) -> usize {
        match self {
            TempData::Owned(cells) => cells.capacity() * std::mem::size_of::<TempCell>(),
            #[cfg(feature = "mmap")]
            TempData::Mapped(_, extra) => extra.capacity() * 25,
            // hashbrown: ~(4-byte key pad to 8 + 16-byte Temporal + control) per slot.
            TempData::Sparse(m) => m.capacity() * 25,
            // Reachable footprint: the Arc-shared base is counted in full — see the
            // matching NumData::Forked arm (roborev 1945).
            TempData::Forked { base, extra } => base.heap_bytes() + extra.capacity() * 25,
        }
    }

    /// Converts a dense cache into the sparse form when it is mostly empty (≥ 3/4 of
    /// terms non-temporal — almost always true), mirroring the numerics cache.
    fn into_sparse_if_worthwhile(self) -> TempData {
        let cells = match &self {
            TempData::Owned(c) => c,
            _ => return self, // mmap'd/already-sparse: leave as is
        };
        let temporal = cells.iter().filter(|c| c.flag != 0).count();
        if temporal * 4 > cells.len() {
            return self; // temporal-dense: the cells are the better representation
        }
        let mut m: rustc_hash::FxHashMap<Id, Temporal> = rustc_hash::FxHashMap::default();
        m.reserve(temporal);
        for (i, c) in cells.iter().enumerate() {
            if let Some(t) = temp_unflag(c.flag, c.instant) {
                m.insert(i as Id + 1, t);
            }
        }
        TempData::Sparse(m)
    }
}

/// The cached temporal value of dictionary term `id`, parsed from its borrowed parts
/// (no `Term` materialisation). `None` for non-temporal terms and ill-formed lexicals.
fn temporal_of_id(dict: &Dict, id: Id) -> Option<Temporal> {
    match dict.term_parts(id) {
        dict::TermParts::Lit { value, datatype, lang: None } => Temporal::of_lit(value, datatype),
        _ => None,
    }
}

/// One dense temporal-cache cell: the flag (see [`temp_flag`]; 0 = not temporal) and
/// the precomputed instant, co-located so a probe is one cache-line touch.
#[derive(Clone, Copy)]
struct TempCell {
    instant: f64,
    flag: u8,
}

/// The temporal-value cache cells for a dictionary (see [`TempData::Owned`]).
/// Parallel when the `parallel` feature is on.
fn temporals_of(dict: &Dict) -> Vec<TempCell> {
    let n = dict.len();
    let cell = |i: usize| -> TempCell {
        match temporal_of_id(dict, i as Id + 1) {
            Some(t) => TempCell { instant: t.instant, flag: temp_flag(t) },
            None => TempCell { instant: f64::NAN, flag: 0 },
        }
    };
    #[cfg(feature = "parallel")]
    {
        use rayon::prelude::*;
        (0..n).into_par_iter().map(cell).collect()
    }
    #[cfg(not(feature = "parallel"))]
    {
        (0..n).map(cell).collect()
    }
}

/// Writes the temporal-value cache to disk (`n` little-endian f64 instants, then `n`
/// flag bytes) so it can be memory-mapped on open instead of recomputed.
#[cfg(feature = "mmap")]
fn write_temporals(path: &std::path::Path, flags: &[u8], instants: &[f64]) -> std::io::Result<()> {
    use std::io::Write;
    let mut f = std::io::BufWriter::new(std::fs::File::create(path)?);
    // SAFETY: reinterpret the contiguous f64 column as bytes for writing.
    let ibytes = unsafe { std::slice::from_raw_parts(instants.as_ptr().cast::<u8>(), std::mem::size_of_val(instants)) };
    f.write_all(ibytes)?;
    f.write_all(flags)?;
    f.flush()
}

/// The f64 value of a term if it is a numeric XSD literal, else NaN.
fn numeric_of(term: &Term) -> f64 {
    match term {
        Term::Literal(l) if is_numeric_dt(l) => l.value().parse::<f64>().unwrap_or(f64::NAN),
        _ => f64::NAN,
    }
}

/// True for `xsd:integer` and its derived integer subtypes (NOT decimal/double/float) —
/// the datatypes whose values are exact integers (parseable as `i128`).
pub fn is_integer_datatype(dt: &str) -> bool {
    dt == xsd::INTEGER.as_str()
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

fn is_numeric_dt(l: &Literal) -> bool {
    is_numeric_datatype_str(l.datatype().as_str())
}

/// `is_numeric_dt` by datatype IRI string — shared with the spilled-dict build, which
/// computes the numeric cache from borrowed term components without an `oxrdf::Term`.
/// (A language-tagged literal's datatype is `rdf:langString`, never numeric.)
pub(crate) fn is_numeric_datatype_str(dt: &str) -> bool {
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
        let (dict, triples) = Self::parse_to_triples(text, format)?;
        Ok(Self::from_parts(dict, triples))
    }

    /// Parses an RDF document into its dictionary + interned triples WITHOUT building the
    /// indexes. The seam that opt-in reasoning hooks into: a caller (e.g. the CLI, which can
    /// depend on `sparq-reason`) parses, materializes the entailed triples, then calls
    /// [`from_parts`](Self::from_parts) — keeping all reasoning out of the core engine.
    pub fn parse_to_triples(text: &str, format: &str) -> Result<(Dict, Vec<[Id; 3]>), String> {
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
                // N-Triples is one statement per line, so the input can be split at
                // newline boundaries and parsed + interned in parallel (each thread
                // builds a partial dictionary, then the partials are merged).
                #[cfg(feature = "parallel")]
                {
                    return parse_ntriples_parallel(bytes);
                }
                #[cfg(not(feature = "parallel"))]
                {
                    let mut d = Dict::new();
                    let t = nt::parse_chunk(bytes, &mut d)?;
                    return Ok((d, t));
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
                // Turtle is not line-oriented, but it splits at top-level statement
                // terminators (with the @prefix preamble shared into each chunk), parsed in
                // parallel with a serial fallback on any mis-split — see parse_turtle_parallel.
                #[cfg(feature = "parallel")]
                {
                    return parse_turtle_parallel(bytes);
                }
                #[cfg(not(feature = "parallel"))]
                {
                    for t in TurtleParser::new().for_slice(bytes) {
                        let t = t.map_err(|e| e.to_string())?;
                        push_triple!(&t.subject, &t.predicate, &t.object);
                    }
                }
            }
        }

        Ok((dict, triples))
    }

    /// Like [`load_str`](Self::load_str) but resolves relative IRIs in the document
    /// against `base` — the entry point for documents that carry no `@base` of their
    /// own (e.g. SHACL shapes graphs addressed by their location, W3C test-suite
    /// manifests). `format`: as [`load_str`]; the line-based formats ("ntriples" /
    /// "nquads") only allow absolute IRIs, so `base` has no effect on them.
    pub fn load_str_with_base(text: &str, format: &str, base: &str) -> Result<Graph, String> {
        let (dict, triples) = Self::parse_to_triples_with_base(text, format, base)?;
        Ok(Self::from_parts(dict, triples))
    }

    /// [`parse_to_triples`](Self::parse_to_triples) with a base IRI for resolving the
    /// document's relative IRIs (Turtle / TriG; the line-based formats have no
    /// relative IRIs and ignore `base`). Parses serially — base-IRI documents are
    /// typically small (shapes graphs, manifests), so the parallel chunked path is
    /// not worth its base-rewriting complexity here.
    pub fn parse_to_triples_with_base(
        text: &str,
        format: &str,
        base: &str,
    ) -> Result<(Dict, Vec<[Id; 3]>), String> {
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
            "ntriples" | "n-triples" | "nquads" | "n-quads" => {
                return Self::parse_to_triples(text, format);
            }
            "trig" | "application/trig" => {
                let parser = TriGParser::new()
                    .with_base_iri(base)
                    .map_err(|e| format!("invalid base IRI {base:?}: {e}"))?;
                for q in parser.for_slice(bytes) {
                    let q = q.map_err(|e| e.to_string())?;
                    push_triple!(&q.subject, &q.predicate, &q.object);
                }
            }
            _ => {
                let parser = TurtleParser::new()
                    .with_base_iri(base)
                    .map_err(|e| format!("invalid base IRI {base:?}: {e}"))?;
                for t in parser.for_slice(bytes) {
                    let t = t.map_err(|e| e.to_string())?;
                    push_triple!(&t.subject, &t.predicate, &t.object);
                }
            }
        }

        Ok((dict, triples))
    }

    /// Load an RDF DATASET (N-Quads / TriG) preserving NAMED GRAPHS as separate sub-graphs, so the
    /// engine can evaluate `GRAPH <iri> { … }` / `GRAPH ?g { … }`. Default-graph triples form the
    /// main graph; each named graph becomes a [`named`](Self::named) entry. Formats without named
    /// graphs defer to [`load_str`](Self::load_str). In-memory only (the mmap path is triple-only).
    pub fn load_dataset(text: &str, format: &str) -> Result<Graph, String> {
        use oxrdf::GraphName;
        use std::collections::HashMap;
        if !matches!(format, "nquads" | "n-quads" | "trig" | "application/trig") {
            return Self::load_str(text, format);
        }
        let bytes = text.as_bytes();
        let mut groups: HashMap<Option<Term>, Vec<[Term; 3]>> = HashMap::new();
        macro_rules! group {
            ($parser:expr) => {
                for q in $parser.for_slice(bytes) {
                    let q = q.map_err(|e| e.to_string())?;
                    let g = match q.graph_name {
                        GraphName::DefaultGraph => None,
                        GraphName::NamedNode(n) => Some(Term::NamedNode(n)),
                        GraphName::BlankNode(b) => Some(Term::BlankNode(b)),
                    };
                    groups
                        .entry(g)
                        .or_default()
                        .push([subject_term(&q.subject), Term::NamedNode(q.predicate), q.object]);
                }
            };
        }
        match format {
            "nquads" | "n-quads" => group!(NQuadsParser::new()),
            _ => group!(TriGParser::new()),
        }
        let build_terms = |triples: &[[Term; 3]]| -> Graph {
            let mut dict = Dict::new();
            let ids: Vec<[Id; 3]> =
                triples.iter().map(|[s, p, o]| [dict.intern(s), dict.intern(p), dict.intern(o)]).collect();
            Self::build(dict, ids)
        };
        let default = groups.remove(&None).unwrap_or_default();
        let mut g = build_terms(&default);
        for (name, triples) in groups {
            if let Some(name) = name {
                g.named.push((name, build_terms(&triples)));
            }
        }
        Ok(g)
    }

    /// Builds a graph from an already-interned dictionary + triple set (e.g. after opt-in
    /// reasoning materialized additional triples). Public counterpart of the internal
    /// [`build`](Self::build).
    pub fn from_parts(dict: Dict, triples: Vec<[Id; 3]>) -> Graph {
        Self::build(dict, triples)
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

    /// Streaming PARALLEL load: reads the (already-decompressed) `reader` in newline-aligned
    /// ~32 MiB blocks and parses each block in parallel, so the full decompressed document is
    /// NEVER materialised in memory — only a few blocks in flight plus the growing
    /// dictionary/triples. (The store itself must fit in RAM; this removes the redundant
    /// full-text copy a read-to-string load would hold alongside it.) For N-Triples; other
    /// formats defer to the serial streaming [`load_reader`].
    ///
    /// The read/decompress runs PIPELINED on its own thread (same 3-stage design as the
    /// external build's `build_external_ntriples_parallel`): stage 1 fills full 32 MiB
    /// blocks from the reader (looping over short `read()`s — a gzip/zstd decompressor
    /// returns 0.4–1.6 MB per call, and flushing a parse+merge round per `read()` was the
    /// measured 3.5–5× streaming-ingest slowdown), stage 2 parses each block on the rayon
    /// pool, stage 3 (the caller's thread) merges dictionaries — so decode overlaps
    /// parse+merge and streaming ingest approaches max(decode, parse).
    #[cfg(feature = "parallel")]
    pub fn load_reader_parallel<R: std::io::Read + Send>(reader: R, format: &str) -> Result<Graph, String> {
        if !matches!(format, "ntriples" | "n-triples") {
            return Self::load_reader(reader, format);
        }
        Self::load_ntriples_pipelined(reader, 32 << 20)
    }

    /// The pipelined N-Triples loader behind [`load_reader_parallel`], with the block size
    /// as a parameter so tests can exercise the multi-block / boundary-straddling paths
    /// cheaply (production always passes 32 MiB).
    #[cfg(feature = "parallel")]
    fn load_ntriples_pipelined<R: std::io::Read + Send>(reader: R, block_size: usize) -> Result<Graph, String> {
        use std::sync::mpsc::sync_channel;
        // ≥2 rayon threads: ONE sharded dict spans all blocks, so the per-block dict merge
        // — the measured serial `merge_remap` that capped load scaling at ~1.8× on 4
        // identical cores — runs parallel across hash-shards; triples carry temporary
        // sharded ids until one parallel final remap. On one thread the proven serial
        // merge is kept (no sharding overhead).
        let sharded = rayon::current_num_threads() > 1;
        let mut sd = dict::ShardedDict::new(if sharded { default_shards() } else { 1 });
        let mut global = Dict::new();
        let mut all: Vec<[Id; 3]> = Vec::new();
        // Raw blocks flow read-thread -> parse; parsed partials flow parse-thread -> merge
        // (this thread). Bounds of 1 keep at most ~3 blocks (+1 block's partials) in
        // flight while letting decode, parse and merge overlap.
        let (tx, rx) = sync_channel::<Vec<u8>>(1);
        type Partials = Vec<(Dict, Vec<[Id; 3]>)>;
        let (ptx, prx) = sync_channel::<Partials>(1);
        std::thread::scope(|scope| -> Result<(), String> {
            // Stage 1 — read (the caller's decompressor) on its own thread, emitting
            // newline-aligned FULL blocks: loop `read()` until the block is full or EOF.
            let producer = scope.spawn(move || -> Result<(), String> {
                let mut reader = reader;
                let mut readbuf = vec![0u8; block_size];
                let mut carry: Vec<u8> = Vec::new();
                loop {
                    let mut filled = 0;
                    while filled < block_size {
                        let n = reader.read(&mut readbuf[filled..]).map_err(|e| e.to_string())?;
                        if n == 0 {
                            break;
                        }
                        filled += n;
                    }
                    if filled == 0 {
                        // EOF: a final line without a trailing newline lives in `carry`.
                        if !carry.is_empty() {
                            let _ = tx.send(std::mem::take(&mut carry));
                        }
                        return Ok(());
                    }
                    // Emit `carry + readbuf[..filled]` up to the last newline; carry the
                    // remainder (a partial line split across the block boundary) forward.
                    let mut block = std::mem::take(&mut carry);
                    block.extend_from_slice(&readbuf[..filled]);
                    let cut = block.iter().rposition(|&b| b == b'\n').map_or(0, |p| p + 1);
                    carry = block[cut..].to_vec();
                    block.truncate(cut);
                    if !block.is_empty() && tx.send(block).is_err() {
                        return Ok(()); // a downstream stage errored and dropped the receiver
                    }
                }
            });
            // Stage 2 — parse+intern each block in parallel (per-chunk local dicts, no
            // shared state), forwarding the partials to the merge stage.
            let parser = scope.spawn(move || -> Result<(), String> {
                for block in rx {
                    let partials = parse_block(&block)?;
                    if ptx.send(partials).is_err() {
                        return Ok(()); // the merge stage errored and dropped the receiver
                    }
                }
                Ok(())
            });
            // Stage 3 (this thread) — dict merge. Blocks arrive in document order, so id
            // assignment matches the non-pipelined load (deterministic).
            for partials in prx {
                if sharded {
                    sharded_extend(&mut sd, &partials, &mut all);
                } else {
                    for (pd, pt) in partials {
                        let remap = global.merge_remap(&pd);
                        remap_extend(&mut all, pt, &remap);
                    }
                }
            }
            // Join parse first (it feeds stage 3 — surface a parse error), then the producer.
            parser.join().map_err(|_| "parse thread panicked".to_string())??;
            producer.join().map_err(|_| "read thread panicked".to_string())?
        })?;
        if sharded {
            let (dict, ids) = finish_sharded(sd, all);
            return Ok(Self::build(dict, ids));
        }
        Ok(Self::build(global, all))
    }

    /// Builds the store + numeric cache from interned triples (shared by the
    /// string and streaming loaders).
    fn build(dict: Dict, triples: Vec<[Id; 3]>) -> Graph {
        let store = TripleStore::from_triples(triples);
        let numerics = NumData::Owned(numerics_of(&dict));
        // Unlike the numerics cache (dense f64 = 8 B/term), the dense temporal cells
        // are 16 B/term — go sparse straight away when temporals are rare (usually:
        // none at all -> an empty map, zero memory), keeping the load-time memory
        // metric flat for non-temporal datasets. Temporal-heavy data stays dense.
        let temporals = TempData::Owned(temporals_of(&dict)).into_sparse_if_worthwhile();
        Graph {
            dict,
            store,
            numerics,
            temporals,
            named: Vec::new(),
            #[cfg(feature = "mmap")]
            wal: None,
        }
    }

    /// Like [`load_str`](Self::load_str) but stores the permutation indexes
    /// BLOCK-COMPRESSED (~4-6 B/triple vs 12) — the memory-bound build for the browser,
    /// trading a bounded per-scan decode for ~2.5x more triples per byte of RAM. Query
    /// results are identical to the raw build.
    pub fn load_str_compressed(text: &str, format: &str) -> Result<Graph, String> {
        let g = Self::load_str(text, format)?;
        Ok(g.into_compressed())
    }

    /// Re-encodes into the memory-bound storage mode: the permutations BLOCK-COMPRESSED and
    /// the dictionary's id→term storage compacted to a single BLOB (no per-term `Box<str>`).
    /// Keeps the numeric cache and term ids. The browser/RAM-constrained path; identical
    /// query results, a small per-scan decode.
    pub fn into_compressed(self) -> Graph {
        let triples: Vec<[Id; 3]> = {
            let scan = self.store.scan(&[None, None, None]);
            scan.rows.iter().map(|r| scan.to_spo(r)).collect()
        };
        Graph {
            store: TripleStore::from_triples_compressed(triples),
            dict: self.dict.into_blob(),
            // The numeric cache is mostly (often entirely) NaN — keep only the real
            // numeric literals when sparse, freeing the dense f64-per-term Vec.
            numerics: self.numerics.into_sparse_if_worthwhile(),
            temporals: self.temporals.into_sparse_if_worthwhile(),
            named: self.named,
            #[cfg(feature = "mmap")]
            wal: self.wal,
        }
    }

    /// Persists the graph to `dir` (the permutation indexes + the dictionary) so it can
    /// later be QUERIED with the indexes MEMORY-MAPPED via [`open`](Self::open) — the
    /// out-of-core path for datasets larger than RAM.
    #[cfg(feature = "mmap")]
    pub fn save(&self, dir: &std::path::Path) -> std::io::Result<()> {
        std::fs::create_dir_all(dir)?;
        self.store.save(dir)?; // folds any delta-overlay into the persisted permutations
        self.dict.save_mmap(dir)?; // includes appended (delta-overlay) terms
        write_numerics(&dir.join("numerics.bin"), &self.dense_numerics())?;
        let (tf, ti) = self.dense_temporals();
        write_temporals(&dir.join("temporals.bin"), &tf, &ti)
    }

    /// Like [`save`](Self::save) but persists the permutation indexes BLOCK-COMPRESSED
    /// (~3-5x smaller on disk; the dictionary/numerics files are unchanged). [`open`]
    /// auto-detects the format per file, serving compressed perms by lazy block-wise
    /// decode off the mapped file; call [`decompress_indexes`](Self::decompress_indexes)
    /// after open to trade RAM for exactly-raw query speed instead.
    #[cfg(feature = "mmap")]
    pub fn save_compressed(&self, dir: &std::path::Path) -> std::io::Result<()> {
        std::fs::create_dir_all(dir)?;
        self.store.save_compressed(dir)?; // folds any delta-overlay, like `save`
        self.dict.save_mmap(dir)?;
        write_numerics(&dir.join("numerics.bin"), &self.dense_numerics())?;
        let (tf, ti) = self.dense_temporals();
        write_temporals(&dir.join("temporals.bin"), &tf, &ti)
    }

    /// Decodes any block-compressed permutation indexes into raw RAM (the load-time
    /// decompression mode for a compressed dir: one up-front decode, then scans are
    /// zero-cost slice borrows, identical to a raw store). No-op on raw/mapped indexes.
    pub fn decompress_indexes(&mut self) {
        self.store.decompress_to_ram();
    }

    /// The full dense numeric cache (one f64 per dictionary term) for persisting: the
    /// owned/mmap'd dense backing extended over any APPENDED terms, or recomputed when
    /// no dense backing covers the dictionary (the sparse browser mode).
    #[cfg(feature = "mmap")]
    fn dense_numerics(&self) -> std::borrow::Cow<'_, [f64]> {
        let n = self.dict.len();
        match &self.numerics {
            NumData::Owned(v) if v.len() == n => std::borrow::Cow::Borrowed(v),
            NumData::Mapped(_, extra) => {
                let dense = self.numerics.as_slice();
                if dense.len() == n {
                    return std::borrow::Cow::Borrowed(dense);
                }
                let mut v = dense.to_vec();
                v.resize(n, f64::NAN);
                for (&id, &val) in extra {
                    v[(id - 1) as usize] = val;
                }
                std::borrow::Cow::Owned(v)
            }
            _ => std::borrow::Cow::Owned(numerics_of(&self.dict)),
        }
    }

    /// The full dense temporal cache columns (flag byte + f64 instant per dictionary
    /// term) for persisting — the temporal twin of [`dense_numerics`](Self::dense_numerics).
    #[cfg(feature = "mmap")]
    fn dense_temporals(&self) -> (Vec<u8>, Vec<f64>) {
        let n = self.dict.len();
        let split = |cells: &[TempCell]| -> (Vec<u8>, Vec<f64>) {
            (cells.iter().map(|c| c.flag).collect(), cells.iter().map(|c| c.instant).collect())
        };
        match &self.temporals {
            TempData::Owned(cells) if cells.len() == n => split(cells),
            TempData::Mapped(m, extra) => {
                let len = TempData::mapped_len(m);
                // SAFETY: temporals.bin starts with `len` f64 (page-aligned mmap base).
                let minst = unsafe { std::slice::from_raw_parts(m.as_ptr().cast::<f64>(), len) };
                let mut flags = m[len * 8..len * 9].to_vec();
                let mut instants = minst.to_vec();
                flags.resize(n, 0);
                instants.resize(n, f64::NAN);
                for (&id, &t) in extra {
                    flags[(id - 1) as usize] = temp_flag(t);
                    instants[(id - 1) as usize] = t.instant;
                }
                (flags, instants)
            }
            _ => split(&temporals_of(&self.dict)),
        }
    }

    /// Opens a graph saved by [`save`](Self::save) with its permutation indexes AND
    /// numeric-value cache MEMORY-MAPPED (paged in on demand) — so a large out-of-core
    /// dataset opens near-instantly without re-parsing every term, and the cache stays
    /// off the heap. The dictionary is loaded into RAM; the big indexes stay on disk. If
    /// `numerics.bin` is absent or stale (a graph saved before this cache existed), the
    /// cache is recomputed, preserving backward compatibility.
    #[cfg(feature = "mmap")]
    pub fn open(dir: &std::path::Path) -> std::io::Result<Graph> {
        // [OPUS-4.8] (review 1593) Finish or roll back any compaction directory swap that a
        // crash interrupted, so `dir` is always present (never lost in the rename window).
        recover_compaction(dir)?;
        let store = TripleStore::open(dir)?;
        // [OPUS-4.8] (review 1325, sub-finding 2) Prefer the mmap dictionary format
        // (`dict-meta.bin`, written by `save_mmap`); fall back to the LEGACY single-file
        // `dict.bin` (written by the older `Dict::save`) so graph directories saved before
        // the mmap dictionary format still open. The legacy dict loads fully into RAM (no
        // mmap), which is the only difference — every term still round-trips.
        let dict = if dir.join("dict-meta.bin").exists() {
            Dict::open_mmap(dir)?
        } else {
            let legacy = dir.join("dict.bin");
            if legacy.exists() {
                Dict::open(&legacy)?
            } else {
                // Neither format present — surface the original mmap error (NotFound on
                // dict-meta.bin), matching the previous behaviour for a corrupt directory.
                Dict::open_mmap(dir)?
            }
        };
        let np = dir.join("numerics.bin");
        let numerics = match std::fs::File::open(&np) {
            Ok(f) if f.metadata()?.len() as usize == dict.len() * std::mem::size_of::<f64>() => {
                // SAFETY: the file is owned by this graph for its lifetime and not mutated.
                NumData::Mapped(unsafe { memmap2::Mmap::map(&f)? }, rustc_hash::FxHashMap::default())
            }
            _ => NumData::Owned(numerics_of(&dict)),
        };
        let tp = dir.join("temporals.bin");
        let temporals = match std::fs::File::open(&tp) {
            Ok(f) if f.metadata()?.len() as usize == dict.len() * 9 => {
                // SAFETY: the file is owned by this graph for its lifetime and not mutated.
                TempData::Mapped(unsafe { memmap2::Mmap::map(&f)? }, rustc_hash::FxHashMap::default())
            }
            // Absent or stale (a graph saved before this cache existed): recompute —
            // backward compatible, like the numerics cache.
            _ => TempData::Owned(temporals_of(&dict)),
        };
        let mut g = Graph { dict, store, numerics, temporals, named: Vec::new(), wal: None };
        // Replay any write-ahead log into the delta-overlay (recovery after a crash or a
        // plain not-yet-compacted close), stopping cleanly at the first torn record and
        // truncating the log there — then keep the log open for further appends.
        for (insert, t) in Wal::replay(dir)? {
            if insert {
                g.apply_delta_mem(&[t], &[]);
            } else {
                g.apply_delta_mem(&[], &[t]);
            }
        }
        g.wal = Some(Wal::open(dir)?);
        Ok(g)
    }

    /// EXTERNAL-MEMORY build: streams an RDF document and writes the on-disk permutation
    /// indexes + dictionary directly, sorting the triples through disk-backed runs so the
    /// dataset's indexes can be CONSTRUCTED without ever holding them all in RAM. Only one
    /// `chunk`-sized buffer of triples (plus the growing dictionary) is resident at a time;
    /// the rest lives in sorted run files that are k-way merged. The result is identical to
    /// `save()` of an in-memory `load`, but bounded-memory — the billion-scale ingest path.
    /// Open it with [`open`](Self::open).
    ///
    /// `chunk` is the number of triples per in-memory run (e.g. 8_000_000 ≈ 96 MB of ids).
    #[cfg(feature = "mmap")]
    pub fn build_external<R: std::io::Read + Send>(
        reader: R,
        format: &str,
        dir: &std::path::Path,
        chunk: usize,
    ) -> Result<(), String> {
        // The SPILLED dictionary (`dict-spill` feature, env-gated via SPARQ_DICT_SPILL):
        // bounded-RSS builds for dictionaries larger than RAM. Output is byte-identical
        // to the (default) sharded path; N-Triples only, like the sharded path.
        #[cfg(feature = "dict-spill")]
        if matches!(format, "ntriples" | "n-triples") {
            if let Some(cfg) = dictspill::SpillConfig::from_env() {
                return Self::build_external_spill(reader, format, dir, chunk, &cfg);
            }
        }
        // The SHARDED-dict N-Triples ingest (parallel dict consolidation) is the DEFAULT
        // when it can run (parallel build, >1 rayon thread); `SPARQ_SHARDED_DICT=0` (or
        // `off`) opts out to the serial-merge path, `=1` keeps forcing it on (its id
        // assignment needs ≥2 threads' shard count to be meaningfully parallel, but it is
        // correct on any). The on-disk FORMAT is identical either way (same writers, same
        // record layouts — only term-id ASSIGNMENT differs), so no format-version bump:
        // stores built by either path open interchangeably.
        #[cfg(feature = "parallel")]
        let sharded = match std::env::var("SPARQ_SHARDED_DICT").as_deref() {
            Ok("0") | Ok("off") => false,
            Ok(_) => true,
            Err(_) => rayon::current_num_threads() > 1,
        };
        #[cfg(not(feature = "parallel"))]
        let sharded = false;
        Self::build_external_opts(reader, format, dir, chunk, sharded)
    }

    /// [`build_external`](Self::build_external) with the sharded-dict choice EXPLICIT
    /// (no env lookup) — for tests and embedders that need a specific path.
    #[cfg(feature = "mmap")]
    pub fn build_external_opts<R: std::io::Read + Send>(
        reader: R,
        format: &str,
        dir: &std::path::Path,
        chunk: usize,
        sharded: bool,
    ) -> Result<(), String> {
        use store::{Perm, BUILT};
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        let tmp = dir.join("tmp");
        std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;

        let mut dict = Dict::new();
        let mut buf: Vec<[Id; 3]> = Vec::with_capacity(chunk);
        let mut runs: Vec<std::path::PathBuf> = Vec::new();
        #[cfg(all(feature = "mmap", feature = "parallel"))]
        let _t_build = std::time::Instant::now();
        #[cfg(all(feature = "mmap", feature = "parallel"))]
        build_timing::reset();

        // Stream-parse + intern, spilling SPO-sorted runs to disk whenever the buffer fills.
        macro_rules! push_triple {
            ($s:expr, $p:expr, $o:expr) => {{
                let s = dict.intern(&subject_term($s));
                let p = dict.intern(&Term::NamedNode($p.clone()));
                let o = dict.intern($o);
                buf.push([s, p, o]);
                if buf.len() >= chunk {
                    extsort::spill_run(&mut buf, &mut runs, &tmp).map_err(|e| e.to_string())?;
                }
            }};
        }

        // PARALLEL sharded-dict ingest for N-Triples (the default — see `build_external`):
        // interns through N hash-shards (no serial `merge_remap`), spilling temporary
        // sharded ids that an order-preserving remap turns into final dense ids after the
        // SPO sort. When opted out (or for other formats / non-parallel builds), the
        // serial-merge path runs.
        #[cfg(all(feature = "mmap", feature = "parallel"))]
        let sharded = matches!(format, "ntriples" | "n-triples") && sharded;
        #[cfg(not(all(feature = "mmap", feature = "parallel")))]
        let sharded = {
            let _ = sharded;
            false
        };
        let mut sharded_remap: Option<(Vec<u64>, u32)> = None;

        if sharded {
            #[cfg(all(feature = "mmap", feature = "parallel"))]
            {
                let mut sd = dict::ShardedDict::new(default_shards());
                build_external_ntriples_sharded(reader, &mut sd, &mut buf, &mut runs, &tmp, chunk)?;
                let t_cons = std::time::Instant::now();
                let (merged, base, stride) = sd.into_merged();
                if build_timing::enabled() {
                    eprintln!("[build-timing] dict consolidate (into_merged): {:.2}s", t_cons.elapsed().as_secs_f64());
                }
                dict = merged;
                sharded_remap = Some((base, stride));
            }
        } else {
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
                    // N-Triples is the billion-scale bulk format: parse it with the custom
                    // byte parser over PARALLEL buffers (per-buffer partial dicts merged into
                    // the running dict), the user-requested "parallelise parsing of the file".
                    // Still bounded-memory: one ~64 MiB buffer + its partials at a time.
                    #[cfg(feature = "parallel")]
                    build_external_ntriples_parallel(reader, &mut dict, &mut buf, &mut runs, &tmp, chunk)?;
                    #[cfg(not(feature = "parallel"))]
                    for t in NTriplesParser::new().for_reader(reader) {
                        let t = t.map_err(|e| e.to_string())?;
                        push_triple!(&t.subject, &t.predicate, &t.object);
                    }
                }
            }
        }
        extsort::spill_run(&mut buf, &mut runs, &tmp).map_err(|e| e.to_string())?;
        buf.shrink_to_fit();
        #[cfg(all(feature = "mmap", feature = "parallel"))]
        if build_timing::enabled() {
            build_timing::report("parse+intern+spill done", _t_build.elapsed().as_secs_f64());
        }

        // Merge the SPO runs into the SPO permutation file (deduplicating).
        let spo_path = dir.join(format!("perm{}.bin", Perm::Spo as usize));
        extsort::kway_merge(&runs, &spo_path).map_err(|e| e.to_string())?;
        #[cfg(all(feature = "mmap", feature = "parallel"))]
        if build_timing::enabled() {
            eprintln!("[build-timing] kway_merge SPO done | {:.2}s wall to here", _t_build.elapsed().as_secs_f64());
        }
        for r in &runs {
            std::fs::remove_file(r).ok();
        }
        // Sharded build: the SPO perm holds TEMPORARY sharded ids — remap them to final dense
        // ids in place (order-preserving, so it stays sorted+deduped) BEFORE the sibling sorts
        // read it, so the permutations and the merged dictionary agree on ids.
        #[cfg(all(feature = "mmap", feature = "parallel"))]
        if let Some((base, stride)) = &sharded_remap {
            remap_perm_file(&spo_path, base, *stride)?;
        }

        // Build the other BUILT permutations by external-sorting the SPO file into each
        // order (re-reading it memory-mapped, so this stays bounded-memory too).
        let (map, n) = extsort::map_perm(&spo_path).map_err(|e| e.to_string())?;
        // SAFETY: perm0 is a whole number of [u32;3] rows written above; map outlives the loop.
        let spo: &[[Id; 3]] =
            unsafe { std::slice::from_raw_parts(map.as_ptr().cast::<[Id; 3]>(), n) };
        let siblings: Vec<Perm> = BUILT.iter().copied().filter(|&p| p != Perm::Spo).collect();
        let sib_sort = |perm: Perm, sub: &std::path::Path, per: usize| -> Result<(), String> {
            std::fs::create_dir_all(sub).map_err(|e| e.to_string())?;
            let out = dir.join(format!("perm{}.bin", perm as usize));
            extsort::external_sort(spo.iter().copied(), perm.order(), &out, sub, per).map_err(|e| e.to_string())
        };
        // The sibling sorts are independent — run them CONCURRENTLY (each in its own tmp
        // subdir, so run files don't collide), sharing the chunk budget so total resident
        // memory stays ~`chunk`. The shared SPO mmap is read-only (paged, no extra RAM).
        // Persisting the DICTIONARY (save_mmap: the term blob + sorted-hash index) and the
        // numeric cache only needs `dict` — it is INDEPENDENT of the permutation sorts. Run
        // it CONCURRENTLY with the sibling sorts (on its own thread) so the multi-hundred-MB
        // dict write is hidden under the sort time instead of being a serial tail. Output is
        // byte-identical (same files); only the wall-clock ordering overlaps.
        std::thread::scope(|scope| -> Result<(), String> {
            let dict_ref = &dict;
            let finalize = scope.spawn(move || -> Result<(), String> {
                dict_ref.save_mmap(dir).map_err(|e| e.to_string())?;
                write_numerics(&dir.join("numerics.bin"), &numerics_of(dict_ref)).map_err(|e| e.to_string())?;
                let (tf, ti) = {
                    let cells = temporals_of(dict_ref);
                    (cells.iter().map(|c| c.flag).collect::<Vec<u8>>(), cells.iter().map(|c| c.instant).collect::<Vec<f64>>())
                };
                write_temporals(&dir.join("temporals.bin"), &tf, &ti).map_err(|e| e.to_string())?;
                Ok(())
            });
            #[cfg(feature = "parallel")]
            {
                use rayon::prelude::*;
                let per = (chunk / siblings.len().max(1)).max(1 << 16);
                siblings
                    .par_iter()
                    .try_for_each(|&perm| sib_sort(perm, &tmp.join(format!("p{}", perm as usize)), per))?;
            }
            #[cfg(not(feature = "parallel"))]
            for &perm in &siblings {
                sib_sort(perm, &tmp, chunk)?;
            }
            finalize.join().map_err(|_| "dict-finalize thread panicked".to_string())??;
            Ok(())
        })?;
        drop(map);
        #[cfg(all(feature = "mmap", feature = "parallel"))]
        if build_timing::enabled() {
            eprintln!("[build-timing] sibling sorts ∥ dict-save done | {:.2}s wall to here", _t_build.elapsed().as_secs_f64());
        }

        // Empty files for the unbuilt permutations so `open` finds all six slots.
        for i in 0..6 {
            let p = dir.join(format!("perm{i}.bin"));
            if !p.exists() {
                std::fs::File::create(&p).map_err(|e| e.to_string())?;
            }
        }
        // Compute per-predicate stats once (a one-time POS/PSO scan) and persist them so
        // query-open never re-scans those indexes — keeping out-of-core open fast + small.
        let store = TripleStore::open(dir).map_err(|e| e.to_string())?;
        store.save_pred_stats(dir).map_err(|e| e.to_string())?;
        std::fs::remove_dir_all(&tmp).ok();
        Ok(())
    }

    /// External-memory build with the SPILLED term dictionary (`dict-spill` feature):
    /// peak RSS is bounded by `cfg.mem_budget` (dictionary included) instead of growing
    /// with the distinct-term count — terms spill to disk and are externally
    /// deduplicated/ranked into EXACTLY the ids the (default) sharded in-RAM
    /// consolidation assigns, so every output file is byte-identical to
    /// [`build_external`](Self::build_external)'s. N-Triples only (the same RDF-star
    /// restriction as the sharded path). Design: research/external-dictionary.md.
    #[cfg(feature = "dict-spill")]
    pub fn build_external_spill<R: std::io::Read + Send>(
        reader: R,
        format: &str,
        dir: &std::path::Path,
        chunk: usize,
        cfg: &dictspill::SpillConfig,
    ) -> Result<(), String> {
        use store::{Perm, BUILT};
        if !matches!(format, "ntriples" | "n-triples") {
            return Err("the dict-spill external build supports N-Triples only".to_string());
        }
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        let tmp = dir.join("tmp");
        std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
        dictspill::ensure_disk(&tmp, cfg.disk_floor)?;
        let _t_build = std::time::Instant::now();
        build_timing::reset();

        // Stages 1-3: decompress + parallel parse + bounded-cache resolve + stage.
        let mut interner =
            dictspill::SpillInterner::new(default_shards(), &tmp, cfg).map_err(|e| e.to_string())?;
        build_external_ntriples_dictspill(reader, &mut interner)?;
        if build_timing::enabled() {
            build_timing::report("parse+route+stage done", _t_build.elapsed().as_secs_f64());
        }

        // Phases 2-4: external dedup/rank. The dictionary files and the numeric/temporal
        // caches are STREAM-written in final-id order here, so no dict-finalize thread
        // (and no resident dictionary) exists later.
        let t_cons = std::time::Instant::now();
        let plan = dictspill::consolidate(interner, dir, &tmp, cfg)?;
        if build_timing::enabled() {
            eprintln!("[build-timing] dict consolidate (spilled): {:.2}s", t_cons.elapsed().as_secs_f64());
        }

        // Phase 5: remap the staged triples to final dense ids, spilling SPO-sorted runs.
        let mut buf: Vec<[Id; 3]> = Vec::with_capacity(chunk.min(1 << 24));
        let mut runs: Vec<std::path::PathBuf> = Vec::new();
        dictspill::remap_staged(plan, &mut buf, &mut runs, &tmp, chunk)?;
        extsort::spill_run(&mut buf, &mut runs, &tmp).map_err(|e| e.to_string())?;
        drop(buf); // free the chunk buffer before the k-way merge + sibling sorts

        // Merge the SPO runs (ids are FINAL already — no `remap_perm_file` pass needed).
        let spo_path = dir.join(format!("perm{}.bin", Perm::Spo as usize));
        extsort::kway_merge(&runs, &spo_path).map_err(|e| e.to_string())?;
        if build_timing::enabled() {
            eprintln!("[build-timing] kway_merge SPO done | {:.2}s wall to here", _t_build.elapsed().as_secs_f64());
        }
        for r in &runs {
            std::fs::remove_file(r).ok();
        }

        // Sibling permutations: the same concurrent external sorts as
        // `build_external_opts` (minus its dict-finalize thread).
        let (map, n) = extsort::map_perm(&spo_path).map_err(|e| e.to_string())?;
        // SAFETY: perm0 is a whole number of [u32;3] rows written above; map outlives the loop.
        let spo: &[[Id; 3]] =
            unsafe { std::slice::from_raw_parts(map.as_ptr().cast::<[Id; 3]>(), n) };
        let siblings: Vec<Perm> = BUILT.iter().copied().filter(|&p| p != Perm::Spo).collect();
        let sib_sort = |perm: Perm, sub: &std::path::Path, per: usize| -> Result<(), String> {
            std::fs::create_dir_all(sub).map_err(|e| e.to_string())?;
            let out = dir.join(format!("perm{}.bin", perm as usize));
            extsort::external_sort(spo.iter().copied(), perm.order(), &out, sub, per).map_err(|e| e.to_string())
        };
        {
            use rayon::prelude::*;
            let per = (chunk / siblings.len().max(1)).max(1 << 16);
            siblings
                .par_iter()
                .try_for_each(|&perm| sib_sort(perm, &tmp.join(format!("p{}", perm as usize)), per))?;
        }
        drop(map);
        if build_timing::enabled() {
            eprintln!("[build-timing] sibling sorts done | {:.2}s wall to here", _t_build.elapsed().as_secs_f64());
        }

        // Empty files for the unbuilt permutations so `open` finds all six slots.
        for i in 0..6 {
            let p = dir.join(format!("perm{i}.bin"));
            if !p.exists() {
                std::fs::File::create(&p).map_err(|e| e.to_string())?;
            }
        }
        let store = TripleStore::open(dir).map_err(|e| e.to_string())?;
        store.save_pred_stats(dir).map_err(|e| e.to_string())?;
        std::fs::remove_dir_all(&tmp).ok();
        Ok(())
    }

    /// The numeric value of a term id, or `None` if it is not a numeric literal.
    /// O(1), no allocation — the engine's fast path for numeric filters. An inline
    /// integer id carries its value directly (no lookup); other ids use the cache.
    #[inline]
    pub fn numeric_value(&self, id: Id) -> Option<f64> {
        if dict::is_inline(id) {
            return Some((id - dict::INLINE_BASE) as f64);
        }
        self.numerics.lookup(id)
    }

    /// The temporal (xsd:dateTime / xsd:dateTimeStamp / xsd:date) value of a term id,
    /// or `None` if it is not a well-formed temporal literal. O(1), no allocation, no
    /// lexical re-parse — the engine's fast path for dateTime FILTER / ORDER BY /
    /// MIN/MAX, the temporal twin of [`numeric_value`](Self::numeric_value).
    #[inline]
    pub fn temporal_value(&self, id: Id) -> Option<Temporal> {
        if dict::is_inline(id) {
            return None; // inline ids are integers, never temporal
        }
        self.temporals.lookup(id)
    }

    /// The lexical form of a term id IF it is an exact-valued numeric literal (an
    /// `xsd:integer` subtype or `xsd:decimal` — NOT float/double, whose value IS its f64).
    /// Used to disambiguate comparisons that the f64 fast path collapses (integers > 2^53,
    /// high-precision decimals); only reached when the f64 values compared equal, so the
    /// allocation is rare. Inline-integer ids format their value directly.
    pub fn exact_numeric_lexical(&self, id: Id) -> Option<String> {
        if dict::is_inline(id) {
            return Some((id - dict::INLINE_BASE).to_string());
        }
        match self.dict.term_parts(id) {
            dict::TermParts::Lit { value, datatype, lang: None }
                if is_integer_datatype(datatype) || datatype == xsd::DECIMAL.as_str() =>
            {
                Some(value.to_string())
            }
            _ => None,
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
        self.dict.heap_bytes() + self.store.heap_bytes() + self.numerics.heap_bytes() + self.temporals.heap_bytes()
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

    /// Iterates every triple of the DEFAULT graph as canonical `[subject, predicate,
    /// object]` dictionary ids, in sorted (S, P, O) order — the borrowing triple
    /// iterator for downstream crates (validation, similarity, export) that today
    /// re-derive it from a raw `store.scan`. Zero allocation per row; resolve ids
    /// lazily with [`Dict::term`](dict::Dict::term) (always) or
    /// [`Dict::term_parts`](dict::Dict::term_parts) (real dictionary ids — see
    /// [`dict::is_inline`]). Because the order is subject-sorted, distinct subjects
    /// fall out of run boundaries; for distinct objects use
    /// [`iter_ids_sorted`](Self::iter_ids_sorted)`(2)`.
    pub fn iter_ids(&self) -> TripleIdIter<'_> {
        TripleIdIter { scan: self.store.scan(&[None, None, None]), i: 0 }
    }

    /// [`iter_ids`](Self::iter_ids) sorted by the canonical column `col`
    /// (0 = subject, 1 = predicate, 2 = object) — e.g. `iter_ids_sorted(2)` yields
    /// object-sorted triples so distinct objects are adjacent.
    pub fn iter_ids_sorted(&self, col: usize) -> TripleIdIter<'_> {
        TripleIdIter { scan: self.store.scan_sorted(&[None, None, None], col), i: 0 }
    }

    // ---- Structural fork (snapshot generations) --------------------------------------

    /// A STRUCTURAL FORK of this graph: a new, independently mutable `Graph` that
    /// SHARES the base's immutable storage — the six permutation indexes, the planner
    /// stats, the dictionary's frozen base and the numeric/temporal caches are all
    /// `Arc`-shared — and carries only the small per-generation delta by value (the
    /// pending store overlay, the dictionary extension, the cache side maps). Cost:
    /// O(pending delta), NOT O(graph) — except the FIRST fork of a flat (never-forked)
    /// graph, which pays a one-time O(n) dictionary/cache freeze to mint the shared
    /// base (the indexes are born shareable and never need freezing).
    ///
    /// The fork and the base then evolve independently through
    /// [`apply_delta`](Self::apply_delta): neither ever mutates shared storage, so
    /// concurrent readers of the base are unaffected (the snapshot-generation pattern:
    /// fork → apply a batch → publish, with [`compact`](Self::compact) folding the
    /// accumulated delta back into flat storage when it grows past a threshold —
    /// which also re-freezes the dictionary so later forks stay cheap).
    ///
    /// New terms interned in a fork get ids above the shared base's high-water mark
    /// (still below the inline-integer range), so a term in the shared base resolves
    /// to the SAME id in every generation. Named graphs are forked recursively. The
    /// fork carries NO write-ahead log (`apply_delta` on it is overlay-only): the
    /// generation pattern is an in-memory serving construct — durability stays with
    /// whatever owns the base.
    pub fn fork(&self) -> Graph {
        Graph {
            dict: self.dict.fork(),
            store: self.store.fork(),
            numerics: self.numerics.fork(),
            temporals: self.temporals.fork(),
            named: self.named.iter().map(|(name, g)| (name.clone(), g.fork())).collect(),
            #[cfg(feature = "mmap")]
            wal: None,
        }
    }

    /// [OPUS-4.8] (sq-5lf) A cheap, logically-INDEPENDENT, IMMUTABLE point-in-time copy
    /// of this graph — O(pending delta), never O(triples). This is the public snapshot
    /// surface beads `sq-3p1` ("Core: cheap O(overlay) Graph::snapshot API") and
    /// `sq-5lf` specify, and what the blocked downstream consumers (sparq-rsp's
    /// true-overlay window eval, sparq-py's `Graph.copy()`, the RDF/JS incremental-update
    /// story) want: hand out an O(1)-ish snapshot that reads exactly the triples present
    /// NOW and is unaffected by any later mutation of `self` (or of the snapshot — it is
    /// immutable). Built on the [structural fork](Self::fork): the six permutation
    /// indexes, planner stats, frozen dictionary base and numeric/temporal caches are
    /// `Arc`-shared; only the small per-generation delta is copied by value. (As with
    /// `fork`, the FIRST snapshot of a flat, never-forked graph pays a one-time O(n)
    /// dictionary/cache freeze to mint the shareable base; subsequent snapshots of the
    /// frozen lineage are O(overlay). Call [`compact`](Self::compact) to refreeze.)
    ///
    /// The returned [`GraphSnapshot`] is [`Send`] + [`Sync`] and derefs to `&Graph`, so it
    /// is queryable exactly like a graph (it can be passed anywhere a `&Graph` is — the
    /// engine reads `store`/`dict` through the deref) but exposes NO mutating method, so a
    /// snapshot can never diverge from its point-in-time state. To obtain a snapshot you
    /// can then keep mutating, use [`fork`](Self::fork) instead (which yields a `Graph`).
    pub fn snapshot(&self) -> GraphSnapshot {
        GraphSnapshot { graph: self.fork() }
    }

    /// Total pending-delta size carried by this graph (and its named graphs): store
    /// overlay entries + dictionary extension terms. This is what a [`fork`](Self::fork)
    /// copies by value — the input to a compaction threshold policy.
    pub fn pending_delta_len(&self) -> usize {
        self.store.overlay_len()
            + self.dict.appended_len()
            + self.named.iter().map(|(_, g)| g.pending_delta_len()).sum::<usize>()
    }

    // ---- Incremental updates (T17): delta-overlay + WAL durability ------------------

    /// Applies an incremental update batch — `deletes` first, then `inserts` (SPARQL's
    /// DELETE/INSERT application order) — through the store's DELTA-OVERLAY: O(batch)
    /// work instead of the O(n) full rebuild. New terms are interned APPEND-ONLY (the
    /// dictionary grows; existing ids never change), so readers of existing ids are
    /// unaffected. For a directory-backed graph (opened via [`open`](Self::open)) the
    /// batch is appended to the write-ahead log and fsync'd BEFORE it is applied, so a
    /// crash replays it on the next open. Fold the overlay back into the immutable base
    /// periodically with [`compact`](Self::compact).
    pub fn apply_delta(&mut self, inserts: &[[Term; 3]], deletes: &[[Term; 3]]) -> Result<(), String> {
        if inserts.is_empty() && deletes.is_empty() {
            return Ok(());
        }
        #[cfg(feature = "mmap")]
        if let Some(w) = &mut self.wal {
            w.append_batch(inserts, deletes).map_err(|e| format!("WAL append failed: {e}"))?;
        }
        self.apply_delta_mem(inserts, deletes);
        Ok(())
    }

    /// [OPUS-4.8] (review 1593) DURABLY empties this graph's default-graph content while
    /// PRESERVING its directory / WAL association — the durable replacement for
    /// `*graph = empty_graph()` on a directory-backed graph (where the latter dropped the WAL
    /// and a reopen restored the old on-disk base). For a directory-backed graph the current
    /// triples are retracted through [`apply_delta`](Self::apply_delta), so the deletions are
    /// WAL-logged + fsync'd and survive a crash/reopen; for an in-memory graph it clears the
    /// store overlay in place (no WAL to keep). Named graphs are untouched.
    pub fn clear_default_durable(&mut self) -> Result<(), String> {
        #[cfg(feature = "mmap")]
        if self.wal.is_some() {
            // Retract every current default-graph triple as a WAL-logged delete batch.
            let triples: Vec<[Term; 3]> = {
                let scan = self.store.scan(&[None, None, None]);
                scan.rows
                    .iter()
                    .map(|r| {
                        let spo = scan.to_spo(r);
                        [self.dict.term(spo[0]), self.dict.term(spo[1]), self.dict.term(spo[2])]
                    })
                    .collect()
            };
            if !triples.is_empty() {
                self.apply_delta(&[], &triples)?;
            }
            return Ok(());
        }
        // In-memory graph: no WAL/dir to preserve — clear the store content in place. The
        // dictionary is intentionally kept (ids stay stable; the empty store references none).
        self.store = TripleStore::from_triples(Vec::new());
        Ok(())
    }

    /// [`apply_delta`](Self::apply_delta) over the whole DATASET, parsing the batch
    /// from two N-Quads documents (N-Triples is the default-graph subset): `deletes`
    /// first, then `inserts`, grouped and applied PER GRAPH — default-graph statements
    /// to this graph, named-graph statements to the matching [`named`](Self::named)
    /// sub-graph (auto-created on first insert; deleting from an absent graph is a
    /// no-op). O(batch) through each graph's delta overlay, no index rebuild. Blank
    /// nodes denote concrete nodes BY LABEL (unlike SPARQL `DELETE DATA`, which cannot
    /// name a bnode), which is what makes quad-level retraction of bnode triples
    /// possible. The string-parsing entry point for bindings (wasm/python) callers.
    pub fn apply_delta_nquads(&mut self, inserts: &str, deletes: &str) -> Result<(), String> {
        use oxrdf::GraphName;
        type Slot = Option<Term>;
        // Per-graph delta: `(slot, inserts, deletes)` — one entry per distinct graph.
        type SlotDelta = (Slot, Vec<[Term; 3]>, Vec<[Term; 3]>);
        fn parse(text: &str) -> Result<Vec<(Slot, [Term; 3])>, String> {
            let mut out = Vec::new();
            for q in NQuadsParser::new().for_slice(text.as_bytes()) {
                let q = q.map_err(|e| e.to_string())?;
                let slot = match q.graph_name {
                    GraphName::DefaultGraph => None,
                    GraphName::NamedNode(n) => Some(Term::NamedNode(n)),
                    GraphName::BlankNode(b) => Some(Term::BlankNode(b)),
                };
                out.push((slot, [subject_term(&q.subject), Term::NamedNode(q.predicate), q.object]));
            }
            Ok(out)
        }
        // Group per slot preserving first-seen order (datasets hold few graphs, so a
        // linear scan beats hashing Terms), keeping each slot's deletes and inserts
        // together so they go through ONE apply_delta call (deletes applied first).
        let mut slots: Vec<SlotDelta> = Vec::new();
        for (is_insert, items) in [(false, parse(deletes)?), (true, parse(inserts)?)] {
            for (slot, t) in items {
                let entry = match slots.iter_mut().find(|(s, _, _)| *s == slot) {
                    Some(e) => e,
                    None => {
                        slots.push((slot, Vec::new(), Vec::new()));
                        slots.last_mut().expect("just pushed")
                    }
                };
                if is_insert {
                    entry.1.push(t);
                } else {
                    entry.2.push(t);
                }
            }
        }
        for (slot, ins, del) in slots {
            match slot {
                None => self.apply_delta(&ins, &del)?,
                Some(name) => {
                    if let Some(i) = self.named.iter().position(|(n, _)| *n == name) {
                        self.named[i].1.apply_delta(&ins, &del)?;
                    } else if !ins.is_empty() {
                        let mut g = Graph::from_parts(Dict::new(), Vec::new());
                        g.apply_delta(&ins, &[])?;
                        self.named.push((name, g));
                    } // deletes against an absent graph: no-op
                }
            }
        }
        Ok(())
    }

    /// The in-memory half of [`apply_delta`](Self::apply_delta) (no WAL append) — also
    /// the target the WAL replays into on [`open`](Self::open).
    fn apply_delta_mem(&mut self, inserts: &[[Term; 3]], deletes: &[[Term; 3]]) {
        // A delete only matters if every term resolves — otherwise the triple cannot be
        // present, and deleting must NOT intern the (absent) terms.
        let del_ids: Vec<[Id; 3]> = deletes
            .iter()
            .filter_map(|[s, p, o]| Some([self.id_of(s)?, self.id_of(p)?, self.id_of(o)?]))
            .collect();
        let old_len = self.dict.len();
        let ins_ids: Vec<[Id; 3]> = inserts
            .iter()
            .map(|[s, p, o]| [self.dict.intern(s), self.dict.intern(p), self.dict.intern(o)])
            .collect();
        // Keep the numeric- and temporal-filter caches covering the grown dictionary.
        self.numerics.extend_for(&self.dict, old_len);
        self.temporals.extend_for(&self.dict, old_len);
        self.store.apply_delta(&ins_ids, &del_ids);
    }

    /// Folds the delta-overlay into a REBUILT immutable base (the periodic compaction
    /// that keeps scans overlay-free). The dictionary is kept as-is — ids are stable;
    /// terms only referenced by deleted triples linger until a full reload (cheap, and
    /// it keeps compaction O(triples) with no re-interning). For a directory-backed
    /// graph the new base is persisted ATOMICALLY (written to a fresh sibling directory,
    /// then swapped in via rename) and the write-ahead log truncated; the graph re-opens
    /// memory-mapped from the new base.
    pub fn compact(&mut self) -> Result<(), String> {
        #[cfg(feature = "mmap")]
        let dir = self.wal.as_ref().map(|w| w.dir.clone());
        // Fold the structural-fork layers first (ids unchanged throughout): named
        // graphs recursively, then this graph's dictionary extension into a fresh
        // frozen base (so the NEXT fork is O(1) again) and the forked caches flat.
        // No-ops on a never-forked graph.
        for (_, g) in &mut self.named {
            g.compact()?;
        }
        if self.dict.is_forked() {
            self.dict = self.dict.compacted();
            let n = self.dict.len();
            let numerics = std::mem::replace(&mut self.numerics, NumData::Sparse(rustc_hash::FxHashMap::default()));
            self.numerics = numerics.fold(n);
            let temporals = std::mem::replace(&mut self.temporals, TempData::Sparse(rustc_hash::FxHashMap::default()));
            self.temporals = temporals.fold(n);
        }
        if self.store.has_overlay() {
            let triples: Vec<[Id; 3]> = {
                let scan = self.store.scan(&[None, None, None]);
                scan.rows.iter().map(|r| scan.to_spo(r)).collect()
            };
            self.store = TripleStore::from_triples(triples);
        } else {
            // Nothing pending: for a directory-backed graph just discard the (no-op) log.
            #[cfg(feature = "mmap")]
            if let Some(w) = &mut self.wal {
                w.truncate().map_err(|e| e.to_string())?;
            }
            return Ok(());
        }
        #[cfg(feature = "mmap")]
        if let Some(dir) = dir {
            // [OPUS-4.8] (review 1593) ROLLBACK-SAFE directory swap. The two-rename swap has a
            // window where the canonical `dir` does not exist (between the two renames); a
            // crash there must NOT lose the dataset. We:
            //   1. write the new base to `compact-new` and fsync the DIRECTORY (so its
            //      existence + contents are durable before it can become canonical);
            //   2. rename `dir` -> `compact-old`, fsync the parent (the rename is durable);
            //   3. rename `compact-new` -> `dir`, fsync the parent.
            // If a crash interrupts the swap, `recover_compaction` (run on every open)
            // deterministically completes or rolls it back from the surviving sibling.
            let new_dir = dir.with_extension("compact-new");
            let old_dir = dir.with_extension("compact-old");
            std::fs::remove_dir_all(&new_dir).ok();
            std::fs::remove_dir_all(&old_dir).ok();
            self.wal = None; // close the log before the directory swap
            self.save(&new_dir).map_err(|e| e.to_string())?;
            fsync_dir(&new_dir).map_err(|e| e.to_string())?;
            let parent = dir.parent().unwrap_or_else(|| std::path::Path::new("."));
            std::fs::rename(&dir, &old_dir).map_err(|e| e.to_string())?;
            fsync_dir(parent).map_err(|e| e.to_string())?;
            std::fs::rename(&new_dir, &dir).map_err(|e| e.to_string())?;
            fsync_dir(parent).map_err(|e| e.to_string())?;
            // Re-open memory-mapped from the new base (fresh, empty WAL); only then drop
            // the old files (open mmaps keep unlinked files alive on unix).
            *self = Graph::open(&dir).map_err(|e| e.to_string())?;
            std::fs::remove_dir_all(&old_dir).ok();
        }
        Ok(())
    }
}

/// [OPUS-4.8] (review 1593) fsync a DIRECTORY so a rename of (or into) it is durable. On
/// failure (e.g. a platform/filesystem that rejects opening a dir) it is a best-effort no-op,
/// matching the durability the rest of the persistence path assumes.
#[cfg(feature = "mmap")]
fn fsync_dir(dir: &std::path::Path) -> std::io::Result<()> {
    match std::fs::File::open(dir) {
        Ok(f) => match f.sync_all() {
            Ok(()) => Ok(()),
            // Some filesystems return EINVAL/unsupported for fsync on a directory handle;
            // that is not a data-loss condition, so treat it as success.
            Err(e) if matches!(e.kind(), std::io::ErrorKind::InvalidInput) => Ok(()),
            Err(e) => Err(e),
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

/// [OPUS-4.8] (review 1593) Recover an INTERRUPTED [`compact`](Graph::compact) directory swap
/// so the canonical `dir` always ends up present after any crash. Deterministic from the
/// surviving siblings (`<dir>.compact-old` / `<dir>.compact-new`):
///
/// - `dir` present: the swap completed (or never started) — just remove any stale siblings.
/// - `dir` MISSING, `compact-new` present: a crash between the two renames — the new base was
///   already fully written + dir-fsynced, so COMPLETE the swap by promoting `compact-new`.
/// - `dir` MISSING, only `compact-old` present: a crash before the new base was ready — ROLL
///   BACK by restoring `compact-old`.
#[cfg(feature = "mmap")]
fn recover_compaction(dir: &std::path::Path) -> std::io::Result<()> {
    let new_dir = dir.with_extension("compact-new");
    let old_dir = dir.with_extension("compact-old");
    let parent = dir.parent().unwrap_or_else(|| std::path::Path::new("."));
    if dir.exists() {
        // Canonical dir is intact; drop any leftover siblings from a crash after the swap.
        std::fs::remove_dir_all(&new_dir).ok();
        std::fs::remove_dir_all(&old_dir).ok();
        return Ok(());
    }
    if new_dir.exists() {
        // Complete the swap: the new base was fully written + synced before the swap began.
        std::fs::rename(&new_dir, dir)?;
        fsync_dir(parent)?;
        std::fs::remove_dir_all(&old_dir).ok();
    } else if old_dir.exists() {
        // Roll back: restore the previous base (the new one never reached durability).
        std::fs::rename(&old_dir, dir)?;
        fsync_dir(parent)?;
    }
    Ok(())
}

/// The append-only write-ahead log of update operations for a directory-backed graph.
///
/// [OPUS-4.8] (review 1593) Entries are framed as ATOMIC BATCHES, not independent per-triple
/// records, so a crash mid-append can never leave a PARTIALLY-applied batch on replay. Each
/// batch is laid out as:
///
/// ```text
/// [BATCH_MAGIC u32][n_records u32][body_len u32]   <- header
/// <body_len bytes of records>                       <- each: [op u8][len u32][N-Triples bytes]
/// [checksum u64][COMMIT_MARKER u32]                 <- commit trailer (written last)
/// ```
///
/// The checksum (FNV-1a over the header + body) plus the trailing commit marker form the
/// COMMIT POINT: `replay` only applies a batch whose trailer is present AND whose checksum
/// matches, and TRUNCATES the log at the start of the first batch that fails either check
/// (the torn tail of an interrupted append). So replay applies exactly the batches that were
/// fully durable — never a prefix of one. The whole batch is `write_all`'d then fsync'd once.
#[cfg(feature = "mmap")]
struct Wal {
    file: std::fs::File,
    dir: std::path::PathBuf,
}

#[cfg(feature = "mmap")]
impl Wal {
    const INSERT: u8 = 0;
    const DELETE: u8 = 1;
    /// Batch frame magic ("WBA1" little-endian) — starts every batch header.
    const BATCH_MAGIC: u32 = 0x31_41_42_57;
    /// Commit marker ("DONE" little-endian) — written last, after the checksum.
    const COMMIT_MARKER: u32 = 0x45_4E_4F_44;

    fn path(dir: &std::path::Path) -> std::path::PathBuf {
        dir.join("wal.log")
    }

    fn open(dir: &std::path::Path) -> std::io::Result<Wal> {
        let file = std::fs::OpenOptions::new().create(true).append(true).open(Self::path(dir))?;
        Ok(Wal { file, dir: dir.to_path_buf() })
    }

    /// FNV-1a 64-bit checksum (no extra deps; deterministic across runs/platforms).
    fn checksum(bytes: &[u8]) -> u64 {
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        for &b in bytes {
            h ^= b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01B3);
        }
        h
    }

    /// Appends one ATOMIC batch — delete records first, then inserts (the order
    /// `apply_delta` applies them) — framed with a header + checksum + commit marker, and
    /// fsyncs, so the WHOLE batch is durable as a unit BEFORE it is applied in memory. A
    /// crash mid-append leaves an uncommitted tail that `replay` discards entirely.
    fn append_batch(&mut self, inserts: &[[Term; 3]], deletes: &[[Term; 3]]) -> std::io::Result<()> {
        use std::io::Write;
        let mut body = Vec::new();
        let mut n_records = 0u32;
        for t in deletes {
            Self::push_record(&mut body, Self::DELETE, t);
            n_records += 1;
        }
        for t in inserts {
            Self::push_record(&mut body, Self::INSERT, t);
            n_records += 1;
        }
        let mut buf = Vec::with_capacity(body.len() + 24);
        buf.extend_from_slice(&Self::BATCH_MAGIC.to_le_bytes());
        buf.extend_from_slice(&n_records.to_le_bytes());
        buf.extend_from_slice(&(body.len() as u32).to_le_bytes());
        buf.extend_from_slice(&body);
        // Checksum covers everything written so far (header + body); the commit marker is
        // appended AFTER the checksum so a torn write can't produce a valid-looking trailer.
        let crc = Self::checksum(&buf);
        buf.extend_from_slice(&crc.to_le_bytes());
        buf.extend_from_slice(&Self::COMMIT_MARKER.to_le_bytes());
        self.file.write_all(&buf)?;
        self.file.sync_data()
    }

    fn push_record(buf: &mut Vec<u8>, op: u8, t: &[Term; 3]) {
        // oxrdf's Display is the canonical N-Triples serialisation of each term.
        let line = format!("{} {} {} .\n", t[0], t[1], t[2]);
        buf.push(op);
        buf.extend_from_slice(&(line.len() as u32).to_le_bytes());
        buf.extend_from_slice(line.as_bytes());
    }

    /// Empties the log (after a compaction persisted the folded base).
    fn truncate(&mut self) -> std::io::Result<()> {
        self.file.set_len(0)?;
        self.file.sync_data()
    }

    /// Reads `dir`'s log into `(is_insert, triple)` ops in append order, batch by batch.
    /// Only fully-committed batches (intact header + body + matching checksum + commit
    /// marker) are applied; the file is TRUNCATED at the start of the first incomplete or
    /// corrupt batch — the torn tail of an interrupted append — so the partial batch is
    /// NEVER partially applied and subsequent appends start at a clean batch boundary.
    fn replay(dir: &std::path::Path) -> std::io::Result<Vec<(bool, [Term; 3])>> {
        let path = Self::path(dir);
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(e),
        };
        let mut ops = Vec::new();
        let mut good = 0usize; // end of the last fully-committed batch
        'batches: while good < bytes.len() {
            let bstart = good;
            // Header: magic + n_records + body_len (12 bytes).
            if bstart + 12 > bytes.len() {
                break; // torn header
            }
            let rd = |o: usize| u32::from_le_bytes([bytes[o], bytes[o + 1], bytes[o + 2], bytes[o + 3]]);
            if rd(bstart) != Self::BATCH_MAGIC {
                break; // not a batch frame (corruption / torn tail)
            }
            let n_records = rd(bstart + 4) as usize;
            let body_len = rd(bstart + 8) as usize;
            let body_start = bstart + 12;
            let Some(body_end) = body_start.checked_add(body_len).filter(|&e| e <= bytes.len()) else {
                break; // torn body
            };
            // Commit trailer: checksum (8) + marker (4).
            let Some(trailer_end) = body_end.checked_add(12).filter(|&e| e <= bytes.len()) else {
                break; // trailer not yet written — uncommitted batch
            };
            let stored_crc = u64::from_le_bytes(bytes[body_end..body_end + 8].try_into().expect("8 bytes"));
            let marker = rd(body_end + 8);
            if marker != Self::COMMIT_MARKER || Self::checksum(&bytes[bstart..body_end]) != stored_crc {
                break; // uncommitted / corrupt batch — discard the tail
            }
            // The batch is committed: parse its records. (A parse failure here would mean a
            // committed-but-corrupt body — treat it as a torn tail and stop, conservatively.)
            let mut pos = body_start;
            let mut batch_ops = Vec::with_capacity(n_records);
            for _ in 0..n_records {
                if pos + 5 > body_end {
                    break 'batches;
                }
                let op = bytes[pos];
                let len = rd(pos + 1) as usize;
                let start = pos + 5;
                let Some(end) = start.checked_add(len).filter(|&e| e <= body_end) else {
                    break 'batches;
                };
                let Some(t) = parse_nt_record(&bytes[start..end]) else {
                    break 'batches;
                };
                batch_ops.push((op == Self::INSERT, t));
                pos = end;
            }
            if batch_ops.len() != n_records {
                break; // body record count disagreed with the header — corrupt
            }
            ops.extend(batch_ops);
            good = trailer_end;
        }
        if good < bytes.len() {
            let f = std::fs::OpenOptions::new().write(true).open(&path)?;
            f.set_len(good as u64)?;
            f.sync_data()?;
        }
        Ok(ops)
    }
}

/// Parses one WAL record body (a single N-Triples statement) into its term triple.
#[cfg(feature = "mmap")]
fn parse_nt_record(bytes: &[u8]) -> Option<[Term; 3]> {
    let mut it = NTriplesParser::new().for_slice(bytes);
    let t = it.next()?.ok()?;
    Some([subject_term(&t.subject), Term::NamedNode(t.predicate), t.object])
}

/// The numeric-value cache for a dictionary: `numerics[id-1]` is the f64 value of term
/// `id` (NaN for non-numeric). Parallel when the `parallel` feature is on.
fn numerics_of(dict: &Dict) -> Vec<f64> {
    let n = dict.len();
    #[cfg(feature = "parallel")]
    {
        use rayon::prelude::*;
        (0..n).into_par_iter().map(|i| numeric_of(&dict.term(i as Id + 1))).collect()
    }
    #[cfg(not(feature = "parallel"))]
    {
        (0..n).map(|i| numeric_of(&dict.term(i as Id + 1))).collect()
    }
}

/// Writes the numeric-value cache to disk (raw little-endian f64) so it can be
/// memory-mapped on open instead of recomputed.
#[cfg(feature = "mmap")]
fn write_numerics(path: &std::path::Path, nums: &[f64]) -> std::io::Result<()> {
    // SAFETY: reinterpret the contiguous f64 cache as bytes for writing.
    let bytes = unsafe { std::slice::from_raw_parts(nums.as_ptr().cast::<u8>(), std::mem::size_of_val(nums)) };
    std::fs::write(path, bytes)
}

fn subject_term(s: &oxrdf::NamedOrBlankNode) -> Term {
    match s {
        oxrdf::NamedOrBlankNode::NamedNode(n) => Term::NamedNode(n.clone()),
        oxrdf::NamedOrBlankNode::BlankNode(b) => Term::BlankNode(b.clone()),
    }
}

/// Splits a byte buffer into ~`target` ranges, each ending on a newline so no
/// (single-line) N-Triples statement is split across a boundary.
#[cfg(feature = "parallel")]
fn newline_chunk_bounds(bytes: &[u8], target: usize) -> Vec<(usize, usize)> {
    let mut bounds = Vec::with_capacity(target);
    let chunk = (bytes.len() / target.max(1)).max(1);
    let mut start = 0;
    while start < bytes.len() {
        let mut end = (start + chunk).min(bytes.len());
        if end < bytes.len() {
            match bytes[end..].iter().position(|&b| b == b'\n') {
                Some(p) => end += p + 1,
                None => end = bytes.len(),
            }
        }
        bounds.push((start, end));
        start = end;
    }
    bounds
}

/// Parses + interns N-Triples in parallel: each chunk builds a partial dictionary +
/// local-id triples, then the partials are merged into one global dictionary with the
/// local ids remapped. Interning is per-thread (no shared lock); the merge is linear.
#[cfg(feature = "parallel")]
/// Per-ISA software prefetch-for-read hint (x86 `prefetcht0`, aarch64 `prfm pldl1keep`, a no-op
/// elsewhere). Correctness-neutral — a prefetch never faults and never changes architectural
/// state; it only asks the CPU to pull a cache line in early.
#[cfg(feature = "parallel")]
#[inline(always)]
fn prefetch_read<T>(p: *const T) {
    #[cfg(target_arch = "x86_64")]
    // SAFETY: _mm_prefetch is defined for any address (the hint is dropped on a bad one).
    unsafe {
        core::arch::x86_64::_mm_prefetch(p as *const i8, core::arch::x86_64::_MM_HINT_T0);
    }
    #[cfg(target_arch = "aarch64")]
    // SAFETY: prfm is a hint — it cannot fault or write memory/registers.
    unsafe {
        core::arch::asm!("prfm pldl1keep, [{0}]", in(reg) p, options(nostack, preserves_flags));
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    let _ = p;
}

/// Compile-time default for the dict-remap software prefetch, chosen per hardware from measured
/// A/B results (the `bench-remap` micro-benchmark, prefetch on vs off; the hint helps or *hurts*
/// depending on the core's hardware prefetcher):
///   x86_64 (Intel/AMD; incl. AWS c7i Sapphire Rapids):   +7.5%  -> ON
///   aarch64 + macOS (Apple M-series):                    +22%   -> ON
///   aarch64 + Linux (AWS Graviton3 / Neoverse-V1, etc.): -10%   -> OFF — the HW prefetcher
///       already saturates the gather, so explicit `prfm` hints only add instruction overhead.
/// Overridable at runtime for re-tuning on new silicon: `SPARQ_PREFETCH=1` forces on,
/// `SPARQ_NO_PREFETCH=1` forces off.
#[cfg(feature = "parallel")]
const PREFETCH_DEFAULT: bool = cfg!(any(
    target_arch = "x86_64",
    all(target_arch = "aarch64", target_os = "macos"),
));

/// Append `triples`, remapped from a partial dict's ids to the merged global ids
/// (`remap[id-1]`; inline-integer ids pass through). The `remap` gather is latency-bound on a
/// large global dictionary (the build-path bottleneck — triple-remap measured at ~3s/50M), so
/// each iteration software-prefetches the gather targets of a triple `DIST` ahead. Hardware-
/// specific (per-ISA prefetch) but correctness-neutral; the prefetch is just a hint.
#[cfg(feature = "parallel")]
fn remap_extend(out: &mut Vec<[Id; 3]>, triples: Vec<[Id; 3]>, remap: &[Id]) {
    const DIST: usize = 32;
    let n = triples.len();
    let base = remap.as_ptr();
    // Per-hardware default (PREFETCH_DEFAULT, measured), with a runtime override. The getenv runs
    // once per merged-dict load — which then processes millions of triples — so it is free.
    let do_prefetch = match (
        std::env::var("SPARQ_PREFETCH").as_deref(),
        std::env::var("SPARQ_NO_PREFETCH").as_deref(),
    ) {
        (Ok("1"), _) => true,
        (_, Ok("1")) => false,
        _ => PREFETCH_DEFAULT,
    };
    let lut = |id: Id| -> Id {
        if id >= dict::INLINE_BASE {
            id
        } else {
            remap[(id - 1) as usize]
        }
    };
    out.reserve(n);
    for i in 0..n {
        if do_prefetch && i + DIST < n {
            for &id in &triples[i + DIST] {
                if id < dict::INLINE_BASE {
                    // SAFETY: id-1 < remap.len() for every dictionary id; prefetch is hint-only.
                    prefetch_read(unsafe { base.add((id - 1) as usize) });
                }
            }
        }
        let [s, p, o] = triples[i];
        out.push([lut(s), lut(p), lut(o)]);
    }
}

/// Borrowing iterator over a graph's default-graph triples as canonical
/// `[subject, predicate, object]` ids — see [`Graph::iter_ids`] /
/// [`Graph::iter_ids_sorted`]. Holds the underlying index scan (a zero-copy
/// borrow on an overlay-free store), so it borrows the graph for its lifetime.
pub struct TripleIdIter<'g> {
    scan: store::Scan<'g>,
    i: usize,
}

impl Iterator for TripleIdIter<'_> {
    type Item = [Id; 3];

    #[inline]
    fn next(&mut self) -> Option<[Id; 3]> {
        let row = self.scan.rows.get(self.i)?;
        self.i += 1;
        Some(self.scan.to_spo(row))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.scan.rows.len() - self.i;
        (n, Some(n))
    }
}

impl ExactSizeIterator for TripleIdIter<'_> {}

/// Isolated micro-benchmark of the latency-bound `remap_extend` gather (the build-path
/// bottleneck the per-ISA prefetch targets). Builds `n` synthetic triples whose ids scatter
/// randomly across a `dict_size`-entry remap table (so the gather misses cache like a real large
/// global dictionary), then times `remap_extend` over `iters` runs and returns the best (ms).
/// Honours `SPARQ_NO_PREFETCH=1`. Used to measure the prefetch's effect per hardware in isolation,
/// undiluted by parsing. Not part of the query/build path.
#[cfg(feature = "parallel")]
pub fn bench_remap(n: usize, dict_size: usize, iters: usize) -> f64 {
    // Cheap deterministic LCG scatter (no rand dep, no Date/Random harness restrictions).
    let mut x: u64 = 0x9E3779B97F4A7C15;
    let mut next = || { x ^= x << 13; x ^= x >> 7; x ^= x << 17; x };
    let ds = dict_size.max(1) as u64;
    let mut triples: Vec<[Id; 3]> = Vec::with_capacity(n);
    for _ in 0..n {
        // ids in [1, dict_size], scattered, so the gather misses cache.
        let s = (1 + (next() % ds)) as Id;
        let p = (1 + (next() % ds)) as Id;
        let o = (1 + (next() % ds)) as Id;
        triples.push([s, p, o]);
    }
    // Identity remap of the right size (values irrelevant to the gather's latency).
    let remap: Vec<Id> = (1..=dict_size as Id).collect();
    let mut best = f64::INFINITY;
    for _ in 0..iters.max(1) {
        let mut out: Vec<[Id; 3]> = Vec::new();
        let t = std::time::Instant::now();
        remap_extend(&mut out, triples.clone(), &remap);
        let ms = t.elapsed().as_secs_f64() * 1e3;
        std::hint::black_box(&out);
        if ms < best { best = ms; }
    }
    best
}

#[cfg(feature = "parallel")]
fn parse_ntriples_parallel(bytes: &[u8]) -> Result<(Dict, Vec<[Id; 3]>), String> {
    let partials = parse_block(bytes)?;
    let total: usize = partials.iter().map(|(_, t)| t.len()).sum();
    // One rayon thread: the sharded merge would only add routing/consolidation overhead —
    // keep the proven serial merge (also the byte-reference the differential tests pin).
    if rayon::current_num_threads() <= 1 {
        let cap = partials.iter().map(|(d, _)| d.len()).max().unwrap_or(0);
        let mut global = Dict::with_capacity(cap);
        let mut all = Vec::with_capacity(total);
        for (pd, ptriples) in partials {
            let remap = global.merge_remap(&pd);
            // Dictionary ids are 1-based and below INLINE_BASE; inline-integer ids carry
            // their value and pass through unchanged. Prefetches the gather (remap_extend).
            remap_extend(&mut all, ptriples, &remap);
        }
        return Ok((global, all));
    }
    // ≥2 threads: SHARDED parallel dict consolidation (the measured serial `merge_remap`
    // ceiling — load plateaued at ~1.8× on 4 identical cores — was exactly this stage).
    let mut sd = dict::ShardedDict::new(default_shards());
    let mut all: Vec<[Id; 3]> = Vec::with_capacity(total);
    sharded_extend(&mut sd, &partials, &mut all);
    Ok(finish_sharded(sd, all))
}

/// The shard count for the parallel in-memory/streaming dict consolidation (2 shards per
/// rayon thread for load balance; same policy as the external sharded build).
#[cfg(feature = "parallel")]
fn default_shards() -> usize {
    (rayon::current_num_threads() * 2).clamp(4, 64)
}

/// Interns a parsed block's partial dicts into the sharded dict and appends the block's
/// triples (remapped to TEMPORARY sharded ids, inline ids passing through) to `all` — the
/// parallel-merge step shared by the in-memory N-Triples loaders. The remap gather runs
/// in parallel (indexed `par_extend`, deterministic order).
#[cfg(feature = "parallel")]
fn sharded_extend(sd: &mut dict::ShardedDict, partials: &[(Dict, Vec<[Id; 3]>)], all: &mut Vec<[Id; 3]>) {
    use rayon::prelude::*;
    let remaps = sd.intern_partials(partials);
    for (pidx, (_, ptriples)) in partials.iter().enumerate() {
        let rm = &remaps[pidx];
        let map = |id: Id| if id >= dict::INLINE_BASE { id } else { rm[id as usize] };
        all.par_extend(ptriples.par_iter().map(|&[s, p, o]| [map(s), map(p), map(o)]));
    }
}

/// Consolidates the sharded dict into one `Dict` (parallel arena move + parallel-hash
/// lookup-table rebuild, so the result serves `lookup`/`intern` like a serially-built
/// dict) and remaps `all` from temporary sharded ids to the final dense ids in parallel.
#[cfg(feature = "parallel")]
fn finish_sharded(sd: dict::ShardedDict, mut all: Vec<[Id; 3]>) -> (Dict, Vec<[Id; 3]>) {
    use rayon::prelude::*;
    let (mut dict, base, stride) = sd.into_merged();
    dict.build_table();
    all.par_iter_mut().for_each(|t| {
        for c in t.iter_mut() {
            *c = dict::remap_sharded(*c, &base, stride);
        }
    });
    (dict, all)
}

/// [OPUS-4.8] (T1) Intern an oxttl-parsed subject (`NamedNode`/`BlankNode`) from its BORROWED
/// `&str`, without materializing an owned `oxrdf::Term`. `subject_term` + `dict.intern(&Term)`
/// built a heap `Term` per S — bumping the `NamedNode`/`BlankNode` ref-count (an `Arc`/`Rc` clone
/// in oxrdf's interned-IRI representation) and constructing the `Term` enum — only for `intern`
/// to immediately re-borrow `.as_str()` and dispatch right back to `intern_iri`/`intern_blank`.
/// Here we dispatch on the borrowed component directly: the only allocation left is the one
/// `intern_*` makes on a genuinely new term (copy into `Box<str>`), which is unavoidable.
#[cfg(feature = "parallel")]
#[inline]
fn intern_subject_ref(dict: &mut Dict, s: &oxrdf::NamedOrBlankNode) -> Id {
    match s {
        oxrdf::NamedOrBlankNode::NamedNode(n) => dict.intern_iri(n.as_str()),
        oxrdf::NamedOrBlankNode::BlankNode(b) => dict.intern_blank(b.as_str()),
    }
}

/// [OPUS-4.8] (T1) Intern an oxttl-parsed object `Term` from its BORROWED components — the object
/// slot is the one that can be a literal or (RDF-star) a quoted triple. IRIs/blank nodes/literals
/// dispatch straight to the component interners with `&str` views; a nested triple term recurses
/// (its s/p/o are interned first, then the triple is stored by component ids, matching
/// `Dict::intern(&Term::Triple(_))`). No owned `Term` is built for the common non-star case.
#[cfg(feature = "parallel")]
#[inline]
fn intern_object_ref(dict: &mut Dict, o: &Term) -> Id {
    match o {
        Term::NamedNode(n) => dict.intern_iri(n.as_str()),
        Term::BlankNode(b) => dict.intern_blank(b.as_str()),
        Term::Literal(l) => {
            dict.intern_lit(l.value(), l.datatype().as_str(), crate::dict::lang_with_dir(l).as_deref())
        }
        // RDF-star quoted triple: rare; fall back to the owned-Term path (handles nesting +
        // content-addressed triple ids identically to the serial parser).
        Term::Triple(_) => dict.intern(o),
    }
}

/// Serial Turtle parse of `bytes` into `dict` — the fallback and the per-chunk worker.
///
/// [OPUS-4.8] (T1) Interns each S/P/O directly from oxttl's already-parsed BORROWED term views
/// (`intern_subject_ref` / `intern_iri` / `intern_object_ref`) instead of first materializing an
/// owned `oxrdf::Term` per component and re-borrowing it in `dict.intern`. oxttl still does all
/// tokenization, grammar and prefixed-name expansion — this only removes the per-triple `Term`
/// heap churn between oxttl's output and the dict.
#[cfg(feature = "parallel")]
fn parse_turtle_chunk(bytes: &[u8], dict: &mut Dict) -> Result<Vec<[Id; 3]>, String> {
    let mut triples = Vec::new();
    for t in TurtleParser::new().for_slice(bytes) {
        let t = t.map_err(|e| e.to_string())?;
        let s = intern_subject_ref(dict, &t.subject);
        let p = dict.intern_iri(t.predicate.as_str());
        let o = intern_object_ref(dict, &t.object);
        triples.push([s, p, o]);
    }
    Ok(triples)
}

/// Skip whitespace and `#`-comments from `i`, returning the next significant byte offset.
#[cfg(feature = "parallel")]
fn skip_ws_comments(bytes: &[u8], mut i: usize) -> usize {
    let n = bytes.len();
    loop {
        while i < n && matches!(bytes[i], b' ' | b'\t' | b'\n' | b'\r') {
            i += 1;
        }
        if i < n && bytes[i] == b'#' {
            while i < n && bytes[i] != b'\n' {
                i += 1;
            }
        } else {
            return i;
        }
    }
}

/// Is the token at `k` a SPARQL-style `PREFIX`/`BASE` directive (keyword + whitespace)?
#[cfg(feature = "parallel")]
fn is_sparql_directive_start(bytes: &[u8], k: usize) -> bool {
    let m = |kw: &[u8]| {
        bytes.len() > k + kw.len()
            && bytes[k..k + kw.len()].eq_ignore_ascii_case(kw)
            && matches!(bytes.get(k + kw.len()), Some(b' ' | b'\t' | b'\n' | b'\r'))
    };
    m(b"prefix") || m(b"base")
}

/// [OPUS-4.8] (T3) Delimit a SPARQL-style `PREFIX`/`BASE` directive starting at `start` (which
/// [`is_sparql_directive_start`] has already confirmed). Unlike a statement or an `@`-directive,
/// the SPARQL form has **no `.` terminator** — it ends right after the closing `>` of its single
/// `IRIREF` (the last token of both `BASE <iri>` and `PREFIX pname: <iri>`). Returns the offset
/// just past that `>`, or `None` (→ caller falls back to serial) if the directive is malformed or
/// truncated, in which case the serial oxttl parser produces the rejection.
///
/// Correctness of treating this as a snapshot span: the byte run `[start, end)` is exactly the
/// directive text oxttl would consume, so replaying it verbatim into a later chunk's preamble
/// reproduces the prefix/base binding identically (confirmed: oxttl accepts mixed `@`/SPARQL
/// forms, SPARQL redefinitions, and relative SPARQL `BASE`). A trailing `.` after the `>` (which
/// is INVALID Turtle for the SPARQL form) is deliberately NOT consumed: it remains in the stream
/// as the start of the next unit, where the per-chunk oxttl parse rejects it — preserving the
/// rejection the W3C `turtle-syntax-bad-base-03` case requires.
///
/// The scan skips inter-token whitespace and `#`-comments but otherwise only has to locate the
/// IRIREF: anything between the keyword and the `<` that is NOT whitespace/comment/`PNAME_NS`
/// makes the directive malformed, so we bail to serial rather than guess.
#[cfg(feature = "parallel")]
fn next_sparql_directive_end(bytes: &[u8], start: usize) -> Option<usize> {
    let n = bytes.len();
    // Past the keyword: `PREFIX` is 6 bytes, `BASE` is 4. `is_sparql_directive_start` guarantees
    // one of them matches at `start` followed by whitespace.
    let kw_len = if bytes[start..].len() >= 6 && bytes[start..start + 6].eq_ignore_ascii_case(b"prefix") {
        6
    } else {
        4 // base
    };
    // Skip whitespace/comments up to the IRIREF (for PREFIX the PNAME_NS sits in between; we
    // don't need to validate it — the only `<` before the directive's own IRIREF is that IRIREF,
    // and oxttl validates the PNAME_NS when it re-parses the snapshot).
    let mut i = skip_ws_comments(bytes, start + kw_len);
    // Find the IRIREF's opening `<`, allowing only a PNAME_NS token (PREFIX form) before it. If
    // we hit a statement-relevant byte (`.`, a quote, EOF) before `<`, the directive is malformed.
    while i < n && bytes[i] != b'<' {
        match bytes[i] {
            b'.' | b'"' | b'\'' | b';' | b',' | b'[' | b']' | b'(' | b')' | b'@' => return None,
            b'#' => i = skip_ws_comments(bytes, i),
            _ => i += 1,
        }
    }
    if i >= n {
        return None; // no IRIREF before EOF
    }
    // Consume the IRIREF `<...>`; `>` cannot appear unescaped inside (it is `>`), so the
    // first `>` closes it. A newline inside an IRIREF is also illegal, but oxttl re-validates the
    // snapshot, so we only need a cheap span here.
    i += 1;
    memchr::memchr(b'>', &bytes[i..]).map(|off| i + off + 1)
}

/// Scan from `start` (in the top-level/normal Turtle state) to the next statement-terminating
/// `.` (one followed by whitespace/EOF/comment, i.e. not a decimal point or PN_LOCAL dot),
/// skipping over IRIs `<...>`, string literals (all four quote forms, with `\` escapes), and
/// `#` comments. Returns the offset just past the `.`, or `None` if EOF is reached first
/// (malformed/incomplete → caller falls back to the serial parser).
///
/// Blank-node syntax (`[...]` property lists, `(...)` collections, `_:` labels) needs no
/// special handling here: in VALID Turtle a `.` followed by whitespace/EOF/`#` cannot occur
/// inside them outside of the strings/IRIs/comments already skipped — DECIMAL/DOUBLE require
/// a digit or exponent immediately after the dot, and PN_LOCAL / BLANK_NODE_LABEL dots must
/// be followed by a continuation character — so every offset returned is a true top-level
/// end-of-statement. (On INVALID input a mis-split makes some chunk fail to parse, and
/// [`parse_turtle_chunked`] redoes the document serially.) See [`turtle_chunks`] for why
/// chunk-independent parsing of the blank nodes themselves is sound.
#[cfg(feature = "parallel")]
fn next_terminator(bytes: &[u8], start: usize) -> Option<usize> {
    let n = bytes.len();
    let mut i = start;
    while i < n {
        // [OPUS-4.8] (T2) SIMD-skip the uninteresting run to the next byte that can change the
        // scan state. At top level only `. < # " ' \` matter — every other byte is copied
        // through. The body-of-the-whole-document terminator pre-scan walks each byte exactly
        // once, so this inner advance is pure critical-path latency; `memchr` runs it at
        // ~tens-of-GB/s instead of one byte/iteration. The structured handlers below
        // (string / IRI / comment skipping, the `.`-terminator and `\`-escape tests) are
        // UNCHANGED, so behaviour — including the review-1398 PN_LOCAL_ESC `\#` fix — is
        // identical; this only fast-forwards over the runs the old `_ => i += 1` arm crawled.
        let a = memchr::memchr3(b'.', b'<', b'#', &bytes[i..]);
        let b = memchr::memchr3(b'"', b'\'', b'\\', &bytes[i..]);
        i += match (a, b) {
            (None, None) => return None, // no interesting byte before EOF → incomplete statement
            (Some(x), None) => x,
            (None, Some(y)) => y,
            (Some(x), Some(y)) => x.min(y),
        };
        match bytes[i] {
            b'#' => {
                match memchr::memchr(b'\n', &bytes[i..]) {
                    Some(off) => i += off + 1,
                    None => i = n,
                }
            }
            b'<' => {
                i += 1;
                match memchr::memchr(b'>', &bytes[i..]) {
                    Some(off) => i += off + 1,
                    None => return None,
                }
            }
            q @ (b'"' | b'\'') => {
                let triple = i + 2 < n && bytes[i + 1] == q && bytes[i + 2] == q;
                if triple {
                    i += 3;
                    loop {
                        if i >= n {
                            return None;
                        }
                        if bytes[i] == b'\\' {
                            i += 2;
                        } else if bytes[i] == q && i + 2 < n && bytes[i + 1] == q && bytes[i + 2] == q {
                            i += 3;
                            break;
                        } else {
                            i += 1;
                        }
                    }
                } else {
                    i += 1;
                    while i < n && bytes[i] != q {
                        i += if bytes[i] == b'\\' { 2 } else { 1 };
                    }
                    if i >= n {
                        return None;
                    }
                    i += 1;
                }
            }
            b'.' => {
                match bytes.get(i + 1) {
                    None | Some(b' ' | b'\t' | b'\n' | b'\r' | b'#') => return Some(i + 1),
                    _ => i += 1,
                }
            }
            // [OPUS-4.8] PN_LOCAL_ESC (review 1398): a prefixed-name local can contain an
            // escaped reserved char, e.g. `ex:foo\#bar` or `ex:a\.b`. Skip the backslash AND
            // the escaped byte so an escaped `#` is NOT read as a comment start (which would
            // swallow a real `.` terminator and could scan past an interspersed
            // `@base`/`@prefix`, leaving a later chunk to parse under a stale preamble). Bail
            // to serial if the `\` is the last byte (malformed).
            b'\\' => {
                if i + 1 >= n {
                    return None;
                }
                i += 2;
            }
            _ => i += 1,
        }
    }
    None
}

/// Split Turtle `bytes` into independently-parseable chunks for parallel parsing, or `None` to
/// fall back to serial. Each chunk is `directive-snapshot + a run of statements`, so every
/// prefix/base in scope at that chunk's start is re-declared in the chunk.
///
/// # Directives anywhere (T3 — no longer a serial cliff), BOTH the `@`- and the SPARQL-style form
/// [OPUS-4.8] Directives are tracked as ordered byte-spans as the body is scanned, and each chunk
/// is prefixed with the **verbatim, in-order concatenation of every directive that precedes the
/// chunk's first statement** (joined by `\n`). This replays the document's directive prelude
/// exactly, so it is correct even for redefinitions and relative `@base <rel>` / `BASE <rel>`
/// (which resolve against the running base): oxttl re-processes the snapshot in the same order the
/// serial parse does, so each chunk sees the identical prefix table and base. A leading run of
/// directives (the common serializer shape) is just the degenerate case where the snapshot is the
/// same for every chunk. Previously ANY directive interspersed among the triples — and ANY
/// SPARQL-style `PREFIX`/`BASE` anywhere — dropped the WHOLE file to serial; that 0× cliff is gone
/// for both forms:
/// - Turtle `@prefix`/`@base`: `.`-terminated, delimited by [`next_terminator`].
/// - SPARQL-style `PREFIX`/`BASE`: NO `.` terminator, delimited at its IRIREF by
///   [`next_sparql_directive_end`]. (An invalid trailing `.` is left in the stream so the per-chunk
///   oxttl parse still rejects it — see that fn.)
///
/// Still bails (→ serial): anything the scanner cannot cleanly delimit (a truncated/malformed
/// directive or statement, which the serial oxttl parser then rejects).
///
/// BLANK NODES do not force a serial fallback. (They used to bail this function out — and 42
/// `_:` labels among the 1.5 M statements of the real Wikidata slice silently forfeited the
/// entire ~2.9× chunk-parallel speedup.) Document-scoped blank-node identity survives
/// chunk-independent parsing because the per-chunk partial dicts are merged into the global
/// [`Dict`], which interns blank nodes BY LABEL (`merge_remap` → `intern_blank`):
///
/// - LABELED `_:x`: oxttl preserves the written label verbatim
///   (`BlankNode::new_unchecked(label)`), so occurrences of `_:x` in *different* chunks intern
///   to the SAME global id — exactly the document-scoped unification the serial parser
///   produces, even for labels reused in arbitrarily distant statements. The dict merge IS the
///   shared label-intern map; no per-chunk label namespacing, boundary scanning, or post-pass
///   unification is needed.
/// - ANONYMOUS `[...]` / `(...)`: a nest is confined to one statement, hence to one chunk, so
///   anonymous nodes never need cross-chunk unification — only cross-chunk *distinctness*.
///   oxttl mints them via `BlankNode::default()`, a fresh random 128-bit id (thread-safe), so
///   distinctness holds with the same probabilistic guarantee the serial parser already relies
///   on WITHIN a single document (a fresh random id colliding with another anonymous id, or
///   with a user-written label, is equally improbable either way — parallelism adds no new
///   collision class, only more draws from the same 2^128 space).
///
/// The parallel result therefore equals the serial parse up to anonymous blank-node ids,
/// which differ run-to-run even between two serial parses.
#[cfg(feature = "parallel")]
fn turtle_chunks(bytes: &[u8], target: usize) -> Option<Vec<Vec<u8>>> {
    let n = bytes.len();

    // A parsed statement: its byte span `[start, end)` and how many `@`-directive spans precede
    // it (so its chunk's synthetic preamble = the in-order concatenation of `dirs[..dirs_before]`).
    struct Stmt {
        start: usize,
        end: usize,
        dirs_before: usize,
    }

    // [OPUS-4.8] (T3) Single pass over the top level: classify each unit as a directive (recorded
    // as an ordered byte-span) or a statement (recorded with the directive count in scope).
    // Directives — BOTH the Turtle `@prefix`/`@base` form (`.`-terminated) AND the SPARQL-style
    // `PREFIX`/`BASE` form (no `.`, delimited at its IRIREF by `next_sparql_directive_end`) — and
    // statements may interleave freely. Each chunk's synthetic preamble replays every directive in
    // scope verbatim, so a mid-body redefinition (either form) is correct.
    let mut dirs: Vec<(usize, usize)> = Vec::new();
    let mut stmts: Vec<Stmt> = Vec::new();
    let mut j = skip_ws_comments(bytes, 0);
    while j < n {
        if is_sparql_directive_start(bytes, j) {
            // SPARQL-style `PREFIX`/`BASE`: a directive span with no `.` terminator.
            let end = next_sparql_directive_end(bytes, j)?;
            dirs.push((j, end));
            j = skip_ws_comments(bytes, end);
            continue;
        }
        let is_directive = bytes[j] == b'@';
        let end = next_terminator(bytes, j)?;
        if is_directive {
            dirs.push((j, end));
        } else {
            stmts.push(Stmt { start: j, end, dirs_before: dirs.len() });
        }
        j = skip_ws_comments(bytes, end);
    }
    if stmts.len() < 2 {
        return None;
    }

    // Build each chunk's synthetic preamble lazily: it depends only on `dirs_before`, which is
    // monotonically non-decreasing in document order, so consecutive statements usually share it.
    // The preamble is the verbatim, in-order bytes of `dirs[..dirs_before]`, joined by `\n` so
    // adjacent directives never run together — an exact replay of the document's directive
    // prelude as seen at that point (correct for redefinitions and relative `@base`).
    let snapshot = |dirs_before: usize| -> Vec<u8> {
        let mut pre = Vec::new();
        for &(s, e) in &dirs[..dirs_before] {
            pre.extend_from_slice(&bytes[s..e]);
            pre.push(b'\n');
        }
        pre
    };

    // Partition the statements into ~target contiguous groups. A group must not straddle a
    // change in `dirs_before` (a directive appearing mid-group), or the statements after the
    // directive would parse under a stale snapshot — so split a group at any such change too.
    let per = (stmts.len() / target.max(1)).max(1);
    let mut chunks: Vec<Vec<u8>> = Vec::new();
    let mut idx = 0;
    while idx < stmts.len() {
        let dirs_before = stmts[idx].dirs_before;
        let group_start = stmts[idx].start;
        let mut end_i = (idx + per).min(stmts.len());
        // Shrink the group so every statement in it shares the same directive snapshot.
        // (Range bounds are snapshotted at loop entry, so the in-loop `end_i` write only
        // takes effect via the immediate `break`.)
        for (k, stmt) in stmts.iter().enumerate().take(end_i).skip(idx + 1) {
            if stmt.dirs_before != dirs_before {
                end_i = k;
                break;
            }
        }
        let body_end = stmts[end_i - 1].end;
        let mut chunk = snapshot(dirs_before);
        chunk.extend_from_slice(&bytes[group_start..body_end]);
        chunks.push(chunk);
        idx = end_i;
    }
    Some(chunks)
}

/// Parse Turtle in parallel by statement-boundary chunking (see [`turtle_chunks`]). If the input
/// is not safely splittable, OR any chunk fails to parse (an over-eager split), it falls back to
/// the serial parser — so the result always equals a plain serial Turtle parse, up to anonymous
/// blank-node ids (which differ run-to-run even between two serial parses; labeled blank nodes
/// and all ground terms are byte-identical).
#[cfg(feature = "parallel")]
fn parse_turtle_parallel(bytes: &[u8]) -> Result<(Dict, Vec<[Id; 3]>), String> {
    let threads = rayon::current_num_threads().max(1);
    if threads == 1 {
        // No parallelism available: chunking is pure overhead (measured ~16% at 1T on the
        // wikidata slice — 1.300 s chunked vs 1.12 s serial) — parse directly.
        let mut dict = Dict::new();
        return parse_turtle_chunk(bytes, &mut dict).map(|t| (dict, t));
    }
    let target = (threads * 4).min(bytes.len() / 8192 + 1).max(1);
    parse_turtle_chunked(bytes, target)
}

/// [`parse_turtle_parallel`] with an explicit chunk-count target — separated so the
/// differential tests can force small documents to fan out.
#[cfg(feature = "parallel")]
fn parse_turtle_chunked(bytes: &[u8], target: usize) -> Result<(Dict, Vec<[Id; 3]>), String> {
    use rayon::prelude::*;
    let serial = || {
        let mut dict = Dict::new();
        parse_turtle_chunk(bytes, &mut dict).map(|t| (dict, t))
    };
    let chunks = match turtle_chunks(bytes, target) {
        Some(c) if c.len() > 1 => c,
        _ => return serial(),
    };
    type Partials = Vec<(Dict, Vec<[Id; 3]>)>;
    let partials: Result<Partials, String> = chunks
        .par_iter()
        .map(|chunk| {
            let mut dict = Dict::new();
            parse_turtle_chunk(chunk, &mut dict).map(|t| (dict, t))
        })
        .collect();
    let partials = match partials {
        Ok(p) => p,
        Err(_) => return serial(), // an over-eager split produced invalid Turtle — redo serially
    };
    let total: usize = partials.iter().map(|(_, t)| t.len()).sum();
    let cap = partials.iter().map(|(d, _)| d.len()).max().unwrap_or(0);
    let mut global = Dict::with_capacity(cap);
    let mut all = Vec::with_capacity(total);
    for (pd, ptriples) in partials {
        let remap = global.merge_remap(&pd);
        remap_extend(&mut all, ptriples, &remap);
    }
    Ok((global, all))
}

/// Streams N-Triples from `reader` in newline-aligned ~64 MiB blocks, parsing+interning
/// each block IN PARALLEL (the custom byte parser, per-block partial dicts merged into
/// the running `dict`), and spilling SPO runs — the parallel-parse path for the external
/// (billion-scale, bounded-memory) build.
///
/// DECOMPRESSION is PIPELINED onto its own thread feeding a bounded channel, so it OVERLAPS
/// parsing+spilling instead of running additively. For a `.bz2` ingest — where the (slow,
/// single-stream) decompress dominates wall-time — this hides the parse cost under the
/// decompress, the largest measured ingest win. At most a few 64 MiB blocks are in flight,
/// so memory stays bounded.
#[cfg(all(feature = "mmap", feature = "parallel"))]
fn build_external_ntriples_parallel<R: std::io::Read + Send>(
    reader: R,
    dict: &mut Dict,
    buf: &mut Vec<[Id; 3]>,
    runs: &mut Vec<std::path::PathBuf>,
    tmp: &std::path::Path,
    chunk: usize,
) -> Result<(), String> {
    use std::sync::mpsc::sync_channel;
    const BLOCK: usize = 64 << 20; // 64 MiB
    // [OPUS-4.8] (review 1357) Cache timing-enabled ONCE; the hot per-block merge/remap loop
    // then takes no `Instant::now()` and no atomic update when SPARQ_BUILD_TIMING is unset.
    let timing = build_timing::enabled();
    let (tx, rx) = sync_channel::<Vec<u8>>(3);
    // Parsed partials flow parse-thread -> merge (this thread). A small bound keeps memory
    // bounded (a couple of blocks' partials in flight) while letting the rayon PARSE of the
    // next block overlap the SERIAL dict-merge of the current one. Profiling showed the
    // merge (merge_remap + triple-remap, ~10.5s/50M) dominates the parallel parse (~5.2s),
    // and they previously ran sequentially per block; this 3-stage pipeline hides the parse.
    type Partials = Vec<(Dict, Vec<[Id; 3]>)>;
    let (ptx, prx) = sync_channel::<Partials>(2);

    std::thread::scope(|scope| -> Result<(), String> {
        // Stage 1 — decompress + read on its own thread, emitting newline-aligned blocks.
        let producer = scope.spawn(move || -> Result<(), String> {
            let mut reader = reader;
            let mut readbuf = vec![0u8; BLOCK];
            let mut carry: Vec<u8> = Vec::new();
            loop {
                // Fill the read buffer (a single read may return less than requested).
                let mut filled = 0;
                while filled < BLOCK {
                    let n = reader.read(&mut readbuf[filled..]).map_err(|e| e.to_string())?;
                    if n == 0 {
                        break;
                    }
                    filled += n;
                }
                if filled == 0 {
                    // EOF: a final line without a trailing newline lives in `carry`.
                    if !carry.is_empty() {
                        let _ = tx.send(std::mem::take(&mut carry));
                    }
                    return Ok(());
                }
                // Emit `carry + readbuf[..filled]` up to the last newline; carry the
                // remainder (a partial line split across the read boundary) to the next.
                let mut block = std::mem::take(&mut carry);
                block.extend_from_slice(&readbuf[..filled]);
                let cut = block.iter().rposition(|&b| b == b'\n').map_or(0, |p| p + 1);
                carry = block[cut..].to_vec();
                block.truncate(cut);
                if tx.send(block).is_err() {
                    return Ok(()); // a downstream stage errored and dropped the receiver
                }
            }
        });

        // Stage 2 — parse+intern each block in parallel (per-chunk local dicts, no shared
        // state), forwarding the partials to the merge stage. Concurrent with stage 3.
        let parser = scope.spawn(move || -> Result<(), String> {
            for block in rx {
                if block.is_empty() {
                    continue;
                }
                let partials = parse_block(&block)?;
                if ptx.send(partials).is_err() {
                    return Ok(()); // the merge stage errored and dropped the receiver
                }
            }
            Ok(())
        });

        // Stage 3 (this thread) — SERIAL dict-merge + triple-remap + spill; owns dict/buf/
        // runs. The id-assignment order is identical to the old sequential path (blocks
        // arrive in order; partials are in chunk order), so the output is byte-identical.
        for partials in prx {
            for (pd, ptriples) in partials {
                let t_merge = build_timing::start(timing);
                let remap = dict.merge_remap(&pd);
                build_timing::record(t_merge, &build_timing::MERGE_NS);
                let t_remap = build_timing::start(timing);
                let map = |id: Id| if id >= dict::INLINE_BASE { id } else { remap[(id - 1) as usize] };
                // Prefetch the remap gather DIST triples ahead — the large-global-dict gather
                // is the build-path bottleneck (per-ISA hint, correctness-neutral).
                let base = remap.as_ptr();
                for i in 0..ptriples.len() {
                    if i + 32 < ptriples.len() {
                        for &id in &ptriples[i + 32] {
                            if id < dict::INLINE_BASE {
                                // SAFETY: id-1 < remap.len(); prefetch is hint-only.
                                prefetch_read(unsafe { base.add((id - 1) as usize) });
                            }
                        }
                    }
                    let [s, p, o] = ptriples[i];
                    buf.push([map(s), map(p), map(o)]);
                    if buf.len() >= chunk {
                        extsort::spill_run(buf, runs, tmp).map_err(|e| e.to_string())?;
                    }
                }
                build_timing::record(t_remap, &build_timing::REMAP_NS);
            }
        }
        // Join parse first (it feeds stage 3 — surface a parse error), then the producer.
        parser.join().map_err(|_| "parse thread panicked".to_string())??;
        producer.join().map_err(|_| "decompression thread panicked".to_string())?
    })
}

/// SHARDED variant of the parallel ingest: same decompress→parse stages, but the merge stage
/// interns into a hash-sharded dictionary (`ShardedDict`) so the dominant dict work runs in
/// parallel across shards instead of through one serial `merge_remap`. Triples are spilled
/// with TEMPORARY sharded ids (`shard*STRIDE+local`); `ShardedDict::into_merged` + an
/// order-preserving `remap_perm_file` pass turn them into final dense ids after the sort.
#[cfg(all(feature = "mmap", feature = "parallel"))]
fn build_external_ntriples_sharded<R: std::io::Read + Send>(
    reader: R,
    sharded: &mut dict::ShardedDict,
    buf: &mut Vec<[Id; 3]>,
    runs: &mut Vec<std::path::PathBuf>,
    tmp: &std::path::Path,
    chunk: usize,
) -> Result<(), String> {
    use std::sync::mpsc::sync_channel;
    const BLOCK: usize = 64 << 20;
    // [OPUS-4.8] (review 1357) Cache timing-enabled once; copied into the merge/remap stages.
    let timing = build_timing::enabled();
    let (tx, rx) = sync_channel::<Vec<u8>>(3);
    type Partials = Vec<(Dict, Vec<[Id; 3]>)>;
    let (ptx, prx) = sync_channel::<Partials>(2);

    std::thread::scope(|scope| -> Result<(), String> {
        // Stage 1 — decompress (identical to the non-sharded pipeline).
        let producer = scope.spawn(move || -> Result<(), String> {
            let mut reader = reader;
            let mut readbuf = vec![0u8; BLOCK];
            let mut carry: Vec<u8> = Vec::new();
            loop {
                let mut filled = 0;
                while filled < BLOCK {
                    let n = reader.read(&mut readbuf[filled..]).map_err(|e| e.to_string())?;
                    if n == 0 {
                        break;
                    }
                    filled += n;
                }
                if filled == 0 {
                    if !carry.is_empty() {
                        let _ = tx.send(std::mem::take(&mut carry));
                    }
                    return Ok(());
                }
                let mut block = std::mem::take(&mut carry);
                block.extend_from_slice(&readbuf[..filled]);
                let cut = block.iter().rposition(|&b| b == b'\n').map_or(0, |p| p + 1);
                carry = block[cut..].to_vec();
                block.truncate(cut);
                if tx.send(block).is_err() {
                    return Ok(());
                }
            }
        });
        // Stage 2 — parse (identical).
        let parser = scope.spawn(move || -> Result<(), String> {
            for block in rx {
                if block.is_empty() {
                    continue;
                }
                let partials = parse_block(&block)?;
                if ptx.send(partials).is_err() {
                    return Ok(());
                }
            }
            Ok(())
        });
        // Stage 4 — triple REMAP + spill on its own thread, so it runs CONCURRENTLY with
        // stage 3's interning of the next batch (they were one serial stage — the measured
        // ~200 s/1 B "dict bucket"; now the critical path is max(intern, remap), not the
        // sum). The remap gather itself is rayon-parallel (indexed map into a scratch that
        // is appended at exact `chunk` boundaries, so the run files — and every downstream
        // byte — stay identical to the old serial loop's).
        type Batch = (Vec<(Dict, Vec<[Id; 3]>)>, Vec<Vec<Id>>);
        let (rtx, rrx) = sync_channel::<Batch>(1);
        let remapper = scope.spawn(move || -> Result<(), String> {
            use rayon::prelude::*;
            let mut scratch: Vec<[Id; 3]> = Vec::new();
            for (partials, remaps) in rrx {
                let t_remap = build_timing::start(timing);
                for (pidx, (_, ptriples)) in partials.iter().enumerate() {
                    let rm = &remaps[pidx];
                    let map = |id: Id| if id >= dict::INLINE_BASE { id } else { rm[id as usize] };
                    ptriples.par_iter().map(|&[s, p, o]| [map(s), map(p), map(o)]).collect_into_vec(&mut scratch);
                    let mut rest: &[[Id; 3]] = &scratch;
                    while !rest.is_empty() {
                        let take = (chunk - buf.len()).min(rest.len());
                        buf.extend_from_slice(&rest[..take]);
                        rest = &rest[take..];
                        if buf.len() >= chunk {
                            extsort::spill_run(buf, runs, tmp).map_err(|e| e.to_string())?;
                        }
                    }
                }
                build_timing::record(t_remap, &build_timing::REMAP_NS);
            }
            Ok(())
        });
        // Stage 3 (this thread) — SHARDED merge: route each partial's (non-inline) terms to
        // shards and intern in parallel (component-based, no Term alloc; the routing and the
        // remap-table scatter are parallel too — see `ShardedDict::intern_partials`), then
        // hand the batch to stage 4 for the triple remap. Batches flow IN ORDER, so id
        // assignment is deterministic and identical to the previous serial-stage version.
        let feed = || -> Result<(), String> {
            for partials in prx {
                let t_merge = build_timing::start(timing);
                let remaps = sharded.intern_partials(&partials);
                build_timing::record(t_merge, &build_timing::MERGE_NS);
                if rtx.send((partials, remaps)).is_err() {
                    return Ok(()); // the remap stage errored and dropped the receiver
                }
            }
            Ok(())
        };
        let fed = feed();
        drop(rtx); // close the channel so stage 4 drains and exits
        remapper.join().map_err(|_| "remap thread panicked".to_string())??;
        fed?;
        parser.join().map_err(|_| "parse thread panicked".to_string())??;
        producer.join().map_err(|_| "decompression thread panicked".to_string())?
    })
}

/// Rewrite a permutation file in place, remapping every temporary sharded id to its final
/// dense id (`dict::remap_sharded`). Order-preserving (temp ids sort like final ids), so the
/// already-sorted, deduplicated file stays sorted — no re-sort needed.
#[cfg(all(feature = "mmap", feature = "parallel"))]
fn remap_perm_file(path: &std::path::Path, base: &[u64], stride: u32) -> Result<(), String> {
    let f = std::fs::OpenOptions::new().read(true).write(true).open(path).map_err(|e| e.to_string())?;
    let len = f.metadata().map_err(|e| e.to_string())?.len() as usize;
    if len == 0 {
        return Ok(());
    }
    // SAFETY: read-write mapping of a freshly-written perm file of whole [u32;3] rows.
    let mut mmap = unsafe { memmap2::MmapMut::map_mut(&f) }.map_err(|e| e.to_string())?;
    let ids: &mut [u32] = unsafe { std::slice::from_raw_parts_mut(mmap.as_mut_ptr().cast::<u32>(), len / 4) };
    use rayon::prelude::*;
    ids.par_iter_mut().for_each(|id| *id = dict::remap_sharded(*id, base, stride));
    mmap.flush().map_err(|e| e.to_string())?;
    Ok(())
}

/// SPILLED-dict variant of the parallel N-Triples ingest: the same decompress→parse
/// stages as the sharded pipeline, but stage 3 routes terms through the BOUNDED spill
/// interner (`dictspill::SpillInterner::intern_batch`) instead of in-RAM shard dicts,
/// and triples are STAGED to disk unsorted (their final ids are unknown until the
/// external dedup completes) rather than spilled as sorted runs. Batches flow IN ORDER,
/// so per-shard first-occurrence order — and the final id assignment — is identical to
/// the sharded in-RAM path.
#[cfg(feature = "dict-spill")]
fn build_external_ntriples_dictspill<R: std::io::Read + Send>(
    reader: R,
    interner: &mut dictspill::SpillInterner,
) -> Result<(), String> {
    use std::sync::mpsc::sync_channel;
    // 16 MiB blocks (vs the sharded path's 64 MiB): the in-flight partial dicts (up to
    // ~3 batches across the two channels) dominate the spill path's RSS FLOOR in the
    // unique-literal regime, and the id assignment is CHUNKING-INVARIANT — per-shard
    // first-occurrence order depends only on the (line, position) order of first
    // occurrences, not on where block/partial boundaries fall — so smaller blocks trade
    // nothing but per-batch overhead. (The differential test builds the reference with
    // the sharded path's 64 MiB blocks, so it verifies this invariance empirically.)
    const BLOCK: usize = 16 << 20;
    // [OPUS-4.8] (review 1357) Cache timing-enabled once for the merge stage below.
    let timing = build_timing::enabled();
    let (tx, rx) = sync_channel::<Vec<u8>>(3);
    type Partials = Vec<(Dict, Vec<[Id; 3]>)>;
    let (ptx, prx) = sync_channel::<Partials>(2);

    std::thread::scope(|scope| -> Result<(), String> {
        // Stage 1 — decompress (identical to the sharded pipeline).
        let producer = scope.spawn(move || -> Result<(), String> {
            let mut reader = reader;
            let mut readbuf = vec![0u8; BLOCK];
            let mut carry: Vec<u8> = Vec::new();
            loop {
                let mut filled = 0;
                while filled < BLOCK {
                    let n = reader.read(&mut readbuf[filled..]).map_err(|e| e.to_string())?;
                    if n == 0 {
                        break;
                    }
                    filled += n;
                }
                if filled == 0 {
                    if !carry.is_empty() {
                        let _ = tx.send(std::mem::take(&mut carry));
                    }
                    return Ok(());
                }
                let mut block = std::mem::take(&mut carry);
                block.extend_from_slice(&readbuf[..filled]);
                let cut = block.iter().rposition(|&b| b == b'\n').map_or(0, |p| p + 1);
                carry = block[cut..].to_vec();
                block.truncate(cut);
                if tx.send(block).is_err() {
                    return Ok(());
                }
            }
        });
        // Stage 2 — parse (identical).
        let parser = scope.spawn(move || -> Result<(), String> {
            for block in rx {
                if block.is_empty() {
                    continue;
                }
                let partials = parse_block(&block)?;
                if ptx.send(partials).is_err() {
                    return Ok(());
                }
            }
            Ok(())
        });
        // Stage 3 (this thread) — bounded-cache resolve + triple staging.
        for partials in prx {
            let t_merge = build_timing::start(timing);
            interner.intern_batch(&partials)?;
            build_timing::record(t_merge, &build_timing::MERGE_NS);
        }
        parser.join().map_err(|_| "parse thread panicked".to_string())??;
        producer.join().map_err(|_| "decompression thread panicked".to_string())?
    })
}

/// Parses one (complete-line) N-Triples byte block in parallel into per-chunk partial
/// dictionaries + local-id triples (no shared state) — the parallelizable half of ingest.
#[cfg(feature = "parallel")]
fn parse_block(bytes: &[u8]) -> Result<ChunkPartials, String> {
    use rayon::prelude::*;
    let target = (rayon::current_num_threads().max(1) * 4).min(bytes.len() / 4096 + 1);
    let bounds = newline_chunk_bounds(bytes, target);
    // [OPUS-4.8] (review 1357) Only timestamp this phase when timing is enabled — one env read
    // per ~16-64 MiB block (not per triple) and no clock/atomic on the common unset path.
    let t_parse = build_timing::start(build_timing::enabled());
    let partials: Vec<(Dict, Vec<[Id; 3]>)> = bounds
        .par_iter()
        .map(|&(s, e)| {
            let mut d = Dict::new();
            let t = nt::parse_chunk(&bytes[s..e], &mut d)?;
            Ok::<_, String>((d, t))
        })
        .collect::<Result<Vec<_>, _>>()?;
    build_timing::record(t_parse, &build_timing::PARSE_NS);
    Ok(partials)
}

/// Env-gated (`SPARQ_BUILD_TIMING`) phase-time accumulators for the parallel ingest path,
/// to attribute wall-time across parallel-parse vs dict-merge vs triple-remap.
#[cfg(feature = "parallel")]
mod build_timing {
    // clippy/dead_code: this build-phase timing helper is consumed only by the
    // `mmap`/`dict-spill` external-build paths; in the default feature set those callers
    // are cfg'd out, leaving some members unused. They are not dead under those features.
    #![allow(dead_code)]
    use std::sync::atomic::AtomicU64;
    pub static PARSE_NS: AtomicU64 = AtomicU64::new(0);
    pub static MERGE_NS: AtomicU64 = AtomicU64::new(0);
    pub static REMAP_NS: AtomicU64 = AtomicU64::new(0);
    pub fn reset() {
        use std::sync::atomic::Ordering::Relaxed;
        PARSE_NS.store(0, Relaxed);
        MERGE_NS.store(0, Relaxed);
        REMAP_NS.store(0, Relaxed);
    }
    pub fn enabled() -> bool {
        std::env::var("SPARQ_BUILD_TIMING").is_ok()
    }
    /// [OPUS-4.8] (review 1357) Take a phase start timestamp only when timing is enabled, so
    /// the hot ingest path performs NO `Instant::now()` clock read when `SPARQ_BUILD_TIMING`
    /// is unset (the common case). Callers cache `enabled()` once per build and pass the flag.
    #[inline]
    pub fn start(timing: bool) -> Option<std::time::Instant> {
        timing.then(std::time::Instant::now)
    }
    /// Accumulate the elapsed nanos for a phase iff its start was taken (timing enabled),
    /// skipping the atomic `fetch_add` entirely otherwise.
    #[inline]
    pub fn record(start: Option<std::time::Instant>, counter: &AtomicU64) {
        if let Some(t) = start {
            counter.fetch_add(t.elapsed().as_nanos() as u64, std::sync::atomic::Ordering::Relaxed);
        }
    }
    pub fn report(stage: &str, secs: f64) {
        use std::sync::atomic::Ordering::Relaxed;
        let (p, m, r) = (
            PARSE_NS.load(Relaxed) as f64 / 1e9,
            MERGE_NS.load(Relaxed) as f64 / 1e9,
            REMAP_NS.load(Relaxed) as f64 / 1e9,
        );
        eprintln!(
            "[build-timing] {stage}: parse(parallel) {p:.2}s | merge_remap(serial) {m:.2}s | triple-remap(serial) {r:.2}s | {secs:.2}s wall to here"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "parallel")]
    #[test]
    fn parallel_turtle_matches_serial() {
        let decoded = |d: &Dict, t: &[[Id; 3]]| -> Vec<String> {
            let mut v: Vec<String> =
                t.iter().map(|&[s, p, o]| format!("{}|{}|{}", d.term(s), d.term(p), d.term(o))).collect();
            v.sort();
            v
        };
        // Blank-node-FREE doc (exercises the parallel statement-split path), >8 KiB so it fans
        // out, with edge cases that stress the terminator scan: decimals, dots inside strings &
        // IRIs, escaped quotes, multi-line ;/, statements, triple-quoted strings with `.`+
        // newlines, and trailing comments.
        let mut ttl = String::from(
            "@prefix : <http://ex/> .\n@prefix ex: <http://example.org/foo.bar#> .\n# header . comment\n",
        );
        for i in 0..500 {
            ttl.push_str(&format!(
                ":s{i} :dec {i}.5 ; :s \"a.b.c\" , \"x\\\"y.z\" ; :iri ex:rel{i} .\n\
                 :s{i} ex:m \"\"\"l1 . still\nl2.\"\"\" ; :p <http://x.y/a.b.{i}> . # trailing . c\n",
            ));
        }
        assert!(ttl.len() > 8192);
        assert!(turtle_chunks(ttl.as_bytes(), 32).is_some(), "blank-node-free doc should fan out");
        let (pd, pt) = parse_turtle_parallel(ttl.as_bytes()).unwrap();
        let mut sd = Dict::new();
        let st = parse_turtle_chunk(ttl.as_bytes(), &mut sd).unwrap();
        assert_eq!(decoded(&pd, &pt), decoded(&sd, &st), "parallel split must equal serial");
        assert!(pt.len() >= 1500);

        // Blank-node docs fan out too (the dict merge unifies labels by term equality — see
        // turtle_chunks). The differential coverage lives in
        // parallel_turtle_bnodes_match_serial; here just pin that the splitter no longer bails.
        let bn = format!(
            "@prefix : <http://ex/> .\n{}",
            ":a :p [ :q :r ] .\n:x :y ( :i1 :i2 ) .\n_:b :z :w .\n".repeat(300)
        );
        assert!(turtle_chunks(bn.as_bytes(), 32).is_some(), "blank nodes must no longer bail to serial");
    }

    /// Decodes a parsed triple sequence to strings with blank nodes renumbered by FIRST
    /// OCCURRENCE in document order. This makes serial-vs-parallel comparison EXACT (plain
    /// `assert_eq!`) rather than requiring graph-isomorphism search: the chunked parse
    /// preserves document order (chunks merge in order), labeled blank nodes keep their
    /// written labels 1:1 in both parses, and anonymous blank nodes are minted at the same
    /// document positions — so the two parses are equivalent under a 1:1 blank-node
    /// relabeling IFF these canonical sequences are equal (the first-occurrence renumbering
    /// is itself the witness mapping).
    #[cfg(feature = "parallel")]
    fn canon_bnodes(d: &Dict, t: &[[Id; 3]]) -> Vec<[String; 3]> {
        let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        let mut out = Vec::with_capacity(t.len());
        for &[s, p, o] in t {
            let mut conv = |id: Id| match d.term(id) {
                Term::BlankNode(b) => {
                    let next = seen.len();
                    format!("_:c{}", *seen.entry(b.as_str().to_owned()).or_insert(next))
                }
                other => other.to_string(),
            };
            out.push([conv(s), conv(p), conv(o)]);
        }
        out
    }

    /// Differential tests for the parallel Turtle path on BLANK-NODE documents — the inputs
    /// that previously forced a serial fallback. Each case asserts the doc actually splits
    /// into >1 chunk and that the chunked parse equals the serial parse after canonical
    /// blank-node renumbering (see [`canon_bnodes`]).
    #[cfg(feature = "parallel")]
    #[test]
    fn parallel_turtle_bnodes_match_serial() {
        let differential = |ttl: &str, target: usize| {
            let chunks = turtle_chunks(ttl.as_bytes(), target).expect("doc must fan out");
            assert!(chunks.len() > 1, "doc must split into multiple chunks");
            let (pd, pt) = parse_turtle_chunked(ttl.as_bytes(), target).unwrap();
            let mut sd = Dict::new();
            let st = parse_turtle_chunk(ttl.as_bytes(), &mut sd).unwrap();
            assert_eq!(
                canon_bnodes(&pd, &pt),
                canon_bnodes(&sd, &st),
                "chunked parse must equal serial up to anonymous bnode ids"
            );
        };

        // 1. Same LABEL in distant statements (first and last, crossing every chunk
        //    boundary) plus labels shared between adjacent statements: the merge must
        //    unify each label to ONE node, exactly like the serial document-scoped parse.
        let mut ttl = String::from("@prefix : <http://ex/> .\n_:shared :starts :here .\n");
        for i in 0..400 {
            ttl.push_str(&format!(":s{i} :p :o{i} .\n_:b{} :links _:b{} .\n", i / 3, i / 3 + 1));
        }
        ttl.push_str("_:shared :ends :here .\n");
        differential(&ttl, 16);

        // 2. Anonymous nests + collections: fresh ids must stay distinct across chunks, and
        //    each nest's internal structure must survive chunking intact.
        let mut anon = String::from("@prefix : <http://ex/> .\n");
        for i in 0..300 {
            anon.push_str(&format!(
                ":r{i} :has [ :p{i} [ :q \"v{i}.w\" ] ; :list ( 1 2.5 \"three\" ) ] .\n"
            ));
        }
        differential(&anon, 16);

        // 3. Pathological ALL-bnode doc: every subject/object a labeled bnode, chained
        //    across consecutive statements so every chunk boundary cuts a shared label.
        let mut chain = String::from("@prefix : <http://ex/> .\n");
        for i in 0..500 {
            chain.push_str(&format!("_:n{i} :next _:n{} .\n", i + 1));
        }
        differential(&chain, 16);

        // 4. The Wikidata shape that motivated the fix: a handful of labeled bnodes sprinkled
        //    through a large ground-statement body (formerly: any one of them forfeited the
        //    whole parallel parse).
        let mut sparse = String::from("@prefix : <http://ex/> .\n");
        for i in 0..600 {
            if i % 97 == 0 {
                sparse.push_str(&format!(":s{i} :somevalue _:sv{} .\n", i / 97));
            } else {
                sparse.push_str(&format!(":s{i} :p :o{i} .\n"));
            }
        }
        differential(&sparse, 16);

        // 5. Bnode-free control through the same harness (the pre-existing fan-out case).
        let mut ground = String::from("@prefix : <http://ex/> .\n");
        for i in 0..400 {
            ground.push_str(&format!(":s{i} :p \"lit {i}\" ; :q {i}.5 .\n"));
        }
        differential(&ground, 16);
    }

    /// [OPUS-4.8] Regression for review 1398: a PN_LOCAL_ESC `\#` in a prefixed-name local
    /// must not be read as a comment start by `next_terminator`. Before the fix, the escaped
    /// `#` swallowed the rest of its line (including the real `.` terminator), so the scanner
    /// could run through an interspersed `@base`/`@prefix` undetected; a later chunk would then
    /// parse under the stale preamble and resolve relative IRIs wrongly. The fix makes the
    /// scanner skip the escaped byte so an interspersed `@base` is seen at its true position —
    /// pre-T3 that meant a serial fallback; post-T3 (directive-snapshot per chunk) it means the
    /// `@base` is correctly attributed to the statements that follow it.
    #[cfg(feature = "parallel")]
    #[test]
    fn parallel_turtle_escaped_hash_in_local() {
        // (a) Escaped `#` in a local does not eat the terminator: the splitter still scans the
        //     statements cleanly and the chunked parse equals the serial parse.
        let mut clean = String::from("@prefix ex: <http://ex/> .\n");
        for i in 0..400 {
            // `ex:foo\#bar{i}` — a valid PN_LOCAL with an escaped `#`. The `\#` must NOT start
            // a comment; the `.` after it is the real top-level terminator.
            clean.push_str(&format!("ex:s{i} ex:p ex:foo\\#bar{i} .\n"));
        }
        assert!(clean.len() > 8192);
        // The escaped-`#` doc must still split (no spurious comment-eating of terminators)…
        assert!(turtle_chunks(clean.as_bytes(), 32).is_some(), "escaped # must not break the split");
        // …and parse identically to the serial parser.
        let (pd, pt) = parse_turtle_chunked(clean.as_bytes(), 32).unwrap();
        let mut sd = Dict::new();
        let st = parse_turtle_chunk(clean.as_bytes(), &mut sd).unwrap();
        assert_eq!(canon_bnodes(&pd, &pt), canon_bnodes(&sd, &st), "escaped-# chunked parse must equal serial");
        assert_eq!(pt.len(), 400);

        // (b) [OPUS-4.8] (T3) An interspersed `@base` AFTER an escaped `#` (with enough
        //     statements to force multiple chunks) must be attributed to its FOLLOWING
        //     statements, which use a RELATIVE IRI that resolves against the new base. The
        //     escaped `#` must not let the scanner run through the `@base` (review 1398) — if it
        //     did, the relative IRIs would resolve against the stale `@base <http://first/>`.
        //     The check is the load-bearing one: chunked == serial. The base is also redefined a
        //     second time so the snapshot must carry BOTH `@base`s in order.
        let mut interspersed = String::from("@prefix ex: <http://ex/> .\n@base <http://first/> .\n");
        for i in 0..150 {
            interspersed.push_str(&format!("ex:s{i} ex:p <rel{i}> ; ex:e ex:foo\\#bar{i} .\n"));
        }
        interspersed.push_str("@base <http://second/> .\n");
        for i in 150..300 {
            interspersed.push_str(&format!("<sub{i}> ex:p <rel{i}> .\n"));
        }
        interspersed.push_str("@base <http://third/> .\n");
        for i in 300..450 {
            interspersed.push_str(&format!("<sub{i}> ex:p <rel{i}> .\n"));
        }
        let chunks = turtle_chunks(interspersed.as_bytes(), 32)
            .expect("T3: interspersed @base no longer forces serial");
        assert!(chunks.len() > 1, "must fan out across the interspersed @base");
        let (pd, pt) = parse_turtle_chunked(interspersed.as_bytes(), 32).unwrap();
        let mut sd = Dict::new();
        let st = parse_turtle_chunk(interspersed.as_bytes(), &mut sd).unwrap();
        assert_eq!(
            canon_bnodes(&pd, &pt),
            canon_bnodes(&sd, &st),
            "interspersed-@base chunked parse must equal serial (relative IRIs resolve against the right base)"
        );
    }

    /// [OPUS-4.8] (T3) Interspersed `@prefix`/`@base` directives among the body statements must
    /// no longer drop the whole document to serial — instead each chunk carries a verbatim
    /// in-order snapshot of the directives in scope at its start, so the chunked parse equals
    /// serial even for prefix REDEFINITIONS. SPARQL-style `PREFIX`/`BASE` (the no-`.` keyword
    /// form) is handled the same way (delimited at its IRIREF) — see case 4.
    #[cfg(feature = "parallel")]
    #[test]
    fn parallel_turtle_interspersed_directives_match_serial() {
        let differential = |ttl: &str, target: usize, expect_fanout: bool| {
            let chunks = turtle_chunks(ttl.as_bytes(), target);
            if expect_fanout {
                let c = chunks.expect("T3: interspersed @-directives must fan out, not bail");
                assert!(c.len() > 1, "must split into multiple chunks");
            }
            let (pd, pt) = parse_turtle_chunked(ttl.as_bytes(), target).unwrap();
            let mut sd = Dict::new();
            let st = parse_turtle_chunk(ttl.as_bytes(), &mut sd).unwrap();
            assert_eq!(
                canon_bnodes(&pd, &pt),
                canon_bnodes(&sd, &st),
                "interspersed-directive chunked parse must equal serial"
            );
        };

        // 1. Prefix REDEFINITION mid-body: the SAME prefix `p:` is rebound several times, so each
        //    block of statements must see the binding in scope at its position. A stale snapshot
        //    would expand `p:x` to the wrong IRI — caught because chunked must equal serial.
        let mut redef = String::from("@prefix p: <http://v0/> .\n");
        for round in 0..6 {
            redef.push_str(&format!("@prefix p: <http://v{round}/> .\n"));
            for i in 0..80 {
                redef.push_str(&format!("p:s{round}_{i} p:p p:o{i} .\n"));
            }
        }
        assert!(redef.len() > 8192);
        differential(&redef, 32, true);

        // 2. Relative `@base` redefinition mid-body with relative-IRI subjects/objects that
        //    resolve against the running base; new prefixes appear partway through too.
        let mut mixed = String::from("@base <http://b0/> .\n@prefix a: <http://a/> .\n");
        for i in 0..100 {
            mixed.push_str(&format!("<s{i}> a:p <o{i}> .\n"));
        }
        mixed.push_str("@base <http://b1/> .\n@prefix b: <http://bb/> .\n");
        for i in 100..200 {
            mixed.push_str(&format!("<s{i}> a:p b:o{i} .\n"));
        }
        differential(&mixed, 16, true);

        // 3. A directive immediately adjacent to statements with no blank lines, and a directive
        //    as the very last top-level unit before EOF (no following statement) — the trailing
        //    directive simply contributes to no later chunk.
        let mut adjacent = String::from("@prefix x: <http://x/> .\n");
        for i in 0..60 {
            adjacent.push_str(&format!("x:s{i} x:p x:o{i} .\n"));
        }
        adjacent.push_str("@prefix y: <http://y/> .\nx:last y:p x:o .\n@prefix z: <http://z/> .\n");
        differential(&adjacent, 8, false);

        // 4. SPARQL-style `PREFIX`/`BASE` (no-`.` keyword form) interspersed and REDEFINED
        //    mid-body — must now FAN OUT (no longer the serial cliff) and match serial. The
        //    snapshot replays each SPARQL directive verbatim, so the binding in scope at each
        //    chunk's start is the right one. Includes a relative SPARQL `BASE <rel>` redefinition
        //    resolved against the running base, and prefixes introduced partway through.
        let mut sparql = String::from("PREFIX s: <http://s0/>\nBASE <http://base0/>\n");
        for round in 0..8 {
            sparql.push_str(&format!("PREFIX s: <http://s{round}/>\nBASE <http://base{round}/>\n"));
            for i in 0..120 {
                sparql.push_str(&format!("s:longkey{round}_{i} s:longpred <relative-iri-{i}> .\n"));
            }
        }
        assert!(sparql.len() > 8192);
        differential(&sparql, 32, true);

        // 5. MIXED `@`-form and SPARQL-style directives in the SAME document, interleaved with
        //    statements — both forms must be tracked in the same ordered snapshot.
        let mut mixedforms = String::from("@prefix a: <http://a/> .\nPREFIX b: <http://b/>\n");
        for i in 0..80 {
            mixedforms.push_str(&format!("a:s{i} b:p a:o{i} .\n"));
        }
        mixedforms.push_str("@base <http://base/> .\nPREFIX c: <http://c/>\n");
        for i in 80..160 {
            mixedforms.push_str(&format!("<s{i}> b:p c:o{i} .\n"));
        }
        differential(&mixedforms, 16, true);

        // 6. A `#`-comment containing `<` and `.` sitting BETWEEN the SPARQL keyword and its
        //    IRIREF — the directive delimiter must skip the whole comment line (so the in-comment
        //    `<`/`.` is never mistaken for the IRIREF / a terminator) and find the real IRIREF.
        let mut commented = String::from("PREFIX p: # a comment with < and . inside\n  <http://c0/>\n");
        for round in 0..6 {
            commented.push_str(&format!(
                "PREFIX p: #redef <bogus> .\n <http://c{round}/>\nBASE # base <x> .\n <http://b{round}/>\n"
            ));
            for i in 0..50 {
                commented.push_str(&format!("p:s{round}_{i} p:p <rel{i}> .\n"));
            }
        }
        differential(&commented, 16, true);
    }

    /// [OPUS-4.8] (T2/T3 rejection parity) Malformed Turtle must still be REJECTED — the
    /// parallel split is never allowed to silently accept invalid input. Whether the splitter
    /// fans out (then a chunk fails oxttl → serial redo) or bails directly, `parse_turtle_chunked`
    /// must return `Err`, exactly as the serial oxttl parser does. An over-eager split that
    /// accidentally parsed invalid Turtle as valid is the one failure the chunked-vs-serial
    /// oracle cannot catch, so it is pinned here directly.
    #[cfg(feature = "parallel")]
    #[test]
    fn parallel_turtle_rejects_malformed() {
        // Each is invalid Turtle for a distinct reason; size them past the 2-statement minimum so
        // the splitter actually engages where it can.
        let bad: &[&str] = &[
            // Unknown prefix used in the body (the body chunk must fail to parse).
            "@prefix ex: <http://ex/> .\nex:s ex:p ex:o .\nbad:s bad:p bad:o .\n",
            // Missing terminator on a statement.
            "@prefix ex: <http://ex/> .\nex:s ex:p ex:o\nex:s2 ex:p ex:o2 .\n",
            // An interspersed @prefix that is itself malformed (no IRI).
            "@prefix ex: <http://ex/> .\nex:s ex:p ex:o .\n@prefix bad .\nex:s2 ex:p ex:o2 .\n",
            // A statement that uses a prefix declared only LATER (Turtle requires prior decl) —
            // the snapshot must NOT retro-actively make it valid.
            "@prefix a: <http://a/> .\na:s a:p later:o .\n@prefix later: <http://l/> .\na:s2 a:p a:o .\n",
            // Bare unescaped control / illegal token sequence.
            "@prefix ex: <http://ex/> .\nex:s ex:p \"unterminated .\nex:s2 ex:p ex:o2 .\n",
            // [OPUS-4.8] (T3 SPARQL-form rejection parity) SPARQL-style `BASE` with an illegal
            // trailing `.` (W3C turtle-syntax-bad-base-03). The SPARQL-directive splitter ends the
            // span at the `>` and leaves the `.` in the stream; the chunk that then begins with `.`
            // must fail oxttl → serial redo → reject. A splitter that swallowed the `.` would
            // silently ACCEPT this invalid document — the over-eager-split corruption class.
            "BASE <http://b/> .\n<s> <http://b/p> <http://b/o> .\n<s2> <http://b/p> <http://b/o2> .\n",
            // SPARQL-style `PREFIX` with an illegal trailing `.`.
            "PREFIX p: <http://b/> .\np:s p:p p:o .\np:s2 p:p p:o2 .\n",
            // SPARQL-style `PREFIX` whose prefixed name is used in the body but the directive's
            // IRIREF is missing (truncated) — must reject, not be silently snapshotted.
            "PREFIX p: badnoiriref\np:s p:p p:o .\np:s2 p:p p:o2 .\n",
        ];
        for (k, src) in bad.iter().enumerate() {
            // Reference: the serial oxttl parser rejects it.
            let mut sd = Dict::new();
            assert!(
                parse_turtle_chunk(src.as_bytes(), &mut sd).is_err(),
                "case {k} should be invalid Turtle (serial oxttl)"
            );
            // The parallel path must reject it too (fan-out + chunk-fail-serial-redo, or a direct
            // bail then serial), at several fan-out targets.
            for target in [1usize, 2, 8, 32] {
                assert!(
                    parse_turtle_chunked(src.as_bytes(), target).is_err(),
                    "case {k} target {target}: parallel path must reject malformed Turtle"
                );
            }
        }
    }

    /// [OPUS-4.8] (B4 — sparq-turtle-path REJECTION ORACLE) The public `Graph::parse_to_triples`
    /// `"turtle"` path MUST reject malformed Turtle, and reject EXACTLY what the oxttl serial
    /// parser rejects (oxttl is the differential reference). This is the load-bearing gate for the
    /// T1 borrowed-slice interner spike: T1 changes how `parse_turtle_chunk` interns oxttl's
    /// already-parsed terms, so it cannot itself accept invalid syntax — but the chunked-vs-serial
    /// byte-identity oracle (`parallel_turtle_bnodes_match_serial`) only proves chunked ≡ serial,
    /// NOT that either rejects what the spec/oxttl rejects. An over-eager accept would be a silent
    /// corruption invisible to that oracle. So we pin REJECTION here directly, on the PUBLIC entry
    /// point (`parse_to_triples`, which dispatches to `parse_turtle_parallel` under `parallel`),
    /// for a dozen-plus malformed inputs spanning the categories the brief calls out: bad IRIs,
    /// bad escapes, unterminated strings, bad prefixes, plus structural errors.
    ///
    /// The assertion is DIFFERENTIAL, not just "sparq errors": for each input we first confirm the
    /// oxttl serial reference rejects it (so the corpus stays honest as oxttl evolves), then assert
    /// the public sparq turtle path rejects it too. Positive controls confirm the harness is not
    /// vacuously rejecting everything.
    #[test]
    fn turtle_path_rejection_oracle() {
        // Differential reference: serial oxttl (the same parser the chunk worker drives).
        let oxttl_serial_rejects = |src: &str| -> bool {
            oxttl::TurtleParser::new()
                .for_slice(src.as_bytes())
                .any(|r| r.is_err())
        };

        // Malformed Turtle, one invalid reason each. A leading valid statement is included where
        // it helps the parallel splitter actually engage (so the chunk path, not only the 1T
        // direct parse, is exercised by the same corpus when run under --features parallel).
        let bad: &[(&str, &str)] = &[
            // ---- bad prefixes / prefixed names ----
            ("undeclared prefix in body",
             "@prefix ex: <http://ex/> .\nex:s ex:p ex:o .\nbad:s bad:p bad:o .\n"),
            ("@prefix without an IRI",
             "@prefix ex: <http://ex/> .\nex:s ex:p ex:o .\n@prefix bad .\nex:s2 ex:p ex:o2 .\n"),
            ("prefix used before its declaration",
             "@prefix a: <http://a/> .\na:s a:p later:o .\n@prefix later: <http://l/> .\na:s2 a:p a:o .\n"),
            ("@prefix missing the colon",
             "@prefix ex <http://ex/> .\nex:s ex:p ex:o .\n"),
            // ---- bad IRIs ----
            ("unterminated IRI ref (no closing >)",
             "@prefix ex: <http://ex/> .\nex:s ex:p <http://no-close .\nex:s2 ex:p ex:o2 .\n"),
            ("space inside an IRI ref",
             "<http://ex/ s> <http://ex/p> <http://ex/o> .\n"),
            // ---- bad string escapes ----
            ("invalid string escape \\q",
             "@prefix ex: <http://ex/> .\nex:s ex:p \"bad \\q escape\" .\nex:s2 ex:p ex:o2 .\n"),
            ("truncated \\u escape (too few hex digits)",
             "@prefix ex: <http://ex/> .\nex:s ex:p \"bad \\u12\" .\nex:s2 ex:p ex:o2 .\n"),
            // ---- unterminated strings ----
            ("unterminated single-quoted string",
             "@prefix ex: <http://ex/> .\nex:s ex:p \"unterminated .\nex:s2 ex:p ex:o2 .\n"),
            ("unterminated triple-quoted string",
             "@prefix ex: <http://ex/> .\nex:s ex:p \"\"\"open\nstill open .\n"),
            // ---- structural ----
            ("missing statement terminator",
             "@prefix ex: <http://ex/> .\nex:s ex:p ex:o\nex:s2 ex:p ex:o2 .\n"),
            ("predicate with no object",
             "@prefix ex: <http://ex/> .\nex:s ex:p .\nex:s2 ex:p ex:o2 .\n"),
            ("literal in subject position",
             "@prefix ex: <http://ex/> .\n\"lit\" ex:p ex:o .\n"),
            ("bad numeric literal (double dot)",
             "@prefix ex: <http://ex/> .\nex:s ex:p 1.2.3 .\nex:s2 ex:p ex:o2 .\n"),
            ("unmatched [ property-list",
             "@prefix ex: <http://ex/> .\nex:s ex:p [ ex:q ex:r .\nex:s2 ex:p ex:o2 .\n"),
            ("unmatched ( collection",
             "@prefix ex: <http://ex/> .\nex:s ex:p ( ex:a ex:b .\nex:s2 ex:p ex:o2 .\n"),
        ];

        for (why, src) in bad {
            assert!(
                oxttl_serial_rejects(src),
                "corpus invariant: oxttl reference must reject `{why}` (update the corpus if oxttl changed)"
            );
            assert!(
                Graph::parse_to_triples(src, "turtle").is_err(),
                "sparq turtle path must REJECT malformed Turtle: {why}"
            );
        }

        // Positive controls: well-formed Turtle MUST parse (so the oracle is not vacuous), covering
        // the constructs the T1 interner has to keep handling — prefixed names, full IRIs, blank
        // nodes, collections, language tags, datatyped + plain literals, RDF-star quoted triples.
        let good: &[&str] = &[
            "@prefix ex: <http://ex/> .\nex:s ex:p ex:o .\n",
            "<http://ex/s> <http://ex/p> \"plain\" .\n",
            "@prefix ex: <http://ex/> .\nex:s ex:p [ ex:q ( 1 2 3 ) ] ; ex:r _:b .\n",
            "@prefix ex: <http://ex/> .\nex:s ex:p \"hi\"@en , \"42\"^^<http://www.w3.org/2001/XMLSchema#integer> .\n",
            "@prefix ex: <http://ex/> .\n<< ex:s ex:p ex:o >> ex:said ex:alice .\n",
        ];
        for src in good {
            assert!(
                !oxttl_serial_rejects(src),
                "corpus invariant: oxttl reference must ACCEPT this well-formed Turtle"
            );
            assert!(
                Graph::parse_to_triples(src, "turtle").is_ok(),
                "sparq turtle path must ACCEPT well-formed Turtle: {src:?}"
            );
        }
    }

    /// [OPUS-4.8] (T2) Differential test for the `memchr`-based `next_terminator` against the
    /// pre-T2 scalar reference. The SIMD-skip must return the SAME offset (or the same `None`)
    /// for every interesting Turtle shape — comments, all four string-quote forms with escapes,
    /// IRIs, decimals (`.`-not-a-terminator), PN_LOCAL_ESC `\#`/`\.`, and EOF-truncation — so it
    /// is a behaviour-preserving speedup, not a re-derivation of the splitter's grammar.
    #[cfg(feature = "parallel")]
    #[test]
    fn next_terminator_memchr_matches_scalar() {
        // The exact pre-T2 scalar scanner, kept here as the oracle.
        fn scalar(bytes: &[u8], start: usize) -> Option<usize> {
            let n = bytes.len();
            let mut i = start;
            while i < n {
                match bytes[i] {
                    b'#' => {
                        while i < n && bytes[i] != b'\n' {
                            i += 1;
                        }
                    }
                    b'<' => {
                        i += 1;
                        while i < n && bytes[i] != b'>' {
                            i += 1;
                        }
                        if i >= n {
                            return None;
                        }
                        i += 1;
                    }
                    q @ (b'"' | b'\'') => {
                        let triple = i + 2 < n && bytes[i + 1] == q && bytes[i + 2] == q;
                        if triple {
                            i += 3;
                            loop {
                                if i >= n {
                                    return None;
                                }
                                if bytes[i] == b'\\' {
                                    i += 2;
                                } else if bytes[i] == q
                                    && i + 2 < n
                                    && bytes[i + 1] == q
                                    && bytes[i + 2] == q
                                {
                                    i += 3;
                                    break;
                                } else {
                                    i += 1;
                                }
                            }
                        } else {
                            i += 1;
                            while i < n && bytes[i] != q {
                                i += if bytes[i] == b'\\' { 2 } else { 1 };
                            }
                            if i >= n {
                                return None;
                            }
                            i += 1;
                        }
                    }
                    b'.' => match bytes.get(i + 1) {
                        None | Some(b' ' | b'\t' | b'\n' | b'\r' | b'#') => return Some(i + 1),
                        _ => i += 1,
                    },
                    b'\\' => {
                        if i + 1 >= n {
                            return None;
                        }
                        i += 2;
                    }
                    _ => i += 1,
                }
            }
            None
        }

        let cases: &[&str] = &[
            ":s :p :o .\n",
            ":s :p :o .",                                  // EOF terminator
            ":s :p 3.14 ; :q 2.5 .\n",                     // decimals are not terminators
            ":s :p \"a.b.c\" .\n",                          // dot inside a string
            ":s :p \"x\\\"y.z\" .\n",                       // escaped quote, then dot in string
            ":s :p 'single.quoted' .\n",                    // single-quote string
            ":s :p \"\"\"l1 . still\nl2.\"\"\" .\n",        // triple-quoted, dots + newline inside
            ":s :p '''a.b''' .\n",                          // triple single-quote
            ":s :p <http://x.y/a.b.c> .\n",                 // dots inside an IRI
            "ex:s ex:p ex:foo\\#bar .\n",                   // PN_LOCAL_ESC \# (review 1398)
            "ex:s ex:p ex:a\\.b .\n",                       // PN_LOCAL_ESC \. — \. is NOT a term
            ":s :p :o .# trailing comment\n:s2 :p :o2 .\n", // comment right after terminator
            "# leading comment with . inside\n:s :p :o .\n",
            ":s :p :o",                                     // no terminator, no EOF dot → None
            ":s :p \"unterminated",                         // unterminated string → None
            ":s :p <unterminated",                          // unterminated IRI → None
            ":s :p \"\"\"unterminated\n.",                  // unterminated triple-quote → None
            "ex:s ex:p ex:x\\",                             // trailing backslash → None
            "",
            ".",                                            // bare dot at EOF is a terminator
            ".5 .\n",                                       // leading decimal then real terminator
        ];
        for (k, c) in cases.iter().enumerate() {
            let b = c.as_bytes();
            // From every start offset, not just 0 — turtle_chunks resumes mid-buffer.
            for start in 0..=b.len() {
                assert_eq!(
                    next_terminator(b, start),
                    scalar(b, start),
                    "case {k} {c:?} at start {start}"
                );
            }
        }
    }

    #[cfg(feature = "mmap")]
    #[test]
    fn save_open_mmap_roundtrip() {
        // Build a graph, persist it, re-open with the indexes memory-mapped, and check
        // the store + dictionary are structurally identical (every triple round-trips).
        let mut nt = String::new();
        for i in 0..3000u32 {
            nt.push_str(&format!(
                "<http://ex/n{}> <http://ex/p{}> \"{}\"^^<http://www.w3.org/2001/XMLSchema#integer> .\n",
                i % 211,
                i % 13,
                i % 500
            ));
        }
        nt.push_str("<http://ex/n0> <http://ex/name> \"caf\\u00e9\"@fr .\n");
        // Temporal literals so the temporals.bin round-trip below has real cells:
        // zoned + floating dateTimes (sub-second), a date, and an ill-formed dateTime
        // (must stay uncached on both sides).
        nt.push_str("<http://ex/n1> <http://ex/at> \"2024-03-15T13:00:00.25Z\"^^<http://www.w3.org/2001/XMLSchema#dateTime> .\n");
        nt.push_str("<http://ex/n2> <http://ex/at> \"2024-03-15T13:00:00\"^^<http://www.w3.org/2001/XMLSchema#dateTime> .\n");
        nt.push_str("<http://ex/n3> <http://ex/on> \"2024-03-15\"^^<http://www.w3.org/2001/XMLSchema#date> .\n");
        nt.push_str("<http://ex/n4> <http://ex/at> \"not-a-date\"^^<http://www.w3.org/2001/XMLSchema#dateTime> .\n");
        let g = Graph::load_str(&nt, "ntriples").unwrap();
        let dir = std::env::temp_dir().join(format!("sparq_mmap_test_{}", std::process::id()));
        g.save(&dir).unwrap();
        let g2 = Graph::open(&dir).unwrap();
        assert_eq!(g.len(), g2.len());
        assert_eq!(g.dict.len(), g2.dict.len());
        let dump = |gg: &Graph| {
            let scan = gg.store.scan(&[None, None, None]);
            let mut v: Vec<(String, String, String)> = scan
                .rows
                .iter()
                .map(|r| {
                    let spo = scan.to_spo(r);
                    (gg.dict.term(spo[0]).to_string(), gg.dict.term(spo[1]).to_string(), gg.dict.term(spo[2]).to_string())
                })
                .collect();
            v.sort();
            v
        };
        assert_eq!(dump(&g), dump(&g2), "mmap-reopened store differs");

        // The numeric-value cache must round-trip through its memory-mapped form: every
        // numeric literal resolves to the same f64 (and non-numerics to None) as before.
        assert!(dir.join("numerics.bin").exists(), "numerics cache not persisted");
        assert!(matches!(g2.numerics, NumData::Mapped(..)), "numerics not mmap'd on open");
        for v in [0u32, 1, 42, 250, 499] {
            let lit = Term::Literal(Literal::new_typed_literal(v.to_string(), xsd::INTEGER));
            if let Some(id) = g.id_of(&lit) {
                assert_eq!(g.numeric_value(id), Some(v as f64));
                assert_eq!(g2.numeric_value(id), Some(v as f64), "mmap'd numeric differs for {v}");
            }
        }
        // A non-numeric term (the language-tagged literal) must be None in both.
        if let Some(id) = g.id_of(&Term::Literal(Literal::new_language_tagged_literal("café", "fr").unwrap())) {
            assert_eq!(g2.numeric_value(id), None);
        }

        // The temporal-value cache must round-trip through its memory-mapped form too:
        // every cached cell (instant bits, tz presence, family) identical, and
        // non-temporal terms None in both.
        assert!(dir.join("temporals.bin").exists(), "temporals cache not persisted");
        assert!(matches!(g2.temporals, TempData::Mapped(..)), "temporals not mmap'd on open");
        for i in 1..=g.dict.len() as Id {
            match (g.temporal_value(i), g2.temporal_value(i)) {
                (None, None) => {}
                (Some(a), Some(b)) => {
                    assert_eq!(a.instant.to_bits(), b.instant.to_bits(), "mmap'd instant differs for id {i}");
                    assert_eq!(a.has_tz, b.has_tz);
                    assert_eq!(a.kind, b.kind);
                }
                (a, b) => panic!("temporal cache presence differs for id {i}: {a:?} vs {b:?}"),
            }
        }
        // The dictionary is memory-mapped (zero resident term storage) and lookup still
        // round-trips: every term resolves to the same id and back to the same term.
        assert!(dir.join("dict-terms.bin").exists(), "mmap dict not persisted");
        for s in 0..50u32 {
            let t = Term::NamedNode(NamedNode::new_unchecked(format!("http://ex/n{}", s % 211)));
            assert_eq!(g.id_of(&t), g2.id_of(&t), "mmap dict lookup differs for {t}");
        }
        // Per-predicate stats are persisted (no POS/PSO re-scan on open) and identical.
        assert!(dir.join("predstats.bin").exists(), "pred stats not persisted");
        for p in 0..13u32 {
            let pred = NamedNode::new_unchecked(format!("http://ex/p{p}"));
            if let Some(pid) = g.id_of(&Term::NamedNode(pred)) {
                assert_eq!(g.store.pred_stat(pid), g2.store.pred_stat(pid), "pred_stat differs for p{p}");
            }
        }
        std::fs::remove_dir_all(&dir).ok();
    }

    /// [OPUS-4.8] Regression for review 1325 (sub-finding 2): a graph directory carrying the
    /// LEGACY single-file `dict.bin` dictionary (saved before the mmap dict format) must still
    /// open via `Graph::open` (falling back from the absent `dict-meta.bin`), with every triple
    /// round-tripping identically.
    #[cfg(feature = "mmap")]
    #[test]
    fn open_falls_back_to_legacy_dict_bin() {
        let mut nt = String::new();
        for i in 0..400u32 {
            nt.push_str(&format!("<http://ex/n{}> <http://ex/p{}> <http://ex/o{}> .\n", i % 97, i % 7, i % 53));
        }
        nt.push_str("<http://ex/n0> <http://ex/name> \"caf\\u00e9\"@fr .\n");
        let g = Graph::load_str(&nt, "ntriples").unwrap();
        // The in-memory build produces an arena dict (base == 0), so `Dict::save` applies.
        let dir = std::env::temp_dir().join(format!("sparq_legacy_dict_test_{}", std::process::id()));
        g.save(&dir).unwrap();
        // Simulate a LEGACY directory: drop every mmap-dict file and write the single-file
        // `dict.bin` instead. The permutation/numerics/temporals files are left untouched.
        for f in ["dict-meta.bin", "dict-terms.bin", "dict-offs.bin", "dict-hash.bin", "dict-hid.bin"] {
            std::fs::remove_file(dir.join(f)).ok();
        }
        g.dict.save(&dir.join("dict.bin")).unwrap();
        assert!(!dir.join("dict-meta.bin").exists());
        assert!(dir.join("dict.bin").exists());
        // `Graph::open` must succeed via the legacy fallback and round-trip every triple.
        let g2 = Graph::open(&dir).unwrap();
        assert_eq!(g.len(), g2.len());
        assert_eq!(g.dict.len(), g2.dict.len());
        let dump = |gg: &Graph| {
            let scan = gg.store.scan(&[None, None, None]);
            let mut v: Vec<(String, String, String)> = scan
                .rows
                .iter()
                .map(|r| {
                    let spo = scan.to_spo(r);
                    (gg.dict.term(spo[0]).to_string(), gg.dict.term(spo[1]).to_string(), gg.dict.term(spo[2]).to_string())
                })
                .collect();
            v.sort();
            v
        };
        assert_eq!(dump(&g), dump(&g2), "legacy-dict-reopened store differs");
        for s in 0..50u32 {
            let t = Term::NamedNode(NamedNode::new_unchecked(format!("http://ex/n{}", s % 97)));
            assert_eq!(g.id_of(&t), g2.id_of(&t), "legacy dict lookup differs for {t}");
        }
        std::fs::remove_dir_all(&dir).ok();
    }

    /// Graph-level COMPRESSED persistence: `save_compressed` → `open` must round-trip the
    /// full graph (terms, numerics, pred stats) identically to the raw path, including a
    /// pending delta-overlay being folded into the compressed permutations on save.
    #[cfg(feature = "mmap")]
    #[test]
    fn compressed_graph_roundtrip() {
        let mut nt = String::new();
        for i in 0..4000u32 {
            nt.push_str(&format!(
                "<http://ex/n{}> <http://ex/p{}> \"{}\"^^<http://www.w3.org/2001/XMLSchema#integer> .\n",
                i % 311,
                i % 11,
                i % 700
            ));
        }
        let mut g = Graph::load_str(&nt, "ntriples").unwrap();
        // A pending overlay (one delete + one insert) must be folded into the compressed save.
        let del = [g.id_of(&Term::NamedNode(NamedNode::new_unchecked("http://ex/n1"))).unwrap(),
                   g.id_of(&Term::NamedNode(NamedNode::new_unchecked("http://ex/p1"))).unwrap(),
                   g.id_of(&Term::Literal(Literal::new_typed_literal("1", xsd::INTEGER))).unwrap()];
        g.store.apply_delta(&[[del[0], del[1], del[0]]], &[del]);
        assert!(g.store.has_overlay());

        let base = std::env::temp_dir().join(format!("sparq_cgraph_{}", std::process::id()));
        let (raw_dir, cmp_dir) = (base.join("raw"), base.join("cmp"));
        g.save(&raw_dir).unwrap();
        g.save_compressed(&cmp_dir).unwrap();

        let a = Graph::open(&raw_dir).unwrap();
        let mut b = Graph::open(&cmp_dir).unwrap();
        let dump = |gg: &Graph| {
            let scan = gg.store.scan(&[None, None, None]);
            let mut v: Vec<(String, String, String)> = scan
                .rows
                .iter()
                .map(|r| {
                    let spo = scan.to_spo(r);
                    (gg.dict.term(spo[0]).to_string(), gg.dict.term(spo[1]).to_string(), gg.dict.term(spo[2]).to_string())
                })
                .collect();
            v.sort();
            v
        };
        assert_eq!(g.len(), b.len(), "overlay not folded into the compressed save");
        assert_eq!(dump(&a), dump(&b), "compressed graph differs from raw graph");
        // Numerics still resolve through the compressed-opened graph.
        let lit = Term::Literal(Literal::new_typed_literal("42", xsd::INTEGER));
        if let Some(id) = b.id_of(&lit) {
            assert_eq!(b.numeric_value(id), Some(42.0));
        }
        // Load-time decompression: identical content, perms now on the heap.
        let lazy_heap = b.store.heap_bytes();
        b.decompress_indexes();
        assert!(b.store.heap_bytes() > lazy_heap, "decompress_indexes must move perms to the heap");
        assert_eq!(dump(&a), dump(&b), "decompressed graph differs");
        std::fs::remove_dir_all(&base).ok();
    }

    #[cfg(feature = "mmap")]
    #[test]
    fn build_external_matches_in_memory() {
        // External-memory build with a TINY chunk so the triples spill across many runs
        // and exercise the k-way merge + per-permutation re-sort. The on-disk result must
        // be byte-for-byte identical to an in-memory load → save (same dedup, same order).
        let mut nt = String::new();
        for i in 0..5000u32 {
            nt.push_str(&format!(
                "<http://ex/n{}> <http://ex/p{}> \"{}\"^^<http://www.w3.org/2001/XMLSchema#integer> .\n",
                i % 173,
                i % 7,
                i % 400
            ));
        }
        // A duplicate line (must be deduped) + a non-integer literal with a language tag.
        nt.push_str("<http://ex/n0> <http://ex/p0> \"0\"^^<http://www.w3.org/2001/XMLSchema#integer> .\n");
        nt.push_str("<http://ex/n0> <http://ex/name> \"caf\\u00e9\"@fr .\n");

        let base = std::env::temp_dir().join(format!("sparq_ext_{}", std::process::id()));
        let mem_dir = base.join("mem");
        let ext_dir = base.join("ext");

        // In-memory load → save (reference), vs streaming external build (chunk = 256).
        let g = Graph::load_str(&nt, "ntriples").unwrap();
        g.save(&mem_dir).unwrap();
        Graph::build_external(nt.as_bytes(), "ntriples", &ext_dir, 256).unwrap();

        let mem = Graph::open(&mem_dir).unwrap();
        let ext = Graph::open(&ext_dir).unwrap();
        assert_eq!(mem.len(), ext.len(), "triple count differs");
        assert_eq!(mem.dict.len(), ext.dict.len(), "dict size differs");

        // Every BUILT permutation file must be byte-identical (same sort, same dedup).
        for &perm in store::BUILT {
            let f = format!("perm{}.bin", perm as usize);
            let a = std::fs::read(mem_dir.join(&f)).unwrap();
            let b = std::fs::read(ext_dir.join(&f)).unwrap();
            assert_eq!(a, b, "permutation {f} differs between in-memory and external build");
        }

        // And the data round-trips through terms.
        let dump = |gg: &Graph| {
            let scan = gg.store.scan(&[None, None, None]);
            let mut v: Vec<(String, String, String)> = scan
                .rows
                .iter()
                .map(|r| {
                    let spo = scan.to_spo(r);
                    (gg.dict.term(spo[0]).to_string(), gg.dict.term(spo[1]).to_string(), gg.dict.term(spo[2]).to_string())
                })
                .collect();
            v.sort();
            v
        };
        assert_eq!(dump(&mem), dump(&ext), "external-built store differs");
        std::fs::remove_dir_all(&base).ok();
    }

    #[test]
    fn ntriples_parallel_matches_sequential() {
        // >4KB so the input spans multiple parallel chunks; subjects/predicates repeat
        // across chunks (exercising the partial-dict merge) and the objects are inline
        // integers (exercising the inline-id passthrough in the remap).
        let mut nt = String::new();
        for i in 0..2000u32 {
            nt.push_str(&format!(
                "<http://ex/n{}> <http://ex/p{}> \"{}\"^^<http://www.w3.org/2001/XMLSchema#integer> .\n",
                i % 137,
                i % 11,
                i % 500
            ));
        }
        // The byte-level parser's risky paths, cross-checked against oxttl: escapes
        // (quote / backslash / newline / \\u), language tags, typed + simple literals,
        // a comment, and a different IRI namespace.
        nt.push_str("# a comment line\n");
        nt.push_str("<http://ex/s> <http://other.org/p> \"a \\\"q\\\" b\\nc \\\\ d \\u00e9\" .\n");
        nt.push_str("<http://ex/s> <http://ex/name> \"caf\\u00e9\"@fr .\n");
        nt.push_str("<http://ex/s> <http://ex/v> \"1.5\"^^<http://www.w3.org/2001/XMLSchema#decimal> .\n");
        nt.push_str("<http://ex/s> <http://ex/plain> \"just a string\" .\n");
        nt.push_str("<http://ex/s> <http://ex/big> \"\\U0001F600 grin\" .\n");
        let par = Graph::load_str(&nt, "ntriples").unwrap(); // parallel (when feature on)
        let seq = Graph::load_reader(nt.as_bytes(), "ntriples").unwrap(); // sequential
        assert_eq!(par.len(), seq.len());
        assert_eq!(par.dict.len(), seq.dict.len());
        // Full structural equality independent of id-assignment order: map every stored
        // triple back to its terms, sort, compare.
        let dump = |g: &Graph| {
            let scan = g.store.scan(&[None, None, None]);
            let mut v: Vec<(String, String, String)> = scan
                .rows
                .iter()
                .map(|r| {
                    let spo = scan.to_spo(r);
                    (g.dict.term(spo[0]).to_string(), g.dict.term(spo[1]).to_string(), g.dict.term(spo[2]).to_string())
                })
                .collect();
            v.sort();
            v
        };
        assert_eq!(dump(&par), dump(&seq));
    }

    /// The streaming pipelined loader must be byte-exact against the serial loader for
    /// readers with SHORT reads (a gzip/zstd decompressor returns a fraction of the block
    /// per `read()` — the measured streaming-ingest defect), read boundaries landing
    /// mid-line, an EOF mid-line (final line without a trailing newline), and empty input.
    #[cfg(feature = "parallel")]
    #[test]
    fn load_reader_parallel_short_reads_match_sequential() {
        /// Returns at most `max` bytes per `read()` call.
        struct ShortReader<'a> {
            data: &'a [u8],
            pos: usize,
            max: usize,
        }
        impl std::io::Read for ShortReader<'_> {
            fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                let n = (self.data.len() - self.pos).min(self.max).min(buf.len());
                buf[..n].copy_from_slice(&self.data[self.pos..self.pos + n]);
                self.pos += n;
                Ok(n)
            }
        }
        let mut nt = String::new();
        for i in 0..3000u32 {
            nt.push_str(&format!(
                "<http://ex/n{}> <http://ex/p{}> \"{}\"^^<http://www.w3.org/2001/XMLSchema#integer> .\n",
                i % 211,
                i % 13,
                i % 700
            ));
        }
        nt.push_str("<http://ex/last> <http://ex/p0> \"no trailing newline\" ."); // EOF mid-line
        let seq = Graph::load_reader(nt.as_bytes(), "ntriples").unwrap();
        // 7 B forces hundreds of reads per line; the others land boundaries mid-line.
        for max in [7usize, 1024, 1 << 16] {
            let par = Graph::load_reader_parallel(ShortReader { data: nt.as_bytes(), pos: 0, max }, "ntriples").unwrap();
            assert_eq!(par.len(), seq.len(), "triple count differs at max={max}");
            assert_eq!(par.dict.len(), seq.dict.len(), "dict size differs at max={max}");
            assert_eq!(dump_terms(&par), dump_terms(&seq), "stored triples differ at max={max}");
        }
        // MULTI-BLOCK: a small block size (vs the production 32 MiB) makes the same
        // document span ~60 blocks, with triple lines (each ~70-100 B, never a multiple
        // of 4096) straddling every block boundary and partial lines carried across
        // pipelined rounds — and the per-block dict merges must still agree.
        for (block, max) in [(4096usize, 997usize), (4096, 1 << 16), (8192, 7)] {
            let par = Graph::load_ntriples_pipelined(ShortReader { data: nt.as_bytes(), pos: 0, max }, block).unwrap();
            assert_eq!(par.len(), seq.len(), "triple count differs at block={block} max={max}");
            assert_eq!(par.dict.len(), seq.dict.len(), "dict size differs at block={block} max={max}");
            assert_eq!(dump_terms(&par), dump_terms(&seq), "stored triples differ at block={block} max={max}");
        }
        // A single line LONGER than the block size (carry outgrows the block).
        let long = format!("<http://ex/s> <http://ex/p> \"{}\" .\n<http://ex/s2> <http://ex/p> \"x\" .", "y".repeat(20_000));
        let lseq = Graph::load_reader(long.as_bytes(), "ntriples").unwrap();
        let lpar = Graph::load_ntriples_pipelined(ShortReader { data: long.as_bytes(), pos: 0, max: 333 }, 4096).unwrap();
        assert_eq!(lpar.len(), lseq.len());
        assert_eq!(dump_terms(&lpar), dump_terms(&lseq));
        // Empty input and input with no final newline at all.
        let empty = Graph::load_reader_parallel(ShortReader { data: b"", pos: 0, max: 7 }, "ntriples").unwrap();
        assert_eq!(empty.len(), 0);
        let one = Graph::load_reader_parallel(
            ShortReader { data: b"<http://ex/s> <http://ex/p> <http://ex/o> .", pos: 0, max: 3 },
            "ntriples",
        )
        .unwrap();
        assert_eq!(one.len(), 1);
        // A malformed document must surface the parse error, not hang the pipeline.
        let bad = Graph::load_reader_parallel(ShortReader { data: b"not ntriples\n", pos: 0, max: 5 }, "ntriples");
        assert!(bad.is_err());
    }

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

    /// The full sorted term-triple set of a graph (overlay merged), for state comparison.
    fn dump_terms(g: &Graph) -> Vec<(String, String, String)> {
        let scan = g.store.scan(&[None, None, None]);
        let mut v: Vec<(String, String, String)> = scan
            .rows
            .iter()
            .map(|r| {
                let spo = scan.to_spo(r);
                (g.dict.term(spo[0]).to_string(), g.dict.term(spo[1]).to_string(), g.dict.term(spo[2]).to_string())
            })
            .collect();
        v.sort();
        v
    }

    fn term_iri(n: u32, ns: &str) -> Term {
        Term::NamedNode(NamedNode::new_unchecked(format!("http://ex/{ns}{n}")))
    }

    /// DIFFERENTIAL: a random sequence of 200 insert/delete batches applied through the
    /// delta-overlay must yield exactly the same sorted triple set as applying each batch
    /// by full set-ops + rebuild — including across periodic `compact()` calls.
    #[test]
    fn overlay_differential_200_batches() {
        let base_ttl = "@prefix : <http://ex/> . :s0 :p0 :o0 . :s1 :p0 :o1 . :s2 :p1 :o2 .";
        let mut g = Graph::load_str(base_ttl, "turtle").unwrap();
        let mut reference: std::collections::BTreeSet<(String, String, String)> =
            dump_terms(&g).into_iter().collect();

        let mut st = 0xDEADBEEFu32;
        let mut rng = move || {
            st ^= st << 13;
            st ^= st >> 17;
            st ^= st << 5;
            st
        };
        let mk = |r: &mut dyn FnMut() -> u32| -> [Term; 3] {
            // A small term universe so inserts/deletes collide often (the hard cases:
            // re-insert, delete-of-pending-insert, delete-absent, re-insert-of-deleted).
            [term_iri(r() % 30, "s"), term_iri(r() % 5, "p"), term_iri(r() % 40, "o")]
        };
        for batch in 0..200 {
            let n_ins = (rng() % 6) as usize;
            let n_del = (rng() % 6) as usize;
            let inserts: Vec<[Term; 3]> = (0..n_ins).map(|_| mk(&mut rng)).collect();
            let deletes: Vec<[Term; 3]> = (0..n_del).map(|_| mk(&mut rng)).collect();

            // Reference semantics: deletes first, then inserts (SPARQL order).
            for [s, p, o] in &deletes {
                reference.remove(&(s.to_string(), p.to_string(), o.to_string()));
            }
            for [s, p, o] in &inserts {
                reference.insert((s.to_string(), p.to_string(), o.to_string()));
            }
            g.apply_delta(&inserts, &deletes).unwrap();

            if batch % 50 == 49 {
                g.compact().unwrap(); // fold the overlay periodically
                assert!(!g.store.has_overlay());
            }
            let got = dump_terms(&g);
            let want: Vec<(String, String, String)> = reference.iter().cloned().collect();
            assert_eq!(got, want, "state diverged after batch {batch}");
            assert_eq!(g.len(), reference.len(), "len diverged after batch {batch}");
        }
    }

    /// Durability roundtrip: updates on a directory-backed graph are WAL-logged and
    /// replayed on re-open; `compact()` folds them into a new persisted base atomically
    /// and truncates the log; the compacted state re-opens identically.
    #[cfg(feature = "mmap")]
    #[test]
    fn wal_replay_and_compact_roundtrip() {
        let mut nt = String::new();
        for i in 0..2000u32 {
            nt.push_str(&format!("<http://ex/s{}> <http://ex/p{}> <http://ex/o{}> .\n", i % 97, i % 7, i % 311));
        }
        let dir = std::env::temp_dir().join(format!("sparq_wal_rt_{}", std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        Graph::load_str(&nt, "ntriples").unwrap().save(&dir).unwrap();

        // Open from disk, apply two update batches (one with a NEW term + a numeric
        // literal — exercising dict append + numeric-cache growth), drop WITHOUT compacting.
        let expected = {
            let mut g = Graph::open(&dir).unwrap();
            let lit = Term::Literal(Literal::new_typed_literal("31250000000", xsd::INTEGER));
            g.apply_delta(
                &[[term_iri(1000, "s"), term_iri(50, "p"), lit.clone()], [term_iri(1, "s"), term_iri(1, "p"), term_iri(1, "o")]],
                &[[term_iri(0, "s"), term_iri(0, "p"), term_iri(0, "o")]],
            )
            .unwrap();
            g.apply_delta(&[], &[[term_iri(1, "s"), term_iri(1, "p"), term_iri(1, "o")]]).unwrap();
            assert!(g.store.has_overlay());
            // The appended numeric literal must be live in the numeric cache.
            let id = g.id_of(&lit).expect("appended literal must be interned");
            assert_eq!(g.numeric_value(id), Some(31_250_000_000.0));
            dump_terms(&g)
        };
        assert!(dir.join("wal.log").metadata().unwrap().len() > 0, "WAL must persist the batches");

        // Re-open: the WAL replays into the overlay — identical state.
        let mut g2 = Graph::open(&dir).unwrap();
        assert!(g2.store.has_overlay(), "replayed WAL must rebuild the overlay");
        assert_eq!(dump_terms(&g2), expected, "WAL replay must restore the exact state");
        let lit = Term::Literal(Literal::new_typed_literal("31250000000", xsd::INTEGER));
        let id = g2.id_of(&lit).expect("replayed literal must be interned");
        assert_eq!(g2.numeric_value(id), Some(31_250_000_000.0), "numeric cache must cover replayed terms");

        // Compact: overlay folded, base re-persisted atomically, WAL truncated.
        g2.compact().unwrap();
        assert!(!g2.store.has_overlay());
        assert_eq!(dump_terms(&g2), expected);
        assert_eq!(dir.join("wal.log").metadata().unwrap().len(), 0, "compact must truncate the WAL");
        assert!(!dir.with_extension("compact-new").exists());
        assert!(!dir.with_extension("compact-old").exists());
        drop(g2);

        // The compacted base re-opens to the same state, with nothing left to replay.
        let g3 = Graph::open(&dir).unwrap();
        assert!(!g3.store.has_overlay());
        assert_eq!(dump_terms(&g3), expected);
        std::fs::remove_dir_all(&dir).ok();
    }

    /// Kill-during-write simulation: a WAL whose trailing BATCH is torn (header, body, or
    /// commit trailer) must replay only the fully-COMMITTED batches and truncate the torn
    /// tail. [OPUS-4.8] (review 1593) Each `apply_delta` is one atomic batch frame:
    /// `[magic][n][body_len] <records> [crc][commit]`.
    #[cfg(feature = "mmap")]
    #[test]
    fn wal_torn_batch_replay() {
        let dir = std::env::temp_dir().join(format!("sparq_wal_torn_{}", std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        Graph::load_str("<http://ex/s0> <http://ex/p0> <http://ex/o0> .", "ntriples").unwrap().save(&dir).unwrap();

        {
            let mut g = Graph::open(&dir).unwrap();
            g.apply_delta(&[[term_iri(1, "s"), term_iri(1, "p"), term_iri(1, "o")]], &[]).unwrap();
            g.apply_delta(&[[term_iri(2, "s"), term_iri(2, "p"), term_iri(2, "o")]], &[]).unwrap();
        }
        let wal_path = dir.join("wal.log");
        let full = std::fs::read(&wal_path).unwrap();
        // Length of the FIRST batch frame: 12 (header) + body_len + 12 (crc+commit).
        let body_len = u32::from_le_bytes([full[8], full[9], full[10], full[11]]) as usize;
        let first_batch_len = 12 + body_len + 12;
        assert!(full.len() > first_batch_len, "test needs two batches");

        // (a) Sever the SECOND batch mid-body (a crash before its commit trailer is synced):
        //     the uncommitted batch is discarded WHOLE — never partially applied.
        let torn = &full[..first_batch_len + 14]; // header + a few body bytes, no trailer
        std::fs::write(&wal_path, torn).unwrap();
        let g = Graph::open(&dir).unwrap();
        let got = dump_terms(&g);
        assert_eq!(got.len(), 2, "only the first committed batch replays");
        assert!(got.iter().any(|(s, _, _)| s.contains("s1")));
        assert!(!got.iter().any(|(s, _, _)| s.contains("s2")));
        assert_eq!(std::fs::read(&wal_path).unwrap().len(), first_batch_len, "torn tail truncated to the batch boundary");

        // (b) Corrupt the FIRST batch's body (flip a byte inside it): the checksum fails, so
        //     the whole batch is rejected and replay stops at the base — never a partial apply.
        let mut corrupt = full[..first_batch_len].to_vec();
        corrupt[14] ^= 0xFF; // a byte inside the first record's body
        std::fs::write(&wal_path, &corrupt).unwrap();
        let g2 = Graph::open(&dir).unwrap();
        assert_eq!(dump_terms(&g2).len(), 1, "checksum-failing batch must not apply");
        assert_eq!(std::fs::read(&wal_path).unwrap().len(), 0, "corrupt leading batch truncates the whole log");
        std::fs::remove_dir_all(&dir).ok();
    }

    /// [OPUS-4.8] (review 1593) A MULTI-record batch must be atomic: if a crash severs the
    /// batch after some of its records are on disk but before the commit trailer, NONE of the
    /// batch's records may be applied (the pre-fix per-record WAL would apply a prefix).
    #[cfg(feature = "mmap")]
    #[test]
    fn wal_torn_multi_record_batch_is_all_or_nothing() {
        let dir = std::env::temp_dir().join(format!("sparq_wal_multi_{}", std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        Graph::load_str("<http://ex/s0> <http://ex/p0> <http://ex/o0> .", "ntriples").unwrap().save(&dir).unwrap();
        {
            let mut g = Graph::open(&dir).unwrap();
            // ONE batch of three inserts.
            let ins: Vec<[Term; 3]> = (1..=3)
                .map(|i| [term_iri(i, "s"), term_iri(i, "p"), term_iri(i, "o")])
                .collect();
            g.apply_delta(&ins, &[]).unwrap();
        }
        let wal_path = dir.join("wal.log");
        let full = std::fs::read(&wal_path).unwrap();
        let body_len = u32::from_le_bytes([full[8], full[9], full[10], full[11]]) as usize;
        // Drop the commit trailer (last 12 bytes) AND part of the last record — a torn append
        // with two of the three records' bytes present but no commit point.
        let torn = &full[..12 + body_len - 5]; // header + most of the body, no trailer
        std::fs::write(&wal_path, torn).unwrap();
        let g = Graph::open(&dir).unwrap();
        // NONE of the three records may have applied — only the base triple remains.
        assert_eq!(dump_terms(&g).len(), 1, "uncommitted multi-record batch must apply nothing");
        assert_eq!(std::fs::read(&wal_path).unwrap().len(), 0, "the torn batch is the whole log → truncated empty");
        std::fs::remove_dir_all(&dir).ok();
    }

    /// [OPUS-4.8] (review 1593) Compaction directory swap must be crash-recoverable: if `dir`
    /// is missing but the new base survives in `compact-new`, `Graph::open` completes the
    /// swap; if only `compact-old` survives, it rolls back — `dir` is never lost.
    #[cfg(feature = "mmap")]
    #[test]
    fn compact_swap_crash_recovery() {
        let base = std::env::temp_dir().join(format!("sparq_compact_recover_{}", std::process::id()));
        let dir = base.join("g");
        // Helper: build a graph dir with one triple naming `tag`.
        let build = |d: &std::path::Path, tag: &str| {
            std::fs::remove_dir_all(d).ok();
            Graph::load_str(&format!("<http://ex/{tag}> <http://ex/p> <http://ex/o> ."), "ntriples")
                .unwrap()
                .save(d)
                .unwrap();
        };

        // (a) Crash BETWEEN the two renames: dir missing, both siblings present. Recovery must
        //     COMPLETE the swap by promoting compact-new (the fully-synced new base).
        build(&dir.with_extension("compact-old"), "old");
        build(&dir.with_extension("compact-new"), "new");
        assert!(!dir.exists());
        let g = Graph::open(&dir).unwrap();
        assert!(dump_terms(&g).iter().any(|(s, _, _)| s.contains("/new")), "must promote compact-new");
        assert!(!dir.with_extension("compact-new").exists());
        assert!(!dir.with_extension("compact-old").exists());

        // (b) Crash BEFORE the new base was ready: dir missing, only compact-old present.
        //     Recovery must ROLL BACK by restoring compact-old.
        std::fs::remove_dir_all(&dir).ok();
        build(&dir.with_extension("compact-old"), "old");
        assert!(!dir.exists());
        let g = Graph::open(&dir).unwrap();
        assert!(dump_terms(&g).iter().any(|(s, _, _)| s.contains("/old")), "must roll back to compact-old");
        assert!(!dir.with_extension("compact-old").exists());

        std::fs::remove_dir_all(&base).ok();
    }

    /// [OPUS-4.8] (review 1593) A durable CLEAR of a directory-backed graph must SURVIVE a
    /// reopen. The pre-fix CLEAR replaced the graph with a fresh in-memory empty graph,
    /// dropping the WAL/dir association, so the on-disk base was untouched and a reopen
    /// restored the cleared data. `clear_default_durable` retracts the triples through the
    /// WAL instead, so the emptiness is persistent.
    #[cfg(feature = "mmap")]
    #[test]
    fn clear_default_durable_survives_reopen() {
        let dir = std::env::temp_dir().join(format!("sparq_clear_durable_{}", std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        let mut nt = String::new();
        for i in 0..20u32 {
            nt.push_str(&format!("<http://ex/s{i}> <http://ex/p> <http://ex/o{i}> .\n"));
        }
        Graph::load_str(&nt, "ntriples").unwrap().save(&dir).unwrap();

        {
            let mut g = Graph::open(&dir).unwrap();
            assert_eq!(dump_terms(&g).len(), 20);
            g.clear_default_durable().unwrap();
            assert_eq!(dump_terms(&g).len(), 0, "clear empties the graph in memory");
            assert!(g.wal.is_some(), "the WAL/dir association is preserved");
        }
        // Reopen: the WAL-logged retractions replay, so the graph stays empty.
        let g2 = Graph::open(&dir).unwrap();
        assert_eq!(dump_terms(&g2).len(), 0, "durable CLEAR must survive the reopen");
        std::fs::remove_dir_all(&dir).ok();
    }

    /// `load_str_with_base`: relative IRIs (subjects, predicates, objects, @prefix-free
    /// docs) resolve against the supplied base, in Turtle and TriG; an invalid base is
    /// a clean error; line-based formats ignore the base.
    #[test]
    fn load_str_with_base_resolves_relative_iris() {
        let g = Graph::load_str_with_base("<a> <p> <../up/o> .", "turtle", "http://ex/dir/").unwrap();
        let scan = g.store.scan(&[None, None, None]);
        let t = scan.to_spo(&scan.rows.as_ref()[0]);
        assert_eq!(g.dict.term(t[0]).to_string(), "<http://ex/dir/a>");
        assert_eq!(g.dict.term(t[1]).to_string(), "<http://ex/dir/p>");
        assert_eq!(g.dict.term(t[2]).to_string(), "<http://ex/up/o>");

        // A document's own @base still wins over the parameter (RFC 3986 layering).
        let g = Graph::load_str_with_base("@base <http://other/> . <a> <p> <o> .", "turtle", "http://ex/").unwrap();
        let scan = g.store.scan(&[None, None, None]);
        let t = scan.to_spo(&scan.rows.as_ref()[0]);
        assert_eq!(g.dict.term(t[0]).to_string(), "<http://other/a>");

        // TriG routes through the base-aware parser too (named graphs folded, as load_str).
        let g = Graph::load_str_with_base("<g> { <a> <p> <o> . }", "trig", "http://ex/").unwrap();
        assert_eq!(g.len(), 1);

        // N-Triples has no relative IRIs: base is accepted and ignored.
        let g = Graph::load_str_with_base("<http://ex/a> <http://ex/p> <http://ex/o> .", "ntriples", "http://b/")
            .unwrap();
        assert_eq!(g.len(), 1);

        assert!(Graph::load_str_with_base("<a> <p> <o> .", "turtle", "not a iri").is_err());
    }

    /// `Dict::iter` covers exactly the real ids `1..=len()` in order, with parts that
    /// reconstruct to the same terms as `term()`.
    #[test]
    fn dict_iter_is_dense_and_complete() {
        let g = Graph::load_str(
            "@prefix : <http://ex/> . :a :p \"lit\"@en . :a :q 99999999999 . _:b :p :a .",
            "turtle",
        )
        .unwrap();
        let d = &g.dict;
        let items: Vec<(Id, String)> = d.iter().map(|(id, _)| (id, d.term(id).to_string())).collect();
        assert_eq!(items.len(), d.len());
        assert_eq!(d.iter().len(), d.len(), "ExactSizeIterator must agree with len()");
        for (i, (id, _)) in items.iter().enumerate() {
            assert_eq!(*id, (i + 1) as Id, "ids must be dense and 1-based");
        }
        let all: Vec<String> = items.into_iter().map(|(_, t)| t).collect();
        assert!(all.iter().any(|t| t == "<http://ex/a>"));
        assert!(all.iter().any(|t| t == "\"lit\"@en"));
    }

    /// `Graph::iter_ids` / `iter_ids_sorted`: full coverage in canonical order, the
    /// requested sort column actually sorted, and the delta-overlay merged in.
    #[test]
    fn graph_iter_ids_orders_and_overlay() {
        let mut g = Graph::load_str(
            "@prefix : <http://ex/> . :b :p :x . :a :p :y . :a :q :x . :c :r 5 .",
            "turtle",
        )
        .unwrap();
        let spo: Vec<[Id; 3]> = g.iter_ids().collect();
        assert_eq!(spo.len(), g.len());
        assert!(spo.windows(2).all(|w| w[0] <= w[1]), "iter_ids must be (S,P,O)-sorted");
        // Same triple set through the object-sorted order.
        let mut by_obj: Vec<[Id; 3]> = g.iter_ids_sorted(2).collect();
        assert!(by_obj.windows(2).all(|w| w[0][2] <= w[1][2]), "col 2 must be sorted");
        let mut spo_sorted = spo.clone();
        spo_sorted.sort_unstable();
        by_obj.sort_unstable();
        assert_eq!(spo_sorted, by_obj);
        // Distinct subjects via run boundaries == 3 (:a :b :c).
        let mut subjects: Vec<Id> = spo.iter().map(|t| t[0]).collect();
        subjects.dedup();
        assert_eq!(subjects.len(), 3);
        // Overlay: an applied delta must appear in (and disappear from) the iterator.
        let t = |s: &str| Term::NamedNode(NamedNode::new_unchecked(format!("http://ex/{s}")));
        g.apply_delta(&[[t("z"), t("p"), t("x")]], &[]).unwrap();
        assert_eq!(g.iter_ids().len(), 5);
        assert_eq!(g.iter_ids().count(), 5);
        g.apply_delta(&[], &[[t("z"), t("p"), t("x")]]).unwrap();
        assert_eq!(g.iter_ids().count(), 4);
    }

    /// REGRESSION (sparq-py TODO): `Graph::save` on a graph whose numeric cache went
    /// SPARSE (`into_compressed`) must not panic — the dense cache is recomputed from
    /// the dictionary on save, and the round-trip preserves numeric query behaviour.
    #[cfg(feature = "mmap")]
    #[test]
    fn save_after_into_compressed_sparse_numerics() {
        let dir = std::env::temp_dir().join(format!("sparq_sparse_save_{}", std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        let g = Graph::load_str(
            "@prefix : <http://ex/> . :a :v 1.5 . :b :v 2.5 . :c :p :a .",
            "turtle",
        )
        .unwrap()
        .into_compressed();
        g.save(&dir).unwrap();
        let g2 = Graph::open(&dir).unwrap();
        assert_eq!(g2.len(), 3);
        // The numeric cache must have round-tripped: 1.5 / 2.5 still resolve.
        let vals: Vec<f64> =
            g2.dict.iter().filter_map(|(id, _)| g2.numeric_value(id)).collect();
        assert!(vals.contains(&1.5) && vals.contains(&2.5), "numerics must survive: {vals:?}");
        // And save_compressed takes the same path.
        std::fs::remove_dir_all(&dir).ok();
        g.save_compressed(&dir).unwrap();
        assert_eq!(Graph::open(&dir).unwrap().len(), 3);
        std::fs::remove_dir_all(&dir).ok();
    }
}

#[cfg(test)]
mod dir_roundtrip_test {
    /// `apply_delta_nquads` routes per graph: default-graph lines to the main graph,
    /// named-graph lines to the matching sub-graph (auto-created on first insert),
    /// deletes-before-inserts, bnodes matched by label, absent-graph deletes a no-op.
    #[test]
    fn apply_delta_nquads_routes_per_graph() {
        use oxrdf::{Literal, NamedNode, Term};
        let mut g = crate::Graph::load_dataset(
            "<http://ex/a> <http://ex/p> \"keep\" .\n\
             <http://ex/a> <http://ex/p> \"drop\" .\n\
             _:b <http://ex/p> \"bnode\" .\n\
             <http://ex/x> <http://ex/p> \"g1-drop\" <http://ex/g1> .",
            "nquads",
        )
        .unwrap();
        g.apply_delta_nquads(
            // inserts: default graph, existing named graph, and a NEW named graph
            "<http://ex/a> <http://ex/p> \"new\" .\n\
             <http://ex/x> <http://ex/p> \"g1-new\" <http://ex/g1> .\n\
             <http://ex/y> <http://ex/p> \"g2-new\" <http://ex/g2> .",
            // deletes: default-graph literal, a bnode triple BY LABEL, and one
            // against an absent graph (must be a no-op, not an error)
            "<http://ex/a> <http://ex/p> \"drop\" .\n\
             _:b <http://ex/p> \"bnode\" .\n\
             <http://ex/x> <http://ex/p> \"g1-drop\" <http://ex/g1> .\n\
             <http://ex/z> <http://ex/p> \"nope\" <http://ex/absent> .",
        )
        .unwrap();
        let lit = |v: &str| Term::Literal(Literal::new_simple_literal(v));
        assert_eq!(g.len(), 2); // keep + new (drop and the bnode triple retracted)
        assert!(g.id_of(&lit("keep")).is_some() && g.id_of(&lit("new")).is_some());
        let named = |g: &crate::Graph, name: &str| {
            let name = Term::NamedNode(NamedNode::new_unchecked(name));
            g.named.iter().find(|(n, _)| *n == name).map(|(_, sub)| sub.len())
        };
        assert_eq!(named(&g, "http://ex/g1"), Some(1)); // g1-drop out, g1-new in
        assert_eq!(named(&g, "http://ex/g2"), Some(1)); // auto-created
        assert_eq!(named(&g, "http://ex/absent"), None); // delete-only: never created
    }

    #[test]
    fn dir_literal_roundtrip() {
        let g = crate::Graph::load_str("@prefix : <http://ex/> . :a :p \"abc\"@en--ltr .", "turtle").unwrap();
        let scan = g.store.scan(&[None, None, None]);
        let t = scan.to_spo(&scan.rows.as_ref()[0]);
        assert_eq!(g.dict.term(t[2]).to_string(), "\"abc\"@en--ltr");
    }

    /// Fork-after-compact must stay O(delta): compaction folds the numeric/temporal
    /// caches into a FRESH shared base but keeps them in the shareable `Forked`
    /// shape (Arc base + empty side map) and re-freezes the dict — so the next
    /// `fork()` is Arc bumps, never an O(dict) cache/dict re-freeze. (Roborev
    /// finding on the initial fold-to-flat implementation.)
    #[test]
    fn compact_keeps_caches_shareable_for_the_next_fork() {
        use crate::{Graph, NumData, TempData};
        use oxrdf::vocab::xsd;
        use oxrdf::{Literal, NamedNode, Term};
        let g = Graph::load_str(
            "@prefix : <http://ex/> . :a :age 30 . :b :p :c .",
            "turtle",
        )
        .unwrap();
        let mut f = g.fork();
        let lit = |v: &str| Term::Literal(Literal::new_typed_literal(v, xsd::DECIMAL));
        let iri = |s: &str| Term::NamedNode(NamedNode::new_unchecked(s));
        f.apply_delta(&[[iri("http://ex/n"), iri("http://ex/age"), lit("4.5")]], &[]).unwrap();
        f.compact().unwrap();
        assert_eq!(f.pending_delta_len(), 0);
        // Caches stay in the shareable Forked shape with an EMPTY side map…
        assert!(
            matches!(&f.numerics, NumData::Forked { extra, .. } if extra.is_empty()),
            "numeric cache must be folded into a fresh shared base"
        );
        assert!(
            matches!(&f.temporals, TempData::Forked { extra, .. } if extra.is_empty()),
            "temporal cache must be folded into a fresh shared base"
        );
        // …and still answer: the folded base covers the post-fork term.
        let id = f.id_of(&lit("4.5")).unwrap();
        assert_eq!(f.numeric_value(id), Some(4.5));
        // The next fork shares those bases (no re-freeze) and keeps working.
        let f2 = f.fork();
        let same_base = match (&f.numerics, &f2.numerics) {
            (NumData::Forked { base: a, .. }, NumData::Forked { base: b, .. }) => {
                std::sync::Arc::ptr_eq(a, b)
            }
            _ => false,
        };
        assert!(same_base, "fork after compact must share the folded cache base");
        assert_eq!(f2.numeric_value(id), Some(4.5));
    }
}
