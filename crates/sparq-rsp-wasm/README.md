# sparq-rsp-wasm

The sparq windowed **RSP-QL stream processor** ([`sparq-rsp`](../sparq-rsp/README.md))
compiled to WebAssembly — the tier-b **"W-rsp"** bundle ([OPUS-4.8] sq-nzcb).

A SEPARATE, lazy-loaded bundle from the lean [`sparq-wasm`](../sparq-wasm/README.md)
triplestore bundle (the [`sparq-reason-wasm`](../sparq-reason-wasm/README.md) pattern): it
carries windowed continuous SPARQL — sliding/tumbling time windows with
RSTREAM / ISTREAM / DSTREAM output — which the lean default browser bundle deliberately does
not. The showcase site loads it ONLY on `/surface/streaming-rsp` (`next/dynamic`,
client-only) so the landing page stays light.

```js
import init, { Rsp } from "./sparq_rsp_wasm.js";
await init();

// AVG(?v) over tumbling 60-tick windows (RANGE 60 STEP 60), full result per window.
const q = Rsp.select(
  "SELECT (AVG(?v) AS ?avg) WHERE { ?s <http://ex/reading> ?v }",
  60,        // range  (logical time unit, same scale as the pushed ts)
  60,        // step   (== range => tumbling; < range => sliding)
  0,         // maxDelay (out-of-order tolerance)
  "rstream", // "rstream" | "istream" | "dstream"
);

// Push timestamped readings; each push returns the windows that just CLOSED, as JSON.
q.push("<http://ex/s1>", "<http://ex/reading>", "10", 0);   // -> "[]" (nothing closed)
q.push("<http://ex/s1>", "<http://ex/reading>", "20", 30);  // -> "[]"
const closed = JSON.parse(
  q.push("<http://ex/s1>", "<http://ex/reading>", "5", 65),  // ts 65 closes [0,60)
);
// [ { "start": 0, "end": 60, "results": { "head": {...}, "results": { "bindings": [...] } } } ]
// closed[0].results is a standard, self-contained SPARQL 1.1 JSON document (AVG = 15.0).

const tail = JSON.parse(q.flush()); // end-of-stream: close the remaining open window(s)
```

## The UI drives the logical clock

`sparq-rsp` is "wasm-safe" by construction — **it reads no wall clock and runs no async
runtime**. Time advances *only* through application-supplied timestamps: each pushed reading
carries a `ts`, a window closes when the watermark (`max_ts − max_delay`) reaches its end,
and the whole pipeline is a pure, replayable function of the pushed `(triple, ts)` sequence.
So the browser tab IS the clock: a slider or a `setInterval` in the page advances `ts`,
nothing in the bundle does. This bundle confirms that wasm-safety claim — it builds for
`wasm32-unknown-unknown` in CI and the headless suite drives the real exports in a genuine
wasm runtime.

## API

`Rsp.select(sparql, range, step, maxDelay, r2s)` registers a continuous SELECT (validated
*now* — a malformed or non-SELECT query throws here, not at the first push) over a time
window. Then:

| method | returns | role |
|---|---|---|
| `push(s, p, o, ts)` | JSON array of closed-window `{start,end,results}` objects | feeds one timestamped triple; the terms are Turtle syntax (`<iri>`, the numeric shorthand `10` / `10.5`, `"lit"`, `"3.0"^^<…#decimal>`, `"hi"@en`, `_:b`) |
| `flush()` | same JSON array | end-of-stream: close every window up to the last `ts` seen (ignoring `maxDelay`) |
| `lateDropped()` | `number` | arrivals dropped as too late (every covering window already closed) |
| `sparql()` | `string` | the registered query text (echo, for the UI) |

Each closed window is `{"start":S,"end":E,"results":<SPARQL-1.1-JSON>}`: half-open `[S,E)`
time bounds plus the R2S-filtered SELECT table in the standard, self-contained SPARQL 1.1
JSON form (so the page hands `.results` to any SPARQL-JSON renderer). The window semantics —
boundary inclusivity, sliding overlap, lateness, empty-window reporting, the R2S multiset
diffs — are exactly [`sparq-rsp`'s](../sparq-rsp/README.md), pinned by that crate's tests;
this bundle is a thin, zero-`unsafe` JS wrapper that owns no parsing or semantics of its own
(triples are parsed via the engine's Turtle path — chosen over N-Triples so the streaming demo
can push the bare numeric shorthand `10` / `10.5` rather than the verbose `"10"^^<…#integer>`;
results via the engine's SPARQL-JSON serialiser — the bundle carries no serde).

This bundle exposes the single-window `ContinuousQuery` SELECT form (the streaming-rsp
showcase). CONSTRUCT/ASK forms and multi-window joins (`ContinuousMultiQuery`) stay in the
native crate for now.

## Build & test

```sh
# Build the wasm artifact (the site's pages.yml does this for /surface/streaming-rsp):
wasm-pack build crates/sparq-rsp-wasm --target web

# Native unit tests (the pure helpers) + headless wasm tests (the real #[wasm_bindgen] API):
cargo test -p sparq-rsp-wasm
wasm-pack test --node            # from crates/sparq-rsp-wasm
```

CI builds + clippies the crate for `wasm32-unknown-unknown` and runs the headless
`wasm-pack test --node` suite, exactly like the other wasm bundles; the wasm-deps guard keeps
the native-only heavy deps (rayon / compression codecs / `sparq-parse`) out of its graph.
