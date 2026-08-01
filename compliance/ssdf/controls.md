<!-- [OPUS-4.8] NIST SSDF (SP 800-218) control→evidence mapping (cert framework `ssdf`,
     epic sq-toze, bead sq-toze.13). Authored while Fable unavailable — re-review when
     Fable returns. NON-CANONICAL timing (EC2 work box) — no measured numbers here. -->

# NIST SSDF (SP 800-218 v1.1) — control → status → evidence

This is the spine the SSDF auditor checks. One row per SSDF **task** (the SP 800-218
practice tasks PO.1–PO.5, PS.1–PS.3, PW.1–PW.9, RV.1–RV.3). Each row carries an honest
status label and **repo-relative evidence** (the file that enforces the control, the test
that regresses it, the `.github/workflows/<wf>.yml` CI job that gates it).

**Status legend** (per the honesty contract):

- **Implemented & verified** — a technical control in the codebase/CI with passing
  evidence (a file, a gating CI job, a test).
- **Partial** — part of the practice is met by live evidence and part is **not**; the
  unmet part is named in the row and carried in [`gap-register.md`](./gap-register.md).
  Used here only for the two rows whose status materially rested on the now-disabled SAST
  lane (see the SAST note below).
- **Audit-ready** — the control + its documentation exist, but a *formal attestation* of
  the practice is an organizational/ISMS act an accredited auditor performs against the
  operating organization; SSDF is an *implementer/producer* framework with no certificate,
  so for a single-maintainer research project these are the practices that are **mapped and
  evidenced** but whose continuous operation is asserted, not externally certified.
- **Gap** — not met; tracked in [`gap-register.md`](./gap-register.md) with a `bd` bead.

**Scope note.** SSDF is written for an *organization* producing software. sparq is a
pre-1.0, best-effort, single-maintainer (`@jeswr`) research project (see `SECURITY.md`,
`AGENTS.md`). Where SSDF assumes an org with roles/RACI/management buy-in, the honest
mapping is "the practice is implemented as a *documented, automated gate* rather than an
org-process artifact." Those are labelled **audit-ready** with the gate cited as evidence.
The **producer = sparq project**; the **acquirer/operator = whoever deploys it** — the
operator's own SSDF programme (their deployment, monitoring, IR) is out of sparq's scope.

<!-- [OPUS-4.8] reconciled with post-remediation re-audit (sq-gbp4); see ZK-verdict cross-ref sweep -->
> The single most load-bearing honesty item in this repo is the documented ZK-verifier
> verdict. [OPUS-4.8] The v1 verifier was **originally found unsound**
> (`research/zk-soundness-audit.md`, kept on record); `sq-1s2` then landed the verifier-side
> binding layer and an **internal post-remediation re-audit** (`research/zk-verifier-reaudit.md`,
> `sq-gbp4`) found the prior findings closed → **"sound as landed for the assumed threat
> model"** — but that is **internal / single-model self-review only, with external
> accredited-cryptographer sign-off still PENDING (`sq-qhy4`, P0) and NO production guarantee**
> (`SECURITY.md`). No SSDF row below claims the `sparq-zk*`/`sparq-mpc` estate provides a
> production security guarantee — PW.4/PW.5/PW.8 are scored on the *engine*
> (`sparq-core`/`-engine`/`-server`), and the crypto scaffold's remediated-but-externally-unaudited
> status is treated as a *correctly disclosed* limitation, not a met control.

<!-- [OPUS-5] CodeQL reconciliation — anchor: cross-cutting gap GX-14 in
     ../gap-register.md; narrative: ASSURANCE.md §11; posture decision: issue #4620. -->
> **SAST is NOT running — CodeQL is operationally disabled, and nothing compensates.**
> `.github/workflows/codeql.yml` has been disabled at the Actions level
> (`disabled_manually`) since **2026-07-18**, by separate maintainer direction (merge
> latency). The file and its triggers are retained on `main`, but GitHub schedules **no**
> run on **any** event (push, pull_request, merge_group, schedule): there is no `CodeQL
> analysis (rust)` check-run, no SARIF upload to code scanning, it feeds `ci-summary`
> nothing, and it gates nothing. Every row below that previously cited CodeQL as
> implemented/verified evidence has been corrected in place; **PW.7.1** and **PW.7.2** —
> whose status materially rested on it — are downgraded to **Partial**. **No compensating
> SAST control exists:** clippy `-D warnings`, the unsafe-count ratchet,
> cargo-deny/cargo-vet and the fuzz/Miri lanes are all live and genuine, but **none**
> performs taint or crypto-misuse analysis. The 35 open critical
> `rust/hard-coded-cryptographic-value` alerts CodeQL left behind were **triaged** under
> issue #4615 and found to be false positives of a single query-model defect — *triaged is
> not covered*, and says nothing about what an enabled scanner would find now. The durable
> posture (re-enable advisory-only / re-enable on a schedule / adopt another SAST / accept
> and document no SAST) is an **open maintainer decision — issue #4620**. Anchor:
> cross-cutting gap **GX-14** (P1) in [`../gap-register.md`](../gap-register.md); narrative
> in `ASSURANCE.md` §11; SSDF slice entry: [`gap-register.md`](./gap-register.md) SSDF-G2.

---

## PO — Prepare the Organization

| Task | Practice | Status | Evidence (file / test / CI job) | Owner |
|---|---|---|---|---|
| **PO.1.1** | Define security requirements for software development; maintain them over time. | Audit-ready | `CONTRIBUTING.md` "Secure coding" section (secure-coding standard) + `SECURITY.md` (hardened-input expectations for `sparq-core`/`-engine`/`-server`) + `research/threat-model.md` (STRIDE, boundaries B1–B5), **consolidated into the org-adoptable [`../policies/policy-secure-sdlc.md`](../policies/policy-secure-sdlc.md) Secure-SDLC policy template** (`sq-5ty0`). The requirements are documented and version-controlled; the formal org-level register/sign-off is the operator/org act (the template's `<FILL-IN>` cells). | @jeswr |
| **PO.1.2** | Define roles and responsibilities for the SDLC throughout. | Audit-ready | `CODEOWNERS` (security-sensitive paths — `sparq-zk*`, `sparq-mpc`, `sparq-core`, `sparq-server`, `.github/`, `deny.toml`, `SECURITY.md` — route to `@jeswr` for required review) + `docs/branch-protection.md` (review-before-merge ruleset). Single-maintainer roles are explicit; a multi-role RACI is N/A for a solo project. | @jeswr |
| **PO.1.3** | Implement supporting toolchains; integrate security tooling; keep it current. | Implemented & verified | The **live** CI toolchain: `.github/workflows/{ci,supply-chain,miri,fuzz,scorecard,dependency-monitoring,release}.yml`; `.github/dependabot.yml` (4 ecosystems) keeps the toolchain + deps current; pinned-by-SHA actions across all workflows. **`codeql.yml` is struck from this evidence:** it is retained on `main` but `disabled_manually` since 2026-07-18, so it runs on no event and is not part of the operating toolchain (GX-14). The status stands on the seven remaining, genuinely-running workflows — but the toolchain has **no SAST tool in it**, and nothing else performs taint/crypto-misuse analysis. | @jeswr |
| **PO.2.1** | Define security checks / criteria for each SDLC stage (gate definitions). | Implemented & verified | `CONTRIBUTING.md` "The gate" + `docs/branch-protection.md` (single required check `ci-summary / gate`) + `.github/PULL_REQUEST_TEMPLATE.md` (per-change re-evaluation checklist tied to AGENTS.md); the per-stage check criteria are tabulated in [`../policies/policy-secure-sdlc.md`](../policies/policy-secure-sdlc.md) §3 (`sq-5ty0`). | @jeswr |
| **PO.2.2** | Provide role-based training / guidance to development personnel. | Audit-ready | `AGENTS.md` + `CONTRIBUTING.md` "Secure coding" are the contributor secure-coding guidance (unsafe policy, input-validation, supply-chain). For a solo/volunteer project this is the guidance artifact; formal training records are an org act. | @jeswr |
| **PO.2.3** | Obtain management commitment / leadership support for secure development. | Audit-ready | Implicit for a single-maintainer project (maintainer = decision authority). The disclosure SLAs in `SECURITY.md` and the gating CI are the standing commitment. No separate management-sign-off artifact applies. | @jeswr |
| **PO.3.1** | Specify which tools (or tool types) are mandated/recommended for each stage. | Implemented & verified | `deny.toml` (cargo-deny policy) + `.github/workflows/supply-chain.yml` (cargo-deny / cargo-vet / cargo-cyclonedx) + `miri.yml` + `fuzz.yml` + `ci.yml` clippy/fmt. The mandated tool set is encoded as gating jobs. **`codeql.yml` is struck as a mandated-tool citation** — the SAST *tool type* is still specified and its config is retained on `main`, but the workflow is `disabled_manually` (since 2026-07-18) so no SAST tool is currently mandated-**and-running**; that slot is empty and unsubstituted (GX-14; posture decision #4620). The status stands because the specification itself — which tool type is required at which stage — is intact and every *other* named tool gates for real; the un-run SAST slot is scored under PW.7.1/PW.7.2, which are **Partial**. | @jeswr |
| **PO.3.2** | Configure tools to improve assurance & enforce; integrate into the SDLC. | Implemented & verified | clippy `-D warnings` (gate, `ci.yml#clippy`), cargo-deny `bans sources licenses` **and** `advisories` gating (`supply-chain.yml#audit`), cargo-vet gating (`supply-chain.yml#vet`), the unsafe-count ratchet (`ci.yml#unsafe-register`, `scripts/unsafe-gate.py`). **CodeQL `security-and-quality` is struck from this evidence:** the query config is still *configured* in `codeql.yml`, but the workflow is `disabled_manually` (since 2026-07-18) — it is therefore **not** enforced and **not** integrated into the SDLC, contributes nothing to `ci-summary`, and blocks nothing (GX-14). The status stands on the four tools above, each of which is verifiably gating; it does **not** extend to the static-analysis class, which has no enforcing tool at all. | @jeswr |
| **PO.3.3** | Generate, collect & safeguard artifacts that provide evidence of tool use. | Implemented & verified | CI uploads: SBOM artifact (`supply-chain.yml#sbom`), per-release SBOM+VEX (`release.yml`), Scorecard SARIF to code-scanning (`scorecard.yml`), conformance/coverage/unsafe-snapshot artifacts (`ci.yml`), fuzz reproducer artifacts (`fuzz.yml`). `ci-summary / gate` is the aggregated evidence-of-pass per PR. | @jeswr |
| **PO.4.1** | Define criteria for software-security checks; gather data to determine if met. | Implemented & verified | The conformance/coverage/perf **ratchets** (`CONTRIBUTING.md` "never lower" rule) + the unsafe-count snapshot `bench/unsafe-snapshot.json` are the quantitative criteria; `ci-summary / gate` decides met/not-met per PR. | @jeswr |
| **PO.4.2** | Implement processes/mechanisms to gather check info from the toolchain. | Implemented & verified | Each gating job writes to `$GITHUB_STEP_SUMMARY` (e.g. `ci.yml` unsafe report, coverage/conformance summaries); `ci-summary.yml#gate` polls every sibling check-run and aggregates the verdict. | @jeswr |
| **PO.5.1** | Separate & protect each development environment / its integrity. | Audit-ready | CI runs on isolated GitHub-hosted runners (per-job ephemeral VMs); `permissions: contents: read` least-privilege default in workflows; release uses OIDC (no long-lived secrets) for Sigstore attestation. There is no sparq-operated prod environment (it is a library); operator-side environment separation is the operator's responsibility. | @jeswr / operator |
| **PO.5.2** | Secure & harden development endpoints / enforce config. | Audit-ready | SHA-pinned actions (no floating tags), least-privilege `permissions:` blocks, `concurrency` guards, the distroless non-root container build (`Dockerfile`, `release.yml#container`). Developer-workstation hardening is the contributor/operator's environment, not a property of the source. | @jeswr / operator |

## PS — Protect the Software

| Task | Practice | Status | Evidence (file / test / CI job) | Owner |
|---|---|---|---|---|
| **PS.1.1** | Store all forms of code with integrity/provenance/least-privilege access control. | Implemented & verified | Git on GitHub (commit history + signed-tag-capable); `docs/branch-protection.md` (no direct pushes incl. admins, PR-only, required review on CODEOWNERS paths); `CODEOWNERS` restricts who can approve security-sensitive paths. | @jeswr |
| **PS.2.1** | Make software integrity verifiable to acquirers (signing / provenance). | Implemented & verified | `release.yml`: `actions/attest-build-provenance` (Sigstore-signed SLSA provenance over each archive + the SBOM/VEX), `SHA256SUMS` over all archives, and buildkit `provenance: mode=max` + `sbom: true` on the ghcr.io image. Verify with `gh attestation verify <file> --repo jeswr/sparq`. | @jeswr |
| **PS.3.1** | Archive & protect each software release (reproducible retrieval of the released bits). | Implemented & verified | GitHub Releases retain the packaged archives + `SHA256SUMS` + SBOM + VEX (`release.yml#release`); the published crates/npm/PyPI artifacts are immutable per their registries; `Cargo.lock` is committed (the exact resolved tree is retrievable). | @jeswr |
| **PS.3.2** | Collect, safeguard & share provenance data for all components of each release (SBOM). | Implemented & verified | Per-release CycloneDX SBOM **and VEX** (`release.yml#sbom-vex`), provenance-attested; the SBOM maps the NTIA minimum elements (supplier, component, version, PURL, dependency relationship, author, timestamp). `cargo auditable build` embeds the dependency manifest *inside* the released `sparq-cli` binary (`release.yml`, readable via `cargo audit bin`). Daily SBOM-artifact in CI (`supply-chain.yml#sbom`). | @jeswr |

## PW — Produce Well-Secured Software

| Task | Practice | Status | Evidence (file / test / CI job) | Owner |
|---|---|---|---|---|
| **PW.1.1** | Use forms of risk modelling (threat modelling, attack surface mapping) to assess. | Implemented & verified | `research/threat-model.md` (STRIDE, boundaries B1–B5, incl. B3 no-auth server boundary + B5 mmap/unsafe boundary); `research/zk-soundness-audit.md` (adversarial ZK soundness audit). | @jeswr |
| **PW.1.2** | Track & maintain security requirements / risk responses over time. | Implemented & verified | Risk responses tracked as `bd` beads (the gap-fix beads under epic `sq-toze`; the unsafe register's `NEEDS-REVIEW` discipline in `CONTRIBUTING.md`); `compliance/memsafety/unsafe-register.md` is the live per-site risk register for the B5 boundary. | @jeswr |
| **PW.1.3** | Where applicable, require approval of exceptions to security requirements. | Implemented & verified | `deny.toml` license/advisory **exceptions** each carry a reason + tracking bead; the unsafe register's `NEEDS-REVIEW`+bead rule (`CONTRIBUTING.md`) is the documented exception path; conformance-ratchet divergences require a recorded spec-justified rationale (`CONTRIBUTING.md` "never lower" rule). | @jeswr |
| **PW.2.1** | Have a qualified person review the software design against security requirements. | Audit-ready | `CODEOWNERS` mandates `@jeswr` review on the security-sensitive paths; `research/` design records (threat-model, zk-soundness-audit) are the documented design-review-of-record. A formally-credentialed independent design reviewer is an external/org act (the ZK estate explicitly carries the "external-cryptographer sign-off out of scope" caveat). | @jeswr |
| **PW.4.1** | Acquire & reuse well-secured components (vetted dependencies). | Implemented & verified | cargo-vet gating (`supply-chain.yml#vet`, `supply-chain/config.toml` imports Mozilla/Google/Bytecode-Alliance/ISRG/Embark/Zcash trusted audit sets); cargo-deny `sources` (crates.io-only) + `bans` + `licenses` gating (`supply-chain.yml#audit`, `deny.toml`); Dependabot (`.github/dependabot.yml`). | @jeswr |
| **PW.4.4** | Verify that acquired components comply with requirements (vuln/license/source checks). | Implemented & verified | cargo-deny `advisories` **GATING** at PR time (`supply-chain.yml#audit`) + the daily watchdog (`dependency-monitoring.yml`) as defence-in-depth; `licenses` allowlist + per-crate exceptions in `deny.toml`; `sources` deny-unknown-registry/git. | @jeswr |
| **PW.5.1** | Adhere to secure coding practices (the secure-coding standard). | Implemented & verified | `CONTRIBUTING.md` "Secure coding" standard (unsafe policy, input validation at the boundary, no unwrap/panic/unbounded-alloc on untrusted input, clean error/log output); enforced by clippy `-D warnings` (`ci.yml#clippy`), `#![forbid(unsafe_code)]` in the non-core crates, and the `// SAFETY:` + register rule on every `unsafe` block. | @jeswr |
| **PW.6.1** | Configure the build process to improve executable security (hardening flags, opts). | Implemented & verified | `cargo auditable build --release --locked` for the released binary (`release.yml`) embeds the dep manifest; `--locked` pins the exact tree; the distroless non-root read-only-friendly container (`Dockerfile`). | @jeswr |
| **PW.6.2** | Determine which compiler/interpreter/build-tool features improve security & use them; preserve provenance/reproducibility. | **Gap (honest statement DELIVERED; CI ratchet remaining)** | The release build is `--locked` + provenance-attested, and the honest reproducibility statement SSDF PW.6.2 asks for is now **documented** ([`../slsa/reproducible-build.md`](../slsa/reproducible-build.md)): a measured double-build of `sparq-cli` is **byte-identical apart from 22 bytes**, all from **one** non-determinism source (the `mimalloc` build-time `__DATE__`/`__TIME__` `.rodata` banner + the build-id it perturbs). The remaining open item (GX-8, bead **sq-toze.9**) is the *enforcement* step — `SOURCE_DATE_EPOCH`/feature-drop + a CI rebuild-and-diff ratchet for a byte-for-byte claim. (Memory-safety hardening — `#![forbid(unsafe_code)]` + Miri — is covered under PW.5/PW.8.) | @jeswr |
| **PW.7.1** | Determine whether code review and/or analysis should be used; configure it. | **Partial — SAST lane disabled, see GX-14** | **Code-review half: met.** PR-only flow with required `@jeswr` review on CODEOWNERS paths (`docs/branch-protection.md`, `CODEOWNERS`) — human review is mandated and enforced. **Automated-analysis half: not met.** The prior claim "SAST = CodeQL `security-and-quality` (`codeql.yml`, gating via ci-summary)" is **false as written**: `codeql.yml` is retained on `main` and its queries are still *configured*, but the workflow is `disabled_manually` (since 2026-07-18), so it never runs and never gated `ci-summary`. Determining-and-configuring a lane that is switched off does not satisfy PW.7.1's second limb. **No other tool substitutes** — clippy is a lint, not taint/crypto-misuse analysis. GX-14 (P1); posture decision **#4620**. | @jeswr |
| **PW.7.2** | Perform the code review / static analysis per org policy. | **Partial — SAST lane disabled, see GX-14** | **Code review: performed.** Every change lands by PR with required CODEOWNERS review and the aggregate `ci-summary / gate` blocking merge. **Static analysis: NOT performed.** Two prior claims in this row were **false as written** and are corrected here: (1) *"CodeQL runs on push/PR/merge_group + schedule"* — it runs on **none** of those; the triggers are retained in `codeql.yml` but GitHub schedules no run on any event while the workflow is `disabled_manually` (since 2026-07-18), so there is no `CodeQL analysis (rust)` check-run and no SARIF upload; (2) *"Code-scanning alerts are kept at zero (policy)"* — code scanning currently reports **35 open `critical` alerts** (`rust/hard-coded-cryptographic-value`). Those 35 were **triaged** under issue #4615 and found to be false positives of a single query-model defect (query matches the sink by parameter name while classifying test code by file path; both clusters sit in `#[cfg(test)]` code) — but **triaged is not covered**, and it says nothing about what an enabled scanner would find now. What survives: clippy `-D warnings` (`ci.yml#clippy`), gating and green — a lint, **not** SAST. Nothing performs taint or crypto-misuse analysis. GX-14 (P1); `ASSURANCE.md` §11; posture decision **#4620**. | @jeswr |
| **PW.8.1** | Determine whether executable code testing should be performed; configure it. | Implemented & verified | `ci.yml` (build + nextest sharded `cargo test --workspace` + doctests), W3C SPARQL/SHACL/inference conformance ratchets; the dynamic-analysis lanes are configured and scheduled (Miri, fuzz). | @jeswr |
| **PW.8.2** | Perform the executable testing & assess results (incl. dynamic/negative testing). | Implemented & verified | `cargo test --workspace` (gating); coverage-guided fuzz over parsers + mmap loader (`fuzz.yml`, libFuzzer, PR-smoke + nightly); Miri UB lane over the `sparq-core` unsafe surface (`miri.yml`, nightly); the mmap-corruption oracle for B5 sites Miri cannot reach (`compliance/memsafety/unsafe-register.md`). | @jeswr |
| **PW.9.1** | Configure software to have secure settings by default. | Audit-ready | The library/server defaults are conservative (the server documents the no-auth boundary B3 and the "front with a gateway / sparq-solid" guidance — `research/threat-model.md`, `SECURITY.md` — an explicit architectural decision, *not* a silent insecure default); the crypto scaffolds are gated behind opt-in features and carry the no-guarantee disclaimer. Operator-side deployment config (TLS, gateway, resource limits) is the operator's responsibility. | @jeswr / operator |
| **PW.9.2** | Document the secure default settings & the security implications of changing them. | Audit-ready | `SECURITY.md` (the no-auth boundary, the research-scaffold caveats), `CONTRIBUTING.md` (input-validation expectations), and the per-surface `skills/<surface>/SKILL.md` document the secure-use envelope. A consolidated operator hardening guide is an operator-facing doc the deploying org owns. | @jeswr / operator |

## RV — Respond to Vulnerabilities

| Task | Practice | Status | Evidence (file / test / CI job) | Owner |
|---|---|---|---|---|
| **RV.1.1** | Gather information from acquirers/users/sources on potential vulnerabilities. | Implemented & verified | `SECURITY.md` (private GHSA + email channels, response SLAs) + `.well-known/security.txt` (RFC 9116 machine-discoverable pointer); `.github/ISSUE_TEMPLATE/security.yml` + `config.yml` redirect public reports to the private channels. | @jeswr |
| **RV.1.2** | Review, analyze & confirm reported potential vulnerabilities (triage). | Audit-ready | `SECURITY.md` documents the triage workflow + targets (acknowledge ≤5 business days, initial assessment ≤10). Continuous operation of triage is asserted by the maintainer (best-effort volunteer project); no external attestation applies. | @jeswr |
| **RV.1.3** | Have a vulnerability-disclosure programme & process to intake reports. | Implemented & verified | GitHub Security Advisories ("Report a vulnerability") + `jesse@jeswr.org` + `.well-known/security.txt` (`Contact`/`Policy`/`Canonical`); coordinated-disclosure language in `SECURITY.md`; `CONTRIBUTING.md` redirects discoverers to the private channels. | @jeswr |
| **RV.1.4** ⚑ | Continuously monitor known-vulnerability sources for the software's components. **(sparq-local sub-task — NOT a standard SP 800-218 v1.1 task id; see footnote ⚑.)** | Implemented & verified | Daily advisory watchdog (`dependency-monitoring.yml` — cargo-deny advisories → single idempotent `security:dependency-vuln` tracking issue) **plus** PR-time `cargo deny check advisories` gating (`supply-chain.yml#audit`) + Dependabot security updates (`.github/dependabot.yml`). | @jeswr |
| **RV.2.1** | Analyze each vulnerability to gather enough information to plan its remediation. | Audit-ready | `SECURITY.md` "initial assessment" step (severity, reproduce, remediation path); the per-release **VEX** (`release.yml`) is the documented exploitability-analysis artifact for flagged advisories. Per-report analysis records are produced per-incident. | @jeswr |
| **RV.2.2** | Develop & implement remediation plans for each vulnerability. | Implemented & verified | Fixes land on `main` and ship in the next release (`SECURITY.md` "Supported versions"); remediation work is tracked as `bd` beads (the ZK-soundness remediation beads anchored on `research/zk-soundness-audit.md`; the gap-fix beads under `sq-toze`); the VEX records non-applicable advisories with justification. | @jeswr |
| **RV.3.1** | Analyze identified vulnerabilities to determine root cause. | Audit-ready | `research/zk-soundness-audit.md` is a worked root-cause analysis (the v1 verifier soundness gaps traced to the missing binding layer); the threat model + register capture systemic causes. Per-incident RCA is produced per report. | @jeswr |
| **RV.3.2** | Analyze the root cause over time to identify patterns / systemic weaknesses. | Audit-ready | The gap register (`research/production-certification-plan.md` §2 + these `compliance/*/gap-register.md` files) is the standing systemic-weakness ledger; recurring causes feed back into `CONTRIBUTING.md`/`deny.toml`/the ratchets. A formal trend-analysis cadence is an org act. | @jeswr |
| **RV.3.3** | Review the SDLC to see if the root cause could be avoided in future (feed back). | Implemented & verified | The "never lower a ratchet — fix the regression" rule (`CONTRIBUTING.md`), the unsafe register's `NEEDS-REVIEW`+bead discipline, and the gating-tool additions (advisories PR-gate, cargo-vet, unsafe ratchet) are concrete SDLC feedback from prior root causes; each `deny.toml` exception carries a bead so it is revisited. | @jeswr |
| **RV.3.4** | Review the SDLC to detect classes of the vulnerability proactively (e.g. add a check). | Implemented & verified | Live gating checks added in direct response to risk classes: the fuzz lane (parse/mmap crash classes), the Miri lane (UB classes on the unsafe surface), the unsafe-count ratchet (unsafe-creep class). Each of those three is a real, running, proactive class-detection control, and the status stands on them. **CodeQL is struck as the "SAST class" evidence:** the workflow is retained but `disabled_manually` (since 2026-07-18), so the SAST class — taint flow and crypto-misuse — is currently **detected by nothing**; no other lane covers it (GX-14; posture decision #4620). The practice is met for the classes that have a live check, and the uncovered SAST class is recorded as a gap rather than papered over. | @jeswr |

> ⚑ **`RV.1.4` is a sparq-local sub-task, not a standard SP 800-218 v1.1 task id.** The
> publication defines RV.1 with exactly three tasks — **RV.1.1 / RV.1.2 / RV.1.3** — and the
> "continuously monitor known-vulnerability sources" obligation is part of RV.1's existing text
> (gather/monitor information about potential vulnerabilities) rather than a separate numbered
> task. We retain `RV.1.4` as an explicitly-flagged sparq-local row because the underlying
> control (the daily advisory watchdog + the PR-time advisories gate + Dependabot) is a real,
> well-evidenced extension of RV.1.3's continuous-monitoring intent; it is **never** asserted as
> a standard framework control number. An assessor mapping IDs back to the publication should
> read this row as evidence *supporting RV.1.3*, sub-labelled `RV.1.4` for local traceability.

---

## Coverage summary

<!-- [OPUS-4.8] sq-ce97: the **Tasks** column is a ROW count. SP 800-218 v1.1 defines
     41 standard tasks (RV.1 has exactly 3 — RV.1.1/1.2/1.3); the 42nd row, `RV.1.4`, is the
     explicitly-flagged sparq-local sub-task (footnote ⚑). The columns below therefore split
     "standard tasks" from "+ local row" so "42" can never be misread as "42 standard SSDF
     tasks." -->
<!-- [OPUS-5] Re-footed for the CodeQL/GX-14 reconciliation: PW.7.1 + PW.7.2 moved from
     "Implemented & verified" to "Partial", so 28/13/1 becomes 26 / 2 partial / 13 / 1
     across the same 42 rows. No row was added or removed. -->

| Practice group | Standard tasks | + sparq-local rows | Rows total | Implemented & verified | Partial | Audit-ready | Gap |
|---|---|---|---|---|---|---|---|
| **PO** (Prepare the Organization) | 13 | 0 | 13 | 7 | 0 | 6 | 0 |
| **PS** (Protect the Software) | 4 | 0 | 4 | 4 | 0 | 0 | 0 |
| **PW** (Produce Well-Secured Software) | 15 | 0 | 15 | 9 | 2 | 3 | 1 |
| **RV** (Respond to Vulnerabilities) | 9 | 1 | 10 | 6 | 0 | 4 | 0 |
| **Total** | **41** | **1** | **42** | **26** | **2** | **13** | **1** |

These totals are re-derived mechanically from the rows above and cross-foot exactly. The
**Standard tasks** column counts the SP 800-218 v1.1 publication tasks (PO 13 + PS 4 + PW 15
+ RV 9 = **41**); RV.1 has exactly three publication tasks (RV.1.1/1.2/1.3), so standard RV is
**9**, not 10. The single **+ sparq-local row** is `RV.1.4` (the daily advisory watchdog —
evidence *supporting* standard task RV.1.3, **not** a separate framework task; see footnote ⚑),
which brings the **Rows total** to **42**. The status columns sum to **26 + 2 + 13 + 1 = 42**
rows, and each group's row count equals its own status split (PO 7+0+6+0=13, PS 4+0+0+0=4,
PW 9+2+3+1=15, RV 6+0+4+0=10). So: **41 standard SSDF tasks + 1 flagged local row = 42 rows;
26 implemented & verified / 2 partial / 13 audit-ready / 1 gap.**

The two **Partial** rows are **PW.7.1** and **PW.7.2** — the static-analysis practices. They
were previously scored "Implemented & verified" on CodeQL; `codeql.yml` has been
`disabled_manually` since **2026-07-18**, so no SAST runs on any event, and **no other control
performs taint or crypto-misuse analysis**. Both rows keep their genuine code-review evidence
and lose their SAST evidence. Anchor: cross-cutting gap **GX-14** (P1) in
[`../gap-register.md`](../gap-register.md) and [`gap-register.md`](./gap-register.md) SSDF-G2;
narrative in `ASSURANCE.md` §11; open posture decision **#4620**.

The single row carrying the **Gap** status is **PW.6.2 reproducible-build** (GX-8 / bead
sq-toze.9) — and even that is now *characterised*: the honest non-repro statement is documented
([`../slsa/reproducible-build.md`](../slsa/reproducible-build.md)), with only the CI
rebuild-and-diff *enforcement* outstanding. The **second** open technical shortfall is the
disabled SAST lane (**GX-14**, P1), scored as the two **Partial** rows above rather than as a
third Gap row because the code-review limb of PW.7.1/PW.7.2 is genuinely met; it is a real
residual, not bookkeeping, and it is registered in [`gap-register.md`](./gap-register.md) as
SSDF-G2. The **audit-ready** rows are the practices that are documented + automated but
whose *continuous operation* / *formal attestation* is an organizational act SSDF leaves to
the producing org — for a single-maintainer volunteer project these are asserted with the
gate/doc cited, not externally certified. No row overclaims, and none presents the
`sparq-zk*`/`sparq-mpc` estate as a met cryptographic control — its no-production-guarantee
status is a correctly-disclosed limitation. [OPUS-4.8] The v1 ZK verifier was originally found
unsound (`research/zk-soundness-audit.md`, kept on record), then `sq-1s2` landed the binding
layer and an internal re-audit (`research/zk-verifier-reaudit.md`, `sq-gbp4`) found the prior
findings closed → "sound as landed for the assumed threat model" — but **internal/single-model
only, external sign-off PENDING (`sq-qhy4`), no production guarantee** (`SECURITY.md`).
