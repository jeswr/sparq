<!-- [OPUS-4.8] sq-k0km — Solid WAC/ACP auth-view feed for the CI bench dashboard. -->
# Solid WAC/ACP auth-view feed

A small, self-asserting per-commit runner that feeds the dashboard's **Solid /
access-control family** card so it stops rendering "not yet reported". It mirrors
the LUBM / SHACL / HDT / RSP template: a crate-example runner wrapped by a
`run.sh` built around a **deterministic count gate**.

It exercises [`sparq-solid`](../../crates/sparq-solid): WAC + ACP authorization-view
materialization over the in-crate fixture
(`sparq_solid::fixture::{wac_fixture, acp_fixture}`).

## Why counts, not timings (honest)

The example (`crates/sparq-solid/examples/bench.rs`) also prints per-phase
millisecond timings, but those are **NON-CANONICAL** (a dev / GitHub-runner box
is not a quiet bench instance) and are **NOT** harvested. What IS harvested and
asserted are the **deterministic structural counts** — a pure function of the
**fixed** in-crate fixture, so they are byte-stable across machines: how many
named graphs / quads the fixture holds, how many triples each materialised auth
view contains, how many graphs are visible to ALICE, and the authorized-subset
row count of the title query (the `query_as` vs `query_as_rewrite` paths
assert-equal in the example).

There is **no competitor perf column**: no Solid peer engine computes a
like-for-like WAC/ACP auth view, and this suite is `featured = false` in
`bench/benchmarks.toml` (a dashboard disposition — a trend/coverage row, never a
head-to-head card).

## The gate: auth-view counts (deterministic)

`run.sh` runs the example, parses its stdout into the counts above, and diffs
each against [`expected.tsv`](./expected.tsv) (DERIVED by running the example, not
guessed). It exits non-zero on any drift — exactly like LUBM's row-count diff —
so a fixture or materialization regression fails the whole `ci-bench` run.

The metric names lead with the `solid` token so the dashboard's `familyOf`
prefix routing buckets them into the Solid family automatically.

## Run it

```sh
cargo build --release -p sparq-solid --example bench
bench/solid/run.sh        # asserts vs expected.tsv; prints the <metric>\t<value>\tcount contract
```

The `solid` hook in [`scripts/ci-bench.sh`](../../scripts/ci-bench.sh) builds the
example and invokes `run.sh` on the **main / local tier** (skipped on the PR
tier, like the RSP/HDT/ZK example hooks, to keep per-PR CI lean), harvesting the
counts into the `customSmallerIsBetter` feed the dashboard reads.
