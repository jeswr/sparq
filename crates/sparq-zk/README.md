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
> for this pipeline today. Config-only **commitment-method registry** (`commit::CommitmentMethod`:
> CLOSED fail-closed selection over `zk:scheme` — `string-canonical` default · `dual-leaf` ·
> the OFF-by-default `commitment-value-only` research dial; sq-zzxt) records the method but adds no
> circuit. The opt-in `dual-leaf` host value encoders (`dual_leaf::{encode_literal, encode_double, encode_decimal}`
> — integer/double/decimal classes, fail-closed; `encode_double` accepts only canonical scientific notation or `INF`/`-INF`/`NaN`; mirror the `filter_value_dl_*` members; sq-xojl + sq-2ezsx, full pipeline `sq-j506`) carry the #769-accepted <!-- [GPT-5.6] sq-vh829 -->
> **INV-VL downgrade** (value↔lexical agreement is trusted-issuer-honesty — open audit obligation
> **CR-G8** / `sq-qhy4`). The OFF-circuit issuer-signature seam is the OPEN `sig::IssuerSignatureScheme` trait (sq-1hsl, `SchnorrBjjScheme` the byte-unchanged default impl) — additive, asserting no soundness claim. The opt-in `vc-bridge` feature (`vc_bridge`, OFF by default; sq-9c5e) verifies a source VC's W3C Data-Integrity proof (`eddsa-rdfc-2022`/`ecdsa-rdfc-2019`) OFF-circuit at ingest, re-commits, and records `zk:sourceCryptosuite` provenance (NOT a re-verifiable in-proof property; no in-circuit VC verification); the same feature carries the `bbs-2023`/`ecdsa-sd-2023` selective-disclosure ingest seam (sq-u5y1f, `ingest_sd_credential_unverified`), which re-commits a disclosed subset + records provenance but verifies NO selective-disclosure proof (`IngestedSdCredential::is_sd_verified()` is always `false`).
> <!-- [OPUS-4.8] privacy-claims-allow: opt-in audit-gated encoding + config/seam plumbing, registered as an OPEN obligation; asserts no soundness/privacy property; sq-qhy4 / CR-G8 -->

Design: [`research/zkp-query-proofs-plan.md`](../../research/zkp-query-proofs-plan.md). Contributing: [`AGENTS.md`](../../AGENTS.md).

## License

[MIT](../../LICENSE).
