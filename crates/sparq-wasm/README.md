# sparq-wasm

The sparq parser + triplestore + SPARQL engine compiled to WebAssembly for the
browser (and Node). Same store and engine as the native build — including the
worst-case-optimal Leapfrog Triejoin for cyclic queries — single-threaded and
with a minimal bundle (no rayon, no serde; results are serialised by hand to
SPARQL 1.1 JSON).

## Build

```sh
# Browser (ES modules)
wasm-pack build --target web --release
# Node (CommonJS) — used by test/smoke.cjs
wasm-pack build --target nodejs --release --out-dir pkg-node
```

`getrandom`'s browser backend is selected via `.cargo/config.toml`
(`--cfg getrandom_backend="wasm_js"`) plus the wasm-only `getrandom` dependency
in `Cargo.toml` (oxrdf pulls `rand` for blank-node ids).

## Use

```js
import init, { Store } from "./pkg/sparq_wasm.js";
await init();

const store = Store.load(turtleText, "turtle"); // or "ntriples" | "nquads" | "trig"
store.size;             // number of triples
store.heapBytes();      // rough in-memory footprint

const json = store.query("SELECT * WHERE { ?s ?p ?o } LIMIT 10");
const { head, results } = JSON.parse(json); // SPARQL 1.1 JSON results

const n = store.count("SELECT ?s WHERE { ?s a <http://ex/Person> }"); // lazy, no materialise
```

`query` supports the M2 surface: BGPs (binary + WCOJ), FILTER, OPTIONAL, UNION,
MINUS, BIND, VALUES, aggregation (GROUP BY / HAVING), ORDER BY, DISTINCT/LIMIT/
OFFSET, sub-SELECT.

### Streaming wins carry over to the browser

The same engine optimisations apply in WASM, where memory and main-thread time are
scarcest. Measured in Node over a 400k-triple graph (`test/perf.cjs`):

| query | time |
|---|--:|
| `SELECT * … LIMIT 10` (early-termination, no full scan) | **0.026 ms** |
| `count(?s a Person)` (lazy, index range size) | **0.010 ms** |
| `count(follows · name)` (count-only join, no row materialise) | 1.5 ms |
| `FILTER(?a > 90)` (pushed into the sorted scan) | 2.3 ms |
| `OPTIONAL` (sort-merge left join) | 64 ms |

`store.count(sparql)` exposes the lazy count directly (a single-pattern scan or a
two-pattern join is counted from the index with no result rows built) — ideal for
"how many?" UI queries on a memory-constrained device.

## Bundle size (release, wasm-opt -Oz)

| artifact | raw | gzip | brotli |
|---|--:|--:|--:|
| `sparq_wasm_bg.wasm` | 787 KB | 276 KB | **210 KB** |
| `sparq_wasm.js` (glue) | 10 KB | 2.9 KB | 2.6 KB |

The bulk is the SPARQL parser (`spargebra`/`peg`) + `oxttl`/`oxrdf` + `rand`
(transitively, for blank-node ids, unused here). Size-reduction levers for the
"minimal bundle" goal (future): drop the `rand` path, a leaner SPARQL parser or
parse-on-demand, `opt-level="z"` for the wasm profile, and `twiggy`-guided
pruning.

## Test

`test/smoke.cjs` loads a small graph and checks load, a triangle (WCOJ) query,
AVG aggregation, and language-tagged literals in the JSON output:

```sh
wasm-pack build --target nodejs --release --out-dir pkg-node
node test/smoke.cjs
```
