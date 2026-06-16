#!/usr/bin/env bash
# [OPUS-4.8] Orchestration automation — the push SCHEDULER (decision layer).
# Bead sq-o09o. Authored by Opus 4.8 (Fable unavailable; flag for re-review when
# Fable returns). Design: research/orchestration-automation-design.md §8.
#
# push-frontier.sh   (READ-ONLY; NO mutation, NO dispatch — fast, deterministic)
#
# Prints the beads that are SAFE TO LAUNCH NOW — the set the orchestrator can feed
# straight into a refill->verify fan-out without creating a conflict or exceeding
# the build-farm CPU ceiling. It is the *decision* layer that sits on top of the
# advisory `refill-candidates.sh` substrate (which only groups + flags contention).
#
# The launchable frontier is computed as:
#
#     bd ready                                    (the real unlock-frontier; deps
#                                                  must be accurate — see sq-o09o
#                                                  deliverable 2 / `bd dep`)
#   MINUS  in-flight beads                        (already has an OPEN PR — i.e. work
#                                                  is in progress; do not re-dispatch)
#   MINUS  conflict-collisions                    (emit at most ONE bead per crate /
#                                                  surface; serialise the hot lanes:
#                                                  the site, and the sparq-server
#                                                  http.rs auth path — <=1 in flight)
#   MINUS  epics / "[epic]"-tagged umbrellas        (issue_type=epic, or a title the
#                                                  maintainer tagged "[epic]", is an
#                                                  UMBRELLA not a work unit — you dispatch
#                                                  its child tasks, never "the epic";
#                                                  excluded so the frontier is launchable)
#   CAPPED at the CPU ceiling                     (min(16, nproc-2); the build farm
#                                                  cannot usefully run more parallel
#                                                  cargo builds than it has cores).
#
# Output: one line per launchable bead — "<id>  <surface>  <title>" — ordered by
# bead priority (P0 first) then id, ready to hand to the orchestrator. Anything not
# emitted is either in-flight, would collide with an already-emitted/in-flight
# surface, or fell past the CPU-ceiling cap.
#
# HONEST design notes on the in-flight signal:
#   * The AUTHORITATIVE in-flight signal is the set of OPEN PRs (`gh pr list`). A bead
#     whose id appears in an open PR's head-branch name or title is in flight.
#   * `git branch -r` is NOT a reliable in-flight signal in this repo: PRs are
#     squash-merged, so a merged feature branch does NOT become an ancestor of main
#     and `git branch -r --no-merged origin/main` flags ~all 370+ stale branches as
#     "unmerged". Using that would subtract almost everything. We therefore treat a
#     remote branch as in-flight ONLY when it also backs an OPEN PR (so it collapses
#     to the PR signal); stale merged branches are correctly ignored. This is called
#     out so a future reader does not "fix" it by trusting `--no-merged`.
#
# Conflict-partition surfaces (one in-flight at a time):
#   * Each crate short-name under crates/ is a surface (>=1 launched per crate).
#   * "site"            — the Next.js site/ tree; serialise to <=1.
#   * "server-auth"     — the sparq-server http.rs auth path (the hot rebase seam);
#                         serialise to <=1 ON TOP OF the per-crate sparq-server cap,
#                         so an auth-path bead and another sparq-server bead never
#                         both launch (they share the crate cap anyway, but the
#                         auth-path lane is named explicitly per the brief).
#
# Run:
#   scripts/push-frontier.sh                       # the launchable frontier
#   scripts/push-frontier.sh --explain             # + a per-bead reason it was kept/dropped (stderr)
#   scripts/push-frontier.sh --dry-run-self-test   # hermetic self-test (no bd/gh/git)
set -euo pipefail

PROG="push-frontier"
log()  { printf '[%s] %s\n' "$PROG" "$*" >&2; }
die()  { printf '[%s] ERROR: %s\n' "$PROG" "$*" >&2; exit 1; }

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EXPLAIN=0

# Ensure the user-local bd is reachable even under a bare PATH (cron/hook context).
case ":$PATH:" in
  *":$HOME/.local/bin:"*) ;;
  *) PATH="$PATH:$HOME/.local/bin" ;;
esac

# --- surface inference (pure-text; the hermetic, testable core) ------------------------
# Given a bead TITLE on $1 and a space-separated list of known crate short-names on $2,
# echo the inferred CONFLICT SURFACE for partitioning. Precedence:
#   1. the sparq-server http.rs AUTH path        -> "server-auth"  (hot rebase seam)
#   2. an explicit sparq-<crate> mention          -> "<crate>"
#   3. the front-end site (site:/ "site"/"page:")  -> "site"
#   4. a leading bracket tag "[tag] ..."           -> "tag"
#   5. a leading "prefix: ..."/"prefix(scope): .." -> "prefix"
#   6. otherwise                                   -> ""  (unscoped; never collides)
infer_surface() {
  local title="$1" crates="$2" c tag lc
  lc="$(printf '%s' "$title" | tr '[:upper:]' '[:lower:]')"

  # 1. The sparq-server auth path — serialise hardest. A bead is on this lane if it
  #    mentions sparq-server (or "server") AND an auth/authz/http.rs token.
  case "$lc" in
    *server*|*http.rs*)
      case "$lc" in
        *auth*|*http.rs*|*bearer*|*wac*|*acl*|*acp*)
          # but only if it's genuinely the server (not e.g. sparq-solid ACP modelling)
          case "$lc" in
            *sparq-server*|*http.rs*|*" server "*|"server "*|*" server:"*)
              printf 'server-auth\n'; return 0 ;;
          esac ;;
      esac ;;
  esac

  # 2. an explicit sparq-<crate> mention anywhere in the title (longest-first list).
  for c in $crates; do
    case "$title" in
      *"$c"*) printf '%s\n' "$c"; return 0 ;;
    esac
  done

  # 3. the front-end site surface.
  case "$lc" in
    site:*|"site "*|*" site:"*|page:*|"page "*|*/surface/*|*"site/src"*|*"tier-"*"wasm"*)
      printf 'site\n'; return 0 ;;
  esac

  # 4. a leading bracket tag: "[cert] ...", "[epic] ..." -> "cert" / "epic".
  case "$title" in
    \[*\]*)
      tag="${title#\[}"; tag="${tag%%\]*}"
      printf '%s\n' "$tag"; return 0 ;;
  esac

  # 5. a leading "prefix: ..." or "prefix(scope): ..." token (e.g. "SBOM:", "deps:").
  case "$title" in
    *:*)
      tag="${title%%:*}"
      tag="${tag%%(*}"                 # strip a "(scope)" suffix.
      case "$tag" in
        *' '*) ;;                       # has a space -> prose, not a prefix tag.
        *) printf '%s\n' "$tag"; return 0 ;;
      esac ;;
  esac

  printf '\n'
}

# --- CPU ceiling ----------------------------------------------------------------------
# The build farm cannot usefully parallelise more cargo builds than it has cores; the
# orchestrator brief uses min(16, nproc-2). nproc-2 leaves headroom for the orchestrator
# + the OS. Floor at 1 so a 1-2 core box still emits something.
cpu_ceiling() {
  local n cap
  n="$(nproc 2>/dev/null || echo 2)"
  cap=$(( n - 2 ))
  [ "$cap" -lt 1 ] && cap=1
  [ "$cap" -gt 16 ] && cap=16
  printf '%s\n' "$cap"
}

# --- self-test (no external calls) -----------------------------------------------------
self_test() {
  local fails=0 got CR
  CR="sparq-zk-compose sparq-server sparq-reason sparq-core sparq-solid sparq-hdt sparq-zk sparq-mpc"
  _check() {
    if [ "$2" = "$3" ]; then printf '  ok   %s\n' "$1"
    else printf '  FAIL %s\n       got:  %q\n       want: %q\n' "$1" "$2" "$3"; fails=$((fails + 1)); fi
  }
  log "running --dry-run self-test (hermetic; no bd/gh/git calls)"

  got="$(infer_surface 'sparq-server: HTTP integration tests for PSS named-graph query+update' "$CR")"
  _check "server (non-auth) -> crate"              "$got" "sparq-server"

  got="$(infer_surface 'sparq-server: bearer-gated auth path hardening on http.rs' "$CR")"
  _check "server auth path -> server-auth"         "$got" "server-auth"

  got="$(infer_surface 'fix(sparq-core): dict spill (sq-glw2)' "$CR")"
  _check "explicit crate mention"                  "$got" "sparq-core"

  got="$(infer_surface 'sparq-solid: support acp:issuer principal' "$CR")"
  _check "solid ACP is NOT server-auth"            "$got" "sparq-solid"

  got="$(infer_surface 'page: /surface/zk (tier-c live) + /surface/mpc walkthrough' "$CR")"
  _check "page: -> site"                           "$got" "site"

  got="$(infer_surface 'site: browser smoke test for ZK prover pre-warm' "$CR")"
  _check "site: -> site"                           "$got" "site"

  got="$(infer_surface 'tier-b wasm bundle: sparq-reason (W-reason)' "$CR")"
  _check "crate mention wins over tier-b-wasm"     "$got" "sparq-reason"

  got="$(infer_surface '[cert][cryptoreview] external audit' "$CR")"
  _check "leading bracket tag"                     "$got" "cert"

  got="$(infer_surface 'deps: hdt 0.4 -> 0.6 (#116)' "$CR")"
  _check "prefix: tag"                             "$got" "deps"

  got="$(infer_surface 'Federation TPF follow-ups: brTPF DoS cap' "$CR")"
  _check "prose-before-colon => unscoped"          "$got" ""

  echo
  if [ "$fails" -eq 0 ]; then log "self-test PASSED"; return 0; fi
  die "self-test FAILED ($fails check(s))"
}

# --- arg parsing -----------------------------------------------------------------------
for arg in "$@"; do
  case "$arg" in
    --dry-run-self-test) self_test; exit 0 ;;
    --explain)           EXPLAIN=1 ;;
    -h|--help)           sed -n '2,49p' "$0"; exit 0 ;;
    *)                   die "unknown argument: $arg (try --explain, --dry-run-self-test, --help)" ;;
  esac
done

command -v bd >/dev/null 2>&1 || die "bd not found (this script reads 'bd ready'); add ~/.local/bin to PATH"

CAP="$(cpu_ceiling)"
[ "$EXPLAIN" -eq 1 ] && log "CPU ceiling = $CAP (min(16, nproc-2); nproc=$(nproc 2>/dev/null || echo '?'))"

# Known crate short-names (from crates/), LONGEST-first so e.g. sparq-zk-compose is
# matched before sparq-zk (a substring would otherwise mis-bucket it).
CRATES=""
if [ -d "$ROOT/crates" ]; then
  for d in "$ROOT"/crates/*/; do
    [ -d "$d" ] || continue
    name="${d%/}"; name="${name##*/}"
    CRATES+="$name"$'\n'
  done
  CRATES="$(printf '%s' "$CRATES" | awk 'NF { print length, $0 }' | sort -rn | cut -d' ' -f2- | tr '\n' ' ')"
fi
[ -n "$CRATES" ] || log "WARNING: no crates/ dir — surface inference falls back to title tags only."

# --- 1. in-flight bead ids (from OPEN PRs — the authoritative signal) ------------------
# A bead is in flight if its id token appears in an open PR's head-branch name or title.
# (See the header note on why git branch -r is unreliable here.)
if command -v gh >/dev/null 2>&1; then
  # head-branch names + titles of every open PR, joined per-line for substring search.
  PR_BLOB="$(gh pr list --state open --limit 300 --json headRefName,title \
    -q '.[] | (.headRefName + " " + .title)' 2>/dev/null || true)"
else
  log "gh not found — in-flight subtraction will be EMPTY (every ready bead treated as launchable)."
  PR_BLOB=""
fi

# Is bead id $1 in flight (its id appears in any open-PR branch/title)?
is_inflight() {
  local id="$1"
  [ -n "$PR_BLOB" ] || return 1
  printf '%s\n' "$PR_BLOB" | grep -qiF -- "$id"
}

# --- 2. read bd ready ------------------------------------------------------------------
READY_JSON="$(bd ready --json 2>/dev/null)" || die "bd ready --json failed"

# Emit "id<TAB>priority<TAB>title" lines, sorted by priority asc (P0 first) then id.
# EXCLUDE epics — an epic is an umbrella, not a dispatchable work unit (see header).
ROWS="$(printf '%s' "$READY_JSON" | python3 -c '
import json,sys
data=json.load(sys.stdin)
def key(it):
    try: p=int(it.get("priority",9))
    except Exception: p=9
    return (p, it.get("id",""))
for it in sorted(data, key=key):
    # Exclude umbrellas: a real epic type, or a bead the maintainer tagged "[epic]"
    # in its title (the explicit convention for "this is a container, not a unit").
    if (it.get("issue_type") or "") == "epic":
        continue
    if (it.get("title") or "").lstrip().lower().startswith("[epic]"):
        continue
    print("\t".join([
        it.get("id",""),
        str(it.get("priority","")),
        (it.get("title","") or "").replace("\t"," ").replace("\n"," "),
    ]))
' 2>/dev/null)" || die "could not parse bd ready --json"

[ -n "$ROWS" ] || { log "no ready beads — frontier is empty."; exit 0; }

# --- 3. compute the frontier: subtract in-flight, partition surfaces, cap at CPU -------
# A surface is "taken" once a bead on it is in flight OR has been emitted. The site and
# server-auth lanes are surfaces too, so the per-surface "<=1" rule serialises them.
declare -A TAKEN          # surface -> 1 once occupied (by in-flight or emitted)
emitted=0

# Seed TAKEN with the surfaces of in-flight beads so we never emit a second bead that
# would collide with work already running.
while IFS=$'\t' read -r id prio title; do
  [ -n "$id" ] || continue
  if is_inflight "$id"; then
    surface="$(infer_surface "$title" "$CRATES")"
    [ -n "$surface" ] && TAKEN["$surface"]=1
    [ "$EXPLAIN" -eq 1 ] && log "DROP  $id  [in-flight: open PR] surface=${surface:-(unscoped)}"
  fi
done <<< "$ROWS"

# Now walk in priority order and emit launchable beads.
OUT=""
while IFS=$'\t' read -r id prio title; do
  [ -n "$id" ] || continue

  if is_inflight "$id"; then
    continue                                   # already accounted for in the seed pass.
  fi

  surface="$(infer_surface "$title" "$CRATES")"

  if [ "$emitted" -ge "$CAP" ]; then
    [ "$EXPLAIN" -eq 1 ] && log "DROP  $id  [CPU ceiling $CAP reached]"
    continue
  fi

  if [ -n "$surface" ] && [ "${TAKEN[$surface]:-0}" = "1" ]; then
    [ "$EXPLAIN" -eq 1 ] && log "DROP  $id  [surface '$surface' already taken (conflict-collision)]"
    continue
  fi

  # Launch it.
  [ -n "$surface" ] && TAKEN["$surface"]=1
  OUT+="$(printf '%s\t%s\t%s' "$id" "${surface:-(unscoped)}" "$title")"$'\n'
  emitted=$((emitted + 1))
  [ "$EXPLAIN" -eq 1 ] && log "KEEP  $id  surface=${surface:-(unscoped)}  P$prio"
done <<< "$ROWS"

# --- 4. print the launchable frontier --------------------------------------------------
if [ -z "$OUT" ]; then
  log "frontier is empty (all ready beads are in-flight or collide). CPU ceiling=$CAP."
  exit 0
fi

# Column-aligned: "<id>  <surface>  <title>".
printf '%s' "$OUT" | awk -F'\t' '
  { ids[NR]=$1; surf[NR]=$2; ttl[NR]=$3
    if (length($1)>iw) iw=length($1)
    if (length($2)>sw) sw=length($2) }
  END { for (i=1;i<=NR;i++) printf "%-*s  %-*s  %s\n", iw, ids[i], sw, surf[i], ttl[i] }
'
