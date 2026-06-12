# ADR — Horizontal scaling of sparq across AWS nodes

**Status: PROPOSED — awaiting user sign-off.** Date: 2026-06-12.
Builds on `research/concurrent-serving.md` (single-node serving design: generation ring,
single sequenced writer + group commit, tiered scheduler, response cache keyed on
(canonical-query hash, visibility-scope hash, per-pod epoch vector), library-first
`sparq-serve` crate), `research/ci-ec2-design.md` (cost-capped EC2 CI), and — read-only —
`prod-solid-server` ADRs 0003 (QLever update contract) and 0012 (app-tier horizontal
scaling). **Design only; no implementation in this wave.**

---

## 1. Context

sparq's deployment seat is the SPARQL engine inside a Solid pod server. prod-solid-server
ADR-0012 already scaled the *application tier* (stateless Node replicas + Redis
coordination) and explicitly punted the engine: *"QLever single-writer assumption …
Multi-region active/active needs QLever replication or graph partitioning. Out of
scope."* This ADR is that missing piece, for sparq in QLever's seat.

The workload (measured/derived in `concurrent-serving.md`):

- **Read-heavy, cacheable, visibility-scoped.** Named graph per resource; pods are
  independently owned datasets; queries run under a per-(session,mode) accessible-graph
  set. ~69 % of real SPARQL endpoint traffic is exact-duplicate queries (Bonifati et al.,
  PVLDB 2017) — the cache-hit path is the throughput story.
- **Writes are small and slow-arriving**: one SPARQL UPDATE per resource write,
  ~300–500 bytes, 8–9 triples. A *very large* deployment is hundreds of updates/s; one
  sparq node sustains **1.1–2.5 K updates/s with flat RSS** (measured, M1). Writes do
  not need horizontal scaling at any foreseeable tenancy.
- **Single-node envelope** (M1, loadgen co-located — conservative): point queries
  **0.93 M ops/s in-process @ 8 threads**; cache hits 23.8 M ops/s on an arc-swapped
  map; today's HTTP server **22–25 K req/s** (the gap is single-node server work —
  sparq-serve waves A–D — *not* a sharding problem). Targets of millions of req/s are a
  **cache-hit-path property** of a single node; horizontal scaling is needed for: HA,
  working sets past one node's RAM, NIC-bound regimes past one box, miss streams beyond
  ~1 M point-queries/s, and isolating analytical load.

**Honest framing up front:** at prod-solid-server's actual current scale (single
instance, ≤ hundreds of writes/s), a 3-node sparq deployment buys **availability and
isolation, not throughput**. The throughput case arrives with tenancy growth (RAM) and
public/cacheable traffic (NIC). The design below must therefore be cheap to *not* use:
stage 1 costs nothing in code and the single-node path stays the supported default,
exactly as ADR-0012's `PSS_COORDINATION_BACKEND=none` keeps single-instance first-class.

## 2. Decision drivers

1. **Compose with the single-node design, don't contradict it.** `concurrent-serving.md`
   §6.8 pre-positioned the hooks: immutable generations are shippable units; the
   single-writer update log is deterministic and replayable; per-pod epochs partition
   cleanly; the cache key (query, scope, epoch-vector) is node-independent.
2. **Pods are the natural partition.** Independently owned, no cross-pod transactions in
   the write contract (every UPDATE touches one resource graph + its parent container
   graph, both in the same pod), access control is per-pod.
3. **AWS**, matching prod-solid-server's posture (ECS-on-EC2, ALB, ElastiCache,
   EventBridge, S3).
4. **Cost discipline**: any *test* infrastructure stays **< $5/month** (the
   `ci-ec2-design.md` OIDC + spot + tag-scoped pattern); ad-hoc experiments ~$5/day;
   small-deployment reference ≤ low hundreds of $/month.
5. **Tail latency**: fan-out is the enemy — at 1-in-100 slow per server, touching 100
   servers per request makes **63 %** of requests slow (Dean & Barroso, CACM 2013).
   Prefer architectures where one request touches one node.
6. **Contract compatibility**: single POST endpoint, Content-Type dispatch, Bearer-gated
   updates, 45 s timeout + 3 retries (ADR-0003) — client-visible behaviour unchanged.

## 3. Candidate architectures

### Option A — Pod-sharded shared-nothing (consistent/rendezvous hashing)

Each node runs a full single-node sparq; pods are assigned to shards by rendezvous
hashing (Thaler & Ravishankar 1998) or a small explicit routing table; a request is
routed to the shard owning its pod(s). No replication; each shard is its pods' writer.

- **For:** linear capacity in both RAM and CPU; zero cross-node coordination on the hot
  path; single-shard routing keeps tail = single-node tail (driver 5); the write
  contract never spans shards (driver 2). Stage-1 form requires **zero sparq code
  changes** — it is N independent single-node servers plus routing.
- **Against:** no HA — a node loss makes its pods unavailable (and loses data unless the
  log/snapshots are durable); queries whose visibility scope spans shards (e.g. a public
  query over all pods) cannot be answered by one node; rebalancing moves data.
- **Precedent:** Virtuoso Cluster partitions indexes by hash across nodes (commercial,
  operationally heavy); Blazegraph's scale-out mode existed but was rarely deployed and
  effectively froze when the team moved to Amazon — the graph-store industry's revealed
  preference at our scale is *not* fine-grained sharding. Pod-granular sharding is much
  coarser and matches the multi-tenant SaaS pattern (tenant-per-shard) rather than
  distributed-join systems. (Judgement beyond the cited facts.)

### Option B — Read-replica fan-out, single writer per shard

Primary applies the sequenced update log; replicas replay it and serve reads; reads go
to any replica, writes to the primary.

- **For:** HA and read scaling without touching the write design — the single-node
  group-commit log **is already** the replication stream (deterministic, replayable —
  Calvin, SIGMOD 2012 / Bohm, PVLDB 2015, collapsed to one sequencer). Epoch bumps ride
  the same log, so cache invalidation on replicas is causal and exact. Replicas also
  give analytical isolation for free (BatchDB, SIGMOD 2017: isolate, don't share).
- **Against:** does not scale RAM (every replica holds the full dataset) or writes;
  replication lag ⇒ bounded staleness for reads (needs a read-your-writes story for
  pod owners).
- **Precedent:** this is how production SPARQL actually scales today: **WDQS runs
  full Blazegraph replicas behind an LB with lag-based `maxlag` back-pressure**
  (Malyshev et al., ISWC 2018; Wikimedia T221774); **Amazon Neptune** is a single
  writer + up to 15 read replicas over a shared distributed log-structured volume
  (Aurora's "the log is the database", SIGMOD 2017/2018) — no sharding at all. QLever
  and Oxigraph ship no distributed mode (single node + external LB is their story).

### Option C — Disaggregated storage (S3) + stateless query nodes

Generations/snapshots live in S3; query nodes are stateless, pull snapshots + log tail,
serve from local memory; writer is a small stateful sequencer writing the log to S3.

- **For:** elastic query fleet; trivially durable (RPO ≈ one log flush); cheap idle
  state; node replacement = re-hydrate from S3. The *snapshot + log-archive* half of
  this is valuable regardless of topology.
- **Against (for the hot path):** sparq is an in-memory engine — a "stateless" node
  still holds the working set in RAM, so S3 buys durability, not memory elasticity;
  S3 GET/PUT latency is ~10s of ms + hydration of a multi-GB image takes minutes, so
  scale-out reaction time is minutes, not seconds; per-request S3 access is two to
  three orders of magnitude off the 9 µs point-query budget. Neptune makes this work
  with a purpose-built page-served storage fleet (6-way replicated, sub-ms) — S3 is not
  that. **Verdict: adopt S3 as the durability/bootstrap plane (stage 2), reject it as
  the query-path storage.** (Latency figures: AWS-documented orders of magnitude;
  hydration estimate is judgement.)

### Option D — What ADR-0012 chose: shared coordination plane, engine as singleton

Redis-coordinated stateless app replicas in front of **one** engine. Correct for the app
tier; by construction it does not scale the engine — 0012 itself lists the engine as the
honest gap. **Not a candidate for sparq itself**, but its *patterns* are adopted
wholesale for our control plane: TTL-lease leadership (0012 C5), one coordination
backend, preflight consistency checks, staged opt-in with `none` as default.

### Comparison

| | A: pod-sharded | B: replicas | C: S3-disaggregated | D: 0012-style |
|---|---|---|---|---|
| RAM scaling | **linear** | none | none (hot set in RAM anyway) | none |
| Read scaling | linear | **linear** | linear (slow elasticity) | app tier only |
| Write scaling | linear (unneeded) | none (unneeded) | none | none |
| HA | none alone | **yes** | yes | no (engine SPOF) |
| Hot-path coordination | none | none (async log) | S3 on miss | Redis on auth path |
| Tail risk | low (1 node/req) | low | medium | low |
| Whole-dataset queries | broken across shards | fine | fine | fine |
| Code delta to sparq | ~0 (stage 1) | log shipping + catch-up | + snapshot/log to S3 | n/a |

## 4. Decision (recommended): A + B composed, S3 as the durability plane

**Pod-sharded shards; each shard = one sequenced writer + 0..N read replicas replaying
the shard's deterministic update log; rendezvous hashing for pod→shard assignment;
snapshots + log archived to S3.** A 3-node minimum deployment is **one shard, three
replicas** (HA before sharding — matching where the actual bottleneck isn't). Sharding
activates only when RAM or NIC pressure demands it.

### 4.1 Data partitioning

- **Shard key = pod IRI** (the pod root; all graphs under it co-locate — every ADR-0003
  update shape touches only graphs within one pod, so writes never span shards).
- Assignment: rendezvous hashing over a **versioned routing table** (epoch-numbered,
  stored in the control plane; v1 may be a static config file). Rendezvous over
  consistent-ring: simpler, no virtual-node tuning, minimal movement on
  membership change. (Judgement.)
- Each shard has its **own term dictionary**. Cross-shard joins are out of scope (§4.7),
  so value-id spaces never need to align. Cost honestly stated: moving a pod between
  shards re-encodes its terms (pods are small — the Solid fixture averages KBs–MBs per
  pod — so this is per-pod milliseconds, judgement).

### 4.2 Request routing

- **Queries:** route by visibility footprint. The auth layer already computes the
  accessible-graph set per (session, mode); its pods map to shards. Single-shard
  footprint (the overwhelmingly common case: owner/app traffic to one pod, public
  reads of one pod) → route to **any replica of that shard**. Multi-shard footprint →
  **rejected with a defined error** in v1 (§4.7).
- **Updates:** Content-Type `application/sparql-update` → the shard **primary** for the
  target pod. Bearer gate unchanged (ADR-0003 Q3).
- **Where the router lives:** v1 = client-side sharding inside prod-solid-server's
  existing store seam (`S3QLeverStore.apply()` / query dispatch picks an endpoint by pod
  hash — mirrors its `apply()` fallback seam philosophy), fronted by one ALB per shard
  group or host-based ALB rules. No new routing tier; a dedicated thin router is a
  later option, not a v1 requirement. (Open question 3.)

### 4.3 The crux: generation ring + per-pod epochs across nodes

Single-node recap: the writer publishes immutable generations; each commit bumps a
**per-pod epoch counter**; the response cache key is (canonical-query hash,
visibility-scope hash, epoch-vector hash) and an entry is valid iff the epochs of the
pods the query touches still match. Multi-node extension — the load-bearing claim of
this ADR — is that **epochs, not generations, are the unit of distribution**:

1. **Generations stay node-local.** Each replica runs its own generation ring over its
   own arc-swapped store. Nothing about ring mechanics changes.
2. **Epochs become shard-global, assigned at sequencing.** The shard primary's group
   commit emits a log record: `(shard_seq, batch of updates, per-pod epoch bumps)`.
   `shard_seq` totally orders the shard; pod epochs are derived from it
   deterministically. Because replicas **replay the identical log in the identical
   order** (Calvin/Bohm determinism), a replica that has applied through `shard_seq = S`
   has *exactly* the same pod-epoch vector as the primary at S — no separate
   invalidation protocol, no broadcast, no races. **Cross-node cache invalidation is
   epoch propagation, and epoch propagation is the replication stream itself.**
3. **Cache entries are valid wherever they travel.** Since the key contract is
   node-independent (§6.8 of the serving doc), a hit computed on replica 1 is correct on
   replica 2 *at the same epochs*; an entry tagged with stale epochs is a miss
   everywhere. This makes an optional **shared external cache tier** (or
   request-coalescing layer) in front of the shard a drop-in — same keys, no new
   invalidation machinery. Deferred until telemetry justifies it (memcache-style
   look-aside + leases, NSDI 2013, is the blueprint).
4. **Consistency model, stated plainly:** per-shard sequential consistency at the
   primary; **bounded-staleness snapshot reads** at replicas (staleness = replication
   lag, exported as a metric and a `Last-Seq` response header). **Read-your-writes** for
   pod owners: an update ack returns `shard_seq`; the client (prod-solid-server) sends
   it back as a session token; the router pins to the primary or any replica with
   `applied_seq ≥ token`. This is WDQS's lag-tolerance pattern (maxlag) turned from a
   global knob into a precise per-session gate. Auth changes (ACL writes) bump the pod's
   auth epoch **in the same log**, so visibility changes invalidate caches with the
   same causality as data changes — no fail-open window beyond replication lag, and the
   lag gate closes even that for the writing session.

### 4.4 Writer placement and ordering guarantees

- **Exactly one sequenced writer per shard** (the single-node writer, unchanged). All
  ordering guarantees are per-shard: updates to one pod are totally ordered; updates to
  different pods on different shards are causally unordered (acceptable: Solid writes
  are single-resource; no cross-pod transaction exists in the contract).
- **Primary election:** TTL lease in the control plane, exactly ADR-0012 C5's
  Redlock-style `SET NX EX` pattern (reusing prod-solid-server's ElastiCache is the
  cheap option — open question 4). Split-brain guard: the log store (S3 conditional
  PUT / EBS volume attach semantics) is the fencing point — a deposed primary's
  `shard_seq` write fails. v1 may use *static* primary assignment (no election) since
  stage 1–2 deployments are operator-supervised.
- **Durability (open question 2):** group-commit batch is written to the durable log
  *before* ack ⇒ RPO 0 at the cost of per-batch fsync/PUT latency inside the 2–5 ms
  window budget (EBS gp3 fsync ~1 ms is compatible; S3 PUT ~10s of ms means S3-only
  logging implies RPO = one archive interval, not 0). ADR-0003's client already retries
  3× on a 45 s budget, so failover-induced duplicate application must be idempotent —
  the update shapes (`DROP SILENT; INSERT DATA`, `DELETE/INSERT WHERE`) are idempotent
  per-resource at the contract level, and the log's `shard_seq` dedupes exactly.

### 4.5 Failure and rebalance story

- **Replica loss:** routing drops it; capacity degrades; replacement node hydrates from
  the latest S3 snapshot + log tail (snapshot = a shipped generation; the §6.8 hook).
- **Primary loss:** lease expires (≤ TTL, e.g. 10 s); the replica with the highest
  `applied_seq` acquires the lease; writes stall ≤ TTL + catch-up; reads continue
  throughout (replicas keep serving — the availability win over today's single node).
- **Pod rebalance (shard split / move):** epoch-fenced move — (1) mark pod write-frozen
  on source (writes 409/queue briefly), (2) ship the pod's graphs + re-encode into the
  target dictionary, (3) bump the routing-table version, (4) unfreeze on target. Only
  moved pods are affected; rendezvous hashing moves ~1/N of pods on shard addition.
  Freeze window is bounded by pod size (small) — but this is the most operationally
  delicate machinery in the design and is **deliberately last** (stage 4).
- **Blast-radius property worth naming:** a shard failure affects only its pods —
  pod-sharding turns "the SPARQL endpoint is down" into "0.x % of pods are degraded".

### 4.6 Autoscaling triggers

Replica count (fast, minutes): point-lane p99 against SLO; CPU; miss-rate × measured
miss cost (the §1.3 mix arithmetic, computed live from existing metrics); NIC
utilization for cache-hit-heavy traffic. Shard count (slow, operator-approved in v1):
RSS high-water vs node RAM (the real trigger — working set growth); sustained update
rate per shard > ~1 K/s (½ the measured single-node ceiling); analytical-lane
saturation. Scale-in: conservative, one step per cool-down, never below 2 replicas/shard
once HA is the point. (Thresholds are judgement pending the §7 benchmark suite of the
serving doc.)

### 4.7 Deliberately out of scope

- **Cross-shard SPARQL queries** (joins over pods on different shards, all-pods public
  analytics). v1 rejects them with a typed error; the visibility-footprint router makes
  this precise rather than silent. Rationale: scatter-gather brings distributed joins,
  cross-shard dictionaries, and Tail-at-Scale fan-out — the entire cost of Option A's
  "against" column — for a query class the Solid workload makes rare. A future
  aggregator node (scoped scatter-gather over SERVICE-like sub-queries) is the named
  escape hatch, not a v1 deliverable.
- **Federation (SERVICE)**, multi-region active/active, incremental view maintenance,
  cross-shard transactions, geo-routing. All deferred.
- **Engine-internal distribution** (distributing one query's operators across nodes,
  QLever/Virtuoso-cluster style): rejected outright for this workload — point-query
  dominated mixes lose to coordination overhead (same logic that rejected MQO in the
  serving doc).

## 5. Cost model sketch (3-node reference, us-east-1, list-price ballpark — verify before commit)

| Item | Choice | ~$/month |
|---|---|---|
| 3 × query/replica nodes | **r8g.large** (2 vCPU, 16 GiB, Graviton) — RAM-biased because the store is in-memory; step to r8g.xlarge (32 GiB) when RSS says so | ~3 × 80 = **240** (on-demand; ~150 with 1-yr savings plan) |
| Load balancer | 1 ALB + LCUs at small scale | ~20–25 |
| Durable log + snapshots | EBS gp3 50 GiB/node (in node price ~$12 total) + S3 (GBs) | ~15 |
| Control plane | reuse prod-solid-server's ElastiCache t4g.micro (≈$10 already paid there) or static config = $0 | 0–10 |
| **Total** | | **~$275–290 on-demand; ~$185–200 reserved** |

(Instance prices are June-2026 recollection of list pricing — **judgement; re-quote
before sign-off.**) Test infra stays inside the existing `ci-ec2-design.md` envelope:
weekly multi-node tests run as **3 processes on one spot box** (the architecture is
port-addressed, not host-addressed) — $0 marginal; a true 3-spot-node soak is ad-hoc
EC2 at ~$5/day, maintainer-triggered only, tag-scoped, budget-alarmed at $5/month.

## 6. Staged adoption (stage 1 touches no single-node code path)

- **Stage 1 — deployment-level pod sharding (zero sparq changes).** N independent
  single-node sparq servers; prod-solid-server's store seam routes by pod hash
  (client-side sharding behind its existing `apply()`/query dispatch). Each node is
  exactly today's binary. Buys: RAM scaling + blast-radius isolation. Proves: routing,
  footprint classification, ops. Cost of being wrong: redeploy to one node.
- **Stage 2 — durable log + snapshot shipping.** The sparq-serve group-commit batch
  record (§6.5 of the serving doc) gains a serialized on-disk/S3 form; snapshot
  export/import of a generation. Single-node behaviour unchanged when disabled
  (the `none` backend discipline from ADR-0012 C4/C12). Buys: RPO story, node
  replacement, and the replication stream — before any replica exists.
- **Stage 3 — read replicas + epoch tokens.** Replica mode = tail the log, replay,
  serve reads; `Last-Seq`/read-your-writes token through prod-solid-server; primary
  lease. Buys: HA + read scaling. This is the first stage with new distributed
  semantics, and it arrives with the stage-2 log already soaked.
- **Stage 4 — rebalancing + autoscaling.** Epoch-fenced pod moves, routing-table
  versioning, scale triggers. Only if tenancy growth demands it.

Each stage is independently shippable and reversible; stages 2–4 land behind the
sparq-serve library API so `sparq-server` single-node consumers never see them.

## 7. Open questions for sign-off

1. **Cross-shard query rejection acceptable for v1?** Queries whose visibility footprint
   spans shards get a typed error (with the aggregator as a named future). Alternative:
   keep one "global" replica replaying *all* shard logs to serve them (RAM cost = whole
   dataset on one node again).
2. **Durability target:** RPO 0 (fsync-to-EBS inside the commit window, S3 as archive)
   vs RPO = one S3 archive interval (~seconds, simpler, no EBS coupling)? Pick per
   deployment or fix one?
3. **Router placement:** client-side sharding inside prod-solid-server's store seam (my
   recommendation, zero new tiers) vs a dedicated thin routing service?
4. **Control plane:** reuse prod-solid-server's ElastiCache Redis for leases + routing
   table (couples the two systems' ops) vs static config in v1 (no election; manual
   failover) vs sparq-owned coordination?
5. **Replication transport:** S3 log objects polled by replicas (simplest, seconds of
   lag) vs direct streaming (TCP/gRPC, sub-second lag, more machinery)? Lag bound
   directly sets replica staleness.
6. **Cost ceiling:** is ~$275/month on-demand (~$190 reserved) for the 3-node reference
   acceptable, and is "3 processes on one spot box" sufficient as the *recurring* CI
   shape (true multi-node only ad-hoc)?
7. **Read-your-writes scope:** is the epoch-token gate required for *all* authenticated
   reads, or only for the writing session (other readers accept bounded staleness, as
   WDQS does)?
8. **Per-shard dictionaries** (and term re-encoding on pod moves) accepted, in exchange
   for never needing cross-shard value-id alignment?
9. **Stage-1 go-ahead:** may stage 1 (pure deployment sharding, zero engine changes) be
   adopted by prod-solid-server independently of stages 2–4 sign-off?

## 8. Sources

- Bonifati, Martens, Timm, PVLDB 11(2), 2017 — ~69 % duplicate endpoint traffic.
- Malyshev et al., ISWC 2018; Wikimedia T126730/T221774 — WDQS replica fan-out + maxlag.
- Verma et al. / Amazon Neptune docs — single writer + ≤15 replicas, shared log-structured
  storage; Aurora "log is the database" (SIGMOD 2017/2018).
- Thomson et al., Calvin, SIGMOD 2012; Faleiro & Abadi, Bohm, PVLDB 2015 — deterministic
  log replay; readers do zero coordination.
- Nishtala et al., Facebook memcache, NSDI 2013 — look-aside cache, leases.
- Dean & Barroso, The Tail at Scale, CACM 2013 — fan-out tail amplification (63 % figure).
- Thaler & Ravishankar 1998 — rendezvous hashing; DeCandia et al., Dynamo, SOSP 2007 —
  consistent hashing precedent.
- BatchDB, SIGMOD 2017 — workload isolation via replicas.
- Virtuoso Cluster docs; Blazegraph scale-out history; QLever/Oxigraph (no distributed
  mode) — engine precedents.
- prod-solid-server ADR-0003 (update contract), ADR-0012 (coordination patterns, Redis
  lease, `none`-backend discipline, engine gap).
- All measured numbers: `research/concurrent-serving.md` (M1, 2026-06-12). Instance
  pricing and pod-size/rebalance estimates: **judgement, marked inline.**
