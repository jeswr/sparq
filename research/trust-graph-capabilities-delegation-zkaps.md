<!-- [OPUS-4.8] Prior-art research authored by Opus 4.8 (1M context) (Fable unavailable) — re-review when Fable returns. -->
# Prior art for the trust graph — domain: capabilities, delegation, and ZKaps

> 🤖 **SPARQ research agent.** Design-for-review prior-art record. NO production code lands
> here. This is one domain slice ("capabilities, delegation, and ZKaps") of the wider
> prior-art survey feeding @jeswr's **trust-graph** proposal to the LWS + Solid WGs. It maps
> the object-capability / delegated-authorization literature and the anonymous-credential /
> Zero-Knowledge-Access-Pass (ZKap) literature onto the trust-graph design, states what each
> gives precisely, and flags the gaps the trust graph must still close. Companion slices
> (verifiable credentials / DIDs; Solid WAC/ACP; reasoning/N3; ODRL policy) are out of scope
> except where they intersect this one.

<!-- separator -->

> **Honesty / scope (read first).** Claims here are graded **implemented-and-verified
> (in sparq) / specified-in-an-external-standard / designed-only / proposed / not-yet-sound**.
> The sparq **ZK/MPC estate is research-stage and NOT externally audited** (open beads
> `sq-qhy4` external accredited-cryptographer sign-off PENDING; MPC is **semi-honest only**).
> Nothing here is a soundness or production-privacy claim; ZK/ZKap mentions are caveated for
> the live privacy-claims gate. No performance numbers are asserted; the work-box is
> non-canonical. Every external claim traces to a cited spec/paper; uncertainties are marked
> **[UNCERTAIN]**.

<!-- separator -->

> **Correction to the brief's premise (load-bearing).** The brief says the trust graph "is
> claimed to be a **superset of ZKaps**". Read literally that claim is **false and should not
> be made unqualified** in the standards doc. A trust graph in the maintainer's framing is an
> **authorization-decision layer** (which sources are trusted for which access-control
> statements, merged with `.acl` rules by reasoning). ZKaps / anonymous credentials are a
> **privacy/unlinkability presentation layer** (how a holder proves an attested statement
> *without* the verifier learning who/which-credential). These are **orthogonal axes**, not a
> containment relation. The trust graph can be a superset of the *expressivity* of a
> ZKap-gated decision (any predicate a ZKap rate-limit/age-gate enforces is expressible as a
> trust-graph rule over an attested statement) **only if** the attested statement is carried
> by a privacy-preserving presentation — and that presentation is exactly the ZKap/anonymous-
> credential machinery, which the trust graph would *consume*, not *replace*. The defensible
> claim is narrower and is stated in §6: **"the trust graph subsumes the *policy expressivity*
> of ZKap-gated access, and composes with — rather than supersedes — the *unlinkability* a
> ZKap provides."** Asserting plain superset would trip an honest reviewer and (for the ZK
> framing) the privacy-claims gate.

---

## 0. What the trust graph is (the design under study), restated precisely

From the brief: a **trust graph** is the set of statements/rules a storage server or resource
uses to decide **which sources it trusts for which access-control statements** — *per source,
per statement-type*. Trusted-source-attested statements (e.g. a government-issued VC
`<Jesse> <age> 25`) **merge** with `.acl`/ACP rules (e.g.
`{?x <age> ?y. FILTER(?y>18)} => {?x <canAccess> <r>}`) via **reasoning** to **derive** access.
It must support **capability delegation** for human *and* AI agents.

Two design obligations fall to *this* domain:

1. **The delegation requirement.** "Trust source X for statement-type T" and "delegate the
   capability to read `<r>` to agent A, attenuated" are the same shape of statement viewed
   two ways — a directed, attenuable, revocable grant. The object-capability / delegated-token
   literature (UCAN, ZCAP-LD, Macaroons, Biscuit, SPKI/SDSI, GNAP) is the body of art for
   *how to express, chain, attenuate, and verify such grants*, including the confused-deputy /
   on-behalf-of hazards that AI agents resurrect. §2–§4.
2. **The "superset of ZKaps" claim.** ZKaps + anonymous credentials are the body of art for
   *holder privacy* — proving an attested statement is true without revealing identity or
   linkable credential bytes. §5 states exactly what they give that a plain VC-presentation
   model does **not**, so the doc's claim can be re-phrased honestly (§6).

### 0.1 What sparq already implements in this area (verified against the code)

The trust graph is **not greenfield** in sparq — `sparq-solid` already ships a *coarse* version
of "which source for which statement", and the doc should build on it, not re-invent it:

- **Issuer-as-trust-dimension is implemented-and-verified.** ACP's `acp:issuer` (the OIDC IdP
  that vouched for a WebID) is a first-class principal dimension:
  `crates/sparq-solid/src/authindex.rs:25` defines `ANY_ISSUER =
  "https://sparq.dev/ns/auth#AnyIssuer"` (the "trust any issuer" top), and an absent
  `acp:issuer` ⇒ issuer-unconstrained. A constrained issuer mints a three-component
  `urn:sparq:triple?agent=A&client=C&issuer=I` principal
  (`research/solid-access-control-design.md` §3.6, "the third principal dimension … is the
  exact twin of the client dimension"). **This is the kernel of a trust graph:** "I trust
  issuer I to vouch for agent A's identity." The trust graph generalises it from *one*
  statement-type (identity-vouching, coarse-grained per-issuer) to *arbitrary* statement-types
  per source.
- **A trusted-fact channel discipline is implemented-and-verified.** Facts the rules consume
  must arrive over a trusted channel, never be forgeable from document content:
  `crates/sparq-solid/src/loader.rs:75-76` — *"The trusted channel for creator/owner facts is
  `AccessProvenance` and nothing else"* — plus a `urn:sparq:` reserved-principal guard
  (`validate_principal_iri`, `loader.rs:51`) that HARD-REJECTS attacker-minted principal IRIs.
  This is precisely the discipline a trust graph needs: **attested statements enter through an
  authenticated channel and are tagged with their source; a document cannot assert its own
  authority.** (Biscuit calls this *fact origin / scoping*; §3.4.)
- **Access control = triples + N3 rules + query rewrite is implemented-and-verified.** WAC/ACP
  are stored as triples and run as N3 rules by `sparq-reason`, materialising an authorization
  view (`<urn:sparq:auth>`) queried with ordinary SPARQL
  (`research/solid-access-control-design.md` §3, D1/D4 fail-closed). The trust graph slots in
  **upstream** of this: it decides *which attested triples are admitted as facts* before the
  `.acl`/ACP rules fire.
- **A documented gap the trust graph directly addresses.** `acp:vc` (VC-gated access) is an
  explicit **documented gap** (`research/solid-access-control-design.md` §7, "Not covered …
  `acp:vc`"). The trust graph is the mechanism that would close it: a VC is an attested
  statement from a source, and `acp:vc` is a rule that trusts that source for that
  statement-type.
- **The ZK estate that a ZKap binding would reuse is research-stage.** `sparq-zk` /
  `sparq-zk-compose` already commit credential terms off-circuit and verify SPARQL predicates
  over hidden typed values, with a **challenge-bound Schnorr HolderPoP** and an in-circuit
  issuer-signature gadget (`research/zk-holder-pop-design.md`,
  `research/zk-signed-credential-representation-design.md`). All of it is **NOT externally
  audited** (`sq-qhy4` pending) and must be treated as **not-yet-sound for production**. This
  is the substrate a privacy-preserving ("ZKap-like") presentation of a trust-graph fact would
  build on — see §5.4 / §6.

So the trust-graph proposal's novel surface, relative to what sparq has, is: (a) making
"trust source X for statement-type T" a **first-class, reasoned-over, queryable graph** rather
than a hard-coded issuer dimension; (b) supporting **multi-hop delegation/attenuation** of that
trust (today's issuer dimension is flat, single-hop); (c) a **privacy-preserving presentation**
option (the ZKap composition). §6 sequences these.

---

## 1. The two axes this domain spans (orientation)

```text
                     PRIVACY / UNLINKABILITY of the presentation
                      low (identifiable)            high (unlinkable)
                  +-------------------------------+-----------------------------+
  EXPRESSIVITY    | UCAN, ZCAP-LD, Biscuit,       | anonymous credentials       |
  of the grant /  | Macaroons, SPKI/SDSI, GNAP    | (Idemix, BBS), delegatable   |
  decision        | — rich delegation/attenuation,| anon-creds — rich predicate  |
  (high)          | NO holder privacy             | proofs AND unlinkable        |
                  +-------------------------------+-----------------------------+
  EXPRESSIVITY    | API keys, bearer JWTs,        | Privacy Pass / ZKaps         |
  (low — single   | OAuth2 scopes                 | (single-attribute "valid     |
  bit / opaque)   |                               | token", unlinkable, rate-    |
                  |                               | limited, double-spend-proof) |
                  +-------------------------------+-----------------------------+
```

The trust graph wants the **top-left expressivity** (reasoned, attenuable, multi-statement
grants) with an **opt-in path to the right column** (unlinkable presentation). No single
existing system occupies the top-right at the expressivity a reasoned `.acl`+VC merge needs;
**delegatable anonymous credentials** (§5.3) come closest and are the research target.

---

## 2. Object-capability foundations (the model the grants instantiate)

**Object-capability (ocap) model.** Authority is a *transferable, unforgeable reference* that
*both designates* a resource *and authorises* an operation on it — "no ambient authority". A
holder can only act through capabilities it has been given, and can pass (a subset of) them on.
This is the conceptual root of every system in §3 and is the model the trust graph's grants
should instantiate. The canonical hazard it solves is the **confused deputy** (Norm Hardy,
1988): a privileged intermediary is tricked into wielding *its own* authority on behalf of a
less-privileged caller because authority was *ambient* (tied to the deputy) rather than
*designated* (carried with the request). This hazard is the through-line to AI agents (§4).

**Why it matters for the trust graph.** "Trust source X for statement-type T" is an ocap-shaped
fact: it designates (source X, statement-type T) and authorises (admit-as-fact). Delegating it
("X may further authorise sub-source Y for a *narrower* T′") is ocap delegation with
attenuation. Modelling trust this way inherits ocap's confused-deputy resistance: authority to
*assert a fact* travels *with the attested statement and its provenance*, never as ambient
server state — which is exactly the `AccessProvenance` discipline already in `sparq-solid`
(§0.1).

---

## 3. Delegated-authorization systems — the key models

Each entry: **what it offers · access-control mechanism · delegation story · trust model ·
limitations.** All are *specified-in-an-external-standard* (or a published paper); none is
implemented in sparq except where §0.1 notes.

### 3.1 UCAN — User Controlled Authorization Networks (v1.0.0-rc)

- **What it offers.** Trustless, public-key-verifiable, *delegable* capability tokens with no
  central authorization server; local-first / offline. Principals are **DIDs** (`did:key`,
  Ed25519/P-256/secp256k1).
- **AC mechanism.** A capability = `subject × command × policy` (`sub`, `cmd`, `args` in the
  invocation; `pol` predicate in the delegation). The **resource owner roots authority** ("the
  Owner of a UCAN resource is the resource server directly") and delegates *downward*. A
  verifier checks the signature chain + the policy predicates against the invocation args.
- **Delegation.** *Principal alignment*: the `aud` (audience) of delegation N MUST equal the
  `iss` (issuer) of delegation N+1, forming a chain rooted at the owner. Each hop MUST
  **restate or attenuate** ("MUST either directly restate or attenuate (diminish) its
  capabilities"); broader subsumes narrower; validity is the intersection of `[nbf, exp]`
  windows. **Revocation** is by UCAN CID — learning a revocation fails all dependent
  validations.
- **Trust model.** Self-certifying: trust is rooted in the resource owner's key, propagated by
  signatures; no PKI / no CA. Verifier must see the *whole* chain.
- **Limitations.** **No holder privacy / no unlinkability** — `iss`/`aud` DIDs and the full
  policy/proof chain are *public to the verifier* (UCAN spec security considerations, verbatim:
  validators "must inspect the full delegation chain, which is necessarily visible"). **No
  confinement** ("impossible to guarantee knowledge of all sub-delegations"). Revocation needs
  a distribution channel (gossip / revocation list). Datalog/policy expressivity is bounded by
  the `pol` predicate language, not a general reasoner.

### 3.2 ZCAP-LD — Authorization Capabilities for Linked Data (W3C CCG, v0.3, work item restarted Apr 2026)

- **What it offers.** Capabilities as a **chain of Linked-Data (JSON-LD) documents** — the
  RDF-native cousin of UCAN. Directly relevant because the trust graph is RDF-native.
- **AC mechanism.** Authority starts at the **target** (which "always has authority to invoke
  itself"); each capability document points to its `parentCapability` (the target or another
  capability). Use is restricted by **`caveat`** properties; a capability *inherits all caveats
  of its parents* and MAY add more. **Invocation** is a signed proof (`capabilityInvocation`
  proof purpose) presenting the chain.
- **Delegation.** The **capability chain** (`parentCapability` links) is the delegation
  mechanism; `capabilityDelegation` proof purpose. Attenuation is monotone caveat accumulation
  (parents' caveats always apply; children only add). Invocation targets may only be narrowed
  (a suffix added to the parent's target).
- **Trust model.** Controller/`invoker`/`delegator` keys; verified with Data-Integrity proofs.
  RDF-native, so caveats can in principle be arbitrary LD predicates — a natural fit for
  reasoning.
- **Limitations.** **No holder privacy** (LD docs + proofs are visible). Caveat *semantics* are
  type-defined and must be agreed out-of-band; no standard caveat vocabulary ⇒ interop risk.
  Less momentum than UCAN; the Solid authorization panel discussed but did not adopt it
  (`solid/authorization-panel#160`). **[UNCERTAIN]** the restarted 2026 work item's current
  caveat-vocabulary direction — flag for the WG.

### 3.3 Macaroons (Google, NDSS 2014)

- **What it offers.** Bearer tokens that **attenuate and contextually confine** via **caveats**,
  built from **nested chained HMACs** — symmetric-key, no PKI, very cheap. Expressiveness "that
  rivals public-key mechanisms like SPKI/SDSI" per the paper.
- **AC mechanism.** The target service holds a root secret; each caveat is an HMAC over the
  running signature, so the holder can *add* caveats (restrict) but cannot *remove* them
  without the secret. Two caveat kinds: **first-party** (predicates the verifier checks
  locally, e.g. `expiry < T`, `op = read`) and **third-party** (require a *discharge macaroon*
  from a named third party — the basis of decentralised, multi-party authorization).
- **Delegation.** Attenuate-then-hand-off: anyone can add caveats and pass the macaroon on; the
  third-party-caveat mechanism delegates a sub-decision to another authority (which mints a
  discharge proving its predicate) — a clean *cross-domain* delegation primitive.
- **Trust model.** Symmetric: the verifier must share (or be able to derive) the root key;
  third-party caveats extend trust to discharge issuers by shared key / pre-agreement.
- **Limitations.** **No holder privacy** (caveats are cleartext; the bearer is linkable across
  uses by the token bytes). **Bearer** ⇒ theft = full use unless caveats bind context. Symmetric
  verification limits *public* verifiability (the verifier needs the secret), unlike
  UCAN/ZCAP/Biscuit. No revocation primitive beyond short expiry + caveats.

### 3.4 Biscuit (Eclipse Biscuit; spec at `eclipse-biscuit/biscuit`)

- **What it offers.** The closest existing system to the trust graph's *reasoning* core:
  **public-key-signed tokens carrying Datalog** (facts, rules, checks) with **offline
  attenuation**. "Merges the public-key signatures of JWT with offline attenuation + caveats of
  macaroons" plus a logic-programming authorization language.
- **AC mechanism.** Authorization is a **Datalog evaluation**: facts/rules from the *authority
  block*, each *attenuation block*, and the *authorizer* are loaded into one Datalog world;
  **checks** (`check if` / `check all` / `reject if`) must all pass, then **allow/deny policies**
  decide (first matching policy wins; deny ⇒ fail). This is structurally the same "merge
  attested facts + local rules → derive a decision" the trust graph wants — Datalog is a subset
  of N3 reasoning.
- **Delegation.** **Offline attenuation** via a **signature chain**: each block carries the next
  public key + a signature by the previous key; a holder appends a block with *more* checks
  (narrower rights) and cannot remove blocks without breaking the chain. Attenuation ephemeral
  keys are destroyed after use.
- **Trust model + the load-bearing mechanism for the trust graph — FACT SCOPING.** Biscuit's
  **fact origin/scoping** is *exactly* the "which source for which statement" primitive. Every
  fact carries an **origin** = the set of blocks that produced it; by default a block's
  rules/checks see only authority + current-block + authorizer facts. A rule can widen its
  trusted set with a **`trusting` annotation** naming origins — including
  **`trusting ed25519/<pubkey>`**, i.e. *"trust facts asserted by blocks signed by this
  external key."* **Third-party blocks** let a *trusted third party* sign a block (extra
  signature, isolated symbol table) attesting facts the token holder could not. **This is a
  cryptographic, per-source, per-fact trust primitive** — the single best prior-art match for
  the trust graph's core, and the doc should cite it as such.
- **Limitations.** **No holder privacy** (Datalog + signatures visible). Datalog is *less*
  expressive than full N3 (no built-in rich RDF reasoning, no SPARQL; stratified Datalog only).
  Scope/trust is keyed on *public keys*, not on RDF/source-identity vocabularies — the trust
  graph would lift this to IRIs/VC-issuers. Third-party-block UX requires an online round-trip
  to the third party at mint time.

### 3.5 SPKI/SDSI (RFC 2693, Experimental)

- **What it offers.** The theoretical foundation: **authorization certificates** binding
  *authorisations* (tags) directly to *keys* (not to names + a separate ACL), plus **SDSI local
  names** (each key is a namespace; `(name K alice)` is local to K) that compose into global
  names without a global CA.
- **AC mechanism.** A 5-tuple cert `(issuer, subject, delegation-bit, authorization-tag,
  validity)`; the **tag** is an s-expression describing the permitted action; verification is
  **tuple reduction** — chaining certs by intersecting tags and aligning issuer/subject keys.
- **Delegation.** Explicit **delegation control bit**: a cert may permit the subject to
  re-delegate (or not); the authorization tag is **intersected** down the chain (attenuation by
  tag-intersection). Local names give *named* delegation ("whoever Alice calls `bob`").
- **Trust model.** Fully decentralised, key-centric, no CA; trust = a reduced cert chain from a
  resource-controlling key to the requester. **This is the intellectual ancestor of the trust
  graph**: "names and authority are *local* to a key/source and composed by reduction" is the
  same move as "trust is *per-source* and composed by reasoning."
- **Limitations.** Only Experimental; never widely deployed; s-expression tags are not
  RDF/LD-friendly; **no holder privacy**; no standard revocation beyond CRL/validity. Tag
  intersection is a fixed algebra, not general reasoning.

### 3.6 GNAP — Grant Negotiation and Authorization Protocol (RFC 9635, Oct 2024)

- **What it offers.** "OAuth 3": a *negotiation* protocol for delegating authority to a specific
  *instance* of client software, returning **access tokens** and/or **subject information**.
  Clients need **no prior registration**; key-proofing (the client proves possession of a key)
  is built in.
- **AC mechanism.** The client requests access (resources + subject info); the Authorization
  Server negotiates, possibly involving the Resource Owner interactively, and issues
  bound/key-proofed access tokens.
- **Delegation.** Delegation is *to the client instance*; the protocol decouples the delegation
  channel (client↔AS) from the user-interaction channel — better than OAuth2's browser-redirect
  for headless / agent clients. **Increasingly cited for AI-agent delegation** (short-lived,
  key-bound, per-instance tokens; §4).
- **Trust model.** Centralised AS trust (like OAuth), but key-proof-bound tokens reduce bearer
  risk. Not self-certifying like UCAN/Biscuit.
- **Limitations.** Token *contents* / decisions are AS-internal — it standardises the
  *protocol*, not the *policy language* or a *capability chain*; no offline attenuation; **no
  holder privacy** by itself. Relevant to the trust graph as the *fetch/negotiate* transport for
  attested statements, not as the decision model.

### 3.7 Comparison table

| System | Verifiability | Attenuation | Trust root | Source-scoped facts? | Holder privacy | RDF-native |
|---|---|---|---|---|---|---|
| UCAN | public-key (DID) | restate/diminish per hop | resource owner key | no (capability, not fact) | none | partial (JSON, IPLD) |
| ZCAP-LD | public-key (DI proof) | monotone caveat accrual | target/controller key | no | none | **yes (JSON-LD)** |
| Macaroons | **symmetric (HMAC)** | add caveats (HMAC chain) | shared root secret | first/third-party caveats | none | no |
| Biscuit | public-key chain | append checks (sig chain) | root key | **YES (fact origin + `trusting ed25519/<pk>`)** | none | no (Datalog) |
| SPKI/SDSI | public-key | tag intersection + deleg-bit | resource key | local names per key | none | no (s-expr) |
| GNAP | key-proof token | n/a (protocol) | AS | no | none | no |

**Takeaway for the trust graph:** **Biscuit's fact-scoping** + **ZCAP-LD's LD-native chains** +
**SPKI's local-names-per-source** are the three primitives to synthesise; *none* of the six
gives holder privacy — that is the ZKap column (§5).

---

## 4. Delegation for human AND AI agents (the confused-deputy / on-behalf-of problem)

The trust graph must support delegation for AI agents, which sharpens classic hazards:

- **Confused deputy, resurrected.** When an agent invokes a tool/MCP server, the tool runs with
  the *agent's* credentials, not the *end-user's*; attacker-controlled content the agent ingests
  (prompt injection) can redirect that ambient authority (Okta/O'Reilly/safeguard.sh analyses,
  2025–2026). The ocap fix (§2) is to make authority **designated and carried with the request**
  — i.e. a *capability the agent holds on behalf of the user*, attenuated to the task, not an
  ambient API key. The trust graph inherits this directly: an AI agent admitting a fact must
  present a *delegated, attenuated grant* whose chain a verifier can reduce to a trusted source.
- **The delegation chain must be checked at action time.** "Authorization in agentic systems
  must check whose permissions apply at the moment an action is taken: the agent's own
  capabilities, AND the capabilities of every principal in the delegation chain" (O'Reilly,
  2026). UCAN/Biscuit chains and SPKI reduction already express this; the trust graph should
  reuse a chain model rather than invent one.
- **Standards in motion (cite, don't overclaim).** IETF **`draft-klrc-aiagent-auth-00`** (Mar
  2026) and the **AIP — Agent Identity Protocol** preprint (verifiable delegation across MCP and
  A2A) target exactly per-instance, short-lived, attenuated agent delegation; **GNAP** (§3.6) is
  the protocol most cited as the OAuth-side substrate. **[UNCERTAIN]** these are early
  Internet-Drafts/preprints — flag as moving targets, not stable dependencies.
- **Mapping.** "Human or AI" should be *uniform* in the trust graph: both are principals (DIDs /
  WebIDs) holding delegated, attenuated grants; the only delta is that AI-agent grants want
  **tighter default attenuation** (narrow `cmd`/caveat, short `exp`, key-proof binding à la GNAP)
  and **auditable provenance** of the whole chain (which sparq's `AccessProvenance` + a
  PROV-O lineage — `sparq-prov`, already shipping — can record).

---

## 5. ZKaps and anonymous credentials — exactly what they give (the "superset" crux)

This is the section that makes the "superset of ZKaps" claim honest. The question is precise:
**what does a ZKap / anonymous credential give that a plain VC-presentation model (UCAN/ZCAP/a
signed VC handed to the verifier) does NOT?** Answer: **unlinkability and minimal disclosure of
the *presentation*** — privacy properties, not authorization expressivity.

### 5.1 Privacy Pass / ZKaps (Zero-Knowledge Access Passes; RFC 9576/9577/9578)

- **What it offers (the exact properties).** Single-use **anonymous authorization tokens** with,
  per RFC 9576 (verbatim property names): **Origin-Client unlinkability**, **Issuer-Client
  unlinkability**, **Attester-Origin unlinkability**, **Redemption-context unlinkability**;
  issuance guarantees **unconditional input secrecy (blindness)**, **one-more-forgery security**,
  **concurrent security**. Roles: **Client, Origin, Issuer, Attester**. Tor/Brave "ZKAPs" are
  this family.
- **AC mechanism.** The token attests *one bit*: "the holder passed attestation (CAPTCHA /
  device check / account check) and was issued a valid token." The Origin checks the issuer
  signature (publicly verifiable variant) and **replay** state. It is **not** an
  attribute/identity statement.
- **Delegation.** **None** — "no mechanism for token transfer between clients; tokens are not
  intrinsically bound to specific users" (RFC 9576). Deliberately non-delegable.
- **Trust model.** Origin trusts the Issuer's key; the Attester vouches for the property out of
  band. **Double-spend / rate-limiting:** base tokens are **single-use** (Origin tracks
  redemption to prevent replay); dedicated **rate-limited / Anonymous-Rate-Limited-Credential**
  extensions (`draft-ietf-privacypass-rate-limit-tokens`, `draft-yun-privacypass-arc`) add
  bounded-use-per-epoch *without* deanonymising — the property a plain VC cannot give.
- **Limitations (the load-bearing point).** **No attribute disclosure, no predicates, no
  delegation.** A ZKap proves "valid + unlinkable + not-double-spent", nothing about *who* or
  *what attribute*. It is **strictly less expressive** than the trust graph on the authorization
  axis, and **strictly more private** on the presentation axis. They are orthogonal — hence the
  §6 reframing.

### 5.2 Classic anonymous credentials — Idemix (CL signatures) and U-Prove

- **What they offer.** Attribute-bearing credentials with **selective disclosure** AND
  **zero-knowledge predicate proofs** (e.g. prove `age ≥ 18` without revealing `age` or
  identity). This is the privacy-preserving carrier for an attested statement like
  `<Jesse> <age> 25` → "prove `age>18`" — exactly the trust graph's age-gate example, but
  unlinkable.
- **Idemix (IBM; Camenisch-Lysyanskaya signatures).** **Multi-show unlinkable**: a holder can
  present the *same* credential arbitrarily many times, each presentation cryptographically
  unlinkable to the others and to issuance, *without* the issuer online. Richest privacy; cost
  is pairing/CL-signature heavy.
- **U-Prove (Microsoft).** **Single-show**: a token is unlinkable to issuance (untraceable) but
  **multiple presentations of the same token are linkable** — multi-show unlinkability requires
  fetching a fresh token per use. Cheaper, but weaker than Idemix on linkability.
- **Modern instantiation — BBS / BBS+ (W3C VC-DI-BBS cryptosuite).** The VC-native realisation:
  the **holder** runs `ProofGen` to derive an **unlinkable** proof disclosing a chosen subset of
  attributes; "the BBS proof value is not linkable to the original BBS signature", and a fresh
  unlinkable proof can be derived each time. Contrast **SD-JWT-VC**: selective-disclosure but
  **NOT unlinkable** — the issuer signature is reused across presentations, so colluding
  verifiers can correlate the holder (techrxiv comparison; ETSI TR 119 476). **This is the exact
  delta** the §6 reframing turns on: *SD-JWT-style VC presentation = linkable; BBS / Idemix /
  ZKap = unlinkable.* The trust graph's example VC (`<Jesse> <age> 25`) presented as a plain
  signed VC or SD-JWT-VC **is linkable**; only a BBS/Idemix/ZK presentation gives the
  unlinkability a "ZKap-superset" claim would imply.
- **Limitations.** No native delegation/attenuation (you cannot hand a Idemix/BBS credential to
  another principal and have them re-prove on your behalf with attenuation) — see §5.3.
  Revocation under unlinkability is hard (accumulator / status-list-with-ZK schemes; sparq has
  `research/zk-statuslist-hide-iri-version.md` and `zk-dual-leaf-issuer-desync-review.md` in
  this space, research-stage).

### 5.3 Delegatable anonymous credentials (the missing top-right corner)

The one body of art that targets **both** axes at once: **delegatable anonymous credentials**
(e.g. the "Anonymous Delegatable Attribute-based Credential" line, ACM DL 2019). They let
authority be **delegated along a chain** *and* presented **unlinkably** — i.e. UCAN-style
attenuation *with* Idemix-style unlinkability. **This is the academic prior art closest to what
a "trust graph that is a superset of ZKaps + supports delegation" would actually need**, and the
doc should name it as the research target rather than claiming the property is already had.
**Status: published cryptographic research, NOT a deployed standard, and NOT in sparq.**
**[UNCERTAIN]** practical performance/maturity for an RDF/Solid setting — open research.

### 5.4 What sparq's ZK estate already provides toward this (verified, caveated)

sparq's `sparq-zk` / `sparq-zk-compose` is a **single-prover ZK** estate that can already prove
SPARQL predicates (FILTER `>`, equality, join) over **hidden** committed credential terms, with
an in-circuit issuer-signature gadget and a challenge-bound HolderPoP
(`research/zk-holder-pop-design.md`, `zk-signed-credential-representation-design.md`,
`zk-hidden-join-design.md`). In trust-graph terms this is the machinery to present
`{?x age ?y. FILTER(?y>18)} ⇒ canAccess` **without revealing `age`** — i.e. an *unlinkable
predicate presentation* of a trust-graph fact. **BUT:** (a) it is **NOT externally audited**
(`sq-qhy4` pending external accredited-cryptographer sign-off) and **must be treated as
not-yet-sound for production**; (b) it does **not** yet do *credential→holder binding without the
issuer online* in full (the `sq-c2ql` HolderPoP design closes part of this but is *designed-only*);
(c) it provides single-prover unlinkable *presentation*, **not delegation** — there is no
delegatable-anon-cred capability in sparq today. The MPC estate (`sparq-mpc`) is
**semi-honest-only** and is the multi-party layer, not a privacy presentation. So: sparq has a
research-stage substrate for the *unlinkable-presentation* half of §6, and **nothing** for the
*delegatable-unlinkable* corner (§5.3).

---

## 6. How this domain maps to the trust-graph design (and the honest "ZKap" reframing)

### 6.1 The mapping

| Trust-graph need | Best prior-art primitive | Reuse in sparq |
|---|---|---|
| "Trust source X for statement-type T" as a fact | **Biscuit fact-origin + `trusting ed25519/<pk>`**; SPKI local-names-per-key | generalise `acp:issuer`/`ANY_ISSUER` + `AccessProvenance` (already shipping) |
| Attested statement enters only via authenticated channel | `AccessProvenance` trusted-channel discipline (sparq) + Biscuit third-party block | **implemented-and-verified** in `sparq-solid` |
| Merge attested facts + `.acl`/ACP rules → derive access | **Biscuit Datalog**, but sparq uses **N3 (more expressive)** | **implemented-and-verified**: `sparq-reason` materialiser |
| Multi-hop attenuable delegation of trust | **UCAN** principal-alignment + restate/attenuate; **ZCAP-LD** monotone caveats; **SPKI** deleg-bit + tag-intersection | **designed-only** — not in sparq's flat issuer dimension |
| RDF/LD-native capability chains | **ZCAP-LD** | aligns with sparq's triples-native AC |
| Human + AI agent delegation, confused-deputy-safe | ocap model; GNAP key-proof tokens; AIP / `draft-klrc-aiagent-auth` | provenance recordable via `sparq-prov` (shipping) |
| **Unlinkable** presentation of an attested statement | **BBS / Idemix / ZKap**; **delegatable anon-creds** for the delegated case | **research-stage** `sparq-zk` (NOT audited); delegated-unlinkable: **absent** |
| Bounded-use / rate-limit without deanonymisation | **Privacy-Pass ARC / rate-limited tokens** | absent — proposed |

### 6.2 The honest reframing of "superset of ZKaps"

**Do not claim plain superset.** The defensible, gate-safe claim is two-part:

1. **Policy-expressivity superset (true, designed-only→reasoned).** Any access decision a ZKap
   gates — "holder passed attestation", "≤ N uses per epoch", "age ≥ 18" — is expressible as a
   trust-graph rule over an attested statement merged with `.acl`/ACP by N3 reasoning. The trust
   graph is *strictly more expressive on the authorization axis* than a single-bit ZKap, and at
   least as expressive as Biscuit-Datalog (N3 ⊇ stratified Datalog). This is a sound "superset"
   *of the decision logic*.
2. **Composition, NOT supersession, of unlinkability (the correction).** A ZKap's *privacy*
   (unlinkable, blind, one-more-unforgeable, rate-limited-without-identity) is a property of the
   **presentation**, which the trust graph **consumes**, not replaces. To match it, the trust
   graph must carry its attested statements via a privacy-preserving presentation — BBS/Idemix
   for the single-show/identity case (§5.2), **delegatable anonymous credentials** for the
   delegated case (§5.3), Privacy-Pass-ARC-style tokens for the rate-limited-anonymous case
   (§5.1). Absent that carrier, the trust graph's presentations are **linkable** (SD-JWT-VC /
   plain signed VC / UCAN chains are all linkable — §3, §5.2) and therefore **NOT a superset of
   a ZKap on the privacy axis.**

**One-line claim for the standards doc:** *"The trust graph subsumes the policy expressivity of
ZKap-gated and capability-token access control, and is designed to **compose** with anonymous-
credential / ZKap presentations to recover their unlinkability — it does not by itself supersede
that unlinkability."* This survives an honest reviewer and the privacy-claims gate.

---

## 7. Gaps this domain leaves that the trust graph must address

1. **Delegated AND unlinkable simultaneously is unsolved off-the-shelf.** Capability systems
   (§3) give delegation/attenuation with **zero** privacy; anonymous credentials (§5.2) give
   unlinkability with **no** delegation. Only **delegatable anonymous credentials** (§5.3) span
   both, and they are **research, not standards, and not in sparq**. The trust graph cannot
   assume this corner exists; it must either (a) scope the unlinkable case to *non-delegated*
   presentations first, or (b) take delegatable-anon-creds as an explicit research dependency.
2. **No standard "source-trust" vocabulary.** Biscuit scopes on public keys, SPKI on local
   names, ZCAP-LD has *no standard caveat vocabulary*. The trust graph needs an **RDF vocabulary
   for "source X trusted for statement-type T (under conditions C)"** — this is net-new and is
   the WG's contribution surface. It must compose with the existing `acp:issuer` / `acp:vc`
   shape (§0.1) and ODRL (sparq's `research/feature-research-odrl-policy.md`).
3. **Statement-type granularity / fact-origin in RDF reasoning.** Biscuit's per-fact *origin*
   tracking is the model, but sparq's N3 materialiser currently keys trust at the *issuer*
   granularity (coarse), not per-statement-type/per-predicate. Bringing fact-origin/provenance
   into the *reasoning* step (so a rule can say "use `age` only from a government issuer") is a
   concrete engine gap. `sparq-prov` (PROV-O lineage) is the hook.
4. **Revocation under the chosen model.** UCAN-by-CID, SPKI-CRL, status-lists — none integrates
   cleanly with unlinkable presentations (revocation-under-unlinkability is hard; sparq's
   `zk-statuslist-*` work is research-stage). The trust graph must pick a revocation story per
   presentation mode.
5. **AI-agent attenuation defaults + auditable chains.** The confused-deputy hazard (§4) demands
   tight default attenuation and end-to-end chain provenance for agent-presented facts; the
   relevant standards (`draft-klrc-aiagent-auth`, AIP, GNAP key-proofing) are **early/moving**.
   The trust graph needs a default-attenuation policy and a provenance-capture obligation, not a
   dependency on an unstable draft.
6. **Confinement / sub-delegation visibility.** UCAN explicitly cannot offer confinement; a
   server cannot enumerate all sub-delegations of a trust grant. The trust graph must decide
   whether it needs confinement (e.g. for high-stakes facts) and, if so, design a
   registration/escrow mechanism — none of the §3 systems provide it.
7. **External audit is a hard gate on the privacy half.** Any unlinkable-presentation path
   reuses sparq's ZK estate, which is **NOT externally audited** (`sq-qhy4`). The trust graph's
   privacy claims are **blocked on that sign-off** and must be presented as designed/research,
   never as a guarantee.

---

## 8. Recommendation

**Adopt a Biscuit-style fact-scoping core, expressed in RDF, reasoned over by N3, with delegation
modelled on UCAN/ZCAP-LD chains, and an explicitly *opt-in, separately-sequenced* unlinkable-
presentation path — and reframe the "superset of ZKaps" claim per §6.2.**

Concretely: (a) lift sparq's existing `acp:issuer`/`AccessProvenance` trust dimension into a
first-class **trust-graph vocabulary** ("source ⊳ statement-type ⊳ conditions") that the N3
materialiser consults *before* `.acl`/ACP rules fire (closes §0.1's `acp:vc` gap); (b) add
**fact-origin/provenance into the reasoning step** (Biscuit's `trusting`, but per-predicate, via
`sparq-prov`); (c) add **multi-hop attenuable delegation** of trust grants (UCAN principal-
alignment + monotone caveats, RDF-native à la ZCAP-LD), with **AI-agent-tight default
attenuation** and **chain provenance**; (d) treat the **unlinkable presentation** (BBS/Idemix/
ZKap, and the harder delegatable-anon-cred corner) as a **later, opt-in, audit-gated** phase, and
in the meantime state the policy-expressivity superset honestly (§6.2) without claiming the
privacy superset.

---

## 9. Phased plan (each phase = a future bead the orchestrator can track)

> These are *proposed* beads for the maintainer; none is started. Phases are ordered by
> dependency; the privacy phases (P5–P6) are explicitly gated on external audit (`sq-qhy4`).

1. **P1 — Trust-graph RDF vocabulary + semantics (design bead).** Define the
   "source ⊳ statement-type ⊳ conditions ⊳ delegable?" vocabulary; reconcile with `acp:issuer`,
   `acp:vc`, ODRL, ZCAP-LD caveats; specify the merge semantics with `.acl`/ACP in N3. *Maps to
   §0.1, §6.1, gap #2.* Deliverable: a `research/` semantics record + a proposed vocabulary IRI
   space. (Doc-only; no audit gate.)
2. **P2 — Per-source/per-predicate fact-origin in the N3 materialiser (design→impl bead).** Bring
   Biscuit-style fact-origin/`trusting` granularity into the reasoning step via `sparq-prov`, so
   a rule can trust a predicate only from a named source. *Maps to gap #3.* Depends on P1.
3. **P3 — Authenticated-channel ingestion of attested statements (impl bead).** Generalise the
   `AccessProvenance` trusted channel (already shipping for creator/owner) to arbitrary attested
   triples + their source binding, with the `validate_principal_iri` guard extended. *Maps to
   §0.1.* Depends on P1.
4. **P4 — Multi-hop attenuable delegation of trust grants (design→impl bead).** UCAN/ZCAP-LD-style
   RDF-native delegation chains with monotone attenuation, principal alignment, time bounds, and
   a chosen revocation story; **AI-agent default-tight attenuation + GNAP-style key-proof binding
   + end-to-end chain provenance.** *Maps to §3.1–3.6, §4, gaps #4–#6.* Depends on P1–P3.
5. **P5 — Unlinkable (non-delegated) presentation path, opt-in, audit-gated (research→impl bead).**
   Carry a trust-graph fact via BBS/Idemix/ZK presentation reusing `sparq-zk` (FILTER/equality
   predicate proofs over hidden terms). **Hard-blocked on external ZK audit `sq-qhy4`; semi-honest
   caveats; privacy-claims gate.** *Maps to §5.1–5.2, §5.4, gap #7.* Depends on P1, P4.
6. **P6 — Delegatable-anonymous-credential corner (research bead).** Investigate delegatable
   anonymous credentials (§5.3) for the *delegated AND unlinkable* case, and Privacy-Pass-ARC-
   style bounded-use-without-identity. **Research-only until a sound, audited scheme exists.**
   *Maps to §5.3, §5.1, gap #1.* Depends on P5.
7. **P7 — Standards-track write-up for LWS + Solid WGs (doc bead).** Fold P1–P6 outcomes into the
   WG proposal with the honest §6.2 framing; do NOT submit the privacy superset claim until P5's
   audit clears. *Maps to §6.2.* Depends on P1–P4 (P5–P6 as future work sections).

---

## 10. Open questions that genuinely need the maintainer

1. **Scope of "superset of ZKaps" for the WG submission.** Confirm the §6.2 two-part reframing
   (policy-expressivity superset + composition-not-supersession of unlinkability). Is the
   policy-only superset claim acceptable for v1, with the privacy composition as future work?
2. **Delegation chain model: RDF-native ZCAP-LD vs UCAN vs a sparq-specific N3 encoding?** ZCAP-LD
   is LD-native but low-momentum; UCAN has momentum but is JSON/IPLD; a native N3 encoding
   maximises reasoner reuse. Which does the maintainer want to propose to the WGs?
3. **Trust granularity: per-issuer (today) vs per-predicate (Biscuit-style)?** P2 assumes
   per-predicate fact-origin is wanted. Confirm — it is a non-trivial reasoning-engine change.
4. **Is the *delegated-AND-unlinkable* corner in scope at all, or explicitly future work?** §5.3
   has no deployed standard; committing to it sets a research dependency. P6 currently scopes it
   as research-only.
5. **Revocation model preference** (UCAN-CID gossip / SPKI-CRL / status-list / ZK-accumulator),
   given it interacts with the unlinkability choice (gap #4).
6. **AI-agent delegation: depend on `draft-klrc-aiagent-auth`/AIP/GNAP, or define a sparq-local
   attenuation policy?** These drafts are early (Mar 2026); the recommendation is sparq-local
   defaults + provenance, treating the drafts as alignment targets, not dependencies. Confirm.

---

## 11. Citations

External specs/papers (all fetched/searched 2026-06; **[UNCERTAIN]** markers in-text flag
moving Internet-Drafts/preprints):

- UCAN 1.0.0-rc spec — <https://github.com/ucan-wg/spec/blob/main/README.md> ;
  <https://ucan.xyz/specification/>
- ZCAP-LD — Authorization Capabilities for Linked Data v0.3 (W3C CCG) —
  <https://w3c-ccg.github.io/zcap-spec/> ; restarted work item, public-credentials list 2026-04 —
  <https://lists.w3.org/Archives/Public/public-credentials/2026Apr/0049.html>
- Macaroons — Birgisson, Politz, Erlingsson, Taly, Vrable, Lentczner, NDSS 2014 —
  <https://research.google/pubs/pub41892/>
- Biscuit — specification, `eclipse-biscuit/biscuit` —
  <https://github.com/eclipse-biscuit/biscuit/blob/main/SPECIFICATIONS.md> ;
  <https://doc.biscuitsec.org/>
- SPKI/SDSI — RFC 2693, SPKI Certificate Theory — <https://www.rfc-editor.org/rfc/rfc2693.html>
- GNAP — RFC 9635, Grant Negotiation and Authorization Protocol —
  <https://www.rfc-editor.org/info/rfc9635/>
- Privacy Pass — RFC 9576 (Architecture), RFC 9577 (HTTP Auth scheme), RFC 9578 (Issuance) —
  <https://www.rfc-editor.org/rfc/rfc9576.html> ;
  rate-limited / ARC drafts — <https://www.ietf.org/archive/id/draft-yun-privacypass-arc-02.html>
- Anonymous credentials — Idemix (Camenisch–Lysyanskaya) / U-Prove (Microsoft); multi-show vs
  single-show — survey via Springer/ScienceDirect; ETSI TR 119 476 (selective-disclosure
  mechanisms) — <https://www.etsi.org/deliver/etsi_tr/119400_119499/119476/01.02.01_60/tr_119476v010201p.pdf>
- BBS / BBS+ — W3C Data Integrity BBS Cryptosuites v1.0 — <https://www.w3.org/TR/vc-di-bbs/> ;
  BBS+ vs SD-JWT unlinkability comparison —
  <https://www.techrxiv.org/doi/pdf/10.36227/techrxiv.175492163.32399388/v1>
- Delegatable anonymous attribute-based credentials — ACM DL 2019 —
  <https://dl.acm.org/doi/pdf/10.1145/3338854>
- Vouchsafe — zero-infrastructure capability-graph model for offline identity/trust (preprint) —
  <https://arxiv.org/pdf/2601.02254>
- Confused deputy / AI-agent delegation — O'Reilly "Who Authorized That?" ; Okta delegation-chain
  ; IETF `draft-klrc-aiagent-auth-00` (2026-03) —
  <https://www.ietf.org/archive/id/draft-klrc-aiagent-auth-00.html> ; AIP — Agent Identity
  Protocol (preprint) — <https://arxiv.org/pdf/2603.24775>

sparq internal (verified against the code/docs in this repo at the cited file:line):

- `crates/sparq-solid/src/authindex.rs:25` (`ANY_ISSUER` trust top), `:86` (`acp:issuer` =
  OIDC issuer dimension)
- `crates/sparq-solid/src/loader.rs:51` (`validate_principal_iri` reserved-principal guard),
  `:75-76` (`AccessProvenance` is the only trusted fact channel)
- `research/solid-access-control-design.md` §3.6 (issuer dimension), §7 (`acp:vc` documented gap)
- `research/zk-holder-pop-design.md`, `research/zk-signed-credential-representation-design.md`,
  `research/zk-hidden-join-design.md`, `research/zk-statuslist-hide-iri-version.md`,
  `research/zk-dual-leaf-issuer-desync-review.md` (ZK estate — research-stage, NOT audited,
  `sq-qhy4` pending), `research/zkp-performance-landscape.md`
- `crates/sparq-prov/` (PROV-O lineage — provenance hook for fact-origin)
- `research/feature-research-odrl-policy.md` (ODRL policy — composition target)
