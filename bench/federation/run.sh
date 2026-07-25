#!/usr/bin/env bash
# [FABLE-5] sq-hmd7l.12 — comparative FEDERATION panel entry point (FedShop-shaped).
#
# Stands up 2 LOCAL sparq-server member endpoints (a FedShop-shaped vendor +
# ratingsite shop federation, corpus generated in-repo, deterministic), puts a
# request-COUNTING reverse proxy in front of each member, and executes the same
# FedShop-shaped federated queries with three engines:
#
#   sparq    — sparq-server `--features service` (SERVICE eval + bound-join)
#   comunica — @comunica/query-sparql (reference federated-SPARQL engine;
#              gather-time npm install into bench/federation/node_modules,
#              git-ignored — NEVER a committed dependency)
#   jena     — Apache Jena `arq` naive SERVICE baseline (OPTIONAL: FED_JENA_ARQ)
#
# INVARIANT: per query, canonical RESULT-SET agreement across engines is asserted
# BEFORE any timing; per-member HTTP request counts + source-selection
# precision/recall are reported alongside wall time — never wall time alone.
#
# USAGE
#   bench/federation/run.sh --smoke   # acceptance: 2 local member endpoints, one
#                                     # FedShop-shaped query, sparq-vs-Comunica
#                                     # result-set agreement; exit 0 = green
#   bench/federation/run.sh           # full panel: 5 queries, timed iters,
#                                     # explicit + virtual regimes, JSON results
#
# TUNABLES (env; safe defaults):
#   SPARQ_SERVER_BIN   server binary (default target/release/sparq-server; MUST be
#                      built `--features service` — probed via --help)
#   FED_PORT_BASE      first port of the panel's loopback block (default 7141)
#   FED_SCALE          products in the generated corpus (default 40 smoke / 500 full)
#   FED_ITERS          timed iterations per engine per query (default 5)
#   FED_JENA_ARQ       path to Jena `arq` -> enables the naive-baseline column
#   FED_COMUNICA_SPEC  npm spec to install (default @comunica/query-sparql; the
#                      RESOLVED version is recorded in the results JSON — pin here
#                      for a canonical gather, e.g. @comunica/query-sparql@4.4.0)
#   FED_JSON_OUT       results JSON path (default, full mode only:
#                      bench/competitor-results/federation-fedshop-<UTC>.json, git-ignored)
#   FED_WORKDIR        scratch dir for corpora + server logs (default /tmp/sparq-federation-bench)
#
# Same contract as the sibling panels (bench/gsp/run.sh, scripts/*-same-box.sh):
# bounded waits everywhere, compare.py owns spawn + EXIT-safe teardown of every
# server/proxy, non-zero exit = the panel is NOT green.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

SMOKE=0
for arg in "$@"; do
  case "$arg" in
    --smoke) SMOKE=1 ;;
    *) echo "[federation] unknown arg: $arg (usage: bench/federation/run.sh [--smoke])" >&2; exit 2 ;;
  esac
done

SPARQ_SERVER_BIN="${SPARQ_SERVER_BIN:-$ROOT/target/release/sparq-server}"
# v4 major by default: v5 pulls undici@8 (Node >= 22.19) — v4 runs on Node 18/20.
# The RESOLVED version is recorded in the results JSON at every gather.
FED_COMUNICA_SPEC="${FED_COMUNICA_SPEC:-@comunica/query-sparql@^4}"

log() { printf '[federation] %s\n' "$*" >&2; }
die() { printf '[federation] ERROR: %s\n' "$*" >&2; exit 1; }

command -v python3 >/dev/null 2>&1 || die "python3 required"
command -v node >/dev/null 2>&1 || die "node required (the Comunica column; install Node.js >= 18)"
command -v npm >/dev/null 2>&1 || die "npm required (gather-time Comunica install)"

if [ ! -x "$SPARQ_SERVER_BIN" ]; then
  log "sparq-server not found at $SPARQ_SERVER_BIN"
  die "build it first: cargo build --release -p sparq-server --features service (or set SPARQ_SERVER_BIN)"
fi
# The federator needs the engine's SERVICE eval compiled in; a stock (feature-off)
# binary's --help carries no --service-allow flag — fail fast + loud, not at query time.
if ! "$SPARQ_SERVER_BIN" --help 2>&1 | grep -q -- '--service-allow'; then
  die "this sparq-server build has no SERVICE federation — rebuild: cargo build --release -p sparq-server --features service"
fi

# ---- 1. hermetic self-test of the driver (no HTTP, no node; fails fast + loud) --------
python3 "$ROOT/bench/federation/compare.py" --self-test || die "compare.py self-test failed"

# ---- 2. gather-time Comunica install (git-ignored node_modules; never committed) ------
if ! node -e "require.resolve('@comunica/query-sparql/package.json', {paths:['$ROOT/bench/federation']}); require.resolve('n3/package.json', {paths:['$ROOT/bench/federation']})" >/dev/null 2>&1; then
  log "installing $FED_COMUNICA_SPEC + n3 into bench/federation/node_modules (gather-only)"
  npm install --no-save --no-audit --no-fund --prefix "$ROOT/bench/federation" \
    "$FED_COMUNICA_SPEC" n3 >/dev/null \
    || die "npm install failed (network needed once; the dep is gather-only, never committed)"
fi

# ---- 3. drive the panel (oracle-before-timing lives in compare.py) --------------------
ARGS=( --sparq-server-bin "$SPARQ_SERVER_BIN" )
[ "$SMOKE" = 1 ] && ARGS+=( --smoke )
[ -n "${FED_SCALE:-}" ] && ARGS+=( --scale "$FED_SCALE" )
[ -n "${FED_ITERS:-}" ] && ARGS+=( --iters "$FED_ITERS" )
[ -n "${FED_JENA_ARQ:-}" ] && ARGS+=( --jena-arq "$FED_JENA_ARQ" )
[ -n "${FED_WORKDIR:-}" ] && ARGS+=( --workdir "$FED_WORKDIR" )
if [ -n "${FED_JSON_OUT:-}" ]; then
  ARGS+=( --json-out "$FED_JSON_OUT" )
elif [ "$SMOKE" = 0 ]; then
  ARGS+=( --json-out "$ROOT/bench/competitor-results/federation-fedshop-$(date -u +%Y%m%dT%H%M%SZ).json" )
fi

exec python3 "$ROOT/bench/federation/compare.py" "${ARGS[@]}"
