#!/usr/bin/env python3
"""Hermetic, stopwatch-free result-set oracle tests for arrow_interop.py."""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import arrow_interop as ai  # noqa: E402


def main():
    oracle = ai.expected()
    exports = {}
    for name in ai.QUERIES:
        query = os.path.join(HERE, name)
        exports[name] = {
            "sparq-arrow": ai.sparq_rows(query),
            "pyoxigraph": ai.pyoxigraph_rows(query),
        }
    ai.compare(exports, oracle)

    poisoned = {name: dict(engines) for name, engines in exports.items()}
    poisoned["select-all.rq"]["pyoxigraph"] = exports["select-all.rq"]["pyoxigraph"][:-1]
    try:
        ai.compare(poisoned, oracle)
    except ValueError:
        print("ok: exact solution multisets agree; mutation is rejected before timing")
        return 0
    print("FAIL: corrupted export crossed the equality gate", file=sys.stderr)
    return 1


if __name__ == "__main__":
    sys.exit(main())
