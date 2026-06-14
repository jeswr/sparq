#!/usr/bin/env bash
# [OPUS-4.8] DBPSB/FEASIBLE corpus fetcher — download ONE pinned DBpedia Databus slice,
# verify its sha256, then emit a FIXED, deterministic CI-sized N-Triples cut.
#
#   bench/dbpsb/fetch.sh [triples=750000] [out=/tmp/dbpsb/dbpsb-<triples>.nt]
#
# Why fetch-and-cache (not a generator): DBpedia is REAL data published on the DBpedia
# Databus (databus.dbpedia.org), CC-BY-SA. There is no local generator — the point of DBPSB
# (the "DBpedia SPARQL Benchmark") + FEASIBLE is to run real query-log-shaped queries over
# real DBpedia. We pin ONE official slice by its EXACT download URL + sha256 so the per-commit
# corpus is byte-reproducible without trusting a mutable "latest" alias.
#
# THE PINNED SLICE
#  - URL    : the official `dbpedia` Databus account, mappings group, `mappingbased-objects`
#             artifact, version 2019.09.01, English (`_lang=en`).
#  - Format : despite the `.ttl.bz2` extension the file is plain N-TRIPLES (one full-IRI
#             `s p o .` per line, no @prefix), so sparq loads it with format `ntriples`
#             through the STREAMED parallel parser (no full decompressed copy in RAM) and a
#             deterministic head-cut by LINE is a deterministic cut by TRIPLE.
#  - Content: ~11.8M object-property triples (dbo:birthPlace, deathPlace, country, occupation,
#             genre, team, award, almaMater, spouse, child, starring, …; 563 distinct
#             predicates in the per-commit cut) — exactly the DBPSB/FEASIBLE predicate domain.
#  - Why this artifact: a SINGLE file with rich predicate variety (vs instance-types, which is
#             rdf:type-only) so the curated query subset can exercise BGP / OPTIONAL / UNION /
#             FILTER / DISTINCT / ORDER+LIMIT / aggregates against one pinned download.
#
# THE PER-COMMIT CUT (deterministic)
#   The full artifact (~11.8M triples) is the EC2/nightly tier. For per-commit we take a
#   DETERMINISTIC head cut to $TRIPLES (default 750000) lines. The artifact is grouped by
#   subject (all triples of a subject are contiguous), and its serialization order is fixed by
#   the pinned sha256, so `bzcat | head -n N` yields the SAME N triples every run — reproducible
#   without re-pinning. 750k triples keeps the per-commit corpus hermetic + sub-second to load.
#
# ATTRIBUTION / LICENSE: DBpedia, CC-BY-SA 4.0. See bench/dbpsb/README.md.
#
# HERMETICITY / CI NOTES:
#  - Network: one HTTPS GET of a ~120 MB bzip2 file, pinned by sha256 (SLICE_SHA256). The
#    compressed slice AND the derived cut are cached, so steady-state per-commit runs do NO
#    network (CI keys actions/cache on the URL+sha — see README "CI wiring").
#  - Tools: curl, bzip2 (bzcat), sha256sum, head — all base. No build step, no generator.
#  - Disk discipline (the slice is large): scratch lives under /tmp ($DBPSB_CACHE). We keep
#    the ~120 MB compressed slice (so re-cutting at a different size needs no re-download) and
#    the ~100 MB cut. We DO NOT keep a full decompressed copy (11.8M triples ~1.6 GB) — the cut
#    is produced by a streaming `bzcat | head` so the full text is never materialised on disk.
#    Pass DBPSB_KEEP_SLICE=0 to delete the compressed slice after cutting (saves ~120 MB; the
#    cut alone is enough for the per-commit run). bench/* scratch is gitignored + regenerable.
set -euo pipefail

TRIPLES="${1:-750000}"
CACHE="${DBPSB_CACHE:-/tmp/dbpsb}"
OUT="${2:-$CACHE/dbpsb-$TRIPLES.nt}"

# Pinned: official DBpedia Databus slice — dbpedia/mappings/mappingbased-objects 2019.09.01 en.
# (Discovered via the Databus SPARQL endpoint; the sha256 + byteSize are the published dataid.)
SLICE_URL="https://databus.dbpedia.org/dbpedia/mappings/mappingbased-objects/2019.09.01/mappingbased-objects_lang=en.ttl.bz2"
SLICE_SHA256="1cd51b2f3673196764f356943627e087f949eb24d7f7494a391a7f8154f7ad7b"
SLICE_BYTES="119919822"

SLICE="$CACHE/mappingbased-objects_lang=en.ttl.bz2"
mkdir -p "$CACHE"

# ---- 1. fetch + verify (skipped if the slice is already cached & valid) -------------------
if [ ! -f "$SLICE" ] || ! echo "$SLICE_SHA256  $SLICE" | sha256sum -c --status - 2>/dev/null; then
  echo "[dbpsb] downloading pinned slice (~$((SLICE_BYTES/1000000)) MB): $SLICE_URL" >&2
  # [OPUS-4.8] Robust fetch for CI: the slice is a ~120 MB file behind a 307 redirect to
  # downloads.dbpedia.org. A bare `curl -fsSL` with no retry/timeout flaked on the GitHub
  # runner (one transient error -> the whole DBPSB suite skipped). Retry on any error
  # (incl. transient HTTP/connection failures) with backoff + a generous cap so a blip no
  # longer drops the suite; sha256 verification below still guards integrity.
  curl -fsSL --retry 5 --retry-delay 5 --retry-all-errors \
       --connect-timeout 30 --max-time 600 -o "$SLICE.tmp" "$SLICE_URL"
  echo "$SLICE_SHA256  $SLICE.tmp" | sha256sum -c - >&2
  mv "$SLICE.tmp" "$SLICE"
  echo "[dbpsb] cached + verified: $SLICE" >&2
fi

# ---- 2. emit the deterministic head cut (streaming; never materialise the full decompress) -
if [ ! -s "$OUT" ]; then
  echo "[dbpsb] cutting first $TRIPLES triples -> $OUT (deterministic head of the pinned slice)" >&2
  # `set -o pipefail` is on; bzcat is killed by SIGPIPE once head has its N lines, which is the
  # intended early-stop (no full decompress). Guard the expected 141/SIGPIPE so the script
  # doesn't abort under `set -e`.
  bzcat "$SLICE" 2>/dev/null | head -n "$TRIPLES" > "$OUT.tmp" || {
    rc=$?; [ "$rc" -eq 141 ] || { echo "[dbpsb] cut failed (rc=$rc)" >&2; exit "$rc"; }
  }
  mv "$OUT.tmp" "$OUT"
fi

# ---- 3. optional: reclaim the compressed slice's disk (cut is self-sufficient) ------------
if [ "${DBPSB_KEEP_SLICE:-1}" = "0" ]; then
  rm -f "$SLICE"
  echo "[dbpsb] removed compressed slice (DBPSB_KEEP_SLICE=0); cut retained: $OUT" >&2
fi

echo "$OUT"
