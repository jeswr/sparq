# Non-coverage test-execution topology — the per-leg wall-time inventory (sq-6vshe.7)

**Status:** design record + shipped instrument. The measurement half is an instrument,
not a table: the table has to be produced by running the instrument against the real
Actions API, which this record cannot do for the reader.
**Author:** Claude Opus 5 (SPARQ researcher tier), 2026-08-01. [OPUS-5]
**Parent design:** `research/ci-structural-speedup.md` §8. **Bead:** `sq-6vshe.7`.
**Non-overlap:** `sq-piapk` owns the coverage-instrumented topology; `sq-fmx4u` owns
*which* legs run. This record is the NON-coverage execution topology only.

---

## 1. What this record corrects in the bead's brief

The bead was written against the 2026-07 shape of CI. Three of its five deliverables have
been overtaken by work that has since landed, and one of them is blocked by a constraint
the brief does not mention. Verified against the tree at `902766e9`:

| Bead deliverable | Actual state |
|---|---|
| (c) "extend the build-once nextest-archive pattern to the slowest non-coverage legs" | **The pattern already exists** — `ci.yml`'s `build-archive` job runs `cargo nextest archive --workspace --all-targets` once and the 5 `test` shards run it with `--archive-file`, never recompiling (`ci.yml:376-540`, `:669-806`). What remains is whether it can be *extended*, and §4 finds that for the obvious candidates it **cannot**. |
| (d) "each doctest compiles + links as its own crate … keep a `cargo test --doc` lane" | **The lane already exists and already runs once**, off the warm archive build inside `build-archive` (`ci.yml:537-538`) — not per-shard. The per-doctest compile cost premise does still hold (§5), but the bill is ~126 compiling doctests, not the ~300 fences a naive count suggests. |
| (e) "fold many tiny always-run legs into fewer fatter jobs" | **The mechanism already exists and is proven** — `scripts/assemble-feature-matrix.py --grouped` + `scripts/run-feature-matrix-group.py` bin-pack the 186 opt-in feature legs into shared-runner groups so consecutive legs reuse one warm `target/`. The remaining work is *applying that same mechanism* to `ci.yml`'s conformance family (§6), not inventing it. |
| (a) per-leg wall-time inventory | **Genuinely missing.** Nothing in the repo collected per-job, per-step durations. This record ships the instrument that does. |
| (b) nextest partition rebalance | **Genuinely open, and blocked on (a).** §3. |

So the bead's real residual content is (a), which gates (b), plus a redirected (c)/(e).

## 2. The instrument — `scripts/ci_leg_walltime_inventory.py`

`sq-6vshe.7(a)` calls its table "the steering data for the whole program". Steering data
has to be measured, and the measurement did not exist. The two scripts that read the same
API answer different questions: `ci_execution_latency_alarm.py` asks "is a lane failing to
fire, or overrunning?" and keeps no per-step decomposition; `ci_summary_gate.py` is gate
logic. Neither can produce a bucket split.

The instrument pulls completed runs, and for every job decomposes the wall-clock using the
per-step `started_at`/`completed_at` the jobs API already returns:

| Bucket | What lands there |
|---|---|
| `queue` | `job.started_at - job.created_at` — runner pickup, not part of the job's wall-clock |
| `setup` | runner set-up/teardown, checkout, toolchain, tool install, cache restore/save, artifact transport, disk reclaim — **the per-leg constant tax (e) is trying to fold away** |
| `fixture` | pinned external corpus fetches (W3C suites etc.) — a separable tax only the conformance legs pay |
| `compile` | `cargo build`/`check`, `nextest archive`, clippy, rustdoc, `wasm-pack build`, maturin |
| `doctest` | the `cargo test --doc` lane — broken out because (d) is a doctest audit and nextest never runs doctests |
| `test` | actual execution: `nextest run`, `cargo test`, `wasm-pack test`, pytest, the conformance/oracle drivers |
| `other` | ratchet/guard/report scripts, plus **anything no rule matched** |
| `unaccounted` | job wall-clock minus the sum of its steps — runner overhead the step list does not itemise; reported, never redistributed |

```sh
# measure (needs an authenticated `gh`), keeping a reproducible dump
scripts/ci_leg_walltime_inventory.py --repo sparq-org/sparq --runs 20 \
  --event pull_request --dump-json /tmp/jobs.json

# re-analyse offline, no network
scripts/ci_leg_walltime_inventory.py --from-json /tmp/jobs.json --top 25
```

### 2.1 The honest limitation, designed in rather than hidden

The Actions API exposes a step's *name*, `started_at` and `completed_at` — and nothing
else. There is no machine-readable "this step was a compile". The split is therefore
inferred from step names, which is an approximation that **decays quietly** as CI grows
step shapes no rule knows: an unmatched step still lands somewhere and the table still
renders. Three things make that visible instead of silent:

1. an unmatched step lands in `other`, never in a flattering bucket;
2. the report always prints the unclassified share of measured wall-clock and names the
   top unmatched steps, so a decayed table says so on its face;
3. `--max-other-pct` turns that into a hard non-zero exit for unattended use.

`scripts/tests/test_ci_leg_walltime_inventory.py` pins the seam that the script's own
`--self-test` structurally cannot see: it classifies **every real step name in the live
`ci.yml` + `feature-matrix.yml` tree** (196 distinct step names; 1 unmatched at the time
of writing, itself a coverage-job step that is excluded anyway) and bounds the unmatched
share, with a non-vacuity control so the bound cannot be met by a rule that matches
everything. It also pins the `sq-piapk` non-overlap (coverage jobs excluded unless opted
in), that a *skipped* leg is never averaged in as a cheap sample, and that every
fail-loud path exits 2 rather than rendering a table over an empty population.

**Read a bucket split with a high `other` share as "this leg is not classified yet", not
as "this leg has no compile".**

## 3. (b) The partition rebalance — what the leg names do and do not mean

`ci.yml:540-660`. The brief's warning is correct and worth restating precisely: the shard
named `bulk N/3` does **not** run "a third of the suite". It runs
`--partition count:N/3` over the set `not ( $HEAVY )`, where `HEAVY` is a two-atom
exact-match filterset (`ci.yml:599`) naming the two vector-recall tests. The three bulk
shards plus the two heavy shards are a genuine partition of the suite — the bulk filter is
the *negation* of `HEAVY`, so adding an atom to `HEAVY` automatically removes it from the
bulk shards and coverage cannot silently drop.

The current split isolates a known-heavy tail and count-partitions the uniform rest. That
is the right *shape* for a severely skewed distribution. What cannot be decided from the
repo is whether it is still the right *parameterisation*:

- the two heavy atoms were identified from a dated measurement recorded in the job's own
  comment; nothing re-checks that they are still the top two, and the documented upkeep
  path is a human noticing nextest's `SLOW [> Ns]` markers in a log;
- `count:K/3` is runtime-blind *within* the bulk set. It is only sound while that set is
  uniform, which is an assumption the comment states and nothing measures;
- the shard count (3) has never been re-derived against the constant tax. Each additional
  bulk shard buys a third of the bulk runtime and costs a full `setup` bucket. Below some
  bulk runtime, a fourth shard is a net loss — and the crossover is exactly the
  `setup`-vs-`test` ratio the instrument measures.

**Do not rebalance before running the instrument.** A rebalance argued from the numbers
baked into the job comment would be a rebalance argued from an undated measurement, which
is how the current skew was introduced in the first place.

## 4. (c) Why the archive cannot simply be extended

The obvious targets — the 10 conformance/oracle jobs in `ci.yml` (`conformance`,
`shacl-`, `geo-`, `solid-`, `odrl-`, `jsonld-`, `service-federation-`,
`inference-conformance`, `text-oracle`, `rsp-oracle`) — each spin their own runner, their
own Swatinem cache restore, and their own compile. Extending `build-archive` to cover them
runs into three independent blockers, and each is sufficient on its own:

1. **Different cargo profile.** Several legs drive `cargo run --profile release-fast -p
   sparq-conformance`. A `nextest archive` carries test binaries built in the test
   profile; a `release-fast` binary is a different artifact entirely.
2. **Different feature sets, and a guard against widening.** The archive is built with
   exactly `approx-ann,filtered-ann,vec-predicate`, and `ci.yml:490-497` records an
   explicit guard (`sq-vya1`) that it carries **no other** opt-in feature — anything else
   belongs in a `feature-matrix.yml` leg. The conformance legs need `shacl-af`,
   `geosparql_rewrite`, `service-loopback`, `http-protocol`, `federation-descriptors`,
   `dl-direct` and more. Adding them to the archive would change what the five `test`
   shards execute, which is precisely what that guard exists to prevent.
3. **Not all of them are test binaries.** `nextest archive` archives test targets;
   `cargo run -p sparq-conformance` is a bin target the archive does not carry.

The `--extract-to .` mechanics are also load-bearing and fragile in a way worth recording:
`crates/sparq-cli/tests/cli_contract.rs` spawns the CLI via `env!("CARGO_BIN_EXE_sparq-cli")`,
which bakes the **build machine's absolute path** into the test binary at archive time.
The shards only work because GitHub-hosted runners check out to the same absolute path and
the archive is extracted into the checkout (`ci.yml:669-806`). Any future consumer of the
archive inherits that constraint.

**Conclusion: (c) as literally worded is not the right lever for these legs.** The
compile-amortisation they need is available by a different, already-proven route — §6.

## 5. (d) The doctest bill, measured statically

Counted over `crates/**/*.rs`, opening rustdoc code fences inside `///`/`//!` comments:

| Fence info-string | Count | Compiles? |
|---|--:|---|
| (none — defaults to Rust) | 108 | compiles **and runs** |
| `rust` | 3 | compiles and runs |
| `no_run` / `rust,no_run` | 11 + 1 | compiles + links, does not run |
| `compile_fail` | 3 | compiles (expected to fail) |
| `ignore` / `rust,ignore` | 8 + 2 | **not** compiled |
| `text` | 107 | not compiled |
| `sh` | 28 | not compiled |
| `json` | 18 | not compiled |
| `js` | 7 | not compiled |
| `n3`, `turtle`, `ntriples`, `xml` | 4 + 1 + 1 + 1 | not compiled |
| **Total fences** | **303** | **~126 compile** |

Two findings follow, and they point in opposite directions:

- **The bead's cost premise holds.** The workspace is `edition = "2021"`
  (`Cargo.toml:33`) on a pinned `1.97.1` toolchain (`rust-toolchain.toml`). rustdoc's
  merged-doctest compilation is an **edition-2024** behaviour, so on this workspace each
  of those ~126 doctests really is compiled and linked as its own crate.
- **The bill is much smaller than a fence count implies, and it is already paid once.**
  167 of the 303 fences (55%) are not Rust at all and a further 10 are `ignore`d — 58%
  of the fences are never compiled by anything — and the lane runs a single time inside
  `build-archive` off an already-warm build — not once per shard, and not once per
  conformance leg.

**Recommendation: do not consolidate doctests, and do not mark working examples `no_run`
to save compile time.** `no_run` still compiles and links — it saves only execution, which
is the cheap half — while costing the property that makes a doctest worth having (the
example is *known to work*, not merely known to typecheck). The honest sequencing is:
measure the `doctest` bucket with the instrument first; act only if it is a materially
large share of `build-archive`'s wall-clock. If it is, the lever with the right cost/value
trade is an **edition-2024 migration** (which buys merged doctests workspace-wide for
free), not hand-degrading examples. Either way the `cargo test --doc` lane must survive —
nextest never runs doctests, so deleting the lane silently deletes the coverage.

## 6. (e) Fold the conformance family — the recommendation

The highest-payoff change this record can point at, and the one it recommends:

**Apply the existing feature-matrix bin-packing pattern to `ci.yml`'s 10-job conformance /
oracle family.** Today each is a separate runner paying `queue + checkout + toolchain +
cache-restore + compile` to run one suite. They all sit on the same `sparq-engine`
dependency stack, so consecutive legs in one job would share a warm `target/` and
recompile only their own crate — exactly the argument `run-feature-matrix-group.py`
already makes for the 186 opt-in legs, on a family whose per-leg work is comparable.

Two properties must be preserved, and both have prior art in that same script:

- **Per-leg check-run identity.** Each conformance leg's check-run name is discovered
  by the `ci-summary` gate aggregator *by name*; folding jobs must keep emitting one
  check-run per leg or the gate silently stops requiring them. `run-feature-matrix-group.py`
  solves this with a results-artifact plus a separate default-branch-owned reporter
  workflow, deliberately split across a trusted boundary so a PR cannot forge check runs.
  Reuse that split; do not invent a second one.
- **Failure isolation.** One folded job means one red job for several suites. The
  per-leg check-runs restore the granularity, but `fail-fast` semantics inside the group
  need to be explicitly "run every leg, report each" rather than "stop at the first red".

Whether to fold 10 legs into 2, 3 or 4 groups is a bin-packing question that needs the
per-leg `compile` and `test` means — i.e. it is gated on §2, like everything else here.

## 7. Phased plan (future beads)

Ordered by payoff-per-risk; each is a separate bead so they can land independently.

1. **Run the instrument and commit the table.** One authenticated run over ~20
   `pull_request` runs of `ci.yml` + `feature-matrix.yml`, output appended to this record
   as a dated measurement. Gates every phase below. *Risk: none — read-only.*
2. **Fold the conformance/oracle family into bin-packed groups** (§6), reusing
   `run-feature-matrix-group.py`'s execution/report split. Largest single win against the
   per-leg constant tax. *Risk: medium — check-run identity and failure isolation are the
   things to get wrong.*
3. **Re-derive the `HEAVY` set and the bulk shard count from the measured table** (§3).
   Includes deciding whether 3 bulk shards is still above the constant-tax crossover.
   *Risk: low-medium — the negation-based bulk filter keeps coverage correct by
   construction, so the failure mode is a worse balance, not a lost test.*
4. **Automate the `HEAVY` upkeep.** Today a new slow test re-skews the shards until a
   human notices a `SLOW [> Ns]` marker in a log. The instrument's machinery is the
   natural home for a periodic "is `HEAVY` still the top-N?" check that files a bead.
   *Risk: low — advisory detector, no gate.*
5. **Decide the doctest question on measured data** (§5): act only if the `doctest`
   bucket is a material share of `build-archive`; prefer an edition-2024 migration to
   hand-degrading examples. *Risk: low to defer, medium to migrate.*

Phases 2 and 3 are disjoint (different jobs, different files) and can run in parallel
once phase 1 lands.

## 8. Open questions for the maintainer

1. **Fold granularity vs. blast radius.** Folding the conformance family trades ~7 runner
   jobs' constant tax for a coarser failure surface — a red group needs one extra click to
   attribute even with per-leg check-runs. Is that trade acceptable on a *gating* family,
   given it was already accepted on the opt-in feature matrix?
2. **Edition 2024.** Merged doctests are one of several reasons to migrate, and the
   migration is a workspace-wide change that would collide with most open PRs. Is it worth
   scheduling on its own merits, with the doctest saving as a side benefit rather than the
   justification?
3. **Cadence.** Should the instrument run periodically (a scheduled, non-gating lane that
   refreshes the table and files a bead when a leg's split moves materially), or stay a
   manually-invoked tool? A scheduled lane would need an advisory-registry declaration.

## 9. Uncertainties, stated

- **No wall-time numbers appear in this record.** The instrument is shipped and tested;
  it has not been run against the live API here, because doing so needs credentials this
  session does not use. Every quantitative claim above is either counted from the tree
  (doctest fences, leg counts, step names) or cited as a dated figure recorded in
  `ci.yml`'s own comments — never re-measured, and never presented as current.
- **The figures in `ci.yml`'s shard comment are undated relative to today.** They are
  quoted here as the *provenance of the current design*, not as evidence for keeping it.
- **The step-name classification is an approximation** (§2.1) and its accuracy on lanes
  outside `ci.yml`/`feature-matrix.yml` is unmeasured. Extend `_RULES` before trusting a
  split for a lane not covered by the test suite's population.
- **The 186 opt-in leg count** is counted from `.github/feature-matrix.d/*.yml` and cross-
  checks against the 186-line `scripts/tests/feature-matrix-legnames.golden.txt`. The
  number of *groups* those legs bin-pack into is not stated here: the assembler needs
  PyYAML, which this session could not run, so any group count would be a guess.
