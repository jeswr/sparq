<!-- [OPUS-4.8] M4 distributed-sig feasibility spike, Opus 4.8 (Fable unavailable). -->

# M4 distributed-signature-over-secret-shared-witness — feasibility verdict

Scope: can sparq M4 verify each holder's issuer-signed graph commitment *inside a
collaborative proof over the secret-shared committed data*, in the decided
honest-majority trust model? And what is the honestly-buildable path?

This verdict is a synthesis spike over four research syntheses (collaborative
zk-SNARKs; sig-verify-over-secret-shared-message; MPC-in-the-head / verifier-side
attestation alternatives; the sparq-specific Schnorr/Baby-JubJub instantiation),
cross-checked against the live estate (`sig.rs`, `proof.rs`, `scan.nr`, `PLAN.md`,
architecture §5.2/§5.3). Sources are cited inline and listed at the end.

---

## 1. Is in-circuit distributed-sig TRACTABLE for M4? Does any construction exist?

**No published construction exists, and it is genuinely novel/unsolved.** The
specific join sparq M4 wants — verify a digital signature whose *signed message
(the graph commitment) is itself secret-shared across N mutually-distrusting
provers*, under a *hidden* issuer key in a set `K`, emitting one ordinary
proof — is, to the best of this survey, unbuilt and unbenchmarked in any venue
as of this survey (2026-06-13). The three ingredients each exist *separately* but the composition
does not:

- **Collaborative proving over a shared witness exists** — Ozdemir-Boneh (USENIX'22,
  eprint 2021/1530) lift Groth16/Plonk/Marlin/Fractal provers into MPC; scalable
  HyperPlonk follow-ups (eprint 2024/940, USENIX'25) and TACEO `co-snarks`
  (coNoir → UltraHonk, sparq's exact verifier) implement it. But **none instantiates
  the circuit with a signature-verification relation over a shared message** —
  Ozdemir-Boneh's *own* slides mark "authentication" as already-solved by plain
  single-prover SNARKs, i.e. explicitly out of scope; their design *pushes*
  heavy non-linear public relations (Merkle, sig) *out* of the MPC via proof
  composition. A signature relation over a shared message is the worst case they
  engineered around.
- **"Signed under one of a set K of keys" exists** — DiStefano (NDSS'24) ZKPVS /
  CDLS / ZKAttest; cost ∝ |K|, |K|≈10 practical. But it is **single-prover**: the
  message (handshake transcript) is held by one party; DiStefano's 2PC shares the
  *session keys*, not the *signed message inside the signature relation*.
- **Signing over data the signers don't see exists** — Coconut (NDSS'19), BBS+
  blinded issuance — but that is the *issuance* side (blinded from issuers), not
  multi-prover verification over a secret-shared witness.

**Why it's hard, not just unbuilt** (the structural reasons, so the thesis can claim
the gap defensibly): (a) sig verification is heavily non-linear (EC scalar-mul +
hash-to-field) — running it as MPC over a shared message means thousands of
multiplication gates on *shared* values, each a comm round in GSZ/SPDZ, exactly the
cost the coZK papers avoid; (b) the hidden-key-set adds an OR/ring structure
(cost ∝ |K|) *on top of* shared-message verification, and every anonymity-set proof
(CDLS/ZKAttest/Coconut) was designed single-prover or issuer-side. The estate already
names this precisely: it is **architecture §5.2 Q1, "the join nobody has built"**, and
`crates/sparq-mpc/src/proof.rs:45-57` is a `NotYetImplemented` scaffold for exactly it.

**Verdict on tractability: NOT tractable as a single M4 step.** It is *two* research
steps out, not one — because the in-circuit signature gadget **is not even built
single-prover yet** (grep confirms no `std::schnorr` / `embedded_curve` / signature
gadget anywhere in `zk/`; `scan.nr` takes `commitments[g]` as a *public input* and
never sees a key, and `sig.rs:29-39` states verification is **verifier-side** in v1).
The collaborative form is a strict superset of an in-circuit single-prover upgrade
(`sig.rs:23-27`) that has not landed. Cost/risk if pursued: in-circuit Baby-JubJub
scalar-mul under secret sharing is "a heavy black box" (`sig.rs:30-34`), it requires
new circuit members + vk recompute, AND it inherits the coZK soundness caveat (§3).

---

## 2. Recommended PATH

**Confirm the verifier-side-attestation interim as M4 v1.** This is the
*commit-and-prove anchor* pattern (Artemis, arXiv 2409.12055 — "signature verified
separately *outside* the SNARK, only the commitment hash linked to the proof")
combined with *authenticated-input-via-distributed-PoK-outside-MPC* (Dutta et al.
"Compute, but Verify", eprint 2022/1648 — the **only** work the 2025 verifiability
survey names as delivering verifiability *and* input authentication, honest-majority
LSSS). It is the **direct generalisation of single-source ZK-remediation #3 to N
sources** (`proof.rs:90-93`, architecture `md:110`).

**M4 v1 delivers:** correctness (the federated aggregate/threshold over secret-shared
committed data) **+** attestation-that-each-source's commitment was signed by a key
in the disclosed set `K`. The signature check stays exactly where it is today —
`bind_issuer_attestations` (`verifier.rs:985+`) over each already-public `C(G_i)`,
byte-bound into the proof by the audit-#1 reconstruction (`verifier.rs:1727-1729`).
The verifier stays an **unchanged single-prover UltraHonk verifier** — favourable,
because the estate already recomputes a *canonical* vk verifier-side (#2) and never
trusts the prover's vk.

**M4 v1 gives up** (state loudly, per the empirical-honesty rule): (1)
**source-unlinkability** — the verifier checks `pk_i` in the clear, so it learns
*which* issuer signed each graph, not merely "some key in K" (`sig.rs:35-39`; the
in-circuit BBS-in-ZK version would be unlinkable); (2) **a single succinct
verifier-unchanged proof binding signature↔witness** — v1 is (proof-of-computation)
+ (separately-verified opening/signature) glued by a commitment, multiple checks;
(3) **stronger trust assumption** — honest-majority LSSS, not malicious-against-N−1;
(4) it must **explicitly bind freshness/replay** (audit #4) since the sig is checked
out-of-circuit (in-circuit this is automatic).

**Smallest-first in-circuit step (the only thing that genuinely *must* federate):**
the **correctness relation over secret-shared committed data when a joined VALUE
stays hidden** — the cumulative-threshold case where per-holder addends never leave
the source. Concretely: federate `scan.nr`'s commitment-recompute + row-soundness
(`scan.nr:75-112`) over secret-shared `enc[g][i]`, each holder proving its own
`commit_fold(enc_i, counts_i) == C(G_i)` and contributing a secret-shared addend,
threshold compared in the secure computation (`backend.rs:107`). **Signature and
key-set membership stay verifier-side** (no in-circuit scalar-mul, accept the
which-issuer leak). Tractable because verifier unchanged + reuses #1/#2/#8
byte-binding verbatim. Still a **spike, not routine engineering** (needs the M3
backend + a fresh coZK audit).

**When to pursue the full in-circuit version (the thesis novelty):** only *after*
(a) RQ1 remediation lands, (b) the in-circuit single-prover Schnorr+set-membership
upgrade lands (`sig.rs:23-27` — currently unbuilt), and (c) M3 backend exists. It is
the genuine research contribution precisely because it is unsolved (§1) — but it is
audit-gated and two steps out, not the M4 starting point.

---

## 3. Honest feasibility envelope

- **Performance:** coZK "≈ single-prover cost (malicious-minority), 2× (N−1
  malicious)" is **real but conditional — 3 Gb/s LAN only, preprocessing EXCLUDED**
  from every headline number (Ozdemir-Boneh §7 verbatim; Ω(n) comm lower bound). The
  scalable 30× numbers are **semi-honest-only**. On a WAN federation none of this
  holds; comm dominates. In-circuit single-prover EdDSA-Poseidon/Baby-JubJub ≈ 4,218
  constraints (arXiv 2301.00823) — but that is single-prover; the secret-shared
  multi-prover cost is unmeasured. **Budget M4 in minutes for the viable regime,
  never "seconds"; the full WAN/dishonest-majority composition has ZERO data points.**
- **Viable regime (the credible first target):** honest-majority (or honest-but-curious
  among cooperating holders), LAN/datacenter, small committed datasets (≤10³–10⁴
  triples/party), few-pattern BGPs, disclosed-property aggregates recomputed by the
  verifier. Aligns with the sub-second-for-≤10³ RQ1 figure (`md:125`, `PLAN.md:19`).
- **Soundness caveats found (current, not historical):** (i) **eprint 2025/1026
  (CRYPTO'25, Garg-Goel-Jain-Roberts-Sekar)** — the coZK *template* (semi-honest MPC
  prover + off-the-shelf malicious compiler) has two pitfalls; proving on an
  **invalid/inconsistent witness leaks honest provers' inputs**, and "state-of-the-art
  malicious compilers as-is are insecure, in general." Patched guidance:
  honest-majority semi-honest-prover ≈ malicious-secure **only if the extended witness
  is validated *before* proving** — does NOT rescue dishonest-majority. (ii) TACEO
  `co-snarks` (closest usable stack, coNoir⇄Barretenberg matches sparq) is
  **explicitly unaudited** and its docs don't reference 2025/1026. (iii) Covert/PVC
  (~354-byte cheating cert, 20–40% overhead) is **detection/deterrence, not soundness** —
  a fallback accountability tier, not equivalent to the ZK proof. (iv) **MPC-in-the-head
  is NOT a candidate** — single-prover simulation, not proof of a real multi-party
  execution; cite only to kill the conflation (reuse only for the cheap key∈K subproof,
  eprint 2021/1656).

---

## 4. Dependency chain — what M4 v1 needs

1. **HARD GATE — RQ1 verifier soundness (M1):** #3/#4 (issuer-sig + key-set membership
   + replay/freshness), #5/#6 (FILTER binding — the £25k/£100k threshold), #8/#9
   (in-circuit per-row attribution + salt separation), #12 (revocation). Until these
   land with negative e2e tests, M4 composes onto a verifier that returns `Ok(())` for
   forged results — theatre (architecture `md:115`, `PLAN.md:59`). **M4 attestation IS
   #3 generalised to N sources; it cannot precede single-source #3.**
2. **M3 — `MpcBackend` (honest-majority first):** Q2 (trust model) resolved to
   honest-majority replicated 3PC secret-sharing (`PLAN.md:90-97`); dishonest-majority
   left as swappable future research. M4 is hard-blocked on M3.
3. **The ZK estate seams M4 v1 reuses verbatim:** #1 `reconstruct_public_inputs`
   (`verifier.rs:1686`) — needs a *federated/multi-source layout variant* spanning all
   holders' `commitments[]`/`rows[]`/`attribution[]`, byte-compared unchanged; #2
   canonical vk recompute — fork-favourable, verifier needs no change; #8 in-circuit
   `attribution[g]` (`scan.nr:143`) — the cleanest generalisation, lift from
   one-holder's-K to graph→issuer-key-set across holders.
4. **M4 v1 specifically needs:** the M3 collaborative-proving backend; the federated
   `reconstruct_public_inputs` layout; freshness/replay binding for the out-of-circuit
   sig; and a **fresh coZK soundness re-audit** against 2025/1026 (validate extended
   witness before proving; do not trust an off-the-shelf malicious compiler). Q1/Q4
   re-evaluated at M4 (`PLAN.md:128-133`).

---

## Primary sources
- Ozdemir & Boneh, *Experimenting with Collaborative zk-SNARKs*, USENIX Sec'22 — eprint 2021/1530.
- Garg, Goel, Jain, Roberts, Sekar, *Malicious Security in Collaborative zk-SNARKs: More than Meets the Eye*, CRYPTO'25 — eprint 2025/1026.
- Liu et al., *Scalable Collaborative zk-SNARK / Distributed Proof Delegation*, USENIX Sec'25 — eprint 2024/940; 2024/143.
- Dutta et al., *Compute, but Verify: MPC over Authenticated Inputs* — eprint 2022/1648 (the attestation pillar, honest-majority LSSS).
- *Artemis: Commit-and-Prove SNARKs* — arXiv 2409.12055 (out-of-circuit signature, commitment anchor).
- DiStefano, NDSS'24 (ZKPVS, signed-under-one-of-K, single-prover); Coconut, NDSS'19 / arXiv 1802.07344 (threshold blind issuance).
- Heimes/Cremers et al., arXiv 2301.00823 (EdDSA-Poseidon/Baby-JubJub ≈ 4,218 constraints; ECDSA ~1.5M); Benchmarking ZK-Circuits in Circom, eprint 2023/681.
- TACEO `co-snarks` (coNoir → UltraHonk; explicitly unaudited): github.com/TaceoLabs/co-snarks.
- Covert+PVC: eprint 2018/1108; MPC-in-the-head: eprint 2021/1656 (key∈K subproof only).
- Estate: `research/mpc-zkp-research-and-architecture.md` §5.2/§5.3; `crates/sparq-mpc/PLAN.md`; `crates/sparq-zk/src/sig.rs`; `crates/sparq-zk-compose/src/verifier.rs` (`reconstruct_public_inputs:1686`, `bind_issuer_attestations:985`); `zk/compose/compose_core/src/scan.nr` (in-circuit `attribution:143`, no sig gadget); `crates/sparq-mpc/src/proof.rs:45-57` (Q1 `NotYetImplemented`).
