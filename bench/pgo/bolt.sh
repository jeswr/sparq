#!/usr/bin/env bash
# [FABLE-5] sq-98w7z.4 — OPTIONAL BOLT stage on top of the PGO evaluation (run.sh).
#
# Per the bead: stack BOLT ONLY if PGO alone shows >= 3% (the run.sh report writes that
# verdict to results/summary.json; this script enforces it — override with --force for
# exploration, the report still labels every number honestly).
#
# Pipeline (the standard llvm-bolt recipe):
#   1. rebuild the PGO binaries with relocations kept (-Clink-arg=-Wl,--emit-relocs);
#      an --emit-relocs link changes layout metadata, not generated code, so this build
#      stands in for 'pgo' as BOLT's input
#   2. profile them with `perf record` on the same workloads run.sh trains on
#      (LBR `-j any,u` when the box supports it; plain cycles + perf2bolt -nl fallback)
#   3. perf2bolt -> llvm-bolt (ext-tsp block reorder, hfsort function reorder,
#      function splitting, ICF)
#   4. measure via `run.sh measure-variant pgo-bolt` (the IDENTICAL measurement code path
#      as baseline/pgo) and regenerate the report, which then includes the pgo-bolt column
#
# Exit codes: 0 = ran (or gate says BOLT not warranted); 2 = environment cannot run BOLT
# (missing llvm-bolt/perf2bolt/perf, or perf events unavailable — common in containers;
# run on the canonical bench box instead).
#
# EVALUATION ONLY — no shipped-artifact change; everything under $PGO_SCRATCH, git-ignored.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
SCRATCH="${PGO_SCRATCH:-/tmp/sparq-pgo}"
TRIPLE="$(rustc -vV | sed -n 's/^host: //p')"
MERGED="$SCRATCH/merged.profdata"
RESULTS="$SCRATCH/results"
PGO_PORT="${PGO_PORT:-3671}"
PGO_INGEST_SF="${PGO_INGEST_SF:-30}"
WATDIV_SF="${WATDIV_SF:-1}"
SP2B_TRIPLES="${SP2B_TRIPLES:-250000}"
BSBM_PC="${BSBM_PC:-300}"

log() { echo "[bolt] $*" >&2; }
die() { echo "[bolt] FATAL: $*" >&2; exit 1; }
skip() { echo "[bolt] SKIP: $*" >&2; exit 2; }

FORCE=0
[ "${1:-}" = "--force" ] && FORCE=1

# ---- gate: PGO >= 3% (from run.sh report) ------------------------------------------------
SUMMARY="$RESULTS/summary.json"
[ -s "$SUMMARY" ] || die "no $SUMMARY — run bench/pgo/run.sh (through the report phase) first"
GM="$(python3 -c "import json,sys; print(json.load(open('$SUMMARY'))['variants'].get('pgo',{}).get('query_geomean_pct','NA'))")"
[ "$GM" != "NA" ] || die "summary.json has no pgo query_geomean_pct — rerun run.sh report"
if [ "$FORCE" != 1 ]; then
  python3 -c "import sys; sys.exit(0 if float('$GM') >= 3.0 else 1)" || {
    log "PGO query geomean is ${GM}% (< 3%) — per the bead gate, BOLT is not warranted. (--force to explore anyway)"
    exit 0
  }
fi
log "PGO query geomean ${GM}% — proceeding with BOLT"

# ---- environment checks ------------------------------------------------------------------
command -v llvm-bolt  >/dev/null 2>&1 || skip "llvm-bolt not on PATH (apt: llvm-bolt / package name varies per distro)"
command -v perf2bolt  >/dev/null 2>&1 || skip "perf2bolt not on PATH (ships with llvm-bolt)"
command -v perf       >/dev/null 2>&1 || skip "perf not on PATH (apt: linux-tools-common + linux-tools-\$(uname -r))"
[ -s "$MERGED" ] || die "no $MERGED — run the PGO train+merge phases first"

PROBE="$SCRATCH/perf-probe.data"
LBR=""
if perf record -e cycles:u -j any,u -o "$PROBE" -- true >/dev/null 2>&1; then
  LBR="-j any,u"; log "LBR branch sampling available"
elif perf record -e cycles:u -o "$PROBE" -- true >/dev/null 2>&1; then
  log "no LBR on this box — plain cycles sampling + perf2bolt -nl (lower-quality profile)"
else
  skip "perf record cannot sample on this box (perf_event_paranoid / container limits) — use the canonical bench box"
fi
rm -f "$PROBE"

# ---- 1. PGO build with relocations kept --------------------------------------------------
log "building pgo-relocs variant..."
( cd "$ROOT" && \
  CARGO_TARGET_DIR="$SCRATCH/target-pgo-relocs" \
  RUSTFLAGS="-Cprofile-use=$MERGED -Cllvm-args=-pgo-warn-missing-function -Clink-arg=-Wl,--emit-relocs" \
  cargo build --release --target "$TRIPLE" -p sparq-cli -p sparq-server )
RELOC_BIN="$SCRATCH/target-pgo-relocs/$TRIPLE/release"
[ -x "$RELOC_BIN/sparq-cli" ] && [ -x "$RELOC_BIN/sparq-server" ] || die "pgo-relocs build incomplete"

# ---- corpora (same pinned generators + cache routing as run.sh) --------------------------
export WATDIV_CACHE="${WATDIV_CACHE:-$SCRATCH/corpora/watdiv}"
export SP2B_CACHE="${SP2B_CACHE:-$SCRATCH/corpora/sp2b}"
export BSBM_CACHE="${BSBM_CACHE:-$SCRATCH/corpora/bsbm}"
WATDIV_NT="$(bash "$ROOT/bench/watdiv/gen.sh" "$WATDIV_SF")"
SP2B_TTL="$(bash "$ROOT/bench/sp2b/gen.sh" "$SP2B_TRIPLES")"
BSBM_NT="$(bash "$ROOT/bench/bsbm/gen.sh" "$BSBM_PC")"
INGEST_NT="$(bash "$ROOT/bench/watdiv/gen.sh" "$PGO_INGEST_SF")"

# ---- 2. perf profiles on the training workloads ------------------------------------------
BOLTDIR="$SCRATCH/bolt"
mkdir -p "$BOLTDIR"

log "perf-profiling sparq-cli workloads..."
# One perf.data for all CLI legs: perf follows children, and perf2bolt keys samples to the
# named binary, so the wrapper shell contributes nothing.
# shellcheck disable=SC2086
perf record -e cycles:u $LBR -o "$BOLTDIR/cli.perf.data" -- bash -c "
  set -e
  '$RELOC_BIN/sparq-cli' bench '$WATDIV_NT' ntriples '$ROOT/bench/watdiv/queries' 2 count       >/dev/null
  '$RELOC_BIN/sparq-cli' bench '$SP2B_TTL'  turtle   '$ROOT/bench/sp2b/queries'   2 count       >/dev/null
  '$RELOC_BIN/sparq-cli' bench '$BSBM_NT'   ntriples '$ROOT/bench/bsbm/queries'   2 materialize >/dev/null
  '$RELOC_BIN/sparq-cli' ingest '$INGEST_NT' full >/dev/null 2>&1
" >/dev/null 2>&1

SERVER_PID=""
PERF_PID=""
stop_perf() {
  # SIGINT is perf record's normal stop signal: it flushes perf.data and exits.
  [ -n "$PERF_PID" ] && { kill -INT "$PERF_PID" 2>/dev/null || true; wait "$PERF_PID" 2>/dev/null || true; PERF_PID=""; }
}
stop_server() {
  [ -n "$SERVER_PID" ] && { kill -TERM "$SERVER_PID" 2>/dev/null || true; wait "$SERVER_PID" 2>/dev/null || true; SERVER_PID=""; }
}
trap 'stop_perf; stop_server' EXIT

if [ "${PGO_SKIP_SERVE:-0}" != 1 ]; then
  log "perf-profiling the sparq-server binary..."
  "$RELOC_BIN/sparq-server" --addr "127.0.0.1:$PGO_PORT" --format ntriples "$WATDIV_NT" >/dev/null 2>&1 &
  SERVER_PID=$!
  for i in $(seq 1 240); do
    curl -sf "http://127.0.0.1:$PGO_PORT/health" >/dev/null 2>&1 && break
    kill -0 "$SERVER_PID" 2>/dev/null || die "server exited before healthy"
    sleep 0.25
  done
  # Bounded attach lifecycle: perf attaches in the background, the driver runs on
  # its own, then perf is interrupted and waited on BEFORE the server stops.
  # (Combining `-p PID` with a workload command would keep perf following the
  # still-alive server after the driver exits — no reliable termination.)
  # shellcheck disable=SC2086
  perf record -e cycles:u $LBR -p "$SERVER_PID" -o "$BOLTDIR/server.perf.data" >/dev/null 2>&1 &
  PERF_PID=$!
  sleep 1   # let perf finish attaching before generating load
  kill -0 "$PERF_PID" 2>/dev/null || { PERF_PID=""; die "perf record failed to attach to the server"; }
  python3 "$HERE/serve_driver.py" --port "$PGO_PORT" --queries-dir "$ROOT/bench/watdiv/queries" --mode train --repeat 5 \
    >/dev/null 2>&1
  stop_perf
  stop_server
  [ -s "$BOLTDIR/server.perf.data" ] || die "server perf.data is empty — the attached recording captured nothing"
fi

# ---- 3. perf2bolt + llvm-bolt ------------------------------------------------------------
NL=""; [ -z "$LBR" ] && NL="-nl"
bolt_one() { # <binary-name> <perf.data>
  local name="$1" pd="$2"
  log "perf2bolt + llvm-bolt: $name..."
  # shellcheck disable=SC2086
  perf2bolt $NL -p "$pd" -o "$BOLTDIR/$name.fdata" "$RELOC_BIN/$name" >/dev/null 2>&1 \
    || die "perf2bolt failed for $name (see $pd)"
  llvm-bolt "$RELOC_BIN/$name" -o "$OUT_BIN/$name" -data "$BOLTDIR/$name.fdata" \
    -reorder-blocks=ext-tsp -reorder-functions=hfsort -split-functions -split-all-cold \
    -icf=1 -use-gnu-stack \
    || die "llvm-bolt failed for $name"
  chmod +x "$OUT_BIN/$name"
}

OUT_BIN="$SCRATCH/target-pgo-bolt/$TRIPLE/release"
mkdir -p "$OUT_BIN"
bolt_one sparq-cli "$BOLTDIR/cli.perf.data"
if [ "${PGO_SKIP_SERVE:-0}" != 1 ]; then
  bolt_one sparq-server "$BOLTDIR/server.perf.data"
else
  # measure-variant only needs sparq-server present when the serve leg runs; copy the
  # un-bolted PGO server so the layout stays uniform.
  cp "$RELOC_BIN/sparq-server" "$OUT_BIN/sparq-server"
fi

# ---- 4. measure + report through the shared code path ------------------------------------
log "measuring pgo-bolt variant..."
bash "$HERE/run.sh" measure-variant pgo-bolt "$OUT_BIN"
bash "$HERE/run.sh" report
log "done — see $RESULTS/REPORT.md (pgo-bolt column)"
