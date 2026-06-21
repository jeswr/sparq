---
name: proceed-and-document
description: The standing proceed-without-the-maintainers-greenlight procedure for any sparq agent that hits a design/judgment-decision bead. Use when a bead is blocked only on a maintainer design greenlight or a choice you would otherwise ask about (NOT an external credential/access block, and NOT an honesty/soundness label) — make the best-judgment choice, document it (PR body + a one-line bead note), open a short SPARQ-agent-self-id GitHub issue so the maintainer can steer post-hoc, and proceed. Referenced by name from .claude/workflows/autonomous-scheduler.js (impl + frontier prompts) and from hand-dispatched agent briefs so both inherit the same rule.
---

# proceed-and-document — proceed on a decision, document it, steer post-hoc

[OPUS-4.8] Internal agent skill. Authored by Opus 4.8 (Fable unavailable — flag
for re-review when Fable returns). This skill is the **reusable, single-source
form** of the standing rule that was previously trapped as inline prompt prose
inside `.claude/workflows/autonomous-scheduler.js`. Design basis:
`research/agent-effectiveness-program.md` §2.3 (the hook/workflow/skill/slash/cron
granularity rule — a *single-agent procedure needing judgment each run* is a
SKILL) and §7 Phase 1.5.1. Coordinates with bead `sq-6psmk` (the durable commit
of the proceed-on-decision update to the scheduler).

> **This skill does NOT define the rule — it carries it.** The authoritative
> wording lives in [`AGENTS.md`](../../../AGENTS.md) → *"STANDING RULE — proceed
> without waiting for the maintainer's greenlight"*. This file is the
> agent-facing procedure that both the workflow strings and a hand-dispatched
> agent point at **by name** so the rule is inherited identically everywhere
> instead of re-typed (and silently drifting) per brief. If the two ever
> disagree, AGENTS.md wins; fix this file to match.

## When this fires

You are working a bead (in the autonomous scheduler **or** hand-dispatched) and
you reach a point where you would normally **stop and ask the maintainer**:

- the bead needs a **design greenlight** you don't have, or
- there is a **decision** (a default value, an API shape, a naming choice, a
  scope cut) you would otherwise raise.

That is **not** a blocker. Proceed.

## What to do (the four steps)

1. **Make the best-judgment choice.** Pick the option a careful maintainer would
   most likely accept; prefer the smallest reversible version (opt-in /
   feature-gated / additive) so a later correction is cheap. Do not gold-plate.
2. **Document the choice in the PR body** — a short "Decision: chose X over Y
   because Z; reversible via …" note, so the diff is reviewable as a *decision*,
   not just code.
3. **Leave a one-line bead note** recording the same choice (`bd note <id> "…"`),
   so the decision is discoverable from the task, not only the PR.
4. **Open a short GitHub issue** flagging the decision for the maintainer to steer
   later. Lead it with the 🤖 SPARQ-agent self-id blockquote (see below), name the
   bead + PR, state the default you chose in one line, and ask the one question
   you'd have asked. This converts "needs-user / awaiting greenlight" into
   "proceeded with default X — see issue #N."

Then **continue the implementation** to a normal opt-in PR.

The SPARQ-agent self-id blockquote (required on the issue, the PR body, and every
comment — `@jeswr` runs several agents on one account):

```text
> 🤖 SPARQ agent — <one line of what this is / why>.
```

## When this does NOT apply (the two hard exceptions)

- **External credential / access you literally cannot obtain.** OS code-signing
  certs (`sq-v286.8`), npm/PyPI publish tokens, the **external accredited
  cryptographer audit** (`sq-qhy4`) — these are real blockers. Do **not**
  fabricate around them; skip the bead with that reason.
- **Honesty / soundness labels.** Proceeding on a *build* never licenses
  proceeding on a *claim*. Never relabel an unaudited ZK/MPC capability
  "sound/proven/secure" — that is gated on the external audit (`sq-qhy4`), a
  credential block, not a greenlight. Keep the honest "research-grade / not
  externally audited / semi-honest-only / no production guarantee" wording, and
  remember the **privacy-claims CI gate is LIVE** (an unqualified ZK/MPC
  privacy/soundness claim fails the build).

Also genuinely-not-implementable (skip, do not force): an **epic**, work already
**done on main**, or a bead so underspecified that *any* choice would be a guess
about the maintainer's intent rather than a defensible default — in the
underspecified case prefer to proceed with a documented default, and only skip
when there is no defensible default at all.

## How it is wired (single-source-of-truth)

- `.claude/workflows/autonomous-scheduler.js` — `implPrompt()` and the Frontier
  brief reference this skill **by name** instead of re-stating the rule. The
  workflow's job is to POINT here, not to re-paste the prose.
- **Hand-dispatched agents** (a brief you write outside the workflow) should say
  *"follow the `proceed-and-document` skill"* rather than re-typing the four
  steps, so the same rule is inherited.

## The durability contract (why this skill exists at all)

Any durable workflow/skill that future agents must inherit needs **all three**,
or it gets silently re-improvised:

1. **Committed** under `.claude/` (this file; the workflow `.js`).
2. **Linked from `AGENTS.md`** — for a workflow, from the *Maintenance loop*
   section / script catalog; for a recurring procedure, from the rule it codifies.
3. **A one-line skill `description`** (the frontmatter above) so it is
   discoverable by name.

Missing any one → it is not durable. This is the standing contract in
`AGENTS.md` (see the maintenance-loop / script-catalog scheduler entry) and in
`research/agent-effectiveness-program.md` §2.3.
