<!-- [OPUS-4.8] Prior-art research authored by Opus 4.8 (1M context) — Fable unavailable; flag for re-review when Fable returns. -->
# Prior art for the "trust graph": trust & data integrity in RDF / Semantic Web

Status: **research / prior-art survey** (design-for-review). NO production code lands
from this document. It is one domain slice of the prior-art base for @jeswr's proposed
**trust graph** — the per-source / per-statement-type set of rules a storage server uses
to decide *which sources it trusts for which access-control statements*, so that
trusted-source-attested statements (e.g. a government VC `<Jesse> <age> 25`) merge with
`.acl`/`.acr` rules via reasoning to derive access. This slice covers **trust & data
integrity in RDF**: named-graph/quad-level trust + provenance, the Semantic-Web "trust
layer", RDF Dataset Canonicalization (RDFC-1.0), W3C VC Data Integrity proof suites,
N3/RDF-Surfaces reasoning with scoped formulae, the cwm/EYE reasoners, and trust
ontologies — i.e. *the core mechanism by which a resource decides which **source** to
trust for which **statements***.

> **Honesty / scope (read first).**
> - This is a **prior-art survey**, not a soundness claim about anything. Where it
>   describes sparq's own shipped estate, it distinguishes implemented-and-verified
>   from designed-only from not-yet-sound, and cites the file.
> - **ZK/MPC caveat:** sparq's ZK/MPC estate is research-stage. `verify_manifest` is
>   remediated + internally re-audited but **external accredited-cryptographer sign-off
>   is PENDING** (`sq-qhy4`); MPC is **semi-honest-only**. Nothing here presents any
>   ZK/MPC property as a proven/production guarantee. The "superset of ZKaps" claim is
>   examined as a *design hypothesis*, not asserted.
> - **No fabricated numbers.** The only figures cited are sparq's own checked-in
>   baselines (the `sparq-solid` materialization measurements in
>   `research/solid-access-control-design.md` §6, recorded on a non-canonical work-box —
>   NOT canonical). No external benchmark numbers are invented.
> - One **uncertainty flagged up front:** "ZKap" / "ZKaps" in @jeswr's framing is almost
>   certainly *his own coined term* from the transfer-of-status report / ISWC DC line of
>   work — a **zero-knowledge access pass/capability** built from VCs — and is **distinct
>   from** Least Authority's unrelated payment-token "ZKAPs" (Tahoe-LAFS / Privacy-Pass
>   blinded tokens). I could not retrieve a public, citable definition of @jeswr's ZKap
>   (the transfer-of-status blog post is a stub linking a PDF I could not read; §0.2).
>   The "superset of ZKaps" mapping in §3.4 is therefore built on the *capability-VC*
>   reading and must be confirmed by the maintainer.

---

## 0. Framing: what the trust graph is, and the gap it sits in

### 0.1 The two distinct decisions an access-control system makes

Every access-control evaluation answers two questions that the literature (and the WGs)
routinely conflate:

1. **Authentication / context trust** — *who is the requester, and which facts about the
   requester do I take on trust?* (Their WebID; the OIDC IdP that vouched for it; their
   client app.) This is the dimension Solid WAC/ACP already model: `acl:agent`,
   `acp:agent`, `acp:client`, `acp:issuer`.
2. **Authorization-statement trust** — *whose **statements** do I trust to **decide**
   access, and for **which** statements?* I.e. when an `.acl` rule says "anyone over 18
   may read R", **whose assertion** of `<Jesse> age 25` do I believe — Jesse's own pod
   (untrusted self-assertion), a UK-government VC (trusted for age), a random third party
   (untrusted)? And does the same source I trust for `age` get trusted for `nationality`?

The **trust graph is the formal object for decision (2)** — a per-source, per-statement-
**type** trust relation, *also expressed as RDF/rules*, that gates which attested
statements are admitted into the reasoning that derives `auth:read`/`auth:write`. It is
the missing predicate between sparq-solid's two existing trusted channels (§0.3).

### 0.2 The maintainer's own line of work (the primary source for the design)

The design descends from @jeswr's **"verifiable data sublanguage"** programme —
SPARQL-over-Verifiable-Credentials with minimal disclosure ("Is Jesse over 21 according
to facts issued by EU or UK governments → reveals only *yes*"), decomposed into RQ1
(single-holder ZK-over-VCs) and RQ2 (federated MPC+ZK). This is already the spine of the
repo's `research/mpc-zkp-research-and-architecture.md` and the `sparq-zk*` crates. Primary
sources, per the repo's own citations:

- **Wright, ISWC 2025 Doctoral Consortium**, CEUR Vol-4085 paper19
  (`https://ceur-ws.org/Vol-4085/paper19.pdf`) — the RQ1/RQ2 decomposition, named-graph
  credential model, minimal-disclosure framing. *(Cited from the repo; I did not
  independently re-fetch the PDF in this slice — flagged as a verify-before-cite item.)*
- **Wright, Transfer-of-Status report, Hilary 2025**
  (`https://blog.jeswr.org/2025/05/06/transfer-of-status`) — the blog post I fetched is a
  **stub** linking a PDF I could not read in this environment; the substantive ZKap /
  trust-graph definitions are presumably in that PDF. **This is the single most important
  gap in my sourcing** (§5, open question 1).

The trust graph as pitched to **LWS (Linked Web Storage WG)** + **Solid CG/WG** is the
*access-control-facing* projection of that programme: it is how the verifiable-data-
sublanguage's attested statements actually drive a storage server's authorization.

### 0.3 Where the gap is in sparq TODAY (verified against the code)

sparq already ships the two *endpoints* this design must bridge, but **not the bridge**:

- **`sparq-solid` (shipped, verified).** WAC `.acl` + ACP `.acr` are stored as ordinary
  named graphs and compiled by **N3 rules** (run by `sparq-reason`) into a materialized
  `<urn:sparq:auth>` authorization view; queries are filtered fail-closed per
  `(WebID, client, issuer)` session
  (`crates/sparq-solid/src/{lib,materialize,loader}.rs`,
  `crates/sparq-solid/rules/*.n3`;
  `research/solid-access-control-design.md`). **Crucially:** `acp:issuer` here is treated
  as a **trusted *context* dimension** — the OIDC IdP that vouched for the WebID, matched
  as a session attribute (`acp-b.n3`, `acp-c.n3`; `Session{issuer}` in `lib.rs`). It is
  **not** a per-statement trust decision over *attested content*. And `acp:vc` (the ACP
  attribute that would actually pull a VC's *claims* into the decision) is **explicitly
  out of scope / unimplemented** (`research/solid-access-control-design.md` §3.6, §7
  item 4).
- **`sparq-zk` / `sparq-zk-compose` (research-stage, not externally audited).** The
  issuer-signed per-named-graph commitment pipeline — RDFC-1.0 canonicalization +
  Poseidon2 commitments + issuer signatures + a disclosed issuer key-set `K` — is exactly
  the cryptographic substrate for "this statement was attested by issuer X"
  (`crates/sparq-zk/src/{encode,commit,sig}.rs`;
  `research/zkp-query-proofs-plan.md`, `research/zk-signed-credential-representation-design.md`).
- **`sparq-prov` (shipped).** PROV-O lineage for *derived* data (`prov:Activity`,
  `prov:wasDerivedFrom`, optional `prov:Agent`) — the provenance vocabulary a trust
  decision can be *recorded against*, but no trust *evaluation* over it
  (`crates/sparq-prov/src/lib.rs`).

**The missing predicate.** Today an attested statement `<Jesse> age 25` *cannot* flow
into the sparq-solid reasoner and merge with an `.acl` rule, because the §2.4 security
boundary of that reasoner **deliberately excludes all content** except `.acl`/`.acr`/group
graphs and a narrow trusted `solidx:` channel (else any writer could self-grant). The
trust graph is precisely the *principled relaxation* of that boundary: a declarative
statement of **which sources' content is admitted, for which statement types**, so that
admitting `<gov> says <Jesse> age 25` is a *trust-graph-licensed* fact rather than a hole.

---

## 1. The key models / specs in this domain

Each entry: what it offers · access-control mechanism · delegation story · trust model ·
limitations. Ordered roughly foundational → modern.

### 1.1 Named Graphs, Provenance and Trust (Carroll, Bizer, Hayes, Stickler, WWW 2005)

- **What it offers.** The foundational extension of RDF from triples to **named graphs
  (quads)** *specifically so that statements can describe, sign, and attach trust policy
  to other graphs*. It gives named graphs an abstract syntax, formal semantics, and an
  N3-based syntax, and frames named graphs as **"a foundation for the Semantic Web trust
  layer."** Publishers communicate *assertional intent* and **sign their graphs**;
  consumers evaluate graphs under **task-specific trust policies** and act only on graphs
  they accept.
- **AC mechanism.** Not access control per se — it is the *substrate*: a consumer applies
  a trust policy (a logical condition over the named graph + its provenance metadata) to
  decide whether to *believe and act on* a graph. "Which source for which statements" is
  exactly the consumer-side trust policy the paper anticipates but does not standardize.
- **Delegation.** Via warrants/signatures: graph G can assert "graph G' is trustworthy" /
  "agent A asserts G'", chaining belief — an informal precursor to capability chaining.
- **Trust model.** Pluggable, *consumer-decided* — the relying party's policy is primary;
  no global trust authority. This is the exact philosophy the trust graph should inherit.
- **Limitations.** Pre-dates DIDs/VCs, RDFC-1.0, and modern signature suites; "trust
  policy" is left to the application; no per-statement-**type** granularity (trust is
  per-graph); no delegation/attenuation formalism.

### 1.2 Annotated RDF / Trust Models for RDF Data (Zimmermann, Lopes, Polleres, Straccia, JWS 2012; Hartig tSPARQL 2009; Chudasama/Carral/… "Trust Models for RDF Data", AAAI 2022)

- **What it offers.** A *general algebraic framework* for attaching annotations (trust
  values, provenance, fuzzy/temporal labels) to RDF triples, with a **semiring**-based
  semantics so reasoning/querying **propagates the annotation** through inference
  (a derived triple's trust = combine of its premises' trust). **tSPARQL** adds `TRUST`
  keywords to SPARQL to query/threshold trust values. The AAAI 2022 line gives
  model-theoretic semantics + complexity for *trust-annotated* RDF.
- **AC mechanism.** Indirect: a query/policy can *threshold* on the propagated trust
  value (admit a fact only if its derived trust ≥ τ). This is the closest formal precedent
  to "merge attested statements with rules and only believe sufficiently-trusted
  derivations."
- **Delegation.** None native; trust is a scalar/lattice label, not a transferable
  capability.
- **Trust model.** Quantitative/lattice (a trust *degree*), propagated by a semiring —
  more general than the boolean "trusted-for-X / not" the trust graph likely wants, but
  the *propagation* machinery is directly reusable.
- **Limitations.** Annotation propagation through a *materializing* N3 reasoner is not
  free (sparq-reason materializes ground triples; carrying a per-triple trust label
  through forward chaining is a real extension); per-statement-**type** trust (trust gov
  for `age` but not `address`) is expressible only by making the annotation depend on the
  predicate — possible but not the framework's focus.

### 1.3 W3C VC Data Integrity 1.0/1.1 + RDF Dataset Canonicalization (RDFC-1.0) + EdDSA/ECDSA cryptosuites

- **What it offers.** The standards-track way to make an RDF graph **issuer-attested**:
  a `DataIntegrityProof` is produced by canonicalizing the credential's RDF dataset with
  **RDFC-1.0** (W3C Rec, the RDF Dataset Canonicalization algorithm — the URDNA2015
  successor; detects dataset-poisoning by default), hashing, and signing with
  `eddsa-rdfc-2022` / `ecdsa-rdfc-2022`. Output: a quad-set whose *content* is bound to an
  issuer key, verifiable by anyone. This is the **"trusted-source-attested statement"**
  primitive the trust graph consumes.
- **AC mechanism.** None itself — it answers "did issuer X attest this graph?", which is
  the **input** to a trust decision, not the decision.
- **Delegation.** None native (VC chains / `evidence` / holder-binding exist but
  delegation is a layer above — UCAN/ZCAP, §1.6).
- **Trust model.** PKI-of-issuers: trust roots at the issuer key (resolved via DID /
  `did:web` / key registry). The relying party decides *which issuer keys* it trusts —
  which is **exactly the trust-graph node** ("I trust key `did:web:gov.uk` for
  statements of type `age`").
- **Limitations.** Per-*graph* (or per-credential) attestation, not per-*statement-type*;
  RDFC-1.0 canonicalization is the well-known cost/complexity center (blank-node
  labeling); no statement-type-scoped trust — the relying party gets "issuer X signed
  this whole graph" and must itself decide which of the graph's statements to admit.
  sparq's `sparq-zk` uses the *same* RDFC-1.0 canonicalization + per-named-graph
  commitment, so the trust graph and the ZK estate share this substrate exactly.

### 1.4 N3 / Notation3 reasoning with scoped formulae + cwm + EYE (W3C N3 CG Report 2023; Berners-Lee/Connolly SWAP)

- **What it offers.** The rule language sparq-solid already uses. `{ … } => { … }`
  forward rules with **scoped negation-as-failure** (`log:notIncludes`/`log:includes`
  over an explicit formula or named source — a *contextually-scoped* NAF, not global),
  list/string/math builtins, and — the load-bearing ones for trust — `log:semantics` /
  `log:conclusion` (pull in, parse, and reason over a remote/local source) and
  `log:notIncludes { } …` scoped to the document "at a given point in time." cwm
  (Berners-Lee, forward) and **EYE** (De Roo/Verborgh, Euler-path backward + forward,
  loop-checking termination) are the reference reasoners.
- **AC mechanism.** N3 rules *are* the policy language: WAC/ACP-as-N3 (sparq-solid),
  AIR / "Accountability in RDF" policy reasoning, and the AMORD-lineage justification/
  proof traces. A rule body can scope what it trusts: `{ <gov> log:semantics ?g. ?g
  log:includes { <Jesse> :age ?a } } => …` is literally "believe the age statement *iff*
  it is in the graph dereferenced from `<gov>`."
- **Delegation.** Expressible but not packaged: a rule can grant on the basis of another
  rule/credential, and `log:semantics` lets a policy *import* a delegated sub-policy —
  but there is no standard attenuation/expiry/revocation envelope (UCAN/ZCAP provide
  that, §1.6).
- **Trust model.** *Per-rule, scoped.* The trust decision is wherever the rule author
  puts the `log:semantics`/scoped-NAF guard. This is the **most direct technical
  realization** of the trust graph: the trust graph is a *set of N3 rules* that say which
  source-graphs are admitted for which predicate patterns.
- **Limitations.** Scoped NAF under a *materializing, non-retracting* engine is only sound
  over predicates complete before the stratum (sparq's own §1.4/§3.5 lesson — stratify
  carefully); `log:semantics` (network import) is a *huge* trust/security surface (SSRF,
  poisoning) that sparq-solid deliberately does **not** enable (it never feeds content to
  the reasoner); termination/decidability needs care (EYE's loop-checking). The trust
  graph must constrain which sources `log:semantics`-style import is allowed from — that
  constraint *is* the trust graph.

### 1.5 RDF Surfaces (Verborgh, De Roo, et al., RuleML+RR 2024) — classical negation on the SW

- **What it offers.** A First-Order-Logic surface syntax over RDF (a sub-language of N3:
  graffiti blank nodes + graph terms) that adds **classical negation** ("explicitly say
  *no*") — used for misuse description, explainability/trust traces, and scope for
  reasoning over streams/queries. Run by **EYE** off-the-shelf. The **Community Solid
  Server v6.0 was extended for policy reasoning using EYE + RDF Surfaces** — a direct
  Solid-WG-relevant precedent for "policy as FOL reasoning over the pod's graphs."
- **AC mechanism.** Policy-as-FOL: a surface can express a prohibition that classically
  *denies* (vs N3's NAF deny), which matters for ODRL-style prohibitions and for
  deny-overrides semantics the trust graph will need.
- **Delegation.** Same as N3 (rule-level), plus the ability to state explicit negative
  authority ("X may *not* delegate Y").
- **Trust model.** Per-surface/per-rule, like N3, but with sound classical negation —
  closer to what a *standards-track* trust-graph semantics would want (no NAF
  non-monotonicity surprises).
- **Limitations.** FOL is semi-decidable; EYE handles the practical fragment but the
  general case does not terminate; sparq-reason today implements **scoped NAF, not RDF
  Surfaces' classical negation** (a real gap if the trust-graph semantics chooses
  classical negation — would be a `sparq-reason` extension).

### 1.6 Capability delegation: ZCAP-LD (W3C CCG) + UCAN (UCAN WG) — the delegation pillar

- **What it offers.** Object-capability authorization as **chained, signed, attenuable**
  documents. **ZCAP-LD** (Authorization Capabilities for Linked Data, W3C-CCG draft):
  capabilities are Linked-Data documents; delegation chains capability documents;
  **caveats** restrict scope (actions, expiry, revocation hook); invocation is separated
  from authorization. **UCAN** (User-Controlled Authorization Network): JWT-structured,
  DID-keyed, public-key-verifiable, **attenuated** chains (each delegation can only
  *narrow* authority), local-first, no central authority. The two are close cousins
  (ZCAP-LD ≈ URL/multi-proof, UCAN ≈ CID/single-proof JWT).
- **AC mechanism.** The capability *is* the grant — possession + valid chain to a root
  authority = authorized. Caveats/attenuation encode "for which resource, which action,
  until when."
- **Delegation.** **This is their entire point** — and the dimension sparq-solid's
  current model lacks (WAC/ACP grant to identities, not transferable capabilities).
  Attenuation (monotone narrowing) is the formal guarantee.
- **Trust model.** Root-authority + cryptographic chain; trust roots at the resource
  controller's key. No global authority; verifiable by any holder.
- **Limitations.** Capability ≠ attribute: ZCAP/UCAN say "the holder of this chain may
  read R," **not** "the holder is over 18 per the UK gov" — they delegate *authority*,
  not *attested attributes*. The trust graph's claimed-superset relationship is precisely
  that it should subsume **both** the attribute-VC path (admit `<Jesse> age 25` from a
  trusted issuer) **and** the capability-delegation path (admit "Alice delegated read to
  Bob's AI agent") as *the same kind of trusted-source-attested statement*. Neither
  ZCAP-LD nor UCAN is RDF-rule-native the way N3 policy is — bridging them into the N3
  reasoning is the design work.

### 1.7 AI-agent delegation: DIDs+VCs for agents; OAuth on-behalf-of; "Authenticated Delegation" (2025)

- **What it offers.** The 2025 wave of *machine* principals: equipping each AI agent with
  a self-controlled **DID + VCs**, OAuth 2.0 **on-behalf-of** / "act"-claim drafts
  (Internet-Draft, not WG-adopted), and **"Authenticated Delegation and Authorized AI
  Agents"** (arXiv 2501.09674) — time-limited, minimally-scoped, auditable credentials
  *tied to a verifiable human principal* with runtime intent checks. MCP 2025-11-05
  mandates OAuth 2.1 with scoped delegation tokens.
- **AC mechanism.** Delegated, scoped tokens / VCs presented by the agent; the resource
  checks the chain back to the human principal + the scope.
- **Delegation.** Human → agent (and agent → sub-agent) with scope/expiry — the
  "capability delegation for human AND AI agents" requirement maps here directly.
- **Trust model.** Human-rooted: an agent's authority is only ever as broad as the human
  principal delegated, cryptographically bound. This is the *exact* shape the trust graph
  needs for the AI-agent half — and it is **VC/DID-native**, so it composes with the
  attribute-VC path (§1.3) under one "trusted-source-attested statement" abstraction.
- **Limitations.** Fast-moving, mostly *not* standards-track yet (drafts); token-centric
  (OAuth) rather than RDF-statement-centric; no Semantic-Web rule integration. The trust
  graph's value-add is to render agent-delegation as RDF statements that *reason* with
  the same `.acl` rules, instead of a parallel OAuth pipe.

### 1.8 Prior RDF access-control surveys + query-rewriting (Kirrane et al. 2017; Stardog/GraphDB FGAC)

- **What it offers.** The systematization sparq-solid already cites: **Kirrane et al.,
  "Access control and the RDF: a survey"** (Semantic Web 8(2), 2017) maps the design
  space — **query-rewriting** enforcement vs **materialized/annotated** enforcement, at
  triple/graph granularity. **Stardog named-graph security** (silently drop unreadable
  graphs) and **GraphDB fine-grained access control** (per-quad-pattern rules) are the
  productized points.
- **AC mechanism.** Either inject filters into the algebra (rewrite) or pre-label
  decisions (materialize) — sparq-solid is the hybrid (materialize the decision, rewrite
  to a dataset view).
- **Delegation / trust.** These survey *enforcement*, not *source-trust*; trust of the
  policy itself is assumed (the policy is the server's own). The trust graph extends them
  by making *which policy statements are admitted* a function of source-trust.
- **Limitations.** None handle externally-attested attribute statements driving the
  policy — they assume a single trusted policy author. That is the precise extension.

---

## 2. How this domain maps to / informs the trust-graph design

The trust graph, expressed in the vocabulary of this domain, is a **per-source,
per-statement-type, RDF-native, reasoning-evaluated trust relation** that *gates which
attested statements enter the authorization derivation*. Concretely, the domain gives the
design a near-complete construction kit:

1. **The attested-statement unit is a Carroll-Bizer named graph (§1.1) made
   issuer-bound by VC Data Integrity / RDFC-1.0 (§1.3).** This is *already* sparq's
   `sparq-zk` per-named-graph commitment unit — so the trust graph's input is
   byte-aligned with the existing crypto estate. A "trusted-source-attested statement" =
   *a named graph whose RDFC-1.0 canonical form is signed by an issuer key the trust graph
   trusts for that statement's predicate(s)*.

2. **The trust relation itself is RDF + N3 rules (§1.4 / §1.5).** The trust graph is *not*
   a new file format; it is a small set of triples + N3 rules such as
   `{ ?g zk:signedBy ?k . ?k trust:trustedFor schema:age . ?g log:includes { ?s schema:age ?a } } => { ?s schema:age ?a }`
   — "import the age statement into the reasoning *iff* its source graph is signed by a
   key trusted (in the trust graph) for `schema:age`." This is the **principled relaxation
   of the §2.4 sparq-solid content/reasoner boundary**: instead of "no content ever," it
   is "content admitted exactly as the trust graph licenses, by predicate."

3. **The merge-with-`.acl` step is the existing sparq-solid materialization (§0.3,
   §1.8).** Once admitted, `<Jesse> age 25` is just another input fact to the same N3
   strata that already compile WAC/ACP. The `.acl` rule
   `{ ?x schema:age ?a . ?a math:greaterThan 18 } => { ?x auth:read ?r }` fires *if and
   only if* the trust graph admitted the age fact — exactly the claimed
   "attested-statements merge with `.acl` rules via reasoning to derive access."

4. **Per-statement-TYPE granularity is the genuine novelty over §1.1/§1.3** (which are
   per-graph) **and over §1.2** (which is per-triple but type-agnostic). The trust graph
   keys trust on *(source, predicate-pattern/shape)* — "trust `did:web:gov.uk` for
   `schema:age` and `schema:nationality`, but not for `acl:agent`." SHACL shapes
   (sparq-shacl ships) are the natural way to express "statement type," so the trust graph
   can reuse `sh:targetClass`/property shapes to scope what a source is trusted *for*.

5. **Delegation rides the same rail (§1.6 / §1.7).** A ZCAP/UCAN delegation or an
   AI-agent on-behalf-of VC is *also* a trusted-source-attested statement: "Alice (root
   authority of R) attests that Bob's agent may read R, attenuated to read-only, expiring
   T." Rendered as RDF and admitted via the trust graph, it reasons with the *same*
   `.acl` rules — unifying attribute-trust and capability-delegation under one mechanism.
   This is the structural basis for the "capability delegation for human AND AI agents"
   requirement, and for the **"superset of ZKaps"** claim (§3.4).

6. **Annotation/propagation semantics (§1.2) is the optional quantitative upgrade.** If the
   trust graph ever needs *degrees* of trust (not boolean trusted-for-X), the
   semiring-annotated-RDF machinery + tSPARQL thresholding is the off-the-shelf semantics;
   but a boolean per-(source,type) relation is simpler and likely sufficient for v1 —
   recommend boolean v1, semiring as a tracked extension.

7. **RDFC-1.0 canonicalization is shared infrastructure, already in sparq** (`sparq-canon`
   / `sparq-zk`). The trust graph does not need new canonicalization — it consumes the
   same canonical named-graph commitment the ZK pipeline produces, so a trust-graph
   decision can be made *with or without* a ZK proof (the proof only adds minimal
   disclosure; the trust relation is the same).

**The clean architectural statement:** the trust graph is the **policy that governs the
relaxation of sparq-solid's content/reasoner boundary** (`solid-access-control-design.md`
§2.4), turning that boundary from "all content excluded" into "content admitted exactly
as a per-(source, statement-type) trust relation licenses," with the relation itself
expressed as RDF + N3 rules and evaluated by the existing materialization pipeline.

---

## 3. The gaps this domain leaves that the trust graph must address

The prior art supplies every *ingredient* but **no assembled system that does all four of:
(per-source × per-statement-type) trust, RDF-rule-native evaluation, attested-content
admission into authorization reasoning, AND unified human/AI capability delegation.** The
specific gaps:

### 3.1 No per-(source, statement-TYPE) trust object exists as a standard
§1.1/§1.3 are per-graph; §1.2 is per-triple but type-agnostic; ZCAP/UCAN are per-resource-
capability. **Nothing standardizes "source S is trusted for statements matching shape T."**
The trust graph must *define this vocabulary* (a `trust:` ontology: `trust:trustsSource`,
`trust:forStatementType` keyed to a SHACL shape or predicate set, `trust:underConditions`)
— this is the design's primary standards-track contribution and the thing to take to LWS/
Solid WG. **Verify**: does the WG want SHACL-shape-scoped or predicate-set-scoped types?

### 3.2 Admitting external content into the reasoner is a security boundary sparq deliberately CLOSED
`solid-access-control-design.md` §2.4 is emphatic: feeding content to the reasoner lets a
writer self-grant. The trust graph *re-opens* this boundary **by policy**, which re-creates
the escalation risk unless: (a) admission is gated on a *verified* issuer signature over
the source graph's RDFC-1.0 commitment (not on self-asserted `zk:signedBy` triples — those
must be checked, not trusted, exactly as the `solidx:` reserved-predicate guard does); and
(b) the trust graph itself is in a *trusted* channel (who may edit the trust graph? — the
resource controller, via `acl:Control`-analogous authority). **This is the single biggest
soundness gap**: the trust graph turns a closed boundary into a policy-controlled one, and
the policy + the signature verification must be airtight. The §2.4 forgery tests
(`acp_forged_*_in_acr_document_does_not_grant`) are the template for the regression suite
the trust graph needs.

### 3.3 Statement-type trust interacts with reasoning/entailment non-trivially
If a source is trusted for `schema:age` but the reasoner *derives* `schema:age` from a
`schema:birthDate` it is **not** trusted for (via an ontology rule), does the derived age
inherit trust? §1.2's annotation-propagation says "trust = combine of premises," which
would *deny* the derived age (correct, conservative). But sparq-reason **materializes
ground triples without trust labels** — so a naive merge would lose the provenance and
admit a laundered fact. **The trust graph must either (a) forbid trusted statements from
participating in untrusted derivations (stratify: admit only *directly attested* facts of
the trusted type, never derived ones), or (b) carry trust annotations through the closure
(the §1.2 extension to sparq-reason).** Recommend (a) for v1 — simpler, sound, and matches
the "directly attested" intuition; flag (b) as the research extension.

### 3.4 The "superset of ZKaps" claim is unproven and term-ambiguous
Two problems: (i) **term ambiguity** — "ZKap" is the maintainer's coined capability-VC
term, not the Least Authority payment token; this MUST be disambiguated in the standards
doc (§5 Q1). (ii) **the superset relation is a design hypothesis, not established.** The
*plausible* reading: a ZKap is a zero-knowledge-provable capability VC ("I hold a valid,
unrevoked, attenuated delegation to read R, proven without revealing my identity"). Under
the trust-graph framing, that is **one species** of trusted-source-attested statement (a
delegation statement, §1.6/§1.7), admitted by the trust graph, optionally with a ZK proof
for minimal disclosure (the sparq-zk pipeline, §1.3). The trust graph is a *superset*
because it *also* admits **attribute** statements (`age 25`) and **arbitrary
predicate-typed** statements, and lets them **merge with `.acl` rules via reasoning** —
which a pure capability token cannot do. **This is a credible superset argument, but it
is a claim to be reviewed under the same caveats as the rest of the ZK estate** (no
soundness asserted; external audit pending, `sq-qhy4`). It must NOT be stated as proven.

### 3.5 Revocation, freshness, and time-bound trust are unsolved at the RDF-rule layer
VC Data Integrity has status lists; UCAN/ZCAP have revocation hooks; but the trust graph
merges *materialized* facts — a revoked/expired attestation must be **re-evaluated**, and
sparq-solid's materialization is currently **full-re-run on change** with no incremental/
temporal maintenance (`solid-access-control-design.md` §4.2, §7 item 3). Temporal trust
("trusted for age *as of* the credential's validity window") needs a time dimension in the
trust rules + the `now`-aware session (`Session{now}` already exists in `lib.rs` — a hook),
and revocation needs the status-list check wired into admission. **This is a real
materialization-semantics gap**, not just a vocabulary one.

### 3.6 Standards alignment: LWS/Solid WG want a story that degrades to today's WAC/ACP
The trust graph must be **strictly additive** — a pod with no trust graph behaves exactly
as WAC/ACP do now (fail-closed, identity-based). The §1.8 query-rewriting/materialization
enforcement and the sparq-solid `<urn:sparq:auth>` view are the back-compat anchor. The WG
will also ask how the trust graph relates to ACP's existing (unimplemented) `acp:vc`
attribute — the trust graph should be positioned as **the formal semantics `acp:vc` always
needed**: ACP says "this policy is satisfied if the context carries VC V," but never
specified *which issuer of V to trust for which claim* — the trust graph is that missing
predicate. **Verify with the WG** whether to extend `acp:vc` or introduce a sibling
`trust:` vocabulary.

---

## 4. Recommendation + phased plan (each phase = a future bead)

**Recommendation.** Build the trust graph as a **new opt-in `sparq-trust` crate that sits
between an attested-source layer and the existing `sparq-solid` materialization**, NOT as a
change to `sparq-core`/`sparq-engine` (per the opt-in-feature architecture). Concretely:
the trust graph is **(a)** a `trust:` RDF vocabulary for the per-(source, statement-type)
relation; **(b)** a set of N3 admission rules run by `sparq-reason` that gate which
*signature-verified* attested named graphs' triples enter the WAC/ACP reasoning; **(c)** a
verification step that checks the RDFC-1.0 issuer signature (reusing `sparq-zk`'s commitment
pipeline) *before* admission, so admission is never on self-asserted trust triples. Boolean
per-(source, type) trust for v1; directly-attested-only (no laundering through untrusted
derivations, §3.3 option a); strictly additive over today's WAC/ACP (§3.6). Treat the
"superset of ZKaps" and any ZK-minimal-disclosure path under the live ZK/MPC caveats —
design-for-review only, no soundness claim. Take the `trust:` vocabulary + the admission
semantics to LWS/Solid WG as the standards contribution.

**Phased plan (ordered; each = a future bead for the orchestrator):**

1. **Disambiguate + ground the design from the maintainer's primary sources.** Read the
   transfer-of-status PDF + ISWC DC paper (the parts on ZKap + trust graph), pin the
   *exact* maintainer definitions of "trust graph" and "ZKap," and confirm the §3.4
   superset reading. **Blocks everything** — resolves §5 Q1.
2. **Define the `trust:` vocabulary + formal admission semantics (design doc).** The
   per-(source, statement-type) relation; statement-type = SHACL shape vs predicate set
   (§3.1); the admission rule shape (§2.2); directly-attested-only stratification (§3.3);
   classical-negation-vs-NAF choice (§1.5). Cross-reference ACP `acp:vc` (§3.6). Output: a
   standards-track-ready vocabulary + semantics record.
3. **Specify the signature-verified admission boundary (security design + threat model).**
   How a trust-graph rule's `zk:signedBy`/issuer premise is *verified* (not trusted) before
   admission, reusing `sparq-zk` RDFC-1.0 commitment + issuer-key check; who may edit the
   trust graph (Control-analogous authority); the forgery/escalation regression suite
   modeled on `solid-access-control-design.md` §2.4. Closes the §3.2 gap on paper.
4. **Prototype `sparq-trust` admission layer (opt-in crate, design + spike).** N3 admission
   rules + a verification shim feeding admitted facts into `sparq-solid` materialization;
   the strictly-additive degrade-to-WAC/ACP property; a fixture where a gov-VC `age` merges
   with an `.acl` `>18` rule to derive access. Measure materialization cost vs the
   `sparq-solid` §6 baseline (work-box, non-canonical).
5. **Delegation unification: human + AI-agent capability statements (design).** Map
   ZCAP/UCAN attenuation + the 2025 authenticated-delegation/on-behalf-of patterns
   (§1.6/§1.7) onto trusted-source-attested *delegation* statements admitted by the trust
   graph; specify attenuation as monotone narrowing in the admission rules. Substantiate
   (or refute) the §3.4 "superset of ZKaps" claim *as a reviewed design argument*.
6. **Revocation, freshness, temporal trust (design + materialization-semantics extension).**
   Wire VC status-list / UCAN-revocation checks into admission; time-bound trust via the
   existing `Session{now}` hook; the incremental/temporal re-materialization gap
   (§3.5, ties to `solid-access-control-design.md` §7 item 3).
7. **Optional quantitative trust + minimal-disclosure (research extension).** Semiring-
   annotated-RDF trust *degrees* + tSPARQL thresholding (§1.2) if boolean proves
   insufficient; and the ZK-minimal-disclosure admission path ("admit *that* a trusted age
   statement exists, without revealing the value") via `sparq-zk` — strictly under the live
   ZK caveats, design-for-review.
8. **Standards engagement: LWS + Solid WG.** Bring phases 2/3/5 to the WGs; position the
   trust graph as the formal semantics `acp:vc` always needed (§3.6); reconcile with the
   shipped `sparq-solid` WAC/ACP back-compat anchor.

---

## 5. Open questions that genuinely need the maintainer

1. **"ZKap" definition + sourcing (BLOCKER).** Confirm the term is your own
   capability-VC construct (not Least Authority's payment ZKAPs), and point me at the
   exact definition (transfer-of-status PDF section / ISWC DC). The §3.4 superset mapping
   depends entirely on this. *(I could not read the PDF in this environment.)*
2. **Statement-TYPE granularity.** Should "what a source is trusted for" be a **SHACL
   shape** (reuses `sparq-shacl`; expressive) or a **predicate/IRI set** (simpler;
   standards-lighter)? This shapes the whole `trust:` vocabulary.
3. **Trusted-derivation policy (§3.3).** v1: admit only *directly-attested* facts of a
   trusted type (no laundering through untrusted ontology derivations) — agreed, or do you
   want trust-annotation propagation through the closure from day one?
4. **Relation to ACP `acp:vc` (§3.6).** Extend the (currently-unimplemented) `acp:vc`
   attribute with trust-graph semantics, or introduce a sibling `trust:` vocabulary that
   composes with both WAC and ACP? Affects the WG pitch.
5. **Who may edit the trust graph?** `acl:Control`-analogous (resource controller only),
   or a separate higher authority? This is the §3.2 escalation-boundary decision.
6. **Boolean vs degree trust (§1.2).** Boolean per-(source, type) for v1, with the
   semiring/tSPARQL degree model as a tracked extension — acceptable?

---

## 6. Citations

External (verified via WebSearch/WebFetch in this slice unless noted):

- Carroll, Bizer, Hayes, Stickler. *Named Graphs, Provenance and Trust.* WWW 2005.
  `https://dl.acm.org/doi/10.1145/1060745.1060835` ·
  `http://wbsg.informatik.uni-mannheim.de/bizer/SWTSGuide/carroll-ISWC2004.pdf`
- Zimmermann, Lopes, Polleres, Straccia. *A General Framework for Representing, Reasoning
  and Querying with Annotated Semantic Web Data.* (annotated RDF / semiring).
  `https://arxiv.org/pdf/1103.1255`
- Hartig. *tSPARQL: a trust-aware extension to SPARQL* (trust-value annotation + query).
  *(cited via the AAAI/ScienceDirect trust-model results below; verify exact ref.)*
- *Trust Models for RDF Data: Semantics and Complexity.* AAAI 2022.
  `https://ojs.aaai.org/index.php/AAAI/article/view/9169`
- W3C. *Verifiable Credential Data Integrity 1.0/1.1.* `https://www.w3.org/TR/vc-data-integrity/`
  · `https://w3c.github.io/vc-data-integrity/`
- W3C. *Data Integrity EdDSA Cryptosuites v1.0* (`eddsa-rdfc-2022`).
  `https://www.w3.org/TR/vc-di-eddsa/` · ECDSA: `https://www.w3.org/TR/vc-di-ecdsa/`
- W3C. *RDF Dataset Canonicalization (RDFC-1.0).* `https://www.w3.org/TR/rdf-canon/`
- W3C N3 Community Group. *Notation3 Language* (report, 2023).
  `https://w3c.github.io/N3/reports/20230703/` · scoped NAF discussion:
  `https://github.com/w3c-cg/N3/issues/18` · `https://notation3.org/`
- Berners-Lee, Connolly, et al. *Notation3 / SWAP* (cwm). `https://www.w3.org/2000/10/swap/doc/paper/index.pdf`
- Verborgh, De Roo, et al. *RDF Surfaces: Enabling Classical Negation on the Semantic Web*
  (RuleML+RR 2024; EYE; CSS v6 policy reasoning). `https://arxiv.org/pdf/2406.10659` ·
  `https://link.springer.com/chapter/10.1007/978-3-031-72407-7_15`
- W3C-CCG. *Authorization Capabilities for Linked Data (ZCAP-LD) v0.3.*
  `https://w3c-ccg.github.io/zcap-spec/`
- UCAN WG. *User-Controlled Authorization Network — Delegation.*
  `https://github.com/ucan-wg/delegation` · `https://ucan.xyz/specification/`
- *Authenticated Delegation and Authorized AI Agents.* arXiv 2501.09674.
  `https://arxiv.org/pdf/2501.09674` · *AI Agents with DIDs and VCs.*
  `https://arxiv.org/html/2511.02841v1`
- Kirrane, Mileo, Decker. *Access control and the Resource Description Framework: a
  survey.* Semantic Web 8(2), 2017.
  `https://www.semantic-web-journal.net/system/files/swj1280.pdf`
- *Query Based Access Control for Linked Data.* `https://arxiv.org/pdf/2007.00461`
- Stardog named-graph security; GraphDB fine-grained access control (productized FGAC,
  cited in `research/solid-access-control-design.md` §1.3).

Internal sparq (read directly in this repo):

- `crates/sparq-solid/{src/lib.rs,src/materialize.rs,src/loader.rs,rules/*.n3}` —
  shipped WAC/ACP-as-N3 + issuer-as-context dimension + `acp:vc` out of scope.
- `research/solid-access-control-design.md` — the §2.4 content/reasoner boundary, the
  materialization pipeline, the §6 (non-canonical work-box) baselines, the §3.6 gaps.
- `crates/sparq-zk/src/{encode,commit,sig}.rs` + `research/zkp-query-proofs-plan.md` +
  `research/zk-signed-credential-representation-design.md` — RDFC-1.0 + per-named-graph
  issuer-signed commitment substrate (research-stage; external audit pending `sq-qhy4`).
- `crates/sparq-prov/src/lib.rs` — PROV-O derivation lineage (no trust evaluation).
- `research/mpc-zkp-research-and-architecture.md` — the verifiable-data-sublanguage /
  RQ1-RQ2 programme + disclosed issuer key-set `K` (the attested-source root of trust).
- `research/feature-research-odrl-policy.md` + `crates/sparq-policy` — ODRL usage-control
  layer above access control (sibling, not the trust graph).

Sources (markdown, for the search-result reminders):
[Carroll-Bizer Named Graphs](https://dl.acm.org/doi/10.1145/1060745.1060835) ·
[Annotated RDF framework](https://arxiv.org/pdf/1103.1255) ·
[Trust Models for RDF (AAAI)](https://ojs.aaai.org/index.php/AAAI/article/view/9169) ·
[VC Data Integrity](https://w3c.github.io/vc-data-integrity/) ·
[EdDSA cryptosuites](https://www.w3.org/TR/vc-di-eddsa/) ·
[RDFC-1.0](https://www.w3.org/TR/rdf-canon/) ·
[N3 report](https://w3c.github.io/N3/reports/20230703/) ·
[RDF Surfaces](https://arxiv.org/pdf/2406.10659) ·
[ZCAP-LD](https://w3c-ccg.github.io/zcap-spec/) ·
[UCAN delegation](https://github.com/ucan-wg/delegation) ·
[Authenticated Delegation for AI agents](https://arxiv.org/pdf/2501.09674) ·
[Kirrane RDF AC survey](https://www.semantic-web-journal.net/system/files/swj1280.pdf)
