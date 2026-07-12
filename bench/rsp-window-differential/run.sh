#!/bin/sh
# [GPT-5.6] sq-no6iy — self-asserting content differential with mutation witness.
set -eu

HERE=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT HUP INT TERM
ENVELOPE=${RSP_WINDOW_ENVELOPE:-$WORK/envelope.json}

cargo run --quiet --release --manifest-path "$HERE/Cargo.toml" -- \
  "$HERE/fixtures/srbench.ts.tsv" > "$WORK/observed.tsv"
python3 "$HERE/compare.py" --observed "$WORK/observed.tsv" \
  --golden "$HERE/fixtures/rsp4j.golden.tsv" --envelope "$ENVELOPE"

# Mutation witness: remove one binding from a temporary observed row. The exact same
# content comparator must reject it, proving this is not a timestamp/count-only gate.
python3 - "$WORK/observed.tsv" "$WORK/mutated.tsv" <<'PY'
import sys
source, destination = sys.argv[1:]
text = open(source, encoding="utf-8").read()
needle = "<http://ex/stB>|<http://ex/NY>|"
if needle not in text:
    raise SystemExit("mutation target absent")
text = text.replace(needle, "<http://ex/stB>|<http://ex/CA>|", 1)
open(destination, "w", encoding="utf-8").write(text)
PY
if python3 "$HERE/compare.py" --observed "$WORK/mutated.tsv" \
  --golden "$HERE/fixtures/rsp4j.golden.tsv" >/dev/null 2>&1; then
  echo "rsp-window-differential: mutation witness was accepted" >&2
  exit 1
fi

echo "rsp-window-differential: mutation witness rejected"
echo "rsp-window-differential: latency envelope $ENVELOPE"
