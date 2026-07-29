#!/usr/bin/env bash
# [OPUS-5] Hermetic both-direction self-test for the `docs-duplication (jscpd)` HARD gate
# (bead sq-lrju). Fixture-only: it never scans the repo corpus, so it costs seconds.
#
# WHY this test exists — the gate has THREE non-obvious ways to go quietly wrong:
#
#   1. VACUITY. jscpd's `exitCode` defaults to 0, so finding clones is NOT by itself
#      fatal; the teeth come entirely from `threshold: 0` causing jscpd to append its
#      `threshold` reporter, which throws. Drop `threshold` from .jscpd.json (or raise
#      it) and the lane still runs, still prints clones, and still exits 0 — a gate that
#      reports and never blocks. Case A pins the teeth; case A' pins that `threshold` is
#      what supplies them.
#
#   2. THE ALLOWLIST FORM. jscpd injects its `jscpd:ignore-start` / `jscpd:ignore-end`
#      grammar into every Prism language, but in `markdown` the OBVIOUS spelling —
#      an HTML comment, `<!-- jscpd:ignore-start -->` — does NOT tokenize as an ignore
#      marker: the markup grammar consumes `<!--` first and `jscpd:` comes back out as a
#      `namespace` token. The marker must be written as a markdown link-reference
#      comment, `[//]: # (jscpd:ignore-start)`. That failure is fail-CLOSED (a mis-spelled
#      marker leaves the gate red, never green), but it is deeply confusing, so cases B
#      and C pin both the working form and the broken one. If case C ever starts passing,
#      jscpd fixed the tokenizer upstream and the docs in .jscpd.json can be simplified.
#
#   3. THE RENDERED-BOOK EXCLUSION. `mdbook build book` writes book/book/, a rendered
#      copy of book/src plus the README/SKILL anchors it {{#include}}s — i.e. duplicated
#      by construction. It is gitignored, so it is absent on a clean CI checkout, but a
#      dirty tree would red the gate on a file nobody wrote. The exclusion glob has to be
#      `**/book/book/**` and NOT `book/book/**`, because config-file `path` entries are
#      resolved to ABSOLUTE paths before fast-glob sees them and a repo-relative ignore
#      glob therefore matches nothing. Case D pins the exclusion; case D' pins that the
#      fixture really does contain a clone, so D is not passing vacuously.
#
# The tuning knobs (mode / minTokens / minLines / threshold / format) are READ OUT of the
# real .jscpd.json, so this test exercises what ships rather than a hand-copied duplicate.
#
# Run:  bash scripts/tests/test_docs_duplication.sh   (exit 0 = all cases pass)
#       JSCPD="node …/jscpd" bash scripts/tests/test_docs_duplication.sh   (use a local jscpd)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONFIG="${ROOT}/.jscpd.json"
# Default matches the pinned invocation in .github/workflows/docs-quality.yml.
JSCPD="${JSCPD:-npx --yes jscpd@4.0.5}"

[ -f "$CONFIG" ] || { echo "FATAL: ${CONFIG} not found"; exit 2; }

# ---- lift the shipped tuning out of the real config (no hand-copied duplicate) ----
# Two statements, not `eval "$(cmd)"`: a command substitution inside `eval` swallows the
# validator's non-zero exit (eval of an empty string succeeds), which would turn a config
# regression into a confusing `unbound variable` further down instead of the FATAL it printed.
# Plain assignment lets `set -e` fail the run on the FATAL, and naming the six variables
# explicitly keeps them visible to shellcheck.
CFG_LINE="$(python3 - "$CONFIG" <<'PY'
import json, sys
KEYS = ("mode", "minTokens", "minLines", "maxLines", "maxSize", "threshold")
cfg = json.load(open(sys.argv[1]))
for key in KEYS:
    if key not in cfg:
        sys.exit(f"FATAL: .jscpd.json is missing the load-bearing key {key!r}")
if cfg.get("format") != ["markdown"]:
    sys.exit(f"FATAL: .jscpd.json format must be [\"markdown\"], got {cfg.get('format')!r}")
if cfg["threshold"] != 0:
    sys.exit(f"FATAL: .jscpd.json threshold must be 0 (the gate's only teeth), got {cfg['threshold']!r}")
if "**/book/book/**" not in cfg.get("ignore", []):
    sys.exit("FATAL: .jscpd.json must ignore '**/book/book/**' (the rendered mdBook output)")
values = [str(cfg[key]) for key in KEYS]
if any(" " in v for v in values):
    sys.exit(f"FATAL: .jscpd.json tuning values must be whitespace-free, got {values!r}")
print(" ".join(values))
PY
)"
read -r CFG_mode CFG_minTokens CFG_minLines CFG_maxLines CFG_maxSize CFG_threshold <<<"$CFG_LINE"

TUNING=(--format markdown --mode "$CFG_mode" --min-tokens "$CFG_minTokens"
        --min-lines "$CFG_minLines" --max-lines "$CFG_maxLines" --max-size "$CFG_maxSize"
        --reporters console)

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

fail=0
pass=0

# Emit N sentences of shared prose — comfortably over the shipped minTokens/minLines.
shared_block() {
  local i
  for i in $(seq 1 12); do
    echo "Shared documentation sentence number ${i}, present verbatim in both fixture pages."
  done
}

# run_case <label> <expected-exit: 0|nonzero> <dir> [extra jscpd args...]
run_case() {
  local label="$1" expect="$2" dir="$3"; shift 3
  local out status
  set +e
  out="$(cd "$WORK" && $JSCPD "${TUNING[@]}" "$@" "$dir" 2>&1)"
  status=$?
  set -e
  local ok
  if [ "$expect" = "0" ]; then
    [ "$status" -eq 0 ] && ok=1 || ok=0
  else
    [ "$status" -ne 0 ] && ok=1 || ok=0
  fi
  if [ "$ok" = "1" ]; then
    pass=$((pass + 1))
    echo "PASS  ${label}  (exit ${status}, expected ${expect})"
  else
    fail=$((fail + 1))
    echo "FAIL  ${label}  (exit ${status}, expected ${expect})"
    printf '        | %s\n' "${out//$'\n'/$'\n'        | }"
  fi
}

# --------------------------------------------------------------------------- #
# A / A' — TEETH. Two pages sharing a block must red *because of* threshold 0.
# --------------------------------------------------------------------------- #
mkdir -p "${WORK}/teeth"
for page in a b; do
  { echo "# Page ${page}"; echo; echo "Unique intro for page ${page}."; echo
    shared_block
    echo; echo "Unique tail for page ${page}."; } > "${WORK}/teeth/${page}.md"
done
run_case "A  duplicate prose reds the gate"                nonzero teeth --threshold "$CFG_threshold"
run_case "A' the teeth come from threshold, not clone count" 0      teeth

# --------------------------------------------------------------------------- #
# B — the WORKING allowlist form: a markdown link-reference comment.
# --------------------------------------------------------------------------- #
mkdir -p "${WORK}/allowed"
for page in a b; do
  { echo "# Page ${page}"; echo; echo "Unique intro for page ${page}."; echo
    echo "[//]: # (jscpd:ignore-start)"; echo
    shared_block
    echo; echo "[//]: # (jscpd:ignore-end)"; echo
    echo "Unique tail for page ${page}."; } > "${WORK}/allowed/${page}.md"
done
run_case "B  [//]: # (jscpd:ignore-*) allowlists the repeat" 0 allowed --threshold "$CFG_threshold"

# --------------------------------------------------------------------------- #
# C — the BROKEN form. An HTML comment does NOT allowlist in markdown; the gate
#     stays red (fail-closed). Flip to expecting 0 only when jscpd fixes this.
# --------------------------------------------------------------------------- #
mkdir -p "${WORK}/htmlcomment"
for page in a b; do
  { echo "# Page ${page}"; echo; echo "Unique intro for page ${page}."; echo
    echo "<!-- jscpd:ignore-start -->"
    shared_block
    echo "<!-- jscpd:ignore-end -->"; echo
    echo "Unique tail for page ${page}."; } > "${WORK}/htmlcomment/${page}.md"
done
run_case "C  the HTML-comment marker does NOT allowlist (fail-closed)" nonzero htmlcomment --threshold "$CFG_threshold"

# --------------------------------------------------------------------------- #
# D / D' — the rendered-mdBook exclusion, exercised through a CONFIG FILE so the
#          `path` entries are absolutised exactly as they are in the real run.
# --------------------------------------------------------------------------- #
mkdir -p "${WORK}/bookcase/book/src" "${WORK}/bookcase/book/book"
{ echo "# Guide page"; echo; shared_block; } > "${WORK}/bookcase/book/src/page.md"
cp "${WORK}/bookcase/book/src/page.md" "${WORK}/bookcase/book/book/page.md"
python3 - "${WORK}/bookcase" "$CFG_mode" "$CFG_minTokens" "$CFG_minLines" <<'PY'
import json, sys
root, mode, min_tokens, min_lines = sys.argv[1], sys.argv[2], int(sys.argv[3]), int(sys.argv[4])
base = {"path": ["book"], "format": ["markdown"], "mode": mode,
        "minTokens": min_tokens, "minLines": min_lines, "threshold": 0}
json.dump({**base, "ignore": ["**/book/book/**"]}, open(f"{root}/with-ignore.json", "w"))
json.dump({**base, "ignore": []}, open(f"{root}/no-ignore.json", "w"))
PY
run_case "D  rendered book/book/ output is excluded"                 0       bookcase --config "${WORK}/bookcase/with-ignore.json"
run_case "D' …and the fixture really is a clone without that glob"   nonzero bookcase --config "${WORK}/bookcase/no-ignore.json"

echo
if [ "$fail" -ne 0 ]; then
  echo "docs-duplication self-test: ${pass} passed, ${fail} FAILED"
  exit 1
fi
echo "docs-duplication self-test: all ${pass} cases passed"
