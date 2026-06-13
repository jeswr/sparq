#!/usr/bin/env python3
"""Gate-count benchmark for zk/xpath float/double operations.

Measures UltraHonk circuit_size (`bb gates -s ultra_honk`) for a representative
set of float-heavy XPath functions, using the amplified-harness pattern from
zk/ieee754/scripts/benchmark_float_ops.py: each op is instantiated N times in a
loop with witness-dependent inputs (pub witnesses defeat constant folding), and
per-call cost is estimated as (gates@big - gates@small) / (big - small).

Run BEFORE the IEEE754 dependency migration to capture the old-API baseline and
AFTER to capture the new-API numbers:

    python3 scripts/bench_float_migration.py --output /tmp/xpath_float_before.json
    # ... migrate ...
    python3 scripts/bench_float_migration.py --output /tmp/xpath_float_after.json
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]  # zk/xpath
XPATH_LIB = REPO / "xpath"

# name -> (kind, body)
#   kind: "double" | "float" decides operand construction
#   body uses `a` / `b` (XsdDouble or XsdFloat) and folds into `acc: u64`.
BENCHMARKS: dict[str, tuple[str, str]] = {
    "add_double": ("double", "acc = acc ^ xpath::numeric_add_double(a, b).to_bits();"),
    "mul_double": ("double", "acc = acc ^ xpath::numeric_multiply_double(a, b).to_bits();"),
    "div_double": ("double", "acc = acc ^ xpath::numeric_divide_double(a, b).to_bits();"),
    "eq_double": ("double", "acc = acc ^ (xpath::numeric_equal_double(a, b) as u64);"),
    "lt_double": ("double", "acc = acc ^ (xpath::numeric_less_than_double(a, b) as u64);"),
    "round_double": ("double", "acc = acc ^ xpath::round_double(a).to_bits();"),
    "floor_double": ("double", "acc = acc ^ xpath::floor_double(a).to_bits();"),
    "ceil_double": ("double", "acc = acc ^ xpath::ceil_double(a).to_bits();"),
    "abs_double": ("double", "acc = acc ^ xpath::abs_double(a).to_bits();"),
    "add_float": ("float", "acc = acc ^ (xpath::numeric_add_float(a, b).to_bits() as u64);"),
    "mul_float": ("float", "acc = acc ^ (xpath::numeric_multiply_float(a, b).to_bits() as u64);"),
    "lt_float": ("float", "acc = acc ^ (xpath::numeric_less_than_float(a, b) as u64);"),
    "round_float": ("float", "acc = acc ^ (xpath::round_float(a).to_bits() as u64);"),
    "floor_float": ("float", "acc = acc ^ (xpath::floor_float(a).to_bits() as u64);"),
    "ceil_float": ("float", "acc = acc ^ (xpath::ceil_float(a).to_bits() as u64);"),
    "int_to_double": ("int", "acc = acc ^ xpath::XsdDouble::from_small_int(n).to_bits();"),
}

OPERANDS = {
    "double": (
        "        let a = xpath::XsdDouble::from_bits(a_bits ^ (i as u64));\n"
        "        let b = xpath::XsdDouble::from_bits(b_bits + (i as u64));\n"
    ),
    "float": (
        "        let a = xpath::XsdFloat::from_bits((a_bits ^ (i as u64)) as u32);\n"
        "        let b = xpath::XsdFloat::from_bits((b_bits + (i as u64)) as u32);\n"
    ),
    "int": ("        let n = (((a_bits + (i as u64)) & 0x7F) as u8) as i8;\n"),
}


def render_main(kind: str, body: str, calls: int) -> str:
    return (
        "fn main(a_bits: pub u64, b_bits: pub u64) -> pub u64 {\n"
        "    let mut acc: u64 = 0;\n"
        f"    for i in 0..{calls} {{\n"
        f"{OPERANDS[kind]}"
        f"        {body}\n"
        "    }\n"
        "    acc\n"
        "}\n"
    )


def parse_circuit_size(stdout: str) -> int:
    try:
        return int(json.loads(stdout)["functions"][0]["circuit_size"])
    except (json.JSONDecodeError, KeyError, IndexError, TypeError, ValueError):
        match = re.search(r'"circuit_size"\s*:\s*(\d+)', stdout)
        if match is None:
            raise ValueError(f"could not parse circuit_size from bb output:\n{stdout}")
        return int(match.group(1))


def measure(root: Path, name: str, kind: str, body: str, calls: int) -> int:
    pkg = root / f"{name}_n{calls}"
    (pkg / "src").mkdir(parents=True)
    (pkg / "Nargo.toml").write_text(
        "[package]\n"
        f'name = "bench_{name}_n{calls}"\n'
        'type = "bin"\n'
        'authors = [""]\n\n'
        "[dependencies]\n"
        f'xpath = {{ path = "{XPATH_LIB}" }}\n'
    )
    (pkg / "src" / "main.nr").write_text(render_main(kind, body, calls))

    compile_result = subprocess.run(
        ["nargo", "compile", "--silence-warnings"], cwd=pkg, capture_output=True, text=True
    )
    if compile_result.returncode != 0:
        raise RuntimeError(
            f"nargo compile failed for {name} N={calls}:\n"
            f"{compile_result.stdout}\n{compile_result.stderr}"
        )
    artifact = pkg / "target" / f"bench_{name}_n{calls}.json"
    gates_result = subprocess.run(
        ["bb", "gates", "-s", "ultra_honk", "-b", str(artifact)],
        cwd=pkg,
        capture_output=True,
        text=True,
    )
    if gates_result.returncode != 0:
        raise RuntimeError(
            f"bb gates failed for {name} N={calls}:\n"
            f"{gates_result.stdout}\n{gates_result.stderr}"
        )
    return parse_circuit_size(gates_result.stdout)


def main() -> int:
    parser = argparse.ArgumentParser(description="Benchmark xpath float op gates")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--n-small", type=int, default=1)
    parser.add_argument("--n-big", type=int, default=8)
    parser.add_argument("--only", type=str, default=None, help="comma-separated benchmark names")
    args = parser.parse_args()

    if shutil.which("bb") is None:
        print("bb is required; install with bbup", file=sys.stderr)
        return 1

    selected = dict(BENCHMARKS)
    if args.only:
        wanted = set(args.only.split(","))
        selected = {k: v for k, v in selected.items() if k in wanted}

    results: dict[str, dict[str, float]] = {}
    tmp = Path(tempfile.mkdtemp(prefix="xpath-float-bench."))
    try:
        print(f"{'benchmark':<16} {'gates@%d' % args.n_small:>10} {'gates@%d' % args.n_big:>10} {'per-call':>10}")
        for name, (kind, body) in selected.items():
            small = measure(tmp, name, kind, body, args.n_small)
            big = measure(tmp, name, kind, body, args.n_big)
            per_call = (big - small) / (args.n_big - args.n_small)
            results[name] = {"small": small, "big": big, "per_call": per_call}
            print(f"{name:<16} {small:>10} {big:>10} {per_call:>10.1f}")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)

    args.output.write_text(
        json.dumps(
            {"n_small": args.n_small, "n_big": args.n_big, "benchmarks": results},
            indent=2,
        )
        + "\n"
    )
    print(f"wrote {args.output}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
