# sparq — consolidated roadmap, dependency graph & parallel-execution plan

Status: the in-memory + out-of-core engine is **SOTA-competitive-to-faster than QLever** on
compute (synthetic 10M/100M + real olympics; 2.3–20×), hardware-tuned across 4 silicon families.
16 commits banked. What follows is everything *left* — features, optimisations, and scaling —
organised into work **threads**, with an explicit map of which threads touch the same code (and
must be serialised) vs which are independent (and can run concurrently, optionally on separate
EC2 boxes).

---

## 1. The work threads

### T1 — Many-core parallelism & NUMA scaling  *(primary goal: near-linear to ~196 cores)*
Source: `research/parallelism-scaling.md`. Touches the hot path: `exec.rs`, `lib.rs`, `dict.rs`.
- **T1.0 (days, low-risk):** mimalloc/jemalloc global allocator (one line, gated off wasm);
  rayon worker pinning; `numactl --interleave=all` probe; **thread-local `LocalVocab` + deferred
  merge** (port the parse-path `ShardedDict` pattern to query time — kills the `exec.rs:57`
  serialization point).
- **T1.1 (1–2 wk):** radix-partitioned parallel hash aggregation (direct GROUP BY fix) + parallel
  hash-join build; same for `distinct`/`minus`.
- **T1.2:** NUMA-aware index placement / per-socket replication; custom pinned threadpool.
- **T1.3 (deep):** morsel-driven pipelined execution; parallelise the serial merge/outer/bind/WCOJ
  joins. The real ceiling-lifter; months.
- **Prereq:** validate on a 2-socket box (M1 cannot reveal NUMA). Needs **T8**.

### T2 — Wikidata ingestion under 24 min
Source: `research/wikidata-ingestion-benchmark.md`, `fast-ingestion.md`. Currently **1.28–1.30 M/s**
on the M1 (≈ QLever's server rate on a laptop). 24 min for ~9.4 B truthy ⇒ need ~6.5 M/s.
- **The one bottleneck is `Dict::merge_remap`** (profiled) — the mandatory global-id serialization
  point; 8 cores net only ~1.3 M/s vs 0.96 single-core. **This is the SAME fix as T1.0's
  thread-local-vocab/sharded-dict** → T2's core *is* a T1 deliverable.
- Also: faster decompression default (zstd over bz2, already supported), bigger external-merge
  buffers, parallel k-way merge.
- **Hard gate:** the full dump needs ~680–850 GB disk + >16 GB dict RAM — **cannot run on the M1
  at all**; needs the big instance. So T2's *validation* is hardware-bound, but its *code* is the
  merge_remap parallelisation (shared with T1).

### T3 — Tagged/inline ValueIds (u64)  *(#23 + #26; foundational)*
Inline numerics/dates/decimals into the id so range FILTER + ORDER BY skip the dict (QLever's main
remaining compute edge). Blocked today: u32 inline range is exhausted → needs **u64 Id**. Changes
the fundamental `Id` type across `dict.rs`, `exec.rs`, the store, `lib.rs`. **Memory tradeoff**
(doubles perm width → may cost bandwidth-bound scans) → must be measured both ways. *Foundational:
every hot-path edit (T1, T4) builds on the Id representation.*

### T4 — Compressed on-disk permutation blocks (PForDelta/prefix)
The one place QLever still wins: smaller on-disk index at billions (a **disk-footprint** win, not
speed — sparq already wins query speed). Touches the store/index + every scan in `exec.rs`. Risk:
decode cost in the hot loop. Source: `term-index-compression.md`, `bit-level-encoding.md`,
`dict-compression-measured.md`.

### T5 — Inference completion  *(sparq-reason; isolated crate)*
- **T5.1 (#27):** OWL-RL single-pass extension — push the monotone OWL subset out of the semi-naive
  fixpoint; parallelise the fixpoint commit (currently serial per-round `all.insert`). Source:
  `inference-sota.md`, `inference.md`.
- **T5.2:** finish N3 builtins beyond v1 (parser marks log:implies/formulae "not supported beyond
  parse", `n3/parser.rs:184`); remaining OWL features.
- **Independent:** `sparq-reason` depends on `sparq-core` but **not** on `exec.rs` → does not
  conflict with T1/T3/T4.

### T6 — RDF 1.2 / SPARQL 1.2
Source: `rdf12-parser.md`, `rdf12-indexing.md`, `sparql12-engine.md`.
- **T6a — parser** (triple terms, RDF-star successor syntax): `sparq-core` parser files —
  **independent** of the exec cluster.
- **T6b — indexing** (triple terms in the store): joins the **store/id cluster** (conflicts with
  T3/T4).
- **T6c — engine** (SPARQL 1.2 algebra/eval): touches `exec.rs` (conflicts with T1).

### T7 — AI / GenAI supports  *(4 new feature-gated crates; isolated)*
Source: `genai-*.md` (5 reports). Crates: `sparq-sim` (structural IRI similarity from the 6 perms —
training-free, the novel edge), `sparq-introspect` (characteristic sets — **also feeds the planner**,
soft-links to T1/optimisation), `sparq-nlq` (NL→SPARQL retrieve-repair loop), `sparq-vectors` (mmap
embedding store + ANN). One-directional dep, zero main-path impact → **independent**. User directive:
sequence *after* inference (T5) + RDF/SPARQL 1.2 (T6) are optimised. First step: synthesise the 5
reports into one prioritised design doc.

### T8 — Measurement harness  *(sparq-bench; additive, independent)*
`sparq-bench` has **no thread-count sweep today**. Add one reporting per-subsystem parallel
*efficiency* vs `RAYON_NUM_THREADS` (load / scan / join / aggregate / serialise / infer). This is the
instrument T1 and T2 are blind without. Additive — conflicts with nothing.

### T9 — SPARQL feature gaps
- **Named graphs / GRAPH** — currently errors (`exec.rs:893`). Needs a quad store or graph-column.
- Property paths (`*`/`+`/`/`), SERVICE federation, remaining aggregates/expressions (the `M2:`
  markers at `exec.rs:2375/2640`). These touch `exec.rs` (conflict with T1).

---

## 2. Dependency & conflict map

Two clusters touch the same files and **must be serialised within the cluster** (one owner / one
branch at a time); everything else is **independent** and can run concurrently on its own
worktree/box.

```
 CORE HOT-PATH CLUSTER (exec.rs + dict.rs + store + lib.rs load)  — serialise internally
   T3 (u64 ids)  ──foundational──►  T1 (parallelism/NUMA)  ◄──shares merge_remap──  T2 (ingestion)
                         │                      ▲
                         └──── T4 (compressed index) ───┘   T6b/T6c, T9 also land here
   Recommended order inside the cluster:  T1.0 (+T2 core)  →  T1.1  →  T3  →  T4 / T6bc / T9 / T1.2 → T1.3

 INDEPENDENT THREADS (separate crates / additive — run fully concurrently, no merge conflicts)
   T5  inference        (sparq-reason)         ── no exec.rs / dict.rs edits
   T7  genai            (new sparq-* crates)    ── isolated; after T5+T6 per directive
   T6a RDF1.2 parser    (sparq-core parser)     ── parser files only
   T8  scaling harness  (sparq-bench)           ── additive instrument
```

**Key dependency facts:**
- **T2 ⊂ T1.0** — the 24-min ingestion bottleneck (`merge_remap`) is the *same* serialization point
  as the query-time vocab fix. Do them as one piece of work; don't fork them.
- **T3 is foundational** to T4 and interacts with T1 (both edit `exec.rs`). Decide T3 early so T4 and
  T1.2/1.3 build on the final Id type — *but* T3's payoff is unproven (memory tradeoff), so gate it
  behind a measurement before committing the whole engine to u64.
- **T1, T2, T8 all need the NUMA/big box** to *measure* (not to write). T8 must land first.
- **T5, T7, T6a, T8 are conflict-free** with the core cluster → these are the threads to run **in
  parallel on cheap EC2 boxes** while the single core-cluster owner works the hot path locally.
- **T7 soft-depends on T1/optimisation** only via characteristic sets (T7's `sparq-introspect`
  doubles as a planner cardinality input) — a nice-to-have link, not a blocker.

---

## 3. Critical path & sequencing

1. **T8 first** (scaling harness) — cheap, unblocks all measurement. Do on M1 + a cheap Linux box.
2. **T1.0 + T2-core together** (allocator + thread-local vocab + sharded merge_remap) — biggest
   single scaling lever; fixes both query aggregation *and* ingestion throughput. Validate on the
   NUMA box.
3. **T1.1** (radix-partitioned operators) — direct GROUP BY/join/distinct scaling.
4. In parallel throughout: **T5** (inference) and **T8/T6a** run on their own boxes/worktrees with
   zero conflict.
5. **T3** (u64 ids) — gated on a measurement; once decided, **T4** (compressed index) follows.
6. **T1.2 → T1.3** (NUMA placement → morsel) — the deep ceiling-lifters, after T1.0/1.1 prove out.
7. **T6 (RDF/SPARQL 1.2)** then **T7 (GenAI)** — per the user's stated sequencing (after inference +
   the core engine are optimised).

---

## 4. EC2 budget-managed execution plan

Goal: run the independent threads concurrently without tripping over each other, **≤ $5/day** EC2
compute + **≤ $10 one-time** for the NUMA validation. Code isolation is via **git worktrees** (free,
local); EC2 is only for *running/measuring* Linux + many-core work.

**Cheap per-thread test boxes (the $5/day budget):**
- Use **one small Graviton/x86 box per active independent thread that needs Linux CI** — e.g.
  `c7g.large` (2 vCPU, ~$0.06/hr) or `t4g.medium` (~$0.03/hr) for inference/parser/genai test runs.
- **Terminate when idle** (each thread's box up only while actively benchmarking). 3 boxes × 4 h/day
  × $0.06 ≈ **$0.72/day** — well under $5. Even a c7g.2xlarge (8 vCPU, ~$0.24/hr) for ~8 h ≈ $1.9.
- **Network discipline:** generate datasets **on the box** (`sparq-bench dump`) — never transfer big
  data; only scp the **827 KB source tarball** in and tiny CSV results out. Egress on results is
  ~$0. This is what kept the whole hardware campaign under ~$2.
- Reuse the proven recipe (key+SG, Ubuntu 24.04 AMI, `aws-bootstrap.sh`/`hw-bench.sh`); always
  `terminate-instances` + delete key/SG at end of a session.

**The NUMA validation box (the ≤ $10 one-time):**
- Need ≥ 2 NUMA nodes to expose the dominant barrier. Target: a 2-socket EPYC (e.g.
  `m7a.48xlarge`, 192 vCPU). On-demand ~$11/hr would blow the cap in <1 h — so use a **spot
  instance** (~$3–4/hr typical) and run the **T8 scaling sweep + T1.0 A/B** in a **single ≤2 h
  session**, then terminate. That keeps it **≤ $8–10**. Capture all numbers in one run (sweep
  `RAYON_NUM_THREADS` 1→192 + `numactl` on/off) so the box is never up idle.
- A cheaper interim datapoint: memory-bandwidth scaling shows up even on a 1-socket high-core box
  (`c7g.16xlarge` spot, ~$1/hr); use it to dry-run the harness before the pricier 2-socket run.

**Concurrency rule to avoid thread collisions:** the **core hot-path cluster has exactly one
owner/branch at a time** (T1→T3→T4 serialised). The **independent threads (T5, T6a, T7, T8) each get
their own worktree + their own cheap box**, so up to ~4 threads progress simultaneously with zero
merge conflicts. Each lands via its own PR/commit; the core-cluster owner rebases as needed.

---

## 5. One-line summary per thread

| thread | what | files / crate | conflicts with | needs HW | independent? |
|---|---|---|---|---|---|
| T1 parallelism/NUMA | near-linear to 196c | exec.rs, lib.rs, dict.rs | T2,T3,T4,T6bc,T9 | yes (measure) | no — core cluster |
| T2 Wikidata 24-min | merge_remap parallel + zstd | dict.rs, lib.rs | T1 (same fix) | yes (big box) | folds into T1 |
| T3 u64 ValueIds | inline numerics/dates | dict.rs, exec.rs, store | T1,T4 | no | no — foundational |
| T4 compressed index | smaller disk @ billions | store, exec.rs | T1,T3 | partial | no — core cluster |
| T5 inference | OWL single-pass #27 + N3 | sparq-reason | — | no | **yes** |
| T6a RDF1.2 parser | triple terms | sparq-core parser | — | no | **yes** |
| T6b/c RDF/SPARQL1.2 | store + algebra | store, exec.rs | T1,T3 | no | no — core cluster |
| T7 GenAI | 4 feature-gated crates | new crates | — | no | **yes** (after T5/T6) |
| T8 scaling harness | thread-count sweep | sparq-bench | — | no | **yes** |
| T9 SPARQL gaps | GRAPH, paths, SERVICE | exec.rs | T1 | no | no — core cluster |
