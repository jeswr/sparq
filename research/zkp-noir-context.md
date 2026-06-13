# Jesse Wright's Noir + SPARQL/RDF zero-knowledge work — reconstructed context

Reconnaissance report assembled 2026-06-12 from Claude persistent memory, the
`zkp-sparql-workspace` master repo, local Noir repos, and local skill/plugin
definitions. For a follow-up research agent. All paths are on this machine.

---

## (a) Chronology

The trajectory is documented by Jesse's own research note
(`~/Documents/GitHub/jeswr/zkp-sparql-workspace/notes/research/03-jesse-prior-work.md`)
and confirmed by git first/last-commit dates in each repo:

| When | Milestone | Evidence |
|---|---|---|
| Feb 2025 | FOSDEM 2025 talk "Are current standards enough? Towards Verifiable Credentials with expressive zero knowledge query" — first public statement of the SPARQL-over-ZK-VC vision | `notes/research/03-jesse-prior-work.md` |
| Apr 2025 | Oxford DPhil Transfer of Status: RQ1 = ZKP of correct SPARQL evaluation over signed VCs; RQ2 = federated MPC; RQ3 = auth from query planning. **This work is the RQ1 anchor.** | same file |
| May–Nov 2025 | ISWC 2025 Doctoral Consortium paper (CEUR Vol-4085 paper-19): **RISC Zero zkVM baseline** — Oxigraph compiled to RISC-V, ED25519 sigs, full SPARQL 1.1 SELECT, but `SELECT * WHERE { ?s ?p ?o }` over 23 triples ≈ **7.5 minutes on M1 16GB**. Repos `jeswr/risc0-sparql-poc`, `jeswr/risc0-ed25519-zk-sparql`, `jeswr/zkSPARQL-bench`. Also earlier `jeswr/circomkit-sparql` (circom; superseded). | same file; `notes/research/02-prior-art-zkp-sparql.md` §1.2 |
| Jul 2025 | `~/Documents/GitHub/jeswr/noir_sparql` — 2-commit spike: "working merkle hashing with persden hash", "WIP: use poseidon2 hash". First Noir attempt. | git log |
| Jul–Aug 2025 | `~/Documents/GitHub/jeswr/noir_sparql_proof` (47 commits) — first end-to-end Noir pipeline (sign → prove → verify). Recorded proof time **6.88 s** (README, measured 2025-07-15). | repo README |
| Aug–Nov 2025 | `~/Documents/GitHub/jeswr/noir_sparql_proof_rust` (5 commits) — variant adding a Rust-generated circuit + in-process `noir_js` proving and a recursive-aggregation attempt (`noir_segment`, vk/proof field constants). | repo README |
| Aug 2025 – Jan 2026+ | `~/Documents/GitHub/jeswr/sparql_noir` (128 commits as of local clone, 2026-01-01; development continued on GitHub `jeswr/sparql_noir` through May 2026, PR #72 etc.) — the **current monolithic system**, npm `@jeswr/sparql-noir`. | git log; workspace state |
| Dec 2025 – Jan 2026 | Supporting libraries spun out: `noir_IEEE754` (f32/f64 floats for xsd:double FILTER arithmetic, 240 commits through 2026-05-19) and `noir_XPath` (XPath 2.0 functions/operators required by SPARQL 1.1, 66 commits). Older `noir_json_parser` (2024–2025) also exists. | git logs; repo READMEs |
| May 2026 | `~/Documents/GitHub/jeswr/zkp-sparql-workspace` created as **master workspace** for an ISWC 2026 research-track paper: Noir circuits + Lean 4 proofs + LaTeX. Wave-based multi-agent workflow; `sparql_noir_modular` spawned 2026-05-04 as a second architecture. | `project_zkp_sparql_workspace.md` memory; `decisions/sparql-noir-modular-alternative.md` |
| 2026-05-13 | Last workspace state update: Wave 17/17.5 closed (sparql_noir#72 readable algebra merged; sparql_noir_modular v0.3 merged), Wave 18 in flight (G5 soundness close, GraphAssertable trait, dependabot drainage). | `notes/state-current.md` |

Paper authorship decided 2026-05-04: Wright + Shadbolt + Jun Zhao + Rui Zhao +
Christoph Braun (`decisions/iswc-paper-authorship.md`).

## (b) Architectures attempted / considered

### 1. zkVM baseline (RISC Zero) — pre-Noir, kept as the comparison baseline
Whole SPARQL engine (Oxigraph) proven in a zkVM. Full SPARQL 1.1 coverage but
~7.5 min for a trivial query. The paper "must beat the zkVM baseline by 1–2
orders of magnitude" (`notes/research/05-noir-and-zk-stack.md` §1.1).

### 2. Early Noir pipelines (`noir_sparql_proof`, `noir_sparql_proof_rust`)
- Canonicalise RDF with RDFC10 (URDNA2015-family).
- Encode each term: `termToString()` → Blake2s hash → field element; combine
  term-type code (0=NamedNode, 1=BlankNode, 2=Literal…) with the hash via
  `poseidon2::bn254::hash_2`.
- Quad = `poseidon2::bn254::hash_4` of the 4 encoded terms; quads form a
  **Poseidon2 Merkle tree**; **ECDSA secp256k1** signature over the root.
- Circuit proves: signature valid + Merkle inclusion of matched triples +
  binding consistency. TS codegen (`generateFunctional.js`) emits per-query
  Noir from templates. The `_rust` variant attempted **recursive aggregation**
  (a Noir verifier circuit over per-binding proofs; `noir_segment` package).
- Stated next steps (README): multiple roots/signatures = multiple credentials;
  blank-node scoping across credentials; multi-BGP circuits; property paths;
  operators; aggregates.

### 3. `sparql_noir` — the monolithic per-query circuit (current primary)
Path: `~/Documents/GitHub/jeswr/sparql_noir` (canonical GitHub `jeswr/sparql_noir`,
workspace checkout at `zkp-sparql-workspace/circuits/sparql_noir`).
- **API** (`README.md`): `sign(dataset)` → Merkle root + signature; `info(query)`
  → disclosed variables; `prove(query, signed)` generates + compiles a Noir
  circuit per query; `verify(proof)`. RDF/JS (`n3`) input.
- **Transform layer is Rust** (`transform/` Cargo crate, compiled to WASM in
  `transform/pkg`) lowering the SPARQL algebra (Pérez–Arenas–Gutiérrez
  formalisation) to Noir; `sparqljs` used on the TS side.
- **Specs** in `sparql_noir/spec/`: `encoding.md` (configurable hashes —
  `h_2`/`h_4` default `pedersen_hash`, `h_s` default `blake3`;
  `Enc_t(term) = h_2(type_code, value_encoding)`; type codes NamedNode=0,
  BlankNode=1, Literal=2, Variable=3, DefaultGraph=4), `algebra.md` (each
  triple pattern → `TripleInput` struct + Merkle-inclusion assertion +
  binding assertions; signature check on dataset roots), plus `config.md`,
  `disclosure.md`, `preprocessing.md`, `proofs.md`.
- **Coverage** (`SPARQL_COVERAGE.md`, 2025-12-11): Full = BGP, JOIN, UNION,
  OPTIONAL (as UNION of left and left+right), FILTER eq/compare/type-tests/
  accessors/BOUND, GRAPH, SELECT, ASK. Accepted-but-verifier-enforced =
  DISTINCT/ORDER BY/LIMIT/OFFSET/REDUCED. Preprocessed = paths `/ + *`,
  VALUES, IN/NOT IN (expansion to JOIN/bounded UNION/disjunction). Not
  supported = aggregates, GROUP BY, subqueries, MINUS, EXISTS/NOT EXISTS,
  CONSTRUCT, SERVICE.
- **Readable algebra rewrite** merged as `sparql_noir#72` (Wave 17): per-operator
  structs (IN/NotIn/IfBool/CoalesceBool/IsNumeric), 10/10 corpus rows with
  backend-gates Δ=0 vs old surface; **SPARQL 1.1 conformance 167/236**.
- A sibling checkout `circuits/sparql_noir-sentinels-wiring` holds sentinel-based
  non-membership wiring work (decision `non-membership-sentinels-transform-wiring.md`).

### 4. `sparql_noir_modular` — property-decomposed alternative (2026-05-04)
Decision doc: `zkp-sparql-workspace/decisions/sparql-noir-modular-alternative.md`.
**Deliberately parallel to, not a replacement of, the monolith** — the paper
benchmarks both as points on the {prover-time, verifier-cost,
soundness-assumption} curve.
- One tiny Noir circuit per atomic property: `filter_eq`, `filter_lt`,
  `filter_gt`, `filter_lang`, `filter_regex` (delegates to noir_XPath),
  `bgp_match` (Merkle inclusion), `bgp_nonmember_prefix3` (sentinel
  non-membership), `binding_consistency` (same `?x` value across rows).
  Each returns a `claim_hash` (Poseidon-2) as public output.
- TS dispatcher (`src/dispatch.ts`): parse → algebra walk → property graph →
  per-module witnesses → **prove in parallel** → JSON manifest
  `{ disclosed, modules, edges }`. Joins/UNION/OPTIONAL are manifest **edges**
  checked in plain JS over revealed data, not circuits.
- Verifier: per-module `verifyProof` + recompute public-input hashes from
  disclosed bindings + complete-cover check (every disclosed row must have a
  full set of property proofs). Soundness gaps tracked as G1–G5; G1–G4 closed
  in v0.2/v0.3 (manifest-level `datasetCommit`, binding_consistency grouping,
  obligation-coverage rejection, downgrade/upgrade attack rejection); G5
  (`bgp_match` value_hash ↔ `binding_consistency` binding) was Wave-18 work.
- Future-work flag: recursive aggregation meta-circuit verifying the manifest.
- Lean angle: per-module proofs are bounded and reusable; composition becomes
  a Lean theorem over `ProofManifest × Query → Bool`, no crypto involved.

### 5. Supporting Noir libraries
- `noir_IEEE754` (`~/Documents/GitHub/jeswr/noir_IEEE754`): IEEE 754 binary32/64,
  all 5 rounding modes, NaN/inf/denormals, FPgen + MPFR oracle tested. Feeds
  xsd:double/float FILTER arithmetic. Heavy gate-count optimisation discipline
  ("Tier-4" kernels) — 14 measured optimisation rules in
  `~/.claude/projects/-Users-jesght-Documents-GitHub-jeswr-noir-IEEE754/memory/`.
  New generic `Float<E,M,RM>` API was landing as of May 2026.
- `noir_XPath` (`~/Documents/GitHub/jeswr/noir_XPath`): XPath 2.0 functions and
  operators required by SPARQL 1.1 (regex, string fns, date/duration types
  with timezone ordering per F&O §10.4.6). PR #39 (Float-API migration) was
  deferred pending noir_IEEE754 main.
- `noir_json_parser` (2024–2025): JSON parsing in Noir; predates this project
  but is Jesse's; relevant to SD-JWT-style inputs.
- `circuits/lampe-literate`: tooling for `// LAMPE-LITERATE` ASCII directives
  linking Noir source to sibling `.lean` proofs (`decisions/lampe-literate-tool.md`).

### 6. Formal verification stack (architecture requirement, not optional)
`notes/research/05-noir-and-zk-stack.md` + memory `reference_noir_verification_tooling.md`:
- L0: nargo built-in underconstrained check (+ `--enable-brillig-constraints-check-lookback`).
- L1: NAVe (SMT/cvc5 ACIR verifier) — **pinned to Noir beta.9, workspace on
  beta.17, "not usable today"**.
- L2: Lampe (Noir→Lean 4 extraction, pinned `@1cd3f4de`) + proven-zk;
  `proofs/Ieee754` scaffolded with eight `sorry`-stubbed obligations.
- L3: manual SAFETY-PROOF doc-comments + adversarial `should_fail_with` tests
  flipping every clause — the de-facto floor for every `unconstrained`+`_verified`
  hint primitive.
- Jesse also has a fork of the Noir compiler itself
  (`~/Documents/GitHub/jeswr/noir-lang-noir` Claude project) doing **SSA
  range-analysis** work (soundness model: over-approximation rule;
  brillig-constraints false-positive triage) — upstream-compiler-level
  optimisation in service of the same gate-count goals.

## (c) Requirements and constraints Jesse expressed (verbatim where found)

1. **Problem statement** (`zkp-sparql-workspace/PAPER-NOTES.md`): given VCs held
   by a prover and a SPARQL query, prove (1) credentials valid, (2) query
   evaluated correctly against committed credential graphs, (3) disclosed
   result is exactly what the query yields — "without revealing any credential
   content beyond what the result entails."
2. **Priority** (Jesse 2026-05-03, `notes/inbox/priorities.md`, quoted in memory
   `feedback_sparql_noir_priority.md`): "It is important to note that the
   primary objective here is to get the SPARQL noir package working and then
   optimised; so as far as focus goes, we should probably now draw our
   attention quite heavily there." noir_IEEE754/noir_XPath are dependencies
   only.
3. **Don't ZK-prove revealed properties** (memory
   `feedback_zkp_no_proof_of_revealed_properties.md`, Jesse 2026-05-03):
   "Information that is revealed in the disclosed output must not be ZK-proven
   inside the circuit." Hence DISTINCT/ORDER BY/LIMIT/COUNT-over-disclosed are
   verifier-side; EXISTS booleans stay in-circuit (the boolean *is* the
   result). If a property is revealed but its underlying data is private,
   **redesign the disclosure** rather than add circuit machinery.
4. **Modularity for comparison study** (Jesse 2026-05-03,
   `notes/inbox/modularity.md`, quoted in
   `feedback_modular_commitment_signature_design.md`): "we should have a
   modular design so that we can switch between each type for signing. Then
   when we come to writing the paper what we can do is analyse the performance
   for each tree type, and provide a detailed assessment of the expressivity
   of each. This is similar to how we are being modular with the signature
   options." Commitment interface = `inclusion(leaf, root, witness)` +
   `non_inclusion(absent_hash, root, witness)`; tree shapes (leaf-hash sorted,
   prefix tree) and signature schemes (BBS+, SD-JWT-VC, post-quantum
   candidate) live as parallel modules. Don't proliferate variants
   speculatively — each must enable a query class others can't.
5. **Everything experimental — break APIs freely**
   (`feedback_zkp_sparql_repos_experimental.md`): no backward-compat shims in
   any workspace sub-repo unless a named external consumer exists.
6. **Formal-verification ambition**: "The decisive factor for Jesse is the
   Lampe → Lean 4 path… The paper can claim *formally verified ZK circuits for
   SPARQL evaluation*" (`notes/research/05-noir-and-zk-stack.md` §1.3).
   Backend-agnostic ACIR is the secondary factor.
7. **Performance bar**: beat the RISC Zero zkVM baseline by 1–2 orders of
   magnitude on equivalent work (same file §1.1).
8. **Workflow constraints** (memory + CLAUDE.md): roborev (codex) review on
   every commit; offload to CI (machine is loaded); async Q&A via
   `questions/<slug>.md` with `ready:` flags; autonomous-decisions-with-log;
   sub-repos pushed to private `jeswr/<name>` + `.subrepos.json`; only files
   marked `ready: true` actionable; per-decision files in `decisions/`.
9. **Hint-and-verify idiom** (skill `~/.claude/skills/noir-idioms/SKILL.md`,
   distilled from this work): compute in `unconstrained`, verify with cheap
   constraints; hint final results not intermediates; every `unsafe` block
   carries a `// Safety:` comment naming the enforced property; ACIR optimises
   gate count, Brillig optimises execution speed.

## (d) Proven to work / abandoned / open (as of 2026-05-13 state)

**Working / merged:**
- Monolith end-to-end sign→prove→verify (npm `@jeswr/sparql-noir`); 167/236
  SPARQL 1.1 conformance; readable per-operator algebra at zero gate cost.
- Modular v0.3: `compileQuery` (algebra coverage Project/Filter/Bgp/Join),
  canonical Merkle, filter_eq/lt/gt, bgp_match, binding_consistency; G1–G4
  soundness mitigations; end-to-end demo (3-triple BGP + hidden-value filter +
  disclosed-value filter) with happy + tampered tests.
- noir_IEEE754 f32/f64 with FPgen/MPFR oracle; noir_XPath regex/string/date.

**Abandoned / superseded:**
- circom (`circomkit-sparql`) and the zkVM implementation (kept only as
  baseline); `noir_sparql` spike; `noir_sparql_proof(_rust)` pipelines;
  `sparql_noir_modular#1` v0.1 closed as superseded.
- NAVe integration (toolchain drift, beta.9 vs beta.17).

**Open at last snapshot:**
- G5 binding (bgp_match value_hash ↔ binding_consistency) — Wave 18 agent.
- Non-membership sentinels (`bgp_nonmember_prefix3`) → full NOT EXISTS.
- Readable-algebra round 3: power-set OPTIONAL, MINUS, EXISTS/NOT EXISTS,
  aggregates, full REGEX/SUBSTR/CONCAT, IEEE-754 + integer FILTER arithmetic,
  subqueries (both architectures).
- noir_XPath#39 blocked on noir_IEEE754 Float<E,M,RM> API landing.
- Cross-type compare matrix in noir_XPath (`promote_to_common_type` trait).
- Lean: eight `sorry`-stubbed Ieee754 obligations; extraction blocked on an
  IEEE754 branch compile failure.
- Monolith-vs-modular benchmark comparison "not captured" yet (HANDOFF-WAVE17).

## (e) Toolchain versions and performance numbers

| Item | Value | Source |
|---|---|---|
| Noir | **1.0.0-beta.17** (`@noir-lang/noir_js`/`noir_wasm`/`noirc_abi`); rejects non-ASCII chars in comments | `sparql_noir/package.json`; workspace gotchas |
| Backend | Barretenberg **UltraHonk** via `@aztec/bb.js 3.0.0-nightly.20251104`; ACIR IR | `sparql_noir/package.json`; noir-js skill |
| Hashes | Poseidon2 (Merkle/claim hashes; "cheap in UltraHonk"), Pedersen default `h_2`/`h_4` in spec, Blake3 string→Field (Blake2s in the older pipeline) | spec/encoding.md; HANDOFF-WAVE17 |
| Signatures | ECDSA secp256k1 (early pipeline); BBS+ / SD-JWT-VC / PQ planned as modules | noir_sparql_proof README; modularity memory |
| Lampe pin | `@1cd3f4de` in `proofs/Ieee754/lakefile.toml` | research/05 |
| zkVM baseline | 23 triples, `SELECT *`: **~7.5 min** (RISC Zero, M1 16GB) | research/02 §1.2 |
| Early Noir pipeline | **6.88 s** proof generation (2025-07-15) | noir_sparql_proof README |
| Modular demo bench | per-member gate counts + prove/verify timing are committed as JSON — `bench/zk-compose/gate_counts_latest.json` / `family_cost_curve.json` (and regression-gated by `crates/sparq-zk-compose/tests/gate_count_snapshot.json`); read those rather than the historical HANDOFF-WAVE17 figures | `bench/zk-compose/*.json` |
| Monolith scale | "one large circuit, often 10^6+ gates dominated by the join machinery" | modular decision doc |
| Conformance | SPARQL 1.1 suite **167/236** (monolith, stable across #72) | state-current.md |
| Braun & Käfer (closest prior art) | sub-second proving, but **no SPARQL algebra** — term-level proofs only | research/02 §1.1 |

## (f) Confidence + gaps

**High confidence** in everything above: it comes from Jesse's own memory
files, decision logs, handoffs, and repo READMEs/specs, mutually consistent.

**Gaps / could not verify:**
1. **Post-2026-05-13 state.** `notes/state-current.md` was last updated
   2026-05-13 (Wave 18 in flight). noir_IEEE754 has local commits to
   2026-05-19. Whether Wave 18 PRs merged, whether the ISWC 2026 paper was
   submitted, and current GitHub HEADs of `jeswr/sparql_noir`/`_modular` were
   not checked (private repos; no `gh` calls made).
2. **Paper draft content** (`zkp-sparql-workspace/paper/`) not read beyond
   PAPER-NOTES and research notes; the LaTeX may contain newer architecture
   decisions.
3. **Session transcripts** were keyword-scanned (`grep -l`) but not deeply
   mined; the project dir `-Users-jesght-Documents-GitHub-jeswr/` (where the
   workspace was bootstrapped) holds transcripts in per-session subdirectories
   that were not individually extracted. Quotes above come from memory files
   that already preserve Jesse's verbatim inbox notes.
4. **Prefix-tree commitment implementation status** unclear — decided as a
   "round-4 second option" but no evidence found that it was built.
5. **BBS+/SD-JWT signature modules**: design intent is documented; no evidence
   either is implemented yet (early pipeline used ECDSA secp256k1; modular
   v0.3 anchors a `datasetCommit` but signature-scheme modules weren't seen).
6. **Exact monolith prover wall-times** per query class not found locally
   (benchmark-results/ dir exists in sparql_noir but was not enumerated).
7. The relationship between this work and the **sparq Rust engine** (this
   repo) is not stated anywhere found — they appear to be separate projects
   (sparq = performance SPARQL engine; zkp work = proofs over VC datasets).

## Source map (paths with signal)

- `~/.claude/projects/-Users-jesght-Documents-GitHub-jeswr/memory/` — MEMORY.md + `project_zkp_sparql_workspace.md`, `feedback_sparql_noir_priority.md`, `feedback_modular_commitment_signature_design.md`, `feedback_zkp_no_proof_of_revealed_properties.md`, `feedback_zkp_sparql_repos_experimental.md`, `reference_noir_verification_tooling.md` (richest source)
- `~/.claude/projects/-Users-jesght-Documents-GitHub-jeswr-noir-IEEE754/memory/` — 9 gate-optimisation feedback rules
- `~/.claude/projects/-Users-jesght-Documents-GitHub-jeswr-noir-lang-noir/memory/` — Noir compiler range-analysis fork
- `~/Documents/GitHub/jeswr/zkp-sparql-workspace/` — README, CLAUDE.md, PAPER-NOTES.md, notes/state-current.md, HANDOFF-NEXT-AGENT.md, HANDOFF-WAVE17.md, decisions/* (esp. sparql-noir-modular-alternative.md), notes/research/00–08
- `~/Documents/GitHub/jeswr/sparql_noir/` — README, spec/{encoding,algebra}.md, SPARQL_COVERAGE.md, package.json
- `~/Documents/GitHub/jeswr/{noir_sparql, noir_sparql_proof, noir_sparql_proof_rust, noir_XPath, noir_IEEE754, noir_json_parser}/` — READMEs + git logs
- `~/.claude/skills/{noir-developer, noir-idioms, noir-js, noir-testing}/SKILL.md`
- `~/.claude/plugins/cache/zkp-sparql-workspace/zkp-workspace-mgmt/0.1.0/` — workspace-mgmt plugin (confirms workspace path + notes/questions/decisions workflow)
- Transcript keyword scan: signal in jeswr-zkp-sparql-workspace (4 files), jeswr-noir-IEEE754 (2), jeswr-noir-lang-noir (2), jeswr-test-lib (1); not deeply mined (see gap 3)
