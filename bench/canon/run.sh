#!/usr/bin/env bash
# [FABLE-5] sq-hmd7l.16 — RDFC-1.0 comparative canonicalization panel entry point.
#
# Compares sparq-canon (driven through its PUBLIC API via the canon_bench
# example) against rdf-canonize (JS, digitalbazaar — the independent reference
# implementation) and the rdf-canon Rust crate (which sparq-canon itself
# delegates to — that column measures sparq's bridge overhead, see README) on:
#
#   PHASE A (gate, before ANY timing): byte-identical canonical N-Quads on the
#           NON-pathological W3C rdf-canon suite fixtures (the vendored
#           crates/sparq-canon/tests/rdf-canon-testdata snapshot is the oracle).
#           A mismatch is red: that engine's timing is skipped, the panel fails.
#   PHASE B (poison/DoS): the suite's pathological high-blank-node-symmetry
#           graphs (poison evals test044/045/046 + negative test074, plus
#           generated blank-node cliques in full mode) under a HARD per-graph
#           wall-clock cap. An engine blowing the cap ("capped"), refusing
#           fail-closed ("guard"), answering wrong bytes ("wrong"), or accepting
#           a must-fail case ("accepted") is an HONEST RECORDED OUTCOME — the
#           DoS-resistance result, not a harness failure.
#   PHASE C (full mode only): advisory min-of-iters timing on the sane set for
#           engines that passed PHASE A. Work-box timings are NON-canonical.
#
# USAGE
#   bench/canon/run.sh --smoke   # sparq only: full sane-set byte-equality gate +
#                                # the vendored negative poison fixture under the
#                                # cap; exit 0 = green (the bead acceptance)
#   bench/canon/run.sh           # full panel; peer columns via env below
#
# PEERS (gather-time only — never committed dependencies):
#   CANON_NODE_MODULES   dir whose node_modules contains rdf-canonize
#                        (npm install rdf-canonize) -> enables the JS column
#   CANON_RDF_CANON_BIN  scratch rdf-canon CLI binary, or "auto" to build it via
#                        scripts/bench-adapters/canon_adapter.sh --build
#
# TUNABLES (env; safe defaults):
#   CANON_BENCH_BIN     sparq runner (default target/release/examples/canon_bench;
#                       auto-built when missing)
#   CANON_CAP_S         HARD per-graph wall-clock cap, seconds (default 10)
#   CANON_ITERS         PHASE C timing iterations (default 3)
#   CANON_CLIQUE_SIZES  extra generated poison clique sizes, full mode (default "12 16 20")
#   CANON_JSON_OUT      results envelope path (suggest the git-ignored
#                       bench/competitor-results/)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

SMOKE=0
for arg in "$@"; do
  case "$arg" in
    --smoke) SMOKE=1 ;;
    *) echo "[canon] unknown arg: $arg (usage: bench/canon/run.sh [--smoke])" >&2; exit 2 ;;
  esac
done

TESTDATA="$ROOT/crates/sparq-canon/tests/rdf-canon-testdata"
CANON_BENCH_BIN="${CANON_BENCH_BIN:-$ROOT/target/release/examples/canon_bench}"
CANON_CAP_S="${CANON_CAP_S:-10}"
CANON_ITERS="${CANON_ITERS:-3}"
CANON_CLIQUE_SIZES="${CANON_CLIQUE_SIZES:-12 16 20}"
CANON_JSON_OUT="${CANON_JSON_OUT:-}"

log() { printf '[canon] %s\n' "$*" >&2; }
die() { printf '[canon] ERROR: %s\n' "$*" >&2; exit 1; }

command -v python3 >/dev/null 2>&1 || die "python3 required"
command -v timeout >/dev/null 2>&1 || die "coreutils timeout required"

if [ ! -x "$CANON_BENCH_BIN" ]; then
  log "canon_bench not found at $CANON_BENCH_BIN — building"
  cargo build --release -p sparq-canon --example canon_bench >&2 \
    || die "build failed: cargo build --release -p sparq-canon --example canon_bench"
fi

TMP="$(mktemp -d "${TMPDIR:-/tmp}/sparq-canon-bench.XXXXXX")"
trap 'rm -rf "$TMP"' EXIT

# ---- 1. manifest-driven fixture sets (runtime-derived; no hard-coded test ids) ----------
python3 - "$TESTDATA/manifest.ttl" > "$TMP/entries.tsv" <<'PYEOF'
import re, sys
src = open(sys.argv[1], encoding="utf-8").read()
rows = []
for b in re.split(r"\n\s*\n", src):
    m = re.search(r"^:(\S+)\s+a\s+rdfc:(RDFC10\w+)", b, re.M)
    if not m or "mf:action" not in b:
        continue
    action = re.search(r"mf:action <([^>]+)>", b).group(1)
    result = re.search(r"mf:result <([^>]+)>", b)
    name = re.search(r'mf:name "([^"]*)"', b)
    hashalg = re.search(r'rdfc:hashAlgorithm "([^"]*)"', b)
    poison = "1" if (name and "poison" in name.group(1)) else "0"
    rows.append((m.group(1), m.group(2), action,
                 result.group(1) if result else "-",
                 hashalg.group(1) if hashalg else "-", poison))
assert len(rows) == 86, f"expected the 86-entry vendored manifest, got {len(rows)}"
sys.stdout.write("".join("\t".join(r) + "\n" for r in rows))
PYEOF

# sane evals: byte-equality oracle set; poison evals + negatives: the DoS panel.
awk -F'\t' '$2=="RDFC10EvalTest" && $6=="0" && $4!="-"' "$TMP/entries.tsv" > "$TMP/sane.tsv"
awk -F'\t' '$2=="RDFC10EvalTest" && $6=="1"' "$TMP/entries.tsv" > "$TMP/poison_eval.tsv"
awk -F'\t' '$2=="RDFC10NegativeEvalTest"' "$TMP/entries.tsv" > "$TMP/negative.tsv"
SANE_N=$(wc -l < "$TMP/sane.tsv"); PE_N=$(wc -l < "$TMP/poison_eval.tsv"); NEG_N=$(wc -l < "$TMP/negative.tsv")
[ "$SANE_N" -ge 60 ] || die "sane eval set too small ($SANE_N) — manifest parse drift"
[ "$PE_N" -ge 1 ] || die "no poison eval fixtures found — manifest parse drift"
[ "$NEG_N" -ge 1 ] || die "no negative (must-fail) fixtures found — manifest parse drift"
log "fixtures: $SANE_N sane evals, $PE_N poison evals, $NEG_N negatives (cap ${CANON_CAP_S}s/graph)"

# ---- 2. engines ---------------------------------------------------------------------------
ENGINES=(sparq)
if [ "$SMOKE" = 0 ]; then
  if [ -n "${CANON_NODE_MODULES:-}" ]; then
    command -v node >/dev/null 2>&1 || die "CANON_NODE_MODULES set but node not on PATH"
    ENGINES+=(rdf-canonize)
  fi
  if [ "${CANON_RDF_CANON_BIN:-}" = "auto" ]; then
    CANON_RDF_CANON_BIN="$(bash "$ROOT/scripts/bench-adapters/canon_adapter.sh" --build)" \
      || die "canon_adapter.sh --build failed"
  fi
  if [ -n "${CANON_RDF_CANON_BIN:-}" ]; then
    [ -x "$CANON_RDF_CANON_BIN" ] || die "CANON_RDF_CANON_BIN=$CANON_RDF_CANON_BIN not executable"
    ENGINES+=(rdf-canon)
  fi
fi
log "engines: ${ENGINES[*]}"

# rdf-canonize ships an aggressive default DoS guard (maxWorkFactor=1) that
# fail-closes even on small legitimate symmetric graphs (measured: 18 of the 64
# approved eval cases). Its documented raised posture (maxWorkFactor=3) is
# byte-exact on the full eval set — so parity + timing run the JS peer at the
# raised posture, while the poison/DoS panel records BOTH postures (the default
# IS the shipped DoS-resistance). sparq and rdf-canon need no such split: their
# default (HNDQ call limit) passes every approved eval as shipped.
CANON_JS_PARITY_WF="${CANON_JS_MAX_WORK_FACTOR:-3}"

# Runs one engine on one input under the HARD cap; returns the exit code.
run_engine() { # engine input sha384(0|1) stdout_file stderr_file [js_work_factor]
  local -a cmd
  case "$1" in
    sparq)        cmd=("$CANON_BENCH_BIN" canon) ;;
    rdf-canonize) cmd=(node "$ROOT/scripts/bench-adapters/canon_adapter.mjs" --engine rdf-canonize --input) ;;
    rdf-canon)    cmd=("$CANON_RDF_CANON_BIN" canon) ;;
    *) die "unknown engine $1" ;;
  esac
  if [ "$1" = rdf-canonize ]; then
    cmd+=("$2"); [ "$3" = 1 ] && cmd+=(--sha384)
    if [ -n "${6:-}" ]; then cmd=(env "CANON_JS_MAX_WORK_FACTOR=$6" "${cmd[@]}"); fi
  else
    [ "$3" = 1 ] && cmd+=(--sha384); cmd+=("$2")
  fi
  timeout -k 2 "$CANON_CAP_S" "${cmd[@]}" > "$4" 2> "$5"
}

# The parity/timing invocation posture for an engine (only the JS peer differs).
parity_wf() { # engine -> js work factor or ""
  if [ "$1" = rdf-canonize ]; then echo "$CANON_JS_PARITY_WF"; else echo ""; fi
}

stderr_metric() { # stderr_file key -> value or "-"
  awk -F= -v k="$2" '$1==k{v=$2} END{print (v=="" ? "-" : v)}' "$1"
}

# ---- 3. PHASE A: byte-equality gate on the sane set (BEFORE any timing) -------------------
RED=0
: > "$TMP/parity.tsv"
for eng in "${ENGINES[@]}"; do
  pass=0; fail=0; wf="$(parity_wf "$eng")"
  while IFS=$'\t' read -r tid kind action result hash poison; do
    in="$TESTDATA/rdfc10/$(basename "$action")"
    exp="$TESTDATA/rdfc10/$(basename "$result")"
    s384=0; [ "$hash" = "SHA384" ] && s384=1
    rc=0; run_engine "$eng" "$in" "$s384" "$TMP/a.out" "$TMP/a.err" "$wf" || rc=$?
    if [ "$rc" = 0 ] && cmp -s "$exp" "$TMP/a.out"; then
      pass=$((pass+1))
    else
      fail=$((fail+1)); RED=1
      log "PARITY FAIL: $eng on $tid (rc=$rc)"
    fi
  done < "$TMP/sane.tsv"
  printf '%s\t%s\t%s\n' "$eng" "$pass" "$fail" >> "$TMP/parity.tsv"
  if [ "$fail" = 0 ]; then
    log "PHASE A: $eng canonical bytes match the W3C oracle on all $pass sane fixtures"
  else
    log "PHASE A: $eng FAILED byte-equality on $fail/$((pass+fail)) fixtures — timing skipped"
  fi
done

# ---- 4. PHASE B: poison / DoS-resistance panel under the hard cap -------------------------
# Fixture list: "<label>\t<input>\t<expected-or-->". Smoke: the first vendored
# negative fixture only (the bead acceptance); full: all poisons + gen-cliques.
: > "$TMP/poison_fixtures.tsv"
if [ "$SMOKE" = 1 ]; then
  IFS=$'\t' read -r tid kind action result hash poison < "$TMP/negative.tsv"
  printf '%s\t%s\t-\n' "$tid" "$TESTDATA/rdfc10/$(basename "$action")" >> "$TMP/poison_fixtures.tsv"
else
  while IFS=$'\t' read -r tid kind action result hash poison; do
    printf '%s\t%s\t%s\n' "$tid" "$TESTDATA/rdfc10/$(basename "$action")" \
      "$TESTDATA/rdfc10/$(basename "$result")" >> "$TMP/poison_fixtures.tsv"
  done < "$TMP/poison_eval.tsv"
  while IFS=$'\t' read -r tid kind action result hash poison; do
    printf '%s\t%s\t-\n' "$tid" "$TESTDATA/rdfc10/$(basename "$action")" >> "$TMP/poison_fixtures.tsv"
  done < "$TMP/negative.tsv"
  for n in $CANON_CLIQUE_SIZES; do
    "$CANON_BENCH_BIN" gen-clique "$n" > "$TMP/clique$n.nq"
    printf 'clique%s\t%s\t-\n' "$n" "$TMP/clique$n.nq" >> "$TMP/poison_fixtures.tsv"
  done
fi

# Engine+posture rows: the JS peer is measured under BOTH its shipped default
# guard and its raised parity posture; the label carries the posture suffix.
POISON_RUNS=()
for eng in "${ENGINES[@]}"; do
  if [ "$eng" = rdf-canonize ]; then
    POISON_RUNS+=("rdf-canonize@default:" "rdf-canonize@wf${CANON_JS_PARITY_WF}:${CANON_JS_PARITY_WF}")
  else
    POISON_RUNS+=("$eng:")
  fi
done

: > "$TMP/poison.tsv"
while IFS=$'\t' read -r label in exp; do
  for run_spec in "${POISON_RUNS[@]}"; do
    col="${run_spec%%:*}"; wf="${run_spec#*:}"; eng="${col%%@*}"
    start=$(date +%s%N)
    rc=0; run_engine "$eng" "$in" 0 "$TMP/p.out" "$TMP/p.err" "$wf" || rc=$?
    wall_us=$(( ($(date +%s%N) - start) / 1000 ))
    # HARD-cap invariant: `timeout` bounds every run; a wall reading past
    # cap + 5s slack means the cap did NOT hold (harness bug — fail loudly).
    [ "$wall_us" -le $(( (CANON_CAP_S + 5) * 1000000 )) ] \
      || die "cap violated: $eng on $label took ${wall_us}us (cap ${CANON_CAP_S}s)"
    case "$rc" in
      0)   if [ "$exp" = "-" ]; then outcome=accepted;   # must-fail case answered
           elif cmp -s "$exp" "$TMP/p.out"; then outcome=ok
           else outcome=wrong; fi ;;
      2)   outcome=guard ;;
      124|137) outcome=capped ;;
      *)   outcome="error($rc)" ;;
    esac
    canon_us=$(stderr_metric "$TMP/p.err" canon_us)
    printf '%s\t%s\t%s\t%s\t%s\n' "$label" "$col" "$outcome" "$wall_us" "$canon_us" >> "$TMP/poison.tsv"
    log "PHASE B: $label / $col -> $outcome (wall ${wall_us}us)"
    # sparq soundness inside the panel: wrong bytes or accepting a must-fail
    # case contradicts the crate's W3C suite -> red. Peer divergence on the
    # poison set is a RECORDED comparative outcome, not a harness failure.
    if [ "$eng" = sparq ] && { [ "$outcome" = wrong ] || [ "$outcome" = accepted ]; }; then
      RED=1; log "SOUNDNESS FAIL: sparq '$outcome' on poison fixture $label"
    fi
    if [ "$SMOKE" = 1 ] && [ "$eng" = sparq ] && [ "$outcome" != guard ] && [ "$outcome" != capped ]; then
      RED=1; log "SMOKE FAIL: expected guard|capped on the negative poison fixture, got $outcome"
    fi
  done
done < "$TMP/poison_fixtures.tsv"

# ---- 5. PHASE C: advisory sane-set timing (full mode, parity-green engines only) ----------
: > "$TMP/timing.tsv"
if [ "$SMOKE" = 0 ]; then
  for eng in "${ENGINES[@]}"; do
    [ "$(awk -F'\t' -v e="$eng" '$1==e{print $3}' "$TMP/parity.tsv")" = 0 ] || continue
    wf="$(parity_wf "$eng")"
    while IFS=$'\t' read -r tid kind action result hash poison; do
      in="$TESTDATA/rdfc10/$(basename "$action")"
      s384=0; [ "$hash" = "SHA384" ] && s384=1
      best_wall=""; best_canon=""
      for _ in $(seq 1 "$CANON_ITERS"); do
        start=$(date +%s%N)
        run_engine "$eng" "$in" "$s384" "$TMP/t.out" "$TMP/t.err" "$wf" || die "timing rerun failed: $eng $tid"
        wall_us=$(( ($(date +%s%N) - start) / 1000 ))
        c=$(stderr_metric "$TMP/t.err" canon_us)
        if [ -z "$best_wall" ] || [ "$wall_us" -lt "$best_wall" ]; then best_wall=$wall_us; fi
        if [ "$c" != "-" ]; then
          if [ -z "$best_canon" ] || awk -v a="$c" -v b="$best_canon" 'BEGIN{exit !((a+0)<(b+0))}'; then
            best_canon=$c
          fi
        fi
      done
      printf '%s\t%s\t%s\t%s\n' "$tid" "$eng" "$best_wall" "${best_canon:--}" >> "$TMP/timing.tsv"
    done < "$TMP/sane.tsv"
    log "PHASE C: $eng timed on the sane set (min of $CANON_ITERS; process wall + in-process canon_us)"
  done
fi

# ---- 6. optional JSON envelope (full mode; land it in git-ignored competitor-results) -----
if [ -n "$CANON_JSON_OUT" ]; then
  # Engine versions, recorded at run time (never hard-coded).
  : > "$TMP/versions.tsv"
  printf 'sparq\t%s\n' "$(git -C "$ROOT" rev-parse --short HEAD 2>/dev/null || echo unknown)" >> "$TMP/versions.tsv"
  for eng in "${ENGINES[@]}"; do
    [ "$eng" = sparq ] && continue
    run_engine "$eng" "$TESTDATA/rdfc10/test001-in.nq" 0 "$TMP/v.out" "$TMP/v.err" || true
    printf '%s\t%s\n' "$eng" "$(stderr_metric "$TMP/v.err" version)" >> "$TMP/versions.tsv"
  done
  CANON_TMP="$TMP" CANON_CAP_S="$CANON_CAP_S" CANON_ITERS="$CANON_ITERS" CANON_RED="$RED" \
  CANON_JS_PARITY_WF="$CANON_JS_PARITY_WF" \
    python3 - "$CANON_JSON_OUT" <<'PYEOF'
import json, os, platform, subprocess, sys, time
tmp = os.environ["CANON_TMP"]
def rows(name, cols):
    out = []
    with open(os.path.join(tmp, name), encoding="utf-8") as f:
        for line in f:
            out.append(dict(zip(cols, line.rstrip("\n").split("\t"))))
    return out
envelope = {
    "suite": "canon-bench",
    "generated_at_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    "cap_s": int(os.environ["CANON_CAP_S"]),
    "iters": int(os.environ["CANON_ITERS"]),
    "red": os.environ["CANON_RED"] != "0",
    "host": {
        "platform": platform.platform(),
        "machine": platform.machine(),
        "nproc": os.cpu_count(),
        "quiet_box": False,  # flip only on a dedicated quiet gather box
        "note": "work-box timings are NON-canonical; see bench/CATALOG.md QUIET-BOX",
    },
    "engines": rows("versions.tsv", ["engine", "version"]),
    "parity": rows("parity.tsv", ["engine", "pass", "fail"]),
    "poison": rows("poison.tsv", ["fixture", "engine", "outcome", "wall_us", "canon_us"]),
    "timing": rows("timing.tsv", ["fixture", "engine", "wall_us_min", "canon_us_min"]),
    "timing_note": "wall_us includes process spawn (node startup dominates the JS column); "
                   "canon_us is the engine-reported in-process canonicalization time",
    "posture_note": "rdf-canonize parity/timing rows use maxWorkFactor="
                    + os.environ["CANON_JS_PARITY_WF"]
                    + " (its raised conformance posture); poison rows carry an explicit "
                      "@default / @wfN posture suffix. sparq and rdf-canon run as shipped "
                      "(default HNDQ call limit) in every phase.",
}
with open(sys.argv[1], "w", encoding="utf-8") as f:
    json.dump(envelope, f, indent=2)
    f.write("\n")
PYEOF
  log "envelope written to $CANON_JSON_OUT"
fi

if [ "$RED" = 0 ]; then
  log "OK: parity green on the sane set; poison outcomes recorded under the ${CANON_CAP_S}s cap"
else
  log "FAILED: see PARITY/SOUNDNESS/SMOKE FAIL lines above"
fi
exit "$RED"
