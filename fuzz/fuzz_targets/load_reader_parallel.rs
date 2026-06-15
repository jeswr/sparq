// Coverage-guided fuzz target for the streaming/parallel RDF ingest entry point.
// Bead sq-ovnf; threat-model T-PARSE-FUZZ. [OPUS-4.8]
//
// SURFACE: `sparq_core::Graph::load_reader_parallel(reader, format)` — the streaming
// loader. For N-Triples it drives `load_ntriples_pipelined` (the read-thread ->
// chunk-parallel-parse -> sharded-dict-merge pipeline, incl. the EOF `carry` path for
// a final line with no trailing newline); for the other formats it delegates to
// `load_reader`. The public entry hardcodes a 32 MiB block, so typical fuzz inputs
// stay single-block; this target still exercises the threaded read->parse->merge
// pipeline + the multi-`read()` short-read assembly + the partial-final-line carry
// that the synchronous `load_str` target does not. (The cross-block straddle inside
// `load_ntriples_pipelined` is unit-tested directly with a tiny block in sparq-core.)
//
// The reader is a plain `&[u8]` cursor over the fuzz bytes (the same `io::Read` a
// decompressor or socket would present). First byte selects the format.
//
// INVARIANT: hostile bytes must produce a clean `Err` OR a valid `Graph` — NEVER a
// panic, an overflow/OOM abort, or UB. libFuzzer flags any crash and saves the input.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    let format = match data[0] % 4 {
        0 => "ntriples",
        1 => "nquads",
        2 => "turtle",
        _ => "trig",
    };
    let body = &data[1..];
    // `load_reader_parallel` takes any `Read + Send`; a byte-slice cursor models a
    // decompressed stream / socket exactly and is `Send`.
    let reader = std::io::Cursor::new(body.to_vec());
    let _ = sparq_core::Graph::load_reader_parallel(reader, format);
});
