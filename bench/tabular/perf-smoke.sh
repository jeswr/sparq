#!/usr/bin/env bash
# [FABLE-5] (sq-lsp7k.8) Perf smoke for the `tabular` materializing CSV->RDF import.
#
# Generates a deterministic synthetic 4-column CSV (no RNG; content is a pure function of
# the row index), then measures the CLI's OWN engine-internal timer for
#   (a) the full one-shot import (CSV -> direct mapping -> parallel NT ingest -> Graph), and
#   (b) the mapping stage alone (--out to /dev/null: CSV -> N-Triples stream, no graph).
# Each row generates 5 triples (4 columns + rdf:type). Wall-clock sensitive: run on a quiet
# box and report ratios/min-of-N per bench/CATALOG.md conventions. Results go to stdout —
# never hard-code the numbers into markdown.
#
# usage: bench/tabular/perf-smoke.sh [rows=1000000] [iters=3]
set -euo pipefail
cd "$(dirname "$0")/../.."
rows="${1:-1000000}"
iters="${2:-3}"

scratch="$(mktemp -d "${TMPDIR:-/tmp}/sparq-tabular-smoke.XXXXXX")"
trap 'rm -rf "$scratch"' EXIT
csv="$scratch/people.csv"

# Deterministic content: id,name,score,active — mixed inferred datatypes.
awk -v n="$rows" 'BEGIN {
  print "id,name,score,active"
  for (i = 1; i <= n; i++)
    printf "%d,person %d,%d.%02d,%s\n", i, i, i % 1000, i % 100, (i % 2 ? "true" : "false")
}' > "$csv"

cargo build --release -p sparq-cli --features tabular >/dev/null
bin=target/release/sparq-cli

echo "tabular-import-smoke rows=$rows triples=$((rows * 5)) iters=$iters (engine-internal timers; min-of-N)"
for mode in load out; do
  for i in $(seq "$iters"); do
    if [ "$mode" = load ]; then
      line="$("$bin" tabular "$csv" 2>&1 >/dev/null | grep 'loaded')"
    else
      line="$("$bin" tabular "$csv" --out /dev/null 2>&1 >/dev/null | grep 'wrote')"
    fi
    echo "  $mode iter=$i: $line"
  done
done
