# Fable Work Plan — a token-efficient operating charter for a scarce high-IQ model

Status: design record (planning). Author: SPARQ synthesis stage. Do not treat any "sound/proven"
label herein as achieved until the external cryptographer audit (`sq-qhy4`) clears.

> This plan is written for **Claude Fable** — an expensive, stronger model coming online for
> sparq — and for the cheap fleet (Opus 4.8 / Sonnet 4.6 / Haiku 4.5) that works under its
> direction. It is deliberately opinionated about *where Fable is NOT allowed to spend tokens*.

---

## 1. Executive summary + the CORE DOCTRINE

Opus 4.8 has already shipped the breadth: six reasoners on a CI-enforced zero-overhead substrate,
SERVICE federation, an SSRF egress perimeter, four W3C/OGC conformance lanes, SHACL 1.2, a
dark-first GUI, and a large ZK/MPC/trust-graph/PKG estate. What remains is a **small residue of
soundness-critical, novel-design, and model-capability-gated work** that a cheaper model would get
*wrong in ways worse than not doing it at all*. That residue — and only that residue — is Fable's job.

**CORE DOCTRINE (token-efficiency operating model).** Fable is a scarce high-IQ resource whose
tokens buy exactly four things: (1) *decomposition* of hard epics into disjoint, crisply-spec'd
child beads with acceptance tests; (2) *subtle-correctness and soundness design* (crypto circuits,
protocol composition, reasoner semantics, estimator/cost-model math, XSD boundary calls);
(3) *novel research judgment* (the revisit-with-Fable model-dependent verdicts); and (4) *final
verdicts* on the escalated subset of PRs. **Everything else is delegated** — broad recon,
disjoint implementation, mechanical verification, doc-sync, benchmarking — to the cheap fleet under
Fable's written spec. The operating rule is: *Fable writes the spec and the proof; the fleet writes
the code and runs the measurement* — **by default.** Fable's default posture is architect/reviewer,
**but Fable may *elect to implement a bead itself*** when the review/verify signal shows the cheap
fleet cannot (repeated mechanical-verify failures on that bead) OR the code is soundness-critical
enough to need Fable-grade authorship. That election is **Fable's own decision, made from the review
— not a fixed prohibition.** Crucially, when Fable implements it does so in a **scoped, isolated
worktree sub-task with a minimal brief — NEVER inline on the orchestration thread** — so authorship
never bloats the scarce main-thread context (§8). One Fable call per epic amortises across N cheap
implementers; one Fable verdict clears a PR that would otherwise wait on the human. If Fable is
reading a whole file to answer a fact, or writing a kernel body it did *not* consciously elect to
author from the review signal, or re-running a settled negative — the doctrine is being violated.

**HONESTY (binding).** This repo mandates empirical honesty and soundness-first. Prior negative
verdicts (terse tokens, neurosymbolic KB) *died on proxy-vs-real-token artifacts* — every
measurement below specifies **real cache-discounted transcript tokens** (`message.usage`,
`1.0·input + 0.1·cache_read + 1.25·cache_creation`; **no `count_tokens`, no char proxy**), **N≥30**,
deterministic/blinded quality grading, and a **pre-registered kill-criterion**. Extensions are never
folded into standards totals. Conformance is never faked. A wrong "completeness/sound" claim is worse
than a null.

---

## 2. Operating model — how Fable collaborates with the cheap fleet

### 2.1 The delegation contract

| Layer | Model | Responsibility | Token posture |
|---|---|---|---|
| Recon | Haiku (`sparq-pkg-nl`, `query-pkg`, `ast-grep`, Explore) | Ground each spec against current code; enumerate call sites/fields; assemble evidence packs | Cheap, high-volume |
| Bulk impl | Sonnet (`sparq-rust-impl`*) | Implement disjoint, de-risked, single-crate beads against a Fable spec + failing test | Cheap bulk |
| Hard impl | Opus (`sparq-rust-feature`, `sparq-site`, `sparq-ci-infra`) | Genuinely-hard implementation, frontend, CI/supply-chain | Mid |
| Mechanical verify | Haiku (`sparq-verify-mechanical`*) | Objective checklist: gates green in both feature states, tests non-vacuous, opt-in respected, README/SKILL synced, no hardcoded perf; **arms clean low-risk PRs, escalates the rest** | Cheap |
| Architect (front) | **Fable** (`sparq-architect`*) | Decompose epic → design record + N disjoint spec'd child beads {crate, model_tier, invariant, acceptance_test} | Scarce; ~1 call/epic |
| Reviewer (verdict) | **Fable** (`sparq-reviewer`*) | Per-PR verdict `{honest, sound_as_scoped, recommend_arm, concerns[]}` on the **escalated subset only** | Scarce; only escalated diffs |
| Escalate-to-implement | **Fable** (scoped worktree sub-task) | The rare escalation outcome: when the review signal shows the fleet cannot land a bead (repeated mechanical-verify failures) OR it is soundness-critical, Fable *elects* (`disposition: fable_implements`, §6.2) to author the fix itself in an **isolated worktree**, then re-enters mechanical-verify | Scarce; rare, review-gated |

`*` = proposed agent (see §6). Fable appears at exactly two points: the **front** (decompose) and the
**escalation target** (verdict — plus the rare `fable_implements` outcome above). It never does recon
and never re-judges the clean PRs the mechanical tier already armed; it writes kernels **only** by its
own review-driven `fable_implements` election (§6.2), in an isolated worktree — never as a default and
never inline on the orchestration thread.

### 2.2 Prompt-cache hygiene & structured briefs

- **Every producing stage returns a SCHEMA'd object.** The schema-guard trap
  (`feedback-workflow-agent-schema-guard`): an unschema'd stage returns a *string*, so a downstream
  `if (r.pr_url)` guard silently evals false and the dependent Fable-verdict stage is *skipped with no
  error*. `autonomous-scheduler.js` already does this via `IMPL_SCHEMA`/`VERDICT_SCHEMA`; every new
  workflow must too.
- **Stable prefixes for cache reuse.** Fable's system prompt + the shared contract are fixed; per-task
  payload goes last so `cache_read` dominates. Evidence packs are Haiku-assembled and diff-scoped so
  Fable reads *only the changed lines + the relevant test + the one audit doc*, never a whole file.
- **Batching.** Verdict fan-outs (`fable-soundness-verdict`) run one Fable call per PR over a
  Haiku-prepared pack — never a single mega-context of all PRs.

### 2.3 Verdict-gated arming (binding discipline)

Arming iterates **verdict objects**, never a blanket loop over PR numbers keyed on CI
`mergeStateStatus` (`feedback-arm-gate-on-verdict` — a blanket loop once armed an `honest=false` PR).
A PR arms iff its own verdict has `honest=true && recommend_arm=true`; anything `honest=false` stays
**held** until the fix is confirmed landed on main. Fable holds only the genuinely external-audit-gated
(`sq-qhy4`) items; it *gives* the verdict the maintainer currently must give for everything else. The
verdict's `disposition` (§6.2: `arm | request_changes | hold | fable_implements`) drives the outcome:
`arm` on a clean `recommend_arm=true`; `request_changes`/`hold` bounce back to the fleet or the audit
gate; **`fable_implements`** is the rare escalation where Fable authors the fix itself in a scoped
isolated worktree, after which the normal mechanical-verify → arm path resumes.

### 2.4 Researcher-PR gotcha

A Fable/Sonnet research fan-out must use Explore/general-purpose or an explicit "findings only, no PR"
instruction — `sparq-researcher` opens its *own* `research/` PR per fragment
(`feedback-workflow-researcher-pr-gotcha`). Reserve PR-opening for the single synthesis/architect stage.

---

## 3. Workstream A — model-capability-gated VERDICTS to re-run (`revisit-with-fable`, #1111)

The maintainer flagged the neurosymbolic-KB direction (#1111) as model-dependent. The honest finding:
the five sub-directions are **not** equally model-gated. Rank by genuine Fable-leverage; every harness
is confirmed **present and re-runnable as a MODEL SWAP, not a rebuild**.

| # | Experiment | Harness (real telemetry) | Fable role | Kill-criterion |
|---|---|---|---|---|
| A1 | **FO-KM: schema.org ≫ gUFO** — the flip candidate | `bench/fo-km/{analyze.py,tasks.jsonl,overlays/*.ttl}` + `pkg-query --extra-graph <overlay> --close owl-rl`; `analyze.py` miner over `[FOKM arm=<fo> task=<id> model=<tier>]` transcripts; paired schema.org−gUFO **accuracy** delta per model tier, Wilcoxon + bootstrap 95% CI, **N≥30** TH/ER/CC | Cheap fleet fills the Haiku→Sonnet→Opus rungs; **Fable only the top arm** (~1 arm × ≥30 tasks). Fable's IQ goes to the *design judgment the run unlocks* | gUFO "wins back" iff at the Fable rung it matches-or-beats schema.org at **p<0.05, CI excludes 0**, AND rich-FO-only structural queries now resolve. If flat across tiers → fluency-penalty is a law; **stop** |
| A2 | **PKG-dogfood** — the cheap NL-tool win (positive, strengthens) | `bench/pkg-dogfood/{tokens_real.py,analyze3.py,tasks/abm_tasks.json}`; real `message.usage`; $-weighted with Fable priced as orchestrator; **N≥30** stratified + the 10+10 `sq-4va4l` boundary set as the **soundness gate** | (1) cheap re-price to confirm the delegation $-win *widens* (no Fable reasoning). (2) **Fable re-curates the PKG** (`crates/sparq-kb` findings, calibrate `pkg:confidence`, tighten provenance) → measure whether the **abstain set shrinks** | Answerable-set may grow **only if** off-PKG stratum stays **1.0 abstain-precision (0 hallucinations)**. Any hallucination on the negative stratum → revert curation |
| A3 | **Provenance-driven extraction fidelity** | `bench/pkg-dogfood` token-A/B re-used end-to-end; extraction graded by a **deterministic grounding-resolver** (citation-grounding rate, justification-entailment rate, sampled audit precision), **N≥30** papers, paired Fable-directed vs Haiku-bulk | Fixture-only A/B (no live access): does Fable-tier extraction earn a **higher trust tier** than the `secx:Conjectured` ceiling cheap extraction forces? | Adopt a higher tier only if grounding+entailment clear a pre-registered bar with 0 mis-cites on audit sample; else keep `secx:Conjectured` |
| A4 | **terse (K:/V()) — token axis DEAD BY ARITHMETIC** | — | **Do NOT re-measure the token axis.** The ~14k session floor dominates for any model; Fable's larger prompt makes the floor *more* dominant. Bank the negative | N/A — pre-declared dead. Only the **URI-hiding accuracy** sub-lever (A4′) is re-runnable |
| A4′ | URI-hiding accuracy (opaque-id rescue) | the existing **blinded N=35** A/B (opaque-id / informative-slug / null-control strata, deterministic answer-set grading), Fable as the *answering* model | cheap fleet builds fixtures; Fable only answers | If the hidden−raw delta on the **opaque-id** stratum regresses toward the informative-slug Δ≈0 → drop URI-hiding for strong models |
| A5 | **ast-grep / compacted-AST self-ergonomics** | passive: `bench/ast-compact/mine.py` + the `RESULTS-astgrep.md` method applied to **Fable's own** `[task/tool-usage]`-tagged working sessions | **Passively mine, do not commission a dedicated fan-out.** Keep the AGENTS.md rule (scoped lookup→Read; whole-file/codemod→compacted skeleton) until data moves | Act only if the passive signal is large and decision-relevant |

**Ordering for Fable time:** A1 (FO-KM ladder) → A2 (PKG re-curation + abstain boundary) → A3
(extraction-fidelity fixture A/B) → [maintainer-unblocked live trawl + EC2 KGE] → A4′ (cheap) →
A5 (passive). The compute-gated companions — FO-KM **Metric-2 KGE** (`crates/sparq-vectors/src/eval.rs`,
`gufo_prior` currently a no-op stub asserted at `eval.rs:1404`) and provenance **Phase-4** weighted
KGE — are **EC2-gated**: Fable *directs*, the fleet + a self-terminating box *executes*, adopt only on
pre-registered lift, abandon honestly if null (the literature reports inconsistent KGE lift).

---

## 4. Workstream B — FEATURES to build (Fable-vs-fleet split)

Ordered by Fable-leverage. "Fable does" = the design/proof only; "Fleet does" = everything mechanical.

### B-tier 1 — soundness-critical crypto (design not credential-gated; the *label* is `sq-qhy4`-gated)

| Bead / epic | Feature | Fable does | Fleet does | Effort |
|---|---|---|---|---|
| `sq-rsd3v` (+`.1`,`.2`) | ZK inference + credentials: in-circuit `derivation_step`, single-use nullifier, N3 witnessed-rule-shape, unlinkable presentation. **Verified absent by grep** — today ships a host-side structural re-check with *zero* antecedent privacy | Design the Poseidon2-domain-separated nullifier + the sound in-circuit RDFS derivation relation (variable-sharing equalities + set-membership anchoring); write the soundness argument | `bb gates` regressions, forge-and-verify tests | XL |
| `sq-qhy4` (audit gate) | **Noir circuit under-constraint + commitment domain-separation** de-risk (does NOT replace the human audit) | Adversarial constraint-satisfaction: construct a candidate malicious witness / colliding leaf for each `zk/compose/*/main.nr`; probe flat-Poseidon2 length-IV cross-method / cross-position(S/P/O) / dual-leaf collisions | Extract every `main.nr` public-input signature; build the circuit↔`reconstruct_public_inputs` (`verifier.rs:4786`) cross-ref table | XL |
| (none) | **ZK verifier `bind_*` composition** — can any single obligation be satisfied while the *aggregate* statement is false? (attribution-bit off-by-one at `verifier.rs:3981`, hidden-revocation index binding, holder-PoP field-0 ordering) | Hold all ~13 `bind_*` obligations simultaneously; search for a satisfying-but-false manifest | Enumerate prover-supplied vs external-anchored vs byte-bound manifest fields | L |
| `sq-tu4e` | **Trust-graph conflict semantics** — is conflicting-issuer deny-on-disagreement even *expressible* under input-only stratified NAF? (a silent-security-hole question). NOT credential-gated | Decide which conflict semantics are expressible in the shipped stratified-NAF engine (`incremental.rs:2130` rejects NAF over derived predicates) vs need an engine extension; termination analysis over external-graph extents | Wire whichever admission rules Fable proves reachable | L |
| `sq-wvne` | **Trust-graph unlinkability** — ZKAPs-grade 3-part composite (hidden-issuer + ZK holder-PoP + nullifier); clear-WebID binding is in tension with anonymity | Design the composition that replaces clear-WebID holder binding with in-ZK holder-PoP + single-use nullifier while preserving unlinkability | Wire the composite once specified | L |
| `sq-0jsc` (+`sq-aaop`,`sq-wj4k`,`sq-km34.*`,`sq-yyro`) | **Malicious-MPC** — UC/composition proofs, IT-MAC authenticated abort, dealer-less VSS/PRSS | Justify why `secure_equal` *opening* a value mid-pipeline composes (naive sequential composition does NOT); reconcile the coZK 2025/1026 inconsistent-extended-witness leak; batched MAC-check RLC soundness + selective-failure/abort leakage | Transport/instrumentation plumbing; `adversarial_tests.rs` | XL |
| `sq-0dksu` (+`sq-nrwqs`,`sq-dz10l`) | **Security-properties ontology** for ZK/MPC proof-admissibility (5/8 built) | The maintainer-requested **critical evaluation** of next steps + assurance-tier semantics (#1001) + DPV-alignment depth (#1002); cross-estate coupling to `sq-0jsc`/`sq-1s2.5` | `sq-nrwqs` resolve annotation graph from bundled `secprop-methods.ttl`; `sq-dz10l` per-method MPC annotations | M |

### B-tier 2 — reasoning frontiers (subtle-correctness; a wrong call ships an unsound reasoner)

| Bead | Feature | Fable does | Fleet does | Effort |
|---|---|---|---|---|
| `sq-bn2t8` | **OWL 2 QL query-rewriting** (PerfectRef/combined) — the last reasoning-profile gap; production tree-witness (`treewitness.rs`) + NP-complete UCQ-minimisation (`minimise.rs`) with **two silent-answer-loss traps** | Spec the strict CQ-shape gate that **REJECTS** non-CQ shapes (never silently loses answers) + sound UCQ minimisation + certain-answer proof; hand-check the DL-Lite oracle extension | Build the rewrite plumbing (`perfectref.rs`) + engine seam + entailment conformance harness | XL |
| (none) | **OWL 2 Direct/DL (hyper)tableau** — 66 direct-semantics cases permanently `OutOfScope`; undecidable in full; no Rust hypertableau exists (`research/reasoner-suite-on-substrate.md §2.6`) | Decide whether a scoped, honestly-labelled EL++-toward-DL or a genuine tableau track is worth it; prove the soundness envelope; specify the fragment + external-audit gate | Classifier/normalisation plumbing once semantics pinned | XL |
| (none) | **Datalog/existential regime** (chase + stratified negation + aggregates — the Nemo/VLog/RDFox tier, `research/inference-sota.md §1.3-1.4`) | Prove the hard parts: chase-termination (acyclicity classes), well-founded/stratified-negation soundness, aggregate stratification | Rule engine + expressivity ratchet | XL |
| **`sq-rhspl`** (steering; linked **#1307** — see note) | **beyond-RL sound boundary** — 13 OWL-RL divergences are provably outside RL; some (differentFrom contrapositives) *are* sound RDF-Based entailments RL omits for polynomiality | Per-rule soundness + termination proof; maintainer steer on the polynomiality trade | Small code once each rule is proven | M |

> **Bead-id note (corrected 2026-07-01).** The beyond-RL steering work is **already tracked** by the
> OPEN bead **`sq-rhspl`** (P3, DESIGN-FIRST + MAINTAINER-STEER, linked to **#1307**); the
> ceiling-hardening bead **`sq-350ms`** is CLOSED (via #1308). An earlier draft of this note wrongly
> called these ids "absent" — that was an artifact of grepping the **stale committed
> `.beads/issues.jsonl` snapshot** (which lags the live Dolt DB by ~1000+ issues) instead of `bd show`.
> Both exist. Do NOT implement the beyond-RL extension without the maintainer's steer.

### B-tier 3 — federation query-optimisation (planning-algorithm judgment)

| Bead | Feature | Split |
|---|---|---|
| `sq-dnko` (epic) + `sq-a35t`/`sq-vf7q`/`sq-7s4z` | Cost-based source selection, ANAPSID-style non-blocking streaming joins w/ spill, live adaptive re-planning | **Fable-gated** design cluster (shares the local estimator/cost-model of §5-perf). Two documented `service` Skips + `sq-sjkj` brTPF pushdown = mechanical fleet |

### B-tier 4 — performance (Opus shipped the substrate; Fable owns the LAYERS ABOVE the kernels)

The substrate kernels (`sparq-substrate`, monomorphic, gate-enforced) are near-optimal — **do NOT
re-plan them.** The wins are the model/estimator/cost-gate layers Opus deliberately deferred.

| Bead | Feature | Fable does | Fleet does | Effort |
|---|---|---|---|---|
| `sq-hvfe` (first block only; **no bead for wiring**) | **Vector-at-a-time engine (M4)** — `chunk.rs` DataChunk + kernels are BUILT behind opt-in `vectorized` but **unreferenced in `exec.rs`** (still 100% row-materialising) | Design the morsel-driven scan→filter→merge-join→count pipeline, selection-vector propagation, VOILA order-free operator spec, row/columnar coexistence, and the invariant keeping wasm-simd128 autovectorisation + **bit-identical bundle when OFF** | Kernel bodies + wiring; EXPLAIN cost categories | XL |
| (none; research §A5) | **Cardinality estimation** — Index-Based Join Sampling + never-underestimate guard. Current `goo_pick`/`pattern_var_ndv` (`exec.rs ~4749/4804`) is a pure independence product, wrong by orders of magnitude on correlated joins | Design the fixed-budget index-probe sampler over the six permutations + a SafeBound/LpBound guard (order-only, never changes results) | Fold into GOO scoring; q-error A/B (EC2-gated) | L |
| (none) | **DPccp/DPhyp join ordering** — there is **NO** DP enumerator anywhere (greedy GOO only); docs falsely claim "GOO/DPccp" | Design DPccp (connected-subgraph enumeration) with a pattern-count fallback threshold; seed cost from the sampler; **fix the doc/comment honesty** | Enumerator body | L |
| `sq-6i40` (redesign, open); `sq-p6p6` (closed, superseded); `sq-0g6g` (EC2 gate) | **Adaptive re-optimization** — the divergence-triggered arm-swap **failed once** (no local win); prior-negative = Fable-worthy | Design a mid-query checkpoint using observed intermediate cardinalities (free, since every intermediate materialises) to re-order / switch strategy / trigger reducers | Wire it; perf validation EC2-gated |  L |
| `sq-0g6g` (EC2 gate); features `yannakakis`,`semijoin-bitmap` exist | **Adaptive reducer dispatch** — two *proven-correct* reducers sit dormant behind static flags + a constant gate (`YANNAKAKIS_MIN_REL=4096`) purely because the default-flip needs EC2 sign-off | Replace the static gate with a **runtime** adaptive cost model that self-guards against the pure-overhead case → unlocks them **without** the EC2 default-flip. Keep a native-adaptive / wasm-opt-in split | Wiring; watch `wasm_bundle_bytes` floor | M |
| (none) | **WCOJ/LFTJ** — `build_trie` (`exec.rs ~4850`) heap-allocs one `Vec<Id>` **per tuple** then re-sorts a *already-sorted* index; `wcoj_global_order` (`~4978`) ignores AGM fractional-edge-cover | Design the permutation→column-order mapping (non-trivial) + AGM/min-width variable ordering | Make `TrieIter` navigate the sorted slice directly; branchless intersection | M each |
| (none; research §A2) | **FastLanes/Elias-Fano compressed-column decode** (`sq-96hp1` impl is mechanical) | Codec choice (storage-format judgement) | Scalar decode kernel proven to autovectorise x86/ARM/wasm; codec A/B EC2-gated | L |

### B-tier 5 — feature-completeness (mostly FLEET; Fable directs, never touches)

Delegate entirely unless flagged. JSON-LD result-format wiring (`sq-oy1f.15`/`.20`/`.3`,
`sq-oy1f.21`/`.22`, `sq-gcs5q`), GeoSPARQL geodesic distance + GML tessellation (`sq-47vu`), RSP
window gaps, full-text analyzers, RDF/XML ratchet (`sq-koshe`), SHACL long-tail
(`sq-11a`/`sq-uz0`/`sq-8ro`/`sq-jvn`/`sq-4ng`/`sq-ou3`, `sq-1jemy`), GUI stubs + wasm-portability spike
(`sq-zeai`, `sq-ixc3`, `sq-tp1m`, `sq-lyp8`, `sq-2mke`), website content-reduction
(`sq-vw3ax.6`/`.8`/`.9`/`.10`), and **doc-currency reconciliation** (`sq-6gob` stale
"SERVICE not implemented"; `sq-oy1f.4` mislabelled blocked) — **all fleet.** SPARQL 1.1/1.2
conformance is at its honest ceiling (1225 pass + 4 proven expected-file divergences = 100%) — **do
NOT re-plan it**; just file the 4 upstream `rdf-tests` issues via the fleet.

---

## 5. Workstream C — the CODEBASE REVIEW CAMPAIGN (by lens)

A standing, cadenced review with a reusable workflow. **Fable reviews only what a cheap model cannot
adjudicate;** the mechanical enumeration that feeds each review is Haiku/Sonnet work.

### 5.1 Review lenses, targets, cadence

| Lens | Real targets (files/crates) | What Fable adjudicates | Cadence |
|---|---|---|---|
| **Security — crypto core** | `crates/sparq-zk-compose/src/verifier.rs`, `crates/sparq-zk/src/{commit,poseidon2,dual_leaf,encode,sig}.rs`, `crates/sparq-mpc/src/{authenticated,robust,oblivious_join,hidden_path,hidden_distinct}.rs` | Circuit under-constraint, commitment domain-separation, `bind_*` composition, MPC abort/leakage (see §4 B-tier 1) | Per ZK/MPC PR + a monthly deep pass |
| **Security — perimeter** | `crates/sparq-engine/src/service.rs`, `crates/sparq-fedclient/src/{source,discovery}.rs`, `crates/sparq-server/src/http.rs` | **Fleet, not Fable.** env-proxy SSRF bypass (`ureq` default `proxy: try_from_env()`, no `.proxy(None)` at `service.rs:1393`/`source.rs:667`/`discovery.rs:675`), redirect re-vet test, `is_forbidden_ip` embedded-v4 completeness, forwarded-WebID deployment note. Fable adjudicates *only* the vetted-proxy-support policy (one line) | Fleet sweep; Fable one-line policy call |
| **Memory-safety** | `crates/sparq-core/src/{dict,store,lib,dictspill}.rs`, `compliance/memsafety/unsafe-register.md` | The `MappedDict::validate` completeness invariant — does every unchecked-trust field have a validated-at-open check? — and the residual UB neither Miri nor ASAN can observe | Quarterly + on any new `unsafe` |
| **Performance** | `crates/sparq-engine/src/{exec,chunk,cs,explain_json}.rs`, `crates/sparq-core/src/{store,compress}.rs`, `crates/sparq-substrate/src/join.rs`, `scripts/check-no-dyn-dispatch.py`, `scripts/perf-gate.py` | Operator-model design, estimator/guard math, cost-gate design (see §4 B-tier 4); **extend the no-dyn gate's scanned set** the moment vectorized kernels wire in | Per perf PR; design-led not review-led |
| **Feature-completeness** | `crates/sparq-conformance/src/scoreboard.rs`, `crates/sparq-conformance/tests/scoreboard_floors.rs`, the reasoner/QL/geo/rsp READMEs | Only the reasoning-frontier soundness envelopes (§4 B-tier 2). Everything else is a fleet ratchet grind | Monthly floor-review |
| **Result-accuracy** | `crates/sparq-substrate/src/{numeric,compare,join}.rs`, `crates/sparq-engine/src/exec.rs`, `crates/sparq-bench/src/fuzz.rs`, `crates/sparq-canon/src/rdf12.rs` | The value-level differential oracle *spec*; the ORDER-BY-vs-relational recheck asymmetry seam; the two-convention XSD-double lexical adjudication; `fn:round` fp reasoning; which mutation survivors are real bugs vs equivalent | Bi-weekly accuracy pass |

### 5.2 Concrete result-accuracy targets (the highest-yield lens — invisible to green tests)

- **Value-level multi-oracle differential (new, XL).** `fuzz.rs:307` checks **cardinality only**
  (`sparq_full != oxi` on solution *counts*) for non-ORDER-BY queries — blind to wrong bound *values*
  that preserve row count; `gen_graph` emits no dateTime/duration/boolean, `gen_query` no
  aggregate/BIND-arithmetic/string/CONSTRUCT. **Fable writes the correctness spec**: compare the full
  canonicalised binding **multiset** term-by-term against the oracle, add a **second independent oracle**
  (Jena CLI / rdflib subprocess) to catch sparq+Oxigraph shared-assumption bugs, and specify the
  oracle-normalisation rules (legitimate impl-defined divergence: GROUP_CONCAT order, double lexical
  form). Fleet builds the harness + corpora + subprocess plumbing + graph-isomorphism compare via
  `sparq-canon`.
- **ORDER BY / MIN / MAX numeric precision (M).** `compare_terms` (`compare.rs:160`) decides numerics
  via `as_f64().partial_cmp`, with **no exact recheck**, whereas the relational `=/<` path
  (`exec.rs cmp_expr:7060`/`equal_expr:7099`) **does** recheck when f64 collapses distinct values
  (integers >2^53, hp-decimals). `num_compare` (`exec.rs:6306`, MIN/MAX) *also* rechecks — so the
  asymmetry is inconsistent even internally. **Fable designs a perf-neutral `exact_cmp` hook** through
  the `CompareTerm` trait (recheck only on f64 tie, no vtable, no hot-loop regression).
- **XSD double/float lexical (M).** `Num::lexical`/`fmt_xsd_double` print integral doubles as plain
  `6` (W3C-suite-pragmatic) while `canonical_lexical` emits `6.0E0` (XSD-mandatory scientific); which
  method each surface (STR/EBV/serialize/aggregate/value_key) calls silently decides conformance. Also
  `as_num` (`exec.rs:7314`) uses `parse::<f64>()` which **rejects** XSD `INF`/`-INF`/`NaN`. **Fable
  adjudicates the per-surface policy** (STR/EBV/serialize must be XSD-canonical); the INF/NaN routing
  fix is mechanical.
- **`fn:round` fp boundary (S).** `Num::round` float tier is `(f+0.5).floor()` (`numeric.rs:431`) —
  the classic double-rounding defect (`0.49999999999999994` → 1, not 0). Fable confirms the failing
  inputs + the correct round-half-to-+∞ for float tiers.
- **Coverage/mutation ratchet gap (fleet, M/L).** `sparq-substrate` — the correctness core — is in
  **none** of `coverage-floor.json`, `coverage-presence.json`, `mutants-baseline.json`,
  `scripts/coverage.sh`, `feature-matrix.yml` (its numeric/compare/join are default-OFF features, so
  `llvm-cov` instruments an empty crate). Fleet adds the `--features numeric,join,compare,rows` measure
  case + seeds floors/baselines (`sq-hbg7`/`sq-8kt3`/`sq-qcnn`/`sq-52su`); **Fable only adjudicates the
  surviving-mutant lists** on `numeric.rs`/`compare.rs`/`exec.rs` (real bug → write the killing
  assertion; equivalent → document like the `sparq-reason` 6 defensive-bound survivors), then promote
  the lane advisory→gating.

### 5.3 Reusable review WORKFLOW (`fable-lens-review`)

```text
Stage 1 (HAIKU recon): enumerate the lens's targets (call sites / field bindings / cast sites /
         public-input signatures) into a SCHEMA'd evidence pack. No judgment.  [query-pkg + ast-grep]
Stage 2 (SONNET, optional): assemble the cross-reference tables (circuit↔reconstruct, manifest-field
         provenance, cast alignment/length) the review needs. SCHEMA'd.
Stage 3 (FABLE / sparq-reviewer): adjudicate ONLY the subtle question for that lens; return
         {findings[], soundness_verdict, follow_up_beads[]}. Reads pack + tables, never whole files.
Stage 4 (orchestrator glue): the fleet files follow_up_beads via `bd create`; arm-on-verdict.
```

Fable touches only Stage 3, on a Haiku/Sonnet-prepared pack. Follow-ups become beads (repo hygiene:
TODOs → beads, never markdown checklists).

---

## 6. Collaboration design — agent roster + workflow catalogue

### 6.1 Existing roster (all `opus` except `sparq-pkg-nl`)

| Agent | Model | Role |
|---|---|---|
| `sparq-rust-feature` | opus | Opt-in feature-gated Rust crate impl; gates both feature states |
| `sparq-site` / `sparq-ci-infra` / `sparq-docs` | opus | Frontend / CI-supply-chain / doc-only reconciliation |
| `sparq-researcher` | opus | Deep research → `research/` design record (opens its **own** PR — see gotcha) |
| `sparq-merge-fixer` | opus | Unblock a stuck/conflicting PR on its existing branch |
| `sparq-perf-reviewer` | opus | Narrow perf-discretion verdict hook on `gh pr merge --auto` |
| `sparq-pkg-nl` | **haiku** | The only non-opus agent: NL→SPARQL→run→NL PKG round-trip |
| `sparq-workload-triage` | opus | Read-only local-vs-EC2 placement (places, never launches) |
| `compliance-engineer` / `compliance-auditor` | opus | Certification control impl + adversarial audit loop |

**Root gap: model homogeneity.** 11/12 dispatchable agents are `opus`, one is `haiku`; there is **no
sonnet tier and no fable tier**. A Fable orchestrator today has no cheaper implementer/verifier below
it and no Fable reviewer to escalate to.

### 6.2 Proposed agents (with model)

| Agent | Model | Purpose | Why this model |
|---|---|---|---|
| `sparq-architect` | **fable** | The missing FRONT stage: epic → design record + N disjoint spec'd child beads {crate, model_tier, invariant, acceptance_test}. Read-only + writes `research/` + emits beads; does NOT implement. Supersedes `sparq-researcher` for the decompose-and-spec job. Inherits `proceed-and-document` | Disjoint decomposition + invariant design is Fable's scarce edge; one call/epic amortises across N cheap impls |
| `sparq-reviewer` | **fable** | The missing final VERDICT-GIVER, invoked **only** on the escalated subset (failed mechanical verify, or ZK/MPC/reasoner/engine-correctness/novel-algo/honesty surfaces). Returns per-PR `{honest, sound_as_scoped, recommend_arm, disposition: arm\|request_changes\|hold\|fable_implements, concerns[]}`. **`disposition=fable_implements`** triggers a **scoped isolated-worktree implementation task authored by Fable** (the fleet couldn't land it, or it is soundness-critical), after which the normal mechanical-verify → arm path resumes. Holds only `sq-qhy4`-gated items | Subtle soundness judgment is exactly what the pipeline currently punts to the human; reading only escalated diffs minimises tokens |
| `sparq-verify-mechanical` | **haiku** | The cheap mechanical verify pass (today runs at opus): gates green both feature states, tests non-vacuous, opt-in respected, README/SKILL synced, no hardcoded perf. Returns `{mechanical_ok, checks[], escalate, escalate_reason}`; **arms clean low-risk PRs, escalates the rest** | Objective checklist a cheap model does reliably; it is the escalation filter that keeps Fable's input tiny |
| `sparq-rust-impl` | **sonnet** | Bulk sibling of `sparq-rust-feature` for well-spec'd disjoint single-crate beads the architect has de-risked; escalates back up if it discovers the bead is actually hard | Once the spec + failing test exist, impl is mechanical; this is the missing cheap bulk-impl tier |
| `sparq-context-monitor` | **haiku** | Out-of-band context-hygiene observer (§8): reads the session transcript `.jsonl` and emits `{should_compact, confidence, reason, what_to_preserve[], externalize_first[]}`. **SIGNALS only** (cannot force compaction); greenlights only when `externalize_first` is empty. Its brief is the §8 trigger ruleset (bead `sq-sgu1.1`) | Watching a transcript for a clean compaction seam is cheap pattern-work; Haiku makes continuous observation ~free vs one Fable turn |

> **Provenance-trailer caveat (needs:user).** Every agent config and the workflow prompt strings
> hardcode `[OPUS-4.8]` + `Co-Authored-By: Claude Opus 4.8`. When the fleet is multi-model this
> **mislabels** Fable/Sonnet/Haiku diffs as Opus. A model-aware per-stage trailer is required — but
> `.claude/agents/*.md` is a **PROTECTED** surface (AGENTS.md rule 11: agents may not rewrite their
> own/sibling configs), so this edit is **maintainer-applied**, not agent-self-editable.

### 6.3 Workflow catalogue (which model does which stage)

| Workflow | Shape (model per stage) | When |
|---|---|---|
| **`fable-architect-drain`** (flagship) | 1 FABLE architect (decompose → design record + N disjoint beads, SCHEMA'd) → 2 HAIKU recon (ground specs, optional) → 3 SONNET fleet fan-out (impl N disjoint beads, `isolation:worktree`, SCHEMA'd) → 4 HAIKU mechanical-verify (arm clean, escalate rest) → 5 FABLE reviewer (escalated diffs only → verdict → arm-on-verdict); **branch:** if the reviewer's `disposition=fable_implements`, Fable authors the fix in an **isolated-worktree sub-task**, then **re-enters Stage 4 mechanical-verify** — the deliberately rare path that spends the scarce resource on code, gated on the review signal. **Fable touches Stage 1 (1 call) + Stage 5 (only failed-verify diffs, plus the rare self-implement branch)** | Default drain for a hard epic or a batch of underspecified/soundness-sensitive beads |
| **`fable-soundness-verdict`** | 1 HAIKU enumerate open honesty/soundness PRs + assemble each evidence pack (diff + tests + `research/zk-*-audit.md` + `SECURITY.md` lines, SCHEMA'd) → 2 FABLE reviewer fan-out (one call/PR → verdict) → 3 orchestrator glue (arm `recommend_arm=true`; hold `sq-qhy4`/`honest=false`) | Periodically, to clear the soundness pile the current loop leaves OPEN for @jeswr |
| **`model-tiered-scheduler-v2`** | Evolve `autonomous-scheduler.js`: keep disk-guard/frontier-read/`isolation:worktree`/schemas/backoff; make `pickAgent` return `{agentType, model}` from the architect-assigned tier; split verify into HAIKU mechanical (arms/escalates) → FABLE reviewer (escalated-only). Frontier + recon run HAIKU; **Fable never appears except as the escalation target** | Steady-state autonomous tick once the tiered agents exist |
| **`fable-lens-review`** (§5.3) | HAIKU recon → SONNET cross-ref tables → FABLE adjudicate → fleet files follow-up beads | The cadenced review campaign |

All new workflows must: give **every producing stage a schema** (schema-guard trap); use
Explore/"findings-only" for any research fan-out (researcher-PR gotcha); **arm on the verdict object**,
never a PR-number loop; and inherit `proceed-and-document` (never relabel unaudited ZK/MPC "sound").

---

## 7. Sequencing — first-week plan + kill-criteria

**Pre-req (fleet, day 0, needs:user for the trailer):** stand up the four proposed agents
(`sparq-architect`/`sparq-reviewer` fable, `sparq-verify-mechanical` haiku, `sparq-rust-impl` sonnet)
and the model-aware provenance trailer. Wire `fable-architect-drain` + `fable-soundness-verdict` with
schemas on every stage.

| Day | Fable action (scarce) | Fleet action (parallel) | Kill-criterion |
|---|---|---|---|
| 1 | **A1 FO-KM top-arm** (Fable-as-NL-tool over ≥30 TH/ER/CC tasks); pre-register K1/K2 margins **before** running | Fill Haiku/Sonnet/Opus rungs; broaden to the `sq-givgo` ≥30-task floor; add gist + BFO overlays | Flip only if schema.org−gUFO closes at Fable rung, **p<0.05, CI excludes 0**; flat → declare fluency-penalty a law, stop |
| 2 | **`sq-tu4e` reachability adjudication** (pure reasoning, NOT credential-gated) — is deny-on-disagreement expressible under input-only stratified NAF? | Enumerate `incremental.rs` NAF-over-derived rejections + external-graph admission rules | If provably unreachable without an engine extension, write the extension spec as a child bead; do not ship a silently-unsatisfiable rule |
| 3 | **`sq-rsd3v.1` nullifier design** (Poseidon2 domain-separated, single-use) + soundness sketch | Extract every `zk/compose/*/main.nr` public-input signature; build the circuit↔reconstruct table | Any candidate colliding leaf / free witness found → the design is unsound, iterate before spec'ing impl |
| 4 | **A2 PKG re-curation** design + `sq-bn2t8` OWL 2 QL CQ-shape-gate spec (reject non-CQ, never lose answers) | Re-price PKG-dogfood with Fable-as-orchestrator; assemble the `sq-4va4l` boundary set | Curation ships only if off-PKG stratum stays **1.0 abstain-precision**; QL rewriter only if the reject-gate is proven answer-preserving |
| 5 | **Perf estimator design** — Index-Based Join Sampling + never-underestimate guard spec; **run `fable-soundness-verdict`** over the open ZK/MPC pile | Build the sampler skeleton behind the design; assemble verdict evidence packs | Estimator ships only if the guard provably never under-estimates (order-only, results unchanged) |

Weekly rhythm thereafter: `fable-architect-drain` on the next B-tier-1/2 epic + `fable-lens-review` on
the bi-weekly accuracy lens + `fable-soundness-verdict` sweep. The mechanical frontier
(`autonomous-scheduler.js` → `model-tiered-scheduler-v2`) drains the ~281 ready beads **without Fable**.

---

## 8. Context hygiene — keeping Fable's thread lean + the compaction observer

**Motivation.** If Fable runs the main orchestration thread, the context window bloats and *every*
Fable turn re-reads a large window at high cost — the scarce-resource doctrine dies if the **THREAD
itself is fat.** Cheap-fleet delegation saves nothing when the orchestrator carries a bloated
transcript into each expensive turn. The fix is a **discipline** (externalize-then-shed) plus a
**cheap out-of-band observer**.

**Principle — externalize-then-shed.** Durable outcomes live in **commits / beads (`bd`) /
auto-memory / a `research/` doc** — *never* in the live thread and *never* in a scratch
`HANDOVER*.md`/`SESSION*.md` (repo hygiene forbids those; see AGENTS.md). The live thread stays a
**thin working set**; anything durable is **shed to a persistent store the moment it is captured.**
The thread is a scratchpad, not a ledger.

**The observer — `sparq-context-monitor` (proposed, model: haiku; §6.2).** A NEW out-of-band agent
that reads the session transcript `.jsonl` and emits a SCHEMA'd signal
`{should_compact: bool, confidence, reason, what_to_preserve[], externalize_first[]}`. Because it is
**Haiku, watching costs ~nothing** relative to one Fable turn. It **SIGNALS only** — it cannot force
compaction; the main thread (or the user via `/compact`, or the harness auto-summary at window
overflow) acts on the signal. Its value is triggering compaction **EARLY at a clean seam**, so the
harness never force-summarizes mid-thought at a bad boundary.

**Precondition gate.** The observer greenlights compaction **only when `externalize_first` is empty**;
otherwise it returns that list ("persist these first") and *defers*. Externalize, then compact —
never the reverse.

**Compaction-trigger ruleset** — compact WHEN (any of):

| Trigger | What the observer looks for | Action |
|---|---|---|
| 1. Clean seam | A coherent unit of work just completed AND its durable outcome is externalized (PR opened/merged, workflow result acted-on, bead created/closed, memory/doc written) | Compact — the safest, cheapest moment |
| 2. Window pressure | Context exceeds a budget threshold (~50% of window, or the per-turn input-token cost crosses a line) | Compact — **OVERRIDE trigger regardless of seam** |
| 3. Large-artifact shed | A big tool result (workflow output, file/log/search dump) was ingested, its conclusion extracted and acted on; the raw bytes are now dead weight | Compact — drop the raw bytes, keep the conclusion |
| 4. Topic switch | The conversation pivots to a task largely independent of the prior one's mechanics | Compact — prior mechanics no longer load-bearing |
| 5. Idle-loop repetition | Many near-identical `/loop` heartbeats have accumulated | Compact — collapse the repeats |
| 6. Pre-hard-reasoning | Right before a known-expensive Fable reasoning turn | Compact FIRST — so it starts from a minimal, relevant context |

**Do NOT compact WHEN (guards):**
- **(a) Mid-task with unsynthesized working state** — a plan half-formed, an edit in flight, a
  decision pending. Compaction would drop load-bearing memory not yet externalized. *Externalize first.*
- **(b) Immediately after an error/retry** where the failure details still drive the next attempt.
- **(c) An un-acted user instruction or un-captured decision** sits in recent context — externalize it
  to a bead/memory/doc, THEN compact.
- **(d) The unique, hard-won reasoning** in the thread is not yet written anywhere durable.

**This ruleset is itself the brief for the `sparq-context-monitor` agent** — Amendment-1 bead
`sq-sgu1.1` will stand it up. `[OPUS-4.8]`

---

## 9. What NOT to spend Fable on (`do_not_spend_fable_on`)

**Hard-gated (Fable must not auto-start):**
- **`sq-qhy4` / #811 — the external accredited-cryptographer audit.** Fable *de-risks* it (pre-finds
  obvious soundness bugs) but never replaces it; the production "sound/proven" label stays gated.
- **EC2 / perf-host benchmarks** — `sq-0g6g`/`sq-v8ixk`/`sq-n7f` canonical gates, `sq-5o5`/`sq-axky`/
  `sq-dbdk`/`sq-pjhc`/`sq-gafdh` gather beads, `sq-bj`/`sq-3l43`/`sq-hoaj`/`sq-w0ax`/`sq-xin1`/
  `sq-yc1q`/`sq-8lgn`/`sq-uubq` heavy tiers, and the FO-KM Metric-2 / provenance Phase-4 KGE runs:
  **design only, no dispatch without a maintainer greenlight.**
- **Dependabot majors** (#1278–1283: arrow 55→59, hdt 0.4→0.7.3, n3 1→2) — breaking-API/MSRV = a
  maintainer decision. Do not migrate major deps.
- **cargo-vet auto-certification / supply-chain auto-approval** — the advisory pipeline is already
  gated; keep it fresh, never auto-certify.
- **Maintainer-gated PRs** — #1155 (ZK VC-cryptosuite bridge, DIRTY, deliberately not auto-armed),
  #1260 (GUI stats, DIRTY), #1084 (release v0.1.0 + signing/publish creds `sq-v286.*`). Do not rebase
  or arm these.
- **Auth model #907 / `sq-70kx2`** (OIDC/JWT + RBAC + graph-level security) — the one feature
  deliberately NOT auto-built; needs the maintainer's security-model input first.
- **Steer/Decision items** — #1001/#1002 (assurance-tier + DPV depth, also `sq-qhy4`-gated), the
  beyond-RL bead (`sq-rhspl`, linked #1307; **maintainer-steer required**), logos (PR #823/#207/`sq-8pbx`),
  #1139 (Semantic Scholar key), `sq-vbq9` (Pages source), #1135 (Copilot/bundle-budget), and the ~40
  post-hoc `Steer:`/`Decision:` issues (these are maintainer *review* items, not new work).

**Soft-gated (a cheaper model is equal or better — do not burn Fable IQ):**
- **Implementation, by DEFAULT** — the fleet writes the code; delegating implementation is Fable's
  default posture, *not* an absolute prohibition. Fable overrides this **only** by its own
  review-driven `fable_implements` decision (§6.2/§6.3), and even then authors in a **scoped, isolated
  worktree** with a minimal brief, never inline on the orchestration thread (§8). Blanket
  kernel-authoring on the main thread remains the anti-pattern this charter exists to prevent.
- **The ~281 ready mechanical beads** — the entire fleet drain. All JSON-LD/Geo/RSP/full-text/RDF-XML/
  SHACL long-tail, GUI stub-filling + wasm-portability spike (`sq-zeai`), website content-reduction,
  doc-currency reconciliation (`sq-6gob`).
- **The SSRF perimeter fixes** (env-proxy `.proxy(None)`, redirect re-vet test, `is_forbidden_ip`
  completeness, forwarded-WebID deployment note) — a stronger model does not change the outcome; Fable
  adjudicates *only* the one-line vetted-proxy policy.
- **SPARQL 1.1/1.2 conformance** — at its honest ceiling (1225 + 4 proven expected-file divergences);
  just file the 4 upstream `rdf-tests` issues via the fleet.
- **The terse token axis (A4)** — **dead by arithmetic**; the empirically-honest move is to bank the
  negative and refuse to re-measure. Re-running a settled token negative on the most expensive model is
  the exact anti-pattern this charter exists to prevent.
- **ast-grep/compacted-AST self-ergonomics (A5)** — passively mine Fable's own transcripts; never a
  dedicated expensive fan-out.
