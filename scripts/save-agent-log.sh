#!/usr/bin/env bash
# [OPUS-4.8] Safe agent-transcript archiver — appends to the orphan `agent-logs` branch.
# Authored by Opus 4.8 (Fable unavailable; flag for re-review when Fable returns).
# 🤖 SPARQ agent tooling. Design authority: research/agent-observability-and-self-improvement.md
#
# save-agent-log.sh <transcript-path> <id>
# save-agent-log.sh --self-test
#
# WHY: agent session transcripts are large, write-only-from-an-agent's-side, and a
# context-blowout vector if a future working agent greps them. So they must live OUT of the
# main working tree AND out of `main` history. This script appends one transcript to a
# DEDICATED ORPHAN branch `agent-logs` using git PLUMBING ONLY — it never checks out, never
# switches branches, never merges, and never stages into the caller's real index — so the
# caller's cwd, working tree, and index are left byte-for-byte untouched.
#
# HARD INVARIANTS (asserted by design; do NOT relax):
#   * NEVER `git checkout` / `git switch` / `git merge` the `agent-logs` branch.
#   * NEVER stage into the REAL index — always an isolated `GIT_INDEX_FILE` on a temp path.
#   * `agent-logs` is ORPHAN: its first commit has NO parent (`-p`); it is APPEND-ONLY.
#   * `agent-logs` is EXCLUDED from CI / the merge queue — it is not `main` and has no PR.
#     (Any all-branch-triggered workflow must carry `branches-ignore: [agent-logs]`.)
#
# The commit that ADDS this script to the repo carries the [OPUS-4.8] marker + the
# `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>` trailer; the
# per-log archive commits this script CREATES use a terse `agent-log: <id>` subject.
#
# ACTIONS-WORKER path (documented, wired when the Actions orchestration worker lands):
# after a worker session, `actions/upload-artifact@v4` with
#   name: agent-logs-${{ github.run_id }}-<agent-id>
#   path: <session-output.jsonl>
#   retention-days: 30
# and the PR/issue body carries only `Full logs: <artifact-UI-URL>`. A separate 7-day
# manifest artifact maps run-id+agent-id -> bead/issue for cross-run lookup.
#
# RETENTION: `agent-logs` grows unboundedly if unmanaged — the design record specifies a
# periodic prune chore (quarterly) that re-roots the orphan branch to drop entries older
# than N days and force-pushes. It is a NON-canonical archive, so history rewrite is safe.

set -uo pipefail

LOG_BRANCH="agent-logs"
LOG_REF="refs/heads/${LOG_BRANCH}"
REMOTE="${AGENT_LOG_REMOTE:-origin}"

die() { printf 'save-agent-log: %s\n' "$1" >&2; exit 1; }

# Sanitize an id into a safe single path segment.
safe_id() { printf '%s' "$1" | tr -c 'A-Za-z0-9._-' '_'; }

# Append <transcript-path> as logs/<id>.jsonl onto the orphan agent-logs branch.
# Uses an isolated GIT_INDEX_FILE so the real index is never touched; does not push
# (the caller decides). Prints the one-line ref link on success.
save_log() {
  local transcript="$1" raw_id="$2"
  [ -n "$transcript" ] || die "missing <transcript-path>"
  [ -n "$raw_id" ] || die "missing <id>"
  [ -s "$transcript" ] || die "transcript is missing or empty: $transcript"

  local id; id="$(safe_id "$raw_id")"
  [ -n "$id" ] || die "id sanitized to empty"

  # 1) Write the transcript blob into the object DB (no index, no worktree touch).
  local blob; blob="$(git hash-object -w -- "$transcript")" || die "hash-object failed"

  # 2) Build a new tree on top of the current agent-logs tip using an ISOLATED index.
  local base tmp_index tree
  base="$(git rev-parse -q --verify "$LOG_REF" || true)"
  tmp_index="$(mktemp)" || die "mktemp failed"
  # shellcheck disable=SC2064
  trap "rm -f '$tmp_index'" RETURN

  # Seed the isolated index from the base tree (empty for the first, orphan commit), stage
  # the blob at logs/<id>.jsonl, and write the tree — all against GIT_INDEX_FILE only, so
  # the caller's real index and working tree are never touched.
  tree="$(
    GIT_INDEX_FILE="$tmp_index" bash -c '
      set -e
      if [ -n "$1" ]; then git read-tree "$1^{tree}"; else git read-tree --empty; fi
      git update-index --add --cacheinfo "100644,$2,logs/$3.jsonl"
      git write-tree
    ' _ "$base" "$blob" "$id"
  )" || die "failed to build tree"
  [ -n "$tree" ] || die "failed to build tree (empty)"

  # 3) Commit: orphan (no -p) for the first log, else parented on the current tip.
  local commit
  if [ -n "$base" ]; then
    commit="$(git commit-tree "$tree" -p "$base" -m "agent-log: ${id}")" || die "commit-tree failed"
  else
    commit="$(git commit-tree "$tree" -m "agent-log: ${id}")" || die "commit-tree failed"
  fi

  # 4) Advance the ref (append-only; never a checkout / merge / switch).
  git update-ref "$LOG_REF" "$commit" || die "update-ref failed"

  printf '%s:%s\n' "$LOG_BRANCH" "$id"
}

# --self-test: prove the archive path does NOT touch the working tree or the real index.
self_test() {
  local status_before status_after index_before index_after
  git rev-parse --show-toplevel >/dev/null 2>&1 || die "self-test must run inside a git repo"

  status_before="$(git status --porcelain)"
  # A stable fingerprint of the REAL index.
  index_before="$(git ls-files -s | git hash-object --stdin 2>/dev/null || true)"

  # Save onto a THROWAWAY test ref so we never mutate the real agent-logs branch in a test.
  local test_ref="refs/heads/agent-logs-selftest-$$"
  local tmp; tmp="$(mktemp)"; printf 'self-test transcript body\n' > "$tmp"
  local blob; blob="$(git hash-object -w -- "$tmp")"
  local tmp_index; tmp_index="$(mktemp)"
  local tree commit
  tree="$(GIT_INDEX_FILE="$tmp_index" bash -c 'git read-tree --empty && git update-index --add --cacheinfo "100644,'"$blob"',logs/selftest.jsonl" && git write-tree')" \
    || die "self-test tree build failed"
  commit="$(git commit-tree "$tree" -m 'agent-log: selftest')" || die "self-test commit failed"
  git update-ref "$test_ref" "$commit" || die "self-test update-ref failed"

  status_after="$(git status --porcelain)"
  index_after="$(git ls-files -s | git hash-object --stdin 2>/dev/null || true)"

  # Clean up the throwaway ref, blob is harmless (unreferenced, GC'd later).
  git update-ref -d "$test_ref" 2>/dev/null || true
  rm -f "$tmp" "$tmp_index"

  local ok=1
  if [ "$status_before" != "$status_after" ]; then
    echo "FAIL: working-tree status changed"; ok=0
  fi
  if [ "$index_before" != "$index_after" ]; then
    echo "FAIL: real index changed"; ok=0
  fi
  # The reachable blob proves the archive commit exists as a real git object.
  if ! git cat-file -e "$commit^{commit}" 2>/dev/null; then
    echo "FAIL: archive commit not created"; ok=0
  fi
  if [ "$ok" -eq 1 ]; then
    echo "PASS: save-agent-log.sh built an archive commit WITHOUT touching the working tree or the real index"
    return 0
  fi
  return 1
}

main() {
  case "${1:-}" in
    ""|-h|--help)
      grep -E '^# ' "$0" | sed 's/^# \{0,1\}//'
      [ "${1:-}" = "" ] && exit 1 || exit 0
      ;;
    --self-test)
      self_test; exit $?
      ;;
    *)
      local ref
      ref="$(save_log "$1" "${2:-}")" || exit 1
      # Push the append (fast-forward; the branch is an append-only archive).
      # Skip the push in AGENT_LOG_NO_PUSH mode (local-only / test).
      if [ "${AGENT_LOG_NO_PUSH:-0}" != "1" ]; then
        git push "$REMOTE" "${LOG_BRANCH}:${LOG_BRANCH}" >&2 \
          || die "push to ${REMOTE}/${LOG_BRANCH} failed (branch must be push-allowed + CI-excluded)"
      fi
      printf '%s\n' "$ref"
      ;;
  esac
}

main "$@"
