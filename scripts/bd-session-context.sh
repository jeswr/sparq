#!/usr/bin/env bash
# SessionStart hook — inject a concise beads (bd) task snapshot so a NEW or
# post-compaction Claude Code session recovers this repo's task state automatically.
#
# Deliberately does NOT run `bd prime` / `bd setup claude`: that path injects generic
# rules ("do NOT use TaskCreate", "do NOT use MEMORY.md files") that conflict with this
# harness's own task tracker and MEMORY.md auto-memory, and it duplicates the beads
# guidance already in AGENTS.md. This hook ships only the useful, non-conflicting bit:
# the unblocked-work list + an open count. See AGENTS.md "Beads session-context hook".
# [OPUS-4.8]
set -uo pipefail

# Graceful no-op when bd isn't installed or this isn't a beads workspace.
command -v bd >/dev/null 2>&1 || exit 0
[ -d .beads ] || exit 0

ready="$(bd ready 2>/dev/null | head -25)"
[ -n "$ready" ] || ready="(no unblocked beads right now — run 'bd ready')"
open_count="$(bd count --status open 2>/dev/null | tr -dc '0-9')"
[ -n "$open_count" ] || open_count="?"

ctx="Beads (bd) is the task tracker for this repo (source of truth: .beads/issues.jsonl — never edit it directly; use bd commands). Unblocked work right now:
${ready}

(${open_count} open beads total. Use 'bd ready', 'bd show <id>', 'bd create \"title\" -t task -p 2'. Title is POSITIONAL; valid --type: bug|feature|task|epic|chore. Full workflow: AGENTS.md 'Task tracking — beads'.)"

# Emit as SessionStart additionalContext (injected into the model's context at session start).
python3 -c 'import json,sys; print(json.dumps({"hookSpecificOutput":{"hookEventName":"SessionStart","additionalContext":sys.argv[1]}}))' "$ctx"
