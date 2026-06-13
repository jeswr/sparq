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
| `writer_spike` | Wave A2 group-commit writer (`sparq-serve`) vs A1 publish-per-update: writer throughput updates/s at `max_batch` 1/16/256, reader p50/p99 latency (open-loop, coordinated-omission-safe) under concurrent writer vs idle | 6.5 |

Headline numbers (Apple M1, 4P+4E, 16 GB, macOS 26.4.1, 2026-06-12) are
tabulated in `research/concurrent-serving.md` §1; re-run on the target
hardware before trusting any absolute value.

## writer_spike — recorded run (Apple M1, release, 2026-06-13, WRITER_UPDATES=30000)

**Writer throughput (closed-loop drain; in-flight capped at min(max_batch, 64) sync feeders):**

| max_batch | updates/s | generations | updates/gen |
|---|---:|---:|---:|
| 1 | 7,064 | 30,000 | 1.0 |
| 16 | 54,865 | 1,875 | 16.0 |
| 256 | 12,014 | 468 | 64.1 |

**Reader latency (open-loop, coordinated-omission-safe, 200µs schedule, 2s window):**

| scenario | p50 µs | p99 µs | max µs |
|---|---:|---:|---:|
| idle (no writer) | 69.4 | 84.5 | 150.7 |
| A2 writer max_batch=1 | 71.2 | 114.2 | 307.3 |
| A2 writer max_batch=16 | 70.9 | 113.8 | 2125.0 |
| A2 writer max_batch=256 | 69.8 | 93.3 | 331.0 |
| A1 publish-per-update | 70.1 | 96.3 | 384.4 |

Honest reading:
- **Group-commit wins on throughput**: batch-16 is ~7.8× batch-1 (54.9k vs 7.1k updates/s)
  — each generation amortises one fork/seal/publish over 16 updates instead of 1.
- **Batch-256 LOSES to batch-16** (12.0k vs 54.9k updates/s). This harness caps in-flight
  work at 64 sync feeders, so a window never actually collects 256 (updates/gen plateaus at
  ~64); the larger per-window seal then folds the pending delta (O(graph) compaction) more
  often, so a too-large `max_batch` is pure overhead here. The §6.5 default (256) only pays
  off when arrival concurrency can fill it — DON'T read this as "bigger batch = faster".
- **Readers never block in EITHER mode**: p50 ~70µs and p99 within ~1.3× of idle whether the
  writer is hammering (A2 any batch) or publishing per-update (A1). The batch-16 max of 2.1ms
  is sampler-thread sleep jitter, not a ring stall (its p99 is 114µs). The generation ring's
  lock-free `current()` is the load-bearing property; group-commit is a writer-side win only.
