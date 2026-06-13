//! [OPUS-4.8] Sparq-side direct HDT decoder — one-shot SPO scan WITHOUT building the
//! upstream `TriplesBitmap` query machinery (wavelet matrix, OP-index, the extra
//! rank/select). See `research/parsing-optimization-plan.md` (levers H1–H4).
//!
//! ## What this avoids vs `hdt::Hdt::read` (the headline, H1+H2)
//!
//! `hdt::TriplesBitmap::read` calls `TriplesBitmap::new`, which — purely to serve
//! pattern/object/predicate *queries* sparq never issues on a bulk load — builds:
//!
//!  * a full `WaveletMatrix<Rank9Sel>` over `sequence_y` (`build_wavelet`), then
//!  * `max_object` worth of `Vec<Vec<u32>>` (the `vec![Vec::with_capacity(4); max_object]`
//!    mega-allocation), one inner Vec per distinct object,
//!  * a per-object `sort_by_cached_key` whose key is `wavelet_y.access(..)`
//!    (cache-hostile, O(log σ) per probe), and
//!  * an OP-index = a `CompactVector` + a `Rank9Sel`-backed `Bitmap`.
//!
//! sparq iterates every triple exactly once in SPO order and never runs a triple
//! pattern, object, or predicate query against the HDT structure, so all of the
//! above is constructed and immediately dropped. This decoder reads the same
//! on-disk bytes (`bitmap_y`, `bitmap_z`, `sequence_y`, `sequence_z`) and walks
//! them sequentially:
//!
//!  * predicate id = `sequence_y.get(pos_y)` — one shift/mask (H2), where the
//!    upstream `SubjectIter` does `wavelet_y.access(pos_y)` (bit-rank ops);
//!  * object id = `sequence_z.get(pos_z)` — one shift/mask;
//!  * the two end-of-run bits come from a *plain bit read* of the raw bitmap
//!    words — we never build `Rank9Sel` over either bitmap, because the full scan
//!    needs sequential `access`, not `rank`/`select`.
//!
//! The byte-level reads of the sequences and bitmaps reuse the upstream
//! `Sequence::read` / our `read_bitmap_words` (which mirrors `Bitmap::read` but
//! keeps the raw words and skips the rank/select build), so every on-disk quirk
//! the upstream relies on — the 3-vbyte PFC preamble, the deliberate vbyte
//! off-by-one, CRC8/CRC32 layout — is handled identically. CRCs are still
//! verified.
//!
//! ## Dictionary (H3 + H4)
//!
//! Each PFC section is decoded **block-sequentially into one reusable buffer**
//! (`decode_section`): front-coding is inherently sequential, so we walk a whole
//! block once — `O(N)` total — instead of the upstream per-id `extract`, which
//! re-walks each block from its start (`O(N · block_size)`) and allocates a fresh
//! `String` per id. Each decoded term is interned into the sparq `Dict` from a
//! borrowed `&str` view of that buffer (H4), so the only allocation per distinct
//! term is the dict's own `Box<str>` (unavoidable — that is its storage); the
//! intermediate owned `String` per id is gone.

use crate::{intern_hdt_term, Error};
use hdt::containers::{ControlInfo, ControlType, Sequence};
use hdt::dict_sect_pfc::DictSectPFC;
use hdt::four_sect_dict::FourSectDict;
use hdt::header::Header;
use sparq_core::dict::{Dict, Id};
use sparq_core::Graph;
use std::io::{BufRead, Read};

/// A raw bitmap: the words exactly as on disk, with no rank/select acceleration.
/// HDT writes the bit vector little-endian, bit `i` living in `words[i >> 6]` at
/// position `i & 63` (LSB-first), so [`bit`](RawBitmap::bit) is a plain shift/mask.
struct RawBitmap {
    words: Vec<u64>,
}

impl RawBitmap {
    /// The end-of-adjacency-run flag at bit position `i`. Equivalent to the
    /// upstream `Bitmap::at_last_sibling(i)` (`Rank9Sel::access`), but without the
    /// rank9 / select-hint structures, which a sequential scan never queries.
    #[inline]
    fn bit(&self, i: usize) -> bool {
        (self.words[i >> 6] >> (i & 63)) & 1 == 1
    }
}

/// Reads one HDT `Bitmap` section, keeping only the raw words and verifying both
/// CRCs — a trimmed copy of `hdt::containers::Bitmap::read` that does NOT build a
/// `Rank9Sel`. The on-disk layout is: type byte (must be 1), vbyte `num_bits`,
/// CRC8 over (type ++ num_bits bytes), the full 64-bit words, the byte-aligned
/// last word, then CRC32-C over the body.
fn read_bitmap_words<R: BufRead>(reader: &mut R) -> Result<RawBitmap, Error> {
    use std::io::{Error as IoError, ErrorKind};

    let mut history: Vec<u8> = Vec::with_capacity(5);

    let mut bitmap_type = [0u8];
    reader.read_exact(&mut bitmap_type)?;
    history.extend_from_slice(&bitmap_type);
    if bitmap_type[0] != 1 {
        return Err(Error::Io(IoError::new(
            ErrorKind::InvalidData,
            format!("unsupported HDT bitmap type {} != 1", bitmap_type[0]),
        )));
    }

    let (num_bits, bytes_read) = read_vbyte(reader)?;
    history.extend_from_slice(&bytes_read);

    let mut crc_code = [0u8];
    reader.read_exact(&mut crc_code)?;
    let crc8 = crc::Crc::<u8>::new(&crc::CRC_8_SMBUS);
    let mut digest = crc8.digest();
    digest.update(&history);
    let crc_calculated = digest.finalize();
    if crc_calculated != crc_code[0] {
        return Err(crc_mismatch(format!("bitmap CRC8 {crc_calculated} != {}", crc_code[0])));
    }

    // All but the last word are full 8-byte words; the last word is byte-aligned.
    let full_byte_amount = if num_bits == 0 { 0 } else { ((num_bits - 1) >> 6) * 8 };
    let mut full_words = vec![0u8; full_byte_amount];
    reader.read_exact(&mut full_words)?;

    let crc32 = crc::Crc::<u32>::new(&crc::CRC_32_ISCSI);
    let mut digest = crc32.digest();
    digest.update(&full_words);

    let mut words: Vec<u64> = Vec::with_capacity(full_byte_amount / 8 + 1);
    for w in full_words.chunks_exact(8) {
        words.push(u64::from_le_bytes(<[u8; 8]>::try_from(w).unwrap()));
    }

    let last_word_bits = if num_bits == 0 { 0 } else { ((num_bits - 1) % 64) + 1 };
    let mut bits_read = 0;
    let mut last_value: u64 = 0;
    while bits_read < last_word_bits {
        let mut buf = [0u8];
        reader.read_exact(&mut buf)?;
        digest.update(&buf);
        last_value |= (buf[0] as u64) << bits_read;
        bits_read += 8;
    }
    words.push(last_value);

    let mut crc_code = [0u8; 4];
    reader.read_exact(&mut crc_code)?;
    let crc_code = u32::from_le_bytes(crc_code);
    let crc_calculated = digest.finalize();
    if crc_calculated != crc_code {
        return Err(crc_mismatch(format!("bitmap CRC32 {crc_calculated} != {crc_code}")));
    }

    Ok(RawBitmap { words })
}

/// Reads a little-endian HDT VByte from a reader, replicating the upstream's
/// deliberate off-by-one (`shift += 7`) so we read the exact same files.
fn read_vbyte<R: Read>(reader: &mut R) -> Result<(usize, Vec<u8>), Error> {
    use std::io::{Error as IoError, ErrorKind};
    let max_bytes = usize::BITS as usize / 7 + 1;
    let mut n: u128 = 0;
    let mut shift = 0u32;
    let mut buf = [0u8];
    let mut bytes_read = Vec::new();
    reader.read_exact(&mut buf)?;
    bytes_read.push(buf[0]);
    while (buf[0] & 0x80) == 0 {
        if bytes_read.len() >= max_bytes {
            return Err(Error::Io(IoError::new(ErrorKind::InvalidData, "VByte does not fit into usize")));
        }
        n |= ((buf[0] & 127) as u128) << shift;
        reader.read_exact(&mut buf)?;
        bytes_read.push(buf[0]);
        shift += 7; // matches upstream's intentional off-by-one
    }
    n |= ((buf[0] & 127) as u128) << shift;
    let n = usize::try_from(n)
        .map_err(|_| Error::Io(IoError::new(ErrorKind::InvalidData, "VByte does not fit into usize")))?;
    Ok((n, bytes_read))
}

fn crc_mismatch(msg: String) -> Error {
    Error::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, msg))
}

/// `decode_vbyte_delta` from the upstream PFC reader (the common-prefix length of a
/// front-coded term): little-endian vbyte with the same intentional off-by-one.
#[inline]
fn decode_vbyte_delta(data: &[u8], offset: usize) -> (usize, usize) {
    let mut n: usize = 0;
    let mut shift: usize = 0;
    let mut byte_amount = 0;
    while (data[offset + byte_amount] & 0x80) == 0 {
        n |= ((data[offset + byte_amount] & 127) as usize) << shift;
        byte_amount += 1;
        shift += 7;
    }
    n |= ((data[offset + byte_amount] & 127) as usize) << shift;
    byte_amount += 1;
    (n, byte_amount)
}

/// Length of the null-terminated string starting at `offset` in `data`.
#[inline]
fn strlen(data: &[u8], offset: usize) -> usize {
    let mut p = offset;
    while p < data.len() && data[p] != 0 {
        p += 1;
    }
    p - offset
}

/// Decodes EVERY string in a PFC section once, block-sequentially, interning each
/// into `dict` and pushing its sparq [`Id`] into `out` in id order (id 1 first).
/// Reuses a single `buf` across the whole section — the structural win of H3+H4
/// over the upstream per-id `extract` (which re-walks each block and allocates a
/// fresh `String` per id).
///
/// PFC layout (per block of `block_size` strings): the first string of the block
/// is stored whole and null-terminated; each later string is
/// `vbyte(common_prefix_len) ++ suffix_bytes ++ 0x00`. Block `b` starts at byte
/// offset `sequence.get(b)`.
fn decode_section(dict: &mut Dict, sect: &DictSectPFC, out: &mut Vec<Id>) -> Result<(), Error> {
    let data: &[u8] = &sect.packed_data;
    let n = sect.num_strings;
    let block_size = sect.block_size.max(1);
    let mut buf: Vec<u8> = Vec::with_capacity(64);

    let mut produced = 0usize;
    let num_blocks = n.div_ceil(block_size);
    for block in 0..num_blocks {
        let mut pos = sect.sequence.get(block);
        // First string of the block is stored whole.
        let slen = strlen(data, pos);
        buf.clear();
        buf.extend_from_slice(&data[pos..pos + slen]);
        pos += slen + 1; // skip the null terminator
        out.push(intern_buf(dict, &buf)?);
        produced += 1;
        if produced == n {
            break;
        }
        // Remaining strings in the block are front-coded against the running buffer.
        let in_block = (n - produced).min(block_size - 1);
        for _ in 0..in_block {
            let (delta, vbyte_bytes) = decode_vbyte_delta(data, pos);
            pos += vbyte_bytes;
            let slen = strlen(data, pos);
            buf.truncate(delta);
            buf.extend_from_slice(&data[pos..pos + slen]);
            pos += slen + 1;
            out.push(intern_buf(dict, &buf)?);
        }
        produced += in_block;
        if produced >= n {
            break;
        }
    }
    Ok(())
}

/// Interns one decoded term, given as raw (front-coded-reconstructed) UTF-8 bytes.
#[inline]
fn intern_buf(dict: &mut Dict, buf: &[u8]) -> Result<Id, Error> {
    let s = std::str::from_utf8(buf)
        .map_err(|e| Error::Term(format!("invalid UTF-8 in HDT dictionary term: {e}")))?;
    intern_hdt_term(dict, s)
}

/// Direct HDT -> sparq [`Graph`] decode (H1–H4). Reads control info + header +
/// the four-section PFC dictionary (CRC-validated via the upstream reader), then
/// decodes the triples section's bitmaps/sequences directly and emits SPO
/// id-triples — never constructing the upstream wavelet matrix / OP-index.
pub fn graph_from_reader<R: BufRead>(mut reader: R) -> Result<Graph, Error> {
    // Global control info + header (header body is not needed for the graph).
    ControlInfo::read(&mut reader).map_err(hdt::hdt::Error::from)?;
    Header::read(&mut reader).map_err(hdt::hdt::Error::from)?;

    // Four-section dictionary: reuse the upstream reader (keeps packed PFC bytes +
    // the offset sequence and validates every section's CRC), then decode + intern
    // block-sequentially ourselves.
    let dict_hdt: FourSectDict = FourSectDict::read(&mut reader)
        .map_err(hdt::hdt::Error::from)?
        .validate()
        .map_err(hdt::hdt::Error::from)?;

    let n_shared = dict_hdt.shared.num_strings;
    let n_subj_only = dict_hdt.subjects.num_strings;
    let n_obj_only = dict_hdt.objects.num_strings;
    let n_pred = dict_hdt.predicates.num_strings;

    let mut dict = Dict::new();

    // sparq Id for each HDT id, decoded once per distinct term, in id order.
    //  shared[k]    -> shared_ids[k]            (k = id - 1, used by both S and O)
    //  subject only -> subj_only_ids[k]         (k = id - n_shared - 1)
    //  object only  -> obj_only_ids[k]          (k = id - n_shared - 1)
    //  predicate    -> pred_ids[k]              (k = id - 1)
    let mut shared_ids: Vec<Id> = Vec::with_capacity(n_shared);
    decode_section(&mut dict, &dict_hdt.shared, &mut shared_ids)?;
    let mut subj_only_ids: Vec<Id> = Vec::with_capacity(n_subj_only);
    decode_section(&mut dict, &dict_hdt.subjects, &mut subj_only_ids)?;
    let mut obj_only_ids: Vec<Id> = Vec::with_capacity(n_obj_only);
    decode_section(&mut dict, &dict_hdt.objects, &mut obj_only_ids)?;
    let mut pred_ids: Vec<Id> = Vec::with_capacity(n_pred);
    decode_section(&mut dict, &dict_hdt.predicates, &mut pred_ids)?;

    // Map an HDT subject/object id (1-based, shared section first) to its sparq Id.
    let map_so = |id: usize, shared: &[Id], only: &[Id]| -> Id {
        if id <= n_shared {
            shared[id - 1]
        } else {
            only[id - n_shared - 1]
        }
    };

    // --- Triples section (H1+H2): read directly, never build TriplesBitmap. ---
    let triples_ci = ControlInfo::read(&mut reader).map_err(hdt::hdt::Error::from)?;
    if triples_ci.control_type != ControlType::Triples {
        return Err(Error::Term(format!("expected triples control info, got {:?}", triples_ci.control_type)));
    }
    match triples_ci.format.as_str() {
        "<http://purl.org/HDT/hdt#triplesBitmap>" => {}
        "<http://purl.org/HDT/hdt#triplesList>" => {
            return Err(Error::Term("triples list format is not supported".into()))
        }
        f => return Err(Error::Term(format!("unknown triples format {f}"))),
    }
    let order = triples_ci.get("order").and_then(|v| v.parse::<u32>().ok());
    if order != Some(1) {
        // Order 1 == SPO. Upstream only correctly supports SPO; we mirror that.
        return Err(Error::Term(format!("unsupported HDT triples order {order:?} (only SPO is supported)")));
    }

    // Same on-disk order as `TriplesBitmap::read`: bitmap_y, bitmap_z, sequence_y,
    // sequence_z. The bitmaps are read raw (no rank/select); the sequences via the
    // upstream reader (CRC-validated branchless shift/mask access).
    let bitmap_y = read_bitmap_words(&mut reader)?;
    let bitmap_z = read_bitmap_words(&mut reader)?;
    let sequence_y = Sequence::read(&mut reader).map_err(|e| crc_mismatch(format!("sequence_y: {e}")))?;
    let sequence_z = Sequence::read(&mut reader).map_err(|e| crc_mismatch(format!("sequence_z: {e}")))?;

    let num_triples = sequence_z.entries;
    let mut triples: Vec<[Id; 3]> = Vec::with_capacity(num_triples);

    // Sequential SPO walk, mirroring `SubjectIter::next` but with predicates from
    // `sequence_y` (H2) and end-of-run bits from raw bitmap reads (no wavelet, no
    // rank/select). `x` is the 1-based subject id; `pos_y`/`pos_z` advance in
    // lock-step with the adjacency runs.
    let max_y = sequence_y.entries;
    let max_z = sequence_z.entries;
    let mut x: usize = 1;
    let mut pos_y: usize = 0;
    let mut pos_z: usize = 0;
    while pos_y < max_y && pos_z < max_z {
        let p = sequence_y.get(pos_y); // predicate id (H2)
        let o = sequence_z.get(pos_z); // object id

        let sid = map_so(x, &shared_ids, &subj_only_ids);
        let pid = pred_ids[p - 1];
        let oid = map_so(o, &shared_ids, &obj_only_ids);
        triples.push([sid, pid, oid]);

        if bitmap_z.bit(pos_z) {
            // last object for this (subject, predicate) run
            if bitmap_y.bit(pos_y) {
                // last predicate for this subject
                x += 1;
            }
            pos_y += 1;
        }
        pos_z += 1;
    }

    Ok(Graph::from_parts(dict, triples))
}
