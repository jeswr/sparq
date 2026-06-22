#!/usr/bin/env python3
# [OPUS-4.8] sq-2m6zm.4 (epic sq-2m6zm). 🤖 SPARQ agent — Wilcoxon + bootstrap +
# break-even, emitting the §5.6 VERDICT OBJECT. Written while Fable unavailable;
# flag for re-review when Fable returns. Spec: research/dogfooding-sparq-knowledge-graph.md §5.
#
# Consumes results.json (deltas) + grades.json (quality) and applies the FROZEN
# PREREG.md bar + kill criteria mechanically. Stdlib-only (no numpy/scipy): an
# exact Wilcoxon signed-rank for small N with a normal-approx fallback, and a
# deterministic-seed median-delta bootstrap. The verdict is decided on the OBJECT,
# never on one number (§5.6). Every number printed is runtime-only / NON-CANONICAL.

from __future__ import annotations

import argparse
import json
import math
import random
import statistics
from itertools import combinations
from pathlib import Path

HERE = Path(__file__).resolve().parent

# Frozen PREREG.md thresholds (§5.1) — do not edit to fit a result.
MIN_RELATIVE_REDUCTION = 0.20  # >= 20% paired-median reduction
MAX_P = 0.05                   # p < 0.05
MIN_N = 30                     # >= 30 tasks
BOOTSTRAP_ITERS = 10000
BOOTSTRAP_SEED = 20260621      # deterministic


def wilcoxon_signed_rank(deltas: list[float]) -> tuple[float, float | None]:
    """Two-sided Wilcoxon signed-rank on the paired deltas (H0: median delta = 0).
    Returns (W, p). Zeros dropped (Wilcoxon convention). Exact null distribution by
    enumeration for n<=20; normal approximation with continuity correction above.
    Returns p=None when n<6 (the exact test has no power to reach p<0.05)."""
    nz = [d for d in deltas if d != 0]
    n = len(nz)
    if n == 0:
        return 0.0, 1.0
    ranks = _rank_abs(nz)
    w_plus = sum(r for d, r in zip(nz, ranks) if d > 0)
    w_minus = sum(r for d, r in zip(nz, ranks) if d < 0)
    w = min(w_plus, w_minus)
    if n < 6:
        return w, None  # underpowered — exact two-sided p cannot reach 0.05
    if n <= 20:
        # exact: enumerate all 2^n sign assignments of the ranks
        total = sum(ranks)
        count_le = 0
        all_assign = 0
        # enumerate via subset sums of the rank multiset
        from itertools import product

        for signs in product((0, 1), repeat=n):
            wp = sum(r for s, r in zip(signs, ranks) if s)
            stat = min(wp, total - wp)
            all_assign += 1
            if stat <= w:
                count_le += 1
        p = count_le / all_assign
        return w, min(1.0, p)
    # normal approximation
    mean_w = n * (n + 1) / 4
    var_w = n * (n + 1) * (2 * n + 1) / 24
    z = (w - mean_w + 0.5) / math.sqrt(var_w)
    p = 2 * (1 - _norm_cdf(abs(z)))
    return w, min(1.0, p)


def _rank_abs(xs: list[float]) -> list[float]:
    order = sorted(range(len(xs)), key=lambda i: abs(xs[i]))
    ranks = [0.0] * len(xs)
    i = 0
    while i < len(order):
        j = i
        while j + 1 < len(order) and abs(xs[order[j + 1]]) == abs(xs[order[i]]):
            j += 1
        avg = (i + 1 + j + 1) / 2  # average rank for ties
        for k in range(i, j + 1):
            ranks[order[k]] = avg
        i = j + 1
    return ranks


def _norm_cdf(x: float) -> float:
    return 0.5 * (1 + math.erf(x / math.sqrt(2)))


def bootstrap_median_ci(deltas: list[float], iters: int, seed: int) -> tuple[float, float]:
    rng = random.Random(seed)
    n = len(deltas)
    meds = []
    for _ in range(iters):
        sample = [deltas[rng.randrange(n)] for _ in range(n)]
        meds.append(statistics.median(sample))
    meds.sort()
    lo = meds[int(0.025 * iters)]
    hi = meds[int(0.975 * iters)]
    return lo, hi


def break_even(c_docread: float, c_query: float, c_ingest: float) -> tuple[float, bool]:
    """N* = C_ingest / (C_docread - C_query). Infinite if the denominator <= 0
    (per-query cost not cheaper than a doc read — the idea never pays back)."""
    denom = c_docread - c_query
    if denom <= 0:
        return float("inf"), True
    return c_ingest / denom, False


def main() -> None:
    ap = argparse.ArgumentParser(description="PKG A/B stats + verdict (sq-2m6zm.4)")
    ap.add_argument("--results", default=str(HERE / "results.json"))
    ap.add_argument("--grades", default=str(HERE / "grades.json"))
    ap.add_argument("--out", default=str(HERE / "verdict.json"))
    args = ap.parse_args()

    res = json.loads(Path(args.results).read_text())
    grades = json.loads(Path(args.grades).read_text())
    rows = res["rows"]
    n = len(rows)

    deltas = [r["delta_realistic"] for r in rows]          # binding baseline
    deltas_naive = [r["delta_naive"] for r in rows]
    eff_A = [r["eff_A_realistic"] for r in rows]
    eff_B = [r["eff_B"] for r in rows]

    med_delta = statistics.median(deltas)
    med_A = statistics.median(eff_A)
    rel_reduction = med_delta / med_A if med_A else 0.0
    w, p = wilcoxon_signed_rank(deltas)
    ci_lo, ci_hi = bootstrap_median_ci(deltas, BOOTSTRAP_ITERS, BOOTSTRAP_SEED)

    # break-even: mean per-task arm-A realistic read vs arm-B per-query (fresh only),
    # with the one-time ingest as C_ingest (all in effective-token proxy units).
    meta = res["meta"]
    import tokens as _t  # noqa: PLC0415

    c_docread = statistics.mean(eff_A)
    # arm B per-query marginal cost = fresh SPARQL+rows only (the amortised cache
    # part is the one-time bucket, not the marginal cost).
    import sys as _sys  # noqa: PLC0415

    _sys.path.insert(0, str(HERE))
    c_query_marginal = statistics.mean(
        [_t.effective(fresh_chars=r["armB_fresh_chars"]) for r in rows]
    )
    c_ingest = _t.effective(fresh_chars=0, cache_read_chars=meta["one_time_ingest_chars"])
    n_star, n_star_inf = break_even(c_docread, c_query_marginal, c_ingest)

    # --- frozen PREREG bar (§5.1) ---
    bar_relative = rel_reduction >= MIN_RELATIVE_REDUCTION
    bar_p = (p is not None) and (p < MAX_P)
    bar_n = n >= MIN_N
    bar_ci = ci_lo > 0
    token_win = bool(bar_relative and bar_p and bar_n and bar_ci)

    # --- quality (§5.2/§5.4 KILL 3) ---
    quality_non_regression = (
        grades["armB_correct"] >= grades["armA_realistic_correct"]
        and grades["armB_unresolvable_total"] <= grades["armA_unresolvable_total"]
    )

    # --- kill criteria (§5.4) ---
    kills = []
    if not token_win:
        reasons = []
        if not bar_relative:
            reasons.append(f"median reduction {rel_reduction:.1%} < 20%")
        if not bar_p:
            reasons.append("Wilcoxon p>=0.05 or underpowered (N<6)")
        if not bar_n:
            reasons.append(f"N={n} < 30 (underpowered by construction)")
        if not bar_ci:
            reasons.append(f"bootstrap median-delta CI [{ci_lo:.1f},{ci_hi:.1f}] includes 0")
        kills.append({"kill": "KILL_1_token", "fired": True, "why": "; ".join(reasons)})
    if n_star_inf:
        kills.append({"kill": "KILL_2_break_even", "fired": True, "why": "N* infinite (per-query >= doc-read)"})
    if not quality_non_regression:
        kills.append({"kill": "KILL_3_quality", "fired": True, "why": "arm B accuracy/resolvability regressed vs arm A"})

    recommend_adopt = bool(token_win and quality_non_regression and not n_star_inf)

    # per-stratum descriptive deltas
    strata: dict[str, list[float]] = {}
    for r in rows:
        strata.setdefault(r["stratum"], []).append(r["delta_realistic"])
    per_stratum = {
        s: {"n": len(d), "median_delta": round(statistics.median(d), 2),
            "min": round(min(d), 2), "max": round(max(d), 2)}
        for s, d in strata.items()
    }

    verdict = {
        # §5.6 verdict object
        "token_win": token_win,
        "token_delta_median_pct": round(rel_reduction * 100, 1),
        "token_delta_ci": [round(ci_lo, 2), round(ci_hi, 2)],
        "quality_delta": {
            "exec_acc": {
                "armA_realistic": grades["armA_accuracy"],
                "armB": grades["armB_accuracy"],
                "non_regression": grades["armB_correct"] >= grades["armA_realistic_correct"],
            },
            "provenance_completeness": {
                "armB_unresolvable_claims": grades["armB_unresolvable_total"],
                "armA_unresolvable_claims": grades["armA_unresolvable_total"],
            },
            "hallucination_rate": {
                "note": "read-payload harness measures resolvability, not generated-prose hallucination; "
                        "armB unresolvable claims is the proxy",
                "armB_unresolvable": grades["armB_unresolvable_total"],
            },
        },
        "break_even_N": (None if n_star_inf else round(n_star, 1)),
        "break_even_infinite": n_star_inf,
        "honest": True,
        "recommend_adopt": recommend_adopt,
        # supporting detail (non-canonical, runtime-only)
        "_detail": {
            "n_tasks": n,
            "median_delta_realistic_eff_tokens": round(med_delta, 2),
            "median_delta_naive_eff_tokens": round(statistics.median(deltas_naive), 2),
            "median_eff_A_realistic": round(med_A, 2),
            "median_eff_B": round(statistics.median(eff_B), 2),
            "wilcoxon_W": w,
            "wilcoxon_p": p,
            "frozen_bar": {
                "relative>=20%": bar_relative,
                "p<0.05": bar_p,
                "N>=30": bar_n,
                "ci_lo>0": bar_ci,
            },
            "per_stratum_delta_realistic": per_stratum,
            "kill_criteria": kills,
            "fidelity": meta["fidelity"],
            "proxy_chars_per_token": meta["chars_per_token_proxy"],
        },
    }
    Path(args.out).write_text(json.dumps(verdict, indent=2) + "\n")
    print(json.dumps(verdict, indent=2))


if __name__ == "__main__":
    main()
