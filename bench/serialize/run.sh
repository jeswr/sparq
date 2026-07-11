#!/usr/bin/env bash
# [FABLE-5] sq-hmd7l.14 — serialization-throughput comparison panel entry point.
#
# Compares sparq's writer matrix (driven through its PUBLIC API via the
# serialize_bench example) against serd (serdi), Redland Raptor (rapper),
# Apache Jena (riot) and an oxrdfio scratch shim, on a deterministic generated
# blank-node-free N-Triples corpus (never committed). No standard serialization
# suite exists — corpus-based throughput with per-output round-trip gates is
# the honest form (see bench/benchmarks.toml `serialize-bench` and
# research/gap-serialize-2026-07.md).
#
#   PHASE A: sparq in-process serialize-only regime (store loaded ONCE;
#            buffered vs streaming vs pretty rows, min-of-iters). Every format's
#            ROUND-TRIP GATE (serialize -> re-parse -> exact store match) runs
#            before its timing; a red gate fails the panel.
#   PHASE B: NEGATIVE self-check — a deliberately corrupted document MUST red
#            the gate (exit 4). Proves the gate is non-vacuous on every run.
#   PHASE C (full mode): cross-engine PIPELINE regime — read NT -> write
#            {turtle, ntriples} as one process, min-of-iters process wall, all
#            engines measured identically (sparq runs its own `pipe` mode under
#            the same stopwatch). Each engine's output is round-trip gated by
#            sparq's `check` BEFORE its row is trusted; a red/absent competitor
#            is an HONEST RECORDED OUTCOME (rapper's Turtle double-literal
#            rewrite, for example), not a harness failure. Only a sparq gate
#            failure reds the panel.
#
# Cross-engine comparability: `mbps_in` (input corpus MB / wall) is the
# corpus-normalized comparable number; `out_bytes` differ per engine (prefix
# policies differ), so per-engine output MB/s is NOT cross-comparable and is
# not printed in PHASE C. Work-box timings are NON-canonical (bench/CATALOG.md).
#
# USAGE
#   bench/serialize/run.sh --smoke   # sparq-only: gate green on nt+turtle(+stream)
#                                    # + the negative self-check; exit 0 = green
#                                    # (the bead acceptance)
#   bench/serialize/run.sh           # full panel; competitor columns below
#
# PEERS (gather-time only — never committed dependencies):
#   SERIALIZE_SERDI    serdi binary (default: `command -v serdi`; absent -> skip)
#   SERIALIZE_RAPPER   rapper binary (default: `command -v rapper`; absent -> skip)
#   SERIALIZE_RIOT     Jena riot binary (default: `command -v riot`; absent -> skip;
#                      JVM startup is inside the process wall — labelled)
#   SERIALIZE_OXRDFIO_BIN  oxrdfio shim binary, or "auto" to build it via
#                      scripts/bench-adapters/serialize_oxrdfio_adapter.sh --build
#
# TUNABLES (env; safe defaults):
#   SERIALIZE_BIN        sparq runner (default target/release/examples/serialize_bench;
#                        auto-built when missing)
#   SERIALIZE_SUBJECTS   corpus subjects (default 40000 full / 1000 smoke; 8 triples each)
#   SERIALIZE_ITERS      PHASE A in-process iterations (default 5 full / 2 smoke)
#   SERIALIZE_PIPE_ITERS PHASE C process-wall iterations per engine+format (default 3)
#   SERIALIZE_JSON_OUT   results envelope path (suggest the git-ignored
#                        bench/competitor-results/)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

SMOKE=0
for arg in "$@"; do
  case "$arg" in
    --smoke) SMOKE=1 ;;
    *) echo "[serialize] unknown arg: $arg (usage: bench/serialize/run.sh [--smoke])" >&2; exit 2 ;;
  esac
done

SERIALIZE_BIN="${SERIALIZE_BIN:-$ROOT/target/release/examples/serialize_bench}"
if [ "$SMOKE" = 1 ]; then
  SERIALIZE_SUBJECTS="${SERIALIZE_SUBJECTS:-1000}"
  SERIALIZE_ITERS="${SERIALIZE_ITERS:-2}"
else
  SERIALIZE_SUBJECTS="${SERIALIZE_SUBJECTS:-40000}"
  SERIALIZE_ITERS="${SERIALIZE_ITERS:-5}"
fi
SERIALIZE_PIPE_ITERS="${SERIALIZE_PIPE_ITERS:-3}"
SERIALIZE_JSON_OUT="${SERIALIZE_JSON_OUT:-}"

log() { printf '[serialize] %s\n' "$*" >&2; }
die() { printf '[serialize] ERROR: %s\n' "$*" >&2; exit 1; }

command -v python3 >/dev/null 2>&1 || die "python3 required"

if [ ! -x "$SERIALIZE_BIN" ]; then
  log "serialize_bench not found at $SERIALIZE_BIN — building"
  cargo build --release -p sparq-engine-serialize \
    --features serialize-rdf,streaming-serialization --example serialize_bench >&2 \
    || die "build failed: cargo build --release -p sparq-engine-serialize --features serialize-rdf,streaming-serialization --example serialize_bench"
fi

TMP="$(mktemp -d "${TMPDIR:-/tmp}/sparq-serialize-bench.XXXXXX")"
trap 'rm -rf "$TMP"' EXIT

# ---- 1. deterministic corpus (generated; never committed) --------------------------------
CORPUS="$TMP/corpus.nt"
"$SERIALIZE_BIN" gen "$SERIALIZE_SUBJECTS" "$CORPUS" >&2
CORPUS_BYTES=$(wc -c < "$CORPUS")
log "corpus: $SERIALIZE_SUBJECTS subjects, $CORPUS_BYTES bytes"

RED=0

# ---- 2. PHASE A: sparq in-process serialize-only regime (gated per format) ---------------
if [ "$SMOKE" = 1 ]; then
  FORMATS="nt,turtle,turtle-stream"
else
  FORMATS="nt,turtle,turtle-pretty,turtle-stream,trig,trig-stream"
fi
log "PHASE A: in-process writer matrix ($FORMATS; min of $SERIALIZE_ITERS)"
if ! "$SERIALIZE_BIN" bench "$CORPUS" --formats "$FORMATS" --iters "$SERIALIZE_ITERS" \
     --json "$TMP/inprocess.json"; then
  RED=1; log "PHASE A FAIL: a sparq round-trip gate is red (see above)"
fi

# ---- 3. PHASE B: the gate must be able to FAIL (non-vacuous self-check) ------------------
"$SERIALIZE_BIN" pipe "$CORPUS" turtle "$TMP/selfcheck.ttl" 2>/dev/null
cp "$TMP/selfcheck.ttl" "$TMP/corrupt.ttl"
printf '<http://sparq.dev/bench/serialize/stray> <http://sparq.dev/bench/serialize/vocab#name> "stray" .\n' >> "$TMP/corrupt.ttl"
rc=0; "$SERIALIZE_BIN" check "$CORPUS" "$TMP/corrupt.ttl" turtle 2>/dev/null || rc=$?
[ "$rc" = 4 ] || die "NEGATIVE SELF-CHECK FAILED: corrupted document returned rc=$rc (want 4) — the round-trip gate is vacuous"
log "PHASE B: negative self-check green (corrupted document reds the gate, rc=4)"

# ---- 4. PHASE C: cross-engine pipeline regime (full mode only) ---------------------------
: > "$TMP/pipeline.tsv"
if [ "$SMOKE" = 0 ]; then
  SERDI="${SERIALIZE_SERDI:-$(command -v serdi || true)}"
  RAPPER="${SERIALIZE_RAPPER:-$(command -v rapper || true)}"
  RIOT="${SERIALIZE_RIOT:-$(command -v riot || true)}"
  OXBIN="${SERIALIZE_OXRDFIO_BIN:-}"
  if [ "$OXBIN" = auto ]; then
    log "building oxrdfio shim (scripts/bench-adapters/serialize_oxrdfio_adapter.sh --build)"
    OXBIN="$("$ROOT/scripts/bench-adapters/serialize_oxrdfio_adapter.sh" --build)" \
      || { log "oxrdfio shim build failed — column skipped"; OXBIN=""; }
  fi

  # emit <engine> <fmt> <outfile>: one pipeline run; nonzero rc = error outcome.
  emit() {
    local eng="$1" fmt="$2" out="$3"
    case "$eng" in
      sparq)   "$SERIALIZE_BIN" pipe "$CORPUS" "$fmt" "$out" 2>/dev/null ;;
      serdi)   "$SERDI" -i ntriples -o "$(cli_fmt "$fmt")" "$CORPUS" > "$out" 2>/dev/null ;;
      rapper)  "$RAPPER" -i ntriples -o "$(cli_fmt "$fmt")" "$CORPUS" > "$out" 2>/dev/null ;;
      riot)    "$RIOT" --syntax=ntriples --output="$(cli_fmt "$fmt")" "$CORPUS" > "$out" 2>/dev/null ;;
      oxrdfio) "$OXBIN" "$CORPUS" "$fmt" "$out" 2>/dev/null ;;
    esac
  }
  cli_fmt() { case "$1" in nt) echo ntriples ;; *) echo "$1" ;; esac; }
  engine_bin() {
    case "$1" in
      sparq) echo "$SERIALIZE_BIN" ;; serdi) echo "$SERDI" ;; rapper) echo "$RAPPER" ;;
      riot) echo "$RIOT" ;; oxrdfio) echo "$OXBIN" ;;
    esac
  }

  for eng in sparq serdi rapper riot oxrdfio; do
    if [ -z "$(engine_bin "$eng")" ]; then
      for fmt in turtle nt; do
        printf '%s\t%s\tabsent\t-\t-\t-\n' "$eng" "$fmt" >> "$TMP/pipeline.tsv"
      done
      log "PHASE C: $eng absent — column skipped (set SERIALIZE_$(echo "$eng" | tr '[:lower:]' '[:upper:]')* to enable)"
      continue
    fi
    for fmt in turtle nt; do
      out="$TMP/$eng.$fmt.out"
      if ! emit "$eng" "$fmt" "$out"; then
        printf '%s\t%s\terror\t-\t-\t-\n' "$eng" "$fmt" >> "$TMP/pipeline.tsv"
        log "PHASE C: $eng/$fmt -> error (emit failed; recorded, column untimed)"
        continue
      fi
      # ROUND-TRIP GATE before the stopwatch is trusted.
      rc=0; "$SERIALIZE_BIN" check "$CORPUS" "$out" "$fmt" 2> "$TMP/gate.err" || rc=$?
      if [ "$rc" != 0 ]; then
        printf '%s\t%s\tred(rc=%s)\t-\t-\t-\n' "$eng" "$fmt" "$rc" >> "$TMP/pipeline.tsv"
        log "PHASE C: $eng/$fmt -> ROUND-TRIP RED (rc=$rc): $(tail -1 "$TMP/gate.err")"
        # sparq soundness inside the panel: its own writer failing the gate
        # contradicts PHASE A -> red panel. Peer divergence is a recorded outcome.
        if [ "$eng" = sparq ]; then RED=1; log "SOUNDNESS FAIL: sparq pipeline output red"; fi
        continue
      fi
      out_bytes=$(wc -c < "$out")
      best=""
      for _ in $(seq 1 "$SERIALIZE_PIPE_ITERS"); do
        start=$(date +%s%N)
        emit "$eng" "$fmt" "$out" || die "timing rerun failed: $eng $fmt"
        wall_us=$(( ($(date +%s%N) - start) / 1000 ))
        if [ -z "$best" ] || [ "$wall_us" -lt "$best" ]; then best=$wall_us; fi
      done
      mbps_in=$(python3 -c "print(f'{$CORPUS_BYTES/1e6/($best/1e6):.1f}')")
      printf '%s\t%s\tgreen\t%s\t%s\t%s\n' "$eng" "$fmt" "$out_bytes" "$best" "$mbps_in" >> "$TMP/pipeline.tsv"
      log "PHASE C: $eng/$fmt -> green, min wall ${best}us (${mbps_in} MB/s over input)"
    done
  done
  log "PHASE C table (engine fmt outcome out_bytes min_wall_us mbps_in):"
  column -t "$TMP/pipeline.tsv" >&2 || cat "$TMP/pipeline.tsv" >&2
fi

# ---- 5. optional JSON envelope (land it in git-ignored competitor-results) ---------------
if [ -n "$SERIALIZE_JSON_OUT" ]; then
  : > "$TMP/versions.tsv"
  printf 'sparq\t%s\n' "$(git -C "$ROOT" rev-parse --short HEAD 2>/dev/null || echo unknown)" >> "$TMP/versions.tsv"
  if [ "$SMOKE" = 0 ]; then
    [ -n "${SERDI:-}" ]  && printf 'serdi\t%s\n'  "$("$SERDI" --version 2>&1 | head -1)" >> "$TMP/versions.tsv"
    [ -n "${RAPPER:-}" ] && printf 'rapper\t%s\n' "$("$RAPPER" --version 2>&1 | head -1)" >> "$TMP/versions.tsv"
    [ -n "${RIOT:-}" ]   && printf 'riot\t%s\n'   "$("$RIOT" --version 2>&1 | head -1)" >> "$TMP/versions.tsv"
    [ -n "${OXBIN:-}" ]  && printf 'oxrdfio\t%s\n' "$("$OXBIN" --version 2>&1 | head -1)" >> "$TMP/versions.tsv"
  fi
  SER_TMP="$TMP" SER_CORPUS_BYTES="$CORPUS_BYTES" SER_SUBJECTS="$SERIALIZE_SUBJECTS" \
  SER_PIPE_ITERS="$SERIALIZE_PIPE_ITERS" SER_RED="$RED" \
    python3 - "$SERIALIZE_JSON_OUT" <<'PYEOF'
import json, os, platform, sys, time
tmp = os.environ["SER_TMP"]
def rows(name, cols):
    out = []
    path = os.path.join(tmp, name)
    if not os.path.exists(path):
        return out
    with open(path, encoding="utf-8") as f:
        for line in f:
            out.append(dict(zip(cols, line.rstrip("\n").split("\t"))))
    return out
inprocess = {}
ip_path = os.path.join(tmp, "inprocess.json")
if os.path.exists(ip_path):
    with open(ip_path, encoding="utf-8") as f:
        inprocess = json.load(f)
envelope = {
    "suite": "serialize-bench",
    "generated_at_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    "corpus": {
        "generator": "serialize_bench gen (deterministic LCG; blank-node-free)",
        "subjects": int(os.environ["SER_SUBJECTS"]),
        "bytes": int(os.environ["SER_CORPUS_BYTES"]),
    },
    "red": os.environ["SER_RED"] != "0",
    "host": {
        "platform": platform.platform(),
        "machine": platform.machine(),
        "nproc": os.cpu_count(),
        "quiet_box": False,  # flip only on a dedicated quiet gather box
        "note": "work-box timings are NON-canonical; see bench/CATALOG.md QUIET-BOX",
    },
    "engines": rows("versions.tsv", ["engine", "version"]),
    "in_process": inprocess,
    "pipeline": rows("pipeline.tsv",
                     ["engine", "format", "round_trip", "out_bytes", "min_wall_us", "mbps_in"]),
    "pipeline_note": "one process per run (min of "
                     + os.environ["SER_PIPE_ITERS"]
                     + "): read NT -> write format; wall includes process spawn "
                       "(JVM startup dominates a riot column); mbps_in = input corpus "
                       "MB / wall (cross-comparable); out_bytes differ per engine "
                       "(prefix policies) so output MB/s is NOT cross-comparable. "
                       "sparq + oxrdfio materialize-then-serialize; serdi/rapper/riot "
                       "run as shipped. red(rc=4) = the engine emitted a document that "
                       "does not round-trip to the input store (recorded outcome).",
}
with open(sys.argv[1], "w", encoding="utf-8") as f:
    json.dump(envelope, f, indent=2)
    f.write("\n")
PYEOF
  log "envelope written to $SERIALIZE_JSON_OUT"
fi

if [ "$RED" = 0 ]; then
  log "OK: round-trip gates green (sparq matrix)$([ "$SMOKE" = 1 ] && echo ' — smoke' || true)"
else
  log "FAILED: see PHASE A/SOUNDNESS FAIL lines above"
fi
exit "$RED"
