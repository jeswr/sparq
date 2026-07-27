#!/usr/bin/env python3
# [OPUS-5] sq-mztg8.3 (FO-KM epic; design research/fo-llm-bridge.md §4.2 Metric 3 /
# §6 Phase 6). 🤖 SPARQ agent — the deterministic grader for the Metric-3 probe.
#
# WHAT THIS IS
# --------------------------------------------------------------------------------
# Turns a file of raw per-session answers into the per-model / per-arm stability table
# in STABILITY.md. Like Metric 1's analyze.py it is DETERMINISTIC with NO model in the
# loop (design §5.2 anti-circularity): every number below is a count over a closed
# four-label vocabulary.
#
# INPUT — one JSON object per line (bench/fo-km/metric3-sessions.jsonl):
#   {"model": "<served model id>", "arm": "ungrounded|gufo|schema-org",
#    "session": <int>, "answer": "p01: OCCURRENT\np02: ...\n"}
# The RAW answer text is stored, not a pre-parsed label map, so the parse step is part
# of what a reader can audit rather than something the run already decided.
#
# THE FIVE REPORTED QUANTITIES
#   1. cs   — CROSS-SESSION CONTRADICTION RATE. The headline, and the direct analogue of
#             the Köhler-Neuhaus measure: over the probes answered in >= 2 sessions of a
#             cell, the fraction whose session labels are NOT unanimous. A model with a
#             stable ontological commitment scores 0.
#   2. dis  — DISSENT RATE, 1 - mean(modal count / sessions). The graded form of `cs`:
#             `cs` counts a probe with one dissenting session the same as a probe that
#             was answered five different ways, `dis` does not. Reported because with a
#             small K the binary measure saturates easily.
#   3. ws   — WITHIN-SESSION INCONSISTENCY. Over the fixture's generic/instance pairs,
#             the fraction of (pair, session) cells where a session puts a KIND in one
#             category and a MEMBER of that kind in another. This is a contradiction
#             inside a single session, so it cannot be explained by sampling across
#             sessions — it is the stronger of the two instability signals.
#   4. und  — UNDECIDABLE RATE (decisiveness guard). An arm can score a perfect `cs`
#             purely by answering UNDECIDABLE to everything. Without this column that
#             degenerate arm would read as the most stable. `cs_dec` re-computes the
#             contradiction rate over probes where the cell committed in EVERY session,
#             so a scaffold cannot buy stability by abstaining.
#   5. adh  — SCAFFOLD ADHERENCE. Over the SC probes for which the arm's own overlay
#             entails a label (stability_probes.jsonl `fo_label`), the fraction of
#             session answers equal to it. This is adherence to THAT ARM'S ontology, not
#             to any universal truth — the arms disagree with each other by design, so
#             the column is only ever read down a single arm's row.
# `cs`, `dis` and `adh` are also broken out per stratum (SC scaffolded / US unscaffolded),
# because the US stratum is the one an in-context FO cannot solve by lookup — it is where
# "does the scaffold TRANSFER" is actually answered.
#
# Usage:
#   python3 bench/fo-km/stability_analyze.py                       # self-check, no run
#   python3 bench/fo-km/stability_analyze.py bench/fo-km/metric3-sessions.jsonl
#   python3 bench/fo-km/stability_analyze.py <sessions> --json out.json

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from collections import Counter

HERE = os.path.dirname(os.path.abspath(__file__))
DEFAULT_PROBES = os.path.join(HERE, "stability_probes.jsonl")
DEFAULT_SESSIONS = os.path.join(HERE, "metric3-sessions.jsonl")

LABELS = ("OCCURRENT", "CONTINUANT", "ABSTRACT", "UNDECIDABLE")
ARMS = ("ungrounded", "gufo", "schema-org")
STRATA = ("SC", "US")

# `p07: CONTINUANT`, tolerating surrounding whitespace, a bullet, or bold markers — a
# formatting slip is not an ontological commitment, so it is parsed rather than scored
# as a contradiction. A line whose LABEL is outside the vocabulary is NOT tolerated: it
# is rejected as unparsed and shows up in the cell's `unparsed lines` note.
ANSWER_LINE = re.compile(r"^\s*[-*>\s]*\**\s*(p\d{2})\**\s*[:\-]\s*\**\s*([A-Za-z]+)\**\s*\.?\s*$")


def parse_answer(text: str, probe_ids: set[str]) -> tuple[dict[str, str], list[str]]:
    """Raw session text -> ({probe_id: LABEL}, [rejected lines]).

    Last write wins if a session repeats a probe id (it has restated its own answer;
    the final statement is the one it stands behind).
    """
    labels: dict[str, str] = {}
    rejected: list[str] = []
    for line in (text or "").splitlines():
        if not line.strip():
            continue
        m = ANSWER_LINE.match(line)
        if not m:
            rejected.append(line.strip())
            continue
        pid, lab = m.group(1), m.group(2).upper()
        if pid not in probe_ids or lab not in LABELS:
            rejected.append(line.strip())
            continue
        labels[pid] = lab
    return labels, rejected


def _rate(hits: int, total: int) -> float | None:
    return round(hits / total, 3) if total else None


def grade_cell(sessions: list[dict], probes: list[dict]) -> dict:
    """All the reported quantities for one (model, arm) cell."""
    by_id = {p["id"]: p for p in probes}
    parsed = [parse_answer(s.get("answer", ""), set(by_id)) for s in sessions]
    per_session = [p[0] for p in parsed]
    n_bad = sum(len(p[1]) for p in parsed)
    n_missing = sum(len(by_id) - len(ls) for ls in per_session)

    # --- 1/2. cross-session agreement, whole and per stratum ---------------------
    unstable: list[str] = []
    dissents: list[float] = []
    strat: dict[str, dict] = {k: {"unstable": [], "dissents": [], "n": 0} for k in STRATA}
    decided_unstable = decided_n = 0
    for pid, probe in by_id.items():
        obs = [ls[pid] for ls in per_session if pid in ls]
        if len(obs) < 2:
            continue  # a single observation cannot agree or disagree with anything
        counts = Counter(obs)
        is_unstable = len(counts) > 1
        dissent = 1.0 - counts.most_common(1)[0][1] / len(obs)
        if is_unstable:
            unstable.append(pid)
        dissents.append(dissent)
        st = strat[probe["kind"]]
        st["n"] += 1
        st["dissents"].append(dissent)
        if is_unstable:
            st["unstable"].append(pid)
        # decisive subset: every session committed to a real category
        if all(o != "UNDECIDABLE" for o in obs):
            decided_n += 1
            decided_unstable += int(is_unstable)

    n_probes_scored = len(dissents)

    # --- 3. within-session generic/instance consistency --------------------------
    seen_pairs = set()
    ws_hits = ws_total = 0
    for pid, probe in by_id.items():
        other = probe.get("pair")
        if not other:
            continue
        key = tuple(sorted((pid, other)))
        if key in seen_pairs:
            continue
        seen_pairs.add(key)
        for ls in per_session:
            if pid in ls and other in ls:
                ws_total += 1
                ws_hits += int(ls[pid] != ls[other])

    # --- 4. decisiveness ---------------------------------------------------------
    all_labels = [lab for ls in per_session for lab in ls.values()]
    n_und = sum(1 for lab in all_labels if lab == "UNDECIDABLE")

    # --- 5. scaffold adherence (per arm; None where the overlay entails nothing) --
    adh_hits = adh_total = 0
    adh_strat = {k: [0, 0] for k in STRATA}
    arm = sessions[0]["arm"] if sessions else None
    for pid, probe in by_id.items():
        gold = (probe.get("fo_label") or {}).get(arm)
        if gold is None:
            continue
        for ls in per_session:
            if pid in ls:
                adh_total += 1
                hit = int(ls[pid] == gold)
                adh_hits += hit
                adh_strat[probe["kind"]][0] += hit
                adh_strat[probe["kind"]][1] += 1

    return {
        "sessions": len(sessions),
        "probes_scored": n_probes_scored,
        "unparsed_lines": n_bad,
        "missing_answers": n_missing,
        "cs": _rate(len(unstable), n_probes_scored),
        "dissent": round(sum(dissents) / len(dissents), 3) if dissents else None,
        "ws": _rate(ws_hits, ws_total),
        "ws_n": ws_total,
        "undecidable": _rate(n_und, len(all_labels)),
        "cs_decided": _rate(decided_unstable, decided_n),
        "decided_probes": decided_n,
        "adherence": _rate(adh_hits, adh_total),
        "by_stratum": {
            k: {
                "n": strat[k]["n"],
                "cs": _rate(len(strat[k]["unstable"]), strat[k]["n"]),
                "dissent": (round(sum(strat[k]["dissents"]) / len(strat[k]["dissents"]), 3)
                            if strat[k]["dissents"] else None),
                "adherence": _rate(adh_strat[k][0], adh_strat[k][1]),
            }
            for k in STRATA
        },
        "unstable_probes": sorted(unstable),
        "label_mix": dict(Counter(all_labels).most_common()),
    }


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description="FO-KM Metric-3 stability grader")
    ap.add_argument("sessions", nargs="?", default=None,
                    help=f"session records jsonl (default {DEFAULT_SESSIONS}); omit to "
                         "self-check the probe battery + print the grading contract")
    ap.add_argument("--probes", default=DEFAULT_PROBES)
    ap.add_argument("--json", dest="json_out", help="write the full per-cell JSON here")
    args = ap.parse_args(argv[1:])

    probes = [json.loads(l) for l in open(args.probes, encoding="utf-8") if l.strip()]
    kinds = Counter(p["kind"] for p in probes)
    pairs = sum(1 for p in probes if p.get("pair")) // 2
    print(f"probes: {len(probes)} {dict(kinds)}, {pairs} generic/instance pairs, "
          f"labels {'/'.join(LABELS)}")

    if not args.sessions:
        print("\nNo session file given — self-check only. Grading contract:")
        print("  * cs        : probes whose labels are not unanimous across sessions")
        print("  * dissent   : 1 - mean(modal label count / sessions)")
        print("  * ws        : generic/instance pairs a SINGLE session labels differently")
        print("  * und       : share of answers that are UNDECIDABLE (decisiveness guard)")
        print("  * cs_dec    : cs restricted to probes the cell committed on every session")
        print("  * adh       : answers matching the arm's OWN overlay-entailed fo_label")
        print("\nNo model is in the loop: every quantity is a count over the closed "
              "label set.")
        return 0

    rows = [json.loads(l) for l in open(args.sessions, encoding="utf-8") if l.strip()]
    cells: dict[tuple[str, str], list[dict]] = {}
    for r in rows:
        cells.setdefault((r["model"], r["arm"]), []).append(r)
    print(f"sessions: {len(rows)} across {len(cells)} (model, arm) cells")

    graded = {f"{m}|{a}": grade_cell(ss, probes) for (m, a), ss in sorted(cells.items())}

    hdr = (f"\n{'model':26s} {'arm':11s} {'K':>2s} {'cs':>5s} {'dis':>5s} {'ws':>5s} "
           f"{'und':>5s} {'cs_dec':>6s} {'adh':>5s}   cs SC/US")
    print(hdr)
    for key, g in graded.items():
        model, arm = key.split("|")

        def f(v: float | None) -> str:
            return f"{v:.2f}" if v is not None else "  - "

        sc, us = g["by_stratum"]["SC"], g["by_stratum"]["US"]
        print(f"{model:26s} {arm:11s} {g['sessions']:>2d} {f(g['cs']):>5s} "
              f"{f(g['dissent']):>5s} {f(g['ws']):>5s} {f(g['undecidable']):>5s} "
              f"{f(g['cs_decided']):>6s} {f(g['adherence']):>5s}   "
              f"{f(sc['cs'])}/{f(us['cs'])}")
        if g["unparsed_lines"] or g["missing_answers"]:
            print(f"{'':26s} {'':11s}  (unparsed lines {g['unparsed_lines']}, "
                  f"missing answers {g['missing_answers']})")

    if args.json_out:
        with open(args.json_out, "w", encoding="utf-8") as fh:
            json.dump(graded, fh, indent=2, sort_keys=True)
        print(f"\nwrote {args.json_out}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
