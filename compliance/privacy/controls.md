<!-- [OPUS-4.8] sq-toze — Privacy control table (GDPR + ISO 27701 + SOC 2 Privacy).
     One row per applicable control. Engineer↔auditor loop (epic sq-toze).
     Re-review when Fable returns. -->

# Privacy — control table (GDPR + ISO/IEC 27701 + SOC 2 Privacy)

Each row maps a privacy obligation to **either** an engine-provided technical capability
(with repo evidence) **or** an operator responsibility (labelled). Evidence paths are
repo-relative. See [`evidence.md`](./evidence.md) for the per-cited file/line/test detail and
[`README.md`](./README.md) for the responsibility-split framing.

## Status legend

`IMPL` = implemented & verified · `AUDIT-READY` = control+docs present, certificate needs the
operator's PIMS/an external auditor · `OPERATOR` = property of the deployment, not the source
(engine hook noted) · `GAP` = engine could help but doesn't yet (→ [`gap-register.md`](./gap-register.md)).

---

## A. GDPR principles (Art. 5) + data-subject rights (Art. 12–22)

| # | Obligation | Status | Engine capability / operator note | Evidence |
|---|---|---|---|---|
| P-1 | **Lawfulness, fairness, purpose limitation (Art. 5(1)(a–b))** — process for a stated lawful purpose only. | **OPERATOR** | The engine has no notion of "purpose"; it answers any query over loaded data. The operator decides what to load, the lawful basis, and which queries to expose. **Engine hook:** `sparq-solid` graph-level authz lets the operator scope *who* sees *what* per purpose. | `crates/sparq-solid/README.md`; `../data-flow.md` §responsibility-split |
| P-2 | **Data minimisation (Art. 5(1)(c))** — process only what is necessary; don't accumulate incidental personal data. | **IMPL** (engine-side defaults) | The engine does **not** log request bodies/queries by default — request logging (`TraceLayer`) is OFF unless `--verbose` is passed. Prometheus metrics are **aggregate counters/histograms only** (per-endpoint/status counts, latency buckets, triple/subscription gauges) — no query text, no result rows, no client identifiers. No analytics/telemetry phones home. | `crates/sparq-server/src/http.rs:1447` (`if config.verbose { …TraceLayer }`, default `verbose:false` at `:276`); `crates/sparq-server/src/metrics.rs` (counters/histogram only) |
| P-3 | **Storage limitation / retention (Art. 5(1)(e))** — keep personal data no longer than necessary; support erasure. | **OPERATOR** (engine provides the erasure mechanism) | Retention *policy* is the operator's. The engine provides the *means*: in-memory by default (data is gone on restart unless `--persist`); full SPARQL `DELETE DATA` / `DELETE … WHERE` / `CLEAR` / `DROP` for selective erasure. **Caveat:** `--persist DIR` is a replayable write-after-write WAL — see P-9. | `crates/sparq-engine/src/update.rs:429-465` (`DeleteData`/`Clear`/`Drop` route through durable `Graph` methods) |
| P-4 | **Right to erasure / "right to be forgotten" (Art. 17)** — delete a data subject's personal data on request. | **IMPL** (mechanism) / **OPERATOR** (process) | The engine supports SPARQL `DELETE`/`DROP`/`CLEAR` (named-graph or pattern-scoped). `DROP GRAPH <g>` removes a whole document; `DELETE { ?s ?p ?o } WHERE { … }` removes a subject's triples. The operator must *identify* the subject's data and *run* the deletion (and purge any `--persist` WAL — P-9). | `crates/sparq-engine/src/update.rs:451-465`; conformance: SPARQL 1.1 Update is W3C-conformance-gated |
| P-5 | **Right of access / portability (Art. 15, 20)** — export a data subject's data in a structured format. | **IMPL** (mechanism) / **OPERATOR** (process) | A `SELECT`/`CONSTRUCT`/`DESCRIBE` query exports the subject's triples in standard, machine-readable RDF (Turtle/N-Triples/JSON results) — RDF is inherently portable + interoperable. The operator scopes + runs the query. | `crates/sparq-engine/src/lib.rs:39-46` (construct/describe entry points); standard SPARQL result serialisations in `sparq-server` |
| P-6 | **Right to rectification (Art. 16)** — correct inaccurate personal data. | **IMPL** (mechanism) | SPARQL `DELETE { old } INSERT { new } WHERE { … }` performs atomic rectification. Multi-op update bodies are all-or-nothing (no partial apply). | `crates/sparq-engine/src/update.rs:481` (`DeleteInsert`); test `multi_op_one_unauthorized_denies_whole_body_no_partial_apply` (`crates/sparq-solid/tests/update.rs:443`) |
| P-7 | **Accuracy (Art. 5(1)(d))** — query results faithfully reflect the loaded data (no silent wrong answers). | **IMPL** (correctness) | Result integrity is the #1 asset in `research/threat-model.md`; the engine is W3C-SPARQL-1.1-conformance-gated and budget hits surface as explicit errors, never silent truncation. (This is a *correctness* control that supports the accuracy principle, not a PII control per se.) | `research/threat-model.md` §assets; `crates/sparq-engine/src/lib.rs:62` (budget errors are explicit, not silent) |
| P-8 | **Accountability / auditability (Art. 5(2))** — demonstrate processing; an audit trail of access. | **OPERATOR** (engine provides opt-in trace + access-control verdicts) | The engine emits **no audit log by default** (data-minimisation, P-2). The operator who needs an access trail enables `--verbose` request logging and/or wraps the server in a gateway that logs WebID + query. `sparq-solid` produces fail-closed allow/deny verdicts per `(WebID, client)` session that an operator can record. **Gap:** no built-in structured, queryable access/audit log (→ [`gap-register.md`](./gap-register.md) PR-G2). | `crates/sparq-server/src/http.rs:171` (TraceLayer, opt-in); `crates/sparq-solid/src/update.rs:272` (allow/deny verdict) |

---

## B. Confidentiality, access control & security of processing (Art. 32 / 27701 / SOC 2 *Confidentiality* + *Privacy*)

| # | Obligation | Status | Engine capability / operator note | Evidence |
|---|---|---|---|---|
| P-9 | **Confidentiality of stored personal data (Art. 32(1)(a–b))** — protect data at rest. | **OPERATOR** | The engine holds data **in process memory** by default and (with `--persist DIR`) writes a plaintext write-ahead log to the operator's filesystem. **There is no engine-side at-rest encryption** — this is deliberately the operator's responsibility (full-disk/volume encryption, OS permissions on the persist dir). **Erasure caveat:** the `--persist` WAL is *append/replay* — a deleted triple's prior INSERT may still be present in earlier WAL segments until the operator compacts/rotates, so a true Art. 17 erasure must also purge the WAL (→ remediation runbook, bead sq-toze.33). | `crates/sparq-server/src/main.rs:14-22` (persist semantics; "in-memory only" default at `:321`) |
| P-10 | **Access control / need-to-know (Art. 32 / 27701 6.6 / SOC 2 CC6)** — restrict who can read/write personal data. | **IMPL** (capability) / **OPERATOR** (must enable) | Two layers: **(a) transport auth** — optional `--auth-token` (Bearer) gates writes; `--auth-token-read` also gates reads; tokens are **constant-time compared** (no timing oracle; identical 401 for missing-vs-wrong). **(b) fine-grained authz** — `sparq-solid` enforces **graph-level WAC/ACP**, **fail-closed**: absence of a grant makes a graph *invisible*, indistinguishable from absent; every query is filtered per `(WebID, client)` session. The core server itself has **no per-user authz** (documented boundary **B3** — front with a gateway / `sparq-solid`). | `crates/sparq-server/src/http.rs:567` (`constant_time_eq`), `:629` (`auth_check`), `:200` (identical-401 note); `crates/sparq-solid/README.md` (fail-closed); tests `crates/sparq-solid/tests/{wac,acp,hardening,update}.rs` |
| P-11 | **No privilege escalation via data (27701 / SOC 2 CC6.1)** — loaded content cannot grant itself access. | **IMPL** | `sparq-solid` feeds the authorisation reasoner **only ACL/ACR + structural facts, never pod content**, so no writable document can grant itself access; the reserved `urn:sparq:` namespace is rejected on input and forged `<urn:sparq:auth>` graphs are stripped at load. | `crates/sparq-solid/README.md` §security-posture; tests `crates/sparq-solid/tests/hardening.rs:13` (forged auth-graph stripped), `:63` (`reserved_session_values_fail_closed`) |
| P-12 | **Error/log hygiene — no personal data leakage in errors/logs (Art. 5(1)(f) / SOC 2 CC6.7)** — error responses and logs must not echo query text, RDF content, or internal paths. | **IMPL** (with one honest caveat) | HTTP error bodies are **generic, structured JSON**: `"internal server error (panic)"`, `"not found"`, `"query worker panicked"`, `"server is at its concurrent-request limit"`, budget messages that name *limits*, not data. No query text / result rows / RDF content / stack traces / filesystem paths are echoed. **Caveat:** a SPARQL *parse/semantic* error on an UPDATE is returned as `"update failed: {e}"` where `{e}` is the parser diagnostic — which can include a *fragment of the malformed query text* (not loaded data). Treated as a low-severity residual (→ [`gap-register.md`](./gap-register.md) PR-G1). | `crates/sparq-server/src/http.rs:1429,1776,1963,2090` (generic bodies); caveat at `:2118` (`update failed: {e}`); `crates/sparq-server/src/metrics.rs` (no PII in metrics) |
| P-13 | **SSRF / data-exfiltration guard on federation (Art. 32 / SOC 2 CC6.6)** — a `SERVICE` clause must not turn attacker query text into outbound requests that exfiltrate or pivot. | **IMPL** | `SERVICE` federation is **deny-all by default**: a `SERVICE <iri>` reaches nothing unless its host is on an explicit `--service-allow` / `SPARQ_SERVICE_ALLOW` allowlist. Documented SSRF guard (threat-model boundary B4). | `crates/sparq-server/src/main.rs:24-29` (deny-all SSRF allowlist); `research/threat-model.md` §B4 |
| P-14 | **Availability under adversarial input (Art. 32(1)(b) / SOC 2 *Availability* for the operator)** — a query cannot exhaust resources and deny the service (which can itself be a privacy-relevant DoS). | **IMPL** (engine-side limits) / **OPERATOR** (SLA) | `QueryBudget` (timeout + max-rows), `--max-body-bytes`, `--max-concurrent` (load-shed), `--max-query-rows`/`--max-results`. Budget hits surface as explicit 413/503, not silent truncation. The *availability SLA* is the operator's. | `crates/sparq-engine/src/lib.rs:56-78` (`QueryBudget`); `crates/sparq-server/src/http.rs:1435` (concurrency load-shed), `:2076` (budget→413) |
| P-15 | **Client-side processing keeps data local (data minimisation / no transfer)** — the WASM port runs the engine in the user's own browser; personal data need not leave the client. | **IMPL** (architecture) | `sparq-wasm` compiles the engine to run **in-browser**; an embedding app can query personal RDF entirely client-side with no server round-trip and no third-party transfer. (The embedding app's own data handling is its responsibility.) | `crates/sparq-wasm/` (`#![forbid(unsafe_code)]` WASM build); `../data-flow.md` §WASM-client-surface |

---

## C. PII-controller/processor obligations (ISO/IEC 27701) + SOC 2 *Privacy* criteria

| # | Obligation | Status | Engine capability / operator note | Evidence |
|---|---|---|---|---|
| P-16 | **27701 7.x — PII *controller* obligations** (notice, consent, purpose, RoPA, DPIA, data-subject request handling). | **OPERATOR** | These are **controller** duties. sparq is not the controller. None of notice/consent/purpose/RoPA live in the engine; the operator's PIMS owns them. | `README.md` §responsibility-split; `../dpia.md` |
| P-17 | **27701 8.x — PII *processor* obligations** (process only on instruction, assist the controller with subject rights, sub-processor transparency, return/delete at end of processing). | **AUDIT-READY** (engine provides the levers; operator's DPA governs) | If an operator runs sparq as a *processor* for a controller, the engine gives the levers: query-on-instruction (it only answers asked queries), erasure/rectification/access via SPARQL (P-4/5/6), and "delete at end" via `DROP`/`CLEAR` + WAL purge. The **DPA, sub-processor list, and instruction discipline are the operator's** — a certificate needs the operator's PIMS + an external auditor. | `crates/sparq-engine/src/update.rs`; this control table P-4/5/6 |
| P-18 | **SOC 2 *Privacy* P1–P8 (notice, choice/consent, collection, use/retention/disposal, access, disclosure, quality, monitoring/enforcement)** | **OPERATOR** (mostly) / **IMPL** (the *disposal* + *access* + *quality* mechanisms) | Notice/choice/collection/disclosure/monitoring are entity-level (operator). The engine *mechanically supports* **P4 retention/disposal** (SPARQL erasure, P-3/4), **P5 access** (export query, P-5), and **P8 quality** (conformance-gated accuracy, P-7). | This table P-3/4/5/7; SOC 2 attestation is external by definition. |
| P-19 | **Sub-processor / supply-chain transparency (27701 8.5.6 / SOC 2 CC9.2)** — disclose components that could process PII. | **IMPL** (mechanism) / cross-ref | A CycloneDX SBOM enumerates every dependency (the `sbom` framework owns this); an operator's RoPA can cite it. No dependency phones home with data (data-minimisation, P-2). | cross-ref `compliance/sbom/` (SBOM framework); `research/production-certification-plan.md` §sbom |

---

## ZK/MPC privacy story — EXPLICIT carve-out (honesty contract)

> **No privacy capability claimed in this table rests on the ZK or MPC estate.**
>
> sparq ships `sparq-zk`, `sparq-zk-compose`, and `sparq-mpc`, which *model* privacy-preserving
> query proofs (zero-knowledge: prove a result without revealing the data) and multi-party
> computation (compute over data split across distrusting parties without pooling it). **If
> sound, these would be powerful privacy-by-cryptography controls.** They are **not yet sound**:
>
> - The **v1 ZK verifier (`verify_manifest`) provides NO meaningful soundness guarantee** — a
>   prover that controls its own side can make it accept arbitrary false results
>   (`SECURITY.md`; `research/zk-soundness-audit.md` — 12 confirmed issues, 6 critical).
> - **`sparq-mpc` provides no confidentiality/correctness/attestation/malicious-security
>   guarantee** — the MPC cryptography is deferred and not implemented (`SECURITY.md`).
>
> Therefore: **the ZK/MPC estate is presented as a research scaffold with NO privacy guarantee**,
> and any future "privacy by cryptography" claim is **GATED on the soundness fix** (bead
> **sq-toze.35**). A control table or maturity score that implied these provide a working privacy
> guarantee would be an automatic high-severity overclaim. They are **out of scope** for every
> IMPL/AUDIT-READY label above and appear here only to be explicitly excluded.

---

## Owner

All rows: **Privacy compliance engineer** (this framework), branch `cert-privacy`, epic
`sq-toze`. Operator-responsibility rows transfer to the **deploying operator's DPO/ISMS** at
deployment; AUDIT-READY rows additionally require an **accredited external auditor** for the
27701/SOC 2 certificate.
