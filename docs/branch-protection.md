<!-- [OPUS-4.8] Governance: branch-protection doc-of-record (bead sq-41ey). -->
# Branch protection — `main`

This is the **doc-of-record** for the branch-protection ruleset on `main`. The settings
themselves are configured **out-of-repo** by the repository owner under
**Settings → Branches → Branch protection rules** (or a repository ruleset) on GitHub;
they cannot be expressed in a tracked file. This document records what those settings
should be, so the intended protection is reviewable and reproducible if the rule is ever
recreated.

## Protected branch

- **`main`** — the only long-lived branch. All changes land via pull request; direct
  pushes are disallowed (including for administrators — see "Other settings").

## Required status checks

There is exactly **ONE** required status check — the aggregator. The live ruleset's
`required_status_checks` rule lists a single context (`gate`, from the `ci-summary`
workflow) and sets `strict_required_status_checks_policy: false` — i.e. PRs are **not**
forced to be re-based up-to-date with `main` before merging. This is consistent with the
solo-maintainer reality (a single serialized merge train: branches are gated and merged
one at a time per `AGENTS.md`, so a strict up-to-date requirement would only add churn
without a second concurrent author to race against). The required check is:

| Required check (job name) | Workflow | What it gates |
|---|---|---|
| **`ci-summary / gate`** | [`.github/workflows/ci-summary.yml`](../.github/workflows/ci-summary.yml) | **The single gate.** Polls Actions workflow-runs plus external check-runs on the PR head commit and passes iff the newest run/attempt of every gating workflow succeeds (`success`/`skipped`/`neutral` are non-failing). |

> **Select only `ci-summary / gate`** in the ruleset's "Require status checks that must
> pass" list. Do **not** add the individual job names below — `ci-summary` already
> aggregates them. This is deliberate: `needs:` cannot span workflows, so requiring each
> job by name was brittle (every rename / added gate broke the rule and silently weakened
> the gate). The aggregator adapts automatically — add or rename jobs freely and the gate
> still covers them, because it discovers the live workflow/check set at run time. See the
> header of `ci-summary.yml` for the full semantics (newest-run resolution, bounded
> cancelled-run re-dispatch, stability window, and self-exclusion).

### What `ci-summary` aggregates (informational — do NOT add these individually)

The gate covers **every newest Actions workflow run** and every external check-run on the
head commit. As of this writing those expose the jobs below; this table is a map for
reviewers, **not** a list of required checks.

> **Advisory/informational checks are non-gating by NAME (sq-wjth).** `ci-summary`
> excludes any check whose name contains the word `advisory` or `informational`
> (case-insensitive) from the gating set — its conclusion, even `failure`, never blocks
> a merge. So a job that should GATE must **not** put either word in its display name
> (e.g. the clippy gate is named `clippy (gate) + fmt (non-blocking)`, *not*
> `… (informational)`), and a new advisory/visibility-only job **should** carry one of
> those words so the aggregator treats it as non-gating automatically. Note "advisories"
> (plural, as in `cargo-deny (advisories + …)`) does **not** match `advisory`, so that
> supply-chain check still gates correctly.

From the **CI** workflow (`.github/workflows/ci.yml`):

| Job name | What it gates |
|---|---|
| `build + test (workspace)` | `cargo build --workspace --all-targets` + `cargo test --workspace`. |
| `clippy (gate) + fmt (non-blocking)` | `cargo clippy --workspace --all-targets -- -D warnings` (the clippy gate; fmt is non-blocking until the one-time reformat lands). |
| `MSRV check (Rust 1.88, declared floor)` | `cargo check` on the pinned MSRV toolchain. |
| `W3C SPARQL conformance (ratchet >= 1229 pass+divergence)` | The W3C SPARQL conformance ratchet (never lower). |
| `W3C SHACL conformance (ratchet — core >= 98, sparql >= 5)` | The W3C SHACL core + SHACL-SPARQL ratchets. |
| `Inference conformance (ratchet >= 1967 pass+divergence)` | The RDFS/OWL-RL/N3/entailment + rdf-turtle inference ratchet. |
| `coverage ratchet + test-presence gate (per-crate)` | The per-crate line-coverage floor + the test-presence gate. |
| `wasm build (sparq-wasm)` | The `wasm32-unknown-unknown` build, the wasm-deps guard, and `wasm-pack test --node`. |

From the security / supply-chain / SAST workflows (aggregated by the gate; all LIVE except
CodeQL, which is operationally disabled via manual workflow-disable — see the OPERATIONALLY DISABLED note):

| Job name | Workflow | What it gates |
|---|---|---|
| `cargo-deny (advisories + bans + sources + licenses)` | `.github/workflows/supply-chain.yml` | `cargo deny check bans sources licenses` (gating); advisories informational until cargo-deny ships CVSS-4.0 support (the daily `dependency-monitoring.yml` is the real advisory watchdog). |
| `generate CycloneDX SBOM` | `.github/workflows/supply-chain.yml` | The CycloneDX SBOM artifact. |
| `CodeQL analysis (rust)` | `.github/workflows/codeql.yml` | CodeQL SAST (`security-and-quality`) over the Rust workspace — resolves Scorecard's `SAST` check. |

> **CodeQL is currently OPERATIONALLY DISABLED (2026-07-18).** The rows and later
> sections below that describe `CodeQL analysis (rust)` as a live, gating per-PR
> check-run reflect the *ruleset's* intent, but the `codeql.yml` workflow is disabled
> via `gh workflow disable` (Actions workflow state `disabled_manually`): the file is
> retained on `main` with live triggers, but GitHub schedules no run, so no CodeQL
> check-run is produced on any event and it neither runs nor gates today. Open PR
> #3427 owns the successor policy (an advisory / retroactive posture) and the file's
> triggers are left untouched here to avoid colliding with it. Read every "CodeQL …
> gates" statement in this document against this note. (See *Merge-queue subset* below.)

From the binding/packaging workflows (when those surfaces are exercised):

| Job name | Workflow | What it gates |
|---|---|---|
| `maturin build + pytest` | `.github/workflows/python.yml` | The `sparq-rdf` PyPI binding (`import sparq`) build + pytest parity suite. |
| (js binding job) | `.github/workflows/js.yml` | The `@jeswr/sparq` npm build/tests. |

> **Benchmarks — the DETERMINISTIC ratchet gates on PRs; the NOISY timing is nightly (sq-6vshe.6,
> maintainer-directed).** On a `pull_request` the `bench.yml` `run + track benchmarks` job runs the
> FAST DETERMINISTIC form only (`ci-bench.sh --deterministic-only`): the byte-count / memory-layout
> ratchet (store/dict/wasm bytes — a pure function of the code, immune to shared-runner noise) IS
> aggregated by the gate (its name has no advisory token) and hard-fails the gate on a real
> regression. `bench.yml` no longer triggers on `merge_group` at all — the deterministic ratchet
> already ran on the PR head, re-runs on push-to-main, and the merged-tree wasm-feature-OFF invariant
> is independently guarded on `merge_group` by `vectorized-feature-off.yml`'s `artifact-exact-equality`
> leg — so the bench check simply does not appear on the merge_group ref and the gate never waits on
> it (the required set is the single `gate` context, not this job by name). The NOISY wall-clock timing
> suites (query latencies + the well-known sp2b/dbpsb/watdiv/bsbm/lubm + cargo-only latency suites)
> were dragging the merge queue and flapping the gate on shared runners, so they are RELOCATED to the
> nightly EC2 lane (`bench-ec2.yml` `nightly-full-bench`, cron, quiet dedicated spot instance) which
> publishes the full at-scale series to the `benchmark-data` branch the Pages dashboard reads — the
> perf-tracking is moved, not lost. The weekly heavy EC2 campaign (`bench-ec2.yml` `ec2-bench`) and
> release/dist workflows remain non-gating (release/dist fire only on tags). The **Scorecard** workflow
> (`scorecard.yml`) re-scores posture on push to `main` and feeds the public OpenSSF
> dashboard/badge (its SARIF upload to GitHub code-scanning is disabled and the job has
> no `security-events: write` — see the Scorecard note later in this document); with
> **CodeQL** operationally disabled (manual workflow-disable — see the OPERATIONALLY
> DISABLED note), the only current GitHub code-scanning feeders are the Trivy
> SARIF uploads (`container-scan.yml` on PR/main/schedule; `release.yml` for
> released images) and no `CodeQL analysis (rust)` per-PR check-run is produced.
>
> **No branch-protection ruleset change is required for this benchmark relocation.** The live ruleset
> requires exactly one context (`gate`), never the `bench` job by name, and the aggregator discovers the
> live check set at run time — so removing bench from `merge_group` and narrowing its PR run to the
> deterministic ratchet needs no ruleset edit. (If the maintainer had ever added the bench job by name
> to the required-checks list, THAT name would now need removing — but per the "select only
> `ci-summary / gate`" rule above, it was never added.)

### Merge-queue subset — the maintainer-directed risk posture (2026-07-18)

<!-- [FABLE-5] PR #3511 review finding 6: this records the DECISION so a future
     reviewer sees the reduced merge_group check-set is deliberate, not an accident. -->

The `merge_group` ref does **not** run an identical check-set to a PR. By explicit
maintainer direction (2026-07-18) the merge queue runs only the **PR-relevant subset**
of lanes, and — by a separate maintainer direction — **CodeQL is operationally
disabled**: the `.github/workflows/codeql.yml` FILE is retained on `main`
BYTE-IDENTICAL to before, with its full trigger set
(`pull_request`/`push`/`merge_group`/`schedule`) UNTOUCHED, but the workflow itself was
disabled via `gh workflow disable` (Actions workflow state `disabled_manually`), so
GitHub does not schedule it on any event — no CodeQL check-run is produced on ANY
trigger, so it neither runs nor gates. (This is an inert-file state, NOT an edit to the
triggers — this PR does not modify codeql.yml at all; open PR #3427 owns the codeql.yml
successor policy — an advisory / retroactive posture — so the file's triggers are left
untouched here to avoid colliding with it.)
Heavy or independent lanes that already ran and gated on the PR head dropped their
`merge_group` trigger because the queue re-run added wall-clock per enqueue with no new
signal: currently `formal-verification.yml` (Kani proofs), `fuzz.yml` (corpus replay),
and `bench.yml` (noisy timing suites; the deterministic ratchet is guarded separately).
This is a **decision, not a defect**.

Why it stays sound:

- The gate polls whatever sibling check-runs actually exist on the ref and requires only
  the single `gate` context — a lane absent from `merge_group` is *never scheduled*, so
  it is never "expected but missing" and the gate never hangs on it.
- Every subset lane still runs and gates on the **PR head** (and the draft→ready-for-review
  re-run), so a real break is caught **before** admission for the code that changed.
- The **safety net for a lane the queue skips is POST-MERGE detection**: each such lane
  runs on `push`-to-`main`, backed by `nightly-full-sweep.yml` and the
  `formal-alarm.yml` / selection-alarm liveness monitors. A break that only a queued
  *combination* of PRs could produce is therefore detected on `main`, then recovered by
  **revert / fix-forward**.
- **There is no POST-MERGE bisection.** `batch-merge.yml` *does* bisect — but only
  **pre-merge**, to isolate which constituent of a failed *omnibus* is at fault (see
  §Omnibus batching). Once a break has landed on `main`, the recovery mechanism is
  revert/fix-forward plus the age-bound liveness backstop; nothing bisects `main`'s
  history automatically. (Workflow comments that previously called batch-merge "the
  bisection recovery net" were corrected in PR #3511 finding 6b to say post-merge
  detection + revert/fix-forward — that correction still holds for `main`.)

The residual risk this accepts: a defect that manifests only in a *specific queued
combination* of PRs (not on any single PR head) reaches `main` and is caught post-merge
rather than pre-merge. The maintainer accepted this trade against per-enqueue wall-clock
and shared-runner contention.

## Draft-tier CI (reduced matrix on draft PR heads)

<!-- [FABLE-5] Draft-tier CI design record (2026-07-17). Motivation: the autonomous
     fleet keeps many draft worker PRs cycling review-fix rounds; every push ran the
     FULL matrix, saturating the org runner pool (gates timing out as false failures,
     the merge queue starving, the load-aware heavy shards deferring). -->

CI is **tiered by the PR's draft state**. A **draft** `pull_request` head runs a
REDUCED sibling set; a **non-draft** `pull_request` head, `push`-to-main, and every
scheduled/dispatch run keep the FULL matrix, byte-identical to before. (`merge_group`
is a SEPARATE axis: it always runs at FULL tier — never draft-tier — but it runs the
maintainer-directed **PR-relevant subset** documented in *Merge-queue subset* above —
several heavy lanes do not trigger there, and CodeQL (though its trigger set still
lists `merge_group`, byte-identical to main) is operationally disabled and so produces
no check-run there either — not a byte-identical copy of the PR check-set.)

**What a draft head runs:**

- the **change-scoped crate legs** — the existing `ci-select` change-based selection
  (affected reverse-dependency closure) already intersects every wide lane and the
  opt-in feature-matrix legs with the diff, on both tiers;
- the **cheap global gates** — clippy/fmt, MSRV, docs-quality (typos / privacy /
  ci-scripts), supply-chain, conformance ratchets for affected crates, pr-title;
- the **`ci-summary / gate` aggregator**, which evaluates exactly the reduced set it
  discovers (it is discovery-based, so no expected-leg list needs maintaining).

**What a draft head skips** (each re-runs at full tier before any merge is possible):

| Skipped on drafts | Where | Kept when |
|---|---|---|
| coverage ratchet (measure + engine split + aggregate) | `ci.yml` | never on drafts (merge_group + ready_for_review re-measure) |
| benchmarks (deterministic ratchet + PR comparison/alert comments) | `bench.yml` | never on drafts |
| CodeQL analysis | `codeql.yml` | never on drafts (push-main + weekly schedule + merge_group + the ready_for_review run keep the `code_scanning` rule fed *when the workflow is enabled*; codeql.yml is byte-identical to `main` — its triggers, including merge_group, are untouched by this PR — but the workflow is currently operationally disabled (`disabled_manually`), so no CodeQL check-run is produced on any trigger today; open PR #3427 owns the successor policy) |
| heavy recall shards (`heavy-diskann`/`heavy-hnsw`) | `ci.yml` `test` | never on drafts (same demotion mechanism as their merge_group demotion) |
| wasm bundle build | `ci.yml` `wasm` | kept iff a wasm-bundle crate is in the affected closure (the existing lane-seed guard — unchanged on both tiers) |
| `artifact-exact-equality` (wasm feature-OFF byte identity) | `vectorized-feature-off.yml` | kept iff `sparq-wasm` is in the affected closure (in-step `ci_select.py` verdict; ci-full label / selector error / full mode ⇒ run) |

**The integrity invariant — a draft-tier gate result must NEVER admit a PR to the
merge queue.** The load-bearing mechanism is rule 1 (structural); rules 2–6 are
belts (all in `scripts/ci_summary_gate.py`, unit-tested in
`scripts/tests/test_ci_summary_gate.py`; the name/trigger wiring pinned by
`scripts/tests/test_ci_select_wiring.py`):

1. **A draft-tier run never produces the required context at all.** The
   `ci-summary` gate job's own check-run name is **tiered**: a draft
   `pull_request` payload renders **`gate, draft-tier`**; every other event/state
   renders exactly `gate` — the sole context named by the ruleset's
   `required_status_checks` rule. A draft-built head therefore carries **no
   `gate` check-run at all**, and branch protection blocks on the *missing*
   required check from the un-draft moment until the full-tier run's fresh
   `gate` concludes. There is no supersession window to race: `gh pr ready &&
   gh pr merge --auto` (the fleet's standard flow) arms and waits; GitHub event
   latency, a *dropped* `ready_for_review` event, or an Actions outage all
   leave the required context ABSENT — blocked — never satisfied by a
   draft-tier result. Stale `gate, draft-tier` check-runs are tier artifacts:
   the gate script excludes them from every sibling set (a completed draft-tier
   verdict, green or red, is superseded by the live full-tier evaluation).
2. **Supersession by re-run.** Every gate-feeding workflow's `pull_request` trigger
   now includes **`ready_for_review`** (the default types are only
   opened/synchronize/reopened — without this the un-draft moment would run
   *nothing* and the head would keep its draft-tier results). Un-drafting therefore
   fires a FULL-tier run on the **same head SHA**, which produces the first (and
   only) `gate` check-run for that head.
3. **The gate knows its tier.** Each `ci-summary` run derives its tier from its own
   trigger payload (`PR_DRAFT`), and the reusable `ci-select` job's check-run **name
   carries a `", draft-tier"` marker** on draft-assembled runs (name-as-contract,
   like the advisory rule).
4. **A full-tier gate refuses a draft-tier leg set — per INSTANCE.** `ci.yml`,
   `bench.yml`, `feature-matrix.yml` and `fuzz.yml` all call the same reusable
   `ci-select` job, so a head SHA carries up to **four** draft-marked selects
   under the IDENTICAL check-run name. A full-tier `pull_request` gate requires
   each draft-marked instance to have its **own, distinct, strictly-later**
   full-tier successor (greedy start-order matching) — the first workflow's
   full-tier select can never release the hold for the other three, whose
   full-tier runs may not have registered any check-runs yet. While any
   instance lacks a successor the set is *still settling* (the
   ready_for_review re-runs are expected); at budget exhaustion the verdict is
   **FAILURE — "stale draft-tier run, full run pending"**, never a pass over
   draft legs.
5. **A draft-tier gate re-checks the PR at conclusion time.** Before emitting
   SUCCESS it re-reads the PR's **current** draft state from the API
   (`pull-requests: read`); if the PR was un-drafted meanwhile it concludes
   FAILURE with the same stale-draft-tier message, and an unreadable state
   fail-closes to FAILURE (a draft PR cannot merge anyway).
6. **Newest workflow run wins (#3505).** [GPT-5.6] The ready_for_review re-run's
   per-PR concurrency groups cancel the in-flight draft-tier runs, leaving terminal
   checks on the same SHA. The gate resolves them by `workflow_id`: only the newest
   run (by creation/run id) and its newest `run_attempt` is authoritative. Every
   older run is a non-event even if its conclusion was `cancelled` or `failure`; a
   genuine failure in the newest run/attempt still REDs. A newest cancelled run is
   re-dispatched once (`actions: write`); if the retry is cancelled or never
   advances, the gate REDs loudly with `superseded-legs, re-run required`.
   Attempt-scoped job listing supplies the completed leg inventory and prevents
   an old attempt that reused the run id from leaking into the verdict; the
   workflow-run conclusion supplies the verdict when an entire job evaporates.

**Why the queue can never latch a draft-tier result.** Rule 1 is structural: the
queue and branch protection admit a PR only on a successful check-run of the
exact context `gate`, and no draft-tier run ever emits one. This matters more
than it may look, because the `merge_group` run deliberately omits two lanes
(`bench.yml`'s deterministic byte ratchet and the heavy recall shards,
sq-6vshe.6) on the premise that their full form already ran on the PR head —
a premise a draft-built head would otherwise break. Rules 2–6 alone would NOT
close that: a concluded draft-tier `gate` success would remain the *latest* run
of the required context from the un-draft moment until the ready_for_review
`ci-summary` run registers its check-run (seconds of event latency; indefinitely
if the event is dropped), and `gh pr ready && gh pr merge --auto` could enqueue
inside that window. With the tiered check name the window does not exist —
so the ready_for_review full-tier PR run (which includes both merge_group-absent
lanes) must conclude before the queue can admit the head.

The ruleset additionally carries a `code_scanning` rule (CodeQL). While CodeQL
is operationally disabled (manual workflow-disable — see the OPERATIONALLY DISABLED note) no
analyses are produced, so this rule currently exerts **no** blocking pressure.
If CodeQL is re-enabled: a draft-built head carries no PR CodeQL analysis
(`analyze` skips on drafts), so the rule would *independently* block such a head
— but it is an **out-of-repo, owner-mutable setting** and evadable in corner
cases (a non-draft PR sharing the same head SHA supplies an analysis for the
commit; the rule may be relaxed during a CodeQL outage), so it is recorded here
as **defense-in-depth only**, never the load-bearing mechanism. Do not weaken
rule 1 on the strength of it.

**Operational notes.** `pull_request`-event CI runs are cancel-superseded per PR
(`concurrency` groups; `bench.yml` now cancels superseded **PR** runs only — its
push/schedule runs still never cancel, protecting the benchmark history — and
`js.yml` gained the standard per-PR group). `merge_group` runs are never cancelled
by these groups. **No required-check name changed** — the ruleset still requires
exactly `ci-summary / gate`, and every full-tier run still emits exactly that
context; a draft-tier run emits the additional, deliberately **non-required**
context `gate, draft-tier` instead (tooling reading a draft head's checks sees
the tier verdict there, while the required `gate` context stays absent until
un-draft — a draft PR cannot merge regardless). Toggling draft state does not
change what ultimately gates a merge; it only defers the heavy lanes to the
un-draft moment.

## Required reviews

> **Solo-maintainer reality (read this first).** sparq is a **single-maintainer,
> agent-driven** repository: every PR is authored by `@jeswr` or by an automated SPARQL
> agent acting on his behalf. GitHub does **not** let an author approve their own PR, so a
> *human-approval* requirement (`required_approving_review_count ≥ 1` and/or
> `require_code_owner_review`) would **deadlock** the merge train — there is no second human
> to approve. The **live ruleset therefore sets `required_approving_review_count: 0` and
> `require_code_owner_review: false` deliberately**, and substitutes a *bot/automated*
> review layer (Copilot code review on push + CodeQL code-scanning gate + the `ci-summary`
> aggregator + conversation-resolution) for the missing second human. This is the same
> reality OpenSSF Scorecard's `Code-Review` / `Branch-Protection` checks score down — see
> [§Solo-maintainer & the Scorecard score](#solo-maintainer--the-scorecard-code-review--branch-protection-score)
> below; the settings here are written to match **what is actually enforced**, not an
> aspirational two-human flow the repo cannot run.

- **Approving reviews — `0` required (deliberate, solo-maintainer).** The live ruleset's
  `pull_request` rule sets `required_approving_review_count: 0` and
  `require_code_owner_review: false`. [`CODEOWNERS`](../CODEOWNERS) still records ownership
  of the high-risk paths (`sparq-zk*`, `sparq-mpc`, `sparq-core`, `sparq-server`,
  `.github/`, `deny.toml`, `SECURITY.md`) so that *if/when* a second trusted reviewer is
  added, code-owner review can be flipped on without re-deriving who owns what; today it
  documents intent rather than gating.
- **Stale-approval dismissal — `false` (no human approvals to dismiss).** With zero required
  human approvals there is nothing to stale-dismiss; the live ruleset sets
  `dismiss_stale_reviews_on_push: false` to match. (Copilot review *does* re-run on push —
  `review_on_push: true`.)
- **Require the automated code review** (GitHub Copilot code review + the CodeQL
  code-scanning review). The live ruleset enables Copilot code review on push
  (`copilot_code_review` rule, `review_on_push: true`) and treats **CodeQL code-scanning
  alerts** as blocking via the `code_scanning` rule (`CodeQL`, `alerts_threshold:
  errors_and_warnings`, `security_alerts_threshold: all`). The CodeQL run is also aggregated
  by `ci-summary` as the `CodeQL analysis (rust)` check-run; the code-scanning *results*
  rule is the complementary alert-severity gate.
- **Require conversation resolution before merging** — all PR review threads (human and
  bot, incl. Copilot/CodeQL) must be resolved (live ruleset `pull_request`
  `required_review_thread_resolution: true`). (Also listed under "Other settings".)
- **Code-quality rule active.** The live ruleset also carries a `code_quality` rule
  (`severity: all`), GitHub's built-in PR quality signal, alongside the checks above.

## History and push rules

- **Require linear history** — merges to `main` must not introduce merge commits. The live
  ruleset enforces this by allowing **only the squash merge method**
  (`pull_request.allowed_merge_methods: ["squash"]`) and a `non_fast_forward` rule, which
  matches the "gate and merge one branch at a time" discipline in `AGENTS.md`.
- **Block force pushes** to `main` (live ruleset `non_fast_forward` rule).
- **Block branch deletion** for `main` (live ruleset `deletion` rule).

## Other settings

- **Do not allow bypassing the above** — the rules apply to administrators too. The live
  ruleset has an **empty `bypass_actors` list** and reports `current_user_can_bypass:
  never`, so the gate is uniform (no bypass actors, including the owner).
- **Require conversation resolution before merging** (all PR review threads resolved —
  `required_review_thread_resolution: true`).

## How this maps to the merge discipline

`AGENTS.md` defines the landing gate as *full-workspace clippy + `cargo test` + the
conformance/perf/coverage ratchets, all green*, with parallel worktrees gated and merged
**one branch at a time**. The single required check — `ci-summary / gate` — is the CI
enforcement of that gate: it aggregates every other check-run, so the gate stays complete
even as jobs are added or renamed. Linear history (squash-only) + the automated review
layer (Copilot + CodeQL code-scanning) + conversation resolution enforce the one-at-a-time
merge discipline — human approvals are **not** required (solo-maintainer; see
[§Solo-maintainer & the Scorecard score](#solo-maintainer--the-scorecard-code-review--branch-protection-score)).
When a new ratchet or gate is added to a CI workflow it is covered
automatically (no ruleset edit needed); update the informational table above so reviewers
keep an accurate map.

### Omnibus batching (merge-queue overflow)

The merge queue on `main` drains individually-armed worker PRs up to its per-window cap
(`max_entries_to_merge: 8`). Whenever **2 or more** reviewed worker PRs (open `sparq-agent/*`
heads carrying `review:pass` with an active auto-merge arm by `app/sparq-orchestrator`)
are waiting at once, the scheduled/event-driven batcher
([`scripts/batch-merge.py`](../scripts/batch-merge.py), run by
[`.github/workflows/batch-merge.yml`](../.github/workflows/batch-merge.yml), every
15 minutes) folds **all** of them — there is no reserved queue window, because an omnibus
occupies one queue entry however many constituents it carries and batching the window too
removes 8 separate head-CI runs per cycle — into one
`sparq-omnibus/<class>-<utcstamp>` integration PR **per change-class** (issue #3433:
`slim` = constituents whose diffs are docs-/orchestration-only per `ci_select`'s audited
allowlist, so a slim batch rides the slim merge-group lane set and never waits on a
full-matrix run; `engine` = everything else, fail-closed; at least 2 and at most 15
constituents per omnibus — batch-15 per 15-minute run tracks the 60-merges/hour target
and keeps a culprit bisect at log2(15) ≈ 4 runs, with the excess left individually armed
for the next run; one omnibus **tree** per class in flight, and a legacy classless
omnibus blocks every class).
Each omnibus is fresh off `main`, built with sequential `--no-ff` merges (a conflicting
constituent is skipped and stays individually armed) and armed so a single
queue slot lands the whole batch; the omnibus body carries `Closes #` refs for every
constituent's issue, and once it merges the batcher closes each contained constituent PR.
The `sparq-omnibus/` prefix (and the absence of any `review:*` label) keeps these PRs out
of the registry's worker-review enumeration, which only admits `sparq-agent/issue-<n>-…`
heads. The omnibus branch/PR is pushed, created and armed with a **sparq-orchestrator App
installation token** (repo secrets `ORCHESTRATOR_APP_ID` / `ORCHESTRATOR_APP_PRIVATE_KEY`):
a `GITHUB_TOKEN`-created PR gets its workflow events suppressed, so the required
`ci-summary / gate` would never report on its head and the merge queue would never admit
it (admission requires the required checks to pass *before* entry). Without those secrets
the batcher fail-softs to hygiene-only mode (no new omnibus is created). Failure handling
is liveness-bounded and **ordered most-exculpatory first**, because a broken batch must
never implicate an innocent constituent:

- an omnibus **conflicting** with `main` (main moved under it) is closed and rebuilt fresh
  off current `main` — checked *before* any gate verdict, and never a constituent fault;
- only a **strict concluded head-`gate` failure on a definitively-`MERGEABLE` head**
  recursively **bisects** the omnibus (halve-and-land, to `DEPTH_CAP`) until a single
  constituent is isolated; that culprit is commented, converted to **draft** and disarmed
  — the registry's `stranded` posture, which escalates to a human — and is never closed.
  Children inherit their parent's change-class. Ambiguous causality (a still-code-bearing
  ineligible sibling) or a constituent branch that **drifted** off the head OID the
  omnibus pinned yields a fresh singleton **re-test** child instead of a conviction;
- an **infra-shaped** ending — `gate` cancelled / timed out, or the age bound
  (`MAX_OMNIBUS_AGE_HOURS`, the backstop for merge-group failures, which report on the
  queue's synthetic ref rather than the PR head) — closes the omnibus **without**
  bisection and without a same-run rebuild; a new root then waits out
  `ROOT_COOLDOWN_HOURS` (read back from GitHub's closed-PR record, since the batcher is
  stateless) so a sustained outage cannot flood the queue with rebuilt roots;
- a young mergeable omnibus whose auto-merge arm was dropped (merge groups drop the arm on
  a failed group) is re-armed idempotently; and a possibly-**truncated** open-PR listing is
  fail-safe — no bisection, no stale-branch deletion and no new root on incomplete data.

In every failure case except the isolated culprit the constituents remain individually
armed, so the failure mode is "no worse than unbatched". The same workflow's `ring` job fires on every push to `main`
and, when the `REGISTRY_RING_TOKEN` secret is configured, pokes the
`jeswr/agent-account-registry` dispatcher so freed capacity is picked up immediately
(fail-soft: without the secret it skips with a notice and the registry cron is the
backstop). The batcher is **not** a required check and never runs on a PR head commit.

> All third-party GitHub Actions across `.github/workflows/*.yml` are **pinned by full
> commit SHA** (with a trailing `# vX.Y.Z` comment that Dependabot follows), resolving
> the Scorecard `Pinned-Dependencies` alerts. The one documented nuance:
> `dtolnay/rust-toolchain` is pinned to the **commit SHA of its `stable` / `1.88`
> branch tip** — that action selects the toolchain from the `action.yml` content at the
> ref (input default `stable`, or a hard-wired `1.88.0`), not from the ref *name*, so
> the SHA pin preserves toolchain selection (verified against the action source).

## Solo-maintainer & the Scorecard Code-Review / Branch-Protection score

<!-- [OPUS-4.8] Solo-maintainer evidence for OpenSSF Scorecard Code-Review /
     Branch-Protection (bead sq-sto1, gap GX-OSSF-3). -->

This section is the **doc-of-record evidence** for why OpenSSF Scorecard's `Code-Review`
and `Branch-Protection` checks score below 10 for this repository, and what *compensating*
controls stand in. It is the in-repo half of gap **GX-OSSF-3**
([`compliance/openssf/gap-register.md`](../compliance/openssf/gap-register.md)); the
remaining half is the maintainer periodically re-confirming the **live** ruleset against
this document (procedure below).

### Why the score is depressed (honest, not a defect)

- **`Code-Review`** — Scorecard infers code review from **merged-PR history** and
  **discounts self-approval**. In a single-maintainer, agent-driven repo there is no second
  human to record an independent approving review, so the history-derived signal is weak by
  construction. The repo does **not** fake this with a self-approval (which Scorecard
  discounts anyway and which the [`AGENTS.md`](../AGENTS.md) honesty posture forbids).
- **`Branch-Protection`** — Scorecard rewards *classic*-branch-protection settings such as
  `required_approving_review_count ≥ 1`, `require_code_owner_review`, and
  stale-review-dismissal. The live model deliberately sets all three to the
  "no second human" values (`0` / `false` / `false`, see [§Required reviews](#required-reviews)),
  so those particular sub-signals do not earn points even though the *substantive*
  protections (no force-push, no deletion, squash-only linear history, conversation
  resolution, CodeQL alert gate, no bypass actors, a required CI aggregator) are all
  present and enforced.

These are **inherent to the operating model**, not fixable code changes — consistent with
the disposition recorded in `compliance/openssf/gap-register.md` (the Scorecard SARIF is no
longer uploaded to code-scanning precisely because these are posture *scores*, not code
alerts).

### Compensating controls (what substitutes for the missing second human)

| Missing classic signal | Compensating control (live + enforced) |
|---|---|
| Independent human approving review | **GitHub Copilot code review on every PR** (`copilot_code_review`, `review_on_push: true`) — an automated, independent reviewer recorded on the PR. |
| Code-owner gate | **CodeQL code-scanning gate** (`code_scanning` rule, `CodeQL`, `errors_and_warnings`) — blocks merge on new alerts; plus the SHA-pinned clippy/test/conformance gate aggregated by `ci-summary`. |
| Review-thread accountability | **Conversation resolution required** (`required_review_thread_resolution: true`) — every Copilot/CodeQL thread must be resolved before merge. |
| "Trusted committer only" | **No bypass actors** (`bypass_actors: []`, `current_user_can_bypass: never`) — the gate applies to the owner too; **squash-only** + **no force-push** + **no deletion** keep history linear and auditable. |

The agent operating discipline (`AGENTS.md`) adds a *process* layer on top: changes land via
PR (never direct push), and an out-of-band Codex/roborev review pass is run before arming a
PR for merge. That review is not visible to Scorecard's history heuristic, but it is the
real independent-review substitute in practice.

### Verifying the live ruleset matches this document

The live ruleset is configured **out-of-repo** and cannot be asserted from a tracked file,
so confirm it with the GitHub API (read-only token is sufficient):

```sh
# List rulesets on the default branch and grab the `main` ruleset id.
gh api repos/jeswr/sparq/rulesets

# Dump the full rule set and eyeball it against this document.
gh api repos/jeswr/sparq/rulesets/<id> | python3 -m json.tool
```

As verified on the date of this commit, the live `main` ruleset
(`enforcement: active`, `bypass_actors: []`) carries exactly these rules, all of which match
the sections above:

| Live rule (`type`) | Key parameters | Doc section |
|---|---|---|
| `deletion` | — | History and push rules |
| `non_fast_forward` | — | History and push rules (force-push + linear history) |
| `pull_request` | `required_approving_review_count: 0`, `require_code_owner_review: false`, `dismiss_stale_reviews_on_push: false`, `required_review_thread_resolution: true`, `allowed_merge_methods: ["squash"]` | Required reviews |
| `required_status_checks` | one context `gate`, `strict_required_status_checks_policy: false` | Required status checks |
| `code_quality` | `severity: all` | Required reviews |
| `code_scanning` | `CodeQL`, `alerts_threshold: errors_and_warnings`, `security_alerts_threshold: all` | Required reviews |
| `copilot_code_review` | `review_on_push: true`, `review_draft_pull_requests: false` | Required reviews |

If a future check finds drift (e.g. a rule added or a parameter changed), update **this
table and the matching section above in the same commit** so the doc-of-record never lags
the live ruleset.
