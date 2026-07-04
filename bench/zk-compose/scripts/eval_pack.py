#!/usr/bin/env python3
# [FABLE-5] sq-gum8.5: zkSPARQL submission-support — deterministic constraint-count
# evaluation pack generator.
#
# Joins the repo's TWO regression-gated, committed constraint-count sources into a
# single per-circuit manifest (`bench/zk-compose/eval_pack.json`) the paper's
# evaluation section can cite as one reproducible artifact:
#
#   1. zk/compose circuit family  — crates/sparq-zk-compose/tests/gate_count_snapshot.json
#      (the in-crate `gate_count_regression` test's baseline; ultra_honk
#      `circuit_size` per compiled member, per the noir-optimisation skill's
#      "bb gates is ground truth" rule), cross-checked against
#      bench/zk-compose/gate_counts_latest.json so the two committed copies can
#      never silently disagree.
#   2. zk/ieee754 float library    — zk/ieee754/bench/float_ops_latest.json +
#      float_conversions_latest.json (small-N/big-N amortised UltraHonk gate
#      measurements per IEEE-754 operation).
#
# What it does NOT do (honesty is load-bearing):
#
#   * It re-measures NOTHING. Every number is read from a committed,
#     regression-gated source; the generator needs no nargo/bb and is therefore
#     deterministic (same committed inputs -> byte-identical output). To refresh
#     the underlying counts after an INTENTIONAL circuit change, re-run
#     bench/zk-compose/scripts/gate_counts.sh (compose) or
#     zk/ieee754/scripts/benchmark_float_ops.py (ieee754) with the pinned
#     toolchain, re-baseline those sources, then re-run this generator.
#   * External systems' figures (ZKLP et al.) are CITED, never re-measured, and
#     carry `cited_not_remeasured: true` plus the exact bibliographic source.
#     Cross-system constraint counts are NOT directly comparable (different
#     proof systems, arithmetisations, and lookup strategies) — the manifest
#     says so rather than implying a comparison.
#   * It asserts no security/privacy/soundness property. The circuits measured
#     here are pre-external-audit (sq-qhy4 open); a gate count is a size fact,
#     not a soundness claim.
#
# Drift guards:
#
#   * TOOLCHAIN: the nargo/bb versions recorded inside each measured source must
#     match the pins in .github/workflows/zk-toolchain.yml (NARGO_VERSION /
#     BB_VERSION). A toolchain bump without a re-measure fails generation.
#   * FRESHNESS CHAIN: gate_counts_latest.json must agree member-for-member with
#     the regression-gated snapshot. A re-baseline that updates one but not the
#     other fails generation.
#   * BYTE-IDENTITY (--check): regenerates the manifest in memory and fails if
#     it differs from the committed bench/zk-compose/eval_pack.json — so a
#     source-count change that forgets to regenerate the pack is a visible
#     failure, not silent staleness. Run it after any zk/ re-baseline.
#
# Usage:
#   bench/zk-compose/scripts/eval_pack.py > bench/zk-compose/eval_pack.json
#   bench/zk-compose/scripts/eval_pack.py --check   # verify committed manifest
#
# CI wiring for --check is tracked as follow-up bead sq-yfhho (workflow files
# are outside this bead's file set); until then it is a documented manual step
# of the re-baseline procedure above and in bench/zk-compose/README.md.

from __future__ import annotations

import hashlib
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.normpath(os.path.join(HERE, "..", "..", ".."))

SNAPSHOT_JSON = os.path.join(ROOT, "crates", "sparq-zk-compose", "tests", "gate_count_snapshot.json")
LATEST_JSON = os.path.join(ROOT, "bench", "zk-compose", "gate_counts_latest.json")
FLOAT_OPS_JSON = os.path.join(ROOT, "zk", "ieee754", "bench", "float_ops_latest.json")
FLOAT_CONV_JSON = os.path.join(ROOT, "zk", "ieee754", "bench", "float_conversions_latest.json")
TOOLCHAIN_YML = os.path.join(ROOT, ".github", "workflows", "zk-toolchain.yml")
OUT_JSON = os.path.join(ROOT, "bench", "zk-compose", "eval_pack.json")

# Hashed inputs = the four MEASURED sources only. The toolchain workflow is a
# guard input (its pins are cross-checked + recorded below) but is deliberately
# NOT hashed: hashing it would make the manifest churn on unrelated workflow
# edits (comments, unrelated steps) even though no count changed.
INPUT_PATHS = {
    "gate_count_snapshot": SNAPSHOT_JSON,
    "gate_counts_latest": LATEST_JSON,
    "ieee754_float_ops": FLOAT_OPS_JSON,
    "ieee754_float_conversions": FLOAT_CONV_JSON,
}


def die(msg: str) -> None:
    print(f"eval_pack.py: ERROR: {msg}", file=sys.stderr)
    sys.exit(2)


def load_json(path: str):
    try:
        with open(path, encoding="utf-8") as f:
            return json.load(f)
    except (OSError, json.JSONDecodeError) as e:
        die(f"cannot read {os.path.relpath(path, ROOT)}: {e}")


def sha256_file(path: str) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


def pinned_toolchain() -> dict:
    """Parse NARGO_VERSION / BB_VERSION pins out of zk-toolchain.yml."""
    try:
        with open(TOOLCHAIN_YML, encoding="utf-8") as f:
            text = f.read()
    except OSError as e:
        die(f"cannot read {os.path.relpath(TOOLCHAIN_YML, ROOT)}: {e}")
    pins = {}
    for key in ("NARGO_VERSION", "BB_VERSION"):
        m = re.search(rf'^\s*{key}:\s*"([^"]+)"', text, re.MULTILINE)
        if not m:
            die(f"pin {key} not found in zk-toolchain.yml — the parse regex or the workflow moved")
        pins[key] = m.group(1)
    return {
        "nargo": pins["NARGO_VERSION"],
        "bb": pins["BB_VERSION"],
        "pinned_in": ".github/workflows/zk-toolchain.yml",
    }


def compose_family(member: str) -> str:
    for prefix, family in (
        ("scan_", "scan"),
        ("join_eq_", "join_eq"),
        ("filter_value_dl_", "filter_value_dual_leaf"),
        ("filter_", "filter"),
        ("holder_", "holder"),
        ("hidden_issuer", "hidden_issuer"),
        ("revoke_", "revocation"),
    ):
        if member.startswith(prefix):
            return family
    return "other"


def build_manifest() -> dict:
    toolchain = pinned_toolchain()
    snapshot = load_json(SNAPSHOT_JSON)
    latest = load_json(LATEST_JSON)
    float_ops = load_json(FLOAT_OPS_JSON)
    float_conv = load_json(FLOAT_CONV_JSON)

    # --- toolchain drift guard -------------------------------------------------
    lat_bb = latest.get("bb_version", "")
    lat_nargo = latest.get("nargo_version", "")
    if toolchain["bb"] != lat_bb:
        die(
            f"toolchain drift: zk-toolchain.yml pins bb {toolchain['bb']} but "
            f"gate_counts_latest.json was measured with bb {lat_bb!r} — re-run "
            "bench/zk-compose/scripts/gate_counts.sh with the pinned toolchain and re-baseline"
        )
    if toolchain["nargo"] not in lat_nargo:
        die(
            f"toolchain drift: zk-toolchain.yml pins nargo {toolchain['nargo']} but "
            f"gate_counts_latest.json records {lat_nargo!r} — re-run "
            "bench/zk-compose/scripts/gate_counts.sh with the pinned toolchain and re-baseline"
        )
    snap_bb = snapshot.get("bb_version_baselined", "")
    snap_nargo = snapshot.get("nargo_version_baselined", "")
    if toolchain["bb"] != snap_bb or toolchain["nargo"] != snap_nargo:
        die(
            "toolchain drift: gate_count_snapshot.json was baselined with "
            f"nargo {snap_nargo!r} / bb {snap_bb!r}, which differs from the "
            f"zk-toolchain.yml pins (nargo {toolchain['nargo']} / bb {toolchain['bb']})"
        )

    # --- freshness-chain guard: latest must equal the regression-gated snapshot -
    snap_members = snapshot.get("members", {})
    latest_members = {
        name: row.get("circuit_size") for name, row in latest.get("benchmarks", {}).items()
    }
    if snap_members != latest_members:
        only_snap = sorted(set(snap_members) - set(latest_members))
        only_latest = sorted(set(latest_members) - set(snap_members))
        diff_vals = sorted(
            m
            for m in set(snap_members) & set(latest_members)
            if snap_members[m] != latest_members[m]
        )
        die(
            "freshness chain broken: gate_count_snapshot.json and gate_counts_latest.json "
            f"disagree (snapshot-only: {only_snap}; latest-only: {only_latest}; "
            f"differing values: {diff_vals}) — re-run gate_counts.sh and re-baseline BOTH"
        )

    zk_compose = {
        member: {
            "circuit_size": size,
            "family": compose_family(member),
        }
        for member, size in sorted(snap_members.items())
    }

    def op_rows(measured: dict) -> dict:
        rows = {}
        for op, row in sorted(measured.get("benchmarks", {}).items()):
            rows[op] = {
                "small": row.get("small"),
                "big": row.get("big"),
                "per_call_estimate": row.get("per_call_estimate"),
                "setup_estimate": row.get("setup_estimate"),
            }
        return rows

    manifest = {
        "_schema": "sparq-zk-eval-pack/1",
        "_provenance": {
            "generator": "bench/zk-compose/scripts/eval_pack.py [FABLE-5] (sq-gum8.5)",
            "note": (
                "Deterministic join of the repo's committed, regression-gated constraint-count "
                "sources; regenerate with the generator, never hand-edit. No number here is "
                "re-measured by this script. Counts are ultra_honk circuit_size per `bb gates "
                "-s ultra_honk` (backend gate count after ACIR->backend transformation). "
                "The circuits are pre-external-audit (sq-qhy4): a gate count is a size fact, "
                "not a soundness or privacy claim."
            ),
            "inputs_sha256": {
                name: {
                    "path": os.path.relpath(path, ROOT),
                    "sha256": sha256_file(path),
                }
                for name, path in sorted(INPUT_PATHS.items())
            },
        },
        "toolchain": toolchain,
        "zk_compose": {
            "measured_with": {
                "tool": latest.get("tool"),
                "source_timestamp": latest.get("timestamp"),
                "regression_gate": "crates/sparq-zk-compose/tests/gate_count.rs",
            },
            "members": zk_compose,
        },
        "ieee754": {
            "measured_with": {
                "tool": "bb gates -s ultra_honk via zk/ieee754/scripts/benchmark_float_ops.py",
                "method": (
                    "small-N/big-N amortisation: per_call_estimate = "
                    "(big.circuit_size - small.circuit_size) / (big.calls - small.calls); "
                    "estimates are derived, circuit_size values are measured"
                ),
                "ops_source_timestamp": float_ops.get("timestamp"),
                "conversions_source_timestamp": float_conv.get("timestamp"),
                "n_small": float_ops.get("n_small"),
                "n_big": float_ops.get("n_big"),
            },
            "float_ops": op_rows(float_ops),
            "float_conversions": op_rows(float_conv),
        },
        "cited_external": {
            "_note": (
                "Figures REPORTED BY THE CITED AUTHORS, reproduced for context only — never "
                "re-measured here, and NOT directly comparable to the counts above (different "
                "proof systems, arithmetisations, constraint/gate definitions, and lookup "
                "strategies). Comparisons in prose must stay qualitative."
            ),
            "zklp": {
                "cited_not_remeasured": True,
                "citation": (
                    "Ernstberger, J.; Zhang, C.; Ciprian, L.; Jovanovic, P.; Steinhorst, S. "
                    "'Zero-Knowledge Location Privacy via Accurate Floating-Point SNARKs'. "
                    "IEEE S&P 2025, pp. 3440-3459. arXiv:2404.14983; IACR ePrint 2024/1842."
                ),
                "constraint_model": (
                    "R1CS constraints in gnark over BN254 (Groth16/Plonk) with LogUp lookups; "
                    "the authors report a 'native' count and a 'lookup (i)' count per op, plus "
                    "a one-time lookup-table cost. NOT the same unit as the UltraHonk "
                    "circuit_size figures above — do not tabulate side by side as if comparable."
                ),
                "reported_figures_source": "ZKLP Table 1 (|T_RC|=2^8), author-reported",
                "reported_r1cs_constraints": {
                    "fp32_native": {"init": 13, "add_sub": 42, "mul": 31, "div": 38,
                                    "sqrt": 23, "cmp": 26},
                    "fp32_lookup_i": {"init": 17, "add_sub": 43, "mul": 33, "div": 38,
                                      "sqrt": 22, "cmp": 7, "one_time_ii_iii": 291},
                    "fp64_native": {"init": 13, "add_sub": 42, "mul": 31, "div": 38,
                                    "sqrt": 23, "cmp": 26},
                    "fp64_lookup_i": {"init": 32, "add_sub": 71, "mul": 57, "div": 60,
                                      "sqrt": 38, "cmp": 11, "one_time_ii_iii": 323},
                },
                "reported_amortised_note": (
                    "ZKLP abstract/section 5.1: about 64 R1CS constraints per operation for "
                    "2^15 FP32 multiplications (amortised); 209 for 2^1 multiplications."
                ),
                "first_claim_guard": (
                    "ZKLP is peer-reviewed and prior, claiming the first fully-IEEE-754 "
                    "compliant ZKP circuits. The zkSPARQL submission MUST NOT assert a 'first "
                    "ZK floats' novelty; its float contribution is the INTEGRATION of "
                    "in-circuit IEEE-754 into typed SPARQL FILTER evaluation, not the "
                    "primitives themselves."
                ),
            },
        },
    }
    return manifest


def render(manifest: dict) -> str:
    return json.dumps(manifest, indent=2, sort_keys=True) + "\n"


def main() -> None:
    check = "--check" in sys.argv[1:]
    manifest = build_manifest()
    out = render(manifest)
    if check:
        try:
            with open(OUT_JSON, encoding="utf-8") as f:
                committed = f.read()
        except OSError as e:
            die(f"--check: cannot read committed manifest {os.path.relpath(OUT_JSON, ROOT)}: {e}")
        if committed != out:
            die(
                "--check: committed eval_pack.json is STALE (differs from a fresh regeneration). "
                "A constraint-count source changed without regenerating the pack — run "
                "bench/zk-compose/scripts/eval_pack.py > bench/zk-compose/eval_pack.json "
                "and commit the result"
            )
        print("eval_pack.py --check: OK (committed manifest is byte-identical to regeneration)")
        return
    sys.stdout.write(out)


if __name__ == "__main__":
    main()
