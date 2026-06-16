<!-- [OPUS-4.8] Adversarial coZK soundness RE-AUDIT vs eprint 2025/1026 (bead sq-9hrn, epic
sq-pwr / milestone M-D), run + consolidated by Opus 4.8 (Fable unavailable) — re-review when
Fable returns. ADVERSARIAL audit deliverable: surfaces risks, does NOT certify soundness. -->

# Adversarial coZK soundness re-audit — collaborative-proof path vs eprint 2025/1026

Bead **sq-9hrn** · epic **sq-pwr** (MPC over federated SPARQL + ZKP of correctness +
attested-source derivation) · milestone **M-D**.

Adversarial re-audit of sparq's **collaborative** (multi-prover) zk-proof path against the
CRYPTO'25 pitfalls in **eprint 2025/1026** (Garg–Goel–Jain–Roberts–Sekar, *Malicious
Security in Collaborative zk-SNARKs: More than Meets the Eye*,
https://eprint.iacr.org/2025/1026). This is **forward-looking**: the collaborative path is
**not built** — every method in `crates/sparq-mpc/src/proof.rs` returns
`MpcError::NotYetImplemented`, and the in-circuit distributed-signature-over-secret-shared-
witness join is the deferred spike (`sq-bjl` / M-E) named "the join nobody has built". So
this audit governs the **design + the intended stack**; it is not a code-soundness verdict
on a running prover (none exists).

Distinct from: the single-prover verifier audit/re-audit (`research/zk-soundness-audit.md`,
`research/zk-verifier-reaudit.md`; bead `sq-qhy4` the external single-prover audit), and from
the UC/composition **design records** (`sq-aaop`/`sq-wj4k`). Those cover the single-prover ZK
estate and the composition framing; **this** is the adversarial coZK re-audit of the
multi-prover path that none of them performs.

---

## Bottom-line verdict — RE-OPEN (gating)

**The collaborative-proving soundness claim is RE-OPEN and remains gated.** No production
attestation/correctness claim over the collaborative path may be made until (a) the path is
actually built, (b) the witness-validation-before-proving precondition of 2025/1026 is
encoded and enforced (the test obligation in §3 below), and (c) an external cryptographer
audit covers the *multi-prover* construction (not just the single-prover verifier `sq-qhy4`).

This RE-OPEN is **not** a finding of a present exploitable hole — there is no shippable
collaborative prover to forge against. It is the honest disposition for an **unbuilt,
audit-gated** path against documented coZK failure modes: the pitfalls 2025/1026 describes
are *exactly* the ones the intended stack would inherit, the literature's "fix" is a
non-trivial precondition sparq has named but not yet encoded as an enforceable check, and the
closest usable implementation (TACEO co-snarks) is itself unaudited and predates the paper.
**The current honest posture — every collaborative-proof method fails closed with a
gate-naming `NotYetImplemented` (`proof.rs:135-150`, contract-tested) — is the correct state;
this audit makes the gate *enforceable at build time* rather than only documentary.**

Per-lens summary (detail in §2):

| # | Lens (2025/1026 failure mode) | Verdict | Why |
|---|---|---|---|
| 1 | Witness-extension leakage (proving an inconsistent/extended witness leaks honest inputs) | **RE-OPEN** | precondition named, not yet encoded as an enforceable validate-before-prove check; the mid-pipeline open (Hole 1/3) is the live instance |
| 2 | Malicious-compiler insecurity (SOTA semi→malicious compilers unsafe as-is) | **RE-OPEN (mitigated-by-design intent, unverified)** | crate intends honest-majority "free" malicious security (Goyal–Song), NOT an off-the-shelf compiler — but no collaborative-proving compiler is selected/audited yet |
| 3 | Honest-majority semi-honest ≈ malicious-secure **iff** extended witness validated before proving | **RE-OPEN (precondition unmet/unencoded)** | trust model matches the "iff" antecedent; the consequent (validate-before-prove) is the unfilled obligation |
| 4 | TACEO co-snarks (coNoir↔Barretenberg) unaudited + predates 2025/1026 | **RE-OPEN (dependency risk, flagged)** | the closest usable stack is explicitly unaudited; adopting it imports an unaudited TCB |

---

## How the lenses map onto the intended stack

The intended collaborative path (architecture §4.3 step 5; `proof.rs`; M4 feasibility doc
§2/§4) is: **N holders are the provers**; each holds a private witness (its committed graph
`C(G_i)`, salaries, row encodings); they run an honest-majority **MPC** to compute the
federated correctness relation (aggregate/threshold over secret-shared values) and to jointly
**produce one zk proof** over the secret-shared witness, verifiable by an **unchanged
single-prover UltraHonk verifier**. This is precisely the coZK template 2025/1026 studies: a
semi-honest MPC prover, optionally lifted toward malicious security.

2025/1026's two headline results bite here directly:
- **Privacy is not free.** Even where *soundness* survives, a malicious prover who steers the
  MPC onto an **inconsistent/extended witness** and then induces an opening can **exfiltrate
  honest provers' private witness bits**. In sparq this is not hypothetical at the MPC layer:
  the crate already documents (design `mpc-malicious-security-design.md` §1 Holes 1–4) that
  the degree-`2t` mid-pipeline opens at minimal `n=2t+1` carry **zero RS redundancy**, so a
  tampered share is undetectable AND — per 2025/1026 — opening a value computed on an
  inconsistent witness is a **confidentiality** hole, not only a correctness one.
- **"Free" malicious security is conditional.** The folklore "honest-majority semi-honest ⇒
  malicious-secure for free" holds for coZK **only if the extended witness is validated
  before proving**, and **SOTA semi-honest→malicious compilers are insecure in general** if
  applied naively. The crate's malicious-security plan leans on the honest-majority "free"
  result (Goyal–Song, `backend.rs:1522` comment; `authenticated.rs` §header) — which is the
  *right family* — but the collaborative-proving compiler that would lift the joint-proving
  MPC is not selected, and the validate-before-prove antecedent is not encoded.

---

## Per-lens dispositions

### Lens 1 — Witness-extension leakage — **RE-OPEN**

**The pitfall.** In coZK, the prover's witness is *extended* (intermediate wire values,
auxiliary advice) beyond the honest inputs. 2025/1026 shows that if the provers prove over an
**inconsistent** extended witness — one not consistent with honest inputs — the protocol can
**leak the honest provers' private inputs** through the proving transcript / induced openings,
even when the verifier still rejects the proof. The defence is to **validate the extended
witness for consistency *before* the proving phase opens or commits to anything derived from
it.**

**Does sparq's design validate the extended witness before proving?** Not yet — and it cannot,
because the proving phase does not exist. What *does* exist is the **MPC-layer** instance of
the same leak, fully documented and currently fail-closed:
- The mid-pipeline degree-`2t` open in `secure_equal`/`join.rs` and the masked-open in
  `compare.rs` are exactly "open a value computed on the (possibly inconsistent) witness."
  At minimal `n=2t+1` there is **zero redundancy** to catch an inconsistent share
  (`mpc-malicious-security-design.md` §1 Hole 1/3; adversarial test
  `tampered_share_in_secure_equality_open_is_undetectable_at_n_eq_2t_plus_1`,
  `adversarial_tests.rs:260`). The design doc names the 2025/1026 confidentiality interaction
  explicitly (§1 Hole 1: "opening a value computed on an *inconsistent witness* can leak
  honest inputs").
- The crate's **planned** mitigation is the IT-MAC line (sq-km34.\*): authenticate every
  shared value with a session-global secret `[α]`, and **batch-MAC-check before any open**, so
  a tampered/inconsistent share is caught *before* the leak path fires
  (`mpc-malicious-security-design.md` §2.5, "Confidentiality-before-open discipline (coZK
  2025/1026)"; "aborts *before* the inconsistent-witness leak path can be exploited"). That
  MAC-check-before-open is the **MPC-layer analogue** of the paper's validate-before-prove
  defence — correct in intent, **not yet landed** (foundation `authenticated.rs`/sq-km34.1 is
  the carrier only; the MAC-carrying multiplication sq-km34.2 and the batched check sq-km34.4
  are OPEN).

For the **collaborative-proving** layer specifically (not the MPC aggregate), the
validate-the-extended-witness-before-proving check has **no design artefact and no test
obligation** today. The compose crate's single-prover circuits enforce in-circuit row/commit
soundness (`scan.nr`), but a *collaborative* prover must additionally ensure the **shared**
extended witness is consistent across holders *before* the joint proving step opens any
derived value. **RE-OPEN.** The encoded test obligation in §3 fills this gap so it is
enforceable when sq-f7bu/sq-bjl is built.

### Lens 2 — Malicious-compiler insecurity — **RE-OPEN (mitigated-by-design intent, unverified)**

**The pitfall.** 2025/1026 shows that taking a semi-honest coZK prover and applying a SOTA
generic semi-honest→malicious MPC compiler **as-is** does **not** in general yield a
malicious-secure collaborative zk-SNARK — the compiler's guarantees do not transfer cleanly to
the proving setting; specific constructions are insecure.

**Is sparq relying on such a compiler?** **Not at the design level** — and this is a point in
sparq's favour, but it is unverified because no collaborative-proving backend is selected:
- The crate's stated malicious-security route is **honest-majority "free" malicious security**
  (Goyal–Song eprint 2020/134, `backend.rs:1522`; `authenticated.rs` header) via IT-MACs
  (SPDZ-family MAC, but honest-majority-scoped), **not** a generic semi→malicious compiler
  bolted onto a dishonest-majority semi-honest prover. The PLAN (`PLAN.md:257`) explicitly
  says "use PATCHED collaborative-SNARK constructions (eprint 2025/1026 soundness …)" and
  defers the coZK soundness question to a re-audit at M4 (`PLAN.md:315`, Q4) — i.e. it does
  **not** assume an off-the-shelf compiler is safe.
- **But:** (a) no collaborative-*proving* compiler/construction has actually been chosen,
  patched, and audited; the design only states the *intent* to use a patched one. (b) The
  honest-majority "free" result is itself the thing 2025/1026 qualifies (Lens 3) — it is
  "free" *only with* the validate-before-prove precondition. So the mitigation-by-design is
  **sound as a stated principle but unverified as an artefact**.

**RE-OPEN.** The verdict is not "sparq relies on an insecure compiler" (it does not, by
design); it is "no audited, patched collaborative-proving construction exists in the codebase
yet, so the claim that the eventual one avoids this pitfall is unverifiable today." The fix is
a hard requirement, encoded as: **any collaborative-proving construction adopted MUST cite the
2025/1026-patched variant it uses and MUST NOT be a naive application of a generic
semi→malicious compiler** — see the test obligation §3 (clause C) and bead below.

### Lens 3 — HM-semi-honest ≈ malicious-secure **iff** extended witness validated before proving — **RE-OPEN (precondition unmet/unencoded)**

**The conditional.** 2025/1026's positive result: an honest-majority semi-honest coZK prover
*does* attain malicious security **for free**, **but only if** the extended witness is
validated for consistency before proving. The antecedent (honest majority) sparq's trust model
**satisfies by construction** — the whole crate is honest-majority-first
(`shamir.rs`/`backend.rs` `CorruptionThreshold::HonestMajority`, Cleve enforced as a type
invariant; dishonest-majority is deliberately scoped out and the registry fail-closes).

So sparq sits squarely in the regime where the "free" result *could* apply — which makes the
**consequent** the whole game. **Is the validate-before-proving precondition met or even
encodable?**
- **Met:** No — it is not implemented (no proving phase exists).
- **Encodable:** Yes — and that is the actionable outcome of this audit. The precondition
  decomposes into two checkable obligations on the future implementation: **(i)** before the
  joint proving step opens/commits to any value derived from the extended witness, the **shared
  extended witness is validated consistent** (each holder's contribution is a well-formed
  sharing consistent with its committed `C(G_i)` and with the cross-holder join), and **(ii)**
  this validation **gates** proving — an inconsistent/extended witness causes a **fail-closed
  abort before any opening**, never a "prove anyway and let the verifier reject" path (which is
  exactly the leak path of Lens 1).

**RE-OPEN.** The trust model makes the "free malicious security" reachable, but the
load-bearing precondition is unfilled. This is the single most important gating requirement
for the collaborative path, and §3 encodes it as a concrete, enforceable test obligation.

### Lens 4 — TACEO co-snarks (coNoir↔Barretenberg) unaudited + predates 2025/1026 — **RE-OPEN (dependency risk, flagged honestly)**

**The dependency.** The closest usable stack to sparq's verifier is **TACEO co-snarks**
(`coNoir → UltraHonk/Barretenberg`), which matches sparq's exact single-prover verifier
(`zk/compose` compiles Noir → UltraHonk; the verifier recomputes a canonical Barretenberg vk).
co-snarks is the only off-the-shelf implementation that would let sparq reuse its unchanged
verifier under a collaborative prover.

**The risk, stated plainly:**
- co-snarks is **explicitly UNAUDITED** (its own repository/docs carry no security audit), and
  its design **predates 2025/1026** — its documentation does not reference the CRYPTO'25
  pitfalls. Adopting it imports an **unaudited cryptographic TCB** into the most
  security-critical path (the thing that would let a relying party believe a federated
  attestation).
- Because it predates the paper, there is **no evidence** it implements the validate-before-
  prove precondition (Lens 3) or avoids the malicious-compiler pitfall (Lens 2). It may; it may
  not — the point is it is **unverified against the very failure modes this audit is about.**

**RE-OPEN.** Any plan that depends on TACEO co-snarks for the collaborative proof MUST treat
that dependency as **unaudited-until-proven-otherwise**: pin the version, re-run this
2025/1026 lens-set against the *specific* co-snarks construction adopted, and gate any
production claim on either an external audit of that co-snarks version or an in-house
construction that is itself audited. Do **not** present a co-snarks-backed collaborative proof
as proving attestation/correctness to a relying party absent that audit.

---

## 3. The encoded requirement — witness-validation-before-proving as a TEST OBLIGATION

This is the deliverable that makes the gate enforceable when the collaborative-proof
implementation lands (sq-f7bu / sq-bjl, M-E). It is written so it can be lifted directly into
the adversarial test suite (`crates/sparq-mpc/src/adversarial_tests.rs` / a new
`collaborative_proof_witness_validation` module) at build time.

**Requirement (R-WV).** *A collaborative-proving implementation MUST validate the shared
extended witness for cross-holder consistency, and MUST abort fail-closed BEFORE any value
derived from the extended witness is opened or committed into the joint proof, whenever the
extended witness is inconsistent or maliciously extended.* No "prove-anyway-and-reject" path
may exist (that is the 2025/1026 leak path).

**Test obligation (must all pass before any collaborative-proving soundness/attestation claim):**

- **T1 — inconsistent-share abort-before-open (Lens 1/3).** Construct an adversarial holder
  that contributes a **degree-inconsistent** (or off-codeword) share of its witness value
  (model after `adversarial_tests.rs::corrupt_one`, applied to the witness sharing rather than
  the open). Drive the collaborative prove path. **MUST:** the path returns a fail-closed abort
  (`MpcError` of the malicious-abort / MAC-check kind) **before** any open or proof-commit step
  executes — assert via an instrumented transport/round-counter that **zero** open-rounds and
  zero proof-commitment of any value derived from the witness occurred after the inconsistency
  was introduced. **MUST NOT:** produce a proof (even a rejected one) or open any derived value.

- **T2 — witness-extension leakage probe (Lens 1).** Construct a malicious prover that proves
  over an **extended witness inconsistent with its committed `C(G_i)`** (e.g. claims row
  encodings that do not fold to its signed commitment). **MUST:** abort before proving, with an
  error citing the consistency failure. **Adversarial assertion:** an honest holder's private
  witness bits (e.g. another flatmate's salary share) are **information-theoretically not
  recoverable** from the transcript observed by the malicious prover up to the abort — assert
  the transcript carries no opening of any honest-derived value (structural: count openings on
  the honest values' lineage = 0 before abort).

- **T3 — validate-before-prove is load-bearing, not advisory (Lens 3).** A differential test:
  disable the witness-consistency validation (a test-only flag) and show that the SAME
  adversarial input from T1/T2 would otherwise reach an open/proof step — proving the
  validation gate is the thing standing between the input and the leak. With validation
  enabled the abort fires first. (This pins that the check is on the critical path, preventing
  a future refactor from silently moving proving ahead of validation.)

- **T4 — commitment-binding of the validated witness (Lens 1).** The validated extended
  witness MUST be the one bound into the proof's public inputs. Construct a prover that
  validates witness A but proves over witness B; **MUST** be rejected at the binding seam (the
  federated `reconstruct_public_inputs` byte-equality, the multi-source generalisation of
  single-prover audit #1). This closes the bait-and-switch between the validated and proven
  witness.

- **C — construction-provenance assertion (Lens 2/4).** A documentary+CI obligation, not a
  runtime test: the adopted collaborative-proving construction MUST be recorded with (a) the
  specific **2025/1026-patched** variant it instantiates (NOT a naive semi→malicious
  compiler), and (b) if TACEO co-snarks (or any third-party coZK stack) is used, the **pinned
  version** plus a re-run of this lens-set against that exact version, and the audit status of
  that version. A collaborative-proving soundness claim is BLOCKED until C is satisfied
  alongside T1–T4.

These obligations are **necessary, not claimed sufficient** — they encode the 2025/1026
precondition and the bind-the-validated-witness discipline; full soundness still additionally
requires the external multi-prover audit (below).

---

## 4. Overall verdict — GATES the production claim

**The overall verdict GATES any collaborative-proving / federated-attestation production claim
(milestones M-C production-claim and M-E).** Specifically:

- The collaborative path may be built, tested, and presented as **research, unaudited**
  (behind the opt-in MPC crate/feature), exactly as the build-out delta risk register already
  requires ("MUST NOT present as proving attestation to a relying party until the coZK
  re-audit (M-D) AND the external `sq-qhy4` audit pass").
- It may **NOT** be presented to a relying party as *proving* correctness or attested-source
  derivation until: **(1)** R-WV is implemented and T1–T4 + C all pass; **(2)** the
  honest-majority malicious-security MPC line (sq-km34.\* MAC-check-before-open) has landed and
  is the basis for the "free malicious security" claim (Lens 2/3); and **(3)** an **external
  cryptographer audit covers the multi-prover construction** — `sq-qhy4` audits the
  *single-prover* verifier only and does **not** discharge this. M-C's verifier-side-attestation
  *interim* (Dutta/Artemis, signature checked out-of-circuit) sidesteps the in-circuit
  collaborative-sig pitfalls but still inherits Lens 1/3 on its MPC aggregate, so it too is
  gated on the malicious-security line for any production claim.

---

## 5. Explicit uncertainty (forward-looking — what this audit CANNOT conclude)

This is an adversarial design audit of an **unbuilt** path. Stated honestly:

- **It cannot certify the path is sound.** There is no collaborative prover to forge against;
  the verdicts are about whether the *intended design* avoids documented failure modes, and
  the answer is "not yet — preconditions unfilled." A CLOSED verdict on any lens would be
  premature and is deliberately withheld.
- **The "free malicious security" applicability is unproven for sparq's specific relation.**
  2025/1026's positive result is stated for a class of constructions; whether sparq's eventual
  federated SPARQL-correctness + commitment-fold relation, under its chosen sharing scheme and
  proof system, lands inside that class is an open question for the eventual external audit.
- **The TACEO co-snarks risk is qualitative.** Absent an audit of a specific pinned version,
  this audit can only flag the dependency as unaudited; it cannot say co-snarks *is* broken,
  only that it is *unverified* against 2025/1026.
- **Performance is out of scope and has zero data points** for the collaborative path — no
  number is asserted here, per the non-canonical-timing rule.

The single most actionable, durable output is the **R-WV requirement + T1–T4/C test
obligation** (§3): it converts the 2025/1026 precondition from a documented caveat into an
enforceable build-time gate, so the gap cannot silently re-open when the path is built.

---

## Methodology

Read the prior estate (`research/mpc-zkp-build-out-delta.md` M-D framing;
`research/mpc-m4-distributed-sig-feasibility.md` §1–§4 — which already names the 2025/1026
pitfalls and the validate-before-prove guidance; `research/mpc-malicious-security-design.md`
§1 Holes 1–4 + §2.5 confidentiality-before-open; `research/zk-verifier-reaudit.md` for the
single-prover binding seam this path would plug into) and the live code
(`crates/sparq-mpc/src/proof.rs` — the deferred `CollaborativeProof`/`Attestation` traits,
all `NotYetImplemented`; `pipeline.rs` — the four-flatmates driver that assembles a real
`ProofStatement` but keeps `prove` the honest stub; `authenticated.rs` — the IT-MAC
foundation, MAC-check-before-open intent; `compare.rs`/`adversarial_tests.rs` — the
mid-pipeline open instances and existing tamper tests; `backend.rs` — the honest-majority
"free malicious security" comment). Each 2025/1026 lens was assigned CLOSED only if the design
provably avoids the pitfall; otherwise RE-OPEN with the specific unmet precondition. Verdicts
prioritise empirical honesty over reassurance: this audit states plainly that the
collaborative path is NOT sound to claim, that the validate-before-prove precondition is
unmet, and that the closest usable stack is unaudited — and encodes the precondition as an
enforceable test obligation so the gate survives into implementation.
