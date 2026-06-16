<!-- [OPUS-4.8] sq-toze / sq-toze.20 — Cryptographic-review gap register. Honesty-critical.
     Authored by Opus 4.8 while Fable 5 unavailable — re-review when Fable returns. -->

# Cryptographic review — gap register

Open gaps for the cryptographic-review framework, severity-ordered. **The headline is not a
list of small holes — it is a single architectural truth (CR-G1) that gates every production
ZK/MPC security claim.** Each gap names its remediation + a `bd` bead (epic `sq-toze`; the ZK
remediation itself lives under epic `sq-1s2`).

---

## HEADLINE — read this before anything else

> ### sparq's bespoke cryptography is NOT externally verified.
>
> - The **v1 ZK verifier** (`sparq-zk` / `sparq-zk-compose`) is published in `SECURITY.md` as
>   **NOT sound** — *"treat any 'verified' result from it as untrusted."* An internal
>   post-remediation re-audit found it *"sound as landed,"* but that is a **single-model,
>   internal, self-review** (Opus 4.8, Fable unavailable, read-only, crypto-chain forge tests
>   `#[ignore]`d) — **necessary but NOT sufficient** to support a production soundness claim.
> - **`sparq-mpc`** provides **NO confidentiality, correctness, attestation, or
>   malicious-security guarantee** — the malicious-security / collaborative-proof core is
>   **DEFERRED** behind honest `NotYetImplemented` stubs.
> - **An external accredited-cryptographer audit (`CR-G1` / bead `sq-qhy4`, P0) is REQUIRED
>   before ANY ZK/MPC security, privacy, integrity, or attestation property may be relied
>   upon in production.** No internal document — including the favourable re-audit, this
>   compliance slice, or any CDMC/maturity score — substitutes for it.
>
> Containment (honest mitigant, NOT a substitute for the audit): all three bespoke-crypto
> crates are `publish = false` and excluded from `sparq-wasm`, so a downstream consumer of the
> published artifacts cannot reach this code by default (CR-13). The gap is real but
> *contained to an opt-in research surface*.

---

## Gap table

| ID | Gap | Severity | Disposition | Remediation | Bead |
|---|---|---|---|---|---|
| **CR-G1** | **No external accredited-cryptographer audit** of the ZK verifier binding layer, the Schnorr-over-Baby-JubJub issuer-signature scheme, the Noir circuits, or the composition seam. All soundness assurance is internal self-review. | **CRITICAL** | **EXTERNAL-REQUIRED** (out of agent scope) | Commission an external accredited cryptographer to re-run the 12-finding adversarial pass + (ideally) a machine-checked formal verification of `reconstruct_public_inputs` ↔ circuit-`main` correspondence. This slice + `research/zk-soundness-audit.md` + `research/zk-verifier-reaudit.md` + `forge_gates.rs` are the evidence pack. **Until it closes, `SECURITY.md`'s "NOT sound" posture is the relying-party truth.** | `sq-qhy4` (P0) |
| **CR-G2** | **Crypto-chain forge tests are `#[ignore]`d** out of default CI (need nargo/bb), so closure of the 12 findings rested on code-reading + 2 anchors, not default-CI execution. | HIGH | **ADDRESSED (verify before relying)** | A toolchain-gated CI lane now runs the `#[ignore]`d `forge_pubinput_*` + real-bb e2e suite on zk changes (`.github/workflows/zk-toolchain.yml`, `zk forge + anchor suite (nargo/bb)`). Confirm the lane is green and required-gated as part of CR-G1's external pass. | `sq-f9tl` (**closed**) |
| **CR-G3** | **Public-input empirical anchors missing for `filter_f64_d*` + k=2 scan members** — only `filter_int_d1` + `scan_k1_n16_r4` had non-circular golden vectors; the rest relied on layout reasoning, so a toolchain bump could silently drift their serialization. | HIGH | **ADDRESSED (verify before relying)** | Probe-captured anchors for the f64 + k=2 members landed under the same toolchain-lane bead. Verify the anchors exist + the lane recaptures-and-compares them. | `sq-f9tl` (**closed**) |
| **CR-G4** | **No FIPS 140-3 validated module; no stated FIPS posture** for an operator with a FIPS deployment constraint. | LOW | **AUDIT-READY** (honest negative claim) | The bespoke ZK primitives are non-FIPS-approved by design; SHA-256 digests are FIPS 180-4 but outside any CMVP module. Add a short, explicit FIPS-scope note to `SECURITY.md` / the crypto crate READMEs so the absence is documented, not implied. | `sq-cu32` (P2, **new**) |
| **CR-G5** | **No constant-time / side-channel analysis** of the secret-bearing paths (issuer signing, MPC share ops); no `dudect`/`ctgrind` measurement; arkworks BN254 field ops not asserted CT. | P2 | **OPEN** | Present exposure is LOW (verify path is over public data; mask draw is effectively-CT-in-expectation). Run a `dudect`/`ctgrind` pass on `sig.rs` signing + the MPC share ops; document accepted leakage. Becomes load-bearing if the in-circuit hidden-key upgrades (`sq-1s2`) grow the secret surface. | `sq-egx6` (P2, **new**) |
| **CR-G6** | **Residual ZK privacy / binding deferrals** (not soundness breaks): in-circuit salt binding not done (salt verified verifier-side-clear); per-graph salt + list-IRI/version disclosed in the clear (linkability handles); `HolderPop` not yet credential-bound (a trusted holder could present another trusted holder's credential). | MEDIUM (privacy) | **OPEN** (tracked) | The in-circuit privacy upgrades. These are explicitly documented deferrals (`research/zk-verifier-reaudit.md` NEW-1/NEW-2), **not** soundness re-openings — a revoked/forged credential still fails; an untrusted holder still fails. | `sq-hyhj`, `sq-93h`, `sq-i1dt`, `sq-42e3` (all open, under `sq-1s2`) |
| **CR-G7** | **`sparq-mpc` malicious security / collaborative-proof (M4, Q1) deferred** — no malicious-security, no distributed-signature-over-secret-shared-witness. | HIGH (for any MPC reliance) | **NOT-SOUND (research-only)** / deferred | Out of scope for this certification epic; the MPC build-out is tracked separately. The honest statement (`SECURITY.md` + `sparq-mpc` manifest) must accompany any mention of MPC. | `sq-bjl` (deferred SPIKE), `sq-34ml` (M4-v1 prerequisites), `sq-ox16` (covert/PVC model) |

---

## What is explicitly NOT a gap (and must not be re-raised as one)

To prevent re-litigation, these are the controls already in place — cite them as evidence, do
not list them as gaps:

- The 12 historical CRITICAL/HIGH findings each have an internal **CLOSED** disposition with a
  reject-path + (mostly) a forge test — `research/zk-verifier-reaudit.md` §"Per-finding
  dispositions" + the 1:1 map. (Closure is *internal*; CR-G1 is the external gate over all of
  it.)
- The forge-and-verify negative suite (`forge_gates.rs`, `sq-ajl`), the differential fuzzer
  (`differential_fuzz.rs`, `sq-61g`), the gate-count ratchet (`gate_count.rs`, `sq-c5f`), and
  the pinned toolchain lane (`zk-toolchain.yml`) all exist and are cited in `controls.md`
  CR-6/7/8/9.
- Sigstore build-provenance attestation, SHA-256 digests, RDFC10, and OS-CSPRNG sourcing are
  **Tier-A sound** (CR-11) — they are not part of the unsound surface and need no remediation.
- Containment (`publish = false` + wasm exclusion, CR-13) is a mitigant, not a gap.

---

## Severity rationale (honesty calibration)

- **CR-G1 is CRITICAL not because the verifier is known-broken today** (the internal re-audit
  argues it is sound as landed) **but because the soundness claim is unverified by anyone
  outside the project** — and for a credential system, "trust us, we audited ourselves" is not
  an acceptable basis for production reliance. The severity reflects the *assurance* gap, which
  is exactly what an external auditor closes.
- **CR-G7 (MPC) is HIGH for any MPC reliance** but the honest framing is "explicitly deferred,
  no guarantee claimed" — there is no *overclaim* to correct, only a deferral to keep visible.
- The privacy residuals (CR-G6) are **MEDIUM** because they are linkability/binding gaps in a
  system whose *soundness* (the load-bearing property) is separately argued — a privacy leak is
  serious for a privacy-preserving system but does not let a forger pass a false result.

> **Auditor note.** If any future framework (e.g. CDMC, privacy) assigns the ZK/MPC estate a
> maturity score implying it provides a guarantee it disclaims, or cites the internal re-audit
> as external certification, that is a high-severity cross-framework finding — flag it against
> this register's HEADLINE. The most load-bearing honesty item in the repo is the documented
> "v1 ZK verifier is NOT sound" verdict; this register exists to keep it un-laundered.
