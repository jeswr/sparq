# Reasoner / Federation / RSP-Geo Program — structure and decomposition (sq-6tykl) [FABLE-5]

> **Status: DESIGN PROPOSAL — to be ratified.** This is a program-structuring and
> decomposition record, not a commitment and not an implementation. Authored by
> Claude Fable 5 as a design/decomposition-only pass; the implementation is fleet
> work tracked in beads. Parent epic: **sq-6tykl**.
>
> Companion design records this builds on (not repeated here):
> `research/shared-eval-substrate.md` (extraction Options A–D, C chosen),
> `research/substrate-remaining-design.md` (delta-seam decision R1),
> `research/service-federation-conformance.md`, `research/federation-client-design.md`,
> `research/rsp-geo-conformance-integration.md`, `research/owl2-el-ql-reasoning-spike.md`.

## 1. Thesis — one zero-overhead eval substrate

The program's architectural claim is that **one evaluation core can serve every
evaluator in the workspace** — the SPARQL engine, the reasoner profiles, SERVICE
federation, RSP, and GeoSPARQL — without a measurable performance tax on any of
them. Concretely, the shared core is `crates/sparq-substrate`, a leaf crate that:

- **depends only on `sparq-core`** (never `sparq-engine`), so the engine and the
  `sparq-core`-only reasoners both consume it with no dependency cycle;
- is **monomorphic in every hot loop**: kernels are generic over a small
  `JoinKeys` descriptor, an `FnMut` emit sink, and a `Budget` cooperative-cancel
  trait — never `Box<dyn>` / `&dyn` / a vtable between a probe and its
  comparison. This is the #1303 constraint, enforced *structurally* by
  `scripts/check-no-dyn-dispatch.py`, not by convention;
- is **entirely behind default-off cargo features** (`rows` / `numeric` / `join`
  / `compare`), so the lean wasm bundle compiles none of it and the
  `wasm_bundle_bytes` floor stays byte-identical (floor value lives in the
  ratchet, not in this document);
- shares a **single id space**: equal terms → equal ids via the one `sparq-core`
  `Dict`, the soundness precondition for any cross-consumer join. The substrate
  deliberately exposes no raw-id constructor.

Four seams define the shared surface:

| # | Seam | Substrate module | Contract |
|---|------|------------------|----------|
| 1 | Join | `join` (merge, radix hash, bind, leapfrog WCOJ, `join::delta::DeltaTable`) | one `JoinKeys`+`Budget` probe body for all consumers |
| 2 | Numeric / value space | `numeric` (`Num`/`Dec`, `as_numeric`, `binop`, formatting) | one XSD value tower; value-space equality/relational compare is the still-parked remainder (sq-v5evr) |
| 3 | Term ordering | `compare` (`compare_terms` over `CompareTerm` + `TermClass`) | one SPARQL/RDF-1.2 total order algorithm; each consumer impls the trait for its term type |
| 4 | Solution mappings / id space | `rows` (`Row`/`Key`/`Posting` SmallVec aliases) | one id-tuple vocabulary; inline-int helpers not yet re-exported |

**Honest current state of the seams:** they are *unified in definition but not
yet in consumption*. The engine has migrated onto all four (it pulls
`numeric`+`join`+`compare` and implements `CompareTerm` for its `Value`); the
reasoners have adopted almost none — the only reasoner consumer today is the
RDFS static predicate join (`sparq-reason/src/substrate_join.rs`, rdfs2/3/7 — plus
the rdfs9 type join since the sq-pbz04.1.1 disposition; only the PropExpand
orientation-swap branch stays hand-rolled there, documented as permanent);
on the stream side, `sparq-rsp` now drives `join::delta::DeltaTable` + `rows`
for its `EvalMode::Delta`/`Snapshot` consecutive-window diff
(`sparq-rsp/src/eval.rs`, sq-2n1q3.4 [FABLE-5] — the windowed ISTREAM/DSTREAM
slide is the same semi-naive Δ-vs-full probe shape, with the previous window
as the persisted build side).
Divergence is concentrated on the reasoner side of every seam, and the single
largest divergence is that `owl_rl_closure` still runs its own hand-rolled
adjacency + UnionFind even though the `join::delta::DeltaTable` seam purpose-built
for it has landed. Closing that gap is the program's long pole.

What deliberately **stays engine-private** (query-shaped; must not leak into the
substrate or the reasoners): the planner/cardinality/GYO machinery,
`Bindings{vars,rows,sorted_by}`, `LocalVocab` interning, the per-WHERE-solution
`QueryBudget`, `ScanCmp` filter pushdown, `service.rs`, serializers,
EXISTS/aggregation, and the `Value` enum + `LitKind` + `value_compare_strict`
(only the ordering *algorithm* was hoisted, because those types also drive the
relational operators). Recording this boundary durably is first-wave item F4.

## 2. Honest estate map — built vs greenfield

| Workstream | Crate(s) | Status | Epic |
|---|---|---|---|
| Eval substrate | `sparq-substrate` | **BUILT** — four move-phases landed verbatim out of `exec.rs` (rows / numeric / join incl. delta / compare); engine consumes it for real; still `publish=false` (stub-gate escape whose trigger to flip is now satisfied) | sq-qonbz |
| RDFS + OWL-RL | `sparq-reason` | **BUILT** — real forward-chaining materialiser, conformance-passing via `Profile::Rdfs`/`Profile::OwlRl`, sound but RL-incomplete with 13 documented divergences; partial substrate hook (rdfs2/3/7 only, sq-yk6or) | sq-pbz04.1 |
| OWL 2 EL | `sparq-reason-el` | **BUILT (partial profile)** — consequence-based CR1–CR5 classifier + optional rbox (CR10/CR11) + hasse; CR6–CR9 (nominals, concrete domains) deferred and honestly surfaced via `Report::skipped_axioms`; zero substrate use yet | sq-pbz04.2 |
| OWL 2 QL | `sparq-reason-ql` | **BUILT (rewriter)** — PerfectRef over DL-Lite_R + tree-witness + UCQ minimisation behind a strict fail-closed CQ-shape gate; the hand-derived DL-Lite_R certain-answer oracle graduated to a pinned *sparq-extension* floor (sq-qo1a9, PR #1316), honestly tallied outside the standards-conformance total; the broader entailment-regime arm remains experimental/OutOfScope | sq-pbz04.3 |
| OWL 2 Direct Semantics | — | **ABSENT / GREENFIELD** — no tableau/SAT reasoner exists; deliberately deferred as research-grade. Research-first: fragment-scoping design record before any impl bead | sq-pbz04.4 |
| RIF-Core | `sparq-reason` (`rif.rs`, `rif-core` feature) | **PARTIAL** — validates RIF-Core to Datalog-safety and lowers to the in-tree N3 chainer; **no RIF conformance-suite arm exists** | sq-pbz04.5 |
| D-entailment | `sparq-reason` (`dtype.rs`, `d-entail` feature) | **BUILT** — `Profile::D`, rdfD1 typing + typed value-space compare, wired into the entailment harness; its comparator is private pending the seam-2 hoist | sq-pbz04.6 |
| SERVICE federation | `sparq-engine/service.rs`, `sparq-conformance` (`service_loopback.rs`, `service_eval.rs`, `sd_gsp.rs`), `sparq-fedplan`, `sparq-fedclient` | **SUBSTANTIALLY BUILT** — real `eval_remote` (VALUES-pushdown bind-join, SRJ/SRX, SSRF egress policy); W3C `sparql11/service` evaluation lane over real in-process `sparq_server::serve` loopback endpoints with a gating floor; SD/GSP lane; planning/client crates are off-by-default standalone members. `sparq-fedplan-mpc` is an unaudited research scaffold, **out of scope** here | sq-my8wd |
| RSP | `sparq-rsp` | **BUILT (engine)** — deterministic, clock-free RSP-QL-style engine (time/count windows, R/I/DSTREAM) with a live expressivity/correctness ratchet. **No executable W3C/CSPARQL conformance suite exists** — framed as expressivity ratchets, never as conformance | sq-2n1q3 |
| GeoSPARQL (+ full-text) | `sparq-geo`, `sparq-text` | **BUILT** — georust-backed GeoSPARQL layer (WKT/GML, function registry, R-tree, reprojection, topology rewrite) with OGC topology + query-rewrite ratchet floors; official CITE suite not MIT-vendorable, so the hand-curated probe is the sanctioned evidence; full-text has no standard → differential BM25 oracle floor | sq-lk3aw |

Nothing in the table above claims unbuilt work as built; the two genuinely
greenfield items are OWL 2 Direct Semantics and the RIF conformance arm.

## 3. Phased milestone structure

Phases express *dependency*, not strict serialization — S and X do not wait on F/R.

### Phase F — Foundation: solidify the substrate (epic sq-qonbz)
Make the substrate a finished, guarded, publishable component before piling
consumers on it. First wave (§5): flip publishable + bench + SKILL (F1), finish
the id-tuple vocabulary (F2), test the delta cancel path (F3), pin the no-dyn
gate over all four modules + record the boundary (F4), land the first semi-naive
consumer (F5 = sq-6w7x6), un-park the value-space comparator (F6 = sq-v5evr).
F5/F6 are the bridge into Phase R: after them, every seam has ≥1 reasoner-side
consumer or an honest reasoned non-adoption.

### Phase R — Per-profile reasoners (epic sq-pbz04, sub-epics sq-pbz04.1–.6)
Each profile advances on two axes, kept honest per profile:
**(a) substrate adoption** where the profile genuinely evaluates
(RL: delta-seam fixpoint; EL: concrete-domain numerics only — saturation joins
are a documented non-adoption, see `crates/sparq-reason-el/src/classify.rs`
section "Substrate adoption evaluation" [SONNET-4.6] sq-pbz04.2.3;
D: shared value comparator; RIF: builtins over the shared tower) — with QL as
one documented counter-example (query-rewriter reusing the engine path, not a
join-substrate consumer) and EL's worklist fixpoint as a second: it is **not** a
join-substrate consumer by structural incompatibility, not by omission; and
**(b) profile completion** (RL: the 13 divergences; EL: CR6–CR9; QL: CQ-gate
broadening + de-experimentalisation where sound; RIF: a conformance arm;
D: datatype-map broadening; Direct: a research-first fragment-scoping record
*before* any implementation bead). Every step is behaviour-neutral-or-ratcheted:
pure refactors must be byte-identical on the entailment ratchets; capability
steps move a floor honestly.

Ordering inside R: RL first (it is the substrate's proving consumer and the long
pole), EL/QL/RIF/D in parallel after (disjoint crates/modules), Direct last and
gated on its design record.

### Phase S — SERVICE federation (epic sq-my8wd)
Mostly built; remaining work is graduation and hardening of the already-landed
loopback conformance lanes plus the off-by-default planning/client crates.
Independent of F/R (the service path rides the engine, which already consumes
the substrate). `sparq-fedplan-mpc` stays out of scope for this program.

### Phase X — RSP + GeoSPARQL/full-text (epics sq-2n1q3, sq-lk3aw)
Fully parallel and substrate-independent (both crates already sit behind opt-in
features with tier-b wasm bundles and live scoreboard ratchets). The work is
expressivity/correctness ratchet growth — explicitly **not** labelled
conformance where no suite exists (RSP, full-text) and OGC-not-W3C for geo.

## 4. Gating discipline — the core stays lean

Every capability in this program is **opt-in**: a separate crate or a
default-off cargo feature. Standing rules, all pre-existing and reaffirmed here:

1. **Lean core:** `sparq-core`/`sparq-engine` default builds gain no new
   dependencies or code paths; the lean wasm bundle floor stays byte-identical.
2. **No-dyn gate:** the substrate's zero-overhead contract is enforced by
   `scripts/check-no-dyn-dispatch.py` (F4 pins its module enumeration).
3. **Behaviour-neutral refactors:** any move-onto-the-substrate change must be
   byte-identical on the relevant ratchet (closure multisets, conformance
   floors); capability changes move floors honestly, never silently.
4. **Honesty labelling:** sparq-extension floors (QL oracle, RIF, RSP, BM25) are
   tallied separately from standards-conformance totals; absent suites are
   named as absent; deferred rules surface in reports (`skipped_axioms`).
5. **No perf numbers in markdown:** ratchets and floors are referenced by name;
   values live in the code/ratchet files that CI checks.
6. **Fleet mechanics:** each impl bead carries crate, `model_tier`, invariant,
   and `acceptance_test`; new public fns get one direct unit test (coverage
   floor); crate READMEs obey the readme-template gate; feature-gated intra-doc
   links use code spans.

## 5. Bead map

### Program layer (pre-existing)
- **sq-6tykl** — program epic (this record's parent).
- **sq-qonbz** — substrate foundation epic · **sq-pbz04** — reasoner-suite epic
  · **sq-my8wd** — SERVICE federation epic · **sq-2n1q3** — RSP epic ·
  **sq-lk3aw** — GeoSPARQL/full-text epic.

### Per-profile sub-epics (created by this decomposition, children of sq-pbz04)
- **sq-pbz04.1** — OWL 2 RL + RDFS: delta-seam adoption + honest RL-completeness push.
- **sq-pbz04.2** — OWL 2 EL: CR6–CR9 build-out + first substrate adoption.
- **sq-pbz04.3** — OWL 2 QL: CQ-shape-gate broadening + sound de-experimentalisation.
- **sq-pbz04.4** — OWL 2 Direct Semantics: research-first fragment scoping (greenfield).
- **sq-pbz04.5** — RIF-Core: conformance arm + builtins over the shared tower.
- **sq-pbz04.6** — D-entailment: shared value-comparator adoption + datatype map.

No sub-epics were created for substrate/federation/RSP/geo — those epics already
existed (above) and duplicating them would violate the tracker's dedup discipline.

### First wave — foundation tasks (disjoint; children of sq-qonbz unless noted)

| Id | Item | Crate / files | Tier |
|---|---|---|---|
| sq-qonbz.4 | F1: flip `publish=false` → publishable + join/numeric micro-bench + SKILL surface | `sparq-substrate` (Cargo.toml, benches/, skills/) | sonnet |
| sq-qonbz.5 | F2: re-export the inline-integer id helpers in `rows.rs` + round-trip parity test | `sparq-substrate` (rows.rs only) | haiku |
| sq-qonbz.6 | F3: direct `Budget` cancellation test for `join::delta::probe_emit` | `sparq-substrate` (join.rs tests only) | haiku |
| sq-qonbz.7 | F4: no-dyn gate enumerates all four hot-loop modules (incl. `join::delta`) + substrate boundary recorded in AGENTS.md | scripts + AGENTS.md | haiku |
| sq-6w7x6 | F5: migrate `owl_rl_closure` onto `join::delta::DeltaTable` (pre-existing, enriched; via sq-qonbz.2) | `sparq-reason` | sonnet |
| sq-v5evr | F6: hoist the value-space equality/relational comparator behind a default-off feature (pre-existing; un-parked P4→P2 — the second-consumer trigger is now met by sq-pbz04.5/.6) | `sparq-substrate` (new module + Cargo.toml) | sonnet |

Conflict discipline: the only shared-file overlap in the wave is
`sparq-substrate/Cargo.toml` between F1 and F6, serialized by a bd dependency
(sq-v5evr depends on sq-qonbz.4). Everything else is file-disjoint.

## 6. Non-goals and honesty ledger

- **No OWL 2 Direct tableau claim.** sq-pbz04.4 produces a scoping design record
  first; implementation beads exist only after ratification.
- **No RSP or full-text "conformance" claim** — no executable standard suite
  exists; the scoreboard rows are expressivity/correctness ratchets and say so.
- **Geo conformance is OGC-shaped, not W3C**, via the sanctioned hand-curated
  probe (CITE not MIT-vendorable); the distance-approximation note stands.
- **QL is not a join-substrate consumer** — by design, not by omission.
- **EL's CR1-CR5 saturation joins are not join-substrate consumers** — by
  structural incompatibility (worklist-event shape, simultaneous read+write on
  S-sets, 3-way triangle join in CR4), not by omission. Concrete-domain numerics
  (`cdomain` feature, sq-pbz04.2.2) ARE adopted. Full non-adoption rationale:
  `crates/sparq-reason-el/src/classify.rs` § "Substrate adoption evaluation"
  [SONNET-4.6] sq-pbz04.2.3.
- **`sparq-fedplan-mpc` is out of scope** for this program (unaudited research
  scaffold; the MPC/ZK program owns it).
- **sq-v5evr's un-park is consumer-justified**, not speculative: D-entailment and
  RIF builtins are the named second consumers.
- **This document contains no performance numbers**; every quantitative claim
  lives in a CI-checked ratchet.
