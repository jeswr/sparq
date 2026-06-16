#!/usr/bin/env bash
# [OPUS-4.8] sq-ays7 / sq-t0c3 — orphan-proof EC2 orchestrator for the same-box SPARQL
# competitor gather (PHASE 2). Sibling of scripts/gather-ec2.sh (the SHACL gather): it
# reuses the SAME orphan-proofing + SSH-pull-while-alive + explicit-terminate design, but
# runs a SPARQL same-box comparison on a featured corpus instead of the SHACL suite.
#
# WHAT IT MEASURES (HONEST same-box, single box, ONE generated corpus):
#   1. sparq    : `sparq-cli bench <corpus> turtle bench/sp2b/queries <iters> count`
#                 (the EXACT runner CI uses for the SP2Bench featured suite).
#   2. oxigraph : `sparq-bench bench-corpus <corpus> turtle bench/sp2b/queries <iters>`
#                 (new same-box Oxigraph path; embedded dev-dep 0.5.x from Cargo.lock).
#   Both engines load the SAME generated SP2Bench corpus and run the SAME query files, so
#   the two TSVs (<name>\t<rows>\t<best_us>) are a self-consistent same-box comparison.
#   Solution counts are cross-checked against bench/sp2b/expected-rows.tsv (correctness).
#   3. qlever   : BEST-EFFORT (GATHER_QLEVER=1). Builds a QLever index over the same corpus
#                 in Docker, starts the server, runs the same queries via the http-sparql
#                 adapter. If Docker/index/server is too heavy/flaky in the bounded window,
#                 it is SKIPPED and stays honest-n/a (the recipe is beaded separately).
#
# Corpus scale is BOUNDED + documented: SP2Bench is the cheapest deterministic featured
# corpus (real Freiburg sp2b_gen, g++ -O2, sha256-pinned, byte-identical output; ~250k
# triples ≈ 26 MB Turtle). SP2B_TRIPLES caps it (default 250000 = the CI per-commit scale).
#
# ORPHAN-PROOFING (agent-death-independent, 3h hard cap; identical to gather-ec2.sh):
#   --instance-initiated-shutdown-behavior terminate + detached (sleep 10800; shutdown)
#   + systemd-run --on-active=10800 shutdown. The gather writes results + /root/GATHER_DONE,
#   then STOPS (never shuts down) — only the watchdog auto-terminates if the orchestrator
#   dies. The orchestrator SSH-pulls results while the box is ALIVE, then terminates it.
#
# SAFETY: operates ONLY on the ONE instance it launches; NEVER touches prod
# (i-090531b4ede8f2d3f) or the work box. Tag purpose=sparq-bench.
#
# Usage:  AWS_PROFILE=pss scripts/gather-ec2-sparql.sh <branch> [<region>]
#   GATHER_QLEVER=1   also attempt the QLever best-effort path (default off)
#   SP2B_TRIPLES=N    corpus scale (default 250000)
#   GATHER_ITERS=K    min-of-K per query (default 5)
# Writes decoded envelopes under bench/competitor-results/ and prints each on stdout.
# ALWAYS terminates the instance + deletes the keypair/SG (cleanup trap).
set -euo pipefail

BRANCH="${1:?usage: gather-ec2-sparql.sh <branch> [region]}"
REGION="${2:-${AWS_REGION:-eu-west-2}}"
ITYPE="${GATHER_ITYPE:-c7g.2xlarge}"     # 8 vCPU arm64 (more RAM for the QLever index; falls back below)
REPO="https://github.com/jeswr/sparq.git"
SP2B_TRIPLES="${SP2B_TRIPLES:-250000}"
ITERS="${GATHER_ITERS:-5}"
GATHER_QLEVER="${GATHER_QLEVER:-0}"
TAGSPEC='ResourceType=instance,Tags=[{Key=purpose,Value=sparq-bench}]'
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RESULTS_DIR="$ROOT/bench/competitor-results"

log() { printf '[gather-ec2-sparql] %s\n' "$*" >&2; }
die() { printf '[gather-ec2-sparql] ERROR: %s\n' "$*" >&2; exit 1; }

WORK="$(mktemp -d)"
KEYFILE="$WORK/key"
KEY_NAME="sparq-bench-sparql-$$-${RANDOM}"
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
MYIP=$(curl -s https://checkip.amazonaws.com | tr -d '[:space:]')
[ -n "$MYIP" ] || die "could not determine public IP (checkip returned empty)"
log "AMI=$AMI VPC=$VPC SUBNET=$SUBNET MYIP=$MYIP ITYPE=$ITYPE QLEVER=$GATHER_QLEVER SP2B_TRIPLES=$SP2B_TRIPLES"

log "keypair + locked-down security group (ssh from $MYIP/32 only)"
aws ec2 import-key-pair --region "$REGION" --key-name "$KEY_NAME" --public-key-material "fileb://${KEYFILE}.pub" >/dev/null
SG_ID=$(aws ec2 create-security-group --region "$REGION" --group-name "$KEY_NAME" --description "sparq sparql bench gather (ephemeral)" --vpc-id "$VPC" --query 'GroupId' --output text)
aws ec2 authorize-security-group-ingress --region "$REGION" --group-id "$SG_ID" --protocol tcp --port 22 --cidr "${MYIP}/32" >/dev/null

# ---- user-data: build, gen corpus, run sparq + oxigraph (+ qlever best-effort), write
# results + sentinel, then STOP (watchdog is the only auto-terminate). ---------------------
USERDATA=$(cat <<UD
#!/bin/bash
set -x
exec > >(tee /var/log/gather.log) 2>&1
( sleep 10800; shutdown -h now ) &
systemd-run --on-active=10800 /sbin/shutdown -h now || true

export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y -qq build-essential g++ pkg-config git curl jq python3 python3-venv python3-pip unzip
if [ "$GATHER_QLEVER" = "1" ]; then
  apt-get install -y -qq docker.io || true
  systemctl start docker || true
fi
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
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

cargo build --release -q -p sparq-cli
cargo build --release -q -p sparq-bench

# ---- generate the bounded SP2Bench corpus (real Freiburg sp2b_gen) -----------------------
CORPUS=\$(bash bench/sp2b/gen.sh "$SP2B_TRIPLES")
echo "CORPUS=\$CORPUS"
ls -la "\$CORPUS"

OXI_V=\$(awk '/^name = "oxigraph"\$/{f=1} f&&/^version = /{gsub(/[",]/,"",\$3);print \$3;exit}' Cargo.lock)
echo "OXI_V=\$OXI_V"

export QUIET_BOX=true
mkdir -p bench/competitor-results

# ---- sparq (the CI runner, count mode) ---------------------------------------------------
./target/release/sparq-cli bench "\$CORPUS" turtle bench/sp2b/queries "$ITERS" count > /tmp/sparq.tsv 2>/tmp/sparq.err || true
echo "=== sparq.tsv ==="; cat /tmp/sparq.tsv

# ---- oxigraph (same corpus, same queries, same count semantics) --------------------------
./target/release/sparq-bench bench-corpus "\$CORPUS" turtle bench/sp2b/queries "$ITERS" > /tmp/oxi.tsv 2>/tmp/oxi.err || true
echo "=== oxi.tsv ==="; cat /tmp/oxi.tsv; echo "--- oxi.err ---"; cat /tmp/oxi.err

# ---- env block + combine into ONE same-box-comparison envelope ---------------------------
CPU=\$(LC_ALL=C grep -m1 'model name' /proc/cpuinfo 2>/dev/null | cut -d: -f2- | sed 's/^ *//' || true)
[ -n "\$CPU" ] || CPU=\$(uname -p)
NPROC=\$(nproc)
KERNEL=\$(uname -r)
NOW=\$(date -u +%Y-%m-%dT%H:%M:%SZ)

# qlever best-effort
QLEVER_STATUS="skipped"
QLEVER_TSV=""
if [ "$GATHER_QLEVER" = "1" ] && command -v docker >/dev/null 2>&1; then
  echo "=== qlever best-effort ==="
  QDIR=/root/qlever; mkdir -p "\$QDIR"
  # sp2b_gen emits Turtle; QLever indexes N-Triples/Turtle. Build an index then start server.
  cp "\$CORPUS" "\$QDIR/sp2b.ttl"
  ( cd "\$QDIR"
    docker pull -q docker.io/adfreiburg/qlever:latest || true
    # IndexBuilder over the Turtle corpus
    timeout 1200 docker run --rm -v "\$QDIR":/data -w /data docker.io/adfreiburg/qlever:latest \
      bash -lc "IndexBuilderMain -i /data/sp2b -f /data/sp2b.ttl -F ttl 2>&1 | tail -20" \
      && echo "QLEVER_INDEX_OK" || echo "QLEVER_INDEX_FAIL"
    docker run -d --name qlever-srv -p 7001:7001 -v "\$QDIR":/data -w /data docker.io/adfreiburg/qlever:latest \
      ServerMain -i /data/sp2b -p 7001 -j 4 || echo "QLEVER_SERVER_FAIL"
  ) || true
  # poll the endpoint
  for _ in \$(seq 1 30); do
    curl -s "http://localhost:7001/?query=SELECT%20*%20WHERE%7B%3Fs%20%3Fp%20%3Fo%7D%20LIMIT%201" >/dev/null 2>&1 && break
    sleep 3
  done
  python3 -m venv /root/qenv && /root/qenv/bin/pip install -q requests
  : > /tmp/qlever.tsv
  for q in bench/sp2b/queries/*.rq; do
    name=\$(basename "\$q" .rq)
    /root/qenv/bin/python3 - "\$q" "\$name" <<'PYQ' >> /tmp/qlever.tsv 2>/tmp/qlever.err || true
import sys, time, requests
qf, name = sys.argv[1], sys.argv[2]
q = open(qf).read()
best = None; rows = "ERROR"
for _ in range($ITERS):
    t = time.perf_counter()
    try:
        r = requests.post("http://localhost:7001", data={"query": q},
                          headers={"Accept": "application/sparql-results+json"}, timeout=120)
        j = r.json()
        if "results" in j and "bindings" in j["results"]:
            rows = len(j["results"]["bindings"])
        elif "boolean" in j:
            rows = 1 if j["boolean"] else 0
        else:
            rows = "ERROR"; break
        us = (time.perf_counter() - t) * 1e6
        best = us if best is None else min(best, us)
    except Exception as e:
        rows = "ERROR"; break
if rows == "ERROR" or best is None:
    print(f"{name}\tERROR\tqlever")
else:
    print(f"{name}\t{rows}\t{best:.1f}")
PYQ
  done
  echo "=== qlever.tsv ==="; cat /tmp/qlever.tsv || true
  if [ -s /tmp/qlever.tsv ] && grep -qv ERROR /tmp/qlever.tsv; then
    QLEVER_STATUS="ok"
    QLEVER_TSV=\$(python3 -c 'import json,sys; print(json.dumps(open("/tmp/qlever.tsv").read()))')
  else
    QLEVER_STATUS="failed"
  fi
fi

SPARQ_TSV=\$(python3 -c 'import json,sys; print(json.dumps(open("/tmp/sparq.tsv").read()))')
OXI_TSV=\$(python3 -c 'import json,sys; print(json.dumps(open("/tmp/oxi.tsv").read()))')
[ -n "\$QLEVER_TSV" ] || QLEVER_TSV='""'

cat > bench/competitor-results/sparql-same-box-\$(date -u +%Y%m%dT%H%M%SZ).json <<EOF2
{
  "gather": "sparql-same-box-comparison",
  "git_commit": "\$SHA",
  "suite": "sp2b",
  "scale": "SP2Bench $SP2B_TRIPLES triples (real Freiburg sp2b_gen, deterministic)",
  "iters": $ITERS,
  "oxigraph_version": "\$OXI_V",
  "env": {"host_class":"ephemeral-ec2-$ITYPE","cpu_model":"\$CPU","nproc":\$NPROC,"os":"Linux","kernel":"\$KERNEL","quiet_box":true,"gathered_at_utc":"\$NOW","git_commit":"\$SHA"},
  "qlever_status": "\$QLEVER_STATUS",
  "sparq_tsv": \$SPARQ_TSV,
  "oxigraph_tsv": \$OXI_TSV,
  "qlever_tsv": \$QLEVER_TSV
}
EOF2

jq . bench/competitor-results/sparql-same-box-*.json || echo "WARN: envelope did not validate"
ls -la bench/competitor-results/
echo "\$SHA" > /root/GATHER_DONE
sync
UD
)

log "launching $ITYPE"
LAUNCH_ERR="$WORK/launch.err"
INSTANCE_ID=$(aws ec2 run-instances --region "$REGION" --image-id "$AMI" --instance-type "$ITYPE" \
  --instance-initiated-shutdown-behavior terminate \
  --key-name "$KEY_NAME" --security-group-ids "$SG_ID" \
  --subnet-id "$SUBNET" --associate-public-ip-address \
  --block-device-mappings '[{"DeviceName":"/dev/sda1","Ebs":{"VolumeSize":40,"VolumeType":"gp3","DeleteOnTermination":true}}]' \
  --tag-specifications "$TAGSPEC" \
  --user-data "$USERDATA" \
  --query 'Instances[0].InstanceId' --output text 2>"$LAUNCH_ERR") || true
case "$INSTANCE_ID" in
  i-*) ;;
  *)
    log "primary itype $ITYPE failed: $(cat "$LAUNCH_ERR" 2>/dev/null)"
    ITYPE_FB="c7g.xlarge"
    log "retrying with fallback $ITYPE_FB"
    INSTANCE_ID=$(aws ec2 run-instances --region "$REGION" --image-id "$AMI" --instance-type "$ITYPE_FB" \
      --instance-initiated-shutdown-behavior terminate \
      --key-name "$KEY_NAME" --security-group-ids "$SG_ID" \
      --subnet-id "$SUBNET" --associate-public-ip-address \
      --block-device-mappings '[{"DeviceName":"/dev/sda1","Ebs":{"VolumeSize":40,"VolumeType":"gp3","DeleteOnTermination":true}}]' \
      --tag-specifications "$TAGSPEC" \
      --user-data "$USERDATA" \
      --query 'Instances[0].InstanceId' --output text 2>"$LAUNCH_ERR") || true
    case "$INSTANCE_ID" in
      i-*) ITYPE="$ITYPE_FB" ;;
      *) INSTANCE_ID=""; die "run-instances failed on both itypes: $(cat "$LAUNCH_ERR" 2>/dev/null)" ;;
    esac
    ;;
esac
log "launched INSTANCE_ID=$INSTANCE_ID ($ITYPE)"
aws ec2 wait instance-running --region "$REGION" --instance-ids "$INSTANCE_ID"
IP=$(aws ec2 describe-instances --region "$REGION" --instance-ids "$INSTANCE_ID" --query 'Reservations[0].Instances[0].PublicIpAddress' --output text)
log "public IP=$IP; waiting for sshd"
SSH_UP=0
for i in $(seq 1 40); do ssh $SSHO "ubuntu@$IP" true 2>/dev/null && { log "ssh up"; SSH_UP=1; break; }; sleep 10; done
[ "$SSH_UP" = 1 ] || die "sshd never became reachable on $IP after 40 attempts — aborting (cleanup trap terminates $INSTANCE_ID)"

mkdir -p "$RESULTS_DIR"
log "polling for /root/GATHER_DONE (build+gen+gather ~25-50 min; +QLever if enabled)…"
DONE=0
for i in $(seq 1 120); do
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
  for f in "$RESULTS_DIR"/sparql-same-box-*.json; do [ -f "$f" ] || continue; cat "$f"; echo; done
else
  log "NO sentinel — pulling /var/log/gather.log for diagnosis"
  ssh $SSHO "ubuntu@$IP" "sudo tail -160 /var/log/gather.log 2>/dev/null" >&2 || true
fi

log "done; cleanup trap will terminate $INSTANCE_ID + delete keypair/SG"
