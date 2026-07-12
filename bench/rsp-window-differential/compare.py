#!/usr/bin/env python3
"""[GPT-5.6] sq-no6iy — fail-closed window-content comparator and latency envelope."""
import argparse
import json
import sys


def read_rows(path, observed):
    rows = []
    with open(path, encoding="utf-8") as source:
        for line_number, line in enumerate(source, 1):
            line = line.rstrip("\n")
            if not line or line.startswith("#") or line.startswith("report_ts\t"):
                continue
            fields = line.split("\t")
            wanted = 3 if observed else 2
            if len(fields) != wanted:
                raise ValueError(f"{path}:{line_number}: expected {wanted} fields")
            rows.append((int(fields[0]), fields[1], int(fields[2]) if observed else None))
    timestamps = [row[0] for row in rows]
    if len(timestamps) != len(set(timestamps)):
        raise ValueError(f"{path}: duplicate report timestamp")
    return rows


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--observed", required=True)
    parser.add_argument("--golden", required=True)
    parser.add_argument("--envelope")
    args = parser.parse_args()
    try:
        observed = read_rows(args.observed, True)
        golden = read_rows(args.golden, False)
    except (OSError, ValueError) as error:
        print(f"rsp-window-differential: {error}", file=sys.stderr)
        return 2
    actual = [(ts, content) for ts, content, _ in observed]
    expected = [(ts, content) for ts, content, _ in golden]
    if actual != expected:
        print("rsp-window-differential: window content differs from RSP4J golden", file=sys.stderr)
        for index, pair in enumerate(zip(expected, actual)):
            if pair[0] != pair[1]:
                print(f"first mismatch at window index {index}", file=sys.stderr)
                break
        if len(actual) != len(expected):
            print(f"window-count mismatch: expected {len(expected)}, observed {len(actual)}", file=sys.stderr)
        return 1
    envelope = {
        "suite": "rsp-window-differential",
        "oracle": "pinned-rsp4j-yasper-event-time-capture",
        "equality": "canonical-multiset-per-report-timestamp",
        "status": "equal",
        "time_model_caveat": (
            "sparq-rsp is clock-free and closes on pushed event-time watermarks; "
            "RSP4J/YASPER is normally wall-clock driven. Latencies are advisory and "
            "not a raw cross-engine throughput comparison."
        ),
        "windows": [
            {"report_ts": ts, "emit_latency_ns": latency}
            for ts, _, latency in observed
        ],
    }
    if args.envelope:
        with open(args.envelope, "w", encoding="utf-8") as destination:
            json.dump(envelope, destination, indent=2)
            destination.write("\n")
    print(f"rsp-window-differential: {len(observed)} windows equal")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

