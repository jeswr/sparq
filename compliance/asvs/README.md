<!-- [OPUS-4.8] sq-toze.10 — OWASP ASVS framework slice. Authored while Fable
     unavailable — re-review when Fable returns. Engineer↔auditor loop (epic sq-toze). -->

# OWASP ASVS L1/L2 — sparq HTTP query API

**Framework:** OWASP Application Security Verification Standard (ASVS) v4.0.3, Levels 1 & 2.
**Target:** `sparq-server` (the HTTP SPARQL-protocol query/update API) and the RDF/SPARQL
**parsers** it feeds (`sparq-core`, vendored `spargebra` in `sparq-engine`), plus the
`sparq-serve` reactive surface. This is the only sparq surface that takes **untrusted
network input**, so ASVS is scoped *here* and not to the wider library/crypto estate.

**Companion docs:**
[`controls/asvs.md`](./controls/asvs.md) — the per-requirement control → status → evidence
table (the spine the auditor checks);
[`evidence.md`](./evidence.md) — per-claim file/line/CI verification with re-run commands;
[`gap-register.md`](./gap-register.md) — open gaps, severity, remediation, the `bd` bead.

## What sparq is, for ASVS purposes

sparq is a **Rust RDF/SPARQL data-engine library plus an HTTP server**, consumed as a
dependency in high-security settings — **not** a session-based web application with user
accounts, browser-rendered HTML, cookies, or a login UI. ASVS was written principally for
the latter. We therefore apply the **input-validation, error/logging, data-protection,
malicious-code, files/resources, configuration, and web-service/API** chapters in full, and
mark the **session/authentication-UI/access-control** chapters **N/A** with a documented
architectural rationale (the threat-model **B3** boundary), *not* as a silent gap.

## The B3 boundary — no authentication by default, by design

`research/threat-model.md` boundary **B3** records sparq-server's deliberate posture: it
ships with **no authentication and no per-user authorization by default**, and is intended
to be **fronted by a gateway / `sparq-solid`** (or run on loopback) in any deployment that
needs identity. This is the single most important scoping decision for ASVS:

- It is an **explicit architectural decision**, surfaced to the operator: the server
  **refuses to bind to a non-loopback address** unless the operator opts in
  (`--allow-remote`) *or* configures full authentication
  (`crates/sparq-server/src/http.rs` `bind_posture()` / `AuthPosture`). Default bind is
  `127.0.0.1:3030`.
- An **optional** QLever-style bearer-token gate exists (`--auth-token` /
  `SPARQ_AUTH_TOKEN`, plus `--auth-token-read` to additionally gate reads), with a
  constant-time comparison and an identical 401 for missing vs wrong tokens. This is a real,
  tested control (chapter V13/V2 below) — but the *full* ASVS authentication lifecycle
  (password policy, MFA, account lockout, credential recovery, session management) is the
  **operator's gateway responsibility**, not sparq's.

So the ASVS V2 (Authentication), V3 (Session Management), and V4 (Access Control) chapters
are **mostly N/A to sparq itself** and **assigned to the operator**, with the *thin
applicable residual* (the optional token gate, the bind posture, the constant-time compare)
mapped as **implemented & verified**.

## Status legend (honesty contract)

- **PASS** — *implemented & verified*: a technical control in the codebase/CI with a
  re-runnable evidence pointer (file:line + test name + CI job). The auditor can re-read or
  re-run the cited artifact.
- **AUDIT-READY** — control + documentation present, but the formal *attestation* (an ASVS
  L2 verification by an accredited assessor, or a penetration test) needs an external body
  we cannot substitute for. Labelled, not claimed as passed.
- **GAP** — not met (or only partially); recorded in [`gap-register.md`](./gap-register.md)
  with a severity, a remediation plan, and a `bd` bead. **Not** papered over.
- **N/A (reason)** — out of scope for a library/server with no session/login UI; the reason
  (and, where relevant, the operator-responsibility assignment) is stated inline.

## Chapter applicability at a glance

| ASVS chapter | Applicability to sparq | Where it lands |
|---|---|---|
| V1 Architecture, Design & Threat Modeling | **Applies** | `research/threat-model.md`, `CONTRIBUTING.md#secure-coding`, this doc |
| V2 Authentication | **Mostly N/A** (operator); thin residual = optional token gate | B3 boundary; token gate PASS |
| V3 Session Management | **N/A** — stateless HTTP query API, no sessions/cookies | operator gateway |
| V4 Access Control | **Mostly N/A** (operator); residual = read/write op-classification gate | B3 boundary |
| V5 Validation, Sanitization & Encoding | **Applies — core** | parsers + body limits + error escaping |
| V6 Stored Cryptography | **Applies narrowly** | constant-time token compare; ZK/MPC **excluded** (not a control) |
| V7 Error Handling & Logging | **Applies — core** | structured JSON errors, panic isolation, opt-in tracing |
| V8 Data Protection | **Applies narrowly** | no PII collection; TLS = proxy responsibility |
| V9 Communication | **Mostly operator** | TLS terminated at proxy by design |
| V10 Malicious Code | **Applies** | supply-chain gates, `forbid(unsafe_code)`; **no SAST** — the CodeQL lane is disabled and gates nothing (GX-14), so L3 V10.1.1 is unmet |
| V11 Business Logic | **Applies narrowly** | DoS/anti-automation limits |
| V12 Files & Resources | **Applies** | SSRF egress allowlist, decompression-bomb cap, no file serving |
| V13 API & Web Service | **Applies — core** | method restriction, content-type, the token gate |
| V14 Configuration | **Applies — core** | secure defaults, bind posture, no secrets in code |

## What is explicitly OUT OF SCOPE here

<!-- [OPUS-4.8] reconciled with post-remediation re-audit (sq-gbp4); see ZK-verdict cross-ref sweep -->
- **The ZK/MPC crypto estate** (`sparq-zk`, `sparq-zk-compose`, `sparq-mpc`). Per
  `SECURITY.md` (§"`sparq-zk` and `sparq-zk-compose` — ZK verifier: remediated, but NOT
  externally audited") and `research/zk-soundness-audit.md` + `research/zk-verifier-reaudit.md`,
  the ZK verifier was originally found unsound, the `sq-1s2` binding layer has landed, and the
  internal re-audit found it "sound as landed for the assumed threat model" — **but** external
  cryptographer sign-off is still PENDING (`sq-qhy4`, P0), there is **no production guarantee**,
  and the MPC layer carries **no guarantee**. They remain research scaffolds, are **not** a
  delivered security control, and are **excluded** from every ASVS claim below. The
  cryptographic review is its own framework (`cryptoreview`); the privacy story is `privacy`. [OPUS-4.8]
- **The full authentication / session / account lifecycle** (V2/V3/V4) — operator gateway
  responsibility per B3 (above).
- **Memory-safety attestation** (the `unsafe` register, Miri, geiger ratchet) — its own
  framework (`compliance/memsafety/`). ASVS V10 references it as evidence but does not
  re-litigate it.
- **Supply-chain attestation depth** (SBOM/VEX/SLSA/cargo-vet) — owned by `sbom` / `slsa` /
  `ssdf`. ASVS V10/V14 cite the gating cargo-deny lane as evidence; the *completeness* of the
  SBOM is assessed there.
