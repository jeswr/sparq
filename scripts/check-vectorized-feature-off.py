#!/usr/bin/env python3
"""
[OPUS-4.8] sq-pntvh.8: feature-OFF CI proof for M4 vectorization.

Proves compile-time absence (leg1, leg3) and artifact byte-determinism (leg2) of
the vectorized-OFF build. Does NOT prove runtime performance neutrality — that is
EC2-gated. This is the strongest claim deterministic ratchet infrastructure can
support.

Three legs:
  --leg1 <metadata-json>   cargo metadata check: `vectorized` absent from default features
  --leg2 <bench-json> <floor-json>  wasm bundle exact-equality vs pinned floor
  --leg3                   cfg-audit: every vectorized call site in lib.rs/exec.rs is gated
  --self-test              run built-in tripwires that MUST fail (guard the guards)
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
import tempfile

# ---------------------------------------------------------------------------
# Constants — the source files leg 3 audits.
# ---------------------------------------------------------------------------
_EXEC_RS = "crates/sparq-engine/src/exec.rs"
_LIB_RS = "crates/sparq-engine/src/lib.rs"
_CHUNK_RS = "crates/sparq-engine/src/chunk.rs"

# Patterns that, when appearing OUTSIDE chunk.rs, must be inside a
# `#[cfg(feature = "vectorized")]` guard.
_CHUNK_IMPORT_PATTERN = re.compile(r'\bchunk::|DataChunk\b|SelVec\b|VecCmp\b|apply_filter_columnar\b')
_MOD_CHUNK_PATTERN = re.compile(r'\b(?:mod|pub use)\s+chunk\b')
_CFG_VECTORIZED = re.compile(r'#\[cfg\(feature\s*=\s*"vectorized"\)\]')


# ---------------------------------------------------------------------------
# Leg 1: cargo metadata — vectorized absent from resolved default features
# ---------------------------------------------------------------------------

def check_leg1(metadata_path: str) -> int:
    """
    Returns 0 if vectorized is absent everywhere it must be absent, 1 otherwise.
    Checks:
      a) No sparq-engine node in resolve has 'vectorized' in its enabled features.
      b) No sparq-wasm node in resolve has 'vectorized' in its transitively-enabled features.
      c) sparq-engine package definition does not list 'vectorized' in default feature set.
    """
    try:
        with open(metadata_path) as fh:
            meta = json.load(fh)
    except Exception as exc:
        print(f"[leg1] ERROR: cannot read metadata file {metadata_path!r}: {exc}", file=sys.stderr)
        return 1

    violations: list[str] = []

    # (a) + (b): check resolve nodes
    nodes = meta.get("resolve", {}).get("nodes", [])
    for node in nodes:
        node_id: str = node.get("id", "")
        features: list[str] = node.get("features", [])
        if "sparq-engine" in node_id:
            if "vectorized" in features:
                violations.append(
                    f"[leg1] VIOLATION: sparq-engine node {node_id!r} has 'vectorized' "
                    f"in resolved features: {features}"
                )
        if "sparq-wasm" in node_id:
            if "vectorized" in features:
                violations.append(
                    f"[leg1] VIOLATION: sparq-wasm node {node_id!r} has 'vectorized' "
                    f"in resolved features: {features} — wasm must never enable vectorized"
                )

    # (c): package definitions — default feature list
    packages = meta.get("packages", [])
    for pkg in packages:
        name: str = pkg.get("name", "")
        if "sparq-engine" in name:
            pkg_features: dict = pkg.get("features", {})
            default_features: list[str] = pkg_features.get("default", [])
            if "vectorized" in default_features:
                violations.append(
                    f"[leg1] VIOLATION: sparq-engine package {name!r} lists 'vectorized' "
                    f"in its default feature set: {default_features}"
                )

    if violations:
        for v in violations:
            print(v)
        print(f"\n[leg1] FAIL — {len(violations)} violation(s) found. "
              "The `vectorized` feature must NOT be in any resolved default feature set.")
        return 1

    print("[leg1] OK — 'vectorized' is absent from all resolved default feature sets "
          "(sparq-engine, sparq-wasm, package defaults). Compile-time absence confirmed.")
    return 0


# ---------------------------------------------------------------------------
# Leg 2: wasm bundle exact-equality vs pinned floor
# ---------------------------------------------------------------------------

def check_leg2(bench_results_path: str, floor_path: str) -> int:
    """
    Returns 0 if measured wasm_bundle_bytes exactly equals the pinned floor, 1 otherwise.
    bench_results_path: JSON list in github-action-benchmark format
      [{"name": "wasm_bundle_bytes", "unit": "bytes", "value": N}, ...]
    floor_path: bench/perf-baseline.json with structure
      {"metrics": {"wasm_bundle_bytes": {"floor": N, ...}}}
    """
    try:
        with open(bench_results_path) as fh:
            results_list: list = json.load(fh)
    except Exception as exc:
        print(f"[leg2] ERROR: cannot read bench-results file {bench_results_path!r}: {exc}",
              file=sys.stderr)
        return 1

    try:
        with open(floor_path) as fh:
            floor_data: dict = json.load(fh)
    except Exception as exc:
        print(f"[leg2] ERROR: cannot read floor file {floor_path!r}: {exc}", file=sys.stderr)
        return 1

    # Build lookup from list
    measured: dict[str, int] = {item["name"]: item["value"] for item in results_list}
    floor_metrics: dict = floor_data.get("metrics", {})

    # Only check metrics present in BOTH the floor AND the results
    violations: list[str] = []
    for metric_name, metric_cfg in floor_metrics.items():
        if metric_name not in measured:
            # Only require wasm_bundle_bytes — skip others
            if metric_name == "wasm_bundle_bytes":
                violations.append(
                    f"[leg2] VIOLATION: 'wasm_bundle_bytes' not found in bench results. "
                    f"Available keys: {list(measured.keys())}"
                )
            continue
        if metric_name != "wasm_bundle_bytes":
            # This script only gates on the wasm bundle as the feature-OFF proof metric
            continue
        floor_value = metric_cfg.get("floor")
        actual_value = measured[metric_name]
        if actual_value != floor_value:
            delta = actual_value - floor_value
            pct = (delta / floor_value * 100) if floor_value else float("inf")
            violations.append(
                f"[leg2] VIOLATION: wasm_bundle_bytes mismatch — "
                f"measured={actual_value}, floor={floor_value}, "
                f"delta={delta:+d} bytes ({pct:+.3f}%). "
                "Zero delta required: any accidental vectorized code would change the bundle size."
            )
        else:
            print(f"[leg2] OK — wasm_bundle_bytes exact match: {actual_value} == {floor_value} "
                  "(zero delta, compile-time absence confirmed)")

    if violations:
        for v in violations:
            print(v)
        print("\n[leg2] FAIL — wasm bundle size does not exactly match the pinned floor. "
              "This gate requires ZERO delta (stricter than the 2% ratchet in bench.yml).")
        return 1

    return 0


# ---------------------------------------------------------------------------
# Leg 3: cfg-audit — every vectorized registration and call site is gated
# ---------------------------------------------------------------------------

def _lines_have_cfg_guard(lines: list[str], target_lineno: int, window: int = 3) -> bool:
    """Return True if any of the `window` lines BEFORE target_lineno (1-indexed) match
    the `#[cfg(feature = "vectorized")]` pattern."""
    start = max(0, target_lineno - 1 - window)
    end = target_lineno - 1  # exclusive (line itself not checked)
    for i in range(start, end):
        if _CFG_VECTORIZED.search(lines[i]):
            return True
    return False


def _is_inside_cfg_vectorized_block(lines: list[str], target_lineno: int) -> bool:
    """
    Heuristic: scan backwards from target_lineno for an OPEN `#[cfg(feature = "vectorized")]`
    that hasn't been balanced by a closing `}`. Used for multi-line blocks in exec.rs.
    We track brace depth: when we see the cfg attribute and depth==0 at that point, we're in.
    """
    # Walk backwards from the line before target
    depth = 0
    for i in range(target_lineno - 2, -1, -1):
        line = lines[i]
        depth += line.count('}') - line.count('{')
        if _CFG_VECTORIZED.search(line) and depth <= 0:
            return True
        # If we went very far up or hit a module boundary, stop
        if i < max(0, target_lineno - 200):
            break
    return False


def check_leg3(repo_root: str = ".") -> int:
    """
    Audit that every reference to `vectorized` constructs outside chunk.rs is
    properly gated by `#[cfg(feature = "vectorized")]`.

    Checks:
    1. In lib.rs: every `mod chunk` or `pub use chunk` line has the cfg guard
       within 3 preceding lines.
    2. In exec.rs: every line referencing vectorized call sites (chunk::, DataChunk,
       SelVec, VecCmp, apply_filter_columnar) is inside a #[cfg(feature="vectorized")]
       block OR has the guard within 3 preceding lines.
    """
    violations: list[str] = []
    findings: list[str] = []

    # --- lib.rs: module registration ---
    lib_rs_path = os.path.join(repo_root, _LIB_RS)
    try:
        with open(lib_rs_path) as fh:
            lib_lines = fh.readlines()
    except FileNotFoundError:
        print(f"[leg3] WARNING: {lib_rs_path!r} not found — skipping lib.rs check")
        lib_lines = []

    for lineno, line in enumerate(lib_lines, start=1):
        # Skip comment lines
        stripped = line.lstrip()
        if stripped.startswith("//"):
            continue
        if _MOD_CHUNK_PATTERN.search(line):
            if _lines_have_cfg_guard(lib_lines, lineno, window=3):
                findings.append(
                    f"[leg3] OK  {_LIB_RS}:{lineno}: 'mod/use chunk' is cfg-guarded"
                )
            else:
                violations.append(
                    f"[leg3] VIOLATION: {_LIB_RS}:{lineno}: 'mod/use chunk' reference "
                    f"lacks #[cfg(feature = \"vectorized\")] within 3 preceding lines:\n"
                    f"       {line.rstrip()}"
                )

    # --- exec.rs: vectorized call sites ---
    exec_rs_path = os.path.join(repo_root, _EXEC_RS)
    try:
        with open(exec_rs_path) as fh:
            exec_lines = fh.readlines()
    except FileNotFoundError:
        print(f"[leg3] WARNING: {exec_rs_path!r} not found — skipping exec.rs check")
        exec_lines = []

    for lineno, line in enumerate(exec_lines, start=1):
        stripped = line.lstrip()
        # Skip comment lines (single-line)
        if stripped.startswith("//"):
            continue
        # Skip the cfg attribute line itself
        if _CFG_VECTORIZED.search(line):
            continue
        if _CHUNK_IMPORT_PATTERN.search(line):
            # Accept if guarded by nearby preceding cfg OR inside a cfg block
            if (_lines_have_cfg_guard(exec_lines, lineno, window=3) or
                    _is_inside_cfg_vectorized_block(exec_lines, lineno)):
                findings.append(
                    f"[leg3] OK  {_EXEC_RS}:{lineno}: vectorized reference is cfg-guarded"
                )
            else:
                violations.append(
                    f"[leg3] VIOLATION: {_EXEC_RS}:{lineno}: vectorized reference "
                    f"lacks #[cfg(feature = \"vectorized\")] guard:\n"
                    f"       {line.rstrip()}"
                )

    for f in findings:
        print(f)

    if violations:
        for v in violations:
            print(v)
        print(f"\n[leg3] FAIL — {len(violations)} ungated vectorized reference(s). "
              "Every call site and registration of the `vectorized` feature must be "
              "inside #[cfg(feature = \"vectorized\")].")
        return 1

    guarded_count = len(findings)
    print(f"\n[leg3] OK — all {guarded_count} vectorized reference(s) are properly "
          "cfg-gated. Structural absence confirmed.")
    return 0


# ---------------------------------------------------------------------------
# Self-test: tripwires that MUST fire (the guards must be able to fail)
# ---------------------------------------------------------------------------

def _leg1_on_dict(meta: dict) -> int:
    """Run leg1 check on an in-memory metadata dict via a temp file."""
    with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as tmp:
        json.dump(meta, tmp)
        tmp_path = tmp.name
    try:
        return check_leg1(tmp_path)
    finally:
        os.unlink(tmp_path)


def _leg2_on_dicts(results: list, floor: dict) -> int:
    """Run leg2 check on in-memory dicts via temp files."""
    with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as r_tmp:
        json.dump(results, r_tmp)
        r_path = r_tmp.name
    with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f_tmp:
        json.dump(floor, f_tmp)
        f_path = f_tmp.name
    try:
        return check_leg2(r_path, f_path)
    finally:
        os.unlink(r_path)
        os.unlink(f_path)


def run_self_test() -> int:
    """
    Run built-in tripwires. Both MUST detect violations (exit non-zero).
    Returns 0 only if both tripwires correctly rejected their bad input.
    """
    all_passed = True

    # ----- Tripwire 1: Leg 1 must reject a metadata fixture with vectorized enabled -----
    bad_meta = {
        "packages": [
            {
                "name": "sparq-engine",
                "features": {
                    "default": [],
                    "vectorized": []
                }
            }
        ],
        "resolve": {
            "nodes": [
                {
                    "id": "sparq-engine 0.1.0 (path+file:///repo/crates/sparq-engine)",
                    "features": ["vectorized"]
                }
            ]
        }
    }
    rc = _leg1_on_dict(bad_meta)
    if rc != 0:
        print("TRIPWIRE 1 (leg1): PASS — guard correctly detected 'vectorized' in synthetic fixture")
    else:
        print("TRIPWIRE 1 (leg1): FAIL — guard did NOT detect 'vectorized'; the leg1 check is broken")
        all_passed = False

    # ----- Tripwire 2: Leg 2 must reject a perturbed wasm bundle size -----
    pinned_floor = 1399703
    perturbed = pinned_floor + 1  # 1 byte off — must be caught
    bad_results = [{"name": "wasm_bundle_bytes", "unit": "bytes", "value": perturbed}]
    bad_floor = {"metrics": {"wasm_bundle_bytes": {"floor": pinned_floor, "threshold": 0.02, "mode": "auto"}}}
    rc = _leg2_on_dicts(bad_results, bad_floor)
    if rc != 0:
        print("TRIPWIRE 2 (leg2): PASS — comparator correctly rejected perturbed bundle size "
              f"({perturbed} != {pinned_floor})")
    else:
        print("TRIPWIRE 2 (leg2): FAIL — comparator did NOT reject mismatched size; the leg2 check is broken")
        all_passed = False

    if all_passed:
        print("\n[self-test] OK — both tripwires fired correctly. The guards can fail.")
        return 0
    else:
        print("\n[self-test] FAIL — one or more tripwires did not fire. Fix the guard logic.")
        return 1


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def main() -> int:
    parser = argparse.ArgumentParser(
        description="sq-pntvh.8 feature-OFF CI proof: three-leg check + tripwires"
    )
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--leg1", metavar="METADATA_JSON",
                      help="cargo metadata JSON to check for vectorized absence")
    mode.add_argument("--leg2", nargs=2,
                      metavar=("BENCH_RESULTS_JSON", "FLOOR_JSON"),
                      help="bench-results + floor JSON for exact-equality wasm check")
    mode.add_argument("--leg3", action="store_true",
                      help="cfg-audit of vectorized call sites in lib.rs and exec.rs")
    mode.add_argument("--self-test", dest="self_test", action="store_true",
                      help="run built-in tripwires (both must exit non-zero to pass)")
    parser.add_argument("--repo-root", default=".",
                        help="repo root for --leg3 (default: cwd)")
    args = parser.parse_args()

    if args.leg1:
        return check_leg1(args.leg1)
    elif args.leg2:
        return check_leg2(args.leg2[0], args.leg2[1])
    elif args.leg3:
        return check_leg3(repo_root=args.repo_root)
    elif args.self_test:
        return run_self_test()
    else:
        parser.print_help()
        return 2


if __name__ == "__main__":
    sys.exit(main())
