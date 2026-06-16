<!-- [OPUS-4.8] sq-toze — Privacy gap register. Honest engine-side gaps + remediation beads.
     Re-review when Fable returns. -->

# Privacy — gap register

Open **engine-side** gaps for the privacy framework — i.e. technical capabilities the engine
*could* provide to make an operator's GDPR/27701/SOC 2 job easier, but does not yet. Each has a
severity, a remediation, and the `bd` bead that tracks it (created under epic **sq-toze** via the
main checkout — never hand-edited into `.beads/`).

> **Scope discipline.** Most privacy "gaps" are **NOT engine gaps** — they are operator
> deployment responsibilities (lawful basis, RoPA, DPA, consent, breach notification, at-rest
> encryption, TLS). Those are *not* listed here as gaps; they are labelled **OPERATOR** in
> [`controls.md`](./controls.md) and detailed in [`../data-flow.md`](../data-flow.md). Listing
> them here would be papering an operator duty as a sparq defect. Only genuine engine-side
> capability gaps appear below.

## OPEN gaps (engine-side)

| ID | Gap | Sev | Remediation | Bead |
|---|---|---|---|---|
| **PR-G1** | **UPDATE parse/semantic errors echo a query-text fragment.** `update_rejection_response` returns `"update failed: {e}"` where `{e}` is the parser diagnostic — which can include a fragment of the (malformed) SPARQL UPDATE text in the 400 body. This is *query text*, not loaded RDF data, but it is a residual leak of caller input in an error body (P-12). Query responses already use generic bodies; UPDATE parse errors are the one exception. | **Low** | Either (a) strip the diagnostic to a generic `"update failed: malformed SPARQL update"` (parity with the query path), or (b) gate the verbose diagnostic behind `--verbose` so it is opt-in like request logging. Add a regression test asserting the default-mode body contains no echoed query fragment. | **sq-toze.32** |
| **PR-G2** | **No built-in structured, queryable access/audit log.** GDPR Art. 5(2) accountability + SOC 2 monitoring want a demonstrable access trail (who queried/updated what, when). Today the engine emits **no** audit log by default (good for minimisation), and `--verbose` produces *unstructured* tower-http trace lines, not a structured `(actor, operation, graphs-touched, verdict, timestamp)` record. `sparq-solid` computes allow/deny verdicts but does not persist them. | **Low** | Optional, off-by-default structured audit sink (JSON lines) capturing `(timestamp, actor/WebID, operation, target graphs, allow/deny, result-row-count)` — *counts, not content*, to stay minimisation-clean. Operator opt-in via a flag. Pairs with the data-minimisation posture doc. | **sq-toze.32** |
| **PR-G3** | **`--persist` WAL is not erasure-complete.** A triple removed by `DELETE`/`DROP` may persist in earlier append-only WAL segments until the operator compacts/rotates the persist directory, so a naive `DELETE` is **not** a complete Art. 17 erasure of the persisted store (P-9). The engine provides no WAL-purge/compaction command, and this caveat is not surfaced to the operator at the point of deletion. | **Medium** | (a) Document the WAL-erasure caveat prominently in the persist README + a retention/erasure operator runbook (the *mechanism* exists — `DROP`/`DELETE` + manual rotation — but the operator must know to rotate). (b) Optionally add a `compact`/`vacuum` admin command that rewrites the WAL dropping superseded segments. | **sq-toze.33** |
| **PR-G4** | **No request-log redaction control.** When an operator enables `--verbose`, `TraceLayer` logs full request lines including SPARQL query text — which can embed personal data (e.g. `FILTER(?email = "alice@…")`). There is no option to redact/hash query bodies in the log. | **Low** | Add a `--log-redact-queries` option (or make `--verbose` log method+path+status only, with query text behind a separate explicit flag), so an operator who needs request logging can keep it PII-clean. | **sq-toze.34** |

## GATED-on-external (honesty contract — not engine gaps to "fix" here)

| ID | Item | Why it is not closeable in this framework | Bead |
|---|---|---|---|
| **PR-X1** | **ZK/MPC "privacy by cryptography" is NOT yet sound.** The estate models private query proofs / MPC but provides **no** guarantee (`SECURITY.md`; `research/zk-soundness-audit.md`). Any privacy claim resting on it would be a high-severity overclaim. | The soundness fix is **cryptographic work** tracked by the ZK remediation beads (epic sq-1s2) + the `cryptoreview` framework — **not** this privacy framework. This register only records that privacy claims are **gated** on it. We do **not** mark it "closed" or claim any privacy benefit from it. | **sq-toze.35** (gate) + epic **sq-1s2** (the crypto fixes) |
| **PR-X2** | **27701 / SOC 2 Privacy certificate** needs the operator's PIMS + an accredited external auditor. | The engine provides the technical levers (controls.md P-17/P-18); the *attestation* is a deployment property an agent cannot issue. Labelled **AUDIT-READY**, not a gap. | n/a (external) |

## NOT gaps (operator responsibilities, recorded so the auditor does not re-flag)

These are **OPERATOR** rows in `controls.md`, NOT engine defects — listed here only to pre-empt
re-flagging:

- **At-rest encryption** of the `--persist` WAL (P-9) — operator's full-disk/volume encryption.
- **TLS / transport encryption** — operator terminates TLS (gateway / reverse proxy).
- **Lawful basis, purpose limitation, consent, privacy notice, RoPA, DPIA sign-off, DPA chain,
  cross-border transfer, breach notification, DPO** — all controller/operator duties (P-1, P-16,
  P-18).
- **Per-user authentication in the core server** — documented boundary **B3** ("front with a
  gateway / `sparq-solid`"), an explicit architectural decision, not a silent gap. The engine
  *does* provide optional Bearer auth (P-10) + `sparq-solid` graph-level authz.

## Honest overall posture

The privacy framework is **substantively AUDIT-READY** with a **respectable engine-side
capability set**: data-minimisation by default (P-2), full SPARQL erasure/rectification/export
(P-3/4/5/6), fail-closed graph-level access control (P-10/11), generic error bodies (P-12),
default-deny SSRF guard (P-13), and in-browser client-side processing (P-15). The open gaps are
**four low/medium engine *conveniences*** (PR-G1..G4) that make an operator's privacy job easier
— none is a security hole, and none is the difference between "the engine can support a
GDPR-compliant deployment" and "it can't" (it can). The two non-engine items (PR-X1 ZK/MPC
not-sound, PR-X2 external certificate) are **gated by design** and explicitly **not** claimed as
privacy guarantees. There is **no overclaim**: the framework never asserts "sparq is GDPR
compliant" (a deployment property) — only the technical capabilities, each cited.
