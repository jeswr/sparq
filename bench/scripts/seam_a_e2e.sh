#!/usr/bin/env bash
# [SONNET-4.6] (sq-pntvh.9) Seam-A end-to-end A/B measurement.
#
# 🤖 SPARQ agent — Measures the exec.rs columnar_filter seam (M4 Phase 5, Seam A)
# end-to-end via two builds of sparq-cli:
#   - feature-OFF: default sparq-cli (scalar row path throughout)
#   - feature-ON:  sparq-cli with --features sparq-engine/vectorized (columnar_filter
#                  seam active for eligible join->FILTER shapes above VEC_MIN_BATCH=256)
#
# WIN criterion (gates sq-pntvh.6): seam_a_filter_on_us < seam_a_filter_off_us
# NO-WIN criterion: ON >= OFF  (honestly defers Phase 6/7 per wiring-roadmap sec8.2)
#
# NOTE — GROUP-BY shape: Phase 4 (sq-pntvh.4) columnar aggregate reducer is NOT yet wired;
# seam_a_groupby_on_us and seam_a_groupby_off_us are expected EQUAL/CLOSE — this is the
# honest before-Phase-4 baseline for the reducer seam.
#
# 64KB SERIAL-CONSOLE DISCIPLINE: build output piped to tail; only metric_us lines +
# verdict emitted to stdout. Use > /dev/console or per bench/ec2-bench.sh pattern.
#
# ORPHAN-SAFETY: this script is run inside a user-data script launched by ec2-bench.sh;
# the outer launcher already installs the watchdog + --instance-initiated-shutdown-behavior
# terminate. Never call this script with explicit `shutdown -h now` inside it.
#
# Usage: bench/scripts/seam_a_e2e.sh [N=500000] [ITERS=5]
#   N:     corpus size (number of subjects). Must be > VEC_MIN_BATCH*10=2560 for a
#          meaningful Seam-A measurement (default 500000 >> VEC_MIN_BATCH=256).
#   ITERS: min-of-N iterations for timing (default 5).
#
# Emits metric_us lines (compatible with ec2-bench.sh/bench-adapters):
#   metric_us seam_a_filter_off=<us>    <- join->FILTER, scalar path (feature-OFF)
#   metric_us seam_a_filter_on=<us>     <- join->FILTER, columnar path (feature-ON)
#   metric_us seam_a_groupby_off=<us>   <- join->GROUP-BY reducer, scalar (feature-OFF)
#   metric_us seam_a_groupby_on=<us>    <- join->GROUP-BY reducer, columnar (feature-ON)
# Plus a final summary line (NOT a metric_us line):
#   seam_a_verdict: WIN/NO-WIN ...      <- compared on the FILTER shape (Seam A)
#
# No perf numbers are committed to the repo; EC2 output goes to the sq-pntvh bead
# comment. Non-canonical: work-box timings differ from a quiet EC2 c6i.2xlarge.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
N="${1:-500000}"
ITERS="${2:-5}"

# FILTER threshold = 70th percentile: keeps ~30% of rows, always >> VEC_MIN_BATCH=256
# for any N >= 1000 (0.30 * 1000 = 300 > 256).
THRESHOLD=$(( N * 7 / 10 ))

BENCH_TMP="$(mktemp -d /tmp/seam-a-bench.XXXXXX)"
trap 'rm -rf "$BENCH_TMP"' EXIT

# ---- 1. Generate synthetic corpus --------------------------------------------
# Two triples per subject: (s, ex:age, i) + (s, ex:tag, i%10).
# Shape rationale: the join ?s ex:age ?age . ?s ex:tag ?tag produces N rows, all
# with inline-integer ?age ids, satisfying the VEC_MIN_BATCH and dtype-precheck
# eligibility gates of columnar_filter (exec.rs §Decline hierarchy).
echo "[seam-a-e2e] generating corpus N=${N}..." >&2
python3 - "$N" "$BENCH_TMP/corpus.nt" <<'PY'
import sys
n = int(sys.argv[1])
# N-Triples requires no space between ^^ and the datatype IRI.
xsd_int = '<http://www.w3.org/2001/XMLSchema#integer>'
with open(sys.argv[2], 'w') as fh:
    for i in range(n):
        s = '<http://ex/s{}>'.format(i)
        fh.write('{} <http://ex/age> "{}"^^{} .\n'.format(s, i, xsd_int))
        fh.write('{} <http://ex/tag> "{}"^^{} .\n'.format(s, i % 10, xsd_int))
PY

# ---- 2. SPARQL query files ---------------------------------------------------
mkdir -p "$BENCH_TMP/queries"

# Shape 1: join (two BGP patterns) + residual numeric FILTER on inline-integer column.
# After the join, all N rows are eligible for Seam-A dispatch (>= VEC_MIN_BATCH,
# inline-integer ids, single sargable FILTER expression). With THRESHOLD = N*0.7,
# ~30% of rows survive, keeping the result non-trivial.
cat > "$BENCH_TMP/queries/filter.rq" <<RQ
SELECT ?s ?age WHERE {
  ?s <http://ex/age> ?age .
  ?s <http://ex/tag> ?tag .
  FILTER(?age > ${THRESHOLD})
}
RQ

# Shape 2: join + GROUP-BY reducer aggregate. NOT yet columnar-wired (Phase 4 pending;
# sq-pntvh.4). Expected equal timing across feature states: honest baseline for Phase 4.
cat > "$BENCH_TMP/queries/groupby.rq" <<'RQ'
SELECT ?tag (SUM(?age) AS ?total) WHERE {
  ?s <http://ex/tag> ?tag .
  ?s <http://ex/age> ?age .
} GROUP BY ?tag
RQ

# ---- 3. Build feature-OFF binary (default sparq-cli) -------------------------
echo "[seam-a-e2e] building feature-OFF sparq-cli..." >&2
cargo build -p sparq-cli --release 2>&1 | tail -3
CLI_OFF="$ROOT/target/release/sparq-cli"

# ---- 4. Build feature-ON binary (sparq-engine/vectorized forwarded) ----------
# Separate --target-dir so the two feature states do not share an artifact cache
# (different code paths compile into the same path with the default target-dir).
VEC_TARGET="/tmp/seam-a-vec"
echo "[seam-a-e2e] building feature-ON (sparq-engine/vectorized)..." >&2
cargo build -p sparq-cli --release \
    --features sparq-engine/vectorized \
    --target-dir "$VEC_TARGET" 2>&1 | tail -3
CLI_ON="$VEC_TARGET/release/sparq-cli"

# ---- 5. Run A/B measurements -------------------------------------------------
echo "[seam-a-e2e] running feature-OFF measurements (iters=${ITERS})..." >&2
"$CLI_OFF" bench "$BENCH_TMP/corpus.nt" ntriples "$BENCH_TMP/queries" "$ITERS" count \
    > "$BENCH_TMP/off.tsv" 2>/dev/null

echo "[seam-a-e2e] running feature-ON measurements (iters=${ITERS})..." >&2
"$CLI_ON" bench "$BENCH_TMP/corpus.nt" ntriples "$BENCH_TMP/queries" "$ITERS" count \
    > "$BENCH_TMP/on.tsv" 2>/dev/null

# ---- 6. Emit metric_us lines -------------------------------------------------
# Emit for both feature states; suffix denotes the state.
emit_metrics() {
    local suffix="$1" tsv="$2"
    while IFS=$'\t' read -r name rows us; do
        [ -n "$name" ] || continue
        case "$us" in
            ERROR*) echo "[seam-a-e2e] QUERY ERROR $name: $rows" >&2 ;;
            *)      printf 'metric_us seam_a_%s_%s=%.0f\n' "$name" "$suffix" "$us" ;;
        esac
    done < "$tsv"
}

emit_metrics "off" "$BENCH_TMP/off.tsv"
emit_metrics "on"  "$BENCH_TMP/on.tsv"

# ---- 7. WIN / NO-WIN verdict -------------------------------------------------
# Compare the FILTER shape only (Seam A). GROUP-BY is the Phase-4 baseline (not gated here).
filter_off="$(awk -F'\t' '$1=="filter"{print $3}' "$BENCH_TMP/off.tsv")"
filter_on="$(awk -F'\t' '$1=="filter"{print $3}' "$BENCH_TMP/on.tsv")"

awk -v off="$filter_off" -v on="$filter_on" 'BEGIN {
    if (off+0 == 0 || on+0 == 0) {
        print "seam_a_verdict: UNKNOWN  (missing filter timing — check stderr for query errors)"
        exit
    }
    if (on < off) {
        pct = (off - on) / off * 100
        printf "seam_a_verdict: WIN  filter_on=%.0fus < filter_off=%.0fus (approx %.0f%% faster)\n", on, off, pct
    } else {
        pct = (on - off) / off * 100
        printf "seam_a_verdict: NO-WIN  filter_on=%.0fus >= filter_off=%.0fus (overhead approx %.0f%%)\n", on, off, pct
    }
}'
