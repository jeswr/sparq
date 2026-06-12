# bench/serve — RESEARCH SPIKES for concurrent serving

**These are measurement spikes, not production code or maintained benchmarks.**
Each `src/bin/*.rs` is a one-shot harness whose numbers calibrate
`research/concurrent-serving.md`; the design verdicts live there, with full
machine context and interpretation. This is a deliberately *standalone* cargo
project (own `[workspace]`) so the repo's root workspace is untouched.

Binaries (all `cargo build --release` here, then `./target/release/<bin>`):

| bin | measures | research §|
|---|---|---|
| `loadgen` | sparq-server HTTP throughput/latency, closed + open loop (coordinated-omission-safe), head-of-line injection | d.i |
| `cache_spike` | result-cache hit-path ceiling (RwLock vs sharded vs Arc-swapped map), Zipfian repeats, all-distinct degenerate overhead | d.ii |
| `point_spike` | non-cached point-BGP SELECT→JSON ceiling, parse share, 1+N threads | a |
| `snapshot_spike` | `AppState` snapshot cost; update latency with/without a pinned reader generation; RSS per retained generation | d.iii |
| `update_stream_spike` | sustained one-UPDATE-per-resource-write stream (the Solid/prod-solid-server write profile): latency windows + RSS stability (the QLever #2481 OOM check), optional concurrent readers | d.iv |
| `stream_spike` | `query_json_chunks_with_budget` time-to-first-chunk vs full evaluation (streams in space, not time) | d.v |

Headline numbers (Apple M1, 4P+4E, 16 GB, macOS 26.4.1, 2026-06-12) are
tabulated in `research/concurrent-serving.md` §1; re-run on the target
hardware before trusting any absolute value.
