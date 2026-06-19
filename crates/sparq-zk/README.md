<!-- [OPUS-4.8] sq-4kr5: internal-stub README for a publish=false crate. -->
# sparq-zk

The **engine-side ZK foundation** for [sparq](../../README.md): the off-circuit
**commitment pipeline** and the **zk-trace seam** behind the
[derived-credentials design](../../research/zkp-query-proofs-plan.md). RDFC10
canonicalization, Poseidon2-BN254 per-graph commitments, graph-scoped term/triple
field encoding, issuer signatures, and the `<urn:sparq:zk>` registry plumbing.

Why it exists: everything here is off-circuit Rust whose outputs (BN254 field
elements, leaf orderings, witness input sets) are exactly what the later Noir
circuits and proof composition will consume — bit-compatible with
`noir-lang/poseidon` and validated against the W3C `rdf-canon` test suite.

> **Internal crate — not published** to crates.io (`publish = false`). Circuits and
> proof composition are later deliverables; **no soundness or privacy claim** is made
> for this pipeline today. A config-only **commitment-method registry**
> (`commit::CommitmentMethod`: closed, fail-closed selection over `zk:scheme` —
> `string-canonical` default · `dual-leaf` · the OFF-by-default `commitment-value-only`
> research dial; sq-zzxt) records the method but adds no circuit; the value-hook /
> `dual-leaf` encoding itself is **PLANNED, NOT implemented, audit-gated**, and `dual-leaf`
> carries the #769-accepted INV-VL downgrade — open external-audit obligations
> (**CR-G8** / `sq-qhy4`; design `research/zk-configurable-commitment-design.md`).
> <!-- [OPUS-4.8] privacy-claims-allow: unimplemented audit-gated encoding + opt-in config plumbing, registered as an OPEN obligation; asserts no soundness/privacy property; sq-qhy4 / CR-G8 -->

Design: [`research/zkp-query-proofs-plan.md`](../../research/zkp-query-proofs-plan.md). Contributing: [`AGENTS.md`](../../AGENTS.md).

## License

[MIT](../../LICENSE).
