<!-- [OPUS-4.8] sq-toze — SHARED DPIA skeleton (owned by the Privacy framework engineer).
     A TEMPLATE: the engine-risk half is filled with evidence; the deployment half is the
     operator's to complete. Referenced by privacy, cra, iso27001. Re-review when Fable returns. -->

# Data Protection Impact Assessment (DPIA) — skeleton

**This is a TEMPLATE, not a completed DPIA.** A GDPR Art. 35 DPIA is a property of a *processing
operation* — a controller, a purpose, a dataset, a lawful basis. **sparq is not a controller and
defines none of those**, so a *completed* DPIA cannot exist in this source tree. What this
document provides is:

1. The **engine-side half** — the data-protection risks intrinsic to the sparq binary, assessed
   with repo evidence + the mitigations the engine already ships (the part an agent *can* honestly
   complete); and
2. A **skeleton for the operator's half** — the deployment-specific sections (controller,
   purpose, lawful basis, necessity/proportionality, consultation) the **operator must complete**
   for their specific processing operation.

> Companion docs: [`data-flow.md`](./data-flow.md) (where data flows + the responsibility split),
> [`privacy/controls.md`](./privacy/controls.md) (the control table), and the STRIDE security
> model at [`../research/threat-model.md`](../research/threat-model.md).

---

## Part A — Operator's half (TEMPLATE — the operator completes)

These sections are **deployment-specific** and **OPERATOR-OWNED**. They are left as prompts; the
engine cannot answer them.

| § | Question (Art. 35) | Who answers |
|---|---|---|
| A1 | **Controller identity + DPO** | Operator |
| A2 | **Nature, scope, context, purposes** of the processing — *what* personal data, about *whom*, *why* | Operator (decides what RDF to load) |
| A3 | **Lawful basis** (Art. 6) + special-category basis (Art. 9) if applicable | Operator |
| A4 | **Necessity & proportionality** — is the engine the minimum-intrusive means? | Operator (sparq's minimisation defaults *help* — see Part B) |
| A5 | **Data subjects + categories of data** | Operator |
| A6 | **Retention period** + erasure schedule | Operator (engine provides the *mechanism* — Part B) |
| A7 | **Recipients / international transfers** (incl. `SERVICE` federation targets) | Operator (sets the allowlist) |
| A8 | **Consultation** — DPO sign-off, data-subject consultation where required | Operator |
| A9 | **Residual-risk acceptance** + sign-off | Operator's DPO |

**The operator MUST complete Part A before relying on this DPIA.** sparq's defaults reduce, but do
not eliminate, the deployment risks the operator owns.

---

## Part B — Engine-side risk assessment (filled, with evidence)

The data-protection risks *intrinsic to the sparq binary*, the residual likelihood/impact given
the engine's shipped mitigations, and the operator action that closes each. Severity is the
**engine-side residual** assuming a competent operator deployment.

| # | Risk (engine-intrinsic) | Engine mitigation (evidence) | Residual | Operator action to close |
|---|---|---|---|---|
| **B1** | **Excessive collection via logging** — personal data captured incidentally in logs | Request logging **OFF by default**; metrics aggregate-only; no telemetry (`crates/sparq-server/src/http.rs:1447`, `:276`; `metrics.rs`) — `privacy/evidence.md` §P-2 | **Low** | Keep `--verbose` off, or enable log redaction (PR-G4 / sq-toze.34) |
| **B2** | **Personal data leaked in an error response** | Generic structured error bodies; no query text/rows/paths echoed (`http.rs:1429,1963,2090`); one residual: UPDATE *parse* errors echo a query fragment | **Low** | Track PR-G1 (sq-toze.32); don't expose raw server to untrusted clients without a gateway |
| **B3** | **Unauthorised access to personal data** (no per-user auth in the bare server — boundary B3) | Optional constant-time Bearer auth (`http.rs:567,629`); `sparq-solid` fail-closed graph-level WAC/ACP (`sparq-solid/README.md`, tests) | **Medium** if neither enabled; **Low** with `sparq-solid` or a gateway | **Front with a gateway or `sparq-solid`** — the documented B3 decision |
| **B4** | **Incomplete erasure** — a `DELETE`d triple survives in the `--persist` WAL | SPARQL `DELETE`/`DROP`/`CLEAR` durable ops (`update.rs:429-465`); in-memory default loses data on restart | **Medium** for persisted stores | Run `DELETE`/`DROP` **and rotate/compact the WAL**; adopt the retention runbook (PR-G3 / sq-toze.33) |
| **B5** | **Data exfiltration via federation** (`SERVICE`/`LOAD` turns query text into outbound requests) | `SERVICE` **deny-all by default**, explicit allowlist (`main.rs:24-29`); SSRF guard, boundary B4 | **Low** | Keep the allowlist tight; review `LOAD` write authorisation |
| **B6** | **Availability DoS** (a pathological query starves the service — a privacy-relevant denial) | `QueryBudget` (timeout + max-rows), body-size + concurrency caps, explicit 413/503 (`engine/src/lib.rs:56-78`; `http.rs:1435,2076`) | **Low** | Set budgets/caps to the deployment's needs |
| **B7** | **At-rest exposure** — plaintext persist WAL / archive on disk | None engine-side **by design** — at-rest encryption is the operator's | **Operator-owned** | Full-disk/volume encryption + FS permissions on the persist dir |
| **B8** | **In-transit exposure** — plaintext HTTP | None engine-side **by design** — TLS is the operator's | **Operator-owned** | Terminate TLS at a gateway/reverse proxy |
| **B9** | **Memory-safety breach of the on-disk loader** (a tampered index → UB) | Covered by `compliance/memsafety/` (the B5 mmap surface: register + Miri + oracle + fuzz) | **Low** (per memsafety attestation) | Control provenance of persisted index files |
| **B10** | **False reliance on ZK/MPC for privacy** — an operator believes the ZK/MPC estate protects data | **Documented NOT sound** (`SECURITY.md`; `research/zk-soundness-audit.md`); excluded from every privacy claim | **N/A** (engine makes no such claim) | **Do not rely on `sparq-zk`/`sparq-mpc` for any privacy guarantee** — gated by sq-toze.35 |

---

## Part C — Conclusion of the engine-side assessment

Assuming a competent operator deployment (TLS termination, disk encryption, `sparq-solid` or a
gateway for authz, tight `SERVICE` allowlist, a retention/erasure process that rotates the WAL),
**the engine-intrinsic data-protection residual risk is LOW.** The engine's conservative defaults
(no logging, in-memory only, deny-all egress, generic errors, fail-closed authz) actively support
a data-minimising deployment.

**What this DPIA does NOT conclude:** it does **not** conclude that any *specific deployment* is
GDPR-compliant or low-risk — that depends entirely on Part A, which only the operator can complete.
And it draws **no** privacy assurance from the ZK/MPC estate (B10).

**Sign-off:** the engine-side half (Part B/C) is the responsibility of the sparq maintainers
(this framework). **Part A and the overall DPIA sign-off are the operator's DPO's** — this
document is a skeleton to accelerate that, not a substitute for it.
