#!/usr/bin/env bash
# [SONNET-4.6] Hermetic end-to-end tests for scripts/bead-export-sync.sh (bead sq-2xdg,
# issue #3290 — the missing CREATE/UPDATE direction that syncs work-box bead creates into
# the committed `.beads/issues.jsonl` source-of-record).
#
# WHY: the script's own `--dry-run-self-test` covers the PURE merge core (no-delete,
# stale-write guard, idempotency, fail-closed parsing, the force-branch predicate). THIS
# harness covers the END-TO-END behaviour — where the script actually reads `bd`, builds a
# commit and pushes it — against a REAL local git repo with a REAL bare origin. The
# invariants pinned here are the ones a refactor could silently break with real damage:
#   1. DEFAULT IS DRY-RUN: no branch is pushed and no PR is opened without --apply.
#   2. --apply publishes the merged board on the rolling branch, and the PUSHED BLOB
#      contains both the new bead AND the bead the local export omitted (no-delete, end to
#      end — not just in the pure merge).
#   3. THE CALLER'S CHECKOUT IS NEVER TOUCHED: HEAD, the branch, the index and the working
#      tree are byte-identical after --apply. (The commit is built with a temp index +
#      commit-tree precisely so this holds while agents are building in the checkout.)
#   4. IDEMPOTENT: a second --apply with the same export is a no-op — no new commit, and
#      `gh pr create` is not called again.
#   5. THE PR TITLE CARRIES NO `sq-` TOKEN, or bead-autoclose.yml would close every bead
#      named in it on each sync.
#   6. A FOREIGN/PARTIAL export (no id overlap) is REFUSED — nothing pushed.
#   7. `bd` absent => inert no-op exit 0 (the script only means anything on the work box).
#
# HERMETIC: `bd` and `gh` are PATH-shadowed by stubs; `git` is REAL but confined to a
# mktemp sandbox with a local bare remote. No network, no real bead tracker, and nothing is
# written outside the sandbox (removed on exit).
#
# Run:  bash scripts/tests/test_bead_export_sync.sh   (exit 0 = all pass, 1 = a failure)
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SRC="${ROOT}/scripts/bead-export-sync.sh"
[ -f "$SRC" ] || {
  echo "FATAL: script not found at ${SRC}"
  exit 2
}

pass=0
fail=0
note_pass() { pass=$((pass + 1)); }
note_fail() {
  fail=$((fail + 1))
  printf 'CASE FAILED: %s\n' "$1"
}
want_contains() {
  if printf '%s' "$2" | grep -qF -- "$3"; then note_pass; else note_fail "$1 (expected to CONTAIN '$3')"; fi
}
want_absent() {
  if printf '%s' "$2" | grep -qF -- "$3"; then note_fail "$1 (expected NOT to contain '$3')"; else note_pass; fi
}
want_eq() {
  if [ "$2" = "$3" ]; then note_pass; else
    note_fail "$1"
    printf '       got:  %q\n       want: %q\n' "$2" "$3"
  fi
}

SANDBOX="$(mktemp -d)"
trap 'rm -rf "$SANDBOX"' EXIT
BIN="${SANDBOX}/bin"
REPO="${SANDBOX}/repo"
ORIGIN="${SANDBOX}/origin.git"
mkdir -p "$BIN"

# --- stub `bd`: `bd export` cats whichever fixture BD_EXPORT_FIXTURE points at. ---------
cat >"${BIN}/bd" <<'EOF'
#!/usr/bin/env bash
[ "${1-}" = "export" ] || { echo "stub bd: unexpected args: $*" >&2; exit 64; }
cat "$BD_EXPORT_FIXTURE"
EOF
chmod +x "${BIN}/bd"

# --- stub `gh`: records every invocation, and reports "no open PR" unless GH_HAS_PR=1. --
cat >"${BIN}/gh" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' "$*" >>"$GH_CALLS"
case "${1-} ${2-}" in
  "pr list")
    if [ "${GH_HAS_PR:-0}" = "1" ]; then echo "https://github.com/jeswr/sparq/pull/999"; fi
    ;;
  "pr create")
    echo "https://github.com/jeswr/sparq/pull/1000"
    ;;
esac
exit 0
EOF
chmod +x "${BIN}/gh"

export PATH="${BIN}:${PATH}"
export GH_CALLS="${SANDBOX}/gh-calls.log"
: >"$GH_CALLS"

rec() { # rec <id> <updated_at> <title>
  printf '{"_type":"issue","id":"%s","title":"%s","status":"open","updated_at":"%s"}\n' "$1" "$3" "$2"
}

# --- a real repo with a real bare origin ------------------------------------------------
git init --quiet --bare "$ORIGIN"
git init --quiet -b main "$REPO"
git -C "$REPO" config user.name "sparq test"
git -C "$REPO" config user.email "test@example.invalid"
mkdir -p "${REPO}/.beads"
{
  rec sq-aaa 2026-07-01T00:00:00Z "alpha"
  rec sq-bbb 2026-07-01T00:00:00Z "beta"
} >"${REPO}/.beads/issues.jsonl"
git -C "$REPO" add .beads/issues.jsonl
git -C "$REPO" commit --quiet -m "seed the board"
git -C "$REPO" remote add origin "$ORIGIN"
git -C "$REPO" push --quiet origin main
git -C "$REPO" fetch --quiet origin main

# The work-box export: sq-aaa updated, sq-ccc NEW, and sq-bbb ABSENT (a bead this box's DB
# does not have — it must survive the sync).
export BD_EXPORT_FIXTURE="${SANDBOX}/export.jsonl"
{
  rec sq-aaa 2026-07-31T00:00:00Z "alpha renamed"
  rec sq-ccc 2026-07-31T00:00:00Z "gamma"
} >"$BD_EXPORT_FIXTURE"

run() { (cd "$REPO" && bash "$SRC" "$@" 2>&1); }
remote_blob() { git -C "$ORIGIN" show "refs/heads/chore/bead-export-sync:.beads/issues.jsonl" 2>/dev/null; }

# --------------------------------------------------------------------------------------
# 1. DEFAULT IS DRY-RUN — reports the delta, pushes nothing, calls no gh mutation.
# --------------------------------------------------------------------------------------
out="$(run)"
want_contains "dry-run reports the new bead" "$out" "1 new"
want_contains "dry-run names it as dry-run" "$out" "DRY-RUN"
want_eq "dry-run pushed no branch" "$(remote_blob)" ""
want_absent "dry-run called no gh pr create" "$(cat "$GH_CALLS")" "pr create"

# --------------------------------------------------------------------------------------
# 2 + 3 + 5. --apply publishes; the no-delete invariant holds on the PUSHED blob; the
#            caller's checkout is untouched; the PR title carries no bead token.
# --------------------------------------------------------------------------------------
head_before="$(git -C "$REPO" rev-parse HEAD)"
branch_before="$(git -C "$REPO" rev-parse --abbrev-ref HEAD)"
status_before="$(git -C "$REPO" status --porcelain)"
out="$(run --apply)"
blob="$(remote_blob)"
want_contains "--apply pushed the rolling branch" "$blob" '"id":"sq-ccc"'
want_contains "the updated record landed" "$blob" '"title":"alpha renamed"'
want_contains "NO-DELETE: the bead absent from this box survived" "$blob" '"id":"sq-bbb"'
want_contains "--apply opened a PR" "$out" "opened the export-sync PR"
want_contains "gh pr create was called" "$(cat "$GH_CALLS")" "pr create"
want_eq "the caller's HEAD is unchanged" "$(git -C "$REPO" rev-parse HEAD)" "$head_before"
want_eq "the caller's branch is unchanged" "$(git -C "$REPO" rev-parse --abbrev-ref HEAD)" "$branch_before"
want_eq "the caller's worktree/index is unchanged" "$(git -C "$REPO" status --porcelain)" "$status_before"
# The PR title must not contain an sq- token (bead-autoclose reads merged PR titles).
title="$(grep -F 'pr create' "$GH_CALLS" | tail -1)"
if printf '%s' "$title" | grep -qE 'title [^-]*sq-[a-z0-9]'; then
  note_fail "the PR title contains an sq- token (bead-autoclose would fire on merge)"
else note_pass; fi

# --------------------------------------------------------------------------------------
# 4. IDEMPOTENT: re-running against the SAME board (now including the sync) is a no-op.
#    Simulate the PR having merged by fast-forwarding origin/main onto the synced commit.
# --------------------------------------------------------------------------------------
synced="$(git -C "$ORIGIN" rev-parse "refs/heads/chore/bead-export-sync")"
git -C "$ORIGIN" update-ref refs/heads/main "$synced"
git -C "$REPO" fetch --quiet origin main
before_calls="$(wc -l <"$GH_CALLS")"
out="$(run --apply)"
want_contains "a re-run after the sync merged is a no-op" "$out" "already up to date"
want_eq "the no-op called gh zero times" "$(wc -l <"$GH_CALLS")" "$before_calls"

# --------------------------------------------------------------------------------------
# 6. A FOREIGN / partial export (no id overlap) is REFUSED — nothing published.
# --------------------------------------------------------------------------------------
rec sq-zzz 2026-07-31T00:00:00Z "some other project" >"$BD_EXPORT_FIXTURE"
before_calls="$(wc -l <"$GH_CALLS")"
head_of_branch_before="$(git -C "$ORIGIN" rev-parse refs/heads/chore/bead-export-sync)"
out="$(run --apply)"
want_contains "a foreign export is refused" "$out" "partial or foreign database"
want_eq "the refusal published nothing" "$(wc -l <"$GH_CALLS")" "$before_calls"
want_eq "the refusal pushed nothing" "$(git -C "$ORIGIN" rev-parse refs/heads/chore/bead-export-sync)" "$head_of_branch_before"
# ...and --force overrides it deliberately (the escape hatch must actually work).
out="$(run --apply --force)"
want_contains "--force overrides the overlap guard" "$out" "pushed"
want_contains "--force still never deletes" "$(remote_blob)" '"id":"sq-aaa"'

# --------------------------------------------------------------------------------------
# 7. `bd` absent => inert no-op, exit 0 (this sync only means anything on the work box).
# --------------------------------------------------------------------------------------
mv "${BIN}/bd" "${BIN}/bd.disabled"
out="$( (cd "$REPO" && bash "$SRC" --apply 2>&1) )"
rc=$?
mv "${BIN}/bd.disabled" "${BIN}/bd"
want_eq "no bd => exit 0 (inert, never reddens a maintenance tick)" "$rc" "0"
want_contains "no bd => explains it only runs on the work box" "$out" "only runs on the work box"

printf '\n%s: %d passed, %d failed\n' "$(basename "$0")" "$pass" "$fail"
[ "$fail" -eq 0 ] || exit 1
exit 0
