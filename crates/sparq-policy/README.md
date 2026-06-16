# sparq-policy

<p>
  <a href="../../LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
</p>

**ODRL usage-control** over the [sparq](../../README.md) engine — the declarative
policy layer *above* access control.

`sparq-solid` answers "may this agent **read** graph G?"; `sparq-policy` answers
"may this party **use** this asset *for purpose P, with obligation O, until time
T, disclosing only to recipient R*?" It parses a [W3C ODRL
2.2](https://www.w3.org/TR/odrl-model/) policy from RDF into a typed model
(Permission / Prohibition / Duty / Action / Constraint) and **evaluates** an
access request to a **fail-closed** ALLOW/DENY [`Decision`]. This is the
**single-node base case**: ODRL over one node's data, reducing to the same
allow/deny shape `sparq-solid` enforces. It is a dependency of nothing in the
workspace (opt-in; `publish = false`).

## 🚀 Quickstart

```rust
# fn main() -> Result<(), String> {
use sparq_policy::{evaluate, parse_policy_str, Request, Value};

// An ODRL Set: alice MAY read asset-X, but only on or before 2026-12-31.
let ttl = r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
<urn:pol/1> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ;
    odrl:target <urn:asset/x> ;
    odrl:assignee <https://alice.ex/me> ;
    odrl:constraint [ odrl:leftOperand odrl:dateTime ;
                      odrl:operator odrl:lteq ;
                      odrl:rightOperand "2026-12-31T00:00:00Z"^^<http://www.w3.org/2001/XMLSchema#dateTime> ] ] .
"#;
let policy = parse_policy_str(ttl, "turtle")?;

// In-window request by alice → ALLOW.
let inside = Request::new("http://www.w3.org/ns/odrl/2/read")
    .on("urn:asset/x").by("https://alice.ex/me")
    .with("http://www.w3.org/ns/odrl/2/dateTime",
          Value::DateTime("2026-06-16T09:00:00Z".into()));
assert!(evaluate(&policy, &inside).allow);

// Out-of-window request → DENY (fail-closed; the constraint is unsatisfied).
let outside = inside.clone()
    .with("http://www.w3.org/ns/odrl/2/dateTime",
          Value::DateTime("2027-01-01T00:00:00Z".into()));
assert!(!evaluate(&policy, &outside).allow);
# Ok(()) }
```

## ✨ Features

- **ODRL model from RDF** — `parse_policy_str` / `parse_policy` extract a typed
  `Policy → {permissions, prohibitions}`, each `Rule` carrying an `Action`,
  `target`, `assignee`/`assigner`, `Constraint`s and `Duty`s. Rule, constraint
  and duty nodes are matched as **blank nodes or IRIs** (real ODRL uses blank
  nodes). Built on the engine sparq already ships — ODRL eval *is* a SPARQL
  workload.
- **Fail-closed evaluator** — `evaluate(&policy, &request) -> Decision { allow,
  matched_rules, unmet_constraints }`. No matching permission **or** a matching
  prohibition ⇒ DENY; a permission grants only when matched **and** all its
  duties are discharged. Prohibitions carve out permissions (ODRL conflict
  default). A malformed/unknown constraint becomes an unsatisfiable guard.
  `matched_prohibition(&policy, &request) -> Option<&Rule>` exposes the
  evaluator's own conflict test (the first prohibition that carves the request
  out), so a downstream materializer can distinguish a *carve-out* deny from a
  plain no-permission deny — `sparq-solid`'s ODRL bridge uses it to emit explicit
  `auth:deny*` triples (deny-overrides). [OPUS-4.8] sq-w693.
- **Constraint operators** — `eq`, `neq`, `lt`, `lteq`, `gt`, `gteq`,
  `isPartOf` (set membership), `isA`. Numeric (`xsd:integer`/`decimal`/…) and
  `xsd:dateTime`/`date` operands compare by magnitude/instant; everything else
  by IRI/string value. Constraints over `purpose` / `recipient` / `dateTime` /
  `count` / `spatial` left-operands are all supported.
- **Duties as obligations** — a permission's `odrl:duty` must appear in the
  request's discharged-duty set, or the permission is denied (the usage-control
  kernel pure access control lacks).
- **Opt-in, lean** — no `unsafe`; depends only on `sparq-core` + `sparq-engine`
  + `oxrdf`; pulled into no core build (`cargo tree -p sparq-core` never shows it).

### Scope & caveats

Single-node only. The headline **federated-disclosure** / ODRL→MPC composition
(per-node ODRL drives the `sparq-mpc` disclosed-vs-hidden split; ODRL `Duty` →
ZK proof obligation) is **deferred** — it inherits the MPC honest-majority/LAN
envelope and the open ZK-soundness remediation. `dateTime` ordering compares the
lexical form; mixed-offset normalization, DPV `purpose` hierarchies, and
`Duty → proof-manifest` discharge are tracked as follow-on beads. See
`research/feature-research-odrl-policy.md`.

**Constraint persistence vs. one-shot** (sq-hiz4, in `sparq-solid`'s opt-in bridge):
`materialize_odrl_permission_conditional` persists a `odrl:recipient`/`odrl:assignee`
constraint (`eq`/`isA`/`isPartOf`) as a **re-checked** ACP `auth:ConditionalGrant`
(agent matcher) — the only constraint with a faithful stateless `(agent, client)`
analogue. `odrl:purpose`/`dateTime`/`count` have none and stay **one-shot** (checked
once at materialization). Mapping table: the
[`usage-control-policy`](../../skills/usage-control-policy/SKILL.md) skill +
[`sparq-solid` README](../sparq-solid/README.md).

## 📚 Learn more

- Skill: [`skills/usage-control-policy/SKILL.md`](../../skills/usage-control-policy/SKILL.md)
- Design record: `research/feature-research-odrl-policy.md` (epic sq-3183)
- W3C [ODRL Information Model 2.2](https://www.w3.org/TR/odrl-model/) ·
  [Formal Semantics](https://w3c.github.io/odrl/formal-semantics/)
- Sibling access-control crate: [`sparq-solid`](../sparq-solid/README.md)

## License

MIT — see [LICENSE](../../LICENSE).
