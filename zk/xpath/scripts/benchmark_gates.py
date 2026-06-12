#!/usr/bin/env python3
"""
Gate Count Benchmark Script for noir_XPath

This script compiles individual benchmark circuits for each XPath operation
and records the gate count for before/after optimization comparisons.

Usage:
    python3 scripts/benchmark_gates.py [--output FILE]
    
Output:
    Creates a JSON file with gate counts for each operation
"""

import argparse
import json
import shutil
import subprocess
import sys
import tempfile
from datetime import datetime
from pathlib import Path

# Operations to benchmark
BENCHMARKS = {
    "numeric_add_int": {
        "inputs": "a: pub i64, b: pub i64",
        "body": """
    numeric_add_int(a, b)
""",
        "return_type": "pub i64",
    },
    "numeric_subtract_int": {
        "inputs": "a: pub i64, b: pub i64",
        "body": """
    numeric_subtract_int(a, b)
""",
        "return_type": "pub i64",
    },
    "numeric_multiply_int": {
        "inputs": "a: pub i64, b: pub i64",
        "body": """
    numeric_multiply_int(a, b)
""",
        "return_type": "pub i64",
    },
    "numeric_divide_int": {
        "inputs": "a: pub i64, b: pub i64",
        "body": """
    numeric_divide_int(a, b)
""",
        "return_type": "pub i64",
    },
    "numeric_mod_int": {
        "inputs": "a: pub i64, b: pub i64",
        "body": """
    numeric_mod_int(a, b)
""",
        "return_type": "pub i64",
    },
    "abs_int": {
        "inputs": "a: pub i64",
        "body": """
    abs_int(a)
""",
        "return_type": "pub i64",
    },
    "datetime_equal": {
        "inputs": "a_micros: pub Field, b_micros: pub Field",
        "body": """
    let a = datetime_from_epoch_microseconds(a_micros);
    let b = datetime_from_epoch_microseconds(b_micros);
    datetime_equal(a, b)
""",
        "return_type": "pub bool",
    },
    "datetime_less_than": {
        "inputs": "a_micros: pub Field, b_micros: pub Field",
        "body": """
    let a = datetime_from_epoch_microseconds(a_micros);
    let b = datetime_from_epoch_microseconds(b_micros);
    datetime_less_than(a, b)
""",
        "return_type": "pub bool",
    },
    "datetime_greater_than": {
        "inputs": "a_micros: pub Field, b_micros: pub Field",
        "body": """
    let a = datetime_from_epoch_microseconds(a_micros);
    let b = datetime_from_epoch_microseconds(b_micros);
    datetime_greater_than(a, b)
""",
        "return_type": "pub bool",
    },
    "year_from_datetime": {
        "inputs": "micros: pub Field",
        "body": """
    let dt = datetime_from_epoch_microseconds(micros);
    year_from_datetime(dt)
""",
        "return_type": "pub i32",
    },
    "month_from_datetime": {
        "inputs": "micros: pub Field",
        "body": """
    let dt = datetime_from_epoch_microseconds(micros);
    month_from_datetime(dt)
""",
        "return_type": "pub u8",
    },
    "day_from_datetime": {
        "inputs": "micros: pub Field",
        "body": """
    let dt = datetime_from_epoch_microseconds(micros);
    day_from_datetime(dt)
""",
        "return_type": "pub u8",
    },
    "hours_from_datetime": {
        "inputs": "micros: pub Field",
        "body": """
    let dt = datetime_from_epoch_microseconds(micros);
    hours_from_datetime(dt)
""",
        "return_type": "pub u8",
    },
    "minutes_from_datetime": {
        "inputs": "micros: pub Field",
        "body": """
    let dt = datetime_from_epoch_microseconds(micros);
    minutes_from_datetime(dt)
""",
        "return_type": "pub u8",
    },
    "seconds_from_datetime": {
        "inputs": "micros: pub Field",
        "body": """
    let dt = datetime_from_epoch_microseconds(micros);
    seconds_from_datetime(dt)
""",
        "return_type": "pub u8",
    },
    "fn_not": {
        "inputs": "a: pub bool",
        "body": """
    fn_not(a)
""",
        "return_type": "pub bool",
    },
}


def get_project_root():
    """Get the project root directory."""
    script_dir = Path(__file__).parent
    return script_dir.parent


def create_benchmark_project(tmpdir: Path, name: str, benchmark: dict) -> Path:
    """Create a temporary Noir project for a single benchmark."""
    project_dir = tmpdir / name
    project_dir.mkdir(parents=True)
    src_dir = project_dir / "src"
    src_dir.mkdir()

    # Create Nargo.toml
    nargo_toml = f"""[package]
name = "{name}"
type = "bin"
authors = ["benchmark"]

[dependencies]
xpath = {{ path = "{get_project_root() / 'xpath'}" }}
"""
    (project_dir / "Nargo.toml").write_text(nargo_toml)

    # Create main.nr
    main_nr = f"""use xpath::{{
    numeric_add_int, numeric_subtract_int, numeric_multiply_int, numeric_divide_int,
    numeric_mod_int, abs_int,
    datetime_from_epoch_microseconds,
    datetime_equal, datetime_less_than, datetime_greater_than,
    year_from_datetime, month_from_datetime, day_from_datetime,
    hours_from_datetime, minutes_from_datetime, seconds_from_datetime,
    fn_not
}};

fn main({benchmark['inputs']}) -> {benchmark['return_type']} {{{benchmark['body']}}}
"""
    (src_dir / "main.nr").write_text(main_nr)

    return project_dir


def run_nargo_info(project_dir: Path) -> dict:
    """Run nargo info and parse the output."""
    result = subprocess.run(
        ["nargo", "info"],
        cwd=project_dir,
        capture_output=True,
        text=True,
    )

    if result.returncode != 0:
        print(f"Error running nargo info: {result.stderr}", file=sys.stderr)
        return {"error": result.stderr}

    # Parse table output
    output = result.stdout
    info = {}

    # Parse the table format:
    # | Package | Function | Expression Width | ACIR Opcodes | Brillig Opcodes |
    lines = output.strip().split('\n')
    
    for line in lines:
        # Look for the main function row
        if '| main' in line or '|main' in line:
            parts = [p.strip() for p in line.split('|') if p.strip()]
            if len(parts) >= 4:
                # parts: [Package, Function, Expression Width, ACIR Opcodes, Brillig Opcodes]
                try:
                    acir_str = parts[3].strip()
                    if acir_str != 'N/A':
                        info["acir_opcodes"] = int(acir_str)
                except (ValueError, IndexError):
                    # If the ACIR opcode count cannot be parsed, omit it from the results.
                    pass
                try:
                    brillig_str = parts[4].strip() if len(parts) > 4 else "N/A"
                    if brillig_str != 'N/A':
                        info["brillig_opcodes"] = int(brillig_str)
                except (ValueError, IndexError):
                    # If the Brillig opcode count cannot be parsed, omit it from the results.
                    pass

    return info


def run_nargo_compile_and_info(project_dir: Path) -> dict:
    """Compile the project and get info."""
    # First compile
    compile_result = subprocess.run(
        ["nargo", "compile"],
        cwd=project_dir,
        capture_output=True,
        text=True,
    )

    if compile_result.returncode != 0:
        print(f"Compilation error: {compile_result.stderr}", file=sys.stderr)
        return {"error": compile_result.stderr, "compile_output": compile_result.stdout}

    # Then get info
    return run_nargo_info(project_dir)


def benchmark_all(output_file: Path = None):
    """Run all benchmarks and collect gate counts."""
    results = {
        "timestamp": datetime.now().isoformat(),
        "git_commit": get_git_commit(),
        "benchmarks": {},
    }

    with tempfile.TemporaryDirectory() as tmpdir:
        tmpdir = Path(tmpdir)

        for name, benchmark in BENCHMARKS.items():
            print(f"Benchmarking {name}...", end=" ", flush=True)

            try:
                project_dir = create_benchmark_project(tmpdir, name, benchmark)
                info = run_nargo_compile_and_info(project_dir)
                results["benchmarks"][name] = info
                
                # Print summary
                if "error" in info:
                    print(f"ERROR: {info['error'][:50]}...")
                elif "acir_opcodes" in info:
                    print(f"ACIR: {info['acir_opcodes']}", end="")
                    if "brillig_opcodes" in info:
                        print(f", Brillig: {info['brillig_opcodes']}", end="")
                    print()
                else:
                    print(f"OK (parsing failed - check raw output)")

            except Exception as e:
                print(f"ERROR: {e}")
                results["benchmarks"][name] = {"error": str(e)}

    # Save results
    if output_file is None:
        output_file = get_project_root() / "gate_counts.json"

    output_file.parent.mkdir(parents=True, exist_ok=True)

    # Load existing results if any (for comparison)
    existing_results = []
    if output_file.exists():
        try:
            with open(output_file) as f:
                data = json.load(f)
                if isinstance(data, list):
                    existing_results = data
                else:
                    existing_results = [data]
        except (json.JSONDecodeError, IOError):
            # If the existing results file is missing, unreadable, or contains
            # invalid JSON, ignore it and proceed with only the new results.
            pass

    # Append new results
    existing_results.append(results)

    with open(output_file, "w") as f:
        json.dump(existing_results, f, indent=2)

    print(f"\nResults saved to {output_file}")

    # Print comparison if we have previous results
    if len(existing_results) > 1:
        print_comparison(existing_results[-2], existing_results[-1])

    return results


def get_git_commit():
    """Get the current git commit hash."""
    try:
        result = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=get_project_root(),
            capture_output=True,
            text=True,
        )
        if result.returncode == 0:
            return result.stdout.strip()[:8]
    except Exception:
        # If git is unavailable or this is not a git repository, fall back to "unknown".
        pass
    return "unknown"


def print_comparison(old_results: dict, new_results: dict):
    """Print a comparison between two benchmark runs."""
    print("\n" + "=" * 60)
    print("COMPARISON WITH PREVIOUS RUN")
    print("=" * 60)
    print(f"Old: {old_results.get('timestamp', 'unknown')} ({old_results.get('git_commit', 'unknown')})")
    print(f"New: {new_results.get('timestamp', 'unknown')} ({new_results.get('git_commit', 'unknown')})")
    print("-" * 60)
    print(f"{'Operation':<30} {'Old ACIR':>12} {'New ACIR':>12} {'Change':>12}")
    print("-" * 60)

    old_benchmarks = old_results.get("benchmarks", {})
    new_benchmarks = new_results.get("benchmarks", {})

    total_old = 0
    total_new = 0

    for name in sorted(set(old_benchmarks.keys()) | set(new_benchmarks.keys())):
        old_info = old_benchmarks.get(name, {})
        new_info = new_benchmarks.get(name, {})

        old_acir = old_info.get("acir_opcodes", "N/A")
        new_acir = new_info.get("acir_opcodes", "N/A")

        if isinstance(old_acir, int) and isinstance(new_acir, int):
            diff = new_acir - old_acir
            pct = (diff / old_acir * 100) if old_acir > 0 else 0
            change = f"{diff:+d} ({pct:+.1f}%)"
            total_old += old_acir
            total_new += new_acir
        else:
            change = "N/A"

        print(f"{name:<30} {str(old_acir):>12} {str(new_acir):>12} {change:>12}")

    print("-" * 60)
    if total_old > 0 and total_new > 0:
        total_diff = total_new - total_old
        total_pct = (total_diff / total_old * 100)
        print(f"{'TOTAL':<30} {total_old:>12} {total_new:>12} {total_diff:+d} ({total_pct:+.1f}%)")
    print("=" * 60)


def print_summary(results: dict):
    """Print a summary table of the latest benchmark results."""
    print("\n" + "=" * 80)
    print("XPath GATE COUNT SUMMARY")
    print("=" * 80)
    print(f"Timestamp: {results.get('timestamp', 'unknown')}")
    print(f"Git commit: {results.get('git_commit', 'unknown')}")
    print("-" * 80)
    print(f"{'Operation':<30} {'ACIR Opcodes':>15} {'Brillig Opcodes':>18}")
    print("-" * 80)

    benchmarks = results.get("benchmarks", {})
    total_acir = 0
    total_brillig = 0

    for name in sorted(benchmarks.keys()):
        info = benchmarks[name]
        acir = info.get("acir_opcodes", "N/A")
        brillig = info.get("brillig_opcodes", "N/A")

        if isinstance(acir, int):
            total_acir += acir
        if isinstance(brillig, int):
            total_brillig += brillig

        print(f"{name:<30} {str(acir):>15} {str(brillig):>18}")

    print("-" * 80)
    print(f"{'TOTAL':<30} {total_acir:>15} {total_brillig:>18}")
    print("=" * 80)


def main():
    parser = argparse.ArgumentParser(
        description="Benchmark gate counts for XPath operations"
    )
    parser.add_argument(
        "--output", "-o",
        type=Path,
        default=None,
        help="Output file for benchmark results (default: gate_counts.json)",
    )
    parser.add_argument(
        "--compare",
        type=Path,
        default=None,
        help="Compare with a previous benchmark file",
    )
    parser.add_argument(
        "--summary", "-s",
        action="store_true",
        help="Print summary of latest benchmark results without running new benchmarks",
    )

    args = parser.parse_args()

    output_file = args.output or get_project_root() / "gate_counts.json"

    if args.summary:
        # Just print summary of existing results
        if not output_file.exists():
            print(f"No benchmark results found at {output_file}")
            print("Run without --summary to generate benchmarks first.")
            sys.exit(1)
        with open(output_file) as f:
            data = json.load(f)
            if isinstance(data, list) and len(data) > 0:
                print_summary(data[-1])
            else:
                print("No benchmark data found")
        return

    if args.compare:
        # Just compare two files
        with open(args.compare) as f:
            data = json.load(f)
            if isinstance(data, list) and len(data) >= 2:
                print_comparison(data[-2], data[-1])
            else:
                print("Need at least 2 benchmark runs to compare")
        return

    # Check nargo is available
    if shutil.which("nargo") is None:
        print("Error: 'nargo' command not found. Please install Noir.", file=sys.stderr)
        sys.exit(1)

    benchmark_all(args.output)


if __name__ == "__main__":
    main()
