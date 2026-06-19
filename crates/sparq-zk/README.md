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

> **Internal crate — not published** to crates.io (`publish = false`). Circuits
> and proof composition are later deliverables; **no soundness or privacy claim**
> is made for this pipeline today (see the ZK verifier soundness status in the
> repo's security records). A field-native value-hook term/literal encoding change
> is **PLANNED** (research-grade, NOT implemented, audit-gated) — it is registered
> as an open external-audit obligation, see gap-register **CR-G8** / `sq-qhy4`.
> <!-- [OPUS-4.8] privacy-claims-allow: forward pointer to an unimplemented, audit-gated encoding change registered as an OPEN obligation; asserts no soundness/privacy property; sq-qhy4 / CR-G8 -->

Design: [`research/zkp-query-proofs-plan.md`](../../research/zkp-query-proofs-plan.md).
Contributing: [`AGENTS.md`](../../AGENTS.md).

## License

[MIT](../../LICENSE).
