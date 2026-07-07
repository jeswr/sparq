#!/usr/bin/env python3
"""
[OPUS-4.8] sq-pntvh.8 / [OPUS-4.8] sq-v3nel: standalone test suite for
check-vectorized-feature-off.py.

Covers leg1 (feature-resolution guard) and the RE-DESIGNED leg2 dynamic byte-identity
check (base-tree vs head-tree feature-OFF wasm, declaration-token gated). No external
frameworks — stdlib only.

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


def _leg2_dynamic(base_bytes: bytes, head_bytes: bytes,
                  base_decl: dict | None, head_decl: dict | None) -> int:
    """Run check_leg2_dynamic on in-memory wasm bytes + declaration dicts via temp files.

    A None declaration writes NO file (missing path) to exercise the fail-safe default
    (an absent declaration reads change_token as 0).
    """
    paths: list[str] = []

    def _write_bytes(data: bytes) -> str:
        with tempfile.NamedTemporaryFile(mode="wb", suffix=".wasm", delete=False) as tmp:
            tmp.write(data)
            paths.append(tmp.name)
            return tmp.name

    def _write_decl(data: dict | None) -> str:
        if data is None:
            # Return a path that does not exist -> _read_change_token defaults to 0.
            return os.path.join(tempfile.gettempdir(), "sq-v3nel-nonexistent-decl.json")
        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as tmp:
            json.dump(data, tmp)
            paths.append(tmp.name)
            return tmp.name

    base_wasm = _write_bytes(base_bytes)
    head_wasm = _write_bytes(head_bytes)
    base_decl_p = _write_decl(base_decl)
    head_decl_p = _write_decl(head_decl)
    try:
        return _check.check_leg2_dynamic(base_wasm, head_wasm, base_decl_p, head_decl_p)
    finally:
        for p in paths:
            try:
                os.unlink(p)
            except OSError:
                pass


# ---------------------------------------------------------------------------
# Leg 1 tests (unchanged behaviour)
# ---------------------------------------------------------------------------

def test_leg1_guard_rejects_vectorized_in_resolve() -> bool:
    bad_meta = {
        "packages": [
            {"name": "sparq-engine", "features": {"default": [], "vectorized": []}}
        ],
        "resolve": {
            "nodes": [
                {
                    "id": "sparq-engine 0.1.0 (path+file:///repo/crates/sparq-engine)",
                    "features": ["vectorized"],
                }
            ]
        },
    }
    rc = _leg1_on_dict(bad_meta)
    if rc != 0:
        print("  PASS — leg1 guard correctly rejected 'vectorized' in sparq-engine resolved node")
        return True
    print("  FAIL — leg1 guard did NOT reject 'vectorized'; the guard is broken")
    return False


def test_leg1_guard_rejects_vectorized_in_default_features() -> bool:
    bad_meta = {
        "packages": [
            {"name": "sparq-engine", "features": {"default": ["vectorized"], "vectorized": []}}
        ],
        "resolve": {
            "nodes": [
                {
                    "id": "sparq-engine 0.1.0 (path+file:///repo/crates/sparq-engine)",
                    "features": [],
                }
            ]
        },
    }
    rc = _leg1_on_dict(bad_meta)
    if rc != 0:
        print("  PASS — leg1 guard correctly rejected 'vectorized' in sparq-engine default features")
        return True
    print("  FAIL — leg1 guard did NOT reject 'vectorized' in default features; the guard is broken")
    return False


def test_leg1_accepts_clean_metadata() -> bool:
    clean_meta = {
        "packages": [
            {"name": "sparq-engine", "features": {"default": [], "vectorized": []}}
        ],
        "resolve": {
            "nodes": [
                {
                    "id": "sparq-engine 0.1.0 (path+file:///repo/crates/sparq-engine)",
                    "features": [],
                }
            ]
        },
    }
    rc = _leg1_on_dict(clean_meta)
    if rc == 0:
        print("  PASS — leg1 correctly accepted clean metadata (vectorized absent from defaults)")
        return True
    print("  FAIL — leg1 incorrectly rejected clean metadata; false positive")
    return False


# ---------------------------------------------------------------------------
# Leg 2 (dynamic byte-identity) tests — sq-v3nel
# ---------------------------------------------------------------------------

_BASE = b"\x00asm\x01\x00\x00\x00feature-off-base-bundle"
_HEAD_DIFF = _BASE + b"X"          # different length + content
_HEAD_SAME_LEN = b"\x00asm\x01\x00\x00\x00feature-off-Xead-bundle"  # same length, diff content


def test_leg2_identical_bytes_pass() -> bool:
    """Byte-identical base/head builds => PASS regardless of declaration token."""
    rc = _leg2_dynamic(_BASE, _BASE, {"change_token": 0}, {"change_token": 0})
    if rc == 0:
        print("  PASS — identical feature-OFF bytes accepted (deterministic, no static pin)")
        return True
    print("  FAIL — identical bytes were rejected; false positive")
    return False


def test_leg2_undeclared_change_rejected() -> bool:
    """Bytes differ + token un-bumped => FAIL (the core tripwire)."""
    rc = _leg2_dynamic(_BASE, _HEAD_DIFF, {"change_token": 3}, {"change_token": 3})
    if rc != 0:
        print("  PASS — undeclared feature-OFF byte change rejected (token unchanged)")
        return True
    print("  FAIL — undeclared change was NOT rejected; the gate is disabled")
    return False


def test_leg2_declared_change_accepted() -> bool:
    """Bytes differ + token bumped => PASS (declared intentional change)."""
    rc = _leg2_dynamic(_BASE, _HEAD_DIFF, {"change_token": 3}, {"change_token": 4})
    if rc == 0:
        print("  PASS — declared feature-OFF byte change accepted (token 3 -> 4)")
        return True
    print("  FAIL — declared change was rejected; false positive")
    return False


def test_leg2_same_length_content_change_caught() -> bool:
    """Same length but different content, undeclared => FAIL (byte-for-byte, not size)."""
    assert len(_HEAD_SAME_LEN) == len(_BASE), "fixture must be equal length"
    rc = _leg2_dynamic(_BASE, _HEAD_SAME_LEN, {"change_token": 0}, {"change_token": 0})
    if rc != 0:
        print("  PASS — same-length content change caught (comparison is byte-for-byte)")
        return True
    print("  FAIL — same-length content change slipped through; the comparison is size-only")
    return False


def test_leg2_missing_base_decl_defaults_zero() -> bool:
    """Absent base declaration file reads token 0; a head bump to 1 => declared => PASS.

    Mirrors THIS PR's own situation: base main has no declaration file yet.
    """
    rc = _leg2_dynamic(_BASE, _HEAD_DIFF, None, {"change_token": 1})
    if rc == 0:
        print("  PASS — missing base decl defaults to 0; head token 1 counts as declared")
        return True
    print("  FAIL — missing base decl mishandled; false negative")
    return False


def test_leg2_both_decls_missing_undeclared_rejected() -> bool:
    """Both declaration files absent (both read 0) + bytes differ => FAIL (fail-safe)."""
    rc = _leg2_dynamic(_BASE, _HEAD_DIFF, None, None)
    if rc != 0:
        print("  PASS — both decls absent => tokens 0==0 => undeclared change rejected")
        return True
    print("  FAIL — absent decls silently passed a byte change; fail-safe broken")
    return False


# ---------------------------------------------------------------------------
# Runner
# ---------------------------------------------------------------------------

def main() -> int:
    tests = [
        ("leg1 rejects vectorized in resolve node", test_leg1_guard_rejects_vectorized_in_resolve),
        ("leg1 rejects vectorized in default features", test_leg1_guard_rejects_vectorized_in_default_features),
        ("leg1 accepts clean metadata (sanity)", test_leg1_accepts_clean_metadata),
        ("leg2 identical bytes pass (sanity)", test_leg2_identical_bytes_pass),
        ("leg2 undeclared change rejected (tripwire)", test_leg2_undeclared_change_rejected),
        ("leg2 declared change accepted (sanity)", test_leg2_declared_change_accepted),
        ("leg2 same-length content change caught (byte-for-byte)", test_leg2_same_length_content_change_caught),
        ("leg2 missing base decl defaults to 0", test_leg2_missing_base_decl_defaults_zero),
        ("leg2 both decls absent => undeclared rejected", test_leg2_both_decls_missing_undeclared_rejected),
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
