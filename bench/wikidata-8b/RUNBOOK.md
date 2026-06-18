# Wikidata full-truthy (~8–9.4 B triples) low-resource ingest — stage 2 runbook

Goal: build sparq's external-memory on-disk indexes for the FULL `latest-truthy`
dump on a deliberately small Graviton box (r8g.large, 2 vCPU / 16 GB — same class
as stage 1), as the flagship "Wikidata on a 16 GB machine" claim. Stage-1 evidence
(`research/wikidata-lowresource-stage1.md`, runs of 2026-06-12 on
i-0b3e0be20affc86cf) showed the triple-sorting side is bounded but the
RAM-resident dictionary is not: 1 B triples already needed a 35.0 GiB total
footprint via swap, and the dict extrapolates to ~280–330 GiB at full truthy.

## 0. HARD LAUNCH GATE — do not skip

**This run is infeasible without dictionary spill-to-disk.** Without it, the dict
alone is ~18–21× this box's RAM and the build would be effectively all-swap
(stage-1 measured 4.8× per-triple slowdown at just 2.2× RAM). dict-spill is now
**MERGED** to public main (`crates/sparq-core/src/dictspill.rs`; PRs #29/#582/#670).
Before launching:

1. Set `SPARQ_SHA` in `scripts/config.sh` to a public-main commit at or after the
   dict-spill merge (the module + the sparq-cli default feature must be present).
2. dict-spill flags/feature RECONCILED against the merged impl (sq-1q3):
   - env gate (`SpillConfig::from_lookup`): `SPARQ_DICT_SPILL=1|on|auto`
     (`0`/`off`/unset keeps the in-RAM consolidation), `SPARQ_DICT_SPILL_BUDGET_MB`,
     `SPARQ_DICT_SPILL_DISK_FLOOR_MB` (default floor 1024). The BUDGET governs the
     dedup caches + external-sort run buffers ONLY — it does NOT count the
     `chunk`×12 B triple-run buffer or the ~0.5–1 GiB parse pipeline, so real peak
     RSS ≈ budget + that floor (8192 MB ⇒ ~9–10 GiB on the 15 GiB box).
   - cargo: feature `dict-spill` is now a **DEFAULT** feature of sparq-cli
     (`crates/sparq-cli/Cargo.toml`: `default = ["mmap","mimalloc","dict-spill"]`,
     forwarding `sparq-core/dict-spill`), so the build command is simply
     `cargo build --release -p sparq-cli` — no `--features` needed. (`CARGO_FEATURES`
     in `config.sh` is therefore empty by default; the old
     `--features sparq-core/dict-spill` was redundant and reached into the sub-crate.)
3. Budget ledger check: ≤ $0.78 of the $30 cap spent; no live
   `purpose=sparq-hw-validation` instances; **NEVER touch i-090531b4ede8f2d3f**.
4. Re-verify the dump headers (`curl -sI`, §3) — the dump rolls weekly and the
   size feeds the disk/duration math.

If dict-spill does NOT merge: the honest largest scale that fits this box is the
already-measured 1 B (stage 1). The fallback that preserves the 8 B claim is a
≥384 GB-RAM machine (e.g. r8g.12xlarge, $1.66/h-class) — a different claim and,
at ~5–14 h, a $10–25 instance cost on its own. Do not attempt 8 B on 16 GB
without spill; stage-1 numbers prove it would burn the budget for nothing.

## 1. Instance + storage spec

| item | value | why |
|---|---|---|
| instance | **r8g.large** on-demand, eu-west-2, profile `pss` | same 2 vCPU Graviton4 / 16 GB box as stage 1 — the low-resource claim IS the point; $0.13838/h |
| AMI | latest `ubuntu/images/hvm-ssd-gp3/ubuntu-noble-24.04-arm64-server-*` (owner 099720109477), resolved at launch | stage 1 resolved to ami-03018a249a89a9ec1 |
| EBS root | **gp3 2,200 GB, 4,000 IOPS, 125 MB/s**, DeleteOnTermination=true | disk budget below; >125 MB/s provisioned throughput is wasted money — r8g.large *sustains* only 78.75 MB/s to EBS (burst 1,250 MB/s ≤30 min) |
| swap | 64 GiB swapfile (script-created) | safety net only; with dict-spill working, sampler should show swap ≲ 2 GiB. Sustained >24 GiB ⇒ spill is broken ⇒ abort |
| tags | `purpose=sparq-hw-validation` on instance, SG, keypair | account hygiene + terminate-guard match |
| network | ephemeral SG, tcp/22 from `$(curl checkip)/32` only; ephemeral ed25519 keypair | copied from stage-1 / `hwrun/launch.sh` pattern |
| dead-man | user-data `shutdown -P +2400` (40 h) + `--instance-initiated-shutdown-behavior terminate`, re-armed in remote driver | absolute cost ceiling $17.04 even if the supervisor dies |

### Disk budget (why 2,200 GB)

Linear from stage-1 measurements (89.6 B/triple on-disk index; 1 B build wrote
173.5 GB total for an 89.5 GB index ⇒ intermediates ≤ 0.94× index — upper bound,
flagged estimate):

| item | 8.0 B | 9.4 B | basis |
|---|--:|--:|---|
| final index | 717 GB | 842 GB | 89.6 B/triple (measured @1B) |
| dump .gz (deleted after recompress) | 71 GB | 71 GB | verified Content-Length |
| recompressed .zst | 56 GB | 66 GB | ~7.0 GB/B-triples (measured @1B: 6.99 GB) |
| extsort temp runs (peak, upper bound) | 675 GB | 790 GB | ≤0.94× index (1B write-amplification) |
| dict spill transient (staged + remap) | ~210 GB | ~250 GB | staged `[u64;3]` triples 24 B/triple (192/226 GB) + per-shard `seq→id` remap files (~8 B/distinct-term); deleted at remap end. NOT 69 B/term — that was the in-RAM dict @100M which spill REMOVES (sq-1q3) |
| dict record files + sort runs (transient) | ~150 GB | ~180 GB | serialized distinct-term bytes + dedup/assign external-sort runs; ≈ on-disk dict (~38.4 B/distinct-term measured @100M, research/external-dictionary.md), deleted as consumed. Partly overlaps the final dict already in "final index" |
| swapfile | 64 GB | 64 GB | fixed |
| OS + rust + sparq build | 15 GB | 15 GB | stage-1 observed |
| **peak total (remap phase)** | **~1,800 GB** | **~2,150 GB** | gz deleted before build; see phase note below — the two dict transient rows do NOT both co-peak |

The dict-spill transients are PHASE-DISJOINT from each other, so the rows do **not**
all sum at once. The heaviest moment is the **remap phase**: final index (growing) +
extsort temp runs + staged `[u64;3]` triples + .zst input + swap + OS. The dict
**record files + dedup/assign sort runs** are freed BEFORE remap starts feeding
extsort, so they peak earlier and separately (their phase total stays well under the
remap peak). Peak total above is the remap co-peak: final index + extsort runs +
staged triples + .zst + swap + OS ≈ 717 + 675 + 192 + 56 + 64 + 15 = ~1,720 GB @8B
(round up to ~1,800 for transient-file slack), ~2,150 GB @9.4 B. The staged-triple file is
the one dict-spill transient that overlaps the extsort-runs peak; it is the +24 B/triple
the OLD budget's 69-B/term row mis-estimated.

2,200 GB leaves ~2–18 % headroom (tighter at 9.4 B than the pre-reconcile estimate —
the staged triples are real co-peak bytes). The remote driver samples `df` every 30 s
and kills the build if free space drops below 50 GB (collect partials, terminate). If
the 9.4-B case runs hot, raise `VOL_GB` to 2,400 (cost: +$0.025/h).

## 2. Cost model

Rates: AWS price list bulk API, retrieved 2026-06-12 (stage-1 ledger), eu-west-2:
r8g.large $0.13838/h; gp3 $0.0928/GB-mo; IOPS >3,000 $0.0058/IOPS-mo;
throughput >125 MB/s $0.0464/MBps-mo (not purchased). 730 h/mo.

| component | rate |
|---|--:|
| r8g.large on-demand | $0.13838/h |
| gp3 2,200 GB | $0.27967/h |
| gp3 +1,000 IOPS | $0.00795/h |
| **total burn** | **$0.42600/h = $10.22/day** |

| scenario | wall | cost |
|---|--:|--:|
| best case (fast download, build @0.58 M/s) | ~8 h | ~$3.4 |
| expected | 12–16 h | $5.1–6.8 |
| worst single attempt (slow download, build @0.19 M/s, 9.4 B) | ~20 h | ~$8.5 |
| + one full build retry | ≤34 h | ≤$14.5 |
| **hard ceiling (40 h dead-man)** | 40 h | **$17.04** |

Project ledger: $30 cap, $0.78 spent (stage 1) ⇒ $29.22 remaining. Worst-case
this run: $17.04 ⇒ total ≤ $17.82. **The $10–20 all-in target holds**, with the
caveat that two complete 14 h build retries do NOT fit — one retry max, then
stop and re-plan. Data transfer: ~71 GB ingress (free), results egress ~MBs.

### Duration estimate (assumptions explicit)

| phase | 8.0 B | 9.4 B | assumption |
|---|--:|--:|---|
| provision + cargo build | ~0 h | ~0 h | overlapped with download (stage 1: 85 s cargo build) |
| download 70.65 GB | 1.2–2.4 h | same | 8.2 MB/s measured @2 ranged workers (stage 1); script uses 4 workers, assume ≤2× |
| recompress gz→zst + line count | 2.0–2.7 h | 2.3–3.1 h | ≤1,179 s/B-lines (stage-1 slice pass incl. pigz tee; ours drops pigz) |
| **build** | 3.8–11.7 h | 4.5–13.7 h | 0.58 M/s (300 M dict-fits-RAM rate, spill ≈ free) … 0.19 M/s (spill costs what swap cost @1B). UNKNOWN until dict-spill is benchmarked — this is the dominant uncertainty |
| validate + collect | 0.5 h | 0.5 h | stage-1 query-mmap opens 1 B in 0.35 s |
| **total** | **8–17 h** | **9–20 h** | |

Cross-check on the pessimistic bound: total writes scale to ~1.46 TB @8.4 B
(173.5 GB/B measured); at the 78.75 MB/s sustained EBS ceiling that is ≥5.1 h of
pure writing — a <4 h build is physically impossible on this box; >14 h means
spill overhead worse than stage-1 swap, which is an abort-grade finding in itself.

## 3. Dump (verified 2026-06-13, `curl -sI` only — nothing downloaded)

| file | size | notes |
|---|--:|---|
| `https://dumps.wikimedia.org/wikidatawiki/entities/latest-truthy.nt.gz` | **70,654,648,844 B (70.65 GB)** | Last-Modified 2026-06-12T12:16:23Z; `Accept-Ranges: bytes` (ranged workers OK); ETag `6a2bf897-107358620c` |
| `…/latest-truthy.nt.bz2` | 42,877,885,960 B | **REJECTED**: bz2 decompression is serial/slow — stage-1 lesson; the 28 GB saved is not worth a serialized multi-hour decompress on 2 vCPU |

Triple count estimate: stage-1 recompression density (9.23 GB gz / B-lines at
pigz -6; the official dump compresses slightly tighter) puts 70.65 GB at roughly
**8–9 B triples**; plan for 8.0–9.4 B. The recompress step's `wc -l` gives the
exact count before the build starts — record it in STATUS.md.

Decompression plan: fetch `.gz` with 4 ranged-HTTP workers, then ONE streaming
recompress pass `rapidgzip -P2 | tee >(wc -l)` → `zstd -3 -T2` (stage-1 measured
zst builds 5–7 % faster than gz, and the pass yields the exact line count, which
is the validation denominator). Then **delete the .gz** (frees 71 GB the disk
budget needs). Fallback if wall-clock is tight: build directly from `.gz`
(supported — fused decompress+parse) and skip the ~2–3 h pass, losing the exact
pre-count and ~5 % build speed.

## 4. Launch sequence

All scripts live in `bench/wikidata-8b/scripts/`, share `config.sh`, and
**default to dry-run** (print every AWS/ssh command, execute nothing). Export
`EXECUTE=1` to run for real. State (instance id, ip, key, ledger) goes to
`bench/wikidata-8b/state/` (gitignored).

```sh
cd bench/wikidata-8b/scripts

# 0. edit config.sh: SPARQ_SHA (post-merge), confirm dict-spill flags reconciled (sq-1q3)
./launch.sh                  # dry-run: review every command it would run
EXECUTE=1 ./launch.sh        # keypair + SG + run-instances + wait + record ip
EXECUTE=1 ./run.sh           # scp remote-8b.sh + mem-sampler.sh, nohup start, poll
#   …hours pass; run.sh polls every 60 s and tails progress markers…
EXECUTE=1 ./collect.sh       # scp ~/results.tgz + ~/r/ → ../results/
EXECUTE=1 ./terminate.sh     # tag-verified terminate + wait + SG/key delete + ledger
```

`run.sh` is restartable: if the supervisor dies, re-running it re-attaches to the
poll loop (the driver runs under nohup on-box and is not affected). After ANY
crash: read STATUS.md, check `state/instance-id`, check the console for tagged
instances — never double-launch.

### The build command (heart of the run)

Executed by `remote-8b.sh`:

```sh
# Flag names + cli feature plumbing RECONCILED against the merged impl (sq-1q3):
# dict-spill is a default sparq-cli feature, so the plain release binary has the spill
# path; the env vars below are the merged SpillConfig::from_lookup gate.
SPARQ_SHARDED_DICT=1 SPARQ_BUILD_TIMING=1 \
SPARQ_DICT_SPILL=1 SPARQ_DICT_SPILL_BUDGET_MB=8192 \
timeout 86400 /usr/bin/time -v \
  sparq-cli build ~/data/truthy.nt.zst ntriples ~/idx 32
```

- `32` = 32 M-triple chunks, same as stage 1 (~0.5 GB triple-sort cap).
- Budget 8,192 MB. The merged `SpillConfig` accounts ONLY the dedup caches + the
  external-sort run buffers under this budget; it EXCLUDES the `chunk`×12 B triple-run
  buffer (32M×12 B ≈ 384 MB at chunk=32) and the ~0.5–1 GiB parse pipeline. So expected
  peak RSS ≈ 8,192 MB + ~1 GiB floor ≈ 9–10 GiB on the 15 GiB-usable box — headroom for
  page cache. This confirms 8,192 MB is safe; it need NOT be lowered for overhead (the
  overhead is already outside the budget). The watchdog still aborts on sustained swap
  >24 GiB as the backstop.
- `timeout 86400` (24 h) is the build-phase hard stop.
- The build emits the `parse+intern+spill done` phase marker (verified against the
  merged `build_timing::report`); the §9 throughput-floor checkpoint greps for it.

## 5. Monitoring

- `mem-sampler.sh` (stage-1 pattern + disk): every 30 s appends
  `<UTC> memused=<MiB> swapused=<MiB> diskfree=<GiB>` to `~/r/mem-sampler.log`.
  The driver also runs the watchdog: kills the build on sustained swap >24 GiB
  (10 consecutive samples) or disk free <50 GB.
- `SPARQ_BUILD_TIMING=1` phase markers in `~/r/build-8B.log`. Stage-1 1 B
  baselines to scale by ~8.4×: parse 1,035 s → expect ≤2.4 h (abort checkpoint:
  not done by T+5 h); merge_remap+remap 2,869 s; sibling sorts ∥ dict-save to
  4,026 s.
- Supervisor side: `run.sh` polls the driver log for `== ` step markers and the
  `DONE` flag file; prints elapsed + last marker each minute.
- What "spill works" looks like: peak RSS ≈ budget + ~2 GiB (NOT the 14.9 GiB
  RAM-clamp of stage 1), swap ≲ 2 GiB transient, CPU util well above the 30 %
  swap-collapse of the 1 B run.

## 6. Validation (after build rc=0)

Same probes as stage 1, via `query-mmap` under `/usr/bin/time -v`:

```sparql
SELECT (COUNT(*) AS ?c) WHERE { ?s ?p ?o }
SELECT (COUNT(*) AS ?c) WHERE { ?s <http://www.wikidata.org/prop/direct/P31> ?o }
SELECT (COUNT(*) AS ?c) WHERE { ?s <http://www.w3.org/2000/01/rdf-schema#label> ?o }
```

Acceptance: opened-triples total == recompress line count minus dedup (stage-1
dedup was ~0.07 % @1B — record the exact number); open-from-cold O(1)-ish
(stage 1: 0.35 s / ~120 MB RSS at 1 B); all three queries return. Record wall +
RSS for each.

## 7. Results collection

`collect.sh` pulls `~/results.tgz` (driver log, build log, query logs,
mem-sampler, machine spec, sha, df history) into `bench/wikidata-8b/results/`.
Verify the tarball expands and `build-8B.log` ends with the index-built line
BEFORE terminating. Then write the numbers into STATUS.md and (post-run) a
`research/wikidata-lowresource-stage2.md` modeled on stage 1, including the cost
ledger.

## 8. TERMINATION checklist (never skip, in order)

1. results.tgz on the supervising machine and readable.
2. `EXECUTE=1 ./terminate.sh` — verifies the target instance carries
   `purpose=sparq-hw-validation`, refuses i-090531b4ede8f2d3f, terminates, waits
   for `terminated`.
3. Confirm zero remaining tagged instances:
   `aws ec2 describe-instances --filters Name=tag:purpose,Values=sparq-hw-validation Name=instance-state-name,Values=pending,running,stopping,stopped`
4. Root volume is DeleteOnTermination=true — confirm no orphan volumes:
   `aws ec2 describe-volumes --filters Name=tag:purpose,Values=sparq-hw-validation`
5. SG + keypair deleted (terminate.sh does this; verify).
6. Ledger updated (`state/cost-ledger.txt` + STATUS.md): launch→terminate wall ×
   $0.426/h, added to the $0.78 running total.

## 9. Abort criteria (any one ⇒ collect partials, terminate, write up honestly)

| trigger | threshold | rationale |
|---|---|---|
| budget | instance lifetime ≥ 40 h (dead-man `shutdown -P +2400`) | $17.04 ceiling keeps project ≤ $17.82 of $30 |
| supervisor budget stop | 30 h elapsed and build not yet in sibling-sort/dict-save phase | $12.78 spent; remaining phases would blow 40 h |
| build wall | `timeout 86400` on the build (24 h) | >24 h ⇒ <0.10 M/s ⇒ thesis failed at this spec |
| throughput floor | `parse+intern+spill done` marker absent at T+5 h after build start | 1 B parse scaled ×8.4 = 2.4 h; 5 h = >2× allowance |
| swap | >24 GiB for 10 consecutive 30 s samples | dict-spill not bounding memory — the run's premise is void; stage-1 1 B peaked 20.4 GiB at 1/8 the scale |
| disk | <50 GB free on / | failure imminent; die with logs intact |
| download | not complete in 4 h or size ≠ Content-Length | network anomaly; retry once then abort |

A second full attempt is allowed ONLY if (spent so far + worst-case retry) ≤ $20
all-in and the first failure's cause is understood and fixed.
