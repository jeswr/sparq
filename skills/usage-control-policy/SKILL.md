---
name: usage-control-policy
description: Evaluate W3C ODRL 2.2 usage-control policies over RDF with the opt-in sparq-policy crate — parse an ODRL Set/Offer/Agreement into a typed Policy of Permission/Prohibition rules (action, target, assignee, Constraint, Duty), then evaluate an access Request to a fail-closed ALLOW/DENY Decision. Use when gating a query/asset by purpose, recipient, time window, count, or a duty obligation; when mapping ODRL to the sparq-solid WAC/ACP allow-deny model; or when wiring usage control above access control. Single-node base case; federated ODRL-to-MPC disclosure is deferred.
---

# usage-control-policy

`sparq-policy` is the declarative **usage-control** layer above access control. Where `sparq-solid` answers "may this agent **read** graph G?", `sparq-policy` answers "may this party **use** this asset *for purpose P, with obligation O, until time T, disclosing only to recipient R*?" — by evaluating a [W3C ODRL 2.2](https://www.w3.org/TR/odrl-model/) policy.

It parses an ODRL policy from RDF into a typed model (`Policy → {permissions, prohibitions}`, each `Rule` carrying an `Action`, `target`, `assignee`/`assigner`, `Constraint`s and `Duty`s) and evaluates an access `Request` to a **fail-closed** `Decision { allow, matched_rules, unmet_constraints }`. This is the **single-node base case**: ODRL over one node's data, reducing to the same allow/deny shape `sparq-solid` enforces.

> **Scope.** Single-node only. The headline federated-disclosure / ODRL→MPC composition (per-node ODRL drives the `sparq-mpc` disclosed-vs-hidden split; ODRL `Duty` → ZK proof obligation) is **deferred** — it inherits the MPC honest-majority/LAN envelope and the open ZK-soundness remediation. See `research/feature-research-odrl-policy.md`.

## Prerequisites

- **Cargo dep** — `sparq-policy` is `publish = false`, a non-default workspace member (nothing in core depends on it; `cargo tree -p sparq-core` never shows it). Add it as a path dep:
  ```toml
  sparq-policy = { path = "crates/sparq-policy" }
  ```
- No external toolchain. It depends only on `sparq-core` + `sparq-engine` + `oxrdf`; zero `unsafe`.

## Quickstart

Parse an ODRL `Set` and evaluate a time-windowed permission. Compiles and runs against the current API.

```rust
use sparq_policy::{evaluate, parse_policy_str, Request, Value};

// alice MAY read asset-X, on or before 2026-12-31, for research purpose.
let ttl = r#"
@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
<urn:pol/1> a odrl:Set ; odrl:permission [
    odrl:action odrl:read ;
    odrl:target <urn:asset/x> ;
    odrl:assignee <https://alice.ex/me> ;
    odrl:constraint [ odrl:leftOperand odrl:dateTime ; odrl:operator odrl:lteq ;
                      odrl:rightOperand "2026-12-31T00:00:00Z"^^xsd:dateTime ] ;
    odrl:constraint [ odrl:leftOperand odrl:purpose ; odrl:operator odrl:eq ;
                      odrl:rightOperand <urn:purpose/research> ] ] .
"#;
let policy = parse_policy_str(ttl, "turtle").expect("parse");

const ODRL: &str = "http://www.w3.org/ns/odrl/2/";
let req = Request::new(format!("{ODRL}read"))
    .on("urn:asset/x")
    .by("https://alice.ex/me")
    .with(format!("{ODRL}dateTime"), Value::DateTime("2026-06-16T09:00:00Z".into()))
    .with(format!("{ODRL}purpose"),  Value::Iri("urn:purpose/research".into()));

let d = evaluate(&policy, &req);
assert!(d.allow);                      // in-window, right party, right purpose
assert_eq!(d.matched_rules.len(), 1);  // the granting permission, for audit
```

## The model

- **`Policy`** — `permissions: Vec<Rule>`, `prohibitions: Vec<Rule>`, optional `iri`. `Set`/`Offer`/`Agreement` parse identically (subtype affects contracting, not single-node eval).
- **`Rule`** — `action: Action`, `target`/`assignee`/`assigner: Option<String>`, `constraints: Vec<Constraint>`, `duties: Vec<Duty>` (duties on permissions only).
- **`Action`** — full IRI. `odrl:use` is the **umbrella** action: a permission for `use` permits *any* requested action. All others match by exact IRI.
- **`Constraint`** — `(left: leftOperand-IRI, operator, right: Value)`. The request supplies the *actual* value for `left` in its `context`; the constraint's `right` is the bound.
- **`Duty`** — an obligation (`action`) that must be in the request's `discharged_duties` set, or the permission is denied.

## Evaluation semantics (fail-closed)

1. A `Rule` **matches** when its action permits the request action, its `target`/`assignee` (if set) agree, and **every** `Constraint` is satisfied (logical AND).
2. A `Permission` grants iff it matches **and** all its `Duty`s are discharged.
3. A matching `Prohibition` **overrides** any permission (carve-out — ODRL Formal Semantics conflict default).
4. **DENY by default:** no matching+discharged permission, or any matching prohibition ⇒ DENY. An empty/malformed policy denies everything; a constraint with no request value, an unknown operator, or a structurally incomplete constraint all fail closed.

## Constraint operators

`eq`, `neq`, `lt`, `lteq`, `gt`, `gteq`, `isPartOf` (set membership: `right` is a `|`/space/comma-separated set), `isA` (identity, ≈ `eq`). Numeric (`xsd:integer`/`decimal`/`double`/…) and `xsd:dateTime`/`date` operands compare by magnitude/instant; everything else by IRI/string value. Order comparison on non-orderable values is `false` (fail-closed). `dateTime` ordering compares the lexical form — mixed-offset normalization is a deferred bead.

## Building the request

`Request::new(action_iri)` then chain `.on(target)`, `.by(party)`, `.with(left_operand_iri, Value)` for each context dimension (`dateTime`, `purpose`, `recipient`, `count`, `spatial`, …), and `.discharge(duty_action_iri)` per discharged duty. `Value` is `Iri` | `Str` | `Num(f64)` | `DateTime(String)`.

## Learn more

- Crate README: [`crates/sparq-policy/README.md`](../../crates/sparq-policy/README.md)
- Design record: `research/feature-research-odrl-policy.md` (epic sq-3183)
- Sibling access-control skill: [`skills/http-server`](../http-server/SKILL.md) (Solid WAC/ACP via `sparq-solid`)
- W3C [ODRL Information Model 2.2](https://www.w3.org/TR/odrl-model/) · [Formal Semantics](https://w3c.github.io/odrl/formal-semantics/)
