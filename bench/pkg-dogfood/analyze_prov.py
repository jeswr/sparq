#!/usr/bin/env python3
# [OPUS-5] sq-2489d.6 (epic sq-2489d, issue #3244). 🤖 SPARQ agent — Phase-7
# end-to-end A/B of the PROVENANCE-DRIVEN KB vs the INERT-PKG baseline.
#
# WHAT THIS IS
# --------------------------------------------------------------------------------
# `research/provenance-driven-genai-kb.md` §5 Phase 7 is the honest payoff test for
# the three capabilities Phases 1-5 landed: **citations** (sparq-nlq `citations`
# feature), **hedging/abstention** (`NlqConfig::qualify`), and
# **provenance-weighting** (sparq-vectors `WeightMode::Provenance`). Each shipped
# behind an on/off ablation with the explicit instruction to CLAIM NOTHING ahead of
# a measurement. This analyzer is that measurement's decision step: it emits ONE
# `research/dogfooding-sparq-knowledge-graph.md` §5.6 verdict object PER CAPABILITY
# so the maintainer keeps only what measurably pays.
#
# The baseline arm is `base` = the INERT PKG: the same 30 frozen tasks answered over
# the same graph with every capability OFF (no citations, no hedge, uniform triple
# weights). Every capability arm is compared PAIRWISE against `base` on the same
# task, so a task's intrinsic difficulty cancels.
#
# It reuses the pkg-dogfood harness rather than forking it:
#   * the frozen task set        -> tasks/abm_tasks.json (N=30, 4 strata)
#   * the real-transcript miner  -> tokens_real.py (cache-discounted eff. tokens)
#   * the graders + loaders      -> analyze3.py (gold-key coverage, NEG_RE abstention)
#   * the statistics + bars      -> stats.py (Wilcoxon, bootstrap CI, break-even, MIN_N)
#
# WHAT IT DELIBERATELY DOES NOT DO
# --------------------------------------------------------------------------------
# It does not RUN the arms. The A/B host (which model, on which box, with which
# capability build) is `needs-maintainer-steer` on this bead, and every verdict here
# is MODEL-DEPENDENT — issue #1111 requires a re-run under Fable before any verdict
# is treated as final. This file is the frozen decision procedure, pre-registered in
# PREREG-phase7.md; it reads transcripts + answers a host produced and returns a
# verdict. It never invents a number, and when the instrumentation a capability's
# claim depends on is absent it returns `honest=false` with a `blocked_reason`
# instead of guessing.
#
# Usage:
#   analyze_prov.py --tasks tasks/abm_tasks.json \
#                   --tokens tok_p7.json [...] \
#                   --answers answers-p7.json [...] \
#                   [--out verdict-phase7.json]
#
# --tokens are tokens_real.py rows tagged `[ABM task=<id> arm=base|cite|hedge|weight]`.
# --answers are the same map/record shapes analyze3.py accepts, and each record MAY
# carry the per-capability instrumentation fields documented in PREREG-phase7.md §4
# (`citations_emitted` / `citations_resolved` / `abstained`).

from __future__ import annotations

import argparse
import json
import statistics as st
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))

import analyze3  # noqa: E402  (sibling harness module; path set above)
import stats  # noqa: E402

BASE_ARM = "base"

# The canned queries whose result ORDER is confidence-driven (`ORDER BY DESC(?conf)`
# in crates/sparq-kb/src/query/canned.rs: FINDINGS_ABOUT, FINDING_PROVENANCE,
# SOURCE_STATUS). Only these can have their ranking moved by a provenance weight.
CONF_RANKED_QUERIES = ("findings-about", "finding-provenance", "source-status")

# --- the PRE-REGISTERED Phase-7 bar (frozen in PREREG-phase7.md; editing it to fit
# a result is forbidden). MIN_N / the token_win bar are IMPORTED from stats.py so
# this harness cannot silently weaken the shared §5 bar it claims to reuse.
MIN_N = stats.MIN_N
MAX_P = stats.MAX_P
BOOTSTRAP_ITERS = stats.BOOTSTRAP_ITERS
BOOTSTRAP_SEED = 20260729  # deterministic, distinct from the sq-2m6zm.4 seed
# Phase-7-specific ADDITIONAL gate: these capabilities are expected to COST tokens
# (a citation footnote is payload) and to pay on the outcome side. A capability may
# therefore spend up to this fraction of the baseline's median effective input
# tokens, provided it wins on the outcome metric.
MAX_TOKEN_OVERHEAD = 0.10


def _capabilities() -> dict[str, dict]:
    """The three Phase-1..4 capabilities, each with the arm label its host writes into
    the `[ABM ... arm=]` tag, the ablation switch that produces it, the task subset it
    is measured on, and the instrumentation its honesty claim depends on."""
    return {
        "citations": {
            "arm": "cite",
            "ablation": "sparq-nlq feature `citations` (Answer::citations)",
            # Citations are measured on ALL strata: on the negative stratum the
            # capability's correct behaviour is to emit ZERO citations rather than
            # invent a source, which is the sharpest fabrication test in the set.
            "relevant": lambda t: True,
            # The baseline emits no citations by construction, so only the arm is
            # required to report them.
            "requires": ("citations_emitted", "citations_resolved"),
            "requires_base": (),
            "on_agent_answer_path": True,
        },
        "hedging": {
            "arm": "hedge",
            "ablation": "sparq-nlq `NlqConfig::qualify` (Nlq::ask_qualified)",
            "relevant": lambda t: True,
            # Needed on BOTH arms: the confidently-wrong guard is a comparison.
            "requires": ("abstained",),
            "requires_base": ("abstained",),
            "on_agent_answer_path": True,
        },
        "provenance_weighting": {
            "arm": "weight",
            "ablation": "sparq-vectors `WeightMode::Provenance` (feature `structure`)",
            "relevant": lambda t: t.get("armB", {}).get("query") in CONF_RANKED_QUERIES,
            "requires": (),
            "requires_base": (),
            # w(t) feeds KGE training + per-Block fusion. The pkg-query answer path
            # this task set exercises is canned SPARQL, not vector retrieval, so on
            # THIS host the capability is not on the answer path and its
            # pre-registered metric stays the Phase-4 Hits@k/MRR ablation
            # (crates/sparq-vectors/examples/kge_ablation.rs).
            "on_agent_answer_path": False,
        },
    }


def _instrumentation(records: dict, task_ids: list[str], arm: str, field: str) -> list:
    """Collect one instrumentation field across a capability's tasks, or [] if the host
    did not report it for every task (partial instrumentation is not a measurement)."""
    vals = [records.get((t, arm), {}).get(field) for t in task_ids]
    return [] if any(v is None for v in vals) else vals


def _citation_metrics(records: dict, task_ids: list[str], arm: str) -> dict | None:
    emitted = _instrumentation(records, task_ids, arm, "citations_emitted")
    resolved = _instrumentation(records, task_ids, arm, "citations_resolved")
    if not emitted or not resolved:
        return None
    total_e, total_r = sum(emitted), sum(resolved)
    return {
        "citations_emitted": total_e,
        "citations_resolved": total_r,
        # Phase 1's metric: every emitted citation resolves to a real in-graph source.
        "resolution_rate": (total_r / total_e) if total_e else 1.0,
        "fabricated": total_e - total_r,
    }


def _abstention_metrics(records: dict, tasks: dict, task_ids: list[str], arm: str) -> dict | None:
    flags = _instrumentation(records, task_ids, arm, "abstained")
    if not flags:
        return None
    should = [tasks[t]["stratum"] == "negative" for t in task_ids]
    tp = sum(1 for a, s in zip(flags, should) if a and s)
    fp = sum(1 for a, s in zip(flags, should) if a and not s)
    fn = sum(1 for a, s in zip(flags, should) if not a and s)
    return {
        "abstentions": sum(1 for a in flags if a),
        "should_abstain": sum(1 for s in should if s),
        "precision": (tp / (tp + fp)) if (tp + fp) else 1.0,
        "recall": (tp / (tp + fn)) if (tp + fn) else 1.0,
        # Answering when the honest answer is "no rows" is the failure this
        # capability exists to remove; over-abstaining is its regression mode.
        "confidently_wrong": fn,
        "over_abstained": fp,
    }


def _paired(deltas: list[float]) -> dict:
    """Wilcoxon + bootstrap median CI over a paired delta list (empty-safe)."""
    if not deltas:
        return {"n": 0, "median": 0.0, "mean": 0.0, "wilcoxon_W": None,
                "wilcoxon_p": None, "ci": [0.0, 0.0]}
    w, p = stats.wilcoxon_signed_rank(deltas)
    lo, hi = stats.bootstrap_median_ci(deltas, BOOTSTRAP_ITERS, BOOTSTRAP_SEED)
    return {
        "n": len(deltas),
        "median": round(st.median(deltas), 4),
        "mean": round(st.fmean(deltas), 4),
        "wilcoxon_W": w,
        "wilcoxon_p": p,
        "ci": [round(lo, 4), round(hi, 4)],
    }


def verdict_for(
    name: str,
    cap: dict,
    tasks: dict,
    tok: dict,
    answers: dict,
    records: dict,
) -> dict:
    """The §5.6 verdict object for ONE capability, vs the inert-PKG `base` arm."""
    arm = cap["arm"]
    blocked: list[str] = []

    relevant = [t for t in sorted(tasks) if cap["relevant"](tasks[t])]
    # Only COMPLETE pairs count: a capability that answered a task the baseline
    # skipped (or vice versa) would bias the paired delta.
    paired_ids = [t for t in relevant if (t, BASE_ARM) in tok and (t, arm) in tok]
    if len(paired_ids) < len(relevant):
        blocked.append(
            f"{len(relevant) - len(paired_ids)} of {len(relevant)} relevant tasks lack a "
            f"complete ({BASE_ARM}, {arm}) transcript pair"
        )

    # --- tokens: delta = base - arm, so POSITIVE means the capability is cheaper.
    tok_deltas = [
        tok[(t, BASE_ARM)]["eff_input_tokens"] - tok[(t, arm)]["eff_input_tokens"]
        for t in paired_ids
    ]
    tok_stat = _paired(tok_deltas)
    base_median = st.median([tok[(t, BASE_ARM)]["eff_input_tokens"] for t in paired_ids]) \
        if paired_ids else 0.0
    rel_reduction = (tok_stat["median"] / base_median) if base_median else 0.0

    # --- outcome: gold-key coverage / honest-abstention, graded by the frozen
    # analyze3 grader. Delta = arm - base, so POSITIVE means the capability helped.
    out_deltas = [
        analyze3.grade(tasks[t], answers.get((t, arm)))
        - analyze3.grade(tasks[t], answers.get((t, BASE_ARM)))
        for t in paired_ids
    ]
    out_stat = _paired(out_deltas)

    # --- the shared §5 token_win bar, imported unchanged from stats.py.
    n_ok = len(paired_ids) >= MIN_N
    token_win = bool(
        rel_reduction >= stats.MIN_RELATIVE_REDUCTION
        and tok_stat["wilcoxon_p"] is not None
        and tok_stat["wilcoxon_p"] < MAX_P
        and n_ok
        and tok_stat["ci"][0] > 0
    )
    # --- the Phase-7 additional gate: a capability may COST tokens up to the
    # pre-registered ceiling if it wins on the outcome side.
    overhead = -rel_reduction  # positive == the capability spends more than base
    token_ok = bool(token_win or overhead <= MAX_TOKEN_OVERHEAD)

    outcome_win = bool(
        out_stat["median"] > 0
        and out_stat["wilcoxon_p"] is not None
        and out_stat["wilcoxon_p"] < MAX_P
        and n_ok
        and out_stat["ci"][0] > 0
    )

    # --- capability-specific honesty metrics (the claims Phases 1-2 must not break).
    quality: dict = {}
    honesty_ok = True
    for reported_arm, fields in ((arm, cap["requires"]), (BASE_ARM, cap["requires_base"])):
        for field in fields:
            if not _instrumentation(records, paired_ids, reported_arm, field):
                blocked.append(
                    f"host did not report `{field}` on arm `{reported_arm}` for every paired task"
                )
    if name == "citations":
        cm = _citation_metrics(records, paired_ids, arm)
        if cm is not None:
            quality["citation"] = cm
            # Phase 1's pre-registered metric: zero fabrication (equivalently, a
            # resolution rate of 1.0 — `fabricated == 0` is the same guard stated on
            # exact integers, so it is the one asserted).
            honesty_ok = honesty_ok and cm["fabricated"] == 0
    if name == "hedging":
        am = _abstention_metrics(records, tasks, paired_ids, arm)
        bm = _abstention_metrics(records, tasks, paired_ids, BASE_ARM)
        if am is not None:
            quality["abstention"] = am
            if bm is not None:
                quality["abstention_base"] = bm
                # Hedging must not answer confidently where the baseline abstained.
                honesty_ok = honesty_ok and am["confidently_wrong"] <= bm["confidently_wrong"]

    # break-even per §5.6: N* = one-time cost / per-task saving. Enabling a cargo
    # feature has no ingestion cost, so a token-saving capability pays back at once
    # and a token-spending one never does — the payoff is outcome-side by design.
    n_star, n_star_inf = stats.break_even(
        st.fmean([tok[(t, BASE_ARM)]["eff_input_tokens"] for t in paired_ids]) if paired_ids else 0.0,
        st.fmean([tok[(t, arm)]["eff_input_tokens"] for t in paired_ids]) if paired_ids else 0.0,
        0.0,
    )

    if not paired_ids:
        blocked.append(f"no ({BASE_ARM}, {arm}) transcript pairs found")
    elif not n_ok:
        blocked.append(f"N={len(paired_ids)} < {MIN_N} (underpowered by construction)")
    if not cap["on_agent_answer_path"]:
        blocked.append(
            "this capability is not on the agent answer path exercised by the frozen "
            "task set (canned SPARQL, not vector retrieval); its pre-registered metric "
            "is the Phase-4 Hits@k/MRR ablation, crates/sparq-vectors/examples/kge_ablation.rs"
        )

    honest = not blocked
    return {
        "capability": name,
        "arm": arm,
        "ablation": cap["ablation"],
        "baseline_arm": BASE_ARM,
        # --- §5.6 verdict object ---
        "token_win": token_win,
        "token_delta_median_pct": round(rel_reduction * 100, 1),
        "token_delta_ci": tok_stat["ci"],
        "quality_delta": {
            "exec_acc": {
                "median_outcome_delta": out_stat["median"],
                "mean_outcome_delta": out_stat["mean"],
                "ci": out_stat["ci"],
                "wilcoxon_p": out_stat["wilcoxon_p"],
                "non_regression": out_stat["ci"][0] >= 0,
            },
            "provenance_completeness": quality.get("citation", {}),
            "hallucination_rate": quality.get("abstention", {}),
        },
        "break_even_N": (None if n_star_inf else round(n_star, 1)),
        "break_even_infinite": n_star_inf,
        "honest": honest,
        "recommend_adopt": bool(honest and outcome_win and token_ok and honesty_ok),
        # --- supporting detail (NON-CANONICAL, runtime-only) ---
        "_detail": {
            "n_pairs": len(paired_ids),
            "n_relevant": len(relevant),
            "token_overhead_pct": round(overhead * 100, 1),
            "outcome_win": outcome_win,
            "token_ok": token_ok,
            "honesty_ok": honesty_ok,
            "tokens": tok_stat,
            "outcome": out_stat,
            "quality": quality,
            "frozen_bar": {
                "token_win_relative>=%d%%" % round(stats.MIN_RELATIVE_REDUCTION * 100): token_win,
                "max_token_overhead<=%d%%" % round(MAX_TOKEN_OVERHEAD * 100): token_ok,
                "p<%s" % MAX_P: out_stat["wilcoxon_p"],
                "N>=%d" % MIN_N: n_ok,
                "outcome_ci_lo>0": out_stat["ci"][0] > 0,
            },
            "blocked_reason": blocked,
        },
    }


def load_records(paths: list[str]) -> dict[tuple[str, str], dict]:
    """The per-(task, arm) answer RECORDS, keeping the instrumentation fields that
    `analyze3.load_answers` (which returns answer text only) drops on the floor."""
    out: dict[tuple[str, str], dict] = {}
    for p in paths:
        data = json.loads(Path(p).read_text(encoding="utf-8"))
        if isinstance(data, dict):
            data = data.get("result", {}).get("results", []) if "result" in data else []
        for r in data:
            if isinstance(r, dict) and r.get("task") and r.get("arm"):
                out[(r["task"], r["arm"])] = r
    return out


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description="Phase-7 provenance-KB vs inert-PKG A/B (sq-2489d.6)")
    ap.add_argument("--tasks", default=str(HERE / "tasks" / "abm_tasks.json"))
    ap.add_argument("--tokens", nargs="+", required=True, help="tokens_real.py output JSON(s)")
    ap.add_argument("--answers", nargs="+", required=True, help="answer map/record JSON(s)")
    ap.add_argument("--out", default=None, help="write the per-capability verdict object here")
    args = ap.parse_args(argv[1:])

    tasks = {t["id"]: t for t in json.loads(Path(args.tasks).read_text(encoding="utf-8"))}
    tok = analyze3.load_tokens(args.tokens)
    answers = analyze3.load_answers(args.answers)
    records = load_records(args.answers)

    verdicts = {
        name: verdict_for(name, cap, tasks, tok, answers, records)
        for name, cap in _capabilities().items()
    }

    print("=== PHASE-7 VERDICT: provenance-driven KB vs inert-PKG baseline ===")
    print("all numbers NON-CANONICAL (work-box, one host, model-dependent; re-run "
          "under Fable per issue #1111 before treating any verdict as final)\n")
    print(f"{'capability':22} {'N':>4} {'tok Δ%':>8} {'outcome Δ':>10} {'honest':>7} {'adopt':>6}")
    for name, v in verdicts.items():
        print(f"{name:22} {v['_detail']['n_pairs']:>4} {v['token_delta_median_pct']:>8.1f} "
              f"{v['quality_delta']['exec_acc']['median_outcome_delta']:>10.3f} "
              f"{str(v['honest']):>7} {str(v['recommend_adopt']):>6}")
    for name, v in verdicts.items():
        for why in v["_detail"]["blocked_reason"]:
            print(f"  ! {name}: {why}")

    out = {"bead": "sq-2489d.6", "epic": "sq-2489d", "capabilities": verdicts}
    if args.out:
        Path(args.out).write_text(json.dumps(out, indent=2) + "\n", encoding="utf-8")
        print(f"\nwrote {args.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
