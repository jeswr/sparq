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

/// A block-compressed, random-accessible sorted permutation.
pub struct CompressedPerm {
    /// One entry per block: (its first triple, its byte offset into `blocks`).
    dir: Vec<([Id; 3], u32)>,
    /// The concatenated encoded blocks.
    blocks: Vec<u8>,
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
        CompressedPerm { dir, blocks, len: rows.len() }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Resident bytes (directory + block stream).
    pub fn heap_bytes(&self) -> usize {
        self.dir.capacity() * std::mem::size_of::<([Id; 3], u32)>() + self.blocks.capacity()
    }

    /// Decodes the block starting at byte `off` into `out` (appending).
    #[inline]
    fn decode_block_at(&self, off: usize, out: &mut Vec<[Id; 3]>) {
        let buf = &self.blocks;
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
            let cur = if d0 == 0 {
                let d1 = get_varint(buf, &mut pos) as Id;
                if d1 == 0 {
                    [prev[0], prev[1], prev[2] + get_varint(buf, &mut pos) as Id]
                } else {
                    [prev[0], prev[1] + d1, get_varint(buf, &mut pos) as Id]
                }
            } else {
                [prev[0] + d0, get_varint(buf, &mut pos) as Id, get_varint(buf, &mut pos) as Id]
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
