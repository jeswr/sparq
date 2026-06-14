//! Block-compressed permutation index: a sorted `[[Id;3]]` permutation stored as
//! fixed-count blocks of lexicographic-delta + LEB128-varint rows, with a sparse
//! directory of (first-triple, byte-offset) for random access. Cuts the index from
//! 12 B/triple to ~4-6 B/triple (measured −55% to −69% on synthetic + real Wikidata)
//! while keeping pattern scans random-accessible: binary-search the directory, decode
//! only the blocks the pattern's key-range touches.
//!
//! This is the storage mode for the memory-bound paths (browser / out-of-core); the
//! native in-memory store keeps raw `[[Id;3]]` (no decode cost). The encoding is the
//! one validated by `sparq-cli probe-compress`.

use crate::dict::Id;

/// Rows per block. A scan decodes whole blocks, so smaller blocks mean less decode
/// waste per probe but a larger directory; 128 keeps the directory at ~0.16 B/triple.
pub const BLOCK: usize = 128;

/// Magic prefix of the COMPRESSED on-disk permutation file format (see
/// [`CompressedPerm::write_to`]). Distinguishes a compressed `perm{i}.bin` from the raw
/// little-endian `[u32;3]` format on open: a raw file starts with the FIRST (i.e.
/// smallest) sorted row, whose leading id would have to be 0x43515053 ≈ 1.13e9 to
/// collide — ids are assigned densely from 1, so the minimum row never gets close.
#[cfg(feature = "mmap")]
pub const FILE_MAGIC: [u8; 8] = *b"SPQCPRM1";

/// Appends `x` to `out` as an unsigned LEB128 varint.
#[inline]
fn put_varint(out: &mut Vec<u8>, mut x: u64) {
    while x >= 0x80 {
        out.push((x as u8) | 0x80);
        x >>= 7;
    }
    out.push(x as u8);
}

/// Reads an unsigned LEB128 varint from `buf` at `*pos`, advancing `*pos`.
///
/// This is the TRUSTED hot-path reader: it indexes `buf` unchecked and is sound ONLY on a
/// block stream already proven well-formed. The in-memory encoder produces valid streams,
/// and a memory-mapped stream is fully validated once at open by
/// [`CompressedPerm::from_mmap`] (via [`get_varint_checked`] / [`validate_block`]) before
/// any scan reaches here — so an attacker-controlled `.spq` cannot drive this OOB. [OPUS-4.8]
#[inline]
fn get_varint(buf: &[u8], pos: &mut usize) -> u64 {
    let mut x = 0u64;
    let mut shift = 0;
    loop {
        let b = buf[*pos];
        *pos += 1;
        x |= ((b & 0x7f) as u64) << shift;
        if b & 0x80 == 0 {
            return x;
        }
        shift += 7;
    }
}

/// [OPUS-4.8] sq-ed2i — bounds-/overflow-CHECKED varint reader for the UNTRUSTED mmap path.
/// Returns `None` if the buffer ends mid-varint or the encoding exceeds 64 bits (a hostile
/// file could otherwise make `get_varint` read past the mapping or shift past `u64`). Used
/// only by the one-time open-time validation, never on the scan hot path.
#[cfg(feature = "mmap")]
#[inline]
fn get_varint_checked(buf: &[u8], pos: &mut usize) -> Option<u64> {
    let mut x = 0u64;
    let mut shift = 0u32;
    loop {
        let b = *buf.get(*pos)?;
        *pos += 1;
        x |= ((b & 0x7f) as u64).checked_shl(shift)?;
        if b & 0x80 == 0 {
            return Some(x);
        }
        shift += 7;
        if shift >= 64 {
            return None; // varint longer than a u64 can hold → malformed
        }
    }
}

/// The encoded block stream: built in RAM, or borrowed from a memory-mapped compressed
/// index file (the lazy out-of-core mode — the OS pages in only the blocks scans touch,
/// and the stream never counts against the heap).
enum Blocks {
    Owned(Vec<u8>),
    #[cfg(feature = "mmap")]
    Mapped { map: memmap2::Mmap, off: usize },
}

impl Blocks {
    #[inline]
    fn bytes(&self) -> &[u8] {
        match self {
            Blocks::Owned(v) => v,
            #[cfg(feature = "mmap")]
            Blocks::Mapped { map, off } => &map[*off..],
        }
    }
}

/// A block-compressed, random-accessible sorted permutation.
pub struct CompressedPerm {
    /// One entry per block: (its first triple, its byte offset into `blocks`).
    dir: Vec<([Id; 3], u32)>,
    /// The concatenated encoded blocks.
    blocks: Blocks,
    len: usize,
}

impl CompressedPerm {
    /// Encodes a sorted permutation (rows already in this permutation's column order).
    pub fn encode(rows: &[[Id; 3]]) -> Self {
        let mut dir = Vec::with_capacity(rows.len() / BLOCK + 1);
        let mut blocks = Vec::with_capacity(rows.len() * 6);
        for chunk in rows.chunks(BLOCK) {
            dir.push((chunk[0], blocks.len() as u32));
            put_varint(&mut blocks, chunk.len() as u64);
            // First row absolute.
            put_varint(&mut blocks, chunk[0][0] as u64);
            put_varint(&mut blocks, chunk[0][1] as u64);
            put_varint(&mut blocks, chunk[0][2] as u64);
            // Remaining rows: lexicographic delta vs the previous row.
            for w in chunk.windows(2) {
                let (p, r) = (w[0], w[1]);
                let d0 = r[0] - p[0];
                put_varint(&mut blocks, d0 as u64);
                if d0 == 0 {
                    let d1 = r[1] - p[1];
                    put_varint(&mut blocks, d1 as u64);
                    if d1 == 0 {
                        put_varint(&mut blocks, (r[2] - p[2]) as u64); // strictly increasing
                    } else {
                        put_varint(&mut blocks, r[2] as u64); // col2 resets → absolute
                    }
                } else {
                    put_varint(&mut blocks, r[1] as u64); // cols 1,2 reset → absolute
                    put_varint(&mut blocks, r[2] as u64);
                }
            }
        }
        CompressedPerm { dir, blocks: Blocks::Owned(blocks), len: rows.len() }
    }

    /// Writes the COMPRESSED on-disk permutation format (auto-detected on open by its
    /// [`FILE_MAGIC`]; raw files keep working). Layout, all little-endian:
    ///
    /// ```text
    /// magic[8] | len u64 | n_blocks u64 | blocks_len u64
    /// directory: n_blocks × { first_row [u32;3], byte_off u32 }
    /// blocks:    blocks_len bytes (the delta+varint block stream of `encode`)
    /// ```
    ///
    /// The byte stream is exactly the in-memory encoding, so [`from_mmap`](Self::from_mmap)
    /// can serve scans straight off the mapped file with no transcode.
    #[cfg(feature = "mmap")]
    pub fn write_to<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        let blocks = self.blocks.bytes();
        w.write_all(&FILE_MAGIC)?;
        w.write_all(&(self.len as u64).to_le_bytes())?;
        w.write_all(&(self.dir.len() as u64).to_le_bytes())?;
        w.write_all(&(blocks.len() as u64).to_le_bytes())?;
        for &(key, off) in &self.dir {
            for c in key {
                w.write_all(&c.to_le_bytes())?;
            }
            w.write_all(&off.to_le_bytes())?;
        }
        w.write_all(blocks)
    }

    /// Opens a compressed permutation from a memory-mapped file written by
    /// [`write_to`](Self::write_to). Only the sparse directory is copied to the heap
    /// (~0.13 B/triple); the block stream stays on disk, decoded block-wise per scan —
    /// the lazy out-of-core mode.
    #[cfg(feature = "mmap")]
    pub fn from_mmap(map: memmap2::Mmap) -> std::io::Result<Self> {
        let bad = |m: &str| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("compressed perm: {m}"));
        let b: &[u8] = &map;
        if b.len() < 32 || b[..8] != FILE_MAGIC {
            return Err(bad("missing FILE_MAGIC header"));
        }
        let rd64 = |i: usize| u64::from_le_bytes(b[i..i + 8].try_into().unwrap());
        // [OPUS-4.8] sq-ed2i: the three header counts are attacker-controlled. Compute the
        // layout with CHECKED arithmetic — the original `32 + n_blocks*16` / `dir_end +
        // blocks_len` were plain `usize` ops that wrap on a hostile `n_blocks`/`blocks_len`
        // near `u64::MAX`, so the length-equality check could pass against an undersized
        // file and the directory loop then read OOB. On any overflow / length mismatch we
        // return a clean error instead.
        let (len, n_blocks, blocks_len) = (rd64(8), rd64(16), rd64(24));
        // Reject sizes that cannot fit this file before any allocation: each directory entry
        // is 16 bytes and lives between the 32-byte header and the block stream, so
        // `n_blocks` is bounded by the actual file length. This also caps the
        // `Vec::with_capacity` below, so a hostile `n_blocks` cannot trigger a huge alloc.
        let n_blocks = usize::try_from(n_blocks).map_err(|_| bad("n_blocks exceeds usize"))?;
        let blocks_len = usize::try_from(blocks_len).map_err(|_| bad("blocks_len exceeds usize"))?;
        let dir_bytes = n_blocks.checked_mul(16).ok_or_else(|| bad("directory size overflows"))?;
        let dir_end = dir_bytes.checked_add(32).ok_or_else(|| bad("directory end overflows"))?;
        let total = dir_end.checked_add(blocks_len).ok_or_else(|| bad("file size overflows"))?;
        if b.len() != total {
            return Err(bad("length does not match header"));
        }
        let len = usize::try_from(len).map_err(|_| bad("len exceeds usize"))?;
        let mut dir = Vec::with_capacity(n_blocks);
        for e in (32..dir_end).step_by(16) {
            let rd32 = |i: usize| u32::from_le_bytes(b[i..i + 4].try_into().unwrap());
            // [OPUS-4.8] sq-ed2i: every block byte-offset must point at a valid start inside
            // the block stream (a non-empty stream has its first block at 0); otherwise
            // `decode_block_at` would index past the mapping. `validate_blocks` below then
            // proves the block at each offset decodes fully in-bounds.
            let off = rd32(e + 12);
            if off as usize >= blocks_len {
                return Err(bad("a directory block offset is past the end of the block stream"));
            }
            dir.push(([rd32(e), rd32(e + 4), rd32(e + 8)], off));
        }
        let perm = CompressedPerm { dir, blocks: Blocks::Mapped { map, off: dir_end }, len };
        // [OPUS-4.8] sq-ed2i: decode-validate every block ONCE, here, so the (unchecked,
        // hot-path) `get_varint`/`decode_block_at` are provably in-bounds on later scans.
        // A bounded one-pass walk: each block's varints must stay within the stream and the
        // decoded row count across all blocks must equal `len`. Any malformation → Err, no
        // panic / OOB / unbounded loop. Peak extra RAM is O(1) (we count, not collect).
        perm.validate_blocks().map_err(|m| bad(&m))?;
        Ok(perm)
    }

    /// [OPUS-4.8] sq-ed2i — one-time structural validation of the mapped block stream.
    /// Walks every block with the bounds-checked varint reader, proving each block decodes
    /// fully within the stream and that the total decoded row count matches the header
    /// `len`. Returns a description on the first malformation. After this passes, the
    /// unchecked hot-path decoders cannot read out of bounds on a corrupt file.
    #[cfg(feature = "mmap")]
    fn validate_blocks(&self) -> Result<(), String> {
        let buf = self.blocks.bytes();
        let mut total_rows: usize = 0;
        for &(_, off) in &self.dir {
            let mut pos = off as usize;
            self.validate_one_block(buf, &mut pos).map(|rows| total_rows += rows).ok_or_else(|| {
                "a compressed block is truncated or has a malformed varint".to_string()
            })?;
        }
        if total_rows != self.len {
            return Err(format!("decoded {total_rows} rows but header declares {}", self.len));
        }
        Ok(())
    }

    /// Validates a single block at `*pos` (advancing it past the block), returning its row
    /// count, or `None` on any out-of-bounds / malformed varint. Mirrors the shape of
    /// [`decode_block_at`] but reads through [`get_varint_checked`] and discards values.
    #[cfg(feature = "mmap")]
    fn validate_one_block(&self, buf: &[u8], pos: &mut usize) -> Option<usize> {
        let count = get_varint_checked(buf, pos)? as usize;
        if count == 0 || count > BLOCK {
            return None; // a block holds 1..=BLOCK rows by construction
        }
        // First row: three absolute varints.
        for _ in 0..3 {
            get_varint_checked(buf, pos)?;
        }
        for _ in 1..count {
            let d0 = get_varint_checked(buf, pos)?;
            if d0 == 0 {
                let d1 = get_varint_checked(buf, pos)?;
                get_varint_checked(buf, pos)?; // col2 (delta or absolute)
                let _ = d1;
            } else {
                get_varint_checked(buf, pos)?; // col1 absolute
                get_varint_checked(buf, pos)?; // col2 absolute
            }
        }
        Some(count)
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Resident bytes (directory + block stream). A memory-mapped block stream counts 0:
    /// its resident pages are OS page cache, not process heap (same as a raw mmap'd perm).
    pub fn heap_bytes(&self) -> usize {
        let stream = match &self.blocks {
            Blocks::Owned(v) => v.capacity(),
            #[cfg(feature = "mmap")]
            Blocks::Mapped { .. } => 0,
        };
        self.dir.capacity() * std::mem::size_of::<([Id; 3], u32)>() + stream
    }

    /// Decodes the block starting at byte `off` into `out` (appending).
    #[inline]
    fn decode_block_at(&self, off: usize, out: &mut Vec<[Id; 3]>) {
        let buf = self.blocks.bytes();
        let mut pos = off;
        let count = get_varint(buf, &mut pos) as usize;
        let mut prev = [
            get_varint(buf, &mut pos) as Id,
            get_varint(buf, &mut pos) as Id,
            get_varint(buf, &mut pos) as Id,
        ];
        out.push(prev);
        for _ in 1..count {
            let d0 = get_varint(buf, &mut pos) as Id;
            // [OPUS-4.8] sq-ed2i: `from_mmap` proves every varint here is in-bounds, but a
            // TAMPERED (checksum-less) block can still hold a delta that makes `prev + d`
            // exceed `u32::MAX`. `+` panics on that overflow in debug; `wrapping_add` cannot
            // panic and matches release wrapping — the resulting (wrong) id is then handled
            // safely by the bounds-checked `Dict::record` (sq-ky2a), so a corrupt perm is
            // wrong-but-safe (the documented trusted-store boundary), never a panic / UB.
            let cur = if d0 == 0 {
                let d1 = get_varint(buf, &mut pos) as Id;
                if d1 == 0 {
                    [prev[0], prev[1], prev[2].wrapping_add(get_varint(buf, &mut pos) as Id)]
                } else {
                    [prev[0], prev[1].wrapping_add(d1), get_varint(buf, &mut pos) as Id]
                }
            } else {
                [prev[0].wrapping_add(d0), get_varint(buf, &mut pos) as Id, get_varint(buf, &mut pos) as Id]
            };
            out.push(cur);
            prev = cur;
        }
    }

    /// Decodes the whole permutation (for stats / full iteration).
    pub fn decode_all(&self) -> Vec<[Id; 3]> {
        let mut out = Vec::with_capacity(self.len);
        for &(_, off) in &self.dir {
            self.decode_block_at(off as usize, &mut out);
        }
        out
    }

    /// Exact count of rows in `[lo, hi]` without materializing the range: decode at most
    /// the two boundary blocks (the interior blocks are full and wholly inside the range),
    /// so this is O(block) regardless of how many rows the range spans — the planner's
    /// cheap cardinality estimate.
    pub fn count_range(&self, lo: [Id; 3], hi: [Id; 3]) -> usize {
        if self.dir.is_empty() || lo > hi {
            return 0;
        }
        let first = self.dir.partition_point(|&(k, _)| k <= lo).saturating_sub(1);
        let last = self.dir.partition_point(|&(k, _)| k <= hi).saturating_sub(1).max(first);
        let mut buf = Vec::with_capacity(BLOCK);
        self.decode_block_at(self.dir[first].1 as usize, &mut buf);
        if first == last {
            let s = buf.partition_point(|r| *r < lo);
            let e = buf.partition_point(|r| *r <= hi);
            return e - s;
        }
        // First block: rows >= lo. Interior blocks: full (BLOCK each). Last block: rows <= hi.
        let first_count = buf.len() - buf.partition_point(|r| *r < lo);
        buf.clear();
        self.decode_block_at(self.dir[last].1 as usize, &mut buf);
        let last_count = buf.partition_point(|r| *r <= hi);
        first_count + (last - first - 1) * BLOCK + last_count
    }

    /// Returns the rows in `[lo, hi]` (inclusive, comparing full triples) by decoding
    /// only the blocks that span that key range, then trimming. The result is sorted —
    /// identical to a binary-search range over the raw permutation.
    pub fn range(&self, lo: [Id; 3], hi: [Id; 3]) -> Vec<[Id; 3]> {
        if self.dir.is_empty() || lo > hi {
            return Vec::new();
        }
        // First block whose first-key could contain `lo`: the last block with first-key
        // <= lo (a key in [lo,hi] could start partway into that block).
        let first = self.dir.partition_point(|&(k, _)| k <= lo).saturating_sub(1);
        // Last block whose first-key <= hi (blocks after it are entirely > hi).
        let last = self.dir.partition_point(|&(k, _)| k <= hi).saturating_sub(1);
        let last = last.max(first);

        let mut decoded = Vec::with_capacity((last - first + 1) * BLOCK);
        for b in first..=last {
            self.decode_block_at(self.dir[b].1 as usize, &mut decoded);
        }
        // Trim to the exact inclusive range.
        let s = decoded.partition_point(|r| *r < lo);
        let e = decoded.partition_point(|r| *r <= hi);
        decoded.drain(..s);
        decoded.truncate(e - s);
        decoded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A deterministic pseudo-random sorted permutation with realistic structure:
    /// clustered subjects (long equal-col0 runs), a few predicates, sparse objects.
    fn sample(n: usize) -> Vec<[Id; 3]> {
        let mut v = Vec::with_capacity(n);
        let mut state = 0x9e3779b9u32;
        let mut next = || {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            state
        };
        for _ in 0..n {
            let s = 1 + (next() % (n as u32 / 8).max(1)); // clustered subjects
            let p = 1 + (next() % 20); // few predicates
            let o = 1 + (next() % 1_000_000); // sparse objects
            v.push([s, p, o]);
        }
        v.sort_unstable();
        v.dedup();
        v
    }

    #[test]
    fn roundtrip_decode_all() {
        for &n in &[0usize, 1, 5, 127, 128, 129, 1000, 5000] {
            let rows = sample(n);
            let c = CompressedPerm::encode(&rows);
            assert_eq!(c.len(), rows.len());
            assert_eq!(c.decode_all(), rows, "decode_all mismatch at n={n}");
        }
    }

    #[test]
    fn range_matches_binary_search() {
        let rows = sample(5000);
        let c = CompressedPerm::encode(&rows);
        // Reference: a raw binary-search range over the sorted rows.
        let raw_range = |lo: [Id; 3], hi: [Id; 3]| -> Vec<[Id; 3]> {
            let s = rows.partition_point(|r| *r < lo);
            let e = rows.partition_point(|r| *r <= hi);
            rows[s..e].to_vec()
        };
        // Full scan, a bound-subject prefix, a bound subject+predicate prefix, an empty
        // range, and the boundaries.
        let cases: &[([Id; 3], [Id; 3])] = &[
            ([Id::MIN; 3], [Id::MAX; 3]),
            ([100, Id::MIN, Id::MIN], [100, Id::MAX, Id::MAX]),
            ([200, 5, Id::MIN], [200, 5, Id::MAX]),
            ([99999999, Id::MIN, Id::MIN], [99999999, Id::MAX, Id::MAX]),
            (rows[0], rows[0]),
            (rows[rows.len() / 2], rows[rows.len() / 2]),
            (rows[rows.len() - 1], rows[rows.len() - 1]),
        ];
        for &(lo, hi) in cases {
            assert_eq!(c.range(lo, hi), raw_range(lo, hi), "range mismatch for {lo:?}..={hi:?}");
        }
    }
}
