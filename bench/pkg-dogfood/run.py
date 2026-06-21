#!/usr/bin/env python3
# [OPUS-4.8] sq-2m6zm.4 (epic sq-2m6zm). 🤖 SPARQ agent — the counterbalanced A/B
# DRIVER for the PKG token measurement. Written while Fable unavailable; flag for
# re-review when Fable returns. Spec: research/dogfooding-sparq-knowledge-graph.md §5.
#
# For each frozen task it measures the READ PAYLOAD (chars) each arm forces into the
# agent's context, then converts to cache-discounted effective input tokens via
# tokens.py (the documented char/token proxy + the canonical §5.1 formula). It
# CHARGES ARM B ALL ITS COSTS (§5.1): schema card + executed SPARQL + answer rows +
# the deferred tool-def tax + amortised one-time ingestion. It emits one JSONL row
# per task plus a header row carrying the amortised one-time costs and the manifest
# verification. Output is consumed by grade.py (correctness) and stats.py (verdict).
#
# Determinism: arm B payloads come from actually RUNNING the pkg-query binary
# (record); arm A payloads come from slicing the pinned corpus. The corpus is
# sha256-verified against corpus-manifest.json before any measurement — a drift
# aborts the run (the pre-registration is over a FROZEN corpus).

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
sys.path.insert(0, str(HERE))
import tokens as tok  # noqa: E402

PKG_QUERY = REPO / "target" / "debug" / "pkg-query"
SKILL = REPO / ".claude" / "skills" / "query-pkg" / "SKILL.md"
AGENTS = REPO / "AGENTS.md"
BEADS = REPO / ".beads" / "issues.jsonl"
INGEST = REPO / "crates" / "sparq-kb" / "ingest" / "pkg-instances.ttl"
MANIFEST = HERE / "corpus-manifest.json"


def verify_corpus() -> None:
    """Abort unless the pinned corpus matches corpus-manifest.json (frozen-corpus
    invariant). A mismatch means the pre-registration no longer applies."""
    man = json.loads(MANIFEST.read_text())
    for rel, exp in man.items():
        p = REPO / rel
        got = hashlib.sha256(p.read_bytes()).hexdigest()
        if got != exp["sha256"]:
            raise SystemExit(
                f"CORPUS DRIFT: {rel} sha256 {got[:12]} != pinned {exp['sha256'][:12]} "
                f"— the frozen task set's gold/payloads no longer hold. Re-freeze before running."
            )


def run_pkg_query(query: str | None, arg: str | None) -> tuple[str, str]:
    """Run the pkg-query helper for arm B. Returns (sparql_printed, rows_printed).
    The helper prints the executed SPARQL on STDERR (the design's "surface the
    SPARQL" rule) and the answer table on STDOUT — the agent reads BOTH, so both
    are charged to arm B."""
    cmd = [str(PKG_QUERY), "--query", query]
    if arg is not None:
        cmd += ["--arg", arg]
    p = subprocess.run(cmd, capture_output=True, text=True)
    if p.returncode != 0:
        raise SystemExit(f"pkg-query failed for {query} {arg}: {p.stderr}")
    return p.stderr, p.stdout


def schema_card_chars() -> int:
    """The one-time introspect schema card (schema-classes + schema-properties)
    the agent reads once to learn what it can ask, then reuses. Charged ONCE,
    amortised over N in the per-task rows."""
    total = 0
    for q in ("schema-classes", "schema-properties"):
        sparql, rows = run_pkg_query(q, None)
        total += len(sparql) + len(rows)
    return total


def tool_def_chars() -> int:
    """The deferred-tool-definition tax: the query-pkg SKILL.md description block
    that must sit in the prompt for the agent to know the tool exists. We charge
    the YAML front-matter `description:` line (what a deferred-tool surface puts in
    the prefix), per §5.1 'the deferred-tool-definition tokens actually pulled in'."""
    text = SKILL.read_text()
    # the front-matter description field is the load-bearing prompt cost
    for line in text.splitlines():
        if line.startswith("description:"):
            return len(line)
    return len(text.split("---")[1]) if "---" in text else 200


def ingest_build_chars() -> int:
    """One-time PKG ingestion cost (§5.1 amortised slice). Proxy: the size of the
    ingested instance file the build produced — the bytes that had to be derived
    once. Amortised over N in the per-task rows. (The CPU/wall of the build is
    non-canonical and not a token cost; the durable artifact size is the honest
    token-equivalent of 'what ingestion produced once'.)"""
    return len(INGEST.read_bytes())


def slice_section(doc: Path, start: str, end: str | None) -> str:
    """Extract a named '## ...' section of a markdown doc (arm A realistic read).
    `start` is a section-heading prefix; `end` is the next heading prefix (or EOF)."""
    lines = doc.read_text().splitlines(keepends=True)
    out, capturing = [], False
    for ln in lines:
        if not capturing and ln.startswith(start):
            capturing = True
            out.append(ln)
            continue
        if capturing:
            if end and ln.startswith(end):
                break
            out.append(ln)
    return "".join(out)


def bd_show_chars(bd_args: str) -> int:
    """Arm A realistic read for a Task question: the `bd <args>` output an agent
    actually reads, NOT the 1.6 MB issues.jsonl. Falls back to a grep of the jsonl
    record if bd is unavailable (so the harness is reproducible offline)."""
    try:
        p = subprocess.run(
            ["bd"] + bd_args.split(), capture_output=True, text=True, timeout=30
        )
        if p.returncode == 0 and p.stdout.strip():
            return len(p.stdout)
    except (FileNotFoundError, subprocess.TimeoutExpired):
        pass
    # offline fallback: the single jsonl record (still tiny vs the whole file)
    bead = bd_args.split()[-1]
    for ln in BEADS.read_text().splitlines():
        if f'"{bead}"' in ln:
            return len(ln)
    return 80  # an empty/"not found" bd response


def measure(tasks_path: Path) -> dict:
    verify_corpus()
    tasks = [json.loads(l) for l in tasks_path.read_text().splitlines() if l.strip()]
    n = len(tasks)

    # one-time arm-B costs, amortised over N (charged at the warm cache-read rate
    # because they sit behind the deferred cache breakpoint, per §1.1).
    schema = schema_card_chars()
    tooldef = tool_def_chars()
    ingest = ingest_build_chars()
    amortised_b_cache_chars = (schema + tooldef + ingest) / n

    rows = []
    for t in tasks:
        # --- ARM B: pkg-query (fresh: SPARQL + rows; cache: amortised one-time) ---
        b = t["armB"]
        sparql, ans = run_pkg_query(b["query"], b.get("arg"))
        b_fresh_chars = len(sparql) + len(ans)
        b_eff = tok.effective(
            fresh_chars=b_fresh_chars, cache_read_chars=amortised_b_cache_chars
        )

        # --- ARM A: realistic read + naive whole-doc read ---
        a = t["armA"]
        if a.get("bd"):
            a_real_chars = bd_show_chars(a["bd"])
            a_naive_chars = len(BEADS.read_bytes())  # the whole jsonl (worst case)
        else:
            sec = slice_section(AGENTS, a["section"], a.get("section_to"))
            a_real_chars = len(sec) if sec else len(AGENTS.read_bytes())
            a_naive_chars = len(AGENTS.read_bytes())
        # arm A pays its read as fresh input the first time (cache write on read).
        a_real_eff = tok.effective(fresh_chars=a_real_chars)
        a_naive_eff = tok.effective(fresh_chars=a_naive_chars)

        rows.append(
            {
                "id": t["id"],
                "stratum": t["stratum"],
                "armA_realistic_chars": a_real_chars,
                "armA_naive_chars": a_naive_chars,
                "armB_fresh_chars": b_fresh_chars,
                "armB_amortised_cache_chars": round(amortised_b_cache_chars, 1),
                "eff_A_realistic": round(a_real_eff, 2),
                "eff_A_naive": round(a_naive_eff, 2),
                "eff_B": round(b_eff, 2),
                "delta_realistic": round(a_real_eff - b_eff, 2),
                "delta_naive": round(a_naive_eff - b_eff, 2),
                # record arm B's answer payload for the grader (paired correctness)
                "armB_answer": ans,
            }
        )

    return {
        "meta": {
            "n_tasks": n,
            "chars_per_token_proxy": tok.CHARS_PER_TOKEN,
            "eff_cache_read_mult": tok.EFF_CACHE_READ,
            "eff_cache_write_mult": tok.EFF_CACHE_WRITE,
            "one_time_schema_card_chars": schema,
            "one_time_tool_def_chars": tooldef,
            "one_time_ingest_chars": ingest,
            "amortised_b_cache_chars_per_task": round(amortised_b_cache_chars, 1),
            "corpus_verified": True,
            "fidelity": "read-payload proxy (NOT count_tokens, NOT full-session A/B) — see PREREG.md",
        },
        "rows": rows,
    }


def main() -> None:
    ap = argparse.ArgumentParser(description="PKG token A/B driver (sq-2m6zm.4)")
    ap.add_argument("--tasks", default=str(HERE / "tasks" / "tasks.jsonl"))
    ap.add_argument("--out", default=str(HERE / "results.json"))
    args = ap.parse_args()
    res = measure(Path(args.tasks))
    Path(args.out).write_text(json.dumps(res, indent=2) + "\n")
    print(f"[run] wrote {args.out}  (n={res['meta']['n_tasks']} tasks)")
    print(f"[run] fidelity: {res['meta']['fidelity']}")


if __name__ == "__main__":
    main()
