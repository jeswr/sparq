# Agent observability + the self-improvement lane

<!-- [OPUS-4.8] Authored by Opus 4.8 (Fable unavailable; flag for re-review when Fable returns). -->

Authority for two agent conventions added to the shared contract (AGENTS.md § *The sub-agent
shared contract* items 12–13). Both exist to keep agent output OUT of every working agent's
context while still routing it back into the loop:

1. **Out-of-scope discovery → a self-filed `self-improvement` GitHub issue** (not an inline
   fix, not a bead).
2. **Agent transcripts/logs live OUT of the working tree and out of `main`** — a three-layer
   store — and working agents never read them.

## 1. Out-of-scope discoveries → a `self-improvement` issue

An agent working a scoped task routinely notices something else that should change — a latent
bug, tech-debt, doc drift, a footgun, a better approach. Fixing it inline is scope-creep: it
breaks the reviewer's model of the diff and the conflict-partition (one crate/surface per
in-flight branch). So the discovery goes to a **GitHub issue**, not the PR:

```sh
gh issue create --label self-improvement \
  --title "<imperative one-line>" \
  --body "> 🤖 SPARQ agent — <one line>

<what / where / why — one line each>"
```

- **SPAM guard + dedupe (mandatory).** File ONLY genuine, actionable, out-of-scope findings —
  never a nit you could fix inline, never a subjective style preference. Dedupe first:
  `gh issue list --state open --label self-improvement --search "<keywords>"`, and skip if an
  open issue already covers it.
- **Self-ID.** The body leads with the `> 🤖 SPARQ agent — <one line>` blockquote, per the
  cross-agent self-identification rule.

### Beads vs issues — the reconciliation

The two channels are NOT interchangeable:

- **Beads (`bd`) = the PLANNED task graph** the orchestrator already owns — the
  dependency-aware backlog it schedules from. An agent in a worktree does not have `bd` on
  PATH; it reports planned follow-ups as a LIST for the orchestrator to bead.
- **A `self-improvement` issue = the git-native channel for NEWLY-DISCOVERED work** an agent
  self-files itself, so a discovery is never lost to a report the orchestrator might not
  re-read. This is the same steer-post-hoc issue channel the *proceed-without-greenlight*
  standing rule already uses.

## 2. The self-improvement lane (NO new agent)

The `self-improvement` label is the LIGHT mechanism — it reuses two already-designed pieces
and adds **zero new agents**:

- **Triage/sweep = the existing `sparq-issue-sweeper` (model: sonnet).** Its sweep is extended
  to also cover `self-improvement` issues each tick: verify the finding against `origin/main`
  (close-satisfied if already fixed), dedupe, and add the missing triage labels
  (`role:<…>` + `priority:P<n>` + `area:<crate>`). Read-mostly and cheap.
- **Dispatch = the existing `scripts/ready-issues.py` chain.** It already surfaces open,
  unblocked, non-in-flight issues; a triaged `self-improvement` issue is routed to the normal
  role agent its `role:` label maps to.
- **Impl = whatever role agent the label routes to**, at that role's own tier (opus for
  `sparq-rust-feature`, sonnet/haiku for mechanical). The lane adds no model cost beyond the
  sweeper tick.

The `self-improvement` label sits in the Phase-0 issue label taxonomy alongside
`role:*` / `priority:*` / `area:*` (see
[`issue-native-orchestration.md`](issue-native-orchestration.md)). Created once:

```sh
gh label create self-improvement --force \
  --description "Agent-discovered out-of-scope work (bug/tech-debt/doc-drift/footgun/better-approach) for the self-improvement triage lane" \
  --color 0e8a16
```

### Migration timing

Issue-native orchestration is Phase 0→1 (see
[`orchestration-activation-runbook.md`](orchestration-activation-runbook.md)). The
`self-improvement` label + `gh issue create` path works TODAY regardless of phase, so it ships
now (git-native). If beads remain the primary tracker pre-Phase-1, the orchestrator may mirror
a high-priority `self-improvement` issue into a bead until the migration completes. Default
chosen: **ship the issue channel now; do not block on Phase 1.**

## 3. Agent-log storage — three layers, nothing an agent can grep in the tree

The failure being designed against: transcripts in the working tree (even `.gitignore`d), in
`main` history, or inlined in a PR/comment/doc body are all context-pollution vectors — a
future agent's broad `grep`/`ast-grep` can load a full transcript and blow up its window. The
store defeats every one of those vectors by keeping durable transcripts OUT of a working
agent's reach:

1. **DURABLE COLD ARCHIVE = the orphan `agent-logs` branch.** A dedicated orphan branch, NEVER
   merged to `main` and NEVER checked out into any worktree. Transcripts are appended via git
   plumbing only (see [`scripts/save-agent-log.sh`](../scripts/save-agent-log.sh)), so the main
   tree stays clean and a broad grep/ast-grep by a future agent cannot load a transcript. It is
   cloned explicitly only by the one debug/self-improvement agent that needs history.
2. **ACTIONS WORKERS = `actions/upload-artifact@v4`, 30-day retention.** The full transcript
   JSON is uploaded per worker session, indexed by `run_id` + `agent_id`; never committed to
   the repo; GC'd by GitHub retention. (Wired when the Actions orchestration worker lands.)
3. **SESSION-LOCAL / INDEX.** The ephemeral `/tmp/claude-*/**/tasks/*.output` (harness-managed,
   agent-not-read) stays as-is. Optionally a **summary-only** row — agent id + bead/issue +
   timestamp + one-line summary, NEVER the transcript body — may be appended to the already
   git-ignored `metrics/runtime/agent-logs.jsonl` (covered by the existing `metrics/runtime/`
   `.gitignore` entry) for cheap cross-session lookup.

PRs / issues / AGENTS.md / docs carry ONLY a one-line link — an `agent-logs:<id>` branch ref or
an Actions artifact URL — **never the log body**.

### Why this defeats every pollution vector

| vector | defeated by |
| --- | --- |
| working-tree grep loads a transcript | logs never land in a worktree — orphan branch is never checked out |
| `main` history carries a transcript | `agent-logs` is never merged to `main`; artifacts never commit — `main` bloat is zero by construction |
| PR/comment/doc body inlines a transcript | only a one-line LINK is ever pasted (item 13 forbids the body) |
| CI runs on every log append | no workflow triggers on all branches (verified below), so a push to `agent-logs` triggers zero CI |

### `scripts/save-agent-log.sh`

`save-agent-log.sh <transcript-path> <id>` appends one transcript to `agent-logs` using
`git hash-object` / an isolated `GIT_INDEX_FILE` / `commit-tree` / `update-ref` / `push`. HARD
invariants asserted in the script header (and by its `--self-test`):

- NEVER `git checkout` / `git switch` / `git merge` the branch.
- NEVER stage into the REAL index — always an isolated `GIT_INDEX_FILE` on a temp path.
- `agent-logs` is ORPHAN — its first commit has no parent; the branch is APPEND-ONLY.
- The caller's cwd, working tree, and index are left byte-for-byte untouched (proven by
  `save-agent-log.sh --self-test`, which fails if `git status` or the real index changes).

### CI exclusion (open question #3)

A `push` glob over all branches would run CI on every log append. **Verified at authoring
time:** every workflow `push:` trigger is scoped to `branches: [main]` (or `benchmark-data`, or
`v*` tags) — none globs `['**']` — so pushing `agent-logs` triggers no CI, and no workflow edit
is needed today. If a future change adds an all-branch trigger, that workflow MUST carry
`branches-ignore: [agent-logs]`, and `agent-logs` must stay out of branch-protection / the merge
queue (it is not `main` and has no PR).

### Retention / prune

`agent-logs` grows unboundedly if unmanaged. The prune is a periodic chore (a
`sparq-issue-sweeper` / CI job, e.g. quarterly) that re-roots the orphan branch to drop entries
older than N days and force-pushes. Because the branch is a NON-canonical archive, a history
rewrite is safe there — confirm no external tooling pins its refs before enabling the chore.
Actions artifacts self-GC via `retention-days`; `main` bloat stays zero.

### The no-read rule

Working agents do NOT `Read` / `cat` / `grep` / `ast-grep` the ephemeral transcripts, the
`agent-logs` branch, or any saved transcript — a full transcript is a guaranteed context
blowout and these are write-only from an agent's side. Log inspection is ONLY the job of the ONE
explicitly-tasked debug/self-improvement agent the maintainer/orchestrator dispatches for exactly
that purpose — never a side-quest inside an unrelated task.

## Model tiers

The self-improvement lane itself adds no new model cost: triage/sweep runs at the existing
`sparq-issue-sweeper` **sonnet** tier (cheap, read-mostly); the actioning impl agent runs at
whatever tier its `role:` label normally uses (opus for `sparq-rust-feature`, sonnet/haiku for
mechanical).

## Links

- AGENTS.md § *The sub-agent shared contract* items 12–13 (the authority; this record is the
  long form).
- [`issue-native-orchestration.md`](issue-native-orchestration.md) — the issue label taxonomy
  the `self-improvement` label extends.
- [`orchestration-activation-runbook.md`](orchestration-activation-runbook.md) — the Phase 0→1
  migration this lane ships ahead of.
- [`scripts/save-agent-log.sh`](../scripts/save-agent-log.sh) — the git-plumbing archiver.
