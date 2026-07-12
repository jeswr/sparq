#!/usr/bin/env bash
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
python3 "$here/analyze.py" --check >/dev/null
echo "hybrid-retrieval RRF fixture: PASS"
