<!-- [OPUS-4.8] Research/design record — lens: "Reasoning, AI & maintainer-aligned frontiers".
     DESIGN-FOR-REVIEW. Investigation only; proposes no crates/ changes. NON-CANONICAL timing
     (no measured numbers). ZK/MPC are NOT externally audited — every privacy claim here is
     caveated. Re-review when Fable returns. -->

# Reasoning, AI & maintainer-aligned frontiers — gap research

> Model: Opus 4.8 (1M context) — Fable unavailable; flag for re-review when Fable returns.
> SPARQ agent 🤖 design-for-review record. **Investigation/design only — no `crates/` changes.**
> Every candidate below is checked against the *actual* code/tests/`Cargo.toml`, not the brief.
> The standing **lean-core** rule holds: each candidate must be an opt-in crate / cargo-feature.

## How to read this

Eight candidate gaps, two strands: **(A) reasoning/semantics** and **(B) the maintainer's
privacy/decentralized estate**. Each has an honest novelty check (what already exists), prior art
with refs, an opt-in story, an effort estimate, and a single decision-question. Effort is
S/M/L/XL relative to existing crates. No performance numbers are asserted — where a system claims
a figure, it is attributed to that system, not to sparq.

A correction to the brief's premise is recorded in §0; it is load-bearing for two candidates.

---

## 0. What already exists in this lens (so proposals are genuine gaps)

Verified by reading the crates, skills, and `research/` records — **not** taken on faith:

- **Reasoning (`sparq-reason`, opt-in):** RDFS (rdfs2/3/5/7/9/11) and a *substantial* **OWL 2 RL**
  forward-chaining materializer (equality via union-find, inverse/symmetric/transitive/functional/
  IFP, equivalent class/property, class-expression rules `cls-*` for someValuesFrom/allValuesFrom/
  hasValue/oneOf/intersection/union, propertyChainAxiom prp-spo2, cardinality/hasKey, the `scm-*`
  schema family), **OWL inconsistency checking** (`inconsistencies()`), full **N3/EYE** rules with
  `log:`/`list:`/`math:` builtins incl. **scoped negation** (`log:notIncludes`), **incremental**
  closure maintenance (counting-based insert/delete) for all three regimes, and **proof-tree
  explanation** (`why()` → `ProofTree`, `to_json`/`to_text`) behind the `explain` feature. PROV-O
  lineage of reasoner derivations exists in `sparq-prov` (`prov_from_proof`).
- **Graph analytics (`sparq-algos`, opt-in):** PageRank, degree centrality, weakly-connected
  components, label-propagation communities — topology view (`NodeGraph`), deterministic, no model.
- **GenAI/vector (`sparq-vectors`, `sparq-nlq`, skills `vector-search`/`genai-retrieval`):** vector
  index + embeddings, RAG retrieval, NL→SPARQL, ontology introspection (`sparq-introspect`).
- **Privacy/verifiable (`sparq-zk`, `sparq-zk-compose`, `sparq-mpc`, `sparq-fedplan-mpc`):** a
  custom **ZK commitment+proof estate** (Poseidon2-BN254 commitments, RDFC-1.0 canon, Schnorr/
  Baby-JubJub issuer signatures, a zk-trace seam, FILTER/threshold/hidden-join circuits) targeting
  **soundness (and within-merge completeness) of SPARQL query results** — *NOT externally audited;
  the v1 verifier is internally re-audited with external accredited-cryptographer sign-off PENDING
  (sq-qhy4); MPC is semi-honest only*. `sparq-canon` provides RDFC-1.0 (the basis for any signing).
- **Solid (`sparq-solid`, opt-in):** WAC + ACP materialized via N3 rules (incl. `acp:issuer`,
  CreatorAgent/OwnerAgent), with a §2.4 reasoner/content security boundary.
- **At-rest:** `vacuum()` erasure (WAL-vacuum) lands on-box; **crypto-erase is design-only**
  (`research/crypto-erase-at-rest.md`, bead sq-du24).
- **Extension functions:** `sparq-engine` already has a **custom extension-function registry**
  (concrete term in → one term out), used by SHACL-AF too.

### Correction to the brief's premise

The brief lists "graph analytics (PageRank/centrality/community — verify if exists)" and "SPARQL+ML
UDFs" as candidate gaps. **Verified: graph analytics already exists** (`sparq-algos`) and a
**SPARQL custom-function registry already exists** (`sparq-engine`, `Engine::register_function`).
So neither is a gap *as stated*. The genuine adjacent gaps are: (i) analytics **callable from
within SPARQL** and **path/GDS-style** algorithms beyond topology PageRank/WCC (candidate R3), and
(ii) **model-backed (ML) UDFs that bridge the existing registry to the vector/GenAI estate**
(folded into R3's decision, not proposed as a standalone "build a UDF mechanism" — that mechanism
exists). This correction is why R3 is scoped narrowly rather than as "add UDFs."

---

# Strand A — Reasoning & semantics

## R1. OWL 2 EL / QL profile reasoning (classification + query rewriting)

**What.** Two opt-in additions to `sparq-reason`: **(a)** an **OWL 2 EL** *classifier* — a
consequence-based saturation procedure (the ELK family) that computes the full class hierarchy
(complete subsumption) for EL ontologies; and **(b)** an **OWL 2 QL** *query-rewriting* reasoner
(PerfectRef / combined approach) that rewrites a SPARQL query against a QL TBox so that ordinary
BGP evaluation over the unmodified ABox returns the certain answers — no materialization.

**Why (concrete).** OWL 2 RL (what sparq has) is **sound but incomplete** for EL and QL: it is a
forward-chainable Horn subset and *cannot* derive all entailments of EL existentials or QL
inverse/role-hierarchy reasoning. The flagship EL ontologies — **SNOMED CT** (~350k classes),
**Gene Ontology**, **ChEBI** — are EL precisely so they can be *classified completely* in seconds;
RL materialization gives wrong (incomplete) class hierarchies on them. QL is the **OBDA** profile:
its whole point is query rewriting over large, unmaterialized stores. A biomedical/Wikidata user
who runs sparq's RL reasoner on a SNOMED-derived ontology gets a silently incomplete subsumption
lattice today.

**Fit.** Strengthens the reasoning estate where it is provably weakest, and EL classification is a
natural input to the GenAI/introspection surface (a complete, materialized hierarchy improves
NL→SPARQL grounding). QL rewriting composes with federation (rewrite once, push BGPs to sources).

**Novelty vs existing.** Genuine gap. Confirmed by reading `sparq-reason/src/`: `Profile` is
`{Rdfs, OwlRl}` only; there is no classification/subsumption-lattice output and no query-rewriting
path. `research/inference-sota.md` already surveys ELK/consequence-based EL as *external* SOTA but
sparq ships none of it. Partial overlap: the `scm-*` rules give *some* TBox subsumption, but not the
complete EL procedure.

**Prior art.** ELK (consequence-based EL classifier, Kazakov/Krötzsch/Simančík) —
<https://www.uni-ulm.de/fileadmin/website_uni_ulm/iui.inst.090/Publikationen/2012/KazKroSim12ELK_TR.pdf>,
<https://github.com/liveontologies/elk-reasoner>; Snorocket (EL, used in SNOMED tooling). OWL 2 QL
rewriting: PerfectRef (Calvanese et al., *DL-Lite*), the **combined approach** (Lutz/Toman/Wolter),
**Ontop** OBDA system <https://ontop-vkg.org/>. No Rust EL/QL reasoner exists in the surveyed
landscape — this would be novel as a Rust crate.

**Opt-in?** Yes — extend `sparq-reason` with `owl-el` / `owl-ql` cargo-features (or a sibling
`sparq-reason-el` crate). Zero default-build cost.

**Effort.** **L** (EL classifier is a well-specified saturation calculus; QL rewriting is a
separate, smaller module but needs care to bound rewriting blow-up). Two phases, EL first.

**Decision-question.** Do you want *complete* reasoning for the EL/QL profiles (the SNOMED/OBDA
use-cases), or is RL's sound-but-incomplete materialization the deliberate ceiling for sparq's
reasoner — with EL/QL left to external tools? If yes, EL-classification-first or QL-rewriting-first?

---

## R2. Datalog± rule engine: stratified negation, aggregation, and the chase (existential rules)

**What.** An opt-in **Datalog±** materializer — recursive rules with **stratified negation**,
**stratified aggregation** (count/sum/min/max in rule heads), arithmetic/datatype builtins, and
optionally **existential rules** (tuple-generating dependencies) via the **restricted (standard)
chase**. This is the rule expressivity *between* N3 and a full production rule engine, with a
declarative, terminating, well-founded semantics.

**Why (concrete).** sparq's N3 path has `log:`-scoped negation and `list:`/`math:` builtins, but it
is *EYE-rule shaped*, not a **stratified datalog with aggregation in rule heads**. Real KG
pipelines want rules like "a `Region` is `highRisk` if `count` of its incidents `> N`" or "infer
`x derivedTotal (sum ?v)` ", and **recursion with negation** ("a node is `safe` if reachable and
NOT reachable-from-quarantine") — exactly the class RDFox/Nemo/VLog target. These are not
expressible in OWL-RL and are awkward/absent in the current N3 builtins (which note
`log:collectAllIn` scoped aggregation as a *future* increment).

**Fit.** Completes the reasoning estate toward the materialization SOTA the maintainer's own
`inference-sota.md` benchmarks against (RDFox/Nemo/VLog). Existential rules (the chase) are the
formal backbone of **OBDA mappings** and **graph completion** — adjacent to the QL work in R1.
Reuses the existing semi-naive fixpoint machinery in `sparq-reason`.

**Novelty vs existing.** Genuine gap, with honest partial overlap: N3 gives *some* negation and
arithmetic, and OWL-RL gives recursive Horn materialization — but **no stratified aggregation in
heads, no well-founded stratification across a user rule program, and no chase / existential
heads**. `inference-sota.md` documents all of this as *external* (Nemo is the closest Rust system,
not sparq's).

**Prior art.** **Nemo** (Rust, in-memory; stratified negation + aggregates + existential rules via
restricted chase) — <https://github.com/knowsys/nemo>, <https://arxiv.org/abs/2308.15897>,
<https://ceur-ws.org/Vol-3801/short3.pdf>. **VLog/Rulewerk** (Skolem/restricted chase) —
<https://iccl.inf.tu-dresden.de/web/VLog/en>. **RDFox** (stratified negation + stratified
aggregation + incremental B/F) — <https://docs.oxfordsemantic.tech/5.6/reasoning.html>. **Soufflé**
(execution engineering: magic sets, RAM IR) — <https://souffle-lang.github.io/>. Stratified
negation / well-founded semantics — <https://www.ijcai.org/proceedings/2018/0259.pdf>.

**Opt-in?** Yes — a `datalog` feature on `sparq-reason` (or `sparq-datalog`). Chase-termination is
undecidable in general, so it ships with an **acyclicity check + a configurable depth/fact bound**
(honesty boundary, mirroring the existing "we bound it" posture).

**Effort.** **XL** (negation+aggregation stratification is tractable on top of the existing
fixpoint; the chase + termination analysis is the heavy, research-grade part). Phase the chase last
or behind its own sub-feature.

**Decision-question.** Is "full datalog± / chase parity with Nemo/RDFox" in scope for sparq, or
should the reasoner stop at OWL profiles (R1) + N3, treating heavyweight rule programs as a
non-goal? If in scope, do you want **stratified-negation+aggregation first** (high value, bounded)
and **existential chase** deferred to its own bead?

---

## R3. SPARQL-callable analytics + model-backed UDFs (bridge the existing registry to GDS-style ops)

**What.** Two thin, opt-in bridges — **not** new mechanisms: **(a)** surface `sparq-algos` results
**as a SPARQL extension** (a `sparq:pagerank(...)` / property-function or a procedure-call form that
binds scores to variables), plus add the **path/GDS algorithms `sparq-algos` lacks** (shortest
path, betweenness/closeness centrality, weighted edges, Louvain/Leiden modularity); **(b)** allow
**model-backed UDFs** — register an extension function whose closure calls an embedding model or a
classifier — using the **already-existing** `Engine::register_function` registry, wired to
`sparq-vectors`.

**Why (concrete).** Today `sparq-algos` is Rust-API-only ("There is no SPARQL-level integration —
call the Rust API directly", per its SKILL). Users who think in SPARQL cannot rank/cluster inside a
query, cannot do `?a sparq:shortestPath ?b`, and cannot say `FILTER(sparq:similarity(?x, "...") >
0.8)` over an embedding model. This is the single most-requested shape in graph DBs (Neo4j GDS,
TigerGraph) and is the natural glue between the analytics, vector, and SPARQL surfaces sparq
*already has separately*.

**Fit.** Pure integration of existing estates (algos + engine registry + vectors); no new trust or
crypto surface. Directly serves the GUI-as-app direction (in-app "rank/cluster these nodes").

**Novelty vs existing.** **Partial overlap, stated honestly** (this is the brief correction from
§0): the extension-function *registry* and the topology PageRank/centrality/WCC algorithms **both
exist**. The gaps are (i) the **SPARQL-callable surface** for them, (ii) **weighted + path + Louvain
GDS algorithms** absent from `sparq-algos`, and (iii) a **documented model-backed-UDF recipe**
binding the registry to `sparq-vectors`. None of these is "invent UDFs."

**Prior art.** Neo4j GDS (`gds.*` procedures) — <https://neo4j.com/docs/graph-data-science/>;
Apache AGE / TigerGraph algorithm libraries; SPARQL property functions (Jena `apf:`); Stardog
`stardog:` extension functions. Vec2SPARQL (embedding UDFs in SPARQL) —
<https://www.biorxiv.org/content/10.1101/463778.full.pdf>.

**Opt-in?** Yes — a `sparql-bridge` feature on `sparq-algos` and the model-UDF recipe gated behind
`sparq-vectors`. Core engine unaffected.

**Effort.** **M** ((a) is plumbing over existing pieces + a few new algorithms; (b) is mostly a
documented pattern + one example function). Phaseable: SPARQL-call surface, then new algorithms,
then the model-UDF recipe.

**Decision-question.** Should `sparq-algos` get a **SPARQL-callable** surface (property-functions or
a `CALL`-style form) and the missing **weighted/path/Louvain** algorithms, and do you want a
**blessed model-backed-UDF recipe** wiring the existing registry to `sparq-vectors` — or do you
prefer analytics stays a deliberately separate Rust-API layer?

---

# Strand B — Privacy / decentralized maintainer frontiers

## P1. W3C Data Integrity signed result-sets / signed datasets (the non-ZK, trust-the-issuer complement)

**What.** An opt-in `sparq-sign` crate that produces and verifies **W3C Verifiable Credentials 2.0
Data Integrity proofs over RDF** — `eddsa-rdfc-2022` (and optionally `bbs-2023` for selective
disclosure) — built **on the RDFC-1.0 canonicalization sparq already has** (`sparq-canon`). Two
uses: **(i) signed datasets** (sign a named graph / a loaded store so a consumer can verify
authenticity+integrity offline), and **(ii) signed SPARQL result-sets** (the server signs the
canonicalized CONSTRUCT graph or a canonical serialization of a SELECT result + the query + a
timestamp, so a downstream consumer has a portable, verifiable "this endpoint asserted this
answer").

**Why (concrete).** This is the **cheap, standards-interop, trust-the-signer** half of verifiable
query results — the 90% case where the consumer trusts the *endpoint's key* but needs tamper-
evidence and non-repudiation (audit trails, data marketplaces, journalistic provenance, agent-to-
agent data exchange). The ZK estate proves *soundness without trusting the prover* (a different,
much heavier guarantee). A consumer who just wants "prove this came from data.gov unmodified" does
not need a SNARK.

**Fit.** Squarely on the maintainer's verifiable-results / VC direction, and **interoperable**: VC
2.0 Data Integrity is the W3C interop format, which sparq's *custom* ZK commitment scheme (Poseidon/
Schnorr) is deliberately **not**. It reuses `sparq-canon` and the existing issuer-signature concepts
in `sparq-zk::sig` (without pulling in circuits).

**Novelty vs existing.** Genuine gap with careful boundaries. Confirmed: `sparq-zk::sig` does
Schnorr-over-Baby-JubJub over *Poseidon commitments* for the ZK pipeline — **not** W3C
`DataIntegrityProof` / `eddsa-rdfc-2022` / BBS over RDFC-1.0. `sparq-canon` provides the canon
substrate but no signing API. There is **no signed-result-set or signed-dataset surface** in the
workspace. This is the W3C-interop, non-ZK lane and does **not** overlap the ZK soundness work — it
should be positioned as its complement.

**Prior art.** W3C VC Data Integrity 1.0 — <https://www.w3.org/TR/vc-data-integrity/>;
`eddsa-rdfc-2022` — <https://www.w3.org/TR/vc-di-eddsa/>; `bbs-2023` (selective disclosure) —
<https://www.w3.org/TR/vc-di-bbs/>; classic authenticated-query (signature chaining / Merkle Hash
Tree) and **VeriDKG** (blockchain ADS for verifiable SPARQL) as the trust-minimized contrast —
RGB-Trie authenticated data structure.

**Privacy honesty.** `eddsa-rdfc-2022` is authenticity+integrity only — **not** confidentiality and
**not** a zero-knowledge property. BBS selective disclosure hides *unrevealed* attributes but is
**not** the ZK-soundness guarantee of the `sparq-zk` estate. State both plainly; no overclaiming.

**Opt-in?** Yes — standalone `sparq-sign` crate, depends on `sparq-canon`; zero default-build cost.

**Effort.** **M** (eddsa-rdfc-2022 over an existing RDFC-1.0 substrate is well-specified; BBS adds
real complexity and would be a later phase).

**Decision-question.** Do you want a **W3C-standards-interop signed-dataset / signed-result lane**
(`eddsa-rdfc-2022` over `sparq-canon`) as the cheap complement to the ZK estate — and is **BBS
selective disclosure** in scope, or out (since the ZK estate is your selective-disclosure story)?

---

## P2. Differential privacy for SPARQL aggregates (DP-schema + bounded-sensitivity COUNT/SUM)

**What.** An opt-in `sparq-dp` crate implementing **differentially-private aggregate answers** over
SPARQL — calibrated noise (Laplace/Gaussian) on `COUNT`/`SUM` whose **sensitivity is bounded by a
declared dp-schema** (star-shaped BGP partition defining the privacy unit), with per-query/global
**ε,δ budget** accounting.

**Why (concrete).** A pod/endpoint that wants to publish *statistics* over sensitive data (health,
location, social) without leaking individuals needs DP, not access control. This is a recognized,
hard, *unsolved-in-practice* problem for RDF because joins blow up sensitivity — and there is a
**peer-reviewed approach designed specifically for SPARQL** (Buil-Aranda, Lobo, Olmedo et al.) that
sparq is unusually well-placed to host given its Solid/privacy direction and its existing aggregate
engine.

**Fit.** Extends the privacy estate into the *statistical-release* quadrant the ZK/MPC work does not
cover (ZK proves answer correctness; DP bounds *what an answer leaks*). Natural for Solid pods that
publish cohort stats; composes with federation (per-source budgets).

**Novelty vs existing.** Genuine gap — confirmed no `laplace`/`differential`/`epsilon` anywhere in
`crates/`. No prior `research/` doc on DP.

**Prior art.** **"Differential privacy and SPARQL"**, Semantic Web Journal 2024 (Dumontier, Kirrane,
Seneviratne, Buil-Aranda, Lobo, Olmedo) — <https://content.iospress.com/articles/semantic-web/sw233474>,
<https://www.semantic-web-journal.net/content/differential-privacy-and-sparql> (dp-schema +
elastic-sensitivity over star joins; COUNT class; **no open-source release found**). DP-S4S
(select-join-aggregate, user-level DP) — <https://arxiv.org/pdf/2603.14994>. Foundational:
Dwork–Roth.

**Privacy honesty.** DP gives a *formal, tunable* guarantee but **only for the supported query
class** (the SWJ paper is COUNT over star-shaped BGPs; its own reviewers flagged proof gaps and
schema-per-query evaluation issues). Sensitivity bounds outside that class are **not** guaranteed —
any sparq DP surface must hard-refuse unsupported queries rather than silently under-noise. This is
a strong honesty boundary; the privacy-claims CI gate applies.

**Opt-in?** Yes — `sparq-dp` crate over the engine's aggregate path; zero default cost.

**Effort.** **L** (the noise mechanisms are easy; the *sound* sensitivity analysis + dp-schema
checker is the hard, must-be-correct part — under-noising is a privacy break, so this needs careful,
narrow scope and possibly external review before any claim).

**Decision-question.** Is **differentially-private aggregate release** a direction you want for
sparq (Solid-pod stats), accepting that v1 must be **narrowly scoped to a provably-bounded query
class** and explicitly refuse everything else? If yes, COUNT-only first, matching the SWJ paper?

---

## P3. Credential-gated query: `acp:vc` / VC-based access control (close the research-open Solid gap)

**What.** Implement **`acp:vc`** — ACP authorization conditioned on the requester presenting a
**Verifiable Credential** satisfying a stated constraint (issuer, type, claim predicate) — so a
query is authorized iff the caller holds e.g. a valid "over-18" or "licensed-clinician" VC. Two
verification backends: **(a)** plain VC verification (signature + issuer + claim check) for the
trust-the-issuer case, and **(b)** the existing `sparq-zk` estate for **zero-knowledge** credential
satisfaction (prove "I hold a valid over-18 VC" without revealing the credential).

**Why (concrete).** `acp:vc` is one of the **explicitly research-open** gaps in
`research/solid-vocab-gaps-design.md`: "needs VC verification machinery (ZK/VC estate)." It is the
canonical privacy-preserving access pattern (prove an attribute, not an identity) and the most
direct fusion of the maintainer's two pillars — **Solid access control** and the **ZK/VC estate** —
that sparq has the pieces for but has not wired together.

**Fit.** Bull's-eye on the maintainer's direction; the design note already anchors it to the
`verifiable-credentials-zk` skill and the `sparq-zk*` crates. Reuses the ACP N3 materializer
(`sparq-solid`) and either P1's plain VC verification or the ZK holder-PoP work already in
`research/zk-holder-pop-design.md`.

**Novelty vs existing.** Genuine, *already-identified* gap (research-open in the solid-vocab doc).
The ZK building blocks (commitments, issuer sigs, holder-PoP design) exist; the **ACP↔VC wiring +
the `acp:vc` materialization rule + the verifier-binding** do not.

**Prior art.** W3C VC Data Model 2.0 — <https://www.w3.org/TR/vc-data-model-2.0/>; Solid ACP `acp:vc`
— <https://solid.github.io/authorization-panel/acp-specification/>; zkSPARQL / "ZK proof of correct
SPARQL evaluation over Verifiable Credentials" — <https://zksparql.org/>; Braun/Wright/Käfer ESWC
2026 (soundness over VCs, cited in sparq's own ZK plan).

**Privacy honesty.** The ZK backend's "prove attribute without revealing credential" guarantee is
**only as strong as the unaudited ZK estate** (external sign-off pending, sq-qhy4). The plain backend
(a) is trust-the-issuer, not zero-knowledge. The §2.4 content-boundary concern from the solid design
must be respected: a VC is an *external* input, not pod content, so it does not re-open the
smuggling surface — but this must be argued in the design, not assumed.

**Opt-in?** Yes — extend `sparq-solid` with an `acp-vc` feature; ZK backend gated behind `sparq-zk`.

**Effort.** **L** (plain-VC backend is M; the ZK-backend binding + a sound `acp:vc` rule that
respects §2.4 is the L part). Phase: plain VC first, ZK backend second.

**Decision-question.** Should `acp:vc` credential-gated query be built now, starting with the
**plain (trust-the-issuer) VC backend** and adding the **ZK backend** once external ZK sign-off
(sq-qhy4) lands — or is it blocked on that audit entirely?

---

## P4. Offline-first CRDT sync for the embedded GUI app (RDF replica + conflict-free merge)

**What.** An opt-in `sparq-sync` crate giving the **embedded-engine GUI app** a **local-first**
story: a CRDT-backed quad replica that mutates offline and **deterministically merges** with a Solid
pod (or peer) on reconnect, with no central coordinator and no lost-update conflicts. Concretely: a
per-quad **OR-Set** (observed-remove set) over named graphs + a causal-context/version-vector, with
a Solid backend (read/write `.ttl` + a CRDT metadata sidecar) and the Solid Notifications protocol
(P5) as the live-update transport.

**Why (concrete).** The maintainer's stated GUI direction is a **downloadable embedded-engine app
with persistent workspaces**. A desktop app that holds a working RDF set *must* tolerate going
offline and later reconciling with a pod — that is the defining requirement of a local-first app.
Without CRDT semantics, two devices editing the same pod graph silently clobber each other.

**Fit.** Directly serves the GUI-as-app + Solid directions; the embedded engine is the ideal place
for it (the engine already owns the quad store + RDFC-1.0 canon for content-addressing).

**Novelty vs existing.** Genuine gap — no CRDT/replica/offline-sync code in `crates/`. The GUI
design record exists but does not cover offline merge.

**Prior art.** **m-ld** (RDF/JSON-LD CRDT replicas) — <https://ceur-ws.org/Vol-2941/paper1.pdf>,
<https://m-ld.org/>; **NextGraph** (local-first Semantic Web, OR-Set graph CRDT) —
<https://docs.nextgraph.org/en/framework/crdts/>; **locorda** (Dart, CRDT→Solid pod sync) —
<https://github.com/locorda/locorda>; Solid CRDT discussions —
<https://forum.solidproject.org/t/application-of-crdts-to-solid/3321>. General CRDT theory: Shapiro
et al.

**Opt-in?** Yes — standalone `sparq-sync` crate; only the GUI/desktop build pulls it in. Core engine
and WASM library stay lean.

**Effort.** **L** (OR-Set over quads + version vectors is well-trodden; the *correct* Solid backend
mapping — where to store CRDT metadata without polluting the content graph, and how to bound
metadata growth/tombstones — is the real design work; honesty: unbounded tombstone growth is the
classic CRDT footgun and must be addressed, not hand-waved).

**Decision-question.** Is offline-first **CRDT sync** part of the embedded-GUI vision (local edits
reconcile with a pod), and if so do you want to **build on or interop with m-ld/NextGraph's model**
versus a native OR-Set — and is a metadata-sidecar-on-pod storage shape acceptable?

---

## P5. Solid Notifications + type/shape-index discovery (live updates + federated resource discovery)

**What.** An opt-in `sparq-solid` extension for **(a)** the **Solid Notifications Protocol**
(WebSocketChannel2023 / WebhookChannel2023 — subscribe to a pod resource, receive change
notifications) on both the *client* side (the embedded app/federation client reacts to pod changes)
and optionally the *server* side (sparq-server emits notifications on resource change); and **(b)**
**type-index / shape-index discovery** (read a pod's `publicTypeIndex` / `solid:forClass`
registrations to **discover which resources hold which RDF types** for query planning and federated
discovery).

**Why (concrete).** Two pillars at once: (a) makes the embedded GUI and the streaming/RSP estate
**reactive to pod changes** (the live-data half of a local-first app, and the transport P4 needs);
(b) the **type index is the standard Solid mechanism for resource discovery** — without it, a client
must blindly crawl a pod to find, say, all `vcard:Contact` resources. Type-index-driven planning is
exactly the federated-discovery direction.

**Fit.** Solid + federation + streaming-RSP convergence; the RSP-QL engine and the federation client
already exist as the consumers.

**Novelty vs existing.** Genuine gap — grep confirms **no** Solid Notifications, inbox, or
type/shape-index code anywhere in `crates/`. `sparq-rsp` does streaming over *windows*, not over
*pod-change notifications*; the federation client discovers via Service Description / VoID, not via
Solid type indexes.

**Prior art.** Solid Notifications Protocol — <https://solid.github.io/notifications/protocol>,
WebSocketChannel2023 <https://solid.github.io/notifications/websocket-channel-2023>, WebhookChannel
<https://solid.github.io/notifications/webhook-channel-2023>; Solid Type Indexes —
<https://github.com/solid/type-indexes>; Shape Trees / shape index —
<https://shapetrees.org/>.

**Opt-in?** Yes — feature on `sparq-solid` (client) and an optional `sparq-server` notification
emitter feature. Core unaffected.

**Effort.** **M** (notifications client is a websocket/webhook subscriber + an event model;
type-index discovery is a small read+parse surface; server-side emission is the larger optional
piece). Phase: client notifications + type-index read first; server emission later.

**Decision-question.** Do you want sparq to be a **Solid Notifications client** (and feed P4 +
RSP), and to use the **type index for federated resource discovery** in planning — and is
**server-side notification emission** from `sparq-server` in scope or deferred?

---

## Recommendation (priority order)

Given the maintainer's clearest stated directions (privacy via ZK/MPC, Solid/decentralized,
federation, GenAI, GUI-as-embedded-app) and the lean-core rule, I recommend in priority order:

1. **P3 — `acp:vc` credential-gated query** (high). It is *already research-open*, fuses the two
   pillars (Solid AC + ZK/VC), reuses the most existing machinery, and has the clearest user story.
   Plain-VC backend first (does not block on the pending ZK audit).
2. **P1 — W3C Data Integrity signed datasets/results** (high). Cheap (reuses `sparq-canon`),
   standards-interop, the obvious non-ZK complement to the ZK estate, and a building block for P3.
3. **R1 — OWL 2 EL classification** (med-high). Closes the most defensible *correctness* gap in the
   reasoner (RL is incomplete for the flagship EL ontologies) and feeds GenAI/introspection.
4. **P5 — Solid Notifications + type-index** (med). Enabling infrastructure for the GUI/federation
   directions and a prerequisite-shaped piece for P4.
5. **P4 — CRDT offline-first sync** (med). High-value for the GUI vision but the heaviest design;
   sequence after P5 (which provides its live transport).
6. **R3 — SPARQL-callable analytics + model-UDF recipe** (med). High leverage-per-effort glue of
   existing estates; do once a maintainer confirms the SPARQL-call surface shape.
7. **P2 — differential privacy** (low-med). Real and aligned, but the *soundness burden is severe*
   (under-noising is a privacy break) and the only found algorithm has reviewer-flagged gaps —
   pursue only with a commitment to narrow scope + external review.
8. **R2 — datalog± / chase** (low-med). The most ambitious; only if "reasoning parity with
   RDFox/Nemo" is genuinely a goal. Stratified-negation+aggregation slice first; chase deferred.

## Phased plan (each phase = a future bead for the orchestrator)

1. **Maintainer triage of this record** — pick which of P3/P1/R1/P5/P4/R3/P2/R2 are in-scope; answer
   the eight decision-questions. (No code; gates the rest.)
2. **P1 phase 1** — `sparq-sign` crate: `eddsa-rdfc-2022` signed-dataset + signed-result over
   `sparq-canon`; verify path; conformance vectors. (Opt-in crate.)
3. **P3 phase 1** — plain-VC `acp:vc` backend: VC verification + ACP materialization rule respecting
   §2.4; fixtures (allow/deny by issuer+claim). (Feature on `sparq-solid`; depends on P1's verify.)
4. **R1 phase 1** — OWL 2 EL consequence-based classifier behind `owl-el`; subsumption-lattice output
   + SNOMED-shaped test ontology; honest completeness statement. (Feature on `sparq-reason`.)
5. **P5 phase 1** — Solid Notifications *client* (WebSocket/Webhook subscribe) + type-index read for
   discovery. (Feature on `sparq-solid`.)
6. **P3 phase 2** — ZK `acp:vc` backend wired to `sparq-zk` holder-PoP, **gated on external ZK
   sign-off (sq-qhy4)**; caveated privacy claims. (Depends on phase 3 + audit.)
7. **R1 phase 2 / R2 phase 1** — OWL 2 QL query-rewriting **or** datalog± stratified-negation+
   aggregation (whichever the triage prioritizes). (Feature on `sparq-reason`.)
8. **P4 phase 1** — `sparq-sync` OR-Set quad replica + version vectors + Solid backend, using P5 as
   live transport; tombstone-growth bound addressed explicitly. (Opt-in crate; GUI build only.)
9. **R3 phase 1** — SPARQL-callable `sparq-algos` surface + weighted/path/Louvain algorithms +
   model-backed-UDF recipe over the existing registry + `sparq-vectors`. (Feature on `sparq-algos`.)
10. **P1 phase 2** — BBS selective-disclosure signing **iff** the triage wants it (else closed as a
    deliberate non-goal vs the ZK estate). (Extends `sparq-sign`.)
11. **P2 phase 1** — `sparq-dp` COUNT-only DP over a declared dp-schema, **narrowly scoped + refuse
    unsupported queries**, pre-claim external review. (Opt-in crate; only if triage greenlights.)
12. **R2 phase 2** — existential-rule chase + acyclicity/termination analysis, behind its own
    sub-feature. (Deferred-most; only if R2 is greenlit.)

## Open questions that genuinely need the maintainer

- **Reasoner ceiling:** is *complete* EL/QL reasoning (R1) and/or datalog±/chase (R2) in scope, or
  is RL+N3's sound-but-incomplete materialization the deliberate, permanent ceiling?
- **Two verifiable lanes:** do you want the W3C-interop **signed** lane (P1, trust-the-signer)
  *alongside* the ZK lane, and how should they be presented so users pick the right guarantee?
- **DP appetite:** is differentially-private release (P2) worth the soundness/audit burden for
  sparq, or out of scope as too easy to get subtly wrong?
- **CRDT build-vs-interop:** for P4, native OR-Set vs interop with m-ld/NextGraph's existing CRDT
  model — and is a CRDT-metadata-sidecar on the pod an acceptable storage shape?
- **`acp:vc` sequencing (P3):** ship the plain-VC backend now and add the ZK backend post-audit, or
  hold the whole feature until external ZK sign-off (sq-qhy4)?

## Uncertainties (honest)

- The SWJ "Differential privacy and SPARQL" algorithm has **reviewer-flagged proof gaps and a
  schema-per-query evaluation concern** (from the published reviews) and **no open-source release was
  found** — so P2's v1 is a *re-implementation from the paper*, not a port, and must be treated as
  research-grade until independently checked. Marked as an uncertainty, not a settled plan.
- No **Rust** OWL 2 EL/QL reasoner was found in the surveyed landscape; R1's effort estimate assumes
  the ELK calculus transfers cleanly to Rust, which is plausible but unverified at code level.
- Effort sizes are relative judgements, **not** measured; no performance numbers are asserted
  anywhere in this record (work-box timings would be non-canonical regardless).
- Whether `acp:vc` (P3) can be made sound *without* re-opening the §2.4 content-smuggling boundary is
  argued as plausible (a VC is an external input, not pod content) but needs a dedicated design note
  before implementation, exactly as the solid-vocab record requires for its other gaps.
