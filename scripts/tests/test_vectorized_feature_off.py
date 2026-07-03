#!/usr/bin/env python3
"""
[OPUS-4.8] sq-pntvh.8: standalone test suite for check-vectorized-feature-off.py.

Runs the same tripwires that --self-test runs, as proper test functions with a
clear summary. No external frameworks — stdlib only.

Usage:
    python3 scripts/tests/test_vectorized_feature_off.py
"""

from __future__ import annotations

import json
import os
import sys
import tempfile

# Locate the script under test relative to this test file.
_SCRIPT_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
_SCRIPT_PATH = os.path.join(_SCRIPT_DIR, "check-vectorized-feature-off.py")

# Import the module by file path (name has hyphens, so importlib.util is required).
import importlib.util as _ilu  # noqa: E402

_spec = _ilu.spec_from_file_location("check_vectorized_feature_off", _SCRIPT_PATH)
_check = _ilu.module_from_spec(_spec)  # type: ignore[arg-type]
_spec.loader.exec_module(_check)  # type: ignore[union-attr]


def _leg1_on_dict(meta: dict) -> int:
    """Run check_leg1 on an in-memory dict via a temp file."""
    with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as tmp:
        json.dump(meta, tmp)
        tmp_path = tmp.name
    try:
        return _check.check_leg1(tmp_path)
    finally:
        os.unlink(tmp_path)


def _leg2_on_dicts(results: list, floor: dict) -> int:
    """Run check_leg2 on in-memory dicts via temp files."""
    with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as r_tmp:
        json.dump(results, r_tmp)
        r_path = r_tmp.name
    with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f_tmp:
        json.dump(floor, f_tmp)
        f_path = f_tmp.name
    try:
        return _check.check_leg2(r_path, f_path)
    finally:
        os.unlink(r_path)
        os.unlink(f_path)


# ---------------------------------------------------------------------------
# Tripwire tests
# ---------------------------------------------------------------------------

def test_leg1_guard_rejects_vectorized_in_resolve() -> bool:
    """
    Tripwire 1a: leg1 MUST reject a metadata fixture where sparq-engine's
    resolved node features include 'vectorized'.
    """
    bad_meta = {
        "packages": [
            {
                "name": "sparq-engine",
                "features": {"default": [], "vectorized": []}
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
        print("  PASS — leg1 guard correctly rejected 'vectorized' in sparq-engine resolved node")
        return True
    else:
        print("  FAIL — leg1 guard did NOT reject 'vectorized'; the guard is broken")
        return False


def test_leg1_guard_rejects_vectorized_in_default_features() -> bool:
    """
    Tripwire 1b: leg1 MUST reject a metadata fixture where sparq-engine lists
    'vectorized' in its package-level default features.
    """
    bad_meta = {
        "packages": [
            {
                "name": "sparq-engine",
                "features": {"default": ["vectorized"], "vectorized": []}
            }
        ],
        "resolve": {
            "nodes": [
                {
                    "id": "sparq-engine 0.1.0 (path+file:///repo/crates/sparq-engine)",
                    "features": []
                }
            ]
        }
    }
    rc = _leg1_on_dict(bad_meta)
    if rc != 0:
        print("  PASS — leg1 guard correctly rejected 'vectorized' in sparq-engine default features")
        return True
    else:
        print("  FAIL — leg1 guard did NOT reject 'vectorized' in default features; the guard is broken")
        return False


def test_leg1_accepts_clean_metadata() -> bool:
    """
    Sanity check: leg1 MUST accept metadata where vectorized is absent from defaults.
    """
    clean_meta = {
        "packages": [
            {
                "name": "sparq-engine",
                "features": {"default": [], "vectorized": []}
            }
        ],
        "resolve": {
            "nodes": [
                {
                    "id": "sparq-engine 0.1.0 (path+file:///repo/crates/sparq-engine)",
                    "features": []
                }
            ]
        }
    }
    rc = _leg1_on_dict(clean_meta)
    if rc == 0:
        print("  PASS — leg1 correctly accepted clean metadata (vectorized absent from defaults)")
        return True
    else:
        print("  FAIL — leg1 incorrectly rejected clean metadata; false positive")
        return False


def test_leg2_comparator_rejects_perturbed_size() -> bool:
    """
    Tripwire 2: leg2 MUST reject a wasm_bundle_bytes value that differs from the
    pinned feature_off_exact by even 1 byte.

    [OPUS-4.8] leg2 now compares against feature_off_exact, NOT floor.  The floor
    (1399703) is the 2%-band ratchet lower bound for bench.yml; the exact equality pin
    is separate so that within-band main drift does not cause a false failure.
    """
    pinned_exact = 1399868  # [OPUS-4.8] CI x86_64 feature_off_exact pin. Re-pin from CI, never aarch64.
    perturbed = pinned_exact + 1
    bad_results = [{"name": "wasm_bundle_bytes", "unit": "bytes", "value": perturbed}]
    bad_floor = {
        "metrics": {
            "wasm_bundle_bytes": {
                "floor": 1399703,
                "threshold": 0.02,
                "mode": "auto",
                "feature_off_exact": pinned_exact,
            }
        }
    }
    rc = _leg2_on_dicts(bad_results, bad_floor)
    if rc != 0:
        print(f"  PASS — leg2 comparator correctly rejected perturbed size "
              f"({perturbed} != feature_off_exact={pinned_exact})")
        return True
    else:
        print("  FAIL — leg2 comparator did NOT reject mismatched size; the comparator is broken")
        return False


def test_leg2_rejects_missing_pin() -> bool:
    """
    [OPUS-4.8] Tripwire 3: leg2 MUST fail-closed when the floor file has no
    'wasm_bundle_bytes' metric at all. The comparison loop only iterates keys present
    in the floor, so a corrupted/edited baseline that drops the pin would otherwise
    yield zero violations and silently pass — disabling the exact-equality gate.
    """
    good_results = [{"name": "wasm_bundle_bytes", "unit": "bytes", "value": 1399868}]
    floor_without_pin = {
        "metrics": {
            # wasm_bundle_bytes intentionally absent; only an unrelated metric present.
            "native_bytes_per_triple": {"floor": 42, "threshold": 0.02, "mode": "auto"},
        }
    }
    rc = _leg2_on_dicts(good_results, floor_without_pin)
    if rc != 0:
        print("  PASS — leg2 fail-closed when the floor file omits wasm_bundle_bytes")
        return True
    else:
        print("  FAIL — leg2 silently passed with no wasm_bundle_bytes pin; the gate is disabled")
        return False


def test_leg2_accepts_exact_match() -> bool:
    """
    Sanity check: leg2 MUST accept results that exactly match the feature_off_exact pin.

    [OPUS-4.8] Uses feature_off_exact, not floor — mirrors the updated check_leg2() logic.
    """
    pinned_exact = 1399868  # [OPUS-4.8] CI x86_64 feature_off_exact pin. Re-pin from CI, never aarch64.
    good_results = [{"name": "wasm_bundle_bytes", "unit": "bytes", "value": pinned_exact}]
    good_floor = {
        "metrics": {
            "wasm_bundle_bytes": {
                "floor": 1399703,
                "threshold": 0.02,
                "mode": "auto",
                "feature_off_exact": pinned_exact,
            }
        }
    }
    rc = _leg2_on_dicts(good_results, good_floor)
    if rc == 0:
        print(f"  PASS — leg2 comparator correctly accepted exact match "
              f"(feature_off_exact={pinned_exact})")
        return True
    else:
        print("  FAIL — leg2 comparator incorrectly rejected an exact match; false positive")
        return False


# ---------------------------------------------------------------------------
# Runner
# ---------------------------------------------------------------------------

def main() -> int:
    tests = [
        ("leg1 rejects vectorized in resolve node", test_leg1_guard_rejects_vectorized_in_resolve),
        ("leg1 rejects vectorized in default features", test_leg1_guard_rejects_vectorized_in_default_features),
        ("leg1 accepts clean metadata (sanity)", test_leg1_accepts_clean_metadata),
        ("leg2 rejects perturbed wasm size (tripwire)", test_leg2_comparator_rejects_perturbed_size),
        ("leg2 fail-closed on missing pin (tripwire)", test_leg2_rejects_missing_pin),
        ("leg2 accepts exact match (sanity)", test_leg2_accepts_exact_match),
    ]

    passed = 0
    failed = 0
    print("=== test_vectorized_feature_off.py ===\n")
    for name, fn in tests:
        print(f"[TEST] {name}")
        ok = fn()
        if ok:
            passed += 1
        else:
            failed += 1
        print()

    print(f"=== Results: {passed} passed, {failed} failed ===")
    if failed:
        print("FAIL")
        return 1
    print("OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
