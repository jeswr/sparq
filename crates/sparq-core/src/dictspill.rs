//! Spillable BUILD-time term dictionary (`dict-spill` feature): bound the peak RSS of
//! `Graph::build_external` by a configurable memory budget, dictionary included.
//!
//! The existing external build bounds the TRIPLE-sort memory (chunked runs + k-way
//! merge) but interns every distinct term into RAM-resident dictionaries
//! (`ShardedDict` + `into_merged`), so build RSS grows with distinct terms — measured
//! swap-bound at 1B Wikidata triples on a 16 GB box (research/wikidata-lowresource-
//! stage1.md). This module replaces the in-RAM interning with seq-tagged term
//! SPILLING + external dedup/rank, reproducing the sharded path's id assignment
//! EXACTLY (byte-identical output files). Design: research/external-dictionary.md.
//!
//! Pipeline (all IO is sequential — no random disk access anywhere):
//!   1. ingest    — route each parsed partial's terms to `hash % n` shards (the same
//!                  routing as `ShardedDict::intern_partials`); per shard, a BOUNDED
//!                  dedup cache maps serialized term -> seq (the per-shard occurrence
//!                  counter). Cache misses append the term to the shard's record file
//!                  (seq = record index, implicit). Triples are staged to disk as
//!                  `[u64;3]` (`(shard+1)<<32 | seq`; inline ids pass through).
//!                  Over-budget caches are CLEARED at batch boundaries (an "epoch") —
//!                  re-spilled duplicates collapse at dedup time, so correctness never
//!                  depends on the cache policy, only IO volume does.
//!   2. dedup     — per shard: external sort of the records by (term bytes, seq);
//!                  groups of equal terms yield the distinct term with its MIN seq
//!                  (= first occurrence) plus a (min_seq, seq) pair per occurrence.
//!   3. assign    — per shard in order: external sort of the distinct terms by
//!                  min_seq; final id = base[shard] + rank (the sharded path's
//!                  assignment). The dictionary files (`dict-terms/offs/hash/hid/
//!                  meta.bin`) plus `numerics.bin`/`temporals.bin` are STREAM-written
//!                  in final-id order — never resident.
//!   4. join      — (seq -> min_seq) ⋈ (min_seq -> final id) -> dense per-shard
//!                  `seq -> final id` remap files (two more small external sorts).
//!   5. remap     — stream the staged triples through per-shard sliding windows over
//!                  the remap files (advanced at the recorded epoch boundaries), and
//!                  feed final-id `[u32;3]` triples into the existing
//!                  `extsort::spill_run` / `kway_merge` machinery. Ids are already
//!                  final and dense, so no `remap_perm_file` pass is needed.

use crate::dict::{self, Dict, Id, TermParts};
use rustc_hash::FxHashMap;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

// ---- Configuration + resource detection -------------------------------------------

/// Resource budget for the spilled-dictionary build. Construct via
/// [`SpillConfig::detected`] (defaults from detected RAM) or set the fields explicitly
/// (the benchmarking harness sets explicit budgets for reproducibility).
#[derive(Clone, Debug)]
pub struct SpillConfig {
    /// Memory budget in bytes governing the dedup caches (ingest) and the external-sort
    /// run buffers (consolidation). NOT counted: the triple run buffer (`chunk` × 12 B,
    /// the pre-existing knob) and the parse pipeline's in-flight blocks (~0.5 GiB).
    pub mem_budget: usize,
    /// Free-disk floor in bytes on the build directory's filesystem: the build aborts
    /// (cleanly, before writing) rather than spill below this.
    pub disk_floor: u64,
}

impl SpillConfig {
    /// Defaults from detected resources: budget = 1/4 of physical RAM (min 64 MiB,
    /// fallback 2 GiB when undetectable), disk floor 1 GiB.
    pub fn detected() -> SpillConfig {
        let budget = detected_ram_bytes()
            .map(|r| ((r / 4) as usize).max(64 << 20))
            .unwrap_or(2 << 30);
        SpillConfig { mem_budget: budget, disk_floor: 1 << 30 }
    }

    /// The env-var gate used by [`Graph::build_external`](crate::Graph::build_external):
    /// `SPARQ_DICT_SPILL=1|on|auto` enables the spilled dictionary (`0`/`off`/unset
    /// keeps the in-RAM consolidation), `SPARQ_DICT_SPILL_BUDGET_MB` /
    /// `SPARQ_DICT_SPILL_DISK_FLOOR_MB` override the detected defaults.
    pub fn from_env() -> Option<SpillConfig> {
        match std::env::var("SPARQ_DICT_SPILL").as_deref() {
            Ok("1") | Ok("on") | Ok("auto") => {}
            _ => return None,
        }
        let mut cfg = SpillConfig::detected();
        if let Some(mb) = env_u64("SPARQ_DICT_SPILL_BUDGET_MB") {
            cfg.mem_budget = (mb as usize).saturating_mul(1 << 20).max(1 << 20);
        }
        if let Some(mb) = env_u64("SPARQ_DICT_SPILL_DISK_FLOOR_MB") {
            cfg.disk_floor = mb.saturating_mul(1 << 20);
        }
        Some(cfg)
    }
}

fn env_u64(name: &str) -> Option<u64> {
    std::env::var(name).ok().and_then(|v| v.parse::<u64>().ok())
}

/// Physical RAM in bytes (unix `sysconf`; `None` where undetectable).
pub fn detected_ram_bytes() -> Option<u64> {
    #[cfg(unix)]
    // SAFETY: sysconf is a pure query; negative results mean "unsupported".
    unsafe {
        let pages = libc::sysconf(libc::_SC_PHYS_PAGES);
        let psize = libc::sysconf(libc::_SC_PAGE_SIZE);
        if pages > 0 && psize > 0 {
            return Some(pages as u64 * psize as u64);
        }
        None
    }
    #[cfg(not(unix))]
    None
}

/// Free disk space in bytes available to this process on `path`'s filesystem
/// (unix `statvfs`; `None` where undetectable).
pub fn free_disk_bytes(path: &Path) -> Option<u64> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let c = std::ffi::CString::new(path.as_os_str().as_bytes()).ok()?;
        // SAFETY: zeroed statvfs out-param + NUL-terminated path; checked return.
        unsafe {
            let mut s: libc::statvfs = std::mem::zeroed();
            if libc::statvfs(c.as_ptr(), &mut s) == 0 {
                return Some(s.f_bavail as u64 * s.f_frsize as u64);
            }
            None
        }
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        None
    }
}

/// Errors when the filesystem holding `path` is below the configured free-disk floor
/// (resource-awareness requirement: refuse to fill the disk; fail fast and clean).
pub(crate) fn ensure_disk(path: &Path, floor: u64) -> Result<(), String> {
    if let Some(free) = free_disk_bytes(path) {
        if free < floor {
            return Err(format!(
                "dict-spill: free disk ({} MiB) under {:?} is below the configured floor ({} MiB); \
                 free space or lower SPARQ_DICT_SPILL_DISK_FLOOR_MB",
                free >> 20,
                path,
                floor >> 20
            ));
        }
    }
    Ok(())
}

// ---- Term record serialization ------------------------------------------------------
// A canonical, self-delimiting byte form of a (non-inline, non-triple) term: equal terms
// serialize identically (IRIs are pre-split by the deterministic `split_iri` in the
// partial dicts), so byte equality == term equality and the external sort's grouping is
// exact. Layout: tag, then length-prefixed leading components, last component implicit.

const TAG_IRI: u8 = 0;
const TAG_LIT: u8 = 1;
const TAG_BLANK: u8 = 2;

fn serialize_termparts(tp: &TermParts, out: &mut Vec<u8>) {
    out.clear();
    match tp {
        TermParts::Iri { prefix, suffix } => {
            out.push(TAG_IRI);
            out.extend_from_slice(&(prefix.len() as u32).to_le_bytes());
            out.extend_from_slice(prefix.as_bytes());
            out.extend_from_slice(suffix.as_bytes());
        }
        TermParts::Lit { value, datatype, lang } => {
            out.push(TAG_LIT);
            out.extend_from_slice(&(value.len() as u32).to_le_bytes());
            out.extend_from_slice(value.as_bytes());
            out.extend_from_slice(&(datatype.len() as u32).to_le_bytes());
            out.extend_from_slice(datatype.as_bytes());
            match lang {
                Some(l) => {
                    out.push(1);
                    out.extend_from_slice(l.as_bytes());
                }
                None => out.push(0),
            }
        }
        TermParts::Blank(b) => {
            out.push(TAG_BLANK);
            out.extend_from_slice(b.as_bytes());
        }
        TermParts::Triple(_) => {
            panic!("RDF-star triple terms are not supported by the spilled-dict bulk interner; use the serial loader")
        }
    }
}

/// Borrowed view of a serialized term record (the inverse of `serialize_termparts`).
fn parse_term(bytes: &[u8]) -> TermParts<'_> {
    #[inline]
    fn rd<'a>(b: &'a [u8], p: &mut usize) -> &'a str {
        let n = u32::from_le_bytes([b[*p], b[*p + 1], b[*p + 2], b[*p + 3]]) as usize;
        *p += 4;
        let s = &b[*p..*p + n];
        *p += n;
        // SAFETY: written from a `&str` in `serialize_termparts`.
        unsafe { std::str::from_utf8_unchecked(s) }
    }
    #[inline]
    fn rest(b: &[u8], p: usize) -> &str {
        // SAFETY: written from a `&str` in `serialize_termparts`.
        unsafe { std::str::from_utf8_unchecked(&b[p..]) }
    }
    let mut p = 1;
    match bytes[0] {
        TAG_IRI => {
            let prefix = rd(bytes, &mut p);
            TermParts::Iri { prefix, suffix: rest(bytes, p) }
        }
        TAG_LIT => {
            let value = rd(bytes, &mut p);
            let datatype = rd(bytes, &mut p);
            let lang = if bytes[p] == 1 { Some(rest(bytes, p + 1)) } else { None };
            TermParts::Lit { value, datatype, lang }
        }
        _ => TermParts::Blank(rest(bytes, p)),
    }
}

// ---- External sort: variable-length term records -------------------------------------

/// Sort key for [`VarSorter`]: by term bytes then seq (dedup grouping; the first of an
/// equal group carries the min seq), or by seq alone (seqs are unique).
#[derive(Clone, Copy, PartialEq, Eq)]
enum VarKey {
    BytesThenSeq,
    SeqOnly,
}

#[inline]
fn var_cmp(mode: VarKey, a: &(u32, Box<[u8]>), b: &(u32, Box<[u8]>)) -> std::cmp::Ordering {
    match mode {
        VarKey::BytesThenSeq => a.1.cmp(&b.1).then(a.0.cmp(&b.0)),
        VarKey::SeqOnly => a.0.cmp(&b.0),
    }
}

/// Bounded-memory external sorter for `(seq, term bytes)` records: RAM runs of at most
/// `budget` accounted bytes are sorted (rayon) and spilled, then k-way merged.
struct VarSorter {
    mode: VarKey,
    budget: usize,
    buf: Vec<(u32, Box<[u8]>)>,
    bytes: usize,
    runs: Vec<PathBuf>,
    tmp: PathBuf,
    tag: String,
}

/// Approximate per-record bookkeeping overhead added to the payload bytes for budget
/// accounting: the allocator's per-`Box<[u8]>` overhead (~48 B for small allocations)
/// plus the 24 B `(u32, Box<[u8]>)` sort slot — measured against real RSS at 100M
/// (the original 40 B figure understated runs by ~40%).
const VAR_REC_OVERHEAD: usize = 72;

impl VarSorter {
    fn new(mode: VarKey, budget: usize, tmp: &Path, tag: &str) -> VarSorter {
        VarSorter {
            mode,
            budget: budget.max(1 << 16),
            buf: Vec::new(),
            bytes: 0,
            runs: Vec::new(),
            tmp: tmp.to_path_buf(),
            tag: tag.to_string(),
        }
    }

    fn push(&mut self, seq: u32, bytes: Box<[u8]>) -> io::Result<()> {
        self.bytes += bytes.len() + VAR_REC_OVERHEAD;
        self.buf.push((seq, bytes));
        if self.bytes >= self.budget {
            self.flush()?;
        }
        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.buf.is_empty() {
            return Ok(());
        }
        let mode = self.mode;
        #[cfg(feature = "parallel")]
        {
            use rayon::prelude::*;
            self.buf.par_sort_unstable_by(|a, b| var_cmp(mode, a, b));
        }
        #[cfg(not(feature = "parallel"))]
        self.buf.sort_unstable_by(|a, b| var_cmp(mode, a, b));
        let path = self.tmp.join(format!("{}-run{}.bin", self.tag, self.runs.len()));
        let mut w = BufWriter::new(std::fs::File::create(&path)?);
        for (seq, bytes) in self.buf.drain(..) {
            w.write_all(&seq.to_le_bytes())?;
            w.write_all(&(bytes.len() as u32).to_le_bytes())?;
            w.write_all(&bytes)?;
        }
        w.flush()?;
        self.runs.push(path);
        self.bytes = 0;
        Ok(())
    }

    fn into_merge(mut self) -> io::Result<VarMerge> {
        self.flush()?;
        let mode = self.mode;
        let mut readers: Vec<BufReader<std::fs::File>> = Vec::with_capacity(self.runs.len());
        for p in &self.runs {
            readers.push(BufReader::new(std::fs::File::open(p)?));
        }
        let mut m = VarMerge { mode, readers, heap: std::collections::BinaryHeap::new(), runs: std::mem::take(&mut self.runs) };
        for i in 0..m.readers.len() {
            m.refill(i)?;
        }
        Ok(m)
    }
}

struct VarHeapEnt {
    mode: VarKey,
    seq: u32,
    bytes: Box<[u8]>,
    run: usize,
}
impl PartialEq for VarHeapEnt {
    fn eq(&self, o: &Self) -> bool {
        self.cmp(o) == std::cmp::Ordering::Equal
    }
}
impl Eq for VarHeapEnt {}
impl PartialOrd for VarHeapEnt {
    fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(o))
    }
}
impl Ord for VarHeapEnt {
    fn cmp(&self, o: &Self) -> std::cmp::Ordering {
        // REVERSED (min-heap via the max-`BinaryHeap`) + run index for total determinism.
        let fwd = match self.mode {
            VarKey::BytesThenSeq => self.bytes.cmp(&o.bytes).then(self.seq.cmp(&o.seq)),
            VarKey::SeqOnly => self.seq.cmp(&o.seq),
        }
        .then(self.run.cmp(&o.run));
        fwd.reverse()
    }
}

/// K-way merge over a [`VarSorter`]'s runs, yielding `(seq, bytes)` in key order.
/// Run files are deleted on drop.
struct VarMerge {
    mode: VarKey,
    readers: Vec<BufReader<std::fs::File>>,
    heap: std::collections::BinaryHeap<VarHeapEnt>,
    runs: Vec<PathBuf>,
}

impl VarMerge {
    fn refill(&mut self, run: usize) -> io::Result<()> {
        let r = &mut self.readers[run];
        let mut head = [0u8; 8];
        match read_full(r, &mut head)? {
            false => Ok(()), // run exhausted
            true => {
                let seq = u32::from_le_bytes([head[0], head[1], head[2], head[3]]);
                let len = u32::from_le_bytes([head[4], head[5], head[6], head[7]]) as usize;
                let mut bytes = vec![0u8; len];
                r.read_exact(&mut bytes)?;
                self.heap.push(VarHeapEnt { mode: self.mode, seq, bytes: bytes.into_boxed_slice(), run });
                Ok(())
            }
        }
    }

    fn next(&mut self) -> io::Result<Option<(u32, Box<[u8]>)>> {
        match self.heap.pop() {
            None => Ok(None),
            Some(e) => {
                self.refill(e.run)?;
                Ok(Some((e.seq, e.bytes)))
            }
        }
    }
}

impl Drop for VarMerge {
    fn drop(&mut self) {
        for p in &self.runs {
            std::fs::remove_file(p).ok();
        }
    }
}

/// Reads exactly `buf.len()` bytes; `Ok(false)` on clean EOF at a record boundary.
fn read_full(r: &mut impl Read, buf: &mut [u8]) -> io::Result<bool> {
    let mut filled = 0;
    while filled < buf.len() {
        let n = r.read(&mut buf[filled..])?;
        if n == 0 {
            if filled == 0 {
                return Ok(false);
            }
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "truncated dict-spill record"));
        }
        filled += n;
    }
    Ok(true)
}

// ---- External sort: fixed-size records ------------------------------------------------

/// A fixed-size, manually-serialized record (no struct padding reaches the disk).
trait PodRec: Copy + Ord {
    const SIZE: usize;
    fn write(&self, w: &mut impl Write) -> io::Result<()>;
    fn read(b: &[u8]) -> Self;
}

/// `(min_seq, seq)` occurrence pair, sorted by min_seq (then seq — unique).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MinSeqPair {
    min_seq: u32,
    seq: u32,
}
impl PodRec for MinSeqPair {
    const SIZE: usize = 8;
    fn write(&self, w: &mut impl Write) -> io::Result<()> {
        w.write_all(&self.min_seq.to_le_bytes())?;
        w.write_all(&self.seq.to_le_bytes())
    }
    fn read(b: &[u8]) -> Self {
        MinSeqPair {
            min_seq: u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
            seq: u32::from_le_bytes([b[4], b[5], b[6], b[7]]),
        }
    }
}

/// `(seq, final id)` remap entry, sorted by seq (unique).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct SeqFinal {
    seq: u32,
    final_id: u32,
}
impl PodRec for SeqFinal {
    const SIZE: usize = 8;
    fn write(&self, w: &mut impl Write) -> io::Result<()> {
        w.write_all(&self.seq.to_le_bytes())?;
        w.write_all(&self.final_id.to_le_bytes())
    }
    fn read(b: &[u8]) -> Self {
        SeqFinal {
            seq: u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
            final_id: u32::from_le_bytes([b[4], b[5], b[6], b[7]]),
        }
    }
}

/// `(content hash, id)` lookup-index entry, sorted by (hash, id) — the CANONICAL tie
/// order `Dict::save_mmap` now also uses, so the streamed `dict-hash.bin`/`dict-hid.bin`
/// are byte-identical to the in-RAM path's.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct HashPair {
    hash: u64,
    id: u32,
}
impl PodRec for HashPair {
    const SIZE: usize = 12;
    fn write(&self, w: &mut impl Write) -> io::Result<()> {
        w.write_all(&self.hash.to_le_bytes())?;
        w.write_all(&self.id.to_le_bytes())
    }
    fn read(b: &[u8]) -> Self {
        HashPair {
            hash: u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]),
            id: u32::from_le_bytes([b[8], b[9], b[10], b[11]]),
        }
    }
}

/// Bounded-memory external sorter for fixed-size records (same run/merge shape as
/// [`VarSorter`], without the per-record allocation).
struct PodSorter<T: PodRec> {
    budget_elems: usize,
    buf: Vec<T>,
    runs: Vec<PathBuf>,
    tmp: PathBuf,
    tag: String,
}

impl<T: PodRec + Send> PodSorter<T> {
    fn new(budget_bytes: usize, tmp: &Path, tag: &str) -> PodSorter<T> {
        PodSorter {
            // Budget by the IN-RAM element size (with alignment padding — e.g. `HashPair`
            // is 12 B on disk but 16 B in the run buffer), not the serialized size.
            budget_elems: (budget_bytes / std::mem::size_of::<T>().max(1)).max(1 << 12),
            buf: Vec::new(),
            runs: Vec::new(),
            tmp: tmp.to_path_buf(),
            tag: tag.to_string(),
        }
    }

    fn push(&mut self, t: T) -> io::Result<()> {
        self.buf.push(t);
        if self.buf.len() >= self.budget_elems {
            self.flush()?;
        }
        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.buf.is_empty() {
            return Ok(());
        }
        #[cfg(feature = "parallel")]
        {
            use rayon::prelude::*;
            self.buf.par_sort_unstable();
        }
        #[cfg(not(feature = "parallel"))]
        self.buf.sort_unstable();
        let path = self.tmp.join(format!("{}-run{}.bin", self.tag, self.runs.len()));
        let mut w = BufWriter::new(std::fs::File::create(&path)?);
        for t in self.buf.drain(..) {
            t.write(&mut w)?;
        }
        w.flush()?;
        self.runs.push(path);
        Ok(())
    }

    fn into_merge(mut self) -> io::Result<PodMerge<T>> {
        self.flush()?;
        let mut readers: Vec<BufReader<std::fs::File>> = Vec::with_capacity(self.runs.len());
        for p in &self.runs {
            readers.push(BufReader::new(std::fs::File::open(p)?));
        }
        let mut m = PodMerge { readers, heap: std::collections::BinaryHeap::new(), runs: std::mem::take(&mut self.runs) };
        for i in 0..m.readers.len() {
            m.refill(i)?;
        }
        Ok(m)
    }
}

/// K-way merge over a [`PodSorter`]'s runs (records in `Ord` order); runs deleted on drop.
struct PodMerge<T: PodRec> {
    readers: Vec<BufReader<std::fs::File>>,
    heap: std::collections::BinaryHeap<std::cmp::Reverse<(T, usize)>>,
    runs: Vec<PathBuf>,
}

impl<T: PodRec> PodMerge<T> {
    fn refill(&mut self, run: usize) -> io::Result<()> {
        let mut b = [0u8; 16];
        debug_assert!(T::SIZE <= 16);
        if read_full(&mut self.readers[run], &mut b[..T::SIZE])? {
            self.heap.push(std::cmp::Reverse((T::read(&b), run)));
        }
        Ok(())
    }

    fn next(&mut self) -> io::Result<Option<T>> {
        match self.heap.pop() {
            None => Ok(None),
            Some(std::cmp::Reverse((t, run))) => {
                self.refill(run)?;
                Ok(Some(t))
            }
        }
    }
}

impl<T: PodRec> Drop for PodMerge<T> {
    fn drop(&mut self) {
        for p in &self.runs {
            std::fs::remove_file(p).ok();
        }
    }
}

// ---- Ingest-side interner --------------------------------------------------------------

/// Approximate per-entry overhead of the dedup cache (Box header + map slot) for budget
/// accounting, added to the term payload bytes.
const CACHE_ENTRY_OVERHEAD: usize = 48;

struct ShardState {
    /// serialized term -> seq, BOUNDED: cleared (epoch reset) when over budget.
    cache: FxHashMap<Box<[u8]>, u32>,
    cache_bytes: usize,
    /// Per-shard occurrence counter; a cache miss's seq equals its record index.
    seq: u32,
    rec: BufWriter<std::fs::File>,
    rec_path: PathBuf,
    /// `(staged-triple count, seq)` at each cache clear — the epoch boundaries that
    /// bound the remap windows in phase 5.
    epochs: Vec<(u64, u32)>,
}

impl ShardState {
    /// Resolve a serialized term to its seq: cache hit reuses, miss assigns the next
    /// seq, spills the record, and caches it.
    fn resolve(&mut self, bytes: &[u8]) -> io::Result<u32> {
        if let Some(&seq) = self.cache.get(bytes) {
            return Ok(seq);
        }
        let seq = self.seq;
        self.seq = self.seq.checked_add(1).ok_or_else(|| {
            io::Error::new(io::ErrorKind::Other, "dict-spill: shard exceeded 2^32 spilled occurrences")
        })?;
        self.rec.write_all(&(bytes.len() as u32).to_le_bytes())?;
        self.rec.write_all(bytes)?;
        self.cache_bytes += bytes.len() + CACHE_ENTRY_OVERHEAD;
        self.cache.insert(bytes.into(), seq);
        Ok(seq)
    }
}

/// The ingest-side state of the spilled-dictionary build: per-shard dedup caches +
/// record spill files, and the staged `[u64;3]` triple file.
pub(crate) struct SpillInterner {
    shards: Vec<ShardState>,
    staged: BufWriter<std::fs::File>,
    staged_path: PathBuf,
    staged_count: u64,
    cache_budget_per_shard: usize,
}

/// Staged-triple encoding: values < 2^32 are pass-through inline ids; otherwise
/// `(shard+1) << 32 | seq`.
const STAGED_DICT_BASE: u64 = 1 << 32;

impl SpillInterner {
    pub(crate) fn new(n: usize, tmp: &Path, cfg: &SpillConfig) -> io::Result<SpillInterner> {
        let n = n.max(1);
        let mut shards = Vec::with_capacity(n);
        for s in 0..n {
            let rec_path = tmp.join(format!("dsp-rec{s}.bin"));
            shards.push(ShardState {
                cache: FxHashMap::default(),
                cache_bytes: 0,
                seq: 0,
                rec: BufWriter::new(std::fs::File::create(&rec_path)?),
                rec_path,
                epochs: vec![(0, 0)],
            });
        }
        let staged_path = tmp.join("dsp-staged.bin");
        Ok(SpillInterner {
            shards,
            staged: BufWriter::new(std::fs::File::create(&staged_path)?),
            staged_path,
            staged_count: 0,
            // Ingest phase: the dedup caches get half the budget (the other half is
            // reserved so the consolidation's sort buffers never exceed the same bound).
            cache_budget_per_shard: (cfg.mem_budget / 2 / n).max(1 << 12),
        })
    }

    /// Ingest one parsed batch — the spill analogue of `ShardedDict::intern_partials`
    /// followed by the triple remap+spill: routes every partial's terms to shards in
    /// the SAME `(partial, local-id)` walk order (so per-shard first-occurrence order —
    /// and therefore the final id assignment — is identical to the sharded path),
    /// resolves them through the bounded caches, and stages the triples with the
    /// resulting `(shard, seq)` ids. Over-budget caches are cleared afterwards (batch
    /// boundaries only, so every staged reference points into its own epoch's window).
    pub(crate) fn intern_batch(&mut self, partials: &[(Dict, Vec<[Id; 3]>)]) -> Result<(), String> {
        use rayon::prelude::*;
        let n = self.shards.len();

        // Route + serialize (parallel over partials; same hash routing as the sharded path).
        let routed: Vec<Vec<Vec<(Id, Box<[u8]>)>>> = partials
            .par_iter()
            .map(|(pd, _)| {
                let mut b: Vec<Vec<(Id, Box<[u8]>)>> = (0..n).map(|_| Vec::with_capacity(pd.len() / n + 1)).collect();
                let mut scratch: Vec<u8> = Vec::new();
                for i in 1..=pd.len() as Id {
                    let tp = pd.term_parts(i);
                    let s = (dict::hash_termparts(&tp) % n as u64) as usize;
                    serialize_termparts(&tp, &mut scratch);
                    b[s].push((i, scratch.as_slice().into()));
                }
                b
            })
            .collect();

        // Per-partial remap (local id -> staged u64 id), scattered by the shards with
        // DISJOINT writes — same SlotPtr pattern (and safety argument) as
        // `ShardedDict::intern_partials`: the hash routing assigns every (partial,
        // local-id) slot to exactly one shard.
        let mut remaps: Vec<Vec<u64>> = partials.iter().map(|(pd, _)| vec![0u64; pd.len() + 1]).collect();
        #[derive(Clone, Copy)]
        struct SlotPtr(*mut u64, usize);
        unsafe impl Send for SlotPtr {}
        unsafe impl Sync for SlotPtr {}
        let scatter: Vec<SlotPtr> = remaps.iter_mut().map(|v| SlotPtr(v.as_mut_ptr(), v.len())).collect();

        self.shards
            .par_iter_mut()
            .enumerate()
            .try_for_each(|(s, shard)| -> io::Result<()> {
                for (pidx, per_partial) in routed.iter().enumerate() {
                    let SlotPtr(ptr, len) = scatter[pidx];
                    for (i, bytes) in &per_partial[s] {
                        let seq = shard.resolve(bytes)?;
                        debug_assert!((*i as usize) < len);
                        // SAFETY: i < len and this (pidx, i) slot is hash-routed to
                        // exactly this shard — disjoint writes, no reads until the
                        // parallel scope ends.
                        unsafe { ptr.add(*i as usize).write(STAGED_DICT_BASE * (s as u64 + 1) | seq as u64) };
                    }
                }
                Ok(())
            })
            .map_err(|e| e.to_string())?;

        // Stage the batch's triples (sequential append; 24 B/triple).
        for (pidx, (_, ptriples)) in partials.iter().enumerate() {
            let rm = &remaps[pidx];
            let mut out: Vec<u8> = Vec::with_capacity(ptriples.len() * 24);
            for &[a, b, c] in ptriples {
                for id in [a, b, c] {
                    let v = if id >= dict::INLINE_BASE { id as u64 } else { rm[id as usize] };
                    out.extend_from_slice(&v.to_le_bytes());
                }
            }
            self.staged.write_all(&out).map_err(|e| e.to_string())?;
            self.staged_count += ptriples.len() as u64;
        }

        // Batch boundary: clear over-budget caches (epoch reset). Clearing ONLY here
        // keeps the invariant that a staged reference's seq lies in the epoch that was
        // current when its triple was staged.
        for shard in &mut self.shards {
            if shard.cache_bytes > self.cache_budget_per_shard {
                shard.cache.clear();
                shard.cache_bytes = 0;
                shard.epochs.push((self.staged_count, shard.seq));
            }
        }
        Ok(())
    }
}

// ---- Consolidation: external dedup/rank + streamed dictionary write ---------------------

/// What phase 5 (triple remap) needs: the staged triples plus per-shard dense remap
/// files with their epoch boundaries.
pub(crate) struct RemapPlan {
    staged_path: PathBuf,
    staged_count: u64,
    shards: Vec<ShardRemap>,
}

struct ShardRemap {
    remap_path: PathBuf,
    /// `(first staged-triple index, first seq)` per epoch, terminated by a
    /// `(u64::MAX, total seq)` sentinel.
    epochs: Vec<(u64, u32)>,
}

/// Streaming builder of the unified prefix/datatype tables (first-appearance order over
/// terms streamed in final-id order — provably the same tables `ShardedDict::into_merged`
/// builds, see research/external-dictionary.md).
#[derive(Default)]
struct TableBuilder {
    names: Vec<Box<str>>,
    index: FxHashMap<Box<str>, u32>,
}

impl TableBuilder {
    fn intern(&mut self, s: &str) -> u32 {
        if let Some(&i) = self.index.get(s) {
            return i;
        }
        let i = self.names.len() as u32;
        let boxed: Box<str> = s.into();
        self.names.push(boxed.clone());
        self.index.insert(boxed, i);
        i
    }
}

/// Phases 2–4: externally dedup + rank the spilled terms, STREAM-write the dictionary
/// (`dict-meta/terms/offs/hash/hid.bin`) and the `numerics.bin`/`temporals.bin` caches in
/// final-id order, and build the per-shard `seq -> final id` remap files. Returns the
/// plan for the triple-remap phase.
pub(crate) fn consolidate(mut st: SpillInterner, dir: &Path, tmp: &Path, cfg: &SpillConfig) -> Result<RemapPlan, String> {
    let n = st.shards.len();
    let sort_budget = (cfg.mem_budget / 2).max(1 << 20);
    let io_err = |e: io::Error| e.to_string();

    // Finish ingest: flush + drop the staged writer and record writers, free the caches.
    st.staged.flush().map_err(io_err)?;
    let staged_path = st.staged_path.clone();
    let staged_count = st.staged_count;
    let mut shard_files: Vec<(PathBuf, u32, Vec<(u64, u32)>)> = Vec::with_capacity(n);
    for mut s in std::mem::take(&mut st.shards) {
        s.rec.flush().map_err(io_err)?;
        s.cache = FxHashMap::default();
        shard_files.push((s.rec_path, s.seq, s.epochs));
    }
    drop(st);
    ensure_disk(tmp, cfg.disk_floor)?;

    // Phase 2 — per shard: external sort records by (term, seq); scan equal-term groups
    // to produce the distinct stream (term + min seq) and a (min_seq, seq) pair per
    // occurrence. Shards run sequentially so the sort budget is never multiplied; the
    // run sorts inside are rayon-parallel.
    let mut counts: Vec<u64> = Vec::with_capacity(n);
    for (s, (rec_path, seq_count, _)) in shard_files.iter().enumerate() {
        let mut sorter = VarSorter::new(VarKey::BytesThenSeq, sort_budget, tmp, &format!("dsp-s{s}-byterm"));
        {
            let mut r = BufReader::new(std::fs::File::open(rec_path).map_err(io_err)?);
            let mut head = [0u8; 4];
            let mut seq: u32 = 0;
            while read_full(&mut r, &mut head).map_err(io_err)? {
                let len = u32::from_le_bytes(head) as usize;
                let mut bytes = vec![0u8; len];
                r.read_exact(&mut bytes).map_err(io_err)?;
                sorter.push(seq, bytes.into_boxed_slice()).map_err(io_err)?;
                seq += 1;
            }
            debug_assert_eq!(seq, *seq_count);
        }
        std::fs::remove_file(rec_path).ok();
        ensure_disk(tmp, cfg.disk_floor)?;

        let mut distinct_w = BufWriter::new(std::fs::File::create(tmp.join(format!("dsp-distinct{s}.bin"))).map_err(io_err)?);
        let mut pairs_w = BufWriter::new(std::fs::File::create(tmp.join(format!("dsp-pairs{s}.bin"))).map_err(io_err)?);
        let mut merge = sorter.into_merge().map_err(io_err)?;
        let mut cur: Option<(Box<[u8]>, u32)> = None;
        let mut count: u64 = 0;
        while let Some((seq, bytes)) = merge.next().map_err(io_err)? {
            let same = matches!(&cur, Some((cb, _)) if **cb == *bytes);
            if same {
                let min = cur.as_ref().unwrap().1;
                MinSeqPair { min_seq: min, seq }.write(&mut pairs_w).map_err(io_err)?;
            } else {
                distinct_w.write_all(&seq.to_le_bytes()).map_err(io_err)?;
                distinct_w.write_all(&(bytes.len() as u32).to_le_bytes()).map_err(io_err)?;
                distinct_w.write_all(&bytes).map_err(io_err)?;
                MinSeqPair { min_seq: seq, seq }.write(&mut pairs_w).map_err(io_err)?;
                cur = Some((bytes, seq));
                count += 1;
            }
        }
        distinct_w.flush().map_err(io_err)?;
        pairs_w.flush().map_err(io_err)?;
        counts.push(count);
    }

    // Final id bases (the sharded path's `bases()`): base[s] = distinct terms in shards < s.
    let mut base: Vec<u64> = Vec::with_capacity(n + 1);
    let mut acc = 0u64;
    base.push(0);
    for &c in &counts {
        acc += c;
        base.push(acc);
    }
    let total = acc;
    if total >= dict::INLINE_BASE as u64 {
        return Err("dictionary exceeded the id capacity (2^31 distinct non-integer terms); widen Id to u64".into());
    }

    // Phase 3 — per shard IN ORDER: sort the distinct stream by min_seq (first-occurrence
    // rank == the sharded path's local id order) and stream-write every dictionary file
    // in final-id order. Nothing term-shaped stays resident.
    ensure_disk(dir, cfg.disk_floor)?;
    let mut prefixes = TableBuilder::default();
    let mut datatypes = TableBuilder::default();
    let mut terms_w = BufWriter::new(std::fs::File::create(dir.join("dict-terms.bin")).map_err(io_err)?);
    let mut offs_w = BufWriter::new(std::fs::File::create(dir.join("dict-offs.bin")).map_err(io_err)?);
    let mut numer_w = BufWriter::new(std::fs::File::create(dir.join("numerics.bin")).map_err(io_err)?);
    let mut tempi_w = BufWriter::new(std::fs::File::create(dir.join("temporals.bin")).map_err(io_err)?);
    let flags_path = tmp.join("dsp-tflags.bin");
    let mut flags_w = BufWriter::new(std::fs::File::create(&flags_path).map_err(io_err)?);
    // The hash-pair sorter is alive across the WHOLE shard loop, concurrently with each
    // shard's distinct-stream sorter — partition the budget between them so their summed
    // run buffers stay within `sort_budget`.
    let mut hash_sorter: PodSorter<HashPair> = PodSorter::new(sort_budget / 4, tmp, "dsp-hash");
    let mut pos: u64 = 0;
    for s in 0..n {
        let distinct_path = tmp.join(format!("dsp-distinct{s}.bin"));
        let mut sorter = VarSorter::new(VarKey::SeqOnly, sort_budget / 2, tmp, &format!("dsp-s{s}-byseq"));
        {
            let mut r = BufReader::new(std::fs::File::open(&distinct_path).map_err(io_err)?);
            let mut head = [0u8; 8];
            while read_full(&mut r, &mut head).map_err(io_err)? {
                let min_seq = u32::from_le_bytes([head[0], head[1], head[2], head[3]]);
                let len = u32::from_le_bytes([head[4], head[5], head[6], head[7]]) as usize;
                let mut bytes = vec![0u8; len];
                r.read_exact(&mut bytes).map_err(io_err)?;
                sorter.push(min_seq, bytes.into_boxed_slice()).map_err(io_err)?;
            }
        }
        std::fs::remove_file(&distinct_path).ok();

        let mut m2f_w = BufWriter::new(std::fs::File::create(tmp.join(format!("dsp-m2f{s}.bin"))).map_err(io_err)?);
        let mut merge = sorter.into_merge().map_err(io_err)?;
        let mut local: u64 = 0;
        while let Some((min_seq, bytes)) = merge.next().map_err(io_err)? {
            local += 1;
            let id = (base[s] + local) as Id; // 1-based dense final id
            let tp = parse_term(&bytes);
            // Stream the dict record (the exact `save_mmap` bytes), its offset, the
            // (hash, id) lookup pair, and the numeric/temporal cache cells.
            offs_w.write_all(&pos.to_le_bytes()).map_err(io_err)?;
            let (rec, numeric, temporal): (dict::StoredRef, f64, Option<crate::temporal::Temporal>) = match tp {
                TermParts::Iri { prefix, suffix } => {
                    (dict::StoredRef::Iri { prefix: prefixes.intern(prefix), suffix }, f64::NAN, None)
                }
                TermParts::Lit { value, datatype, lang } => {
                    let numeric = if lang.is_none() && crate::is_numeric_datatype_str(datatype) {
                        value.parse::<f64>().unwrap_or(f64::NAN)
                    } else {
                        f64::NAN
                    };
                    let temporal = if lang.is_none() { crate::temporal::Temporal::of_lit(value, datatype) } else { None };
                    (dict::StoredRef::Lit { value, datatype: datatypes.intern(datatype), lang }, numeric, temporal)
                }
                TermParts::Blank(b) => (dict::StoredRef::Blank(b), f64::NAN, None),
                TermParts::Triple(_) => unreachable!("triple terms cannot enter the spilled interner"),
            };
            pos += dict::write_record(&mut terms_w, &rec).map_err(io_err)?;
            hash_sorter.push(HashPair { hash: dict::hash_termparts(&tp), id }).map_err(io_err)?;
            numer_w.write_all(&numeric.to_le_bytes()).map_err(io_err)?;
            let (instant, flag) = match temporal {
                Some(t) => (t.instant, crate::temp_flag(t)),
                None => (f64::NAN, 0u8),
            };
            tempi_w.write_all(&instant.to_le_bytes()).map_err(io_err)?;
            flags_w.write_all(&[flag]).map_err(io_err)?;
            m2f_w.write_all(&min_seq.to_le_bytes()).map_err(io_err)?;
            m2f_w.write_all(&(id as u32).to_le_bytes()).map_err(io_err)?;
        }
        debug_assert_eq!(local, counts[s]);
        m2f_w.flush().map_err(io_err)?;
    }
    terms_w.flush().map_err(io_err)?;
    offs_w.flush().map_err(io_err)?;
    numer_w.flush().map_err(io_err)?;

    // temporals.bin layout = all instants then all flags: append the spilled flags.
    flags_w.flush().map_err(io_err)?;
    drop(flags_w);
    {
        let mut fr = BufReader::new(std::fs::File::open(&flags_path).map_err(io_err)?);
        std::io::copy(&mut fr, &mut tempi_w).map_err(io_err)?;
        tempi_w.flush().map_err(io_err)?;
        std::fs::remove_file(&flags_path).ok();
    }

    // dict-meta.bin (version header + prefixes + datatypes + term count) — identical bytes to
    // `save_mmap`. [OPUS-4.8] (review 1409) The version header records the id-space partition.
    {
        let mut meta = BufWriter::new(std::fs::File::create(dir.join("dict-meta.bin")).map_err(io_err)?);
        meta.write_all(&dict::DICT_META_MAGIC.to_le_bytes()).map_err(io_err)?;
        meta.write_all(&dict::DICT_META_VERSION.to_le_bytes()).map_err(io_err)?;
        meta.write_all(&dict::INLINE_BASE.to_le_bytes()).map_err(io_err)?;
        meta.write_all(&(prefixes.names.len() as u32).to_le_bytes()).map_err(io_err)?;
        for p in &prefixes.names {
            dict::write_str(&mut meta, p).map_err(io_err)?;
        }
        meta.write_all(&(datatypes.names.len() as u32).to_le_bytes()).map_err(io_err)?;
        for d in &datatypes.names {
            dict::write_str(&mut meta, d).map_err(io_err)?;
        }
        meta.write_all(&total.to_le_bytes()).map_err(io_err)?;
        meta.flush().map_err(io_err)?;
    }

    // dict-hash.bin + dict-hid.bin: externally sorted by (hash, id) — the canonical
    // order `save_mmap` also produces.
    {
        let mut hw = BufWriter::new(std::fs::File::create(dir.join("dict-hash.bin")).map_err(io_err)?);
        let mut iw = BufWriter::new(std::fs::File::create(dir.join("dict-hid.bin")).map_err(io_err)?);
        let mut merge = hash_sorter.into_merge().map_err(io_err)?;
        while let Some(p) = merge.next().map_err(io_err)? {
            hw.write_all(&p.hash.to_le_bytes()).map_err(io_err)?;
            iw.write_all(&p.id.to_le_bytes()).map_err(io_err)?;
        }
        hw.flush().map_err(io_err)?;
        iw.flush().map_err(io_err)?;
    }

    // Phase 4 — per shard: (seq -> min_seq) ⋈ (min_seq -> final id) -> dense
    // `seq -> final id` remap file (sorted by seq; every seq present exactly once).
    let mut shards_out: Vec<ShardRemap> = Vec::with_capacity(n);
    for (s, (_, seq_count, mut epochs)) in shard_files.into_iter().enumerate() {
        ensure_disk(tmp, cfg.disk_floor)?;
        let pairs_path = tmp.join(format!("dsp-pairs{s}.bin"));
        // `by_min`'s merge readers feed `by_seq`'s run buffer concurrently — split the
        // budget between the two sorters.
        let mut by_min: PodSorter<MinSeqPair> = PodSorter::new(sort_budget / 2, tmp, &format!("dsp-s{s}-bymin"));
        {
            let mut r = BufReader::new(std::fs::File::open(&pairs_path).map_err(io_err)?);
            let mut b = [0u8; 8];
            while read_full(&mut r, &mut b).map_err(io_err)? {
                by_min.push(MinSeqPair::read(&b)).map_err(io_err)?;
            }
        }
        std::fs::remove_file(&pairs_path).ok();

        let m2f_path = tmp.join(format!("dsp-m2f{s}.bin"));
        let mut by_seq: PodSorter<SeqFinal> = PodSorter::new(sort_budget / 2, tmp, &format!("dsp-s{s}-final"));
        {
            let mut m2f = BufReader::new(std::fs::File::open(&m2f_path).map_err(io_err)?);
            let mut merge = by_min.into_merge().map_err(io_err)?;
            let mut cur: Option<(u32, u32)> = None; // (min_seq, final id)
            let next_m2f = |m2f: &mut BufReader<std::fs::File>| -> io::Result<Option<(u32, u32)>> {
                let mut b = [0u8; 8];
                Ok(read_full(m2f, &mut b)?.then(|| {
                    (
                        u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
                        u32::from_le_bytes([b[4], b[5], b[6], b[7]]),
                    )
                }))
            };
            while let Some(p) = merge.next().map_err(io_err)? {
                while cur.map_or(true, |(m, _)| m < p.min_seq) {
                    cur = next_m2f(&mut m2f).map_err(io_err)?;
                    if cur.is_none() {
                        return Err("dict-spill: remap join exhausted the assignment stream".into());
                    }
                }
                let (m, f) = cur.unwrap();
                if m != p.min_seq {
                    return Err("dict-spill: remap join mismatch (missing canonical seq)".into());
                }
                by_seq.push(SeqFinal { seq: p.seq, final_id: f }).map_err(io_err)?;
            }
        }
        std::fs::remove_file(&m2f_path).ok();

        let remap_path = tmp.join(format!("dsp-remap{s}.bin"));
        {
            let mut w = BufWriter::new(std::fs::File::create(&remap_path).map_err(io_err)?);
            let mut merge = by_seq.into_merge().map_err(io_err)?;
            let mut expect: u32 = 0;
            while let Some(sf) = merge.next().map_err(io_err)? {
                if sf.seq != expect {
                    return Err("dict-spill: remap stream is not dense over seqs".into());
                }
                expect = expect.wrapping_add(1);
                w.write_all(&sf.final_id.to_le_bytes()).map_err(io_err)?;
            }
            if expect != seq_count {
                return Err("dict-spill: remap stream length mismatch".into());
            }
            w.flush().map_err(io_err)?;
        }
        epochs.push((u64::MAX, seq_count)); // sentinel: last epoch covers the tail
        shards_out.push(ShardRemap { remap_path, epochs });
    }

    Ok(RemapPlan { staged_path, staged_count, shards: shards_out })
}

/// Per-shard sliding window over a dense remap file, advanced at epoch boundaries —
/// resident memory is one epoch's seq range (bounded by the ingest cache budget).
struct ShardWindow {
    r: BufReader<std::fs::File>,
    epochs: Vec<(u64, u32)>,
    /// Index into `epochs` of the NEXT boundary to cross (current epoch is `next - 1`).
    next: usize,
    seq_base: u32,
    window: Vec<u32>,
}

impl ShardWindow {
    fn open(plan: &ShardRemap) -> io::Result<ShardWindow> {
        Ok(ShardWindow {
            r: BufReader::new(std::fs::File::open(&plan.remap_path)?),
            epochs: plan.epochs.clone(),
            next: 1,
            seq_base: 0,
            window: Vec::new(),
        })
    }

    /// Final id of `seq` for a triple staged at index `t` (loads epochs lazily; every
    /// reference points into the epoch active when its triple was staged, so windows
    /// only ever move FORWARD and reads stay sequential).
    fn map(&mut self, t: u64, seq: u32) -> io::Result<u32> {
        while self.next < self.epochs.len() - 1 && self.epochs[self.next].0 <= t {
            self.next += 1;
        }
        let (lo, hi) = (self.epochs[self.next - 1].1, self.epochs[self.next].1);
        if self.window.is_empty() || self.seq_base != lo {
            // Load this epoch's remap slice [lo, hi) — consecutive in the file.
            let len = (hi - lo) as usize;
            let mut bytes = vec![0u8; len * 4];
            self.r.read_exact(&mut bytes)?;
            self.window = bytes.chunks_exact(4).map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect();
            self.seq_base = lo;
        }
        let i = (seq - self.seq_base) as usize;
        self.window.get(i).copied().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "dict-spill: staged reference outside its epoch window")
        })
    }
}

/// Phase 5 — stream the staged triples through the per-shard remap windows, pushing
/// final-id `[u32;3]` triples into the EXISTING run-spill machinery (`buf`/`runs`).
/// Cleans up the staged + remap files.
pub(crate) fn remap_staged(
    plan: RemapPlan,
    buf: &mut Vec<[Id; 3]>,
    runs: &mut Vec<PathBuf>,
    tmp: &Path,
    chunk: usize,
) -> Result<(), String> {
    let io_err = |e: io::Error| e.to_string();
    let mut windows: Vec<ShardWindow> = plan.shards.iter().map(ShardWindow::open).collect::<io::Result<_>>().map_err(io_err)?;
    {
        let mut r = BufReader::with_capacity(1 << 20, std::fs::File::open(&plan.staged_path).map_err(io_err)?);
        let mut rec = [0u8; 24];
        for t in 0..plan.staged_count {
            r.read_exact(&mut rec).map_err(io_err)?;
            let mut out = [0 as Id; 3];
            for (k, c) in rec.chunks_exact(8).enumerate() {
                let v = u64::from_le_bytes([c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]]);
                out[k] = if v < STAGED_DICT_BASE {
                    v as Id // inline-integer id, passed through at ingest
                } else {
                    let s = (v / STAGED_DICT_BASE) as usize - 1;
                    windows[s].map(t, (v % STAGED_DICT_BASE) as u32).map_err(io_err)?
                };
            }
            buf.push(out);
            if buf.len() >= chunk {
                crate::extsort::spill_run(buf, runs, tmp).map_err(io_err)?;
            }
        }
    }
    std::fs::remove_file(&plan.staged_path).ok();
    for s in &plan.shards {
        std::fs::remove_file(&s.remap_path).ok();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn term_record_roundtrip() {
        let cases: Vec<TermParts> = vec![
            TermParts::Iri { prefix: "http://ex/", suffix: "a" },
            TermParts::Iri { prefix: "", suffix: "urn:x" },
            TermParts::Lit { value: "v", datatype: "http://www.w3.org/2001/XMLSchema#string", lang: None },
            TermParts::Lit { value: "café", datatype: "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString", lang: Some("fr") },
            TermParts::Lit { value: "", datatype: "http://www.w3.org/2001/XMLSchema#string", lang: None },
            TermParts::Blank("b0"),
        ];
        let mut buf = Vec::new();
        for tp in &cases {
            serialize_termparts(tp, &mut buf);
            let rt = parse_term(&buf);
            assert_eq!(dict::hash_termparts(tp), dict::hash_termparts(&rt));
            let mut buf2 = Vec::new();
            serialize_termparts(&rt, &mut buf2);
            assert_eq!(buf, buf2);
        }
    }

    #[test]
    fn pod_sorter_external_runs() {
        let tmp = std::env::temp_dir().join(format!("sparq_dsp_pod_{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        let mut s: PodSorter<SeqFinal> = PodSorter::new(1, &tmp, "t"); // tiny budget -> many runs
        let mut expect: Vec<u32> = Vec::new();
        let mut x = 12345u32;
        for _ in 0..20000 {
            x = x.wrapping_mul(1103515245).wrapping_add(12345);
            let v = x % 100000;
            expect.push(v);
            s.push(SeqFinal { seq: v, final_id: !v }).unwrap();
        }
        expect.sort_unstable();
        let mut m = s.into_merge().unwrap();
        let mut got = Vec::new();
        while let Some(sf) = m.next().unwrap() {
            assert_eq!(sf.final_id, !sf.seq);
            got.push(sf.seq);
        }
        assert_eq!(got, expect);
        drop(m);
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn var_sorter_groups_equal_terms() {
        let tmp = std::env::temp_dir().join(format!("sparq_dsp_var_{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        let mut s = VarSorter::new(VarKey::BytesThenSeq, 1, &tmp, "t");
        for seq in 0..1000u32 {
            let term = format!("term{}", seq % 37);
            s.push(seq, term.into_bytes().into_boxed_slice()).unwrap();
        }
        let mut m = s.into_merge().unwrap();
        let mut groups = 0;
        let mut last: Option<(Box<[u8]>, u32)> = None;
        while let Some((seq, bytes)) = m.next().unwrap() {
            match &last {
                Some((lb, ls)) if **lb == *bytes => assert!(seq > *ls, "within a group seqs ascend"),
                _ => groups += 1,
            }
            last = Some((bytes, seq));
        }
        assert_eq!(groups, 37);
        drop(m);
        std::fs::remove_dir_all(&tmp).ok();
    }
}
