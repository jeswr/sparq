// [OPUS-4.8] sq-vvu9d — zkSPARQL: Zero-Knowledge Query Proofs over SPARQL.
//
// RESTART FROM THE CODEBASE (maintainer directive, 2026-07-04). This draft was re-authored
// from what is ACTUALLY in the sparq repository — the `sparq-zk` / `sparq-zk-compose` crates,
// the Noir circuits under `zk/`, the vendored security-properties ontologies under
// crates/sparq-trust/ontologies/zkp-sparql/ and crates/sparq-policy/ontologies/, and the
// code-synced SKILL.md / SECURITY.md surfaces. The pre-existing zksparql.org site and the
// earlier ISWC submission texts are DEPRECATED and were NOT used as a source; where those
// authors' PUBLISHED prior art is cited (the `sec-prop` security-properties vocabulary,
// ISWC 2025) it is cited as external related work, not inherited.
//
// Every candidate-normative clause names the code that realises it (crate + item), so a
// reviewer can check the text against the implementation. Details that exist in code but are
// pinned to a specific proving toolchain (the bb public-input byte layout) are marked
// descriptive / at-risk, not normative.
//
// HONESTY: the entire zkSPARQL estate is a research scaffold and is NOT externally audited
// (open external-audit gate sq-qhy4). An internal, single-model re-audit found the binding
// layer "sound as landed for the threat model the prior audit assumed", but that is explicitly
// NOT a production guarantee and does not replace external sign-off (SECURITY.md). This draft
// states that plainly and repeatedly; it must never be edited into claiming a settled
// production guarantee while sq-qhy4 is open. (Provenance: dispatched as Claude Opus 4.8 while
// Fable was unavailable; flagged for ZK re-review.)

#import "_lib/spec.typ": spec-head, sotd, intro-section, references, dfn, note, cite

#set document(title: "zkSPARQL: Zero-Knowledge Query Proofs over SPARQL")
#set text(size: 11pt)
#set par(justify: true)
#set heading(numbering: "1.")

#spec-head()

#intro-section("abstract", "Abstract")[
  zkSPARQL is a proposal for proving, in zero knowledge, that the answer to a SPARQL query is
  correct against one or more committed RDF graphs, without revealing them. This document
  describes the interfaces realised by the sparq reference
  implementation, read directly from its source: the committed data model (RDF Dataset
  Canonicalization with a Poseidon2 sponge commitment over the BN254 scalar field, and issuer
  attestation signatures over the Baby Jubjub curve); the supported SPARQL fragment, its
  circuit family, and a normative algebraic definition of the fragment; the #dfn[proof
  manifest] interchange object, whose member schema is transcribed here from the Rust type;
  the verifier-nonce challenge–response; the verifier's fail-closed obligation set and its four
  audit gates; the external trust anchors a relying party supplies; and the layered
  security-properties vocabulary with its ODRL admissibility profile. Parts that do not yet
  exist — a registered media type, a JSON-LD context, and a wire protocol — are marked as
  proposals. The entire scheme is research-grade: it has not been externally audited,
  and no production guarantee is claimed (see the Security and Privacy Considerations,
  section 17).
]

#sotd()

#intro-section("audit-status", "Implementation and audit status")[
  A reference implementation of everything marked *implemented* below exists in the sparq
  repository #cite("SPARQ") as two opt-in, unpublished crates — `sparq-zk` (commitment and
  attestation) and `sparq-zk-compose` (circuit family, manifest, verifier) — together with an
  adversarial ("forge") test suite. However, the implementation has *not* been reviewed by an
  external cryptographer: the external audit gate (tracked in-repo as *sq-qhy4*, P0) is open.
  An internal, single-model re-audit found the verifier's binding layer *sound as landed for
  the threat model its prior audit assumed*, but that finding is #strong[not] a production
  guarantee, was produced by an LLM agent, and does not replace external sign-off. Until
  sq-qhy4 closes, soundness and attestation are *not production-ready*; every positive security
  property of this scheme is at most a *claim*, and a passing verification is not a guarantee
  that the proven SPARQL statement holds against an adversarial prover. This draft never
  asserts otherwise, and section 12.3 makes the corresponding over-claim rule normative for
  annotations.
]

= Introduction

This section is informative.

Verifiable-credential ecosystems let a holder present signed RDF data to a verifier. zkSPARQL
extends that interaction from "show the data" to "prove a query over the data": a
#dfn[prover] evaluates a SPARQL query #cite("SPARQL11-QUERY") over RDF graphs that an
#dfn[issuer] has committed to and signed, and produces a #dfn[proof manifest] — a bundle of
zero-knowledge sub-proofs plus bindings — that a #dfn[verifier] checks without seeing the
underlying graphs. Example uses include proving that a credential attribute satisfies a
threshold `FILTER`, that a value was not revoked at an authoritative snapshot, or that two
hidden credentials agree on a join key.

The flow is challenge–response:

+ The verifier mints a fresh #dfn[verifier nonce] and hands it to the prover.
+ The prover evaluates the query over its committed source graphs and produces a proof
  manifest whose every sub-proof commits to that nonce.
+ The verifier checks the manifest against its own #dfn[trust anchors] (trusted issuer keys,
  an authoritative revocation snapshot, a holder registry, and its nonce store), enforcing a
  fixed, fail-closed obligation set (section 10).
+ Optionally, a policy engine decides whether the *method* used is admissible for the purpose,
  by reasoning over the security-properties vocabulary (sections 12 and 13).

This document describes the interfaces of that pipeline as realised in the sparq reference
implementation, so they can be reviewed, critiqued, and cited. It is an Unofficial Proposal
Draft; see the Status of This Document.

= Document scope and maturity

== What this document is — and is not

This document is an *architecture and interface overview* of the zkSPARQL query-proof
pipeline, transcribed from the reference implementation and published in specification form so
the design can be reviewed and cited. It is #strong[not] a specification from which an
independent party could build a provably-sound prover or verifier — soundness rests on the  // privacy-claims-allow: explicit anti-overclaim — negated usage ("NOT a spec from which a provably-sound verifier could be built"), not a settled soundness claim (sq-qhy4)
Noir circuits and the proving backend, neither of which is externally audited (section 17.1),
and several security-load-bearing encodings are pinned to a specific proving toolchain
(section 8.4). A reader needing a conformance target should read section 2.2 for the exact
clauses this draft fixes normatively, and section 4.3 for what may — and may not — be claimed
against it.

== The normative kernel

Only the following clauses of this document are candidate-normative, and the RFC-2119
requirement keywords of section 4.1 are confined to them. Each is stable by design intent and
implementable from this text together with the cited source:

+ the committed-data-model algorithm identities — RDFC-1.0 canonicalisation, the term/leaf
  encoding, and the per-graph Poseidon2/BN254 sponge commitment (section 6.1) — and the
  cryptosuite resolution rule (section 6.3);
+ the fragment boundary and its algebraic semantics (sections 7.1–7.2), the bounded-depth
  statement and its depth-disclosure requirements for property paths (section 7.5), the
  admitted FILTER-expression shape (section 7.6), and the ban on representing a disclosed-base
  entailment re-check as a zero-knowledge proof (section 7.4);
+ the manifest type identifier and the trust-status of each declared member (sections 8.1–8.2);
+ the verifier-nonce discipline (section 9);
+ the verifier's fail-closed discipline: fail closed on any check that cannot be completed
  (section 4.3), the prefilter is not verification (section 10.2), a self-declared circuit
  identifier is never sufficient (sections 7.3 and 10.3), and first-failure rejection with no
  partial results and no warning downgrades (section 10.6);
+ the externality of every trust anchor, including the key-set subset rule (section 11);
+ the vocabulary scope and over-claim rules (sections 12.2–12.3), the provenance-only reading
  of `zk:sourceCryptosuite` (section 15), and the relying-party audit-status obligations and
  deployment cautions (sections 17.1–17.2 and 17.5–17.6);
+ the ODRL admissibility profile rules (section 13).

Everything else — in particular the public-input byte layout of section 8.4 and the
one-line obligation glosses of section 10 — is *descriptive* of the reference implementation
and carries no conformance force. The implementation remains the source of truth for the
exact clause of each obligation.

== Toolchain-pinned and unaudited cores

The following are security-load-bearing but rest on artefacts this text does not, and cannot,
fix normatively:

+ the exact in-circuit constraint relation of each circuit of section 7.3 — realised in the
  Noir sources under `zk/` and proved with an external backend, neither externally audited;
+ the public-input byte layout of section 8.4, which is *empirically pinned* to one
  Barretenberg release and is therefore descriptive, not normative;
+ the manifest hash, because the canonicalisation the reference implementation applies before
  hashing (`ProofManifest::canonicalize`) is an implementation detail this draft records but
  does not standardise (section 8.1).

Until these are respecified toolchain-independently and audited, a second implementation cannot
be guaranteed byte-interoperable, and no production soundness may be claimed.

= Related work

This section is informative.

*Verifiable query evaluation over databases.* IntegriDB #cite("INTEGRIDB") and vSQL
#cite("VSQL") prove SQL query answers correct against a committed, outsourced database, and
ZKSQL #cite("ZKSQL") extends the guarantee to zero knowledge: the answer is proven correct
while the database's records stay hidden. zkSPARQL targets the same class of guarantee for
RDF and SPARQL, with three structural differences visible in the code: the data model is
graph-shaped, with blank nodes and per-graph canonicalisation (section 6); trust is rooted in
*issuer attestation signatures* over per-graph commitments rather than in a single data
owner's commitment, so proofs compose across many small signed graphs (credentials); and the
admissibility of a proof *method* is itself policy-controlled (section 13).

*Anonymous credentials and selective disclosure.* CL signatures #cite("CL02"), the BBS line
of multi-message signatures #cite("BBS04"), and the `bbs-2023` / `ecdsa-sd-2023`
Data-Integrity cryptosuites #cite("VC-DI") let a holder reveal a subset of signed attributes,
sometimes with simple predicates. zkSPARQL generalises the *statement language*: instead of
disclosing attribute subsets, the holder proves that a SPARQL query — joins, typed value
filters, revocation state — evaluates as claimed over the signed data (section 7), without
disclosing the data. It does not replace credential-level selective disclosure: ingest of
`bbs-2023` credentials is an explicitly deferred seam (section 15).

*Zero-knowledge proofs over RDF and SPARQL.* The annotation and admissibility layer of this
document (sections 12–13) extends the `sec-prop` security-properties vocabulary of Wright,
Shadbolt, Zhao, Zhao and Braun #cite("SEC-PROP") — prior published work of this document's
editor, vendored into the sparq repository under MIT with its provenance record. The research
agenda is stated in #cite("WRIGHT-DC25"). Braun, Wright and Käfer #cite("BWK26") prove
soundness of SPARQL query results via *selectively disclosed views* of the queried dataset — a
disclosure-based mechanism, where zkSPARQL keeps the source graphs hidden and proves algebra
evaluation in-circuit. The sparq estate described here is an engine-integrated implementation
with its own commitment scheme, circuit family, manifest format, verifier obligation set, and
admissibility layer; it is not a wire-compatible implementation of any of the above.

= Terminology and conformance

== Requirement keywords

The key words #strong[MUST], #strong[MUST NOT], #strong[REQUIRED], #strong[SHALL],
#strong[SHALL NOT], #strong[SHOULD], #strong[SHOULD NOT], #strong[RECOMMENDED], #strong[MAY],
and #strong[OPTIONAL] in this document are to be interpreted as described in #cite("RFC2119")
and #cite("RFC8174") when, and only when, they appear in all capitals, as shown here. Their
use is confined to the normative-kernel clauses enumerated in section 2.2; all other text is
descriptive.

== Terms

- A #dfn[committed graph] is an RDF graph #cite("RDF11-CONCEPTS") together with a
  cryptographic commitment to its canonical form (section 6.1).
- A #dfn[sub-proof] is a single zero-knowledge proof for one circuit of the family in
  section 7.3, carried inside a proof manifest.
- A #dfn[proof manifest] is the JSON interchange object of section 8 bundling sub-proofs,
  their public metadata, and bindings.
- A #dfn[trust anchor] is an input the verifier obtains out of band from the relying party —
  never from the manifest (section 11).
- A #dfn[holder] is the party that controls the credentials a proof draws on; a holder may
  be disclosed ("clear") or hidden behind a proof-of-knowledge tier (section 10.4).

== Conformance and the fail-closed rule

A #dfn[zkSPARQL verifier] checks proof manifests (sections 9–11); a #dfn[zkSPARQL prover]
produces them (sections 6–9); an #dfn[admissibility policy engine] evaluates whether an
annotated method satisfies an ODRL policy (sections 12–13). Over-arching every verifier
clause: a verifier #strong[MUST] fail closed — it #strong[MUST] reject the whole manifest on
any check it cannot complete, and #strong[MUST NOT] return a partial or best-effort result.
Because the exact clause of each obligation (section 10) lives in the implementation, this
draft states verifier obligations as *necessary* conditions; full verifier conformance is
defined against the reference implementation until a successor draft transcribes every clause.

#note[
  This document couples its normative force to the code deliberately. Every candidate-normative
  clause names the crate item that realises it, so "conformance to this draft" means "agrees
  with the cited reference behaviour". A future revision may lift individual clauses to
  implementation-independent normativity once they are audited.
]

= Threat model and security goals

This section defines the adversary model against which the mechanisms of sections 6–11 are
the mitigation, and what each claimed security property *means* for this scheme. It is placed
before the mechanisms deliberately: every obligation in section 10 exists to counter a
capability listed here. The properties themselves are *claims* — none is externally audited
(section 17.1).

== Parties and trust relationships

- The #dfn[issuer] holds a signing key and attests committed graphs (section 6.3). The
  verifier trusts an issuer's *key* exactly insofar as the relying party placed it in the
  external key set K (section 11). Whether the issuer's attested *content* is true in the
  world is out of cryptographic scope: attestation transfers trust, it does not create it.
- The #dfn[holder] / #dfn[prover] controls the credentials and produces manifests. The
  verifier extends it *no* integrity trust: everything the prover sends is adversarial input
  until checked — including the manifest's own declared `key_set`, `query`, and
  `status_snapshots` (section 8.2). Conversely the prover extends the verifier no privacy
  trust: the scheme's hiding goals exist because the verifier is assumed curious.
- The #dfn[verifier] acts for a #dfn[relying party], which supplies every trust anchor
  (section 11). The verifier is trusted by the relying party to enforce the full obligation
  set; a verifier that skips checks voids all guarantees silently, which is why the
  discipline is fail-closed.
- The #dfn[admissibility policy engine] (section 13) reasons over *declared annotations*,
  not cryptography; it is trusted only for policy evaluation (section 17.6).

== Adversary capabilities

+ *Malicious prover.* Controls manifest contents entirely and adaptively: it can submit
  arbitrary JSON, forged or mutated sub-proofs, proofs generated against substitute circuits
  or keys, manifests replayed from earlier sessions or other verifiers, its own `key_set`
  entries, non-canonical serialisations, and claims about constructs outside the fragment.
  Its goal is to make the verifier accept a false SPARQL statement (soundness break), to
  reuse a proof (replay), or to smuggle in an untrusted issuer (attestation break).
+ *Malicious or curious verifier.* Receives the manifest and chooses the nonce adversarially.
  Its goal is to learn anything about the committed graphs beyond the proven statement and
  the public inputs (hiding break), or to link two presentations by the same holder
  (unlinkability break — tracked as a vocabulary dimension, section 12.1, not a settled
  property).
+ *Colluding holder and issuer.* Can mint attestations for arbitrary content. The scheme
  does not defend the relying party against attested-but-false real-world content
  (garbage-in); it defends only the binding between what was attested and what is proven.
  Collusion confers no capability against *other* holders' privacy or other issuers' keys.
+ *Network adversary.* Transport is currently out of band and unspecified (section 14); a
  confidential, authenticated channel is assumed and the network adversary is otherwise out
  of scope until a transport binding exists.
+ *Quantum adversary.* Explicitly conceded: the post-quantum posture is a settled negative
  (section 17.3).

== Security goals

The table gives each goal's meaning *for this scheme*, the mechanism intended to enforce it,
and its honest status. "Claim" means: implemented and exercised by the reference test suite,
but not externally audited (gate sq-qhy4, section 17.1).

#table(
  columns: 4,
  align: (left, left, left, left),
  table.header[Goal][Meaning in this scheme][Primary mechanism][Status],
  [Completeness], [An honest prover holding graphs that satisfy the query, with valid
    attestations and a fresh nonce, can produce a manifest the verifier accepts.],
    [Circuit family (section 7.3); prover pipeline.], [Claim (unaudited).],
  [Soundness], [If the verifier accepts a manifest under trust anchors (K, snapshot,
    registry, nonce), then the claimed statement holds — in the sense of section 7.2 — over
    graphs whose commitments are attested by keys in K.], [The full obligation set and audit
    gates (section 10).], [Claim (unaudited); the hidden-holder tiers are explicitly *not yet*
    sound (section 17.2).],
  [Binding], [A commitment identifies at most one canonical graph; the prover cannot open it
    to different data — the standard binding notion for commitment schemes
    #cite("PEDERSEN91").], [Poseidon2/BN254 sponge commitment over the RDFC-1.0 canonical form
    (section 6.1).], [Claim (unaudited); fails against a quantum adversary (section 17.3).],
  [Hiding / zero-knowledge], [The manifest reveals nothing about the committed graphs beyond
    the proven statement, the public inputs, and the leakage dimensions declared in
    section 12.], [ZK proof system (Noir circuits, backend proving; section 7.3); commitment
    hiding.], [Claim (unaudited); leakage dimensions are tracked, not bounded (section 17.4).],
  [Replay resistance], [An accepting manifest is bound to a single-use verifier nonce and
    cannot be accepted twice, nor transplanted to another request.], [Nonce discipline
    (section 9); audit gate 4 (section 10.5).], [Claim (unaudited); degraded by a
    non-durable nonce store (section 17.5).],
)

== Out of scope

Issuer content veracity (see above); side channels beyond the declared leakage dimensions
(timing is not modelled); denial of service; transport security (until section 14 is
realised); the quantum adversary (settled negative); and availability. The known deviations —
the hidden-holder tiers, the optional dual-leaf value lane — are catalogued in section 17.2
rather than silently excluded here.

= Committed data model

== Canonicalisation, term encoding, and the graph commitment

Each source RDF graph is canonicalised and then committed. Read from `sparq-zk` (`encode` and
`commit` modules):

+ The graph #strong[MUST] be canonicalised with RDF Dataset Canonicalization (RDFC-1.0)
  #cite("RDF-CANON"), so that commitment values are independent of blank-node labelling and
  triple order. Leaf order is the canonical N-Quads (code-point-sorted) order.
+ Each canonical term #strong[MUST] be encoded to one field element as
  $"Enc"_t("term") = h_2("type_code", h_s("value"))$. Here $h_n$ denotes the Poseidon2 sponge
  hash of an $n$-element input over the BN254 scalar field #cite("POSEIDON2") #cite("BN06") —
  a fixed-width $t = 4$ permutation used at rate 3, with the capacity initialised to
  $n dot 2^64$ (the noir-lang/poseidon `Poseidon2::hash`) — so $h_2$ and $h_3$ are
  length-domain-separated by construction; $h_s$ is a Blake3 hash folded into a field element,
  and `type_code` is `1` for an IRI, `2`
  for a literal, and `3` for a blank node. For an IRI, $h_s$ ranges over the IRI string; for a
  literal, over its canonical N-Triples token (lexical form, language tag, and datatype); for a
  blank node, the encoding is $h_2("blank_code", h_2("salt"_G, "blake3"("canonical_label")))$,
  so blank-node identity is salted per graph (closing the cross-graph blank-node correlation
  channel of section 7.2).
+ Each canonical *triple* #strong[MUST] be encoded to one *leaf*
  $"leaf" = h_3("Enc"_t(s), "Enc"_t(p), "Enc"_t(o))$ — a single Poseidon2 sponge hash of the
  three term encodings.
+ The graph commitment `C(G)` #strong[MUST] be a single Poseidon2 sponge over the
  leaf sequence in canonical order — the same $h_n$ construction defined above, with $n$ the
  leaf count, so its length-bearing capacity gives domain
  separation per leaf count — one commitment per source graph. This is a sequential sponge, not
  a Merkle tree (a Merkle arrangement for very large graphs is a deferred deliverable).

#note[
  Editor's note — the encoding above is the default `string-canonical` method
  (`zk:poseidon2-rdfc10-v1`, section 6.2). The exact field-folding of a Blake3 digest and the
  Poseidon2 round parameters are pinned by the `sparq-zk` `poseidon2` and `encode` modules;
  this section fixes the structure (type-code layering, per-graph salt for blank nodes, the
  three-term leaf, the sponge over canonical-order leaves), which is what a reviewer needs to
  check the commitment is order- and label-independent.
]

== Commitment methods (configuration axis)

The committed-graph *method* — which leaf shape a graph was committed under, and therefore
which circuit family may verify it — is a closed, fail-closed configuration enum
(`sparq-zk` `commit::CommitmentMethod`). The methods are `string-canonical`
(`zk:poseidon2-rdfc10-v1`, the default and back-compatibility anchor), `dual-leaf`
(`zk:poseidon2-dualleaf-v1`, opt-in), and `value-only` (`zk:poseidon2-valuehook-v1`, an
off-by-default research/benchmark dial that is #strong[never] a production default).
`from_scheme_iri` returns nothing for an unknown IRI (no default). The `dual-leaf` and
`value-only` leaf encodings are only partly built and carry a documented value↔lexical
downgrade (INV-VL / gap CR-G8, section 17.2); they are opt-in and unaudited.

== Issuer attestation signatures

An issuer attests a committed graph by signing its commitment (`sparq-zk` `sig` module):

+ The attestation signature scheme is a Schnorr signature #cite("SCHNORR91") over the Baby
  Jubjub curve #cite("EIP2494") with a Poseidon2-derived challenge, identified by the
  cryptosuite IRI `https://sparq.dev/ns/zk#poseidon2-schnorr-v1`
  (`SignatureScheme::POSEIDON2_SCHNORR_V1_IRI`). The signed message binds the commitment and,
  as progressively bound variants, the per-graph salt, the status reference, and a holder
  binding.
+ A verifier #strong[MUST] reject an attestation whose cryptosuite identifier it cannot
  resolve. There is #strong[no] default cryptosuite: `from_cryptosuite_iri` returns nothing
  for an unknown IRI, so an unresolved suite is a hard failure, not a fallback (fail-closed).

= Query fragment and circuit family

== The supported SPARQL fragment

The provable fragment of SPARQL #cite("SPARQL11-QUERY") is small and is extended along a single
principle: a construct is admitted only when *result membership* (section 7.2) is *monotone* —
a disclosed witness stays valid as the world gains data — and its statement is realisable in
the fixed circuit family (section 7.3) without a hidden completeness claim. That is the
open-world-assumption-conforming, federation-free core; closed-world negation, whole-pattern
completeness, and federation are excluded on semantics, not on cost. The design record
`research/zksparql-fragment-extension.md` derives the disposition of every construct.

Each row of the table below carries one implementation tier, so a claim of coverage traces to
implemented behaviour or is labelled a proposal:

- *core* — provable end-to-end today (`sparq-zk-compose` `build` / `verifier`): BGP scans
  (per-graph commitment recompute, row soundness, in-circuit scan completeness); the value
  `FILTER` lanes — non-negative integer (`filter_int`), the integer-valued `xsd:double`
  fragment (`filter_f64`), signed integer (`filter_signed_int`), fixed-point `xsd:decimal`
  (`filter_decimal`), and, behind the off-by-default `dual-leaf` feature, the value-dictionary
  lanes (`filter_value_dl*`) — whose accepted *expression* shape is section 7.6; and the
  single-prover equality `JOIN` (`join_eq`) over hidden credentials, where the join term stays
  private;
- *gate* — accepted by the query-side fragment gate (`sparq-zk::verify::fragment_query`), which
  re-derives the extended structure from the query text, and, for bounded paths, backed by the
  realised `path_reach` circuit family (section 7.5) — but *not yet bound end-to-end*: the
  manifest schema and verifier dispatch that tie an extended query to sub-proofs are in
  progress, so the live stage-1 compose verifier still fails closed on every *gate* construct
  until that work lands;
- *proposal* — designed in the record, not yet in any gate.

#table(
  columns: 3,
  align: (left, left, left),
  table.header[Construct][Tier][Disposition and reason],
  [`SELECT` / `ASK`], [core], [Membership, resp. non-emptiness, of a solution mapping.],
  [BGP], [core], [Scan circuits (row soundness + per-scan completeness in-circuit).],
  [`JOIN`], [core], [`join_eq` hidden equality join; cross-graph blank-node exclusion (the "Q6"
    guard, section 7.2) retained.],
  [`FILTER`], [core], [Value comparison over the datatype lanes; monotone under
    error-as-unsatisfied. Accepted expression shape: section 7.6.],
  [`DISTINCT` / `REDUCED` / `LIMIT` / `OFFSET` / projection], [core], [Membership-indifferent
    modifiers (outer level only; a `LIMIT`/`OFFSET` *inside* a subquery is rejected).],
  [Property path: predicate `iri`, inverse `^p`, sequence `p1/p2`, alternative `p1|p2`],
    [gate], [Rewrites into BGP / `UNION` (section 7.5); no new statement.],
  [Property path closure `p?` / `p*` / `p+` over an atomic step], [gate], [Bounded-depth
    existence within a disclosed depth (section 7.5).],
  [`UNION`], [gate], [Per-solution branch attribution; eval is the set union of branch evals.],
  [`VALUES`], [gate], [Inline public rows re-derived from the query text; `UNDEF` cells are
    wildcards; a triple-term cell is rejected (no committed leaf lane).],
  [Subquery (nested `SELECT`)], [gate], [Monotone when composed of in-fragment operators; inner
    non-projected variables are existential and renamed apart.],
  [`BIND` (`Extend`)], [proposal], [Deterministic expression over the section 7.6 estate;
    fail-closed today.],
  [Extended `FILTER` expressions], [proposal], [The general expression estate (section 7.6).],
  [`FILTER EXISTS` (positive)], [proposal], [Monotone, but deferred until the SPARQL 1.2
    `EXISTS` substitution semantics is pinned.],
  [Negated property set `!(p1|…|pn)`], [proposal], [Monotone existence of a differently-predicated
    triple; deferred until after the `path_reach` family binds.],
  [`ORDER BY`], [OUT], [Membership-indifferent but implies an unproved top-k claim; may re-enter
    only with an explicit "order-not-proved" manifest flag.],
  [`OPTIONAL` (`LeftJoin`)], [OUT], [Non-monotone: an unbound optional side asserts no
    compatible extension exists — a closed-world claim. Rewrite to `JOIN` or a `UNION` of cases.],
  [`MINUS`], [OUT], [Closed-world set difference; non-monotone.],
  [`FILTER NOT EXISTS`], [OUT], [Closed-world negation; non-monotone.],
  [Aggregation (`GROUP BY`, `HAVING`, `COUNT`, `SUM`, `AVG`, `MIN`, `MAX`, …)], [OUT],
    [An aggregate value is a completeness claim over the whole pattern; composed-pattern
    completeness is not proved.],
  [`GRAPH`], [OUT], [Naming graphs discloses the attribution the trust model hides.],
  [`SERVICE`], [OUT], [Federation, excluded by the fragment principle.],
  [`CONSTRUCT` / `DESCRIBE`], [OUT], [Result form outside the membership property; instantiate
    templates client-side from a proved mapping.],
)

Outside the fragment, a prover #strong[MUST NOT] emit a manifest claiming coverage of an
excluded or not-yet-implemented construct, and a verifier encountering such a claim
#strong[MUST] reject it (fail-closed). The gate rejects every form not tiered *core* or *gate*
above with a fixed reason — additionally including `FROM` / `FROM NAMED` dataset clauses, a
closure over a non-atomic path (section 7.5), and a `LIMIT`/`OFFSET` inside a subquery — and
never silently drops or downgrades an unrecognised construct.

== Formal semantics of the fragment

This subsection is candidate-normative (section 2.2): it defines the fragment by mapping it
onto the SPARQL algebra of Pérez, Arenas and Gutiérrez #cite("PAG09"), as adopted by the
SPARQL 1.1 recommendation #cite("SPARQL11-QUERY"), over the RDF 1.1 graph model
#cite("RDF11-CONCEPTS") under simple entailment #cite("RDF11-MT"). It is the semantic anchor
for the correctness obligation `bind_query_correctness` (section 10.4).

*Data model.* Let I, B, and L be the pairwise-disjoint sets of IRIs, blank nodes, and
literals, and V a set of variables disjoint from all three. An RDF graph G is a finite set
of triples in (I ∪ B) × I × (I ∪ B ∪ L). Each committed graph (section 6.1) is one such
graph, fixed by its RDFC-1.0 canonical form.

*Solution mappings.* A solution mapping μ is a partial function from V to I ∪ B ∪ L, with
domain dom(μ). Two mappings μ1 and μ2 are *compatible* when μ1(v) = μ2(v) for every variable
v in dom(μ1) ∩ dom(μ2); their union μ1 ∪ μ2 is then itself a mapping.

*Grammar.* A fragment pattern P over the committed graphs G1, …, Gn is generated by the
stratified grammar:

```
S ::= BGP | Filter(C, S)
P ::= S | Join(S1, S2)
```

subject to: a `BGP` (a finite set of triple patterns over terms and variables) is evaluated
against *exactly one* committed graph; `C` is a value constraint drawn from the
datatype-bucketed comparison forms of section 7.1; and `Join` is the equality join of
section 7.1, whose two sub-patterns S1 and S2 are evaluated over *distinct* committed graphs
and share at least one variable. The stratification is deliberate and matches the realised
circuit family: join nesting (`Join` over a `Join` result) and filters over join results are
*outside* the fragment — every `FILTER` binds to a single scan's slot via a binding edge, and
each equality join spans exactly two scans. A manifest #strong[MAY] carry several
`Join(Si, Sj)` obligations over pairwise-distinct scan pairs (the `join_edges` vector).

*Evaluation.* The evaluation eval(P) is a set of solution mappings:

- eval(BGP over G) = the set of mappings μ with dom(μ) = vars(BGP) such that replacing each
  variable v in BGP by μ(v) yields a subgraph of G;
- eval(Filter(C, S)) = the set of μ in eval(S) such that μ satisfies C under the SPARQL 1.1
  operator semantics (section 17 of #cite("SPARQL11-QUERY")), with expression errors treated
  as *not satisfied*;
- eval(Join(S1, S2)) = the set of unions μ1 ∪ μ2 where μ1 is in eval(S1), μ2 is in eval(S2),
  and μ1 and μ2 are compatible.

*Blank nodes across graphs.* Blank-node identity is scoped to a single graph
#cite("RDF11-CONCEPTS"), and per-graph canonicalisation (section 6.1) does not — and cannot —
align blank-node labels *across* committed graphs. Cross-graph equality of blank nodes is
therefore semantically meaningless in this fragment: a union μ1 ∪ μ2 in which some variable
v ∈ dom(μ1) ∩ dom(μ2) is bound to a blank node is *excluded from eval(Join(S1, S2))* — since
S1 and S2 range over distinct committed graphs, such a binding would assert a cross-graph
blank-node identity. This exclusion is the semantic minimum that the implementation's "Q6"
guard (`verify::recheck`, section 10.3) enforces.

*The correctness property.* The target property of `bind_query_correctness` (section 10.4)
is *result membership*: a manifest that discloses a solution mapping μ (or claims that a
solution exists) for a fragment pattern P over committed graphs G1, …, Gn is correct if and
only if μ is a member of eval(P) — respectively eval(P) is non-empty — as defined above.

#note[
  Editor's note — three boundaries of this definition are deliberate. (1) It is a *set*
  semantics; whether the implementation preserves duplicate-solution multiplicities (the bag
  semantics of #cite("PAG09")) is scoped out here. (2) Projection (`SELECT` variable lists)
  is defined at the query layer, not in the membership property. (3) Result *completeness* is
  proved for the in-circuit BGP scan but not for the whole pattern. None of these boundaries
  weakens the membership property, but all three must be settled before an
  implementation-independent successor.
]

*The extended constructs.* The grammar above is the end-to-end *core* (section 7.1). The
*gate*-tier extensions take their standard SPARQL 1.1 semantics under this same set semantics
and preserve the membership property: `UNION` (and the path alternative `p1|p2`) evaluates to
the set union of its branches, so a solution is attributed to the branch that witnesses it;
`VALUES` restricts eval to solutions compatible with an inline row, treating an `UNDEF` cell as
a wildcard; a subquery projects its inner evaluation existentially, so its non-projected
variables never bind an outer variable of the same name. The non-recursive path forms —
predicate, inverse (endpoint swap), and sequence (a fresh non-projected intermediate) — are the
SPARQL 1.1 path translation into BGP and `UNION` and add no new statement. The recursive
closures `p?`/`p*`/`p+` are the one construct whose proved statement is *weaker* than the
SPARQL operator — bounded existence rather than full reachability — and are defined separately
in section 7.5, where eval_k(P) ⊆ eval(P) makes the one-directional equivalence explicit.

== The circuit family

Each sub-proof is generated against exactly one circuit of a fixed, named family
(`sparq-zk-compose` `manifest::CircuitId`), each realising one operator instance of the
fragment (or one auxiliary statement: revocation, issuer-set membership, holder binding). The
circuits are authored in Noir #cite("NOIR") across three source estates under `zk/` — an
in-tree IEEE-754 library (`zk/ieee754`; canonical here, with a published face maintained at
`sparq-org/noir_IEEE754`), an XPath 2.0 function library (`zk/xpath`), and the
compiled per-property family workspace (`zk/compose`, one compiled binary per shape bucket) —
and proved with the Barretenberg backend (see section 16 on toolchain pinning). The family
(with the fixed shape parameters each variant carries) is:

#table(
  columns: 2,
  align: (left, left),
  table.header[Circuit identifier][Statement proved (descriptive gloss)],
  [`Scan { k, n, r }`], [A BGP scan over `k` committed graph(s) matches; `n`, `r` are the
    compiled slot/row capacity buckets (in-circuit commitment recompute + scan completeness).],
  [`FilterInt { d }`], [A non-negative integer-lane `FILTER` holds; `d` is the digit bucket.],
  [`FilterF64 { d }`], [An integer-valued `xsd:double`-lane `FILTER` holds.],
  [`FilterSignedInt { md }`], [A signed-integer-lane `FILTER` holds.],
  [`FilterDecimal { id, fd }`], [A fixed-point `xsd:decimal`-lane `FILTER` holds.],
  [`FilterValueDl` / `FilterValueDlF64` / `FilterValueDlDecimal`], [Value-dictionary-lane
    `FILTER` for integer / `xsd:double` / `xsd:decimal` (opt-in `dual-leaf`; section 17.2).],
  [`RevokeUnset { depth }`], [The revocation bit at a hidden index is unset in a committed
    status snapshot of the given Merkle depth.],
  [`HiddenIssuer { depth }`], [A committed graph was signed by *some* key in an attested key
    set, without disclosing which issuer.],
  [`HolderPok`], [Holder proof-of-knowledge (hidden-holder tier; see section 17.2).],
  [`HolderSet { depth }`], [Holder set membership (hidden-holder tier; see section 17.2).],
  [`JoinEq { n_a, n_b }`], [Two hidden credentials agree on an equality join key without
    disclosing it.],
)

The prover derives the compiled shape bucket from the data; the verifier *re-derives* the
circuit identifier from the statement each sub-proof is bound to (section 10.3), and a manifest
#strong[MUST NOT] be accepted on its self-declared identifier alone. An
out-of-bucket shape is a clean rejection, never a silently unprovable member.

The bounded-depth path family `path_reach_d{k}` (section 7.5) is realised in `zk/compose` — as
compiled `path_reach_d{depth}_k{graphs}_n{slots}` members over the same shape lattice — but is
*not yet* a `manifest::CircuitId` member: binding it into the manifest schema and verifier
dispatch is in progress (the *gate* tier of section 7.1). The compile-time depth is the
member's identity — a distinct depth is a distinct verification key — so a verifier learns the
disclosed bound from which member it accepts; the `depth_bound` public input re-states it.

== Entailment regimes

Only *simple entailment* is proved in zero knowledge (`manifest::EntailmentRegime::Simple`;
`Rdfs`/`Owl` are placeholders). A manifest #strong[MAY] declare `Rdfs`/`Owl` derivation steps
(`derivation_steps`), but these are re-checked by the verifier against *disclosed* bases
(`bind_entailment`, section 10.4): every step must be a well-formed, regime-admitted rule
instance whose antecedents are grounded in an earlier step or a disclosed scan row. A
non-`Simple` regime with no grounded steps is rejected (fail-closed). The derivation bases are
revealed to the verifier; the in-circuit closure proof is deferred. A prover #strong[MUST NOT]
represent a disclosed-base re-check as a zero-knowledge entailment proof.

== Bounded-depth property paths

The recursive path operators `p+`, `p*`, and `p?` are admitted under an *explicitly bounded*
statement: the circuit proves reachability within a disclosed depth, never unbounded closure.
Only a closure over an *atomic* step — a predicate IRI, possibly inverted (`(^p)+` proves `p+`
between the swapped endpoints) — is expressible; the `path_reach` family
(`sparq_zk_compose_core::path`, section 7.3) proves chains all carrying a single predicate, so a
closure over a sequence, alternative, nested closure, or negated property set is rejected
fail-closed. The non-recursive path forms are rewrites into the *core* fragment (section 7.2)
and carry no new statement.

For a closure over predicate `p` from `s` to `o`, a `path_reach` sub-proof proves — and a
manifest #strong[MUST] be read as claiming — exactly the boxed statement, no more:

#note[
  There exists a chain of committed triples (t_1, …, t_ℓ) with 1 ≤ ℓ ≤ k
  (0 ≤ ℓ ≤ k for `*` and `?`; k = 1 for `?`), each t_i a member of a committed graph in
  the disclosed attribution set, each carrying predicate p, chained object-to-subject,
  connecting μ(s) to μ(o) — where *k is a public input* disclosed in the manifest and fixed by
  the circuit member (the `depth_bound` input; a distinct k is a distinct circuit and
  verification key). The exact length ℓ within the bound stays hidden.
]

The following are normative; each is a realised in-circuit constraint of `path_reach_check`
together with a verifier obligation the composition layer (the *gate* tier, section 7.1) must
carry:

+ *k is public and surfaced.* Proofs at different k are *different statements*. A verifier
  #strong[MUST] expose k to the consumer and #strong[MUST] reject a manifest whose claimed
  path depth exceeds the bound of the circuit member it is bound to. A surface that renders
  "path exists" without k misstates the claim.
+ *Existence only — never absence.* A bounded path proof is monotone: it #strong[MUST NOT] be
  read as asserting that no longer path exists, nor that the reachable set is complete. Failure
  to prove at depth k proves nothing. Every bounded witness is a genuine SPARQL `p+`/`p*`
  solution — eval_k(P) ⊆ eval(P) (section 7.2) — so membership is preserved and
  completeness holds only up to k. (Any walk between two nodes has a simple walk of length at
  most the committed union's node count, so a k at least that count restores per-pair
  completeness; worth stating, never assumed.)
+ *Zero-length case.* For `p*` and `p?` the zero-length path holds only when μ(s) = μ(o)
  #strong[and] that term occurs as a subject or object of a committed triple: the circuit
  #strong[requires] an occurrence witness, matching SPARQL's zero-length-path term universe;
  bare equality is not sufficient, and a predicate-position occurrence does not count. `p+`
  #strong[MUST NOT] admit the zero-length case.
+ *Inert padding.* A chain shorter than k pads with pass-through steps that #strong[MUST NOT]
  advance the endpoint or draw a graph into the attribution set; padding rows are the family's
  primary forgery surface and are covered by dedicated forge-negative tests.
+ *Cross-graph chain links.* When a path's attribution set admits more than one graph, every
  chain-equated term — the interior chain nodes and the zero-length endpoint equality —
  #strong[MUST] carry the coarse non-blank-node obligation, extending the cross-graph
  blank-node exclusion (the "Q6" guard, section 7.2) from join edges to path links.
+ *Cycles.* Path semantics is existence-based set semantics, so a witness chain need not be
  simple; cycles are harmless for membership.

Source attribution is *chain-relative*: the attribution bits attest the graphs of the
existential witness the prover chose — exact within that witness (a triple present in several
graphs sets every such graph's bit) — not of every possible chain between the endpoints. Like
the rest of the estate these circuits are research-grade and #strong[not] externally audited
(section 17.1): they *expand* the internally-re-audited surface, and a bounded path proof is
not a production guarantee while the external audit gate (sq-qhy4) is open.

== FILTER expressions

// [OPUS-4.8] sq-3kd2g.5: scope the admitted query-side FILTER constant to canonical
// non-negative xsd:integer (the only lane sparq-zk::verify binds today, via filter_int); the
// f64/signed/decimal/value_dl circuits exist but are not yet query-side reachable. Mirrors the
// section 7.1 circuit-exists-vs-bound-end-to-end tiering. Do NOT widen while that wiring is
// unbuilt.
A `FILTER` is admitted only as a *conjunction* (`&&`, flattened) of atomic value comparisons of
the form `?var op c`, where `op` is one of `=`, `!=`, `<`, `<=`, `>`, `>=` and `c` is a
canonical non-negative `xsd:integer` literal — the only constant the query-side `FILTER`
re-derivation (`sparq-zk::verify`, shared by the stage-1 compose verifier and the
extended-fragment gate) binds, through the `filter_int` lane. `?var != c` is recognised as its
`Not(Equal(…))` parse; a `const op ?var` comparison is flipped so the variable is on the left;
a non-canonical `xsd:integer` lexical form (leading zero, sign, whitespace) is rejected because
the `filter_int` lane can bind only the canonical non-negative token, so no honest proof could
match any other form. Each comparison binds slot-wise to a scan row, and error-as-unsatisfied
(section 7.2) makes the shape monotone.

The other value `FILTER` lanes — the integer-valued `xsd:double` fragment (`filter_f64`),
signed integer (`filter_signed_int`), fixed-point `xsd:decimal` (`filter_decimal`), and, behind
the off-by-default `dual-leaf` feature, the value-dictionary lanes (`filter_value_dl*`) — exist
as composable circuit members (section 7.1) but are #strong[not] yet reachable from a
query-side `FILTER`: no binding path wires a query constant of those datatypes to them today.
This is the same realised-but-not-yet-bound-end-to-end distinction the *gate* tier of
section 7.1 draws for the `path_reach` family — the circuit exists, but the query-side binding
that would reach it does not — and until that wiring lands the gate #strong[MUST] reject a
`FILTER` whose constant is not a canonical non-negative `xsd:integer`.

Every other `FILTER` form is rejected fail-closed and #strong[MUST NOT] be silently disclosed
unproven: a variable–variable comparison (`?a op ?b`), disjunction (`||`), a general negation
`!(…)` other than the `!=` shape, an arithmetic operand (`?a + c op d`), a function call,
`IN` / `NOT IN`, `BOUND`, and `EXISTS` / `NOT EXISTS`.

The broader expression estate is a *proposal* (design record §5), not part of the accepted
fragment: the general SPARQL 1.1 function library, the three-valued (error-carrying) logical
layer for `&&` / `||` / `!` / `IF` / `COALESCE`, the term accessors (`isIRI`, `datatype`,
`lang`, `str`, …), and the string, numeric, date-component, and hash functions. Admitting them
is the composition work of the fragment-extension program's later phases; until a lane lands,
the gate #strong[MUST] reject a query that uses it.

= The proof manifest

== Typing and canonical serialisation

A proof manifest is a JSON object (`sparq-zk-compose` `manifest::ProofManifest`,
`to_json`/`from_json` via serde). Its `type` member #strong[MUST] be the value
`urn:sparq:zk:ProofManifest` (the field defaults to this constant when absent).

Every hash of a manifest — for nonce binding, deduplication, or audit — is defined over the
manifest's *canonical serialised form* (`ProofManifest::canonicalize`, which sorts the
self-contained `binding_edges` and `join_edges` into their derived total order before
serialising). This draft records that the reference implementation canonicalises before hashing,
but does #strong[not] standardise the
canonicalisation algorithm; two independent implementations cannot yet be expected to agree on
a manifest hash, and manifest-hash interoperability is expressly *not* offered by this draft
(section 2.3).

== Member schema

The following members are transcribed from the `ProofManifest` Rust struct. The *trust status*
column is candidate-normative: it records whether each member is a trust anchor, a mere
narrowing claim, or informational — the load-bearing distinction the codex #1 soundness fix
codified (section 11).

#table(
  columns: 3,
  align: (left, left, left),
  table.header[Member][Content][Trust status],
  [`type`], [The string `urn:sparq:zk:ProofManifest`.], [Schema marker.],
  [`query`], [The SPARQL query text the proof attests a result for.], [*Re-parsed, never
    trusted*: the verifier re-parses it (`verify::recheck`).],
  [`issuers`], [`did:key` references for the committed graphs.], [Informational provenance
    only.],
  [`key_set`], [The prover's declared issuer verification keys (hex Baby-JubJub points).],
    [*Narrowing claim only*: accepted only as a *subset* of the external anchor K
    (section 11); never the trust anchor (codex #1).],
  [`commitment_attestations`], [One issuer attestation per distinct scan commitment.],
    [Checked against the external K (audit gate 3).],
  [`attributions`], [Per-pattern graph-attribution sets. The indices are *scan-local*:
    `attributions[pattern]` indexes the answering scan's own `commitments` vector, and the
    verifier maps them to committed-graph *identity* (`global_attributions`) before deriving
    cross-graph obligations.], [Fed to the Q6 cross-graph blank-node-join guard; enforced as a
    superset by `bind_attributions`.],
  [`join_obligations`], [Declared non-blank-node join obligations `(variable, i, j)`.],
    [Manifest side of the join gate.],
  [`entailment_regime`], [`Simple` \| `Rdfs` \| `Owl`.], [Enforced by `bind_entailment`.],
  [`derivation_steps`], [Inference steps justifying derived triples (empty for `Simple`).],
    [Re-checked against disclosed, grounded bases (section 7.4).],
  [`binding`], [The binding mode — `Challenge { challenge }` (the v1 default), or
    `HolderPop { challenge, holder, pop, cryptosuite }` (clear-key holder
    proof-of-possession, checked by `bind_holder_pop`); both carry the verifier nonce as
    `challenge`.],
    [Bound to the verifier's own nonce (section 9).],
  [`revocation`], [Optional status reference `(status_list, index, version)`.],
    [Issuer-bound; a status-bound credential with this omitted is *rejected* (`bind_revocation`,
    section 10.4).],
  [`status_snapshots`], [Disclosed status-list bitstrings.], [*Prover copy is a tripwire
    only*; the bit decision reads the relying party's authoritative snapshot (section 11).],
  [`sub_proofs`], [Array of sub-proof objects (`{ inputs, proof_hex }`).], [Each verified
    against a recomputed key + reconstructed inputs (section 10.5).],
  [`binding_edges`], [Binding-consistency edges between sub-proofs.], [Enforce operand
    identity across sub-proofs (scan row/slot == consuming filter operand).],
  [`join_edges` / `hidden_revocation` / `hidden_issuer_attestations` / `holder_pok_proofs` /
    `holder_set_proofs`], [Optional privacy-upgrade layers (hidden join / hidden-index
    revocation / hidden issuer / hidden-key holder proof-of-possession / hidden-holder set).],
    [Additive; the clear-path checks always run (section 17.2).],
)

Three JSON value-level conventions apply throughout (descriptive, from the implementation).
Every field element (`FieldHex` — commitments, term encodings, the challenge) is rendered as a
`0x`-prefixed, 64-nibble, lowercase big-endian hexadecimal string. Pattern indices follow the
order in which the BGP triple patterns appear in the re-parsed query text
(`attributions[i]` describes the query's i-th pattern). In a binding edge, `from_slot` selects
the operand column of a disclosed scan row as `0` = subject, `1` = predicate, `2` = object.

== Sub-proof encoding

Each sub-proof carries its statement in `inputs` (a typed `ProofInputs` variant matching the
circuit) and its proof in `proof_hex`: a single hex-encoded blob produced by `sparq-zk-compose`
`verifier::encode_artifacts` with the layout

```
proof_hex = hex( LP(proof) ‖ LP(public_inputs) ‖ vk )
LP(x)     = ( len(x) as u32, big-endian, 4 bytes ) ‖ x
```

— a 4-byte big-endian length-prefixed proof, a 4-byte big-endian length-prefixed public-input
segment, and the verification key as the trailing remainder (not length-prefixed). The
verification key carried here is the prover's and is #strong[never] trusted: the verifier
recomputes the canonical key (audit gate 2, section 10.5).

== Public-input encoding (descriptive; at-risk)

This subsection is *descriptive and at-risk*; it deliberately attaches no RFC-2119
requirement. With the pinned toolchain of section 16, the encoding of a sub-proof's
public-input segment (which `verifier::reconstruct_public_inputs` byte-compares against, audit
gate 1) is:

- each public-input field element is encoded as exactly 32 bytes, big-endian;
- structs and arrays are flattened in row-major order (declaration order of the circuit's
  `main`);
- booleans encode as `0` or `1`; `u32`/`u64` values encode as their integer value;
- the segment carries no header and no per-element length prefix;
- public-input field 0 is the verifier nonce (section 9).

#note[
  This byte layout is *empirically pinned* to a specific Barretenberg release
  (`bb 5.0.0-nightly.20260324`, driven as a subprocess alongside `nargo 1.0.0-beta.21`, bb
  target `noir-recursive`); it was determined by observation against real `bb` output, not
  derived from a backend specification, and it is not guaranteed stable across `bb` releases. A
  reverse-engineered, toolchain-fragile layout is not a conformance requirement, so this draft
  states it descriptively. A MUST-level layout will be introduced only once it is specified
  independently of the `bb` toolchain (section 2.3); until then, interoperability across
  backend versions is not promised (section 16).
]

== Worked example (illustrative)

The example below is *illustrative only*: the elided hex (`…`) is not real proof material,
the scan's trailing zero-padded rows (up to `r`) are elided, and the optional
privacy-upgrade members are absent. The member names and serde tagging shapes (`"circuit"`,
`"kind"`, `"mode"`) mirror the implementation's serialisation exactly. It is *not* a test
vector and cannot be verified; portable conformance fixtures are open future work
(section 16).

```json
{
  "type": "urn:sparq:zk:ProofManifest",
  "query": "SELECT ?s ?o WHERE { ?s <http://ex/age> ?o FILTER(?o >= 18) }",
  "issuers": ["did:key:zIssuer"],
  "key_set": ["1c0aa5b7…e977"],
  "commitment_attestations": [
    { "commitment": "0x2f1e…", "issuer_public_key": "1c0aa5b7…e977",
      "signature": "…", "cryptosuite": "https://sparq.dev/ns/zk#poseidon2-schnorr-v1" }
  ],
  "attributions": [[0]],
  "entailment_regime": "simple",
  "binding": { "mode": "challenge", "challenge": "0x00…2a" },
  "sub_proofs": [
    { "inputs": { "circuit": "scan",
        "id": { "kind": "scan", "k": 1, "n": 16, "r": 4 },
        "commitments": ["0x2f1e…"],
        "pattern_is_const": [false, true, false],
        "pattern_const_enc": ["0x00…00", "0x17c4…", "0x00…00"],
        "rows": [["0x08a1…", "0x17c4…", "0x2b9d…"], …],
        "row_count": 1,
        "attribution": [true] },
      "proof_hex": "…" },
    { "inputs": { "circuit": "filter_int",
        "id": { "kind": "filter_int", "d": 2 },
        "operand_enc": "0x2b9d…",
        "op": "ge", "bound": 18, "expected": true },
      "proof_hex": "…" }
  ],
  "binding_edges": [{ "from_proof": 0, "from_row": 0, "from_slot": 2, "to_proof": 1 }]
}
```

= Challenge–response: the verifier nonce

The proof request is a nonce challenge (`sparq-zk-compose` `verifier::VerifierNonce`,
`SeenNonces`):

+ The verifier #strong[MUST] mint a fresh #dfn[verifier nonce] — a BN254 scalar field
  element, exchanged in hexadecimal form — and deliver it to the prover *before* proving
  begins. Nonces #strong[MUST NOT] be reused across requests.
+ The prover #strong[MUST] commit the nonce as *public-input field 0 of every sub-proof* in
  the manifest, and carry it in `binding` — as the `challenge` member of either binding mode,
  `Challenge` or `HolderPop` (section 8.2).
+ The verifier #strong[MUST] record the nonce as used (single-use, `SeenNonces::record_fresh`)
  *before* running the cryptographic checks of section 10.5, so that a manifest that fails late
  cannot be replayed against the same nonce.
+ If a manifest binds a nonce other than the one issued for the request, the verifier
  #strong[MUST] reject with a nonce-binding mismatch (`NonceBindingMismatch`) *and*
  #strong[MUST] still burn the issued nonce — once the nonce is recorded, no subsequent
  rejection is a free retry (rejections raised by the structural prefilter or the entailment
  re-check, which run before the nonce is recorded, do not consume it). The nonce (not
  the manifest's declared `binding`) is fed as public-input field 0 during reconstruction, so a
  proof committed under any other challenge fails the byte-compare of audit gate 1.

How the nonce is delivered and how the manifest is submitted is out of band and currently
unspecified; section 14 proposes a transport binding.

= Verification

== Overview

The reference verifier exposes the full-binding entry point `verifier::verify_manifest`, which
takes the manifest, the circuit prover, a work directory, and the relying party's trust anchors
— the trusted `KeySet`, the `RevocationPolicy`, the `HolderRegistry`, the
`HolderBindingPolicy`, the entailment policy, the fresh `VerifierNonce`, and the `SeenNonces`
store — and returns `Result<(), CheckError>`. Sections 10.2–10.5 describe its obligation set;
the fail-closed discipline of sections 10.2 and 10.6 is kernel-normative (section 2.2). Failure
yields a `CheckError` variant pinpointing the failed gate.

== Entry points

The reference verifier exposes two entry points:

+ a *structural prefilter* (`prefilter_manifest_structure`) covering shape and consistency
  checks, which is #strong[not] sufficient on its own — it runs no backend, binds nothing to a
  proof, and enforces no freshness — and #strong[MUST NOT] be treated as verification; and
+ *full verification* (`verify_manifest`), which runs the prefilter (whose stages include the
  structural re-checks of section 10.3), the binding obligations of section 10.4, and the
  cryptographic checks of section 10.5, in a fail-closed pipeline.

Only full verification performs the (unaudited — section 17.1) checking this document describes.

== Structural re-checks

Full verification performs the following structural re-checks as the first, mandatory stage of
its pipeline — they are stages of `prefilter_manifest_structure`, so they also run when the
prefilter is invoked alone. The blank-node guard and the attribution-arity check live in
`sparq-zk` (`verify::recheck`); the circuit-identifier re-derivation and the
strictly-increasing commitment ordering live in `sparq-zk-compose`
(`prefilter_manifest_structure` itself):

- the *blank-node guard* (the "Q6" guard): a cross-graph join on a blank node is rejected,
  enforcing the semantic exclusion of section 7.2 against a malicious prover — keyed on
  committed-graph identity, not the scan-local index, so a genuine cross-scan join over two
  distinct committed graphs must declare its non-blank-node `join_obligations`;
- *attribution arity*: the attribution structure is well-formed;
- *circuit-identifier re-derivation*: the identifier each sub-proof claims is re-derived from
  the statement it is bound to, and the two must agree;
- *strictly-increasing commitment ordering*: the manifest's graph commitments are strictly
  increasing, giving a canonical order and excluding duplicates.

== Binding obligations

The reference verifier enforces the twelve binding obligations below (`sparq-zk-compose`
`verifier::bind_*`); each is fail-closed. The obligation *set* is covered by the adversarial
("forge") test suite of the reference implementation. The one-line glosses are descriptive:
the implementation remains the source of truth for each obligation's exact clause (section 2.2).

#table(
  columns: 2,
  align: (left, left),
  table.header[Obligation][Descriptive gloss],
  [`bind_query_correctness`], [Every query BGP pattern has a scan binding its constant slots,
    and every `FILTER` has a slot-bound, true-verdict filter sub-proof reachable via a binding
    edge; target property is result membership under section 7.2.],
  [`bind_attributions`], [`manifest.attributions[pattern]` is a *superset* of each answering
    scan's proof-bound in-circuit attribution bits (closes the attribution-collapse forge).],
  [`bind_issuer_attestations`], [Every issuer key used is a member of the *external* trusted
    key set K (section 11) — never merely of the manifest's own key list — and its Schnorr
    signature over the commitment verifies. This obligation *is* audit gate 3 (section 10.5).],
  [`bind_revocation`], [The disclosed status reference is issuer-bound, and the liveness bit is
    read from the relying party's *authoritative* snapshot (never the prover's), within the
    freshness window.],
  [`bind_joins`], [Each `JoinEdge`'s `join_eq` proof binds its public commitments byte-for-byte
    to the two scans' graph commitments, and its slots to the query-derived slots.],
  [`bind_entailment`], [The declared entailment regime is honoured and derivation steps are
    re-checked against disclosed, grounded bases (section 7.4).],
  [`bind_holder_pop`], [Holder proof-of-possession (clear-key tier) is a valid Schnorr over the
    challenge, the holder is in the external `HolderRegistry`, and (under `require_binding`) the
    presented key matches the issuer-attested holder digest.],
  [`bind_holder_binding`], [The clear (disclosed) holder key's digest equals the attestation's
    `holder_pk_digest`, and the disclosed key matches the presented key.],
  [`bind_hidden_revocation`], [Hidden-index revocation: the proof's public Merkle root equals
    the root derived from the relying party's authoritative snapshot.],
  [`bind_hidden_issuer_attestations`], [Hidden issuer: the proof's public key-set root equals
    the root derived from the authoritative external K, without disclosing which issuer.],
  [`bind_holder_pok`], [Hidden-holder proof-of-knowledge tier — explicitly *not yet* sound;
    opt-in only (section 17.2).],
  [`bind_holder_set`], [Hidden-holder set-membership tier — explicitly *not yet* sound;
    opt-in only (section 17.2).],
)

== Cryptographic checks and the four audit gates

The scheme defines exactly four #dfn[audit gates] — the cross-cutting binding checks whose
failure would each void soundness on its own:

#table(
  columns: 3,
  align: (left, left, left),
  table.header[Audit gate][Definition][Enforced by],
  [1 — public-input reconstruction], [The verifier independently reconstructs the expected
    public-input bytes of every sub-proof — with the verifier nonce at field 0 — and compares
    them byte-for-byte against the manifest's segment; any difference rejects.],
    [`reconstruct_public_inputs` (`PublicInputMismatch`).],
  [2 — canonical verification key], [The verification key of every sub-proof is recomputed
    from the canonical circuit named by the re-derived `CircuitId`; the prover's key is never
    trusted.], [`canonical_vk` recomputation.],
  [3 — issuer signature and key set], [Every issuer key used is bound to the external key
    set K, and the issuer's signature over the graph commitment verifies.],
    [`bind_issuer_attestations` (section 10.4), per scan.],
  [4 — nonce single-use and binding], [The verifier nonce is fresh, single-use, recorded
    before the cryptographic checks, and burnt on mismatch.], [The nonce discipline of
    section 9 (`SeenNonces`, `NonceBindingMismatch`).],
)

In addition to the four audit gates, each sub-proof is verified by the proving backend
against the recomputed key and reconstructed inputs (*backend proof verification*). This
check carries no audit-gate number: the audit gates are the binding checks layered *around*
backend verification, which is meaningless without gates 1 and 2 pinning what is verified.
Full verification runs, fail-closed and in order: nonce single-use recorded and binding checked
(gate 4); then, per sub-proof, gate 1; gate 2; backend proof verification. Gate 3 is enforced
per scan by `bind_issuer_attestations`.

== Fail-closed error handling

Every failure mode maps to an explicit variant of a closed error taxonomy
(`verifier::CheckError`, on the order of eighty variants). A conforming verifier
#strong[MUST] reject the whole manifest on the *first* failed check, #strong[MUST NOT] return
partial results, and #strong[MUST NOT] downgrade any error to a warning.

= External trust anchors

All trust anchors are inputs from the relying party, passed to `verify_manifest`. A conforming
verifier #strong[MUST] obtain each of the following out of band and #strong[MUST NOT] accept
any of them from the manifest:

+ the *trusted issuer key set K* (`KeySet`) — the manifest #strong[MAY] carry its own
  `key_set`, but it is accepted only if it is a *subset* of K; an empty K trusts no issuer, so
  any scan carrying commitments is rejected;
+ the *authoritative status-list snapshot* governing revocation (`RevocationPolicy`,
  `StatusListSnapshot`) — the prover's `status_snapshots` copy is only a tamper tripwire;
+ the *holder registry* and *holder-binding policy* (`HolderRegistry`, `HolderBindingPolicy`);
+ the *fresh verifier nonce* of section 9 (`VerifierNonce`);
+ the *seen-nonce store* enforcing single use (`SeenNonces`) — this store #strong[SHOULD] be
  durable across verifier restarts (the reference implementation provides a durable file-backed
  store `FileSeenNonces` (flock + fsync, single-host) and a test-only `InMemorySeenNonces` that
  forgets burned nonces on restart).

#note[
  The subset rule for K is codified from experience: an earlier revision that trusted the
  manifest's own key list was a review-identified soundness hole (the "codex #1" fix, recorded
  in the `key_set` doc comment). Externalising every trust anchor is what closed it.
]

= Security-properties vocabulary

zkSPARQL methods are *annotated* with machine-readable security properties so that policy
engines can reason about them (section 13). The vocabulary is layered and vendored into the
repository.

== Base vocabulary

The base vocabulary is the vendored `sec-prop` ontology, namespace
`https://w3id.org/zkp-sparql/sec-prop#`, whose eight base security-property classes are
`Unlinkability`, `SourceCredentialDisclosure`, `PostQuantumForgery`, `PostQuantumSnooping`,
`SignatureTypeLeakage`, `ProofSizeLeakage`, `CircuitAudit`, and `ValidityPeriodLeakage`
#cite("SEC-PROP"). The `sec-prop` vocabulary is prior published work of this document's editor
and collaborators (Wright, Shadbolt, Zhao, Zhao, Braun #cite("SEC-PROP")); sections 12 and 13
derive from it, and it is vendored into sparq under MIT with its provenance record (co-authored
for the ISWC 2025 work, MIT-licensed by the 2026-06-21 decision).

#note[
  Editor's note — the `w3id.org/zkp-sparql/` identifiers were minted as placeholders while the
  source repository was private. Before this draft advances, the permanent-identifier redirect
  must be confirmed live and stable.
]

== The secx extension

The sparq extension (`secx`, declared in `secprop-ext.ttl` under the *same*
`https://w3id.org/zkp-sparql/sec-prop#` namespace — the `secx`/`sec-prop` distinction is
prose-only, and the extension `owl:imports` the base, it does not fork it) adds the orthogonal
proof-system dimensions `ZeroKnowledgeType`, `Soundness`, `Completeness`, `Hiding`, `Binding`,
`Anonymity`, `Setup`, `Interactivity`, `SelectiveDisclosure`, and `SingleUse`, plus four
orthogonal axes:

- `AssuranceLevel`, ordered `Proven` > `Claimed` > `Conjectured` (the sparq ZK default is
  `Claimed`);
- `AuditStatus`, ordered `ExternallyAudited` > `InternallyReviewed` > `Unreviewed`, plus the
  distinguished value `ExternalSignOffPending` (the live sq-qhy4 state);
- `Assumption` (e.g. `IssuerHonesty` — carried by the dual-leaf lane — `DiscreteLog`,
  `RandomOracle`, `HonestMajority`, `SemiHonest`);
- `PropertyScope`, distinguishing `QueryProofLayer` (default) from `SourceLayerOnly`.

A property that holds at the source layer only #strong[MUST NOT] be used to satisfy a
query-proof-layer constraint: source-layer facts do not transfer to the query-proof layer.

The dimension names shadow the security goals of section 5.3 deliberately: an annotation is a
machine-readable *claim* about a goal, and the assurance axis records how settled the claim is.

== The over-claim rule

While the external audit gate (sq-qhy4) is open (`sparq-zk` `secprop` module, behind the
`secprop-annotations` feature):

+ No sparq zkSPARQL method #strong[MAY] be annotated `secx:Proven` for any *positive* privacy
  or soundness property; such properties are at most `secx:Claimed` with
  `AuditStatus ExternalSignOffPending`.
+ Only *settled negative* facts — for example `PQForgeable`, `Replayable`, `SchemeRevealed`
  — #strong[MAY] carry `Proven`.

The reference implementation enforces this rule mechanically with three machine-checkable
guards over the annotation graph (`ontologies/secprop-methods.ttl`):
`audit_overclaim_violations` (no `Proven` on a positive property while the gate is open),
`completeness_violations` (every production-selectable method is annotated), and
`source_layer_transfer_violations` (a `SourceLayerOnly` property never satisfies a query-proof
constraint).

= Policy-controlled admissibility

Relying parties express *which* proof methods they accept as ODRL 2.2 policies #cite("ODRL22")
over the vocabulary of section 12, using the sparq security-property profile
(`odrl-secprop-profile.ttl`, `sparq-policy`; profile IRI
`https://sparq.dev/ns/odrl-secprop-profile#`, which declares fifteen `secx:requires…`
leftOperands; reduced by `sparq-trust` `admissibility` / `admit`):

+ A policy using any `secx:requires…` left-operand #strong[MUST] assert
  `odrl:profile <https://sparq.dev/ns/odrl-secprop-profile#>`.
+ Each such left-operand carries exactly one `secx:overDimension` fact identifying the
  property dimension it constrains.
+ Only the operator `odrl:gteq` is given a reduction; a constraint using any other operator
  #strong[MUST] be treated as *unsatisfied* — which denies.
+ A method is admissible only if it satisfies *every* constraint of the policy
  (default-deny).
+ In the fail-closed pre-check gate (`admit_with_precheck`), the outcomes are `Admitted`,
  `Denied`, `UnknownMethod`, `MalformedConstraint`, and `ReductionError` — and a reduction
  error or a malformed constraint #strong[MUST] be treated as a denial.

Base admission additionally checks the issuer's Schnorr signature over the RDFC-1.0
commitment, a SHACL #cite("SHACL") statement-type scope constraint, a reserved-predicate guard,
and — for clear holders — a WebID holder binding (the credential subject equals the session
agent).

#note[
  A consequence worth stating plainly: a policy requiring
  `requiresAssurance odrl:gteq secx:Proven` on a positive property mechanically denies *every*
  current sparq zkSPARQL method while the external audit (sq-qhy4) is open. That is by design —
  it is the honest default for high-assurance relying parties.
]

= Transport, media type, and interchange

This section is entirely a *proposal*: none of it exists in the reference implementation
today. The manifest is a bare JSON object tagged with a URN, and both nonce issuance and
manifest submission are out of band.

== Media type (proposal)

A registered media type is proposed for the proof manifest — candidate
`application/zksparql+json`, with an `application/zksparql+ld+json` variant once a JSON-LD
context exists — and a companion type for the nonce challenge. Until registration,
implementations exchanging manifests over HTTP have no content-type contract.

== JSON-LD context (proposal)

A JSON-LD context for the manifest is proposed so that a manifest can round-trip as a W3C
Verifiable Presentation #cite("VC-DATA-MODEL") and be consumed by generic data-integrity
processors. No such context exists today; the manifest does not currently round-trip.

== Wire protocol (proposal)

A challenge–response HTTP binding is proposed: an endpoint issuing single-use nonces and an
endpoint accepting manifest submissions bound to them. No server endpoint, job model, or
asynchronous proving exists in the implementation, so any binding written here would be
speculative and is deferred to a subsequent draft.

= Relationship to W3C Verifiable Credentials

This section is informative.

- A *VC cryptosuite bridge* — off-circuit Data-Integrity verification of `eddsa-rdfc-2022`
  and `ecdsa-rdfc-2019` (P-256) source credentials #cite("VC-DI") at ingest — is designed as
  an opt-in `vc-bridge` feature but is *not merged to the main line* at the time of writing
  (it lives on a feature branch and plugs into the `IssuerSignatureScheme` seam). Any claim of
  VC ingest must be caveated accordingly.
- In that bridge design, the P-384 profile of `ecdsa-rdfc-2019` is *not* implemented and
  fails closed as an unsupported key curve; like the bridge itself, this behaviour is not on
  the main line — on main there is no VC-ingest path at all.
- Ingest of `bbs-2023` / `ecdsa-sd-2023` selective-disclosure credentials is an explicitly
  *deferred* seam: there is no in-repo BBS verifier.
- In-circuit re-verification of the source credential's proof is deliberately *out of scope*:
  the query proof does not re-verify the source VC signature inside the circuit. The
  `zk:sourceCryptosuite` annotation is provenance only, and #strong[MUST NOT] be read as
  evidence that the source proof was verified in zero knowledge.

= Conformance testing and toolchain pinning

Two conformance gaps are open:

+ *No portable test vectors.* An adversarial forge-test suite exists covering the manifest
  format and the verifier obligation set, but it is internal to the Rust implementation, and
  the cryptographic-chain forge tests and real `bb` prove/verify cases are `#[ignore]`d in
  default CI (they require the nargo/bb toolchain). A conformance suite of portable fixtures
  (manifests that must verify, and mutated manifests that must fail with a specific error
  class) is required future work.
+ *Toolchain pinning.* The circuit family is pinned to an external toolchain
  (`nargo 1.0.0-beta.21`, `bb 5.0.0-nightly.20260324`, bb target `noir-recursive`, with the
  in-circuit Poseidon fixed to the `noir-lang/poseidon` `v0.3.0` tag) driven by subprocess, and
  the public-input byte layout of section 8.4 is empirically determined and therefore
  descriptive, not normative. A toolchain change could silently shift the serialisation with no
  failing test. Until the layout is specified toolchain-independently (section 2.3),
  cross-version interoperability is out of reach and even reference-level compatibility can
  only be claimed against the pinned toolchain.

= Security and Privacy Considerations

The threat model and the meaning of each security goal are given in section 5; this section
records the honest status of those goals and the known deviations.

== Audit status

The entire zkSPARQL estate is research-grade and has *not* been externally audited; the
external cryptographer audit is an open gate (sq-qhy4). An internal, single-model re-audit
found the verifier's binding layer *sound as landed for the threat model its prior audit
assumed*, but that finding was produced by an LLM agent, rests partly on code-reading rather
than on tests running in CI, and does #strong[not] replace external sign-off. Accordingly:

+ A relying party #strong[MUST NOT] treat a passing verification as a settled guarantee that
  the proven SPARQL statement holds against an adversarial prover.
+ Soundness and attestation are *not production-ready*; deployments that need a production
  guarantee are out of scope for this draft until the audit closes.
+ The over-claim rule of section 12.3 applies to every annotation surface: positive
  properties are at most `Claimed`, with audit status `ExternalSignOffPending`.

== Known-unsound and downgraded components

- The hidden-holder tiers (`bind_holder_pok`, `bind_holder_set`) are explicitly labelled *not
  yet* sound in the implementation and its documentation; remediation is tracked internally
  (epic sq-1s2). They are opt-in only, and verifiers #strong[SHOULD] leave them disabled
  unless the residual risk is understood and accepted.
- The optional dual-leaf value lane carries an accepted, documented value↔lexical invariant
  downgrade (INV-VL, gap CR-G8, #769): value–lexical agreement on the value-`FILTER` lane
  rests on trusted-issuer honesty (recorded as a `secx:IssuerHonesty` assumption) and is not
  machine-enforced. It is opt-in, partial, and unaudited.
- Only simple entailment is proved in zero knowledge; `Rdfs`/`Owl` derivations are
  disclosed-base re-checks (section 7.4), which reveal the derivation bases to the verifier
  and limit entailment coverage.

== Post-quantum posture

The post-quantum posture is a *settled negative*. The issuer signature suite in scope
(Schnorr over Baby Jubjub) and the related credential suites (EdDSA, BBS+) rest on
discrete-log hardness and fall to a Shor-capable adversary; commitment binding likewise breaks
under a cryptographically relevant quantum computer, so *retrospective* soundness of previously
accepted proofs fails as well. The vocabulary records this honestly as negative
`PostQuantumForgery` / `PostQuantumSnooping` facts — these negatives are among the few
annotations permitted to carry `Proven` (section 12.3). The scheme makes *no* FIPS or CMVP
claim; it is deliberately built on ZK-friendly, non-FIPS-approved primitives (BN254,
Poseidon2, Baby Jubjub), and the signing path is not constant-time (a documented residual).

== Leakage and unlinkability

`SignatureTypeLeakage`, `ProofSizeLeakage`, `ValidityPeriodLeakage`, and the unlinkability
dimensions are tracked as vocabulary dimensions so that policies can constrain them; their
values for sparq methods are at most `Claimed` and are *not* settled guarantees. By default,
issuer attestation is checked in the clear (revealing which issuer signed) and a clear-path
`revocation.index` is disclosed (a linkability channel) unless the hidden-issuer / hidden-index
circuits are enabled. Verifiers and relying parties should assume that proof size, timing, and
suite choice may leak information about the underlying credentials until an audit says
otherwise.

== Replay and nonce hygiene

Replay resistance rests entirely on the nonce discipline of section 9: single-use recording
*before* the cryptographic checks, burn-on-mismatch, and a durable seen-nonce store. A
verifier using a non-durable store (`InMemorySeenNonces`) forgets burned nonces on restart and
#strong[SHOULD NOT] be exposed where replay across restarts matters; a multi-host deployment
#strong[SHOULD] back `SeenNonces` with a database uniqueness / compare-and-set store.

== Admissibility reasons over annotations, not cryptography

The admissibility engine of section 13 reasons over *declared annotations*, not over the
cryptography itself. An "Admitted" outcome means the method's declared properties satisfy the
policy — it is not, and #strong[MUST NOT] be presented as, an independent cryptographic
finding.

= References

#references((
  ("RFC2119", [Bradner, S. #emph[Key words for use in RFCs to Indicate Requirement Levels].
    RFC 2119, IETF, March 1997.]),
  ("RFC8174", [Leiba, B. #emph[Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words].
    RFC 8174, IETF, May 2017.]),
  ("SPARQL11-QUERY", [Harris, S.; Seaborne, A. (eds). #emph[SPARQL 1.1 Query Language].
    W3C Recommendation, 21 March 2013. https://www.w3.org/TR/sparql11-query/.]),
  ("PAG09", [Pérez, J.; Arenas, M.; Gutierrez, C. #emph[Semantics and Complexity of SPARQL].
    ACM Transactions on Database Systems 34(3), article 16, 2009.]),
  ("RDF11-CONCEPTS", [Cyganiak, R.; Wood, D.; Lanthaler, M. (eds). #emph[RDF 1.1 Concepts and
    Abstract Syntax]. W3C Recommendation, 25 February 2014.
    https://www.w3.org/TR/rdf11-concepts/.]),
  ("RDF11-MT", [Hayes, P.; Patel-Schneider, P. (eds). #emph[RDF 1.1 Semantics].
    W3C Recommendation, 25 February 2014. https://www.w3.org/TR/rdf11-mt/.]),
  ("RDF-CANON", [Longley, D.; Kellogg, G.; et al. (eds). #emph[RDF Dataset Canonicalization
    (RDFC-1.0)]. W3C Recommendation, 2024. https://www.w3.org/TR/rdf-canon/.]),
  ("POSEIDON2", [Grassi, L.; Khovratovich, D.; Schofnegger, M. #emph[Poseidon2: A Faster
    Version of the Poseidon Hash Function]. AFRICACRYPT 2023; IACR ePrint 2023/323.]),
  ("PEDERSEN91", [Pedersen, T. P. #emph[Non-Interactive and Information-Theoretic Secure
    Verifiable Secret Sharing]. CRYPTO '91, LNCS 576, Springer, 1992. (Source of the standard
    hiding/binding commitment notions used in section 5.3.)]),
  ("SCHNORR91", [Schnorr, C. P. #emph[Efficient Signature Generation by Smart Cards].
    Journal of Cryptology 4(3), 1991.]),
  ("BN06", [Barreto, P. S. L. M.; Naehrig, M. #emph[Pairing-Friendly Elliptic Curves of Prime
    Order]. SAC 2005, LNCS 3897, Springer, 2006. (BN254 / alt-bn128 is the 254-bit instance
    standardised for Ethereum in EIP-196/EIP-197.)]),
  ("EIP2494", [Bellés-Muñoz, M.; Baylina, J. #emph[EIP-2494: Baby Jubjub Elliptic Curve].
    Ethereum Improvement Proposals, 2020.]),
  ("NOIR", [Aztec Labs. #emph[The Noir Programming Language]. https://noir-lang.org/.]),
  ("VC-DATA-MODEL", [Sporny, M.; et al. (eds). #emph[Verifiable Credentials Data Model v2.0].
    W3C Recommendation, 2025. https://www.w3.org/TR/vc-data-model-2.0/.]),
  ("VC-DI", [Sporny, M.; Longley, D.; et al. (eds). #emph[Verifiable Credential Data
    Integrity 1.0] and its cryptosuites (eddsa-rdfc-2022, ecdsa-rdfc-2019, bbs-2023,
    ecdsa-sd-2023). W3C Recommendations, 2025. https://www.w3.org/TR/vc-data-integrity/.]),
  ("ODRL22", [Iannella, R.; Villata, S. (eds). #emph[ODRL Information Model 2.2].
    W3C Recommendation, 15 February 2018. https://www.w3.org/TR/odrl-model/.]),
  ("SHACL", [Knublauch, H.; Kontokostas, D. (eds). #emph[Shapes Constraint Language (SHACL)].
    W3C Recommendation, 20 July 2017. https://www.w3.org/TR/shacl/.]),
  ("SEC-PROP", [Wright, J.; Shadbolt, N.; Zhao, Jun; Zhao, Rui; Braun, C. #emph[Zero-Knowledge
    Proof of Correct SPARQL Evaluation over Verifiable Credentials]. Prior work of this
    document's editor and collaborators; vocabulary source repository
    https://github.com/jeswr/sparql-zkp-ontologies, namespace `https://w3id.org/zkp-sparql/`.
    The `sec-prop` sub-vocabulary is vendored, with the sparq `secx` extension, in the sparq
    repository (MIT). Declared for citation integrity; sections 12–13 derive from it.]),
  ("WRIGHT-DC25", [Wright, J. #emph[Towards Provable Provenance and Privacy-Preserving Queries  // privacy-claims-allow: prior-work reference title (Wright, ISWC 2025 DC), not a sparq claim
    in Decentralised Data Architectures]. ISWC 2025 Companion Volume (Doctoral Consortium),
    CEUR-WS Vol-4085, paper 19, Nara, Japan, November 2025.
    https://ceur-ws.org/Vol-4085/paper19.pdf.]),
  ("BWK26", [Braun, C.; Wright, J.; Käfer, T. #emph[Proving Soundness of SPARQL Query Results
    Using Selective Disclosure of RDF Datasets and Zero-Knowledge Proofs]. In: The Semantic
    Web, Springer, 2026. DOI 10.1007/978-3-032-25156-5_16.]),
  ("INTEGRIDB", [Zhang, Y.; Katz, J.; Papamanthou, C. #emph[IntegriDB: Verifiable SQL for
    Outsourced Databases]. ACM CCS 2015.]),
  ("VSQL", [Zhang, Y.; Genkin, D.; Katz, J.; Papadopoulos, D.; Papamanthou, C. #emph[vSQL:
    Verifying Arbitrary SQL Queries over Dynamic Outsourced Databases]. IEEE Symposium on
    Security and Privacy, 2017.]),
  ("ZKSQL", [Li, X.; Weng, C.; Xu, Y.; Wang, X.; Rogers, J. #emph[ZKSQL: Verifiable and
    Efficient Query Evaluation with Zero-Knowledge Proofs]. Proceedings of the VLDB Endowment
    16(8), 1804–1816, 2023.]),
  ("CL02", [Camenisch, J.; Lysyanskaya, A. #emph[A Signature Scheme with Efficient
    Protocols]. SCN 2002, LNCS 2576, Springer, 2003.]),
  ("BBS04", [Boneh, D.; Boyen, X.; Shacham, H. #emph[Short Group Signatures]. CRYPTO 2004,
    LNCS 3152, Springer, 2004. (Origin of the BBS/BBS+ multi-message signature line used for
    selective disclosure.)]),
  ("SPARQ", [The sparq project. #emph[sparq: an RDF + SPARQL engine with a zero-knowledge
    query-proof estate (reference implementation)]. https://github.com/sparq-org/sparq.]),
))
