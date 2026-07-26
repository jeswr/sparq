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
#            that its exit status is not swallowed by `|| true` /
#            `continue-on-error`. Deleting the call site must fail THIS test,
#            not merely go unnoticed.
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
WORKFLOW="$REPO_ROOT/.github/workflows/docs-quality.yml"
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
  if grep -q -- 'test_model_attribution\.sh' "$WORKFLOW"; then
    pass "B1c docs-quality.yml runs THIS test file"
  else
    fail "B1c docs-quality.yml does not run scripts/tests/test_model_attribution.sh"
  fi
  # B1d — the PR-level backstop must be present AND must declare itself advisory
  # through the script's own --advisory mode. If someone "promotes" it by deleting
  # --advisory, that is fine (it becomes hard). What must never happen is the
  # invocation disappearing, or being neutered with `|| true` instead — B2 catches
  # the latter, this catches the former.
  if grep -q -- 'check-model-attribution\.py --advisory' "$WORKFLOW" \
     || grep -qE -- 'check-model-attribution\.py.*--check-commits' "$WORKFLOW"; then
    pass "B1d docs-quality.yml runs the PR-level --check-commits backstop"
  else
    fail "B1d the PR-level --check-commits backstop is not wired at all"
  fi

  # B2 — the invocations must not be neutered. A step that is conditionally
  # skipped, marked continue-on-error, or `|| true`-ed is a vacuous gate: it
  # reports green while checking nothing. This parses the workflow structurally
  # rather than grepping, because an `if:` can sit on the step OR its job.
  seam_rc=0
  python3 - "$WORKFLOW" <<'PY' || seam_rc=$?
import sys, yaml
wf = yaml.safe_load(open(sys.argv[1]))
needles = ("check-model-attribution.py", "test_model_attribution.sh")
found = 0
bad = []
for job_name, job in (wf.get("jobs") or {}).items():
    job_if = job.get("if")
    for step in job.get("steps") or []:
        run = step.get("run") or ""
        if not any(n in run for n in needles):
            continue
        found += 1
        where = f"{job_name}/{step.get('name', '?')}"
        if step.get("if") is not None:
            bad.append(f"{where}: step carries an `if:` ({step['if']!r}) so it can be skipped")
        if job_if is not None:
            bad.append(f"{where}: job carries an `if:` ({job_if!r}) so the whole gate can be skipped")
        if step.get("continue-on-error"):
            bad.append(f"{where}: continue-on-error swallows the failure")
        for line in run.splitlines():
            s = line.strip()
            if any(n in s for n in needles) and ("|| true" in s or s.endswith("|| :")):
                bad.append(f"{where}: `|| true` discards the gate's exit status")
if found < 4:
    bad.append(
        "expected 4 gate invocations (self-test, check-briefs, test script, "
        f"PR-level backstop), found {found}"
    )
for b in bad:
    print("  ", b, file=sys.stderr)
sys.exit(1 if bad else 0)
PY
  if [ "$seam_rc" -eq 0 ]; then
    pass "B2 gate steps are unconditional and do not swallow failure"
  else
    fail "B2 a gate step is skippable or swallows its exit status"
  fi
fi

echo
if [ "$FAILURES" -ne 0 ]; then
  echo "test_model_attribution.sh: $FAILURES FAILURE(S)"
  exit 1
fi
echo "test_model_attribution.sh: all checks passed"
