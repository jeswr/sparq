#!/usr/bin/env python3
# [FABLE-5] sq-98w7z.4 — hermetic tests for report.py's hard correctness differential.
#
# The differential must reject result-set SHAPE drift, not just row-count drift on
# the intersection: a truncated variant TSV, a missing suite, or extra variant
# queries must all exit nonzero and leave NO gate-usable summary.json behind
# (bolt.sh gates BOLT on summary.json's pgo geomean).
import json
import subprocess
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPORT = HERE / "report.py"

BASELINE_TSV = "q1\t10\t100.0\nq2\t20\t200.0\nq3\t30\t300.0\n"


def make_results(variant_tsvs):
    # One results dir: baseline + one 'pgo' variant, watdiv suite only (a suite
    # absent from BOTH sides is legitimately skipped).
    root = Path(tempfile.mkdtemp(prefix="pgo-report-test-"))
    (root / "baseline").mkdir()
    (root / "baseline" / "watdiv.tsv").write_text(BASELINE_TSV)
    (root / "pgo").mkdir()
    for name, body in variant_tsvs.items():
        (root / "pgo" / name).write_text(body)
    return root


def run_report(root):
    return subprocess.run(
        [sys.executable, str(REPORT), str(root), "baseline", "pgo"],
        capture_output=True,
        text=True,
    )


def check(label, root, expect_rc_zero, expect_summary):
    proc = run_report(root)
    ok_rc = (proc.returncode == 0) == expect_rc_zero
    ok_summary = (root / "summary.json").is_file() == expect_summary
    if ok_rc and ok_summary:
        print(f"ok: {label}")
        return 0
    print(f"FAIL: {label}: rc={proc.returncode} summary={ (root / 'summary.json').is_file() }", file=sys.stderr)
    print(proc.stderr, file=sys.stderr)
    return 1


def main():
    failures = 0

    # Identical shape + rows -> passes and writes the gate summary.
    root = make_results({"watdiv.tsv": "q1\t10\t90.0\nq2\t20\t180.0\nq3\t30\t270.0\n"})
    failures += check("identical shape passes", root, expect_rc_zero=True, expect_summary=True)
    if (root / "summary.json").is_file():
        summary = json.loads((root / "summary.json").read_text())
        if "query_geomean_pct" not in summary["variants"].get("pgo", {}):
            print("FAIL: passing run must record the pgo query geomean", file=sys.stderr)
            failures += 1

    # Truncated variant TSV (missing query) -> nonzero, no summary.
    root = make_results({"watdiv.tsv": "q1\t10\t90.0\nq2\t20\t180.0\n"})
    failures += check("truncated variant fails", root, expect_rc_zero=False, expect_summary=False)

    # Extra variant query -> nonzero, no summary.
    root = make_results({"watdiv.tsv": BASELINE_TSV + "q4\t40\t400.0\n"})
    failures += check("extra variant query fails", root, expect_rc_zero=False, expect_summary=False)

    # Wholly missing variant suite -> nonzero, no summary — even with a STALE
    # summary.json from a previous good run lying around.
    root = make_results({})
    (root / "summary.json").write_text('{"variants": {"pgo": {"query_geomean_pct": 99.0}}}\n')
    failures += check("missing variant suite fails + drops stale summary", root, expect_rc_zero=False, expect_summary=False)

    # Row-count drift on an identical shape -> nonzero, no summary.
    root = make_results({"watdiv.tsv": "q1\t10\t90.0\nq2\t21\t180.0\nq3\t30\t270.0\n"})
    failures += check("row-count mismatch fails", root, expect_rc_zero=False, expect_summary=False)

    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
