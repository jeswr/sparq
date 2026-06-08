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
scarcest. The whole `count()` family is **lazy** — counted straight from the sorted
indexes with no result row built per solution — so "how many?" UI queries stay on the
main thread without jank. Measured in Node over a 400k-triple graph (`test/perf.cjs`,
2020 M1 MacBook Air):

| query | time |
|---|--:|
| `SELECT * … LIMIT 10` (early-termination, no full scan) | **0.06 ms** |
| `count(?s a Person)` (lazy, index range size) | **0.02 ms** |
| `count(follows · name)` (count-only 2-pattern join) | 4.2 ms |
| `count(name·age·city)` (lazy **N-pattern star** count, Σ_s Π c_i(s)) | **2.5 ms** |
| `count(FILTER(?a > 90))` (binary-searched range size on inline ints) | **0.07 ms** |
| `count(name OPTIONAL age)` (lazy left-join, Σ_s c_l·max(1,c_r)) | **2.1 ms** |
| `FILTER(?a > 90)` **materialise** (same predicate, builds rows) | 7.0 ms |
| `OPTIONAL` **materialise** (sort-merge left join, builds rows) | 167 ms |

The lazy-count wins built for the native engine carry over unchanged (same
`sparq-core`/`sparq-engine`): counting a filtered pattern is **~95× faster** than
materialising it (0.07 vs 7 ms), and counting an OPTIONAL is **~80× faster** than
materialising it (2.1 vs 167 ms) — a large saving on a memory-constrained device.

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
