#!/usr/bin/env bash
# CI benchmark emitter (roadmap G2). Runs the engine over a freshly-generated synthetic dataset and
# emits results as github-action-benchmark `customSmallerIsBetter` JSON (all metrics are
# times/latencies — smaller is better), so perf is tracked + graphed across commits and a large
# regression raises an alert. Covers the main subsystems: load (parse+build), query (count +
# materialize = join compute), serialisation (json), and RDFS inference.
#
#   scripts/ci-bench.sh [scale_entities=200000] [out.json=bench-results.json]
#
# GitHub-hosted runners are small + noisy, so absolute numbers drift; the value is the cross-commit
# TREND and large-regression alerting (the workflow sets a generous threshold + fail-on-alert=false).
#
# [OPUS-4.8] The load-bearing REGRESSION GATES are the DETERMINISTIC (runner-noise-immune) metrics —
# store/dict bytes-per-{triple,term} (memory layout), wasm_bundle_bytes, and two added on a FIXED
# corpus: store_bytes_per_triple_small (a SECOND scale, catches per-triple-overhead regressions the
# primary scale hides) and parse_ns_per_byte (parse cost on a fixed byte count; emitted as ns/byte =
# 1000/(MB/s) so it reads smaller-is-better under customSmallerIsBetter). Wall-clock latencies stay
# trend-only on free runners.
set -euo pipefail
cd "$(dirname "$0")/.."
SCALE="${1:-200000}"
OUT="${2:-bench-results.json}"
CLI=target/release/sparq-cli
GEN=target/release/sparq-bench
Q=bench/qlever-synthetic/queries
# [OPUS-4.8] Operator-coverage suite (one query per SPARQL operator family). Wall-clock /
# trend-only like the query latencies above — NOT a hard perf-gate (only the byte/parse
# metrics in scripts/perf-gate.py are gated). Registry: bench/benchmarks.toml (operator-coverage).
OPQ=bench/operators/queries
# [OPUS-4.8] SP2Bench (well-known suite) — per-commit subset + fixed-corpus generator.
# Trend-only latency like the operator suite, PLUS a hard expected-rows correctness diff.
# Registry: bench/benchmarks.toml (sp2b); details: bench/sp2b/README.md. (sq-0jp)
SP2B_GEN=bench/sp2b/gen.sh
SP2B_Q=bench/sp2b/queries
SP2B_EXP=bench/sp2b/expected-rows.tsv
SP2B_TRIPLES="${SP2B_TRIPLES:-250000}"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
RES="$TMP/res.tsv"; : > "$RES"
add() { printf '%s\t%s\t%s\n' "$1" "$2" "$3" >> "$RES"; }

"$GEN" dump "$SCALE" "$TMP/data.nt" >/dev/null 2>&1

# load (seconds, smaller better) — the "loaded … in Xs" line goes to stderr; min over 3 runs.
: > "$TMP/loads"
for _ in 1 2 3; do
  "$CLI" bench "$TMP/data.nt" ntriples "$Q" 1 count >/dev/null 2>"$TMP/e" || true
  grep -oE 'in [0-9.]+s' "$TMP/e" | head -1 | grep -oE '[0-9.]+' | head -1 >> "$TMP/loads" || true
done
loadbest=$(sort -n "$TMP/loads" | head -1)
[ -n "${loadbest:-}" ] && add load_s s "$loadbest"

# Memory regression metrics (DETERMINISTIC — not runner-noise) from the load summary line
# "store ~X GB (W B/triple), dict ~Y GB (… Z B/term)". These directly track memory efficiency
# across commits, satisfying the "no regressions in memory usage" requirement.
btriple=$(grep -oE '\([0-9]+ B/triple\)' "$TMP/e" | head -1 | grep -oE '[0-9]+' | head -1)
[ -n "${btriple:-}" ] && add store_bytes_per_triple bytes "$btriple"
bterm=$(grep -oE '[0-9]+ B/term' "$TMP/e" | head -1 | grep -oE '[0-9]+' | head -1)
[ -n "${bterm:-}" ] && add dict_bytes_per_term bytes "$bterm"

# [OPUS-4.8] Two more DETERMINISTIC (runner-noise-immune) regression GATES on a FIXED corpus.
# The bytes-per-triple figures vary with scale (fixed per-graph overhead amortises differently),
# so a SECOND, fixed scale catches per-triple-overhead regressions that the primary scale hides;
# and a parse-cost metric on a FIXED byte count tracks raw ingest throughput. The corpus comes
# from the DETERMINISTIC synthetic generator (same bytes every run, independent of $SCALE), so
# its size + memory layout are reproducible — these are the load-bearing gates on noisy runners
# (the wall-clock query latencies above are trend-only). PARSE_FIX is intentionally small so the
# min-of-N parse timing is the least-preempted (= most reproducible) wall-clock signal we can get
# without pulling the heavy standalone bench/parse crate into per-commit CI.
PARSE_FIX="${PARSE_FIX:-50000}"
"$GEN" dump "$PARSE_FIX" "$TMP/fix.nt" >/dev/null 2>&1
FIX_BYTES=$(wc -c < "$TMP/fix.nt" | tr -d ' ')

# parse throughput on the fixed corpus. github-action-benchmark's customSmallerIsBetter treats
# ALL metrics as smaller-is-better, so we emit parse COST as nanoseconds-per-byte (= 1000 / MB/s):
# a parse SLOWDOWN raises it and correctly trips the alert, whereas a raw MB/s metric would alert
# on improvements and miss regressions. min-of-5 over the deterministic byte count.
: > "$TMP/fixloads"
for _ in 1 2 3 4 5; do
  "$CLI" bench "$TMP/fix.nt" ntriples "$Q" 1 count >/dev/null 2>"$TMP/fe" || true
  grep -oE 'in [0-9.]+s' "$TMP/fe" | head -1 | grep -oE '[0-9.]+' | head -1 >> "$TMP/fixloads" || true
done
fixbest=$(sort -n "$TMP/fixloads" | head -1)
if [ -n "${fixbest:-}" ] && [ "${FIX_BYTES:-0}" -gt 0 ]; then
  nspb=$(python3 -c "import sys;print(round(float(sys.argv[1])*1e9/float(sys.argv[2]),4))" "$fixbest" "$FIX_BYTES")
  add parse_ns_per_byte ns/byte "$nspb"
fi

# store bytes-per-triple at the SECOND (fixed) scale — fully deterministic memory layout.
btriple2=$(grep -oE '\([0-9]+ B/triple\)' "$TMP/fe" | head -1 | grep -oE '[0-9]+' | head -1)
[ -n "${btriple2:-}" ] && add store_bytes_per_triple_small bytes "$btriple2"

# query latencies — bench reports min-of-iters micros per query (TSV: name<TAB>rows<TAB>micros).
for mode in count materialize json; do
  "$CLI" bench "$TMP/data.nt" ntriples "$Q" 3 "$mode" 2>/dev/null > "$TMP/o" || true
  while IFS=$'\t' read -r name _rows us; do
    [ -n "${us:-}" ] && add "${name}_${mode}_us" us "$us"
  done < "$TMP/o"
done

# [OPUS-4.8] operator-coverage latencies — one *.rq per SPARQL operator family
# (BGP/star/chain/triangle, UNION, OPTIONAL/!bound, MINUS, FILTER {num,string,IN,EXISTS},
# BIND, VALUES, aggregates+GROUP BY+HAVING, DISTINCT, ORDER BY+LIMIT/OFFSET, property paths
# {+,*,?,seq,alt,inverse,negated-set}, subquery, ASK/CONSTRUCT/DESCRIBE). Same TSV runner +
# naming as the query latencies, namespaced `op_<query>_<mode>_us`. Graph forms (CONSTRUCT/
# DESCRIBE) are routed by `bench` to the construct/describe executor (rows = produced triples).
# Wall-clock / trend-only: these are NOT in the deterministic perf-gate (scripts/perf-gate.py).
if [ -d "$OPQ" ]; then
  for mode in count materialize json; do
    "$CLI" bench "$TMP/data.nt" ntriples "$OPQ" 3 "$mode" 2>/dev/null > "$TMP/op" || true
    while IFS=$'\t' read -r name rows us; do
      # skip any query that errored (rows == ERROR) so the JSON stays valid + numeric.
      [ "${rows:-}" = "ERROR" ] && continue
      [ -n "${us:-}" ] && add "op_${name}_${mode}_us" us "$us"
    done < "$TMP/op"
  done
fi

# [OPUS-4.8] SP2Bench per-commit subset (sq-0jp). Builds+caches the real Freiburg sp2b_gen
# and generates a FIXED, deterministic 250k-triple DBLP-in-RDF corpus (Turtle), then runs the
# 14 sub-second canonical queries (the 3 pathological ones — q05a/q06/q12a — are in
# queries-heavy/ for the EC2/nightly tier). Emits `sp2b_<query>_<mode>_us` (trend-only, NOT in
# scripts/perf-gate.py) AND a HARD expected-rows equality check on count mode: a solution-count
# drift on the fixed corpus is a correctness regression and fails the run. The whole block is
# guarded — if the generator can't be fetched/built (no network/g++), it is skipped gracefully
# so ci-bench still emits valid JSON for the rest of the metrics.
if [ -x "$SP2B_GEN" ] && [ -d "$SP2B_Q" ]; then
  if SP2B_CORPUS="$("$SP2B_GEN" "$SP2B_TRIPLES" 2>/dev/null)" && [ -s "${SP2B_CORPUS:-}" ]; then
    for mode in count materialize json; do
      "$CLI" bench "$SP2B_CORPUS" turtle "$SP2B_Q" 3 "$mode" 2>/dev/null > "$TMP/sp2b.$mode" || true
      while IFS=$'\t' read -r name rows us; do
        [ "${rows:-}" = "ERROR" ] && continue
        [ -n "${us:-}" ] && add "sp2b_${name}_${mode}_us" us "$us"
      done < "$TMP/sp2b.$mode"
    done
    # HARD differential: count-mode solution counts must match the committed expected sizes.
    if [ -f "$SP2B_EXP" ]; then
      sp2b_fail=0
      while IFS=$'\t' read -r q exp; do
        case "$q" in \#*|"") continue;; esac
        got=$(awk -F'\t' -v k="$q" '$1==k{print $2}' "$TMP/sp2b.count")
        if [ "${got:-MISSING}" != "$exp" ]; then
          echo "ERROR: sp2b $q rows=${got:-MISSING} expected=$exp (correctness regression)" >&2
          sp2b_fail=1
        fi
      done < "$SP2B_EXP"
      [ "$sp2b_fail" = 0 ] || exit 1
    fi
  else
    echo "note: sp2b skipped (generator unavailable — no network/g++?)" >&2
  fi
fi

# RDFS inference (seconds) — instance-heavy: SCALE individuals under a depth-20 class hierarchy.
{ echo '@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .'; echo '@prefix : <http://ex/> .'
  for k in $(seq 0 19); do echo ":c$k rdfs:subClassOf :c$((k+1)) ."; done
  for j in $(seq 1 "$SCALE"); do echo ":x$j a :c0 ."; done; } > "$TMP/inf.ttl"
infs=$("$CLI" reason "$TMP/inf.ttl" turtle rdfs 2>&1 | grep -oE 'in [0-9.]+s' | grep -oE '[0-9.]+' | head -1)
[ -n "${infs:-}" ] && add rdfs_infer_s s "$infs"

# WASM bundle size (bytes, deterministic) — enforces the "zero wasm bundle impact" rule for
# opt-in features: any feature that leaks into the browser bundle shows up here per commit.
# Skipped gracefully when the wasm target isn't installed.
if rustup target list --installed 2>/dev/null | grep -q wasm32-unknown-unknown; then
  if cargo build --release -q -p sparq-wasm --target wasm32-unknown-unknown 2>/dev/null; then
    WASM_BIN=$(ls target/wasm32-unknown-unknown/release/*.wasm 2>/dev/null | head -1)
    if [ -n "${WASM_BIN:-}" ]; then
      add wasm_bundle_bytes bytes "$(wc -c < "$WASM_BIN" | tr -d ' ')"
    fi
  fi
fi

python3 - "$RES" > "$OUT" <<'PY'
import sys, json
rows = [l.rstrip("\n").split("\t") for l in open(sys.argv[1]) if l.strip()]
print(json.dumps([{"name": n, "unit": u, "value": float(v)} for n, u, v in rows], indent=2))
PY
echo "wrote $OUT:"; cat "$OUT"
