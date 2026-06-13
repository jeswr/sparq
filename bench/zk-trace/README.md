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

This criterion bench compares untraced vs traced (recorder armed + drained each iter)
execution of bgp-star / chain / triangle / filter / optional over 100 entities
(~900 triples), reporting the per-query trace-capture overhead. Run it for the numbers:

```sh
cd bench/zk-trace && cargo bench
```

Honest read: capture is a constant-factor cost (a few × at this scale) — it
materializes the full consumed input set (dictionary ids deduped, terms once
per unique triple) and forces the result-preserving plan changes (disabled
COUNT/LIMIT/sargable pushdowns). This is the proving path, run once per proof,
on credential-scale data — not the hot query path. When the recorder is
DISARMED the cost is one thread-local bool read per scan (the `zk` feature is
non-default and entirely cfg'd out of release/wasm builds).
