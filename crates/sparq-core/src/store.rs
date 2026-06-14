//! Triple store: the six sorted permutation indexes over dictionary-encoded
//! triples (Hexastore / RDF-3X / QLever design).
//!
//! Storing all six orderings (SPO SOP PSO POS OSP OPS) means every triple
//! pattern is answered by a single contiguous range (binary search on the
//! bound prefix), and the scan output is sorted by the remaining positions —
//! which is exactly what merge joins need. M1 holds each permutation as a
//! sorted `Vec<[Id; 3]>`; later milestones replace these with block-compressed,
//! optionally memory-mapped columns.

use crate::dict::Id;
#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// The six permutations. Each names the order of (subject, predicate, object)
/// columns as stored.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Perm {
    Spo,
    Sop,
    Pso,
    Pos,
    Osp,
    Ops,
}

/// The permutations actually built and searched. The full six give every triple
/// pattern a sorted scan in the order any merge join wants. The `compact-index` set
/// {SPO, POS, OSP} still answers EVERY triple pattern from one index (SPO→S*/SP*,
/// POS→P*/PO*, OSP→O*/OS*) at half the memory, at the cost of some merge joins (and
/// some lazy-count fast paths) falling back to hashing / sorting.
// Compact set on wasm ALWAYS (memory-bound target), or on native opt-in via the
// `compact-index` feature (for testing). Keyed on `target_arch` — NOT just a feature —
// so the wasm choice does not leak to the native build via Cargo feature unification.
#[cfg(not(any(target_arch = "wasm32", feature = "compact-index")))]
pub const BUILT: &[Perm] = &[Perm::Spo, Perm::Sop, Perm::Pso, Perm::Pos, Perm::Osp, Perm::Ops];
#[cfg(any(target_arch = "wasm32", feature = "compact-index"))]
pub const BUILT: &[Perm] = &[Perm::Spo, Perm::Pos, Perm::Osp];

impl Perm {
    pub const ALL: [Perm; 6] = [Perm::Spo, Perm::Sop, Perm::Pso, Perm::Pos, Perm::Osp, Perm::Ops];

    /// The column indices (into a canonical [s,p,o] triple) in this permutation's
    /// sort order. e.g. POS -> [1,2,0].
    #[inline]
    pub fn order(self) -> [usize; 3] {
        match self {
            Perm::Spo => [0, 1, 2],
            Perm::Sop => [0, 2, 1],
            Perm::Pso => [1, 0, 2],
            Perm::Pos => [1, 2, 0],
            Perm::Osp => [2, 0, 1],
            Perm::Ops => [2, 1, 0],
        }
    }
}

/// A triple pattern over ids: `None` is a variable (wildcard), `Some(id)` is
/// bound.
pub type Pattern = [Option<Id>; 3];

use rustc_hash::{FxHashMap, FxHashSet};

/// Per-predicate statistics for cardinality estimation (a characteristic-set-lite
/// summary): how many triples use the predicate, and how many *distinct* subjects
/// and objects it relates. Lets the planner estimate join result sizes.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct PredStat {
    pub count: usize,
    pub ndv_subj: usize,
    pub ndv_obj: usize,
}

/// A permutation index's storage: either an in-memory `Vec` (built / loaded) or, with
/// the `mmap` feature, a memory-mapped on-disk file — so a dataset larger than RAM can
/// be queried, the OS paging in only the working set (out-of-core).
enum PermData {
    Owned(Vec<[Id; 3]>),
    #[cfg(feature = "mmap")]
    Mapped(memmap2::Mmap),
    /// Block-compressed (~4-6 B/triple vs 12). The memory-bound storage mode for the
    /// browser: scans decode only the blocks the key-range touches. See [`compress`].
    Compressed(crate::compress::CompressedPerm),
}

impl Default for PermData {
    fn default() -> Self {
        PermData::Owned(Vec::new())
    }
}

impl PermData {
    /// Borrows the rows as a contiguous slice. Valid only for the raw (Owned/Mapped)
    /// modes — the compressed mode has no flat layout, so callers that may hold a
    /// compressed perm must go through [`rows_in`](Self::rows_in) instead.
    #[inline]
    fn as_slice(&self) -> &[[Id; 3]] {
        match self {
            PermData::Owned(v) => v,
            #[cfg(feature = "mmap")]
            PermData::Mapped(m) => {
                let bytes: &[u8] = m;
                let n = bytes.len() / std::mem::size_of::<[Id; 3]>();
                // SAFETY: the file is a whole number of little-endian [u32;3] triples and
                // an mmap is page-aligned (>= the 4-byte alignment of `u32`).
                unsafe { std::slice::from_raw_parts(bytes.as_ptr().cast::<[Id; 3]>(), n) }
            }
            PermData::Compressed(_) => unreachable!("as_slice on a compressed permutation"),
        }
    }

    /// The rows matching the inclusive key range `[lo, hi]`, sorted. Raw modes binary-
    /// search and BORROW a sub-slice (no allocation); the compressed mode decodes only
    /// the spanning blocks and returns an OWNED `Vec`. Either way the operators above
    /// receive a `&[[Id;3]]` (via the `Cow`), so their algorithms are unchanged.
    #[inline]
    fn rows_in(&self, lo: [Id; 3], hi: [Id; 3]) -> std::borrow::Cow<'_, [[Id; 3]]> {
        match self {
            PermData::Compressed(c) => std::borrow::Cow::Owned(c.range(lo, hi)),
            _ => {
                let rows = self.as_slice();
                let s = lower_bound(rows, &lo);
                let e = upper_bound(rows, &hi);
                std::borrow::Cow::Borrowed(&rows[s..e])
            }
        }
    }

    /// Cheap count of rows in `[lo, hi]` (for the planner) — no full materialization.
    #[inline]
    fn count_in(&self, lo: [Id; 3], hi: [Id; 3]) -> usize {
        match self {
            PermData::Compressed(c) => c.count_range(lo, hi),
            _ => {
                let rows = self.as_slice();
                upper_bound(rows, &hi) - lower_bound(rows, &lo)
            }
        }
    }

    #[inline]
    fn len(&self) -> usize {
        match self {
            PermData::Compressed(c) => c.len(),
            _ => self.as_slice().len(),
        }
    }

    fn heap_bytes(&self) -> usize {
        match self {
            PermData::Owned(v) => v.capacity() * std::mem::size_of::<[Id; 3]>(),
            #[cfg(feature = "mmap")]
            PermData::Mapped(_) => 0, // resident pages are charged to the OS page cache, not the heap
            PermData::Compressed(c) => c.heap_bytes(),
        }
    }
}

/// Pending updates layered over the immutable base indexes (the T17 delta-overlay):
/// triples INSERTED since the last compaction, and base triples DELETED since then.
/// Consulted at scan time — the base stays immutable (and mmap-able), and an update
/// batch costs O(batch) instead of the O(n) full rebuild. Invariants kept by
/// [`TripleStore::apply_delta`]: `added` is canonical-SPO sorted + deduplicated and
/// DISJOINT from both the base and `deleted`; `deleted` only ever holds base triples.
///
/// `Clone` because [`TripleStore::fork`] carries the overlay into the forked store
/// BY VALUE (O(overlay), bounded by the compaction policy) while the base indexes are
/// shared structurally.
#[derive(Default, Clone)]
struct Overlay {
    added: Vec<[Id; 3]>,
    deleted: FxHashSet<[Id; 3]>,
}

impl Overlay {
    /// The `added` triples matching the inclusive `[lo, hi]` key range, as rows in
    /// `perm` column order, SORTED in that order (re-sorted per scan — the overlay is
    /// small between compactions, so this is O(k log k) on k matching insertions).
    fn added_rows(&self, perm: Perm, lo: [Id; 3], hi: [Id; 3]) -> Vec<[Id; 3]> {
        let order = perm.order();
        let mut rows: Vec<[Id; 3]> = self
            .added
            .iter()
            .map(|t| [t[order[0]], t[order[1]], t[order[2]]])
            .filter(|r| *r >= lo && *r <= hi)
            .collect();
        rows.sort_unstable();
        rows
    }

    /// Merges the (perm-sorted) base rows with the overlay for one scan: base rows whose
    /// canonical triple is deleted are dropped, and the matching `added` rows are merge-
    /// interleaved — so the output keeps the permutation's sort order, preserving the
    /// guarantees downstream merge joins rely on. `added` is disjoint from the base, so
    /// no duplicate handling is needed.
    fn merge(&self, base: &[[Id; 3]], perm: Perm, lo: [Id; 3], hi: [Id; 3]) -> Vec<[Id; 3]> {
        let add = self.added_rows(perm, lo, hi);
        let order = perm.order();
        let mut out = Vec::with_capacity(base.len() + add.len());
        let mut ai = 0;
        let check_deleted = !self.deleted.is_empty();
        for &row in base {
            if check_deleted {
                let mut spo = [0; 3];
                spo[order[0]] = row[0];
                spo[order[1]] = row[1];
                spo[order[2]] = row[2];
                if self.deleted.contains(&spo) {
                    continue;
                }
            }
            while ai < add.len() && add[ai] < row {
                out.push(add[ai]);
                ai += 1;
            }
            out.push(row);
        }
        out.extend_from_slice(&add[ai..]);
        out
    }

    /// How many overlay triples fall in the `[lo, hi]` range of `perm` — the exact
    /// correction to a base range count. O(|overlay|).
    fn count_correction(&self, perm: Perm, lo: [Id; 3], hi: [Id; 3]) -> (usize, usize) {
        let order = perm.order();
        let in_range = |t: &[Id; 3]| {
            let r = [t[order[0]], t[order[1]], t[order[2]]];
            r >= lo && r <= hi
        };
        let add = self.added.iter().filter(|t| in_range(t)).count();
        let del = self.deleted.iter().filter(|t| in_range(t)).count();
        (add, del)
    }

    fn is_empty(&self) -> bool {
        self.added.is_empty() && self.deleted.is_empty()
    }

    fn heap_bytes(&self) -> usize {
        self.added.capacity() * std::mem::size_of::<[Id; 3]>() + self.deleted.capacity() * 13
    }
}

pub struct TripleStore {
    // Each permutation in its column order, sorted (so binary search on a bound prefix
    // is a plain lexicographic comparison of the leading columns) — owned or mmap'd.
    //
    // Behind an `Arc` so [`fork`](Self::fork) can SHARE the immutable base indexes
    // across snapshot generations (the structural fork): every store is born
    // shareable, a fork is an Arc bump. The only post-build mutation,
    // [`decompress_to_ram`](Self::decompress_to_ram), goes through `Arc::get_mut`
    // (it runs on freshly opened, never-yet-shared stores). Cost when unused: one
    // extra pointer indirection per scan/estimate CALL (not per row) — measured in
    // the flat-read benchmark as within noise.
    perms: std::sync::Arc<[PermData; 6]>,
    // Per-predicate stats keyed by predicate id (for the cost-based planner).
    // Arc-shared across forks like the permutations (read-only after build).
    pred_stats: std::sync::Arc<FxHashMap<Id, PredStat>>,
    // The delta-overlay of pending updates, `None` when there are none — so the scan
    // hot path pays exactly one (perfectly predicted) branch when no update happened.
    // NOTE: `pred_stats` is not overlay-adjusted (planner estimates only); `estimate`
    // and `len` are exact.
    overlay: Option<Box<Overlay>>,
}

impl TripleStore {
    /// Builds the [`BUILT`] permutation indexes from canonical [s,p,o] triples (all
    /// six by default; just SPO/POS/OSP under `compact-index`). SPO is sorted (in
    /// parallel) and deduplicated first; the rest are independent and built concurrently.
    pub fn from_triples(triples: Vec<[Id; 3]>) -> Self {
        let perms = Self::build_raw_perms(triples);
        let pred_stats = Self::compute_pred_stats(&perms);
        TripleStore { perms: std::sync::Arc::new(perms), pred_stats: std::sync::Arc::new(pred_stats), overlay: None }
    }

    /// Like [`from_triples`](Self::from_triples) but stores each permutation
    /// BLOCK-COMPRESSED (~4-6 B/triple vs 12) — the memory-bound storage mode for the
    /// browser, where holding 2.5x more triples in the same RAM matters more than the
    /// per-scan decode cost. Cardinality stats are computed from the raw perms *before*
    /// encoding (so neither the build nor the planner ever decodes a whole index).
    pub fn from_triples_compressed(triples: Vec<[Id; 3]>) -> Self {
        let raw = Self::build_raw_perms(triples);
        let pred_stats = Self::compute_pred_stats(&raw);
        let mut perms: [PermData; 6] = std::array::from_fn(|_| PermData::default());
        for (i, pd) in raw.into_iter().enumerate() {
            if let PermData::Owned(v) = pd {
                if !v.is_empty() {
                    perms[i] = PermData::Compressed(crate::compress::CompressedPerm::encode(&v));
                }
            }
        }
        TripleStore { perms: std::sync::Arc::new(perms), pred_stats: std::sync::Arc::new(pred_stats), overlay: None }
    }

    /// Builds the [`BUILT`] raw permutation indexes from canonical [s,p,o] triples (all
    /// six by default; just SPO/POS/OSP under `compact-index`). SPO is sorted (in
    /// parallel) and deduplicated first; the rest are independent and built concurrently.
    fn build_raw_perms(mut triples: Vec<[Id; 3]>) -> [PermData; 6] {
        // Deduplicate via the SPO ordering first.
        #[cfg(feature = "parallel")]
        triples.par_sort_unstable();
        #[cfg(not(feature = "parallel"))]
        triples.sort_unstable();
        triples.dedup();

        let build = |order: [usize; 3]| -> Vec<[Id; 3]> {
            let mut v: Vec<[Id; 3]> = triples
                .iter()
                .map(|t| [t[order[0]], t[order[1]], t[order[2]]])
                .collect();
            v.sort_unstable();
            v
        };

        // Build every BUILT permutation except SPO (which is the deduped `triples`).
        let to_build: Vec<Perm> = BUILT.iter().copied().filter(|&p| p != Perm::Spo).collect();
        #[cfg(feature = "parallel")]
        let built: Vec<Vec<[Id; 3]>> = to_build.par_iter().map(|p| build(p.order())).collect();
        #[cfg(not(feature = "parallel"))]
        let built: Vec<Vec<[Id; 3]>> = to_build.iter().map(|p| build(p.order())).collect();

        // Place each built permutation at its canonical slot; the rest stay empty.
        let mut perms: [PermData; 6] = std::array::from_fn(|_| PermData::default());
        perms[Perm::Spo as usize] = PermData::Owned(triples);
        for (p, v) in to_build.into_iter().zip(built) {
            perms[p as usize] = PermData::Owned(v);
        }
        perms
    }

    /// Persists the permutation indexes to `dir` (one raw little-endian `[u32;3]` file
    /// per permutation) so they can be memory-mapped later via [`open`](Self::open) —
    /// the on-disk side of out-of-core querying.
    #[cfg(feature = "mmap")]
    pub fn save(&self, dir: &std::path::Path) -> std::io::Result<()> {
        self.save_with(dir, false)
    }

    /// Like [`save`](Self::save) but writes each permutation BLOCK-COMPRESSED (the
    /// delta+varint format of [`crate::compress`], ~3-5x smaller on disk). The files are
    /// auto-detected by [`open`](Self::open) via [`crate::compress::FILE_MAGIC`], so old
    /// raw directories keep working and the two formats can be mixed.
    #[cfg(feature = "mmap")]
    pub fn save_compressed(&self, dir: &std::path::Path) -> std::io::Result<()> {
        self.save_with(dir, true)
    }

    #[cfg(feature = "mmap")]
    fn save_with(&self, dir: &std::path::Path, compressed: bool) -> std::io::Result<()> {
        std::fs::create_dir_all(dir)?;
        for (i, p) in self.perms.iter().enumerate() {
            // Raw modes borrow zero-copy; a compressed perm is decoded back to raw rows so
            // `save` is total (e.g. a `load_str_compressed` graph can still be persisted).
            let rows: std::borrow::Cow<[[Id; 3]]> = match p {
                PermData::Compressed(c) => std::borrow::Cow::Owned(c.decode_all()),
                _ => std::borrow::Cow::Borrowed(p.as_slice()),
            };
            // A pending delta-overlay is FOLDED into every BUILT permutation on save, so
            // the persisted base always reflects the full current state (unbuilt perms
            // stay empty). The in-memory overlay is untouched (`save` takes `&self`).
            let rows: std::borrow::Cow<[[Id; 3]]> = match &self.overlay {
                Some(ov) if BUILT.contains(&Perm::ALL[i]) => {
                    std::borrow::Cow::Owned(ov.merge(&rows, Perm::ALL[i], [Id::MIN; 3], [Id::MAX; 3]))
                }
                _ => rows,
            };
            let path = dir.join(format!("perm{i}.bin"));
            if compressed && !rows.is_empty() {
                // Unbuilt (empty) permutations stay raw-empty so `open` skips them by size.
                let mut w = std::io::BufWriter::new(std::fs::File::create(path)?);
                crate::compress::CompressedPerm::encode(&rows).write_to(&mut w)?;
                std::io::Write::flush(&mut w)?;
            } else {
                // SAFETY: reinterpret the contiguous [u32;3] rows as bytes for writing.
                let bytes = unsafe { std::slice::from_raw_parts(rows.as_ptr().cast::<u8>(), std::mem::size_of_val(rows.as_ref())) };
                std::fs::write(path, bytes)?;
            }
        }
        self.save_pred_stats(dir)
    }

    /// Persists the per-predicate stats so `open` need not RE-SCAN the POS/PSO indexes
    /// (a ~2-permutation read — the dominant out-of-core open cost + resident RSS once the
    /// dict is mmap'd). Small: a handful of fields per distinct predicate.
    #[cfg(feature = "mmap")]
    pub fn save_pred_stats(&self, dir: &std::path::Path) -> std::io::Result<()> {
        use std::io::Write;
        let mut w = std::io::BufWriter::new(std::fs::File::create(dir.join("predstats.bin"))?);
        w.write_all(&(self.pred_stats.len() as u64).to_le_bytes())?;
        for (&p, s) in self.pred_stats.iter() {
            w.write_all(&p.to_le_bytes())?;
            w.write_all(&(s.count as u64).to_le_bytes())?;
            w.write_all(&(s.ndv_subj as u64).to_le_bytes())?;
            w.write_all(&(s.ndv_obj as u64).to_le_bytes())?;
        }
        w.flush()
    }

    /// Loads persisted per-predicate stats (written by [`save_pred_stats`]); `None` if the
    /// file is absent (an older saved dir) so the caller falls back to recomputing.
    #[cfg(feature = "mmap")]
    fn load_pred_stats(dir: &std::path::Path) -> Option<FxHashMap<Id, PredStat>> {
        use std::io::Read;
        let mut r = std::io::BufReader::new(std::fs::File::open(dir.join("predstats.bin")).ok()?);
        fn rd8(r: &mut impl Read) -> Option<u64> {
            let mut b = [0u8; 8];
            r.read_exact(&mut b).ok()?;
            Some(u64::from_le_bytes(b))
        }
        // The predicate id is written as a little-endian `Id` (u32, 4 bytes) by
        // `save_pred_stats` — this loader used to read 8 bytes for it, mis-framing every
        // record, so the load ALWAYS failed and `open` silently fell back to recomputing
        // the stats, paging in the whole POS+PSO indexes (~24 B/triple of resident memory
        // and most of the out-of-core open time). Measured in research/memory-tiering.md.
        fn rd_id(r: &mut impl Read) -> Option<Id> {
            let mut b = [0u8; std::mem::size_of::<Id>()];
            r.read_exact(&mut b).ok()?;
            Some(Id::from_le_bytes(b))
        }
        // [OPUS-4.8] sq-f5jh: `predstats.bin` is an UNTRUSTED on-disk file (trust boundary
        // B5). `n` is a u64 count read straight from it, and `reserve(n)` was unbounded — a
        // single flipped count byte could ask `FxHashMap` to pre-allocate billions of slots
        // (~17 B each) and ABORT the process (uncatchable OOM DoS; under llvm-cov's added
        // memory pressure this is the residual rc=101 / coverage-undercount trigger). Each
        // record on disk is `size_of::<Id>() + 24` bytes (id + three u64s), so the file
        // length is a hard upper bound on the real record count: clamp the reservation to it
        // (the per-record `read_exact`s below still error cleanly via `?`/`None` if the file
        // actually ends early). We never reserve for more records than can possibly fit.
        let n = rd8(&mut r)? as usize;
        const PREDSTAT_REC_BYTES: usize = std::mem::size_of::<Id>() + 24; // id + count + ndv_subj + ndv_obj
        let file_len = std::fs::metadata(dir.join("predstats.bin")).ok()?.len() as usize;
        let max_records = file_len.saturating_sub(8) / PREDSTAT_REC_BYTES; // 8-byte header
        let mut stats = FxHashMap::default();
        stats.reserve(n.min(max_records));
        for _ in 0..n {
            let p = rd_id(&mut r)?;
            let count = rd8(&mut r)? as usize;
            let ndv_subj = rd8(&mut r)? as usize;
            let ndv_obj = rd8(&mut r)? as usize;
            stats.insert(p, PredStat { count, ndv_subj, ndv_obj });
        }
        Some(stats)
    }

    /// Opens a store whose permutations are MEMORY-MAPPED from `dir` (written by
    /// [`save`](Self::save)). The 6 (or 3, compact) index files stay on disk; the OS
    /// pages in only the ranges a query touches, so datasets larger than RAM are
    /// queryable. Per-predicate stats are recomputed from the mapped POS/PSO indexes.
    #[cfg(feature = "mmap")]
    pub fn open(dir: &std::path::Path) -> std::io::Result<Self> {
        let mut perms: [PermData; 6] = std::array::from_fn(|_| PermData::default());
        for (i, slot) in perms.iter_mut().enumerate() {
            let path = dir.join(format!("perm{i}.bin"));
            let file = std::fs::File::open(&path)?;
            if file.metadata()?.len() == 0 {
                continue; // an empty (unbuilt, e.g. compact-index) permutation
            }
            // SAFETY: the file is owned by this store for its lifetime and is not mutated.
            let map = unsafe { memmap2::Mmap::map(&file)? };
            // FORMAT AUTO-DETECTION: a block-compressed file (written by `save_compressed`)
            // starts with FILE_MAGIC; anything else is the original raw [u32;3] format.
            // Compressed perms are served lazily — block-wise decode off the mapped file.
            *slot = if map.len() >= 8 && map[..8] == crate::compress::FILE_MAGIC {
                PermData::Compressed(crate::compress::CompressedPerm::from_mmap(map)?)
            } else {
                PermData::Mapped(map)
            };
        }
        // Use the persisted stats if present (no POS/PSO re-scan — keeps open fast and the
        // resident set small); else recompute (backward compatible with older saved dirs).
        let pred_stats = Self::load_pred_stats(dir).unwrap_or_else(|| Self::compute_pred_stats(&perms));
        Ok(TripleStore { perms: std::sync::Arc::new(perms), pred_stats: std::sync::Arc::new(pred_stats), overlay: None })
    }

    /// Per-predicate stats: count + distinct objects from POS (always built), and
    /// distinct subjects from PSO when it is built (the full six-permutation index),
    /// else approximated by the count (under `compact-index`, where PSO is absent —
    /// the planner then treats subjects as non-selective, which is safe for ordering).
    fn compute_pred_stats(perms: &[PermData; 6]) -> FxHashMap<Id, PredStat> {
        // Full-range `rows_in`: raw modes borrow the whole slice (zero-copy, as before);
        // a compressed perm (an opened compressed dir missing predstats.bin) is decoded.
        let pos = perms[Perm::Pos as usize].rows_in([Id::MIN; 3], [Id::MAX; 3]); // [P, O, S]
        let pso = perms[Perm::Pso as usize].rows_in([Id::MIN; 3], [Id::MAX; 3]); // [P, S, O], empty under compact-index
        let mut stats: FxHashMap<Id, PredStat> = FxHashMap::default();
        // POS: count + distinct O per P. ndv_subj defaults to count (refined below).
        let mut i = 0;
        while i < pos.len() {
            let p = pos[i][0];
            let (mut count, mut ndv_o, mut last_o) = (0usize, 0usize, None);
            while i < pos.len() && pos[i][0] == p {
                count += 1;
                if last_o != Some(pos[i][1]) {
                    ndv_o += 1;
                    last_o = Some(pos[i][1]);
                }
                i += 1;
            }
            stats.insert(p, PredStat { count, ndv_subj: count, ndv_obj: ndv_o });
        }
        // PSO (when built): exact distinct S per P.
        let mut i = 0;
        while i < pso.len() {
            let p = pso[i][0];
            let (mut ndv_s, mut last_s) = (0usize, None);
            while i < pso.len() && pso[i][0] == p {
                if last_s != Some(pso[i][1]) {
                    ndv_s += 1;
                    last_s = Some(pso[i][1]);
                }
                i += 1;
            }
            stats.entry(p).or_default().ndv_subj = ndv_s;
        }
        stats
    }

    /// Stats for a predicate id (for the cost-based planner), if present.
    pub fn pred_stat(&self, predicate: Id) -> Option<PredStat> {
        self.pred_stats.get(&predicate).copied()
    }

    pub fn len(&self) -> usize {
        let base = self.perms[0].len();
        match &self.overlay {
            Some(ov) => base + ov.added.len() - ov.deleted.len(),
            None => base,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Whether a delta-overlay of pending updates exists (i.e. updates were applied
    /// since the base was built / last compacted).
    pub fn has_overlay(&self) -> bool {
        self.overlay.is_some()
    }

    /// [OPUS-4.8] (sq-5lf) Strong-reference count of the `Arc`-shared base permutation
    /// indexes — i.e. how many stores currently SHARE this exact base storage. 1 for a
    /// freshly built / just-compacted store; bumps by one for each live
    /// [`fork`](Self::fork) / [`Graph::snapshot`](crate::Graph::snapshot) of it. Used to
    /// PROVE structural sharing in tests (a cheap snapshot bumps this count rather than
    /// duplicating the index memory). Two stores share a base iff this is > 1 and they
    /// were derived from the same lineage.
    pub fn base_strong_count(&self) -> usize {
        std::sync::Arc::strong_count(&self.perms)
    }

    /// Whether the store (base merged with any overlay) contains the canonical triple.
    pub fn contains(&self, t: [Id; 3]) -> bool {
        match &self.overlay {
            Some(ov) => {
                !ov.deleted.contains(&t)
                    && (ov.added.binary_search(&t).is_ok() || self.base_contains(t))
            }
            None => self.base_contains(t),
        }
    }

    /// Whether the immutable BASE (ignoring the overlay) contains the canonical triple —
    /// one binary search of the SPO permutation (always built, in every index set).
    #[inline]
    fn base_contains(&self, t: [Id; 3]) -> bool {
        self.perms[Perm::Spo as usize].count_in(t, t) > 0
    }

    /// Applies an update batch as a DELTA-OVERLAY: `deletes` first, then `inserts`
    /// (SPARQL's DELETE/INSERT order), each O(log n + batch · overlay) — instead of the
    /// O(n) rebuild. Set semantics: re-inserting a present triple and deleting an absent
    /// one are no-ops; a delete of a pending insertion simply retracts it. When the
    /// overlay nets out to nothing it is dropped entirely, so an untouched (or fully
    /// reverted) store scans with zero overhead.
    pub fn apply_delta(&mut self, inserts: &[[Id; 3]], deletes: &[[Id; 3]]) {
        if inserts.is_empty() && deletes.is_empty() {
            return;
        }
        let mut ov = self.overlay.take().unwrap_or_default();
        for t in deletes {
            if let Ok(i) = ov.added.binary_search(t) {
                ov.added.remove(i); // retract a pending insertion
            } else if self.base_contains(*t) {
                ov.deleted.insert(*t);
            }
        }
        for t in inserts {
            if ov.deleted.remove(t) {
                continue; // re-insert of a deleted base triple: just undelete
            }
            if self.base_contains(*t) {
                continue; // already present in the base
            }
            if let Err(i) = ov.added.binary_search(t) {
                ov.added.insert(i, *t);
            }
        }
        self.overlay = if ov.is_empty() { None } else { Some(ov) };
    }

    /// Decodes every block-compressed permutation into its raw in-RAM form, so later
    /// scans are pure binary-search slice borrows (zero decode cost) — the LOAD-TIME
    /// DECOMPRESSION mode for an opened compressed directory: pay one full decode up
    /// front, query at exactly raw-store speed. Raw/mapped permutations are untouched.
    ///
    /// Runs on freshly built/opened stores (load-time), which are never yet forked;
    /// on a structurally SHARED store (post-[`fork`](Self::fork)) it is a no-op —
    /// decompression is an optimisation, never a correctness requirement.
    pub fn decompress_to_ram(&mut self) {
        let Some(perms) = std::sync::Arc::get_mut(&mut self.perms) else {
            return; // shared with a fork: leave the (immutable) base untouched
        };
        for slot in perms {
            if let PermData::Compressed(c) = slot {
                *slot = PermData::Owned(c.decode_all());
            }
        }
    }

    /// A structural FORK of this store: the immutable base permutation indexes and
    /// planner stats are SHARED (Arc bumps, O(1)); the pending delta-overlay is
    /// carried by value (O(overlay), bounded by the compaction policy). The fork and
    /// the original then evolve independently through [`apply_delta`](Self::apply_delta)
    /// — neither ever mutates the shared base, so existing readers are unaffected.
    pub fn fork(&self) -> TripleStore {
        TripleStore {
            perms: std::sync::Arc::clone(&self.perms),
            pred_stats: std::sync::Arc::clone(&self.pred_stats),
            overlay: self.overlay.clone(),
        }
    }

    /// Number of pending overlay entries (insertions + deletions) — the input to a
    /// compaction threshold policy (a fork costs O(this); folding it costs O(n)).
    pub fn overlay_len(&self) -> usize {
        self.overlay.as_ref().map_or(0, |ov| ov.added.len() + ov.deleted.len())
    }

    /// Heap footprint of the permutation indexes in bytes (for benchmarking). Memory-
    /// mapped permutations contribute 0 — their resident pages are OS page cache.
    pub fn heap_bytes(&self) -> usize {
        self.perms.iter().map(PermData::heap_bytes).sum::<usize>()
            + self.overlay.as_ref().map_or(0, |ov| ov.heap_bytes())
    }

    /// Chooses the permutation whose sort order places all bound pattern
    /// positions as a contiguous prefix, so the matches form one range. Returns
    /// the permutation and the number of leading bound columns.
    fn choose(pattern: &Pattern) -> (Perm, usize) {
        // Prefer an order where every bound position precedes every unbound one.
        let bound = |i: usize| pattern[i].is_some();
        for &perm in BUILT {
            let order = perm.order();
            // count leading bound columns
            let mut lead = 0;
            while lead < 3 && bound(order[lead]) {
                lead += 1;
            }
            // valid if all bound positions are within the leading prefix
            let total_bound = (0..3).filter(|&i| bound(i)).count();
            if lead == total_bound {
                return (perm, lead);
            }
        }
        (Perm::Spo, 0)
    }

    /// Like [`choose`], but among the permutations whose sort order places every
    /// bound position as a prefix, prefers one whose first *unbound* column is
    /// `sort_col` (a position 0..3 into a canonical triple). This makes the scan
    /// output sorted by that column, enabling a merge join on it.
    fn choose_sorted(pattern: &Pattern, sort_col: usize) -> (Perm, usize) {
        let bound = |i: usize| pattern[i].is_some();
        let total_bound = (0..3).filter(|&i| bound(i)).count();
        // Prefer: bound positions form the leading prefix AND column `sort_col`
        // is the first column after the prefix.
        for &perm in BUILT {
            let order = perm.order();
            let mut lead = 0;
            while lead < 3 && bound(order[lead]) {
                lead += 1;
            }
            if lead == total_bound && lead < 3 && order[lead] == sort_col {
                return (perm, lead);
            }
        }
        Self::choose(pattern)
    }

    /// Returns the contiguous slice of rows (in `perm` order) matching the bound
    /// prefix of the pattern, together with the chosen permutation.
    pub fn scan(&self, pattern: &Pattern) -> Scan<'_> {
        let (perm, lead) = Self::choose(pattern);
        self.scan_with(pattern, perm, lead)
    }

    /// Scans choosing a permutation whose output is sorted by canonical column
    /// `sort_col` (when possible), for merge joins.
    pub fn scan_sorted(&self, pattern: &Pattern, sort_col: usize) -> Scan<'_> {
        let (perm, lead) = Self::choose_sorted(pattern, sort_col);
        self.scan_with(pattern, perm, lead)
    }

    /// The inclusive [lo, hi] key bounds for a pattern's bound prefix in `perm` order.
    #[inline]
    fn bounds(pattern: &Pattern, perm: Perm, lead: usize) -> ([Id; 3], [Id; 3]) {
        let order = perm.order();
        let mut lo = [Id::MIN; 3];
        let mut hi = [Id::MAX; 3];
        for k in 0..lead {
            let v = pattern[order[k]].unwrap();
            lo[k] = v;
            hi[k] = v;
        }
        (lo, hi)
    }

    fn scan_with(&self, pattern: &Pattern, perm: Perm, lead: usize) -> Scan<'_> {
        let (lo, hi) = Self::bounds(pattern, perm, lead);
        let base = self.perms[perm as usize].rows_in(lo, hi);
        // The single overlay branch on the scan hot path: with no pending updates the
        // base range is returned untouched (borrowed, zero copies); with an overlay the
        // deleted triples are filtered out and the inserted ones merge-interleaved, so
        // the rows keep the permutation's sort order (merge joins stay valid).
        let rows = match &self.overlay {
            None => base,
            Some(ov) => std::borrow::Cow::Owned(ov.merge(&base, perm, lo, hi)),
        };
        Scan { rows, perm }
    }

    /// Estimated number of matches for a pattern (the range length) — the cardinality
    /// estimate used by the greedy planner. Cheap for every storage mode: raw modes
    /// subtract binary-search bounds; the compressed mode counts via the block directory
    /// decoding at most two boundary blocks (never the whole range).
    pub fn estimate(&self, pattern: &Pattern) -> usize {
        let (perm, lead) = Self::choose(pattern);
        let (lo, hi) = Self::bounds(pattern, perm, lead);
        let base = self.perms[perm as usize].count_in(lo, hi);
        match &self.overlay {
            None => base,
            Some(ov) => {
                let (add, del) = ov.count_correction(perm, lo, hi);
                base + add - del
            }
        }
    }
}

/// A range of rows in a permutation's column order. Borrowed from the raw index, or
/// owned when decoded from a compressed permutation — uniformly a `&[[Id;3]]` to callers.
pub struct Scan<'a> {
    pub rows: std::borrow::Cow<'a, [[Id; 3]]>,
    pub perm: Perm,
}

impl<'a> Scan<'a> {
    /// Maps a stored row back to a canonical [s,p,o] triple.
    #[inline]
    pub fn to_spo(&self, row: &[Id; 3]) -> [Id; 3] {
        let order = self.perm.order();
        let mut out = [0; 3];
        out[order[0]] = row[0];
        out[order[1]] = row[1];
        out[order[2]] = row[2];
        out
    }
}

/// First index where `rows[i] >= key` comparing only the leading columns that
/// are constrained (MIN acts as -inf in unconstrained columns of `key`).
fn lower_bound(rows: &[[Id; 3]], key: &[Id; 3]) -> usize {
    rows.partition_point(|row| row < key)
}

/// First index where `rows[i] > key` (MAX acts as +inf).
fn upper_bound(rows: &[[Id; 3]], key: &[Id; 3]) -> usize {
    rows.partition_point(|row| row <= key)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The compressed store must answer EVERY triple pattern with the exact same rows
    /// (and the same `estimate`) as the raw store — across all bound/unbound shapes and
    /// many key ranges, including misses and the boundaries of the id space.
    #[test]
    fn compressed_scans_match_raw() {
        // A few-predicate, clustered-subject, sparse-object graph spanning many blocks.
        let mut triples: Vec<[Id; 3]> = Vec::new();
        let mut st = 0x12345677u32;
        let mut rng = || {
            st ^= st << 13;
            st ^= st >> 17;
            st ^= st << 5;
            st
        };
        for _ in 0..40_000 {
            triples.push([1 + rng() % 800, 1 + rng() % 12, 1 + rng() % 5000]);
        }
        let raw = TripleStore::from_triples(triples.clone());
        let cmp = TripleStore::from_triples_compressed(triples);
        assert_eq!(raw.len(), cmp.len());

        let dump = |s: &Scan| {
            let mut v: Vec<[Id; 3]> = s.rows.iter().map(|r| s.to_spo(r)).collect();
            v.sort_unstable();
            v
        };
        // Probe values: present ids, absent ids, and the id-space boundaries.
        let svals = [None, Some(1), Some(400), Some(801), Some(Id::MAX)];
        let pvals = [None, Some(1), Some(6), Some(13)];
        let ovals = [None, Some(1), Some(2500), Some(5001)];
        for &s in &svals {
            for &p in &pvals {
                for &o in &ovals {
                    let pat: Pattern = [s, p, o];
                    let rs = raw.scan(&pat);
                    let cs = cmp.scan(&pat);
                    assert_eq!(dump(&rs), dump(&cs), "rows differ for pattern {pat:?}");
                    // estimate() must equal the true match count for both modes.
                    assert_eq!(cmp.estimate(&pat), dump(&rs).len(), "estimate wrong for {pat:?}");
                }
            }
        }
        // Per-predicate stats must be identical (computed from raw before encoding).
        for p in 1..=13 {
            assert_eq!(raw.pred_stat(p), cmp.pred_stat(p), "pred_stat differs for {p}");
        }
    }
    /// The COMPRESSED on-disk format must round-trip exactly: `save_compressed` → `open`
    /// must answer every triple pattern with the same rows, in the same scan order, with
    /// the same `estimate`, as the raw `save` → `open` of the same store — whether served
    /// lazily (block-wise off the mapped file) or after `decompress_to_ram`. Also checks
    /// FORMAT AUTO-DETECTION (magic header present iff compressed; both dirs open through
    /// the same `open`), the old-format compat (the raw dir keeps working untouched), and
    /// that the compressed files are genuinely smaller.
    #[cfg(feature = "mmap")]
    #[test]
    fn compressed_save_open_roundtrip() {
        let mut triples: Vec<[Id; 3]> = Vec::new();
        let mut st = 0xBEEF5EEDu32;
        let mut rng = || {
            st ^= st << 13;
            st ^= st >> 17;
            st ^= st << 5;
            st
        };
        for _ in 0..60_000 {
            triples.push([1 + rng() % 1500, 1 + rng() % 17, 1 + rng() % 9000]);
        }
        let store = TripleStore::from_triples(triples);

        let base = std::env::temp_dir().join(format!("sparq_cperm_{}", std::process::id()));
        let raw_dir = base.join("raw");
        let cmp_dir = base.join("cmp");
        store.save(&raw_dir).unwrap();
        store.save_compressed(&cmp_dir).unwrap();

        // Auto-detection bytes: compressed BUILT perms carry FILE_MAGIC and are smaller;
        // raw files never start with the magic.
        let mut raw_total = 0u64;
        let mut cmp_total = 0u64;
        for &perm in BUILT {
            let f = format!("perm{}.bin", perm as usize);
            let r = std::fs::read(raw_dir.join(&f)).unwrap();
            let c = std::fs::read(cmp_dir.join(&f)).unwrap();
            assert_ne!(&r[..8], &crate::compress::FILE_MAGIC, "raw {f} must not carry the magic");
            assert_eq!(&c[..8], &crate::compress::FILE_MAGIC, "compressed {f} must carry the magic");
            assert!(c.len() < r.len(), "{f}: compressed ({}) not smaller than raw ({})", c.len(), r.len());
            raw_total += r.len() as u64;
            cmp_total += c.len() as u64;
        }
        assert!(cmp_total * 2 < raw_total, "expected >2x overall perm compression, got {raw_total}/{cmp_total}");

        let raw = TripleStore::open(&raw_dir).unwrap(); // old format: still opens (compat)
        let lazy = TripleStore::open(&cmp_dir).unwrap(); // auto-detected compressed
        let mut eager = TripleStore::open(&cmp_dir).unwrap();
        eager.decompress_to_ram(); // load-time decompression mode
        assert!(matches!(lazy.perms[Perm::Spo as usize], PermData::Compressed(_)), "compressed file not auto-detected");
        assert!(matches!(raw.perms[Perm::Spo as usize], PermData::Mapped(_)), "raw file must stay mmap'd");
        assert!(matches!(eager.perms[Perm::Spo as usize], PermData::Owned(_)), "decompress_to_ram must own the rows");
        assert_eq!(raw.len(), lazy.len());
        assert_eq!(raw.len(), eager.len());

        // Every pattern shape (and the merge-join sorted variants) must yield identical
        // rows in identical order, and identical estimates, across the three stores.
        let svals = [None, Some(1), Some(700), Some(1501)];
        let pvals = [None, Some(1), Some(9), Some(18)];
        let ovals = [None, Some(1), Some(4500), Some(9001)];
        for &s in &svals {
            for &p in &pvals {
                for &o in &ovals {
                    let pat: Pattern = [s, p, o];
                    for sort_col in [None, Some(0), Some(1), Some(2)] {
                        fn scans<'a>(g: &'a TripleStore, pat: &Pattern, sort_col: Option<usize>) -> Scan<'a> {
                            match sort_col {
                                None => g.scan(pat),
                                Some(c) => g.scan_sorted(pat, c),
                            }
                        }
                        let (r, l, e) = (scans(&raw, &pat, sort_col), scans(&lazy, &pat, sort_col), scans(&eager, &pat, sort_col));
                        assert_eq!(r.perm, l.perm);
                        assert_eq!(r.rows, l.rows, "lazy rows differ for {pat:?} sort {sort_col:?}");
                        assert_eq!(r.rows, e.rows, "eager rows differ for {pat:?} sort {sort_col:?}");
                    }
                    assert_eq!(raw.estimate(&pat), lazy.estimate(&pat), "estimate differs for {pat:?}");
                }
            }
        }
        // Pred stats: persisted identically; and recomputable from compressed perms when
        // predstats.bin is missing (the fallback decodes, it must not panic or differ).
        std::fs::remove_file(cmp_dir.join("predstats.bin")).unwrap();
        let refallback = TripleStore::open(&cmp_dir).unwrap();
        for p in 1..=18 {
            assert_eq!(raw.pred_stat(p), lazy.pred_stat(p), "persisted pred_stat differs for {p}");
            assert_eq!(raw.pred_stat(p), refallback.pred_stat(p), "recomputed pred_stat differs for {p}");
        }
        std::fs::remove_dir_all(&base).ok();
    }

    /// `load_pred_stats` must actually LOAD the persisted file — not silently return
    /// `None` and let `open` fall back to recomputing (which pages in the whole POS+PSO
    /// indexes, defeating the point of persisting the stats). Regression test for the
    /// id-width mis-framing bug: the predicate id is written as a 4-byte `Id`, and the
    /// loader read 8 — so every load failed and `compressed_save_open_roundtrip` above
    /// still passed (recomputed == recomputed). Asserts the load is `Some` AND exact.
    #[cfg(feature = "mmap")]
    #[test]
    fn pred_stats_load_is_some_and_exact() {
        let mut triples: Vec<[Id; 3]> = Vec::new();
        let mut st = 0xACCE55u32;
        let mut rng = || {
            st ^= st << 13;
            st ^= st >> 17;
            st ^= st << 5;
            st
        };
        for _ in 0..30_000 {
            triples.push([1 + rng() % 900, 1 + rng() % 23, 1 + rng() % 4000]);
        }
        let store = TripleStore::from_triples(triples);
        let dir = std::env::temp_dir().join(format!("sparq_predstats_{}", std::process::id()));
        store.save(&dir).unwrap();
        let loaded = TripleStore::load_pred_stats(&dir)
            .expect("persisted predstats.bin must load, not fall back to a POS+PSO re-scan");
        assert_eq!(loaded, *store.pred_stats, "loaded pred stats must equal the saved ones");
        std::fs::remove_dir_all(&dir).ok();
    }

    /// The delta-overlay must be INVISIBLE in results: a store with an overlay must answer
    /// every triple pattern with exactly the rows of a store REBUILT from the merged triple
    /// set — and the rows must stay sorted in the scan's permutation order (the guarantee
    /// merge joins rely on). Also: `estimate` stays exact, `len`/`contains` agree, and an
    /// overlay that nets out to nothing is dropped (zero-overhead empty case).
    #[test]
    fn overlay_scans_match_rebuild() {
        let mut st = 0xC0FFEEu32;
        let mut rng = || {
            st ^= st << 13;
            st ^= st >> 17;
            st ^= st << 5;
            st
        };
        let mut triples: Vec<[Id; 3]> = Vec::new();
        for _ in 0..20_000 {
            triples.push([1 + rng() % 400, 1 + rng() % 9, 1 + rng() % 2000]);
        }
        let mut store = TripleStore::from_triples(triples.clone());

        // Deletes: a sample of base triples (plus an absent one — must be a no-op).
        // Inserts: fresh triples (plus a base duplicate — must be a no-op).
        triples.sort_unstable();
        triples.dedup();
        let deletes: Vec<[Id; 3]> = triples.iter().step_by(7).copied().chain([[9999, 9999, 9999]]).collect();
        let mut inserts: Vec<[Id; 3]> = (0..500).map(|_| [401 + rng() % 50, 10 + rng() % 3, 2001 + rng() % 100]).collect();
        inserts.push(triples[3]); // already in the base
        store.apply_delta(&inserts, &deletes);
        assert!(store.has_overlay());

        // Reference: the same set operations applied eagerly, then a full rebuild.
        let mut reference: Vec<[Id; 3]> = triples.clone();
        let del: std::collections::HashSet<[Id; 3]> = deletes.iter().copied().collect();
        reference.retain(|t| !del.contains(t));
        reference.extend(inserts.iter().copied().filter(|t| !del.contains(t)));
        reference.sort_unstable();
        reference.dedup();
        let rebuilt = TripleStore::from_triples(reference.clone());

        assert_eq!(store.len(), rebuilt.len(), "len must reflect the overlay");
        for t in &reference {
            assert!(store.contains(*t));
        }
        assert!(!store.contains([9999, 9999, 9999]));

        let svals = [None, Some(1), Some(200), Some(420), Some(9999)];
        let pvals = [None, Some(1), Some(5), Some(11)];
        let ovals = [None, Some(2), Some(1500), Some(2050)];
        for &s in &svals {
            for &p in &pvals {
                for &o in &ovals {
                    let pat: Pattern = [s, p, o];
                    for sort_col in [None, Some(0), Some(1), Some(2)] {
                        let (ov_scan, rb_scan) = match sort_col {
                            None => (store.scan(&pat), rebuilt.scan(&pat)),
                            Some(c) => (store.scan_sorted(&pat, c), rebuilt.scan_sorted(&pat, c)),
                        };
                        // Rows must be SORTED in the chosen permutation's order…
                        assert!(ov_scan.rows.windows(2).all(|w| w[0] <= w[1]), "unsorted overlay scan for {pat:?}");
                        // …and identical (same perm choice — `choose` is pattern-only) to the rebuild.
                        assert_eq!(ov_scan.perm, rb_scan.perm);
                        assert_eq!(ov_scan.rows, rb_scan.rows, "rows differ for {pat:?} sort {sort_col:?}");
                    }
                    assert_eq!(store.estimate(&pat), rebuilt.scan(&pat).rows.len(), "estimate wrong for {pat:?}");
                }
            }
        }

        // Reverting every EFFECTIVE change must drop the overlay entirely (no residual
        // overhead): re-insert the base triples that were deleted, delete the genuinely
        // new ones (the no-op delete/insert from above never entered the overlay).
        let eff_del: Vec<[Id; 3]> = deletes.iter().copied().filter(|t| triples.binary_search(t).is_ok()).collect();
        let eff_add: Vec<[Id; 3]> = inserts.iter().copied().filter(|t| triples.binary_search(t).is_err()).collect();
        store.apply_delta(&eff_del, &eff_add);
        assert!(!store.has_overlay(), "a fully reverted overlay must be dropped");
        let full = store.scan(&[None, None, None]);
        let mut orig: Vec<[Id; 3]> = triples.clone();
        orig.sort_unstable();
        assert_eq!(full.rows.as_ref(), &orig[..]);
    }
}
