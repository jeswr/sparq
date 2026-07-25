<!-- [FABLE-5] sq-pymgf — per-axis gap record: the SHACL SPARQL-constraint (sh:sparql)
SLICE, split out of the whole-suite SHACL mix as its own reported axis. Canonical
numbers land here only from a quiet-box gather with provenance (sq-7d3dj.33.3);
work-box readings are NON-canonical by construction and are never transcribed here. -->

# Gap record — SHACL SPARQL-constraint slice (`sh:sparql`) (2026-07)

**Axis:** sub-axis of the SHACL validation axis (epic `sq-hmd7l`; whole-suite record:
`research/shacl-baseline-2026-07.md`, perf gap D8). This record covers ONLY the
`sh:sparql` (W3C SHACL §5.2) slice — the constraint kind routed through
`sparq-engine`'s native SPARQL evaluation, where engine-native constraint execution
should be a differentiator, and where the pre-batching validator was honestly BEHIND
Jena (baseline §2–§3).
**Status:** harness + pinned oracle SHIPPED (this bead, `sq-pymgf`); canonical
comparative numbers NOT-MEASURED (ride the quiet-box gather `sq-7d3dj.33.3`).
**Harness:** `crates/sparq-shacl/examples/sparql_constraint_bench.rs` (single-crate,
self-asserting) + the pinned oracle `bench/shacl/expected-sparql.tsv`.
**Feeds:** the master table in `research/perf-dominance-gap-2026-07.md` (via
`sq-hmd7l.27`'s successor consolidations).

## 1. Why a separate slice

`scripts/bench/shacl-same-box.sh` runs the whole `(data × shapes)` mix and reports
`sparql_heavy` as ONE workload row; the per-commit suite (`bench/shacl/run.sh`)
gates one single-constraint `sparql_constraint` workload. Neither reports the
`sh:sparql` path per shape, so the slice's engine-native story (focus-node batching
into one `VALUES` execution, `sq-7d3dj.33.1`; engine bind-join follow-up
`sq-7d3dj.33.2`) had no axis of its own. This harness validates ONLY
`bench/shacl/shapes-sparql/sparql_heavy.ttl` — three `sh:sparql` constraints over
the three biggest LUBM target classes — and reports each shape separately plus the
whole-slice `TOTAL`.

## 2. Protocol (oracle before stopwatch — INVARIANT)

1. Deterministic corpus: the LUBM(1) seed-0 ABox (`bench/shacl/gen.sh 1 0`,
   cached), the same substrate as `bench/shacl/expected.tsv`.
2. **Hard gate before any timing:** the runner asserts, per shape AND for the
   total, `conforms` / `violations` / `focus_nodes` against the pinned
   `bench/shacl/expected-sparql.tsv` (whose header carries the independent
   raw-ABox cross-check for every count), and exits 1 on any drift WITHOUT
   emitting a timing row. A drift is a recorded correctness finding, never
   rounded. Two internal consistency checks back the per-shape split: each
   single-shape count must equal the `sh:sourceShape`-grouped count from the
   full-set validation, and the per-shape counts must sum to the total.
3. Timing is ADVISORY and per shape (best-of-N single-shape validate on a loaded
   graph) plus the `TOTAL` row (all three shapes, one model — the number
   comparable to the same-box script's `sparql_heavy` row):

   ```sh
   cargo run -p sparq-shacl --release --example sparql_constraint_bench -- --smoke  # gate, iters=1
   cargo run -p sparq-shacl --release --example sparql_constraint_bench            # best-of-3
   ```

## 3. External competitor column — documented gather step

The harness itself stays single-crate and self-asserting; the pySHACL column is a
GATHER step (never a per-commit dependency), reusing the registered adapters:

```sh
# pySHACL (Tier-1 W3C reference impl), validate-only best-of-N, same corpus:
python3 -m venv /tmp/shacl-bench-venv && /tmp/shacl-bench-venv/bin/pip install pyshacl
mkdir -p /tmp/shapes-sparql-only && cp bench/shacl/shapes-sparql/sparql_heavy.ttl /tmp/shapes-sparql-only/
/tmp/shacl-bench-venv/bin/python scripts/bench/pyshacl-shacl-bench.py \
  "$(bench/shacl/gen.sh 1 0 | head -1)" nt /tmp/shapes-sparql-only 3
```

- pySHACL emits the whole-file row only (= this harness's `TOTAL`); a per-shape
  pySHACL split would group its report by `sh:sourceShape` — a gather-side
  refinement, not required for the slice comparison.
- Count reduction/cross-check uses `scripts/bench-adapters/shacl_report_count.py`
  semantics; the per-engine dedup caveat from `bench/shacl/expected.tsv` applies
  (these three shapes are single-route, so counts are expected to agree — the
  baseline's `sparql_heavy` crosscheck agreed across sparq/Jena/pySHACL at both
  scales).
- Jena SHACL rides the existing `scripts/bench/shacl-same-box.sh` driver
  (`JenaShaclBench.java`) unchanged.

## 4. Pinned slice oracle (deterministic counts, LUBM(1) seed-0)

| Workload (shape) | Target class | focus_nodes | violations |
|---|---|---:|---:|
| `GradCourseShape` | `ub:GraduateStudent` | 1 874 | 3 738 |
| `ProfessorLoadShape` | `ub:FullProfessor` | 125 | 183 |
| `UndergradAdvisorShape` | `ub:UndergraduateStudent` | 5 916 | 422 |
| `TOTAL` | (all three, disjoint) | 7 915 | 4 343 |

Each is derived from the runner AND independently cross-checked against the raw
ABox (typed-edge solution counts, no SHACL engine — see the `expected-sparql.tsv`
header). `TOTAL` = 4 343 equals the count all three engines agreed on for
`sparql_heavy` in the baseline crosscheck (`research/shacl-baseline-2026-07.md` §2).

## 5. Results — comparative timing

> NOT-MEASURED (canonical). The pre-batching whole-mix reading
> (`research/shacl-baseline-2026-07.md` §2: sparq BEHIND Jena on `sparql_heavy`,
> growing with scale) predates the landed `sh:sparql` focus-node batching
> (`sq-7d3dj.33.1`, PR #1737) and the SHACL-1.2 node-expression batching
> (`sq-7d3dj.33.5`, PR #1754), so it must not be cited for the current validator.
> The canonical per-shape slice numbers (sparq vs pySHACL, plus Jena from the
> same-box harness) land in the `sq-7d3dj.33.3` quiet-box gather, which should run
> this example alongside `scripts/bench/shacl-same-box.sh`.

| Workload | count crosscheck | sparq-shacl | pySHACL | Jena SHACL | Verdict |
|---|---|---|---|---|---|
| `GradCourseShape` | pinned (oracle §4) | NOT-MEASURED | NOT-MEASURED | NOT-MEASURED | NOT-MEASURED |
| `ProfessorLoadShape` | pinned (oracle §4) | NOT-MEASURED | NOT-MEASURED | NOT-MEASURED | NOT-MEASURED |
| `UndergradAdvisorShape` | pinned (oracle §4) | NOT-MEASURED | NOT-MEASURED | NOT-MEASURED | NOT-MEASURED |
| `TOTAL` | pinned + 3-engine agreement (baseline §2) | NOT-MEASURED | NOT-MEASURED | NOT-MEASURED | NOT-MEASURED |

## 6. Verdict + plan

No comparative verdict is claimable until the canonical gather runs (verdict
vocabulary: CLEARLY-AHEAD / AHEAD-BUT-NOT-OOM / PARITY / BEHIND / NOT-MEASURED /
NOT-COMPARABLE). Per the dominance mandate, any BEHIND or PARITY row measured
there must carry a root cause and an immediate P1 profiling-first fix bead — the
known engine-level lever is the bind-join/SIP follow-up `sq-7d3dj.33.2`. Action:
`sq-7d3dj.33.3`'s quiet-box run gathers this slice in the same wave (noted on that
bead).
