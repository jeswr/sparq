<!-- [OPUS-4.8] sq-2q1x: internal-stub README for a publish=false crate; full design lives in research/mpc-untrusted-planner-routing-design.md. -->
# sparq-fedplan-mpc

**Phase-1 skeleton** of the opt-in seam between cost-based federated source
selection (`sparq-fedplan`) and MPC-over-federated-SPARQL routing (`sparq-mpc`) —
without coupling the two upstreams to each other. Behind the **`fedplan-mpc` cargo
feature, OFF by default**; the default build compiles an empty crate and pulls in
neither upstream. Design: [`research/mpc-untrusted-planner-routing-design.md`](../../research/mpc-untrusted-planner-routing-design.md).

What lands here: the **`SourcePrivacyDescriptor`** (posture is **default-deny** — a
predicate is disclosable only when the source explicitly marks it `Public`; a
planner cannot widen it, the source re-enforces fail-closed) plus typed, panic-free
stubs (`select_private_sources` / `route_operators` / `assemble_leakage_envelope`)
that return an honest `SeamError::Deferred { phase, gated_on }`.

> **Internal crate — not published** (`publish = false`). **No soundness or privacy
> claim.** It performs **no** MPC and runs **no** privacy-bearing logic. The MPC
> estate (`sparq-mpc`) is **research-grade, honest-majority semi-honest only, and
> NOT externally audited** — the cryptographer sign-off (`sq-qhy4`) and coZK
> re-audit (`sq-9hrn`) are pending. <!-- privacy-claims-allow: NEGATIVE/scoped — denies any privacy/soundness property; audits sq-qhy4 / sq-9hrn pending -->
> The privacy-bearing phases (source-selection pruning, disclosed/hidden routing,
> leakage-envelope assembly, authenticated-input attestation) are **deferred and
> audit-gated**; nothing here is a working protocol.

Design: [`research/mpc-untrusted-planner-routing-design.md`](../../research/mpc-untrusted-planner-routing-design.md).
Contributing: [`AGENTS.md`](../../AGENTS.md).

## License

[MIT](../../LICENSE).
