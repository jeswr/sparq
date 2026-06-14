#!/usr/bin/env bash
# [OPUS-4.8] PER-CRATE line-coverage measurement (sq-hbg7).
#
# WHY PER-CRATE (and not one `cargo llvm-cov --workspace` run)
# -----------------------------------------------------------
# A single whole-workspace instrumented run does NOT finish in a bounded CI window:
# the sparq-vectors recall/diskann HNSW accuracy gates (two NON-#[ignore]d tests,
# 50k vectors x 100 queries each) run ~minutes EACH under llvm-cov instrumentation,
# and the zk bb prove tests are heavy (those are already #[ignore]d, so the default
# test set skips them). Measuring PER-CRATE lets us (a) attribute coverage to the
# right crate, (b) apply the per-crate quirks below, and (c) EXCLUDE the pathological
# tests from the per-commit set by name (documented, no silent truncation).
#
# PER-CRATE QUIRKS this script encodes (discovered by the 2026-06-14 coverage audit,
# research/coverage-and-benchmark-plan.md §2.2/§2.3):
#   - sparq-core      MUST be measured with `--features mmap,dict-spill`. Two distinct
#                     reasons, BOTH load-bearing for a representative number:
#                       * dict-spill  -> dictspill.rs / extsort.rs (the spillable build
#                                       pipeline) are otherwise compiled out / 0%.
#                       * mmap        -> the on-disk store SECURITY surface this gate is
#                                       meant to guard is `#[cfg(feature = "mmap")]`:
#                                       `Dict::open_mmap` validation, `CompressedPerm::
#                                       from_mmap`, and the `tests/mmap_corruption_oracle.rs`
#                                       integration test (itself `#![cfg(feature="mmap")]`)
#                                       + the byte-identical save/open roundtrips. Without
#                                       `mmap` that code is compiled out, so the line% is
#                                       computed over a DIFFERENT (smaller, security-free)
#                                       denominator — a non-representative measurement.
#                     dict-spill ALREADY pulls in mmap transitively (see Cargo.toml:
#                     `dict-spill = ["mmap", ...]`), so naming `mmap` here is currently a
#                     no-op for the number (measured identical: 91.23% either way) — it is
#                     stated EXPLICITLY so a future refactor that decouples dict-spill from
#                     mmap cannot silently drop the security-code coverage this gate exists
#                     to enforce. [OPUS-4.8]
#   - sparq-vectors   the two `*_recall_at_10_vs_brute_force_on_50k` tests (HNSW +
#                     DiskANN) are EXCLUDED from the per-commit subset via `--skip`
#                     (they dominate wall-clock under instrumentation). They are
#                     measured in the NIGHTLY tier (COVERAGE_TIER=nightly) so their
#                     lines still count there.
#   - sparq-cli       reports ~0% line coverage: its tests spawn the COMPILED binary
#                     as a subprocess (assert_cmd / CARGO_BIN_EXE), which is NOT the
#                     instrumented profile. This is a measurement artifact, not a real
#                     gap — its floor is set to 0 and annotated in the floor file.
#   - sparq-gpu       GPU kernels need a device; CPU-only coverage is low by design.
#   - sparq-py        EXCLUDED entirely (pyo3 / maturin toolchain, not an llvm-cov
#                     target) and sparq-bench (no tests, harness crate).
#   - W3C fixtures    are fetched first (SPARQL + inference + SHACL); the SHACL /
#                     reason-explain / EYE / rdf-turtle suites SKIP when fixtures are
#                     absent, so without this the numbers are misleadingly low + flaky.
#
# WHAT THE CONFORMANCE BINARIES DO / DON'T contribute
# ---------------------------------------------------
# The two W3C suites run as the sparq-conformance / sparq-inference-conformance
# BINARIES (`cargo run`), NOT as `cargo test`. `cargo llvm-cov --package
# sparq-conformance` runs that crate's `cargo test` (a handful of unit tests), so its
# line% is low and is NOT a meaningful gate — its floor is set to 0 and annotated.
# The W3C suites' deep exercise of sparq-core / sparq-engine is already reflected by
# those crates' own large unit+integration test bodies; re-running the binaries under
# llvm-cov to merge their profraw into core/engine is left to the NIGHTLY tier (it
# roughly doubles the core/engine cost for a few extra points — not worth per-commit).
#
# OUTPUT: writes a machine-readable per-crate summary to $OUT (default
# target/coverage/coverage-summary.json):
#   { "generated": "<iso8601>", "tier": "...", "toolchain": "...",
#     "crates": { "<crate>": { "lines_pct": <float>, "lines_covered": N,
#                              "lines_total": N, "seconds": N, "skipped_tests": [...],
#                              "features": [...], "measured": true } } }
#
# USAGE:
#   scripts/coverage.sh                 # per-commit tier (FAST + medium crates)
#   COVERAGE_TIER=nightly scripts/coverage.sh   # all crates incl. heavy vectors
#   COVERAGE_TIER=full    scripts/coverage.sh   # every measurable crate, no skips
#   COVERAGE_CRATES="sparq-core sparq-parse" scripts/coverage.sh   # ad-hoc subset
#
# Honours the AGENTS.md "no silent truncation" rule: every crate that is excluded /
# skipped / measured-with-a-quirk is recorded explicitly in the JSON and printed.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

TIER="${COVERAGE_TIER:-per-commit}"          # per-commit | nightly | full
OUT="${COVERAGE_OUT:-$ROOT/target/coverage/coverage-summary.json}"
FETCH_FIXTURES="${COVERAGE_FETCH_FIXTURES:-1}"
mkdir -p "$(dirname "$OUT")"

# ---- crate tiers ------------------------------------------------------------
# FAST/MEDIUM crates measured EVERY commit. (Timings in the floor file header.)
PER_COMMIT_CRATES=(
  sparq-core sparq-engine sparq-cli sparq-server sparq-serve
  sparq-reason sparq-shacl sparq-geo sparq-text sparq-hdt sparq-rsp
  sparq-introspect sparq-sim sparq-solid sparq-nlq sparq-mpc
  sparq-parse sparq-gpu sparq-wasm sparq-zk sparq-zk-compose
  sparq-vectors                # measured with the two 50k tests SKIPPED (see below)
  sparq-conformance            # low %, floor 0 (test driver) — kept for presence
)
# Crates whose HEAVY tests are only run in the nightly tier.
NIGHTLY_ONLY_NOTE="sparq-vectors heavy 50k recall/diskann tests run only in nightly tier"

# Tests EXCLUDED from the per-commit subset, by name (libtest --skip substring).
# DOCUMENTED here so the exclusion is never silent.
VECTORS_HEAVY_SKIP="recall_at_10_vs_brute_force_on_50k"   # matches HNSW + DiskANN 50k

# ---- fixtures (idempotent; no-ops when already pinned) ----------------------
if [ "$FETCH_FIXTURES" = "1" ]; then
  echo "==> Fetching pinned W3C fixtures (idempotent)…"
  ./scripts/fetch-conformance.sh        || { echo "WARN: SPARQL fixture fetch failed"; }
  ./scripts/fetch-inference-suites.sh   || { echo "WARN: inference fixture fetch failed"; }
  ./crates/sparq-shacl/fetch-shacl-tests.sh || { echo "WARN: SHACL fixture fetch failed"; }
fi

# ---- pick the crate list for this run --------------------------------------
if [ -n "${COVERAGE_CRATES:-}" ]; then
  read -r -a CRATES <<<"$COVERAGE_CRATES"
else
  CRATES=("${PER_COMMIT_CRATES[@]}")
fi

TOOLCHAIN="$(rustc --version 2>/dev/null || echo unknown)"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/sparq-cov.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT

declare -a ROWS=()
TOTAL_START=$(date +%s)

measure() {
  local crate="$1"; shift
  local features=()           # array of feature names recorded in JSON
  local skips=()              # array of skipped-test substrings recorded in JSON
  local -a cargo_args=(--package "$crate")
  local -a test_args=()
  local subcmd=""             # "" => `cargo llvm-cov`, "test" => filterable form

  case "$crate" in
    sparq-core)
      # mmap is named EXPLICITLY (not relied on via the dict-spill -> mmap transitive
      # dep) so the on-disk-store security surface this gate guards is always compiled +
      # exercised. See the PER-CRATE QUIRKS header for the full rationale. [OPUS-4.8]
      cargo_args+=(--features mmap,dict-spill); features+=("mmap" "dict-spill") ;;
    sparq-vectors)
      # Only skip the heavy 50k tests OUTSIDE the nightly/full tiers.
      if [ "$TIER" = "per-commit" ]; then
        subcmd="test"; test_args+=(--skip "$VECTORS_HEAVY_SKIP"); skips+=("$VECTORS_HEAVY_SKIP")
      fi ;;
  esac

  local start end rc=0 json="$WORK/$crate.json"
  start=$(date +%s)
  if [ -n "$subcmd" ]; then
    cargo llvm-cov "$subcmd" "${cargo_args[@]}" --summary-only --json \
      --output-path "$json" -- "${test_args[@]}" >/dev/null 2>"$WORK/$crate.err" || rc=$?
  else
    cargo llvm-cov "${cargo_args[@]}" --summary-only --json \
      --output-path "$json" >/dev/null 2>"$WORK/$crate.err" || rc=$?
  fi
  end=$(date +%s)

  if [ "$rc" -ne 0 ] || [ ! -s "$json" ]; then
    echo "  !! $crate FAILED to measure (rc=$rc) — see error tail:"
    tail -8 "$WORK/$crate.err" | sed 's/^/     /'
    ROWS+=("$(python3 - "$crate" "$((end-start))" <<'PY'
import json,sys
print(json.dumps({"crate":sys.argv[1],"seconds":int(sys.argv[2]),"measured":False}))
PY
)")
    return
  fi

  local feats_json skips_json
  feats_json=$(printf '%s\n' "${features[@]:-}" | python3 -c 'import sys,json;print(json.dumps([l for l in sys.stdin.read().split("\n") if l]))')
  skips_json=$(printf '%s\n' "${skips[@]:-}"     | python3 -c 'import sys,json;print(json.dumps([l for l in sys.stdin.read().split("\n") if l]))')

  local row
  row=$(python3 - "$crate" "$json" "$((end-start))" "$feats_json" "$skips_json" <<'PY'
import json,sys
crate,path,secs,feats,skips=sys.argv[1],sys.argv[2],int(sys.argv[3]),json.loads(sys.argv[4]),json.loads(sys.argv[5])
d=json.load(open(path))
t=d["data"][0]["totals"]["lines"]   # llvm-cov shape: {count, covered, percent}
print(json.dumps({"crate":crate,"lines_pct":round(t["percent"],2),
  "lines_covered":t["covered"],"lines_total":t["count"],
  "seconds":secs,"features":feats,"skipped_tests":skips,"measured":True}))
PY
)
  ROWS+=("$row")
  printf "  %-20s lines=%6s%%  %4ss%s%s\n" "$crate" \
    "$(echo "$row" | python3 -c 'import sys,json;print(json.load(sys.stdin)["lines_pct"])')" \
    "$((end-start))" \
    "$([ ${#features[@]} -gt 0 ] && echo "  feat=${features[*]}")" \
    "$([ ${#skips[@]} -gt 0 ] && echo "  skip=${skips[*]}")"
}

echo "==> Measuring per-crate line coverage (tier=$TIER)…"
for c in "${CRATES[@]}"; do measure "$c"; done
TOTAL_END=$(date +%s)

# ---- assemble the summary JSON ---------------------------------------------
# NB: write rows to a file and pass its path as argv — do NOT pipe rows into a
# `python3 - <<'PY'` heredoc (the heredoc captures stdin, so a pipe would be lost).
ROWS_FILE="$WORK/rows.ndjson"
: > "$ROWS_FILE"
[ ${#ROWS[@]} -gt 0 ] && printf '%s\n' "${ROWS[@]}" > "$ROWS_FILE"
python3 - "$OUT" "$TIER" "$TOOLCHAIN" "$((TOTAL_END-TOTAL_START))" "$NIGHTLY_ONLY_NOTE" "$ROWS_FILE" <<'PY'
import json,sys,datetime
out,tier,toolchain,total,note,rows_file=sys.argv[1:7]
total=int(total)
crates={}
for line in open(rows_file):
    line=line.strip()
    if not line: continue
    r=json.loads(line); crates[r.pop("crate")]=r
doc={"generated":datetime.datetime.now(datetime.timezone.utc).isoformat(),"tier":tier,
     "toolchain":toolchain,"total_seconds":total,"note":note,"crates":crates}
json.dump(doc,open(out,"w"),indent=2,sort_keys=True); open(out,"a").write("\n")
print(f"\n==> wrote {out}  ({len(crates)} crates, {total}s total)")
PY
