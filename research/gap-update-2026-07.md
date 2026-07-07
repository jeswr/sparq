<!-- [SONNET-4.6] sq-hmd7l.5 — SPARQL-UPDATE competitor gap record. -->
# SPARQL UPDATE competitor gap — July 2026

Part of the comparative-benchmarking-everything program
(`research/comparative-benchmarking-everything.md` §4 row 16, epic sq-hmd7l).

## Status

**NON-canonical first read.** The harness (`scripts/bench/update-same-box.sh`) is the
durable deliverable. Numbers from the shared work box are directional only — do not bake
into docs or dashboards. Canonical numbers ride sq-hmd7l.26 (quiet EC2 run).

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

**NON-canonical — work box, not a quiet EC2 instance.** All numbers below are
directional only.

No canonical rows yet. See sq-hmd7l.26 for the scheduled quiet-box harvest.

## BSBM Explore-and-Update

The BSBM Explore-and-Update workload (the standard multi-engine update benchmark suite)
is a follow-up axis. `bench/bsbm` notes it as EC2/nightly tier — heavier corpus and
longer run than the PSS interactive-CRUD set. Tracked separately from this record.

## Follow-ups

- sq-hmd7l.26: canonical wave-1 execution on a quiet EC2 box (this record's numbers land there)
- BSBM Explore-and-Update column: EC2/nightly tier follow-up (see `bench/bsbm`)
