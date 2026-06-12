#!/usr/bin/env python3
"""Compare float operation gate benchmarks against a baseline JSON file."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
DEFAULT_BASELINE = ROOT / "bench" / "float_ops_latest.json"


def load_estimates(path: Path) -> dict[str, float]:
    data = json.loads(path.read_text())
    benchmarks = data.get("benchmarks")
    if not isinstance(benchmarks, dict):
        raise ValueError(f"{path} does not contain a benchmarks object")

    estimates: dict[str, float] = {}
    for name, entry in benchmarks.items():
        if not isinstance(entry, dict) or "per_call_estimate" not in entry:
            raise ValueError(f"{path} benchmark {name} is missing per_call_estimate")
        estimates[name] = float(entry["per_call_estimate"])
    return estimates


def main() -> int:
    parser = argparse.ArgumentParser(description="Compare float benchmark per-call gate estimates")
    parser.add_argument("candidate", type=Path, help="Benchmark JSON produced by benchmark_float_ops.py")
    parser.add_argument("--baseline", type=Path, default=DEFAULT_BASELINE)
    parser.add_argument("--ops", nargs="+", help="Subset of operations to compare")
    parser.add_argument("--max-regression", type=float, default=1.0, help="Allowed per-call gate increase")
    args = parser.parse_args()

    baseline = load_estimates(args.baseline)
    candidate = load_estimates(args.candidate)
    names = args.ops if args.ops is not None else sorted(baseline)

    print(f"{'benchmark':<12} {'baseline':>12} {'candidate':>12} {'delta':>12}")
    print("-" * 52)

    failed = False
    for name in names:
        if name not in baseline:
            raise ValueError(f"{name} is missing from baseline {args.baseline}")
        if name not in candidate:
            raise ValueError(f"{name} is missing from candidate {args.candidate}")

        delta = candidate[name] - baseline[name]
        failed = failed | (delta > args.max_regression)
        print(f"{name:<12} {baseline[name]:>12.1f} {candidate[name]:>12.1f} {delta:>12.1f}")

    if failed:
        print(f"\nregression exceeded {args.max_regression:.1f} gates/call", file=sys.stderr)
        return 1

    return 0


if __name__ == "__main__":
    raise SystemExit(main())