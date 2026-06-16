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

where *conflict-collision* = a bead whose file-area overlaps an in-flight bead
(`sparq-server`/`http.rs` auth path is the one contended surface — reserve for ONE
branch at a time per `AGENTS.md`). This frontier is exactly what `push-frontier.sh`
must print (§4). The two independent ceilings from `AGENTS.md` both apply and are
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
const TOKEN_BUDGET = env.SCHED_TOKEN_BUDGET;     // hard stop (see §7 runaway guard)
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
2. **Token budget hit** — `spent ≥ SCHED_TOKEN_BUDGET`. Hard stop (runaway guard §7).
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

## 7. Honest risks + guards

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

## 8. Options considered (trade-offs)

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

## 9. Phased implementation plan (each phase = a future bead under a scheduler epic)

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

## 10. Open questions for the maintainer

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
