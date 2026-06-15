<!-- [OPUS-4.8] Authored by Opus 4.8 (Fable unavailable; flag for re-review when Fable returns). -->
# Deep Taxonomy (DeepTaxonomy) — rule-heavy N3 forward-closure reasoning

The canonical rule-heavy inference micro-benchmark: a **single** instance fact, a **deep**
`rdfs`-style `:sc` (subclass) chain, and **one** transitivity meta-rule. It is the workload where
NAIVE forward chaining blows up (O(N²) — each new derived type re-joins the whole relation) while
a semi-naive, delta-indexed fixpoint is effectively linear. The reasoner under test is
`sparq-reason` (the closure under RDFS/OWL-RL/N3); here it runs the **N3** profile
(`sparq-cli reason … n3`). Registry entry: `deep-taxonomy` in
[`bench/benchmarks.toml`](../benchmarks.toml).

> **Attribution.** DeepTaxonomy is a community N3-reasoner benchmark used by the EYE reasoner and
> the RR-2023 literature. We **reuse** the existing in-repo generator
> [`bench/inference/gen_deeptaxonomy.py`](../inference/gen_deeptaxonomy.py) — this suite does NOT
> rewrite it; `gen.sh` is a thin, cache-backed wrapper around it (same generator the EYE
> head-to-head in [`bench/inference/eye-comparison.md`](../inference/eye-comparison.md) uses, so
> the closure sizes match).

## Layout

```
bench/deep-taxonomy/
├── gen.sh           thin wrapper REUSING bench/inference/gen_deeptaxonomy.py; emits a DT N3 corpus per depth
├── run.sh           self-asserting runner: materialize the N3 closure per tier, query it, assert expected.tsv
├── query.rq         class-membership probe: SELECT ?c WHERE { :i a ?c }  (returns depth+1 rows)
└── expected.tsv     DETERMINISTIC per-depth closure_triples (= 2·depth+1) + query_rows (= depth+1)
```

## The workload (per depth N)

`gen_deeptaxonomy.py N` emits:

```
:i a :n0 .                                 # one instance fact
{ ?x a ?c . ?c :sc ?d } => { ?x a ?d } .   # one transitivity meta-rule
:n0 :sc :n1 . :n1 :sc :n2 . … :n{N-1} :sc :nN .   # a depth-N :sc chain
```

After the N3 forward closure, `:i` is a member of **every** class `:n0 … :nN`. The closure is
therefore fully determined by the depth:

- **closure triple-count** `= 2·N + 1` — the `N` chain edges + the `N` derived `:i a :nK` + the
  seed `:i a :n0`. (dt1k → 2,001; dt10k → 20,001; dt100k → 200,001 — these match the closure
  cross-check table in [`bench/inference/eye-comparison.md`](../inference/eye-comparison.md), where
  sparq and EYE agree.)
- **`query.rq` rows** `= N + 1` — `:i` is now a member of `:n0 … :nN`.

## Self-asserting closure-size gate

`run.sh` is the point of the suite. For each depth tier it materializes the closure with
`sparq-cli reason <corpus> n3 n3 <out.nt>`, runs `query.rq` over the closure, and asserts BOTH the
materialized closure triple-count AND the query row-count against the deterministic values in
`expected.tsv`. Any divergence is a `sparq-reason` correctness regression (a missing/extra
entailment, a broken fixpoint) and **exits non-zero** — the metric latencies are trend-only, but
this structural gate is a hard correctness check (deterministic + load-robust, so it never flakes
on a noisy runner). The closure-size cross-check matches EYE's (cited above), so a regression that
keeps the timings green but changes the closure is still caught.

## Run it

```sh
cargo build --release -p sparq-cli
bench/deep-taxonomy/run.sh                       # default per-commit tiers: dt1k + dt10k
DEEPTAX_DEPTHS="1000 10000 100000" bench/deep-taxonomy/run.sh   # add dt100k (EC2/nightly)
```

`run.sh` prints one metric TSV line per tier-metric on stdout
(`<metric>\t<value>\t<unit>`) and the assertion diagnostics on stderr.

CI wires the per-commit subset through [`scripts/ci-bench.sh`](../../scripts/ci-bench.sh) (the
`deeptax` hook, mirroring the LUBM hook: it invokes the self-asserting `run.sh` and harvests its
TSV) as metrics:

- `deeptax_d<DEPTH>_closure_s` — N3 forward-closure wall seconds (engine-internal timer)
- `deeptax_d<DEPTH>_query_us` — `query.rq` min-of-iters latency (µs), count mode
- `deeptax_d<DEPTH>_closure_triples` — materialized closure triple-count (deterministic structural)

All three are **trend-only** (NOT in the deterministic hard perf-gate
`scripts/perf-gate.py`); the closure-size + query-row assertions inside `run.sh` are the hard
correctness gate. `run.sh` failing fails the whole `ci-bench` run.

## Tiering — per-commit vs EC2/nightly

Unlike `watdiv`/`bsbm`/`lubm`, this suite needs **no** heavyweight toolchain — only `python3`
(stdlib) plus the built `sparq-cli` — so it runs on the per-commit tier by default at a SMALL depth
pair (**dt1k + dt10k**; closures 2,001 / 20,001 triples — sub-second). The numeric `d<DEPTH>`
metric token lets the dashboard derive a **depth-scaling axis** (two points → a scaling curve).
**dt100k** (depth 100,000; closure 200,001) is opt-in via `DEEPTAX_DEPTHS` for the EC2/nightly
tier (its EYE reference was extrapolated, never run — see below).

## Dashboard + EYE competitor

The dashboard ([`bench/dashboard/dashboard.js`](../dashboard/dashboard.js)) features Deep Taxonomy
as a recognised public suite (`featuredSuiteOf` routes the `deeptax_…` metrics; `sizeAxisOf` reads
the `_d<DEPTH>` depth axis for the scaling chart), with readable labels generated into
[`bench/dashboard/metric-labels.json`](../dashboard/metric-labels.json) by
[`scripts/gen-metric-labels.py`](../../scripts/gen-metric-labels.py) (derived from this suite's
`expected.tsv` tiers — re-run the generator, or `--check` gates drift, if you add a tier).

EYE (the N3 reasoner reference) numbers are surfaced as **external-reference baselines** — real,
cited measurements at EYE's OWN scale/machine, shown SEPARATELY from sparq's same-scale metrics so
a mismatched figure is never aligned into a same-row comparison. They live in the dashboard's
competitor data ([`bench/dashboard/competitors.json`](../dashboard/competitors.json) + its
byte-mirror `COMPETITORS_DATA` in `dashboard.js`), cited from
[`bench/inference/eye-comparison.md`](../inference/eye-comparison.md): **dt1k** and **dt10k** EYE
forward-closure wall-clock numbers exist (idle-machine M1); **dt100k is `n/a` — EYE was not run at
that depth** (extrapolated only; NOT fabricated). See the honesty note in `competitors.json`.
