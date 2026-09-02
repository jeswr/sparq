# pattern-scope overhead envelope (sq-lrtc3.3)

Measured cost of the masked-subgraph materialization design for pattern-scoped ODRL
targets — design record `research/odrl-pattern-scoped-targets-2026-07.md` (§4 defines
the three dimensions: scope-application/build cost, per-query overhead vs the
graph-granular view path, amortization break-even).

Reproduce:

```sh
cargo run -p sparq-solid --features pattern-scope --release --example pattern_scope_bench
```

Since [OPUS-5] `sq-nc3c6` (replica cache + write-path invalidation, design record §6)
the driver also emits the **repeat-scoped-query** dimension the cache exists for:
`scoped_build_cold_ms` (cache dropped before each timed run — the pre-cache cost) vs
`scoped_build_warm_ms` (served from the replica cache), and the per-iteration cost of a
repeat scoped-query loop with the cache live (`repeat_scoped_query_cached_ms`) vs. with
it dropped every iteration (`repeat_scoped_query_rebuilt_ms`, i.e. the pre-cache
behaviour of one full rebuild per scoped query). Files predating that bead carry the
older `scoped_build_ms` key instead, and none of the newer keys.

Results live in the dated `workbox-*.json` files next to this README. Work-box
numbers are **non-canonical** (the session host is a shared EC2 work box, not a quiet
benchmark instance); they bound the envelope's shape, not its canonical values.
No numbers are restated here or in any markdown — read the JSONs.
