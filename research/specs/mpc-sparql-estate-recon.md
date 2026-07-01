# MPC-SPARQL Estate Recon

> Grounding for the MPC-SPARQL Proposed Spec (bead sq-rvgr2.3). Read-only recon; not itself normative.

## Summary

The MPC-over-federated-SPARQL estate (epic sq-pwr, parent sq-0jsc) is one of the most
densely-developed tracks in the repo: M0–M3 of sparq-mpc plus the whole single-prover ZK
binding layer in sparq-zk-compose are MERGED and tested. ~10 design records (headed by
`research/mpc-zkp-federated-sparql-design.md`) already lay out the four-guarantee framing,
six-step protocol flow, threat model, and milestone/bead spine.

The load-bearing honesty fact is that the **collaborative ZK proof itself**
(`crates/sparq-mpc/src/proof.rs` `prove`/`verify`, `Attestation`) is an honest
`NotYetImplemented` stub — the pipeline assembles the `ProofStatement` shape but emits **NO
proof**, so guarantee C (attested-source) is DESIGNED-not-built at the federated/MPC layer.

Every production soundness/attestation/privacy claim is HARD-GATED on the external
accredited-cryptographer audit **sq-qhy4** (P0, OPEN); the estate's soundness today rests
entirely on internal single-model (Opus 4.8) self-audits.

---

## What Is Built and Tested

- **Honest-majority Shamir t-of-n backend** over F_p = 2^61−1 Mersenne prime:
  `crates/sparq-mpc/src/field.rs:34`, `shamir.rs:98` (`Share{x: party idx ≥1, y: Fp}`),
  `ShamirBackend`/`MacSession`; CSPRNG masking via ChaCha20 (`rng.rs`/`chacha.rs`, sq-1vt);
  insecure deterministic RNG only behind a test feature.

- **BGW degree reduction** — `shamir.rs::degree_reduce`, sq-dvuc CLOSED
  (`research/mpc-zkp-build-out-delta.md:55-59`).

- **Disclosed-key global-IRI equi-join** (crypto-free) and **hidden-value join** via
  secret-shared field equality: `crates/sparq-mpc/src/join.rs` (`DisclosedKeyJoin`,
  `HiddenValueJoin`), `lib.rs:335`.

- **Secure comparison** that opens ONLY the boolean verdict bit, never the operands:
  `crates/sparq-mpc/src/compare.rs` (`disclose_threshold_verdict`, `secure_greater_than`,
  `secure_equal_to_bit`); Rabbit-style full-field decomposition to 2^60 (`lib.rs:314–326`).

- **Malicious-secure (honest-majority, with-abort) twins** partially built: IT-MAC
  authenticated sharing foundation `authenticated.rs` (sq-km34.1), `auth_compare.rs`
  (sq-ka8m), `auth_disclose.rs` (sq-6fv7) — MAC-checked verdict before open at minimal
  n=2t+1 (`lib.rs:131–151`).

- **Oblivious shuffle** (Waksman/Benes) + **sort** (Batcher odd-even mergesort) and
  set-returning oblivious output path: `oblivious.rs`, `oblivious_join.rs` (sq-18lk/sq-jnkm,
  `lib.rs:217–233`).

- **Robust Reed-Solomon / Berlekamp-Welch reconstruction** (detect/correct tampered shares):
  `robust.rs` (sq-m34i, `lib.rs:211–215`).

- **Bounded-length property paths**: disclosed-key crypto-free unroll (`bounded_path.rs`) and
  hidden-intermediate exactly-k chain (`hidden_path.rs`) + hidden-key DISTINCT
  (`hidden_distinct.rs`) + planner guard (sq-py8h.*).

- **Term→Fp domain-separated join-key encoder**: `encode(term) =
  reduce_mod_p(SHA-512(DOMAIN_TAG || ntriples(term)))`,
  `DOMAIN_TAG="sparq-mpc/term-join-key/v1\0"` (`term_encode.rs:87,98`); stated birthday bound
  q²/2^62; fail-closed `KeyEncoder` collision-detection path (sq-dl81).

- **Three-axis configurable security descriptor** `AdversaryModel × OutputGuarantee ×
  CorruptionThreshold` + `PublicVerifiability`, Cleve impossibility enforced as a type-level
  invariant, per-operator `OperatorClass` reporting: `backend.rs:106–180`, `lib.rs:279–283`
  (sq-mq8q).

- **End-to-end federated pipeline driver** for the four-flatmates scenario composing
  `holder → share → disclosed-key join → secure sum → threshold-verdict → ProofStatement`,
  with explicit per-operator Disclosed/Hidden routing and a differential-vs-union-store
  correctness anchor: `crates/sparq-mpc/src/pipeline.rs:233 run_federated` (sq-6y92).

- **Network-tier loopback transport** (Tier 2): star-coordinator, each party a process
  holding only its Shamir column, hand-written length-prefixed wire codec with
  `Message{Shares,Open,Step,Swap,Done}` and `StepCode{SumAndOpen=1,MulAndOpen=2,
  SwapNetworkAndOpen=3}`, field elements as canonical u64 big-endian (8 bytes):
  `transport.rs:88–151` (sq-tg6b).

- **In-process benchmark matrix harness** with deterministic communication/round/multiplication
  counters (Tier 1): `bench.rs`, `metrics.rs` (sq-sxm).

- **Single-prover ZK verifier** public-input reconstruction binding challenge/nonce +
  commitments + FILTER operands + per-row attribution:
  `crates/sparq-zk-compose/src/verifier.rs:4786 reconstruct_public_inputs`.

- **Single-prover issuer-attestation binding** anchored on an EXTERNAL relying-party key-set K:
  `verifier.rs:2242 bind_issuer_attestations`.

- **Freshness/replay**: single-use verifier nonce with burn-on-mismatch and
  `NonceBindingMismatch`, nonce bound as public-input field 0 of every sub-proof:
  `verifier.rs:4601–4644` (sq-3v2/#4).

- **Untrusted-planner → MPC routing seam** crate `sparq-fedplan-mpc`:
  `privacy.rs`, `selection.rs`, `routing.rs`, `envelope.rs`, `seam.rs`, `combination.rs` —
  privacy-aware default-deny source selection + disclosed/hidden `route_operators` + leakage-
  envelope dual ratification (navigator §3.4, all DONE).

- **Adversarial test suites**: `witness_validation_tests.rs` (sq-7leq CLOSED),
  `owa_omission_tests.rs` (sq-2fms), `adversarial_tests.rs` (sq-nuok).

---

## Designed, Not Built

- **Collaborative ZK proof over a secret-shared witness** (`prove`/`verify`) —
  `crates/sparq-mpc/src/proof.rs:135–149`: both methods return `MpcError::NotYetImplemented`;
  `Proof` (`proof.rs:155`) is a deliberately field-less opaque placeholder (proof system
  Noir/UltraHonk vs collaborative-SNARK unchosen).

- **Distributed issuer-signature attestation over a secret-shared witness** —
  `proof.rs:94–116 Attestation` + `AttestationShare` placeholder (carries NO fields at M0);
  the unsolved Q1 "join nobody has built" (sq-bjl DEFERRED,
  `research/mpc-m4-distributed-sig-feasibility.md §1`).

- **M4-v1 verifier-side authenticated-input attestation gate** (Dutta eprint 2022/1648 +
  Artemis arXiv 2409.12055 commit-and-prove anchor) — bead sq-f7bu OPEN, no code
  (`research/mpc-zkp-build-out-delta.md M-C`).

- **Federated / multi-source `reconstruct_public_inputs` layout** — bead sq-34ml OPEN
  (blocks sq-f7bu); today `reconstruct_public_inputs` and `bind_issuer_attestations` are
  single-prover / single-manifest.

- **Full honest-majority IT-MAC malicious-with-abort** — beads sq-km34.2–.9 OPEN;
  design `research/mpc-malicious-security-design.md`.

- **ORQ-style O(n log n) oblivious sort-merge / linear circuit-PSI join** to replace the
  all-pairs hidden join — beads sq-ujz8/sq-t21/sq-h99 OPEN (navigator §3.1).

- **In-circuit distributed signature** over secret-shared witness (source-unlinkable
  single-proof attestation, the thesis novelty) — sq-bjl DEFERRED spike, unsolved in the
  literature.

- **Dishonest-majority (SPDZ/MASCOT/Overdrive) backend + WAN constant-round family + DP
  output-cardinality** with epsilon budget — beads sq-j5ok/sq-38zk/sq-shk5/sq-4i39 OPEN;
  registry FAIL-CLOSES rather than downgrading.

- **Fresh coZK soundness re-audit** of the collaborative path vs eprint 2025/1026 —
  `research/mpc-cozk-reaudit.md` exists (sq-9hrn) but is an adversarial risk-surfacing pass,
  NOT a soundness certificate.

- **`secprop` property-admissibility pre-check in the MPC-SPARQL federated admission path** —
  the pre-check primitive itself is BUILT and tested (`sparq_trust::admit_with_precheck`, opt-in
  `secprop-precheck`, consuming the vendored ISWC-2025 `sec-prop:` ontology via the
  `secprop-admissibility` N3 reduction; `tests/secprop_precheck_e2e.rs`, sq-dt5hv Phase 5). What
  remains OPEN for MPC-SPARQL is wiring it into the federated / multi-source admission path —
  today `admit_with_precheck` is exercised only in single-party `sparq-trust` tests (navigator §3.3).

---

## Candidate Normative Surface

The following are the spec-ready normative anchors grounded in code. Items marked (DESIGN)
are designed-not-built and must not be stated as implemented:

1. **Party/role model**: a conformant MPC-SPARQL federation MUST distinguish four roles —
   Issuer (honest signer, root of trust, pk in disclosed key-set K), Holder/data-source
   (mutually-distrusting, possibly-malicious; acts as BOTH an MPC compute party AND a
   collaborative prover), Verifier/relying-party (honest-but-curious; issues a fresh challenge
   N, checks the proof), and an UNTRUSTED Planner whose plan MUST NOT be trusted for soundness
   (arch §4.1; `pipeline.rs`).

2. **Each Holder MUST be exactly one honest-majority compute party**: `n >= 2t+1` required for
   any multiplication/comparison, checked fail-closed (`pipeline.rs:244`).

3. **Secret-share format** (F_p = 2^61−1 Mersenne, `field.rs:34`): a Share is
   `(x: party index, u64, always ≥1; y: f(x) in F_p)`; x=0 is RESERVED for the secret and
   MUST NEVER be handed to a party (`shamir.rs:98`).

4. **Network wire protocol** (star-coordinator simulation, `transport.rs:88`): messages are
   self-describing length-prefixed frames — `Shares(Vec<Share>)` coordinator→party,
   `Open(Vec<Share>)` party→coordinator, `Step(StepCode)`, `Swap{a,b,swap}`, `Done`;
   `StepCode` in `{SumAndOpen=1,MulAndOpen=2,SwapNetworkAndOpen=3}`; field elements travel as
   canonical u64 big-endian, 8 bytes (`FIELD_BYTES`).

5. **Per-operator disclosure routing (RQ2a) MUST be explicit and inspectable**: every operator
   is tagged `Disclosed` (crypto-free, global-IRI key, computed in the clear) or
   `Hidden(OperatorClass)` (secret-shared inside the MPC) — `pipeline.rs:93 Routing`,
   `:110 OperatorRouting`, surfaced in `FederatedResponse.routing`.

6. **No-proof-of-revealed-properties convention**: anything a deterministic function of the
   disclosed multiset (DISTINCT/ORDER BY/LIMIT/OFFSET/COUNT/SUM/AVG/MIN/MAX) MUST be
   recomputed by the verifier outside the cryptographic core, never proven in-circuit (arch
   §2 conv#4).

7. **Secure aggregate**: zero-round local share-addition; a threshold comparison MUST disclose
   ONLY the boolean verdict bit and MUST NOT reconstruct the exact sum across the API boundary
   (`pipeline.rs:270–287`; structural assertion at `:528–534`).

8. **Disclosed issuer key-set K** MUST be canonicalised (sort + dedup) so the
   `ProofStatement`/public-inputs digest is invariant to caller order and multiplicity
   (`pipeline.rs:319–322`; `ProofStatement.disclosed_key_set` at `proof.rs:74`).

9. **Join-key encoding for PRIVATE keys**: `encode(term) =
   reduce_mod_p(SHA-512(DOMAIN_TAG || ntriples(term)))` with
   `DOMAIN_TAG = b"sparq-mpc/term-join-key/v1\0"` (`term_encode.rs:87,98`).

10. **Commitment + attestation format**: each named graph carries C(G_i) issuer-signed as
    `Sign(pk_i, {C(G_i), true-triple-count, salt})` (`arch §4.3 step 1`); signed message via
    Schnorr-over-Baby-JubJub / Poseidon2 challenge with domain tags `ZKSIG_C3/C4`
    (`verifier.rs:2428–2443`).

11. **Attestation-binding trust anchor** MUST be the EXTERNAL relying-party key-set K, NEVER
    the prover-supplied `manifest.key_set`; prover MAY narrow but MUST NOT widen K
    (`verifier.rs:2242–2344`).

12. **Correctness-proof public-input binding**: field 0 of every sub-proof MUST be the
    verifier's fresh nonce; the canonical verifier key is recomputed verifier-side from the
    re-derived `CircuitId` (NEVER the prover's `vk`) (`verifier.rs:4685–4707`).

13. **Freshness/replay**: verifier nonce MUST be single-use (recorded BEFORE the crypto gate,
    burn-on-mismatch) — mandatory, no opt-out path (`verifier.rs:4601–4644`).

14. **`ProofStatement`** is the public statement the collaborative proof binds:
    `{ query (canonical SPARQL digest), disclosed_key_set K, challenge N, disclosed_result }`
    (`proof.rs:70–85`).

15. **Collaborative-proof MUST-check obligations** (DESIGN, gated): the joint proof MUST bind
    (a) `Disclosed(pi)` subset of `Eval_PAG(Q,D)`; (b) each contributing `C(G_i)` issuer-
    signed under a key in K; (c) the query digest, FILTER operator/operand/verdict, per-row
    source-graph attribution, and challenge N (`proof.rs:12–23`, arch §4.3 step 5).

16. **Security-model vocabulary**: guarantees reported PER-OPERATOR over three orthogonal axes
    `AdversaryModel(SemiHonest/Covert-with-epsilon/Malicious) × OutputGuarantee(Abort/
    Fairness/GuaranteedOutput) × CorruptionThreshold(HonestMajority/DishonestMajority)` plus
    `PublicVerifiability`, with Cleve's impossibility enforced as a type invariant
    (`backend.rs:106–180`).

17. **Security-considerations MUST-state**: the verifier-side-attestation v1 GIVES UP
    (1) source-unlinkability, (2) a single succinct signature-to-witness proof, (3) malicious-
    against-N-1, and (4) needs explicit out-of-circuit freshness binding; and NO
    soundness/attestation/privacy property is production-claimable until sq-qhy4 lands.

---

## Gaps (Spec Must Address)

1. **No wire format for the DEPLOYED protocol**: `transport.rs` is an explicit
   star-coordinator SIMULATION harness; the deployed full party-mesh is unspecified. The spec
   must define the real inter-party message set, not the simulation.

2. **Federated / multi-source public-input byte layout does not exist**: `reconstruct_public_inputs`
   is single-manifest; the ordering/packing of all holders' commitments/rows/attribution under
   ONE federated statement is designed-only (sq-34ml).

3. **No message/handshake formats** for the verifier↔federation exchange (delivery of Q +
   fresh nonce N, delivery/agreement of trusted key-set K, the response envelope) are
   formalised — arch §4.3 steps 2/6 are prose only.

4. **Collaborative `Proof` and `AttestationShare` are field-less placeholders** (`proof.rs:113,155`):
   proof-system choice (Noir/UltraHonk vs collaborative-SNARK), byte layout, and distributed
   attestation share representation are all unspecified.

5. **Canonical query normalisation/digest is undefined**: `pipeline.rs` uses
   `FederatedQuery.federated_query` VERBATIM and declares canonicalisation "the caller's
   responsibility" (`pipeline.rs:154–160`).

6. **Cross-graph blank-node identity for join keys is out of scope** of the encoder (matches
   on label only, `term_encode.rs:68–77`); the spec must define bnode join semantics or
   mandate `sparq-canon` relabelling.

7. **Federation-level revocation freshness-window and status-list reference handling** is
   single-prover (`bind_revocation`, `verifier.rs:2168`); the multi-source revocation/freshness
   policy is unspecified.

8. **Corruption-threshold normativity**: the honest-majority assumption is asserted (n≥2t+1)
   but whether a federation MUST refuse to run below a threshold is not pinned as a normative
   MUST.

9. **Leakage-envelope numeric budgets / DP epsilon accounting** exist as a mechanism but the
   normative leakage vocabulary and epsilon-budget format are deferred (sq-shk5).

10. **coZK witness-validation-before-proving** is only a TEST obligation (`witness_validation_tests.rs`);
    the spec must state validation of the extended witness BEFORE proving as a normative MUST
    for any future collaborative-proof implementation (eprint 2025/1026 pitfall).

---

## Honesty Flags

- **The 'ZKP of correctness' at the MPC/federated layer is NOT built**: `proof.rs prove/verify`
  are honest `MpcError::NotYetImplemented` stubs. Any spec sentence implying a working
  federated correctness proof today would be false.

- **Guarantee C (attested-source derivation) at the federated layer is DESIGN-only**; only the
  SINGLE-PROVER attestation binding (`bind_issuer_attestations`) is built.

- **ALL soundness rests on internal single-model self-audits** (`research/zk-soundness-audit.md`,
  `research/mpc-cozk-reaudit.md`); the external accredited-cryptographer audit **sq-qhy4** (P0)
  has NOT run. SECURITY.md correctly publishes "treat as untrusted". Every production
  soundness/attestation/privacy claim MUST be caveated as NOT-audited / NOT-production-claimable.

- **The collaborative (multi-prover) path is NOT covered** by the single-prover re-audit;
  coZK malicious security is unsettled — eprint 2025/1026 shows proving on an
  invalid/inconsistent witness can LEAK honest provers' inputs; the closest usable stack
  (TACEO co-snarks / coNoir-Barretenberg) is explicitly UNAUDITED.

- **Default security tier is honest-majority SEMI-HONEST, not malicious**: the pipeline
  threshold comparison is `OperatorClass::Comparison @ semi-honest-only`; IT-MAC
  malicious-with-abort is only partially landed (sq-km34.* OPEN).

- **In-circuit distributed signature** over a secret-shared witness (source-unlinkable
  attestation) is UNSOLVED in the literature and correctly deferred (sq-bjl); it MUST NEVER
  be presented with a performance number.

- **The Term→Fp join-key encoder is a STATISTICAL birthday bound** (q²/2^62), not a security
  guarantee; the in-circuit encoding-correctness proof is the unbuilt M4 job.

- **Dishonest-majority malicious correctness for ANY SPARQL operator on WAN is NOT achieved**
  in the literature and has ZERO performance data points; the registry FAIL-CLOSES rather than
  downgrading.

- **Benchmarks are NON-CANONICAL**: only deterministic metrics (bb gate counts, MPC
  round/byte COUNTERS, nargo info opcodes) are quotable; netem LAN/WAN wall-clock needs
  CAP_NET_ADMIN and is gated to sq-hoaj.

- **`sparq-trust` is a research-grade PoC** (clear-text admission, no unlinkability,
  operator-asserted issuer keys); the vendored ISWC-2025 `sec-prop:` ontology is currently
  DATA — nothing loads it (navigator §3.3).

---

> **Empirical-honesty reminder**: ZK and MPC estates are NOT production-sound until the
> external cryptographer audit sq-qhy4 completes. All work-box benchmarks are non-canonical;
> do not hard-code them in documentation or tests.

---

*Recon captured by Sonnet 4.6 under the Fable program; [SONNET-4.6]*
