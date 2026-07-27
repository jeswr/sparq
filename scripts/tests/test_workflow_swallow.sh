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
   && [[ "$seam_call" == *"--min-invocations 4"* ]] \
   && [[ "$seam_call" == *"--require-exec scripts/tests/test_workflow_swallow.sh"* ]]; then
  pass "W6 ...with the 4-call floor and --require-exec on THIS suite"
else
  fail "W6 the seam invocation is missing --min-invocations 4 and/or --require-exec"
fi
# W6b — `--exclude` is RETIRED. It exempted the seam step from the invocation count AND
# from every neutering check, so one additive `continue-on-error: true` disarmed the gate
# while it printed OK. Invocations are now resolved at command position; re-adding the
# flag is a hard argparse failure rather than a silently accepted no-op.
if [[ "$seam_call" == *"--exclude"* ]]; then
  fail "W6b the seam invocation still passes the retired --exclude self-exemption"
else
  pass "W6b the retired --exclude self-exemption is not passed"
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
# it starts exempting steps the merge gate still gates.
#
# CORRECTION. W9 was `grep -c 'advisory|informational' <file> >= 1` in all three files —
# a MENTION count, while the PR body claimed it "asserts the predicate is still shared
# with both consumers, so it cannot drift into calling a gating job advisory". A reviewer
# disproved that by execution: widening this script's regex by `soft-fail|non-required|
# besteffort` left 34/34 green, and replacing the pattern outright with `zzznevermatches`
# ALSO left W9 green — the module docstring quotes the old pattern, so the grep still
# found it. That is the same "grep satisfied by an adjacent line" shape this suite exists
# to eliminate, reproduced on its own headline design claim.
#
# Now structural, and in three parts:
#   (a) the compiled pattern SOURCE is byte-identical across all three files (AST-
#       extracted from the assignment, so a docstring mention cannot satisfy it);
#   (b) the predicate is not inert — it still matches `advisory` and `informational`;
#   (c) the QUANTIFIER that matters: nothing this guard exempts may be a job the
#       aggregator still gates. Probed over names including the exact widening words the
#       reviewer used, applying each predicate the way its own file applies it
#       (ci_summary_gate lowercases its input at the call site; the other two carry
#       re.IGNORECASE — asserted structurally, not assumed).
if python3 - "$ROOT" "$CHECKER" <<'PY'
import ast
import pathlib
import re
import sys

root, checker = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
FILES = {
    "aggregator": (root / "scripts/ci_summary_gate.py", "ADVISORY_NAME_TOKEN_RE"),
    "registry": (root / "scripts/check-advisory-registry.py", "ADVISORY_RE"),
    "this-guard": (checker, "ADVISORY_RE"),
}
problems = []
compiled = {}
patterns = {}

for role, (path, var) in FILES.items():
    tree = ast.parse(path.read_text(encoding="utf-8"))
    call = None
    for node in ast.walk(tree):
        if not isinstance(node, ast.Assign):
            continue
        if not any(isinstance(t, ast.Name) and t.id == var for t in node.targets):
            continue
        if isinstance(node.value, ast.Call):
            call = node.value
    if call is None:
        problems.append(f"{role}: no `{var} = re.compile(...)` assignment found in {path.name}")
        continue
    if not (call.args and isinstance(call.args[0], ast.Constant) and isinstance(call.args[0].value, str)):
        problems.append(f"{role}: {var} is not compiled from a string literal")
        continue
    patterns[role] = call.args[0].value
    flags = 0
    for arg in call.args[1:] + [kw.value for kw in call.keywords]:
        for sub in ast.walk(arg):
            if isinstance(sub, ast.Attribute) and sub.attr in ("IGNORECASE", "I"):
                flags |= re.IGNORECASE
    # How this file APPLIES the predicate: does its call site lower() the input?
    lowers = False
    for node in ast.walk(tree):
        if (isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute)
                and node.func.attr == "search"
                and isinstance(node.func.value, ast.Name) and node.func.value.id == var):
            for sub in ast.walk(node):
                if isinstance(sub, ast.Attribute) and sub.attr == "lower":
                    lowers = True
    rx = re.compile(patterns[role], flags)
    compiled[role] = (lambda rx=rx, lowers=lowers: (lambda name: bool(rx.search(name.lower() if lowers else name))))()

# (a) byte-identical pattern source
if len(set(patterns.values())) > 1:
    problems.append(f"the advisory pattern has DRIFTED apart: {patterns}")

# (b) not inert
if not problems:
    for role, pred in compiled.items():
        for must in ("advisory", "informational"):
            if not pred(f"coverage {must} lane"):
                problems.append(f"{role}: the advisory predicate no longer matches {must!r} — it is inert")

# (c) this guard may never exempt what the aggregator still gates
PROBES = [
    "advisory", "informational", "ADVISORY coverage", "Informational lane",
    "coverage shadow (soft-fail)", "non-required extras", "besteffort sweep",
    "nightly full sweep", "quick-gates", "gate", "advisory-registry check",
]
if not problems:
    for name in PROBES:
        if compiled["this-guard"](name) and not compiled["aggregator"](name):
            problems.append(
                f"job name {name!r} would be EXEMPTED by this guard while the merge "
                f"aggregator still gates it — a real gating step with a swallowed "
                f"failure would go unreported"
            )

for p in problems:
    print(f"  {p}", file=sys.stderr)
sys.exit(1 if problems else 0)
PY
then
  pass "W9 the advisory predicate is byte-identical to, and no wider than, the aggregator's"
else
  fail "W9 the advisory predicate has drifted from ci_summary_gate.py / check-advisory-registry.py"
fi

# W10 — this suite must itself be EXECUTED by CI, at command position. `shellcheck x.sh`
# does not count; that is the whole distinction the orphan check exists to make.
#
# Two corrections. (1) It was a regex over raw YAML text; it now goes through the same
# PyYAML-parsing, command-position resolver CI uses, so an indentation change or a
# `run:` written as a flow scalar cannot fool it. (2) Read its honest limit: W10 lives
# INSIDE the suite, so the mutant row "delete this suite from CI -> reds W10" is circular
# as CI evidence — once the line is gone, W10 never runs. The DURABLE check is the
# separate gating `--require-exec` on the seam step (asserted present by W6), which runs
# whether or not this suite does. W10 is the local/dev signal, and C7 below is the red
# test for the durable one.
if python3 "$SEAM" --workflow "$WORKFLOW" \
     --needle check-workflow-swallow.py --min-invocations 1 \
     --require-exec scripts/tests/test_workflow_swallow.sh >/dev/null 2>&1
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

# Byte-for-byte the invocation docs-quality.yml runs (W6 pins that), so these mutants
# test what CI executes and not a convenient variant of it.
seam_on_copy() {
  python3 "$SEAM" --workflow "$COPY" \
    --needle check-workflow-swallow.py \
    --min-invocations 4 \
    --require-exec scripts/tests/test_workflow_swallow.sh 2>&1
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

# C6 — THE MIS-ARGUED TRAP, and how it is now closed STRUCTURALLY rather than by a flag.
# #4385's first real CI run printed "5 invocation(s)" where the hand-run printed 4: the
# seam step's own `--needle check-workflow-swallow.py` argument was counted as a fifth
# invocation, so deleting a real call site would have left 4 and still cleared the floor.
# The first patch was `--exclude check-workflow-seam.py` — which then exempted that step
# from all five neutering checks as well, and is now RETIRED. Invocations are resolved at
# command position, so the meta-step's `--needle` argument is not an invocation at all.
#
# Both directions, on the pristine copy and on a deletion:
#   * pristine  -> exactly 4, i.e. the guard's own arguments are NOT inflating the count
#     (C0 asserts the same number; here it is the CONTROL for the mutation below);
#   * deleted   -> 3, reds, and names the mention-vs-execution gap.
# Asserted on the EXIT CODE, because both the pass and the fail line contain
# "invocation(s)" and a substring grep would be satisfied either way.
restore_copy
pristine_rc=0
pristine_out="$(seam_on_copy)" || pristine_rc=$?
mutate "$COPY" '      - name: "check-workflow-swallow allowlist ratchet + corpus floors — GATING"
        run: |
          python3 scripts/check-workflow-swallow.py --check-allowlist \
            --min-workflows 60 --min-run-steps 600
' ''
deleted_rc=0
deleted_out="$(seam_on_copy)" || deleted_rc=$?
if [ "$pristine_rc" -eq 0 ] && printf '%s' "$pristine_out" | grep -q '4 invocation(s)' \
   && [ "$deleted_rc" -ne 0 ] \
   && printf '%s' "$deleted_out" | grep -q 'call site was deleted' \
   && printf '%s' "$deleted_out" | grep -q 'is not being RUN'; then
  pass "C6 the guard's own --needle argument is not counted, so a deleted call site reds"
else
  fail "C6 the mis-argued trap is not closed: pristine(rc=$pristine_rc)='$(printf '%s' "$pristine_out" | head -1)' deleted(rc=$deleted_rc)='$(printf '%s' "$deleted_out" | head -1)'"
fi

# C7 — THE ONE-LINE DELETION OF THIS SUITE, which had no hard gate at all. Delete only
# `bash scripts/tests/test_workflow_swallow.sh`, leaving the adjacent `shellcheck` line.
# Measured on the previous head: EVERY leg stayed green — self-test, allowlist (GATING),
# corpus, test-wiring, seam wiring (GATING), seam self-test — while all 34 assertions
# below stopped running, disarming W5-W10, the only things holding --min-invocations and
# the corpus floors in the YAML. `--check-test-wiring` did report it, but runs --advisory
# and exits 0. The seam step's `--require-exec` is the hard gate; this is its red test.
restore_copy
mutate "$COPY" '          bash scripts/tests/test_workflow_swallow.sh
' ''
expect_finding "C7 deleting the ONE line that RUNS this suite (leaving \`shellcheck\`) is caught" \
  'never EXECUTED at command position' "$(seam_on_copy)"

# C8 — and the gating seam step is not exempt from its OWN neutering checks. Under the
# retired `--exclude` this survived: one additive `continue-on-error: true` on the seam
# step (a keyword already present elsewhere in this workflow) disarmed it silently, after
# which C7's deletion goes unreported and the job stays green. A step can never red its
# own neutering, so the catch has to come from a different step — this assertion runs
# from the suite, which CI executes as its own step.
restore_copy
mutate "$COPY" '      - name: "The workflow-swallow guard is soundly WIRED into this workflow — GATING"' \
               '      - name: "The workflow-swallow guard is soundly WIRED into this workflow — GATING"
        continue-on-error: true'
expect_finding "C8 continue-on-error on the GATING SEAM step itself is caught" \
  'continue-on-error swallows' "$(seam_on_copy)"

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
