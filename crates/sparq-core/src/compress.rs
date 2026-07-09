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

/// [FABLE-5] sq-7d3dj.32.2.6 — appends `x` to `out` as a ZIGZAG-mapped LEB128 varint (used by
/// the SPQCPRM2 spike to encode a col2 frame-of-reference offset, which can be negative). Zigzag
/// maps `0,-1,1,-2,2,…` to `0,1,2,3,4,…` so a small-magnitude signed offset stays a short varint.
#[cfg(feature = "spqcprm2")]
#[inline]
fn put_zigzag_varint(out: &mut Vec<u8>, x: i64) {
    put_varint(out, ((x << 1) ^ (x >> 63)) as u64);
}

/// [FABLE-5] sq-7d3dj.32.2.6 — reads a zigzag-mapped LEB128 varint written by [`put_zigzag_varint`],
/// advancing `*pos`. The inverse zigzag maps the unsigned varint back to a signed offset.
#[cfg(feature = "spqcprm2")]
#[inline]
fn get_zigzag_varint(buf: &[u8], pos: &mut usize) -> i64 {
    let u = get_varint(buf, pos);
    ((u >> 1) as i64) ^ -((u & 1) as i64)
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

/// [OPUS-4.8] sq-wihld (survey §A1) — OPT-IN per-block Bloom filters on the leading
/// (column-0) ids of a block-compressed permutation. The directory's first-triple already
/// gives an implicit min/max zone map that prunes RANGE scans, but for an EQUALITY-BOUND
/// leading column (a point/prefix lookup, `lo[0] == hi[0]`) whose id falls inside several
/// overlapping blocks' `[min,max]` spans — the common case on a high-NDV subject/object
/// column — nothing skips a block that cannot contain the id. A tiny per-block Bloom bitset
/// fixes exactly that gap: probe the block's filter and skip its `decode_block_at` when the
/// id is provably absent.
///
/// CORRECTNESS (load-bearing): zero false NEGATIVES by construction — a leading id that is
/// present in a block always probes as "maybe present", so no matching row is ever skipped.
/// A false POSITIVE costs one wasted block decode whose rows the existing range trim then
/// discards, so the `range` output is byte-identical to the no-Bloom path. The filter is an
/// in-RAM/build-time acceleration only: it is NEVER written to the on-disk `SPQCPRM1` format,
/// so a perm built with the feature on and one built with it off persist identically.
///
/// This module is compiled only under the `block-bloom` cargo feature; with the feature off
/// the `CompressedPerm` carries no `bloom` field and `range` is the original code verbatim.
#[cfg(feature = "block-bloom")]
mod block_bloom {
    use super::{Id, BLOCK};

    /// Bits per block filter. A block holds at most [`BLOCK`] (=128) rows, so at most 128
    /// distinct leading ids; 256 bits (32 bytes/block ≈ 0.25 B/triple) keeps the
    /// false-positive rate low at the [`HASHES`] count below while staying a flat,
    /// WASM-trivial bitset. The directory already costs ~0.13 B/triple, so this roughly
    /// triples the resident directory of a Bloom-enabled column — paid only on the high-NDV
    /// columns the density gate admits.
    const BITS: usize = 256;
    /// Machine words (u64) per block filter.
    const WORDS: usize = BITS / 64;
    /// Hash probes per id. Two independent probes (double hashing off one 64-bit hash) is
    /// the sweet spot for ~128 keys in 256 bits; more probes would over-fill the bitset.
    const HASHES: u32 = 2;

    /// FNV-1a 64-bit hash of a leading-column id, the seed for double hashing. Deterministic
    /// and endian-stable (we hash the little-endian id bytes), so a filter built on one host
    /// probes identically on another — though filters are never serialised, this keeps the
    /// build reproducible.
    #[inline]
    fn hash_id(id: Id) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for b in id.to_le_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
        h
    }

    /// A flat array of fixed-size per-block Bloom filters over each block's distinct
    /// column-0 ids, one filter per directory entry (same index space as the directory).
    pub struct BlockBloomDir {
        /// `WORDS` u64 words per block, laid out contiguously: block `b` occupies
        /// `words[b * WORDS .. (b + 1) * WORDS]`.
        words: Vec<u64>,
    }

    impl BlockBloomDir {
        /// Inserts `id` into block `b`'s filter via [`HASHES`] double-hashed probes.
        #[inline]
        fn insert(words: &mut [u64], b: usize, id: Id) {
            let h = hash_id(id);
            let (h1, h2) = (h as usize, ((h >> 32) | 1) as usize); // odd step ⇒ distinct probes
            let base = b * WORDS;
            for k in 0..HASHES as usize {
                let bit = (h1.wrapping_add(k.wrapping_mul(h2))) % BITS;
                words[base + bit / 64] |= 1u64 << (bit % 64);
            }
        }

        /// `true` if block `b`'s filter says `id` MIGHT be present (never a false negative);
        /// `false` ⇒ `id` is definitely absent, so the block can be skipped.
        #[inline]
        pub fn maybe_contains(&self, b: usize, id: Id) -> bool {
            let h = hash_id(id);
            let (h1, h2) = (h as usize, ((h >> 32) | 1) as usize);
            let base = b * WORDS;
            for k in 0..HASHES as usize {
                let bit = (h1.wrapping_add(k.wrapping_mul(h2))) % BITS;
                if self.words[base + bit / 64] & (1u64 << (bit % 64)) == 0 {
                    return false;
                }
            }
            true
        }

        /// Resident bytes of the filter array.
        #[inline]
        pub fn heap_bytes(&self) -> usize {
            self.words.capacity() * std::mem::size_of::<u64>()
        }

        /// Builds one filter per block over each block's distinct leading (column-0) ids,
        /// but ONLY when the leading column is high-NDV enough for a filter to ever skip a
        /// block. Returns `None` for a low-NDV leading column (e.g. a predicate-leading
        /// permutation, where every block's id set is tiny and overlapping blocks are rare),
        /// so dense columns pay no filter bytes. The density gate: build only if the average
        /// number of DISTINCT leading ids per block is at least [`MIN_AVG_DISTINCT_PER_BLOCK`]
        /// — i.e. the column actually varies fast enough within a block that an out-of-cluster
        /// point lookup lands in blocks that do not contain it.
        pub fn build(blocks: &[&[[Id; 3]]]) -> Option<Self> {
            if blocks.is_empty() {
                return None;
            }
            // First pass: estimate density (total distinct leading ids across blocks).
            let mut total_distinct: usize = 0;
            for chunk in blocks {
                let mut prev: Option<Id> = None;
                for r in chunk.iter() {
                    if prev != Some(r[0]) {
                        total_distinct += 1;
                        prev = Some(r[0]);
                    }
                }
            }
            let avg = total_distinct as f64 / blocks.len() as f64;
            if avg < MIN_AVG_DISTINCT_PER_BLOCK {
                return None; // low-NDV leading column: a Bloom filter would never skip.
            }
            // Second pass: build the filters. Rows within a block are sorted, so equal
            // leading ids are contiguous — insert each distinct id once.
            let mut words = vec![0u64; blocks.len() * WORDS];
            for (b, chunk) in blocks.iter().enumerate() {
                let mut prev: Option<Id> = None;
                for r in chunk.iter() {
                    if prev != Some(r[0]) {
                        Self::insert(&mut words, b, r[0]);
                        prev = Some(r[0]);
                    }
                }
            }
            Some(BlockBloomDir { words })
        }
    }

    /// Density gate (see [`BlockBloomDir::build`]). A column whose blocks average fewer than
    /// this many distinct leading ids is so clustered that the min/max zone map already
    /// prunes effectively and a Bloom filter would virtually never skip a block — so we skip
    /// the filter to keep the directory lean. Chosen conservatively (a full BLOCK of distinct
    /// ids is 128; this admits columns where at least an eighth of a block's rows start a new
    /// leading id).
    const MIN_AVG_DISTINCT_PER_BLOCK: f64 = (BLOCK / 8) as f64;
}

/// A block-compressed, random-accessible sorted permutation.
pub struct CompressedPerm {
    /// One entry per block: (its first triple, its byte offset into `blocks`).
    dir: Vec<([Id; 3], u32)>,
    /// The concatenated encoded blocks.
    blocks: Blocks,
    len: usize,
    /// [OPUS-4.8] sq-wihld — OPT-IN (`block-bloom` feature) per-block Bloom filters over the
    /// leading column, parallel to `dir`. `None` when the feature built no filter for this
    /// perm (low-NDV leading column, empty perm, or a perm opened from disk — filters are not
    /// serialised). Used only to skip blocks on an equality-bound leading column in `range`.
    #[cfg(feature = "block-bloom")]
    bloom: Option<block_bloom::BlockBloomDir>,
    /// [FABLE-5] sq-7d3dj.32.2.6 — which block-stream encoding this perm's `blocks` uses. Only
    /// present under the `spqcprm2` spike feature; with the feature off the struct is byte-for-
    /// byte the default (there is only SPQCPRM1) and the decode hot path is unchanged. A perm
    /// built by `encode` / `from_mmap` is always `V1`; only `encode_v2` produces `V2`.
    #[cfg(feature = "spqcprm2")]
    format: Format,
}

/// [FABLE-5] sq-7d3dj.32.2.6 — the block-stream encoding a [`CompressedPerm`] carries. `V1` is
/// the shipped `SPQCPRM1` (col2 reset written ABSOLUTE); `V2` is the `SPQCPRM2` spike (col2
/// reset frame-of-reference: a zigzag delta from the block's first-row col2). The two decode
/// through different block readers, so a perm's `format` picks the reader — see `decode_block_at`.
#[cfg(feature = "spqcprm2")]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Format {
    /// The shipped absolute-col2-reset encoding (`SPQCPRM1`).
    V1,
    /// The frame-of-reference col2-reset spike encoding (`SPQCPRM2`).
    V2,
}

/// Encodes one block (`chunk`, 1..=`BLOCK` sorted rows) into `out`, appending its
/// `count` varint, first row absolute, then per-row lexicographic deltas. Shared by the
/// in-RAM [`CompressedPerm::encode`] and the streaming [`CompressedPermWriter`] so both
/// emit a BYTE-IDENTICAL block stream. [OPUS-4.8] sq-vkz7
#[inline]
fn encode_block(chunk: &[[Id; 3]], out: &mut Vec<u8>) {
    put_varint(out, chunk.len() as u64);
    // First row absolute.
    put_varint(out, chunk[0][0] as u64);
    put_varint(out, chunk[0][1] as u64);
    put_varint(out, chunk[0][2] as u64);
    // Remaining rows: lexicographic delta vs the previous row.
    for w in chunk.windows(2) {
        let (p, r) = (w[0], w[1]);
        let d0 = r[0] - p[0];
        put_varint(out, d0 as u64);
        if d0 == 0 {
            let d1 = r[1] - p[1];
            put_varint(out, d1 as u64);
            if d1 == 0 {
                put_varint(out, (r[2] - p[2]) as u64); // strictly increasing
            } else {
                put_varint(out, r[2] as u64); // col2 resets → absolute
            }
        } else {
            put_varint(out, r[1] as u64); // cols 1,2 reset → absolute
            put_varint(out, r[2] as u64);
        }
    }
}

/// [FABLE-5] sq-7d3dj.32.2.6 — magic prefix of the `SPQCPRM2` on-disk format (frame-of-reference
/// col2 reset). NOT written by the store's default save path — the store still writes and
/// auto-detects only [`FILE_MAGIC`] (`SPQCPRM1`). This constant reserves the version marker so a
/// future migration (after the spike measures a win) can write and auto-detect V2 without a
/// collision. Held here alongside the V1 magic so the two markers stay adjacent and distinct.
#[cfg(all(feature = "spqcprm2", feature = "mmap"))]
pub const FILE_MAGIC_V2: [u8; 8] = *b"SPQCPRM2";

/// [FABLE-5] sq-7d3dj.32.2.6 — SPIKE: encodes one block into the `SPQCPRM2` frame-of-reference
/// stream. Byte-for-byte identical to [`encode_block`] EXCEPT the `reset_d1` col2 (written when
/// `d0 == 0 && d1 != 0`): SPQCPRM1 emits `r[2]` absolute; SPQCPRM2 emits the zigzag delta
/// `r[2] - first_col2`, where `first_col2` is the block's first-row col2 (the frame origin). The
/// decoder ([`decode_block_v2_at`]) reads that same first-row col2 at block start, so it can add
/// the offset back with no extra state. All other write sites are unchanged, so a V2 block that
/// happens to contain no `reset_d1` row is byte-identical to its V1 block.
#[cfg(feature = "spqcprm2")]
#[inline]
fn encode_block_v2(chunk: &[[Id; 3]], out: &mut Vec<u8>) {
    put_varint(out, chunk.len() as u64);
    let first_col2 = chunk[0][2];
    put_varint(out, chunk[0][0] as u64);
    put_varint(out, chunk[0][1] as u64);
    put_varint(out, chunk[0][2] as u64);
    for w in chunk.windows(2) {
        let (p, r) = (w[0], w[1]);
        let d0 = r[0] - p[0];
        put_varint(out, d0 as u64);
        if d0 == 0 {
            let d1 = r[1] - p[1];
            put_varint(out, d1 as u64);
            if d1 == 0 {
                put_varint(out, (r[2] - p[2]) as u64); // strictly increasing
            } else {
                // FRAME-OF-REFERENCE: col2 reset written as a signed delta from the block's
                // first-row col2 instead of the absolute id (the SPQCPRM1 `reset_d1` bucket).
                put_zigzag_varint(out, r[2] as i64 - first_col2 as i64);
            }
        } else {
            put_varint(out, r[1] as u64); // cols 1,2 reset → absolute (unchanged from V1)
            put_varint(out, r[2] as u64);
        }
    }
}

impl CompressedPerm {
    /// Encodes a sorted permutation (rows already in this permutation's column order).
    pub fn encode(rows: &[[Id; 3]]) -> Self {
        let mut dir = Vec::with_capacity(rows.len() / BLOCK + 1);
        let mut blocks = Vec::with_capacity(rows.len() * 6);
        for chunk in rows.chunks(BLOCK) {
            dir.push((chunk[0], blocks.len() as u32));
            encode_block(chunk, &mut blocks);
        }
        // [OPUS-4.8] sq-wihld — build the OPT-IN per-block Bloom directory over the leading
        // column (off the same chunks, so it is exactly aligned with `dir`). `build` returns
        // `None` on a low-NDV leading column, so dense columns pay nothing.
        #[cfg(feature = "block-bloom")]
        let bloom = block_bloom::BlockBloomDir::build(&rows.chunks(BLOCK).collect::<Vec<_>>());
        CompressedPerm {
            dir,
            blocks: Blocks::Owned(blocks),
            len: rows.len(),
            #[cfg(feature = "block-bloom")]
            bloom,
            #[cfg(feature = "spqcprm2")]
            format: Format::V1,
        }
    }

    /// [FABLE-5] sq-7d3dj.32.2.6 — SPIKE: encodes a sorted permutation into the `SPQCPRM2`
    /// frame-of-reference block stream. Identical to [`encode`](Self::encode) except the col2
    /// that resets after a middle-column change (the `reset_d1` bucket the sq-7d3dj.32.2.4
    /// attribution found dominant at scale) is written as a ZIGZAG DELTA from the block's
    /// first-row col2 rather than an absolute varint. When a block's objects cluster (the
    /// common case for related subject/predicate rows) the frame offset is smaller than the
    /// absolute id, so fewer varint bytes. Everything else — first row, `d0`/`d1`/`d2` deltas,
    /// and the `reset_d0` (cols 1,2 absolute) shape — is byte-for-byte the SPQCPRM1 encoder.
    ///
    /// The returned perm's `format` is `V2`, so its own `decode_all` / `range` decode through
    /// the matching reader; it is NOT written by the store's default save path (which stays
    /// SPQCPRM1), so this is a measurement/round-trip spike, not a format migration.
    #[cfg(feature = "spqcprm2")]
    pub fn encode_v2(rows: &[[Id; 3]]) -> Self {
        let mut dir = Vec::with_capacity(rows.len() / BLOCK + 1);
        let mut blocks = Vec::with_capacity(rows.len() * 6);
        for chunk in rows.chunks(BLOCK) {
            dir.push((chunk[0], blocks.len() as u32));
            encode_block_v2(chunk, &mut blocks);
        }
        #[cfg(feature = "block-bloom")]
        let bloom = block_bloom::BlockBloomDir::build(&rows.chunks(BLOCK).collect::<Vec<_>>());
        CompressedPerm {
            dir,
            blocks: Blocks::Owned(blocks),
            len: rows.len(),
            #[cfg(feature = "block-bloom")]
            bloom,
            format: Format::V2,
        }
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
        // [OPUS-4.8] sq-wihld — the on-disk `SPQCPRM1` format carries NO Bloom filters (they
        // are an in-RAM acceleration, never serialised), so a memory-mapped perm has none. The
        // mmap/out-of-core path therefore behaves exactly as before this feature; the Bloom
        // skip only ever fires on an in-RAM `encode`d perm. This keeps the on-disk format and
        // the mmap scan path byte-identical regardless of the `block-bloom` feature.
        let perm = CompressedPerm {
            dir,
            blocks: Blocks::Mapped { map, off: dir_end },
            len,
            #[cfg(feature = "block-bloom")]
            bloom: None,
            // [FABLE-5] sq-7d3dj.32.2.6 — the on-disk `FILE_MAGIC` gate here matches only the
            // SPQCPRM1 magic, so a mapped perm is always V1. SPQCPRM2 is an in-RAM measurement
            // spike that the store never writes; it therefore never reaches this open path.
            #[cfg(feature = "spqcprm2")]
            format: Format::V1,
        };
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
        // [OPUS-4.8] sq-wihld — the OPT-IN per-block Bloom directory (when built) is resident
        // heap; count it so `heap_bytes` stays honest for the memory-budget paths.
        #[cfg(feature = "block-bloom")]
        let bloom = self.bloom.as_ref().map_or(0, block_bloom::BlockBloomDir::heap_bytes);
        #[cfg(not(feature = "block-bloom"))]
        let bloom = 0;
        self.dir.capacity() * std::mem::size_of::<([Id; 3], u32)>() + stream + bloom
    }

    /// Decodes the block starting at byte `off` into `out` (appending).
    #[inline]
    fn decode_block_at(&self, off: usize, out: &mut Vec<[Id; 3]>) {
        // [FABLE-5] sq-7d3dj.32.2.6 — dispatch on the perm's block-stream format. With the
        // `spqcprm2` feature off there is only V1 and this is the original code verbatim.
        #[cfg(feature = "spqcprm2")]
        if self.format == Format::V2 {
            return self.decode_block_v2_at(off, out);
        }
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

    /// [FABLE-5] sq-7d3dj.32.2.6 — decodes one `SPQCPRM2` frame-of-reference block at byte
    /// `off` into `out`. Mirrors [`decode_block_at`] exactly except the `reset_d1` col2 is
    /// reconstructed from the zigzag frame offset plus the block's first-row col2 (captured
    /// as `first_col2` from the absolute first row), inverting [`encode_block_v2`]. The same
    /// `wrapping_add` id-overflow safety as the V1 decoder applies (a tampered block is
    /// wrong-but-safe, never a panic / OOB — the documented trusted-store boundary).
    #[cfg(feature = "spqcprm2")]
    #[inline]
    fn decode_block_v2_at(&self, off: usize, out: &mut Vec<[Id; 3]>) {
        let buf = self.blocks.bytes();
        let mut pos = off;
        let count = get_varint(buf, &mut pos) as usize;
        let mut prev = [
            get_varint(buf, &mut pos) as Id,
            get_varint(buf, &mut pos) as Id,
            get_varint(buf, &mut pos) as Id,
        ];
        let first_col2 = prev[2] as i64; // the frame origin the encoder used
        out.push(prev);
        for _ in 1..count {
            let d0 = get_varint(buf, &mut pos) as Id;
            let cur = if d0 == 0 {
                let d1 = get_varint(buf, &mut pos) as Id;
                if d1 == 0 {
                    [prev[0], prev[1], prev[2].wrapping_add(get_varint(buf, &mut pos) as Id)]
                } else {
                    // Invert the frame-of-reference: absolute col2 = first_col2 + signed offset.
                    let off2 = get_zigzag_varint(buf, &mut pos);
                    let abs = (first_col2.wrapping_add(off2)) as u64 as Id;
                    [prev[0], prev[1].wrapping_add(d1), abs]
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

        // [OPUS-4.8] sq-wihld — when the LEADING column is equality-bound (`lo[0] == hi[0]`,
        // i.e. a point/prefix lookup on a constant subject/object), an OPT-IN per-block Bloom
        // filter can skip blocks whose `[min,max]` zone-map span overlaps the constant but
        // that do not actually contain it (the high-NDV overlapping-block case the min/max map
        // cannot prune). The filter has zero false negatives, so any block it skips provably
        // holds no row with that leading id — the trimmed range below is unchanged. We only
        // consult it for the leading-equality shape; a range / full scan runs the loop as before.
        #[cfg(feature = "block-bloom")]
        let bloom_key: Option<Id> = match &self.bloom {
            Some(_) if lo[0] == hi[0] => Some(lo[0]),
            _ => None,
        };

        let mut decoded = Vec::with_capacity((last - first + 1) * BLOCK);
        for b in first..=last {
            #[cfg(feature = "block-bloom")]
            if let (Some(key), Some(bloom)) = (bloom_key, &self.bloom) {
                if !bloom.maybe_contains(b, key) {
                    continue; // block provably holds no row with this leading id.
                }
            }
            self.decode_block_at(self.dir[b].1 as usize, &mut decoded);
        }
        // Trim to the exact inclusive range.
        let s = decoded.partition_point(|r| *r < lo);
        let e = decoded.partition_point(|r| *r <= hi);
        decoded.drain(..s);
        decoded.truncate(e - s);
        decoded
    }

    /// [OPUS-4.8] sq-wihld — test-only: was a per-block Bloom directory built for this perm?
    /// Lets a test confirm the density gate admitted a high-NDV leading column (and so the
    /// skip path is actually exercised), versus declining a low-NDV one.
    #[cfg(all(test, feature = "block-bloom"))]
    fn has_bloom(&self) -> bool {
        self.bloom.is_some()
    }

    /// [OPUS-4.8] sq-wihld — test-only: for an equality-bound leading id `key`, returns
    /// `(candidate_blocks, bloom_skipped)` over the zone-map span the `range` loop would
    /// visit — `candidate_blocks` is the number of blocks whose `[min,max]` span overlaps the
    /// point lookup (what the min/max zone map alone leaves to decode) and `bloom_skipped` is
    /// how many of those the Bloom filter proves cannot contain `key` (so `range` skips their
    /// decode). A positive `bloom_skipped` proves the optimisation does real work; the value is
    /// purely diagnostic and never asserted as a performance number.
    #[cfg(all(test, feature = "block-bloom"))]
    fn bloom_skip_stats(&self, key: Id) -> (usize, usize) {
        let lo = [key, Id::MIN, Id::MIN];
        let hi = [key, Id::MAX, Id::MAX];
        if self.dir.is_empty() {
            return (0, 0);
        }
        let first = self.dir.partition_point(|&(k, _)| k <= lo).saturating_sub(1);
        let last = self.dir.partition_point(|&(k, _)| k <= hi).saturating_sub(1).max(first);
        let candidates = last - first + 1;
        let bloom = self.bloom.as_ref().expect("bloom_skip_stats requires a built filter");
        let skipped = (first..=last).filter(|&b| !bloom.maybe_contains(b, key)).count();
        (candidates, skipped)
    }
}

/// [OPUS-4.8] sq-vkz7 — STREAMING writer of the [`CompressedPerm`] on-disk format
/// ([`FILE_MAGIC`] `SPQCPRM1`). Encodes the FoR+varint block stream as sorted rows arrive,
/// so the external-memory build can emit compressed perms STRAIGHT FROM the merge tail —
/// no raw-write-then-reopen-then-`decode_all`-then-`encode` recompress second pass over an
/// 84+ GB index. The byte stream it produces is BYTE-IDENTICAL to
/// `CompressedPerm::encode(rows).write_to(w)` for the same `rows` (proven in tests).
///
/// The format is `header[32] | directory | blocks`, so the directory (which we only finish
/// once the last block is sealed) must physically precede the block stream. We therefore
/// buffer the SPARSE directory in RAM (one 16-byte entry per [`BLOCK`] rows ≈ 0.13 B/triple
/// — the same directory the open path already holds resident) and stream the block bytes to
/// a side file; [`finish`](Self::finish) writes `header | directory` then appends the block
/// side file. The block COPY is over the already-compressed stream (~2.5× smaller than raw),
/// and there is no decode / re-sort — the saving the bead targets.
#[cfg(feature = "mmap")]
pub struct CompressedPermWriter {
    /// One (first-triple, byte-offset-into-blocks) entry per sealed block.
    dir: Vec<([Id; 3], u32)>,
    /// The current (not-yet-sealed) block's rows, up to [`BLOCK`].
    cur: Vec<[Id; 3]>,
    /// Reusable scratch for one encoded block.
    scratch: Vec<u8>,
    /// The block stream, written to a side file as blocks are sealed. `None` only after
    /// [`finish`](Self::finish) has taken it to close the write handle before the read.
    body: Option<std::io::BufWriter<std::fs::File>>,
    /// Path of the block side file (removed by [`finish`](Self::finish)).
    body_path: std::path::PathBuf,
    /// Running length of the block stream in bytes (the next block's byte offset).
    blocks_len: u64,
    /// Total rows pushed so far.
    len: u64,
}

#[cfg(feature = "mmap")]
impl CompressedPermWriter {
    /// Creates a writer that will produce the compressed perm at `out`, staging the block
    /// stream in a sibling temp file `<out>.blocks` (same directory ⇒ same filesystem, so
    /// the final assembly copy never crosses devices).
    pub fn create(out: &std::path::Path) -> std::io::Result<Self> {
        let mut body_path = out.as_os_str().to_owned();
        body_path.push(".blocks");
        let body_path = std::path::PathBuf::from(body_path);
        let body = std::io::BufWriter::new(std::fs::File::create(&body_path)?);
        Ok(CompressedPermWriter {
            dir: Vec::new(),
            cur: Vec::with_capacity(BLOCK),
            scratch: Vec::with_capacity(BLOCK * 6),
            body: Some(body),
            body_path,
            blocks_len: 0,
            len: 0,
        })
    }

    /// Appends one row. Rows MUST arrive in this permutation's sorted column order and be
    /// already deduplicated (the merge tail guarantees both); the delta encoder relies on
    /// `row >= prev` and strictly-increasing within an equal-prefix run.
    pub fn push(&mut self, row: [Id; 3]) -> std::io::Result<()> {
        self.cur.push(row);
        self.len += 1;
        if self.cur.len() == BLOCK {
            self.seal_block()?;
        }
        Ok(())
    }

    /// Seals the current full/partial block: records its directory entry then encodes it to
    /// the block side file. A no-op if the current block is empty.
    fn seal_block(&mut self) -> std::io::Result<()> {
        if self.cur.is_empty() {
            return Ok(());
        }
        // The directory byte-offset must fit u32, exactly as `CompressedPerm::encode`'s does
        // (the directory stores `u32` offsets). A single perm's block stream is bounded by
        // u32 by construction of the format; surface an error rather than silently truncate.
        let off = u32::try_from(self.blocks_len).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "compressed perm: block stream exceeds 4 GiB (u32 directory offset overflow)",
            )
        })?;
        self.dir.push((self.cur[0], off));
        self.scratch.clear();
        encode_block(&self.cur, &mut self.scratch);
        let body = self.body.as_mut().expect("body present until finish() consumes the writer");
        std::io::Write::write_all(body, &self.scratch)?;
        self.blocks_len += self.scratch.len() as u64;
        self.cur.clear();
        Ok(())
    }

    /// Finishes the stream: seals any partial block, then writes the final `SPQCPRM1` file to
    /// `out` (`header | directory | blocks`) and removes the block side file. For a non-empty
    /// perm the bytes are identical to `CompressedPerm::encode(all_rows).write_to(out)`.
    ///
    /// EMPTY-PERM POLICY: when no rows were pushed, `out` is written as a ZERO-byte file
    /// (NOT a bare 32-byte header), matching `TripleStore::save_compressed` which leaves an
    /// unbuilt permutation raw-empty so `open` skips it by size. This keeps the streaming
    /// build byte-identical to a raw build followed by `recompress`.
    pub fn finish(mut self, out: &std::path::Path) -> std::io::Result<()> {
        self.seal_block()?;
        // Flush and CLOSE the block write handle before re-opening it for reading (portable;
        // avoids a concurrent write+read handle to the same file). `take()` drops the inner
        // `File` here rather than at end of scope.
        let mut body = self.body.take().expect("body present until finish() consumes the writer");
        std::io::Write::flush(&mut body)?;
        drop(body);
        if self.len == 0 {
            // Unbuilt/empty perm: raw-empty file, like `save_compressed`'s `!rows.is_empty()`.
            std::fs::File::create(out)?;
            std::fs::remove_file(&self.body_path).ok();
            return Ok(());
        }
        // Re-open the staged block stream for the assembly copy.
        let mut body_rd = std::io::BufReader::new(std::fs::File::open(&self.body_path)?);
        let mut w = std::io::BufWriter::new(std::fs::File::create(out)?);
        {
            use std::io::Write;
            w.write_all(&FILE_MAGIC)?;
            w.write_all(&self.len.to_le_bytes())?;
            w.write_all(&(self.dir.len() as u64).to_le_bytes())?;
            w.write_all(&self.blocks_len.to_le_bytes())?;
            for &(key, off) in &self.dir {
                for c in key {
                    w.write_all(&c.to_le_bytes())?;
                }
                w.write_all(&off.to_le_bytes())?;
            }
        }
        std::io::copy(&mut body_rd, &mut w)?;
        std::io::Write::flush(&mut w)?;
        drop(body_rd);
        std::fs::remove_file(&self.body_path).ok();
        Ok(())
    }
}

#[cfg(feature = "mmap")]
impl Drop for CompressedPermWriter {
    fn drop(&mut self) {
        // If `finish` was not called (e.g. a build error unwound past us), don't leak the
        // staged block side file. `finish` consumes `self`, so reaching `drop` with a path
        // still present means the writer was abandoned.
        std::fs::remove_file(&self.body_path).ok();
    }
}

/// [FABLE-5] sq-7d3dj.32.2.4 — SPIKE-ONLY per-field byte attribution over the
/// [`encode_block`] stream. This whole module is `#[cfg(test)]`: it adds ZERO bytes to
/// the shipped `SPQCPRM1` stream and does not touch the encode/decode hot path (the
/// `wasm_bundle_bytes` `feature_off_exact` floor is unmoved). It exists to adjudicate the
/// §4 root-cause hypotheses (H1/H2/H3) of `research/compressed-memory-profile.md` for why
/// the compressed store's B/triple grows 36.75 (1M) → 48.75 (10M) then plateaus at 48.75
/// (50M): it mirrors `encode_block` field-for-field and attributes each emitted varint's
/// byte length to the field class that produced it. All numbers this module prints are
/// NON-canonical work-box measurements (never a doc/test perf number).
#[cfg(test)]
mod byte_attribution {
    use super::{encode_block, Id, BLOCK};

    /// Byte length of the unsigned LEB128 varint that `put_varint` would emit for `x`
    /// (mirrors its `while x >= 0x80 { x >>= 7 }` loop exactly): `⌈bits/7⌉`, min 1.
    #[inline]
    fn varint_len(x: u64) -> usize {
        let mut x = x;
        let mut n = 1usize;
        while x >= 0x80 {
            x >>= 7;
            n += 1;
        }
        n
    }

    /// The disjoint field classes the block stream's bytes are attributed to. These are
    /// exactly the write sites in [`encode_block`]; every byte the encoder emits lands in
    /// exactly one class, so `total()` equals the real encoded block-stream length.
    #[derive(Clone, Copy, Default, Debug)]
    struct FieldBytes {
        /// The per-block `count` varint (one per block).
        count: u64,
        /// The block's FIRST row, written as three absolute varints (col0/col1/col2).
        first_row_abs: u64,
        /// col1+col2 written ABSOLUTE because the leading column changed (`d0 != 0`).
        reset_d0: u64,
        /// col2 written ABSOLUTE because the middle column changed (`d0 == 0 && d1 != 0`).
        reset_d1: u64,
        /// The `d0` (leading-column) delta varint on every non-first row.
        d0: u64,
        /// The `d1` (middle-column) delta varint, emitted only when `d0 == 0`.
        d1: u64,
        /// The `d2` (trailing-column) delta varint, emitted only when `d0 == 0 && d1 == 0`.
        d2: u64,
    }

    impl FieldBytes {
        fn total(&self) -> u64 {
            self.count + self.first_row_abs + self.reset_d0 + self.reset_d1 + self.d0 + self.d1 + self.d2
        }

        fn add(&mut self, o: &FieldBytes) {
            self.count += o.count;
            self.first_row_abs += o.first_row_abs;
            self.reset_d0 += o.reset_d0;
            self.reset_d1 += o.reset_d1;
            self.d0 += o.d0;
            self.d1 += o.d1;
            self.d2 += o.d2;
        }
    }

    /// Attributes the bytes of ONE block (`chunk`, sorted, 1..=`BLOCK` rows) to field
    /// classes, mirroring [`encode_block`] write-site for write-site. This is a pure
    /// re-derivation of the same varint lengths — no bytes are emitted — so its `total()`
    /// is asserted equal to the real `encode_block` output length in a self-check test.
    fn attribute_block(chunk: &[[Id; 3]], fb: &mut FieldBytes) {
        fb.count += varint_len(chunk.len() as u64) as u64;
        fb.first_row_abs += (varint_len(chunk[0][0] as u64)
            + varint_len(chunk[0][1] as u64)
            + varint_len(chunk[0][2] as u64)) as u64;
        for w in chunk.windows(2) {
            let (p, r) = (w[0], w[1]);
            let d0 = r[0] - p[0];
            fb.d0 += varint_len(d0 as u64) as u64;
            if d0 == 0 {
                let d1 = r[1] - p[1];
                fb.d1 += varint_len(d1 as u64) as u64;
                if d1 == 0 {
                    fb.d2 += varint_len((r[2] - p[2]) as u64) as u64; // strictly increasing
                } else {
                    fb.reset_d1 += varint_len(r[2] as u64) as u64; // col2 resets → absolute
                }
            } else {
                // cols 1,2 reset → absolute
                fb.reset_d0 += (varint_len(r[1] as u64) + varint_len(r[2] as u64)) as u64;
            }
        }
    }

    /// Attributes a whole permutation (sorted rows in this permutation's column order),
    /// blocking exactly as [`CompressedPerm::encode`] does (`rows.chunks(BLOCK)`), and
    /// adding the resident directory cost (16 B/block: one `([Id;3], u32)` per block).
    fn attribute_perm(rows: &[[Id; 3]]) -> (FieldBytes, u64) {
        let mut fb = FieldBytes::default();
        let mut n_blocks = 0u64;
        for chunk in rows.chunks(BLOCK) {
            attribute_block(chunk, &mut fb);
            n_blocks += 1;
        }
        let dir_bytes = n_blocks * 16;
        (fb, dir_bytes)
    }

    /// A deterministic xorshift PRNG (same family as the file's `sample`), seeded so runs
    /// are reproducible across hosts.
    struct Rng(u64);
    impl Rng {
        fn new(seed: u64) -> Self {
            Rng(seed | 1)
        }
        #[inline]
        fn next(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }
        /// A Zipf-ish skewed draw in `1..=n`: bias toward small ids (frequent terms get
        /// small, insertion-ordered ids — the H3 mechanism). Squaring a uniform in [0,1)
        /// concentrates mass near 0; `1 +` maps to `1..=n`.
        #[inline]
        fn skewed(&mut self, n: u64) -> u64 {
            let u = (self.next() >> 11) as f64 / (1u64 << 53) as f64; // uniform [0,1)
            1 + ((u * u) * (n.saturating_sub(1)) as f64) as u64
        }
    }

    /// Synthesises `n_triples` canonical `[s, p, o]` triples over a term space of
    /// `n_terms` distinct ids, mimicking WatDiv skew: a small predicate vocabulary,
    /// skewed (Zipf-ish) subjects and objects so frequent terms take small ids — the
    /// insertion-order-gives-small-ids property H3 turns on. Deduplicated + returned
    /// UNSORTED (the caller re-sorts per permutation).
    fn synth_watdiv(n_triples: usize, n_terms: u64, seed: u64) -> Vec<[Id; 3]> {
        let mut rng = Rng::new(seed);
        // Reserve the low id band for predicates (WatDiv has ~85 predicates); subjects &
        // objects range across the whole term space but skewed toward small ids.
        let n_preds: u64 = 85;
        let mut set = std::collections::HashSet::with_capacity(n_triples);
        let mut v = Vec::with_capacity(n_triples);
        let mut guard = 0usize;
        while v.len() < n_triples && guard < n_triples * 4 {
            guard += 1;
            let s = rng.skewed(n_terms) as Id;
            let p = (1 + (rng.next() % n_preds)) as Id;
            let o = rng.skewed(n_terms) as Id;
            let t = [s, p, o];
            if set.insert(t) {
                v.push(t);
            }
        }
        v
    }

    /// Sorts a copy of `triples` into `perm`'s column order (the exact rows
    /// `from_triples_compressed` would hand `CompressedPerm::encode` for that permutation).
    fn perm_rows(triples: &[[Id; 3]], order: [usize; 3]) -> Vec<[Id; 3]> {
        let mut rows: Vec<[Id; 3]> =
            triples.iter().map(|t| [t[order[0]], t[order[1]], t[order[2]]]).collect();
        rows.sort_unstable();
        rows.dedup();
        rows
    }

    /// The six permutation column orders (kept local so the spike does not depend on
    /// `store::Perm`, which is out of this module's scope).
    const PERM_ORDERS: [(&str, [usize; 3]); 6] = [
        ("SPO", [0, 1, 2]),
        ("SOP", [0, 2, 1]),
        ("PSO", [1, 0, 2]),
        ("POS", [1, 2, 0]),
        ("OSP", [2, 0, 1]),
        ("OPS", [2, 1, 0]),
    ];

    /// SELF-CHECK: the attribution's `total()` must equal the REAL `encode_block` output
    /// length for every block, byte-for-byte — otherwise the attribution table is a
    /// fiction. Runs across block boundaries and both reset shapes.
    #[test]
    fn attribution_total_equals_real_encoded_length() {
        for &n_terms in &[100_000u64, 1_000_000, 5_000_000] {
            let triples = synth_watdiv(20_000, n_terms, 0xA5A5);
            for (_, order) in PERM_ORDERS {
                let rows = perm_rows(&triples, order);
                // Real encoded block-stream length.
                let mut real = Vec::new();
                for chunk in rows.chunks(BLOCK) {
                    encode_block(chunk, &mut real);
                }
                // Attributed length.
                let (fb, _dir) = attribute_perm(&rows);
                assert_eq!(
                    fb.total(),
                    real.len() as u64,
                    "attribution total != real encoded length (n_terms={}, order={:?})",
                    n_terms,
                    order
                );
            }
        }
    }

    /// [FABLE-5] sq-7d3dj.32.2.4 — the MEASUREMENT. Prints a per-field, per-permutation
    /// byte-attribution table at three id-density regimes (100K / 1M / 5M distinct terms),
    /// each with the same triple:term ratio so the ONLY moving variable is term-space bits.
    /// Run with `cargo test -p sparq-core --lib byte_attribution -- --nocapture` to see the
    /// tables. All numbers are NON-canonical work-box measurements. `#[ignore]` so the
    /// default `cargo test` stays fast (the 5M-term regime allocates a few hundred MB); the
    /// self-check test above runs unconditionally and pins the attribution's correctness.
    #[test]
    #[ignore = "spike measurement: run explicitly with --ignored --nocapture (allocates ~hundreds of MB)"]
    fn measure_byte_attribution_across_id_density() {
        // Fixed triple:term ratio (~10 triples/term, roughly WatDiv's density) so the id
        // count grows with the term space and the regimes are comparable per-triple.
        let regimes: [(&str, usize, u64); 3] = [
            ("100K-term", 1_000_000, 100_000),
            ("1M-term", 10_000_000, 1_000_000),
            ("5M-term", 50_000_000, 5_000_000),
        ];

        println!("\n===== sq-7d3dj.32.2.4 per-field byte attribution (NON-canonical work-box) =====");
        for (label, want_triples, n_terms) in regimes {
            let triples = synth_watdiv(want_triples, n_terms, 0x5EED);
            let n = triples.len() as f64;
            let bits = (64 - (n_terms).leading_zeros()) as usize;
            println!(
                "\n--- regime {} : {} distinct triples, {} terms (~{} term-space bits) ---",
                label, triples.len(), n_terms, bits
            );
            println!(
                "{:<5} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9} {:>10} {:>9}",
                "perm", "count", "first", "reset_d0", "reset_d1", "d0", "d1", "d2", "dir", "blk_tot", "B/triple"
            );

            let mut agg = FieldBytes::default();
            let mut agg_dir = 0u64;
            let mut agg_rows = 0u64;
            for (name, order) in PERM_ORDERS {
                let rows = perm_rows(&triples, order);
                let (fb, dir) = attribute_perm(&rows);
                let blk_tot = fb.total();
                let bpt = (blk_tot + dir) as f64 / rows.len().max(1) as f64;
                println!(
                    "{:<5} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9} {:>10} {:>9.3}",
                    name, fb.count, fb.first_row_abs, fb.reset_d0, fb.reset_d1, fb.d0, fb.d1, fb.d2, dir, blk_tot, bpt
                );
                agg.add(&fb);
                agg_dir += dir;
                agg_rows += rows.len() as u64;
            }
            // Per-triple attribution summed across the six permutations (the store's
            // block-stream B/triple = sum over perms / #triples). This is the quantity the
            // §4 table (6.125 → 8.125 B/triple/perm) is about.
            let per_triple = |b: u64| b as f64 / n;
            println!(
                "sum/triple  count={:.3}  first={:.3}  reset_d0={:.3}  reset_d1={:.3}  d0={:.3}  d1={:.3}  d2={:.3}  dir={:.3}  ALL={:.3}",
                per_triple(agg.count),
                per_triple(agg.first_row_abs),
                per_triple(agg.reset_d0),
                per_triple(agg.reset_d1),
                per_triple(agg.d0),
                per_triple(agg.d1),
                per_triple(agg.d2),
                per_triple(agg_dir),
                per_triple(agg.total() + agg_dir),
            );
            let _ = agg_rows;
        }
        println!("\n===== end attribution — verdict adjudicated in the bead note =====\n");
    }

    /// A UNIFORM (non-skewed) id draw in `1..=n`, for the H3 discriminator: if the growth
    /// were purely LEB128 quantization of the term space (H1/H2, blind to id assignment),
    /// swapping skewed→uniform ids at the SAME term-space size would not change B/triple.
    /// If frequent-terms-get-small-ids (H3) is what holds the plateau, uniform ids should
    /// be MORE expensive (they spread mass across the whole bit width).
    fn synth_uniform(n_triples: usize, n_terms: u64, seed: u64) -> Vec<[Id; 3]> {
        let mut rng = Rng::new(seed);
        let n_preds: u64 = 85;
        let mut set = std::collections::HashSet::with_capacity(n_triples);
        let mut v = Vec::with_capacity(n_triples);
        let mut guard = 0usize;
        while v.len() < n_triples && guard < n_triples * 4 {
            guard += 1;
            let s = (1 + rng.next() % n_terms) as Id;
            let p = (1 + (rng.next() % n_preds)) as Id;
            let o = (1 + rng.next() % n_terms) as Id;
            let t = [s, p, o];
            if set.insert(t) {
                v.push(t);
            }
        }
        v
    }

    fn agg_per_triple(triples: &[[Id; 3]]) -> (FieldBytes, u64, u64) {
        let mut agg = FieldBytes::default();
        let mut agg_dir = 0u64;
        let mut rows_tot = 0u64;
        for (_, order) in PERM_ORDERS {
            let rows = perm_rows(triples, order);
            let (fb, dir) = attribute_perm(&rows);
            agg.add(&fb);
            agg_dir += dir;
            rows_tot += rows.len() as u64;
        }
        (agg, agg_dir, rows_tot)
    }

    /// [FABLE-5] sq-7d3dj.32.2.4 — H1/H2/H3 DISCRIMINATOR. Two controlled sweeps at a
    /// FIXED triple count so the only variable is (a) term-space bits and (b) skewed vs
    /// uniform id assignment. NON-canonical work-box. Run with `--ignored --nocapture`.
    ///
    /// Sweep A (bit-width, skewed): fixed 4M triples, term space 250K → 4M (18→22 bits).
    /// Isolates how B/triple moves with term-space bits when frequent terms keep small ids.
    ///
    /// Sweep B (skew vs uniform): fixed 4M triples, 4M terms, skewed vs uniform id draw.
    /// If uniform is materially costlier, the plateau is H3 (small-ids-for-frequent-terms),
    /// not pure H1/H2 bit-width quantization.
    #[test]
    #[ignore = "spike discriminator: run explicitly with --ignored --nocapture"]
    fn discriminate_h1_h2_h3() {
        const N: usize = 4_000_000;
        println!("\n===== sq-7d3dj.32.2.4 H1/H2/H3 discriminator (NON-canonical work-box) =====");

        println!("\n--- Sweep A: fixed {} triples, vary term-space bits (SKEWED ids) ---", N);
        println!("{:<10} {:>6} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9} {:>10}", "terms", "bits", "reset_d0", "reset_d1", "d0", "d1", "d2", "first", "B/triple");
        for &n_terms in &[250_000u64, 500_000, 1_000_000, 2_000_000, 4_000_000] {
            let triples = synth_watdiv(N, n_terms, 0xC0FFEE);
            let n = triples.len().max(1) as f64;
            let bits = 64 - n_terms.leading_zeros();
            let (agg, dir, _) = agg_per_triple(&triples);
            let pt = |b: u64| b as f64 / n;
            println!(
                "{:<10} {:>6} {:>9.3} {:>9.3} {:>9.3} {:>9.3} {:>9.3} {:>9.3} {:>10.3}",
                n_terms, bits, pt(agg.reset_d0), pt(agg.reset_d1), pt(agg.d0), pt(agg.d1), pt(agg.d2), pt(agg.first_row_abs), pt(agg.total() + dir)
            );
        }

        println!("\n--- Sweep B: fixed {} triples, 4M terms, SKEWED vs UNIFORM ids ---", N);
        println!("{:<10} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9} {:>10}", "assign", "reset_d0", "reset_d1", "d0", "d1", "d2", "first", "B/triple");
        for (label, triples) in [
            ("skewed", synth_watdiv(N, 4_000_000, 0xC0FFEE)),
            ("uniform", synth_uniform(N, 4_000_000, 0xC0FFEE)),
        ] {
            let n = triples.len().max(1) as f64;
            let (agg, dir, _) = agg_per_triple(&triples);
            let pt = |b: u64| b as f64 / n;
            println!(
                "{:<10} {:>9.3} {:>9.3} {:>9.3} {:>9.3} {:>9.3} {:>9.3} {:>10.3}",
                label, pt(agg.reset_d0), pt(agg.reset_d1), pt(agg.d0), pt(agg.d1), pt(agg.d2), pt(agg.first_row_abs), pt(agg.total() + dir)
            );
        }
        println!("\n===== end discriminator =====\n");
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

    /// [OPUS-4.8] sq-bif — `count_range` is the planner's cheap cardinality estimate and must
    /// return EXACTLY `range(lo, hi).len()` without materialising the range. It only decodes
    /// the two boundary blocks and adds `BLOCK` for each full interior block
    /// (`first_count + (last - first - 1) * BLOCK + last_count`), so an off-by-one in that
    /// arithmetic — or a wrong interior-block count — yields a count that disagrees with the
    /// materialised range. This was previously only reached transitively through the planner;
    /// here we pin the invariant directly, across single-block, two-block and many-interior-
    /// block ranges, the empty / inverted range, and the exact boundary keys.
    #[test]
    fn count_range_equals_materialised_range_len() {
        let rows = sample(5000);
        let c = CompressedPerm::encode(&rows);
        assert!(rows.len() > 3 * BLOCK, "need several blocks to exercise the interior arithmetic");

        // Reference count: the length of the materialised range (itself proven correct above).
        let want = |lo: [Id; 3], hi: [Id; 3]| -> usize { c.range(lo, hi).len() };

        let cases: &[([Id; 3], [Id; 3])] = &[
            // Whole permutation: spans every block (first + many interior + last).
            ([Id::MIN; 3], [Id::MAX; 3]),
            // A bound-subject prefix and a bound subject+predicate prefix.
            ([100, Id::MIN, Id::MIN], [100, Id::MAX, Id::MAX]),
            ([200, 5, Id::MIN], [200, 5, Id::MAX]),
            // A subject that does not exist → empty count.
            ([99999999, Id::MIN, Id::MIN], [99999999, Id::MAX, Id::MAX]),
            // Exact single-row ranges at the start, middle, and end.
            (rows[0], rows[0]),
            (rows[rows.len() / 2], rows[rows.len() / 2]),
            (rows[rows.len() - 1], rows[rows.len() - 1]),
            // A wide interior span that crosses many full blocks (lo/hi land partway into
            // their boundary blocks, so first_count and last_count are both partial).
            (rows[BLOCK / 3], rows[rows.len() - BLOCK / 3]),
            // Two adjacent rows that straddle a block boundary.
            (rows[BLOCK - 1], rows[BLOCK]),
        ];
        for &(lo, hi) in cases {
            assert_eq!(c.count_range(lo, hi), want(lo, hi), "count_range != range().len() for {lo:?}..={hi:?}");
        }

        // An inverted range (lo > hi) and an empty permutation both count 0.
        assert_eq!(c.count_range([10, 0, 0], [1, 0, 0]), 0, "inverted range counts 0");
        let empty = CompressedPerm::encode(&[]);
        assert_eq!(empty.count_range([Id::MIN; 3], [Id::MAX; 3]), 0, "empty perm counts 0");

        // Every single-row range over the whole permutation counts exactly 1 (each row is
        // present once), which independently pins the boundary-block partition_point logic.
        for &r in &rows {
            assert_eq!(c.count_range(r, r), 1, "each present row counts exactly once: {r:?}");
        }
    }

    /// [OPUS-4.8] sq-vkz7 — for a NON-EMPTY perm the STREAMING [`CompressedPermWriter`] must
    /// produce a file BYTE-IDENTICAL to the in-RAM `encode(rows).write_to(..)`. Covers block
    /// boundaries (127/128/129), a partial last block, and a large run. The EMPTY case must
    /// instead produce a zero-byte file (the `save_compressed`/`recompress` empty-perm rule).
    #[cfg(feature = "mmap")]
    #[test]
    fn stream_writer_byte_identical_to_encode_write_to() {
        let tmp = std::env::temp_dir().join(format!("sq_vkz7_stream_{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        for &n in &[0usize, 1, 5, 127, 128, 129, 256, 1000, 5000] {
            let rows = sample(n);

            // Streaming writer to a file.
            let out = tmp.join(format!("perm_{n}.bin"));
            let mut w = CompressedPermWriter::create(&out).unwrap();
            for &r in &rows {
                w.push(r).unwrap();
            }
            w.finish(&out).unwrap();
            let got = std::fs::read(&out).unwrap();

            // The block side file must be cleaned up by finish() in every case.
            let mut side = out.as_os_str().to_owned();
            side.push(".blocks");
            assert!(!std::path::Path::new(&side).exists(), "block side file leaked at n={n}");

            if rows.is_empty() {
                assert!(got.is_empty(), "empty perm must be a zero-byte file, got {} bytes", got.len());
                continue;
            }

            // Reference: in-RAM encode then write_to a byte buffer — must match exactly.
            let mut want = Vec::new();
            CompressedPerm::encode(&rows).write_to(&mut want).unwrap();
            assert_eq!(got, want, "stream vs encode+write_to byte mismatch at n={n}");

            // And it must round-trip back through from_mmap to the same rows.
            let f = std::fs::File::open(&out).unwrap();
            // SAFETY: we own and just wrote this file; nothing else mutates it during the test.
            let map = unsafe { memmap2::Mmap::map(&f) }.unwrap();
            let perm = CompressedPerm::from_mmap(map).unwrap();
            assert_eq!(perm.len(), rows.len(), "len mismatch at n={n}");
            assert_eq!(perm.decode_all(), rows, "decoded rows mismatch at n={n}");
        }
        std::fs::remove_dir_all(&tmp).ok();
    }

    /// [OPUS-4.8] sq-wihld — a HIGH-NDV leading column: many distinct subjects, several with
    /// runs long enough to straddle block boundaries (so a different bound subject's lookup
    /// lands in a block that does not contain it — the exact overlapping-block case a per-block
    /// Bloom filter prunes but the min/max zone map cannot). Sorted + deduplicated, like a real
    /// permutation column.
    #[cfg(feature = "block-bloom")]
    fn high_ndv_sample(n: usize) -> Vec<[Id; 3]> {
        let mut v = Vec::with_capacity(n);
        let mut state = 0x1234_5678u32;
        let mut next = || {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            state
        };
        for _ in 0..n {
            // Wide subject domain (high NDV) but a fraction of subjects repeat a few times,
            // producing runs that cross BLOCK boundaries → overlapping per-block [min,max].
            let s = 1 + (next() % (n as u32 / 2).max(1));
            let reps = 1 + (next() % 4); // 1..=4 rows per (s) on average
            for _ in 0..reps {
                let p = 1 + (next() % 8);
                let o = 1 + (next() % 5_000_000);
                v.push([s, p, o]);
            }
        }
        v.sort_unstable();
        v.dedup();
        v.truncate(n.max(1));
        v
    }

    /// [OPUS-4.8] sq-wihld (survey §A1) — LOAD-BEARING equivalence: with the `block-bloom`
    /// feature ON, `range` over an equality-bound LEADING column must return EXACTLY the rows
    /// a raw binary search returns — the Bloom skip never drops a matching row (zero false
    /// negatives by construction). This is the feature-on-vs-off result-equivalence contract:
    /// the raw binary search is the feature-OFF oracle (it has no Bloom path), so equality here
    /// proves the optimisation is correctness-neutral. We also confirm the density gate built a
    /// filter for this high-NDV column AND that the filter skips at least one candidate block
    /// across the keys — otherwise the test would pass trivially without exercising the skip.
    #[cfg(feature = "block-bloom")]
    #[test]
    fn bloom_range_equals_binary_search_and_skips_blocks() {
        let rows = high_ndv_sample(8000);
        assert!(rows.len() > 4 * BLOCK, "need many blocks to exercise overlap + skipping");
        let c = CompressedPerm::encode(&rows);

        // The density gate must admit this high-NDV leading column, or the skip path is never
        // reached and the test is vacuous.
        assert!(c.has_bloom(), "high-NDV leading column should get a Bloom directory");

        // Feature-OFF oracle: a raw binary-search range over the sorted rows (no Bloom).
        let raw_range = |lo: [Id; 3], hi: [Id; 3]| -> Vec<[Id; 3]> {
            let s = rows.partition_point(|r| *r < lo);
            let e = rows.partition_point(|r| *r <= hi);
            rows[s..e].to_vec()
        };

        // Every DISTINCT leading id present, plus a swathe of ABSENT ids (the false-positive /
        // definite-absent path). For each, the Bloom-enabled `range` must match the oracle.
        let mut present: Vec<Id> = rows.iter().map(|r| r[0]).collect();
        present.dedup();
        let max_present = *present.last().unwrap();

        let mut total_candidates = 0usize;
        let mut total_skipped = 0usize;
        for &k in &present {
            let lo = [k, Id::MIN, Id::MIN];
            let hi = [k, Id::MAX, Id::MAX];
            assert_eq!(c.range(lo, hi), raw_range(lo, hi), "present key {k} range mismatch");
            let (cand, skip) = c.bloom_skip_stats(k);
            total_candidates += cand;
            total_skipped += skip;
            // A present key must NEVER be skipped in the block(s) that hold it: the materialised
            // range is non-empty, so at least one candidate block survived the filter.
            assert!(cand > skip, "present key {k} had all candidate blocks skipped");
        }
        // Absent keys interleaved through and beyond the domain: results are empty, and these
        // are where the Bloom earns its keep (definite-absent → skip without decode).
        for k in (1..=max_present + 16).step_by(7) {
            let lo = [k, Id::MIN, Id::MIN];
            let hi = [k, Id::MAX, Id::MAX];
            assert_eq!(c.range(lo, hi), raw_range(lo, hi), "absent/odd key {k} range mismatch");
        }

        // The optimisation must do REAL work on this high-NDV column: across the present keys
        // at least one candidate block was Bloom-skipped (proves it is engaged, not a no-op).
        // This is a structural assertion, not a performance number.
        assert!(
            total_skipped > 0,
            "Bloom skipped no blocks ({total_skipped}/{total_candidates}) — optimisation never engaged"
        );
    }

    /// [OPUS-4.8] sq-wihld — the Bloom skip must be inert for NON-equality-bound shapes (a full
    /// scan and a multi-key range), and for ALL shapes it must agree with the raw binary search.
    /// Re-runs the existing `range` oracle cases under the feature so the feature-on path is
    /// proven byte-for-byte equivalent to the raw permutation across the same query shapes the
    /// plan-agreement tests rely on.
    #[cfg(feature = "block-bloom")]
    #[test]
    fn bloom_inert_for_non_equality_shapes() {
        let rows = high_ndv_sample(6000);
        let c = CompressedPerm::encode(&rows);
        let raw_range = |lo: [Id; 3], hi: [Id; 3]| -> Vec<[Id; 3]> {
            if lo > hi {
                return Vec::new(); // matches `range`'s inverted-range early-out
            }
            let s = rows.partition_point(|r| *r < lo);
            let e = rows.partition_point(|r| *r <= hi);
            rows[s..e].to_vec()
        };
        let mid = rows[rows.len() / 2][0];
        let cases: &[([Id; 3], [Id; 3])] = &[
            // Full scan (leading column NOT equality-bound).
            ([Id::MIN; 3], [Id::MAX; 3]),
            // A multi-key leading RANGE (lo[0] != hi[0]) — the Bloom path is bypassed.
            ([mid, Id::MIN, Id::MIN], [mid + 50, Id::MAX, Id::MAX]),
            // An empty / inverted range.
            ([Id::MAX, 0, 0], [Id::MIN, 0, 0]),
        ];
        for &(lo, hi) in cases {
            assert_eq!(c.range(lo, hi), raw_range(lo, hi), "non-equality shape {lo:?}..={hi:?}");
        }
    }

    /// [OPUS-4.8] sq-wihld — `decode_all` (whole-permutation iteration) is unaffected by the
    /// Bloom directory: it never consults the filter, so it must reproduce the rows exactly,
    /// feature on or off.
    #[cfg(feature = "block-bloom")]
    #[test]
    fn bloom_decode_all_roundtrips() {
        for &n in &[0usize, 1, 200, 2000, 8000] {
            let rows = high_ndv_sample(n);
            let c = CompressedPerm::encode(&rows);
            assert_eq!(c.decode_all(), rows, "decode_all mismatch at n={n}");
            assert_eq!(c.len(), rows.len(), "len mismatch at n={n}");
        }
    }

    // ===== [FABLE-5] sq-7d3dj.32.2.6 — SPQCPRM2 frame-of-reference spike =====

    /// A sorted permutation whose col2 (object) CLUSTERS within each block — the shape the
    /// frame-of-reference col2-reset targets: many `reset_d1` rows (middle column advances, so
    /// col2 is written as an absolute in SPQCPRM1) whose objects sit near the block's first
    /// object. Long equal-subject runs with a moving predicate produce dense `reset_d1` rows.
    #[cfg(feature = "spqcprm2")]
    fn clustered_col2_sample(n: usize) -> Vec<[Id; 3]> {
        let mut v = Vec::with_capacity(n);
        let mut state = 0x2545_f491u32;
        let mut next = || {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            state
        };
        for _ in 0..n {
            // Few subjects (long equal-col0 runs), moderate predicate churn (drives reset_d1),
            // and objects drawn from a LOCAL window around a per-subject base so a block's col2
            // clusters — exactly where a frame offset beats an absolute id.
            let s = 1 + (next() % (n as u32 / 64).max(1));
            let base = 1 + (s.wrapping_mul(97) % 4_000_000);
            let p = 1 + (next() % 40);
            let o = base + (next() % 4096); // objects within a 4K window of the subject's base
            v.push([s, p, o]);
        }
        v.sort_unstable();
        v.dedup();
        v
    }

    /// [FABLE-5] sq-7d3dj.32.2.6 — DIRECT unit test for `encode_v2`: the V2 in-RAM writer must
    /// round-trip to the exact input rows across block boundaries and both reset shapes, and its
    /// perm carries `Format::V2` (so it decodes through the frame-of-reference reader). This is
    /// the load-bearing correctness invariant of the spike (a lossy FoR would silently corrupt).
    #[cfg(feature = "spqcprm2")]
    #[test]
    fn encode_v2_roundtrips_and_is_v2() {
        for &n in &[0usize, 1, 5, 127, 128, 129, 256, 1000, 5000] {
            let rows = clustered_col2_sample(n);
            let c = CompressedPerm::encode_v2(&rows);
            assert_eq!(c.len(), rows.len(), "v2 len mismatch at n={n}");
            assert_eq!(c.decode_all(), rows, "v2 decode_all mismatch at n={n}");
            assert_eq!(c.format, Format::V2, "encode_v2 must produce a V2 perm at n={n}");
        }
        // Also round-trip the existing clustered/sparse `sample` shape so the FoR is proven on
        // BOTH object distributions (clustered AND sparse), not just its favourable case.
        for &n in &[0usize, 1, 129, 5000] {
            let rows = sample(n);
            assert_eq!(CompressedPerm::encode_v2(&rows).decode_all(), rows, "v2 roundtrip on sample n={n}");
        }
    }

    /// [FABLE-5] sq-7d3dj.32.2.6 — DIFFERENTIAL: SPQCPRM1 and SPQCPRM2 are two encodings of the
    /// SAME logical data. Decoding either must yield the IDENTICAL rows — the frame-of-reference
    /// changes only the col2-reset BYTES, never the decoded value. We also confirm the two
    /// encoders produce DIFFERENT block streams on data with `reset_d1` rows (so the spike is
    /// actually exercising the new path, not silently emitting the V1 stream).
    #[cfg(feature = "spqcprm2")]
    #[test]
    fn spqcprm1_vs_spqcprm2_decode_identical() {
        for &n in &[0usize, 1, 129, 1000, 5000] {
            let rows = clustered_col2_sample(n);
            let v1 = CompressedPerm::encode(&rows);
            let v2 = CompressedPerm::encode_v2(&rows);
            assert_eq!(v1.decode_all(), v2.decode_all(), "v1/v2 decode divergence at n={n}");
            assert_eq!(v1.decode_all(), rows, "v1 decode != input at n={n}");
            assert_eq!(v2.decode_all(), rows, "v2 decode != input at n={n}");
        }
        // On a col2-clustered shape with many reset_d1 rows the two block streams MUST differ
        // (otherwise the FoR path is never taken and the differential is vacuous). We compare
        // the raw encoded bytes of a single well-populated block via the byte buffers.
        let rows = clustered_col2_sample(4000);
        let mut b1 = Vec::new();
        let mut b2 = Vec::new();
        for chunk in rows.chunks(BLOCK) {
            encode_block(chunk, &mut b1);
            encode_block_v2(chunk, &mut b2);
        }
        assert_ne!(b1, b2, "v1 and v2 block streams identical — the reset_d1 FoR path never fired");
    }

    /// [FABLE-5] sq-7d3dj.32.2.6 — a V2 perm's `range` must return EXACTLY the raw binary-search
    /// range, identical to the V1 `range_matches_binary_search` oracle. The frame-of-reference
    /// touches only the decode of col2-reset bytes, so random access is correctness-neutral.
    #[cfg(feature = "spqcprm2")]
    #[test]
    fn v2_range_matches_binary_search() {
        let rows = clustered_col2_sample(5000);
        let c = CompressedPerm::encode_v2(&rows);
        let raw_range = |lo: [Id; 3], hi: [Id; 3]| -> Vec<[Id; 3]> {
            let s = rows.partition_point(|r| *r < lo);
            let e = rows.partition_point(|r| *r <= hi);
            rows[s..e].to_vec()
        };
        let cases: &[([Id; 3], [Id; 3])] = &[
            ([Id::MIN; 3], [Id::MAX; 3]),
            ([rows[rows.len() / 4][0], Id::MIN, Id::MIN], [rows[rows.len() / 4][0], Id::MAX, Id::MAX]),
            ([99_999_999, Id::MIN, Id::MIN], [99_999_999, Id::MAX, Id::MAX]),
            (rows[0], rows[0]),
            (rows[rows.len() / 2], rows[rows.len() / 2]),
            (rows[rows.len() - 1], rows[rows.len() - 1]),
        ];
        for &(lo, hi) in cases {
            assert_eq!(c.range(lo, hi), raw_range(lo, hi), "v2 range mismatch for {lo:?}..={hi:?}");
        }
    }

    /// [FABLE-5] sq-7d3dj.32.2.6 — DIRECT unit test for the zigzag varint round-trip
    /// (`put_zigzag_varint` / `get_zigzag_varint`): the signed frame offset must survive
    /// encode→decode across zero, both signs, and the extremes.
    #[cfg(feature = "spqcprm2")]
    #[test]
    fn zigzag_varint_roundtrips() {
        let cases = [0i64, 1, -1, 2, -2, 127, -128, 300, -300, i32::MAX as i64, i32::MIN as i64, i64::MAX, i64::MIN];
        for &x in &cases {
            let mut buf = Vec::new();
            put_zigzag_varint(&mut buf, x);
            let mut pos = 0;
            assert_eq!(get_zigzag_varint(&buf, &mut pos), x, "zigzag roundtrip failed for {x}");
            assert_eq!(pos, buf.len(), "zigzag reader did not consume all bytes for {x}");
        }
    }

    /// [FABLE-5] sq-7d3dj.32.2.6 — DIRECT unit test that the reserved V2 version marker is the
    /// distinct 8-byte `SPQCPRM2` magic (never equal to the V1 magic), so a future migration can
    /// auto-detect it without a collision.
    #[cfg(all(feature = "spqcprm2", feature = "mmap"))]
    #[test]
    fn file_magic_v2_is_distinct() {
        assert_eq!(&FILE_MAGIC_V2, b"SPQCPRM2");
        assert_ne!(FILE_MAGIC_V2, FILE_MAGIC, "V2 magic must differ from V1");
    }

    /// [FABLE-5] sq-7d3dj.32.2.6 — the MEASUREMENT: block-stream B/triple for SPQCPRM1 vs
    /// SPQCPRM2 at 1M/10M-row scale, summed over the six permutations (the store's real cost).
    /// All numbers are NON-canonical work-box measurements (never a doc/test perf number).
    /// `#[ignore]` so default `cargo test` stays fast; run with
    /// `cargo test -p sparq-core --features spqcprm2 --lib v2_measure -- --ignored --nocapture`.
    #[cfg(feature = "spqcprm2")]
    #[test]
    #[ignore = "spike measurement: run explicitly with --ignored --nocapture (allocates hundreds of MB)"]
    fn v2_measure_bytes_per_triple() {
        let orders: [(&str, [usize; 3]); 6] = [
            ("SPO", [0, 1, 2]), ("SOP", [0, 2, 1]), ("PSO", [1, 0, 2]),
            ("POS", [1, 2, 0]), ("OSP", [2, 0, 1]), ("OPS", [2, 1, 0]),
        ];
        println!("\n===== sq-7d3dj.32.2.6 SPQCPRM1 vs SPQCPRM2 B/triple (NON-canonical work-box) =====");
        for &n in &[1_000_000usize, 10_000_000] {
            let triples = clustered_col2_sample(n);
            let n_tri = triples.len().max(1) as f64;
            let (mut v1_bytes, mut v2_bytes) = (0u64, 0u64);
            for (_, order) in orders {
                let mut rows: Vec<[Id; 3]> =
                    triples.iter().map(|t| [t[order[0]], t[order[1]], t[order[2]]]).collect();
                rows.sort_unstable();
                rows.dedup();
                for chunk in rows.chunks(BLOCK) {
                    let mut b1 = Vec::new();
                    let mut b2 = Vec::new();
                    encode_block(chunk, &mut b1);
                    encode_block_v2(chunk, &mut b2);
                    v1_bytes += b1.len() as u64;
                    v2_bytes += b2.len() as u64;
                }
            }
            let v1_bpt = v1_bytes as f64 / n_tri;
            let v2_bpt = v2_bytes as f64 / n_tri;
            let delta_pct = (v2_bpt - v1_bpt) / v1_bpt * 100.0;
            println!(
                "n={:>10} distinct={:>10}  SPQCPRM1={:>7.3} B/tri  SPQCPRM2={:>7.3} B/tri  delta={:+.2}%",
                n, triples.len(), v1_bpt, v2_bpt, delta_pct
            );
        }
        println!("===== end measurement — verdict in the bead note / PR body =====\n");
    }
}
