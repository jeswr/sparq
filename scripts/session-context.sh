#!/usr/bin/env bash
# SessionStart hook — inject a concise READY-ISSUE snapshot so a NEW or post-compaction
# Claude Code session recovers this repo's task state automatically.
#
# The repo migrated from beads (`bd`) to GitHub issues on 2026-07-17 (docs/bd-migration.md);
# GitHub issues are now the sole tracker. This hook prints the dispatchable READY frontier —
# the issues carrying `status:ready` with a clear conflict partition — computed by
# scripts/ready-issues.py (the issue-native replacement for `bd ready` / push-frontier.sh).
#
# Graceful no-op when `gh` is unavailable/unauthenticated or the network is down: a hook must
# never block or error a session start. See AGENTS.md "Task tracking — GitHub issues".
# [FABLE-5]
set -uo pipefail

# Always operate from the repo root, regardless of the CWD the hook is invoked from
# (the script lives in <root>/scripts/).
# shellcheck disable=SC1007  # `CDPATH= cd` is the intended empty-CDPATH prefix, not a typo.
script_dir="$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]:-$0}")" && pwd)" || exit 0
cd "$script_dir/.." 2>/dev/null || exit 0

# Graceful no-op when the tools this needs aren't present.
command -v gh >/dev/null 2>&1 || exit 0
command -v python3 >/dev/null 2>&1 || exit 0

REPO="sparq-org/sparq"

# Compute the ready frontier (fail-soft: any error → empty, never a red session start).
frontier="$(timeout 12 python3 scripts/ready-issues.py --repo "$REPO" 2>/dev/null | head -25)" || frontier=""
[ -n "$frontier" ] || frontier="(no ready issues surfaced right now — run 'python3 scripts/ready-issues.py')"

# Open-issue count (fail-soft; the search API gives a real total, unlike a --limit'd list).
open_total="$(timeout 8 gh search issues --repo "$REPO" --state open --json number \
  --jq 'length' 2>/dev/null | tr -dc '0-9')" || open_total=""
[ -n "$open_total" ] || open_total="?"

ctx="GitHub issues (sparq-org/sparq) are the task tracker for this repo (beads/\`bd\` was retired 2026-07-17 — see docs/bd-migration.md). Dispatchable READY frontier now (status:ready, conflict-partitioned by area:<crate>):
${frontier}

(~${open_total} open issues total. Use 'python3 scripts/ready-issues.py' for the frontier, 'gh issue list', 'gh issue view <n>', 'gh issue create --label \"area:<crate>,priority:P2,role:impl\"'. status:ready is a POSITIVE attestation; needs:user parks maintainer-gated work. Full workflow: AGENTS.md 'Task tracking — GitHub issues'.)"

# Emit as SessionStart additionalContext (injected into the model's context at session start).
python3 -c 'import json,sys; print(json.dumps({"hookSpecificOutput":{"hookEventName":"SessionStart","additionalContext":sys.argv[1]}}))' "$ctx"
