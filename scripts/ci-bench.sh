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
set -euo pipefail
cd "$(dirname "$0")/.."
SCALE="${1:-200000}"
OUT="${2:-bench-results.json}"
CLI=target/release/sparq-cli
GEN=target/release/sparq-bench
Q=bench/qlever-synthetic/queries
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

# query latencies — bench reports min-of-iters micros per query (TSV: name<TAB>rows<TAB>micros).
for mode in count materialize json; do
  "$CLI" bench "$TMP/data.nt" ntriples "$Q" 3 "$mode" 2>/dev/null > "$TMP/o" || true
  while IFS=$'\t' read -r name _rows us; do
    [ -n "${us:-}" ] && add "${name}_${mode}_us" us "$us"
  done < "$TMP/o"
done

# RDFS inference (seconds) — instance-heavy: SCALE individuals under a depth-20 class hierarchy.
{ echo '@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .'; echo '@prefix : <http://ex/> .'
  for k in $(seq 0 19); do echo ":c$k rdfs:subClassOf :c$((k+1)) ."; done
  for j in $(seq 1 "$SCALE"); do echo ":x$j a :c0 ."; done; } > "$TMP/inf.ttl"
infs=$("$CLI" reason "$TMP/inf.ttl" turtle rdfs 2>&1 | grep -oE 'in [0-9.]+s' | grep -oE '[0-9.]+' | head -1)
[ -n "${infs:-}" ] && add rdfs_infer_s s "$infs"

python3 - "$RES" > "$OUT" <<'PY'
import sys, json
rows = [l.rstrip("\n").split("\t") for l in open(sys.argv[1]) if l.strip()]
print(json.dumps([{"name": n, "unit": u, "value": float(v)} for n, u, v in rows], indent=2))
PY
echo "wrote $OUT:"; cat "$OUT"
