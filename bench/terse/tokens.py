#!/usr/bin/env python3
# [OPUS-4.8] sq-bzign (epic sq-2m6zm). 🤖 SPARQ agent — per-(task,arm) EFFECTIVE-input-token
# accounting for the sparq-terse Phase-5 A/B. Written while Fable unavailable; flag for re-review
# when Fable returns. Spec: research/llm-ergonomic-sparql-surface.md §5.1/§5.2 + PREREG.md.
#
# TWO modes, by fidelity (PREREG "measurement-fidelity caveat"):
#
#   --transcripts <dir>  REAL: mine each `[TERSE task=<id> arm=<A|B|C>]`-tagged sub-agent
#                        transcript's `message.usage` and sum the cache-discounted effective input
#                        tokens (1.0*input + 0.1*cache_read + 1.25*cache_creation) — identical to
#                        bench/pkg-dogfood/tokens_real.py. fidelity="full-session-transcript".
#                        (Use when a fan-out exists; none is available in this single-session env.)
#
#   default (no flag)    PROXY: the deterministic INPUT-AUTHORING cost the design names as the
#                        dominant lever (§1.6/§7): per arm the CONTEXT CARD it carries (charged once
#                        behind the prompt-cache breakpoint -> cache_read ×0.1, amortised /N) PLUS
#                        the AUTHORED query (fresh ×1.0). Card + query char-counts -> token estimate
#                        via a documented char/token ratio -> the canonical effective-token formula.
#                        fidelity="input-authoring-proxy".
#
# Both feed the SAME verdict schema; the verdict records which fidelity produced it. All numbers are
# runtime-only / NON-CANONICAL (never frozen into committed markdown).

from __future__ import annotations

import argparse
import glob
import json
import os
import re
import sys

EFF_FRESH, EFF_CACHE_READ, EFF_CACHE_WRITE = 1.0, 0.1, 1.25
# Documented char->token proxy. ~3.7 chars/token is the conservative English+code mix for the Claude
# tokenizer family (the exact tokenizer needs the count_tokens API / an ANTHROPIC_API_KEY, neither
# available here). The RATIO between arms is what the verdict turns on, and it is invariant to this
# constant; it is stated openly so the estimate is reproducible and easy to re-point.
CHARS_PER_TOKEN = 3.7

TAG = re.compile(r"\[TERSE task=(\S+) arm=([ABC])\]")


def est_tokens(text: str) -> int:
    return max(1, round(len(text) / CHARS_PER_TOKEN))


# ---------------------------------------------------------------------------- REAL transcript miner
def mine_transcripts(transcript_dir: str) -> list[dict]:
    rows = []
    for path in sorted(glob.glob(os.path.join(transcript_dir, "agent-*.jsonl"))):
        task = arm = None
        eff = 0.0
        fresh = cr = cw = ot = 0
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
                u = (obj.get("message") or {}).get("usage") or {}
                fi = int(u.get("input_tokens") or 0)
                rd = int(u.get("cache_read_input_tokens") or 0)
                wr = int(u.get("cache_creation_input_tokens") or 0)
                ot += int(u.get("output_tokens") or 0)
                fresh += fi
                cr += rd
                cw += wr
                eff += EFF_FRESH * fi + EFF_CACHE_READ * rd + EFF_CACHE_WRITE * wr
        if task:
            rows.append({"task": task, "arm": arm, "eff_input_tokens": round(eff, 1),
                         "output_tokens": ot, "fresh": fresh, "cache_read": cr, "cache_write": cw,
                         "fidelity": "full-session-transcript"})
    return rows


# ------------------------------------------------------------------- input-authoring PROXY accountant
def proxy_rows(cards: dict[str, str], refs: dict, task_ids: list[str]) -> list[dict]:
    """cards: {'A': schema-card text, 'B': legend-card text, 'C': legend-card text}.
    The card is billed once behind the cache breakpoint (cache_read, ×0.1) and AMORTISED over N
    tasks; the authored query is fresh (×1.0) per task. We report the per-task effective-input as
    card_amortised_eff + query_eff so the paired delta is the authoring cost the design predicts."""
    n = len(task_ids)
    rows = []
    card_tok = {arm: est_tokens(cards[arm]) for arm in ("A", "B", "C")}
    for arm in ("A", "B", "C"):
        # card amortised + charged at the warm cache-read multiplier (the §1.6 caching property)
        card_eff_amort = EFF_CACHE_READ * card_tok[arm] / n
        for tid in task_ids:
            q = refs[tid][arm]
            qtok = est_tokens(q)
            query_eff = EFF_FRESH * qtok
            rows.append({
                "task": tid, "arm": arm,
                "card_tokens": card_tok[arm],
                "card_eff_amortised": round(card_eff_amort, 2),
                "query_tokens": qtok,
                "query_eff": round(query_eff, 2),
                "eff_input_tokens": round(card_eff_amort + query_eff, 2),
                "eff_input_tokens_no_card": round(query_eff, 2),
                "output_tokens": qtok,  # authoring a query: the output IS the query
                "fidelity": "input-authoring-proxy",
            })
    return rows


def read(path: str) -> str:
    with open(path, encoding="utf-8") as fh:
        return fh.read()


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--transcripts", default=None, help="REAL mode: dir of tagged agent-*.jsonl")
    ap.add_argument("--refs", help="reference_queries.json (proxy mode)")
    ap.add_argument("--tasks", help="terse_tasks.json (proxy mode, for the task id list)")
    ap.add_argument("--schema-card", help="arm-A schema card text file (proxy mode)")
    ap.add_argument("--legend-card", help="arm-B/C legend card text file (proxy mode)")
    ap.add_argument("--out", required=True)
    args = ap.parse_args(argv[1:])

    if args.transcripts:
        rows = mine_transcripts(args.transcripts)
        print(f"[tokens] REAL: mined {len(rows)} tagged transcripts", file=sys.stderr)
    else:
        for req in (args.refs, args.tasks, args.schema_card, args.legend_card):
            if not req:
                print("proxy mode needs --refs --tasks --schema-card --legend-card", file=sys.stderr)
                return 2
        refs = json.load(open(args.refs, encoding="utf-8"))
        task_ids = [t["id"] for t in json.load(open(args.tasks, encoding="utf-8"))]
        cards = {"A": read(args.schema_card),
                 "B": read(args.legend_card),
                 "C": read(args.legend_card)}
        rows = proxy_rows(cards, refs, task_ids)
        print(f"[tokens] PROXY: {len(rows)} (task,arm) rows over N={len(task_ids)} tasks; "
              f"cards A/B/C = {est_tokens(cards['A'])}/{est_tokens(cards['B'])}/"
              f"{est_tokens(cards['C'])} tok", file=sys.stderr)
    with open(args.out, "w", encoding="utf-8") as fh:
        json.dump(rows, fh, indent=2)
    print(f"[tokens] wrote {args.out}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
