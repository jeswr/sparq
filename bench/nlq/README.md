<!-- [OPUS-4.8] sq-k0km — sparq-nlq OFFLINE NL->SPARQL feed for the CI bench dashboard. -->
# sparq-nlq offline NL→SPARQL feed

A small, self-asserting per-commit runner that feeds the dashboard's **GenAI
family** card (`similarity · introspection · NL→SPARQL`) so it stops rendering
"not yet reported". It mirrors the LUBM / SHACL / HDT / RSP / Solid template: a
crate-example runner wrapped by a `run.sh` built around a **deterministic count
gate**.

It exercises [`sparq-nlq`](../../crates/sparq-nlq)'s **offline** core: schema
grounding (the `sparq-introspect` scan + token-budgeted summary render), prompt
construction, and the `extract → spargebra-validate → budgeted-execute` loop.

## Fully offline, deterministic (honest)

The LLM is a tiny **in-example fixed-completion stub** — **no network, no `live`
feature, no Anthropic round-trip** — so the example measures the crate's
deterministic core, never a model call. The synthetic typed graph is generated
in-example at a **pinned** `N` (the `run.sh` `NLQ_N` default), so every harvested
value is a pure function of `N` + the synthetic schema and is byte-stable across
machines. The per-phase microsecond timings the example prints are
**NON-CANONICAL** and are **NOT** harvested.

What IS harvested and asserted are the deterministic counts: the synthetic triple
count, the grounded-prompt **char length** (a token-budget proxy — smaller is a
leaner prompt), the `ask` loop's result-row count, and its repair-round count.

This is a `featured = false` trend/coverage row, not a head-to-head competitor
card. **HONESTY:** this is the *offline* deterministic loop only — LIVE
NL→SPARQL exec-accuracy on a canonical host is a **separate** concern (beads
`sq-qidj` / `sq-g0lw`, EC2-blocked) and is **not** claimed by this feed.

## Sibling GenAI suites are EC2/dataset-gathered, not here

The other GenAI suites — `sim-olympics-eval` (entity-similarity AUC/precision)
and `introspect-olympics` (introspection build/output) — depend on the
**gitignored** `bench/qlever-olympics/olympics.nt` (~1.78M triples), which is
**not present in CI**. They are therefore **not** wired into this feed; their
metrics are EC2/dataset-gathered. NL→SPARQL alone lights up the GenAI family row.

## The gate: offline-loop counts (deterministic)

`run.sh` runs the example at the pinned `N`, parses its stdout into the counts
above, and diffs each against [`expected.tsv`](./expected.tsv) (DERIVED by
running the example, not guessed). It exits non-zero on any drift, so a grounding
or generation regression fails the whole `ci-bench` run. If you change `NLQ_N`
you must re-derive `expected.tsv`.

The metric names lead with the `nlq` token so the dashboard's `familyOf` prefix
routing buckets them into the GenAI family automatically.

## Run it

```sh
cargo build --release -p sparq-nlq --example bench
bench/nlq/run.sh          # asserts vs expected.tsv; prints the <metric>\t<value>\t<unit> contract
```

The `nlq` hook in [`scripts/ci-bench.sh`](../../scripts/ci-bench.sh) builds the
example and invokes `run.sh` on the **main / local tier** (skipped on the PR
tier, like the Solid/RSP/HDT/ZK example hooks), harvesting the counts into the
`customSmallerIsBetter` feed the dashboard reads.
