#!/usr/bin/env bash
# [OPUS-4.8] sq-8dp3 — orphan-proof EC2 orchestrator for the competitor gather, with
# the RESULTS-LOSS FIX. The prior gather (sq-gbq0) lost the jena-shacl +
# rdf-validate-shacl envelopes because the box self-terminated on a post-gather
# `shutdown +1min` BEFORE the orchestrator could SSH-pull files off a
# DeleteOnTermination volume. The correct design: the orphan-proof self-terminate is
# right, but it must be the 3h WATCHDOG BACKSTOP ONLY — the gather must NOT eagerly
# shut the box down, and the orchestrator must pull results OUT while the box is
# still alive, then terminate explicitly. Results can therefore never be lost to a
# shutdown race.
#
# CHANNEL: SSH/scp pull. (A first cut tried framing envelopes to the serial console +
# ec2:get-console-output — role-compatible, but on Nitro/Graviton (c7g) the console
# API does NOT expose userspace /dev/console writes and the buffer is wiped on
# termination, so it captured nothing. SSH works on Nitro and needs no S3 / no
# instance profile.) An ephemeral keypair + a locked-down SG (port 22 from THIS host
# only) are created per run and deleted in cleanup.
#
# ORPHAN-PROOFING (agent-death-independent, triple backstop, 3h hard cap):
#   --instance-initiated-shutdown-behavior terminate
#   + detached  (sleep 10800; shutdown -h now)
#   + systemd-run --on-active=10800 shutdown -h now
#   + at now + 3h  (best-effort)
# The gather user-data writes results + a /root/GATHER_DONE sentinel, then STOPS (it
# does NOT shut down) — only the watchdog auto-terminates if the orchestrator dies.
# Tag purpose=sparq-bench.
#
# SAFETY: operates ONLY on the ONE instance it launches; NEVER touches prod
# (i-090531b4ede8f2d3f) or the work box. c7g.xlarge (4 vCPU) — within the 16-vCPU
# On-Demand bucket given prod+workbox already use 10.
#
# Usage:  AWS_PROFILE=pss scripts/gather-ec2.sh <branch> [<region>]
# Writes decoded envelopes under bench/competitor-results/ and prints each on stdout.
# ALWAYS terminates the instance + deletes the keypair/SG (cleanup trap).
set -euo pipefail

BRANCH="${1:?usage: gather-ec2.sh <branch> [region]}"
REGION="${2:-${AWS_REGION:-eu-west-2}}"
ITYPE="c7g.xlarge"                       # 4 vCPU arm64 (c7g.2xlarge FAILED VcpuLimitExceeded)
REPO="https://github.com/jeswr/sparq.git"
SCALE="${GATHER_SCALE:-50000}"
ITERS="${GATHER_ITERS:-4}"
TAGSPEC='ResourceType=instance,Tags=[{Key=purpose,Value=sparq-bench}]'
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RESULTS_DIR="$ROOT/bench/competitor-results"

log() { printf '[gather-ec2] %s\n' "$*" >&2; }

WORK="$(mktemp -d)"
KEYFILE="$WORK/key"
KEY_NAME="sparq-bench-$$-${RANDOM}"
INSTANCE_ID=""; SG_ID=""
ssh-keygen -t ed25519 -N '' -f "$KEYFILE" -q
SSHO="-i $KEYFILE -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o ConnectTimeout=15"

cleanup() {
  set +e
  if [ -n "$INSTANCE_ID" ]; then
    log "cleanup: terminating $INSTANCE_ID"
    aws ec2 terminate-instances --region "$REGION" --instance-ids "$INSTANCE_ID" >/dev/null 2>&1
    aws ec2 wait instance-terminated --region "$REGION" --instance-ids "$INSTANCE_ID" >/dev/null 2>&1
    log "cleanup: $INSTANCE_ID terminated"
  fi
  [ -n "$SG_ID" ] && aws ec2 delete-security-group --region "$REGION" --group-id "$SG_ID" >/dev/null 2>&1
  aws ec2 delete-key-pair --region "$REGION" --key-name "$KEY_NAME" >/dev/null 2>&1
  rm -rf "$WORK"
}
trap cleanup EXIT

log "resolve AMI / network in $REGION"
AMI=$(aws ec2 describe-images --region "$REGION" --owners 099720109477 \
  --filters "Name=name,Values=ubuntu/images/hvm-ssd-gp3/ubuntu-noble-24.04-arm64-server-*" "Name=state,Values=available" \
  --query 'sort_by(Images,&CreationDate)[-1].ImageId' --output text)
VPC=$(aws ec2 describe-vpcs --region "$REGION" --filters Name=isDefault,Values=true --query 'Vpcs[0].VpcId' --output text)
SUBNET=$(aws ec2 describe-subnets --region "$REGION" --filters Name=vpc-id,Values="$VPC" "Name=default-for-az,Values=true" --query 'Subnets[0].SubnetId' --output text)
MYIP=$(curl -s https://checkip.amazonaws.com)
log "AMI=$AMI VPC=$VPC SUBNET=$SUBNET MYIP=$MYIP"

log "keypair + locked-down security group (ssh from $MYIP/32 only)"
aws ec2 import-key-pair --region "$REGION" --key-name "$KEY_NAME" --public-key-material "fileb://${KEYFILE}.pub" >/dev/null
SG_ID=$(aws ec2 create-security-group --region "$REGION" --group-name "$KEY_NAME" --description "sparq bench gather (ephemeral)" --vpc-id "$VPC" --query 'GroupId' --output text)
aws ec2 authorize-security-group-ingress --region "$REGION" --group-id "$SG_ID" --protocol tcp --port 22 --cidr "${MYIP}/32" >/dev/null

# ---- user-data: build, gather (FIXED harness), write results + sentinel, then STOP.
# It does NOT shut down — only the 3h watchdog does (orphan backstop). The orchestrator
# pulls results while the box is still alive, then terminates it explicitly. ------------
USERDATA=$(cat <<UD
#!/bin/bash
set -x
exec > >(tee /var/log/gather.log) 2>&1
# --- triple orphan-proof backstop (agent-death-independent, 3h hard cap) ---
( sleep 10800; shutdown -h now ) &
systemd-run --on-active=10800 /sbin/shutdown -h now || true
command -v at >/dev/null 2>&1 && echo "/sbin/shutdown -h now" | at now + 3 hours || true

export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y -qq build-essential pkg-config git curl jq python3 python3-venv python3-pip nodejs npm default-jdk-headless raptor2-utils unzip at
( sleep 10800; shutdown -h now ) &
command -v at >/dev/null 2>&1 && echo "/sbin/shutdown -h now" | at now + 3 hours || true
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
# rustup -y is synchronous, but the cargo SHIM symlink can lag a moment behind the
# env file; export PATH AND wait for the binary so the build can't race past it.
export PATH="/root/.cargo/bin:\$PATH"
. /root/.cargo/env || true
for _ in \$(seq 1 30); do command -v cargo >/dev/null 2>&1 && break; sleep 2; done
command -v cargo || echo "WARN: cargo still not on PATH"

cd /root
git clone -q "$REPO" sparq
cd sparq
git fetch -q origin "$BRANCH"
git checkout -q "$BRANCH"
SHA=\$(git rev-parse --short HEAD)
echo "GATHER_SHA=\$SHA"

cargo build --release -q -p sparq-bench
cargo build --release -q -p sparq-shacl --example bench_shacl || true

python3 -m venv /root/shaclenv
/root/shaclenv/bin/pip install -q pyshacl==0.31.0 rdflib

JENA_VER=5.2.0
curl -sL -o /tmp/jena.zip "https://archive.apache.org/dist/jena/binaries/apache-jena-\${JENA_VER}.zip" || \
  curl -sL -o /tmp/jena.zip "https://dlcdn.apache.org/jena/binaries/apache-jena-\${JENA_VER}.zip"
unzip -q /tmp/jena.zip -d /opt
export JENA_HOME="/opt/apache-jena-\${JENA_VER}"

mkdir -p /root/jsmods && cd /root/jsmods
npm init -y >/dev/null 2>&1
npm i --no-audit --no-fund rdf-validate-shacl @rdfjs/data-model @rdfjs/dataset @rdfjs/term-map @rdfjs/namespace @rdfjs/parser-n3 @rdfjs/environment clownface >/dev/null 2>&1
cd /root/sparq

mapfile -t ARTS < <(bash bench/shacl/gen.sh 1 0)
DATA="\${ARTS[0]}"
SHAPES="\${ARTS[1]}"
echo "DATA=\$DATA SHAPES=\$SHAPES"

SHACL_DATA_TTL=/tmp/abox.ttl
rapper -i ntriples -o turtle "\$DATA" > "\$SHACL_DATA_TTL" 2>/dev/null || cp "\$DATA" "\$SHACL_DATA_TTL"

export QUIET_BOX=true

./scripts/gather-competitors.sh --run --only oxigraph --scale "$SCALE" --iters "$ITERS" || echo "oxigraph gather failed"

SHACL_DATA="\$SHACL_DATA_TTL" SHACL_SHAPES="\$SHAPES" ITERS="$ITERS" \
  PATH="/root/shaclenv/bin:\$PATH" \
  ./scripts/gather-competitors.sh --run --only pyshacl || echo "pyshacl gather failed"

SHACL_DATA="\$SHACL_DATA_TTL" SHACL_SHAPES="\$SHAPES" ITERS="$ITERS" \
  PATH="/opt/apache-jena-\${JENA_VER}/bin:/root/shaclenv/bin:\$PATH" \
  ./scripts/gather-competitors.sh --run --only jena-shacl || echo "jena gather failed"

SHACL_DATA="\$SHACL_DATA_TTL" SHACL_SHAPES="\$SHAPES" ITERS="$ITERS" JS_MODULES_DIR=/root/jsmods \
  ./scripts/gather-competitors.sh --run --only rdf-validate-shacl || echo "rvs gather failed"

ls -la bench/competitor-results/ || true
# sentinel LAST — the orchestrator waits for this, then pulls, then terminates.
echo "\$SHA" > /root/GATHER_DONE
sync
# Do NOT shut down here — the watchdog is the only auto-terminate (orphan backstop).
UD
)

log "launching $ITYPE"
INSTANCE_ID=$(aws ec2 run-instances --region "$REGION" --image-id "$AMI" --instance-type "$ITYPE" \
  --instance-initiated-shutdown-behavior terminate \
  --key-name "$KEY_NAME" --security-group-ids "$SG_ID" \
  --subnet-id "$SUBNET" --associate-public-ip-address \
  --block-device-mappings '[{"DeviceName":"/dev/sda1","Ebs":{"VolumeSize":30,"VolumeType":"gp3","DeleteOnTermination":true}}]' \
  --tag-specifications "$TAGSPEC" \
  --user-data "$USERDATA" \
  --query 'Instances[0].InstanceId' --output text)
log "launched INSTANCE_ID=$INSTANCE_ID"
aws ec2 wait instance-running --region "$REGION" --instance-ids "$INSTANCE_ID"
IP=$(aws ec2 describe-instances --region "$REGION" --instance-ids "$INSTANCE_ID" --query 'Reservations[0].Instances[0].PublicIpAddress' --output text)
log "public IP=$IP; waiting for sshd"
for i in $(seq 1 40); do ssh $SSHO "ubuntu@$IP" true 2>/dev/null && { log "ssh up"; break; }; sleep 10; done

mkdir -p "$RESULTS_DIR"
# ---- poll (via SSH) for the GATHER_DONE sentinel; pull results while the box is ALIVE ----
log "polling for /root/GATHER_DONE (build+gather ~20-40 min)…"
DONE=0
for i in $(seq 1 90); do
  sleep 30
  if ssh $SSHO "ubuntu@$IP" "sudo test -f /root/GATHER_DONE" 2>/dev/null; then
    log "  [$i] sentinel present — pulling results"
    DONE=1; break
  fi
  STATE=$(aws ec2 describe-instances --region "$REGION" --instance-ids "$INSTANCE_ID" --query 'Reservations[0].Instances[0].State.Name' --output text 2>/dev/null || echo unknown)
  log "  [$i] state=$STATE; waiting for sentinel"
  [ "$STATE" = "terminated" ] && { log "  instance terminated before sentinel — results may be lost"; break; }
done

if [ "$DONE" = 1 ]; then
  ssh $SSHO "ubuntu@$IP" "sudo tar -C /root/sparq/bench -cf - competitor-results 2>/dev/null" | tar -C "$ROOT/bench" -xf - 2>/dev/null \
    && log "pulled result envelopes:" || log "tar pull failed"
  for f in "$RESULTS_DIR"/*.json; do [ -f "$f" ] || continue; cat "$f"; echo; done
else
  log "NO sentinel — pulling /var/log/gather.log for diagnosis"
  ssh $SSHO "ubuntu@$IP" "sudo tail -120 /var/log/gather.log 2>/dev/null" >&2 || true
fi

log "done; cleanup trap will terminate $INSTANCE_ID + delete keypair/SG"
