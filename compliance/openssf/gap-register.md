<!-- [OPUS-4.8] OpenSSF gap register (epic sq-toze, bead sq-toze.15). Authored while Fable
     unavailable — re-review when Fable returns. Every open gap: severity, remediation,
     target, bd bead. No gap is papered over. -->

# OpenSSF — gap register

Open gaps for the OpenSSF slice (Scorecard + Best-Practices badge). Severity follows the
plan's legend: **P1** needed for a perfect score on a high-value framework; **P2/P3** raise
maturity. Each row carries its tracking `bd` bead under epic `sq-toze`.

| ID | Gap | Sev | Status | Remediation | Target | Bead |
|---|---|---|---|---|---|---|
| **GX-4** | **OpenSSF Best-Practices (CII) badge not filed.** The questionnaire is fully drafted and evidence-grounded (`evidence.md` §Badge) and sparq meets the passing bar, but no entry exists on bestpractices.dev — so Scorecard's `CII-Best-Practices` check scores 0 and there is no badge to display. | P1 | OPEN — external (maintainer) | Maintainer creates the bestpractices.dev project entry and transcribes the drafted answers in `evidence.md` §Badge. This is a human-owned external filing step — the agent cannot file it. Once filed, link the badge in `README.md` and re-run Scorecard. | maintainer to file | **sq-toze.5** (blocks **sq-toze.15**) |
| **GX-OSSF-2** | **Registry publishes are not signing-attested.** GitHub release assets carry Sigstore SLSA provenance (Scorecard `Signed-Releases` satisfied), but crates.io / npm / PyPI publishes are **manual** and unattested at the registry. npm supports `--provenance`; PyPI supports attestations/Trusted Publishing; crates.io has no first-party artifact-signature scheme. | P3 | OPEN | Add `--provenance` to the npm publish; wire PyPI attestation / Trusted Publishing; document the crates.io limitation honestly (no laundering it as "signed"). | next release-tooling pass | **sq-jgt3** |
| **GX-OSSF-3** | **Scorecard Code-Review / Branch-Protection score is depressed by the solo-maintainer reality.** The ruleset (`docs/branch-protection.md`) requires ≥1 CODEOWNERS review + Copilot auto-review, but Scorecard infers review from *merged-PR history* and discounts self-approval, so a solo-maintained repo may not score 10 even with the rule enabled. The live ruleset is also out-of-repo (cannot be asserted from a file). | P3 | OPEN — partly external | Ensure PRs carry an independent (human or trusted-bot) review recorded in history, or document the solo-maintainer reality; and confirm the **live** GitHub ruleset matches `docs/branch-protection.md` so Branch-Protection scores fully. | maintainer to verify live ruleset | **sq-sto1** |

## Cross-framework gaps that also touch OpenSSF (owned elsewhere — referenced, not duplicated)

These are tracked in their owning slice's register; they surface in OpenSSF scoring but the
remediation lives in the supply-chain / slsa frameworks.

| ID | Gap (as it touches OpenSSF) | Owning slice | Bead |
|---|---|---|---|
| **GX-8** | **No reproducible-build attestation.** Touches Badge `build_reproducible` (answered **Unmet** honestly in `evidence.md`; it is SUGGESTED-not-required at the passing level, so it does not block the bronze badge). | slsa / cra | (slsa gap-register; gap-fix under sq-toze) |

## Closed gaps (cite as evidence, do not re-open)

| ID | Gap | Closed by | Evidence |
|---|---|---|---|
| **GX-3** | No `.well-known/security.txt` (RFC 9116) machine-discoverable disclosure pointer. | **CLOSED** | [`.well-known/security.txt`](../../.well-known/security.txt) (bead **sq-toze.4**, closed). Strengthens Scorecard `Security-Policy` + Badge `vulnerability_report_private`. |
| **GX-6** | No documented secure-coding standard. | **CLOSED** | [`CONTRIBUTING.md`](../../CONTRIBUTING.md) "Secure coding" (bead **sq-toze.7**). Strengthens Badge `contribution_requirements`. |
| **GX-5** | No per-site unsafe justification register. | **CLOSED** | [`compliance/memsafety/unsafe-register.md`](../memsafety/unsafe-register.md) (bead **sq-toze.6**). Strengthens Badge `dynamic_analysis_unsafe`. |
| **GX-1** | PR-time **advisory** gating was degraded (cargo-deny CVSS-4.0 parse blocker, sq-q8de). | **CLOSED** | [`supply-chain.yml`](../../.github/workflows/supply-chain.yml) `audit` job now runs `cargo deny check advisories` as a **real PR/push/merge_group gate** (no `continue-on-error`) over a fail-closed [`deny.toml`](../../deny.toml); closed by **#210** (bead **sq-toze.2**, closed; the CVSS-4.0 blocker sq-q8de is resolved). The daily watchdog ([`dependency-monitoring.yml`](../../.github/workflows/dependency-monitoring.yml)) is now defence-in-depth. Touches Scorecard `Vulnerabilities` / Badge `vulnerabilities_*`. |

## Honest bottom line

The OpenSSF posture is **strong**: every repository-level Scorecard check that can be
asserted from the codebase/CI is **implemented & verified**, and the Best-Practices badge is
**answer-ready at the passing level** (with silver/gold reach on warnings, signed releases,
and fuzz+Miri analysis). The **only blocking gap to a displayed badge / full Scorecard
posture is the external badge filing (GX-4)** — a human-owned step the agent cannot perform.
The two remaining gaps (GX-OSSF-2 registry-publish signing, GX-OSSF-3 solo-maintainer review
score) are maturity nudges, not posture failures. No control is overclaimed; the advisory
PR-gate is **un-degraded** (GX-1 closed by #210 / sq-toze.2 — now two gating `cargo deny`
steps over a fail-closed `deny.toml`), and the unattested registry publishes are recorded,
not laundered.
