# Private-registry worker — the self-maintenance orchestrator runs INSIDE the account registry

> Status: design record for maintainer review, reflecting the **maintainer-locked decision of
> 2026-07-16**: the autonomous worker runs as a GitHub Actions job **inside the private
> `jeswr/agent-account-registry` repo**, next to the model-account secrets. `[FABLE-5]`
>
> This record **supersedes the token-placement sections** of
> [`issue-native-orchestration.md`](issue-native-orchestration.md) (the "DECISION NEEDED —
> model-token placement" section, options A/B) and of
> [`orchestration-activation-runbook.md`](orchestration-activation-runbook.md) (step 0, "Token
> boundary"), and **revises** the cost-model section of the former (workers no longer run on the
> free public runner). Everything else in those records — the issue-native tracker, the trust
> containment, the readiness/routing/lease machinery — stands and is reused as-is (§5.2).

## 0. TL;DR — what was decided, and what this record adds

**Goal (locked):** a ready GitHub issue in a target repo → auto-triaged → dispatched to a worker
that runs a model headless, gates the result, opens a PR under a bot identity, which auto-merges
through the target repo's own CI. The manual/session version of this loop already works; this
makes it autonomous, and makes it scale to the maintainer's **other repos** later.

**Architecture (locked, chosen over a Lambda/API-gateway proxy AND over copying tokens into every
target repo's secrets):** the worker job runs **in the private registry repo's GitHub Actions**.

- Model-account tokens stay in **one** private repo (`ACCT01_TOKEN`, `ACCT02_TOKEN`, … as
  registry secrets) — never duplicated into public repos, so onboarding target repo N+1 adds
  **zero** new credential copies.
- No hosted proxy/broker service to build, deploy, patch, or keep alive.
- The worker uses its claimed ACCT secret to run the model CLI, and a **GitHub-App installation
  token** (App `sparq-orchestrator`, App ID 4300853, owned by `sparq-org`; installed + verified —
  the registry's `verify-app` run of 2026-07-16 minted a `sparq-org/sparq` token) to check out,
  branch, and open the PR against the **target** public repo.
- Each target repo keeps its **own** `ci-summary` + branch-protection + merge-queue gates — the
  correctness backstop is unchanged and stays in the target repo.
- Accepted trade-off: private-repo Actions minutes are **metered** (public-repo standard runners
  are free). This is modest next to model-API/subscription spend at current volume; **if** volume
  later makes it material, the documented evolution is free public-runner workers plus a private
  credential broker (§7) — noted, deliberately **not designed here**.

**What this record adds:** the end-to-end flow with trust boundaries (§2), the per-repo policy
table (§3), the security model (§4), a concrete PR-by-PR implementation plan reusing the merged
Phase-0..2 estate (§5), and a canary + failure-mode plan (§6).

## 1. Corrections to the brief's premise (verified against the actual estate)

Checked against the merged sparq tree and the live private registry on 2026-07-16:

1. **"Account-issue reaction-lock concurrency model" → it is a CAS lease ledger now.** The
   reaction-lock was disproven by the GPT-5.6 review (C3: GitHub allows one reaction per identity,
   so N same-bot workers all see one 🚀 and all believe they own a slot). The registry already
   ships the replacement: `scripts/select-and-claim.py` compare-and-swaps `data/leases.json`
   (unique `claim_id`, `expires_at`, blob-SHA CAS), and `groom-leases.yml` reclaims expired leases
   every 15 minutes. This design reuses the **lease ledger**; the *one-issue-per-account registry
   shape* (issues #1 `acct01`/OpenAI, #2 `acct02`/Anthropic) is reused exactly as the brief says.
2. **The target repo cannot fire a `repository_dispatch` "authenticated via the App".** Minting an
   App installation token requires the App **private key**, which must never sit in a public repo
   (it spans both repos and all App permissions). So the target-side doorbell needs either a
   separate narrow credential or no credential at all. §2.1 designs the dispatch trigger honestly:
   a **registry-side cron poll is the floor** (zero public-repo credentials), with an optional
   narrow-PAT doorbell as a later latency optimization.
3. **App installation scope:** the verify-app log shows the App installed on **`sparq-org`**
   (organization, selected repositories) and successfully minting a `sparq-org/sparq` token. It is
   **not** currently installed on the `jeswr` account — which only matters for one optional
   capability (registry secret write-back on refresh-token rotation, §4.4) and is listed as an
   open question (§8). The App ID (4300853) is maintainer-provided; the run log masks it (it is
   stored as a secret), but the slug/owner/token-mint are confirmed from the log.
4. **Phase 0–2 are MERGED, not pending.** PRs #2283/#2284/#2285 landed: `scripts/ready-issues.py`
   (fail-closed readiness), `scripts/trust-gate.py` + `triage.py` + `promote.py` +
   `triage-issue.yml` + `promote-on-approval.yml` (live trust containment),
   `scripts/route-resolve.py` + `orchestration/routing.toml` (routing), `scripts/dispatch-plan.py`
   (pure dry-run planner), `scripts/bd-to-issues.py` (migration tooling). The remaining build is
   exactly the worker + dispatcher + policy + grooming in this record.

## 2. End-to-end flow and trust boundaries

```text
 TARGET repo (public, e.g. sparq-org/sparq)          REGISTRY repo (private, jeswr/agent-account-registry)
 ─────────────────────────────────────────           ──────────────────────────────────────────────────────
 issue opened/edited                                  dispatch.yml
   └► triage-issue.yml            ── B1 ──►             triggers: schedule (cron) | workflow_dispatch
        trust-gate (author repo-permission)                       | repository_dispatch (doorbell, §2.1 —
        third-party → trust:untrusted quarantine                    payload IGNORED: a wake-up, never data)
   promote-on-approval.yml (👍 revision-bound)          ┌───────────────────────────────────────────────┐
        → labels: status:ready role:* priority:*        │ job PLAN (unprivileged: permissions:{} ,       │
                  area:<crate>                          │           NO secrets)                          │
                                                        │   checkout TARGET main (public)                │
                                                        │   ready-issues.py → dispatch-plan.py           │
                                                        │   (routing.toml read from the TARGET repo)     │
                                                        │   → plan.json artifact (data, schema-checked)  │
                                                        ├─────────────────── B3 ────────────────────────┤
                                                        │ job CLAIM+LAUNCH (privileged)                  │
                                                        │   validate plan.json strictly                  │
                                                        │   policy/repos.toml → account pool, caps,      │
                                                        │     gate profile, arm policy        (§3)       │
                                                        │   select-and-claim.py --claim (CAS lease)      │
                                                        │   mark issue status:in-progress (App token)    │
                                                        │   spawn worker.yml per plan row                │
                                                        └───────────────────────────────────────────────┘
                                                       worker.yml  (one issue = one run; timeout-capped)
                                                        1. mint App installation token — scoped to THE
                                                           ONE target repo, minimal permissions, ≤1 h
                                                        2. materialize the ONE claimed ACCT credential
                                                           into an ISOLATED $HOME (broker-refresh cores)
                                                        3. checkout target @ main (App token)
                                                        4. run harness headless (claude -p / codex exec)
                                                           on the issue brief            ── B2 ──
                                                        5. crate-scoped gates (policy gate profile)
                                                        6. push branch + open PR as
 PR arrives  ◄──────────────────────────────────────       sparq-orchestrator[bot] (🤖 self-ID body)
   └► ci-summary + branch protection + merge queue      7. arm auto-merge per policy
      (the correctness backstop — B4,                   8. always(): release lease (claim_id), set issue
       unchanged, lives in the TARGET repo)                status (deferred on failure), write-back check
```

Trust boundaries:

- **B1 — third-party content never reaches a model.** The live target-side containment
  (`triage-issue.yml` quarantines non-collaborator authors as `trust:untrusted`;
  `promote-on-approval.yml` promotes only on a maintainer 👍 that post-dates the last edit) plus
  `ready-issues.py`'s **positive attestation** rule (an issue is dispatchable only with
  `status:ready` set by that pipeline — absence of a quarantine label is not enough). The worker
  **re-verifies at the last step**: before the model reads issue content, it re-fetches the issue
  and re-runs `trust-gate.py` on the author + confirms `status:ready` still present and the body
  unedited since planning (revision-bound, review S2). Fails closed to `status:deferred`.
- **B2 — secrets vs the model process.** The model step's environment contains exactly two
  credentials: its own claimed account's **short-lived access credential** in an isolated `$HOME`,
  and the **target-scoped App installation token** (≤1 h, minimal permissions). It never sees
  `REGISTRY_ADMIN_APP_ID/KEY`, other accounts' tokens, `REGISTRY_ADMIN_TOKEN`, or the registry's
  own `GITHUB_TOKEN`. GitHub Actions injects a secret only into steps that reference it — the
  App-key mint and the credential materialization are separate earlier steps.
- **B3 — untrusted code vs the privileged context.** The dispatcher's PLAN job executes
  target-repo scripts (`ready-issues.py` etc. — single-sourced from the target's protected main)
  but runs **unprivileged**: `permissions: {}`, no secrets; its output is a strictly-validated
  JSON artifact. The privileged CLAIM job runs only registry-resident code. **No
  `pull_request`-triggered workflow in the registry ever carries secrets** — the registry has no
  fork PRs by construction (private), and the worker/dispatcher trigger set is exactly
  `schedule | workflow_dispatch | repository_dispatch` (all of which execute default-branch code).
- **B4 — worker output vs target main.** The PR must pass the target's `ci-summary` aggregate +
  branch protection + merge queue like any human PR. The App is **not** on any branch-protection
  bypass list, so its contents:write cannot land anything on `main` outside the gates.

### 2.1 The dispatch trigger, honestly

`repository_dispatch` against the private registry requires a caller credential with
`contents: write` on the registry (GitHub's minimum for that endpoint) — and the App key can't
live in the public repo (§1.2). Two mechanisms, layered:

- **Floor (build first): registry-side cron.** `dispatch.yml` on `schedule` (e.g. every 10
  minutes) polls each enabled target repo's ready frontier via the public API. Zero credentials in
  any public repo; latency is bounded by the cron period, which is small next to a model run's
  duration. Also gives `workflow_dispatch` for manual ticks.
- **Optional later: a narrow-PAT doorbell.** A target-repo workflow on
  `issues: [labeled]` (label = `status:ready`) fires `repository_dispatch` at the registry using a
  **fine-grained PAT scoped to only the registry repo with only contents:write** (the API
  minimum), stored as a target-repo secret. Preconditions and blast-radius honesty: (a) the
  registry's default branch must be **protected (PR-only)** first, so a leaked doorbell PAT cannot
  alter the code the registry's privileged scheduled/dispatch workflows execute (they run
  default-branch code); (b) residual exposure of a leaked doorbell PAT = read registry contents
  (scripts + ledger + account **metadata**; tokens are secrets and unreadable) + write non-default
  branches + ring the doorbell. (c) The dispatcher **ignores the dispatch payload entirely** — a
  doorbell is a wake-up, never a data channel; readiness is always recomputed from the API. Defer
  building this until cron latency is actually felt (§8 Q1).

## 3. The per-repo policy table

Lives in the **registry** (private — it names account pools and spend posture):
`policy/repos.toml`, consumed by a small pure resolver (`scripts/policy-resolve.py`, self-tested
like the other cores).

```toml
# policy/repos.toml — which target repos the private worker serves, and how.
# PRIVATE. No tokens here (tokens are secrets); this is the operational profile per target.

[repos."sparq-org/sparq"]
enabled                = true
routing                = "orchestration/routing.toml" # path IN the target repo (public, no
                                                      # account info): role → model_chain + agent
account_pool           = ["acct01", "acct02"]         # registry account issues this repo may claim
max_concurrent         = 2                            # repo-level cap (see the three layers below)
worker_timeout_minutes = 90                           # hard job timeout; hung run → killed + reclaimed
gate_profile           = "crate-scoped"               # none | lint-only | crate-scoped | workspace
arm_auto_merge         = true                         # arm `gh pr merge --auto` after opening; the
                                                      # target's merge queue + gates decide the rest
max_attempts           = 2                            # red-CI retry budget per issue → then needs:user
dispatch               = "cron"                       # cron | cron+doorbell (§2.1)
trust                  = "collaborators"              # trust-gate mode: write+ authors are trusted
```

Design points:

- **Routing stays in the target repo; policy lives in the registry.** `orchestration/routing.toml`
  (already merged, public, consumed by `route-resolve.py`) answers *"what model chain + which role
  agent for this kind of work?"* — that is knowledge about the work and carries no account
  information, so it belongs with the code it routes. The policy table answers *"which accounts,
  how many at once, which gates, arm or not?"* — that is spend/credential posture and stays
  private. The registry **filters** the target's model chain by the repo's `account_pool` (an
  account serves a model only if it is both in the chain and in the pool).
- **Role→agent routing is reused unchanged**: the plan row from `dispatch-plan.py` already carries
  `{number, priority, package, role, model_chain, agent, escalate}`. The worker maps `agent` to
  the matching role prompt (the `.claude/agents/*.md` file in the target checkout, passed to the
  harness as the system-prompt/agent selection) and `escalate=true` chain-exhaustion to
  `needs:user` instead of a weaker model — exactly the routing table's contract.
- **Concurrency is three-layered**, each reusing an existing mechanism:
  1. **per-account** — `max_concurrent_workers` in the account issue, enforced globally (across
     all codebases) by the CAS lease ledger;
  2. **per-repo** — `max_concurrent` here, enforced by the dispatcher counting its own live
     leases whose `holder` is prefixed by the target repo;
  3. **per-package** — `ready-issues.py`'s conflict partition (one in-flight issue per
     `area:<crate>`; a package-less issue takes the global partition), already merged.
  Plus a GitHub Actions `concurrency` group per `(target, issue)` on `worker.yml` so a duplicate
  spawn for the same issue can never run twice.
- **`gate_profile`** maps to the pre-PR local gate the worker runs in the target checkout:
  `crate-scoped` = fmt + clippy `-D warnings` + tests for the issue's `area:<crate>` packages
  (the same commands the role agents run today); `workspace` = the full-workspace gate;
  `lint-only`/`none` for doc-ish repos. Whatever runs locally, the **target's `ci-summary` stays
  authoritative** — the local gate only saves a doomed PR from burning target CI.
- **Scaling to repo N+1** = one new `[repos."owner/name"]` block + the target repo carrying the
  label taxonomy, a `routing.toml`, and its own branch-protection/CI backstop. No new tokens
  anywhere.

## 4. Security model

### 4.1 What can act, on what, with which credential

| Actor / step | Credential | Scope | Lifetime |
|---|---|---|---|
| Target-side triage/promotion workflows | default `GITHUB_TOKEN` (target) | issues:write on the target only | per-job |
| Dispatcher PLAN job | none (`permissions: {}`) | public API reads only | per-job |
| Dispatcher CLAIM job | registry `GITHUB_TOKEN` (contents:write) | the lease ledger CAS | per-job |
| App-token mint step | `REGISTRY_ADMIN_APP_ID/KEY` (registry secrets) | mint only; key never leaves the step | per-step |
| Issue status marks, checkout, push, PR, arm | App **installation token** | **one** target repo; contents:write, pull_requests:write, issues:write | ≤ 1 hour |
| Model harness step | claimed ACCT credential (isolated `$HOME`) + the App installation token | its own provider account; the one target repo | access token short-lived; App token ≤ 1 h |
| Lease release / grooming | registry `GITHUB_TOKEN` | ledger + registry issues | per-job |

The App installation token is minted per-run with `actions/create-github-app-token` (already
SHA-pinned in `verify-app.yml`) using `owner:` + `repositories:` narrowing — a worker for sparq
can never touch another installed repo even though the App's installation covers more.

### 4.2 Untrusted input (the primary control)

Only maintainer/collaborator-authored or maintainer-👍-promoted content is ever read by a model
(B1). This is the **primary** defense against prompt injection, because the model necessarily
runs with two capabilities that CI gates cannot fully police: its own account's access token and
push rights to non-protected branches of the target. The trust chain is: quarantine at arrival
(live) → positive `status:ready` attestation required for planning (merged) → last-step
re-verification in the worker (this build). Each link fails closed to no-model-action.

### 4.3 What a compromised/misbehaving model step could still do (residual, stated honestly)

- **Exfiltrate its own account's access credential** — it must hold it to run. Mitigation:
  `broker-refresh.py`'s access-token-only materialization (the long-lived refresh token stays in
  the registry secret; only a short-lived access token enters the worker `$HOME`) where the
  provider CLI supports running on a bare access token; where the CLI needs the full credential
  file (codex `auth.json`), the exposure is the stored credential and revocation is
  provider-side logout — this asymmetry is real and noted in §8 Q3.
- **Push junk branches / open junk PRs on the target** — bounded by B4 (protected main, required
  `ci-summary`, merge queue, no bypass) plus the `max_attempts` budget (§3) so a red-CI loop
  cannot burn unbounded model spend or CI minutes.
- **Egress anything it can read in its sandbox** — Actions runners have open egress; we do not
  claim otherwise. What it can read is bounded by B2 (no registry-admin or sibling-account
  material in its environment).

### 4.4 Account tokens and refresh (the operational concern)

The ACCT secrets are provider **OAuth credential files** (codex `~/.codex/auth.json` shape:
`{auth_mode, OPENAI_API_KEY, tokens{access,refresh,id}, last_refresh}`; Claude Code
`~/.claude/.credentials.json` shape: `claudeAiOauth{accessToken, refreshToken, expiresAt}` — or
alternatively an `ANTHROPIC_API_KEY`). The worker materializes the claimed credential into an
isolated `$HOME` (never a shared/live home) using the already-written `broker-refresh.py` cores,
and the provider CLI refreshes its own access token on demand — no reverse-engineered OAuth.

**Known concern — refresh-token rotation:** if a provider rotates the refresh token on use, the
registry's stored secret goes stale after a run and the *next* run fails auth. Two facts are
unverified until the canary observes them (deliberately not asserted here): whether codex and/or
Claude Code rotate refresh tokens per-refresh, and how long each provider's access token lives.
The plan handles both outcomes: the worker's `always()` epilogue diffs the materialized credential
file against what it restored; if changed, it **writes the secret back**. Secret write-back needs
a credential with secrets:write on the registry — the registry `GITHUB_TOKEN` cannot do it, so
this is either `REGISTRY_ADMIN_TOKEN` (the fine-grained PAT the enrollment broker already
expects) or installing the App on `jeswr` with repo-secrets write (§8 Q2). Until wired, a stale
credential fails visibly: auth failure → `status:deferred` + an alarm issue in the registry —
never a silent hang.

Session caps are handled as data, not errors: the known CLI cap signature (a fast exit with a
"session limit … resets <time>" message) → `status:deferred` + lease release + a retry on a later
tick, letting `select-and-claim` fall through the model chain to another account when one exists.

## 5. Implementation plan

### 5.1 New files, per repo

**In `jeswr/agent-account-registry` (private):**

| File | What it is |
|---|---|
| `policy/repos.toml` | the per-repo policy table (§3) |
| `scripts/policy-resolve.py` | pure resolver + `--self-test` (same pattern as `route-resolve.py`) |
| `scripts/worker-prep.sh` | credential materialization into an isolated `$HOME`, reusing `broker-refresh.py` cores; plus the last-step trust re-verification call |
| `.github/workflows/worker.yml` | the worker job (§2 steps 1–8); `workflow_dispatch`-only inputs `{target_repo, issue, model, agent, secret_ref, claim_id, dry_run}`; `dry_run` defaults **true** |
| `.github/workflows/dispatch.yml` | the dispatcher: `schedule` + `workflow_dispatch` (+ a `repository_dispatch` trigger that ignores its payload); PLAN job (unprivileged) → CLAIM+LAUNCH job (privileged) |
| `groom-issues` step (extends `groom-leases.yml`) | resets stale `status:in-progress` (lease expired + no open PR) → `status:ready`; enforces `max_attempts` → `needs:user`; sweeps stale bot PRs |

**In `sparq-org/sparq` (public):** nothing required for the cron floor. Optional later:
`.github/workflows/dispatch-doorbell.yml` (§2.1) once its preconditions are met. One docs PR
updates `orchestration-activation-runbook.md` to point at this record and record canary results.

### 5.2 Reused as-is (no changes)

- **Target-side, merged:** `scripts/ready-issues.py`, `scripts/trust-gate.py`,
  `scripts/triage.py` + `scripts/promote.py` + `triage-issue.yml` + `promote-on-approval.yml`,
  `scripts/route-resolve.py`, `scripts/dispatch-plan.py`, `orchestration/routing.toml`,
  `scripts/bd-to-issues.py` (tracker migration — still gated on the maintainer's go-ahead; the
  canary does not require the bulk migration).
- **Registry-side, live:** account issues #1/#2 + `ACCT01_TOKEN`/`ACCT02_TOKEN`,
  `scripts/select-and-claim.py` + `data/leases.json` + `groom-leases.yml`,
  `scripts/broker-refresh.py`, `scripts/account-login.sh` + `set-up-account.yml` (enrollment),
  `verify-app.yml` + the App secrets, `data/cache-affinity.json`.

### 5.3 Phase order — each item one small PR (a future bead)

1. **REG-1 — policy substrate.** `policy/repos.toml` + `scripts/policy-resolve.py` (+ README
   section). Pure, no triggers, self-tested. Unblocks everything.
2. **REG-2 — the worker, dry.** `scripts/worker-prep.sh` + `.github/workflows/worker.yml` with
   `dry_run` defaulting true (claims nothing live on dry: synthetic lease, App checkout, **no**
   model, no PR — proves steps 1–3 + release). Manually triggered only.
3. **REG-3 — worker, live path.** Flip the guarded live path in `worker.yml`: real claim, model
   step, gate profile, PR + arm-per-policy, `always()` release + status epilogue + credential
   write-back check. Still `workflow_dispatch`-only.
4. **REG-4 — the dispatcher.** `dispatch.yml` (PLAN unprivileged → CLAIM+LAUNCH privileged),
   cron **disabled** initially (`workflow_dispatch` only); enable the schedule after the canary.
5. **REG-5 — grooming.** The `groom-issues` extension (stale in-progress reset, `max_attempts`
   budget, stale-PR sweep).
6. **CANARY — §6 executed end-to-end**, results recorded in the sparq runbook doc PR; then enable
   the dispatch cron with `max_concurrent = 1` for a burn-in period.
7. **Optional, later:** SPARQ-1 doorbell (`dispatch-doorbell.yml` + registry default-branch
   protection first); REG-6 secret write-back credential decision (§8 Q2); repo N+1 onboarding
   (one policy row, proving the scaling claim).

Dependencies: REG-1 → REG-2 → REG-3 → REG-4 → CANARY; REG-5 parallel after REG-1; doorbell and
write-back independent afterwards.

## 6. Canary and failure modes

### 6.1 Canary (a labeled throwaway issue, end-to-end)

1. **Dry tier:** run `worker.yml` with `dry_run=true` on a synthetic row — verifies claim/release
   bookkeeping, App-token checkout of sparq, and that no secret fragments appear in the run log.
2. **Live canary:** the maintainer (or the session bot — a trusted author) opens a throwaway sparq
   issue: trivially verifiable and doc-scoped (e.g. "fix a stated typo in one named doc"), labeled
   `role:docs`, `priority:P4`, `status:ready`, an appropriate `area:*`, plus `canary`. Then
   observe, in order:
   - the dispatcher tick lists it in `plan.json` and CLAIM writes a lease (a `claim_id` visible in
     `data/leases.json`) + `status:in-progress`;
   - a `worker.yml` run starts, the model runs headless, the local gate passes;
   - a PR opens on sparq authored by `sparq-orchestrator[bot]`, body led by the 🤖 SPARQ-agent
     blockquote, `Fixes #<issue>` linkage, auto-merge armed (policy);
   - the target's `ci-summary` goes green, the merge queue merges, the issue auto-closes;
   - the lease is **released** (ledger empty again), `cache-affinity.json` updated;
   - the credential write-back check reports whether the stored credential rotated (§4.4 — this is
     where the rotation question gets its answer);
   - the run log contains no unmasked token material (grep the log for credential substrings).
3. **Failure drills (each must degrade, not break):**
   - set the account's `max_concurrent_workers` to 0 → expect `none-free` → `status:deferred`,
     no worker;
   - cancel a live worker mid-run → expect the lease reclaimed by `groom-leases` within its cron
     period and the issue reset to `status:ready` by `groom-issues`;
   - open the same canary as a **third-party** author → expect quarantine (`trust:untrusted`),
     never planned, never dispatched;
   - make the model's change fail the target gate → expect an unmerged red PR, one retry, then
     `needs:user` at the `max_attempts` budget.

Acceptance = all of the above observed; only then enable the cron.

### 6.2 Failure modes (honest)

| Failure | Detection | Response |
|---|---|---|
| LLM/provider offline or erroring | harness exits non-zero quickly | `status:deferred` + comment + lease release; later ticks retry (never crash-looped) |
| Account at session/weekly cap | known CLI cap signature in output | as above; `select-and-claim` may serve the chain from another pooled account |
| Hung worker | `worker_timeout_minutes` kills the job; lease `expires_at` passes | `groom-leases` reclaims the slot; `groom-issues` resets the issue to `status:ready` |
| Model output fails target CI | red `ci-summary` on the PR | PR sits unmerged (backstop); retry within `max_attempts`, then `needs:user` + close the stale PR |
| Stored credential goes stale (rotation) | auth failure at materialization / CLI start | `status:deferred` + a registry alarm issue; fixed by write-back (§4.4) or re-enrollment via the login broker |
| Prompt injection via third-party content | trust chain (B1) at three links | quarantined; never read by a model; maintainer 👍 is the only promotion path |
| Doorbell spam (if built) | n/a — payload ignored by design | wasted dispatcher ticks only; concurrency group + caps bound it |
| Duplicate dispatch of one issue | lease CAS + per-issue Actions concurrency group | second claim loses the CAS / second spawn queues and no-ops on the `status:in-progress` check |
| Two dispatcher ticks overlap | Actions `concurrency` group on `dispatch.yml` | serialized; the lease CAS is the true mutex regardless |

The `in-progress` label is advisory bookkeeping; the **lease is the mutex**. Anything that dies
frees itself by expiry, and nothing merges anywhere except through the target repo's own gates.

## 7. The documented future option (not designed here)

If sustained volume ever makes metered private-repo minutes material: move the model step to
**free public-runner workers in each target repo**, fed short-lived credentials by a **private
broker** (the OIDC-validating allocator shape sketched in the superseded token-placement section
of [`issue-native-orchestration.md`](issue-native-orchestration.md)). That re-introduces a hosted
component and a public-side credential surface, which is exactly what today's decision avoids —
so it is recorded as the escape hatch, with the trigger being cost observed in practice, not
designed speculatively.

## 8. Open questions for the maintainer

1. **Doorbell now or later?** The cron floor ships first; is dispatch latency of one cron period
   acceptable for the foreseeable future (recommended: yes, defer the doorbell)?
2. **Secret write-back credential (§4.4):** `REGISTRY_ADMIN_TOKEN` fine-grained PAT (secrets:write
   on the registry), or install the App on `jeswr` with repo-secrets permission? The App route
   avoids another long-lived PAT but widens the App's footprint.
3. **codex credential shape:** the codex CLI wants the full `auth.json` (including the refresh
   token) in its `$HOME`, which weakens the access-token-only posture for `acct01` specifically.
   Accept for now (single account, provider-side revocation), or gate codex-harness work until an
   access-token-only run mode is verified?
4. **Arm posture at switch-on:** `arm_auto_merge = true` from the first live tick (locked goal is
   full autonomy), or a short burn-in with arming limited to `role:docs` while the canary evidence
   accumulates? The policy knob supports either per repo.
5. **Tracker migration timing:** the canary runs on hand-labeled issues without the bulk
   `bd-to-issues.py` migration; sustained autonomy wants the migration executed (~895 issues) —
   still gated on your go-ahead per the activation runbook.
