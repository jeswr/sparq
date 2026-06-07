# Benchmarks

`crates/sparq-bench` runs an identical dataset and query workload through **sparq**
and through **Oxigraph** (a mature, widely-used Rust SPARQL engine), and:

1. checks that both return the **same number of solutions** for every query — a
   differential correctness cross-check against an independent implementation;
2. reports **load time**, **per-query time** (min of *K* iterations, warm), and
   **peak process RSS** (measured in a clean subprocess, right after load).

```
cargo run -p sparq-bench --release -- --scale 50000 --iters 4
```

## Methodology & honesty notes

- **What this is.** A like-for-like, in-process comparison on a synthetic
  WatDiv-flavoured social graph (each entity is a `ex:Person` with name/age/city
  attributes and `ex:follows` edges → star joins, chains, triangles).
- **What this is *not* (yet).** Oxigraph is *a* SOTA-class engine but not the
  fastest; QLever and RDFox are the engines to beat on the public benchmarks, and
  those comparisons (SP2Bench, WatDiv, WDBench, etc. from docs.qlever.dev) are
  still TODO. We compare against Oxigraph first because it embeds cleanly as a
  Rust crate, giving a continuous, reproducible correctness + perf gate.
- **Memory caveat.** Oxigraph's *in-memory* `Store` (`Store::new()`, no RocksDB)
  is measured. Oxigraph's primary, optimised backend is on-disk RocksDB; the
  in-memory store is not its most space-efficient mode, so the memory comparison
  flatters sparq somewhat. Peak RSS also includes the parse buffer (the same
  Turtle bytes for both). sparq additionally self-reports its store footprint
  (dictionary + six permutation indexes).
- Timings are min-of-K (warm cache). Measured on the development machine
  (Apple Silicon, darwin); absolute numbers are machine-specific, ratios less so.

## Results — scale 50 000 entities (~400 k triples)

All seven queries returned **identical solution counts** to Oxigraph.

| stage / query | sparq | oxigraph | speedup |
|---|--:|--:|--:|
| load          | 577 ms | 615 ms | 1.07× |
| scan-type     | 4.27 ms | 21.7 ms | **5.08×** |
| star-3        | 20.8 ms | 79.7 ms | **3.82×** |
| chain-2 (800k rows) | 170 ms | 568 ms | **3.35×** |
| triangle (cyclic → WCOJ) | 310 ms | 679 ms | **2.19×** |
| filter-age    | 7.96 ms | 11.5 ms | 1.44× |
| count-edges   | 6.72 ms | 22.3 ms | **3.32×** |
| optional      | 20.6 ms | 48.4 ms | 2.34× |
| **peak RSS (after load)** | **71 MiB** | 178 MiB | **2.5× lighter** |

(sparq store self-estimate: 42.9 MiB for 400 k triples ≈ 6 permutations × 12 B +
dictionary.)

### Reading the numbers

- sparq is **1.4×–5.1× faster** on every query in the workload, and ~par on load
  (it builds six sorted permutations, which costs more than a single index but
  buys single-range pattern scans and merge-join-ready orderings).
- The **triangle** is the WCOJ case: Leapfrog Triejoin avoids the large
  intermediate a binary plan would build. It still wins here (2.19×); the gap
  grows on denser/skewed cyclic data (where binary intermediates blow up).
- Memory is *better* than Oxigraph's in-memory store at rest despite six
  permutations — but see the caveat above; the real memory test is vs QLever's
  compressed columns, and column compression (M3) is the lever there.

## Next

- Compare against QLever and (where feasible) RDFox on the public benchmark
  datasets and query sets from docs.qlever.dev/benchmarks.
- Larger scales; cold-cache and QMpH (queries/hour) metrics.
- Re-measure memory against QLever (compressed) and after M3 column compression.
