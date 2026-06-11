# Hardware validation (T2 ingestion + T1/NUMA scaling) — BLOCKED, June 2026

Attempted 2026-06-11 (branch `hardware-validation`, engine @ `72b35e3`, AWS profile `pss`,
eu-west-2). **Every rung of the sanctioned launch ladder failed at `RunInstances`; zero
instances launched; $0.00 spent.** Errors below are verbatim. Orchestration used for the
attempt is committed at `hwrun/launch.sh` (ladder + dead-man + cleanup trap) and
`hwrun/remote.sh` (the full on-box plan: ranged-parallel truthy-prefix download overlapped
with the release build, lscpu/numactl topology capture, thread-scaling sweep, NUMA A/B
incl. a forced-remote `--cpunodebind=0 --membind=1` diagnostic, timed 1 B-triple sharded-dict
ingestion + sanity COUNTs). Both scripts are launch-ready and were never reached past the
launch call.

## The ladder, verbatim (attempt 2, 2026-06-11T11:10–11:13Z; attempt 1 at 11:05–11:06Z was identical for rungs 1–4)

**Rung 1 — spot c7i.48xlarge (192 vCPU, 2 sockets, 384 GB):**

```
--- RUNG 1 (spot c7i.48xlarge) FAILED 2026-06-11T11:10:40Z ---
aws: [ERROR]: An error occurred (AuthFailure.ServiceLinkedRoleCreationNotPermitted) when calling the RunInstances operation: The provided credentials do not have permission to create the service-linked role for EC2 Spot Instances.
```

**Rung 2 — on-demand c7i.48xlarge:**

```
--- RUNG 2 (on-demand c7i.48xlarge) FAILED 2026-06-11T11:11:10Z ---
aws: [ERROR]: An error occurred (VcpuLimitExceeded) when calling the RunInstances operation: You have requested more vCPU capacity than your current vCPU limit of 16 allows for the instance bucket that the specified instance type belongs to. Please visit http://aws.amazon.com/contact-us/ec2-request to request an adjustment to this limit.
```

**Rung 3 — on-demand c7i.24xlarge (96 vCPU):** same `VcpuLimitExceeded` (11:12:41Z).

**Rung 4 — on-demand c7i.4xlarge (16 vCPU):** same `VcpuLimitExceeded` (11:12:54Z).

**Rung 4b — on-demand c7g.4xlarge (16 vCPU, Graviton):** same `VcpuLimitExceeded` (11:13:00Z).

Full raw log (both attempts): `hwrun/results/launch-errors.log`,
`hwrun/results/launch-errors-attempt1.log`.

## Why even the 16-vCPU fallback now fails (it worked in the June hardware campaign)

The on-demand quota `L-1216C47A` ("Running On-Demand Standard (A, C, D, H, I, M, R, T, Z)
instances") is still **16 vCPU**, and it is a *running-total* bucket that **includes the
T family**. The user's terraform-managed `i-090531b4ede8f2d3f` (t3.large, `pss-solid-test`)
is running and consumes 2 of the 16 → only **14 vCPU of headroom**. The prior campaign's
c7i.4xlarge/c7g.4xlarge runs (`research/hw-bench-results.md`) fit because the bucket was
empty then; with the prod box up (and it must never be touched), **no 16-vCPU instance can
launch at all** — the smallest sanctioned rung is arithmetically impossible, not flaky.

Confirmed by enumeration: the t3.large is the *only* running instance in eu-west-2.

- Spot has its own quota bucket, but spot is blocked earlier: the `pss` deploy role cannot
  create the EC2-Spot service-linked role (verbatim above) and cannot touch IAM
  (`iam:GetRole` → AccessDenied, re-verified this session).
- The role cannot self-service a quota bump either: `servicequotas:GetServiceQuota` →
  AccessDenied (re-verified); a `RequestServiceQuotaIncrease` attempt was additionally ruled
  out this session as an unauthorised account-level change.

## What unblocks it (one-time, by an account admin)

1. **Spot SLR (cheapest path, ~$3.5/hr for the 192-core box):** run once with admin creds:
   `aws iam create-service-linked-role --aws-service-name spot.amazonaws.com`, and ensure the
   spot vCPU quota (`L-34B43A08`, "All Standard Spot Instance Requests") is ≥ 192.
2. **Or raise `L-1216C47A`** to ≥ 194 (192 + the prod t3.large's 2) for the on-demand
   c7i.48xlarge rung — ~$8.9/hr, still <$10 for the planned ≤60-min session.
3. **Or sanction an ≤8-vCPU box now** (fits the 14-vCPU headroom): `r7i.2xlarge`
   (8 vCPU, **64 GB**) is the right shape — 64 GB matters because the 1 B-triple T2 slice
   needs ~25–35 GB of build dict, which the 16 GB c7i.2xlarge cannot hold. This yields
   1→8-core curves + the 1 B ingest, but **no NUMA evidence** (1 node) and no 16-core point.

Once any of these lands, the run is `bash hwrun/launch.sh` — everything else (dead-man
shutdown, terminate-on-shutdown, tag-verified cleanup trap, 500 GB gp3, results scp) is
already wired and was exercised up to the launch call.

## Fallback numbers obtained instead (local M1, same SHA, same harness)

No EC2 box could launch, so the *only* obtainable partial-scale evidence is the identical
`sparq-cli scaling` sweep `hwrun/remote.sh` would have run, executed on the dev M1
(Apple M1, 4P+4E, 16 GB; macOS) at `72b35e3` — synthetic fixture `sparq-bench dump 2000000`
(~16 M triples), query suite `bench/qlever-synthetic/queries`, threads 1/2/4/8, best of 2.

| subsystem | 1 thr (ms) | 2 thr | 4 thr | 8 thr | speedup @8 | eff @8 |
|---|--:|--:|--:|--:|--:|--:|
| load (parse+intern+index, 16 M) | 50 788 | 32 956 | 27 854 | 28 177 | **1.80×** | 0.23 |
| q02_type_person | 437 | 286 | 274 | 260 | 1.68× | 0.21 |
| q03_star3 | 2 843 | 1 441 | 1 206 | 1 129 | **2.52×** | 0.31 |
| q04_follows_name | 9 288 | 7 011 | 5 974 | 5 322 | 1.75× | 0.22 |
| q06_filter_age | 116 | 87 | 125 | 106 | 1.09× | 0.14 |
| q09_count_edges | 1.6 | 0.2 | 0.1 | 0.1 | (sub-2 ms — warmup noise, not signal) | — |
| q10_optional_age | 1 679 | 1 107 | 933 | 1 350 | 1.24× (regresses 4→8) | 0.16 |

Reading (with the caveat that the M1 is 4P+4E, so ideal @8 is well below 8×):

- **LOAD plateaus at 4 threads (1.82×) and is flat at 8 (1.80×)** — the
  `Dict::merge_remap` global-id serialization point already profiled on this machine
  reproduces at this SHA. This is precisely the bottleneck the T1.0/T2 sharded-dict work
  targets, and exactly what the big-box sweep was meant to quantify past 8 cores.
- **Queries cap at 1.7–2.5× by 8 threads**, and q10 *regresses* 4→8 — consistent with the
  serial-merge/aggregation stages in `parallelism-scaling.md`. None of this can distinguish
  "M1 E-core asymmetry" from "real serialization ceiling"; only the uniform-core many-socket
  box can, which is the blocked measurement.
- **No NUMA inference is possible from this machine** (single node). The
  >20%-off-linear-past-one-socket question (the trigger for a NUMA-aware follow-up thread)
  remains **unanswered, not answered-negative**.

Raw TSV: `hwrun/results/m1-scaling.tsv`.

## Rung 5 MEASURED (2026-06-11): the same sweep on homogeneous x86 cores

The sanctioned ≤8-vCPU rung (option 3 above) ran on an **on-demand r7i.2xlarge**
(eu-west-2; Xeon Platinum 8488C Sapphire Rapids; **8 vCPU = 4 physical cores × 2 HT;
1 socket, 1 NUMA node; 64 GB**), engine @ `ef86e66`, identical harness/fixture/queries
(`sparq-cli scaling`, `sparq-bench dump 2000000` ≈ 16 M triples, threads 1/2/4/8, best
of 2). Raw TSV: `hwrun/results/rung5/scaling-x86.tsv` (a second, consistent sweep from
attempt 1 is in `hwrun/results/rung5-attempt1/`). This **replaces the M1 sweep as the
primary partial-scale datapoint** — the M1 numbers above stay for cross-arch context.

| subsystem | 1 thr (ms) | 2 thr | 4 thr | 8 thr | speedup @4 | speedup @8 | eff @8 |
|---|--:|--:|--:|--:|--:|--:|--:|
| load (parse+intern+index, 16 M) | 13 423 | 9 313 | 7 409 | 6 732 | **1.81×** | **1.99×** | 0.25 |
| q02_type_person | 181 | 148 | 133 | 130 | 1.36× | 1.39× | 0.17 |
| q03_star3 | 1 146 | 774 | 622 | 586 | 1.84× | 1.96× | 0.24 |
| q04_follows_name | 3 343 | 2 321 | 1 745 | 1 635 | 1.92× | 2.04× | 0.26 |
| q06_filter_age | 53 | 37 | 31 | 25 | 1.69× | 2.07× | 0.26 |
| q09_count_edges | 0.0 | 0.0 | 0.0 | 0.0 | (sub-ms — noise) | — | — |
| q10_optional_age | 852 | 607 | 456 | 405 | 1.87× | 2.10× | 0.26 |

Reading (caveat: the **clean homogeneous-core point is @4 threads** = 4 identical
physical cores; the @8 point adds only hyperthreads, so ideal @8 is ~4.8–5×, not 8×):

- **The merge_remap question is ANSWERED: it is a REAL serialization ceiling, not M1
  E-core asymmetry.** Load reaches only **1.81× on 4 identical physical x86 cores**
  (M1 @4 P-cores: 1.82× — statistically the same number on a completely different
  microarchitecture/OS), and HT adds just +0.18× (1.99× @8; M1 @8: 1.80×). The
  plateau-by-4-threads shape reproduces exactly on homogeneous cores. **Record this as
  a measured optimization target**: `Dict::merge_remap`/dict-consolidation serialization
  caps load scaling at ~1.8× regardless of core quality. (Corroborated at 1 B scale:
  the rung-5 build logged ~200 s/1 B serial dict-bucket time and only 31% CPU
  utilisation — `research/wikidata-ingestion-benchmark.md`.)
- **Query speedups (1.4–2.1× @8) also reproduce the M1 caps** on homogeneous cores —
  the serial-merge/aggregation stages of `parallelism-scaling.md` are likewise real,
  not core-asymmetry artifacts. (q10's M1 4→8 regression does not reproduce: 1.87→2.10×
  here — that one *was* an M1/E-core or memory artifact.)
- Absolute single-thread load is 3.8× the M1's (13.4 s vs 50.8 s for 16 M) — DDR5
  Sapphire Rapids server vs fanless laptop; cross-machine absolute comparisons remain
  apples-to-oranges, which is why speedup ratios are the signal here.
- **Still no NUMA evidence** (1-node box) and no >8-thread point: the
  >20%-off-linear-past-one-socket question remains **unanswered**, pending the
  quota-blocked big box.

## Status of the two threads

- **T2 (Wikidata <24 min):** **partial-scale MEASURED at 1 B real truthy triples on the
  rung-5 box — 737.8 s, 1.355 M/s, 51.5 GB peak RSS, COUNTs correct** (full numbers +
  labeled extrapolations: `research/wikidata-ingestion-benchmark.md`). Full-truthy
  validation remains quota-blocked (needs big box + spill dict or ≥300 GB RAM).
- **T1 / "T12" (many-core + NUMA):** 1–8-thread scaling now measured on homogeneous x86
  (table above): merge_remap ceiling confirmed real. NUMA + >8-thread behaviour remain
  hardware-blocked.

## Cost accounting

| item | value |
|---|---|
| instances launched (launch-ladder attempts, 2026-06-11 am) | 0 — $0.00 |
| rung-5 instances (2026-06-11 pm) | 2 × r7i.2xlarge on-demand (i-06b07ada7108200a1 ~8 min: download-failure attempt; i-025057480cb99a88a ~120 min: successful run + idle tail from a local orchestrator ssh loss, caught by manual tag-verified terminate) |
| instance-hours | ~2.13 |
| EBS/keypair/SG residue | none (verified by tag query after both rung-5 instances; prod `i-090531b4ede8f2d3f` untouched throughout) |
| estimated spend | **~$1.50** (~$1.29 compute + ~$0.21 gp3) — target ≤$1.50, hard cap $3: met |
