#!/usr/bin/env bash
# Teeth for scripts/check-workflow-swallow.py — the corpus-wide guard against a GATING
# workflow step throwing away an earned failure, and against a test suite being deleted
# from CI while its own assertions still pass.
#
# Three rules this file follows, because each one has failed in this repository before:
#
#  1. MUTATE A COPY OF A REAL WORKFLOW, not a toy fixture. A synthetic fixture proves the
#     checker can parse the shape its author imagined. PART C mutates copies of the real
#     .github/workflows/docs-quality.yml and ci.yml.
#  2. INCLUDE AN UNMUTATED CONTROL. Without one, every "the mutant died" line is also
#     satisfied by a checker that always fails. C0/D0/E0/F0 are those controls.
#  3. NEVER `git checkout -- .` to restore. Every mutation happens inside a `cp -r`
#     snapshot under a mktemp dir; the real tree is never written.
#
# Behaviour-kill vs crash-kill: a mutant that dies because the checker CRASHED proves
# nothing about the assertion. `expect_finding` therefore requires the exact diagnostic
# substring AND asserts no Python traceback was emitted.
#
# Run:  bash scripts/tests/test_workflow_swallow.sh   (exit 0 = all pass)

# SC2016 (expressions don't expand in single quotes) fires on the expected-diagnostic
# needles, which quote shell syntax verbatim (`||`, `if:`) precisely so it is NOT
# expanded — matching the literal text the checker prints. SC1003 fires on the fixture
# line that ends in a real trailing backslash, which is the Y11 mutant's whole point.
# Both must be file-wide, so this directive precedes the first command.
# shellcheck disable=SC2016,SC1003

set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT/scripts/check-workflow-swallow.py"
SEAM="$ROOT/scripts/check-workflow-seam.py"
WORKFLOW="$ROOT/.github/workflows/docs-quality.yml"
ALLOWLIST="$ROOT/scripts/check-workflow-swallow.allowlist.json"

PASSES=0
FAILS=0
pass() { PASSES=$((PASSES + 1)); echo "  ok   $*"; }
fail() { FAILS=$((FAILS + 1)); echo "  FAIL $*"; }

TMP="$(mktemp -d)"
cleanup() { rm -rf "$TMP"; }
trap cleanup EXIT

# --- helpers ---------------------------------------------------------------

# expect_finding <label> <expected-substring> <output>
# A finding must be the RIGHT finding and must not be a traceback.
expect_finding() {
  local label="$1" needle="$2" out="$3"
  if printf '%s' "$out" | grep -q 'Traceback (most recent call last)'; then
    fail "$label — CRASH-KILL, not a behaviour-kill (the checker raised)"
    return
  fi
  if printf '%s' "$out" | grep -qF -- "$needle"; then
    pass "$label"
  else
    fail "$label — expected '$needle', got: $(printf '%s' "$out" | head -3 | tr '\n' ' ')"
  fi
}

expect_no_finding() {
  local label="$1" forbidden="$2" out="$3"
  if printf '%s' "$out" | grep -qF -- "$forbidden"; then
    fail "$label — FALSE POSITIVE: '$forbidden' was reported"
  else
    pass "$label"
  fi
}

# Apply an exact literal replacement, asserting the mutation actually landed. A mutation
# that silently no-ops turns the whole assertion below it into a vacuous pass — this is
# the marker check that catches it.
mutate() {
  local file="$1" from="$2" to="$3"
  python3 - "$file" "$from" "$to" <<'PY'
import sys, pathlib
path, frm, to = sys.argv[1], sys.argv[2], sys.argv[3]
p = pathlib.Path(path)
text = p.read_text(encoding="utf-8")
if frm not in text:
    sys.exit(f"MUTATION MARKER FAILED: {frm!r} not present in {path}")
p.write_text(text.replace(frm, to, 1), encoding="utf-8")
PY
}

echo "PART A — the checker's own hermetic teeth"

if out="$(python3 "$CHECKER" --self-test 2>&1)" && printf '%s' "$out" | grep -q 'self-test: OK'; then
  pass "A1 --self-test passes (classification cases + corpus mutants + clean control)"
else
  fail "A1 --self-test failed: $out"
fi

out="$(python3 "$CHECKER" --census 2>&1)"
if printf '%s' "$out" | grep -qE 'census: [0-9]+ workflow\(s\), [0-9]+ run: step\(s\)'; then
  pass "A2 --census reports a real population over the live corpus"
else
  fail "A2 --census produced no population line: $out"
fi

# The taxonomy is the reason this guard is shippable at all; assert it did not collapse
# into "everything is a finding" or "nothing is".
if printf '%s' "$out" | grep -q 'AMPLIFIER' && printf '%s' "$out" | grep -q 'CAPTURE-READ'; then
  pass "A3 the census distinguishes benign classes (AMPLIFIER / CAPTURE-READ) from findings"
else
  fail "A3 the census lost its benign classes — the taxonomy has collapsed"
fi

echo
echo "PART B — the wiring in the real docs-quality.yml"

# W1-W4: each mode must actually be invoked. A mode nobody runs is a mode that rots.
for mode in "--self-test" "--check-allowlist" "--check-corpus" "--check-test-wiring"; do
  if grep -q -- "check-workflow-swallow.py $mode" "$WORKFLOW" \
     || grep -q -- "check-workflow-swallow.py --advisory $mode" "$WORKFLOW"; then
    pass "W1-4 docs-quality.yml invokes check-workflow-swallow.py $mode"
  else
    fail "W1-4 check-workflow-swallow.py $mode is not wired into docs-quality.yml"
  fi
done

# W5/W6 — the mis-argued-wiring trap, one level up. #4385's first real CI run printed
# "5 invocation(s)" where the hand-run printed 4, because --exclude was in the typed
# command and missing from the YAML: the step's own --needle argument counted as a fifth
# gate invocation, so DELETING a real call site would have left 4 and still cleared the
# floor. Assert both flags are present in the YAML itself, not in anyone's shell history.
seam_call="$(sed -n '/check-workflow-seam\.py --workflow .*docs-quality/,/^      - /p' "$WORKFLOW" \
             | grep -A6 'needle check-workflow-swallow' | tr '\n' ' ')"
if [ -n "$seam_call" ]; then
  pass "W5 the seam checker is pointed at check-workflow-swallow.py"
else
  fail "W5 no seam invocation targets check-workflow-swallow.py"
fi
if [ -n "$seam_call" ] \
   && [[ "$seam_call" == *"--exclude check-workflow-seam.py"* ]] \
   && [[ "$seam_call" == *"--min-invocations 4"* ]]; then
  pass "W6 ...with --exclude (so its own --needle is not counted) and the 4-call floor"
else
  fail "W6 the seam invocation is missing --exclude and/or --min-invocations 4"
fi

# W7/W8 — the same trap at CORPUS level. Without the floors, a glob typo or a moved
# directory scans nothing and prints a clean bill of health.
if grep -q -- "--min-workflows 60" "$WORKFLOW" && grep -q -- "--min-run-steps 600" "$WORKFLOW"; then
  pass "W7 the corpus floors (--min-workflows/--min-run-steps) are in the YAML"
else
  fail "W7 the corpus floors are missing from the YAML — a scan of nothing would pass"
fi
if [ "$(grep -c -- "--min-run-steps" "$WORKFLOW")" -ge 3 ]; then
  pass "W8 every corpus-scanning invocation carries a floor"
else
  fail "W8 a corpus-scanning invocation has no floor"
fi

# W9 — the advisory predicate is the repository's, not this script's private opinion.
# If ci_summary_gate.py ever changes what it excludes, this guard must change with it or
# it starts calling gating steps advisory.
sig_gate="$(grep -c 'advisory|informational' "$ROOT/scripts/ci_summary_gate.py")"
sig_reg="$(grep -c 'advisory|informational' "$ROOT/scripts/check-advisory-registry.py")"
sig_this="$(grep -c 'advisory|informational' "$CHECKER")"
if [ "$sig_gate" -ge 1 ] && [ "$sig_reg" -ge 1 ] && [ "$sig_this" -ge 1 ]; then
  pass "W9 the advisory predicate is shared with ci_summary_gate.py + check-advisory-registry.py"
else
  fail "W9 the advisory predicate has drifted apart from the aggregator's"
fi

# W10 — this suite must itself be EXECUTED by CI, at command position. `shellcheck x.sh`
# does not count; that is the whole distinction the orphan check exists to make.
if python3 - "$WORKFLOW" <<'PY'
import re, sys, pathlib
text = pathlib.Path(sys.argv[1]).read_text(encoding="utf-8")
sys.exit(0 if re.search(r"(?m)^\s*bash scripts/tests/test_workflow_swallow\.sh\s*$", text) else 1)
PY
then
  pass "W10 this suite is executed (not merely shellcheck'd) by docs-quality.yml"
else
  fail "W10 this suite is never executed by CI — its own assertions would be orphaned"
fi

echo
echo "PART C — mutants against a COPY of the real docs-quality.yml (seam wiring)"

WFDIR="$TMP/workflows"
cp -r "$ROOT/.github/workflows" "$WFDIR"
COPY="$WFDIR/docs-quality.yml"
cp "$COPY" "$TMP/docs-quality.pristine.yml"

GATING_STEP='          python3 scripts/check-workflow-swallow.py --check-allowlist \'

restore_copy() { cp "$TMP/docs-quality.pristine.yml" "$COPY"; }

seam_on_copy() {
  python3 "$SEAM" --workflow "$COPY" \
    --needle check-workflow-swallow.py \
    --exclude check-workflow-seam.py --min-invocations 4 2>&1
}

# C0 — UNMUTATED CONTROL. Everything below is meaningless without this line.
out="$(seam_on_copy)"
if printf '%s' "$out" | grep -q '4 invocation(s)'; then
  pass "C0 CONTROL: the unmutated copy reports exactly 4 invocations, all sound"
else
  fail "C0 CONTROL: the unmutated copy is not clean — $out"
fi

# C1 — the Y11 shape: `|| true` written across a backslash continuation. This is the
# spelling that survived #4385's original line-by-line scan while it printed a pass.
restore_copy
mutate "$COPY" "$GATING_STEP"$'\n            --min-workflows 60 --min-run-steps 600' \
               "$GATING_STEP"$'\n            --min-workflows 60 --min-run-steps 600 \\\n            || true'
expect_finding "C1 Y11: \`|| true\` on a backslash-continuation line is caught" \
  '`||` fallback on the gate' "$(seam_on_copy)"

# C2 — the same swallow on one line.
restore_copy
mutate "$COPY" "--min-workflows 60 --min-run-steps 600" \
               "--min-workflows 60 --min-run-steps 600 || true"
expect_finding "C2 \`|| true\` on the same logical line is caught" \
  '`||` fallback on the gate' "$(seam_on_copy)"

# C3 — a step-level `if:` that can never match.
restore_copy
mutate "$COPY" '      - name: "check-workflow-swallow allowlist ratchet + corpus floors — GATING"' \
               '      - name: "check-workflow-swallow allowlist ratchet + corpus floors — GATING"
        if: github.event_name == '"'"'never'"'"''
expect_finding "C3 a step \`if:\` that can never match is caught" \
  'step carries an `if:`' "$(seam_on_copy)"

# C4 — continue-on-error.
restore_copy
mutate "$COPY" '      - name: "check-workflow-swallow allowlist ratchet + corpus floors — GATING"' \
               '      - name: "check-workflow-swallow allowlist ratchet + corpus floors — GATING"
        continue-on-error: true'
expect_finding "C4 continue-on-error on the gating step is caught" \
  'continue-on-error swallows' "$(seam_on_copy)"

# C5 — DELETE a call site. This is what --min-invocations exists for.
restore_copy
mutate "$COPY" '      - name: "check-workflow-swallow allowlist ratchet + corpus floors — GATING"
        run: |
          python3 scripts/check-workflow-swallow.py --check-allowlist \
            --min-workflows 60 --min-run-steps 600
' ''
expect_finding "C5 deleting a call site is caught by the invocation floor" \
  'call site was deleted' "$(seam_on_copy)"

# C6 — THE MIS-ARGUED TRAP, both directions. Without --exclude the seam step's own
# --needle argument is counted, so the floor of 4 is met by 3 real call sites plus the
# step's own arguments: a deleted call site passes. Assert the trap is real AND that
# --exclude closes it.
restore_copy
mutate "$COPY" '      - name: "check-workflow-swallow allowlist ratchet + corpus floors — GATING"
        run: |
          python3 scripts/check-workflow-swallow.py --check-allowlist \
            --min-workflows 60 --min-run-steps 600
' ''
without_exclude="$(python3 "$SEAM" --workflow "$COPY" \
                     --needle check-workflow-swallow.py --min-invocations 4 2>&1)"
without_rc=$?
with_exclude="$(seam_on_copy)"
# The exit CODE is the assertion, not a substring: both the OK line and the failure line
# contain "invocation(s)", so grepping for it would pass in either direction.
if [ "$without_rc" -eq 0 ] \
   && printf '%s' "$with_exclude" | grep -q 'call site was deleted'; then
  pass "C6 a deleted call site passes WITHOUT --exclude and reds WITH it (the trap is closed)"
else
  fail "C6 the --exclude trap is not demonstrated: without='$(printf '%s' "$without_exclude" | head -1)' with='$(printf '%s' "$with_exclude" | head -1)'"
fi

restore_copy

echo
echo "PART D — the allowlist RATCHET, against a copy of the real corpus"

allowlist_on_copy() {
  python3 "$CHECKER" --check-allowlist --workflows-dir "$WFDIR" \
    --allowlist "$ALLOWLIST" --min-workflows 60 --min-run-steps 600 2>&1
}

# D0 — UNMUTATED CONTROL.
out="$(allowlist_on_copy)"
if printf '%s' "$out" | grep -q 'allowlist clean'; then
  pass "D0 CONTROL: every exemption is still live against the unmutated corpus"
else
  fail "D0 CONTROL: the allowlist is not clean on an unmutated corpus — $out"
fi

# D1 — an exemption must EXPIRE when the construct it exempts goes away. Note the mutation
# is the GOOD change (removing a `|| true`): the ratchet's cost is that the entry must be
# deleted with it, and that is the intended contract.
mutate "$WFDIR/bench.yml" 'git rebase --abort 2>/dev/null || true' 'git rebase --abort'
expect_finding "D1 a stale exemption is a HARD failure (the allowlist ratchets)" \
  'STALE EXEMPTION' "$(allowlist_on_copy)"
cp "$ROOT/.github/workflows/bench.yml" "$WFDIR/bench.yml"

# D2 — editing an exempted command invalidates its exemption rather than silently
# covering the new text, because the key is a content fingerprint.
mutate "$WFDIR/formal-verification.yml" 'pkill -9 -x cbmc 2>/dev/null || true' \
                                        'pkill -9 -x cbmc --signal KILL 2>/dev/null || true'
expect_finding "D2 editing an exempted command invalidates that exemption" \
  'STALE EXEMPTION' "$(allowlist_on_copy)"
cp "$ROOT/.github/workflows/formal-verification.yml" "$WFDIR/formal-verification.yml"

# D3 — the corpus floor. A scan that reached almost nothing must red, not print OK. This
# is the anti-vacuity assertion; without it a glob typo is indistinguishable from a clean
# repository.
EMPTY="$TMP/empty-workflows"
mkdir -p "$EMPTY"
cp "$ROOT/.github/workflows/docs-quality.yml" "$EMPTY/"
expect_finding "D3 a scan that reached only 1 workflow reds instead of printing OK" \
  'the corpus glob is not reaching the workflows' \
  "$(python3 "$CHECKER" --check-allowlist --workflows-dir "$EMPTY" --allowlist "$ALLOWLIST" \
       --min-workflows 60 --min-run-steps 600 2>&1)"

echo
echo "PART E — advisory vs gating is decided STRUCTURALLY (both directions, real content)"

corpus_on_copy() {
  python3 "$CHECKER" --advisory --check-corpus --workflows-dir "$WFDIR" \
    --allowlist "$ALLOWLIST" --min-workflows 60 --min-run-steps 600 2>&1
}

cp -r "$ROOT/.github/workflows/." "$WFDIR/"

# E0 — CONTROL: exactly the one known finding, no more.
out="$(corpus_on_copy)"
if printf '%s' "$out" | grep -q '1 gating step(s) discard an earned failure'; then
  pass "E0 CONTROL: the unmutated corpus reports exactly the one known open finding"
else
  fail "E0 CONTROL: unexpected finding count on the unmutated corpus — $(printf '%s' "$out" | head -1)"
fi

# E1 — the SAME construct planted in a GATING job of a real workflow IS a finding.
mutate "$WFDIR/supply-chain.yml" '    steps:' '    steps:
      - name: planted
        run: python3 scripts/planted-gate.py --check || true'
expect_finding "E1 a planted \`|| true\` in a gating job of a real workflow is a finding" \
  'supply-chain.yml' "$(corpus_on_copy)"
cp "$ROOT/.github/workflows/supply-chain.yml" "$WFDIR/supply-chain.yml"

# E2 — the SAME construct in an ADVISORY-named job is NOT a finding. This is the half
# that makes the guard shippable, and a name list could not justify it: the job is
# excluded because ci_summary_gate.py excludes it, and check-advisory-registry.py forces
# it to carry an owner and promotion criteria.
#
# Asserted by COUNT, not by the absence of a filename. "the filename is not in the
# output" is also true of an empty output, so it would pass vacuously; the count moving
# from 1 to 1 while the identical construct in E1 moved it to 2 is the real evidence.
mutate "$WFDIR/mutants-diff.yml" '    steps:' '    steps:
      - name: planted
        run: python3 scripts/planted-gate.py --check || true'
out="$(corpus_on_copy)"
if printf '%s' "$out" | grep -q '1 gating step(s) discard an earned failure' \
   && ! printf '%s' "$out" | grep -q 'mutants-diff.yml'; then
  pass "E2 the same construct in an advisory-named job is NOT a finding (count still 1)"
else
  fail "E2 advisory-job exclusion broke — $(printf '%s' "$out" | head -1)"
fi
cp "$ROOT/.github/workflows/mutants-diff.yml" "$WFDIR/mutants-diff.yml"

# E3 — and not in a workflow that never produces a PR check-run either.
mutate "$WFDIR/nightly-full-sweep.yml" '    steps:' '    steps:
      - name: planted
        run: python3 scripts/planted-gate.py --check || true'
out="$(corpus_on_copy)"
if printf '%s' "$out" | grep -q '1 gating step(s) discard an earned failure' \
   && ! printf '%s' "$out" | grep -q 'nightly-full-sweep.yml'; then
  pass "E3 the same construct in a non-PR-triggered workflow is NOT a finding (count still 1)"
else
  fail "E3 non-PR-trigger exclusion broke — $(printf '%s' "$out" | head -1)"
fi
cp "$ROOT/.github/workflows/nightly-full-sweep.yml" "$WFDIR/nightly-full-sweep.yml"

# E4 — mention vs claim. A step whose NAME says "advisory" while it hard-gates must not be
# excused; three real steps in docs-quality.yml are exactly that shape.
mutate "$WFDIR/supply-chain.yml" '    steps:' '    steps:
      - name: Enforce the advisory registry (C2) — GATING
        run: python3 scripts/planted-gate.py --check || true'
expect_finding "E4 an 'advisory'-NAMED step that hard-gates is still a finding (mention vs claim)" \
  'supply-chain.yml' "$(corpus_on_copy)"
cp "$ROOT/.github/workflows/supply-chain.yml" "$WFDIR/supply-chain.yml"

echo
echo "PART F — a suite deleted from CI while its own assertions still pass"

orphans_on_copy() {
  python3 "$CHECKER" --advisory --check-test-wiring --workflows-dir "$WFDIR" \
    --root "$ROOT" --min-run-steps 40 2>&1
}

# F0 — CONTROL: the model-attribution suite is wired today. Asserted against the reported
# population, so an empty or crashed run cannot satisfy it by saying nothing.
out="$(orphans_on_copy)"
if printf '%s' "$out" | grep -qE 'of [0-9]+ test suite\(s\)|every one of [0-9]+ test suite' \
   && ! printf '%s' "$out" | grep -q 'scripts/tests/test_model_attribution.sh'; then
  pass "F0 CONTROL: the scan reports a real suite population and does not orphan a wired suite"
else
  fail "F0 CONTROL: $(printf '%s' "$out" | head -1)"
fi

# F1 — delete ONLY the `bash …` line. The `shellcheck …` line and a `--needle …` argument
# both survive, so a filename grep is still satisfied — and the suite no longer runs.
mutate "$WFDIR/docs-quality.yml" '          bash scripts/tests/test_model_attribution.sh
' ''
survivors="$(grep -c 'test_model_attribution.sh' "$WFDIR/docs-quality.yml")"
out="$(orphans_on_copy)"
if [ "$survivors" -ge 2 ]; then
  pass "F1a the mutated file still NAMES the suite $survivors times (a grep would pass)"
else
  fail "F1a the mutation removed every mention, so this proves nothing about grep-vs-command-position"
fi
expect_finding "F1b ...and the suite is nonetheless reported as an orphan" \
  'scripts/tests/test_model_attribution.sh' "$out"
cp "$ROOT/.github/workflows/docs-quality.yml" "$WFDIR/docs-quality.yml"

# F2 — deleting THIS suite's own call site is caught too, so the guard cannot be disarmed
# by removing the line that runs it.
mutate "$WFDIR/docs-quality.yml" '          bash scripts/tests/test_workflow_swallow.sh
' ''
expect_finding "F2 deleting THIS suite's own CI call site is caught" \
  'scripts/tests/test_workflow_swallow.sh' "$(orphans_on_copy)"
cp "$ROOT/.github/workflows/docs-quality.yml" "$WFDIR/docs-quality.yml"

# F3 — CONTROL for the floor: pointing the suite count at a floor it cannot reach reds.
expect_finding "F3 a tests-tree that reached too few suites reds instead of printing OK" \
  'the tests tree is not being reached' \
  "$(python3 "$CHECKER" --advisory --check-test-wiring --workflows-dir "$WFDIR" \
       --root "$TMP" --min-run-steps 40 2>&1)"

echo
echo "test_workflow_swallow.sh: $PASSES passed, $FAILS failed"
[ "$FAILS" -eq 0 ] || exit 1
echo "test_workflow_swallow.sh: all checks passed"
