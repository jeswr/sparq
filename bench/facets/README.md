# Facet-count self-relative benchmark

<!-- [GPT-5.6] sq-ywe8p -->

This harness compares `sparq-introspect`'s indexed facet path with an intentionally
naive full-scan grouping oracle over the same deterministic generated graph and requests.
Every scenario must match in candidate count and the complete retained distributions
before the harness prints either timing row.

```sh
cargo run -p sparq-introspect --release --example facet_bench -- --smoke
cargo run -p sparq-introspect --release --example facet_bench
```

Rows are tab-separated as `scenario`, `implementation`, `count`, and elapsed
microseconds. The final JSON envelope always records `canonical:false`: work-box
timings are advisory, not publishable benchmark evidence. A canonical quiet-box run
must preserve the same equality gate.
