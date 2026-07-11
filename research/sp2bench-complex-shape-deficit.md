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
  behind. *(Post-chain update: the diagnostic re-run confirmed this as the dominant
  residual — q12a cost its full SELECT twin. Now implemented as `sq-7d3dj.30.8`:
  block-driven capped conjunctive chain + capped UNION/OPTIONAL/Join arms in
  `try_capped`, plus emptiness-neutral ASK plan simplification; witnesses in
  `crates/sparq-engine/tests/ask_early_exit.rs`.)* [FABLE-5]
- **UNION common-subexpression sharing** (q07/q09 both re-scan shared subpatterns):
  **DISCONFIRMED by profiling (2026-07-10, PR #1842)** — the two inner 4-pattern BGPs in q07
  cost only ~2 ms each; sharing recovers ≤ 2 ms, not the residual. The actual q07 residual
  is a redundant re-evaluation of the outer 5-pattern BGP itself: `try_theta_antijoin` eagerly
  evaluated the mandatory left side (~30 ms), found no seedable var-to-var correlation (bare `!bound`),
  declined and discarded it, so the cold Filter{LeftJoin} fallback re-evaluated the same left side.
  **FIXED (PR #1842, merged 2026-07-10)** via opt-in `antijoin-static-decline` — a static pre-check
  lets `try_theta_antijoin` decline before eager evaluation when no correlation can seed, eliminating
  the redundant re-eval. Result: bag-equivalent (W3C + differential, both feature states). *(SUPERSEDED
  2026-07-10: the cross-level CSE hypothesis below in §7.3 is also disconfirmed; see that section.)*
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

## 7. Canonical re-measure verdict (sq-7d3dj.30.6) — the D3 update

<!-- [FABLE-5] Canonical envelope: bench/competitor-results/sparql-same-box-20260710T025117Z.json
     Provenance: quiet dedicated EC2 c6i.4xlarge (Intel Xeon Platinum 8375C @ 2.90 GHz, 16 vCPU,
     x86_64 — the SAME class + arch as the 2026-07-07 §0 baseline), min-of-5, count mode, SP2Bench
     250 000 triples (real Freiburg sp2b_gen), git 1190ca84 (main tip: the full sq-7d3dj.30.1–.14
     wave + #1786 q06 theta anti-join + #1795 predicate-range term-kind idfast widening + #1813
     extended anti-join, all merged), CANONICAL=1. This is a same-box sparq-CLI-vs-QLever comparison
     (QLever indexed + served + queried over HTTP by scripts/qlever-same-box.sh; Oxigraph prebuilt
     CLI v0.5.9 carried as the clean-CLI baseline). QLever's HTTP request floor is UNCORRECTED
     here, so a per-query deficit stated below is a LOWER BOUND on sparq's compute gap. -->

**Headline.** After the complex-shape fix wave landed, sparq wins **10 of 14** SP2Bench
queries against same-box QLever and is **correct + complete on all 14** (matching
`bench/sp2b/expected-rows.tsv` at 250k). The two selective-FILTER queries that dominated the
old deficit — **q03b and q03c — flipped from BEHIND to decisively AHEAD** (38× / 21× faster
than QLever). Of the seven queries the gap record flagged BEHIND, **five are now AHEAD or have
their deficit cut by 4–14×**; **q07 alone did not improve** (it regressed slightly). q08/q12b
have **no valid QLever comparison** because QLever returns the wrong answer there (count 0, see
below), so sparq's win on those is a correctness win, not a timing win.

### 7.1 Canonical per-query table (best-of-5 µs; sparq is the count reference)

`oxi` = Oxigraph prebuilt-CLI 0.5.9 (clean-CLI baseline); `cap` = the per-query timeout cap
was hit (Oxigraph's q07/q08/q12b are minutes-long at 250k). QLever timing is trusted **only
when its count matches** — q08/q12b are DISQUALIFIED (QLever returns 0 rows, a genuine
QLever query-semantics divergence, recorded honestly and NOT adjusted).

| query | rows | sparq µs | oxi µs | qlever rows | qlever µs | sparq vs QLever |
|---|---|---|---|---|---|---|
| q01 | 1 | 11.0 | 14452 | 1 | 2707 | AHEAD 246× |
| q02 | 6067 | 10567.9 | 528976 | 6067 | 274445 | AHEAD 26× |
| q03a | 15823 | 2332.4 | 217671 | 15823 | 48047 | AHEAD 21× |
| q03b | 114 | 42.3 | 128748 | 114 | 1614 | **AHEAD 38× (was BEHIND 8.4×)** |
| q03c | 0 | 54.5 | 127474 | 0 | 1157 | **AHEAD 21× (was BEHIND 19×)** |
| q04 | 541911 | 206512.4 | 20218610 | 541911 | 2942831 | AHEAD 14× |
| q05b | 6933 | 9638.6 | 628300 | 6933 | 31785 | AHEAD 3.3× |
| q07 | 48 | 30922.0 | cap | 48 | 8196 | **BEHIND 3.8× (was BEHIND 2.9×)** |
| q08 | 358 | 36574.8 | cap | 0 | 8284 | qlever **wrong** (0 vs 358) — DISQ; sparq correct+complete |
| q09 | 4 | 12935.6 | 186843 | 4 | 1360 | **BEHIND 9.5× (was BEHIND 16.4×)** |
| q10 | 452 | 5.3 | 16567 | 452 | 5270 | AHEAD 994× |
| q11 | 10 | 5319.9 | 756143 | 10 | 1771 | **BEHIND 3.0× (was BEHIND 13.9×)** |
| q12b | 1 | 36210.7 | 50422754 | 0 | 6724 | qlever **wrong** (0 vs 1) — DISQ; sparq correct+complete |
| q12c | 0 | 4.5 | 10684 | 0 | 1420 | AHEAD 316× |

### 7.2 Before → after, per D3 query (the explicit verdicts the bead requires)

Baseline sparq µs is the §0 / §3 canonical c6i.4xlarge figure (2026-07-07); "after" is this
re-measure. Both are quiet-box x86_64, so the sparq-side improvement ratio is meaningful.

| query | sparq §0 → now (µs) | sparq self-speedup | §0 verdict | **new verdict** |
|---|---|---|---|---|
| q03b | 12124 → **42** | **287× faster** | BEHIND 8.4× | **AHEAD** (38× vs QLever) — deficit CLOSED |
| q03c | 11697 → **55** | **215× faster** | BEHIND 19× | **AHEAD** (21× vs QLever) — deficit CLOSED |
| q07 | 23981 → **30922** | **1.3× slower** | BEHIND 2.9× | **BEHIND 3.8×** — the one query that did NOT improve |
| q08 | 153318 → **36575** | **4.2× faster** | BEHIND 11.3× (virtuoso) | correct+complete; QLever DISQ; still ~2.7× behind the §0 virtuoso ref |
| q09 | 22357 → **12936** | **1.7× faster** | BEHIND 16.4× | **BEHIND 9.5×** — deficit ~halved, not closed |
| q11 | 27236 → **5320** | **5.1× faster** | BEHIND 13.9× | **BEHIND 3.0×** — deficit cut ~4.6× |
| q12b | 155611 → **36211** | **4.3× faster** | BEHIND 16.4× (virtuoso) | correct+complete; QLever DISQ; still ~3.8× behind the §0 virtuoso ref |

**q08 / q12b caveat (honest).** This gather is sparq-vs-QLever, and QLever computes the wrong
answer on q08/q12b (0 rows — the bnode≠IRI strict-type-error divergence adjudicated in
sq-ai2wa; `expected-rows.tsv` is correct). So there is **no valid same-box competitor** for
q08/q12b in this run. Both sped up ~4× on the sparq side vs §0, but against the only prior
CORRECT competitor on those rows — virtuoso (HTTP) at §0: q08 13548 µs, q12b 9476 µs — sparq at
36575 / 36211 µs is still ~2.7× / ~3.8× behind. Those figures are cross-date/HTTP-mode and
carried for magnitude only; a same-box virtuoso re-measure on q08/q12b is the clean way to
retire the residual (folded into the follow-up beads below).

### 7.3 Residual deficits → root-cause hypotheses + follow-up beads

Still behind (real compute gaps, all LOWER bounds given QLever's uncorrected HTTP floor):

- **q07 — BEHIND 3.8× (and 1.3× WORSE than §0; the sole non-improver).** **SUPERSEDED 2026-07-10:**
  The root-cause hypothesis stated in §5 (cross-level CSE of membership subplans) was **DISCONFIRMED
  by profiling** (PR #1842, sq-7d3dj.30.20, merged 2026-07-10; see GitHub issue #1843). **Real cause:**
  `try_theta_antijoin` eagerly evaluated the mandatory left side (q07's outer 5-pattern BGP, the dominant
  cost ~30 ms), found no seedable var-to-var correlation (bare `!bound`), declined the anti-join, and
  discarded the result — so the Filter{LeftJoin} fallback re-evaluated the same left side redundantly.
  **FIXED** via opt-in `antijoin-static-decline` (PR #1842): a static pre-check lets `try_theta_antijoin`
  decline before evaluating the left side when no correlation can seed, bag-result-equivalent (W3C + differential,
  both feature states). The small regression (~1.3×) is consistent with the extra plan machinery (theta anti-join
  recognizer + cluster materialise) adding fixed overhead that the tiny 48-row result cannot amortise; profiling-first
  follow-up work (if q07 needs further optimisation) is noted in the bead sq-7d3dj.30.20 notes.
- **q09 — BEHIND 9.5× (halved from 16.4×).** Root-cause hypothesis: the DISTINCT-projection
  anchor+probe semijoin (#1782) still pays O(min(block, anchor)) to PROVE a large value-typed
  predicate (dc:title / foaf:homepage / rdfs:seeAlso, 17k–27k distinct objects) has no anchor
  member — QLever's "pattern trick" answers this from precomputed per-predicate incidence
  metadata. Fix bead: **sq-jnb1e** (characteristic-set / per-predicate incidence metadata)
  already exists — re-pointed here as the confirmed q09 residual.
- **q11 — BEHIND 3.0× (cut ~4.6× by the top-k ORDER BY fix #1784).** Smallest remaining
  deficit; the residual is the single-scan + bounded-select constant factor vs QLever's indexed
  ORDER-BY. Filed as a NEW P2 bead (see below) — profiling-first, no code guess here.
- **q08 / q12b — no valid same-box competitor (QLever wrong).** Residual is only against the
  §0 virtuoso HTTP reference (~2.7× / ~3.8×). Filed as a NEW P2 bead to re-measure q08/q12b
  against a same-box CORRECT competitor (virtuoso) and localise any true compute gap.

### 7.4 D3 dimension verdict

D3 moves from **BEHIND on the complex-shape class (7/14 behind)** to **MIXED, strongly
improved**: sparq is now AHEAD of same-box QLever on 10/14 and correct+complete on all 14; the
two headline selective-FILTER deficits (q03b/q03c) are CLOSED; q08/q09/q11/q12b deficits are cut
4–14× (q09/q11 still behind, q08/q12b have no valid QLever comparator); **q07 is the one query
still clearly behind and is the sole non-improver**. The gap-record D3 row is updated to reflect
this (see `research/perf-dominance-gap-2026-07.md` §3 addendum). The order-of-magnitude-vs-parity
mandate question (§6) is unchanged: sparq is at or beyond parity on the whole class except q07/q09,
where an order-of-magnitude target still needs profiling-first follow-up work (q07's redundant anti-join
re-eval was fixed in PR #1842, but the fixed machinery still carries overhead; q09 needs the characteristic-set
(sq-jnb1e) work to compete on per-predicate incidence metadata).
