<!-- [OPUS-4.8] sq-inzv: internal-stub README for a publish=false crate; full surface lives in skills/usage-control-policy/SKILL.md. -->
# sparq-policy

**ODRL usage-control** over the [sparq](../../README.md) engine — the declarative
policy layer *above* access control. Where `sparq-solid` answers "may this agent
**read** graph G?", `sparq-policy` answers "may this party **use** this asset for
purpose P, with obligation O, until time T, disclosing only to recipient R?" It
parses a [W3C ODRL 2.2](https://www.w3.org/TR/odrl-model/) policy from RDF into a
typed model (Permission / Prohibition / Duty / Action / Constraint) and **evaluates**
a request to a **fail-closed** ALLOW/DENY `Decision`. The full public-API surface —
the fail-closed evaluator, every constraint dimension (purpose / recipient /
dateTime / spatial / count), the deny-overrides conflict semantics (with a fail-closed refusal of unimplementable `odrl:conflict` strategies, sq-ihqbl), the static
conflict/containment lints, the stateful `count-enforcement` counter stores, and the
opt-in `secprop-leftoperands` security-property ODRL profile (research-grade; sq-qhy4) —
lives in [`skills/usage-control-policy/SKILL.md`](../../skills/usage-control-policy/SKILL.md).

> **Internal crate — not published** to crates.io (`publish = false`); opt-in,
> dependency-of-nothing in the workspace. **Single-node only.** The headline
> federated-disclosure / ODRL→MPC composition (per-node ODRL driving the
> `sparq-mpc` disclosed-vs-hidden split; `Duty` → ZK proof obligation) is
> **deferred** — it would inherit the MPC honest-majority/LAN envelope and the
> open ZK-soundness remediation. <!-- privacy-claims-allow: NEGATIVE — names the deferred ZK/MPC composition + its open soundness remediation; sq-qhy4 -->

**Conformance:** ratcheted against the MIT-licensed [SolidLab ODRL Test Suite](https://github.com/SolidLabResearch/ODRL-Test-Suite) through the real `evaluate` path (`tests/odrl_test_suite.rs`, sq-tmsd6; **67/68 pass** after the constraint-matching batch added `odrl:LogicalConstraint` compound constraints, party/asset collection membership, and the `odrl:use` action hierarchy — sq-euhr3/sq-k7itg/sq-a0zef; 1 documented not-implemented divergence — the SKILL has the oracle).

How-to: [`skills/usage-control-policy/SKILL.md`](../../skills/usage-control-policy/SKILL.md) · Design: [`research/feature-research-odrl-policy.md`](../../research/feature-research-odrl-policy.md) · Sibling: [`sparq-solid`](../sparq-solid/README.md) · Contributing: [`AGENTS.md`](../../AGENTS.md).

## License

[MIT](../../LICENSE).
