<!-- [OPUS-4.8] Prior-art research for the trust-graph design (LWS/Solid WG track).
     Authored by Opus 4.8 (1M context); Fable unavailable — flag for re-review when Fable returns.
     Design-for-review: investigation only, NO implementation. -->
# Trust-graph prior art — domain: ODRL (Open Digital Rights Language)

> Scope: this is the **ODRL slice** of the prior-art survey feeding @jeswr's *trust-graph*
> design (to propose to the LWS + Solid WGs). The trust graph = the set of statements/rules a
> storage server or resource uses to decide **WHICH SOURCES it trusts for WHICH access-control
> statements** (per-source, per-statement-type), where trusted-source-attested statements (e.g.
> a government VC `<Jesse> <age> 25`) **merge with `.acl` rules** (e.g.
> `{?x <age> ?y. FILTER(?y>18)} => {?x <canAccess> <r>}`) via reasoning to derive access; with
> **capability delegation for human AND AI agents**; claimed a **superset of ZKaps**.
>
> This doc asks one question of ODRL: **what does ODRL already give the trust graph, and what
> does it leave unsolved?** It is deliberately narrower than the existing
> [`research/feature-research-odrl-policy.md`](./feature-research-odrl-policy.md) (which frames
> ODRL as a *usage-control / disclosure-control* layer); here ODRL is examined purely as a
> **policy-evaluation substrate for source-scoped trust + delegation**.
>
> Honesty posture (mandatory): claims below trace to cited specs/papers or to **read** sparq
> code; where I infer or extrapolate I say so. No ZK/MPC property is presented as proven — the
> v1 ZK verifier is remediated + internally re-audited but **external accredited-cryptographer
> sign-off is pending** (`sq-qhy4`) and `sparq-mpc` is semi-honest-only. No measured numbers.

---

## 0. What sparq already implements in this area (verified against code, not the brief)

Before surveying ODRL externally, the ground truth in-repo — because the trust-graph design
must build on what exists, and one sparq primitive is **already a degenerate trust graph**:

- **`sparq-solid` has an `acp:issuer` dimension** (`crates/sparq-solid/src/authindex.rs`,
  `crates/sparq-solid/rules/acp-a.n3` / `acp-b.n3`). A `Session` carries `{agent, client,
  issuer, now}`; an ACP matcher can constrain on `acp:issuer` (the OIDC IdP that vouched for
  the WebID). The N3 rules treat issuer as "the exact twin of the client dimension"
  (`acp-a.n3` comment, `[OPUS-4.8] sq-3jtd.6`): `auth:AnyIssuer ⊒ concrete issuer`, an absent
  `acp:issuer` means issuer-unconstrained. **This is a real, shipped, per-principal "which
  source vouched for the identity" check** — but it is coarse: it trusts an issuer *for the
  identity assertion itself* (authentication), not *per-statement-type* (it cannot say "trust
  gov.uk for `<age>` but not for `<role>`"). The trust graph generalises exactly this axis.
- **`acp:CreatorAgent` / `acp:OwnerAgent`** provenance matchers (`acp-a.n3`,
  `crates/sparq-solid/src/lib.rs` `materialize_with_provenance`): the storage layer supplies
  `created`/`owns` facts over a **trusted channel** (the `provenance` argument is documented as
  "the **trusted channel** for those facts"). This is the embryo of *the server deciding which
  facts to trust because of where they came from* — i.e. a trust-graph edge, hard-coded to one
  source (the storage layer) and one statement-type (creation/ownership).
- **Enforcement is reasoning-then-materialise-then-restrict.** WAC/ACP are N3 rule strata run
  by `sparq-reason`'s N3/EYE-class engine; they materialise an allow-list authorization view
  into a reserved named graph (`<urn:sparq:auth>`), and queries are restricted to the
  authorized graph-set via a zero-copy `DatasetView`, **fail-closed** (`D4`,
  `research/solid-access-control-design.md`). **The trust-graph's "attested statements merge
  with `.acl` rules via reasoning" is mechanically the same pipeline** — it adds *attested
  facts* as extra inputs to the same N3 closure and lets the rules derive `canAccess`.
- **`sparq-policy` is the ODRL bridge** (`crates/sparq-policy/`): a typed ODRL 2.2 subset
  (`model.rs`: `Policy`/`Rule`/`Action`/`Constraint`/`Operator`/`Value`/`Duty`) +
  fail-closed evaluator (`eval.rs`), deny-overrides, the common constraint operators
  (`eq/neq/lt/lteq/gt/gteq/isPartOf/isA`), `purpose`/`spatial` subsumption, and a
  count-enforcement store. **`publish=false`, single-node, dependency-of-nothing.** It does
  **not** model `assigner`-chained delegation, `grantUse`/`nextPolicy`, issuer-trust
  constraints, or any VC binding — `assigner` is carried "informational for single-node
  evaluation" only (`model.rs:51-53`). So the ODRL *delegation* and *trust* surface the trust
  graph needs is **designed-only / not-yet-built** here.
- The repo has a **VC-in-ZK estate** (`research/zk-signed-credential-representation-design.md`,
  `sparq-zk`/`sparq-zk-compose`) that proves SPARQL predicates over *issuer-signed* committed
  values. That is the cryptographic counterpart to the trust graph's "trusted-source-attested
  statement": it already reasons about *who signed a value*, in-circuit.

**Brief-premise check.** The brief's framing is **sound and not contradicted by the code**, with
two corrections worth surfacing to the maintainer: (a) sparq is **not** starting from zero on
source-scoped trust — the `acp:issuer` + `CreatorAgent`/`OwnerAgent` machinery is a *coarse,
single-axis trust graph already in production code*, and the design should be positioned as
**generalising** it (per-statement-type, multi-source, VC-attested), not inventing it; (b) the
"superset of ZKaps" claim needs a precise definition of *ZKaps* to be defensible — see §5; as
literally worded it is plausible for the **declaration/derivation** layer but **not** for the
*cryptographic capability-token* layer unless the trust graph also subsumes ZCAP-LD-style signed
delegation, which ODRL alone does not give.

---

## 1. The ODRL model, mapped onto the trust-graph's questions

ODRL 2.2 ([W3C Rec, Information Model](https://www.w3.org/TR/odrl-model/);
[Vocabulary & Expression](https://www.w3.org/TR/odrl-vocab/)) is a deontic policy language: a
**Policy** (`Set`/`Offer`/`Agreement`) contains **Rules** (`Permission`/`Prohibition`/`Duty`),
each binding an **Action** to an **Asset** (`target`), optionally to **Party**s
(`assigner`/`assignee`), narrowed by **Constraint**s `(leftOperand, operator, rightOperand)` and
(for permissions) gated by **Duty** obligations. The relevant question is not "can ODRL express
a permission" (yes) but **"can ODRL express *source-scoped trust* and *delegation*?"**

### 1.1 Trust dimension — ODRL has the *slot* but not the *construct*

ODRL constraints can carry identity/attribute dimensions: `recipient` (party receiving the
outcome), `purpose`, `systemDevice`, `partOf` party-collection membership, plus the open
`leftOperand` extension point (profiles add their own). **But ODRL core has no construct for
"this attribute value is *attested by issuer X* and I *trust* X for *this attribute*."** The
W3C ODRL spec defines no party-trust / credential-validation / issuer-provenance mechanism at
all — verified against the [vocabulary](https://www.w3.org/TR/odrl-vocab/): party identity is a
bare IRI; there is no signature, no issuer, no trust anchor. A constraint compares a
right-operand against *something in the evaluation request / state of the world*, but **ODRL is
silent on how that "something" earned its place in the state of the world** — which is exactly
the trust graph's whole subject.

The closest published attempt is the **Gaia-X ODRL-VC profile** ([`gitlab.com/gaia-x/lab/
policy-reasoning/odrl-vc-profile`](https://gitlab.com/gaia-x/lab/policy-reasoning/odrl-vc-profile)):
it adds `ovc:leftOperand` ("a way to refer to a W3C Verifiable Credential attribute to evaluate
against an `odrl:rightOperand`", a **JSONPath** into the VC, e.g.
`$.credentialSubject.gx:legalAddress.gx:countrySubdivisionCode`) and `ovc:credentialSubjectType`
(e.g. `gx:LegalParticipant`), reasoned over a graph DB with SPARQL. This is the **nearest prior
art to the trust graph's "VC `<Jesse> <age> 25` feeds the access decision"** — but, verified
from its README, it **has no issuer-trust construct**: "No example demonstrates trusting specific
issuers for particular claims … focuses on structural validation rather than issuer provenance."
So even the VC-aware ODRL profile *binds the claim to a credential type but not to a trusted
issuer*. The trust graph's defining feature — **per-(issuer, statement-type) trust** — is
unbuilt in the ODRL world.

### 1.2 Delegation — ODRL has `grantUse`/`nextPolicy`, but it is *unverified policy assertion*

ODRL **does** express delegation, via three vocabulary actions (verified from
[odrl-vocab](https://www.w3.org/TR/odrl-vocab/)):

- **`odrl:grantUse`** — "To grant the use of the Asset to third parties … enables the assignee
  to create policies for the use of the Asset for third parties." This is ODRL's delegation
  primitive: a permission with `action=grantUse` lets the assignee *issue downstream policies*.
- **`odrl:nextPolicy`** — "To grant the specified Policy to a third party for their use" — the
  downstream policy that `grantUse` authorises.
- **`odrl:derive` / `odrl:transform` / `odrl:install`** — derivative-asset actions; a derived
  asset "may have a next policy applied."

This is genuine delegation *semantics*, and it covers the trust-graph requirement that a party
delegate a capability. **But ODRL delegation is *declarative, not cryptographic*.** A `grantUse`
permission is a triple anyone can write; ODRL gives **no proof that the delegator actually held
the authority to delegate**, no signed delegation chain, no root-of-trust, no attenuation
verification. Delegation validity is whatever the *evaluator's reasoning* decides — which throws
the burden straight back onto "which sources do we trust for the `grantUse` statement," i.e. the
trust graph. **So ODRL's delegation is *exactly the kind of statement the trust graph must
adjudicate*, not a substitute for it.** Contrast **ZCAP-LD** (§3.2), which makes delegation a
signed, chain-verified object-capability.

### 1.3 The merge step — ODRL evaluation *is* the "attested statements ⋈ rules ⇒ access" pattern

The trust graph's core mechanic ("attested statements **merge** with `.acl` rules **via
reasoning** to derive access") is *structurally* ODRL evaluation. The emerging ODRL formal
semantics ([W3C ODRL Formal Semantics CG](https://w3c.github.io/odrl/formal-semantics/);
[*Evaluation and Comparison Semantics for ODRL*, arXiv 2509.05139](https://arxiv.org/html/2509.05139v1))
defines evaluation as a function of `{Policy, State-of-the-World, Evaluation-Request}` →
**Compliance Report**, where the *state of the world* is a relation of facts and a rule is a
boolean query over it. The arXiv work is explicit that this "enables straightforward
implementation in SQL or SPARQL," and the
[SolidLab ODRL-Evaluator](https://github.com/SolidLabResearch/ODRL-Evaluator) implements it as
**N3 rules on the EYE reasoner** over three RDF quad-lists `{Policy, Request, State-of-World}`.

The trust-graph insight is: **attested statements are *facts injected into the state of the
world*, gated by trust-graph rules that decide which attestations are admitted.** Then ordinary
ODRL/`.acl` rule evaluation derives access. sparq already runs this exact engine
(`sparq-reason` N3 for `materialize_*`), so the merge step is **in-family**, not novel
machinery. **What ODRL's state-of-the-world model does *not* specify is the admission gate** —
*how a fact gets into the state of the world and whether it should be believed*. That gate is
the trust graph. ODRL assumes the state of the world is given and trustworthy; the trust graph
is precisely the missing theory of *how the state of the world is assembled from attested
sources*.

---

## 2. ODRL's access-control mechanism, delegation story, trust model — the structured view

| Dimension | ODRL 2.2 core | ODRL-VC (Gaia-X profile) | OAC (ODRL Access-Control profile) |
|---|---|---|---|
| **AC mechanism** | Deontic Permission/Prohibition over Action×Asset×Party, narrowed by `(left,op,right)` constraints; deny-overrides; closed-world default (unlicensed ⇒ denied) | Same + VC-claim constraints via `ovc:leftOperand` (JSONPath) + `ovc:credentialSubjectType` | Maps ODRL actions→Solid access modes (`Use/Collect`→Read, `Store/MakeAvailable`→Write); DPV for GDPR constraints |
| **Delegation** | `grantUse`→`nextPolicy` (declarative; assignee may issue downstream policies); `derive` for derivative assets | Inherits ODRL; no extra delegation construct | Requirement-vs-Preference matching derives ACL/ACP grants; no signed delegation chain |
| **Trust model** | **None.** Party = bare IRI; no issuer, signature, or trust anchor; state-of-world assumed trustworthy | VC claims bound to **credential type**, **not issuer** — no per-issuer trust construct | Trust inherited from Solid auth (WebID/OIDC); issuer-trust not modelled |
| **Eval substrate** | Spec silent (formal semantics CG emerging) | Graph DB + SPARQL/JSONPath | N3/SHACL-class; derives authorizations + audit `Agreement` |
| **Statement-type granularity** | Per-Action/Asset; **not** per-issuer-per-attribute | Per VC-attribute, but issuer-agnostic | Per access-mode |

Sources for the row above: [odrl-model](https://www.w3.org/TR/odrl-model/),
[odrl-vocab](https://www.w3.org/TR/odrl-vocab/),
[ODRL formal semantics](https://w3c.github.io/odrl/formal-semantics/),
[arXiv 2509.05139](https://arxiv.org/html/2509.05139v1),
[ODRL-VC profile](https://gitlab.com/gaia-x/lab/policy-reasoning/odrl-vc-profile),
[OAC profile](https://besteves4.github.io/odrl-access-control-profile/oac.html),
[SolidLab ODRL-Evaluator](https://github.com/SolidLabResearch/ODRL-Evaluator).

---

## 3. Adjacent models the trust graph must reckon with (because ODRL alone is insufficient)

ODRL gives the *policy/derivation* layer but not the *trust/delegation-proof* layer. Two
adjacent standards fill exactly the gaps ODRL leaves, and the trust graph either subsumes or
composes with them:

### 3.1 W3C Verifiable Credentials + the issuer-trust question

[VC Data Model](https://www.w3.org/TR/vc-data-model-2.0/) gives the *attested statement*:
issuer asserts claims about a subject, cryptographically verifiable, with the four-role model
(issuer / holder / verifier / verifiable data registry). VC verification proves **integrity +
issuer-signature**, but the **"do I trust this issuer for this claim" decision is explicitly
out of VC scope** — it is delegated to a *trust framework* (EBSI trust model, schemes like
iSHARE). The trust graph **is that trust framework, expressed as RDF rules** — and uniquely
**per-statement-type**, which neither VC nor the surveyed ODRL-VC profile expresses. This is the
single most defensible novelty: *RDF-native, reasoning-evaluated, per-(issuer, predicate) trust
admission of VC claims into the state of the world that `.acl` rules then consume.*

### 3.2 ZCAP-LD (Authorization Capabilities for Linked Data) — the delegation-proof gap

[ZCAP-LD v0.3](https://w3c-ccg.github.io/zcap-spec/) is the object-capability model the trust
graph's "capability delegation" requirement most directly maps to, and it is where the
"superset of ZKaps" claim must be tested. Verified from the spec:

- **Chain model.** A *root capability* (`invocationTarget` = the resource; `controller` = the
  resource's own key) is the root of trust — *the target authorises itself*. Delegated
  capabilities carry `parentCapability` and a `proof.capabilityChain` (root referenced by ID,
  immediate parent embedded). Verification walks root→leaf, validating at each hop that the
  signer is in the currently-authorised set.
- **Attenuation via caveats.** A delegated zcap MUST be **no less restrictive** than its parent:
  `expires` not later than parent, `allowedAction` a subset, `invocationTarget` only narrowed
  (URL-suffix attenuation). This is *monotone capability attenuation* — a property ODRL's
  declarative `grantUse` does **not** enforce.
- **Trust model = authority by possession.** "What matters is holding valid cryptographic keys,
  not who the key-holder is." **There is *no per-claim per-issuer trust notion*** — trust is
  entirely chain-based from the resource's own root key.
- **VC vs ZCAP-LD split.** The spec is explicit: use VCs for the *initial authority decision*
  (correlation/reasoning), use ZCAP-LD as the *mechanism to exercise authority*. They are
  complementary, not interchangeable.

**Why this matters for the trust graph.** The trust graph wants *both* halves ZCAP-LD separates:
the VC-driven *reasoning* decision (which it does natively, per §3.1) **and** a delegation
mechanism. ODRL's delegation is unverified; ZCAP-LD's is signed-and-attenuated but **rooted in
the resource's own key, not in a per-issuer trust graph**. The trust-graph opportunity is a
**hybrid**: express the delegation *chain* and its *attenuation* as RDF statements whose
*admission* is gated by trust-graph rules and whose *integrity* is backed by VC/Data-Integrity
proofs — i.e. **make ZCAP-LD-style attenuated delegation a first-class set of trusted statements
in the same reasoning closure that consumes attested attributes.** No surveyed system unifies
"per-issuer attribute trust" + "attenuated capability delegation" in one RDF-reasoned model.
(`ssi-zcap-ld` exists as a Rust crate — [lib.rs/crates/ssi-zcap-ld](https://lib.rs/crates/ssi-zcap-ld)
— relevant if a future bead needs a verified-delegation building block; flag for the pre-add
supply-chain screen if ever pulled in.)

---

## 4. How this domain MAPS TO / informs the trust-graph design

1. **Adopt ODRL's evaluation frame, replace its trust silence.** Use the ODRL formal-semantics
   `{Policy, State-of-the-World, Request} → Report` shape (arXiv 2509.05139 / SolidLab
   evaluator) as the *evaluation contract*, but prepend a **trust-admission stratum**: an N3
   rule stratum that, given attested statements (VC-derived quads tagged with issuer) and the
   trust graph (per-(issuer, predicate) trust edges), emits the *admitted* facts that populate
   the state of the world. This slots directly in front of sparq's existing `materialize_*` N3
   strata — the trust graph is **a new first stratum**, not a new engine.
2. **Generalise the shipped `acp:issuer` axis from identity to statement-type.** `acp:issuer`
   already trusts an IdP *for the identity assertion*. The trust graph extends the same matcher
   shape to `trustsIssuerForPredicate(issuer, predicate)` (or a richer pattern over statement
   *shape*, not just predicate). This is an incremental, in-family generalisation of code that
   exists (`authindex.rs`, `acp-a.n3`), which de-risks the proposal and gives the WGs a concrete
   migration path from ACP's issuer dimension.
3. **Model attested statements as reified, issuer-tagged quads.** A VC claim
   `<Jesse> <age> 25` enters as a quad in a per-issuer named graph (or with an issuer
   annotation), exactly mirroring how `sparq-solid` stores ACL docs as named graphs and supplies
   `CreatorAgent` facts over a trusted channel. The trust-admission stratum reads the issuer tag;
   `.acl` rules read only the *admitted* triple — clean separation of *who said it* from *what
   was said*.
4. **Use ODRL `grantUse`/`nextPolicy` as the *vocabulary* for delegation, ZCAP-LD attenuation as
   the *discipline*.** Express delegated capabilities in ODRL terms (so the design speaks the
   data-spaces dialect), but borrow ZCAP-LD's monotone-attenuation and chain-to-root invariants
   as **trust-graph rules** that the reasoner enforces (delegated `allowedAction ⊆ parent`,
   `expires ≤ parent`, target only narrowed). Back chain integrity with VC/Data-Integrity proofs
   rather than ODRL's bare assertion.
5. **Human and AI agents are the same Party machinery.** ODRL `Party`/`assignee` and `acp:client`
   already distinguish *agent* (WebID) from *application/client*. An AI agent is modelled as a
   delegated capability whose `assignee` is the agent's identifier and whose attenuation
   (action/purpose/time/count constraints — all already in `sparq-policy`'s constraint model)
   bounds it. The count-enforcement store in `sparq-policy` is directly reusable for "AI agent
   may exercise this delegation N times." No new principal model is needed — only the
   delegation-chain admission of (3)/(4).
6. **Composition with the ZK estate is *optional and downstream*, not on the critical path.**
   The trust graph's *declaration + derivation* is pure reasoning and buildable today on
   `sparq-reason`/`sparq-solid`/`sparq-policy`. The ZK-attested variant (prove a derived
   `canAccess` without revealing the underlying VC value) maps to the existing
   `zk-signed-credential-representation-design.md` direction, but **inherits its NOT-yet-sound
   status** — keep it as a clearly-labelled future phase, never a v1 claim.

---

## 5. GAPS the trust graph must address (what ODRL leaves unsolved)

- **G1 — No per-issuer-per-statement-type trust construct anywhere in ODRL (or its VC profile).**
  This is *the* gap and *the* novelty. ODRL constraints can mention attributes; neither core
  ODRL nor the Gaia-X ODRL-VC profile can say "trust issuer X for predicate P but not Q." The
  trust graph must define this vocabulary and its reasoning semantics from scratch (informed by,
  but not provided by, ODRL).
- **G2 — No theory of how the state of the world is *assembled from attested sources*.** ODRL
  evaluation *assumes* a trustworthy state of the world. The trust graph is the missing
  *admission* layer. Must specify: conflict resolution when two trusted issuers attest
  contradictory values; non-monotonicity (revocation/expiry of a VC withdrawing a fact);
  fail-closed default (an un-admitted attestation must not leak into `.acl` derivation).
- **G3 — Delegation in ODRL is unverified assertion.** `grantUse`/`nextPolicy` carry no proof of
  delegator authority, no attenuation enforcement, no chain-to-root. The trust graph must add
  ZCAP-LD-style discipline (signed chain, monotone attenuation) as reasoning rules — ODRL gives
  the words, not the guarantees.
- **G4 — ODRL has no malformed/conflicting-policy *safety* default for trust.** ODRL conflict
  semantics (deny-overrides default) cover permission/prohibition conflict but say nothing about
  *trust* conflict (issuer-A-trusted-for-P vs an explicit distrust). The trust graph must define
  trust-conflict resolution and ensure it composes with `sparq-solid`'s fail-closed `D4`.
- **G5 — "Superset of ZKaps" needs a precise, defensible scoping.** *ZKaps* (zero-knowledge
  attribute-based credentials / capability tokens — the BBS+/AnonCreds lineage of unlinkable,
  selective-disclosure capability proofs) bundle (a) an *attested attribute*, (b) a *predicate
  proof* over it, and (c) optionally an *unlinkable/anonymous* presentation. The trust graph
  **cleanly supersets (a)+(b) at the declaration/derivation layer** (attested statement + rule →
  access is strictly more expressive than a fixed predicate token, because the rule is arbitrary
  N3). It supersets the *cryptographic* presentation (c) **only if** wired to the ZK estate —
  which is NOT-yet-sound-for-production (`sq-qhy4`). **Recommendation: claim "superset of ZKaps
  *at the policy/derivation layer*; the unlinkable-presentation property is provided by the
  (pending-external-audit) ZK composition, not by the trust graph itself."** As bare-worded the
  claim over-reaches on the crypto half; scoped, it is honest and still strong.
- **G6 — No standard wire/discovery for trust statements.** ODRL/VC/ZCAP-LD each have their own
  serialisation; the trust graph needs a discovery story (where does a server publish "I trust
  these issuers for these predicates"? how does a delegate present its chain?). This is a
  standards-track design question, not something ODRL answers.

---

## 6. Recommendation

**Adopt ODRL as the *policy-derivation vocabulary and evaluation frame*, but treat the trust
graph as a *new trust-admission layer ODRL has never had*, generalising sparq's already-shipped
`acp:issuer` / provenance-fact machinery.** Concretely: position the trust graph to the WGs as
**"the missing admission theory in front of the ODRL/`.acl` state of the world"** — VC claims
in, per-(issuer, statement-type) trust rules adjudicate, ODRL/`.acl` rules derive access out —
with ZCAP-LD-discipline delegation as trusted statements in the same closure, and the ZK estate
as an **optional, clearly-caveated** unlinkable-presentation back end (never a v1 soundness
claim). Scope the "superset of ZKaps" claim to the policy/derivation layer per **G5**.

The build is **incremental on what exists**: the engine (N3 reasoning + `DatasetView`
restriction), the principal model (`Session{agent,client,issuer}`), the ODRL types
(`sparq-policy`), and the attested-fact channel (`materialize_with_provenance`) are all in
place. The trust graph is a new *first stratum* + a delegation-admission ruleset + a trust
vocabulary — not a new engine.

---

## 7. Phased plan (each phase = a future bead for the orchestrator)

1. **Trust vocabulary + state-of-world admission stratum (design + RFC).** Define the
   `trustsIssuerForStatement` vocabulary (per-issuer, per-predicate / per-statement-shape) and
   the N3 admission stratum that emits *admitted* issuer-tagged facts into the state of the
   world; specify fail-closed default and the VC→quad reification convention. Doc + draft N3,
   no enforcement yet. *(Addresses G1, G2, map-to §4.1/§4.3.)*
2. **Generalise `acp:issuer` → per-statement-type trust in `sparq-solid` rules.** Extend the
   shipped `acp-a/b.n3` issuer dimension from identity-issuer to statement-type-scoped trust,
   reusing the AuthIndex matcher shape; TDD against allow/deny-by-(issuer,predicate) fixtures.
   *(Builds on shipped sq-3jtd.6; map-to §4.2.)*
3. **Attested-statement merge with `.acl`/ODRL derivation.** Wire admitted facts as inputs to
   the existing `materialize_*` closure so a rule like `{?x age ?y. FILTER(?y>18)} =>
   {?x canAccess ?r}` derives access from a trusted-issuer-attested `<x> <age>`; conflict +
   revocation/expiry (non-monotonic) handling. *(Addresses G2, G4; map-to §4.1/§4.3.)*
4. **Delegation admission: ODRL `grantUse`/`nextPolicy` + ZCAP-LD attenuation as rules.** Model
   delegated capabilities as trusted statements; enforce monotone attenuation
   (`allowedAction ⊆ parent`, `expires ≤ parent`, target-narrowing) and chain-to-root in N3;
   integrity via VC/Data-Integrity proof references. Cover human + AI-agent assignees; reuse the
   `sparq-policy` count store for bounded exercise. *(Addresses G3; map-to §4.4/§4.5.)*
5. **Conflict, revocation, and trust-graph self-governance.** Trust-conflict resolution
   (trusted-vs-distrusted issuer), VC revocation/expiry withdrawing admitted facts, and *who may
   edit the trust graph itself* (meta-trust) — all fail-closed and composing with `D4`.
   *(Addresses G2/G4.)*
6. **Standards-track write-up + WG proposal (LWS/Solid).** Consolidate 1–5 into the proposal,
   with the **scoped** ZKaps-superset claim (G5) and a discovery/wire story (G6); position vs
   VC trust frameworks, ZCAP-LD, and the Gaia-X ODRL-VC profile. *(Addresses G5, G6.)*
7. **(Optional, gated) ZK-attested trust-graph derivation.** Prove a derived `canAccess` without
   revealing the underlying VC value, via the existing signed-credential ZK direction —
   **explicitly gated on the open external ZK soundness sign-off (`sq-qhy4`)**; never a v1 claim.
   *(Map-to §4.6; the unlinkable-presentation half of G5.)*

---

## 8. Open questions for @jeswr (genuinely need the maintainer)

- **Q1 — ZKaps definition.** Which "ZKaps" do you mean precisely (the anonymous-credential /
  BBS+ "ZK attribute-based access tokens" lineage, or a specific paper/system)? The
  superset-claim scoping in **G5** depends on this; please pin it before the WG write-up.
- **Q2 — Trust granularity.** Is per-*predicate* trust enough, or do you need per-*statement-shape*
  (e.g. trust gov.uk for `<age>` *only when subject = the authenticating WebID*)? This changes
  the admission-stratum rule complexity materially.
- **Q3 — Delegation backing.** Do you want delegation chains cryptographically verified
  (ZCAP-LD / Data-Integrity proofs) in v1, or declarative-and-trust-graph-adjudicated first,
  crypto later? Phase 4 can go either way.
- **Q4 — Relationship to the existing usage-control framing.** Should the trust graph and the
  `research/feature-research-odrl-policy.md` usage-control/disclosure line be one unified
  `sparq-policy` story, or kept as distinct proposals to the WGs?

---

## 9. Citations

- ODRL Information Model 2.2 — https://www.w3.org/TR/odrl-model/
- ODRL Vocabulary & Expression 2.2 (grantUse, nextPolicy, derive, recipient, purpose) — https://www.w3.org/TR/odrl-vocab/
- ODRL Formal Semantics (W3C CG) — https://w3c.github.io/odrl/formal-semantics/
- Evaluation and Comparison Semantics for ODRL (arXiv 2509.05139) — https://arxiv.org/html/2509.05139v1
- SolidLab ODRL-Evaluator (N3/EYE implementation) — https://github.com/SolidLabResearch/ODRL-Evaluator
- ODRL-VC profile (Gaia-X, VC-claim constraints; no issuer-trust construct) — https://gitlab.com/gaia-x/lab/policy-reasoning/odrl-vc-profile
- OAC — ODRL Profile for Access Control — https://besteves4.github.io/odrl-access-control-profile/oac.html
- W3C Verifiable Credentials Data Model 2.0 — https://www.w3.org/TR/vc-data-model-2.0/
- VC Data Integrity 1.1 — https://w3c.github.io/vc-data-integrity/
- ZCAP-LD — Authorization Capabilities for Linked Data v0.3 (chain, caveats, authority-by-possession) — https://w3c-ccg.github.io/zcap-spec/
- ssi-zcap-ld (Rust) — https://lib.rs/crates/ssi-zcap-ld
- Improving ODRL 2.2: current limitations and theoretical solutions (CEUR Vol-3977/OPAL2025-6) — https://ceur-ws.org/Vol-3977/OPAL2025-6.pdf
- A Formal Foundation for ODRL (Pucella & Weissman, arXiv cs/0601085) — https://arxiv.org/pdf/cs/0601085
- sparq internal (read for this doc): `crates/sparq-solid/src/{lib,authindex}.rs`,
  `crates/sparq-solid/rules/acp-a.n3` / `acp-b.n3`, `crates/sparq-policy/src/model.rs`,
  `research/solid-access-control-design.md`, `research/zk-signed-credential-representation-design.md`,
  `research/feature-research-odrl-policy.md`.
</content>
</invoke>
