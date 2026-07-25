# bench/federation — comparative federation panel (FedShop-shaped)

[FABLE-5] sq-hmd7l.12 — the first comparative END-TO-END federation harness
(registry id `federation-fedshop`; the fedplan criterion micro is NOT comparative).

## What it measures

A FedShop-shaped shop federation — a **vendor** member (offers) + a **ratingsite**
member (reviews), sharing product IRIs — is generated in-repo (deterministic seed,
per-member sha256 recorded) and served by **2 local sparq-server member endpoints**
(same-box replicas: network variance controlled; LargeRDFBench's public endpoints
are dead, so local replicas are the only honest option). A request-**counting
reverse proxy** sits in front of each member so every engine's per-member HTTP
request counts are measured uniformly.

Engines, over the SAME FedShop-shaped federated queries:

| column | what runs | regime |
|---|---|---|
| `sparq` | sparq-server `--features service` (SERVICE eval + bound-join) | explicit SERVICE |
| `comunica` | `@comunica/query-sparql` (reference federated-SPARQL engine) | explicit SERVICE **and** virtual (`sources=[all members]`, engine-side source selection) |
| `jena` | Apache Jena `arq` (naive SERVICE baseline) | explicit SERVICE — optional: `FED_JENA_ARQ=/path/to/arq` |

**Invariant (never wall time alone):** per query, the canonical **result-set
multisets must agree across engines BEFORE any timing is reported**; per-member
request counts + source-selection precision/recall (vs *measured* per-member
block-relevance ground truth) are always reported alongside wall time.

Timing regimes differ by engine nature and are recorded in the results JSON, not
hidden: sparq = HTTP round-trip to the federator; comunica = engine-internal exec
(process startup excluded); jena = process wall including JVM startup.

## Run it

```sh
cargo build --release -p sparq-server --features service
bash bench/federation/run.sh --smoke   # acceptance: one query, sparq-vs-Comunica agreement
bash bench/federation/run.sh           # full panel: 5 queries, timed, explicit + virtual
```

Comunica is installed **at gather time** into `bench/federation/node_modules`
(git-ignored, never a committed dependency); the resolved npm version is recorded
in the results JSON. Default spec `@comunica/query-sparql@^4` (v5 needs Node ≥ 22).
Results: `bench/competitor-results/federation-fedshop-<UTC>.json` (git-ignored).
Tunables (`FED_SCALE`, `FED_ITERS`, `FED_PORT_BASE`, …): see the `run.sh` header.
Hermetic unit layer: `python3 bench/federation/compare.py --self-test`.

## Honest scope (v1)

- The corpus is FedShop-**shaped** (BSBM-style shop federation, cross-member joins
  on product IRIs), generated in-repo — not the upstream dockerized FedShop
  distribution. Same-box local replicas are the variance-controlled equivalent.
- sparq has **no server-exposed virtual-federation endpoint** (sparq-fedclient is
  a library) — the sparq virtual-regime cell is an honest `n/a`; a fedclient
  runner column is tracked as follow-up bead sq-hmd7l.47.
- SolidBench (pod-shaped federation over many small sources) is a follow-up
  candidate suite, not covered here.
- Numbers gathered on a shared work box are **NON-canonical** (bench/CATALOG.md
  rules); first-read observations live in `research/gap-federation-2026-07.md`,
  flagged as such.
