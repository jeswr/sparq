#!/usr/bin/env bash
# [OPUS-5] sq-ffaa9 — INSTANCE-SIDE durable results uploader for `purpose=sparq-bench` boxes.
#
# 🤖 SPARQ agent. Runs ON the bench box, backgrounded from user-data by the single line
# `bench_s3_userdata_line` emits (see scripts/bench/bench-s3-results.sh). It streams the
# gather's envelopes + telemetry to S3 while the box is alive, so the operator can read
# the run back AFTER the box has self-terminated — the channel the Nitro serial console
# (garbled/truncated on x86_64 c6i) and SSH (dies on a saturated box) both fail to be.
#
# Living in the repo rather than inline in user-data is deliberate: the launchers render
# user-data through UNQUOTED heredocs, so inline `$` would be eaten by the launcher's
# shell, and the 16384B user-data limit is already tight. The instance has cloned the
# repo by the time this starts, so the whole uploader costs ~150B of user-data.
#
# ENV:
#   S3_URI         (required) s3://<bucket>/<run-id> — no trailing slash
#   UPLOAD_PATHS   comma-separated files/dirs to upload (default: the standard set)
#   SENTINELS      comma-separated files that mean "gather over" (default DONE+FAILED)
#   INTERVAL_S     seconds between ticks (default 60)
#   MAX_S          hard cap; always below the launcher watchdog (default 21600 = 6h)
#   MAX_UPLOAD_MB  skip any single path larger than this (default 64) so a stray corpus
#                  directory can never be shipped to S3 by accident
#
# BEST-EFFORT BY CONSTRUCTION: every step is `|| true`-guarded and the script never
# exits non-zero. It is a telemetry side-car — it must never be able to fail a gather.
#
# ONLY `aws s3 cp` IS USED, NEVER `aws s3 sync`: the instance role is deliberately
# write-only (s3:PutObject + s3:AbortMultipartUpload, object-scoped — see the SECURITY
# POSTURE note in bench-s3-results.sh). `sync` diffs the destination and therefore needs
# ListBucket+GetObject, which that role does not grant; it would fail closed on every
# tick. scripts/tests/test_bench_s3_results.sh pins this.
set -uo pipefail

S3_URI="${S3_URI:-}"
UPLOAD_PATHS="${UPLOAD_PATHS:-/root/sparq/bench/competitor-results,/root/sparq/bench/gather-out,/root/axis-results,/var/log/gather.log,/root/GATHER_STEP}"
SENTINELS="${SENTINELS:-/root/GATHER_DONE,/root/GATHER_FAILED}"
INTERVAL_S="${INTERVAL_S:-60}"
MAX_S="${MAX_S:-21600}"
MAX_UPLOAD_MB="${MAX_UPLOAD_MB:-64}"

log() { printf '[s3-uploader %s] %s\n' "$(date -u +%H:%M:%SZ)" "$*"; }

[ -n "$S3_URI" ] || { log "S3_URI unset — nothing to do"; exit 0; }

# ---- AWS CLI v2, installed only if absent (Ubuntu 24.04 ships none) -------------------
ensure_aws() {
  command -v aws >/dev/null 2>&1 && return 0
  local arch=x86_64
  [ "$(uname -m)" = "aarch64" ] && arch=aarch64
  log "installing AWS CLI v2 ($arch)"
  command -v unzip >/dev/null 2>&1 || { DEBIAN_FRONTEND=noninteractive apt-get install -y -qq unzip >/dev/null 2>&1 || true; }
  curl -fsSL "https://awscli.amazonaws.com/awscli-exe-linux-${arch}.zip" -o /tmp/awscliv2.zip 2>/dev/null || { log "WARN: CLI download failed"; return 1; }
  unzip -q -o /tmp/awscliv2.zip -d /tmp 2>/dev/null || { log "WARN: CLI unzip failed"; return 1; }
  /tmp/aws/install --update >/dev/null 2>&1 || /tmp/aws/install >/dev/null 2>&1 || { log "WARN: CLI install failed"; return 1; }
  export PATH="/usr/local/bin:$PATH"
  command -v aws >/dev/null 2>&1
}

# ---- one upload pass over UPLOAD_PATHS ----------------------------------------------
upload_once() {
  local p sz
  IFS=',' read -r -a _paths <<<"$UPLOAD_PATHS"
  for p in "${_paths[@]}"; do
    [ -n "$p" ] || continue
    [ -e "$p" ] || continue
    sz=$(du -sm "$p" 2>/dev/null | cut -f1)
    if [ -n "${sz:-}" ] && [ "$sz" -gt "$MAX_UPLOAD_MB" ] 2>/dev/null; then
      log "SKIP $p (${sz}MB > ${MAX_UPLOAD_MB}MB cap)"
      continue
    fi
    if [ -d "$p" ]; then
      if aws s3 cp --recursive --only-show-errors "$p" "$S3_URI/$(basename "$p")/" >/dev/null 2>&1
      then log "uploaded dir $p"; else log "WARN: upload failed for dir $p"; fi
    else
      if aws s3 cp --only-show-errors "$p" "$S3_URI/$(basename "$p")" >/dev/null 2>&1
      then log "uploaded file $p"; else log "WARN: upload failed for file $p"; fi
    fi
  done
}

sentinel_present() {
  local s
  IFS=',' read -r -a _sent <<<"$SENTINELS"
  for s in "${_sent[@]}"; do [ -n "$s" ] && [ -f "$s" ] && return 0; done
  return 1
}

ensure_aws || { log "AWS CLI unavailable — durable channel inactive for this run"; exit 0; }

# Provenance object first, so an operator inspecting a half-finished run still learns
# WHICH box/commit produced the partial envelopes sitting next to it.
{
  printf 'instance_id=%s\n' "$(curl -fsS -m 5 http://169.254.169.254/latest/meta-data/instance-id 2>/dev/null || echo unknown)"
  printf 'instance_type=%s\n' "$(curl -fsS -m 5 http://169.254.169.254/latest/meta-data/instance-type 2>/dev/null || echo unknown)"
  printf 'arch=%s\n' "$(uname -m)"
  printf 'started_utc=%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  printf 'commit=%s\n' "$(git -C /root/sparq rev-parse HEAD 2>/dev/null || echo unknown)"
} > /tmp/upload-provenance.txt 2>/dev/null || true
aws s3 cp --only-show-errors /tmp/upload-provenance.txt "$S3_URI/_provenance.txt" >/dev/null 2>&1 || true

log "streaming to $S3_URI every ${INTERVAL_S}s (cap ${MAX_S}s)"
START=$(date +%s)
STATUS=deadline
while :; do
  upload_once
  if sentinel_present; then STATUS=sentinel; break; fi
  [ $(( $(date +%s) - START )) -ge "$MAX_S" ] && break
  sleep "$INTERVAL_S"
done

# Final pass AFTER the sentinel so the last-written envelopes are captured, then a
# completion marker the operator (and bench_s3_fetch) can use to distinguish "the run
# finished and shipped" from "the box died mid-gather".
upload_once
{
  printf 'status=%s\n' "$STATUS"
  printf 'finished_utc=%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  printf 'elapsed_s=%s\n' "$(( $(date +%s) - START ))"
} > /tmp/upload-complete.txt 2>/dev/null || true
aws s3 cp --only-show-errors /tmp/upload-complete.txt "$S3_URI/_UPLOAD_COMPLETE.txt" >/dev/null 2>&1 || true
log "done (status=$STATUS)"
exit 0
