// [FABLE-5] sq-rvgr2.2 — zkSPARQL: Zero-Knowledge Query Proofs over SPARQL.
//
// Authored STRICTLY from the prose estate-recon digest (research/specs/zksparql-estate-recon.md
// on the docs/fable-program-recon-records branch); no crate sources were read while drafting.
// Code-level details the digest does not pin are flagged inline as Editor's notes and are to be
// transcribed from the implementation in a subsequent draft — the implementation remains the
// source of truth for those details until then.
//
// HONESTY: the entire zkSPARQL estate is research-grade and NOT externally audited (open
// external-audit gate sq-qhy4). This draft states that plainly and repeatedly; it must never be
// edited into claiming a settled production guarantee while that gate is open.
// (Provenance: dispatched as Claude Fable 5; exact model marker reconciled post-hoc by the
// orchestrator from the transcript.)

#import "_lib/spec.typ": spec-head, sotd, intro-section, references, dfn, note, cite

#set document(title: "zkSPARQL: Zero-Knowledge Query Proofs over SPARQL")
#set text(size: 11pt)
#set par(justify: true)
#set heading(numbering: "1.")

#spec-head()

#intro-section("abstract", "Abstract")[
  zkSPARQL is a proposal for proving, in zero knowledge, that the answer to a SPARQL query is
  correct with respect to one or more committed RDF graphs, without revealing the graphs
  themselves. This document specifies the interfaces realised by the sparq reference
  implementation: the committed data model (RDF Dataset Canonicalization with Poseidon2
  commitments over the BN254 scalar field, and issuer attestation signatures over Baby Jubjub);
  the supported SPARQL fragment and its circuit family; the #dfn[proof manifest] interchange
  object; the verifier-nonce challenge–response; the verifier's fail-closed obligation set; the
  external trust anchors a relying party supplies; and the layered security-properties
  vocabulary with its ODRL admissibility profile. Parts that do not yet exist — a registered
  media type, a JSON-LD context, and a wire protocol — are named as explicit proposals. The
  entire scheme is research-grade: it has not been externally audited, and no production
  guarantee is claimed (see the Security and Privacy Considerations, section 14).
]

#sotd()

#intro-section("audit-status", "Implementation and audit status")[
  A reference implementation of everything marked *implemented* below exists in the sparq
  repository #cite("SPARQ") as two opt-in, unpublished crates, with adversarial ("forge")
  tests. However, the implementation has *not* been reviewed by an external cryptographer: the
  external audit gate (tracked in-repo as *sq-qhy4*) is open. Until it closes, soundness and
  attestation are *not production-ready*; every positive security property of this scheme is at
  most a *claim*, and a passing verification is not a guarantee that the proven SPARQL
  statement holds against an adversarial prover. This draft never asserts otherwise, and
  section 9.3 makes the corresponding over-claim rule normative for annotations.
]

= Introduction

This section is informative.

Verifiable-credential ecosystems let a holder present signed RDF data to a verifier. zkSPARQL
extends that interaction from "show the data" to "prove a query over the data": a
#dfn[prover] evaluates a SPARQL query #cite("SPARQL11-QUERY") over RDF graphs that an
#dfn[issuer] has committed to and signed, and produces a #dfn[proof manifest] — a bundle of
zero-knowledge sub-proofs plus bindings — that a #dfn[verifier] checks without seeing the
underlying graphs. Example uses include proving that a credential attribute satisfies a
threshold FILTER, that a value was not revoked at an authoritative snapshot, or that two
hidden credentials agree on a join key.

The flow is challenge–response:

+ The verifier mints a fresh #dfn[verifier nonce] and hands it to the prover.
+ The prover evaluates the query over its committed source graphs and produces a proof
  manifest whose every sub-proof commits to that nonce.
+ The verifier checks the manifest against its own #dfn[trust anchors] (trusted issuer keys,
  an authoritative revocation snapshot, a holder registry, and its nonce store), enforcing a
  fixed, fail-closed obligation set (section 7).
+ Optionally, a policy engine decides whether the *method* used is admissible for the purpose
  at hand, by reasoning over the security-properties vocabulary (sections 9 and 10).

This document writes down the interfaces of that pipeline as they exist in the sparq
reference implementation, so they can be reviewed, critiqued, and cited. Where the
implementation leaves an interface unspecified (transport, media type, JSON-LD form), this
document proposes one and labels it a proposal. It is an Unofficial Proposal Draft; see the
Status of This Document.

= Terminology and conformance

== Requirement keywords

The key words #strong[MUST], #strong[MUST NOT], #strong[REQUIRED], #strong[SHALL],
#strong[SHALL NOT], #strong[SHOULD], #strong[SHOULD NOT], #strong[RECOMMENDED], #strong[MAY],
and #strong[OPTIONAL] in this document are to be interpreted as described in #cite("RFC2119")
and #cite("RFC8174") when, and only when, they appear in all capitals, as shown here.

== Terms

- A #dfn[committed graph] is an RDF graph #cite("RDF11-CONCEPTS") together with a
  cryptographic commitment to its canonical form (section 3.1).
- A #dfn[sub-proof] is a single zero-knowledge proof for one circuit of the family in
  section 4.2, carried inside a proof manifest.
- A #dfn[proof manifest] is the JSON interchange object of section 5 bundling sub-proofs,
  their public metadata, and bindings.
- A #dfn[trust anchor] is an input the verifier obtains out of band from the relying party —
  never from the manifest (section 8).
- A #dfn[holder] is the party that controls the credentials a proof draws on; a holder may
  be disclosed ("clear") or hidden behind a proof-of-knowledge tier (section 7.3).

== Conformance classes

This document defines three conformance classes:

+ a #dfn[zkSPARQL prover], which produces proof manifests (sections 3–6);
+ a #dfn[zkSPARQL verifier], which checks them (sections 6–8) — a conforming verifier
  #strong[MUST] implement *every* obligation of section 7 and #strong[MUST] fail closed on
  any check it cannot complete;
+ an #dfn[admissibility policy engine], which evaluates whether an annotated proof method
  satisfies an ODRL policy (sections 9–10).

#note[
  Only material in the numbered sections of this document is candidate-normative, and even
  that is a *proposal*: the reference implementation remains the source of truth wherever an
  Editor's note marks a detail as pending transcription.
]

= Committed data model

== Graph canonicalisation and commitment

Each source RDF graph is canonicalised and then committed:

+ The graph #strong[MUST] be canonicalised with RDF Dataset Canonicalization (RDFC-1.0)
  #cite("RDF-CANON"), so that commitment values are independent of blank-node labelling and
  triple order.
+ The commitment #strong[MUST] be computed with the Poseidon2 permutation #cite("POSEIDON2")
  over the BN254 scalar field, per graph (one commitment per source graph).

#note[
  Editor's note — the exact leaf encoding of canonical triples into field elements, and the
  hashing arrangement above the leaves, are pinned by the implementation and are to be
  transcribed into a subsequent draft; this draft does not respecify them. The implementation
  also carries an optional "dual-leaf" value lane whose invariants are an accepted, documented
  downgrade; it is opt-in and unaudited (see section 14.2).
]

== Issuer attestation signatures

An issuer attests a committed graph by signing its commitment:

+ The attestation signature scheme is Schnorr over the Baby Jubjub curve #cite("EIP2494")
  with a Poseidon2-derived challenge, identified by the cryptosuite IRI
  `https://sparq.dev/ns/zk#poseidon2-schnorr-v1`.
+ A verifier #strong[MUST] reject an attestation whose cryptosuite identifier it cannot
  resolve. There is #strong[no] default cryptosuite: an unresolved suite is a hard failure,
  not a fallback (fail-closed).

= Query fragment and circuit family

== The supported SPARQL fragment

The provable fragment of SPARQL #cite("SPARQL11-QUERY") is deliberately small and bucketed:

- basic graph pattern (BGP) scans over committed graphs;
- value `FILTER` constraints, bucketed by datatype lane: integer, `xsd:double`,
  signed integer, `xsd:decimal`, and a value-dictionary lane;
- a single-prover equality `JOIN` across hidden credentials.

`OPTIONAL`, `UNION`, property paths, aggregation, and subqueries are *not* part of the
fragment. A prover #strong[MUST NOT] emit a manifest claiming coverage of a construct outside
this fragment, and a verifier encountering such a claim #strong[MUST] reject it.

#note[
  Editor's note — the fragment is currently defined by the circuit family below rather than by
  a normative grammar. Producing a normative grammar (and its mapping onto the SPARQL algebra)
  is required future work for this specification.
]

== The circuit family

Each sub-proof is generated against exactly one circuit of a fixed, named family. Circuits are
authored in Noir #cite("NOIR") and proved with the Barretenberg backend (see section 13 on
toolchain pinning). The family is:

#table(
  columns: 2,
  align: (left, left),
  table.header[Circuit identifier][Statement proved (informative gloss)],
  [`Scan`], [A BGP scan matches against a committed graph.],
  [`FilterInt`], [An integer-lane FILTER constraint holds.],
  [`FilterF64`], [An `xsd:double`-lane FILTER constraint holds.],
  [`FilterSignedInt`], [A signed-integer-lane FILTER constraint holds.],
  [`FilterDecimal`], [An `xsd:decimal`-lane FILTER constraint holds.],
  [`FilterValueDl`], [A value-dictionary-lane FILTER constraint holds.],
  [`RevokeUnset`], [A revocation bit is unset in a committed status snapshot.],
  [`HiddenIssuer`], [The issuer of a hidden credential lies in an attested set.],
  [`HolderPok`], [Holder proof-of-knowledge (hidden-holder tier; see section 14.2).],
  [`HolderSet`], [Holder set membership (hidden-holder tier; see section 14.2).],
  [`JoinEq`], [Two hidden credentials agree on an equality join key.],
)

The circuit identifier bound into a manifest is re-derived by the verifier (section 7.2); a
manifest #strong[MUST NOT] be accepted on the strength of its self-declared identifier alone.

== Entailment regimes

Only *simple entailment* is proved in zero knowledge. A manifest #strong[MAY] declare RDFS/OWL
derivation steps, but these are re-checked by the verifier against *disclosed* bases
(obligation `bind_entailment`, section 7.3) — they are not proven in-circuit, and the
derivation bases are revealed to the verifier. An in-circuit closure proof is explicitly
deferred. A prover #strong[MUST NOT] represent a disclosed-base re-check as a zero-knowledge
entailment proof.

= The proof manifest

== Typing and canonical serialisation

A proof manifest is a JSON object. Its type member #strong[MUST] be the value
`urn:sparq:zk:ProofManifest`. Before any hash of the manifest is computed (for binding,
deduplication, or audit), the manifest #strong[MUST] first be put into its canonical
serialised form; hashing a non-canonical serialisation is not conformant.

The following sketch is illustrative only (non-normative) and deliberately incomplete:

```json
{
  "type": "urn:sparq:zk:ProofManifest",
  "key_set": [ "...issuer keys, accepted only as a subset of the external trust anchor K..." ],
  "sub_proofs": [ { "circuit": "Scan", "proof_hex": "..." } ]
}
```

#note[
  Editor's note — the complete member table of the manifest (bindings, attributions,
  entailment declarations, revocation references, holder-binding material) and the exact
  canonicalisation algorithm are pinned by the implementation and are to be transcribed into a
  subsequent draft. The manifest also has *no* JSON-LD context today; giving it one is a
  proposal in section 11.
]

== Sub-proof encoding

Each sub-proof is carried as a single hex-encoded blob with the layout
`len | proof | len | public-inputs | vk` — a length-prefixed proof, a length-prefixed
public-input segment, and the verification key.

== Public-input encoding

Within a sub-proof's public-input segment:

- each public-input field element #strong[MUST] be encoded as exactly 32 bytes, big-endian;
- structs and arrays #strong[MUST] be flattened in row-major order;
- booleans #strong[MUST] encode as `0` or `1`; `u32`/`u64` values encode as their integer
  value;
- the segment carries #strong[no] header and no per-element length prefix.

#note[
  This byte layout is *empirically pinned* to a specific Barretenberg release
  (`bb 5.0.0-nightly.20260324`, driven as a subprocess alongside `nargo 1.0.0-beta.21`); it
  was determined by observation, not derived from a backend specification. Making the layout
  normative and toolchain-independent is required future work (section 13) — until then,
  interoperability across backend versions is not promised.
]

= Challenge–response: the verifier nonce

The proof request is a nonce challenge:

+ The verifier #strong[MUST] mint a fresh #dfn[verifier nonce] — a BN254 scalar field
  element, exchanged in hexadecimal form — and deliver it to the prover *before* proving
  begins. Nonces #strong[MUST NOT] be reused across requests.
+ The prover #strong[MUST] commit the nonce as *public-input field 0 of every sub-proof* in
  the manifest.
+ The verifier #strong[MUST] record the nonce as used (single-use) *before* running the
  cryptographic gates of section 7.4, so that a manifest that fails late cannot be replayed
  against the same nonce.
+ If a manifest binds a nonce other than the one issued for the request, the verifier
  #strong[MUST] reject with a nonce-binding mismatch *and* #strong[MUST] still burn the
  issued nonce.

How the nonce is delivered and how the manifest is submitted is out of band and currently
unspecified; section 11 proposes a transport binding.

= Verification

== Entry points

The reference verifier exposes two entry points:

+ a *structural prefilter* covering stages 1–2 (shape and consistency checks), which is
  #strong[not] sufficient on its own and #strong[MUST NOT] be treated as verification; and
+ *full verification*, which runs the prefilter, the structural re-checks of section 7.2, the
  binding obligations of section 7.3, and the cryptographic gates of section 7.4, in a
  fail-closed pipeline.

Only full verification confers the (unaudited — section 14.1) checking this document
describes.

== Structural re-checks

Full verification #strong[MUST] re-check, independently of the prefilter:

- the *blank-node guard*: cross-graph joins on blank nodes are rejected (the "Q6" guard);
- *attribution arity*: the attribution structure is well-formed;
- *circuit-identifier re-derivation*: the circuit identifier each sub-proof claims is
  re-derived from the statement it is bound to, and the two must agree;
- *strictly-increasing commitment ordering*: the manifest's graph commitments are strictly
  increasing, giving a canonical order and excluding duplicates.

#note[
  Editor's note — the precise condition of the Q6 blank-node guard is pinned by the
  implementation and is to be transcribed into a subsequent draft.
]

== Binding obligations

A conforming verifier #strong[MUST] enforce all twelve binding obligations below. Each is
fail-closed; each has adversarial ("forge") tests in the reference implementation. The
one-line glosses are informative — the implementation remains normative for each obligation's
exact clause until a subsequent draft transcribes them.

#table(
  columns: 2,
  align: (left, left),
  table.header[Obligation][Informative gloss],
  [`bind_query_correctness`], [The proven circuit statements correspond to the claimed SPARQL
    query under the fragment semantics of section 4.],
  [`bind_attributions`], [The disclosed attribution set satisfies the superset rule binding
    result rows to source graphs.],
  [`bind_issuer_attestations`], [Every issuer key used is a member of the *external* trusted
    key set K (section 8) — never merely of the manifest's own key list.],
  [`bind_revocation`], [Revocation sub-proofs are bound to the authoritative status-list
    snapshot supplied by the relying party.],
  [`bind_joins`], [Join sub-proofs are bound to the participating graph commitments and
    variable slots.],
  [`bind_entailment`], [The declared entailment regime is honoured and derivation steps are
    re-checked against disclosed bases (section 4.3).],
  [`bind_holder_pop`], [Holder proof-of-possession is bound to the manifest.],
  [`bind_holder_binding`], [The clear (disclosed) holder is bound per the holder-binding
    policy and registry.],
  [`bind_hidden_revocation`], [Revocation checking for hidden credentials.],
  [`bind_hidden_issuer_attestations`], [Issuer attestation checking for hidden credentials.],
  [`bind_holder_pok`], [Hidden-holder proof-of-knowledge tier — explicitly *not yet sound*;
    opt-in only (section 14.2).],
  [`bind_holder_set`], [Hidden-holder set-membership tier — explicitly *not yet sound*;
    opt-in only (section 14.2).],
)

#note[
  Editor's note — the superset direction enforced by `bind_attributions` is pinned by the
  implementation; its exact statement is to be transcribed into a subsequent draft.
]

== Cryptographic gates

After the obligations above, a conforming verifier #strong[MUST] run all of the following,
in a fail-closed sequence:

+ *Public-input reconstruction* (audit gate 1): the verifier independently reconstructs the
  expected public-input bytes for every sub-proof — with the verifier nonce at field 0 — and
  compares them *byte-for-byte* against the manifest's public-input segment. Any difference
  is a rejection.
+ *Canonical verification-key recomputation* (audit gate 2): the verification key for each
  sub-proof #strong[MUST] be recomputed from the canonical circuit; a manifest-supplied key
  is never trusted as-is.
+ *Backend proof verification* (audit gate 3 companion): each sub-proof is verified by the
  proving backend against the recomputed key and reconstructed inputs. (Audit gate 3 proper —
  issuer-signature and key-set binding — is enforced per scan by
  `bind_issuer_attestations` above.)
+ *Nonce single-use and binding* (audit gate 4): as specified in section 6.

== Fail-closed error handling

Every failure mode maps to an explicit variant of a closed error taxonomy (roughly seventy
variants in the reference implementation). A conforming verifier #strong[MUST] reject the
whole manifest on the *first* failed check, #strong[MUST NOT] return partial results, and
#strong[MUST NOT] downgrade any error to a warning.

= External trust anchors

All trust roots are inputs from the relying party. A conforming verifier #strong[MUST]
obtain each of the following out of band and #strong[MUST NOT] accept any of them from the
manifest:

+ the *trusted issuer key set K* — the manifest #strong[MAY] carry its own key list, but it
  is accepted only if it is a *subset* of K;
+ the *authoritative status-list snapshot* governing revocation, per the relying party's
  revocation policy;
+ the *holder registry* and *holder-binding policy*;
+ the *fresh verifier nonce* of section 6;
+ the *seen-nonce store* enforcing single use — this store #strong[SHOULD] be durable across
  verifier restarts (the reference implementation provides both a durable file-backed store
  and an in-memory store; the in-memory store forgets burned nonces on restart).

#note[
  The subset rule for K is codified from experience: an earlier revision that trusted the
  manifest's own key list was a review-identified soundness hole. Externalising every trust
  anchor is what closed it.
]

= Security-properties vocabulary

zkSPARQL methods are *annotated* with machine-readable security properties so that policy
engines can reason about them (section 10). The vocabulary is layered.

== Base vocabulary

The base vocabulary is the vendored `sec-prop` ontology, namespace
`https://w3id.org/zkp-sparql/sec-prop#`, defining eight property dimensions:
`UnlinkabilityStrength`, `UnlinkabilityScope`, `PostQuantumForgery`, `PostQuantumSnooping`,
`SignatureTypeLeakage`, `ProofSizeLeakage`, `CircuitAudit`, and `ValidityPeriodLeakage`
#cite("SEC-PROP").

#note[
  Editor's note — the `w3id.org/zkp-sparql/` identifiers were minted as placeholders while
  the source repository was private. Before this draft advances, the permanent-identifier
  redirect must be confirmed live and stable.
]

== The secx extension

The sparq extension vocabulary (`secx`) adds the dimensions `ZeroKnowledgeType`, `Soundness`,
`Completeness`, `Hiding`, `Binding`, `Anonymity`, `Setup`, `Interactivity`,
`SelectiveDisclosure`, and `SingleUse`, plus four orthogonal axes:

- `AssuranceLevel`, ordered `Proven` > `Claimed` > `Conjectured`;
- `AuditStatus`, including the value `ExternalSignOffPending`;
- `Assumption` (e.g. the multi-party trust assumptions referenced by composed systems);
- `PropertyScope`, distinguishing `QueryProofLayer` from `SourceLayerOnly`.

A property that holds at the source layer only #strong[MUST NOT] be used to satisfy a
query-proof-layer constraint: source-layer facts do not transfer to the query-proof layer.

== The over-claim rule

While the external audit gate (sq-qhy4) is open:

+ No sparq zkSPARQL method #strong[MAY] be annotated `secx:Proven` for any *positive* privacy
  or soundness property; such properties are at most `secx:Claimed` with
  `AuditStatus ExternalSignOffPending`.
+ Only *settled negative* facts — for example `PQForgeable`, `Replayable`, `SchemeRevealed`
  — #strong[MAY] carry `Proven`.

The reference implementation enforces this rule mechanically with three machine-checkable
guards over the annotation graph: an over-claim guard, a source-layer-transfer guard, and a
completeness guard.

= Policy-controlled admissibility

Relying parties express *which* proof methods they accept as ODRL 2.2 policies
#cite("ODRL22") over the vocabulary of section 9:

+ A policy using any `secx:requires` left-operand #strong[MUST] assert
  `odrl:profile` `https://sparq.dev/ns/odrl-secprop-profile#`.
+ Each such left-operand carries exactly one `secx:overDimension` fact identifying the
  property dimension it constrains.
+ Only the operator `odrl:gteq` is given a reduction; a constraint using any other operator
  #strong[MUST] be treated as *unsatisfied* — which denies.
+ A method is admissible only if it satisfies *every* constraint of the policy
  (default-deny).
+ In the fail-closed pre-check gate, the outcomes are Admitted, Denied, and ReductionError —
  and a reduction error #strong[MUST] be treated as a denial.

Base admission additionally checks the issuer's Schnorr signature over the RDFC-1.0
commitment, a SHACL #cite("SHACL") scope constraint, a reserved-predicate guard, and — for
clear holders — a WebID holder binding.

#note[
  A consequence worth stating plainly: a policy requiring
  `requiresAssurance odrl:gteq secx:Proven` on a positive property mechanically denies *every*
  current sparq zkSPARQL method while the external audit (sq-qhy4) is open. That is by design
  — it is the honest default for high-assurance relying parties.
]

= Transport, media type, and interchange

This section is entirely a *proposal*: none of it exists in the reference implementation
today. The manifest is a bare JSON object tagged with a URN, and both nonce issuance and
manifest submission are out of band.

== Media type (proposal)

A registered media type is proposed for the proof manifest — candidate
`application/vc+zksparql+json`, with a `+ld` variant once a JSON-LD context exists — and a
companion type for the nonce challenge. Until registration, implementations exchanging
manifests over HTTP have no content-type contract at all.

== JSON-LD context (proposal)

A JSON-LD context for the manifest is proposed so that a manifest can round-trip as a W3C
Verifiable Presentation #cite("VC-DATA-MODEL") and be consumed by generic data-integrity
processors. No such context exists today; the manifest does not currently round-trip.

== Wire protocol (proposal)

A challenge–response HTTP binding is proposed: an endpoint issuing single-use nonces and an
endpoint accepting manifest submissions bound to them. A live-service and asynchronous-proving
posture has been *designed only* (no server endpoint, job model, or async proving exists in
the implementation); any binding written here would be speculative and is deferred to a
subsequent draft.

= Relationship to W3C Verifiable Credentials

This section is informative.

- A *VC cryptosuite bridge* — off-circuit Data-Integrity verification of `eddsa-rdfc-2022`
  and `ecdsa-rdfc-2019` (P-256) source credentials #cite("VC-DI") at ingest — is implemented
  but *unmerged* at the time of writing (it exists only on a feature branch, as an opt-in
  feature). Any claim of VC ingest #strong[MUST] be caveated accordingly.
- The P-384 profile of `ecdsa-rdfc-2019` is *not* implemented and fails closed as an
  unsupported key curve.
- Ingest of `bbs-2023` / `ecdsa-sd-2023` selective-disclosure credentials is an explicitly
  *deferred* seam: there is no in-repo BBS verifier.
- In-circuit re-verification of the source credential's proof is deliberately *out of scope*:
  the query proof does not re-verify the source VC signature inside the circuit. The
  `zk:sourceCryptosuite` annotation is provenance only, and #strong[MUST NOT] be read as
  evidence that the source proof was verified in zero knowledge.

= Conformance testing and toolchain pinning

Two conformance gaps are open:

+ *No portable test vectors.* Adversarial forge tests exist for the manifest format and for
  each obligation of section 7.3, but they are internal to the Rust implementation. A
  conformance suite of portable fixtures (manifests that must verify, and mutated manifests
  that must fail with a specific error class) is required future work.
+ *Toolchain pinning.* The circuit family is pinned to an external toolchain
  (`nargo 1.0.0-beta.21`, `bb 5.0.0-nightly.20260324`) driven by subprocess, and the
  public-input byte layout of section 5.3 is empirically determined. Until the layout is
  specified normatively, cross-version interoperability is out of reach and conformance can
  only be claimed against the pinned toolchain.

= Security and Privacy Considerations

== Audit status

The entire zkSPARQL estate is research-grade and has *not* been externally audited; the
external cryptographer audit is an open gate (sq-qhy4). Accordingly:

+ A relying party #strong[MUST NOT] treat a passing verification as a settled guarantee that
  the proven SPARQL statement holds against an adversarial prover.
+ Soundness and attestation are *not production-ready*; deployments that need a production
  guarantee are out of scope for this draft until the audit closes.
+ The over-claim rule of section 9.3 applies to every annotation surface: positive properties
  are at most `Claimed`, with audit status `ExternalSignOffPending`.

== Known-unsound and downgraded components

- The hidden-holder tiers (`bind_holder_pok`, `bind_holder_set`) are explicitly labelled *not
  yet sound* in the implementation and its documentation; remediation is tracked internally.
  They are opt-in only, and verifiers #strong[SHOULD] leave them disabled unless the residual
  risk is understood and accepted.
- The optional dual-leaf value lane carries an accepted, documented invariant downgrade and
  is likewise unaudited; it is opt-in.
- Only simple entailment is proved in zero knowledge; RDFS/OWL derivations are disclosed-base
  re-checks (section 4.3), which both weakens the zero-knowledge property for the disclosed
  bases and limits the entailment coverage.

== Post-quantum posture

The post-quantum posture is a *settled negative*. All issuer signature suites in scope
(Schnorr over Baby Jubjub, EdDSA, BBS+) rest on discrete-log hardness and fall to a
Shor-capable adversary; commitment binding likewise breaks under a cryptographically relevant
quantum computer, so *retrospective* soundness of previously accepted proofs fails as well.
The vocabulary records this honestly as negative `PostQuantumForgery` / `PostQuantumSnooping`
facts — these negatives are among the few annotations permitted to carry `Proven`
(section 9.3).

== Leakage and unlinkability

`SignatureTypeLeakage`, `ProofSizeLeakage`, `ValidityPeriodLeakage`, and the unlinkability
dimensions are tracked as vocabulary dimensions so that policies can constrain them; their
values for sparq methods are at most `Claimed` and are *not* settled guarantees. Verifiers and
relying parties should assume that proof size, timing, and suite choice may leak information
about the underlying credentials until an audit says otherwise.

== Replay and nonce hygiene

Replay resistance rests entirely on the nonce discipline of section 6: single-use recording
*before* the cryptographic gates, burn-on-mismatch, and a durable seen-nonce store. A verifier
using a non-durable store forgets burned nonces on restart and #strong[SHOULD NOT] be exposed
where replay across restarts matters.

== Admissibility reasons over annotations, not cryptography

The admissibility engine of section 10 reasons over *declared annotations*, not over the
cryptography itself. An "Admitted" outcome means the method's declared properties satisfy the
policy — it is not, and must not be presented as, an independent cryptographic finding.

= References

#references((
  ("RFC2119", [Bradner, S. #emph[Key words for use in RFCs to Indicate Requirement Levels].
    RFC 2119, IETF, March 1997.]),
  ("RFC8174", [Leiba, B. #emph[Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words].
    RFC 8174, IETF, May 2017.]),
  ("SPARQL11-QUERY", [Harris, S.; Seaborne, A. (eds). #emph[SPARQL 1.1 Query Language].
    W3C Recommendation, 21 March 2013. https://www.w3.org/TR/sparql11-query/.]),
  ("RDF11-CONCEPTS", [Cyganiak, R.; Wood, D.; Lanthaler, M. (eds). #emph[RDF 1.1 Concepts and
    Abstract Syntax]. W3C Recommendation, 25 February 2014.
    https://www.w3.org/TR/rdf11-concepts/.]),
  ("RDF-CANON", [Longley, D.; Kellogg, G.; et al. (eds). #emph[RDF Dataset Canonicalization
    (RDFC-1.0)]. W3C Recommendation, 2024. https://www.w3.org/TR/rdf-canon/.]),
  ("POSEIDON2", [Grassi, L.; Khovratovich, D.; Schofnegger, M. #emph[Poseidon2: A Faster
    Version of the Poseidon Hash Function]. AFRICACRYPT 2023; IACR ePrint 2023/323.]),
  ("EIP2494", [Bellés-Muñoz, M.; Baylina, J. #emph[EIP-2494: Baby Jubjub Elliptic Curve].
    Ethereum Improvement Proposals, 2020.]),
  ("NOIR", [Aztec Labs. #emph[The Noir Programming Language]. https://noir-lang.org/.]),
  ("VC-DATA-MODEL", [Sporny, M.; et al. (eds). #emph[Verifiable Credentials Data Model v2.0].
    W3C Recommendation, 2025. https://www.w3.org/TR/vc-data-model-2.0/.]),
  ("VC-DI", [Sporny, M.; Longley, D.; et al. (eds). #emph[Verifiable Credential Data
    Integrity 1.0] and its cryptosuites (eddsa-rdfc-2022, ecdsa-rdfc-2019, bbs-2023).
    W3C Recommendations, 2025. https://www.w3.org/TR/vc-data-integrity/.]),
  ("ODRL22", [Iannella, R.; Villata, S. (eds). #emph[ODRL Information Model 2.2].
    W3C Recommendation, 15 February 2018. https://www.w3.org/TR/odrl-model/.]),
  ("SHACL", [Knublauch, H.; Kontokostas, D. (eds). #emph[Shapes Constraint Language (SHACL)].
    W3C Recommendation, 20 July 2017. https://www.w3.org/TR/shacl/.]),
  ("SEC-PROP", [#emph[sec-prop: a security-properties vocabulary for zero-knowledge proof
    systems]. Namespace `https://w3id.org/zkp-sparql/sec-prop#`; vendored, with the sparq
    `secx` extension, in the sparq repository (MIT).]),
  ("SPARQ", [The sparq project. #emph[sparq: an RDF + SPARQL engine with a zero-knowledge
    query-proof estate (reference implementation)]. https://github.com/jeswr/sparq.]),
))
