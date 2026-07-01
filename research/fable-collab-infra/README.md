<!-- [OPUS-4.8] apply-guide for the Fable collaboration tier drafts. Research doc, not a crate README — repo readme-template rules do not apply here. -->
# Fable collaboration tier — apply guide

**`[OPUS-4.8]`** This directory holds the **DRAFT** infrastructure for the **Fable collaboration tier**:
a token-efficiency operating model where **Claude Fable** (an expensive, stronger model) acts as
**architect + reviewer** while a cheap fleet — **Opus 4.8 / Sonnet 4.6 / Haiku 4.5** — does the
mechanical work. The design is `research/fable-work-plan.md` **§6** (agent roster + workflow catalogue)
and **§8** (context hygiene / the compaction observer). Tracked by beads **`sq-sgu1.1`** (agents) and
**`sq-sgu1.2`** (workflows).

These are drafts staged for the maintainer to apply. This PR does **not** open a merge-train PR, does
**not** commit, and touches **no** protected file — it only lays down the drafts + this guide.

## What is here

| Path | What it is |
|---|---|
| `agents/*.md` | Five **agent-config drafts** for the tier (see roster below). Each begins with a DRAFT HTML comment because `.claude/agents/` is **Self-Modification-protected** (AGENTS.md **rule 11**) and an agent may not write into it. |
| `README.md` | This apply guide. |

The **three Fable-tier workflow scripts** are **not** staged here — `.claude/workflows/` is **not**
protected, so they ship directly into it (see Apply step 2).

## Apply steps

**1. Move the agent configs into place.** `.claude/agents/` is protected, so the drafts are staged
here for a **maintainer-applied** move:

```
git mv research/fable-collab-infra/agents/sparq-architect.md         .claude/agents/
git mv research/fable-collab-infra/agents/sparq-reviewer.md          .claude/agents/
git mv research/fable-collab-infra/agents/sparq-verify-mechanical.md .claude/agents/
git mv research/fable-collab-infra/agents/sparq-rust-impl.md         .claude/agents/
git mv research/fable-collab-infra/agents/sparq-context-monitor.md   .claude/agents/
```

Then **strip the leading `<!-- DRAFT … -->` comment** from each file — it is a staging marker, not
part of the config. Filename `==` the frontmatter `name` value + `.md` (house convention), so no
rename is needed.

**2. The workflows are already runnable.** The three scripts —
**`fable-architect-drain`** (flagship), **`fable-soundness-verdict`**, and **`fable-lens-review`** —
ship directly in `.claude/workflows/` (that path is not protected), so **no move is needed**. Each is
invokable **now** via the **Workflow tool by name** (its exported `meta.name`), exactly like
`autonomous-scheduler`. Note: **`model-tiered-scheduler-v2`** in the catalogue is *not* a fourth new
script — it is the planned **evolution of `autonomous-scheduler.js`** (tiered `pickAgent` + a split
Haiku-mechanical → Fable-escalated verify stage) and lands as an edit to that file, not a fresh one.

## The model roster (`research/fable-work-plan.md` §6.2)

| Agent | Model | Purpose (one line) |
|---|---|---|
| **`sparq-architect`** | **fable** | FRONT decompose stage: one hard epic → **exactly one** `research/` design record **+** N **disjoint**, spec'd child beads `{crate, model_tier, invariant, acceptance_test}`. Reads + decides + specs; does **not** implement. Supersedes `sparq-researcher` for the decompose-and-spec job. |
| **`sparq-reviewer`** | **fable** | ESCALATED-tier final **verdict-giver**; fires only on the escalated subset (failed mechanical-verify, or a ZK/MPC/reasoner/engine-correctness/novel-algo/honesty surface). Read-only; returns `{honest, sound_as_scoped, recommend_arm, disposition, concerns}`. Rare `disposition=fable_implements` authors the fix in a scoped isolated worktree. |
| **`sparq-verify-mechanical`** | **haiku** | Cheap **objective** verify checklist (gates green in both feature states via `ci-summary`, non-vacuous tests, opt-in respected, README/`SKILL.md` synced, no hard-coded perf, coverage floor held). **Arms** clean low-risk PRs, **escalates** the rest. The filter that keeps Fable's input tiny. |
| **`sparq-rust-impl`** | **sonnet** | Bulk Rust implementer for **well-spec'd, disjoint, single-crate** beads the architect de-risked (spec **+** failing acceptance test). Same HARD gates as `sparq-rust-feature`; **escalates back up** (`needs_architect=true`) if the bead turns out hard, cross-crate, or underspecified. |
| **`sparq-context-monitor`** | **haiku** | Out-of-band **context-hygiene** observer (§8): reads the live session `.jsonl` and emits `{should_compact, confidence, reason, what_to_preserve, externalize_first}`. **SIGNALS ONLY** — never edits, commits, or forces `/compact`. |

Model mapping (§6): **fable** for architecture + escalated review (scarce, ~1 call/epic and
escalated-diffs-only); **sonnet** for cheap bulk impl; **haiku** for the two continuous cheap watchers
(mechanical verify + context monitor).

## The provenance-trailer retrofit (honest caveat)

The repo currently **hard-codes** the Opus-4.8 attribution literals — the inline `[OPUS-4.8]` marker and
the `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>` commit trailer — across the
existing agent configs and the workflow prompt strings. Once the fleet is **multi-model**, those
literals **MISLABEL** Fable / Sonnet / Haiku diffs as Opus. That is a real honesty defect, not cosmetic.

This PR does **NOT** retrofit the whole repo. The correct fix is a **model-aware, per-stage** trailer
(parameterize the model name/version instead of hard-coding Opus 4.8), but most of the affected files are
the **protected** `.claude/agents/*.md` configs (AGENTS.md rule 11) and the maintainer-owned workflow
prompts — an agent may not rewrite them. So the scope taken here is narrow and honest:

- The **five new drafts** are made **model-aware**: `sparq-architect` / `sparq-reviewer` tag `[FABLE]` +
  a `Claude Fable` trailer; `sparq-rust-impl` tags `[SONNET-4.6]` + a `Claude Sonnet 4.6` trailer; the
  two haiku watchers commit nothing, so only their **authored notes** carry `[OPUS-4.8]` (they were
  drafted by Opus while Fable was unavailable — flag for re-review when Fable returns).
- The **repo-wide retrofit** (make the AGENTS.md § *sub-agent shared contract* trailer rule and the
  existing configs/workflow prompts model-aware) is left as tracked work under bead **`sq-sgu1.1`**.

## Open confirmations (maintainer)

Two things need a maintainer check before the tier runs live — both are honest unknowns, not defects:

1. **Is the `model:` frontmatter field honored for the new tiers?** Only `opus` and `haiku` are
   **observed** as live `model:` values today (`sparq-perf-reviewer`/`sparq-rust-feature` = `opus`,
   `sparq-pkg-nl` = `haiku`). The new drafts introduce the bare aliases **`fable`** and **`sonnet`**.
   Confirm the harness / `pickAgent` dispatch **resolves** these aliases; until confirmed, an unresolved
   alias **silently falls back to the default model** (a Fable stage would run cheap, a Sonnet stage
   could run at the wrong tier). Each draft already flags this inline for re-review.
2. **Do the `agentType` refs in the workflows resolve after the configs are applied?** The three
   workflow scripts dispatch by **agent name** (`agentType: 'sparq-architect'`, `'sparq-reviewer'`,
   `'sparq-verify-mechanical'`, `'sparq-rust-impl'`). Those names resolve only **after** Apply step 1
   moves the configs into `.claude/agents/`. Run one workflow after applying and confirm no
   agent-not-found fallthrough.

**`[OPUS-4.8]`** Drafted by Opus 4.8 while Fable was unavailable; flag the whole tier for re-review when
Fable returns.
