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
//! ## Dictionary (H3 + H4 + H6)
//!
//! Each PFC section is decoded **block-sequentially into one reusable buffer**
//! (`decode_section`): front-coding is inherently sequential, so we walk a whole
//! block once — `O(N)` total — instead of the upstream per-id `extract`, which
//! re-walks each block from its start (`O(N · block_size)`) and allocates a fresh
//! `String` per id. Each decoded term is interned into the sparq `Dict` from a
//! borrowed `&str` view of that buffer (H4), so the only allocation per distinct
//! term is the dict's own `Box<str>` (unavoidable — that is its storage); the
//! intermediate owned `String` per id is gone.
//!
//! [OPUS-4.8] sq-s506 (lever H6): a `FourSectDict` holds four INDEPENDENT PFC blobs
//! (shared / subjects / objects / predicates). Decoding+interning is purely CPU after
//! the (sequential) read, and the four sections share no state, so we decode all four
//! **concurrently on the rayon pool** — each into its OWN partial `Dict` (front-coding
//! within a section stays sequential; only the four sections run in parallel) — then
//! merge the partials into the final dict via [`Dict::merge_remap`] in the FIXED
//! section order (shared, subjects, objects, predicates). Because `merge_remap` walks a
//! partial's term arena in first-seen (= id) order and re-interns into the target, and
//! the target is fed the four partials in that exact order, the final dict's term arena
//! — and therefore every assigned sparq `Id` and every downstream byte — is BIT-FOR-BIT
//! identical to the old sequential `decode_section`-into-one-dict path. CRITICAL: the
//! `shared` section feeds BOTH the subject (S) and object (O) id spaces; merging it
//! FIRST (before subjects/objects) reproduces HDT's mandated id layout (shared ids
//! first, then section-local ids), so an HDT id resolves to the same sparq `Id` whether
//! it is referenced as S or O.

use crate::{intern_hdt_term, Error};
use hdt::containers::{ControlInfo, ControlType, Sequence};
use hdt::dict_sect_pfc::DictSectPFC;
use hdt::four_sect_dict::FourSectDict;
use hdt::header::Header;
use sparq_core::dict::{is_inline, Dict, Id};
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
    // [OPUS-4.8] sq-tzwa (OOM-DoS hardening): `num_bits` is a VByte read straight from an
    // attacker-controlled `.hdt` file, and `full_byte_amount` is derived from it with NO check
    // against the bytes remaining in the stream. The previous `vec![0u8; full_byte_amount]`
    // ALLOCATED (and zeroed) the full declared size BEFORE the `read_exact` that would discover
    // the bytes don't exist — so a hostile `num_bits ≈ 2^60` forces a multi-exabyte allocation
    // → an uncatchable OOM abort (DoS). The reader is a generic streaming `BufRead`, so there is
    // no cheap "remaining bytes" to bound against up front (sparq-core's slice-backed loaders
    // can; this one can't). Instead read THROUGH a `take(full_byte_amount)` adaptor with
    // `read_to_end`, which grows the buffer only as bytes ACTUALLY arrive (doubling, never a
    // single declared-size pre-allocation), then verify the full count was delivered — failing
    // closed exactly as the prior `read_exact` did, but without ever trusting the declared
    // length enough to pre-allocate it. Same fail-closed intent as sq-f5jh's bounded reads in
    // sparq-core (`read_str_bounded` / `load_pred_stats`).
    let mut full_words: Vec<u8> = Vec::new();
    let read = reader
        .take(full_byte_amount as u64)
        .read_to_end(&mut full_words)?;
    if read != full_byte_amount {
        return Err(Error::Io(IoError::new(
            ErrorKind::UnexpectedEof,
            format!("HDT bitmap body truncated: declared {full_byte_amount} bytes, got {read}"),
        )));
    }

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

/// Decodes EVERY string in a PFC section once, block-sequentially, into a FRESH
/// partial [`Dict`], returning that partial dict plus a per-HDT-id vector of the
/// term's PARTIAL-LOCAL sparq [`Id`] in id order (id 1 first). Reuses a single `buf`
/// across the whole section — the structural win of H3+H4 over the upstream per-id
/// `extract` (which re-walks each block and allocates a fresh `String` per id).
///
/// [OPUS-4.8] sq-s506 (H6): decoding into a SELF-CONTAINED partial dict (rather than a
/// shared `&mut Dict`) is what lets the four sections run concurrently — they touch no
/// shared state. The caller merges the partials into the final dict in the fixed
/// section order via [`Dict::merge_remap`] and composes the returned local ids with
/// that merge's remap, reproducing the sequential id assignment exactly (see the module
/// docs). The returned local ids may be INLINE ids (e.g. canonical small `xsd:integer`
/// literals, which are never stored in the partial arena); the caller passes those
/// through `merge_remap` unchanged.
///
/// PFC layout (per block of `block_size` strings): the first string of the block
/// is stored whole and null-terminated; each later string is
/// `vbyte(common_prefix_len) ++ suffix_bytes ++ 0x00`. Block `b` starts at byte
/// offset `sequence.get(b)`.
fn decode_section(sect: &DictSectPFC) -> Result<(Dict, Vec<Id>), Error> {
    let data: &[u8] = &sect.packed_data;
    let n = sect.num_strings;
    let block_size = sect.block_size.max(1);
    let mut buf: Vec<u8> = Vec::with_capacity(64);
    // [OPUS-4.8] sq-s506: pre-size the partial dict from the known section term count
    // (`sect.num_strings`) so its term arena + lookup table are reserved up front instead of
    // repeatedly growing/rehashing while we intern. Some terms collapse to INLINE ids (never
    // stored in the arena), so `n` is an UPPER bound — `with_capacity` only reserves, it does
    // not over-allocate the stored set.
    //
    // [OPUS-4.8] sq-tzwa (OOM-DoS hardening): `sect.num_strings` is a VByte read straight from
    // an attacker-controlled `.hdt` file and is parsed INDEPENDENTLY of `packed_data`'s length
    // by upstream `DictSectPFC::read` (num_strings and packed_length are separate VBytes whose
    // CRC8 does not relate them). A hostile section can therefore declare `num_strings = 2^60`
    // with a tiny `packed_data`, turning the unbounded `Dict::with_capacity(n)` /
    // `Vec::with_capacity(n)` below into a multi-exabyte allocation → an OOM abort (DoS) BEFORE
    // the decode loop ever touches `packed_data`. Cap the reservation against the bytes that
    // actually exist: every decoded string occupies AT LEAST its 1-byte null terminator in
    // `packed_data`, so a section can hold no more than `packed_data.len()` strings. This caps
    // only the RESERVATION (the loop still honours `n` and a genuinely over-stated count fails
    // closed when the lone bytes are exhausted). Mirrors sq-f5jh's bounded-read discipline in
    // sparq-core (`read_str_bounded` / `load_pred_stats`).
    let cap = n.min(data.len());
    let mut dict = Dict::with_capacity(cap);
    let mut out: Vec<Id> = Vec::with_capacity(cap);

    let mut produced = 0usize;
    let num_blocks = n.div_ceil(block_size);
    for block in 0..num_blocks {
        let mut pos = sect.sequence.get(block);
        // First string of the block is stored whole.
        let slen = strlen(data, pos);
        buf.clear();
        buf.extend_from_slice(&data[pos..pos + slen]);
        pos += slen + 1; // skip the null terminator
        out.push(intern_buf(&mut dict, &buf)?);
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
            out.push(intern_buf(&mut dict, &buf)?);
        }
        produced += in_block;
        if produced >= n {
            break;
        }
    }
    Ok((dict, out))
}

/// Merges a section's partial dict into the final `dict` (in the caller's fixed
/// section order) and rewrites its per-HDT-id local ids to FINAL sparq ids in place.
///
/// [OPUS-4.8] sq-s506: `merge_remap` walks `partial`'s term arena in first-seen (= id)
/// order and re-interns each into `dict`, so calling it on the four partials in section
/// order reproduces the sequential single-dict interning byte-for-byte. The remap it
/// returns is indexed by the partial's NON-inline ids; inline local ids (which never
/// entered the arena) are global and pass through unchanged.
fn merge_section(dict: &mut Dict, partial: &Dict, local_ids: &mut [Id]) {
    let remap = dict.merge_remap(partial);
    for id in local_ids.iter_mut() {
        if !is_inline(*id) {
            *id = remap[(*id - 1) as usize];
        }
    }
}

/// Interns one decoded term, given as raw (front-coded-reconstructed) UTF-8 bytes.
#[inline]
fn intern_buf(dict: &mut Dict, buf: &[u8]) -> Result<Id, Error> {
    let s = std::str::from_utf8(buf)
        .map_err(|e| Error::Term(format!("invalid UTF-8 in HDT dictionary term: {e}")))?;
    intern_hdt_term(dict, s)
}

/// The sparq `Dict` plus the per-HDT-id sparq [`Id`] for each of the four sections —
/// the result of decoding a `FourSectDict`. `shared` is indexed by `(hdt_id - 1)` and is
/// shared by the S and O id spaces; `subj_only` / `obj_only` are indexed by
/// `(hdt_id - n_shared - 1)`; `pred` by `(hdt_id - 1)`.
struct DictDecode {
    dict: Dict,
    shared: Vec<Id>,
    subj_only: Vec<Id>,
    obj_only: Vec<Id>,
    pred: Vec<Id>,
}

/// Decodes a `FourSectDict` into one sparq `Dict` plus per-section id vectors.
///
/// [OPUS-4.8] sq-s506 (lever H6): the four INDEPENDENT PFC blobs are decoded
/// CONCURRENTLY on the rayon pool (or straight-line via the single-thread fast path when the
/// pool has `<= 1` thread), each into its OWN partial dict (no shared state),
/// then merged into the final dict in the FIXED section order (shared, subjects, objects,
/// predicates). `merge_remap` re-interns each partial's terms in arena (= first-seen)
/// order, so feeding the partials in this order reproduces the old sequential single-dict
/// interning BIT-FOR-BIT — and, in particular, the `shared` section is merged FIRST so
/// its terms keep the lowest ids and an HDT id resolves to the same sparq Id whether it
/// is referenced as a subject or as an object.
fn decode_dict(dict_hdt: &FourSectDict) -> Result<DictDecode, Error> {
    let (sh, su, ob, pr) = (
        &dict_hdt.shared,
        &dict_hdt.subjects,
        &dict_hdt.objects,
        &dict_hdt.predicates,
    );
    // [OPUS-4.8] sq-s506: when the rayon pool is effectively single-threaded
    // (`current_num_threads() <= 1`, e.g. `RAYON_NUM_THREADS=1`) the two nested `join`s buy
    // no parallelism — only join/closure overhead — so decode the four sections straight-line
    // instead. The rayon path is kept for genuinely multi-threaded pools. BOTH paths decode
    // the SAME four sections into their OWN partial dicts and feed them to the identical
    // section-order merge below, so the id layout is bit-for-bit identical either way (guarded
    // by the `parallel_matches_sequential_reference` / `shared_section_so_id_layout` tests).
    let (
        shared_partial,
        mut shared,
        subj_partial,
        mut subj_only,
        obj_partial,
        mut obj_only,
        pred_partial,
        mut pred,
    ) = if rayon::current_num_threads() <= 1 {
        // Single-threaded fast path: no join overhead, same section order.
        let (shared_partial, shared) = decode_section(sh)?;
        let (subj_partial, subj_only) = decode_section(su)?;
        let (obj_partial, obj_only) = decode_section(ob)?;
        let (pred_partial, pred) = decode_section(pr)?;
        (
            shared_partial,
            shared,
            subj_partial,
            subj_only,
            obj_partial,
            obj_only,
            pred_partial,
            pred,
        )
    } else {
        // Two nested `join`s fan the four CPU-bound section decodes across the pool; each
        // returns its `Result<(Dict, Vec<Id>), Error>` so any malformed section is surfaced
        // (handled with `?` below), never swallowed.
        let ((shared_res, subj_res), (obj_res, pred_res)) = rayon::join(
            || rayon::join(|| decode_section(sh), || decode_section(su)),
            || rayon::join(|| decode_section(ob), || decode_section(pr)),
        );
        let (shared_partial, shared) = shared_res?;
        let (subj_partial, subj_only) = subj_res?;
        let (obj_partial, obj_only) = obj_res?;
        let (pred_partial, pred) = pred_res?;
        (
            shared_partial,
            shared,
            subj_partial,
            subj_only,
            obj_partial,
            obj_only,
            pred_partial,
            pred,
        )
    };
    debug_assert_eq!(shared.len(), dict_hdt.shared.num_strings);
    debug_assert_eq!(subj_only.len(), dict_hdt.subjects.num_strings);
    debug_assert_eq!(obj_only.len(), dict_hdt.objects.num_strings);
    debug_assert_eq!(pred.len(), dict_hdt.predicates.num_strings);

    // Merge the partials into the final dict IN SECTION ORDER (shared, subjects, objects,
    // predicates) — identical to the sequential decode's interning order — and rewrite
    // each section's local ids to final ids.
    let mut dict = Dict::new();
    merge_section(&mut dict, &shared_partial, &mut shared);
    merge_section(&mut dict, &subj_partial, &mut subj_only);
    merge_section(&mut dict, &obj_partial, &mut obj_only);
    merge_section(&mut dict, &pred_partial, &mut pred);
    Ok(DictDecode {
        dict,
        shared,
        subj_only,
        obj_only,
        pred,
    })
}

/// Direct HDT -> sparq [`Graph`] decode (H1–H4 + H6). Reads control info + header +
/// the four-section PFC dictionary (CRC-validated via the upstream reader), then
/// decodes the triples section's bitmaps/sequences directly and emits SPO
/// id-triples — never constructing the upstream wavelet matrix / OP-index.
pub fn graph_from_reader<R: BufRead>(mut reader: R) -> Result<Graph, Error> {
    // Global control info + header (header body is not needed for the graph).
    ControlInfo::read(&mut reader).map_err(hdt::hdt::Error::from)?;
    Header::read(&mut reader).map_err(hdt::hdt::Error::from)?;

    // Four-section dictionary: reuse the upstream reader (keeps packed PFC bytes +
    // the offset sequence and validates every section's CRC), then decode + intern
    // ourselves (the four PFC sections concurrently — see `decode_dict`).
    let dict_hdt: FourSectDict = FourSectDict::read(&mut reader)
        .map_err(hdt::hdt::Error::from)?
        .validate()
        .map_err(hdt::hdt::Error::from)?;

    let n_shared = dict_hdt.shared.num_strings;
    let DictDecode {
        dict,
        shared: shared_ids,
        subj_only: subj_only_ids,
        obj_only: obj_only_ids,
        pred: pred_ids,
    } = decode_dict(&dict_hdt)?;

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
    // [OPUS-4.8] sq-tzwa (OOM-DoS hardening): `sequence_z.entries` is a count from the
    // attacker-controlled file. By here `Sequence::read` has already CONSUMED + CRC-validated
    // the sequence's backing words, so `entries` is bounded-by-construction against bytes that
    // genuinely arrived — but cap the `Vec::with_capacity` reservation against that backing
    // anyway so a malformed `entries` over-stating its own sequence can never drive an
    // unbounded `[Id; 3]` (12 bytes/entry) pre-allocation. The tight upper bound is
    // `data.len()` 64-bit words × 64 bits ÷ 1-bit-minimum-per-entry = `data.len() * 64`
    // entries; capping there can never shrink a valid file's reservation. Mirrors sq-f5jh's
    // `stats.reserve(n.min(max_records))` in sparq-core's `load_pred_stats`.
    let cap_triples = num_triples.min(sequence_z.data.len().saturating_mul(64));
    let mut triples: Vec<[Id; 3]> = Vec::with_capacity(cap_triples);

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

#[cfg(test)]
mod tests {
    //! [OPUS-4.8] sq-s506 (H6) decode-equivalence + shared-section id-layout tests.
    //!
    //! These guard the one thing the parallel-decode refactor could silently break: the
    //! HDT id layout. The `shared` section feeds BOTH the S and O id spaces and must keep
    //! the LOWEST ids; subject-only / object-only terms must get ids OFFSET past the
    //! shared block. If the parallel merge ever stopped merging `shared` first (or merged
    //! a section out of order), a SHARED term would resolve to different sparq ids as S vs
    //! O and every triple would mis-resolve — so we assert that invariant directly, plus a
    //! full triple-for-triple equivalence against an independent SEQUENTIAL reference
    //! decode of the same archive.
    use super::*;
    use std::collections::BTreeSet;
    use std::path::{Path, PathBuf};

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
    }

    /// Reads the fixture's `FourSectDict` (the upstream reader; ground truth for the
    /// section boundaries and the term each HDT id denotes).
    fn read_fixture_dict(bytes: &[u8]) -> Option<FourSectDict> {
        let mut r = std::io::Cursor::new(bytes);
        ControlInfo::read(&mut r).ok()?;
        Header::read(&mut r).ok()?;
        FourSectDict::read(&mut r).ok()?.validate().ok()
    }

    fn fixture_bytes() -> Option<Vec<u8>> {
        match std::fs::read(fixture("snikmeta.hdt")) {
            Ok(b) => Some(b),
            Err(_) => {
                eprintln!("skipping: fixture snikmeta.hdt absent");
                None
            }
        }
    }

    /// An INDEPENDENT, deliberately simple SEQUENTIAL reference: decode the four PFC
    /// sections one-after-another into a SINGLE shared dict (the pre-sq-s506 algorithm),
    /// reproducing the id layout the format mandates. Cross-checks that the parallel
    /// `decode_dict` / `graph_from_reader` assigns the identical sparq ids.
    fn seq_ref_section(dict: &mut Dict, sect: &DictSectPFC, out: &mut Vec<Id>) {
        // Decode the section in isolation, then `merge_remap` it into the shared dict —
        // strictly serially and one section at a time.
        let (partial, mut local) = decode_section(sect).expect("reference decode");
        let remap = dict.merge_remap(&partial);
        for id in local.iter_mut() {
            if !is_inline(*id) {
                *id = remap[(*id - 1) as usize];
            }
        }
        out.extend_from_slice(&local);
    }

    /// Render a graph's triples to a sorted set of string triples — the resolved-term
    /// view, so any id-layout bug shows up as a wrong term.
    fn term_triple_set(g: &Graph) -> BTreeSet<[String; 3]> {
        g.iter_ids()
            .map(|spo| {
                [
                    g.dict.term(spo[0]).to_string(),
                    g.dict.term(spo[1]).to_string(),
                    g.dict.term(spo[2]).to_string(),
                ]
            })
            .collect()
    }

    /// The term `intern_hdt_term` produces for an upstream term string, rendered back to
    /// its N-Triples form (handles inline ids: we read back the RETURNED id, not id 1).
    fn rendered_term(s: &str) -> String {
        let mut probe = Dict::new();
        let id = intern_hdt_term(&mut probe, s).expect("upstream term must parse");
        probe.term(id).to_string()
    }

    /// The shared-section S/O offset invariant — the one thing this refactor could break.
    ///
    /// Decode the dictionary via the (parallel) `decode_dict` and assert, against the
    /// upstream reader as ground truth, that:
    ///  * the SHARED block is laid out FIRST — every shared HDT id `k` maps (via the S id
    ///    space) to the SAME sparq id as it does via the O id space, and that sparq id is
    ///    exactly `shared[k - 1]` for both;
    ///  * subject-only and object-only HDT ids take the OFFSET path past the shared block
    ///    (`only[id - n_shared - 1]`) and resolve to the term the upstream reader gives
    ///    for that id — i.e. they are NOT mis-read as shared ids.
    #[test]
    fn shared_section_so_id_layout() {
        let Some(bytes) = fixture_bytes() else { return };
        let dict_hdt = read_fixture_dict(&bytes).expect("fixture FourSectDict must parse");
        let n_shared = dict_hdt.shared.num_strings;
        let n_subj_only = dict_hdt.subjects.num_strings;
        let n_obj_only = dict_hdt.objects.num_strings;
        assert!(n_shared > 0, "fixture must exercise the shared section");
        assert!(n_subj_only > 0 || n_obj_only > 0, "fixture must exercise an offset section");

        let dd = decode_dict(&dict_hdt).expect("parallel dict decode");

        // The S-view map and the O-view map BOTH start with the shared block.
        let map_s = |id: usize| {
            if id <= n_shared {
                dd.shared[id - 1]
            } else {
                dd.subj_only[id - n_shared - 1]
            }
        };
        let map_o = |id: usize| {
            if id <= n_shared {
                dd.shared[id - 1]
            } else {
                dd.obj_only[id - n_shared - 1]
            }
        };

        // Shared ids: identical sparq id whether referenced as S or O, and it IS the
        // shared-block entry (the lowest-id block). This is the bug the brief warns about.
        for k in 1..=n_shared {
            assert_eq!(map_s(k), map_o(k), "shared HDT id {k}: S and O views must agree");
            assert_eq!(map_s(k), dd.shared[k - 1], "shared HDT id {k}: must use the shared block");
            let as_subj = dict_hdt.id_to_string(k, hdt::IdKind::Subject).unwrap();
            let as_obj = dict_hdt.id_to_string(k, hdt::IdKind::Object).unwrap();
            assert_eq!(as_subj, as_obj, "shared HDT id {k}: HDT S/O views must be one term");
        }

        // Subject-only ids take the offset path and resolve to the upstream subject term.
        for off in 0..n_subj_only {
            let hdt_id = n_shared + 1 + off; // first subject-only id is n_shared + 1
            let sparq_id = map_s(hdt_id);
            assert_eq!(sparq_id, dd.subj_only[off], "subject-only id {hdt_id}: offset path");
            let want = dict_hdt.id_to_string(hdt_id, hdt::IdKind::Subject).unwrap();
            assert_eq!(dd.dict.term(sparq_id).to_string(), rendered_term(&want), "subj-only {hdt_id} term");
        }

        // Object-only ids likewise take the offset path against the OBJECT id space.
        for off in 0..n_obj_only {
            let hdt_id = n_shared + 1 + off;
            let sparq_id = map_o(hdt_id);
            assert_eq!(sparq_id, dd.obj_only[off], "object-only id {hdt_id}: offset path");
            let want = dict_hdt.id_to_string(hdt_id, hdt::IdKind::Object).unwrap();
            assert_eq!(dd.dict.term(sparq_id).to_string(), rendered_term(&want), "obj-only {hdt_id} term");
        }
    }

    /// Full equivalence: the parallel `graph_from_reader` must produce the identical
    /// resolved-triple set as the independent sequential reference decode — a single
    /// shared-section offset bug would change which terms triples resolve to and fail here.
    #[test]
    fn parallel_matches_sequential_reference() {
        let Some(bytes) = fixture_bytes() else { return };

        let parallel = graph_from_reader(std::io::Cursor::new(&bytes)).expect("parallel decode");
        let sequential = seq_ref_graph(&bytes).expect("sequential reference decode");

        assert_eq!(parallel.len(), sequential.len(), "triple counts must match");
        assert_eq!(
            term_triple_set(&parallel),
            term_triple_set(&sequential),
            "parallel and sequential decode must resolve to the identical triple set"
        );
    }

    /// Sequential reference decoder: the dict half done serially (shared, subjects,
    /// objects, predicates into one dict), with the production triples walk re-derived
    /// here so it is a fully independent oracle (it shares no code with the parallel path
    /// beyond the byte-level section readers).
    fn seq_ref_graph(bytes: &[u8]) -> Result<Graph, Error> {
        let mut r = std::io::Cursor::new(bytes);
        ControlInfo::read(&mut r).map_err(hdt::hdt::Error::from)?;
        Header::read(&mut r).map_err(hdt::hdt::Error::from)?;
        let dict_hdt = FourSectDict::read(&mut r)
            .map_err(hdt::hdt::Error::from)?
            .validate()
            .map_err(hdt::hdt::Error::from)?;
        let n_shared = dict_hdt.shared.num_strings;
        let mut dict = Dict::new();
        let mut shared_ids = Vec::new();
        seq_ref_section(&mut dict, &dict_hdt.shared, &mut shared_ids);
        let mut subj_ids = Vec::new();
        seq_ref_section(&mut dict, &dict_hdt.subjects, &mut subj_ids);
        let mut obj_ids = Vec::new();
        seq_ref_section(&mut dict, &dict_hdt.objects, &mut obj_ids);
        let mut pred_ids = Vec::new();
        seq_ref_section(&mut dict, &dict_hdt.predicates, &mut pred_ids);

        let map_so = |id: usize, shared: &[Id], only: &[Id]| -> Id {
            if id <= n_shared {
                shared[id - 1]
            } else {
                only[id - n_shared - 1]
            }
        };

        let triples_ci = ControlInfo::read(&mut r).map_err(hdt::hdt::Error::from)?;
        assert_eq!(triples_ci.control_type, ControlType::Triples);
        let bitmap_y = read_bitmap_words(&mut r)?;
        let bitmap_z = read_bitmap_words(&mut r)?;
        let sequence_y = Sequence::read(&mut r).map_err(|e| crc_mismatch(format!("sequence_y: {e}")))?;
        let sequence_z = Sequence::read(&mut r).map_err(|e| crc_mismatch(format!("sequence_z: {e}")))?;
        let mut triples = Vec::new();
        let (max_y, max_z) = (sequence_y.entries, sequence_z.entries);
        let (mut x, mut pos_y, mut pos_z) = (1usize, 0usize, 0usize);
        while pos_y < max_y && pos_z < max_z {
            let p = sequence_y.get(pos_y);
            let o = sequence_z.get(pos_z);
            let sid = map_so(x, &shared_ids, &subj_ids);
            let pid = pred_ids[p - 1];
            let oid = map_so(o, &shared_ids, &obj_ids);
            triples.push([sid, pid, oid]);
            if bitmap_z.bit(pos_z) {
                if bitmap_y.bit(pos_y) {
                    x += 1;
                }
                pos_y += 1;
            }
            pos_z += 1;
        }
        Ok(Graph::from_parts(dict, triples))
    }

    // ───────────────────── [OPUS-4.8] sq-tzwa OOM-DoS hardening oracles ─────────────────────
    //
    // CORRUPTION ORACLE for the unbounded pre-allocation guards. Each test crafts a section /
    // bitmap header that declares a HUGE element count (the value an attacker plants in a
    // hostile `.hdt` to weaponise `Vec::with_capacity(n)` / `vec![0u8; n]` into a multi-exabyte
    // allocation → OOM abort) but supplies only a few real bytes, and asserts the loader fails
    // CLOSED with an `Err` instead of attempting the allocation. A regression that removed the
    // cap would NOT return `Err` here — it would OOM-abort the whole test process, the very DoS
    // we are guarding against.

    /// Encodes a `usize` as an HDT little-endian VByte with the upstream's deliberate off-by-one
    /// (`shift += 7`, high bit `0x80` marks the LAST byte) — the exact inverse of [`read_vbyte`].
    fn encode_vbyte_off_by_one(mut n: usize) -> Vec<u8> {
        let mut out = Vec::new();
        loop {
            let byte = (n & 0x7f) as u8;
            n >>= 7;
            if n == 0 {
                out.push(byte | 0x80); // last byte: high bit set
                break;
            }
            out.push(byte); // continuation: high bit clear
        }
        out
    }

    /// Site 2 (`read_bitmap_words`): a CRC8-valid bitmap header whose `num_bits` is enormous but
    /// whose body is truncated. The old `vec![0u8; full_byte_amount]` would try to allocate
    /// `num_bits/8 ≈ 2^57` zeroed bytes BEFORE `read_exact` could fail; the bounded
    /// `take(..).read_to_end(..)` path must instead reach a clean `Err` with no giant allocation.
    #[test]
    fn read_bitmap_words_rejects_huge_num_bits_without_oom() {
        let huge_num_bits: usize = 1usize << 60; // ⇒ full_byte_amount ≈ 2^57 bytes
        let mut header: Vec<u8> = Vec::new();
        header.push(1u8); // bitmap type (must be 1)
        header.extend_from_slice(&encode_vbyte_off_by_one(huge_num_bits));
        // CRC8 over (type byte ++ num_bits vbyte), exactly as the reader computes it.
        let crc8 = crc::Crc::<u8>::new(&crc::CRC_8_SMBUS);
        let mut digest = crc8.digest();
        digest.update(&header);
        header.push(digest.finalize());
        header.extend_from_slice(&[0u8; 8]); // body: a handful of bytes, nowhere near declared

        let mut r = std::io::Cursor::new(header);
        assert!(
            read_bitmap_words(&mut r).is_err(),
            "a bitmap header declaring 2^60 bits with a truncated body must be a clean Err, \
             never an OOM-abort or a giant pre-allocation"
        );
    }

    /// Site 1 (`decode_section`): a `DictSectPFC` whose `num_strings` is enormous but whose
    /// `packed_data` is a single byte. The old `Dict::with_capacity(num_strings)` /
    /// `Vec::with_capacity(num_strings)` would attempt a multi-exabyte reservation before the
    /// decode loop ran; the `n.min(packed_data.len())` cap holds the reservation at 1 and the
    /// decode fails closed on the empty (null-only) term — never an OOM-abort.
    #[test]
    fn decode_section_rejects_huge_num_strings_without_oom() {
        // `Sequence` / `DictSectPFC` are imported at module scope (via `super::*`). A single
        // 8-bit entry at offset 0 (`get(0) == 0`); packed_data is one null byte.
        let sequence = Sequence {
            entries: 1,
            bits_per_entry: 8,
            data: vec![0usize],
        };
        let sect = DictSectPFC {
            num_strings: 1usize << 60, // attacker-declared, vastly exceeds packed_data.len()
            block_size: 8,
            sequence,
            packed_data: std::sync::Arc::<[u8]>::from(vec![0u8]),
        };
        assert!(
            decode_section(&sect).is_err(),
            "a dictionary section declaring 2^60 strings with 1 byte of packed data must be a \
             clean Err, never an OOM-abort or a 2^60-capacity reservation"
        );
    }

    /// End-to-end sibling: the public `load_reader` must reach a clean `Err` (not a panic, not
    /// an OOM) on a structurally-valid archive whose dictionary section over-states its string
    /// count — the realistic delivery vector for the Site 1 bug. Built by surgically inflating
    /// the shared section's `num_strings` VByte in a real fixture and repairing the section CRC8
    /// so the corrupted count actually reaches our decoder.
    #[test]
    fn load_reader_rejects_inflated_dict_count_without_oom() {
        let Some(bytes) = fixture_bytes() else { return };
        let Some(corrupt) = inflate_shared_num_strings(&bytes) else {
            eprintln!("skipping: could not locate the shared section num_strings vbyte");
            return;
        };
        let owned = corrupt.clone();
        let res = std::panic::catch_unwind(move || crate::load_reader(std::io::Cursor::new(owned)));
        match res {
            Ok(Ok(_)) => panic!("an inflated dictionary string count must not load successfully"),
            Ok(Err(_)) => {} // clean Err — the guarded, correct outcome
            Err(_) => {
                panic!("an inflated dictionary string count must be a clean Err, not a panic/abort")
            }
        }
    }

    /// Locates the SHARED `DictSectPFC`'s `num_strings` VByte in a valid archive, rewrites it to
    /// a huge (but still in-`usize`) value, and repairs the section's CRC8 so the inflated count
    /// survives the upstream meta-CRC and reaches our `decode_section`. `None` if the layout
    /// can't be located (the test then skips). The dictionary section is found by its `0x02` PFC
    /// preamble followed by a `(num_strings, packed_length, block_size, crc8)` meta whose CRC8
    /// checks out (the shared section is the first such).
    fn inflate_shared_num_strings(bytes: &[u8]) -> Option<Vec<u8>> {
        let crc8 = crc::Crc::<u8>::new(&crc::CRC_8_SMBUS);
        for start in 0..bytes.len() {
            if bytes[start] != 0x02 {
                continue;
            }
            let mut p = start + 1;
            let (_, ns_bytes) = try_read_vbyte(bytes, &mut p)?;
            try_read_vbyte(bytes, &mut p)?; // packed_length
            try_read_vbyte(bytes, &mut p)?; // block_size
            if p >= bytes.len() {
                continue;
            }
            let meta_end = p; // crc byte position
            let mut digest = crc8.digest();
            digest.update(&[0x02]);
            digest.update(&bytes[start + 1..meta_end]);
            if digest.finalize() != bytes[meta_end] {
                continue; // not a real section meta (or not the layout we expect)
            }
            // Found it. Rebuild with an inflated num_strings + repaired CRC8.
            let ns_start = start + 1;
            let ns_end = ns_start + ns_bytes.len();
            let inflated = encode_vbyte_off_by_one(1usize << 60);
            let mut out = Vec::with_capacity(bytes.len() + inflated.len());
            out.extend_from_slice(&bytes[..ns_start]); // up to (not incl) the num_strings vbyte
            out.extend_from_slice(&inflated); // inflated num_strings
            out.extend_from_slice(&bytes[ns_end..meta_end]); // packed_length + block_size vbytes
            let mut digest = crc8.digest();
            digest.update(&[0x02]);
            digest.update(&out[ns_start..out.len()]);
            out.push(digest.finalize()); // repaired section CRC8
            out.extend_from_slice(&bytes[meta_end + 1..]); // the rest of the archive verbatim
            return Some(out);
        }
        None
    }

    /// Minimal off-by-one VByte reader over a byte slice (the inverse of
    /// [`encode_vbyte_off_by_one`]); advances `*p` past the consumed bytes. Returns the value
    /// and the raw bytes it spanned, or `None` on a truncated/oversized vbyte.
    fn try_read_vbyte(b: &[u8], p: &mut usize) -> Option<(usize, Vec<u8>)> {
        let start = *p;
        let mut n: u128 = 0;
        let mut shift = 0u32;
        loop {
            let byte = *b.get(*p)?;
            *p += 1;
            n |= ((byte & 0x7f) as u128) << shift;
            if (byte & 0x80) != 0 {
                break;
            }
            shift += 7;
            if shift > 128 {
                return None;
            }
        }
        Some((usize::try_from(n).ok()?, b[start..*p].to_vec()))
    }
}
