<!-- [OPUS-4.8] SBOM-framework adversarial audit — epic sq-toze, framework `sbom`, PR #229
     (branch cert-sbom). Independent verification by the SBOM auditor. NON-CANONICAL timing. -->
# SBOM + supply-chain — audit findings (round 1)

> 🤖 SPARQ agent. Adversarial audit of the SBOM certification slice on branch `cert-sbom`
> (draft PR #229), epic `sq-toze`. Independence: no source or `compliance/sbom/` file was
> edited; only this file under `compliance/audit/` was written.

**VERDICT: FINDINGS: 6** (1 medium, 5 low) — no critical/high. The SBOM *posture* is genuinely
strong and the *conclusions* (which controls are met, which are gaps) are essentially right. The
findings are **accuracy / internal-consistency defects in the evidence pack itself** — a recorded
probe value that is factually false, headline counts that don't match the spine, and a few rows
whose "verified/published" framing outruns the evidence on a repo that has never cut a release.
None is an overclaimed *control*; all are fixable in the docs without code changes.

---

## What I independently verified (and it held up)

I re-ran the central probe myself: `cargo cyclonedx --all --format json` with the cited tool
version (cargo-cyclonedx **0.5.9**, the recorded version) against this branch, inspecting
`crates/sparq-server/sparq-server.cdx.json` (then deleted the artifacts, as the script does).
Corroborated:

- `bomFormat=CycloneDX`, **`specVersion=1.3`**, `serialNumber` present, `metadata.timestamp` present
  (`2026-06-15T23:43:04Z`), `metadata.tools = cargo-cyclonedx/0.5.9` — matches CDX-1, N6, N7.
- **166 components**; `name` 166/166, `version` 166/166, `purl` (`pkg:cargo/...`) 166/166,
  `licenses` 166/166 (real SPDX, e.g. `{"expression":"MIT"}`), `externalReferences` 166/166 (real
  `vcs`/`website` URLs) — N2/N3/N4, CDX-2/CDX-4 substantiated.
- `dependencies[]` = **167 nodes, 111 with non-empty `dependsOn`** — N5 substantiated (real graph,
  not a stub).
- `components[].supplier` = **0/166**, `metadata.supplier`/`metadata.authors` **absent** — the N1/GS-1
  partial-gap is real.
- VEX↔deny sync: `deny.toml [advisories].ignore` and `vex.cdx.json vulnerabilities[].id` are **both
  exactly `{RUSTSEC-2024-0436, RUSTSEC-2025-0134}`**; both crates (`paste`, `rustls-pemfile`) are
  **actually in `Cargo.lock`** (VEX not stale) — VEX-1/2/3/4 substantiated.
- `Dockerfile:L70` `cargo auditable build` and `release.yml#package` `cargo auditable build` present
  (DEP-5); cargo-vet `--locked` GATING job + `supply-chain/config.toml` imports
  (Mozilla/Google/Bytecode-Alliance/ISRG/Embark/Zcash) + exemptions present (DEP-4);
  `.well-known/security.txt` present (GX-3 closed); advisories gate is **GATING** not
  `continue-on-error` (GX-1 closed) — all verified by file inspection.
- All five gap beads exist: **sq-toze.26 (GS-1), .27 (GS-3), .28 (GS-4), .29 (GS-5), .9 (GS-2/GX-8)**
  — all `open`; the closed-out beads (.2/.3/.4/.8) are all `closed`.

So the *substance* is sound. The findings below are about the evidence pack's internal accuracy.

---

## Findings

### F-1 (medium) — Recorded NTIA probe is factually wrong: `author` is present on 144/166, not "absent on all 166"
**Control:** N1 / GS-1; `evidence.md §3`; `controls/sbom.md` N1 row.
**What I checked:** my own `cargo cyclonedx --all` run, then
`python3` over `components[].author`. **Result: `author` is populated on 144/166 components**
(e.g. `spargebra → "Tpt <thomas@pellissier-tanon.fr>"`, `aho-corasick → "Andrew Gallant
<jamslam@gmail.com>"`). Only `supplier` is 0/166.
**Why it fails:** `evidence.md §3` records the row `components[].supplier / .author | **absent on all
166** | GS-1`, and the N1 control row asserts "probe ... shows `supplier/author` absent on all 166
components." Both are **false**: the CycloneDX `author` field — the 1.3-era carrier of
component-originator identity, the closest field to NTIA "Supplier Name" — is present on the large
majority of components. An evidence pack whose *recorded probe value contradicts the actual tool
output* is not trustworthy as evidence, even though the N1 *conclusion* (no fully-populated
per-component supplier; `supplier` proper empty) survives. Misrecording the probe in the direction
that makes the gap look worse is still an evidence-integrity defect an external auditor would catch
by re-running the one command the pack tells them to run.
**Remediation:** correct `evidence.md §3` and the N1 row to state the true observation: `supplier`
0/166 **and `author` 144/166** (22 first-party/workspace crates such as `sparq-core`/`sparq-engine`
and a few deps lack it). Re-frame N1 as "the NTIA *Supplier Name* slot (`supplier`/`publisher`) is
unpopulated; component-*author* identity IS present on 144/166 via the crate `authors` metadata" —
which is both more honest and slightly *strengthens* the NTIA story. Tracked under the existing
**sq-toze.26**; no new bead required (doc-accuracy fix).

### F-2 (low) — Headline counts contradict the control spine: "22 implemented & verified / 32 controls" vs the spine's 28/31
**Control:** `controls.md`, `README.md`, `controls/sbom.md §Summary counts`.
**What I checked:** classified every control row in `controls/sbom.md`. **The spine has 31 rows: 28
"Implemented & verified", 3 "Gap" (N1, CDX-3, INT-3).** The "Summary counts" line even *enumerates*
the verified set — "N2–N7, CDX-1/2/4, all VEX, all SIG, all PUB, DEP-1..6, INT-1/2" — which sums to
6+3+4+3+4+6+2 = **28**, yet labels it "**22** controls". `controls.md` and `README.md` repeat "22
implemented & verified" and "32 controls / 5 gaps".
**Why it fails:** the authoritative artifact (the spine) says 28/3 of 31; the headline says 22/5 of
32. The numbers don't reconcile in either direction (`22 ≠ 28`; `31 ≠ 32`; the spine marks only 3
gaps, not 5). The "5 gaps" framing folds in GS-3 (which has **no control row** — it's a scope item)
and GS-5 (whose mapped control **VEX-3 is marked "Implemented & verified"** with drift-automation as
an inline P2 sub-gap). An auditor reading the headline and then counting the spine finds a
mismatch.
**Remediation:** make the headline match the spine: state "**31 control rows: 28 implemented &
verified, 3 gaps (N1, CDX-3, INT-3)**", and separately list "5 tracked open gap *items* (GS-1..5),
two of which (GS-3 scope, GS-5 automation) sit behind controls that are otherwise met." Fix the "22"
arithmetic everywhere it appears (`controls.md`, `README.md`, spine summary). Doc-only.

### F-3 (low) — Internal contradiction on how many NTIA elements are weakened (1 vs 2)
**Control:** N1/N6; `README.md` honesty paragraph vs `controls/sbom.md §NTIA verdict` + `evidence.md`.
**What I checked:** `README.md:47` says GS-1 leaves "supplier/author NTIA fields ... weakening
**two** of the seven NTIA elements." `controls/sbom.md:31` says "**5 of 7** elements fully met; N1 ...
the **one** genuine NTIA-completeness gap"; `evidence.md:61` agrees ("emits 5/7 ... N6 at tool-author
granularity").
**Why it fails:** the README claims two NTIA elements are weakened; the spine and evidence claim one.
They cannot both be the authoritative count. (Given F-1, the truer reading is closer to "one element
— Supplier Name — partial; Author/N6 met.")
**Remediation:** reconcile to a single number consistent with the corrected F-1 probe (recommend:
N1 Supplier Name = partial; all others met → "one weakened element"). Doc-only.

### F-4 (medium-low) — Release-gated controls marked "Implemented & verified" have never actually executed (zero releases)
**Control:** SIG-1, SIG-2, SIG-3, PUB-1, PUB-2, VEX-4, DEP-5 (the release/`v*`-triggered controls).
**What I checked:** `git tag -l 'v*'` → **none**; `gh release list` → **empty**; the repo has **no
attestations** (`gh api .../attestations/...` 404). `release.yml` triggers only on `push: tags: v*`.
**Why it fails:** the SBOM-publication, per-release-VEX, SHA256SUMS, image-SBOM, cargo-auditable and
SLSA-attestation controls are **wired and config-correct**, but their *effect* has never been
produced — no SBOM/VEX has ever been attached to a release, nothing has ever been Sigstore-attested.
Several rows assert this in the present/perfect tense ("per-release VEX **published**", SBOM+VEX
"**attached to every Release**", "**SLSA-attested**"). For a config control, "implemented & verified"
is defensible *if* it means "the workflow is verified to be correctly wired"; it is **not** the same
as "verified to produce the artifact", which is what the prose implies. On a repo with zero releases
the stronger reading is unsupported.
**Why only medium-low:** the wiring genuinely is correct and reviewable; this is a *framing* gap, not
a missing control. But an external auditor distinguishes "control designed" from "control operating
effectively", and these rows currently read as the latter.
**Remediation:** add one honest qualifier to the release-gated rows (or a single note at the top of
section D/E): "verified at the **configuration** level (workflow wiring); **operating-effectiveness
evidence pending the first `v*` release** — no release has been cut yet." Alternatively cut a test
tag to produce a real attested SBOM and cite its `gh attestation verify` output. Doc-only unless a
test release is chosen.

### F-5 (low) — Gap register says bead sq-toze.8 is "OPEN (close-out)" but it is already `closed`
**Control:** DEP-4, DEP-5; `gap-register.md §Notable NON-gaps` (GX-7 row); `controls/sbom.md` DEP-4/5.
**What I checked:** `.beads/issues.jsonl` → **sq-toze.8 status = `closed`** ("[cert][gap GX-7]
cargo-auditable + cargo-vet"). The gap register says the bead "remains **open** only as an
administrative close-out" and *recommends the auditor confirm it be closed*; DEP-4/DEP-5 say "bead
sq-toze.8 OPEN".
**Why it fails:** stale bead-state claim. The control itself is wired and verified (not in dispute),
but the register's stated bead status is wrong, and its recommendation ("close sq-toze.8") is moot.
**Remediation:** update the three references to reflect sq-toze.8 = `closed`; drop the
"recommend close-out" note. Doc-only.

### F-6 (low) — SBOM root identity leaks the absolute build-machine path; no probe note that the published root `bom-ref` is a local-path ref
**Control:** INT-1/INT-2 (SBOM-binds-to-shipped-artifact) integrity hygiene; `evidence.md §3`.
**What I checked:** in the generated SBOM, the root `metadata.component.bom-ref` =
`path+file:///home/ubuntu/sparq/.claude/worktrees/.../crates/sparq-server#0.1.0` and root `purl` =
`pkg:cargo/sparq-server@0.1.0?download_url=file://.`; dependency-graph node refs likewise carry
`path+file:///<abs-path>` for workspace members. `release.yml#sbom` runs the **same**
`cargo cyclonedx --all`, so the published SBOM carries the **CI runner's absolute path** in the root
and workspace-member refs.
**Why it fails:** not a control failure, but (a) it leaks the build host's directory layout into a
published artifact (a minor information-hygiene issue), and (b) the evidence pack never records that
the root/component refs are local-path-based rather than registry purls, which an integrator
verifying SBOM→artifact binding will notice. Worth disclosing so it isn't mistaken for tampering.
**Remediation:** add one line to `evidence.md §3` noting that workspace-member refs are
`path+file://` local-path refs (root `purl` `download_url=file://.`) — inherent to cargo-cyclonedx
0.5.9 resolving path members — and that this is expected; optionally strip/normalize the absolute
path in `scripts/gen-sbom-vex.sh` post-processing. I filed a bead for the optional code hardening
(see below). Doc note is sufficient to clear the finding.

---

## Honest answers to the three report questions

1. **Is the NTIA "minimum elements" completeness claim honest given the supplier-name gap?**
   *Mostly yes, but with the F-1 accuracy defect.* The engineer does **not** claim full NTIA
   completeness — N1 is correctly marked a gap, the policy template (§2.2) flags the "known
   exception," and the README §honesty calls it out. The dishonesty risk is **inverted**: the
   recorded probe *understates* what the SBOM carries by claiming `author` is "absent on all 166"
   when it is present on 144/166 (F-1), and the count of weakened elements is internally
   contradictory (F-3, 1 vs 2). So the *bottom line* ("not fully NTIA-complete; one
   supplier-related element partial") is honest; the *supporting numbers* are wrong and must be
   corrected before sign-off.

2. **Is the gap register complete (real gap set, each beaded, nothing overclaimed)?**
   *Substantially yes.* GS-1 (N1 supplier), GS-2/GX-8 (reproducible build), GS-3 (JS SBOM), GS-4
   (spec 1.3-vs-1.5), GS-5 (VEX↔deny drift automation) are the real residual set, each has an
   existing `open` bead (sq-toze.26/.9/.27/.28/.29), and no *control* is overclaimed — every
   "verified" row cites a real file/job, and I re-ran the load-bearing probe. The register's defects
   are F-2 (the 22/5 headline doesn't match the spine's 28/3) and F-5 (stale sq-toze.8 status), plus
   the F-4 framing of release-gated controls. No missing gap was found.

3. **Overclaim audit:** **no critical/high overclaim.** The ZK/MPC honesty tripwire is **not
   implicated** — the SBOM slice makes no cryptographic-soundness claim; its only crypto-adjacent
   evidence is Sigstore *provenance* over artifacts (a supply-chain attestation, correctly scoped).
   The VEX is honestly modeled (`not_affected` on two genuinely-unmaintained-but-not-exploitable
   transitive informational advisories, with justifications). The findings are accuracy/consistency
   issues, not laundered guarantees.

---

## Coverage note

**Assessed (with independent verification):** all 7 NTIA rows (re-ran the probe), CDX-1/2/4
(inspected real output), CDX-3 (confirmed specVersion=1.3), all VEX rows (sync + staleness checked
against Cargo.lock), DEP-1..6 (deny.toml + supply-chain.yml jobs + config.toml read), INT-1/2/3
(script + lockfile + reproducibility claim), and the gap register + beads.
**Assessed at config-level only (could not exercise):** SIG-1/2/3, PUB-1/2, VEX-4, DEP-5 — these are
`v*`-release-triggered and **no release has ever run** (this is F-4 itself). I verified the workflow
wiring is correct but could not observe a produced/attested artifact. `gh attestation verify` /
`cosign verify-attestation` / `cargo audit bin` are unrunnable until a release exists; an external
auditor should re-check after the first `v*` tag.
**Not in scope of this framework (correctly deferred):** operator product-level SBOM aggregation,
deployed-image re-scanning, the JS-lockfile SBOM (GS-3) — all honestly marked operator-responsibility
or P2 gaps.

---

## New work to capture as a bead

- **(recommended new bead, P2, `sbom`)**: strip/normalize the absolute build-machine path from the
  generated SBOM root `bom-ref`/workspace-member refs in `scripts/gen-sbom-vex.sh` (the F-6
  code-hardening half). The doc-note half clears F-6 on its own; the bead tracks the optional
  sanitization. **NOTE:** the `bd` CLI is not installed in this audit environment (verified:
  `command -v bd/beads/bead` all empty), and the auditor must not hand-edit `.beads/`. The engineer
  /orchestrator must create this bead under epic `sq-toze` (next free id is `sq-toze.30`) with the
  CLI when remediating F-6.

(The doc-accuracy findings F-1..F-5 are remediated in `compliance/sbom/` by the engineer; no new
beads needed — F-1 rolls under the existing sq-toze.26.)

---

**FINDINGS: 6** (1 medium F-1, 1 medium-low F-4, 4 low F-2/F-3/F-5/F-6). Not signing off this round:
the evidence pack contains a **factually false recorded probe value (F-1)** and **headline counts
that contradict the control spine (F-2)** — both must be corrected before an honest sign-off, because
an external auditor re-running the one documented command would catch F-1 immediately. No critical or
high findings; the underlying SBOM controls are genuinely implemented. Re-audit on the engineer's
fixes. Standing caveat: SLSA-attestation operating-effectiveness (F-4) and any external accredited
SBOM attestation remain externally verifiable only after the first real release.
