// Coverage-guided fuzz target for the HDT binary container decoder.
// Bead sq-3dyje.5; threat-model T-PARSE-FUZZ (downloaded .hdt files). [SONNET-4.6]
//
// SURFACE: `sparq_hdt::graph_from_reader(reader)` — the public one-shot decode entry
// point that parses an HDT container from any `BufRead` (typically a file or network
// stream) and builds an in-memory `sparq_core::Graph`.  This is the path that would be
// reachable from a malicious or truncated .hdt download.
//
// INVARIANT: hostile bytes must produce a clean `Err(sparq_hdt::Error::*)` — NEVER a
// panic, OOB access, integer-overflow abort (overflow-checks on in this profile), or UB.
// `graph_from_reader` is the primary attack surface: it reads raw binary (not UTF-8)
// directly from its `BufRead`, so we feed raw bytes without any UTF-8 conversion.
#![no_main]

use libfuzzer_sys::fuzz_target;
use std::io::BufReader;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    // Raw bytes — HDT is a binary format, no UTF-8 wrapping needed.
    let reader = BufReader::new(Cursor::new(data));
    // The contract: Ok(graph) or Err(_). A panic/abort here is the finding.
    let _ = sparq_hdt::graph_from_reader(reader);
});
