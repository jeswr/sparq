# Autonomous programmatic scheduler for sparq orchestration — design for review

> 🤖 SPARQ agent — design-for-maintainer-review record. `[OPUS-4.8]` (authored by
> Opus 4.8; Fable unavailable — flag for re-review when Fable returns). **No
> implementation in this PR**: this is a design + phased-bead plan only.
>
> **Goal.** Remove the orchestrator (the conversational lead session) from
> *per-agent dispatch*. Today a human-in-the-loop LLM picks each bead, writes each
> brief, fires each sub-agent, watches each PR, and arms each merge. This design
> proposes a **self-driving scheduler**: a programmatic loop that polls the
> launchable bead frontier, triages and *places* each bead on the cheapest viable
> compute (the local 8-core work box vs ephemeral EC2), fans out
> implement → adversarial-verify, auto-arms the verified-clean non-perf PRs, and
> re-polls — keeping the human in the loop only for performance discretion,
> canonical-number changes, and genuine `needs:user` decisions.

## 0. TL;DR + the recommendation

Build the scheduler as a **long-running `ultracode` Workflow** (Anthropic's dynamic
workflows, shipped 2026-05-28; see §1) — a JavaScript driver that calls
`agent()`/`parallel()`/`pipeline()`/`phase()` to stand agents up *programmatically*,
spending zero model tokens on the coordination glue. The Workflow runs in the
background to completion: it **loops until the frontier is empty or a token/cost
budget is hit**, then exits. There is **no always-on OS daemon** (verified — §1.3);
cross-run continuity over hours/days (reacting to merges) comes from the existing
**event-driven merge-watchers** plus a **cron/`/loop` re-kick** of the
conversational loop that re-launches the Workflow when new work unblocks.

Place work with a **heuristic triage agent**: estimate each bead's compute tier +
parallelism, then **bin-pack** — light beads (docs/scripts/config/small features)
run **LOCAL**, packed up to the CPU ceiling `min(16, cores−2) = 6`; heavy beads
(full feature builds, benchmark campaigns, full feature-matrix) go to **EC2**,
sharing one ephemeral instance across several cheap jobs and dedicating an instance
to a heavy/parallel benchmark — all inside the build-farm's enforced instance cap +
≤ $5/day cost ceiling. The estimate **is a heuristic and will mis-estimate**; the
design treats every estimate as advisory and makes the *ceilings* (CPU, cost,
instance count, per-instance watchdog) the hard constraints that bound the blast
radius of a bad estimate.

Keep the **existing safety model unchanged**: adversarial-verify stays the automated
honesty/correctness gate; the **`sparq-perf-reviewer` PreToolUse hook** (PR #396)
stays the performance-discretion gate on the `gh pr merge … --auto` arming step;
**only verified-clean non-perf PRs auto-arm**; perf-affecting PRs are withheld for
the perf-reviewer; canonical-number changes additionally surface to the maintainer.

## 1. Ground truth — the ACTUAL harness mechanics (stated honestly)

I verified the substrate against the real codebase, the harness toolset, and the
public record rather than taking the brief on faith. Three corrections / clarifications
to the brief's premise follow; the brief is broadly right about the *shape* but
over-states what already exists.

### 1.1 The Workflow tool (`ultracode`) — REAL, but not currently a tool I can call

The brief describes "The Workflow tool (ultracode): JS scripts call
`agent()`/`pipeline()`/`parallel()`/`phase()`; agents are stood up BY THE SCRIPT
(programmatic). A Workflow runs in the background to completion." This is **confirmed
real**: Anthropic shipped *dynamic workflows* + the `ultracode` effort level on
**2026-05-28** (requires Claude Code **v2.1.154+** and a model that supports `xhigh`
effort). A dynamic workflow is a short JavaScript program Claude writes on the fly to
coordinate sub-agents; a runtime executes it in the background, fanning out tens to
hundreds of parallel sub-agents, and **the coordination code spends zero model
tokens**. It is supported in the CLI, Desktop, IDE extensions, headless `claude -p`,
and the Agent SDK.

**Honest caveat (verified):** in *this* session's toolset there is **no `Workflow`
tool surfaced to me** (the deferred-tool set exposes `EnterWorktree`, `Monitor`,
`WebSearch`, etc., but no `Workflow`/`ultracode` callable), and there is **no
`ultracode` binary on the box** nor any `pipeline(`/`parallel(`/`phase(`/`agent(`
reference anywhere in the repo or `~/.claude/`. The Workflow runtime is invoked via
`/effort ultracode` / "use a workflow", not as a JSON tool I call directly. So this
design targets the **documented `ultracode` Workflow API surface** as the intended
implementation substrate; the exact `agent()/parallel()/pipeline()/phase()` signatures
must be confirmed against the live runtime when Phase 1 starts (open question OQ-1).

### 1.2 The three substrate pieces — INTENDED interfaces, NOT yet on `main`

The brief names `scripts/push-frontier.sh`, `scripts/ec2-buildfarm.sh`, and the
perf-gate as "currently being built". Verified status:

| Piece | Brief's framing | Verified status |
|---|---|---|
| `scripts/push-frontier.sh` | launchable frontier printer | **Does not exist** on `main` or any branch. `bd ready`-driven frontier selection exists only as **prose** in `AGENTS.md` (§§ "Maximise parallelism", "Maintenance loop"). The branch `origin/sq-o09o-push-scheduler` is a *SPARQL-server query scheduler* (`crates/sparq-serve/src/scheduler.rs`), **unrelated** to orchestration. |
| `scripts/ec2-buildfarm.sh` | ephemeral cost-capped build/test farm | **Does not exist** by that name. What exists: `scripts/ec2-bench.sh` (ephemeral **spot** instance, `trap cleanup EXIT` always-terminate, ≤ $5/mo CI cadence) + `research/ci-ec2-design.md` (on `origin/ec2-buildfarm`) + the parallel-bench recipe in memory (≤ $5/**day**, `--instance-initiated-shutdown-behavior terminate`, user-data ≤ 45-min watchdog, tag `purpose=sparq-bench`, serial-console result retrieval). The *generic build-farm* is intended, not built. |
| perf-gate | `sparq-perf-reviewer` agent + `gh pr merge` arm hook | **REAL and in-flight** — PR #396 (`perf-discretion-gate`) adds `.claude/agents/sparq-perf-reviewer.md` (a read-only `{perf_affecting, perf_ok, evidence, concerns}` verdict agent) wired as a **`PreToolUse` agent-hook on `Bash`** that fires only on a `gh pr merge … --auto` arming command and returns ALLOW for non-perf/evidenced-clean PRs while returning DENY for perf-affecting-and-not-OK ones. `scripts/perf-gate.py` (the deterministic-vs-timing ratchet) and `bench/perf-baseline.json` are on `main`. |

These named scripts are this design's **assumed substrate interfaces**; Phase 1
*builds* `push-frontier.sh` and `ec2-buildfarm.sh` to the contracts in §§4–5 because
they do not yet exist. I do **not** redesign the perf-gate — I consume it as-is.

### 1.3 There is NO 24/7 orchestration daemon — confirmed

No systemd unit, no crontab, no orchestration service exists in the repo. Today's
orchestration is an **event-driven LLM** that fans out background shell jobs
(`gh run watch <run-id> --exit-status &`) and worktree sub-agents, with a
**`SessionStart` hook** (`scripts/bd-session-context.sh`, injects `bd ready` +
open-count) and a lightweight **maintenance-loop sweep** as a *safety-net floor*
(`AGENTS.md`: "The loop is a SAFETY NET, not the cadence … the primary mode is
event-driven and eager"). The only persistent OS daemon is **roborev** (code review,
fed by a `post-commit` hook) — it reviews, it does not schedule or merge. CPU
budgeting is **prose** ("cap concurrent cargo-heavy agents to the core budget"); no
`nproc`-derived integer is computed anywhere for the orchestrator today. This design
makes that ceiling explicit and computed.

**Consequence for the design (load-bearing):** the scheduler **cannot** be a daemon.
It is a *bounded Workflow run*. Continuity across the multi-hour merge-train drain
relies on (a) the merge-watchers that already exist, and (b) a **cron/`/loop`
re-kick** that re-enters the conversational loop and re-launches the Workflow when the
frontier refills. The doc says this plainly and does not imply always-on behaviour.

### 1.4 Frontier definition + ceilings (the brief's `min(16, cores−2)`)

`nproc` on this box = **8**, so `min(16, cores−2) = min(16, 6) = 6` — the LOCAL
concurrency ceiling. The launchable frontier the brief specifies is:

```text
frontier = (bd ready)  −  (in-flight beads)  −  (conflict-collisions)   capped at 6 LOCAL
```

where *conflict-collision* = a bead whose file-area overlaps an in-flight bead, applied
as a per-crate **conflict-partition: ≤ 1 bead per crate/surface**, with `site` and the
`sparq-server`/`http.rs` `server-auth` path serialised to ≤ 1 (reserve for ONE branch at
a time per `AGENTS.md`). This frontier is exactly what `push-frontier.sh` must print
(§4). Two refinements (sq-8rpq), both load-bearing:

* **The conflict-partition is canonical over the COMBINED set.** If the scheduler ever
  *falls back* to a raw `bd ready` list (push-frontier unavailable, or it blends in extra
  beads), it MUST re-apply the ≤ 1-per-crate / server+site→1 dedup to the **combined**
  push-frontier + bd-ready set — never dispatch the bd-ready fallback un-deduped, or two
  beads on the same crate launch at once and conflict (the original sq-8rpq symptom:
  two `sparq-server` beads dispatched together).
* **In-flight is reserved by open PR + UNPUSHED worktree branches, not every branch.**
  PRs are squash-merged, so a finished feature branch is *not* an ancestor of `main` but
  *was* pushed; the harness never auto-removes its worktree, so hundreds of stale
  branches accumulate. `push-frontier.sh` / `refill-candidates.sh` reserve a worktree
  branch only when it has **unpushed local commits** (`origin/<branch>..HEAD ≠ 0`, or
  never pushed) — an `--no-merged`/ancestor test would flag all of them and empty the
  frontier. Run `worktree-gc.sh --apply` at idle so stale worktrees do not pile up.
* **Partition by primary-CODE crate, not just the label surface (sq-6ip4).** A bead's
  conflict SURFACE is inferred from its title label (e.g. `genai`/`introspect` for its
  READMEs/SKILLs), but its actual `.rs` changes may land in a *different* crate. Wave
  `wm5fcnlqj` picked two `sparq-core` beads in one wave (#597 + #598) because their labels
  diverged yet both edited `crates/sparq-core/src/lib.rs` → merge-conflict risk.
  `push-frontier.sh` therefore reserves each bead on **two lanes**: its label surface
  *and* its **primary-code crate**, inferred from an explicit `crates/<crate>/…/*.rs` path
  in the bead's title+description (a known-crate, `.rs`-only probe — `infer_code_crate`).
  Two beads touching one crate's source never co-launch even when their labels differ.

The two independent ceilings from `AGENTS.md` both apply and are
HARD: the **box CPU budget** (6 cargo-heavy locally) and the **Anthropic API
rate-limit** (~10 concurrent agents trips throttling → keep a *sustained* fleet,
refill as agents land, never a thundering-herd burst). EC2 adds a third hard ceiling:
the **build-farm instance cap + ≤ $5/day cost ceiling**.

## 2. Problem framing

The orchestrator is the bottleneck and the single point of token spend for
coordination. Every dispatch decision (which bead, where, with what brief) costs
model tokens and human attention. Three forces make this worth automating:

1. **Coordination is mechanical.** Frontier selection, conflict-avoidance by
   file-area, fan-out up to a ceiling, arm-on-green — these are rules, not judgment.
   `ultracode` exists precisely so the *code* does coordination at zero token cost
   and the *model* does only the judgment (triage, verify, the genuinely ambiguous).
2. **Compute placement is a cost-optimisation, not a vibe.** A docs bead and a
   full-feature-matrix build have ~100× different compute cost; running both on the
   one quiet work box serialises wall-clock-sensitive measurements behind cheap work.
   Memory already authorises **parallel EC2 bench instances at ≤ $5/day** for exactly
   this — but launching/sharing/reaping them is manual today.
3. **The safety rails already exist.** adversarial-verify + the perf-discretion hook
   + the `ci-summary` gate + roborev mean "auto-arm verified-clean non-perf" is a
   *small* delta over today's manual arm, not a new trust model.

Non-goals: replacing `ci-summary` / branch protection (they remain the authoritative
merge gate); replacing the merge-watchers (the scheduler *drives new work*, the
watchers *drain merges* — same separation as today); any 24/7 service.

## 3. The self-driving scheduler Workflow loop

The scheduler is one `ultracode` Workflow (`scheduler.workflow.js`, intended path
`scripts/workflows/`). Pseudocode of the driver (coordination = zero-token JS; each
`agent(...)` is a model call):

```js
// scheduler.workflow.js  — intended ultracode Workflow (Phase 2). [OPUS-4.8]
const TOKEN_BUDGET = env.SCHED_TOKEN_BUDGET;     // hard stop (see §8 runaway guard)
const COST_CEIL    = env.SCHED_EC2_USD_DAY ?? 5; // ≤ $5/day, enforced by build-farm
let spent = loadCheckpoint();                    // resumeFromRunId crash-recovery (§3.4)

while (!budgetHit(spent, TOKEN_BUDGET)) {
  // (1) POLL the launchable frontier (zero-token shell call)
  const frontier = sh("scripts/push-frontier.sh --json");   // bd ready − in-flight − conflicts, cap 6
  if (frontier.length === 0) break;                          // termination: frontier empty

  // (2) TRIAGE + PLACE — one cheap model call returns a placement plan (§4)
  const plan = await agent("workload-triage", { beads: frontier, ceilings: CEILINGS, costLeft: COST_CEIL - spent.ec2Usd });

  // (3) PLACE: pack LOCAL up to 6; bin-pack EC2 jobs onto shared/dedicated instances (§4–5)
  const local = plan.local;                       // ≤ 6 cargo-heavy locally; docs/research free
  const farm  = await placeOnFarm(plan.ec2);      // scripts/ec2-buildfarm.sh provision/lease

  // (4) FAN OUT impl → adversarial-verify as a per-bead pipeline (parallel across beads)
  const results = await parallel([...local, ...farm].map(job =>
    pipeline([
      phase("implement", () => agent(job.implAgent, job.brief)),       // sparq-rust-feature / -docs / -site …
      phase("verify",    () => agent("adversarial-verify", job.pr)),   // automated honesty/correctness gate (§6)
    ])
  ));

  // (5) AUTO-ARM verified-clean NON-PERF only; perf-affecting → withheld by the hook (§6)
  for (const r of results.filter(r => r.verifyClean)) {
    sh(`gh pr merge ${r.pr} --auto --squash`);     // PreToolUse perf-reviewer hook gates THIS line
  }                                                 // hook DENY ⇒ left for maintainer perf review

  spent = checkpoint(accumulate(spent, results)); // (7) persist progress (§3.4)
  // (6) RE-POLL: loop top re-runs push-frontier.sh — newly-unblocked beads appear next pass
}
emit("scheduler run complete", spent);            // Workflow exits; NOT a daemon
```

### 3.1 Loop stages

- **Poll.** `push-frontier.sh --json` returns the capped frontier (§4). Pure shell,
  zero tokens.
- **Triage + place.** ONE `workload-triage` model call per pass returns a placement
  plan for the whole frontier (cheaper than per-bead calls). Heuristic; see §4.
- **Fan out.** Each bead becomes a `pipeline([implement, verify])`; beads run in
  `parallel(...)`. The driver respects the LOCAL cap (6) and the EC2 lease set; it
  never launches more than the sustainable fleet (API-rate-limit aware — stagger
  launches, refill as jobs land, per `AGENTS.md`).
- **Auto-arm.** For each verified-clean result the driver runs
  `gh pr merge <n> --auto --squash`. The **`sparq-perf-reviewer` PreToolUse hook**
  intercepts that exact command: ALLOW (non-perf / evidenced-clean) → armed;
  DENY (perf-affecting + not-OK, or undeterminable) → left for the maintainer. The
  driver records the DENY and moves on — it does **not** retry-arm.
- **Re-poll.** Loop top re-runs the frontier. A bead that just unblocked (its blocker
  merged) enters next pass.

### 3.2 Termination conditions (all bounded — no infinite run)

1. **Frontier empty** — `push-frontier.sh` returns `[]` (nothing ready, minus
   in-flight/conflicts). Clean exit.
2. **Token budget hit** — `spent ≥ SCHED_TOKEN_BUDGET`. Hard stop (runaway guard §8).
3. **Cost ceiling hit** — cumulative EC2 spend would exceed ≤ $5/day; the farm refuses
   new leases and the driver stops *placing* EC2 work (LOCAL work may continue).
4. **Wall-clock cap** — a `SCHED_MAX_MINUTES` deadline (the Workflow runtime itself is
   background-to-completion; this is belt-and-braces against a hung pipeline).
5. **Kill-switch** — presence of a `scripts/.scheduler-stop` sentinel or a
   `needs:human-stop` bead halts the loop after the current pass (runaway-agent guard).

### 3.3 Re-kicking across merges (the no-daemon reality)

Because the Workflow exits when the frontier drains, multi-hour continuity is NOT the
Workflow's job. Two existing mechanisms cover it:

- **Merge-watchers (event-driven, primary).** The existing background CI-watchers
  (`gh run watch … &`) drain the merge train one branch at a time. When a PR merges,
  beads it blocked unblock. If the scheduler Workflow is still running, its next
  `push-frontier.sh` poll picks them up; if it already exited, →
- **Cron / `/loop` re-kick (safety-net floor).** A low-frequency `/loop` (or the
  `/schedule` cron skill) re-enters the **conversational loop**, which checks "is the
  frontier non-empty and no scheduler Workflow in flight?" and, if so, **re-launches
  the scheduler Workflow**. This is the same "SAFETY NET, not the cadence" model
  `AGENTS.md` already mandates — the re-kick exists to guarantee forward progress when
  nothing else is driving, not as a clock. The doc is explicit: this is *not* a 24/7
  daemon; it is a bounded Workflow re-launched on demand and on a slow floor.

### 3.4 Idempotency + crash-recovery (`resumeFromRunId`)

- **Single-flight.** Before launching, the re-kick checks for an in-flight scheduler
  run (a `scripts/.scheduler-lock` PID/run-id file); never run two schedulers at once
  (double-arming hazard). The lock records the `runId`.
- **Resume.** `ultracode` workflows support resuming a prior run; on a crash/restart
  the re-kick passes `resumeFromRunId` (OQ-1: confirm the exact API) so completed
  pipeline phases are not re-run and in-flight PRs are reconciled, not duplicated.
- **Idempotent placement.** Every job carries the bead id; `push-frontier.sh`
  subtracts *in-flight* beads (a bead with an open PR or a live worktree), so a resume
  never re-dispatches a bead that already has a branch. This reuses today's
  ground-truth reconciliation (`git worktree list` + `gh pr list` + `pgrep -af cargo`).
- **EC2 idempotency.** Leases are tagged with the bead id + run id; a resume reclaims
  or reaps existing leases by tag rather than launching duplicates. Orphan-check
  (`describe-instances` by tag + `pgrep -af ec2-buildfarm`) runs at every re-kick.

## 4. Workload-triage / placement agent + policy

A `workload-triage` agent (model call, one per pass) consumes the frontier and emits
a placement plan. **Stated plainly: the compute estimate is a heuristic and *will*
mis-estimate** — it is advisory input to a bin-packer whose *ceilings* are the real
safety guarantee.

### 4.1 The heuristic (honest about its limits)

Per bead, estimate `{tier, parallelism, est_minutes, mem}` from cheap, observable
signals:

- **Crate / file-area** — `research/`, `skills/`, `docs/`, `*.md`, config → LIGHT
  (no Rust build). `sparq-core`/`sparq-engine` hot paths, `bench/**`, `zk/**`,
  feature-matrix builds → HEAVY. `sparq-server` → contended (serialise).
- **Bead type** (`bd show`) — `chore`/`docs`/`spike` lean light; `feature`/`bug` on a
  heavy crate lean heavy; an `epic` is decomposed, never placed directly.
- **Rough historical build cost** — a small `bench/build-cost.json` lookup of
  recent per-crate `cargo build/test` wall-time *on this box* (seed it from observed
  values; refine over time). **NON-canonical** (work-box timing) — used only for
  *placement*, never reported as a perf result.

It mis-estimates predictably: a one-line fix in `sparq-core` looks HEAVY (whole-crate
rebuild) but is fast; a "docs" bead that regenerates the dashboard is heavier than it
looks. The design does **not** try to be clever here — it accepts mis-estimates and
relies on the ceilings + the per-instance watchdog + the cost cap to bound the cost
of any wrong guess.

### 4.2 Placement policy (one paragraph)

Place LIGHT beads **LOCAL**, packing them onto the 8-core box up to the
`min(16, cores−2) = 6` cargo-heavy ceiling (docs/research/config agents do not count
against it and parallelise freely, subject only to the API-rate-limit fleet cap);
place HEAVY beads on **EC2** via `ec2-buildfarm.sh`, **bin-packing** cheap heavy jobs
onto a *shared* leased instance (several feature-builds time-sliced on one
c7g.xlarge) while **dedicating** a fresh instance to each wall-clock-sensitive /
parallel **benchmark** job (a benchmark must own a quiet box — §5); the bin-packer's
objective is *maximum parallelism per dollar* — fill a leased instance before
provisioning another, prefer the smallest Graviton that fits, reuse a warm lease
within its watchdog window rather than relaunching, and **never** exceed the
build-farm's enforced instance cap (`SCHED_MAX_INSTANCES`, default small, e.g. 3) or
the ≤ $5/day cost ceiling, both of which are hard stops that the farm refuses to
cross regardless of what the (heuristic, fallible) estimate says.

### 4.3 How placement respects the farm's enforced caps

`ec2-buildfarm.sh` (Phase 4) is the **sole** authority on instances. The triage agent
*requests*; the farm *grants or refuses*. The farm enforces, independent of the
scheduler:

- `--instance-initiated-shutdown-behavior terminate` + user-data **≤ 45-min
  watchdog** (`( sleep 2700; shutdown -h now ) &` as the first boot action) → every
  instance self-terminates even if the lease manager dies (the disowned-launcher
  hazard from memory: the farm launcher runs in the **foreground**, dies with its
  agent, and its `trap cleanup EXIT` terminates by tag — never `nohup`/`&`-detached).
- `tag purpose=sparq-bench` (matches the orphan-check filter already in memory).
- An **AWS Budgets** alarm + a pre-lease cost check: the farm queries cumulative
  tagged spend and refuses a lease that would breach ≤ $5/day.
- A hard `SCHED_MAX_INSTANCES` cap; spot pricing; never touches prod
  `i-090531b4ede8f2d3f` or the dev box `i-00f76802f345b6b77`.
- **Spot interruption tolerance** — build/test jobs are idempotent (re-runnable from
  the bead branch); a spot reclaim just re-leases and re-runs, no state lost.

## 5. Compute-sharing for benchmarks (the maintainer's example)

Benchmarks are the case where placement matters most, and where the NON-canonical
caveat is load-bearing. Policy:

- **Dedicate, don't share, for wall-clock-sensitive benchmark runs.** A benchmark that
  measures throughput/latency needs a *quiet* box; co-tenanting it with a build
  poisons the measurement. So each such benchmark job gets a **dedicated** ephemeral
  instance (sized to the sweep: c7g.large/xlarge for routine, a larger Graviton only
  for a rare manual NUMA sweep, per `ci-ec2-design.md`'s cost math).
- **Fan a benchmark *matrix* across instances, share within a config.** A matrix
  (e.g. several scales × codecs) fans out as N dedicated instances **in parallel**
  (each quiet, each its own config), then results are gathered and the instances
  reaped. Where two *non-timing* steps of a benchmark share setup (e.g. a build that
  feeds two measurements), they reuse the one warm lease within its watchdog window —
  sharing the *build*, not the *measurement*.
- **Reuse leases across cheap heavy builds, not across measurements.** The bin-packer
  packs several feature-matrix *builds* onto one shared instance (CPU-bound, timing
  doesn't matter); it never packs two *measurements* together.
- **Orphan-proof + cost-cap intact.** Every benchmark lease carries the ≤ 45-min (or,
  for a sanctioned long sweep, ≤ 3-hour) watchdog, `terminate`-on-shutdown, the tag,
  the foreground launcher, and the orphan-check at re-kick. Result retrieval uses the
  **serial-console marker** pattern from memory
  (`=== SPARQ_BENCH_RESULT === … === END ===` via `aws ec2 get-console-output`) when
  no keypair/SSM/S3 is available, or the `ec2-bench.sh` SSH-fetch path when it is.
- **NON-canonical caveat PRESERVED (hard rule).** Any number a build-farm / EC2
  instance produces is **NON-canonical** — it is *not* a CI-runner / controlled-quiet-box
  result. The scheduler returns benchmark output **labelled NON-canonical**, never
  bakes it into markdown, and never lets it arm a perf claim. The authoritative perf
  source remains the CI `bench.yml` series + `bench/perf-baseline.json`. The
  `sparq-perf-reviewer` hook (§6) independently enforces this on any PR that tries to
  present such a number as canonical. This is consistent with the privacy-claims +
  no-hardcoded-perf-numbers gates that are live on `main`.

## 6. Safety / merge model (unchanged trust model)

The scheduler **automates dispatch, not trust**. The gates are exactly today's:

1. **adversarial-verify = the automated honesty/correctness gate.** Every impl PR
   passes through an `adversarial-verify` phase (the existing verify agent) before it
   can be armed. It catches fabrication, scope creep, false claims, broken tests —
   the honesty job. A verify failure → the PR is *not* armed; the bead is bounced back
   (re-brief or surfaced to the maintainer). This is non-negotiable and is the same
   bar a human orchestrator applies today.
2. **Auto-arm ONLY verified-clean NON-perf PRs.** The driver runs
   `gh pr merge … --auto` only for verify-clean results. The **`sparq-perf-reviewer`
   PreToolUse hook** (PR #396) intercepts that command and is the
   **performance-discretion gate**: `perf_affecting=false` → ALLOW (armed);
   `perf_affecting=true ∧ perf_ok=true` (evidenced-clean) → ALLOW;
   `perf_affecting=true ∧ perf_ok=false` (or undeterminable) → **DENY**, withheld for
   maintainer perf review. The scheduler treats a DENY as terminal-for-this-pass
   (records it, does not retry).
3. **Perf-affecting → perf-reviewer (human discretion retained).** This is the
   *entire reason* manual arming existed; the design preserves it 1:1 — the human
   keeps discretion over performance, the machine handles everything else.
4. **Canonical-number changes additionally surface to the maintainer.** The
   perf-reviewer flags `canonical_number_change` (a `bench/perf-baseline.json` floor
   move or a published-results artifact) in `evidence` even on ALLOW; the scheduler
   forwards that to a `needs:user` surface so a floor change — a policy decision — is
   always seen, never silently auto-armed.
5. **`ci-summary` + branch protection remain the authoritative merge gate.** Arming
   ≠ merging. `--auto` only merges once `ci-summary` is green and review threads are
   resolved; the merge-watchers do the actual squash-merge. The scheduler never
   bypasses branch protection, roborev, or the `ci-summary` aggregator.

Net new trust delta vs today: the *arm* decision for **verified-clean non-perf** PRs
moves from human to machine. Everything else (verify bar, perf discretion,
canonical-number surfacing, `ci-summary`) is unchanged.

## 7. Oversight + observability (design requirement)

Automating dispatch removes the human from the *loop* but must **not** remove them
from *oversight*. A self-driving scheduler that the maintainer cannot watch, pause, or
stop is a runaway hazard, not a productivity win. This section makes oversight a
first-class, **non-optional** part of the design: every autonomous decision is
**logged with its reason**, the live state is **readable at a glance**, the maintainer
has a **PAUSE and a KILL** control, and the two classes of decision that are *policy*
rather than *mechanics* — performance and canonical numbers — are **escalated**, never
silently auto-armed. The existing runaway checkpoints (no-auto-merge-of-perf,
adversarial-verify, the perf-gate) stay exactly as in §6; this section adds the
*visibility and the brakes* on top of them.

### 7.1 Structured decision LOG (append-only, every launch + arm + reason)

The scheduler emits a single **append-only, structured (JSON-lines) decision log** —
one record per autonomous decision, each carrying *why*. This is the audit trail: after
the fact, the maintainer (or a post-mortem) can reconstruct exactly what the scheduler
did and on what basis. It is the scheduler's analogue of the bead `interactions.jsonl`
ledger — machine-readable, append-only, never rewritten.

- **Path + format.** `scripts/.scheduler/decisions.jsonl` (gitignored — it is run state,
  not a tracked narrative doc, per the §0 / `AGENTS.md` no-scratch-docs rule). One JSON
  object per line, never edited in place. A record:

  ```jsonc
  { "ts": "2026-06-17T21:50:20Z", "run_id": "wf_…", "pass": 3,
    "event": "launch",              // launch | place | arm | arm_withheld | escalate
    //                                 | skip | pause | resume | kill | terminate
    "bead": "sq-xxxx",
    "decision": "dispatch LOCAL",   // the action taken
    "reason": "LIGHT tier (docs); 2/6 local slots free; no crate conflict",
    "evidence": { "tier": "light", "local_inflight": 2, "conflict": null },
    "outcome": "ok" }               // ok | denied | error | deferred
  ```

- **Every launch decision is logged** — which bead, where it was placed (LOCAL / which
  EC2 lease), and the reason (tier estimate, free-slot count, conflict-partition
  result). A bead the frontier *skipped* is logged too (`event:"skip"`, with the reason:
  crate conflict, in-flight, ceiling full) so "why didn't it pick bead X?" is always
  answerable from the log, not from re-running the heuristic.
- **Every arm decision is logged** — `event:"arm"` (the `gh pr merge … --auto` ran) or
  `event:"arm_withheld"` when the `sparq-perf-reviewer` hook returned DENY, with the
  hook's `{perf_affecting, perf_ok, concerns}` verdict copied into `evidence`. A withheld
  arm is *also* an escalation (§7.4) — the log and the escalation surface are written
  together so the two never diverge.
- **Every escalation, pause, resume, and kill is logged** with the trigger (sentinel
  file appeared, `needs:human-stop` bead, token/cost ceiling hit, canonical-number
  change detected). The log is the single source of truth for "what made it stop".
- **Reason is mandatory.** A record without a non-empty `reason` is a bug; the driver
  asserts it before appending. The whole point is that *no autonomous action is
  unexplained*.
- **Rotation.** The driver rotates the file at a size cap (e.g. 50 MB →
  `decisions.<ts>.jsonl`) so a long-running fleet never fills the disk; rotation itself
  is logged. (Disk-pressure discipline per the repo's disk-space rule.)

### 7.2 Readable live STATUS surface

Distinct from the *historical* log, the maintainer needs a **right-now** view: what is
the scheduler doing *this second*? The driver maintains a single, **human-readable,
atomically-rewritten** status file plus a one-line renderer.

- **Path.** `scripts/.scheduler/STATUS.md` (gitignored run state — *not* a committed
  `STATUS.md` narrative doc; it lives under the scheduler's dot-dir and is regenerated,
  never reviewed/merged). Rewritten atomically (write-temp-then-rename) at the end of
  every pass and on every state transition so a reader never sees a half-written file.
- **Contents (at a glance):** run id + start time + elapsed; current pass number; the
  live fleet — each in-flight bead with its placement (LOCAL / EC2-lease-id), its
  pipeline phase (`implement` / `verify`), and elapsed; LOCAL slot usage (`4/6`); EC2
  lease count + cumulative tagged spend vs the ≤ $5/day ceiling; token spend vs
  `SCHED_TOKEN_BUDGET`; the count of armed / withheld PRs this run; the **pause/kill
  state**; and the **open-escalation count** (PRs withheld for perf, canonical-number
  changes awaiting the maintainer). It ends with the tail of the most recent decision
  reasons so the *why* of the current state is visible without opening the JSONL.
- **`scripts/scheduler-status.sh`** — a zero-token shell renderer that prints
  `STATUS.md` (or a compact one-line summary for a terminal/notification:
  `sched run wf_… pass 3 · local 4/6 · ec2 1 ($1.20/5) · armed 2 withheld 1 · ESCALATIONS 1 · RUNNING`).
  This is what a `/loop` re-kick or a glance command shows the maintainer; it is also
  what the stalled-frontier alert (§8) reads.
- **Liveness.** The status file carries a `heartbeat_ts` updated every pass; a renderer
  that sees a stale heartbeat with state `RUNNING` reports `STALE?` so a hung Workflow
  is visible (it cannot update its own status when wedged — staleness *is* the signal).

### 7.3 PAUSE / KILL switch (graceful brake + hard stop)

The maintainer must be able to **stop the autonomy at any time** without killing
in-flight work mid-PR. Two distinct controls, both honoured at the **pass boundary**
(the loop checks them at the top of every pass, after the current parallel batch
settles — so no half-implemented bead is abandoned):

- **PAUSE (graceful).** A `scripts/.scheduler/PAUSE` sentinel file **or** a
  `needs:human-stop` bead halts *new* dispatch: the loop finishes the in-flight batch
  (lets running pipelines complete and their clean non-perf PRs arm normally), then
  **stops placing new work** and idles in a poll-only state, rewriting `STATUS.md` to
  `PAUSED` and logging `event:"pause"`. Removing the sentinel (or closing the bead)
  resumes (`event:"resume"`). PAUSE is reversible and loses no work — it is the
  "hold on, let me look" brake.
- **KILL (hard stop).** A `scripts/.scheduler-stop` sentinel (the existing kill-switch
  name from §8/§10) **or** a `needs:human-stop` bead with a `kill` marker halts the loop
  **after the current pass**, then **reaps**: it tears down EC2 leases by tag
  (`describe-instances` + terminate), releases the single-flight lock, writes
  `STATUS.md` to `KILLED`, logs `event:"kill"` + per-lease `event:"terminate"`, and
  exits the Workflow. In-flight local pipelines are allowed to finish their current
  phase (so a PR is never left in a corrupt half-open state) but no new phase starts.
  KILL is the "stop everything now" control; it is **not** reversible — re-kicking
  starts a fresh run.
- **Belt-and-braces.** Even if neither switch is honoured (a wedged driver), the
  independent ceilings still bound the blast radius: the ≤ 45-min per-instance watchdog
  terminates every leased box, the `SCHED_TOKEN_BUDGET` hard-stops model spend, and the
  single-flight lock prevents a second scheduler from compounding the problem. The
  switches are the *fast* brake; the ceilings are the *backstop* (§8).
- **Discoverability.** Both sentinel paths and the `needs:human-stop` bead label are
  documented in `STATUS.md`'s footer and in `scheduler-status.sh --help`, so the
  maintainer never has to read code to find the stop control.

### 7.4 ESCALATE perf / canonical-number changes (policy, not mechanics)

Two classes of change are **policy decisions reserved to the maintainer** and must be
escalated, never silently auto-armed — this is the human-discretion boundary from §6
made into an explicit, *visible* surface:

- **Performance-affecting PRs.** The `sparq-perf-reviewer` PreToolUse hook (§6, item 2)
  already returns DENY for a perf-affecting-and-not-evidenced-clean PR, so it never
  auto-arms.
  The oversight layer makes the *withholding visible*: every DENY writes an
  `event:"escalate"` decision-log record **and** increments the `STATUS.md`
  open-escalation count **and** appends the PR to a `scripts/.scheduler/escalations.jsonl`
  queue with the hook's `concerns`. The maintainer sees "1 PR withheld for perf review"
  in the status one-liner and can resolve it; the scheduler never retries the arm.
- **Canonical-number changes.** Any change that moves a *canonical* number — a
  `bench/perf-baseline.json` floor move, an edit to a published-results artifact, or a
  hard-coded perf number entering markdown (the no-hardcoded-perf rule) — is a **policy
  change**, even when the perf hook would otherwise ALLOW. The perf-reviewer flags
  `canonical_number_change` in its `evidence` (§6, item 4); on that flag the scheduler **forces
  an escalation regardless of the ALLOW**: it withholds the arm, logs `event:"escalate"`
  with `reason:"canonical-number change — maintainer policy decision"`, and surfaces a
  `needs:user` bead. A floor change is *never* auto-armed. (This composes with the
  privacy-claims + no-hardcoded-perf gates already live on `main` — the scheduler is one
  more enforcement point, not a bypass.)
- **The escalation surface is the `needs:user` queue.** Escalations map onto the existing
  human-in-the-loop tracker surface (`bd list -l needs:user`, per `AGENTS.md`) so nothing
  awaiting a maintainer decision is lost in a chat scroll or a JSONL file the maintainer
  never opens. The `escalations.jsonl` queue + the `STATUS.md` count are the *fast* view;
  the `needs:user` bead is the *durable* one. They are written together.

### 7.5 The runaway checkpoints are PRESERVED (not replaced by oversight)

Oversight is *additive*. The three runaway checkpoints from §6 remain the hard safety
floor and are **not** weakened by adding visibility:

1. **No auto-merge of perf-affecting PRs** — the perf hook DENY (now also an escalation).
2. **adversarial-verify gates every PR pre-arm** — the honesty/correctness gate.
3. **The perf-gate** (`scripts/perf-gate.py` + `bench/perf-baseline.json`) — the
   deterministic-vs-timing ratchet on the merge.

The decision log, STATUS surface, and PAUSE/KILL switch make these checkpoints
*observable and interruptible*; they do not move the gates. A scheduler with full
oversight but a removed checkpoint would be *worse* than today, not better — so the
checkpoints stay, and oversight watches them.

## 8. Honest risks + guards

- **No-daemon limit (inherent).** The scheduler is a bounded Workflow, not an
  always-on service. Between runs, nothing is driving except the merge-watchers and
  the cron re-kick. If both the watchers and the re-kick fail, work stalls silently
  until the next session. *Mitigation:* the cron/`/loop` floor is the explicit
  forward-progress guarantee; a stalled-frontier alert (a re-kick that finds a
  non-empty frontier with no scheduler in flight for > T) surfaces to the maintainer.
  This is a real limit, not a bug — it is the honest consequence of having no daemon.
- **Compute estimation is a heuristic — it WILL mis-estimate (§4.1).** A wrong tier
  estimate wastes some compute (a LIGHT bead sent to EC2, or a HEAVY bead clogging a
  LOCAL slot). *Mitigation:* the estimate is advisory; the ceilings (CPU 6, instance
  cap, ≤ $5/day, ≤ 45-min watchdog) bound the cost of any single bad guess to cents;
  `build-cost.json` refines over time. Never presented as a perf measurement.
- **Runaway cost.** A bug that leases instances in a loop. *Guards (defence in
  depth):* (1) farm-enforced `SCHED_MAX_INSTANCES`; (2) pre-lease cumulative-spend
  check vs ≤ $5/day; (3) AWS Budgets alarm backstop; (4) per-instance ≤ 45-min
  `terminate`-on-shutdown watchdog (worst case, every leaked instance dies in 45 min);
  (5) orphan-check (`describe-instances` by tag + `pgrep -af ec2-buildfarm`) at every
  re-kick; (6) foreground launcher (no disowned process relaunching instances — the
  documented 2026-06-15 hazard).
- **Runaway agent / merge.** A bug that arms PRs it shouldn't, or loops fanning
  agents. *Guards:* (1) the perf-reviewer hook returns DENY for anything perf-affecting; (2)
  adversarial-verify gates every PR pre-arm; (3) `ci-summary` + branch protection gate
  the actual merge (arming ≠ merging); (4) `SCHED_TOKEN_BUDGET` hard stop; (5)
  single-flight lock (no two schedulers); (6) `scripts/.scheduler-stop` kill-switch +
  `needs:human-stop` bead halt the loop after the current pass; (7) API-rate-limit
  fleet cap prevents thundering-herd bursts.
- **The CPU + cost ceilings are HARD constraints, not targets.** `min(16, cores−2)=6`
  LOCAL, `SCHED_MAX_INSTANCES` + ≤ $5/day EC2. The scheduler must treat a ceiling
  breach as "do not place", never "place anyway and apologise".
- **Workflow-API uncertainty (OQ-1).** The exact `ultracode`
  `agent()/parallel()/pipeline()/phase()/resumeFromRunId` signatures are not verifiable
  from this session; Phase 1 must confirm them against the live v2.1.154+ runtime
  before Phase 2 builds the loop on top.
- **adversarial-verify is itself an LLM** — it can miss things. *Mitigation:* it is
  one of *several* independent gates (roborev's non-Anthropic reviewer, `ci-summary`,
  the perf hook, the maintainer on canonical numbers); no single gate is the sole
  trust anchor.

## 9. Options considered (trade-offs)

| Option | Pros | Cons | Verdict |
|---|---|---|---|
| **A. `ultracode` Workflow loop (recommended)** | zero-token coordination; programmatic fan-out; reuses every existing gate; bounded run = no daemon risk | bounded run needs a re-kick; depends on a runtime whose API I couldn't verify here (OQ-1) | **Adopt** |
| B. Always-on OS daemon (systemd) | true continuity | contradicts the verified no-daemon posture; new long-lived attack surface + secret handling; harder to bound cost/runaway; over-engineered for a single-maintainer repo | Reject |
| C. GitHub Actions cron only | already have CI crons; no new runtime | Actions can't stand up *worktree sub-agents* or call the Anthropic API as an orchestrator; can't do local-vs-EC2 placement; wrong layer | Reject as the *scheduler*; keep Actions for the merge/flow-on side it already owns |
| D. Keep manual dispatch, add only `push-frontier.sh` | smallest change; human keeps full control | does not meet the goal (remove orchestrator from per-agent dispatch); the bottleneck remains | Partial — `push-frontier.sh` is Phase 1 of A anyway |

Prior art consulted (cost-aware bin-packing for ephemeral/spot batch workers — the
Kubernetes/Karpenter "least-waste bin-packing + spot for fault-tolerant batch"
pattern, and the *Stratus* cost-aware container scheduler) confirms the §4.2 policy
is conventional: **pack batch jobs densely on the fewest nodes, use spot for
fault-tolerant short-lived work, dedicate for latency-sensitive runs.** sparq's twist
is the local-box tier + the benchmark-must-be-quiet constraint + the NON-canonical
caveat. (Sources in the report-back to the orchestrator.)

## 10. Phased implementation plan (each phase = a future bead under a scheduler epic)

Proposed epic: **`autonomous scheduler`** (orchestrator to create via `bd create`;
this design doc is the epic's anchor). Ordered, each phase a crisp future bead:

1. **Phase 1 — `push-frontier.sh` + frontier contract.** Build
   `scripts/push-frontier.sh --json` printing `bd ready − in-flight − conflict-collisions`
   capped at `min(16, cores−2)`. In-flight = open PR or live worktree; conflict =
   file-area overlap (reserve `sparq-server`). Unit-tested with a hermetic `bd`
   fixture. *Also* in Phase 1: confirm the live `ultracode` Workflow API
   (`agent/parallel/pipeline/phase/resumeFromRunId`) — resolve OQ-1. Depends on:
   nothing. Unblocks: 2.
2. **Phase 2 — `workload-triage` agent + `build-cost.json` seed.** A read-only agent
   that consumes the frontier and emits a placement plan `{local[], ec2[]}` from the
   §4.1 heuristic; seed `bench/build-cost.json` from observed per-crate build times
   (NON-canonical, placement-only). Depends on: 1. Unblocks: 3, 4.
3. **Phase 3 — `ec2-buildfarm.sh` lease/reap + bin-packer.** The cost-capped,
   orphan-proof, foreground build-farm: lease (spot, tag, ≤ 45-min watchdog,
   `terminate`-on-shutdown, foreground + `trap cleanup EXIT`), bin-pack cheap heavy
   builds onto a shared lease, dedicate per benchmark, enforce `SCHED_MAX_INSTANCES` +
   pre-lease ≤ $5/day check + AWS Budgets backstop + orphan-check. Depends on: 2.
   Unblocks: 5, 6.
4. **Phase 4 — scheduler Workflow loop (no auto-arm yet).** `scheduler.workflow.js`:
   poll → triage → place LOCAL → fan out `pipeline([implement, verify])` → re-poll;
   termination conditions; single-flight lock; `resumeFromRunId` crash-recovery.
   **Arming stays manual in this phase** (dry-run the arm, log what it *would* arm).
   Depends on: 1, 2. Unblocks: 5.
5. **Phase 5 — auto-arm verified-clean non-perf (consume the perf hook).** Turn on
   `gh pr merge … --auto` for verify-clean results, relying on the merged
   `sparq-perf-reviewer` PreToolUse hook (PR #396) to gate perf-affecting PRs and the
   canonical-number surface for floor changes. Gated behind a config flag, default
   off, enabled after Phase 4's dry-run is observed clean. Depends on: 4, and PR #396
   merged to `main`. Unblocks: 6, 7.
6. **Phase 6 — EC2 placement + benchmark compute-sharing.** Wire the bin-packer
   (Phase 3) into the loop: place HEAVY beads on shared leases, dedicate per
   benchmark, fan a benchmark matrix across instances, return results with the
   NON-canonical caveat. Depends on: 3, 5. Unblocks: 7.
7. **Phase 7 — cross-run re-kick + crash-recovery hardening.** The cron/`/loop` floor
   that re-launches the scheduler when the frontier refills and no run is in flight;
   stalled-frontier alert; full `resumeFromRunId` reconciliation of in-flight
   PRs/leases; the `scripts/.scheduler-stop` kill-switch + `needs:human-stop` halt.
   Depends on: 5 (and 6 for EC2 lease reconciliation). Closes the epic.

Suggested labels per phase: `scheduler`, plus `needs:user` on Phase 5's enablement
flag (the maintainer should green-light auto-arm going live).

## 11. Open questions for the maintainer

- **OQ-1 (blocking Phase 2/4).** The exact `ultracode` Workflow API
  (`agent()/parallel()/pipeline()/phase()` signatures, background-run lifecycle,
  `resumeFromRunId` semantics, token-budget hooks) is **not verifiable from this
  session** — confirm against the live v2.1.154+ runtime. If the API differs
  materially, §3's loop shape may need adjustment.
- **OQ-2.** Auto-arm rollout posture: enable Phase 5 globally, or first scope it to
  *only* `research/`/`docs/`/`skills/` (lowest-risk, definitionally non-perf) PRs and
  widen after observation?
- **OQ-3.** EC2 hard caps: confirm `SCHED_MAX_INSTANCES` (proposed 3) and whether the
  ceiling is ≤ $5/**day** (parallel-bench memory) vs the ≤ $5/**month** CI-bench
  cadence — they are different budgets; the scheduler should use the per-day one for
  on-demand placement and respect the AWS Budgets alarm as the backstop.
- **OQ-4.** Should `build-cost.json` live in-repo (versioned, reviewed) or stay a
  gitignored local cache (it is NON-canonical work-box timing — keeping it out of the
  repo avoids any appearance of baking perf numbers into the tree)?
- **OQ-5.** Kill-switch ergonomics: a sentinel file vs a `needs:human-stop` bead vs
  both — which does the maintainer want as the canonical "stop the autonomy now"
  control?

## 12. Multi-account agent fan-out — spilling overflow to a SECOND account `[OPUS-4.8]`

> **Status:** DESIGN ONLY (bead sq-fi8v). The implementation needs a **second account
> credential** the maintainer does not yet supply — that makes it `needs:user`. This
> section designs the routing + isolation so that turning it on is a **clean
> credential-flip** (§12.6): provide one secret, set one env var, and the spill path
> activates with **zero code changes**. Until then the scheduler runs single-account
> exactly as §§1–11 describe; account-B routing is dormant (the spill predicate is
> always-false when the credential is absent). This is the standing answer to the
> recurring **weekly-limit stall** the orchestration tick warns about — instead of
> idling the fleet when account-A nears its cap, route the overflow to account-B.

### 12.1 The problem + an HONEST correction to the brief's framing

The brief asks to "launch headless `claude -p` subprocesses under a SEPARATE account
credential … to scale past single-session **weekly** limits." Two corrections, both
verified against the live harness rather than taken on faith:

* **Verified (the mechanism is real).** The installed CLI is **`claude` v2.1.177** and
  exposes headless print mode `-p` / `--print` plus **`--max-budget-usd <amount>`** (the
  CLI help states the budget flag "only works with `--print`"). So a cost-capped
  headless subprocess — `claude -p "<brief>" --max-budget-usd N …` — is a real,
  callable primitive today, not aspirational. This section's spill mechanism is
  therefore buildable; only the *second credential* is missing.
* **Correction (what "limit" means + why a 2nd account, not a 2nd session).** The cap
  the orchestration tick actually trips is the **account/plan usage limit** (the
  weekly/whatever-window quota on the maintainer's plan), **not** a per-*session* limit.
  A second *session* on the **same** account shares that same pooled quota, so it buys
  nothing once the account is near its cap — it would just hit the same wall in
  parallel. The only thing that genuinely adds headroom is a **distinct billing
  identity**: a second Anthropic account / API key, or a 3P provider (Bedrock / Vertex /
  Foundry) account, whose usage is metered **separately**. So the brief's instinct is
  right but the precise framing is "route overflow to a **separately-metered account**,"
  and the mechanism is a headless subprocess that authenticates as that other identity.
  The headroom is bounded by **account-B's own plan limits** — this is *more capacity*,
  not *unlimited* capacity (honest: it does not make the limit disappear, it adds a
  second bucket).

Why a subprocess and not the in-process `agent()` fan-out from §3: a Workflow `agent()`
call (or any sub-agent the orchestrator spawns) runs under **this session's** credential
— the *same* account that is near its cap. Authenticating as a *different* account from
inside one process is fragile (the credential is read from the environment / settings at
client construction; see §12.3). Spawning a **fresh `claude -p` process with a different
environment block** is the clean isolation boundary: the child reads *its* env, never the
parent's. So account-B work is always an OS subprocess, never an in-process agent.

### 12.2 Where this slots into the scheduler loop (the routing point)

The placement bin-packer of §4 gains a **third tier** beyond LOCAL and EC2: **ACCOUNT-B
(headless-spill)**. The routing decision lives in the same `workload-triage` /
`placeOnFarm` step, gated by a single spill predicate evaluated each pass:

```text
spill_to_B  ⇐  ACCOUNT_B_ENABLED                      // credential present (§12.6)
            ∧  near_weekly_limit(account_A)            // §12.4 detector
            ∧  frontier_nonempty_after_local_ec2       // there is overflow to place
            ∧  not cost_ceiling_hit(account_B)         // §12.5 per-run + cumulative cap
```

When `spill_to_B` holds, the driver routes the **next eligible bead(s)** (LIGHT/medium,
LOCAL-tier work — see §12.7 eligibility) to a headless `claude -p` subprocess
authenticated as account-B, instead of launching an account-A agent. Account-A keeps
draining the work it already holds; only *new* dispatch spills. This mirrors §3.1's
"refill as agents land" discipline — B is just another sustainable lane with its own
ceiling. Every spill decision is written to the §7.1 decision log
(`event:"place"`, `decision:"dispatch ACCOUNT-B"`, `reason:"account-A near weekly cap;
B enabled; under B cost ceiling"`) and the §7.2 STATUS surface gains a
`acctB n ($x/cap)` field so the maintainer can see spill at a glance.

### 12.3 Credential-isolation model (never co-mingled) — the load-bearing safety property

The hard rule: **account-A and account-B credentials never share a process environment,
never appear in the same settings file, and are never both resolvable at once.** Each
headless child gets a **purpose-built, minimal environment block** containing exactly one
identity. Verified credential-resolution facts from the CLI help (`claude --help`,
v2.1.177) that this model relies on:

* **First-party Anthropic auth is strictly `ANTHROPIC_API_KEY` (or `apiKeyHelper` via
  `--settings`); OAuth and the keychain are *never* read in headless `-p` mode.** That is
  exactly what we want for a subprocess: auth is a pure function of the child's
  environment, with no ambient OAuth/keychain leakage from the parent session.
* **3P providers (Bedrock / Vertex / Foundry) use their own credentials** (e.g. the AWS
  credential chain for Bedrock, ADC for Vertex). So an account-B that is a *cloud
  provider* is isolated by giving the child *only* that provider's env vars (and
  `CLAUDE_CODE_USE_BEDROCK=1` / `CLAUDE_CODE_USE_VERTEX=1` as applicable), with
  `ANTHROPIC_API_KEY` **unset** in the child.
* **A documented foot-gun we must defend against (from the auth precedence rules):** a
  *set* `ANTHROPIC_API_KEY` shadows every other credential source, and an **empty**
  `ANTHROPIC_API_KEY=""` still *wins its precedence slot* and authenticates with an empty
  key (a guaranteed-failing request, not a fall-through). So the child env must either
  carry account-B's key in `ANTHROPIC_API_KEY` **or have the variable truly unset**
  (`env -u ANTHROPIC_API_KEY` / not present in the constructed block) — never set-to-empty.

The isolation is built as a **launcher that constructs the child environment from
scratch** rather than inheriting the parent's:

```bash
# scripts/spill-launch.sh  — intended Phase B launcher. [OPUS-4.8]
# Builds a MINIMAL env block for account-B; never inherits account-A's credential.
# Reads account-B's secret from a SECRET STORE, not from a file in the repo (§12.6).
set -euo pipefail
brief_file="$1"; budget_usd="$2"; worktree="$3"          # see §12.4 caller

# 1. Resolve account-B's secret name (env var NAME only; value comes from the store).
secret_name="${SPARQ_ACCT_B_SECRET_NAME:?account-B not configured}"   # e.g. "SPARQ_ACCT_B_ANTHROPIC_API_KEY"
acct_b_key="$(load_secret "$secret_name")"               # from keychain/SM/CI secret — NEVER printed, NEVER logged

# 2. Run the child with a SCRUBBED, single-identity environment.
#    `env -i` starts empty; we add back only PATH/HOME and the ONE account-B credential.
#    --max-budget-usd is the hard per-run cap (§12.5).
env -i \
    PATH="$PATH" HOME="$HOME" \
    ANTHROPIC_API_KEY="$acct_b_key" \
    claude -p "$(cat "$brief_file")" \
      --max-budget-usd "$budget_usd" \
      --output-format json \
      --permission-mode acceptEdits \
      --add-dir "$worktree" \
    > "$worktree/.spill-run.json" 2>"$worktree/.spill-run.err"
unset acct_b_key                                          # scrub from the launcher's own env
```

Key isolation invariants (all enforced by the launcher, none relying on the model):

1. **`env -i` (start-from-empty) + explicit allowlist** guarantees account-A's
   `ANTHROPIC_API_KEY` (and any OAuth / `ANTHROPIC_AUTH_TOKEN`) is **physically absent**
   from the child — not merely overridden. This is the never-co-mingled property made
   mechanical, and it is the reason a subprocess (not an in-process agent) is required.
2. **The secret value is loaded at launch from a secret store and lives only in a shell
   variable for the duration of the `env` call**, then is `unset`. It is never written to
   a file in the repo, never echoed, never passed on a visible command line as a literal
   (it's an env var, which does not appear in `ps`/argv), and never committed.
3. **For a 3P-provider account-B**, swap the `ANTHROPIC_API_KEY=` line for the provider's
   own credential vars (`AWS_*` + `CLAUDE_CODE_USE_BEDROCK=1`, or the Vertex/Foundry
   equivalents) and **omit `ANTHROPIC_API_KEY` entirely** — the provider auth path is
   self-contained per the verified precedence rules.
4. **One child = one identity = one worktree** (§12.5). No child ever sees two accounts.

### 12.4 Detecting the approaching weekly limit (the spill trigger)

The detector `near_weekly_limit(account_A)` must be **conservative** (spill *before* the
wall, not after) and must **not** itself burn account-A quota probing. Inputs, in
priority order:

* **The orchestration tick's own weekly-limit warning** is the *primary* signal — this
  section is explicitly its standing answer. When the tick raises that warning, the
  detector returns true. (The tick already surfaces "approaching weekly limit"; the
  scheduler subscribes to it rather than re-deriving it.)
* **A rate-limit / usage-exhaustion observation from account-A's own work** is the
  *secondary* signal: when an account-A `claude` run returns a usage-limit / quota error
  (the plan-limit analogue of the API's 429 `rate_limit_error`), the detector flips to
  true for the rest of the window and the driver stops placing *new* account-A work
  (in-flight account-A pipelines finish; §12.7).
* **An explicit maintainer override** (`scripts/.scheduler/SPILL` sentinel, or a
  `needs:spill` bead label) forces spill on regardless of the detector — the manual
  counterpart to the §7.3 PAUSE/KILL controls.

Honest caveat: there is **no programmatic plan-usage meter** exposed to the session (the
usage-aware-shutdown memory note records exactly this — the plan meter is not pollable).
So the detector is **event-driven** (tick warning + observed exhaustion), **not** a
percentage gauge. The design does not pretend to read a remaining-quota number it cannot
read; it reacts to the warning the harness *does* emit and to observed limit errors. This
is the same "no daemon / event-driven" honesty as §1.3.

### 12.5 Per-run `--max-budget-usd` cost cap + cumulative ceiling

Two independent caps, both HARD, mirroring §4.3's EC2 ceiling discipline:

* **Per-run cap (verified flag).** Every `claude -p` spill child is launched with
  **`--max-budget-usd <amount>`** (the real, `--print`-gated flag verified in v2.1.177).
  This bounds the **dollar spend of a single run** — the model stops when the run's API
  spend would exceed the cap. Default conservative (e.g. a small per-bead cap; exact value
  is OQ-7), tunable per tier. A LIGHT docs bead gets a smaller cap than a medium feature
  bead. This is the blast-radius bound on any single mis-estimated spill.
* **Cumulative daily ceiling (driver-enforced).** The driver tracks total account-B spend
  this window (summing each child's reported spend from its `--output-format json`
  result) against a `SCHED_ACCT_B_USD_DAY` ceiling. When cumulative B spend would breach
  it, `cost_ceiling_hit(account_B)` returns true and the spill predicate (§12.2) goes
  false — no further B children launch this window (account-A in-flight work continues).
  This is the cost analogue of the §4.3 "the farm refuses a lease that would breach
  ≤ $5/day" rule: **a ceiling breach means "do not spill," never "spill anyway."**

Because `--max-budget-usd` is enforced by the child process itself, the per-run cap holds
even if the driver crashes mid-run (the child still stops at its cap) — defence in depth,
same shape as the §4.3 ≤ 45-min watchdog.

**Honesty boundary (no perf/cost numbers baked in).** This section deliberately states
**no concrete dollar figures** for the caps — they are configuration the maintainer sets
(OQ-7), and any number measured on the work box is NON-canonical (the EC2/work-box rule).
The doc gives the *mechanism* and the *ceiling discipline*, not a canonical price.

### 12.6 EXACTLY what the maintainer must provide to activate it (the credential-flip)

This is the `needs:user` step. Activation is **one secret + one env var name**, with
**zero code changes** — the spill path is built dormant and flips live when the
credential is present:

1. **Provision the second identity.** Either (a) a second Anthropic account / API key
   (a distinct billing identity — *not* a second key on the same account, which shares the
   quota and buys no headroom — §12.1), or (b) a 3P-provider account (Bedrock / Vertex /
   Foundry) with its own credentials.
2. **Store the secret in a SECRET STORE — never in the repo.** Put account-B's key (or the
   provider credential set) in the maintainer's chosen secret store: the OS keychain, a
   secrets manager, or — for CI — a **GitHub Actions secret**. The repo holds **only the
   secret's NAME**, never its value. Documented secret-NAME convention (the value is never
   committed and this doc, in a public repo, contains no key):
   * **Anthropic account-B:** secret name **`SPARQ_ACCT_B_ANTHROPIC_API_KEY`**, surfaced to
     the launcher via the indirection var **`SPARQ_ACCT_B_SECRET_NAME`** (so the launcher
     reads *which* secret to load, never an inline value).
   * **Bedrock account-B:** the standard AWS credential env set (`AWS_ACCESS_KEY_ID`,
     `AWS_SECRET_ACCESS_KEY`, `AWS_SESSION_TOKEN`, `AWS_REGION`) plus
     `CLAUDE_CODE_USE_BEDROCK=1`; **Vertex:** ADC + `CLAUDE_CODE_USE_VERTEX=1`. Same rule:
     names/flags live in config, values live in the store.
3. **Flip the enable flag.** Set **`SPARQ_ACCT_B_ENABLED=1`** (the `ACCOUNT_B_ENABLED`
   predicate input of §12.2). With it unset / `0`, the spill predicate is always false and
   the scheduler behaves exactly as §§1–11 (single-account) — verified by a unit test that
   asserts no `claude -p` child is ever launched when the flag is off.
4. **(Optional) set the caps.** `SCHED_ACCT_B_USD_DAY` (cumulative ceiling, §12.5) and a
   per-tier `--max-budget-usd` default; both have conservative built-in defaults so the
   flip is safe even if the maintainer sets neither.

That is the entire activation: **provide the secret to the store, set
`SPARQ_ACCT_B_SECRET_NAME` (or the provider vars) + `SPARQ_ACCT_B_ENABLED=1`.** No code
edit, no rebuild — a clean credential-flip. The doc **never hardcodes or prints any key**
(this is a public repo); it documents only NAMES and the flip procedure.

### 12.7 git + worktree coordination — account-B agents must not collide with account-A

Account-B children are ordinary worktree agents from git's point of view; the collision
guard is the **same conflict-partition the §1.4 frontier already enforces**, extended so
that the *account dimension* never overlaps on a bead:

* **One bead → one worktree → one account.** A spill child gets its **own fresh git
  worktree** (the launcher's `--add-dir "$worktree"` + a `git worktree add` for a branch
  named, e.g., `acctB/<bead-id>`), exactly as an account-A agent does. The
  in-flight-reservation set of §1.4 (open PR or unpushed worktree branch) **already**
  subtracts that bead from the frontier, so account-A can never simultaneously pick a bead
  that a B child holds. **No bead is ever worked by both accounts at once** — the existing
  reservation is the guarantee; the account dimension is just metadata on the reservation.
* **The conflict-partition (§1.4) is applied across the COMBINED A+B in-flight set.** A B
  child counts against the ≤ 1-per-crate / server+site→1 partition exactly like an A
  agent. Two beads touching one crate's source never co-launch *regardless of which
  account* runs them — the partition is account-agnostic, so a B child on `sparq-core` and
  an A agent on `sparq-core` cannot both be live. This reuses the sq-6ip4
  primary-code-crate inference verbatim; nothing new is needed for safety.
* **B children obey the same merge gates (§6).** A B child opens a normal PR on its
  `acctB/<bead-id>` branch; that PR passes through **adversarial-verify**, the
  **`sparq-perf-reviewer` PreToolUse hook**, and **`ci-summary`** identically — the trust
  model does not change because the *author* was a different account. The decision log
  records the authoring account so a post-mortem can attribute work, but arming is
  governed by verify-clean + non-perf exactly as in §6.
* **Reconciliation + idempotency (§3.4) extends to B.** The single-flight lock,
  `resumeFromRunId` reconciliation, and the worktree-GC sweep treat `acctB/*` branches the
  same as account-A branches — a resume never re-dispatches a bead a B child already holds
  (its worktree branch has unpushed commits → reserved). Orphaned B children are reaped
  the same way orphaned EC2 leases are (PID + worktree-branch reconciliation at re-kick).
* **API-rate-limit fleet cap is PER-ACCOUNT.** The ~10-concurrent ceiling (§1.4) is a
  *per-account* throttle, so account-B has its **own** sustainable-fleet budget. Spilling
  to B therefore genuinely widens total throughput (A's fleet + B's fleet) rather than
  reshuffling a single shared budget — this is the actual scaling win, bounded by B's own
  plan limits (§12.1).

Eligibility (which beads spill): **LOCAL-tier (LIGHT/medium) beads** are the natural spill
candidates — a headless `claude -p` child runs on the same work box and is cheapest to
coordinate. HEAVY/EC2 beads stay on the §4 EC2 path (account identity is orthogonal to
*where* the compute runs; a B child could in principle drive an EC2 lease, but Phase B
scopes spill to LOCAL-tier to keep the first cut simple — OQ-8). Wall-clock-sensitive
**benchmarks never spill** (the NON-canonical / quiet-box rules of §5 are unchanged).

### 12.8 Honest risks + guards specific to multi-account spill

* **The limit does not disappear — it doubles the bucket.** Account-B has its *own* plan
  limit; spill adds a second metered bucket, it does not grant unlimited capacity. When
  *both* near their caps, the scheduler is genuinely out of headroom and falls back to the
  §3.2 frontier-drains / idle behaviour. *Stated plainly so no one over-claims.*
* **Credential mishandling is the top risk.** *Guards:* `env -i` scrubbed child env
  (§12.3); secret loaded from a store, never the repo; value never logged / echoed / argv'd;
  truly-unset (never empty) `ANTHROPIC_API_KEY`; one-identity-per-child; a CI check that
  greps the repo (and this doc) for anything key-shaped and fails the build (composes with
  the existing code-scanning-at-zero posture).
* **Cost runaway on B.** *Guards (defence in depth):* per-run `--max-budget-usd` enforced
  by the child itself; driver-enforced cumulative `SCHED_ACCT_B_USD_DAY` ceiling that flips
  the spill predicate off; both independent of the (fallible) tier estimate; the §7.3
  KILL switch reaps B children alongside A work.
* **Detector false-negative (spill too late).** If the detector misses the warning,
  account-A simply hits its wall and the §12.4 secondary signal (observed exhaustion
  error) flips spill on — late, but it recovers. False-*positive* (spill too early) just
  spends a little B budget on work A could have done; bounded by the B cost ceiling.
* **Wrong-account attribution in the audit trail.** *Guard:* the decision log (§7.1)
  records the authoring account on every launch / arm so spill is always attributable; the
  STATUS one-liner shows the live B fleet + spend.
* **Verify gate is still an LLM (§8).** Unchanged — B's PRs pass the *same* independent
  gates (adversarial-verify, roborev's non-Anthropic reviewer, `ci-summary`, the perf
  hook); the authoring account changes nothing about the trust anchors.

### 12.9 Phased plan (each phase = a future bead under the scheduler epic)

These extend the §10 epic; they depend on the single-account loop (§10 Phase 4) existing
first, but the **design** is complete now and activation is gated only on the
`needs:user` credential.

* **Phase B1 — spill launcher + credential-isolation harness.** Build
  `scripts/spill-launch.sh` (§12.3): `env -i` scrubbed single-identity child,
  secret-from-store loader, `claude -p --max-budget-usd … --output-format json` invocation,
  result / err capture. Unit-test the isolation invariants with a **fake / sentinel key**
  (no real credential): assert the child env contains exactly one identity, that
  account-A's key is absent, and that `SPARQ_ACCT_B_ENABLED` unset ⇒ no child is launched.
  Depends on: §10 Phase 1. Unblocks: B2.
* **Phase B2 — weekly-limit detector + spill predicate.** Implement
  `near_weekly_limit(account_A)` (§12.4: tick-warning subscription + observed-exhaustion
  secondary + manual `SPILL` sentinel) and the §12.2 spill predicate, gated by
  `ACCOUNT_B_ENABLED`. Wire into the `workload-triage` / `placeOnFarm` routing point as the
  third (ACCOUNT-B) tier. Depends on: B1, §10 Phase 4. Unblocks: B3.
* **Phase B3 — cost caps + cumulative ceiling.** Per-run `--max-budget-usd` defaults per
  tier; driver-side cumulative `SCHED_ACCT_B_USD_DAY` tracking from child JSON results;
  ceiling flips the spill predicate off. Depends on: B2. Unblocks: B4.
* **Phase B4 — worktree / PR coordination + reconciliation.** `acctB/<bead-id>` worktrees,
  combined-set conflict-partition (§12.7), B-author attribution in the decision log,
  resume / orphan reconciliation of `acctB/*`. Depends on: B2 (and §10 Phase 5 for the
  shared arm path). Unblocks: B5.
* **Phase B5 — credential-flip activation (`needs:user`).** The maintainer provides the
  second credential to the secret store and sets `SPARQ_ACCT_B_SECRET_NAME` (or the 3P
  provider vars) + `SPARQ_ACCT_B_ENABLED=1` (§12.6). No code change — turn-key flip;
  observe one live spill run end-to-end (verify-clean PR armed) before relying on it.
  Depends on: B1–B4 and the credential. Closes the multi-account extension.

Suggested labels: `scheduler`, `multi-account`, plus **`needs:user` on Phase B5** (the
credential is the gate).

### 12.10 Open questions for the maintainer (multi-account)

* **OQ-6 (blocking B5).** Which account-B identity — a **second Anthropic account / API
  key**, or a **3P provider** (Bedrock / Vertex / Foundry)? This sets which env block the
  launcher constructs (§12.3) and which secret name to provision (§12.6). (Reminder: a
  second key on the *same* account shares the quota and adds no headroom — §12.1.)
* **OQ-7.** The cost caps: the per-run `--max-budget-usd` default(s) per tier and the
  cumulative `SCHED_ACCT_B_USD_DAY` ceiling. The design states the mechanism; the dollar
  values are the maintainer's policy (no canonical number is baked into the doc).
* **OQ-8.** Spill eligibility: LOCAL-tier only for the first cut (§12.7), or should an
  account-B child also be allowed to drive an EC2 lease (account identity is orthogonal to
  compute location, but it widens the first implementation)?
* **OQ-9.** Detector aggressiveness: how early before the weekly wall should spill engage
  — strictly on the tick's warning, or also a maintainer-set earlier trigger? (No pollable
  quota meter exists — §12.4 — so this is about which *event* trips it, not a percentage.)
* **OQ-10.** Secret-store choice for the activation step: OS keychain, a secrets manager,
  or GitHub Actions secret (for a CI-driven scheduler)? This decides what `load_secret`
  binds to in `spill-launch.sh`.
