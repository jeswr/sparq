#!/usr/bin/env bash
# sparq-reason (N3 engine) vs EYE — same machine, same workloads, min-of-3
# wall seconds. Both engines parse the document, run the forward closure, and
# SERIALIZE the full closure (sparq: `reason <f> n3 n3 /dev/null`; EYE:
# `--pass > /dev/null`), so the comparison covers the whole pipeline.
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
set -euo pipefail
cd "$(dirname "$0")/../.."
CLI="${SPARQ_CLI:-target/release/sparq-cli}"
EYE="${EYE:-eye}"
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

mo3() { # min-of-3 wall seconds for "$@"
  local best=""
  for _ in 1 2 3; do
    local t0 t1 t
    t0=$(python3 -c 'import time; print(time.time())')
    "$@" > /dev/null 2>&1
    t1=$(python3 -c 'import time; print(time.time())')
    t=$(python3 -c "print(f'{$t1-$t0:.3f}')")
    if [ -z "$best" ] || python3 -c "exit(0 if $t < $best else 1)"; then best="$t"; fi
  done
  echo "$best"
}

printf '%-10s %12s %12s\n' workload sparq eye
for w in socrates dt1k dt10k dt100k anc500 grid30; do
  s=$(mo3 "$CLI" reason "$B/$w.n3" n3 n3 /dev/null)
  e=$(mo3 "$EYE" --quiet --nope "$B/$w.n3" --pass)
  printf '%-10s %11ss %11ss\n' "$w" "$s" "$e"
done
