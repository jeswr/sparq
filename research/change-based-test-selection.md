# Change-based test + benchmark selection for CI (sq-fmx4u)

**Status:** design record (provisional — graduates into the AGENTS.md gate table +
`.github/` docs once implemented). Authored under the proceed-and-document rule.
**Author:** Claude Fable 5 (SPARQ architect tier), 2026-07-02. [FABLE-5]
**Implementation:** decomposed into disjoint child beads under `sq-fmx4u` (§8).

---

## 1. Problem

Every PR — and every merge-queue entry, since each queue entry re-runs its own
checks — executes the full required matrix: per-crate `cargo test`, the ~50
per-crate opt-in feature legs (`feature-matrix.yml`), benchmark jobs, fuzz
smoke, the coverage ratchet, wasm builds, CodeQL, and the docs gates — on the
order of ninety required checks. Runner job-slots are the binding resource.
Most PRs touch exactly one crate (a coverage test in `sparq-geo`, a leaf
feature in `sparq-rsp`), yet pay the whole matrix. Throughput is therefore
bounded by *matrix width × queue depth*, not by verification need.

**Goal.** For each PR, run a package's test + benchmark jobs only when that
package **or its transitive dependency closure** changed in the diff; skip the
rest with a green `skipped` conclusion that still satisfies branch protection,
the `ci-summary` aggregator, and the merge queue. A wrong skip is a *silent*
correctness hole (a red test simply never runs), so the design is governed by
an explicit fail-safe invariant (§2), a fail-closed mechanism (§4.3), and a
full-run backstop (§6).

**Expected effect** (estimate, not a measurement — the shadow-mode rollout in
§6.4 produces the real numbers): a leaf- or near-leaf-crate PR shrinks from
the full matrix to roughly its own closure's jobs plus the always-run lanes,
i.e. an order-of-magnitude reduction in runner-minutes for the common case,
and correspondingly shorter merge-queue occupancy per entry.

## 2. The skip invariant (normative)

> **A crate C's jobs may be skipped for a revision pair (base, head) only if
> every path changed between base and head is either (a) owned by a package
> outside C's transitive dependency closure, or (b) on the audited SAFE list
> of paths that no build, test, benchmark, or CI definition reads. Any changed
> path that is neither — and any failure to compute the above — forces the
> FULL matrix.**

Equivalently: *a test is skipped ⇒ provably no change could affect it.*
Skipping is an optimization applied to a proof of non-interference; absence of
proof means run. Every rule below is an instantiation of this invariant, and
every implementation choice defaults to running more, never less.

## 3. Affected-set algorithm

Computed once per workflow run by a cheap pre-job (`select`, §5.1) running a
small selector script (§7 position P1).

### 3.1 Changed paths

- `pull_request`: `git diff --name-only --no-renames <base-sha>...<head-sha>`
  (three-dot = against the merge-base with the target branch). `--no-renames`
  reports a rename as delete + add so **both** paths are attributed —
  conservative for moves between crates.
- `merge_group`: same diff against the target branch tip. A queue entry's diff
  therefore contains the union of every queued change ahead of it — exactly
  the conservative set, so selection stays sound inside the queue while still
  saving width (§7 P8). *(The full soundness argument, including the case where
  the payload's `base_sha` yields only the entry's own change rather than the
  union, is §10.)*
- `schedule` / `workflow_dispatch` / the `ci-full` label: no diff — `mode=full`
  (§6).
- If the base SHA cannot be fetched (shallow-clone trouble, force-push race):
  `mode=full` (§4.3).

### 3.2 Path → package ownership

From `cargo metadata --format-version 1 --locked`:

- Build the set of **in-repo path packages** — all workspace members *plus*
  any non-member path dependencies that live inside the repository (e.g. a
  vendored path dep). Each package owns the directory of its `Cargo.toml`.
- A changed file is owned by the package with the **longest matching directory
  prefix** (handles nested crates).
- Files owned by no package are looked up in the explicit ownership map
  `ci/path-ownership.toml` (§4.2): an entry attributes them to listed crates,
  or marks them SAFE. **An unmapped, unowned path forces `mode=full`.**

Granularity is deliberately whole-crate: any change under a crate's directory
marks the crate changed — no "only `benches/` changed" refinement (§7 P4).
This includes the crate's `Cargo.toml`, `build.rs`, `README.md` (routinely
pulled into doctests via `#[doc = include_str!(...)]`), tests, benches, and
fixtures under the crate dir.

### 3.3 Reverse-dependency closure

- Dependency edges are taken from the **package-level** dependency lists in
  `cargo metadata` (`packages[].dependencies` filtered to in-repo path
  packages), including **normal, dev, and build** kinds, **optional**
  dependencies regardless of feature activation, and **target-specific**
  dependencies (e.g. `cfg(target_arch = "wasm32")`) regardless of host. This
  is a superset of any feature-resolved graph, hence conservative (§7 P2).
  Dev-dependencies matter: if A is a dev-dep of B, changing A can break B's
  tests.
- `affected = ⋃ over changed crates c of ({c} ∪ reverse-closure(c))`,
  intersected with the workspace-member set for job selection.
- Feature-unification note: CI test jobs build per-crate (`cargo test -p C` /
  per-leg `-p C --features ...`), so a feature or dependency declaration
  change in member manifest `A/Cargo.toml` can only influence builds whose
  graph contains A — exactly A's reverse closure. (`[patch]` and `[profile]`
  are only honored in the workspace **root** manifest, which is a full-run
  trigger.) Any member-manifest change that alters resolved versions also
  rewrites `Cargo.lock` → full run anyway.

### 3.4 Edge cases (all resolve to "run more")

| Case | Resolution |
|---|---|
| New crate added | root `Cargo.toml` members change → full run |
| Crate removed / renamed | root `Cargo.toml` changes → full run |
| File moved between crates | `--no-renames` marks both crates changed |
| Deleted file inside a surviving crate | still prefix-owned by that crate |
| Symlink whose target crosses crate dirs | audit item (§4.2); unowned → full |
| `build.rs` or test reading outside its crate dir (`../`, `include_str!` escaping the crate) | must be enumerated in the ownership map by the audit (bead 2); unmapped shared dirs are unowned → full |
| Selector cannot parse metadata / diff / map | `mode=full` (§4.3) |

## 4. Fail-safe rules

### 4.1 Unconditional full-run triggers

Any changed path matching the following forces `mode=full`, no closure math:

| Trigger | Why a skip would be unsound |
|---|---|
| `Cargo.lock` | resolved versions feed every build |
| root `Cargo.toml` | members, `[workspace.dependencies]`, lints, profiles, `[patch]` |
| `rust-toolchain`, `rust-toolchain.toml` | compiler version affects all codegen |
| `.cargo/**` | rustflags / registries / build config are global |
| `.github/**` | the CI definition itself, including the selection wiring |
| `scripts/**` | shared gate/coverage/check scripts executed by CI |
| `deny.toml`, supply-chain / vet / audit / SBOM config | changes what "green" means for the supply-chain lanes |
| `ci/path-ownership.toml` | the selection policy itself |
| any path owned by no package and not SAFE-listed | unattributable ⇒ unprovable ⇒ full (§2) |

Changes to the selector script or its tests are subsumed by `scripts/**`; a
selection-logic PR therefore always validates against the full matrix.

### 4.2 The ownership map — `ci/path-ownership.toml`

A small, checked-in, audited policy file. Three ownership-verdict forms plus the
monotone `readers` union (below):

```toml
# Attribute a non-crate path to the crates whose tests read it:
[[map]]
pattern = "testsuites/w3c-sparql/**"
crates = ["sparq-conformance"]

# Prove a path inert for the Rust matrix (docs lanes still run):
[[map]]
pattern = "research/**"
safe = true

# Additional-readers: union extra reader crates for a CRATE-OWNED path where a
# real dep edge would be a cycle (monotone; see below):
[[map]]
pattern = "crates/sparq-solid/rules/**"
readers = ["sparq-reason"]

# Everything unmapped and unowned => mode=full (implicit default).
```

Rules:

- **The SAFE list starts empty.** Entries are added only by the audit (bead
  2), which greps for out-of-crate inputs — `include_str!`/`include_bytes!`
  escaping a crate dir, `../` path literals in tests/build scripts, repo-level
  files ingested by tests (e.g. `sparq-kb` PKG tests read `AGENTS.md` and
  `skills/**` — those paths must map to `sparq-kb`, not SAFE). Candidate SAFE
  entries to evaluate, not presume: `research/**`, `site/**` (owns its own CI
  lane), `.beads/**`.
- A map-validity unit test asserts every `crates` name is a current workspace
  member and every literal pattern root exists — the map cannot silently rot
  when crates move.

**Additional-readers (monotone union)** — sq-m4bxc [FABLE-5]. A fourth verdict
form, `readers = [...]`, added as a *deliberate deviation from pure
input-relocation*. The two closed residuals were sibling reads across a boundary
where the "attribute the shared input to both crates" fix is not available: the
input lives *inside* a crate dir (so crate-prefix ownership, steps 1–2, wins
before the map is ever consulted for a `crates` attribution) **and** the reading
crate is not in the owner's reverse-dependency closure. Relocation was rejected
because it would move a *vendored crypto ontology*
(`crates/sparq-trust/ontologies/zkp-sparql/secprop-ext.ttl`) and a *crate's core
authorization rule corpus* (`crates/sparq-solid/rules/*.n3`) out of their owning
crates and rewrite production `include_str!` / runtime include paths in
`sparq-trust`, `sparq-solid`, `sparq-zk` and `sparq-reason`. The dep-edge
alternative is a **cargo cycle** in both cases (`sparq-trust` depends on
`sparq-zk`; `sparq-solid` depends on `sparq-reason`), so `reason -> solid` /
`zk -> trust` edges are impossible.

`readers` is the one verdict form consulted **even for a crate-owned path**: it
UNIONS the listed crates into the changed-crate set *in addition to* the path's
normal prefix owner, without changing the ownership verdict. This makes it
strictly **monotone / fail-safe** — adding a `readers` entry can only ENLARGE the
affected set (it never rescues an unowned/unmapped path from `mode=full`), so it
cannot introduce the unsound skip §2 forbids; a unit test pins that monotonicity
property. The out-of-crate-input audit (bead 2) treats a sibling read as covered
iff the reader is listed in a matching `readers` entry, and the map-validity test
extends to `readers` names. Residual 3 (`sparq-conformance`'s
`scoreboard_floors.rs` reading sibling **test sources** at a statically
unresolvable runtime path) was *not* coverable by `readers` — the union matches a
literal/glob pattern, and a path assembled at runtime collapses to the workspace
root — so it stayed acknowledged, backstopped by the nightly full run (§6.1),
pending its own design pass.

**Residual 3 closed by input-relocation** — sq-z1xv8 [SONNET-4.6]. The read
existed only because a floor enforced in one crate's runner had to reach
`sparq-conformance`'s central `scoreboard::SUITES` without a dep edge, so the
number was spelled twice and reconciled by reading the other crate's test source.
The eleven such floors (W3C SHACL core + SHACL-SPARQL, both OGC GeoSPARQL lanes,
Solid WAC/ACP decision parity + the two differential oracles, the SolidLab ODRL
suite, the sparq-text BM25 oracle, the sparq-rsp expressivity oracle) moved into a
new leaf crate, `sparq-conformance-floors`: zero dependencies, `publish = false`,
constants only. Each enforcing crate takes it as a **dev-dependency** (shipping
graph untouched) and `sparq-conformance` as a plain dependency, so both sides read
ONE compile-time `const` — the same cannot-drift shape the six JSON-LD lanes
already had via `sparq_conformance::floors` — and the guard reads no foreign source
at all. This is genuine relocation, not a coverage waiver: the dep edges put every
enforcing crate in the floors crate's reverse-dependency closure, so a floor change
cannot selection-skip the lane that enforces it. The surviving textual reads are
rooted at `CARGO_MANIFEST_DIR` and a unit test
(`textual_guard_reads_only_crate_local_sources`) fails if a row ever names a path
outside the crate, so the hole cannot silently reopen. `KNOWN_RESIDUALS` in
`scripts/ci_audit_inputs.py` is now **empty** — every out-of-crate input in the
workspace is covered by a trigger, a map attribution, a dep closure or a `readers`
union.

### 4.3 Fail-closed mechanics

- The selector traps **all** internal errors and emits `mode=full` with exit
  0 — selector bugs degrade to the status quo, never to a skip.
- Downstream job guards are written so an **empty/missing output means run**:
  the run-condition is `mode != 'selected' || crate ∈ affected`, and an unset
  `mode` (select job failed, output lost) satisfies the first disjunct.
- If the `select` job itself hard-fails, `ci-summary` goes **red** (§5.3) —
  an infrastructure failure blocks merge rather than silently running full or
  silently skipping.

## 5. CI wiring

### 5.1 The `select` pre-job

One cheap job (<1 min): checkout with enough history to resolve the
merge-base, run `scripts/ci_select.py` (stdlib-only Python — no compile step,
`cargo metadata` is already JSON; precedent: `coverage-gate.py`,
`check-readme-template.py`; §7 P1). Outputs:

- `mode` ∈ {`selected`, `full`} + a human `reason`
- `affected` — JSON array of workspace-member names (when `selected`)
- a step-summary table: changed files → owning crates → closure → skipped
  count, so every PR shows its selection reasoning to reviewers.

### 5.2 Consumption — which lanes are scoped

**Phase 1 scopes only the wide multipliers** (this is where the ~90-check
width lives); the singleton lanes stay always-run so the initial soundness
surface is minimal (§7 P7):

| Lane | Phase 1 | Mechanism |
|---|---|---|
| per-crate test jobs | **scoped** | job-level `if:` guard (below) |
| `feature-matrix.yml` ~50 opt-in legs | **scoped** | same guard, keyed on the leg's `-p` crate |
| benchmark jobs | **scoped** | same guard |
| workspace nextest archive / bulk partitions | **scoped by filterset** | if a job runs a cross-crate partition, don't skip the job — narrow it: `cargo nextest run -E 'package(a) + package(b) ...'` over the affected set. The archive/compile job runs whenever `affected ≠ ∅` |
| lint / fmt / clippy, docs-quality, readme-template | always-run (cheap) | — |
| coverage ratchet | always-run (single job; exception: skip only when `affected = ∅`, since coverage is a function of compiled code + tests) | — |
| CodeQL, supply-chain (deny/vet/SBOM) | always-run (security semantics are whole-repo) | — |
| fuzz smoke, wasm builds, perf-gate | always-run in phase 1; **phase 2** scopes them by mapping each to the crate closure it exercises (bead 6) | — |
| `select` itself, `ci-summary` | always-run | — |

The wiring bead reconciles this table against the real job topology in
`ci.yml` / `feature-matrix.yml` / the bench workflows — the design fixes the
*semantics* (guard expression, filterset narrowing, fail-closed defaults);
the implementation maps them onto the actual jobs.

**The guard** (job-level `if:` on a static matrix — §7 P5): a job skipped by
a job-level `if:` is decided server-side (no runner slot consumed), keeps its
check name, and reports conclusion `skipped`, which GitHub treats as
satisfying a required status check. Implementation traps to honor:

- Do **not** `fromJSON` a possibly-empty output inside `if:` (expression
  error ⇒ workflow failure). Use delimited string containment instead:
  `contains(needs.select.outputs.affected, format('"{0}"', matrix.crate))` —
  the quote delimiters prevent `sparq-core` matching inside
  `sparq-core-foo`.
- Full run-condition:
  `needs.select.outputs.mode != 'selected' || contains(...)` — empty output
  ⇒ run (§4.3).

> **⚠️ ERRATUM (bead sq-fmx4u.7, 2026-07-05, PR #1437). [HAIKU-4.5]**
>
> **The prescriptive design above is NOT implementable as written for the
> feature-matrix legs.** The design (and §7 P5) specify a *job-level* `if:`
> guard keyed on `matrix.crate` — e.g.
> `contains(needs.select.outputs.affected, format('"{0}"', matrix.crate))`
> evaluated in `jobs.<job_id>.if`. GitHub Actions does **not** make the
> `matrix` context available in `jobs.<job_id>.if`: per the Actions
> *contexts-availability* table, a job-level `if:` can reference only
> `github`, `needs`, `vars`, and `inputs` — **not `matrix`**. A `matrix.crate`
> reference there silently evaluates to the empty string, so under enforcement
> the guard degrades to `contains(affected, '""')`, which is always false and
> would skip **every** leg unconditionally — precisely the silent-unsound-skip
> class the skip invariant (§2) forbids. This is an infeasibility of the
> original design, **not** an optimization trade-off: the job-level
> `matrix.crate` guard cannot be built on GitHub Actions at all.
>
> **What shipped instead** (both fail-closed, both pinned by
> `scripts/tests/test_ci_select_wiring.py`, which additionally asserts that no
> job-level `if:` ever references `matrix.`):
>
> 1. **Feature-matrix legs → assembly-time leg filtering** in the unit-tested
>    `scripts/assemble-feature-matrix.py` (see
>    `.github/workflows/feature-matrix.yml`, the `select`/`setup` jobs). The
>    per-leg keep/drop decision is made in Python before the matrix is
>    materialized; an unassembled leg spawns **no** check-run, and the polling
>    `ci-summary` aggregator never waits on a check that does not exist
>    (requiredness flows through `ci-summary / gate`, not per-leg names —
>    verified against live branch protection by bead sq-fmx4u.4). Shadow /
>    full / missing selection outputs fail-close to the full leg set inside the
>    assembler.
> 2. **Heavy `crates/sparq-vectors` shards → a step-level bash guard** in the
>    test step (see `.github/workflows/ci.yml`, the shard test step). The *step*
>    context — unlike the job-level `if:` — **does** see `matrix`, so the guard
>    reads `${{ matrix.crate }}` and `exit 0`s with a skip notice (reporting
>    `success`, not conclusion `skipped`) when the shard's crate is absent from
>    the affected closure. Fail-closed: only `mode = 'selected'` with a
>    non-empty crate and a definite non-membership can skip; empty/missing
>    selection outputs fall through to a run.
>
> The prescriptive text and §7 P5 are retained above as the original decision
> record; this erratum is the correction. Where §5.2/§7 P5 say "job-level
> `if:` guard" for the feature-matrix legs, read "assembly-time filtering";
> the job-level `if:` guard remains accurate only for the per-crate lanes whose
> guard keys on `needs`-derived outputs, not on `matrix`.

### 5.3 `ci-summary` (the aggregator)

- `if: always()`; **fails** if the `select` job did not succeed; **fails** on
  any needed job with result `failure` or `cancelled`; treats `skipped` as
  satisfied **only when** `select` succeeded. Summary line reports
  "`selected` mode: N of M crate-jobs run, K skipped by selection".
- Branch protection / merge-queue requiredness continues to flow through
  `ci-summary` plus the individually-required checks. Because
  selection-skipped required checks report `skipped` (which branch protection
  counts as satisfied), the required-check *names* never go missing — this is
  why the static-matrix guard beats dynamic matrix generation, where an
  unspawned job leaves a required check "expected" forever and blocks merge.
**§5.3 graduation note (bead sq-fmx4u.4, verified 2026-07-03). [SONNET-4.6]**
Verified against the live ruleset (`gh api repos/sparq-org/sparq/rulesets/17688455`,
id 17688455, last updated 2026-07-02):

- `required_status_checks` contains **exactly one entry**:
  `{"context":"gate","integration_id":15368}` — `ci-summary / gate` is the
  sole required check. No individual per-crate matrix legs, no individual
  `opt-in <X>` legs appear in the required list.
- `merge_queue` uses `grouping_strategy: "ALLGREEN"` and
  `check_response_timeout_minutes: 60`; the queue blocks only on the required
  `gate` check, not on absent or skipped siblings.
- A selection-skipped job (`conclusion=skipped`) from a static-matrix `if:`
  guard **reports a complete check-run** (the `skipped` conclusion is present,
  never "expected but missing"). `ci_summary_gate.py` already treats `skipped`
  as non-failing when select succeeded — so the gate passes, the queue
  unblocks, nothing hangs.
- Feature-matrix legs filtered at assembly time (unassembled — no check-run
  spawned) are also safe: since no leg name is individually required, the merge
  queue never waits for them; the only thing it blocks on is `gate`, which the
  aggregator always produces.
- **Mechanism chosen: plain-skip. No shim is needed.** The skipped-but-green
  shim (guard moved inside the job as a first step, so the job always occupies
  a slot but exits early) would be required only if a per-crate leg name
  appeared in `required_status_checks` — it does not. That condition is the
  only way plain-skip could cause a merge-queue hang.
- **What sq-fmx4u.5 can safely flip**: set the repo variable `CI_SELECT_MODE`
  to the literal string `"enforce"`. No ruleset change is required; no
  individual branch-protection entries need editing. The merge queue will not
  hang; skipped jobs will satisfy the gate through `ci-summary`.
- Three properties make this safe and must not drift; they are pinned by
  `scripts/tests/test_ci_select_wiring.py` (`TestRequiredCheckAnchor`):
  (1) the `ci-summary / gate` job name is exactly `"gate"` (matches the
  ruleset's `context:"gate"`); (2) that job has no `if:` guard (always runs,
  so the merge queue always gets a response within the 60-minute timeout
  window); (3) `ci-summary.yml` triggers on `merge_group` (required for the
  gate to produce a check-run on queue entries at all).

## 6. Correctness safeguards

1. **Nightly scheduled FULL run** on `main` (`schedule` event ⇒ `mode=full`).
   Any nightly failure whose job was selection-skipped in the PRs that landed
   since the last green nightly is *prima facie* a selection bug: auto-file a
   P1 bead/issue with the offending job + suspect PRs.
2. **`ci-full` label**: applying it forces `mode=full`; the workflow reacts to
   `labeled`/`unlabeled` so toggling re-evaluates. One auditable override, no
   commit-message magic.
3. **`workflow_dispatch`** manual full run for ad-hoc verification.
4. **Shadow-mode rollout**: a flag under which the selector computes and
   *reports* the would-skip set while every job still runs. Enforce only
   after a shadow window (order of twenty PRs) shows **zero** cases where a
   would-have-been-skipped job failed for a reason attributable to the PR.
   The shadow report is the honest measurement of both soundness and savings.
5. **Selector self-tests** (bead 1 + bead 2): golden diffs → expected sets —
   leaf change ⇒ exactly that crate; `sparq-core` change ⇒ all members;
   dev-dep and optional-dep edges propagate; every §4.1 trigger class ⇒ full;
   unowned path ⇒ full; internal error ⇒ full; plus a test pinned against the
   real workspace metadata so graph-shape regressions surface in review.
6. **Transparency**: the per-PR step-summary (§5.1) makes every skip decision
   reviewable where reviewers already look.

**§6 graduation note (bead sq-fmx4u.5, ENFORCEMENT FLIPPED 2026-07-03). [FABLE-5]**
The shadow rollout is now flipped to **enforce by default** — a not-affected
crate's wide-lane tests/benchmarks are actually skipped. This is the culmination
of the epic and is proven-safe by bead sq-fmx4u.4: the live ruleset requires
exactly one check (`ci-summary / gate`), so a selection-skipped leg reports
`skipped` (= satisfied) and can never individually hang the merge queue. No
ruleset change was needed. What landed:

- **Enforce flip** (`.github/workflows/ci-select.yml`): `--shadow` is now added
  *only* when the repo variable `CI_SELECT_MODE` is the literal `"shadow"` (the
  report-only rollback escape hatch); any other value — including unset and
  `"enforce"` — enforces. The pre-flip `!= "enforce" ⇒ --shadow` default is gone.
- **`ci-full` label override** (safeguard 2): the select step computes
  `CI_FULL_LABEL = contains(github.event.pull_request.labels.*.name, 'ci-full')`
  and, when true, runs the selector with `--full` (mode=full, nothing skipped).
  `ci.yml` + `feature-matrix.yml` now trigger on `pull_request:` types
  `[opened, synchronize, reopened, labeled, unlabeled]`, so toggling the label
  re-evaluates selection.
- **Nightly full-matrix backstop** (safeguard 1): the existing `schedule` cron on
  `ci.yml` resolves to `mode=full` by construction (a non-PR event carries no
  diff), and a new `selection-backstop` job asserts that `mode=full` invariant
  fail-loud on `schedule`/`workflow_dispatch` (it REDs if a scheduled run were
  ever narrowed). `workflow_dispatch` (safeguard 3) is the ad-hoc full run.
- **Fail-safe preserved** (non-negotiable): `ci_select.py` is unchanged — every
  §4.1 trigger (shared crate / build file / `.github/**` / `Cargo.lock` /
  selector-self change) and any internal error still return `mode=full`, so
  enforce never skips a test for an affected crate or its reverse-dep closure.
- **Tests** (`scripts/tests/`): `EnforceRolloutTests` (affected⇒RUNS,
  not-affected⇒SKIPS via the exact shard-guard membership rule, ci-full⇒full,
  nightly⇒full, fail-safe trigger⇒full, selector-error⇒full, a mutation check on
  the quoted-needle guard) + wiring inspection (enforce-default, ci-full override,
  label-toggle triggers, the nightly backstop job).

**Shipped** (sq-va7at, PR #1526): the §6.1 *selection-bug alarm* now correlates
failed nightly jobs with suspect landed PRs and auto-files a deduplicated issue.

## 7. Firm positions (decision record)

- **P1 — selector is stdlib-only Python** (`scripts/ci_select.py`), not a
  Rust xtask and not a third-party action: zero build latency in the pre-job,
  JSON-native, unit-testable via `unittest`, covered by the `scripts/**`
  full-run trigger for self-changes. *Rejected:* `dorny/paths-filter`-style
  path lists (no dependency-closure semantics — unsound for a workspace);
  guppy/`determinator` (right semantics, prior art worth mirroring in the
  golden tests, but drags a Rust build into the pre-job and hides the rule
  set we need to be explicit and auditable).
- **P2 — package-level dependency edges, all kinds, features ignored**:
  superset of every feature-resolved graph ⇒ conservative (§3.3).
- **P3 — ownership covers all in-repo path packages**, longest-prefix match.
- **P4 — whole-crate granularity**; no sub-crate path refinement. Marginal
  extra savings, real soundness risk.
- **P5 — static matrix + job-level `if:` guards**, not dynamic matrix
  generation: skipped jobs cost no runner slot, required-check names survive
  as `skipped` (= satisfied), and the workflow diff is a guard per job rather
  than a rewrite. Dynamic matrices leave required checks "expected"/missing.
- **P6 — fail-closed at every layer** (§4.3): selector error ⇒ full; empty
  outputs ⇒ run; select hard-failure ⇒ red aggregator.
- **P7 — phase 1 scopes only the wide lanes**; singleton lanes (coverage,
  CodeQL, fuzz, wasm, perf-gate) stay always-run until phase 2 scopes them
  deliberately (bead 6). Minimizes the initial soundness surface where the
  throughput win is smallest.
- **P8 — selection applies to `merge_group` too**: the queue-entry diff vs
  the target tip is the union of queued content — conservative by
  construction, and queue width is where the throughput pain concentrates.
  *(sq-6vshe.18, 2026-07-31: the soundness argument is written out in §10,
  which also covers the batch-stacking case and closes the maintainer's
  stricter-rule option — `event == merge_group ⇒ mode = full` — as
  **KEEP-selected**.)*
- **P9 — the SAFE list starts empty** and only audit-proven entries join it.
- **P10 — enforce only after the shadow window** (§6.4); nightly full +
  `ci-full` label remain permanent backstops. *(sq-fmx4u.5, 2026-07-03:
  ENFORCEMENT FLIPPED — enforce is now the default; `CI_SELECT_MODE=shadow` is the
  report-only rollback escape hatch. See the §6 graduation note.)*

## 8. Implementation plan — child beads (disjoint)

| # | Bead | What | Tier |
|---|---|---|---|
| 1 | selector core | `scripts/ci_select.py` + unit tests: diff → ownership → reverse closure → `{mode, reason, affected}`; error ⇒ full | sonnet |
| 2 | fail-safe rules + audit | §4.1 trigger set + `ci/path-ownership.toml` + the repo-wide out-of-crate-input audit + per-rule tests | sonnet |
| 3 | matrix wiring | `select` job + `if:` guards / nextest filtersets across ci.yml, feature-matrix.yml, bench workflows + `ci-summary` semantics | sonnet |
| 4 | protection reconciliation | verify skipped-required-check semantics vs real branch protection + merge queue; shim only if needed | sonnet |
| 5 | backstops + rollout | nightly full run, `ci-full` label, `workflow_dispatch`, shadow mode + enforcement flip | sonnet |
| 6 | phase-2 scoping | fuzz/wasm/perf-gate closures; `affected = ∅` coverage skip | sonnet |

Dependency order: 1 → 2 → 3 → {4, 5} → 6. Beads carry their own acceptance
tests; see the bead records under `sq-fmx4u`.

## 9. Graduation

Once enforced, fold the operative rules (§2, §4.1, the scoped-lane table)
into the AGENTS.md gate documentation, keep the ownership map + selector as
the living source of truth, and rewrite this record's "will" into "does" — or
delete it in favor of the CI docs, per the research-record graduation rule.

## 10. Appendix A — merge-group selection soundness (the §7 P8 memo)

<!-- [OPUS-5] bead sq-6vshe.18 / issue #3073 — LEVER 4b of
     research/ci-mergequeue-speedup-2026-07.md §3.4b. Codifies the union-diff /
     batch-stacking soundness argument that previously lived only in a bead note, and
     closes the open maintainer decision on the stricter `event==merge_group ⇒ full`
     rule. INVARIANT: docs-only — no selector behaviour changes in this section; any
     behaviour change goes through its own PR. -->

**Bead sq-6vshe.18, 2026-07-31.** Selection has run on `merge_group` since the
enforcement flip (§6 graduation note, §7 P8) and demonstrably skips there — run
`29105286547` had **every** test shard selection-skipped under a successful `select`,
leaving a coverage shard as the entry's critical path (`ci-mergequeue-speedup-2026-07.md`
§2.2). The argument that makes those skips *sound* was, until this section, a bead note.
This appendix states it, states its premises, states what would falsify it, and records
the maintainer decision it was blocking.

### 10.1 What the selector actually sees on `merge_group`

`.github/workflows/ci-select.yml` binds `BASE`/`HEAD` from the event payload
(`github.event.pull_request.base.sha || github.event.merge_group.base_sha`, and the
matching `head_sha`) and passes them to the selector as `--base`/`--head` for both the
`pull_request` and `merge_group` cases; `scripts/ci_select.py` then runs
`git diff --name-only --no-renames <base>...<head>` (`git_changed_paths`). The workflow
checks out at `fetch-depth: 0` so the merge-base is always resolvable, and an
unfetchable base fail-closes to `mode=full` regardless.

`head_sha` is the tip of the queue entry's ephemeral
`gh-readonly-queue/…` branch, which contains **this entry's change stacked on top of
every entry ahead of it in the group**. That much is not in doubt.

`base_sha` admits two readings, and this record does not claim to have settled which
GitHub uses for a stacked entry:

* **(U) union reading** — `base_sha` is the *target branch tip* the group was formed on.
  The three-dot diff is then the union of entries `1..N` — §3.1's original wording.
* **(S) stacked reading** — `base_sha` is the head of entry `N-1`'s queue branch. Since
  `head` descends from `base`, three-dot degenerates to two-dot and the diff is
  **entry N's own change only** — strictly *smaller* than the union.

Reading (U) is trivially sound: a superset of the real change set feeds a monotone
closure (§3.3, P2), so the selected job set is a superset of the necessary one. The
non-obvious case — and the one the maintainer flagged — is (S). §10.2 shows the
selection is sound there too, so the decision does not hang on resolving (U) vs (S).

### 10.2 The batch-stacking argument (soundness under reading (S))

Let the group hold entries `1..K` in queue order, and let `T_i` denote the tree at
entry `i`'s head (i.e. changes `1..i` applied). Under ALLGREEN every entry runs its own
checks on its own `T_i` (§10.3, premise A). Write `Δ_i` for entry `i`'s own change set
and `cl(Δ)` for the reverse-dependency closure of `Δ` (§3.3).

Under reading (S), entry `i`'s run selects `cl(Δ_i)` and executes those crates' tests
**against `T_i`** — not against `Δ_i` in isolation. That pairing is what carries the
argument:

> **Claim.** For every workspace crate `C` and every entry `i ≤ K`, `C`'s test outcome
> on `T_i` is *identical* to its outcome on `T_j`, where `j ≤ i` is the last entry at
> which `C` was selected (`j = 0`, the already-gated target tip, if `C` was never
> selected) — and that run at `T_j` actually happened.

Note what the claim deliberately does **not** assume: that `C` was run at `T_{i-1}`.
Under selection it usually was not, which is exactly why the induction has to be over
*outcomes*, not over observed runs.

*Proof sketch, by induction on `i`.* Base `i = 0`: `T_0` is the target branch tip — a
previously merged, gate-passed state — and `j = 0` trivially. Step: assume the claim for
`i-1`. Two cases for entry `i`.

* `C ∈ cl(Δ_i)`. Then entry `i` selected `C` and ran it **on `T_i`** (that pairing is the
  whole point: the ephemeral queue branch already carries `Δ_1..Δ_i`). So `j = i` and the
  claim holds directly.
* `C ∉ cl(Δ_i)`. The closure is taken over package-level edges of *all* kinds with
  features ignored, hence a superset of any real influence path (§3.3, §7 P2); so by the
  skip invariant's contrapositive (§2), `Δ_i` cannot change `C`'s outcome, i.e.
  outcome(`C`, `T_i`) = outcome(`C`, `T_{i-1}`). Apply the induction hypothesis. ∎

Since `T_0` is green and the claim pairs every crate's outcome with a run that actually
happened at a tree containing every change that could have moved it, no red can hide in
the batch. Two corollaries are worth stating explicitly, because they are what the "two
individually green PRs that are red together" worry is really about:

* **Cross-entry interaction is covered.** A bug needing both `Δ_a` and `Δ_b` (`a < b`) to
  manifest fails in some crate `C` whose outcome depends on both, so
  `C ∈ cl(Δ_a) ∩ cl(Δ_b)`. Entry `b`'s run selects `C` and runs it on `T_b`, which
  contains both changes. The *later* entry is the one that catches it — which is why
  queue order does not matter.
* **The merged result is covered.** `main` after the batch has the content of `T_K`. A
  crate last run at `T_i` for `i < K` is, by the claim, one whose outcome `Δ_{i+1..K}`
  cannot change, so its `T_i` verdict is still its `T_K` verdict. Squash-merging mints a
  new SHA (cf. issue #1897, which is a *lever-1* concern about which SHA carries the
  green verdict) but does not change the *tree content*, and this argument is about
  trees — so the squash does not weaken it.

Note that (S) is the *tighter* selection and (U) the looser one; the argument above
covers (S), and (U) selects a superset of it. Soundness therefore holds under either
reading, which is why resolving the payload semantics is a nice-to-have, not a blocker.

### 10.3 Load-bearing premises (what must not drift)

The argument is not unconditional. It rests on exactly four premises:

| # | Premise | Where it is established | Pinned by |
|---|---|---|---|
| A | `grouping_strategy: ALLGREEN` — **every** entry runs its own required checks on its own `T_i` | live ruleset `17688455`, verified 2026-07-03 (§5.3 graduation note); `docs/branch-protection.md` *Merge-queue throughput settings* | **nothing in-repo** — see the residual below |
| B | `ci-summary / gate` is the **sole** required context, so a `skipped` leg satisfies protection and no per-leg name can hang the queue | §5.3 graduation note (same verification) | `scripts/tests/test_ci_select_wiring.py::TestRequiredCheckAnchor` (gate job named `gate`, no `if:` guard, `ci-summary.yml` triggers on `merge_group`) |
| C | The closure is a **superset** of real influence — all dependency kinds, features ignored, whole-crate granularity | §3.3, §7 P2/P4 | the selector's golden tests (§6.5) |
| D | Every ambiguity **fails closed to `mode=full`** — unmapped path, unresolvable base, any internal error | §4.1, §4.3, §7 P6 | `ci_select.py` fail-safe boundary + its per-rule tests |

**What would falsify the argument.** Flipping the queue to `HEADGREEN` breaks premise A
outright: only the head entry's checks would run, so only `cl(Δ_K)` would be exercised
and `cl(Δ_1..K-1)` would go untested on the merged tree — the induction loses every step
but the last. Adding an individual per-crate leg to `required_status_checks` breaks
premise B (a selection-skipped leg would then be a required check that can go missing).
Sub-crate path refinement (rejected in P4) would put premise C at risk.

**Residual (unpinned).** Premise A is a *remote ruleset* setting. Unlike premise B, no
in-repo test can observe it, so a maintainer flipping `ALLGREEN → HEADGREEN` would
silently invalidate this argument with no gate going red. The `gh api …/rulesets/<id>`
recipe in `docs/branch-protection.md` is the manual check; mechanising it is captured as
follow-up work, not done here (docs-only bead).

### 10.4 Decision: the stricter `event == merge_group ⇒ mode = full` rule

**The option.** A one-line change in `ci_select.py` — drop `merge_group` from the
`args.event not in ("pull_request", "merge_group")` guard — would make every queue entry
run the full matrix. It is the maximally conservative posture and needs no argument at
all to justify.

**The cost.** It would reverse most of lever 4. The 2026-07-10 profile
(`ci-mergequeue-speedup-2026-07.md` §2.1–2.2 — the tables there and the run logs they
cite are the source of truth for every figure below, which is why none is restated
here) records two distinct entry shapes: `ci.yml` dominates the median entry wall, an
entry whose test shards are selection-skipped lands near the low end of that
distribution with a coverage shard as its pole (run `29105286547`), and an
engine-touching full-matrix entry sits near the top on the serial
`build+archive → slowest shard` chain (run `29105265898`). Forcing full would put
*every* entry on the second, materially slower shape, and would restore the full opt-in
`feature-matrix` leg set in place of the much smaller selected engine leg set.
**Projected effect: the median entry wall reverts to the full-matrix shape**, plus the
runner-slot contention that lengthens tails during a drain. That is a qualitative
projection from the cited profile, **not** a measured post-change result — the honest
way to obtain a real number is to flip it and sample merge-group entry walls, which is
precisely the experiment the projection says is not worth running.

**Recommendation: KEEP selection on `merge_group` (do not adopt the stricter rule).**
Rationale, in order of weight:

1. §10.2 shows the skips are sound under *both* readings of the payload, given premises
   A–D — so the stricter rule buys defence against premise *drift*, not against a known
   hole.
2. That drift is already fenced twice over. The **nightly full-matrix backstop** (§6.1)
   re-runs everything on `main` and would surface any batch-induced red within a day,
   and the **sq-va7at selection alarm** (`scripts/ci_selection_alarm.py`, shipped in
   PR #1526) correlates a nightly failure with the PRs that landed since the last green
   nightly and auto-files a deduplicated issue naming the suspects.
3. The failure being defended against is *recoverable*: a red on `main` is visible,
   attributable, and revertible — unlike a silently-lowered gate.
4. `ci-full` (§6.2) remains the per-change escape hatch, and `CI_SELECT_MODE=shadow`
   remains the global one, so adopting the stricter rule permanently is not the only way
   to get a conservative run when one is wanted.

Authored under the **proceed-and-document** rule: this closes the decision as *keep-
selected* so the rest of lever 4 is unblocked. It is a recommendation with its premises
and its falsifiers written down, not a maintainer sign-off — a maintainer who weighs
premise A's unpinned status more heavily than the projected slowdown can overturn it with
the one-line change above, and §10.3 says exactly what to re-check if they do.

### 10.5 Explicit non-extensions (REJECTED — recorded so they are not re-proposed)

Lever 4 is **not** being extended to the following. Each was considered and rejected on
measured grounds; the shared shape is *small win, real risk*.

| Candidate | Position in the profile | Verdict |
|---|---|---|
| conformance ratchets | short legs, run in parallel | **REJECTED** — never the pole; scoping them adds selector surface for no wall-clock |
| `container-scan` | parallel with the critical path | **REJECTED** — parallel, not the pole |
| CodeQL language-scoping | confirmed non-pole (§2.1) | **REJECTED** — it is the *security* gate and the `code_scanning` ruleset expects analyses to be produced; narrowing the analysed language set risks a missing-analysis stall for a saving the profile shows as negligible |

(Positions from `ci-mergequeue-speedup-2026-07.md` §2.1–2.2, 2026-07-10 profile; the
per-leg timings stay in that record's tables and in the run logs it links, not here.)
