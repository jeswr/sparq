#!/usr/bin/env python3
"""Hermetic tests for the Phase-7 provenance-KB A/B analyzer (`bench/pkg-dogfood/analyze_prov.py`).

The analyzer's whole value is that it REFUSES to recommend a capability the evidence does
not support, so every guard in `recommend_adopt` / `honest` has a NAMED test here that
goes red when that guard is deleted or inverted:

* ``TestRecommendGate``    — one test per conjunct of ``recommend_adopt``.
* ``TestHonestyGate``      — one test per blocking reason that sets ``honest = false``.
* ``TestVerdictShape``     — the §5.6 verdict object's keys and sign conventions.
* ``TestSharedBarsReused`` — the shared §5 bars are IMPORTED from ``stats.py``, not
  re-declared, so this harness cannot silently weaken the bar it claims to reuse.
* ``TestArmTagBroadening`` — ``tokens_real.py`` mines the Phase-7 named arms without
  losing the original ``A``/``B``/``C`` attribution.

Stdlib unittest. No network, no gh, no git. Fixtures are synthetic transcripts built over
the REAL frozen task set, so a task-set edit that breaks the analyzer reds here.
"""

# [OPUS-5] sq-2489d.6 (epic sq-2489d, issue #3244). 🤖 SPARQ agent.

from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path
from types import ModuleType

REPO_ROOT = Path(__file__).resolve().parents[2]
BENCH = REPO_ROOT / "bench" / "pkg-dogfood"
TASKS_PATH = BENCH / "tasks" / "abm_tasks.json"


def _load(name: str, path: Path) -> ModuleType:
    if str(BENCH) not in sys.path:
        sys.path.insert(0, str(BENCH))
    spec = importlib.util.spec_from_file_location(name, path)
    mod = importlib.util.module_from_spec(spec)
    sys.modules[name] = mod
    spec.loader.exec_module(mod)
    return mod


ap = _load("analyze_prov", BENCH / "analyze_prov.py")
tokens_real = _load("tokens_real", BENCH / "tokens_real.py")
stats_mod = _load("stats", BENCH / "stats.py")
TASKS = {t["id"]: t for t in json.loads(TASKS_PATH.read_text(encoding="utf-8"))}


def answer_for(task: dict, *, good: bool) -> str:
    """A synthetic answer that the frozen `analyze3.grade` scorer grades 1.0 (good) or
    strictly worse (bad), using the task's own gold keys."""
    if task["stratum"] == "negative":
        return "none — 0 rows" if good else "here are the three matching rows: alpha beta"
    gold = task["gold_keys"]
    keys = gold if good else gold[:1]
    return "answer mentioning " + " and ".join(keys)


def fixture(
    *,
    arm: str,
    outcome_win: bool = True,
    token_overhead: float = 0.05,
    cite_fabricated: int = 0,
    instrument: bool = True,
    n_tasks: int | None = None,
    drop_pair_for: str | None = None,
    base_confidently_wrong: int = 0,
) -> tuple[dict, dict, dict]:
    """Build (tok, answers, records) for a `base` vs `arm` run over the frozen task set."""
    ids = sorted(TASKS)[: n_tasks if n_tasks is not None else len(TASKS)]
    tok: dict[tuple[str, str], dict] = {}
    answers: dict[tuple[str, str], str] = {}
    records: dict[tuple[str, str], dict] = {}
    base_eff = 1000.0
    # `confidently_wrong` on base = a negative-stratum task base failed to abstain on.
    negatives = [t for t in ids if TASKS[t]["stratum"] == "negative"]
    base_wrong = set(negatives[:base_confidently_wrong])
    for i, t in enumerate(ids):
        if t == drop_pair_for:
            continue
        eff_base = base_eff + i  # slight spread so the paired stats are not degenerate
        tok[(t, "base")] = {"eff_input_tokens": eff_base, "output_tokens": 100}
        tok[(t, arm)] = {"eff_input_tokens": eff_base * (1.0 + token_overhead), "output_tokens": 100}
        answers[(t, "base")] = answer_for(TASKS[t], good=False)
        answers[(t, arm)] = answer_for(TASKS[t], good=outcome_win)
        records[(t, "base")] = {"task": t, "arm": "base"}
        records[(t, arm)] = {"task": t, "arm": arm}
        if instrument:
            emitted = 2
            records[(t, arm)]["citations_emitted"] = emitted
            records[(t, arm)]["citations_resolved"] = emitted
            records[(t, arm)]["abstained"] = TASKS[t]["stratum"] == "negative"
            records[(t, "base")]["abstained"] = t not in base_wrong and TASKS[t]["stratum"] == "negative"
    if cite_fabricated and instrument:
        first = next(t for t in ids if (t, arm) in records)
        records[(first, arm)]["citations_resolved"] -= cite_fabricated
    return tok, answers, records


def verdict(name: str, **kw) -> dict:
    cap = ap._capabilities()[name]
    tok, answers, records = fixture(arm=cap["arm"], **kw)
    return ap.verdict_for(name, cap, TASKS, tok, answers, records)


class TestVerdictShape(unittest.TestCase):
    def test_every_capability_emits_a_5_6_verdict_object(self):
        required = {
            "token_win", "token_delta_median_pct", "token_delta_ci", "quality_delta",
            "break_even_N", "break_even_infinite", "honest", "recommend_adopt",
        }
        for name in ap._capabilities():
            v = verdict(name)
            self.assertTrue(required <= set(v), f"{name} missing {required - set(v)}")
            self.assertEqual(
                {"exec_acc", "provenance_completeness", "hallucination_rate"},
                set(v["quality_delta"]),
            )

    def test_the_three_landed_capabilities_are_the_ones_measured(self):
        self.assertEqual(
            {"citations", "hedging", "provenance_weighting"}, set(ap._capabilities())
        )

    def test_token_delta_is_positive_when_the_capability_is_cheaper(self):
        v = verdict("citations", token_overhead=-0.20)
        self.assertGreater(v["token_delta_median_pct"], 0)
        self.assertLess(v["_detail"]["token_overhead_pct"], 0)

    def test_token_delta_is_negative_when_the_capability_costs_more(self):
        v = verdict("citations", token_overhead=0.05)
        self.assertLess(v["token_delta_median_pct"], 0)

    def test_a_token_spending_capability_never_breaks_even(self):
        v = verdict("citations", token_overhead=0.05)
        self.assertTrue(v["break_even_infinite"])
        self.assertIsNone(v["break_even_N"])


class TestRecommendGate(unittest.TestCase):
    """One test per conjunct of `recommend_adopt = honest AND outcome_win AND token_ok
    AND honesty_ok`. Deleting or inverting any conjunct reds one of these."""

    def test_a_clean_outcome_win_within_the_token_ceiling_is_adopted(self):
        # The acceptance criterion of sq-2489d.6: `honest=true AND recommend=true` is
        # REACHABLE for a capability that actually pays.
        for name in ("citations", "hedging"):
            v = verdict(name)
            self.assertTrue(v["honest"], f"{name}: {v['_detail']['blocked_reason']}")
            self.assertTrue(v["recommend_adopt"], name)

    def test_no_outcome_win_is_not_adopted_however_cheap(self):
        v = verdict("citations", outcome_win=False, token_overhead=-0.50)
        self.assertFalse(v["_detail"]["outcome_win"])
        self.assertFalse(v["recommend_adopt"])

    def test_token_overhead_above_the_ceiling_is_not_adopted_despite_an_outcome_win(self):
        over = ap.MAX_TOKEN_OVERHEAD + 0.05
        v = verdict("citations", token_overhead=over)
        self.assertTrue(v["_detail"]["outcome_win"])
        self.assertFalse(v["_detail"]["token_ok"])
        self.assertFalse(v["recommend_adopt"])

    def test_token_overhead_at_the_ceiling_is_still_adopted(self):
        v = verdict("citations", token_overhead=ap.MAX_TOKEN_OVERHEAD)
        self.assertTrue(v["_detail"]["token_ok"])
        self.assertTrue(v["recommend_adopt"])

    def test_one_fabricated_citation_kills_adoption(self):
        v = verdict("citations", cite_fabricated=1)
        self.assertEqual(1, v["quality_delta"]["provenance_completeness"]["fabricated"])
        self.assertLess(v["quality_delta"]["provenance_completeness"]["resolution_rate"], 1.0)
        self.assertFalse(v["_detail"]["honesty_ok"])
        self.assertFalse(v["recommend_adopt"])

    def test_hedging_that_answers_confidently_where_base_abstained_is_not_adopted(self):
        # base abstains on every negative task; the hedge arm is then made to answer
        # one of them, so its confidently-wrong count exceeds the baseline's.
        cap = ap._capabilities()["hedging"]
        tok, answers, records = fixture(arm="hedge")
        neg = next(t for t in sorted(TASKS) if TASKS[t]["stratum"] == "negative")
        records[(neg, "hedge")]["abstained"] = False
        v = ap.verdict_for("hedging", cap, TASKS, tok, answers, records)
        self.assertGreater(
            v["quality_delta"]["hallucination_rate"]["confidently_wrong"],
            v["_detail"]["quality"]["abstention_base"]["confidently_wrong"],
        )
        self.assertFalse(v["_detail"]["honesty_ok"])
        self.assertFalse(v["recommend_adopt"])

    def test_abstention_precision_and_recall_are_scored_against_the_negative_stratum(self):
        v = verdict("hedging")
        m = v["quality_delta"]["hallucination_rate"]
        self.assertEqual(
            sum(1 for t in TASKS.values() if t["stratum"] == "negative"), m["should_abstain"]
        )
        self.assertEqual(1.0, m["precision"])
        self.assertEqual(1.0, m["recall"])


class TestHonestyGate(unittest.TestCase):
    """One test per blocking reason. `honest = false` must never coexist with a
    recommendation, and must always carry a stated reason."""

    def test_dishonest_verdicts_are_never_recommended(self):
        for kw in ({"n_tasks": 8}, {"instrument": False}, {"drop_pair_for": sorted(TASKS)[0]}):
            v = verdict("citations", **kw)
            self.assertFalse(v["honest"], kw)
            self.assertFalse(v["recommend_adopt"], kw)
            self.assertTrue(v["_detail"]["blocked_reason"], kw)

    def test_a_complete_but_underpowered_run_is_still_blocked(self):
        # Every pair present, nothing missing — the ONLY defect is that the run is
        # under `MIN_N`. Isolates the task-count guard from the pairing guard.
        n = ap.MIN_N - 1
        subset = {t: TASKS[t] for t in sorted(TASKS)[:n]}
        cap = ap._capabilities()["citations"]
        # A large token saving and a clean outcome win, so `MIN_N` is the only thing
        # standing between this run and a recommendation.
        tok, answers, records = fixture(
            arm=cap["arm"], n_tasks=n,
            token_overhead=-(stats_mod.MIN_RELATIVE_REDUCTION + 0.1),
        )
        v = ap.verdict_for("citations", cap, subset, tok, answers, records)
        self.assertEqual(n, v["_detail"]["n_pairs"])
        self.assertEqual(n, v["_detail"]["n_relevant"])
        self.assertEqual(
            [f"N={n} < {ap.MIN_N} (underpowered by construction)"],
            v["_detail"]["blocked_reason"],
        )
        self.assertFalse(v["honest"])
        self.assertFalse(v["recommend_adopt"])
        # `N >= MIN_N` is a conjunct of BOTH win predicates, not only of `honest`.
        self.assertFalse(v["_detail"]["outcome_win"])
        self.assertFalse(v["token_win"])

    def test_missing_capability_instrumentation_is_blocked_not_defaulted(self):
        v = verdict("citations", instrument=False)
        self.assertFalse(v["honest"])
        self.assertTrue(
            any("citations_emitted" in r for r in v["_detail"]["blocked_reason"]),
            v["_detail"]["blocked_reason"],
        )
        # ...and no citation metric is invented from the missing field.
        self.assertEqual({}, v["quality_delta"]["provenance_completeness"])

    def test_hedging_requires_the_abstention_flag_on_both_arms(self):
        cap = ap._capabilities()["hedging"]
        tok, answers, records = fixture(arm="hedge")
        for t in sorted(TASKS):
            records[(t, "base")].pop("abstained")
        v = ap.verdict_for("hedging", cap, TASKS, tok, answers, records)
        self.assertFalse(v["honest"])
        self.assertTrue(any("`base`" in r for r in v["_detail"]["blocked_reason"]))

    def test_an_incomplete_arm_pair_is_blocked(self):
        dropped = sorted(TASKS)[0]
        v = verdict("citations", drop_pair_for=dropped)
        self.assertFalse(v["honest"])
        self.assertEqual(len(TASKS) - 1, v["_detail"]["n_pairs"])
        # The unpaired task must be named as its own reason — not merely swept up by
        # the task-count guard, which would hide the bias an unpaired arm introduces.
        self.assertIn(
            f"1 of {len(TASKS)} relevant tasks lack a complete (base, cite) transcript pair",
            v["_detail"]["blocked_reason"],
        )

    def test_a_missing_answer_on_either_arm_is_blocked_not_graded_as_zero(self):
        # A transcript pair is not an ANSWER pair: `analyze3.grade(task, None)` returns
        # 0.0, so an ungraded task must leave the pairing rather than be scored zero.
        for missing_arm in ("base", "cite"):
            cap = ap._capabilities()["citations"]
            tok, answers, records = fixture(arm=cap["arm"])
            del answers[(sorted(TASKS)[0], missing_arm)]
            v = ap.verdict_for("citations", cap, TASKS, tok, answers, records)
            self.assertFalse(v["honest"], missing_arm)
            self.assertFalse(v["recommend_adopt"], missing_arm)
            self.assertEqual(len(TASKS) - 1, v["_detail"]["n_pairs"], missing_arm)
            self.assertIn(
                f"1 of {len(TASKS)} transcript-paired tasks lack a complete "
                f"(base, cite) answer pair",
                v["_detail"]["blocked_reason"],
            )

    def test_absent_baseline_answers_cannot_manufacture_an_outcome_win(self):
        # The exact bias this guard exists to stop, and the one the token-pair guard
        # does NOT catch: every token pair present, every arm answer and record present,
        # but the baseline was never answered. Grading the absent baselines as 0.0 gave
        # the arm a maximal, significant `outcome_win` and `recommend_adopt=true`.
        # Citations require no baseline instrumentation, so nothing else blocks it.
        cap = ap._capabilities()["citations"]
        tok, answers, records = fixture(arm=cap["arm"])
        for t in sorted(TASKS):
            del answers[(t, "base")]
        v = ap.verdict_for("citations", cap, TASKS, tok, answers, records)
        self.assertEqual(0, v["_detail"]["n_pairs"])
        self.assertEqual(len(TASKS), v["_detail"]["n_relevant"])
        self.assertFalse(v["_detail"]["outcome_win"])
        self.assertFalse(v["honest"])
        self.assertFalse(v["recommend_adopt"])

    def test_a_null_answer_counts_as_missing_not_as_a_wrong_answer(self):
        # `analyze3.load_answers` yields `None` for a record whose `answer` key is absent;
        # that is missing evidence, not a scored-zero response.
        cap = ap._capabilities()["citations"]
        tok, answers, records = fixture(arm=cap["arm"])
        answers[(sorted(TASKS)[0], "base")] = None
        v = ap.verdict_for("citations", cap, TASKS, tok, answers, records)
        self.assertEqual(len(TASKS) - 1, v["_detail"]["n_pairs"])
        self.assertFalse(v["honest"])
        self.assertFalse(v["recommend_adopt"])

    def test_provenance_weighting_is_reported_off_the_agent_answer_path(self):
        # It is measured by the Phase-4 Hits@k/MRR ablation, not by this agent A/B —
        # the analyzer says so rather than emitting an under-powered flattering verdict.
        v = verdict("provenance_weighting")
        self.assertFalse(v["honest"])
        self.assertFalse(v["recommend_adopt"])
        self.assertTrue(
            any("kge_ablation" in r for r in v["_detail"]["blocked_reason"]),
            v["_detail"]["blocked_reason"],
        )

    def test_provenance_weighting_is_scoped_to_confidence_ordered_queries(self):
        cap = ap._capabilities()["provenance_weighting"]
        relevant = [t for t in TASKS.values() if cap["relevant"](t)]
        self.assertTrue(relevant, "no task exercises a confidence-ordered canned query")
        self.assertLess(len(relevant), len(TASKS))
        for t in relevant:
            self.assertIn(t["armB"]["query"], ap.CONF_RANKED_QUERIES)


class TestSharedBarsReused(unittest.TestCase):
    def test_the_shared_bars_are_imported_from_stats_not_redeclared(self):
        self.assertEqual(stats_mod.MIN_N, ap.MIN_N)
        self.assertEqual(stats_mod.MAX_P, ap.MAX_P)
        self.assertEqual(stats_mod.BOOTSTRAP_ITERS, ap.BOOTSTRAP_ITERS)
        # ...and they are ALIASES, not copied literals that could drift apart.
        src = (BENCH / "analyze_prov.py").read_text(encoding="utf-8")
        for bar in ("MIN_N", "MAX_P", "BOOTSTRAP_ITERS"):
            self.assertIn(f"{bar} = stats.{bar}", src)

    def test_token_win_still_applies_the_shared_relative_reduction_bar(self):
        # A saving smaller than the shared bar is NOT a token_win, even though it
        # passes the Phase-7 overhead ceiling.
        small = stats_mod.MIN_RELATIVE_REDUCTION / 2
        v = verdict("citations", token_overhead=-small)
        self.assertFalse(v["token_win"])
        self.assertTrue(v["_detail"]["token_ok"])
        big = verdict("citations", token_overhead=-(stats_mod.MIN_RELATIVE_REDUCTION + 0.1))
        self.assertTrue(big["token_win"])

    def test_the_phase7_overhead_ceiling_is_an_addition_not_a_relaxation(self):
        self.assertGreater(ap.MAX_TOKEN_OVERHEAD, 0.0)
        self.assertLess(ap.MAX_TOKEN_OVERHEAD, stats_mod.MIN_RELATIVE_REDUCTION)


class TestArmTagBroadening(unittest.TestCase):
    """`tokens_real.py`'s arm tag was broadened from `[ABC]` to a name so the Phase-7
    arms attribute; the original 3-arm attribution must not regress."""

    def _mine(self, arms: list[str]) -> list[dict]:
        with tempfile.TemporaryDirectory() as d:
            for i, arm in enumerate(arms):
                lines = [
                    json.dumps({"type": "user", "text": f"[ABM task=t0{i + 1} arm={arm}]"}),
                    json.dumps(
                        {
                            "type": "assistant",
                            "message": {"usage": {
                                "input_tokens": 100,
                                "cache_read_input_tokens": 1000,
                                "cache_creation_input_tokens": 40,
                            }},
                        }
                    ),
                ]
                (Path(d) / f"agent-{i}.jsonl").write_text("\n".join(lines) + "\n")
            return tokens_real.mine(d)

    def test_named_phase7_arms_are_attributed(self):
        rows = self._mine(["base", "cite", "hedge", "weight"])
        self.assertEqual(["base", "cite", "hedge", "weight"], sorted(r["arm"] for r in rows))
        self.assertTrue(all(r["task"] for r in rows))

    def test_the_original_three_arm_labels_still_attribute(self):
        rows = self._mine(["A", "B", "C"])
        self.assertEqual(["A", "B", "C"], sorted(r["arm"] for r in rows))

    def test_effective_input_tokens_use_the_canonical_cache_discounts(self):
        row = self._mine(["base"])[0]
        self.assertEqual(
            round(1.0 * 100 + 0.1 * 1000 + 1.25 * 40, 1), row["eff_input_tokens"]
        )


class TestEndToEnd(unittest.TestCase):
    def test_main_writes_a_per_capability_verdict_file(self):
        cap = ap._capabilities()["citations"]
        tok, answers, records = fixture(arm=cap["arm"])
        with tempfile.TemporaryDirectory() as d:
            tok_path = Path(d) / "tok.json"
            ans_path = Path(d) / "ans.json"
            out_path = Path(d) / "verdict-phase7.json"
            tok_path.write_text(json.dumps(
                [{"task": t, "arm": a, **r} for (t, a), r in tok.items()]
            ))
            ans_path.write_text(json.dumps(
                [{**records[(t, a)], "answer": ans} for (t, a), ans in answers.items()]
            ))
            rc = ap.main([
                "analyze_prov.py", "--tasks", str(TASKS_PATH),
                "--tokens", str(tok_path), "--answers", str(ans_path),
                "--out", str(out_path),
            ])
            self.assertEqual(0, rc)
            out = json.loads(out_path.read_text())
        self.assertEqual("sq-2489d.6", out["bead"])
        self.assertEqual(set(ap._capabilities()), set(out["capabilities"]))
        self.assertTrue(out["capabilities"]["citations"]["recommend_adopt"])


if __name__ == "__main__":
    unittest.main()
