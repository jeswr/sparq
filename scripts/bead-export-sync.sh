#!/usr/bin/env bash
# [SONNET-4.6] Durable bead persistence — export-sync the work-box bd DB to origin/main
# (bead sq-2xdg, issue #3290).
#
# bead-export-sync.sh [--dry-run | --apply] [--export-file F] [--branch B] [--json] [--force]
#
# WHY THIS EXISTS
# ---------------
# `.beads/issues.jsonl` is the committed source-of-record for the bead board, but NOTHING
# syncs bead CREATEs into it:
#   * agent PRs are contractually forbidden from staging `.beads/` (the shared sub-agent
#     contract: explicit-path staging, never `.beads/`) — because a worktree's copy is
#     stale and would clobber the board;
#   * `bead-autoclose.yml` (#830) is CLOSE-ONLY and, since #2475, issue-native — it closes
#     the migrated GitHub issue and never writes the JSONL at all;
#   * `reconcile-merged-beads.sh` likewise only ever CLOSES.
# So every `bd create` lands in the work-box Dolt DB and (at best) an uncommitted
# `.beads/issues.jsonl`, and the GitHub source-of-record silently rots: at the time this
# script was written the committed JSONL had not moved in ~13 days while the board kept
# growing. Two consequences: the tracker on GitHub does not reflect reality, and a work-box
# loss takes the entire delta with it. This script is the missing CREATE/UPDATE direction.
#
# It runs on the WORK BOX (the box that has `bd` + the Dolt DB) — a GitHub Actions runner
# cannot do this job, because the data only exists here. It is also why the write lands as
# a normal PR from the maintainer's `gh` credentials rather than from CI: a `GITHUB_TOKEN`
# push to protected `main` is rejected (GH013, sq-roe3) and a `GITHUB_TOKEN`-authored PR
# triggers no workflows, so `ci-summary / gate` would never report and it would hang
# unmergeable forever (both failure modes are documented in bead-autoclose.yml).
#
# DESIGN — the same discipline as reconcile-merged-beads.sh / disk-guard.sh:
#   * DEFAULT IS --dry-run: it reports the delta and mutates NOTHING (no commit, no push,
#     no PR). Publishing requires the explicit --apply flag.
#   * THE MERGE IS ADDITIVE-AND-UPDATE, NEVER DELETE. A bead present in the committed
#     JSONL but absent from the local export is KEPT verbatim. This is the load-bearing
#     safety property: a fresh, partial, or half-imported box can never erase the
#     source-of-record, which is exactly the disaster this script would otherwise enable.
#     Genuine deletions stay a manual, reviewed edit.
#   * STALE-WRITE GUARD: a fresh record replaces a committed one only when its `updated_at`
#     is strictly NEWER. An older (or timestamp-less) record never overwrites a newer
#     committed one, so a lagging box cannot roll the board back.
#   * MINIMAL DIFF: committed line ORDER is preserved and unchanged records are emitted
#     byte-identically; new records are appended sorted by id. A recurring PR therefore
#     shows only the real delta instead of a full-file reorder.
#   * FOREIGN-DB GUARD: if the local export shares less than MIN_OVERLAP_PCT of the
#     committed ids it is refused (wrong project / empty DB) unless --force.
#   * NO WORKING-TREE MUTATION: the commit is built with `hash-object` + a TEMP index +
#     `commit-tree` and pushed as a ref. The orchestrator's checkout, index, branch and
#     `.beads/` are never touched — this script is safe to run while agents are working.
#   * ONE ROLLING PR: a fixed branch, rebuilt on top of the CURRENT origin/main each run
#     and force-pushed, so it never conflicts and never accumulates stale PRs.
#   * IDEMPOTENT: when the merge produces no change, it is a logged no-op.
#
# The PR title deliberately carries NO `sq-XXXX` token: `bead-autoclose.yml` extracts bead
# tokens from the merged PR title and would close sq-2xdg (and any bead named there) on
# every single sync. Bead ids belong in the PR BODY, which autoclose does not read.
#
# Run:
#   scripts/bead-export-sync.sh                    # dry-run (default): report the delta
#   scripts/bead-export-sync.sh --apply            # push the rolling branch + open/refresh the PR
#   scripts/bead-export-sync.sh --json             # machine-readable delta for a scheduler
#   scripts/bead-export-sync.sh --dry-run-self-test   # hermetic self-test (no bd/gh/git/net)
set -uo pipefail # NOT -e: a maintenance tick is best-effort and must not abort its caller.

PROG="bead-export-sync"
log() { printf '[%s] %s\n' "$PROG" "$*" >&2; }
die() {
  printf '[%s] ERROR: %s\n' "$PROG" "$*" >&2
  exit 1
}

# Ensure the user-local bd is reachable even under a bare PATH (cron/hook context).
case ":$PATH:" in
*":$HOME/.local/bin:"*) ;;
*) PATH="$PATH:$HOME/.local/bin" ;;
esac

JSONL_REL=".beads/issues.jsonl"
BRANCH="chore/bead-export-sync"
BASE_REF="origin/main"
BASE_BRANCH="main"
# A local export sharing less than this share of the committed board is treated as a
# wrong/partial database rather than a legitimate delta.
MIN_OVERLAP_PCT=50

# --------------------------------------------------------------------------------------
# The merge core (pure: two files in, one file out). Additive-and-update, never delete.
# Prints one `k=v` summary line: added/updated/kept/stale/missing/unkeyed/overlap/total.
# Exits 3 on unparseable input (fail CLOSED — a corrupt export must never be published).
# --------------------------------------------------------------------------------------
merge_export() { # merge_export <committed.jsonl> <fresh.jsonl> <out.jsonl>
  python3 - "$1" "$2" "$3" <<'PY'
import json
import sys

committed_path, fresh_path, out_path = sys.argv[1], sys.argv[2], sys.argv[3]


def load(path):
    """[(id_or_None, raw_line, obj)] in file order. Raw lines are preserved BYTE-EXACT so
    an unchanged record re-emits identically and the PR diff stays minimal."""
    out = []
    with open(path, encoding="utf-8") as fh:
        for lineno, line in enumerate(fh, 1):
            raw = line.rstrip("\n")
            if not raw.strip():
                continue
            try:
                obj = json.loads(raw)
            except ValueError as exc:
                sys.stderr.write("parse error: %s:%d: %s\n" % (path, lineno, exc))
                raise SystemExit(3)
            if not isinstance(obj, dict):
                sys.stderr.write("parse error: %s:%d: record is not an object\n" % (path, lineno))
                raise SystemExit(3)
            out.append((obj.get("id"), raw, obj))
    return out


def newer(fresh_obj, committed_obj):
    """True iff the fresh record is STRICTLY newer. Timestamps are RFC3339-UTC in the bd
    export, so a lexicographic compare is a chronological compare. A missing timestamp on
    either side is 'unknown', and unknown NEVER wins — a lagging or timestamp-less box must
    not roll a newer committed record back."""
    f, c = fresh_obj.get("updated_at"), committed_obj.get("updated_at")
    if not isinstance(f, str) or not isinstance(c, str) or not f or not c:
        return False
    return f > c


committed = load(committed_path)
fresh = load(fresh_path)

fresh_by_id, unkeyed = {}, 0
for rid, raw, obj in fresh:
    if rid is None:
        unkeyed += 1
        continue
    fresh_by_id[rid] = (raw, obj)

merged, seen = [], set()
updated = kept = stale = missing = 0
for rid, raw, obj in committed:
    if rid is None:
        merged.append(raw)  # id-less record: pass through untouched
        continue
    seen.add(rid)
    hit = fresh_by_id.get(rid)
    if hit is None:
        # PRESENT ON MAIN, ABSENT LOCALLY -> keep. The no-delete invariant.
        merged.append(raw)
        missing += 1
        continue
    f_raw, f_obj = hit
    if f_raw == raw:
        merged.append(raw)
        kept += 1
    elif newer(f_obj, obj):
        merged.append(f_raw)
        updated += 1
    else:
        merged.append(raw)  # stale/unknown fresh record: committed wins
        stale += 1

# New beads append at the end, sorted by id so the output is deterministic regardless of
# the order `bd export` happened to emit them in.
added_ids = sorted(rid for rid in fresh_by_id if rid not in seen)
for rid in added_ids:
    merged.append(fresh_by_id[rid][0])

committed_ids = seen
overlap = 100
if committed_ids:
    overlap = int(100 * len(committed_ids & set(fresh_by_id)) / len(committed_ids))

with open(out_path, "w", encoding="utf-8") as fh:
    for raw in merged:
        fh.write(raw + "\n")

sys.stdout.write(
    "added=%d updated=%d kept=%d stale=%d missing=%d unkeyed=%d overlap=%d total=%d\n"
    % (len(added_ids), updated, kept, stale, missing, unkeyed, overlap, len(merged))
)
sys.stdout.write("added_ids=%s\n" % ",".join(added_ids[:20]))
PY
}

# The rolling PR title. MUST NOT contain an `sq-XXXX` token — see the header note about
# bead-autoclose.yml closing every bead named in a merged PR title.
pr_title() { # pr_title <added> <updated>
  printf 'chore(beads): export-sync the bead board (%s new, %s updated)' "$1" "$2"
}

# A branch we are willing to FORCE-push. Only the dedicated rolling branch qualifies.
is_safe_force_branch() {
  case "${1-}" in
  "" | main | master | HEAD | */main | */master) return 1 ;;
  *) return 0 ;;
  esac
}

# --------------------------------------------------------------------------------------
# Hermetic self-test — the merge invariants + the two guard predicates. No bd/gh/git/net.
# --------------------------------------------------------------------------------------
self_test() {
  local fails=0 tmp out summary
  _check() { # _check "<label>" "<got>" "<want>"
    if [ "$2" = "$3" ]; then
      printf '  ok   %s\n' "$1"
    else
      printf '  FAIL %s\n       got:  %q\n       want: %q\n' "$1" "$2" "$3"
      fails=$((fails + 1))
    fi
  }
  _contains() { # _contains "<label>" "<haystack-file>" "<needle>"
    if grep -qF -- "$3" "$2"; then printf '  ok   %s\n' "$1"; else
      printf '  FAIL %s (expected output to CONTAIN %q)\n' "$1" "$3"
      fails=$((fails + 1))
    fi
  }
  log "running --dry-run self-test (hermetic; no bd/gh/git calls)"
  command -v python3 >/dev/null 2>&1 || die "python3 not found (required by the merge core)"

  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' RETURN
  out="${tmp}/out.jsonl"

  rec() { # rec <id> <updated_at> <title>
    printf '{"_type":"issue","id":"%s","title":"%s","status":"open","updated_at":"%s"}\n' "$1" "$3" "$2"
  }

  # committed board: two beads.
  {
    rec sq-aaa 2026-07-01T00:00:00Z "alpha"
    rec sq-bbb 2026-07-01T00:00:00Z "beta"
  } >"${tmp}/committed.jsonl"

  # 1. a NEW bead is added; 2. a bead missing from the export is KEPT (no-delete);
  # 3. a NEWER record updates; 4. an OLDER record does NOT.
  {
    rec sq-aaa 2026-07-09T00:00:00Z "alpha renamed" # newer  -> updates
    rec sq-ccc 2026-07-09T00:00:00Z "gamma"         # new    -> appended
  } >"${tmp}/fresh.jsonl"
  summary="$(merge_export "${tmp}/committed.jsonl" "${tmp}/fresh.jsonl" "$out" | head -1)"
  _check "new+update+missing summary" "$summary" \
    "added=1 updated=1 kept=0 stale=0 missing=1 unkeyed=0 overlap=50 total=3"
  _contains "the updated record wins" "$out" '"title":"alpha renamed"'
  _contains "the bead absent from the export is KEPT (no-delete)" "$out" '"id":"sq-bbb"'
  _contains "the new bead is appended" "$out" '"id":"sq-ccc"'

  # A STALE export must not roll the board back.
  {
    rec sq-aaa 2026-06-01T00:00:00Z "alpha OLD"
    rec sq-bbb 2026-07-01T00:00:00Z "beta"
  } >"${tmp}/stale.jsonl"
  summary="$(merge_export "${tmp}/committed.jsonl" "${tmp}/stale.jsonl" "$out" | head -1)"
  _check "older updated_at never overwrites" "$summary" \
    "added=0 updated=0 kept=1 stale=1 missing=0 unkeyed=0 overlap=100 total=2"
  _contains "the committed record survives a stale export" "$out" '"title":"alpha"'

  # Idempotency: merging the committed board with itself is byte-identical.
  merge_export "${tmp}/committed.jsonl" "${tmp}/committed.jsonl" "$out" >/dev/null
  if cmp -s "${tmp}/committed.jsonl" "$out"; then printf '  ok   identical input => byte-identical output\n'; else
    printf '  FAIL identical input produced a diff\n'
    fails=$((fails + 1))
  fi

  # A FOREIGN export shares no ids -> overlap 0 (the caller refuses on this).
  rec sq-zzz 2026-07-09T00:00:00Z "not our board" >"${tmp}/foreign.jsonl"
  summary="$(merge_export "${tmp}/committed.jsonl" "${tmp}/foreign.jsonl" "$out" | head -1)"
  _check "foreign export reports overlap=0" "${summary##*overlap=}" "0 total=3"

  # A CORRUPT export must fail CLOSED (never publish a half-parsed board).
  local rc
  printf '{"_type":"issue","id":"sq-aaa"\n' >"${tmp}/corrupt.jsonl"
  merge_export "${tmp}/committed.jsonl" "${tmp}/corrupt.jsonl" "$out" >/dev/null 2>&1
  rc=$?
  _check "unparseable export exits 3 (fail closed)" "$rc" "3"

  # The PR title must carry NO bead token, or bead-autoclose closes beads on every sync.
  if pr_title 4 7 | grep -qE 'sq-[a-z0-9]'; then
    printf '  FAIL PR title contains an sq- token (bead-autoclose would fire)\n'
    fails=$((fails + 1))
  else
    printf '  ok   PR title carries no sq- token\n'
  fi

  # The force-push guard.
  if is_safe_force_branch "chore/bead-export-sync"; then
    printf '  ok   rolling branch is force-safe\n'
  else
    printf '  FAIL rolling branch rejected by the force guard\n'
    fails=$((fails + 1))
  fi
  for bad in main master HEAD "" "origin/main"; do
    if is_safe_force_branch "$bad"; then
      printf '  FAIL force guard accepted %q\n' "$bad"
      fails=$((fails + 1))
    else printf '  ok   force guard rejects %q\n' "$bad"; fi
  done

  echo
  if [ "$fails" -eq 0 ]; then
    log "self-test PASSED"
    return 0
  fi
  die "self-test FAILED ($fails check(s))"
}

# --------------------------------------------------------------------------------------
# Arg parsing
# --------------------------------------------------------------------------------------
APPLY=0
JSON=0
FORCE=0
EXPORT_FILE=""
while [ "$#" -gt 0 ]; do
  case "$1" in
  --dry-run-self-test)
    self_test
    exit 0
    ;;
  --apply) APPLY=1 ;;
  --dry-run) APPLY=0 ;;
  --json) JSON=1 ;;
  --force) FORCE=1 ;;
  --export-file)
    shift
    [ "$#" -gt 0 ] || die "--export-file needs a path"
    EXPORT_FILE="$1"
    ;;
  --branch)
    shift
    [ "$#" -gt 0 ] || die "--branch needs a name"
    BRANCH="$1"
    ;;
  -h | --help)
    sed -n '2,60p' "$0"
    exit 0
    ;;
  *) die "unknown argument: $1 (try --apply, --dry-run, --json, --force, --dry-run-self-test, --help)" ;;
  esac
  shift
done

command -v python3 >/dev/null 2>&1 || die "python3 not found (required by the merge core)"
command -v git >/dev/null 2>&1 || die "git not found"

ROOT="$(git rev-parse --show-toplevel 2>/dev/null)"
[ -n "$ROOT" ] || die "not inside a git checkout"

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

# --------------------------------------------------------------------------------------
# 1. The FRESH export (the work-box truth).
# --------------------------------------------------------------------------------------
FRESH="${WORK}/fresh.jsonl"
if [ -n "$EXPORT_FILE" ]; then
  [ -f "$EXPORT_FILE" ] || die "--export-file not found: $EXPORT_FILE"
  cp -- "$EXPORT_FILE" "$FRESH"
else
  if ! command -v bd >/dev/null 2>&1; then
    # Inert off the work box: only the box with the Dolt DB can produce the export, and a
    # guess is worse than a no-op. Exit 0 so a maintenance tick is not reddened by this.
    log "bd not found — this sync only runs on the work box (the box holding the bead DB). No-op."
    exit 0
  fi
  if ! bd export >"$FRESH" 2>"${WORK}/bd.err"; then
    log "bd export failed — nothing published. stderr:"
    sed -n '1,20p' "${WORK}/bd.err" >&2
    exit 1
  fi
fi
[ -s "$FRESH" ] || die "the bead export is EMPTY — refusing to publish (fail closed)"

# --------------------------------------------------------------------------------------
# 2. The COMMITTED board, read from origin/main (NOT the working tree — a local edit must
#    never be mistaken for the published source-of-record).
# --------------------------------------------------------------------------------------
git -C "$ROOT" fetch --quiet origin "$BASE_BRANCH" 2>/dev/null ||
  log "warning: could not fetch origin/$BASE_BRANCH — using the local ref"
BASE_SHA="$(git -C "$ROOT" rev-parse "$BASE_REF" 2>/dev/null)"
[ -n "$BASE_SHA" ] || die "cannot resolve $BASE_REF"

COMMITTED="${WORK}/committed.jsonl"
if ! git -C "$ROOT" show "${BASE_SHA}:${JSONL_REL}" >"$COMMITTED" 2>/dev/null; then
  die "$JSONL_REL is absent from $BASE_REF — refusing to invent the source-of-record"
fi

# --------------------------------------------------------------------------------------
# 3. Merge (additive-and-update, never delete) + the guards.
# --------------------------------------------------------------------------------------
MERGED="${WORK}/merged.jsonl"
SUMMARY="$(merge_export "$COMMITTED" "$FRESH" "$MERGED")" || die "merge failed — nothing published"
COUNTS="$(printf '%s\n' "$SUMMARY" | head -1)"
ADDED_IDS="$(printf '%s\n' "$SUMMARY" | grep '^added_ids=' | cut -d= -f2-)"

# Parse the k=v summary into locals.
get() { printf '%s\n' "$COUNTS" | tr ' ' '\n' | grep "^$1=" | cut -d= -f2-; }
N_ADDED="$(get added)"
N_UPDATED="$(get updated)"
N_STALE="$(get stale)"
N_MISSING="$(get missing)"
OVERLAP="$(get overlap)"

if [ "${OVERLAP:-0}" -lt "$MIN_OVERLAP_PCT" ] && [ "$FORCE" -eq 0 ]; then
  die "the local export shares only ${OVERLAP}% of the committed board (< ${MIN_OVERLAP_PCT}%) — \
this looks like a partial or foreign database, not a delta. Nothing published. Re-run with --force if intended."
fi

CHANGED=1
cmp -s "$COMMITTED" "$MERGED" && CHANGED=0

log "delta vs ${BASE_REF}: ${N_ADDED} new, ${N_UPDATED} updated, ${N_MISSING} on-main-but-absent-locally (kept), ${N_STALE} stale-ignored, overlap ${OVERLAP}%"
[ -n "$ADDED_IDS" ] && log "new beads (first 20): ${ADDED_IDS}"
[ "$N_STALE" != "0" ] && log "note: ${N_STALE} local record(s) were OLDER than the committed copy and were ignored (stale-write guard)"
[ "$N_MISSING" != "0" ] && log "note: ${N_MISSING} committed bead(s) are absent from this box's DB — KEPT (this sync never deletes)"

json_bool() {
  if [ "${1-0}" -eq 1 ]; then printf 'true'; else printf 'false'; fi
}

emit_json() { # emit_json <pr-url-or-empty>
  printf '{"added":%s,"updated":%s,"stale":%s,"missing":%s,"overlap":%s,"changed":%s,"applied":%s,"branch":"%s","pr":"%s"}\n' \
    "${N_ADDED:-0}" "${N_UPDATED:-0}" "${N_STALE:-0}" "${N_MISSING:-0}" "${OVERLAP:-0}" \
    "$(json_bool "$CHANGED")" "$(json_bool "$APPLY")" "$BRANCH" "${1-}"
}

if [ "$CHANGED" -eq 0 ]; then
  log "the committed board is already up to date — no-op."
  [ "$JSON" -eq 1 ] && emit_json ""
  exit 0
fi

if [ "$APPLY" -eq 0 ]; then
  log "DRY-RUN: would push '${BRANCH}' (on top of ${BASE_SHA:0:12}) and open/refresh a PR. Re-run with --apply."
  [ "$JSON" -eq 1 ] && emit_json ""
  exit 0
fi

# --------------------------------------------------------------------------------------
# 4. Publish. The commit is BUILT, never checked out: hash-object + a temp index +
#    commit-tree, then a ref push. The caller's working tree, index and branch are
#    untouched, so this is safe to run while agents are building in this checkout.
# --------------------------------------------------------------------------------------
is_safe_force_branch "$BRANCH" || die "refusing to force-push branch '${BRANCH}'"
command -v gh >/dev/null 2>&1 || die "gh CLI not found (needed to open the export-sync PR)"

BLOB="$(git -C "$ROOT" hash-object -w -- "$MERGED")" || die "hash-object failed"
export GIT_INDEX_FILE="${WORK}/index"
git -C "$ROOT" read-tree "$BASE_SHA" || die "read-tree failed"
git -C "$ROOT" update-index --add --cacheinfo "100644,${BLOB},${JSONL_REL}" || die "update-index failed"
TREE="$(git -C "$ROOT" write-tree)" || die "write-tree failed"
unset GIT_INDEX_FILE

MSG="$(pr_title "$N_ADDED" "$N_UPDATED")

Export-sync of the work-box bead database into the committed source-of-record
($JSONL_REL). Generated by scripts/bead-export-sync.sh --apply.

new: ${N_ADDED}  updated: ${N_UPDATED}  kept-because-absent-locally: ${N_MISSING}  stale-ignored: ${N_STALE}"

COMMIT="$(git -C "$ROOT" commit-tree "$TREE" -p "$BASE_SHA" -m "$MSG")" ||
  die "commit-tree failed (is git user.name/user.email configured?)"

git -C "$ROOT" push --force origin "${COMMIT}:refs/heads/${BRANCH}" ||
  die "push failed — nothing was opened"
log "pushed ${COMMIT:0:12} to ${BRANCH}"

PR_URL="$(gh pr list --head "$BRANCH" --state open --json url --jq '.[0].url' 2>/dev/null)"
if [ -n "$PR_URL" ] && [ "$PR_URL" != "null" ]; then
  log "refreshed the open export-sync PR: ${PR_URL}"
else
  BODY="> 🤖 SPARQ automation — \`scripts/bead-export-sync.sh\` (bead sq-2xdg / issue #3290)

Durable bead persistence: bead CREATEs live only in the work-box bd database until
something syncs them to \`main\`. Agent PRs never stage \`.beads/\`, and \`bead-autoclose\`
is close-only — so this is the missing create/update direction.

| | |
|---|---|
| new beads | ${N_ADDED} |
| updated | ${N_UPDATED} |
| on main but absent from this box (kept) | ${N_MISSING} |
| stale local records ignored | ${N_STALE} |
| id overlap with main | ${OVERLAP}% |

New bead ids (first 20): ${ADDED_IDS:-none}

This branch is rebuilt on top of the current \`main\` on every run, so it never conflicts.
The merge is **additive-and-update, never delete**, and a local record older than the
committed one is ignored — a partial or lagging box cannot erase or roll back the board."
  PR_URL="$(gh pr create --base "$BASE_BRANCH" --head "$BRANCH" \
    --title "$(pr_title "$N_ADDED" "$N_UPDATED")" --body "$BODY" 2>&1)" ||
    die "gh pr create failed: ${PR_URL}"
  PR_URL="$(printf '%s\n' "$PR_URL" | tail -1)"
  log "opened the export-sync PR: ${PR_URL}"
fi

[ "$JSON" -eq 1 ] && emit_json "$PR_URL"
exit 0
