//! The `.spqv` on-disk vector store: one f32 embedding per dictionary term id, in a
//! single flat memory-mapped file.
//!
//! # File format (version 2, little-endian)
//!
//! ```text
//! offset 0   magic        b"SPQV"                          4 bytes
//! offset 4   version      u32 = 2                          4 bytes
//! offset 8   dim          u32                              4 bytes
//! offset 12  count        u64                              8 bytes
//! offset 20  reserved     zero padding                     12 bytes
//! offset 32  fingerprint  graph fingerprint                24 bytes
//!            (dict_len: u64, triple_count: u64, content_hash: u64)
//! offset 56  data         [count × dim] f32, dense         count·dim·4 bytes
//! offset 56+count·dim·4   id→slot index                    count·8 bytes
//!            (id: u32, slot: u32) pairs sorted by id ascending
//! ```
//!
//! [OPUS-4.8] (sq-32i5) The **fingerprint** binds the store to the graph it was built against
//! (see [`crate::fingerprint`]): a store keyed by dictionary id is meaningless against a graph
//! whose ids have shifted, so [`VectorStore::check_graph`] errors on a mismatch instead of letting
//! a query silently mis-resolve. Version-1 files (no fingerprint block, 32-byte header) still open,
//! but `check_graph` reports them as unverifiable — see [`VectorStore::open`] and the back-compat
//! note there.
//!
//! [OPUS-4.8] (sq-wlzi) **ID-KEYED STALENESS CONTRACT.** Because every embedding is keyed by the
//! build-time dictionary id, a store is valid ONLY against the **exact graph generation it was built
//! against**. To serve it, persist that graph (`Graph::save`) and reopen THAT graph (`Graph::open`,
//! which mmaps the **frozen** dict id order — both gated by `sparq-core`'s `mmap` feature) to resolve
//! query terms — **never re-parse the source RDF** (`Graph::load_str` et al.): sparq-core's parallel
//! sharded dict merge assigns thread-count-dependent ids, so a re-parse produces a *different*
//! `id → term` binding and `get`/`nearest_term` mis-resolve. `check_graph` is a
//! backstop, **not** a sufficient guard for this case — the sq-xhiv fingerprint folds the term *set*
//! and is thread-count-stable, so it PASSES a re-parse of the same RDF even though the ids permuted.
//! See [`crate::fingerprint`] for the full rationale and `tests/staleness_contract.rs` for the
//! round-trip-vs-trap demonstration.
//!
//! Coverage is **sparse by design**: in an RDF graph only entities get embeddings, not
//! every literal, so vectors are stored densely in *insertion* slots and the trailing
//! sorted `(id, slot)` index maps dictionary ids to slots — `get` on an mmap'd store is
//! one binary search plus a contiguous slice, no per-vector allocation. The data and
//! index sections both start at multiples of 4 from the page-aligned map, so the f32
//! casts are always aligned.
//!
//! Build phase ([`VectorStore::create`] + [`VectorStore::put`]) accumulates vectors in
//! RAM; [`VectorStore::finalize`] writes the file once and re-opens it memory-mapped,
//! so the same handle serves reads before *and* after finalization. For datasets whose
//! dense data does not fit in RAM, [`StreamingWriter`] appends each vector to the data
//! section as it is `put` and spills the id→slot index to a sidecar file, so build-phase
//! memory is O(1) (finalize transiently holds the 8-byte-per-vector index to sort it —
//! a `dim × 4 / 8` reduction, e.g. 192× for 384-d f32). Both writers produce
//! byte-identical version-1 files.

use crate::fingerprint::{self, Fingerprint, FINGERPRINT_LEN};
use memmap2::Mmap;
use rustc_hash::FxHashMap;
use sparq_core::dict::Id;
use sparq_core::Graph;
use std::io::{BufWriter, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

/// First four bytes of every `.spqv` file.
pub const SPQV_MAGIC: [u8; 4] = *b"SPQV";
/// Current format version. [OPUS-4.8] (sq-32i5) v2 adds the 24-byte graph fingerprint block at
/// offset 32; v1 files (32-byte header, no fingerprint) still open but cannot be verified.
pub const SPQV_VERSION: u32 = 2;
/// Header length of a version-1 file (no fingerprint block).
const HEADER_LEN_V1: usize = 32;
/// Header length of the current (version-2) format: the v1 header + the fingerprint block.
const HEADER_LEN: usize = HEADER_LEN_V1 + FINGERPRINT_LEN;

/// [OPUS-4.8] A 4-byte-aligned owned byte buffer. A plain `Vec<u8>` has alignment 1, so its base
/// pointer may land on an odd address; `slot_vector` casts `&[u8]` slices of the buffer to
/// `&[f32]` via `from_raw_parts`, which is UNDEFINED BEHAVIOR on an unaligned pointer. Backing the
/// owned bytes with a `Vec<u32>` (alignment 4) guarantees the base — and therefore every
/// `HEADER_LEN + slot·dim·4` offset (all multiples of 4) — is f32-aligned. See review 1874.
struct AlignedBytes {
    /// Backing storage; only `len` bytes are logically valid (the last word may be padding).
    words: Vec<u32>,
    len: usize,
}

impl AlignedBytes {
    fn from_vec(bytes: Vec<u8>) -> AlignedBytes {
        let len = bytes.len();
        // ceil(len / 4) words; the final word is zero-padded.
        let words = vec![0u32; len.div_ceil(4)];
        let mut ab = AlignedBytes { words, len };
        // SAFETY: `words` holds at least `len` bytes (rounded up to a word) and is u32-aligned
        // (≥ align(u8)); the destination region is exclusively owned here.
        let dst =
            unsafe { std::slice::from_raw_parts_mut(ab.words.as_mut_ptr() as *mut u8, len) };
        dst.copy_from_slice(&bytes);
        ab
    }

    fn as_bytes(&self) -> &[u8] {
        // SAFETY: `words` is u32-aligned and holds ≥ `len` initialized bytes (the copy above);
        // f32/u8 reads of this region are in bounds. The base is 4-byte aligned by construction.
        unsafe { std::slice::from_raw_parts(self.words.as_ptr() as *const u8, self.len) }
    }
}

/// Read-phase backing bytes: a memory map ([`VectorStore::open`]) or an owned
/// buffer ([`VectorStore::open_from_bytes`] — environments without a
/// filesystem). Both deref to `[u8]`; every read path is shared.
enum Bytes {
    Map(Mmap),
    /// [OPUS-4.8] f32-aligned owned bytes (see `AlignedBytes`) so the `slot_vector` f32 cast is
    /// always aligned — a plain `Vec<u8>` is alignment 1 and would risk UB. Review 1874.
    Owned(AlignedBytes),
}

impl std::ops::Deref for Bytes {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        match self {
            Bytes::Map(m) => m,
            Bytes::Owned(v) => v.as_bytes(),
        }
    }
}

enum Backing {
    /// Build phase: vectors accumulate in RAM; `finalize` writes the file.
    Build { data: Vec<f32>, slots: FxHashMap<Id, u32>, ids: Vec<Id> },
    /// Read phase: the whole file is memory-mapped or held as owned bytes.
    Map(Bytes),
}

/// A flat per-term-id f32 vector store backed by one `.spqv` file. See the module docs
/// for the format and the build/read lifecycle.
pub struct VectorStore {
    dim: usize,
    path: PathBuf,
    backing: Backing,
    /// [OPUS-4.8] (sq-32i5) The graph fingerprint bound to this store: `Some` for the current
    /// (version-2) format, `None` for a legacy version-1 file that predates fingerprinting (it
    /// opens but cannot be verified — see [`open`](Self::open)) and for a freshly `create`d store
    /// until [`with_fingerprint`](Self::with_fingerprint) is called.
    /// [`check_graph`](Self::check_graph) uses it to reject a stale store before a query
    /// can mis-resolve.
    fingerprint: Option<Fingerprint>,
    /// Byte offset where the dense vector data begins: [`HEADER_LEN`] for a version-2 file (the
    /// current build path), [`HEADER_LEN_V1`] for an opened legacy version-1 file. Every read path
    /// (`get`, `iter`, `slot_vector`, the trailing index) keys off this so both versions are read
    /// correctly by the same code.
    data_offset: usize,
    /// slot→id for mmap mode (the on-disk index is sorted by id, the data by slot);
    /// built lazily on the first [`iter`](Self::iter), O(count) once.
    inverse: std::sync::OnceLock<Vec<Id>>,
    /// [OPUS-4.8] (sq-pi44) The in-RAM delta sidecar: vectors appended/updated and ids tombstoned
    /// since the immutable base was built. `None` until the first delta mutation
    /// ([`add`](Self::add)/[`remove`](Self::remove)/[`update`](Self::update)) allocates it, bound to
    /// the base fingerprint. Every read path
    /// ([`get`](Self::get)/[`iter`](Self::iter)/[`len`](Self::len)) consults it transparently;
    /// [`compact`](Self::compact) folds it back into a fresh base. Gated behind the opt-in `delta`
    /// feature, so the default build carries no delta state. See [`crate::delta`].
    #[cfg(feature = "delta")]
    delta: Option<crate::delta::VectorDelta>,
}

impl VectorStore {
    /// Starts a new store that will be written to `path` (build phase). `dim` is fixed
    /// for the store's lifetime; every [`put`](Self::put) must match it.
    pub fn create<P: Into<PathBuf>>(path: P, dim: usize) -> Result<VectorStore, String> {
        if dim == 0 || dim > u32::MAX as usize {
            return Err(format!("invalid vector dimension {dim}"));
        }
        if cfg!(target_endian = "big") {
            return Err(".spqv is a little-endian format; big-endian targets are unsupported".into());
        }
        Ok(VectorStore {
            dim,
            path: path.into(),
            backing: Backing::Build { data: Vec::new(), slots: FxHashMap::default(), ids: Vec::new() },
            fingerprint: None,
            data_offset: HEADER_LEN,
            inverse: std::sync::OnceLock::new(),
            #[cfg(feature = "delta")]
            delta: None,
        })
    }

    /// [OPUS-4.8] (sq-32i5) Binds this build-phase store to `graph`'s fingerprint, which
    /// [`finalize`](Self::finalize) then embeds in the `.spqv` header. Call it before `finalize`
    /// (most naturally right after `create`, against the SAME graph whose term ids the vectors are
    /// keyed by). A store finalized WITHOUT a fingerprint is written in the version-2 format with a
    /// zeroed fingerprint, which [`check_graph`](Self::check_graph) treats as "unverified"; prefer
    /// setting it so a stale store is caught. Chains: `VectorStore::create(p, d)?.with_fingerprint(g)`.
    #[must_use]
    pub fn with_fingerprint(mut self, graph: &Graph) -> VectorStore {
        self.fingerprint = Some(Fingerprint::of(graph));
        self
    }

    /// Memory-maps an existing `.spqv` file read-only. Cheap: only the header and the trailing
    /// `count·8`-byte id→slot index are read eagerly (the index is validated so no later read can
    /// panic on a corrupt file); the vector data — the overwhelming bulk of the file — is paged in
    /// by the OS on access.
    ///
    /// [OPUS-4.8] (sq-32i5) Back-compat: a current (version-2) file's graph fingerprint is read into
    /// [`fingerprint`](Self::fingerprint) for [`check_graph`](Self::check_graph). A legacy version-1
    /// file (32-byte header, no fingerprint) still opens — its `fingerprint` is `None`, so
    /// `check_graph` reports it as unverifiable rather than silently passing. Rebuild such a store to
    /// enable the staleness check.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<VectorStore, String> {
        let path = path.as_ref();
        let file = std::fs::File::open(path).map_err(|e| format!("open {}: {e}", path.display()))?;
        // SAFETY: read-only map of a regular file; we treat concurrent external
        // modification of the file as out of contract (same stance as sparq-core's
        // mmap'd dictionary/indexes).
        let map = unsafe { Mmap::map(&file) }.map_err(|e| format!("mmap {}: {e}", path.display()))?;
        Self::open_validated(Bytes::Map(map), path.to_path_buf(), &path.display().to_string())
    }

    /// Opens a `.spqv` document held entirely in memory — for environments
    /// without a filesystem (the bytes were fetched, embedded, or decompressed
    /// by the caller). Validation is identical to [`open`](Self::open); reads
    /// borrow the owned buffer instead of a memory map. The handle is
    /// read-only (`put`/`finalize` behave as on any opened store).
    pub fn open_from_bytes(bytes: Vec<u8>) -> Result<VectorStore, String> {
        // [OPUS-4.8] Copy into a 4-byte-aligned backing so the read-phase f32 casts are aligned
        // (a plain Vec<u8> is alignment 1 — casting its slices to &[f32] is UB). Review 1874.
        Self::open_validated(Bytes::Owned(AlignedBytes::from_vec(bytes)), PathBuf::new(), "<bytes>")
    }

    /// Shared header/index validation behind [`open`](Self::open) and
    /// [`open_from_bytes`](Self::open_from_bytes).
    fn open_validated(map: Bytes, path: PathBuf, origin: &str) -> Result<VectorStore, String> {
        if cfg!(target_endian = "big") {
            return Err(".spqv is a little-endian format; big-endian targets are unsupported".into());
        }
        // The version-1 header is the smallest valid header; version 2 adds a fixed-size block.
        if map.len() < HEADER_LEN_V1 {
            return Err(format!("{origin}: truncated header"));
        }
        if map[0..4] != SPQV_MAGIC {
            return Err(format!("{origin}: not a .spqv file (bad magic)"));
        }
        let version = u32::from_le_bytes(map[4..8].try_into().unwrap());
        // [OPUS-4.8] (sq-32i5) Both v1 (no fingerprint, 32-byte header) and v2 (fingerprint,
        // 56-byte header) open; the header length and the offset where the vector data begins
        // depend on the version, so every downstream read keys off `data_offset` below.
        let (data_offset, fingerprint): (usize, Option<Fingerprint>) = match version {
            1 => (HEADER_LEN_V1, None),
            2 => {
                if map.len() < HEADER_LEN {
                    return Err(format!("{origin}: truncated version-2 header (fingerprint block)"));
                }
                // [OPUS-4.8] (sq-32i5) An all-zero block (a v2 store finalized without
                // `with_fingerprint`) decodes to `None` ("unverifiable"), not a zero fingerprint
                // that would surface as a spurious "DIFFERENT graph" mismatch.
                let fp = Fingerprint::from_bytes_opt(&map[HEADER_LEN_V1..HEADER_LEN]);
                (HEADER_LEN, fp)
            }
            v => return Err(format!("{origin}: unsupported .spqv version {v}")),
        };
        let dim = u32::from_le_bytes(map[8..12].try_into().unwrap()) as usize;
        let count64 = u64::from_le_bytes(map[12..20].try_into().unwrap());
        if dim == 0 {
            return Err(format!("{origin}: zero dimension"));
        }
        let count: usize = count64
            .try_into()
            .map_err(|_| format!("{origin}: count {count64} exceeds the address space"))?;
        // Checked size arithmetic: a malformed header must be rejected here, not wrap
        // around and pass the size check (or panic later in `get`/`iter`).
        let expect = count
            .checked_mul(dim)
            .and_then(|n| n.checked_mul(4))
            .and_then(|data| count.checked_mul(8).and_then(|idx| data.checked_add(idx)))
            .and_then(|body| body.checked_add(data_offset))
            .ok_or_else(|| format!("{origin}: dim={dim} count={count} overflows the file size"))?;
        if map.len() != expect {
            return Err(format!(
                "{origin}: file is {} bytes, expected {expect} for dim={dim} count={count}",
                map.len()
            ));
        }
        // Validate the trailing id→slot index up front (one sequential pass over
        // count·8 bytes — a small fraction of the file; the vector data itself stays
        // untouched/unpaged): ids strictly ascending (sorted + unique, what `get`'s
        // binary search assumes) and every slot in range and used exactly once (what
        // `iter`'s inverse map assumes). After this, no read path can panic on a
        // corrupt index.
        {
            let index = &map[data_offset + count * dim * 4..];
            let mut slot_seen = vec![false; count];
            let mut prev_id: Option<Id> = None;
            for i in 0..count {
                let id = u32::from_le_bytes(index[i * 8..i * 8 + 4].try_into().unwrap());
                let slot =
                    u32::from_le_bytes(index[i * 8 + 4..i * 8 + 8].try_into().unwrap()) as usize;
                if prev_id.is_some_and(|p| p >= id) {
                    return Err(format!(
                        "{origin}: index entry {i} (id {id}) is not strictly ascending"
                    ));
                }
                prev_id = Some(id);
                if slot >= count || slot_seen[slot] {
                    return Err(format!(
                        "{origin}: index entry {i} has invalid or duplicate slot {slot} (count {count})"
                    ));
                }
                slot_seen[slot] = true;
            }
        }
        Ok(VectorStore {
            dim,
            path,
            backing: Backing::Map(map),
            fingerprint,
            data_offset,
            inverse: std::sync::OnceLock::new(),
            #[cfg(feature = "delta")]
            delta: None,
        })
    }

    /// Appends `vector` for term `id` (build phase only). Errors after
    /// [`finalize`](Self::finalize)/[`open`](Self::open), on a dimension mismatch, on a
    /// non-finite component, on an all-zero vector (no direction — cosine is undefined
    /// on it, so the exact and HNSW searchers could not even agree on its rank), or if
    /// `id` already has a vector.
    pub fn put(&mut self, id: Id, vector: &[f32]) -> Result<(), String> {
        validate_vector(self.dim, id, vector)?;
        let Backing::Build { data, slots, ids } = &mut self.backing else {
            return Err("put on a finalized/opened store (the .spqv file is immutable)".into());
        };
        if slots.contains_key(&id) {
            return Err(format!("id {id} already has a vector"));
        }
        slots.insert(id, ids.len() as u32);
        ids.push(id);
        data.extend_from_slice(vector);
        Ok(())
    }

    /// Writes the `.spqv` file and re-opens it memory-mapped; the handle then serves
    /// reads from the map and rejects further [`put`](Self::put)s. Idempotent on an
    /// already-finalized/opened store.
    pub fn finalize(&mut self) -> Result<(), String> {
        let Backing::Build { data, slots, ids } = &mut self.backing else {
            return Ok(());
        };
        let count = ids.len();
        let mut header = [0u8; HEADER_LEN];
        header[0..4].copy_from_slice(&SPQV_MAGIC);
        header[4..8].copy_from_slice(&SPQV_VERSION.to_le_bytes());
        header[8..12].copy_from_slice(&(self.dim as u32).to_le_bytes());
        header[12..20].copy_from_slice(&(count as u64).to_le_bytes());
        // [OPUS-4.8] (sq-32i5) Embed the graph fingerprint (offset 32..56). A store finalized
        // without one (no `with_fingerprint`) writes a zeroed block, which `open` decodes back to
        // `None` (`Fingerprint::from_bytes_opt`) — reported by `check_graph` as "unverifiable",
        // NOT as a spurious "DIFFERENT graph" mismatch; prefer setting it explicitly.
        if let Some(fp) = self.fingerprint {
            header[HEADER_LEN_V1..HEADER_LEN].copy_from_slice(&fp.to_bytes());
        }

        let mut index: Vec<(Id, u32)> = slots.iter().map(|(&id, &slot)| (id, slot)).collect();
        index.sort_unstable();
        let mut index_bytes = Vec::with_capacity(count * 8);
        for (id, slot) in index {
            index_bytes.extend_from_slice(&id.to_le_bytes());
            index_bytes.extend_from_slice(&slot.to_le_bytes());
        }

        // f32 → LE bytes: a plain cast on little-endian targets (`create`/`open` reject
        // big-endian above).
        // SAFETY: f32 has no invalid bit patterns and align(f32) ≥ align(u8).
        let data_bytes =
            unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * 4) };

        let mut file = std::fs::File::create(&self.path)
            .map_err(|e| format!("create {}: {e}", self.path.display()))?;
        use std::io::Write;
        file.write_all(&header)
            .and_then(|()| file.write_all(data_bytes))
            .and_then(|()| file.write_all(&index_bytes))
            .and_then(|()| file.flush())
            .map_err(|e| format!("write {}: {e}", self.path.display()))?;
        drop(file);

        let reopened = VectorStore::open(&self.path)?;
        self.backing = reopened.backing;
        self.fingerprint = reopened.fingerprint;
        self.data_offset = reopened.data_offset;
        self.inverse = std::sync::OnceLock::new();
        Ok(())
    }

    /// The vector for term `id`, or `None` if it has none. Works in both phases:
    /// a hash lookup during build, a binary search over the mmap'd index after.
    ///
    /// [OPUS-4.8] (sq-pi44) With the `delta` feature, the in-RAM delta is consulted FIRST: a
    /// tombstoned id reads as absent, an appended/updated id reads its delta vector (shadowing the
    /// base), and any other id reads the immutable base — so search transparently sees the delta.
    pub fn get(&self, id: Id) -> Option<&[f32]> {
        #[cfg(feature = "delta")]
        if let Some(delta) = &self.delta {
            if delta.tombstones.contains(&id) {
                return None;
            }
            if let Some(v) = delta.appended.get(&id) {
                return Some(v.as_slice());
            }
        }
        self.base_get(id)
    }

    /// The vector for `id` from the immutable base only (no delta) — the original `get` body. The
    /// delta-aware [`get`](Self::get) calls this after consulting the delta.
    fn base_get(&self, id: Id) -> Option<&[f32]> {
        match &self.backing {
            Backing::Build { data, slots, .. } => {
                let slot = *slots.get(&id)? as usize;
                Some(&data[slot * self.dim..(slot + 1) * self.dim])
            }
            Backing::Map(map) => {
                // [OPUS-4.8] (sq-pi44) The BASE count (not the delta-aware `len`): this reads the
                // on-disk index, and the delta-aware `len` calls back into `base_get`, so using it
                // here would recurse infinitely.
                let count = self.base_len();
                let index = &map[self.data_offset + count * self.dim * 4..];
                let id_at = |i: usize| -> Id {
                    u32::from_le_bytes(index[i * 8..i * 8 + 4].try_into().unwrap())
                };
                let (mut lo, mut hi) = (0usize, count);
                while lo < hi {
                    let mid = lo + (hi - lo) / 2;
                    if id_at(mid) < id {
                        lo = mid + 1;
                    } else {
                        hi = mid;
                    }
                }
                if lo == count || id_at(lo) != id {
                    return None;
                }
                let slot =
                    u32::from_le_bytes(index[lo * 8 + 4..lo * 8 + 8].try_into().unwrap()) as usize;
                Some(self.slot_vector(map, slot))
            }
        }
    }

    /// Number of stored vectors — the EFFECTIVE count once the in-RAM delta (appended/updated
    /// vectors and tombstones) is applied. With the `delta` feature off, or no delta yet, this is
    /// the base count.
    pub fn len(&self) -> usize {
        #[cfg(feature = "delta")]
        if let Some(delta) = &self.delta {
            // effective = base − (base ids that are tombstoned) − (base ids shadowed by an append)
            //             + (appended ids). Tombstones and appends are disjoint by construction, so
            // a base id is double-counted with the appended set only via the `append` branch.
            let base = self.base_len();
            let removed = delta.tombstones.iter().filter(|&&id| self.base_get(id).is_some()).count();
            let shadowed =
                delta.appended.keys().filter(|&&id| self.base_get(id).is_some()).count();
            return base - removed - shadowed + delta.appended.len();
        }
        self.base_len()
    }

    /// The base (on-disk / build-phase) vector count, ignoring any in-RAM delta — the original
    /// `len` body. The delta-aware [`len`](Self::len) builds on this.
    fn base_len(&self) -> usize {
        match &self.backing {
            Backing::Build { ids, .. } => ids.len(),
            Backing::Map(map) => u64::from_le_bytes(map[12..20].try_into().unwrap()) as usize,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The store's vector dimension.
    pub fn dim(&self) -> usize {
        self.dim
    }

    /// [OPUS-4.8] (sq-32i5) The graph fingerprint this store was built against, or `None` for a
    /// legacy version-1 file (predates fingerprinting) or a store finalized without
    /// [`with_fingerprint`](Self::with_fingerprint). See [`check_graph`](Self::check_graph).
    pub fn fingerprint(&self) -> Option<Fingerprint> {
        self.fingerprint
    }

    /// [OPUS-4.8] (sq-32i5) **Checked open guard**: verifies this store was built against `graph`
    /// by recomputing `graph`'s fingerprint and comparing it to the stored one. Returns a
    /// descriptive `Err` if they differ — the store is keyed by `graph`'s dictionary ids, so a
    /// mismatch means a query would silently resolve to the WRONG vectors. A legacy version-1 file
    /// (no stored fingerprint) also errors, as "unverifiable" rather than silently passing.
    ///
    /// Call this once after [`open`](Self::open) (it is O(dict_len), not per-query) before issuing
    /// `get`/`nearest_term` against `graph`. The term-by-query entry points
    /// ([`crate::ann::nearest_term_exact_checked`], `DiskAnnIndex::nearest_term_checked`) run it.
    pub fn check_graph(&self, graph: &Graph) -> fingerprint::CheckResult {
        let origin = if self.path.as_os_str().is_empty() {
            "<bytes>".to_string()
        } else {
            self.path.display().to_string()
        };
        fingerprint::check_against(self.fingerprint, graph, fingerprint::Artifact::Store, &origin)
    }

    /// Iterates all `(id, vector)` pairs — what ANN index construction consumes. Base entries come
    /// first in insertion-slot order, then any delta-appended entries.
    ///
    /// [OPUS-4.8] (sq-pi44) With the `delta` feature, the EFFECTIVE set is yielded: a tombstoned
    /// base id is skipped, a base id shadowed by a delta append is skipped (the appended vector is
    /// yielded instead, from the delta tail), and delta-only appends are appended after the base —
    /// so search over `iter` transparently unions base+delta and honours tombstones.
    pub fn iter(&self) -> impl Iterator<Item = (Id, &[f32])> + '_ {
        let base_count = self.base_len();
        let base = (0..base_count)
            .map(move |slot| match &self.backing {
                Backing::Build { data, ids, .. } => {
                    (ids[slot], &data[slot * self.dim..(slot + 1) * self.dim])
                }
                Backing::Map(map) => (self.slot_id(map, slot), self.slot_vector(map, slot)),
            })
            .filter(move |&(id, _)| self.base_id_is_live(id));
        base.chain(self.delta_iter())
    }

    /// Whether base id `id` survives the delta into the effective view: not tombstoned and not
    /// shadowed by a delta append (the append is yielded from the delta tail instead, to avoid a
    /// duplicate). Always `true` with the `delta` feature off / no delta.
    #[allow(unused_variables)]
    fn base_id_is_live(&self, id: Id) -> bool {
        #[cfg(feature = "delta")]
        if let Some(delta) = &self.delta {
            return !delta.tombstones.contains(&id) && !delta.appended.contains_key(&id);
        }
        true
    }

    /// The delta-appended `(id, vector)` pairs (empty with the `delta` feature off / no delta), to
    /// chain after the base in [`iter`](Self::iter).
    fn delta_iter(&self) -> impl Iterator<Item = (Id, &[f32])> + '_ {
        #[cfg(feature = "delta")]
        {
            self.delta
                .iter()
                .flat_map(|d| d.appended.iter().map(|(&id, v)| (id, v.as_slice())))
        }
        #[cfg(not(feature = "delta"))]
        {
            std::iter::empty()
        }
    }

    /// The id stored at insertion `slot` (mmap mode). The on-disk index only maps
    /// id→slot (sorted by id, for `get`'s binary search); the slot→id direction is
    /// rebuilt lazily in RAM rather than duplicated in the file.
    fn slot_id(&self, map: &Bytes, slot: usize) -> Id {
        self.inverse_ids(map)[slot]
    }

    /// slot→id for mmap mode, built once (O(count)) and cached.
    fn inverse_ids(&self, map: &Bytes) -> &[Id] {
        self.inverse.get_or_init(|| {
            let count = u64::from_le_bytes(map[12..20].try_into().unwrap()) as usize;
            let index = &map[self.data_offset + count * self.dim * 4..];
            let mut inv = vec![0 as Id; count];
            for i in 0..count {
                let id = u32::from_le_bytes(index[i * 8..i * 8 + 4].try_into().unwrap());
                let slot =
                    u32::from_le_bytes(index[i * 8 + 4..i * 8 + 8].try_into().unwrap()) as usize;
                inv[slot] = id;
            }
            inv
        })
    }

    fn slot_vector<'m>(&self, map: &'m Bytes, slot: usize) -> &'m [f32] {
        let start = self.data_offset + slot * self.dim * 4;
        let bytes = &map[start..start + self.dim * 4];
        debug_assert_eq!(bytes.as_ptr() as usize % std::mem::align_of::<f32>(), 0);
        // SAFETY: the backing base is 4-byte aligned — a memory map is page-aligned, and the
        // owned path uses `AlignedBytes` (u32-backed, alignment 4) precisely so this cast is
        // aligned ([OPUS-4.8] review 1874; a raw Vec<u8> is alignment 1 and would be UB here).
        // `start` is a multiple of 4 (data_offset [32 or 56] + slot·dim·4), so the pointer stays
        // f32-aligned; the range is in bounds (validated in `open`); f32 accepts any bit pattern;
        // the slice borrows the backing, owned by `self`.
        unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const f32, self.dim) }
    }
}

// [OPUS-4.8] (sq-pi44) The incremental delta layer: add / remove / update against a finalized store
// plus `compact`. Gated behind the opt-in `delta` feature so the default build carries no delta
// surface. See [`crate::delta`] for the design and the honest in-RAM-only scope.
#[cfg(feature = "delta")]
impl VectorStore {
    /// The in-RAM delta sidecar, allocated on first mutation and bound to the base store's
    /// fingerprint (the generation the appended/tombstoned ids are keyed against).
    fn delta_mut(&mut self) -> &mut crate::delta::VectorDelta {
        let generation = self.fingerprint;
        self.delta.get_or_insert_with(|| crate::delta::VectorDelta::new(generation))
    }

    /// [OPUS-4.8] (sq-pi44) **Add a NEW vector** for term `id` against an already-finalized store,
    /// writing it to the in-RAM delta (no file rebuild). Errors on the same vector validation as
    /// [`put`](Self::put) (dimension, finiteness, non-zero direction), or if `id` ALREADY has an
    /// effective vector (present in the base and not tombstoned, or already in the delta) — use
    /// [`update`](Self::update) to replace an existing one. May be called before or after
    /// `finalize`; on a build-phase store it is equivalent to `put`.
    pub fn add(&mut self, id: Id, vector: &[f32]) -> Result<(), String> {
        validate_vector(self.dim, id, vector)?;
        // On a build-phase store, route to the in-RAM data exactly like `put` (no delta needed).
        if matches!(self.backing, Backing::Build { .. }) {
            return self.put(id, vector);
        }
        if self.get(id).is_some() {
            return Err(format!(
                "id {id} already has a vector; use `update` to replace it"
            ));
        }
        let delta = self.delta_mut();
        delta.tombstones.remove(&id);
        delta.appended.insert(id, vector.to_vec());
        Ok(())
    }

    /// [OPUS-4.8] (sq-pi44) **Remove** term `id`'s vector from the effective view by tombstoning it
    /// in the in-RAM delta (no file rebuild). A subsequent [`get`](Self::get)/[`iter`](Self::iter)
    /// excludes it. Works whether the id lives in the base or only in the delta. Returns `true` if
    /// the id had an effective vector that this removed, `false` if it had none (a no-op). On a
    /// build-phase store the vector is dropped directly from the in-RAM build set.
    pub fn remove(&mut self, id: Id) -> bool {
        let dim = self.dim;
        // Build phase: drop directly from the accumulating data (re-pack the dense slots).
        if let Backing::Build { data, slots, ids } = &mut self.backing {
            let Some(slot) = slots.remove(&id) else { return false };
            let slot = slot as usize;
            // Remove the dense vector and the id at `slot`, then renumber the slots after it.
            data.drain(slot * dim..(slot + 1) * dim);
            ids.remove(slot);
            for s in slots.values_mut() {
                if (*s as usize) > slot {
                    *s -= 1;
                }
            }
            return true;
        }
        let had = self.get(id).is_some();
        if had {
            let delta = self.delta_mut();
            delta.appended.remove(&id);
            delta.tombstones.insert(id);
        }
        had
    }

    /// [OPUS-4.8] (sq-pi44) **Update** (replace) the vector for an EXISTING term `id`, writing the
    /// new vector to the in-RAM delta (no file rebuild). Errors on the same vector validation as
    /// [`put`](Self::put), or if `id` has no effective vector to replace (use [`add`](Self::add)
    /// for a new id). On a build-phase store it rewrites the in-RAM vector in place.
    pub fn update(&mut self, id: Id, vector: &[f32]) -> Result<(), String> {
        validate_vector(self.dim, id, vector)?;
        let dim = self.dim;
        if self.get(id).is_none() {
            return Err(format!("id {id} has no vector to update; use `add` for a new id"));
        }
        // Build phase: rewrite the dense slot in place.
        if let Backing::Build { data, slots, .. } = &mut self.backing {
            if let Some(&slot) = slots.get(&id) {
                let slot = slot as usize;
                data[slot * dim..(slot + 1) * dim].copy_from_slice(vector);
                return Ok(());
            }
        }
        let delta = self.delta_mut();
        delta.tombstones.remove(&id);
        delta.appended.insert(id, vector.to_vec());
        Ok(())
    }

    /// [OPUS-4.8] (sq-pi44) Whether this store has any pending in-RAM delta (appends or tombstones)
    /// not yet folded into the base by [`compact`](Self::compact).
    pub fn has_delta(&self) -> bool {
        self.delta.as_ref().is_some_and(|d| !d.is_empty())
    }

    /// [OPUS-4.8] (sq-pi44) The pending delta (appended/updated vectors + tombstones + the
    /// generation it is bound to), or `None` if none has been started. Read-only; for the
    /// generation-tie staleness check ([`apply_delta`](Self::apply_delta)) and introspection.
    pub fn delta(&self) -> Option<&crate::delta::VectorDelta> {
        self.delta.as_ref()
    }

    /// [OPUS-4.8] (sq-pi44) Removes and returns the pending in-RAM delta, leaving the store reading
    /// the bare base again. Pair with [`apply_delta`](Self::apply_delta) to move a delta from one
    /// store handle to another (the generation tie guards a mismatch).
    pub fn take_delta(&mut self) -> Option<crate::delta::VectorDelta> {
        self.delta.take()
    }

    /// [OPUS-4.8] (sq-pi44) Installs `delta` onto this store, REJECTING it if its bound graph
    /// generation does not match this base store's fingerprint — a delta built against generation N
    /// must not be applied to a base of generation M ≠ N (its ids would mis-key). Both a present
    /// mismatch and a delta-without-generation against a fingerprinted base (or vice-versa) are
    /// errors; only an exact generation match (or both `None`, the unverified-by-design case) is
    /// accepted. Replaces any delta already present.
    pub fn apply_delta(&mut self, delta: crate::delta::VectorDelta) -> Result<(), String> {
        if delta.generation() != self.fingerprint {
            return Err(format!(
                "delta generation mismatch: the delta was built against {}, this store is {} — \
                 applying it would mis-key the appended/tombstoned ids",
                describe_generation(delta.generation()),
                describe_generation(self.fingerprint),
            ));
        }
        self.delta = Some(delta);
        Ok(())
    }

    /// [OPUS-4.8] (sq-pi44) **Compaction**: folds the in-RAM delta into a FRESH base `.spqv` at
    /// `out_path`, returning the opened, fully validated store. The result is **equivalent to a
    /// from-scratch rebuild over the same final vector set**: its `get`/`iter`/`len` agree exactly
    /// with a store built by `create` + `put` over `self.iter()`'s effective `(id, vector)` pairs.
    /// The new base is bound to `graph`'s fingerprint (pass the SAME graph the effective ids are
    /// keyed by); the compacted store carries no delta.
    ///
    /// `out_path` may equal the base path — the new file is written first, then opened, so a
    /// same-path compaction replaces the old base atomically enough for the single-writer contract
    /// (concurrent external mutation is out of contract, as for [`open`](Self::open)).
    pub fn compact<P: Into<PathBuf>>(
        &self,
        out_path: P,
        graph: &Graph,
    ) -> Result<VectorStore, String> {
        let out_path = out_path.into();
        let mut fresh = VectorStore::create(&out_path, self.dim)?.with_fingerprint(graph);
        // Collect first so the new file is independent of `self`'s mmap (which may be the same
        // path): `iter()` yields the effective base+delta view in a deterministic order, and `put`
        // re-sorts the id→slot index on `finalize`, so the output is order-independent of the
        // collection order — i.e. byte-for-byte a from-scratch rebuild for the same id→vector map.
        let pairs: Vec<(Id, Vec<f32>)> =
            self.iter().map(|(id, v)| (id, v.to_vec())).collect();
        for (id, v) in &pairs {
            fresh.put(*id, v)?;
        }
        fresh.finalize()?;
        Ok(fresh)
    }

    /// [OPUS-4.8] (sq-7e50) The conventional persisted-delta sidecar path for a `.spqv` base at
    /// `base`: the base path with `d` substituted for the trailing `v` (`foo.spqv` → `foo.spqd`),
    /// or `base` + `.spqd` if it does not end in `.spqv`. Used by [`save_delta`](Self::save_delta)
    /// and [`open_with_delta`](Self::open_with_delta) so a delta lives next to its base by default.
    pub fn sibling_delta_path(base: &Path) -> PathBuf {
        let s = base.as_os_str().to_string_lossy();
        if let Some(stem) = s.strip_suffix(".spqv") {
            PathBuf::from(format!("{}.spqd", stem))
        } else {
            let mut p = base.as_os_str().to_os_string();
            p.push(".spqd");
            PathBuf::from(p)
        }
    }

    /// [OPUS-4.8] (sq-7e50) **Persists the in-RAM delta** to a `.spqd` sidecar at
    /// [`sibling_delta_path`](Self::sibling_delta_path) of this store's base path, so incremental
    /// add/remove/update survive a process restart without a [`compact`](Self::compact). Returns the
    /// sidecar path written. Crash-durable: the bytes are written to a sibling `.spqd-tmp`, `fsync`ed
    /// (`sync_all`), then atomically `rename`d over the final path — a crash mid-write leaves the
    /// previous sidecar (or none) intact, never a half-written file at the live path (the same
    /// discipline as [`StreamingWriter`]). The header carries this store's bound graph fingerprint
    /// (the generation tie), so [`open_with_delta`](Self::open_with_delta) rejects the sidecar
    /// against a mismatched base.
    ///
    /// A store with NO pending delta writes an empty (header-only) sidecar — replaying it is a no-op,
    /// and it still carries the generation so a stale base is still caught. See [`save_delta_to`](Self::save_delta_to) to
    /// choose the path explicitly.
    pub fn save_delta(&self) -> Result<PathBuf, String> {
        let path = Self::sibling_delta_path(&self.path);
        self.save_delta_to(&path)?;
        Ok(path)
    }

    /// [OPUS-4.8] (sq-7e50) Like [`save_delta`](Self::save_delta) but persists the `.spqd` sidecar to
    /// an explicit `path`. Serializes the in-RAM delta (or an empty delta bound to this store's
    /// fingerprint if none has been started — so the generation tie is still recorded), writing it
    /// crash-durably (tmp + `sync_all` + atomic rename).
    pub fn save_delta_to<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let path = path.as_ref();
        // An unstarted delta persists as an empty delta bound to THIS store's generation, so a
        // reopened base still validates the (empty) sidecar against its fingerprint.
        let bytes = match &self.delta {
            Some(d) => d.to_bytes(self.dim),
            None => crate::delta::VectorDelta::new(self.fingerprint).to_bytes(self.dim),
        };
        let tmp = {
            let mut p = path.as_os_str().to_os_string();
            p.push("-tmp");
            PathBuf::from(p)
        };
        // Write the full sidecar to the tmp path, fsync it, then atomically rename it over the
        // final path. On any error the partial tmp is removed so a live `.spqd` is never replaced
        // by a half-written one.
        let write_tmp = || -> std::io::Result<()> {
            let mut f = std::fs::File::create(&tmp)?;
            f.write_all(&bytes)?;
            f.sync_all()?;
            drop(f);
            std::fs::rename(&tmp, path)
        };
        write_tmp().map_err(|e| {
            std::fs::remove_file(&tmp).ok();
            format!("save_delta {}: {e}", path.display())
        })
    }

    /// [OPUS-4.8] (sq-7e50) Whether a persisted `.spqd` delta sidecar exists at
    /// [`sibling_delta_path`](Self::sibling_delta_path) of `base`.
    pub fn has_persisted_delta(base: &Path) -> bool {
        Self::sibling_delta_path(base).exists()
    }

    /// [OPUS-4.8] (sq-7e50) **Opens a `.spqv` base AND replays its persisted `.spqd` delta**, so a
    /// store reopened after a restart reads the SAME effective (base + delta) view it had before the
    /// handle dropped — no [`compact`](Self::compact) required. Equivalent to
    /// [`open`](Self::open)`(base)` followed by reading the sidecar at
    /// [`sibling_delta_path`](Self::sibling_delta_path) and [`apply_delta`](Self::apply_delta)ing it,
    /// so all of the existing guards apply:
    ///
    ///   * the base is fully validated exactly as [`open`](Self::open),
    ///   * the persisted delta's `dim` must match the base (else a descriptive `Err`),
    ///   * the persisted delta's base-generation fingerprint must match the base store's bound
    ///     fingerprint — `apply_delta` REJECTS a sidecar written against a different graph generation
    ///     (the sq-32i5 generation tie), so a stale `.spqd` against a rebuilt base can never mis-key,
    ///   * a truncated / corrupt sidecar is rejected by [`VectorDelta::from_bytes`](crate::delta::VectorDelta)
    ///     (the exact-length partial-write guard), never read out of bounds.
    ///
    /// If NO sidecar exists this is exactly [`open`](Self::open) (the store reads the bare base).
    pub fn open_with_delta<P: AsRef<Path>>(base: P) -> Result<VectorStore, String> {
        let base = base.as_ref();
        let delta_path = Self::sibling_delta_path(base);
        Self::open_with_delta_at(base, &delta_path)
    }

    /// [OPUS-4.8] (sq-7e50) Like [`open_with_delta`](Self::open_with_delta) but reads the persisted
    /// delta from an explicit `delta_path`. If `delta_path` does not exist, returns the bare opened
    /// base (no error — a base with no sidecar is the no-delta case).
    pub fn open_with_delta_at<P: AsRef<Path>, Q: AsRef<Path>>(
        base: P,
        delta_path: Q,
    ) -> Result<VectorStore, String> {
        let base = base.as_ref();
        let delta_path = delta_path.as_ref();
        let mut store = VectorStore::open(base)?;
        if !delta_path.exists() {
            return Ok(store);
        }
        let bytes = std::fs::read(delta_path)
            .map_err(|e| format!("read {}: {e}", delta_path.display()))?;
        let (delta, delta_dim) = crate::delta::VectorDelta::from_bytes(&bytes)
            .map_err(|e| format!("{}: {e}", delta_path.display()))?;
        if delta_dim != store.dim {
            return Err(format!(
                "{}: delta dimension {} does not match the base store dimension {}",
                delta_path.display(),
                delta_dim,
                store.dim
            ));
        }
        // `apply_delta` enforces the generation tie (delta fingerprint == base fingerprint).
        store
            .apply_delta(delta)
            .map_err(|e| format!("{}: {e}", delta_path.display()))?;
        Ok(store)
    }
}

/// [OPUS-4.8] (sq-pi44) A one-line description of a delta/store generation for the mismatch error.
#[cfg(feature = "delta")]
fn describe_generation(fp: Option<Fingerprint>) -> String {
    match fp {
        Some(f) => format!(
            "generation(dict_len={}, triples={}, hash={:#018x})",
            f.dict_len, f.triple_count, f.content_hash
        ),
        None => "an unbound/unverified generation".to_string(),
    }
}

/// Shared `put` validation: dimension, finiteness, non-zero direction.
fn validate_vector(dim: usize, id: Id, vector: &[f32]) -> Result<(), String> {
    if vector.len() != dim {
        return Err(format!("vector has dim {}, store has dim {dim}", vector.len()));
    }
    if vector.iter().any(|v| !v.is_finite()) {
        return Err(format!("vector for id {id} has a non-finite component"));
    }
    if vector.iter().all(|&v| v == 0.0) {
        return Err(format!("vector for id {id} is all-zero (cosine is undefined on it)"));
    }
    Ok(())
}

/// A streaming `.spqv` builder for stores whose dense data would not fit in
/// RAM: every [`put`](Self::put) appends the vector straight to the file's
/// data section and spills the id to a sidecar file, so build-phase memory is
/// O(1) regardless of store size. [`finalize`](Self::finalize) sorts the
/// spilled id→slot index (transiently 8 bytes per vector in RAM — a
/// `dim·4 / 8` reduction over the in-RAM builder, 192× at 384-d), appends it,
/// patches the header count, and opens the finished store.
///
/// The output is byte-identical to [`VectorStore::create`]`+put+finalize` for
/// the same `put` sequence and the same fingerprint (same version-2 format). One
/// contract difference, documented: a DUPLICATE id is detected at `finalize`
/// (the sort reveals it) rather than at `put`, since detecting it eagerly
/// would need the in-RAM id set this writer exists to avoid.
///
/// [OPUS-4.8] (sq-32i5) To bind the streaming store to its graph (so a stale store is caught —
/// see [`crate::fingerprint`]), build it with [`create_with_fingerprint`](Self::create_with_fingerprint);
/// a plain [`create`](Self::create) writes an unverifiable store (all-zero fingerprint).
///
/// ```no_run
/// use sparq_vectors::StreamingWriter;
/// let mut w = StreamingWriter::create("/tmp/big.spqv", 384).unwrap();
/// w.put(7, &[0.1; 384]).unwrap();
/// let store = w.finalize().unwrap(); // a normal, validated VectorStore
/// assert!(store.get(7).is_some());
/// ```
pub struct StreamingWriter {
    dim: usize,
    path: PathBuf,
    data: BufWriter<std::fs::File>,
    ids: BufWriter<std::fs::File>,
    ids_path: PathBuf,
    count: u64,
}

impl StreamingWriter {
    /// Starts a streaming store build at `path` (the sidecar id spill lives at
    /// `path` + `.ids-tmp` until [`finalize`](Self::finalize) removes it). The store is written
    /// WITHOUT a graph fingerprint (unverifiable — [`check_graph`](VectorStore::check_graph) errors);
    /// use [`create_with_fingerprint`](Self::create_with_fingerprint) to bind it to a graph.
    pub fn create<P: Into<PathBuf>>(path: P, dim: usize) -> Result<StreamingWriter, String> {
        Self::create_inner(path, dim, None)
    }

    /// [OPUS-4.8] (sq-32i5) Like [`create`](Self::create) but embeds `graph`'s fingerprint in the
    /// header, so [`check_graph`](VectorStore::check_graph) can reject the finished store if it is
    /// later queried against a different graph generation. Pass the SAME graph whose term ids the
    /// vectors are keyed by.
    pub fn create_with_fingerprint<P: Into<PathBuf>>(
        path: P,
        dim: usize,
        graph: &Graph,
    ) -> Result<StreamingWriter, String> {
        Self::create_inner(path, dim, Some(Fingerprint::of(graph)))
    }

    fn create_inner<P: Into<PathBuf>>(
        path: P,
        dim: usize,
        fingerprint: Option<Fingerprint>,
    ) -> Result<StreamingWriter, String> {
        if dim == 0 || dim > u32::MAX as usize {
            return Err(format!("invalid vector dimension {dim}"));
        }
        if cfg!(target_endian = "big") {
            return Err(".spqv is a little-endian format; big-endian targets are unsupported".into());
        }
        let path = path.into();
        let mut file = std::fs::File::create(&path)
            .map_err(|e| format!("create {}: {e}", path.display()))?;
        // Header with a zero count placeholder; finalize patches offset 12. The fingerprint
        // (offset 32..56) is known up front, so it is written here, not patched.
        let mut header = [0u8; HEADER_LEN];
        header[0..4].copy_from_slice(&SPQV_MAGIC);
        header[4..8].copy_from_slice(&SPQV_VERSION.to_le_bytes());
        header[8..12].copy_from_slice(&(dim as u32).to_le_bytes());
        if let Some(fp) = fingerprint {
            header[HEADER_LEN_V1..HEADER_LEN].copy_from_slice(&fp.to_bytes());
        }
        file.write_all(&header).map_err(|e| format!("write {}: {e}", path.display()))?;
        let ids_path = {
            let mut p = path.clone().into_os_string();
            p.push(".ids-tmp");
            PathBuf::from(p)
        };
        let ids_file = std::fs::File::create(&ids_path)
            .map_err(|e| format!("create {}: {e}", ids_path.display()))?;
        Ok(StreamingWriter {
            dim,
            path,
            data: BufWriter::new(file),
            ids: BufWriter::new(ids_file),
            ids_path,
            count: 0,
        })
    }

    /// Appends `vector` for term `id` to the data section. Validation matches
    /// [`VectorStore::put`] except that a duplicate `id` is reported at
    /// [`finalize`](Self::finalize), not here (see the type docs).
    pub fn put(&mut self, id: Id, vector: &[f32]) -> Result<(), String> {
        validate_vector(self.dim, id, vector)?;
        // f32 → LE bytes: a plain cast on little-endian targets (`create` rejects
        // big-endian).
        // SAFETY: f32 has no invalid bit patterns and align(f32) ≥ align(u8).
        let bytes =
            unsafe { std::slice::from_raw_parts(vector.as_ptr() as *const u8, vector.len() * 4) };
        self.data.write_all(bytes).map_err(|e| format!("write {}: {e}", self.path.display()))?;
        self.ids
            .write_all(&id.to_le_bytes())
            .map_err(|e| format!("write {}: {e}", self.ids_path.display()))?;
        self.count += 1;
        Ok(())
    }

    /// Number of vectors written so far.
    pub fn len(&self) -> usize {
        self.count as usize
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Sorts and appends the id→slot index, patches the header count, fsyncs,
    /// removes the sidecar, and opens the finished store (memory-mapped,
    /// fully validated — exactly as [`VectorStore::open`] would). On error the
    /// partial `.spqv` and sidecar are removed.
    pub fn finalize(self) -> Result<VectorStore, String> {
        let StreamingWriter { dim: _, path, data, ids, ids_path, count } = self;
        let cleanup = |msg: String| -> String {
            std::fs::remove_file(&path).ok();
            std::fs::remove_file(&ids_path).ok();
            msg
        };
        let result = (|| -> Result<VectorStore, String> {
            drop(ids); // flush the spill before reading it back
            let id_bytes = std::fs::read(&ids_path)
                .map_err(|e| format!("read {}: {e}", ids_path.display()))?;
            debug_assert_eq!(id_bytes.len() as u64, count * 4);
            // The transient 8-bytes-per-vector index: (id, slot), sorted by id.
            let mut index: Vec<(Id, u32)> = id_bytes
                .chunks_exact(4)
                .enumerate()
                .map(|(slot, b)| (u32::from_le_bytes(b.try_into().unwrap()), slot as u32))
                .collect();
            index.sort_unstable();
            if let Some(w) = index.windows(2).find(|w| w[0].0 == w[1].0) {
                return Err(format!("id {} has two vectors (duplicate put)", w[0].0));
            }
            let mut file = data
                .into_inner()
                .map_err(|e| format!("flush {}: {e}", path.display()))?;
            let mut index_bytes = Vec::with_capacity(index.len() * 8);
            for (id, slot) in index {
                index_bytes.extend_from_slice(&id.to_le_bytes());
                index_bytes.extend_from_slice(&slot.to_le_bytes());
            }
            file.write_all(&index_bytes)
                .and_then(|()| file.seek(SeekFrom::Start(12)).map(|_| ()))
                .and_then(|()| file.write_all(&count.to_le_bytes()))
                .and_then(|()| file.sync_all())
                .map_err(|e| format!("write {}: {e}", path.display()))?;
            drop(file);
            std::fs::remove_file(&ids_path).ok();
            VectorStore::open(&path)
        })();
        result.map_err(cleanup)
    }
}

// [OPUS-4.8] sq-hkud (epic sq-toze, gap MS-G4) — Kani bounded model-checking proof of the
// `.spqv` on-disk-format validator. This is the formal-methods complement to the Miri (UB),
// fuzz (libFuzzer corpus), oracle (deterministic corruption sweep) and ASan (sanitised
// corpus) coverage already in place. Those four EXECUTE the validator on SPECIFIC inputs
// (the corpus, plus whatever libFuzzer reaches); Kani PROVES the safety property over EVERY
// input up to a bounded buffer size — closing the residual "did the corpus miss a hostile
// header/index?" gap that motivated MS-G4. It is a complement, not a replacement: Kani's
// bound is small, so fuzz/oracle/ASan still carry the unbounded + UB coverage.
//
// WHY THIS VALIDATOR IS A GOOD KANI TARGET (the MS-G4 feasibility verdict):
//   * `VectorStore::open_from_bytes` is a PURE function of an owned `Vec<u8>` — no file I/O,
//     no `mmap`, no FFI, no syscalls (the constructs Kani cannot model). It runs the SAME
//     `open_validated` header/length/bounds logic as the mmap'd `open`, so proving it proves
//     the validator that the B5 hostile-on-disk-file boundary depends on.
//   * The state space is BOUNDABLE: the checked size arithmetic ties `count`/`dim` to
//     `map.len()`, so a small symbolic buffer bounds every loop and every allocation. A
//     buffer just past `HEADER_LEN` exercises every branch (magic, version 1/2/other,
//     `dim == 0`, the `checked_mul`/`checked_add` overflow rejections, the truncated-header
//     and exact-size checks, and at least one trip of the ascending-id / in-range-slot
//     index loop).
//
// UNTESTED-HERE: Kani is almost certainly NOT installed in the authoring environment, so
// this harness has NOT been run; it is written against Kani's documented API
// (`kani::any()`, `kani::assume()`, `#[kani::proof]`, `#[kani::unwind]`) and is VALIDATED ON
// THE FIRST CI RUN of the `kani` lane (.github/workflows/kani.yml). It is `#[cfg(kani)]`, so
// the normal `cargo build`/`clippy`/`test` never compile it and the crate is unaffected.
#[cfg(kani)]
mod kani_proofs {
    use super::*;

    /// The header is the smallest valid `.spqv` prefix; pick a symbolic buffer a little
    /// larger so the harness can also build a well-formed one-entry store (header + one
    /// `dim`-wide f32 vector + one 8-byte index entry) and reach the index-validation loop —
    /// not only the early-return header checks. Kept small so model checking terminates.
    const MAX_LEN: usize = HEADER_LEN + 32;

    /// PROPERTY: `open_from_bytes` (hence the shared `open_validated` validator) never
    /// panics, never reads out of bounds, and never triggers UB for ANY byte buffer up to
    /// `MAX_LEN` bytes — it always returns cleanly (`Ok` for a well-formed buffer, `Err` for
    /// every malformed one). Kani explores all such buffers symbolically. The success path
    /// of `open_from_bytes` does an aligned-copy + the full validation; the failure path is a
    /// plain `Err`. Neither may abort.
    #[kani::proof]
    // (sq-kycq5) [SONNET-4.6] Loop census for this harness: the setup loop `for b in
    // bytes.iter_mut()` runs up to `len` times where `len <= MAX_LEN`; the
    // `AlignedBytes::from_vec` aligned copy covers the same `len` bytes; the fingerprint
    // scan in `Fingerprint::from_bytes_opt` covers FINGERPRINT_LEN = 24 bytes; and the
    // index-validation loop `for i in 0..count` runs at most 2 times within MAX_LEN = 88
    // (a dim=1 count=2 store needs 56 + 8 + 16 = 80 bytes; count=3 needs 92 > 88).
    // All loops are bounded by MAX_LEN = HEADER_LEN + 32 = 56 + 32 = 88.
    // Therefore the required unwind bound is MAX_LEN + 1 = 89. The previous bound of 40
    // was LESS than MAX_LEN (40 < 88) and its comment "// > MAX_LEN" was factually wrong;
    // that bound fired an unwinding assertion in the nightly Kani lane (an INCOMPLETE proof,
    // not a counterexample). See the sq-gnvfc fix for the sibling harness for the same
    // pattern. NOTE: raising the bound may make this harness slower (more unwinding steps);
    // that is acceptable — the lane's per-harness timeout keeps it visible, and an honest
    // timeout beats a false incomplete proof.
    #[kani::unwind(89)] // MAX_LEN + 1 = 88 + 1; bounds every loop in the harness cone
    fn open_from_bytes_never_panics() {
        // A symbolic length in `0..=MAX_LEN`, then a symbolic buffer of that length.
        let len: usize = kani::any();
        kani::assume(len <= MAX_LEN);
        let mut bytes = vec![0u8; len];
        for b in bytes.iter_mut() {
            *b = kani::any();
        }
        // The property is simply: this call returns (it must not panic / OOB / UB). We do
        // not constrain the result — both `Ok` and `Err` are correct outcomes; the harness
        // proves the validator is TOTAL over the bounded input domain.
        let _ = VectorStore::open_from_bytes(bytes);
    }

    /// PROPERTY (focused): a buffer that already carries the magic + version-2 tag still
    /// never panics in the size-arithmetic + index-validation tail. Fixing the prefix
    /// shrinks the symbolic surface so Kani spends its budget on the arithmetic/loop logic
    /// (the part the corpus is least likely to have exhausted) rather than on the magic byte.
    #[kani::proof]
    // [OPUS-4.8] (sq-gnvfc) The harness buffer is HEADER_LEN + 16 = 72 concrete bytes. The
    // setup loop `bytes[8..].iter_mut()` runs 64 iterations, and every other per-element pass
    // over the full 72-byte buffer (aligned copy, the fingerprint scan, the index loop) needs
    // at most 72 steps; 73 > 72 bounds them all. The old bound of 40 < 64 fired an unwinding
    // assertion (an INCOMPLETE proof, not a counterexample) on the setup loop.
    #[kani::unwind(73)]
    fn open_validated_v2_tail_never_panics() {
        let mut bytes = vec![0u8; HEADER_LEN + 16];
        bytes[0..4].copy_from_slice(&SPQV_MAGIC);
        bytes[4..8].copy_from_slice(&2u32.to_le_bytes()); // version 2
        // dim and count are symbolic so the checked-overflow + size-mismatch + index loop
        // are all exercised; the fingerprint block and the trailing bytes stay symbolic too.
        for b in bytes[8..].iter_mut() {
            *b = kani::any();
        }
        let _ = VectorStore::open_from_bytes(bytes);
    }

    // DOMAIN-COVERAGE SELF-CHECK, part 1 of 2 (the sq-og8u8 anti-vacuity pattern). Lesson of
    // sq-sqtk2.1 (2026-07-04): a bound can SILENTLY prune the very inputs a harness means to
    // cover, so it passes VACUOUSLY while reporting nothing wrong. The two totality harnesses
    // above prove `open_from_bytes` NEVER panics over a bounded symbolic byte domain — but
    // that is worthless if the ACCEPT path (the aligned copy + the full header / version /
    // size / fingerprint validation) is unreachable within `MAX_LEN` and every in-domain
    // buffer is rejected at the first size check.
    //
    // The self-check is split in two because a `#[kani::proof]` form of the accept-path pin
    // does NOT terminate practically under Kani 0.67 (measured locally: > 20 min, dominated by
    // CBMC's memory-model churn in the `AlignedBytes` raw-pointer copy + the error-arm
    // `format!`/alloc machinery — the same cost class that keeps the two totality harnesses
    // above over the nightly lane's per-harness budget). Splitting loses nothing: the input is
    // fully CONCRETE, so native execution checks the identical property, and the buffer lies
    // INSIDE the symbolic harnesses' domain, so their (eventual) no-panic/no-UB verdict covers
    // it symbolically too.
    //
    //   part 1 (here, compile-time, evaluated whenever Kani builds this crate): the bounded
    //   domain is big enough to CONTAIN a well-formed store — `MAX_LEN >= HEADER_LEN`. A
    //   re-scope that shrank the fuzzed domain below the minimal valid store goes red at
    //   kani-build time.
    //
    //   part 2 (`domain_bounded_buffer_contains_an_accepted_store` in `fingerprint_tests`,
    //   every `cargo test` run): the minimal well-formed `HEADER_LEN`-byte version-2 buffer —
    //   which part 1 proves in-domain — actually validates `Ok`, so the totality harnesses'
    //   domain genuinely contains an ACCEPTED store and their proof is not vacuously
    //   reject-only. [OPUS-4.8] sq-og8u8
    const _DOMAIN_ADMITS_A_WELL_FORMED_STORE: () =
        assert!(MAX_LEN >= HEADER_LEN, "the fuzzed domain must contain a minimal valid store");

    // DOMAIN-COVERAGE SELF-CHECK for the v2-tail harness (sq-og8u8 anti-vacuity pattern): the
    // focused `open_validated_v2_tail_never_panics` harness fixes a 72-byte (HEADER_LEN + 16)
    // buffer with the magic + version-2 prefix. Its interesting tail is the index-validation
    // loop (`for i in 0..count`), which is only reached when count >= 1 — a bound or budget that
    // silently forced count = 0 would make the harness pass VACUOUSLY over the reject/skip path.
    // With version 2 (data_offset = HEADER_LEN) the body budget is HEADER_LEN + 16 - HEADER_LEN =
    // 16 bytes; a minimal 1-entry dim=1 store needs count*dim*4 + count*8 = 1*4 + 8 = 12 <= 16. ✓
    // Its runtime companion `v2_tail_domain_admits_an_indexed_store` (in `fingerprint_tests`)
    // proves a concrete such buffer validates `Ok`. [OPUS-4.8] sq-gnvfc
    const _V2_TAIL_DOMAIN_ADMITS_AN_INDEXED_STORE: () = assert!(
        HEADER_LEN + 16 >= HEADER_LEN + 4 + 8,
        "v2-tail harness domain must fit >= 1 index entry (dim=1 count=1: 4-byte vector + 8-byte slot)"
    );
}

#[cfg(test)]
mod fingerprint_tests {
    // [OPUS-4.8] (sq-32i5) End-to-end checked-open tests for the `.spqv` store: build against
    // graph A then (1) open/query against A → OK + correct neighbours, (2) open/query against a
    // DIFFERENT graph B → descriptive Err (NOT wrong results), (3) fingerprint round-trips through
    // serialize→deserialize, (4) a legacy version-1 file opens but reports as unverifiable.
    use super::*;
    use crate::ann::{nearest_term_exact, nearest_term_exact_checked};
    use oxrdf::{NamedNode, Term};
    use sparq_core::Graph;
    use std::sync::atomic::{AtomicU64, Ordering};

    static SEQ: AtomicU64 = AtomicU64::new(0);

    fn tmp(tag: &str) -> PathBuf {
        let n = SEQ.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("sparq_fp_{tag}_{}_{n}.spqv", std::process::id()))
    }

    fn graph(ttl: &str) -> Graph {
        Graph::load_str(ttl, "turtle").expect("load test turtle")
    }

    fn iri(s: &str) -> Term {
        Term::NamedNode(NamedNode::new(s).unwrap())
    }

    // A and B have DIFFERENT dictionaries and triple counts.
    const A: &str = r#"
        @prefix ex: <http://example.org/> .
        ex:alice ex:knows ex:bob .
        ex:bob ex:knows ex:carol .
    "#;
    const B: &str = r#"
        @prefix ex: <http://example.org/> .
        ex:dave ex:likes ex:eve .
        ex:eve ex:likes ex:frank .
        ex:frank ex:likes ex:gina .
    "#;

    /// Builds a 4-d store over graph `g`, giving `alice` a vector nearest to `bob` and far from
    /// `carol`, so a correct `nearest_term(alice)` ranks `bob` first.
    fn build_store(g: &Graph, path: &std::path::Path) -> VectorStore {
        let alice = g.id_of(&iri("http://example.org/alice")).unwrap();
        let bob = g.id_of(&iri("http://example.org/bob")).unwrap();
        let carol = g.id_of(&iri("http://example.org/carol")).unwrap();
        let mut s = VectorStore::create(path, 4).unwrap().with_fingerprint(g);
        s.put(alice, &[1.0, 0.0, 0.0, 0.0]).unwrap();
        s.put(bob, &[0.9, 0.1, 0.0, 0.0]).unwrap(); // close to alice
        s.put(carol, &[0.0, 0.0, 0.0, 1.0]).unwrap(); // far from alice
        s.finalize().unwrap();
        s
    }

    /// DOMAIN-COVERAGE SELF-CHECK, part 2 of 2 (the sq-og8u8 anti-vacuity pattern — part 1,
    /// the compile-time `MAX_LEN >= HEADER_LEN` binding, lives in `kani_proofs`; see the
    /// rationale there): the minimal well-formed version-2 buffer — exactly `HEADER_LEN`
    /// bytes: magic + version 2 + `dim = 1` + `count = 0` (no data, no index) + an all-zero
    /// fingerprint (decodes to `None`/unverifiable, still valid) — validates `Ok`. Because
    /// part 1 proves this buffer lies INSIDE the Kani totality harnesses' `MAX_LEN` domain,
    /// this test going green means those harnesses' domain genuinely CONTAINS an accepted
    /// store: their no-panic proof is not vacuously covering only the reject branches. If a
    /// format change makes every in-domain buffer rejectable, this goes red on every
    /// `cargo test` run. [OPUS-4.8] sq-og8u8
    #[test]
    fn domain_bounded_buffer_contains_an_accepted_store() {
        let mut bytes = vec![0u8; 56]; // HEADER_LEN — pinned numerically on purpose:
        assert_eq!(bytes.len(), HEADER_LEN, "minimal v2 store is exactly the header");
        bytes[0..4].copy_from_slice(&SPQV_MAGIC);
        bytes[4..8].copy_from_slice(&SPQV_VERSION.to_le_bytes());
        bytes[8..12].copy_from_slice(&1u32.to_le_bytes()); // dim = 1 (dim == 0 is rejected)
        // count = 0 (bytes 12..20 stay zero) ⇒ no data + no index; reserved + fingerprint
        // stay all-zero (a valid v2 store finalized without `with_fingerprint`).
        let store = VectorStore::open_from_bytes(bytes)
            .expect("a minimal well-formed v2 store must validate — else the accept path is vacuous");
        assert_eq!(store.dim(), 1);
        assert!(store.fingerprint().is_none(), "all-zero fingerprint decodes to None");
    }

    /// DOMAIN-COVERAGE SELF-CHECK for the v2-tail harness (sq-og8u8 anti-vacuity pattern,
    /// sq-gnvfc — companion to the compile-time `_V2_TAIL_DOMAIN_ADMITS_AN_INDEXED_STORE`
    /// const in `kani_proofs`). Proves a concrete 72-byte buffer in the
    /// `open_validated_v2_tail_never_panics` harness's domain (the magic + version-2 prefix,
    /// count = 1, dim = 2) validates `Ok` — so the index-validation loop `for i in 0..count`
    /// runs exactly once. This confirms the focused harness genuinely covers the indexed-store
    /// path, not only the size-mismatch / bad-index reject branches. [OPUS-4.8] sq-gnvfc
    #[test]
    fn v2_tail_domain_admits_an_indexed_store() {
        // Buffer layout for a valid v2 store with dim = 2, count = 1:
        //   [0..4]   SPQV_MAGIC
        //   [4..8]   version = 2 (LE u32)
        //   [8..12]  dim = 2 (LE u32)
        //   [12..20] count = 1 (LE u64)
        //   [20..32] reserved (zeros) — fills the rest of HEADER_LEN_V1 (32 bytes)
        //   [32..56] fingerprint block (all-zero = no fingerprint, still a valid v2) — HEADER_LEN
        //   [56..64] vector data: one dim=2 f32 vector (2 * 4 = 8 bytes)
        //   [64..72] index: one entry — id = 1 (u32 LE) + slot = 0 (u32 LE) = 8 bytes
        //   total = 72 bytes = HEADER_LEN + 16
        let mut bytes = vec![0u8; HEADER_LEN + 16];
        assert_eq!(bytes.len(), 72, "buffer must be HEADER_LEN + 16 = 72 bytes");
        bytes[0..4].copy_from_slice(&SPQV_MAGIC);
        bytes[4..8].copy_from_slice(&2u32.to_le_bytes()); // version 2
        bytes[8..12].copy_from_slice(&2u32.to_le_bytes()); // dim = 2
        bytes[12..20].copy_from_slice(&1u64.to_le_bytes()); // count = 1
        // fingerprint block [32..56] stays all-zero (valid: no fingerprint).
        // vector data [56..64]: f32 values [1.0, 0.0] in LE.
        bytes[56..60].copy_from_slice(&1.0f32.to_le_bytes());
        bytes[60..64].copy_from_slice(&0.0f32.to_le_bytes());
        // index [64..72]: entry 0 — id = 1 (u32 LE), slot = 0 (u32 LE).
        bytes[64..68].copy_from_slice(&1u32.to_le_bytes()); // id = 1
        bytes[68..72].copy_from_slice(&0u32.to_le_bytes()); // slot = 0
        let store = VectorStore::open_from_bytes(bytes).expect(
            "72-byte v2 store with dim=2 count=1 must validate — the index loop must be reachable \
             in the focused harness domain",
        );
        assert_eq!(store.dim(), 2);
        assert!(store.fingerprint().is_none(), "all-zero fingerprint block decodes to None");
    }

    #[test]
    fn open_against_build_graph_ok_and_correct() {
        let ga = graph(A);
        let path = tmp("ok");
        build_store(&ga, &path);
        let store = VectorStore::open(&path).unwrap();
        // (1) Checked open against the SAME graph → OK.
        assert!(store.check_graph(&ga).is_ok());
        // ...and the neighbours are correct: bob is alice's nearest.
        let got = nearest_term_exact_checked(&store, &ga, &iri("http://example.org/alice"), 1)
            .expect("checked query against the build graph must succeed");
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].0, iri("http://example.org/bob"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn query_against_different_graph_errs_not_wrong() {
        let ga = graph(A);
        let path = tmp("mismatch");
        build_store(&ga, &path);
        let store = VectorStore::open(&path).unwrap();
        let gb = graph(B); // different dict ids AND triple count
        // (2) Checked open/query against a DIFFERENT graph → descriptive Err, not silent wrong data.
        let err = store.check_graph(&gb).expect_err("a mismatched graph must be rejected");
        assert!(err.contains("fingerprint mismatch"), "err was: {err}");
        let qerr = nearest_term_exact_checked(&store, &gb, &iri("http://example.org/dave"), 1)
            .expect_err("a checked query against a mismatched graph must error");
        assert!(qerr.contains("wrong results"), "err was: {qerr}");
        // The UNCHECKED path is the foot-gun this guards: it does NOT error. We only assert the
        // checked path refuses; the unchecked one is documented as the unsafe form.
        let _silent = nearest_term_exact(&store, &gb, &iri("http://example.org/dave"), 1);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn fingerprint_survives_round_trip() {
        let ga = graph(A);
        let path = tmp("rt");
        let built = build_store(&ga, &path);
        let want = built.fingerprint().expect("built store has a fingerprint");
        // (3) Reopen from disk: the stored fingerprint must equal the one set at build, and the
        // live graph's recomputed fingerprint.
        let reopened = VectorStore::open(&path).unwrap();
        assert_eq!(reopened.fingerprint(), Some(want));
        assert_eq!(reopened.fingerprint(), Some(Fingerprint::of(&ga)));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn store_without_fingerprint_is_unverifiable() {
        // A store finalized WITHOUT `with_fingerprint` writes an all-zero block; check_graph must
        // not silently certify it against any non-empty graph.
        let ga = graph(A);
        let alice = ga.id_of(&iri("http://example.org/alice")).unwrap();
        let path = tmp("nofp");
        let mut s = VectorStore::create(&path, 4).unwrap();
        s.put(alice, &[1.0, 0.0, 0.0, 0.0]).unwrap();
        s.finalize().unwrap();
        let store = VectorStore::open(&path).unwrap();
        // [OPUS-4.8] (sq-32i5) An all-zero fingerprint block decodes to `None`, so check_graph
        // reports it as unverifiable ("carries no graph fingerprint") rather than a spurious
        // "DIFFERENT graph" mismatch — and never silently certifies it.
        assert_eq!(store.fingerprint(), None);
        let err = store
            .check_graph(&ga)
            .expect_err("unbound store must not be certified");
        assert!(err.contains("carries no graph fingerprint"), "err: {err}");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn legacy_v1_file_opens_but_is_unverifiable() {
        // (4) Back-compat: hand-craft a version-1 `.spqv` (32-byte header, no fingerprint block)
        // and confirm it still OPENS and READS correctly, but `check_graph` reports it as
        // unverifiable (the documented legacy path) rather than silently passing.
        let path = tmp("legacy");
        let dim = 4usize;
        let vectors: [(Id, [f32; 4]); 2] = [(7, [1.0, 0.0, 0.0, 0.0]), (3, [0.0, 1.0, 0.0, 0.0])];
        let count = vectors.len();
        // ids must be ascending in the index; slots follow insertion order here.
        let mut header = [0u8; HEADER_LEN_V1];
        header[0..4].copy_from_slice(&SPQV_MAGIC);
        header[4..8].copy_from_slice(&1u32.to_le_bytes()); // version 1
        header[8..12].copy_from_slice(&(dim as u32).to_le_bytes());
        header[12..20].copy_from_slice(&(count as u64).to_le_bytes());
        let mut bytes = header.to_vec();
        for (_, v) in &vectors {
            for x in v {
                bytes.extend_from_slice(&x.to_le_bytes());
            }
        }
        // index: (id, slot) sorted by id ascending → (3, slot1), (7, slot0)
        let mut idx: Vec<(Id, u32)> =
            vectors.iter().enumerate().map(|(slot, (id, _))| (*id, slot as u32)).collect();
        idx.sort_unstable();
        for (id, slot) in idx {
            bytes.extend_from_slice(&id.to_le_bytes());
            bytes.extend_from_slice(&slot.to_le_bytes());
        }
        std::fs::write(&path, &bytes).unwrap();

        let store = VectorStore::open(&path).expect("a legacy v1 file must still open");
        assert!(store.fingerprint().is_none(), "v1 file carries no fingerprint");
        // Reads work against the v1 (offset-32) layout.
        assert_eq!(store.get(7), Some(&[1.0, 0.0, 0.0, 0.0][..]));
        assert_eq!(store.get(3), Some(&[0.0, 1.0, 0.0, 0.0][..]));
        // ...but it cannot be verified against a graph.
        let err = store.check_graph(&graph(A)).expect_err("a v1 file must not certify a graph");
        assert!(err.contains("predates graph-fingerprinting") || err.contains("legacy"), "err: {err}");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn streaming_writer_with_fingerprint_round_trips() {
        let ga = graph(A);
        let alice = ga.id_of(&iri("http://example.org/alice")).unwrap();
        let path = tmp("stream");
        let mut w = StreamingWriter::create_with_fingerprint(&path, 4, &ga).unwrap();
        w.put(alice, &[1.0, 0.0, 0.0, 0.0]).unwrap();
        let store = w.finalize().unwrap();
        assert_eq!(store.fingerprint(), Some(Fingerprint::of(&ga)));
        assert!(store.check_graph(&ga).is_ok());
        assert!(store.check_graph(&graph(B)).is_err());
        std::fs::remove_file(&path).ok();
    }
}
