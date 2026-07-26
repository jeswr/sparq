<!-- [OPUS-5] sq-rbh10 (GitHub #3241) — EL Phase E4 "concurrent lock-free saturation
(ELK §6 contexts + AtomicBool + work-stealing)". DESIGN-FOR-REVIEW ONLY — no production
code changed by this record. Corrects the bead's premise: E4 is PARTIALLY landed already
(sq-wy3i6 shipped a bulk-synchronous compute-parallel engine behind the `par` feature);
this record measures that engine, shows why it is Amdahl-capped, and specifies what the
ELK §6 architecture would actually have to change. -->

> 🤖 **SPARQ agent** — design record for @jeswr's review. No implementation; every number
> below is a NON-CANONICAL work-box measurement, labelled as such.

# EL Phase E4 — concurrent lock-free saturation (ELK §6 contexts): measurement + design

**Status:** DESIGN-FOR-REVIEW / measurement record. **Bead:** sq-rbh10 (#3241).
**Crate:** `crates/sparq-reason-el` (opt-in `par` feature).
**Predecessors:** sq-wy3i6 (E4 bulk-synchronous parallel saturation, commit `2894b66e`),
sq-q0o82 (E4 follow-up: the compute/apply attribution seam, #3976).
**Spike this phase comes from:** `research/owl2-el-ql-reasoning-spike.md` §5, EL-track E4.

**Conclusion in one line:** the landed BSP engine parallelises the *cheap* half of
saturation — on a link-free taxonomy shape **~80% of saturation time is in the sequential
apply phase**, so end-to-end speedup measured ≈ **1.0×** at 2 and 4 threads — and the only
way past that ceiling is exactly ELK's move (make the dedup-insert itself context-local and
parallel); **but** before spending that concurrency budget, a **single-threaded indexing
defect** found while measuring (the CR4/CR5 membership arm scans *every role* for *every*
derived membership — measured **~26× compute inflation** from 256 inert roles) must be fixed,
because it both wastes more time than E4 can win back and changes which phase dominates.

---

## 0. Premise check (honesty first — what the bead got right and what it got wrong)

| Claim in bead sq-rbh10 / #3241 | Verdict | Evidence |
|---|---|---|
| E4 = "concurrent lock-free saturation (ELK §6 contexts + `AtomicBool` + work-stealing)" is the outstanding work | **PARTLY WRONG — corrected** | A *parallel* E4 engine already landed (sq-wy3i6): `par` feature, `Classifier::classify_par` / `classify_graph_par`, `crates/sparq-reason-el/src/classify.rs:745-1092`. It is **bulk-synchronous**, not ELK-style: `saturate_par_inner` alternates a parallel COMPUTE phase with a **sequential APPLY** phase. There are **no contexts, no `AtomicBool`, and no work-stealing** anywhere in the crate: `AtomicBool` occurs nowhere in `crates/sparq-reason-el`, and the single occurrence of "work-stealing" is `examples/par_phase_bench.rs:43`, which names it as the *hypothetical* refinement this record is about. So the *literal* E4 architecture is genuinely outstanding — the bead is not a duplicate — but it is a **refinement of a landed engine**, not a greenfield phase. |
| "Optional multiplier; sub-linear, larger ontologies benefit most" | **UNVERIFIED as stated, and misleading for the landed engine** | Measured below: on the shapes this repo can generate, the landed engine's multiplier is ≈1.0× because it does not parallelise the dominant phase. "Larger ontologies benefit most" is a property of the ELK design, not of BSP-compute-only. |
| "Depends on E3" | **TRUE** | E3 (`hasse`, sq-s2nob) and the RBox E2 (`rbox`, sq-xetf7) are landed; `par` composes with both. |
| Acceptance: "same subsumption-count correctness vs ELK as E3" | **ALREADY SATISFIED by the landed engine** | Closure identity at every thread count is pinned by `crates/sparq-reason-el/tests/par_differential.rs` and the `el-suite-par` conformance differential; `examples/par_phase_bench.rs` re-asserts it per run. |
| Acceptance: "a measured (non-canonical) speedup on a large ontology" | **NOT SATISFIABLE IN-REPO TODAY** | This repo vendors no large ontology; the only large-ontology path is the external gather harness `scripts/bench/reason-el-same-box.sh` (see `research/gap-reason-el-2026-07.md`). `examples/par_phase_bench.rs` says so in its own doc comment: the real-ontology row "needs an ontology this repo does not vendor". Any claim of a large-ontology speedup must come from that harness on a quiet box, and would be non-canonical until a dedicated run re-measures it. |

**The correction that matters most.** sq-q0o82 deliberately deferred the "should apply be
parallel too?" decision to a *measurement* rather than a guess, and built
`ParPhaseStats` / `examples/par_phase_bench.rs` to take it. That measurement had never been
run and recorded. **This record runs it.** The answer is unambiguous, and it reframes
sq-rbh10 from "an optional multiplier" into "the only remaining way to get a multiplier at
all — after a cheaper single-threaded fix lands first."

---

## 1. What is actually implemented today

`crates/sparq-reason-el/src/classify.rs`:

* **Sequential engine** (`saturate_inner`, line 199 → `drain`, lines 255-341). A worklist of
  `(X, D)` "D just entered S(X)" pairs; each pop fires CR1/CR2/CR3/CRs1 and the
  membership-triggered half of CR4/CR5, alternated with `cr6_pass`.
* **Parallel engine** (`saturate_par_inner`, lines 888-960), `par` feature only. Per round:
  1. `derive_frontier` (line 990) partitions the frontier across at most `threads` scoped workers
     (`std::thread::scope`; minimum 64 frontier items per worker). Each worker runs
     `derive_chunk` — the same rules as `drain`, but **read-only**: it *emits* conclusions
     against the round-start snapshot instead of applying them.
  2. A **sequential** apply loop walks the per-chunk `Derived` buffers in chunk order and
     inserts through the single-threaded `add` / `add_link` / `add_link_rbox`, so the
     link-triggered half of CR4/CR5 and the whole `rbox` CR10/CR11 closure run unchanged.
* **State** (`Saturation`, lines 161-172): `s: Vec<FxHashSet<Concept>>` plus **global**
  `r_pred: FxHashMap<Role, FxHashMap<Concept, FxHashSet<Concept>>>` and the mirror `r_succ`.
  Plain owned collections, no interior mutability, so `&Saturation` is `Sync` and the scoped
  borrow needs no `unsafe` (the crate is `#![forbid(unsafe_code)]`).

Two properties of the landed engine are load-bearing for everything below, and both hold:

* **Closure identity.** All rules are monotone and bounded, so the fixpoint is the unique
  least fixpoint regardless of thread count, chunk boundaries, or rule order — argued in the
  module doc (`classify.rs:745-818`), pinned by `tests/par_differential.rs`.
* **Output determinism does not depend on scheduling.** `materialize_lattice`
  (`src/lib.rs:442-485`) **sorts** by dict id before emitting. So bit-identical triples
  follow from set-identical closure alone. This is the fact that makes a *non-deterministically
  scheduled* engine (Option D below) acceptable at all — a point worth stating explicitly,
  because "work-stealing breaks reproducibility" is the usual objection and here it does not.

---

## 2. The measurement (the gate sq-q0o82 set)

**Box (NON-CANONICAL — a shared work box, not a quiet canonical instance):** Intel Xeon
Platinum 8573C, **4 vCPU**, 15 GiB RAM, rustc 1.88.0, `--release`, features `par,rbox`.
Wall-clock here is **trend-only** and belongs in no README, SKILL, or site copy. The
deterministic counters (`rounds`, `frontier_items`, `derived_members`, `derived_links`) are
thread-invariant by construction and are the trustworthy columns.

Run as `cargo run -p sparq-reason-el --features par,rbox --release --example par_phase_bench`
(synthetic mode) and in gather mode over generated N-Triples probes described inline.

### 2.1 Probe A — link-free taxonomy: apply dominates

20 000 leaves under 16 branches under a 5-deep chain above `Root` (20 021 input triples).
**Zero** existential axioms, so the apply phase does nothing but `add` (an `FxHashSet`
insert + a queue push) — 140 111 derived memberships, 0 derived links.

| threads | compute_s | apply_s | apply_frac |
|---|---|---|---|
| 1 | 0.000996 | 0.004659 | **0.824** |
| 2 | 0.001350 | 0.005040 | **0.789** |
| 4 | 0.001506 | 0.004869 | **0.764** |

### 2.2 Probe B — the built-in wide-taxonomy + existential-traversal shape

`par_phase_bench`'s own synthetic shape at leaves = 20 000 (220 058 frontier items,
120 017 derived memberships, 40 000 derived links, one role):

| threads | compute_s | apply_s | apply_frac |
|---|---|---|---|
| 1 | 0.003140 | 0.014433 | **0.821** |
| 2 | 0.002801 | 0.015365 | **0.846** |
| 4 | 0.002221 | 0.015198 | **0.873** |

Repeat runs of the compute column (0.00365 / 0.00273 / 0.00236 · 0.00353 / 0.00282 /
0.00216 · 0.00406 / 0.00274 / 0.00236 at 1 / 2 / 4 threads) confirm the split is stable,
not a one-shot artefact.

**Reading.** `apply_frac ≈ 0.80` on *both* shapes, and Probe A shows it is **not** about
`add_link`: with zero links it is still 0.76-0.82. The apply phase is expensive because the
**dedup-insert itself is the expensive operation** — ≈33 ns per `add` (hash + probe + insert
+ queue push) against ≈5.5 ns per frontier item on the compute side. Derivation is an index
lookup and a `Vec` push; the insert is the part that touches a growing hash set. **The
landed engine parallelises the ~17% and serialises the ~83%.**

### 2.3 Probe C — the defect found while measuring: compute is O(#roles × #memberships)

Probe B's shape plus *K inert roles*, each carrying exactly one link (`D_i ⊑ ∃r_i.D_i`) so
each contributes one `r_pred` key and ~0.05% extra derived work:

| K inert roles | frontier_items | compute_s @1 | ns / frontier item | apply_s @1 | apply_frac |
|---|---|---|---|---|---|
| 0 | 220 058 | 0.00372 | 16.9 | 0.01464 | 0.797 |
| 64 | 220 378 | 0.02719 | 123 | 0.01534 | 0.361 |
| 256 | 221 338 | 0.09612 | **434** | 0.01746 | 0.154 |

64 roles that derive essentially nothing inflate compute **7.3×**; 256 roles inflate it
**25.8×**. The cause is structural and is in **both** engines:

```rust
// classify.rs:312 (drain) and classify.rs:1073 (derive_chunk) — per frontier item:
for (&r, preds_by_succ) in &sat.r_pred {
    let Some(preds) = preds_by_succ.get(&x) else { continue };
    if let Some(es) = ix.exists_sub.get(&(r, d)) { /* CR4 */ }
    if d == BOTTOM { /* CR5 */ }
}
```

Every derived membership walks **every role** in `r_pred` and does a hash probe per role.
The cost model predicts the measurement quantitatively: 220 058 items × 257 roles = 56.6 M
probes at the measured 0.0956 s ⇒ **1.69 ns/probe**; the K = 64 row gives 1.90 ns/probe on
the same model. That agreement is why this is reported as a defect rather than a hypothesis.

**The fix is cheap and single-threaded.** CR4 can only fire for a membership `(X, D)` if some
axiom `∃r.D ⊑ E` exists — i.e. only for roles in a **small set determined by `D` alone**.
Adding one index to `AxiomIndex` — `exists_sub_roles: FxHashMap<Concept, Vec<Role>>` (the
roles `r` for which `exists_sub` has any key `(r, D)`) — turns the loop into "look up `D`;
usually `None`; otherwise probe `r_pred[r][x]` for a handful of roles". CR5's
`d == BOTTOM` arm keeps the all-roles walk, but only on the `BOTTOM` membership, which is
rare and already terminal. This is a behaviour-preserving reindexing: same rule instances,
same closure.

*Filed as a follow-up rather than done here* (this record is design-only, and the fix is a
sequential-engine perf change, not E4). It is **P1 in the plan in §6** because it changes
which phase dominates and therefore what E4 is optimising.

### 2.4 End-to-end: what the landed engine actually buys

Saturation-only totals (compute + apply), same probes:

| shape | 1 thread | 2 threads | 4 threads | speedup @4 |
|---|---|---|---|---|
| Probe C, K = 0 (1 role) | 18.6 ms | 18.2 ms | 19.1 ms | **≈ 1.0×** |
| Probe C, K = 256 roles | 113.5 ms | 66.1 ms | 64.9 ms | **≈ 1.75×** |

So the landed BSP engine **does** deliver a real multiplier — but *only* on the shape whose
compute phase is inflated by the §2.3 defect. Fix the defect and the K = 256 row collapses
toward the K = 0 row, where BSP wins nothing. **The measured multiplier is currently a
measurement of the defect, not of the parallelism.** Stating that plainly is the single most
important honest finding in this record.

Amdahl, applied to the post-fix regime: with an 80% sequential fraction, the ceiling is
1/0.8 = **1.25×** at *any* thread count. Even a perfect compute phase cannot reach 1.3×.

---

## 3. Why ELK's architecture is different in kind

**Sourcing caveat (honesty).** Network research tools were not available in this session, so
the ELK description below is reconstructed from (a) the citations already recorded in
`research/owl2-el-ql-reasoning-spike.md` §4 — Kazakov, Krötzsch & Simančík, *"The Incredible
ELK"*, JAR 53(1) 2014, <https://link.springer.com/article/10.1007/s10817-013-9296-3> — and
(b) the author's recollection of that paper's §6 and its ISWC 2011 predecessor. **Every
architectural claim in this section must be re-verified against the primary text before any
of it is implemented**; it is recorded here as a design hypothesis, not as an established
citation. Nothing downstream in this record depends on a *quantitative* ELK claim.

The structural difference is not "ELK uses threads and we use rounds". It is **where the
deduplicating insert happens**:

* sparq's engine has a **global** `S` and a **global** `R`. A conclusion derived by any
  worker must be inserted into shared state, so insertion is pulled out into a sequential
  phase. That is the 80%.
* ELK assigns every conclusion to a **context** — identified by the concept `X` the
  conclusion is *about*. A context owns its own `S(X)`, its own **incoming** links
  (predecessors, the direction CR4/CR5 need), and its own local todo queue. The inference
  rules are formulated so that **all premises of a rule instance live in one context**;
  cross-context effects happen only by *producing* a conclusion into another context's queue
  (CR3 sends to the successor; CR4 sends to each predecessor). Insertion is therefore
  **context-local**, and a worker that owns a context does derivation *and* insertion with no
  shared mutable state and no barrier.
* Mutual exclusion without locks: each context carries an activation flag (`AtomicBool`). A
  producer that pushes into a context's queue attempts to flip the flag `false → true`; the
  thread that wins the flip is the one that puts the context on the shared active-context
  queue. So at most one worker processes a given context at a time, and no worker ever waits.
  Deactivation re-checks the queue after clearing the flag, to close the lost-wakeup race.
* Idle workers take contexts from the shared active queue (this is the "work-stealing" the
  spike names — load balancing across a shared pool of activated contexts, not per-worker
  deque stealing in the Cilk sense; this distinction is one of the things to re-verify).

Three consequences that matter here:

1. **The derive/apply split disappears**, which is precisely the 80% sequential fraction.
2. **The `r_pred` global map disappears** — incoming links move into the context that needs
   them. That *also* eliminates the §2.3 role scan structurally: a context's incoming links
   are already keyed by the context, so there is no all-roles walk. (This is why §2.3 is
   sequenced *before* E4 rather than being subsumed by it: we want the cheap version of that
   win now, and we do not want E4's benefit measured against an artificially slow baseline.)
3. **There is no round barrier**, so tail rounds with a thin frontier stop being a scaling
   floor. The probes above all converge in 4-8 rounds; a real ontology's tail is longer.

---

## 4. Options

### Option A — close sq-rbh10 as superseded; keep the BSP engine

*Cost:* zero. *Benefit:* zero. **Rejected as an honest disposition**, because it would leave
the README's `par` feature description (accurate: "identical closure at every thread count")
implicitly suggesting a performance purpose it does not currently serve on the dominant
shape. If the maintainer chooses A, the honest follow-through is to say in the crate README
that `par` is a *determinism-preserving* parallel engine whose measured benefit is
shape-dependent and currently small — not to leave the reader to infer a multiplier.

### Option B — fix the CR4 role indexing first (single-threaded)

Add `AxiomIndex::exists_sub_roles` and replace the all-roles walk in both `drain` and
`derive_chunk`. No feature gate (it is a pure win in both engines), no new dependency, no
API change, no closure change. Verified by the existing `tests/differential.rs` +
`tests/par_differential.rs` + the `el-suite` conformance differential, plus a new
`par_phase_bench` role-sweep row asserting the *relative* ratio does not grow with role count
(a no-hidden-linear-factor assertion in the style of `snomed_go_scale_bench`, so no hard-coded
timing enters the tree).

**This is the highest value-per-risk change identified by this record**, and it is *not*
concurrency work. Every real ontology with more than a handful of object properties pays the
current cost on every derived membership. (The exact object-property counts of GO / SNOMED CT
/ OpenGALEN are **not verified here** — no network access this session; the same-box harness
`scripts/bench/reason-el-same-box.sh` can report them, and should, before B's win is
characterised as anything more specific than "linear in the number of roles carrying links".)

### Option C — owner-partitioned apply, keeping bulk-synchronous rounds

Keep the round structure; make the **apply** phase parallel by *partitioning ownership of the
`S` rows*. Worker `w` owns a contiguous `chunks_mut` slice of `sat.s`. Round becomes:

1. **Derive** (parallel, as today) but bucket each conclusion `(X, E)` by `owner(X)`.
2. Barrier.
3. **Apply** (parallel): each worker inserts only into rows it owns, from the buckets
   addressed to it, in deterministic (source-worker, then emission) order.

* **No locks, no `unsafe`, no new dependency** — `chunks_mut` gives disjoint `&mut` slices
  under `thread::scope`; buckets are moved, not shared.
* **Determinism is preserved exactly** (deterministic bucket order ⇒ deterministic queue),
  so the current "bit-identical at every thread count" property survives unchanged, and
  `tests/par_differential.rs` keeps its full strength.
* **Links stay sequential in the first cut.** `add_link` / `add_link_rbox` mutate the global
  `r_succ` / `r_pred` and read `S(f)` for an arbitrary `f`, so parallelising them needs the
  link localisation that is Option D's real content. Probe A shows this is acceptable: on a
  link-free shape apply is *entirely* member inserts, and even Probe B's link-bearing shape
  derives 3× more memberships than links.
* **Ceiling:** removes the member-insert half of the 80%. Not a full ELK; the round barrier
  and the sequential link apply remain.
* **Risk:** moderate. The one genuinely new invariant is "a worker only ever writes rows it
  owns", which the type system enforces via `chunks_mut` rather than by review.

### Option D — full ELK §6 contexts + activation flag + shared active-context queue

The real thing. Requires, in safe Rust:

* `struct Context { s: FxHashSet<Concept>, incoming: FxHashMap<Role, FxHashSet<Concept>>,
  outgoing: …, todo: Vec<Conclusion>, active: AtomicBool }` and `contexts: Vec<Mutex<Context>>`
  (or `Vec<Context>` behind a per-worker ownership scheme). **`forbid(unsafe_code)` forces a
  `Mutex` where ELK's Java relies on the activation flag alone** — but the mutex is taken
  **once per activation**, not per insert, and is uncontended by the flag's guarantee, so the
  amortised cost is a handful of nanoseconds per activation against hundreds of inserts.
  This is the key feasibility finding: the crate's no-`unsafe` rule does **not** block D.
* Re-keying links into contexts (the `r_pred` elimination from §3).
* A shared active-context queue. `Mutex<VecDeque<ContextId>>` is adequate to start; genuine
  per-worker work-stealing deques would need a new dependency (`crossbeam-deque`; `rayon` is
  already a workspace dependency but its API is not a natural fit for an activation-driven
  queue) — **defer that dependency until a measurement says the shared queue is the
  bottleneck.**
* Termination detection (an `AtomicUsize` of outstanding activations, or a barrier-and-recheck
  round at quiescence).
* CR6 (safe nominals) stays a between-fixpoint sequential pass, exactly as in both current
  engines — its `S(X) := S(X) ∪ S(Y)` merge crosses contexts by construction.

* **Ceiling:** the whole saturation becomes parallel; scaling is bounded by context
  granularity and memory bandwidth, which is where the spike's "larger ontologies benefit
  most" claim actually comes from.
* **Risk:** high, and it is *not* correctness risk in the usual sense — the least-fixpoint
  argument plus the sorted emission (§1) mean the *output* stays bit-identical even under a
  non-deterministic schedule. The risk is (a) a lost-wakeup bug in the activation protocol
  silently losing a rule firing (produces an *incomplete* hierarchy — a silent wrong answer,
  the worst failure mode this crate has), and (b) a large rewrite of the state layout that
  the `abox`, `cdomain`, `hasse`, and `incremental` features all read.

**Mitigation for (a) — mandatory, not optional.** Any D implementation ships with a
*schedule-adversarial* differential: run the concurrent engine N times at several thread
counts against the sequential closure on the whole fixture corpus, plus a deliberately
pathological scheduler (single-item chunks, randomised context order) so a lost wakeup shows
up as a missing subsumption rather than as an occasional flake in CI. `tests/par_differential.rs`
already has the shape; it would need the stress dimension.

---

## 5. Recommendation

**Sequence B → C → (measure) → D, and do not start D until B and C have been measured.**

1. **B first.** It is a small, feature-flag-free, closure-preserving reindex that removes a
   cost every multi-role ontology pays on every derived membership, and it is a prerequisite
   for measuring E4 honestly — today's 1.75× "parallel speedup" is largely a measurement of
   the defect B removes.
2. **C second.** It captures the majority of the available parallel win (the member-insert
   half of an 80% sequential fraction) while **preserving the exact determinism property the
   crate currently advertises**, needs no new dependency, no `unsafe`, and no state-layout
   rewrite that the four other feature gates would have to follow.
3. **Then measure on a real ontology** via `scripts/bench/reason-el-same-box.sh` + the
   `par_phase_bench` gather mode. Only if apply *still* dominates, or if the round barrier
   shows up as a tail-latency floor, does D's risk buy anything.
4. **D last, and only on evidence.** It is the architecture the spike names and the one that
   scales; it is also a rewrite of the saturation state that `abox` / `cdomain` / `hasse` /
   `incremental` all sit on. It should not be spent speculatively.

**On the bead's acceptance criterion.** "A measured (non-canonical) speedup on a large
ontology" cannot be met inside this repo (§0). The acceptance for each phase below is
therefore written as: closure identity (in-repo, hard) **plus** a same-box gather row
recorded in a gap record (external, non-canonical, explicitly flagged) — the same discipline
`research/gap-reason-el-2026-07.md` already established for the ELK comparison.

---

## 6. Phased plan (each phase = a future bead; ordered)

1. **P1 — CR4 role indexing (`exists_sub_roles`).** Option B. `crates/sparq-reason-el/src/classify.rs`
   only; both engines; no feature gate; no API change. **Gate:** existing differential +
   conformance suites green in both feature states; a `par_phase_bench` role-sweep row
   asserting compute time per frontier item does **not** grow with inert role count (relative
   assertion, no hard-coded timing). *Depends on: nothing.*
2. **P2 — owner-partitioned parallel apply (`par`).** Option C. Bucket-by-owner in
   `derive_frontier`, `chunks_mut` apply, links still sequential. **Gate:**
   `tests/par_differential.rs` unchanged and still passing (bit-identical output at every
   thread count), `el-suite-par` green, a new determinism-stress case at 1/2/4/8 threads.
   *Depends on: P1 (so the measurement it is justified by is not defect-inflated).*
3. **P3 — real-ontology phase attribution + speedup row.** Run the gather mode over a
   riot-converted GO / OpenGALEN / SNOMED dump through `scripts/bench/reason-el-same-box.sh`;
   record `apply_frac` and the thread sweep in a gap record, flagged non-canonical, with the
   subsumption-count agreement against ELK reported *before* any timing row (the invariant
   `research/gap-reason-el-2026-07.md` §1 already sets). **This is the bead that decides
   whether P4 happens.** *Depends on: P2.*
4. **P4 — ELK §6 contexts + activation flag (`par`).** Option D, gated on P3's evidence.
   Context-local `S` + incoming links, `AtomicBool` activation, shared active-context queue,
   termination detection; CR6 stays a sequential between-fixpoint pass. **Gate:** the
   schedule-adversarial differential from §4 (randomised context order + single-item chunks,
   N repeats × several thread counts) plus everything P2 gates on. *Depends on: P3.*
5. **P5 — (only if P4's queue is measured as the bottleneck) per-worker work-stealing deques.**
   Adds a `crossbeam-deque` dependency behind `par`; justified by a measurement, never by the
   architecture diagram. *Depends on: P4.*

Dependency edges: P1 → P2 → P3 → P4 → P5. P1 is independently valuable and should land
regardless of whether the maintainer greenlights the rest.

---

## 7. Open questions for the maintainer

1. **Is E4 worth the concurrency budget at all?** After P1 the single-threaded engine gets
   materially faster on multi-role ontologies. If EL classification is not on a latency
   critical path for any real sparq use case, the honest answer may be "land P1, close
   sq-rbh10 as Option A with the README wording fix", and spend the budget elsewhere.
2. **Determinism: hard requirement or nice-to-have?** C preserves bit-identical output *by
   construction*; D preserves it only *via the sorted emission* (§1) and the least-fixpoint
   argument, which is a weaker guarantee to defend in review even though it is sound. If
   bit-identical-by-construction is a hard requirement, the plan stops at C.
3. **Is a new dependency (`crossbeam-deque`) acceptable under `par`** if P4's shared queue
   measures as the bottleneck? The crate today has zero concurrency dependencies and the
   default build is wasm-safe; `par` is already native-only, so the blast radius is contained.
4. **Which large ontology should P3 use as the canonical shape?** GO is the smallest useful
   one; OpenGALEN stresses roles (which is exactly what P1 changes); SNOMED CT has licence
   constraints that affect whether the row can be published.

---

## 8. What this record does NOT claim

* **No canonical performance numbers.** Every figure in §2 is work-box wall-clock on a shared
  4-vCPU box, reported as ratios and trends. None of it belongs in a README, `SKILL.md`, the
  site, or a release note, and none of it is a target.
* **No claim about ELK's own performance**, and no comparison to it. The only ELK cross-check
  this crate makes is the **subsumption-count oracle** (`research/gap-reason-el-2026-07.md`).
* **No verified claim about ELK's internal architecture.** §3 is a design hypothesis
  reconstructed without access to the primary text this session, flagged as such, and must be
  re-verified before P4 is designed in detail.
* **No claim that the landed `par` engine is wrong.** It is correct, it is well-tested, its
  closure identity is pinned, and its README description is accurate. The finding is that it
  parallelises the minority phase — which is exactly what sq-q0o82 was built to find out.
* **No implementation.** Nothing in `crates/` was changed by this record.

---

## 9. Appendix — reproducing the probes

Probe B is `par_phase_bench`'s own synthetic mode. Probes A and C are gather-mode runs over
N-Triples generated by the script below (written to `/tmp`, never into the repo — this record
adds no fixtures). Nothing here is a fixture to keep; it is recorded so the §2 rows can be
re-taken or contradicted.

```bash
cargo build -p sparq-reason-el --features par,rbox --release --example par_phase_bench
# Probe B — the built-in synthetic wide taxonomy + existential traversal:
./target/release/examples/par_phase_bench 20000
# Probes A and C — generated below, then:
./target/release/examples/par_phase_bench /tmp/el_pure_taxonomy.nt ntriples   # Probe A
./target/release/examples/par_phase_bench /tmp/el_roles_256.nt     ntriples   # Probe C, K=256
```

```python
EX = "http://sparq.dev/bench/el-par#"
SC = "<http://www.w3.org/2000/01/rdf-schema#subClassOf>"
OP = "<http://www.w3.org/2002/07/owl#onProperty>"
SV = "<http://www.w3.org/2002/07/owl#someValuesFrom>"
LEAVES, BRANCHES = 20_000, 16
def iri(n): return f"<{EX}{n}>"

# Probe A — link-free taxonomy with a 5-deep chain above Root (no existential axioms).
out = [f"{iri('M%d' % b)} {SC} {iri('Root')} ." for b in range(BRANCHES)]
prev = "Root"
for c in range(1, 6):
    out.append(f"{iri(prev)} {SC} {iri('C%d' % c)} .")
    prev = "C%d" % c
out += [f"{iri('L%d' % k)} {SC} {iri('M%d' % (k % BRANCHES))} ." for k in range(LEAVES)]
open("/tmp/el_pure_taxonomy.nt", "w").write("\n".join(out) + "\n")

# Probe C — Probe B's shape plus K inert roles, each carrying exactly one link.
def roles_probe(K, path):
    out = [f"{iri('M%d' % b)} {SC} {iri('Root')} ." for b in range(BRANCHES)]
    for k in range(LEAVES):
        m, rn = "M%d" % (k % BRANCHES), "__restr_%d" % k
        out += [f"{iri('L%d' % k)} {SC} {iri(m)} .",
                f"{iri('L%d' % k)} {SC} {iri(rn)} .",
                f"{iri(rn)} {OP} {iri('r0')} .",
                f"{iri(rn)} {SV} {iri(m)} ."]
    out += [f"{iri('__restr_root')} {OP} {iri('r0')} .",
            f"{iri('__restr_root')} {SV} {iri('Root')} .",
            f"{iri('__restr_root')} {SC} {iri('Marked')} ."]
    for i in range(K):                       # one inert role each: D_i subClassOf exists r_di.D_i
        out += [f"{iri('D%d' % i)} {SC} {iri('__d_restr_%d' % i)} .",
                f"{iri('__d_restr_%d' % i)} {OP} {iri('r_d%d' % i)} .",
                f"{iri('__d_restr_%d' % i)} {SV} {iri('D%d' % i)} ."]
    open(path, "w").write("\n".join(out) + "\n")

for K in (0, 64, 256):
    roles_probe(K, f"/tmp/el_roles_{K}.nt")
```
