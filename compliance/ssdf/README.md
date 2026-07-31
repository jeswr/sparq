<!-- [OPUS-4.8] SSDF framework intro (cert framework `ssdf`, epic sq-toze, bead
     sq-toze.13). Re-review when Fable returns. NON-CANONICAL timing. -->

# NIST SSDF (SP 800-218 v1.1) — secure-software-development framework

The **NIST Secure Software Development Framework** (SP 800-218 v1.1) is the canonical
*implementer-side* secure-SDLC framework: a set of practices (grouped **PO** Prepare the
Organization, **PS** Protect the Software, **PW** Produce Well-Secured Software, **RV**
Respond to Vulnerabilities) that a software *producer* follows so the artifacts it ships
are well-secured. sparq is shipped as a dependency (crates.io / npm / PyPI / ghcr), so SSDF
is a high-fit framework — and, because the practices map directly onto sparq's existing
gate stack, it is largely a **mapping** exercise rather than a build-out.

## Why SSDF (vs a certification framework)

SSDF is **not a certificate**. There is no "SSDF auditor" who issues a pass/fail seal; it is
the practice catalogue that producers self-attest to (e.g. in a US federal software
attestation form). So the deliverables here are an honest **practice → evidence** mapping a
deploying organization can lift into its own attestation — not a pending external cert.

## What this folder contains

| File | Purpose |
|---|---|
| [`controls.md`](./controls.md) | The spine: every SSDF task (PO/PS/PW/RV, 42 task rows) → status → repo evidence (file / test / CI job) → owner. |
| [`evidence.md`](./evidence.md) | By-artifact index resolving each control claim to a concrete, checkable location + how to reproduce it locally. |
| [`gap-register.md`](./gap-register.md) | Open gaps (severity, remediation, target, `bd` bead). |

## Scope — what's in, what's the operator's job

**In scope (the producer = sparq project):**
- The secure-SDLC *gates* — clippy `-D warnings`, `cargo test`/conformance
  ratchets, Miri, fuzz, the unsafe-count ratchet, cargo-deny advisories/bans/sources/
  licenses (gating), cargo-vet, the CycloneDX SBOM + VEX, SLSA build provenance, and the
  `ci-summary / gate` aggregator. **CodeQL SAST is deliberately absent from this list:**
  `.github/workflows/codeql.yml` is retained on `main` but has been **disabled at the Actions
  level (`disabled_manually`) since 2026-07-18** by separate maintainer direction (merge
  latency), so it runs on **no** event, produces no check-run or SARIF, and gates nothing.
  **Nothing compensates** — none of the live gates above performs taint or crypto-misuse
  analysis. Anchor: cross-cutting gap **GX-14** (P1) in
  [`../gap-register.md`](../gap-register.md); narrative in `ASSURANCE.md` §11; open posture
  decision **#4620**.
- The secure-coding standard (`CONTRIBUTING.md`), the threat model
  (`research/threat-model.md`), the dependency policy (`deny.toml`), and the coordinated
  vulnerability-disclosure programme (`SECURITY.md` + `.well-known/security.txt`).

**Out of scope (the operator/deploying organization owns):**
- SSDF practices about the *operating environment* — production deployment, environment
  separation in *their* infra (PO.5 partially), runtime monitoring, and their own incident
  response. sparq is a library/engine; the deploying org runs the SSDF programme for its
  service.
- The server's **authentication/authorization** is, by documented design (threat-model
  boundary **B3**), the operator's responsibility ("front with a gateway / sparq-solid") —
  an explicit architectural decision, not an SSDF gap.
- **Formal external attestation** of the audit-ready practices — for a single-maintainer
  volunteer project these are documented + automated and asserted; an org adopting sparq
  performs the formal attestation against its own ISMS.

## Honesty posture

The coverage summary in `controls.md` reports **26 implemented & verified / 2 partial / 13
audit-ready / 1 gap** across **42 rows = 41 standard SP 800-218 v1.1 tasks + 1 flagged
sparq-local row**
(`RV.1.4`, the daily advisory watchdog — evidence supporting standard task RV.1.3, **not** a
separate framework task; see the footnote in `controls.md`). <!-- [OPUS-4.8] sq-ce97: 41
standard (RV.1 has 3 tasks) + 1 local row = 42 rows. [OPUS-5] tally re-footed 28/13/1 →
26/2/13/1 for the CodeQL/GX-14 reconciliation; no row added or removed. -->
The two **partial** rows are **PW.7.1 / PW.7.2** (static analysis): they were scored on CodeQL,
which has been `disabled_manually` since **2026-07-18**, so **no SAST runs** and **nothing
compensates** — clippy, the unsafe ratchet, cargo-deny/cargo-vet, fuzz and Miri are all live and
genuine but none performs taint or crypto-misuse analysis. The 35 open critical
`rust/hard-coded-cryptographic-value` alerts were **triaged** (issue #4615) as false positives of
one query-model defect — *triaged is not covered*. Anchor: **GX-14** (P1) in
[`../gap-register.md`](../gap-register.md); `ASSURANCE.md` §11; open posture decision **#4620**.
The single row scored as an outright gap is **PW.6.2 reproducible-build**
(GX-8, bead **sq-toze.9**) — now *characterised*: the honest reproducibility statement is
documented ([`../slsa/reproducible-build.md`](../slsa/reproducible-build.md)), with only the CI
rebuild-and-diff enforcement outstanding. No row presents the `sparq-zk*` / `sparq-mpc`
research scaffold as a met security control. [OPUS-4.8] The v1 ZK verifier was **originally found
unsound** (`research/zk-soundness-audit.md`, kept on record), then `sq-1s2` landed the binding
layer and an **internal post-remediation re-audit** (`research/zk-verifier-reaudit.md`, `sq-gbp4`)
found the prior findings closed → **"sound as landed for the assumed threat model"** — but that
verdict is **internal / single-model self-review only, with external accredited-cryptographer
sign-off still PENDING (`sq-qhy4`, P0) and NO production guarantee** (`SECURITY.md`). That
remediated-but-externally-unaudited posture is a correctly-disclosed limitation, and SSDF
PW.4/PW.5/PW.8 are scored on the engine, never on the crypto scaffold.
<!-- [OPUS-4.8] reconciled with post-remediation re-audit (sq-gbp4); see ZK-verdict cross-ref sweep -->
