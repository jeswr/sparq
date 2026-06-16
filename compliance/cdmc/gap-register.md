<!-- [OPUS-4.8] CDMC gap register — epic sq-toze, branch cert-cdmc. -->
# CDMC gap register — sparq

> 🤖 SPARQ agent. Open CDMC capability gaps with severity, remediation, target maturity, and the
> tracking **bead intent** under epic `sq-toze`. **Bead-creation note:** the `bd` CLI (Dolt-backed)
> is **not on PATH in the cert worktree**, so these beads could not be created in-session. The lead
> must run `cd <main checkout> && bd create … --epic sq-toze` for each, then re-export. Each row
> below is a ready-to-file bead spec. (Matches the orchestration contract: subagents capture
> discovered work as beads via the main checkout; orchestrator re-exports.)

Severity — **P0** unblocks a CDMC key-control theme · **P1** raises a capability one level · **P2**
polish. None of these are *security* gaps (sparq's security posture is 4 and covered by the other
framework slices); they are **data-management-maturity** gaps specific to CDMC.

| ID | Gap | Cap (cur→tgt) | Sev | Remediation (see `recommendations.md`) | Bead intent (epic sq-toze) |
|---|---|---|---|---|---|
| CD-1 | **No first-class data lineage.** Provenance is partial (named-graph + WAL + reason `explain`); no W3C-PROV ingest-event → graph → result capture. | 6.2 (2→3) | P0 | W3C-PROV-O `prov:Activity` per load/UPDATE, surfaced via VoID/SD descriptors + `explain`. | `CDMC: W3C-PROV data-lineage capture (ingest→graph→result)` |
| CD-2 | **ADDRESSED (`sq-0bxp`, branch `feat-access-audit-log`).** ~~No per-query access audit trail.~~ An **opt-in per-query access audit log** now ships on `sparq-server` (cargo feature `audit-log` + runtime `--audit-log` / `SPARQ_AUDIT_LOG=1`): one structured `tracing` event per query/update/Graph-Store request under target `sparq_server::audit` — requester identity (a Bearer-token fingerprint or `anonymous`, never the secret), operation class, a non-reversible query fingerprint (not the full query text — the #241 info-leak posture), the access decision (allowed/denied + reason), the HTTP status / row count, and duration. Off ⇒ zero overhead. See `crates/sparq-server/src/audit.rs` + `skills/http-server/SKILL.md` → "Access audit log". **Re-score 3.2 (2→3).** | 3.2 (2→**3**) | P0 | ~~Opt-in structured access log~~ — **landed.** Operator routes the `sparq_server::audit` target to their sink (`RUST_LOG`); WAC-session-identity + authorised-graph-set enrichment (once `sparq-solid` fronts the server) is a follow-up. | `CDMC: opt-in structured access audit log (identity + graphs touched)` — **done (sq-0bxp)** |
| CD-3 | **Classification taxonomy + encryption-at-rest are silently operator-owned.** No documented handoff; mmap files + HTTP are plaintext. | 2.2/4.3 (2/3) | P1 | Deployment guide: SHACL sensitivity shapes → WAC/ACP wiring + operator KMS/TLS-gateway pattern; explicit boundary doc. | `CDMC: classification + encryption-at-rest operator deployment guide` |
| CD-4 | **No usage control (ODRL).** Entitlements are access-control only; no purpose/recipient/duty enforcement. | 3.1 (3→4) | P1 | Land the single-node ODRL gate over `sparq-solid` (the buildable-today slice from the ODRL research record). | `CDMC: ODRL single-node usage-control gate over sparq-solid` |
| CD-5 | **Retention is possible but not policy-driven.** Generation-ring ages out by age/count, not by declarative per-graph retention rule. | 5.1 (3→4) | P1 | Bind `TimeTravelConfig`/`time-travel-max-age` to declarative per-graph TTL → automatic DROP/age-out; document operator retention policy. | `CDMC: declarative per-graph retention policy mechanism` |
| CD-6 | **No machine-readable dataset-ownership convention.** Ownership decision has no recorded home in the loaded graph. | 1.2 (2→3) | P2 | VoID/DCAT dataset header convention (`dcterms:publisher`, `dcat:contactPoint`, sensitivity) surfaced by introspect/descriptors. | `CDMC: VoID/DCAT dataset-ownership header convention + surfacing` |
| CD-7 | **No published CDMC operator-responsibility split.** Deploying teams may assume the engine satisfies governance controls it delegates. | 1.1 / cross-cutting | P2 | Short deployment doc condensing this scorecard's operator-owned column. | `CDMC: publish operator-responsibility split deployment doc` |
| CD-8 | **Catalogue capability is not CI-gated.** The VoID + Service-Description catalogue (`crates/sparq-server/src/descriptors.rs`, #219) is real with 6 `#[test]`s but lives behind the **default-OFF** `federation-descriptors` cargo feature; no CI lane compiles or runs it (`.github/workflows/ci.yml` uses default features only), so a stock build returns `404` for `/.well-known/void` and a regression in the module would pass all gates silently. This holds 2.1 at maturity 3. | 2.1 (3→4) | P1 | Add a CI lane that runs `cargo test -p sparq-server --features federation-descriptors` (and include the feature in a clippy/check `--features` matrix), so the 6 descriptor tests gate every PR. | bead **sq-kzfi** (filed by the auditor, epic `sq-toze`) + the broader feature-matrix bead |

## Explicitly NOT gaps (auditor anchors — do not re-open)

- **ZK/MPC not contributing to 4.2/4.3.** This is **correct by the honesty contract**, not a gap to
  close by crediting the scaffold. `research/zk-soundness-audit.md` is binding. The crypto remediation
  itself is tracked by the existing ZK remediation beads, **not** by CDMC.
- **No per-user authentication in `sparq-server`.** Documented architectural decision (threat-model
  **B3**: "front with a gateway / sparq-solid"). The optional bearer-token write gate (sq-zcby) +
  `sparq-solid` WAC/ACP are the in-scope entitlement controls; full user auth is operator-owned.
- **Plaintext mmap / plaintext HTTP.** Operator-deployment (TLS gateway + disk/KMS encryption),
  recorded as operator-owned in `controls.md` 4.3, with the handoff doc tracked as CD-3 — not a
  sparq-source defect.

## Re-score triggers

Per the orchestration runbook, CDMC scores the **current** codebase state. If **CD-1** (lineage) or
**CD-2** (access audit) — the two most likely to drive a `crates/` change — land before
consolidation, **re-score capabilities 6.2 and 3.2** and update `scorecard.md` accordingly.
Likewise, if **CD-8** (bead `sq-kzfi`) lands a CI lane that compiles and runs the
`federation-descriptors` catalogue tests, **re-score capability 2.1 from 3 → 4** (and Component 2's
average back to ~3.5).
