#!/usr/bin/env python3
"""Shared helpers for Noir gate benchmark scripts."""

from __future__ import annotations

import json
import re
import subprocess
from pathlib import Path


def run(command: list[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(command, cwd=cwd, capture_output=True, text=True)


def parse_circuit_size(stdout: str) -> int:
    try:
        parsed = json.loads(stdout)
        return int(parsed["functions"][0]["circuit_size"])
    except (json.JSONDecodeError, KeyError, IndexError, TypeError, ValueError):
        match = re.search(r'"circuit_size"\s*:\s*(\d+)', stdout)
        if match is None:
            raise ValueError(f"could not parse circuit_size from bb output:\n{stdout}")
        return int(match.group(1))


def measure_package_circuit_size(
    package: Path,
    artifact: Path,
    label: str,
    calls: int,
    *,
    include_breakdown: bool = False,
    include_bbup_hint: bool = False,
) -> int:
    compile_result = run(["nargo", "compile", "--silence-warnings"], package)
    if compile_result.returncode != 0:
        raise RuntimeError(
            f"nargo compile failed for {label} N={calls}:\n"
            f"{compile_result.stdout}\n{compile_result.stderr}"
        )

    command = ["bb", "gates", "-s", "ultra_honk", "-b", str(artifact)]
    if include_breakdown:
        command.append("--include_gates_per_opcode")

    gates_result = run(command, package)
    if gates_result.returncode != 0:
        if include_bbup_hint and ("Length is too large" in gates_result.stderr or "Length is too large" in gates_result.stdout):
            raise RuntimeError(
                f"bb gates failed for {label} N={calls}:\n"
                "bb reported 'Length is too large', which usually means bb is not compatible "
                "with the installed nargo/noirc version. Run `bbup`, confirm `bb --version`, "
                "then retry this benchmark.\n"
                f"{gates_result.stdout}\n{gates_result.stderr}"
            )
        raise RuntimeError(
            f"bb gates failed for {label} N={calls}:\n"
            f"{gates_result.stdout}\n{gates_result.stderr}"
        )

    return parse_circuit_size(gates_result.stdout)
