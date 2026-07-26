#!/usr/bin/env bash
# [OPUS-5] sq-ffaa9 — DURABLE S3 results channel for `purpose=sparq-bench` EC2 boxes.
#
# 🤖 SPARQ agent. WHY THIS EXISTS: on the canonical x86_64 Nitro boxes (c6i.4xlarge,
# the class every `canonical-*-bench.sh` launcher uses) the two existing retrieval
# channels can BOTH fail at once:
#   * `aws ec2 get-console-output` — the Nitro serial console does not reliably expose
#     userspace `/dev/console` writes, the 64KB ring wraps under a chatty build, and the
#     buffer is wiped on terminate. Recovered text is truncated/garbled or absent.
#   * SSH/scp pull — dies when the box saturates (the exact condition a heavy gather
#     creates) and is gone the instant the watchdog terminates the box.
# When both fail the run is UNRETRIEVABLE and the compute spend is wasted. This module
# adds a third, TERMINATION-INDEPENDENT channel: the box streams envelopes + logs to S3
# while it runs, and the operator reads them back from S3 afterwards — even from a box
# that already self-terminated.
#
# TWO ROLES IN ONE FILE:
#   * sourced  — defines `bench_s3_*` helpers for the launchers (no side effects).
#   * executed — the operator CLI: plan / provision / check / fetch / teardown.
#
# ------------------------------------------------------------------------------------
# OPERATOR CLI
#   scripts/bench/bench-s3-results.sh plan            # print the exact IAM/S3 objects
#                                                     # + policy JSON. Makes NO aws call.
#   AWS_PROFILE=… …/bench-s3-results.sh provision     # idempotent create (ADMIN creds)
#   AWS_PROFILE=… …/bench-s3-results.sh check         # verify bucket + role + that the
#                                                     # role's trust admits EC2 + that the
#                                                     # profile CARRIES that role
#   AWS_PROFILE=… …/bench-s3-results.sh fetch <run-id> <dest-dir>
#   AWS_PROFILE=… …/bench-s3-results.sh teardown      # remove what provision created
#   scripts/bench/bench-s3-results.sh --self-test     # pure-predicate self-checks
#
# `provision` needs IAM + S3 admin rights, which the bench role deliberately does NOT
# have — that is the MAINTAINER credential step this bead is blocked on. Everything
# else here runs on the ordinary bench credentials.
#
# ------------------------------------------------------------------------------------
# ENV (all optional; the channel is OFF and every launcher behaves EXACTLY as before
# when SPARQ_BENCH_S3_BUCKET is unset — this is a strictly additive capability):
#   SPARQ_BENCH_S3_BUCKET      bucket for results. UNSET => channel disabled.
#   SPARQ_BENCH_INSTANCE_PROFILE  instance profile to attach (default sparq-bench-results)
#   SPARQ_BENCH_S3_ROLE        IAM role inside that profile   (default sparq-bench-results)
#   SPARQ_BENCH_S3_RETENTION_DAYS  lifecycle expiry in days   (default 30)
#   REGION / AWS_REGION        default eu-west-2
#
# ------------------------------------------------------------------------------------
# SECURITY POSTURE — the instance role is WRITE-ONLY, on purpose:
#   allowed:  s3:PutObject, s3:AbortMultipartUpload   on  arn:aws:s3:::<bucket>/*
#   NOT granted: GetObject, ListBucket, DeleteObject, and no bucket-level action at all.
# A compromised or misbehaving bench box therefore cannot read back another run's
# results, enumerate the bucket, or destroy history — it can only append its own
# objects. The uploader is written against exactly this grant: it uses
# `aws s3 cp --recursive` (PutObject only) and NEVER `aws s3 sync`, which would need
# ListBucket+GetObject to diff and would fail closed under this policy.
# `scripts/tests/test_bench_s3_results.sh` pins that coupling so a future edit cannot
# silently widen the policy or introduce a `sync`.
#
# `set -euo pipefail` is applied ONLY when this file is EXECUTED — sourcing it must not
# change the launcher's shell options behind its back.
# (`if`, not `[ … ] && …`: the latter returns 1 when sourced, which the launchers'
# own `set -e` would treat as a fatal error at the `.` line.)
if [ "${BASH_SOURCE[0]}" = "$0" ]; then set -euo pipefail; fi

BENCH_S3_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BENCH_S3_UPLOADER_REL="scripts/bench/s3-results-uploader.sh"

: "${SPARQ_BENCH_INSTANCE_PROFILE:=sparq-bench-results}"
: "${SPARQ_BENCH_S3_ROLE:=sparq-bench-results}"
: "${SPARQ_BENCH_S3_RETENTION_DAYS:=30}"
: "${SPARQ_BENCH_S3_BUCKET:=}"
BENCH_S3_REGION="${REGION:-${AWS_REGION:-eu-west-2}}"

bench_s3_log() { printf '[bench-s3] %s\n' "$*" >&2; }

# --------------------------------------------------------------------------------- #
# Sourceable helpers — used by the canonical launchers.
# --------------------------------------------------------------------------------- #

# rc 0 iff the durable channel is configured.
bench_s3_enabled() { [ -n "${SPARQ_BENCH_S3_BUCKET:-}" ]; }

# bench_s3_uri <run-id>  ->  s3://<bucket>/<run-id>
bench_s3_uri() { printf 's3://%s/%s' "$SPARQ_BENCH_S3_BUCKET" "$1"; }

# rc 0 iff the instance profile EXISTS *and* carries EXACTLY the configured role.
#
# Existence is NOT the useful predicate. An instance profile can exist while holding no
# role at all, or while holding SOMEONE ELSE'S role — and a box launched with either
# gets credentials that cannot PutObject, which is precisely the silent unretrievable
# run this module exists to prevent. `provision` (verify), `check` (report) and
# `bench_s3_iam_args` (degrade) all route through this one predicate so the three can
# never drift into disagreeing about what "attached" means.
#
# Needs only iam:GetInstanceProfile — the same grant `bench_s3_passrole_policy` already
# hands the launcher principal; --query is evaluated client-side.
# bench_s3_profile_has_role [profile] [role]
bench_s3_profile_has_role() {
  local profile="${1:-$SPARQ_BENCH_INSTANCE_PROFILE}" role="${2:-$SPARQ_BENCH_S3_ROLE}" roles
  roles=$(aws iam get-instance-profile --instance-profile-name "$profile" \
    --query 'InstanceProfile.Roles[].RoleName' --output text 2>/dev/null) || return 1
  # Whitespace-pad both sides so the match is an exact WHOLE role name, never a prefix:
  # `sparq-bench-results` must not be satisfied by `sparq-bench-results-readonly`.
  # `--output text` renders a list tab-separated (sometimes newline-separated); squeeze
  # either into single spaces first.
  case " $(printf '%s' "$roles" | tr -s '[:space:]' ' ') " in
    *" $role "*) return 0 ;;
    *) return 1 ;;
  esac
}

# The `--iam-instance-profile` argument for `aws ec2 run-instances`, or NOTHING.
#
# Degrades to empty (channel silently off, launcher unchanged) rather than failing the
# launch when the profile has not been provisioned yet, does not carry the upload role,
# or the caller lacks iam:Get* — an unretrievable run is bad, but a run that never
# STARTS because of a missing IAM read grant is worse, and the operator gets a loud
# warning either way. Attaching a profile KNOWN not to carry the role would be the worst
# of the three: the launch looks provisioned and the uploads fail on the box.
#
# SCOPE: this verifies role MEMBERSHIP only, not the role's trust document — the launcher
# principal is granted iam:GetInstanceProfile but deliberately not iam:GetRole. A role EC2
# cannot assume is caught by `check`, which provision converges; run it before a canonical
# gather rather than relying on launch time to notice.
bench_s3_iam_args() {
  bench_s3_enabled || return 0
  if ! bench_s3_profile_has_role; then
    bench_s3_log "WARN: instance profile '$SPARQ_BENCH_INSTANCE_PROFILE' is not found/readable, or does not carry role '$SPARQ_BENCH_S3_ROLE' — launching WITHOUT the durable S3 results channel."
    bench_s3_log "WARN: run '$BENCH_S3_DIR/bench-s3-results.sh provision' with admin credentials first (bead sq-ffaa9)."
    return 0
  fi
  printf -- '--iam-instance-profile Name=%s' "$SPARQ_BENCH_INSTANCE_PROFILE"
}

# The ONE user-data line that starts the instance-side uploader, or `true` when the
# channel is off.
#
# CONTRACT (pinned by the self-test): the emitted line contains NO `$` at all. Every
# launcher renders its user-data through an UNQUOTED heredoc, where an unescaped `$`
# would be expanded by the LAUNCHER's shell instead of the instance's. Keeping the line
# dollar-free makes it safe to interpolate into any of them without per-site escaping.
# All variability is passed as pre-expanded `env` assignments here, and the uploader
# itself is a committed, reviewable, separately-tested script in the cloned repo — so
# this stays ~150 bytes against the hard 16384B user-data limit.
#
# bench_s3_userdata_line <run-id> <dir>[,<dir>…]
bench_s3_userdata_line() {
  local run_id="$1" dirs="$2"
  bench_s3_enabled || { printf 'true'; return 0; }
  printf 'setsid nohup env S3_URI=%s UPLOAD_PATHS=%s bash %s >/var/log/s3-uploader.log 2>&1 < /dev/null &' \
    "$(bench_s3_uri "$run_id")" "$dirs" "$BENCH_S3_UPLOADER_REL"
}

# Pull a run's objects back down. Works on a box that has ALREADY terminated — the
# whole point of the channel. Never fails the caller.
# bench_s3_fetch <run-id> <dest-dir>
bench_s3_fetch() {
  local run_id="$1" dest="$2"
  bench_s3_enabled || return 0
  mkdir -p "$dest"
  bench_s3_log "fetching $(bench_s3_uri "$run_id")/ -> $dest"
  if aws s3 cp --recursive --only-show-errors "$(bench_s3_uri "$run_id")/" "$dest/" 2>/dev/null; then
    bench_s3_log "durable S3 copy retrieved into $dest"
  else
    bench_s3_log "WARN: no objects retrieved from $(bench_s3_uri "$run_id")/ (box may never have reached the uploader)"
  fi
  return 0
}

# --------------------------------------------------------------------------------- #
# Policy documents — single source of truth for `plan`, `provision` and `check`.
# --------------------------------------------------------------------------------- #

bench_s3_trust_policy() {
  cat <<'JSON'
{"Version":"2012-10-17","Statement":[{"Effect":"Allow","Principal":{"Service":"ec2.amazonaws.com"},"Action":"sts:AssumeRole"}]}
JSON
}

# Write-only, single-bucket, object-scoped. See the SECURITY POSTURE note above.
bench_s3_write_policy() {
  local bucket="${1:-$SPARQ_BENCH_S3_BUCKET}"
  cat <<JSON
{"Version":"2012-10-17","Statement":[{"Sid":"SparqBenchResultsWriteOnly","Effect":"Allow","Action":["s3:PutObject","s3:AbortMultipartUpload"],"Resource":"arn:aws:s3:::${bucket}/*"}]}
JSON
}

# The grant the LAUNCHER principal needs (attaching an instance profile requires
# iam:PassRole on the role — a launch without it fails with an explicit AccessDenied).
bench_s3_passrole_policy() {
  local account="${1:-<ACCOUNT_ID>}"
  cat <<JSON
{"Version":"2012-10-17","Statement":[{"Sid":"SparqBenchPassRole","Effect":"Allow","Action":"iam:PassRole","Resource":"arn:aws:iam::${account}:role/${SPARQ_BENCH_S3_ROLE}","Condition":{"StringEquals":{"iam:PassedToService":"ec2.amazonaws.com"}}},{"Sid":"SparqBenchReadProfile","Effect":"Allow","Action":"iam:GetInstanceProfile","Resource":"arn:aws:iam::${account}:instance-profile/${SPARQ_BENCH_INSTANCE_PROFILE}"}]}
JSON
}

# rc 0 iff <document> lets the EC2 service assume the role; rc 1 otherwise.
#
# Existence of the role is NOT the useful predicate either. `provision` skips create-role
# for a role that already exists, so a SAME-NAMED role made for something else — or one
# whose trust document drifted by hand — can be attached to the instance profile, satisfy
# every existence and membership check, and be reported READY, while EC2 cannot assume it
# at all: the box boots with NO role credentials and every upload fails. That is the same
# silent unretrievable run this module exists to prevent, so the trust relationship is
# converged by `provision` and checked by `check` rather than assumed by both.
#
# Shape-tolerant, because `get-role` output varies: `Statement` may be an object or a list,
# `Action` and `Principal.Service` a string or a list, and the document arrives either
# already decoded (awscli normally decodes it) or still URL-encoded as the IAM API returns
# it on the wire.
#
# HONEST SCOPE: this is a DRIFT DETECTOR, not an IAM policy evaluator. It looks for an
# Allow of sts:AssumeRole to ec2.amazonaws.com; it does not interpret Deny statements or
# Conditions, so a document can satisfy it and still be refused by IAM at assume time.
# bench_s3_trust_allows_ec2 <document>
bench_s3_trust_allows_ec2() {
  if command -v python3 >/dev/null 2>&1; then
    BENCH_S3_TRUST_DOC="$1" python3 -c '
import json, os, sys
from urllib.parse import unquote

raw = os.environ["BENCH_S3_TRUST_DOC"].strip()
try:
    doc = json.loads(raw)
except ValueError:
    doc = None
if isinstance(doc, str) or doc is None:
    # Still URL-encoded: bare, or wrapped in a JSON string by --output json.
    doc = json.loads(unquote(doc if isinstance(doc, str) else raw))
if not isinstance(doc, dict):
    sys.exit(1)

def listify(value):
    return value if isinstance(value, list) else [value]

for stmt in listify(doc.get("Statement") or []):
    if not isinstance(stmt, dict) or stmt.get("Effect") != "Allow":
        continue
    actions = [str(a).lower() for a in listify(stmt.get("Action") or [])]
    if not any(a in ("sts:assumerole", "sts:*", "*") for a in actions):
        continue
    principal = stmt.get("Principal")
    principal = principal if isinstance(principal, dict) else {}
    services = [str(s).lower() for s in listify(principal.get("Service") or [])]
    if "ec2.amazonaws.com" in services:
        sys.exit(0)
sys.exit(1)
' 2>/dev/null
    return
  fi
  # No python3: fall back to a substring test. `ec2.amazonaws.com` survives URL-encoding
  # verbatim (only the punctuation around it is escaped), so this needs no decoding, and it
  # still catches the drift that actually happens — a trust document that does not name the
  # EC2 service principal AT ALL. It cannot tell a mis-shaped document from a good one.
  case "$1" in *ec2.amazonaws.com*) return 0 ;; *) return 1 ;; esac
}

# The role's EFFECTIVE trust document, as awscli renders it. rc mirrors `get-role`, so a
# missing role (or no iam:GetRole grant) is distinguishable from a readable-but-wrong one.
# bench_s3_role_trust_doc [role]
bench_s3_role_trust_doc() {
  aws iam get-role --role-name "${1:-$SPARQ_BENCH_S3_ROLE}" \
    --query 'Role.AssumeRolePolicyDocument' --output json 2>/dev/null
}

bench_s3_lifecycle() {
  cat <<JSON
{"Rules":[{"ID":"sparq-bench-results-expiry","Status":"Enabled","Filter":{"Prefix":""},"Expiration":{"Days":${SPARQ_BENCH_S3_RETENTION_DAYS}},"AbortIncompleteMultipartUpload":{"DaysAfterInitiation":1}}]}
JSON
}

# --------------------------------------------------------------------------------- #
# CLI subcommands
# --------------------------------------------------------------------------------- #

cmd_plan() {
  local bucket="${SPARQ_BENCH_S3_BUCKET:-sparq-bench-results-<ACCOUNT_ID>}"
  cat <<EOF
[bench-s3 plan] NO AWS CALL IS MADE BY THIS SUBCOMMAND.

Region ................ $BENCH_S3_REGION
Bucket ................ $bucket
IAM role .............. $SPARQ_BENCH_S3_ROLE
Instance profile ...... $SPARQ_BENCH_INSTANCE_PROFILE
Object retention ...... ${SPARQ_BENCH_S3_RETENTION_DAYS} days (lifecycle expiry)

Objects 'provision' creates (idempotently), in order:
  1. s3  create-bucket $bucket (LocationConstraint=$BENCH_S3_REGION unless us-east-1)
  2. s3api put-public-access-block  (all four blocks ON)
  3. s3api put-bucket-lifecycle-configuration  (expire after ${SPARQ_BENCH_S3_RETENTION_DAYS}d)
  4. iam create-role $SPARQ_BENCH_S3_ROLE          (trust: ec2.amazonaws.com)
     — or, when the role already exists, iam update-assume-role-policy with the SAME
       trust document, so a drifted/foreign trust principal is repaired, not inherited
  5. iam put-role-policy  sparq-bench-results-write (WRITE-ONLY, object-scoped)
  6. iam create-instance-profile $SPARQ_BENCH_INSTANCE_PROFILE
  7. iam add-role-to-instance-profile

--- trust policy (role $SPARQ_BENCH_S3_ROLE) ---
$(bench_s3_trust_policy)
--- permission policy (role $SPARQ_BENCH_S3_ROLE) ---
$(bench_s3_write_policy "$bucket")
--- bucket lifecycle ---
$(bench_s3_lifecycle)

--- ALSO REQUIRED, on the LAUNCHER principal (e.g. the 'pss' profile) ---
Attaching an instance profile needs iam:PassRole; without it run-instances fails
AccessDenied. Attach this to whatever principal runs the canonical launchers:
$(bench_s3_passrole_policy)

Then export, for every canonical launch:
  export SPARQ_BENCH_S3_BUCKET=$bucket
EOF
}

cmd_provision() {
  command -v aws >/dev/null || { bench_s3_log "ERROR: aws CLI not found"; return 2; }
  local account bucket
  account=$(aws sts get-caller-identity --query Account --output text)
  if [ -z "$account" ] || [ "$account" = "None" ]; then bench_s3_log "ERROR: could not resolve the AWS account id"; return 2; fi
  bucket="${SPARQ_BENCH_S3_BUCKET:-sparq-bench-results-$account}"
  bench_s3_log "account=$account bucket=$bucket region=$BENCH_S3_REGION"

  # 1-3. bucket, locked down + self-expiring (cost + data-retention hygiene).
  if aws s3api head-bucket --bucket "$bucket" >/dev/null 2>&1; then
    bench_s3_log "bucket $bucket already exists — skipping create"
  elif [ "$BENCH_S3_REGION" = "us-east-1" ]; then
    aws s3api create-bucket --bucket "$bucket" --region "$BENCH_S3_REGION" >/dev/null
  else
    aws s3api create-bucket --bucket "$bucket" --region "$BENCH_S3_REGION" \
      --create-bucket-configuration "LocationConstraint=$BENCH_S3_REGION" >/dev/null
  fi
  aws s3api put-public-access-block --bucket "$bucket" --public-access-block-configuration \
    'BlockPublicAcls=true,IgnorePublicAcls=true,BlockPublicPolicy=true,RestrictPublicBuckets=true' >/dev/null
  aws s3api put-bucket-lifecycle-configuration --bucket "$bucket" \
    --lifecycle-configuration "$(bench_s3_lifecycle)" >/dev/null
  bench_s3_log "bucket ready (public access blocked, ${SPARQ_BENCH_S3_RETENTION_DAYS}d expiry)"

  # 4-5. role + trust relationship + write-only policy.
  if aws iam get-role --role-name "$SPARQ_BENCH_S3_ROLE" >/dev/null 2>&1; then
    bench_s3_log "role $SPARQ_BENCH_S3_ROLE already exists — converging its trust + permission policies in place"
    # CONVERGE, do not merely reuse. An existing same-named role can carry an absent or
    # wrong trust principal (created for another service, or drifted by hand); it would
    # still attach to the profile and pass every membership check while EC2 could not
    # assume it, so the box would come up with no credentials and upload nothing. Provision
    # is the sole convergence authority for the trust document — `check` only reports on it.
    aws iam update-assume-role-policy --role-name "$SPARQ_BENCH_S3_ROLE" \
      --policy-document "$(bench_s3_trust_policy)" >/dev/null
  else
    aws iam create-role --role-name "$SPARQ_BENCH_S3_ROLE" \
      --description "sparq canonical bench boxes: write-only results upload (sq-ffaa9)" \
      --assume-role-policy-document "$(bench_s3_trust_policy)" >/dev/null
  fi
  aws iam put-role-policy --role-name "$SPARQ_BENCH_S3_ROLE" \
    --policy-name sparq-bench-results-write \
    --policy-document "$(bench_s3_write_policy "$bucket")" >/dev/null

  # 6-7. instance profile.
  if aws iam get-instance-profile --instance-profile-name "$SPARQ_BENCH_INSTANCE_PROFILE" >/dev/null 2>&1; then
    bench_s3_log "instance profile $SPARQ_BENCH_INSTANCE_PROFILE already exists"
  else
    aws iam create-instance-profile --instance-profile-name "$SPARQ_BENCH_INSTANCE_PROFILE" >/dev/null
  fi
  # Idempotent: re-adding an already-attached role returns LimitExceeded/EntityAlreadyExists.
  # But a nonzero rc is AMBIGUOUS — it equally covers "the profile already holds a
  # DIFFERENT role" (a profile takes at most one), AccessDenied, invalid input and plain
  # service failure. Reading every failure as "already attached" would print `provisioned`
  # over a profile whose credentials cannot PutObject, so on failure VERIFY membership and
  # stop loudly when it is absent, rather than handing launchers a known-broken profile.
  if ! aws iam add-role-to-instance-profile --instance-profile-name "$SPARQ_BENCH_INSTANCE_PROFILE" \
      --role-name "$SPARQ_BENCH_S3_ROLE" >/dev/null 2>&1; then
    if bench_s3_profile_has_role; then
      bench_s3_log "role $SPARQ_BENCH_S3_ROLE already attached to $SPARQ_BENCH_INSTANCE_PROFILE"
    else
      bench_s3_log "ERROR: could not attach role $SPARQ_BENCH_S3_ROLE to instance profile $SPARQ_BENCH_INSTANCE_PROFILE,"
      bench_s3_log "ERROR: and the profile does not carry it — it may hold a different role (a profile takes"
      bench_s3_log "ERROR: at most one; detach with 'aws iam remove-role-from-instance-profile'), or these"
      bench_s3_log "ERROR: credentials lack iam:AddRoleToInstanceProfile. NOT provisioned: a box launched with"
      bench_s3_log "ERROR: this profile could not upload its results."
      return 1
    fi
  fi

  # IAM is eventually consistent: a profile used by run-instances too soon after
  # creation fails with "Invalid IAM Instance Profile name". Poll before declaring done —
  # on ROLE MEMBERSHIP, not mere existence, since existence is what the profile already
  # had before the role was ever attached.
  local attached=0
  for _ in $(seq 1 30); do
    if bench_s3_profile_has_role; then attached=1; break; fi
    sleep 2
  done
  if [ "$attached" != 1 ]; then
    bench_s3_log "ERROR: instance profile $SPARQ_BENCH_INSTANCE_PROFILE never reported role $SPARQ_BENCH_S3_ROLE as attached."
    bench_s3_log "ERROR: NOT provisioned — re-run 'check' before launching; boxes would upload nothing."
    return 1
  fi

  cat >&2 <<EOF
[bench-s3] provisioned. Enable the channel for canonical launches with:

    export SPARQ_BENCH_S3_BUCKET=$bucket

[bench-s3] STILL REQUIRED (separate principal): iam:PassRole on the launcher identity.
$(bench_s3_passrole_policy "$account")
EOF
}

cmd_check() {
  command -v aws >/dev/null || { bench_s3_log "ERROR: aws CLI not found"; return 2; }
  local rc=0
  bench_s3_enabled || { bench_s3_log "SPARQ_BENCH_S3_BUCKET is unset — durable channel DISABLED (launchers fall back to console+SSH only)"; return 1; }
  if aws s3api head-bucket --bucket "$SPARQ_BENCH_S3_BUCKET" >/dev/null 2>&1; then
    bench_s3_log "OK   bucket $SPARQ_BENCH_S3_BUCKET reachable"
  else
    bench_s3_log "FAIL bucket $SPARQ_BENCH_S3_BUCKET not reachable"; rc=1
  fi
  # The role EXISTING is not the guarantee either: it must be assumable BY EC2, or the box
  # launches with the profile attached and receives no credentials at all.
  local trust trust_rc=0
  if ! trust=$(bench_s3_role_trust_doc); then
    bench_s3_log "FAIL role $SPARQ_BENCH_S3_ROLE missing (or not readable with these credentials)"; rc=1
  elif [ -z "$trust" ] || [ "$trust" = "null" ]; then
    bench_s3_log "FAIL role $SPARQ_BENCH_S3_ROLE exists but its trust policy could not be read — withholding READY rather than assuming it is correct"; rc=1
  else
    bench_s3_trust_allows_ec2 "$trust" || trust_rc=$?
    if [ "$trust_rc" = 0 ]; then
      bench_s3_log "OK   role $SPARQ_BENCH_S3_ROLE exists and its trust policy lets ec2.amazonaws.com call sts:AssumeRole"
    else
      bench_s3_log "FAIL role $SPARQ_BENCH_S3_ROLE exists but its trust policy does NOT let ec2.amazonaws.com call sts:AssumeRole:"
      bench_s3_log "FAIL   a box launched with $SPARQ_BENCH_INSTANCE_PROFILE would receive NO role credentials and every upload would fail."
      bench_s3_log "FAIL   re-run 'provision' with admin credentials — it converges the trust policy — then check again."
      rc=1
    fi
  fi
  # The profile EXISTING is not the guarantee launchers need — it must carry the upload
  # role, or the box boots with credentials that cannot PutObject and the run is lost.
  if bench_s3_profile_has_role; then
    bench_s3_log "OK   instance profile $SPARQ_BENCH_INSTANCE_PROFILE carries role $SPARQ_BENCH_S3_ROLE"
  elif aws iam get-instance-profile --instance-profile-name "$SPARQ_BENCH_INSTANCE_PROFILE" >/dev/null 2>&1; then
    bench_s3_log "FAIL instance profile $SPARQ_BENCH_INSTANCE_PROFILE exists but does NOT carry role $SPARQ_BENCH_S3_ROLE — a box launched with it cannot upload results"; rc=1
  else
    bench_s3_log "FAIL instance profile $SPARQ_BENCH_INSTANCE_PROFILE missing"; rc=1
  fi
  if [ "$rc" = 0 ]; then
    bench_s3_log "durable results channel READY"
  else
    bench_s3_log "durable results channel NOT ready — run 'provision' with admin credentials"
  fi
  return "$rc"
}

cmd_teardown() {
  command -v aws >/dev/null || { bench_s3_log "ERROR: aws CLI not found"; return 2; }
  bench_s3_log "removing role/profile (the bucket + its objects are KEPT — delete deliberately)"
  aws iam remove-role-from-instance-profile --instance-profile-name "$SPARQ_BENCH_INSTANCE_PROFILE" \
    --role-name "$SPARQ_BENCH_S3_ROLE" >/dev/null 2>&1 || true
  aws iam delete-instance-profile --instance-profile-name "$SPARQ_BENCH_INSTANCE_PROFILE" >/dev/null 2>&1 || true
  aws iam delete-role-policy --role-name "$SPARQ_BENCH_S3_ROLE" --policy-name sparq-bench-results-write >/dev/null 2>&1 || true
  aws iam delete-role --role-name "$SPARQ_BENCH_S3_ROLE" >/dev/null 2>&1 || true
  bench_s3_log "teardown done"
}

# Pure-predicate self-checks: no AWS call, no network, no filesystem mutation.
cmd_self_test() {
  local fails=0
  chk() { if [ "$2" = "$3" ]; then printf 'ok   %s\n' "$1"; else printf 'FAIL %s (want %q, got %q)\n' "$1" "$2" "$3"; fails=$((fails + 1)); fi; }

  ( SPARQ_BENCH_S3_BUCKET="" ; bench_s3_enabled ) && { printf 'FAIL enabled-when-unset\n'; fails=$((fails + 1)); } || printf 'ok   disabled when bucket unset\n'
  chk "uri composition" "s3://b/r1" "$(SPARQ_BENCH_S3_BUCKET=b bench_s3_uri r1)"
  chk "userdata line is 'true' when disabled" "true" "$(SPARQ_BENCH_S3_BUCKET='' bench_s3_userdata_line r1 /d)"

  local line
  line="$(SPARQ_BENCH_S3_BUCKET=b bench_s3_userdata_line r1 /d)"
  case "$line" in *'$'*) printf 'FAIL userdata line contains $ (unquoted-heredoc hazard)\n'; fails=$((fails + 1)) ;; *) printf 'ok   userdata line is dollar-free\n' ;; esac
  case "$line" in *"$BENCH_S3_UPLOADER_REL"*) printf 'ok   userdata line invokes the committed uploader\n' ;; *) printf 'FAIL userdata line does not invoke %s\n' "$BENCH_S3_UPLOADER_REL"; fails=$((fails + 1)) ;; esac

  # The write-only grant and the uploader's verbs must stay coupled.
  case "$(bench_s3_write_policy b)" in
    *'"s3:PutObject"'*) printf 'ok   policy grants PutObject\n' ;;
    *) printf 'FAIL policy is missing PutObject\n'; fails=$((fails + 1)) ;;
  esac
  case "$(bench_s3_write_policy b)" in
    *GetObject*|*ListBucket*|*DeleteObject*) printf 'FAIL policy is wider than write-only\n'; fails=$((fails + 1)) ;;
    *) printf 'ok   policy is write-only (no Get/List/Delete)\n' ;;
  esac

  # Trust predicate: the drift it must catch is an EXISTING role that is attachable and
  # reports healthy, but that EC2 cannot assume — so the box gets no credentials.
  if bench_s3_trust_allows_ec2 "$(bench_s3_trust_policy)"; then printf 'ok   own trust policy admits the EC2 service principal\n'; else printf 'FAIL own trust policy rejected by the trust predicate\n'; fails=$((fails + 1)); fi
  if bench_s3_trust_allows_ec2 '{"Version":"2012-10-17","Statement":[{"Effect":"Allow","Principal":{"Service":"lambda.amazonaws.com"},"Action":"sts:AssumeRole"}]}'
  then printf 'FAIL trust predicate accepts a non-EC2 service principal\n'; fails=$((fails + 1)); else printf 'ok   trust predicate rejects a non-EC2 service principal\n'; fi
  if bench_s3_trust_allows_ec2 '{"Version":"2012-10-17","Statement":[]}'
  then printf 'FAIL trust predicate accepts a trust policy with no statement\n'; fails=$((fails + 1)); else printf 'ok   trust predicate rejects an empty trust policy\n'; fi
  # `get-role` may hand the document back still URL-encoded, as the IAM API returns it.
  if bench_s3_trust_allows_ec2 '"%7B%22Version%22%3A%222012-10-17%22%2C%22Statement%22%3A%5B%7B%22Effect%22%3A%22Allow%22%2C%22Principal%22%3A%7B%22Service%22%3A%22ec2.amazonaws.com%22%7D%2C%22Action%22%3A%22sts%3AAssumeRole%22%7D%5D%7D"'
  then printf 'ok   trust predicate decodes a URL-encoded trust document\n'; else printf 'FAIL trust predicate cannot read a URL-encoded trust document\n'; fails=$((fails + 1)); fi

  if command -v python3 >/dev/null 2>&1; then
    local doc
    for doc in "$(bench_s3_trust_policy)" "$(bench_s3_write_policy b)" "$(bench_s3_passrole_policy 1234)" "$(bench_s3_lifecycle)"; do
      printf '%s' "$doc" | python3 -c 'import json,sys; json.load(sys.stdin)' \
        || { printf 'FAIL policy document is not valid JSON\n'; fails=$((fails + 1)); }
    done
    printf 'ok   all policy documents parse as JSON\n'
  fi

  [ "$fails" = 0 ] || { printf '%d self-test failure(s)\n' "$fails"; return 1; }
  printf 'bench-s3-results self-test: all checks passed\n'
}

# --------------------------------------------------------------------------------- #
# Dispatch — only when EXECUTED. Sourcing this file has no side effects beyond
# defining the helpers + the env defaults above.
# --------------------------------------------------------------------------------- #
if [ "${BASH_SOURCE[0]}" = "$0" ]; then
  case "${1:-}" in
    plan)        cmd_plan ;;
    provision)   cmd_provision ;;
    check)       cmd_check ;;
    teardown)    cmd_teardown ;;
    fetch)       [ "$#" -ge 3 ] || { bench_s3_log "usage: $0 fetch <run-id> <dest-dir>"; exit 2; }; bench_s3_fetch "$2" "$3" ;;
    --self-test) cmd_self_test ;;
    -h|--help|"") sed -n '2,61p' "$0"; exit 0 ;;
    *)           bench_s3_log "unknown subcommand: $1 (try plan|provision|check|fetch|teardown|--self-test)"; exit 2 ;;
  esac
fi
