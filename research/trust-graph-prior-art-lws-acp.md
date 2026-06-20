<!-- [OPUS-4.8] research record (design-for-review): prior-art digest of the maintainer's
     OWN lws-acp/docs work as primary input to the trust-graph design. Read-heavy, no
     implementation. -->

# Prior art for the trust-graph design — domain: jeswr's own `lws-acp/docs`

Status: **research / design-for-review** (no implementation). This record digests the
maintainer's **own earlier work** — the five design notes under
[`jeswr/lws-acp/docs`](https://github.com/jeswr/lws-acp/tree/main/docs) — as the *primary*
prior art for a proposed **trust graph**: the set of statements/rules a storage server uses
to decide **which sources it trusts for which access-control statements** (per-source,
per-statement-type), so that trusted-source-attested facts (e.g. a government VC
`<Jesse> <age> 25`) **merge with** `.acl`/ACP rules via reasoning to derive access.

The maintainer's framing is explicit and is honoured here: *"a lot of that could be crap."*
This doc is **critical, not slavish** — it says which lws-acp ideas are sound and reusable,
which to drop or rework, and **where the trust-graph design must diverge**, with reasons. It
also corrects two premises (see §0) and grounds every external claim in a cited spec.

Companion in-repo records this builds on (do not duplicate them):
[`research/solid-access-control-design.md`](solid-access-control-design.md) (the **shipped**
`sparq-solid` WAC/ACP materializer — the architecture record),
[`research/feature-research-odrl-policy.md`](feature-research-odrl-policy.md) (ODRL policy
bridge), and the `zk-*` records for the credential-privacy layer.

---

## 0. Two corrections to the brief's premises (honesty first)

The brief is largely right, but two premises need adjusting against the **actual code** and
the **actual ACP spec**:

- **C1 — "ACP can already do issuer-scoped access; the trust graph just adds VC claims."**
  Partly. `sparq-solid` **already ships** the `acp:issuer` dimension (sq-3jtd.6): a matcher
  can require that the OIDC IdP that vouched for a WebID is a specific issuer, and the
  session carries an asserted `issuer`
  ([`crates/sparq-solid/rules/acp-a.n3`](../crates/sparq-solid/rules/acp-a.n3),
  [`materialize.rs`](../crates/sparq-solid/src/materialize.rs); design
  [`solid-access-control-design.md`](solid-access-control-design.md) §3.6). **But** ACP's
  `acp:issuer` and the trust graph's "trusted source" are **not the same thing**: ACP's
  issuer is *the IdP that authenticated the requester's WebID*, matched by **IRI equality**
  against the request Context. The trust-graph "trusted source" is *the issuer of an
  attested **statement*** (a VC claim), and trust is **per-statement-type** (issuer X is
  trusted for `age`, not for `clearanceLevel`). ACP has **no** machinery for the latter — see
  C2. So the trust graph is **not** a thin extension of `acp:issuer`; it is a new layer.

- **C2 — "ACP's `acp:vc` matcher already reasons over credential claims."** **False.** The
  normative ACP spec (Solid Authorization Panel, *Access Control Policy*,
  <https://solid.github.io/authorization-panel/acp-specification/>) defines the `vc`
  attribute as matching on VC **type** only: *"In a Matcher, vc attributes define a set of
  types of Verifiable Credentials (VC), at least one of which MUST match the Context… A VC
  type present in the Context MUST be a valid VC presented as part of the resource access
  request."* There is **no** mechanism to constrain on a claim inside the VC (no `age > 18`),
  **no** normative notion of a *trusted issuer for a credential type*, and **no** capability
  delegation. `sparq-solid` does **not** implement `acp:vc` at all (verified: zero
  occurrences of `acp:vc`/`vc`/`credential` in `crates/sparq-solid/src` and `/rules` —
  it is the documented out-of-scope gap in `solid-access-control-design.md` §3.6/§7.4). So
  the claim-level reasoning the trust graph wants is **genuinely new** in both the ACP spec
  and the sparq codebase. This is the central gap (§5).

Net: the trust graph is **a superset of ACP's evidence handling**, not a tweak to it. The
lws-acp datalog notes (§1.2, §2.5) *already* sketched the right shape for that superset —
which is exactly why they are the most valuable prior art.

---

## 1. The lws-acp corpus — what each doc actually says

Enumerated via the GitHub contents API
(`https://api.github.com/repos/jeswr/lws-acp/contents/docs`); fetched 2026-06-20. Five files,
all reachable. Below: filename → the load-bearing ideas, then a verdict.

### 1.1 `layering.md` — a 15-layer (Layer 0–14) authorization stack

A deliberately OSI-style decomposition. The load-bearing layers for the trust graph:

- **Layer 0 — Logical Semantics.** "Abstract evaluation model (Assertion Graph, Targets,
  Conditions with predicates / quantifiers, Combining algorithms, closed-world deny)."
  Technology-neutral: "can be written in denotational form or **Datalog-style core**."
- **Layer 1 — Core Policy Meta-Model.** An RDF vocabulary (named **"LWS-APL CLv2"**):
  `PolicySet, Policy, Rule, Target, Condition, Predicate, Query, ShapeConstraint,
  ExternalCheck, Effect, Obligation`.
- **Layer 2 — Paradigm Profiles.** ACL / RBAC / ABAC / ReBAC / **Capability** / Lattice /
  OrBAC / Risk / **Zanzibar** profiles, each a *constrained predicate subset* of Layer 0.
- **Layer 3 — Domain Vocabularies.** Concrete IRIs (`storage:Read`, `storage:Write`,
  `mediaType`, `owner`, `group`, `purpose`).
- **Layer 4 — Evidence & Admission Layer (THE trust layer).** "**Trust Policy** governs
  which external artifacts become **Input Assertions**" from `VC, VP, JWT, MTLS identity,
  ZCAP, PrivacyPass, HTTP-Signature, local DB row`, after "freshness, revocation, binding
  (subject/app/resource), shape/namespace constraints, attribution," normalizing each to
  "RDF assertions + provenance/integrity metadata into the **Assertion Graph**."
- **Layers 5–9** — protocol bindings (UMA 2.0, GNAP, OIDC), session/tokenization (ZCAP
  chains / macaroons — "caveats map to predicates"), cryptographic binding (Ed25519,
  **BBS+ selective disclosure**, Merkle proofs), transport/caching, enforcement (PEP at the
  storage server; "**Delegated enforcement (object capabilities carrying attenuated
  rights)**").
- **Layers 10–14** — governance, observability (Obligations / `mustLog`, "**Decision
  explanation graph (minimal justification subset)**"), conformance, threat controls
  ("Determinism & totality for predicates (timeouts, failure ⇒ false)"; "Recursion depth &
  result-set cardinality limits"), evolution.

Author-flagged open questions (verbatim): predicate-recursion limits vs expressivity
(formal tractability bounds); canonical explanation representation (proof-tree minimality);
privacy-preserving logging; harmonizing selective disclosure with caching/replay; **unified
revocation across capabilities, credentials, and policies**.

**Verdict — partly reusable, mostly over-built.** The single load-bearing idea is the clean
separation between **truth conditions** (Layers 0–3: *what facts entail access*) and
**evidence admission** (Layer 4: *which facts the server is willing to believe, and from
whom*). That separation **is** the trust graph's spine and we keep it. But the 15-layer
taxonomy is **too elaborate to standardise**: the doc itself warns of "fragmentation through
uncontrolled profile proliferation," and most layers (5–14) are off-the-shelf plumbing
(OIDC, TLS, HTTP caching) that a Solid/LWS WG proposal should *reference*, not *re-specify*.
**Drop the layer numbering; keep the Layer-0/Layer-4 split as the only two new normative
surfaces.**

### 1.2 `datalog-core.md` — stratified Datalog as the common semantics

This is the **best** doc for our purposes. Key content (verbatim rule shapes):

- Fragment: **stratified Datalog with recursion and negation**. Recursion via
  `reachable(Rel,X,Y) :- edge(Rel,X,Y). reachable(Rel,X,Z) :- edge(Rel,X,Y),
  reachable(Rel,Y,Z).`; negation via `not revoked(Cap)`. "All external predicates must be
  **pure, terminating, and side-effect free**."
- Decision: query `permit(S,R,A)` / `deny(S,R,A)`; "Missing facts yield rule failure,
  aligning with **closed-world deny-by-default**"; combining via `deny-overrides /
  permit-overrides / first-applicable / risk-minimizing`.
- **Trusted attestations enter as extensional predicates** — the load-bearing pattern:

  ```prolog
  roleAssign(S, Role) :- attrCert(Ev, S, Role), issuedBy(Ev, Iss),
                         trustedIssuer(Iss), fresh(Ev), notRevoked(Ev).
  ```

  i.e. an attested fact is admitted into the rule body **only if** its evidence was issued
  by a `trustedIssuer`, is `fresh`, and `notRevoked`.
- Delegation/capabilities: `capDelegates(CapParent,CapChild)`,
  `capCaveat(Cap,CaveatType,Value)`, `delegationDepthLe`, `caveatsOk`, with
  `canUseCap(S,R,A) :- capability(S,Cap), capAuthorizes(Cap,R,A), caveatsOk(...),
  not revoked(Cap).`

**Verdict — reuse this almost wholesale; it is the trust graph's skeleton.** The
`trustedIssuer(Iss)`-guarded admission rule is *precisely* the "per-source trust" mechanism
the trust graph needs — and it is strictly **more** general than ACP's type-only `acp:vc`.
Two improvements the trust graph must make on top of it:
  1. **Make trust per-statement-type, not per-issuer-globally.** lws-acp's `trustedIssuer(Iss)`
     is a *global* "this issuer is trusted." The brief requires *per-statement-type* trust
     (`trustedFor(Iss, age)`, not `trustedFor(Iss, clearanceLevel)`). This is a one-predicate
     change — `trustedIssuer(Iss)` → `trustsSourceFor(Iss, Pred)` in the admission rule — but
     it is the whole point of the new design, so it must be first-class and named.
  2. **Reconcile "closed-world deny" with RDF's open-world data.** lws-acp asserts
     closed-world deny-by-default. The *access decision* should indeed be closed-world
     (absence of a grant = deny — `sparq-solid` already does this, design D4). But the
     *attested facts* (`<Jesse> age 25`) live in an **open** RDF world. The trust graph must
     pin the boundary: **fact admission is open-world (you only ever add believed facts);
     the access derivation over the admitted facts is closed-world.** lws-acp blurs this; the
     trust graph must state it (it maps cleanly onto `sparq-solid`'s already-shipped
     **stratified NAF discipline** — negation only over *complete* predicates,
     `solid-access-control-design.md` §1.4/§3.5).

### 1.3 `expressivity-matrix.md` — 14 AC models × 14 feature axes

Compares ACL, RBAC, AGDLP, ABAC, ReBAC, OrBAC, Cap-based, PERMIS, Lattice(BLP), Chinese Wall,
Risk-based, IDN, Apache Fortress, Zanzibar against axes incl. delegation/attenuation,
capability possession, recursion necessity, external-function reliance, revocation
granularity, **formal tractability**. Sketches an *informal containment* ordering
(non-recursive ⊂ +role-hierarchy ⊂ +general-recursion {ReBAC, Cap, Zanzibar, PERMIS} ⊂
+lattice ⊂ +numeric/external). **No supremacy claim** is made. **PERMIS** is the only model
with native "attribute issuers; delegation graphs"; capability possession is native only to
Cap-based.

**Verdict — useful as a checklist, not as architecture.** Reusable: the **axes** are a good
conformance checklist for "what must the trust graph be able to express" (delegation,
recursion, issuer-attribution, revocation granularity, tractability). The honest finding for
the brief's claim that the trust graph is *"a superset of ZKaps"*: the matrix shows there is
**no single most-expressive model** — every model is a *predicate profile* over the same
datalog core. So the defensible claim is **not** "trust graph ⊃ ZKaps" as models, but
"**the datalog core can encode ZKaps (and ZCAP, and ACP-VC) as one profile among several**,"
which `datalog-core.md` already demonstrates for capabilities. The standards-track doc should
make the *weaker, true* claim and drop any unqualified "superset" language. **PERMIS** is the
closest external prior art to the trust graph's issuer-attribution and deserves a citation.

### 1.4 `model-encodings.md` — concrete Datalog encodings per model

Gives the per-model rule bodies (`permit(S,R,A) :- aclEntry(R,A,S).` for ACL;
`reachable`+`permit(S,R,read) :- reachable(ownerOf,S,R).` for ReBAC; the PERMIS
`roleAssign … trustedIssuer …` rule for evidence; the `delegationDepthLe` recursion for
capability chains), plus the normalization to a common core:
`decision(S,R,A,permit) :- permit(S,R,A), not deny(S,R,A).` (deny-overrides). CLv2 mapping
uses `lws:Predicate, lws:allOf, lws:not`.

**Verdict — reuse as the worked-examples appendix.** These are the "Rosetta stone" that
proves the single datalog core subsumes ACL/RBAC/ReBAC/Zanzibar/capability — the strongest
evidence for the trust-graph-as-unifier pitch to the WG. **Caveat to fix:** `lws:allOf` /
`lws:not` collide in spirit with ACP's existing `acp:allOf`/`acp:noneOf` combinator
vocabulary, which `sparq-solid` already implements. The trust graph should **reuse ACP's
combinator vocabulary** where it overlaps rather than minting a parallel `lws:` set —
otherwise we fragment the very vocabulary we are trying to unify.

### 1.5 `layering-lws-context.md` — placing the stack in LWS/Solid

Frames the stack for "Linked Web Storage" but, by its own admission, "**does not explicitly
map layers to Solid concepts**" (storage servers, `.acl`, WAC, WebID) — it stays
storage-agnostic. It does name the trust vocabulary — "**Trust Policy vocab (anchors,
issuers, admission rules)**" — and proposes caching admission results keyed by
`(evidence hash, trust policy version)`. Crucially it admits **delegation (human↔AI,
human↔human) is "Not addressed"**, and capability attenuation is deferred future work.

**Verdict — the weakest doc; supersede it.** Its honest self-assessment ("not addressed,"
"left for subsequent documents") tells us where the *real* work is: the LWS/Solid binding and
the delegation story. The one durable idea is the **admission-result cache key**
`(evidence hash, trust-policy version)` — it composes perfectly with `sparq-solid`'s existing
**epoch-keyed session cache** (`solid-access-control-design.md` §4.2/§4.3): the trust-policy
version *is* a second epoch dimension. Keep that; rebuild the rest on the shipped sparq
substrate.

---

## 2. How this domain maps onto the trust-graph design

The trust graph = **lws-acp Layer 0 (datalog truth conditions) + lws-acp Layer 4 (evidence
admission), realised on the shipped `sparq-solid` N3 reasoner**, with three named additions.
The mapping is concrete because `sparq-solid` already runs N3 rules over an Assertion Graph
and already materializes a per-principal auth view:

| lws-acp concept | trust-graph realisation on sparq | status today |
|---|---|---|
| Layer 0 datalog core | N3 forward rules in `crates/sparq-solid/rules/*.n3` over a merged Assertion Graph | **shipped** (WAC/ACP) |
| Layer 4 `trustedIssuer`-guarded admission | a new **trust-graph stratum** that admits VC claims into the reasoning store *only* when `trustsSourceFor(Iss, Pred)` holds | **designed-only** (this doc) |
| per-statement-type trust | `trustsSourceFor(Issuer, Predicate)` triples — the "trust graph" itself, stored as ordinary RDF | **proposed** |
| `acp:vc` (type match) | superseded by claim-level admission; ACP `acp:vc` becomes a *special case* (admit any claim from a VC of type T) | **gap → §5** |
| capability delegation (`capDelegates`, caveats) | a delegation stratum reusing ZCAP-LD chain semantics, caveats→N3 conditions | **proposed → §5.3** |
| admission cache `(evidence hash, policy version)` | second epoch on the existing `SessionCache` | **shipped substrate** |
| Decision explanation graph | reuse the shipped **PROV-O lineage** (`crates/sparq-solid/src/provenance.rs`, sq-m3i0) | **shipped substrate** |

The **trust-graph stratum** slots cleanly into the shipped pipeline because `sparq-solid`'s
materializer is *already* a stratified, fail-closed N3 reasoner over a controlled Assertion
Graph with a hard content/reasoner boundary (`solid-access-control-design.md` §2.4) — the
exact security property a trust layer needs. The new stratum runs **before** the ACP strata:
it derives believed claims (`believes(<Jesse>, age, 25)`) from admitted VCs, and those
believed claims then feed an ABAC-style ACP/N3 rule `{?x age ?y. FILTER(?y>18)} =>
{?x canAccess <r>}`. This is the "VC merges with `.acl` via reasoning" the brief describes,
made precise.

---

## 3. What is sound and reusable vs what to drop (the critical verdict the brief asked for)

**Keep (sound):**

1. The **Layer-0 / Layer-4 split** — truth conditions vs evidence admission. *The* idea.
2. **Stratified Datalog as the common semantics** (`datalog-core.md`) — matches the shipped
   `sparq-reason` N3 engine and its NAF discipline exactly; near-zero adaptation cost.
3. The **`trustedIssuer`-guarded admission rule** as the trust primitive — generalise to
   per-statement-type.
4. The **per-model encodings** (`model-encodings.md`) as the proof-of-unification appendix.
5. The **admission cache key** `(evidence hash, policy version)` — composes with the shipped
   epoch cache.

**Rework (right idea, wrong form):**

6. `trustedIssuer(Iss)` → `trustsSourceFor(Iss, Pred)` (per-statement-type). One predicate,
   but it is the design's reason to exist.
7. The 15-layer stack → **two normative surfaces** (truth conditions + admission); reference
   the rest, don't re-specify it.
8. `lws:allOf`/`lws:not` → reuse ACP's `acp:allOf`/`acp:noneOf` combinators to avoid
   vocabulary fragmentation.
9. Closed-world deny → split into **open-world fact admission** + **closed-world access
   derivation**, pinned explicitly.

**Drop (crap, per the maintainer's own invitation):**

10. The OSI-layer **numbering** as a normative artifact — it invites profile proliferation
    the doc itself warns against.
11. Re-specifying off-the-shelf protocol/crypto/transport (Layers 5–8) — cite UMA/GNAP/OIDC/
    TLS/ZCAP-LD; do not re-draft them.
12. Any **unqualified "superset of ZKaps"** claim — replace with the true, weaker
    "one datalog profile encodes ZKaps/ZCAP/ACP-VC."
13. `layering-lws-context.md`'s deferral of delegation and the LWS binding — those are the
    *actual* deliverables; supersede the doc.

---

## 4. Gaps the trust graph must address (not solved by lws-acp or by sparq today)

- **G1 — Claim-level admission with per-statement-type trust.** No prior art (ACP, WAC, or
  lws-acp's *global* `trustedIssuer`) gives per-`(issuer, predicate)` trust over VC **claims**.
  This is the core deliverable.
- **G2 — The "trust graph" as an authored, versioned, revocable artifact.** Who writes
  `trustsSourceFor` triples, where do they live (a server-level trust document vs per-resource
  `.acr`), and how are they revoked? lws-acp flags "unified revocation" as open; sparq has no
  trust document at all yet. Needs a storage + authoring model (likely: a server-scoped trust
  graph + per-resource overrides, both ordinary RDF, both gated like `.acr`).
- **G3 — Delegation for human AND AI agents.** Explicitly *"Not addressed"* in lws-acp; ACP
  has **no delegation**; `sparq-solid` has none. ZCAP-LD (<https://w3c-ccg.github.io/zcap-spec/>)
  supplies chain + caveat semantics, and `datalog-core.md` shows the `delegationDepthLe` /
  `caveatsOk` encoding — but the **AI-agent** case (an agent acting *on behalf of* a human,
  with attenuated, auditable authority) is genuinely new and is where the standards-track
  novelty is. Needs: a delegation stratum + a principal model that distinguishes
  "agent-acting-as" from "agent-as-self."
- **G4 — Credential privacy.** A government VC `<Jesse> age 25` admitted *in the clear* into
  the server's reasoning store **discloses the exact age** to the server — the opposite of
  what a privacy-preserving age gate wants. The trust graph must define how a **claim is
  admitted without revealing it** (the ZK/`sparq-zk` estate: prove `age > 18` to the admission
  rule without revealing `25`). **Caveat (honesty gate):** sparq's v1 ZK verifier is
  remediated and internally re-audited but **external accredited-cryptographer sign-off is
  pending** (sq-qhy4), and `sparq-mpc` is honest-majority semi-honest only — so any "admit a
  claim under ZK" path is **designed/proposed, not a proven production guarantee**, and must
  be presented as such. This is exactly the "trust graph is a superset of ZKaps" intuition,
  but it is the **hardest** and **least-sound** part, not the easy win the phrase implies.
- **G5 — Decidability/cost under the open admission layer.** lws-acp leaves "tractability
  bounds" open. Admitting facts from many sources and reasoning to a fixpoint must stay
  bounded; the shipped measured baseline (`solid-access-control-design.md` §6: ACP
  materialization is sub-second at ~1.1k graphs, but a two-unbound-atom rule once blew up to
  117 s) is the cautionary precedent — the trust stratum's rules must keep every seeding
  direction one-side-bound.
- **G6 — Explanation / accountability.** lws-acp wants a "minimal justification subset";
  sparq ships PROV-O lineage but not *minimal* proof trees. A WG audience will ask "why was
  access granted?" — the trust graph should emit a justification naming the admitted VC, its
  issuer, and the trust statement that admitted it.

---

## 5. Recommendation

**Adopt lws-acp's Layer-0/Layer-4 datalog split as the trust graph's foundation, realised as
a new pre-stratum on the shipped `sparq-solid` N3 reasoner, with per-statement-type trust as
the one genuinely new primitive — and present it to the WGs as a *unification* of ACP-VC /
ZCAP / ABAC under one datalog core, NOT as a "superset of ZKaps."** The ZK-privacy and
AI-delegation pieces are real and valuable but are the *least-mature* parts and must be
shipped behind the existing privacy caveats.

Concretely: keep `datalog-core.md`'s admission rule, generalise `trustedIssuer(Iss)` to
`trustsSourceFor(Iss, Pred)`, reuse ACP's combinator vocabulary and `sparq-solid`'s
stratified-NAF + epoch-cache + content/reasoner-boundary substrate, and supersede the layer
taxonomy and the LWS-context doc with a two-surface normative model.

### Phased plan (each phase = a future bead the orchestrator can track)

1. **Trust-graph vocabulary + semantics note** — pin `trustsSourceFor(Issuer, Predicate)`,
   the open-world-admission / closed-world-derivation boundary, and the ACP-combinator reuse;
   one worked example (government age VC + `.acl` age gate → grant). Doc-only, WG-facing.
2. **Claim-level admission stratum (designed-only spec)** — an N3 pre-stratum that admits VC
   claims into the Assertion Graph gated by `trustsSourceFor` + freshness + revocation,
   feeding the existing ACP/ABAC strata; specify how it preserves the §2.4 content/reasoner
   boundary. No code; spec + soundness argument. (Addresses **G1**, closes the `acp:vc` gap.)
3. **Trust-document storage & authoring model** — where `trustsSourceFor` triples live
   (server-scoped trust graph + per-resource `.acr` overrides), how they are write-gated
   (reuse Control), and the revocation/versioning model (second epoch on the session cache).
   (Addresses **G2**.)
4. **Delegation stratum (human + AI agents)** — map ZCAP-LD chains + caveats onto the
   `delegationDepthLe`/`caveatsOk` datalog encoding; define an "acting-as" principal so an AI
   agent carries attenuated, auditable authority. Spec + soundness; the standards-novel piece.
   (Addresses **G3**.)
5. **Privacy-preserving admission (ZK) — feasibility spec, heavily caveated** — how a claim
   is admitted via a `sparq-zk` proof (`age>18` without revealing `25`) instead of in the
   clear; explicitly mark as designed/not-yet-externally-audited (sq-qhy4 pending). Bounds the
   "superset of ZKaps" claim to what is honestly achievable. (Addresses **G4**.)
6. **Justification / explanation output** — extend the shipped PROV-O lineage to emit a
   minimal "why granted" subgraph naming the admitted VC, issuer, and trust statement.
   (Addresses **G6**.)
7. **Cost & decidability validation** — once 1–3 are specced, a measured spike (one-side-bound
   rule discipline) confirming the trust stratum stays sub-second at pod scale, against the
   §6 baseline. NON-canonical work-box timing caveat applies. (Addresses **G5**.)

Phases 1–3 are the core and are sequential; 4, 5, 6 each depend on 2 and can run in parallel;
7 depends on 1–3.

---

## 6. Open questions that genuinely need the maintainer

- **Q1 (trust-document scope).** Is the trust graph **server-scoped** (one trust document the
  storage server controls), **resource-scoped** (per-`.acr`), or both with override
  precedence? lws-acp implies server-scoped ("Trust Policy"); the brief's `.acl`-merge framing
  could read either way. This decides G2's storage model.
- **Q2 ("superset of ZKaps" — how strong a claim for the WG?).** Do you want the standards
  doc to make the strong "superset" claim (which I'd push back on as not provable as stated),
  or the defensible "one datalog profile encodes ZKaps/ZCAP/ACP-VC"? My recommendation is the
  latter; confirm before I draft WG-facing text.
- **Q3 (AI-agent delegation model).** Is "AI agent acts on behalf of human" a *new principal
  kind* (acting-as) or a *capability chain* (human delegates a ZCAP to the agent)? Both are
  expressible; the choice drives the principal model in phase 4.
- **Q4 (privacy posture).** For phase 5, is admitting claims **in the clear** acceptable for
  v1 (server learns `age=25`), with ZK admission as a later opt-in — or is ZK admission a
  hard requirement from the start? This materially changes scope and the soundness story.
- **Q5 (relationship to ODRL).** `feature-research-odrl-policy.md` exists; should the trust
  graph's *obligations* (lws-acp's `mustLog`/`Obligation`) reuse the ODRL bridge, or stay in
  the access/trust vocabulary?

---

## Citations

- jeswr, **lws-acp/docs** (primary prior art): `datalog-core.md`, `layering.md`,
  `expressivity-matrix.md`, `model-encodings.md`, `layering-lws-context.md` —
  <https://github.com/jeswr/lws-acp/tree/main/docs> (fetched 2026-06-20).
- Solid Authorization Panel, **Access Control Policy (ACP)** —
  <https://solid.github.io/authorization-panel/acp-specification/> (the `vc`/`issuer`/`agent`/
  `client` matcher definitions; type-only VC matching; no delegation, no trusted-issuer model).
- W3C CCG, **Authorization Capabilities for Linked Data (ZCAP-LD) v0.3** —
  <https://w3c-ccg.github.io/zcap-spec/> (delegation chains + caveats; the delegation prior art).
- Solid, **Web Access Control (WAC)** — <https://solidproject.org/TR/wac>.
- Kirrane, Mileo, Decker, *Access control and the resource description framework: a survey*,
  Semantic Web 8(2), 2017 — <https://www.semantic-web-journal.net/system/files/swj1280.pdf>
  (query-rewriting vs materialized RDF access control; the hybrid sparq adopts).
- In-repo: [`research/solid-access-control-design.md`](solid-access-control-design.md) (the
  **shipped** WAC/ACP materializer, issuer dimension, content/reasoner boundary, measured
  baseline); [`crates/sparq-solid/rules/acp-a.n3`](../crates/sparq-solid/rules/acp-a.n3),
  [`crates/sparq-solid/src/materialize.rs`](../crates/sparq-solid/src/materialize.rs);
  [`research/feature-research-odrl-policy.md`](feature-research-odrl-policy.md).
- External claim-level prior art worth citing in the WG draft: **PERMIS** (attribute issuers +
  delegation graphs — the `expressivity-matrix.md` standout); BBS+ selective-disclosure VCs and
  ZK-SD-VCs (IOTA Identity docs) for the privacy posture in phase 5 — kept caveated per the
  pending external ZK audit (sq-qhy4).
