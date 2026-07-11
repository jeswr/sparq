<!-- [FABLE-5] sq-hmd7l.16 — per-axis gap record: RDFC-1.0 canonicalization.
First-read only; work-box readings are NON-canonical by construction and are never
transcribed here. -->

# Gap record — RDFC-1.0 canonicalization (2026-07)

**Axis:** RDF Dataset Canonicalization (epic `sq-hmd7l` comparative-benchmarking).
**Status:** harness DELIVERED + smoke green; full three-engine panel exercised on the
work box (parity green, poison outcomes recorded); no canonical quiet-box run yet (§4).
**Harness:** `bench/canon/` (`run.sh` + `crates/sparq-canon/examples/canon_bench.rs` +
`scripts/bench-adapters/canon_adapter.{mjs,sh}`), registered as `canon-bench`.

## 1. Engines and honest scope

| Engine | What runs | Comparability |
|---|---|---|
| sparq | `sparq-canon` public API (`canonicalize_quads` / `_with::<Sha384>`) via the `canon_bench` example — bridge included | subject |
| rdf-canonize (JS, digitalbazaar) | `NQuads.parse` + `canonize({algorithm:'RDFC-1.0'})` | the **independent reference implementation** — the real cross-implementation check |
| rdf-canon (Rust, zkp-ld/yamdan) | `canonicalize_quads` driven directly via a gather-time scratch CLI | **NOT independent at the default pin**: sparq-canon delegates its algorithm to this same crate, so this column isolates sparq's oxrdf-0.3↔0.2 bridge + guard configuration overhead; a different `CANON_RDF_CANON_VERSION` turns it into an upstream-drift check |

Scope invariants (enforced by `run.sh`, non-negotiable):

- **Bytes before stopwatch:** every engine must produce byte-identical canonical
  N-Quads on the 61 non-pathological W3C eval fixtures (vendored suite = oracle)
  before any timing; a mismatch reds the panel and skips that engine's timing.
- **HARD per-graph wall-clock cap** (`timeout`, default 10 s) on the poison panel;
  a cap-hit is recorded as `capped` — an honest DoS-resistance *result*. The harness
  separately asserts the cap actually bounded the run (cap violation = harness bug).
- Outcome vocabulary: `ok` / `guard` (fail-closed refusal) / `capped` / `wrong` /
  `accepted` — see `bench/canon/README.md`.

## 2. First-read findings (work box — NON-canonical, no timings transcribed)

1. **Three-way byte-parity is green on the full sane set** (61/61 per engine,
   SHA-256 + the SHA-384-parameterized case). The axis starts from byte-identical
   canonical output, not from timings.
2. **Default-posture conformance differs sharply, and it is a DoS-guard trade-off,
   not an algorithm bug.** rdf-canonize's shipped default guard (`maxWorkFactor=1`)
   fail-closes on 18 of the 64 approved eval cases — including small, fully
   legitimate symmetric graphs (e.g. the 2-blank-node cycle `test022`) and the three
   `poison – evil` evals the spec marks "computable given defined limits". At its
   documented raised posture (`maxWorkFactor=3`) it is byte-exact on all of them.
   sparq (and the rdf-canon crate it delegates to) pass **all 64 approved evals as
   shipped** (default HNDQ call limit) while still refusing the negative 10-node
   clique fail-closed. The panel therefore runs the JS column's parity/timing at its
   raised posture and records poison rows under BOTH postures, labelled.
3. **Nobody blew the cap** on the vendored poisons or generated cliques (n ≤ 20):
   all engines either computed the computable poisons or refused fail-closed within
   the cap. The DoS story at these sizes is guard-quality, not wall-clock blow-up;
   larger `CANON_CLIQUE_SIZES` sweeps stay available for a canonical run.
4. **No `wrong` or `accepted` outcomes anywhere** — no engine returned non-matching
   bytes under the cap, and none accepted the must-fail case at default limits.

## 3. Canonical protocol (when the gather run happens)

Dedicated quiet EC2 box (tag `sparq-bench`, orphan-proof self-terminate), engines
pinned and recorded in the envelope (sparq commit; rdf-canonize npm version; rdf-canon
crate version):

```sh
cargo build --release -p sparq-canon --example canon_bench
npm install rdf-canonize --prefix /tmp/canon-js
CANON_NODE_MODULES=/tmp/canon-js CANON_RDF_CANON_BIN=auto \
CANON_JSON_OUT=bench/competitor-results/canon-$(date -u +%Y%m%dT%H%M%SZ).json \
bash bench/canon/run.sh
```

Results land git-ignored under `bench/competitor-results/`; nothing is committed
(repo hygiene: no hard-coded perf numbers). The envelope separates process-inclusive
`wall_us` from engine-reported in-process `canon_us` (node startup dominates the JS
column's wall).

## 4. Deferred / follow-ups

- Canonical quiet-box gather run feeding the perf-dominance master table, including a
  larger clique/degenerate-structure sweep to find each guard's actual cap frontier,
  and resolving the registry `unverified_pin` on `rdf-canonize-js` with the npm
  version the run pins.
- Issued-identifier-map parity (`RDFC10MapTest`) across engines — sparq's map surface
  is W3C-gated by the crate suite; the peers' map APIs are not driven by this panel.
