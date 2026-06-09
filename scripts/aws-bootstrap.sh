#!/usr/bin/env bash
# Bootstrap a fresh host (AWS c7i / c7g Ubuntu, the Dell XPS, or any Linux box) for sparq's
# per-platform hardware benchmark + tuning, then run the harness.
#
# On the target host:
#   git clone <sparq repo> && cd sparq        # (or scp the tree over)
#   ./scripts/aws-bootstrap.sh [scale_entities]
#
# Installs the Rust toolchain + build deps, runs scripts/hw-bench.sh, and prints the CSV row.
# Send that row back (or just the printed table) and I'll hand-tune any loop whose tier-vs-
# native delta is large for this ISA (AVX2/AVX-512 on x86, Neoverse on Graviton).
set -euo pipefail
SCALE="${1:-500000}"

echo "== host: $(uname -s)/$(uname -m), $(nproc 2>/dev/null || echo '?') cores =="

# Build deps (Debian/Ubuntu; harmless if already present).
if command -v apt-get >/dev/null; then
  sudo apt-get update -qq
  sudo apt-get install -y -qq build-essential git pkg-config curl
fi

# Rust toolchain.
if ! command -v cargo >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
fi
# shellcheck disable=SC1091
[ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"

[ -f Cargo.toml ] || { echo "run this from the sparq repo root (Cargo.toml not found)"; exit 1; }

echo "== running the per-platform benchmark (scale=$SCALE) =="
./scripts/hw-bench.sh "$SCALE" hw-bench-results.csv

echo
echo "Done. Paste the row above (or the hw-bench-results.csv) back. If a column's tier value"
echo "is much faster than native, that loop autovectorizes well for this ISA already; if a"
echo "hot loop shows little tier uplift, it's the candidate for a hand-written AVX2/AVX-512/"
echo "NEON kernel — tell me and I'll write + validate it against this host."
