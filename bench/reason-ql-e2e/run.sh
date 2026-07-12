#!/usr/bin/env bash
# [GPT-5.6] sq-mg1wx: correctness-first OWL 2 QL end-to-end comparison driver.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
mode="${1:-}"
out="$(mktemp)"
trap 'rm -f "$out"' EXIT

cargo_args=(run -p sparq-reason-ql --example ql_endtoend_bench --features experimental)
if [[ "$mode" != "--smoke" ]]; then
  cargo_args+=(--release)
fi
(cd "$root" && cargo "${cargo_args[@]}") >"$out"

# The answer gate consumes the complete output before any timing report is rendered.
awk -F '\t' '
  NR==FNR { if (FNR > 1) expected[$1]=$2; next }
  FNR==1 { next }
  { seen[$1]=1; if (!($1 in expected) || $2 != expected[$1]) bad=1 }
  END {
    for (id in expected) if (!(id in seen)) bad=1
    if (bad) exit 1
  }
' "$here/expected.tsv" "$out" || {
  echo "FAIL: sparq answer-set sizes differ from expected.tsv; timing suppressed" >&2
  exit 1
}

ontop_file="${ONTOP_RESULTS:-}"
if [[ -n "$ontop_file" ]]; then
  awk -F '\t' '
    NR==FNR { if (FNR > 1) expected[$1]=$2; next }
    FNR==1 { next }
    { seen[$1]=1; if (!($1 in expected) || $2 != expected[$1]) bad=1 }
    END { for (id in expected) if (!(id in seen)) bad=1; if (bad) exit 1 }
  ' "$here/expected.tsv" "$ontop_file" || {
    echo "FAIL: Ontop answer-set sizes differ from expected.tsv; timing suppressed" >&2
    exit 1
  }
fi

printf 'case\tanswers\tsparq_rewriter_phase_ms\tsparq_end_to_end_ms\tontop_end_to_end_ms\n'
awk -F '\t' -v ontop="$ontop_file" '
  BEGIN {
    if (ontop != "") while ((getline line < ontop) > 0) {
      split(line, f, "\t"); if (f[1] != "case") ontop_ms[f[1]]=f[3]
    }
  }
  FNR==1 { next }
  { competitor=(($1 in ontop_ms) ? ontop_ms[$1] : "NA"); print $1 "\t" $2 "\t" $3 "\t" $4 "\t" competitor }
' "$out"
