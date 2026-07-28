#!/usr/bin/env bash
# [OPUS-5] sq-98w7z.3 — one-command A/B of the two oxttl legs, plus an optional
# per-commit attribution probe.
#
#   ./ab.sh                      # generate the corpus if absent, then A/B both legs
#   ./ab.sh --corpus my.ttl      # A/B against a real corpus (e.g. the sq-wrn61 slice)
#   ./ab.sh --attribute <sha>    # extra leg pinned to an arbitrary oxigraph commit
#
# Both legs are built in every configuration the README, the registry entry and
# research/oxttl-prefixed-name-alloc-2026-07.md quote — default, mimalloc, rdf-12,
# count-alloc and rdf-12+count-alloc, ten builds in all — so one command really
# does reproduce every recorded row.
#
# Every timing this prints is NON-CANONICAL: it is whatever box you ran it on.
# The allocation counts come from no clock, so they do not move with box speed or
# load: repeated runs of the SAME binary over the same corpus return the same
# counts. That is not box-independence — the shim counts every allocation the whole
# binary makes, so toolchain, target, dependency resolution and features can all
# move them. Record the configuration this script prints alongside any count.
set -euo pipefail

cd "$(dirname "$0")"
HERE="$PWD"
CORPUS="$HERE/data/wd-shape-60mb.ttl"
ITERS=5
ATTRIBUTE=()

while [ $# -gt 0 ]; do
  case "$1" in
    --corpus) CORPUS="$2"; shift 2 ;;
    --iters) ITERS="$2"; shift 2 ;;
    --attribute) ATTRIBUTE+=("$2"); shift 2 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done

# The repo pins a toolchain this harness does not need and that a sandbox may not
# be able to install; honour an explicit override if the caller set one.
CARGO=(cargo)
if [ -n "${HARNESS_TOOLCHAIN:-}" ]; then CARGO=(cargo "+${HARNESS_TOOLCHAIN}"); fi

echo "# oxttl prefixed-name A/B — $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo
echo "Box: $(uname -srm) / $(nproc) cores. NON-CANONICAL — timings are box-specific."
echo

if [ ! -f "$CORPUS" ]; then
  echo "corpus $CORPUS absent — generating" >&2
  mkdir -p "$(dirname "$CORPUS")"
  ( cd released && "${CARGO[@]}" build --release >/dev/null )
  ./released/target/release/oxttl-prefix-alloc-released gen 60 "$CORPUS"
fi

# "features|target-dir|mode" — one entry per configuration the docs claim this
# command reproduces. Each gets its OWN target dir, so no build ever overwrites
# another's binary (a count-alloc build must never clobber a timed one). `time`
# legs print the wall-clock table; `count` legs drop ONLY that table — the
# counting shim's atomics perturb the clock — and keep the configuration block
# (corpus, oxttl features, allocator, iterations, digest) that identifies which
# build each allocation row came from.
LEGS=(
  "|target|time"                                 # system allocator, default features
  "mimalloc|target-mi|time"                      # the allocator sparq-cli ingest ships with
  "rdf-12|target-rdf12|time"                     # the feature set the sparq workspace enables
  "count-alloc|target-count|count"               # allocation counts, default features
  "rdf-12,count-alloc|target-rdf12-count|count"  # ... and under rdf-12, to check they match
)

for leg in released upstream; do
  bin="oxttl-prefix-alloc-$leg"
  for spec in "${LEGS[@]}"; do
    IFS='|' read -r feats dir mode <<<"$spec"
    args=(build --release --target-dir "$dir")
    if [ -n "$feats" ]; then args+=(--features "$feats"); fi
    ( cd "$leg" && "${CARGO[@]}" "${args[@]}" >/dev/null )
    if [ "$mode" = count ]; then
      # Name the exact cargo features, then suppress the wall-clock table (its
      # header plus the separator and the single data row) while keeping every
      # other line, so the two count configurations are told apart in the
      # captured stdout by more than their identical build name.
      echo "### cargo features: ${feats}"
      "./$leg/$dir/release/$bin" bench "$CORPUS" --iters 3 \
        | awk '/^\| build \| oxttl \| s \(min\)/ { skip = 3 } skip { skip--; next } { print }'
    else
      "./$leg/$dir/release/$bin" bench "$CORPUS" --iters "$ITERS"
    fi
    echo
  done
done

# Per-commit attribution: "which upstream commit actually bought the delta?".
# Generates a throwaway wrapper manifest per sha under .attrib/ (gitignored) from
# the upstream manifest, so the measured source stays the single shared main.rs.
for sha in ${ATTRIBUTE+"${ATTRIBUTE[@]}"}; do
  d="$HERE/.attrib/$sha"
  mkdir -p "$d"
  sed -e "s/9af7d59/$sha/g" \
      -e "s/oxttl-prefix-alloc-upstream/ox-$sha/g" \
      -e "s|path = \"../src/main.rs\"|path = \"$HERE/src/main.rs\"|" \
      upstream/Cargo.toml > "$d/Cargo.toml"
  ( cd "$d" && "${CARGO[@]}" build --release >/dev/null )
  "$d/target/release/ox-$sha" bench "$CORPUS" --iters "$ITERS"
  echo
done
