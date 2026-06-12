#!/usr/bin/env python3
"""Amortised gate-count benchmarks for generated float operations.

Each benchmark creates a temporary binary package depending on this library,
compiles it with nargo, and measures the compiled circuit with `bb gates`.
Two call counts are measured so the script can estimate per-call cost while
reducing the effect of the one-time UltraHonk padding/setup floor.
"""

from __future__ import annotations

import argparse
import json
import shutil
import sys
import tempfile
from datetime import datetime
from pathlib import Path

from benchmark_utils import measure_package_circuit_size


ROOT = Path(__file__).resolve().parent.parent

FORMATS = {
    "f16": {"type": "f16", "uint": "u16", "a": "0x3c00", "b": "0x4000"},
    "f32": {"type": "f32", "uint": "u32", "a": "0x3f800000", "b": "0x40000000"},
    "f64": {"type": "f64", "uint": "u64", "a": "0x3ff0000000000000", "b": "0x4000000000000000"},
    "f128": {
        "type": "f128",
        "uint": "u128",
        "a": "0x3fff0000000000000000000000000000",
        "b": "0x40000000000000000000000000000000",
    },
}

OPS = {
    "add": "+",
    "sub": "-",
    "mul": "*",
    "div": "/",
}

# Boolean predicates, unary methods, and casts are measured with the
# conversion-harness pattern: a private array of input witnesses keeps every
# call live (no CSE) and the results are XOR-folded. Per-call estimates for
# these therefore include one `new()` decode per call.
PREDICATE_OPS = ["eq", "ne", "lt", "le", "gt", "ge"]
UNARY_OPS = ["floor", "ceil", "trunc", "round_ties_even", "sqrt"]
CAST_OPS = ["to_u64", "to_i64"]
METHOD_OPS = PREDICATE_OPS + UNARY_OPS + CAST_OPS


def benchmark_names() -> list[str]:
    return [f"{op}_{fmt}" for fmt in FORMATS for op in list(OPS) + METHOD_OPS]


def render_main(fmt_name: str, op: str, calls: int) -> str:
    spec = FORMATS[fmt_name]
    float_type = spec["type"]
    uint_type = spec["uint"]

    if op in OPS:
        symbol = OPS[op]
        body = [
            f"let a: {float_type} = {float_type}::new(a_bits);",
            f"let b: {float_type} = {float_type}::new(b_bits);",
            f"let mut acc: {float_type} = a;",
        ]

        for _index in range(calls):
            body.append(f"acc = acc {symbol} b;")

        body.append("acc.bits()")
        rendered_body = "\n    ".join(body)

        return f"""use sparq_ieee754::{{f128, f16, f32, f64}};

fn main(a_bits: pub {uint_type}, b_bits: pub {uint_type}) -> pub {uint_type} {{
    {rendered_body}
}}
"""

    if op in PREDICATE_OPS:
        return f"""use sparq_ieee754::{{f128, f16, f32, f64}};

fn main(a_bits: [{uint_type}; {calls}], b_bits: pub {uint_type}) -> pub bool {{
    let b: {float_type} = {float_type}::new(b_bits);
    let mut acc: bool = false;

    for index in 0..{calls} {{
        acc = acc ^ {float_type}::new(a_bits[index]).{op}(b);
    }}

    acc
}}
"""

    if op in UNARY_OPS:
        return f"""use sparq_ieee754::{{f128, f16, f32, f64}};

fn main(a_bits: [{uint_type}; {calls}]) -> pub {uint_type} {{
    let mut acc: {uint_type} = 0;

    for index in 0..{calls} {{
        acc = acc ^ {float_type}::new(a_bits[index]).{op}().bits();
    }}

    acc
}}
"""

    if op in CAST_OPS:
        cast_suffix = " as u64" if op == "to_i64" else ""

        return f"""use sparq_ieee754::{{f128, f16, f32, f64}};

fn main(a_bits: [{uint_type}; {calls}]) -> pub u64 {{
    let mut acc: u64 = 0;

    for index in 0..{calls} {{
        acc = acc ^ ({float_type}::new(a_bits[index]).{op}(){cast_suffix});
    }}

    acc
}}
"""

    raise ValueError(f"unknown op: {op}")


def write_package(root: Path, name: str, fmt_name: str, op: str, calls: int) -> Path:
    package = root / name
    (package / "src").mkdir(parents=True)
    (package / "Nargo.toml").write_text(
        f"""[package]
name = "{name}"
type = "bin"
authors = ["bench"]

[dependencies]
sparq_ieee754 = {{ path = "{ROOT}" }}
"""
    )
    (package / "src" / "main.nr").write_text(render_main(fmt_name, op, calls))
    return package


def build_and_measure(name: str, fmt_name: str, op: str, calls: int, include_breakdown: bool) -> dict[str, object]:
    with tempfile.TemporaryDirectory(prefix=f"float_bench_{name}_{calls}_") as tmp:
        temp_root = Path(tmp)
        package_name = f"bench_{name}_{calls}"
        package = write_package(temp_root, package_name, fmt_name, op, calls)
        artifact = package / "target" / f"{package_name}.json"

        return {
            "calls": calls,
            "circuit_size": measure_package_circuit_size(
                package,
                artifact,
                name,
                calls,
                include_breakdown=include_breakdown,
                include_bbup_hint=True,
            ),
        }


def parse_name(name: str) -> tuple[str, str]:
    op, fmt = name.rsplit("_", 1)
    if (op not in OPS and op not in METHOD_OPS) or fmt not in FORMATS:
        raise ValueError(f"unknown benchmark: {name}")
    return fmt, op


def main() -> int:
    parser = argparse.ArgumentParser(description="Benchmark generated float operation gates")
    parser.add_argument("--ops", nargs="+", default=benchmark_names(), help="Benchmarks like add_f16 div_f64")
    parser.add_argument("--n-small", type=int, default=1)
    parser.add_argument("--n-big", type=int, default=8)
    parser.add_argument("--output", type=Path, default=ROOT / "bench" / "float_ops_latest.json")
    parser.add_argument("--include-breakdown", action="store_true")
    args = parser.parse_args()

    if shutil.which("bb") is None:
        print("bb is required for gate benchmarks; install with bbup", file=sys.stderr)
        return 1

    results: dict[str, object] = {
        "timestamp": datetime.now().isoformat(),
        "n_small": args.n_small,
        "n_big": args.n_big,
        "benchmarks": {},
    }

    print(f"{'benchmark':<22} {'gates@small':>12} {'gates@big':>12} {'per-call':>12} {'setup':>12}")
    print("-" * 74)

    for name in args.ops:
        fmt_name, op = parse_name(name)
        small = build_and_measure(name, fmt_name, op, args.n_small, args.include_breakdown)
        big = build_and_measure(name, fmt_name, op, args.n_big, args.include_breakdown)
        per_call = (big["circuit_size"] - small["circuit_size"]) / (args.n_big - args.n_small)
        setup = small["circuit_size"] - (args.n_small * per_call)

        results["benchmarks"][name] = {
            "small": small,
            "big": big,
            "per_call_estimate": per_call,
            "setup_estimate": setup,
        }

        print(f"{name:<22} {small['circuit_size']:>12} {big['circuit_size']:>12} {per_call:>12.1f} {setup:>12.1f}")

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(results, indent=2) + "\n")
    print(f"\nwrote {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())