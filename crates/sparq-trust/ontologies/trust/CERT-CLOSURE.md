<!-- [GPT-5.6] sq-ry0by. 🤖 SPARQ agent — certification-edge closure semantics. -->
# Certification-edge closure semantics

> **Status:** This note describes the shipped, default-off `cert-graph` research
> prototype. It is intended for human review, including review by the Solid community;
> it does not define a standard or make a security guarantee.

## Where the closure fits

`derive_effective_rules` runs before the unchanged admission gate. It starts with the
pod's direct `TrustRule` anchors and examines signed `trustx:Certification` edges. A
surviving edge appends a rule for the certified issuer; a rejected edge contributes
nothing. The closure runs up to `depth_bound` rounds: the first round's anchors are the
direct rules, and each later round's anchors are exactly the rules the previous round
derived — so a framework-of-frameworks chain (A certifies B, B certifies C) derives a
rule for C at a bound of two or more. A chain longer than the bound has its tail left
underived. With a depth bound of zero, or with no surviving edge, the direct rules are
returned unchanged and in their original order.

A rule derived at one round may act as a certifier at the next only if its own statement
shape selects certification statements: `sh:targetSubjectsOf` or `sh:targetObjectsOf` over
`trustx:certifies`, or `sh:targetClass` over `trustx:Certification`. A certification confers
the authority to issue statements of a scope, not the authority to confer, which is
stronger. Being certified for an attribute therefore makes an entity an issuer, not a
registrar, and its derived rule cannot extend a chain even to a target inside its own shape.
Direct anchors are exempt, because those are the pod's own explicit decision.

Depth cannot multiply authority. Each edge contributes at most one rule to the whole
closure, at the shallowest round that admits it, so the derived rule count never exceeds
the edge count whatever the bound. The cycle gate enforces that: once an edge has fired,
its certified issuer holds a matching-key rule, which the gate then rejects. A visited
set keyed on the certifier and certified-issuer identities states the same bound
redundantly and is not the guard.

The closure is fail-closed and attenuation-only. A derived rule cannot exceed the
authority of the direct anchor used to derive it:

- its resource scope remains the anchor's scope;
- its freshness is the smaller of the anchor freshness and the remaining certificate
  window; and
- its statement shape must be provably contained by the anchor shape.

Each of these is re-derived at every round against the rule the previous round produced,
not against the root anchor, so the chain telescopes: a rule derived at round N is
contained in the one derived at round N-1, and so on back to a direct anchor. An edge
that broadens at any round is rejected at that round, which also leaves the rest of its
chain underived.

Shape containment has two directions. Targets are **contravariant**: the certificate's
target set must be a subset of the anchor's target set, because an additional SHACL
target selects more nodes. Conformance constraints are **covariant**: the certificate
must contain at least the anchor's constraints, because additional constraints reduce
the set of conforming nodes. If the bounded structural matcher cannot prove both
conditions, the edge is rejected rather than guessed safe. `AnyService` adds no shape
of its own and therefore inherits the anchor shape; it does not remove the ceiling.

## Gates and rejection outcomes

`explain_edge` reports the first failed gate. The implementation evaluates the gates in
the following order, except that the depth-zero check occurs before per-edge evaluation.

| Gate | Required condition | Rejection outcome |
| --- | --- | --- |
| Depth | The configured bound is not zero. A bound of zero permits no round at all. | `EdgeRejection::OverDepth` |
| Cycle | The edge is not self-certifying, and the certified issuer does not already hold a matching-key rule in the closure so far, whether direct or derived at an earlier round. | `EdgeRejection::Cyclic` |
| Anchor and key | A rule available to this round anchors the named certifier under the same verification key. That is the direct rules at round one, and thereafter the previous round's derived rules that also select certification statements — so an edge deeper than the bound, or one whose certifier holds only an attribute scope, is never evaluated and reports this outcome rather than `EdgeRejection::OverDepth`. | `EdgeRejection::NoAnchor` |
| Signature | The domain-separated certification message has a decodable signature that verifies under the anchored certifier key. | `EdgeRejection::SignatureInvalid` |
| Validity window | The inclusive window is well formed and contains the evaluation instant. | `EdgeRejection::OutOfWindow` |
| Attenuation | At least one matching anchor for this round is provably narrowed by the certificate scope under the containment rule above. | `EdgeRejection::Broadening` |

Passing every gate produces a rule for the certified issuer and its signed key. The
chosen anchor for that round still bounds the resource scope, statement shape, and
freshness, and that anchor is itself bounded by a direct pod anchor. An edge can
therefore narrow who may issue what, where, and for how long; it cannot create authority
outside a direct pod anchor, at any depth.

## Honest scope

This is a clear-path authorisation preprocessing layer. It verifies the certificate
edge's existing signature primitive, but makes **no zero-knowledge, privacy, anonymity,
unlinkability, or settled cryptographic-soundness claim**. Framework trust and the
certifier's anchor key remain trust inputs rather than facts proven by this closure.
External accredited-cryptographer review of the related ZK estate is still pending
under `sq-qhy4`. The implementation is research-grade and must not be presented as
closing that review gate.
