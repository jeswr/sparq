# Issue-native self-managing orchestration — design record

**Status:** design + Phase 0 landing. Supersedes the human-in-the-loop model of
[`orchestration-automation-design.md`](orchestration-automation-design.md) and the EC2/bd-based
[`autonomous-scheduler-design.md`](autonomous-scheduler-design.md) for the dispatch/merge path.
Maintainer-directed 2026-07-13. `[OPUS-4.8]`

## Why

The PR-drain has been run by a human/model orchestrator on a `/loop`: sync main, pick disjoint ready
beads, spin workers, review, merge. Two structural problems force a redesign:

1. **`bd` isn't in git.** `.beads/` is a gitignored Dolt DB living only in one checkout — CI can't
   read/write it and there's no durable multi-actor source of truth. → **GitHub issues become the
   tracker.**
2. **Dispatch/routing/merge were deliberately manual** (see `orchestration-automation-design.md`
   §1.3, which kept these with a human because a runaway auto-merge is the highest-severity failure).
   The maintainer now wants them automated, with **CI as the correctness backstop** and one new
   safeguard against **untrusted third-party input**.

## Shape (all on free infra)

A new issue (from the maintainer or the system) is auto-labeled `model/role/package/priority`; a cron
+ (later) issue-opened workflow dispatches workers in priority order. Each worker is a **GitHub
Actions job on the free standard runner in the public sparq repo**; it asks a **private account
registry** for a model account (respecting per-account limits + a cross-codebase reaction-lock +
prompt-cache affinity), implements on a branch off the merge-queue tip, and opens a **draft** PR. A
**batcher** clusters disjoint draft PRs into one CI run and merges; a **groomer** keeps the backlog +
fleet healthy. No manual per-item orchestration.

## Cost model (non-billable)

- **Workers run on the FREE standard `ubuntu-latest` runner** in the **public** sparq repo — public
  repos get unlimited free standard-runner minutes. **No billable larger runners.** RAM is the
  free-tier bound (~16 GB); crate-scoped builds fit; a rare over-16 GB task is a manual exception, not
  a reason to pay.
- **Private-repo Actions minutes are kept ~zero.** The registry is read via **API** (issues,
  reactions, files, secrets) from the public worker using the automation token — API reads cost no
  Actions minutes. Heavy compute never runs in the private repo.

## Tracker: GitHub issues (full migration from bd)

bd's features are reproduced on issues + workflows:

| bd feature | issue-native replacement |
|---|---|
| dependency edges | GitHub **native issue dependencies** (blocked-by/blocks); body-marker `Blocked-by: #NN` fallback |
| `bd ready` | `scripts/ready-issues.py`: open ∧ no open blocker ∧ no `needs:*`/`trust:untrusted` ∧ not in-flight ∧ no package-conflict, priority-ordered |
| priority (0–4) | `priority:P0..P4` labels |
| type/labels | native labels (`role:`, `model:`, `status:`, `trust:`) + existing `area:<crate>` = **package** |
| conflict-partition (push-frontier) | package = `area:<crate>` label; one in-progress worker per package |
| auto-close on merge | issue-native retarget of `bead-autoclose.yml` |
| `sq-…` PR tokens | preserved via a `sq-… ↔ #NN` map written at migration |

**Label taxonomy** (created in Phase 0): `priority:P0..P4` · `role:{impl,review,docs,research,perf,ci,site,soundness}`
· `status:{ready,in-progress,blocked,deferred,untriaged}` · `trust:untrusted` · `self-improvement`
(agent-discovered out-of-scope work — bug/tech-debt/doc-drift/footgun/better-approach — for the
self-improvement triage lane; see [`agent-observability-and-self-improvement.md`](agent-observability-and-self-improvement.md))
· package = existing
`area:<crate>` · `model:*` — the **models** are `haiku,sonnet,opus,fable,opus5` (Anthropic/Claude,
via the `claude` CLI) and `terra` (OpenAI/GPT, via the `codex` CLI). **`codex` is the HARNESS, not a
model** (as `claude` is the harness for Claude models); the selector resolves a model → (account,
harness).

**Routing** (`orchestration/routing.toml`, public — no account info): `(role/label) → (model-chain,
agent)` with first-class **model fallback chains**. Every chain token resolves to a concrete
provider model id via the table's `[models]` catalog — tokens are never re-pointed at a different
model. Per maintainer doctrine (updated 2026-07-24), **Opus 5 (`claude-opus-5`) is the primary
code-writer, used heavily**: it enters as the NEW `opus5` token (catalog entry
`provider_model = "claude-opus-5"`) at the head of every chain that previously led with `fable` or
`opus`. The `fable`/`opus` tokens keep resolving to `claude-fable-5`/`claude-opus-4-8` and remain in
the chains solely as tail/downgrade fallbacks — downgrade-authored work is stamped with its actual
tier and tagged for re-review under Opus 5; `haiku`/`sonnet` do cheap or mechanical work; `terra`
(GPT) is a cross-provider implementation fallback. The registry consumes the chain to pick the first
available account.

## Private account registry (separate private repo `jeswr/agent-account-registry`)

**No public repo ever holds account handles, limits, usage, tokens, or selection logic.** The
registry (its own README documents the full schema):
- **One issue per account** (body = YAML: provider, tier caps, reset schedule, `max_concurrent_workers`,
  `secret_ref`). **Tokens live only in GitHub secrets** named by `secret_ref`.
- **Lease mutex (cross-codebase) — CAS ledger, not reactions.** The GPT-5.6 review
  ([issue-native-orchestration-review-gpt56.md](issue-native-orchestration-review-gpt56.md), C3)
  showed reaction-counting cannot be a mutex (GitHub allows one reaction per identity, so same-bot
  workers all see one 🚀). Replaced with a **compare-and-swap lease ledger** (`data/leases.json` in the
  private registry): claim = read the file + its blob SHA, reclaim expired leases, append a lease with
  a **unique `claim_id` + `expires_at`** if a slot is free, then `PUT` with the read SHA (a concurrent
  writer changed the SHA → 409 → retry). Every codebase CAS-updates the SAME ledger, so capacity is
  global. Release + heartbeat are keyed by `claim_id`; the groomer reclaims by expiry.
- **`select-and-claim`** (`scripts/select-and-claim.py`, private, self-tested): walk the model chain →
  filter by cap + reset → prefer **prompt-cache affinity** (most-recent same-`package+role` within the
  ~5-min Anthropic cache TTL) → CAS-claim → return `{account, secret_ref, claim_id}`.
- **`deferred`:** all eligible accounts full → the sparq issue is labeled `status:deferred`; a later
  tick retries.

## DECISION NEEDED — model-token placement

The public worker needs the chosen account's **token** to run the agent. Two options (maintainer
sign-off before real tokens are wired):

- **(A) Masked public-repo secret (recommended, free).** Store each account token as a secret in the
  public sparq repo; the worker reads it by handle via `toJSON(secrets)` (so no handle appears in the
  workflow file). GitHub secrets are encrypted at rest, **masked in logs**, and **blocked from
  fork-PR workflows**. Account *metadata + selection* still live only in the private repo; only the
  opaque token value sits (masked) in the public repo. Zero extra infra, fully free.
- **(B) Private proxy.** A small service (the bot) holds the tokens and proxies model calls so the
  token never touches the public repo. Strongest isolation, but needs a running service and complex
  agent-CLI plumbing (agents expect direct provider access).

Recommendation **REVISED after the GPT-5.6 review (S1/D4): (B) the private broker.** The review shows
(A) is **not an adequate isolation boundary** — an autonomous, repo-writing agent + `toJSON(secrets)`
can exfiltrate *every* repo secret; masking is log-redaction, not DLP; `pull_request_target` /
same-repo branches defeat fork-withholding. Adopt (B): a private allocator/broker that validates the
public worker's GitHub **OIDC** identity, atomically grants a **lease**, and returns a **short-lived,
model-scoped, single-claim** capability (or proxies the model call) — no long-lived provider/registry/
admin credential ever sits in the public repo. **Interim remediation already done:** the seeded
account tokens were moved out of the public repo into the private registry. This decision awaits the
maintainer's final sign-off before Phase 3 wires worker credentials.

## Automation identity + the trust safeguard

- **Identity.** The automation must act as a stable identity the trust-gate can recognize and that can
  **trigger downstream workflows** (the default `GITHUB_TOKEN` cannot trigger further workflows). Options:
  a **GitHub App** (recommended; scoped, rotatable, own login) or a **fine-grained PAT** on a machine
  account. Requires a one-time maintainer setup (App creation isn't fully CLI-scriptable); until then a
  maintainer PAT secret is the stopgap.
- **Trust is derived from repo permission (no hard-coded logins).** `scripts/trust-gate.py` calls
  `repos/OWNER/REPO/collaborators/USER/permission`: **write / maintain / admin** (anyone who could
  already push code) → trusted; the automation identity → trusted; everyone else (read / triage /
  none = third-party) → untrusted. This auto-tracks the repo's collaborators/maintainers.
- **Untrusted-input safeguard (fails closed).** trust-gate is consulted before any model reads
  issue/PR/comment CONTENT. Third-party content → label `trust:untrusted`, **no** model action,
  and `notify-maintainer` @-mentions you ("react 👍 to approve"). **Your 👍 promotes it**
  (`promote-on-approval` un-quarantines) — the ONLY path by which third-party input reaches a model.
  This defends against prompt-injection via third-party issues / PR comments.

## Graceful LLM-offline degradation

Every LLM step tolerates the model being offline/capped by setting a **retry state**, never erroring:
triage failure → `status:untriaged` (a `retriage.yml` cron re-picks it); dispatch with no account →
`status:deferred`; grooming → re-queue. No outage drops work.

## CI economy (batch → bisect-only-on-failure → debounce)

1. **Batch:** workers open **draft** PRs (not individually CI-armed); a batcher clusters disjoint
   (package-partitioned → conflict-free) branches into **one** integration PR → **one** CI run; green
   → merge all. Collapses N CI runs into 1.
2. **Bisect only on a meaningful (non-flake) failure:** binary-split the batch, quarantine the culprit
   (issue → `ready` + note), merge the passing subset. Individual merge-queue entries happen only here.
3. **Debounce main CI:** while merges are actively landing (a PR opened/merged in the last 30–60 min),
   defer the expensive main-branch CI to a quiet window — no start-then-cancel churn.

The existing `ci-summary / gate` + branch protection remain the correctness backstop, unchanged.

## Phases

0. Foundations — registry repo, labels, routing table, trust-gate, this record. **(this PR)**
1. Migrate bd → issues + `ready-issues.py`.
2. Auto-labeling (`triage-issue.yml`) + trust workflows (`notify-maintainer`, `promote-on-approval`, `retriage`).
3. Dispatch engine (`dispatch-fleet.sh`) + the free-runner worker + private `select-and-claim`.
4. Batched merge (`batch-merge.yml`) + main-CI debounce + issue auto-close.
5. Grooming agent.
6. Fully Actions-native issue-opened trigger.

## What needs the maintainer

- **Account tokens** (paste into private-repo secrets; I can seed the one local `~/.codex` account).
- **Automation identity** (App install or a machine-account PAT).
- **Token-placement sign-off** (A vs B above).
