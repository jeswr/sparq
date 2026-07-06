<!-- [OPUS-4.8] sq-i6du2.7 (epic sq-i6du2, #1613). Written while Fable unavailable; flag for re-review when Fable returns. -->
# bench/ac — access-controlled-query benchmark harness

Runs the **WAC / ACP / ODRL** access-controlled-query benchmark: for every use-case
dataset the `sparq-acbench` crate generates, it re-checks the generator's
by-construction expected output against the crate's shared procedural oracle. Design +
workload taxonomy: `research/ac-query-benchmark.md` (§2–§3). Substrate crate (generators
+ workload/oracle engine): `crates/sparq-acbench`.

Standalone cargo project (own `[workspace]` table, same isolation pattern as `bench/parse`
+ `bench/dict`): the `ac-bench` driver path-depends on the dev-only crate `sparq-acbench`
and **links no engine crate**, so the oracle is structurally independent of the system
under test. `Cargo.lock` is committed for reproducible `--locked` builds; `target/` is
gitignored.

## Run

```sh
# per-commit smoke tier (fixed seed; the CI lane) — builds the driver, runs the lanes:
bench/ac/run.sh --smoke

# a larger deterministic tier (nightly / EC2):
bench/ac/run.sh --sf 10
```

The runner streams a per-suite TSV table (`suite<TAB>lane<TAB>model<TAB>status<TAB>detail`)
plus a pass/fail/skip summary, and is **fail-closed**: any W1/W2/W3 mismatch against the
by-construction oracle makes the driver exit non-zero, and `run.sh` propagates that exit.
It is also registered as the `ac-oracle` suite in `bench/benchmarks.toml` and runs under
`scripts/bench/run-all-benchmarks.sh --only ac-oracle`.

## Lanes

| lane | what it checks | here |
| --- | --- | --- |
| **W1** | per-request decision agreement (oracle re-evaluated over the intent table == the generator's expected decision) | runs |
| **W2-oracle** | result-set: the generator's closed-form rows == authorized ∩ candidate (no over-share / under-share) | runs |
| **W3** | ACL-write churn: a grant that survives a revoke (a stale grant) flips the expected `Deny` and fails the lane | runs (base-table-checkable deltas) |
| **W2-live** / **W4-query** | the real `query_as` output + the concurrency query sub-lane | **Skipped-with-reason** — engine-linked, owned by bead **sq-kvvcl** |

A use-case generator that is not yet implemented (its `generate` `todo!()`s) is
**Skipped-with-reason** for its whole suite, never counted as a pass. A documented
ODRL temporal/embargo divergence (see `crates/sparq-acbench/tests/consortium.rs`) is
parked as a visible skip rather than a spurious fail; every non-temporal disagreement
still fails-closed.

## Honesty

Every wall-clock line the driver prints is **advisory + NON-CANONICAL** on a shared work
box (the QUIET-BOX convention in `bench/CATALOG.md`); no number here is committed to
markdown. The load-robust contract is the deterministic fail-closed oracle exit code.
