#!/usr/bin/env bash
# Runtime tier dispatcher: pick the fastest sparq-cli binary this CPU can run.
#
# Ship the per-tier binaries from `dist/` (or .github/workflows/dist.yml artifacts)
# alongside this launcher; it selects the best one for the host and exec's it. This is the
# "fat package" alternative to per-machine compilation — one download, optimal binary.
#
#   dist/
#     sparq-cli-arm64-darwin  sparq-cli-arm64-linux
#     sparq-cli-x64-baseline  sparq-cli-x64-v3  sparq-cli-x64-v4
#   sparq-launch.sh   ->   exec dist/sparq-cli-<best-tier> "$@"
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
DIST="${SPARQ_DIST:-$HERE/dist}"

arch="$(uname -m)"; os="$(uname -s)"
pick=""

case "$os/$arch" in
  Darwin/arm64|Darwin/aarch64)
    pick=arm64-darwin ;;                       # one binary — native unlocks nothing extra
  Linux/aarch64|Linux/arm64)
    pick=arm64-linux ;;                        # neoverse/+lse build runs on all armv8.2+
  Linux/x86_64|*/x86_64|*/amd64)
    # Walk down from the richest ISA the CPU actually advertises.
    flags=" $(grep -m1 '^flags' /proc/cpuinfo 2>/dev/null | cut -d: -f2-) "
    has() { case "$flags" in *" $1 "*) return 0;; *) return 1;; esac; }
    if has avx512f && has avx512bw && has avx512vl && has avx512dq && has avx512cd; then
      pick=x64-v4                              # x86-64-v4
    elif has avx2 && has bmi2 && has fma && has movbe; then
      pick=x64-v3                              # x86-64-v3 — the common server tier
    else
      pick=x64-baseline                        # SSE2 — runs anywhere
    fi ;;
  *) echo "sparq-launch: unsupported $os/$arch" >&2; exit 1 ;;
esac

bin="$DIST/sparq-cli-$pick"
if [ ! -x "$bin" ]; then
  # Fall back to baseline if the chosen tier wasn't shipped.
  for fb in "$pick" x64-baseline arm64-linux arm64-darwin; do
    [ -x "$DIST/sparq-cli-$fb" ] && { bin="$DIST/sparq-cli-$fb"; pick="$fb"; break; }
  done
fi
[ -x "$bin" ] || { echo "sparq-launch: no usable binary in $DIST" >&2; exit 1; }
[ -n "${SPARQ_LAUNCH_VERBOSE:-}" ] && echo "sparq-launch: $os/$arch -> $pick" >&2
exec "$bin" "$@"
