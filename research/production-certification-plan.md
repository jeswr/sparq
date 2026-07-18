<!-- [OPUS-4.8] Deep-research plan for production hardening + security certification (epic sq-toze). -->
# sparq production-certification plan

Deep-research plan for hardening `sparq` for production use **as a dependency in high-risk,
high-security settings**, modeled on the proven `jeswr/prod-solid-server` (PSS) compliance process
(parallel engineer↔auditor loop per framework → consolidation, with an honesty contract). This
document is the spine the lead reviews before launching the engineer/auditor waves. It defines:
**(1)** the target framework set + rationale (and what we cut from PSS's set, with reasons); **(2)**
the skills to install; **(3)** the grounded, prioritized gap register (what sparq does *not* yet have).

> **NON-CANONICAL timing.** This session runs on an EC2 work box; no measured timings are baked here.

## 0 — What sparq is, and why its framework set differs from PSS's

PSS is a **Solid (LDP) HTTP API + storage server** — a deployed *service* that authenticates users
(Solid-OIDC/DPoP), authorises with WAC, and stores *personal data in pods* on S3 + a QLever index.
Its 10-framework set (asvs, cis, sbom, iso27001, iso27701, iso27017-18, soc2, gdpr, cyberess, cdmc) is
the right set for a **personal-data SaaS the operator runs**.

sparq is different in three load-bearing ways, and the framework set must follow:

1. **It is primarily a Rust *library* + crypto estate, consumed as a dependency** — not (only) a
   service. The dominant risk surface is therefore **supply-chain integrity, build provenance, and
   memory safety**, not user-session auth. This is why we **add** the Rust/library/crypto-critical
   frameworks (SSDF, SLSA, OpenSSF, CRA, memory-safety attestation, a crypto review).
2. **`sparq-server` has, by documented design, no authentication or per-user authz** (threat-model
   boundary **B3**: "front with a gateway / sparq-solid"). So ASVS still applies (input validation,
   DoS limits, error/log hygiene, config) but the auth/session/access-control families are largely
   **the operator's responsibility**, mapped as an explicit architectural decision, not a silent gap.
3. **sparq does not itself collect personal data** — it is a data *engine*; the deploying operator is
   the GDPR controller for whatever RDF they load. The personal-data frameworks (GDPR, ISO 27701,
   SOC 2 Privacy) therefore apply *narrowly* (what the binary can touch: loaded datasets, query logs,
   the WASM client surface) + carry the **ZK/MPC privacy story** (what sparq *would* offer if the
   crypto were sound — and it is documented as **not yet sound**).

### What we KEEP from PSS (adapted)

| PSS framework | Keep? | Adaptation for sparq |
|---|---|---|
| OWASP ASVS L2 | **Keep** | Re-scoped to `sparq-server`/`sparq-serve` as an HTTP *query* API: SPARQL/RDF input validation, DoS/limits, error/log hygiene, config, the documented no-auth boundary B3 as an explicit decision. Auth/session families → operator. |
| CIS Docker | **Keep** | The distroless/non-root/SHA-pinned `Dockerfile`; add a Trivy/Dockle CI scan. (CIS *Node* benchmark → dropped; sparq has no Node runtime — the JS surface is the WASM port, covered under memory-safety + supply-chain.) |
| SBOM + supply-chain | **Keep** | CycloneDX per-release + VEX; cargo-deny advisories PR-gate; cargo-auditable/cargo-vet. The strongest-fit framework — this is a dependency. |
| ISO/IEC 27001 Annex A | **Keep (audit-ready)** | Narrow applicable set for a library/server; explicit operator-vs-sparq responsibility split. |
| ISO/IEC 27701 (PIMS) | **Keep, FOLD into "privacy"** | Folded with GDPR + SOC2-Privacy into one `privacy` worktree (narrow: data a library can touch + the ZK/MPC story). |
| ISO/IEC 27017 + 27018 (cloud) | **CUT** (mostly) | These are *cloud-service-provider* controls (S3 buckets, tenant isolation, PII in public cloud). sparq is not a cloud service; it has no S3/object-store/multi-tenant surface of its own. The thin residual (operator deploys it in cloud) is captured as operator responsibility in ISO 27001 + the data-flow doc. Cut to avoid a hollow control table. |
| SOC 2 Type II (TSC) | **Keep, FOLD into "privacy"** | Only the Privacy + Confidentiality criteria apply narrowly; Security TSC is covered by ASVS/SSDF/memory-safety; Availability is a library SLA the operator owns. Folded into `privacy`, not a standalone worktree. |
| UK GDPR / GDPR | **Keep, FOLD into "privacy"** | Controller = operator; sparq = technical means. Honest data-flow + DPIA of what the binary touches + the ZK/MPC privacy story. |
| UK Cyber Essentials | **CUT** | The five controls (firewalls, secure config, access control, malware, patch management) are *organisational/deployment* controls for the org running infrastructure — not properties of a Rust library. They map to the operator's environment, not sparq's source. Replaced by **EU CRA**, which is the regime that actually binds a "product with digital elements" placed on the market. |
| CDMC scoring | **Keep** | sparq is a data engine (RDF ingest → Dict/index → results); score the 6 components/14 capabilities, with cataloguing/classification largely operator-owned. |

### What we ADD (Rust / library / crypto-critical)

| New framework | Why it applies to sparq |
|---|---|
| **NIST SSDF (SP 800-218)** | The canonical *secure-software-development* framework for a code artifact shipped as a dependency. Maps PO/PS/PW/RV practice groups to sparq's existing gate stack, threat model, fuzz/Miri, vuln disclosure, dependency policy — mostly a mapping exercise, high coverage already. |
| **SLSA build provenance** | A dependency's supply-chain trust hinges on *verifiable build provenance*. sparq already emits `attest-build-provenance` + buildkit `provenance: mode=max` on release; declare the honest level (≈L2 on GitHub-hosted runners) + the gap to higher (reproducibility, cargo-auditable). |
| **OpenSSF Scorecard + Best-Practices (CII) Badge** | The de-facto open-source-security posture signal consumers check. Scorecard is wired (`scorecard.yml`, `publish_results`); add the Best-Practices self-certification + raise the score. |
| **EU Cyber Resilience Act (CRA)** | sparq, distributed on crates.io/npm/PyPI/ghcr, is a "product with digital elements." CRA's Annex I essential requirements + vuln-handling obligations (coordinated disclosure, SBOM, security updates, support period) are the *binding regulatory* regime — replaces Cyber Essentials as the right "you must do this to ship in the EU" anchor. |
| **Memory-safety attestation** | sparq's headline safety claim. Attest the `#![forbid(unsafe_code)]` posture (31 of the 34 lib crates as of 2026-06-19 — verify live with `grep -lE 'forbid\(unsafe_code\)' $(git ls-files 'crates/*/src/lib.rs') \| wc -l`; the three lib crates that do NOT forbid are `sparq-core`, `sparq-vectors`, `sparq-zk-compose`), enumerate + justify the concentrated `sparq-core` unsafe (mmap/dict-spill/SIMD = threat-model boundary **B5**), and the Miri + fuzz + oracle coverage; promote cargo-geiger to a gating unsafe-count ratchet. |
| **Cryptographic review (ZK/MPC)** | sparq ships `sparq-zk`/`sparq-zk-compose`/`sparq-mpc`. A documented crypto review of the soundness/privacy claims (anchored on `research/zk-soundness-audit.md`), constant-time + FIPS considerations, and the explicit "research scaffold, NO guarantee" statement. External-cryptographer sign-off is out of agent scope. |

### Final target set (12 worktrees)

`asvs`, `cis`, `sbom`, `ssdf`, `slsa`, `openssf`, `memsafety`, `iso27001`, `cra`, `privacy`
(GDPR+27701+SOC2-Privacy+data-flow+dpia), `cryptoreview`, `cdmc`. **Cut from PSS:** iso27017-18 (no
cloud-service surface), cyberess (org/deployment controls, superseded by CRA), and CIS-Node (no Node
runtime). This is a defensible ~12-framework set right-sized to a Rust security-critical data-engine
dependency.

## 1 — Skills to install

PSS-style compliance work is largely *evidence-gathering + adversarial review*; the heavy crypto-review
framework leans on sparq's existing ZK/MPC skills. Plan:

| Skill | Source | Where it applies |
|---|---|---|
| **test-driven-development** | `jeswr/solid-ai-coding` (install) | Every framework that closes a *technical* gap (memsafety, sbom, cis, asvs) — gap-fixes land with a regression test first. |
| **secure-coding / threat-modelling** (if present in PSS `.agents/skills`) | `jeswr/prod-solid-server` `.agents/skills/` | ASVS, SSDF, CRA, threat-model extension. Install whichever PSS shipped; if none, author a thin sparq one. |
| **(NEW) `rust-supply-chain-attestation`** | author in sparq `.agents/skills/` | SBOM (CycloneDX + VEX + NTIA elements), SLSA level evidence, cargo-deny/vet/auditable. Codifies the cargo-specific attestation recipe so the sbom + slsa engineers don't reinvent it. |
| **(NEW) `unsafe-rust-attestation`** | author in sparq `.agents/skills/` | memsafety framework: how to enumerate `unsafe`, write per-site justifications, wire the cargo-geiger ratchet, and what Miri can/can't reach (the mmap sites). |
| **mpc-protocols**, **noir-optimisation**, **noir-circuit-patterns**, **verifiable-credentials-zk**, **sparql-formal-semantics** | already in repo | `cryptoreview` framework — these are the existing ZK/MPC reference skills the crypto-review engineer reads to assess soundness/privacy claims honestly. |
| **code-review / security-review** | built-in | Auditor agents may invoke for an extra adversarial pass on `crates/` changes. |

The two NEW sparq-specific skills are small and high-leverage (they prevent the supply-chain and
memory-safety engineers from re-deriving the cargo/Rust-specific recipe). Authoring them is its own
bead (`skills-install`).

## 2 — Gap register (grounded, prioritized)

Status of the existing posture (from a code/CI audit of this branch) is summarized so we **do not
re-propose existing controls**. Legend: **P0** blocks a perfect score on a high-value framework; **P1**
needed for a perfect score; **P2** raises maturity / nice-to-have.

### Already done — DO NOT re-propose (cite as evidence)
clippy `-D warnings` (gating), CodeQL SAST (gating when this was written; advisory-at-merge with
retroactive daily alert triage since 2026-07-17 — `docs/branch-protection.md` §CodeQL is advisory),
OpenSSF Scorecard (`scorecard.yml`, published),
cargo-deny bans/sources/licenses (gating), CycloneDX SBOM *in CI as artifact*, cargo-fuzz (PR smoke +
nightly), Miri lane (nightly, `sparq-core`), SLSA `attest-build-provenance` + buildkit `provenance:
mode=max` + SHA256SUMS on release, distroless non-root SHA-pinned `Dockerfile` + docker-smoke gate,
SHA-pinned actions (policy), ci-summary branch-protection aggregator, `SECURITY.md` (GHSA + email,
response targets, the ZK-not-sound + MPC-deferred caveats), `research/threat-model.md` (STRIDE,
boundaries B1–B5), `#![forbid(unsafe_code)]` in 31 crates, daily dependency-monitoring advisory
watchdog, Dependabot (4 ecosystems), CODEOWNERS, CONTRIBUTING.md (with disclosure redirect).

### Cross-cutting gaps

| ID | Gap | Sev | Framework(s) | Bead |
|---|---|---|---|---|
| GX-1 | **cargo-deny advisories PR-gate is degraded** (`continue-on-error`, can't parse CVSS-4.0 advisories — sq-q8de). The *real* gate is the daily watchdog; PR-time vuln gating is absent. | P0 | sbom, ssdf, cra | gap-fix |
| GX-2 | **No checked-in / per-release SBOM with VEX.** SBOM exists only as a CI artifact; no VEX (Vulnerability Exploitability eXchange) to justify non-applicable advisories. CRA/SBOM want a published, signed, per-release SBOM. | P0 | sbom, cra, slsa | gap-fix |
| GX-3 | **No `.well-known/security.txt` (RFC 9116).** SECURITY.md exists but the machine-discoverable disclosure pointer is absent. | P1 | cra, openssf, asvs | gap-fix |
| GX-4 | **No OpenSSF Best-Practices (bestpractices.dev/CII) badge / self-certification.** Scorecard badge is eligible but the Best-Practices questionnaire isn't filled. | P1 | openssf | gap-fix |
| GX-5 | **Unsafe surface has no per-site justification register, and cargo-geiger is informational only** (no gating ratchet). The B5 mmap sites can't run under Miri — coverage is via oracle + fuzz but isn't *attested* in one place. | P0 | memsafety, ssdf | gap-fix |
| GX-6 | **No CONTRIBUTING secure-coding section** (unsafe-review checklist, input-validation guidance). SSDF PW + ASVS V1 want a documented secure-coding standard. | P1 | ssdf, asvs, iso27001 | gap-fix |
| GX-7 | **No cargo-auditable / cargo-vet.** Binaries don't embed their dependency manifest; no per-dependency audit attestations. Raises SLSA/supply-chain assurance. | P2 | slsa, sbom, ssdf | gap-fix |
| GX-8 | **No reproducible-build evidence.** SLSA higher levels + CRA integrity want a documented reproducibility claim (or an honest "not reproducible because…"). | P2 | slsa, cra | gap-fix |

### Per-framework gaps (beyond cross-cutting)

- **asvs** (P1): re-scope V1–V14 to an HTTP query API; document the B3 no-auth boundary as an explicit
  decision (not a gap); add an ASVS-control test job for input-validation/DoS-limit controls. Verify
  error/log hygiene (no SPARQL/RDF content or internal paths leaked in error responses/logs).
- **cis** (P1): add a Trivy/Dockle container-hardening scan job to CI (currently no automated CIS-Docker
  scan — only the smoke test). Add a `HEALTHCHECK`-equivalent attestation.
- **sbom** (P0): GX-1, GX-2; map the NTIA minimum elements + supplier enrichment; per-release publish.
- **ssdf** (P1): write the SP 800-218 PO/PS/PW/RV → sparq-control mapping (mostly existing controls);
  GX-5, GX-6 are the real holes.
- **slsa** (P1): declare the honest level + threat-coverage; GX-7, GX-8 to raise it.
- **openssf** (P1): fill the Best-Practices questionnaire (GX-4); raise the Scorecard score (signed
  releases, GX-3, branch-protection evidence already strong).
- **memsafety** (P0): the unsafe-justification register + cargo-geiger ratchet (GX-5); attest the
  Miri/fuzz/oracle coverage matrix over every `sparq-core` unsafe site (the B5 boundary).
- **iso27001** (audit-ready): map the narrow applicable Annex A set; explicit operator-responsibility
  split. Mostly documentation; no large code gap.
- **cra** (P1): map Annex I essential requirements + the vuln-handling obligations; GX-1/2/3 are the
  binding holes (SBOM + coordinated disclosure machine-discoverability + security-update channel).
- **privacy** (audit-ready): write `compliance/data-flow.md` + `dpia.md` honestly scoped to what the
  binary touches (loaded RDF, query logs, WASM client) + the ZK/MPC privacy story (flagged not-sound).
  Honest controller(operator)/processor(sparq-as-tool) split. No large code gap; the risk is boilerplate.
- **cryptoreview** (P0 honesty): the documented review must *preserve and extend* the
  `research/zk-soundness-audit.md` verdict — v1 verifier NOT sound, MPC NO guarantee — and never let a
  control table or CDMC score launder a scaffold into a "verified" guarantee. The gap is producing the
  consolidated review doc + constant-time/FIPS notes; the *crypto itself* is tracked by the existing ZK
  remediation beads, not this epic.
- **cdmc** (P1): score the 6 components; cataloguing/classification of loaded datasets is largely
  operator-owned — say so; recommendations target lineage/retention over the Dict/index + the
  HDT/compressed-archive ingest surface.

### Honest top-level posture statement

sparq's *technical* security posture is already strong (the "already done" list is long and CI-gating).
The certification gaps are concentrated in **(a) supply-chain attestation completeness** (advisories
PR-gate, per-release SBOM+VEX, cargo-auditable/vet — GX-1/2/7), **(b) memory-safety attestation
formality** (the unsafe register + geiger ratchet — GX-5), and **(c) the documentation/evidence packs**
the audit-ready frameworks need (privacy data-flow/DPIA, ISO 27001 mapping, CRA mapping, the crypto
review). The ZK/MPC estate must be presented exactly as the existing audit found it — **not sound** —
and certification work must never contradict that. There is **no** finding that sparq currently
*overclaims* security; the honesty contract is already lived in SECURITY.md, and this plan preserves it.
