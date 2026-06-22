# Parallelism Scaling to 100s of Cores — Architectural Audit & Roadmap

**Scope.** Find every architectural barrier that would cap `sparq`'s scaling on a
196-core, multi-socket NUMA server (dual EPYC / Ampere), across loading/parsing,
SPARQL execution, RDFS/OWL inference, and serialization. Grounded in the actual
code (file:line). The M1 measurement (~1.7× on 1→8 threads for parallel GROUP BY)
is a poor proxy — E-cores + a single shared ~68 GB/s bus — but it *also* surfaces
genuine algorithmic serial fractions that the proper hardware cannot fix on its
own. This report distinguishes the two and gives a prioritized, measure-and-keep
roadmap. **Nothing here is implemented; this is research only.**

A blunt but central fact discovered in the audit: **there is no thread-pool
configuration, no thread pinning, and no NUMA awareness anywhere in the codebase.**
Every parallel section uses rayon's *default global thread pool* (`grep` for
`ThreadPool`/`build_global`/`num_threads`/`affinity`/`numa` returns nothing but
`rayon::current_num_threads()`). No custom allocator is registered either
(`grep` for `global_allocator`/`jemalloc`/`mimalloc` is empty → system `malloc`).
At 8 cores on one die this is invisible; at 196 cores across 2–8 NUMA nodes it is
the dominant ceiling.

---

## PHASE 1 — Map of current parallelism

Legend: ★ = embarrassingly parallel today; ⚠ = parallel but bottlenecked; ✖ = serial.

| Subsystem | Parallel today | Serial today | Data structure forcing seriality |
|---|---|---|---|
| **N-Triples parse** | ★ per-chunk parse+intern into per-thread `Dict` (`lib.rs:837-844`, `parse_ntriples_parallel`) | ✖ `merge_remap` of partials into one global `Dict` (`lib.rs:850-855`); ✖ `remap_extend` triple gather (`lib.rs:760-794`) | One growing global `Dict.table` (`HashTable<Id>`) interned single-threaded; `remap` LUT gather is latency-bound |
| **Turtle parse** | ★ statement-boundary chunked parse (`lib.rs:1058-1064`, `parse_turtle_parallel`) | ✖ same serial `merge_remap` + `remap_extend`; ✖ falls back to *fully serial* on any blank node or interspersed directive (`lib.rs:1009-1015`) | Document-scoped blank-node identity; single global dict merge |
| **Streaming/external build** | ⚠ 3-stage pipeline: decompress ∥ parallel-parse ∥ **serial** merge (`lib.rs:1090-1196`); ★ sibling-permutation sorts run concurrently (`lib.rs:548-555`) | ✖ Stage-3 dict-merge+remap is single-threaded by construction (comment: "merge ~10.5s/50M dominates parse ~5.2s", `lib.rs:1103-1106`) | Stage 3 owns the one global `Dict`; ordering kept byte-identical → inherently serial |
| **Sharded dict (opt-in)** | ★ `ShardedDict::intern_partials` interns N shards in parallel, no cross-shard lock (`dict.rs:1056-1100`); ★ `remap_perm_file` `par_iter_mut` (`lib.rs:1299-1301`) | ✖ serial *routing* scan that hashes every term to a bucket (`dict.rs:1060-1067`); ✖ `into_merged` unify+move (`dict.rs:1124-1173`) | Gated behind `SPARQ_SHARDED_DICT` env + external build + N-Triples only |
| **Index build** | ★ `par_sort_unstable` SPO dedup; ★ 5 sibling perms built `par_iter` (`store.rs:202-221`) | ✖ `compute_pred_stats` single linear scan of POS/PSO (`store.rs:319-353`) | One `Vec<[Id;3]>` per perm; stats are a sequential run-length pass |
| **Numeric cache** | ★ `numerics_of` `into_par_iter().map` (`lib.rs:665-675`) | — | Dense `Vec<f64>` parallel-filled |
| **Scan → Bindings** | ★ `par_iter().filter_map(build_row)` ≥ PAR_THRESHOLD (`exec.rs:1728-1733`) | ✖ LIMIT path serial (must stop early); ✖ small scans | `Vec<Row=SmallVec<[Id;4]>>`, per-row heap alloc when >4 cols |
| **hash_join** | ⚠ probe phase `par_iter().fold/reduce` (`exec.rs:1899-1914`) | ✖ **build phase serial** — one `FxHashMap<Key,Posting>` (`exec.rs:1877-1881`) | Single shared hash table; concurrent inserts would need locks (comment, `exec.rs:1898`) |
| **merge_join** | ✖ fully serial two-pointer walk (`exec.rs:1771-1801`) | ✖ entire join | Sequential merge over two sorted `Vec<Row>` |
| **left_outer_merge / left_outer_join** | ✖ fully serial (`exec.rs:2091-2153`, `2051-2086`) | ✖ entire join; `sort_unstable_by_key` is serial (`exec.rs:2106-2107`) | Sequential merge; per-row `eval_expr` for OPTIONAL filter |
| **bind_join** | ✖ fully serial group-by-then-scan (`exec.rs:1944-1969`) | ✖ entire join | `FxHashMap<Id,Vec<usize>>` groups, serial index lookups |
| **WCOJ (Leapfrog Triejoin)** | ✖ fully serial recursion (`exec.rs:1417-1444`, `lftj_recurse`); ★ trie sort `sort_unstable` is serial too | ✖ entire join + `build_trie` (`exec.rs:1449-1489`) | Single recursive `out: &mut Vec<Row>`; one mutable cursor stack per trie |
| **group_aggregate** | ⚠ per-group `eval_aggregate` `par_iter` ≥ PAR_THRESHOLD (`exec.rs:2301-2314`) | ✖ **serial group-hash build** (`exec.rs:2276-2285`); ✖ **serial intern of results** via `value_to_id` (`exec.rs:2327-2334`) | One `FxHashMap<Key,Vec<usize>>` + `order: Vec<Key>`; result interning needs `&mut LocalVocab` |
| **distinct_bindings** | ✖ serial `HashSet` retain (`exec.rs:2409-2412`) | ✖ entire op | One `HashSet<Row>` |
| **minus_bindings** | ✖ serial build + filter (`exec.rs:2196-2226`) | ✖ entire op | One `FxHashMap<Key,()>` |
| **order_bindings** | ★ keys `par_iter`; ★ `par_sort_by` ≥ PAR_THRESHOLD (`exec.rs:2443-2469`) | ✖ small inputs | `Vec<(key, Row)>` |
| **apply_filter** | ★ `par_iter` keep-vector ≥ PAR_THRESHOLD (`exec.rs:2483-2496`) | ✖ serial `retain` compaction (`exec.rs:2505-2510`) | `Vec<bool>` then sequential retain |
| **extend_bindings (BIND)** | ✖ **fully serial** — `eval_expr` + `value_to_id` per row (`exec.rs:2250-2261`) | ✖ entire op | `&mut LocalVocab` interning of each computed term |
| **values_bindings (VALUES)** | ✖ serial (`exec.rs:2229-2247`) | ✖ entire op | `&mut LocalVocab` interning of absent terms |
| **Serialization (single-pattern JSON)** | ★ `par_chunks` → per-chunk `String` frags, in order (`exec.rs:266-301`) | ✖ small results | `Vec<String>` fragments concatenated serially |
| **Serialization (general JSON)** | ★ `par_chunks` per-chunk frags (`exec.rs:364-391`) | ✖ small results; serial concat | Same |
| **eval_select materialise** | ★ `par_iter().map(materialise)` (`exec.rs:129-135`) | — | Per-row `Vec<Option<Term>>` (Term allocs) |
| **RDFS inference** | ★ ABox `sweep` rayon `fold/reduce` (`rdfs.rs:226-236`); ★ one-shot `dedup_derived` `par_sort_unstable` (`rdfs.rs:270-273`) | ✖ TBox saturation `transitive_closure` serial (`rdfs.rs:128-143`); ✖ `original` `FxHashSet` build serial (`rdfs.rs:305`); ✖ `original_in_order` serial sort (`rdfs.rs:361-365`) | TBox closure is a serial BFS map; one `FxHashSet<[Id;3]>` of all input triples |
| **OWL-RL inference** | ✖ **semi-naive fixpoint is essentially serial** (`owl.rs:319-729`) — per round: `Schema::build`, `Axioms::build`, delta joins, `all.insert` dedup-commit (`owl.rs:686-722`) all single-threaded | ✖ almost the entire materializer | `all: FxHashSet<[Id;3]>` mutated each round; `UnionFind` for sameAs; serial `cand` extend |

**Key constants/structures named:**
`PAR_THRESHOLD = 50_000` (exec.rs:32); `Row = SmallVec<[Id;4]>` (exec.rs:25);
`Key = SmallVec<[Id;2]>` (exec.rs:37); `Posting = SmallVec<[usize;2]>` (exec.rs:42);
`LocalVocab { terms: Vec<Term>, ids: FxHashMap<Term,Id> }` (exec.rs:57-61);
six `PermData` permutations as flat `Vec<[Id;3]>`/mmap/compressed (store.rs:77-84);
RDFS `PAR_THRESHOLD = 4096` (rdfs.rs:123).

---

## PHASE 2 — Architectural scaling diagnoses

### B0. NUMA-obliviousness (THE dominant barrier; M1 cannot reveal it)
There is one global rayon pool, no pinning, and a *single* index shared by all
workers. On a 196-core box spanning 2–8 NUMA nodes:

- The six `PermData` permutations live wherever the build thread's allocator
  happened to place them (first-touch policy → likely node 0). Every scan
  `par_iter` over `scan.rows` (`exec.rs:1731`) on a remote socket pays
  ~1.5–2.5× memory latency and contends on one node's memory controller and the
  cross-socket interconnect (Infinity Fabric / CXL). The flat-`Vec` layout means
  a hot predicate range is a *single* contiguous region with no per-node replica.
- rayon's work-stealing is NUMA-blind: a worker on node 7 steals a morsel whose
  data is on node 0. Morsel-driven systems (Leis et al., SIGMOD 2014) get
  **~30× on 32 cores** precisely by keeping morsels and operator state
  NUMA-local and dispatching them lock-free; sparq does the opposite.
- The numeric cache (`NumData::Owned(Vec<f64>)`, lib.rs:37) and dictionary blob
  are likewise single-node. `numeric_value` (lib.rs:588) is on the FILTER /
  ORDER BY hot path → remote gathers under load.

**Why it caps scaling:** beyond one socket, aggregate memory bandwidth should
grow with sockets, but a single-node placement pins all traffic to one
controller. Effective bandwidth saturates ≈ 1 node's worth regardless of core
count → speedup plateaus near (#cores on the home node). This is invisible on M1
(one memory domain) and is the single largest reason 196-core scaling will fail
without work.

### B1. Shared mutable `LocalVocab` interning — the query-time serialization point
`LocalVocab::intern` (`exec.rs:66-74`) takes `&mut self` and pushes to a `Vec`
+ `FxHashMap`. Every operator that produces *computed* terms must funnel through
it single-threaded:
- `group_aggregate` explicitly splits into a parallel read-only aggregate phase
  and a **serial intern phase** (`exec.rs:2300-2334`, comment: "Interning the
  results … stays serial and in `order`").
- `extend_bindings` (BIND) is fully serial *because* each row calls
  `value_to_id` → `local.intern` (`exec.rs:2250-2255`).
- `values_bindings` (VALUES) same (`exec.rs:2241`).

For a GROUP BY producing ~490k groups of skewed size, the parallel aggregate is
followed by a serial walk of 490k `value_to_id` calls plus the serial
`FxHashMap` group-build (B5). By Amdahl, even a 5–10% serial fraction caps
speedup at 10–20× — consistent with the M1's 1.7× once you add bandwidth and
E-cores. Note sparq **already solved this exact problem for parse-time** with
per-thread partial dicts + deferred merge (and `ShardedDict`); the query-time
`LocalVocab` simply never got the same treatment.

### B2. Global allocator contention from per-row `SmallVec`
`Row = SmallVec<[Id;4]>` inlines ≤4 ids, but **every join/projection/aggregate
that produces >4 columns heap-allocates per row**, and `Key`/`Posting` spill past
2. Join/group outputs are millions of rows. With the *system* allocator (no
jemalloc/mimalloc registered) and 196 threads all in `malloc`/`free`, the
allocator's global locks become the hottest contention point — a well-documented
many-core failure mode (mimalloc/jemalloc papers: per-thread arenas exist
specifically to avoid this). Concretely:
- `hash_join` probe `par_iter().fold` clones `build.rows[bi]` per match
  (`exec.rs:1888`) → per-match alloc on every worker.
- `lftj_recurse` `out.push(Row::from_slice(current))` (`exec.rs:1427`) — per
  result-tuple alloc (though LFTJ is serial today anyway, B-WCOJ).
- `scan_to_bindings` `par_iter` builds a `Row` per surviving row (`exec.rs:1716`).

On M1 (8 threads) this is mild; at 196 threads it can dominate. **One line**
(register mimalloc/jemalloc as `#[global_allocator]`) is a near-free first
experiment — though it must stay off the wasm build (B-wasm).

### B3. Row-at-a-time `SmallVec` layout — bandwidth & cache
The engine is row-oriented: a `Bindings` is `Vec<Row>`, each `Row` a separate
(possibly heap) allocation. This is the opposite of the Vectorwise/DuckDB
column-batch model. Consequences at scale:
- Poor spatial locality: iterating a column (FILTER on one var, ORDER BY key)
  strides across scattered rows, not a packed array → more cache lines touched,
  more bandwidth, no SIMD.
- The bandwidth wall the hardware research already measured (`exec.rs:934`:
  "sequential numeric access … the layout fix … 8–15× win") is exactly the
  row-vs-column issue, fixed *only* for the pushed-down sargable-filter scan, not
  for general operators.
- More bandwidth per result row means scaling is bandwidth-bound sooner — and on
  NUMA, bandwidth-bound = node-bound (B0).

### B5. Amdahl serial fractions (algorithmic, hardware can't fix)
Even with perfect NUMA placement these stay serial:
- **Serial group-hash build** (`exec.rs:2276-2285`): one `FxHashMap` insert per
  input row, first-seen `order` vector. For a large GROUP BY this is a big serial
  prefix before the parallel aggregate phase even starts.
- **Serial dict-merge** of parse partials (`lib.rs:850-855`, `1164-1190`): the
  build-path bottleneck the prefetch hints (lib.rs:760-794) only *soften*.
- **Serial `remap_extend`** triple gather: latency-bound, ~3s/50M measured.
- **Serial TBox saturation** (`rdfs.rs:325-328`): small, but a fixed serial cost.
- **Serial RDFS `original` set + `original_in_order` sort** (`rdfs.rs:305,361`):
  O(N) serial hashing + an O(N log N) serial sort wrapping the parallel core.
- **OWL fixpoint dedup-commit** (`owl.rs:686-722`): `all.insert` per candidate,
  per round, single-threaded — the join phases could parallelize but the commit
  serializes every round.

### B6. Synchronization in dedup and join-build
- `hash_join` build (`exec.rs:1877-1881`), `minus` table (`exec.rs:2197-2200`),
  `distinct` `HashSet` (`exec.rs:2410`), `bind_join` groups (`exec.rs:1945-1948`),
  `group_aggregate` groups (`exec.rs:2276`) are all **single shared maps built
  serially**. These are textbook radix-partitionable: partition by `hash(key) %
  P` so each thread owns a partition lock-free (Wang et al. NUMA radix
  aggregation; Balkesen et al. radix hash join). sparq does none of it.
- RDFS `dedup_derived` *is* one-shot parallel (`rdfs.rs:270-273`) — the good
  pattern; the others should copy it.

### B7. rayon fork/join overhead & single global pool
`PAR_THRESHOLD = 50_000` gates fan-out, but every parallel region is a fresh
`par_iter`/`fold`/`reduce` — i.e. fork-join per operator. A query pipeline of
N operators forks/joins N times, each a global barrier. Morsel-driven execution
instead keeps a persistent worker set pulling morsels across the *whole*
pipeline, amortizing scheduling and preserving locality. The single global pool
also means concurrent queries (a server!) fight over the same workers with no
admission control.

### B-WCOJ. WCOJ and all merge/outer/bind joins are fully serial
`lftj_recurse` (cyclic BGPs → triangles etc.), `merge_join`, `left_outer_merge`,
`left_outer_join`, `bind_join` have **no parallel path at all**. For workloads
dominated by these (graph-shaped queries, OPTIONAL-heavy queries) the engine
scales at ~1× no matter the core count. The M1 GROUP BY number doesn't even
exercise these.

---

## PHASE 3 — SOTA many-core techniques mapped to sparq

### 3.1 Morsel-driven parallelism (Leis et al., SIGMOD 2014; ~30× on 32 cores)
Split each pipeline's input into fixed-size *morsels*; a **lock-free dispatcher**
hands NUMA-local morsels to a persistent worker pool that runs the whole pipeline
to the next pipeline-breaker; work-stealing balances skew. **vs sparq:** sparq's
`par_chunks`/`par_iter` is single-operator fork-join with static chunking
(`chunk = len / (threads*4)`, exec.rs:271) and *no* NUMA locality. Adopting
morsels means: (a) a persistent pool (replace ad-hoc `par_iter`), (b) morsel =
a contiguous index sub-range tagged with its home node, (c) pipeline operators
(scan→filter→join-probe→aggregate-partition) fused so a morsel flows through
without materializing a `Vec<Row>` between each. This is the **deep rewrite** that
unlocks true many-core scaling; the rest are incremental wins that buy time.

### 3.2 NUMA-aware placement, pinning, per-socket replication/partitioning
- **Replicate** the (read-only, immutable) six permutation indexes + dict + numeric
  cache **per socket** (interleave or explicit per-node copies) so every scan is
  node-local. For Wikidata-scale (~40 GB à la QLever) replication across 2–4
  nodes is affordable; for larger, **partition** by predicate/subject hash and
  route morsels to the owning node. Tools: `libnuma`/`hwloc`, or `numactl
  --interleave` as a zero-code first probe, or a custom pinned thread pool
  (`rayon::ThreadPoolBuilder::start_handler` to pin each worker via
  `sched_setaffinity`). **vs sparq:** today literally nothing — first-touch on
  node 0. This pairs with B0 and is the highest-leverage hardware-dependent fix.

### 3.3 Radix-partitioned parallel hash aggregation & join
Partition keys by high bits of the hash so each thread builds/probes a private
hash table that fits in L2 and lives on its node; merge partition results
(Balkesen/Teubner radix hash join — "fastest algorithm"; Wang NUMA radix
aggregation with task-stealing). **Maps to:** replace the serial builds in
`group_aggregate` (exec.rs:2276), `hash_join` (exec.rs:1877), `minus`
(exec.rs:2197), `distinct` (exec.rs:2410) with a radix-partition pass + per-
partition parallel build. Byte-identical results require care on GROUP BY's
first-seen `order` (must re-impose a deterministic order after partitioned build,
or sort — sparq already sorts elsewhere).

### 3.4 Lock-free / sharded / thread-local vocab with deferred merge
**sparq already does this for parse** (`ShardedDict`, dict.rs:992; per-thread
partial dicts, lib.rs:837). Apply the identical idea to query-time `LocalVocab`:
each worker interns into a thread-local vocab in a private id sub-range, then a
deferred merge/remap (exactly like `merge_remap`/`remap_sharded`) at the
materialization boundary. This directly removes B1 and unblocks parallel BIND /
VALUES / aggregate-result production. Because the local-vocab id range
`[3·2^30, 2^32)` (exec.rs:48) is already partitionable, sharded sub-ranges are
natural.

### 3.5 Per-worker arena/bump allocation + columnar batch execution
- **Arena/bump:** allocate `Row`s and intermediate columns from a per-worker
  bump arena reset per morsel → kills B2 allocator contention without a global
  allocator swap. (Cheaper interim: register mimalloc/jemalloc globally.)
- **Columnar vectors (Vectorwise/DuckDB):** represent a batch as
  `struct ColumnBatch { cols: Vec<Vec<Id>>, len }` instead of `Vec<SmallVec>`.
  SIMDable, bandwidth-friendly, cache-dense. This is the natural home for the
  existing inline-integer ValueIds and the sargable-filter fast path. Large rewrite
  touching every operator; do it *with* morsels, not before.

### 3.6 Parallel external/dictionary build at scale
The external build already runs sibling-perm sorts concurrently and overlaps
decompression (lib.rs:541-562, 1090-1196), and the sharded dict parallelizes
interning. Remaining serial spots: the stage-3 merge for the *non-sharded* path,
`compute_pred_stats` (store.rs:319), and the routing scan in `intern_partials`
(dict.rs:1060). Make sharded-dict the **default** (not env-gated) once validated,
and parallelize pred-stats with a per-predicate partitioned reduction.

### 3.7 What QLever / Oxigraph / DuckDB / Umbra / academic RDF stores do that sparq doesn't
- **QLever:** scales to 7B-triple Wikidata in ~40 GB; sorted-merge-centric
  operators with **lazy/streaming results** and pipelined parallelism, plus
  inline ValueIds (sparq mirrors the ValueId idea, exec.rs:46-48, but not the
  lazy pipelined execution). sparq materializes a full `Vec<Row>` between
  operators — the anti-pattern QLever avoids.
- **DuckDB/Umbra:** morsel-driven, **vectorized columnar** execution, NUMA-aware
  scheduling, push-based pipelines. sparq is row-at-a-time, fork-join, NUMA-blind.
- **Academic RDF (RDF-3X, TriAD, Trinity.RDF):** TriAD uses asynchronous
  inter-node message passing + shard-based locality; Trinity.RDF graph-parallel
  exploration. The shared lesson: **data-placement-aware scheduling**, which sparq
  lacks entirely.

### 3.8 Measurement methodology (you MUST validate on the target)
The M1 cannot reveal B0/B2/B7. Recommend:
- **Hardware:** AWS `m7a`/`c7a` (AMD EPYC Genoa, up to 192 vCPU, multi-NUMA) or
  `c7g`/`m7g`/`r8g` (Graviton, single-socket many-core) for an ARM datapoint, and
  a **bare-metal** `*.metal` (e.g. `m7a.metal-48xl`) so NUMA topology and pinning
  are real (not hypervisor-flattened). A 2-socket bare-metal box is essential to
  see cross-socket effects.
- **Harness:** extend `sparq-bench` (which today has *no* thread-count sweep —
  `main.rs` only varies `--scale`/`--iters`) with a sweep over
  `RAYON_NUM_THREADS ∈ {1,2,4,8,16,32,64,96,128,196}`, reporting **per-subsystem
  speedup AND parallel efficiency** (= speedup / threads). Subsystems: parse,
  index build, RDFS sweep, OWL fixpoint, scan, hash_join, group_aggregate, sort,
  JSON serialize. Add `numactl --membind`/`--cpunodebind` variants and an
  allocator A/B (system vs mimalloc). Plot efficiency vs threads — the knee
  identifies the binding bottleneck per subsystem. Keep the measure-and-keep
  discipline: only land a change if its efficiency curve improves on the target.

---

## PHASE 4 — Prioritized roadmap (scaling-impact × tractability)

Ranked. "Validate on 196-core?" flags whether M1 can confirm the win.

### Tier 0 — Quick architectural wins (days; low risk; mostly hardware-dependent to *measure*)

1. **NUMA placement via pinning + interleave/replication.** *(Biggest win.)*
   Pin rayon workers (`ThreadPoolBuilder::start_handler` + affinity) and
   first-touch-interleave or per-socket-replicate the 6 perms + dict + numeric
   cache. Files: a new pool init in `sparq-engine`/`sparq-cli`; placement at
   `TripleStore::from_triples`/`open` (store.rs) and `NumData` (lib.rs:37).
   Effect: lifts the bandwidth/efficiency plateau from ~1 node to all nodes —
   plausibly the difference between ~10× and ~100×+. Risk: low (read-only data;
   results unchanged). **Must validate on multi-socket bare metal.** Quickest
   probe: run under `numactl --interleave=all` — zero code, measures the ceiling.

2. **Global allocator swap (mimalloc or jemalloc).** One line
   (`#[global_allocator]`), feature-gated OFF for wasm. Targets B2. Effect:
   removes `malloc` lock contention; literature shows 30–50% on alloc-heavy
   many-thread workloads. Risk: negligible (byte-identical results). **Validate on
   196-core** (invisible on M1). Keep only if measured.

3. **Thread-local `LocalVocab` + deferred merge.** Reuse the proven parse-time
   pattern (`ShardedDict`/`merge_remap`/`remap_sharded`) for query-time interning.
   Files: `LocalVocab` (exec.rs:57), `group_aggregate` intern phase
   (exec.rs:2327), `extend_bindings` (exec.rs:2250), `values_bindings`
   (exec.rs:2229). Targets B1 → unblocks parallel BIND/VALUES/aggregate output.
   Effect: removes a 5–15% serial fraction → meaningfully lifts the Amdahl ceiling.
   Risk: medium — must preserve deterministic ids/order for byte-identical results
   (mirror the order-preserving remap invariant already proven in dict.rs tests).
   Partly visible on M1.

### Tier 1 — Radix-partitioned parallel operators (1–2 weeks; medium risk)

4. **Radix-partitioned parallel hash aggregation** (`group_aggregate`,
   exec.rs:2266). Replace serial group-hash build (B5) + keep parallel aggregate.
   This is *the* fix for the measured 1.7× GROUP BY. Effect: near-linear group
   build + aggregate. Risk: medium (GROUP BY first-seen `order` determinism —
   re-sort to a canonical order after partitioned build). Partly visible on M1;
   full win needs the target.

5. **Parallel `hash_join` build phase + radix partitioning** (exec.rs:1852).
   Today build is serial, probe parallel. Partition both sides by `hash(key)%P`;
   per-partition parallel build+probe. Also parallelize **`minus`** (exec.rs:2178)
   and **`distinct`** (exec.rs:2409) the same way; copy RDFS's one-shot parallel
   dedup pattern. Effect: removes the serial build prefix on every hash join.
   Risk: medium (unordered output is already allowed for hash join).

6. **Parallelize `apply_filter` compaction & `scan_to_bindings` LIMIT path.**
   The keep-vector is parallel but `retain` (exec.rs:2505) is serial; use a
   parallel prefix-sum compaction. Small but cheap.

### Tier 2 — Inference scaling (1–2 weeks; medium risk)

7. **Parallelize OWL-RL fixpoint commit + per-round joins** (owl.rs:319-729).
   The semi-naive join phases are data-parallel over `delta`; the `all.insert`
   commit (owl.rs:686-722) is the serial barrier — make it a parallel
   partitioned dedup per round (like `dedup_derived`). `Schema::build`/
   `Axioms::build` rebuilt per round are serial scans → cache/incrementalize.
   Effect: OWL inference (today ~serial) gains the RDFS sweep's parallelism. Risk:
   medium-high (fixpoint correctness; sameAs union-find interleaving).

8. **Parallelize RDFS TBox closure + `original` set build** (rdfs.rs:128,305,361).
   Smaller serial fractions; parallelize the set/sort wrappers. Low risk.

### Tier 3 — Deep rewrites (months; high risk; the real ceiling-lifters)

9. **Morsel-driven, push-based, pipelined execution with a NUMA-aware lock-free
   dispatcher** (replaces the per-operator fork-join model across exec.rs).
   Targets B0/B3/B7 together. Effect: this is what gets Leis-style ~30×/32-core →
   ~linear/196-core. Risk: high (rewrites the engine core; must keep byte-identical
   results and the wasm sequential path). Needs the target hardware throughout.

10. **Columnar/vectorized batch operators** (`ColumnBatch` replacing `Vec<Row>`).
    Targets B3 (bandwidth/SIMD) and pairs with #9 and per-worker arenas (#2-arena).
    Risk: high (touches every operator + serialization). Do after morsels prove out.

11. **Parallel WCOJ / merge / outer / bind joins** (exec.rs:1417,1771,2091,1944).
    Today fully serial → ~1× on graph-shaped queries. Parallelize LFTJ by
    partitioning the top variable's domain across workers (independent subtrees);
    parallelize merge joins by co-partitioning the sorted inputs. Risk: high
    (LFTJ recursion + ordered output). High impact for graph workloads the GROUP BY
    benchmark never touches.

### Constraints honored throughout
- **Byte-identical results** + external-vs-in-memory byte-identity: every Tier 1+
  item must re-impose deterministic order (sort or order-preserving remap) — the
  invariants already enforced by the dict/sharded tests and the JSON
  chunk-ordering code (exec.rs:271 "Chunks stay in order → identical bytes").
- **WASM:** rayon is feature-gated `parallel` (off for wasm); all proposals stay
  behind `#[cfg(feature = "parallel")]` with the existing sequential fallbacks.
  The allocator swap and any `libnuma` calls must be excluded from `wasm32`.
- **Measure-and-keep:** land only changes whose per-subsystem efficiency curve
  improves *on the target hardware* — which is why building the thread-sweep
  harness (3.8) and provisioning a 2-socket bare-metal instance are prerequisites
  for validating Tier 0–3.

### Honesty about speculation
- **Proven/low-risk:** allocator swap, thread-local vocab (same pattern already in
  the codebase), radix-partitioned aggregation/join (standard DB technique).
- **Strongly expected but hardware-gated:** NUMA placement/replication win
  (cannot be confirmed on M1; magnitude depends on dataset fit per node).
- **Speculative until prototyped:** the exact speedup of the morsel/columnar
  rewrite for *this* row-model engine, and parallel-LFTJ load balance under skew.
  These are the highest-ceiling but least-certain items — prototype + measure
  before committing.

---

## ADDENDUM — re-review (Fable 5, 2026-06-10)

A fresh-model re-review of this analysis against the code, after the first Tier-0 items landed.
What it confirmed, corrected, and re-prioritized:

**Now stale (done since the original analysis):**
- Tier-0 #2 (allocator): mimalloc is registered in `sparq-cli` (commit 5fed087) **and now
  `sparq-server`** (the re-review found the server binary had been missed — the long-running,
  concurrent-query process that needs it most).
- §3.8 (harness): `sparq-cli scaling` now sweeps per-subsystem parallel efficiency over thread
  counts (T8); the original "no thread-count sweep" claim no longer holds.

**Corrections to the original analysis:**
1. **B1's fix is much simpler than proposed.** The original prescribed a sharded/thread-local
   vocab with id-range partitioning + deferred merge (high complexity, id-determinism risk). But
   `value_to_id` is *mostly read-only*: small integers inline into the id with **zero** vocab
   access (`try_inline` in `dict::lookup`), and graph-dict / already-local terms are read-only
   lookups. The landed fix (`value_to_id_readonly`, commit 3ab301d) does a parallel resolve pass
   and serially interns only the genuine misses, in row order → **byte-identical ids with no new
   id-space machinery**. Measured on M1 @8 threads: BIND-new-strings 1.92×, BIND-numeric 1.48×,
   GROUP_CONCAT 1.16×, COUNT 1.06×. The residual serial fraction is map-inserts of *novel terms
   only*, so the win grows with core count. The sharded-vocab design remains the escalation path
   only if the NUMA box shows the miss-intern residue matters at 196 threads.
2. **"Allocator contention is invisible on M1" was wrong.** mimalloc measured **+29% on the
   parallel join at just 8 threads** (5fed087). This *strengthens* B2: if arena contention is
   visible at 8 threads, it would have been brutal at 196. Already banked.
3. **`numactl --interleave=all` caveat** (§3.2/Tier-0 #1): interleaving is a *ceiling probe*, not
   a fix — it raises aggregate bandwidth across controllers but makes ~(N−1)/N of accesses remote.
   A flat→improved efficiency curve under interleave confirms B0 is binding; the real fix is
   still pinning + node-local placement/replication. (The original text implies this but reads as
   if interleave were remediation.)
4. **Single-socket many-core is a cheaper first validation step than 2-socket bare metal.** A
   Graviton `c8g/r8g.24xlarge` (96 vCPU, one socket, uniform memory) isolates B1/B2/B5/B7 from
   NUMA entirely — if efficiency collapses *there*, the algorithmic serial fractions (not B0) are
   binding and can be fixed before paying for the 2-socket box. Recommend sequencing the ≤$10
   budget: ~1h single-socket 96-vCPU sweep first, then the 2-socket metal run for B0.

**Updated Tier-0/1 priority (post-landings):**
remaining highest-value, in order — (a) radix-partitioned group build + hash-join build (Tier-1
# 4/#5: with B1 largely gone, the serial group-hash build at `exec.rs` group_aggregate and the
serial hash_join build are now the dominant *algorithmic* serial fractions); (b) parallel
`distinct`/`minus` via the same partitioning; (c) NUMA placement (gated on hardware, unchanged);
(d) morsels/columnar (Tier-3, unchanged — only after the above prove insufficient on target).

## Sources
- Leis et al., *Morsel-Driven Parallelism: A NUMA-Aware Query Evaluation Framework
  for the Many-Core Age*, SIGMOD 2014 —
  https://15721.courses.cs.cmu.edu/spring2016/papers/p743-leis.pdf
- Balkesen et al., *Multi-Core, Main-Memory Joins: Sort vs. Hash Revisited*, VLDB —
  http://www.vldb.org/pvldb/vol7/p85-balkesen.pdf
- Lang et al., *Massively Parallel NUMA-aware Hash Joins* —
  https://15721.courses.cs.cmu.edu/spring2020/papers/17-hashjoins/lang-imdm2013.pdf
- *Scalable Parallelization of RDF Joins on Multicore Architectures*, EDBT 2019 —
  https://openproceedings.org/2019/conf/edbt/EDBT19_paper_143.pdf
- *Toward Efficient In-memory Data Analytics on NUMA Systems* (incl. radix NUMA
  aggregation) — https://arxiv.org/pdf/1908.01860
- QLever — https://ad-publications.cs.uni-freiburg.de/CIKM_qlever_BB_2017.pdf
- mimalloc/jemalloc many-thread arena contention —
  https://nickb.dev/blog/default-musl-allocator-considered-harmful-to-performance/
