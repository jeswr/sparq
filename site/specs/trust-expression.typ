// [SONNET-4.6] sq-6syab.1 — Trust Expression verifier-holder contract.
// Honesty: the ZK estate is internally re-audited but has no external accredited-
// cryptographer sign-off (sq-qhy4 remains open); sparq-mpc is honest-majority
// semi-honest only. This proposal makes no unqualified privacy or soundness claim.

#import "_lib/spec.typ": spec-head, sotd, intro-section, references, dfn, note, cite

#set document(title: "Trust Expression: A Verifier–Holder Contract for Framework-Anchored Attestations")
#set text(size: 11pt)
#set par(justify: true)
#set heading(numbering: "1.")

#spec-head()

#intro-section("abstract", "Abstract")[
  This document defines a small verifier-to-holder contract for asking a SPARQL question
  subject to explicit source-trust requirements. A request contains exactly one SPARQL query,
  one RDF trust envelope, and one nonce. The envelope supports directly enumerated issuers and
  issuers certified by a governance framework. In both modes, acceptance depends on positive,
  time-windowed status attestations; absence is never interpreted as non-revocation. The
  response uses RDF 1.2 reifiers with PROV-O attribution normatively, with an informative,
  lossless named-graph mapping for SPARQL 1.1 systems.
]

#sotd()

#intro-section("security-standing", "Security standing")[
  This proposal defines data shapes and fail-closed verification obligations, not a
  cryptographic guarantee. The optional ZK realization builds on
  `sparq_zk_compose::verifier::bind_issuer_attestations` and
  `sparq_zk_compose::verifier::bind_hidden_issuer_attestations`; that estate is internally
  re-audited but has no external accredited-cryptographer sign-off while *sq-qhy4* remains
  open. The optional MPC realization builds on the verifier-side attestation gate in
  `sparq_mpc` and is honest-majority semi-honest only. Conformance means that an
  implementation follows this document and rejects unmet requirements; it does not establish
  an externally audited privacy or soundness property.
]

= Introduction

Trust frameworks commonly publish positive artifacts: trusted-list or register entries
identify certified issuers, and status services attest credential state. A verifier should be
able to ask a holder one ordinary SPARQL question while stating which such artifacts it will
accept, without adding keywords to SPARQL or hiding policy in implementation configuration.

This document therefore defines #dfn[Trust Expression] as a thin contract over RDF and
SPARQL. Trust requirements are data. Query evaluation remains ordinary SPARQL over the
admissible portion of a response dataset.

== Design goals

- One query, one reusable RDF envelope, and one nonce; no new SPARQL syntax.
- Two composable trust modes: enumerated parties and framework-certified issuers.
- Open-world, monotone non-revocation expressed by existence of positive evidence.
- A certification-scope layer capable of service-, type-, predicate-set-, and shape-level
  certification.
- Statement-level provenance in the response using RDF 1.2 reifiers and PROV-O.

= Terminology and conformance

The key words #strong[MUST], #strong[MUST NOT], #strong[REQUIRED], #strong[SHALL],
#strong[SHALL NOT], #strong[SHOULD], #strong[SHOULD NOT], #strong[RECOMMENDED],
#strong[MAY], and #strong[OPTIONAL] are to be interpreted as described in
#cite("RFC2119") and #cite("RFC8174") when they appear in all capitals.

A #dfn[verifier] sends a request. A #dfn[holder] evaluates it and returns a response. A
#dfn[contributing statement] is an RDF statement used in deriving a returned query answer. A
#dfn[trust envelope] is the RDF graph containing one or more
`trustx:TrustRequirements` resources. A #dfn[reference instant] is the
`xsd:dateTime` selected by `trustx:requiresValidStatusAt`.

A conforming holder #strong[MUST] implement sections 3 through 7. A conforming verifier
#strong[MUST] independently check the response as specified in section 7. An implementation
#strong[MUST] reject malformed, ambiguous, stale, or incomplete evidence; it #strong[MUST
NOT] turn missing evidence into a positive result.

= Request contract

== Three request components

[TX-REQ-001] A request #strong[MUST] contain exactly:

+ one SPARQL 1.1 `ASK` or `SELECT` query $Q$;
+ one RDF trust envelope $T R$; and
+ one unpredictable nonce supplied by the verifier.

[TX-REQ-002] The envelope #strong[MUST] contain at least one
`trustx:TrustRequirements` resource whose `trustx:question` identifies $Q$, and exactly
one `trustx:requiresValidStatusAt` reference instant per resource.

[TX-REQ-003] The holder #strong[MUST] bind the nonce, the canonical query representation,
and the canonical trust envelope to the response. A replay with another nonce or a response
whose query or envelope differs #strong[MUST] be rejected.

#note[
  The nonce is the freshness challenge already represented by the zkSPARQL manifest; this
  proposal does not invent a second challenge protocol. The relevant estate item is
  `sparq_zk_compose::manifest::ProofManifest`.
]

== One query

The query expresses only the data question. Trust conditions #strong[MUST NOT] require
extension keywords, magic `SERVICE` IRIs, or implementation-specific query syntax. For
example:

```sparql
PREFIX ex: <https://example.test/vocab#>
ASK {
  <did:example:alice> ex:age ?age .
  FILTER (?age >= 18)
}
```

= Trust envelope and trust modes

This section normatively defines the `trustx:` certification-scope layer inline. Until a
standards body assigns a namespace, `trustx:` denotes
`https://w3id.org/sparq/trust-expression#`; these terms are non-standard proposal terms.

== Common shape

```turtle
@prefix trustx: <https://w3id.org/sparq/trust-expression#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

<urn:requirements:age> a trustx:TrustRequirements ;
  trustx:question <urn:query:age> ;
  trustx:requiresValidStatusAt "2026-07-05T00:00:00Z"^^xsd:dateTime .
```

Multiple requirements resources express alternatives: a contributing statement is
admissible when it satisfies every condition of at least one resource. Conditions within
one resource are AND-combined.

== Enumerated-parties mode

[TX-MODE-ENUM] An enumerated-parties requirement #strong[MUST] contain one or more
`trustx:trustsIssuer` values. Every contributing statement admitted under that requirement
#strong[MUST] be attributed to one listed issuer and have a covering status attestation.

```turtle
<urn:requirements:direct> a trustx:TrustRequirements ;
  trustx:question <urn:query:age> ;
  trustx:trustsIssuer <did:web:x.example>, <did:web:y.example>,
                      <did:web:z.example> ;
  trustx:requiresValidStatusAt "2026-07-05T00:00:00Z"^^xsd:dateTime .
```

Issuer-to-key binding is an external trust anchor. A DID spelling alone does not establish
the binding.

== Framework-certified mode

[TX-MODE-FRAMEWORK] A framework requirement #strong[MUST] contain
`trustx:trustsFramework` and `trustx:requiresScopeConformance true`. Each contributing
statement #strong[MUST] be attributed to an issuer covered at the reference instant by a
valid `trustx:Certification` under that framework, and #strong[MUST] conform to its scope.

```turtle
<urn:requirements:framework> a trustx:TrustRequirements ;
  trustx:question <urn:query:age> ;
  trustx:trustsFramework trustx:eIDAS2 ;
  trustx:requiresScopeConformance true ;
  trustx:requiresValidStatusAt "2026-07-05T00:00:00Z"^^xsd:dateTime .
```

`trustx:eIDAS2` and `trustx:DIATF` #strong[SHOULD] use `rdfs:seeAlso` to connect to the
corresponding regulatory individuals vendored in
`sparq_trust::secprop` rather than minting duplicate regulatory requirements.

== Combining the modes

The two examples above, placed in one envelope and bound to the same query, express direct
issuer trust #emph[or] framework certification. A holder #strong[MUST NOT] combine a
partially satisfied direct requirement with a partially satisfied framework requirement.

= Positive status and non-revocation

[TX-STATUS-001] “Unrevoked” #strong[MUST] mean that a signed
`trustx:StatusAttestation` positively states `trustx:status trustx:valid` for the credential
and its validity interval covers the reference instant:

```turtle
_:status a trustx:StatusAttestation ;
  trustx:credential <urn:credential:age-1> ;
  trustx:status trustx:valid ;
  trustx:validFrom "2026-07-01T00:00:00Z"^^xsd:dateTime ;
  trustx:validUntil "2026-07-08T00:00:00Z"^^xsd:dateTime ;
  prov:wasAttributedTo <did:web:status.example> .
```

[TX-STATUS-002] Absence of a revocation statement, absence of a status-list entry, or a
failed lookup #strong[MUST NOT] be treated as valid status. Expired, not-yet-valid,
unverifiable, or conflicting evidence #strong[MUST] fail closed.

This rule is monotone under RDF open-world semantics: adding a covering positive attestation
can establish admissibility; lack of a revocation triple cannot. The clear-path estate item is
`sparq_trust::status_list::VerifyingLiveStatusCheck`; signed lists are represented by
`sparq_trust::status_list::SignedStatusList`, and RDF justification is produced by
`sparq_trust::status_list::justify_status_decision`. The optional ZK realization uses
`sparq_zk_compose::revocation` at an authoritative snapshot, subject to the audit caveat in
Security Considerations.

= Normative certification-scope vocabulary

The following terms and meanings are normative:

#table(
  columns: 2,
  align: (left, left),
  table.header[Term][Normative meaning],
  [`trustx:Framework`], [A governance framework operating a trusted list or register.],
  [`trustx:Certification`], [A signed, time-windowed attestation that an issuer is certified
    under a framework for a scope.],
  [`trustx:certifies`], [Relates a certification to the certified issuer.],
  [`trustx:underFramework`], [Relates a certification to exactly one framework.],
  [`trustx:scope`], [Relates a certification to what the issuer is certified to issue.],
  [`trustx:AnyServiceScope`], [The whole certified service, without claiming attribute-level
    granularity.],
  [`trustx:validFrom`, `trustx:validUntil`], [Inclusive start and exclusive end of the
    certification validity interval.],
)

[TX-SCOPE-001] A certification #strong[MUST] identify its issuer, framework, scope,
validity interval, certifying authority, and verifiable attestation binding.

[TX-SCOPE-002] A scope #strong[MAY] be `trustx:AnyServiceScope`, an attestation-type or
rulebook IRI, a predicate set, or a SHACL shape. Predicate scoping #strong[SHOULD] reuse the
`trust:forPredicate` to `trust:forShape` desugaring implemented by
`sparq_trust::vocab::desugar_for_predicate`.

[TX-SCOPE-003] For every contributing statement, a holder #strong[MUST] demonstrate that
its type or predicate is included by the issuer's certification scope at the reference
instant. `trustx:AnyServiceScope` includes statements issued by that certified service; it
does not imply attribute-level approval.

[TX-SCOPE-004] Scope conformance constrains what a verifier accepts. A certification is a
trust-anchored delegation statement; it does not prove that an issuer never produced material
outside its certified scope.

= Response encoding

== RDF 1.2 reifiers (normative)

[TX-RESP-001] The response #strong[MUST] contain the query answer and, for every
contributing statement, an RDF 1.2 reifier whose `rdf:reifies` object is the corresponding
triple term. The reifier #strong[MUST] carry `prov:wasAttributedTo` for its issuer and
`trustx:coveredBy` for the covering positive status attestation. Framework-mode responses
#strong[MUST] also carry `trustx:underCertification`.

```turtle
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix prov: <http://www.w3.org/ns/prov#> .
@prefix ex: <https://example.test/vocab#> .

_:claim rdf:reifies <<( <did:example:alice> ex:age 25 )>> ;
  prov:wasAttributedTo <did:web:x.example> ;
  trustx:coveredBy _:status ;
  trustx:underCertification _:certification .
```

The response #strong[MUST] be sufficient for the verifier to reconstruct the contributing
statement, issuer, status evidence, certification, framework, scope, and validity windows.
RDF 1.2 triple-term parsing in the reference estate is provided by
`sparq_core::nt::triple_term`; the SPARQL triple-term matching surface is not yet implemented.

== Lossless named-graph and PROV-O mapping (informative)

For SPARQL 1.1 systems, each reifier can be mapped to a named graph. Assign the reifier a
stable IRI $g$; put the reified triple alone in graph $g$; and put all properties whose
subject is the reifier into a metadata graph, replacing that subject with $g$. The inverse
maps each such graph IRI back to one reifier and its sole triple term.

```trig
<urn:reifier:claim> {
  <did:example:alice> ex:age 25 .
}
<urn:response:metadata> {
  <urn:reifier:claim> prov:wasAttributedTo <did:web:x.example> ;
    trustx:coveredBy _:status ;
    trustx:underCertification _:certification .
}
```

The mapping is lossless only when each mapped named graph contains exactly one triple and graph
IRIs do not collide. This mapping is informative; it does not replace the normative encoding.

= Evaluation and verification

[TX-EVAL-001] A holder #strong[MUST] evaluate $Q$ only over statements satisfying one
complete requirements resource. It #strong[MUST] return provenance for every contributing
statement and #strong[MUST] bind the response to the nonce, $Q$, and $T R$.

[TX-EVAL-002] A verifier #strong[MUST] independently check issuer attribution, signatures,
positive status coverage, certification validity, framework identity, and scope conformance.
It then evaluates a reference query $Q'$: $Q$'s graph patterns conjoined with the
admissibility patterns generated from $T R$ over the provenance response. Failure to derive
an admissible answer #strong[MUST] yield no binding (or false for `ASK`).

[TX-EVAL-003] An optional `trustx:methodPolicy` may identify an ODRL proof-method policy.
Such a policy is orthogonal to source trust and is evaluated by
`sparq_trust::admissibility::admissible`; this document does not restate that algorithm.

In a federated realization, mode 1 enumerates and mode 2 derives the trusted issuer-key set
consumed by `sparq_zk_compose::verifier::bind_issuer_attestations` or
`sparq_zk_compose::verifier::bind_hidden_issuer_attestations`. No additional MPC protocol is
specified here; any use of `sparq_mpc` retains its honest-majority semi-honest limitation.

= Security Considerations

== Cryptographic standing

The ZK path rests on internal re-audit only and has no external accredited-cryptographer
sign-off while `sq-qhy4` is open. MPC is honest-majority semi-honest only. Requirements in
this document say “conforms” or “fails closed”; they do not establish an externally audited
soundness or privacy guarantee.

== Framework anchors and scope

Framework-certified mode bottoms out in signed certification or trusted-list artifacts
published by the framework operator. Scope conformance constrains what the verifier accepts
and records what the issuer was authorized to issue. It cannot retroactively establish that
the issuer never issued outside that scope.

== Clear-path disclosure and completeness

Without an optional ZK manifest, the verifier re-checks admissibility over the returned RDF
and verifies the underlying attestation signatures, but must account for the completeness of
what the holder chose to disclose. Under the open-world assumption this contract asks only
monotone existence questions; it does not infer negative facts from omission.

== Freshness and caching

Status attestations have validity windows. A status change inside a window is not visible
until newer positive evidence is obtained. The verifier chooses the reference instant and
acceptable windows in the envelope; this proposal defines no universal duration.

== SPARQL 1.2 implementation gap

The normative encoding uses RDF 1.2 reifiers. The reference estate parses triple terms through
`sparq_core::nt::triple_term` but cannot yet match triple terms in SPARQL. The informative
named-graph mapping keeps the reference verification query runnable by SPARQL 1.1 systems.

== Performance evidence

This proposal states no performance result. Any integration analysis must draw measurements
from the project's canonical evidence pipeline; local work-box timings are non-canonical.

= Conformance cases

A conformance suite #strong[SHOULD] cover: enumerated-mode success; revoked and stale status;
untrusted issuer; framework-mode success; out-of-scope issuance; expired or invalid
certification; and a lossless round trip between the normative and informative encodings.
Every negative case #strong[MUST] fail closed.

= References

#references((
  ("RFC2119", [Bradner, S. #emph[Key words for use in RFCs to Indicate Requirement Levels].
    RFC 2119, IETF.]),
  ("RFC8174", [Leiba, B. #emph[Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words].
    RFC 8174, IETF.]),
  ("RDF12", [W3C. #emph[RDF 1.2 Concepts and Abstract Syntax]. Candidate Recommendation.]),
  ("SPARQL11", [W3C. #emph[SPARQL 1.1 Query Language]. Recommendation.]),
  ("PROVO", [W3C. #emph[PROV-O: The PROV Ontology]. Recommendation.]),
  ("SHACL", [W3C. #emph[Shapes Constraint Language (SHACL)]. Recommendation.]),
))
