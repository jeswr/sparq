<!-- [OPUS-4.8] sq-toze — ISO/IEC 27001 AUDIT findings (compliance-auditor). Adversarial
     independent verification of PR #232 / branch worktree-agent-a98e0664c880acf78
     (cert-iso27001). NON-CANONICAL timing (EC2 box). Re-review when Fable returns. -->

# ISO/IEC 27001:2022 — audit findings (sparq, PR #232)

> 🤖 SPARQ agent — independent compliance **auditor** (epic `sq-toze`). I do not edit
> source or `compliance/iso27001/`; I verify and report. The engineer remediates.

**Under audit:** `compliance/iso27001/{README,controls,evidence,gap-register,soa-template}.md`
on PR #232 (head `worktree-agent-a98e0664c880acf78`). The engineer mapped all 93 Annex A
(ISO/IEC 27002:2022) controls → **26 IMPL & verified / 27 audit-ready / 40 N/A(operator)**,
with **0 open Annex-A control gaps** + **2 readiness gaps** (GAP-ISO-1, GAP-ISO-2).

**Verdict: `FINDINGS: 2`** (both **low**; the framework is substantively sound and honest).
The audit-ready vs verified split is honest, A.8.24 correctly excludes the ZK/MPC estate,
and the "v1 ZK verifier NOT sound" verdict is intact and consistently referenced. The two
findings are small accuracy defects in the *favourable* (overclaim) direction — flagged
because that is exactly the direction an auditor must not wave through.

---

## What I independently verified (all PASS)

Every load-bearing IMPL claim was re-checked against the actual repo at HEAD (not just the
doc). The following all hold:

| Claim (controls.md / evidence.md) | Check | Result |
|---|---|---|
| `#![forbid(unsafe_code)]` in 20 of 25 crates; 5 with unsafe are sparq-core/-vectors/-cli/-zk-compose/-bench (A.8.27) | `grep -rl forbid(unsafe_code) crates/` → 20; the 5 without exactly match | **PASS** |
| `cargo deny check advisories` GATING, GX-1 un-degraded (A.8.8) | `supply-chain.yml` L65–66: `cargo deny check advisories` with **no** `continue-on-error` on any deny step | **PASS** |
| clippy `-D warnings` GATING (A.5.36/A.8.28) | `ci.yml` L300 `clippy (deny warnings)`; the L304 `continue-on-error` attaches to the *rustfmt* step, not clippy | **PASS** |
| `scripts/unsafe-gate.py --check` is a *required* gating lane (A.8.28) | `ci.yml` L638; lane name `unsafe-register (count ratchet)` has no advisory/informational token ⇒ ci-summary treats it as REQUIRED; `bench/unsafe-snapshot.json` floor present + coherent (sparq-core=42, total 56) | **PASS** |
| CODEOWNERS at repo root, 37 lines, owner lines for zk*/mpc/core/server/CI (A.5.2/A.5.3/A.8.4) | `wc -l CODEOWNERS`=37; explicit lines for all cited high-risk paths | **PASS** |
| `.well-known/security.txt` (RFC 9116) present, future `Expires` (A.6.8/A.5.24) | exists; `Expires: 2027-06-15`, Contact GHSA + mailto, Policy → SECURITY.md | **PASS** |
| A.8.24 makes **no** claim over ZK/MPC; signing + operator TLS only | `SECURITY.md` L65–73 + `research/zk-soundness-audit.md` (6 CRITICALs) intact; the A.8.24 row routes soundness to `cryptoreview` and claims only Sigstore/SLSA + operator TLS | **PASS (critical honesty gate held)** |
| SLSA `attest-build-provenance@a2bbfa25…` + SBOM+VEX attested on release + SHA256SUMS (A.5.21/A.8.4) | `release.yml` L108/L145 (SHA-pinned), `scripts/gen-sbom-vex.sh`, checked-in `supply-chain/vex.cdx.json`, SHA256SUMS L176 | **PASS** |
| CycloneDX SBOM in CI (A.5.9/A.5.21) | `supply-chain.yml` L95–109 `cargo cyclonedx --all --format json` | **PASS** |
| Dependabot 4 ecosystems (A.5.7/A.8.8) | `dependabot.yml`: cargo, github-actions, npm, pip | **PASS** |
| Scorecard published (A.5.6) | `scorecard.yml` L44 `publish_results: true` | **PASS** |
| `QueryBudget` is a real enforced primitive, not a stub (A.8.6) | `sparq-engine/src/lib.rs:64` struct + `exec.rs` cooperative deadline/row-cap checks at coarse sites | **PASS** |
| threat-model boundaries B3 (no-auth) + B5 (mmap unsafe) (A.8.26/A.8.27) | `research/threat-model.md` L94 (B3), L101–105 (B5); `verify_manifest` at `sparq-zk-compose/src/verifier.rs:3951` | **PASS** |
| `SPARQ_AUTH_TOKEN` optional bearer token exists (A.5.15/A.8.5) | `sparq-server/src/http.rs:359` reads env; `_READ` gate at L362 | **PASS** |
| CONTRIBUTING §Secure coding + §Input-validation no-leak rule (A.8.12/A.8.28/A.6.3) | `CONTRIBUTING.md` L68 (Secure coding), L95–102 ("do not leak RDF/SPARQL content, internal paths, or stack") | **PASS** |
| `unsafe-register.md` attests sparq-core sites (A.8.27/A.8.28) | `compliance/memsafety/unsafe-register.md` present; coverage matrix (miri / mmap-oracle / forbid-20 / ratchet) coherent; the 5 unsafe crates match the snapshot exactly | **PASS** |
| Audit-ready controls cite a real doc-of-record (SECURITY.md, threat-model, CONTRIBUTING, CODEOWNERS, docs/branch-protection.md) | all five exist | **PASS** |
| GAP-ISO-1 (no ISMS artifacts) + GAP-ISO-2 (no operator-deploy doc) are the real readiness gap set | confirmed; correctly labelled documentation/template gaps, not code controls; both honestly *not* upgraded to PASS | **PASS** |

The audit-ready clustering in A.5/A.6 is **correctly** held as audit-ready (management-system
acts the repo cannot substitute for), not silently upgraded. The 40 N/A(operator) controls
(A.7 physical block, B3 access-control family, runtime monitoring/backup/availability,
network controls) are honestly scoped, not used to dodge a control sparq owns. **No
overclaim of a control's *status* was found.**

---

## Findings

### Finding 1 — Roll-up control counts overstate IMPL by 2 (and understate N/A by 2)

- **Severity:** Low
- **Control / artifact:** `controls.md` §Roll-up table; row "A.8 Technological (34): 16 IMPL /
  6 AUDIT-READY / 12 N/A(operator)" and the **headline "Total 26 IMPL / 40 N/A(operator)"**.
- **What I checked:** Enumerated every A.8 row's primary status from the controls table:
  - Pure `IMPL`: A.8.4, .7, .8, .9, .18, .19, .25, .26, .27, .28, .29, .32 = **12**
  - IMPL-leaning dual: A.8.6 (`N/A(op) → IMPL(partial)`), A.8.16 (`IMPL(repo) / N/A(op)`) = **2**
  - ⇒ a defensible A.8 IMPL bucket is **14**, not 16.
  - `AUDIT-READY`: A.8.12, .15, .24, .31 (4 pure) + A.8.2, A.8.5 (`N/A(op) → AUDIT-READY`) = **6** ✓
  - Pure `N/A(op)`: A.8.1, .3, .10, .11, .13, .14, .17, .20, .21, .22, .23, .30, .33, .34 = **14**
  - ⇒ A.8 split should read **14 / 6 / 14** (= 34), giving a grand total of **24 IMPL /
    27 AUDIT-READY / 42 N/A(op)** = 93, not the stated **26 / 27 / 40**.
- **Why it fails:** The A.8 IMPL=16 figure cannot be reconstructed from the rows; the only way
  to reach 16 is to bucket the `N/A(op) → AUDIT-READY` rows (A.8.2, A.8.5) as IMPL, which they
  are not. The error is in the **overclaim direction** — the headline advertises *two more*
  "implemented & verified" controls than the table supports, and *two fewer* operator-owned
  controls. For a doc whose whole credibility rests on an honest verified/audit-ready/N-A
  split, a self-inconsistent headline count is a (minor) integrity defect a Stage-1 auditor
  would query.
- **Remediation:** Recount the roll-up directly from the controls table and adopt a single,
  documented rule for how dual-status rows (`N/A(op) → AUDIT-READY`, `IMPL(repo) / N/A(op)`)
  are bucketed (recommend: bucket by the *sparq-side* primary status and add a footnote listing
  the dual rows). Correct the A.8 row to 14/6/14 (or state the bucketing rule that yields 16)
  and reconcile the headline. Bead below.

### Finding 2 — A.5.19 / A.5.21 "cargo-vet per-dependency attestations" overstates the current evidence (it is exemptions, not audits)

- **Severity:** Low
- **Control / artifact:** `controls.md` A.5.19 ("cargo-vet per-dependency attestations
  (GATING)") and A.5.21; `evidence.md` §"Vulnerability management" ("`cargo-vet check` is also
  GATING (per-dependency attestations)").
- **What I checked:** `supply-chain.yml` L80–92 runs `cargo vet --locked` as a gating lane
  (verified — real gate). But the substrate: `supply-chain/audits.toml` contains **zero**
  first-party audits (`[audits]` empty, 35 bytes), and `supply-chain/config.toml` carries
  **349 `[[exemptions.*]]`** entries (`grep -cE 'exemptions\.'` = 349) plus six trusted
  `[imports.*]` audit sets (Mozilla, Google, ISRG, Embark, Bytecode-Alliance, Zcash).
- **Why it fails:** With every crate exempted (or covered by an imported audit), `cargo vet
  --locked` passes today by *exemption*, not by sparq having *attested* any dependency. The
  control text "per-dependency **attestations**" implies sparq performed/holds the audits; in
  fact the gate currently provides the weaker (still useful) guarantee that **no new,
  un-exempted, un-imported dependency can enter the tree silently** — a supply-chain *ratchet*.
  Calling 349 exemptions "attestations" mildly overstates the assurance level a relying party
  would read into A.5.19.
- **Remediation:** Reword A.5.19/A.5.21 + the evidence line to state precisely what the lane
  proves: "`cargo vet --locked` (GATING) enforces a supply-chain **ratchet** — every dependency
  must be covered by a trusted imported audit (Mozilla/Google/ISRG/Embark/BCA/Zcash) or an
  explicit `[[exemptions]]` entry; **0 first-party audits exist yet** (`audits.toml` empty),
  so new un-covered crates cannot enter silently, but sparq does not yet attest dependencies
  itself." Optionally track reducing the 349 exemptions as a supply-chain maturity bead (the
  `sbom`/`slsa` worktrees own GX-7). Bead below.

---

## Observations (NOT findings — recorded so the next round sees the check was made)

- **Forward cross-references to not-yet-merged peer frameworks are acceptable.** `controls.md`
  cross-refs `compliance/{asvs,sbom,slsa,cis,cra,cryptoreview}/`, `compliance/data-flow.md`,
  `compliance/dpia.md`, and `compliance/policies/`, **none of which exist on `main` yet**
  (only `compliance/{audit,memsafety}/` do). In a parallel engineer-loop these are peer
  deliverables; `soa-template.md` explicitly states their absence "is part of the respective
  framework's remediation, not a separate ISO 27001 gap." The only cross-ref that is
  load-bearing to a *this-framework* IMPL claim — `compliance/memsafety/unsafe-register.md`
  (A.8.27/A.8.28) — **does** exist and is coherent. No finding. **Watch item:** if the privacy
  / asvs worktrees do not land, A.8.12 (no-leak *verification*), A.5.34 and A.8.11 (privacy)
  become dangling; they are correctly AUDIT-READY/cross-ref today, so not a gap now.
- **Plan-vs-doc forbid-crate count discrepancy resolved in the doc's favour.** The cert plan
  (`research/production-certification-plan.md`) says "forbid in 23 crates"; the engineer
  re-counted to **20 of 25** and the live count confirms 20. The engineer was *more* accurate
  than the planning doc. No finding.
- **GAP-ISO-3 (CODEOWNERS) "resolved on inspection" is genuine.** I independently confirmed
  CODEOWNERS is at the repo root (37 lines, populated), not missing/at `.github/`. The
  engineer's recorded self-correction is honest.

---

## Coverage note

- **Assessed:** all 93 Annex A rows for status-honesty; every IMPL claim in A.8 + the
  supplier/development/vuln-management IMPL claims in A.5/A.6 against real files/CI; the A.8.24
  crypto-exclusion honesty gate (SECURITY.md + zk-soundness-audit.md); the cargo-deny/cargo-vet
  gating substrate; CODEOWNERS / security.txt / branch-protection doc-of-record existence; the
  unsafe-register + ratchet; QueryBudget enforcement; the two readiness gaps + beading note;
  the roll-up arithmetic.
- **Could NOT fully assess (external by definition / out of agent scope):**
  - The **branch-protection ruleset itself** is configured out-of-repo on GitHub; I verified
    the doc-of-record (`docs/branch-protection.md`) but cannot witness the live ruleset. The
    doc correctly marks A.5.3/A.8.4 enforcement as AUDIT-READY at the certificate level for
    this reason — honest.
  - **Whether CI lanes actually pass on the PR branch** — I verified the workflow *definitions*
    are gating and not `continue-on-error`; I did not re-run the full CI matrix (the evidence
    pack is a static-definition claim, which is the appropriate ISO evidence form).
  - The **ISO 27001 certificate** itself — issued only by an accredited body after a Stage 1 +
    Stage 2 ISMS audit. Correctly labelled external throughout; nothing in the PR claims it.
  - **External-cryptographer sign-off** of the ZK/MPC estate — out of agent scope; correctly
    routed to `cryptoreview`. A.8.24 makes no crypto-soundness claim, as required.

---

## Verdict

`FINDINGS: 2` (both **low**, both overclaim-direction accuracy defects: a self-inconsistent
roll-up count, and "attestations" overstating cargo-vet exemptions). **No** high/critical
finding: the audit-ready vs verified split is honest, the N/A(operator) scoping does not dodge
a control sparq owns, and **A.8.24 correctly excludes the `sparq-zk`/`-zk-compose`/`-mpc`
estate** with the "v1 verifier NOT sound" verdict intact and consistently referenced. Fix the
two wordings/counts and this framework signs off (standing caveat: the certificate and the
external-cryptographer item remain external by definition).

**Beads to create** (main checkout, `bd create … --epic sq-toze`; not hand-edited into
`.beads/`):
- `iso27001: correct controls.md roll-up A.8 IMPL count (14 not 16) + document dual-status
  bucketing rule; reconcile headline to 24/27/42` (P2, sq-toze) — Finding 1.
- `iso27001: reword A.5.19/A.5.21 cargo-vet evidence — "ratchet via exemptions/imports",
  not "per-dependency attestations" (audits.toml empty)` (P2, sq-toze) — Finding 2.

> 🤖 SPARQ agent — auditor pass on PR #232 (epic `sq-toze`). Findings above are for the
> compliance-engineer to remediate; I do not edit `compliance/iso27001/` or source.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
