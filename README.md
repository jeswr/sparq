# agent-logs — orphan archive branch (DO NOT MERGE, DO NOT READ)

This is a DEDICATED ORPHAN branch that holds durable agent session transcripts as
`logs/<id>.jsonl`. It exists so transcripts stay OUT of the main working tree and out of
`main` history — a full transcript is a context-blowout vector if a working agent greps it.

## Hard rules

- **NEVER merged to `main`.** It has no PR and is excluded from CI / the merge queue.
- **NEVER checked out into a worktree.** Appends are git-plumbing only, via
  `scripts/save-agent-log.sh` on `main`.
- **Working agents do NOT read this branch** (AGENTS.md § sub-agent shared contract item 13).
  Log inspection is only the one explicitly-tasked debug/self-improvement agent's job.
- **Append-only archive.** The first commit is orphan (no parent); each log is one commit.
  A periodic prune re-roots the branch to drop entries older than N days (safe: non-canonical).

Authority: `research/agent-observability-and-self-improvement.md` on `main`.

> 🤖 SPARQ agent — orphan agent-log archive; see AGENTS.md items 12-13.
