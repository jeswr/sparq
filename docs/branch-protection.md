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
| **`ci-summary / gate`** | [`.github/workflows/ci-summary.yml`](../.github/workflows/ci-summary.yml) | **The single gate.** Polls every *other* check-run on the PR head commit and passes iff none failed (`success`/`skipped`/`neutral` are non-failing). |

> **Select only `ci-summary / gate`** in the ruleset's "Require status checks that must
> pass" list. Do **not** add the individual job names below — `ci-summary` already
> aggregates them. This is deliberate: `needs:` cannot span workflows, so requiring each
> job by name was brittle (every rename / added gate broke the rule and silently weakened
> the gate). The aggregator adapts automatically — add or rename jobs freely and the gate
> still covers them, because it discovers the live set of check-runs at run time. See the
> header of `ci-summary.yml` for the full semantics (stability window + self-exclusion).

### What `ci-summary` aggregates (informational — do NOT add these individually)

The gate covers **every** check-run on the head commit. As of this writing those are the
jobs below; this table is a map for reviewers, **not** a list of required checks.

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

From the security / supply-chain / SAST workflows (all now LIVE and aggregated by the gate):

| Job name | Workflow | What it gates |
|---|---|---|
| `cargo-deny (advisories + bans + sources + licenses)` | `.github/workflows/supply-chain.yml` | `cargo deny check bans sources licenses` (gating); advisories informational until cargo-deny ships CVSS-4.0 support (the daily `dependency-monitoring.yml` is the real advisory watchdog). |
| `generate CycloneDX SBOM` | `.github/workflows/supply-chain.yml` | The CycloneDX SBOM artifact. |
| `CodeQL analysis (rust)` | `.github/workflows/codeql.yml` | CodeQL SAST (`security-and-quality`) over the Rust workspace — resolves Scorecard's `SAST` check. |

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
> (`scorecard.yml`) re-scores posture on push to `main` and feeds the code-scanning
> dashboard; **CodeQL** + **Scorecard** therefore both feed the gate/dashboard even
> though only the per-commit `CodeQL analysis (rust)` check is a per-PR check-run.
>
> **No branch-protection ruleset change is required for this benchmark relocation.** The live ruleset
> requires exactly one context (`gate`), never the `bench` job by name, and the aggregator discovers the
> live check set at run time — so removing bench from `merge_group` and narrowing its PR run to the
> deterministic ratchet needs no ruleset edit. (If the maintainer had ever added the bench job by name
> to the required-checks list, THAT name would now need removing — but per the "select only
> `ci-summary / gate`" rule above, it was never added.)

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
