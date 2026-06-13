# Stage-1 archive (provenance)

Originals from the 2026-06-12 stage-1 run on i-0b3e0be20affc86cf (r8g.large,
eu-west-2), copied out of the supervising machine's volatile /tmp/sparq-wdbench.
Full analysis: research/wikidata-lowresource-stage1.md. Full raw logs:
/tmp/sparq-wdbench/results-final.tgz (not committed — per-build time -v and
query logs; these are the load-bearing subset).

- remote.sh / remote2.sh — the stage-1 on-box drivers (basis for ../scripts/remote-8b.sh)
- builds.txt — per-build rc / wall / RSS / swap-delta / index bytes / query tails
- mem-sampler.log — 15-30 s memused/swapused samples across all builds (the 1B
  20.4 GiB swap peak is here)
- machine.txt — lscpu/free/df/uname capture of the r8g.large
- slices.txt, sha.txt, cli-help.txt, disk-final.txt — slice sizes, engine SHA
  (4aac23af), CLI surface at the time, end-state disk
