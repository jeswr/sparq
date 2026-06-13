#!/usr/bin/env bash
# Pull results (or partials) off the box into bench/wikidata-8b/results/.
# Safe to run repeatedly. DRY-RUN by default; EXECUTE=1 to act.
set -uo pipefail
. "$(dirname "$0")/config.sh"
banner

if [ "$DRY" = 1 ]; then IP='<instance-ip>'; else
  IP=$(cat "$STATE/ip") || { echo "no $STATE/ip" >&2; exit 1; }
fi
SSHO="-i $STATE/key -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o ConnectTimeout=15"
STAMP=$(date -u +%Y%m%dT%H%M%SZ)
DEST="$RESULTS/$STAMP"

X "make results dir" mkdir -p "$DEST"
XS "fetch results.tgz (falls back to raw ~/r for partials)" "
  scp $SSHO ubuntu@$IP:results.tgz '$DEST/' 2>/dev/null \
  || scp $SSHO -r ubuntu@$IP:r '$DEST/r-partial'"
XS "verify tarball + show build tail" "
  if [ -f '$DEST/results.tgz' ]; then
    tar -tzf '$DEST/results.tgz' >/dev/null && tar -xzf '$DEST/results.tgz' -C '$DEST' &&
    tail -5 '$DEST/r/build-8B.log' 2>/dev/null; fi; true"

echo
echo "Collected to $DEST. Verify build-8B.log ends with the built-indexes line"
echo "BEFORE terminating (RUNBOOK §7-8). Then: EXECUTE=1 ./terminate.sh"
