<!-- [OPUS-4.8] State-of-the-direction navigator + honest feasibility consolidation for the
MPC-over-federated-SPARQL track (epic sq-pwr). Read-heavy design-for-review, NO implementation.
Opus 4.8 (Fable unavailable) — re-review when Fable returns. Date: 2026-06-22. -->

# MPC over federated SPARQL with ZKP of correctness + attested-source derivation — verified state-of-the-direction + composition map

> 🤖 **SPARQ agent** [OPUS-4.8] — design-for-review record. Author: Opus 4.8 (Fable
> unavailable — flag for re-review when Fable returns). Date: 2026-06-22. Epic: **sq-pwr**
> (parent research track **sq-0jsc**).

**Status:** Design-for-review. **Read-heavy navigator, not a re-derivation.** This record exists
to (1) correct a wrong premise, (2) give the maintainer and future agents ONE verified,
code-grounded ledger of what in this direction is *implemented-and-tested* vs *designed-only* vs
*blocked*, so nobody re-derives the architecture again, and (3) pin the single genuine remaining
research frontier and the external gate that bounds every soundness claim. It makes **no** new
unqualified privacy/soundness/attestation claim.

---

## 0. Correction to the brief's premise (verified against the code, not the brief)

The task brief frames this as a **fresh** "deep-research-first, design-for-review" direction to be
designed from prior art and the maintainer's sources, with a new architecture doc to be written.
**That premise is substantially wrong, and the honest first job is to say so.**

This is in fact one of the **most densely-developed tracks in the entire repository** — it is
*designed, sequenced, beaded, AND substantially merged*. Verified against the worktree of
`origin/main` at the time of writing:

- **A full first-pass architecture already exists** — [`mpc-zkp-research-and-architecture.md`]
  (the four guarantees, the six-step protocol flow, the threat model, the RQ1-remediation hard
  gate). Writing a second architecture would *duplicate* it almost verbatim, which the
  repo-hygiene rules and that doc's own successor explicitly forbid ("does **not** re-derive any of
  them").
- **A milestone/bead map exists** — [`mpc-zkp-build-out-delta.md`] (epic `sq-pwr`, the M-A…M-F
  milestone spine).
- **The whole supporting estate exists and is current** — a per-operator **capability matrix**
  [`mpc-sparql-capability-matrix.md`], a **security-models + benchmarks** record
  [`mpc-security-models-and-benchmarks.md`], an **IT-MAC malicious-security** design
  [`mpc-malicious-security-design.md`], a **bounded property-path** operator design
  [`mpc-bounded-property-path-design.md`], an **untrusted-planner → MPC-routing seam** design
  [`mpc-untrusted-planner-routing-design.md`], an **M4 distributed-signature feasibility** verdict
  [`mpc-m4-distributed-sig-feasibility.md`], an **adversarial coZK re-audit**
  [`mpc-cozk-reaudit.md`], plus ~20 single-prover ZK design records (`zk-*.md`) and the
  **soundness-audit + re-audit** pair (`zk-soundness-audit.md` → `zk-verifier-reaudit.md`).
- **The maintainer's own prior art is already vendored INTO the repo** — his ISWC 2025
  ZKP-SPARQL ontology (with Shadbolt / J. Zhao / R. Zhao) is at
  `crates/sparq-trust/ontologies/zkp-sparql/`, MIT-licensed by the maintainer's 2026-06-21
  decision, and the sparq `secprop` ontology *extends* its `sec-prop:` namespace.
- **Most of the architecture is MERGED CODE, not paper.** M0–M3 of the MPC backend, the
  single-prover attestation binding, the in-circuit issuer-signature gadget, the modular
  commitment interfaces, the untrusted-planner routing seam, and the trust-graph admission PoC are
  all on `main` and tested (the ledger in §3 cites file:line evidence).

So the correct, honest, non-duplicative deliverable is **not** another architecture. It is this
navigator: a single verified ledger that fixes the "fresh direction" premise, confirms the
designed composition against the *actual* code, states the per-component feasibility honestly, and
points at the one frontier and the one external gate that matter. Everything substantive about
*how* the three layers compose already lives in the records above and is cited, not repeated.

---

## 1. The direction in four sentences (the architecture, faithful to the existing record)

A set of mutually-distrusting **holders** (the four-flatmates use case: each holder a Solid Pod of
issuer-signed Verifiable-Credential named graphs) jointly evaluate one federated SPARQL query over
the *union* of their privately-held graphs and return **one minimal answer** (e.g. the boolean
"cumulative salary > £100k"). The intended end-state binds three guarantees into that answer:
**(B) correctness** — the answer is the correct Pérez–Arenas–Gutiérrez evaluation of the query over
the committed data (run in **MPC**, `sparq-mpc`); **(C) attested-source derivation** — every
contributing graph commitment was issuer-signed under a key in a disclosed key-set `K` (the
**attestation/ZK** layer, `sparq-zk` / `sparq-zk-compose` + the trust ontology); and **(A)
confidentiality** — only the declared leakage envelope (join keys, the one verdict bit) is revealed,
with the disclosed/hidden split made by the **untrusted-planner routing seam** (`sparq-fedplan-mpc`).
The fourth guarantee, **(D) malicious security**, is honest-majority semi-honest by default with an
opt-in IT-MAC malicious-with-abort path in flight; the load-bearing convention throughout is
**no-proof-of-revealed-properties** (anything a deterministic function of the disclosed multiset —
DISTINCT/ORDER/LIMIT/COUNT/SUM/AVG — is recomputed by the verifier in the clear, never proven in
the cryptographic core), which is what keeps the secure surface small enough to be tractable.

For the *why*, the prior art, the trust/threat model, and the protocol flow, read
[`mpc-zkp-research-and-architecture.md`] §2–§4 — this record does not restate them.

---

## 2. The four guarantees kept distinct (the one framing worth re-pinning)

The single most important conceptual discipline the estate enforces — and the one a navigator
*should* re-surface because every honesty error in this space comes from collapsing it — is that
four guarantees are **distinct and independently satisfiable**, and three hard pitfalls are encoded
as standing assumptions:

| Guarantee | What it means here | Pitfall it is NOT |
|---|---|---|
| **(A) Confidentiality** | verifier learns only the declared leakage envelope | ≠ correctness — a "no party learns X" system can still return a wrong answer |
| **(B) Correctness** | answer = `Eval_PAG(Q, D)` over the committed data | ≠ confidentiality |
| **(C) Attestation** | each contributing `C(G_i)` is issuer-signed under a key in `K` | **commitment ≠ attestation** — proving over committed data says nothing about *who signed it* |
| **(D) Malicious security** | a deviating party is caught (abort), not just a passive one | **honest-majority ≠ secure-against-N−1** |

These map one-to-one onto the maintainer's own ISWC-2025 `sec-prop:` properties now vendored at
`crates/sparq-trust/ontologies/zkp-sparql/vocab/sec-prop.yaml.ld`: e.g. `sec-prop:Unlinkability`,
`sec-prop:SourceCredentialDisclosure` (the "which credentials sourced the answer" leak — guarantee
A), `sec-prop:CircuitAudit` (the "accepting witness implies the SPARQL semantics" obligation —
guarantee B), and the post-quantum forgery/snooping pair (the issuer-signature trust root under
guarantee C). The vendored ontology is therefore **not** background colour — it is the controlled
vocabulary the design's security claims should be expressed in, and the sparq `secprop` ontology
(`research/security-properties-ontology-design.md`, epic `sq-0dksu`) extends it rather than forking
it.

---

## 3. The verified implemented-vs-designed-vs-blocked ledger (the load-bearing contribution)

This is the part that did not exist in one place before: a **single map**, checked against the code
on `main`, of every component of the direction, so the maintainer can see at a glance what is real.
Status legend: **DONE** = on `main` and tested · **STUB** = honest scaffold that fails loudly ·
**DESIGNED** = design record exists, no/partial code · **DEFERRED/BLOCKED** = held behind a named
gate.

### 3.1 MPC layer — `crates/sparq-mpc` (guarantees B, D, and the secure half of A)

| Component | Status | Evidence |
|---|---|---|
| M0 scaffold + per-holder local sub-evaluation | **DONE** | `holder.rs`, `PLAN.md#M0` |
| M2 disclosed-key equi-join (global-IRI join, crypto-free) | **DONE** | `join.rs` (`DisclosedKeyJoin`) |
| M3 honest-majority Shamir backend (`t`-of-`n`, `F_p=2^61−1`) | **DONE** | `shamir.rs`; bead `sq-pwr.1` ✓ |
| BGW degree reduction (the arithmetic keystone) | **DONE** | `shamir.rs::degree_reduce`; matrix §reconciled |
| Hidden-value join (secret-shared equality) | **DONE** | `join.rs`, `oblivious_join.rs` |
| Secure comparison opening only the verdict bit (the £100k `>`) | **DONE** | `compare.rs` |
| Bounded-length property paths | **DONE** | `bounded_path.rs`; design `mpc-bounded-property-path-design.md` |
| Robust reconstruct (Berlekamp–Welch) / oblivious shuffle / CSPRNG masking | **DONE** | `robust.rs`, `oblivious.rs`, `chacha.rs`; `sq-1vt` ✓ |
| 3-axis configurable security model (AdversaryModel × OutputGuarantee × CorruptionThreshold) | **DONE** | `backend.rs` |
| Tier-1 benchmark harness (modelled counters + loopback + netem profiles) | **DONE** | `bench.rs`, `metrics.rs`, `netprofiles.rs` |
| IT-MAC authenticated sharing foundation | **DONE** | `authenticated.rs`; design `mpc-malicious-security-design.md` |
| Witness-validation-before-proving test obligation (coZK 2025/1026) | **DONE** | `witness_validation_tests.rs`; bead `sq-7leq` ✓ |
| Full IT-MAC malicious-with-abort (MAC-mult, batched check, registry wiring) | **DESIGNED** | `mpc-malicious-security-design.md`; `sq-km34.*` OPEN |
| ORQ-style O(n log n) sort-merge / linear circuit-PSI join | **DESIGNED** | beads `sq-ujz8` / `sq-t21` / `sq-h99` OPEN |
| **Collaborative ZK proof over secret-shared witness (`prove`/`verify`)** | **STUB** | `proof.rs` returns `MpcError::NotYetImplemented` — verified |

The pipeline driver (`pipeline.rs::run_federated`) composes holder → share → join →
secure-threshold → `ProofStatement` for the four-flatmates scenario and **honestly emits a
`FederatedResponse` with NO proof** — it assembles the public-statement *shape* the eventual proof
will bind but does not fabricate one (`proof.prove` stays the loud stub). This is correct and is the
single most important honesty fact about the whole direction.

### 3.2 Attestation / ZK layer — `crates/sparq-zk` + `crates/sparq-zk-compose` (guarantee C, single-prover)

| Component | Status | Evidence |
|---|---|---|
| In-circuit issuer-signature gadget (Schnorr over Baby-JubJub, Poseidon2 challenge) | **DONE** | `sparq-zk/src/sig.rs`; bead `sq-z9l` ✓ |
| Modular commit interface (Poseidon2 leaf commit; closed method enum + open signature trait) | **DONE** | `sparq-zk/src/commit.rs`, `registry.rs` |
| Verifier public-input reconstruction (binds challenge/nonce, commitments, FILTER operands, per-row source attribution) | **DONE** | `sparq-zk-compose/src/verifier.rs::reconstruct_public_inputs` |
| Issuer-attestation binding (key ∈ external `K`, salt-bound sig verify, status-list ref, salt uniqueness) | **DONE** | `verifier.rs::bind_issuer_attestations`; tests in `tests/forge_gates.rs` |
| Hidden-issuer set-membership (Merkle membership over `K`, hides *which* issuer) | **DONE** | `verifier.rs::bind_hidden_issuer_attestations` |
| Hidden cross-credential join (joined entity out of public inputs) | **DONE** | `sq-bwwl` ✓ (PR #170) |
| RQ1 verifier-soundness remediation (#3/#4/#5/#6/#8/#9/#12) | **DONE** | epic `sq-1s2` 17/17; re-audit `zk-verifier-reaudit.md` (`sq-gbp4`) |
| **Federated / multi-source `reconstruct_public_inputs` layout + out-of-circuit freshness binding** | **DESIGNED** | bead `sq-34ml` OPEN (M4-v1 prerequisite) |

Crucial nuance the survey corrected: `bind_issuer_attestations` already iterates a
`Vec<CommitmentAttestation>`, so it is **multi-attestation single-prover** today — but the
*federated* layout spanning all holders' shares/rows/attribution under one statement (sq-34ml) is
the missing wiring, and the binding is **verifier-side / out-of-circuit**, not the in-circuit
distributed gadget.

### 3.3 Attested-source / trust layer — `crates/sparq-trust` + vendored ontology (guarantee C, policy side)

| Component | Status | Evidence |
|---|---|---|
| Vendored ISWC-2025 `sec-prop:`/`sig-impl:`/`sec-req:`/`prov-ext:` ontology (MIT) | **DONE (data)** | `crates/sparq-trust/ontologies/zkp-sparql/`; `PROVENANCE.md` |
| Claim-level admission (canonicalise + commit, CHECKED issuer sig, SHACL scope, trust:scope containment) | **DONE (PoC)** | `sparq-trust/src/admit.rs::admit_static` |
| DID resolution (`did:key` offline; `did:web` pluggable, opt-in) | **DONE (PoC, opt-in)** | `sparq-trust/src/did.rs`; `sq-pfae.3` |
| `secprop` property-admissibility pre-check (consume the vendored `sec-prop:` graph in `admit.rs`) | **DESIGNED** | bead `sq-dt5hv` OPEN |

`sparq-trust` is explicitly a **research-grade PoC**, not a shipped security feature (its README
says so): clear-text admission, no unlinkability, operator-asserted issuer keys by default.

### 3.4 Federation-routing seam — `crates/sparq-fedplan-mpc` (the disclosed/hidden split, guarantee A's planning side)

| Component | Status | Evidence |
|---|---|---|
| Source-privacy descriptor + privacy-aware source selection (default-deny) | **DONE** | `privacy.rs`, `selection.rs`; `sq-pwr` Phase 1–2 |
| Disclosed/hidden routing pass (`route_operators`, policy-parameterised) | **DONE** | `routing.rs`; bead `sq-i1wh2` ✓ |
| Leakage-envelope assembly + dual (holder fail-closed + verifier budget) ratification | **DONE** | `envelope.rs`; bead `sq-pwr.2` ✓ |
| FedUP-style result-aware source-combination pruning | **DONE** | `sq-pwr.3` ✓ |
| Untrusted-plan **soundness** re-validation (bind plan to the collaborative proof) | **BLOCKED** | depends on `proof.rs` ceasing to be a stub + audits |

### 3.5 PROV-O attested-derivation (the complementary line, epic `sq-2489d`)

The "attested/provenance-bound source derivation" the brief links to the PROV-O GenAI-KB work is a
**distinct, complementary track**, not the same crypto. `research/provenance-driven-genai-kb.md`
(bead `sq-bxse0`) makes PROV-O / DQV / CiTO load-bearing for the *knowledge-base* layer
(quality→embedding-weight, citations, answer-qualification) — it is **soft, declarative
provenance** for agent KM, where the MPC+ZKP track is **hard, cryptographic** attestation of query
results. They share the vocabulary instinct (`prov:wasDerivedFrom` is in both the vendored
`prov-ext:` and the GenAI-KB design) but address different threat models. The honest relationship:
the GenAI-KB line is the *unauthenticated* provenance layer for the agent's own KB; the MPC+ZKP line
is the *authenticated* derivation layer for a relying party. **Do not conflate them**; the brief's
linkage is real at the vocabulary level only. (Note that `sq-2489d` is also flagged
`revisit-with-fable` — its verdicts are model-dependent.)

---

## 4. The opt-in, feature-gated crate layout (confirmed against `main`, not proposed afresh)

The brief asks for "an OPT-IN, feature-gated crate layout (keep sparq-core/engine lean)." This
**already exists and is verified** — it is not a new proposal. The entire direction lives in
peripheral opt-in members; the lean core (`sparq-core` / `sparq-engine`) has **zero** dependency on
any of it:

- `sparq-mpc` — MPC primitives + protocol (the secure compute); insecure RNG behind the
  off-by-default `insecure-test-rng` feature.
- `sparq-zk` / `sparq-zk-compose` — single-prover commitment / circuit / verifier estate; the
  research-only value-only commitment behind `commitment-value-only`.
- `sparq-trust` — trust-graph admission PoC + the vendored ontology (data only; nothing
  `include_str!`s the ontology today, so vendoring it did not change the build).
- `sparq-fedplan-mpc` — the untrusted-planner → routing glue, off by default (`fedplan-mpc`
  feature), depending on `sparq-fedplan` + `sparq-mpc` but coupling neither into the other.

The one architectural recommendation worth restating because it is a *decision the maintainer owns*:
the routing-seam design (`mpc-untrusted-planner-routing-design.md` §9 Q1) deliberately chose a
**standalone `sparq-fedplan-mpc` crate** over a `mpc` feature inside `sparq-fedplan`, to keep the
trusted local cost-planner uncoupled from the untrusted-plan + privacy-policy concern. That choice
is already implemented; it stands, and this navigator endorses it.

---

## 5. Honest feasibility verdict, per component (non-sycophantic)

| Sub-component | Verdict | Honest basis |
|---|---|---|
| **MPC correctness (B), honest-majority semi-honest, LAN, ≤10³–10⁴ triples/party, few-pattern BGP, verifier-recomputed aggregates** | **FEASIBLE — and substantially BUILT** | M0–M3 + degree-reduction + secure-compare + bounded paths are on `main` and tested; aligns with the published sub-second-for-≤10³-triples single-prover anchor |
| **MPC malicious-with-abort (D), honest-majority IT-MAC** | **FEASIBLE — designed + foundation built, finish in flight** | `authenticated.rs` lands; `sq-km34.*` enumerates the remaining MAC-mult/batched-check work; no ZK gate (pure MPC layer) |
| **Verifier-side attestation (C, federated, the M4-v1 interim)** | **FEASIBLE-BUT-UNBUILT — the buildable next step** | single-prover binding exists (`bind_issuer_attestations`); needs the federated layout + freshness binding (`sq-34ml`) then the gate assembly (`sq-f7bu`); checks the sig *out-of-circuit*, so it **gives up source-unlinkability and a single succinct sig↔witness proof** |
| **In-circuit distributed signature over secret-shared witness (C, source-unlinkable, the thesis novelty)** | **RESEARCH-GRADE, UNSOLVED — correctly DEFERRED** | "the join nobody has built" — no published system verifies an issuer signature over a *secret-shared* witness inside a collaborative proof; held as spike `sq-bjl`; budget as research risk, **never** "seconds" |
| **Confidentiality routing (A), disclosed/hidden split + leakage envelope** | **FEASIBLE — BUILT** | `sparq-fedplan-mpc` routing + envelope + dual ratification on `main` and differentially tested |
| **Dishonest-majority malicious correctness for ANY SPARQL operator, WAN** | **NOT ACHIEVED IN THE LITERATURE — deliberately scoped out** | no published system delivers it; the registry **fail-closes** rather than downgrading; carried as future research (`sq-j5ok`/`sq-38zk`), not capability |
| **Any production soundness/attestation/privacy CLAIM (all of the above)** | **HARD-GATED — EXTERNAL sign-off PENDING** | the entire ZK soundness rests on *internal, single-model (Opus 4.8)* audits; the external accredited-cryptographer audit `sq-qhy4` (P0) has **not** run; SECURITY.md correctly publishes the conservative "treat as untrusted" posture until it does |

**The single honest headline:** the *correctness + confidentiality + verifier-side-attestation*
composition is feasible and largely **already built** in the viable regime (honest-majority,
cooperating holders, LAN, small data); the *source-unlinkable in-circuit attestation* is the one
genuine research frontier and is correctly deferred; and **no security property may be presented as
proven to a relying party until `sq-qhy4` lands** — because today the verifying party would be
trusting a chain of self-audits. The maintainer's own `sec-prop:CircuitAudit` open-question
("closing the full algebra-fragment proof") and the standing `sq-qhy4` gate say the same thing.

### A note on the maintainer's "could be crap" self-assessment

The vendored-ontology provenance note records that the maintainer is an unreliable judge of his own
work's value ("could be crap" self-assessment — verify before discounting). Verified here: the
vendored `sec-prop:` ontology is **not** crap — it is a clean, SHACL-validated, provenance-stamped,
eight-property security vocabulary that maps directly onto this design's four guarantees and is the
right controlled vocabulary for the design's claims. It should be *used* (consumed by `admit.rs` per
`sq-dt5hv`, and as the vocabulary for any security-property assertions), not merely cited. The one
thing it cannot self-certify is the **soundness** of the ZK constructions those properties describe
— that is exactly what `sq-qhy4` exists to provide, and no ontology substitutes for it.

---

## 6. Genuinely new gaps this navigator surfaces (small, honest)

The estate is so complete that almost nothing here is un-beaded. After deduplicating against
`sq-pwr`'s 11 children, `sq-km34.*`, `sq-0jsc`, and the routing-seam phases, the only genuinely
*new* items this navigator identifies are documentation/navigation hygiene, not new engineering:

1. **No single cross-doc navigator existed for this direction** until this record — the architecture,
   delta, matrix, security-models, malicious-security, bounded-path, routing-seam, M4-feasibility,
   coZK-reaudit, and ~20 ZK docs were each correct but **un-indexed**, which is precisely why a
   "fresh direction" brief was issued for an already-saturated track. A standing index entry (this
   doc, plus a one-line pointer from `AGENTS.md` / a `SKILL.md`) prevents the next re-derivation.
2. **The brief↔reality drift is itself a recurring failure mode** worth a tracked guard: research
   briefs for this track keep assuming it is unstarted. The mitigation is the index above; no code
   bead is warranted, but the orchestrator should route future MPC/ZK research briefs through the
   index first.

Everything *substantive* (M4-v1 gate assembly `sq-f7bu`, its prerequisites `sq-34ml`, the deferred
in-circuit spike `sq-bjl`, the IT-MAC finish `sq-km34.*`, the SOTA-join replacements
`sq-ujz8`/`sq-t21`/`sq-h99`, the composition/UC records `sq-aaop`/`sq-wj4k`, the DM/WAN frontier,
the `secprop` pre-check `sq-dt5hv`, and the external audit `sq-qhy4`) is **already beaded**. The
honest outcome the brief allows for — "no large new milestone tree needed" — holds, more strongly
than the build-out-delta even claimed: the build-out-delta's three filed deltas (`sq-f7bu`,
`sq-9hrn`, the matrix refresh) have since landed (`sq-9hrn` ✓, matrix reconciled), leaving `sq-f7bu`
as the one OPEN buildable next step gated by `sq-34ml`.

---

## 7. Recommendation

1. **Do not write or commission another architecture for this direction.** It exists, is current,
   and is partly stale only at the bead-status level (already reconciled in the capability matrix).
   Treat [`mpc-zkp-research-and-architecture.md`] as canonical; treat *this* doc as its index +
   verified status ledger.
2. **The next buildable step is `sq-f7bu`** (the M4-v1 verifier-side authenticated-input attestation
   gate), unblocked once **`sq-34ml`** (federated `reconstruct_public_inputs` layout +
   out-of-circuit freshness binding) lands. Both are honest-majority, no-new-crypto, opt-in, and
   ship behind the loud "research, unaudited collaborative path" label. Build them deliberately with
   the witness-validation obligation (`sq-7leq`, ✓) honoured.
3. **Keep `sq-bjl` deferred.** The in-circuit distributed signature over a secret-shared witness is
   the thesis novelty and is unsolved in the literature; it is two research steps out and must never
   be presented with a performance number.
4. **Treat `sq-qhy4` as the master gate on every claim.** Until the external cryptographer signs off,
   no soundness/attestation/privacy property is production-claimable; the live privacy-claims CI gate
   and SECURITY.md's conservative posture are correct and must not be softened.
5. **Use the vendored `sec-prop:` vocabulary as the claim language** (and wire `sq-dt5hv` to consume
   it), rather than re-coining security-property terms.

---

## 8. Phased plan (ordered; each item is an EXISTING bead — nothing new to file as engineering)

This plan is deliberately a *pointer* list, because the engineering is already beaded; ordering them
is the value. Each is opt-in, touches neither the lean core nor the audit posture.

1. **`sq-34ml`** *(OPEN, P2)* — M4-v1 prerequisites: federated/multi-source
   `reconstruct_public_inputs` layout + explicit out-of-circuit freshness/replay binding. The
   buildable next step's foundation.
2. **`sq-f7bu`** *(OPEN, P2, depends `sq-34ml`)* — M4-v1 verifier-side authenticated-input
   attestation gate assembly (Dutta/Artemis) → the verifiable federated four-flatmates response;
   negative e2e tests (forged source / replayed manifest / out-of-`K` key all REJECT). Ship behind
   the opt-in, "research, unaudited" label. **Not production-claimable until `sq-qhy4`.**
3. **`sq-km34.*`** *(OPEN, P2, no ZK gate)* — finish honest-majority IT-MAC malicious-with-abort
   (MAC-carrying mult, batched MAC-check, malicious equality/comparison, registry wiring, adversarial
   catch-tests, the AXIS-1 bench lift). Internally actionable now.
4. **`sq-ujz8` / `sq-t21` / `sq-h99`** *(OPEN)* — replace the all-pairs hidden join with the
   ORQ-style O(n log n) sort-merge / linear circuit-PSI join; differentially tested against the
   all-pairs oracle. Internally actionable now.
5. **`sq-dt5hv`** *(OPEN)* — `secprop` property-admissibility pre-check in `sparq-trust::admit.rs`
   consuming the vendored `sec-prop:`/`sig-impl:` graphs. Wires the maintainer's ontology into the
   admission path.
6. **`sq-wj4k`** *(DONE)* / **`sq-aaop`** *(OPEN)* — the composition / UC posture design records
   (justify the mid-pipeline `secure_equal` open; honest-majority-UC-without-setup; carry the coZK
   2025/1026 caveat). `sq-wj4k` is recorded in
   [`mpc-composition-uc-posture.md`](./mpc-composition-uc-posture.md) (posture + per-stage
   obligations); `sq-aaop`, if retained, is the formal simulator-sketch/paper-grade write-up. Gate
   the collaborative-proof soundness story.
7. **`sq-bjl`** *(DEFERRED, P1)* — the in-circuit distributed-signature-over-secret-shared-witness
   SPIKE. Research-novel; do only as a deliberate, audit-gated spike after 1–2 + the audits.
8. **`sq-qhy4`** *(OPEN, P0, EXTERNAL)* — the accredited-cryptographer audit that gates **every**
   production claim above. The only true external blocker; agent-out-of-scope by definition.

---

## 9. Open questions that genuinely need the maintainer

1. **Index placement.** Should this navigator's index role be promoted into a durable home —
   `AGENTS.md` § on the MPC/ZK estate, or a `skills/<surface>/SKILL.md` — so future research briefs
   for this track hit the index *before* a "fresh direction" prompt is issued? (The recurring
   brief↔reality drift in §6 is the symptom; the fix is a one-line standing pointer, which is a
   structure call the maintainer owns.)
2. **M4-v1 disclosure posture.** The verifier-side interim (`sq-f7bu`) *reveals which issuer signed*
   (`pk_i` checked in clear), trading source-unlinkability for buildability. Is that an acceptable v1
   for the driving use case (the four cooperating flatmates may not care which of *them* signed,
   only that the landlord accepts), or is source-unlinkability a hard v1 requirement that forces the
   deferred in-circuit spike up the priority order?
3. **`sec-prop:` as a runtime artefact.** The vendored ontology is currently *data* nothing loads.
   Beyond the `sq-dt5hv` admission pre-check, do you want the `sec-prop:` verdicts surfaced at query
   time (a presented response self-describing which security properties it attains/fails), or kept
   as documentation-grade vocabulary only?

---

### Cross-references (canonical, not duplicated here)

- Architecture (canonical): [`mpc-zkp-research-and-architecture.md`](./mpc-zkp-research-and-architecture.md)
- Milestone/bead spine: [`mpc-zkp-build-out-delta.md`](./mpc-zkp-build-out-delta.md)
- Per-operator capability tiers: [`mpc-sparql-capability-matrix.md`](./mpc-sparql-capability-matrix.md)
- Security models + benchmarks: [`mpc-security-models-and-benchmarks.md`](./mpc-security-models-and-benchmarks.md)
- Malicious security (IT-MAC): [`mpc-malicious-security-design.md`](./mpc-malicious-security-design.md)
- Bounded property paths: [`mpc-bounded-property-path-design.md`](./mpc-bounded-property-path-design.md)
- Untrusted-planner routing seam: [`mpc-untrusted-planner-routing-design.md`](./mpc-untrusted-planner-routing-design.md)
- M4 distributed-sig feasibility: [`mpc-m4-distributed-sig-feasibility.md`](./mpc-m4-distributed-sig-feasibility.md)
- Adversarial coZK re-audit: [`mpc-cozk-reaudit.md`](./mpc-cozk-reaudit.md)
- ZK soundness audit + re-audit: [`zk-soundness-audit.md`](./zk-soundness-audit.md), [`zk-verifier-reaudit.md`](./zk-verifier-reaudit.md)
- Complementary (soft) provenance line: [`provenance-driven-genai-kb.md`](./provenance-driven-genai-kb.md)
- Maintainer's vendored prior art: `crates/sparq-trust/ontologies/zkp-sparql/` (`PROVENANCE.md`)
