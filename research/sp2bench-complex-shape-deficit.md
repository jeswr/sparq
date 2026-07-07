<!-- [FABLE] Authored by Claude Fable 5 (Fable-tier architect stage). Bead sq-7d3dj.30.
Profiling-first root-cause analysis + fix decomposition for the SP2Bench complex-shape
query-latency deficit (gap-table dimension D3, research/perf-dominance-gap-2026-07.md).
All work-box numbers below are NON-CANONICAL (shared EC2 work box) and are used only for
RELATIVE attribution; the canonical numbers are quoted from the 2026-07-07 c6i.4xlarge
envelope with provenance. -->

# SP2Bench complex-shape deficit — root cause + fix decomposition

**Status:** design record / decomposition (architect stage; no implementation here).
**Date:** 2026-07-07. **Bead:** sq-7d3dj.30 (parent epic `sq-7d3dj`; child fix beads
sq-7d3dj.30.1–.6 created by this record). **Feeds:** the standing performance-dominance
mandate; gap-table dimension **D3** (`research/perf-dominance-gap-2026-07.md`, PR #1727).

## 0. Problem (canonical, from the 2026-07-07 matrix)

On the canonical 5-engine SP2Bench-250k matrix (quiet c6i.4xlarge, min-of-5, git
`0ab87b2a`, 2026-07-07), sparq CLI wins all 14 queries vs the clean CLI baseline
(oxigraph, 8.3–4072×) but **loses 7 of 14 vs virtuoso/qlever (HTTP mode)**, and every
deficit **survives / widens** under the most aggressive HTTP-overhead correction, so
they are real compute losses:

| query | sparq µs | best correct competitor µs | deficit |
|---|---|---|---|
| q03b | 12 124 | virtuoso 1 443 | 8.4× |
| q03c | 11 697 | virtuoso 613 | 19× (competitor near the HTTP floor ⇒ pure-compute gap is far larger) |
| q07 | 23 981 | qlever 8 277 | 2.9× |
| q08 | 153 318 | virtuoso 13 548 | 11.3× (qlever disqualified: count 0, wrong) |
| q09 | 22 356 | qlever 1 362 | 16.4× |
| q11 | 27 236 | qlever 1 961 | 13.9× |
| q12b | 155 611 | virtuoso 9 476 | 16.4× (qlever disqualified: count 0, wrong) |

Honesty note carried over from the gap record: sparq is the only engine both correct and
complete on q07/q08/q12b (oxigraph errored q07/q08; qlever returned wrong empty results
on q08/q12b). The deficits above are only to **correct** competitors.

## 1. Method

Profiling-first, per the bead. Local reproduction on the (shared, **non-canonical**)
work box: `sparq-cli bench /tmp/sp2b/sp2b-250000.ttl turtle bench/sp2b/queries 5 count`
at git `78dd640b3` reproduced the canonical *relative* shape exactly (q03b/c flat vs
q03a; q12b ≈ q08; q11/q09 in the tens of ms). Root-cause attribution used:

- `sparq_engine::explain_analyze` (the T22 explain surface) via a throwaway `/tmp`
  harness — per-operator plan + row counts + wall time for each losing query;
- `perf record` (cpu-clock, call graphs) on a query-loop binary for the one query
  whose explain trace did not localise the cost (q11);
- code reading of `crates/sparq-engine/src/exec.rs` (GOO planner, filter split, join
  operators, `order_bindings`, `eval_ask`, `distinct_bindings`) and `dp.rs` (the
  opt-in DPccp planner).

All local timings below are **non-canonical** and quoted only as relative evidence.
Canonical re-measurement is its own child bead (sq-7d3dj.30.6).

## 2. Per-query root causes (measured, not hypothesised)

### 2.1 q03b / q03c — FILTER `?property = <iri>` is never folded into the pattern

**The gap record's hypothesis was wrong in an instructive way.** It guessed "sparq
scans where virtuoso uses an *object/range index* on selective FILTERs". The actual
query shape is a **predicate-variable equality filter**:

```sparql
?article rdf:type bench:Article .
?article ?property ?value
FILTER (?property = swrc:month)
```

`explain_analyze` (local): the plan seeds from the type scan (est 17 134 articles),
bind-joins each article into `?article ?property ?value` — enumerating **every**
property row of every article (est 250 117) — and only then applies a **post-join
Filter** on `?property`. Cost is therefore identical for q03a/q03b/q03c regardless of
selectivity (local: 16.8 / 16.0 / 15.9 ms), exactly matching the canonical flat cost.

Mechanism in code: the sargable-filter path (`extract_sargable`, exec.rs) only
recognises *numeric* and *temporal* literal comparisons; an equality against a
`NamedNode` is not sargable, falls into the residual, and there is **no rewrite** that
substitutes the constant into the triple pattern. Constants written directly in a
pattern *are* resolved to dictionary ids and become indexed lookups
(`prepare_pattern`), so the fix is a pure **algebra rewrite**, not a new index:
`FILTER(?v = <iri>)` ⇒ substitute `<iri>` for `?v` in the scope's triple patterns
(bind `?v` back if projected). After the rewrite q03b becomes a two-pattern BGP whose
GOO seed is the P-bound `?article swrc:month ?value` scan; q03c (`swrc:isbn`, 0
matches) short-circuits at `prepare_pattern` if the IRI is absent from the dictionary.

`=` on IRIs is term-identity, so the substitution is exact — see §4 for why the
rewrite is deliberately **restricted to IRI constants** (never numeric/plain literals),
which keeps it disjoint from the open sq-lr2ii sargable-decimal bug.

### 2.2 q08 / q12b — the Union subtree is planned blind to the 1-row `?erdoes` binding

`explain_analyze` (local): the outer Join has two children — the `?erdoes` BGP
(`foaf:name "Paul Erdoes"` seed, **1 row**, 17 µs) and the big Union — but the Union
branches are planned and evaluated **independently of that binding**, because
`?erdoes` is just an unbound variable inside them. The 5-pattern branch therefore
computes the full creator×creator×creator self-join over the whole corpus —
**238 766 rows** (local 248 ms) — before the hash join throws all but 991 away against
the 1-row side. Local totals: q08 315 ms, q12b 347 ms.

Root cause: **no sideways information passing (SIP) across graph-pattern operators**.
Binary joins *inside* a BGP use bind joins, but `Join(A, B)` at the graph-pattern
level evaluates `B` cold even when `A` has already produced 1 row. Virtuoso/qlever
evaluate the union correlated on the erdoes binding.

q12b (ASK of the same pattern) additionally shows that **ASK does not short-circuit
through joins**: `eval_ask` wraps the pattern in `Slice{length:1}`, but only
single-pattern scans have a capped path (`try_capped`); the Join/Union fully
materialises and q12b costs the same as q08 (canonical 155.6 vs 153.3 ms; local 347 vs
315 ms). The SIP fix collapses most of the q12b deficit too (virtuoso's own q12b ≈ its
q08, i.e. it does not meaningfully short-circuit either); a dedicated ASK early-exit
through joins is recorded as a possible follow-up, not a child bead (§5).

### 2.3 q09 — 77 k Union rows materialised to answer a 4-row DISTINCT ?predicate

`explain_analyze` (local): each Union branch seeds from the 20 602-row
`?person rdf:type foaf:Person` scan and bind-joins every person into
`?s ?p <person>` / `<person> ?p ?o`, materialising **76 924 rows** (26.9 ms) that
`distinct_bindings` (a post-hoc `HashSet` retain over full rows) reduces to **4**.

Two compounding gaps: (a) **no distinct-projection pushdown** — `Distinct{Project{?p}}`
still materialises full-width rows of the whole join; (b) the per-person probe ranges
(SPO for `<s> ?p ?o`, OSP/OPS for `?s ?p <o>`) are **sorted by predicate**, so a
loose/skip scan could jump to the next distinct predicate per range — and a global
already-seen predicate set makes later probes near-free (the same observation behind
qlever's "pattern trick", which is why qlever answers q09 in ~1.4 ms). The chosen fix
is the general operator (distinct-projection-aware evaluation with permutation-order
skip scanning), not a q09-special-case.

### 2.4 q11 — full stable sort with per-comparison term re-derivation for a top-60 slice

Shape: single 17 663-row scan, `ORDER BY ?ee LIMIT 10 OFFSET 50`. `explain_analyze`
(local): the BGP scan is **391 µs**; the total is **30.6 ms** — >98 % of the time is in
`OrderBy` + `Slice`. `perf` attributes it: 44 % in the full `driftsort`, with
`lit_kind` (14.7 %), `is_numeric_dt` (6.1 %), `value_compare_strict` (4.9 %) hot —
i.e. for non-numeric/non-temporal sort cells (`SortCell::Val(Term)`, the IRI case
here) the comparator re-derives term kind/class on **every comparison**, and the
engine has **no top-k path**: `Slice{Project{OrderBy}}` fully sorts all rows
(`order_bindings`, exec.rs) and then slices 10.

Fix: bounded-selection (size `offset+limit` heap / partial select, with the row's
input index as tie-breaker so output stays byte-identical to the current stable sort)
threaded from the Slice arm through order-preserving unary ops to `order_bindings`,
plus a precomputed cheap collation key for `Val` cells so comparisons stop calling
`lit_kind` per pair.

### 2.5 q07 — greedy GOO locks a bad join order (and !bound negation materialises)

`explain_analyze` (local, total 35.6 ms): the outer 5-pattern BGP is **21.8 ms** of
it. GOO seeds from the est-9 `subClassOf` scan, fans through `?doc rdf:type ?class`
(est 47 966) and the est-250 117 `?bag2 ?member2 ?doc` probe, and only joins the
**est-139** `?doc2 dcterms:references ?bag2` pattern **last** — greedy never
reconsiders, though seeding near the est-139 pattern gives a far smaller intermediate
profile. This is precisely the shape the existing **opt-in DPccp planner** (`dp.rs`,
`dp-planner` feature, Cout-optimal bushy trees, result-equivalent by construction) was
built for; it is currently dark in the CLI (feature off, planner never installed).

Secondary: the `OPTIONAL { … } FILTER(!bound(?v))` negation idiom materialises the
full left join (10 380 rows) and filters afterwards; there is no rewrite to an
anti-join even though a `GraphPattern::Minus` executor exists. The rewrite (guarded by
the standard well-designedness conditions) helps q07's outer level and composes with
the q03 rewrite in the same new pass.

q07 is the smallest deficit (2.9×) and its cost is spread; the two fixes above are
both general-purpose planner improvements rather than a q07 special case.

## 3. Fix decomposition (what, where, why disjoint)

Five implementation beads + one canonical re-measure bead. **Disjointness rule:** no
two parallel beads touch the same file; the three beads that must touch
`crates/sparq-engine/src/exec.rs` are **sequenced** with `bd dep` edges (smallest
first) instead of being pretend-parallel.

| bead | fix | crate / files | tier | depends on |
|---|---|---|---|---|
| sq-7d3dj.30.1 | Algebra rewrite pass: (a) `FILTER(?v = <iri>)` constant substitution into patterns; (b) `OPTIONAL`+`FILTER(!bound)` → anti-join (Minus) | sparq-engine: **new** `src/rewrite.rs` + hook in `src/lib.rs` + new test file | opus | — (parallel-safe) |
| sq-7d3dj.30.2 | Top-k ORDER BY: bounded selection for `Slice{…OrderBy}` + cheap precomputed collation key | sparq-engine: `src/exec.rs` + new test file | sonnet | — |
| sq-7d3dj.30.3 | SIP: bind-join at graph-pattern level — evaluate Join's big child (incl. Union branches) correlated on a small already-evaluated child | sparq-engine: `src/exec.rs` + new test file | opus | .30.2 (file) |
| sq-7d3dj.30.4 | Distinct-projection pushdown + loose (skip) index scan for `Distinct{Project}` over BGP/Union | sparq-engine: `src/exec.rs` + new test file | opus | .30.3 (file) |
| sq-7d3dj.30.5 | DPccp planner on by default for small BGPs when the `dp-planner` feature is compiled; enable the feature in sparq-cli | sparq-engine `src/dp.rs` + `crates/sparq-cli/Cargo.toml` + new test file | sonnet | — (parallel-safe) |
| sq-7d3dj.30.6 | Canonical re-measure: quiet-box 5-engine SP2Bench re-run after .1–.5 land; update the D3 verdict honestly | no crate code (EC2 gather protocol + envelope) | sonnet | .30.1–.30.5 |

Expected per-query coverage (directional, from §2; verified only by .30.6):
q03b/c ← .30.1; q08/q12b ← .30.3 (+.30.1's anti-join unused, +.30.5 harmless);
q09 ← .30.4; q11 ← .30.2; q07 ← .30.5 + .30.1(b).

Every bead carries `{crate, model_tier, invariant, acceptance_test}` in its `bd`
description so the implementing fleet can pick it up cold; the load-bearing invariant
on all five implementation beads is **result-equivalence** (bag semantics) proven by
the W3C conformance suite + a targeted new test per bead + the differential fuzzer.

## 4. Interaction with sq-lr2ii (open sargable-FILTER decimal bug) — stated explicitly

sq-lr2ii: the existing **numeric** sargable FILTER path collapses high-precision
`xsd:decimal` to f64 and returns wrong `=`/`<`/`>`/`<=`/`>=` rows. Two of this
record's beads touch adjacent territory and are designed to **avoid, not widen** it:

- **sq-7d3dj.30.1** restricts the equality-substitution rewrite to **IRI constants
  only** (term-identity, no value space, no numeric promotion, no f64 anywhere).
  Literal constants — where SPARQL `=` is *value* equality (`"1"^^xsd:int =
  "01"^^xsd:integer`, and the lr2ii decimal-precision class) — are **explicitly out of
  scope** and keep flowing through the existing residual/expression path, which
  sq-lr2ii's own notes show is exact. The bead's acceptance test includes a negative
  case asserting a numeric-literal equality FILTER is NOT rewritten.
- **sq-7d3dj.30.2** reuses the ORDER BY `SortCell` machinery, which already carries
  the id for exact tie re-checks (sq-rikm7); the bead's invariant is byte-identical
  output to the current sort path, so it cannot introduce a new f64-collapse surface.

Neither bead *fixes* sq-lr2ii (that stays its own open P1 bug on the numeric path);
they are constructed so the fix there and the work here cannot conflict semantically.

## 5. Non-goals / deferred

- **ASK early-termination through joins** (streaming first-solution evaluation): real
  but subsumed for q12b by .30.3 at this corpus scale, and it would touch the same
  `exec.rs` eval paths as three other beads. Revisit after .30.6 if q12b is still
  behind.
- **UNION common-subexpression sharing** (q07/q09 both re-scan shared subpatterns):
  measured secondary to the fixes above; fold into a later bead only if .30.6 shows a
  residual gap.
- **A new object/range index**: the gap record's original hypothesis for q03b/c —
  disconfirmed by profiling (§2.1); no new index is needed for this query class.

## 6. Honesty / limits

- All timings in §2 are **work-box, non-canonical**, used for relative attribution
  only. The canonical deficits are the §0 table (c6i.4xlarge envelope, 2026-07-07).
  No new canonical claim is made here; .30.6 is the only place verdicts get updated.
- Expected wins are **directional brackets**, not promises: e.g. q03b's post-rewrite
  cost should approach its selectivity (114 rows), but the actual number comes from
  the canonical re-run.
- The D3 mandate question from the gap record §6 (is parity with the HTTP engines
  acceptable, or is order-of-magnitude required on this class too?) remains open with
  the maintainer; this decomposition targets the deficit either way and does not need
  the answer to proceed (proceed-and-document).
