<!-- [FABLE-5] sq-hmd7l.21 — GSP same-box panel. sq-hmd7l.36 — PATCH-dialect panel. -->
# Graph Store Protocol same-box panel — sparq-server vs Fuseki GSP / Oxigraph GSP (+ CSS, loose)

HTTP **Graph Store Protocol** (SPARQL 1.1 GSP) PUT/GET/DELETE/PATCH
round-trips, all engines on one box, **loopback only** (the driver hard-refuses
any non-loopback URL — no bypass flag). Registered as `gsp-bench` in
[`bench/benchmarks.toml`](../benchmarks.toml); lineage:
[`bench/serve-throughput`](../serve-throughput/) (loopback HTTP regime) +
[`bench/pss-update-set`](../pss-update-set/) (the LDP-CRUD shapes).

## The invariant: round-trip content agreement BEFORE timing

Per workload and per engine, a HARD correctness gate runs first; a red gate
skips that engine's timing and fails the run (exit ≠ 0):

- **size-sweep** — PUT a deterministic, bnode-free, ASCII-only N-Triples payload
  (default 1 KB → 10 MB), GET it back as `application/n-triples`, and require
  the returned triple **set to equal the sent set exactly** (ground triples, so
  set equality *is* isomorphism); then DELETE and require an absent read
  (404, or 200-with-zero-triples — engines legitimately differ here).
- **crud-replay** — the PSS LDP-CRUD stream projected onto GSP verbs; a
  client-side reference model replays the same stream and every touched graph's
  final state must match a GET against the engine.
- **patch-panel** (`sq-hmd7l.36`) — RFC 5789 PATCH per **dialect**: a
  deterministic ground delete/insert patch stream (replace / insert-only /
  delete-only shapes) over a seeded graph, gated on final-content agreement
  with the client-side reference model. Dialects: `application/sparql-update`
  (`DELETE DATA`/`INSERT DATA`; GRAPH-wrapped for GSP addressing, bare for
  CSS's LDP resources) and `text/n3` Solid **N3-Patch**
  (`solid:InsertDeletePatch`). Support is **probed, never assumed**: a
  405/415/501 on the first patch records an honest **n/a** row —
  Fuseki/Oxigraph GSP implement no PATCH dialect, and a stock sparq build
  answers 415 on `text/n3` (that dialect is double-opt-in: the `n3-patch`
  cargo feature **and** the runtime flag; set `GSP_N3_PATCH=1` on a
  `--features n3-patch` build to light it up). A 2xx with wrong final content
  is a red gate, not an n/a.

The gates are themselves regression-tested: `compare.py --self-test` (hermetic,
no HTTP) proves an honest in-memory store passes them, a tampering store (drops
a triple on read) and a lossy PATCH store (204 but ignores deletes) fail them,
and a 405-only store records n/a.

## Columns

| column | GSP endpoint | posture |
|---|---|---|
| sparq-server | `<base>/sparql/graph?graph=<iri>` | subject |
| Fuseki | `<dataset>/data?graph=<iri>` | like-for-like HTTP RDF store |
| Oxigraph server | `<base>/store?graph=<iri>` | like-for-like HTTP RDF store |
| Community Solid Server | LDP resource URL (direct) | **LOOSE** — Solid/LDP stack (persistence, authz, negotiation) is INCLUDED; labelled on every row, **never averaged** with the like-for-like columns |

CRUD projection honesty: `set_acl`'s conditional `DELETE/INSERT/WHERE` swap is
not GSP-expressible — it is projected to a PUT-replace of a one-triple acl
graph; `delete_document` cannot remove the single containment triple from the
container graph (that needs PATCH), so containment survives in BOTH the engine
and the reference model. PATCH-dialect honesty: **CSS is the only peer** for
the Solid PATCH dialects — a **loose** comparison by construction, labelled on
every row; Fuseki/Oxigraph record honest n/a there.

## Run it

```sh
cargo build -p sparq-server --release
bash bench/gsp/run.sh --smoke     # acceptance: sparq loopback, small sizes; exit 0 = green
bash bench/gsp/run.sh             # full 1 KB..10 MB sweep + 200-op CRUD replay
```

Competitor columns attach via env (each server already running on loopback;
see `scripts/fuseki-same-box.sh` / `scripts/oxigraph-serve-same-box.sh` for the
pinned engine recipes):

```sh
GSP_FUSEKI_URL=http://127.0.0.1:3030/ds \
GSP_OXIGRAPH_URL=http://127.0.0.1:7878 \
GSP_CSS_URL=http://127.0.0.1:3000 \
GSP_JSON_OUT=bench/competitor-results/gsp-$(date -u +%Y%m%dT%H%M%SZ).json \
bash bench/gsp/run.sh
```

Tunables: `SPARQ_SERVER_BIN`, `GSP_PORT` (7033), `GSP_ITERS`, `GSP_SIZES`,
`GSP_PATCH_OPS`, `GSP_N3_PATCH` (1 = run sparq with the opt-in `text/n3`
N3-Patch dialect enabled; needs a `--features n3-patch` build — on a stock
build the env var is ignored and the row honestly reads n/a),
`GSP_MAX_BODY_BYTES` (the server's default 1 MiB `--max-body-bytes` is raised
to clear the 10 MB PUT bodies — configure competitors equivalently).

## Honesty

Latency/throughput here are wall-clock and **NON-canonical on a shared box**
(`bench/CATALOG.md` QUIET-BOX rule); canonical numbers come only from a quiet
EC2 run, land git-ignored under `bench/competitor-results/`, and are never
committed. First-read findings: `research/gap-gsp-2026-07.md`.
