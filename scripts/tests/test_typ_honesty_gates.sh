#!/usr/bin/env bash
# [OPUS-4.8] bead sq-mkza — end-to-end negative-test proof that the two honesty gates
# actually scan the paper-factory `.typ` sources (`site/papers/**/*.typ`) and catch a
# planted violation, while leaving the legitimate accessor / comment / negator paths
# alone. Authored by Opus 4.8 (Fable unavailable; flag for re-review when Fable returns).
#
# WHY: the highest-stakes OUTWARD surface is a published paper. Before sq-mkza the
# no-perf-numbers gate scanned only `*.md` and the privacy-claims gate scanned only
# `*.md *.mdx *.tsx *.ts`, so a hard-coded "12× faster" or an unqualified
# "maliciously secure" typed straight into a `.typ` paper tripped NEITHER gate. This
# harness PINS that the coverage now exists and does not silently regress.
#
# It runs the REAL gate scripts end-to-end (not a re-derived regex):
#   1. PERF gate  — driven by explicit `.typ` paths (it dispatches `.typ` → the
#                   accessor-aware Typst scan). Planted result-number => exit 1;
#                   comment-only + accessor-driven => exit 0.
#   2. PRIVACY gate — driven via a throwaway git repo (the gate enumerates its surface
#                   with `git ls-files`, so the fixture must be TRACKED). A planted
#                   unqualified claim in `site/papers/planted.typ` => exit 1; the same
#                   claim hedged / allow-marked => exit 0.
#
# HONEST SCOPE: these gates catch only the COARSE hard-coded-number / unqualified-phrase
# class. A subtle semantic overclaim phrased without a result-shaped unit or a forbidden
# phrase is NOT caught here — that remains Stage-5 human review (plus the build-time
# headline() canonical gate in site/papers/_lib/bench.typ).
#
# Run:  bash scripts/tests/test_typ_honesty_gates.sh   (exit 0 = all cases pass)
# Hermetic: no network; the privacy case uses a self-contained `git init` in a tmpdir.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PERF_GATE="${ROOT}/scripts/check-no-perf-numbers.py"
PRIV_GATE="${ROOT}/scripts/check-privacy-claims.sh"

[ -f "$PERF_GATE" ] || { echo "FATAL: perf gate not found at ${PERF_GATE}"; exit 2; }
[ -f "$PRIV_GATE" ] || { echo "FATAL: privacy gate not found at ${PRIV_GATE}"; exit 2; }

pass=0
fail=0
note() { printf '  %s\n' "$1"; }
expect_exit() {  # expect_exit <want-code> <label> ; reads actual code from $? via caller
  local want="$1" label="$2" got="$3"
  if [ "$got" -eq "$want" ]; then
    pass=$((pass + 1)); note "PASS  [$label] exit=$got (wanted $want)"
  else
    fail=$((fail + 1)); note "FAIL  [$label] exit=$got (wanted $want)"
  fi
}

TMP="$(mktemp -d)"
cleanup() { rm -rf "$TMP"; }
trap cleanup EXIT

echo "== 1. no-perf-numbers gate scans paper .typ sources =="

# 1a. Planted result-shaped number typed into paper PROSE must be caught (exit 1).
mkdir -p "${TMP}/perf/site/papers"
cat > "${TMP}/perf/site/papers/planted.typ" <<'TYP'
== Results
On WatDiv our engine is 12× faster than the baseline.
TYP
set +e
python3 "$PERF_GATE" --enforce "${TMP}/perf/site/papers/planted.typ" >/dev/null 2>&1
code=$?
set -e
expect_exit 1 "perf: planted '12× faster' in .typ prose => fail" "$code"

# 1b. The SAME number in a Typst comment + an accessor-driven value must NOT trip (exit 0):
#     this is the narrow/accessor-aware net — results flow through #headline()/#ev().
cat > "${TMP}/perf/site/papers/clean.typ" <<'TYP'
// dev note: an earlier draft said 12× faster — do not bake that in.
The recall floor is #headline("ann.recall_at_10_floor") over 50,000 × 32 vectors.
The crossover is #ev("filtered_ann.prefilter_crossover") of the store.
TYP
set +e
python3 "$PERF_GATE" --enforce "${TMP}/perf/site/papers/clean.typ" >/dev/null 2>&1
code=$?
set -e
expect_exit 0 "perf: comment + accessor + setup-counts in .typ => clean" "$code"

echo "== 2. check-privacy-claims gate scans paper .typ sources =="

# The privacy gate enumerates its surface with `git ls-files`, so the fixture must be a
# TRACKED file in a git repo. Build a throwaway repo that mirrors site/papers/ and copy
# in the REAL gate script so its in-repo `cd $ROOT` + ls-files run against the fixture.
REPO="${TMP}/repo"
mkdir -p "${REPO}/scripts" "${REPO}/site/papers"
cp "$PRIV_GATE" "${REPO}/scripts/check-privacy-claims.sh"
(
  cd "$REPO"
  git init -q
  git config user.email t@t && git config user.name t
)

run_priv() {  # run_priv -> echoes exit code of the gate over the current fixture tree
  (
    cd "$REPO"
    git add -A >/dev/null 2>&1
    set +e
    bash scripts/check-privacy-claims.sh >/dev/null 2>&1
    echo $?
  )
}

# 2a. Planted unqualified soundness/privacy claim in paper prose must be caught (exit 1).
cat > "${REPO}/site/papers/planted.typ" <<'TYP'
== Security
Our MPC protocol is maliciously secure, and the verifier is sound.
TYP
code="$(run_priv)"
expect_exit 1 "privacy: planted 'maliciously secure' + 'verifier is sound' in .typ => fail" "$code"

# 2b. The same mentions, but HEDGED (negator) and ALLOW-MARKED, must PASS (exit 0) —
#     proving the existing inline-allow / negator-exemption machinery applies to .typ.
cat > "${REPO}/site/papers/planted.typ" <<'TYP'
== Security
The verifier is NOT yet sound pending external audit.
We do not claim a privacy-preserving guarantee. // privacy-claims-allow: hedged paper caveat
TYP
code="$(run_priv)"
expect_exit 0 "privacy: hedged + allow-marked claim in .typ => clean" "$code"

echo ""
echo "test_typ_honesty_gates: ${pass} passed, ${fail} failed."
[ "$fail" -eq 0 ] || exit 1
echo "test_typ_honesty_gates: OK — both honesty gates cover site/papers/**/*.typ."
