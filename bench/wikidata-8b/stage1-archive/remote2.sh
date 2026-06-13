#!/usr/bin/env bash
# Follow-up: 1B build (correct filename), fixed in-memory 100M test.
set -uo pipefail
R="$HOME/r"; CLI="$HOME/sparq/target/release/sparq-cli"
exec > >(tee -a "$R/driver2.log") 2>&1
echo "=== driver2 start $(date -u +%FT%TZ) ==="
rm -f "$HOME/DONE2"

( while true; do echo "$(date -u +%FT%TZ) $(free -m | awk '/Mem:/{printf "memused=%s ", $3} /Swap:/{printf "swapused=%s\n", $3}')"; sleep 30; done ) >> "$R/mem-sampler.log" 2>&1 &
SAMPLER=$!

echo "== BUILD 1B-zst (retry, correct file) == $(date -u +%FT%TZ)"
rm -rf "$HOME/idx"
sync; echo 3 | sudo tee /proc/sys/vm/drop_caches >/dev/null
SW0=$(free -m | awk '/Swap:/{print $3}')
SPARQ_SHARDED_DICT=1 SPARQ_BUILD_TIMING=1 timeout 18000 /usr/bin/time -v \
  "$CLI" build "$HOME/data/1000M.nt.zst" ntriples "$HOME/idx" 32 > "$R/build-1B-zst.log" 2>&1
RC=$?
SW1=$(free -m | awk '/Swap:/{print $3}')
echo "build 1B-zst rc=$RC swap_delta_MiB=$((SW1-SW0))" | tee -a "$R/builds.txt"
grep -E "Elapsed|Maximum resident" "$R/build-1B-zst.log" | tee -a "$R/builds.txt"
du -sb "$HOME/idx" 2>/dev/null | tee -a "$R/builds.txt"
if [ $RC -eq 0 ]; then
  for q in \
    'SELECT (COUNT(*) AS ?c) WHERE { ?s ?p ?o }' \
    'SELECT (COUNT(*) AS ?c) WHERE { ?s <http://www.wikidata.org/prop/direct/P31> ?o }' \
    'SELECT (COUNT(*) AS ?c) WHERE { ?s <http://www.w3.org/2000/01/rdf-schema#label> ?o }'; do
    echo "Q[1B-zst]: $q" >> "$R/queries-1B-zst.log"
    /usr/bin/time -v "$CLI" query-mmap "$HOME/idx" "$q" >> "$R/queries-1B-zst.log" 2>&1
  done
  tail -40 "$R/queries-1B-zst.log"
fi
rm -rf "$HOME/idx"

echo "== IN-MEMORY 100M retry (systemd-run --scope, no --wait) == $(date -u +%FT%TZ)"
sudo systemd-run --scope --collect -p MemoryMax=15G -p MemorySwapMax=0 \
  /usr/bin/time -v "$CLI" query "$HOME/data/100M.nt.zst" ntriples \
  'SELECT (COUNT(*) AS ?c) WHERE { ?s ?p ?o }' > "$R/inmem-100M.log" 2>&1
echo "inmem-100M rc=$?" | tee -a "$R/builds.txt"
tail -25 "$R/inmem-100M.log" | tee -a "$R/builds.txt"

kill $SAMPLER 2>/dev/null
df -h / | tail -1 > "$R/disk-final.txt"
tar -C "$HOME" -czf "$HOME/results.tgz" r
echo "=== driver2 done $(date -u +%FT%TZ) ==="
touch "$HOME/DONE2"
