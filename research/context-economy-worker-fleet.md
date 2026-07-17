# Context economy for the headless worker fleet — survey + ranked plan [FABLE-5]

> 🤖 SPARQ agent — design-for-maintainer-review. Survey + measured inventory + ranked
> recommendations + a pre-registered pilot plan. **Nothing in this record is adopted**;
> every token-saving claim below is gated on the real-effective-token A/Bs in §5.

Status: **design-only / proposed** (2026-07). Companion/successor to
[`agent-efficiency-tooling.md`](agent-efficiency-tooling.md) (interactive-orchestrator
era) — this record covers the **headless one-shot worker fleet** (registry-dispatched
`claude -p` / `codex exec` runs in Actions, one worker per issue; see
[`orchestration-private-registry-worker.md`](orchestration-private-registry-worker.md)).

## 0. Grounding rules (binding on every claim in this record)

This repo has already **falsified three plausible token-savers with real-token A/Bs**:

1. **Agent-invoked ast-grep/outline-FIRST exploration** cost ~21k effective tokens MORE
   than a scoped Read — a precision tool, not a saver
   ([`bench/pkg-dogfood/RESULTS-astgrep.md`](../bench/pkg-dogfood/RESULTS-astgrep.md)).
2. **A terse keyword syntax layer** saved ~0% real tokens — the session floor dominates;
   the apparent ~20% was a char-proxy artifact ([`bench/terse/RESULTS.md`](../bench/terse/RESULTS.md)).
3. **An early doc-read-vs-pkg char proxy** reversed on real measurement
   ([`bench/pkg-dogfood/RESULTS.md`](../bench/pkg-dogfood/RESULTS.md)).

The ONE measured big win: **delegating the verbose middle to a cheap-model round-trip
tool** (Haiku PKG NL-tool ~30× cheaper at equal quality; `pkg-query` roughly halved
Opus effective tokens vs doc-reads, N=30 — same record).

Two consequences, applied throughout:

- **Category discipline.** Distinguish sharply between (a) *agent-invoked exploration
  tooling* — the measured loser category — and (b) *precomputed, deterministic,
  prompt-prefix artifacts* amortized across a fleet + provider prompt caching. All three
  falsifications were category (a); category (b) is **unmeasured here** and has different
  economics. Every recommendation below states which category it is in.
- **No codification without a real-token A/B.** Char/byte proxies are inadmissible as
  verdicts. All figures in this record are **estimates from this investigation**
  (chars/4 unless stated, measured on the 2026-07 checkout + registry) — sizing inputs
  for the pilot design, **not canonical benchmarks**.

## 1. Current worker context economics (measured inventory)

### 1.1 What a claude-harness worker's prompt actually is

From the registry's `scripts/worker-live.sh` (`run_model` / `_run_headless_harness`;
CLIs pinned in `worker-prep.sh`: `@anthropic-ai/claude-code@2.1.177`,
`@openai/codex@0.144.1`):

```bash
claude -p --model X --permission-mode acceptEdits \
  --allowedTools Bash,Edit,Read,Write,Glob,Grep \
  --append-system-prompt-file .claude/agents/$agent.md \
  --no-session-persistence  < task-prompt.txt
```

Request assembly order, with sizes (estimates from this investigation):

| # | Layer | Size (est.) | Shared across a batch? |
|---|-------|-------------|------------------------|
| 1 | Claude Code builtin system prompt + 6 tool schemas | ~6–9k tok (not locally measurable) | yes |
| 2 | Appended role file `.claude/agents/$agent.md` | ~1.1–4.6k tok (4.4–18.5 KB) | yes, per role |
| 3 | env/gitStatus block — **contains the per-issue branch name** | ~0.3k tok | **NO — poisoned** (§1.3) |
| 4 | `CLAUDE.md` auto-load (886 B) | ~0.2k tok | yes |
| 5 | Generated brief: shared contract (~1,145 chars) with **`{scope}` interpolated mid-contract** (~char 431) | ~0.3k tok | **partially — poisoned** (§1.3) |
| 6 | Issue tail (bead title + description; N=1258 sample) | median ~130 tok, p90 ~300, max ~800 | no (this is the payload) |

Fixed overhead ≈ 10–14k tokens before the per-issue tail; **>98% of turn-1 bytes are
batch-identical** across same-role workers dispatched off one `origin/main` SHA. The
per-issue payload is tiny. The review leg is instructions (~0.4k tok) before a diff
capped at 400 KB (~100k tok max) — instructions-first is already the right order.

### 1.2 The auto-load chain asymmetry (the biggest doc-shaped item)

- **claude**: auto-loads ONLY `CLAUDE.md` (886 B), which instructs "read `AGENTS.md`
  first". A compliant worker then pays a **~36k-token Read** of `AGENTS.md`
  (143,992 B / 670 lines — under the 2000-line Read cap, so it arrives whole) in the
  conversation layer. Intra-run caching prices its *re*-reads at ~0.1×, but the first
  cost plus its share of every later turn's prefix is the largest single deterministic
  item in a worker run. Whether workers actually comply is unobserved (log-withholding
  policy); this is the compliant-path cost.
- **codex**: natively auto-loads `AGENTS.md` but **silently truncates** the combined
  instruction docs at `project_doc_max_bytes` (32 KiB default on the pinned version;
  the run uses `--ignore-user-config`, so defaults apply). Measured section offsets put
  byte 32,768 **inside the orchestrator-only "Orchestration — delegate to sub-agents"
  section**: ~13 KB of what codex loads is delegation text a worker must not act on, and
  every later worker-relevant section (hygiene, upstream rules, contribution lessons) is
  **never loaded, with no warning**. This is a **correctness bug** independent of any
  token claim.
- **Tiering measurement**: of `AGENTS.md`'s ~142.9k chars (~35.7k tok), the
  orchestrator-only sections (maintenance loop, contribution/arming workflow, post-batch
  checklist, sub-agent delegation, cadence, parallelism, roborev, beads, CI monitoring…)
  total ~109k chars ≈ **~27.4k tok (~76%)** — content the generated brief explicitly
  *overrides* for workers ("Orchestration contract (overrides any interactive/worktree/PR
  instructions…)"; workers may not commit, open PRs, call GitHub APIs, or run `bd`).
  The worker-relevant remainder is ~8.4k tok — which would also fit under codex's 32 KiB
  auto-load cap.

### 1.3 Two per-run byte differences currently defeat cross-worker prefix sharing

1. `run_model` runs `git switch -c sparq-agent/issue-N-<runID>-<attempt>` **before**
   launching the model, so the per-run-unique branch name lands in the system-prompt
   gitStatus block — after the shared builtin+role region but before the shared
   CLAUDE.md + contract bytes. The model never commits (the host asserts HEAD unchanged
   and commits in `publish_pr`), so branch creation can move **after** the model run
   with zero behavior change.
2. `{scope}` (the area label, ~20 B) is interpolated at ~char 431 of the 1,145-char
   shared contract, splitting it; moving it to the tail next to the issue makes the
   entire pre-issue prompt scope-independent.

Post-fix, the only per-issue bytes are the ~0.5–3.2 KB issue tail. An alternative or
complement is the documented fleet flag `--exclude-dynamic-system-prompt-sections`
(present in CLI 2.1.177; SDK `excludeDynamicSections`), Anthropic's own answer for
fleets — it moves all per-session env context into the first user message so identical
configurations share a cache entry across machines, at the cost of slightly demoting
env context.

### 1.4 Prompt-cache mechanics — verified corrections to fleet assumptions

- **Subscription-auth workers already run on a 1-HOUR cache TTL**, not the ~5min the
  fleet assumed (Claude Code requests 1h automatically on subscription; reads also
  refresh the TTL at no cost; sub-agents inside a run stay at 5m). The
  affinity/batching window is far wider than believed.
- **Caches are per-organization/account on both providers.** Each pool subscription is
  its own org; cross-account cache sharing is impossible *by design*. Account affinity +
  temporal batching is the only routing lever.
- **Concurrent fire pays N writes**: parallel requests with identical prefixes ALL pay
  the full cache write until the first response begins streaming. A batch fired
  simultaneously on one account gets zero cross-worker sharing; stagger the first
  worker, then fire the rest.
- Prices (provider list, for weighting only): Anthropic reads ~0.1×, writes 1.25× (5m)
  / 2× (1h); OpenAI caching is automatic ≥1024-token prefixes, cached input ~90% off on
  GPT-5.x. For our plan-billed workers the operative currency is **quota-window
  consumption**, and "cache reads are ~free against quota" is itself a **hypothesis to
  verify** (via per-account usage deltas), not a fact.
- **Bounded win**: cross-run sharing only converts each *additional* same-account
  worker's turn-1 shared-prefix cost from write to read — roughly (1.25−0.1)× the
  ~12–20k-token shared region ≈ **~14–23k token-equivalents per extra worker per
  batch** (estimate). A cheap enabler, not a headline saver. Intra-run turn-to-turn
  caching (the dominant economics) already works regardless.

### 1.5 A potentially dominant hidden cost: the PreToolUse Opus agent hook

Committed `.claude/settings.json` (so present in every worker checkout) defines a
`PreToolUse` matcher on **every Bash call** with (a) the cheap `check-pr-arm-base.py`
command hook and (b) a `type: "agent"` hook — `sparq-perf-reviewer`, `model: "opus"`,
~1.9 KB prompt + 10.6 KB agent file (~3–5k tok input per invocation). Its own prompt
short-circuits to "allow" for anything that isn't `gh pr merge --auto` — but the worker
container has **no `gh` binary** and the brief forbids GitHub APIs, so *if* agent hooks
execute in fresh-HOME non-interactive `claude -p`, every cargo/git/test Bash call pays
an Opus round-trip to conclude "not applicable". At 30–100 Bash calls per
implementation run that is ~100–400k Opus input tokens of pure overhead per worker
(estimate) — which would dwarf everything else in this record. **Whether the hook fires
headless is UNVERIFIED** (folder-trust/hook-approval gating may skip it); §5 pilot E is
the verify-first canary. The fix is trivial either way (command-hook prefilter on the
arming pattern, or a worker-mode settings override in the container). The `SessionStart`
bd hook provably no-ops in-container (`bd` not installed; script exits 0).

## 2. Technique survey (what the field actually supports)

### 2.1 Repo maps / AST skeletons as prompt artifacts

- **Aider's tree-sitter repo map** (PageRank-ranked, token-budgeted) is the flagship
  implementation but has **no published effectiveness benchmark** (verified by fetching
  the announcement post — no map-on/map-off numbers exist); its adoption scale is
  usability evidence, not token economics. It is also NOT a stable prefix (re-ranked per
  conversation). Aider's own FAQ documents the failure mode most relevant to our
  Haiku/Sonnet tier: irrelevant map content "will often distract or confuse the LLM",
  and weaker models sometimes try to edit code *in the map*.
- **Agentless (FSE 2025)** is the one strong measured pre-injection datapoint: an
  AST-derived signature skeleton **beat full file content for edit-site localization**
  (58.3% vs 53.7% ground-truth containment) while being far smaller, and precomputed-tree
  prompting beat embedding retrieval for file localization (78.7% vs 70.3%). Caveats: a
  fixed pipeline (not an agent that already has grep/Read), Python, 2024-era models. The
  cell we care about — *adding* a deterministic skeleton to an already-agentic worker —
  is unmeasured industry-wide.
- **Null-hypothesis counter-evidence**: Anthropic's SWE-bench-Verified SoTA scaffold
  used ONLY bash+edit tools; Claude Code and Cline both dropped indexes for agentic
  search. But those compare against embeddings-RAG on *quality*, never against
  deterministic cached-prefix maps on *tokens* — nobody has published our cell.
- **llms.txt**: unread in practice (a 137k-site crawl study found ~97% of llms.txt
  files get zero traffic; no major platform committed). Not our slot anyway — sparq's
  mechanically-loaded equivalent is `AGENTS.md`/`CLAUDE.md` (§1.2).
- **Embedding/vector code-index MCP servers**: triple mismatch for this fleet —
  non-deterministic (no byte-identical artifact at a SHA, no prompt-cache leverage),
  per-run index build/sync or a hosted DB dependency for one-shot Actions workers, and
  *agent-invoked exploration tooling* — the category falsified three times here. The
  only quantified claim in that space is vendor-published. Rejected without a pilot.

### 2.2 Rust-native deterministic artifact routes (feasibility measured locally)

- **rustdoc JSON** (nightly-only, `-Z unstable-options`): byte-deterministic at a
  pinned nightly, warm ~1–5s/crate, but a verbose intermediate (hundreds of KB per
  small crate) needing a distiller (`cargo-public-api`), and — critical for this
  workspace — a **default-features run missed every `#[cfg(feature=…)]`-gated item**
  in the probe crate. The opt-in-feature architecture makes gated items exactly the
  interesting surface; `--all-features` has its own known rustdoc traps here. Format
  version churns across nightlies (pin it).
- **Grep/awk signature-card prototype** (~90-line generator, zero deps): per-crate
  cards with one-line pub fn/struct/enum/trait signatures + `[feature=…]` tags +
  workspace deps + test entry points; **byte-identical across runs** (sha256-verified,
  `LC_ALL=C` sorted walk), 0.05–0.25s/crate. Probe sizes: sparq-core ≈4.1k tok,
  sparq-hdt ≈0.6k, sparq-terse ≈0.55k. Known blind spots: methods lose their
  `impl` context, macro-generated API invisible, silent under-reporting on syntax
  drift (mitigate with a weekly item-count canary vs rustdoc JSON on one crate).
- **Sizing verdict (honest)**: the full 61-crate card set is ≈138k tokens; even a
  names-only floor is ≈21k (6,537 public items × a few tokens each). **A full-workspace
  map in the shared prefix is dead on arrival.** The only viable shape is **scoped
  per-issue injection** (target crate + direct workspace deps, ~0.5–10k tok) into the
  brief — which the harness already supports via the per-issue `packages` scope.
- **Existing artifacts are not cards**: crate READMEs are template-capped narrative
  (the probe README names ~8 of 20 public items, no signatures); `skills/*/SKILL.md`
  are per-surface (~13–36k tok each) not per-crate. Complements, not substitutes — do
  not bend the template-gated READMEs into cards.

### 2.3 What auto-caching already does (and the measurement gap)

Both CLIs re-send the whole transcript per tool call; the unchanged prefix re-prices at
~0.1× (Anthropic) / ~0.1× (OpenAI cached) automatically — the dominant intra-run
economics need no engineering. What is missing is **visibility**: the harness captures
plain text only, so the fleet has zero telemetry on `cache_read_input_tokens` vs
`cache_creation_input_tokens` per run. Adding `--output-format json` to the claude
invocation (and parsing codex usage output) with the **host extracting only the
usage/cost fields** — no content — is compatible with the never-read-transcripts policy
and is the prerequisite for every pilot in §5. A read:write ratio below ~70% flags
something silently varying the prefix mid-run.

## 3. Ranked recommendations

Prerequisite **P0 — per-run usage telemetry** (§2.3): not a saver, the *measurement
enabler*. Ship first; nothing below is decidable without it.

Each entry: **mechanism** (cache-hit / fewer-reads / cheaper-model / shorter-prefix),
expected magnitude (estimate) + evidence quality, how it differs from the three
falsified approaches, and cache key / invalidation.

### R1 — Verify, then fix, the PreToolUse Opus agent-hook leak in headless workers

- **Mechanism**: cheaper-model, inverted — removing an expensive-model round-trip that
  guards Bash calls which can never be in scope in-container (§1.5).
- **Expected**: if the hook fires headless, ~100–400k Opus input tokens per
  implementation worker (estimate) — larger than every other lever combined; if it
  doesn't fire, zero. Hence **verify-first** (pilot E), fix either way (the fix also
  hardens against future CLI behavior changes).
- **Evidence quality**: hook config verified by direct inspection of the committed
  settings; firing behavior in fresh-HOME `claude -p` **unverified**.
- **Vs the falsifications**: not a prompt technique at all — overhead removal; no new
  tooling enters the loop.
- **Cache key/invalidation**: n/a. Fix = scope the matcher via a cheap command-hook
  prefilter (only escalate on the arming pattern) or ship a worker-mode settings
  override in the container image.

### R2 — Worker-tier AGENTS.md (≤32 KiB worker-relevant core) + fix codex truncation

- **Mechanism**: fewer-reads + shorter-prefix (a precomputed, deterministic, tracked
  artifact — category (b)).
- **Expected**: ~27k tok saved per compliant claude worker per run (estimate: 36k Read
  → ~8.4k tiered read, §1.2), fleet-wide; AND the codex silent-truncation fix. The
  truncation fix is a **correctness change needing no A/B** (rules after byte 32,768
  are currently invisible to codex workers); the token claim is gated on pilot D.
- **Evidence quality**: byte/section measurement local + codex `project_doc_max_bytes`
  documented; the token saving is unmeasured (and the ast-grep lesson warns a leaner
  prefix can *force* compensating exploration reads — the A/B must judge full-run
  effective tokens, not file size).
- **Vs the falsifications**: the terse falsification compressed *syntax*; this removes
  *content the brief already overrides* (workers are forbidden from most of what the
  orchestrator sections describe). Risk retained: a worker occasionally needs an
  orchestrator-section fact (e.g. CI-shape rules affecting code it writes) — the tier
  must keep those.
- **Cache key/invalidation**: it's a tracked file — invalidation is git itself; the
  tier must be maintained alongside `AGENTS.md` (same-PR discipline, like the
  public-API → SKILL.md rule).

### R3 — Scoped per-crate deterministic API card injected into the brief

- **Mechanism**: fewer-reads (precomputed prefix artifact substituting for exploratory
  Reads of `lib.rs`/module files during localization).
- **Expected**: breaks even if it prevents roughly ONE exploratory Read or one
  wrong-crate detour per run (a single file Read is typically 5–50k uncached tokens
  that then inflate every later turn's prefix — estimate). Upside per the Agentless
  ablation is better localization; downside per the aider FAQ is distraction on
  familiar crates. **Both outcomes are live** — that's what pilot B decides.
- **Evidence quality**: strongest published adjacent evidence in the survey (Agentless
  FSE 2025 ablations) + local feasibility measured (§2.2); in-fleet effect unmeasured.
  Structurally matches the one measured winner (a precomputed deterministic answer
  replacing agent exploration, cf. pkg-query) — but the analogy is not evidence.
- **Vs the falsifications**: all three losses were **agent-invoked** at answer time;
  this is **precomputed at the SHA, injected in the prefix, deterministic** — the
  unmeasured cell the falsifications never touched. Scoped-only: the full-workspace
  variant is rejected on sizing alone (§2.2).
- **Cache key/invalidation**: grep-route card key = hash(per-crate `git rev-parse
  <SHA>:crates/<crate>` tree OID ‖ root `Cargo.toml` OID) — constant-time, exact;
  regenerate only touched crates per SHA. A rustdoc-route card would additionally need
  `Cargo.lock` OID + the pinned nightly version. Generator-drift is the real staleness
  risk (silent under-reporting) — add the item-count canary.

### R4 — Prefix-stability ordering fixes in the worker harness

- **Mechanism**: cache-hit (make everything before the issue tail byte-identical
  across a batch): move `git switch -c` to after the model run; move `{scope}` to the
  brief tail; optionally adopt `--exclude-dynamic-system-prompt-sections`.
- **Expected**: bounded — ~14–23k token-equivalents saved per *additional*
  same-account worker per batch (estimate, §1.4). Free hygiene with zero behavior
  change (the model never commits), but honestly a cheap enabler, not a headline.
- **Evidence quality**: harness code verified by inspection; the exact prefix
  divergence point is inferred from prompt layout, so pilot A includes a one-request
  token-counting canary before claiming the full shared length.
- **Vs the falsifications**: no new content, no new tooling — pure byte ordering.
- **Cache key/invalidation**: the prefix bytes themselves; invalidated by any role-file
  / CLAUDE.md / CLI-version / model / tool-list change (all already batch-constant).

### R5 — Account-affinity routing + wave batching inside the (1-hour) cache TTL

- **Mechanism**: cache-hit (route same-(role, model) work to the account that last ran
  that shape within the TTL; stagger the first worker so the write lands before the
  rest fire).
- **Expected**: bounded by R4's ceiling and only pays *after* R4 (today the branch-name
  divergence defeats it). The 1h subscription TTL (§1.4) makes the window practical.
- **Evidence quality**: provider cache-scoping and TTL documented; the registry's
  affinity tie-break already exists in selection code but the `cache-affinity.json`
  store is a dead stub (zero readers/writers) and lease history is erased on release —
  re-key on (model, role, harness) and either write the store at claim time or promote
  the lease-derived signal within a TTL window.
- **Vs the falsifications**: pure routing; no prompt content changes.
- **Cache key/invalidation**: (account/org, model, byte-exact shared prefix) within
  TTL; refreshed on read. **Tension to respect**: affinity concentration trades against
  expiry-priority usage routing and risks per-account session caps — never promote
  affinity above usage-eligibility.

### R6 — Extend the measured winner: cheap-model delegation of the verbose middle

- **Mechanism**: cheaper-model — the only lever with an in-repo real-token win at
  scale (Haiku NL-tool / pkg-query, §0). The per-run cost pools that cross-run caching
  cannot touch (exploration reads, build/test output digestion) are exactly the pools
  this attacks.
- **Expected/evidence**: already measured (N=30, adopt verdict) for the PKG question
  class; extension to new question classes (e.g. build-log triage) needs its own §5-style
  A/B each time.
- **Vs the falsifications**: it IS the proven pattern the falsifications sharpened.
- **Cache key/invalidation**: n/a (delegation, not an artifact).

### Anti-recommendations (do not pilot)

- Embedding/vector code-index MCP servers (§2.1) — non-deterministic, dependency-heavy,
  agent-invoked: triple mismatch.
- `llms.txt` — unread in practice; our mechanically-loaded slot already exists.
- Full-workspace map/card set in the shared prefix — rejected on measured sizing (§2.2).
- Any terse/compression syntax layer — already falsified at real fidelity.
- Speculative tiering/summarizing of the `skills/` tree — per-skill reads are
  agent-invoked exploration (the measured-loser category); only the deterministic
  auto-load/brief-path artifacts qualify for prefix treatment.
- A dedicated cache pre-warmer request — with a 1h refresh-on-read TTL and wave
  cadence, a warmer is a pure extra write.

## 4. What this changes about the three falsifications — and what it does not

The falsifications stand untouched: agent-invoked exploration tooling remains the
measured loser, and nothing here re-opens it. The novel claim of this record is
narrower: the **precomputed deterministic prefix artifact** cell (R2/R3) was never
tested by those A/Bs, has one strong published adjacent result (Agentless) and one
documented failure mode (aider distraction), and is cheap to test properly on this
fleet once P0 telemetry exists. The cache-hit levers (R4/R5) and the leak fix (R1) are
not token-saving *techniques* at all — they are harness hygiene with bounded, largely
arithmetic upside, still verified by the same telemetry.

## 5. Pilot measurement plan (pre-registered)

**Telemetry (P0, prerequisite).** Add `--output-format json` to the claude invocation
(parse codex usage output equivalently); the **host extracts usage fields only** —
`input_tokens`, `cache_creation_input_tokens`, `cache_read_input_tokens`,
`output_tokens`, cost, turn count, Read/Bash tool-call counts — never transcript
content (policy-compatible; same telemetry class as the pkg-dogfood A/B). Definitions,
fixed up front:

- **Effective input tokens** = `input + 1.25×cache_creation + 0.1×cache_read`
  (Anthropic list-price weights; codex analog uses its cached-input discount).
- **Primary metric** = provider-price-weighted cost per run at the run's model list
  prices (matches `bench/pkg-dogfood` precedent). Secondary: effective input tokens,
  Read count, turn count. **Quality gate on every pilot**: PR gate pass-rate + review
  verdict rate must be non-inferior (arm − control ≥ −5 percentage points).
- **Design**: N ≥ 20 runs per arm (N=30 preferred, matching in-repo precedent), paired
  by issue where feasible (same issue, both arms, same `origin/main` SHA), dispatch
  order randomized. **Anti-contamination**: arms must not share an account within the
  cache TTL (a warm arm-A prefix must not subsidize arm B); record account + dispatch
  timestamps so cache state is auditable.
- **Decision rule**: adopt only on threshold met AND quality non-inferior; a reversal
  (arm worse) is recorded in `bench/` like the prior falsifications. No char proxies.

**Pilot A — prefix reordering + identical shared brief (cache-hit; R4).**
Arm = harness with post-model branch creation + tail-`{scope}`; control = current
harness. Same-account batches of ≥3 workers, first worker staggered ~30–60s.
*Mechanism check first*: workers 2..N must show turn-1 `cache_read_input_tokens` > 0
spanning the shared region (plus a one-request token-counting canary to locate the
divergence point). **Success threshold: ≥5% median reduction in price-weighted cost per
batch** (bounded lever, low bar, near-zero risk); adopt on mechanism-check pass even at
lower savings if cost is non-inferior, since it is also the R5 prerequisite.

**Pilot B — precomputed per-crate API card in the shared prefix vs control
(fewer-reads; R3).** Arm = brief with the target crate('s) card(s) injected at a
byte-stable position before the issue tail; control = current brief. Cards generated at
the batch SHA by the deterministic generator (§2.2 grep route), scoped to target crate
+ direct workspace deps, hard cap ~5k tok. **Success threshold: ≥10% median reduction
in price-weighted cost, OR median Read-tool count reduced by ≥1 with cost
non-inferior — with the quality gate.** Pre-registered failure call: if arm cost is
≥5% WORSE, record the distraction effect as falsification #4 and stop. Stratify
reporting by whether the issue's crate is "familiar" (recently touched by the fleet) —
the aider FAQ predicts the loss concentrates there.

**Pilot C — account-affinity + wave batching inside the TTL (cache-hit; R5).**
Requires pilot A's arm. Arm = same-(role, model) waves routed to one account within
the 1h TTL, first-fire staggered; control = current expiry-priority scatter.
*Mechanism check*: batch-level `cache_read` share rises vs control. **Success
threshold: ≥5% median reduction in price-weighted cost per worker AND zero increase in
session-cap incidents** (capped-account dead workers are a known failure mode —
affinity must never outrank usage-eligibility).

**Pilot D — worker-tier AGENTS.md (fewer-reads/shorter-prefix; R2).** Arm = brief
points workers at the ≤32 KiB worker tier; control = status quo. **Success threshold:
≥15% median reduction in price-weighted cost with the quality gate** (the bar is
higher because the risk — a worker missing a needed rule — is a correctness risk, so
the win must be clearly worth it). The codex truncation **correctness fix ships
regardless of this pilot's outcome** (restructure so worker-relevant rules precede the
32 KiB boundary, or raise `project_doc_max_bytes` in the worker config); only the
token claim waits.

**Pilot E — hook-firing canary (verify-first; R1).** One-off, before everything
except P0: stub repo + a counting command hook + the committed agent hook;
`claude -p --output-format json` a few Bash-heavy tasks; diff usage with/without the
agent hook present in settings. If agent hooks fire headless: fix immediately (no
threshold — it is pure overhead); re-run one baseline batch afterwards so pilots A–D
measure against the fixed floor.

**Sequencing**: P0 → E → A → {B, D in parallel, separate accounts} → C. Every pilot's
results land in `bench/` (the sanctioned home for measured numbers) with a one-line
qualitative pointer here; adoption phases become beads on acceptance.

## 6. Cross-references

- Falsification + win records: [`bench/pkg-dogfood/RESULTS.md`](../bench/pkg-dogfood/RESULTS.md),
  [`bench/pkg-dogfood/RESULTS-astgrep.md`](../bench/pkg-dogfood/RESULTS-astgrep.md),
  [`bench/terse/RESULTS.md`](../bench/terse/RESULTS.md).
- Measurement protocol precedent: [`agent-effectiveness-program.md`](agent-effectiveness-program.md)
  (§5 shared protocol), [`dogfooding-sparq-knowledge-graph.md`](dogfooding-sparq-knowledge-graph.md).
- Interactive-era predecessor: [`agent-efficiency-tooling.md`](agent-efficiency-tooling.md)
  (its cache-hygiene ranking anticipated §1.4; its AGENTS.md sizing predates the file's
  growth).
- Fleet architecture: [`orchestration-private-registry-worker.md`](orchestration-private-registry-worker.md).
- External: Agentless (FSE 2025, arXiv:2407.01489); aider repo-map docs + FAQ;
  Anthropic prompt-caching + Claude Code caching docs; OpenAI prompt-caching guide;
  codex `project_doc_max_bytes` docs.
