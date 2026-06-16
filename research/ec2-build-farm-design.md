# EC2 build farm — ephemeral, orphan-proof, cost-capped branch gating

> Status: design + shipped launcher (`scripts/ec2-buildfarm.sh`). `[OPUS-4.8]` (authored by
> Opus 4.8, 1M context; Fable unavailable — flag for re-review when Fable returns).
> Scope: a BUILD farm — it runs the *merge gate* of one branch (build + clippy `-D warnings` +
> tests in BOTH feature states) on a disposable EC2 box and self-terminates. It is **not** a
> benchmark farm and **not** a CI replacement; see §7 for what it is and is not.

## 0. Why this exists

The session and every sub-agent run on the persistent EC2 **work box** (8 vCPU). That box is the
throughput ceiling for `cargo` gating: AGENTS.md caps concurrent *cargo-heavy* agents to the core
budget (~6 in practice) before the box thrashes and wall-clock gating slows for everyone. When many
branches are ready to gate at once, the gate — not the implementation — becomes the serialisation
point.

The maintainer greenlit using more EC2 to lift that ceiling, **with explicit cost caps and the
project's non-negotiable EC2 safety rules**. This farm offloads the *gate of a single branch* to an
ephemeral instance, so N branches can be gated in parallel beyond the local core budget. Each
instance is short-lived (one gate, then gone), so total cost is bounded by the per-instance runtime
× the concurrency cap — but the orphan-proofing and the cap are the load-bearing parts, not the
nice-to-haves.

## 1. The three EC2 farms — keep them separate

There are now three *distinct* fleets, each with its **own purpose tag** so a launcher or
orphan-checker only ever sees (and can only ever terminate) its own:

| Farm | Tag (`purpose=`) | Launcher | Job |
|---|---|---|---|
| Per-commit perf CI | `sparq-ci-bench` | `scripts/ec2-bench.sh` (OIDC, in `bench-ec2.yml`) | scaling/ingest benchmark, results JSON |
| Competitor / same-box gather | `sparq-bench` | `scripts/gather-ec2.sh`, `scripts/qlever-same-box.sh` | competitor envelopes (the sq-sxso lane) |
| **Build farm (this doc)** | **`sparq-buildfarm`** | **`scripts/ec2-buildfarm.sh`** | **build + clippy + test x2 of a branch** |

The tags are an **allow-list, not a deny-list**: each farm's orphan-check returns *only* instances
carrying its exact tag, so the farms can never collide and the build farm can never touch a bench
box (or vice versa). This farm is **completely separate** from the competitor-gather (which is the
same-box `sq-sxso` lane that stalls when co-resident) — they share no instance, no tag, no script.

## 2. The launcher interface (`scripts/ec2-buildfarm.sh`)

```
scripts/ec2-buildfarm.sh <branch|#PR> [region]   # launch ephemeral box, run gate, self-terminate
scripts/ec2-buildfarm.sh --dry-run <branch|#PR>   # print the plan + the user-data; launch NOTHING
scripts/ec2-buildfarm.sh --self-test              # hermetic unit test (no aws, no network)
scripts/ec2-buildfarm.sh --orphan-check [--apply] # find (and with --apply terminate) buildfarm orphans
scripts/ec2-buildfarm.sh --region <r> …           # override region (default eu-west-2)
```

* **Ref token** — accepts a branch name (`my-feature`, `feat/x-y`), or a PR token (`#123`, `PR-7`,
  `123`). A PR token resolves to the PR's head ref via `gh` if present, else falls back to the
  immutable `pull/<n>/head` ref (works with no `gh`). A name containing a non-digit is always a
  branch.
* **Exit code** — `0` iff the remote gate verdict is `PASS`; non-zero on `FAIL`/`UNKNOWN`, so a
  caller (orchestrator or a CI step) can branch on it directly.
* **Stdout** — the machine-readable result line:
  `{"ref":"…","sha":"…","verdict":"PASS|FAIL","features_b":"approx-ann,filtered-ann,vec-predicate"}`.
  Stderr carries the progress log and the gate-log tail.
* **Env overrides** — `AWS_PROFILE` (use `pss`), `BUILDFARM_INSTANCE_TYPE`, `BUILDFARM_FEATURES`
  (the second feature state), `BUILDFARM_MAX_LIFETIME_SEC` (clamped 600..10800),
  `BUILDFARM_MAX_FLEET` (clamped 1..16), `BUILDFARM_SPOT` (1=spot default, 0=on-demand),
  `BUILDFARM_DISK_GB`.

## 3. The gate it reproduces (faithful to AGENTS.md "Merge discipline")

The user-data runs, on a 16-vCPU box, exactly the merge gate — in **both feature states**:

1. `cargo build --workspace`
2. `cargo clippy --workspace --exclude sparq-py --all-targets -- -D warnings`  **(GATING)**
3. `cargo test --workspace`  — **feature state A: default (opt-in OFF)**
4. `cargo build  --workspace --features approx-ann,filtered-ann,vec-predicate`
   `cargo clippy --workspace --exclude sparq-py --all-targets --features approx-ann,filtered-ann,vec-predicate -- -D warnings`
   `cargo test   --workspace --features approx-ann,filtered-ann,vec-predicate`  — **feature state B: the CI opt-in set**

The opt-in set in state B is the **same one `ci.yml`'s nextest archive + doctests carry**
(`approx-ann,filtered-ann,vec-predicate`) — verified to resolve cleanly against the workspace.
`sparq-py` is excluded exactly as AGENTS.md/CI prescribe (it needs the Python/maturin toolchain).
`cargo fmt --all --check` runs but is **ADVISORY / non-gating**, matching `ci.yml` (rustfmt is
informational until the deferred one-time `cargo fmt --all` reformat lands).

**Honest scope of the verdict.** This reproduces the *core* gate (build/clippy/test x2). It does
**not** run the conformance/perf ratchets, coverage gate, wasm-target legs, SBOM/deny, CodeQL, or
the full per-crate `feature-matrix.yml` legs — those still run in GitHub Actions on the PR. The farm
is a fast **pre-push / pre-merge confidence check** that catches the overwhelmingly most common
breakages (a compile error, a clippy `-D warnings` regression, a failing test, a feature-state-only
break) without burning the local box; the authoritative gate remains `ci-summary` on the PR.

## 4. Orphan-proofing (the non-negotiable safety guarantee)

An instance must **never** silently outlive its job, even if the controller (this script, the agent,
or the whole session) dies. Three independent mechanisms, each of which alone bounds the lifetime:

1. **`--instance-initiated-shutdown-behavior terminate`** — any `shutdown` from inside the box
   *destroys* the instance (not stop). So either watchdog firing terminates, not parks, the box.
2. **Detached sleep watchdog** — `( sleep $MAX_LIFETIME_SEC; shutdown -h now ) &`, armed in
   user-data **before any apt-get**. No package dependency; works the instant cloud-init starts.
3. **systemd-run transient timer** — `systemd-run --on-active=${MAX_LIFETIME_SEC}s /sbin/shutdown -h now`,
   also armed before apt-get. systemd is pre-installed on the AMI, so it arms even if the detached
   subshell's process group is reaped.

The user-data writes the result + a `/root/BUILD_DONE` sentinel and then **STOPS** — it does **not**
shut down on completion. The controller polls (over SSH) for the sentinel, pulls the result while
the box is alive, then terminates it **explicitly** (a `trap … EXIT` cleanup that always runs, even
on error/cancel). This is the sq-8dp3 results-loss fix carried over from `gather-ec2.sh`: an eager
post-job `shutdown` would race the controller's pull on a `DeleteOnTermination` volume and lose the
result. So: the watchdog is the **only** auto-terminate (orphan backstop); the happy path is an
explicit terminate after the pull.

**`MAX_LIFETIME_SEC` = 7200 s (2 h)** by default (clamped to `[600, 10800]`). A from-scratch
build + clippy + test ×2 on a 16-vCPU box is ~20–40 min, so 2 h is generous headroom while bounding
a worst-case orphan to ≤2 h of spend (≤~$0.27 at the spot price below). **Quote: hard max lifetime
2 h; concurrency cap 12 (ceiling 16).**

Further belt-and-braces: ephemeral per-run keypair + a security group that only allows SSH **from
the controller's IP /32**, both deleted in cleanup; `run-instances` output is validated to be a real
`i-…` id before any wait/poll/terminate (a failed launch — `VcpuLimitExceeded`, no spot capacity —
aborts instead of spinning); `sshd`-never-reachable aborts (→ the EXIT trap terminates the box)
rather than hammering an unreachable host for the full poll window.

## 5. Cost cap (hard, enforced) + cost model

**Enforcement.** Before launching, the script counts running+pending `purpose=sparq-buildfarm`
instances and **refuses** to launch a `(count+1)`-th once `count >= MAX_FLEET`
(default 12, clamped to ≤16). Verified: with a simulated fleet of 5 and cap 3 the launcher exits 1
*before* any `run-instances` call. This is a real ceiling, not a comment.

**Instance + pricing (measured, eu-west-2, 2026-06-16).** `c7g.4xlarge` (16 vCPU Graviton3, arm64)
on **spot**: observed **$0.095–0.136/hr** across AZs (use ~$0.13/hr for estimates). On-demand list
is ~$0.62/hr. Spot is the default: the box is disposable and the job is idempotently
re-launchable, so a spot reclaim just means "re-run", with no result loss (the result is pulled
before the explicit terminate; a reclaim before the sentinel is detected and reported, not silently
swallowed).

**Per-gate cost.** A ~40-min worst-case gate at ~$0.13/hr ≈ **$0.087 per branch gate** (spot);
on-demand ≈ $0.41. A typical ~25-min gate ≈ **$0.054** (spot).

**Cost at the cap (the worst case the maintainer should price).** All 12 boxes running for the full
2 h hard-lifetime simultaneously (i.e. every gate hung to the watchdog — pathological):
`12 × 2 h × $0.13 = $3.12` for that 2-h burst. Sustained, that is at most `$3.12` per 2-h window if
the farm were *continuously* saturated to the cap with hung jobs — but jobs finish in ~25–40 min and
the box terminates immediately, so realistic spend is **per-gate, not per-hour**: e.g. gating 40
branches in a busy day ≈ `40 × $0.054 ≈ $2.16/day`. **Honest envelope: the hard ceiling on
*instantaneous* spend is 12 × $0.13/hr ≈ $1.56/hr; a realistic busy day is ~$2–4/day.** This is the
ephemeral-instance bound the maintainer greenlit; it is far below a standing fleet.

**Backstops the maintainer should add (one-time, console — this role can't do IAM):** an AWS Budgets
alarm on `purpose=sparq-buildfarm` (e.g. $20/mo) and, if the farm is ever CI-triggered, the
tag-scoped OIDC role pattern already designed in `research/ci-ec2-design.md` (scope its
`ec2:RunInstances` to `aws:RequestTag/purpose=sparq-buildfarm`). The launcher today runs from the
work box under `AWS_PROFILE=pss`.

**vCPU-limit note.** The current On-Demand vCPU usage in eu-west-2 is ~10 (prod `t3.large` 2 vCPU +
dev `r7g.2xlarge` 8 vCPU), close to a 16-vCPU On-Demand standard bucket. Spot uses a **separate**
quota, which is the other reason the farm defaults to spot — it does not compete with prod/dev for
the On-Demand bucket. If a real run hits `VcpuLimitExceeded` even on spot, the maintainer should
request a spot-instances quota increase for the standard family in eu-west-2.

## 6. Safety invariants (never touch prod/dev)

* **Region eu-west-2, profile pss, tag `purpose=sparq-buildfarm`** (distinct from `sparq-bench` /
  `sparq-ci-bench`).
* **Hard never-touch list**: prod `i-090531b4ede8f2d3f` and dev `i-00f76802f345b6b77` are removed
  from any kill set unconditionally, and the self-test asserts they can never be in it (mirrors
  `orphan-check-bench.sh`). They also never enter the candidate set because they don't carry the
  buildfarm tag — the hard exclusion is the belt-and-braces second line.
* The launcher only ever terminates the **one** instance id it itself launched (the cleanup trap),
  and `--orphan-check` only ever terminates tag-matched, prod/dev-excluded instances.
* **Orphan-checks before launch** (advisory, to surface any leak + enforce the cap honestly) **and
  after** (the cleanup trap; a fresh session can also run `--orphan-check` to sweep). Verified clean
  against live AWS at authoring time (no buildfarm orphans).

## 7. How the orchestrator invokes it + composition with the push-scheduler

This farm is the *cargo-heavy* relief valve for the rule in AGENTS.md: "Cap concurrent *cargo-heavy*
agents to the core budget; doc/research/config agents parallelise freely." The composition:

* **When the local core budget is full and more branches are ready to gate**, the orchestrator
  offloads a branch's gate to the farm instead of queueing it behind the local cargo agents:
  `AWS_PROFILE=pss scripts/ec2-buildfarm.sh <branch>` (run in the **background** per the
  background-dispatch rule; the launcher is a long-running poll). The exit code / result JSON tells
  the orchestrator PASS/FAIL without spending a local core or a model token on log-spelunking.
* **It does not replace `ci-summary`.** A farm PASS is a *pre-push confidence* signal: it says "this
  branch builds, lints `-D warnings`-clean, and tests green in both feature states" so the
  orchestrator can push/merge-arm with confidence the authoritative PR gate will go green — it does
  not authorise skipping the PR + `ci-summary` flow (branch protection still requires it).
* **Composition with the push-scheduler / orchestration automation**
  (`research/orchestration-automation-design.md`). That doc keeps *detection + bookkeeping*
  mechanical (scripts/hooks/monitors) and *judgment* (review a diff, arm auto-merge, resolve a
  conflict) with the orchestrator. The build farm slots into the **mechanical** plane as a
  *deterministic gate executor*: a script the orchestrator (or a future monitor) fires to get a
  PASS/FAIL, exactly like `orphan-check.sh` / `gate-retrigger.sh` are deterministic mechanics. It
  must **not** auto-merge on a green farm result — that is the judgment step that stays with the
  orchestrator (the doc's highest-severity failure mode is an unattended mechanism that auto-merges).
  The push-scheduler decides *what* to gate and *when* to push; the farm is the *capacity* that lets
  it gate more than ~6 branches at once. Its orphan-check should be added to the cron safety-net
  sweep alongside the bench orphan-check (one more `--orphan-check` call), so a dead controller's
  box is reaped by the floor even if the cleanup trap never ran.

## 8. Validation status

* **YAML/script validity**: shellcheck-clean (`shellcheck scripts/ec2-buildfarm.sh`).
* **Hermetic self-test** (`--self-test`): 19 checks PASS — clamp bounds, ref resolution, the
  prod/dev hard-exclusion, the distinct tag, and that the rendered user-data contains BOTH
  orphan-proof watchdogs, the GATING `-D warnings` clippy, BOTH feature states, the sentinel, and
  *exactly two* `shutdown` calls (no eager third that would race the result pull).
* **Dry-run** (`--dry-run`): exercised end-to-end against real read-only AWS — resolves the ref,
  queries the live fleet (0/12) and orphan-check (clean), renders the full user-data, launches
  nothing.
* **Cost-cap refusal**: verified the launcher refuses (exit 1) before any `run-instances` when the
  fleet ≥ cap.
* **Gate fidelity**: the `--exclude sparq-py` crate and the `approx-ann,filtered-ann,vec-predicate`
  feature set were confirmed to resolve against the actual workspace and to match the set `ci.yml`
  uses.
* **Real instance launched?** **No.** Per the task constraint, no real build-farm instance was
  launched in this work — only read-only describe/spot-price calls and the dry-run. The first real
  run is the maintainer's to trigger (`AWS_PROFILE=pss scripts/ec2-buildfarm.sh <branch>`), at which
  point the orphan-proofing + cost-cap should be confirmed on the live box.
