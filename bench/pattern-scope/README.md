# pattern-scope overhead envelope (sq-lrtc3.3)

Measured cost of the masked-subgraph materialization design for pattern-scoped ODRL
targets — design record `research/odrl-pattern-scoped-targets-2026-07.md` (§4 defines
the three dimensions: scope-application/build cost, per-query overhead vs the
graph-granular view path, amortization break-even).

Reproduce:

```sh
cargo run -p sparq-solid --features pattern-scope --release --example pattern_scope_bench
```

Since sq-nc3c6 (§6, the bounded sharded replica cache) the driver also reports the
**amortization** the cache buys, so build cost is split into two fields rather than one:

- `cold_build_ms` — `scoped_dataset` on a FRESH store (empty replica cache). Sampled
  over fresh stores on purpose: a repeat call on the same store is a cache hit, so a
  best-of loop over one store would silently report the warm number under the old
  `scoped_build_ms` name.
- `warm_build_ms` — the same call once the scope class is cached: what a repeat scoped
  query actually pays.
- `first_build_query_ms` / `repeat10_build_query_ms` — one cold (build + query) round,
  and ten consecutive rounds in total, i.e. the curve a session issuing several queries
  under one scope sees.

The `workbox-2026-07-11.json` file predates the cache and carries the older
`scoped_build_ms` field; it is kept as the pre-cache baseline, not deleted.

Results live in the dated `workbox-*.json` files next to this README. Work-box
numbers are **non-canonical** (the session host is a shared EC2 work box, not a quiet
benchmark instance); they bound the envelope's shape, not its canonical values.
No numbers are restated here or in any markdown — read the JSONs.
