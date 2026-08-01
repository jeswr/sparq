# CI structural speedup — the levers beyond selection, per-job caching, and coverage sharding (sq-6vshe)

**Status:** design record (provisional — operative rules graduate into the AGENTS.md gate
docs as the child beads land). Authored under the proceed-and-document rule.
**Author:** Claude Fable 5 (SPARQ architect tier), 2026-07-03. [FABLE-5]
**Implementation:** decomposed into disjoint child beads under `sq-6vshe` (§9). No
implementation in this PR.

---

## 1. Scope — what this record deliberately does NOT cover

Three tactical fixes are already in flight. This record designs around them, never
re-proposes them, and each child bead states its non-overlap explicitly:

| In flight | What it covers | What it structurally CANNOT reach |
|---|---|---|
| **sq-fmx4u** change-based selection (`research/change-based-test-selection.md`) | skip a crate's jobs when neither it nor its reverse closure changed; phase 2 (`sq-fmx4u.6`) change-scopes fuzz/wasm/perf-gate | the PR that **changes `sparq-engine`**: everything depends on engine, so the affected set is ~the whole matrix and selection skips ~nothing |
| **Per-job build caching** (rust-cache/sccache wiring, Opus, in progress) | restore engine + deps instead of recompiling in each job | the **changed crate itself** always recompiles; the **first** job of a PR on a cold/thrashed key; cache-key/backend/priming policy |
| **sq-piapk** coverage engine-shard | shrink the ~668s `sparq-engine` coverage shard via cross-runner profraw merge / shared instrumented build | the reason the shard is ~668s in the first place (one monolithic crate), and every *non-coverage* leg |

Observed shape as of 2026-07 (dated observations, not gates): ~40-crate workspace, ~90
required checks per PR, ~50 opt-in feature legs, coverage ratchet 4-way-sharded ≈17 min
floored by the engine shard. Maintainer target: **well under 17 min wall-clock**,
including the worst case.

## 2. The residual cost model

Assume the three in-flight fixes land and work. What still costs?

1. **The engine-touching PR** — plausibly the modal heavy PR (engine is the largest crate
   and where the SPARQL work happens; `sq-6vshe.3` measures churn instead of assuming).
   Selection skips ~nothing (mid-graph ⇒ near-full reverse closure), caching cannot help
   the changed crate, and all ~50 feature legs + the engine coverage shard + every
   downstream test run. **Nothing in flight helps this PR.** It is the design center of
   this record.
2. **Matrix width as defined**, not as selected: the ~50 feature legs exist per-feature by
   construction. fmx4u decides *which* legs run; nothing questions *what the leg set is*.
3. **Per-leg constant tax** × ~90 checks: queue, checkout, toolchain, cache restore/save —
   paid even by trivial jobs.
4. **Every compile-bearing leg's flag-level waste**: full debuginfo, incremental bookkeeping,
   the default linker — multiplied across the whole matrix.
5. **Cold first jobs**: per-job caching makes the *n*-th job warm; nothing primes the first.
6. **Always-run poles** (fuzz, CodeQL, benchmarks): fmx4u.6 change-scopes them, but when
   they *do* run they still run their full per-PR form, and CodeQL is deliberately
   always-run there ("security semantics are whole-repo").

Levers 1–6 map onto beads `sq-6vshe.{3,4} / .2 / .7 / .1 / .5 / .6` respectively.

## 3. Q1 — is `sparq-engine` too monolithic? Position: **yes, split it — but measure first, cut second**

### 3.1 The structural argument

A single large mid-graph crate is the worst case for every mechanism this program and the
in-flight work rely on, simultaneously:

- **Caching:** any engine change invalidates engine + its whole downstream, in every job.
  Cache granularity is crate granularity; one big crate = one big invalidation unit.
- **Selection:** fmx4u's affected-set is computed at crate granularity; engine's reverse
  closure is ~the workspace, so engine PRs get no benefit. Splitting engine into sub-crates
  makes the *sub-crate* the selection unit — an exec-only change stops implying
  storage-glue and serialization test runs, and downstream crates whose narrowed deps
  exclude the changed seam drop out of the affected set entirely.
- **Compile parallelism / pipelining:** cargo parallelizes *across* crates; within a crate
  the frontend is essentially serial. Engine is a long serial pole on every critical path
  (core→parse/substrate→**engine**→server→solid/…). A split into parallel seams turns the
  pole into a diamond — but only if the seams are siblings, not a chain (§3.3).
- **Coverage:** the ratchet is per-crate; one crate = one indivisible ~668s instrumented
  test binary. sq-piapk attacks this *tactically* (shard one crate across runners);
  decomposition removes it *structurally* — 3–5 crates shard trivially on the existing
  4-way matrix. If the split lands, piapk's machinery likely becomes unnecessary
  (coordinate; piapk is P3 and explicitly says "profile engine's compile-vs-test split
  first" — `sq-6vshe.3` **is** that profile, generalized).
- **Dev/agent iteration:** every incremental rebuild after touching one engine module pays
  the whole-crate frontend. With a large agent fleet compiling all day, this recurring cost
  plausibly rivals the CI bill.

### 3.2 The honest counter-arguments (these are why Phase 0 exists)

- **Generics can gut the win.** If engine is monomorphization-heavy, most *codegen* already
  happens downstream at instantiation sites; splitting moves frontend time but less codegen
  than hoped. `cargo llvm-lines` + `--timings` frontend/codegen attribution answers this
  empirically — it can legitimately return NO-GO.
- **Cross-crate inlining on hot eval paths.** The reasoner program's greenlit doctrine is a
  *zero-overhead* shared eval substrate with no perf regression. Seams crossing hot paths
  need `#[inline]`/thin-LTO and must pass the existing perf-neutrality gate — that gate is
  the proof obligation, not a hope.
- **Feature forwarding.** The opt-in feature surface must keep its names: `sparq-engine`
  stays as a facade (`pub use` re-exports; features forward to sub-crate features), so
  external users and the feature-matrix keys see a compatible surface.
- **Crate-count overhead.** 3–5 new crates = new coverage floors, README-template gates,
  per-crate legs, and a wider bookkeeping surface. Bounded and one-time, but real — which
  is why the answer is 3–5 seams, **not** 15 micro-crates.
- **Churn.** Extraction PRs conflict with in-flight engine work; stage one seam per PR,
  least-entangled first, with short coordinated freeze windows.

### 3.3 Seam hypotheses (to be validated, not presumed)

Candidate seams, chosen for *sibling-ness* (parallel compilation) and narrow interfaces:
algebra/planning · physical exec/eval · update+storage glue · results/serialization · the
opt-in feature surfaces still living inside engine as gated modules. Phase 0 replaces this
list with the measured module graph + interface-width table.

### 3.4 Decision rule (§3.5 referenced by the beads)

Proceed from Phase 0 (`sq-6vshe.3`) to execution (`sq-6vshe.4`) **iff**:
(i) ≥2 seams each carry a substantial share of engine's frontend time behind a narrow
interface (few cross-seam `pub` items, no orphan-rule tangles); and
(ii) the predicted critical-path reduction for the modal engine-touching PR is material
(order 25%+, from the timings model). Otherwise **NO-GO**: record the negative result and
stop the program at the cheap levers — an honest no-split verdict is a valid outcome.

**This is the one genuinely big architectural call in this record — maintainer decision
required** (proceed-and-document applies: Phase 0 proceeds now, the RFC + steer issue give
the maintainer the real decision point before any cut).

## 4. Q2 — the ~50-leg feature matrix is over-specified. Position: **pyramid it**

### 4.1 What the matrix actually guarantees — a class analysis

| Class | Failure mode | What catches it | Needs per-feature build+test leg? |
|---|---|---|---|
| A | incomplete feature gating: code under feature X references an item that needs undeclared feature Y; compiles today only via unification | **per-feature `cargo check`** (compile error in isolation) | **No — check suffices**, no codegen/link/test needed |
| B | feature-*sensitive runtime behavior*: tests assert behavior in a specific feature state, incl. `cfg(not(feature))` asserts (the class that already bit under `--workspace` feature unification — see AGENTS'/memory CI-debugging notes) | a **test run in that exact feature state** | **Yes — but only for features that HAVE such tests** |
| C | non-additivity / unification breakage in the maximal direction | the **all-features** test leg + the workspace bulk lanes | No per-feature leg |
| D | link/codegen-only per-feature failures (build-script quirks, symbol issues) — rare | a **full each-feature build+test**, but it does not need to be per-PR | Nightly backstop suffices |

### 4.2 The pyramid (bead `sq-6vshe.2`)

- **T1 (per-PR, cheap):** one job (sharded 2–4× if needed) running
  `cargo hack check --each-feature`-style over the opt-in set with a **shared target dir**
  — the dependency stack below each feature-gated crate builds once instead of ~50 times,
  and `check` skips codegen/link/test entirely. Covers class A for *every* feature.
- **T2 (per-PR, curated):** full build+test legs **only** for features with
  feature-sensitive tests, enumerated by a small checked-in detector (grep for
  `cfg(feature=…)` / `cfg(not(feature=…))` in test code), whose output list is
  human-reviewed and **ratcheted** (a feature leaves T2 only when its sensitive tests do).
  Covers class B exactly where it exists. Expected minority of the ~50.
- **T3 (per-PR, existing):** all-features + no-default-features + default test legs (class C).
- **Backstop (nightly):** full each-feature **build+test**, folded into the fmx4u nightly
  full run (class D + detector-false-negative insurance). A nightly failure auto-files a
  P1 bead per the fmx4u §6.1 protocol.

Net: worst-case PR width drops from ~50 build+test legs to ~a-dozen T2/T3 legs plus one
cheap check job. This composes with fmx4u (remaining legs still carry selection guards) and
multiplies with §3 (each leg also gets cheaper if engine splits).

**Rejected:** pairwise/powerset combination covers — ~50 features ⇒ combinatorial legs for
interactions that Cargo doctrine already requires to be additive; class C is the practical
non-additivity net, and T1+T3+nightly bound the residual risk at far lower cost. Also
rejected: dropping per-feature verification entirely (loses class A per-feature locality,
which is the matrix's real daily catch).

Implementation note: some "feature legs" are opt-in *crates* already covered by per-crate
test lanes — the implementation dedupes those against the real `feature-matrix.yml`
topology (this record fixes semantics, not YAML).

## 5. Q3 — build-profile + execution flags. Position: **do the boring trio now; sequence before cache priming**

Per-PR CI builds are clean-ish, non-interactive, and throwaway. Flags tuned for interactive
dev are pure waste there:

- **`CARGO_INCREMENTAL=0`** — incremental adds bookkeeping to builds that never increment,
  and bloats every cache. Safe unconditionally (verify the cache action doesn't already set it).
- **`debug = "line-tables-only"`** for CI dev/test profiles — debuginfo emission and the
  resulting link/artifact sizes are a large share of debug-build cost across ~40 crates ×
  test bins × ~50 legs; line-tables keep `file:line` in backtraces (the part CI failures
  actually use) while dropping variable/type info. Set via CI env
  (`CARGO_PROFILE_DEV_DEBUG=…`), **not** repo `.cargo/`or the root manifest — those are
  fmx4u full-run triggers and would also change local dev.
- **Fast linker** (mold; lld as the boring fallback) via CI-env RUSTFLAGS — the matrix
  links hundreds of test binaries per full run; the default linker is the slowest part of
  many warm-cache legs.
- **Leave `codegen-units` alone** (dev default is already parallel); **defer
  `profile.test` opt-level=1** experiments until the §8 inventory shows runtime-dominated
  legs (compile-cost increase is guaranteed, runtime win is not).

**Sequencing rule:** flag changes rekey every compiler-output cache once. Land `sq-6vshe.1`
**before** the cache-priming topology (§6) stabilizes, so caches are primed against the
final flag set — and record the chosen env in the key schema.

## 6. Q4 — cache topology beyond per-job wiring. Position: **prime on main, key on {rustc, target, family, lockfile}, PR-read-only**

The in-flight work makes individual jobs *able* to restore. Three policy gaps remain, and
they are policy, not wiring (bead `sq-6vshe.5`):

1. **Priming.** On push to `main` (+ nightly, catching toolchain bumps), a prime job builds
   the canonical **feature families** — workspace default, all-features, wasm target, the
   T1 check family — and saves. PR jobs restore from the base branch, so the **first** job
   of a fresh PR is warm. Without this, every PR's first wave rebuilds the world once per
   family.
2. **Key schema.** Primary key `{rustc version, target triple, feature-family, Cargo.lock
   hash}`; prefix fallback may drop **only** the lockfile component; **never** fall back
   across rustc or family (cross-family hits are how caches thrash at the 10GB GHA cap;
   cross-rustc hits are how they rot). Staleness cannot mask failures by construction:
   sccache keys on the full compiler invocation hash (stale ⇒ miss, never wrong output),
   and the fmx4u nightly full run stays **cache-cold** as the from-scratch buildability
   proof.
3. **Poisoning discipline (public repo).** PR-context jobs — fork PRs especially — get
   **read-only** access to any shared backend; writes only from `main`/`schedule` contexts.
   GHA cache gives this natively via branch scoping. An S3 backend must reproduce it via
   role separation using the repo's established **GitHub-OIDC → scoped IAM role** pattern
   (`research/ci-ec2-design.md` hard rules: no long-lived keys, never trust
   `pull_request` triggers with credentials).
4. **Backend sizing.** ~90 jobs × several families vs the 10GB GHA repo cap is a thrash
   audit waiting to fail; if it does, sccache+S3 with a ~30-day lifecycle expiry costs
   pennies against the runner-minutes it saves. Bucket + role are an infra change:
   proceed-and-document with a steer issue.

## 7. Q5 — always-run heavy lanes. Position: **per-PR keeps only the deterministic slice**

fmx4u.6 already decides *whether* an unaffected PR runs these lanes. The structural
question is what the per-PR variant should *be* when it runs, and where the full variant
lives (bead `sq-6vshe.6`):

| Lane | Per-PR (proposed) | Full form | Why the net doesn't weaken |
|---|---|---|---|
| cargo-fuzz | build targets + **deterministic corpus-replay** of the committed corpus/known crashers (seconds) | nightly/continuous randomized fuzzing with corpus persistence | replay deterministically catches reintroductions; a few minutes of per-PR *random* fuzzing has near-zero marginal detection probability — it is pole, not net |
| benchmarks | compile + **single-iteration smoke** (criterion `--test` mode); `bench-full` label opts back in | `main`-push series (G2) + the EC2 lanes (`research/ci-ec2-design.md`) unchanged | per-PR numbers on shared runners are noise-prone and non-canonical anyway; the smoke keeps "it compiles and runs" |
| CodeQL | **measure first**; if a post-piapk wall-time pole: full analysis at `merge_group` + nightly, nothing (or a light pass) per-push | merge-queue run blocks every merge | the repo merges through the queue, so alerts-at-zero still gates every merge; per-push analysis is redundant *for protection* and only buys earlier signal — **maintainer security-posture sign-off required** |
| wasm | — (stays entirely under fmx4u.6 change-scoping; nothing added here) | — | — |

**Demotion protocol (load-bearing):** any lane demoted off per-push that fails on
nightly/merge-queue **auto-files a P1 bead** naming the job + suspect PRs — mirroring the
fmx4u §6.1 nightly-failure protocol — so demoted lanes cannot silently rot.

## 8. Test-execution topology (non-coverage)

The steering-data bead (`sq-6vshe.7`): a per-leg wall-time inventory decomposed into
constant overhead vs compile vs run (this table arbitrates several deferred choices above);
rebalance the existing nextest bulk/HEAVY partitions against measured runtimes (leg names
lie — "bulk N/M" runs `not(HEAVY)`); extend the build-once **nextest archive** pattern
(compile once, run partitioned across runners) to the slowest non-coverage legs; audit the
workspace **doctest** bill (each doctest compiles + links as its own crate; nextest doesn't
run doctests, so a `cargo test --doc` lane must survive any consolidation); and fold tiny
always-run legs into fewer, fatter jobs where the per-job constant tax dominates their
runtime. Explicitly disjoint from sq-piapk (coverage-instrumented topology is piapk's).

**Child record:** `research/ci-leg-walltime-inventory.md` (`sq-6vshe.7`) — ships the
steering-data instrument (`scripts/ci_leg_walltime_inventory.py`) and corrects three of
this section's five premises against the tree as it now stands: the build-once archive and
the run-once doctest lane already exist in `ci.yml`, and the fold-into-fatter-jobs
mechanism already exists in the feature-matrix bin-packer. It also finds that extending
the archive to the conformance legs is blocked (different cargo profile, the `sq-vya1`
archive-feature guard, non-test targets) and redirects that lever to bin-packing.

## 9. Program — beads, ordered by payoff-per-risk

| # | Bead | What (one line) | Payoff | Risk | Tier |
|---|---|---|---|---|---|
| 1 | `sq-6vshe.1` (P1) | CI build-profile trio: line-tables-only debuginfo, incremental off, fast linker | multiplicative across every compile-bearing leg | low | sonnet |
| 2 | `sq-6vshe.2` (P1) | feature-matrix pyramid: T1 check-tier + curated T2 test legs + T3 + nightly backstop | worst-case width ~50 legs → ~a dozen + one check job | medium (guarantee argument; backstopped) | sonnet + opus review |
| 3 | `sq-6vshe.3` (P1) | engine split Phase 0: measured seam map + RFC + go/no-go (§3.4) | none direct; de-risks the biggest lever | none (read-only) | opus |
| 4 | `sq-6vshe.5` (P2) | cache topology: prime-on-main, key schema, PR-read-only, backend sizing | first job of every PR warm; no thrash | low-medium | sonnet |
| 5 | `sq-6vshe.6` (P2) | heavy-lane placement: fuzz replay, bench smoke, CodeQL placement, demotion auto-bead | removes multi-minute always-run poles | medium (detection-delay window; backstopped) | sonnet |
| 6 | `sq-6vshe.7` (P2) | non-coverage test-exec topology: inventory, partition rebalance, archives, doctest audit | slowest legs + per-leg constant tax | low-medium | sonnet |
| 7 | `sq-6vshe.4` (P2, **gated** on .3 + maintainer) | execute the engine split behind a facade, one seam per PR | the big structural one — the engine-touching PR itself | high (API churn, perf, conflicts) | opus, staged |

Dependency structure: `.1 → (informs .5 keys)`; `.3 → .4`; `.7` informs `.1`'s deferred
opt-level experiment; everything else independent — the fleet can run `.1/.2/.3` in
parallel now.

## 10. Maintainer decisions flagged

1. **The engine split** (`sq-6vshe.4`) — the one genuinely big architectural call; Phase 0
   RFC + steer issue is the decision package.
2. **CodeQL placement** (`sq-6vshe.6c`) — security-posture change even though merge-gating
   is preserved.
3. **sccache S3 backend** (`sq-6vshe.5`) — new bucket + OIDC role (established pattern,
   small spend); proceed-and-document.

## 11. Graduation

As beads land: fold the operative rules (the pyramid's tier definitions + detector ratchet,
the cache key schema + poisoning rule, the heavy-lane placement table + demotion protocol)
into the AGENTS.md gate documentation and the CI docs; rewrite this record's "would" into
"does" or delete sections in favor of the living docs, per the research-record graduation
rule. A NO-GO from Phase 0 gets recorded here as a dated negative result.
