// Coverage-guided fuzz target for the sparq JSON-LD 1.1 document pipeline.
// Bead sq-oy1f.46 (security: attacker-reachable ingest via application/ld+json GSP
// PUT/POST + CLI). Authored by Claude Fable 5 [SONNET-4.6]. [SONNET-4.6]
//
// SURFACE: the full sparq-jsonld document pipeline —
//   bytes → Json::parse → expand() → flatten_expanded()
// PLUS the oxjsonld toRdf leg via sparq-core::Graph::load_str("jsonld"):
//   bytes → Graph::load_str(text, "jsonld") → quads drained into Graph
// (sparq-core is already a dependency here; enabling the `jsonld` feature adds
// the oxjsonld parser on that code path without a new top-level dep.)
//
// The two legs exercise distinct code paths:
//   * sparq-jsonld leg: the in-house AST parser + Expansion (§5) + Flattening (§7)
//     + Node Map Generation (§7.2). Deeply nested @context chains, @list nesting,
//     scoped contexts, @reverse, @included, and @json literals must NOT blow the
//     stack; every invalid input must produce a JsonLdError, never a panic.
//   * oxjsonld leg (via sparq-core `jsonld` feature): the Oxigraph JSON-LD-to-RDF
//     parser that sparq-core dispatches attacker-controlled GSP PUT/POST bodies to
//     when the server is built with JSON-LD support. Also default-on in sparq-py
//     and sparq-cli. A panic here is a real fail-open in a default-on ingest path.
//
// INPUT LAYOUT (one byte corpus → two legs; the selector byte keeps libFuzzer
// exploring both branches):
//
//   data[0]: selector byte
//     bit 0 = 0  → sparq-jsonld leg only
//     bit 0 = 1  → BOTH legs (sparq-jsonld first, then oxjsonld via sparq-core)
//   data[1..]: the JSON-LD document payload (UTF-8 best-effort via lossy conv)
//
// INVARIANT (fail-closed): on ANY byte sequence the pipeline MUST return a
// structured error — it MUST NOT panic, overflow the stack, abort on overflow
// checks, or trigger UB. libFuzzer (with the address sanitizer, overflow checks,
// and debug assertions all enabled in the release profile) reports any crash;
// a crash here is a real P1 bug in an attacker-reachable ingest path.
//
// RECURSION + SIZE GUARD: the sparq-jsonld expansion algorithm carries an
// internal recursion depth counter; libFuzzer's -rss_limit_mb (2048 in CI) +
// the address sanitizer catch any remaining stack overflow as a crash. The
// -max_len default (4096 unless overridden) bounds individual seed/corpus inputs.
#![no_main]

use libfuzzer_sys::fuzz_target;
use sparq_jsonld::{expand, flatten_expanded, Json, JsonLdOptions, NoopLoader};

fuzz_target!(|data: &[u8]| {
    if data.len() < 2 {
        // Need at least the selector byte and one payload byte.
        return;
    }
    let selector = data[0];
    let payload = &data[1..];

    // Best-effort UTF-8 conversion: invalid bytes become the replacement char
    // (U+FFFD) so the parser still sees a &str and explores its deep paths.
    // The real GSP ingest reads bytes directly (oxjsonld) or via a &str decoded
    // at the HTTP-body boundary (sparq-jsonld) — both attacker-controlled.
    let text = String::from_utf8_lossy(payload);

    // --- sparq-jsonld leg: Json::parse → expand → flatten_expanded ---
    //
    // Json::parse is the document-level entry every sparq-jsonld caller uses.
    // expand() carries the full Expansion Algorithm (§5.1) over the AST;
    // NoopLoader refuses every remote @context fetch with a structured error
    // (no panics on remote-context references). flatten_expanded() (§7.1)
    // exercises the Node Map Generation + sort + de-dup path.
    if let Ok(doc) = Json::parse(&text) {
        let opts = JsonLdOptions::default();
        let loader = NoopLoader;

        // Expansion: scoped contexts, @nest, @reverse, @included, @json literals,
        // container maps — deep recursion on malicious @context chains must stay
        // bounded.
        if let Ok(expanded) = expand(&doc, &opts, &loader) {
            // Flattening: node-map build + named-graph fold + sort by @id.
            let _ = flatten_expanded(&expanded);
        }
    }

    // --- oxjsonld toRdf leg via sparq-core Graph::load_str ---
    //
    // Only runs when the selector bit is 1, so libFuzzer explores sparq-jsonld-
    // only inputs on half the corpus and both legs on the other half. The oxjsonld
    // parser is the SAME parser the server GSP PUT handler calls on production
    // traffic when the `jsonld` feature is active; a panic here is a real crash.
    //
    // sparq-core is always a dependency here (parse_rdf_str / graph_open already
    // use it). The `jsonld` feature on sparq-core wires in oxjsonld; without it
    // Graph::load_str("jsonld") returns Err("unrecognised format") — still a
    // structured error, not a panic, so the invariant holds in both builds.
    if selector & 1 == 1 {
        // Graph::load_str delegates to oxjsonld::JsonLdParser::new().for_slice()
        // and folds quads into the default graph. Any parse error → Err(String).
        // A panic inside oxjsonld is caught by libFuzzer as a crash.
        let _ = sparq_core::Graph::load_str(&text, "jsonld");
    }
});
