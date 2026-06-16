# Orchestration automation — codifying the lead orchestrator's recurring behaviours

> Status: research/design record for maintainer review. `[OPUS-4.8]`
> Scope: design-for-review only — **no automation is implemented here**. This doc
> inventories the lead orchestrator's repeated per-tick / per-event behaviours,
> classifies each as **mechanical** (safe to automate) or **judgment-requiring**
> (must stay with the orchestrator/human), maps each mechanical behaviour to the
> right mechanism (cron / Workflow / hook / monitor / script), and proposes a
> phased, low-risk stand-up plan with explicit tear-down.
>
> This is the *orchestration* analogue of the existing `research/maintenance-flow-on-automation-design.md`, which codified merge-time *content* maintenance (a new crate must ship a bench/SKILL.md, etc.). That doc handles "did this change ship its flow-on artifacts"; this doc handles "stop the orchestrator hand-executing the same operational loop every 20 minutes." The two are complementary and share the same load-bearing idiom: **CI cannot write to `bd`, so a workflow mints a GitHub issue and the orchestrator reconciles it into a bead** (already live in `flow-on.yml`).

## 0. TL;DR — the recommendation

The orchestrator today runs a hand-executed 20-minute cron tick (`/loop`) plus a
swarm of per-event micro-behaviours. Most of the *mechanical* parts of that tick
are being paid for in **orchestrator tokens and attention every fire**, when they
should be deterministic scripts fired by the right event. The recommendation, in
one sentence:

**Push the deterministic detection + bookkeeping down into scripts/hooks/monitors;
keep cron as a thin safety-net sweep; keep the irreducible *judgment* (review a
diff for fabrication, choose WHAT to refill, resolve a semantic conflict, sign off
a merge) with the orchestrator — and do NOT let any mechanism auto-merge an
unreviewed PR or auto-close a bead on a merge it didn't verify.**

The split:

| Plane | What goes here | Mechanism | Why |
|---|---|---|---|
| **Detect + bookkeep (deterministic)** | orphan-check, gate-re-trigger detection, bead-export, close-on-*verified*-merge, typos/privacy-marker reword candidates, "PR went green" / "agent finished" signals | **scripts** (the testable mechanics) fired by **hooks** (event reactions) and **monitors** (event streams) | deterministic, unit-testable, cheap, no model tokens per fire |
| **Decide + act (judgment)** | review a PR diff for fabricated numbers, decide it's safe to arm auto-merge, choose which beads to refill the fleet with, resolve a semantic merge conflict, write a design-for-review | **orchestrator** (woken by a monitor event), never an unattended mechanism | over-automating these is the dangerous failure mode |
| **Guarantee forward progress (floor)** | the periodic sweep that catches anything the event path missed | **cron** (the existing `/loop`), but **thinned** — it should usually find nothing to do | AGENTS.md already says the loop is "a SAFETY NET, not the cadence" |

Concretely we recommend, in priority order: (1) `scripts/orphan-check.sh` +
`scripts/bead-close-on-merge.sh` + `scripts/gate-retrigger.sh` as testable
mechanics; (2) a `SessionStart` hook running the orphan-check (one-shot, cheap)
and a `PostToolUse` hook that re-exports beads after a `bd close`; (3) a
session-armed **merge-watcher monitor** that wakes the orchestrator with
"merged: PR #N" so close-on-merge is event-driven not tick-driven; (4) keep the
cron tick as a thinned safety-net. We **do not** recommend turning the whole tick
into a Workflow script (§5): the tick's load-bearing steps are judgment, and a
Workflow that auto-merged or auto-refilled would be the highest-severity failure
mode in the whole system. We recommend a Workflow **only** for the one genuinely
deterministic fan-out/verify shape — and even there, gated behind orchestrator
sign-off on merge.

---

## 1. The behaviour inventory + mechanical-vs-judgment classification

This is the rigorous boundary. The test for "mechanical": *could a deterministic
script with no model in the loop do this correctly every time, such that a wrong
result is a bug to fix rather than a judgment call?* If a wrong result would be a
**misjudgement** (a fabricated number waved through, the wrong bead refilled, a
real conflict mis-resolved), it is **judgment** and must NOT be unattended.

### 1.1 The 20-minute cron tick (`/loop`) steps

| # | Behaviour | Mechanical or judgment? | Rationale |
|---|---|---|---|
| T1 | **Detect** an open PR has gone `ci-summary`-green + threads resolved | **Mechanical** | Pure GitHub-state query — `gh pr checks` + `gh pr view --json reviewDecision,statusCheckRollup` + review-thread resolution. Deterministic. |
| T1b | **Decide to merge** that PR + **arm auto-merge** | **JUDGMENT** | Merging is an irreversible action on `main`. Even with green CI, the orchestrator (or human) is the one accountable for "this diff is safe + reviewed for fabrication/contamination." Never unattended. |
| T2 | **Close the mapped bead when a PR actually merges** | **Mechanical — IFF merge is *verified*** | The map PR#→bead is deterministic; the danger is closing a bead for a PR that *didn't* merge (closed-unmerged, or a stale event). Safe to automate only keyed off a *verified* `merged_at != null` from the API, never off a parsed log line. |
| T3 | **EC2 orphan-check** (`describe-instances tag:purpose=sparq-bench` + `ps aux\|grep gather`; terminate orphans) | **Mechanical** | This is the textbook script: a fixed tag filter, a fixed exclusion list (never prod/dev), a fixed kill predicate. The *exclusion correctness* is the only risk and is a script-correctness bug (testable), not a judgment. **Out of scope per the brief (no EC2/aws) — designed but not built here.** |
| T4 | **Keep code-scanning = 0** (screen + resolve/dismiss CodeQL/Scorecard alerts) | **JUDGMENT** | *Detecting* a non-zero count is mechanical (`gh api .../code-scanning/alerts`); *resolving* (fix vs. dismiss-with-reason) is a security judgment. Automate the detection→wake; never the dismissal. |
| T5 | **Refill the fleet** to ~4-5 background subagents from `bd ready` (distinct surfaces; serialise site + sparq-server) | **JUDGMENT (the choice) over a mechanical substrate** | *Counting* live agents and *listing* `bd ready` on disjoint file-areas is mechanical; *choosing which* beads to dispatch, writing each brief, and respecting the "one branch per contended surface" rule is judgment. The substrate can be scripted (a "ready, disjoint, not-contended" candidate list); the dispatch decision stays with the orchestrator. |
| T6 | **Bead newly-discovered work** (`bd create` + `bd export`) | **Mixed** | `bd export` after any `bd` mutation is **mechanical** (a hook). Deciding *what* is a new bead is **judgment** (already delegated: sub-agents `bd create` their own discovered work per AGENTS.md). |
| T7 | **AGENTS.md 8-step maintenance sweep** (cross-poll sibling PSS charter; triage from-pss issues; recurring-tasks→beads) | **Mixed, mostly judgment** | *Fetching* the sibling charter + *listing* open issues/alerts is mechanical (a script can assemble the briefing). *Deciding* what's "genuinely portable," writing the adapted prose, triaging an unclear issue (post a clarifying comment, don't guess) is judgment. Automate the **briefing assembly**, not the decisions. |
| T8 | **Honesty enforcement** (no ZK/MPC overclaim; work-box numbers non-canonical; paper numbers traceable) | **Mostly mechanical detection, judgment on edge cases** | Already partly enforced as CI gates: `check-no-perf-numbers.py`, `check-privacy-claims.sh`. Those are the mechanical core. The residual — judging whether a *new* phrasing is an overclaim — is judgment, but the gate catches the common cases and the privacy-claims gate is already live. |

### 1.2 The per-event micro-behaviours (done by hand today)

| # | Behaviour | Mechanical or judgment? | Rationale |
|---|---|---|---|
| E1 | On each agent completion: **review the PR diff for fabrication/contamination** | **JUDGMENT (irreducible)** | This is the single most important human-judgment step in the loop. A fabricated benchmark number, a contaminated test (asserting a value it computed rather than an independent oracle), a hallucinated citation — these are exactly what a deterministic check *cannot* catch and what roborev (codex, non-Anthropic) + the orchestrator exist to catch. **Never automate.** |
| E2 | On each agent completion: **arm auto-merge** | **JUDGMENT** | Same as T1b — armed only after E1 passes. |
| E3 | **Close the mapped bead on merge** | **Mechanical (verified merge)** | Same as T2. |
| E4 | **Bead the agent's deferred items** | **Mechanical capture of a judgment** | The agent already reports its deferred items; turning a reported item into `bd create` is mechanical, but *which* items are real follow-ups is the agent's judgment (already delegated). |
| E5 | **Rebase conflicting PRs** (esp. the hot `sparq-server` `http.rs` auth path) | **JUDGMENT for semantic conflicts; mechanical for trivial ones** | A *textual* conflict in non-overlapping regions can be auto-rebased; a *semantic* conflict on the auth path (two PRs both editing the handler) MUST be resolved by an agent that understands both intents. Auto-rebasing a semantic conflict silently is a contamination risk. Detect-and-wake; resolve with a (merge-fixer) sub-agent, not a blind `git rebase`. |
| E6 | **Reword recurring `typos` false-positives** (the past-tense SQL-keyword forms and `invok`-/`AND`-derived words the spell-checker mis-flags) | **Mechanical (a known dictionary)** | These are a *finite, recurring* set. A script can detect a `typos` failure whose flagged word is on the known-good list and either auto-add the allow entry or surface a one-line reword suggestion. Low risk (cosmetic, CI-visible). |
| E7 | **Add `privacy-claims-allow:` markers** for negated/historical ZK mentions | **Mechanical detection, judgment on the marker** | `check-privacy-claims.sh` already exists. Detecting the failure is mechanical; deciding a flagged sentence is genuinely negated/historical (vs. a real overclaim) is the judgment the gate is *designed to force*. Surface the failure + the candidate marker; the orchestrator confirms. |
| E8 | **Re-trigger a stale `gate` aggregator** (empty commit / rerun) | **Mechanical** | "`ci-summary` is stuck pending past a settle window with all siblings terminal" is a deterministic condition; the remedy (`gh run rerun` / push an empty commit) is mechanical. The AGENTS.md merge-mechanics note ("green but BLOCKED self-resolves ~11min") means the script must *wait out* the known window before acting — encode that threshold. |
| E9 | **Rebase pre-path-filter PRs** | **Mechanical** | "This PR's base predates the path-filter change" is a deterministic comparison; the rebase is mechanical (no semantic conflict by construction). |
| E10 | **Bead bookkeeping**: close-on-merge, export the `.beads/issues.jsonl` mirror, dedupe | **Mechanical** | Export-after-mutation is a hook; dedupe is a script (`bd`-query-based). |

### 1.3 The honest boundary, restated

**Five behaviours must stay with the orchestrator/human and are dangerous to
automate:** E1 (review for fabrication), T1b/E2 (decide-to-merge + arm), T5 (the
*choice* of what to refill), E5 (resolve a *semantic* conflict), T4/E7 (the
*resolution* of a security alert / privacy-claim — as opposed to its detection).
Everything else in the inventory is mechanical detection or bookkeeping and is
safe to push into a script/hook/monitor.

The unifying pattern for the judgment items: **automate the DETECTION and the
WAKE, never the DECISION.** A monitor that emits "PR #N is green and needs your
merge decision" is safe and useful; a workflow that *merges* it is not.

---

## 2. Per-behaviour mechanism mapping + stand-up/tear-down lifecycle

The mechanism palette (capabilities confirmed against the harness): **CronCreate**
(session-only recurring prompt; dies with the session; 7-day auto-expire),
**Workflow** (deterministic background JS orchestration script — `agent()`,
`parallel()`, `pipeline()`, `phase()`; resumable; structured results),
**Hooks** (`.claude/settings.json` — `PreToolUse`/`PostToolUse`/`Stop`/`SessionStart`;
deterministic event reactions; **the only one that persists in the repo** and
takes effect on the *next* session), **Monitor** (a background script whose
stdout lines become chat events that wake the loop — the merge-watcher is one),
**Shell scripts** (`scripts/` — the testable mechanics every other mechanism calls).

A load-bearing constraint from the existing estate (`flow-on.yml`): **CI/GitHub
Actions cannot write to `bd`** (no credentials, and `.beads/` must never be
hand-edited in a branch). So any *server-side* detection (a GitHub workflow)
that wants to create work must **mint a GitHub issue** (labelled `flow-on`/`auto`,
idempotent) that the orchestrator reconciles into a bead — exactly the pattern
already shipped. Client-side mechanisms (hooks/monitors/scripts running in the
session) *can* run `bd` directly.

| Behaviour | Best mechanism | Who fires it / when | Stand-up | Tear-down |
|---|---|---|---|---|
| **T3/E EC2 orphan-check** (designed only — no EC2 here) | `scripts/orphan-check.sh` called by a **`SessionStart` hook** (+ optionally a low-frequency cron) | session start (one-shot); the brief says a fresh session must orphan-check first | add the hook to `.claude/settings.json` (committed; effective next session) | remove the hook entry; the script is inert if `aws` isn't configured (graceful no-op like `bd-session-context.sh`) |
| **T2/E3 close-bead-on-verified-merge** | `scripts/bead-close-on-merge.sh` called from the **merge-watcher monitor's** event handler (orchestrator runs it on the "merged: PR #N" event) | when the monitor emits a *verified* merge | arm the merge-watcher monitor once per session (see below); ship the script | `TaskStop` the monitor; the script stays as a manual tool |
| **T6/E10 bead-export after mutation** | **`PostToolUse` hook** matching `bd close`/`bd create`, running `bd export` to the resync branch lane | after any `bd` mutation in the session | add the `PostToolUse` hook (committed) | remove the hook; export remains a manual `bd export` |
| **E8 gate-retrigger** | `scripts/gate-retrigger.sh` (waits out the known ~11-min self-resolve window, then reruns) called by the orchestrator on a monitor "stuck-gate" event | monitor detects `ci-summary` pending past threshold with all siblings terminal | ship script + add the condition to the PR-state monitor | remove the monitor condition |
| **E6 typos reword / E7 privacy-marker** | `scripts/ci-reword-suggest.sh` (read-only: prints the candidate reword/marker for a known false-positive) surfaced on a monitor "red lint" event | monitor sees the `typos`/privacy check fail; orchestrator confirms | ship the script with the known-good dictionary | remove from monitor; script stays manual |
| **T1 / E "PR went green" detection** | **Monitor** (per-PR `gh pr checks` poll that exits on terminal, *or* one session-armed PR-state monitor) | armed when a PR is opened; emits on green/red | arm per-PR `Bash run_in_background` watch (one notification) or a session monitor (many) | the per-PR watch self-exits; the session monitor `TaskStop`s at session end |
| **"agent finished" detection** | harness completion notification (already native) + the AGENTS.md reconcile rule for unnotified agents | on agent completion | none (native) | none |
| **T5 refill substrate** (candidate list only) | `scripts/refill-candidates.sh` → prints `bd ready` beads on disjoint file-areas, excluding contended surfaces already in flight | orchestrator runs it when deciding to refill | ship script | manual tool; nothing to tear down |
| **T7 AGENTS.md sweep briefing** (assembly only) | `scripts/charter-sweep-briefing.sh` → fetches sibling charter + open issues/alerts into one read-only briefing | orchestrator runs it at sweep time (or a low-freq cron emits it) | ship script | manual tool |
| **T1b/E2 merge decision, E1 review, E5 semantic conflict, T4/E7 resolution** | **orchestrator** (woken by the relevant monitor/notification) | on the wake event | n/a — these are the human-judgment core | n/a |
| **The safety-net sweep** | **CronCreate** (`/loop`), **thinned** | every 20 min, session-only | already running | `/loop` stops; dies with the session; 7-day auto-expire is a natural backstop |

### 2.1 Lifecycle principles

- **Hooks are the only repo-persistent mechanism** and take effect on the *next*
  session — so they suit *always-on, deterministic, low-risk* reactions
  (bead-export-after-mutation, session-start orphan-check). They are also the
  hardest to "tear down in a hurry" (you must edit + commit `.claude/settings.json`
  and it lands next session), so a hook must be conservative and idempotent, with a
  graceful no-op path (the pattern `bd-session-context.sh` already follows).
- **Monitors are session-scoped** and the right home for *event streams that wake
  judgment* (merge-watcher, PR-state, stuck-gate, red-lint). Arm once per session;
  `TaskStop` to disarm immediately if one misbehaves (e.g. emits too much — the
  harness auto-stops noisy monitors anyway). They die with the session, which is the
  desired "tear-down for free."
- **Scripts are the testable substrate** every other mechanism calls. They have
  no lifecycle of their own (a tool sitting in `scripts/`), are unit-testable
  (`scripts/tests/`), and degrade to a manual command if their caller is removed.
- **Cron (`/loop`) is the floor, not the clock** (AGENTS.md already states this).
  Its job after this design lands is to *catch what the event path missed*, and it
  should usually find nothing.

---

## 3. The composed system

```text
                       ┌───────────────────────── a session is running ─────────────────────────┐
                       │                                                                          │
  SessionStart hook ───▶ bd-session-context.sh (live) + orphan-check.sh (NEW)                     │
                       │                                                                          │
  Monitors (armed once)│  merge-watcher ──"merged: PR #N (verified)"──▶ bead-close-on-merge.sh    │
                       │  pr-state ───────"PR #N green / red / stuck"──▶ wake orchestrator        │
                       │  red-lint ───────"typos/privacy failed"───────▶ ci-reword-suggest.sh     │
                       │                                                                          │
  PostToolUse hook ────▶ after `bd close`/`bd create`: bd export → chore-beads-resync lane        │
                       │                                                                          │
  Orchestrator (judgment, woken by the above):                                                    │
        E1 review diff ─▶ E2 arm auto-merge ─▶ (merge happens) ─▶ merge-watcher fires ─▶ T2 close  │
        T5 refill (uses refill-candidates.sh substrate) ─▶ dispatch background sub-agents          │
        E5 semantic conflict ─▶ dispatch merge-fixer sub-agent                                     │
        T4/E7 alert/privacy resolution                                                             │
                       │                                                                          │
  CronCreate /loop (thinned safety-net): every 20 min, run the AGENTS.md sweep; usually a no-op    │
                       │   (assembles its briefing via charter-sweep-briefing.sh)                  │
                       └──────────────────────────────────────────────────────────────────────────┘

  Server-side (GitHub Actions, cannot touch bd): flow-on.yml (live) mints `flow-on` issues
  on merge → orchestrator reconciles into beads. Same pattern available for any future
  server-side detector (e.g. a "code-scanning alert opened" issue).
```

The event-driven path does the work the instant something is ready (per AGENTS.md
"primary mode is event-driven and eager"); the cron sweep is the floor.

---

## 4. Verdict on Workflow-vs-cron for the tick (the concrete split)

**Verdict: do NOT convert the recurring tick into a single deterministic Workflow.
Keep cron as the thinned safety-net, push detection/bookkeeping into
scripts+hooks+monitors, and reserve the Workflow tool for ONE narrow shape.**

### 4.1 Why not a Workflow-driven tick

The Workflow tool's strength is *deterministic fan-out/verify/synthesize that
needs no turn-by-turn human judgment*. The orchestration tick is the opposite: its
load-bearing steps (§1.3) are exactly the judgment ones — review-for-fabrication,
decide-to-merge, choose-what-to-refill, resolve-semantic-conflict. A Workflow that
ran those unattended would be the **single highest-severity failure mode** in the
entire system (§6): a runaway loop that merges unreviewed PRs and auto-closes
beads. The "ultracode" default of authoring+running a Workflow for every
substantial task is the wrong model *for the tick* — the tick is not a task, it's
a supervision loop. (Workflows remain the right tool for substantial *deterministic
sub-tasks* the orchestrator delegates, just not for the supervisory tick itself.)

The token-cost argument for a Workflow (less per-tick model cost) is real but is
better captured a different way: push the *deterministic* parts out of the
model's hands entirely (into scripts/hooks/monitors that cost zero model tokens),
so the cron tick that remains is already thin. A Workflow would still spend model
tokens on every `agent()` call inside it; scripts spend none.

### 4.2 The concrete split

| Tick concern | → goes to | Notes |
|---|---|---|
| Detect green PRs / red main / stuck gate | **monitors + scripts** | zero model tokens; wakes orchestrator on the event |
| Close bead on *verified* merge | **script** (merge-watcher event) | verified-merge guard mandatory |
| Bead export after mutation | **PostToolUse hook** | always-on, repo-persistent |
| Orphan-check | **SessionStart hook + script** | one-shot per session (design only — no EC2 here) |
| Charter-sweep briefing assembly | **script** (optionally a low-freq cron emit) | read-only |
| Refill *candidate list* | **script** | substrate only |
| **Refill DECISION + dispatch** | **orchestrator** | judgment |
| **Review diff, decide merge, arm auto-merge** | **orchestrator** | judgment — never a Workflow |
| **Resolve semantic conflict, resolve security/privacy alert** | **orchestrator (+ merge-fixer sub-agent)** | judgment |
| The catch-all sweep that guarantees progress | **cron `/loop`, thinned** | the floor |
| A genuinely deterministic *delegated* batch (the one Workflow candidate) | **Workflow** | see §4.3 |

### 4.3 The one Workflow candidate

There is exactly one shape in the orchestrator's repertoire that fits the
Workflow tool well: **a deterministic refill→verify fan-out** where the *set* of
beads is already chosen by the orchestrator and each is independent (disjoint
file-areas). A Workflow could `parallel([...])` the sub-agents, collect each one's
in-worktree gate result, and return a structured "ready-for-review" list — *without
merging*. This buys determinism + resumability for the fan-out mechanics while
keeping every merge decision (E1/E2) with the orchestrator: the Workflow's output
is a review queue, not a merge. This is **opt-in and bounded** (a fixed bead list
in, a structured result out — no open-ended loop), which sidesteps the runaway-loop
failure mode. Recommend prototyping this *after* the script/hook/monitor layer is
proven, not before (Phase 4 below), and only if the manual fan-out proves to be
the bottleneck.

---

## 5. Phased stand-up plan (each phase a future bead)

Ordered lowest-risk / highest-value first. Each item below becomes a bead under a
new epic (`bd create`; the orchestrator files them — sub-agents/this doc do not
hand-edit `.beads/`). Every phase includes its **disable/rollback** so a
misbehaving automation is torn down in one step.

1. **Bead A — `scripts/bead-close-on-merge.sh` + tests (the verified-merge guard).**
   A script that, given a PR number, closes the mapped bead **only if**
   `gh pr view --json mergedAt` is non-null (verified merge), and is idempotent
   (no-op if the bead is already closed). Ship with `scripts/tests/`. *Highest
   value, lowest risk:* it is the bookkeeping that's most error-prone by hand and
   the guard makes the dangerous case (close-on-non-merge) impossible. **Disable:**
   stop calling it; it's a manual tool with no side-effects until invoked.

2. **Bead B — `PostToolUse` hook: `bd export` after a `bd` mutation.**
   Add to `.claude/settings.json` a `PostToolUse` hook matching `bd close`/`bd create`
   that runs `bd export` onto the `chore-beads-resync-*` lane (never into a feature
   branch — per AGENTS.md). Graceful no-op when `bd` absent. *Low risk* (export is
   idempotent, regenerates the JSONL). **Disable:** remove the hook entry + commit;
   effective next session — so keep it conservative.

3. **Bead C — `scripts/orphan-check.sh` + `SessionStart` hook (design-built, EC2
   wiring deferred per the no-EC2 scope).** The script encodes the fixed tag filter
   (`purpose=sparq-bench`), the **prod/dev exclusion list as a hard allow-not-touch
   set**, the `ps`-based gather predicate, and a `--dry-run` default (prints what it
   *would* terminate). The `SessionStart` hook runs it `--dry-run` first; actually
   terminating requires `--run`. *Risk is the exclusion correctness* — mitigated by
   dry-run default + a test asserting prod/dev IDs are never in the kill set.
   **Disable:** remove the hook; the script is a manual `--dry-run` tool.

4. **Bead D — merge-watcher monitor (session-armed) wired to Bead A.**
   A `Monitor` that polls `gh` for newly-merged PRs and emits one `merged: PR #N`
   event per *verified* merge; the orchestrator's handler runs `bead-close-on-merge.sh`.
   Makes T2/E3 event-driven instead of tick-driven. **Disable:** `TaskStop` the
   monitor (instant); dies with the session anyway.

5. **Bead E — `scripts/gate-retrigger.sh` + `scripts/ci-reword-suggest.sh` (CI
   firefighting mechanics).** `gate-retrigger.sh` waits out the known
   ~self-resolve window (encode the threshold from AGENTS.md merge-mechanics)
   before rerunning; `ci-reword-suggest.sh` is read-only and prints the candidate
   reword/privacy-marker for a *known* false-positive (the DELETEd/DROPped/… set).
   Both surfaced via a session PR-state/red-lint monitor; the orchestrator confirms
   before acting. **Disable:** remove the monitor condition; scripts stay manual.

6. **Bead F — `scripts/refill-candidates.sh` (refill substrate, NOT the decision).**
   Prints `bd ready` beads on disjoint file-areas, excluding contended surfaces
   (`sparq-server`, site) already in flight (derived from `git worktree list` +
   open PRs). The orchestrator reads it and *decides*. **Disable:** stop calling
   it; manual tool.

7. **Bead G — `scripts/charter-sweep-briefing.sh` (AGENTS.md-sweep briefing
   assembly, NOT the decisions).** Assembles the sibling-charter fetch + open
   issues + code-scanning alert count into one read-only briefing for the
   orchestrator's sweep. **Disable:** manual tool.

8. **Bead H — thin the cron `/loop` prompt** to lean on the above (it becomes a
   safety-net that *checks* the event path ran, rather than re-doing it), and
   document the composed system in AGENTS.md (a short "Orchestration automation"
   subsection pointing at the scripts/hooks/monitors). **Disable:** revert the
   `/loop` prompt.

9. **Bead I (optional, last) — prototype the bounded refill→verify Workflow**
   (§4.3) *only if* manual fan-out is the proven bottleneck; bounded input (a
   fixed bead list), structured review-queue output, **no auto-merge**. **Disable:**
   don't run it; the manual fan-out path remains.

10. **Bead J (optional upstream) — `flow-on.yml`-style server-side detector for
    "code-scanning alert opened"** that mints a `flow-on` issue the orchestrator
    reconciles (T4 detection only — never auto-dismiss). Reuses the live pattern.

Dependencies: Bead A blocks Bead D (the monitor calls the script); Bead H depends
on A–G existing (so it can reference them); Beads B, C, E, F, G are independent of
each other; Bead I depends on the script/monitor layer being proven; Bead J is
independent and optional.

---

## 6. Honesty / safety — failure modes + guardrails

The whole point of the mechanical-vs-judgment boundary (§1.3) is that the
*dangerous* failure modes all live in over-automating judgment. Enumerated, with
the guardrail that prevents each:

| Failure mode | Severity | Guardrail |
|---|---|---|
| **A hook/Workflow auto-merges an UNREVIEWED PR** | **Critical** | Merge is *always* an orchestrator decision (E2). No mechanism in this design merges. The merge-watcher *observes* merges; it never causes them. A Workflow (§4.3) returns a review queue, never a merge. Branch protection (`ci-summary` + conversation-resolution + Copilot/CodeQL review) is the server-side backstop even if a client mechanism misbehaved. |
| **Auto-close a bead for a PR that DIDN'T merge** (closed-unmerged, or a stale/spoofed event) | High | `bead-close-on-merge.sh` closes **only** when `gh pr view --json mergedAt` is non-null (verified against the API, not a parsed monitor line). Idempotent. A unit test asserts a closed-unmerged PR closes no bead. The monitor line is a *wake*, not the source of truth. |
| **Orphan-check terminates a PROD/DEV box** (exclusion list wrong) | **Critical** (irreversible) | `--dry-run` default; explicit prod/dev **allow-not-touch** ID set; a test asserting those IDs are never in the kill set; terminate only matches the *exact* `purpose=sparq-bench` tag (allow-list semantics, not deny-list). Out of scope to *build* here (no EC2) — designed with the guardrail baked in. |
| **A Workflow burns tokens in a runaway loop** | High | No unbounded Workflow in this design. The one Workflow candidate (§4.3) has bounded input (a fixed bead list) and structured output — no open-ended loop. The cron tick stays the thinned safety-net, not a Workflow. |
| **`bd export` hook corrupts/conflicts `.beads/`** | Medium | Export goes to a dedicated `chore-beads-resync-*` lane, **never a feature branch** (AGENTS.md). `bd export` regenerates the JSONL deterministically (idempotent). Hook no-ops if `bd` absent. |
| **Auto-reword changes meaning / auto-rebase corrupts a semantic conflict** | Medium–High | `ci-reword-suggest.sh` is **read-only** (prints a candidate; the orchestrator applies). Auto-rebase is restricted to *textual* non-overlapping conflicts and *pre-path-filter* rebases (no semantic merge); a semantic conflict (the `http.rs` auth path) is detected-and-escalated to a merge-fixer sub-agent, never blind-rebased. |
| **A monitor floods the chat / hides a failure** | Low–Medium | Monitor filters cover *both* success and failure signals (the Monitor tool auto-stops noisy monitors); each monitor is `TaskStop`-able instantly; verified-state checks (not log-line parsing) drive the consequential actions. |
| **A hook misbehaves and can't be turned off mid-session** | Medium | Hooks are repo-persistent and effective next session — so they're limited to *conservative, idempotent, no-op-on-absence* reactions (export, dry-run orphan-check). Anything risky stays a monitor/script (instantly disableable), not a hook. |
| **Gate-retrigger fires during the normal self-resolve window** (unnecessary churn) | Low | `gate-retrigger.sh` waits out the documented self-resolve window before acting (AGENTS.md: "green but BLOCKED self-resolves ~11min; don't over-forensic"). |
| **Honesty erosion: a fabricated number waved through** | **Critical** | E1 (review-for-fabrication) is explicitly *never* automated — it stays with the orchestrator + roborev (codex, non-Anthropic, cross-family). The mechanical honesty gates (`check-no-perf-numbers.py`, `check-privacy-claims.sh`) catch the common cases; the privacy-claims gate is live and kept caveated. |

The meta-guardrail: **every mechanism in the design either (a) is read-only /
detection-only, (b) performs an idempotent bookkeeping action behind a verified
precondition, or (c) wakes the orchestrator for a judgment.** None performs an
irreversible or judgment-laden action unattended.

---

## 7. Open questions for the maintainer

1. **Hook persistence vs. churn.** A `PostToolUse` `bd export` hook fires on every
   `bd` mutation in *every* session. Is that desirable, or would you prefer the
   export stay an explicit orchestrator step (the status quo) to keep the
   `chore-beads-resync-*` lane under deliberate control? (The design proposes the
   hook; it's the most "always-on" of the recommendations and the hardest to
   disable mid-session.)
2. **Cron lifetime.** CronCreate dies with the session + 7-day auto-expire. Is the
   thinned `/loop` the intended permanent home for the safety-net sweep, or do you
   want the sweep to survive session restarts via some other mechanism (the only
   repo-persistent one is a hook, which is the wrong shape for a periodic sweep)?
   If session-survival matters, that's a genuine gap in the palette worth naming.
3. **Workflow appetite (§4.3).** Do you want the bounded refill→verify Workflow
   prototyped at all, or is the manual fan-out (with the candidate-list script as
   substrate) sufficient? The Workflow buys determinism/resumability at the cost of
   a new authored artifact to maintain.
4. **Server-side `bd` writes.** The whole design routes server-side detection
   through `flow-on`-style GitHub issues because CI can't touch `bd`. If you'd
   accept a narrowly-scoped CI token + a guarded `bd` write path, several
   reconciliation hops collapse — but that's a credential/security decision
   (a `needs:user` item), not something to bake in.
5. **Role-specific sub-agents.** The brief references 6 role agents
   (sparq-rust-feature/site/ci-infra/merge-fixer/researcher/docs, PR #369), but
   `main` currently carries only the 3 compliance agents in `.claude/agents/`.
   This design assumes a `merge-fixer` agent exists for E5; if that PR hasn't
   landed, Bead E's escalation target needs confirming.

---

## 8. The push-scheduler — the refill DECISION's mechanical substrate `[OPUS-4.8]`

§1.1 T5 split *refill* into a **judgment** (which beads to dispatch, the brief, the
one-branch-per-contended-surface rule) over a **mechanical substrate**. §5 Bead F
shipped the first half of that substrate: `scripts/refill-candidates.sh`, which
*groups* `bd ready` by inferred surface and *flags* contention — advisory only, no
subtraction, no cap. That leaves the orchestrator to do the arithmetic by hand on
every refill: subtract what's already in flight, pick at most one bead per
collision surface, and stop at the build-farm's core budget. That arithmetic is
**deterministic** and therefore belongs in a script. `scripts/push-frontier.sh`
(bead `sq-o09o`) is that script: it turns the advisory candidate list into the
**launchable frontier** — the exact set the orchestrator can hand straight to the
one bounded Workflow (§4.3 / Bead I) without creating a conflict or over-subscribing
the cores. It is still read-only and dispatch-free; the *choice to dispatch* and the
*brief* stay with the orchestrator (T5 remains judgment). The script just makes the
input to that judgment a single computed line-list instead of a manual derivation.

### 8.1 The model

```text
   trigger:  the merge-watcher monitor (§2 Bead D) — "merged: PR #N (verified)"
             ─▶ a slot freed ─▶ recompute the frontier ─▶ surface it to the orchestrator
                                                          (the refill DECISION, T5)

   frontier = bd ready                       (real edges — see §8.3; the unlock-frontier
                                              is only true if the edges are accurate)
            − epics                          (an epic is an umbrella, not a work unit —
                                              you dispatch its child tasks, never "the epic")
            − in-flight                       (a bead whose id backs an OPEN PR; see §8.2)
            − conflict-collisions             (≤1 bead per conflict surface; the site and the
                                              sparq-server http.rs auth path serialise to ≤1)
            ⌈capped⌉ at the CPU ceiling        (min(16, nproc − 2) — the binding constraint, §8.4)
```

The merge-watcher is the *trigger*, not a poll: the frontier only changes when a PR
merges (a surface frees) or a bead lands (`bd ready` grows). Recomputing on that
event — rather than every 20-minute tick — is the same "push detection down,
event-drive the wake" idiom as the rest of the design (§0). This is why the bead is
"push-start unblocked beads on **every merge**": the merge is the edge that unblocks
the next layer of `bd ready`, and the scheduler exists to make that unblock visible
the instant it happens.

### 8.2 Conflict-partition + the honest in-flight signal

The collision partition assigns every ready bead a **conflict surface** (the unit
that must serialise to ≤1 concurrent worktree):

- each `sparq-<crate>` short-name under `crates/` (one launch per crate at a time);
- **`site`** — the Next.js `site/` tree (`site:`/`page:`/`/surface/` beads): ≤1, because
  parallel agents on the static export collide on the same `site/src` files and the
  single export gate;
- **`server-auth`** — the `sparq-server` `http.rs` auth path: ≤1. This is the hot
  rebase seam called out across the agent briefs (two PRs both editing the auth
  handler is the canonical *semantic* conflict, E5). It is a strict sub-lane of the
  `sparq-server` crate cap, named explicitly so the scheduler never proposes a
  second auth-path bead even when it would otherwise pick a different `sparq-server`
  task.

The **in-flight** subtraction has one honest subtlety, load-bearing enough to encode
in the script and restate here: **`git branch -r` is not a usable in-flight signal in
this repo.** Because PRs are *squash-merged*, a merged feature branch never becomes
an ancestor of `main`, so `git branch -r --no-merged origin/main` reports ~all 370+
historical branches as "unmerged" — using it would subtract essentially everything
and the frontier would always be empty. The authoritative in-flight signal is
therefore the set of **open PRs** (`gh pr list`): a bead is in flight iff its id
appears in an open PR's head-branch name or title. A remote branch only counts when
it also backs an open PR (so it collapses to the PR signal); stale merged branches
are correctly ignored. A future reader must not "fix" this by trusting `--no-merged`.

### 8.3 The frontier is only as good as the edges (deliverable 2)

`bd ready` is the true unlock-frontier **only if** the dependency edges are real. In
practice many edges lived only as prose in titles/descriptions — `… (dep X)`,
`after Y`, `requires Z`, `blocked by …` — which `bd ready` cannot see, so a bead
would show "ready" while a human reading the text knew it was blocked. Closing that
gap is a prerequisite for the scheduler: a wrong frontier dispatches work that will
immediately stall on an unbuilt prerequisite. The conservative rule applied: **add an
edge only where the text justifies it** (an explicit `dep`/`after`/`requires`/`blocked
by`/`builds on`/`deferred remainder of`, or an epic→phase membership), and only where
the named blocker is itself still open/in-progress/deferred (a closed blocker is
already satisfied and needs no edge); never invent an edge from adjacency or topical
similarity. "Relates"/"complements"/"distinct from"/"related to" are explicitly **not**
blocking edges and were left as-is. The fedclient (`sq-dnko`) phase chain was already
correctly wired and was not touched.

One structural limit surfaced: `bd` rejects a `blocks` edge between a task and an
epic when *adding* it (`tasks can only block other tasks, not epics`), even though
historical epic→task edges exist in the DB. A genuine "task gated until an epic
lands" dependency (e.g. the privacy-fed-join task gated on the ZK-remediation epic)
therefore cannot be expressed as a fresh `blocks` edge and needs epic *membership*
(parent/child) or a dependency on the epic's terminal phase bead — a modeling
decision left for the maintainer rather than forced.

### 8.4 Identified blockers / binding constraints

- **CPU ceiling = the binding constraint.** The launchable count is capped at
  `min(16, nproc − 2)` because the work is `cargo` builds and the box cannot usefully
  run more parallel compiles than it has cores (minus headroom for the orchestrator +
  OS). On the current work box (`nproc` = 8) the cap is **6** — and the ready frontier
  routinely *exceeds* it, so the cap, not the dependency graph, is what throttles
  throughput today. Everything else (edge accuracy, conflict-partition) only matters
  up to this number of slots.
- **The EC2 build-farm is the scale lever.** The one way to lift the binding
  constraint is more cores: an orphan-proof EC2 build-farm (the same self-terminating,
  `purpose=sparq-bench`-tagged pattern the bench instances already use) would raise the
  per-tick launchable count from ~6 toward the 16 hard ceiling. That ceiling is itself
  deliberate — beyond ~16 parallel feature branches the *merge* side (conflict
  resolution on shared seams like `http.rs`, the serialised `ci-summary` gate) becomes
  the bottleneck, so unbounded fan-out buys nothing. Standing up the farm is an
  EC2/cost decision (out of scope here, per the no-EC2 constraint), captured as the
  scale lever rather than built.
- **The merge gate stays discretionary for performance.** The scheduler deliberately
  does **not** auto-merge and does **not** front-run the gate: it only proposes
  launches. The merge decision (E1 review-for-fabrication, E2 arm-auto-merge) remains
  the orchestrator's judgment (§1.3), and the `ci-summary` aggregator + branch
  protection remain the server-side floor. Pushing more starts does not entitle the
  system to push more merges; the merge side is rate-limited by review + the single
  required gate by design, and trying to automate past that is the §6 critical
  failure mode.

### 8.5 Where it sits in the lifecycle

`push-frontier.sh` is a **script** (§2.1) — the testable substrate, no lifecycle of
its own, degrades to a manual command. It composes with the existing pieces rather
than replacing them: `refill-candidates.sh` answers "*what's ready and where's the
contention?*" (the briefing); `push-frontier.sh` answers "*given that, what is safe
to launch right now?*" (the computed frontier); the orchestrator answers "*and which
of those do I actually dispatch, with what brief?*" (the irreducible judgment, T5).
The merge-watcher (Bead D) is its trigger; the bounded refill→verify Workflow (Bead I,
§4.3) is its natural consumer — the frontier is exactly the *fixed bead list in* that
the Workflow needs to stay bounded and non-runaway. It performs no mutation and no
dispatch, so it sits squarely inside the meta-guardrail (§6): read-only / detection-
only, waking a judgment, never taking an irreversible action unattended.
