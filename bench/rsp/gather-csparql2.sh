#!/usr/bin/env bash
# [SONNET-4.6] (sq-rpdae) RSP4J/csparql2 gather — the FULL-SPARQL arm of the bounded
# count-matched-replay protocol (research/comparative-benchmarking-everything.md sec 5.2).
# GATHER-TIME ONLY: needs a JDK, a maven binary, and network for the first build; RSP4J is
# NOT a committed dependency and this script never runs in CI.
#
# WHAT IT ESTABLISHES (verdict, not a widened surface — see research/gap-rsp-2026-07.md):
# csparql2's R2ROperatorSPARQL can EXPRESS the aggregate scenarios YASPER's TP dialect
# cannot, but it does NOT widen the count-comparable surface. This script reproduces the
# three findings behind that verdict:
#   * the upstream SafeIterator deadlock (step 2 patches it EXPLICITLY — without the patch
#     the runner HANGS rather than failing),
#   * the multi-window scenarios emitting zero rows,
#   * the single-window aggregate counts, admitted only WITH a machine-attached protocol
#     caveat recording that the agreement is window-alignment-contingent.
#
# TOOLCHAIN (both constraints are real and were hit while evaluating sq-rpdae):
#   * BUILD with JDK 11: the dsms module pins lombok 1.18.20, which cannot run under
#     JDK 17 (LombokProcessor cannot access jdk.compiler's JavacProcessingEnvironment).
#   * The runner itself is plain Java 11 source, so one JDK 11 serves both steps.
#
# Usage: bench/rsp/gather-csparql2.sh [out-envelope-dir]
#   env: MVN=/path/to/mvn  JAVA_HOME=/path/to/jdk11  RSP4J_SRC=/path/to/checkout
#        CANONICAL=1 (quiet box only)
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
OUTDIR="${1:-$ROOT/bench/canonical-competitor-results/$(date -u +%Y-%m-%d)}"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"

# Pinned upstream: same commit as gather-rsp4j.sh, but note the csparql2 module carries
# its OWN version (2.0.0) — it is NOT the 1.0.1 the api/yasper/operatorapi modules use.
RSP4J_REPO="https://github.com/streamreasoning/rsp4j"
RSP4J_COMMIT="c46e0f674fb740b85543e73d8013196b18763abd"
CSPARQL2_VERSION="2.0.0"
RSP4J_SRC="${RSP4J_SRC:-/tmp/rsp4j-src}"
MVN="${MVN:-mvn}"

command -v "$MVN" >/dev/null || { echo "[csparql2-gather] ERROR: maven not found (set MVN=)" >&2; exit 1; }
JAVA="${JAVA_HOME:+$JAVA_HOME/bin/}java"
JAVAC="${JAVA_HOME:+$JAVA_HOME/bin/}javac"
command -v "$JAVA" >/dev/null || { echo "[csparql2-gather] ERROR: java not found (set JAVA_HOME to a JDK 11)" >&2; exit 1; }

# ---- 1. build csparql2 (+ deps) at the pinned commit ---------------------------------
if [ ! -d "$RSP4J_SRC/.git" ]; then
  git clone "$RSP4J_REPO" "$RSP4J_SRC"
fi
git -C "$RSP4J_SRC" fetch -q origin "$RSP4J_COMMIT" 2>/dev/null || true
git -C "$RSP4J_SRC" checkout -q "$RSP4J_COMMIT"
git -C "$RSP4J_SRC" checkout -q -- .   # drop any patch from a previous gather

# ---- 2. the DECLARED upstream patch --------------------------------------------------
# EsperGGWindowOperator.getContent() leaks the Esper statement read lock by never
# close()ing its SafeIterator; the next external-clock CurrentTimeEvent then needs the
# write lock on the same thread and parks forever. This is applied EXPLICITLY, is
# recorded in the envelope, and is the only modification made to upstream.
PATCH_TARGET="$RSP4J_SRC/csparql2/src/main/java/org/streamreasoning/rsp4j/csparql2/operators/EsperGGWindowOperator.java"
python3 - "$PATCH_TARGET" <<'PY'
import sys
p = sys.argv[1]
s = open(p, encoding="utf-8").read()
old = """            SafeIterator<EventBean> iterator = statement.safeIterator();
            JenaGraphContent events = new JenaGraphContent();
            events.setLast_timestamp_changed(now);
            while (iterator.hasNext()) {
                events.add(iterator.next());
            }
            return events;"""
new = """            SafeIterator<EventBean> iterator = statement.safeIterator();
            try {
                JenaGraphContent events = new JenaGraphContent();
                events.setLast_timestamp_changed(now);
                while (iterator.hasNext()) {
                    events.add(iterator.next());
                }
                return events;
            } finally {
                iterator.close();
            }"""
if old not in s:
    sys.exit("[csparql2-gather] ERROR: SafeIterator deadlock patch no longer applies — "
             "upstream changed; re-verify the deadlock before trusting any result.")
open(p, "w", encoding="utf-8").write(s.replace(old, new))
print("[csparql2-gather] applied the SafeIterator close() patch (declared, recorded)")
PY

(cd "$RSP4J_SRC" && "$MVN" -B -q -pl api,dsms,io,csparql2 -am -DskipTests install)

# ---- 3. compile the runner -----------------------------------------------------------
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
(cd "$RSP4J_SRC" && "$MVN" -B -q -pl csparql2 dependency:build-classpath -Dmdep.outputFile="$WORK/cp.txt")
CP="$(cat "$WORK/cp.txt")"
for m in api dsms io csparql2; do
  CP="$CP:$RSP4J_SRC/$m/target/classes"
done
"$JAVAC" -encoding UTF-8 -nowarn -cp "$CP" -d "$WORK" "$HERE/rsp4j/Csparql2ReplayRunner.java"

# ---- 4. drive csparql2 + run the count-match gate per scenario ------------------------
# ts-offset 1 + non-empty-content false is the closest the driver can get to the oracle's
# half-open window; see the runner header for why exact alignment is not reachable.
CAVEAT="PROTOCOL CAVEAT (sq-rpdae): csparql2's S2R is an Esper SLIDING win:time snapshotted \
at the external-clock advance that crosses the boundary — (T-range, T] — NOT the oracle's \
aligned [k*step, k*step+range). A count-match here therefore evidences per-window ROW-COUNT \
agreement on THIS replay's event placement, and does NOT evidence equivalent window \
semantics: the tumbling_groupby_join witness diverges (w0 = 1 vs oracle 2) precisely \
because a left-edge triple has expired by snapshot time. Do NOT promote these scenarios \
to the published comparable surface on this basis. See research/gap-rsp-2026-07.md."

mkdir -p "$OUTDIR"
CANON_ARGS=()
if [ "${CANONICAL:-0}" = "1" ]; then CANON_ARGS+=(--canonical); fi

status=0
for spec in \
  "tumbling_avg:single_window" \
  "sliding_sum:single_window" \
  "tumbling_groupby_join:single_window" \
  "srbench_join:srbench" \
  "srbench_groupby_state:srbench" ; do
  sc="${spec%%:*}"; replay="$HERE/replay/${spec##*:}.ts.tsv"
  "$JAVA" -cp "$WORK:$CP" Csparql2ReplayRunner \
    --replay "$replay" --scenario "$sc" \
    --ts-offset 1 --non-empty-content false \
    --version "csparql2 $CSPARQL2_VERSION (pinned $RSP4J_COMMIT, locally built, SafeIterator patch applied)" \
    > "$WORK/$sc.tsv"
  # A NOT-COUNT-MATCHED scenario is an EXPECTED outcome here, not a script failure: the
  # gate exits 3 and writes an envelope with zero timing rows, which is the finding.
  python3 "$HERE/rsp4j_compare.py" count-match \
    --scenario "$sc" \
    --competitor "$WORK/$sc.tsv" \
    --replay "$replay" \
    --engine rsp4j-csparql2 \
    --protocol-caveat "$CAVEAT" \
    --out "$OUTDIR/csparql2-count-match-$sc-$STAMP.json" "${CANON_ARGS[@]}" || status=$?
done

echo "[csparql2-gather] envelopes: $OUTDIR/csparql2-count-match-*-$STAMP.json" >&2
if [ "$status" -ne 0 ]; then
  echo "[csparql2-gather] NOTE: at least one scenario was NOT-COUNT-MATCHED (expected — see the verdict above)" >&2
fi
