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
    pub fn get(&self, id: Id) -> Option<&[f32]> {
        match &self.backing {
            Backing::Build { data, slots, .. } => {
                let slot = *slots.get(&id)? as usize;
                Some(&data[slot * self.dim..(slot + 1) * self.dim])
            }
            Backing::Map(map) => {
                let count = self.len();
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

    /// Number of stored vectors.
    pub fn len(&self) -> usize {
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
    /// ([`crate::ann::nearest_term_exact_checked`], [`DiskAnnIndex::nearest_term_checked`]) run it.
    pub fn check_graph(&self, graph: &Graph) -> fingerprint::CheckResult {
        let origin = if self.path.as_os_str().is_empty() {
            "<bytes>".to_string()
        } else {
            self.path.display().to_string()
        };
        fingerprint::check_against(self.fingerprint, graph, fingerprint::Artifact::Store, &origin)
    }

    /// Iterates all `(id, vector)` pairs in insertion-slot order (the dense data
    /// order) — what ANN index construction consumes.
    pub fn iter(&self) -> impl Iterator<Item = (Id, &[f32])> + '_ {
        let count = self.len();
        (0..count).map(move |slot| match &self.backing {
            Backing::Build { data, ids, .. } => {
                (ids[slot], &data[slot * self.dim..(slot + 1) * self.dim])
            }
            Backing::Map(map) => (self.slot_id(map, slot), self.slot_vector(map, slot)),
        })
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
