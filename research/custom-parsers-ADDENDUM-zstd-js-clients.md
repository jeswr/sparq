# ADDENDUM from the orchestrator (user directive, 2026-06-12) — zstd scope expansion

> For the custom-parsers/compression agent: read this BEFORE writing your zstd verdicts.

The user has clarified that zstd's value does NOT hinge on browser-native Content-Encoding
support: **Solid applications can decompress zstd in their JavaScript code** (e.g. fzstd,
zstd-wasm, or a decoder shipped inside a sparq client library). Implications for your
deliverables:

1. Do NOT down-weight the zstd paths (multi-frame parallel compression, custom vocabulary
   dictionaries) for lack of browser-native decoding. The realistic consumer matrix is:
   - Browser native Content-Encoding: gzip (universal), br, zstd where shipped (Chrome 123+;
     verify Firefox/Safari current status and cite).
   - **Browser JS-level decoding (Solid apps)**: zstd via a JS/WASM decoder — full zstd
     feature set INCLUDING custom dictionaries becomes usable. Measure decode-side cost
     honestly (JS/WASM zstd decode throughput vs native gzip DecompressionStream) so the
     client-side trade-off is quantified, or cite credible published numbers if measuring
     a JS decoder is out of scope for this worktree.
   - Server-to-server (prod-solid-server → sparq sidecar): native zstd both sides, dictionaries
     trivially shareable.
2. The custom-dictionary protocol sketch should therefore include how a client OBTAINS the
   dictionary (e.g. a `/dictionary` endpoint keyed by dataset generation; dictionary id echoed
   in a response header so clients know which dict decodes the body) — design-level only,
   implementation belongs to the serving wave.
3. Benchmark additions (cheap): zstd-with-vocab-dictionary ratio/speed on SMALL responses
   (the case dictionaries exist for) at levels 1–3, vs gzip -1/-6 and plain zstd — this is
   the number that justifies (or kills) the whole dictionary path.
4. Record in your final report under a "client decode matrix" so the orchestrator can hand
   the JS-decoder integration to the bindings-parity wave (sparq js client could bundle fzstd
   or similar).
