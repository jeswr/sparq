# G3 — cost-capped, secure EC2-deferred benchmark CI

The heavy benchmarks (many-core/NUMA scaling, large-scale ingestion) can't run on free GitHub
runners. They run on EC2, but a **public** repo with EC2-triggering CI is dangerous if done wrong,
and must stay **under $5/month**. This is the security + cost design.

> **STATUS ([OPUS-5], #3784) — the automated lane is RETIRED; this record is now the revival spec.**
> The AWS OIDC role this design depends on was deliberately descoped, so `bench-ec2.yml`'s crons
> were removed: every scheduled tick failed in `Configure AWS credentials (OIDC)` before any
> benchmark ran and — carrying no advisory declaration — that failure **gated** `main`. The workflow
> is now **manual dispatch only** (`lane: heavy` / `lane: full-suite`), guarded by
> `vars.AWS_BENCH_ROLE_ARN != ''` so a dispatch without the role skips instead of failing.
> Everything below still describes what to create to bring the lane back, so the `schedule:` block
> quoted later is **aspirational, not current** — re-add it in the same change that re-provisions the
> role. Maintainer-run EC2 benchmarking outside this CI lane is NOT descoped.

<!-- separate blockquotes (MD028) -->

> **G2 — free per-commit perf CI (Pages toggle).** `.github/workflows/bench.yml` runs
> `scripts/ci-bench.sh` on every push to `main` and on `pull_request` (PRs compute + comment but
> do **not** auto-push, so the published series isn't polluted), storing points via
> github-action-benchmark on the `benchmark-data` branch under `dev/bench`. **One-time owner
> action (cannot be set from a workflow):** Settings → Pages → Source = "Deploy from a branch",
> Branch = `benchmark-data` / `/ (root)` → Save. The dashboard then renders at
> <https://sparq.jeswr.org/dev/bench>. [OPUS-4.8]

## Threat model & the three hard rules

1. **No long-lived AWS keys in GitHub secrets.** Use **GitHub OIDC → a scoped IAM role** that CI
   assumes for a short-lived token. A leaked workflow log then exposes nothing reusable.
2. **Never run on untrusted triggers.** The EC2 workflow runs ONLY on `schedule` (weekly) and
   `workflow_dispatch` (a maintainer clicks "run"). **Never** `pull_request` — a fork PR must not be
   able to assume the role or spend money. Guard every job with
   `if: github.repository == 'sparq-org/sparq'`.
3. **Self-limiting spend.** Spot instance + hard `timeout-minutes` + **always-terminate** (even on
   failure/cancel) + a low weekly cadence. 4–5 runs/month × a ~30-min spot box ≈ **well under $5**.

## The OIDC trust (created once, in AWS — the browser/console step)

1. **IAM → Identity providers → Add provider → OpenID Connect**
   - Provider URL: `https://token.actions.githubusercontent.com`
   - Audience: `sts.amazonaws.com`
2. **IAM → Roles → Create role → Web identity** → the provider above, audience `sts.amazonaws.com`.
   Trust policy scoped to THIS repo + protected refs only:
   ```json
   {
     "Version": "2012-10-17",
     "Statement": [{
       "Effect": "Allow",
       "Principal": { "Federated": "arn:aws:iam::<ACCOUNT_ID>:oidc-provider/token.actions.githubusercontent.com" },
       "Action": "sts:AssumeRoleWithWebIdentity",
       "Condition": {
         "StringEquals": { "token.actions.githubusercontent.com:aud": "sts.amazonaws.com" },
         "StringLike": {
           "token.actions.githubusercontent.com:sub":
             "repo:sparq-org/sparq:ref:refs/heads/main:runner_environment:github-hosted"
         }
       }
     }]
   }
   ```
   (The `sub` condition pins it to the `main` branch of `sparq-org/sparq` — fork PRs and other refs
   cannot assume it. The `runner_environment` segment is **only** present if the repository's
   subject claim has been customised with `include_claim_keys: ["repo", "ref", "runner_environment"]`
   — with the DEFAULT claim template the `sub` is just `repo:sparq-org/sparq:ref:refs/heads/main`
   and this policy will not match. Pick one and apply both halves together; see item 2 of the
   checklist below.)

   > **ORG MIGRATION — the live role is still on the OLD org, and that is why the lane died.**
   > The repository moved `jeswr/sparq` → `sparq-org/sparq`. The workflow was updated (`bench-ec2.yml`
   > guards `github.repository == 'sparq-org/sparq'`) but **the IAM trust policy was not**, so
   > `sts:AssumeRoleWithWebIdentity` has been refused ever since: `bench-ec2.yml` failed every
   > scheduled run from 2026-07-15 to 2026-07-25 with 13 × `Credentials could not be loaded`, and the
   > `schedule:` triggers were then retired in #3785. This is the GOOD failure mode — the role was
   > **not** widened to compensate, so a role that can launch EC2 instances is not over-permitted.
   > Anyone recreating or repointing the role must apply the checklist below.

3. **Permissions policy** — least-privilege, tag-scoped so CI can only touch its own instances:
   ```json
   {
     "Version": "2012-10-17",
     "Statement": [
       { "Effect": "Allow", "Action": ["ec2:RunInstances"], "Resource": "*",
         "Condition": { "StringEquals": { "aws:RequestTag/purpose": "sparq-ci-bench" } } },
       { "Effect": "Allow", "Action": ["ec2:CreateTags"], "Resource": "*",
         "Condition": { "StringEquals": { "ec2:CreateAction": "RunInstances" } } },
       { "Effect": "Allow", "Action": ["ec2:TerminateInstances","ec2:DescribeInstances","ec2:DescribeInstanceStatus","ec2:DescribeSpotInstanceRequests"], "Resource": "*",
         "Condition": { "StringEquals": { "ec2:ResourceTag/purpose": "sparq-ci-bench" } } },
       { "Effect": "Allow", "Action": ["ec2:DescribeImages","ec2:DescribeSubnets","ec2:DescribeSecurityGroups"], "Resource": "*" }
     ]
   }
   ```
   No IAM, no S3-write beyond a single results prefix (add an S3 statement only if results go via
   S3). The role can launch/terminate ONLY `purpose=sparq-ci-bench`-tagged instances.
4. Set the role ARN as a **repo *variable*** (not a secret — an ARN isn't sensitive):
   `AWS_BENCH_ROLE_ARN`.
5. **Budget backstop:** an AWS Budgets monthly alarm at $5 on the `purpose=sparq-ci-bench` tag.

### Trust-policy hardening checklist (MAINTAINER — requires AWS console access)

**No agent has, or should have, AWS credentials.** Nothing in this repo can apply any of this; it is
a written checklist for the human who owns the AWS account. Items 1–4 are the conditions that make
this role safe for a *public* repository whose CI can spend money.

1. **Repointing the `sub` to the new org is the whole fix for the dead lane.**
   `repo:jeswr/sparq:…` → `repo:sparq-org/sparq:…`. Nothing else is required to revive it.
2. **`runner_environment` and `workflow_ref` can only be enforced through `sub`.** IAM evaluates
   exactly two condition keys from the GitHub OIDC provider — `token.actions.githubusercontent.com:aud`
   and `…:sub`. Every other claim (`runner_environment`, `workflow_ref`, `repository_owner`,
   `job_workflow_ref`, `environment`, `actor`) is present in the JWT but is **not** a usable IAM
   condition key, so a `StringEquals` on `…:runner_environment` silently matches nothing and buys
   nothing. The only way to bind them is to fold them into the `sub` claim itself, via the
   repository's **Actions → OIDC → customize the subject claim** setting (`include_claim_keys`), and
   then match the resulting composite string. The intended binding for this role is
   **`runner_environment=github-hosted`** — a self-hosted runner must not be able to assume a role
   that launches EC2 instances. `include_claim_keys` is a **repository-level** setting: changing it
   rewrites the `sub` of *every* workflow in the repo, so the IAM policy and the claim-key list must
   be changed together or all OIDC in the repo breaks. If that coupling is not wanted, keep the short
   `sub` (`repo:sparq-org/sparq:ref:refs/heads/main`) and accept that `runner_environment` is
   **unenforced** — but record that choice here rather than leaving it implicit.
3. **The policy must admit NO `…:pull_request` `sub` form.** A `pull_request`-triggered run gets
   `sub = repo:sparq-org/sparq:pull_request`. A role that can launch EC2 instances must never be
   assumable from a PR context — including a PR from a fork. Concretely: no `StringLike` pattern in
   the trust policy may match `repo:sparq-org/sparq:pull_request`, which rules out the tempting
   wildcard `repo:sparq-org/sparq:*`. Verify the pattern against that exact string before saving.
   The workflow-side `if:` fork guard is defence in depth, not the boundary — the trust policy is.
4. **The `ref:refs/heads/main` pin also blocks `workflow_dispatch` from any non-default branch.**
   A maintainer dispatching this workflow from a topic branch gets `sub = …:ref:refs/heads/<branch>`
   and the assume-role fails. That is plausibly intentional (it stops an unreviewed branch from
   spending money) but it was never written down, and it is now the lane's only supported trigger
   since #3785 retired the cron. **Decide and record:** keep the pin (dispatch must be from `main`),
   or widen it deliberately to an enumerated allowlist — never to `repo:sparq-org/sparq:*`, per (3).

## The workflow (`.github/workflows/bench-ec2.yml`)

> **The live file no longer matches this sketch, deliberately.** #3785 removed BOTH `schedule:`
> blocks (a cron tick against a role that cannot be assumed is a guaranteed failure, every night)
> and added a `vars.AWS_BENCH_ROLE_ARN != ''` role-present guard so a dispatch while the role is
> descoped SKIPS visibly instead of failing at the OIDC step. The lane is therefore
> **dispatch-only** today; the sketch below shows the shape to restore *together with* the
> trust-policy fix, not the current state. Re-adding `schedule:` also puts the lane back in the
> scope of the cron-only liveness alarm (`scripts/cron_lane_liveness.py`), which is derived from
> the `schedule:` block itself — so a future silent death gets an issue rather than 12 days of
> nothing.

```yaml
on:
  schedule: [{ cron: '0 6 * * 1' }]   # Mondays 06:00 UTC — the rate limit (≈4–5/month)
  workflow_dispatch:                  # + manual maintainer trigger
permissions: { id-token: write, contents: write }
concurrency: { group: bench-ec2, cancel-in-progress: false }
jobs:
  ec2-bench:
    if: github.repository == 'sparq-org/sparq'   # never on forks
    runs-on: ubuntu-latest
    timeout-minutes: 60                       # hard wall-clock cap
    steps:
      - uses: actions/checkout@v4
      - uses: aws-actions/configure-aws-credentials@v4
        with: { role-to-assume: ${{ vars.AWS_BENCH_ROLE_ARN }}, aws-region: eu-west-2 }
      - run: bash scripts/ec2-bench.sh        # launch SPOT, run, ALWAYS terminate, fetch results
      - uses: benchmark-action/github-action-benchmark@v1   # track the EC2 series like G2
        with: { tool: customSmallerIsBetter, output-file-path: ec2-bench-results.json, auto-push: true, gh-pages-branch: benchmark-data, benchmark-data-dir-path: dev/bench-ec2 }
```

## `scripts/ec2-bench.sh` (to implement)

Since the repo is now **public**, the instance can `git clone` it with no credentials — no SSH key
in CI. Flow: `run-instances` a **spot** box (tagged `purpose=sparq-ci-bench`) with **user-data**
that installs Rust, clones the repo at the triggering SHA, runs the scaling sweep + an ingestion
bench, writes `ec2-bench-results.json`, and **self-terminates** (`shutdown -h` with
`InstanceInitiatedShutdownBehavior=terminate`). The runner polls instance state, pulls the results
(via S3 or SSM), and asserts termination. A `trap`/`always()` terminate-by-tag step guarantees no
orphaned instance even if the run is cancelled.

## Cost math

c7g.4xlarge spot ≈ $0.25/hr; a 30-min run ≈ $0.13. Weekly + occasional manual ≈ 6 runs/month ≈
**$0.80/month** — order of magnitude under the $5 cap, with the AWS Budget alarm as a hard backstop.
A rare 192-vCPU NUMA sweep (manual dispatch only) at spot ~$4/hr for ≤30 min ≈ $2, still within a
month's budget if used sparingly.

## Status

Design complete. Execution needs: (a) the IAM OIDC provider + role created in AWS (console/browser
— the `PSSSingleInstanceDeploy` SSO role lacks `iam:*`), (b) the `AWS_BENCH_ROLE_ARN` repo variable,
(c) `scripts/ec2-bench.sh` + `bench-ec2.yml` committed. (a) is the gating step.
