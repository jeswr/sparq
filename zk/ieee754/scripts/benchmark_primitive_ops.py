#!/usr/bin/env python3
"""Amortised gate-count probes for Noir primitive operations.

Each benchmark creates a tiny standalone Noir binary, compiles it, and measures
it with `bb gates -s ultra_honk`. The reported per-iteration estimate is the
amortised cost of repeating the operation inside a witness-dependent loop.
"""

from __future__ import annotations

import argparse
import json
import shutil
import tempfile
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path

from benchmark_utils import measure_package_circuit_size

ROOT = Path(__file__).resolve().parent.parent


@dataclass(frozen=True)
class PrimitiveType:
    name: str
    mask: str | None = None


@dataclass(frozen=True)
class Operation:
    name: str
    body_template: str
    supports_field: bool = True
    supports_int: bool = True


TYPES = [
    PrimitiveType("Field"),
    PrimitiveType("u128", "127"),
    PrimitiveType("u64", "63"),
    PrimitiveType("u32", "31"),
    PrimitiveType("u16", "15"),
]

OPERATIONS = [
    Operation("add", "acc = acc + y;"),
    Operation("sub", "acc = acc - y;"),
    Operation("mul", "acc = acc * y;"),
    Operation("div", "acc = acc / y;"),
    Operation("rem", "acc = acc % y;", supports_field=False),
    Operation("eq_select", "acc = if acc == y { acc + 1 } else { acc + 2 };"),
    Operation("lt_select", "acc = if acc < y { acc + 1 } else { acc + 2 };", supports_field=False),
    Operation("bit_and_mix", "acc = (acc & y) + 1;", supports_field=False),
    Operation("bit_or_mix", "acc = (acc | y) + 1;", supports_field=False),
    Operation("bit_xor_mix", "acc = (acc ^ y) + 1;", supports_field=False),
    Operation("shl_one_mix", "acc = (acc << 1) + y;", supports_field=False),
    Operation("shr_one_mix", "acc = (acc >> 1) + y;", supports_field=False),
]


def supported(operation: Operation, primitive_type: PrimitiveType) -> bool:
    if primitive_type.name == "Field":
        return operation.supports_field
    return operation.supports_int


def render_main(operation: Operation, primitive_type: PrimitiveType, calls: int) -> str:
    body = operation.body_template
    return f"""fn main(x: pub {primitive_type.name}, y: pub {primitive_type.name}) -> pub {primitive_type.name} {{
    let mut acc: {primitive_type.name} = x;

    for _ in 0..{calls} {{
        {body}
    }}

    acc
}}
"""


def write_package(root: Path, operation: Operation, primitive_type: PrimitiveType, calls: int) -> Path:
    package_name = f"primitive_{primitive_type.name.lower()}_{operation.name}_{calls}"
    package = root / package_name
    (package / "src").mkdir(parents=True)
    (package / "Nargo.toml").write_text(
        f"""[package]
name = "{package_name}"
type = "bin"
authors = ["bench"]
"""
    )
    (package / "src" / "main.nr").write_text(render_main(operation, primitive_type, calls))
    return package


def build_and_measure(operation: Operation, primitive_type: PrimitiveType, calls: int) -> int:
    with tempfile.TemporaryDirectory(prefix=f"primitive_bench_{primitive_type.name}_{operation.name}_{calls}_") as tmp:
        package = write_package(Path(tmp), operation, primitive_type, calls)
        artifact = package / "target" / f"primitive_{primitive_type.name.lower()}_{operation.name}_{calls}.json"
        return measure_package_circuit_size(package, artifact, f"{primitive_type.name} {operation.name}", calls)


def main() -> int:
    parser = argparse.ArgumentParser(description="Benchmark Noir primitive operation gate costs")
    parser.add_argument("--n-small", type=int, default=1)
    parser.add_argument("--n-big", type=int, default=64)
    parser.add_argument("--output", type=Path, default=ROOT / "bench" / "primitive_ops_latest.json")
    args = parser.parse_args()

    if shutil.which("bb") is None:
        raise RuntimeError("bb is required for gate benchmarks; install with bbup")

    results: dict[str, object] = {
        "timestamp": datetime.now().isoformat(),
        "n_small": args.n_small,
        "n_big": args.n_big,
        "benchmarks": {},
    }

    print(f"{'type':<8} {'operation':<12} {'gates@small':>12} {'gates@big':>12} {'per-iter':>10}")
    print("-" * 62)

    for primitive_type in TYPES:
        for operation in OPERATIONS:
            if not supported(operation, primitive_type):
                continue

            small = build_and_measure(operation, primitive_type, args.n_small)
            big = build_and_measure(operation, primitive_type, args.n_big)
            per_iter = (big - small) / (args.n_big - args.n_small)
            name = f"{primitive_type.name}_{operation.name}"

            results["benchmarks"][name] = {
                "type": primitive_type.name,
                "operation": operation.name,
                "small": {"calls": args.n_small, "circuit_size": small},
                "big": {"calls": args.n_big, "circuit_size": big},
                "per_iter_estimate": per_iter,
            }

            print(f"{primitive_type.name:<8} {operation.name:<12} {small:>12} {big:>12} {per_iter:>10.1f}")

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(results, indent=2) + "\n")
    print(f"\nwrote {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
