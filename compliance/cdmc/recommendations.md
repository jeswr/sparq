<!-- [OPUS-4.8] CDMC recommendations — epic sq-toze, branch cert-cdmc. -->
# CDMC recommendations — sparq

> 🤖 SPARQ agent. Prioritised, evidence-linked recommendations to raise sparq's CDMC maturity.
> Each maps a sub-4 capability from `scorecard.md` to a concrete, repo-grounded remediation and a
> tracking bead under epic `sq-toze` (see `gap-register.md` for the bead intents; the lead creates
> them in the main checkout — `bd` is not on PATH in the cert worktree).

Legend — **P0** unblocks a CDMC key-control theme; **P1** raises a capability one level; **P2**
nice-to-have / polish.

| ID | Capability (current→target) | Recommendation | Why it moves the needle | Sev | Bead (intent) |
|---|---|---|---|---|---|
| CD-R1 | 6.2 Lineage (2→3) | Capture **data lineage** as W3C-PROV-O triples: a `prov:Activity` per load/UPDATE event linking source (file/URL/graph) → the named graph → (optionally) a result digest, surfaced via the existing `/.well-known/void` + Service-Description descriptors and the `sparq-reason` `explain` path. | Lineage is a CDMC **key-control theme**; sparq already has the named-graph quad model + WAL/journal to anchor it. Highest credibility-per-effort. | P0 | CD-1 |
| CD-R2 | 3.2 Access audit (2→3) | Add an **opt-in structured access log**: per request emit `{identity (WAC WebID / token-scope), op (read/write), graphs_touched, decision, query_digest, ts}`. `sparq-solid` already computes the authorised graph set per session; emit it instead of discarding it. JSON-lines, off by default. | CDMC requires access be **tracked with audit trails**; sparq today has only aggregate Prometheus counters, no per-subject trail. | P0 | CD-2 |
| CD-R3 | 2.2 Classification (3→4) + 4.3 (2) | Publish a **classification/sensitivity deployment guide**: SHACL "sensitivity shapes" the operator applies to tag graphs/predicates, wired to the WAC/ACP entitlement layer, plus the at-rest/in-transit encryption pattern (operator KMS + TLS-terminating gateway). Explicitly hand off the operator-owned axis. | Turns two silent operator-owned 2s into a *documented boundary* an auditor can verify; raises classification because the hook becomes first-class. | P1 | CD-3 |
| CD-R4 | 3.1 Entitlements (3→4) | Land the **ODRL usage-control gate over `sparq-solid`** (single-node, `research/feature-research-odrl-policy.md` §scope-buildable-today): ODRL `Permission`/`Prohibition`/`Duty` over actions, evaluated above WAC/ACP. Keep the federated disclosure-control design explicitly *research-grade*. | Moves entitlements from access-control to genuine **usage control** (CDMC's "entitlements managed & enforced") — and it is the documented buildable slice, not the MPC-blocked one. | P1 | CD-4 |
| CD-R5 | 5.1 Lifecycle (3→4) | Add a **retention-policy mechanism**: bind the generation-ring `TimeTravelConfig`/`time-travel-max-age` to a declarative per-graph retention rule (TTL → automatic DROP/age-out), and document the operator's retention-policy responsibility. | CDMC wants **retention enforced**, not just possible. The ring + WAL already age generations out; expose it as policy. | P1 | CD-5 |
| CD-R6 | 1.2 Ownership (2→3) | Document and (optionally) validate a **dataset-ownership convention**: a VoID/DCAT header per dataset (`dcterms:publisher`, `dcat:contactPoint`, sensitivity) that `sparq-introspect`/descriptors surface, so "ownership is established" has a machine-readable home in the loaded graph. | Gives the operator's ownership decision a *recorded, queryable* place — sparq's role is to carry and expose it, which is exactly the right scoping. | P2 | CD-6 |
| CD-R7 | 1.1 / cross-cutting | Publish the **CDMC operator-responsibility split** as a short deployment doc (this scorecard's "operator-owned" column, condensed) so deploying teams know which CDMC controls they inherit vs. must implement. | Prevents the most common compliance error — assuming the engine satisfies governance controls it explicitly delegates. Pure documentation. | P2 | CD-7 |

## Cross-references

- The **Protection & Privacy (4.x)** recommendations are intentionally thin: sparq's *security*
  posture is already at 4 and is covered by the `asvs`/`cis`/`sbom`/`ssdf`/`slsa`/`openssf`/
  `memsafety`/`cra` framework slices — CDMC should **cite**, not duplicate, them. The *privacy*
  axis is bounded by the **ZK/MPC-not-sound** verdict and is owned by the `cryptoreview` +
  `privacy` slices; CDMC adds **no** crypto-protection recommendation that would imply ZK/MPC is a
  usable control.
- CD-R1/CD-R2 are the recommendations most likely to drive a `crates/` change; if they land before
  consolidation, **re-score 6.2 and 3.2** per the orchestration runbook.
