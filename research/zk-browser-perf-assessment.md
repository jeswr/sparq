# In-browser ZK age-gate: honest performance assessment (slow proving, large proof)

Status: **diagnostic assessment, June 2026.** [OPUS-4.8] Synthesises five parallel
investigations (backend-flavour + proof-size, empirical Node measurement, circuit,
setup/SRS, threading/cross-origin-isolation) into a single ranked verdict for the
in-browser age-gate prover at `site/src/lib/zk-prover.ts` (the
`proveAgeEligibility` / `loadProver` / `prewarmProver` path, PR #372). It proves the
car-hire age-gate sub-circuit `zk/compose/filter_int_d2` (committed ACIR at
`site/public/zk/filter_int_d2.json`) with `@noir-lang/noir_js` +
`@aztec/bb.js` (Barretenberg `UltraHonkBackend`) entirely in the tab.

## 0. Honesty boundary (read first)

- The sparq ZK estate is **research-grade, internally reviewed only, and NOT
  externally audited** (bead `sq-qhy4`). Nothing below may weaken **what the proof
  proves**, the **zero-knowledge (age-hiding) property**, or the not-externally-audited
  caveat. Every recommendation here is a **perf / config / presentation** lever only.
- All timing numbers in this document are **indicative, measured in Node on this EC2
  work box (kernel `-aws`), and NON-canonical** — they are not browser numbers and must
  not be baked into docs, tests, or the page's honesty panel. They were taken on the
  demo's exact circuit + a representative witness (age = 30) with the exact pinned
  toolchain (`@noir-lang/noir_js@1.0.0-beta.21`, `@aztec/bb.js@5.0.0-nightly.20260324`).
  Proof-size numbers are deterministic (thread-independent) and reproduce exactly.
- The measurement runs left the worktree clean: scratch scripts were written under
  `/tmp` and deleted, and `package-lock.json` was restored.

## 1. The two complaints, root-caused

Both maintainer complaints are real and **each has a clean, proof-preserving root
cause**. Critically, they are independent: the *slow* axis is the deployment host
forcing single-threaded WASM; the *large* axis is inherent to UltraHonk plus one
cosmetic mislabel.

| Complaint | Dominant root cause | Class | Fixable? |
| --- | --- | --- | --- |
| Proving is **much slower than expected** | Forced single-thread on GitHub Pages (no cross-origin isolation → no `SharedArrayBuffer` → `maxThreads()` returns 1) | misconfig / host constraint | Partly — deployment lever, ~4x available |
| Proof is **much larger than expected** | (a) UltraHonk proofs are inherently KB-scale, not SNARK-scale; (b) a cosmetic UI mislabel makes the number read ~32x too big; (c) a ZK-preserving flavour can shave ~43% | (a) inherent, (b) error, (c) config | (b) trivially, (c) low-risk, (a) no |

## 2. Ranked findings

Ranked by **impact x confidence x safety**. Findings 1–3 are actionable; 4–6 set
expectations (inherent — not bugs).

### Rank 1 — Forced single-threaded proving on GitHub Pages (SLOW; the dominant lever)

- **Class:** misconfig surfaced honestly / inherent host constraint.
- **Evidence:** `maxThreads()` (`site/src/lib/zk-prover.ts:152-156`) returns `1` unless
  `window.crossOriginIsolated` is true. bb.js worker-thread fan-out is gated on
  `SharedArrayBuffer`, which requires cross-origin isolation (COOP `same-origin` +
  COEP `require-corp`/`credentialless`). **GitHub Pages cannot set those response
  headers**, so the deployed demo is permanently single-threaded.
- **Empirical (indicative, Node, non-canonical):** same circuit + witness, warm
  `generateProof`, default flavour:

  | threads | prove (ms) | verify (ms) |
  | --- | --- | --- |
  | 1 | ~1285-1295 | ~174 |
  | 2 | ~766 | — |
  | 4 | ~474 | — |
  | 8 | ~306-358 | ~63 |

  Proof bytes were **identical (14656 B)** at every thread count — threads change only
  speed, never the proof. Measured speedup t=1 → t=8 ≈ **4.2x** (warm).
- **Impact:** On Pages (t=1) the user pays roughly the full single-threaded cost; on an
  8-core client that is ~4x slower than the multithreaded path. This is the dominant
  contributor to "much slower than expected". Browser WASM will be somewhat slower than
  this Node figure, but the **threads axis is the same lever**.
- **Fix (config/deployment only, never touches the proven statement):**
  1. **Accept + pre-warm** (already partly done via `prewarmProver`) — zero risk, no speedup.
  2. **`coi-serviceworker` shim** to synthesise cross-origin isolation on Pages so
     `window.crossOriginIsolated` becomes true and `maxThreads()` returns up to 8 —
     restores ~4x. Adds a service-worker dependency.
  3. **Host the ZK route on a surface that can set COOP/COEP** — restores ~4x, larger move.
- **Effort:** medium (shim) / large (re-host). **Risk:** low-to-medium — changes only the
  thread count, never the proof semantics or the not-externally-audited caveat. **Expected
  gain:** up to ~4x faster proving on a multicore client; **none on plain Pages without a shim**.

### Rank 2 — Proof byte count is mislabelled "fields" in the UI (LARGE-perception; trivial fix)

- **Class:** genuine error (cosmetic).
- **Evidence:** `site/src/components/zk-car-hire.tsx:386`
  `<Stat label="Proof size (fields)" value={r.proofByteLength.toString()} />`. The value
  is `proof.proof.length`, which is **bytes (= fields x 32)**, surfaced as `proofByteLength`
  in `zk-prover.ts:205`. A ~14.6 KB proof therefore renders as a `14656` "fields" number,
  which reads as absurdly large for a field count (the true field count is ~458).
- **Impact:** Misleading UI; a very likely contributor to the "proof much larger than
  expected" impression. **Affects no proof bytes and no security property** — pure
  presentation.
- **Fix:** relabel to "Proof size (bytes)", or report `proofByteLength / 32` under a
  "fields" label, or show `${(len/1024).toFixed(1)} KB`. Touches no proving / verifying code.
- **Effort:** trivial. **Risk:** low. **Expected gain:** removes a false ~32x perception of
  bloat; no byte change.

### Rank 3 — Default UltraHonk flavour larger than a ZK-preserving alternative (LARGE; ~43% available)

- **Class:** inherent default-config choice (not a bug); a smaller ZK-preserving flavour
  is reachable via an unused options arg.
- **Evidence:** `zk-prover.ts:196` calls `backend.generateProof(witness)` with **no
  options**, which resolves to the default poseidon2-oracle, full-ZK UltraHonk flavour.
  Measured proof sizes (deterministic, thread-independent), same circuit + witness:

  | flavour (options) | bytes | fields | ZK preserved? |
  | --- | --- | --- | --- |
  | default (poseidon2, ZK) | 14656 | 458 | yes |
  | `verifierTarget: 'noir-recursive'` (poseidon2, ZK) | 14656 | 458 | yes |
  | `verifierTarget: 'evm'` (keccak, `disableZk:false`) | 8384 | 262 | **yes** |
  | `evm-no-zk` (keccak, `disableZk:true`) — **UNSAFE** | 7424 | 232 | **NO** |
  | `noir-recursive-no-zk` (`disableZk:true`) — **UNSAFE** | 13120 | 410 | **NO** |

  VK is 3680 B; 5 public inputs in all cases. At t=8, `evm` proved in ~301-302 ms and
  verified in ~55-67 ms — **no measured time penalty vs default**.
- **Impact:** Switching the default to `{ verifierTarget: 'evm' }` cuts the wire proof
  **14656 → 8384 bytes (1.75x / ~43% smaller)** with no measured time cost, while keeping
  `disableZk:false` (full zero-knowledge).
- **Fix (config-only, proof-preserving):** pass `{ verifierTarget: 'evm' }` to **both**
  `generateProof(witness, opts)` **and** `verifyProof(proof, opts)` in `proveAgeEligibility`
  (and any `getVerificationKey` path), then re-confirm `verified: true` and re-measure.
- **Effort:** low. **Risk:** medium and **must be weighed**:
  - **Only the `disableZk:false` keccak flavour (`verifierTarget:'evm'`) is acceptable.**
    The `*-no-zk` variants are smaller still but **MUST NOT be used** — they drop the ZK
    masking and would weaken what the proof hides about the private age. We verified the
    current default proof **is randomised** (two proofs of the same witness have different
    `sha256`, so masking is present) and the age digits are **not** among the 5 public
    inputs (`challenge`, `operand_enc`, `op=3/GE`, `bound=25`, `expected=1`). A `disableZk`
    proof would lose that.
  - `evm`/keccak is *intended* for EVM verification; for the in-tab self-verify demo,
    **prove and verify must pass the same options** (`verified:true` confirmed in
    measurement).
  - This is a **bb.js nightly**; the `verifierTarget` API surface can shift between
    versions — **pin and re-test on any upgrade**.
- **Expected gain:** ~43% smaller proof, no time penalty, ZK fully preserved.

### Rank 4 — UltraHonk proofs are inherently KB-scale (INHERENT; set expectations)

- **Class:** inherent to UltraHonk. Not a bug, not fixable by a flavour swap.
- **Evidence:** a poseidon2 UltraHonk proof for this circuit is ~458 BN254 field
  elements x 32 B ≈ **14.6 KB raw**; the keccak flavour is ~262 fields ≈ 8.4 KB. This is
  **orders of magnitude** larger than a Groth16 ~200-byte proof. UltraHonk has **no
  compact mode**; the only way to a SNARK-scale single proof is **recursive aggregation**
  (a larger architectural change, out of scope here).
- **Impact:** If the expectation was "SNARK-scale (~200 B)", the proof will *always* look
  large. The honest framing is: this is the cost of a transparent, no-trusted-setup
  UltraHonk proof, and the realistic floor here is ~8.4 KB (keccak, Rank 3), not ~200 B.
- **Fix:** none for soundness — **document the expectation**. Recursion/aggregation is the
  only real size lever and is a separate, larger piece of work.
- **Effort:** n/a (detection). **Risk:** n/a. **Expected gain:** correct expectations, no byte change.

### Rank 5 — ZK masking (`disableZk:false`) adds proving work (INHERENT; do NOT remove)

- **Class:** inherent and **required**.
- **Evidence:** the default and `verifierTarget:'evm'` flavours keep `disableZk:false`,
  adding proof-hiding/masking rows. The `*-no-zk` variants drop them.
- **Impact:** masking costs some prove time and proof size — but it **is** the
  zero-knowledge property. Removing it (any `*-no-zk` flavour) would make the private age
  recoverable in principle and **breaks the whole point of the demo**.
- **Fix:** keep `disableZk:false`. **Do not** trade it for speed or size.
- **Effort:** n/a. **Risk:** **high if removed** (breaks the privacy claim on a
  not-externally-audited estate, `sq-qhy4`). **Expected gain:** n/a — this is a guardrail.

### Rank 6 — Classic proof-inflation bugs are ABSENT (INHERENT; ruled out)

- **Class:** detection — confirms the wire proof is already lean for its flavour.
- **Evidence (from bb.js `5.0.0-nightly.20260324` `backend.js`):**
  - **No hex-doubling:** `generateProof` returns `proof.proof` as a **raw** `Uint8Array`
    (`fields x 32` bytes); `uint8ArrayToHex` is applied only to `publicInputs`, never the
    proof. `zk-prover.ts:205` reports the raw byte length — the true wire size.
  - **No VK on the wire:** `generateProof` passes `verificationKey: new Uint8Array(0)` and
    returns only `{proof, publicInputs}`; the VK (itself several KB) is recomputed lazily
    at verify time, never attached.
  - **No public-input inflation:** only **5** small public inputs, returned **separately**
    (not prepended into the proof) by this nightly.
  - **No recursion/IPA flag** set on the default path.
- **Impact:** rules out the easy "it's accidentally 2x / VK-bundled / input-bloated"
  explanations. The size is the genuine UltraHonk cost (Rank 4) minus the Rank 3 saving,
  nothing more.
- **Fix:** none — current behaviour is correct. **Effort/Risk:** n/a.

## 3. The single highest-confidence, low-risk, high-impact fix to do FIRST

Following the data rather than the prior: the brief guessed cross-origin isolation, but
**that fix is medium-risk (service-worker shim) and yields nothing on plain Pages without
extra infrastructure**. The evidence points instead to **Rank 2 — relabel the UI
"fields" → bytes/KB** as the first fix:

- **Highest confidence:** it is a verified, exact mislabel at one line
  (`zk-car-hire.tsx:386`).
- **Lowest risk:** pure presentation; touches **no** proving, verifying, circuit, or
  caveat code — it **cannot** weaken proof correctness or the ZK property.
- **High impact on the actual complaint:** the complaint is *perception* ("much larger than
  expected"); a `14656`-as-"fields" readout overstates the size ~32x. Correct labelling
  removes the single largest source of the false impression instantly.

Rank 3 (`verifierTarget:'evm'`, a real ~43% byte reduction) is the strongest *substantive*
size win and is low-effort, but it carries flavour-semantics + nightly-API risk and needs a
verify-side change plus re-measurement, so it should land as a **reviewed follow-up**, not the
zero-deliberation first move. Rank 1 (threads / cross-origin isolation) is the dominant *speed*
lever but is medium-effort and ineffective on plain Pages without a shim.

## 4. Bead candidates (other fixes)

- **`verifierTarget:'evm'` keccak flavour for a ~43% smaller, still-ZK proof** — pass the
  option to both `generateProof` and `verifyProof`, keep `disableZk:false`, confirm
  `verified:true`, re-measure, pin the bb.js nightly and re-test on upgrade. (Rank 3.)
- **Cross-origin isolation for bb.js multithreading (~4x faster proving)** — evaluate a
  `coi-serviceworker` shim on Pages, or re-host the ZK route where COOP/COEP can be set.
  Config/deployment only; no proof-semantics change. (Rank 1.)
- **Document the inherent UltraHonk proof-size floor** in the honesty panel / SECURITY
  notes — set the "KB not bytes" expectation; note recursion/aggregation as the only
  sub-KB path and that it is out of scope. (Rank 4.)
- **Guardrail note: never adopt a `*-no-zk` flavour** — record in the ZK skill / SECURITY
  notes that `disableZk:true` is forbidden because it strips the age-hiding property.
  (Rank 5.)

## 5. What was measured vs. what was reasoned

- **Measured (Node, this box, non-canonical):** thread-count → prove/verify time table
  (Rank 1); per-flavour proof byte/field sizes (Rank 3); proof randomisation across two
  runs of the same witness; the 5 public inputs and absence of age digits among them.
- **Reasoned from source (deterministic, version-pinned):** the default flavour selection
  with no options; raw-bytes (no hex-doubling), VK-free, separate-public-inputs wire format
  (Rank 6); `maxThreads()` gating on `window.crossOriginIsolated`; the UI mislabel at
  `zk-car-hire.tsx:386`.
- **Not measured:** real browser WASM timings (would require a Pages-isolated harness);
  these would be **slower** than the Node figures but driven by the **same** threads lever.
