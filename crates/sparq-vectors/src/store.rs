//! The `.spqv` on-disk vector store: one f32 embedding per dictionary term id, in a
//! single flat memory-mapped file.
//!
//! # File format (version 1, little-endian)
//!
//! ```text
//! offset 0   magic        b"SPQV"                          4 bytes
//! offset 4   version      u32 = 1                          4 bytes
//! offset 8   dim          u32                              4 bytes
//! offset 12  count        u64                              8 bytes
//! offset 20  reserved     zero padding                     12 bytes
//! offset 32  data         [count × dim] f32, dense         count·dim·4 bytes
//! offset 32+count·dim·4   id→slot index                    count·8 bytes
//!            (id: u32, slot: u32) pairs sorted by id ascending
//! ```
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

use memmap2::Mmap;
use rustc_hash::FxHashMap;
use sparq_core::dict::Id;
use std::io::{BufWriter, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

/// First four bytes of every `.spqv` file.
pub const SPQV_MAGIC: [u8; 4] = *b"SPQV";
/// Current format version.
pub const SPQV_VERSION: u32 = 1;
const HEADER_LEN: usize = 32;

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
            inverse: std::sync::OnceLock::new(),
        })
    }

    /// Memory-maps an existing `.spqv` file read-only. Cheap: only the 32-byte header
    /// and the trailing `count·8`-byte id→slot index are read eagerly (the index is
    /// validated so no later read can panic on a corrupt file); the vector data — the
    /// overwhelming bulk of the file — is paged in by the OS on access.
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
        if map.len() < HEADER_LEN {
            return Err(format!("{origin}: truncated header"));
        }
        if map[0..4] != SPQV_MAGIC {
            return Err(format!("{origin}: not a .spqv file (bad magic)"));
        }
        let version = u32::from_le_bytes(map[4..8].try_into().unwrap());
        if version != SPQV_VERSION {
            return Err(format!("{origin}: unsupported .spqv version {version}"));
        }
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
            .and_then(|body| body.checked_add(HEADER_LEN))
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
            let index = &map[HEADER_LEN + count * dim * 4..];
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
        Ok(VectorStore { dim, path, backing: Backing::Map(map), inverse: std::sync::OnceLock::new() })
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
                let index = &map[HEADER_LEN + count * self.dim * 4..];
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
            let index = &map[HEADER_LEN + count * self.dim * 4..];
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
        let start = HEADER_LEN + slot * self.dim * 4;
        let bytes = &map[start..start + self.dim * 4];
        debug_assert_eq!(bytes.as_ptr() as usize % std::mem::align_of::<f32>(), 0);
        // SAFETY: the backing base is 4-byte aligned — a memory map is page-aligned, and the
        // owned path uses `AlignedBytes` (u32-backed, alignment 4) precisely so this cast is
        // aligned ([OPUS-4.8] review 1874; a raw Vec<u8> is alignment 1 and would be UB here).
        // `start` is a multiple of 4 (HEADER_LEN=32 + slot·dim·4), so the pointer stays
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
/// the same `put` sequence (same version-1 format — no format change). One
/// contract difference, documented: a DUPLICATE id is detected at `finalize`
/// (the sort reveals it) rather than at `put`, since detecting it eagerly
/// would need the in-RAM id set this writer exists to avoid.
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
    /// `path` + `.ids-tmp` until [`finalize`](Self::finalize) removes it).
    pub fn create<P: Into<PathBuf>>(path: P, dim: usize) -> Result<StreamingWriter, String> {
        if dim == 0 || dim > u32::MAX as usize {
            return Err(format!("invalid vector dimension {dim}"));
        }
        if cfg!(target_endian = "big") {
            return Err(".spqv is a little-endian format; big-endian targets are unsupported".into());
        }
        let path = path.into();
        let mut file = std::fs::File::create(&path)
            .map_err(|e| format!("create {}: {e}", path.display()))?;
        // Header with a zero count placeholder; finalize patches offset 12.
        let mut header = [0u8; HEADER_LEN];
        header[0..4].copy_from_slice(&SPQV_MAGIC);
        header[4..8].copy_from_slice(&SPQV_VERSION.to_le_bytes());
        header[8..12].copy_from_slice(&(dim as u32).to_le_bytes());
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
