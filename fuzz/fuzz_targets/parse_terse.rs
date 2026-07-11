// Coverage-guided fuzz target for the terse SPARQL parser.
// Bead sq-3dyje.5; threat-model T-PARSE-FUZZ (untrusted terse query strings). [SONNET-4.6]
//
// SURFACE: `sparq_terse::terse_to_sparql(src)` — the crate's single public entry point.
// It takes a terse (keyword-expanded) SPARQL/UPDATE string and:
//   1. Runs the keyword expansion layer (K: substitution + V() vector-search construct).
//   2. Passes the result through the underlying spargebra SPARQL parser.
// Both stages run in this single call; a TerseError from either is a clean outcome.
//
// INVARIANT: hostile text must produce Ok(Expansion) or a clean TerseError — never a
// panic, OOB, stack overflow, integer-overflow abort (overflow-checks on in this profile),
// or UB.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // terse_to_sparql takes &str; convert bytes lossily so the fuzzer can drive the
    // keyword layer and parser with arbitrary byte sequences.
    let text = String::from_utf8_lossy(data);
    // The contract: Ok(expansion) or Err(TerseError). A panic here is the finding.
    let _ = sparq_terse::terse_to_sparql(&text);
});
