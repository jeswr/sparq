# Adversarial design and security review

Date: 2026-07-14  
Scope: every file supplied under `sparq/` and `registry/`.

## Executive verdict

**Do not enable autonomous dispatch or place real provider credentials in the public repository yet.** The design has useful foundations, but the supplied implementation is still Phase 0/1 scaffolding and several of its claimed safety properties do not hold:

- The recommended public-secret design gives an autonomous, repo-writing agent access to long-lived provider credentials and, via `toJSON(secrets)`, potentially every repository secret (`sparq/design-record.md:86-102`). Fork withholding does not protect issue/schedule workflows, same-repository branches, `pull_request_target`, or code and workflow changes made by a compromised agent.
- The trust policy is fail-closed inside the small `verdict()` function, but the pipeline around it is fail-open: readiness requires only the *absence* of `trust:untrusted`, and it dispatches unlabeled and `status:untriaged` issues (`sparq/ready-issues.py:22-23`, `sparq/ready-issues.py:72-80`). Approval and permission are unauthenticated command-line assertions (`sparq/trust-gate.py:85-100`), and approval is not bound to a content revision.
- The migration cannot migrate anything: `--apply` prints a message and returns success (`sparq/bd-to-issues.py:123-128`). Its separate dependency-record parser also drops a normal `issue_id`/`depends_on_id` record (`sparq/bd-to-issues.py:38-43`).
- The reaction mutex is not a mutex. GitHub permits one reaction of a given type per user; a repeated create returns the existing reaction. Multiple workers using the same stable bot can therefore all observe one reaction and all believe they own a slot (`registry/README.md:32-40`, `registry/README.md:61-62`).
- The broker writes the Anthropic token to a raw login log before extracting it (`registry/account-login.sh:51`, `registry/account-login.sh:67-69`), has no explicit cleanup, installs unpinned code in the credential-bearing job (`registry/set-up-account.yml:29`, `registry/set-up-account.yml:47-53`), and assumes one fine-grained PAT can operate across two resource owners (`registry/set-up-account.yml:4-7`, `registry/set-up-account.yml:81-90`).

The public standard-runner cost premise itself is sound as of this review: GitHub documents public-repository standard runners as free/unlimited and `ubuntu-latest` as 4 vCPU/16 GB RAM/14 GB SSD. That does not make the whole pipeline non-billable or secure; provider usage, storage, private-repository setup runs, concurrency limits, and abuse/rate limits remain.

## Top bugs and gaps to fix before go-live

| Rank | Severity | Problem | Required fix |
|---|---|---|---|
| 1 | Critical | Long-lived model and registry credentials cross into a public repo job that runs an autonomous agent and repo-controlled code (`sparq/design-record.md:86-102`). | Choose the proxy/broker boundary, issue a short-lived single-claim capability, and keep provider tokens plus registry administration credentials out of all public-repo jobs. Do not pass `toJSON(secrets)` to any process. |
| 2 | Critical | Trust is negative-label based and therefore fail-open on new, unlabeled, untriaged, or gate-failure issues (`sparq/ready-issues.py:22-23`, `sparq/ready-issues.py:72-80`). | Require positive, bot-attested `status:ready` and `trust:trusted`/revision-bound approval states plus required role/model/package metadata at the final dispatch step. Serialize trust+claim+dispatch. |
| 3 | Critical | Prompt injection is not contained by author permission or a thumbs-up (`sparq/design-record.md:111-119`; `sparq/trust-gate.py:30-40`). | Gate every content object and all transitively fetched data, bind approval to an immutable digest, invalidate it on edit, and run the model with no raw secrets and narrowly scoped write tools. Treat promotion as consent to inspect hostile data, not as proof that it is safe instruction text. |
| 4 | Critical | `bd-to-issues.py --apply` is a successful no-op, so no issues, map, native dependencies, or parent links are created (`sparq/bd-to-issues.py:91-128`). | Implement a resumable two-pass migration with durable source IDs, recovery checkpoints, verification, and an end-to-end test on a copied tracker before changing production. |
| 5 | Critical | Reaction counts cannot distinguish concurrent claims by the same automation identity and the check/add/recount sequence is not a linearizable capacity allocation (`registry/README.md:32-40`, `registry/README.md:61-62`). | Replace it with a transactional lease service/ledger or one serialized central allocator using compare-and-swap. Return a unique claim ID with expiry and require it for release/renewal. |
| 6 | Critical | The login job executes unpinned checkout/provider code before and during credential creation, while all steps share one runner/user (`registry/set-up-account.yml:29`, `registry/set-up-account.yml:47-53`). | Pin actions to commit SHAs and CLI packages to verified versions/checksums; remove the curl-to-shell fallback; isolate login and secret storage behind a minimal reviewed broker/protected environment. |
| 7 | High | Native GitHub blocked-by dependencies are ignored, the fetch is capped at 1,000, and multi/no-package issues defeat conflict partitioning (`sparq/ready-issues.py:40-45`, `sparq/ready-issues.py:131-146`). | Fetch all issues and native dependency nodes with pagination; fail closed on incomplete data; model a set of touched partitions, with a global partition for unknown/cross-cutting work. |
| 8 | High | A single fine-grained PAT cannot be scoped to both `jeswr/agent-account-registry` and `sparq-org/sparq`; the workflow also uses the target token for registry issue creation and then invokes `gh` without `GH_TOKEN` (`registry/set-up-account.yml:81-90`). | Use the registry job's `github.token` for registry issues and a separate short-lived target-repo App installation token solely for secret creation. Do not use a cross-owner PAT. |
| 9 | High | Aggregate CI results do not automatically satisfy required checks on the original PR heads; binary bisection is unsound for interacting or multiple failures (`sparq/design-record.md:127-137`). | Make the tested integration commit the sole merge unit, or post a required check tied to each exact merge candidate and retest the final queue commit. Handle multi-failure/interaction cases and impose a maximum debounce delay. |
| 10 | High | Routing comments promise security precedence and a `terra -> OpenAI -> codex` mapping that the TOML does not encode; registry model names conflict with it (`sparq/routing.toml:7-16`, `sparq/routing.toml:67-72`; `registry/README.md:17-25`). | Add a validated model catalog and explicit route precedence. Use canonical provider model IDs separately from internal aliases and harness names. Add routing tests for every taxonomy label and fallback-exhaustion case. |

## 1. Security review

### S1 — Critical: masked public-repository secrets are not an adequate isolation boundary

The recommendation relies on encryption at rest, log masking, and fork-PR secret withholding (`sparq/design-record.md:91-101`). Those properties are real but do not address the relevant attacker:

1. A normal fork `pull_request` does not receive Actions secrets, but a `pull_request_target` workflow does; checking out or executing fork code there exposes them. Issue, comment, schedule, dispatch, and push events are not fork-PR events at all.
2. Workers create branches in the base repository (`sparq/design-record.md:23-29`). A same-repository PR is not protected as a fork. More importantly, even if workflow YAML comes from a trusted ref, tests, build scripts, actions, compiler hooks, or other repo files from an agent branch can encode or transmit secrets.
3. An autonomous actor with repository write access can propose or land workflow/build changes. A compromised maintainer, App, dependency, or model can exfiltrate a token by transforming it (base64, chunking, encryption, network requests); masking is log redaction, not data-loss prevention.
4. `toJSON(secrets)` deliberately materializes every available secret, not only the selected account token (`sparq/design-record.md:91-95`). One compromised worker therefore expands from one account to the entire secret set. It also exposes secret names/account references to the process.

GitHub's own documentation says repository secrets are available to all workflows in the repository and are merely withheld from fork `pull_request` runs; it separately warns that `pull_request_target` receives base-repo secrets. See [secret types](https://docs.github.com/en/code-security/reference/secret-security/secret-types) and [`pull_request_target` security](https://docs.github.com/en/actions/reference/security/securely-using-pull_request_target).

**Fix:** use the private proxy/broker option. The public worker should receive a short-lived, audience-bound, one-claim credential that can invoke only the selected model, with a hard token/budget/time ceiling and no registry access. The broker holds provider credentials. Independently prevent the worker identity from modifying workflow/security paths; use CODEOWNERS plus branch/ruleset enforcement, pin all actions, disable `pull_request_target` for code execution, and ensure secret-bearing jobs never check out or execute worker/PR-controlled code. Environment approval can reduce risk but contradicts full autonomy and is not a substitute for the broker.

### S2 — Critical: the trust gate accepts unverified assertions and dispatch is fail-open

`fetch_permission()` is sensibly fail-closed on API failure (`sparq/trust-gate.py:47-55`), and treating effective `write`/`maintain`/`admin` as trusted is internally consistent (`sparq/trust-gate.py:27-40`). The enforcement boundary is nevertheless bypassable if any caller-controlled value reaches the CLI:

- `--permission write`, `--maintainer-approved`, and `--bot <author>` directly assert every trust fact; the script verifies none of them (`sparq/trust-gate.py:85-100`). `--repo` is also caller-selected, so a user with write access to some other repo could be reported trusted if the workflow passes an untrusted repo value.
- Nothing verifies who supplied the thumbs-up, that it is a thumbs-up, or that it still exists. A maintainer's approval is not bound to the issue/comment body hash. GitHub authors can edit their issue or comment after approval and retain the reaction.
- `ready-issues.py` checks only for `trust:untrusted` (`sparq/ready-issues.py:22`, `sparq/ready-issues.py:48-49`). If the trust workflow is delayed, fails, is disabled, or loses a race with dispatch, absence of the quarantine label is treated as trust. `status:untriaged` is also not busy/gated (`sparq/ready-issues.py:23`, `sparq/ready-issues.py:72-80`).
- Labels are mutable state, not an attestation. A label removal can make content ready without passing the gate.

**Fix:** production mode should accept only immutable event identifiers, hard-bind the repository to `github.repository`, fetch the author/permission/reactions itself with a minimally scoped token, and validate that an authorized maintainer approved the exact `{object-id, updated_at/body-digest}`. On `issues.edited`, `issue_comment.edited`, PR synchronize, or review edit, invalidate approval. Dispatch must require positive bot-owned state (`trust:trusted` or a signed check/attestation) and `status:ready`; absence or API incompleteness must deny. Remove raw assertion flags from production or hide them behind an explicit test-only mode.

### S3 — Critical: the gate is not a sound prompt-injection defense

The design equates “can push code” with “safe for a model to read” and says approval is the only path by which third-party input reaches a model (`sparq/design-record.md:111-119`). This is an authorization policy, not prompt-injection containment:

- A write collaborator can push repository code but normally cannot read provider tokens, private registry data, or every secret. Letting their text control a secret-bearing agent grants materially more authority than ordinary write access.
- A promoted third-party body remains adversarial text. A thumbs-up means a maintainer chose to process it; it does not make instructions embedded in it benign.
- Event-author gating does not cover nested/transitive content. A trusted maintainer comment can ask the model to review an external PR, after which the model reads the untrusted PR title/body/diff, comments, CI logs, linked issues, web pages, or files. Each may contain instructions.
- Unconditionally trusting configured bot authors (`sparq/trust-gate.py:33-35`) permits provenance laundering when a bot quotes, summarizes, or copies third-party content.

**Fix:** keep provenance per content object and enforce it in the tool/data layer, not only in the prompt. Give the model a delimited, non-instruction data view of approved content; do not let it freely fetch comments/URLs/logs. Require separate authorization for side effects, prohibit access to secret values, restrict network egress, and constrain repo writes to a branch and allowed paths. Add deterministic policy checks to reject workflow, auth, secret, ruleset, and orchestration changes unless separately approved. Revalidate authority before push/PR/merge. A second, isolated reviewer should inspect security-sensitive diffs without inheriting the writer's untrusted conversation.

### S4 — High: the login broker records a live token and does not clean the runner

The statement that the token is “NEVER” logged is false (`registry/account-login.sh:5`; `registry/README.md:83-91`):

- `claude setup-token` prints the long-lived token, and stdout/stderr are redirected wholesale into `$OUTDIR/.login.out` (`registry/account-login.sh:47-51`). The token is then recovered from that log (`registry/account-login.sh:67-69`). Thus the token is logged to disk once before it is copied to the mode-600 token file. The log's containing directory is mode 700, which limits access, but the raw log is neither explicitly mode 600 nor deleted.
- The shell wrapper itself prints only status words (`OK`, `FAILED`, or `TIMEOUT`), and the workflow comments only `signin.txt`; it does not intentionally echo the raw provider credential to the Actions console (`registry/account-login.sh:39-45`, `registry/account-login.sh:64-72`; `registry/set-up-account.yml:63-69`). The posted URL/device code is not the resulting token, but it is a short-lived bearer enrollment capability visible to every registry reader.
- OpenAI credentials and the Anthropic fallback credentials remain under the runner's real `$HOME` (`registry/account-login.sh:42-43`, `registry/account-login.sh:70-71`). `LOGIN_DIR`, `$LOG`, `$TOKEN`, and provider credential directories have no `always()` cleanup step (`registry/set-up-account.yml:55-98`). On failure, timeout, missing `REGISTRY_ADMIN_TOKEN`, or cancellation, sensitive material persists until GitHub destroys the VM and is available to all later steps and post-step hooks in the job.
- The generated provider credential is not yet a GitHub Actions secret while it is captured, so it is not automatically masked as that secret. `REGISTRY_ADMIN_TOKEN` is injected from `secrets` and is masked, and `gh secret set ... < "$LOGIN_DIR/token"` correctly keeps the provider credential out of argv/stdout (`registry/set-up-account.yml:73-81`). Those good details do not protect a transformed value, raw local log, malicious same-job process, or future accidental `cat`.
- OpenAI stores an entire `auth.json`, while Anthropic may store either a raw token or an entire credentials JSON (`registry/account-login.sh:42-43`, `registry/account-login.sh:67-71`). Registry metadata records no credential format (`registry/README.md:17-26`), so a worker cannot reliably know whether to export an environment token or reconstruct a CLI credentials file.

GitHub-hosted VMs are ephemeral and are decommissioned after the job, which is useful eventual cleanup, but it is not isolation between steps in the same job. GitHub also warns that structured secrets are harder to redact reliably; see [GitHub-hosted runners](https://docs.github.com/en/actions/reference/runners/github-hosted-runners) and the [secrets reference](https://docs.github.com/en/actions/reference/security/secrets).

**Fix:** set `umask 077`; run with a dedicated temporary `HOME`/`CODEX_HOME`; never persist unredacted provider output; parse a stream/FIFO into a 0600 token file while writing only a redacted diagnostic log. Register runtime masks immediately for any scalar credential, while still assuming masking can fail. Add a final `if: always()` cleanup step that kills/waits for login children and removes `LOGIN_DIR`, temporary home, Codex auth, and Claude credentials. Do not continue to later steps after an unstored credential. Define `credential_format` and restore only the minimum credential needed by the matching harness. Prefer the broker architecture so no credential file crosses into an untrusted worker at all.

### S5 — Critical: the credential-bearing setup job has a supply-chain and trigger problem

The job runs `actions/checkout@v4`, latest unpinned npm packages, and an unverified curl-to-shell fallback (`registry/set-up-account.yml:29`, `registry/set-up-account.yml:47-53`). A malicious install can leave a background process on the shared runner and steal both the freshly generated provider credential and a later step's admin environment, even though `ADMIN` is scoped to that later step. `|| true` also converts installation failure into a delayed, confusing login failure.

The trigger checks the *issue author* but not the actor applying the label (`registry/set-up-account.yml:19-23`). A collaborator who can label an old owner-authored issue can start a login. The sign-in URL/code is posted to an issue visible to every reader of the private registry (`registry/set-up-account.yml:63-67`); a reader can race the maintainer and authorize a different account. No post-login check proves which provider account was authorized.

**Fix:** pin every action to a full commit SHA, pin CLI versions and verify signed/checksummed artifacts, remove curl-to-shell, and fail immediately if installation fails. Require both `github.actor` and the issue author to be an authorized admin, use a protected environment with an explicit maintainer approval for account enrollment, serialize setup runs, deliver the device code out of band or restrict it to a dedicated private broker, and verify the resulting provider account identity against the enrollment request before registration.

### S6 — High: cross-repository GitHub credentials are underspecified and overpowered

The design says a public worker reads/writes the private registry with an “automation token” (`sparq/design-record.md:37-39`, `sparq/design-record.md:104-110`) but never defines where that token lives, its lifetime, or its permissions. The public repo's `GITHUB_TOKEN` cannot access the private registry. Storing a PAT or App private key in the public repo recreates the public-secret problem and makes compromise expose private account metadata and lock manipulation.

The setup workflow specifically requests one fine-grained PAT for secret write on `sparq-org/sparq` and issue write on this registry (`registry/set-up-account.yml:4-7`). GitHub fine-grained PATs are limited to resources owned by one selected user or organization, while the named owners are `jeswr` and `sparq-org`. Lines 83-87 then use `ADMIN` for registry issue creation, which will fail for a target-only token. Lines 88-90 invoke `gh` without setting `GH_TOKEN` at all in that step, so comment/close normally fail (and are hidden by `|| true`). See [GitHub's fine-grained PAT limitations](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens).

**Fix:** use `${{ github.token }}` only for operations on the registry in the registry workflow. Mint a distinct, short-lived App installation token for the target repository and use it only for `gh secret set`. For runtime claims, have a private broker authenticate the public job with GitHub OIDC and return a narrowly scoped claim capability; do not put a long-lived private-registry credential or App private key in the public repository.

## 2. Correctness review

### C1 — High: readiness is only partially dependency-aware and conflict-partitioned

Within a complete, clean in-memory snapshot, `compute_ready()` does sort candidates by priority and issue number and selects at most one issue for the single package string it derives (`sparq/ready-issues.py:56-89`). The included fixture validates that narrow case (`sparq/ready-issues.py:92-127`). It is not the claimed production frontier:

1. **Native dependencies are ignored.** `_fetch()` requests only `number,labels,body,state` and counts `Blocked-by: #NN` text markers (`sparq/ready-issues.py:131-146`). Its docstring's claim about native dependency counts is not implemented. An issue with an open native blocker and no marker is dispatched.
2. **The snapshot is incomplete above 1,000 open issues.** The hard `--limit 1000` is not pagination (`sparq/ready-issues.py:134-137`). An omitted blocker appears closed, and an omitted in-progress issue does not reserve its package. API errors fail the command, but truncation silently fails open.
3. **The state machine is fail-open.** Every open issue not bearing one of three busy labels or a gate prefix is eligible (`sparq/ready-issues.py:22-23`, `sparq/ready-issues.py:72-80`). No `status:ready`, priority, role, model, or package is required. `status:untriaged` is eligible. Conversely, `status:deferred` and `status:blocked` can never be retried by this engine unless a separate, not-yet-supplied process removes the label.
4. **Multi-package conflicts are lost.** `package_of()` picks the alphabetically first `area:` label (`sparq/ready-issues.py:40-45`). An issue labeled `area:a, area:b` can run beside an `area:b` issue. Unlabeled/cross-cutting issues reserve nothing and all run concurrently. Package labels also do not account for shared root files such as lockfiles, workspace configuration, generated assets, or CI definitions.
5. **Priority can be nondeterministic on invalid metadata.** `labels_of()` returns a set and `priority_of()` returns the first matching member (`sparq/ready-issues.py:28-37`). Two priority labels can therefore choose different priorities across processes. `_PRIO` also accepts P5-P9 even though the taxonomy ends at P4.
6. **The pure API can reserve a closed issue.** Its initial in-progress scan does not check state (`sparq/ready-issues.py:63-70`). The CLI currently supplies open issues only, but the documented/testable function accepts closed records.
7. **Selection is not a dispatch claim.** Two concurrent scheduler runs can compute the same frontier before either applies `status:in-progress`, launching duplicate issue/package workers. Intra-call partitioning does not solve inter-process races.
8. **Body markers are weak identifiers.** Only same-repo `#N` markers with one spelling are recognized (`sparq/ready-issues.py:142-143`); cross-repo blockers, renamed syntax, deleted/inaccessible issues, and graph-fetch failures are not represented.

**Fix:** query native dependency nodes (GraphQL if necessary) and all open issues with cursor pagination; include a marker only as a validated compatibility fallback. Refuse dispatch if the graph is incomplete. Validate exactly one P0-P4 priority, an allowed role/model, positive ready/trust attestation, and at least one declared partition. Represent partitions as a set and reserve a global/shared-files partition for unknown or cross-cutting work. Serialize scheduler runs with a repository-wide concurrency group and perform an idempotent issue claim keyed by issue/revision before starting a worker. Recompute/unblock state on dependency closure and explicitly transition deferred work back to ready.

### C2 — Critical: the migration neither preserves the graph nor implements idempotency

The module advertises an idempotent two-pass migration, native dependency resolution, and a durable mapping (`sparq/bd-to-issues.py:3-15`), but none exists after planning. `--apply` emits “would create” and exits zero (`sparq/bd-to-issues.py:123-128`). It does not build issue bodies, create labels/issues, create native dependency/sub-issue links, write a map, or verify results.

The planning pass also has concrete graph loss/corruption cases:

- A separate record such as `{"_type":"dependency","issue_id":"sq-a","depends_on_id":"sq-b"}` enters the edge branch, but that branch reads neither `issue_id` nor `depends_on_id`, then unconditionally continues (`sparq/bd-to-issues.py:38-43`). The edge is dropped. Those keys are handled only for dependencies embedded inside an issue (`sparq/bd-to-issues.py:47-53`).
- Every edge type except exact `parent-child` becomes a readiness blocker (`sparq/bd-to-issues.py:83-88`). Related/discovered/duplicate/custom edges would be silently changed into blocking semantics. Duplicate embedded/separate edges are not deduplicated.
- Edges are kept only when both endpoints are among the migrated open/in-progress set (`sparq/bd-to-issues.py:75-88`). Links to closed, deferred, blocked, or missing nodes disappear, so historical graph preservation is false even where readiness would regard a closed blocker as satisfied.
- Direction (`from` is dependent, `to` is blocker) is assumed rather than checked against real export fixtures (`sparq/bd-to-issues.py:38-42`, `sparq/bd-to-issues.py:83-87`). Parent links are only accumulated; their intended child/parent orientation is never verified.
- No source descriptions, comments, status, timestamps, or acceptance data are assembled into the issue payload. Only title/labels and a summary are shown (`sparq/bd-to-issues.py:105-121`).
- An export with zero migratable beads crashes at `next(iter(...))` (`sparq/bd-to-issues.py:117`). Malformed/dangling edges and duplicate IDs are silently ignored/overwritten (`sparq/bd-to-issues.py:44-53`).

There is also no idempotency mechanism: no hidden immutable `bd-id` marker, lookup of already-created issues, uniqueness constraint, checkpoint, or resume behavior. A crash between pass 1 and pass 2 would be unrecoverable without manual reconstruction; a naive rerun would duplicate issues.

**Fix:** first define and fixture the exact `bd export` schema and permitted edge types/directions. Parse all representations into a deduplicated typed graph, reject unknown/dangling/cyclic-invalid data with a report, and decide explicitly whether closed nodes become closed GitHub issues so links remain representable. In pass 1, upsert by a machine-readable marker such as `<!-- bd-id:sq-... -->`, persist each returned issue number immediately in a versioned map, and verify title/body/labels. In pass 2, idempotently upsert native dependency and sub-issue links by source edge ID, retaining body markers only as fallback. On rerun, reconcile rather than create. Exit nonzero if any object/edge is missing, and add interruption/resume, duplicate-record, empty-export, closed-blocker, and mixed-edge tests.

### C3 — Critical: the reaction lock is unsafe across concurrent dispatchers

The central issue does make state visible across repositories, but visibility is not atomic allocation. GitHub's reaction API returns `200` when the caller already added that reaction and `201` only for a new reaction; see [Create reaction for an issue](https://docs.github.com/en/rest/reactions/reactions#create-reaction-for-an-issue). Consequently:

- All workers using the same App/machine identity can create at most one rocket on an account issue. For cap 1, dispatchers A and B can both precheck zero, A gets `201`, B gets the existing reaction, both recount one, and both proceed. For cap greater than one, a single identity cannot represent the capacity at all. One worker's release can delete the reaction while another false claimant still runs.
- With distinct identities, add-then-recount can transiently exceed the cap and has no deterministic winner. Multiple contenders observing `count > cap` may all back off, causing avoidable zero/under-allocation. A caller that observed `<= cap` does not receive an immutable ownership record; later state is inferred from a shared count.
- Reactions from humans or unrelated bots count as workers. The design does not say how list pagination, API failures, rate limits, or a `200 existing` response are handled.
- The reaction is added before the receipt (`registry/README.md:42-49`, `registry/README.md:61-62`). A crash in between leaves an orphan with no receipt, precisely the case the receipt-based groomer cannot classify.
- A receipt has no unique claim/reaction ID or lease expiry (`registry/README.md:42-49`). Multiple historical receipts cannot be unambiguously paired with the current reaction. Reclaiming by “run ended” requires credentials to inspect every source repo, and inaccessible/rate-limited/deleted runs are unspecified.
- Release is only prose (`registry/README.md:34-37`). No `always()`/post-job release, cancellation behavior, heartbeat, idempotent delete, or implemented groomer is supplied. Force cancellation and runner loss will leak slots until a future component guesses that they are stale.

**Fix:** use a real lease primitive. The cleanest design is a private allocator with a transaction/conditional update over `{account, slot}` and unique `{claim_id, run_attempt, holder, issued_at, expires_at}` records. It should return success only after commit, support heartbeat/renewal, and make release/reclaim conditional on the claim ID. If GitHub must be the datastore, serialize all allocation through one central dispatcher and use a SHA/ETag compare-and-swap ledger with retry; do not use reaction counts. Reclaim expired leases conservatively, verify the source run with a dedicated read credential, and retain an audit log separate from ownership state.

### C4 — High: account setup is not concurrency-safe and is internally inconsistent

The next handle is computed by scanning titles and adding one (`registry/set-up-account.yml:40-45`). Concurrent enrollment runs can choose the same handle/secret, overwrite the same target secret, and create duplicate account issues. Add a workflow-wide `concurrency` group, reserve the handle atomically before login, and recheck uniqueness before writing the secret.

As supplied, the workflow invokes `scripts/account-login.sh` (`registry/set-up-account.yml:61`) but the supplied script is `registry/account-login.sh`. If that reflects the repository layout, every run fails. Move the file to the invoked path or change the invocation and add a broker smoke test.

The workflow comment says missing `REGISTRY_ADMIN_TOKEN` “no-ops” (`registry/set-up-account.yml:4-7`), but it has already captured a live credential by then, leaves it on disk, reports success from the storage step, and asks for a rerun that must repeat login (`registry/set-up-account.yml:71-80`). Treat missing storage authority as a preflight failure before device login. If secret storage succeeds but metadata creation fails, record/reconcile the orphaned secret rather than silently leaving inconsistent state.

## 3. Design review

### D1 — Medium: the free-runner premise is accurate but “non-billable” is too broad

`ubuntu-latest` in a public repository is currently a free, unlimited standard runner with the stated 16 GB RAM (`sparq/design-record.md:31-39`); larger runners remain billable. See [GitHub Actions billing](https://docs.github.com/en/billing/concepts/product-billing/github-actions) and [runner specifications](https://docs.github.com/en/actions/reference/runners/github-hosted-runners). The plan still has costs and limits omitted from its model:

- Provider/model subscriptions or API usage are the dominant variable cost.
- The private `set-up-account` workflow itself runs for up to 20 minutes on `ubuntu-latest` (`registry/set-up-account.yml:17-24`), consuming private-repository included/billable minutes. That is rare, not zero.
- Artifact/cache/log storage can be billed, and public jobs remain subject to concurrency, job, API-rate, availability, and abuse controls. “Unlimited minutes” is not a capacity/SLA guarantee.
- The public runner has only 14 GB SSD as currently documented, which may be a tighter Rust workspace constraint than RAM. No preflight estimates disk/RAM or bounds worker fan-out.

**Fix:** call the property “no GitHub-hosted standard-runner minute charge for public jobs,” budget provider spend and storage separately, set per-account and global daily token/run caps, cap fleet concurrency, add API backoff, and define overload/manual fallback. Use the private setup runner only for rare enrollment and account for it. Add resource-class labels and preflight disk/RAM checks rather than relying on a manual exception after failure.

### D2 — High: routing intent is clear, but the configuration cannot enforce it

Fable is indeed used heavily: it is first in defaults and the `impl`, `site`, `ci`, `perf`, and `research` routes, and is the docs fallback (`sparq/routing.toml:18-53`). The conceptual statement that `codex` is a harness and `terra` is a model alias is also the right separation (`sparq/routing.toml:7-15`). The actual configuration has gaps:

- There is no structured model catalog mapping `terra` to provider `openai`, harness `codex`, and a concrete provider model ID/version. `terra` is not a public OpenAI model ID; as an internal alias it needs a definition. The registry example instead lists `codex` and `gpt-5.6` as account “models” (`registry/README.md:17-25`), directly conflicting with the rule that Codex is not a model. A selector matching the literal chain `terra` will find no account unless undocumented private logic special-cases it.
- Precedence is unspecified. The label rule says it overrides role “regardless of role,” but it appears after role rules (`sparq/routing.toml:22-72`). A first-match implementation would route an `impl+auth` issue to Fable. Exact `match_labels = ["zk", ...]` also does not match the `area:<crate>` taxonomy demonstrated by `area:sparq-zk` (`sparq/ready-issues.py:93-101`).
- CI has no cross-provider fallback; perf/research fall from Fable to expensive Opus rather than Terra; review/soundness have only Opus (`sparq/routing.toml:34-64`). An Anthropic outage can stall security review indefinitely despite the claimed graceful degradation.
- `escalate` is only a comment/field and the consuming behavior is absent (`sparq/routing.toml:15-16`, `sparq/routing.toml:55-72`). No schema/version validation prevents misspelled roles/models/agents or an empty chain.

Current OpenAI docs expose concrete model IDs rather than `terra` (for example [GPT-5.3-Codex](https://developers.openai.com/api/docs/models/gpt-5.3-codex)); the locally installed Codex CLI does support `codex login --device-auth`, so the harness choice is plausible. What is missing is the alias-to-provider-model contract.

**Fix:** add a versioned catalog such as `models.terra = {provider="openai", harness="codex", provider_model="...", credential_format="codex-auth-json"}` and analogous Claude entries. Keep harness, provider model, and account capability as separate fields. Define priority as `security-label override > explicit role > defaults`, use real/full label names or patterns, validate TOML against a schema, and test every role/override/fallback. Give critical lanes an independent-provider or explicit human fallback, and implement/test `escalate` before relying on it.

### D3 — High: CI batching/economy claims exceed what branch protection can safely support

Package labels are a scheduling hint, not proof that branches are disjoint (`sparq/design-record.md:127-133`). Rust crates commonly touch shared `Cargo.lock`, workspace manifests, generated code, CI, docs, and cross-crate behavior. Git textual non-conflict also does not imply semantic independence.

The “one integration PR -> merge all” step is underspecified. Required check results are associated with an exact commit/ref. A green integration PR does not automatically satisfy checks on each original draft PR, and merging original PRs one by one produces commits different from the tested integration commit. Bypassing branch protection to do so would remove the stated correctness backstop (`sparq/design-record.md:137`). Merging only the integration PR preserves the tested tree but needs an explicit policy for authorship, original PR closure, issue autoclose, auditability, and later base changes.

Binary split assumes a monotonic single culprit (`sparq/design-record.md:132-133`). Two independently failing branches, a failure caused only by interaction between branches, order dependence, or flaky tests can make both halves pass or both fail and defeat the algorithm. “Meaningful non-flake” has no classifier or retry budget.

Debouncing main CI for 30-60 minutes can starve forever under steady merges and delays detection exactly when autonomous changes are landing (`sparq/design-record.md:134-137`). Canceling an old run is safe only if a newer run covers a descendant commit and produces a required conclusion; security/supply-chain checks should not be debounced.

**Fix:** build the batch as one immutable integration commit on the current protected-base/merge-queue head, run the full required suite on that exact commit, and merge that commit as the sole unit. Revalidate base SHA immediately before merge. Derive conflict sets from changed-file policy plus declared packages, and reject shared/security/orchestration paths from normal batches. On failure, support multiple culprits and interaction testing; bound flake retries and quarantine inconclusive batches. Give batching a maximum wait/size and main CI a maximum deferral deadline; never debounce security, policy, migration, or release-integrity checks.

### D4 — Critical: cross-repo token flow has no viable least-privilege implementation as written

GitHub's secrets API can set/list metadata but cannot return a stored secret's plaintext. Therefore the public worker cannot retrieve the selected token from the private registry by API; option A necessarily duplicates the plaintext into the public repo, and option B requires a live broker. The current design sits between those choices: selection happens privately, `secret_ref` crosses to the public worker, and `toJSON(secrets)` exposes all public-repo secrets (`sparq/design-record.md:68-102`; `registry/README.md:75-80`).

The operator documentation is contradictory about the source of truth: the design's maintainer checklist says to put account tokens in private-repo secrets (`sparq/design-record.md:149-153`), the registry README says token values live only in its repository/organization secrets (`registry/README.md:29-30`), but the setup workflow explicitly stores them in the public target repository (`registry/set-up-account.yml:25-27`, `registry/set-up-account.yml:81-82`). Following the wrong instruction either leaves workers unable to retrieve plaintext or puts credentials in the higher-risk public boundary without informed sign-off.

At least four distinct authorities are being conflated:

1. public repo issue/branch/PR write;
2. private registry metadata/claim read-write;
3. target repository secret administration during rare enrollment; and
4. provider model invocation.

A stable PAT/App token spanning them creates a high-value, long-lived credential in the least trusted environment. A GitHub App is preferable to a PAT, but installation tokens are minted per installation; if repositories live under different owners/installations, use separate short-lived tokens. The App private key itself must not be placed in the public repository.

**Fix:** separate the authorities and publish one unambiguous credential-flow specification. The public workflow uses its ephemeral `GITHUB_TOKEN` only for sparq operations. It presents GitHub OIDC plus run/repo/commit claims to a private allocator. The allocator validates the workflow identity, atomically grants a lease, and either proxies the model request or issues a short-lived provider-scoped capability. A distinct enrollment service has target-secret write but no runtime role. The allocator's registry token has no public-repo contents write. Log claim IDs and model/budget metadata, never provider credentials or global secret references.

## 4. Additional implementation gaps

- The private `select-and-claim`, stale groomer, worker, trust workflows, batcher, debounce logic, and auto-close path are future prose rather than reviewable code (`sparq/design-record.md:139-147`; `registry/README.md:51-67`). None of their safety claims should be treated as implemented controls.
- The readiness and trust self-tests exercise only happy-path pure functions (`sparq/ready-issues.py:92-127`; `sparq/trust-gate.py:58-80`). Add adversarial integration tests for concurrent dispatch, edited approvals, nested untrusted PR content, API truncation/failure, multiple labels/partitions, native dependencies, cancellation, lease expiry, and secret-bearing job isolation.
- `fetch_permission()` discards all error detail (`sparq/trust-gate.py:47-55`). Failing closed is correct, but emit a non-sensitive reason/metric so an outage is distinguishable from a genuine untrusted user and cannot silently stall the fleet.
- The setup provider parser defaults to Anthropic on missing/ambiguous text (`registry/set-up-account.yml:37-45`). Require a structured issue form value and reject ambiguity; silent defaulting can enroll the wrong credential type.
- Receipt/cache-affinity comments and a mutable JSON file are proposed as both audit and scheduling state (`registry/README.md:42-67`) without a consistency model. Keep cache affinity advisory and outside the correctness-critical lease transaction; never let a cache preference bypass provider/model/capacity validation.

## Minimum launch gate

Before any autonomous writer receives real credentials, require all of the following:

1. A brokered credential boundary with no long-lived provider, registry, App-private-key, or admin secret in the public repo.
2. A positive, revision-bound trust attestation enforced at the last point before model invocation, plus provenance filtering for everything the model can fetch.
3. A transactional unique lease and an idempotent issue/package dispatch claim with cancellation, expiry, heartbeat, and tested reclamation.
4. A completed, resumable migration tested against a real export copy, followed by graph/count reconciliation.
5. A paginated, native-dependency-aware readiness engine with validated state and multi/global conflict partitions.
6. Exact-commit batch CI that satisfies branch protection without bypass and cannot defer main/security checks indefinitely.
7. Pinned, verified enrollment dependencies; isolated temporary auth state; runtime masking; and unconditional cleanup.
8. End-to-end adversarial tests demonstrating that a fork author, issue commenter, same-repo worker branch, edited approved comment, compromised model output, canceled run, and two simultaneous dispatchers cannot obtain credentials, bypass trust, duplicate work, exceed capacity, or merge an untested tree.

Until those gates pass, run the pipeline in dry-run/read-only mode with synthetic credentials and no merge permission.
