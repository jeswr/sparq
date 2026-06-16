<!-- [OPUS-4.8] EU CRA compliance audit — adversarial review of cert-cra / PR #233. Epic sq-toze.
     Independent re-verification by the SPARQ compliance-auditor. NON-CANONICAL timing. -->
# EU CRA — audit findings (cert-cra / PR #233)

> 🤖 **SPARQ agent** — adversarial internal audit of the EU Cyber Resilience Act evidence on
> branch `cert-cra` (`1c75983`), draft PR #233. Verdict at the bottom.

**Auditor mandate:** independently verify the CRA claims against the *actual* repo — do not
rubber-stamp. Re-run the load-bearing gates myself; hunt overclaiming; confirm the ZK/MPC
exclusion and the CE-marking non-assertion; confirm the gap set is real and beaded.

**Date:** 2026-06-16 · **Scope reviewed:** `compliance/cra/{README,controls,evidence,gap-register}.md`
(the only files PR #233 adds — `git diff --stat origin/main...origin/cert-cra` = 4 files, +316/-0).
The underlying code/config/CI that the claims cite is **byte-identical** between `cert-cra` and
`main` (`git diff origin/cert-cra origin/main -- deny.toml .github/workflows/supply-chain.yml
.github/workflows/release.yml SECURITY.md .well-known/security.txt` → empty), so re-running on the
worktree HEAD exercises exactly what the PR ships.

---

## Verdict: **SIGN-OFF** — FINDINGS: 0

No open findings. Every "implemented & verified" row I sampled is backed by evidence that
actually demonstrates the control; the audit-ready / operator-split rows are honestly labelled and
not dodged; the ZK/MPC estate is excluded from every security claim; CE marking / the EU
declaration of conformity are explicitly **not** asserted; and the three open gaps are the real
set, each carrying a live `bd` bead. Standing caveat (carried, not a finding): the conformity
assessment, EU declaration of conformity, CE marking, and the *act* of Article-14 reporting are
organizational/legal acts reserved to the manufacturer/steward and remain **external** — correctly
labelled *audit-ready / out of agent scope*, never self-certified.

---

## What I independently checked (the load-bearing items)

### A. I.2 "no known exploitable vulnerabilities" rests on a REAL passing gate ✔ (re-run)

The single most load-bearing CRA claim. I re-ran both gates on the PR tree:

```
$ cargo deny check advisories
advisories ok                       → exit 0
$ cargo deny check bans sources licenses
bans ok, licenses ok, sources ok    → exit 0   (only duplicate-version WARNINGs, non-fatal by policy)
```

cargo-deny 0.19.8. The gate is genuinely **passing and fail-closed**, not cosmetic:

- `deny.toml` confirms the claimed posture: `[advisories] yanked = "deny"`; **exactly two**
  `ignore` entries, both `unmaintained` *informational* advisories on transitive deps with no safe
  upgrade — `RUSTSEC-2024-0436` (paste, via wgpu) and `RUSTSEC-2025-0134` (rustls-pemfile, via
  ureq→rustls-native-certs); each carries a justification + bead (sq-l8bv / sq-g2xs). No
  vulnerability and no yanked crate is tolerated. The advisories schema is v2 (no severity
  knobs), so **any** un-ignored advisory fails the check — the claim "removing a justified ignore
  re-flags it" is structurally true.
- `[licenses]` is a permissive-only allowlist with per-crate, per-version exceptions (CeCILL-B /
  AFL-3.0 / MPL-2.0 / Apache-WITH-LLVM-exception / CDLA-Permissive-2.0), scoped so the broad
  allowlist stays permissive-only. No strong/network copyleft is broadly allowed.
- `[sources]` rejects unknown registry/git; crates.io is the only allowed registry.
- The PR-time gate is in `.github/workflows/supply-chain.yml#audit`, both steps `run:` with **no
  `continue-on-error`** (the only occurrence of that string in the file is inside a comment noting
  the *former* degraded state was resolved). The `vet` job (`cargo vet --locked`) and `sbom` job
  are likewise plain gating steps. The advisories degradation gap (GX-1, sq-toze.2) is genuinely
  closed.

**Conclusion:** I.2 = *implemented & verified* is grounded. Not overclaimed.

### B. Secure-by-default server posture I.2(a) ✔ (built + tested)

`bind_posture()` (`crates/sparq-server/src/http.rs:441`) fail-closes a non-loopback bind; `main.rs`
turns `BindPosture::RemoteRefused` into a non-zero exit *before* binding. I built and ran the
dedicated test module:

```
$ cargo test -p sparq-server --lib bind_posture
7 passed; 0 failed   (incl. non_loopback_without_optin_is_refused,
                            write_only_token_still_refused_without_optin,
                            full_auth_allows_remote_bind_without_optin)
```

The optional Bearer auth (`SPARQ_AUTH_TOKEN` / `SPARQ_AUTH_TOKEN_READ`) and the DoS-limit flags
(`--max-body-bytes`, `--max-decompress-ratio` zip-bomb guard, `--max-concurrent`, `--max-results`)
are all present in `main.rs` as cited. I.2(a)/(f) are real.

### C. Operator-split / audit-ready rows are honest, not dodges ✔

- **I.2(b) auth** — labelled *audit-ready (operator split)*: the **mechanism** (optional Bearer +
  read-gating) is sparq's and present; per-user authz is boundary B3 → gateway/sparq-solid. Honest.
- **I.2(c) TLS / at-rest** — *operator (TLS/at-rest)*: TLS terminates at the operator proxy; the
  plaintext-Bearer sniffability is *called out* rather than hidden. Honest.
- **I.2(e) data-min** — *operator-owned (controller)*: sparq processes only the loaded RDF + issued
  SPARQL and emits no telemetry of its own; the no-leakage-in-errors/logs obligation is kept as
  sparq's. Honest scoping for a data engine.
- **I.2(j) logging** — *operator split*: sparq provides the signal (`metrics.rs` + tracing),
  monitoring/retention is the operator SOC. Honest.
- **I.2(g) "minimise impact on other networks"** is *more* than claimed: SERVICE federation is
  **default-DENY-ALL** (`crates/sparq-server/src/service_config.rs` — empty allowlist denies every
  SERVICE clause; 10 `service_config` tests pass). The SSRF surface is closed by default, which
  strengthens rather than inflates the claim. No overclaim here.

### D. ZK/MPC excluded from every security claim ✔ (honesty tripwire)

- `SECURITY.md` §"research scaffolds with NO security guarantee" is intact: the **v1 ZK verifier is
  NOT sound** (`sparq-zk`/`sparq-zk-compose`, anchored on `research/zk-soundness-audit.md`) and
  `sparq-mpc` is *cryptography deferred*.
- The CRA tree references ZK/MPC only to **exclude** it: README nuance #4, controls.md header note
  ("no row's 'met' status depends on them"), evidence E7. Grepping the CRA tree for any
  ZK/MPC-as-guarantee phrasing returns only the two honest "NO-guarantee scope" mentions in the
  Annex II A.4 *known-limitations* row. **No CRA confidentiality/integrity row rests on the crypto
  estate.** Tripwire clear.

### E. CE marking / EU declaration of conformity correctly NOT asserted ✔

Every mention of CE marking / declaration of conformity / conformity assessment in the CRA tree is
in a **negating** context: README §1 ("cannot produce … the manufacturer's organizational acts"),
controls.md §"formal conformity layer" (CRA-CA.2/CRA-CA.3 = *audit-ready / out of agent scope*,
"This document does not assert CE conformity"), evidence E7. The README status legend and one-line
posture both state CE is not claimed. The open-source-steward (Art. 24) vs manufacturer split is
described honestly and made framework-neutral. No self-certification of a conformity act anywhere.

### F. The supporting evidence pack ✔

- **SBOM/VEX (II.1):** per-release CycloneDX in `release.yml#sbom` + `scripts/gen-sbom-vex.sh`; CI
  SBOM artifact in `supply-chain.yml#sbom`; checked-in `supply-chain/vex.cdx.json` is **1:1** with
  `deny.toml` — exactly the two RUSTSEC ids, both `not_affected`. Quality gaps (GS-1/3/4) are
  honestly deferred, not claimed as met.
- **Signed releases / provenance (II.7):** `release.yml` has real `actions/attest-build-provenance`
  (SHA-pinned `@a2bbfa2… v4.1.0`) over archives + SBOM/VEX, `SHA256SUMS`, `cargo auditable build`,
  and buildkit `provenance: mode=max` + `sbom: true` on the ghcr image. Labelled *implemented &
  verified (with raise)* — the raises (dist.yml lane provenance GX-9, registry-package provenance
  GX-10) are carried as gaps, not papered over. Honest SLSA framing (~L2-ish on GitHub-hosted
  runners), consistent with the repo's SLSA posture.
- **Dependabot (II.2):** 4 ecosystems (cargo / github-actions / npm / pip); groups use
  `applies-to: version-updates` only, so **security** updates stay ungrouped (one PR each) — exactly
  as controls.md claims.
- **security.txt (II.5/.6):** RFC 9116 fields present; `Expires: 2027-06-15T00:00:00Z` is in the
  future. Two `Contact:` entries + `Policy`/`Canonical`.

### G. The three gaps are the real set and are beaded ✔

`gap-register.md` lists exactly **GX-CRA-1** (support/EOL period statement, Annex II A.6),
**GX-CRA-2** (Article 14 ENISA/CSIRT reporting runbook), **GX-CRA-3** (single named cybersecurity-
policy doc, Art. 24/13). I confirmed all three beads exist in the main checkout's
`.beads/issues.jsonl` with correctly-mapped titles:

- `sq-f8tv` — "[cert][CRA][gap GX-CRA-1] No concrete CRA support/EOL period statement (Annex II A.6)"
- `sq-iy3p` — "[cert][CRA][gap GX-CRA-2] No Article 14 ENISA/CSIRT reporting runbook …"
- `sq-d43g` — "[cert][CRA][gap GX-CRA-3] Adopt a single named cybersecurity policy doc …"

These IDs are the set named in the audit task. (They are not committed onto the `cert-cra` branch's
`.beads` snapshot — expected, since beads are exported on `main` and re-synced; the live tracker
holds them.) The cross-cutting GX-9/GX-10/GX-12 are correctly attributed to the slsa/sbom/cis
worktrees and *cited, not duplicate-fixed*. The honesty note correctly declines to record the
conformity-assessment/CE acts as "fixable gaps with a bead."

---

## Coverage note

**Assessed (independently re-verified):** Annex I Part I — I.1, I.2(a)–(j); Annex I Part II —
II.1–II.8; Annex II — A.1–A.6; the formal conformity layer CRA-CA.1–CRA-CA.6. The Annex I/II
requirement set is **complete** in the mapping — no applicable essential requirement or
vulnerability-handling obligation is silently omitted. Re-ran: `cargo deny check advisories`,
`cargo deny check bans sources licenses`, `cargo test -p sparq-server --lib bind_posture`,
`cargo test -p sparq-server --lib service_config`. Inspected: `deny.toml`, `supply-chain.yml`,
`release.yml`, `dependabot.yml`, `Dockerfile`, `.well-known/security.txt`, `supply-chain/vex.cdx.json`,
`SECURITY.md`, `crates/sparq-server/src/{main.rs,http.rs,service_config.rs}`, `.beads/issues.jsonl`.

**Could not fully verify from the source tree (no finding — out of scope by design):**
- Live **release-artifact provenance** (`gh attestation verify <archive>`) requires a published
  Release; verified the workflow *wiring* is correct and SHA-pinned, not a live attestation.
- The **act** of Article-14 reporting, the signed **conformity assessment**, **EU declaration of
  conformity**, **CE marking**, and external cryptographer sign-off of the ZK estate are
  organizational/legal/external acts — correctly labelled *audit-ready / out of agent scope*. These
  are the standing external caveat, not open findings.

**No new remediation bead required** — this audit produced zero findings, so there is no
code/config fix to track. The three documented gaps already carry beads (sq-f8tv/iy3p/d43g).

---

`FINDINGS: 0` · `SIGN-OFF` (with the standing external-auditor / external-cryptographer caveat above).
