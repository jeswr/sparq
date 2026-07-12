#!/usr/bin/env bash
set -euo pipefail

# [GPT-5.6] Standalone runner for the existing in-crate read-response allocation example.
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
MODE="${1:---release}"
case "$MODE" in
  --smoke) PROFILE=debug; CARGO_PROFILE=() ;;
  --release) PROFILE=release; CARGO_PROFILE=(--release) ;;
  *) echo "usage: $0 [--smoke|--release]" >&2; exit 2 ;;
esac

STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
RESULT_DIR="${LWS_CORE_READPATH_RESULTS_DIR:-$HERE/results}"
ENVELOPE="${LWS_CORE_READPATH_ENVELOPE:-$RESULT_DIR/lws-core-readpath-$STAMP.json}"
RAW="$(mktemp)"
trap 'rm -f "$RAW"' EXIT
cd "$ROOT"
cargo run -p sparq-lws-core "${CARGO_PROFILE[@]}" --example read_response_alloc_microbench | tee "$RAW"
python3 "$HERE/emit_envelope.py" --input "$RAW" --output "$ENVELOPE" \
  --revision "$(git rev-parse HEAD)" --rustc "$(rustc --version)" --profile "$PROFILE"
python3 - "$ENVELOPE" <<'PY'
# [GPT-5.6] Mutation witness: malformed or internally inconsistent output cannot pass smoke.
import json
import pathlib
import sys

data = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
measurements = data["measurements"]
assert data["correctness_gate"] == "byte_identical_headers_asserted_by_example"
assert measurements["saved"] == max(measurements["before"] - measurements["after"], 0)
PY
echo "envelope: $ENVELOPE"
