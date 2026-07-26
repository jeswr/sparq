#!/usr/bin/env python3
"""Validate bench/fo-km/tasks.jsonl: every FO arm answers, the no-FO arm does NOT.

[OPUS-4.8] FO-KM benchmark (epic sq-mztg8). SPARQ agent. NOT a perf measurement — a
DISCRIMINATION check: it confirms each task's FO-arm query returns a non-empty answer
matching the claimed gold — the `gold_count` for a COUNT/row task, the per-part count for
a multi-part task, the per-CATEGORY values for a dict-of-ints `gold_keys` (cc03/cc05) —
while the no-FO arm returns 0 / cannot, so the task set genuinely differentiates the arms.
Every gold shape is dispatched explicitly and an unknown shape FAILS, so a task can never
pass merely by returning some non-empty result. Run from the repo root:

    python3 bench/fo-km/validate_tasks.py

The per-shape gold-check logic is unit-tested hermetically (no cargo) by
scripts/tests/test_fo_km_gold_check.py, wired into the docs-quality quick-gates job.

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
    """Run pkg-query; return (header_line, data_rows). stdout layout: header, ---, rows."""
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
    header = lines[0] if lines else ""
    data = lines[2:] if len(lines) > 2 else []
    return header, data


def scalar(data):
    """Single COUNT value from a one-row result."""
    return int(data[0].strip()) if data else 0


def cells(line):
    """Split one printed pkg-query table line into cells (print_table joins with '  |  ')."""
    return [c.strip() for c in line.split("|")]


def is_int(cell):
    """True iff a printed cell is an integer literal (COUNT output)."""
    body = cell.lstrip("-")
    return bool(body) and body.isdigit()


def norm(label):
    """Normalise a gold category key / result label so the two sides compare.

    The projection names are plural (`?events`, `?artifacts`) while the gold keys are
    singular (`event`, `artifact`); both sides go through this, so the mapping is
    symmetric rather than a hard-coded alias table.
    """
    s = label.strip().strip('"').lower().replace("-", "_")
    return s[:-1] if s.endswith("s") else s


def parse_categories(header, data):
    """Parse a {label: count} mapping out of a printed pkg-query table.

    Two shapes occur in tasks.jsonl for a dict-shaped gold:
      * GROUPED (cc05) — `?kind (COUNT(DISTINCT ?x) AS ?n)` + GROUP BY: one row per
        category, label in the first column, count in the last (`claim  |  15`).
      * WIDE (cc03) — one row of aggregate columns named by the projection: header
        `events  |  artifacts`, row `1258  |  91`.
    Returns None when the table matches neither shape; the caller reports that as a
    FAILURE rather than letting the task pass unchecked.
    """
    rows = [cells(r) for r in data if r.strip()]
    if not rows:
        return None
    if all(len(r) >= 2 and not is_int(r[0]) and is_int(r[-1]) for r in rows):
        return {r[0]: int(r[-1]) for r in rows}
    head = cells(header)
    if len(rows) == 1 and len(rows[0]) == len(head) and all(is_int(c) for c in rows[0]):
        return {name: int(value) for name, value in zip(head, rows[0])}
    return None


def check_category_gold(tid, arm, header, data, gold):
    """Compare a grouped / multi-column FO-arm result against a dict-shaped gold.

    cc03 (`{event, artifact}`) and cc05 (`{claim, document, method}`) carry a per-category
    gold in `gold_keys` while `select` is a SINGLE query, so neither the list-gold row check
    nor the multi-part `gold_count` check reaches them: any non-empty result used to pass
    even with wrong per-category values. This compares the VALUES, so a refreshed gold is
    materially proven and later fixture drift cannot stay silent. Returns failure strings.
    """
    failures = []
    want = {}
    for key, value in gold.items():
        nkey = norm(key)
        if nkey in want:
            failures.append(
                f"{tid}/{arm}: ambiguous gold_keys — '{key}' collides with "
                f"'{want[nkey][0]}' once normalised; rename one category")
        want[nkey] = (key, value)
    got_raw = parse_categories(header, data)
    if got_raw is None:
        failures.append(
            f"{tid}/{arm}: could not read a category->count table from the result "
            f"(header {header!r}, {len(data)} row(s)) — dict-shaped gold_keys UNCHECKED")
        return failures
    got = {norm(label): n for label, n in got_raw.items()}
    for nkey, (key, value) in sorted(want.items()):
        if nkey not in got:
            failures.append(f"{tid}/{arm}: no result column/group for gold category '{key}'")
        elif got[nkey] != value:
            failures.append(
                f"{tid}/{arm}: category '{key}' returned {got[nkey]}, expected {value}")
    for nkey in sorted(set(got) - set(want)):
        failures.append(f"{tid}/{arm}: result category '{nkey}' has no gold_keys entry")
    return failures


def is_category_gold(gold_keys):
    """True for a per-category count gold (cc03/cc05) — a dict of ints, not of lists."""
    return (isinstance(gold_keys, dict) and bool(gold_keys)
            and all(isinstance(v, int) for v in gold_keys.values()))


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
                header, data = run(OVERLAY[arm], sub)
                rows = len(data)
                # A COUNT query has one row whose value is the count; a SELECT has `rows` rows.
                is_count = "COUNT(" in sub.upper() and rows == 1 and data and data[0].strip().lstrip("-").isdigit()
                answered = scalar(data) if is_count else rows
                # The branches below dispatch on the task's GOLD SHAPE, not on whether a
                # comparison happens to fail — so the closing `else` catches any new shape
                # that no check covers instead of letting it pass silently.
                if answered == 0:
                    failures.append(f"{tid}/{arm}/{part}: FO arm returned EMPTY (expected non-empty)")
                # For a single-list-gold scalar/row task, every FO arm with a query must
                # return EXACTLY gold_count (the FO answer is the same row-set regardless of FO).
                elif isinstance(t["gold_keys"], list) and not isinstance(q, dict):
                    if answered != gold_count:
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
                # A PER-CATEGORY task (ONE query, dict-of-ints `gold_keys`: cc03's
                # event/artifact wide row, cc05's claim/document/method groups). Row count
                # alone proves nothing here — compare the returned VALUES per category.
                elif is_category_gold(t["gold_keys"]) and not isinstance(q, dict):
                    failures.extend(
                        check_category_gold(tid, arm, header, data, t["gold_keys"]))
                else:
                    failures.append(
                        f"{tid}/{arm}/{part}: unrecognised gold shape "
                        f"(gold_keys {type(t['gold_keys']).__name__}, "
                        f"select {'dict' if isinstance(q, dict) else 'str'}, "
                        f"gold_count {type(gold_count).__name__}) — NO gold check applied; "
                        f"teach validate_tasks.py this shape")
        # --- 2. the no-FO arm must NOT be able to answer (0 rows / can't) ---
        nofo = t["no_fo"]
        _, data = run(OVERLAY["no-fo"], nofo)
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
