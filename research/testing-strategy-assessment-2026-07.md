# Testing + correctness-guarantee strategy — whole-repo assessment (2026-07)

<!-- 🤖 SPARQ agent [FABLE-5] — maintainer-commissioned assessment (front 1 of 3), 2026-07-10.
     Method: every current-state claim below was verified against the code / CI configs in this
     tree (commit 569dc0e91), not taken from prior records or the commissioning brief on faith.
     Where the brief's premises were wrong, §2.1 corrects them. -->

**Status:** assessment + decomposition (design-only — no implementation in this PR).
**Epic:** `sq-3dyje` — *testing + correctness-guarantee program* — with 13 disjoint children
`sq-3dyje.1`–`sq-3dyje.13` (§9; the table's `#` column is the child suffix).
**Builds on (does not duplicate):** `research/mechanized-proof-program.md` (the proof
program + assurance ladder + anti-vacuity §5.1–§5.2), `research/test-quality-program-plan.md`
(sq-qcnn coverage/mutation), `research/differential-testing-value-level.md` (sq-qcnn.4–.10),
`research/test-coverage-decomposition.md` (sq-bif), `research/change-based-test-selection.md`
(sq-fmx4u, shipped), `research/web-gui-test-program.md` (sq-ymr2e, site/GUI — out of scope
here), and the ZK records (`research/zk-correctness-and-proof-program.md`,
`research/zk-test-bench-design.md`, `research/zk-verifier-reaudit.md`) for proof-obligation
scope only.

## 0. Bottom line up front

sparq's testing estate is **strong and unusually honest at four layers** — unit/integration
scale (7,975 `#[test]`s, 45 crates with `tests/` dirs), W3C conformance ratchets (SPARQL 1229,
inference 1967, SHACL, JSON-LD, GeoSPARQL, Turtle, and a gating HTTP-Protocol lane), a
value-aware differential program vs Oxigraph (per-PR blocking smoke + nightly advancing seeds
+ auto-filed repro beads), and a mutation/coverage ratchet pair that measures test *quality*,
not just line %. The mechanized-proof program (4 Kani suites, 41 harnesses, a drift-checked
`ci/formal-verification.toml` manifest, and the mandatory anti-vacuity self-checks) is real
and correctly scoped.

The honest gaps, in priority order:

1. **Property-based testing does not exist at all** (zero `proptest`/`quickcheck` in the
   workspace — the commissioning brief's "~2 proptest files" were a comment and an
   `arbitrary`-based diff-fuzz harness). This is the cheapest unowned layer with the highest
   marginal value: it is the tier of record the proof program's own NOT-TRACTABLE ledger
   points at for dict bijectivity (C-2), parser round-trips (C-4), and the engine's real
   `Value` ordering.
2. **SPARQL UPDATE has no differential oracle** — the differential fuzzer (9 query
   categories) is query-only; `update.rs` (2,309 LOC) rides on unit tests + conformance only.
3. **Fuzzing covers 4 of 52 crates directly** (6 targets, not the brief's "51") — untrusted-
   input surfaces with zero targets include the remote SPARQL-results parsers
   (SRJ/SRX/entity-expansion in `sparq-engine-service` — parses *remote federation
   responses*), HDT binary decode, RDFC-1.0 poison graphs, RIF-XML/N3, WKT, and terse.
4. **Two crates have committed evidence of vacuous assurance**: `sparq-fedclient` /
   `sparq-fedplan` sit at ≥90% line coverage with **0 mutations caught** (553/534 survivors,
   honestly annotated in `bench/mutants-baseline.json` by sq-qcnn.29 — but no open bead
   drives the fix).
5. **Concurrency testing is zero** (no loom — the brief's "~4 loom files" are `Bloom`-filter
   false positives) and the OCC transaction path has no lost-update/interleaving test.
6. `sparq-metamorph` implements TLP/NoREC/cross-engine oracles but **no CI lane drives
   them** over generated queries.

"Guarantee the correctness of ALL code" cannot honestly mean "prove everything" (§6): the
achievable, defensible standard — adopted as this program's goal — is that **every
correctness-critical surface carries at least two independent guarantee mechanisms with
different failure modes, and no surface's guarantee rests on line coverage alone.** §9 creates
13 disjoint beads that close the enumerated gaps to that standard.

## 1. Scope + method

- **In scope:** all Rust crates (52 under `crates/` + `vendor/spargebra` + `fuzz/`), their CI
  correctness lanes, and the formal-methods estate.
- **Out of scope:** site/GUI testing (owned by sq-ymr2e / `research/web-gui-test-program.md`);
  benchmark methodology (sq-hmd7l); CI throughput (sq-fmx4u / sq-6vshe, shipped or owned).
- **ZK/MPC carve-out (honesty rule):** this record enumerates where the ZK/MPC surfaces sit
  in the guarantee map (§4) but makes **no soundness assessment and no soundness claim**. The
  v1 ZK verifier is internally re-audited with **external accredited-cryptographer sign-off
  PENDING (sq-qhy4)**; `sparq-mpc` targets a semi-honest (honest-majority) model only. Their
  functional-correctness obligations and test machinery are owned by
  `research/zk-correctness-and-proof-program.md` (M0–M3 built; T1–T4 obligations encoded in
  `sparq-mpc/src/lib.rs`) and are deliberately **not re-decomposed here** — nothing in this
  record or its beads may be cited as a cryptographic-soundness guarantee.
- **Method:** counts and mechanisms below were re-measured in this tree (grep/find/read of
  the actual workflows, ratchet JSONs, and test files), cross-checked against two
  independent sub-agent audits. Commands are cited where a number is load-bearing.

### 1.1 Corrected premises (what the commissioning brief got wrong)

| Brief's premise | Measured reality |
|---|---|
| "fuzz (51 targets)" | **6** cargo-fuzz targets (`fuzz/fuzz_targets/`), exercising sparq-core (4), sparq-engine (1), sparq-shacl (1), sparq-jsonld (1, shared) |
| "only ~2 proptest files" | **0** property-based tests. `grep -rl "proptest\|quickcheck" crates/ --include='*.rs'` hits a comment (`sargable_decimal_precision.rs`: "No proptest dep") and the `arbitrary`-based SHACL diff-fuzz harness; no crate has a proptest/quickcheck dev-dependency |
| "concurrency (loom, ~4 files)" | **0** loom tests. The grep hits are the word "Bloom" (`compress.rs`, `semijoin.rs`) |
| "miri, ~10 refs" | Miri is a **nightly 12-shard lane over sparq-core only** (the sole unsafe crate), `parallel` feature, mmap/dict-spill excluded (Miri cannot model file-backed mappings); `cfg(miri)` markers in code: 0 |
| "kani proofs (~4 files)" | Correct: 4 files, **41 harnesses** across 5 CI matrix suites (per-suite breakdown §3) |
| (assumed missing) SPARQL protocol conformance | The W3C SPARQL 1.1 Protocol (HTTP) + Service-Description/GSP lanes **are wired and gating** in `ci.yml` (loopback harness, pinned `HTTP_PROTOCOL_FLOOR`, `http-protocol` feature) |

## 2. The testing pyramid that actually exists (measured)

| Layer | Mechanism | Scope | Cadence / gate | Evidence |
|---|---|---|---|---|
| Unit + integration | `#[test]` | 7,975 tests; 45/52 crates have `tests/` dirs; heaviest: `sparq-engine` (~15k LOC of `tests/`, incl. 8 in-tree differentials), `sparq-server` (~15.5k LOC incl. protocol/hardening/wire-contract) | per-PR, **blocking** (change-based selection, fail-safe) | `crates/*/tests`, `ci.yml` |
| Line-coverage ratchet | `cargo llvm-cov` per crate, floor = measured−2, only rises; K=3 robust re-measure | floors 90–97 for real-logic crates; artifact crates presence-gated at 0 | per-PR, **blocking** | `bench/coverage-floor.json`, `scripts/coverage-gate.py` |
| Mutation ratchet | cargo-mutants, per-crate surviving-count ceilings, ~3,230 mutants, 3 shards; feature-aware invocation | seeded crates in `bench/mutants-baseline.json`; wasm/cli/gpu excluded | nightly, **advisory** (promotion to gating = open sq-qcnn.27) | `ci.yml` `mutants-nightly-advisory` |
| W3C conformance ratchets | `sparq-conformance` binaries + in-source floors | SPARQL 1.1 query/update/results/syntax ≥1229; inference (RL/EL/QL/RDF/RDFS/RIF) ≥1967; SHACL core ≥98 + SHACL-SPARQL; JSON-LD 1.1 (toRdf 413 / fromRdf 51 / compact 186 / frame 61 / expand / flatten); Turtle 313/0; SPARQL entailment 47/0; GeoSPARQL 119+38; SPARQL 1.1 Protocol (HTTP) + SD/GSP loopback lanes | per-PR, **blocking** | `ci.yml`, `crates/sparq-conformance` |
| Differential (cross-engine) | `sparq-bench` fuzzer vs in-process Oxigraph 0.5; 9 query categories (bgp/filter/equality/optional/union/minus/limit/distinct/order) + 3 store-mode differentials (mmap/block-compressed/dict-spill); adjudicated divergence allowlist (`bench/differential-divergences.json`, 2 classes) | fixed-window smoke **blocking per-PR**; nightly 12-shard advancing window (~120k seeds/category) auto-files P1 bead + issue with deterministic repro | per-PR + nightly | `fuzz.yml` `differential-smoke`, `differential.yml` |
| Differential (in-tree) | engine self-differentials: DP-vs-greedy planner, Yannakakis/semijoin-vs-baseline, vectorized-vs-scalar byte identity, dict-spill/consolidation, parallel-vs-serial load, fork | run as normal tests | per-PR, **blocking** | `crates/sparq-engine/tests/`, `crates/sparq-core/tests/` |
| Differential (SHACL, 3 oracles) | `arbitrary`-driven shapes/data vs pySHACL 0.31.0 + Jena 5.2.0 + Node shacl-engine; asserts `sh:conforms` + per-focus-node violated-constraint sets | 2,000 seeds/night | nightly, advisory | `shacl-diff-fuzz.yml`, `crates/sparq-shacl/tests/diff_fuzz.rs` |
| Metamorphic | `sparq-metamorph`: TLP, NoREC, cross-engine `Verdict`s; self-test with injected `FilterDropsRow` bug | library + self-tests only — **no CI driver generates queries against it** | (gap — §5) | `crates/sparq-metamorph/src/` |
| Fuzz | cargo-fuzz, auto-discovered targets | 6 targets: `parse_rdf_str`, `load_reader_parallel`, `graph_open`, `parse_sparql`, `validate_shacl`, `jsonld_pipeline` | per-PR corpus replay (`-runs=0`) **blocking**; main-push 120 s/target; nightly 600 s/target + corpus cache | `fuzz.yml` |
| Property-based | — | **none** | — | §1.1 |
| Concurrency (model) | — | **none** (no loom; no lost-update test on the OCC txn path) | — | §1.1 |
| UB detection | Miri (tree-borrows), 12 shards + doctests | sparq-core, `parallel` feature (mmap/dict-spill excluded by tool limitation) | nightly, non-blocking; watched by `formal-alarm.yml` no-verdict alarm | `miri.yml` |
| Sanitizers | ASan (`-Zbuild-std`) | mmap-corruption corpus (`sparq-core`) + `sparq-vectors` store/diskann | nightly, informational | `asan.yml` |
| Formal (bounded proof) | Kani/CBMC, 5 matrix suites, drift-checked manifest, change-coupled PR gating for healthy suites | §3 | nightly + `fv-select` on touching PRs | `kani.yml`, `formal-verification.yml`, `ci/formal-verification.toml` |
| Lane-health meta | `formal_lane_alarm.py`: a lane with no successful verdict on main for > N hours becomes a loud issue (the miri-cancelled-22-nights lesson) | kani/miri/asan/differential | scheduled | `formal-alarm.yml` |

## 3. The formal-methods estate — proven vs tested vs neither

Verified per-suite (`grep "#\[kani::proof\]"`; harness semantics read in-file):

| Suite (crate / file) | Harnesses | What is actually proven | Caveats |
|---|---|---|---|
| `sparq-substrate/src/compare.rs` | 26 | `compare_terms` total-order laws (reflexivity, antisymmetry-consistency, within-class totality, transitivity incl. the 2^53 collapse tiers, triple-term recursion) — **over an in-harness model `CompareTerm` type**, bounded | The engine's real `Value` impl is explicitly NOT covered (ledgered in the proof program §6) — closed by bead 3, §9 |
| `sparq-core/src/dict.rs` | 7 | inline-id round-trip bijection + 4-region id-partition disjointness (complete-domain); dict-file validator totality never-panics (bounded, `unwind(28)`) + domain self-checks | mmap-validator totality suites `pr_gate=false` pending bound-tightening |
| `sparq-engine/src/reduce.rs` | 6 | columnar reducers ≡ independent scalar fold (bounded slice ≤8, complete over values); `narrow_sum_to_i64` iff-in-range; adversarial-domain self-check | |
| `sparq-vectors/src/store.rs` | 2 | `.spqv` open-never-panics on hostile bytes — harnesses exist but the claim is **not currently re-established by CI** (nightly budget/unsupported-construct failures; sq-otpxg) | Tier of record meanwhile: unit/exhaustive tests incl. the concrete accept-path pin |

Everything else is **tested, not proven** — and for most of it that is the right call. The
proof program's tool triage (Creusot/Prusti NOT NOW, Lean research-tier, hax N/A) and its
NOT-TRACTABLE ledger were re-checked against the current code and **stand**; this record does
not relitigate them. Deductive whole-engine verification remains infeasible at this codebase's
velocity; conformance + differential + property layers are the honest guarantee there.

The **anti-vacuity program is load-bearing and must bind every new harness**: every future
Kani bead in §9 inherits `research/mechanized-proof-program.md` §5.1–§5.2 verbatim (mandatory
domain-coverage self-check: exact-image/domain pinning + `kani::cover!` witness survival or
the sanctioned SPLIT form) — a mutation spot-check alone does not catch an
`assume(false)`-pruned domain (the sq-sqtk2.1 vacuity hole).

## 4. Risk-surface × guarantee map

Guarantee tiers: **P** proved (bounded/complete, Kani) · **D** differential vs independent
oracle · **C** conformance-pinned (W3C) · **M** mutation-measured · **U** unit/integration ·
**F** fuzzed (robustness) · **PB** property-based · **—** nothing dedicated.

| Surface (files, size) | Has today | Missing for the two-mechanism standard |
|---|---|---|
| Engine eval core (`exec.rs` 19.2k LOC, `eqjoin`/`semijoin`/`chunk*`) | U(207 in-file) + D(in-tree differentials + Oxigraph fuzzer) + C + M | PB result-equivalence vs an in-test reference evaluator (bead 3) |
| Join kernels (`sparq-substrate/join.rs`, 1.3k LOC merge/hash/bind/LFTJ) | D (via engine) + U | bounded Kani merge-join ≡ nested-loop ("plausible next wave" per proof-program §6) (bead 10) |
| Term ordering (`compare.rs` 1.5k LOC) | P(model) + C + U | PB over real term encodings — closes the model-fidelity TCB gap (beads 1, 3) |
| Numeric tower (`numeric.rs` 1.8k LOC) | U(65) + D(`sparq-difftest/numeric.rs`) + M | PB vs exact i128/BigDecimal reference oracle (bead 1) |
| Dictionary (`dict.rs` 3.6k, `dictspill.rs` 1.7k, `compress.rs` 2.2k) | P(inline ids) + U + D(spill/consolidation/fork differentials) + F + Miri/ASan | PB full-term intern/lookup bijectivity (C-2 tier of record) (bead 2) |
| N-Triples/N-Quads custom parser (`nt.rs` 1.2k LOC, hand-written byte-level) | F + D(parallel-vs-serial) + C(Turtle-suite common path) + U | PB parse∘serialize round-trip (C-4) (bead 2); the §8 checklist applies |
| Turtle/TriG serial parse | delegated to `oxttl` (not hand-written today); native tokenizer is **design-only** (sq-jocpn, unmerged) | §8 checklist is the acceptance frame for sq-jocpn when it lands |
| SPARQL parser (`vendor/spargebra`, 2.8k LOC, patched vendor) | C(syntax suites) + F(`parse_sparql`) + U(vendor recursion-depth) | — (adequate; algebra rewrites below) |
| Algebra rewrites (`rewrite.rs` 770 LOC, `dp.rs` 819 LOC) | U + D(`dp_planner_differential`, `rewrite_pass`) + C | PB plan-equivalence over random plans (folded into bead 3's eval-equivalence, which subsumes rewrite output) |
| Aggregates + expression eval (`aggregate.rs`, exec paths) | C + U + D(vectorized-vs-scalar) | generator extension owned by open sq-qcnn.6 (aggregates/BIND/string fns) — endorsed, not duplicated |
| UPDATE path (`update.rs` 2.3k LOC) + OCC txn (`txn.rs` 525 LOC) + `store.rs` overlay | U(37+) + C(update suites) | **D: no differential oracle for updates at all** (bead 4); no concurrent-writer lost-update/interleaving test (bead 12) |
| RDF-star / quoted triples (nt.rs triple terms, engine rdfstar tests, canon rdf12) | U + C(rdf12 profile) | cross-engine differential — recommend extending sq-qcnn.6's generator with quoted-triple patterns (note added to that bead; no new bead) |
| Canonicalization (`sparq-canon/rdf12.rs` 1.3k LOC, RDFC-1.0) | C(W3C RDFC-1.0 suite incl. poison negatives) + U + M | PB determinism under bnode-relabel + quad permutation; idempotence (bead 8); fuzz on hostile N-Quads + poison depth (bead 5) |
| Remote-result parsers (SRJ/SRX/TSV in `sparq-engine-service` — untrusted federation input) | U(direct, sq-qcnn.34) + M | **F: zero fuzz on a remote-input parser** (bead 5) |
| HDT binary decode (`sparq-hdt`) | U + C-ish fixtures + M(82% caught) | F on hostile bytes (bead 5) |
| Reason parsers (RIF-XML, N3) | U(72+66) + C(RIF/inference ratchets) | F (bead 5) |
| Geo (WKT parse, DE-9IM) | U + C(OGC ratchets ≥119+38) + M(66% caught — sq-qcnn owns raising) | F on WKT (bead 5) |
| Federation planner/client (`sparq-fedplan`, `sparq-fedclient`) | U + coverage ≥90 | **mutation shows 0 caught (534/553 survivors)** — in-process behavioral assertions absent; beads 6–7 |
| Server/protocol (`http.rs` 9.1k LOC) | U(~15.5k LOC tests) + C(Protocol/GSP gating lanes) + M(unseeded — open sq-qcnn.41) | adequate for now; HTTP-layer fuzz noted as future work (not beaded — axum/hyper do the parsing; the payload parsers are covered via beads 5 + existing targets) |
| WASM ports (7 `wasm_bindgen_test` files, JS-boundary contracts) | U(wasm32) + native tests carry semantics | native-vs-wasm result parity on a pinned conformance sample (bead 11) |
| ZK/MPC/trust (`sparq-zk*`, `sparq-mpc`, `sparq-trust`) | U at scale (e.g. 408 inline tests + 1.7k-LOC adversarial suite in sparq-mpc; 14.8k LOC of zk-compose integration tests) + M + determinism/fail-closed coverage (sq-qcnn.24/.25/.26/.38, no soundness claim) | owned by the ZK program + **sq-qhy4 external audit (PENDING)**; deliberately NOT beaded here; no soundness claim is made or implied |
| Reasoners (RL/EL/QL/DL, RIF, D-entailment) | C(≥1967 ratchet) + U + M | adequate two-mechanism coverage; differential vs external reasoners is future work if divergences surface |

## 5. Gap analysis (ranked by risk × cost-to-close)

1. **No property-based layer anywhere** — the one systematic-input-diversity mechanism the
   estate lacks; directly named by the proof program's ledger as the tier of record for C-2,
   C-4, and the real-`Value` order laws. Cheap (dev-dep + test files), zero runtime risk.
2. **UPDATE differential absent** — the write path is the only major engine surface with a
   single guarantee family (unit+conformance, same failure mode: both are fixed corpora).
3. **Fuzz coverage 4/52 crates** — specifically the *untrusted-input* parsers outside the
   core loaders: remote SPARQL-results (SRJ/SRX), HDT, RDFC-1.0 input, RIF-XML/N3, WKT, terse.
4. **Vacuous-assurance evidence in fedclient/fedplan** (0 mutants caught at ≥90% coverage) —
   known, annotated, unowned.
5. **Metamorphic oracles unwired** — TLP/NoREC find single-engine logic bugs differential
   testing structurally cannot (they need no second engine and no allowlist adjudication).
6. **Concurrency untested** — OCC txn/overlay path has no interleaving or lost-update tests;
   loom applicability itself unassessed (may honestly be "not applicable — coarse locking").
7. **WASM semantic parity unverified** — wasm32 tests cover the JS boundary; nothing compares
   query *results* wasm-vs-native.
8. **Kani vectors suite not re-established in CI** (owned: sq-otpxg) and solid harnesses
   runner-constrained (sq-sqtk2.6/.7) — noted for completeness; already owned, not re-beaded.

## 6. What "guarantee correctness of ALL code" can honestly mean

An absolute guarantee does not exist: every mechanism bottoms out in a TCB (rustc/LLVM,
Kani/CBMC, oracle engines' own correctness, the W3C corpora's coverage, hardware). Deductive
whole-engine proof (Lean/Creusot-class) remains expert-months-per-theorem against a codebase
this size and velocity — re-affirmed, not re-litigated, from the proof program's tool triage.
The defensible standard this program adopts:

> **Every correctness-critical surface carries ≥2 independent guarantee mechanisms with
> different failure modes** (e.g. conformance corpus + randomized differential; bounded proof
> + property-based generation; unit assertions + mutation measurement), **and no surface's
> guarantee rests on line coverage alone.** Where a mechanism is claimed, its bounds and TCB
> are stated (`PROVED (bounded)` never plain "proved"; divergence allowlists adjudicated and
> counted, never silent).

§4's third column is exactly the delta to that standard; §9's beads close it. Three standing
rules fall out:

- **Fixed corpora and randomized generation are different mechanisms; two fixed corpora are
  not.** (This is why UPDATE fails the standard despite W3C update conformance.)
- **Line coverage is a floor detector, mutation is the assertion detector, and neither
  generates inputs** — the estate has both ratchets; the missing input-generation layers are
  property-based + fuzz breadth (beads 1–5, 8).
- **A proof without its anti-vacuity self-check is not a mechanism** (§3; mandatory
  §5.1–§5.2 inheritance).

## 7. Recommendation (tiered by rigor and cost)

### 7.1 Property-based testing — make it standing (NEW layer)

Adopt `proptest` as a dev-dependency in exactly four crates, with named invariants:

| Crate | Invariants (property tests) |
|---|---|
| `sparq-substrate` | total-order axioms (reflexivity/antisymmetry/transitivity/totality-within-class) over **generated real term encodings** incl. NaN, ±0.0, 2^53±k, decimal/int cross-tier; numeric tower ops vs an exact i128/BigDecimal reference oracle; promotion commutes with value |
| `sparq-core` | dict `intern`/`lookup` bijectivity over generated terms (unicode IRIs, long strings crossing the inline boundary, lang tags, canonical/non-canonical numeric forms; both heap and spill-activated states); N-Triples parse∘serialize∘parse fixpoint (the C-4 ledger item) |
| `sparq-engine` | end-to-end result-equivalence: random small graphs × random BGP/filter/optional queries, engine (all opt-in plan paths: GOO/DPccp/vectorized) ≡ an in-test naive nested-loop reference, **multiset semantics**; join-order permutation invariance; ORDER BY sorted-output law over the real `Value` path (closes the model-fidelity TCB gap) |
| `sparq-canon` | RDFC-1.0 canonical output byte-identical under blank-node relabeling + quad permutation; idempotence; `issue_quads_with` ↔ `canonicalize_quads_with` agreement |

Where property tests are **overkill**: crates whose logic is glue over externally-tested
libraries (arrow, py bindings, mcp), and the wasm shims (parity bead 11 covers semantics).

### 7.2 Mechanized proof — warranted vs overkill

- **Warranted (one new wave):** bounded merge-join ≡ nested-loop reference over the
  `sparq-substrate` join kernels at tiny sizes — the proof program's own "plausible next
  wave" (B-3). Harness-only diff, §5.1–§5.2 anti-vacuity mandatory. That is the *only* new
  proof obligation this assessment adds.
- **Not warranted now (stays on the ledger, tier of record = tests):** optimizer plan-space
  proofs (B-4), full aggregation semantics (B-6), dict string bijectivity as proof (C-2 —
  property tests are the honest tier), parser grammar proofs (C-4 — round-trip tests),
  Creusot/Prusti/Lean adoption, anything mmap/file-backed (Kani and Miri both excluded by
  tool limitations; ASan + corruption oracles are the tier).
- **Do not make the Kani lane merge-blocking yet** — the program's soak rationale stands;
  `fv-select` already gates PRs that touch proved files, which is the right shape.

### 7.3 Differential testing — endorse and extend

- **Endorse without duplication:** the value-level multi-oracle program (open sq-qcnn.5/.6/
  .7/.8/.9/.10) is correctly designed and already owns binding-multiset comparison, the
  second oracle (Jena/rdflib), N-way triage, generator extension, and CI ratcheting. No bead
  here touches those files; a note is added to sq-qcnn.6 to include quoted-triple (RDF-star)
  generator patterns.
- **Extend (new):** a SPARQL **UPDATE differential** — generate ground-term update sequences
  (INSERT DATA / DELETE DATA / DELETE-INSERT-WHERE / CLEAR / DROP, no bnodes in v1), apply to
  sparq and Oxigraph, compare final stores canonically + query probes after each step;
  deterministic seed repro + its own nightly lane file (`differential-update.yml`, so it
  cannot conflict with the open qcnn CI beads).
- **Wire the metamorphic oracles:** a nightly lane driving TLP/NoREC over the existing
  generator corpus, mirroring `differential.yml`'s repro + auto-file-bead pattern. TLP/NoREC
  catch what cross-engine differentials cannot (bugs shared with the oracle, and surfaces
  where no oracle exists).

### 7.4 Fuzzing — close the untrusted-input gaps

One wave of 6 targets in the existing auto-discovered `fuzz/` workspace (no workflow edits
needed): SRJ/SRX remote-result parsing (`sparq-engine-service`), HDT container decode,
RDFC-1.0 canonicalization input (with the poison-graph work bound), RIF-XML + N3, WKT, terse.
Each target: hostile bytes → clean `Err` or valid output, never panic/OOB/timeout-blowup.

### 7.5 Mutation honesty — fedclient/fedplan

Two kill beads (one per crate): add in-process behavioral assertions (planner produces the
planned shape; client pushdown/streaming decisions observable via direct calls), drive the
survivor counts materially down, honest re-baseline. Sequenced **before** the sq-qcnn.27
advisory→gating promotion so the promotion doesn't enshrine ceilings of 553/534.

### 7.6 Concurrency + WASM parity

- A **spike** on the OCC txn path: write the lost-update/write-write-conflict interleaving
  tests; return an honest verdict on loom applicability (if the path is coarse-locked,
  document "loom not applicable" and close — do not cargo-cult loom).
- **WASM parity:** run a pinned W3C SPARQL query sample under wasm32 (`wasm-bindgen-test`)
  and assert result equality with committed expectations that the native suite verifies —
  the cheapest honest check that the wasm build computes the same answers.

## 8. The new-parser correctness checklist (reusable — applies to sq-jocpn and every future hand-written parser)

A hand-written parser (replacing or bypassing a reference implementation) ships only with ALL
of:

1. **Round-trip property tests** — `parse ∘ serialize ∘ parse` fixpoint over generated inputs
   (proptest), plus `serialize ∘ parse` identity on canonical forms, in the parser's crate.
2. **Differential fuzz vs the reference it replaces** — a cargo-fuzz target feeding identical
   bytes to both (e.g. native tokenizer vs `oxttl`), asserting identical triple
   streams/errors-modulo-documented-divergences; wired into the auto-discovered `fuzz/`
   workspace so per-PR corpus replay + nightly randomized runs apply automatically.
3. **Conformance-suite tie** — the relevant W3C ratchet floor (e.g. TurtleTests) unchanged or
   raised in the same PR; byte/count-identical parse on the suite corpus.
4. **Robustness target** — hostile-input fuzz (never panic/OOB), separate from (2), if the
   grammar entry point is new.
5. **Feature/fallback discipline** — a fast-path parser that falls back to the reference on
   unrecognized shapes must differential-test the *dispatch decision* too (fallback taken ⇒
   results identical).
6. **Honest perf claim** — measured on the canonical bench path, never a work-box number in
   docs (house rule).

This checklist is codified into `AGENTS.md` by bead 13 and referenced from the sq-jocpn bead.

## 9. Child beads (created with this record; disjoint by construction)

Epic `sq-3dyje`; row `#N` below is bead `sq-3dyje.N`. One crate/surface per bead; **no two
beads touch the same file**. Same-crate pairs (1,10) and (3,12) are dep-sequenced
NON-parallel (shared crate surface), as is (6,7) (shared `bench/mutants-baseline.json`). All
Kani work inherits §5.1–§5.2 anti-vacuity verbatim. Tiers: cheapest sound model.

| # | Bead | Crate/surface | P | Tier | Files (exclusive) |
|---|---|---|---|---|---|
| 1 | proptest: substrate order axioms + numeric tower vs reference | sparq-substrate | 1 | sonnet | `crates/sparq-substrate/Cargo.toml`, `crates/sparq-substrate/tests/proptest_order_numeric.rs` |
| 2 | proptest: dict bijectivity + NT round-trip (C-2/C-4) | sparq-core | 1 | sonnet | `crates/sparq-core/Cargo.toml`, `crates/sparq-core/tests/proptest_roundtrip.rs` |
| 3 | proptest: eval ≡ reference, join-order invariance, ORDER BY real-`Value` laws | sparq-engine | 1 | opus | `crates/sparq-engine/Cargo.toml`, `crates/sparq-engine/tests/proptest_eval.rs` |
| 4 | UPDATE differential vs Oxigraph + nightly lane | sparq-bench | 1 | sonnet | `crates/sparq-bench/src/update_fuzz.rs`, `crates/sparq-bench/src/main.rs` (mod line), `.github/workflows/differential-update.yml` (new) |
| 5 | fuzz wave: 6 untrusted-input targets | fuzz/ | 1 | sonnet | `fuzz/Cargo.toml`, `fuzz/fuzz_targets/{parse_srj_srx,hdt_open,canonicalize_nquads,parse_rif_n3,parse_wkt,parse_terse}.rs` |
| 6 | mutation-kill: fedclient behavioral assertions (553 → materially down) | sparq-fedclient | 1 | sonnet | `crates/sparq-fedclient/**` (tests), `bench/mutants-baseline.json` (its entry) |
| 7 | mutation-kill: fedplan behavioral assertions (534 → materially down) | sparq-fedplan | 1 | sonnet | `crates/sparq-fedplan/**` (tests) — baseline re-seed coordinated after 6 merges (same JSON file ⇒ dep-sequenced) |
| 8 | proptest: RDFC-1.0 determinism/idempotence | sparq-canon | 2 | sonnet | `crates/sparq-canon/Cargo.toml`, `crates/sparq-canon/tests/proptest_canon_determinism.rs` |
| 9 | metamorphic nightly lane (TLP/NoREC driver + workflow) | sparq-metamorph | 2 | sonnet | `crates/sparq-metamorph/**`, `.github/workflows/metamorph.yml` (new) |
| 10 | Kani: bounded merge-join ≡ nested-loop (B-3 next wave) | sparq-substrate | 2 | opus | `crates/sparq-substrate/src/join.rs` (`#[cfg(kani)]` module only), `ci/formal-verification.toml`, `.github/workflows/kani.yml` — **dep: after bead 1** |
| 11 | WASM parity: pinned conformance sample wasm32 ≡ native | sparq-wasm | 2 | sonnet | `crates/sparq-wasm/tests/conformance_parity.rs` |
| 12 | spike: OCC txn lost-update tests + loom applicability verdict | sparq-engine | 2 | sonnet | `crates/sparq-engine/tests/txn_concurrency.rs` — **dep: after bead 3** |
| 13 | codify §8 checklist into AGENTS.md | AGENTS.md | 2 | haiku | `AGENTS.md` |

Deps (edges created): `sq-3dyje.10` after `.1`; `sq-3dyje.12` after `.3`; `sq-3dyje.7` after
`.6` (shared `bench/mutants-baseline.json`); `.6`/`.7` both noted as prerequisites of the
sq-qcnn.27 gating promotion (comment added there); an RDF-star generator note was added to
sq-qcnn.6 and the §8 checklist reference to sq-jocpn. Everything else parallelizes freely.

## 10. Non-duplication register

Owned elsewhere — deliberately **not** beaded here: value-level multi-oracle differential +
generator extension + triage + CI ratchet (open sq-qcnn.5–.10); coverage/mutation raises and
lane repair (sq-qcnn.27/.28/.30/.41); Kani Phase-1 wiring + runner constraints
(sq-sqtk2.5/.6/.7), vectors-lane repair (sq-otpxg); native Turtle tokenizer (sq-jocpn —
consumes §8); site/GUI testing (sq-ymr2e.\*); ZK/MPC functional-correctness + CI gating
(zk-correctness program beads) and **external audit (sq-qhy4, PENDING — no soundness claim)**;
`cfg(miri)` input-scaling (sq-0s15k); Copilot/CI throughput programs.

## 11. Honesty caveats

- Work-box measurements are non-canonical; no performance numbers appear in this record.
- "PROVED" anywhere above means bounded/complete-domain Kani with stated bounds and the
  Kani/CBMC TCB — never more. The `sparq-vectors` totality claim is currently NOT
  re-established by CI (§3).
- Nothing in this record or its beads asserts cryptographic soundness of the ZK/MPC estate:
  the v1 verifier's external accredited-cryptographer audit is PENDING (sq-qhy4) and
  `sparq-mpc` is semi-honest-only. Beads 1–13 are all outside the ZK/MPC surface.
- The two sub-agent audits this record synthesizes were spot-verified with independent
  greps (proptest/kani/loom counts, mutants baseline, protocol-lane wiring); the brief's
  incorrect premises are corrected in §1.1 rather than silently adopted.
