# STATUS — Wikidata 8B full-truthy ingest (stage 2)

Crash protocol: this file is updated at every milestone (prep AND execution). If a
session dies, the next session reads this file first, then `RUNBOOK.md`, then
`state/` (instance id / ip / cost ledger). NEVER re-launch without checking
`state/instance-id` and the AWS console for live `purpose=sparq-hw-validation`
instances first.

## Phase: PREP (zero cloud spend)

- [x] 2026-06-13 stage-1 archive reviewed (`/tmp/sparq-wdbench/`, research/wikidata-lowresource-stage1.md)
- [x] 2026-06-13 dump URL/size verified via `curl -sI` (NO download):
  - `latest-truthy.nt.gz` = 70,654,648,844 B (70.65 GB), Last-Modified 2026-06-12T12:16:23Z, `Accept-Ranges: bytes`
  - `latest-truthy.nt.bz2` = 42,877,885,960 B (rejected: serial decompression, stage-1 lesson)
- [x] 2026-06-13 dict-spill in-flight branch inspected (`../sparq-dictspill` @ 5069b33):
  candidate flags `SPARQ_DICT_SPILL`, `SPARQ_DICT_SPILL_BUDGET_MB`,
  `SPARQ_DICT_SPILL_DISK_FLOOR_MB`; cargo feature `dict-spill` (sparq-core only,
  NOT yet plumbed into sparq-cli's features) — reconcile against merged impl (sq-1q3).
- [x] RUNBOOK.md written (spec, cost model, duration estimates, abort criteria)
- [x] scripts/ written (config/launch/run/collect/terminate/remote-8b/mem-sampler),
  all default to DRY-RUN; `EXECUTE=1` required for any AWS mutation
- [x] dry-run smoke test of launch/terminate/run/collect (no AWS calls made)

## Phase: EXECUTION (NOT STARTED — blocked)

Launch gate (ALL must hold before `EXECUTE=1`):
- [ ] dict-spill merged to public main; `SPARQ_SHA` set to a post-merge commit
- [ ] dict-spill flag/feature placeholders in `scripts/config.sh` + `scripts/remote-8b.sh`
      reconciled against the merged flag names / cli feature plumbing (sq-1q3)
- [ ] budget re-checked: ledger shows ≤ $0.78 spent of $30 cap
- [ ] no live `purpose=sparq-hw-validation` instances; i-090531b4ede8f2d3f untouched

Execution milestones (tick + timestamp as they happen):
- [ ] launch (instance id, ip recorded in state/)
- [ ] download complete (size == 70,654,648,844 or current Content-Length)
- [ ] recompress complete (line count recorded — this IS the triple count)
- [ ] build started (UTC timestamp; abort clock runs from here)
- [ ] build complete (wall, peak RSS, peak swap, index bytes)
- [ ] validation queries pass (COUNT(*) == distinct triples)
- [ ] results collected to bench/wikidata-8b/results/
- [ ] TERMINATED + verified + ledger updated  ← never skip
