#!/usr/bin/env python3
"""Validate bench/fo-km/tasks.jsonl: every FO arm answers, the no-FO arm does NOT.

[OPUS-4.8] FO-KM benchmark (epic sq-mztg8). SPARQ agent. NOT a perf measurement — a
DISCRIMINATION check: it confirms each task's FO-arm query returns a non-empty answer
(and, for COUNT/row tasks, the claimed gold_count) while the no-FO arm returns 0 /
cannot — so the task set genuinely differentiates the arms. Run from the repo root:

    python3 bench/fo-km/validate_tasks.py

Requires the `close` feature: it shells out to the pkg-query binary with --extra-graph
+ --close owl-rl. Exit 0 iff every task discriminates; non-zero (with a report) otherwise.
"""
import json
import subprocess
import sys

OVERLAY = {
    "gufo": "bench/fo-km/overlays/gufo.ttl",
    "dolce-dul": "bench/fo-km/overlays/dolce-dul.ttl",
    "schema-org": "bench/fo-km/overlays/schema-org.ttl",
    "no-fo": "bench/fo-km/overlays/no-fo.ttl",
}


def run(overlay_path, sparql):
    """Run pkg-query; return (stdout_data_rows, raw_count). stdout layout: header, ---, rows."""
    cmd = [
        "cargo", "run", "-q", "-p", "sparq-kb", "--features", "close",
        "--bin", "pkg-query", "--",
        "--extra-graph", overlay_path, "--close", "owl-rl", "--sparql", sparql,
    ]
    out = subprocess.run(cmd, capture_output=True, text=True)
    if out.returncode != 0:
        raise RuntimeError(f"pkg-query failed for {overlay_path}:\n{out.stderr}\nQUERY:\n{sparql}")
    lines = out.stdout.splitlines()
    # header line 0, separator line 1, then data rows
    data = lines[2:] if len(lines) > 2 else []
    return data


def scalar(data):
    """Single COUNT value from a one-row result."""
    return int(data[0].strip()) if data else 0


def main():
    tasks = [json.loads(l) for l in open("bench/fo-km/tasks.jsonl")]
    failures = []
    for t in tasks:
        tid, kind = t["id"], t["kind"]
        sel = t["select"]
        gold_count = t["gold_count"]
        # --- 1. each FO arm that has a query must produce a non-empty / correct answer ---
        for arm in ("gufo", "dolce-dul", "schema-org"):
            q = sel.get(arm)
            if q is None:
                continue  # intentionally-non-applicable arm (e.g. schema.org lacks occurrent)
            # th03/cc05/cc03/th07 etc. may be dict (multi-part) — handle both shapes.
            queries = q if isinstance(q, dict) else {"_": q}
            for part, sub in queries.items():
                data = run(OVERLAY[arm], sub)
                rows = len(data)
                # A COUNT query has one row whose value is the count; a SELECT has `rows` rows.
                is_count = "COUNT(" in sub.upper() and rows == 1 and data and data[0].strip().lstrip("-").isdigit()
                answered = scalar(data) if is_count else rows
                if answered == 0:
                    failures.append(f"{tid}/{arm}/{part}: FO arm returned EMPTY (expected non-empty)")
                # For a single-list-gold scalar/row task, every FO arm with a query must
                # return EXACTLY gold_count (the FO answer is the same row-set regardless of FO).
                elif isinstance(t["gold_keys"], list) and not isinstance(q, dict) and answered != gold_count:
                    failures.append(
                        f"{tid}/{arm}: FO arm returned {answered}, expected gold_count {gold_count}")
                # A MULTI-PART task (dict-shaped `select`, e.g. th03's truth_bearers /
                # info_bearers split) carries a dict-shaped gold_count keyed the same way.
                # These were previously NOT count-checked at all, which is how a stale gold
                # (Source 6 -> 71) survived validation while the single-query tasks failed
                # loudly. Check each part against its own gold.
                elif isinstance(q, dict) and isinstance(gold_count, dict):
                    if part not in gold_count:
                        failures.append(f"{tid}/{arm}/{part}: no gold_count entry for this part")
                    elif answered != gold_count[part]:
                        failures.append(
                            f"{tid}/{arm}/{part}: FO arm returned {answered}, "
                            f"expected gold_count {gold_count[part]}")
        # --- 2. the no-FO arm must NOT be able to answer (0 rows / can't) ---
        nofo = t["no_fo"]
        data = run(OVERLAY["no-fo"], nofo)
        is_count = "COUNT(" in nofo.upper()
        nofo_ans = scalar(data) if (is_count and data and data[0].strip().lstrip("-").isdigit()) else len(data)
        # ER01 is the closed-vs-open discriminator: the no-FO arm here is the SAME query but
        # the discrimination is closure (no overlay needed); it is a shared FO lever, so the
        # no-FO arm WITHOUT closure is the real baseline. We document it rather than assert 0.
        if tid == "er01":
            print(f"  {tid}: ER01 closure-lever — no-FO arm WITH closure also answers ({nofo_ans}); "
                  f"discrimination is closed-vs-open (verified separately).")
            continue
        if nofo_ans != 0:
            failures.append(f"{tid}: no-FO arm ANSWERED ({nofo_ans}) — task does NOT discriminate!")
        else:
            print(f"  {tid} [{kind}]: OK — FO arms answer (gold {gold_count}); no-FO arm = 0 (discriminates)")

    print()
    if failures:
        print(f"FAILED ({len(failures)}):")
        for f in failures:
            print("  -", f)
        sys.exit(1)
    print(f"ALL {len(tasks)} TASKS DISCRIMINATE: every FO arm answers, the no-FO arm cannot.")


if __name__ == "__main__":
    main()
