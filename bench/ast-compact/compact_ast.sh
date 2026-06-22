#!/usr/bin/env bash
# [OPUS-4.8] sq (issue #1080) — COMPACTED-AST view generator for the agent-effectiveness
# A/B. 🤖 SPARQ agent. Authored by Opus 4.8 (Fable unavailable; flag for re-review).
#
# Produces a compacted structural skeleton of a Rust file: every top-level/impl item
# (struct/enum/trait/impl/fn/macro) as `LINE: <signature>`, so an agent can SEE and
# WORK OVER the file's shape without reading its bytes. This is the artifact arm B
# generates first and manipulates (the maintainer's "compacted representation" idea).
#
# Usage:  compact_ast.sh FILE.rs
# Needs:  ast-grep on PATH (skill rule: invoke as `ast-grep`, never `sg`).
set -euo pipefail
f="${1:?usage: compact_ast.sh FILE.rs}"
command -v ast-grep >/dev/null || { echo "ast-grep not on PATH" >&2; exit 2; }

# Pull the load-bearing item kinds as one-line signatures with their start line.
# We match item *headers* and print "line: first-line-of-text" so the dump is the
# skeleton, not the bodies. Sort by line, dedup.
emit() {  # pattern
  ast-grep run --pattern "$1" --lang rust "$f" --json=compact 2>/dev/null \
    | python3 -c '
import json,sys
try: d=json.load(sys.stdin)
except Exception: sys.exit(0)
for m in d:
    ln=m["range"]["start"]["line"]+1
    head=m["text"].split("{")[0].split("\n")[0].strip()
    print(f"{ln}\t{head}")
'
}

{
  emit 'struct $N $$$'
  emit 'enum $N $$$'
  emit 'trait $N $$$'
  emit 'impl $$$ { $$$ }'
  emit 'fn $N($$$) $$$'
  emit 'pub fn $N($$$) $$$'
  emit 'macro_rules! $N { $$$ }'
} | sort -n -u | awk -F'\t' '{printf "%6d  %s\n", $1, $2}'
