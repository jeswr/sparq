#!/usr/bin/env bash
# [FABLE-5] sq-hmd7l.3 — same-box GeoSPARQL comparison harness: sparq-geo vs
# jena-fuseki-geosparql (the registered like-for-like compliance peer: the only
# triplestore with full GML+WKT, per bench/competitors.json `geosparql-jena`).
# Built on the scripts/bench/shacl-same-box.sh template; emits one
# canonical-competitor-results-shaped ENVELOPE JSON so a future canonical gather
# is ingestible by scripts/bench/ingest-canonical-competitors.mjs unchanged.
#
# GeoSPARQL had NO competitor baseline (registered gather-only, never run); this
# script is the durable, reusable gather recipe. A run on the shared work box is
# NON-canonical (canonical:false in the envelope, always); the canonical run is
# the quiet-box wave (design record research/comparative-benchmarking-everything.md
# §4; execution bead sq-hmd7l.26). CANONICAL=1 only there.
#
# WORKLOADS (shared corpus: the FIXED ~100k-point CRS84 corpus, bench/geo/gen.sh,
# seed 20260615 — the same substrate the per-commit bench/geo gate asserts):
#   within10km / within50km   great-circle radius counts around POINT(0 51)
#   nearest_k10 / nearest_k100 k-nearest (rendered for SPARQL peers as
#                              ORDER BY geof:distance ... LIMIT k — the
#                              standard-visible form; see queries-jena/*.rq)
#   geof_within                geof:sfWithin(point, 1°x1° box) filter count
# geo_compliance_pass (the OGC fixture ratchet) is sparq-only and carried in
# sparq_tsv but is NOT a competitor row (replaying the fixture set against Jena
# is out of scope here; Jena is the like-for-like COVERAGE bar per the registry).
#
# METHODOLOGY / INVARIANT (counts-not-coordinates, per the bench/geo gate design):
#   * sparq is timed IN-PROCESS via bench/geo/run.sh (examples/bench_geo: load +
#     index once, best-of-N per workload), which HARD-asserts every count vs
#     bench/geo/expected.tsv — the sparq-side oracle.
#   * jena-geosparql is timed over HTTP: the corpus is loaded into an in-memory
#     jena-fuseki-geosparql dataset, then each pinned query in
#     bench/geo/queries-jena/*.rq is POSTed via
#     scripts/bench-adapters/http_sparql_adapter.py (best-of-N; query_us includes
#     HTTP framing + SPARQL-JSON parse — a recorded mode ASYMMETRY vs sparq's
#     in-process index surface, never adjusted away).
#   * NO TIMING WITHOUT RESULT-SET-SIZE AGREEMENT per query: a competitor timing
#     enters the envelope's timing table ONLY where its count equals sparq's
#     expected.tsv-gated count. A disagreement is itself a recorded RESULT
#     (both counts kept, the timing withheld, nothing adjusted). Both engines'
#     within* metric is spherical great-circle on the same mean sphere (sparq:
#     haversine; jena: spatialF:nearby — the standard geof:distance+uom:metre
#     form is non-executable on jena 5.4.0, root-caused in sq-a8anf /
#     research/gap-geo-2026-07.md §6d), so only float-boundary edge points may
#     legitimately differ.
#   * per-workload wall-clock cap (TIMEOUT_S, covering the whole best-of-N
#     series); a timeout/error degrades to an honest ERROR row, never a number.
#   * a competitor that cannot run (no java, no jar, download or start-up
#     failure) is GRACEFULLY SKIPPED: the envelope records the reason and the
#     script still exits 0 (a skip is a missing column + a re-run action, never
#     a sparq win — design record §4 point 6).
#
# Geographica: the reviewer-recognized real-world suite is deliberately NOT
# wired here — deferred to bead sq-hmd7l.29 (recorded in the envelope).
#
# USAGE
#   scripts/bench/geo-same-box.sh                 # both engines, full iters
#   ONLY=sparq GEO_SMOKE=1 scripts/bench/geo-same-box.sh   # the acceptance smoke
#
# TUNABLES (env; all have safe defaults):
#   ITERS           best-of-N per workload      (default 3; 1 under GEO_SMOKE=1)
#   TIMEOUT_S       per-workload cap (whole best-of-N series), s   (default 300)
#   ONLY            engine subset of "sparq jena-geosparql"        (default both)
#   OUT_DIR         envelope output dir (default /tmp/geo-same-box-results; a
#                   canonical run points this at
#                   bench/canonical-competitor-results/<date>/)
#   CANONICAL       1 = dedicated quiet-box run (default 0: NON-canonical)
#   GEO_SMOKE       1 = smoke mode (iters=1, envelope flagged smoke:true)
#   FUSEKI_GEO_VERSION  jena-fuseki-geosparql version       (default 5.4.0)
#   FUSEKI_GEO_JAR  path to the executable uber-jar; auto-downloaded from Maven
#                   Central to /tmp/jena-geosparql/ if absent and GEO_FETCH_JENA=1
#   GEO_FETCH_JENA  0 = never download (absent jar -> graceful skip; default 1)
#   FUSEKI_PORT     jena-fuseki-geosparql port              (default 3039)
#   FUSEKI_READY_S  server load+spatial-index readiness cap (default 300)
#   BENCH_GEO       the sparq bench_geo binary (default: build it)
#
# SCRATCH: the jar lives under /tmp/jena-geosparql (gather-only dep, NOT
# committed — engines stay out of git per AGENTS.md): rm -rf /tmp/jena-geosparql
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
cd "$ROOT"

GEO_SMOKE="${GEO_SMOKE:-0}"
if [ "$GEO_SMOKE" = 1 ]; then ITERS="${ITERS:-1}"; else ITERS="${ITERS:-3}"; fi
TIMEOUT_S="${TIMEOUT_S:-300}"
ONLY="${ONLY:-sparq jena-geosparql}"
OUT_DIR="${OUT_DIR:-/tmp/geo-same-box-results}"
CANONICAL="${CANONICAL:-0}"
FUSEKI_GEO_VERSION="${FUSEKI_GEO_VERSION:-5.4.0}"
FUSEKI_GEO_JAR="${FUSEKI_GEO_JAR:-/tmp/jena-geosparql/jena-fuseki-geosparql-$FUSEKI_GEO_VERSION.jar}"
GEO_FETCH_JENA="${GEO_FETCH_JENA:-1}"
FUSEKI_PORT="${FUSEKI_PORT:-3039}"
FUSEKI_READY_S="${FUSEKI_READY_S:-300}"
BENCH_GEO="${BENCH_GEO:-$ROOT/target/release/examples/bench_geo}"
ADAPTER="$ROOT/scripts/bench-adapters/http_sparql_adapter.py"
QUERIES="$ROOT/bench/geo/queries-jena"
# The 5 query-replayable workloads (geo_compliance_pass is sparq-only, see header).
WORKLOADS="within10km within50km nearest_k10 nearest_k100 geof_within"

log() { printf '[geo-same-box] %s\n' "$*" >&2; }
want() { [[ " $ONLY " == *" $1 "* ]]; }

mkdir -p "$OUT_DIR"
TMP="$(mktemp -d /tmp/geo-same-box.XXXXXX)"
FUSEKI_PID=""
# shellcheck disable=SC2317 # invoked via trap
cleanup() {
  [ -n "$FUSEKI_PID" ] && kill "$FUSEKI_PID" 2>/dev/null && wait "$FUSEKI_PID" 2>/dev/null
  rm -rf "$TMP"
}
trap cleanup EXIT

# ---- 0. sparq binary + the shared fixed corpus --------------------------------
if [ ! -x "$BENCH_GEO" ]; then
  log "building sparq bench_geo (cargo -p sparq-geo --example bench_geo)"
  cargo build --release -q -p sparq-geo --example bench_geo
  BENCH_GEO="$ROOT/target/release/examples/bench_geo"
fi
# The FIXED ~100k-point corpus (cached, deterministic). expected.tsv is pinned to
# this scale, so the corpus size is intentionally NOT tunable here.
CORPUS="$(BENCH_GEO="$BENCH_GEO" "$ROOT/bench/geo/gen.sh" 100000)"
NPOINTS="$(wc -l < "$CORPUS")"
log "corpus=$CORPUS (~$NPOINTS triples)"

GIT_COMMIT="$(git -C "$ROOT" rev-parse --short HEAD 2>/dev/null || echo unknown)"

# ---- 1. sparq: the self-asserting in-process leg (counts gated vs expected.tsv)
SPARQ_STATUS="not-run"
if want sparq; then
  log "sparq: bench/geo/run.sh x$ITERS (cap $((TIMEOUT_S * 6))s; counts hard-asserted vs expected.tsv)"
  if ITERS="$ITERS" BENCH_GEO="$BENCH_GEO" timeout "$((TIMEOUT_S * 6))" \
      bash "$ROOT/bench/geo/run.sh" > "$TMP/sparq.tsv" 2> "$TMP/sparq.err"; then
    SPARQ_STATUS="ok"
  else
    SPARQ_STATUS="failed"
    log "sparq FAILED/timeout or count drift (see $TMP/sparq.err):"
    tail -5 "$TMP/sparq.err" >&2 || true
  fi
fi

# ---- 2. jena-geosparql: HTTP leg (graceful skip when the engine is absent) -----
JENA_STATUS="not-run"
JENA_VER=""
JENA_JAR_SHA=""
JENA_READY_S="n/a"
ENDPOINT="http://127.0.0.1:$FUSEKI_PORT/ds"
if want jena-geosparql; then
  JENA_STATUS="pending"
  if ! command -v java >/dev/null 2>&1; then
    JENA_STATUS="skipped: java not on PATH"
  elif [ ! -f "$FUSEKI_GEO_JAR" ]; then
    if [ "$GEO_FETCH_JENA" = 1 ]; then
      log "downloading jena-fuseki-geosparql $FUSEKI_GEO_VERSION to $(dirname "$FUSEKI_GEO_JAR") (gather-only dep)"
      mkdir -p "$(dirname "$FUSEKI_GEO_JAR")"
      MAVEN_URL="https://repo1.maven.org/maven2/org/apache/jena/jena-fuseki-geosparql/$FUSEKI_GEO_VERSION/jena-fuseki-geosparql-$FUSEKI_GEO_VERSION.jar"
      if ! curl -fsSL -o "$FUSEKI_GEO_JAR.tmp" "$MAVEN_URL"; then
        rm -f "$FUSEKI_GEO_JAR.tmp"
        JENA_STATUS="skipped: jar download failed ($MAVEN_URL)"
      else
        mv "$FUSEKI_GEO_JAR.tmp" "$FUSEKI_GEO_JAR"
      fi
    else
      JENA_STATUS="skipped: jar absent at $FUSEKI_GEO_JAR and GEO_FETCH_JENA=0"
    fi
  fi

  if [ "$JENA_STATUS" = "pending" ]; then
    JENA_VER="jena-fuseki-geosparql-$FUSEKI_GEO_VERSION ($(java -version 2>&1 | head -1))"
    JENA_JAR_SHA="$(sha256sum "$FUSEKI_GEO_JAR" | cut -d' ' -f1)"
    log "starting $ENDPOINT (in-memory dataset, corpus loaded as N-Triples; readiness cap ${FUSEKI_READY_S}s)"
    T_START="$(date +%s)"
    # Format is inferred from the .nt extension; -rf = read file, -p = port.
    java -jar "$FUSEKI_GEO_JAR" -rf "$CORPUS" -p "$FUSEKI_PORT" \
      > "$TMP/fuseki.log" 2>&1 &
    FUSEKI_PID=$!
    READY=0
    DEADLINE=$((T_START + FUSEKI_READY_S))
    while [ "$(date +%s)" -lt "$DEADLINE" ]; do
      if ! kill -0 "$FUSEKI_PID" 2>/dev/null; then break; fi
      if python3 "$ADAPTER" --endpoint "$ENDPOINT" --query 'ASK {}' \
          --engine ready >/dev/null 2>&1; then
        READY=1; break
      fi
      sleep 2
    done
    if [ "$READY" != 1 ]; then
      JENA_STATUS="skipped: server not ready in ${FUSEKI_READY_S}s (see fuseki.log tail below)"
      tail -5 "$TMP/fuseki.log" >&2 || true
    else
      JENA_READY_S="$(( $(date +%s) - T_START ))"
      log "jena-geosparql ready in ${JENA_READY_S}s (load + spatial index; advisory)"
      : > "$TMP/jena-geosparql.tsv"
      for wl in $WORKLOADS; do
        log "jena-geosparql: $wl x$ITERS (cap ${TIMEOUT_S}s)"
        if timeout "$TIMEOUT_S" python3 "$ADAPTER" --endpoint "$ENDPOINT" \
            --query-file "$QUERIES/$wl.rq" --engine jena-geosparql \
            --iters "$ITERS" --json > "$TMP/$wl.raw" 2> "$TMP/$wl.json"; then
          # --json writes the result JSON to stderr; a COUNT-wrapped query's
          # comparable size is count_value (the scalar), else len(bindings).
          if PARSED="$(python3 - "$TMP/$wl.json" <<'PYEOF'
import json, sys
line = open(sys.argv[1]).read().strip().splitlines()[-1]
d = json.loads(line)
c = d["count_value"] if d.get("count_value") is not None else d["count"]
print("%s\t%s" % (c, d["query_us"]))
PYEOF
          )"; then
            printf '%s\t%s\n' "$wl" "$PARSED" >> "$TMP/jena-geosparql.tsv"
          else
            printf '%s\tERROR\tsidecar-parse-error\n' "$wl" >> "$TMP/jena-geosparql.tsv"
          fi
        else
          log "jena-geosparql: $wl FAILED/timeout"
          printf '%s\tERROR\ttimeout-or-error\n' "$wl" >> "$TMP/jena-geosparql.tsv"
        fi
      done
      JENA_STATUS="ok"
    fi
  fi
  [ "$JENA_STATUS" = "ok" ] || log "jena-geosparql: $JENA_STATUS"
fi

# ---- 3. assemble the envelope (canonical-competitor-results JSON shape) --------
TS="$(date -u +%Y%m%dT%H%M%SZ)"
OUT="$OUT_DIR/geo-points100k-${TS}.json"
CANONICAL="$CANONICAL" GEO_SMOKE="$GEO_SMOKE" GIT_COMMIT="$GIT_COMMIT" \
CORPUS="$CORPUS" NPOINTS="$NPOINTS" ITERS="$ITERS" TIMEOUT_S="$TIMEOUT_S" \
ONLY="$ONLY" WORKLOADS="$WORKLOADS" TMP="$TMP" OUT="$OUT" \
SPARQ_STATUS="$SPARQ_STATUS" JENA_STATUS="$JENA_STATUS" JENA_VER="$JENA_VER" \
JENA_JAR_SHA="$JENA_JAR_SHA" JENA_READY_S="$JENA_READY_S" \
python3 - <<'PYEOF'
import json, os, platform

tmp = os.environ["TMP"]
only = os.environ["ONLY"].split()
workloads = os.environ["WORKLOADS"].split()
canonical = os.environ["CANONICAL"] == "1"
smoke = os.environ["GEO_SMOKE"] == "1"


def read_tsv(engine):
    path = os.path.join(tmp, "%s.tsv" % engine)
    rows = {}
    if os.path.exists(path):
        for line in open(path):
            f = line.rstrip("\n").split("\t")
            if len(f) >= 3:
                rows[f[0]] = {"count": f[1], "us": f[2]}
    return rows


engines_meta = {}
if "sparq" in only:
    engines_meta["sparq"] = {
        "version": os.environ["GIT_COMMIT"],
        "mode": (
            "in-process (bench/geo/run.sh -> examples/bench_geo: load + index "
            "once, best-of-N per workload; every count HARD-asserted vs "
            "bench/geo/expected.tsv)"
        ),
    }
if "jena-geosparql" in only:
    engines_meta["jena-geosparql"] = {
        "version": os.environ.get("JENA_VER", ""),
        "jar_sha256": os.environ.get("JENA_JAR_SHA", ""),
        "mode": (
            "HTTP (jena-fuseki-geosparql in-memory dataset over the SAME corpus; "
            "scripts/bench-adapters/http_sparql_adapter.py POST -> SPARQL-JSON, "
            "best-of-N; query_us includes HTTP framing + result parse — a "
            "recorded mode ASYMMETRY vs sparq's in-process index surface)"
        ),
        "load_ready_s_advisory": os.environ.get("JENA_READY_S", "n/a"),
    }

data = {e: read_tsv(e) for e in engines_meta}
statuses = {}
if "sparq" in engines_meta:
    statuses["sparq"] = os.environ["SPARQ_STATUS"]
if "jena-geosparql" in engines_meta:
    statuses["jena-geosparql"] = os.environ["JENA_STATUS"]

note_canonical = (
    "CANONICAL: dedicated quiet box, one engine active at a time on the SAME "
    "fixed CRS84 corpus + pinned query files; counts cross-checked before any "
    "timing is trusted."
    if canonical
    else "NON-canonical FIRST READ: shared work box (not a dedicated quiet "
    "instance). Timings are directional only — do NOT bake into docs or "
    "dashboards. The harness (scripts/bench/geo-same-box.sh) is the durable "
    "deliverable; the canonical run is the quiet-box wave (sq-hmd7l.26)."
)

# INVARIANT: no timing without result-set-size agreement per query.
sparq_rows = data.get("sparq", {})
cross = {}
timings = {}
for w in workloads:
    s = sparq_rows.get(w, {}).get("count", "n/a")
    j = data.get("jena-geosparql", {}).get(w, {}).get("count", "n/a")
    agree = s == j and s not in ("n/a", "ERROR")
    cross[w] = {"sparq": s, "jena-geosparql": j, "agree": agree}
    row = {}
    if "sparq" in engines_meta:
        row["sparq_us"] = sparq_rows.get(w, {}).get("us", "n/a")
    if "jena-geosparql" in engines_meta:
        ju = data["jena-geosparql"].get(w, {}).get("us", "n/a")
        if agree:
            row["jena_geosparql_us"] = ju
        elif j in ("n/a", "ERROR"):
            row["jena_geosparql_us"] = j
        else:
            # Disagreement IS the result: both counts stay recorded above; the
            # timing is WITHHELD (never adjusted, never silently dropped).
            row["jena_geosparql_us"] = (
                "WITHHELD(count-disagree: sparq=%s jena=%s)" % (s, j)
            )
    timings[w] = row

envelope = {
    "gather": "geo-same-box-comparison",
    "wave": "geo-competitor-baseline (sq-hmd7l.3)",
    "canonical": canonical,
    "canonical_note": note_canonical,
    "smoke": smoke,
    "git_commit": os.environ["GIT_COMMIT"],
    "suite": "geo",
    "scale": "fixed CRS84 point corpus, %s triples (%s)"
    % (os.environ["NPOINTS"], os.environ["CORPUS"]),
    "iters": int(os.environ["ITERS"]),
    "timeout_s_per_workload": int(os.environ["TIMEOUT_S"]),
    "queries": "bench/geo/queries-jena/<workload>.rq (pinned per-engine renderings)",
    "engines": engines_meta,
    "statuses": statuses,
    "count_crosscheck": cross,
    "count_crosscheck_note": (
        "COUNTS-NOT-COORDINATES (bench/geo gate design): only result-set SIZES "
        "are compared — float geometry is not bit-stable. sparq counts are "
        "additionally hard-gated vs bench/geo/expected.tsv by run.sh. A "
        "disagreement is a recorded RESULT; the competitor timing for that "
        "workload is withheld, never adjusted. Metric note (sq-a8anf): both "
        "engines' within* is spherical great-circle on the same mean sphere "
        "(sparq haversine; jena spatialF:nearby — jena 5.4.0 geof:distance is "
        "planar SRS-native-unit Euclidean and its degree->metre conversion "
        "throws, so the standard geof:distance+uom:metre rendering is "
        "non-executable there; research/gap-geo-2026-07.md §6d), so only "
        "float-boundary edge points may legitimately differ."
    ),
    "timings": timings,
    "timings_note": (
        "per-workload best-of-N microseconds. INVARIANT: a jena timing appears "
        "ONLY where count_crosscheck agrees. MODE ASYMMETRY: sparq is in-process "
        "(index surface), jena is a full HTTP SPARQL round-trip — recorded, not "
        "adjusted; sparq-geo is not wired into sparq-server, so a symmetric "
        "HTTP-vs-HTTP row is not currently possible."
    ),
    "sparq_only_rows_note": (
        "geo_compliance_pass (OGC fixture ratchet) rides in sparq_tsv but is "
        "NOT a competitor row: replaying the fixture set against Jena is out of "
        "scope here; Jena is the like-for-like GML+WKT COVERAGE bar per "
        "bench/competitors.json."
    ),
    "geographica": (
        "TODO (deferred, recorded): wire a Geographica real-world subset as a "
        "second workload family — bead sq-hmd7l.29; not in scope of sq-hmd7l.3."
    ),
    "env": {
        "host": platform.node(),
        "machine": platform.machine(),
        "os": platform.platform(),
    },
}
for e in engines_meta:
    envelope["%s_tsv" % e.replace("-", "_")] = "\n".join(
        "%s\t%s\t%s" % (w, r["count"], r["us"]) for w, r in data[e].items()
    )

with open(os.environ["OUT"], "w") as fh:
    json.dump(envelope, fh, indent=2)
    fh.write("\n")
print(os.environ["OUT"])
PYEOF
log "envelope: $OUT"

log "done. Gather-only scratch: /tmp/jena-geosparql (delete when finished)."
