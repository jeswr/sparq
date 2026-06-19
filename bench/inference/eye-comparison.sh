#!/usr/bin/env bash
# sparq-reason (N3 engine) vs EYE — same machine, same workloads, wall
# seconds (sparq min-of-3; EYE min-of-3 on the quick cells, single run on the
# minutes-long ones, dt100k skipped — see the loop). Both engines parse the
# document, run the forward closure, and SERIALIZE the full closure (sparq:
# `reason <f> n3 n3 /dev/null`; EYE: `--pass > /dev/null`), so the comparison
# covers the whole pipeline.
#
#   EYE=$HOME/.local/bin/eye SPARQ_CLI=target/release/sparq-cli \
#     bench/inference/eye-comparison.sh
#
# Workloads (generated here, scalable):
#   socrates   — vendored EYE case (startup/latency floor)
#   dt1k/10k/100k — DeepTaxonomy (gen_deeptaxonomy.py): 1 instance, N-deep
#                subClassOf chain, one transitivity meta-rule
#   anc500     — transitive ancestor chain, 500 links (quadratic closure ~125k)
#   grid30     — 30x30 grid reachability (edge → reach; reach+edge → reach)
# Results: bench/inference/eye-comparison.md
#
# [OPUS-4.8] (sq-k5qq) Strictly-additive machine-readable emit: set SPARQ_INFER_JSON to a
# path and the run ALSO writes the SAME per-workload wall seconds to that file as a stable,
# hand-built JSON document (the shell analogue of the Rust harnesses' dependency-free
# `format!`-JSON). STDOUT (the printf table) is byte-for-byte unchanged whether or not the
# env var is set; the seconds are best-effort, MEASURED on the running host — ADVISORY +
# NON-CANONICAL (stated in the emitted `note`) — nothing is committed.
set -euo pipefail
cd "$(dirname "$0")/../.."
CLI="${SPARQ_CLI:-target/release/sparq-cli}"
EYE="${EYE:-eye}"
JSON_OUT="${SPARQ_INFER_JSON:-}"
JSON_ROWS=""   # accumulated `{ ... }` row objects, comma-joined at the end
B="$(mktemp -d)"; trap 'rm -rf "$B"' EXIT

cp crates/sparq-reason/tests/eye/socrates.n3 "$B/socrates.n3"
python3 bench/inference/gen_deeptaxonomy.py 1000   > "$B/dt1k.n3"
python3 bench/inference/gen_deeptaxonomy.py 10000  > "$B/dt10k.n3"
python3 bench/inference/gen_deeptaxonomy.py 100000 > "$B/dt100k.n3"
{ echo '@prefix : <http://ex/> .'
  echo '{ ?x :ancestor ?y . ?y :ancestor ?z } => { ?x :ancestor ?z } .'
  for j in $(seq 1 500); do echo ":p$j :ancestor :p$((j+1)) ."; done; } > "$B/anc500.n3"
python3 - > "$B/grid30.n3" <<'EOF'
N = 30
print('@prefix : <http://ex/> .')
print('{ ?x :edge ?y } => { ?x :reach ?y } .')
print('{ ?x :reach ?y . ?y :edge ?z } => { ?x :reach ?z } .')
for i in range(N):
    for j in range(N):
        if i + 1 < N: print(f':n{i}_{j} :edge :n{i+1}_{j} .')
        if j + 1 < N: print(f':n{i}_{j} :edge :n{i}_{j+1} .')
EOF

mon() { # min-of-N wall seconds: mon <runs> <cmd…>
  local runs="$1"; shift
  local best=""
  for _ in $(seq 1 "$runs"); do
    local t0 t1 t
    t0=$(python3 -c 'import time; print(time.time())')
    "$@" > /dev/null 2>&1
    t1=$(python3 -c 'import time; print(time.time())')
    t=$(python3 -c "print(f'{$t1-$t0:.3f}')")
    if [ -z "$best" ] || python3 -c "exit(0 if $t < $best else 1)"; then best="$t"; fi
  done
  echo "$best"
}

# [OPUS-4.8] (sq-k5qq) Append one JSON row to the accumulator. Args: workload, sparq_s,
# eye_s ("skipped" emits a null + "skipped": true). Workload names are static idents, so a
# JSON-safe quote is enough; numeric seconds are emitted as bare JSON numbers.
add_json_row() {
  [ -n "$JSON_OUT" ] || return 0
  local w="$1" sparq="$2" eye="$3" eye_field
  if [ "$eye" = skipped ]; then
    eye_field='"eye_seconds": null, "eye_skipped": true'
  else
    eye_field="\"eye_seconds\": $eye, \"eye_skipped\": false"
  fi
  local row="{ \"workload\": \"$w\", \"sparq_seconds\": $sparq, $eye_field }"
  if [ -z "$JSON_ROWS" ]; then JSON_ROWS="$row"; else JSON_ROWS="$JSON_ROWS,$row"; fi
}

printf '%-10s %12s %12s\n' workload sparq eye
for w in socrates dt1k dt10k dt100k anc500 grid30; do
  s=$(mon 3 "$CLI" reason "$B/$w.n3" n3 n3 /dev/null)
  printf '%-10s %11ss ' "$w" "$s"
  # EYE on dt100k is ≈9 h at its observed ≈N^1.95 DT scaling — never run it by
  # default (set EYE_DT100K=1 if you truly mean it). dt10k/anc500/grid30 are
  # minutes per EYE run: single run, documented as such in eye-comparison.md.
  if [ "$w" = dt100k ] && [ -z "${EYE_DT100K:-}" ]; then
    printf '%12s\n' 'skipped'
    add_json_row "$w" "$s" skipped
    continue
  fi
  eruns=3; case "$w" in dt10k|dt100k|anc500|grid30) eruns=1;; esac
  e=$(mon "$eruns" "$EYE" --quiet --nope "$B/$w.n3" --pass)
  printf '%11ss\n' "$e"
  add_json_row "$w" "$s" "$e"
done

# [OPUS-4.8] (sq-k5qq) Strictly-additive JSON emit: write the accumulated rows only when
# SPARQ_INFER_JSON was set. The STDOUT table above is unchanged either way.
if [ -n "$JSON_OUT" ]; then
  {
    printf '{\n'
    printf '  "harness": "sparq-reason vs EYE (eye-comparison)",\n'
    printf '  "note": "every wall-second figure is best-effort, MEASURED on the running host — ADVISORY, NON-CANONICAL (this dev box) — do not bake into committed files",\n'
    printf '  "rows": [ %s ]\n' "$JSON_ROWS"
    printf '}\n'
  } > "$JSON_OUT"
  echo "wrote results JSON to $JSON_OUT" >&2
fi
