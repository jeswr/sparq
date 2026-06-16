<!-- [OPUS-4.8] sq-toze — Compliance index. LEAD-OWNED at consolidation; each framework
     engineer writes their own row. This worktree (cert-privacy) seeds the index + the
     Privacy row only. Other rows are placeholders for the lead/other engineers to fill. -->

# sparq compliance — certification readiness

Per-framework certification evidence for `sparq` (a Rust RDF/SPARQL data-engine + HTTP server +
crypto estate consumed as a dependency in high-security settings). Each framework has its own
folder; the engineer↔auditor loop (epic **sq-toze**) drives each to zero auditor findings.

> **Honesty contract.** Every control claim cites concrete evidence (file/test/CI). The ZK/MPC
> estate is documented as **NOT yet sound** (`SECURITY.md`, `research/zk-soundness-audit.md`) and
> is **excluded** from every security/privacy guarantee claim. No framework claims a deployment
> property (e.g. "sparq is GDPR compliant") — only the technical capabilities the engine provides.

## Status legend

- **IMPLEMENTED & VERIFIED** — a technical control in the codebase/CI with passing, re-runnable
  evidence.
- **AUDIT-READY** — control + documentation in place, but the *certificate* needs an accredited
  external body / the operator's ISMS/PIMS / an external cryptographer.
- **OPERATOR-RESPONSIBILITY** — a property of the deployment, not the source (engine hook noted).
- **GAP** — not met; in the framework's `gap-register.md` with a `bd` bead.

## Frameworks

| Framework | Folder | Posture | Engineer status |
|---|---|---|---|
| **Privacy** (GDPR + ISO 27701 + SOC 2 Privacy) | [`privacy/`](./privacy/) | **AUDIT-READY** — engine provides a respectable privacy-capability set (data-min by default, SPARQL erasure, fail-closed graph authz, generic errors, SSRF guard); cert + deployment duties are operator/external | ✅ slice complete (this worktree) |
| Memory-safety attestation | [`memsafety/`](./memsafety/) | substantively PASS (unsafe register + ratchet + Miri/oracle/fuzz) | see `memsafety/audit-log.md` |
| OWASP ASVS L2 | `asvs/` | _(other worktree)_ | — |
| CIS Docker | `cis/` | _(other worktree)_ | — |
| SBOM + supply-chain | `sbom/` | _(other worktree)_ | — |
| NIST SSDF | `ssdf/` | _(other worktree)_ | — |
| SLSA provenance | `slsa/` | _(other worktree)_ | — |
| OpenSSF Scorecard/Best-Practices | `openssf/` | _(other worktree)_ | — |
| ISO/IEC 27001 | `iso27001/` | _(other worktree)_ | — |
| EU CRA | `cra/` | _(other worktree)_ | — |
| Cryptographic review (ZK/MPC) | `cryptoreview/` | _(other worktree)_ | — |
| CDMC scoring | `cdmc/` | _(other worktree)_ | — |

## Shared cross-framework artifacts (owned by the Privacy engineer)

| File | What |
|---|---|
| [`data-flow.md`](./data-flow.md) | Everywhere the binary can touch data + the operator/engine responsibility split |
| [`dpia.md`](./dpia.md) | DPIA *skeleton* — engine-risk half filled with evidence; operator completes the deployment half |
| [`threat-model.md`](./threat-model.md) | Privacy lens on the STRIDE model (references `research/threat-model.md`, does not fork it) |

## The operator-vs-engine split (the recurring frame)

sparq is a **data engine**, not a service that collects personal data or makes controller
decisions. Across the personal-data frameworks the recurring frame is: **the engine provides
technical capabilities** (access control, deletion, minimisation, confidentiality, auditability)
**with repo evidence; the deploying operator is the data controller** and owns lawful basis,
purpose, retention policy, TLS, at-rest encryption, and the certificate. See
[`privacy/README.md`](./privacy/README.md).
