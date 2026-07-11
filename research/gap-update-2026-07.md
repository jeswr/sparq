<!-- [SONNET-4.6] sq-hmd7l.5 — SPARQL-UPDATE competitor gap record. -->
# SPARQL UPDATE competitor gap — July 2026

Part of the comparative-benchmarking-everything program
(`research/comparative-benchmarking-everything.md` §4 row 16, epic sq-hmd7l).

## Status

**CANONICAL wave-1 rows recorded** (sq-hmd7l.26, quiet EC2 box — see the canonical
section below). Headline: **sparq is BEHIND oxigraph** on the PSS interactive-CRUD
update set (p99 ~6.6×, throughput ~8.2×); fuseki column absent (container readiness
failure, harness bead filed). The harness (`scripts/bench/update-same-box.sh`)
remains the durable deliverable.

## Harness

`bench/pss-update-set/compare.py` — extended in sq-hmd7l.5 to drive three engines:

| engine | kind | update endpoint | query endpoint |
|---|---|---|---|
| sparq | loopback `sparq-server` | `POST /sparql` (application/sparql-update) | `POST /sparql` (application/sparql-query) |
| fuseki | Docker `stain/jena-fuseki` | `POST /ds/update` | `POST /ds/query` |
| oxigraph | Docker `oxigraph/oxigraph` | `POST /update` | `POST /query` |

All three engines use the same HTTP `application/sparql-update` POST shape. The
workload is the PSS LDP-CRUD update set (interleaved DELETE/INSERT/DROP representing a
Solid pod-server's CRUD stream — `put_document`, `set_acl`, `delete_document` ops).

Shell orchestration: `scripts/bench/update-same-box.sh` (follows the `shacl-same-box.sh`
template: `ONLY=` engine filtering, `TIMEOUT_S` per-engine cap, `canonical:false` off the
quiet box, one JSON envelope per run).

## Store-state oracle (invariant)

After each engine completes the full workload, a COUNT query is run against its query
endpoint:

```sparql
SELECT (COUNT(*) AS ?n) WHERE { GRAPH ?g { ?s ?p ?o } }
```

This counts named-graph quads — the only graph layer the PSS update set writes to.
Disagreement across engines is recorded honestly in `count_crosscheck.all_agree`; latency
rows are never suppressed on a mismatch. This is the sq-hmd7l.5 INVARIANT.

## Parity gate

Relative only: `sparq p99 ≤ competitor p99 × tolerance` (default tolerance = 1.0). No
absolute wall-clock threshold is committed (forbidden by AGENTS.md). A single-engine
smoke run skips the gate entirely.

## Engine availability on this box

| engine | status | note |
|---|---|---|
| sparq | built from workspace | `cargo build --release -p sparq-server` |
| fuseki | Docker (requires daemon + image pull) | `stain/jena-fuseki`, in-memory dataset |
| oxigraph | Docker (requires daemon + image pull) | `oxigraph/oxigraph` |

Engines not runnable on the current box skip gracefully with `status: absent` in the
envelope. The smoke test (`ONLY=sparq bash scripts/bench/update-same-box.sh --smoke`)
exercises only the sparq loopback path.

## First-read rows

No first-read rows were recorded; the canonical wave-1 rows below are the first
measured numbers in this record.

## CANONICAL wave-1 rows (`sq-hmd7l.26`)

Provenance: dedicated quiet box `i-01a735e27b1764317` (c6i.4xlarge, eu-west-2, tag
`sparq-bench`, self-terminated, orphan-check clean), commit `cb1fc98c`
(= `origin/main` `343aee547` + wave-1 runner-only fixes), envelope UTC
`2026-07-10T00:47:07Z`, `CANONICAL=1`, 200 interactive PSS LDP-CRUD updates,
loopback HTTP for every engine (symmetric surface). Count crosscheck: **green** —
post-workload named-graph quad COUNT sparq 350 = oxigraph 350, `all_agree: True`.
Rows transcribed from the harness `compare.py` table in
`axis-results/update/run.log` (the envelope's row-ingest dropped them on the
parity-gate failure exit — harness bug, bead filed below; the log rows carry the
same provenance as the envelope in the same pulled directory).

| engine | p50 (ms) | p99 (ms) | max (ms) | throughput (/s) | status |
|---|---|---|---|---|---|
| sparq | 3.50 | 4.01 | 24.91 | 275 | ok |
| oxigraph | 0.45 | 0.61 | 0.84 | 2245 | ok |
| fuseki | — | — | — | — | absent (container never became ready — bead filed below) |

**Verdict: BEHIND.** sparq p99 4.01 ms vs oxigraph p99 0.61 ms (~6.6× behind); p50
3.50 ms vs 0.45 ms (~7.8×); throughput 275/s vs 2245/s (~8.2×). The harness parity
gate (`sparq p99 ≤ competitor p99 × 1.0`) FAILED. This is an honest engine-side gap:
both engines were measured over the same loopback HTTP POST surface, so server
framing does not explain it. Root-cause is unprofiled as of this record — plausible
directions for the profiling-first fix bead (to be filed by `sq-hmd7l.27`): per-update
parse/plan overhead, index maintenance on small interleaved DELETE/INSERT/DROP ops,
or allocation churn in the update path; max 24.91 ms also shows a long-tail outlier
absent in oxigraph (0.84 ms max). No spin: sparq loses this axis today.

Harness follow-ups filed from wave-1:

- **sq-do5fx** — `update-same-box.sh` envelope drops `latency_ms` /
  `count_crosscheck` when `compare.py` exits non-zero on a parity-gate FAIL — a
  measured FAIL is a valid result and must still be ingested into the envelope.
- **sq-l7diu** — the `stain/jena-fuseki` container did not become ready within the
  30 s window on a fresh box, leaving the fuseki column absent; switch the leg to
  the direct `apache-jena-fuseki` 5.4.0 distribution already proven by
  `fts-same-box.sh` on the same box (or extend readiness + surface the container log).

## BSBM Explore-and-Update

The BSBM Explore-and-Update workload (the standard multi-engine update benchmark suite)
is a follow-up axis. `bench/bsbm` notes it as EC2/nightly tier — heavier corpus and
longer run than the PSS interactive-CRUD set. Tracked separately from this record.

## Follow-ups

- sq-hmd7l.26: canonical wave-1 execution on a quiet EC2 box (this record's numbers land there)
- BSBM Explore-and-Update column: EC2/nightly tier follow-up (see `bench/bsbm`)
