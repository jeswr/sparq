<!-- [GPT-5.6] sq-ry0by. 🤖 SPARQ agent — certification-edge closure semantics. -->
# Certification-edge closure semantics

> **Status:** This note describes the shipped, default-off `cert-graph` research
> prototype. It is intended for human review, including review by the Solid community;
> it does not define a standard or make a security guarantee.

## Where the closure fits

`derive_effective_rules` runs before the unchanged admission gate. It starts with the
pod's direct `TrustRule` anchors and examines signed `trustx:Certification` edges. A
surviving edge appends a rule for the certified issuer; a rejected edge contributes
nothing. Derived rules are not reused as anchors, so the shipped closure is depth one.
With a depth bound of zero, or with no surviving edge, the direct rules are returned
unchanged and in their original order.

The closure is fail-closed and attenuation-only. A derived rule cannot exceed the
authority of the direct anchor used to derive it:

- its resource scope remains the anchor's scope;
- its freshness is the smaller of the anchor freshness and the remaining certificate
  window; and
- its statement shape must be provably contained by the anchor shape.

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
| Depth | The configured bound permits the edge. The shipped implementation accepts direct-anchor edges only. | `EdgeRejection::OverDepth` |
| Cycle | The edge is not self-certifying, and the certified issuer is not already a matching direct anchor. | `EdgeRejection::Cyclic` |
| Anchor and key | A direct rule anchors the named certifier under the same verification key. | `EdgeRejection::NoAnchor` |
| Signature | The domain-separated certification message has a decodable signature that verifies under the anchored certifier key. | `EdgeRejection::SignatureInvalid` |
| Validity window | The inclusive window is well formed and contains the evaluation instant. | `EdgeRejection::OutOfWindow` |
| Attenuation | At least one matching anchor is provably narrowed by the certificate scope under the containment rule above. | `EdgeRejection::Broadening` |

Passing every gate produces a rule for the certified issuer and its signed key. The
chosen direct anchor still bounds the resource scope, statement shape, and freshness.
An edge can therefore narrow who may issue what, where, and for how long; it cannot
create authority outside a direct pod anchor.

## Honest scope

This is a clear-path authorisation preprocessing layer. It verifies the certificate
edge's existing signature primitive, but makes **no zero-knowledge, privacy, anonymity,
unlinkability, or settled cryptographic-soundness claim**. Framework trust and the
certifier's anchor key remain trust inputs rather than facts proven by this closure.
External accredited-cryptographer review of the related ZK estate is still pending
under `sq-qhy4`. The implementation is research-grade and must not be presented as
closing that review gate.
