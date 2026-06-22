# Wikidata ingestion benchmark — measured status (real dump, M1)

Measured this session on the **real** Wikidata "truthy" dump (40 GB `.bz2` downloaded;
~9.4 B triples / ~1.08 TB decompressed), 50 M-triple prefix, on a **2020 MacBook Air M1
(8-core, 16 GB, fanless)**. Numbers are the sparq out-of-core external-memory build (parse +
k-way-merge sort + all 6 permutation indexes, bounded RAM). Cross-engine comparison is
hardware-contextualized and primary-sourced (workflow `wf_3a85b619-5e6`, 8 agents).

## Measured (50 M real-Wikidata sample)

| stage | rate / time | note |
|---|---|---|
| bz2 single-stream decompress | **0.93 M/s** (102 MB/s) | the bottleneck *from `.bz2`* |
| zstd recompress (one-time, -9 -T0) | 611 MB/s; 5.54 GB→335 MB (16.6×) | gets you off bz2 |
| oxttl parse only (1 thread) | 1.37 M/s | |
| parse + dict intern (1 thread) | 0.96 M/s | **dict halves it**; 8.46 M distinct in 20 M (~42%) |
| **full parallel build from `.nt`** | **1.28 M/s** (39.0 s) | peak RAM **2.69 GB**, index 4.5 GB (90 B/triple) |
| **full parallel build from `.zst`** | **1.30 M/s** (38.5 s) | zstd decompress fully hidden under parse; **byte-identical** index; COUNT ✓ |

Index opens via mmap in 2.2 s; deduped 50 M→49.95 M (real Wikidata has dup triples).

## Cross-engine comparison (⚠️ = far larger hardware than sparq's fanless M1)

| engine | wall-clock | throughput | hardware | dataset | includes |
|---|--:|--:|---|---|---|
| **sparq (measured)** | 39 s / 50 M | **1.28–1.30 M/s** | **M1 Air 8c/16 GB fanless** | 50 M truthy | parse+sort+6 perms, out-of-core, 2.69 GB RAM |
| **QLever** | 4.4 h | ~1.26 M/s | Ryzen 9 16c/128 GB/7.3 TB NVMe ⚠️ | ~20 B full | parse+vocab+global-ids+6 perms, external-memory |
| QLever (Wikimedia 2026) | 6 h | — | AWS 32 vCPU/256 GB ⚠️ | ~20 B | on-disk index |
| Virtuoso | 10 h | 0.33 M/s | 24c/378 GB/4×SSD ⚠️ | ~11.9 B | parallel bulk load, on-disk |
| Virtuoso (Wikimedia 2026) | 20 h | — | AWS 32 vCPU/256 GB ⚠️ | ~20 B | on-disk |
| GraphDB 9.10 | 32 h | 0.14 M/s | AWS 16 vCPU/128 GB ⚠️ | 16.3 B | parse+index+persist |
| **Jena TDB2 xloader** | 40 h | 0.046 M/s | Ryzen 9 16c/128 GB/NVMe ⚠️ | 6.6 B truthy | node table + 3 indexes, external-memory |
| Neptune (Blazegraph-lineage) | 3 d 2 h | 0.062 M/s | AWS 16 vCPU/128 GB ⚠️ | 16.3 B | parse+index+persist |
| Blazegraph (WDQS 2024) | 5.2 d | 0.037 M/s | 6c/64 GB/4 TB NVMe ⚠️ | 16.6 B | load only |
| Oxigraph | ">1 week" (2023, dated) | — | unspecified | full | RocksDB bulk_load |
| RDFox (vendor "24 min") | 24 min | ~6.5 M/s | **undisclosed, in-memory** ⚠️⚠️ | unstated (likely 15 B+) | in-RAM load; unverifiable |
| RDFox (ESWC 2023, independent) | 6 h 25 m | 0.71 M/s | AWS 128 vCPU/**1,952 GB** in-mem ⚠️⚠️ | 16.3 B | parse+index+persist |

**DBLP 390 M, identical 16c/128 GB server (the one apples-to-apples bench):** QLever 1.7,
Virtuoso 0.7, Oxigraph 0.6, Stardog 0.5, GraphDB 0.4, Jena 0.2, Blazegraph <0.1 M/s. sparq's
1.28–1.30 (on a **laptop**) sits between Virtuoso and QLever. (DBLP is homogeneous → cheaper
dict than Wikidata, so it flatters everyone vs a real Wikidata dict.)

## Headline

**The fairest architectural peer is QLever** — same design (dict-encoded ids + 6 perms +
external-memory build). QLever's ~1.26 M/s on a 16-core/128 GB **server** is essentially
identical to sparq's **1.28–1.30 M/s on a fanless 8-core/16 GB laptop**. **Per-core and
per-GB-RAM, sparq is competitive-to-better than every documented engine.** QLever's only edge
is that it had the disk+RAM to actually *finish* 20 B; this M1 cannot.

## Why the full dump can't run on THIS machine (in binding order — speed is last)

1. **Disk (hard blocker).** 6 perms × 9.4 B: raw-key floor 6×9.4 B×12 B = **677 GB**; at the
   *measured* 90 B/triple (incl. dict) ≈ **~847 GB**. Quote **~680–850 GB**. Free: **56–67 GB**
   → 10–12× short. No speed lever changes this.
2. **Dict RAM (hard blocker).** ~42% distinct ⇒ ~4 B unique terms (mostly once-seen value
   literals). The in-RAM dict exceeds 16 GB long before the build finishes.
3. **(Only after 1+2) Parallel-build serialization.** 8 cores net ~1.30 M/s vs single-core
   parse+intern 0.96 M/s — i.e. ~6× of theoretical 8-core throughput is lost, almost
   certainly at `Dict::merge_remap` (the one mandatory global-id serialization point).

So: **you cannot build full Wikidata truthy on a 67 GB-disk/16 GB-RAM laptop regardless of
engine speed.** The 5× gap to RDFox's "24 min" is *separate* and secondary — and that claim
is an in-memory load on an undisclosed large-RAM server, very likely a larger dump, so it's
un-normalized on both hardware and triple count.

## PROFILING CONFIRMED (2026-06-09) — `merge_remap` is the serialization

`SPARQ_BUILD_TIMING=1` phase instrumentation on the real 50 M `.zst` build attributes the
16.49 s parse phase:

| sub-phase | time | parallel? |
|---|--:|:--:|
| parse (8 cores, per-chunk local dicts) | 5.23 s | ✅ |
| **`Dict::merge_remap` (re-intern into global dict)** | **7.57 s** | ❌ serial |
| triple-remap (apply remap to every triple) | 2.96 s | ❌ serial |

Full wall: parse 16.5 s → kway-merge SPO 18.8 s → sibling sorts 32.0 s → finalize 38.5 s.
**`merge_remap` (7.57 s) alone exceeds the entire parallel parse (5.23 s)** — the dict work is
done twice, the second time single-threaded. The serial dict portion (merge + remap = 10.5 s)
is ~27 % of total wall and 100 % serial. Confirmed (corroborated by a `samply` function-level
profile). This is exactly what the sharded parallel dict must eliminate.

## Prioritized next steps (real wins, measure-first)

1. **[measure-first, gates all] Profile the parallel-build serialization.** Flamegraph the
   8-core build; confirm whether the ~6× loss is `merge_remap` (→ step 2) or
   bandwidth/external-merge I/O (→ dict reforms are marginal). Highest value, low cost.
2. **[real win, the big one — only if step 1 confirms merge_remap] Sharded parallel dict.**
   Per-thread local intern (zero contention) → hash-partition terms across shards → parallel
   per-shard dedup + prefix-sum for global id ranges → parallel column rewrite. Replaces the
   serial `merge_remap`. Target **1.3 → 3–4 M/s** on 8 cores (won't reach 6.5 alone). High.
   - **2a DONE (commit pending): 3-stage parse∥merge pipeline.** A first, low-risk,
     byte-identical step toward 2: parse (rayon) and the serial dict-merge now run on
     separate stages so the parse of block N+1 overlaps the merge of block N (previously
     sequential per block). Measured real 50 M `.zst`, interleaved best-of-4: 38.77 s →
     36.66 s (**−5.4 %**, every pair new<old). Modest because it only *hides the parse* under
     the merge; the merge itself is still serial.
   - **2b DONE (`SPARQ_SHARDED_DICT=1`): hash-sharded parallel dictionary.** `ShardedDict` —
     N independent `Dict` shards (term routes to `hash%N`) so the dominant dict work runs in
     parallel, contention-free. A term gets a temp id `shard*STRIDE+local` (STRIDE=
     INLINE_BASE/N); these sort in the SAME order as the final dense ids `base[shard]+local`
     (base = prefix-sum of shard sizes), so the externally-sorted SPO needs only an
     order-preserving `remap_perm_file` pass — no re-sort. `into_merged` consumes the shards
     into one regular `Dict` (moving term strings, only remapping the small prefix/datatype id
     fields), reusing the existing `save_mmap`/`numerics_of`; the merge interns straight from
     the partials' borrowed components (`term_parts`) — no `Term` alloc. Measured real 50M
     `.zst`, interleaved best-of-4: **36.3 s → 29.9 s (−17.6%**, every pair faster). Validated:
     identical query answers vs the non-sharded build on the synthetic (q02=1.25M, q04=5M,
     q06=140625, … all match) + COUNT 49,951,624 on real 50M + a roundtrip/order/dense-id unit
     test. NOT byte-identical (different but dense ids), so opt-in (env-gated) pending soak
     before defaulting.
3. **[real win, dual benefit] Extend inline tagged ValueIds beyond small ints** to dateTime,
   decimal/double, boolean, short langString. Much of Wikidata's 42%-distinct mass is
   dates/quantities/coords that pack inline and **never enter the dict** — shrinks *both* the
   dict-RAM blocker *and* the intern tax. **Measure the literal-type histogram of the 8.46 M
   distinct terms first** to size it.
5. **[done/real] Overlap dict-save with the sibling sorts.** The dictionary persist
   (`save_mmap` term-blob + sorted-hash index + numeric cache, ~6.5 s / multi-hundred-MB)
   ran as a serial tail *after* the 5 sibling permutation sorts (~13 s), yet it only needs
   the dict + SPO — independent of the perm sorts. Now it runs on its own thread concurrently
   with the sorts (`std::thread::scope`), hiding the dict write under the sort time. Measured
   real 50M `.zst`, interleaved best-of-4: 37.2 s → 34.1 s (**−8.2%**, every settled pair
   faster), byte-identical output, COUNT correct. Stacks with the parse∥merge pipeline (#2a).
4. **[done/real] Keep source on `.zst`/`.gz`.** Decompress fully hidden, `.zst` build
   byte-identical to `.nt`. Prefer over lbzip2/pbzip2 (parallel bzip2 burns parse cores).

**Deferred:** 6→3 perms + external spill dict (the only things that make a full build feasible
on a *bigger* box — halve disk to ~340–425 GB, ~1.3→~2 M/s — but perm-cut is a query-coverage
tradeoff and a spill dict likely slows intern 10–30%; measure-first on the query side; even
done, the full build stays infra-blocked on *this* M1). Radix sort (marginal; sort is hidden
under parse+I/O until merge_remap is fixed).

## Bottom line

The pipeline is **correct and per-core competitive with the best engine (QLever) on a
fraction of the hardware**. "Beat RDFox on this M1" is not achievable — not on throughput
(per-core we already match QLever and beat the rest) but because the output doesn't fit on
disk and the dict doesn't fit in RAM. Beating it would need (a) ~1 TB scratch + a spill dict
to hold the output at all, then (b) the sharded-dict fix to lift ~1.3→3–4 M/s, then (c) an
apples-to-apples run no published source currently provides.

## Hardware-validation attempt (2026-06-11) — INFRA-BLOCKED, $0 spent

The planned timed gz→queryable-store run (full truthy if it fit, else a ≥1 B-triple slice +
labeled linear extrapolation) was attempted on the `hardware-validation` branch via
`hwrun/launch.sh` + `hwrun/remote.sh` (ranged-parallel prefix download overlapped with the
release build; `SPARQ_SHARDED_DICT=1` out-of-core build of the slice; sanity COUNTs over
mmap). **No instance could be launched**: spot is still blocked by the missing EC2-Spot
service-linked role, the on-demand Standard-bucket quota is still 16 vCPU, and the running
prod t3.large consumes 2 of those 16 — so even the 16-vCPU fallback boxes
(c7i.4xlarge/c7g.4xlarge, which worked in the June campaign when the bucket was empty) now
fail with `VcpuLimitExceeded`. Verbatim errors, the quota arithmetic, and the three concrete
unblock options (spot SLR; quota ≥194; or an 8-vCPU/64 GB `r7i.2xlarge`, the smallest box
whose RAM holds a 1 B-triple build dict) are in `research/hardware-validation-blocked.md`.

**T2 verdict: blocked — target neither met nor refuted.** The <24-min question remains open
pending hardware; the code side (sharded dict + zstd source + out-of-core path) is committed
and the on-box methodology is launch-ready (`bash hwrun/launch.sh` once any unblock lands).

## Rung 5 MEASURED (2026-06-11) — 1 B real-truthy triples on r7i.2xlarge

The ≤8-vCPU unblock option above was sanctioned and run (branch `t2-rung5`, engine @
`ef86e66`, orchestration `hwrun/rung5-launch.sh` + `hwrun/rung5-remote.sh`, raw results
`hwrun/results/rung5/`). Box: **on-demand r7i.2xlarge, eu-west-2** — Xeon Platinum 8488C
(Sapphire Rapids), **8 vCPU = 4 physical cores × 2 HT, 1 NUMA node, 64 GB RAM**, 400 GB
gp3 (6000 IOPS / 500 MB/s). Input: first 10 GiB of the live `latest-truthy.nt.gz`
(2-connection verified ranged download, 1284 s), sliced to **exactly 1,000,000,000
N-Triples lines** (verified by `wc -l` tee) and recompressed `zstd -1` (6.9 GB, 198 s with
`rapidgzip -P 8`). Two instances were used (attempt 1's 14-way parallel download was
rate-limited into a broken gz — fixed to 2 verified connections and rerun); **total
~2.13 instance-hours, estimated spend ~$1.50** (`hwrun/results/rung5-cost.txt`).

### Measured

| metric | value |
|---|---|
| timed build (`.zst` → queryable 6-perm out-of-core store, `SPARQ_SHARDED_DICT=1`, 64 M-triple runs) | **737.8 s (12 min 18 s)** |
| **throughput** | **1.355 M triples/s** (1.07 M/s end-to-end if the one-time 198 s gz→zst slice pass is included) |
| peak RAM (max RSS) | **51.5 GB** — the 64 GB box is genuinely required at this scale; a 16 GB machine cannot hold the 1 B-triple build dict |
| CPU utilisation | 249% of 800% (~31%) — confirms large serial/IO-bound phases |
| index on disk | 84 GB for 999,282,473 distinct triples (~90 B/triple incl. dict — matches the 50 M measurement) |
| dedup | 1,000,000,000 → 999,282,473 (0.07% duplicate triples in real truthy, as at 50 M) |
| disk IO during build (iostat, 30 s samples) | avg 100 MB/s read / 218 MB/s write; **peak 461 MB/s write ≈ 92% of the provisioned 500 MB/s** — the sibling-sort phase intermittently brushes the gp3 throughput cap |
| sanity COUNTs over mmap (open ~3.5 s) | `COUNT(*)`=999,282,473 in 27 ms; `wdt:P31` 19 ms; `rdfs:label` 18 ms — all return |

Phase split (`SPARQ_BUILD_TIMING=1`), per 1 B triples: parse(parallel) 138.1 s ∥
dict-merge bucket (timer label `merge_remap(serial)`) **137.9 s** + triple-remap(serial)
**62.2 s** → 221.2 s wall to end of parse+intern (the 3-stage pipeline overlaps parse
under merge); k-way SPO merge +31.2 s; **sibling sorts ∥ dict-save +409.6 s (55% of
wall — the dominant phase at this scale, IO-heavy, see iostat above)**; finalize +75.7 s
= 737.8 s. Note: even with the sharded dict ON, the dict consolidation+remap bucket
still costs **~200 s serial per 1 B triples**.

### Extrapolations (labeled)

- **Full truthy (~9.4 B) on this same 8-vCPU box, LINEAR-RATE ASSUMPTION:** 9.4 B ÷
  1.355 M/s ≈ **6,940 s ≈ 1 h 56 m**. This is a **lower bound, and the run is not
  actually possible on this box**: peak RSS was already 51.5 GB at 1 B, and the in-RAM
  dict grows with distinct terms (~42% of truthy), so 9.4 B would blow past 64 GB long
  before completion (needs the deferred spill dict or ~300 GB+ RAM), and the index +
  scratch needs ~800 GB+ disk. Linearity is an ASSUMPTION on top of that.
- **What 192 vCPU would need for <24 min (RATE×CORES LINEARITY IS AN ASSUMPTION — and a
  measured-false one):** <24 min for 9.4 B ⇒ ≥6.53 M/s ⇒ **4.82× the measured 8-vCPU
  rate**, i.e. a 192-vCPU box needs to realise only ~20% of naive 24× linear scaling
  (naive linearity would give 32.5 M/s ≈ 4.8 min). **But** the same-box sweep says rate
  does NOT scale linearly with cores (load reaches only ~1.8× at 4 threads, ~2.0× at
  8 — see `research/hardware-validation-blocked.md`), and Amdahl on the measured phase
  split caps it: with the ~200 s/1 B serial dict bucket, max speedup =
  737.8/200.1 ≈ **3.7× ⇒ ≥31 min for full truthy on ANY core count**. So <24 min is
  unreachable until the serial dict consolidation drops below ~153 s/1 B (the <24-min
  per-1 B budget) — **that ~200 s/1 B serial bucket is now a measured optimisation
  target, not a profile-extrapolated one.** A real 192-core run also still needs the
  quota unblock and NUMA validation.

**T2 verdict update: partial-scale VALIDATED at 1 B real triples — 12.3 min, 1.36 M/s,
correct COUNTs, out-of-core, 51.5 GB peak RAM, on an 8-vCPU ~$0.61/h box.** The <24-min
full-truthy target remains neither met nor refuted on real hardware (192-core box still
quota-blocked), but the path is now quantified: shrink/parallelise the ~200 s/1 B serial
dict bucket, then re-measure many-core scaling.

## Rung-5 dict-bucket re-measurement (sq-3l43) — instrumentation + re-launch

The ~200 s/1 B SERIAL dict bucket above (`merge_remap(serial)` 137.9 s + `triple-remap
(serial)` 62.2 s) was measured at engine **`ef86e66`**, which is BEFORE the
`dict-consolidation` branch landed on main (parallel `intern_partials`, pipelined remap,
parallel `into_merged`, sharded-default). `research/dict-consolidation-verdict.md` PROJECTS
that bucket down to single-digit seconds at 1 B — an EXTRAPOLATION from 40 M synthetic on a
contended M1, never re-measured at 1 B real truthy. sq-3l43 is the cheap precursor to the
deferred 192-core validation (sq-bj3): one ~$1.50 r7i.2xlarge 1 B build on **current main**
to confirm or refute the bucket reduction.

Two defeaters in the existing harness made that re-measurement uninterpretable, both fixed
here (no fabricated numbers — this PR ships the instrumentation + re-launch, the on-box run
records the result to `hwrun/results/rung5/t2-dict-bucket.txt`):

1. **Stale engine pin.** `hwrun/rung5-launch.sh`/`rung5-remote.sh` defaulted `SHA=ef86e66`
   — the PRE-consolidation commit. Re-running them would have re-measured the OLD serial
   path. Default is now `origin/main`; the resolved commit is recorded to `engine-sha.txt`.
2. **Mislabelled phase report.** On the sharded DEFAULT path, `SPARQ_BUILD_TIMING=1` printed
   the now-PARALLEL intern / PIPELINED remap occupancy under the labels `merge_remap(serial)`
   / `triple-remap(serial)` — so a reader would have compared overlapped per-stage occupancy
   against the old ADDITIVE-serial ~200 s and drawn the wrong conclusion. The report now
   distinguishes the path (`sparq_core` `build_timing::PathKind`): the sharded/spill paths
   read `intern(parallel-occupancy)` / `triple-remap(pipelined-occupancy)` and surface the
   serial consolidation step as its own `dict-consolidate(serial)` term (the `into_merged`
   bucket — the interpretable dict-consolidation cost on the new path). The non-sharded
   serial build keeps the honest `merge_remap(serial)` / `triple-remap(serial)` labels.

**Status:** instrumentation + orchestration ready; the 1 B r7i.2xlarge run is launchable
with `bash hwrun/rung5-launch.sh` (uses `AWS_PROFILE=pss`, self-terminating, tag
`purpose=sparq-hw-validation`, dead-man `shutdown -h +150`). The on-box clone checks out
`origin/main`, so the box must be run AFTER this fix merges (otherwise it rebuilds the
mislabelled report). The dict-consolidation bucket number at 1 B real truthy on
post-consolidation main is NOT YET recorded — when the box runs, capture `dict consolidate
(into_merged)` + the `dict-consolidate(serial)` term from `t2-dict-bucket.txt` here and
compare against the projected single-digit seconds.

### 2026-06-17 launch attempt — BLOCKED by the account envelope (NOT yet measured) [OPUS-4.8]

The post-`#570` run was attempted on 2026-06-17 against `origin/main`. It is currently
**blocked by the AWS account quota/IAM envelope this work box runs under — no `dict-
consolidate(serial)` figure was produced, and none is recorded here (no fabrication).**
Both launch paths fail at `RunInstances`:

1. **On-demand `VcpuLimitExceeded`.** The original 2026-06-11 run fit the 16-vCPU Standard
   on-demand bucket (`L-1216C47A`) because only the 2-vCPU prod `t3.large` was running
   (14-vCPU headroom). Since **2026-06-13** the 8-vCPU `r7g.2xlarge` agent dev box
   `sparq-dev-claude` — *the box this orchestrator itself runs on*, which must never be
   terminated — also occupies that bucket. Standard usage is now 2 + 8 = 10 vCPU, so a
   fresh 8-vCPU `r7i.2xlarge` (→ 18) trips the limit. The 1 B build needs ≥64 GB RAM
   (51.5 GB peak RSS measured), and the smallest 64 GB box is an `x.2xlarge` (8 vCPU), so
   the 6-vCPU on-demand headroom cannot host it. The prod box and the dev box are both
   off-limits, so on-demand cannot proceed without a quota increase.
2. **Spot `AuthFailure.ServiceLinkedRoleCreationNotPermitted`.** Spot draws on a *separate*
   quota bucket (`L-34B43A08`), and a spot `--dry-run` reports "would have succeeded", so
   spot is this bead's natural unblock (also cheaper: ~$0.21/h vs ~$0.61/h on-demand).
   `hwrun/rung5-launch.sh` now takes `SPARQ_SPOT=1` to launch one-time spot (interrupt =
   terminate, so the self-cleaning + dead-man contract holds). But the real spot
   `RunInstances` is refused: the scoped SSO deploy role (`AWSReservedSSO_PSSSingleInstance
   Deploy`) lacks permission to create the one-time `AWSServiceRoleForEC2Spot`
   service-linked role (and cannot even `iam:GetRole` to check whether it exists).

**Unblock — one of:** (a) a privileged principal creates the `AWSServiceRoleForEC2Spot`
SLR once (`aws iam create-service-linked-role --aws-service-name spot.amazonaws.com`),
after which `SPARQ_SPOT=1 bash hwrun/rung5-launch.sh` runs end-to-end under the separate
spot quota; or (b) a Standard on-demand vCPU limit bump to ≥24 (then plain `bash
hwrun/rung5-launch.sh`); or (c) run it from a session that is NOT on the 8-vCPU dev box so
the on-demand headroom is free again. Tracked on sq-3l43; the deferred 192-core full
validation is sq-bj3.

### 2026-06-18 re-confirmation — BOTH paths STILL blocked (no change, no fabrication) [OPUS-4.8]

Re-probed live from the same work-box session before re-attempting; the account envelope
is unchanged from the 2026-06-17 attempt above, so **the `dict-consolidate(serial)` figure
at 1 B is still NOT produced and none is recorded here.** Evidence captured this session:

- Running instances unchanged: prod `t3.large` (2 vCPU) + dev `r7g.2xlarge`
  `sparq-dev-claude` (8 vCPU) = 10 vCPU Standard on-demand. No `purpose=sparq-hw-validation`
  instances exist (orphan-clean before and after).
- **On-demand still `VcpuLimitExceeded`.** The sanctioned self-terminating launch
  (`bash hwrun/rung5-launch.sh`, `--instance-initiated-shutdown-behavior terminate` +
  dead-man `shutdown -h +150` + cleanup trap) aborted at `RunInstances`: *"requested more
  vCPU capacity than your current vCPU limit of 16 … for the instance bucket"* (10 used + 8
  for a fresh `r7i.2xlarge` = 18 > 16). The pre-flight + cleanup trap left no instance,
  keypair, or SG behind. NB: a bare `--dry-run` RunInstances misleadingly reports "would
  have succeeded" because dry-run does NOT evaluate the per-bucket vCPU quota — only a real
  `RunInstances` surfaces the limit, so the dry-run cannot be used as the go/no-go signal.
- **Spot still `AuthFailure.ServiceLinkedRoleCreationNotPermitted`.** A real spot
  `RunInstances` probe is refused: the scoped SSO deploy role
  (`AWSReservedSSO_PSSSingleInstanceDeploy`) cannot create the one-time
  `AWSServiceRoleForEC2Spot` service-linked role, and cannot `iam:GetRole` to check whether
  it already exists. `servicequotas:GetServiceQuota` is likewise denied, so the limits
  cannot be read from this principal — only inferred from the live `RunInstances` verdicts.

Not run in-place on the dev box instead: it is **aarch64 (Neoverse-V1)**, not the x86-64
(Xeon Platinum 8488C) of the rung-5 baseline, so a figure from it would not be comparable
to the 137.9 s + 62.2 s x86 split this re-measurement compares against; it is also the
shared agent box (a ~12-min full-core 1 B build would contend with other agents, and the
~84 GB index would crowd the shared root volume). The bead scopes a dedicated,
self-terminating r7i.2xlarge precisely to avoid both. Unblock paths (a)/(b)/(c) above are
unchanged; producing the figure remains the responsibility of whoever satisfies one of them.
