# Novel-Contribution Inventory + Identification Process

`[OPUS-4.8]` — Academic paper factory, epic **sq-gum8** phase 2 (novel-contribution
intake). This document does two things: (1) defines a **repeatable PROCESS** for
identifying genuinely-novel academic contributions from this evolving project, and
(2) records the **current ranked inventory** with a non-sycophantic honesty/readiness
verdict per candidate.

**Empirical-honesty is the load-bearing constraint of this whole document.** Two facts
govern every verdict below and must never be softened in a paper:

1. **The ZK/MPC security stack has NO external audit.** The single-prover ZK verifier
   was internally re-audited "sound-as-landed for its threat model" (bead sq-gbp4, after
   a v1 that was documented *NOT sound* — `research/zk-soundness-audit.md`), but no
   accredited external cryptographer has reviewed it; the crypto forge tests are
   toolchain-gated out of default CI; the collaborative/multi-prover path is explicitly
   re-open (`research/mpc-cozk-reaudit.md`) and the malicious-secure MPC layer is a stub.
   Bead **sq-qhy4** (external cryptographer audit) is **open and externally gated**.
   *No ZK/MPC security, privacy, integrity, or attestation property may be claimed as
   proven in a paper.* A coZK/MPC paper today is an **honest design / limitations /
   negative-result** contribution, NOT a soundness proof. Cite sq-qhy4 as the gate.
2. **All wall-clock numbers on the dev work-box are NON-CANONICAL.** The session runs on
   an AWS EC2 box (`-aws` kernel) and most figures elsewhere were gathered on an M1
   laptop or ephemeral EC2. Only **deterministic integer metrics** are canonical today:
   W3C/OGC conformance counts, byte-identity invariants, recall floors, gate/round/byte
   counts, differential-fuzz pass. A speed claim needs the **canonical runner**
   (`research/ci-ec2-design.md`, blocked on one IAM admin step) before publication.

---

## Part 1 — The identification PROCESS (the factory's intake)

This is a **re-runnable** procedure. It is designed to be triggered automatically as the
project evolves (see "Re-run triggers"), and to feed candidates to the downstream
paper-writing stages of the factory.

### 1.1 Inputs to scan (the corpus)

| Source | What to extract | Where |
| --- | --- | --- |
| **Crates** (`crates/*`) | what is actually IMPLEMENTED vs designed-only; lib/README capability tables; `#[ignore]`/`NotYetImplemented`/stub markers | `crates/sparq-*/src`, `README.md` |
| **`research/` docs** | the *claimed* novel technique, prior-art comparison, the author's own honesty hedges, negative results | `research/*.md` |
| **Beads** (`bd` / `.beads/issues.jsonl`) | open audit/gating beads (esp. sq-qhy4, sq-9hrn, sq-1gir), "designed-not-built" markers, the venue/factory epic itself | `.beads/issues.jsonl` |
| **Benchmark deltas** | which numbers exist, on what hardware, canonical vs work-box; documented negative results | `research/BENCHMARKS.md`, `bench/`, `research/*-measured.md`, `research/hw-bench-results.md` |
| **Conformance scoreboard** | the only fully-canonical evidence: ratchet floors per spec family | `crates/sparq-conformance/src/scoreboard.rs` |
| **Skills** (`skills/*/SKILL.md`) | distilled, measured findings already deemed durable knowledge | `.claude` skill tree |

### 1.2 The four screening criteria (a candidate must pass all four to be a "contribution")

For each candidate technique, score it on:

1. **Novelty vs prior art.** Is this a *new algorithm/construction/finding*, or a
   faithful re-implementation / engineering integration of clearly-cited prior art?
   Name the prior systems (ANAPSID, CostFed, ACORN, RDFox, QLever, SPDZ, coZK 2025/1026,
   …). **Integration is allowed to be a contribution** — but only if framed as a
   *systems/integration* contribution at the right venue, never dressed as a new
   algorithm. A measured *negative result* / *honest correction* is also a contribution
   if it overturns a literature expectation.
2. **Evidence available.** Is there canonical evidence (deterministic metric) *today*, or
   only work-box numbers / a design? Map to the readiness verdict (1.3).
3. **Generality.** Does the claim generalise beyond sparq's exact code, or is it a local
   tuning artefact? (e.g. a per-ISA prefetch tuning is local; a "filter-as-exact-BGP"
   pattern is general.)
4. **Honesty / soundness status.** For crypto: is the property *proven*, or merely
   *implemented and internally reviewed*? For perf: canonical or not? **This is the gate
   that vetoes overclaiming.** A candidate that can only be stated honestly as WIP/design
   must be *labelled* as such, not promoted.

### 1.3 The readiness verdict (assigned to every candidate)

- **PUBLISHABLE-NOW** — the central claim is *sound* and the supporting evidence is
  *canonical* today (deterministic conformance / correctness / recall / byte-identity, or
  a design/methodology contribution that needs no benchmark). Can pilot the factory now.
- **NEEDS-CANONICAL-BENCHMARKS** — the claim is honest and the technique real, but the
  headline rests on work-box numbers. Publishable once re-gathered on the canonical
  runner (`research/ci-ec2-design.md`) against a fair (non-Docker, native) baseline.
- **NOT-YET-SOUND / NEEDS-EXTERNAL-AUDIT** — ZK/MPC. Must be framed as honest
  design / limitations / negative result, **never** a security claim. Cite **sq-qhy4**.

### 1.4 Ranking

Rank candidates by **readiness × impact**: a PUBLISHABLE-NOW candidate with a real
audience outranks a high-impact-but-unaudited crypto claim. Impact = venue tier reachable
× breadth of the claim × strength of the differentiator vs the named prior art.

### 1.5 Re-run triggers (how the intake stays current)

This process re-runs (ideally wired into the maintenance-automation framework,
`research/maintenance-flow-on-automation-design.md`) when any of:

- **A new crate or opt-in feature lands** → screen it through 1.2.
- **A benchmark improves or a canonical run completes** → re-evaluate every
  NEEDS-CANONICAL-BENCHMARKS candidate; promote if now canonical-backed.
- **A conformance floor rises** (`scoreboard.rs` change) → strengthens any
  conformance-backed claim; re-rank.
- **An audit bead changes state** — esp. **sq-qhy4** (external ZK audit), **sq-9hrn**
  (coZK re-audit), **sq-1gir** (forge tests in CI). A sq-qhy4 pass would move the
  single-prover ZK candidate out of NOT-YET-SOUND. *Until then it cannot move.*
- **A new `research/*-measured.md` or negative-result doc** appears → a measured
  correction may itself be a (small) contribution.

The re-run is cheap: re-scan 1.1, re-score 1.2, re-assign 1.3, re-rank 1.4, diff against
this document's inventory, and open/close paper-candidate beads accordingly.

---

## Part 2 — Current inventory (ranked by readiness × impact)

Working titles, venues, and verdicts below. Venues are my judgement (no phase-1 venue-map
doc exists in-tree yet); RDF/semantic-web work targets **ISWC / ESWC**, systems work
**VLDB / SIGMOD / CIDR / EDBT**, security work **USENIX Security / CCS / PoPETs**.

### Tier A — PUBLISHABLE-NOW candidates (pilot the factory with these)

#### A1. RDF-native filtered-ANN: the filter predicate is an exact, transitively-pushed-down BGP
- **Working title:** *"Filter-as-Query: Predicate-Constrained Vector Search where the
  Filter is an Exact RDF Basic Graph Pattern over the Engine's Own Dictionary Ids."*
- **Novel claim:** Filtered-ANN systems (ACORN SIGMOD'24, NaviX'25, PathFinder) take a
  *scalar metadata predicate*. sparq's filter is **an exact BGP evaluated against the
  engine's permutation indexes** to produce an `IdMask` over the *same dict-id space* the
  vectors are keyed on — no metadata mirroring, and generalised to **transitive /
  connected-component pushdown** (the constraining set is the join-graph component
  reachable from `?node` through shared variables) with a stated **answer-safety
  ("narrow-never-widen")** argument. The connected-component pushdown is the freshest
  sub-idea.
- **Target venue:** ISWC / ESWC (research track) as a *systems/integration* paper; could
  reach EDBT. Not SIGMOD/VLDB-novel on the ANN core.
- **Evidence:** IMPLEMENTED — `crates/sparq-vectors/src/{filter.rs,rewrite.rs}`,
  selectivity-gated prefilter vs filtered-traversal crossover. **Canonical evidence:**
  deterministic *recall vs exact-filtered ground truth* (`tests/filtered.rs`). Caching
  ("+caching" in the brief) is **NOT implemented** — do not claim it.
- **HONESTY / READINESS: PUBLISHABLE-NOW** *as a systems/integration paper* with the
  recall (correctness) evidence. The ANN machinery is unmodified prior art — a reviewer
  pressing "what's new vs ACORN" must be answered with **exactness + same-id-space +
  transitive pushdown + answer-safety**, not speed. If a *latency* claim is wanted, it
  becomes NEEDS-CANONICAL-BENCHMARKS. Honestly: this is the strongest novelty in the repo
  but it is an integration claim, not an algorithmic one — frame it as such.

#### A2. Same-box honest-benchmark methodology (reproducibility / methods contribution)
- **Working title:** *"Honest Same-Box Benchmarking for RDF Engines: Differential-
  Correctness-Gated, Hardware-Labelled, Negative-Results-Inclusive."*
- **Novel claim:** Not a new suite (WatDiv/SP2Bench/WDBench exist). The contribution is
  *methodological rigour*: gather-once on an idle box, all engines back-to-back, result
  **COUNTs cross-checked before any timing is trusted**, explicit non-canonical-hardware
  labelling, hardware-contextualised cross-engine tables, no hard-coded perf in git, and a
  documented negative-results ledger. (`research/competitor-benchmark-landscape.md` §5,
  `research/ci-ec2-design.md`, `research/BENCHMARKS.md`.)
- **Target venue:** a reproducibility / experiments-&-analysis track (VLDB E&A, or an
  ISWC resources/reproducibility track), or a methods note. *Not* a full research paper on
  its own.
- **Evidence:** the methodology needs no benchmark to describe. IMPLEMENTED registry seam
  (`bench/competitors.json`, gather scripts, per-commit deterministic dashboard).
- **HONESTY / READINESS: PUBLISHABLE-NOW** as a methods/reproducibility contribution. Be
  clear it is methodology, not a performance result. Its real value is that it
  *strengthens every other paper's evaluation section*. (The EC2-OIDC canonical lane is
  designed-not-executed — one IAM step.)

#### A3. GeoSPARQL conformance + opt-in spatial crate (conformance-backed, modest)
- **Working title:** *"A Conformant, Opt-In GeoSPARQL Layer for a Dictionary-Id RDF
  Engine."*
- **Novel claim:** Low. Faithful GeoSPARQL 1.0/1.1 subset (WKT/GML literals, `geof:`
  simple-features/Egenhofer/RCC8 relations via DE-9IM, R-tree index) as an opt-in crate.
  The only mild angle is the clean opt-in-crate architecture (no wasm/core coupling) and
  the cross-family conformance ratchet.
- **Target venue:** at most an ISWC resources/in-use track or a demo.
- **Evidence:** **Canonical** — OGC GeoSPARQL topology ratchet floor **119**
  (`crates/sparq-conformance/src/scoreboard.rs`), `crates/sparq-geo`.
- **HONESTY / READINESS: PUBLISHABLE-NOW** but **low-impact**; honestly *not a research
  novelty*, only a resources/in-use contribution. Listed for completeness; do not pilot
  the factory with it.

### Tier B — NEEDS-CANONICAL-BENCHMARKS (real, honest, but headline rests on work-box numbers)

#### B1. Out-of-core SPARQL compute matching QLever in single-digit-MB (the strongest *systems* thread)
- **Working title:** *"Matching a State-of-the-Art SPARQL Engine on Compute While
  Committing Single-Digit Megabytes: 6 Permutations, Lazy Counts, Inline Tagged
  ValueIds, and Bind Joins."*
- **Novel claim:** an architectural result — matching/beating QLever on compute across
  few→100M triples, in-RAM *and* out-of-core, via the 6-permutation restore +
  lazy/streaming counts + tagged inline integer ValueIds + bind join (`BENCHMARKS.md`).
  Plus per-core ingest throughput (`research/wikidata-ingestion-benchmark.md`) competitive
  with QLever/Virtuoso/RDFox on a fraction of the hardware. The component techniques are
  individually known (inline ValueIds are QLever's idea); the *combination + measured
  out-of-core memory discipline* is the claim.
- **Target venue:** VLDB / CIDR / EDBT (systems).
- **Evidence:** all numbers **M1 / Docker-on-macOS / ephemeral-EC2 — NON-CANONICAL**, and
  the docs themselves flag the Docker handicap. Correctness is canonically gated
  (differential fuzz vs Oxigraph).
- **HONESTY / READINESS: NEEDS-CANONICAL-BENCHMARKS.** Must be re-gathered on the canonical
  runner against **native (non-Docker) QLever** before any speed/memory headline. Today,
  only the *architecture description* + *correctness* are publishable; the headline is not.

#### B2. Served characteristic-set source statistics for federation (`scs:` VoID extension)
- **Working title:** *"Be a Better Federation Source: Serving Mined Characteristic Sets in
  VoID for Remote Source Selection."*
- **Novel claim:** characteristic sets (Neumann-Moerkotte ICDE'11), VoID, and char-set
  source selection (CostFed, Odyssey) are prior art. The systems angle: sparq mines exact
  char-sets locally anyway and **serves them as a machine-consumable source-selection
  input** (an `scs:` vocab) — which almost no endpoint does. Asset reuse, honestly
  labelled. (`crates/sparq-introspect`, `research/feature-research-federation.md`.)
- **Target venue:** ISWC / ESWC (federation/systems).
- **Evidence:** serializer/parser is deterministic; but a *federation-benefit* claim
  (does serving char-sets actually improve a remote optimiser's selection?) needs a
  canonical federated benchmark.
- **HONESTY / READINESS: NEEDS-CANONICAL-BENCHMARKS** for the benefit claim; the *mechanism*
  is PUBLISHABLE-NOW as a short systems note. Modest impact; a credible secondary paper.

#### B3. ODRL→ACP/WAC conditional-grant bridge (the *implemented half* of a bigger thesis)
- **Working title:** *"Compiling ODRL Usage Policies into a Queryable Access-Control View
  for Solid/SPARQL."*
- **Novel claim:** the genuinely-novel thesis — **ODRL Duty → ZK proof-obligation** and
  **ODRL draws the MPC disclosed-vs-hidden boundary** — is *designed-only and blocked on
  the unsound ZK estate*. The *implemented* single-node bridge (ODRL → `<urn:sparq:auth>`
  view, deny-overrides, `auth:ConditionalGrant` for stateful constraints) is largely a
  Rust re-instantiation of OAC / Pandit-class mappings.
  (`crates/sparq-policy`, `crates/sparq-solid/src/odrl_bridge.rs`.)
- **Target venue:** ISWC / ESWC (policy track), or PoPETs once the ZK half is real.
- **Evidence:** correctness is test-gated; the novel federated/ZK composition is unbuilt.
- **HONESTY / READINESS:** implemented half is **PUBLISHABLE-NOW but not novel enough
  alone**; the novel half is **NOT-YET-SOUND** (ZK-blocked). Best as a *vision/design*
  paper that honestly ships only the single-node bridge and frames the ZK-disclosure
  composition as future work (cite sq-qhy4).

### Tier C — NOT-YET-SOUND / NEEDS-EXTERNAL-AUDIT (ZK/MPC — honest design / negative results ONLY)

> Every candidate in this tier **must cite sq-qhy4** and be framed as design / limitations /
> negative result. **No security, privacy, integrity, or attestation property is proven.**

#### C1. The composition thesis: malicious-secure correctness + attested inputs + full SPARQL + global-IRI federation
- **Working title (honest):** *"Toward Verifiable Federated SPARQL: A Design and a
  Feasibility Envelope for Composing ZK Query-Proofs with Multi-Party Computation."*
- **Novel claim:** the *composition* is the contribution — no published system composes
  malicious-secure correctness + attested inputs + full SPARQL + **global-IRI** federation
  (the disqualifying feature vs node-local-id graph-MPC: GOOSE/SMPG/GORAM). Global IRIs as
  public join keys make disclosed-key federated join crypto-free.
- **Target venue (honest framing):** a design/vision or SoK at PoPETs / a workshop; *not*
  a security-proof venue.
- **Evidence:** RQ1 single-prover ZK substantially built + internally re-audited; RQ2 MPC
  primitives built (degree-reduction, secure compare, bounded paths, disclosed-key join,
  3-axis security model); the attestation join is the honest `NotYetImplemented` stub.
- **HONESTY / READINESS: NOT-YET-SOUND / NEEDS-EXTERNAL-AUDIT.** Publishable *today only*
  as an honest design + feasibility-envelope + negative-results contribution. The
  "verifiable" / "secure" framing is forbidden until sq-qhy4 closes.

#### C2. In-circuit hidden cross-credential join (single-prover ZK)
- **Novel claim:** prove two scan sub-proofs share a value at chosen slots **without
  disclosing the joined term**, bound to issuer-attested commitments; a hiding commitment
  defeats dictionary attacks. No prior graph-MPC hides a cross-credential join key in
  single-prover ZK. (`zk/compose/compose_core/src/join.nr`, `verifier.rs` `bind_joins`.)
- **HONESTY / READINESS: NOT-YET-SOUND / NEEDS-EXTERNAL-AUDIT.** IMPLEMENTED, rides the
  internally-re-audited single-prover binding discipline — but **no external audit**
  (sq-qhy4), and forge tests are toolchain-gated out of CI (sq-1gir). Gate counts
  unmeasured by policy. Honest framing: a *construction + design* contribution, not a
  proven-secure one.

#### C3. coZK soundness re-audit as a witness-validation negative result
- **Novel claim:** an adversarial re-audit of the collaborative-proof path against eprint
  **2025/1026** that yields **R-WV** — a *witness-validation-before-proving test
  obligation* converting the paper's precondition into an enforceable build-time gate.
  (`research/mpc-cozk-reaudit.md`, bead sq-9hrn.)
- **HONESTY / READINESS: NOT-YET-SOUND / NEEDS-EXTERNAL-AUDIT** — and a **genuine negative
  result**: the collaborative path is unbuilt (`proof.rs` all `NotYetImplemented`); the
  re-audit *cannot certify soundness*. This is publishable as an honest negative-result /
  methodology finding (a security-engineering lessons contribution), which is itself a
  legitimate (small) contribution.

#### C4. MPC capability matrix + bounded-property-path negative result + 3-axis security model
- **Novel claim:** the *honest per-operator feasibility envelope* for MPC over SPARQL: a
  3-axis (adversary × output-guarantee × threshold) configurable model with a fail-closed
  registry that *refuses* (never silently downgrades) infeasible requests; bounded
  fixed-`k` property paths feasible with a precise leakage statement, **unbounded paths
  proven OUT-OF-REACH** (data-dependent iteration leaks diameter). The negative results
  and the capability accounting are the contribution.
  (`research/mpc-sparql-capability-matrix.md`, `mpc-bounded-property-path-design.md`,
  `mpc-security-models-and-benchmarks.md`.)
- **HONESTY / READINESS: NOT-YET-SOUND / NEEDS-EXTERNAL-AUDIT.** The malicious-secure layer
  is a stub (every Shamir op reports `SemiHonest`); degree-reduction has no consistency
  check. Publishable today only as an honest *capability map + negative results + design*,
  explicitly semi-honest, citing sq-qhy4.

### Tier D — NOT actually novel / NOT publishable as contributions (stated plainly)

Listing these so the factory does not waste a paper slot on them:

- **`vec:` magic predicate** — every commercial store exposes vectors in SPARQL; pure
  integration. (Useful only as the seam that makes A1 expressible.)
- **Cost-based source selection (HiBISCuS + CostFed) and ANAPSID-style streaming join** —
  faithful, well-tested *re-implementations* of clearly-cited prior art. Citable as "we
  implement X," not as novelty.
- **Bidirectional / meet-in-the-middle predicate transfer** — the technique is prior art
  (predicate transfer CIDR'24, exact-bitmap semi-join "Not Yannakakis" CIDR'26). Only the
  *measured density-conditional negative finding* (smaller payoff for a compact-intermediate
  WCOJ engine) is a minor data point — and it is designed-only, not shipped.
- **Spillable external term dictionary with byte-identity** — excellent engineering with
  strong (canonical) byte-identity correctness evidence, but external-memory dict build is
  known; not an algorithmic novelty.
- **Dict-id-keyed vector colocation, per-section PQ, structural-sketch vectors, KGE-for-
  cardinality (GNCE hook)** — colocation is a natural engineering choice (HDT/QLever
  colocate similarly); the `[novel]`-tagged extensions are self-hedged and mostly
  designed-only.
- **Bit-level encoding, dict compression (FSST/PFC), data-structure surveys** — these are
  *honest measured rejections / triage records* (Roaring crossover, BSI slower, FSST a
  ~20% browser-only lever). Valuable as honesty artefacts; no novel technique.
- **Inference (RDFS/OWL-RL/N3) and property paths** — competent SOTA re-implementations
  (RDFox/VLog/EYE cited); W3C-conformant (canonical floors 1967 / 33-of-33). The
  "win-both-regimes hybrid" inference idea is an unrealised aspiration. No novelty claim.
- **SHACL** — **CORRECTION: SHACL-AF (Advanced Features — `sh:rule`/`sh:TripleRule`/
  `sh:SPARQLRule`/`sh:expression`/`sh:SPARQLTarget`) is NOT implemented** (grep-confirmed
  in `crates/sparq-shacl/src`). What ships is SHACL **Core** (98/98 W3C, canonical) +
  SHACL-SPARQL §5.2 (floor 5) + §6 custom components. Do **not** claim SHACL-AF. The
  conformant Core+SPARQL engine is table-stakes parity with Jena/pySHACL, not novel.
- **Conformance ratchet methodology** — excellent discipline (deterministic floors:
  SPARQL 1229, inference 1967, SHACL 98, SHACL-SPARQL 5, GeoSPARQL 119); belongs in an
  artefact/reproducibility appendix, not a standalone paper.

---

## Part 3 — Recommendations (what to pilot, what must wait)

**Pilot the factory with A1 + A2 together.** A1 (RDF-native filtered-ANN with exact
transitive-BGP filter + answer-safety) is the strongest genuine novelty and has *canonical
correctness/recall evidence today*; A2 (the honest benchmarking methodology) is a
low-risk methods contribution that *strengthens A1's (and every paper's) evaluation
section*. Both are PUBLISHABLE-NOW and need no canonical runner and no external audit.

**Must wait for the canonical runner (NEEDS-CANONICAL-BENCHMARKS):** B1 (the QLever-parity
out-of-core systems result — the highest-impact systems claim, but every number is
work-box/Docker today) and the benefit claims of B2 (served char-sets).

**Must wait for external audit (NOT-YET-SOUND, cite sq-qhy4):** all of Tier C — C1
(composition thesis), C2 (hidden join), C3 (coZK negative result), C4 (MPC capability
map). These can be written *today* only as honest design / limitations / negative-result
contributions; they may **never** assert a proven security/privacy/integrity/attestation
property until sq-qhy4 (and, for the multi-prover path, sq-9hrn) close. C3 (the coZK
witness-validation negative result) is the most immediately writable Tier-C item *because*
it is honestly a negative result, not a security claim.

**Do not spend a paper slot on Tier D.** Several plausible-looking "contributions" are
faithful re-implementations of cited prior art or honest measured rejections — say so.

---

> 🤖 SPARQ agent — epic sq-gum8 phase 2. Non-sycophantic by mandate.
