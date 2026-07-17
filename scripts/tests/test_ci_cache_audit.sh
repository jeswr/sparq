#!/usr/bin/env bash
# [FABLE-5] Hermetic self-tests for scripts/ci-cache-audit.sh (bead sq-6vshe.5).
#
# WHY: the audit feeds `gh api` JSONL to a `python3 - <<'PY'` heredoc. A heredoc
# occupies stdin (it is where `python3 -` reads its program), so piping the API
# output into it silently discards every entry — the audit reported 0 bytes /
# 0 entries / OK regardless of real usage (PR #2521 review). The script now lands
# the JSONL in a temp file the Python opens explicitly; these cases pin that the
# entries actually flow through and the verdict thresholds classify correctly.
#
# HERMETIC: `gh` is PATH-shadowed by a stub that cats a synthetic JSONL fixture
# (GH_REPO is set, so the stub never needs `repo view`). No network, no real gh.
#
# Run:  bash scripts/tests/test_ci_cache_audit.sh   (exit 0 = all pass, 1 = a failure)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="${ROOT}/scripts/ci-cache-audit.sh"

[ -f "$SCRIPT" ] || { echo "FATAL: script not found at ${SCRIPT}"; exit 2; }

pass=0
fail=0
expect() {  # expect <description> <fixture-output-var must contain pattern? yes|no> <pattern>
  local desc="$1" want="$2" pattern="$3"
  if [ "$want" = "yes" ]; then
    if grep -qF -- "$pattern" <<<"$OUT"; then pass=$((pass + 1)); else
      fail=$((fail + 1)); printf 'CASE FAILED: %s  (missing: %s)\n' "$desc" "$pattern"
    fi
  else
    if grep -qF -- "$pattern" <<<"$OUT"; then
      fail=$((fail + 1)); printf 'CASE FAILED: %s  (unexpected: %s)\n' "$desc" "$pattern"
    else pass=$((pass + 1)); fi
  fi
}

SANDBOX="$(mktemp -d)"
trap 'rm -rf "$SANDBOX"' EXIT
BIN="${SANDBOX}/bin"
mkdir -p "$BIN"

# Stubbed gh: only `gh api …` is expected (GH_REPO short-circuits `gh repo view`);
# it emits the fixture named by STUB_JSONL, ignoring pagination/jq flags.
cat >"${BIN}/gh" <<'EOF'
#!/usr/bin/env bash
[ "${1:-}" = "api" ] || { echo "gh stub: unexpected subcommand: $*" >&2; exit 9; }
cat "${STUB_JSONL}"
EOF
chmod +x "${BIN}/gh"

run_audit() {  # run_audit <fixture-path> → sets OUT and RC
  set +e
  OUT="$(env GH_REPO="stub/repo" STUB_JSONL="$1" PATH="${BIN}:${PATH}" \
         bash "$SCRIPT" 2>&1 </dev/null)"
  RC=$?
  set -e
}

# --------------------------------------------------------------------------- #
# 1. NON-EMPTY input must yield NON-ZERO totals (the PR #2521 regression: the
#    heredoc ate the pipe and everything counted as 0 entries / OK). Two entries,
#    2 GiB main-scoped + 1 GiB PR-scoped, hash suffixes differing → ONE family.
# --------------------------------------------------------------------------- #
ok_fixture="${SANDBOX}/ok.jsonl"
cat >"$ok_fixture" <<'EOF'
{"key":"v0-rust-shared-deadbeef00","ref":"refs/heads/main","size_in_bytes":2147483648,"last_accessed_at":"2026-07-01T00:00:00Z"}
{"key":"v0-rust-shared-cafebabe11","ref":"refs/pull/1/merge","size_in_bytes":1073741824,"last_accessed_at":"2026-07-02T00:00:00Z"}
EOF
run_audit "$ok_fixture"
if [ "$RC" -eq 0 ]; then pass=$((pass + 1)); else
  fail=$((fail + 1)); echo "CASE FAILED: OK fixture exits nonzero (rc=$RC)"; echo "$OUT"
fi
expect "entries are actually counted"              yes "2 entries"
expect "bytes are actually summed (3 GiB total)"   yes "3.00 GiB"
expect "hash suffixes collapse to one family"      yes "across 1 families"
expect "main/PR scope split is computed"           yes "2.00 GiB main-scoped / 1.00 GiB PR/branch-scoped"
expect "30% of budget classifies OK"               yes "**OK**"
expect "the 0-entries vacuous-audit bug is dead"   no  "0 entries"
expect "totals are not silently zero"              no  "0.00 GiB of"

# --------------------------------------------------------------------------- #
# 2. 8 GiB (80% of the 10 GiB budget) → PRESSURE.
# --------------------------------------------------------------------------- #
pressure_fixture="${SANDBOX}/pressure.jsonl"
cat >"$pressure_fixture" <<'EOF'
{"key":"v0-rust-shared-deadbeef00","ref":"refs/heads/main","size_in_bytes":8589934592,"last_accessed_at":"2026-07-01T00:00:00Z"}
EOF
run_audit "$pressure_fixture"
if [ "$RC" -eq 0 ]; then pass=$((pass + 1)); else
  fail=$((fail + 1)); echo "CASE FAILED: PRESSURE fixture exits nonzero (rc=$RC)"; echo "$OUT"
fi
expect "80% of budget classifies PRESSURE" yes "**PRESSURE**"
expect "PRESSURE is not OK"                no  "**OK**"

# --------------------------------------------------------------------------- #
# 3. 9.5 GiB (95% of budget) → THRASH.
# --------------------------------------------------------------------------- #
thrash_fixture="${SANDBOX}/thrash.jsonl"
cat >"$thrash_fixture" <<'EOF'
{"key":"v0-rust-shared-deadbeef00","ref":"refs/heads/main","size_in_bytes":10200547328,"last_accessed_at":"2026-07-01T00:00:00Z"}
EOF
run_audit "$thrash_fixture"
if [ "$RC" -eq 0 ]; then pass=$((pass + 1)); else
  fail=$((fail + 1)); echo "CASE FAILED: THRASH fixture exits nonzero (rc=$RC)"; echo "$OUT"
fi
expect "95% of budget classifies THRASH" yes "**THRASH LIKELY**"

# --------------------------------------------------------------------------- #
echo ""
echo "test_ci_cache_audit: ${pass} passed, ${fail} failed."
[ "$fail" -eq 0 ] || exit 1
echo "test_ci_cache_audit: OK — entries flow to the Python (no heredoc/stdin clash) + thresholds classify."
