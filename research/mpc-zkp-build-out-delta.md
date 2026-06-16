<!-- [OPUS-4.8] MPC+ZKP build-out design-DELTA + milestone/bead map, Opus 4.8 (Fable
unavailable) — design-for-review; re-review when Fable returns. ADDITIVE to the existing
MPC+ZKP estate; not a re-derivation. -->

# MPC + ZKP build-out — design-delta + milestone/bead map (epic sq-pwr)

**Status:** Additive planning record for epic **sq-pwr** ("MPC over federated SPARQL with
ZKP of correctness + attested-source derivation"). Author: Opus 4.8 (Fable unavailable —
flag for re-review). Date: 2026-06-16.

**What this is.** A *delta* over an already-rich estate: the architecture, the per-operator
capability matrix, the ZK soundness audit + re-audit, and the M4 feasibility verdict all
exist and are current. This record does **not** re-derive any of them. It (1) states the
current ground truth concisely, (2) maps the gap from "designed + (now) sound" toward
"sound + federated" onto concrete milestones, each with its hard dependency, opt-in
structuring, and a done-definition, and (3) keeps an honest risk register separating what is
externally-gated from what is internally-actionable. **The headline honest outcome:** the
build-out is already designed, sequenced, and *substantially landed*; the genuine remaining
frontier is narrow (the collaborative-proof / distributed-attestation join), correctly
deferred and audit-gated. Only a **small** number of genuinely un-beaded delta items remain
— enumerated in §3, beaded in §6.

---

## 1. State of play (concise — the estate this builds on)

Read these; this record extends, does not duplicate, them:
- **Architecture** — [`mpc-zkp-research-and-architecture.md`](./mpc-zkp-research-and-architecture.md):
  the four guarantees (A confidentiality / B correctness / C attestation / D malicious),
  the protocol flow (§4.3), and the RQ1-remediation hard gate (§5.1).
- **Capability matrix** — [`mpc-sparql-capability-matrix.md`](./mpc-sparql-capability-matrix.md):
  per-operator × per-configuration tiers (BUILT/KNOWN/OPEN/IMPOSSIBLE) and the sequenced
  gap roadmap (§8). **Note: this matrix is dated 2026-06-15 and is already partly stale —
  several of its OPEN keystones have since landed (see below).**
- **ZK soundness** — [`zk-soundness-audit.md`](./zk-soundness-audit.md) (the original "v1 is
  BROKEN" finding) **and its reversal** [`zk-verifier-reaudit.md`](./zk-verifier-reaudit.md)
  (post-remediation, "**SOUND as landed**" for the assumed threat model — all 12 findings
  CLOSED with code evidence).
- **M4 feasibility** — [`mpc-m4-distributed-sig-feasibility.md`](./mpc-m4-distributed-sig-feasibility.md):
  the verifier-side-attestation interim is the buildable M4 v1; the in-circuit
  distributed-signature-over-secret-shared-witness is the thesis-novel, unsolved, audit-gated
  spike.

**Ground truth verified against `main` (2026-06-16), correcting stale doc claims:**

- **ZK soundness gate is CLOSED.** Epic **sq-1s2** (the RQ1 verifier-soundness remediation)
  is **17/17 complete**. The re-audit (sq-gbp4) independently re-ran all 12 findings against
  the landed verifier and found every one CLOSED, with forge-and-verify reject paths and a
  non-circular empirical anchor on the public-input reconstruction. The single-prover ZK
  estate (`crates/sparq-zk`, `crates/sparq-zk-compose`, `zk/compose`) is sound *for its
  threat model* — a relying party that supplies the trust anchors (key set, nonce, status
  snapshot). **The in-circuit hidden cross-credential join (`sq-bwwl`) has also landed**
  (PR #170): the joined entity stays out of public inputs — the largest single-prover
  privacy gap is closed.
- **The MPC keystone has LANDED.** The matrix names degree reduction (`sq-dvuc`) as the
  keystone blocking ~half the operator surface. **`sq-dvuc` (degree reduction),
  `sq-rrz4` (secure comparison opening only the verdict bit — the £100k `>`), and
  `sq-py8h` (bounded property paths) are all CLOSED on `main`.** `compare.rs`,
  `degree_reduce` in `shamir.rs`, and `bounded_path.rs` exist and are tested. The
  end-to-end federated pipeline driver (`pipeline.rs`, `sq-6y92`) composes holder → share →
  join → secure-threshold → ProofStatement for the four-flatmates scenario.
- **Malicious-security (IT-MAC) is in flight, fully decomposed.** `authenticated.rs`
  (`sq-km34.1`, CLOSED) lands the authenticated-sharing type + shared MAC key; the remaining
  IT-MAC work (MAC-carrying mult, batched MAC-check, malicious equality/comparison, registry
  wiring, adversarial catch-tests, the bench AXIS-1 lift) is beaded as `sq-km34.2–.9` (OPEN).
- **The 3-axis configurable security model is BUILT** (`backend.rs`: AdversaryModel ×
  OutputGuarantee × CorruptionThreshold), with Cleve (no fairness/GOD without honest
  majority) enforced as a *type invariant*. Robust reconstruct (Berlekamp–Welch), oblivious
  shuffle (sound) + sort network, CSPRNG masking, result-size protection, and the tier-1/2/3
  benchmark harness (modelled counters + real loopback transport + netem profiles) are all
  BUILT.

**The one true remaining frontier:** guarantee **C** (attestation) *federated* — i.e.
binding "each source's committed graph was issuer-signed under a key in K" to a *single
collaborative proof over secret-shared committed data*. This is "the join nobody has built"
(architecture §5.2 Q1; M4 doc §1) — unsolved in the literature, correctly held as a
**DEFERRED spike** (`sq-bjl`), with a buildable verifier-side-attestation *interim* (Dutta
authenticated-input + Artemis commit-and-prove anchor) as M4 v1.

---

## 2. Gap → milestone map

The estate already partitions the work into a CHEAP (BUILT) zone, a MEDIUM zone (mostly now
landed/beaded behind the keystone), and an OUT-OF-REACH zone. The milestones below are the
*remaining* steps to move sq-pwr from "designed + sound single-prover + cheap-MPC landed"
toward "sound + federated", **at decision granularity, deduplicated against the existing
bead estate** (existing beads referenced, not re-created). Each milestone is opt-in: all
MPC↔ZK work lives in `crates/sparq-mpc` / `crates/sparq-zk*` / `zk/compose`; the lean core
(`sparq-core`/`sparq-engine`) is never touched, and the insecure-randomness path is already
behind the off-by-default `insecure-test-rng` cargo feature.

### M-A. Finish honest-majority malicious security (IT-MAC) — *internally actionable now*
- **Goal:** promote the semi-honest equality/comparison/join opens to malicious-with-abort
  at honest majority (close the one no-RS-redundancy hole at minimal `N=2t+1`, and the
  coZK-2025/1026 confidentiality interaction on the mid-pipeline open).
- **Crate(s):** `sparq-mpc` (`authenticated.rs`, `shamir.rs`, `compare.rs`, `join.rs`,
  `backend.rs` registry, `metrics.rs`/`bench.rs`).
- **Hard dependency:** degree reduction (`sq-dvuc`, CLOSED) + the authenticated-sharing
  foundation (`sq-km34.1`, CLOSED). **No ZK-soundness gate** — this is pure MPC-layer.
- **Opt-in:** new `new_malicious` backend selected by the fail-closed `SecurityRequirement`
  registry; semi-honest stays the default; no new crate needed.
- **Done-definition:** a tampering party is CAUGHT (adversarial test), the registry exposes
  an `AuthenticatedAbort` descriptor and refuses to downgrade, and the bench matrix carries
  AXIS-1 (semi-honest vs malicious) so the authentication lift is MEASURED.
- **Beaded:** `sq-km34.2–.9` (OPEN) — no new bead needed. Tracked here as the near-term
  internally-actionable spine.

### M-B. SOTA hidden-join cost + privacy (replace all-pairs) — *internally actionable now*
- **Goal:** replace the `O(|L|·|R|)` all-pairs `HiddenValueJoin` with an ORQ-style
  O(n log n) oblivious sort-merge join consuming the BUILT shuffle, emitting a shuffled
  padded prefix; wire the landed result-size/match-bit protection (`sq-jnkm`) into the join
  path. Closes the L1 (result cardinality) and L2 (per-pair match-graph / fan-out) leaks.
- **Crate(s):** `sparq-mpc` (`oblivious_join.rs`, `join.rs`).
- **Hard dependency:** degree reduction + secure comparator (`sq-dvuc`, `sq-rrz4` — both
  CLOSED) + oblivious shuffle (`sq-18lk`, CLOSED). **No ZK-soundness gate.**
- **Opt-in:** same crate; the all-pairs path stays as the correctness oracle for differential
  tests.
- **Done-definition:** federated join == plaintext inner-join over the union (existing
  differential test) at sub-quadratic cost, with the per-pair match graph no longer opened.
- **Beaded:** `sq-ujz8` (sort-merge join, OPEN) + `sq-y32f` (wire `sq-jnkm` into the join,
  OPEN). No new bead.

### M-C. M4 v1 — verifier-side authenticated-input attestation, federated — *ZK-foundation gated (NOW UNGATED), audit-gated for production*
- **Goal:** the *buildable* M4: each holder evaluates its fragment, contributes secret-shared
  committed data; the federated correctness relation (aggregate/threshold) runs in MPC; and
  attestation ("each source's `C(G_i)` signed by a key in `K`") is checked **verifier-side**
  (Dutta authenticated-input / Artemis commit-and-prove anchor), byte-bound into the proof by
  a *federated/multi-source* `reconstruct_public_inputs` layout, with explicit
  freshness/replay binding because the signature is checked out-of-circuit.
- **Crate(s):** `sparq-mpc` (`pipeline.rs`, `proof.rs`) + `sparq-zk-compose` (the federated
  `reconstruct_public_inputs` layout variant + `bind_issuer_attestations` reuse).
- **Hard dependency:** the RQ1 ZK-soundness gate — **NOW SATISFIED** (`sq-1s2` 17/17 +
  re-audit). M3 backend (`sq-pwr.1`, CLOSED) + the in-circuit single-prover sig gadget
  (`sq-z9l`, CLOSED). **Production claim additionally gated on the external cryptographer
  audit (`sq-qhy4`, P0) AND a fresh coZK soundness re-audit (see M-D).**
- **Opt-in:** stays a single-prover UltraHonk verifier (favourable — canonical vk recomputed
  verifier-side already); no new crate, no core change.
- **What v1 gives up (state loudly):** source-*unlinkability* (`pk_i` checked in clear →
  verifier learns *which* issuer), a single succinct sig↔witness proof (v1 is
  proof-of-computation + separately-verified signature glued by a commitment), and
  malicious-against-N−1 (honest-majority LSSS only).
- **Done-definition:** federated correctness + verifier-side attestation for the
  four-flatmates threshold, with negative e2e tests (forged source, replayed manifest,
  out-of-K key all REJECT) and the freshness binding closing audit-#4 out-of-circuit.
- **Beaded:** prerequisites in `sq-34ml` (OPEN: freshness/replay binding + federated
  reconstruct layout). The **attestation-gate assembly itself** (the Dutta/Artemis interim
  gate that consumes `sq-34ml`'s prereqs and produces the verifiable federated response) is
  **NOT yet its own bead** → **FILED (§6, M-C bead).**

### M-D. Fresh coZK soundness re-audit (eprint 2025/1026) — *gating M-C/M-E production claims*
- **Goal:** before *any* collaborative-proving claim, validate the construction against the
  CRYPTO'25 coZK pitfalls: proving on an inconsistent/extended witness can LEAK honest
  provers' inputs, and "SOTA malicious compilers as-is are insecure in general." Required
  guidance: honest-majority semi-honest-prover ≈ malicious-secure *only if the extended
  witness is validated before proving*; the closest usable stack (TACEO co-snarks /
  coNoir⇄Barretenberg) is explicitly **unaudited** and predates 2025/1026.
- **Crate(s):** none directly — a research/audit deliverable governing `sparq-mpc::proof` +
  `sparq-zk-compose` when M4 v1 / the M4 spike is built.
- **Hard dependency:** none to *start the analysis*; it is a prerequisite to *shipping* M-C
  (and M-E) with any soundness claim. Distinct from `sq-qhy4` (external accredited audit of
  the **single-prover** verifier + circuits) and from `sq-aaop`/`sq-wj4k` (UC/composition
  *design records*, not an adversarial coZK re-audit).
- **Opt-in:** N/A (analysis); its verdict gates whether the collaborative-proof code may be
  presented as proving anything.
- **Done-definition:** a written re-audit verdict (CLOSED/RE-OPEN per the 2025/1026 lenses) +
  a witness-validation requirement encoded as a test obligation on the collaborative path.
- **Beaded:** **NOT yet a bead** → **FILED (§6, M-D bead).**

### M-E. M4 spike — in-circuit distributed signature over secret-shared witness — *RESEARCH-NOVEL, audit-gated, DEFERRED*
- **Goal:** the thesis contribution — verify each holder's issuer signature *inside* a
  collaborative proof over secret-shared committed data, yielding source-*unlinkable*
  attestation in a single verifier-unchanged proof. Smallest first step: federate `scan.nr`'s
  commitment-recompute + row-soundness over secret-shared `enc[g][i]` with the signature/key-
  set membership still verifier-side, then lift to in-circuit scalar-mul.
- **Crate(s):** `sparq-mpc` + `zk/compose` (a federated `scan.nr` variant).
- **Hard dependency:** M4 v1 (M-C) shipped + M-D coZK re-audit + the external `sq-qhy4` audit.
  It is two research steps out and **unsolved in the literature** — budget as research risk,
  never "seconds."
- **Opt-in:** behind the same opt-in MPC crate/feature; never default.
- **Done-definition (spike):** a worked smallest-federated-relation prototype with a fresh
  coZK soundness verdict and negative e2e tests — explicitly a spike, not routine engineering.
- **Beaded:** `sq-bjl` (DEFERRED spike) — no new bead; referenced as the terminal research
  milestone.

### M-F. Dishonest-majority + WAN frontier — *KNOWN/OPEN, deliberately scoped out of v1*
- **Goal:** the realistic cross-org trust model (dishonest-majority SPDZ/MASCOT/Overdrive
  behind the backend trait, with the preprocessing tax made budgetable) and the WAN-tier
  protocol family (constant-round comparison / Boolean backend), plus a DP output-cardinality
  mode with ε-budget tracking.
- **Crate(s):** `sparq-mpc` (`backend.rs` trait + `BackendInfo`, a future SPDZ backend).
- **Hard dependency:** none structural; honestly scoped LAN-first / honest-majority-first.
  **No published system delivers dishonest-majority-malicious correctness for any SPARQL
  operator** — the registry correctly *refuses* rather than downgrading.
- **Opt-in:** swappable backend behind the trait; the registry fail-closes today.
- **Done-definition:** design records + `BackendInfo.requires_preprocessing`/PQ/trusted-setup
  fields so a federation can budget the usually-hidden-dominant offline cost.
- **Beaded:** `sq-j5ok` (DM backend design record), `sq-38zk` (WAN constant-round + Boolean),
  `sq-4i39` (`requires_preprocessing` field), `sq-shk5` (DP cardinality), `sq-ox16` (covert/PVC
  tier), `sq-aaop`/`sq-wj4k` (composition/UC records), `sq-yyro` (PRSS/dealer-less VSS) — all
  OPEN, all beaded. No new bead.

---

## 3. The genuine un-beaded delta (this is the whole net-new of this record)

After deduplicating against `sq-1s2` (17/17 CLOSED), `sq-0jsc`'s 19 children, and the
`sq-pwr`/`sq-bjl`/`sq-34ml`/`sq-km34.*` lines, **only two genuinely-actionable items have no
bead** (everything else in §2 is already tracked or correctly deferred):

1. **M4-v1 verifier-side attestation gate assembly** (M-C). `sq-34ml` files the *prerequisites*
   (freshness binding + federated reconstruct layout); `sq-bjl` is the *deferred in-circuit
   spike*. The buildable interim *gate itself* — the Dutta/Artemis verifier-side
   authenticated-input attestation that consumes `sq-34ml` and produces the verifiable
   federated four-flatmates response — falls between them with no home. **→ new bead.**
2. **Fresh coZK soundness re-audit against eprint 2025/1026** (M-D). Named as a hard
   requirement in the M4 doc (§4 step 4) and the matrix (§8 #12 caveat), but no bead exists:
   `sq-qhy4` audits the *single-prover* estate; `sq-aaop`/`sq-wj4k` are *design records*. The
   adversarial coZK re-audit of the *collaborative* path is un-beaded. **→ new bead** (P2,
   gating M-C/M-E production claims — distinct from, and complementary to, the external P0
   `sq-qhy4`).

A third, smaller honesty item is worth a bead but is lower-priority:

3. **Stale-doc reconciliation** — the capability matrix (`mpc-sparql-capability-matrix.md`,
   §1.2/§4.2/§8) still marks `sq-dvuc`/`sq-rrz4`/`sq-py8h` as OPEN and the keystone as
   blocking; they have since landed. A doc-refresh pass (per the docs-stay-current rule)
   keeps the canonical matrix honest. **→ new bead** (P3, doc-only).

**Everything else is already beaded or correctly deferred.** This is the honest "no large new
milestone tree needed" outcome the brief allows for: the build-out is designed and sequenced;
the delta is three small filed items plus a state-of-play correction.

---

## 4. Honesty / risk register

| Item | Externally-gated? | Internally-actionable now? | Must-NOT-ship-default-until |
|---|---|---|---|
| M-A IT-MAC malicious security | No | **Yes** (`sq-km34.*`) | — (opt-in malicious backend; semi-honest stays default) |
| M-B SOTA hidden join | No | **Yes** (`sq-ujz8`/`sq-y32f`) | — (differential-tested against all-pairs oracle) |
| M-C M4 v1 attestation gate | **Production claim: yes** (`sq-qhy4` P0 + coZK re-audit M-D) | **Yes** (build path is ungated now `sq-1s2` is CLOSED) | **MUST NOT present as proving attestation to a relying party until the coZK re-audit (M-D) AND the external `sq-qhy4` audit pass.** Ship behind opt-in; label "research, unaudited collaborative path." |
| M-D coZK re-audit | Partly (best done with external cryptographer eyes, but the analysis is startable internally) | **Yes** (start the adversarial pass) | gates any collaborative-proving soundness claim |
| M-E in-circuit distributed-sig spike | **Yes** (audit-gated + research-novel) | Only as a deliberate spike, post-M-C/M-D | **DEFERRED**; never default; budget as research risk, never "seconds" |
| M-F DM/WAN/DP frontier | No (but DM-malicious correctness for SPARQL is *unachieved in the literature*) | Design records yes; impl no | registry **refuses** DM-malicious today (fail-closed) — keep it refusing until a real backend + audit exist |

**Hard honesty constraints carried forward (do not contradict):**
- The single-prover ZK estate is **sound as landed for its threat model** (re-audit), *but*
  the external accredited-cryptographer audit (`sq-qhy4`, P0) has **not** run — no production
  ZK security claim until it does (and `sq-toze.35`/`sq-toze.23` track the standing "not-yet-
  externally-audited" posture). The collaborative (multi-prover) path is **not** covered by
  the single-prover re-audit and needs its own coZK re-audit (M-D) before any claim.
- The privacy-preserving federated join (M4) is **gated on the ZK-soundness landing** — that
  gate is now satisfied for the build, but the *attestation soundness claim* remains gated on
  M-D + `sq-qhy4`.
- No fabricated performance numbers anywhere; the viable regime is honest-majority /
  cooperating holders / LAN / ≤10³–10⁴ triples/party / few-pattern BGPs, and the full
  composition (coZK ⊕ malicious-DM ⊕ oblivious BGP joins ⊕ attested inputs ⊕ WAN) has **zero**
  performance data points.

---

## 5. Non-canonical timing note

Any wall-clock observed while developing these milestones on this work-box (an AWS EC2 host,
`-aws` kernel) is **NON-CANONICAL** — do not bake it into docs, tests, or claims. Only
deterministic metrics (constraint/gate counts via `bb gates`; MPC round/byte *counter*
output; `nargo info` opcode counts) are quotable. Shaped LAN/WAN MPC wall-clock requires
`CAP_NET_ADMIN`/netem and is gated to the orphan-proof EC2 bench run (`sq-hoaj`); the harness
never fabricates a shaped number.

---

## 6. Beads filed by this record

Only the genuinely-un-beaded §3 items (everything else is referenced, not re-created):
- **M-C** — *MPC M4-v1: verifier-side authenticated-input attestation GATE assembly* (consume
  `sq-34ml`; Dutta/Artemis; produce the verifiable federated four-flatmates response). P2.
  BLOCKED until the coZK re-audit (M-D) + external `sq-qhy4` pass before any production
  attestation claim; depends on `sq-34ml`.
- **M-D** — *Fresh coZK soundness re-audit (eprint 2025/1026) of the collaborative-proof path*
  (validate-extended-witness-before-proving; do not trust an off-the-shelf malicious
  compiler; TACEO co-snarks unaudited). P2. Distinct from the external single-prover
  `sq-qhy4`.
- **doc** — *Refresh `mpc-sparql-capability-matrix.md`: `sq-dvuc`/`sq-rrz4`/`sq-py8h` LANDED,
  keystone no longer blocking* (docs-stay-current). P3, doc-only.

(IDs recorded in the PR body / orchestrator report once created via the shared `bd` checkout.)
