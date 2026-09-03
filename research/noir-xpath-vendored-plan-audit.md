<!-- [OPUS-5] sq-mi1b8 (#3151): completion audit for the two vendored-plan beads sq-t9d
(Phase-0) and sq-t9v (streams A-H). AUDIT ONLY — no production code, and none is possible
here: the audited tree lives in sparq-org/noir_XPath, not in this repo. -->
# `noir_XPath` vendored-plan audit — Phase-0 (`sq-t9d`) and streams A–H (`sq-t9v`)

Completion audit for bead **`sq-mi1b8`** (issue **#3151**), which asks whether the two beads the
`sq-u3u15` repo-hygiene sweep lifted out of `zk/xpath/IMPLEMENTATION_PLAN.md` and
`zk/xpath/PARALLEL_WORK_ITEMS.md` are substantially complete, and to audit each stream against
its qt3tests acceptance criteria.

Sits alongside [`noir-xpath-xsd-decimal-design.md`](noir-xpath-xsd-decimal-design.md) — the other
record produced from the same sweep's deleted plan documents — and under
[`zk-correctness-and-proof-program.md`](zk-correctness-and-proof-program.md), which owns the
`sq-3x7dl.*` XPath correctness findings. It does not restate or contradict either.

---

## 0. Honesty framing (read first)

- **This is git archaeology, not a live test run.** Every claim below is read off the sparq tree
  at **`f9e7aa49`** — the last commit that still contained the vendored `zk/xpath` sub-project,
  immediately before **`19bf414a`** (`sq-5reoy`, #1602) deleted it. Nothing here was re-executed:
  `nargo` cannot run against a tree that no longer exists in this repo.
- **"Passing" means *was gated*, not *was re-verified today*.** The passing evidence is that
  `.github/workflows/zk-toolchain.yml` at `f9e7aa49` ran `nargo test` over the dynamically
  detected non-stub `test_packages` with an **empty `KNOWN_FAILING=()`** array, and that lane was
  a merge precondition for `f9e7aa49` landing on `main`. No stream package was exempted. That is
  CI-gate evidence, not an independent re-run.
- **The audited code is no longer in this repo.** Per `sq-5reoy` (#1599/#1602) the tree was
  externalized; `git ls-files zk/xpath` now returns only the differential harness
  (`zk/xpath/differential`, `zk/xpath/scripts`, `zk/xpath/tests/differential_oracle`). The audited
  library is [`sparq-org/noir_XPath`](https://github.com/sparq-org/noir_XPath), which this repo
  consumes at the tag pinned in `.github/workflows/xpath-differential.yml`.
- **This record says nothing about the face repo's *current* state.** It audits the tree as
  published, not whatever `main` there holds now. A regression or an addition upstream after the
  externalization is outside what any evidence in this repo can speak to.
- **No soundness or privacy claim.** The ZK estate remains research-grade and **NOT externally
  audited** (`sq-qhy4`). A green qt3 package proves agreement with the W3C expected value on the
  sampled inputs; it proves nothing about circuit soundness.

---

## 1. Both beads are unactionable *in this repo*

`sq-t9d` and `sq-t9v` were written against the **vendored** sub-project — they name
`Nargo.toml` workspace members, `scripts/generate_tests.py`, `xpath/src/*.nr`, and
`.github/workflows/ci.yml` paths that all moved to the face repo. Neither bead can be worked, or
its acceptance criteria re-checked by running anything, from `sparq`. Whatever their verdict, they
cannot stay on the launchable frontier as sparq work.

That is the same disposition [`noir-xpath-xsd-decimal-design.md`](noir-xpath-xsd-decimal-design.md)
reached for `sq-n5e7p`: decide here, implement (if anything remains) in the face repo behind an
`XPATH_TAG` bump.

---

## 2. `sq-t9d` — Phase-0 workspace/CI setup

The bead body carries the unchecked boxes from `IMPLEMENTATION_PLAN.md` § "Phase 0: Project
Setup". Each one, against `f9e7aa49`:

| Phase-0 item | Evidence at `f9e7aa49` | Verdict |
| --- | --- | --- |
| Convert to workspace structure | `zk/xpath/Nargo.toml` is a `[workspace]` with an explicit `members` list | **done** |
| Create `xpath/` main package | `zk/xpath/xpath/Nargo.toml` (`type = "lib"`), 26 modules under `src/` | **done** |
| Create `xpath_unit_tests/` package | `zk/xpath/xpath_unit_tests/`, 14 test modules wired from `src/main.nr` | **done** |
| Create `test_packages/` directory | 360 generated package directories, all 360 workspace members — 113 with real assertions, 247 stub-wired against unimplemented functions | **done** |
| Create `scripts/` directory | `generate_tests.py`, `benchmark_gates.py`, `bench_float_migration.py`, `cleanup_stubs.py`, `README.md` | **done** |
| Add ieee754 dependency to `xpath` | `xpath/Nargo.toml` deps on `sparq_ieee754` (migrated from vendored `noir_IEEE754` v0.3.1 — see the deleted `VENDOR.md` "Migration" section) | **done** |
| Configure workspace members in root `Nargo.toml` | the `members` list above | **done** |
| GitHub Actions workflow for testing | `zk/xpath/.github/workflows/ci.yml` — `lint` (`nargo fmt --check`) + `test` (`nargo test --workspace`) + a `test-summary` aggregator | **done** |
| Configure test chunking for parallel CI | **partial — see below** | **substantially done** |

**The one item that is not literally what the plan asked for.** "Test chunking for parallel CI"
exists as **source-file** chunking, not as per-chunk CI jobs: `generate_tests.py` splits each
function's generated cases into `chunk_N.nr` files at `chunk_size = 50`, which keeps individual
Noir compilation units small. CI parallelism is a **matrix over Noir toolchain versions** read
from `.github/noir-versions.json`, with each matrix leg running the whole workspace. Both halves
of the intent — bounded compile units, parallel legs — are present; the specific
"chunk the test run across jobs" shape is not. This is a shape difference in the face repo's own
CI, not a gap in sparq.

**A related observation, recorded because it bears on what "CI pipeline running" buys.** The
vendored `ci.yml`'s test step is `nargo test --workspace`, and by `f9e7aa49` **all 360**
`test_packages` were workspace members — of which **247 are stub-wired** (a `stub_`-prefixed
import whose body asserts `false, "not available in ZK"`, or the
`No qt3tests cases could be converted` placeholder). A `--workspace` run over that membership
cannot be green. sparq's own lane deliberately did not inherit it: the comment block in
`.github/workflows/zk-toolchain.yml` at `f9e7aa49` states outright that it does **not** run
`nargo test --workspace` from the workspace root, and instead detects the non-stub subset by
grepping for those markers, so the gate "grows honestly with real coverage" as `sq-3x7dl.15`
retires stubs. The audit below therefore measures against **sparq's** detector, not against the
vendored workflow.

**Verdict: `sq-t9d` is COMPLETE.** Every deliverable it lists ("working workspace structure",
"module files with function stubs", "CI pipeline running") is present and was present at
externalization. The bead's own body already flagged this ("Several already done per recent
commits; file says STATUS COMPLETE elsewhere").

---

## 3. `sq-t9v` — parallel work streams A–H

`PARALLEL_WORK_ITEMS.md` gave each stream an acceptance criterion of the form *"test packages
generated and passing"*, plus, for the four streams that required new code (C, D, G, H),
*"functions implemented"* and *"unit tests passing"*.

Method: for each stream, take the package list from its own **"Files to Create"** section, then
check three things at `f9e7aa49` — (1) the package directory exists, (2) it is a `members` entry
in the workspace `Nargo.toml` (a non-member is never run), and (3) it is **not stub-wired**
(the zk-toolchain lane's own detector: no `stub_` import and no
`No qt3tests cases could be converted` placeholder marker, either of which means the package
asserts `false` for an unimplemented function and is deliberately excluded from the gate).

The **audited** column below is deliberately wider than the plan's own list: it also counts
adjacent packages on the same type surface that the plan never enumerated — `opsubtract_dates`
and `opadd_daytimeduration_to_date` (C), `opsubtract_times` and `opadd_daytimeduration_to_time`
(D), and `opboolean_equal` (F). Those five are extra evidence, not acceptance criteria, so the
parenthesised number gives the plan-enumerated subtotal and the two can be told apart.

| Stream | Scope | Packages audited (plan-named) | All members | All non-stub | Gated qt3 cases | Library code |
| --- | --- | --- | --- | --- | --- | --- |
| **A** | duration components | 4 (4) | yes | yes | 24 | pre-existing `duration.nr` |
| **B** | dateTime↔duration arithmetic | 8 (8) | yes | yes | 46 | pre-existing `datetime.nr` / `duration.nr` |
| **C** | `xs:date` type + functions | 8 (6) | yes | yes | 52 | `XsdDate` in `types.nr`; `date.nr`; `xpath_unit_tests/src/date_tests.nr` |
| **D** | `xs:time` type + functions | 8 (6) | yes | yes | 51 | `XsdTime` in `types.nr`; `time.nr`; `xpath_unit_tests/src/time_tests.nr` |
| **E** | numeric unary operators | 2 (2) | yes | yes | 68 | pre-existing `numeric.nr` |
| **F** | boolean comparison | 3 (2) | yes | yes | 79 | pre-existing `boolean.nr` |
| **G** | float/double rounding | 6 (6) | yes | yes | 187 | `round_/ceil_/floor_` × `float`/`double` in `numeric_types.nr` |
| **H** | timezone adjustment | 3 (3) | yes | yes | 23 | `adjust_{datetime,date,time}_to_timezone{,_none}` in `datetime.nr` / `date.nr` / `time.nr` |

All **42** audited packages — the **37** the plan enumerates plus the five extras — exist, are
workspace members, and carry real assertions against real library functions: 530 gated
qt3-derived `#[test]` functions in total. One naming note: Stream B's plan spells its packages
`xpath_test_opdaytime_duration_*`; the generator emitted `xpath_test_opdaytimeduration_*`, and
all eight are present under the generator's spelling. Spot-checked bodies confirm they assert
values rather than call stubs (e.g.
`fn_days_from_duration1args_2` asserts `days_from_duration(duration_from_microseconds(1339199000000)) == 15`;
`fn_adjust_datetime_to_timezone1args_2` asserts all six components of the adjusted result).

Streams C and D also satisfy their **new-code** criteria: `XsdDate` and `XsdTime` are declared in
`types.nr`, every function the plan lists is `pub` in `date.nr` / `time.nr` (plus `_le` / `_ge`
and `_with_tz` constructors beyond the list), and each has a dedicated unit-test module in
`xpath_unit_tests`. Stream H's three adjustment functions are implemented as **two functions
each** — `adjust_X_to_timezone(x, i16)` and `adjust_X_to_timezone_none(x)` — rather than the
plan's sketched `Option<i16>` parameter. That is an API shape difference, not a coverage gap:
both arms of the F&O contract (adjust-to, remove-timezone) are reachable and the qt3 packages
exercise them.

### 3.1 The one residual — `fn:round-half-to-even` (Stream G)

Stream G's *tasks* list `round_half_to_even_float` and `round_half_to_even_double` alongside the
six round/ceiling/floor functions. Those two are **not implemented**. The integer form
`round_half_to_even_int(value, precision)` does exist in `numeric.nr` (re-exported as
`round_half_to_even` from `xpath_fn.nr`), but the generated qt3 package
`xpath_test_fnround_half_to_even` is **stub-wired**: its 66 cases each call
`stub_fnround_half_to_even()`, whose body asserts `false, "not available in ZK"`. The package is
therefore excluded from the gate by the zk-toolchain lane's stub detector, exactly as designed —
it is honest, not masked, but it is also not coverage.

Note the asymmetry: an integer implementation exists yet even the integer qt3 cases run against
the stub, so wiring the generator to `round_half_to_even_int` would convert part of that package
without any new circuit code. That is upstream work and is captured as a follow-up rather than
done here.

Stream G's own **acceptance criteria** name only the six round/ceiling/floor packages, all of
which are green — so the residual is a gap against the stream's *task list*, not against the box
it asks to tick.

**Verdict: `sq-t9v` is SUBSTANTIALLY COMPLETE.** All eight streams meet their stated acceptance
criteria. The single residual (`fn:round-half-to-even` float/double, and its stub-wired qt3
package) is narrow, already visible in the gate's own stub partition, and belongs to the face
repo.

---

## 4. Recommended disposition

- **`sq-t9d` — close.** Complete; the one shape difference (CI chunking) is in the face repo's
  workflow and does not change the deliverable.
- **`sq-t9v` — close.** Substantially complete against its acceptance criteria; the residual is
  narrower than the bead and is tracked separately.
- Neither should be re-dispatched as sparq work. If the residual is picked up, it lands in
  `sparq-org/noir_XPath` and reaches this repo as an `XPATH_TAG` bump in
  `.github/workflows/xpath-differential.yml` and
  `zk/xpath/scripts/run_differential_harness.sh` — the two places the pin is kept in lock-step.

## 5. Re-running this audit

The evidence is reproducible from this repo's history without a network fetch:

```sh
# the tree as published to the face repo
REF=f9e7aa49

# Phase-0 shape
git show $REF:zk/xpath/Nargo.toml | head            # [workspace] + members
git ls-tree -r --name-only $REF zk/xpath/ | grep -v test_packages

# a stream package: member? non-stub? real assertions?
PKG=xpath_test_fnadjust_datetime_to_timezone
git show $REF:zk/xpath/Nargo.toml | grep "test_packages/$PKG"
git show $REF:zk/xpath/test_packages/$PKG/src/chunk_0.nr | grep -c '^#\[test\]'
git show $REF:zk/xpath/test_packages/$PKG/src/chunk_0.nr | grep 'stub_' || echo "non-stub"

# the gate that ran them, and its (empty) exemption list
git show $REF:.github/workflows/zk-toolchain.yml | grep -n 'KNOWN_FAILING'
```

**License:** MIT
