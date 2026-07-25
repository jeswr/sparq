// Coverage-guided fuzz target for the RIF-Core XML + N3 rule parsers.
// Bead sq-3dyje.5; threat-model T-PARSE-FUZZ (untrusted rule documents). [SONNET-4.6]
//
// SURFACES:
//   * `sparq_reason::rif_xml::import(bytes)` — RIF-Core XML presentation syntax parser.
//     Parses the W3C RIF-Core XML dialect from raw bytes (quick-xml underneath) into a
//     `rif::Document`, performing Or-split / Exists-flatten desugaring and a fail-closed
//     taxonomy.  An `ImportError` of any variant is a clean outcome.
//   * `sparq_reason::reason_n3_terms(src, base)` — N3 rules parser + forward-chainer.
//     Takes a Notation3 text and returns the full N3Closure.  A `String` error from the
//     parser or chainer is a clean outcome.
//
// The first byte of the fuzz input selects which surface is exercised (so a single corpus
// covers both code paths):
//   data[0] & 1 == 0  → RIF-XML path (bytes fed raw — XML is byte-driven, not UTF-8)
//   data[0] & 1 == 1  → N3 text path  (remainder converted to UTF-8 lossily)
//
// INVARIANT: hostile bytes must produce Ok(...) or a clean Err — never a panic, stack
// overflow from runaway recursion, OOB access, integer-overflow abort (overflow-checks on
// in this profile), or UB.
#![no_main]

use libfuzzer_sys::fuzz_target;
use sparq_reason::rif_xml;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    let ctrl = data[0];
    let body = &data[1..];

    if ctrl & 1 == 0 {
        // RIF-XML path: feed raw bytes (XML is byte-level; import() accepts &[u8]).
        // Any ImportError variant is a clean outcome — not a finding.
        let _ = rif_xml::import(body);
    } else {
        // N3 text path: convert bytes to UTF-8 lossily, as the N3 parser takes &str.
        let src = String::from_utf8_lossy(body);
        // base = None so no document resolver is invoked (no I/O in the fuzzer).
        let _ = sparq_reason::reason_n3_terms(&src, None);
    }
});
