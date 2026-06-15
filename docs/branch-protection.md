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

PRs must be **up to date with `main`** before merging. There is now exactly **ONE**
required status check — the aggregator:

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

> Heavy benchmarks (`bench.yml`, `bench-ec2.yml`) and release/dist workflows are **not**
> aggregated as per-PR gates (bench runs on PRs for feedback but its own hard gate
> self-fails; release/dist fire only on tags). The **Scorecard** workflow
> (`scorecard.yml`) re-scores posture on push to `main` and feeds the code-scanning
> dashboard; **CodeQL** + **Scorecard** therefore both feed the gate/dashboard even
> though only the per-commit `CodeQL analysis (rust)` check is a per-PR check-run.

## Required reviews

- **At least 1 approving review**, and **require review from Code Owners**
  (see [`CODEOWNERS`](../CODEOWNERS)). A change to a high-risk path
  (`sparq-zk*`, `sparq-mpc`, `sparq-core`, `sparq-server`, `.github/`, `deny.toml`,
  `SECURITY.md`) therefore needs the listed owner's approval.
- **Dismiss stale approvals** when new commits are pushed.
- **Require the automated code review** (GitHub Copilot code review + the CodeQL
  code-scanning review). Enable "Request Copilot code review automatically" for `main`
  PRs, and treat **CodeQL code-scanning alerts** as blocking — pair the ruleset with a
  *code-scanning results* check requiring no new `error`/`high`-severity CodeQL alerts.
  (The CodeQL run itself is also aggregated by `ci-summary` as the `CodeQL analysis
  (rust)` check-run; the code-scanning *results* requirement is the complementary
  alert-severity gate.)
- **Require conversation resolution before merging** — all PR review threads (human and
  bot, incl. Copilot/CodeQL) must be resolved. (Also listed under "Other settings".)

## History and push rules

- **Require linear history** — merges to `main` must not introduce merge commits
  (use squash or rebase merges). This matches the "gate and merge one branch at a time"
  discipline in `AGENTS.md`.
- **Block force pushes** to `main`.
- **Block branch deletion** for `main`.

## Other settings

- **Do not allow bypassing the above** — apply the rules to administrators too
  (include administrators / no bypass actors), so the gate is uniform.
- **Require conversation resolution before merging** (all PR review threads resolved).

## How this maps to the merge discipline

`AGENTS.md` defines the landing gate as *full-workspace clippy + `cargo test` + the
conformance/perf/coverage ratchets, all green*, with parallel worktrees gated and merged
**one branch at a time**. The single required check — `ci-summary / gate` — is the CI
enforcement of that gate: it aggregates every other check-run, so the gate stays complete
even as jobs are added or renamed. Linear history + one CODEOWNERS approval + the
up-to-date requirement + conversation resolution enforce the one-at-a-time merge
discipline. When a new ratchet or gate is added to a CI workflow it is covered
automatically (no ruleset edit needed); update the informational table above so reviewers
keep an accurate map.

> All third-party GitHub Actions across `.github/workflows/*.yml` are **pinned by full
> commit SHA** (with a trailing `# vX.Y.Z` comment that Dependabot follows), resolving
> the Scorecard `Pinned-Dependencies` alerts. The one documented nuance:
> `dtolnay/rust-toolchain` is pinned to the **commit SHA of its `stable` / `1.88`
> branch tip** — that action selects the toolchain from the `action.yml` content at the
> ref (input default `stable`, or a hard-wired `1.88.0`), not from the ref *name*, so
> the SHA pin preserves toolchain selection (verified against the action source).
