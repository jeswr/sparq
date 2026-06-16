<!-- [OPUS-4.8] EU CRA Article 14 ENISA/CSIRT incident-reporting runbook. Bead sq-iy3p
     (gap GX-CRA-2, epic sq-toze). ADOPTABLE OPERATOR RUNBOOK — NOT a certification claim.
     NON-CANONICAL timing; org-specific details are <FILL-IN> placeholders. -->
# EU CRA Article 14 — incident-reporting runbook (template)

> **What this is.** An **adoptable operating runbook** that an entity which is the
> **manufacturer or open-source-software steward of record** for a product built on, or
> distributing, `sparq` can use to discharge the **Regulation (EU) 2024/2847 (Cyber
> Resilience Act) Article 14** reporting obligation. It operationalises the
> early-warning → notification → final-report timeline, the ENISA single-reporting-platform
> routing, and the coordination with `sparq`'s existing coordinated-disclosure flow.
>
> **What this is NOT.** This is **not** a claim that `sparq` is CRA-conformant, CE-marked,
> or that any reporting has occurred. It is **not** legal advice. `sparq` is a **library /
> component** distributed as a dependency; the Article 14 reporting duty falls on the
> **manufacturer of the product** that incorporates `sparq` (or, on the lighter-touch
> steward route, on the open-source-software steward of record — see
> [`README.md`](./README.md) §"Open-source-steward nuance"). The act of reporting is an
> **organisational and legal duty reserved to that party** and cannot be performed by the
> source tree, by a CI job, or by an agent. Every party-specific detail below is a
> **`<FILL-IN>`** placeholder the adopting entity must complete and have reviewed by its own
> legal/regulatory function.
>
> This runbook **extends, and does not contradict,** the existing private coordinated
> vulnerability disclosure (CVD) process in [`../../SECURITY.md`](../../SECURITY.md). The CVD
> process governs how a reporter reaches the project and how a fix is coordinated; **this
> runbook governs the separate, additional duty to notify the authorities** when the trigger
> conditions in Article 14 are met.

## 1. Scope and applicability

| Item | Value |
|---|---|
| Regulation | Regulation (EU) 2024/2847 — Cyber Resilience Act, Article 14 |
| Obligation start date | Article 14 reporting obligations apply from **2026-09-11** (the main body of CRA obligations applies from 2027-12-11) — non-canonical, confirm against the published Regulation |
| Who this runbook is for | The **manufacturer** of a product with digital elements that incorporates `sparq`, or the **open-source-software steward of record** relying on the Article 24 lighter-touch route |
| What `sparq` contributes | The **technical evidence** to support a report (SBOM/VEX for affected-component identification, the coordinated-disclosure record, the advisory text) — **not** the reporting act itself |
| Adopting entity | `<FILL-IN: legal entity name, registration number>` |
| Establishment / main place of business in the EU | `<FILL-IN: Member State of establishment, or the single point of contact for non-EU manufacturers>` |
| Determines competent CSIRT routing | The above establishment Member State (see §4) |

### What triggers a report (and what does not)

Article 14 requires reporting of **two distinct event classes**. A report is required only
when the event concerns **the product with digital elements** that the adopting entity has
**placed on the market** (i.e. a shipped product incorporating `sparq`), not for an internal
pre-release defect found and fixed before shipping.

1. **An actively exploited vulnerability** contained in the product — a vulnerability for
   which the adopting entity has evidence that a malicious actor has **successfully
   exploited or is exploiting** it (not merely that a PoC exists or that the vulnerability is
   theoretically exploitable).
2. **A severe incident having an impact on the security of the product** — an event that
   negatively affects, or is capable of negatively affecting, the ability of the product to
   protect the availability, authenticity, integrity, or confidentiality of sensitive or
   important data or functions.

**Out of trigger scope (route through `SECURITY.md` CVD only, no Article 14 report):**

- A privately reported vulnerability with **no evidence of active exploitation** and no
  severe incident → ordinary coordinated disclosure + fix + GHSA, per `SECURITY.md`.
- A defect found and remediated **before** the affected version is placed on the market.
- A report concerning a **research-scaffold crate that carries no security guarantee**
  (`sparq-zk`, `sparq-zk-compose`, `sparq-mpc`) — these provide **no** security property
  today by documented design (see `SECURITY.md` and
  [`../../research/zk-soundness-audit.md`](../../research/zk-soundness-audit.md)); a "break"
  of a property they never claimed is **not** a CRA-reportable vulnerability of the shipped
  product. (A memory-safety or DoS bug in `sparq-core`/`sparq-engine`/`sparq-server` reached
  from untrusted input **is** in scope if actively exploited in a shipped product.)

> **Honesty note.** Whether a given event meets the legal threshold for "actively exploited"
> or "severe incident" is a **judgement reserved to the adopting entity and its legal
> counsel**. This runbook gives the mechanics; it does not, and cannot, make the
> determination. When in doubt, the conservative posture is to prepare an early warning and
> consult counsel before the 24-hour clock expires.

## 2. Roles and responsibilities (`<FILL-IN>` for the adopting org)

| Role | Responsibility | Holder |
|---|---|---|
| **Incident lead** | Owns the Article 14 clock once a trigger is confirmed; decides early-warning dispatch; coordinates the report content | `<FILL-IN: name / on-call rota>` |
| **Security contact** | Receives the inbound report via the `SECURITY.md` channels; performs first triage; raises to the incident lead | `<FILL-IN>` (today: `jesse@jeswr.org` + the GHSA channel for the upstream project) |
| **Legal / regulatory** | Confirms the legal threshold (actively-exploited / severe-incident), approves report content, owns the relationship with the authority | `<FILL-IN: legal counsel / DPO if data involved>` |
| **Technical SME** | Identifies the affected component + versions from the SBOM/VEX; assesses corrective measures | `<FILL-IN>` |
| **Communications** | Prepares user-facing advisory + (where required by the CSIRT) public disclosure coordination | `<FILL-IN>` |
| **ENISA-platform account owner** | Holds the credentials/registration for the ENISA single reporting platform | `<FILL-IN>` |

## 3. The Article 14 reporting timeline

Three reports are due to the **coordinating CSIRT and to ENISA**, submitted through the
**single reporting platform** (see §4). The clocks start **from the moment the adopting
entity becomes aware** of the actively exploited vulnerability or the severe incident.

| Stage | Deadline (from awareness) | Content (high level) |
|---|---|---|
| **1. Early warning** | **within 24 hours** | A first notification that an actively exploited vulnerability / severe incident exists; whether it is suspected to be caused by unlawful or malicious acts; and the Member States where the product is (to the entity's knowledge) made available. May be terse. |
| **2. Vulnerability / incident notification** | **within 72 hours** | An update on the early warning: general information available about the vulnerability/incident, its status, and any **corrective or mitigating measures** taken and recommended to users. |
| **3. Final report** | for an **actively exploited vulnerability: within 14 days** of a corrective or mitigating measure being available; for a **severe incident: within 1 month** of the 72-hour notification | A description of the vulnerability/incident (severity, impact); where available, information on the actor/exploitation; and the security update or corrective measures put in place. |

> **Clock confirmation.** The exact deadlines, the platform mechanics, and any intermediate
> obligations are governed by the **published text of Regulation (EU) 2024/2847 and the
> implementing acts / ENISA platform documentation**. The values above are recorded as a
> working aid (non-canonical) and the adopting entity **must confirm them against the
> current official sources** before relying on them.

### Timeline at a glance

```text
T+0       Awareness of actively-exploited vuln OR severe incident (in a SHIPPED product)
  |
  |--- triage + legal threshold confirmation (§1, §5 step 2-3) ------------------+
  v                                                                             |
T+24h     EARLY WARNING  -> ENISA single reporting platform -> coordinating CSIRT
  |
  v
T+72h     NOTIFICATION   -> update: corrective/mitigating measures
  |
  v
[vuln]  T + (measure available) + 14 days : FINAL REPORT
[incident] T+72h + 1 month               : FINAL REPORT
```

## 4. Routing — ENISA single reporting platform and the CSIRT

- Reports are submitted to the **single reporting platform** established under the CRA, which
  routes the notification to the **CSIRT designated as coordinator** and to **ENISA**.
- The **coordinating CSIRT** is determined by the adopting entity's place of establishment /
  main place of business in the Union; for a non-EU manufacturer it is determined by the
  authorised representative / importer arrangements. Record the applicable national CSIRT and
  any registration steps here:

| Routing item | Value |
|---|---|
| Member State of establishment | `<FILL-IN>` |
| Coordinating national CSIRT | `<FILL-IN: name + intake URL/contact>` |
| ENISA single-reporting-platform URL | `<FILL-IN: official platform URL once published>` |
| Platform account / registration reference | `<FILL-IN>` |
| Fallback channel if the platform is unavailable | `<FILL-IN: the CSIRT's direct intake, per its published procedure>` |

> The single reporting platform and the precise CSIRT routing are **operational details set
> by ENISA and the Member States**; confirm them against official ENISA / national-CSIRT
> guidance at adoption time. Do not assume the URLs/contacts above without verification.

## 5. The procedure (step by step)

1. **Inbound.** A potential issue arrives via a `SECURITY.md` channel (private GHSA report or
   the security email), via telemetry, or via a third-party/CSIRT notification. The
   **security contact** acknowledges per the `SECURITY.md` response targets.
2. **Triage.** The technical SME reproduces/assesses. Determine: (a) does it affect a
   **shipped** product version, and (b) is there **evidence of active exploitation** or is
   this a **severe incident**? If neither — handle as ordinary CVD (`SECURITY.md`), **stop**;
   no Article 14 report. Record the decision and its rationale.
3. **Threshold confirmation.** If (a)+(b) may hold, the **incident lead** raises to
   **legal/regulatory**. Legal confirms the Article 14 threshold. **The 24-hour clock is
   treated as running from awareness** — prepare the early warning in parallel with this
   confirmation; do not wait for a perfect assessment.
4. **Affected-component identification.** Use the per-release **CycloneDX SBOM** and the
   checked-in **VEX** to pin the exact affected component(s), version range, and PURL — see
   [`evidence.md`](./evidence.md) (II.1) and `supply-chain/vex.cdx.json`. This produces the
   precise product/version identification the report requires.
5. **Early warning (≤ 24h).** The ENISA-platform account owner submits the early warning via
   the platform (§4) using the §6 content checklist (early-warning fields). The incident lead
   logs the submission timestamp + reference number in the incident record.
6. **Notification (≤ 72h).** Submit the update, including the corrective/mitigating measures
   prepared so far (see step 8). Update the incident record.
7. **Final report (14 days / 1 month).** Submit the final report once the corrective measure
   is available (vuln) or within one month of the notification (incident).
8. **Remediate + disseminate (in parallel, per `SECURITY.md` + controls II.2/II.4/II.7).**
   Land the fix, ship the next release of the affected artifact, publish a GHSA + changelog
   entry, and issue the user-facing advisory. The CRA reporting duty is **in addition to**,
   not a replacement for, the coordinated public disclosure.
9. **Close-out.** Record lessons learned; file follow-up work as beads under the project's
   tracker; review whether the support-period statement (GX-CRA-1) or this runbook needs
   updating.

## 6. Report content checklist

Each stage carries a defined minimum content. The fields below map the Article 14 stages to
the evidence `sparq` can supply versus what the adopting entity must author.

| Field | Early warning (24h) | Notification (72h) | Final report | Source |
|---|---|---|---|---|
| Reporting entity identity + contact | yes | yes | yes | `<FILL-IN>` org identity; cf. Annex II-A.1 (`SECURITY.md`, `security.txt`) |
| Product identity + affected version(s) | yes | yes | yes | SBOM/VEX (§5 step 4); `Cargo.toml` / release version |
| Nature: actively-exploited vuln or severe incident | yes | yes | yes | triage record |
| Suspected to be caused by unlawful/malicious acts? | yes | update | confirm | incident lead + legal |
| Member States where product is made available (to entity's knowledge) | yes | update | yes | `<FILL-IN>` distribution record |
| General information / status | — | yes | yes | technical SME |
| Corrective / mitigating measures taken + recommended to users | — | yes | yes | the fix + advisory (`SECURITY.md`, `CHANGELOG.md`, GHSA) |
| Severity + impact | — | (if known) | yes | severity assessment |
| Actor / exploitation detail (where available) | — | (if known) | yes | incident analysis |
| The security update / corrective measure put in place | — | (if available) | yes | release + GHSA + provenance (controls II.4/II.7) |

> Confirm the **exact mandatory fields and formats** against the ENISA platform schema and
> the CRA implementing acts at adoption time — the platform may mandate fields beyond this
> working checklist.

## 7. Interaction with the existing disclosure flow

| Concern | `SECURITY.md` CVD process | This Article 14 runbook |
|---|---|---|
| Who it serves | The reporter and the public | The competent authorities (CSIRT + ENISA) |
| Trigger | Any suspected vulnerability | **Actively-exploited** vuln **or severe incident** in a shipped product |
| Timeline | Best-effort ack 5 / assess 10 business days | Hard: 24h / 72h / 14-day or 1-month |
| Output | Private fix coordination → GHSA + release | Authority notifications via the ENISA platform |
| Relationship | Upstream; feeds triage (§5 step 1) | Downstream + additional; never replaces public CVD |

The two are complementary: the CVD process produces the technical material (affected
versions, fix, advisory) that the Article 14 reports cite, and the SBOM/VEX supply-chain
estate provides the affected-component identification. **Neither substitutes for the other.**

## 8. Records to retain

Keep an **incident record** per Article 14 event: awareness timestamp; triage + threshold
decision and rationale; each report's submission timestamp + platform reference; the
corrective measures; and the close-out. These feed the Annex VII technical documentation
(CRA-CA.4) the manufacturer/steward retains. Retention period: `<FILL-IN: per the CRA / org
records-retention policy>`.

## 9. Honesty statement (restated)

- This is an **adoptable runbook + template**, not a certification artifact and not a
  statement that any report has been or must be made by the `sparq` project itself.
- `sparq` is a **library / component**; the Article 14 obligation is the
  **manufacturer's / steward-of-record's**, not the source tree's.
- The runbook **does not assert CRA conformity, CE marking, or an EU declaration of
  conformity** — those remain the organisational acts recorded as *audit-ready* in
  [`controls.md`](./controls.md) (CRA-CA.2/CA.3) and the consolidated register.
- **No security claim here rests on the ZK/MPC crates**, which are documented as providing
  **no** security guarantee (`SECURITY.md`, `research/zk-soundness-audit.md`).
- Dates, deadlines, and platform/CSIRT routing are **non-canonical working aids**; confirm
  them against the official Regulation, implementing acts, and ENISA/CSIRT guidance before
  relying on them.

## References

- [`README.md`](./README.md) — CRA scope, the manufacturer-vs-steward nuance, the
  operator/engine split.
- [`controls.md`](./controls.md) — CRA-CA.5 (this runbook addresses it), II.2/II.4/II.7
  (remediation + disclosure + secure distribution).
- [`evidence.md`](./evidence.md) — SBOM/VEX and advisory-flow evidence the reports cite.
- [`gap-register.md`](./gap-register.md) — GX-CRA-2 (addressed by this runbook), GX-CRA-1
  (support period), GX-CRA-3 (cybersecurity policy).
- [`../../SECURITY.md`](../../SECURITY.md) — the coordinated vulnerability disclosure flow
  this runbook extends.
- [`../../research/zk-soundness-audit.md`](../../research/zk-soundness-audit.md) — the
  documented "v1 ZK verifier is NOT sound" verdict.
- Regulation (EU) 2024/2847 (Cyber Resilience Act), Article 14 and Annexes I/II/VII.
