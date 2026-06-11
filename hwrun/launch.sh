#!/usr/bin/env bash
# T2 + T12(=roadmap T1) hardware-validation orchestrator. Walks the launch ladder
# (spot c7i.48xlarge -> on-demand c7i.48xlarge -> on-demand c7i.24xlarge -> on-demand
# c7i.4xlarge), records every launch failure VERBATIM, drives hwrun/remote.sh on the
# first box that comes up, scp's ~/results back to hwrun/results/, and ALWAYS tears
# everything down (cleanup trap modeled on scripts/ec2-bench.sh).
#
# Cost guards (non-negotiable):
#   * user-data dead-man `shutdown -h +N` + --instance-initiated-shutdown-behavior terminate
#   * cleanup trap terminates instance + deletes ephemeral keypair/SG on ANY exit
#   * everything tagged purpose=sparq-hw-validation
#   * start/stop wall-clock recorded for $-accounting in hwrun/results/cost.txt
#
# SAFETY: i-090531b4ede8f2d3f (prod-solid-server) is terraform-managed and NEVER touched.
# We only ever act on the instance id we launched ourselves, and double-check its
# purpose=sparq-hw-validation tag before terminating.
set -uo pipefail

export AWS_PROFILE=pss
REGION=eu-west-2
SHA="${SPARQ_SHA:-72b35e3}"
HW="$(cd "$(dirname "$0")" && pwd)"
OUT="$HW/results"; mkdir -p "$OUT"
ERRLOG="$OUT/launch-errors.log"; : > "$ERRLOG"
PROTECTED_ID="i-090531b4ede8f2d3f"

WORK="$(mktemp -d)"
KEYFILE="$WORK/key"
KEY_NAME="sparq-hw-$$-${RANDOM}"
INSTANCE_ID=""; SG_ID=""; LAUNCH_EPOCH=0; ITYPE=""
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
        echo "instance $INSTANCE_ID ($ITYPE) ran ~${MIN} min" | tee -a "$OUT/cost.txt"
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
AMI_ARM=$(aws ec2 describe-images --region "$REGION" --owners 099720109477 \
  --filters "Name=name,Values=ubuntu/images/hvm-ssd-gp3/ubuntu-noble-24.04-arm64-server-*" "Name=state,Values=available" \
  --query 'sort_by(Images,&CreationDate)[-1].ImageId' --output text)
VPC=$(aws ec2 describe-vpcs --region "$REGION" --filters Name=isDefault,Values=true --query 'Vpcs[0].VpcId' --output text)
SUBNETS=$(aws ec2 describe-subnets --region "$REGION" --filters Name=vpc-id,Values="$VPC" "Name=default-for-az,Values=true" \
  --query 'Subnets[].SubnetId' --output text)
SUBNET1=$(echo "$SUBNETS" | awk '{print $1}')
MYIP=$(curl -s https://checkip.amazonaws.com)
echo "AMI=$AMI VPC=$VPC SUBNETS=$SUBNETS MYIP=$MYIP"

echo "== ephemeral keypair + SG =="
aws ec2 import-key-pair --region "$REGION" --key-name "$KEY_NAME" --public-key-material "fileb://${KEYFILE}.pub" \
  --tag-specifications 'ResourceType=key-pair,Tags=[{Key=purpose,Value=sparq-hw-validation}]' >/dev/null
SG_ID=$(aws ec2 create-security-group --region "$REGION" --group-name "$KEY_NAME" \
  --description "sparq hw validation (ephemeral)" --vpc-id "$VPC" \
  --tag-specifications 'ResourceType=security-group,Tags=[{Key=purpose,Value=sparq-hw-validation}]' \
  --query 'GroupId' --output text)
aws ec2 authorize-security-group-ingress --region "$REGION" --group-id "$SG_ID" --protocol tcp --port 22 --cidr "${MYIP}/32" >/dev/null

# try_launch <rung-name> <itype> <spot:0|1> <deadman-min> <subnet> [ami]
try_launch() {
  local rung="$1" itype="$2" spot="$3" deadman="$4" subnet="$5" ami="${6:-$AMI}"
  local market=""
  # plain text: AWS CLI v2 base64-encodes --user-data itself
  printf '#!/bin/bash\nshutdown -h +%s\n' "$deadman" > "$WORK/udata.txt"
  [ "$spot" = "1" ] && market="--instance-market-options MarketType=spot"
  echo "== RUNG $rung: launch $( [ "$spot" = 1 ] && echo spot || echo on-demand ) $itype (deadman +${deadman}m) =="
  local err id
  id=$(aws ec2 run-instances --region "$REGION" --image-id "$ami" --instance-type "$itype" \
    --key-name "$KEY_NAME" --security-group-ids "$SG_ID" --subnet-id "$subnet" --associate-public-ip-address \
    $market \
    --user-data "file://$WORK/udata.txt" \
    --instance-initiated-shutdown-behavior terminate \
    --block-device-mappings '[{"DeviceName":"/dev/sda1","Ebs":{"VolumeSize":500,"VolumeType":"gp3","Throughput":500,"Iops":6000,"DeleteOnTermination":true}}]' \
    --tag-specifications 'ResourceType=instance,Tags=[{Key=purpose,Value=sparq-hw-validation},{Key=Name,Value=sparq-hw-validation}]' \
    --query 'Instances[0].InstanceId' --output text 2>"$WORK/err.txt")
  if [ -n "$id" ] && [ "$id" != "None" ] && [[ "$id" == i-* ]]; then
    INSTANCE_ID="$id"; ITYPE="$itype"; LAUNCH_EPOCH=$(date +%s)
    echo "RUNG $rung SUCCEEDED: $id" | tee -a "$ERRLOG"
    return 0
  fi
  err=$(cat "$WORK/err.txt")
  { echo "--- RUNG $rung ($( [ "$spot" = 1 ] && echo spot || echo on-demand ) $itype) FAILED $(date -u +%FT%TZ) ---";
    echo "$err"; } | tee -a "$ERRLOG"
  return 1
}

LAUNCHED=""
DEADMAN=70; SLICE=1000000000
if try_launch 1 c7i.48xlarge 1 "$DEADMAN" "$SUBNET1"; then LAUNCHED=1
else
  # spot capacity errors are AZ-specific — give the other default subnets one shot each
  LASTERR=$(tail -3 "$ERRLOG")
  if echo "$LASTERR" | grep -qi 'InsufficientInstanceCapacity\|capacity-not-available\|Unsupported'; then
    for s in $SUBNETS; do
      [ "$s" = "$SUBNET1" ] && continue
      if try_launch "1-alt($s)" c7i.48xlarge 1 "$DEADMAN" "$s"; then LAUNCHED=1; break; fi
    done
  fi
fi
[ -z "$LAUNCHED" ] && { try_launch 2 c7i.48xlarge 0 "$DEADMAN" "$SUBNET1" && LAUNCHED=2; }
[ -z "$LAUNCHED" ] && { try_launch 3 c7i.24xlarge 0 "$DEADMAN" "$SUBNET1" && LAUNCHED=3; }
if [ -z "$LAUNCHED" ]; then
  DEADMAN=160  # 16 vCPU box is ~$0.77/hr; give the same workload more wall-clock
  try_launch 4 c7i.4xlarge 0 "$DEADMAN" "$SUBNET1" && LAUNCHED=4
fi
if [ -z "$LAUNCHED" ]; then
  # rung 4b: the other explicitly-sanctioned 16-vCPU fallback (Graviton, arm64 AMI)
  try_launch 4b c7g.4xlarge 0 "$DEADMAN" "$SUBNET1" "$AMI_ARM" && LAUNCHED=4b
fi
if [ -z "$LAUNCHED" ]; then
  echo "ALL RUNGS FAILED — see $ERRLOG"
  exit 1
fi
echo "$LAUNCHED" > "$OUT/rung.txt"

aws ec2 wait instance-running --region "$REGION" --instance-ids "$INSTANCE_ID"
IP=$(aws ec2 describe-instances --region "$REGION" --instance-ids "$INSTANCE_ID" \
  --query 'Reservations[0].Instances[0].PublicIpAddress' --output text)
LIFECYCLE=$(aws ec2 describe-instances --region "$REGION" --instance-ids "$INSTANCE_ID" \
  --query 'Reservations[0].Instances[0].InstanceLifecycle' --output text)
echo "instance $INSTANCE_ID ($ITYPE, lifecycle=${LIFECYCLE:-on-demand}) ip=$IP rung=$LAUNCHED" | tee -a "$OUT/cost.txt"

echo "== wait for sshd =="
UP=""
for i in $(seq 1 40); do ssh $SSHO "ubuntu@$IP" true 2>/dev/null && { UP=1; break; }; sleep 10; done
[ -z "$UP" ] && { echo "ssh never came up" | tee -a "$ERRLOG"; exit 1; }

# thread sweep list sized to the box
NPROC=$(ssh $SSHO "ubuntu@$IP" nproc)
case "$NPROC" in
  192) THREADS="1,2,4,8,16,32,64,96,128,192"; NUMA_AB=1 ;;
  96)  THREADS="1,2,4,8,16,32,48,64,96";      NUMA_AB=1 ;;
  16)  THREADS="1,2,4,8,16";                  NUMA_AB=0 ;;
  8)   THREADS="1,2,4,8";                     NUMA_AB=0 ;;
  *)   THREADS="1,2,4,8,$NPROC";              NUMA_AB=1 ;;
esac
echo "nproc=$NPROC threads=$THREADS slice=$SLICE"

echo "== run remote.sh (slice=$SLICE threads=$THREADS) =="
scp $SSHO "$HW/remote.sh" "ubuntu@$IP:remote.sh"
ssh $SSHO "ubuntu@$IP" "nohup bash remote.sh $SLICE $THREADS $NUMA_AB $SHA >/dev/null 2>&1 & echo started"

# poll for completion; remote.sh logs '== remote done' at the end. Hard ceiling just
# under the dead-man so we still scp partials before the box shuts itself down.
CEILING=$(( DEADMAN - 8 ))
DONE=""
for i in $(seq 1 $(( CEILING * 60 / 30 )) ); do
  sleep 30
  if ! ssh $SSHO "ubuntu@$IP" true 2>/dev/null; then echo "box unreachable (spot reclaim / shutdown?)"; break; fi
  if ssh $SSHO "ubuntu@$IP" "grep -q 'remote done' results/onbox.log 2>/dev/null"; then DONE=1; break; fi
done
[ -n "$DONE" ] && echo "remote run completed" || echo "remote run DID NOT signal completion — fetching partials" | tee -a "$ERRLOG"

echo "== fetch results =="
scp $SSHO -r "ubuntu@$IP:results" "$OUT/onbox" 2>/dev/null || \
  scp $SSHO -r "ubuntu@$IP:results/*" "$OUT/" 2>/dev/null || true
ls -la "$OUT" "$OUT/onbox" 2>/dev/null

echo "== done; cleanup trap will terminate =="
