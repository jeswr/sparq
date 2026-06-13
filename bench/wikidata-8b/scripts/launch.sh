#!/usr/bin/env bash
# Launch the 8B-run instance: ephemeral keypair + SG (22/tcp from MYIP only),
# r8g.large + gp3 2200GB, dead-man shutdown, all tagged purpose=sparq-hw-validation.
# Adapted from stage-1 (/tmp/sparq-wdbench archive) and hwrun/launch.sh.
# DRY-RUN by default; EXECUTE=1 to launch for real. ZERO retries up the size
# ladder on purpose — the low-resource spec IS the experiment.
set -uo pipefail
. "$(dirname "$0")/config.sh"
banner

# --- launch gate (RUNBOOK §0) ---
if [ -z "$SPARQ_SHA" ]; then
  echo "REFUSING: SPARQ_SHA is empty. Set it to a public-main commit that includes the dict-spill merge." >&2
  [ "$DRY" = 1 ] || exit 1
  SPARQ_SHA="<UNSET — launch would refuse>"
fi
if [ -f "$STATE/instance-id" ]; then
  echo "REFUSING: $STATE/instance-id exists ($(cat "$STATE/instance-id")). Terminate + clean state first." >&2
  [ "$DRY" = 1 ] || exit 1
fi

KEY_NAME="sparq-wd8b-$$-${RANDOM}"
echo "$KEY_NAME" > "$STATE/key-name.dryrun"   # real name persisted below on EXECUTE

X "generate ephemeral ed25519 keypair" \
  ssh-keygen -t ed25519 -N '' -f "$STATE/key" -q

X "resolve latest Ubuntu 24.04 arm64 AMI (read-only)" \
  aws ec2 describe-images --region "$REGION" --owners 099720109477 \
    --filters "Name=name,Values=ubuntu/images/hvm-ssd-gp3/ubuntu-noble-24.04-arm64-server-*" "Name=state,Values=available" \
    --query 'sort_by(Images,&CreationDate)[-1].ImageId' --output text
if [ "$DRY" = 1 ]; then AMI='<resolved-ami>'; else
  AMI=$(aws ec2 describe-images --region "$REGION" --owners 099720109477 \
    --filters "Name=name,Values=ubuntu/images/hvm-ssd-gp3/ubuntu-noble-24.04-arm64-server-*" "Name=state,Values=available" \
    --query 'sort_by(Images,&CreationDate)[-1].ImageId' --output text)
fi

if [ "$DRY" = 1 ]; then VPC='<default-vpc>'; SUBNET='<default-subnet>'; MYIP='<my-ip>'; else
  VPC=$(aws ec2 describe-vpcs --region "$REGION" --filters Name=isDefault,Values=true --query 'Vpcs[0].VpcId' --output text)
  SUBNET=$(aws ec2 describe-subnets --region "$REGION" --filters Name=vpc-id,Values="$VPC" "Name=default-for-az,Values=true" \
    --query 'Subnets[0].SubnetId' --output text)
  MYIP=$(curl -s https://checkip.amazonaws.com)
fi
echo "AMI=$AMI VPC=$VPC SUBNET=$SUBNET MYIP=$MYIP KEY=$KEY_NAME"

X "import keypair (tagged)" \
  aws ec2 import-key-pair --region "$REGION" --key-name "$KEY_NAME" \
    --public-key-material "fileb://$STATE/key.pub" \
    --tag-specifications "ResourceType=key-pair,Tags=[{Key=purpose,Value=$TAG_PURPOSE}]"

XS "create SG + allow 22 from MYIP/32 only" "
  SG_ID=\$(aws ec2 create-security-group --region '$REGION' --group-name '$KEY_NAME' \
    --description 'sparq wd8b (ephemeral)' --vpc-id '$VPC' \
    --tag-specifications 'ResourceType=security-group,Tags=[{Key=purpose,Value=$TAG_PURPOSE}]' \
    --query GroupId --output text) &&
  aws ec2 authorize-security-group-ingress --region '$REGION' --group-id \"\$SG_ID\" \
    --protocol tcp --port 22 --cidr \"$MYIP/32\" >/dev/null &&
  echo \"\$SG_ID\" > '$STATE/sg-id'"

# dead-man: power off after DEADMAN_MIN no matter what; shutdown-behavior=terminate
UDATA="$STATE/udata.txt"
printf '#!/bin/bash\nshutdown -P +%s\n' "$DEADMAN_MIN" > "$UDATA"

XS "run-instances $INSTANCE_TYPE + gp3 ${VOL_GB}GB/${VOL_IOPS}iops/${VOL_THROUGHPUT}MBps (dead-man +${DEADMAN_MIN}m)" "
  IID=\$(aws ec2 run-instances --region '$REGION' --image-id '$AMI' --instance-type '$INSTANCE_TYPE' \
    --key-name '$KEY_NAME' --security-group-ids \"\$(cat '$STATE/sg-id')\" --subnet-id '$SUBNET' \
    --associate-public-ip-address \
    --user-data 'file://$UDATA' \
    --instance-initiated-shutdown-behavior terminate \
    --block-device-mappings '[{\"DeviceName\":\"/dev/sda1\",\"Ebs\":{\"VolumeSize\":$VOL_GB,\"VolumeType\":\"gp3\",\"Iops\":$VOL_IOPS,\"Throughput\":$VOL_THROUGHPUT,\"DeleteOnTermination\":true}}]' \
    --tag-specifications 'ResourceType=instance,Tags=[{Key=purpose,Value=$TAG_PURPOSE},{Key=Name,Value=sparq-wd8b}]' \
    --query 'Instances[0].InstanceId' --output text) &&
  echo \"\$IID\" > '$STATE/instance-id' &&
  echo '$KEY_NAME' > '$STATE/key-name' &&
  date -u +%s > '$STATE/launch-epoch' && date -u +%FT%TZ > '$STATE/launch-time'"

XS "wait running + record public ip" "
  IID=\$(cat '$STATE/instance-id') &&
  aws ec2 wait instance-running --region '$REGION' --instance-ids \"\$IID\" &&
  aws ec2 describe-instances --region '$REGION' --instance-ids \"\$IID\" \
    --query 'Reservations[0].Instances[0].PublicIpAddress' --output text > '$STATE/ip' &&
  echo \"instance \$IID ip \$(cat '$STATE/ip')\""

XS "wait for sshd (up to ~7 min)" "
  IP=\$(cat '$STATE/ip'); for i in \$(seq 1 40); do
    ssh -i '$STATE/key' -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o ConnectTimeout=15 \
      ubuntu@\"\$IP\" true 2>/dev/null && { echo ssh up; exit 0; }; sleep 10; done; echo 'ssh never came up' >&2; exit 1"

echo
echo "Launched (or would launch) $INSTANCE_TYPE / gp3 ${VOL_GB}GB. Burn: \$$HOURLY_BURN_USD/h."
echo "Next: EXECUTE=1 ./run.sh   (then collect.sh, then terminate.sh — ALWAYS terminate)"
