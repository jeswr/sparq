#!/usr/bin/env bash
# [FABLE-5] sq-98w7z.4 — rustc PGO (profile-guided optimisation) EVALUATION harness for the
# sparq-cli + sparq-server binaries. Instrument -> train -> use -> A/B, producing a per-query
# delta table (report.py). EVALUATION ONLY: nothing here touches the shipped release profile,
# dist.yml, or any artifact — adoption is a separate decision bead gated on a canonical
# quiet-box re-measure of these numbers.
#
# Background: the release profile is already saturated on the classic levers (fat LTO,
# codegen-units=1, panic=abort; -Ctarget-cpu tiers measured zero uplift — see
# research/hw-bench-results.md + research/dependency-bottleneck-analysis-2026-07.md). PGO is
# the remaining unexplored profile-feedback lever; BOLT (bolt.sh) stacks on top ONLY if PGO
# alone shows >= 3% (the bead's gate).
#
# Method (the standard rustc PGO recipe, https://doc.rust-lang.org/rustc/profile-guided-optimization.html):
#   1. baseline   : plain `cargo build --release` (the shipped profile, untouched)
#   2. instrument : same profile + RUSTFLAGS=-Cprofile-generate=<dir>
#   3. train      : run the instrumented binaries on the REAL workloads —
#                     query : watdiv / sp2b / bsbm per-commit query mixes (`sparq-cli bench`)
#                     ingest: decompress+parse+index build (`sparq-cli ingest ... full`)
#                     serve : the sparq-server BINARY on loopback HTTP (serve_driver.py)
#   4. merge      : llvm-profdata merge (the rustup llvm-tools component, LLVM-matched)
#   5. use        : same profile + RUSTFLAGS=-Cprofile-use=<merged.profdata>
#   6. measure    : identical workloads on baseline vs PGO binaries; engine-internal min-of-N
#                   for queries (contention-robust), wall-clock for ingest, loopback req/s+p50
#                   for serve
#   7. report     : report.py — delta table + HARD correctness differential (identical row
#                   counts / response bytes between variants; a PGO build must never change
#                   results) + the >= 3% BOLT verdict
#
# Usage:
#   bench/pgo/run.sh                # all phases in order (each phase is cached/idempotent)
#   bench/pgo/run.sh <phase>...     # corpora | build-baseline | build-instr | train | merge |
#                                   # build-pgo | measure | report | clean
#   bench/pgo/run.sh measure-variant <label> <bindir>   # (internal; reused by bolt.sh so
#                                                       # every variant is measured by the
#                                                       # exact same code path)
# Env knobs (defaults in brackets):
#   PGO_SCRATCH      scratch root, target dirs + profiles + results   [/tmp/sparq-pgo]
#   ITERS            query-bench iterations per variant               [7]
#   INGEST_ITERS     ingest wall-clock repetitions (min taken)        [3]
#   PGO_INGEST_SF    WatDiv scale factor for the ingest corpus        [30]
#   WATDIV_SF / SP2B_TRIPLES / BSBM_PC   query-mix corpora pins       [1 / 250000 / 300]
#   PGO_PORT         loopback port for the serve workload             [3671]
#   SERVE_REPEAT / SERVE_BATCHES   serve driver sizing                [25 / 3]
#   PGO_SKIP_SERVE=1 skip the sparq-server train+measure legs (cli-only iteration)
#   PGO_MIN_FREE_GB  abort threshold for free space on the scratch fs [15]
#
# HONESTY / DISK RULES (bench/CATALOG.md conventions):
#   - Wall-clock numbers from a shared work box are NON-canonical; report.py stamps the
#     environment + load into the report. Adoption claims gate on the canonical quiet-box
#     re-measure bead. Query timings use the engine-internal min-of-N (ratio-robust under
#     contention); ingest/serve are wall-clock sensitive — prefer a quiet box.
#   - Everything this script produces is git-ignored and regenerable. Three release target
#     dirs + corpora need real disk: run `bench/pgo/run.sh clean` when done.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"

SCRATCH="${PGO_SCRATCH:-/tmp/sparq-pgo}"
ITERS="${ITERS:-7}"
INGEST_ITERS="${INGEST_ITERS:-3}"
PGO_INGEST_SF="${PGO_INGEST_SF:-30}"
WATDIV_SF="${WATDIV_SF:-1}"
SP2B_TRIPLES="${SP2B_TRIPLES:-250000}"
BSBM_PC="${BSBM_PC:-300}"
PGO_PORT="${PGO_PORT:-3671}"
SERVE_REPEAT="${SERVE_REPEAT:-25}"
SERVE_BATCHES="${SERVE_BATCHES:-3}"
PGO_MIN_FREE_GB="${PGO_MIN_FREE_GB:-15}"

# Route the suite corpus caches under the scratch root (each gen.sh honors its cache env
# var; a caller-set value wins), so ONE root holds everything and `clean` removes it all —
# the default /tmp is a small tmpfs on some boxes.
export WATDIV_CACHE="${WATDIV_CACHE:-$SCRATCH/corpora/watdiv}"
export SP2B_CACHE="${SP2B_CACHE:-$SCRATCH/corpora/sp2b}"
export BSBM_CACHE="${BSBM_CACHE:-$SCRATCH/corpora/bsbm}"

PROFDIR="$SCRATCH/profraw"
MERGED="$SCRATCH/merged.profdata"
RESULTS="$SCRATCH/results"
TRIPLE="$(rustc -vV | sed -n 's/^host: //p')"
PACKAGES=(-p sparq-cli -p sparq-server)

log() { echo "[pgo] $*" >&2; }
die() { echo "[pgo] FATAL: $*" >&2; exit 1; }

bindir() { echo "$SCRATCH/target-$1/$TRIPLE/release"; }

check_disk() {
  mkdir -p "$SCRATCH"
  local avail_kb
  avail_kb="$(df -Pk "$SCRATCH" | awk 'NR==2{print $4}')"
  local avail_gb=$((avail_kb / 1024 / 1024))
  log "free space on scratch fs: ${avail_gb} GB (threshold ${PGO_MIN_FREE_GB} GB)"
  [ "$avail_gb" -ge "$PGO_MIN_FREE_GB" ] \
    || die "only ${avail_gb} GB free on $(df -Pk "$SCRATCH" | awk 'NR==2{print $6}') — need >= ${PGO_MIN_FREE_GB} GB (3 release target dirs + corpora). Free space or set PGO_SCRATCH/PGO_MIN_FREE_GB."
}

# llvm-profdata must come from the RUSTUP llvm-tools component so its LLVM version matches
# rustc's; a distro llvm-profdata is accepted only as a fallback (merge fails loudly on a
# version mismatch, so a silent wrong-answer is not a risk).
find_profdata() {
  local p="$(rustc --print sysroot)/lib/rustlib/$TRIPLE/bin/llvm-profdata"
  if [ ! -x "$p" ]; then
    log "llvm-profdata not in sysroot — installing the rustup llvm-tools component..."
    rustup component add llvm-tools >/dev/null 2>&1 \
      || rustup component add llvm-tools-preview >/dev/null 2>&1 \
      || true
  fi
  if [ -x "$p" ]; then echo "$p"; return; fi
  command -v llvm-profdata >/dev/null 2>&1 \
    || die "no llvm-profdata (rustup component add llvm-tools failed and none on PATH)"
  log "WARNING: using system llvm-profdata ($(command -v llvm-profdata)) — must match rustc's LLVM major"
  command -v llvm-profdata
}

# ---- corpora (delegated to the pinned, deterministic per-suite generators) ---------------
phase_corpora() {
  log "ensuring corpora (watdiv SF=$WATDIV_SF, sp2b $SP2B_TRIPLES, bsbm pc=$BSBM_PC, ingest watdiv SF=$PGO_INGEST_SF)..."
  WATDIV_NT="$(bash "$ROOT/bench/watdiv/gen.sh" "$WATDIV_SF")"
  SP2B_TTL="$(bash "$ROOT/bench/sp2b/gen.sh" "$SP2B_TRIPLES")"
  BSBM_NT="$(bash "$ROOT/bench/bsbm/gen.sh" "$BSBM_PC")"
  INGEST_NT="$(bash "$ROOT/bench/watdiv/gen.sh" "$PGO_INGEST_SF")"
  for f in "$WATDIV_NT" "$SP2B_TTL" "$BSBM_NT" "$INGEST_NT"; do
    [ -s "$f" ] || die "corpus missing/empty: $f"
  done
  log "corpora ready"
}

# Re-resolve corpus paths without regenerating (gen.sh caches; calling it again is a no-op
# echo of the cached path, but keep one resolver so every phase agrees on paths).
resolve_corpora() { phase_corpora; }

# ---- builds ------------------------------------------------------------------------------
# Explicit --target keeps RUSTFLAGS off build scripts / proc-macros (the documented rustc
# PGO recipe), so only the shipped binaries are instrumented/optimised. Each variant gets
# its own CARGO_TARGET_DIR so re-runs are incremental per variant.
build_variant() { # <name> [extra RUSTFLAGS...]
  local name="$1"; shift
  log "building variant '$name' (RUSTFLAGS: ${*:-<none>})..."
  ( cd "$ROOT" && \
    CARGO_TARGET_DIR="$SCRATCH/target-$name" RUSTFLAGS="$*" \
    cargo build --release --target "$TRIPLE" "${PACKAGES[@]}" )
  [ -x "$(bindir "$name")/sparq-cli" ] || die "build '$name' produced no sparq-cli"
  [ -x "$(bindir "$name")/sparq-server" ] || die "build '$name' produced no sparq-server"
}

phase_build_baseline() { build_variant baseline; }
phase_build_instr()    { mkdir -p "$PROFDIR"; build_variant instr "-Cprofile-generate=$PROFDIR"; }
phase_build_pgo() {
  [ -s "$MERGED" ] || die "no $MERGED — run the train + merge phases first"
  build_variant pgo "-Cprofile-use=$MERGED -Cllvm-args=-pgo-warn-missing-function"
}

# ---- server helpers ----------------------------------------------------------------------
SERVER_PID=""
start_server() { # <bindir> <corpus.nt>
  "$1/sparq-server" --addr "127.0.0.1:$PGO_PORT" --format ntriples "$2" >/dev/null 2>&1 &
  SERVER_PID=$!
  local i
  for i in $(seq 1 240); do
    if curl -sf "http://127.0.0.1:$PGO_PORT/health" >/dev/null 2>&1; then return 0; fi
    kill -0 "$SERVER_PID" 2>/dev/null || die "sparq-server exited before becoming healthy"
    sleep 0.25
  done
  die "sparq-server did not become healthy on port $PGO_PORT within 60s"
}
# Graceful SIGTERM so the instrumented server flushes its .profraw at exit (SIGKILL would
# lose the profile). Escalates to SIGKILL only after a 15s grace period.
stop_server() {
  [ -n "$SERVER_PID" ] || return 0
  kill -TERM "$SERVER_PID" 2>/dev/null || true
  local i
  for i in $(seq 1 60); do
    kill -0 "$SERVER_PID" 2>/dev/null || { SERVER_PID=""; return 0; }
    sleep 0.25
  done
  log "WARNING: server $SERVER_PID ignored SIGTERM for 15s — SIGKILL (profile data lost if instrumented)"
  kill -9 "$SERVER_PID" 2>/dev/null || true
  wait "$SERVER_PID" 2>/dev/null || true
  SERVER_PID=""
}
trap stop_server EXIT

# ---- train (instrumented binaries; timings meaningless, coverage is the point) -----------
phase_train() {
  resolve_corpora
  local B; B="$(bindir instr)"
  [ -x "$B/sparq-cli" ] || die "no instrumented build — run build-instr first"
  log "training on query mixes (watdiv/sp2b/bsbm)..."
  "$B/sparq-cli" bench "$WATDIV_NT" ntriples "$ROOT/bench/watdiv/queries" 2 count       >/dev/null
  "$B/sparq-cli" bench "$WATDIV_NT" ntriples "$ROOT/bench/watdiv/queries" 1 materialize >/dev/null
  "$B/sparq-cli" bench "$SP2B_TTL"  turtle   "$ROOT/bench/sp2b/queries"   2 count       >/dev/null
  "$B/sparq-cli" bench "$BSBM_NT"   ntriples "$ROOT/bench/bsbm/queries"   2 materialize >/dev/null
  log "training on ingest (decompress+parse+index build, watdiv SF=$PGO_INGEST_SF)..."
  "$B/sparq-cli" ingest "$INGEST_NT" full >/dev/null 2>&1
  if [ "${PGO_SKIP_SERVE:-0}" != 1 ]; then
    log "training sparq-server on loopback HTTP..."
    start_server "$B" "$WATDIV_NT"
    python3 "$HERE/serve_driver.py" --port "$PGO_PORT" --queries-dir "$ROOT/bench/watdiv/queries" --mode train --repeat 3
    python3 "$HERE/serve_driver.py" --port "$PGO_PORT" --queries-dir "$ROOT/bench/bsbm/queries"   --mode train --repeat 2
    stop_server
  else
    log "PGO_SKIP_SERVE=1 — skipping the server training leg"
  fi
  log "train done; profraw files: $(find "$PROFDIR" -name '*.profraw' | wc -l)"
}

phase_merge() {
  local PD; PD="$(find_profdata)"
  local n; n="$(find "$PROFDIR" -name '*.profraw' 2>/dev/null | wc -l)"
  [ "$n" -gt 0 ] || die "no .profraw under $PROFDIR — run the train phase first"
  log "merging $n profraw files with $PD..."
  "$PD" merge -o "$MERGED" "$PROFDIR"
  [ -s "$MERGED" ] || die "llvm-profdata merge produced no output"
  log "merged profile: $MERGED ($(du -h "$MERGED" | cut -f1))"
}

# ---- measure (identical code path for every variant — fairness by construction) ----------
measure_variant() { # <label> <bindir>
  local label="$1" B="$2" out="$RESULTS/$1"
  resolve_corpora
  [ -x "$B/sparq-cli" ] || die "no sparq-cli under $B"
  mkdir -p "$out"
  log "[$label] query suites (engine-internal min of $ITERS)..."
  "$B/sparq-cli" bench "$WATDIV_NT" ntriples "$ROOT/bench/watdiv/queries" "$ITERS" count       2>/dev/null > "$out/watdiv.tsv"
  "$B/sparq-cli" bench "$SP2B_TTL"  turtle   "$ROOT/bench/sp2b/queries"   "$ITERS" count       2>/dev/null > "$out/sp2b.tsv"
  "$B/sparq-cli" bench "$BSBM_NT"   ntriples "$ROOT/bench/bsbm/queries"   "$ITERS" materialize 2>/dev/null > "$out/bsbm.tsv"
  log "[$label] ingest wall-clock (min of $INGEST_ITERS, watdiv SF=$PGO_INGEST_SF)..."
  local best_ns="" i t0 t1 dt
  for i in $(seq 1 "$INGEST_ITERS"); do
    t0="$(date +%s%N)"
    "$B/sparq-cli" ingest "$INGEST_NT" full >/dev/null 2>&1
    t1="$(date +%s%N)"
    dt=$((t1 - t0))
    if [ -z "$best_ns" ] || [ "$dt" -lt "$best_ns" ]; then best_ns="$dt"; fi
  done
  awk -v ns="$best_ns" 'BEGIN{printf "%.3f\n", ns/1e9}' > "$out/ingest.txt"
  if [ "${PGO_SKIP_SERVE:-0}" != 1 ]; then
    log "[$label] serve loopback (best of $SERVE_BATCHES batches x $SERVE_REPEAT passes)..."
    start_server "$B" "$WATDIV_NT"
    python3 "$HERE/serve_driver.py" --port "$PGO_PORT" --queries-dir "$ROOT/bench/watdiv/queries" \
      --mode measure --repeat "$SERVE_REPEAT" --batches "$SERVE_BATCHES" --out "$out/serve.json"
    stop_server
  fi
  log "[$label] done -> $out"
}

capture_env() {
  mkdir -p "$RESULTS"
  {
    echo "date_utc: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "host: $(hostname) ($(uname -srm))"
    echo "rustc: $(rustc -V)"
    echo "cpu: $(sed -n 's/^model name[[:space:]]*: //p' /proc/cpuinfo | head -1) x$(nproc)"
    echo "loadavg_at_measure: $(cut -d' ' -f1-3 /proc/loadavg)"
    echo "pins: watdiv_sf=$WATDIV_SF sp2b=$SP2B_TRIPLES bsbm_pc=$BSBM_PC ingest_sf=$PGO_INGEST_SF iters=$ITERS"
  } > "$RESULTS/env.txt"
}

phase_measure() {
  capture_env
  measure_variant baseline "$(bindir baseline)"
  measure_variant pgo      "$(bindir pgo)"
}

phase_report() {
  local labels=(baseline pgo)
  [ -d "$RESULTS/pgo-bolt" ] && labels+=(pgo-bolt)
  python3 "$HERE/report.py" "$RESULTS" "${labels[@]}"
}

phase_clean() {
  log "removing $SCRATCH (target dirs, profiles, corpora caches, results)..."
  rm -rf "$SCRATCH"
  log "note: if you overrode WATDIV_CACHE/SP2B_CACHE/BSBM_CACHE to a shared location, those caches are yours to keep or remove"
}

# ---- dispatch ----------------------------------------------------------------------------
main() {
  cd "$ROOT"
  if [ "${1:-}" = "measure-variant" ]; then
    [ $# -eq 3 ] || die "usage: run.sh measure-variant <label> <bindir>"
    check_disk; measure_variant "$2" "$3"; return
  fi
  if [ "${1:-}" = "clean" ]; then phase_clean; return; fi
  check_disk
  local phases=("$@")
  [ ${#phases[@]} -gt 0 ] || phases=(corpora build-baseline build-instr train merge build-pgo measure report)
  local ph
  for ph in "${phases[@]}"; do
    case "$ph" in
      corpora)        phase_corpora ;;
      build-baseline) phase_build_baseline ;;
      build-instr)    phase_build_instr ;;
      train)          phase_train ;;
      merge)          phase_merge ;;
      build-pgo)      phase_build_pgo ;;
      measure)        phase_measure ;;
      report)         phase_report ;;
      *) die "unknown phase '$ph' (corpora|build-baseline|build-instr|train|merge|build-pgo|measure|report|clean)" ;;
    esac
  done
}

main "$@"
