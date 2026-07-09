#!/usr/bin/env bash
# [OPUS-4.8] (sq-k0km) Solid WAC/ACP per-commit runner — the self-asserting entry point CI
# calls, mirroring bench/hdt/run.sh + bench/rsp/run.sh + bench/fts/run.sh. Self-contained:
#   1. run the sparq-solid `bench` EXAMPLE (the crate is isolated — not a sparq-cli
#      dependency, so the runner is a crate example, like HDT's bench_oracle / RSP's
#      rsp_oracle). It builds the in-crate ~1.1k-graph WAC fixture (+ the ACP variant),
#      materialises each auth view, and runs the authorized-subset queries.
#   2. PARSE the example's human-readable stdout into the DETERMINISTIC STRUCTURAL COUNTS
#      (named-graph / quad / auth-triple / authorized-row counts — all a pure function of
#      the FIXED in-crate fixture, so byte-stable across machines: NOT wall-clock) and
#      ASSERT each vs expected.tsv (exit 1 on any drift — the HARD correctness gate). The
#      example does NOT emit a TSV itself, so the parse lives here (no crate change needed):
#      these counts are the deterministic load-bearing signal, the ms timings the example
#      also prints are NON-CANONICAL (dev/gh-runner box) and are NOT harvested.
#   3. forward on STDOUT the 3-column `<metric>\t<value>\t<unit>` contract the ci-bench
#      `solid` hook consumes — the asserted DETERMINISTIC counts, every one a `count` unit.
#      Correctness lives in expected.tsv, exactly like LUBM / FTS / HDT / RSP.
#
# These are the access-control auth-view materialisation COUNTS (how many graphs/quads the
# WAC + ACP fixtures contain, how many triples each materialised auth view holds, how many
# graphs are visible to alice, and the authorized-subset row count of the title query). They
# light up the dashboard's Solid / access-control family row (familyOf: `solid_*` -> solid).
# This is a TREND/coverage feed, NOT a head-to-head competitor card (no Solid peer engine
# computes a like-for-like WAC auth view — featured = false in bench/benchmarks.toml).
#
# Usage: bench/solid/run.sh   (run from anywhere; honours $SOLID_BIN override)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
EXP="$ROOT/bench/solid/expected.tsv"
# The runner: a sparq-solid example (the crate is isolated — not a sparq-cli dependency).
# Override with $SOLID_BIN for a custom build. NB the example's target name is `bench`, which
# collides with other crates' `--example bench` binaries at target/release/examples/bench;
# the ci-bench hook builds THIS crate's example immediately before invoking run.sh so the
# binary on disk is sparq-solid's, but $SOLID_BIN lets a caller point at an unambiguous path.
BIN="${SOLID_BIN:-$ROOT/target/release/examples/bench}"

if [ ! -x "$BIN" ]; then
  echo "[solid] ERROR: sparq-solid bench example not found at $BIN" >&2
  echo "[solid]   build: cargo build --release -p sparq-solid --example bench" >&2
  exit 1
fi

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# ---- 1. run the example (human-readable stdout; deterministic structural counts within) ---
"$BIN" > "$TMP/solid.out" 2>"$TMP/solid.err" || {
  echo "[solid] ERROR: sparq-solid bench example exited non-zero" >&2
  cat "$TMP/solid.err" >&2 || true
  exit 1
}

# ---- 2. extract the DETERMINISTIC counts from the stdout (pure function of the fixture) ----
# Each grep targets a stable, anchored phrase the example prints. A missing capture leaves the
# var empty so the expected.tsv cross-check below fails loudly (never a silent skip). NB: each
# capture ends `|| true` so that under `set -euo pipefail` a NON-matching grep (format drift)
# yields an empty var + a LOUD expected.tsv failure below, instead of a silent `set -e` abort
# of the whole script with no diagnostic (the exact bug this file hit after #1629, sq-yxtkm).
emit() { printf '%s\t%s\t%s\n' "$1" "$2" "count" >> "$TMP/solid.tsv"; }
: > "$TMP/solid.tsv"

# "WAC fixture: <G> named graphs, <Q> quads"
wac_graphs=$(grep -oE 'WAC fixture: [0-9]+ named graphs' "$TMP/solid.out" | grep -oE '[0-9]+' | head -1 || true)
wac_quads=$(grep -oE 'WAC fixture: [0-9]+ named graphs, [0-9]+ quads' "$TMP/solid.out" | grep -oE '[0-9]+ quads' | grep -oE '[0-9]+' | head -1 || true)
# The first "auth view: <T> triples" line is the WAC materialisation; the second is ACP.
mapfile -t auth_triples < <(grep -oE 'auth view: [0-9]+ triples' "$TMP/solid.out" | grep -oE '[0-9]+' || true)
wac_auth=${auth_triples[0]:-}
acp_auth=${auth_triples[1]:-}
# "alice readable graphs: <N>"
alice_graphs=$(grep -oE 'alice readable graphs: [0-9]+' "$TMP/solid.out" | grep -oE '[0-9]+' | head -1 || true)
# The FULL-dataset GRAPH-?g query row count (first "rows: <N>" line) and the authorized-subset
# row count (the "rows: <N> (authorized subset...)" line — v1 and v2 assert-equal in the
# example). The parenthetical may carry a trailing annotation (e.g. "(authorized subset,
# union-always)" / "(authorized subset, union)" since #1629), so match up to the closing paren
# rather than requiring ")" immediately after "subset".
full_rows=$(grep -oE '    rows: [0-9]+' "$TMP/solid.out" | grep -oE '[0-9]+' | head -1 || true)
auth_rows=$(grep -oE 'rows: [0-9]+ \(authorized subset[^)]*\)' "$TMP/solid.out" | grep -oE '[0-9]+' | head -1 || true)

[ -n "$wac_graphs" ]  && emit solid_wac_named_graphs      "$wac_graphs"
[ -n "$wac_quads" ]   && emit solid_wac_quads             "$wac_quads"
[ -n "$wac_auth" ]    && emit solid_wac_auth_triples      "$wac_auth"
[ -n "$acp_auth" ]    && emit solid_acp_auth_triples      "$acp_auth"
[ -n "$alice_graphs" ]&& emit solid_alice_readable_graphs "$alice_graphs"
[ -n "$full_rows" ]   && emit solid_full_dataset_rows     "$full_rows"
[ -n "$auth_rows" ]   && emit solid_authorized_rows       "$auth_rows"

# ---- 3. assert vs expected.tsv (exit 1 on any drift) + forward the hook contract ----------
fail=0
seen=0
while IFS=$'\t' read -r metric value unit; do
  [ -n "$metric" ] || continue
  exp="$(awk -F'\t' -v k="$metric" '$1==k{print $2}' "$EXP")"
  if [ -z "$exp" ]; then
    echo "[solid] ERROR: $metric has no expected.tsv entry" >&2; fail=1; continue
  fi
  if [ "$value" != "$exp" ]; then
    echo "[solid] ERROR: $metric=$value expected=$exp (Solid auth-view count regression)" >&2; fail=1
  fi
  seen=$((seen+1))
  printf '%s\t%s\t%s\n' "$metric" "$value" "$unit"
done < "$TMP/solid.tsv"

# Every expected metric must have been emitted (a vanished count = a parse break or regression).
exp_rows=$(grep -cvE '^#|^$' "$EXP")
if [ "$seen" != "$exp_rows" ]; then
  echo "[solid] ERROR: emitted $seen count metrics, expected $exp_rows (a count metric vanished — parse break or fixture change)" >&2; fail=1
fi

if [ "$fail" = 0 ]; then
  echo "[solid] OK: all $seen WAC/ACP auth-view counts match expected.tsv (graphs/quads/auth-triples/authorized-rows)" >&2
else
  echo "[solid] FAILED: one or more auth-view counts diverged from expected.tsv" >&2
fi
exit "$fail"
