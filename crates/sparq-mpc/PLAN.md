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

### M2 — `GlobalJoin` protocol
Join holders' partials on GLOBAL IRIs (architecture §2 #6, §4.3 step 4).
- First: the **disclosed-key equi-join** path (`JoinPlan::key_disclosed`) —
  plaintext join over disclosed IRIs OUTSIDE the cryptographic core (convention
  #4). Crypto-free, invariant to Q1/Q2 — the natural first implementation.
- Then: design the **hidden-value** path (circuit-PSI / oblivious join) — gated
  on M3's backend and on **Q3** (BGP-join obliviousness cost; how much the
  out-of-circuit handling collapses, and for which fragment — RQ2b).
- Soundness obligation: the join must NOT trust the untrusted planner for
  correctness (architecture §4.1).

### M3 — `MpcBackend` (honest-majority first)
First concrete secret-sharing impl behind the trait (architecture §3.1, §4.2).
- **DECISION POINT Q2** must be resolved first: confirm honest-majority is
  acceptable for the use case (cooperating flatmates vs external landlord), and
  pick LAN secret-sharing vs WAN garbled-circuit.
- Implement honest-majority (replicated 3PC SS) `share_private_input` /
  `run_secure` / `reconstruct_disclosed` for the cumulative-aggregate sub-case.
- Dishonest-majority remains future research, swappable behind the same trait.

### M4 — Collaborative proof + distributed attestation  ⚠️ THE HARD PROBLEM (SPIKE)
The contribution AND the principal research risk (architecture §5.2 Q1, §3.4).
- **SPIKE, not routine engineering:** distributed signature/commitment-opening
  over secret-shared witnesses inside a collaborative proof.
- Use PATCHED collaborative-SNARK constructions (eprint 2025/1026 soundness
  pitfalls) + a FRESH soundness audit (architecture §5.2 Q4).
- Bind: correctness (`Disclosed(π) ⊆ Eval_PAG(Q,D)`), attested source
  (issuer-key set-membership over K), query digest, FILTER, per-row attribution,
  fresh challenge.
- Hard-blocked on M1 (the whole ZK foundation) and on M3 (the chosen backend).

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
- **Q1** decided/spiked at **M4** (gates the collaborative proof + attestation).
- **Q2** decided at **M3** (gates the first MpcBackend; influences M2's hidden
  path).
- **Q3** (BGP-join obliviousness cost / RQ2b) analysed at **M2** hidden path.
- **Q4** (coZK soundness post-2025/1026) re-audited at **M4**.
- **Hard dependency:** M1 (ZK foundation #3/#4/#5/#6/#8/#9/#12) gates M4 entirely
  and is a prerequisite for any relying-party meaning.
