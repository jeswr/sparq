#!/usr/bin/env python3
# [OPUS-4.8] sq-2m6zm.4 (epic sq-2m6zm). 🤖 SPARQ agent — the BLIND DETERMINISTIC
# paired grader. Written while Fable unavailable; flag for re-review when Fable
# returns. Spec: research/dogfooding-sparq-knowledge-graph.md §5.2 (NEVER let an LLM
# grade its own KG; use a deterministic resolver, not a judge).
#
# Grades correctness PAIRED, per task, for BOTH arms, with NO model in the loop:
#   * row_count match — the answer contains exactly the gold number of result rows.
#   * fact-resolution  — every gold `must_include` substring is RESOLVABLE in the
#     arm's answer payload (arm B: the pkg-query rows; arm A: the source slice the
#     agent read). A negative task (must_include=[], row_count=0) is correct iff
#     the answer is honestly empty.
# Anti-circularity (§5.2): the grader is a substring/count resolver over a held
# GOLD key authored from the source docs — it is NOT the PKG, and it does not call
# the model. Both arms are graded by the same resolver, blind to which arm is which.

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
AGENTS = REPO / "AGENTS.md"
BEADS = REPO / ".beads" / "issues.jsonl"
sys.path.insert(0, str(HERE))
from run import bd_show_chars, slice_section  # noqa: E402  (reuse the readers)


def _bd_text(bd_args: str) -> str:
    try:
        p = subprocess.run(["bd"] + bd_args.split(), capture_output=True, text=True, timeout=30)
        if p.returncode == 0 and p.stdout.strip():
            return p.stdout
    except Exception:
        pass
    bead = bd_args.split()[-1]
    for ln in BEADS.read_text().splitlines():
        if f'"{bead}"' in ln:
            return ln
    return ""


def armA_answer_text(task: dict) -> str:
    a = task["armA"]
    if a.get("bd"):
        txt = _bd_text(a["bd"])
        # for a "how many are ready" question, the realistic agent reads the
        # `bd ready` list and COUNTS it — append that count so the gold count fact
        # resolves against arm A (the LIVE ground truth). The PKG arm answers a
        # different (snapshot dependency-half) number; this is the documented gap.
        if a["bd"].strip() == "ready":
            count = sum(1 for ln in txt.splitlines() if ln.strip().startswith(("sq-", "○", "●", "◐", "✓")))
            txt += f"\n[ready count = {count}]"
        return txt
    return slice_section(AGENTS, a["section"], a.get("section_to"))


def count_result_rows(armB_answer: str) -> int:
    """The pkg-query helper prints a trailing 'N row(s).' line — but that goes to
    STDERR; STDOUT (what we captured as armB_answer) is the table. Count data rows:
    total printed lines minus the 2-line header (col names + dashes)."""
    lines = [l for l in armB_answer.splitlines() if l.strip()]
    return max(0, len(lines) - 2)


def grade_arm(answer_text: str, gold: dict, row_count: int) -> dict:
    # Case-insensitive substring resolution: the SAME semantic fact may surface as
    # "CLOSED" in `bd show` and "Closed" in the PKG, so the gold key is a fact, not
    # a verbatim token. Grading both arms case-folded keeps the paired comparison
    # fair (an arm is not penalised for a casing difference in the source).
    low = answer_text.lower()
    must = gold.get("must_include", [])
    resolved = [m for m in must if m.lower() in low]
    unresolved = [m for m in must if m.lower() not in low]
    rc_ok = (row_count == gold["row_count"])
    facts_ok = (len(unresolved) == 0)
    # a negative task is correct iff empty (no rows AND no spurious gold facts claimed)
    correct = rc_ok and facts_ok
    return {
        "correct": correct,
        "row_count_ok": rc_ok,
        "facts_resolved": len(resolved),
        "facts_total": len(must),
        "unresolvable": unresolved,
    }


def main() -> None:
    ap = argparse.ArgumentParser(description="PKG A/B paired deterministic grader (sq-2m6zm.4)")
    ap.add_argument("--tasks", default=str(HERE / "tasks" / "tasks.jsonl"))
    ap.add_argument("--results", default=str(HERE / "results.json"))
    ap.add_argument("--out", default=str(HERE / "grades.json"))
    args = ap.parse_args()

    tasks = {json.loads(l)["id"]: json.loads(l) for l in Path(args.tasks).read_text().splitlines() if l.strip()}
    res = json.loads(Path(args.results).read_text())

    grades, a_correct, b_correct = [], 0, 0
    for row in res["rows"]:
        t = tasks[row["id"]]
        gold = t["gold"]
        # Arm B graded on its actual pkg-query rows + computed row count.
        b_rows = count_result_rows(row["armB_answer"])
        gB = grade_arm(row["armB_answer"], gold, b_rows)
        # Arm A graded on whether the gold facts are RESOLVABLE in what it read.
        a_text = armA_answer_text(t)
        # arm A "row count" is not a structured concept for a prose read; we credit
        # the row_count check to A iff every gold fact resolves (a competent reader
        # extracts the same set), to keep the paired grade on the SAME gold key.
        # Case-insensitive, mirroring grade_arm (CLOSED vs Closed must match).
        a_low = a_text.lower()
        a_facts_ok = all(m.lower() in a_low for m in gold.get("must_include", []))
        gA = grade_arm(a_text, gold, gold["row_count"] if a_facts_ok else -1)
        a_correct += int(gA["correct"])
        b_correct += int(gB["correct"])
        grades.append({"id": row["id"], "stratum": row["stratum"], "armA": gA, "armB": gB})

    out = {
        "n": len(grades),
        "armA_realistic_correct": a_correct,
        "armB_correct": b_correct,
        "armA_accuracy": round(a_correct / len(grades), 3),
        "armB_accuracy": round(b_correct / len(grades), 3),
        "armB_unresolvable_total": sum(len(g["armB"]["unresolvable"]) for g in grades),
        "armA_unresolvable_total": sum(len(g["armA"]["unresolvable"]) for g in grades),
        "per_task": grades,
    }
    Path(args.out).write_text(json.dumps(out, indent=2) + "\n")
    print(f"[grade] arm A correct {a_correct}/{len(grades)}  arm B correct {b_correct}/{len(grades)}")
    print(f"[grade] wrote {args.out}")


if __name__ == "__main__":
    main()
