<!-- [FABLE-5] sq-i858h first-read gap record (epic sq-hmd7l). The bundle-bytes
section is DETERMINISTIC (toolchain-pinned wasm-pack --release artifact). ALL
latency rows are NON-CANONICAL first reads from the shared work box; a canonical
gather sets CANONICAL=1 on a dedicated quiet EC2 box (tag sparq-bench). -->

# Browser SHACL competitor gap — first-read record (2026-07-12)

**Status:** harness delivered + first-read gathered. Agreement gate GREEN on
every workload; all latency rows ADVISORY (`canonical:false`).
**Bead:** sq-i858h. **Epic:** sq-hmd7l (comparative-benchmarking-everything).
**Harness:** `bench/shacl-wasm/` — `run.sh --smoke` (acceptance),
`run.sh` (best-of-N + envelope). Both engines run **in the same Node process**;
**no timing row without a green per-workload `#violations` + `conforms`
agreement gate**, each engine also matched against the hand-derived
`expected.tsv` constants (both-engines-broken cannot pass as agreement).

This is the **browser half** of the SHACL competitor story. The native
same-box half (sparq vs pySHACL vs Jena SHACL) is
`scripts/bench/shacl-same-box.sh` + `research/shacl-baseline-2026-07.md`.

## Pins (recorded in the envelope at gather time)

- **sparq-shacl-wasm** — `wasm-pack build --target nodejs --release`, default
  features (no `shacl-af`), wasm-pack 0.15.0, this repo at the gather commit.
- **rdf-validate-shacl 0.6.5** (Zazuko; `bench/competitors.json` id
  `rdf-validate-shacl`, kind `js-lib`) + the `@rdfjs/*`/clownface stack from
  the registry's install recipe — gather-only npm deps, never committed.
- **Runtime** node v20.20.2, same process for both engines.
- **Data × shapes** — the committed `bench/shacl` workloads verbatim
  (`shapes/*.ttl` + `shapes-sparql/sparql_heavy.ttl`) over the vendored
  hand-countable micro-ABox `bench/shacl-wasm/data/abox.ttl`.

## Correctness — cross-engine agreement (deterministic)

All four SHACL-Core workloads agree exactly, per-workload, on both
`#violations` and `conforms`, and match the hand-derived constants:

| workload | expected | sparq-shacl-wasm | rdf-validate-shacl | gate |
|---|---:|---:|---:|---|
| cardinality | 6 | 6 | 6 | green |
| class_nodekind | 4 | 4 | 4 | green |
| datatype_range | 5 | 5 | 5 | green |
| node_paths | 4 | 4 | 4 | green |
| sparql_constraint | 4 | 4 | — capability-absent | green (sparq-only) |
| sparql_heavy | 10 | 10 | — capability-absent | green (sparq-only) |

**Capability gap (sparq AHEAD):** `rdf-validate-shacl` implements SHACL Core
only — no SHACL-SPARQL (`sh:sparql`, W3C SHACL §5.2). Its column on the two
`sh:sparql` workloads is **absent, never a fabricated 0**; sparq is
self-asserted against the vendored constants there. In the browser runtime,
sparq is the only engine of this pair that can run SPARQL-constraint shapes.

## DETERMINISTIC — bundle bytes (wasm-pack --release, nodejs target)

Recorded per toolchain in every envelope (wasm-pack 0.15.0, 2026-07-12):

| artifact | bytes | gzip-9 (wire) |
|---|---:|---:|
| `sparq_shacl_wasm_bg.wasm` | 2 430 436 | 884 451 |
| `sparq_shacl_wasm.js` (glue) | 10 988 | 2 786 |

**Honest read (sparq BEHIND on raw payload, by design-tradeoff):** a pure-JS
SHACL library ships far fewer wire bytes than a wasm engine bundle. The peer's
unpacked npm footprint for the exact registry dep set is ~10.4 MB, but that is
**not** a wire size (no tree-shaking/minification), so this record makes **no
byte-ratio claim** — a bundler-built minified peer bundle is the comparable
number and is follow-up work (bead below). Mitigations already shipped: the
SHACL bundle is a **separate, lazy-loaded** artifact (never on the landing
page; `next/dynamic` on `/surface/shacl` only), built `-Oz`, `shacl-af` off by
default.

## ADVISORY — same-runtime latency, first read (NON-canonical, shared work box)

One-shot end-to-end (parse data + shapes + validate + reduce), best-of-3,
micro-ABox (envelope `shacl-wasm-vendored-20260712T023932Z.json`; directional
only — do NOT bake into docs/dashboards):

- sparq-shacl-wasm one-shot e2e: ~0.6–1.0 ms per core workload.
- rdf-validate-shacl one-shot e2e: ~15–23 ms (dominated by the RDF/JS
  streaming-parser setup per document).
- rdf-validate-shacl **validate-only** on pre-parsed datasets (the steady-state
  an RDF/JS app sees): ~1.1–2.1 ms — i.e. even the peer's parse-free
  steady-state is slower than sparq's parse-included one-shot on this corpus.

Directionally sparq LEADS on every timed workload at this (micro) scale.
Caveats before any dominance claim: the corpus is deliberately tiny (the gate
substrate, not a load test), the box is shared, and the peer's e2e is
parser-bound. A scale-tier browser corpus + quiet-box gather is where a citable
number would come from (sq-hmd7l.39/40 wasm-compare wave; SHACL can join it).

## Surface gap noted (sparq)

`sparq-shacl-wasm`'s `Validator` is deliberately **stateless one-shot** — there
is no pre-parsed/persistent-graph validate on this bundle, so a validate-only
column for sparq is structurally absent (the lean `sparq-wasm` bundle's
`Store.validate` behind its `shacl` feature is that shape). At micro scale the
one-shot still wins; at scale-tier corpora repeat-validation without re-parse
may matter. Follow-up bead filed (below) rather than widening this bundle now.

## Follow-ups filed

- **sq-c6c2s** — minified browser-bundle byte comparison (esbuild-built
  rdf-validate-shacl bundle vs the wasm artifact) + evaluate further size-trim
  for `sparq-shacl-wasm`.
- **sq-01xlp** — evaluate a pre-parsed/stateful `Validator` variant (or bless
  the lean bundle's `Store.validate` as the stateful path) for scale-tier
  repeat validation.
- Scale-tier + quiet-box canonical gather rides the existing wasm-compare wave
  (sq-hmd7l.39 / sq-hmd7l.40) — the harness here accepts any data file via
  its `--data` seam.
