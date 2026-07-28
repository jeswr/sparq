#!/usr/bin/env bash
# [OPUS-5] sq-98w7z.3 — one-command A/B of the two oxttl legs, plus an optional
# per-commit attribution probe.
#
#   ./ab.sh                      # generate the corpus if absent, then A/B both legs
#   ./ab.sh --corpus my.ttl      # A/B against a real corpus (e.g. the sq-wrn61 slice)
#   ./ab.sh --attribute <sha>    # extra leg pinned to an arbitrary oxigraph commit
#
# Every timing this prints is NON-CANONICAL: it is whatever box you ran it on.
# The allocation counts are not — they are deterministic for a given corpus.
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

for leg in released upstream; do
  bin="oxttl-prefix-alloc-$leg"
  ( cd "$leg" && "${CARGO[@]}" build --release >/dev/null )
  "./$leg/target/release/$bin" bench "$CORPUS" --iters "$ITERS"
  echo
  # Separate target dir: a count-alloc build must never overwrite the timed one.
  ( cd "$leg" && "${CARGO[@]}" build --release --features count-alloc --target-dir target-count >/dev/null )
  "./$leg/target-count/release/$bin" bench "$CORPUS" --iters 3 | sed -n '/allocs\/parse/,$p'
  echo
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
