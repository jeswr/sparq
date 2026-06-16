<!-- [OPUS-4.8] sq-toze / sq-toze.20 — INDEPENDENT AUDIT of the cryptographic-review slice.
     Adversarial auditor pass for PR #235 / branch cert-cryptoreview. Authored by Opus 4.8
     (1M context) while Fable 5 unavailable — re-review when Fable returns. NON-CANONICAL timing. -->

# Cryptographic-review slice — independent audit findings

**Framework:** Cryptographic review of sparq's crypto estate (`sq-toze.20`, epic `sq-toze`).
**Under audit:** `compliance/cryptoreview/{README.md, controls.md, evidence.md, gap-register.md}`
on branch `cert-cryptoreview` / draft PR **#235**.
**Auditor posture:** maximally skeptical. This is the single most honesty-critical slice in the
certification set: the one HARD-FAIL condition is any control/statement that presents the ZK or
MPC estate as a delivered cryptographic guarantee. I verified every load-bearing claim against
the ACTUAL repo and the authoritative honesty sources, not against the slice's own assertions.

---

## Authoritative sources cross-checked (read independently, not via the slice)

- `SECURITY.md` (branch) — the canonical published posture. Confirmed it still states the v1 ZK
  verifier is **NOT sound** / *"Treat any 'verified' result from it as untrusted"* and that
  `sparq-mpc` provides **"no confidentiality, correctness, attestation, or malicious-security
  guarantee."** The slice does not override or soften either.
- `research/zk-soundness-audit.md` — verbatim assurance line *"v1 verifier soundness is BROKEN…
  provides NO meaningful soundness guarantee."* Preserved, cited, not laundered (CR-2 ✓).
- `research/zk-verifier-reaudit.md` — the post-remediation re-audit. Read in full: it self-scopes
  to *"the threat model the prior audit assumed,"* self-flags **READ-ONLY** (line 14), single-model
  (`Opus 4.8 (Fable 5 unavailable) — re-review when Fable returns`, line 1), and explicitly holds
  the caveat that the crypto-chain forge tests are `#[ignore]`d (line 40-41). It does **not** claim
  external sufficiency anywhere, and its NEW-1 recommendation is itself "stand up a toolchain-gated
  CI lane." The slice's characterization (internal, necessary-not-sufficient, AUDIT-READY not
  certified) is faithful to the source.

---

## What I independently verified against the live repo (evidence, re-run)

| Claim under audit | How I checked it | Result |
|---|---|---|
| CR-13 containment: all 3 crypto crates `publish = false` | `git show …:crates/{sparq-zk,sparq-zk-compose,sparq-mpc}/Cargo.toml \| grep publish` | **TRUE** — `publish = false` present in all three (lines 12/12/29). |
| CR-13 containment: crypto crates excluded from `sparq-wasm` | Read `crates/sparq-wasm/Cargo.toml` dependency block | **TRUE** — `sparq-wasm` deps are only `sparq-core` + `sparq-engine` (default-features = false). No `sparq-zk` / `-zk-compose` / `-mpc`. |
| Containment: `sparq-engine` `zk` feature can't pull crypto into wasm | Read `crates/sparq-engine/Cargo.toml` features | **TRUE** — `zk = []` is an empty trace-seam feature, NOT in `default = ["parallel","regex","digest"]`, and `sparq-zk` is not a dependency of `sparq-engine` at all. |
| CR-12: insecure RNG gated OFF by default | `crates/sparq-mpc/Cargo.toml` + `src/rng.rs` | **TRUE** — `[features] insecure-test-rng = []` (empty/off); real path is `ChaCha20Rng::from_os_rng()`; `InsecureTestRng` is `#[cfg(any(test, feature = "insecure-test-rng"))]`. |
| CR-3 caveat: forge tests are `#[ignore]`d | `crates/sparq-zk-compose/tests/forge_gates.rs \| grep ignore` | **TRUE** — `forge_pubinput_*` carry `#[ignore = "slow: full bb prove…"]` (lines 563/581/607). Structural forges run in default CI; crypto-chain ones do not. |
| CR-10 binding actually exists (not a phantom check) | `crates/sparq-zk-compose/src/verifier.rs` doc + symbols | **TRUE** — `reconstruct_public_inputs`, byte-equality vs prover `public_inputs` under the verifier's own nonce, `PublicInputMismatch` reject path all present. |
| Re-audit #2 fix (canonical vk, not prover vk) | `crates/sparq-zk-compose/src/driver.rs` `canonical_vk` / `verify_with` | **TRUE** — `verify_with` documented "uses the recomputed canonical vk… NEVER the prover's `art.vk`/`art.public_inputs`." |
| CR-14 Schnorr verify identity-key/subgroup guard | `crates/sparq-zk/src/sig.rs` | **TRUE** — identity key rejected as a binding key; `k.is_zero()` degenerate-R guard present. |
| CR-9 pinned toolchain lane exists + non-floating pins | `.github/workflows/zk-toolchain.yml` | **TRUE** — exists; `NARGO_VERSION: "1.0.0-beta.21"`, `BB_VERSION: "5.0.0-nightly.20260324"`, explicit "never float to latest" comments; merge_group check-run "zk forge + anchor suite (nargo/bb)". |
| CR-11 Tier-A release evidence is real | `.github/workflows/release.yml` | **TRUE** — `attest-build-provenance@a2bbfa25…` (SHA-pinned v4.1.0) over archives + SBOM + VEX; `sha256sum -- * > SHA256SUMS`; `provenance: mode=max` on the image. |
| CR-G1 bead `sq-qhy4` is real, OPEN, P0, external | `bd show sq-qhy4` | **TRUE** — `○ sq-qhy4 · [cert][cryptoreview] External accredited-cryptographer audit … (REQUIRED before any production ZK security claim) [P0 · OPEN]`. |
| The two "new" gap beads exist | `bd show sq-cu32` / `sq-egx6` | **TRUE** — both OPEN (FIPS P2; constant-time P2). |

### The four "honesty-test" trip conditions the slice defines for itself — all PASS

1. **CR-4 / CR-5 are NOT marked PASS.** Confirmed: CR-4 = `NOT-SOUND (research-only) / EXTERNAL-REQUIRED`; CR-5 = `NOT-SOUND (research-only)`. No row upgrades the scaffold to a guarantee.
2. **The internal re-audit (CR-3) is NOT cited as external certification.** CR-3 status is `AUDIT-READY`, with all three caveats (single-model, read-only, `#[ignore]`d forge tests) spelled out; README §2 and the gap-register HEADLINE both state the published "NOT sound" posture governs until CR-G1 closes.
3. **No maturity/score statement implies a disclaimed guarantee.** The slice carries no CDMC/maturity number for the ZK/MPC estate; it adds an explicit cross-framework auditor note forbidding any other framework from doing so.
4. **CR-G1 is NOT omitted or buried.** It is the README's "single most load-bearing sentence" (§0), the gap-register HEADLINE (before the gap table), and the first/CRITICAL row of the gap table. It is headlined, not softened.

---

## Findings

### Finding 1 — LOW (hygiene, not a substantive overclaim): the new cert beads are not yet exported to the committed `.beads/issues.jsonl`

- **Control:** the bead-traceability convention that underpins CR-G1/CR-G4/CR-G5 (`controls.md`,
  `gap-register.md` cite `sq-qhy4`, `sq-cu32`, `sq-egx6`).
- **What I checked:** `git show origin/cert-cryptoreview:.beads/issues.jsonl | grep '"id":"sq-qhy4"'`
  (and `sq-cu32`, `sq-egx6`) on both the branch and `main` → **0 matches** for each; for contrast
  `sq-gbp4`, `sq-f9tl`, `sq-61g`, `sq-c5f`, `sq-hyhj`, `sq-1gir`, etc. are all present. I then ran
  the live tracker: `bd show sq-qhy4` / `sq-cu32` / `sq-egx6` → **all three exist, OPEN, with the
  exact titles the slice describes** (sq-qhy4 is P0 external-audit; the other two P2).
- **Why it is only LOW, not a finding against the headline:** the beads are real and correctly
  prioritized in the source of truth (the `bd` database). The slice does NOT overclaim — the
  external-audit requirement is genuinely tracked as an OPEN P0. The only defect is that the
  branch's checked-in JSONL snapshot predates today's (2026-06-16) bead creation, so a reader who
  verifies traceability from the committed file alone (rather than `bd`) cannot resolve three IDs.
- **Remediation (orchestrator, not the engineer's code):** re-export `bd` to `.beads/issues.jsonl`
  on `cert-cryptoreview` so `sq-qhy4`, `sq-cu32`, `sq-egx6` appear in the committed snapshot before
  merge. Per project convention agents do not hand-edit `issues.jsonl`; the orchestrator re-exports.
  No code/source change and no slice-doc change is required. This does not gate sign-off.

**No high or critical finding.** Specifically: **no soundness-overclaim was found.** No control,
evidence line, score, or gap statement presents the ZK or MPC estate as a delivered cryptographic
guarantee; every reference to the favourable internal re-audit is explicitly walled off from the
published "NOT sound" posture; the Tier-A/Tier-B split holds and Tier-B never borrows Tier-A
assurance.

---

## Coverage note

**Assessed (verified against repo or authoritative source):** CR-1 (inventory), CR-2 (original
audit preserved), CR-3 (re-audit calibration), CR-4 (ZK disposition), CR-5 (MPC disposition),
CR-6 (forge `#[ignore]` caveat), CR-8/CR-9 (gate ratchet + pinned toolchain lane), CR-10
(reconstruct binding presence), CR-11 (Tier-A primitives + release attestation), CR-12 (RNG gate),
CR-13 (containment: publish=false + wasm exclusion + engine zk-feature), CR-14 (Schnorr verify
guards present; CT-analysis-absent claim is honest), CR-15 (FIPS negative claim), CR-16 / CR-G6
(privacy residuals tracked). Re-audit #2 vk-fix spot-checked in `driver.rs`.

**Assessed at the documentary level only (could not fully execute, and why):**
- The forge suite and the empirical bb anchors (CR-6/CR-7/CR-10) were verified to **exist and be
  `#[ignore]`d/toolchain-gated** but I did **not** run `bb prove`/`bb verify` end-to-end — the
  nargo/bb toolchain is not provisioned in this audit environment, which is exactly the CR-G2/CR-G3
  condition the slice already discloses. This is not a new gap; it is the disclosed one.
- The substantive *cryptographic soundness* of the remediated verifier is **out of agent scope by
  construction** (CR-G1). I verified the binding gates exist in code and that the slice does not
  claim soundness — I did **not**, and cannot, certify the math. That is precisely what the external
  accredited cryptographer (`sq-qhy4`, P0) must do.

---

## Verdict

The cryptographic-review slice is honest, conservative, and accurately calibrated. It does not
present ZK or MPC as a delivered guarantee; it preserves `SECURITY.md`'s "NOT sound / no MPC
guarantee" posture; it correctly records the internal re-audit as necessary-but-not-sufficient
progress, **not** as certification; the `publish = false` + wasm-exclusion containment is verified
TRUE in the manifests themselves; and CR-G1 (the P0 external-cryptographer-audit requirement,
bead `sq-qhy4`, OPEN) is properly headlined in three places. The single finding is a LOW
JSONL-export-lag hygiene item for the orchestrator and does not gate sign-off.

**SIGN-OFF** — with the standing caveat that the external-accredited-cryptographer audit (CR-G1 /
`sq-qhy4`) and any formal verification of `reconstruct_public_inputs` ↔ circuit-`main`
correspondence remain **external** items an agent cannot substitute for; this sign-off attests to
the *honesty and accuracy of the readiness slice*, not to the cryptographic soundness of the
Tier-B estate (which `SECURITY.md` continues to disclaim).

FINDINGS: 1 (LOW; non-blocking)

> 🤖 SPARQ agent — independent cryptographic-review auditor (Opus 4.8, 1M context). Re-review when
> Fable returns. References PR #235, epic `sq-toze`.
