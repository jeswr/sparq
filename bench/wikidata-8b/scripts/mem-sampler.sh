#!/usr/bin/env bash
# Stage-1 mem-sampler pattern + disk-free column.
# Output: <UTC> memused=<MiB> swapused=<MiB> diskfree=<GiB>
# Usage: mem-sampler.sh [interval_s=30]
set -u
INT="${1:-30}"
while true; do
  echo "$(date -u +%FT%TZ) $(free -m | awk '/Mem:/{printf "memused=%s ", $3} /Swap:/{printf "swapused=%s", $3}') diskfree=$(df -BG --output=avail / | tail -1 | tr -dc 0-9)"
  sleep "$INT"
done
