# Design — remaining sq-qonbz scope: the OWL-RL semi-naive fixpoint onto the shared substrate join

> 🤖 SPARQ agent — design record produced by Claude Fable (architect tier) for epic **sq-qonbz**.
> [FABLE-5] Companion to `research/shared-eval-substrate.md` (the original extraction design);
> this record covers only what **remains** of the epic. Implementation is delegated to the
> beads in §8; the orchestrator reconciles the model marker on implementation PRs.

**Provenance and tier honesty.** This design was produced at brief tier: from the epic's
prose brief, the live bead state (`sq-qonbz`, `sq-yk6or`, `sq-6w7x6`, `sq-v5evr`), and the
prior design record — deliberately **without reading `crates/` code**. The seam sketch in §4
therefore fixes *decisions and contracts*, not line-level names. The implementing agent must
validate the mapping in §5 against the real `emit_delta` shape in `owl.rs` and may adjust
names/orientations freely; what is **not** negotiable is decision R1 (§3), the determinism
contract (§4.2), and behaviour-neutrality (§6).

## 1. Where the epic actually stands

The substrate crate is built and the **engine side is done**: id-tuple row/key vocabulary,
XSD numeric tower, the four join kernels (merge / hash / bind / leapfrog-trie WCOJ), and the
SPARQL term total-order comparison all live in `sparq-substrate`, and the engine is migrated
onto all four modules (PRs #1290, #1296, #1300, #1303, #1306). The zero-overhead contract
holds throughout: kernels monomorphic over the concrete `Id = u32` and SmallVec row/key
aliases, generic type parameters instead of trait objects at hot callsites, enforced
structurally by the no-dyn-dispatch CI gate (`scripts/check-no-dyn-dispatch.py`).

On the **reasoner side**, sq-yk6or (PR #1301) landed the RDFS single-pass predicate join
(rdfs7/rdfs2/rdfs3) on the shared `build_table` + `probe_emit` + `hash_probe_serial` kernels,
behind the default-off `substrate-join` feature, byte-identical to the plain branch and with
the generic `Budget` cancellation seam from #1300.

**The one remaining gap** between the epic's stated goal (substrate shared by the engine AND
the reasoners) and reality: the OWL-RL closure (`owl.rs` `owl_rl_closure`) is a delta-driven
semi-naive fixpoint — per-round `Δ ⋈ full ∪ full ⋈ Δ` with union-find `sameAs`
canonicalisation — an *incremental, mutating* join shape the static build/probe kernel does
not express, so sq-yk6or correctly kept it on the hand-rolled FxHashMap adjacency and
deferred it to **sq-6w7x6**. sq-6w7x6 is the practical long pole: downstream epic
**sq-pbz04** (full reasoner suite on the substrate) gates on sq-qonbz closing.

## 2. Problem statement

Adopt the shared substrate join inside the OWL-RL semi-naive fixpoint such that:

1. behaviour is **byte-identical** — the OWL-RL ratchet and the 13 documented divergences
   must not move by a byte;
2. `UnionFind` `sameAs` canonicalisation is **kept**, with its existing merge policy;
3. the zero-overhead contract holds — no `Box<dyn>` in hot loops, monomorphic kernels,
   no-dyn-dispatch gate green;
4. performance is **neutral or better** on every input shape the fixpoint can see, including
   adversarial deep-chain ontologies (transitive rules ⇒ round count scales with chain depth).

## 3. Decision R1 — delta-aware seam, not per-round static rebuild

sq-6w7x6 names two candidate framings. Verdict:

**(a) CHOSEN — add a delta-aware build/probe seam to `sparq-substrate::join`.** A persistent
build-side table that is built once, *extended* with each round's Δ, and probed only by Δ.
Per-round cost is O(|Δ| + matches) — the same asymptotics the hand-rolled FxHashMap adjacency
already achieves, so the migration is perf-neutral *by construction* and the review burden
reduces to constant-factor checks.

**(b) REJECTED — frame each round as a static hash join over the current full set.** This
reuses the existing kernel unchanged, which is seductive, but rebuilding the build side from
the full set every round costs O(rounds × |full|). OWL-RL contains transitive/chain rules
(property transitivity, subclass/subproperty chains) where the number of semi-naive rounds
grows with chain depth, so (b) degrades to quadratic on chain-shaped inputs that the current
hand-rolled path handles linearly. That is a perf **regression**, which the epic's contract
forbids. (b) would only be acceptable under a "rounds are few" assumption we cannot make.

This mirrors the epic's original Option-C reasoning: pay a small, one-time seam-design cost
in the leaf crate rather than a recurring runtime cost in every consumer.

## 4. The seam: `sparq-substrate::join` delta module

### 4.1 Shape

A new module alongside the static kernels (working name `join::delta`; final naming is the
implementor's). One new type — a persistent, extendable build-side table:

- **Layout**: a `Vec<Row>` arena plus `FxHashMap<Key, SmallVec<row-offset>>`. Values in the
  map are offsets into the arena, not row copies — extend never rehashes existing rows'
  payloads, and match enumeration order falls out of offset order (§4.2).
- **Operations**:
  - `build(rows, JoinKeys)` — initial construction; internally shares whatever the static
    `build_table` path does, so there is one hashing/keying discipline in the crate;
  - `extend(delta_rows)` — append a round's Δ to the arena and map (the semi-naive
    round boundary);
  - `rebuild(rows)` — full reconstruction, provided for the *consumer's* union-find
    merge-epoch policy (§5.2). The seam never decides when to rebuild;
  - `probe_emit(delta, JoinKeys, budget, emit)` — enumerate matches for each Δ row,
    driving a caller-supplied emit closure.
- **Zero-overhead contract** (unchanged from the rest of the crate): monomorphic over the
  concrete `Id = u32` and the SmallVec `Row`/`Key` aliases; the emit closure is a generic
  `F: FnMut(..)` parameter; the cancellation budget is the existing generic `Budget`
  parameter (the reasoner passes `NoBudget`, as it does today — closure-level budget wraps
  `materialise`, not per-join); **no `Box<dyn>` anywhere**; `#[inline]` on the probe hot
  path. `scripts/check-no-dyn-dispatch.py` covers the new module structurally.

### 4.2 Determinism contract (load-bearing for byte-identity)

The ratchet comparison in §6 is byte-exact, so the seam must not introduce iteration-order
nondeterminism relative to the hand-rolled path. The seam guarantees: **matches for a probe
key are enumerated in insertion order** (ascending arena offset). Given an identical
insertion sequence, the consumer's emission sequence is identical. This is why the layout is
arena + offset lists rather than key → owned-row buckets. The guarantee is documented on the
type and pinned by a direct unit test.

### 4.3 What the seam is *not*

Not a general incremental-view-maintenance engine, not a Differential-Dataflow-alike, and not
delta-aware on the *probe* side (Δ is always the probe side by construction). It is the
smallest API that expresses "persistent build side + append + probe" — the exact shape
semi-naive needs and the hand-rolled code already embodies. Scope discipline here is what
keeps the perf review tractable.

## 5. Consumer mapping: `owl_rl_closure`

### 5.1 Round algebra

Semi-naive `Δout = ΔR ⋈ S_full ∪ R_full ⋈ ΔS` (minus double counting) maps as: keep one
persistent delta-table per adjacency **orientation** the rule set probes, `extend` each at
the round boundary with the round's canonicalised Δ, and drive every join by probing Δ
against the opposite orientation's persistent table. **No per-round full-side scan or rebuild
exists anywhere in the loop.** The round algebra itself — which rules fire, how ΔR⋈ΔS
double-counting is avoided, emission dedup — stays exactly as today: the seam replaces the
*map mechanics*, not the algorithm.

### 5.2 UnionFind boundary (policy stays in `owl.rs`)

`sameAs` canonicalisation is untouched: rows are canonicalised to representatives *before*
insertion, exactly as now. When a union merges representatives and the current code
invalidates/rebuilds its FxHashMap adjacency, the migrated code calls the seam's `rebuild` at
the same program points with the same re-canonicalised rows. The seam is **policy-free**: it
neither observes the UnionFind nor decides rebuild timing, so merge-epoch behaviour cannot
drift. (If the current code does something lazier than rebuild-on-merge, the implementor
replicates *that* — behaviour-neutrality outranks seam elegance; `rebuild` is the ceiling,
not a mandate.)

### 5.3 Feature gating

Same discipline as sq-yk6or/PR #1301: the migration sits behind `sparq-reason`'s
**default-off `substrate-join` feature**, with an identical-output cross-assert test against
the plain branch (pattern: the existing `substrate_join_emits_identical_plain_branch`). The
tier-b reason-wasm bundle does not enable the feature, so wasm bytes stay identical. Doc
comments in always-compiled positions must use code spans, not intra-doc links, for
feature-gated items (the recurring rustdoc-gate trap).

## 6. Behaviour- and perf-neutrality verification

Per implementing PR, in order of authority:

1. **OWL-RL ratchet byte-identical**, including all 13 documented divergences — unchanged
   list, unchanged bytes. RDFS stays 48/48.
2. **Cross-assert test** (feature on vs plain branch) proves closure-output identity inside
   one test binary, independent of the ratchet corpus.
3. **Deterministic floors untouched**: store/dict bytes, `wasm_bundle_bytes` (feature off in
   all shipped bundles ⇒ byte-identical by construction).
4. **Structural gates**: no-dyn-dispatch green over the new module; coverage ratchet — every
   new substrate public fn gets a **direct** unit test (thin fns reached only indirectly sit
   at ~0% line coverage and sink the crate below its floor).
5. **Timing**: A/B closure timing on representative + chain-shaped inputs reported in the PR
   body only. No numbers are committed to markdown (repo hygiene), and this box's timings are
   non-canonical (EC2 work box) — gate only on the deterministic metrics above.

## 7. Explicit non-goals and the sq-v5evr carve-out

- **Residual hand-rolled shapes stay, documented.** The rdf:type/rdfs9 + PropExpand
  orientation-swap branches have a non-uniform combine step that a uniform kernel does not
  express without cost; sq-yk6or documented them in `substrate_join.rs` as accepted
  residuals. The epic's exit criterion is *"every uniform join shape shared; residuals
  individually documented"* — not "zero hand-rolled loops". After sq-qonbz.2, the semi-naive
  fixpoint moves **off** that residual list; the orientation-swap branches remain on it.
  *(Disposition update, sq-pbz04.1.1 [FABLE-5]: on per-branch inspection the rdf:type/rdfs9
  branch turned out to be a UNIFORM join after all — an object-keyed probe with a fixed
  permutation combine, expressible directly by `JoinKeys`' `(build_col, probe_col)` pairs —
  and was ADOPTED (`sweep_type_join`), consistent with the exit criterion above. Only the
  PropExpand branch remains on the residual list, now documented as permanent: its per-match
  combine is data-dependent (the swap flag) and cascades into a second join on the derived
  predicate. Rationale + pinning tests live in `substrate_join.rs` / `rdfs.rs`.)*
- **sq-v5evr stays parked (P4) and is not a blocker.** The full `Value`/`LitKind` value-space
  hoist (equality + relational compare beyond the ordering already shared via `CompareTerm`)
  earns its keep only with a second concrete consumer — e.g. OWL-RL datatype rules or RIF
  builtins needing value-space equality. Until then it would be a speculative, sprawling,
  non-perf-neutral move (per the sq-vezew analysis). **sq-qonbz closes with sq-v5evr open by
  design**; the close-out bead records the carve-out and the un-park trigger so the epic gate
  to sq-pbz04 does not silently hold it hostage.
- **No planner/serializer leakage.** The substrate stays a leaf: nothing in this remaining
  scope adds a dependency from `sparq-substrate` to engine or reasoner types.

## 8. Decomposition (beads, dependency-ordered, parent sq-qonbz)

| # | Bead | Scope | Depends on | Closes |
|---|------|-------|------------|--------|
| 1 | **sq-qonbz.1** (P2) | Substrate-only: `join::delta` persistent table — build/extend/rebuild/probe_emit, insertion-order determinism, direct unit tests, no-dyn gate | — | — |
| 2 | **sq-qonbz.2** (P2) | `owl_rl_closure` migration onto the seam behind `substrate-join`; cross-assert + ratchet byte-identity; divergence-doc update | sq-qonbz.1 | **sq-6w7x6** |
| 3 | **sq-qonbz.3** (P2) | Close-out audit: verify + close sq-yk6or and sq-6w7x6, residual sweep of sparq-reason, sq-v5evr carve-out note, SKILL/README upkeep for the grown substrate API, close **sq-qonbz** | sq-qonbz.2 | **sq-qonbz** → unblocks sq-pbz04 |

sq-6w7x6 now carries dependency edges on sq-qonbz.1/.2 in the tracker, so it is not
drain-ready until the seam exists. The split is deliberate: `.1` is a leaf-crate change with
zero behaviour risk (parallelises freely under the disjoint-crate wave pattern); `.2`
concentrates all behaviour-neutrality risk in one reviewable PR; `.3` is bookkeeping that
turns the epic gate green honestly.

## 9. Risks and open questions

- **R-order**: if the hand-rolled adjacency iterates a FxHashMap directly anywhere on an
  output-affecting path, its order was never deterministic and byte-identity across the swap
  needs care — the cross-assert test (§6.2) is the arbiter; if outputs are set-canonicalised
  before comparison this risk vanishes. Implementor of sq-qonbz.2 must check which holds.
- **R-epoch**: union-merge handling lazier than rebuild-on-merge (e.g. stale-row tolerance
  with emission-time canonicalisation) must be replicated, not "improved" — §5.2.
- **R-scope**: pressure to generalise the seam (probe-side deltas, retractions) is deferred
  until a second consumer (RSP incremental eval is the likely one) states a concrete need —
  same discipline that parked sq-v5evr.

*Record ends. Beads sq-qonbz.1/.2/.3 are the actionable decomposition.*
