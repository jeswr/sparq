//! [OPUS-5] (issue #3699) The **shared read backing** for a memory-mappable on-disk artifact:
//! a memory map on native targets, an f32-aligned owned buffer everywhere else.
//!
//! Extracted verbatim from `sparq-vectors::store` (where it was `pub(crate)` and shared by the
//! `.spqv` vector store and the `.spqg` graph). It lives here because the `.spqg` reader in
//! [`crate::graph`] goes through it, and `sparq-vectors` now imports it back from this crate so
//! there is exactly ONE mmap site and ONE aligned-owned-bytes implementation in the tree.
//!
//! Every read path derefs to `[u8]`, so the mapped and owned paths are byte-identical downstream
//! and a single validator covers both.

use std::path::Path;

#[cfg(not(target_arch = "wasm32"))]
use memmap2::Mmap;

/// A 4-byte-aligned owned byte buffer. A plain `Vec<u8>` has alignment 1, so its base pointer may
/// land on an odd address; readers cast `&[u8]` slices of the buffer to `&[f32]` via
/// `from_raw_parts`, which is UNDEFINED BEHAVIOR on an unaligned pointer. Backing the owned bytes
/// with a `Vec<u32>` (alignment 4) guarantees the base — and therefore every 4-multiple offset
/// into it — is f32-aligned.
pub struct AlignedBytes {
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
        let dst = unsafe { std::slice::from_raw_parts_mut(ab.words.as_mut_ptr() as *mut u8, len) };
        dst.copy_from_slice(&bytes);
        ab
    }

    fn as_bytes(&self) -> &[u8] {
        // SAFETY: `words` is u32-aligned and holds ≥ `len` initialized bytes (the copy above);
        // f32/u8 reads of this region are in bounds. The base is 4-byte aligned by construction.
        unsafe { std::slice::from_raw_parts(self.words.as_ptr() as *const u8, self.len) }
    }
}

/// Read-phase backing bytes: a memory map ([`open_backing`]) or an owned buffer (the
/// filesystem-less / wasm path). Both deref to `[u8]`; every read path is shared.
pub enum Bytes {
    /// Native-only: memmap2 is target-gated out of wasm32 builds, where the owned-bytes backing
    /// serves every read instead.
    #[cfg(not(target_arch = "wasm32"))]
    Map(Mmap),
    /// f32-aligned owned bytes (see [`AlignedBytes`]) so a reader's f32 cast is always aligned —
    /// a plain `Vec<u8>` is alignment 1 and would risk UB.
    Owned(AlignedBytes),
}

impl Bytes {
    /// Copies `bytes` into the f32-aligned owned backing — the shared `open_from_bytes` path.
    pub fn owned(bytes: Vec<u8>) -> Bytes {
        Bytes::Owned(AlignedBytes::from_vec(bytes))
    }
}

/// Opens the read backing for an on-disk artifact: a read-only memory map on native targets, a
/// buffered `std::fs::read` into the f32-aligned owned backing on wasm32 (memmap2 is target-gated
/// out of wasm builds; a wasm target WITH a filesystem, e.g. WASI, reads the whole file — on
/// `wasm32-unknown-unknown` the read fails with a clean I/O error, and the `open_from_bytes`
/// entry points are the filesystem-less paths). Validation downstream is identical for both.
pub fn open_backing(path: &Path) -> Result<Bytes, String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let file = std::fs::File::open(path).map_err(|e| format!("open {}: {e}", path.display()))?;
        // SAFETY: read-only map of a regular file; we treat concurrent external modification of
        // the file as out of contract (the same stance the rest of the tree's mmap'd artifacts
        // take).
        let map = unsafe { Mmap::map(&file) }.map_err(|e| format!("mmap {}: {e}", path.display()))?;
        Ok(Bytes::Map(map))
    }
    #[cfg(target_arch = "wasm32")]
    {
        let bytes = std::fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
        Ok(Bytes::owned(bytes))
    }
}

impl std::ops::Deref for Bytes {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            Bytes::Map(m) => m,
            Bytes::Owned(v) => v.as_bytes(),
        }
    }
}
