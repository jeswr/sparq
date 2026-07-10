// Coverage-guided fuzz target for the RDFC-1.0 dataset canonicalization input.
// Bead sq-3dyje.5; threat-model T-PARSE-FUZZ (untrusted N-Quads from the network or
// files submitted to the canonicalization service). [SONNET-4.6]
//
// SURFACES:
//   * `sparq_canon::parse_nquads(input)` — N-Quads parse (the input side).
//   * `sparq_canon::canonicalize_nquads(input)` — parse + full RDFC-1.0 canonicalize.
//
// The upstream `rdf_canon` crate enforces an HNDQ poison-graph call limit (default 4000
// iterations).  A poison graph — a blank-node pattern whose isomorphism partition blows
// up the HNDQ recursion — will exhaust the call limit and return
// `Err(CanonError::Canonicalization(...))` rather than hanging.  The fuzzer therefore
// exercises all three exit paths:
//   1. Parse error  → `Err(CanonError::Bridge(...))`
//   2. Poison graph → `Err(CanonError::Canonicalization(...))`
//   3. Valid input  → `Ok(canonical_nquads_string)`
//
// For path 3 we additionally run `parse_nquads` on the output and assert the two
// parses agree on the number of quads (round-trip stability for valid inputs).
//
// INVARIANT: any input must produce `Ok(...)` or a clean `Err(...)` — never a panic,
// unbounded work, OOB, integer-overflow abort (overflow-checks on in this profile), or UB.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // N-Quads is a text format; convert bytes lossily so the fuzzer can drive the
    // parser with arbitrary byte sequences including non-UTF-8 prefixes.
    let text = String::from_utf8_lossy(data);

    // Exercise the combined parse + canonicalize path.
    if let Ok(canonical) = sparq_canon::canonicalize_nquads(&text) {
        // Round-trip stability: the canonical output must itself parse cleanly
        // and yield the same number of quads as the input.
        let input_quads = sparq_canon::parse_nquads(&text)
            .map(|v| v.len())
            .unwrap_or(0);
        let canon_quads = sparq_canon::parse_nquads(&canonical)
            .map(|v| v.len())
            .unwrap_or(0);
        // The canonical form must be idempotent: re-canonicalizing it must
        // produce the same string (RDFC-1.0 is deterministic + stable).
        let recanon = sparq_canon::canonicalize_nquads(&canonical);
        debug_assert_eq!(
            recanon.as_deref().ok(),
            Some(canonical.as_str()),
            "RDFC-1.0 canonicalization is not idempotent"
        );
        // Quad count must be preserved across canonicalization.
        debug_assert_eq!(
            input_quads, canon_quads,
            "quad count changed after canonicalization: {} -> {}",
            input_quads, canon_quads
        );
    }
    // Err (parse error or poison-graph limit) is a clean outcome, not a finding.

    // Also exercise the parse-only path independently.
    let _ = sparq_canon::parse_nquads(&text);
});
