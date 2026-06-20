<!-- [OPUS-4.8] Prior-art research authored by Opus 4.8 (Fable 5 unavailable) — re-review when Fable returns. -->
# Prior-art domain: RBAC / ABAC / NGAC / ReBAC formal models, mapped to the trust-graph design

> Research-for-review record. NO production code lands here. Domain feed #1 (of several)
> for the maintainer's **"trust graph"** design — the per-source, per-statement-type set of
> rules a storage server uses to decide *which sources it trusts for which access-control
> statements*, where trusted-source-attested statements (e.g. a government VC
> `<Jesse> <age> 25`) merge with `.acl`/`.acr` rules via reasoning to derive access,
> with capability delegation for human **and** AI agents, claimed to be a superset of
> ZKaps. This doc covers the **formal access-control-model** prior art (RBAC, ABAC, XACML,
> NGAC, ReBAC/Zanzibar/OpenFGA) plus the two adjacent families this design actually needs:
> **distributed trust-management** (RT, XACML delegation) and **capability/credential
> delegation** (ZCAP-LD, UCAN, VC trust registries). Companion sibling research (ODRL,
> ZK/MPC) lives in `research/feature-research-odrl-policy.md`,
> `research/mpc-zkp-research-and-architecture.md`, and the `zk-*` records.
> [OPUS-4.8]

## 0. Honesty preamble + a correction to the brief's premise

**Verified against the codebase, not taken on faith.** Two corrections / refinements the
maintainer should note before reading the survey:

1. **sparq is NOT a blank slate on access control or on trust-of-source.** It already ships
   `sparq-solid` (`crates/sparq-solid`): WAC + ACP encoded as N3 rules, run by the
   `sparq-reason` N3/EYE-class reasoner, materialising a per-principal **authorization view**
   (`<urn:sparq:auth>`) that a zero-copy `DatasetView` enforces at query time
   (`research/solid-access-control-design.md`, *shipped*). It also already has **two of the
   exact primitives the trust-graph generalises**:
   - an **`acp:issuer` dimension** (sq-3jtd.6) — ACP can scope a grant to "the OIDC issuer
     that vouched for this WebID", minting a three-component principal
     `urn:sparq:triple?agent=A&client=C&issuer=I`. That is *already* a coarse, single-axis
     "trust which source asserted the subject's identity" knob.
   - a **trusted-fact channel** `AccessProvenance` (`crates/sparq-solid/src/provenance.rs`,
     sq-3jtd.5): per-resource creator/owner facts are accepted **only** through a Rust API
     the storage layer (PSS) calls, *never* from pod content or even from the `.acr`
     document itself (the loader hard-rejects `solidx:`-namespaced triples smuggled into a
     control doc). This is a hand-built, single-purpose instance of exactly the question the
     trust-graph asks generically: *which channel/source is authoritative for which
     statement-type?*
   - `sparq-zk-compose` carries an **external issuer trust-anchor key set** — the relying
     party commits to "the set of issuer public keys I trust" and a credential signed
     outside it is rejected (`crates/sparq-zk-compose/tests/e2e.rs`, `forge_gates.rs`).
     That *is* a trust-of-source list, today, for the ZK credential path.

   So the honest framing is: **the trust-graph is a generalisation and unification of
   primitives sparq already has in hard-coded, single-axis form** — not a greenfield. That
   is a strength for the proposal (it has a working substrate and an existence proof) and it
   sharpens the novelty claim (the contribution is the *general per-source/per-statement-type
   trust layer*, not the idea of caring about issuers at all).

2. **"Superset of ZKaps" is plausible as an *expressiveness* claim but must be stated
   carefully.** ZKaps (Least Authority's Zero-Knowledge Access Passes) are a *specific*
   construction: blinded, unlinkable single-use passes built on Privacy Pass, redeemed
   without revealing which pass. A trust-graph that derives access from attested attributes
   can *model the access-granting semantics* of a ZKap-style capability, and sparq's ZK
   estate can supply the *unlinkability*; but the trust-graph by itself does not deliver
   ZKap's cryptographic blinding/unlinkability — that is the job of `sparq-zk`/`sparq-mpc`,
   which are **research-stage, internally re-audited, and PENDING external accredited-
   cryptographer sign-off (sq-qhy4); MPC is semi-honest-only.** So the correct claim is:
   *the trust-graph generalises the **authorization model** ZKaps express, and composes with
   sparq's (not-yet-externally-audited) ZK layer to recover the privacy property* — not "the
   trust-graph is, on its own, a proven superset of ZKaps." All ZK/MPC mentions here are
   caveated for the live privacy-claims gate.

Everything below is grounded in the cited specs/papers; where I am uncertain I say so.

## 1. The five formal models, each on the same axes

The brief asks, for each model: subject/attribute/role/policy modelling; how policy is
evaluated; the delegation/administration story; the trust model; and what a trust-graph of
trusted-source-scoped statements would ADD. I take them in increasing relevance to the
design.

### 1.1 RBAC — ANSI/INCITS 359 (Ferraiolo–Kuhn / Sandhu RBAC96)

- **What it offers / how it models.** A *Reference Model* of four components: Core RBAC,
  Hierarchical RBAC, Static SoD, Dynamic SoD. Core RBAC is five sets — `USERS`, `ROLES`,
  `OPS`, `OBS`, `PRMS` (permissions = `OPS × OBS`) — plus relations `UA ⊆ USERS × ROLES`
  (user-assignment) and `PA ⊆ PRMS × ROLES` (permission-assignment), and `SESSIONS` mapping
  a user to an activated subset of their roles. A role is "a means for naming a many-to-many
  relationship among users and permissions." Hierarchical RBAC adds a partial order on roles
  where senior roles inherit junior permissions.
- **Evaluation.** `access(user, op, obj)` ⇔ ∃ role *r* activated in the session with
  `(user,r)∈UA*` (`*` = hierarchy closure) and `((op,obj),r)∈PA`. Pure set membership +
  transitive closure — cheap, and the canonical "before-the-fact audit" (per-user and
  per-object review) is *trivial* because roles are explicit named relations.
- **Delegation / administration.** Not in the core standard; ARBAC97 and many RBAC-delegation
  extensions exist but are not part of INCITS 359. Administration is "an administrator edits
  `UA`/`PA`."
- **Trust model.** Single administrative authority; roles are asserted by that authority. No
  notion of "a *different* source asserted this user-role fact and do I trust it."
- **Limitation vs trust-graph.** Roles are *coarse pre-computed bundles*; there is no attribute
  reasoning, no environment, and crucially **no provenance on the `UA` facts** — RBAC cannot
  express "trust HR for the `manager` role but trust the government for `over-18`."

**What the trust-graph adds:** roles become a *derived* view — `role(?x, manager)` is just a
trust-graph rule firing on an HR-attested attribute. The trust-graph subsumes RBAC's
many-to-many naming while making the *source* of each assignment first-class.

Sources: [CSRC RBAC / role-engineering](https://csrc.nist.gov/projects/role-based-access-control/role-engineering-and-rbac-standards),
[ANSI INCITS 359 overview](https://blog.ansi.org/ansi/role-based-access-control-rbac-incits-359/),
[CSRC RBAC FAQ](https://csrc.nist.gov/projects/role-based-access-control/faqs).

### 1.2 ABAC — NIST SP 800-162

- **What it offers / how it models.** "Authorization to perform a set of operations is
  determined by evaluating *attributes* of the subject, object, requested operation, and
  (optionally) environment against *policy/rules/relationships*." No roles required; access
  is a Boolean function of attribute values. Functional decomposition into PEP / PDP / PIP /
  PAP (the same four points XACML names).
- **Evaluation.** PEP intercepts → PDP evaluates policy, pulling missing attributes from PIPs
  → ALLOW/DENY (+ obligations). The decision is a logical formula over attribute name/value
  pairs.
- **Delegation / administration.** SP 800-162 treats attribute and policy *management* as
  enterprise governance, not a formal delegation calculus. It explicitly raises **trusted
  attribute sources / Attribute Authorities** as a first-class operational concern:
  organisations collect attributes "from authoritative sources" and a core challenge is
  "ensuring trusted sources of attribute information."
- **Trust model — this is the load-bearing one for the design.** ABAC's PIP abstraction
  *names the problem the trust-graph solves but does not formalise it*: an Attribute
  Authority (AA) maintains/issues attributes, the PDP consumes them, and **whether the PDP
  should trust a given AA for a given attribute is an out-of-band, deployment-level decision.**
  There is no standardised, machine-checkable per-attribute "I trust AA *X* for attribute
  *a*" statement inside the model.
- **Limitation vs trust-graph.** Exactly that gap: ABAC assumes attributes arrive
  pre-trusted; provenance-of-attribute and per-(source, attribute-type) trust scoping live in
  prose, not in the policy language.

**What the trust-graph adds:** it *is* the missing formal PIP-trust layer — a queryable graph
of `trustedFor(source, statementType, constraints)` statements that the reasoner consults
when an attested attribute (a VC) is offered, so "trust gov for `age`, trust the employer for
`department`" becomes data, not configuration.

Sources: [NIST SP 800-162 (final, upd2)](https://csrc.nist.gov/pubs/sp/800/162/upd2/final),
[SP 800-162 PDF](https://nvlpubs.nist.gov/nistpubs/specialpublications/nist.sp.800-162.pdf).

### 1.3 XACML 3.0 — OASIS (the standard ABAC policy language, + its Delegation Profile)

- **What it offers / how it models.** XML (also JSON) policy language for ABAC. Hierarchy
  `PolicySet → Policy → Rule`; each has a `Target` (attribute-match predicate selecting
  applicability) and Rules have a `Condition` and an `Effect` (Permit/Deny). Attributes are
  pulled by `AttributeDesignator`/`AttributeSelector` from four categories (subject, resource,
  action, environment). PEP/PDP/PIP/PAP + a Context Handler.
- **Evaluation.** Collect attributes → match Targets → evaluate Conditions → combine
  conflicting results with a **combining algorithm** (`deny-overrides`, `permit-overrides`,
  `first-applicable`, `only-one-applicable`, ordered variants) → return Permit/Deny/
  NotApplicable/Indeterminate **+ Obligations/Advice** the PEP must/should honour. Performance
  is "directly related to the number of policies considered," with policy-load and
  request-evaluation as the two cost phases (NIST SP 800-178).
- **Delegation / administration — the closest *standardised* analogue to the trust-graph.**
  The **XACML 3.0 Administration and Delegation Profile** introduces a `PolicyIssuer` element.
  A policy *without* `PolicyIssuer` is a **trusted policy** (the root of authority); a policy
  *with* one is an issued/untrusted policy that must be authorised by **reduction**:
  > "The process by which the authority of a policy associated with an issuer is verified. The
  > value of an unauthorized policy is discarded before combination, i.e., an unauthorized
  > policy is treated as if it did not exist."

  Reduction builds **administrative requests** mapping the issued policy's situation into the
  reserved `…:attribute-category:delegated:*` and `…:delegate` categories, then searches for a
  path of authorisation edges from the untrusted policy back to a trusted policy; absent such a
  path the policy is discarded. `MaxDelegationDepth` bounds chain length (administratively, not
  cryptographically).
- **Trust model.** Binary at the policy level: trusted (issuer-less) vs delegated. NIST
  SP 800-178 is blunt about the limit: XACML's decentralised administration "is **only a
  partial solution** in that it is dependent on trusted and untrusted policies, where trusted
  policies are assumed valid, and **their origin is established outside the delegation model**.
  Furthermore, the XACML delegation model does not provide a means for imposing policy over
  modification of access policies, and offers no direct administrative method for imposing
  policy over the management of its attributes."
- **Limitation vs trust-graph.** (i) The "who is trusted" root is *outside* the model. (ii)
  Reduction is over *policies*, not over *attribute statements* — XACML cannot natively say
  "trust this issuer for `age` but not for `clearance`"; you encode it by hand as
  administrative policies. (iii) It is a logical-formula engine, so NIST notes it "cannot do
  either type of [per-user/per-object] review efficiently."

**What the trust-graph adds:** it re-expresses XACML's delegation-reduction idea but (a) over
RDF *attribute statements* with per-(source, statement-type, constraint) granularity, (b) with
the trust root itself as *queryable graph data* rather than an out-of-band assumption, and (c)
on an engine (SPARQL + N3) where per-user/per-object review is a query, not an intractable
formula-inversion.

Sources: [XACML 3.0 core spec](https://docs.oasis-open.org/xacml/3.0/xacml-3.0-core-spec-os-en.html),
[XACML 3.0 Administration & Delegation Profile](https://docs.oasis-open.org/xacml/3.0/xacml-3.0-administration-v1-spec-en.html),
[NIST SP 800-178 (XACML vs NGAC)](https://nvlpubs.nist.gov/nistpubs/specialpublications/nist.sp.800-178.pdf).

### 1.4 NGAC — the Policy Machine, INCITS 565-2020 (the structurally closest model)

- **What it offers / how it models.** A **relations-and-architecture** ABAC standard. State =
  a directed graph whose vertices are users, **user attributes (UA)**, **object attributes
  (OA)**, objects, and **policy classes (PC)**, and whose edges fall in four categories:
  - **assignments** (`x → y`: x is contained in / gets the attributes of y),
  - **associations** `(ua, ops, oa)`: the privilege-granting edge — "members of UA may
    perform operations *ops* on the objects reachable under OA,"
  - **prohibitions** (deny relations, including conditional/dynamic), and
  - **obligations** (event → administrative-action triggers; the dynamic/history dimension).
- **Evaluation.** A decision `can(user, op, obj)` ⇔ there is an association `(ua, ops, oa)`
  with `user` assigned (transitively) into `ua`, `obj` contained (transitively) under `oa`,
  `op ∈ ops`, **consistent across every policy class** the object is under, and **not blocked
  by a prohibition.** The NGAC reference implementation computes this with a **single
  combining algorithm over the non-conflicting applicable policies**, **entirely in memory**,
  with a **linear** algorithm that is "not linear in relation to the entire access control
  data set, but only to the portion relevant to a particular user" (NIST SP 800-178). Both
  **per-user and per-object review** are efficient — NGAC's headline advantage over
  XACML/ACL/RBAC.
- **Delegation / administration — the strongest in the field.** Administration is *the same
  graph and the same decision function*: administrative operations (create/assign/associate/
  delete) are themselves controlled by associations over administrative attributes. NIST
  SP 800-178: NGAC "enables a systematic and policy-preserving approach to the creation of
  administrative roles and delegation of administrative capabilities, beginning with a single
  administrator and an empty set of access-control data … administrative capabilities down to
  the granularity of a single configuration element, and it can deny users administrative
  capabilities down to the same granularity." This is genuine, fine-grained, *in-model*
  delegation — no out-of-band trust root.
- **Trust model.** Closed, single-authority-rooted graph; the authority structure is itself
  in the graph (admin associations), but NGAC is not designed for *cross-organisational,
  cryptographically-attested* sources — every vertex/edge is asserted by the (delegated)
  administrators of one Policy Machine. There is no native "this UA fact came from an external
  issuer's signed VC; do I trust that issuer for it?"
- **Limitation vs trust-graph.** (i) Single-PM, intra-organisational: no built-in story for
  *federated, attested, mutually-distrusting* sources. (ii) Attributes are assigned by graph
  edges the PM owns, not derived from *external signed statements with provenance*. (iii) Open
  research: the **safety problem** (can an unsafe state be reached via obligations?) is
  studied and, in general settings, hard — relevant because a trust-graph that admits external
  attested facts inherits an analogous safety question.

**What the trust-graph adds and what it should BORROW.** NGAC is the model the trust-graph
most resembles and should *steal from*: (a) **decision = reachability over a labelled graph**,
which is exactly how sparq already materialises `<urn:sparq:auth>` and is a natural SPARQL/N3
workload; (b) **administration-as-the-same-graph** — the trust-graph's "who may edit which
trust statements" should itself be trust-graph data, NGAC-style, not an out-of-band root à la
XACML; (c) **per-user/per-object review as a first-class, tractable query.** What the
trust-graph adds *over* NGAC: every edge can be **provenance-scoped to an external attested
source**, so the graph spans organisational trust boundaries and the decision function gains a
"…and I trust *source S* to assert this edge for this statement-type" guard.

Sources: [NIST Policy Machine / NGAC](https://www.nist.gov/identity-access-management/policy-machine-and-next-generation-access-control),
[NIST SP 800-178](https://nvlpubs.nist.gov/nistpubs/specialpublications/nist.sp.800-178.pdf),
[Safety Analysis in the NGAC Model (arXiv 2505.06406)](https://arxiv.org/pdf/2505.06406),
[INCITS 565-2020 (NGAC)](https://dokumen.pub/incits-565-2020-information-technology-next-generation-access-control-ngac-april-10-2020nbsped.html).

### 1.5 ReBAC / Zanzibar / OpenFGA (the model the industry actually deploys)

- **What it offers / how it models.** Relationship-Based Access Control: authorization is a
  graph of **relation tuples**. Zanzibar's grammar (USENIX ATC '19):

  ```text
  ⟨tuple⟩    ::= ⟨object⟩'#'⟨relation⟩'@'⟨user⟩
  ⟨object⟩   ::= ⟨namespace⟩':'⟨objectid⟩
  ⟨user⟩     ::= ⟨user_id⟩ | ⟨userset⟩
  ⟨userset⟩  ::= ⟨object⟩'#'⟨relation⟩
  ```

  e.g. `doc:readme#viewer@user:alice`, or the *userset* form
  `doc:readme#viewer@group:eng#member` ("every member of group:eng is a viewer"). Per-namespace
  **userset rewrite rules** define relations computationally from three leaf operators:
  `_this` (stored tuples), `computed_userset` (relation X implies relation Y on the same
  object — e.g. editor ⇒ viewer), and **`tuple_to_userset`** (follow a tuple to another object
  and take *its* userset — e.g. "viewer of a doc = viewer of its parent folder"), combined with
  union/intersection/exclusion.
- **Evaluation.** `Check(user, object#relation)` succeeds iff a direct tuple exists *or*
  recursively the user is in an indirect userset; servers fan out and traverse the relation
  graph. Indirection (`tuple_to_userset`/usersets) **is** delegation/inheritance.
- **Consistency.** **Zookies/zedtokens** (a Spanner-TrueTime timestamp) solve the "new enemy"
  problem — checks evaluate at a snapshot, so a stale ACL is never applied to new content.
- **Delegation.** Native and elegant *within* the model: usersets and tuple-to-userset are
  delegation/group/inheritance. **OpenFGA** (the CNCF open re-implementation) adds
  **conditions** (ABAC predicates on tuples) and **contextual tuples** (request-time tuples,
  e.g. derived from token claims) — bridging ReBAC toward ABAC.
- **Trust model.** Tuples are written by trusted application servers into a single logical
  store; the model has **no notion of an externally-attested or mutually-distrusting source**.
  Who-may-write-which-tuple is an application concern outside the tuple model. There is no
  per-source trust scoping and no signed-statement provenance.
- **Limitation vs trust-graph.** (i) Tuples are flat triples with no issuer/provenance/
  cryptographic attestation. (ii) No attribute *reasoning* beyond the rewrite operators
  (OpenFGA conditions are limited predicates, not a general reasoner). (iii) Federation across
  distrusting orgs is out of scope.

**What the trust-graph adds:** RDF *is* the relation-tuple model generalised
(`object#relation@user` ≈ a quad with provenance); sparq's `<urn:sparq:auth>` materialised
grants already look like a Zanzibar relation set. The trust-graph adds (a) **provenance/issuer
on every tuple**, (b) **general N3 reasoning** in place of fixed rewrite operators, and (c)
**attested external sources**. Conversely the design should *borrow Zanzibar's freshness
discipline* (the zookie/new-enemy problem is real for any system that caches a materialised
auth view — sparq's epoch-bump-on-rematerialise is the same idea and should be stated in those
terms).

Sources: [Zanzibar (USENIX ATC '19) — authzed mirror](https://authzed.com/zanzibar),
[Zanzibar paper PDF](https://www.usenix.org/system/files/atc19-pang.pdf),
[OpenFGA concepts](https://openfga.dev/docs/concepts),
[OpenFGA conditions/contextual tuples](https://openfga.dev/docs/modeling/token-claims-contextual-tuples).

## 2. The two adjacent families the design genuinely needs (the AC models above don't cover)

The brief scopes RBAC/ABAC/NGAC/ReBAC, but those models *all* assume attributes/tuples arrive
pre-trusted. The trust-graph's defining feature — *per-source trust of attested statements +
delegation for human and AI agents* — lives in two adjacent literatures that the design must
engage, or it will reinvent them.

### 2.1 Distributed trust management — RT (role-based trust management)

The RT family (Li, Mitchell, Winsborough, *Design of a Role-Based Trust-Management Framework*,
IEEE S&P 2002) is the canonical formal answer to "**access control across administrative
domains where principals issue credentials about each other.**" RT credentials are statements
like "A.r ←— B" / "A.r ←— B.r1" / linked and intersection roles, with semantics given by a
**translation to Datalog**. It provides *localized authority over roles*, *delegation in role
definition*, *linked roles* (A trusts whoever B says is an r1), *intersection* (threshold /
separation-of-duty), and a distributed **credential-chain-discovery** algorithm for finding the
relevant credentials at internet scale. **This is essentially the trust-graph's theory already
worked out** — minus RDF, minus zero-knowledge, minus AI-agent delegation. The maintainer
should cite RT as the formal antecedent and frame the trust-graph as "RT's
attested-statements-as-Datalog idea, realised over RDF/N3 with VC-attested facts, ZK-capable
disclosure, and capability delegation."

Sources: [Design of a Role-Based Trust-Management Framework (semanticscholar)](https://www.semanticscholar.org/paper/Design-of-a-role-based-trust-management-framework-Li-Mitchell/9536ec8ebe6a7f21e6911f4bfccffd682c747ab4),
[Distributed Credential Chain Discovery in Trust Management (JCS 2003)](https://journals.sagepub.com/doi/10.3233/JCS-2003-11102),
[An Introduction to the RT Framework (Springer)](https://link.springer.com/chapter/10.1007/978-3-540-74810-6_9).

### 2.2 Capability / credential delegation — ZCAP-LD, UCAN, VC trust registries

This family supplies the *delegation* leg (human and AI agents) and the ZKaps connection.

- **ZCAP-LD** (W3C-CCG, *Authorization Capabilities for Linked Data*): object-capability
  authority expressed as **linked-data documents** signed with Linked Data Proofs, **delegated
  by chaining** capability documents, narrowable by **caveats** (restrict actions, enable
  revocation). Because it is linked data, it is *natively RDF* — it composes with the
  trust-graph rather than competing.
- **UCAN** (User-Controlled Authorization Network): a cryptographically-verifiable,
  hierarchical, content-addressed delegation token with a minimal policy language and
  invocation/delegation separation. Closely related to ZCAP-LD (the differences are
  formatting, URL-vs-CID addressing, single-vs-multiple proofs).
- **VC trust registries / issuer trust lists** (OpenID4VC / EUDI): the "triangle of trust"
  (Issuer–Holder–Verifier) plus a **verifiable data registry** the verifier queries to decide
  *which issuers it trusts for which credential types*. **This is literally the trust-graph's
  core question, already named in the SSI ecosystem** — the trust-graph generalises the trust
  registry from "list of trusted issuers" to "graph of per-source, per-statement-type,
  constraint-scoped trust rules merged with local `.acl` policy by reasoning."
- **AI-agent delegation** (the design's explicit requirement). The 2025–2026 standards motion
  is **OAuth 2.0 Token Exchange (RFC 8693)** on-behalf-of flows, an OAuth extension adding
  `requested_actor`/`actor_token` for agent delegation, SPIFFE/SVID workload identity, and the
  **intersection rule** (an agent's effective permission = user's permission ∩ agent's allowed
  capability) as the mitigation for the **confused-deputy** problem. The Stanford
  *Authenticated Delegation and Authorized AI Agents* work formalises delegated agent
  authority. **Implication for the trust-graph:** capability delegation must carry an
  *actor/agent* dimension distinct from the *subject/user* dimension, and the decision must
  enforce the intersection (the agent never gets more than the delegating human had) — this is
  the AI-agent-specific correctness property the design must state and the reasoner must
  enforce.

Sources: [ZCAP-LD v0.3 (W3C-CCG)](https://w3c-ccg.github.io/zcap-spec/),
[UCAN specification](https://ucan.xyz/specification/),
[UCAN delegation (GitHub)](https://github.com/ucan-wg/delegation),
[OpenID4VC / trust registries (iGrant.io)](https://docs.igrant.io/concepts/openID4vc/),
[OAuth Token Exchange RFC 8693 — agent on-behalf-of (IETF draft)](https://www.ietf.org/archive/id/draft-oauth-ai-agents-on-behalf-of-user-02.txt),
[Authenticated Delegation and Authorized AI Agents (Stanford)](https://digitaleconomy.stanford.edu/publication/authenticated-delegation-and-authorized-ai-agents),
[ZKAPs (Least Authority)](https://leastauthority.com/static/slides/HelloDecentralization2021_ZKAPs.pdf).

## 3. How this domain MAPS TO the trust-graph design

Synthesising the survey into the design's own terms (the running example: a government VC
`<Jesse> <age> 25` merging with `.acl` rule `{?x <age> ?y. FILTER(?y>18)} => {?x <canAccess> <r>}`):

| Trust-graph concept | Closest prior-art primitive | What the trust-graph generalises |
|---|---|---|
| Per-source, per-statement-type trust rule | ABAC Attribute Authority / PIP trust (prose); VC trust registry; XACML `PolicyIssuer` | Makes the *out-of-band trust assumption* into **queryable RDF data** with statement-type + constraint granularity |
| Attested attribute statement (the VC triple) | ABAC subject attribute; RT credential; OpenFGA contextual tuple | Adds **cryptographic provenance** + (optionally) ZK disclosure on the attribute |
| `.acl`/`.acr` rule merged with attributes via reasoning | XACML Rule/Condition; NGAC association; OpenFGA condition | Uses a **general N3 reasoner** (already shipped) instead of a fixed formula/rewrite operator set |
| Access decision = derived `<canAccess>` triple | NGAC reachability; Zanzibar Check; sparq `<urn:sparq:auth>` view | **Materialised, queryable** decision (per-user *and* per-object review are SPARQL queries) — the NGAC/RBAC tractability win, which XACML lacks |
| Capability delegation (human) | ZCAP-LD/UCAN chains; XACML delegation reduction; RT linked roles | Delegation as **trust-graph statements** reduced by reasoning; trust root is itself graph data (NGAC-style), not external (XACML's gap) |
| Capability delegation (AI agent) | OAuth 8693 actor_token; intersection rule; confused-deputy mitigation | Adds an **actor dimension** + the enforced **agent ⊆ delegator** intersection invariant |
| Trust-of-issuer for the attested fact | `acp:issuer` (already in sparq); ZK issuer key-set (already in sparq); OpenID4VC trust list | Generalises sparq's *single-axis hard-coded* issuer trust into a **general per-source/per-type trust graph** |

The single most important mapping: **NGAC is the structural template, RT is the formal theory,
the VC trust registry is the trust primitive, and ZCAP-LD/UCAN are the delegation primitive —
and sparq already has the *engine* (N3 reasoner + materialised auth view + DatasetView
enforcement) plus *single-axis instances* of the trust primitive (`acp:issuer`,
`AccessProvenance`, the ZK issuer key-set).** The design's novelty is the *unification* into a
general, queryable, reasoning-merged, provenance-scoped trust layer — none of the surveyed
systems do all four (general reasoning ∧ external attested provenance ∧ tractable review ∧
human+agent delegation), and none do it over RDF on a SPARQL engine.

## 4. The GAPS this prior art leaves that the trust-graph MUST address

1. **No model formalises per-(source, statement-type) trust as data.** ABAC names it (PIP
   trust), XACML approximates it at policy granularity (`PolicyIssuer` + reduction), VC trust
   registries do it as a flat issuer list. None gives a *graph of constraint-scoped trust
   rules per statement-type*. The trust-graph must define this object precisely — its
   vocabulary, its merge semantics with `.acl`/`.acr`, and its own access control (who may
   assert trust statements — answer, NGAC-style: the trust-graph itself).
2. **Provenance + attestation are absent from the AC models.** RBAC/ABAC/NGAC/ReBAC assume
   pre-trusted facts. The trust-graph must specify how an attested statement (VC) enters
   reasoning with its provenance intact and *cannot be forged by a writer* — sparq's existing
   `AccessProvenance` + `solidx:`-reserved-predicate rejection (`research/solid-access-control-
   design.md` §2.4) is the pattern to generalise, not re-invent.
3. **Open-world / non-monotonic safety.** Admitting external attested facts + N3
   negation-as-failure raises the NGAC **safety problem** in a federated setting: can a new
   credential or a trust-statement change reach an unintended grant? sparq's reasoner is
   **no-retraction NAF, stratified** (`research/solid-access-control-design.md` §1.4/§3.5) —
   the trust-graph's rules must stay stratified, and the design must state the safety property
   it guarantees (and the one it does not).
4. **Freshness / "new enemy."** Any materialised decision view (sparq's `<urn:sparq:auth>`,
   Zanzibar's cache) risks applying stale trust/credentials to new resources or vice-versa.
   Zanzibar solves it with zookies; sparq has epoch-bump-on-rematerialise. The design must
   make credential **revocation** and trust-statement change propagate to the view, and state
   the consistency model in zookie terms. (sparq has incremental-maintenance as a known
   follow-up, not yet shipped.)
5. **AI-agent delegation correctness.** No surveyed AC model carries an actor/agent dimension
   with the intersection (agent ⊆ delegator) invariant; that comes from the OAuth/agent
   literature. The trust-graph must add it and the reasoner must enforce it (a derived agent
   grant must be provably ≤ the delegating human's grant).
6. **The ZKaps superset claim needs the ZK layer, which is not yet externally sound.** The
   trust-graph can model the *authorization semantics* of ZKaps, but the *unlinkable,
   blinded-disclosure* property is delivered only by `sparq-zk`/`sparq-mpc`, which are
   **research-stage, internally re-audited, PENDING external accredited-cryptographer sign-off
   (sq-qhy4); MPC semi-honest-only.** The proposal must phrase the ZKaps relationship as
   *authorization-model superset + privacy via the (caveated) ZK layer*, never as a standalone
   proven cryptographic superset.
7. **Standards-track positioning.** For the LWS/Solid WGs the design must say precisely how the
   trust-graph relates to WAC/ACP (it *extends* them — attested-attribute matchers, of which
   `acp:vc`/`acp:issuer` are the seed) and to ODRL usage control (the layer above —
   `research/feature-research-odrl-policy.md`), and whether it is a new vocabulary, an ACP
   profile, or both.

## 5. Recommendation

**Adopt NGAC's structure + RT's theory + the VC-trust-registry primitive, realised on sparq's
existing N3-reasoner / materialised-auth-view substrate, and explicitly framed as a
generalisation of sparq's already-shipped single-axis trust primitives.** Concretely:

- **Model the trust-graph as RDF + N3 rules**, decision = a derived/materialised grant view
  (NGAC reachability ≈ today's `<urn:sparq:auth>`), so per-user *and* per-object review stay
  query-tractable (the NGAC/RBAC advantage XACML lacks).
- **Make trust-of-source first-class data** — a `trustedFor(source, statementType, constraint)`
  vocabulary — generalising `acp:issuer`, `AccessProvenance`, and the ZK issuer key-set, with
  the trust-graph's own administration governed NGAC-style by the trust-graph itself (avoid
  XACML's out-of-band-root gap).
- **Reuse the anti-forgery boundary** (`AccessProvenance` channel + reserved-predicate
  rejection) for attested-statement provenance; keep rules **stratified** for safety.
- **Add an actor/agent dimension with the intersection invariant** for AI-agent delegation,
  borrowing OAuth 8693 / Stanford authenticated-delegation semantics.
- **Cite RT, the XACML Delegation Profile, NGAC, Zanzibar, ZCAP-LD/UCAN, and the VC trust
  registry as the antecedents**, positioning the contribution as their *unification over RDF
  with attested provenance + (caveated) ZK disclosure*.
- **State the ZKaps relationship honestly** (authorization-model superset; privacy via the
  not-yet-externally-audited ZK layer).

This is a *feed* document; it does not decide the vocabulary or the standards artefact — those
are the maintainer's calls and the open questions below.

## 6. Phased plan (each phase = a future bead for the orchestrator)

1. **Trust-graph data model + vocabulary spec.** Define `trustedFor(source, statementType,
   constraint)` and the attested-statement representation; specify merge semantics with
   WAC/ACP; decide new-vocab vs ACP-profile. (Generalises `acp:issuer`/`acp:vc`.) *Future bead.*
2. **Reasoning + materialisation design.** Specify the N3 rule strata that merge attested
   attributes + trust statements + `.acl`/`.acr` into the auth view; prove stratification /
   state the safety property; reuse the `AccessProvenance` anti-forgery boundary. *Future bead.*
3. **Capability delegation — human.** Map ZCAP-LD/UCAN delegation chains + XACML-style
   reduction into trust-graph statements; define revocation + the freshness/consistency model
   (zookie-equivalent over the epoch). *Future bead.*
4. **Capability delegation — AI agent.** Add the actor dimension + the enforced agent ⊆
   delegator intersection invariant (OAuth 8693 / confused-deputy mitigation); reasoner test
   that a derived agent grant is provably ≤ the delegating human's. *Future bead.*
5. **ZKaps relationship + privacy composition.** Formalise the authorization-model-superset
   claim; specify the (caveated) composition with `sparq-zk`/`sparq-mpc` for unlinkable
   disclosure; keep all privacy claims gate-compliant. *Future bead (blocked on / caveated by
   sq-qhy4 external sign-off).*
6. **Standards-track write-up for LWS/Solid WGs.** Position vs WAC/ACP/ODRL; the antecedent
   citation set; the conformance/profile decision. *Future bead.*

## 7. Open questions that genuinely need the maintainer

- **Vocabulary vs profile.** Is the trust-graph a *new* W3C-track vocabulary, an **ACP
  profile** (attested-attribute matchers extending `acp:vc`/`acp:issuer`), or both? This
  changes the standards artefact and the sparq surface.
- **Trust root.** Should "who may assert trust statements" be governed *entirely* by the
  trust-graph itself (NGAC self-administration, no out-of-band root) or anchored to a
  deployment root key? The NGAC-pure answer is cleaner but heavier; the anchored answer
  matches XACML/VC-registry practice.
- **Reasoning power vs decidability.** How much N3 expressivity does the merge need, and can it
  stay within the stratified-NAF envelope sparq's reasoner guarantees? (Full RT is Datalog;
  unrestricted N3 + external facts can be undecidable.)
- **Agent-delegation strength.** Is the agent ⊆ delegator intersection enforced *statically*
  (in the rules) or *dynamically* (per request)? And does an AI agent's authority need to be
  *attenuable* mid-chain (UCAN-style caveats) or only fixed at delegation time?
- **ZKaps scope.** Is the "superset of ZKaps" claim meant as *authorization-expressiveness*
  (defensible now) or as a *cryptographic* claim (blocked on external ZK sign-off, sq-qhy4)?
  The proposal's framing depends on this.
- **Federation boundary.** Does the trust-graph operate within one storage server (the
  WAC/ACP scope today) or across federated sparq nodes (which pulls in the MPC envelope and
  its honest-majority/LAN limits)?
