#!/usr/bin/env python3
# [OPUS-4.8] sq-mztg8 (FO-KM epic; design research/foundational-ontology-km-benchmark.md,
# PR #1106; harness PR #1107). 🤖 SPARQ agent — the Metric-1 token-miner + coverage grader.
# Written while Fable unavailable; flag for re-review when Fable returns.
#
# WHAT THIS IS
# --------------------------------------------------------------------------------
# The reproducible analyzer for FO-KM Metric 1 (the AGENT A/B). It turns a directory of
# per-(arm, task) Haiku NL-tool transcripts into the RESULTS.md table:
#
#   1. TOKEN MINER — reads each fresh sub-agent's `agent-*.jsonl` Claude Code transcript
#      and sums the cache-discounted EFFECTIVE INPUT TOKENS straight from every assistant
#      message's `message.usage` block:
#          effective_input = 1.0*input + 0.1*cache_read + 1.25*cache_creation
#      (the canonical §5.1 multipliers; identical to bench/pkg-dogfood/tokens_real.py).
#      NO count_tokens API, NO char proxy — these are the tokens the model actually
#      consumed. Closure build CPU/wall cost is NON-canonical and never charged here.
#
#   2. COVERAGE GRADER — deterministic, NO model in the loop (anti-circularity, design
#      §5.2). For each (arm, task) it grades the agent's final NL answer + the rows it
#      read back against the task's gold from tasks.jsonl:
#        * count-coverage   — the gold COUNT appears in the answer (for COUNT/scalar tasks)
#        * entity-coverage  — every gold local-name (gold_keys list) is resolvable in the
#                             answer payload; fraction resolved is the per-task score and a
#                             task is "correct" at/above THRESHOLD (default 1.0 = all keys)
#        * concept-coverage — for a partition task whose gold is a {bucket: count} dict,
#                             every bucket's count appears (CC partitions / cc05 / th03)
#      An honestly EMPTY answer on an arm that genuinely cannot answer (select[arm] is null)
#      is recorded as an ABSTAIN, not a wrong answer.
#
#   3. AGGREGATE — per-arm accuracy (mean per-task correct), abstain count, MEDIAN effective
#      input tokens, and model-price-weighted total $ (Haiku $1/Mtok in, $5/Mtok out).
#
# Each transcript's first user turn carries the attribution tag
#     [FOKM task=<id> arm=<no-fo|gufo|dolce-dul|schema-org>]
# written verbatim by the orchestrator that fans out the per-(arm, task) runs (see README).
#
# Usage:
#   analyze.py <transcript-dir> [--tasks bench/fo-km/tasks.jsonl] [--threshold 1.0] [--json out.json]
#
# <transcript-dir> holds one agent-*.jsonl per sub-agent (one (arm, task) each). With no
# transcript dir the script still validates the tasks file + prints the grading contract,
# so it is runnable as a self-check without a run on hand.

from __future__ import annotations

import argparse
import glob
import json
import os
import re
import statistics
import sys

# --- §5.1 cache-discount multipliers (same as metrics_row.py / tokens_real.py). Kept as
#     literals so the analyzer runs standalone on a transcript dir with no repo on path. ---
EFF_FRESH, EFF_CACHE_READ, EFF_CACHE_WRITE = 1.0, 0.1, 1.25

# --- Haiku list price (USD per Mtok). The whole 4-arm run uses Haiku, so $ is comparable
#     across arms; output is priced too because the NL-tool emits a verbose NL answer. ---
HAIKU_IN_PER_MTOK, HAIKU_OUT_PER_MTOK = 1.0, 5.0

ARMS = ("no-fo", "gufo", "dolce-dul", "schema-org")

# Attribution tag the orchestrator writes into each sub-agent's opening user turn.
TAG = re.compile(r"\[FOKM task=(\S+) arm=(no-fo|gufo|dolce-dul|schema-org)\]")


# --------------------------------------------------------------------------------
# 1. token miner
# --------------------------------------------------------------------------------
def mine_transcript(path: str) -> dict:
    """Sum cache-discounted effective input tokens + output tokens from one transcript."""
    task = arm = None
    eff = 0.0
    fresh = cache_read = cache_write = out_tok = 0
    n_assist = 0
    answer_chunks: list[str] = []
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            line = line.rstrip("\n")
            if not line:
                continue
            if task is None:
                m = TAG.search(line)
                if m:
                    task, arm = m.group(1), m.group(2)
            try:
                obj = json.loads(line)
            except (json.JSONDecodeError, ValueError):
                continue
            if obj.get("type") != "assistant":
                continue
            msg = obj.get("message") or {}
            usage = msg.get("usage") or {}
            fi = int(usage.get("input_tokens") or 0)
            rd = int(usage.get("cache_read_input_tokens") or 0)
            wr = int(usage.get("cache_creation_input_tokens") or 0)
            ot = int(usage.get("output_tokens") or 0)
            if fi or rd or wr:
                n_assist += 1
                fresh += fi
                cache_read += rd
                cache_write += wr
                out_tok += ot
                eff += EFF_FRESH * fi + EFF_CACHE_READ * rd + EFF_CACHE_WRITE * wr
            # Collect the assistant TEXT so the grader can read the final NL answer.
            for block in (msg.get("content") or []):
                if isinstance(block, dict) and block.get("type") == "text":
                    answer_chunks.append(block.get("text") or "")
    return {
        "agent_file": os.path.basename(path),
        "task": task,
        "arm": arm,
        "eff_input_tokens": round(eff, 1),
        "fresh": fresh,
        "cache_read": cache_read,
        "cache_write": cache_write,
        "output_tokens": out_tok,
        "assistant_turns": n_assist,
        "answer_text": "\n".join(answer_chunks),
    }


# --------------------------------------------------------------------------------
# 2. coverage grader (deterministic, no model)
# --------------------------------------------------------------------------------
def _flatten_gold_keys(gold_keys) -> list[str]:
    """gold_keys is either a list of local-names or a {bucket: [names]} dict."""
    if isinstance(gold_keys, dict):
        out: list[str] = []
        for v in gold_keys.values():
            out.extend(v if isinstance(v, list) else [str(v)])
        return out
    if isinstance(gold_keys, list):
        return [str(k) for k in gold_keys]
    return [str(gold_keys)]


def _gold_counts(gold_count) -> list[int]:
    """gold_count is an int, or a {bucket: count} dict (partition / CC tasks)."""
    if isinstance(gold_count, dict):
        return [int(v) for v in gold_count.values()]
    return [int(gold_count)]


def grade(answer_text: str, task: dict, threshold: float) -> dict:
    """Return {score, correct, mode, resolved, total} for one (arm, task) answer.

    Coverage modes (design §5.2):
      * concept-coverage — gold is a {bucket: count} dict: every bucket count must appear.
      * entity-coverage  — gold_keys is a real entity list: fraction of local-names that
                           resolve in the answer; correct iff >= threshold.
      * count-coverage   — gold_keys is a synthetic single sentinel (e.g. "top-1277"):
                           the gold COUNT integer must appear in the answer.
    Case-insensitive substring resolution (a fact, not a verbatim token).
    """
    low = answer_text.lower()
    gold_keys = task["gold_keys"]
    gold_count = task["gold_count"]

    # concept-coverage: a {bucket: count} partition — every bucket count must surface.
    if isinstance(gold_count, dict):
        counts = _gold_counts(gold_count)
        resolved = [c for c in counts if re.search(rf"(?<!\d){c}(?!\d)", answer_text)]
        score = len(resolved) / len(counts) if counts else 0.0
        return {"mode": "concept-coverage", "score": round(score, 3),
                "correct": score >= threshold, "resolved": len(resolved), "total": len(counts)}

    keys = _flatten_gold_keys(gold_keys)
    # A single synthetic sentinel key (e.g. "top-1277" / "events-1255") => count-coverage:
    # grade on the gold COUNT integer appearing, not the made-up sentinel string.
    looks_synthetic = (len(keys) == 1 and not keys[0].startswith(("find-", "skill-", "surface-", "doc-", "task-")))
    if looks_synthetic:
        n = int(gold_count) if not isinstance(gold_count, dict) else 0
        hit = bool(re.search(rf"(?<!\d){n}(?!\d)", answer_text))
        return {"mode": "count-coverage", "score": 1.0 if hit else 0.0,
                "correct": hit, "resolved": int(hit), "total": 1}

    # entity-coverage: fraction of gold local-names resolvable in the answer payload.
    resolved = [k for k in keys if k.lower() in low]
    score = len(resolved) / len(keys) if keys else 0.0
    return {"mode": "entity-coverage", "score": round(score, 3),
            "correct": score >= threshold, "resolved": len(resolved), "total": len(keys)}


def arm_can_answer(task: dict, arm: str) -> bool:
    """select[arm] is null where an FO honestly cannot draw the distinction => abstain-OK."""
    sel = task.get("select") or {}
    if arm == "no-fo":
        return True  # the incumbent always *attempts* (its no_fo query) — it may return 0
    return sel.get(arm) is not None


def is_abstain(answer_text: str) -> bool:
    """An honest abstention: the answer says it cannot answer / found nothing."""
    low = answer_text.lower()
    return any(p in low for p in (
        "cannot answer", "can't answer", "no answer", "unable to", "not answerable",
        "0 rows", "no rows", "empty result", "returned nothing", "i abstain", "abstain",
    )) and not re.search(r"\b\d{2,}\b", answer_text)  # no big count present => truly empty


# --------------------------------------------------------------------------------
# 3. aggregate
# --------------------------------------------------------------------------------
def aggregate(rows: list[dict], tasks_by_id: dict, threshold: float) -> dict:
    per_arm: dict[str, dict] = {a: {"graded": [], "tokens": [], "out": [], "abstain": 0,
                                    "by_kind": {}} for a in ARMS}
    for r in rows:
        tid, arm = r.get("task"), r.get("arm")
        if not tid or arm not in per_arm or tid not in tasks_by_id:
            continue
        task = tasks_by_id[tid]
        kind = task["kind"]
        bucket = per_arm[arm]
        bucket["tokens"].append(r["eff_input_tokens"])
        bucket["out"].append(r["output_tokens"])
        # Abstain accounting: an arm that legitimately cannot answer AND honestly abstains.
        if (not arm_can_answer(task, arm)) and is_abstain(r["answer_text"]):
            bucket["abstain"] += 1
            continue
        g = grade(r["answer_text"], task, threshold)
        bucket["graded"].append(g["correct"])
        bucket["by_kind"].setdefault(kind, []).append(g["correct"])

    summary = {}
    for arm, b in per_arm.items():
        n = len(b["graded"])
        acc = (sum(1 for c in b["graded"] if c) / n) if n else None
        med_tok = round(statistics.median(b["tokens"])) if b["tokens"] else None
        total_in_tok = sum(b["tokens"])
        total_out_tok = sum(b["out"])
        total_usd = (total_in_tok / 1e6) * HAIKU_IN_PER_MTOK + (total_out_tok / 1e6) * HAIKU_OUT_PER_MTOK
        by_kind = {k: round(sum(1 for c in v if c) / len(v), 2) for k, v in b["by_kind"].items() if v}
        summary[arm] = {
            "accuracy": round(acc, 2) if acc is not None else None,
            "abstain": b["abstain"],
            "n_graded": n,
            "median_eff_input_tokens": med_tok,
            "total_usd": round(total_usd, 2),
            "by_kind": by_kind,
        }
    return summary


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description="FO-KM Metric-1 token-miner + coverage grader")
    ap.add_argument("transcript_dir", nargs="?",
                    help="dir of agent-*.jsonl transcripts (one per (arm, task)); "
                         "omit to self-check the tasks file + print the grading contract")
    ap.add_argument("--tasks", default="bench/fo-km/tasks.jsonl",
                    help="gold task set (default bench/fo-km/tasks.jsonl)")
    ap.add_argument("--threshold", type=float, default=1.0,
                    help="entity-coverage fraction required for 'correct' (default 1.0)")
    ap.add_argument("--json", dest="json_out", help="write the per-run + summary JSON here")
    args = ap.parse_args(argv[1:])

    tasks = [json.loads(l) for l in open(args.tasks, encoding="utf-8") if l.strip()]
    tasks_by_id = {t["id"]: t for t in tasks}
    kinds = {}
    for t in tasks:
        kinds[t["kind"]] = kinds.get(t["kind"], 0) + 1
    print(f"tasks: {len(tasks)} {kinds}")

    if not args.transcript_dir:
        print("\nNo transcript dir given — self-check only. Grading contract:")
        print("  * count-coverage   : the gold integer COUNT appears in the answer")
        print("  * entity-coverage  : every gold local-name resolves (threshold-fraction)")
        print("  * concept-coverage : every {bucket: count} partition value appears")
        print("  * abstain          : select[arm] is null AND the answer honestly says empty")
        print("\nFormula (effective input tokens): "
              "1.0*input + 0.1*cache_read + 1.25*cache_creation")
        return 0

    paths = sorted(glob.glob(os.path.join(args.transcript_dir, "agent-*.jsonl")))
    rows = [mine_transcript(p) for p in paths]
    tagged = [r for r in rows if r["task"]]
    print(f"transcripts: {len(rows)} | FOKM-tagged: {len(tagged)}")
    summary = aggregate(rows, tasks_by_id, args.threshold)

    print(f"\n{'arm':18s} {'acc':>5s} {'abst':>5s} {'med_eff_tok':>12s} {'total_$':>8s}  by-kind")
    for arm in ARMS:
        s = summary[arm]
        acc = f"{s['accuracy']:.2f}" if s["accuracy"] is not None else "  - "
        med = f"{s['median_eff_input_tokens']:,}" if s["median_eff_input_tokens"] else "-"
        bk = " ".join(f"{k}={v}" for k, v in sorted(s["by_kind"].items()))
        print(f"{arm:18s} {acc:>5s} {s['abstain']:>5d} {med:>12s} {s['total_usd']:>8.2f}  {bk}")

    if args.json_out:
        with open(args.json_out, "w", encoding="utf-8") as fh:
            json.dump({"per_run": rows, "summary": summary}, fh, indent=2)
        print(f"\nwrote {args.json_out}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
