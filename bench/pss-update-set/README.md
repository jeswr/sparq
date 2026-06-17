<!-- [OPUS-4.8] sq-b4lo / gh-52 — re-review when Fable returns. -->
# sparq vs QLever — PSS write-path parity (the single-writer acceptance bar)

The **write-path** companion to [`bench/qlever-olympics`](../qlever-olympics/) (which
compares the **read** path). This directory holds the benchmark that backs gh-52's
Phase-2 acceptance criterion for sparq's documented serving contract — *"N readers
against 1 sequenced writer + external coordination"*
(see [`crates/sparq-server/README.md`](../../crates/sparq-server/README.md) →
**Concurrency contract**).

## The criterion (gh-52, set by the PSS agent)

> **Parity-or-better vs the QLever-over-HTTP write path on PSS's actual update set** —
> NOT bulk ingest. Reference starting targets (non-gating): single-writer sustained
> *≥ a few hundred small updates/sec* and *p99 write-commit < ~50 ms*. The **binding**
> gate is QLever-parity, because a migration acceptance bar is relative, not absolute.

## The update set

The interactive **LDP-CRUD** shapes a Solid pod-server issues — small
`DELETE … ; INSERT DATA …` per resource + its `.acl` write, plus the heaviest burst
(pod **provisioning**). These are the exact shapes pinned for *correctness* in
[`crates/sparq-server/tests/named_graphs.rs`](../../crates/sparq-server/tests/named_graphs.rs)
(gh-47) — here measured for *throughput/latency*:

| op | shape | PSS operation |
|---|---|---|
| `put_document`   | `DELETE WHERE { GRAPH <r> {…} } ; INSERT DATA { GRAPH <r> {…} GRAPH <c> {…ldp:contains…} }` | `putDocument` |
| `set_acl`        | `DELETE {…} INSERT {…} WHERE { OPTIONAL {…} }` (idempotent single-valued pointer) | `setAclPointer` / `putContainer` |
| `delete_document`| `DROP SILENT GRAPH <r> ; DELETE WHERE { GRAPH <c> {…ldp:contains…} }` | resource DELETE |
| `provision`      | one multi-op `INSERT DATA` creating a container + child resources + `.acl`s | pod provisioning burst |

## Two ways to run it

### 1. sparq-only smoke (no QLever) — fast, in-process

The single-writer harness drives sparq's **production** `AppState` writer (the same
`apply_update` the HTTP `application/sparql-update` path calls) through the update set
and reports sustained writes/s + commit-latency p50/p99/max. Optionally gate against a
*recorded* QLever p99:

```sh
# build JUST this bin (the other bench/serve spikes carry their own API drift):
( cd bench/serve && cargo build --release --bin pss_update_throughput )
./bench/serve/target/release/pss_update_throughput --profile crud --updates 20000 --readers 4
./bench/serve/target/release/pss_update_throughput --profile provision --updates 2000 --children 8
# parity gate against a recorded QLever p99 (ms); exit 1 on regression:
./bench/serve/target/release/pss_update_throughput --profile crud --qlever-baseline-ms <recorded> --tolerance 1.0
```

`--readers N` runs concurrent point-query readers; their query count is reported (not
gated) — the "readers never block the writer" half of the contract.
`--base-triples N` sizes the pre-existing working set (default 1M is a large-tenant
stress case; PSS's real per-pod set is KBs–MBs, e.g. `--base-triples 50000`).

**Read the p99, not the single-client throughput.** `pss_update_throughput` is a SINGLE
synchronous client (the interactive PSS shape — each LDP request blocks on its own
commit), so its `updates/s` is bounded by `1/group-commit-window` (≈ 333/s at the 3 ms
default) *no matter how fast the writer is* — one client never fills a batch window. The
**binding interactive metric is p99 commit latency** (must sit inside the LDP request
budget). The writer's true aggregate ceiling under CONCURRENT clients is
[`writer_spike`](../serve/README.md#writer_spike) (group-commit throughput at
`max_batch` 1/16/256), not this binary.

### 2. true differential vs a running QLever — the binding gate

`compare.py` fires the **same** updates at BOTH a running sparq-server and a running
QLever endpoint over HTTP, on one box, in one run, and asserts sparq parity-or-better.
This is the only fair way to assert the *relative* bar (QLever's write numbers are
machine-specific and are deliberately **not** committed anywhere — repo hygiene forbids
hard-coded perf).

```sh
# sparq (no auth = writable):
cargo run -p sparq-server --release -- --addr 127.0.0.1:7020

# QLever (see ../qlever-olympics/README.md for the venv + qlever CLI; the Qleverfile
# must enable updates and an ACCESS_TOKEN):
../../.qlever-venv/bin/qlever start          # serving on :7019

python3 compare.py 200 --qlever-token <ACCESS_TOKEN>   # 200 interactive updates each
```

`compare.py` exits 1 if sparq's p99 regresses past `QLever p99 × --tolerance`
(default 1.0 = exact parity-or-better).

## Why this isn't a horizontal-scale benchmark

gh-52's scope was explicitly locked to **option (a)** — the documented contract +
this acceptance bar — and **not** a distributed/replicated writer (option (b)), which
PSS confirmed is over-engineering for the single-instance Phase-2 deployment. The
single sequenced writer IS the write ceiling by design; this benchmark proves it clears
the QLever-parity bar PSS needs, it does not try to scale past it. The
horizontal-scaling design (if/when tenancy demands it) lives in
[`research/adr-horizontal-scaling.md`](../../research/adr-horizontal-scaling.md).
