# Agent-effectiveness program — measure first, then adopt by ROI [OPUS-4.8]

> 🤖 SPARQ agent — design-for-maintainer-review. No implementation here; this is the
> measurement backbone + a ranked, gated adoption plan. Each phase below is a future bead.

**Status: design-only / proposed. Nothing in this doc is adopted yet.**

This record extends — does **not** duplicate — the cost-side survey in
[`agent-efficiency-tooling.md`](./agent-efficiency-tooling.md) (epic **sq-lhwo**, #430).
That doc establishes the cost model and ranks levers; this doc adds a concrete, phased
adoption plan for the *effectiveness* surface: AST/symbolic navigation, repeated-task
workflows/skills, and prose-only vector lookup — every item **gated** on a measured win.

**The measurement protocol is NOT re-invented here.** The falsifiable A/B accounting +
kill-criteria + verdict-object spec is **single-sourced** in
[`dogfooding-sparq-knowledge-graph.md`](./dogfooding-sparq-knowledge-graph.md) **§5**
(the sparq-PKG track). This program **adopts that spec verbatim** as the shared experiment
contract — the same thresholds, the same statistics, the same verdict object — so the two
tracks cannot drift into overlapping-but-incompatible protocols. §1 below states only the
*generalisations* this program adds on top of §5 (the full data-source join, the composite
first-shot signal, the anti-gaming rules); §5 of this doc then points back to the shared
spec and the division of labour. See §5 for what is shared vs. what is track-specific.

The single governing rule (shared with both docs): **no tool, skill, workflow, or feature
change is adopted on a vendor number or a hunch — it is adopted only after a
baseline-then-change measurement on THIS repo passes the pre-registered bar in
`dogfooding-sparq-knowledge-graph.md` §5.** The measurement backbone is therefore Section 1,
and it leads everything else.

---

## 1. The measurement backbone (instrument-FIRST)

The honesty problem this program exists to solve: every percentage cited for agent
tooling — including the "−16.58% output tokens / −28.64% runtime" AGENTS.md study
(Lulla et al., arXiv 2601.20404) and every "10×/49×/95–99% token reduction" memory/index
claim — is *someone else's measurement on someone else's repo*. None translates without
local measurement (`agent-efficiency-tooling.md` §7–8). The instrument below converts
"asserted win" into "measured win on our task mix or it does not ship."

### 1.1 Build on what already exists — do NOT rebuild token accounting

The Phase-1 instrument is already shipped (bead **sq-dhss**, #767):
[`scripts/agent-telemetry/agent_telemetry.py`](../scripts/agent-telemetry/agent_telemetry.py)
parses a Claude Code transcript JSONL and emits the efficiency primitives:

- top-level `agent_count`, `subagent_count`, `wave_duration_seconds`, `session_ids`;
- `rollup{input_tokens, output_tokens, cache_read_input_tokens,
  cache_creation_input_tokens (+5m/1h split), cache_hit_ratio, total_input_side_tokens}`;
- per-agent `agents[]{agent_id, attribution_agent, label, tokens, duration_seconds,
  tool_calls, assistant_turns, models, is_sidechain}`.

It is stdlib-only, tested against the synthetic fixture
`scripts/agent-telemetry/tests/fixture_transcript.jsonl`
(`test_agent_telemetry.py`), and tags every number `non_canonical: true`.
**The framework's job is to ADD a per-PR/task collection row + quality-pairing + the §5
A/B protocol on TOP of this — never to re-derive token counts.** It is also the canonical
accounting engine named by `dogfooding-sparq-knowledge-graph.md` §5: both tracks **diff two
of its JSON reports** rather than hand-counting tokens.

The other three data stores already exist and join on commit SHA / PR / bead / session:

| Signal | Source (already present) | Join key |
|---|---|---|
| token / cache / per-agent efficiency | `agent_telemetry.py` over the session transcript | session id |
| review **pushback** count + severity + $ + duration | `roborev summary/list/show/cost --json` (non-Anthropic `codex` reviewer; severity High/Med/Low) | git SHA |
| **CI first-pass** + rework + diff churn | `gh pr view N --json statusCheckRollup,commits,additions,deletions,changedFiles,reviews,reviewDecision`; `gh run list --json conclusion,attempt,workflowName` | PR ↔ SHA |
| task identity + dependency context | `bd show/list --json` | bead id |
| quality floors already gated in CI | conformance scoreboard, `scripts/perf-gate.py`, mutation ratchet `bench/mutants-baseline.json`, coverage floor | PR |

### 1.2 The metric+quality PAIRING rule (the anti-gaming core)

Every efficiency metric is **inadmissible alone**. The canonical attack is *"fewer
tokens, worse output."* Defeat it by binding each efficiency metric to a mandatory
**quality pair**, and adopting only when the efficiency metric wins **and** the quality
pair is **non-inferior**. The non-inferiority test, the significance bar, and the verdict
object are the shared spec in `dogfooding-sparq-knowledge-graph.md` §5.1–§5.6 — this table
only enumerates the *pairs* this program adds for the effectiveness surface:

| Efficiency metric (source) | Mandatory quality pair |
|---|---|
| input/output tokens per task (`telemetry.rollup`) | first-shot-success rate |
| first-shot-success (composite, §1.3) | post-merge revert / defect rate (backfilled) |
| roborev pushback count + severity (`roborev`) | seeded-bug **canary** find-rate (a quiet reviewer must still catch known bugs) |
| CI-gate-pass-first-try (`gh` attempt==1) | mutation-score Δ + coverage Δ (gates green because tests are *real*, not weakened) |
| wall-clock / $ (`telemetry` / `roborev cost`) | the same pairs above |

This mirrors the repo's own `perf-gate.py` philosophy (`AGENTS.md`): **deterministic
metrics are hard-gated; timing/noise metrics are advisory and never block.** Here:
token / wall-clock / $ are advisory-direction; the quality floors (mutation, coverage,
conformance, first-shot) are hard. An efficiency win that trips a quality floor is an
automatic reject regardless of magnitude. The token comparison itself is on
**cache-discounted effective input tokens** (`1.0*fresh + 0.1*cache_read + 1.25*cache_write`,
`dogfooding-…` §5.1) — never raw nominal input, which would invalidly ignore arm A's
~0.1× cached reads.

### 1.3 First-shot-success is a COMPOSITE, reported sub-flag-by-sub-flag

Two distinct senses were conflated in casual usage; both are required, and reporting the
sub-flags separately defeats gaming (you cannot claim first-shot by suppressing one
signal, and you can see *which* gate an agent/tool trips most):

```text
first_shot = first_ci_attempt_all_green   # gh: run attempt==1, all checks SUCCESS
          AND no_rework                    # see below — NOT just "no fix commit"
          AND roborev_blocking_findings==0 # roborev: no High/Med finding that forced a change
          AND review_changes_requested==0  # gh: no change-request review
```

`no_rework` is the primary quality guard the whole A/B leans on, so it must be **robust to
history rewriting**. Deriving it from commit *messages* ("no fix/address/rebase commit")
is gameable by `git commit --amend` + force-push (the rewrite leaves no extra commit). The
harness therefore derives `no_rework` from **push/ref events, not commit text**:

- `no_rework = (force_push_count == 0) AND (post-first-push commit count == 0)`, where
  `force_push_count` is read from the PR timeline / reflog
  (`gh api repos/:owner/:repo/pulls/N/timeline` → `forced_pushed`/`head_ref_force_pushed`
  events; or the local `git reflog` of the PR branch), **and**
- any post-first-push push that changes the diff (force-push OR new commit) counts as
  rework regardless of its commit message.

A force-push after the first push is itself the rework signal — counting it closes the
amend/squash hole that a message-grep leaves open.

Every sub-flag is derived **independently** from `gh` + `git` + `roborev` — **never
agent-self-reported.** The independent, adversarial `codex` reviewer is the
fabrication check the maintainer already relies on.

### 1.4 The collection harness (ONE new script)

Propose `scripts/agent-telemetry/metrics_row.py` (sibling to `agent_telemetry.py`,
stdlib-only, with a synthetic-fixture unittest). Given a PR number or merged SHA range it
**joins** all sources into one append-only JSONL row keyed by `(sha, pr, bead, session, arm)`:

```text
{ pr, bead, sha, arm(treatment|control), agent_type, surface,
  tokens_in, tokens_out, cache_read, cache_write, effective_input_tokens,
  cache_hit_ratio, wall_clock_s, usd_est(opt-in price flags only),
  first_ci_attempt, ci_first_green(bool), force_push_count, post_first_push_commits,
  no_rework(bool), churn_added, churn_deleted,
  roborev_findings{high,med,low}, roborev_verdict, review_changes_requested,
  first_shot(composite bool),
  # QUALITY columns:
  coverage_delta, mutation_score_delta, conformance_floor_moved,
  seeded_canary_find_rate, post_merge_revert(bool, backfilled later) }
```

`effective_input_tokens` is the §1.2 cache-discounted figure, so a "win" that is purely a
cache artifact (nominal input did not drop) is visible per `dogfooding-…` §5.4 KILL-1.
Honesty rules (inherited from the existing telemetry tool): the JSONL lives under a
**git-ignored** path (it carries session-local, non-canonical numbers and may contain
repo-internal detail); the **committed** artifacts are the script + the synthetic-fixture
test + the verdict schema only — never a measured number in markdown, never a real
transcript. Prices are **opt-in flags**, never hard-coded (a stale price in a measurement
tool is itself a lie); `roborev cost` is an explicit lower bound (only some agents report
$) — report its coverage.

### 1.5 The A/B protocol is the shared §5 spec — applied here

This program does **not** define its own statistics. The experiment design — frozen,
stratified, counterbalanced within-task A/B; record/replay-pinned model behaviour; charge
the treatment arm *all* its costs incl. amortised one-time setup; the pre-registered bar
(**≥20% paired-median effective-input reduction AND Wilcoxon p<0.05 over ≥30 tasks AND a
bootstrap 95% CI on the median delta whose lower bound is >0**); the break-even-N
computation; the three-axis arm-blinded quality grading; and the verdict object — is the
shared contract in `dogfooding-sparq-knowledge-graph.md` **§5.1–§5.6**. Read it as part of
this plan.

What this program contributes to that shared spec for the effectiveness surface:

1. **Stratify the frozen corpus by surface** (rust-feature / site / ci-infra / docs) **and
   by size**, in addition to the `dogfooding-…` §5.5 query-type strata (point-lookup /
   multi-hop / synthesis / negative). Report per-stratum, because a tool can win on large
   pattern-heavy tasks and lose on the small-task stratum.
2. **Bind the first-shot composite (§1.3)** as the primary *quality* guard for tool/skill
   A/Bs (the `dogfooding-…` track's quality axes are answer-accuracy / provenance /
   hallucination, which are right for retrieval but not for a code-edit tool).
3. **Interleave within one session window** so both arms share model + cache-warmth +
   season conditions and drift cancels (the §5.1 counterbalance, made explicit for the
   fan-out setting).

**Single verdict object.** Adoption decisions use the `dogfooding-…` §5.6 verdict object —
`{token_win, token_delta_median_pct, token_delta_ci, quality_delta{…}, break_even_N,
break_even_infinite, honest, recommend_adopt}` — with `recommend_adopt = true` requiring
`token_win` AND quality non-regression AND a finite `break_even_N`. No competing schema and
no second threshold set is introduced here.

### 1.6 Kill-criteria are the shared §5.4 spec — plus two surface-specific guards

The mechanical kill criteria (KILL-1 token / KILL-2 break-even / KILL-3 quality) are
`dogfooding-sparq-knowledge-graph.md` **§5.4** — used verbatim. Two additional guards are
specific to the *tooling* surface this program covers:

- **Prefix tax** (an MCP not behind Tool-Search deferral re-bills the prefix every turn
  across every parallel agent) is already a `dogfooding-…` §5.5 overclaim trap; here it is
  promoted to a hard reject **on the small-task stratum** even if the tool wins on large
  tasks.
- **Honesty-sensitive defect**: a single roborev **High** that proves a real defect on a
  ZK/MPC/security surface is a hard reject (the repo's existing arm-on-verdict discipline).

**Experiment STOP rule** (the §5.1 confound guard, applied per-wave): abort and re-baseline
if the *control* arm's metrics drift mid-run — a model/effort switch or cache-regime change
crept in.

### 1.7 Anti-gaming rules beyond pairing (each tied to a concrete attack)

These supplement the `dogfooding-…` §5.5 overclaim traps (nominal-vs-effective tokens, MCP
prefix tax, task-set selection bias, circular quality measurement):

1. **No self-report** — the harness derives every number from transcript + GitHub +
   roborev; agents never write their own token/quality figures into the row.
2. **Rework is ref-event-derived, not message-derived** (§1.3) — closes the amend/
   force-push hole in the primary quality guard.
3. **Quality canaries** — a small seeded-defect task set validates "fewer pushbacks =
   cleaner code"; a reviewer that misses seeded bugs is *blinded*, not better.
4. **Churn-normalize** — report tokens-per-changed-line (+ bead-completion check) so an
   agent cannot "win" by doing less work or truncating the task.
5. **Mutation + coverage as the CI-first-pass pair** — the structural defense against
   "green because tests were deleted/weakened."
6. **Report cache-hit-ratio alongside tokens** — so a "win" that is really just warm-cache
   (a confound, not a tool improvement) is visible and excluded.

### 1.8 Governance

The harness is **cheap and model-free** (deterministic scripts over already-emitted JSON;
no model-in-the-loop per row — an expensive measurement layer would itself be a token
regression). It **ranks and gates; it never auto-adopts.** Adoption stays a maintainer
judgment on the verdict object, and auto-merge/auto-adopt on a green metric is explicitly
out of scope (merging is irreversible; fabrication-detection is irreducibly human/codex —
`orchestration-automation-design.md` E1/T1b). All numbers are work-box/session-local and
**non-canonical** (`MEMORY: project-ec2-execution-env`) — used for before/after deltas and
ranking, never baked into committed markdown or a gate threshold.

---

## 2. Prioritised improvements by ROI

Ranked by **measurable-win-per-effort**, lowest-risk first. Every entry is **gated on the
§1 baseline + the shared §5 verdict** unless marked *free / no-product*. The borrowed
percentages below are **advisory only** — they rank candidates *before* the baseline
exists and carry **no weight** in an adoption decision; the verdict object does. They are
this-repo-unverified vendor/third-party numbers and must be treated as such (§6).

### 2.1 Tier 0 — free, no product, do-now (highest ROI)

These are discipline/config changes with first-party or already-shipped backing; they cost
only the §1 instrument to confirm.

- **Capture the cache baseline (the gate on everything).** The instrument exists
  (sq-dhss) but **no baseline was ever captured** — epic sq-lhwo is still flying blind.
  Wire `agent_telemetry.py` into a `Stop`/`SubagentStop` hook (or a thin wrapper over the
  live session JSONL), run it across **one** refill→verify wave, record cache-hit-ratio +
  cache-write cost to a runtime-only log. Cache reads cost **0.1×** input vs **1.25–2×**
  for writes, and each sub-agent + each worktree gets its **own** cold cache — the
  fan-out cache-miss is the named root cost (`agent-efficiency-tooling.md` §1–2).

- **Put AGENTS.md on a context budget.** At origin/main HEAD it is **105,185 bytes**
  (~26K est tokens). It is the prefix the orchestrator AND every role agent re-establish;
  on a cold fan-out turn every extra ~1K tokens is paid at the **1.25–2×** write multiplier
  × fan-out width. Growth is steep and monotonic: a pinned earlier anchor is **17,405
  bytes** on 2026-06-14 (commit `27c95f90`) — i.e. roughly **6×** in a week. The exact
  origin point of the older "~64.5 KB" figure is not pinned to a commit, so it is dropped;
  the pinned anchor + the current HEAD size are the load-bearing facts, and the **direction
  (sharply up, no guard)** is what matters. The "Proactively maintain this file" rule
  (`AGENTS.md`) drives this with no countervailing trim. Add a soft token/line budget
  (a stdlib lint in `docs-quality.yml` that warns past a threshold) and pair "maintain"
  with "**and trim/relocate when you do**" so durable detail migrates to a `SKILL.md` or
  `research/` instead of accreting in the always-cached prefix. Trim by **relocation, not
  deletion** (a deleted gate rule is a regression).

- **Codify + apply the Explore-vs-general-purpose-vs-Workflow decision.** Currently only
  in orchestrator auto-memory (`feedback-workflow-researcher-pr-gotcha`), so sub-agents and
  fresh sessions don't inherit it. Read-only survey/verify/frontier legs should use
  **`agentType: 'Explore'`** (lighter read-only system prompt, no Write/Edit/PR tooling in
  the cached prefix → smaller cache-write + removes the spurious-PR risk the gotcha
  documents). Concretely, in `.claude/workflows/autonomous-scheduler.js`: the **Frontier**
  phase agent (`agentType: 'general-purpose'`, the `phase: 'Frontier'` call) and the
  **Verify** phase agent (`agentType: 'general-purpose'`, the `phase: 'Verify'` call) are
  both explicitly "READ-ONLY; do NOT git-checkout", and `pickAgent()` defaults non-Rust
  surfaces to general-purpose. Switch those read-only phases to `Explore`; keep mutating
  impl on the role agents. **Limit: Explore is read-only — never route a mutating bead to
  it (it would silently no-op).**

- **Add a re-read-avoidance rule** to the sub-agent shared contract: "Do not re-Read a
  file you just edited to verify (Edit fails loudly if it didn't apply); do not re-Grep /
  re-Glob a surface you already searched this turn — reuse the result." A compounding
  per-agent token leak across fan-out, removed by one charter line.

- **Encode the "three tools, agent chooses" query→tool map** in AGENTS.md / briefs (free,
  matches the measured AGENTS.md-study win): exact text/errors → **Grep**; code
  shape/refactor → **AST/ast-grep**; callers/defs/types → **LSP**; conceptual "where do
  we…" over **design prose** → **semantic search** (if §2.4 adopts it) else Grep over
  `research/`. This captures most retrieval value at zero infra cost.

### 2.2 Tier 1 — AST / symbolic code-navigation (gated, low friction)

The honest framing from the industry-wide retreat from vectors-for-code (Anthropic removed
vector search from Claude Code May 2025; Cursor/Windsurf/Cline/Devin/Sourcegraph Amp
dropped vectors for tool-driven search): **code questions are mostly not text questions and
need three distinct tools the agent picks per query, not one index.** This harness already
has two of the three — Grep/Glob (lexical) and LSP (graph/structural). The gap AST fills is
**structural pattern matching** without LSP server overhead.

The query-type → tool map (the load-bearing result; the percentages are advisory, §6):

| Query type | Best tool | Why |
|---|---|---|
| exact strings / errors / logs | grep/ripgrep | zero false-negatives, no setup, fails *loudly* |
| code shapes / call patterns / codemods | **AST (ast-grep)** | avoids comment/string false positives that text grep hits |
| fuzzy symbol names, ranked | BM25 / symbol search | higher recall than ripgrep (third-party est.) |
| caller / usage relationships | graph / LSP find_usages | resolved edges, zero false positives, fast on large repos |
| **intent** ("the auth-related code", "where do we handle staleness") | semantic / hybrid | the **only** defensible niche for embeddings (§2.4) |

**Adopt-here (gated):**

- **`ast-grep` as a SKILL** (`.claude/skills/ast-grep/SKILL.md` + example rules). Tree-sitter
  structural matching, zero runtime deps, Rust CLI; ideal for sparq's 30-crate / ~606-file
  Rust workspace ("find all impls of trait X", "locate all calls to fn Y with pattern Z").
  **Two install/invocation gotchas the skill MUST mandate** (verified on this box):
  - **`ast-grep` is NOT installed here** (`which ast-grep` → not found), so the skill must
    carry an explicit install step (`cargo install ast-grep --locked` or the pinned release
    binary) **and a verify step** (`ast-grep --version`) before any rule runs.
  - **Do not invoke it as `sg`.** The convenience alias collides with the system binary:
    `/usr/bin/sg` is a symlink to **`newgrp`** on this box (`ls -l /usr/bin/sg` →
    `sg -> newgrp`). The skill must mandate the full **`ast-grep`** command name in every
    example and forbid the `sg` alias.

  A third-party estimate of "20–40% file-read reduction vs grep on pattern-heavy tasks"
  exists but **no independent controlled agent benchmark vs grep exists**, so this is the
  Phase-1 A/B candidate (§8) and the number is advisory only (§6). Setup cost low (one
  install); learning curve medium (YAML rule syntax). An `ast-grep-mcp` server exists (four
  tools: dump_syntax_tree, test_match_code_rule, find_code, find_code_by_rule) but **must**
  load behind Tool-Search deferral if used, per the §1.6 prefix-tax reject.

- **Structured reading discipline (outline → skeleton → selective read)**, applied
  regardless of any index tool. Third-party reports put read operations at the majority of
  agent tokens and outline-before-full-file / skeleton reads at large savings — **all
  advisory** (§6). Add to `sparq-rust-feature.md` + `sparq-researcher.md`: "if a file is
  > 200 lines, first obtain an outline/skeleton (functions + signatures), read only the
  sections you need." Cost: discipline only.

**Deferred / measured-trial (not now):**

- **Serena** (LSP-over-MCP, MIT; 40+ langs incl. Rust): symbol-level edit tools.
  The independent ManoMano trial (36K-LOC Java) passed tests where vanilla Claude+LSP
  failed, at comparable $ — but this is a **quality/reliability win, NOT a token cut**
  (prompt-caching of symbol reads kept token count flat). Barriers: rust-analyzer cold
  start on a 30-crate workspace is slow (minutes), and the "zombie language server on exit"
  issue is documented + widespread. Frame any adoption as a *refactor-quality* upgrade,
  measure latency + edit-error-rate, pre-index once in a background task.

- **codebase-memory-mcp** (tree-sitter KG + Hybrid-LSP, 158 grammars): **self-reported**
  large token / tool-call reduction — but with a **documented quality drop**, preprint not
  peer-reviewed. Risk: subtle bugs on cryptographic code. Head-to-head measured trial only;
  gate on the shared §5.6 verdict object (token win AND quality non-regression AND finite
  break-even-N), else archive as "measured and rejected."

**Skip:** SCIP/Sourcegraph (no standalone agent MCP, needs a Sourcegraph instance, markets
precision not token reduction); aider repo-map (internal to aider, not a standalone MCP;
RepoMapper MCP is early-stage); claude-context (needs an embedding provider or self-hosted
Milvus — cost/ops creep, not justified for this brief).

### 2.3 Tier 2 — repeated-task workflows / skills / hooks (durability, zero-token coordination)

The granularity decision rule (the core deliverable), by *who-drives* × *when-it-fires* ×
*token-cost*:

| Primitive | When to use |
|---|---|
| **HOOK** (SessionStart/Stop/PreToolUse) | harness fires deterministically, **zero model tokens**, on a lifecycle event → invariant context injection + guardrails that must never be forgotten |
| **WORKFLOW** (`.claude/workflows/*.js`, ultracode) | code-driven fan-out, coordination spends **zero model tokens** → multi-agent loops with mechanical coordination (merge-train, bead-frontier scheduler, bench matrix) |
| **SKILL** (`SKILL.md`) | model-read procedure loaded on demand, judgment stays with the model → a **single-agent** repeatable procedure needing reasoning each run (issue-triage, charter sweep, proceed-and-document) |
| **SLASH-COMMAND** | the human-typed entry point / thin alias → the *ergonomic trigger*, not the logic home |
| **CRON / RemoteTrigger / loop** | the *clock* that re-kicks a workflow/skill → the **safety-net floor only**, never the primary cadence |

Rule of thumb: deterministic+lifecycle → hook; multi-agent+mechanical → workflow;
single-agent+judgment → skill; scheduled re-kick → cron/loop; human handle → slash-command.

**The biggest concrete gap:** the *proceed-and-document* standing rule and the merge/triage
briefs are **trapped as inline prompt strings** inside `autonomous-scheduler.js`. The impl
and verify briefs are at least named functions (`implPrompt()` and `verifyPrompt()`, defined
right after `pickAgent()` near the top of the file), but the **Frontier brief is a bare
inline string literal** in the `phase: 'Frontier'` agent call (there is **no**
`frontierPrompt` function) — and none of the three is reusable outside the workflow. So a
**hand-dispatched** agent (outside the workflow) does not inherit them. Fix: hoist each
recurring brief into a `SKILL.md` and have **both** the workflow strings AND hand-dispatched
agents reference the skill by name — the single-source-of-truth pattern `AGENTS.md` already
applies to the sub-agent shared contract, applied one level up to the recurring-procedure
layer.

**A durability gap:** `autonomous-scheduler.js`'s own header claims it is "linked from
AGENTS.md so it is not lost on a session restart" — but **AGENTS.md contains no such
link.** The durability contract for any durable workflow/skill is: (1) committed to
`.claude/`, (2) linked from AGENTS.md's maintenance-loop section, (3) given a one-line
skill description so it is discoverable. Missing any one → it gets re-improvised.

**Adopt-here:**

- Extract a **`proceed-and-document` SKILL.md** from the inline scheduler prose
  (best-judgment choice + document in PR/bead + open a feedback issue, do NOT block on
  greenlight); point both the workflow strings and hand-dispatched agents at it. Land with
  sq-6psmk.
- **Add the missing AGENTS.md link** to `autonomous-scheduler.js`, and make
  "committed-file + AGENTS.md-link + one-line skill description" the standing durability
  contract for every future durable workflow.
- Extract a **`charter-sweep` SKILL.md** from the existing AGENTS.md maintenance-loop prose
  (pull sibling charters, fold portable conventions, push outbound, watch threads, graduate
  shipped research). Parameterise `--pull-only / --push-only / --watch-only`.
- Extract an **`issue-triage` SKILL.md** (screen `gh issue list` → bead actionable ones with
  `--external-ref` + severity-mapped priority → comment bead id → close on merge →
  ask-on-unclear, PSS-aware); invoke from the maintenance-loop workflow.
- Build a **`merge-train` WORKFLOW** (`.claude/workflows/merge-train.js`): reconcile
  worktrees → merge each ci-summary-green + threads-resolved PR one-at-a-time → watch main
  CI → reap worktrees. Parameterise `maxMerges` + `--dry-run`. (Replaces improvised
  background `gh run watch` jobs.)
- Wire a **`Stop` hook** that runs the safety-net sweep tail deterministically at turn end
  (reconcile worktrees, surface `bd list -l needs:user` count, re-kick the scheduler **IF**
  frontier non-empty AND no run in flight) — **gated on a sentinel** so it never fights an
  intentional stop. Harness-native "the loop is a safety net, not the cadence" at zero
  model-token cost. *Risk: an ungated Stop hook that re-injects work can create a
  non-terminating session — must be bounded by a `.scheduler-stop` kill-switch.*
- Build **`scripts/scheduler-status.sh`** (zero-token renderer of run-id / pass /
  local-slots / ec2-spend / armed-withheld / escalations) read by the Stop-hook re-kick and
  the maintainer at a glance.
- A **thin slash-command layer** (`/charter-sweep`, `/triage`, `/merge-train`,
  `/scheduler`) as ergonomic handles only — logic stays in the skill/workflow.

**Parameterisation contract** for every durable workflow/skill (what makes them re-runnable,
not re-improvised): (a) a scope/filter arg, (b) a cap (maxBeads / cost / wall-clock),
(c) a `--dry-run`. Fix the recurring *shape* in the committed file; vary only *scope* per
invocation. *Risk: over-factoring into many tiny skills re-creates the duplication this is
meant to remove — each skill must name its authority and the workflow strings must POINT to
it, not re-state it.* Continuity caveats: cron/loop is the low-frequency **re-kick floor
only** (in-session CronCreate dies with the session; recurring jobs auto-expire after 7
days; only `durable:true` or a RemoteTrigger cloud routine survives — there is deliberately
**no orchestration daemon** in this repo). Freshly-edited hooks/settings are **not
hot-loaded mid-session** — any hook change is a next-session change.

### 2.4 Tier 3 — vector / semantic lookup (narrow, gated, prose-only)

**The honest line is corpus-shaped, not "vectors win" or "grep wins":** for **code**,
grep+AST+LSP win and the industry abandoned vectors; for **discursive design prose +
ambiguous intent + multi-hop synthesis**, embeddings retain a **real but modest** edge.
Amazon Science's "Keyword search is all you need" (arXiv 2602.23368, AAAI 2026) found
agentic keyword search reached most of RAG's faithfulness / context-recall /
answer-correctness with **no vector store** — but explicitly **degraded on discursive
prose**, large docs, and ambiguous queries, with **no claim of generalization to code or
multi-hop.** Embeddings' real edge is concentrated exactly on **this repo's `research/`
design prose** (124 docs, ~3.27 MB), **not its ~606 Rust files.**

The pro-vector evidence is real but **vendor-sourced, relative, model-dependent** (§6):
Cursor's internal retrieval-accuracy uplift is relative-to-undisclosed-baseline and varies
by model, and Cursor still recommends grep for exact errors/names; Turbopuffer's "wasted
reads 1-in-3 → 1-in-8" is a *precision* win, not quality; Milvus's "40% fewer tokens" omits
its own infra cost. None is independent proof of a token win for our local fan-out.

Failure modes (all named, all relevant): **(1) stale index** — every edit makes a chunk
stale; the agent reads code that no longer exists; **(2) chunking heuristics** — fixed
windows lose context / retrieve noise (AST/heading boundaries beat them); **(3) silent
drift** — vectors fail *silently* (wrong-but-plausible chunk → agent confidently calls a
non-existent method), whereas grep fails *loudly* (no match = no result); **(4) embedder
drift** — changing the model invalidates the whole index; **(5) structural blindness** —
embeddings don't capture imports/call-graph/type edges (the original reason Claude Code
dropped them).

**Lightest-weight thing worth trying (and only this):** a **local, AST-chunked embedding
index scoped to the PROSE corpus only** — `research/*.md`, `AGENTS.md`, the `SKILL.md`s,
bead text — exposed as **one** `semantic_search` tool the agent calls **only** for
intent/synthesis/"where did we decide X" questions. **Do NOT index the Rust tree** (grep +
LSP already win the lexical/structural/graph rows there, and a code embedding index just
adds staleness + a quality-drop risk). `cocoindex-code` (Rust + tree-sitter, local
SentenceTransformers, incremental re-index, no external DB) is the cheapest off-the-shelf
realisation; `git grep` + **BM25** (no model) is the even-cheaper floor.

**Gate it, do not adopt it:** this is exactly the `dogfooding-sparq-knowledge-graph.md` §5
A/B — run a before/after on ~15–20 real "intent" questions over `research/` (e.g. "what is
the propose-then-verify invariant", "where did we reject hyperbolic geometry") comparing
**(a)** plain Grep/Glob exploration, **(b)** git-grep+BM25, **(c)** the local AST-chunk
index — measuring cache-discounted effective tokens AND the three quality axes (answer
accuracy / provenance-completeness / hallucination). Adopt only if the §5.6 verdict object
returns `recommend_adopt` on THIS corpus (the Amazon-Science result predicts the margin is
small and the failures are the discursive/ambiguous cases).

**Mitigate the failure modes from day one:** (staleness) tie re-index to a git
post-commit/file-change hook and pin freshness to HEAD, or re-rank hits against **live**
file content before returning; (chunking) tree-sitter / markdown-heading boundaries, never
fixed windows; (silent drift) the tool returns **path + line span** so the agent re-reads
real source — *fail loud*, never the embedded chunk as ground truth; (embedder drift) pin
the embedder version in the index header, rebuild on change; (tool tax) load via Tool-Search
deferral behind the cache breakpoint.

### 2.5 Evidenced coding-agent best-practices (cite)

What is actually working, with evidence-grade markers (all percentages advisory, §6):

- **Prompt-cache hygiene** — *[independent + first-party]*. Cache reads 0.1× vs writes
  1.25–2×; the AGENTS.md consolidation study (Lulla et al., arXiv 2601.20404) measured
  −16.58% output tokens / −28.64% runtime across 124 PRs / 10 repos. **The single largest
  cost lever**, and this repo's brief discipline already targets it.
- **Sub-agent decomposition + tight feedback** — *[this-workflow-observable +
  SWE-agent comparative]* (Jimenez et al., arXiv 2405.15793). Smallest context-independent
  briefs, per-worktree gating, merge-one-at-a-time + re-gate, beads (no inline TODOs),
  ratchets that only ever rise. The Lulla study named structured AGENTS.md as the key
  gate-passing differentiator vs vanilla agents.
- **Test-driven / pre-implementation spec** — *[SWE-bench observation]*. `test-driven-development`
  skill + structural ratchets (conformance, perf baselines, coverage floors, unsafe
  snapshots, mutation ceilings, all gate-checked before accepting a change). SWE-bench
  leaders report higher pass rates when tests/gates exist before the fix attempt.
- **Tool Search / deferred loading** — *[official Anthropic first-party]*: large input-token
  reduction at hundreds of tools, held pass-rate + improved accuracy. The **precondition**
  for any MCP index server (so its schema does not re-bill the prefix every turn). Codified
  in `AGENTS.md` but **never verified active** on this model/provider — verify, then it
  gates Tiers 1–3 MCP options. Provider caveat: not available on Haiku, Vertex AI, or
  custom gateways (tools load into the prefix) — document in AGENTS.md if an alternative
  provider is ever used.
- **ReAct-style plan-then-act + self-correcting loops** — *[this-workflow-observable +
  published SOTA]* (Text2SPARQL'25 leaders mKGQAgent / ARUQULA, arXiv 2510.02200).
- **Cross-crate parity gates (differential fuzzing)** — *[this-workflow-observable]*: a
  finding in one path is evidence to check the others (TriG if Turtle is fixed; all
  operators if one is); enforced via the `fuzz` lane + differential oracles.

**Hyped with weak evidence (caution):** "AI memory" products claiming 49–71.5×/95–99% token
reduction are *[anecdotal/vendor]*, never independently replicated — the strongest
*independent* memory benchmark (Sandelin, 2026) measured only **15–28%** on a single
author's setup; most "95–99% optimizers" apply to cache hits Claude Code already gets
automatically; graph-memory-MCP products have **no controlled coding-agent benchmark**.
Hard-coded perf numbers are non-canonical (hardware + thermal state matter).

---

## 3. What THIS workflow already does well vs the gaps

**Already strong (do not re-litigate):**

- **Brief discipline is done** (Phase-2, sq-or5m / #799): the sub-agent shared contract is
  single-sourced in `AGENTS.md`; role briefs are lean and point at it ("state the task, not
  the contract"). Cache-hygiene rules are codified (stable invariant prefix, pinned
  model+effort, read-only agents may share the main checkout, defer MCP loading).
- **The telemetry instrument exists** (Phase-1, sq-dhss / #767): `agent_telemetry.py`
  reports per-agent/per-wave tokens, cache-read vs cache-creation (5min/1hr TTL split), and
  cache-hit-ratio.
- **An adversarial, non-Anthropic reviewer is wired** (roborev/codex) with a queryable SQL
  store — the fabrication check the whole measurement backbone leans on.
- **A deterministic-vs-advisory gate split already exists** (`perf-gate.py`) — the exact
  template the metric+quality pairing copies.
- **Role-specific agents + an autonomous bead-frontier workflow** exist
  (`autonomous-scheduler.js`, sq-sgu1).

**The gaps (ranked by measurable-win-per-effort):**

1. **No baseline was ever captured** — the instrument shipped, the *measurement* did not;
   every %-claim remains borrowed. **This is the gate on every other adoption.**
2. **AGENTS.md prefix bloat** (105,185 bytes at HEAD, sharply up from a pinned 17,405 B on
   2026-06-14, no budget guard) — actively fighting the cache hygiene it preaches.
3. **Explore-vs-general-purpose decision not in AGENTS.md** — read-only scheduler legs run
   as heavyweight general-purpose agents (bigger cache-write + spurious-PR risk).
4. **No re-read-avoidance rule** — a compounding per-agent token leak.
5. **Recurring briefs trapped as inline workflow strings** (proceed-and-document, merge,
   triage; the Frontier brief isn't even a named function) — re-typed, not reused;
   hand-dispatched agents don't inherit them.
6. **Missing durability links** (scheduler not linked from AGENTS.md; no `Stop` hook; no
   `scheduler-status.sh`; no slash-command layer).
7. **Tool-Search deferral codified but unverified;** Phases 3–6 of the efficiency plan never
   beaded.

---

## 4. Cross-references — extend, do not duplicate

- **`agent-efficiency-tooling.md` (epic sq-lhwo, #430):** the cost-side survey + ranked
  shortlist + phased plan. This program **is** the missing measurement backbone for that
  epic's telemetry-first mandate (§8/§10 Phase-1), extended from "token telemetry per wave"
  to "a falsifiable per-change experiment with anti-gaming guards." Do not re-rank its
  levers; consume them.
- **`dogfooding-sparq-knowledge-graph.md` (sparq-PKG track):** "sparq AS its own KG", and —
  load-bearing for this doc — the **single home of the shared A/B measurement protocol
  (§5)**: counterbalanced within-task A/B, the ≥20%/Wilcoxon/≥30-task bar + bootstrap-CI,
  break-even-N, the three-axis arm-blinded quality grading, and the verdict object. This
  program references that spec rather than redefining it; see §5 here for the division of
  labour. This program is store-agnostic and works on plain JSONL + `jq` first; the PKG
  track may *optionally* back the metrics rows / the prose index with a sparq graph.
- **`feature-research-vector-genai.md` (epic sq-3183)** + **`structure-aware-vectorisation.md`
  (epic sq-0wo9e)** + `crates/sparq-vectors/src/`: the eventual structure-aware,
  provenance-carrying `vec:`-in-SPARQL implementation — the sparq counterpart whose
  acceptance test is **"beat the §2.4 general baseline on the same intent-query set."**
- **`autonomous-scheduler-design.md` / `orchestration-automation-design.md`:** the workflow
  primitives, the no-daemon continuity model, and the mechanical-vs-judgment boundary
  (adoption must not be auto-gated).
- **`genai-benchmarks-and-synthesis.md`:** the dogfoodable sparq-GenAI agenda
  (grammar-constrained decoding N2, provenance-carrying answers N5, self-correcting NL→SPARQL
  N6) — gated, not vendor-hyped.

---

## 5. Division of labour: this program vs the sparq-PKG dogfooding track

The measurement protocol is **shared, single-sourced in
`dogfooding-sparq-knowledge-graph.md` §5** — it is NOT "owned" by this program and
"consumed" by the other. Both tracks use the **same** statistics, the **same** thresholds,
and the **same** verdict object. The split is purely *what each track measures*, not *who
defines the experiment*:

| Concern | This program (general, with-or-without-sparq) | sparq-PKG dogfooding track |
|---|---|---|
| A/B protocol, significance bar, kill-criteria, verdict object | **uses the shared `dogfooding-…` §5 spec** | **authors + houses** the shared §5 spec |
| Surface/size stratification + first-shot composite as the quality guard for *tool/skill* A/Bs | **adds these to the shared spec** | uses answer-accuracy / provenance / hallucination axes |
| Collection harness | plain JSONL + `jq` (the honest baseline), `metrics_row.py` | **may** back the rows with a sparq graph for querying |
| Code retrieval | grep + LSP + AST (the three tools); semantic only for prose | — |
| Prose/intent semantic search | **off-the-shelf local AST-chunk baseline** (§2.4) | **structure-aware `vec:`-in-SPARQL** build, validated *against* this baseline |
| Acceptance criterion | shared §5.6 verdict returns `recommend_adopt` on the frozen corpus | beat **this** general baseline on the same intent set |

Why single-source §5 and not duplicate it: the two docs were independently drafting
*overlapping-but-non-identical* protocols (an earlier draft of this doc carried a "15%
detectable floor" and lacked a break-even-N, a verdict schema, and an explicit arm-blinded
grader, while `dogfooding-…` §5 already had all of those plus a ≥20% adopt bar). Two
near-duplicate protocols with different thresholds is exactly the partial duplication this
program is meant to avoid, so the `dogfooding-…` §5 spec — the more complete one — is the
**single** authority and this doc references it. If §5 ever moves (e.g. the PKG record is
retired), the spec must be relocated, not re-forked.

The two tracks meet at exactly two seams: **(1)** the prose semantic index (the PKG build
must beat the general baseline, never be measured only against itself), and **(2)**
re-read-avoidance (§2.1) is where a queryable context store would later plug in. Keep the
measurement instrument **decoupled from the thing being measured** — the framework must work
with plain JSONL first. Scope guard: this program does **not** build structure-aware
vectorisation (that collides with sq-0wo9e/sq-3183); it owns the off-the-shelf baseline +
the data-source join + the query→tool decision-map only.

---

## 6. Risks & hype to avoid

- **Borrowed numbers.** Every % here and in the cited docs is someone else's repo, and in
  this doc they are **advisory ranking aids only** — they carry no weight in an adoption
  decision; only the shared §5.6 verdict object does. The closed Phase-1/Phase-2 beads must
  **not** create the impression a sparq-specific saving was proven — the instrument shipped,
  the measurement did not.
- **Self-fighting bloat.** AGENTS.md's "proactively maintain" rule grows the always-cached
  prefix monotonically (17,405 B → 105,185 B in a week) while the same file preaches cache
  hygiene; trim by relocation.
- **History-rewrite gaming of first-shot.** `--amend` + force-push can hide a rework commit;
  the harness derives `no_rework` from ref/force-push events, not commit messages (§1.3).
- **Silent drift is the worst failure for engine-critical code** — a vector index can make
  an agent confidently call a non-existent method; the Rust tree is **excluded** from any
  embedding index and every hit re-reads live source.
- **Warm-cache masquerade.** A "win" that is really just warm cache is a confound — report
  cache-hit-ratio + the cache-discounted effective-token components alongside every token
  number and stratify; use interleaved/counterbalanced A/B, never before/after.
- **Underpower.** Below the shared §5.1 bar (≥20% paired-median, Wilcoxon p<0.05, ≥30 tasks,
  bootstrap CI lower-bound >0) the honest verdict is "no evidence," not "no effect" and not
  "win."
- **Don't over-invest in memory/index products before the baseline.** Their evidence is
  vendor/self-report or single-team; one carries a measured quality drop; the win is cache
  hygiene + discipline (free), not a product.
- **Serena = quality, not tokens.** Frame it as a refactor-reliability upgrade; measure both.
- **Over-automation.** The framework ranks and gates; it never auto-merges/auto-adopts on a
  green metric. Stop hooks that re-inject work must be sentinel-gated and bounded.
- **Provider/mid-session caveats.** Tool-Search deferral is Claude-Code-only; edited
  hooks/AGENTS.md are next-session, not hot-loaded.

---

## 7. Phased plan

Each phase is a future bead under epic **sq-lhwo**, cross-referenced to sq-3183/sq-0wo9e.

**Phase 0 — preconditions (config only, no measurement needed):**

1. Verify Tool-Search / deferred tool loading is active on this model/provider; mandate it
   as a precondition for any MCP adoption.
2. Quick-win discipline edits (no product): re-read-avoidance rule + the three-tools
   query→tool map into AGENTS.md; outline-before-full-file into the role briefs.

**Phase 1 — stand up the instrument + capture the BASELINE + A/B the top-1 low-risk tool
(the gate on everything):**

1. Build `scripts/agent-telemetry/metrics_row.py` + synthetic-fixture test (joins
   telemetry + roborev + gh + bd into the §1.4 row, incl. the ref-event-derived `no_rework`
   and the cache-discounted `effective_input_tokens`; no token re-derivation).
2. Wire `agent_telemetry.py` into a `Stop`/`SubagentStop` hook (or live-JSONL wrapper);
   run one refill→verify wave; record the **real** cache-hit-ratio + cache-write cost to a
   runtime-only log. *This is the deliverable sq-dhss instrumented but never exercised.*
3. Freeze the A/B task corpus (stratified by surface + size + the `dogfooding-…` §5.5
   query-type strata) + the pre-registration template + the seeded-bug canary set, all
   against the shared §5 spec.
4. **A/B the `ast-grep` SKILL** (the highest-ROI low-risk tool) against grep on a fixed
   pattern-heavy task set, per the shared §5 protocol; adopt iff the §5.6 verdict returns
   `recommend_adopt`.
5. Put AGENTS.md on a token budget (lint warn in `docs-quality.yml`) + add the
   "trim/relocate when you maintain" clause.

**Phase 1.5 — durability + read-only-agent-type (after the baseline exists):**

1. Extract `proceed-and-document` SKILL.md (with sq-6psmk); add the missing AGENTS.md link
   to `autonomous-scheduler.js`; codify the Explore decision matrix in AGENTS.md and switch
   the scheduler's Frontier/Verify phases + `pickAgent()` default to `Explore`.
2. Extract `charter-sweep` + `issue-triage` skills; build the `merge-train` workflow + the
   `Stop`-hook safety-net (sentinel-gated) + `scripts/scheduler-status.sh` + the thin
   slash-command layer.

**Phase 2 — measured trials, optional (concurrent, each gated on the shared §5 verdict):**

- `codebase-memory-mcp` head-to-head on the 30-crate workspace.
- Serena spike (gate: refactor-error-rate materially better at neutral-or-better $; manage
  rust-analyzer lifecycle + the zombie-process issue).
- The §2.4 prose-only semantic index A/B (grep vs BM25 vs local AST-chunk embeddings on the
  intent-query set).

**Phase 3 — sparq-native dogfooding (separate track, validated against this baseline):**
grammar-constrained SPARQL decoding (N2), provenance-carrying answers (N5), self-correcting
NL→SPARQL (N6), and the structure-aware `vec:` prose index — each behind `--features genai`,
each with accuracy + perf benchmarks, each acceptance-tested to **beat the §2.4 general
baseline** on the same intent set.

**Out of scope here:** Anthropic Memory tool/Stores (not wired to the CC CLI); Letta/Zep/
Basic-Memory (wrong shape for solo-maintainer fan-out); Sourcegraph MCP (cloud cost/ops
counter to the goal); any auto-merge/auto-adopt on a green metric.

---

## 8. Bead breakdown (proposed, under sq-lhwo)

1. `metrics_row.py` harness + fixture test (the join layer, incl. ref-event `no_rework` +
   cache-discounted effective-token columns).
2. Capture + recur the cache-hit baseline (wire telemetry into a Stop hook, one wave).
3. The frozen A/B task corpus + pre-registration template (against the shared §5 spec).
4. The seeded-bug canary set (pushback-quality validation).
5. Churn-normalized + cache-hit-ratio reporting columns.
6. Wire the mutation ratchet + coverage floor + conformance scoreboard as the
   CI-first-pass quality pair.
7. `ast-grep` SKILL.md + example rules + install/verify step + `sg`-collision guard + the
   Phase-1 A/B.
8. AGENTS.md token budget lint + "trim/relocate" clause.
9. `proceed-and-document` skill + the AGENTS.md durability link (with sq-6psmk).
10. Explore decision matrix in AGENTS.md + scheduler Frontier/Verify → `Explore`.
11. `charter-sweep` + `issue-triage` skills; `merge-train` workflow; sentinel-gated `Stop`
    hook; `scheduler-status.sh`; slash-command layer.
12. Verify Tool-Search deferral active; document provider constraints in AGENTS.md.

Each is independent and parallelizable except where noted; **#1–#2 gate all adoption.**
