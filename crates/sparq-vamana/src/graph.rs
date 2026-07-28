//! A **persistent, on-disk Vamana ANN index** (`.spqg`) over any [`VectorSource`]: a fixed-degree
//! proximity graph laid out for memory-mapped, locality-friendly reads, so a built index is
//! **saved once and reopened without a rebuild**.
//!
//! # Why this exists
//!
//! The mainstream pure-Rust ANN crates build an index in RAM and rebuild it on every process
//! start. `instant-distance`'s HNSW, for example, is a *closed* graph: its adjacency is not
//! exposed, so it cannot be laid out on disk. This index is therefore **self-contained** — the
//! Vamana graph is built here and its on-disk encoding is owned end to end.
//!
//! # Algorithm (Vamana, the DiskANN graph)
//!
//! Build (in RAM, once): start from a random `R`-regular graph, then for each node run a greedy
//! search from a fixed medoid to collect visited candidates and `RobustPrune` them to at most `R`
//! out-neighbours (the α-pruned edge set that gives DiskANN its short search paths); edges are
//! made undirected and re-pruned when a node exceeds degree `R`. Two passes (α = 1.0 then
//! α = `alpha`) per the DiskANN paper. Search: greedy beam search of width `L` from the medoid.
//! Distance is **cosine**: vectors are L2-normalized at build time and Euclidean-on-unit-vectors
//! is rank-equivalent to cosine (`cos = 1 − d²/2`), so reported scores and rankings match an exact
//! cosine scan.
//!
//! # On-disk format (`.spqg`, version 2, little-endian)
//!
//! ```text
//! offset 0    magic       b"SPQG"                       4 bytes
//! offset 4    version     u32 = 2                       4 bytes
//! offset 8    dim         u32                           4 bytes
//! offset 12   degree R    u32   (max out-neighbours)    4 bytes
//! offset 16   count       u64   (nodes = source vectors) 8 bytes
//! offset 24   medoid      u32   (entry-point slot)      4 bytes
//! offset 28   enc_tag     u32   (0 = none, 1 = PQ cache) 4 bytes
//! offset 32   token       opaque staleness token       24 bytes
//! offset 56   nodes       [count × NODE_RECORD]         count·record bytes
//!
//! one NODE_RECORD (all fields little-endian, 4-byte aligned):
//!   id        u32                  the caller's VectorId
//!   degree    u32                  number of valid neighbours (≤ R)
//!   vector    [dim] f32            the L2-NORMALIZED vector (search reads it here)
//!   nbrs      [R] u32              neighbour SLOT indices; entries ≥ degree are padding
//!
//! the optional PQ_SECTION (present iff enc_tag == 1, appended after nodes):
//!   magic     b"SPQP"              4 bytes
//!   cb_len    u64                  length of the codebook block that follows
//!   codebook  [cb_len] bytes       ProductQuantizer::to_bytes (dim, m, k + centroids)
//!   stride    u32                  code length M (== codebook M)
//!   codes     [count × M] bytes    per-slot PQ codes, slot order (== node-record order)
//! ```
//!
//! A node's vector and its adjacency are **co-located in one record** (the DiskANN locality
//! property): one contiguous read per visited node — the OS pages in a record and gets both the
//! distance input and the next hops, no scatter. Records start at multiples of 4 from the
//! page-aligned map, so the `f32` cast is always aligned.
//!
//! # Honest scope vs. full DiskANN
//!
//! The default build is the **Vamana on-disk graph with full-precision vectors searched from the
//! mmap**. Full DiskANN additionally keeps a **PQ-compressed** copy of every vector resident in
//! RAM to rank candidates without touching disk, reading full-precision vectors only to re-rank
//! the beam. That quantization layer ([`crate::quant`] — [`ProductQuantizer`] + [`EncodedStore`] +
//! [`DistanceTable`]) is wired into the search path: build with [`VamanaIndex::build_with_pq`] and
//! the greedy search ranks each visited node's neighbours by an ADC [`DistanceTable`] lookup
//! against the in-RAM codes (no disk), reading the full-precision vector from the mmap only for
//! the final beam it re-ranks — DiskANN's "search on PQ, re-rank on disk" loop. Without the PQ
//! section (the plain [`build`](VamanaIndex::build) path) the search is unchanged: it computes
//! every candidate distance from the full-precision mmap. At the scales where the graph fits in
//! page cache the two are recall-equivalent up to the PQ approximation; PQ matters most when the
//! vectors themselves exceed RAM. The header's 4 reserved bytes carry an **encoding tag**
//! (`0` = no cache, `1` = a trailing PQ section), so the PQ variant is a backwards-compatible
//! addition: a plain reader sees the tag and a longer file but the node-record layout is
//! byte-identical.
//!
//! # Staleness (the header token)
//!
//! Node records store the caller's [`VectorId`]s and neighbour entries are stored SLOTS, so an
//! index is valid ONLY against the exact vector generation it was built against. This crate cannot
//! know what "the same generation" means for a given consumer, so the header carries an **opaque
//! 24-byte [`StalenessToken`]** the consumer supplies at build time and compares itself on open
//! (see [`VamanaIndex::staleness_token`]). An index built without one reads back as `None`
//! ("unverifiable"), never as a zero token that would look like a mismatch. `sparq-vectors`
//! populates it with an RDF graph fingerprint; a stand-alone consumer can use a content hash, a
//! generation counter, or a random build id.
//!
//! Version-1 files (no token, 32-byte header) still open but report `None`.

use crate::backing::{open_backing, Bytes};
use crate::quant::{DistanceTable, EncodedStore, PqConfig, ProductQuantizer};
use crate::source::{VectorId, VectorSource};
use std::cmp::Ordering;
use std::io::Write;
use std::path::{Path, PathBuf};

/// First four bytes of every `.spqg` file.
pub const SPQG_MAGIC: [u8; 4] = *b"SPQG";
/// Current on-disk graph format version. v2 adds the 24-byte staleness-token block at offset 32;
/// v1 files (32-byte header) still open but cannot be verified.
pub const SPQG_VERSION: u32 = 2;
/// Byte length of the opaque staleness token in the `.spqg` header.
pub const STALENESS_TOKEN_LEN: usize = 24;
/// Header length of a legacy version-1 file (no token block). Public so a tool that rewrites or
/// inspects a `.spqg` header does not have to hard-code the offset.
pub const SPQG_HEADER_LEN_V1: usize = 32;
/// Header length of the current (version-2) format: the v1 header + the token block.
pub const SPQG_HEADER_LEN: usize = SPQG_HEADER_LEN_V1 + STALENESS_TOKEN_LEN;
const HEADER_LEN_V1: usize = SPQG_HEADER_LEN_V1;
const HEADER_LEN: usize = SPQG_HEADER_LEN;
/// Byte offset of the encoding tag (the 4 reserved bytes of the v1/v2 header).
const ENC_TAG_OFFSET: usize = 28;
/// Encoding tag: no PQ candidate cache — the file ends at the last node record (default).
const ENC_TAG_NONE: u32 = 0;
/// Encoding tag: a **PQ candidate-cache section** follows the node records (codebook + per-slot
/// codes). See `PqSection` for the trailing block's layout.
const ENC_TAG_PQ: u32 = 1;
/// First four bytes of the trailing PQ section (a guard against a truncated/garbled tail).
const PQ_SECTION_MAGIC: [u8; 4] = *b"SPQP";

/// An **opaque staleness token**: [`STALENESS_TOKEN_LEN`] consumer-defined bytes stored in the
/// `.spqg` header and handed back verbatim on open. This crate never interprets it — it only
/// distinguishes "absent" (an all-zero block, or a legacy v1 file) from "present".
///
/// The consumer decides what generation identity means and compares the token itself; see the
/// module docs. An all-zero token is indistinguishable from "no token", so a consumer whose
/// identity scheme can legitimately produce 24 zero bytes should offset it by one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StalenessToken([u8; STALENESS_TOKEN_LEN]);

impl StalenessToken {
    /// Wraps the consumer's [`STALENESS_TOKEN_LEN`] identity bytes.
    pub fn new(bytes: [u8; STALENESS_TOKEN_LEN]) -> StalenessToken {
        StalenessToken(bytes)
    }

    /// The raw token bytes, for the consumer's own comparison / decoding.
    pub fn as_bytes(&self) -> &[u8; STALENESS_TOKEN_LEN] {
        &self.0
    }

    /// Decodes a header token block, returning `None` for an **all-zero block** — an index written
    /// without a token. Decoding that to `Some([0; 24])` would surface as a spurious "different
    /// generation" mismatch instead of the accurate "unverifiable". `bytes` must be at least
    /// [`STALENESS_TOKEN_LEN`] long (the caller validated the header length first).
    fn from_header_bytes(bytes: &[u8]) -> Option<StalenessToken> {
        let block = &bytes[..STALENESS_TOKEN_LEN];
        if block.iter().all(|&b| b == 0) {
            None
        } else {
            let mut out = [0u8; STALENESS_TOKEN_LEN];
            out.copy_from_slice(block);
            Some(StalenessToken(out))
        }
    }
}

/// Vamana build/search parameters. Defaults follow the DiskANN paper's small-graph regime.
#[derive(Clone, Copy, Debug)]
pub struct VamanaConfig {
    /// Max out-degree `R` (neighbours per node). Higher = better recall, larger file.
    pub degree: usize,
    /// Build-time beam width `L` (candidate list size during construction; must be ≥ `degree`).
    pub build_beam: usize,
    /// Search-time beam width `L` (the recall knob; must be ≥ the `k` you query).
    pub search_beam: usize,
    /// RobustPrune relaxation `α` ≥ 1.0 (1.0 = pure nearest; >1 keeps longer-range edges
    /// that shorten search paths). DiskANN uses 1.2.
    pub alpha: f32,
    /// Graph-construction RNG seed — fixed by default so builds are reproducible.
    pub seed: u64,
}

impl Default for VamanaConfig {
    fn default() -> Self {
        VamanaConfig {
            degree: 32,
            build_beam: 100,
            search_beam: 100,
            alpha: 1.2,
            seed: 0x5350_5147_0001,
        }
    }
}

/// Squared Euclidean distance over two equal-length unit vectors (rank-equivalent to cosine;
/// avoids the `sqrt` that the greedy search does not need). Eight accumulator lanes so the
/// loop auto-vectorizes.
fn sq_dist(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());
    const LANES: usize = 8;
    let mut acc = [0f32; LANES];
    let chunks = a.len() / LANES;
    for c in 0..chunks {
        for l in 0..LANES {
            let d = a[c * LANES + l] - b[c * LANES + l];
            acc[l] += d * d;
        }
    }
    for i in chunks * LANES..a.len() {
        let d = a[i] - b[i];
        acc[0] += d * d;
    }
    acc.iter().sum()
}

/// L2-normalizes `v`; `None` for an all-zero vector (no direction). Source vectors are never zero
/// (see [`VectorSource`]'s contract), so `None` only arises for a degenerate query.
fn normalized(v: &[f32]) -> Option<Vec<f32>> {
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    (norm > 0.0).then(|| v.iter().map(|x| x / norm).collect())
}

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// A `(distance, slot)` ordered nearest-first. `f32` distances are finite here (unit vectors),
/// so `total_cmp` never sees a NaN; ties break on slot for determinism.
#[derive(Clone, Copy, PartialEq)]
struct Cand {
    dist: f32,
    slot: u32,
}
impl Eq for Cand {}
impl Ord for Cand {
    fn cmp(&self, other: &Self) -> Ordering {
        self.dist.total_cmp(&other.dist).then(self.slot.cmp(&other.slot))
    }
}
impl PartialOrd for Cand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// ───────────────────────────── build (in RAM) ─────────────────────────────

/// The in-RAM Vamana graph during construction: normalized vectors + per-node adjacency.
struct Builder {
    dim: usize,
    degree: usize,
    /// Normalized vectors, row-major, slot order (matches the source's `iter` order).
    vectors: Vec<f32>,
    ids: Vec<VectorId>,
    /// Out-neighbour slots per node (≤ `degree` after pruning).
    adj: Vec<Vec<u32>>,
    medoid: u32,
}

impl Builder {
    fn vector(&self, slot: u32) -> &[f32] {
        let s = slot as usize * self.dim;
        &self.vectors[s..s + self.dim]
    }

    fn dist(&self, a: u32, b: u32) -> f32 {
        sq_dist(self.vector(a), self.vector(b))
    }

    /// Greedy search from `start` toward `query`'s slot, returning the set of slots *visited*
    /// (the candidate pool RobustPrune draws from). The working set is kept to `beam` closest.
    fn greedy_visit(&self, query: u32, start: u32, beam: usize) -> Vec<u32> {
        let n = self.ids.len();
        let qv = self.vector(query);
        let mut working: Vec<Cand> = Vec::with_capacity(beam + self.degree);
        let mut visited: Vec<u32> = Vec::new();
        let mut in_working = vec![false; n];
        let mut expanded = vec![false; n];
        working.push(Cand { dist: sq_dist(qv, self.vector(start)), slot: start });
        in_working[start as usize] = true;
        loop {
            let next = working
                .iter()
                .filter(|c| !expanded[c.slot as usize])
                .min()
                .copied();
            let Some(cur) = next else { break };
            expanded[cur.slot as usize] = true;
            visited.push(cur.slot);
            for &nbr in &self.adj[cur.slot as usize] {
                if in_working[nbr as usize] {
                    continue;
                }
                in_working[nbr as usize] = true;
                let d = sq_dist(qv, self.vector(nbr));
                working.push(Cand { dist: d, slot: nbr });
            }
            if working.len() > beam {
                working.sort_unstable();
                working.truncate(beam);
            }
        }
        visited
    }

    /// DiskANN's RobustPrune: from candidate slots `cands` (plus `node`'s current neighbours),
    /// pick ≤ `degree` out-neighbours for `node`, keeping `p` only if no already-kept `k` is
    /// `alpha`× closer to `p` than `node` is — the diversification that bounds search paths.
    fn robust_prune(&self, node: u32, mut cands: Vec<u32>, alpha: f32) -> Vec<u32> {
        cands.extend_from_slice(&self.adj[node as usize]);
        cands.retain(|&c| c != node);
        cands.sort_unstable();
        cands.dedup();
        // Closest candidate to `node` first.
        cands.sort_by(|&a, &b| self.dist(node, a).total_cmp(&self.dist(node, b)));
        let mut kept: Vec<u32> = Vec::with_capacity(self.degree);
        for p in cands {
            if kept.len() >= self.degree {
                break;
            }
            // Keep p unless some already-kept k is alpha× closer to p than node is.
            let keep = kept.iter().all(|&k| alpha * self.dist(k, p) > self.dist(node, p));
            if keep {
                kept.push(p);
            }
        }
        kept
    }

    /// Add the directed edge node→nbr, re-pruning node if it overflows `degree`.
    fn add_edge(&mut self, node: u32, nbr: u32, alpha: f32) {
        if node == nbr || self.adj[node as usize].contains(&nbr) {
            return;
        }
        self.adj[node as usize].push(nbr);
        if self.adj[node as usize].len() > self.degree {
            let cands = std::mem::take(&mut self.adj[node as usize]);
            self.adj[node as usize] = self.robust_prune(node, cands, alpha);
        }
    }
}

/// Builds the Vamana graph over every vector in `source`, returning the in-RAM [`Builder`].
fn build_graph<S: VectorSource + ?Sized>(source: &S, cfg: &VamanaConfig) -> Builder {
    let dim = source.dim();
    let mut vectors: Vec<f32> = Vec::with_capacity(source.len() * dim);
    let mut ids: Vec<VectorId> = Vec::with_capacity(source.len());
    for (id, v) in source.iter() {
        let nv = normalized(v).expect("a VectorSource never yields zero vectors (see its contract)");
        vectors.extend_from_slice(&nv);
        ids.push(id);
    }
    let n = ids.len();
    let degree = cfg.degree.max(1);

    // Medoid (greedy entry point): the node closest to the centroid — O(n·dim), an adequate
    // start for greedy search (the exact O(n²) medoid is not worth it here).
    let medoid = if n == 0 {
        0
    } else {
        let mut centroid = vec![0f32; dim];
        for s in 0..n {
            for (c, &x) in centroid.iter_mut().zip(&vectors[s * dim..(s + 1) * dim]) {
                *c += x;
            }
        }
        for c in centroid.iter_mut() {
            *c /= n as f32;
        }
        (0..n)
            .min_by(|&a, &b| {
                sq_dist(&vectors[a * dim..(a + 1) * dim], &centroid)
                    .total_cmp(&sq_dist(&vectors[b * dim..(b + 1) * dim], &centroid))
            })
            .unwrap_or(0) as u32
    };

    // Random initial graph so greedy search has edges to follow on pass one.
    let mut adj: Vec<Vec<u32>> = vec![Vec::new(); n];
    if n > 1 {
        let mut state = cfg.seed;
        let want = degree.min(n - 1);
        for (s, out) in adj.iter_mut().enumerate() {
            while out.len() < want {
                let cand = (splitmix64(&mut state) % n as u64) as u32;
                if cand as usize != s && !out.contains(&cand) {
                    out.push(cand);
                }
            }
        }
    }

    let mut b = Builder { dim, degree, vectors, ids, adj, medoid };

    if n > 1 {
        // Random processing order (DiskANN). Two passes: α=1.0 then α=cfg.alpha.
        let mut order: Vec<u32> = (0..n as u32).collect();
        let mut state = cfg.seed ^ 0xABCD_1234;
        for i in (1..order.len()).rev() {
            let j = (splitmix64(&mut state) % (i as u64 + 1)) as usize;
            order.swap(i, j);
        }
        for &alpha in &[1.0f32, cfg.alpha.max(1.0)] {
            for &node in &order {
                let visited = b.greedy_visit(node, b.medoid, cfg.build_beam.max(degree));
                let pruned = b.robust_prune(node, visited, alpha);
                b.adj[node as usize] = pruned.clone();
                // Make edges undirected (Vamana): add node into each neighbour's list.
                for &nbr in &pruned {
                    b.add_edge(nbr, node, alpha);
                }
            }
        }
    }
    b
}

// ───────────────────────────── persistence ─────────────────────────────

/// Byte length of one node record: id + degree + vector + neighbour slots.
const fn record_len(dim: usize, degree: usize) -> usize {
    8 + dim * 4 + degree * 4
}

/// The trailing PQ candidate-cache section: the fitted quantizer plus every vector's PQ code,
/// ready to be appended after the node records when the encoding tag is `1`.
struct PqSection {
    pq: ProductQuantizer,
    codes: EncodedStore,
}

impl PqSection {
    /// Serializes the section to the on-disk byte layout documented in the module header
    /// (`SPQP` magic, codebook length + codebook, stride, then the flat per-slot codes).
    fn to_bytes(&self) -> Vec<u8> {
        let cb = self.pq.to_bytes();
        let codes = self.codes.codes();
        let mut out = Vec::with_capacity(4 + 8 + cb.len() + 4 + codes.len());
        out.extend_from_slice(&PQ_SECTION_MAGIC);
        out.extend_from_slice(&(cb.len() as u64).to_le_bytes());
        out.extend_from_slice(&cb);
        out.extend_from_slice(&(self.codes.stride() as u32).to_le_bytes());
        out.extend_from_slice(codes);
        out
    }
}

/// Writes the built graph to `path` as a `.spqg` file (one node record per node). `token`
/// (offset 32..56) binds the index to its generation; `None` writes an unverifiable (all-zero)
/// block. `pq` (when `Some`) appends a `PqSection` after the node records and sets the encoding
/// tag to [`ENC_TAG_PQ`]; its `codes` MUST be in the same slot order as the node records.
fn write_graph(
    b: &Builder,
    path: &Path,
    token: Option<StalenessToken>,
    pq: Option<&PqSection>,
) -> Result<(), String> {
    if cfg!(target_endian = "big") {
        return Err(".spqg is a little-endian format; big-endian targets are unsupported".into());
    }
    let count = b.ids.len();
    let r = b.degree;
    let mut header = [0u8; HEADER_LEN];
    header[0..4].copy_from_slice(&SPQG_MAGIC);
    header[4..8].copy_from_slice(&SPQG_VERSION.to_le_bytes());
    header[8..12].copy_from_slice(&(b.dim as u32).to_le_bytes());
    header[12..16].copy_from_slice(&(r as u32).to_le_bytes());
    header[16..24].copy_from_slice(&(count as u64).to_le_bytes());
    header[24..28].copy_from_slice(&b.medoid.to_le_bytes());
    // Encoding tag at offset 28..32: `1` when a PQ section is appended below, else `0`. Staleness
    // token at offset 32..56; `None` leaves a zeroed block, which reads back as "unverifiable".
    let enc_tag = if pq.is_some() { ENC_TAG_PQ } else { ENC_TAG_NONE };
    header[ENC_TAG_OFFSET..ENC_TAG_OFFSET + 4].copy_from_slice(&enc_tag.to_le_bytes());
    if let Some(t) = token {
        header[HEADER_LEN_V1..HEADER_LEN].copy_from_slice(t.as_bytes());
    }

    let file = std::fs::File::create(path).map_err(|e| format!("create {}: {e}", path.display()))?;
    let mut w = std::io::BufWriter::new(file);
    w.write_all(&header).map_err(|e| format!("write {}: {e}", path.display()))?;

    let mut rec = vec![0u8; record_len(b.dim, r)];
    for slot in 0..count {
        let nbrs = &b.adj[slot];
        let deg = nbrs.len().min(r) as u32;
        rec[0..4].copy_from_slice(&b.ids[slot].to_le_bytes());
        rec[4..8].copy_from_slice(&deg.to_le_bytes());
        let vec_off = 8;
        let v = &b.vectors[slot * b.dim..(slot + 1) * b.dim];
        // f32 → LE bytes (little-endian target asserted above). SAFETY: f32 has no invalid bit
        // patterns and align(f32) ≥ align(u8); the read borrows `b.vectors`.
        let v_bytes = unsafe { std::slice::from_raw_parts(v.as_ptr() as *const u8, b.dim * 4) };
        rec[vec_off..vec_off + b.dim * 4].copy_from_slice(v_bytes);
        let nbr_off = vec_off + b.dim * 4;
        for (i, entry) in rec[nbr_off..].chunks_exact_mut(4).enumerate() {
            let val = if i < deg as usize { nbrs[i] } else { 0u32 };
            entry.copy_from_slice(&val.to_le_bytes());
        }
        w.write_all(&rec).map_err(|e| format!("write {}: {e}", path.display()))?;
    }
    // Trailing PQ candidate-cache section (encoding tag == 1).
    if let Some(section) = pq {
        w.write_all(&section.to_bytes())
            .map_err(|e| format!("write {}: {e}", path.display()))?;
    }
    let f = w.into_inner().map_err(|e| format!("flush {}: {e}", path.display()))?;
    f.sync_all().map_err(|e| format!("fsync {}: {e}", path.display()))?;
    Ok(())
}

// ───────────────────────────── on-disk search ─────────────────────────────

/// A **persistent on-disk Vamana index** opened over a `.spqg` file (memory-mapped on native
/// targets). Built once with [`build`](Self::build) / [`build_with_pq`](Self::build_with_pq),
/// reopened with [`open`](Self::open) at near-zero cost (mmap + header validation, no rebuild) or
/// from fetched/embedded bytes with [`open_from_bytes`](Self::open_from_bytes) (the
/// wasm/filesystem-less path — memmap2 is target-gated out of wasm32 builds). Search reads node
/// records directly from the backing; see the module docs for the format and the honest scope vs.
/// full DiskANN.
pub struct VamanaIndex {
    map: Bytes,
    dim: usize,
    degree: usize,
    count: usize,
    medoid: u32,
    record_len: usize,
    search_beam: usize,
    /// The staleness token this index was built with, or `None` for a legacy version-1 file / an
    /// index built without one. See [`staleness_token`](Self::staleness_token).
    token: Option<StalenessToken>,
    /// Byte offset where node records begin: [`HEADER_LEN`] (v2) or [`HEADER_LEN_V1`] (legacy v1).
    /// Every record read keys off this so both versions are read correctly by the same code.
    data_offset: usize,
    /// The in-RAM PQ candidate cache (codebook + per-slot codes), or `None` for a plain index
    /// (encoding tag `0`). When present, [`search_slots`](Self::search_slots) ranks each visited
    /// node's neighbours by an ADC table lookup against these codes and re-ranks the final beam
    /// off the mmap; when `None` it computes every distance from the mmap as before.
    pq: Option<PqCache>,
}

// Hand-written (the read backing is not `Debug`) — prints the header facts a caller actually
// wants when an index turns up in an assertion or a log, never the mapped body.
impl std::fmt::Debug for VamanaIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VamanaIndex")
            .field("dim", &self.dim)
            .field("degree", &self.degree)
            .field("count", &self.count)
            .field("medoid", &self.medoid)
            .field("search_beam", &self.search_beam)
            .field("has_pq_cache", &self.pq.is_some())
            .field("staleness_token", &self.token)
            .finish()
    }
}

/// The decoded PQ candidate cache: the fitted quantizer and the per-slot codes (slot order ==
/// node-record order, so a graph slot indexes the codes directly).
struct PqCache {
    pq: ProductQuantizer,
    codes: EncodedStore,
}

impl VamanaIndex {
    /// Builds the Vamana graph over `source` with `cfg`, writes the `.spqg` file at `path`, and
    /// opens it memory-mapped. The build is in RAM (one-off); the open is cheap forever after.
    ///
    /// `token` binds the index to the caller's vector generation (see the module docs); pass
    /// `None` for an unverifiable index.
    pub fn build<S: VectorSource + ?Sized, P: AsRef<Path>>(
        source: &S,
        path: P,
        cfg: VamanaConfig,
        token: Option<StalenessToken>,
    ) -> Result<VamanaIndex, String> {
        Self::build_inner(source, path, cfg, token, None)
    }

    /// Builds the Vamana graph **with a PQ candidate cache**: in addition to the full-precision
    /// node records, it fits a [`ProductQuantizer`] over `source` (with `pq_cfg`), encodes every
    /// vector into the in-RAM code cache, and persists both alongside the graph (the trailing PQ
    /// section, encoding tag `1`). The opened index then searches DiskANN-style: rank candidates
    /// on the RAM codes (no disk), re-rank the final beam off the mmap. Recall is approximate (the
    /// PQ approximation) but no disk page is touched until the re-rank, so it scales to sources
    /// whose full-precision vectors exceed RAM.
    ///
    /// Errors if `pq_cfg` is invalid for `source`'s dimension (see [`ProductQuantizer::fit`]) or
    /// the source is empty (PQ needs at least one training vector — use the plain
    /// [`build`](Self::build) for an empty source).
    pub fn build_with_pq<S: VectorSource + ?Sized, P: AsRef<Path>>(
        source: &S,
        path: P,
        cfg: VamanaConfig,
        pq_cfg: PqConfig,
        token: Option<StalenessToken>,
    ) -> Result<VamanaIndex, String> {
        let pq = ProductQuantizer::fit(source.dim(), source.iter().map(|(_, v)| v), pq_cfg)?;
        let codes = pq.encode_store(source)?;
        let section = PqSection { pq, codes };
        Self::build_inner(source, path, cfg, token, Some(section))
    }

    fn build_inner<S: VectorSource + ?Sized, P: AsRef<Path>>(
        source: &S,
        path: P,
        cfg: VamanaConfig,
        token: Option<StalenessToken>,
        pq: Option<PqSection>,
    ) -> Result<VamanaIndex, String> {
        if cfg.build_beam < cfg.degree {
            return Err(format!(
                "build_beam {} must be ≥ degree {} (the candidate pool can't be smaller than the out-degree)",
                cfg.build_beam, cfg.degree
            ));
        }
        let b = build_graph(source, &cfg);
        write_graph(&b, path.as_ref(), token, pq.as_ref())?;
        let mut idx = Self::open(path)?;
        idx.search_beam = cfg.search_beam;
        Ok(idx)
    }

    /// Opens a `.spqg` file memory-mapped, **without rebuilding** — the whole point of this
    /// index. Validates the header, the medoid, and that the file size matches `count` records
    /// (plus the PQ section when the encoding tag declares one); the records themselves page in on
    /// access, so their *contents* are validated lazily instead — a record's neighbour entries are
    /// bounds-checked against `count` (in `node_neighbours`) at the moment a search reads them.
    /// Between the two, no search can read out of bounds, whatever the file contains.
    ///
    /// A version-2 file's staleness token is read for [`staleness_token`](Self::staleness_token).
    /// A legacy version-1 file (32-byte header, no token) still opens — its token is `None`.
    ///
    /// On wasm32 (memmap2 target-gated out) this reads the whole file into the same f32-aligned
    /// owned backing [`open_from_bytes`](Self::open_from_bytes) uses — identical validation, no
    /// map. `wasm32-unknown-unknown` has no filesystem, so there the read fails with a clean I/O
    /// error and `open_from_bytes` is the supported path.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<VamanaIndex, String> {
        let path = path.as_ref();
        let map = open_backing(path)?;
        Self::open_validated(map, &path.display().to_string())
    }

    /// Opens a `.spqg` document held entirely in memory — for environments without a filesystem
    /// (the bytes were fetched, embedded, or decompressed by the caller). Validation is identical
    /// to [`open`](Self::open); record reads borrow the owned buffer (f32-aligned, so the vector
    /// casts are as sound as on the page-aligned map) instead of a memory map.
    pub fn open_from_bytes(bytes: Vec<u8>) -> Result<VamanaIndex, String> {
        Self::open_validated(Bytes::owned(bytes), "<bytes>")
    }

    /// Shared header/size validation behind [`open`](Self::open) and
    /// [`open_from_bytes`](Self::open_from_bytes).
    fn open_validated(map: Bytes, origin: &str) -> Result<VamanaIndex, String> {
        if cfg!(target_endian = "big") {
            return Err(".spqg is a little-endian format; big-endian targets are unsupported".into());
        }
        if map.len() < HEADER_LEN_V1 {
            return Err(format!("{origin}: truncated header"));
        }
        if map[0..4] != SPQG_MAGIC {
            return Err(format!("{origin}: not a .spqg file (bad magic)"));
        }
        let version = u32::from_le_bytes(map[4..8].try_into().unwrap());
        // Both v1 (no token, 32-byte header) and v2 (token, 56-byte header) open; the offset where
        // node records begin depends on the version, so every record read keys off `data_offset`.
        let (data_offset, token): (usize, Option<StalenessToken>) = match version {
            1 => (HEADER_LEN_V1, None),
            2 => {
                if map.len() < HEADER_LEN {
                    return Err(format!("{origin}: truncated version-2 header (token block)"));
                }
                // An all-zero block (a v2 index built without a token) → `None` ("unverifiable"),
                // not a zero token that would surface as a spurious mismatch.
                (
                    HEADER_LEN,
                    StalenessToken::from_header_bytes(&map[HEADER_LEN_V1..HEADER_LEN]),
                )
            }
            v => return Err(format!("{origin}: unsupported .spqg version {v}")),
        };
        let dim = u32::from_le_bytes(map[8..12].try_into().unwrap()) as usize;
        let degree = u32::from_le_bytes(map[12..16].try_into().unwrap()) as usize;
        let count: usize = u64::from_le_bytes(map[16..24].try_into().unwrap())
            .try_into()
            .map_err(|_| format!("{origin}: node count exceeds the address space"))?;
        let medoid = u32::from_le_bytes(map[24..28].try_into().unwrap());
        if dim == 0 {
            return Err(format!("{origin}: zero dimension"));
        }
        let record_len = record_len(dim, degree);
        // Checked size arithmetic — reject a malformed header before it wraps past the bounds check.
        // `nodes_end` is the byte just past the last node record; the file is exactly that long for
        // a plain index, or longer (a trailing PQ section) when the encoding tag is `1`.
        let nodes_end = count
            .checked_mul(record_len)
            .and_then(|body| body.checked_add(data_offset))
            .ok_or_else(|| {
                format!("{origin}: dim={dim} degree={degree} count={count} overflows the file size")
            })?;
        // Encoding tag at offset 28..32 (reserved bytes of the header).
        let enc_tag =
            u32::from_le_bytes(map[ENC_TAG_OFFSET..ENC_TAG_OFFSET + 4].try_into().unwrap());
        let pq = match enc_tag {
            ENC_TAG_NONE => {
                if map.len() != nodes_end {
                    return Err(format!(
                        "{origin}: file is {} bytes, expected {nodes_end} for dim={dim} degree={degree} count={count}",
                        map.len()
                    ));
                }
                None
            }
            ENC_TAG_PQ => {
                if map.len() < nodes_end {
                    return Err(format!(
                        "{origin}: file is {} bytes, shorter than the {nodes_end}-byte node body",
                        map.len()
                    ));
                }
                Some(Self::parse_pq_section(&map[nodes_end..], count, dim, origin)?)
            }
            t => return Err(format!("{origin}: unsupported encoding tag {t}")),
        };
        if count > 0 && medoid as usize >= count {
            return Err(format!("{origin}: medoid {medoid} out of range (count {count})"));
        }
        Ok(VamanaIndex {
            map,
            dim,
            degree,
            count,
            medoid,
            record_len,
            search_beam: VamanaConfig::default().search_beam,
            token,
            data_offset,
            pq,
        })
    }

    /// Parses the trailing PQ section (`tail` begins right after the last node record). Validates
    /// the magic, the codebook (which re-checks its own `dim`/`m`/`k`), that the codebook's `dim`
    /// matches the graph's, that exactly `count × M` code bytes are present (no trailing slop) so a
    /// later code read is always in bounds, and that every code byte names a centroid the codebook
    /// actually has (`< K`). Errors are descriptive.
    fn parse_pq_section(
        tail: &[u8],
        count: usize,
        dim: usize,
        origin: &str,
    ) -> Result<PqCache, String> {
        let hdr = 4 + 8; // magic + codebook length
        if tail.len() < hdr {
            return Err(format!("{origin}: truncated PQ section header"));
        }
        if tail[0..4] != PQ_SECTION_MAGIC {
            return Err(format!("{origin}: PQ section has bad magic"));
        }
        let cb_len: usize = u64::from_le_bytes(tail[4..12].try_into().unwrap())
            .try_into()
            .map_err(|_| format!("{origin}: PQ codebook length exceeds the address space"))?;
        let cb_end = hdr
            .checked_add(cb_len)
            .and_then(|e| e.checked_add(4)) // + stride u32
            .ok_or_else(|| format!("{origin}: PQ section length overflows"))?;
        if tail.len() < cb_end {
            return Err(format!("{origin}: truncated PQ codebook"));
        }
        let pq = ProductQuantizer::from_bytes(&tail[hdr..hdr + cb_len])
            .map_err(|e| format!("{origin}: {e}"))?;
        if pq.dim() != dim {
            return Err(format!("{origin}: PQ codebook dim {} != graph dim {dim}", pq.dim()));
        }
        let stride = u32::from_le_bytes(tail[hdr + cb_len..cb_end].try_into().unwrap()) as usize;
        if stride != pq.m() {
            return Err(format!("{origin}: PQ section stride {stride} != codebook M {}", pq.m()));
        }
        let want_codes = count
            .checked_mul(stride)
            .ok_or_else(|| format!("{origin}: PQ code array length overflows"))?;
        if tail.len() != cb_end + want_codes {
            return Err(format!(
                "{origin}: PQ code array is {} bytes, expected {want_codes} (count {count} × M {stride})",
                tail.len() - cb_end
            ));
        }
        // Every code byte is a centroid index into its subspace's row of the search-time ADC table
        // (`tables[s · K + c]`, `M × K` entries). `K` comes off the wire and may be well under 256
        // while a persisted code stays an arbitrary `u8`, so a forged byte `c ≥ K` would silently
        // read a neighbouring subspace's row — or, past the last row, index out of bounds and panic
        // mid-`nearest`. Reject it here so a malformed file fails at open, like every other check.
        let k = pq.k();
        if k < 256 {
            if let Some((i, &c)) = tail[cb_end..].iter().enumerate().find(|(_, &c)| c as usize >= k)
            {
                return Err(format!(
                    "{origin}: PQ code {} at slot {} subspace {} is out of range (K {})",
                    c,
                    i / stride,
                    i % stride,
                    k
                ));
            }
        }
        let codes =
            EncodedStore::from_parts((0..count as u32).collect(), tail[cb_end..].to_vec(), stride)
                .map_err(|e| format!("{origin}: {e}"))?;
        Ok(PqCache { pq, codes })
    }

    /// Whether this index carries an in-RAM PQ candidate cache (built via
    /// [`build_with_pq`](Self::build_with_pq) / persisted as the trailing PQ section). When `true`,
    /// [`nearest`](Self::nearest) ranks candidates on the codes and re-ranks the beam off the mmap;
    /// when `false` it searches full-precision throughout.
    pub fn has_pq_cache(&self) -> bool {
        self.pq.is_some()
    }

    /// The index's vector dimension.
    pub fn dim(&self) -> usize {
        self.dim
    }
    /// Number of indexed nodes (= source vectors at build time).
    pub fn len(&self) -> usize {
        self.count
    }
    /// Whether the index holds no nodes.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
    /// The search-time beam width this index searches with (from the build config, or
    /// [`VamanaConfig::default`]'s when the index was reopened).
    pub fn search_beam(&self) -> usize {
        self.search_beam
    }

    /// The opaque staleness token this index was built with, or `None` for a legacy version-1 file
    /// / an index built without one. The consumer compares it against its own current generation
    /// token; this crate never interprets it. See the module docs.
    pub fn staleness_token(&self) -> Option<StalenessToken> {
        self.token
    }

    /// The normalized vector stored in `slot`'s record, read directly from the map.
    fn node_vector(&self, slot: u32) -> &[f32] {
        let start = self.data_offset + slot as usize * self.record_len + 8;
        let bytes = &self.map[start..start + self.dim * 4];
        debug_assert_eq!(bytes.as_ptr() as usize % std::mem::align_of::<f32>(), 0);
        // SAFETY: the backing base is f32-aligned — a memory map is page-aligned, and the owned
        // backing (`open_from_bytes` / wasm32) is 4-byte-aligned by construction (`AlignedBytes`,
        // see backing.rs) — and `start` is a multiple of 4 (data_offset [32 or 56] +
        // slot·record_len[a multiple of 4] + 8), so the pointer is f32-aligned; the range is in
        // bounds (validated in `open_validated`); f32 accepts any bit pattern; slice borrows map.
        unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const f32, self.dim) }
    }

    /// `slot`'s caller-supplied [`VectorId`] (record field 0).
    fn node_id(&self, slot: u32) -> VectorId {
        let start = self.data_offset + slot as usize * self.record_len;
        u32::from_le_bytes(self.map[start..start + 4].try_into().unwrap())
    }

    /// `slot`'s valid out-neighbour slots (the first `degree` neighbour entries), **bounds-checked
    /// against `count`**.
    ///
    /// The neighbour entries are the one record field [`open`](Self::open) cannot check up front
    /// without reading every record — which would page the entire file in and defeat the lazy open
    /// this format exists for — so they are checked HERE, on the single path every searcher reads
    /// adjacency through: a stored degree above the header's `R` is clamped, and an entry naming a
    /// slot that does not exist (`>= count`) is dropped. A corrupt or hostile `.spqg` therefore
    /// loses that edge (a worse-connected graph, possibly worse recall) instead of indexing
    /// `in_working` / `node_vector` out of bounds and panicking mid-search.
    fn node_neighbours(&self, slot: u32) -> impl Iterator<Item = u32> + '_ {
        let start = self.data_offset + slot as usize * self.record_len;
        let deg = u32::from_le_bytes(self.map[start + 4..start + 8].try_into().unwrap()) as usize;
        let nbr_off = start + 8 + self.dim * 4;
        let count = self.count;
        (0..deg.min(self.degree))
            .map(move |i| {
                let o = nbr_off + i * 4;
                u32::from_le_bytes(self.map[o..o + 4].try_into().unwrap())
            })
            .filter(move |&nbr| (nbr as usize) < count)
    }

    /// Greedy beam search of width `beam` from the medoid toward `query` (already normalized),
    /// returning the `k` best `(slot, cosine)` pairs, best first. Each visited node is one
    /// contiguous record read from the map.
    ///
    /// When the index carries a PQ candidate cache (built via
    /// [`build_with_pq`](Self::build_with_pq)) the beam is driven by RAM-only ADC distance
    /// estimates against the codes ([`search_slots_pq`](Self::search_slots_pq)) and only the final
    /// beam is re-ranked off the mmap; otherwise every candidate distance is the exact mmap
    /// distance, as before.
    fn search_slots(&self, query: &[f32], k: usize, beam: usize) -> Vec<(u32, f32)> {
        if self.count == 0 {
            return Vec::new();
        }
        if let Some(cache) = &self.pq {
            return self.search_slots_pq(cache, query, k, beam);
        }
        let beam = beam.max(k).max(1);
        let mut working: Vec<Cand> = Vec::with_capacity(beam + self.degree);
        let mut in_working = vec![false; self.count];
        let mut expanded = vec![false; self.count];
        let start = self.medoid;
        working.push(Cand { dist: sq_dist(query, self.node_vector(start)), slot: start });
        in_working[start as usize] = true;
        loop {
            let next = working.iter().filter(|c| !expanded[c.slot as usize]).min().copied();
            let Some(cur) = next else { break };
            expanded[cur.slot as usize] = true;
            let neighbours: Vec<u32> = self.node_neighbours(cur.slot).collect();
            for nbr in neighbours {
                if in_working[nbr as usize] {
                    continue;
                }
                in_working[nbr as usize] = true;
                let d = sq_dist(query, self.node_vector(nbr));
                working.push(Cand { dist: d, slot: nbr });
            }
            if working.len() > beam {
                working.sort_unstable();
                working.truncate(beam);
            }
        }
        working.sort_unstable();
        working.truncate(k);
        // d² over unit vectors → cosine: cos = 1 − d²/2.
        working.into_iter().map(|c| (c.slot, 1.0 - c.dist / 2.0)).collect()
    }

    /// DiskANN's "search on PQ, re-rank on disk" greedy beam search. The frontier `working` holds
    /// **PQ-estimated** `d²` (an ADC [`DistanceTable`] lookup against the in-RAM codes — no disk
    /// touched while traversing), so the beam is ranked from RAM. After the walk, the surviving
    /// beam is **re-ranked against the full-precision mmap vectors** and the top `k` of *those*
    /// exact distances are returned, so the reported cosine is exact even though the path was
    /// guided by the lossy codes. `query` is already L2-normalized (the cosine convention).
    fn search_slots_pq(
        &self,
        cache: &PqCache,
        query: &[f32],
        k: usize,
        beam: usize,
    ) -> Vec<(u32, f32)> {
        let beam = beam.max(k).max(1);
        let table = DistanceTable::new(&cache.pq, query);
        // PQ-estimated d² for a slot's code (slot order == code order).
        let pq_dist = |slot: u32| table.distance(cache.codes.code(slot as usize));
        let mut working: Vec<Cand> = Vec::with_capacity(beam + self.degree);
        let mut in_working = vec![false; self.count];
        let mut expanded = vec![false; self.count];
        let start = self.medoid;
        working.push(Cand { dist: pq_dist(start), slot: start });
        in_working[start as usize] = true;
        loop {
            let next = working.iter().filter(|c| !expanded[c.slot as usize]).min().copied();
            let Some(cur) = next else { break };
            expanded[cur.slot as usize] = true;
            let neighbours: Vec<u32> = self.node_neighbours(cur.slot).collect();
            for nbr in neighbours {
                if in_working[nbr as usize] {
                    continue;
                }
                in_working[nbr as usize] = true;
                working.push(Cand { dist: pq_dist(nbr), slot: nbr });
            }
            if working.len() > beam {
                working.sort_unstable();
                working.truncate(beam);
            }
        }
        // Re-rank the surviving beam against full-precision mmap vectors (the only disk reads), then
        // keep the top `k` by EXACT distance — so the returned cosine matches the full-precision
        // searchers even though the traversal was PQ-guided.
        working.sort_unstable();
        working.truncate(beam);
        let mut reranked: Vec<Cand> = working
            .into_iter()
            .map(|c| Cand { dist: sq_dist(query, self.node_vector(c.slot)), slot: c.slot })
            .collect();
        reranked.sort_unstable();
        reranked.truncate(k);
        // d² over unit vectors → cosine: cos = 1 − d²/2.
        reranked.into_iter().map(|c| (c.slot, 1.0 - c.dist / 2.0)).collect()
    }

    /// **Filtered** greedy beam search: traverse the Vamana graph *predicate-agnostically* (expand
    /// through every neighbour, beam-truncated toward `query` — exactly as
    /// [`search_slots`](Self::search_slots) does, so connectivity is preserved) but only **accept**
    /// into the result the slots whose id passes `accept`. ACORN / NaviX-style predicate-aware
    /// acceptance. Returns the `k` accepted slots closest to `query`, best first.
    #[cfg(feature = "filtered")]
    fn search_slots_filtered(
        &self,
        query: &[f32],
        k: usize,
        beam: usize,
        accept: impl Fn(VectorId) -> bool,
    ) -> Vec<(u32, f32)> {
        if self.count == 0 {
            return Vec::new();
        }
        let beam = beam.max(k).max(1);
        // `working`: the traversal frontier (ALL nodes, mask-agnostic — drives expansion exactly as
        // the unfiltered search, so the graph's short paths are kept). `accepted`: masked nodes seen
        // so far, kept separately so a masked hit is never lost when the beam truncates it out.
        let mut working: Vec<Cand> = Vec::with_capacity(beam + self.degree);
        let mut accepted: Vec<Cand> = Vec::new();
        let mut in_working = vec![false; self.count];
        let mut expanded = vec![false; self.count];
        let consider = |slot: u32, dist: f32, accepted: &mut Vec<Cand>| {
            if accept(self.node_id(slot)) {
                accepted.push(Cand { dist, slot });
            }
        };
        let start = self.medoid;
        let start_d = sq_dist(query, self.node_vector(start));
        working.push(Cand { dist: start_d, slot: start });
        in_working[start as usize] = true;
        consider(start, start_d, &mut accepted);
        loop {
            let next = working.iter().filter(|c| !expanded[c.slot as usize]).min().copied();
            let Some(cur) = next else { break };
            expanded[cur.slot as usize] = true;
            let neighbours: Vec<u32> = self.node_neighbours(cur.slot).collect();
            for nbr in neighbours {
                if in_working[nbr as usize] {
                    continue;
                }
                in_working[nbr as usize] = true;
                let d = sq_dist(query, self.node_vector(nbr));
                consider(nbr, d, &mut accepted);
                working.push(Cand { dist: d, slot: nbr });
            }
            if working.len() > beam {
                working.sort_unstable();
                working.truncate(beam);
            }
        }
        accepted.sort_unstable();
        accepted.dedup_by_key(|c| c.slot);
        accepted.truncate(k);
        // d² over unit vectors → cosine: cos = 1 − d²/2.
        accepted.into_iter().map(|c| (c.slot, 1.0 - c.dist / 2.0)).collect()
    }

    /// **Predicate-constrained (filtered) approximate top-`k`**: like [`nearest`](Self::nearest)
    /// but the result is restricted to the ids `accept` admits, while the traversal still hops
    /// through non-matching nodes for connectivity.
    ///
    /// `beam` is the traversal beam width. Because accepted nodes are a SUBSET of visited nodes,
    /// a caller should widen it over [`search_beam`](Self::search_beam) so `k` accepted nodes can
    /// still be collected. An all-zero `query` returns no results (same contract as
    /// [`nearest`](Self::nearest)).
    ///
    /// Panics if `query.len() != self.dim()` (a programming error).
    #[cfg(feature = "filtered")]
    pub fn nearest_filtered_by(
        &self,
        query: &[f32],
        k: usize,
        beam: usize,
        accept: impl Fn(VectorId) -> bool,
    ) -> Vec<(VectorId, f32)> {
        assert_eq!(query.len(), self.dim, "query dim {} != index dim {}", query.len(), self.dim);
        let Some(q) = normalized(query) else { return Vec::new() };
        self.search_slots_filtered(&q, k, beam, accept)
            .into_iter()
            .map(|(slot, cos)| (self.node_id(slot), cos))
            .collect()
    }

    /// Approximate top-`k` ids by cosine similarity to `query`, best first. An all-zero `query`
    /// returns no results.
    ///
    /// Panics if `query.len() != self.dim()` (a programming error).
    pub fn nearest(&self, query: &[f32], k: usize) -> Vec<(VectorId, f32)> {
        assert_eq!(query.len(), self.dim, "query dim {} != index dim {}", query.len(), self.dim);
        let Some(q) = normalized(query) else { return Vec::new() };
        self.search_slots(&q, k, self.search_beam.max(k))
            .into_iter()
            .map(|(slot, cos)| (self.node_id(slot), cos))
            .collect()
    }
}

/// Default path for a vector file's sibling `.spqg` graph artifact: the path with its extension
/// replaced by `spqg` (so `entities.spqv` → `entities.spqg`). A convenience for the common
/// "graph lives next to the vectors" layout; any path works with [`VamanaIndex::build`].
pub fn sibling_graph_path(store_path: &Path) -> PathBuf {
    store_path.with_extension("spqg")
}
