#!/usr/bin/env python3
# [OPUS-4.8] sq-2m6zm.4 (epic sq-2m6zm). 🤖 SPARQ agent — the effective-token
# accounting for the PKG token A/B. Written while Fable unavailable; flag for
# re-review when Fable returns. Spec: research/dogfooding-sparq-knowledge-graph.md §5.1.
#
# WHAT THIS IS — and is NOT
# --------------------------------------------------------------------------------
# This computes the CACHE-DISCOUNTED EFFECTIVE INPUT TOKENS of a *read payload* —
# the bytes an arm forces into the agent's context for one task. It reuses the
# canonical §5.1 formula from scripts/agent-telemetry/metrics_row.py
# (effective_input = 1.0*fresh + 0.1*cache_read + 1.25*cache_write).
#
# It does NOT call a Claude tokenizer. The §5 GOLD path uses count_tokens (the only
# exact Claude tokenizer; tiktoken is wrong for Claude — see the claude-api skill)
# over per-session transcripts. Neither an API key nor multi-session transcripts
# are available in this environment, so token counts are ESTIMATED from a char-based
# proxy. The proxy is documented + conservative; see PREREG.md "measurement-fidelity
# caveat". Every number is runtime-only / NON-CANONICAL.

from __future__ import annotations

import importlib.util
import os
from pathlib import Path

# --- reuse the canonical effective-token formula (no re-derivation) --------------
# metrics_row.py lives at repo-root scripts/agent-telemetry/; import it by path so
# we use ITS multipliers, not a copy that could drift.
_REPO = Path(__file__).resolve().parents[2]
_MR = _REPO / "scripts" / "agent-telemetry" / "metrics_row.py"


def _load_metrics_row():
    spec = importlib.util.spec_from_file_location("metrics_row", _MR)
    mod = importlib.util.module_from_spec(spec)
    assert spec and spec.loader
    spec.loader.exec_module(mod)  # type: ignore[union-attr]
    return mod


_MROW = _load_metrics_row()
effective_input_tokens = _MROW.effective_input_tokens  # the §5.1 formula, canonical
EFF_CACHE_READ = _MROW.EFF_CACHE_READ
EFF_CACHE_WRITE = _MROW.EFF_CACHE_WRITE

# --- the documented char/token proxy --------------------------------------------
# A token is ~CHARS_PER_TOKEN characters of English prose for the Claude tokenizer
# family (a widely-published, conservative ballpark; the repo's own bench/nlq uses
# "char length as a token-budget proxy"). This is a PROXY, not count_tokens. It is
# applied IDENTICALLY to both arms, so a per-task DELTA cancels first-order proxy
# bias even if the absolute scale is off. Overridable for sensitivity analysis.
CHARS_PER_TOKEN = float(os.environ.get("PKG_AB_CHARS_PER_TOKEN", "3.8"))


def chars_to_tokens(chars: int) -> int:
    """Documented char->token PROXY (NOT count_tokens). Identical for both arms."""
    return max(1, round(chars / CHARS_PER_TOKEN)) if chars else 0


def effective(fresh_chars: int, cache_read_chars: int = 0, cache_write_chars: int = 0) -> float:
    """Effective input tokens of a read payload: char counts -> token proxy ->
    the canonical §5.1 cache-discount formula. In this read-payload model the
    payload an agent must read for the FIRST time on a task is `fresh`; amortised
    one-time costs (schema card, tool def, ingestion) that sit behind the cache
    breakpoint are charged as `cache_read` (the warm-turn 0.1x), per §1.1's
    "AGENTS.md / skills sit in the cached prefix served at ~0.1x"."""
    return effective_input_tokens(
        chars_to_tokens(fresh_chars),
        chars_to_tokens(cache_read_chars),
        chars_to_tokens(cache_write_chars),
    )
