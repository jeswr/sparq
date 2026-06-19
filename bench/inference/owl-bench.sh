#!/usr/bin/env bash
# Closure-throughput micro-bench for the RDFS and OWL-RL materialization paths
# (used to verify the <5% regression budget when adding rules to owl.rs).
# Workloads:
#   1. rdfs-instances   — 100k individuals under a depth-20 subClassOf chain, profile=rdfs
#   2. owl-route-rdfs   — same data through profile=owl (no OWL features → single-pass route)
#   3. owl-transitive   — 50k-edge transitive chain (semi-naive fixpoint, prp-trp)
#   4. owl-restrictions — 50k individuals × someValuesFrom/hasValue/intersection restrictions
#                         + equivalence + inverse (the class-feature fixpoint path)
# Prints min-of-3 seconds per workload (parses the CLI's "in Xs" line).
#
# [OPUS-4.8] (sq-k5qq) Strictly-additive machine-readable emit: set OWL_BENCH_JSON to a
# path and the run ALSO writes the SAME per-workload min-of-3 seconds to that file as a
# stable, hand-built JSON document (the shell analogue of the Rust harnesses'
# dependency-free `format!`-JSON). STDOUT (the printf table) is byte-for-byte unchanged
# whether or not the env var is set; the seconds are best-effort, MEASURED on the running
# host — ADVISORY + NON-CANONICAL (stated in the emitted `note`) — nothing is committed.
set -euo pipefail
cd "$(dirname "$0")/../.."
CLI="${SPARQ_CLI:-target/release/sparq-cli}"
JSON_OUT="${OWL_BENCH_JSON:-}"
JSON_ROWS=""   # accumulated `{ ... }` row objects, comma-joined at the end
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

# 1+2: instance-heavy RDFS
{ echo '@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .'; echo '@prefix : <http://ex/> .'
  for k in $(seq 0 19); do echo ":c$k rdfs:subClassOf :c$((k+1)) ."; done
  for j in $(seq 1 100000); do echo ":x$j a :c0 ."; done; } > "$TMP/rdfs.ttl"

# 3: transitive chain
{ echo '@prefix : <http://ex/> .'; echo '@prefix owl: <http://www.w3.org/2002/07/owl#> .'
  echo ':link a owl:TransitiveProperty .'
  for j in $(seq 1 50000); do echo ":n$j :link :n$((j+1)) ."; done; } > "$TMP/trans.ttl"
# NOTE: a 50k pure chain is quadratic in closure size; cap at 2k for tractability.
{ echo '@prefix : <http://ex/> .'; echo '@prefix owl: <http://www.w3.org/2002/07/owl#> .'
  echo ':link a owl:TransitiveProperty .'
  for j in $(seq 1 2000); do echo ":n$j :link :n$((j+1)) ."; done; } > "$TMP/trans.ttl"

# 4: restriction-heavy ABox
{ echo '@prefix : <http://ex/> .'; echo '@prefix owl: <http://www.w3.org/2002/07/owl#> .'
  echo '@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .'
  echo ':R1 a owl:Restriction ; owl:onProperty :hasPet ; owl:someValuesFrom :Dog .'
  echo ':R2 a owl:Restriction ; owl:onProperty :status ; owl:hasValue :active .'
  echo ':Happy owl:intersectionOf ( :R1 :R2 ) .'
  echo ':Happy owl:equivalentClass :Joyful .'
  echo ':hasPet owl:inverseOf :petOf .'
  for j in $(seq 1 50000); do
    echo ":x$j :hasPet :d$j . :d$j a :Dog . :x$j a :R2 ."
  done; } > "$TMP/restr.ttl"

run() { # name file profile
  local best=""
  for _ in 1 2 3; do
    local t
    t=$("$CLI" reason "$2" turtle "$3" 2>&1 | grep -oE 'in [0-9.]+s' | head -1 | grep -oE '[0-9.]+')
    if [ -z "$best" ] || awk "BEGIN{exit !($t < $best)}"; then best="$t"; fi
  done
  printf '%-18s %ss\n' "$1" "$best"
  # [OPUS-4.8] (sq-k5qq) accumulate one JSON row for the optional --json emit. Workload
  # name + profile are static idents (JSON-safe to quote); seconds is a bare JSON number.
  if [ -n "$JSON_OUT" ]; then
    local row="{ \"workload\": \"$1\", \"profile\": \"$3\", \"seconds\": $best }"
    if [ -z "$JSON_ROWS" ]; then JSON_ROWS="$row"; else JSON_ROWS="$JSON_ROWS,$row"; fi
  fi
}
run rdfs-instances "$TMP/rdfs.ttl" rdfs
run owl-route-rdfs "$TMP/rdfs.ttl" owl
run owl-transitive "$TMP/trans.ttl" owl
run owl-restrictions "$TMP/restr.ttl" owl

# [OPUS-4.8] (sq-k5qq) Strictly-additive JSON emit: write the accumulated rows only when
# OWL_BENCH_JSON was set. The STDOUT table above is unchanged either way.
if [ -n "$JSON_OUT" ]; then
  {
    printf '{\n'
    printf '  "harness": "sparq-reason owl-bench",\n'
    printf '  "note": "every min-of-3 second figure is best-effort, MEASURED on the running host — ADVISORY, NON-CANONICAL (this dev box) — do not bake into committed files",\n'
    printf '  "rows": [ %s ]\n' "$JSON_ROWS"
    printf '}\n'
  } > "$JSON_OUT"
  echo "wrote results JSON to $JSON_OUT" >&2
fi
