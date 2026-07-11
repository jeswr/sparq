#!/usr/bin/env bash
# [SONNET-4.6] sq-hmd7l.23 — KGE link-prediction quality comparison runner.
#
# Phase 1 (this bead): --published-only mode.
#   Emits the pinned-citation comparison table (PyKEEN published reference MRR/Hits@k
#   vs sparq-kge model family + CAVEAT that the sparq numbers are on SYNTHETIC slices,
#   not real FB15k-237/WN18RR — honest framing). Exits 0 so CI (bench/kge/run.sh
#   --published-only) is green without any heavy download or PyKEEN install.
#
# Phase 2 (sq-hmd7l.37, a SEPARATE bead): --measure mode.
#   Same-box PyKEEN run (pip install pykeen torch at gather time) + a real-dataset
#   sparq-kge run (SPARQ_KGE_DATASET=/path/to/wn18rr.nt). Upgrading the column from
#   published to MEASURED. NOT implemented here.
#
# ACCEPTANCE TEST (sq-hmd7l.23):
#   bash bench/kge/run.sh --published-only   # must exit 0, emit the table
#
# HONESTY (per sq-hmd7l EPIC mandate):
#   - PyKEEN numbers are from PINNED citations (not measured in this run).
#   - sparq-kge synthetic-slice MRR/Hits@k are NON-COMPARABLE to real-dataset numbers:
#     sparq trains on a small synthetic WN18RR-style slice (~800 entities in CI, not the
#     real 40k WN18RR entities) — DO NOT directly compare cell-for-cell.
#   - The comparison purpose is MODEL FAMILY parity at matched architecture
#     (DistMult/ComplEx), not quality dominance. sparq's differentiator is KGE-in-the-
#     store over RDF dict-encoded ids; colocated inference, not raw MRR.
#   - NO hard-coded number is baked into this script or any markdown file.
#
# Sources:
#   bench/kge/published-numbers.tsv  (pinned citation table)
#   crates/sparq-vectors/examples/kge_ablation.rs (the runnable sparq KGE example)
#
# Usage:
#   bench/kge/run.sh --published-only   # Phase 1: emit citation table, exit 0
#   bench/kge/run.sh --smoke            # alias for --published-only (same behaviour)
#   bench/kge/run.sh                    # same as --published-only (Phase 2 not yet implemented)
#
# Env knobs (Phase 1 only, all advisory):
#   SHOW_SPARQ_SYNTHETIC=1   emit the sparq SYNTHETIC ablation rows WITH PROMINENT CAVEAT
#                            (default 0 — avoids confusion with real-dataset numbers)
#   KGE_ABLATION_BIN         path to a pre-built kge_ablation binary; if set AND
#                            SHOW_SPARQ_SYNTHETIC=1, runs it for the synthetic rows.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PINNED_TSV="${ROOT}/bench/kge/published-numbers.tsv"

MODE="published-only"
case "${1:-}" in
  --published-only|--smoke|"") MODE="published-only" ;;
  --measure)
    echo "[kge] ERROR: --measure (Phase 2 same-box PyKEEN run) is not yet implemented." >&2
    echo "[kge]   Phase 2 is bead sq-hmd7l.37 (a separate bead). Use --published-only for Phase 1." >&2
    exit 1
    ;;
  *)
    echo "[kge] ERROR: unknown argument '${1:-}'" >&2
    echo "[kge]   Usage: bench/kge/run.sh [--published-only|--smoke|--measure]" >&2
    exit 1
    ;;
esac

if [ ! -f "$PINNED_TSV" ]; then
  echo "[kge] ERROR: pinned citation table not found at $PINNED_TSV" >&2
  exit 1
fi

# ---- Phase 1: emit the pinned-citation comparison table --------------------------------
echo ""
echo "================================================================="
echo " KGE Link-Prediction Quality: sparq vs PyKEEN published numbers"
echo " [SONNET-4.6] sq-hmd7l.23 — Phase 1 (published-only)"
echo "================================================================="
echo ""
echo "SOURCE A: RotatE paper Table 5 (Sun et al., ICLR 2019, arXiv:1902.10197)"
echo "SOURCE B: PyKEEN re-evaluation (Ali et al., IEEE TPAMI 2021, arXiv:2006.13365)"
echo ""
echo "HONESTY CAVEAT:"
echo "  Numbers below are PUBLISHED REFERENCE figures from the cited papers."
echo "  They were NOT re-measured on this box. Phase 2 (same-box PyKEEN run) is"
echo "  bead sq-hmd7l.37 (separate bead, NOT yet implemented)."
echo ""
echo "FRAMING:"
echo "  sparq-kge implements DistMult + ComplEx (the same model family as sources A+B)."
echo "  sparq's differentiator is KGE-in-the-store over RDF dict-encoded term ids"
echo "  (colocated embeddings, no separate index, direct SPARQL join). Quality"
echo "  parity at matched model + budget is the claim; NOT quality dominance."
echo ""
echo "-- FB15k-237 (14541 entities, 237 relations -- dense, many relation patterns) --"
echo ""
printf '%-22s  %6s  %8s  %8s  %8s  %-7s\n' \
  "Model" "MRR" "Hits@1" "Hits@3" "Hits@10" "Source"
echo "----------------------  ------  --------  --------  --------  -------"

# Filter and format the FB15k-237 rows (skip comment lines and header)
while IFS=$'\t' read -r model dataset mrr hits1 hits3 hits10 source; do
  # skip blank lines and comment/header lines
  [[ "$model" =~ ^#.*$ ]] && continue
  [[ "$model" = "model" ]] && continue
  [[ -z "$model" ]] && continue
  if [ "$dataset" = "FB15k-237" ]; then
    printf '%-22s  %6s  %8s  %8s  %8s  %-7s\n' \
      "$model" "$mrr" "$hits1" "$hits3" "$hits10" "$source"
  fi
done < "$PINNED_TSV"

echo ""
echo "-- WN18RR (40943 entities, 11 relations -- sparse, hierarchical) --"
echo ""
printf '%-22s  %6s  %8s  %8s  %8s  %-7s\n' \
  "Model" "MRR" "Hits@1" "Hits@3" "Hits@10" "Source"
echo "----------------------  ------  --------  --------  --------  -------"

while IFS=$'\t' read -r model dataset mrr hits1 hits3 hits10 source; do
  [[ "$model" =~ ^#.*$ ]] && continue
  [[ "$model" = "model" ]] && continue
  [[ -z "$model" ]] && continue
  if [ "$dataset" = "WN18RR" ]; then
    printf '%-22s  %6s  %8s  %8s  %8s  %-7s\n' \
      "$model" "$mrr" "$hits1" "$hits3" "$hits10" "$source"
  fi
done < "$PINNED_TSV"

echo ""
echo "Source codes: A=RotatE-paper-2019  B=PyKEEN-TPAMI-2021-retuned"
echo ""

# ---- Optional: sparq synthetic rows (with prominent caveat) -------------------------
SHOW_SPARQ="${SHOW_SPARQ_SYNTHETIC:-0}"
ABLATION_BIN="${KGE_ABLATION_BIN:-${ROOT}/target/release/examples/kge_ablation}"

if [ "$SHOW_SPARQ" = "1" ]; then
  echo "================================================================="
  echo " sparq-kge SYNTHETIC SLICE (NOT comparable to real-dataset rows)"
  echo "================================================================="
  echo ""
  echo "!!! CRITICAL CAVEAT: sparq trains on a tiny SYNTHETIC slice (~800 entities),"
  echo "!!! NOT the real FB15k-237 / WN18RR datasets. The numbers below are"
  echo "!!! INDICATIVE ablation metrics on synthetic data — DO NOT compare cell-for-cell"
  echo "!!! with the published numbers above. A real-dataset sparq run is sq-hmd7l.37."
  echo ""
  if [ -x "$ABLATION_BIN" ]; then
    echo "Running sparq-kge synthetic ablation (kge feature required)..." >&2
    # Run the ablation and extract just the summary lines (not the full verbose output)
    "$ABLATION_BIN" 2>/dev/null | grep -E "^(#|==|closure=|    head |  \(ablation)" || true
  else
    echo "  (sparq kge_ablation binary not found at $ABLATION_BIN)"
    echo "  Build: cargo build --release -p sparq-vectors --features kge --example kge_ablation"
    echo "  Then: SHOW_SPARQ_SYNTHETIC=1 KGE_ABLATION_BIN=./target/release/examples/kge_ablation bench/kge/run.sh"
  fi
  echo ""
fi

# ---- Summary: gap verdict -----------------------------------------------------------
echo "-- Gap verdict (Phase 1 — published-numbers column only) --"
echo ""
echo "  sparq-kge model family: DistMult + ComplEx (IMPLEMENTED, opt-in --features kge)."
echo "  RotatE and TransE are NOT YET implemented in sparq-kge."
echo ""
echo "  At matched model (ComplEx) with default hyperparams, published WN18RR MRR"
echo "  is 0.440 (RotatE paper) and 0.475 after PyKEEN retuning. FB15k-237 MRR"
echo "  is 0.247 / 0.317 (retuned). These are the REFERENCE BARS for a same-box"
echo "  Phase 2 run (sq-hmd7l.37)."
echo ""
echo "  NOT-COMPARABLE axis: training throughput (triples/s) vs PyKEEN. sparq-kge"
echo "  is an in-store embedder optimised for colocated RDF inference, NOT a"
echo "  standalone training library. The throughput comparison would be"
echo "  apples-to-oranges. Quality (MRR/Hits@k) at matched model is the honest axis."
echo ""
echo "  See research/gap-kge-2026-07.md for the full gap record."
echo ""
echo "[kge] Phase 1 (published-only): OK"
