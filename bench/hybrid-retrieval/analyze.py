#!/usr/bin/env python3
"""Deterministic weighted RRF reference over recorded ranking lists."""

# [GPT-5.6] sq-ljp19: keep the reference independent of engine implementation details.

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parent
DEFAULT_INPUT = ROOT / "fixtures" / "rankings.json"
DEFAULT_EXPECTED = ROOT / "fixtures" / "expected.json"


def reciprocal_rank_fusion(
    arms: dict[str, list[str]], weights: dict[str, float], rrf_k: int, top_k: int
) -> list[dict[str, Any]]:
    """Fuse the first top_k results of every arm, retaining rank provenance."""
    scores: dict[str, float] = {}
    provenance: dict[str, dict[str, int]] = {}
    for arm in sorted(arms):
        if arm not in weights:
            raise ValueError(f"missing weight for arm {arm!r}")
        seen: set[str] = set()
        for rank, document in enumerate(arms[arm][:top_k], start=1):
            if document in seen:
                raise ValueError(f"duplicate document {document!r} in arm {arm!r}")
            seen.add(document)
            scores[document] = scores.get(document, 0.0) + weights[arm] / (rrf_k + rank)
            provenance.setdefault(document, {})[arm] = rank

    # Document id is the final tie-breaker, making ties reproducible.
    ordered = sorted(scores, key=lambda document: (-scores[document], document))[:top_k]
    return [
        {
            "document": document,
            "score": f"{scores[document]:.12f}",
            "ranks": provenance[document],
        }
        for document in ordered
    ]


def overlap(left: list[str], right: list[str], top_k: int) -> int:
    return len(set(left[:top_k]) & set(right[:top_k]))


def rank_delta(document: str, fused: list[str], arm: list[str], top_k: int) -> int:
    """Positive means fusion promoted a document; missing means rank top_k + 1."""
    fused_rank = fused.index(document) + 1
    arm_rank = arm[:top_k].index(document) + 1 if document in arm[:top_k] else top_k + 1
    return arm_rank - fused_rank


def analyze(data: dict[str, Any]) -> dict[str, Any]:
    top_k = int(data["top_k"])
    rrf_k = int(data["rrf_k"])
    weights = {arm: float(weight) for arm, weight in data["weights"].items()}
    if top_k < 1 or rrf_k < 0 or any(weight < 0 for weight in weights.values()):
        raise ValueError("top_k must be positive; rrf_k and weights must be non-negative")

    query_reports = []
    fused_hits = 0
    arm_hits = {arm: 0 for arm in sorted(weights)}
    for query in data["queries"]:
        arms = query["arms"]
        if set(arms) != set(weights):
            raise ValueError(f"query {query['id']!r} arms do not match configured weights")
        fused = reciprocal_rank_fusion(arms, weights, rrf_k, top_k)
        fused_ids = [row["document"] for row in fused]
        relevant = set(query["relevant"])
        fused_hits += len(set(fused_ids) & relevant)
        for arm in arm_hits:
            arm_hits[arm] += len(set(arms[arm][:top_k]) & relevant)
        query_reports.append(
            {
                "id": query["id"],
                "fused": fused,
                "overlap_at_k": {
                    arm: overlap(fused_ids, arms[arm], top_k) for arm in sorted(arms)
                },
                "rank_deltas": {
                    arm: {
                        document: rank_delta(document, fused_ids, arms[arm], top_k)
                        for document in fused_ids
                    }
                    for arm in sorted(arms)
                },
            }
        )

    best_single_hits = max(arm_hits.values(), default=0)
    demonstrated = "lift" if fused_hits > best_single_hits else (
        "parity" if fused_hits == best_single_hits else "behind"
    )
    claim = data.get("claim", "parity")
    if claim == "lift" and demonstrated != "lift":
        raise ValueError(
            f"unsupported lift claim: fused hits={fused_hits}, best single arm={best_single_hits}"
        )
    if claim not in {"lift", "parity", "behind"}:
        raise ValueError(f"unknown claim {claim!r}")

    return {
        "parameters": {"rrf_k": rrf_k, "top_k": top_k, "weights": weights},
        "queries": query_reports,
        "ablation": {
            "relevant_at_k": {"fused": fused_hits, "single_arms": arm_hits},
            "demonstrated": demonstrated,
            "claim": claim,
        },
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", type=Path, default=DEFAULT_INPUT)
    parser.add_argument("--expected", type=Path, default=DEFAULT_EXPECTED)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()

    try:
        report = analyze(json.loads(args.input.read_text(encoding="utf-8")))
        if args.check:
            expected = json.loads(args.expected.read_text(encoding="utf-8"))
            if report != expected:
                print("hybrid-retrieval fixture mismatch", file=sys.stderr)
                return 1
        print(json.dumps(report, indent=2, sort_keys=True))
        return 0
    except (KeyError, TypeError, ValueError, json.JSONDecodeError, OSError) as error:
        print(f"hybrid-retrieval analysis failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
