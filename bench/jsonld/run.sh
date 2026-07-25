#!/usr/bin/env bash
# [FABLE-5] sq-hmd7l.15 — bench/jsonld runner (see README.md).
#
#   run.sh --smoke    offline, sparq-only, self-asserting (the CI-shaped mode):
#                     1. expand each vendored fixture and require DEEP EQUALITY
#                        (the conformance comparator) against the vendored
#                        jsonld.js-generated expected expanded document.
#                     2. flatten + toRdf: require canonical-RDF-dataset equality
#                        (blank-node-blind) against the vendored expectations.
#                     3. compact: require the losslessness invariant
#                        reparse(compact(D, ctx)) ≡ D.
#                     4. assert the deterministic anchors vs expected.tsv.
#                     5. NEGATIVE self-test: the comparator must be able to say
#                        NO (a cross-fixture compare must fail) — the equality
#                        gate is proven non-vacuous on every run.
#   run.sh --gather   same gates, then peer throughput via scripts/bench-adapters/
#                     (jsonld.js; titanium when TITANIUM_CP is set). Timing rows
#                     are emitted ONLY for fixture×op pairs whose output-equality
#                     gate passed. Results land in bench/competitor-results/
#                     (git-ignored). Delegates to gather.py.
#
# Usage: bench/jsonld/run.sh --smoke|--gather   (run from anywhere)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FIX="$ROOT/bench/jsonld/fixtures"
EXP="$ROOT/bench/jsonld/expected.tsv"
BIN="${BENCH_JSONLD:-$ROOT/target/release/examples/bench_jsonld}"
MODE="${1:---smoke}"

if [ ! -x "$BIN" ]; then
  echo "[jsonld] ERROR: bench_jsonld not found at $BIN" >&2
  echo "[jsonld]   build: cargo build --release -p sparq-conformance --features jsonld-suite --example bench_jsonld" >&2
  exit 1
fi

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
fail=0
FIXTURES=(wdc-product people-graph context-heavy)

# ---- 1–3. output-equality + losslessness gates (HARD; exit 1 on any drift) ---
for n in "${FIXTURES[@]}"; do
  "$BIN" expand "$FIX/$n.jsonld" --out "$TMP/$n.expanded.json"
  if ! "$BIN" equal "$TMP/$n.expanded.json" "$FIX/expected/$n.expanded.jsonld" >/dev/null; then
    echo "[jsonld] ERROR: $n expand != vendored jsonld.js expansion (deep-equality)" >&2; fail=1
  fi
  "$BIN" flatten "$FIX/$n.jsonld" --out "$TMP/$n.flattened.json"
  if ! "$BIN" dataset-equal "$TMP/$n.flattened.json" "$FIX/expected/$n.flattened.jsonld" >/dev/null; then
    echo "[jsonld] ERROR: $n flatten != vendored expectation (canonical dataset)" >&2; fail=1
  fi
  if ! "$BIN" dataset-equal "$FIX/$n.jsonld" "$FIX/expected/$n.nq" >/dev/null; then
    echo "[jsonld] ERROR: $n toRdf != vendored jsonld.js N-Quads (canonical dataset)" >&2; fail=1
  fi
  if ! "$BIN" roundtrip-compact "$FIX/$n.jsonld" "$FIX/contexts/$n-context.jsonld" >/dev/null; then
    echo "[jsonld] ERROR: $n compact round-trip is NOT lossless" >&2; fail=1
  fi
done

# ---- 4. deterministic anchors vs expected.tsv --------------------------------
seen=0
for n in "${FIXTURES[@]}"; do
  while IFS=$'\t' read -r key val; do
    exp="$(awk -F'\t' -v k="${n}_${key}" '$1==k{print $2}' "$EXP")"
    if [ -z "$exp" ]; then
      echo "[jsonld] ERROR: ${n}_${key} has no expected.tsv entry" >&2; fail=1; continue
    fi
    if [ "$val" != "$exp" ]; then
      echo "[jsonld] ERROR: ${n}_${key}=$val expected=$exp (structure regression)" >&2; fail=1
    fi
    seen=$((seen+1))
  done < <("$BIN" counts "$FIX/$n.jsonld")
done
exp_rows=$(grep -cvE '^#|^$' "$EXP")
if [ "$seen" != "$exp_rows" ]; then
  echo "[jsonld] ERROR: asserted $seen anchors, expected $exp_rows (an anchor vanished)" >&2; fail=1
fi

# ---- 5. NEGATIVE self-test: the gate must be able to FAIL --------------------
if "$BIN" equal "$FIX/expected/wdc-product.expanded.jsonld" "$FIX/expected/people-graph.expanded.jsonld" >/dev/null; then
  echo "[jsonld] ERROR: comparator judged two DIFFERENT documents equal — gate is vacuous" >&2; fail=1
fi

if [ "$fail" != 0 ]; then
  echo "[jsonld] FAILED: output-equality / anchor drift (see above)" >&2
  exit 1
fi
echo "[jsonld] OK: expand deep-equality + flatten/toRdf dataset-equality + compact losslessness green on ${#FIXTURES[@]} fixtures" >&2

if [ "$MODE" = "--gather" ]; then
  exec python3 "$ROOT/bench/jsonld/gather.py" "${@:2}"
fi
