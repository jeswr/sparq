#!/usr/bin/env bash
# [OPUS-5] Hermetic self-tests for the sq-ffaa9 durable S3 results channel
# (scripts/bench/bench-s3-results.sh + scripts/bench/s3-results-uploader.sh).
#
# 🤖 SPARQ agent. HERMETIC: `aws` is PATH-shadowed by a stub that RECORDS every
# invocation and replies from a scripted table; `curl`, the network and the real AWS
# account are never touched, and nothing outside the mktemp sandbox is written.
#
# The invariants pinned here are the ones a future refactor could silently break, each
# of which would reintroduce the exact failure this bead exists to fix:
#   1. OFF BY DEFAULT — with SPARQ_BENCH_S3_BUCKET unset, `bench_s3_iam_args` and
#      `bench_s3_fetch` are no-ops and `bench_s3_userdata_line` renders `true`, so every
#      wired launcher behaves EXACTLY as it does today.
#   2. LAUNCH IS NEVER BLOCKED BY IAM — a missing/unreadable instance profile degrades
#      to "no S3 channel" with a warning, never to a failed run-instances.
#   3. DOLLAR-FREE USER-DATA LINE — the launchers render user-data through UNQUOTED
#      heredocs; a `$` in the emitted line would be expanded by the LAUNCHER's shell.
#   4. WRITE-ONLY GRANT ⇔ cp-ONLY UPLOADER — the instance policy grants PutObject and
#      NOT Get/List/Delete, and the uploader therefore uses `aws s3 cp`, never
#      `aws s3 sync` (which needs ListBucket+GetObject and would fail closed). Widening
#      the policy, or introducing a `sync`, must break this test.
#   5. PLAN IS OFFLINE — `plan` is the reviewable artifact for the maintainer
#      credential step, so it must make ZERO AWS calls.
#   6. PROVISION IS IDEMPOTENT — re-running against existing resources succeeds.
#   7. ATTACHMENT IS VERIFIED, NOT ASSUMED — the profile must carry EXACTLY the upload
#      role. add-role-to-instance-profile returns nonzero both benignly (already
#      attached) and fatally (profile holds another role, no grant, service failure);
#      reading every failure as benign is how a `provisioned`/`READY` verdict could be
#      printed over a profile whose credentials cannot PutObject — the silent
#      unretrievable run this whole module exists to prevent. So provision must FAIL,
#      check must withhold READY, and the launcher must omit the profile instead.
#   8. THE TRUST RELATIONSHIP IS CONVERGED AND CHECKED — the same failure one level down:
#      create-role is skipped for an EXISTING same-named role, so one carrying a foreign or
#      absent trust principal would be attached, pass every existence/membership check and
#      be reported READY while EC2 cannot assume it and the box gets NO credentials.
#      Provision must repair it (update-assume-role-policy) and check must withhold READY
#      until it is repaired.
#
# Run:  bash scripts/tests/test_bench_s3_results.sh   (exit 0 = all pass, 1 = a failure)
#
# SC1090: $LIB is computed from $ROOT — sourcing the script under test IS the test.
# shellcheck disable=SC1090
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LIB="$ROOT/scripts/bench/bench-s3-results.sh"
UPLOADER="$ROOT/scripts/bench/s3-results-uploader.sh"

[ -f "$LIB" ] || { echo "FATAL: $LIB not found"; exit 2; }
[ -f "$UPLOADER" ] || { echo "FATAL: $UPLOADER not found"; exit 2; }

pass=0; fail=0
ok()   { pass=$((pass + 1)); printf 'ok   %s\n' "$1"; }
bad()  { fail=$((fail + 1)); printf 'FAIL %s\n' "$1"; }
want() { if [ "$2" = "$3" ]; then ok "$1"; else bad "$1 (want [$2], got [$3])"; fi; }

SANDBOX="$(mktemp -d)"
trap 'rm -rf "$SANDBOX"' EXIT
BIN="$SANDBOX/bin"; mkdir -p "$BIN"
CALLS="$SANDBOX/aws-calls.log"

# --------------------------------------------------------------------------- #
# `aws` stub. Records every call; behaviour switched by AWS_STUB_MODE:
#   all-ok      every call succeeds (existence checks => resource already exists)
#   fresh       STATEFUL empty account: an existence check succeeds only if the matching
#               create-* was already recorded in this log. This is what makes the
#               first-run create path and the second-run idempotent path BOTH real —
#               under `all-ok` provision correctly skips every create, so asserting the
#               creates there would only pin the stub, not the script.
#   no-profile  get-instance-profile fails (absent), everything else succeeds
#   all-fail    every call fails
#
# ORTHOGONAL to the mode, AWS_STUB_PROFILE_ROLE decides which role an EXISTING instance
# profile reports (`iam get-instance-profile --query …Roles[].RoleName`):
#   unset            the role the script asks for, i.e. a correctly-attached profile
#   ""               a profile that exists carrying NO role
#   <another name>   a profile carrying SOMEONE ELSE'S role — and, as on real AWS where a
#                    profile holds at most one role, add-role-to-instance-profile then
#                    FAILS (LimitExceeded). This is the case that used to be silently
#                    swallowed as "already attached".
# Under `fresh` the profile carries the role only once add-role has actually been
# recorded, so the provision path's post-attach verification is real rather than stubbed.
#
# ALSO orthogonal, AWS_STUB_ROLE_TRUST decides the trust document an EXISTING role reports
# (`iam get-role --query Role.AssumeRolePolicyDocument`):
#   unset/good  a correct ec2.amazonaws.com + sts:AssumeRole trust
#   wrong       a same-named role trusting a DIFFERENT service — attachable, reports
#               healthy, and yet EC2 cannot assume it, so the box gets no credentials
#   none        a trust document with no statement at all
#   encoded     the good document still URL-encoded, as the IAM API returns it on the wire
# A `wrong`/`none` role becomes good once `iam update-assume-role-policy` has been recorded
# in the SAME call log — that is what makes provision's trust convergence observable rather
# than stubbed, and lets one log show check flipping to READY only AFTER the repair.
# --------------------------------------------------------------------------- #
cat > "$BIN/aws" <<'STUB'
#!/usr/bin/env bash
seen() { grep -q -- "$1" "$AWS_CALL_LOG" 2>/dev/null; }
printf '%s\n' "$*" >> "$AWS_CALL_LOG"
# The role list get-instance-profile reports for an existing profile.
stub_profile_roles() {
  case "${AWS_STUB_MODE:-all-ok}" in
    fresh) seen "iam add-role-to-instance-profile" && printf '%s\n' "${AWS_STUB_PROFILE_ROLE-sparq-bench-results}" ;;
    *)     printf '%s\n' "${AWS_STUB_PROFILE_ROLE-sparq-bench-results}" ;;
  esac
}
# The trust document get-role reports for an existing role. `provision` repairs it.
stub_role_trust() {
  local kind="${AWS_STUB_ROLE_TRUST:-good}"
  seen "iam update-assume-role-policy" && kind=good
  case "$kind" in
    wrong)   printf '%s\n' '{"Version":"2012-10-17","Statement":[{"Effect":"Allow","Principal":{"Service":"lambda.amazonaws.com"},"Action":"sts:AssumeRole"}]}' ;;
    none)    printf '%s\n' '{"Version":"2012-10-17","Statement":[]}' ;;
    encoded) printf '%s\n' '"%7B%22Version%22%3A%222012-10-17%22%2C%22Statement%22%3A%5B%7B%22Effect%22%3A%22Allow%22%2C%22Principal%22%3A%7B%22Service%22%3A%22ec2.amazonaws.com%22%7D%2C%22Action%22%3A%22sts%3AAssumeRole%22%7D%5D%7D"' ;;
    *)       printf '%s\n' '{"Version":"2012-10-17","Statement":[{"Effect":"Allow","Principal":{"Service":"ec2.amazonaws.com"},"Action":"sts:AssumeRole"}]}' ;;
  esac
}
case "${AWS_STUB_MODE:-all-ok}" in
  all-fail) exit 1 ;;
  no-profile)
    case "$1 $2" in "iam get-instance-profile") exit 1 ;; esac ;;
  fresh)
    case "$1 $2" in
      "s3api head-bucket")        seen "s3api create-bucket" || exit 1 ;;
      "iam get-role")             seen "iam create-role" || exit 1 ;;
      "iam get-instance-profile") seen "iam create-instance-profile" || exit 1 ;;
    esac ;;
esac
# A profile already holding a different role cannot take another one.
case "$1 $2" in
  "iam add-role-to-instance-profile")
    if [ -n "${AWS_STUB_PROFILE_ROLE:-}" ] && [ "${AWS_STUB_MODE:-all-ok}" != "fresh" ]; then exit 254; fi ;;
esac
case "$1 $2" in
  "sts get-caller-identity") echo "111122223333" ;;
  "iam get-instance-profile")
    case "$*" in *"InstanceProfile.Roles"*) stub_profile_roles ;; esac ;;
  "iam get-role")
    case "$*" in *"Role.AssumeRolePolicyDocument"*) stub_role_trust ;; esac ;;
esac
exit 0
STUB
chmod +x "$BIN/aws"
export AWS_CALL_LOG="$CALLS"
export PATH="$BIN:$PATH"

reset_calls() { : > "$CALLS"; }

# --------------------------------------------------------------------------- #
# 1. Pure-predicate self-test inside the script itself.
# --------------------------------------------------------------------------- #
if bash "$LIB" --self-test >/dev/null 2>&1; then ok "embedded --self-test passes"; else bad "embedded --self-test failed"; fi

# --------------------------------------------------------------------------- #
# 2. OFF BY DEFAULT (invariant 1).
# --------------------------------------------------------------------------- #
reset_calls
OUT=$(unset SPARQ_BENCH_S3_BUCKET; . "$LIB"; bench_s3_iam_args)
want "iam_args empty when channel disabled" "" "$OUT"
want "no AWS call when channel disabled" "0" "$(wc -l < "$CALLS" | tr -d ' ')"

OUT=$(unset SPARQ_BENCH_S3_BUCKET; . "$LIB"; bench_s3_userdata_line run1 /root/x)
want "userdata line is a no-op when disabled" "true" "$OUT"

DEST="$SANDBOX/fetch-disabled"
( unset SPARQ_BENCH_S3_BUCKET; . "$LIB"; bench_s3_fetch run1 "$DEST" ) >/dev/null 2>&1
if [ -d "$DEST" ]; then bad "fetch created a dest dir while disabled"; else ok "fetch is a no-op when disabled"; fi

# --------------------------------------------------------------------------- #
# 3. Enabled path: profile present => the launch arg is emitted (invariant 2).
# --------------------------------------------------------------------------- #
reset_calls
OUT=$(AWS_STUB_MODE=all-ok SPARQ_BENCH_S3_BUCKET=bkt SPARQ_BENCH_INSTANCE_PROFILE=prof bash -c ". '$LIB'; bench_s3_iam_args")
want "iam_args emitted when the profile exists" "--iam-instance-profile Name=prof" "$OUT"

# ... and a MISSING profile degrades instead of failing the launch.
reset_calls
OUT=$(AWS_STUB_MODE=no-profile SPARQ_BENCH_S3_BUCKET=bkt SPARQ_BENCH_INSTANCE_PROFILE=prof bash -c ". '$LIB'; bench_s3_iam_args" 2>/dev/null)
RC=$?
want "missing profile => empty iam_args (launch not blocked)" "" "$OUT"
want "missing profile => rc 0 (launch not blocked)" "0" "$RC"

# ... and so does a profile that EXISTS but does not carry the upload role. Attaching it
# would look provisioned while the box's credentials could not PutObject.
reset_calls
OUT=$(AWS_STUB_MODE=all-ok AWS_STUB_PROFILE_ROLE=someone-elses-role \
  SPARQ_BENCH_S3_BUCKET=bkt SPARQ_BENCH_INSTANCE_PROFILE=prof bash -c ". '$LIB'; bench_s3_iam_args" 2>/dev/null)
RC=$?
want "profile carrying the WRONG role => empty iam_args" "" "$OUT"
want "profile carrying the WRONG role => rc 0 (launch not blocked)" "0" "$RC"
reset_calls
OUT=$(AWS_STUB_MODE=all-ok AWS_STUB_PROFILE_ROLE= \
  SPARQ_BENCH_S3_BUCKET=bkt SPARQ_BENCH_INSTANCE_PROFILE=prof bash -c ". '$LIB'; bench_s3_iam_args" 2>/dev/null)
want "profile carrying NO role => empty iam_args" "" "$OUT"

# Membership is an exact whole-name match, never a prefix.
reset_calls
OUT=$(AWS_STUB_MODE=all-ok AWS_STUB_PROFILE_ROLE=sparq-bench-results-readonly \
  SPARQ_BENCH_S3_BUCKET=bkt SPARQ_BENCH_INSTANCE_PROFILE=prof bash -c ". '$LIB'; bench_s3_iam_args" 2>/dev/null)
want "role match is exact, not a prefix" "" "$OUT"

# --------------------------------------------------------------------------- #
# 4. DOLLAR-FREE user-data line (invariant 3).
# --------------------------------------------------------------------------- #
LINE=$(SPARQ_BENCH_S3_BUCKET=bkt bash -c ". '$LIB'; bench_s3_userdata_line myrun /root/axis-results")
case "$LINE" in
  *'$'*) bad "userdata line contains \$ — an unquoted heredoc would expand it launcher-side" ;;
  *)     ok "userdata line is dollar-free (unquoted-heredoc safe)" ;;
esac
case "$LINE" in
  *"S3_URI=s3://bkt/myrun"*) ok "userdata line carries the pre-expanded S3 URI" ;;
  *) bad "userdata line is missing the S3 URI (got: $LINE)" ;;
esac
case "$LINE" in
  *"scripts/bench/s3-results-uploader.sh"*) ok "userdata line invokes the committed uploader" ;;
  *) bad "userdata line does not invoke the committed uploader" ;;
esac
# It must stay tiny — the launchers enforce a hard 16384B user-data limit.
if [ "${#LINE}" -le 400 ]; then ok "userdata line is compact (${#LINE}B <= 400B)"; else bad "userdata line is ${#LINE}B — too much of the 16384B user-data budget"; fi

# --------------------------------------------------------------------------- #
# 5. WRITE-ONLY grant <-> cp-only uploader (invariant 4).
# --------------------------------------------------------------------------- #
POLICY=$(SPARQ_BENCH_S3_BUCKET=bkt bash -c ". '$LIB'; bench_s3_write_policy bkt")
case "$POLICY" in *'"s3:PutObject"'*) ok "instance policy grants PutObject" ;; *) bad "instance policy lacks PutObject" ;; esac
case "$POLICY" in
  *GetObject*|*ListBucket*|*DeleteObject*|*'"s3:*"'*) bad "instance policy is WIDER than write-only" ;;
  *) ok "instance policy is write-only (no Get/List/Delete/wildcard)" ;;
esac
case "$POLICY" in *'"arn:aws:s3:::bkt/*"'*) ok "instance policy is scoped to the one bucket's objects" ;; *) bad "instance policy is not object-scoped to the bucket" ;; esac

# Comment lines are stripped first — the uploader's header DOCUMENTS why `sync` is
# banned, and matching that prose would make this check fire on its own explanation.
UPLOADER_CODE=$(grep -vE '^[[:space:]]*#' "$UPLOADER")
if printf '%s' "$UPLOADER_CODE" | grep -qE '\baws s3 sync\b'; then
  bad "uploader uses 'aws s3 sync' — needs ListBucket+GetObject, which the write-only role denies"
else
  ok "uploader never uses 'aws s3 sync'"
fi
if printf '%s' "$UPLOADER_CODE" | grep -qE '\baws s3 cp\b'; then ok "uploader uploads with 'aws s3 cp' (PutObject only)"; else bad "uploader has no 'aws s3 cp' upload"; fi

# --------------------------------------------------------------------------- #
# 6. PLAN IS OFFLINE (invariant 5).
# --------------------------------------------------------------------------- #
reset_calls
PLAN=$(SPARQ_BENCH_S3_BUCKET=bkt bash "$LIB" plan 2>&1); RC=$?
want "plan exits 0" "0" "$RC"
want "plan makes ZERO aws calls" "0" "$(wc -l < "$CALLS" | tr -d ' ')"
case "$PLAN" in *iam:PassRole*) ok "plan documents the launcher-side iam:PassRole grant" ;; *) bad "plan omits the iam:PassRole grant the launcher needs" ;; esac

# --------------------------------------------------------------------------- #
# 7. PROVISION: idempotent, and passes the write-only policy (invariant 6).
# --------------------------------------------------------------------------- #
# 7a. First run against an EMPTY account creates the whole object graph.
reset_calls
AWS_STUB_MODE=fresh SPARQ_BENCH_S3_BUCKET=bkt bash "$LIB" provision >/dev/null 2>&1
want "provision succeeds on an empty account" "0" "$?"
for expect in "s3api create-bucket" "s3api put-public-access-block" "s3api put-bucket-lifecycle-configuration" \
              "iam create-role" "iam put-role-policy" "iam create-instance-profile" "iam add-role-to-instance-profile"; do
  if grep -q -- "$expect" "$CALLS"; then ok "provision calls: $expect"; else bad "provision never called: $expect"; fi
done
if grep -q 'put-role-policy.*s3:PutObject' "$CALLS"; then ok "provision attaches the PutObject policy"; else bad "provision did not attach a PutObject policy"; fi
if grep -qE 'put-role-policy.*(GetObject|ListBucket|DeleteObject)' "$CALLS"; then bad "provision attached a wider-than-write-only policy"; else ok "provision attaches nothing wider than write-only"; fi
if grep -q 'create-bucket.*LocationConstraint' "$CALLS"; then ok "provision sets LocationConstraint outside us-east-1"; else bad "provision omitted LocationConstraint (create-bucket fails outside us-east-1)"; fi
if grep -q 'put-public-access-block.*BlockPublicAcls=true' "$CALLS"; then ok "provision blocks public access on the bucket"; else bad "provision left the bucket without a public-access block"; fi

# 7b. Re-running over the SAME (now-populated) account still succeeds and creates nothing twice.
CREATES_BEFORE=$(grep -c -- 'create-' "$CALLS")
AWS_STUB_MODE=fresh SPARQ_BENCH_S3_BUCKET=bkt bash "$LIB" provision >/dev/null 2>&1
want "provision is idempotent (2nd run succeeds)" "0" "$?"
want "2nd provision issues no new create-*" "$CREATES_BEFORE" "$(grep -c -- 'create-' "$CALLS")"

# 7c. FAILING to attach the role must NOT be reported as success. add-role-to-instance-profile
# returns nonzero both when the role is already attached (benign) and when the profile holds
# a different role / the caller lacks the grant (fatal); provision must tell them apart by
# verifying membership, not assume the benign reading and print "provisioned".
#
# The `timeout` is load-bearing, not defensive plumbing. Swallow the attach failure again
# and provision still eventually errors — but only after burning the 30x2s IAM
# eventual-consistency poll waiting for a role that will never appear, so a plain rc check
# would stay green over the reintroduced bug. A KNOWN-failed attach must be diagnosed from
# the failure itself, in well under a second; rc 124 here means that regressed.
reset_calls
timeout 15 env AWS_STUB_MODE=all-ok AWS_STUB_PROFILE_ROLE=someone-elses-role SPARQ_BENCH_S3_BUCKET=bkt \
  bash "$LIB" provision >"$SANDBOX/prov-wrong.out" 2>&1
want "provision FAILS promptly when the role cannot be attached (124 = fell into the poll)" "1" "$?"
# Anchored on the SUCCESS banner, not the bare word: the failure path legitimately says
# "NOT provisioned", and a substring match would score that honest message as a failure.
if grep -qF '[bench-s3] provisioned.' "$SANDBOX/prov-wrong.out"; then
  bad "provision printed its success banner over a profile that lacks the upload role"
else
  ok "provision does not claim success when the role is unattached"
fi
if grep -q 'ERROR' "$SANDBOX/prov-wrong.out"; then ok "provision reports the attach failure loudly"; else bad "provision failed silently"; fi

# 7d. An EXISTING same-named role whose trust document does not admit EC2 must be REPAIRED,
# not inherited. create-role is skipped for a role that already exists, so without an
# explicit update-assume-role-policy such a role attaches cleanly, passes every existence
# and membership check, is reported READY and is handed to every launcher — while EC2
# cannot assume it, the instance receives NO role credentials, and every upload fails.
reset_calls
AWS_STUB_MODE=all-ok AWS_STUB_ROLE_TRUST=wrong SPARQ_BENCH_S3_BUCKET=bkt bash "$LIB" provision >/dev/null 2>&1
want "provision succeeds over an existing role with a drifted trust policy" "0" "$?"
if grep -q -- 'iam update-assume-role-policy' "$CALLS"
then ok "provision converges the existing role's trust policy"
else bad "provision never called update-assume-role-policy — a drifted trust principal survives"; fi
if grep -q 'update-assume-role-policy.*ec2\.amazonaws\.com.*sts:AssumeRole' "$CALLS"
then ok "provision writes the EC2 sts:AssumeRole trust document"
else bad "provision's trust convergence did not pass the EC2 trust document"; fi
# Same call log, so the stub now reports the REPAIRED trust: check must flip to READY only
# after the repair, which is what makes the pre-repair failures below non-vacuous.
AWS_STUB_MODE=all-ok AWS_STUB_ROLE_TRUST=wrong SPARQ_BENCH_S3_BUCKET=bkt bash "$LIB" check >/dev/null 2>&1
want "check is READY once provision has repaired the trust policy" "0" "$?"

# --------------------------------------------------------------------------- #
# 8. check: reports NOT-ready when a resource is missing OR mis-wired.
# --------------------------------------------------------------------------- #
AWS_STUB_MODE=no-profile SPARQ_BENCH_S3_BUCKET=bkt bash "$LIB" check >/dev/null 2>&1
want "check fails when the instance profile is missing" "1" "$?"
AWS_STUB_MODE=all-ok SPARQ_BENCH_S3_BUCKET=bkt bash "$LIB" check >/dev/null 2>&1
want "check passes when everything exists" "0" "$?"
# READY must mean uploads can actually happen — a profile that exists but carries the
# wrong role (or none) is NOT ready, however healthy the bucket and role look.
AWS_STUB_MODE=all-ok AWS_STUB_PROFILE_ROLE=someone-elses-role SPARQ_BENCH_S3_BUCKET=bkt \
  bash "$LIB" check >"$SANDBOX/check-wrong.out" 2>&1
want "check is NOT ready when the profile carries the wrong role" "1" "$?"
if grep -q 'READY' "$SANDBOX/check-wrong.out"; then bad "check reported READY for a profile that cannot upload"; else ok "check withholds READY when the profile lacks the role"; fi
AWS_STUB_MODE=all-ok AWS_STUB_PROFILE_ROLE= SPARQ_BENCH_S3_BUCKET=bkt bash "$LIB" check >/dev/null 2>&1
want "check is NOT ready when the profile carries no role at all" "1" "$?"

# ... and READY must also mean EC2 can actually ASSUME the role. A role that exists but
# trusts another service (or nothing) is attachable and looks healthy, yet leaves the box
# with no credentials — get-role succeeding is not evidence the channel works.
# reset_calls first: a stale update-assume-role-policy from 7d would make the stub report
# the repaired document and score these vacuously.
for kind in wrong none; do
  reset_calls
  AWS_STUB_MODE=all-ok AWS_STUB_ROLE_TRUST=$kind SPARQ_BENCH_S3_BUCKET=bkt \
    bash "$LIB" check >"$SANDBOX/check-trust.out" 2>&1
  want "check is NOT ready when the role's trust policy is '$kind'" "1" "$?"
  if grep -q 'READY' "$SANDBOX/check-trust.out"
  then bad "check reported READY for a role EC2 cannot assume (trust '$kind')"
  else ok "check withholds READY for a role EC2 cannot assume (trust '$kind')"; fi
  if grep -q 'sts:AssumeRole' "$SANDBOX/check-trust.out"
  then ok "check names the broken trust relationship in its diagnosis (trust '$kind')"
  else bad "check does not say WHY it is not ready (trust '$kind')"; fi
done
# The IAM API returns AssumeRolePolicyDocument URL-encoded; a decode failure must not be
# mistaken for a broken trust policy and block a perfectly good channel.
reset_calls
AWS_STUB_MODE=all-ok AWS_STUB_ROLE_TRUST=encoded SPARQ_BENCH_S3_BUCKET=bkt bash "$LIB" check >/dev/null 2>&1
want "check reads a URL-encoded trust document as valid" "0" "$?"

# --------------------------------------------------------------------------- #
# 9. fetch: right URI, and never fatal to the caller.
# --------------------------------------------------------------------------- #
reset_calls
AWS_STUB_MODE=all-ok SPARQ_BENCH_S3_BUCKET=bkt bash -c ". '$LIB'; bench_s3_fetch myrun '$SANDBOX/out'" >/dev/null 2>&1
if grep -q 's3 cp --recursive .*s3://bkt/myrun/' "$CALLS"; then ok "fetch pulls s3://bkt/myrun/ recursively"; else bad "fetch did not issue the expected recursive cp"; fi
reset_calls
AWS_STUB_MODE=all-fail SPARQ_BENCH_S3_BUCKET=bkt bash -c ". '$LIB'; bench_s3_fetch myrun '$SANDBOX/out2'" >/dev/null 2>&1
want "fetch is non-fatal when S3 has nothing" "0" "$?"

# --------------------------------------------------------------------------- #
# 10. Both scripts are syntactically valid bash.
# --------------------------------------------------------------------------- #
if bash -n "$LIB";      then ok "bench-s3-results.sh parses";    else bad "bench-s3-results.sh has a syntax error"; fi
if bash -n "$UPLOADER"; then ok "s3-results-uploader.sh parses"; else bad "s3-results-uploader.sh has a syntax error"; fi
# The uploader must never be able to fail its gather.
if grep -q '^set -uo pipefail' "$UPLOADER"; then ok "uploader does not run under 'set -e' (best-effort side-car)"; else bad "uploader must not use 'set -e' — a telemetry failure would abort the gather"; fi

# --------------------------------------------------------------------------- #
# 11. Every wired launcher renders a user-data whose uploader line survives its
#     UNQUOTED heredoc, and passes the S3 launch arg to run-instances.
# --------------------------------------------------------------------------- #
# Each pattern is anchored at the USE SITE, not merely somewhere in the file: a bare
# `grep BENCH_S3_IAM_ARGS` would still pass after the run-instances argument was deleted,
# because the assignment line would remain.
for f in multi-axis-box.sh canonical-beir-bench.sh canonical-competitor-bench.sh canonical-materialize-bench.sh; do
  L="$ROOT/scripts/bench/$f"
  [ -f "$L" ] || { bad "$f missing"; continue; }
  if bash -n "$L"; then ok "$f parses"; else bad "$f has a syntax error"; fi
  if grep -qE '^[[:space:]]*\.[[:space:]]+.*bench-s3-results\.sh"?$' "$L"
  then ok "$f sources the S3 channel"; else bad "$f does not source bench-s3-results.sh"; fi
  # the arg is its own continuation line inside the run-instances invocation
  if grep -qE '^[[:space:]]*[$]BENCH_S3_IAM_ARGS[[:space:]]+\\$' "$L"
  then ok "$f passes the instance profile to run-instances"
  else bad "$f does not pass BENCH_S3_IAM_ARGS to run-instances"; fi
  # the uploader line is interpolated into the rendered user-data
  if grep -qE '^[[:space:]]*[$]BENCH_S3_UPLOAD_LINE[[:space:]]*$' "$L"
  then ok "$f starts the uploader from user-data"
  else bad "$f does not interpolate BENCH_S3_UPLOAD_LINE into user-data"; fi
  if grep -qE '^[[:space:]]*bench_s3_fetch[[:space:]]' "$L"
  then ok "$f pulls the durable copy back"; else bad "$f never calls bench_s3_fetch"; fi
done

printf '\n%d passed, %d failed\n' "$pass" "$fail"
[ "$fail" = 0 ] || exit 1
