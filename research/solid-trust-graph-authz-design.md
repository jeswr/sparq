# Trust-graph authorisation for Solid/LWS: trusted-source-scoped statements + rules

Status: **design-for-review** (research record). This is a *proposal* to be taken to the
LWS and Solid working groups, not a shipped feature and not a security guarantee. It builds
on, and composes with, the *shipped* Solid access-control substrate in `crates/sparq-solid`
([`research/solid-access-control-design.md`](solid-access-control-design.md)) and the
research-stage ZK/credential estate in `crates/sparq-zk` / `crates/sparq-zk-compose`. The
privacy/unlinkability half of this design depends on that ZK estate, whose external
accredited-cryptographer sign-off is **pending** (bead `sq-qhy4`) and whose MPC layer is
honest-majority semi-honest only — see [§7 Honest limitations](#7-honest-limitations--comparison).
No performance numbers are quoted here; any timing taken on the work box is non-canonical.

<!-- [OPUS-4.8] lead-designer trust-graph authz design record; companion to the six in-flight
prior-art surveys (PRs #942/#943/#945/#946/#947/#949). -->

GitHub issue: [#940](https://github.com/jeswr/sparq/issues/940).

## 0. The request (verbatim)

> There is an authorisation system design that I would like to prototype as we do this to
> propose to the LWS and Solid working groups. The idea is that often we want to provide
> RBAC and ABAC on the solid ecosystem; and the only way of coming close to that is using
> the acp:vc matchers which are very hacky because the matcher is "do you have a verifiable
> credential of this exact access grant shape." Specifically, the way that the access control
> system would work is by having a "trust graph" which is the set of statements and/or rules
> that are used to determine the sources that a given storage server or resource will trust
> for making statements about access control. These rules can be specific enough to say that
> only some types of statements are trusted from particular sources. In Solid today the way
> that this effectively works is that the .acl and the .acl of parent resources is the one
> and only trusted source of information. If we consider the case of, e.g., wanting to enable
> people over the age of 18 to access a resource, then we need to think about this a little
> differently. In this case the trust graph may trust the .acl document and statements about
> age issued by trusted governments. This then lets someone present a verifiable credential
> where the age statement `<Jesse> <age> 25 .` can be merged with the .acl rule saying
> `{ ?x <age> ?y . FILTER(?y > 18) } => { ?x <canAccess> <resourceX> }` to get to the fact
> that I can access the resource. ... support capability delegation for (human and AI)
> agents. Show that with the right ontology terminology, this addresses a superset of the
> features of technologies such as ZKaps.

## 1. Motivation — why `acp:vc` is a hack, and what RBAC+ABAC on Solid needs

### 1.1 The deployed substrate, verified

Two things are true in Solid today and in `sparq-solid` specifically; both were checked
against the code, not assumed:

- **`.acl`/`.acr` is the one and only trusted source.** `sparq-solid` materialises a
  `<urn:sparq:auth>` allow-list by running WAC + ACP as N3 rules
  (`crates/sparq-solid/rules/*.n3`) over **only** the `.acl`/`.acr`/group graphs plus
  loader-synthesised structural facts; pod *content* is deliberately excluded from the
  reasoner (`research/solid-access-control-design.md` §2.4). The author of the control
  document is the sole trust root.
- **`acp:vc` matches a credential's *type IRI*, not its *claims*.** ACP's `acp:vc` matcher
  (per the ACP spec) is satisfied when a presented Verifiable Credential *is of a stated
  type*; it does **not** read the credential's contents. You can require "a VC of type
  `:AgeCredential`"; you cannot say "…whose `age` is greater than 18". That is the
  maintainer's "exact-credential-shape hack": to approximate ABAC you must mint a credential
  whose very *shape* encodes the grant, and the issuer effectively makes the access decision
  for you. **`acp:vc` is not implemented in `sparq-solid` at all** — `grep -rc "acp:vc"
  crates/sparq-solid/` returns zero — so claim-level credential reasoning is genuinely new
  here, not an extension of existing code (correction **C2**, §1.3).

### 1.2 What RBAC and ABAC actually require

- **RBAC** (ANSI/INCITS 359): access is set-membership in named role↔permission relations.
  On Solid you can fake a role with an `acl:agentGroup`, but the group membership has no
  *provenance* — you cannot say "trust HR for the *manager* role but trust a government for
  *over-18*". A role should be a *derived view* of source-attested attributes, not a flat,
  single-authority list.
- **ABAC** (NIST SP 800-162): access is a boolean function of subject/object/environment
  *attributes*. The standard names "trusted attribute sources / Attribute Authorities" as a
  first-class operational concern — but leaves *which authority is trusted for which
  attribute* **out of band**, a deployment decision, not a machine-checkable statement.
  XACML 3.0 approximates this only at *policy* granularity (its `PolicyIssuer` +
  reduction); VC trust registries do it only as a *flat issuer list* per credential *type*.
  None expresses **per-(source, statement-type)** trust as data that a reasoner merges with
  the local `.acl` rules.

That missing object — *"resource R trusts source S for statements of type T"*, as RDF the
reasoner consumes — is what this design adds. ABAC then falls out: the age rule
`{ ?x age ?y . FILTER(?y > 18) } => { ?x canAccess R }` fires over attested attributes; and
RBAC falls out as the special case where T is a role-assignment predicate.

### 1.3 Corrections to the brief's premise (be honest up front)

- **C1.** sparq-solid's `acp:issuer` dimension is **not** per-statement trust. It is the OIDC
  issuer that vouched for the requester's WebID (`Session.issuer`, matched as an IRI in
  `authindex.rs`; `ANY_ISSUER` is the top). It is an *identity* dimension, not a
  *which-source-for-which-claim* dimension. The trust graph generalises it; it does not reuse
  its semantics.
- **C2.** `acp:vc` is *type-only* and *unimplemented* in this repo. Claim-level reasoning over
  credential contents — the heart of this design — does not exist today.
- **C3 (the "blank slate" correction).** The design is **not** greenfield. sparq already ships
  *single-axis instances* of the trust primitive that this design unifies: the `acp:issuer`
  identity dimension; the `AccessProvenance` trusted-fact channel with its `solidx:`
  reserved-predicate anti-forgery guard (§2.4 of the AC design); the PROV-O lineage in
  `crates/sparq-prov`; and the `crates/sparq-zk` issuer-signed-commitment registry
  (`zk:issuerKey`, `zk:statusList`). The novelty is the *unification* of these into one
  general, queryable, per-(source, statement-type, constraint) trust layer — and the strings
  "trust graph" / "ZKaps" appear nowhere in the repo, so the *framing and vocabulary* are new
  even where a mechanism is partially present.

## 2. The trust-graph model

### 2.1 Definition

A **trust graph** is an RDF graph of **trust statements** plus the local **`.acl`/`.acr`
rules**, that together determine which externally-attested statements a resource (or server)
will *admit* into its access-control derivation, scoped per *source* and per *statement-type*.

Formally, the access decision is a two-stage reasoning pipeline:

```text
   attested statements (VC claims, signed graphs)
                 │
                 ▼
   ┌──────────────────────────────────┐
   │  ADMISSION stratum  (NEW)         │   gate: trust statement + verified issuer
   │  "is this fact from a source I    │          signature + freshness/revocation
   │   trust for this statement-type?" │
   └──────────────────────────────────┘
                 │  admitted facts (issuer-tagged)
                 ▼
   ┌──────────────────────────────────┐
   │  DERIVATION stratum  (EXISTING)   │   the shipped sparq-solid WAC/ACP N3 rules
   │  ".acl rules over the admitted    │   + any ABAC rule (age>18 ⇒ canAccess)
   │   facts ⇒ canAccess"              │
   └──────────────────────────────────┘
                 │
                 ▼
        <urn:sparq:auth>  (allow-list, fail-closed)
```

The admission stratum is the genuinely new contribution; the derivation stratum is the
existing `sparq-solid` materialiser, which already turns a merged assertion graph into a
materialised `<urn:sparq:auth>` view.

### 2.2 Trusted-source-scoped trust (the one new primitive)

The core relation is **per-(source, statement-type)** — strictly finer than every surveyed
prior art:

- Named Graphs / VC Data Integrity attest **per graph** (issuer signs a whole graph).
- Annotated-RDF / tSPARQL attach a trust value **per triple**, but *type-agnostic*.
- VC trust registries (DIF Trust Establishment, EUDI Trusted Lists) trust an issuer **per
  credential type**, as a flat list — no reasoning, no merge with local rules.
- ZCAP-LD/UCAN trust **per resource-capability**.

None expresses "trust source S for statements *of type T* (and only T)". The trust graph
does, and feeds it to the reasoner.

The **degenerate case** recovers today's Solid exactly: a trust graph whose only statement is
"trust the `.acl`/`.acr` document for all access-control predicates" reproduces WAC/ACP
unchanged. This is the strict-additivity property (G6, §7): **a pod with no trust graph
behaves exactly as WAC/ACP do now.** Solid-OIDC is the *n = 1* case (one issuer, one
statement-type: the identity assertion).

### 2.3 Proposed ontology terms

All IRIs below under `https://sparq.dev/ns/trust#` (prefix `trust:`) are **NON-STANDARD,
invented for this proposal**; a WG would rename/rehome them. They are placeholders to make
the semantics concrete, not a claim of standardisation.

| Term (proposed, non-standard) | Domain → Range | Meaning |
|---|---|---|
| `trust:TrustPolicy` | class | A set of admission rules scoping a resource/server. |
| `trust:trustsSourceFor` | `trust:Source` → predicate/shape | Admit statements of this type from this source. |
| `trust:source` | rule → `trust:Source` | The attesting source a trust rule names. |
| `trust:forPredicate` | rule → `rdf:Property` | Statement-type as a predicate IRI (coarse). |
| `trust:forShape` | rule → `sh:NodeShape` | Statement-type as a SHACL shape (fine; uses shipped `sparq-shacl`). |
| `trust:Source` | class | An attesting authority, identified by an issuer key / DID. |
| `trust:issuerKey` | `trust:Source` → key/DID | Verification key the source signs with (aligns with `zk:issuerKey`). |
| `trust:scope` | rule → resource/container | Where the trust rule applies (server-wide vs per-`.acr`). |
| `trust:freshWithin` | rule → `xsd:duration` | Maximum staleness admitted (consulted against `Session.now`). |
| `trust:admitted` | (internal) | Marks a fact that passed admission; analogous to `solidx:` internal vocab. |

The choice between `trust:forPredicate` (predicate IRI) and `trust:forShape` (SHACL shape) is
the **statement-type granularity open question** (§8). The design proposes predicate-IRI for
v1 (cheap, decidable) with `trust:forShape` as the expressive upgrade, reusing the shipped
`sparq-shacl` engine.

### 2.4 Relation to named-graph/quad trust and VC issuer trust

The *unit of attestation* is a Carroll-Bizer named graph made issuer-bound via VC Data
Integrity / RDFC-1.0 — which is **byte-aligned with sparq's existing crypto estate**:
`sparq-zk` already commits per-named-graph over RDFC-1.0-canonicalised leaves
(`crates/sparq-zk/src/{canon,commit,sig}.rs`), and `crates/sparq-canon` already wraps
RDFC-1.0 as a public API. So a trust graph's *input* needs no new canonicalisation
infrastructure: it consumes the same signed-graph unit the ZK estate already produces.

The trust statement itself is the **machine-readable, reasoning-driven form** of the VCDM 2.0
trust model — the spec already says "verifiers trust certain issuers for certain claims" but
*deliberately leaves that decision out-of-band and non-machine-readable*. The trust graph
reifies exactly that out-of-band decision as in-graph triples, per source **and** per
statement-type, then lets the reasoner derive access.

## 3. Evaluation and construction

### 3.1 Worked example: the age-over-18 case

**Inputs.**

```n3
# (a) The .acl rule (ABAC), authored by the resource controller — TRUSTED root:
{ ?x schema:age ?y . ?y math:greaterThan 18 } => { ?x auth:read <resourceX> } .

# (b) The trust statement, in the resource's trust policy — also controller-authored:
[] a trust:TrustRule ;
   trust:source     <https://gov.example/issuer> ;
   trust:issuerKey  <did:web:gov.example#key-1> ;
   trust:forPredicate schema:age ;
   trust:scope      <resourceX> ;
   trust:freshWithin "P30D"^^xsd:duration .

# (c) The presented credential graph G, signed by the gov issuer over its RDFC-1.0 commitment:
#   <Jesse> schema:age 25 .            (claim)
#   plus a VC Data Integrity proof binding G to did:web:gov.example#key-1
```

**Admission stratum.** A fact `<Jesse> schema:age 25` is admitted **iff** all hold:

1. there is a trust rule whose `trust:forPredicate` is `schema:age` and whose `trust:scope`
   covers `<resourceX>`;
2. the credential graph G carrying that triple is verified to be **signed by the key**
   `trust:issuerKey` names — a *checked* signature over G's RDFC-1.0 commitment, **not** a
   self-asserted "I am signed" triple (this is the load-bearing soundness condition, §3.3);
3. the credential is fresh (within `trust:freshWithin` of `Session.now`) and not revoked
   (status-list check);
4. the credential subject `<Jesse>` **binds to the authenticated requester** (the holder
   binding / subject-to-WebID join, §3.4).

Only then does `<Jesse> schema:age 25` enter the derivation stratum as a `trust:admitted`
fact tagged with its issuer.

**Derivation stratum.** The shipped sparq-solid materialiser runs rule (a) over the admitted
fact; `math:greaterThan` fires; `<Jesse> auth:read <resourceX>` lands in `<urn:sparq:auth>`.
A subsequent query is rewritten to that allow-list exactly as today.

### 3.2 Construction (how to build both graphs)

- The **trust graph is authored in the same trusted channel as `.acl`/`.acr`** — i.e. behind
  `acl:Control` (WAC) / ACR-write (ACP). Whoever may write the policy may write the trust
  rules; nothing else may. This is the NGAC self-administration discipline (administration is
  the same graph, governed by the same decision function) and it *avoids XACML's documented
  out-of-band-root gap* (NIST SP 800-178: trusted policies are "assumed valid, and their
  origin is established outside the delegation model").
- **Scope** is a v1 open question (§8): server-wide trust policy, per-`.acr` trust rules, or
  both. The design recommends *both*, with per-`.acr` rules narrowing (never broadening) a
  server default — monotone like ACP inheritance.

### 3.3 Soundness — the boundary this re-opens, and how it stays closed

This is the single most important correctness claim and the place an adversarial reviewer
should look first.

sparq-solid **deliberately closed** the content/reasoner boundary (§2.4 of the AC design):
pod *content* is never fed to the reasoner, because a writer who could inject `acl:`/`solidx:`
triples could self-grant. The trust graph **re-opens that boundary by policy** — it *admits*
external content into the derivation — which re-creates the escalation risk unless admission
is gated correctly. The design's safety argument:

1. **Admission requires a *verified* issuer signature, never a self-asserted one.** A graph
   claiming `trust:issuerKey ...` proves nothing; the signature over its RDFC-1.0 commitment
   must be *checked* against the key the trust rule names, exactly as the loader today checks
   `validate_principal_iri` and *rejects* forged `solidx:` predicates
   (`is_reserved_derivation_predicate`). An untrusted source's statement is **not usable**:
   no matching `trust:trustsSourceFor` ⇒ not admitted ⇒ invisible to the derivation
   (fail-closed, D4).
2. **Statement-type scoping is enforced at admission, not derivation.** A source trusted for
   `schema:age` cannot launder an `acl:agent` or `solidx:creator` triple in: those predicates
   are *not* in its `trust:forPredicate`/`trust:forShape` set, so the admission rule never
   fires for them. The reserved-derivation-predicate guard remains in force *under* this — a
   trust rule cannot grant a source the right to assert `solidx:` internal vocabulary.
3. **The trust graph itself is Control-gated.** Because trust rules live in the same channel
   as `.acl`, an attacker cannot insert a self-favouring trust rule without already holding
   write access to the policy — at which point they could edit `.acl` directly. No new escalation
   surface is opened *by the trust graph as data*; the new surface is purely the *admitted
   external facts*, which (1)+(2) gate.

**Monotonicity / non-monotonicity.** The shipped reasoner is **no-retraction, stratified
negation-as-failure** (sparq-reason; the AC design's stratification discipline). Two
consequences the design must honour:

- Admission rules must be **stratified ahead of** derivation: all admission decisions for a
  predicate complete before any derivation rule reads it, so scoped-NAF stays sound. A
  two-unbound-atom admission rule once blew up the seeding in a related context; every trust
  rule must keep each seeding direction **one-side-bound**.
- Revocation/expiry is *non-monotone* (a previously-admitted fact is withdrawn). Because the
  materialiser is full-re-run-on-change (epoch-bump-on-rematerialise, no incremental
  maintenance yet), revocation propagates by re-materialisation, *not* by retraction inside a
  single run. v1 must therefore re-evaluate on credential expiry/revocation and on trust-graph
  change; incremental admission maintenance is a named follow-up, not shipped (G5/§8).

**Entailment laundering (G3).** If a source is trusted for `schema:age` but the reasoner can
*derive* age from an untrusted `schema:birthDate`, does the derived age inherit trust?
**v1 admits only directly-attested facts of a trusted type** — no laundering of derived facts
through the trust boundary. Trust-annotation *propagation* through the closure (the
annotated-RDF/semiring extension to sparq-reason) is a research alternative, explicitly out
of v1 scope.

**Termination.** Admission rules are guarded conjunctions over signed-graph facts + trust
facts; with predicate-IRI typing and one-side-bound seeding they terminate. SHACL-shape typing
(`trust:forShape`) runs the shipped, terminating `sparq-shacl` validator as a side condition,
not as recursive rule expansion.

**Issuer-key distribution.** A trust rule names `trust:issuerKey` as a DID/verification-method
IRI. **sparq has no DID resolver today** — `did:example:...` appears only as opaque example
IRIs in the zk registry. A pluggable `did:key`/`did:web` resolver feeding the existing
`sparq-zk` signature check is a required prototype phase (P2, §6); until then, keys are
supplied directly (the `zk:issuerPublicKey` material), which is honest but not yet
DID-resolved.

### 3.4 The subject-to-requester join (holder binding)

The credential says `<Jesse> schema:age 25`; the requester authenticated as some WebID. The
admission rule must **bind the credential subject to the authenticated principal** — otherwise
anyone could present Jesse's age credential. This is a hard join and is its own correctness
obligation: v1 requires the credential's `credentialSubject` (or a holder-binding proof, e.g.
SD-JWT-VC `cnf`) to equal `Session.agent`. Presenting a third party's credential without
holder binding must **not** admit the fact.

### 3.5 Conflict resolution

Two trusted issuers may attest contradictory values (`age 25` vs `age 17`). The derivation
stratum inherits ACP's normative **deny-overrides** for the *access* decision, but the
*admission* of contradictory *facts* needs its own rule. v1 recommendation: admit both
issuer-tagged facts and let the derivation be *conservative* (if any trusted attestation fails
the predicate, and policy is "all trusted sources must agree", deny). The general
trust-conflict semantics (preference order over sources, threshold/k-of-n) is a designed-only
extension, flagged in §7.

## 4. Capability delegation for human AND AI agents

### 4.1 Two trust roots, one derivation

The surveyed prior art has **two unreconciled trust roots**: VC/issuer-rooted *attribute*
trust (which the §3 admission stratum handles) and ZCAP/UCAN *resource*-rooted *capability*
trust (delegation chains). A delegation is *also* a trusted-source-attested statement, so it
rides the same admission rail: a ZCAP-LD/UCAN delegation document is a signed graph; admitting
it means *verifying the chain to a root the trust graph anchors* and treating its caveats as
trust-graph conditions.

### 4.2 Attenuation and on-behalf-of

- **Attenuation (monotone narrowing).** Each delegation hop may only *restrict* (ZCAP-LD: a
  child capability's `allowedAction ⊆ parent`, `expires ≤ parent`, target only
  suffix-narrowed; UCAN: each hop restates or attenuates; SPKI tag-intersection). The trust
  graph models a delegated capability as RDF whose caveats are *conditions in the admission
  rule* — and the engine must *enforce* monotonicity, never derive a broader grant than the
  chain permits.
- **On-behalf-of / AI agents.** An AI agent is a distinct principal (its own DID/key) holding
  an **attenuated child** of its human principal's authority. The correctness invariant —
  absent from every surveyed AC model and the load-bearing delegation-security claim — is the
  **intersection rule**: an agent's effective permissions = `delegator-permissions ∩
  agent-allowed-scope`, enforced per request. "Human vs AI" is just an attested attribute /
  caveat on the delegate. This maps to OAuth Token Exchange (RFC 8693) on-behalf-of and the
  Stanford "Authenticated Delegation" model; the trust-graph value-add is rendering the
  delegation as RDF that reasons with the *same* `.acl` rules.

### 4.3 Confused-deputy resistance

Object-capability discipline (designation = authority carried *with* the request, no ambient
authority) is what resists the confused-deputy problem. The trust graph must therefore (a)
carry the delegation chain *with the invocation* (not look it up ambiently), (b) bind the
invocation to the holder's key (GNAP-style key-proofing / DPoP), and (c) keep the agent's
authority provably `⊆` the delegator's via the intersection rule. PROV-O lineage
(`sparq-prov`) records the chain for audit.

### 4.4 Revocation under delegation

Revoking a delegated capability is harder than revoking a credential (the chain may be deep).
v1 consults a status list per link and re-materialises on change; deep-chain incremental
revocation is a named follow-up. `crates/sparq-zk-compose/src/revocation.rs` already models a
revocation primitive to build on.

## 5. The superset-of-ZKaps argument (honest and precise)

The brief asks to show the model expresses a **superset of ZKaps' features**. The honest,
defensible version splits the claim into two layers — one provable from the authorisation
model, one that requires composition with the (un-audited) ZK estate.

### 5.1 Two distinct things called "ZKaps"

- **ZKAPs (Least Authority / Privacy Pass, RFC 9576-9578):** single-use, *unlinkable*,
  *anonymous* authorisation tokens. A token attests **one bit** ("the holder passed
  attestation") with strong unlinkability (origin/issuer/redemption). It carries **no
  attributes, no predicates, and is deliberately non-delegable.**
- **"ZKaps" as the maintainer's term** for a *capability-VC* / ZK-attested access grant
  (predicate-proof over a hidden attribute, e.g. "prove age ≥ 18 without revealing 25").

The argument must be precise about which is meant, because they pull in opposite directions:
ZKAPs are *more private* and *less expressive*; capability-VCs are *more expressive* and
*conditionally private*.

### 5.2 What the trust graph subsumes (authorisation-model layer — defensible)

On the **policy/derivation axis**, the trust graph is strictly more expressive:

- A ZKAP-gated or capability-token-gated access is the special case where the trust rule is
  "admit a one-bit attestation from issuer S and grant" — an N3 rule is a strict superset of a
  fixed-predicate token check (an arbitrary `{…}=>{…}` rule subsumes Biscuit-Datalog, which
  subsumes a single predicate).
- The trust graph *also* admits arbitrary attribute statements that **merge with `.acl`
  rules** (the age>18 derivation), which a pure capability/one-bit token **cannot** express.
- Delegation (ZCAP/UCAN attenuated chains) is *also* subsumed, as a species of
  trusted-source-attested statement (§4).

So **at the authorisation-model level, the trust graph expresses a superset of the *features*
of ZKAPs and of capability-VCs** — this is the claim that is defensible from the model.

### 5.3 What it does NOT get for free (privacy/unlinkability layer — caveated)

A **plain VC-presentation** model does **not** preserve ZKAPs' *unlinkability/anonymity*. When
a credential is admitted in the clear, the verifier learns the exact value (`age 25`, not just
"≥ 18") and can correlate presentations. To **match** ZKAPs' privacy property, the trust-graph
*derivation* must be discharged by a zero-knowledge proof — "prove `canAccess` was derived
without revealing the underlying facts" — composed with sparq's ZK estate
(`sparq-zk`/`sparq-zk-compose`). That estate is **research-stage and NOT externally audited**:
the v1 verifier is remediated and internally re-audited but external accredited-cryptographer
sign-off is **pending** (`sq-qhy4`), and even the *in-the-clear* issuer-key check in
`crates/sparq-zk/src/sig.rs` **discloses which issuer signed** (a documented privacy leak;
the hidden-issuer set-membership upgrade is not yet reached). `sparq-mpc` is honest-majority
semi-honest only.

**Therefore the honest framing for the WGs is:**

> The trust graph expresses a superset of ZKAPs' features **at the authorisation/derivation
> layer** (attribute-conditioned grants, arbitrary-predicate rules, delegation — none of which
> a one-bit token expresses). It does **not** provide ZKAPs' unlinkable/anonymous *presentation*
> property for free; that property requires composing the derivation with sparq's ZK estate,
> which is designed/prototyped but **not yet a proven, externally-audited guarantee**. The
> privacy half is a research hypothesis under live caveats, not a settled result.

This is a *strictly weaker and true* claim than "superset of ZKaps", and is the one to take to
the working groups. See §7 for the adversarial-review limitations.

## 6. Prototype plan (decomposed; each phase a future bead)

Build on sparq's existing estate; do not add a new engine. Sequencing: **P1→P2→P3→P4** is the
core spine; **P5, P6, P7** depend on P3/P4; **P7 (privacy) is hard-gated on `sq-qhy4`**.

1. **P1 — Trust-graph vocabulary + semantics note (design-review).** Pin the `trust:` terms
   (§2.3), the admission/derivation two-stratum semantics, the degenerate-`.acl` equivalence,
   and the strict-additivity property. Doc + vocab only; no code. *Blocks all prototype
   phases.*
2. **P2 — DID resolution + issuer-key binding (prototype).** Pluggable `did:key`/`did:web`
   resolver feeding issuer keys into the existing `sparq-zk` signature check. Closes the
   "no DID resolver" gap. *Blocked-on: P1.*
3. **P3 — Claim-level admission stratum (prototype, the core deliverable, closes the `acp:vc`
   gap / G1).** N3 admission rules + Rust wiring that verify an issuer signature over a
   credential's RDFC-1.0 commitment, check `trust:trustsSourceFor`, freshness and the
   subject-to-requester holder binding, and inject admitted facts (issuer-tagged) ahead of the
   sparq-solid materialiser. Reuse `sparq-canon`, `sparq-zk/sig`, the `solidx:`/`urn:sparq:`
   reserved-predicate guards. *Blocked-on: P1, P2.*
4. **P4 — Trust-document storage/authoring model (prototype, G2).** Where trust rules live
   (server vs per-`.acr`), how they are Control-gated, versioned, and revoked; the admission
   cache key (evidence hash + policy version) composed with the sparq-solid epoch cache.
   *Blocked-on: P1.*
5. **P5 — Delegation stratum for human + AI agents (prototype, G3).** ZCAP-LD/UCAN chain
   verification rooted at the resource, monotone attenuation, the agent⊆delegator intersection
   invariant, confused-deputy resistance, PROV-O audit. Build on
   `sparq-zk-compose/revocation.rs`. *Blocked-on: P3.*
6. **P6 — Live status/revocation + minimal justification (prototype, G5/G6).** Fetch+decode a
   W3C Bitstring Status List; gate derivations on validity; emit a *minimal* PROV-O
   justification subset for each grant (sparq has PROV-O lineage but not minimal proof trees).
   *Blocked-on: P3.*
7. **P7 — Privacy/ZK admission feasibility (design + caveated prototype, G4).** Show the
   trust-graph derivation can run under sparq's ZK estate to give ZKAP-equivalent unlinkable
   predicate access; selective disclosure via BBS+ as a parallel cryptosuite once in-circuit
   cost is measured. **Hard-gated on external sign-off `sq-qhy4`; stays designed/research,
   never a v1 guarantee.** *Blocked-on: P3 + `sq-qhy4`.*
8. **P8 — Cost/decidability spike (prototype).** Bound admission-rule evaluation cost and
   confirm one-side-bound seeding everywhere; work-box timings are non-canonical. *Blocked-on:
   P1–P4.*

### 6.1 LWS/Solid-WG proposal framing

Take to the WGs the **weaker true claim**: the trust graph is the *formal semantics ACP's
unimplemented `acp:vc` always needed* — the missing "which issuer is trusted for which claim"
predicate — positioned as **strictly additive** to WAC/ACP (no trust graph ⇒ unchanged
behaviour) and **below** ODRL usage control (`crates/sparq-policy`). Present per-(source,
statement-type) trust as the *one new primitive*; present claim-level admission as the unifier
of ACP-VC / ZCAP / ABAC; present ZK-private admission and AI-agent delegation as the
*least-mature* parts, explicitly behind the privacy caveats. Whether to land it as new vocab,
an ACP profile, or both is an open WG question (§8).

## 7. Honest limitations & comparison

*(An adversarial review will expand this section; it is deliberately left prominent.)*

- **Privacy is not free and not proven.** §5.3: a plain VC-presentation discloses the exact
  attribute value and is linkable; matching ZKAPs' unlinkability requires the ZK estate, which
  is **not externally audited** (`sq-qhy4` pending) and whose in-the-clear issuer check
  *leaks which issuer signed*. MPC is semi-honest only. No line here asserts a settled
  ZK/MPC privacy or soundness property.
- **The superset claim is authorisation-model-level only.** It is *not* a standalone
  cryptographic superset; the unlinkable-presentation half is composition with the un-audited
  ZK layer (§5).
- **Re-opening the §2.4 boundary is the principal soundness risk.** The argument in §3.3 holds
  *only* if admission verifies real signatures (never self-asserted trust triples) and
  enforces statement-type scoping; a bug there is a privilege-escalation, not a cosmetic
  defect. This needs adversarial forgery tests mirroring the existing
  `acp_forged_*_in_acr_document_does_not_grant` suite.
- **Decidability/cost bounds are open.** Trust rules must keep every seeding direction
  one-side-bound; a two-unbound-atom rule has blown up seeding before. No formal complexity
  bound is proven here (P8).
- **Revocation/freshness is full-re-run, not incremental.** Stale-grant ("new enemy") risk is
  bounded by re-materialisation, not by retraction; incremental/temporal maintenance is not
  shipped.
- **No DID resolver today.** Issuer keys are example IRIs; P2 is a prerequisite for the
  end-to-end story.
- **Entailment laundering is excluded by fiat in v1** (directly-attested facts only); the
  trust-propagation alternative is unbuilt.
- **AI-agent delegation standards are early/moving** (draft-klrc-aiagent-auth, OAuth
  on-behalf-of drafts, GNAP); the intersection invariant and default-tight attenuation are
  sparq-local design choices, not adopted standards.
- **Conflict resolution beyond deny-overrides + conservative-admission is designed-only.**

## 8. Open questions for the maintainer

1. **Trust-document scope:** server-wide, per-`.acr`, or both (recommendation: both,
   per-`.acr` narrowing)?
2. **Statement-type granularity:** predicate-IRI (v1, decidable) vs SHACL shape (expressive)
   vs graph pattern?
3. **Superset claim strength for the WG:** the weaker true one (recommended) or a stronger
   framing the maintainer wants to defend?
4. **AI-agent delegation:** a new principal *kind* or just a capability chain with a
   human/AI attribute (recommendation: the latter — fewer new primitives)?
5. **Privacy posture for v1:** in-the-clear admission now with ZK later (recommended,
   honest), or ZK from the start (blocks on `sq-qhy4`)?
6. **Obligations:** reuse the ODRL bridge (`sparq-policy`) for duties/usage-control above the
   access decision, or keep them separate?
7. **Standards-track shape:** new `trust:` vocabulary, an ACP profile that seeds matchers from
   `acp:vc`/`acp:issuer`, or both?

## 9. Prior art and sources

### 9.1 jeswr's own lws-acp design (build on, credit, and diverge)

This design **builds on jeswr's `lws-acp/docs`** (<https://github.com/jeswr/lws-acp/tree/main/docs>:
`datalog-core.md`, `layering.md`, `expressivity-matrix.md`, `model-encodings.md`,
`layering-lws-context.md`), digested in the in-flight companion record
`research/trust-graph-prior-art-lws-acp.md` (PR #943). He noted some of it "could be crap";
crediting the good and dropping the bad with a one-line reason:

**KEEP (sound, reused):**

- The **Layer-0 / Layer-4 split** — truth-condition rules (Layer 0) separated from
  evidence/credential *admission* (Layer 4). This design's two strata (derivation / admission)
  *are* that split, made concrete on the shipped reasoner.
- **Stratified Datalog** as the technology-neutral core; every AC paradigm is a predicate
  profile. Matches the shipped stratified-NAF reasoner.
- The **trusted-issuer-guarded admission rule** pattern (PERMIS-style `roleAssign :- attrCert,
  issuedBy, trustedIssuer, fresh, notRevoked`) — generalised here from *global* `trustedIssuer`
  to *per-statement-type* `trustsSourceFor`.
- The **per-model encodings** ("Datalog Rosetta stone") as the evidence that one core subsumes
  ACL/RBAC/ReBAC/Zanzibar/Capability — the backbone of the superset argument's
  authorisation-model half.
- The **admission cache key** = evidence hash + policy version (composes with the sparq-solid
  epoch cache).

**REWORK:**

- `trustedIssuer` (global) → `trustsSourceFor` (per-statement-type). *Reason: global issuer
  trust cannot say "gov for age, not for role" — the whole point.*
- 15 layers → **2 normative surfaces** (admission + derivation). *Reason: the 15-layer stack
  re-specifies off-the-shelf protocols (OIDC/UMA/GNAP/TLS/ZCAP-LD) the WG should reference, not
  re-standardise.*
- `lws:allOf`/`lws:not` combinators → reuse ACP `acp:allOf`/`acp:noneOf`. *Reason: they collide
  with the ACP vocab already shipped in `sparq-solid`; reuse, don't fork.*
- Closed-world deny → **split into open-world fact admission + closed-world access derivation**.
  *Reason: admitting attested facts is open-world; the access decision stays closed-world
  fail-closed (D4).*

**DROP (with one-line reason):**

- **OSI-style layer numbering as normative** — *presentational scaffolding, not semantics; a WG
  will not standardise a 15-layer OSI analogy.*
- **Re-specifying OIDC/UMA/GNAP/TLS/ZCAP-LD** — *off-the-shelf; reference them, do not
  re-standardise them.*
- **The unqualified "superset-of-ZKaps" claim** — *over-stated; the honest split (§5) replaces
  it. The expressivity-matrix doc's own finding is "no single most-expressive model and no
  supremacy claim", which undercuts any unqualified superset.*
- **`layering-lws-context.md`'s deferral of the LWS binding and delegation** — *it admits it
  does not map layers to Solid concepts and defers delegation; this design supersedes it by
  binding to the shipped sparq-solid substrate and specifying delegation (§4). Kept idea: the
  admission cache key.*

### 9.2 In-flight companion prior-art surveys (this design synthesises them)

Six prior-art surveys were produced for this design and are **open PRs against `main`** at
time of writing (not yet merged), cited by path + PR:

- `research/trust-graph-prior-art-lws-acp.md` — PR #943 (jeswr's lws-acp digest).
- `research/trust-graph-prior-art-odrl.md` — PR #942 (ODRL evaluation frame; the missing
  admission theory).
- `research/trust-graph-prior-art-*` (RBAC/ABAC/NGAC/ReBAC + trust-management) — PR #945.
- `research/trust-graph-rdf-integrity-prior-art.md` — PR #946 (named-graph/quad trust,
  RDFC-1.0, N3 scoped reasoning).
- `research/trust-graph-prior-art-semweb-ac.md` — PR #947 (WAC/ACP/`acp:vc`, AIR,
  Protune/PeerTrust, Shi3ld, Kirrane survey).
- `research/trust-graph-capabilities-delegation-zkaps.md` — PR #949 (UCAN/ZCAP/Macaroons/
  Biscuit/SPKI/Privacy-Pass/anon-creds).

### 9.3 In-repo substrate (verified for this design)

- `crates/sparq-solid/` — WAC/ACP as N3 rules (`rules/*.n3`), `acp:issuer` dimension
  (`src/authindex.rs`), `AccessProvenance` + `solidx:` reserved-predicate guard
  (`src/loader.rs`, `src/provenance.rs`), materialised `<urn:sparq:auth>` view
  (`src/materialize.rs`). `acp:vc` has **zero** occurrences.
- `crates/sparq-reason/` — stratified-NAF N3 reasoner (the derivation engine).
- `crates/sparq-canon/` — RDFC-1.0 canonicalisation (W3C Rec) as a public API.
- `crates/sparq-zk/`, `crates/sparq-zk-compose/` — per-named-graph RDFC-1.0 commitment +
  issuer signature (`src/{canon,commit,sig}.rs`), `zk:issuerKey`/`zk:statusList` registry
  (`src/registry.rs`), revocation (`src/revocation.rs`). **Not externally audited (`sq-qhy4`);
  the in-the-clear issuer check discloses which issuer signed.**
- `crates/sparq-policy/` — ODRL 2.2 subset + fail-closed evaluator (the usage-control layer
  above access control).
- `crates/sparq-prov/` — PROV-O lineage for derived data.
- `research/solid-access-control-design.md` (§2.4 boundary, the shipped AC substrate);
  `research/feature-research-odrl-policy.md`; the `zk-*` and `mpc-*` design records.

### 9.4 External standards and literature

- Solid WAC <https://solidproject.org/TR/wac>; ACP <https://solidproject.org/TR/acp>;
  Solid-OIDC <https://solid.github.io/solid-oidc/>.
- W3C VC Data Model 2.0 <https://www.w3.org/TR/vc-data-model-2.0/>; VC Data Integrity +
  RDFC-1.0 <https://www.w3.org/TR/rdf-canon/>; VC-DI-BBS <https://www.w3.org/TR/vc-di-bbs/>;
  Bitstring Status List <https://www.w3.org/TR/vc-bitstring-status-list/>; DID Core
  <https://www.w3.org/TR/did-1.0/>.
- W3C-CCG ZCAP-LD <https://w3c-ccg.github.io/zcap-spec/>; UCAN
  <https://ucan.xyz/specification/>; Macaroons (Google, NDSS 2014); Biscuit
  <https://github.com/eclipse-biscuit/biscuit>; SPKI/SDSI RFC 2693; GNAP RFC 9635;
  Privacy Pass RFC 9576-9578.
- NIST SP 800-162 (ABAC); ANSI/INCITS 359 (RBAC); INCITS 565-2020 (NGAC); NIST SP 800-178
  (XACML vs NGAC); OASIS XACML 3.0 + Administration & Delegation Profile.
- Li, Mitchell, Winsborough, *Design of a Role-Based Trust-Management Framework* (IEEE S&P
  2002) — the formal antecedent (attested statements → Datalog, cross-domain delegation).
- Carroll, Bizer, Hayes, Stickler, *Named Graphs, Provenance and Trust* (WWW 2005); Zanzibar
  (USENIX ATC '19); Kirrane, Mileo, Decker, *Access control and the RDF: a survey* (Semantic
  Web 8(2), 2017).
- More, Ramacher, Alber, Herzl, *Extending Expressive Access Policies with Privacy Features*
  (2022) — closest published prior art to the privacy half (predicate proofs over private
  attributes).
- OAuth Token Exchange RFC 8693; draft-klrc-aiagent-auth; Stanford *Authenticated Delegation
  and Authorized AI Agents* (arXiv 2501.09674).
- ZKAPs (Least Authority) <https://leastauthority.com/product-development/zkaps/>.
