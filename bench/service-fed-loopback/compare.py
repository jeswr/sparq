#!/usr/bin/env python3
"""Canonical solution-multiset oracle for sq-139od's SERVICE differential."""

from __future__ import annotations

import argparse
from collections import Counter
import json
from pathlib import Path
import platform
import subprocess
import sys

XSD_STRING = "http://www.w3.org/2001/XMLSchema#string"


def canonical_term(term: dict) -> tuple:
    """Return a strict, hashable SPARQL Results JSON term representation."""
    term_type = term.get("type")
    value = term.get("value")
    if term_type in ("uri", "iri") and isinstance(value, str):
        return ("iri", value)
    if term_type == "bnode":
        raise ValueError(
            "blank-node result bindings are outside this fixture corpus; refusing "
            "label-sensitive or identity-collapsing comparison"
        )
    if term_type in ("literal", "typed-literal") and isinstance(value, str):
        language = term.get("xml:lang")
        if language is not None:
            if not isinstance(language, str):
                raise ValueError("literal xml:lang is not a string")
            return ("literal", value, "lang", language.lower())
        datatype = term.get("datatype", XSD_STRING)
        if not isinstance(datatype, str):
            raise ValueError("literal datatype is not a string")
        return ("literal", value, "datatype", datatype)
    if term_type == "triple" and isinstance(value, dict):
        try:
            return (
                "triple",
                canonical_term(value["subject"]),
                canonical_term(value["predicate"]),
                canonical_term(value["object"]),
            )
        except KeyError as error:
            raise ValueError(f"triple result is missing {error.args[0]}") from error
    raise ValueError(f"unsupported or malformed result term: {term!r}")


def canonical_row(row: dict) -> tuple:
    if not isinstance(row, dict):
        raise ValueError("solution row is not an object")
    return tuple(sorted((variable, canonical_term(term)) for variable, term in row.items()))


def canonical_multiset(rows: list[dict]) -> Counter:
    if not isinstance(rows, list):
        raise ValueError("solution set is not an array")
    return Counter(canonical_row(row) for row in rows)


def render_difference(sparq: Counter, comunica: Counter) -> str:
    missing = comunica - sparq
    extra = sparq - comunica
    return (
        "canonical solution-multiset mismatch; "
        f"sparq_missing={sum(missing.values())} sparq_extra={sum(extra.values())}; "
        f"missing_sample={list(missing.items())[:1]!r} "
        f"extra_sample={list(extra.items())[:1]!r}"
    )


def command_version(command: list[str]) -> str:
    try:
        completed = subprocess.run(
            command, check=False, capture_output=True, text=True, timeout=10
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        return f"unavailable ({error})"
    output = (completed.stdout or completed.stderr).strip().splitlines()
    return output[0] if output else f"exit {completed.returncode} (no output)"


def compare(raw: dict) -> tuple[dict, bool]:
    if raw.get("schema_version") != 1:
        raise ValueError("raw driver schema_version must be 1")
    if raw.get("endpoint_count", 0) < 2:
        raise ValueError("raw driver output proves fewer than two endpoints ran")
    fixtures = raw.get("fixtures")
    if not isinstance(fixtures, list) or not fixtures:
        raise ValueError("raw driver output contains no fixtures")

    comparisons = []
    all_equal = True
    seen = set()
    for fixture in fixtures:
        fixture_id = fixture.get("id")
        if not isinstance(fixture_id, str) or not fixture_id or fixture_id in seen:
            raise ValueError(f"invalid or duplicate fixture id: {fixture_id!r}")
        seen.add(fixture_id)
        expected_rows = fixture.get("expected_rows")
        if not isinstance(expected_rows, int) or expected_rows <= 0:
            raise ValueError(f"fixture {fixture_id!r} has vacuous expected_rows")
        sparq_rows = fixture.get("sparq")
        comunica_rows = fixture.get("comunica")
        if not isinstance(sparq_rows, list) or not isinstance(comunica_rows, list):
            raise ValueError(f"fixture {fixture_id!r} has a non-array result set")
        if len(sparq_rows) != expected_rows:
            raise ValueError(
                f"fixture {fixture_id!r}: sparq rows {len(sparq_rows)} != "
                f"committed expected_rows {expected_rows}"
            )
        if len(comunica_rows) != expected_rows:
            raise ValueError(
                f"fixture {fixture_id!r}: Comunica rows {len(comunica_rows)} != "
                f"committed expected_rows {expected_rows}"
            )
        sparq_canonical = canonical_multiset(sparq_rows)
        comunica_canonical = canonical_multiset(comunica_rows)
        equal = sparq_canonical == comunica_canonical
        all_equal &= equal
        comparisons.append(
            {
                "fixture": fixture_id,
                "query_file": fixture.get("query_file"),
                "sparq_rows": len(sparq_rows),
                "comunica_rows": len(comunica_rows),
                "verdict": "parity" if equal else "behind",
                "gap_root_cause": None
                if equal
                else render_difference(sparq_canonical, comunica_canonical),
            }
        )

    envelope = {
        "schema_version": 1,
        "suite_id": raw.get("suite_id"),
        "axis": "Federation / SERVICE result equivalence",
        "bead": raw.get("bead"),
        "canonical": False,
        "measurement": "correctness-only; no performance measurements",
        "invariant": "sparq and Comunica canonical solution multisets are equal for every fixture",
        "overall_verdict": "parity" if all_equal else "behind",
        "driver": raw.get("driver"),
        "endpoint_count": raw.get("endpoint_count"),
        "versions": {
            "comunica": raw.get("comunica_version", "unknown"),
            "node": command_version(["node", "--version"]),
            "rustc": command_version(["rustc", "--version"]),
        },
        "host": {
            "system": platform.system(),
            "release": platform.release(),
            "machine": platform.machine(),
        },
        "comparisons": comparisons,
    }
    return envelope, all_equal


def self_test() -> None:
    iri_a = {"type": "uri", "value": "http://example.test/a"}
    iri_b = {"type": "uri", "value": "http://example.test/b"}
    plain = {"type": "literal", "value": "x"}
    typed = {"type": "literal", "value": "x", "datatype": XSD_STRING}
    row_a = {"x": iri_a, "label": plain}
    row_b = {"x": iri_b}

    assert canonical_multiset([row_a, row_b]) == canonical_multiset(
        [row_b, {"label": typed, "x": iri_a}]
    ), "row order and plain/xsd:string spelling must not affect equality"
    assert canonical_multiset([row_a, row_a]) != canonical_multiset(
        [row_a]
    ), "duplicate multiplicity must affect equality"
    assert canonical_multiset([row_a]) != canonical_multiset(
        [{"x": iri_b, "label": plain}]
    ), "same-cardinality term mutation must affect equality"

    mutant = {
        "schema_version": 1,
        "suite_id": "self-test",
        "bead": "sq-139od",
        "endpoint_count": 2,
        "driver": "self-test",
        "fixtures": [
            {
                "id": "mutation-witness",
                "query_file": "self-test.rq",
                "expected_rows": 1,
                "sparq": [{"x": iri_a}],
                "comunica": [{"x": iri_b}],
            }
        ],
    }
    envelope, equal = compare(mutant)
    assert not equal and envelope["overall_verdict"] == "behind", (
        "mutation witness must turn the differential red"
    )


def print_table(envelope: dict) -> None:
    print("fixture\tsparq_rows\tcomunica_rows\tverdict\tgap/root cause")
    for row in envelope["comparisons"]:
        print(
            f"{row['fixture']}\t{row['sparq_rows']}\t{row['comunica_rows']}\t"
            f"{row['verdict']}\t{row['gap_root_cause'] or '-'}"
        )
    print(f"overall\t-\t-\t{envelope['overall_verdict']}\t-")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", type=Path)
    parser.add_argument("--envelope", type=Path)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()

    if args.self_test:
        self_test()
        print("service-fed-loopback comparator self-test: PASS")
        if args.input is None:
            return 0
    if args.input is None or args.envelope is None:
        parser.error("--input and --envelope are required unless only --self-test is used")

    raw = json.loads(args.input.read_text(encoding="utf-8"))
    envelope, equal = compare(raw)
    args.envelope.parent.mkdir(parents=True, exist_ok=True)
    args.envelope.write_text(json.dumps(envelope, indent=2) + "\n", encoding="utf-8")
    print_table(envelope)
    print(f"envelope: {args.envelope}", file=sys.stderr)
    return 0 if equal else 1


if __name__ == "__main__":
    raise SystemExit(main())
