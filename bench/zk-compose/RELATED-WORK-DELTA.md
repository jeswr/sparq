<!-- [FABLE-5] sq-gum8.5 — related-work hardening note for the zkSPARQL ISWC 2026 submission.
     Citations verified 2026-07-04 (venues/DOIs/quotes traced to primary sources; see the
     per-work "source" lines). This note supports the AUTHORS' rebuttal / camera-ready
     readiness — it does NOT rewrite the paper. Every claim about the sparq estate is scoped
     "design + implementation, pre-external-audit" (sq-qhy4 open). -->

# zkSPARQL related-work delta

A reviewer-facing map of the closest prior work to the zkSPARQL submission
([zksparql.org](https://zksparql.org/), spec source `site/specs/zksparql.typ`), the honest
differentiation for each, and the exact claim wording the submission can defend. Compiled for
sq-gum8.5. The spec's `= Related work` section was hardened in the same change; this note is
the longer working record (evidence, quotes, and the claim-wording recommendations a rebuttal
would draw on).

All statements about the sparq/zkSPARQL estate are scoped **design + implementation,
pre-external-audit** — the ZK verifier and Noir circuits have NOT completed external
accredited-cryptographer review (bead sq-qhy4; forge/soundness tests are toolchain-gated,
sq-1gir). Do not read any line here as a proven security, privacy, or soundness guarantee.

## The one thing to fix first: fragment-scope claim vs implementation

The live landing page states the system supports
*"a non-trivial fragment of the SPARQL 1.1 algebra (BGP, Join, Filter, OPTIONAL, UNION,
bounded property paths, EXISTS, NOT EXISTS, and MINUS)"*. The in-repo spec
(`site/specs/zksparql.typ` section 7.1) and the compiled circuit family disagree: the spec
says *"OPTIONAL, UNION, property paths, aggregation, and subqueries are not part of the
fragment"*, and the compiled family (`zk/compose/`, enumerated in `eval_pack.json`) is
`Scan` / `Filter{Int,F64,SignedInt,Decimal,ValueDl}` / `JoinEq` plus the credential-layer
circuits (`RevokeUnset`, `HiddenIssuer`, `HolderPok`, `HolderSet`) — **no** OPTIONAL, UNION,
property-path, EXISTS, or MINUS circuit exists in this repository today.

This is the single most likely honesty challenge. Either (a) the paper's evaluation covers a
broader fragment realised outside this repo's `zk/compose/` tree, in which case the mapping
from paper claim to committed artifact needs to be stated explicitly; or (b) the broader-
fragment wording is aspirational and should be scoped to what is closed end-to-end. **Do not
leave the two documents contradicting each other** — a reviewer who reads both will treat the
gap as an overclaim. Recommended framing: state precisely which operators are
proof-carrying-today vs. specified-but-not-yet-circuit, exactly as the spec already does.

## Per-work delta

### ZKLP — Ernstberger et al., IEEE S&P 2025

- **Source.** *Zero-Knowledge Location Privacy via Accurate Floating-Point SNARKs*, IEEE S&P
  2025 pp. 3440–3459; arXiv:2404.14983; IACR ePrint 2024/1842; dblp `conf/sp/ErnstbergerZCJS25`.
- **What it does.** A library of IEEE-754-compliant f32/f64 SNARK circuits (init/add/sub/
  mul/div/sqrt/compare, incl. subnormals, NaN, ±∞, round-half-to-even) in gnark over R1CS
  (BN254; Groth16/Plonk) with LogUp lookups, validated against Berkeley TestFloat, applied to
  hexagonal-grid location predicates.
- **Why it matters.** It claims *"the first set of Zero-Knowledge Proof (ZKP) circuits that
  are fully compliant to the IEEE 754 standard"* (abstract). This is peer-reviewed and prior.
- **Differentiation.** zkSPARQL's float work is the *integration* of in-circuit IEEE-754 into
  typed SPARQL value-FILTER evaluation over committed RDF (the `xsd:double` FILTER lane),
  not the float primitives in isolation.
- **Defensible wording.** Fine: "in-circuit IEEE-754 typed value filters." **Forbidden:** any
  "first ZK floats / first IEEE-754 in ZK" phrasing — ZKLP owns it. The landing page's
  "worked numeric primitive (IEEE 754 add\_f32) closed end-to-end" is acceptable (it does not
  claim primacy).
- **Numbers.** ZKLP's per-op constraint counts are **R1CS + LogUp**, a different unit from
  this estate's UltraHonk `circuit_size`. They are recorded in `eval_pack.json`
  (`cited_external.zklp`, `cited_not_remeasured: true`) but **must not** be tabulated
  side-by-side with the sparq gate counts as if comparable.

### ZKGraph — arXiv:2507.00427, 2025 (preprint)

- **Source.** *Zero-Knowledge Verifiable Graph Query Evaluation via Expansion-Centric Operator
  Decomposition*, arXiv:2507.00427 (Jul 2025); code `github.com/Hao8172/ZKGraph`. No
  peer-reviewed venue found — treat as preprint.
- **What it does.** Decomposes graph queries into an "expansion" (node→neighbour) primitive +
  attribute circuits, verified in Halo2 (PLONKish, KZG), so an owner proves query-execution
  correctness without disclosing the graph. Evaluated on LDBC SNB; queries expressed in Cypher
  over **property graphs**.
- **Differentiation.** Property-graph / Cypher, **not RDF/SPARQL**. No blank-node or per-graph
  canonicalisation model; no issuer-attestation / multi-credential composition.
- **Watch-out.** ZKGraph self-positions as *"the first system to leverage non-interactive
  zero-knowledge proofs for the confidential and verifiable evaluation of arbitrary graph
  queries."* A reviewer may juxtapose it with any zkSPARQL "first" wording. **Cite it**, and
  scope zkSPARQL's novelty to RDF/SPARQL semantics + signed-credential composition rather than
  to "first ZK graph queries."

### PoneglyphDB — Gu, Fang, Nawab, SIGMOD 2025

- **Source.** *PoneglyphDB: Efficient Non-interactive Zero-Knowledge Proofs for Arbitrary
  SQL-Query Verification*, Proc. ACM Manag. Data 3 (SIGMOD 2025), DOI 10.1145/3709713;
  arXiv:2411.15031.
- **What it does.** Non-interactive ZK (Halo2/PLONKish, recursive composition) circuits for
  SQL operators — range checks/filters, sorting, group-by, joins, aggregation — composed into
  whole-query proofs; host keeps raw data. Evaluated on TPC-H.
- **Differentiation.** Relational (SQL) not RDF/SPARQL; single committed database, not issuer-
  signed multi-graph credentials. The paper does **not** address NULLs / three-valued logic /
  outer joins (verified: no mention in the full text), so SPARQL OPTIONAL-style semantics are
  genuinely out of its scope — a real point of distinction if zkSPARQL closes them.
- **Defensible wording.** "the first *non-interactive* ZK proofs of SQL query results" belongs
  to PoneglyphDB; zkSPARQL should not claim non-interactivity as novel per se.

### ZKSQL — Li et al., PVLDB 16(8), 2023

- **Source.** *ZKSQL: Verifiable and Efficient Query Evaluation with Zero-Knowledge Proofs*,
  PVLDB 16(8) 1804–1816, 2023; DOI 10.14778/3594512.3594513.
- **What it does.** Interactive (VOLE-based) ZK over SQL evaluation steps incl. ZK joins;
  authenticated set operations; TPC-H. The interactive SQL predecessor to PoneglyphDB.
- **Differentiation.** Interactive protocol vs zkSPARQL's non-interactive offline-checkable
  manifest; SQL not SPARQL. Already cited in the spec.

### VeriDKG — Zhou et al., PVLDB 17(4), 2023

- **Source.** *VeriDKG: A Verifiable SPARQL Query Engine for Decentralized Knowledge Graphs*,
  PVLDB 17(4) 912–925, 2023; DOI 10.14778/3636218.3636242.
- **What it does.** An authenticated data structure (RGB-Trie + accumulator, blockchain-
  maintained) so clients verify SPARQL results are correct/complete/fresh. It is the *closest
  system on the SPARQL axis* a reviewer will name.
- **Differentiation — the important one.** VeriDKG provides **integrity and completeness, not
  zero knowledge**: its abstract claims *"both data integrity and query verifiability"* and
  contains no confidentiality/privacy claim; the queried data is **not** hidden. It is
  therefore *complementary*, not a competitor — cite it precisely as an integrity system so a
  reviewer cannot say "SPARQL verifiability is already solved."

### zk-creds — Rosenberg et al., IEEE S&P 2023; Crescent — ePrint 2024/2013

- **Sources.** zk-creds: IEEE S&P 2023 pp. 1882–1900, ePrint 2022/878. Crescent: IACR ePrint
  2024/2013 (2024, **preprint** — no confirmed peer-reviewed venue; cite as ePrint, not CCS).
- **What they do.** Prove possession of *existing, unmodified* credentials (e-passports;
  JWT / ISO mDL) in zero knowledge, adding selective disclosure / unlinkability without issuer
  cooperation.
- **Differentiation.** They prove *possession / attribute disclosure of one credential*.
  zkSPARQL generalises the statement language to *SPARQL query evaluation over the signed
  data* (joins, typed filters, revocation), spanning multiple credentials. Cite as the
  credential-layer state of the art the query layer sits above.

### Braun, Wright & Käfer — ESWC 2026 (the authors' own line)

- **Source.** *Proving Soundness of SPARQL Query Results Using Selective Disclosure of RDF
  Datasets and Zero-Knowledge Proofs*, The Semantic Web — ESWC 2026, LNCS 16549,
  DOI 10.1007/978-3-032-25156-5\_16; repo `github.com/uvdsl/rdf-zkp-sparql`.
- **What it does ("zkRDF").** A **data-centric** approach: the holder guarantees *soundness of
  query results* by proving properties **about the queried RDF dataset**, exposing only the
  minimal info the proof needs. Abstract: *"Unlike existing methods that prove query
  execution, we establish soundness of query results by proving properties about the queried
  RDF dataset."* It also reports it *"outperforms an approach that proves query execution by
  three orders of magnitude."*
- **The self-delta a reviewer WILL probe.** This is the authors' own prior/companion work, and
  it explicitly claims a large speed advantage over the *prove-the-execution* strategy that
  zkSPARQL embodies. The submission must answer "why prove execution at all?" up front. The honest
  answer (now in the spec): the two occupy opposite points of a trade-off — zkRDF discloses
  selective *views* of the dataset and is cheaper; zkSPARQL keeps the source graphs hidden and
  proves the algebra operators in-circuit, covering operator statements (in-circuit joins,
  typed filters) that a disclosure-of-views argument does not directly reach. Complementary,
  per-query-choosable, **not** competing. Do not cite the 3-orders figure without this framing.

## Name-collision heads-up (not related work, but worth knowing)

Dan Yamamoto (IIJ) et al. presented a *"zk-SPARQL"* (verifiable, privacy-preserving SPARQL
over VCs) at IIW 35 (Nov 2022) and in slide decks — **no citable peer-reviewed paper found**
under that exact name, but the naming near-collision with "zkSPARQL" is worth a reviewer-
facing footnote. A related published chapter exists (*RDF-Based Semantics for Selective
Disclosure and Zero-Knowledge Proofs on Verifiable Credentials*, Springer 2025,
DOI 10.1007/978-3-031-94575-5\_21) — verify authorship before citing.

## Fresh-search residue (2025–2026)

- **Newer IEEE-754 ZK float circuits than ZKLP: none found.** Predecessor is Garg et al.,
  *Succinct Zero Knowledge for Floating Point Computations*, CCS 2022 (add/mul only, no
  concrete implementation). ZKLP remains the reference.
- Adjacent verifiable-DB preprints a thorough reviewer *might* raise but which are off-axis:
  V3DB (ZK verifiable vector search, arXiv:2603.03065, 2026, preprint — UNVERIFIED beyond
  title/abstract); a minor ICBTA 2024 "privacy-enabled databases" workshop paper. Neither is
  RDF/SPARQL query-evaluation ZK; mention only if a reviewer surfaces them.
- SparqLog (PVLDB 16(13), 2023) is a Datalog-based SPARQL 1.1 *engine* — **no ZK / verifiable
  / privacy content**. It is not related work for this submission; do not cite it as such.

## Recommended claim-wording changes for the authors

1. **Reconcile the fragment claim** (top of this note): make the paper's stated fragment match
   what is proof-carrying in the artifact, or explicitly separate proof-carrying-today from
   specified-but-not-yet-circuit operators.
2. **Never claim "first ZK floats / first IEEE-754 in ZK"** — attribute to ZKLP; frame the
   float contribution as *integration into typed SPARQL FILTER evaluation*.
3. **Scope any "first ZK graph/SPARQL query" novelty** to *RDF/SPARQL semantics over
   issuer-signed multi-credential data*, and cite ZKGraph (graph/Cypher) + VeriDKG
   (SPARQL-integrity) so the novelty is stated as a delta, not a vacuum.
4. **Cite VeriDKG as integrity-not-ZK** so "SPARQL verifiability already exists" is answered.
5. **Pre-empt the Braun–Wright–Käfer 3-orders comparison** with the complementary-trade-off
   framing rather than ignoring it.
6. Keep every soundness/privacy statement scoped pre-external-audit (sq-qhy4) — the pack's
   evidence is constraint *sizes* (a size is not a soundness claim), not a security result.
