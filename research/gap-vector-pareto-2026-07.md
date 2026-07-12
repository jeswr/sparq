# Internal vector-index Pareto envelope

<!-- [GPT-5.6] sq-0ut5x -->

`crates/sparq-vectors/examples/pareto_bench.rs` measures sparq's three approximate index kinds on
the deterministic vector benchmark corpora. It sweeps each index's build-quality parameters and
records recall@10 against the crate's exact-kNN oracle, advisory build time, and advisory query
latency.

The harness refuses to emit an envelope if any measured configuration misses its established
recall floor: HNSW and PQ require at least 0.95 recall@10; DiskANN requires at least 0.90. HNSW uses
the inclusive floor rather than exact recall equality because its rayon-parallel construction can
move a boundary neighbour. Only rows that pass this gate enter the per-index Pareto calculation.

Run the fast acceptance workload with:

```sh
cargo run -p sparq-vectors --release --example pareto_bench --features approx-ann -- --smoke
```

The JSON output is schema `sparq.vector-pareto.v1`. Its timings are explicitly non-canonical until
the full workload is gathered on a quiet benchmark host; this record therefore makes no fixed
performance claim.
