#!/usr/bin/env python3
"""Amortised gate-count benchmarks for generated integer-to-float conversions.

Each benchmark creates a temporary binary package depending on this library,
compiles it with nargo, and measures the compiled circuit with `bb gates`.
Two call counts are measured so the script can estimate per-conversion cost
while reducing the effect of the one-time UltraHonk padding/setup floor.
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
    "f16": {"type": "f16", "uint": "u16"},
    "f32": {"type": "f32", "uint": "u32"},
    "f64": {"type": "f64", "uint": "u64"},
    "f128": {"type": "f128", "uint": "u128"},
}

INPUT_TYPES = {
    "u8": "u8",
    "u16": "u16",
    "u32": "u32",
    "u64": "u64",
    "u128": "u128",
    "i8": "i8",
    "i16": "i16",
    "i32": "i32",
    "i64": "i64",
}


def benchmark_names() -> list[str]:
    return [f"{input_name}_to_{fmt_name}" for fmt_name in FORMATS for input_name in INPUT_TYPES]


def render_main(fmt_name: str, input_name: str, calls: int) -> str:
    spec = FORMATS[fmt_name]
    float_type = spec["type"]
    uint_type = spec["uint"]
    input_type = INPUT_TYPES[input_name]

    return f"""use sparq_ieee754::{{f128, f16, f32, f64}};

fn main(inputs: [{input_type}; {calls}]) -> pub {uint_type} {{
    let mut acc: {uint_type} = 0 as {uint_type};

    for index in 0..{calls} {{
        let converted: {float_type} = {float_type}::from(inputs[index]);
        acc = acc ^ converted.bits();
    }}

    acc
}}
"""


def write_package(root: Path, name: str, fmt_name: str, input_name: str, calls: int) -> Path:
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
    (package / "src" / "main.nr").write_text(render_main(fmt_name, input_name, calls))
    return package


def build_and_measure(name: str, fmt_name: str, input_name: str, calls: int, include_breakdown: bool) -> dict[str, object]:
    with tempfile.TemporaryDirectory(prefix=f"float_conversion_{name}_{calls}_") as tmp:
        temp_root = Path(tmp)
        package_name = f"bench_{name}_{calls}"
        package = write_package(temp_root, package_name, fmt_name, input_name, calls)
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
    if "_to_" not in name:
        raise ValueError(f"unknown benchmark: {name}")
    input_name, fmt_name = name.split("_to_", 1)
    if input_name not in INPUT_TYPES or fmt_name not in FORMATS:
        raise ValueError(f"unknown benchmark: {name}")
    return fmt_name, input_name


def main() -> int:
    parser = argparse.ArgumentParser(description="Benchmark generated integer-to-float conversion gates")
    parser.add_argument("--conversions", nargs="+", default=benchmark_names(), help="Benchmarks like u32_to_f16 i64_to_f64")
    parser.add_argument("--n-small", type=int, default=1)
    parser.add_argument("--n-big", type=int, default=8)
    parser.add_argument("--output", type=Path, default=ROOT / "bench" / "float_conversions_latest.json")
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

    print(f"{'benchmark':<16} {'gates@small':>12} {'gates@big':>12} {'per-call':>12} {'setup':>12}")
    print("-" * 68)

    for name in args.conversions:
        fmt_name, input_name = parse_name(name)
        small = build_and_measure(name, fmt_name, input_name, args.n_small, args.include_breakdown)
        big = build_and_measure(name, fmt_name, input_name, args.n_big, args.include_breakdown)
        per_call = (big["circuit_size"] - small["circuit_size"]) / (args.n_big - args.n_small)
        setup = small["circuit_size"] - (args.n_small * per_call)

        results["benchmarks"][name] = {
            "input_type": input_name,
            "float_type": fmt_name,
            "small": small,
            "big": big,
            "per_call_estimate": per_call,
            "setup_estimate": setup,
        }

        print(f"{name:<16} {small['circuit_size']:>12} {big['circuit_size']:>12} {per_call:>12.1f} {setup:>12.1f}")

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(results, indent=2) + "\n")
    print(f"\nwrote {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())