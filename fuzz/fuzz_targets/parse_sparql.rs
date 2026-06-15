// Coverage-guided fuzz target for the SPARQL parser entry point.
// Bead sq-ovnf; threat-model T-PARSE-FUZZ. [OPUS-4.8]
//
// SURFACE: `sparq_engine::PreparedQuery::parse(sparql)` — the parse-once seam that
// wraps the (vendored) `spargebra` SPARQL-syntax-to-algebra parser. Every string
// query entry point (`query` / `ask` / `count` / `construct` / ...) funnels through
// this, so it is THE parser surface. libFuzzer mutates raw bytes; we feed them as a
// `&str` via `from_utf8_lossy` (the public API takes `&str`).
//
// INVARIANT: hostile query text must produce a clean `Err(String)` OR a valid
// `PreparedQuery` — NEVER a panic, a recursion/stack-overflow abort, an
// overflow/OOM abort, or UB. libFuzzer flags any crash and saves the input.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let sparql = String::from_utf8_lossy(data);
    let _ = sparq_engine::PreparedQuery::parse(&sparql);
});
