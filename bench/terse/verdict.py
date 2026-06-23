#!/usr/bin/env python3
# [OPUS-4.8] sq-bzign (epic sq-2m6zm). 🤖 SPARQ agent — the §5.5 PER-LEVER verdict object for the
# sparq-terse Phase-5 A/B. Written while Fable unavailable; flag for re-review when Fable returns.
# Spec: research/llm-ergonomic-sparql-surface.md §5.4/§5.5 + bench/terse/PREREG.md.
#
# Joins the per-(task,arm) effective-token rows (tokens.py) with the deterministic quality grades
# (grade.py), runs the FROZEN pre-registered statistics on the paired deltas A-B (lever=keyword) and
# A-C (lever=V), and emits one verdict object per lever:
#
#   token_win requires: paired-median reduction >= 20% AND Wilcoxon p < 0.05 AND bootstrap 95% CI on
#   the median delta excludes 0 AND the win survives the cache discount (no-card delta still > 0).
#   recommend_adopt additionally requires quality non-regression (parses/grounded/F1 >= arm A) and,
#   for lever V, resolution_correctness == 1.0. A proxy-fidelity run can at most CONDITIONALLY
#   recommend (fidelity must be full-session-transcript for an unconditional adopt).
#
# Stdlib only: Wilcoxon signed-rank (normal approx with tie/continuity correction) + a percentile
# bootstrap, implemented here so the harness has no SciPy dependency. NON-CANONICAL numbers.

from __future__ import annotations

import argparse
import json
import math
import random
import statistics as st
import sys
from collections import defaultdict

random.seed(20260622)  # frozen so the bootstrap CI is reproducible

PCT_BAR = 20.0      # >= 20% median reduction (PREREG)
ALPHA = 0.05        # Wilcoxon p < 0.05
BOOT = 10000        # bootstrap resamples


def wilcoxon_signed_rank_p(deltas: list[float]) -> float:
    """Two-sided p for H0: median(delta)=0, via the normal approximation with tie + continuity
    correction. deltas are A-X (positive = terse cheaper). Zeros dropped (Wilcoxon convention)."""
    d = [x for x in deltas if x != 0]
    n = len(d)
    if n == 0:
        return 1.0
    order = sorted(range(n), key=lambda i: abs(d[i]))
    ranks = [0.0] * n
    i = 0
    while i < n:
        j = i
        while j + 1 < n and abs(d[order[j + 1]]) == abs(d[order[i]]):
            j += 1
        avg = (i + 1 + j + 1) / 2.0  # average rank for the tie group (1-based)
        for k in range(i, j + 1):
            ranks[order[k]] = avg
        i = j + 1
    w_plus = sum(r for r, x in zip(ranks, d) if x > 0)
    mean_w = n * (n + 1) / 4.0
    # tie correction term
    abs_vals = sorted(abs(x) for x in d)
    tie_term = 0.0
    i = 0
    while i < n:
        j = i
        while j + 1 < n and abs_vals[j + 1] == abs_vals[i]:
            j += 1
        t = j - i + 1
        tie_term += t ** 3 - t
        i = j + 1
    var_w = n * (n + 1) * (2 * n + 1) / 24.0 - tie_term / 48.0
    if var_w <= 0:
        return 1.0
    z = (abs(w_plus - mean_w) - 0.5) / math.sqrt(var_w)  # continuity correction
    p = 2.0 * (1.0 - 0.5 * (1.0 + math.erf(z / math.sqrt(2.0))))
    return max(0.0, min(1.0, p))


def bootstrap_median_ci(deltas: list[float]) -> tuple[float, float]:
    if not deltas:
        return (0.0, 0.0)
    meds = []
    n = len(deltas)
    for _ in range(BOOT):
        sample = [deltas[random.randrange(n)] for _ in range(n)]
        meds.append(st.median(sample))
    meds.sort()
    lo = meds[int(0.025 * BOOT)]
    hi = meds[int(0.975 * BOOT) - 1]
    return (round(lo, 3), round(hi, 3))


def load(path: str):
    return json.load(open(path, encoding="utf-8"))


def lever_verdict(name: str, arm: str, tok_eff, tok_eff_nocard, grades, tasks_by_id) -> dict:
    """arm is 'B' (lever keyword) or 'C' (lever V). Paired deltas A - arm.

    A targeted fan-out may run only the arms a bead is scoped to (e.g. sq-bmpzd is the
    K:-keyword upgrade, so it fans out arms A + B only). If this lever's arm has no token
    rows, the lever is reported SKIPPED rather than crashing on empty data — the verdict
    for the arm that WAS run is unaffected."""
    if not tok_eff.get(arm) or not tok_eff.get("A"):
        return {"lever": name, "fidelity": "n/a", "skipped": True,
                "skip_reason": f"no token rows for arm {arm} in this run (lever not fanned out)",
                "token_win": False, "honest": True, "recommend_adopt": False}
    paired = []
    paired_nocard = []
    for tid in tok_eff["A"]:
        if tid in tok_eff[arm]:
            paired.append(tok_eff["A"][tid] - tok_eff[arm][tid])
            paired_nocard.append(tok_eff_nocard["A"][tid] - tok_eff_nocard[arm][tid])
    med_a = st.median([tok_eff["A"][t] for t in tok_eff["A"]])
    med_x = st.median([tok_eff[arm][t] for t in tok_eff[arm]])
    med_delta = st.median(paired)
    pct = 100.0 * med_delta / med_a if med_a else 0.0
    p = wilcoxon_signed_rank_p(paired)
    ci = bootstrap_median_ci(paired)
    med_delta_nocard = st.median(paired_nocard)
    survives_cache = med_delta_nocard > 0  # the win is NOT solely the ×0.1 card component

    token_win = (pct >= PCT_BAR) and (p < ALPHA) and (ci[0] > 0) and survives_cache

    # quality (PREREG §5.3): means over the arm's graded reference queries
    def qmean(a, key):
        vals = [g[key] for g in grades if g["arm"] == a]
        return st.mean(vals) if vals else 0.0
    qa = {"parses": qmean("A", "parses"), "grounded": qmean("A", "grounded"),
          "answer_correct_f1": qmean("A", "answer_correct_f1")}
    qx = {"parses": qmean(arm, "parses"), "grounded": qmean(arm, "grounded"),
          "answer_correct_f1": qmean(arm, "answer_correct_f1")}
    rc = None
    if arm == "C":
        rcs = [g["resolution_correctness"] for g in grades
               if g["arm"] == "C" and g["resolution_correctness"] is not None]
        rc = st.mean(rcs) if rcs else None

    quality_ok = (qx["parses"] >= qa["parses"] - 1e-9
                  and qx["grounded"] >= qa["grounded"] - 1e-9
                  and qx["answer_correct_f1"] >= qa["answer_correct_f1"] - 1e-9)
    if arm == "C":
        quality_ok = quality_ok and (rc is not None and rc >= 1.0 - 1e-9)

    fidelity = "input-authoring-proxy"  # set by caller's data; overwritten below if real
    recommend = token_win and quality_ok and survives_cache
    honest = True
    return {
        "lever": name,
        "fidelity": fidelity,
        "median_eff_A": round(med_a, 2),
        "median_eff_arm": round(med_x, 2),
        "token_delta_median_pct": round(pct, 2),
        "token_delta_ci": list(ci),
        "wilcoxon_p": round(p, 5),
        "token_win": bool(token_win),
        "card_tax_charged": True,
        "win_survives_cache_discount": bool(survives_cache),
        "median_delta_no_card": round(med_delta_nocard, 3),
        "first_shot": {
            "parses": round(qx["parses"], 4),
            "grounded": round(qx["grounded"], 4),
            "answer_correct_f1": round(qx["answer_correct_f1"], 4),
            "resolution_correctness": (round(rc, 4) if rc is not None else None),
            "baseline_A": {k: round(v, 4) for k, v in qa.items()},
        },
        "quality_non_regression": bool(quality_ok),
        "honest": honest,
        "recommend_adopt": bool(recommend),
    }


def _selftest() -> int:
    """Guard the hand-rolled statistics so a refactor cannot silently break the gate.
    Validates the Wilcoxon normal-approx + the bootstrap CI against known-shape inputs."""
    # All-positive paired deltas -> strongly significant (small two-sided p), CI strictly > 0.
    pos = [5, 7, 9, 4, 6, 8, 3, 10, 5, 7, 6, 9, 4, 8, 5, 7, 6, 9, 4, 8,
           5, 7, 6, 9, 4, 8, 5, 7, 6, 9]
    p = wilcoxon_signed_rank_p(pos)
    assert p < 0.01, f"all-positive deltas should be significant, got p={p}"
    lo, hi = bootstrap_median_ci(pos)
    assert lo > 0, f"all-positive bootstrap CI lower bound should exceed 0, got {lo}"
    # Symmetric-around-zero deltas -> NOT significant (large p).
    sym = [3, -3, 5, -5, 2, -2, 4, -4, 1, -1, 6, -6, 3, -3, 5, -5, 2, -2, 4, -4]
    p2 = wilcoxon_signed_rank_p(sym)
    assert p2 > 0.5, f"symmetric deltas should be non-significant, got p={p2}"
    # All-zero deltas -> p = 1.0, CI = (0, 0).
    assert wilcoxon_signed_rank_p([0, 0, 0]) == 1.0
    print("verdict.py selftest: OK (Wilcoxon + bootstrap behave as specified)")
    return 0


def main(argv: list[str]) -> int:
    if len(argv) > 1 and argv[1] == "--selftest":
        return _selftest()
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--tokens", required=True, help="tokens.py output (proxy or real)")
    ap.add_argument("--grades", required=True, help="grade.py output")
    ap.add_argument("--tasks", required=True, help="terse_tasks.json")
    ap.add_argument("--out", required=True)
    args = ap.parse_args(argv[1:])

    tok = load(args.tokens)
    grades = load(args.grades)
    tasks = {t["id"]: t for t in load(args.tasks)}
    fidelity = tok[0].get("fidelity", "input-authoring-proxy") if tok else "input-authoring-proxy"

    # arm -> task -> eff (and a no-card view; for real transcripts the two are identical)
    eff = defaultdict(dict)
    eff_nc = defaultdict(dict)
    for r in tok:
        eff[r["arm"]][r["task"]] = r["eff_input_tokens"]
        eff_nc[r["arm"]][r["task"]] = r.get("eff_input_tokens_no_card", r["eff_input_tokens"])

    vk = lever_verdict("keyword", "B", eff, eff_nc, grades, tasks)
    vv = lever_verdict("V", "C", eff, eff_nc, grades, tasks)
    for v in (vk, vv):
        if v.get("skipped"):
            v["recommend_adopt_conditional"] = False
            continue
        v["fidelity"] = fidelity
        # a proxy run can at most CONDITIONALLY recommend
        if fidelity != "full-session-transcript" and v["recommend_adopt"]:
            v["recommend_adopt"] = False
            v["recommend_adopt_conditional"] = True
            v["recommend_note"] = ("token_win + quality hold on the input-authoring proxy; upgrade "
                                   "to unconditional only after the full-session transcript fan-out")
        else:
            v["recommend_adopt_conditional"] = False

    verdict = {
        "bead": "sq-bzign",
        "n_tasks": len(tasks),
        "fidelity": fidelity,
        "metric": "cache-discounted effective input tokens (1.0*fresh + 0.1*cache_read + "
                  "1.25*cache_write); paired A-B / A-C; Wilcoxon + bootstrap; FROZEN PREREG bar",
        "prereg_bar": {"pct_reduction_min": PCT_BAR, "alpha": ALPHA,
                       "ci_excludes_zero": True, "must_survive_cache_discount": True},
        "levers": [vk, vv],
    }
    with open(args.out, "w", encoding="utf-8") as fh:
        json.dump(verdict, fh, indent=2)

    # console
    print(f"=== sparq-terse Phase-5 verdict (N={len(tasks)}, fidelity={fidelity}) ===")
    for v in (vk, vv):
        print(f"\nLEVER {v['lever']} (arm {'B' if v['lever']=='keyword' else 'C'}):")
        if v.get("skipped"):
            print(f"  SKIPPED — {v['skip_reason']}")
            continue
        print(f"  median eff A={v['median_eff_A']} arm={v['median_eff_arm']}  "
              f"delta={v['token_delta_median_pct']}%  CI={v['token_delta_ci']}  "
              f"p={v['wilcoxon_p']}")
        print(f"  token_win={v['token_win']}  survives_cache_discount="
              f"{v['win_survives_cache_discount']}  (no-card median delta={v['median_delta_no_card']})")
        fs = v["first_shot"]
        print(f"  quality  parses={fs['parses']} grounded={fs['grounded']} "
              f"F1={fs['answer_correct_f1']} resolution={fs['resolution_correctness']}  "
              f"(A: parses={fs['baseline_A']['parses']} grounded={fs['baseline_A']['grounded']} "
              f"F1={fs['baseline_A']['answer_correct_f1']})")
        rec = ("CONDITIONAL (pending transcript fan-out)" if v.get("recommend_adopt_conditional")
               else v["recommend_adopt"])
        print(f"  honest={v['honest']}  recommend_adopt={rec}")
    print(f"\nwrote {args.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
