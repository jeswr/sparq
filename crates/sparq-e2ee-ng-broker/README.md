<!-- [OPUS-5] sq-tag1q.18: internal-stub README for a publish=false crate; full posture + public-API surface live in skills/e2ee-ng/SKILL.md and the design record. -->
# sparq-e2ee-ng-broker

Opt-in **opaque broker** for the sparq E2EE-NG profile: stores and routes
encrypted blocks and topics for clients built on `sparq-e2ee-ng`, speaking the
versioned client/broker messages of the design record §8.4. Ships the
`sparq-e2ee-ng-brokerd` daemon (length-prefixed deterministic CBOR over TCP).

> **Internal crate — not published** to crates.io (`publish = false`).
> **No security guarantee — research-grade and externally unaudited (`sq-qhy4`).**
> <!-- privacy-claims-allow: NEGATIVE/scoped — explicitly denies any proven soundness/privacy claim; sq-qhy4 pending -->
> Every confidentiality / integrity / authorization / revocation property of the
> profile it serves is **designed/intended, not proven**. Per design §5 a
> conforming broker still observes topic membership, subscription/publication
> patterns, timing, sizes, and storage volume — this **MUST NOT** be described as
> hiding access patterns, membership, volume, or timing. It is not trusted for
> integrity or availability. The daemon implements **no transport authentication
> and no TLS**.
> **Crate boundary (design §7):** it depends on `sparq-e2ee-ng` and nothing else
> in the workspace — it does **not** link `sparq-engine`/`sparq-core`, proven by
> `tests/boundary.rs`.

Design: [`research/e2ee-nextgraph-variant-gpt56-2026-07.md`](../../research/e2ee-nextgraph-variant-gpt56-2026-07.md).
How-to: [`skills/e2ee-ng/SKILL.md`](../../skills/e2ee-ng/SKILL.md).
Contributing: [`AGENTS.md`](../../AGENTS.md).

## License

[MIT](../../LICENSE).
