#!/usr/bin/env python3
"""Amortised gate-count probes for Noir cast patterns.

The probes compile tiny standalone Noir binaries and measure them with
`bb gates -s ultra_honk`. Each cast benchmark is compared to a same-carrier
addition baseline, so the reported delta is the per-iteration cost of the
extra cast/range-check work in that pattern.
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
class Benchmark:
    name: str
    value_type: str
    body: str
    baseline: str | None = None


BENCHMARKS = [
    Benchmark("field_add", "Field", "acc = acc + y;"),
    Benchmark(
        "field_to_u128_to_field",
        "Field",
        "acc = ((acc as u128) as Field) + y;",
        "field_add",
    ),
    Benchmark(
        "field_assert_128",
        "Field",
        "acc.assert_max_bit_size::<128>();\n        acc = acc + y;",
        "field_add",
    ),
    Benchmark(
        "field_to_u64_to_field",
        "Field",
        "acc = ((acc as u64) as Field) + y;",
        "field_add",
    ),
    Benchmark(
        "field_assert_64",
        "Field",
        "acc.assert_max_bit_size::<64>();\n        acc = acc + y;",
        "field_add",
    ),
    Benchmark("u128_add", "u128", "acc = acc + y;"),
    Benchmark(
        "u128_to_field_to_u128",
        "u128",
        "acc = ((acc as Field) as u128) + y;",
        "u128_add",
    ),
    Benchmark(
        "u128_to_u64_to_u128",
        "u128",
        "acc = ((acc as u64) as u128) + y;",
        "u128_add",
    ),
    Benchmark("u64_add", "u64", "acc = acc + y;"),
    Benchmark(
        "u64_to_field_to_u64",
        "u64",
        "acc = ((acc as Field) as u64) + y;",
        "u64_add",
    ),
    Benchmark(
        "u64_to_u128_to_u64",
        "u64",
        "acc = ((acc as u128) as u64) + y;",
        "u64_add",
    ),
]


def render_main(benchmark: Benchmark, calls: int) -> str:
    body = "\n        ".join(benchmark.body.splitlines())
    return f"""fn main(x: pub {benchmark.value_type}, y: pub {benchmark.value_type}) -> pub {benchmark.value_type} {{
    let mut acc: {benchmark.value_type} = x;

    for _ in 0..{calls} {{
        {body}
    }}

    acc
}}
"""


def write_package(root: Path, benchmark: Benchmark, calls: int) -> Path:
    package_name = f"cast_{benchmark.name}_{calls}"
    package = root / package_name
    (package / "src").mkdir(parents=True)
    (package / "Nargo.toml").write_text(
        f"""[package]
name = "{package_name}"
type = "bin"
authors = ["bench"]
"""
    )
    (package / "src" / "main.nr").write_text(render_main(benchmark, calls))
    return package


def build_and_measure(benchmark: Benchmark, calls: int) -> int:
    with tempfile.TemporaryDirectory(prefix=f"cast_bench_{benchmark.name}_{calls}_") as tmp:
        package = write_package(Path(tmp), benchmark, calls)
        artifact = package / "target" / f"cast_{benchmark.name}_{calls}.json"
        return measure_package_circuit_size(package, artifact, benchmark.name, calls)


def main() -> int:
    parser = argparse.ArgumentParser(description="Benchmark Noir cast gate costs")
    parser.add_argument("--n-small", type=int, default=1)
    parser.add_argument("--n-big", type=int, default=32)
    parser.add_argument("--output", type=Path, default=ROOT / "bench" / "cast_costs_latest.json")
    args = parser.parse_args()

    if shutil.which("bb") is None:
        raise RuntimeError("bb is required for gate benchmarks; install with bbup")

    benchmark_results: dict[str, object] = {}
    results: dict[str, object] = {
        "timestamp": datetime.now().isoformat(),
        "n_small": args.n_small,
        "n_big": args.n_big,
        "benchmarks": benchmark_results,
    }

    print(f"{'benchmark':<26} {'gates@small':>12} {'gates@big':>12} {'per-iter':>10} {'delta':>10}")
    print("-" * 76)

    per_iter_by_name: dict[str, float] = {}
    for benchmark in BENCHMARKS:
        small = build_and_measure(benchmark, args.n_small)
        big = build_and_measure(benchmark, args.n_big)
        per_iter = (big - small) / (args.n_big - args.n_small)
        per_iter_by_name[benchmark.name] = per_iter
        delta = None
        if benchmark.baseline is not None:
            delta = per_iter - per_iter_by_name[benchmark.baseline]

        benchmark_results[benchmark.name] = {
            "value_type": benchmark.value_type,
            "baseline": benchmark.baseline,
            "small": {"calls": args.n_small, "circuit_size": small},
            "big": {"calls": args.n_big, "circuit_size": big},
            "per_iter_estimate": per_iter,
            "delta_vs_baseline": delta,
        }

        delta_text = "" if delta is None else f"{delta:10.1f}"
        print(f"{benchmark.name:<26} {small:>12} {big:>12} {per_iter:>10.1f} {delta_text}")

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(results, indent=2) + "\n")
    print(f"\nwrote {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
