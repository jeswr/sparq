#!/usr/bin/env python3
"""Convert read_response_alloc_microbench output into a validated JSON envelope."""

# [GPT-5.6] Keep this emitter stdlib-only so the standalone harness has no Python packages.
import argparse
import datetime
import json
import pathlib
import re
import sys

MEASUREMENT = re.compile(r"^(BEFORE|AFTER).*?([0-9]+) alloc ops$", re.MULTILINE)
DELTA = re.compile(r"^DELTA\s+-([0-9]+) alloc ops \(([0-9]+)% fewer\)$", re.MULTILINE)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", required=True, type=pathlib.Path)
    parser.add_argument("--output", required=True, type=pathlib.Path)
    parser.add_argument("--revision", required=True)
    parser.add_argument("--rustc", required=True)
    parser.add_argument("--profile", required=True, choices=("debug", "release"))
    args = parser.parse_args()
    raw = args.input.read_text(encoding="utf-8")
    measurements = {name.lower(): int(value) for name, value in MEASUREMENT.findall(raw)}
    delta_match = DELTA.search(raw)
    if set(measurements) != {"before", "after"} or delta_match is None:
        print("invalid microbench output: missing allocation measurements", file=sys.stderr)
        return 2
    saved, percent = (int(value) for value in delta_match.groups())
    if saved != max(measurements["before"] - measurements["after"], 0):
        print("invalid microbench output: inconsistent allocation delta", file=sys.stderr)
        return 2
    envelope = {
        "schema_version": 1,
        "benchmark": "lws-core-read-response-allocation-path",
        "canonical": False,
        "measurement_kind": "heap_alloc_or_realloc_operations",
        "generated_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "source_revision": args.revision,
        "rustc": args.rustc,
        "profile": args.profile,
        "command": "cargo run -p sparq-lws-core --example read_response_alloc_microbench",
        "correctness_gate": "byte_identical_headers_asserted_by_example",
        "measurements": {**measurements, "saved": saved, "percent_fewer": percent},
        "raw_output": raw.splitlines(),
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(envelope, indent=2) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
