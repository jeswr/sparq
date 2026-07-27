#!/usr/bin/env bash
# Named test for the model-attribution gate (scripts/check-model-attribution.py).
#
# Two halves, because this repo has MEASURED that a gate's Python can be
# perfectly correct while the gate is still vacuous at the workflow seam:
#
#   PART A — the gate's own teeth. Drive the REAL script against planted
#            fixtures and assert it reds on each defect class and stays green
#            on each legitimate shape. Includes MUTATION checks: break the
#            thing the gate is supposed to catch and require a non-zero exit.
#
#   PART B — the YAML seam. Assert docs-quality.yml actually INVOKES the script
#            on every PR, that the invocation is not skipped by an `if:`, and
#            that its exit status is not swallowed. Deleting the call site must
#            fail THIS test, not merely go unnoticed.
#
#            The swallow/skip analysis itself lives in
#            scripts/check-workflow-seam.py, NOT in a heredoc here. It used to
#            be a heredoc, and that is exactly how it shipped broken: it matched
#            `|| true` LINE BY LINE, so a `|| true` written after a backslash
#            continuation was invisible and the whole seam test passed 20/20
#            over a hard gate whose exit status was discarded. A checker you
#            cannot point a mutant at is a checker nobody mutates. B3 below now
#            does point mutants at it — including that one.
#
# Fixtures below contain literal marker strings while attributing nothing:
# model-attribution-exempt: fixtures for the marker gate
#
# Stdlib/POSIX only, hermetic, no network.
# SC2016 (expressions don't expand in single quotes) is exactly what the fixtures
# below want: the backticks and `[MARKER]` brackets are LITERAL markdown that the
# gate must read verbatim, not shell to be evaluated. Disabled file-wide (this
# directive precedes the first command, so it applies to the whole file) with that
# reason, rather than silenced per-line.
# shellcheck disable=SC2016
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GATE="$REPO_ROOT/scripts/check-model-attribution.py"
SEAM="$REPO_ROOT/scripts/check-workflow-seam.py"
WORKFLOW="$REPO_ROOT/.github/workflows/docs-quality.yml"
NEEDLES=(--needle check-model-attribution.py --needle test_model_attribution.sh
         --exclude check-workflow-seam.py --min-invocations 4
         --require-exec scripts/tests/test_model_attribution.sh)
FAILURES=0

pass() { printf '  ok   %s\n' "$1"; }
fail() { printf '  FAIL %s\n' "$1"; FAILURES=$((FAILURES + 1)); }

# expect_rc <expected-rc> <description> <command...>
expect_rc() {
  local want="$1" desc="$2"; shift 2
  local got=0
  "$@" >/dev/null 2>&1 || got=$?
  if [ "$got" = "$want" ]; then pass "$desc"; else fail "$desc (want rc=$want, got rc=$got)"; fi
}

echo "PART A — the gate's own teeth"

expect_rc 0 "the script's hermetic --self-test passes" python3 "$GATE" --self-test
expect_rc 0 "the REAL repo's agent briefs pass (allowlisted debt aside)" \
  python3 "$GATE" --repo "$REPO_ROOT" --check-briefs

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# --- A1: the reported defect shape reds -------------------------------------
mkdir -p "$TMP/briefs/.claude/agents"
cat >"$TMP/briefs/.claude/agents/bad.md" <<'EOF'
- **Model provenance (you are Sonnet, not Opus).** Tag your notes/code with
  **`[SONNET-4.6]`** and commit with **`Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>`**.
EOF
expect_rc 1 "A1 hard-coded 'you are Sonnet' brief directive is REJECTED" \
  python3 "$GATE" --repo "$TMP/briefs" --check-briefs

# --- A2: legitimate authorship stamps must NOT red --------------------------
# If this ever reds, the gate would be forcing agents to rewrite accurate
# history, which AGENTS.md item 5 explicitly forbids.
mkdir -p "$TMP/good/.claude/agents"
cat >"$TMP/good/.claude/agents/good.md" <<'EOF'
<!-- [OPUS-4.8] Single-source: AGENTS.md wins if this drifts. -->
## Engine-semantics trap catalog [FABLE-5]
[OPUS-4.8] Authored by Opus 4.8 (1M context); flag for re-review.
- **Markers follow the RUNNING model.** Derive your inline marker + `Co-Authored-By`
  trailer from the harness's actual runtime model.
- Existing `[OPUS-4.8]` stamps in this file are accurate history — leave them.
- Use `Co-Authored-By: Claude MODEL <noreply@anthropic.com>`.
EOF
expect_rc 0 "A2 legitimate authorship stamps + model-aware wording PASS" \
  python3 "$GATE" --repo "$TMP/good" --check-briefs

# --- A3: MUTATION — plant a wrong marker in the good brief; it must red -----
# This is the deletion question: if this mutant survives, the gate is vacuous.
cp -r "$TMP/good" "$TMP/mutant"
printf -- '- Tag your code with `[HAIKU-4.5]` always.\n' >>"$TMP/mutant/.claude/agents/good.md"
expect_rc 1 "A3 MUTANT: a newly hard-coded [HAIKU-4.5] directive is caught" \
  python3 "$GATE" --repo "$TMP/mutant" --check-briefs

# --- A4: MUTATION — the allowlist must not cover a DIFFERENT violation ------
# Content-fingerprint keying means editing the exempted line lapses the
# exemption. A path-keyed allowlist would silently swallow this.
mkdir -p "$TMP/alw/.claude/agents" "$TMP/alw/scripts"
cat >"$TMP/alw/.claude/agents/x.md" <<'EOF'
- Tag your code with `[SONNET-4.6]` always.
EOF
FP="$(python3 "$GATE" --repo "$TMP/alw" --print-fingerprints | awk 'NR==1{print $1}')"
cat >"$TMP/alw/scripts/check-model-attribution.allowlist.json" <<EOF
{"entries":[{"path":".claude/agents/x.md","fingerprint":"$FP","reason":"fixture","tracking":"#0"}]}
EOF
expect_rc 0 "A4a a correctly-fingerprinted allowlist entry exempts its line" \
  python3 "$GATE" --repo "$TMP/alw" --check-briefs
printf -- '- Tag your code with `[OPUS-4.8]` always.\n' >>"$TMP/alw/.claude/agents/x.md"
expect_rc 1 "A4b MUTANT: allowlist does NOT cover a new violation in the same file" \
  python3 "$GATE" --repo "$TMP/alw" --check-briefs

# --- A5: the allowlist ratchets — a stale entry fails -----------------------
mkdir -p "$TMP/stale/.claude/agents" "$TMP/stale/scripts"
printf -- '- Markers follow the RUNNING model.\n' >"$TMP/stale/.claude/agents/y.md"
cat >"$TMP/stale/scripts/check-model-attribution.allowlist.json" <<'EOF'
{"entries":[{"path":".claude/agents/y.md","fingerprint":"deadbeef","reason":"fixture","tracking":"#0"}]}
EOF
expect_rc 1 "A5 a STALE allowlist entry (defect already fixed) fails the gate" \
  python3 "$GATE" --repo "$TMP/stale" --check-briefs

# --- A6: an allowlist entry without a reason/tracking is rejected -----------
mkdir -p "$TMP/noreason/.claude/agents" "$TMP/noreason/scripts"
printf -- '- Tag your code with `[SONNET-4.6]` always.\n' >"$TMP/noreason/.claude/agents/z.md"
FP2="$(python3 "$GATE" --repo "$TMP/noreason" --print-fingerprints | awk 'NR==1{print $1}')"
cat >"$TMP/noreason/scripts/check-model-attribution.allowlist.json" <<EOF
{"entries":[{"path":".claude/agents/z.md","fingerprint":"$FP2","reason":"","tracking":""}]}
EOF
expect_rc 1 "A6 an allowlist entry with no reason/tracking is REJECTED" \
  python3 "$GATE" --repo "$TMP/noreason" --check-briefs

# --- A7: a corrupt allowlist must fail CLOSED, never mute the gate ----------
mkdir -p "$TMP/corrupt/.claude/agents" "$TMP/corrupt/scripts"
printf -- '- Tag your code with `[SONNET-4.6]` always.\n' >"$TMP/corrupt/.claude/agents/z.md"
printf 'not json {' >"$TMP/corrupt/scripts/check-model-attribution.allowlist.json"
expect_rc 1 "A7 a corrupt allowlist fails CLOSED (does not silently mute)" \
  python3 "$GATE" --repo "$TMP/corrupt" --check-briefs

# --- A8: the commit gate — the actual reported symptom ----------------------
GITREPO="$TMP/gitrepo"
git init -q -b main "$GITREPO"
git -C "$GITREPO" config user.email t@example.com
git -C "$GITREPO" config user.name t
echo seed >"$GITREPO/seed.txt"
git -C "$GITREPO" add seed.txt
git -C "$GITREPO" commit -qm seed
BASE="$(git -C "$GITREPO" rev-parse HEAD)"

printf '// [OPUS-5] honest\n' >"$GITREPO/a.rs"
git -C "$GITREPO" add a.rs
git -C "$GITREPO" commit -qm "feat: a

Co-Authored-By: Claude Opus 5 <n@a.com>"
expect_rc 0 "A8a an HONEST commit (marker == trailer) passes" \
  python3 "$GATE" --repo "$GITREPO" --check-commits "$BASE..HEAD"
HONEST="$(git -C "$GITREPO" rev-parse HEAD)"

# The exact PR #4361 shape: an Opus 5 commit stamping [SONNET-4.6].
printf '// [SONNET-4.6] mis-attributed\n' >"$GITREPO/b.rs"
git -C "$GITREPO" add b.rs
git -C "$GITREPO" commit -qm "feat: b

Co-Authored-By: Claude Opus 5 <n@a.com>"
expect_rc 1 "A8b MUTANT: an Opus-5 commit stamping [SONNET-4.6] is caught (the PR #4361 shape)" \
  python3 "$GATE" --repo "$GITREPO" --check-commits "$HONEST..HEAD"
LYING="$(git -C "$GITREPO" rev-parse HEAD)"

# Symmetry: the trap must not be Sonnet-specific, or the next deprecation
# re-runs this incident.
printf '// [OPUS-4.8] mis-attributed the other way\n' >"$GITREPO/c.rs"
git -C "$GITREPO" add c.rs
git -C "$GITREPO" commit -qm "feat: c

Co-Authored-By: Claude Sonnet 4.6 <n@a.com>"
expect_rc 1 "A8c MUTANT: the check is model-SYMMETRIC (Sonnet commit stamping [OPUS-4.8] also caught)" \
  python3 "$GATE" --repo "$GITREPO" --check-commits "$LYING..HEAD"
SYM="$(git -C "$GITREPO" rev-parse HEAD)"

printf '// [OPUS-4.8] undeclared\n' >"$GITREPO/d.rs"
git -C "$GITREPO" add d.rs
git -C "$GITREPO" commit -qm "feat: d"
expect_rc 1 "A8d a commit that stamps a marker but declares NO model fails CLOSED" \
  python3 "$GATE" --repo "$GITREPO" --check-commits "$SYM..HEAD"
UNDECL="$(git -C "$GITREPO" rev-parse HEAD)"

# Cleanup of stale markers must always remain possible.
printf '// cleaned\n' >"$GITREPO/b.rs"
git -C "$GITREPO" add b.rs
git -C "$GITREPO" commit -qm "chore: drop stale marker

Co-Authored-By: Claude Opus 5 <n@a.com>"
expect_rc 0 "A8e REMOVING a stale marker is never flagged (cleanup stays possible)" \
  python3 "$GATE" --repo "$GITREPO" --check-commits "$UNDECL..HEAD"

echo
echo "PART B — the YAML seam (measured: this is where vacuity lives)"

if [ ! -f "$WORKFLOW" ]; then
  fail "B0 $WORKFLOW not found"
else
  # B1 — the call sites must EXIST. Deleting either step reds this test.
  if grep -q -- 'check-model-attribution\.py --self-test' "$WORKFLOW"; then
    pass "B1a docs-quality.yml invokes the gate's --self-test"
  else
    fail "B1a docs-quality.yml does NOT invoke check-model-attribution.py --self-test"
  fi
  if grep -q -- 'check-model-attribution\.py --check-briefs' "$WORKFLOW"; then
    pass "B1b docs-quality.yml invokes the ENFORCING --check-briefs"
  else
    fail "B1b docs-quality.yml does NOT invoke check-model-attribution.py --check-briefs"
  fi
  # B1c — resolved at COMMAND POSITION, not by filename. A filename grep is
  # satisfied by the adjacent `shellcheck scripts/tests/test_model_attribution.sh`,
  # so deleting the ONE line that actually RUNS this suite left the grep green,
  # the seam checker at "4 invocation(s) … OK", and this file's own output
  # byte-identical at 42 ok / 0 FAIL — while none of it ran in CI.
  #
  # Read the honest limit of this assertion: it lives INSIDE the suite it is
  # about, so if the suite is unwired this line does not run either. The check
  # that actually protects the repo is the SEPARATE gating seam step, asserted
  # by B1f below; this one is the local/dev signal.
  expect_rc 0 "B1c docs-quality.yml EXECUTES this test file (command position, not a mention)" \
    python3 "$SEAM" --workflow "$WORKFLOW" --needle check-model-attribution.py \
      --exclude check-workflow-seam.py --min-invocations 1 \
      --require-exec scripts/tests/test_model_attribution.sh
  # B1d — the PR-level backstop must be present AND must declare itself advisory
  # through the script's own --advisory mode. If someone "promotes" it by deleting
  # --advisory, that is fine (it becomes hard). What must never happen is the
  # invocation disappearing, or being neutered with `|| true` instead. This
  # catches the former; B2/B3a-b catch the latter — and B3a is specifically the
  # backslash-continuation spelling that the earlier version of B2 did NOT catch,
  # so "B2 catches `|| true`" is now a tested claim rather than an assumed one.
  if grep -q -- 'check-model-attribution\.py --advisory' "$WORKFLOW" \
     || grep -qE -- 'check-model-attribution\.py.*--check-commits' "$WORKFLOW"; then
    pass "B1d docs-quality.yml runs the PR-level --check-commits backstop"
  else
    fail "B1d the PR-level --check-commits backstop is not wired at all"
  fi

  # B1e — the seam checker itself must be wired, or B2/B3 below are checking a
  # workflow nobody runs the checker against in CI. The ARGUMENTS are asserted
  # too, because a wired-but-mis-argued invocation is the same vacuity one level
  # up: without --exclude, this step's own `--needle check-model-attribution.py`
  # counts as an extra gate invocation and the --min-invocations floor stops
  # detecting a deleted call site. That happened — the flag was present in the
  # hand-run command and missing in the YAML, and only the real CI run's
  # "5 invocation(s)" showed it.
  seam_call="$(sed -n '/check-workflow-seam\.py --workflow/,/^      - /p' "$WORKFLOW" | tr '\n' ' ')"
  if [ -n "$seam_call" ]; then
    pass "B1e docs-quality.yml runs check-workflow-seam.py against itself"
  else
    fail "B1e the seam checker is not wired into docs-quality.yml"
  fi
  if [ -n "$seam_call" ] \
     && [[ "$seam_call" == *"--exclude check-workflow-seam.py"* ]] \
     && [[ "$seam_call" == *"--min-invocations 4"* ]] \
     && [[ "$seam_call" == *"--require-exec scripts/tests/test_model_attribution.sh"* ]]; then
    pass "B1f ...with --exclude, the 4-call floor, and --require-exec on THIS suite"
  else
    fail "B1f the seam invocation is missing --exclude / --min-invocations 4 / --require-exec"
  fi

  # B2 — the invocations must not be neutered. A step that is conditionally
  # skipped, marked continue-on-error, `|| true`-ed, or run under a relaxed
  # shell is a vacuous gate: it reports green while checking nothing. Parsed
  # structurally (an `if:` can sit on the step OR its job) and over LOGICAL
  # shell lines (a `|| true` can sit after a backslash continuation).
  # The message names the forms actually covered. The previous wording — "gate
  # steps are unconditional and do not swallow failure" — was printed verbatim
  # over a hard gate whose exit status WAS swallowed, which is how a reassuring
  # summary line becomes the thing that hides the defect.
  expect_rc 0 "B2 no gate step is skipped, continue-on-error'd, \`||\`-swallowed (incl. across a continuation), or run under a relaxed shell" \
    python3 "$SEAM" --workflow "$WORKFLOW" "${NEEDLES[@]}"

  # B3 — MUTATE THE SEAM ITSELF. B2 passing is worth nothing unless B2 would
  # have failed on each way the wiring can rot. Every mutant is applied to a
  # COPY of the real workflow, so these test the shipped file's actual shape.
  #
  # B3a is the regression that motivated extracting the checker: the earlier
  # line-by-line version passed this mutant while the hard --check-briefs gate
  # ran with its exit status thrown away.
  MUT="$TMP/mut.yml"
  ENFORCE='        run: python3 scripts/check-model-attribution.py --check-briefs'

  mutate() {  # mutate <replacement-python-literal>
    python3 - "$WORKFLOW" "$MUT" "$ENFORCE" "$1" <<'PY'
import sys
src, dst, anchor, repl = sys.argv[1:5]
text = open(src, encoding="utf-8").read()
if anchor + "\n" not in text:
    sys.exit("anchor line not found in workflow — update the test")
open(dst, "w", encoding="utf-8").write(text.replace(anchor + "\n", repl, 1))
PY
  }

  mutate '        run: |
          python3 scripts/check-model-attribution.py --check-briefs \
            || true
'
  expect_rc 1 "B3a MUTANT: \`|| true\` on a BACKSLASH-CONTINUATION line is caught" \
    python3 "$SEAM" --workflow "$MUT" "${NEEDLES[@]}"

  mutate '        run: python3 scripts/check-model-attribution.py --check-briefs || true
'
  expect_rc 1 "B3b MUTANT: \`|| true\` on the same line is caught" \
    python3 "$SEAM" --workflow "$MUT" "${NEEDLES[@]}"

  mutate '        if: github.event_name == '"'"'never'"'"'
        run: python3 scripts/check-model-attribution.py --check-briefs
'
  expect_rc 1 "B3c MUTANT: a never-true step \`if:\` is caught" \
    python3 "$SEAM" --workflow "$MUT" "${NEEDLES[@]}"

  mutate '        continue-on-error: true
        run: python3 scripts/check-model-attribution.py --check-briefs
'
  expect_rc 1 "B3d MUTANT: continue-on-error is caught" \
    python3 "$SEAM" --workflow "$MUT" "${NEEDLES[@]}"

  mutate '        run: |
          set +e
          python3 scripts/check-model-attribution.py --check-briefs
          echo done
'
  expect_rc 1 "B3e MUTANT: \`set +e\` in the run body is caught" \
    python3 "$SEAM" --workflow "$MUT" "${NEEDLES[@]}"

  mutate ''
  expect_rc 1 "B3f MUTANT: DELETING the enforcing step is caught" \
    python3 "$SEAM" --workflow "$MUT" "${NEEDLES[@]}"

  # B3h — THE ONE-LINE DELETION. Remove only the line that RUNS this suite,
  # leaving the adjacent `shellcheck` mention. Every filename-based check stays
  # green and this file's own output stays byte-identical (42 ok / 0 FAIL), so
  # nothing but a command-position resolution can see it.
  python3 - "$WORKFLOW" "$MUT" <<'PY'
import sys
src, dst = sys.argv[1:3]
text = open(src, encoding="utf-8").read()
line = "          bash scripts/tests/test_model_attribution.sh\n"
if line not in text:
    sys.exit("the suite's execution line is not where the test expects it")
open(dst, "w", encoding="utf-8").write(text.replace(line, "", 1))
PY
  expect_rc 1 "B3h MUTANT: deleting the ONE line that RUNS this suite is caught" \
    python3 "$SEAM" --workflow "$MUT" "${NEEDLES[@]}"
  # ...and it is caught for the right reason, not by the invocation floor.
  # (Captured to a file, not piped: `set -o pipefail` above would otherwise let
  # the checker's intended rc=1 make the `if` condition false regardless of the
  # grep — a test that fails for its own plumbing proves nothing.)
  python3 "$SEAM" --workflow "$MUT" "${NEEDLES[@]}" >"$TMP/b3i.txt" 2>&1 || true
  if grep -q 'never EXECUTED at command position' "$TMP/b3i.txt"; then
    pass "B3i ...and the reason given is the orphaned execution, not the call-site count"
  else
    fail "B3i B3h red for the wrong reason (want 'never EXECUTED at command position')"
  fi

  # B3g — and the checker must not be a rubber stamp in the other direction:
  # the unmutated copy passes, so B3a-f fail for their mutation and not because
  # the copy itself is broken.
  cp "$WORKFLOW" "$MUT"
  expect_rc 0 "B3g the UNMUTATED copy passes (B3a-f red on the mutation, not the copy)" \
    python3 "$SEAM" --workflow "$MUT" "${NEEDLES[@]}"

  # B4 — the seam checker's own hermetic mutants.
  expect_rc 0 "B4 check-workflow-seam.py --self-test passes (9 mutants killed)" \
    python3 "$SEAM" --self-test
fi

echo
echo "PART C — the REAL CI shape (actions/checkout's shallow merge-ref graft)"
#
# This whole part exists because the backstop was INERT in CI while passing
# every test on a normal clone. `actions/checkout` at fetch-depth 1 checks out
# `refs/pull/N/merge` and GRAFTS it: the commit's parents are recorded in the
# object but invisible to every revision walk. `git show` then renders the
# entire repository as additions of that one commit — 15,123 findings on PR
# #4385's own head (docs-quality run 89851383239), every one attributed to
# `refs/pull/4385/merge`, including files the PR never touched. A single real
# misattribution was indistinguishable from that noise.
#
# A test on a normal clone CANNOT see this. So the fixture below is the CI shape
# itself, built from git plumbing: a merge ref fetched at depth 1 into a fresh
# repository, exactly as the runner does it.

CI="$TMP/ci"
mkdir -p "$CI"
UPSTREAM="$CI/upstream"
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@e.com
export GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@e.com
git init -q -b main "$UPSTREAM"
# Baseline history carrying PRE-EXISTING markers in files the PR never touches.
# These are the 15,123: if any of them is reported, the range is wrong.
for i in 1 2 3 4 5; do
  mkdir -p "$UPSTREAM/dir$i"
  printf 'legacy [SONNET-4.6] stamp\n' >"$UPSTREAM/dir$i/legacy$i.md"
  git -C "$UPSTREAM" add -A
  git -C "$UPSTREAM" commit -qm "main $i"
done
git -C "$UPSTREAM" checkout -qb pr main
# The PR's own single commit: an Opus 5 commit stamping [SONNET-4.6] — the
# #4361 defect shape. Exactly ONE finding is the correct answer.
printf '// [SONNET-4.6] the ONE real finding\n' >"$UPSTREAM/newfile.rs"
git -C "$UPSTREAM" add -A
git -C "$UPSTREAM" commit -qm "feat: pr

Co-Authored-By: Claude Opus 5 <n@a.com>"
PR_HEAD="$(git -C "$UPSTREAM" rev-parse pr)"
PR_BASE="$(git -C "$UPSTREAM" rev-parse pr~1)"
# main advances after the PR branched, as it always does in this repo.
git -C "$UPSTREAM" checkout -q main
printf 'later\n' >"$UPSTREAM/dir1/later.txt"
git -C "$UPSTREAM" add -A
git -C "$UPSTREAM" commit -qm "main moves on"
# GitHub's refs/pull/N/merge: main merged with the PR head.
git -C "$UPSTREAM" checkout -q --detach main
git -C "$UPSTREAM" merge -q --no-ff -m "Merge pull request #1" pr
git -C "$UPSTREAM" update-ref refs/pull/1/merge "$(git -C "$UPSTREAM" rev-parse HEAD)"
git -C "$UPSTREAM" update-ref refs/pull/1/head "$PR_HEAD"
git -C "$UPSTREAM" checkout -q main
MERGE_SHA="$(git -C "$UPSTREAM" rev-parse refs/pull/1/merge)"

# --- the runner: actions/checkout@v7 defaults --------------------------------
WS="$CI/ws"
git init -q -b main "$WS"
git -C "$WS" remote add origin "$UPSTREAM"
git -C "$WS" fetch -q --no-tags --depth=1 origin "+$MERGE_SHA:refs/remotes/pull/1/merge"
git -C "$WS" checkout -q --force -B main refs/remotes/pull/1/merge
# ...and the step's own `git fetch --depth=200 origin main`, which is a no-op
# for the graft because the merge ref is unreachable from main.
git -C "$WS" fetch -q --depth=200 origin main || true

if [ "$(git -C "$WS" rev-parse --is-shallow-repository)" = "true" ] \
   && [ -z "$(git -C "$WS" log -1 --format=%P HEAD)" ]; then
  pass "C0 fixture really is the CI shape (shallow, HEAD has no walkable parents)"
else
  fail "C0 fixture is NOT grafted — every assertion below would be vacuous"
fi

# C1 — the OLD invocation. It must no longer pretend to have checked anything.
OLD_OUT="$TMP/old.txt"
old_rc=0
python3 "$GATE" --repo "$WS" --advisory --check-commits "origin/main..HEAD" >"$OLD_OUT" 2>&1 || old_rc=$?
if grep -q 'INCOMPLETE' "$OLD_OUT" && ! grep -q -- '— OK' "$OLD_OUT"; then
  pass "C1 a grafted range reports INCOMPLETE and never claims OK"
else
  fail "C1 a grafted range did not report INCOMPLETE (or still claimed OK)"
fi
if [ "$(grep -c 'legacy' "$OLD_OUT" || true)" = "0" ]; then
  pass "C1b the whole-tree blowout is gone (zero findings in untouched files)"
else
  fail "C1b untouched files are STILL reported: $(grep -c 'legacy' "$OLD_OUT") line(s)"
fi
if [ "$old_rc" = "0" ]; then
  pass "C1c INCOMPLETE is reported WITHOUT reddening the advisory build"
else
  fail "C1c the advisory step exited $old_rc on an INCOMPLETE range"
fi

# C2 — the NEW invocation, wired exactly as docs-quality.yml wires it: fetch the
# PR's own head ref to depth commits+1, then derive the range from the event.
git -C "$WS" fetch -q --no-tags --depth=2 origin "+refs/pull/1/head:refs/remotes/origin/pr/1/head"
EVENT="$TMP/event.json"
cat >"$EVENT" <<JSON
{"pull_request":{"number":1,"commits":1,
 "head":{"sha":"$PR_HEAD"},"base":{"sha":"$PR_BASE"}}}
JSON
NEW_OUT="$TMP/new.txt"
new_rc=0
GITHUB_EVENT_NAME=pull_request GITHUB_EVENT_PATH="$EVENT" \
  python3 "$GATE" --repo "$WS" --advisory --check-commits-pr >"$NEW_OUT" 2>&1 || new_rc=$?

if [ "$(grep -c 'adds marker' "$NEW_OUT" || true)" = "1" ]; then
  pass "C2 the range is EXACTLY the PR's own commits (1 finding, not the tree)"
else
  fail "C2 expected exactly 1 finding, got $(grep -c 'adds marker' "$NEW_OUT" || true)"
fi
if grep -q 'newfile.rs' "$NEW_OUT" && ! grep -q 'legacy' "$NEW_OUT"; then
  pass "C2b the finding is the PR's own line, and no untouched file is named"
else
  fail "C2b wrong finding set (want newfile.rs only)"
fi
if ! grep -q 'INCOMPLETE' "$NEW_OUT"; then
  pass "C2c nothing was left unscannable once the PR head ref is fetched"
else
  fail "C2c still reporting unscannable commits after the correct fetch"
fi
if [ "$new_rc" = "0" ]; then
  pass "C2d the ADVISORY step still exits 0 (findings reported, build not red)"
else
  fail "C2d the advisory step exited $new_rc — it must not red the build"
fi
expect_rc 1 "C2e WITHOUT --advisory the same findings DO fail (promotable to HARD)" \
  env GITHUB_EVENT_NAME=pull_request GITHUB_EVENT_PATH="$EVENT" \
  python3 "$GATE" --repo "$WS" --check-commits-pr

# C3 — INVERSION. Drop the PR-head fetch and the gate must say it could not
# look, never that it looked and found nothing. This is the property that
# stops the backstop silently going inert again.
WS2="$CI/ws2"
git init -q -b main "$WS2"
git -C "$WS2" remote add origin "$UPSTREAM"
git -C "$WS2" fetch -q --no-tags --depth=1 origin "+$MERGE_SHA:refs/remotes/pull/1/merge"
git -C "$WS2" checkout -q --force -B main refs/remotes/pull/1/merge
NOFETCH_OUT="$TMP/nofetch.txt"
nofetch_rc=0
GITHUB_EVENT_NAME=pull_request GITHUB_EVENT_PATH="$EVENT" \
  python3 "$GATE" --repo "$WS2" --advisory --check-commits-pr >"$NOFETCH_OUT" 2>&1 || nofetch_rc=$?
if grep -q 'INCOMPLETE' "$NOFETCH_OUT" && ! grep -q -- '— OK' "$NOFETCH_OUT"; then
  pass "C3 a missing PR-head fetch reports INCOMPLETE, never a false OK"
else
  fail "C3 a missing PR-head fetch was reported as a clean pass"
fi
if [ "$nofetch_rc" = "0" ]; then
  pass "C3b ...and still does not red the advisory build"
else
  fail "C3b the advisory step exited $nofetch_rc with no PR-head fetch"
fi

# C4 — off a pull_request event (merge_group / push) the mode is a declared
# no-op, and says so rather than inventing a range.
C4_OUT="$TMP/c4.txt"
GITHUB_EVENT_NAME=merge_group GITHUB_EVENT_PATH="$EVENT" \
  python3 "$GATE" --repo "$WS" --advisory --check-commits-pr >"$C4_OUT" 2>&1
if grep -q 'not a pull_request event' "$C4_OUT" && ! grep -q -- '— OK' "$C4_OUT"; then
  pass "C4 on merge_group/push the mode declares itself inapplicable"
else
  fail "C4 the mode invented a result off a pull_request event"
fi

echo
if [ "$FAILURES" -ne 0 ]; then
  echo "test_model_attribution.sh: $FAILURES FAILURE(S)"
  exit 1
fi
echo "test_model_attribution.sh: all checks passed"
