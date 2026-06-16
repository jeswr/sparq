#!/usr/bin/env bash
# [OPUS-4.8] Safe worktree garbage-collection sweep (bead sq-6xdr).
# Authored by Opus 4.8 (Fable unavailable; flag for re-review when Fable returns).
#
# worktree-gc.sh [--dry-run | --apply] [--root <dir>] [--main <path>] [--base <ref>]
#
# WHY: the Claude Code harness creates a fresh git worktree per agent under
# `.claude/worktrees/` but NEVER auto-removes a finished agent's worktree. They pile up
# (366+ this session) and each carries a `target/` build dir (~8-12G); together they filled
# the disk. This is the MANUAL / idle-time broom that removes ONLY the worktrees that are
# provably safe to delete — never one that still holds work.
#
# SAFETY PREDICATE — a worktree is SAFE-to-remove iff ALL of:
#   (a) MERGED-or-GONE: its HEAD commit is already an ancestor of the base ref
#       (default origin/main) — i.e. every commit it holds is on the remote, in main —
#       OR its branch is GONE on origin (the upstream/origin ref it tracked is deleted,
#       which is what a merged-then-branch-deleted PR looks like) AND its HEAD is still
#       reachable from some remote-tracking ref (so we are not about to drop the only copy
#       of a commit). The ancestor-of-base test is the load-bearing one: if HEAD is in
#       origin/main, there is by definition nothing to lose.
#   (b) CLEAN: `git -C <wt> status --porcelain` is empty (no uncommitted / untracked work).
#   (c) NO UNPUSHED COMMITS: HEAD has no commits absent from its push target (its upstream,
#       else origin/<branch>). When (a) holds via ancestor-of-base this is automatic; we
#       still assert it independently so a branch that is "gone on origin" but carries local
#       commits the remote never saw is REJECTED.
#   (d) NOT THE MAIN CHECKOUT: the main repo (default /home/ubuntu/sparq) is a HARD,
#       unconditional exclusion — removed from the candidate set before any test, asserted
#       by the self-test. We also only ever consider worktrees physically under <root>
#       (default the harness `.claude/worktrees/` dir): allow-list by location, not deny-list.
#
# Any worktree that is locked, prunable (its dir is already gone), bare, detached with
# unreachable HEAD, dirty, has unpushed commits, or whose branch is neither merged nor gone
# is KEPT and reported with the reason it was kept. When in doubt, KEEP.
#
# DEFAULT IS --dry-run: prints the safe-to-remove set with a per-worktree reason and a
#   reclaimable-size estimate (du -sh of each safe dir + a total). Touches nothing.
# --apply: `git worktree remove --force` each safe worktree, then `git worktree prune`.
#   The clean-status check (b) requires `git status --porcelain` to be EMPTY, so the SAFE set
#   has no uncommitted or untracked work. --force is needed only because these worktrees carry
#   a git-IGNORED `target/` build dir (ignored files never appear in `status --porcelain`, so
#   they cannot fail check (b), and plain `git worktree remove` would refuse over them); those
#   build artifacts are regenerable. --force here therefore discards only ignored regenerables.
#
# NOTE: do not run --apply while other agents may be using sibling worktrees concurrently;
# run it at idle. The predicate cannot misclassify a busy worktree (an active agent has
# either uncommitted work → (b) fails, or unmerged/unpushed commits → (a)/(c) fail), but
# removing a worktree mid-build still aborts that build.
#
# Run:
#   scripts/worktree-gc.sh                      # dry-run (default): list safe-to-remove + size
#   scripts/worktree-gc.sh --apply              # actually remove the safe set, then prune
#   scripts/worktree-gc.sh --base origin/main   # change the "merged" base ref (default)
#   scripts/worktree-gc.sh --dry-run-self-test  # hermetic self-test (no git mutation)
set -euo pipefail

PROG="worktree-gc"
log()  { printf '[%s] %s\n' "$PROG" "$*" >&2; }
die()  { printf '[%s] ERROR: %s\n' "$PROG" "$*" >&2; exit 1; }

# --- configuration (overridable by flags / env) ----------------------------------------
# The harness worktree root. Only worktrees physically under this dir are EVER candidates
# (allow-list by location). Everything outside it is kept untouched.
DEFAULT_ROOT="/home/ubuntu/sparq/.claude/worktrees"
# The main checkout — a HARD, unconditional never-remove. `git worktree remove` would refuse
# the main working tree anyway, but we drop it from the candidate set first regardless.
DEFAULT_MAIN="/home/ubuntu/sparq"
# The ref a worktree's HEAD must be an ancestor of to count as "merged".
DEFAULT_BASE="origin/main"

# --- pure, testable safety classifier ---------------------------------------------------
# is_protected_path <path> <main>
#   Hard exclusion: true (0) iff <path> is the main checkout (exact, after normalising a
#   trailing slash). Pure string compare — no git, no fs access — so it is unit-testable and
#   can never be tricked into NOT protecting main.
is_protected_path() {
  local p="${1%/}" main="${2%/}"
  [ "$p" = "$main" ]
}

# under_root <path> <root>
#   Allow-list by location: true (0) iff <path> is the root itself or strictly beneath it.
#   Prevents a stray worktree elsewhere on disk from ever entering the candidate set.
under_root() {
  local p="${1%/}" root="${2%/}"
  [ "$p" = "$root" ] && return 1   # the root dir itself is not a worktree we manage
  case "$p/" in
    "$root"/*) return 0 ;;
    *)         return 1 ;;
  esac
}

# --- git-backed per-worktree probes (each isolated so the classifier reads cleanly) ------
# All run with `git -C <wt>` so they ask THAT worktree, not this one.

# wt_is_clean <wt> : 0 iff `git status --porcelain` is empty (no uncommitted/untracked).
wt_is_clean() {
  local wt="$1" out
  out="$(git -C "$wt" status --porcelain 2>/dev/null)" || return 1
  [ -z "$out" ]
}

# wt_head <wt> : print the worktree HEAD commit sha (empty on failure).
wt_head() { git -C "$wt" rev-parse --verify -q HEAD 2>/dev/null; }

# wt_head_in_base <wt> <base> : 0 iff HEAD is an ancestor of <base> (fully merged; nothing
# to lose). This single test satisfies both predicate (a) "merged" and (c) "no unpushed".
wt_head_in_base() {
  local wt="$1" base="$2"
  git -C "$wt" merge-base --is-ancestor HEAD "$base" 2>/dev/null
}

# wt_branch <wt> : print the worktree's checked-out branch short name (empty if detached).
wt_branch() {
  local b
  b="$(git -C "$1" symbolic-ref -q --short HEAD 2>/dev/null)" || return 0
  printf '%s' "$b"
}

# wt_unpushed_count <wt> <branch> : number of commits on HEAD absent from the push target.
# Push target = configured upstream if any, else origin/<branch>. If neither exists (branch
# was never pushed) we return a large sentinel so the caller treats it as "has unpushed work"
# and KEEPS the worktree — never-pushed local commits must not be dropped.
wt_unpushed_count() {
  local wt="$1" branch="$2" upstream target
  upstream="$(git -C "$wt" rev-parse --abbrev-ref --symbolic-full-name '@{upstream}' 2>/dev/null)" || upstream=""
  if [ -n "$upstream" ] && git -C "$wt" rev-parse --verify -q "$upstream" >/dev/null 2>&1; then
    target="$upstream"
  elif [ -n "$branch" ] && git -C "$wt" rev-parse --verify -q "origin/$branch" >/dev/null 2>&1; then
    target="origin/$branch"
  else
    printf '999999'   # no push target ⇒ treat as unpushed ⇒ KEEP
    return 0
  fi
  git -C "$wt" rev-list --count "$target..HEAD" 2>/dev/null || printf '999999'
}

# wt_branch_gone <wt> <branch> : 0 iff the branch tracked an upstream that no longer exists
# (the merged-then-branch-deleted shape) OR there is simply no origin/<branch> ref while an
# upstream was configured. A branch that never had an upstream is NOT "gone".
wt_branch_gone() {
  local wt="$1" branch="$2" track
  [ -n "$branch" ] || return 1
  track="$(git -C "$wt" for-each-ref --format='%(upstream:track)' "refs/heads/$branch" 2>/dev/null)" || return 1
  [ "$track" = "[gone]" ]
}

# wt_head_reachable_from_remote <wt> : 0 iff HEAD is contained in at least one
# remote-tracking ref (refs/remotes/*). Guards the "gone" path: never delete a worktree whose
# HEAD exists only locally.
wt_head_reachable_from_remote() {
  local wt="$1" head
  head="$(wt_head "$wt")" || return 1
  [ -n "$head" ] || return 1
  git -C "$wt" for-each-ref --contains "$head" --format='%(refname)' refs/remotes 2>/dev/null \
    | grep -q .
}

# classify_reason <wt> <base> : echo a single token describing the SAFE basis, or a KEEP
# reason. Exit 0 ⇒ safe to remove; exit 1 ⇒ keep. This is the predicate in one place.
#   safe tokens : MERGED  | BRANCH-GONE
#   keep  tokens: dirty | unpushed=<n> | detached-unreachable | not-merged-not-gone | unreadable
classify_reason() {
  local wt="$1" base="$2" branch unpushed
  # (b) CLEAN — independent of everything else; a dirty tree is always KEEP.
  if ! wt_is_clean "$wt"; then echo "dirty"; return 1; fi

  branch="$(wt_branch "$wt")"

  # (a) primary: HEAD already in base ⇒ merged ⇒ nothing to lose. Also satisfies (c).
  if wt_head_in_base "$wt" "$base"; then echo "MERGED"; return 0; fi

  # (c) any commit not on the push target ⇒ KEEP (covers never-pushed branches via sentinel).
  unpushed="$(wt_unpushed_count "$wt" "$branch")"
  if [ "$unpushed" != "0" ]; then echo "unpushed=$unpushed"; return 1; fi

  # (a) secondary: branch gone on origin AND HEAD still lives on a remote ref ⇒ safe.
  if wt_branch_gone "$wt" "$branch"; then
    if wt_head_reachable_from_remote "$wt"; then echo "BRANCH-GONE"; return 0; fi
    echo "gone-but-head-local-only"; return 1
  fi

  echo "not-merged-not-gone"; return 1
}

# --- hermetic self-test (no git mutation; exercises the pure predicates) -----------------
self_test() {
  local fails=0
  _ok()   { printf '  ok   %s\n' "$1"; }
  _fail() { printf '  FAIL %s\n       %s\n' "$1" "$2"; fails=$((fails + 1)); }
  _expect_true()  { if "$@"; then _ok "$LBL"; else _fail "$LBL" "expected success, got failure"; fi; }
  _expect_false() { if "$@"; then _fail "$LBL" "expected failure, got success"; else _ok "$LBL"; fi; }
  log "running --dry-run self-test (hermetic; no git mutation)"

  local M="/home/ubuntu/sparq" R="/home/ubuntu/sparq/.claude/worktrees"

  LBL="main checkout is protected (exact)"
  _expect_true  is_protected_path "$M" "$M"
  LBL="main checkout protected despite trailing slash"
  _expect_true  is_protected_path "$M/" "$M"
  LBL="a worktree dir is NOT the protected main"
  _expect_false is_protected_path "$R/agent-x" "$M"

  LBL="worktree under root is a candidate"
  _expect_true  under_root "$R/agent-x" "$R"
  LBL="nested path under root is a candidate"
  _expect_true  under_root "$R/agent-x/sub" "$R"
  LBL="the main checkout is NOT under the worktree root"
  _expect_false under_root "$M" "$R"
  LBL="the root dir itself is not a managed worktree"
  _expect_false under_root "$R" "$R"
  LBL="a sibling dir outside root is rejected"
  _expect_false under_root "/home/ubuntu/sparq-wt-covgate" "$R"
  LBL="a path that merely shares a prefix is rejected"
  _expect_false under_root "${R}-evil/agent-x" "$R"

  echo
  if [ "$fails" -eq 0 ]; then log "self-test PASSED"; return 0; fi
  die "self-test FAILED ($fails check(s))"
}

# --- arg parsing -------------------------------------------------------------------------
APPLY=0
ROOT="$DEFAULT_ROOT"
MAIN="$DEFAULT_MAIN"
BASE="$DEFAULT_BASE"
while [ "$#" -gt 0 ]; do
  case "$1" in
    --dry-run-self-test) self_test; exit 0 ;;
    --apply)             APPLY=1 ;;
    --dry-run)           APPLY=0 ;;
    --root)              shift; [ "$#" -gt 0 ] || die "--root needs a value"; ROOT="$1" ;;
    --root=*)            ROOT="${1#--root=}" ;;
    --main)              shift; [ "$#" -gt 0 ] || die "--main needs a value"; MAIN="$1" ;;
    --main=*)            MAIN="${1#--main=}" ;;
    --base)              shift; [ "$#" -gt 0 ] || die "--base needs a value"; BASE="$1" ;;
    --base=*)            BASE="${1#--base=}" ;;
    -h|--help)           sed -n '2,60p' "$0"; exit 0 ;;
    *)                   die "unknown argument: $1 (try --apply, --root, --main, --base, --dry-run-self-test, --help)" ;;
  esac
  shift
done

ROOT="${ROOT%/}"
MAIN="${MAIN%/}"

# --- locate a git dir to drive the porcelain enumeration ---------------------------------
# We need a real git context. The main checkout is the natural one; fall back to this script's
# own worktree if --main was pointed elsewhere. We only ever READ the worktree list from it.
GIT_CTX="$MAIN"
if ! git -C "$GIT_CTX" rev-parse --git-dir >/dev/null 2>&1; then
  GIT_CTX="$(git rev-parse --show-toplevel 2>/dev/null)" \
    || die "no git repository found (tried --main='$MAIN' and CWD)"
fi
log "enumerating worktrees from git context: $GIT_CTX"
log "root=$ROOT  main(protected)=$MAIN  base(merged-ref)=$BASE  mode=$([ "$APPLY" -eq 1 ] && echo APPLY || echo dry-run)"

# Confirm the base ref resolves (a typo'd --base would make NOTHING look merged → silent
# under-removal, which is safe, but warn so the operator knows).
git -C "$GIT_CTX" rev-parse --verify -q "$BASE" >/dev/null 2>&1 \
  || log "WARNING: base ref '$BASE' does not resolve — nothing will classify as MERGED (fetch first?)."

# --- enumerate worktrees (porcelain) and classify ---------------------------------------
SAFE_DIRS=()
SAFE_REASON=()
declare -A SEEN_REASON_COUNT=()
kept=0
total_considered=0

# Parse `git worktree list --porcelain`: records separated by blank lines; we only need the
# `worktree <path>` line plus the `prunable`/`locked`/`bare`/`detached` markers.
cur_path=""; cur_prunable=0; cur_locked=0; cur_bare=0
process_record() {
  [ -n "$cur_path" ] || return 0

  # (d) HARD exclusion: never the main checkout.
  if is_protected_path "$cur_path" "$MAIN"; then
    return 0   # silently skip; it is not a candidate by construction
  fi
  # Allow-list by location: only worktrees under the harness root are candidates.
  under_root "$cur_path" "$ROOT" || return 0

  total_considered=$((total_considered + 1))

  # Structural KEEPs first (cheap, and some make git probes meaningless).
  if [ "$cur_bare" -eq 1 ];    then log "KEEP  $cur_path  (bare worktree)";              kept=$((kept+1)); return 0; fi
  if [ "$cur_locked" -eq 1 ];  then log "KEEP  $cur_path  (locked)";                     kept=$((kept+1)); return 0; fi
  if [ "$cur_prunable" -eq 1 ];then log "KEEP  $cur_path  (already prunable; run prune)";kept=$((kept+1)); return 0; fi
  if [ ! -d "$cur_path" ];     then log "KEEP  $cur_path  (path missing on disk)";       kept=$((kept+1)); return 0; fi

  local reason
  if reason="$(classify_reason "$cur_path" "$BASE")"; then
    SAFE_DIRS+=("$cur_path")
    SAFE_REASON+=("$reason")
    SEEN_REASON_COUNT["$reason"]=$(( ${SEEN_REASON_COUNT["$reason"]:-0} + 1 ))
  else
    log "KEEP  $cur_path  ($reason)"
    kept=$((kept+1))
  fi
}

while IFS= read -r line; do
  case "$line" in
    "worktree "*) cur_path="${line#worktree }" ;;
    "prunable"*)  cur_prunable=1 ;;
    "locked"*)    cur_locked=1 ;;
    "bare")       cur_bare=1 ;;
    "")           process_record
                  cur_path=""; cur_prunable=0; cur_locked=0; cur_bare=0 ;;
  esac
done < <(git -C "$GIT_CTX" worktree list --porcelain)
process_record   # flush the final record (porcelain may not end with a blank line)

# --- report ------------------------------------------------------------------------------
echo
log "considered $total_considered worktree(s) under $ROOT  (kept $kept, safe-to-remove ${#SAFE_DIRS[@]})"

if [ "${#SAFE_DIRS[@]}" -eq 0 ]; then
  log "nothing safe to remove. Done."
  exit 0
fi

log "SAFE-to-remove set (reason — path):"
for i in "${!SAFE_DIRS[@]}"; do
  printf '  %-12s %s\n' "${SAFE_REASON[$i]}" "${SAFE_DIRS[$i]}" >&2
done
log "reason breakdown:"
for r in "${!SEEN_REASON_COUNT[@]}"; do
  printf '  %-12s %d\n' "$r" "${SEEN_REASON_COUNT[$r]}" >&2
done

# Reclaimable-size estimate. du can be slow over many target/ dirs; do a single du -sc so we
# get a grand total too. Failures (a dir vanished mid-run) are non-fatal — KEEP the estimate
# best-effort rather than abort the whole sweep.
log "reclaimable-size estimate (du -sh; may take a moment over target/ dirs):"
if du -sh "${SAFE_DIRS[@]}" 2>/dev/null | sed 's/^/  /' >&2; then :; else
  log "  (du could not size some dirs; estimate is partial)"
fi
if total_size="$(du -sch "${SAFE_DIRS[@]}" 2>/dev/null | tail -n1 | cut -f1)"; then
  log "estimated TOTAL reclaimable: ${total_size:-unknown}"
fi

if [ "$APPLY" -eq 0 ]; then
  echo
  log "DRY-RUN — nothing removed. Re-run with --apply (at idle; not while sibling agents build) to remove the set above."
  exit 0
fi

# --- --apply: remove the safe set, then prune --------------------------------------------
log "--apply: removing ${#SAFE_DIRS[@]} safe worktree(s)..."
removed=0; failed=0
for wt in "${SAFE_DIRS[@]}"; do
  # Re-assert the hard exclusion at the point of mutation (defence in depth — this can never
  # fire because main is dropped during enumeration, but if it ever did we abort, not delete).
  if is_protected_path "$wt" "$MAIN"; then
    die "refusing to remove protected main checkout: $wt"
  fi
  if git -C "$GIT_CTX" worktree remove --force "$wt" 2>/dev/null; then
    log "  removed: $wt"; removed=$((removed+1))
  else
    log "  FAILED to remove (kept): $wt"; failed=$((failed+1))
  fi
done
log "pruning stale worktree administrative entries..."
git -C "$GIT_CTX" worktree prune -v 2>&1 | sed 's/^/  /' >&2 || true
log "done: removed $removed, failed $failed, of ${#SAFE_DIRS[@]} safe candidate(s)."
[ "$failed" -eq 0 ]
