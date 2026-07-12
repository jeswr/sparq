#!/usr/bin/env python3
"""[GPT-5.6] Re-run the sq-6xasp.1 positive and negative wasm probes."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent
ROUTER = ROOT / "router-probe" / "Cargo.toml"
VERIFIER = ROOT / "verifier-no-network" / "Cargo.toml"


def run(*args: str, expect_success: bool = True) -> subprocess.CompletedProcess[str]:
    """Run one probe command and enforce its expected exit state."""
    print("+", " ".join(args), flush=True)
    result = subprocess.run(args, cwd=ROOT, text=True, capture_output=True, check=False)
    sys.stdout.write(result.stdout)
    sys.stderr.write(result.stderr)
    if expect_success and result.returncode != 0:
        raise SystemExit(f"command unexpectedly failed with {result.returncode}")
    if not expect_success and result.returncode == 0:
        raise SystemExit("negative verifier probe unexpectedly compiled for wasm32")
    return result


run("cargo", "test", "--manifest-path", str(ROUTER))
run(
    "cargo",
    "clippy",
    "--manifest-path",
    str(ROUTER),
    "--all-targets",
    "--",
    "-D",
    "warnings",
)
run("wasm-pack", "test", "--node", str(ROUTER.parent))

tree = run(
    "cargo",
    "tree",
    "--manifest-path",
    str(ROUTER),
    "--target",
    "wasm32-unknown-unknown",
    "-e",
    "normal",
).stdout
for forbidden in ("tokio ", "mio ", "rustls ", "aws-lc", "reqwest "):
    if forbidden in tree:
        raise SystemExit(f"router probe unexpectedly includes native dependency: {forbidden}")

failure = run(
    "cargo",
    "check",
    "--manifest-path",
    str(VERIFIER),
    "--target",
    "wasm32-unknown-unknown",
    expect_success=False,
)
failure_text = failure.stdout + failure.stderr
if "aws-lc-sys" not in failure_text:
    raise SystemExit("verifier failure did not reach the expected aws-lc-sys blocker")

verifier_tree = run(
    "cargo",
    "tree",
    "--manifest-path",
    str(VERIFIER),
    "--target",
    "wasm32-unknown-unknown",
    "-e",
    "features",
).stdout
for required in (
    "solid-oidc-verifier",
    'jsonwebtoken feature "aws_lc_rs"',
    "aws-lc-sys",
):
    if required not in verifier_tree:
        raise SystemExit(f"verifier graph is missing expected blocker evidence: {required}")
for excluded in ("reqwest ", "hickory-resolver ", "tokio "):
    if excluded in verifier_tree:
        raise SystemExit(f"no-network verifier probe unexpectedly includes: {excluded}")

print("sq-6xasp.1 probe verdict reproduced", flush=True)
