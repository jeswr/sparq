# pattern-scope overhead envelope (sq-lrtc3.3)

Measured cost of the masked-subgraph materialization design for pattern-scoped ODRL
targets — design record `research/odrl-pattern-scoped-targets-2026-07.md` (§4 defines
the three dimensions: scope-application/build cost, per-query overhead vs the
graph-granular view path, amortization break-even).

Reproduce:

```sh
cargo run -p sparq-solid --features pattern-scope --release --example pattern_scope_bench
```

Results live in the dated `workbox-*.json` files next to this README. Work-box
numbers are **non-canonical** (the session host is a shared EC2 work box, not a quiet
benchmark instance); they bound the envelope's shape, not its canonical values.
No numbers are restated here or in any markdown — read the JSONs.
