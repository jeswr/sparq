#!/usr/bin/env bash
# [OPUS-4.8] sq-vw3ax.12.1 — dedicated same-box Apache Jena Fuseki load->serve->query->teardown
# recipe. Authored by Opus 4.8 (Fable unavailable; flag for re-review when Fable returns).
#
# WHY THIS SCRIPT EXISTS
#   Fuseki (competitors.json id `fuseki`, kind `http-sparql`) is the Tier-1 SPARQL-1.1
#   reference SERVER + a correctness oracle. Like QLever it is NOT a file-in/answer-out CLI:
#   to benchmark it you must (1) BULK-LOAD the corpus into a TDB2 store (tdb2.tdbloader),
#   then (2) start a long-lived fuseki-server HTTP endpoint, then (3) query it over HTTP via
#   the SHARED scripts/bench-adapters/http_sparql_adapter.py (the same POST->SPARQL-JSON->count
#   unit the QLever + Virtuoso paths use), then (4) STOP the server and delete the store.
#
#   This is the self-contained, BOUNDED, ORPHAN-SAFE recipe (the Fuseki sibling of
#   scripts/qlever-same-box.sh + scripts/virtuoso-same-box.sh). Every process it starts and
#   every byte of scratch it writes is removed by an EXIT trap — on success, failure, timeout,
#   or SIGINT. Every wait is hard-bounded; there is no unbounded poll anywhere.
#
# TWO BACKENDS (same Apache-2.0 bits -> identical results; pick per box):
#   * jena  : the apache-jena / apache-jena-fuseki tarballs (pure Java, NO Docker/root).
#             Set FUSEKI_SERVER=<.../fuseki-server> and TDBLOADER=<.../tdb2.tdbloader>
#             (or put them on PATH). This is the exact competitors.json run_recipe.
#   * docker: an official/community Fuseki image (default stain/jena-fuseki) — bulk-load with
#             the tdb2.tdbloader the image bundles, then serve. Needs a running Docker daemon.
#   Default: `docker` when a Docker daemon is reachable, else `jena`. Override with
#   FUSEKI_BACKEND=jena|docker.
#
# USAGE
#   scripts/fuseki-same-box.sh <corpus-file> <ttl|nt> <queries-dir> <iters> <out-tsv>
#     <corpus-file>  dataset (Turtle .ttl or N-Triples .nt)
#     <format>       ttl | nt
#     <queries-dir>  dir of *.rq SPARQL queries (e.g. bench/sp2b/queries)
#     <iters>        min-of-K per query
#     <out-tsv>      where to write the result TSV (<name>\t<rows>\t<best_us>)
#
#   Exit status: 0 if the store loaded, the server became ready, and >=1 query produced a
#   non-ERROR row; non-zero otherwise. The caller treats a non-zero exit as "fuseki
#   skipped/failed" and keeps the dashboard cell honest-n/a — it NEVER hangs the gather.
#
# TUNABLES (env; all have safe defaults):
#   FUSEKI_BACKEND        jena | docker                (default: docker if daemon up, else jena)
#   FUSEKI_SERVER         path to fuseki-server        (jena backend; else PATH)
#   TDBLOADER             path to tdb2.tdbloader       (jena backend; else PATH)
#   FUSEKI_IMAGE          Docker image (docker backend)(default docker.io/stain/jena-fuseki)
#   FUSEKI_PORT           HTTP port                    (default 3030)
#   FUSEKI_DS             dataset path segment         (default ds)
#   FUSEKI_LOAD_TIMEOUT   bulk-load hard cap, s        (default 900)
#   FUSEKI_PULL_TIMEOUT   image-pull hard cap, s       (default 600, docker backend)
#   FUSEKI_READY_TIMEOUT  server-readiness cap, s      (default 120)
#   FUSEKI_QUERY_TIMEOUT  per-HTTP-query cap, s        (default 60)
#   FUSEKI_MIN_FREE_GB    abort load if free disk <    (default 10)
#   FUSEKI_NAME           docker container name        (default fuseki-srv-$$)
#   JAVA_HOME             JDK home for the jena backend (auto-detected from `java` if unset)
#
# OUTPUT TSV FORMAT (matches the harness's <name>\t<rows>\t<best_us>, like sparq/oxigraph):
#   q01<TAB>1<TAB>4567.8        — rows + best wall micros over <iters> runs
#   q07<TAB>ERROR<TAB>fuseki    — query failed (server error / timeout)
set -euo pipefail

CORPUS="${1:?usage: fuseki-same-box.sh <corpus> <ttl|nt> <queries-dir> <iters> <out-tsv>}"
FORMAT="${2:?missing format (ttl|nt)}"
QUERIES_DIR="${3:?missing queries dir}"
ITERS="${4:?missing iters}"
OUT_TSV="${5:?missing out-tsv}"

FUSEKI_PORT="${FUSEKI_PORT:-3030}"
FUSEKI_DS="${FUSEKI_DS:-ds}"
FUSEKI_IMAGE="${FUSEKI_IMAGE:-docker.io/stain/jena-fuseki}"
FUSEKI_LOAD_TIMEOUT="${FUSEKI_LOAD_TIMEOUT:-900}"
FUSEKI_PULL_TIMEOUT="${FUSEKI_PULL_TIMEOUT:-600}"
FUSEKI_READY_TIMEOUT="${FUSEKI_READY_TIMEOUT:-120}"
FUSEKI_QUERY_TIMEOUT="${FUSEKI_QUERY_TIMEOUT:-60}"
FUSEKI_MIN_FREE_GB="${FUSEKI_MIN_FREE_GB:-10}"
FUSEKI_NAME="${FUSEKI_NAME:-fuseki-srv-$$}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ADAPTER="$SCRIPT_DIR/bench-adapters/http_sparql_adapter.py"

log()  { printf '[fuseki] %s\n' "$*" >&2; }
die()  { printf '[fuseki] ERROR: %s\n' "$*" >&2; exit 1; }
have() { command -v "$1" >/dev/null 2>&1; }

case "$FORMAT" in ttl|nt) ;; *) die "format must be 'ttl' or 'nt' (got '$FORMAT')" ;; esac
case "$ITERS" in ''|*[!0-9]*) die "iters must be a positive integer (got '$ITERS')" ;; esac
[ "$ITERS" -ge 1 ] || die "iters must be >= 1"
[ -f "$CORPUS" ] || die "corpus file not found: $CORPUS"
[ -d "$QUERIES_DIR" ] || die "queries dir not found: $QUERIES_DIR"
have python3 || die "python3 required (http_sparql_adapter client)"
[ -f "$ADAPTER" ] || die "shared adapter not found: $ADAPTER"

# ---- pick a backend --------------------------------------------------------------------
docker_up() { have docker && docker info >/dev/null 2>&1; }
BACKEND="${FUSEKI_BACKEND:-}"
if [ -z "$BACKEND" ]; then
  if docker_up; then BACKEND="docker"; else BACKEND="jena"; fi
fi
case "$BACKEND" in
  jena|docker) ;;
  *) die "FUSEKI_BACKEND must be 'jena' or 'docker' (got '$BACKEND')" ;;
esac
log "backend: $BACKEND (port $FUSEKI_PORT, dataset /$FUSEKI_DS)"

# ---- scratch store dir + ALWAYS-RUN teardown -------------------------------------------
STORE_DIR="$(mktemp -d "${TMPDIR:-/tmp}/fuseki-db.XXXXXX")"
SRV_PID=""
cleanup() {
  set +e
  # kill the jena-backend server if we started one, and rm -f the docker container if any
  [ -n "$SRV_PID" ] && kill "$SRV_PID" >/dev/null 2>&1
  if [ "$BACKEND" = "docker" ] && have docker; then
    docker rm -f "$FUSEKI_NAME" >/dev/null 2>&1
  fi
  rm -rf "$STORE_DIR" >/dev/null 2>&1
  log "teardown: stopped server (if any) + removed store dir"
}
trap cleanup EXIT
trap 'exit 130' INT TERM

# ---- df guard (bulk load is the disk-heavy step) ---------------------------------------
free_gb="$(df -BG --output=avail "$STORE_DIR" 2>/dev/null | tail -1 | tr -dc '0-9' || true)"
if [ -n "$free_gb" ]; then
  log "disk: ${free_gb} GB free (floor ${FUSEKI_MIN_FREE_GB} GB)"
  [ "$free_gb" -ge "$FUSEKI_MIN_FREE_GB" ] \
    || die "free disk ${free_gb} GB < floor ${FUSEKI_MIN_FREE_GB} GB — refusing Fuseki bulk load"
else
  log "could not read free disk space; skipping df guard"
fi

CORPUS_DIR="$(cd "$(dirname "$CORPUS")" && pwd)"
CORPUS_FILE="$(basename "$CORPUS")"
ENDPOINT="http://localhost:${FUSEKI_PORT}/${FUSEKI_DS}/query"

# ======================================================================================
# BACKEND: jena tarball (pure Java; the competitors.json run_recipe)
# ======================================================================================
start_jena() {
  local fs="${FUSEKI_SERVER:-fuseki-server}" tl="${TDBLOADER:-tdb2.tdbloader}"
  have "$fs" || die "fuseki-server not found — set FUSEKI_SERVER=<apache-jena-fuseki/fuseki-server> or put it on PATH"
  have "$tl" || die "tdb2.tdbloader not found — set TDBLOADER=<apache-jena/bin/tdb2.tdbloader> or put it on PATH"
  # ensure JAVA_HOME so the jena launcher scripts find a JDK
  if [ -z "${JAVA_HOME:-}" ] && have java; then
    JAVA_HOME="$(dirname "$(dirname "$(readlink -f "$(command -v java)")")")"
    export JAVA_HOME
  fi
  log "bulk-load (<= ${FUSEKI_LOAD_TIMEOUT}s) via $tl --loc $STORE_DIR $CORPUS_FILE [$FORMAT]"
  if ! timeout "$FUSEKI_LOAD_TIMEOUT" "$tl" --loc "$STORE_DIR" "$CORPUS" >&2; then
    local rc=$?
    [ "$rc" = 124 ] && die "tdb2.tdbloader hit the ${FUSEKI_LOAD_TIMEOUT}s timeout"
    die "tdb2.tdbloader failed (rc=$rc)"
  fi
  log "starting fuseki-server on :$FUSEKI_PORT over the TDB2 store"
  # [OPUS-4.8] FUSEKI_BASE into the scratch dir so fuseki-server does NOT litter the caller's
  # cwd with a `run/` working area (backups/logs/system_files/templates). Without this the repo
  # root gets polluted every run; the scratch dir is removed by the EXIT trap.
  export FUSEKI_BASE="$STORE_DIR/fuseki-base"
  mkdir -p "$FUSEKI_BASE"
  # --tdb2 --loc serves the bulk-loaded store; /$FUSEKI_DS mounts query+data endpoints.
  "$fs" --port "$FUSEKI_PORT" --tdb2 --loc "$STORE_DIR" "/$FUSEKI_DS" >>"$STORE_DIR/fuseki.log" 2>&1 &
  SRV_PID=$!
}

# ======================================================================================
# BACKEND: docker (official/community Fuseki image; bundles tdb2.tdbloader)
# ======================================================================================
start_docker() {
  docker_up || die "docker backend selected but the Docker daemon is not reachable (start it, or FUSEKI_BACKEND=jena)"
  log "pulling $FUSEKI_IMAGE (<= ${FUSEKI_PULL_TIMEOUT}s)"
  timeout "$FUSEKI_PULL_TIMEOUT" docker pull -q "$FUSEKI_IMAGE" >/dev/null \
    || log "pull failed/slow — continuing with any locally cached image"
  log "bulk-load (<= ${FUSEKI_LOAD_TIMEOUT}s) via tdb2.tdbloader inside $FUSEKI_IMAGE"
  if ! timeout "$FUSEKI_LOAD_TIMEOUT" docker run --rm \
        -v "$CORPUS_DIR":/in:ro -v "$STORE_DIR":/db "$FUSEKI_IMAGE" \
        tdb2.tdbloader --loc /db "/in/$CORPUS_FILE" >&2; then
    local rc=$?
    [ "$rc" = 124 ] && die "tdb2.tdbloader (docker) hit the ${FUSEKI_LOAD_TIMEOUT}s timeout"
    die "tdb2.tdbloader (docker) failed (rc=$rc)"
  fi
  log "starting fuseki-server container '$FUSEKI_NAME' on :$FUSEKI_PORT"
  docker rm -f "$FUSEKI_NAME" >/dev/null 2>&1 || true
  docker run -d --name "$FUSEKI_NAME" -p "${FUSEKI_PORT}:${FUSEKI_PORT}" \
    -v "$STORE_DIR":/db "$FUSEKI_IMAGE" \
    /jena-fuseki/fuseki-server --port "$FUSEKI_PORT" --tdb2 --loc /db "/$FUSEKI_DS" >/dev/null \
    || die "failed to start fuseki-server container"
}

case "$BACKEND" in
  jena)   start_jena ;;
  docker) start_docker ;;
esac

# ---- bounded readiness poll (FOR loop with hard count, NOT `while :`) -------------------
ready=0
poll_n=$(( FUSEKI_READY_TIMEOUT / 2 )); [ "$poll_n" -ge 1 ] || poll_n=1
for _ in $(seq 1 "$poll_n"); do
  # docker backend: if the container has already died, stop polling immediately.
  if [ "$BACKEND" = "docker" ]; then
    if ! docker ps --filter "name=^${FUSEKI_NAME}$" --filter status=running -q | grep -q .; then
      log "server container exited during startup — dumping last logs:"; docker logs --tail 30 "$FUSEKI_NAME" >&2 2>&1 || true
      die "Fuseki server did not stay up"
    fi
  elif [ -n "$SRV_PID" ] && ! kill -0 "$SRV_PID" >/dev/null 2>&1; then
    log "fuseki-server process exited during startup — last log:"; tail -30 "$STORE_DIR/fuseki.log" >&2 2>/dev/null || true
    die "Fuseki server did not stay up"
  fi
  if curl -fsS --max-time 5 --data-urlencode "query=SELECT * WHERE { ?s ?p ?o } LIMIT 1" \
      -H 'Accept: application/sparql-results+json' "$ENDPOINT" >/dev/null 2>&1; then
    ready=1; break
  fi
  sleep 2
done
[ "$ready" = 1 ] || die "server not ready within ${FUSEKI_READY_TIMEOUT}s — aborting (no hang)"
log "server ready at $ENDPOINT"

# ---- run queries over HTTP via the SHARED adapter, emit TSV ----------------------------
: > "$OUT_TSV"
shopt -s nullglob
any_ok=0
for q in "$QUERIES_DIR"/*.rq; do
  name="$(basename "$q" .rq)"
  # http_sparql_adapter.py emits `<engine>\t<count>\t<query_us>`; reformat to <name>\t<rows>\t<us>.
  # A transport/parse error (adapter exit 1) or a per-query timeout becomes an ERROR row.
  # [OPUS-4.8] The brace-group `|| true` is LOAD-BEARING under `set -euo pipefail`: without it a
  # per-query `timeout` firing (rc=124) propagates through `pipefail` and set -e ABORTS the whole
  # script mid-loop (observed on a slow large-result query like SP2Bench q04) instead of recording
  # a single ERROR row and moving on. The group makes the left side of the pipe always exit 0.
  row="$({ timeout "$FUSEKI_QUERY_TIMEOUT" python3 "$ADAPTER" \
             --endpoint "$ENDPOINT" --query-file "$q" --engine fuseki --iters "$ITERS" 2>/dev/null || true; } \
         | awk -F'\t' -v n="$name" 'NR==1{printf "%s\t%s\t%s\n", n, $2, $3}')"
  [ -n "$row" ] || row="$name	ERROR	fuseki"
  printf '%s\n' "$row" >> "$OUT_TSV"
  case "$row" in *$'\tERROR\t'*) ;; *) any_ok=1 ;; esac
done
shopt -u nullglob

log "wrote $OUT_TSV:"; cat "$OUT_TSV" >&2 || true
[ "$any_ok" = 1 ] || die "no query produced a non-ERROR row"
log "done (>=1 query succeeded)"
