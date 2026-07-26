<!-- [SONNET-4.6] sq-hmd7l.9 — OWL 2 QL rewriting competitor gap record.
Harness description + first-read methodology for the sparq-reason-ql (PerfectRef) vs Ontop
comparison on the NPD + Requiem suites. No fabricated numbers; NO hard-coded performance
numbers presented as canonical. Every timing row this harness collects is flagged
NON-canonical (canonical:false) until a dedicated quiet EC2 run re-measures it. -->

# OWL 2 QL rewriting competitor gap — 2026-07

**Status:** harness record / first-read methodology description (NON-canonical).
**Date:** 2026-07-18.
**Bead:** sq-hmd7l.9.
**Epic:** sq-hmd7l (comparative-benchmarking-everything).
**Competitor:** `ontop` — the mainstream OBDA / OWL 2 QL system (Apache-2.0).
**Canonical run:** deferred to a dedicated quiet-box wave (`quiet_box_sensitive = true`).

---

## 0. Prior state

`sparq-reason-ql` had two hermetic examples and no external comparison harness:
`ql_rewrite_bench` (closed-form UCQ sizes on generated chains + the sq-pbz04.3.5 corpus
profile — untouched by this bead) and `ql_endtoend_bench` (sq-mg1wx: NPD/Requiem-*shaped*
embedded fixtures, rewrite-then-execute; its Ontop column is an externally-supplied TSV,
see `research/gap-reason-ql-e2e-2026-07.md`). `bench/benchmarks.toml` carried the
`reason-ql-npd` registry stub (from sq-hmd7l.1) with every field `TBD`. This bead delivers
the durable NPD + Requiem harness (`scripts/bench/reason-ql-same-box.sh` + the
`ql_npd_requiem_bench` example) and fills the stub in.

---

## 1. Metrics — TWO, both required

1. **Rewrite wall time** — the rewriter phase only, per query.
2. **Output UCQ size** — the disjunct count of the emitted union of conjunctive queries,
   per query. Deterministic and load-robust (a pure function of TBox + query), so it is
   the citable axis even on a noisy box.

The honest win condition is **smaller-or-equal UCQ at lower latency**; a smaller UCQ alone
is a WIN on the size axis even at equal time. sparq reports both the raw PerfectRef size
and the **minimised** size (`rewrite_production` — the query actually emitted); the
minimised count is the size-axis datum.

## 2. Regime labelling (the column-comparability invariant)

Ontop couples rewriting to SQL translation, and its CLI exposes **no isolated rewriter
phase and no UCQ-size readout**. Every column is therefore labelled with its regime, in
the example banner and in the envelope:

| column | regime |
|---|---|
| sparq `raw_rewrite_ms` / `min_rewrite_ms` | **rewriter-phase only** (in-process `rewrite` / `rewrite_production`, no execution) |
| sparq `exec_ms` / `e2e_ms` | minimised-UCQ execution / rewrite+execute over the loaded data |
| ontop | **end-to-end only** (`ontop query` CLI over its OBDA stack) |

The ONLY cross-engine time comparison permitted is sparq `e2e_ms` vs Ontop end-to-end,
and only over the **same data** (the NPD instance materialised to RDF via
`ontop materialize`, loaded into sparq with `--abox`). Ontop's end-to-end column must
never be read against sparq's rewriter-phase column. Isolating Ontop's rewriter phase
needs a small Java driver against its internal API — a recorded follow-up, not faked here.

## 3. Equivalence-before-timing (the sq-hmd7l.9 invariant)

No timing row exists without a passed result-set agreement check:

- **In-engine (always):** `ql_npd_requiem_bench` executes the raw PerfectRef UCQ and the
  minimised production UCQ over the same data and requires result-**set** equality
  (minimisation must never change answers); a disagreement aborts the run. Without real
  data the check runs over a set of deterministic **per-disjunct witness databases**: the
  frozen canonical instance of every CQ disjunct of the original, raw, and minimised queries
  (variables/blank nodes frozen to fresh per-disjunct IRIs; IRI/literal constants kept),
  each disjunct's instance held in its **own isolated database**, with raw and minimised
  required to agree over each. Every disjunct — query-only predicates and multi-atom joins
  included — matches at least its own instance, so on disjunct D's own database the retained
  UCQ reproduces D's frozen head only via a homomorphism into D's instance, i.e. only when D
  was genuinely subsumed; dropping a non-subsumed disjunct fails agreement on that disjunct's
  database (the per-CQ canonical-database containment argument; regression-tested in the
  example). **Isolation is load-bearing**: unioning the instances into one graph would share
  the retained IRI/literal constants and let a retained disjunct bridge a fact from a dropped
  disjunct's instance with a fact from a third instance through a shared constant, reproducing
  the dropped head and masking the unsound drop (the constant-bridge regression pins this).
  Witness mode is FAIL-CLOSED on modifiers:
  a query whose original/raw/minimised tree carries a re-applied FILTER/VALUES (the
  B3/B4 pass-through) is reported per-row as `needs-abox` with NO timing row — the
  modifier could reject every frozen binding and make the agreement vacuous — so such
  queries are only timed under `--abox` real data (regression-tested in the example).
- **Cross-engine (same-data regime only):** per-query answer counts from both engines are
  recorded in the envelope (`cross_engine_counts_agree`); neither engine is ground truth —
  a disagreement is recorded and investigated, never adjusted.
- **Hermetic acceptance (`--smoke`):** embedded NPD/Requiem-shaped fixtures with
  hand-verified closed-form minimised-UCQ sizes and pinned certain-answer counts, each
  asserted before its timing row. `ONLY=sparq bash scripts/bench/reason-ql-same-box.sh
  --smoke` is the exit-0 acceptance path (no network, no JVM).

## 4. Suites

Gather-only (fetched to `/tmp`, NOT committed):

- **NPD** — the Norwegian Petroleum Directorate benchmark (`github.com/ontop/npd-benchmark`):
  the QL ontology (riot → N-Triples for sparq's TBox extraction) + the repo's SPARQL
  queries. Queries outside the fail-closed CQ gate (OPTIONAL/FILTER/aggregation/…) are
  reported per-row as `out-of-scope` — on the real NPD mix that count is itself an honest
  coverage datum, not an error. The Ontop end-to-end column needs a loaded NPD PostgreSQL
  instance (`NPD_JDBC_PROPERTIES`); absent, it records an honest ERROR.
- **Requiem** — the Requiem test-suite ontologies + queries (the standard corpus of the
  rewriting literature: A/S/U/V/P5/UX by default). The datalog-style queries are
  translated to `SELECT DISTINCT` SPARQL by the harness (predicate local names resolved
  against the ontology's IRIs; an unresolvable predicate SKIPS the query with a log line,
  never a guess). Requiem ships **no data and no mappings**, so the Ontop OBDA CLI has
  nothing to answer over there: the Requiem Ontop column is an honest `not-applicable`
  pending the Java rewriter-phase driver; sparq's UCQ sizes + rewrite times stand alone.
  The default download URL is the Oxford ISG tools page and has NOT been verified from
  this box — set `REQUIEM_ZIP` to a local copy if it has rotted (the harness records the
  failure and skips, never fabricates).

## 5. First-read verdict

**PARTIAL — harness + hermetic acceptance built; no external gather executed.** This box
has no network access to the suites, no JVM/Ontop run, and is a shared work box, so NO
comparative numbers are claimed: not for wall time, not for UCQ size, not for either suite.
Everything this record asserts is structural (what the harness measures and how it labels
it). The deterministic evidence that exists today is the `--smoke` fixture: closed-form
minimised-UCQ sizes, raw-vs-minimised result-set agreement, pinned certain-answer counts.

**NON-canonical:** any timing a first full run collects on a shared box is directional
only (`canonical:false` in every envelope) and must not be baked into docs or dashboards;
the harness is the durable deliverable. A dedicated quiet-box run sets `CANONICAL=1`.

## 6. Follow-ups (captured as issues, not fixed here)

- Ontop rewriter-phase isolation via a small Java driver against its internal API, so the
  rewrite-time axis gains a true like-for-like competitor column.
- Canonical quiet-box gather (NPD PostgreSQL instance + Requiem zip pinned by sha256) in
  the sq-hmd7l canonical wave.
