#!/usr/bin/env bash
# [SONNET-4.6] run_differential_harness.sh -- PROOF M1 (sq-3x7dl.14.1).
#
# Drives the IEEE 754 differential oracle harness:
#   1. Build the Rust harness (zk/ieee754/differential/).
#   2. Generate the oracle Noir test file using native Rust f32/f64 + half::f16.
#   3. Create a temporary Nargo package depending on sparq_ieee754.
#   4. Run `nargo test` and assert all 24 test functions pass (oracle mode).
#   5. SELF-TEST: re-run with --inject-fault, assert `nargo test` FAILS.
#      This proves the harness is non-vacuous (wired to the real Noir circuit).
#
# Usage:
#   ./scripts/run_differential_harness.sh                  # oracle + self-test (normal CI)
#   ./scripts/run_differential_harness.sh --oracle-only    # skip fault self-test (debug)
#   ./scripts/run_differential_harness.sh --update-committed  # regenerate committed oracle file
#
# The --update-committed flag writes the generated oracle to
#   zk/ieee754/tests/differential_oracle/src/lib.nr
# Run this after any change to the corpus (main.rs fixes 1-3, etc.) to keep the
# committed copy current.  CI diffs against this path to detect drift.
#
# Requirements: cargo and nargo on PATH (see .github/workflows/zk-toolchain.yml
# for the pinned toolchain install -- NARGO_VERSION=1.0.0-beta.21).
#
# This is VERIFICATION not proof -- see zk/ieee754/differential/README.md for the TCB.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
IEEE754_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
HARNESS_DIR="${IEEE754_DIR}/differential"
COMMITTED_ORACLE="${IEEE754_DIR}/tests/differential_oracle/src/lib.nr"

ORACLE_ONLY=false
UPDATE_COMMITTED=false
for arg in "$@"; do
    case "$arg" in
        --oracle-only) ORACLE_ONLY=true ;;
        --update-committed) UPDATE_COMMITTED=true ;;
    esac
done

# Verify tools are available.
if ! command -v cargo &> /dev/null; then
    echo "ERROR: cargo not found -- install Rust toolchain first" >&2
    exit 1
fi
if ! command -v nargo &> /dev/null; then
    echo "ERROR: nargo not found -- see .github/workflows/zk-toolchain.yml for install" >&2
    exit 1
fi

echo "[differential-harness] Building Rust harness..."
cargo build --manifest-path "${HARNESS_DIR}/Cargo.toml" --release --locked 2>&1
HARNESS_BIN="${HARNESS_DIR}/target/release/sparq-ieee754-differential"

# Temporary directory for oracle and fault Nargo packages; cleaned on exit.
TMPDIR_BASE="$(mktemp -d "${TMPDIR:-/tmp}/sparq-ieee754-differential.XXXXXX")"
cleanup() {
    rm -rf "${TMPDIR_BASE}"
}
trap cleanup EXIT

# Write a Nargo.toml for a test package depending on sparq_ieee754.
write_nargo_toml() {
    local pkg_dir="$1"
    local pkg_name="$2"
    mkdir -p "${pkg_dir}/src"
    cat > "${pkg_dir}/Nargo.toml" <<NARGO
[package]
name = "${pkg_name}"
type = "lib"
authors = [""]

[dependencies]
sparq_ieee754 = { path = "${IEEE754_DIR}" }
NARGO
}

# ---------------------------------------------------------------------------
# Step 1: Oracle mode -- generate correct expected values and run nargo test.
# ---------------------------------------------------------------------------
ORACLE_DIR="${TMPDIR_BASE}/oracle"
write_nargo_toml "${ORACLE_DIR}" "sparq_ieee754_differential_oracle"

echo "[differential-harness] Generating oracle Noir test file..."
"${HARNESS_BIN}" --output "${ORACLE_DIR}/src/lib.nr"

if [ "${UPDATE_COMMITTED}" = "true" ]; then
    echo "[differential-harness] Updating committed oracle: ${COMMITTED_ORACLE}"
    "${HARNESS_BIN}" --output "${COMMITTED_ORACLE}"
    echo "[differential-harness] Committed oracle updated (stage and commit the change)."
fi

echo "[differential-harness] Running nargo test (oracle mode)..."
cd "${ORACLE_DIR}"
nargo test

echo "[differential-harness] Oracle mode: all differential tests passed."

if [ "${ORACLE_ONLY}" = "true" ]; then
    echo "[differential-harness] --oracle-only: skipping self-test."
    exit 0
fi

# ---------------------------------------------------------------------------
# Step 2: Self-test (inject-fault) -- prove the harness is non-vacuous.
# A deliberately bit-flipped expected value MUST cause nargo test to fail.
# If nargo test passes on the fault-injected file, the harness is broken.
# ---------------------------------------------------------------------------
FAULT_DIR="${TMPDIR_BASE}/fault"
write_nargo_toml "${FAULT_DIR}" "sparq_ieee754_differential_fault"

echo "[differential-harness] Generating fault-injected Noir test file (self-test)..."
"${HARNESS_BIN}" --inject-fault "${FAULT_DIR}/src/lib.nr"

echo "[differential-harness] Running nargo test on fault-injected file (MUST fail)..."
cd "${FAULT_DIR}"
if nargo test 2>/dev/null; then
    echo "ERROR: nargo test PASSED on a deliberately bit-flipped expected value." >&2
    echo "ERROR: The differential harness is VACUOUS -- it is not wired to the Noir circuit." >&2
    exit 1
fi

echo "[differential-harness] Self-test PASSED: fault injection correctly detected."
echo "[differential-harness] Differential harness is non-vacuous and wired to sparq_ieee754."
