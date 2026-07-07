#!/usr/bin/env python3
"""
[OPUS-4.8] sq-pntvh.8 / [OPUS-4.8] sq-v3nel: feature-OFF CI proof for M4 vectorization.

Proves compile-time absence (leg1, leg3) and artifact byte-DETERMINISM (leg2) of
the vectorized-OFF build. Does NOT prove runtime performance neutrality — that is
EC2-gated. This is the strongest claim deterministic ratchet infrastructure can
support.

Three legs:
  --leg1 <metadata-json>   cargo metadata check: `vectorized` absent from default features
  --leg2-dynamic <base.wasm> <head.wasm> <base-decl.json> <head-decl.json>
                           DYNAMIC byte-identity: compare the feature-OFF wasm built
                           from the BASE (target-branch/merge-base) tree against the one
                           built from the HEAD (PR/merged) tree IN THE SAME CI RUN. No
                           static byte pin => no merge-order dependence. Identical bytes
                           => pass. Differing bytes => pass only if DECLARED (change_token
                           bumped in bench/feature-off-declaration.json); else fail.
  --leg3                   cfg-audit: every vectorized call site in lib.rs/exec.rs is gated
  --self-test              run built-in tripwires that MUST fail (guard the guards)

[OPUS-4.8] sq-v3nel (2026-07-07): leg 2 was RE-DESIGNED from a static exact-equality
pin (bench/perf-baseline.json metrics.wasm_bundle_bytes.feature_off_exact) to this
DYNAMIC same-run base-vs-head byte comparison. The static pin was ORDER-DEPENDENT: any
PR touching always-compiled engine/core code changed the merged-tree bundle bytes, the
correct pin value depended on merge ORDER, and armed PRs cycled BLOCKED while serial
re-pin commits chased a moving target (see the retirement _comment in perf-baseline.json).
The dynamic check has no static pin: a PR that does not touch always-compiled code yields
byte-identical builds and passes deterministically; a PR that does must DECLARE the change
by bumping change_token (audit-visible, reviewed diff). The SIZE of an accepted change is
still governed, unchanged, by the wasm_bundle_bytes floor ratchet (+/-2% band) in bench.yml.

Declaration mechanism (least-gameable + audit-visible, chosen over a PR-body line):
  bench/feature-off-declaration.json carries an integer "change_token". A PR that
  intentionally changes always-compiled feature-OFF bytes must INCREMENT that token in a
  reviewed diff (and add a dated rationale line to its _comment). A PR-body marker was
  REJECTED: it is invisible in merge_group events (no PR body on the merged commit set)
  and is not part of the reviewed diff. The token lives in its OWN file, NOT inside
  perf-baseline.json, because scripts/perf-gate.py's auto-ratchet-down rewrite
  (write_baseline) reconstructs each metric with only floor/threshold/mode and would
  silently wipe any extra field on the next push-to-main improvement commit.
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
# Leg 2: DYNAMIC byte-identity — feature-OFF wasm, base tree vs head tree (sq-v3nel)
# ---------------------------------------------------------------------------

def _read_change_token(decl_path: str) -> int:
    """Read the integer 'change_token' from a feature-off declaration JSON file.

    Fail-SAFE default: a missing file, missing field, or non-integer value reads as 0.
    Because an ABSENT declaration reads the same as an UN-bumped one, an undeclared byte
    change still fails the gate (the check compares base vs head token for INEQUALITY).
    """
    try:
        with open(decl_path) as fh:
            data = json.load(fh)
    except (OSError, ValueError):
        return 0
    tok = data.get("change_token", 0) if isinstance(data, dict) else 0
    try:
        return int(tok)
    except (TypeError, ValueError):
        return 0


def check_leg2_dynamic(base_wasm: str, head_wasm: str,
                       base_decl: str, head_decl: str) -> int:
    """DYNAMIC byte-identity gate for the feature-OFF wasm bundle (sq-v3nel).

    Compares the feature-OFF wasm built from the BASE (target-branch / merge-base) tree
    against the one built from the HEAD (PR / merged) tree IN THE SAME CI RUN. Same
    toolchain + same runner in one run => a deterministic comparison with NO static byte
    pin, so NO merge-order dependence.

    Policy:
      * bytes byte-for-byte IDENTICAL  -> PASS. The PR touched no always-compiled code
        that reaches the feature-OFF bundle. (This is the common case for non-engine PRs.)
      * bytes DIFFER + change DECLARED -> PASS. 'declared' == change_token differs between
        the base-tree and head-tree bench/feature-off-declaration.json (author bumped it).
        The SIZE of the change is governed SEPARATELY, unchanged, by the wasm_bundle_bytes
        floor ratchet (+/-2% band) in bench.yml — this gate governs INTENT, not magnitude.
      * bytes DIFFER + NOT declared    -> FAIL. Either accidental default-path/vectorized
        code leaked into the feature-OFF build (remove / cfg-gate it) or an intentional
        always-compiled change was not declared (bump change_token).

    Returns 0 on pass, 1 on fail/error.
    """
    try:
        with open(base_wasm, "rb") as fh:
            base_bytes = fh.read()
    except OSError as exc:
        print(f"[leg2] ERROR: cannot read base wasm {base_wasm!r}: {exc}", file=sys.stderr)
        return 1
    try:
        with open(head_wasm, "rb") as fh:
            head_bytes = fh.read()
    except OSError as exc:
        print(f"[leg2] ERROR: cannot read head wasm {head_wasm!r}: {exc}", file=sys.stderr)
        return 1

    base_len, head_len = len(base_bytes), len(head_bytes)

    if base_bytes == head_bytes:
        print(
            f"[leg2] OK — feature-OFF wasm is byte-for-byte IDENTICAL to the base tree "
            f"({head_len} bytes). No always-compiled code changed the default build; "
            "deterministic same-run comparison, no static pin, no merge-order dependence."
        )
        return 0

    # Bytes differ (size and/or content). Require an audit-visible declaration.
    delta = head_len - base_len
    pct = (delta / base_len * 100) if base_len else float("inf")
    base_tok = _read_change_token(base_decl)
    head_tok = _read_change_token(head_decl)

    print(
        f"[leg2] feature-OFF wasm DIFFERS from the base tree: "
        f"base={base_len} head={head_len} bytes, size delta={delta:+d} ({pct:+.3f}%). "
        "Comparison is byte-for-byte, so same-length content changes are also detected."
    )

    if head_tok != base_tok:
        print(
            f"[leg2] OK — change DECLARED: bench/feature-off-declaration.json change_token "
            f"{base_tok} -> {head_tok}. The feature-OFF byte change is intentional. Its SIZE "
            "is governed by the wasm_bundle_bytes floor ratchet (+/-2% band) in bench.yml, "
            "which is unchanged by this gate."
        )
        return 0

    print(
        f"[leg2] VIOLATION: the feature-OFF wasm bundle changed vs the base tree "
        f"(size delta {delta:+d} bytes, {pct:+.3f}%) but the change is NOT declared "
        f"(change_token unchanged at {base_tok} in bench/feature-off-declaration.json).\n"
        "  - If this is ACCIDENTAL vectorized / default-path code leaking into the "
        "feature-OFF build, REMOVE it or gate it behind #[cfg(feature = \"vectorized\")].\n"
        "  - If this is an INTENTIONAL change to always-compiled engine/core code, DECLARE "
        "it: increment \"change_token\" in bench/feature-off-declaration.json (add a dated "
        "rationale line to its _comment). If the SIZE moves > +/-2%, ALSO raise "
        "metrics.wasm_bundle_bytes.floor in bench/perf-baseline.json (the bench.yml ratchet)."
    )
    print("\n[leg2] FAIL — undeclared feature-OFF bundle change. Declare it or remove it.")
    return 1


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


def _leg2_dynamic_on_bytes(base_bytes: bytes, head_bytes: bytes,
                           base_tok: int | None, head_tok: int | None) -> int:
    """Run the dynamic leg2 check on in-memory wasm bytes + declaration tokens.

    A token of None writes an EMPTY declaration file (missing field) to exercise the
    fail-safe default (reads as 0). Otherwise writes {"change_token": <int>}.
    """
    paths: list[str] = []

    def _write(data: bytes | str, suffix: str) -> str:
        mode = "wb" if isinstance(data, bytes) else "w"
        with tempfile.NamedTemporaryFile(mode=mode, suffix=suffix, delete=False) as tmp:
            tmp.write(data)
            paths.append(tmp.name)
            return tmp.name

    base_wasm = _write(base_bytes, ".wasm")
    head_wasm = _write(head_bytes, ".wasm")
    base_decl = _write("{}" if base_tok is None else json.dumps({"change_token": base_tok}), ".json")
    head_decl = _write("{}" if head_tok is None else json.dumps({"change_token": head_tok}), ".json")
    try:
        return check_leg2_dynamic(base_wasm, head_wasm, base_decl, head_decl)
    finally:
        for p in paths:
            os.unlink(p)


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

    # ----- Tripwire 2: Leg 2 (dynamic) must reject an UNDECLARED byte change -----
    # [OPUS-4.8] sq-v3nel: the dynamic check compares base-tree vs head-tree feature-OFF
    # wasm bytes in the SAME run. Differing bytes with an un-bumped change_token MUST fail.
    rc = _leg2_dynamic_on_bytes(b"\x00wasm-base", b"\x00wasm-headX", base_tok=0, head_tok=0)
    if rc != 0:
        print("TRIPWIRE 2 (leg2): PASS — dynamic check correctly rejected an UNDECLARED "
              "feature-OFF byte change (bytes differ, change_token un-bumped)")
    else:
        print("TRIPWIRE 2 (leg2): FAIL — dynamic check did NOT reject an undeclared change; "
              "the leg2 check is broken")
        all_passed = False

    # ----- Tripwire 3: Leg 2 (dynamic) must ACCEPT a declared byte change (guard sanity) -----
    rc = _leg2_dynamic_on_bytes(b"\x00wasm-base", b"\x00wasm-headX", base_tok=0, head_tok=1)
    if rc == 0:
        print("TRIPWIRE 3 (leg2): PASS — dynamic check accepted a DECLARED change "
              "(bytes differ, change_token 0 -> 1)")
    else:
        print("TRIPWIRE 3 (leg2): FAIL — dynamic check rejected a declared change; false positive")
        all_passed = False

    if all_passed:
        print("\n[self-test] OK — all tripwires fired correctly. The guards can fail (and "
              "the dynamic leg2 accepts a declared change).")
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
    mode.add_argument("--leg2-dynamic", dest="leg2_dynamic", nargs=4,
                      metavar=("BASE_WASM", "HEAD_WASM", "BASE_DECL", "HEAD_DECL"),
                      help="DYNAMIC byte-identity: base-tree vs head-tree feature-OFF wasm "
                           "+ base/head feature-off-declaration.json (no static pin)")
    mode.add_argument("--leg3", action="store_true",
                      help="cfg-audit of vectorized call sites in lib.rs and exec.rs")
    mode.add_argument("--self-test", dest="self_test", action="store_true",
                      help="run built-in tripwires (both must exit non-zero to pass)")
    parser.add_argument("--repo-root", default=".",
                        help="repo root for --leg3 (default: cwd)")
    args = parser.parse_args()

    if args.leg1:
        return check_leg1(args.leg1)
    elif args.leg2_dynamic:
        return check_leg2_dynamic(args.leg2_dynamic[0], args.leg2_dynamic[1],
                                  args.leg2_dynamic[2], args.leg2_dynamic[3])
    elif args.leg3:
        return check_leg3(repo_root=args.repo_root)
    elif args.self_test:
        return run_self_test()
    else:
        parser.print_help()
        return 2


if __name__ == "__main__":
    sys.exit(main())
