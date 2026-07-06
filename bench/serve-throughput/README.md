# bench/serve-throughput — canonical loopback HTTP-throughput harness

> 🤖 SPARQ agent. sq-7d3dj.5 (epic sq-7d3dj, roadmap item 5 of
> `research/optimization-audit-2026-07.md`).

The sparq HTTP surface had **no throughput metric** anywhere in `bench/`, so every
HTTP-lane optimisation (sq-7d3dj.10 / .12 / .13) was unmeasurable. This harness is the
**prerequisite** that unblocks that lane. It stands up the **real** in-process server —
`sparq_server::serve` + `router` + `AppState`, the exact stack the `sparq` binary runs —
on an ephemeral `127.0.0.1:0` loopback port over a small fixed synthetic corpus, drives
concurrent SPARQL SELECT/ASK queries against it with a dependency-free keep-alive HTTP/1.1
client, and reports **{req/s, p50/p99 latency (ms), peak RSS}**.

It changes **no server behaviour** — it is measurement infrastructure only. Like
`bench/serve` and `bench/memtier`, it is a deliberately *standalone* cargo project (own
`[workspace]`), so the repo's root workspace and its clippy/test gate are untouched.

## Honesty — NON-CANONICAL on a shared box

`req/s` and latency are **wall-clock sensitive**. On this work box (or any busy host) the
absolute numbers are **directional brackets only**, never a claim — exactly how the
competitor-gather and the perf-gate treat every wall-clock metric. The **canonical**
numbers come from a **quiet EC2 runner** (the `ci-bench-ec2` disposition). Nothing here is
wired into the deterministic perf-gate (`scripts/perf-gate.py`): `req/s` is a **trend / EC2
metric**, not a gated ratchet. Every report carries the NON-CANONICAL note — as a header
comment line (`# …`) on the text summary, and as a top-level `"note"` field (plus
`"canonical": false`) in the `--json` document.

## What it measures

| series | what | why |
|---|---|---|
| throughput | a fixed small SELECT (bound-subject point query) + an ASK, at concurrency **1/8/32** (the server's default `max_concurrent` is 32), closed-loop | req/s + p50/p99 ms per concurrency level — the HTTP-lane baseline |
| peak RSS | process `VmHWM` high-water mark while serving one **large** SELECT (full-graph scan, whole result materialised) | captures the whole-result materialisation cost the streaming beads (sq-7d3dj.12 / .13) target |

Closed-loop (each connection fires back-to-back) measures the server's **saturation
throughput** at a concurrency level; latency is interpreted relative to that level (the
classic closed-loop caveat).

## Run

Build + run here, or via the sibling script:

```sh
# in bench/serve-throughput
cargo run --release --bin serve_throughput
cargo run --release --bin serve_throughput -- --triples 200000 --conns 1,8,32 --duration 5 \
    --json /tmp/serve-throughput.json --label ec2

# from the repo root — builds + runs + optional --json emit
scripts/serve-throughput-bench.sh                         # default run
scripts/serve-throughput-bench.sh --smoke                 # fast tiny CI-gate run
scripts/serve-throughput-bench.sh --json /tmp/st.json     # + machine-readable emit
```

Flags: `--triples N` (corpus size), `--conns a,b,c` (concurrency levels), `--duration S`
(seconds per measured cell), `--warmup S`, `--json PATH` (strictly-additive,
dependency-free document; STDOUT is unchanged), `--label TAG`, `--smoke` (fast gate run).

## Registration

Registered in `bench/benchmarks.toml` as `serve-throughput` (category `serve`,
`quiet_box_sensitive = true`). The `serve` family is a non-capability family, so it is
correctly exempt from a dashboard card (`scripts/check-new-bench-registered.py`).

## Unblocks

sq-7d3dj.10 (multi-core SELECT-JSON serialize), sq-7d3dj.12 (chunked CSV/TSV streaming),
sq-7d3dj.13 (pull-streaming body) — none of which may claim a win before a canonical
before/after run of this harness on the EC2 runner.

## License

MIT (workspace default).
