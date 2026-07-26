#!/usr/bin/env bash
# [SONNET-4.6] run_differential_harness.sh — PROOF M1 (sq-3x7dl.14.2).
#
# Drives the XPath differential oracle harness:
#   1. Run the Rust harness's own unit tests (the F&O references, the self-expiring
#      oracle-divergence guards, the fault-injection non-vacuity check).
#   2. Build the harness (zk/xpath/differential/).
#   3. Generate the oracle Noir test file — every expected value read back from sparq's
#      own Rust SPARQL/XSD scalar evaluator, cross-checked bit-for-bit against native f64
#      (doubles) and against an explicit XPath F&O 5.4.3 window (substring).
#   4. Create a temporary Nargo package depending on the RELEASED noir_XPath face repo.
#   5. Run `nargo test` — every differential test function must pass.
#   6. SELF-TEST: re-generate with --inject-fault and assert `nargo test` FAILS. A pass
#      there means the harness is VACUOUS (not wired to the circuit) and this script
#      exits non-zero.
#
# Usage:
#   ./scripts/run_differential_harness.sh                     # full run (normal CI)
#   ./scripts/run_differential_harness.sh --oracle-only       # skip the fault self-test
#   ./scripts/run_differential_harness.sh --generate-only     # no nargo; oracle file only
#   ./scripts/run_differential_harness.sh --update-committed  # refresh the committed golden
#
# --generate-only exists because the ORACLE half needs only cargo. CI runs it on every
# PR as a cheap drift guard; the nargo half is gated on the pinned Noir toolchain.
#
# The noir_XPath source is NOT in this repo — it was externalized to the
# sparq-org/noir_XPath face repo (sq-5reoy / #1599). Override the pinned dependency with:
#   XPATH_GIT=https://github.com/sparq-org/noir_XPath  XPATH_TAG=v0.2.0
#   XPATH_PATH=/local/checkout/xpath            # a local path dep instead of the git dep
#
# Requirements: cargo always; nargo unless --generate-only (see
# .github/workflows/zk-toolchain.yml for the pinned toolchain install).
#
# This is VERIFICATION, not proof — see zk/xpath/differential/README.md for the TCB.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
XPATH_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
HARNESS_DIR="${XPATH_DIR}/differential"
COMMITTED_ORACLE="${XPATH_DIR}/tests/differential_oracle/src/lib.nr"

# Pinned released face repo. A floating default would silently change what is being
# verified, so both the URL and the TAG are explicit and overridable.
XPATH_GIT="${XPATH_GIT:-https://github.com/sparq-org/noir_XPath}"
XPATH_TAG="${XPATH_TAG:-v0.2.0}"
XPATH_PATH="${XPATH_PATH:-}"

ORACLE_ONLY=false
GENERATE_ONLY=false
UPDATE_COMMITTED=false
for arg in "$@"; do
    case "$arg" in
        --oracle-only) ORACLE_ONLY=true ;;
        --generate-only) GENERATE_ONLY=true ;;
        --update-committed) UPDATE_COMMITTED=true ;;
        *) echo "ERROR: unknown argument: $arg" >&2; exit 2 ;;
    esac
done

if ! command -v cargo &> /dev/null; then
    echo "ERROR: cargo not found — install the Rust toolchain first" >&2
    exit 1
fi
if [ "${GENERATE_ONLY}" = "false" ] && ! command -v nargo &> /dev/null; then
    echo "ERROR: nargo not found — see .github/workflows/zk-toolchain.yml for the pinned install," >&2
    echo "       or pass --generate-only to run just the oracle half." >&2
    exit 1
fi

# The harness's OWN tests gate everything below: they pin the F&O reference
# implementations, assert each recorded sparq-engine divergence still reproduces (so a
# stale workaround cannot survive an engine fix), and prove --inject-fault corrupts
# exactly one LIVE assertion. Deliberately NOT --locked: the harness path-deps on
# in-repo crates whose manifests bump on lanes that never run this script, so a pinned
# lock drifts silently on main and would fail an innocent PR (the sq-q134e lesson).
echo "[xpath-differential] Running harness unit tests..."
cargo test --manifest-path "${HARNESS_DIR}/Cargo.toml"

echo "[xpath-differential] Building the Rust harness..."
cargo build --manifest-path "${HARNESS_DIR}/Cargo.toml" --release
HARNESS_BIN="${HARNESS_DIR}/target/release/sparq-xpath-differential"

if [ "${UPDATE_COMMITTED}" = "true" ]; then
    echo "[xpath-differential] Updating the committed oracle: ${COMMITTED_ORACLE}"
    "${HARNESS_BIN}" --output "${COMMITTED_ORACLE}"
    echo "[xpath-differential] Committed oracle updated (stage and commit the change)."
fi

if [ "${GENERATE_ONLY}" = "true" ]; then
    GEN_TMP="$(mktemp "${TMPDIR:-/tmp}/sparq-xpath-oracle.XXXXXX.nr")"
    trap 'rm -f "${GEN_TMP}"' EXIT
    "${HARNESS_BIN}" --output "${GEN_TMP}"
    if ! diff -u "${COMMITTED_ORACLE}" "${GEN_TMP}"; then
        echo "ERROR: the committed oracle file has DRIFTED from the generator's output." >&2
        echo "       Regenerate with: bash zk/xpath/scripts/run_differential_harness.sh --update-committed" >&2
        exit 1
    fi
    echo "[xpath-differential] --generate-only: committed oracle matches the generator. No drift."
    exit 0
fi

# Temporary Nargo packages for the oracle and fault runs; cleaned on exit.
TMPDIR_BASE="$(mktemp -d "${TMPDIR:-/tmp}/sparq-xpath-differential.XXXXXX")"
cleanup() { rm -rf "${TMPDIR_BASE}"; }
trap cleanup EXIT

# Write a Nargo.toml for a test package depending on noir_XPath. The dependency KEY is
# the module alias the generated file imports (`use xpath::…`), so it stays `xpath`
# regardless of the face repo's own package name.
write_nargo_toml() {
    local pkg_dir="$1"
    local pkg_name="$2"
    mkdir -p "${pkg_dir}/src"
    {
        echo "[package]"
        echo "name = \"${pkg_name}\""
        echo "type = \"lib\""
        echo "authors = [\"\"]"
        echo
        echo "[dependencies]"
        if [ -n "${XPATH_PATH}" ]; then
            echo "xpath = { path = \"${XPATH_PATH}\" }"
        else
            echo "xpath = { git = \"${XPATH_GIT}\", tag = \"${XPATH_TAG}\" }"
        fi
    } > "${pkg_dir}/Nargo.toml"
}

if [ -n "${XPATH_PATH}" ]; then
    echo "[xpath-differential] noir_XPath source: local path ${XPATH_PATH}"
else
    echo "[xpath-differential] noir_XPath source: ${XPATH_GIT} @ ${XPATH_TAG}"
fi

# ---------------------------------------------------------------------------
# Step 1: oracle mode — generate the correct expected values and run nargo test.
# ---------------------------------------------------------------------------
ORACLE_DIR="${TMPDIR_BASE}/oracle"
write_nargo_toml "${ORACLE_DIR}" "sparq_xpath_differential_oracle"

echo "[xpath-differential] Generating the oracle Noir test file..."
"${HARNESS_BIN}" --output "${ORACLE_DIR}/src/lib.nr"

# Drift guard: the committed golden must equal what the generator just produced, so a
# corpus or oracle change cannot land without the reviewable diff.
if ! diff -u "${COMMITTED_ORACLE}" "${ORACLE_DIR}/src/lib.nr"; then
    echo "ERROR: the committed oracle file has DRIFTED from the generator's output." >&2
    echo "       Regenerate with: bash zk/xpath/scripts/run_differential_harness.sh --update-committed" >&2
    exit 1
fi

echo "[xpath-differential] Running nargo test (oracle mode)..."
(cd "${ORACLE_DIR}" && nargo test)

echo "[xpath-differential] Oracle mode: all differential tests passed."

if [ "${ORACLE_ONLY}" = "true" ]; then
    echo "[xpath-differential] --oracle-only: skipping the fault self-test."
    exit 0
fi

# ---------------------------------------------------------------------------
# Step 2: self-test (inject-fault) — prove the harness is non-vacuous.
# A deliberately corrupted expected value MUST make nargo test fail. If it passes, the
# generated file is not actually exercising the circuit and every green run is worthless.
# ---------------------------------------------------------------------------
FAULT_DIR="${TMPDIR_BASE}/fault"
write_nargo_toml "${FAULT_DIR}" "sparq_xpath_differential_fault"

echo "[xpath-differential] Generating the fault-injected Noir test file (self-test)..."
"${HARNESS_BIN}" --inject-fault "${FAULT_DIR}/src/lib.nr"

echo "[xpath-differential] Running nargo test on the fault-injected file (MUST fail)..."
if (cd "${FAULT_DIR}" && nargo test) 2>/dev/null; then
    echo "ERROR: nargo test PASSED on a deliberately corrupted expected value." >&2
    echo "ERROR: the differential harness is VACUOUS — it is not wired to the Noir circuit." >&2
    exit 1
fi

echo "[xpath-differential] Self-test PASSED: fault injection correctly detected."
echo "[xpath-differential] The differential harness is non-vacuous and wired to noir_XPath."
