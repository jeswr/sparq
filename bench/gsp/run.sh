#!/usr/bin/env bash
# [FABLE-5] sq-hmd7l.21 — Graph Store Protocol same-box panel entry point.
#
# Stands up the REAL sparq-server on a loopback port (empty default graph, no
# auth = writable) and drives its GSP surface through bench/gsp/compare.py:
# PUT/GET/DELETE round-trips over a resource-size sweep, the PSS LDP-CRUD
# stream projected onto GSP verbs, and the PATCH-dialect panel (sq-hmd7l.36:
# application/sparql-update always; text/n3 Solid N3-Patch when opted in — see
# GSP_N3_PATCH below). The correctness gates run BEFORE any timing is reported
# (compare.py exits non-zero on any gate failure; an engine that does not
# implement a PATCH dialect records an honest n/a row instead).
#
# Same lineage/contract as scripts/sparq-server-same-box.sh: bounded waits
# everywhere, an EXIT trap that always stops the server, non-zero exit = the
# panel is NOT green.
#
# USAGE
#   bench/gsp/run.sh --smoke     # acceptance: sparq loopback only, small sizes,
#                                # hermetic self-test + round-trip gate; exit 0 = green
#   bench/gsp/run.sh             # full sweep (1 KB..10 MB); competitor columns via env
#
# Competitor columns (each must already be RUNNING on a loopback port; start
# them with scripts/fuseki-same-box.sh lineage / `oxigraph serve` /
# `npx @solid/community-server` — see bench/gsp/README.md):
#   GSP_FUSEKI_URL     e.g. http://127.0.0.1:3030/ds     (Fuseki dataset base)
#   GSP_OXIGRAPH_URL   e.g. http://127.0.0.1:7878        (Oxigraph server base)
#   GSP_CSS_URL        e.g. http://127.0.0.1:3000        (CSS pod base — LOOSE column)
#
# TUNABLES (env; safe defaults):
#   SPARQ_SERVER_BIN    server binary        (default target/release/sparq-server)
#   GSP_PORT            sparq HTTP port      (default 7033)
#   GSP_READY_TIMEOUT   readiness cap, s     (default 60)
#   GSP_ITERS           timed iters per size (default: compare.py's per-mode default)
#   GSP_SIZES           CSV byte sizes       (default: compare.py's per-mode default)
#   GSP_PATCH_OPS       PATCH stream length  (default: compare.py's per-mode default)
#   GSP_N3_PATCH        1 = enable sparq's OPT-IN text/n3 Solid N3-Patch dialect
#                       (exports SPARQ_N3_PATCH=1 to the spawned server; the binary
#                       must be built with `--features n3-patch` — a stock build
#                       ignores the env var and the text/n3 row honestly reads
#                       n/a via 415). Default 0: the panel measures a stock build.
#   GSP_JSON_OUT        JSON results path    (default: none; git-ignored dir suggested:
#                                             bench/competitor-results/)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

SMOKE=0
for arg in "$@"; do
  case "$arg" in
    --smoke) SMOKE=1 ;;
    *) echo "[gsp] unknown arg: $arg (usage: bench/gsp/run.sh [--smoke])" >&2; exit 2 ;;
  esac
done

SPARQ_SERVER_BIN="${SPARQ_SERVER_BIN:-$ROOT/target/release/sparq-server}"
GSP_PORT="${GSP_PORT:-7033}"
GSP_READY_TIMEOUT="${GSP_READY_TIMEOUT:-60}"
# The server's default --max-body-bytes is 1 MiB — deliberately raised here to
# clear the sweep's 10 MB PUT bodies (the competitor servers must be configured
# equivalently; Fuseki/Oxigraph have no such low default cap).
GSP_MAX_BODY_BYTES="${GSP_MAX_BODY_BYTES:-67108864}"

log() { printf '[gsp] %s\n' "$*" >&2; }
die() { printf '[gsp] ERROR: %s\n' "$*" >&2; exit 1; }

command -v python3 >/dev/null 2>&1 || die "python3 required"
if [ ! -x "$SPARQ_SERVER_BIN" ]; then
  log "sparq-server not found at $SPARQ_SERVER_BIN"
  die "build it first: cargo build -p sparq-server --release (or set SPARQ_SERVER_BIN)"
fi

# ---- 1. hermetic self-test of the driver (no HTTP; fails fast + loud) ------------------
python3 "$ROOT/bench/gsp/compare.py" --self-test || die "compare.py self-test failed"

# ---- 2. spawn sparq-server, empty + writable, loopback only ----------------------------
SRV_PID=""
SRV_LOG="$(mktemp "${TMPDIR:-/tmp}/sparq-gsp.XXXXXX.log")"
cleanup() {
  set +e
  [ -n "$SRV_PID" ] && kill "$SRV_PID" >/dev/null 2>&1
  rm -f "$SRV_LOG" >/dev/null 2>&1
  log "teardown: stopped sparq-server (if any)"
}
trap cleanup EXIT
trap 'exit 130' INT TERM

# Opt-in Solid N3-Patch dialect (sq-hmd7l.36): the env route (not --n3-patch)
# so a stock feature-off binary starts cleanly and its text/n3 row reads n/a.
if [ "${GSP_N3_PATCH:-0}" = 1 ]; then
  export SPARQ_N3_PATCH=1
  log "N3-Patch runtime opt-in requested (needs a --features n3-patch build)"
fi

log "starting sparq-server on 127.0.0.1:$GSP_PORT (empty graph, GSP writable, max-body-bytes=$GSP_MAX_BODY_BYTES)"
"$SPARQ_SERVER_BIN" --addr "127.0.0.1:${GSP_PORT}" \
  --max-body-bytes "$GSP_MAX_BODY_BYTES" >"$SRV_LOG" 2>&1 &
SRV_PID=$!

ready=0
poll_n=$(( GSP_READY_TIMEOUT * 2 )); [ "$poll_n" -ge 1 ] || poll_n=1
for _ in $(seq 1 "$poll_n"); do
  if ! kill -0 "$SRV_PID" >/dev/null 2>&1; then
    log "sparq-server exited during startup — last log:"; tail -20 "$SRV_LOG" >&2 || true
    die "sparq-server did not stay up"
  fi
  if curl -fsS --max-time 5 "http://127.0.0.1:${GSP_PORT}/health" >/dev/null 2>&1; then
    ready=1; break
  fi
  sleep 0.5
done
[ "$ready" = 1 ] || die "server not ready within ${GSP_READY_TIMEOUT}s — aborting (no hang)"
log "server ready at http://127.0.0.1:${GSP_PORT}"

# ---- 3. drive the panel (gate-before-timing lives in compare.py) -----------------------
ARGS=( --sparq "http://127.0.0.1:${GSP_PORT}" )
[ "$SMOKE" = 1 ] && ARGS+=( --smoke )
[ -n "${GSP_ITERS:-}" ] && ARGS+=( --iters "$GSP_ITERS" )
[ -n "${GSP_SIZES:-}" ] && ARGS+=( --sizes "$GSP_SIZES" )
[ -n "${GSP_PATCH_OPS:-}" ] && ARGS+=( --patch-ops "$GSP_PATCH_OPS" )
[ -n "${GSP_FUSEKI_URL:-}" ] && ARGS+=( --fuseki "$GSP_FUSEKI_URL" )
[ -n "${GSP_OXIGRAPH_URL:-}" ] && ARGS+=( --oxigraph "$GSP_OXIGRAPH_URL" )
[ -n "${GSP_CSS_URL:-}" ] && ARGS+=( --css "$GSP_CSS_URL" )
[ -n "${GSP_JSON_OUT:-}" ] && ARGS+=( --json-out "$GSP_JSON_OUT" )

python3 "$ROOT/bench/gsp/compare.py" "${ARGS[@]}"
log "panel green (all round-trip gates passed)"
