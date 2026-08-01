#!/usr/bin/env bash
# [FABLE-5] (sq-hmd7l.20) RSP4J/YASPER gather — the REAL-engine leg of the bounded
# count-matched-replay protocol (research/comparative-benchmarking-everything.md
# sec 5.2). GATHER-TIME ONLY: needs a JDK 16+ (Rsp4jReplayRunner.java uses `record`), a
# maven binary, and network for the first build; RSP4J is NOT a committed dependency and
# this script never runs in CI.
#
# Pipeline:
#   1. clone + build RSP4J at the PINNED commit (jars land in ~/.m2, Apache-2.0),
#   2. compile bench/rsp/rsp4j/Rsp4jReplayRunner.java against them,
#   3. drive YASPER with the pinned timestamped replay (event-time, no wall clock),
#   4. run the count-match gate (rsp4j_compare.py) — the envelope gets timing rows
#      ONLY if every non-excluded window count-matches the oracle; every row carries
#      the machine-attached time-model caveat.
#
# SCALED=1 adds the MATCHED-WORKLOAD THROUGHPUT leg (sq-3f5ay). The default replay is the
# pinned 19-event oracle export — deterministic, but far too small for a sustained-rate
# claim, which is why the throughput axis was NOT-MEASURED. With SCALED=1 the script
# instead regenerates the pinned ~10^5-event scaled replay, drives BOTH engines from that
# one file (sparq via the `replay_runner` example, YASPER via the same Java runner), and
# passes sparq's per-window counts as the oracle side. The INVARIANT is unchanged: the
# count-match gate runs first and admits sustained triples/s for EITHER engine only if
# every window agrees.
#
# Usage: bench/rsp/gather-rsp4j.sh [out-envelope.json]
#        SCALED=1 bench/rsp/gather-rsp4j.sh [out-envelope.json]
#   env: MVN=/path/to/mvn  RSP4J_SRC=/path/to/checkout  CANONICAL=1 (quiet box only)
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
SCALED="${SCALED:-0}"
LEG="rsp4j-yasper-count-match"
if [ "$SCALED" = "1" ]; then LEG="rsp4j-yasper-scaled-throughput"; fi
OUT="${1:-$ROOT/bench/canonical-competitor-results/$(date -u +%Y-%m-%d)/$LEG-$(date -u +%Y%m%dT%H%M%SZ).json}"

# Pinned upstream: the repo moved to the streamreasoning org (the registry's original
# riccardotommasini URL 404s — see bead note); jar version 1.0.1 on master.
RSP4J_REPO="https://github.com/streamreasoning/rsp4j"
RSP4J_COMMIT="c46e0f674fb740b85543e73d8013196b18763abd"
RSP4J_VERSION="1.0.1"
RSP4J_SRC="${RSP4J_SRC:-/tmp/rsp4j-src}"
MVN="${MVN:-mvn}"

command -v java >/dev/null || { echo "[rsp4j-gather] ERROR: java not found (gather needs a JVM)" >&2; exit 1; }
command -v "$MVN" >/dev/null || { echo "[rsp4j-gather] ERROR: maven not found (set MVN=; e.g. the apache-maven-3.9.x binary tarball)" >&2; exit 1; }

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

# ---- 0. the replay + (scaled only) the sparq-side oracle leg -------------------------
REPLAY="$HERE/replay/srbench.ts.tsv"
ORACLE_ARGS=()
if [ "$SCALED" = "1" ]; then
  command -v cargo >/dev/null || { echo "[rsp4j-gather] ERROR: cargo not found (the scaled leg runs sparq's replay_runner)" >&2; exit 1; }
  REPLAY="$HERE/replay/scaled_srbench.ts.tsv"   # generated, gitignored
  # Regenerate + sha256-verify against the committed pin: both engines must be driven
  # from the SAME bytes, and a drifted generator must fail loudly rather than quietly
  # comparing two different workloads.
  python3 "$HERE/rsp4j_compare.py" gen-replay --out "$REPLAY"
  cargo build --release -q -p sparq-rsp --example replay_runner
  "$ROOT/target/release/examples/replay_runner" \
    --replay "$REPLAY" --scenario srbench_join > "$WORK/sparq.tsv"
  ORACLE_ARGS=(--oracle-runner "$WORK/sparq.tsv")
fi

# ---- 1. build RSP4J at the pinned commit -------------------------------------------
if [ ! -d "$RSP4J_SRC/.git" ]; then
  git clone "$RSP4J_REPO" "$RSP4J_SRC"
fi
git -C "$RSP4J_SRC" fetch -q origin "$RSP4J_COMMIT" 2>/dev/null || true
git -C "$RSP4J_SRC" checkout -q "$RSP4J_COMMIT"
JARS="$HOME/.m2/repository/org/streamreasoning/rsp4j"
if [ ! -f "$JARS/operatorapi/$RSP4J_VERSION/operatorapi-$RSP4J_VERSION.jar" ]; then
  (cd "$RSP4J_SRC" && "$MVN" -q -pl api,yasper,operatorapi -am -DskipTests install)
fi

# ---- 2. compile the runner -----------------------------------------------------------
(cd "$RSP4J_SRC" && "$MVN" -q -pl operatorapi dependency:build-classpath -Dmdep.outputFile="$WORK/cp.txt")
CP="$(cat "$WORK/cp.txt")"
for m in api yasper operatorapi io; do
  CP="$CP:$JARS/$m/$RSP4J_VERSION/$m-$RSP4J_VERSION.jar"
done
# -encoding UTF-8 is REQUIRED, not cosmetic: the runner's comments contain non-ASCII, so
# javac fails outright (not warns) on a box whose default charset is US-ASCII (sq-rpdae).
javac -encoding UTF-8 -cp "$CP" -d "$WORK" "$HERE/rsp4j/Rsp4jReplayRunner.java"

# ---- 3. drive YASPER with the SAME replay --------------------------------------------
java -cp "$WORK:$CP" Rsp4jReplayRunner \
  --replay "$REPLAY" \
  --scenario srbench_join \
  --version "$RSP4J_VERSION (pinned $RSP4J_COMMIT, locally built)" \
  > "$WORK/rsp4j.tsv"

# ---- 4. the count-match gate (timing admitted only on agreement) ---------------------
mkdir -p "$(dirname "$OUT")"
CANON_ARGS=()
if [ "${CANONICAL:-0}" = "1" ]; then CANON_ARGS+=(--canonical); fi
python3 "$HERE/rsp4j_compare.py" count-match \
  --scenario srbench_join \
  --competitor "$WORK/rsp4j.tsv" \
  --replay "$REPLAY" \
  --out "$OUT" "${CANON_ARGS[@]}" "${ORACLE_ARGS[@]}"
echo "[rsp4j-gather] envelope: $OUT" >&2
