#!/usr/bin/env bash
# [OPUS-5] issue #4571 — hermetic self-test for scripts/verify-release-provenance.sh.
#
# WHY IT EXISTS. The script under test is the EVIDENCE ENGINE for SL-B3-b: its verdict is what
# flips the control from AR to IV. It runs for the first time when a release is being cut, and
# its dangerous failure mode is the silent one — a check that passes vacuously (empty asset set,
# a bundle that covers nothing, an asset silently exempted) would manufacture "verified" out of
# nothing, which is worse than no check at all. So every fail-closed branch is exercised HERE, on
# every PR, with a STUB `slsa-verifier` (the real one needs a signed bundle from a real release).
#
# The stub reads $STUB_DB — one `<bundle>|<asset>` pair per line — and exits 0 iff the pair is
# listed. That is exactly the property the real verifier has (digest-based subject matching) and
# nothing more, so the test pins the script's LOGIC, not the verifier's.
#
# Run: bash scripts/tests/test_verify_release_provenance.sh
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT/scripts/verify-release-provenance.sh"
TAG="v9.9.9"
CLI_BUNDLE="sparq-cli-${TAG}.intoto.jsonl"
ARTIFACTS_BUNDLE="sparq-artifacts-${TAG}.intoto.jsonl"

WORK="$(mktemp -d)"
trap 'rm -rf -- "$WORK"' EXIT

failures=0
pass() { echo "  ok   — $1"; }
fail() {
  echo "  FAIL — $1" >&2
  failures=$((failures + 1))
}
# check <ok-label> <fail-label> <cmd...> — an if/else, deliberately not `A && pass || fail`
# (which runs the failure branch whenever `pass` itself fails; shellcheck SC2015).
check() {
  local ok="$1" bad="$2"
  shift 2
  if "$@"; then pass "$ok"; else fail "$bad"; fi
}

# ---- stub slsa-verifier ----------------------------------------------------------------------
mkdir -p "$WORK/bin"
cat >"$WORK/bin/slsa-verifier" <<'STUB'
#!/usr/bin/env bash
# stub: `slsa-verifier verify-artifact <asset> --provenance-path <bundle> --source-uri <uri>`
set -uo pipefail
if [ "${1:-}" = "version" ]; then echo "stub-verifier 0.0.0"; exit 0; fi
asset="${2:-}"; bundle=""; uri=""
shift 2 || true
while [ "$#" -gt 0 ]; do
  case "$1" in
    --provenance-path) bundle="$2"; shift 2 ;;
    --source-uri) uri="$2"; shift 2 ;;
    *) shift ;;
  esac
done
[ -n "$uri" ] || { echo "stub: no --source-uri" >&2; exit 2; }
grep -Fqx "${bundle}|${asset}" "$STUB_DB" || { echo "stub: FAILED: no matching subject" >&2; exit 1; }
echo "stub: PASSED: verified SLSA provenance"
STUB
chmod +x "$WORK/bin/slsa-verifier"
export PATH="$WORK/bin:$PATH"

# ---- fixture builder ---------------------------------------------------------------------------
# make_release <dir> — a well-formed release: two archives (archives bundle), one .deb + one SBOM
# (artifacts bundle), one alias copy of an archive, both bundles, and a matching SHA256SUMS.
make_release() {
  local dir="$1"
  rm -rf -- "$dir"
  mkdir -p "$dir"
  (
    cd "$dir" || exit 1
    echo "archive-x64" >"sparq-cli-${TAG}-x64-baseline.tar.gz"
    echo "archive-arm" >"sparq-cli-${TAG}-arm64-linux.tar.gz"
    cp "sparq-cli-${TAG}-x64-baseline.tar.gz" sparq-cli-x64-linux.tar.gz # version-stable alias
    echo "deb" >"sparq-gui_${TAG}_amd64.deb"
    echo "sbom" >"sparq-cli-${TAG}.sbom.cdx.json"
    echo "dsse-archives" >"$CLI_BUNDLE"
    echo "dsse-artifacts" >"$ARTIFACTS_BUNDLE"
    sha256sum -- * >SHA256SUMS
  )
  {
    echo "${CLI_BUNDLE}|sparq-cli-${TAG}-x64-baseline.tar.gz"
    echo "${CLI_BUNDLE}|sparq-cli-${TAG}-arm64-linux.tar.gz"
    echo "${CLI_BUNDLE}|sparq-cli-x64-linux.tar.gz"
    echo "${ARTIFACTS_BUNDLE}|sparq-gui_${TAG}_amd64.deb"
    echo "${ARTIFACTS_BUNDLE}|sparq-cli-${TAG}.sbom.cdx.json"
  } >"$WORK/stub-db.txt"
  export STUB_DB="$WORK/stub-db.txt"
}

run_script() { # run_script <dir> -> stdout+stderr in $OUT, status in $STATUS
  OUT="$(bash "$SCRIPT" --tag "$TAG" --repo jeswr/sparq --dir "$1" 2>&1)"
  STATUS=$?
}

expect_pass() { # expect_pass <label> <dir>
  run_script "$2"
  if [ "$STATUS" -eq 0 ]; then pass "$1"; else
    fail "$1 (expected exit 0, got $STATUS)"
    echo "$OUT" | awk '{ print "       " $0 }' >&2
  fi
}

expect_fail() { # expect_fail <label> <dir> <substring the message must contain>
  run_script "$2"
  if [ "$STATUS" -eq 0 ]; then
    fail "$1 (expected non-zero exit, got 0 — a fail-closed branch is not closed)"
  elif ! printf '%s' "$OUT" | grep -qF "$3"; then
    fail "$1 (failed as expected but the message never mentioned '$3')"
    echo "$OUT" | awk '{ print "       " $0 }' >&2
  else
    pass "$1"
  fi
}

echo "== scripts/verify-release-provenance.sh =="

# 1. HAPPY PATH — every asset is a subject of one of the two bundles.
D="$WORK/happy"
make_release "$D"
expect_pass "a well-formed release verifies" "$D"
run_script "$D"
counts_reported() {
  printf '%s' "$OUT" | grep -q "archives bundle .... ${CLI_BUNDLE} — 3 asset(s) verified" &&
    printf '%s' "$OUT" | grep -q "artifacts bundle ... ${ARTIFACTS_BUNDLE} — 2 asset(s) verified"
}
check "the evidence block reports the per-bundle counts (3 archives incl. alias, 2 artifacts)" \
  "the evidence block did not report the expected per-bundle counts" counts_reported

# 2. A MISSING BUNDLE is the "the trusted builder never ran / was not attached" case.
D="$WORK/no-cli-bundle"
make_release "$D"
rm -f "$D/$CLI_BUNDLE"
expect_fail "a missing archives bundle reds" "$D" "$CLI_BUNDLE"

D="$WORK/no-artifacts-bundle"
make_release "$D"
rm -f "$D/$ARTIFACTS_BUNDLE"
expect_fail "a missing artifacts bundle reds" "$D" "$ARTIFACTS_BUNDLE"

# 3. AN EMPTY BUNDLE is not a bundle (a 0-byte upload must not read as "attached").
D="$WORK/empty-bundle"
make_release "$D"
: >"$D/$CLI_BUNDLE"
expect_fail "a zero-byte bundle reds" "$D" "missing or empty"

# 4. SHA256SUMS must EXIST, LIST both bundles, and MATCH the published bytes.
D="$WORK/no-sums"
make_release "$D"
rm -f "$D/SHA256SUMS"
expect_fail "a missing SHA256SUMS reds" "$D" "SHA256SUMS is missing or empty"

D="$WORK/unlisted-bundle"
make_release "$D"
grep -v "  ${CLI_BUNDLE}\$" "$D/SHA256SUMS" >"$D/SHA256SUMS.tmp" && mv "$D/SHA256SUMS.tmp" "$D/SHA256SUMS"
expect_fail "a bundle absent from SHA256SUMS reds" "$D" "not listed in SHA256SUMS"

D="$WORK/tampered"
make_release "$D"
echo "tampered" >"$D/sparq-cli-${TAG}-x64-baseline.tar.gz"
expect_fail "an asset whose digest no longer matches SHA256SUMS reds" "$D" "does not match the published bytes"

# 5. AN UNCOVERED ASSET — the subjects set silently lost a lane. This is the check that a plain
#    "did slsa-verifier exit 0 once" script would miss entirely.
D="$WORK/uncovered"
make_release "$D"
echo "orphan" >"$D/sparq-gui_${TAG}_x64.msi"
(cd "$D" && rm -f SHA256SUMS && sha256sum -- * >SHA256SUMS)
expect_fail "an asset covered by NEITHER bundle reds" "$D" "verify against NEITHER"

# 6. VACUITY — a directory with only the bundles + SHA256SUMS must never read as "verified".
D="$WORK/vacuous"
mkdir -p "$D"
(
  cd "$D" || exit 1
  echo "dsse-archives" >"$CLI_BUNDLE"
  echo "dsse-artifacts" >"$ARTIFACTS_BUNDLE"
  sha256sum -- * >SHA256SUMS
)
expect_fail "an asset-less release reds instead of passing vacuously" "$D" "zero assets were verified"

# 7. A BUNDLE THAT COVERS NOTHING IT PUBLISHED — both bundles must each cover >= 1 asset, so a
#    lane whose subjects all vanished cannot hide behind the other lane's successes.
D="$WORK/one-sided"
make_release "$D"
rm -f "$D/sparq-gui_${TAG}_amd64.deb" "$D/sparq-cli-${TAG}.sbom.cdx.json"
(cd "$D" && rm -f SHA256SUMS && sha256sum -- * >SHA256SUMS)
expect_fail "an artifacts bundle covering none of the published assets reds" "$D" "covering nothing it published"

# 8. ARGUMENT + TOOL PRECONDITIONS.
failed_with() { # failed_with <substring> — the last run must have red WITH that message
  [ "$STATUS" -ne 0 ] && printf '%s' "$OUT" | grep -qF -- "$1"
}

OUT="$(bash "$SCRIPT" --repo jeswr/sparq --dir "$WORK/happy" 2>&1)"
STATUS=$?
check "a missing --tag reds" "a missing --tag did not red with a usable message" \
  failed_with "--tag is required"

OUT="$(SLSA_VERIFIER=definitely-not-installed bash "$SCRIPT" --tag "$TAG" --dir "$WORK/happy" 2>&1)"
STATUS=$?
check "a missing slsa-verifier reds (never skipped as 'unavailable')" \
  "a missing slsa-verifier did not red" failed_with "not found on PATH"

# 9. The workflow that runs this in CI must actually call the script, on the publish event.
WF="$ROOT/.github/workflows/release-verify.yml"
[ -f "$WF" ] || fail "missing .github/workflows/release-verify.yml"
if [ -f "$WF" ]; then
  check "release-verify.yml invokes the script" \
    "release-verify.yml does not invoke scripts/verify-release-provenance.sh" \
    grep -q "scripts/verify-release-provenance.sh" "$WF"
  check "release-verify.yml runs on release:published" \
    "release-verify.yml lost its release:published trigger" \
    grep -q "types: \[published\]" "$WF"
  # Trigger-shaped only (a `^  pull_request:` key), so the header prose explaining WHY there is
  # no such trigger does not trip the check. NEGATED: absence is the passing condition.
  no_pr_trigger() { ! grep -qE "^  (pull_request|merge_group):" "$WF"; }
  check "release-verify.yml is PR-invisible (no pull_request/merge_group trigger)" \
    "release-verify.yml gained a PR/merge-queue trigger — it would register a check-run the gate aggregator does not expect" \
    no_pr_trigger
fi

echo
if [ "$failures" -ne 0 ]; then
  echo "FAILED: $failures check(s)" >&2
  exit 1
fi
echo "all checks passed"
