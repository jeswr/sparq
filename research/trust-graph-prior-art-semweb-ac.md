# Prior art for the "trust graph": Semantic-Web access control

> Research record for the **trust-graph** design (maintainer @jeswr, intended for the LWS +
> Solid WGs). Domain: **Semantic-Web access control** — WAC, ACP (incl. the `acp:vc`
> matcher the design calls out), Solid `.acl` inheritance, WebID / Solid-OIDC, and the older
> rule-based AC literature (AIR, Rei, KAoS, Protune/PeerTrust, Shi3ld). This doc surveys
> that prior art, maps it onto the trust-graph design, and states the gaps the design must
> close.
> Model: Opus 4.8 (Fable unavailable — flag for re-review when Fable returns). [OPUS-4.8]

## 0. The design under study (restated, so the gaps are concrete)

A **trust graph** is the set of statements/rules a storage server (or a single resource)
uses to decide **which sources it trusts for which access-control statements** —
*per-source, per-statement-type*. Trusted-source-attested statements (e.g. a
government-issued VC `<Jesse> <age> 25`) **merge with `.acl`-style rules** (e.g.
`{ ?x <age> ?y . FILTER(?y > 18) } => { ?x <canAccess> <r> }`) **via reasoning** to derive
an access decision. It must support **capability delegation** for human *and* AI agents, and
is claimed to be a **superset of ZKaps** (zero-knowledge access passes / capabilities).

Two things make this *not* just "WAC with credentials":

1. **The trust dimension is first-class and statement-typed.** Today's Solid AC trusts the
   *resource owner's* policy document and (in ACP) trusts *that a VC of a named type was
   presented*. It does **not** let a policy say "I trust issuer `gov.example` for
   `<…> age "…"` triples but not for `<…> creditScore "…"`." The trust graph makes
   *(source, statement-shape) → trusted?* an explicit, reasoned-over relation.
2. **Decisions are derived by reasoning over credential *contents*, not by matching
   credential *shapes*.** This is the maintainer's "exact-credential-shape hack" critique of
   ACP's `acp:vc` (verified normatively below): ACP matches the *VC type IRI only*, never the
   claims inside.

## 1. Verification against sparq's own reality (do not take the brief on faith)

The brief is a *design* proposal; sparq already ships a substantial slice of the *enforcement*
machinery it would sit on. Verified by reading the code on `origin/main`:

- **`crates/sparq-solid`** implements WAC **and** ACP as **N3 rule strata** that materialize a
  per-principal authorization view (`<urn:sparq:auth>`), enforced by query rewriting / a
  zero-copy `DatasetView`. Design + measured baseline: `research/solid-access-control-design.md`
  (status: **shipped**). ACP coverage verified in `rules/acp-{a,b,c}.n3` and `tests/acp.rs`:
  `agent` / `client` / **`issuer`** (sq-3jtd.6) / `CreatorAgent` / `OwnerAgent` (sq-3jtd.5),
  `allOf`/`anyOf`/`noneOf`, allow/deny with normative deny-overrides.
- **`acp:vc` is NOT implemented.** Confirmed: `solid-access-control-design.md` §3.6 lists it
  under "Not covered … `acp:vc`, `acl:accessToClass`, custom ACP modes", and the
  `sparq-solid/README` support line enumerates agent/client/issuer/Creator/Owner but **not**
  `acp:vc`. So the exact matcher the trust-graph design critiques is *absent* in sparq today —
  the design is not displacing an implemented feature; it is choosing what to build *instead of*
  the `acp:vc` shape-match. **This is the right premise.**
- **The "trusted-source / attested-statement" half already has crypto scaffolding** in
  `crates/sparq-zk-compose` (`issuer.rs`, `derivation.rs`, `manifest.rs`, `revocation.rs`):
  proving a result derives **only** from sources whose signing key is in a trusted **issuer
  set** (set-membership), with revocation. That is exactly the *trust-of-issuer* primitive the
  trust graph needs — but it is in the **ZK estate**, whose v1 verifier is **remediated +
  internally re-audited with external accredited-cryptographer sign-off still PENDING**
  (sq-qhy4), and so cannot be presented as a proven guarantee.
- **`crates/sparq-policy` is ODRL-only** (usage control: permission/prohibition/duty,
  purpose/recipient/time/`count`). It does **no** credential-content reasoning and is **not** a
  trust graph. (`research/feature-research-odrl-policy.md` is the companion usage-control study.)
- **No capability/ZCAP/delegation primitive exists** anywhere in the workspace
  (`grep` for `zcap`/`capability`/`delegat` finds only unrelated "capability-aware pushdown" in
  the federation client). Delegation for the trust graph is **greenfield**.

Correction to a likely-implicit premise: the design's "superset of ZKaps" claim leans on the ZK
estate, which is **not yet externally signed off**. The *design* can be a superset on paper;
any *deployed* claim of ZKap-grade privacy must stay caveated until sq-qhy4 clears.

## 2. The key models / specs in this domain

### 2.1 Web Access Control (WAC) — the deployed Solid baseline

- **What it offers.** A small RDF vocabulary (`acl:`) attaching `acl:Authorization`s to
  resources: subject (`acl:agent` WebID / `acl:agentGroup` / `acl:agentClass`
  {`foaf:Agent`=public, `acl:AuthenticatedAgent`}), object (`acl:accessTo` / `acl:default`),
  modes (`Read`/`Write`/`Append`/`Control`), and a coarse `acl:origin` app dimension. Inheritance
  is **nearest-ancestor** (the closest container `.acl` shadows higher ones).
- **AC mechanism.** Per-request: discover the effective `.acl` (own, else nearest ancestor),
  match (agent ∧ origin ∧ mode); fail-closed. Pure RDF triples; no rules, no FILTER, no
  arithmetic.
- **Delegation.** Only `acl:Control` (read/write of the **`.acl` resource itself**) — i.e. the
  ability to *re-write the policy*. There is **no scoped, attenuating delegation** of a subset
  of one's own access to a third party; "delegation" = "make you a co-administrator of the ACL".
- **Trust model.** Single root of trust: **whoever can write the `.acl`** (the resource owner /
  a Control holder). Identity is a WebID; the *binding* of request→WebID is delegated entirely to
  the auth layer (WebID-TLS historically, Solid-OIDC now). WAC trusts **no external issuer** and
  reasons over **no credential** — a subject is a literal WebID on the ACL.
- **Limitations (for the trust graph).** No attributes/credentials at all; no condition
  language (can't say "age > 18"); origin is the only app dimension and it is a bare string;
  no delegation-with-attenuation. WAC is the *floor* the trust graph must subsume as its
  degenerate "trust the owner's literal grant" case.
- Spec: <https://solidproject.org/TR/wac>.

### 2.2 Access Control Policy (ACP) — and the `acp:vc` "exact-shape" hack

- **What it offers.** A richer model: each resource has an **Access Control Resource (ACR)**
  holding **Access Controls** → **Policies** (`acp:allow`/`acp:deny` of modes) → **Matchers**
  combined via `acp:allOf`/`anyOf`/`noneOf`. Inheritance is **cumulative over all ancestors**
  (`acp:memberAccessControl` is transitive). Matchers test a **Context** on attributes
  `acp:agent`, `acp:client`, `acp:issuer`, **`acp:vc`** (same-attribute = OR, cross-attribute =
  AND). Normative deny-overrides.
- **AC mechanism.** Per-request evaluation of effective policies against the request Context;
  a satisfied `allow` grants a mode iff no satisfied `deny` denies it. ACP **does** model the
  user/app pair natively (`acp:client`) and the IdP (`acp:issuer`).
- **The `acp:vc` matcher — verified normative semantics.** The ACP TR defines `acp:vc` as "a set
  of types of Verifiable Credentials (VC), at least one of which MUST match the Context"; a VC in
  the Context "MUST be a valid VC presented as part of the resource access request." Matching is
  **type-IRI inclusion only** — `if (context.vcs.includes(vc)) …`. **ACP performs no semantic
  reasoning over credential *contents*** (no examining claims, no conditional logic on attribute
  values). This is precisely the maintainer's "exact-credential-shape hack": ACP can require
  *"present a VC of type `:AgeCredential`"* but **cannot** express *"…whose `age` value is > 18"*
  — that logic must live outside ACP entirely. *(Source: ACP TR, <https://solidproject.org/TR/acp>,
  fetched 2026-06-20.)*
- **Delegation.** Like WAC, "delegation" is write-access to the ACR (administer the policy).
  No attenuating capability chain.
- **Trust model.** Trusts the ACR writer (owner). `acp:issuer` lets a policy bind to *which IdP
  asserted the WebID* — a coarse, **identity-level** trust dimension (one trusted issuer per
  WebID assertion), **not** a *(issuer, statement-type)* trust relation. `acp:vc` trusts "some
  valid VC of this type was presented" but the validity/issuer-trust check is out of ACP's scope.
- **Limitations (for the trust graph).** The headline one is **no content reasoning** — `acp:vc`
  is a shape gate, not a claim evaluator. Combinatorial matchers are boolean over fixed
  attributes; there is no rule language, no `age > 18`, no joining a credential's subject to the
  requester. This is the gap the trust graph's *reasoning-over-attested-statements* fills.
- Spec: <https://solidproject.org/TR/acp>. sparq's faithful (minus `acp:vc`) implementation:
  `research/solid-access-control-design.md`.

### 2.3 WebID-TLS and Solid-OIDC — the identity binding both rely on

- **What they offer.** The *authentication* layer that turns an HTTP request into a trusted
  WebID, which WAC/ACP then authorize. **WebID-TLS** (legacy) binds a client TLS cert's public
  key to a WebID profile document. **Solid-OIDC** (current) = OIDC Authorization-Code + PKCE with
  a **DPoP-bound** ID/access token; the `webid` scope yields a token whose payload carries
  `webid`, `iss`, and `aud` (incl. `azp` + the literal `solid`); the **client** is itself
  identified by a dereferenceable **Client ID Document** (a `solid:oidcRegistration`).
- **AC mechanism.** None directly — they feed *identity* (the WebID), *issuer* (`iss`, the OIDC
  IdP), and *client* (the `azp`/Client ID) into the AC layer. ACP's `acp:agent`/`acp:client`/
  `acp:issuer` map 1:1 onto these.
- **Delegation.** OIDC has no native attenuating delegation; **UMA 2.0** (below) is the
  delegation profile layered on OAuth.
- **Trust model.** Trust the IdP that signed the ID token (and TLS for transport). The verifier
  trusts `iss` to have authenticated the WebID. This is the **only** cryptographic trust root in
  deployed Solid AC — and it is *identity*, not *claims*.
- **Limitation (for the trust graph).** Solid-OIDC vouches for *"this requester is WebID W,
  per IdP I"* — a single attested statement (`I says: requester = W`). The trust graph
  generalizes this to *arbitrary* attested statements from *arbitrary* issuers, each trusted only
  for the statement-types the trust graph admits. Solid-OIDC is the **n=1 special case**.
- Specs: <https://solid.github.io/solid-oidc/>, <https://github.com/solid/webid-oidc-spec>.

### 2.4 AIR (Accountability In RDF) — N3 rules + tracked justifications

- **What it offers.** An N3/cwm-based production-rule language (MIT DIG) for **policy reasoning
  with explanations**: rules (`air:pattern` → `air:assert`), **nested rule activation**
  (a rule's action can activate another rule), `air:alt` alternatives, and — the differentiator —
  **auditable trace-based justifications** as *RDF data* (so a justification can itself be
  reasoned over). Supports closed-world / scoped negation.
- **AC mechanism.** Compliance reasoning: derive `air:compliant-with` / `air:non-compliant-with`
  with a justification chain, rather than a bare allow/deny. Designed for *accountable* (after-
  the-fact auditable) privacy/usage policies.
- **Delegation.** No first-class delegation primitive; delegation would be modeled as further
  rules/facts.
- **Trust model.** Trusts the rule/fact authors; AIR's contribution is making *why* a decision
  was reached inspectable and re-reasonable, not *who* is trusted for *what*.
- **Limitations (for the trust graph).** No credential/issuer trust model and no delegation;
  but its **justification-as-RDF** idea is directly relevant — the trust graph's derivations
  ("access granted **because** issuer I attested age=25 **and** rule R fired") want exactly
  AIR-style provenance, which sparq already part-owns (`sparq-prov` PROV-O lineage for reasoner
  materialization, sq-m3i0).
- Source: MIT DIG AIR, <https://dig.csail.mit.edu/2009/AIR/>; survey context in Kirrane et al.

### 2.5 Rei / KAoS / Ponder / Protune+PeerTrust — the classic SW policy languages

From Tonti et al.'s comparison and the Kirrane survey:

- **Rei** (UMBC): RDF/OWL + logic-like rules; **deontic** concepts (right/prohibition/
  obligation/dispensation) plus **speech acts** — `delegate`, `revoke`, `request`, `cancel` —
  so **delegation is first-class** and runtime conflict resolution uses **meta-policies**
  (priority/precedence). Trust = the policy author; no credential-issuer model.
- **KAoS** (IHMC): OWL-(DL) policies (positive/negative authorizations + obligations),
  *design-time* deductive classification of policies; strong on consistency checking, weaker on
  runtime conditions; delegation supported via policy but DL-bound.
- **Ponder** (Imperial): not Semantic-Web (object-oriented), but the deontic + **role/delegation**
  reference model the SW languages were compared against.
- **Protune** (REWERSE; extends **PSPL** + **PeerTrust**): **the deepest trust model in this
  domain.** Rule/meta-rule Horn clauses with **deductive *and* abductive** reasoning;
  crucially it distinguishes **credentials** (*"certified by a third party"*) from
  **declarations** (*"not certified"*) as **provisional predicates**, and supports **automated
  trust negotiation** — iterative, bilateral credential disclosure driven by policies, with
  **policy explanations**. Ontologies associate *evidences* with conditions to drive negotiation.
- **AC mechanism (common).** Rule evaluation over a description of the requester/context; some
  (Protune/PeerTrust) negotiate the evidence interactively.
- **Trust model (the lesson).** Protune/PeerTrust already separate *"who certified this
  attribute"* from the attribute itself — the **conceptual ancestor of the trust graph's
  (source, statement) trust relation** — but they predate VCs/DIDs and do it with bespoke
  credential formats and **no cryptographic selective disclosure**.
- **Limitations (for the trust graph).** No standard credential layer (pre-VC), no
  cryptographic attestation/ZK, no modern (WebID/OIDC) identity binding; Rei/KAoS delegation is
  ACL-administrative or DL-bound, not object-capability attenuation. But Rei's **delegation
  speech acts + meta-policy conflict resolution** and Protune's **certified-vs-declared +
  negotiation** are the two ideas the trust graph should consciously re-adopt in modern dress.
- Sources: Tonti, Bradshaw et al., *Semantic Web Languages for Policy Representation and
  Reasoning: A Comparison of KAoS, Rei, and Ponder* (ISWC 2003,
  <https://link.springer.com/chapter/10.1007/978-3-540-39718-2_27>); Kirrane et al. survey §3.

### 2.6 Shi3ld — context-aware, triple-level, SPARQL-ASK enforcement

- **What it offers.** A pluggable AC filter for *generic* SPARQL endpoints/triple stores using
  **only** SW languages: named graphs hold policies; **Access Conditions are SPARQL `ASK`
  queries** evaluated against a per-request **context** graph (device/time/location/identity).
  Protection down to **triple level**.
- **AC mechanism.** For each protected named graph, run its policy's `ASK`; if it succeeds for
  the request context, expose the graph. No engine modification required.
- **Delegation.** None first-class.
- **Trust model.** Trusts the policy author and a trusted context provider; no issuer/credential
  trust model.
- **Limitations / lesson.** Shi3ld is the canonical demonstration that **an AC condition can be
  *a query over an RDF context*** — directly analogous to the trust graph evaluating an `.acl`
  rule (`age > 18`) over the *merge* of attested statements + context. sparq's enforcement model
  (materialize a view, then restrict the dataset) is a more efficient evolution of Shi3ld's
  per-graph-`ASK` (the Kirrane survey calls this the query-rewriting vs annotation/materialization
  axis). The trust-graph delta over Shi3ld is the **cryptographically-attested, issuer-typed**
  source of the context facts.
- Sources: Costabello, Villata et al., *Shi3ld* / *Context-Aware Access Control for RDF Graph
  Stores* (ECAI 2012, <https://www-sop.inria.fr/members/Serena.Villata/Resources/ecai2012ac.pdf>).

### 2.7 The enforcement axis — query-rewriting vs annotation/materialization (Kirrane survey)

- **What it offers.** Kirrane, Mileo & Decker, *Access Control and the RDF: A Survey* (Semantic
  Web J. 8(2), 2017) is the field map: it categorizes RDF AC by **specification** (which policy
  language), **enforcement** (query-rewriting — inject graph/FILTER restrictions into the algebra;
  vs annotation/materialization — pre-label each triple/graph with its decision), implementation,
  and infrastructure, and explicitly treats **delegation/propagation of authorizations** and
  **trust negotiation** as requirements.
- **Lesson for the trust graph.** (i) sparq's design is the *hybrid* the survey points at —
  **materialize** the per-principal decision by reasoning, then **rewrite** to the authorized set
  (cheap rewrite because the policy logic ran at materialization). (ii) The survey flags
  **delegation + revocation algorithms** and **trust/provenance** as *open* — i.e. the trust
  graph is aimed at acknowledged gaps, not solved problems. (iii) It situates Protune's trust
  negotiation as the credentials-meet-policy frontier.
- Source: <https://www.semantic-web-journal.net/system/files/swj1280.pdf>.

### 2.8 ZCAP-LD — object-capability delegation for Linked Data (the delegation reference)

- **What it offers.** W3C-CCG *Authorization Capabilities for Linked Data*: authority encoded as
  a **chain of JSON-LD capability documents**. Root authority = the **target resource's
  `controller`**; each delegation adds a `parentCapability` link + a `capabilityDelegation`
  proof signed by an authorized controller; **caveats** (a.k.a. *attenuation*) are restrictions
  that are **inherited from all parents and may only narrow** (path/query-scoping, expiry, action
  limits). **Invocation** carries a `capabilityInvocation` proof; verification **traverses the
  chain root→leaf**, checking each proof and that no step **broadens** authority.
- **AC mechanism.** **Authority by possession**, not by identity — answers "do you hold a valid,
  un-revoked, sufficiently-un-attenuated capability?" rather than "who are you?". Resists
  ambient-authority and confused-deputy attacks (principle of least authority).
- **Delegation.** This *is* the delegation model — attenuating, chained, cryptographically
  verifiable, **the** mature SW answer to the design's "capability delegation for human AND AI
  agents" requirement. An AI sub-agent is just another controller key holding an attenuated child
  capability.
- **Trust model.** Root of trust = the target's controller (the resource owner) — same root as
  WAC/ACP, but delegation is **sovereign and offline-verifiable** (no central ACL re-write).
- **Limitations (for the trust graph).** ZCAP-LD is **identity/possession**-based: a caveat is a
  restriction on *use of the capability*, **not** a reasoned condition over the holder's
  *attributes/credentials*. It cannot say "this capability is valid only for holders the trust
  graph derives to be > 18". So ZCAP-LD and the trust graph are **complementary**: ZCAP-LD gives
  attenuating delegation; the trust graph gives attribute/credential-conditioned grants. The
  design's "superset" framing is most defensible as *"the trust graph + a ZCAP-style chain
  subsumes ZKaps"*, where the **caveat language is replaced by trust-graph rules over attested
  statements**.
- Spec: <https://w3c-ccg.github.io/zcap-spec/>.

### 2.9 UMA 2.0 — party-to-party delegation over OAuth

- **What it offers.** Kantara *User-Managed Access*: separates **resource owner** from
  **requesting party**, with an **Authorization Server** that runs **claims-gathering** (interact
  with the requesting party to collect the claims a policy needs) before issuing a token. Enables
  **asynchronous, cross-party delegation** decoupled from the owner being online.
- **AC mechanism.** Policy at the AS decides token issuance from gathered claims + a permission
  ticket; the RS enforces the token.
- **Delegation.** First-class party-to-party delegation (the OAuth-world analogue of the trust
  graph's human/AI-agent delegation), and **claims-gathering is the protocol hook** where a trust
  graph's required attested statements would be collected.
- **Trust model.** Trust the AS + the claim issuers it accepts. UMA is **agnostic about which
  claims/credentials** — it is the *protocol envelope*; the trust graph would supply the **policy
  + trust-of-issuer semantics** inside it.
- **Limitation (for the trust graph).** No semantic reasoning or issuer-per-statement-type trust;
  it standardizes the *flow*, not the *decision logic*.
- Sources: UMA 2.0 Grant (<https://docs.kantarainitiative.org/uma/wg/rec-oauth-uma-grant-2.0.html>);
  *From Access Control to Usage Control with UMA* (arXiv 2411.05622 / 2601.18761).

### 2.10 Policy + SSI + ZK — the closest construction to "ZKaps as a special case"

- **More, Ramacher, Alber, Herzl, *Extending Expressive Access Policies with Privacy Features***
  (2022). A **policy language whose credential conditions compile to SNARK circuits**: a
  domain expert writes a policy (TPL/Horn-style `accept` predicate over credential attributes);
  a **policy compiler** splits it into attributes to **reveal** vs statements to **prove in
  zero-knowledge**, emitting circuits so the user proves *statements over private attributes*
  (e.g. age ≥ threshold) **without revealing the attribute**, with the revealed parts bound to a
  commitment for consistency. Builds on CL-signatures / W3C verifiable presentations.
- **Why it matters most here.** This is the **published prior art closest to the trust-graph's
  ZKaps-superset claim**: it already (i) reasons over credential *contents* via a policy
  language, (ii) proves predicate conditions in ZK, (iii) ties revealed-vs-hidden to the policy.
  It is essentially "ZKaps driven by an expressive policy". The trust-graph deltas are: it uses
  **RDF/N3 + SPARQL** as the policy+data language (not bespoke TPL), it makes the **(issuer,
  statement-type) trust relation explicit and reasoned-over**, and it targets **Solid `.acl`
  merge semantics** — but More et al. is the strongest evidence the construction is *feasible*
  and the strongest yardstick to benchmark "superset of ZKaps" against. Honest caveat: sparq's
  own ZK path that would realize this (`sparq-zk`/`sparq-zk-compose`) is **internally re-audited
  but awaiting external accredited-cryptographer sign-off** (sq-qhy4) and `sparq-mpc` is
  semi-honest-only — so the *capability* is designed/prototyped, not a proven production guarantee.
- Source: arXiv 2212.02454.

## 3. How this domain maps onto the trust-graph design

| Trust-graph need | Closest prior art | What maps | What is missing |
|---|---|---|---|
| Owner's literal grant (degenerate case) | WAC `acl:Authorization` | trust-the-owner base case | attributes, conditions, delegation |
| Policy on user/app/IdP | ACP `acp:agent`/`client`/`issuer` | exact 3-dimension principal; **already in `sparq-solid`** | per-statement-type issuer trust; content reasoning |
| "Present a credential" | ACP `acp:vc` | the *shape* the design replaces | **content reasoning — the whole point** |
| Conditions over context/attributes (`age>18`) | Shi3ld `ASK`; Protune rules | AC-condition-as-query; rule over a context graph | cryptographic, issuer-typed source of facts |
| Certified-vs-declared facts | Protune/PeerTrust provisional predicates | the **(certified-by-third-party)** distinction | VCs/DIDs, ZK, modern identity |
| Reason-to-a-decision + audit | AIR justifications; `sparq-prov` PROV-O | derivation as RDF, re-reasonable | issuer trust; ZK-compatible provenance |
| Attenuating delegation (human + AI agent) | **ZCAP-LD** chains + caveats; UMA claims-gathering | object-capability delegation, offline-verifiable | caveats are use-restrictions, **not attribute conditions** |
| "Superset of ZKaps" | **More et al. 2022**; sparq `sparq-zk-compose` | policy→circuit, predicate proofs, issuer-set membership | RDF/N3 policy language; (issuer, statement-type) trust; **external ZK sign-off pending** |

The unifying picture: the trust graph is the **merge layer** that ACP's `acp:vc` deliberately
left out. ACP/WAC supply the *grant rules* (`{ ?x age ?y . FILTER(?y>18) } => canAccess`),
Solid-OIDC/VCs supply the *attested statements* (`I says: <Jesse> age 25`), Protune supplies the
*certified-vs-declared* discipline, ZCAP-LD/UMA supply *delegation*, AIR/PROV supply *justifiable
derivation*, and More et al./`sparq-zk-compose` supply the *ZK realization* of "prove the
derivation without revealing the facts". sparq already owns the enforcement substrate
(`sparq-solid` materialize→`DatasetView`) and the reasoner (`sparq-reason` N3 strata) — the trust
graph is a **new policy/trust layer over existing parts**, structurally just like the ODRL bridge
(`sparq-policy`) was.

## 4. Gaps this prior art leaves — what the trust graph must add

1. **Per-(source, statement-type) trust, reasoned-over.** Nothing in deployed SW AC expresses
   "issuer I is trusted for `age` triples but not `creditScore` triples." ACP's `acp:issuer` is
   identity-level (one IdP per WebID); Protune is certified-vs-declared but not *typed* per
   statement. The trust graph must define a **trust vocabulary** —
   `?source trusts:assertsFor (?predicate-shape | ?graph-pattern)` — and feed it into the same N3
   strata `sparq-solid` already runs. **Open question for @jeswr:** at what granularity is a
   "statement-type" — predicate IRI? a SHACL shape? a SPARQL graph pattern? This decides
   tractability.
2. **Content reasoning over credentials (the anti-`acp:vc`).** The merge of attested statements
   with `.acl` rules via reasoning is the core novelty; it requires the **attested statement's
   subject to bind to the requester** (a join the survey notes is hard) and the credential's
   claims to enter the reasoner as *trusted* facts — exactly the `sparq-solid` §2.4 smuggling
   boundary (only trusted channels may inject `solidx:`-style facts), now generalized to "only
   issuer-attested triples enter as facts, tagged with their issuer."
3. **Attenuating delegation for human *and* AI agents.** No SW AC system combines attribute-
   conditioned grants with **ZCAP-style attenuating chains**. The design must marry them: a
   delegated capability whose caveat is *itself a trust-graph rule* over the delegatee's attested
   attributes (so an AI sub-agent inherits a *narrowed, attribute-conditioned* slice). Revocation
   (which the survey flags as unsolved) must be designed in, not bolted on (`sparq-zk-compose`
   already has `revocation.rs` for the issuer-set side).
4. **Justifiable, ZK-compatible derivations.** AIR/PROV give re-reasonable justifications; the
   trust graph wants the *same* justification to be the public statement a ZK proof discharges
   ("granted because the derivation used only issuer-set-trusted facts"). This binds to
   `sparq-zk-compose`'s manifest — but **stays caveated** until external sign-off (sq-qhy4).
5. **The "superset of ZKaps" claim needs a precise reduction.** More et al. is the yardstick:
   the design must show ZKaps = the special case where the trust-graph rule is "holder is in
   set S" and the proof hides which member — and must **not** overstate it while the ZK estate's
   external audit is pending. Until then the claim is **designed-only**, not proven.
6. **Conflict resolution + fail-closed composition.** Rei/KAoS meta-policies and ACP's normative
   deny-overrides show conflict resolution is mandatory once rules + multiple issuers interact;
   the trust graph inherits `sparq-solid`'s fail-closed posture but must define precedence when
   two issuers attest contradictory statements (e.g. two `age` values).

## 5. Phased plan (each phase = a future bead)

Ordered; later phases depend on earlier. These are **proposed beads** for the orchestrator to
create — nothing here is implemented by this research doc.

1. **`trust-graph-vocab`** *(design-for-review)* — define the trust vocabulary and semantics:
   `(source, statement-shape) → trusted` at a chosen granularity (predicate IRI vs SHACL shape vs
   graph pattern — resolve the §4.1 open question with @jeswr), the attested-statement
   representation (reification / RDF-star quoted triple `<<…>>` per the maintainer's
   `queryable-credentials` model), and the fail-closed conflict-resolution rule. Deliverable: a
   `research/` spec + a vocabulary draft for the LWS/Solid WGs. **No code.**
2. **`trust-graph-merge-rules`** *(prototype)* — N3 rule strata (in the `sparq-solid` /
   `sparq-reason` family) that **merge issuer-attested triples with `.acl`/ACP grant rules** to
   derive `auth:*` grants, generalizing the §2.4 trusted-fact-injection boundary to
   "only issuer-tagged attested triples enter as facts." Reuses the existing materialize→
   `DatasetView` enforcement. Includes the subject-binds-to-requester join. **Gate: WAC/ACP
   suites still green; a new trust-graph fixture.**
3. **`trust-graph-issuer-trust`** — wire the *(issuer, statement-type)* trust relation to the
   existing **issuer-set / set-membership** machinery in `sparq-zk-compose` (`issuer.rs`,
   `revocation.rs`) **non-ZK first** (plaintext issuer-set check), so the trust dimension works
   before any ZK claim. Keep all ZK mentions caveated.
4. **`trust-graph-delegation`** *(design-for-review, then prototype)* — a **ZCAP-LD-style
   attenuating delegation** layer where a caveat may be a **trust-graph rule** over the
   delegatee's attested attributes; covers human and AI-agent sub-delegation; designs revocation
   from the start. Reconcile with UMA claims-gathering as the collection protocol. **Open
   question: own vocabulary vs adopt ZCAP-LD verbatim + caveat-as-rule extension.**
5. **`trust-graph-justification`** — emit AIR/PROV-style **RDF justifications** for each derived
   grant (reuse `sparq-prov`), structured so they can later serve as the public statement a ZK
   proof discharges. Auditable allow/deny with "because" chains.
6. **`trust-graph-zkaps-reduction`** *(research, gated on sq-qhy4)* — formalize and prototype the
   **ZKaps-as-special-case** reduction against the More et al. 2022 construction and
   `sparq-zk-compose`; benchmark. **Hard-blocked** on external accredited-cryptographer sign-off;
   until then the "superset of ZKaps" claim ships as **designed-only**, never as a proven
   property (privacy-claims gate).
7. **`trust-graph-conflict-and-negotiation`** *(design)* — meta-policy conflict resolution
   (Rei-style precedence; ACP deny-overrides as the base) for contradictory multi-issuer
   attestations, and optionally **Protune-style trust negotiation** (iterative credential
   request) as the interactive face of the design, mapped onto UMA claims-gathering.

## 6. Open questions that genuinely need the maintainer

- **Statement-type granularity** (§4.1 / phase 1): predicate IRI, SHACL shape, or SPARQL graph
  pattern? This is the single biggest tractability/expressiveness lever.
- **Delegation vocabulary**: extend **ZCAP-LD** with caveat-as-trust-graph-rule, or mint a
  native trust-graph delegation vocabulary? ZCAP-LD buys ecosystem alignment; native buys
  uniformity with the N3 rule layer.
- **Scope of the "superset of ZKaps" claim** for the standards-track doc: present as
  **designed-only** (recommended while sq-qhy4 is pending) or hold it back entirely until external
  sign-off? The privacy-claims CI gate will reject an unqualified version.
- **Where the trust graph lives** relative to ODRL: is *(issuer, statement-type)* trust an input
  to the *access* decision only (sit beside `sparq-solid`), or should it also gate ODRL *usage*
  duties (compose with `sparq-policy`)? The ODRL study (`feature-research-odrl-policy.md`)
  suggests a clean seam either way.

## 7. Recommendation

Build the trust graph as a **new policy/trust layer over the shipped `sparq-solid` enforcement
substrate and `sparq-reason` N3 strata** — structurally identical to how `sparq-policy` (ODRL)
was added — rather than as a new engine concept. Sequence: **vocabulary + merge semantics first
(phases 1–2, design-for-review + prototype), trust-of-issuer non-ZK next (phase 3), then
delegation (phase 4)**; treat the **ZKaps-superset reduction (phase 6) as research gated on the
pending external ZK sign-off** and keep every ZK/MPC mention caveated. The design's premise is
sound and aimed at acknowledged open problems (the Kirrane survey flags delegation, revocation,
and trust/provenance as unsolved); its single most novel and defensible contribution is the
**reasoned merge of issuer-attested statements with `.acl` rules** — the thing ACP's `acp:vc`
deliberately omits — with More et al. 2022 as the closest construction to benchmark against.

## 8. Citations

- WAC — <https://solidproject.org/TR/wac>
- ACP (incl. `acp:vc` normative semantics) — <https://solidproject.org/TR/acp>
- Solid-OIDC — <https://solid.github.io/solid-oidc/> ; WebID-OIDC — <https://github.com/solid/webid-oidc-spec>
- AIR (Accountability In RDF), MIT DIG — <https://dig.csail.mit.edu/2009/AIR/>
- Tonti, Bradshaw et al., *KAoS, Rei, and Ponder: a comparison* (ISWC 2003) —
  <https://link.springer.com/chapter/10.1007/978-3-540-39718-2_27>
- Protune / PeerTrust / PSPL — surveyed in Kirrane et al. (below) §3.2.2
- Costabello, Villata et al., *Shi3ld / Context-Aware Access Control for RDF Graph Stores* (ECAI
  2012) — <https://www-sop.inria.fr/members/Serena.Villata/Resources/ecai2012ac.pdf>
- Kirrane, Mileo, Decker, *Access Control and the RDF: A Survey* (Semantic Web J. 8(2), 2017) —
  <https://www.semantic-web-journal.net/system/files/swj1280.pdf>
- ZCAP-LD — *Authorization Capabilities for Linked Data* (W3C-CCG) — <https://w3c-ccg.github.io/zcap-spec/>
- UMA 2.0 Grant (Kantara) — <https://docs.kantarainitiative.org/uma/wg/rec-oauth-uma-grant-2.0.html> ;
  *From Access Control to Usage Control with UMA* — <https://arxiv.org/pdf/2411.05622>
- More, Ramacher, Alber, Herzl, *Extending Expressive Access Policies with Privacy Features* (2022)
  — <https://arxiv.org/pdf/2212.02454>
- J. Wright, *queryable-credentials* (trusted-issuer claims + ZKP-over-SPARQL/N3 model) —
  <https://github.com/jeswr/queryable-credentials>
- *Towards Provable Provenance and Privacy-Preserving …* (CEUR Vol-4085, paper 19) —
  <https://ceur-ws.org/Vol-4085/paper19.pdf>
- sparq internal: `research/solid-access-control-design.md`,
  `research/feature-research-odrl-policy.md`, `crates/sparq-solid/` (rules + tests),
  `crates/sparq-zk-compose/` (`issuer.rs`, `derivation.rs`, `revocation.rs`),
  `crates/sparq-policy/`, `research/mpc-zkp-research-and-architecture.md`.
