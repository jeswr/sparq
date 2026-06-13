# zk-trace overhead benches

Criterion benches for the zk-trace seam (`crates/sparq-engine` `zk` feature,
plan §4.E module B): per-operator proof-input capture cost, **traced** (recorder
armed + drained each iteration, the proving path) vs **untraced** (default
execution), across every plan shape the trace covers.

Standalone cargo project (own `[workspace]`, same isolation pattern as
`bench/zk` / `bench/parse` / `bench/serve`): criterion never touches the root
workspace or the wasm build. `sparq-engine` is pulled WITH the `zk` feature, so
this project is deliberately NOT a root-workspace member.

Run:

```sh
cd bench/zk-trace && cargo bench
```

Workloads (`100` and `1000` synthetic entities; ~9 triples/entity): BGP star /
chain / triangle (WCOJ), FILTER, OPTIONAL, UNION, DISTINCT, COUNT.

The honest point of these benches: capture is NOT free when armed — the
recorder materializes the consumed input set (dictionary ids deduped, terms
once per unique triple) and forces result-preserving plan changes (disabled
COUNT/LIMIT/sargable pushdowns). The `untraced` arm confirms the disarmed cost
is the one-thread-local-read floor.

## Baseline

Record `criterion` means here once measured on the reference machine.
