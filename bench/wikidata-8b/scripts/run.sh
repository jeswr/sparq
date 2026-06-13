#!/usr/bin/env bash
# Ship remote-8b.sh + mem-sampler.sh to the launched box, start the driver under
# nohup, then poll (restartable: re-running just re-attaches to the poll loop).
# DRY-RUN by default; EXECUTE=1 to act.
set -uo pipefail
. "$(dirname "$0")/config.sh"
banner

if [ "$DRY" = 1 ]; then IP='<instance-ip>'; else
  IP=$(cat "$STATE/ip") || { echo "no $STATE/ip — run launch.sh first" >&2; exit 1; }
fi
SSHO="-i $STATE/key -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o ConnectTimeout=15"
[ -n "$SPARQ_SHA" ] || { echo "REFUSING: SPARQ_SHA unset (dict-spill merge gate, RUNBOOK §0)" >&2; [ "$DRY" = 1 ] || exit 1; SPARQ_SHA='<UNSET>'; }

X "scp driver + sampler" \
  scp $SSHO "$SCRIPTS_DIR/remote-8b.sh" "$SCRIPTS_DIR/mem-sampler.sh" "ubuntu@$IP:"

# [OPUS-4.8] roborev 2013: the idempotency check must NOT use `pgrep -f
# remote-8b.sh`. This whole inline command runs in a remote shell whose own
# command text contains `remote-8b.sh` (the nohup line below), so pgrep -f would
# match the wrapper shell itself and always report already-running, so the
# driver would never start. Bracketing the pattern doesn't help here either —
# the wrapper's command line still contains a literal `bash remote-8b.sh`
# substring. Use a PID file written from the backgrounded job's own `$!` and a
# `kill -0` liveness check, which is unambiguous and cannot match the wrapper.
XS "start driver under nohup (idempotent: skips if already running/done)" "
  ssh $SSHO ubuntu@$IP '
    if [ -f DONE ]; then echo already-done; exit 0; fi
    if [ -f driver.pid ] && kill -0 \"\$(cat driver.pid)\" 2>/dev/null; then echo already-running; exit 0; fi
    nohup bash remote-8b.sh \"$SPARQ_SHA\" \"$CHUNK_MILLIONS\" \"$DEADMAN_MIN\" \
      \"$BUILD_TIMEOUT_S\" \"$SWAP_ABORT_MIB\" \"$DISK_ABORT_FREE_GB\" \"$PARSE_CHECKPOINT_S\" \
      \"$DUMP_URL\" \"$DL_WORKERS\" \"$DICT_SPILL_ENV\" \"$CARGO_FEATURES\" \
      >/dev/null 2>&1 & echo \$! > driver.pid; echo started'"

# Poll loop: every 60 s print elapsed, last step marker, last sampler line, abort flag.
# Supervisor-side budget stop at 30 h (RUNBOOK §9) — warns loudly, never auto-extends.
if [ "$DRY" = 1 ]; then
  echo "DRY  [poll loop] every 60s: ssh tail driver.log marker + mem-sampler tail; exit when DONE; warn at 30h elapsed (\$12.78)"
  exit 0
fi
T0=$(cat "$STATE/launch-epoch" 2>/dev/null || date -u +%s)
while true; do
  sleep 60
  NOW=$(date -u +%s); EL_H=$(( (NOW - T0) / 3600 ))
  if ! ssh $SSHO "ubuntu@$IP" true 2>/dev/null; then
    echo "$(date -u +%FT%TZ) box unreachable (dead-man fired? spot-style surprise?) — run collect.sh if possible, then terminate.sh"
    exit 2
  fi
  MARK=$(ssh $SSHO "ubuntu@$IP" "grep '^== ' r/driver.log 2>/dev/null | tail -1; tail -1 r/mem-sampler.log 2>/dev/null; cat r/abort.txt 2>/dev/null" 2>/dev/null)
  echo "$(date -u +%FT%TZ) elapsed=${EL_H}h (\$$(echo "$EL_H * $HOURLY_BURN_USD" | bc 2>/dev/null || echo '?')) | $MARK"
  if echo "$MARK" | grep -q '^ABORT:'; then
    echo "DRIVER ABORTED — collect.sh then terminate.sh NOW"; exit 3
  fi
  if ssh $SSHO "ubuntu@$IP" "test -f DONE" 2>/dev/null; then
    echo "driver DONE — run collect.sh then terminate.sh"; exit 0
  fi
  if [ "$EL_H" -ge 30 ]; then
    echo "*** 30h SUPERVISOR BUDGET STOP (RUNBOOK §9): unless build is in final phase, collect partials + terminate NOW ***"
  fi
done
