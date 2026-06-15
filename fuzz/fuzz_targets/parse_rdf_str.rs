// Coverage-guided fuzz target for the RDF string-ingest entry point.
// Bead sq-ovnf; threat-model T-PARSE-FUZZ. [OPUS-4.8]
//
// SURFACE: `sparq_core::Graph::load_str(text, format)` — the synchronous RDF parser
// that drives all four text formats (N-Triples / N-Quads / Turtle / TriG, incl. the
// parallel chunked paths under the `parallel` feature). libFuzzer mutates raw bytes;
// the first byte selects the format so a single corpus exercises every parser, and
// the remainder is the document. Non-UTF-8 input is fed via `from_utf8_lossy` so the
// fuzzer can still drive the parser with arbitrary bytes (the public API takes `&str`,
// so this is the closest hostile-bytes surface to a real caller).
//
// INVARIANT: hostile bytes must produce a clean `Err` OR a valid `Graph` — NEVER a
// panic, an integer-overflow abort (overflow-checks are on in this profile), an
// allocator OOM-abort, or UB. libFuzzer treats any of those as a crash and saves the
// reproducing input.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    // First byte picks the format so the corpus reaches every parser branch.
    let format = match data[0] % 4 {
        0 => "ntriples",
        1 => "nquads",
        2 => "turtle",
        _ => "trig",
    };
    let text = String::from_utf8_lossy(&data[1..]);
    // The contract: Ok(graph) or Err(msg). Either is fine; a panic/abort is the bug.
    let _ = sparq_core::Graph::load_str(&text, format);
});
