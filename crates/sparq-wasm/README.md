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

## Browser memory bound (the scale ceiling)

The browser is the memory-constrained target — wasm32 linear memory caps at 4 GB and a
real tab is happier under ~2 GB. The wasm build therefore enables `compact-index`: only
**three** permutation indexes (SPO, POS, OSP) instead of six — every triple pattern is
still answered by one of them, but ~half the index memory (some merge joins fall back to
hashing). Measured in Node over synthetic N-Triples (`test/mem.cjs`, M1), with the same
compact prefix-factored dictionary + byte-level parser as native:

| triples | load | throughput | store | B/triple | browser ceiling |
|--:|--:|--:|--:|--:|---|
| 175 k | 110 ms | 1.6 M/s | 10 MB | 57 | ~35 M @2 GB / ~70 M @4 GB |
| 2.1 M | 1.37 s | 1.5 M/s | 140 MB | 67 | ~30 M @2 GB / ~60 M @4 GB |

So a browser tab holds roughly **30–70 M triples** — about double the six-permutation
build (the three permutations drop the index from 72 → 36 B/triple). The byte-level parser
keeps load throughput high (~1.5 M/s, single-threaded — wasm has no rayon) without
allocating an `oxrdf::Term` per term. Correctness of the reduced-permutation engine is
held to the same bar as native (198k differential fuzz cases vs Oxigraph with `compact-index`
on). The native build keeps all six permutations for maximum query speed.

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
