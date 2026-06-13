#!/usr/bin/env bash
# Shared config for the Wikidata 8B run. Source from the other scripts.
# EVERYTHING defaults to dry-run: export EXECUTE=1 to mutate anything.

export AWS_PROFILE="${AWS_PROFILE:-pss}"
REGION="${REGION:-eu-west-2}"

# --- spec (see RUNBOOK §1) -------------------------------------------------
INSTANCE_TYPE="${INSTANCE_TYPE:-r8g.large}"
VOL_GB="${VOL_GB:-2200}"
VOL_IOPS="${VOL_IOPS:-4000}"
VOL_THROUGHPUT="${VOL_THROUGHPUT:-125}"   # baseline; >125 is wasted on r8g.large (78.75 MB/s sustained)
DEADMAN_MIN="${DEADMAN_MIN:-2400}"        # 40 h hard cost ceiling ($17.04)

# --- sparq build -----------------------------------------------------------
# MUST be a public-main commit WITH dict-spill merged. Empty = refuse to launch.
SPARQ_SHA="${SPARQ_SHA:-}"
CHUNK_MILLIONS="${CHUNK_MILLIONS:-32}"
# TODO(dict-spill): verify env names + values against the merged implementation.
# Candidates from in-flight branch ../sparq-dictspill @ 5069b33:
#   SPARQ_DICT_SPILL=1|on|auto, SPARQ_DICT_SPILL_BUDGET_MB, SPARQ_DICT_SPILL_DISK_FLOOR_MB
DICT_SPILL_ENV="${DICT_SPILL_ENV:-SPARQ_DICT_SPILL=1 SPARQ_DICT_SPILL_BUDGET_MB=8192}"
# TODO(dict-spill): verify cargo feature plumbing — on the in-flight branch the
# feature lives on sparq-core only (sparq-cli does not re-export it).
CARGO_FEATURES="${CARGO_FEATURES:---features sparq-core/dict-spill}"

# --- data ------------------------------------------------------------------
DUMP_URL="${DUMP_URL:-https://dumps.wikimedia.org/wikidatawiki/entities/latest-truthy.nt.gz}"
DL_WORKERS="${DL_WORKERS:-4}"

# --- abort thresholds (RUNBOOK §9) ------------------------------------------
BUILD_TIMEOUT_S="${BUILD_TIMEOUT_S:-86400}"     # 24 h build hard stop
SWAP_ABORT_MIB="${SWAP_ABORT_MIB:-24576}"       # sustained swap > 24 GiB => spill broken
DISK_ABORT_FREE_GB="${DISK_ABORT_FREE_GB:-50}"  # < 50 GB free => die with logs intact
PARSE_CHECKPOINT_S="${PARSE_CHECKPOINT_S:-18000}" # parse marker absent at T+5 h => abort

# --- safety ------------------------------------------------------------------
PROTECTED_ID="i-090531b4ede8f2d3f"   # prod-solid-server: NEVER touched
TAG_PURPOSE="sparq-hw-validation"
INSTANCE_RATE_USD="0.13838"          # eu-west-2 on-demand, price list 2026-06-12
HOURLY_BURN_USD="0.426"              # instance + gp3 2200GB + 1000 extra IOPS

# --- state -------------------------------------------------------------------
SCRIPTS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
STATE="$SCRIPTS_DIR/../state"
RESULTS="$SCRIPTS_DIR/../results"
mkdir -p "$STATE"

# --- dry-run executor ----------------------------------------------------------
# X "description" cmd args...   — prints in dry-run, executes under EXECUTE=1.
# XS "description" 'shell string' — same, via bash -c (for pipes/redirections).
DRY=$([ "${EXECUTE:-0}" = "1" ] && echo 0 || echo 1)
X()  { local d="$1"; shift; if [ "$DRY" = 1 ]; then printf 'DRY  [%s]\n  + %s\n' "$d" "$*"; else echo "+ $*"; "$@"; fi; }
XS() { local d="$1"; shift; if [ "$DRY" = 1 ]; then printf 'DRY  [%s]\n  + %s\n' "$d" "$1"; else echo "+ $1"; bash -c "$1"; fi; }
banner() {
  if [ "$DRY" = 1 ]; then
    echo "=== DRY RUN (no AWS calls, no spend). Export EXECUTE=1 to run for real. ==="
  else
    echo "=== EXECUTE MODE — real AWS mutations follow ==="
  fi
}
