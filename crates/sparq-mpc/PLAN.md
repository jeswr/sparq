<!-- [OPUS-4.8] sparq-mpc build plan. Blueprint: research/mpc-zkp-research-and-architecture.md. -->
# sparq-mpc — Build Plan (MPC over federated SPARQL, RQ2)

Blueprint: `research/mpc-zkp-research-and-architecture.md`.
ZK foundation it sits on: `research/zk-soundness-audit.md`.

**Goal (Jesse's):** widest query coverage, MINIMISE inter-node data sharing,
best performance, ZKP of (a) result correctness AND (b) the authoritative /
attested source. Privacy-first.

**The two design forks everything fork-dependent waits on:**
- **Q1 (research risk):** distributed signature / commitment-opening inside the
  collaborative proof over a *secret-shared* witness — unsolved in the
  literature. "The join nobody has built." (architecture §5.2 Q1)
- **Q2 (trust model):** honest-majority vs dishonest-majority — reshapes the MPC
  primitive, round structure, and preprocessing. (architecture §5.2 Q2)

The honest feasibility verdict (architecture §5.3): the *viable* first regime is
honest-majority, LAN/datacenter, small committed datasets (≤10³–10⁴ triples/
party), few-pattern BGPs, disclosed-property aggregates recomputed by the
verifier. Anything beyond that (dishonest-majority malicious on a WAN, heavy
BGP joins) is unquantified research risk — never budget it as "seconds".

---

## Milestones

### M0 — Scaffold + local sub-evaluation  ✅ (this milestone)
Invariant structure only; no MPC primitive, no collaborative proof.
- Crate `crates/sparq-mpc` (native-only; NOT in the wasm build — verified via
  `cargo tree -p sparq-wasm`).
- Module structure mapping to architecture sections (`backend`, `holder`,
  `join`, `partial`, `proof`).
- **REAL, tested:** per-holder local SPARQL sub-evaluation (`holder.rs`) — each
  holder evaluates a fragment over its OWN graphs via `sparq-engine` and ships
  only the disclosed partial. This is invariant to Q1/Q2.
- Trait boundaries (interfaces + docs only): `MpcBackend`, `GlobalJoin`,
  `CollaborativeProof` / `Attestation`. All crypto methods → honest
  `MpcError::NotYetImplemented` naming the gate.
- This plan; `STATUS.md`.

### M1 — ZK foundation  ⛔ HARD DEPENDENCY (gating, not optional)
The MPC proof layer is MEANINGLESS until the RQ1 verifier-soundness remediation
lands (architecture §5.1). Cross-ref `research/zk-soundness-audit.md`:
- **#3** issuer-signature + key-set membership (the attestation half is #3
  generalised to multiple sources — cannot be built before #3 lands single-
  source).
- **#4** replay/freshness binding (verifier-issued nonce bound into public
  inputs).
- **#5 / #6** FILTER operator/bound/verdict + operand-slot binding (the
  £25k/£100k threshold IS the FILTER — directly load-bearing for the use case).
- **#8 / #9** per-row source-graph attribution in-circuit + salt separation (the
  inter-source disclosure control RQ2a depends on).
- **#12** revocation (a revoked credential must not feed a federated aggregate).

(#1/#2 — public-input reconstruction + canonical-vk pinning — already fixed on
main; the proof layer plugs into that seam.)

**Gate:** do not start M3/M4 until #3/#4/#5/#6/#8/#9/#12 are closed AND have
negative e2e tests.

### M2 — `GlobalJoin` protocol  ✅ disclosed-key DONE; hidden-value now in M3
Join holders' partials on GLOBAL IRIs (architecture §2 #6, §4.3 step 4).
- **DONE ✅ — the disclosed-key equi-join** (`JoinPlan::key_disclosed == true`):
  [`DisclosedKeyJoin`] in `src/join.rs`. A crypto-free, plaintext equi-join over
  the disclosed global-IRI key, computed OUTSIDE the cryptographic core
  (convention #4). Invariant to Q1/Q2 — needs NO MPC primitive because a global
  IRI is a stable public cross-holder identifier. [OPUS-4.8]
  - **Semantics:** a faithful SPARQL inner join under PAG *compatible-mappings*
    over ALL shared columns (not just the planner-named key), folded left-to-
    right; output is the union schema, rows canonicalised so the disclosed
    multiset is order-independent.
  - **Soundness (§4.1):** does NOT trust the untrusted planner — independently
    verifies the named `join_var` is projected by every partial (else a
    `Protocol` error, never a silent empty join) and enforces compatibility on
    every shared var so a malicious plan cannot induce a result disagreeing with
    PAG eval over the union.
  - **Tested (`cargo test -p sparq-mpc`):** DIFFERENTIAL — the federated join of
    per-holder partials *equals* evaluating the whole query over the UNION of the
    holders' graphs in one `sparq-engine` store (2-holder, 3-holder chain,
    multi-row fan-out); plus empty-result holder, single-holder identity, empty
    federation, and the absent-key soundness error. Native-only invariant
    re-verified: `cargo tree -p sparq-wasm` still excludes `sparq-mpc`.
- **DONE in M3 ✅ — the hidden-value path** (`key_disclosed == false`): PRIVATE
  join values via the secret-shared equality test (circuit-PSI core). See M3
  ([`HiddenValueJoin`]). The disclosed-key [`DisclosedKeyJoin`] still routes the
  hidden regime away (it is the crypto-free path), so asking it to handle private
  keys remains an honest `NotYetImplemented` — the private capability lives in
  the dedicated type, not faked into the disclosed-key one.

### M3 — `MpcBackend` (honest-majority Shamir) + hidden-value join  ✅ DONE [OPUS-4.8]
First concrete secret-sharing impl behind the trait + the hidden-value join.
- **Q2 RESOLVED for v1: honest-majority, semi-honest** (Jesse's decision:
  honest-majority now, configurable long-term). The four flatmates *cooperate*
  among themselves (honest-but-curious) to prove an aggregate to an external
  landlord — the regime Shamir serves. LAN secret-sharing chosen (the aggregate
  is linear → zero-round under Shamir; WAN garbled-circuit unneeded for v1).
- **Scheme: Shamir `t`-of-`n` over `F_p`** (`p = 2^61-1` Mersenne, dependency-
  free `u128` reduction). Chosen over **replicated 3PC** because the use case is
  "any N cooperating flatmates" (replicated is n=3-only) and the secured
  aggregate is linear → Shamir's free local addition is the right cost profile.
  `t = ⌊(n-1)/2⌋` (honest majority). `field.rs` + `shamir.rs`. [OPUS-4.8]
- **Implemented (REAL, in-process multi-party simulation):**
  - `MpcBackend` for `ShamirBackend`: `share_private_input` (Shamir-share a
    holder's private salary), `run_secure` (cumulative sum — zero rounds),
    `reconstruct_disclosed` (Lagrange at 0 → the disclosed integer; the verifier
    recomputes `> £100k` OUTSIDE the crypto, M5).
  - Secret-shared **equality test** (`secure_equal`): `d=a-b`, mask by fresh
    nonzero `r`, one Shamir multiplication (`mul_shares_raw`, degree 2t), open
    `m=d·r`; `m==0 ⇔ a==b`, leaking ONLY the match bit. Keys never reconstructed.
  - `HiddenValueJoin`: all-pairs oblivious join on a PRIVATE key driven by the
    equality test, disclosing only the matching payload columns.
- **Tested (`cargo test -p sparq-mpc --release`, 32 pass):** DIFFERENTIAL —
  (a) secure cumulative sum == plaintext sum; (b) hidden-value join == plaintext
  inner join over the union (overlap, no-overlap, multi-match fan-out, empty
  side). Plus: field arithmetic vs reference modulo; share/reconstruct round-
  trip; the threshold actually hides (a <=t-share set is consistent with a
  DIFFERENT secret — information-theoretic hiding witness); reconstruction below
  t+1 errors; the equality primitive in isolation (n=5,t=2). Native-only
  invariant re-verified: `cargo tree -p sparq-wasm` excludes `sparq-mpc`.
- **Security model (stated, not papered over):** honest-majority **semi-honest**.
  Privacy holds while `< t+1` parties pool shares; each party learns only its
  shares + the disclosed output. **NOT malicious-secure** (guarantee D) — a
  malicious party feeding inconsistent shares is out of scope for v1. Malicious
  honest-majority (VSS / IT-MACs, ≈2×) is future hardening behind the SAME trait.
- **Honest scope / what is scaffolded (not faked):** the join is `O(|L|·|R|)`
  all-pairs (real circuit-PSI uses cuckoo bins — that ~linear optimisation is
  **Q3 / RQ2b, NOT done**); the key→`Fp` encoding is the holder's responsibility
  and needs a collision-resistant hash whose correctness is proven in-circuit in
  production (here controlled so the differential is exact); the simulation RNG
  is a deterministic SplitMix64 — **production needs a CSPRNG** for the dealer's
  masking coefficients (flagged in `shamir.rs`, not hidden).
- **Configurability (Jesse's requirement):** the trust model is a property of
  the chosen `MpcBackend` value (`BackendInfo::trust_model`), never hardcoded in
  the join/proof layer. A dishonest-majority (SPDZ/MASCOT) backend slots in
  behind the SAME trait — adds preprocessing (Beaver triples + MACs) and a
  MAC-check before opening, changes only `type Share`, leaves
  `share_private_input`/`run_secure`/`reconstruct_disclosed` SIGNATURES and all
  callers untouched. Documented in `backend.rs`.
- **Feasibility envelope (honest — minutes, not seconds):** one multiplication
  round per candidate pair → `|L|·|R|` secure equality tests for the join; the
  all-pairs structure IS the cost center the literature flags (ORQ SOSP'25: TPC-H
  joins under MPC run minutes-to-tens-of-minutes; "joins are the cost center,
  obliviousness forces worst-case padding"). Viable regime only: honest-majority,
  LAN, ≤10³–10⁴ rows/holder. Do NOT extrapolate to WAN / dishonest-majority
  (zero published data points).

### M4 — Collaborative proof + distributed attestation  ⚠️ THE HARD PROBLEM (SPIKE)
The contribution AND the principal research risk (architecture §5.2 Q1, §3.4).
**This is what remains after M3** — M3 gives confidentiality (A) over real
secret-shared computation, but NOT correctness-proof (B) or attestation (C) to a
relying party. Those are M4, and M4 is the Q1 dependency:
- **SPIKE, not routine engineering:** distributed signature/commitment-opening
  over secret-shared witnesses inside a collaborative proof — **Q1, unsolved in
  the literature** ("the join nobody has built"). M3's secret-sharing layer is
  the substrate the collaborative witness would be shared over, but verifying a
  BBS+/EdDSA signature over that shared witness inside one emitted proof is the
  open research problem; M3 does NOT touch it.
- Use PATCHED collaborative-SNARK constructions (eprint 2025/1026 soundness
  pitfalls) + a FRESH soundness audit (architecture §5.2 Q4).
- Bind: correctness (`Disclosed(π) ⊆ Eval_PAG(Q,D)`), attested source
  (issuer-key set-membership over K), query digest, FILTER, per-row attribution,
  fresh challenge.
- **Hard-blocked on M1** (the whole ZK foundation #3/#4/#5/#6/#8/#9/#12 — until
  these land, a collaborative proof composes onto a verifier that returns `Ok`
  for forged results: theatre) **AND on M3** (the chosen backend — now DONE).
  M3 → M4 is gated on Q1 + the ZK estate, NOT on more MPC-primitive work.

### M5 — Disclosed-aggregate recompute OUTSIDE the crypto
Implement convention #4 end-to-end: DISTINCT / ORDER BY / LIMIT / OFFSET /
COUNT / SUM / AVG / MIN / MAX over the DISCLOSED multiset recomputed by the
verifier; joins/UNIONs/OPTIONALs over disclosed values as out-of-circuit checks.
HAVING / GROUP BY over hidden multisets forbidden. Keeps cross-source
join/aggregation reasoning out of the cryptographic core wherever values are
disclosed (architecture §4.3 step 3).

### M6 — Performance + benchmarks
Measure the *viable* regime only (honest-majority, LAN, small data). Per
disk-space + empirical-honesty discipline: check `df` during runs, cap dataset
size, clean `/tmp` scratch, bench data git-ignored. Report real envelopes —
minutes, not "seconds" — and never extrapolate to the WAN/dishonest-majority
regime that has zero published data points.

---

## Decision-point / dependency summary
- **Q1** decided/spiked at **M4** (gates the collaborative proof + attestation) —
  **the principal thing that remains.** Unsolved in the literature; SPIKE.
- **Q2** ✅ DECIDED at **M3**: honest-majority Shamir for v1 (configurable behind
  the trait long-term). Done.
- **Q3** (BGP-join obliviousness cost / RQ2b) — surfaced concretely at **M3**:
  the hidden-value join is `O(|L|·|R|)` all-pairs; the ~linear cuckoo-bin /
  oblivious-hashing optimisation and the fragment it applies to is **still open**
  (analysis + impl remain).
- **Q4** (coZK soundness post-2025/1026) re-audited at **M4**.
- **Hard dependency:** M1 (ZK foundation #3/#4/#5/#6/#8/#9/#12) gates M4 entirely
  and is a prerequisite for any relying-party meaning.
