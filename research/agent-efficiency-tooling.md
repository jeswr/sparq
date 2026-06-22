# Agent-efficiency tooling — cutting the token/$ cost of parallel coding agents [OPUS-4.8]

> 🤖 SPARQ agent — design-for-maintainer-review. No implementation here; this is a
> survey + ranked recommendation + phased plan (each phase a future bead).

Status: **design-only / proposed**. Nothing in this doc is adopted yet.

## 0. What this is, and a correction to the brief's premise

The brief asked for tooling that **reduces the cost (tokens / $) of our parallel coding
agents without reducing performance**, flagging two areas the maintainer believed exist:
(a) **structured / persistent context storage** so agents don't re-derive, and
(b) **codebase indexing / retrieval** so agents find code with fewer tokens than broad
file reads.

Both areas are real and have shipping tooling. But **one premise needs correcting up
front**, because it changes the ranking:

> **Correction — the dominant cost lever for THIS workflow is not a memory or index
> product; it is prompt-cache hygiene plus brief discipline.** Two measured facts drive
> this: (1) Claude Code already does automatic prompt caching, and a cache *read* costs
> **0.1× (10%)** of base input while a cache *write* costs **1.25×** (5-min) or **2×**
> (1-hour) — so the biggest single win is arranging our parallel agents so they pay the
> 10% read rate instead of re-paying full input; and (2) each **sub-agent and each
> worktree gets its OWN cache** (see §2), so our fan-out pattern is *systematically*
> missing cache it could hit. A code-index MCP server is a real but **second-order**
> lever, and it carries a per-turn tool-definition tax that can erase its own savings on
> small tasks. The honest ranking below puts cache hygiene first.

This is consistent with the maintainer's own recorded feedback (MEMORY.md:
`feedback-subagent-delegation`, `feedback-background-dispatch`) that *agent discipline*
(smallest context-independent briefs) is the large lever — and with the one **independent,
peer-style measurement** I found that bears directly on us (§3): a real study of
`AGENTS.md`-style context files measured a **16.58% reduction in output tokens** and a
**28.64% reduction in runtime** — i.e. our existing `AGENTS.md` investment is *already* a
measured efficiency win, not overhead.

### Repo facts this doc is grounded in (measured locally on this checkout)

| Fact | Value | Why it matters |
| --- | --- | --- |
| `AGENTS.md` size | ~64.5 KB, ~16 K tokens (chars/4 estimate) | Loaded into the "project context" cache layer once per session — cache-friendly, but it is the prefix every sub-agent re-establishes |
| Tracked files | 2 958 | Broad `grep`/`ls` fan-out is expensive at this size |
| Rust crates | 30 (`sparq-core` … `sparq-zk-compose`) | A symbol index that spans the whole workspace in one pass is attractive |
| Rust source files | 405 `.rs` | rust-analyzer over a 30-crate workspace has real first-index latency (§4) |
| TS/TSX files | 71 (`site/`) | Small; broad reads are cheap here — indexing buys little on the site |
| Sum of `.claude/agents/*.md` briefs | ~67.5 KB | Re-sent per dispatched agent; trimming these is free token savings |

> **Honesty note on numbers.** Token counts marked "chars/4" are estimates, not a real
> tokenizer count. All external benchmark figures below are tagged **[independent]**,
> **[vendor/author self-report]**, or **[anecdotal]**; do not treat a self-report as
> validation. Per the EC2-execution-env rule, no work-box timings appear here as
> canonical.

---

## 1. The actual cost model (so we can rank honestly)

Every Claude Code turn re-sends the whole context; prompt caching reuses the unchanged
**prefix** by exact match. Pricing (canonical, from the Claude API prompt-caching docs):

- **Cache read (hit):** 0.1× base input (~10%).
- **Cache write:** 1.25× base input (5-min TTL) or 2× (1-hour TTL).
- **Min cacheable prefix:** 1 024 tokens for Opus 4.8 / Sonnet 4.x / Haiku 4.5 (varies by
  model; 2 048 for Opus 4.7, 4 096 for Opus 4.5/4.6).
- **TTL:** 5 min default; 1 hour opt-in. Each cache hit **resets the timer**. On a Claude
  subscription Claude Code requests the **1-hour TTL automatically** for the *main*
  conversation (no extra charge on-plan); **sub-agents use the 5-min TTL even on a
  subscription**.

Three structural consequences for us:

1. **Cache, not memory, is where the bulk tokens live.** A 16 K-token `AGENTS.md` prefix
   served from cache costs ~1.6 K-token-equivalents; re-written it costs ~20 K. The
   difference between a cache hit and a cache miss on our standard prefix dwarfs anything a
   memory product saves per query.
2. **Our fan-out pattern fights the cache by construction.** Cache scope is *per machine +
   per working directory*; **each git worktree has its own working directory and therefore
   its own cache** (the docs say this explicitly: "two sessions in different directories …
   miss each other's cache. That includes worktrees of the same repository"). Our charter
   mandates a separate worktree per mutating agent (`AGENTS.md` §"Worktree isolation"). So
   N parallel agents each pay a cold first turn and a 5-min-TTL sub-agent cache.
3. **MCP tool definitions live in the prefix and are not free.** Connecting an MCP server
   adds its tool schemas; if they load into the prefix (the case on some providers/models,
   or for `alwaysLoad` servers) they invalidate cache on connect and re-bill every turn.
   The mitigation is **Tool Search / deferred tool loading** (§5), which moves them behind
   the cache breakpoint.

---

## 2. Anthropic prompt caching — does Claude Code use it for us automatically?

**Yes, automatically, and it is the highest-ROI lever** — but our usage pattern is leaving
savings on the table. Findings (all from the official Claude Code prompt-caching doc):

- Claude Code orders each request as **system prompt → project context (CLAUDE.md / auto
  memory) → conversation**, putting rarely-changing content first so the prefix matches.
  We benefit from this for free today.
- **Editing `AGENTS.md`/`CLAUDE.md` mid-session does not invalidate cache — but also does
  not apply** until `/clear`, `/compact`, or restart. (Relevant: our SessionStart hook
  injects bead state as `additionalContext`; that is fine, it is set once at session
  start.)
- **Sub-agents build their own cache from cold** (own system prompt + tool set), 5-min TTL.
  The parent's cache is untouched. A **fork** (not a sub-agent) inherits the parent prefix
  and reads the parent cache.
- **Worktrees miss each other's cache** (per-directory scope). Parallel sessions in the
  *same* directory share cache.
- **Cache-invalidating actions to avoid mid-task:** switching model, switching effort
  level, toggling fast mode, connecting/disconnecting an MCP server whose tools load into
  the prefix, `/compact`, upgrading Claude Code, resuming after an upgrade.
- The Agent SDK exposes a way to **suppress per-machine system-prompt sections to share
  cache across machines/processes** ("improve prompt caching across users and machines") —
  relevant if we ever drive fleets via the SDK rather than interactive worktrees.

**What we can actually do** (no product purchase, just configuration / discipline):

- Pin **model + effort at session start** and avoid mid-task switches (each switch is a
  full cache rebuild). Our `opusplan`-style toggling, if used, is a model switch each time.
- Keep `AGENTS.md` and per-agent briefs **stable within a session**; front-load the
  invariant shared brief so it sits in the cached prefix; put per-task variation last.
- Prefer **fewer, longer-lived agents** over many short cold-start sub-agents where the
  task allows — every sub-agent pays a cold first turn at the 5-min TTL.
- Consider **co-locating serial agents in one working directory** when isolation is not
  required (read-only research/review agents may share the main checkout per our charter),
  so they share a warm cache instead of each cutting a fresh worktree.
- For SDK-driven fleets (future), use the **shared-prefix system-prompt** option.

**Evidence quality:** the pricing, TTLs, scope rules, and sub-agent/worktree behaviour are
**[official Anthropic documentation]**, not third-party. The *magnitude* of savings for our
specific fleet is **unmeasured by us** — see the open question in §8.

---

## 3. Structured / persistent memory (area a) — is there something better than our files?

**Short answer: no, not for a solo-maintainer multi-agent workflow. Keep the file pattern;
it is the pragmatic best and is itself measured to help.**

We already run: `MEMORY.md` auto-memory, `AGENTS.md`/`CLAUDE.md` as durable knowledge, a
SessionStart hook injecting `bd` task state, and **beads** as a dependency-aware tracker.
Surveyed alternatives and the honest verdict:

| Tool | Plugs into Claude Code? | Evidence | Beats our file pattern? |
| --- | --- | --- | --- |
| **Anthropic Memory tool / Memory Stores** (beta) | Memory tool is **Messages-API only**, not the CC CLI; Memory Stores is **Managed Agents only** (~$0.08/session-hour) | Vendor pattern docs, **no isolating benchmark** [vendor] | **No** — it is a client-side wrapper around the same file ops we already do directly |
| **mem0 (+ MCP)** | Yes (cloud HTTP or self-hosted Qdrant+Ollama; CC plugin with lifecycle hooks) | LoCoMo ~90% **[vendor]**; one **[independent]** controlled coding-agent benchmark = **15–28%** token saving (1 author, modest) | **Maybe**, only if we *measure* a re-derivation problem first; self-host to avoid cloud cost |
| **@modelcontextprotocol/server-memory** (knowledge-graph) | Yes (MCP) | Structure documented; **no coding-agent token benchmark** | **No** — overkill; we don't need queryable entity relations |
| **Letta (MemGPT)** | Yes (MCP) | Terminal-Bench result conflates orchestration + memory | **No** — agent self-managed memory tiers don't match our stateless-session design |
| **Zep / Graphiti** | Yes (MCP) | Enterprise support-bot framing; no relevant benchmark | **No** — wrong product shape |
| **Basic Memory / Obsidian-MCP** | Yes (MCP) | "71.5×" claim **[anecdotal]**, not reproduced | **No** — this *is* our pattern with prettier visualisation |

**Load-bearing independent evidence (the one I'd actually cite to the maintainer):**
the study *"On the Impact of AGENTS.md Files on the Efficiency of AI Coding Agents"*
(Lulla, Mohsenimofidi, Galster, Zhang, Baltes, Treude; arXiv 2601.20404; 10 repos, 124 PRs)
measured **−16.58% output tokens** and **−28.64% median runtime** when an `AGENTS.md` was
present. **[independent]** — this validates the investment we already made and argues for
*sharpening* `AGENTS.md` and the per-agent briefs rather than bolting on a memory server.

**Verdict:** status quo wins. The only candidate worth a *measured trial* (not adoption) is
**self-hosted mem0**, and only if §8's measurement shows real re-derivation cost.

---

## 4. Codebase indexing / retrieval (area b) — index once, query cheaply

This is the area with the most products and the noisiest claims. The honest split:

### 4.1 Serena (LSP-backed semantic tools over MCP) — the strongest *integration* fit

- **What:** MCP server giving symbol-level tools (`find_symbol`, `find_referencing_symbols`,
  `insert_after_symbol`, `replace_symbol_body`, …) over **Language Server Protocol**, so
  the agent reads/edits at symbol granularity instead of dumping files. MIT, ~25 K stars,
  active (v1.5.x, May 2026), maintained by the `oraios` org.
- **Claude Code integration:** Yes — runs as an MCP server (uvx / docker / HTTP). Rust and
  TypeScript both supported via their language servers.
- **Index model:** LSP-backed; **pre-indexing recommended for large projects**
  (`serena project index`) or "the first tool application may be very slow"; it then
  auto-updates on file change.
- **Evidence:** The best I found is **[independent]** but single-team: ManoMano's "Project
  Aegis" (Sonnet 4.5, Feb 2026, 36 K-LOC Java) — Claude+Serena **passed all 1 017 tests**
  on a refactor where vanilla Claude and Claude+built-in-LSP **failed**, at comparable $
  ($27.30 vs $23.54 / $28.63). **Crucial honesty point: this is a *quality/cost* win, not a
  token-reduction win** — Serena "read over 69 million tokens while keeping API cost
  contained" via **prompt caching**, not by reading fewer tokens. So Serena's value for us
  is *more reliable edits at similar cost*, which is adjacent to the brief but not the
  token-cut the brief asked for. Vendor tool-list framing claims fewer steps; that part is
  **[vendor]**.
- **Cost for THIS repo:** real. rust-analyzer over a **30-crate** workspace has meaningful
  first-index latency and a known **zombie-language-server** issue on client exit
  **[anecdotal but widely reported]**; we'd pre-index, manage the rust-analyzer process,
  and disable Serena's redundant tools to limit its prefix tool-definition tax.

### 4.2 codebase-memory-mcp (tree-sitter graph + lightweight type resolution)

- **What:** Single static binary, **zero runtime deps**, indexes a codebase into a SQLite
  knowledge graph (functions, classes, call chains, routes); 158 tree-sitter grammars +
  a "Hybrid LSP" C type-resolver for 9 languages **including Rust and TypeScript**; MIT,
  ~4 K stars, v0.8.1 (Jun 2026).
- **Claude Code integration:** Yes — installs as an MCP server (it can auto-configure CC).
- **Evidence — read carefully:** the headline "**10× fewer tokens, 83% answer quality vs
  92%, 2.1× fewer tool calls across 31 repos**" comes from arXiv **2603.27277**
  (*"Codebase-Memory: Tree-Sitter-Based Knowledge Graphs for LLM Code Exploration via MCP"*).
  **This preprint is authored by the tool's own authors and is not peer-reviewed — it is a
  [vendor/author self-report], NOT independent validation.** The repo's "**99.2% / 120×**"
  figure is a single self-measured scenario **[vendor self-report]**. The numbers are
  internally consistent and the codebase is real, but we must not present them as proven.
  Note the honest caveat *they* report: **a ~9-point quality drop (83% vs 92%)** — i.e. a
  real "without reducing performance?" risk.
- **Cost for THIS repo:** **lowest of the index options** — one static binary, milliseconds
  to re-index, one pass over all 30 crates. Attractive *if* a measured trial confirms the
  token saving and the quality drop is acceptable for our task mix.

### 4.3 The rest of the index landscape (honest one-liners)

| Tool | CC integration today? | Token evidence | Verdict for us |
| --- | --- | --- | --- |
| **aider repo-map** (tree-sitter + PageRank) | **No** — internal to aider; a standalone reimpl (`RepoMapper`) has an MCP, ~181 stars, early | **[anecdotal]** only | Skip — concept is good, no maintained CC-native path |
| **universal-ctags + `ctags-mcp`** | Yes (`ctags-mcp`, v1.0.0, small) | "~80%" **[anecdotal/unsourced]**; ctags itself is 40-yr-proven | Cheap fallback; definitions-only (no refs/types) |
| **SCIP** (`rust-analyzer scip`, `scip-typescript`) | **No consumer MCP** — SCIP is an IDE *format*, not an LLM layer | N/A (not positioned for token cut) | Skip unless we build an adapter — not worth it |
| **Sourcegraph MCP** (+ Cody→Amp lineage) | Yes (GA), but needs a **Sourcegraph instance** (cloud ~$/mo or self-host) | **No token measurement**; markets precision | Skip — adds cloud cost + ops; misaligned with cost-cutting |
| **tree-sitter-analyzer** | Yes (MCP) | Strong **call-graph accuracy**, but **no token benchmark** (TOON-format efficiency is a design claim) | Watch; not a measured token win |
| **mcp-server-tree-sitter** | Yes (MCP) | None | Skip — too thin |

**Net for area (b):** the only two worth our time are **Serena** (mature, best integration,
but its win is *edit quality at similar cost*, not fewer tokens) and **codebase-memory-mcp**
(cheapest to run, biggest *claimed* token cut, but **self-reported** + a measured quality
drop). Neither is a free lunch; both must be **measured on our repo before adoption**.

---

## 5. Tool Search / deferred tool loading — the cheap, official, do-it-now lever

Independent of any third-party product, Claude Code supports **Tool Search Tool / deferred
tool loading**: tool definitions are made discoverable on demand instead of loaded into the
prefix. Anthropic's published figures **[vendor, but first-party and specific]**: input
tokens drop **58% at 96 tools, 84% at 251, 92% at 508**, pass-rate held at 100%, and MCP-eval
accuracy *improved* (e.g. Opus 4.5 79.5%→88.1%). Claude Code auto-enables deferral when
deferrable tool definitions exceed ~10% of the context window.

**Why this matters for us specifically:** *if* we adopt any MCP code-index server (§4), its
tool schemas would otherwise sit in the prefix and re-bill every turn across every parallel
agent. Deferred loading keeps them behind the cache breakpoint, so a server connecting or
changing its tool list **only appends** and doesn't invalidate the cached prefix. This is the
mitigation that makes an index-MCP affordable in a fan-out workflow. **Caveat:** deferral is
unavailable/limited on Haiku, on Vertex AI, and behind a custom `ANTHROPIC_BASE_URL` gateway
— there, MCP tools load into the prefix and the tax returns.

---

## 6. Ranked shortlist — what would actually cut our parallel-agent cost

Ranked by **(measured-ROI × integration-cleanliness-today × honesty-of-evidence)**:

1. **Prompt-cache hygiene + brief discipline (CONFIG/PROCESS, no product).** Highest ROI,
   zero cost, evidence is first-party-official + one independent `AGENTS.md` study. Pin
   model/effort per session; stabilise the shared prefix; sharpen `AGENTS.md` and trim the
   ~67 KB of agent briefs; prefer warm-cache co-located serial agents where isolation isn't
   required; for SDK fleets, share the system-prefix.
2. **Tool Search / deferred tool loading (CONFIG, official).** Make this a *precondition*
   for adopting any MCP server; first-party 58–92% tool-token reduction with held pass-rate.
3. **Serena (MCP), measured trial.** Best-integrated, most mature index option; strongest
   *independent* evidence — but the win is **edit quality at similar cost**, not a token
   cut. Adopt only behind deferred loading + after a trial on our 30-crate workspace.
4. **codebase-memory-mcp (MCP), measured trial.** Cheapest to run, largest *claimed* token
   cut — but **self-reported** and with a measured ~9-pt quality drop. Trial head-to-head
   vs. plain Read/Grep on a fixed task set; adopt only if the token cut is real *and*
   quality holds for our tasks.
5. **self-hosted mem0 (MCP), conditional.** Only if §8 measurement shows real
   re-derivation cost; one independent benchmark = a modest 15–28%.

## 7. Honest "skip / not worth it / immature / vaporware" list

- **Anthropic Memory tool / Memory Stores** for *our* use — not wired into the CC CLI;
  Managed-Agents-only and metered; no advantage over our file pattern.
- **@modelcontextprotocol/server-memory, Letta, Zep, Basic-Memory/Obsidian-MCP** — wrong
  shape for a solo-maintainer fan-out; no relevant token evidence; the "71.5×" claim is
  uncorroborated.
- **SCIP / LSIF as an agent tool** — IDE format, **no consumer MCP**; not worth building an
  adapter.
- **Sourcegraph MCP / Cody(→Amp)** — needs a Sourcegraph instance (cloud cost/ops), markets
  precision not token savings; counter to a cost-cutting goal. (Note: individual Cody is
  EOL; Cody→Amp lineage means product churn risk.)
- **aider repo-map** — not usable from Claude Code; the standalone `RepoMapper` MCP is
  immature.
- **"95–99% token reduction" optimizer MCPs** (e.g. token-optimizer-mcp, TokenSave) — claims
  apply to **cache hits on repeats**, which Claude Code's own caching already gives us;
  treat as **[anecdotal]** and redundant.
- **Trusting any single self-reported "10×/49×/120×" number** — including codebase-memory's
  own preprint — as proof. Measure on our repo or don't claim it.

## 8. Single highest-ROI recommendation

**Adopt prompt-cache hygiene + brief discipline first (free, official, measured-adjacent),
and gate every later step on our OWN before/after token measurement.** Concretely, the first
move is to *measure* our real fleet — turn on the `cache_read_input_tokens` /
`cache_creation_input_tokens` statusline (and the OTel exporter if we want org-wide numbers),
run a representative refill→verify wave, and see how much we are paying in cold sub-agent /
per-worktree cache writes. That single measurement tells us whether the cache lever (likely
large) or an index MCP (second-order) is where the money is — and it is the honest gate
before spending integration effort on Serena or codebase-memory-mcp.

## 9. Open questions for the maintainer

1. **Do we have any baseline token/$ telemetry per agent wave?** Without it, every %-saving
   below is someone else's number on someone else's repo. (Drives Phase 1.)
2. **Isolation vs. cache trade-off:** are read-only research/review agents *required* to use
   worktrees, or may they share the main checkout to ride a warm cache? The charter already
   permits read-only agents to share — confirming this unlocks free cache reuse.
3. **Privacy posture for index MCPs:** codebase-memory-mcp is fully local (good); but would
   you ever accept a server needing an external embedding API (e.g. claude-context →
   OpenAI + Milvus)? I'd recommend a **local-only** constraint given the ZK/crypto code; flag
   if that's a hard rule so we can prune candidates.
4. **Risk tolerance for the ~9-pt quality drop** reported for graph-index retrieval — is a
   token cut worth any measurable quality regression on engine-critical code, or is the bar
   "no regression"?

## 10. Phased plan (each phase = a future bead)

1. **Bead — Instrument agent-wave token cost (measurement, blocking).** Stand up the
   cache-token statusline + (optionally) the OTel exporter; capture `cache_read` vs
   `cache_write` vs `input` for one real refill→verify wave; record the per-worktree cold-start
   cost. *Output:* a baseline we own. *Depends on:* nothing.
2. **Bead — Cache-hygiene + brief-discipline pass (config/process).** Pin model/effort
   conventions; front-load the invariant shared brief into the cached prefix; trim the
   ~67 KB of `.claude/agents/*.md` and tighten `AGENTS.md`; document the
   worktree-vs-warm-cache rule for read-only agents. *Depends on:* Phase 1 (to measure the
   delta).
3. **Bead — Enable + verify Tool Search / deferred tool loading.** Confirm deferral is active
   on our model/provider; make "MCP tools must defer" a precondition in agent briefs.
   *Depends on:* Phase 1.
4. **Bead — Serena trial on the 30-crate workspace (spike).** Pre-index, measure first-index
   latency + zombie-process handling, disable redundant tools, run a fixed edit/refactor task
   set, compare $/tokens/quality vs. baseline. Adopt-or-reject verdict. *Depends on:* Phase 3.
5. **Bead — codebase-memory-mcp head-to-head trial (spike).** Index once; run the same fixed
   structural-question + edit task set vs. plain Read/Grep; verify the *claimed* token cut on
   our repo and whether the ~9-pt quality drop materialises. Adopt-or-reject. *Depends on:*
   Phase 3. (Parallel to Phase 4.)
6. **Bead — (conditional) self-hosted mem0 trial.** Only if Phase 1 shows real
   re-derivation cost; A/B one agent branch self-hosted; expect ~15–28%, not 49×. *Depends
   on:* Phase 1 result.

---

### Sources

Official / first-party:

- Claude Code — *How Claude Code uses prompt caching* (scope, TTL, sub-agent/worktree
  behaviour): <https://code.claude.com/docs/en/prompt-caching>
- Claude API — *Prompt caching* (pricing multipliers, min-prefix, breakpoints):
  <https://platform.claude.com/docs/en/build-with-claude/prompt-caching>
- Anthropic — *Introducing advanced tool use* (Tool Search / deferred-loading figures):
  <https://www.anthropic.com/engineering/advanced-tool-use>
- Anthropic — *Memory tool* docs:
  <https://platform.claude.com/docs/en/agents-and-tools/tool-use/memory-tool>

Independent:

- Lulla et al., *On the Impact of AGENTS.md Files on the Efficiency of AI Coding Agents*,
  arXiv 2601.20404 (−16.58% output tokens, −28.64% runtime; 10 repos / 124 PRs).
- ManoMano Tech, *Project Aegis: Benchmarking AI agents and why Serena is our new must-have*
  (single-team, Sonnet 4.5, 36 K-LOC Java):
  <https://medium.com/manomano-tech/project-aegis-benchmarking-ai-agents-and-why-serena-is-our-new-must-have-311673db35dd>
- Markus Sandelin, *The first controlled benchmark of AI memory in coding agents* (15–28%):
  <https://medium.com/@mrsandelin/the-first-controlled-benchmark-of-ai-memory-in-coding-agents-8e0bb776d39e>

Vendor / author self-report (treat as claims, not validation):

- Serena (oraios): <https://github.com/oraios/serena>
- codebase-memory-mcp (DeusData) + preprint arXiv 2603.27277 (10×/83% self-reported):
  <https://github.com/DeusData/codebase-memory-mcp>
- claude-context (Zilliz; requires Milvus + embedding key; ~40% self-eval):
  <https://github.com/zilliztech/claude-context>
- mem0: <https://mem0.ai/blog/state-of-ai-agent-memory-2026>
