#!/usr/bin/env bash
# CI benchmark emitter (roadmap G2). Runs the engine over a freshly-generated synthetic dataset and
# emits results as github-action-benchmark `customSmallerIsBetter` JSON (all metrics are
# times/latencies — smaller is better), so perf is tracked + graphed across commits and a large
# regression raises an alert. Covers the main subsystems: load (parse+build), query (count +
# materialize = join compute), serialisation (json), and RDFS inference.
#
#   scripts/ci-bench.sh [scale_entities=200000] [out.json=bench-results.json]
#
# [OPUS-4.8] (sq-dzfu) CHEAP TIMING RE-MEASURE MODE for the perf-gate's best-of-N flake fix:
#   scripts/ci-bench.sh --parse-only [out.json=bench-results.json]
# emits ONLY the TIMING metric (parse_ns_per_byte) over the fixed corpus — NO wasm build, NO big-scale
# corpus, NO well-known suites — so the perf-gate (scripts/perf-gate.py --remeasure-cmd) can re-measure
# just the noisy wall-clock metric quickly when it trips the band, take the BEST reading, and only fail
# on a real (every-reading) regression. Reuses the exact same min-of-N parse measurement as the full run.
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
# [OPUS-4.8] (sq-dzfu) --parse-only: cheap TIMING re-measure for the perf-gate best-of-N flake fix.
# When present as the first arg, ONLY the parse_ns_per_byte metric is (re-)measured (see measure_parse
# below) and written out — no big corpus, no wasm build, no well-known suites.
PARSE_ONLY=0
if [ "${1:-}" = "--parse-only" ]; then
  PARSE_ONLY=1
  shift
  OUT="${1:-bench-results.json}"
  SCALE=0
else
  SCALE="${1:-200000}"
  OUT="${2:-bench-results.json}"
fi
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
# [OPUS-4.8] DBPSB/FEASIBLE (well-known suite) — per-commit subset over a pinned, fetched +
# cached DBpedia Databus slice (NO generator; fetch.sh verifies sha256 + emits a deterministic
# 750k-triple N-Triples cut). Trend-only latency like sp2b, PLUS a hard expected-rows
# correctness diff. Registry: bench/benchmarks.toml (dbpsb); details: bench/dbpsb/README.md.
DBPSB_FETCH=bench/dbpsb/fetch.sh
DBPSB_Q=bench/dbpsb/queries
DBPSB_EXP=bench/dbpsb/expected-rows.tsv
DBPSB_TRIPLES="${DBPSB_TRIPLES:-750000}"
# [OPUS-4.8] WatDiv (Waterloo SPARQL Diversity, sq-13i) — per-commit subset over the FIXED SF=1
# corpus from the real Waterloo generator (gen.sh; g++ + Boost). Trend-only latency like sp2b,
# PLUS a hard expected-rows correctness diff (count mode). Registry: bench/benchmarks.toml (watdiv);
# details: bench/watdiv/README.md. Mirrors the sp2b inline pattern.
WATDIV_GEN=bench/watdiv/gen.sh
WATDIV_Q=bench/watdiv/queries
WATDIV_EXP=bench/watdiv/expected-rows.tsv
WATDIV_SF="${WATDIV_SF:-1}"
# [OPUS-4.8] BSBM (Berlin SPARQL Benchmark, Explore mix) — per-commit subset over the FIXED -pc 300
# corpus from the prebuilt bsbmtools distribution (gen.sh; JRE + unzip). The Explore mix includes a
# CONSTRUCT (query12) + a DESCRIBE (query09), so the expected-rows diff runs against MATERIALIZE
# mode (graph-valued forms report produced-triple counts). Trend-only latency + hard correctness
# diff. Registry: bench/benchmarks.toml (bsbm); details: bench/bsbm/README.md.
BSBM_GEN=bench/bsbm/gen.sh
BSBM_Q=bench/bsbm/queries
BSBM_EXP=bench/bsbm/expected-rows.tsv
BSBM_PC="${BSBM_PC:-300}"
# [OPUS-4.8] LUBM (Lehigh University Benchmark) — the REASONING suite. Its run.sh is fully
# self-contained: it builds the LUBM(1) corpus (gen.sh; javac + rapper), materializes the OWL-RL
# closure with `sparq-cli reason`, runs BOTH the extensional + entailed tiers in count mode, and
# asserts every solution count vs expected-rows.tsv internally (exit 1 on mismatch). So the hook
# below just invokes run.sh (the self-asserting path) and harvests its count-mode TSV lines into
# `lubm_<query>_count_us`. Registry: bench/benchmarks.toml (lubm); details: bench/lubm/README.md.
LUBM_RUN=bench/lubm/run.sh
# [OPUS-4.8] Deep Taxonomy (DeepTaxonomy, sq-1hgz; parent sq-i0nm "design D") — the rule-heavy N3
# REASONING suite. Like LUBM, its run.sh is the self-asserting entry point: it reuses the existing
# generator (bench/inference/gen_deeptaxonomy.py via bench/deep-taxonomy/gen.sh), materializes the
# N3 forward closure per depth tier with `sparq-cli reason … n3`, runs query.rq over the closure,
# and asserts BOTH the closure triple-count AND the query row-count vs expected.tsv internally
# (exit 1 on any mismatch). So the hook below just invokes run.sh and harvests its metric TSV into
# `deeptax_d<DEPTH>_{closure_s,query_us,closure_triples}`. Registry: bench/benchmarks.toml
# (deep-taxonomy); details: bench/deep-taxonomy/README.md. (sq-1hgz)
DEEPTAX_RUN=bench/deep-taxonomy/run.sh
DEEPTAX_DEPTHS="${DEEPTAX_DEPTHS:-1000 10000}"
# [OPUS-4.8] SHACL VALIDATION suite (sq-7iai, design §3.1) — the cleanest competitor surface.
# Reuses the LUBM(1) ABox as its data substrate (so it shares the javac/rapper guard) x the 5
# committed shape graphs in bench/shacl/shapes/. Like LUBM, run.sh is the self-asserting entry
# point: it validates with examples/bench_shacl, asserts each workload's violations/conforms/
# focus_nodes vs expected.tsv internally (exit 1 on drift), and prints the
# `<workload>\t<violations>\t<validate_us>` contract on stdout, harvested below into the ADVISORY
# `shacl_<workload>_validate_us` (trend-only, NOT in scripts/perf-gate.py). Registry:
# bench/benchmarks.toml (shacl-validate-bench); details: bench/shacl/README.md.
SHACL_RUN=bench/shacl/run.sh
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
RES="$TMP/res.tsv"; : > "$RES"
add() { printf '%s\t%s\t%s\n' "$1" "$2" "$3" >> "$RES"; }

emit_json() {
  python3 - "$RES" <<'PY'
import sys, json
rows = [l.rstrip("\n").split("\t") for l in open(sys.argv[1]) if l.strip()]
print(json.dumps([{"name": n, "unit": u, "value": float(v)} for n, u, v in rows], indent=2))
PY
}

# [OPUS-4.8] (sq-dzfu) measure_parse — the TIMING metric (parse_ns_per_byte) over the FIXED corpus.
# Factored out so BOTH the full run and the cheap `--parse-only` re-measure path use the IDENTICAL
# measurement (min-of-5 parse timing on the deterministic byte count). Leaves $TMP/fe populated so the
# full run can still extract store_bytes_per_triple_small from the same load summary afterwards.
# PARSE_FIX is intentionally small so the min-of-N parse timing is the least-preempted (= most
# reproducible) wall-clock signal we can get without pulling the heavy bench/parse crate into CI.
measure_parse() {
  local PARSE_FIX="${PARSE_FIX:-50000}"
  "$GEN" dump "$PARSE_FIX" "$TMP/fix.nt" >/dev/null 2>&1
  local FIX_BYTES; FIX_BYTES=$(wc -c < "$TMP/fix.nt" | tr -d ' ')
  # parse throughput on the fixed corpus. github-action-benchmark's customSmallerIsBetter treats ALL
  # metrics as smaller-is-better, so we emit parse COST as nanoseconds-per-byte (= 1000 / MB/s): a
  # parse SLOWDOWN raises it and correctly trips the alert, whereas a raw MB/s metric would alert on
  # improvements and miss regressions. min-of-5 over the deterministic byte count.
  : > "$TMP/fixloads"
  for _ in 1 2 3 4 5; do
    "$CLI" bench "$TMP/fix.nt" ntriples "$Q" 1 count >/dev/null 2>"$TMP/fe" || true
    grep -oE 'in [0-9.]+s' "$TMP/fe" | head -1 | grep -oE '[0-9.]+' | head -1 >> "$TMP/fixloads" || true
  done
  local fixbest; fixbest=$(sort -n "$TMP/fixloads" | head -1)
  if [ -n "${fixbest:-}" ] && [ "${FIX_BYTES:-0}" -gt 0 ]; then
    local nspb; nspb=$(python3 -c "import sys;print(round(float(sys.argv[1])*1e9/float(sys.argv[2]),4))" "$fixbest" "$FIX_BYTES")
    add parse_ns_per_byte ns/byte "$nspb"
  fi
}

# [OPUS-4.8] (sq-dzfu) --parse-only: measure ONLY the TIMING metric and emit it, then exit. This is the
# cheap re-measure path the perf-gate shells out to (best-of-N) — it deliberately skips the big corpus,
# the wasm build, and every well-known suite so a re-measure of the noisy parse metric is fast.
if [ "$PARSE_ONLY" = "1" ]; then
  measure_parse
  emit_json > "$OUT"
  echo "wrote $OUT (--parse-only):"; cat "$OUT"
  exit 0
fi

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
# (the wall-clock query latencies above are trend-only).
#
# The parse_ns_per_byte (TIMING) measurement is factored into measure_parse() above so the perf-gate's
# best-of-N re-measure path (--parse-only) uses the IDENTICAL min-of-5 loop. It leaves $TMP/fe populated
# with the load summary so the deterministic store_bytes_per_triple_small can be extracted right after.
measure_parse

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

# [OPUS-4.8] DBPSB/FEASIBLE per-commit subset. fetch.sh downloads ONE sha256-pinned DBpedia
# Databus slice (mappingbased-objects_lang=en 2019.09.01, CC-BY-SA — see bench/dbpsb/README.md)
# and emits a FIXED, deterministic 750k-triple N-Triples cut, then runs the 13 sub-second
# curated FEASIBLE/DBPSB queries (BGP/star/chain joins, OPTIONAL, UNION, FILTER, DISTINCT,
# ORDER+LIMIT, aggregates, subquery, negation-by-OPTIONAL+!bound). Emits `dbpsb_<query>_<mode>_us`
# (trend-only, NOT in scripts/perf-gate.py) AND a HARD expected-rows equality check on count
# mode (a solution-count drift on the fixed cut is a correctness regression and fails the run).
# Guarded — if the slice can't be fetched (no network), the whole block is skipped gracefully so
# ci-bench still emits valid JSON for everything else. (CI keys actions/cache on the pinned
# URL+sha so the steady state does NO network — see bench/dbpsb/README.md "CI wiring".)
if [ -x "$DBPSB_FETCH" ] && [ -d "$DBPSB_Q" ]; then
  if DBPSB_CUT="$("$DBPSB_FETCH" "$DBPSB_TRIPLES" 2>/dev/null)" && [ -s "${DBPSB_CUT:-}" ]; then
    for mode in count materialize json; do
      "$CLI" bench "$DBPSB_CUT" ntriples "$DBPSB_Q" 3 "$mode" 2>/dev/null > "$TMP/dbpsb.$mode" || true
      while IFS=$'\t' read -r name rows us; do
        [ "${rows:-}" = "ERROR" ] && continue
        [ -n "${us:-}" ] && add "dbpsb_${name}_${mode}_us" us "$us"
      done < "$TMP/dbpsb.$mode"
    done
    # HARD differential: count-mode solution counts must match the committed expected sizes.
    if [ -f "$DBPSB_EXP" ]; then
      dbpsb_fail=0
      while IFS=$'\t' read -r q exp; do
        case "$q" in \#*|"") continue;; esac
        got=$(awk -F'\t' -v k="$q" '$1==k{print $2}' "$TMP/dbpsb.count")
        if [ "${got:-MISSING}" != "$exp" ]; then
          echo "ERROR: dbpsb $q rows=${got:-MISSING} expected=$exp (correctness regression)" >&2
          dbpsb_fail=1
        fi
      done < "$DBPSB_EXP"
      [ "$dbpsb_fail" = 0 ] || exit 1
    fi
  else
    echo "note: dbpsb skipped (pinned slice unavailable — no network?)" >&2
  fi
fi

# [OPUS-4.8] WatDiv per-commit subset (sq-13i). Builds+caches the real Waterloo generator and
# emits a FIXED, deterministic SF=1 corpus (N-Triples), then runs the 16 sub-ms Basic-Testing
# queries (Linear/Star/Snowflake/Complex; the 4 SF=1-empty ones live in queries-heavy/ for the
# EC2/nightly tier). Emits `watdiv_<query>_<mode>_us` (trend-only, NOT in scripts/perf-gate.py)
# AND a HARD expected-rows equality check on count mode (a solution-count drift on the fixed
# corpus is a correctness regression and fails the run). Guarded on g++ (the generator needs
# g++ + Boost; gen.sh handles Boost) — if g++/the generator is unavailable, the whole block is
# skipped gracefully so ci-bench still emits valid JSON for everything else. (CI keys
# actions/cache on /tmp/watdiv so the steady state does NO rebuild — see bench/watdiv/README.md.)
if command -v g++ >/dev/null 2>&1 && [ -x "$WATDIV_GEN" ] && [ -d "$WATDIV_Q" ]; then
  if WATDIV_CORPUS="$(bash "$WATDIV_GEN" "$WATDIV_SF" 2>/dev/null)" && [ -s "${WATDIV_CORPUS:-}" ]; then
    for mode in count materialize json; do
      "$CLI" bench "$WATDIV_CORPUS" ntriples "$WATDIV_Q" 3 "$mode" 2>/dev/null > "$TMP/watdiv.$mode" || true
      while IFS=$'\t' read -r name rows us; do
        [ "${rows:-}" = "ERROR" ] && continue
        [ -n "${us:-}" ] && add "watdiv_${name}_${mode}_us" us "$us"
      done < "$TMP/watdiv.$mode"
    done
    # HARD differential: count-mode solution counts must match the committed expected sizes.
    if [ -f "$WATDIV_EXP" ]; then
      watdiv_fail=0
      while IFS=$'\t' read -r q exp; do
        case "$q" in \#*|"") continue;; esac
        got=$(awk -F'\t' -v k="$q" '$1==k{print $2}' "$TMP/watdiv.count")
        if [ "${got:-MISSING}" != "$exp" ]; then
          echo "ERROR: watdiv $q rows=${got:-MISSING} expected=$exp (correctness regression)" >&2
          watdiv_fail=1
        fi
      done < "$WATDIV_EXP"
      [ "$watdiv_fail" = 0 ] || exit 1
    fi
  else
    echo "note: watdiv skipped (generator unavailable — no network/g++/boost?)" >&2
  fi
else
  echo "note: watdiv skipped (g++ not on PATH)" >&2
fi

# [OPUS-4.8] BSBM Explore-mix per-commit subset. gen.sh fetches+caches the prebuilt bsbmtools
# distribution (JRE-only; sha256-pinned zip) and emits a FIXED, deterministic -pc 300 corpus
# (~116k N-Triples), then runs the 11 sub-ms Explore queries. The Explore mix includes a CONSTRUCT
# (query12) + a DESCRIBE (query09), so the HARD expected-rows diff runs against MATERIALIZE mode
# (the graph-valued forms report produced-triple counts through the bench runner; counts are
# mode-independent for this mix). Emits `bsbm_<query>_<mode>_us` (trend-only, NOT in
# scripts/perf-gate.py). Guarded on java + unzip (gen.sh needs a JRE to run the generator and
# unzip to unpack the distribution) — if either is absent, the whole block is skipped gracefully
# so ci-bench still emits valid JSON. (CI keys actions/cache on /tmp/bsbm — see bench/bsbm/README.md.)
if command -v java >/dev/null 2>&1 && command -v unzip >/dev/null 2>&1 && [ -x "$BSBM_GEN" ] && [ -d "$BSBM_Q" ]; then
  if BSBM_CORPUS="$(bash "$BSBM_GEN" "$BSBM_PC" 2>/dev/null)" && [ -s "${BSBM_CORPUS:-}" ]; then
    for mode in count materialize json; do
      "$CLI" bench "$BSBM_CORPUS" ntriples "$BSBM_Q" 3 "$mode" 2>/dev/null > "$TMP/bsbm.$mode" || true
      while IFS=$'\t' read -r name rows us; do
        [ "${rows:-}" = "ERROR" ] && continue
        [ -n "${us:-}" ] && add "bsbm_${name}_${mode}_us" us "$us"
      done < "$TMP/bsbm.$mode"
    done
    # HARD differential: result sizes must match the committed expected sizes. The mix includes a
    # CONSTRUCT + a DESCRIBE, so we diff MATERIALIZE mode (produced-triple counts for graph forms).
    if [ -f "$BSBM_EXP" ]; then
      bsbm_fail=0
      while IFS=$'\t' read -r q exp; do
        case "$q" in \#*|"") continue;; esac
        got=$(awk -F'\t' -v k="$q" '$1==k{print $2}' "$TMP/bsbm.materialize")
        if [ "${got:-MISSING}" != "$exp" ]; then
          echo "ERROR: bsbm $q rows=${got:-MISSING} expected=$exp (correctness regression)" >&2
          bsbm_fail=1
        fi
      done < "$BSBM_EXP"
      [ "$bsbm_fail" = 0 ] || exit 1
    fi
  else
    echo "note: bsbm skipped (generator unavailable — no network/java?)" >&2
  fi
else
  echo "note: bsbm skipped (java/unzip not on PATH)" >&2
fi

# [OPUS-4.8] LUBM reasoning suite per-commit. Unlike watdiv/bsbm (inline sp2b-style), LUBM's
# run.sh is the self-asserting entry point: it ensures the LUBM(1) corpus + Univ-Bench TBox
# (gen.sh), materializes the OWL-RL closure with `sparq-cli reason`, runs the extensional tier on
# the raw ABox + the entailed tier on the closure (count mode), and asserts EVERY count vs
# expected-rows.tsv internally (exit 1 on any mismatch). So we invoke run.sh and harvest its
# count-mode TSV (it prints `<query>\t<rows>\t<min_us>` lines for both tiers on stdout) into
# `lubm_<query>_count_us` (trend-only, NOT in scripts/perf-gate.py). Guarded on javac + rapper
# (gen.sh compiles the UBA generator with javac and converts RDF/XML via rapper) — if either is
# absent, skipped gracefully so ci-bench still emits valid JSON. run.sh failing (a reasoner OR
# engine correctness regression) fails the whole ci-bench run. (CI keys actions/cache on /tmp/lubm.)
if command -v javac >/dev/null 2>&1 && command -v rapper >/dev/null 2>&1 && [ -x "$LUBM_RUN" ]; then
  if CLI="$CLI" ITERS=3 bash "$LUBM_RUN" > "$TMP/lubm.tsv" 2>"$TMP/lubm.err"; then
    while IFS=$'\t' read -r name rows us; do
      [ "${rows:-}" = "ERROR" ] && continue
      [ -n "${us:-}" ] && add "lubm_${name}_count_us" us "$us"
    done < "$TMP/lubm.tsv"
  else
    # run.sh exits non-zero ONLY on a hard correctness mismatch (the gen/tool guards above already
    # ensure the toolchain is present), so surface its diagnostics and fail the run.
    echo "ERROR: lubm run.sh failed (correctness regression or reasoner/engine error)" >&2
    cat "$TMP/lubm.err" >&2 || true
    exit 1
  fi
else
  echo "note: lubm skipped (javac/rapper not on PATH)" >&2
fi

# [OPUS-4.8] Deep Taxonomy reasoning suite per-commit (sq-1hgz). Like the LUBM hook above, this
# suite's run.sh is the self-asserting entry point: it reuses bench/inference/gen_deeptaxonomy.py
# (via gen.sh) to build the DT N3 corpus at each depth tier, materializes the N3 forward closure
# with `sparq-cli reason … n3`, runs query.rq over the closure, and asserts EVERY closure
# triple-count + query row-count vs expected.tsv internally (exit 1 on any mismatch). So we invoke
# run.sh and harvest its metric TSV (it prints `<metric>\t<value>\t<unit>` lines on stdout) into
# `deeptax_d<DEPTH>_{closure_s,query_us,closure_triples}` (trend-only, NOT in scripts/perf-gate.py;
# closure_triples is a deterministic structural metric, never an alert source). UNLIKE
# watdiv/bsbm/lubm this suite needs NO heavyweight toolchain — only python3 (always present) + the
# built sparq-cli — so it runs on the per-commit tier by default at a SMALL depth pair (dt1k+dt10k;
# closures 2,001 / 20,001 triples, ~0.1 s total). dt100k is opt-in via DEEPTAX_DEPTHS for the
# EC2/nightly tier. Guarded on python3 + run.sh present, mirroring the LUBM guard shape: if python3
# is somehow absent it is skipped gracefully so ci-bench still emits valid JSON. run.sh failing (a
# reasoner OR engine correctness regression) fails the whole ci-bench run.
if command -v python3 >/dev/null 2>&1 && [ -x "$DEEPTAX_RUN" ]; then
  if CLI="$CLI" ITERS=3 DEEPTAX_DEPTHS="$DEEPTAX_DEPTHS" bash "$DEEPTAX_RUN" > "$TMP/deeptax.tsv" 2>"$TMP/deeptax.err"; then
    while IFS=$'\t' read -r name value unit; do
      [ -n "${name:-}" ] && [ -n "${value:-}" ] && add "$name" "${unit:-}" "$value"
    done < "$TMP/deeptax.tsv"
  else
    # run.sh exits non-zero ONLY on a hard correctness mismatch (the python3 guard above ensures
    # the toolchain is present), so surface its diagnostics and fail the run.
    echo "ERROR: deep-taxonomy run.sh failed (closure-size regression or reasoner/engine error)" >&2
    cat "$TMP/deeptax.err" >&2 || true
    exit 1
  fi
else
  echo "note: deep-taxonomy skipped (python3 not on PATH)" >&2
fi

# [OPUS-4.8] SHACL validation suite per-commit (sq-7iai). Reuses the LUBM(1) ABox, so it shares
# the javac + rapper guard (gen.sh delegates to bench/lubm/gen.sh). run.sh is the self-asserting
# entry point: it validates the ABox against the 5 committed shape graphs with examples/bench_shacl
# and asserts each workload's violations/conforms/focus_nodes vs expected.tsv internally (exit 1 on
# any drift). The example is built on demand (sparq-shacl is isolated — not a sparq-cli dependency —
# so the bench runner is its own --example binary). We harvest run.sh's `<workload>\t<violations>\t
# <validate_us>` stdout into the ADVISORY `shacl_<workload>_validate_us` (trend-only, NOT in
# scripts/perf-gate.py). run.sh failing (a SHACL correctness regression) fails the whole ci-bench
# run, exactly like LUBM. Skipped gracefully when javac/rapper are absent (PRs install no toolchain).
SHACL_BIN=target/release/examples/bench_shacl
if command -v javac >/dev/null 2>&1 && command -v rapper >/dev/null 2>&1 && [ -x "$SHACL_RUN" ]; then
  # Always (re)build: rust-cache restores target/, so a file-exists check can run a STALE
  # bench_shacl. Let cargo's own staleness detection decide (a no-op rebuild is cheap), and do
  # NOT swallow the build error (`|| true`) — a real compile failure must fail the gate, not
  # silently skip the SHACL correctness check on main. Mirrors how the other suites assume a
  # freshly-built binary.
  if ! cargo build --release -q -p sparq-shacl --example bench_shacl; then
    echo "ERROR: bench_shacl example failed to build" >&2
    exit 1
  fi
  if BENCH_SHACL="$SHACL_BIN" ITERS=3 bash "$SHACL_RUN" > "$TMP/shacl.tsv" 2>"$TMP/shacl.err"; then
    while IFS=$'\t' read -r name violations us; do
      [ "${violations:-}" = "ERROR" ] && continue
      [ -n "${us:-}" ] && add "shacl_${name}_validate_us" us "$us"
    done < "$TMP/shacl.tsv"
  else
    echo "ERROR: shacl run.sh failed (correctness regression)" >&2
    cat "$TMP/shacl.err" >&2 || true
    exit 1
  fi
else
  echo "note: shacl skipped (javac/rapper not on PATH)" >&2
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

emit_json > "$OUT"
echo "wrote $OUT:"; cat "$OUT"
