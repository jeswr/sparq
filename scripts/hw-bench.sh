#!/usr/bin/env bash
# Per-platform hardware benchmark harness for sparq.
#
# Builds the richest build tier the HOST CPU supports (the same -Ctarget-cpu tiers
# .github/workflows/dist.yml ships), then runs ingest / query / serialisation / inference
# benchmarks and prints a per-platform throughput table. Run this on each target — Apple
# Silicon, Intel Mac, x86-64 (v2/v3/v4), Graviton/aarch64-linux, Windows (Git Bash) — to get
# the per-hardware numbers the build tiers + per-ISA prefetch need validating against. It is
# the measurement step that cannot be done on a single dev machine.
#
#   ./scripts/hw-bench.sh [scale_entities] [out.csv]
#
# Compares the host-native default build against the tier-tuned build (target-cpu), so the
# AVX2/AVX-512/Neoverse uplift over baseline is measured directly. The result line is appended
# to out.csv (default hw-bench-results.csv) so runs from several machines accumulate into one
# table.
set -euo pipefail
cd "$(dirname "$0")/.."

SCALE="${1:-500000}"            # entities; ~8x triples
OUT="${2:-hw-bench-results.csv}"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

arch="$(uname -m)"; os="$(uname -s)"
# Pick the richest -Ctarget-cpu the host advertises (mirrors sparq-launch.sh's ladder).
cpu=""
case "$os/$arch" in
  Darwin/arm64|Darwin/aarch64) cpu="" ;;                 # Apple silicon: native unlocks nothing
  Darwin/x86_64)               cpu="x86-64-v3" ;;        # Intel Mac (Haswell+)
  Linux/aarch64|Linux/arm64)   cpu="neoverse-n1" ;;      # Graviton/Ampere/RPi4+
  *x86_64*|*amd64*)
    f=" $(grep -m1 '^flags' /proc/cpuinfo 2>/dev/null | cut -d: -f2-) "
    has() { case "$f" in *" $1 "*) return 0;; *) return 1;; esac; }
    if has avx512f && has avx512bw && has avx512vl; then cpu="x86-64-v4"
    elif has avx2 && has bmi2 && has fma;            then cpu="x86-64-v3"
    elif has sse4_2;                                  then cpu="x86-64-v2"
    else cpu="x86-64"; fi ;;
esac
echo "host: $os/$arch  tier-cpu: ${cpu:-native}"

echo "== building (native default + tier ${cpu:-native}) =="
cargo build --release -q -p sparq-cli -p sparq-bench
cp target/release/sparq-cli "$TMP/sparq-native"
if [ -n "$cpu" ]; then
  RUSTFLAGS="-Ctarget-cpu=$cpu" cargo build --release -q -p sparq-cli
fi
cp target/release/sparq-cli "$TMP/sparq-tier"

echo "== generating $SCALE-entity dataset =="
./target/release/sparq-bench dump "$SCALE" "$TMP/data.nt" >/dev/null 2>&1
n=$(wc -l < "$TMP/data.nt")
# One query per dir (sparq-cli bench runs every .rq in the dir; one-per-dir keeps the timing
# attributable). scan = serialisation-heavy; join = a 2-pattern join.
mkdir -p "$TMP/scan" "$TMP/join"
echo 'SELECT ?s ?o WHERE { ?s <http://ex/follows> ?o }'                          > "$TMP/scan/q.rq"
echo 'SELECT ?a ?n WHERE { ?a <http://ex/follows> ?b . ?b <http://ex/name> ?n }' > "$TMP/join/q.rq"
# An RDFS inference workload (instance-heavy: many subjects under a depth-20 hierarchy).
{ echo '@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .'; echo '@prefix : <http://ex/> .'
  for k in $(seq 0 19); do echo ":c$k rdfs:subClassOf :c$((k+1)) ."; done
  for j in $(seq 1 "$SCALE"); do echo ":x$j a :c0 ."; done; } > "$TMP/inf.ttl"

# Best (max) load throughput (M/s), 3 runs. (Portable: no bash-4 features.)
loadmps() { b=0; m=0; for _ in 1 2 3; do
    b=$("$1" bench "$TMP/data.nt" ntriples "$TMP/scan" 1 count 2>&1 | grep -oE '[0-9.]+ M/s' | head -1 | grep -oE '[0-9.]+')
    awk "BEGIN{exit !(${b:-0}>$m)}" && m=$b; done; echo "$m"; }
# Best (min) time (us) of the single query in dir $2, mode $3, 3 runs.
qtime() { v=0; m=99999999; for _ in 1 2 3; do
    v=$("$1" bench "$TMP/data.nt" ntriples "$2" 3 "$3" 2>/dev/null | awk -F'\t' 'NR==1{print $NF}')
    awk "BEGIN{exit !(${v:-99999999}<$m)}" && m=$v; done; echo "$m"; }
inftime() { "$1" reason "$TMP/inf.ttl" turtle rdfs 2>&1 | grep -oE 'in [0-9.]+s' | grep -oE '[0-9.]+'; }

echo "== benchmarking ($n triples) =="
ln=$(loadmps "$TMP/sparq-native");                      lt=$(loadmps "$TMP/sparq-tier")
jn=$(qtime "$TMP/sparq-native" "$TMP/scan" json);       jt=$(qtime "$TMP/sparq-tier" "$TMP/scan" json)
mn=$(qtime "$TMP/sparq-native" "$TMP/join" materialize); mt=$(qtime "$TMP/sparq-tier" "$TMP/join" materialize)
fn=$(inftime "$TMP/sparq-native");                      ft=$(inftime "$TMP/sparq-tier")

hdr="host,arch,tier_cpu,triples,load_native_Mps,load_tier_Mps,json_native_us,json_tier_us,mat_native_us,mat_tier_us,infer_native_s,infer_tier_s"
row="$(uname -n),$arch,${cpu:-native},$n,$ln,$lt,$jn,$jt,$mn,$mt,$fn,$ft"
[ -f "$OUT" ] || echo "$hdr" > "$OUT"
echo "$row" >> "$OUT"
echo
echo "$hdr"
echo "$row"
echo
echo "appended to $OUT — collect this from each platform (Apple Silicon, Intel Mac, x86-64"
echo "v2/v3/v4, Graviton c7g, Windows) to fill the per-hardware throughput table. The tier vs"
echo "native delta IS the per-ISA (AVX2/AVX-512/Neoverse) uplift; where it is large, that hot"
echo "loop is the candidate for a hand-written kernel on that ISA."
