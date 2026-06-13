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

## Baseline

The criterion benches above report mean time + triples/s throughput for RDFC10
canonicalization (`canon/{iri,bnode}/{64,256,1024}`), end-to-end commitment
(`commit/…`), the canon-precomputed leaves+fold path, and the raw Poseidon2
primitives. Run `cargo bench` (above) for the numbers — criterion writes its own
reports under `target/criterion`. The load-bearing qualitative reading:

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
