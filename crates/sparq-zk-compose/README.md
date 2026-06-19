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
> ([`research/zk-soundness-audit.md`](../../research/zk-soundness-audit.md)). The
> cryptography is **not** FIPS-approved
> ([`compliance/cryptoreview/fips-posture.md`](../../compliance/cryptoreview/fips-posture.md)).
> Authored by Opus 4.8 (Fable 5 unavailable) — flag for re-review when Fable returns.

How-to + the covered/deferred matrix: [`skills/zk-query-proofs/SKILL.md`](../../skills/zk-query-proofs/SKILL.md).
Benchmarks (gate counts, timing): [`bench/zk-compose/`](../../bench/zk-compose).
Contributing: [`AGENTS.md`](../../AGENTS.md).

## Testing

Two layers:

- **Toolchain-free glue unit tests** (fast, run by default `cargo test`): the
  `#[cfg(test)]` modules in `build.rs` / `driver.rs` / `toml.rs` exercise the
  NON-cryptographic composition plumbing — circuit-family id derivation
  (`derive_*_id`: smallest-fitting bucket, EXACT digit-count discipline,
  out-of-family `None` never a wrong member, package-name determinism), the
  nargo/bb subprocess wrapper's typed error classification (`Spawn` vs `Tool`
  vs `Io`) and tag-based witness-path isolation, and `Prover.toml` witness
  serialization (`FieldHex` hex round-trip, scalar-vs-array shape, declaration
  order, the recoverable missing-witness error arms). These assert nothing about
  in-circuit soundness or any privacy property — the verifier is **not-yet-sound**
  (see the caveat above, sq-qhy4).
- **Toolchain e2e tests** (`tests/`): the real prove/verify path via nargo/bb,
  which skip cleanly when the toolchain is absent from `PATH`.

## License

[MIT](../../LICENSE).
