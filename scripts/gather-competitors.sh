#!/usr/bin/env bash
# [OPUS-4.8] gather-competitors.sh (sq-t0c3) — authored by Opus 4.8 (Fable
# unavailable; flag for re-review when Fable returns).
#
# Orchestrates gathering competitor-engine numbers for the perf dashboard's
# side-by-side comparison, recording each competitor's VERSION + ENVIRONMENT
# alongside its results. The registry of competitors (pinned versions, recipes,
# which sparq suites each is comparable on) is bench/competitors.json.
#
# SAFE BY DEFAULT — this script DRY-RUNS unless you pass --run:
#   * dry-run (default)  : prints exactly what WOULD happen per engine + a tool
#                          availability + env report. Runs NO benchmark, pulls
#                          NO image, builds NO dataset. Always safe.
#   * --run              : actually executes the gather for the selected engines.
#                          Heavy. Caps dataset size, watches `df`, cleans /tmp.
#
# Engines: oxigraph (embedded Rust dep) | qlever (Docker) | eye (N3 binary).
# Select with --only <id[,id...]>; default is all THREE in dry-run, but --run
# requires you to NAME the engines (no accidental "run everything heavy").
#
# Usage:
#   scripts/gather-competitors.sh                      # dry-run, all engines (safe)
#   scripts/gather-competitors.sh --only oxigraph      # dry-run, one engine
#   scripts/gather-competitors.sh --run --only oxigraph [--scale N] [--iters K]
#   scripts/gather-competitors.sh --run --only eye
#   scripts/gather-competitors.sh --run --only qlever --suite olympics [--pass compute|endtoend]
#   scripts/gather-competitors.sh --list               # list registry + exit
#   scripts/gather-competitors.sh --help
#
# qlever --pass selects the compare.py pass: `compute` (engine-only, the default)
# or `endtoend` (compute + full SPARQL-JSON serialisation).
#
# What a real (--run) gather does, per engine:
#   oxigraph : cargo run -p sparq-bench --release -- --scale N --iters K
#              (the compare path already times sparq AND Oxigraph + cross-checks
#              solution counts). Records the resolved oxigraph version from
#              Cargo.lock + the host env block.
#   eye      : delegates to bench/inference/eye-comparison.sh (sparq vs EYE N3
#              closure, min-of-N wall seconds). Records `eye --version` + env.
#   qlever   : delegates to bench/qlever-<suite>/compare.py against a running
#              QLever server (Docker image). Records the resolved image DIGEST
#              (the version pin) + env. compare.py does not report a server build
#              string, so the digest IS the version identifier; --run refuses a
#              live gather when the digest cannot be resolved (no Docker / image
#              not pulled). Requires Docker + a started server; the script tells
#              you the exact prep commands rather than starting a multi-GB server.
#
# Output (only with --run): one JSON results file per engine/suite under
#   bench/competitor-results/  (GIT-IGNORED, regenerable — see bench/CATALOG.md
#   disk discipline). The dashboard SEAM (engines/values) in bench/competitors.json
#   is committed ONLY deliberately by a maintainer from a reviewed results file —
#   this script never edits the tracked JSON.
#
# Disk discipline (per bench/CATALOG.md): caps the synthetic --scale, runs a `df`
# watchdog (aborts when free space < MIN_FREE_GB), uses mktemp -d + trap-clean for
# scratch, and never deletes tracked files.

set -euo pipefail

# --- locate repo root (works from a worktree too) ----------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

REGISTRY="bench/competitors.json"
RESULTS_DIR="bench/competitor-results"

# --- defaults / knobs --------------------------------------------------------
DO_RUN=0                 # 0 = dry-run (safe default); 1 = actually run
ONLY=""                  # comma list of engine ids; empty = all
SUITE="olympics"         # which qlever-* suite (olympics|synthetic|100m)
PASS="compute"           # qlever compare.py pass (compute|endtoend)
SCALE="${SCALE:-20000}"  # oxigraph synthetic entities (capped below)
ITERS="${ITERS:-3}"      # min-of-K
MAX_SCALE="${MAX_SCALE:-200000}"   # hard cap on --scale (disk/cpu guard)
MIN_FREE_GB="${MIN_FREE_GB:-10}"   # df watchdog floor
EYE_BIN="${EYE:-eye}"
SPARQ_CLI="${SPARQ_CLI:-target/release/sparq-cli}"

usage() {
  # print the header comment block (lines 2 .. the line before `set -euo pipefail`)
  local last; last="$(grep -n '^set -euo pipefail' "${BASH_SOURCE[0]}" | head -1 | cut -d: -f1)"
  sed -n "2,$((last - 2))p" "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
  exit "${1:-0}"
}

log()  { printf '%s\n' "$*" >&2; }
hdr()  { printf '\n=== %s ===\n' "$*" >&2; }
warn() { printf 'WARN: %s\n' "$*" >&2; }
die()  { printf 'ERROR: %s\n' "$*" >&2; exit 1; }

have() { command -v "$1" >/dev/null 2>&1; }

# require_int <flag-name> <value> [min] — fail with a clear message (not a raw
# `integer expression expected` from a later `[ ... -gt ... ]`) when a numeric
# knob is empty / non-numeric (digits only — these knobs are all non-negative) /
# below a floor.
require_int() {
  local name="$1" val="$2" min="${3:-}"
  case "$val" in
    ''|*[!0-9]*) die "$name must be a non-negative integer (got '${val}')" ;;
  esac
  if [ -n "$min" ] && [ "$val" -lt "$min" ]; then
    die "$name must be >= $min (got '$val')"
  fi
}

# need_val <flag> <count-remaining> — clear error when a value-taking flag has no
# argument. Without it, the inner `shift` would consume the flag and the loop's
# trailing `shift` would then run on an empty arg list, which under `set -e`
# aborts the script SILENTLY (no message) — the same cryptic failure mode the
# numeric guards below avoid.
need_val() { [ "$2" -ge 2 ] || die "$1 requires a value (e.g. $1 <value>)"; }

# --- arg parse ---------------------------------------------------------------
while [ $# -gt 0 ]; do
  case "$1" in
    --run)    DO_RUN=1 ;;
    --only)   need_val "$1" "$#"; ONLY="$2"; shift ;;
    --only=*) ONLY="${1#*=}" ;;
    --suite)  need_val "$1" "$#"; SUITE="$2"; shift ;;
    --suite=*) SUITE="${1#*=}" ;;
    --pass)   need_val "$1" "$#"; PASS="$2"; shift ;;
    --pass=*) PASS="${1#*=}" ;;
    --scale)  need_val "$1" "$#"; SCALE="$2"; shift ;;
    --scale=*) SCALE="${1#*=}" ;;
    --iters)  need_val "$1" "$#"; ITERS="$2"; shift ;;
    --iters=*) ITERS="${1#*=}" ;;
    --list)   ACTION_LIST=1 ;;
    -h|--help) usage 0 ;;
    *) die "unknown arg: $1 (try --help)" ;;
  esac
  shift
done

# --- validate numeric knobs up front -----------------------------------------
# Without this, an empty/non-numeric value (e.g. `--scale` with no argument, or
# SCALE=abc) would reach a `[ ... -gt ... ]` and, under `set -e`, abort with a
# cryptic `integer expression expected` instead of a clear message.
require_int "--scale"   "$SCALE"       1
require_int "--iters"   "$ITERS"       1
require_int "MAX_SCALE" "$MAX_SCALE"   1
require_int "MIN_FREE_GB" "$MIN_FREE_GB" 0

# --- validate the qlever compare.py pass -------------------------------------
case "$PASS" in
  compute|endtoend) ;;
  *) die "--pass must be 'compute' or 'endtoend' (got '$PASS')" ;;
esac

# --- cap the synthetic scale (disk/cpu guard) --------------------------------
if [ "$SCALE" -gt "$MAX_SCALE" ]; then
  warn "--scale $SCALE exceeds MAX_SCALE $MAX_SCALE; capping to $MAX_SCALE (raise MAX_SCALE to override)"
  SCALE="$MAX_SCALE"
fi

# --- registry presence -------------------------------------------------------
[ -f "$REGISTRY" ] || die "registry $REGISTRY not found (run from a sparq checkout)"
if have jq; then
  jq . "$REGISTRY" >/dev/null || die "$REGISTRY does not parse as JSON"
fi

want() { # want <id> -> 0 if this engine is selected
  [ -z "$ONLY" ] && return 0
  case ",$ONLY," in *",$1,"*) return 0 ;; *) return 1 ;; esac
}

# --- df watchdog -------------------------------------------------------------
free_gb() { df -BG --output=avail . 2>/dev/null | tail -1 | tr -dc '0-9'; }
check_df() {
  local f; f="$(free_gb || true)"
  [ -n "$f" ] || { warn "could not read free disk space; skipping df guard"; return 0; }
  log "disk: ${f} GB free (floor ${MIN_FREE_GB} GB)"
  [ "$f" -ge "$MIN_FREE_GB" ] || die "free disk ${f} GB < floor ${MIN_FREE_GB} GB — aborting (per bench disk discipline)"
}

# --- host env block (recorded in every results file) -------------------------
env_json() {
  local cpu nproc_n os kernel host_class quiet
  cpu="$(LC_ALL=C grep -m1 'model name' /proc/cpuinfo 2>/dev/null | cut -d: -f2- | sed 's/^ *//' || true)"
  [ -n "$cpu" ] || cpu="$(uname -p 2>/dev/null || echo unknown)"
  nproc_n="$(nproc 2>/dev/null || echo 0)"
  os="$(uname -s 2>/dev/null || echo unknown)"
  kernel="$(uname -r 2>/dev/null || echo unknown)"
  # host_class: best-effort label — "github-runner" if GITHUB_ACTIONS, else hostname.
  if [ "${GITHUB_ACTIONS:-}" = "true" ]; then host_class="github-runner"; else host_class="$(hostname 2>/dev/null || echo local)"; fi
  # quiet_box: only EC2/dedicated quiet boxes give trustworthy absolute wall-clock.
  # Normalize to a strict JSON boolean — QUIET_BOX is emitted UNQUOTED below, so a
  # non-boolean value (e.g. QUIET_BOX=1) would otherwise produce invalid JSON.
  # Treat 1/true/yes/on (any case) as true; everything else (incl. unset) as false.
  case "$(printf '%s' "${QUIET_BOX:-}" | tr '[:upper:]' '[:lower:]')" in
    1|true|yes|on) quiet="true" ;;
    *)             quiet="false" ;;
  esac
  printf '{"host_class":"%s","cpu_model":"%s","nproc":%s,"os":"%s","kernel":"%s","quiet_box":%s,"gathered_at_utc":"%s","git_commit":"%s"}' \
    "$host_class" "$cpu" "${nproc_n:-0}" "$os" "$kernel" "$quiet" \
    "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$(git rev-parse --short HEAD 2>/dev/null || echo unknown)"
}

# --- versions per engine -----------------------------------------------------
oxigraph_version() {
  # exact resolved version from Cargo.lock; fall back to the Cargo.toml pin.
  local v=""
  if [ -f Cargo.lock ]; then
    v="$(awk '/^name = "oxigraph"$/{f=1} f&&/^version = /{gsub(/[",]/,"",$3);print $3;exit}' Cargo.lock || true)"
  fi
  [ -n "$v" ] || v="$(have jq && jq -r '.competitors[]|select(.id=="oxigraph").pinned_version' "$REGISTRY" || echo unknown)"
  printf '%s' "$v"
}
eye_version() { "$EYE_BIN" --version 2>/dev/null | head -1 || echo "not-installed"; }
qlever_image_digest() {
  have docker || { echo "docker-not-installed"; return; }
  docker inspect --format '{{index .RepoDigests 0}}' docker.io/adfreiburg/qlever:latest 2>/dev/null \
    || echo "image-not-pulled"
}

mkdir_results() { mkdir -p "$RESULTS_DIR"; }
stamp() { date -u +%Y%m%dT%H%M%SZ; }

# write a results envelope: write_result <engine> <suite> <version> <payload-json>
write_result() {
  local engine="$1" suite="$2" version="$3" payload="$4"
  mkdir_results
  local out
  out="$RESULTS_DIR/${engine}-${suite}-$(stamp).json"
  printf '{"engine":"%s","suite":"%s","version":"%s","env":%s,"result":%s}\n' \
    "$engine" "$suite" "$version" "$(env_json)" "$payload" > "$out"
  log "wrote $out"
  if have jq; then jq . "$out" >/dev/null || warn "$out did not validate as JSON"; fi
}

# =============================================================================
# --list
# =============================================================================
if [ "${ACTION_LIST:-0}" = 1 ]; then
  if have jq; then
    jq -r '.competitors[] | "\(.id)\t\(.kind)\t\(.pinned_version)\n  comparable: \([.comparable_suites[].sparq_suite]|join(", "))"' "$REGISTRY"
  else
    cat "$REGISTRY"
  fi
  exit 0
fi

# =============================================================================
# env + tooling report (always shown)
# =============================================================================
hdr "host environment"
log "$(env_json)"

hdr "tooling availability"
for t in cargo docker python3 jq "$EYE_BIN"; do
  if have "$t"; then log "  [present] $t"; else log "  [MISSING] $t"; fi
done

if [ "$DO_RUN" -eq 1 ]; then
  hdr "MODE: --run (live)  selected=${ONLY:-<none — refusing, see below>}"
  [ -n "$ONLY" ] || die "--run requires --only <id[,id...]> (refusing to run every heavy gather by default)"
  check_df
else
  hdr "MODE: dry-run (default — safe). Pass --run --only <id> to execute."
fi

# =============================================================================
# oxigraph (embedded Rust dep)
# =============================================================================
if want oxigraph; then
  hdr "oxigraph (embedded-rust-dep)"
  OXI_V="$(oxigraph_version)"
  log "  version (resolved): $OXI_V"
  log "  would run: cargo run -p sparq-bench --release -- --scale $SCALE --iters $ITERS"
  log "  comparable sparq suites: sparq-bench-compare, sp2b, watdiv, bsbm, dbpsb, lubm(extensional)"
  if [ "$DO_RUN" -eq 1 ]; then
    have cargo || die "cargo not found — cannot run the oxigraph harness"
    log "  running sparq-bench compare (scale=$SCALE iters=$ITERS)…"
    OUT="$(cargo run -p sparq-bench --release -- --scale "$SCALE" --iters "$ITERS" 2>&1)" \
      || die "sparq-bench failed:\n$OUT"
    # Store the raw harness stdout as the payload (the compare table is the result;
    # a maintainer maps it into the dashboard seam deliberately — no auto-commit).
    PAYLOAD="$(printf '%s' "$OUT" | python3 -c 'import json,sys; print(json.dumps({"raw": sys.stdin.read()}))' 2>/dev/null \
      || printf '{"raw":"see stderr"}')"
    write_result oxigraph "sparq-bench-compare" "$OXI_V" "$PAYLOAD"
    check_df
  fi
fi

# =============================================================================
# eye (N3 reasoner binary)
# =============================================================================
if want eye; then
  hdr "eye (binary — N3 reasoner)"
  EYE_V="$(eye_version)"
  log "  version: $EYE_V  (binary: $EYE_BIN)"
  log "  would run: EYE=$EYE_BIN SPARQ_CLI=$SPARQ_CLI bench/inference/eye-comparison.sh"
  log "  comparable sparq suites: inference-eye-comparison (socrates/DeepTaxonomy/anc500/grid30)"
  if [ "$DO_RUN" -eq 1 ]; then
    have "$EYE_BIN" || die "eye binary '$EYE_BIN' not found — set EYE=/path/to/eye (see bench/competitors.json install_recipe)"
    [ -x "$SPARQ_CLI" ] || die "sparq-cli not built at '$SPARQ_CLI' — run: cargo build -p sparq-cli --release"
    log "  running eye-comparison.sh (this skips EYE on dt100k unless EYE_DT100K=1)…"
    OUT="$(EYE="$EYE_BIN" SPARQ_CLI="$SPARQ_CLI" bash bench/inference/eye-comparison.sh 2>&1)" \
      || die "eye-comparison.sh failed:\n$OUT"
    PAYLOAD="$(printf '%s' "$OUT" | python3 -c 'import json,sys; print(json.dumps({"raw": sys.stdin.read()}))' 2>/dev/null \
      || printf '{"raw":"see stderr"}')"
    write_result eye "inference-eye-comparison" "$EYE_V" "$PAYLOAD"
    check_df
  fi
fi

# =============================================================================
# qlever (Docker image)
# =============================================================================
if want qlever; then
  hdr "qlever (docker-image)"
  Q_DIGEST="$(qlever_image_digest)"
  log "  image digest: $Q_DIGEST"
  log "  suite: bench/qlever-${SUITE}  (olympics|synthetic|100m)"
  log "  PREP (you run these — heavy, multi-GB, NOT auto-started here):"
  log "    cd bench/qlever-${SUITE} && qlever get-data && qlever index && qlever start"
  log "  would run: python3 bench/qlever-${SUITE}/compare.py $ITERS $PASS"
  log "  comparable sparq suites: qlever-${SUITE}, watdiv, bsbm, lubm(extensional)"
  if [ "$DO_RUN" -eq 1 ]; then
    have python3 || die "python3 not found — needed by compare.py"
    CMP="bench/qlever-${SUITE}/compare.py"
    [ -f "$CMP" ] || die "no harness for suite '$SUITE' ($CMP missing); valid: olympics|synthetic|100m"
    [ -x "$SPARQ_CLI" ] || die "sparq-cli not built at '$SPARQ_CLI' — run: cargo build -p sparq-cli --release"
    # The DIGEST is the only version identifier we can record (compare.py reports
    # no server build string), so a result with a sentinel digest would not be
    # "versioned". Refuse a live gather unless the digest resolved — fail fast
    # with the actionable prep instead of writing an unversioned result file.
    case "$Q_DIGEST" in
      docker-not-installed)
        die "qlever --run needs Docker to resolve the image digest (the version pin); install Docker, then: docker pull docker.io/adfreiburg/qlever:latest" ;;
      image-not-pulled)
        die "qlever image not pulled — cannot record a version digest. Run: docker pull docker.io/adfreiburg/qlever:latest  (then start the server per PREP above)" ;;
    esac
    # We do NOT pull/start a multi-GB QLever server automatically. compare.py needs
    # a server already running on its PORT; it fails fast (HTTP error) if not.
    warn "qlever gather assumes a QLever server is already running for suite '$SUITE'"
    warn "  (run the PREP commands above first). compare.py will error out if not."
    OUT="$(python3 "$CMP" "$ITERS" "$PASS" 2>&1)" \
      || { warn "compare.py failed (server not up? see PREP above):\n$OUT"; OUT="FAILED: $OUT"; }
    PAYLOAD="$(printf '%s' "$OUT" | python3 -c 'import json,sys; print(json.dumps({"raw": sys.stdin.read()}))' 2>/dev/null \
      || printf '{"raw":"see stderr"}')"
    write_result qlever "qlever-${SUITE}-${PASS}" "$Q_DIGEST" "$PAYLOAD"
    check_df
  fi
fi

hdr "done"
if [ "$DO_RUN" -eq 0 ]; then
  log "Dry-run only — nothing was executed. To gather for real:"
  log "  scripts/gather-competitors.sh --run --only oxigraph --scale 50000 --iters 4"
  log "  scripts/gather-competitors.sh --run --only eye"
  log "  scripts/gather-competitors.sh --run --only qlever --suite olympics --pass compute   (server must be up)"
  log "Results land (git-ignored) in $RESULTS_DIR/ ; commit a dashboard snapshot"
  log "into bench/competitors.json (engines/values) only deliberately, from a reviewed result."
fi
