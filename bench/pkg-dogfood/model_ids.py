#!/usr/bin/env python3
# [FABLE-5] sq-2m6zm.9 (#1111 re-attempt program, thread A). 🤖 SPARQ agent — per-transcript
# SERVING-MODEL-identity miner for the Fable-subject benchmark re-runs.
#
# WHY THIS EXISTS (bead invariant): Fable sessions can silently serve a different model
# mid-run, so a "Fable-subject" benchmark row is only valid if the transcript itself
# evidences the serving model. This miner reads each `agent-*.jsonl` Claude Code
# transcript in a run directory and reports, per transcript, the set of model ids seen
# on `type == "assistant"` lines (`message.model`) with per-model assistant-line counts.
# The analysis marks a subject cell VALID only when every assistant line was served by
# the expected subject model; mixed/substituted cells are flagged and excluded, never
# counted as the subject model.
#
# Works for both bench/pkg-dogfood ([ABM task= arm=]) and bench/fo-km ([FOKM task= arm=])
# run directories — it re-uses the tag regexes only for attribution and does not grade.
#
# Usage:
#   model_ids.py <transcript-dir> [expected-model-prefix] [out.json]
#
# Exit code 0 always (reporting tool); the VALID/INVALID column is the output.

from __future__ import annotations

import glob
import json
import os
import re
import sys

TAGS = (
    re.compile(r"\[ABM task=(\S+) arm=([ABC])\]"),
    re.compile(r"\[FOKM task=(\S+) arm=(no-fo|gufo|dolce-dul|schema-org)\]"),
)


def mine(path: str) -> dict:
    task = arm = None
    models: dict[str, int] = {}
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            if task is None:
                for tag in TAGS:
                    m = tag.search(line)
                    if m:
                        task, arm = m.group(1), m.group(2)
                        break
            try:
                obj = json.loads(line)
            except (json.JSONDecodeError, ValueError):
                continue
            if obj.get("type") != "assistant":
                continue
            model = (obj.get("message") or {}).get("model")
            if model:
                models[model] = models.get(model, 0) + 1
    return {
        "agent_file": os.path.basename(path),
        "task": task,
        "arm": arm,
        "models": models,
    }


def main(argv: list[str]) -> int:
    if len(argv) < 2:
        print("usage: model_ids.py <transcript-dir> [expected-model-prefix] [out.json]",
              file=sys.stderr)
        return 2
    expected = argv[2] if len(argv) > 2 else None
    rows = [mine(p) for p in sorted(glob.glob(os.path.join(argv[1], "agent-*.jsonl")))]
    n_valid = n_invalid = 0
    for r in rows:
        ids = sorted(r["models"])
        if expected:
            ok = len(ids) >= 1 and all(i.startswith(expected) for i in ids)
            r["valid_for_expected"] = ok
            n_valid += ok
            n_invalid += not ok
            flag = "VALID" if ok else "INVALID"
        else:
            flag = ""
        print(f"  {r['agent_file']:38s} task={str(r['task']):8s} arm={str(r['arm']):11s} "
              f"models={','.join(f'{i}:{r['models'][i]}' for i in ids)} {flag}")
    if expected:
        print(f"expected prefix '{expected}': {n_valid} valid, {n_invalid} invalid "
              f"of {len(rows)} transcripts")
    if len(argv) > 3:
        with open(argv[3], "w", encoding="utf-8") as fh:
            json.dump(rows, fh, indent=2)
        print(f"wrote {argv[3]}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
