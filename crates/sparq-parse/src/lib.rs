//! sparq-parse: compressed serialization of query results (D4).
//!
//! The crate's first real module: chunk-at-a-time compression of serialized
//! query results, designed around one format property — **multi-member gzip and
//! multi-frame zstd are valid single streams**. Each serialized chunk (e.g. one
//! element of `sparq_engine::query_json_chunks_with_budget`'s output) is
//! compressed as an *independent* gzip member / zstd frame, and the members are
//! concatenated in order. The concatenation decodes as ONE stream with stock
//! decoders (`Content-Encoding: gzip` in browsers, `MultiGzDecoder`, `gzip -d`,
//! Python `gzip`, the `zstd` CLI), so chunks can be compressed in parallel as
//! they are produced and emitted in order: compression overlaps serialization
//! instead of following it, and the first bytes hit the wire sooner.
//!
//! Measured results + design notes: `research/custom-parsers-D4-compressed-serialization.md`
//! (benchmarks under `bench/parse/`, `compress-bench` binary). Numbers gate on
//! the house baseline in `research/custom-parsers-baseline.md`.
//!
//! Hard constraint (unchanged from the scaffold): this crate must NOT enter the
//! wasm dependency graph (`sparq-wasm` must never depend on it, directly or
//! transitively) — flate2/zstd/rayon stay out of the bundle.
//!
//! # API sketch
//!
//! ```
//! use sparq_parse::{Codec, CompressedSink, Mode, decode_gzip_concat};
//!
//! let chunks: Vec<String> = vec!["{\"head\":...".into(), "...rest".into()];
//! let mut sink = CompressedSink::new(Codec::Gzip { level: 6 }, Mode::Parallel);
//! let mut wire = Vec::new();
//! for c in &chunks {
//!     sink.push(c.as_bytes());
//!     for member in sink.try_drain().unwrap() {
//!         wire.extend_from_slice(&member); // stream to the client immediately
//!     }
//! }
//! for member in sink.finish().unwrap() {
//!     wire.extend_from_slice(&member);
//! }
//! assert_eq!(decode_gzip_concat(&wire).unwrap(), chunks.concat().into_bytes());
//! ```

use std::collections::BTreeMap;
use std::io::{self, Read, Write};
use std::sync::{Arc, Condvar, Mutex};

use zstd::dict::{DecoderDictionary, EncoderDictionary};

/// A zstd dictionary prepared (digested) once for a fixed compression level, so
/// per-response compression does not re-digest the dictionary bytes. Cheap to
/// share across threads behind the `Arc` in [`Codec::ZstdDict`].
pub struct PreparedDict {
    encoder: EncoderDictionary<'static>,
    decoder: DecoderDictionary<'static>,
    raw: Vec<u8>,
    level: i32,
}

impl PreparedDict {
    /// Digests `dict_bytes` (e.g. the output of [`train_dictionary`]) for
    /// compression at `level` and for decompression.
    pub fn new(dict_bytes: Vec<u8>, level: i32) -> Self {
        PreparedDict {
            encoder: EncoderDictionary::copy(&dict_bytes, level),
            decoder: DecoderDictionary::copy(&dict_bytes),
            raw: dict_bytes,
            level,
        }
    }

    /// The raw dictionary bytes (what a `/dictionary` endpoint would serve).
    pub fn raw(&self) -> &[u8] {
        &self.raw
    }

    /// The compression level the encoder side was prepared for.
    pub fn level(&self) -> i32 {
        self.level
    }

    /// The prepared decoder side (for `zstd::bulk::Decompressor::with_prepared_dictionary`).
    pub fn decoder(&self) -> &DecoderDictionary<'static> {
        &self.decoder
    }
}

/// Trains a zstd dictionary from sample messages (wraps `zstd::dict::from_samples`).
/// `max_size` is the dictionary size cap in bytes (the addendum's vocabulary
/// dictionaries are small: 16–112 KiB).
pub fn train_dictionary<S: AsRef<[u8]>>(samples: &[S], max_size: usize) -> io::Result<Vec<u8>> {
    zstd::dict::from_samples(samples, max_size)
}

/// Which compressed format each pushed chunk becomes one member/frame of.
#[derive(Clone)]
pub enum Codec {
    /// One RFC 1952 gzip member per chunk (flate2). Level 0–9; 6 is the gzip CLI
    /// default, 1 the speed end. Concatenated members are a valid gzip stream.
    Gzip { level: u32 },
    /// One zstd frame per chunk. Concatenated frames are a valid zstd stream.
    Zstd { level: i32 },
    /// One zstd frame per chunk, compressed with a prepared dictionary (the
    /// small-response path: the dictionary supplies the JSON/vocabulary context
    /// a tiny message cannot carry itself). Decoding requires the same dictionary.
    ZstdDict(Arc<PreparedDict>),
}

impl Codec {
    /// Human-readable tag used by benches/reports.
    pub fn name(&self) -> String {
        match self {
            Codec::Gzip { level } => format!("gzip -{level}"),
            Codec::Zstd { level } => format!("zstd -{level}"),
            Codec::ZstdDict(d) => format!("zstd -{} +dict({}B)", d.level, d.raw.len()),
        }
    }
}

/// Compresses one chunk into one self-contained gzip member / zstd frame.
pub fn compress_member(codec: &Codec, chunk: &[u8]) -> io::Result<Vec<u8>> {
    match codec {
        Codec::Gzip { level } => {
            let mut enc = flate2::write::GzEncoder::new(
                Vec::with_capacity(chunk.len() / 4 + 64),
                flate2::Compression::new(*level),
            );
            enc.write_all(chunk)?;
            enc.finish()
        }
        Codec::Zstd { level } => zstd::bulk::compress(chunk, *level),
        Codec::ZstdDict(d) => {
            let mut c = zstd::bulk::Compressor::with_prepared_dictionary(&d.encoder)?;
            c.compress(chunk)
        }
    }
}

/// Serial (compress on `push`, caller's thread) or parallel (compress on the
/// rayon pool, members still emitted in push order).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Serial,
    Parallel,
}

struct Shared {
    /// member index → compression result, for members finished but not yet drained.
    ready: Mutex<BTreeMap<usize, io::Result<Vec<u8>>>>,
    cv: Condvar,
}

/// Feed serialized chunks in, drain compressed members/frames out **in order**.
///
/// Each `push` becomes exactly one gzip member / zstd frame; the concatenation
/// of the drained members (in emission order) is a valid gzip/zstd stream whose
/// decompression is byte-identical to the concatenation of the pushed chunks.
///
/// In [`Mode::Parallel`] the chunks compress concurrently on the rayon pool;
/// [`try_drain`](CompressedSink::try_drain) hands back the contiguous prefix
/// that is ready (so the caller can put early members on the wire while later
/// ones are still compressing) and [`finish`](CompressedSink::finish) waits for
/// the rest. Compression errors surface on drain, at the failed member's
/// position.
pub struct CompressedSink {
    codec: Codec,
    mode: Mode,
    shared: Arc<Shared>,
    next_in: usize,
    next_out: usize,
}

impl CompressedSink {
    pub fn new(codec: Codec, mode: Mode) -> Self {
        CompressedSink {
            codec,
            mode,
            shared: Arc::new(Shared { ready: Mutex::new(BTreeMap::new()), cv: Condvar::new() }),
            next_in: 0,
            next_out: 0,
        }
    }

    /// Number of chunks pushed so far.
    pub fn pushed(&self) -> usize {
        self.next_in
    }

    /// Submits one serialized chunk; it becomes member/frame number
    /// [`pushed`](CompressedSink::pushed)` - 1` of the output stream.
    pub fn push(&mut self, chunk: &[u8]) {
        let idx = self.next_in;
        self.next_in += 1;
        match self.mode {
            Mode::Serial => {
                let out = compress_member(&self.codec, chunk);
                self.shared.ready.lock().unwrap().insert(idx, out);
            }
            Mode::Parallel => {
                // The chunk is copied so the compression task is 'static; a 64 KiB
                // memcpy is noise next to compressing those bytes.
                let owned = chunk.to_vec();
                let codec = self.codec.clone();
                let shared = Arc::clone(&self.shared);
                rayon::spawn(move || {
                    let out = compress_member(&codec, &owned);
                    shared.ready.lock().unwrap().insert(idx, out);
                    shared.cv.notify_all();
                });
            }
        }
    }

    /// Non-blocking: removes and returns the contiguous run of finished members
    /// starting at the emission cursor. An `Err` is a compression failure of the
    /// member at the cursor (the stream is unusable past that point).
    pub fn try_drain(&mut self) -> io::Result<Vec<Vec<u8>>> {
        let mut ready = self.shared.ready.lock().unwrap();
        let mut out = Vec::new();
        while let Some(r) = ready.remove(&self.next_out) {
            self.next_out += 1;
            out.push(r?);
        }
        Ok(out)
    }

    /// Waits for every outstanding member and returns the remainder, in order.
    pub fn finish(mut self) -> io::Result<Vec<Vec<u8>>> {
        let total = self.next_in;
        let mut out = Vec::with_capacity(total - self.next_out);
        let mut ready = self.shared.ready.lock().unwrap();
        while self.next_out < total {
            match ready.remove(&self.next_out) {
                Some(r) => {
                    self.next_out += 1;
                    out.push(r?);
                }
                None => ready = self.shared.cv.wait(ready).unwrap(),
            }
        }
        Ok(out)
    }
}

/// One-shot convenience: compress a batch of chunks into ordered members/frames.
/// [`Mode::Parallel`] uses the rayon pool; order is preserved by construction.
pub fn compress_chunks<S: AsRef<[u8]> + Sync>(chunks: &[S], codec: &Codec, mode: Mode) -> io::Result<Vec<Vec<u8>>> {
    match mode {
        Mode::Serial => chunks.iter().map(|c| compress_member(codec, c.as_ref())).collect(),
        Mode::Parallel => {
            use rayon::prelude::*;
            chunks.par_iter().map(|c| compress_member(codec, c.as_ref())).collect()
        }
    }
}

/// Decodes a concatenation of gzip members as one stream (what a browser does
/// for `Content-Encoding: gzip`). Note: flate2's plain `GzDecoder` stops after
/// the FIRST member — multi-member streams need `MultiGzDecoder` (this is the
/// classic client caveat; see the research note).
pub fn decode_gzip_concat(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut out = Vec::new();
    flate2::read::MultiGzDecoder::new(data).read_to_end(&mut out)?;
    Ok(out)
}

/// Decodes a concatenation of zstd frames as one stream (the zstd stream decoder
/// continues across frame boundaries by default, like the `zstd` CLI).
pub fn decode_zstd_concat(data: &[u8]) -> io::Result<Vec<u8>> {
    zstd::stream::decode_all(data)
}

/// [`decode_zstd_concat`] for dictionary-compressed frames: the dictionary is
/// loaded once and applies to every frame in the stream.
pub fn decode_zstd_concat_with_dict(data: &[u8], dict: &[u8]) -> io::Result<Vec<u8>> {
    let mut out = Vec::new();
    zstd::stream::read::Decoder::with_dictionary(data, dict)?.read_to_end(&mut out)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Chunks shaped like the engine's SPARQL-JSON output: a head, then row
    /// fragments with repeated structure but distinct values (so members differ
    /// and ordering bugs cannot cancel out).
    fn json_chunks(n: usize) -> Vec<Vec<u8>> {
        let mut chunks = vec![br#"{"head":{"vars":["s","p","o"]},"results":{"bindings":["#.to_vec()];
        for i in 0..n {
            let mut c = Vec::new();
            for j in 0..200 {
                c.extend_from_slice(
                    format!(
                        r#"{{"s":{{"type":"uri","value":"http://www.wikidata.org/entity/Q{i}_{j}"}},"p":{{"type":"uri","value":"http://www.wikidata.org/prop/direct/P{j}"}},"o":{{"type":"literal","value":"row {i} col {j}"}}}},"#
                    )
                    .as_bytes(),
                );
            }
            chunks.push(c);
        }
        chunks.push(b"]}}".to_vec());
        chunks
    }

    fn concat(chunks: &[Vec<u8>]) -> Vec<u8> {
        chunks.iter().flat_map(|c| c.iter().copied()).collect()
    }

    fn run_sink(chunks: &[Vec<u8>], codec: Codec, mode: Mode) -> Vec<Vec<u8>> {
        let mut sink = CompressedSink::new(codec, mode);
        let mut members = Vec::new();
        for c in chunks {
            sink.push(c);
            members.extend(sink.try_drain().unwrap());
        }
        members.extend(sink.finish().unwrap());
        members
    }

    #[test]
    fn multi_member_gzip_roundtrips() {
        let chunks = json_chunks(20);
        for mode in [Mode::Serial, Mode::Parallel] {
            let members = run_sink(&chunks, Codec::Gzip { level: 6 }, mode);
            assert_eq!(members.len(), chunks.len(), "one member per chunk ({mode:?})");
            assert_eq!(decode_gzip_concat(&concat(&members)).unwrap(), concat(&chunks), "{mode:?}");
        }
    }

    #[test]
    fn multi_frame_zstd_roundtrips() {
        let chunks = json_chunks(20);
        for mode in [Mode::Serial, Mode::Parallel] {
            let members = run_sink(&chunks, Codec::Zstd { level: 3 }, mode);
            assert_eq!(members.len(), chunks.len(), "one frame per chunk ({mode:?})");
            assert_eq!(decode_zstd_concat(&concat(&members)).unwrap(), concat(&chunks), "{mode:?}");
        }
    }

    #[test]
    fn parallel_members_equal_serial_members() {
        // Same codec + same input must give byte-identical members regardless of
        // mode — this pins both determinism and the in-order emission contract.
        let chunks = json_chunks(32);
        for codec in [Codec::Gzip { level: 1 }, Codec::Zstd { level: 1 }] {
            let serial = run_sink(&chunks, codec.clone(), Mode::Serial);
            let parallel = run_sink(&chunks, codec.clone(), Mode::Parallel);
            assert_eq!(serial, parallel, "{}", codec.name());
        }
    }

    #[test]
    fn compress_chunks_matches_sink() {
        let chunks = json_chunks(8);
        let codec = Codec::Zstd { level: 3 };
        let batch = compress_chunks(&chunks, &codec, Mode::Parallel).unwrap();
        assert_eq!(batch, run_sink(&chunks, codec, Mode::Serial));
    }

    #[test]
    fn empty_chunks_and_empty_stream() {
        // An empty chunk is a legal (if pointless) member; an empty sink yields
        // an empty stream.
        let chunks = vec![b"".to_vec(), b"x".to_vec(), b"".to_vec()];
        let members = run_sink(&chunks, Codec::Gzip { level: 6 }, Mode::Parallel);
        assert_eq!(decode_gzip_concat(&concat(&members)).unwrap(), b"x");
        let none = CompressedSink::new(Codec::Zstd { level: 3 }, Mode::Parallel).finish().unwrap();
        assert!(none.is_empty());
    }

    fn small_samples(n: usize) -> Vec<Vec<u8>> {
        (0..n)
            .map(|i| {
                format!(
                    r#"{{"head":{{"vars":["p","o"]}},"results":{{"bindings":[{{"p":{{"type":"uri","value":"http://www.wikidata.org/prop/direct/P{}"}},"o":{{"type":"literal","value":"point response {i}"}}}}]}}}}"#,
                    i % 50
                )
                .into_bytes()
            })
            .collect()
    }

    #[test]
    fn dictionary_frames_decode_with_the_dictionary() {
        let samples = small_samples(300);
        let dict = train_dictionary(&samples, 16 * 1024).unwrap();
        let prepared = Arc::new(PreparedDict::new(dict.clone(), 3));
        let codec = Codec::ZstdDict(Arc::clone(&prepared));

        // Held-out small responses (not in the training set).
        let responses = small_samples(320).split_off(300);
        let frames = compress_chunks(&responses, &codec, Mode::Parallel).unwrap();
        let stream = concat(&frames);

        // Decodes with the dictionary — as one multi-frame stream and per frame.
        assert_eq!(decode_zstd_concat_with_dict(&stream, &dict).unwrap(), concat(&responses));
        let mut d = zstd::bulk::Decompressor::with_prepared_dictionary(prepared.decoder()).unwrap();
        for (frame, want) in frames.iter().zip(&responses) {
            assert_eq!(&d.decompress(frame, want.len() + 1).unwrap(), want);
        }

        // And does NOT decode without it (proves the dictionary is load-bearing).
        assert!(decode_zstd_concat(&stream).is_err(), "dict frames must not decode dict-less");

        // Dictionary actually helps on these sizes: strictly smaller than plain zstd.
        let plain = compress_chunks(&responses, &Codec::Zstd { level: 3 }, Mode::Serial).unwrap();
        let (with, without): (usize, usize) =
            (frames.iter().map(Vec::len).sum(), plain.iter().map(Vec::len).sum());
        assert!(with < without, "dict {with} B should beat plain {without} B on small responses");
    }

    #[test]
    fn drain_overlaps_production() {
        // try_drain hands back a contiguous prefix while later chunks are still
        // being pushed — the property the streaming server path relies on.
        let chunks = json_chunks(64);
        let mut sink = CompressedSink::new(Codec::Zstd { level: 1 }, Mode::Parallel);
        let mut members = Vec::new();
        let mut drained_early = false;
        for c in &chunks {
            sink.push(c);
            let got = sink.try_drain().unwrap();
            drained_early |= !got.is_empty() && sink.pushed() < chunks.len();
            members.extend(got);
        }
        members.extend(sink.finish().unwrap());
        assert_eq!(members.len(), chunks.len());
        assert_eq!(decode_zstd_concat(&concat(&members)).unwrap(), concat(&chunks));
        // Not asserted as a hard invariant (scheduling), but record the intent:
        // on any real machine some members are ready before the last push.
        if !drained_early {
            eprintln!("note: no member drained before the final push (scheduling fluke)");
        }
    }
}
