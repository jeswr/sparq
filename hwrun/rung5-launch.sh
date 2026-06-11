#!/usr/bin/env bash
# Rung-5 orchestrator (sanctioned 2026-06-11): ONE on-demand r7i.2xlarge (8 vCPU, 64 GB,
# ~$0.60/hr eu-west-2) for the T2 partial-scale 1 B-triple Wikidata ingestion measurement
# the 16 GB M1 cannot run, plus the x86 homogeneous-core scaling sweep. Adapted from
# hwrun/launch.sh (the launch-ladder orchestrator whose rungs 1-4b are all quota-blocked;
# see research/hardware-validation-blocked.md). 8 vCPU fits the 14-vCPU headroom of the
# 16-vCPU L-1216C47A bucket (prod t3.large consumes 2).
#
# Cost guards (non-negotiable):
#   * BUDGET: target <=2.5 h on-box (~$1.50 + ~$0.11/hr EBS); hard cap $3
#   * user-data dead-man `shutdown -h +150` + --instance-initiated-shutdown-behavior terminate
#   * cleanup trap terminates instance + deletes ephemeral keypair/SG on ANY exit
#   * pre-flight refuses to run if any purpose=sparq-hw-validation instance already exists
#   * everything tagged purpose=sparq-hw-validation; terminate only after tag re-check
#   * start/stop wall-clock recorded for $-accounting in hwrun/results/rung5-cost.txt
#
# SAFETY: i-090531b4ede8f2d3f (prod-solid-server) is terraform-managed and NEVER touched.
# We only ever act on the instance id we launched ourselves, and double-check its
# purpose=sparq-hw-validation tag before terminating.
set -uo pipefail

export AWS_PROFILE=pss
REGION=eu-west-2
SHA="${SPARQ_SHA:-ef86e66}"
ITYPE=r7i.2xlarge
DEADMAN=150          # minutes; box self-destructs even if this script dies
SLICE=1000000000
THREADS="1,2,4,8"
HW="$(cd "$(dirname "$0")" && pwd)"
OUT="$HW/results"; mkdir -p "$OUT"
ERRLOG="$OUT/rung5-errors.log"; : > "$ERRLOG"
PROTECTED_ID="i-090531b4ede8f2d3f"

echo "== pre-flight: no orphan sparq-hw-validation instances =="
ORPHANS=$(aws ec2 describe-instances --region "$REGION" \
  --filters Name=tag:purpose,Values=sparq-hw-validation \
            Name=instance-state-name,Values=pending,running,stopping,stopped \
  --query 'Reservations[].Instances[].InstanceId' --output text)
if [ -n "$ORPHANS" ]; then
  echo "ABORT: orphan validation instance(s) exist: $ORPHANS" | tee -a "$ERRLOG"
  exit 1
fi

WORK="$(mktemp -d)"
KEYFILE="$WORK/key"
KEY_NAME="sparq-hw-rung5-$$-${RANDOM}"
INSTANCE_ID=""; SG_ID=""; LAUNCH_EPOCH=0
ssh-keygen -t ed25519 -N '' -f "$KEYFILE" -q

cleanup() {
  set +e
  echo "== cleanup $(date -u +%FT%TZ) =="
  if [ -n "$INSTANCE_ID" ] && [ "$INSTANCE_ID" != "$PROTECTED_ID" ]; then
    # verify it is OURS by tag before terminating
    TAGOK=$(aws ec2 describe-instances --region "$REGION" --instance-ids "$INSTANCE_ID" \
      --query 'Reservations[0].Instances[0].Tags[?Key==`purpose`]|[0].Value' --output text 2>/dev/null)
    if [ "$TAGOK" = "sparq-hw-validation" ]; then
      aws ec2 terminate-instances --region "$REGION" --instance-ids "$INSTANCE_ID" >/dev/null 2>&1
      aws ec2 wait instance-terminated --region "$REGION" --instance-ids "$INSTANCE_ID" >/dev/null 2>&1
      END_EPOCH=$(date +%s)
      if [ "$LAUNCH_EPOCH" -gt 0 ]; then
        MIN=$(( (END_EPOCH - LAUNCH_EPOCH + 59) / 60 ))
        echo "instance $INSTANCE_ID ($ITYPE) ran ~${MIN} min" | tee -a "$OUT/rung5-cost.txt"
      fi
      echo "terminated $INSTANCE_ID"
    else
      echo "REFUSING to terminate $INSTANCE_ID (tag=$TAGOK)" | tee -a "$ERRLOG"
    fi
  fi
  [ -n "$SG_ID" ] && aws ec2 delete-security-group --region "$REGION" --group-id "$SG_ID" >/dev/null 2>&1
  aws ec2 delete-key-pair --region "$REGION" --key-name "$KEY_NAME" >/dev/null 2>&1
  echo "cleanup done"
}
trap cleanup EXIT

SSHO="-i $KEYFILE -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o ConnectTimeout=15"

echo "== resolve AMI / network =="
AMI=$(aws ec2 describe-images --region "$REGION" --owners 099720109477 \
  --filters "Name=name,Values=ubuntu/images/hvm-ssd-gp3/ubuntu-noble-24.04-amd64-server-*" "Name=state,Values=available" \
  --query 'sort_by(Images,&CreationDate)[-1].ImageId' --output text)
VPC=$(aws ec2 describe-vpcs --region "$REGION" --filters Name=isDefault,Values=true --query 'Vpcs[0].VpcId' --output text)
SUBNET1=$(aws ec2 describe-subnets --region "$REGION" --filters Name=vpc-id,Values="$VPC" "Name=default-for-az,Values=true" \
  --query 'Subnets[0].SubnetId' --output text)
MYIP=$(curl -s https://checkip.amazonaws.com)
echo "AMI=$AMI VPC=$VPC SUBNET=$SUBNET1 MYIP=$MYIP"

echo "== ephemeral keypair + SG =="
aws ec2 import-key-pair --region "$REGION" --key-name "$KEY_NAME" --public-key-material "fileb://${KEYFILE}.pub" \
  --tag-specifications 'ResourceType=key-pair,Tags=[{Key=purpose,Value=sparq-hw-validation}]' >/dev/null
SG_ID=$(aws ec2 create-security-group --region "$REGION" --group-name "$KEY_NAME" \
  --description "sparq hw validation rung5 (ephemeral)" --vpc-id "$VPC" \
  --tag-specifications 'ResourceType=security-group,Tags=[{Key=purpose,Value=sparq-hw-validation}]' \
  --query 'GroupId' --output text)
aws ec2 authorize-security-group-ingress --region "$REGION" --group-id "$SG_ID" --protocol tcp --port 22 --cidr "${MYIP}/32" >/dev/null

echo "== RUNG 5: launch on-demand $ITYPE (deadman +${DEADMAN}m, 400 GB gp3) =="
# plain text: AWS CLI v2 base64-encodes --user-data itself
printf '#!/bin/bash\nshutdown -h +%s\n' "$DEADMAN" > "$WORK/udata.txt"
INSTANCE_ID=$(aws ec2 run-instances --region "$REGION" --image-id "$AMI" --instance-type "$ITYPE" \
  --key-name "$KEY_NAME" --security-group-ids "$SG_ID" --subnet-id "$SUBNET1" --associate-public-ip-address \
  --user-data "file://$WORK/udata.txt" \
  --instance-initiated-shutdown-behavior terminate \
  --block-device-mappings '[{"DeviceName":"/dev/sda1","Ebs":{"VolumeSize":400,"VolumeType":"gp3","Throughput":500,"Iops":6000,"DeleteOnTermination":true}}]' \
  --tag-specifications 'ResourceType=instance,Tags=[{Key=purpose,Value=sparq-hw-validation},{Key=Name,Value=sparq-hw-validation-rung5}]' \
  --query 'Instances[0].InstanceId' --output text 2>"$WORK/err.txt")
if [ -z "$INSTANCE_ID" ] || [ "$INSTANCE_ID" = "None" ] || [[ "$INSTANCE_ID" != i-* ]]; then
  INSTANCE_ID=""
  { echo "--- RUNG 5 (on-demand $ITYPE) FAILED $(date -u +%FT%TZ) ---"; cat "$WORK/err.txt"; } | tee -a "$ERRLOG"
  exit 1
fi
LAUNCH_EPOCH=$(date +%s)
echo "RUNG 5 SUCCEEDED: $INSTANCE_ID" | tee -a "$ERRLOG"

aws ec2 wait instance-running --region "$REGION" --instance-ids "$INSTANCE_ID"
IP=$(aws ec2 describe-instances --region "$REGION" --instance-ids "$INSTANCE_ID" \
  --query 'Reservations[0].Instances[0].PublicIpAddress' --output text)
echo "instance $INSTANCE_ID ($ITYPE, on-demand) ip=$IP launched $(date -u +%FT%TZ)" | tee -a "$OUT/rung5-cost.txt"

echo "== wait for sshd =="
UP=""
for i in $(seq 1 40); do ssh $SSHO "ubuntu@$IP" true 2>/dev/null && { UP=1; break; }; sleep 10; done
[ -z "$UP" ] && { echo "ssh never came up" | tee -a "$ERRLOG"; exit 1; }

echo "== run rung5-remote.sh (slice=$SLICE threads=$THREADS sha=$SHA) =="
scp $SSHO "$HW/rung5-remote.sh" "ubuntu@$IP:remote.sh"
ssh $SSHO "ubuntu@$IP" "nohup bash remote.sh $SLICE $THREADS $SHA >/dev/null 2>&1 & echo started"

# poll for completion; remote.sh logs '== remote done' at the end. Hard ceiling just
# under the dead-man so we still scp partials before the box shuts itself down.
CEILING=$(( DEADMAN - 8 ))
DONE=""
for i in $(seq 1 $(( CEILING * 60 / 30 )) ); do
  sleep 30
  if ! ssh $SSHO "ubuntu@$IP" true 2>/dev/null; then echo "box unreachable (shutdown?)"; break; fi
  if ssh $SSHO "ubuntu@$IP" "grep -q 'remote done' results/onbox.log 2>/dev/null"; then DONE=1; break; fi
done
[ -n "$DONE" ] && echo "remote run completed" || echo "remote run DID NOT signal completion — fetching partials" | tee -a "$ERRLOG"

echo "== fetch results =="
mkdir -p "$OUT/rung5"
scp $SSHO -r "ubuntu@$IP:results/*" "$OUT/rung5/" 2>/dev/null || true
ls -la "$OUT/rung5" 2>/dev/null

echo "== done; cleanup trap will terminate =="
