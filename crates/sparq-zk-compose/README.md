<!-- [OPUS-4.8] sq-puyy: trimmed to the concise internal-stub README template (sq-9jw5). -->
# sparq-zk-compose

ZK proof **composition** for [sparq](../../README.md) — stage 2 of the query-proof
design ([`research/zkp-query-proofs-plan.md`](../../research/zkp-query-proofs-plan.md)
v3, §S4.E). Drives the per-property Noir circuit family at
[`zk/compose/`](../../zk/compose) into a full query-result proof
(`manifest::ProofManifest` + the nargo/bb subprocess prover) and verifies one
(`verifier::verify_manifest`).

> **Internal crate — not published** to crates.io (`publish = false`): nothing in
> the workspace depends on it, so default and wasm builds are byte-identical either way.

<!-- separate, distinct blockquotes: the internal-crate note above vs. the soundness caveat below (MD028) -->

> **NOT-yet-sound** (standing caveat — sq-qhy4 / sq-9hrn; remediation epic sq-1s2).
> No soundness, zero-knowledge, or privacy property is claimed as achieved; the
> verifier's soundness is the subject of the open external audit
> ([external](../../research/zk-soundness-audit.md); internal re-audits:
> [binding](../../research/zk-verifier-reaudit.md) +
> [membership/PoK](../../research/zk-membership-pok-reaudit.md)). Not FIPS-approved
> ([fips-posture](../../compliance/cryptoreview/fips-posture.md)). The opt-in `dual-leaf` value lane (`filter_value_dl_{int,f64,decimal}` datatype-class members + the `dispatch` fail-closed `(method × circuit)` matrix; sq-xojl/sq-cfmv/sq-2ezsx) carries the #769-accepted **INV-VL downgrade**, an open audit obligation (**CR-G8** / `sq-qhy4`) — no soundness/privacy claim (API in the SKILL). The OFF-by-default `extended-fragment` wave-1 lane (`CircuitId::PathReach` bounded-depth property-path member + the fail-closed `verifier::dispatch_fragment` routing gate over a `FragmentManifest`; sq-3kd2g.6) is a STRUCTURAL routing gate only and asserts no soundness/privacy property. Opus 4.8 — re-review when Fable returns. <!-- [OPUS-4.8] privacy-claims-allow: opt-in dual-leaf value lane + fail-closed dispatch matrix + OFF-by-default extended-fragment routing gate; INV-VL downgrade framed as an OPEN audit obligation; every lane explicitly asserts no soundness/privacy property; sq-qhy4 / CR-G8 -->

How-to + the covered/deferred matrix: [`skills/zk-query-proofs/SKILL.md`](../../skills/zk-query-proofs/SKILL.md).
Benchmarks (gate counts, timing): [`bench/zk-compose/`](../../bench/zk-compose).
Contributing: [`AGENTS.md`](../../AGENTS.md).

## License

[MIT](../../LICENSE).
