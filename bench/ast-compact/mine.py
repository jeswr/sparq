#!/usr/bin/env python3
# [OPUS-4.8] issue #1080 — compacted-AST A/B transcript miner + grader. 🤖 SPARQ agent.
# Reuses the bench/pkg-dogfood method: cache-discounted effective input tokens
# (1.0*input + 0.1*cache_read + 1.25*cache_creation) mined from each fresh sub-agent's
# message.usage, attributed by the [task=<id> arm=<A|B>] tag at the start of its brief.
# Grades the final answer text against the task gold_keys (case-insensitive substring
# coverage). All numbers are runtime/NON-CANONICAL (work-box, list-price context).
import json, sys, glob, os, re, time

SUBAGENT_GLOB = os.path.expanduser(
    "~/.claude/projects/-home-ubuntu-sparq/*/subagents/agent-*.jsonl"
)
TAG_RE = re.compile(r"\[task=([A-Za-z0-9._-]+)\s+arm=([AB])\]")


def _int(x):
    try:
        return int(x)
    except (TypeError, ValueError):
        return 0


def first_user_text(path):
    with open(path) as fh:
        for line in fh:
            try:
                r = json.loads(line)
            except Exception:
                continue
            if r.get("type") == "user":
                c = r.get("message", {}).get("content")
                if isinstance(c, list):
                    c = " ".join(
                        x.get("text", "") for x in c if isinstance(x, dict)
                    )
                return str(c or "")
    return ""


def last_assistant_text(path):
    out = ""
    with open(path) as fh:
        for line in fh:
            try:
                r = json.loads(line)
            except Exception:
                continue
            if r.get("type") == "assistant":
                c = r.get("message", {}).get("content")
                if isinstance(c, list):
                    txt = " ".join(
                        x.get("text", "") for x in c if isinstance(x, dict) and x.get("type") == "text"
                    )
                    if txt.strip():
                        out = txt
    return out


def eff_input_tokens(path):
    """Cache-discounted effective INPUT tokens summed over the transcript's turns."""
    tot = 0.0
    with open(path) as fh:
        for line in fh:
            try:
                r = json.loads(line)
            except Exception:
                continue
            if r.get("type") != "assistant":
                continue
            u = r.get("message", {}).get("usage", {})
            if not isinstance(u, dict):
                continue
            tot += 1.0 * _int(u.get("input_tokens"))
            tot += 0.1 * _int(u.get("cache_read_input_tokens"))
            tot += 1.25 * _int(u.get("cache_creation_input_tokens"))
    return tot


def grade(answer, gold_keys):
    a = answer.lower()
    hit = sum(1 for k in gold_keys if k.lower() in a)
    return hit / len(gold_keys) if gold_keys else 0.0


def main():
    tasks = json.load(open(os.path.join(os.path.dirname(__file__), "tasks.json")))["tasks"]
    gold = {t["id"]: t for t in tasks}
    # Only consider transcripts created after the marker time, if given (epoch secs).
    since = float(sys.argv[1]) if len(sys.argv) > 1 else 0.0
    # Find the newest transcript per (task,arm) tag.
    best = {}  # (task,arm) -> (mtime, path)
    for p in glob.glob(SUBAGENT_GLOB):
        try:
            mt = os.path.getmtime(p)
        except OSError:
            continue
        if mt < since:
            continue
        head = first_user_text(p)[:600]
        m = TAG_RE.search(head)
        if not m:
            continue
        tid, arm = m.group(1), m.group(2)
        if tid not in gold:
            continue
        key = (tid, arm)
        if key not in best or mt > best[key][0]:
            best[key] = (mt, p)

    rows = []
    for (tid, arm), (mt, p) in sorted(best.items()):
        ans = last_assistant_text(p)
        rows.append(
            {
                "task": tid,
                "arm": arm,
                "class": gold[tid]["class"],
                "eff_input_tokens": round(eff_input_tokens(p)),
                "quality": round(grade(ans, gold[tid]["gold_keys"]), 3),
                "transcript": os.path.basename(p),
            }
        )
    json.dump(rows, sys.stdout, indent=2)
    print()


if __name__ == "__main__":
    main()
