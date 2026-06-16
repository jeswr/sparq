#!/usr/bin/env bash
# [OPUS-4.8] Hermetic both-direction self-tests for the _typos.toml allow-list
# (bead sq-hyzq — CI hygiene: stop recurring SPARQL-keyword false-positives blocking
# the docs-quality `typos` gate). Authored by Opus 4.8 (Fable unavailable; flag for
# re-review when Fable returns).
#
# WHY: typos (crate-ci/typos) splits an ALL-CAPS SPARQL keyword written with a lowercase
# English suffix at the case boundary and flags the keyword-minus-tail or the dangling
# 3-char fragment — DELETEd->DELET, INSERTed->INSER, "3 OPTIONALs"->OPTIONA, LOADed->Ded.
# These are not typos; they are how anyone naturally writes SPARQL verbs in prose, so they
# kept blocking the HARD gate and forcing pointless reword round-trips on doc PRs. The fix
# adds one narrow `extend-ignore-re` regex (>=3 capitals immediately followed by 1-3
# lowercase letters) plus an `invokable` word entry. This harness pins BOTH directions so
# the allow-list can't silently over- or under-reach later:
#   - should-PASS (SUPPRESSED): the keyword-suffix family + `invokable` must NOT be flagged.
#   - should-FAIL (STILL FIRES): a genuine ALL-CAPS misspelling and ordinary lowercase
#                                typos MUST still be caught, so the gate keeps its teeth.
#
# HERMETIC w.r.t. repo content/network: every case is a one-line fixture written to a
# temp file and run through the SHIPPED `typos --config _typos.toml` binary in isolation —
# no repo scan, no network. It exercises the exact config the gate ships (no duplication).
#
# Run:  bash scripts/tests/test_typos_allowlist.sh   (exit 0 = all pass, 1 = a case failed)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONFIG="${ROOT}/_typos.toml"

[ -f "$CONFIG" ] || { echo "FATAL: _typos.toml not found at ${CONFIG}"; exit 2; }
command -v typos >/dev/null 2>&1 || { echo "FATAL: 'typos' binary not on PATH"; exit 2; }

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
FIX="${TMP}/fixture.md"

# Echo "FAIL" if typos would report >=1 misspelling on this line, "PASS" otherwise.
# Uses the SHIPPED config; `--isolated` ignores any ambient/parent _typos.toml so only the
# project config under test applies.
verdict() {
  printf '%s\n' "$1" > "$FIX"
  if typos --isolated --config "$CONFIG" "$FIX" >/dev/null 2>&1; then
    echo PASS   # exit 0 => no typo found => suppressed/clean
  else
    echo FAIL   # non-zero => typo reported
  fi
}

pass=0
fail=0
check() {  # check <expected FAIL|PASS> <line>
  local want="$1" line="$2" got
  got="$(verdict "$line")"
  if [ "$got" = "$want" ]; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    printf 'CASE FAILED: wanted=%-4s got=%-4s  | %s\n' "$want" "$got" "$line"
  fi
}

# --------------------------------------------------------------------------- #
# Direction A — these MUST be SUPPRESSED (recurring false-positives the bead names).
# The whole "ALL-CAPS SPARQL keyword + lowercase suffix" family, plus `invokable`.
# --------------------------------------------------------------------------- #
check PASS "The triples were DELETEd from the default graph."
check PASS "We INSERTed three quads and then SELECTed the bindings."
check PASS "The query CONSTRUCTed a graph and DESCRIBEd the resource."
check PASS "The named graph was CLEARed and the dump was LOADed."
check PASS "We UNIONed two patterns and MATCHed the BGP."
check PASS "Product detail with 3 OPTIONALs and one nested OPTIONALs."
check PASS "This helper is invokable from the CLI."

# --------------------------------------------------------------------------- #
# Direction B — these MUST still FIRE (the gate keeps catching real misspellings).
# --------------------------------------------------------------------------- #
# A genuine ALL-CAPS misspelling has no lowercase suffix, so the regex does NOT shield it.
check FAIL "The header says RECIEVE instead of RECEIVE."
# Ordinary lowercase typos are untouched by the keyword regex.
check FAIL "Fix teh wording in the paragraph."
check FAIL "Re-order hte two clauses."
# A bare keyword with a real lowercase typo glued on a DIFFERENT word must still fire:
# `seperate` is lowercase, unrelated to the regex, and stays caught.
check FAIL "Keep the two stores seperate."

# --------------------------------------------------------------------------- #
echo ""
echo "test_typos_allowlist: ${pass} passed, ${fail} failed."
[ "$fail" -eq 0 ] || exit 1
echo "test_typos_allowlist: OK — keyword-suffix false-positives suppressed, real typos still caught."
