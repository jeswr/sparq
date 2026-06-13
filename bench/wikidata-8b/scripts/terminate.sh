#!/usr/bin/env bash
# Tag-verified terminate + SG/keypair cleanup + cost ledger entry.
# Refuses the protected instance unconditionally. DRY-RUN by default.
# [OPUS-4.8] roborev 2016 (High): errexit added. Without it, the tag-verify XS
# below could fail (its `exit 1` only leaves the `bash -c` subshell) yet
# `terminate-instances` would still run on an unverified instance. With errexit,
# a failing XS aborts the whole script before any destructive AWS call.
set -euo pipefail
. "$(dirname "$0")/config.sh"
banner

if [ "$DRY" = 1 ]; then IID='<instance-id>'; else
  IID=$(cat "$STATE/instance-id") || { echo "no $STATE/instance-id" >&2; exit 1; }
fi

if [ "$IID" = "$PROTECTED_ID" ]; then
  echo "REFUSING: $IID is the protected prod instance. NEVER terminate it." >&2
  exit 1
fi

XS "verify purpose tag BEFORE terminating (refuse otherwise)" "
  TAG=\$(aws ec2 describe-instances --region '$REGION' --instance-ids '$IID' \
    --query 'Reservations[0].Instances[0].Tags[?Key==\`purpose\`]|[0].Value' --output text) &&
  [ \"\$TAG\" = '$TAG_PURPOSE' ] || { echo \"REFUSING: tag=\$TAG\" >&2; exit 1; }"

X "terminate" aws ec2 terminate-instances --region "$REGION" --instance-ids "$IID"
X "wait terminated" aws ec2 wait instance-terminated --region "$REGION" --instance-ids "$IID"

XS "delete ephemeral SG + keypair" "
  [ -f '$STATE/sg-id' ] && aws ec2 delete-security-group --region '$REGION' --group-id \"\$(cat '$STATE/sg-id')\" || true
  [ -f '$STATE/key-name' ] && aws ec2 delete-key-pair --region '$REGION' --key-name \"\$(cat '$STATE/key-name')\" || true"

XS "confirm nothing tagged is left (instances + volumes)" "
  aws ec2 describe-instances --region '$REGION' \
    --filters Name=tag:purpose,Values=$TAG_PURPOSE Name=instance-state-name,Values=pending,running,stopping,stopped \
    --query 'Reservations[].Instances[].InstanceId' --output text
  aws ec2 describe-volumes --region '$REGION' --filters Name=tag:purpose,Values=$TAG_PURPOSE \
    --query 'Volumes[].VolumeId' --output text"

XS "cost ledger entry" "
  T0=\$(cat '$STATE/launch-epoch' 2>/dev/null || echo 0); T1=\$(date -u +%s)
  if [ \"\$T0\" -gt 0 ]; then
    H=\$(echo \"scale=2; (\$T1 - \$T0)/3600\" | bc)
    C=\$(echo \"scale=2; \$H * $HOURLY_BURN_USD\" | bc)
    echo \"\$(date -u +%FT%TZ) $IID $INSTANCE_TYPE gp3-${VOL_GB}GB wall=\${H}h cost=\\\$\${C} (rate \\\$$HOURLY_BURN_USD/h)\" | tee -a '$STATE/cost-ledger.txt'
  fi"

XS "archive state files (prevents accidental reuse; keeps audit trail)" "
  mkdir -p '$STATE/done' && for f in instance-id ip launch-epoch launch-time sg-id key-name; do
    [ -f \"$STATE/\$f\" ] && mv \"$STATE/\$f\" '$STATE/done/'; done; true"

echo
echo "Terminated + cleaned (or would). Update STATUS.md + the \$30 ledger NOW (RUNBOOK §8)."
