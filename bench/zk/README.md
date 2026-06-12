# sparq-zk commitment-pipeline benches

Criterion throughput benches for the stage-1 ZK commitment pipeline
(`crates/sparq-zk`): RDFC10 canonicalization, leaf encoding + Poseidon2-BN254
fold, end-to-end per-graph commitment, and the raw Poseidon2 primitives.

Standalone cargo project (own `[workspace]`, same isolation pattern as
`bench/parse` / `bench/serve`): criterion never touches the root workspace
or the wasm build.

Run:

```sh
cd bench/zk && cargo bench
```

Graph shapes: `iri` = all-ground triples (RDFC10 cheap path); `bnode` =
bnode chain with literal attributes (exercises canonical labeling,
non-pathological).

## Baseline (2026-06-13, fanless M1, zk-core branch @ post-verify-hook)

| bench | mean | throughput |
|---|--:|--:|
| canon/iri/64 | 513 µs | 124.8k triples/s |
| canon/iri/256 | 2.43 ms | 105.3k triples/s |
| canon/iri/1024 | 6.69 ms | 153.1k triples/s |
| canon/bnode/64 | 880 µs | 72.8k triples/s |
| canon/bnode/256 | 3.36 ms | 76.2k triples/s |
| canon/bnode/1024 | 9.32 ms | 109.9k triples/s |
| commit/iri/64 (end-to-end) | 8.90 ms | 7.2k triples/s |
| commit/iri/256 | 43.2 ms | 5.9k triples/s |
| commit/iri/1024 | 167.9 ms | 6.1k triples/s |
| commit/bnode/64 | 8.58 ms | 7.5k triples/s |
| commit/bnode/256 | 62.4 ms | 4.1k triples/s |
| commit/bnode/1024 | 186.6 ms | 5.5k triples/s |
| commit/leaves+fold/64 (canon precomputed) | 9.08 ms | 7.1k triples/s |
| commit/leaves+fold/256 | 26.5 ms | 9.7k triples/s |
| commit/leaves+fold/1024 | 128.6 ms | 8.0k triples/s |
| poseidon2/permutation | 92.5 µs | — |
| poseidon2/hash40 (commitment-scale fold) | 583 µs | — |

Reading of the baseline (empirical honesty, not spin):

- Canonicalization is NOT the bottleneck (~100k triples/s); the Poseidon2
  arithmetic is. End-to-end commitment sits at ~5–8k triples/s — fine for
  the stage-1 credential scale (tens-to-hundreds of triples per graph:
  ~10 ms/credential), but the Poseidon2 permutation at ~92 µs is a
  correctness-first port (noir-lang constants, straightforward ark-ff
  arithmetic, zero tuning). Known headroom if a later stage needs it:
  precomputed round-constant layout, fewer Montgomery conversions, batch
  leaf hashing. Do not optimize before a stage actually needs it — the
  cross-test against nargo pins the semantics either way.
- `leaves+fold` vs `commit` at n=64 are within noise of each other
  (canonicalization is a small fraction at that scale).
