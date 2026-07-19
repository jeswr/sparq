<!-- [OPUS-4.8] MPC composition / UC posture design record for sparq-mpc: the standalone/UC
statement the architecture (a composition) currently lacks — justify the mid-protocol
`secure_equal` open; honest-majority UC-without-setup (Canetti FOCS'01) as the argument for the
default; carry the coZK 2025/1026 composition-security caveat on the collaborative-proof layer.
Design-for-review (no code, doc-only), Opus 4.8 (Fable unavailable) — re-review when Fable
returns. Date: 2026-07-19. Bead sq-wj4k (parent #2629 → epic sq-pwr). -->

# MPC composition / UC posture for sparq-mpc

**Status:** Deep-research design record (no implementation; **doc-only**). Author: Opus 4.8
(Fable unavailable — flag for re-review). Date: 2026-07-19. Bead **sq-wj4k**
(parent **#2629**; sibling **sq-aaop**; epic **sq-pwr** — MPC over federated SPARQL + ZKP of
correctness + attested-source derivation).

**Scope.** sparq's MPC layer is not one protocol — it is a **composition** of sub-protocols
(holder → share → join → secure-aggregate → reconstruct → proof, driven by
`pipeline.rs::run_federated`). Yet no **standalone / UC (universally-composable) statement**
records *which* security results the pieces enjoy, *which* composition theorem carries them into
the whole, and *what obligations* each open/reveal step imposes on the stages downstream of it.
This record captures that posture. It is the honest framing artifact the security-models synthesis
named as a gap ([`mpc-security-models-and-benchmarks.md`](./mpc-security-models-and-benchmarks.md)
§ "Composability / UC"; [`mpc-sparql-capability-matrix.md`](./mpc-sparql-capability-matrix.md)
§ "Composability / UC"), and item 6 of the architecture roadmap
([`mpc-zkp-federated-sparql-design.md`](./mpc-zkp-federated-sparql-design.md) §8).

**This record EXTENDS, does not duplicate:**
- [`mpc-security-models-and-benchmarks.md`](./mpc-security-models-and-benchmarks.md) — the 3-axis
  `AdversaryModel × OutputGuarantee × CorruptionThreshold` taxonomy, the fail-closed registry, and
  the leakage analysis (L2 join match-structure leak). That doc *names* the composability gap; **this**
  doc is the composition/UC treatment.
- [`mpc-cozk-reaudit.md`](./mpc-cozk-reaudit.md) — the adversarial coZK re-audit vs eprint 2025/1026.
  That doc is the *soundness verdict* on the (unbuilt) collaborative-proof path; **this** doc places its
  witness-extension pitfall inside the composition framing (why it is a *composition-security* failure,
  and which composition obligation encodes the fix).
- [`mpc-malicious-security-design.md`](./mpc-malicious-security-design.md) — the IT-MAC upgrade to
  malicious-with-abort. That doc raises the *per-operator adversary tier*; **this** doc is orthogonal —
  it is about *how the operators compose*, at whatever tier each is realized.
- [`mpc-distributed-randomness-design.md`](./mpc-distributed-randomness-design.md) — which already
  cites Canetti UC-without-setup for honest majority (FOCS'01) for the randomness beacon; **this** doc
  makes that citation load-bearing for the *whole pipeline's* default trust model.

**One-line posture (stated up front, not over-claimed):** sparq-mpc's default —
**honest-majority Shamir, semi-honest today** — is a defensible *composition* choice because
honest-majority protocols admit **UC security without any setup assumption** (Canetti, FOCS'01), so
the pieces compose without importing a CRS/PKI trust root; the one mid-protocol reveal that naive
sequential composition does **not** automatically justify (`secure_equal`'s open) is justified
**only** by modeling the reveal explicitly in the ideal functionality every downstream stage composes
against; and the collaborative-proof stage inherits a **composition-security obligation** (validate
the extended witness before proving, per coZK 2025/1026) that is **named but not yet encoded** — so
that stage stays fail-closed (`NotYetImplemented`) and no composed soundness claim may be made until
`sq-qhy4` (external audit) lands. **Nothing here is a production security claim; this is the design
posture, not a proof of security.**

---

## 0. Ground truth: the composition, verified against `origin/main`

The pipeline driver `crates/sparq-mpc/src/pipeline.rs` (`run_federated`, the four-flatmates worked
example) composes six stages, each already existing in isolation:

| # | Stage | Code | What it computes | Reveals mid-protocol? |
|---|-------|------|------------------|-----------------------|
| 1 | **Holder local eval** | `holder::Holder::evaluate_local` | each party evaluates a query fragment over its OWN graph via `sparq-engine`; raw graphs never leave | no (local only) |
| 2 | **Share** | `MpcBackend::share_private_input` (Shamir, `shamir.rs`) | secret-share each private value (e.g. salary) `t`-of-`n`, `t=⌊(n−1)/2⌋` | no |
| 3 | **Join** | `join::DisclosedKeyJoin` (clear IRIs) / `join::HiddenValueJoin` (`secure_equal`) | equi-join; hidden-value variant opens `m=(a−b)·r` per pair | **YES — the danger open (§3)** |
| 4 | **Secure aggregate** | `MpcBackend::run_secure` (`shamir.rs`, zero-round local add) | cumulative SUM over hidden shares | no (linear, local) |
| 5 | **Reconstruct** | threshold open of the DISCLOSED result only | open the final aggregate / verdict bit | yes — the *intended* output open |
| 6 | **Proof** | `proof::ProofStatement` / `CollaborativeProof` | attach a ZK/collaborative proof of correct evaluation | **stub — `NotYetImplemented` (§4)** |

Load-bearing facts (all `origin/main`): the crate is an **in-process multi-party simulation** (every
party is a function call — NO real network, NO concurrent sessions, NO broadcast, NO round counter);
the default is **honest-majority Shamir, semi-honest**; `secure_equal` opens exactly one masked field
element per pair (`join.rs`, `secure_equal_leaks_full_bipartite_match_graph` pins the leak); every
`proof.rs` collaborative method returns `MpcError::NotYetImplemented`. **The composition is real code;
its security is the design question this record frames — it is not asserted as proven.**

---

## 1. Why a standalone/UC statement is needed (and which flavor)

A protocol that is secure *in isolation* need not stay secure when its output feeds another protocol,
or when run alongside other sessions. Two composition frameworks bound this:

- **Sequential modular composition (Canetti, J. Cryptology 2000; Goldreich Vol. 2 §7).** If each
  sub-protocol securely realizes an ideal functionality `F_i` in the standalone (stand-alone-simulation)
  sense, then the composed protocol that calls them **sequentially** (one finishes before the next
  starts, no concurrency) securely realizes the composed functionality. This is the *weakest* theorem
  that covers a straight-line pipeline like `run_federated`. **Precondition that bites sparq:** every
  value a sub-protocol OPENS mid-computation must be part of `F_i`'s *defined output* — the simulator
  for stage `i` must be able to produce that opened value's distribution from `F_i`'s output alone.
  An open that is NOT in `F_i` is unsimulatable and breaks the modular-composition premise (§3).
- **Universal Composability (Canetti, FOCS'01, *Universally Composable Security*).** The strong
  framework: security is preserved under **arbitrary concurrent composition** with any environment.
  UC is what sparq would *want* the day the in-process sim becomes a real networked protocol running
  concurrently with other federation queries. The pivotal result for the default:

> **Honest-majority protocols achieve UC security WITHOUT any setup assumption** (Canetti FOCS'01;
> Canetti–Lindell–Ostrovsky–Sahai STOC'02 for the `n ≥ 3`, `t < n/2` regime). Dishonest-majority UC
> is IMPOSSIBLE in the plain model and requires setup (a CRS / PKI / correlated randomness).

**This is the composition argument FOR the honest-majority default.** Choosing HM is not only a
threshold choice — it is the choice that lets the whole pipeline aspire to UC-without-setup, i.e. to
compose *without* importing an external trust root (a CRS ceremony or PKI) into the TCB. A
dishonest-majority backend (the truthfully-refused `sq-j5ok` slot) would drag setup back in. So the
default is composition-justified, and this record is the place that says so on the record.

**Which flavor sparq claims today:** the honest, minimal claim is **sequential modular composition of
standalone-secure semi-honest sub-protocols** — because the artifact is an in-process, single-session,
straight-line sim, sequential composition is *exactly* the theorem that applies, and it is achievable
now. UC-without-setup is the **target** the HM default keeps reachable, not a claim on today's code
(no networked concurrent protocol exists to be UC-secure). Stating more would be overclaim.

---

## 2. The composition obligations, per stage

For each stage: the ideal functionality it must realize, the standalone result that applies, and the
**residual obligation** the composition imposes.

| Stage | Ideal functionality `F` | Standalone result | Composition obligation (residual) |
|-------|-------------------------|-------------------|-----------------------------------|
| 1 Holder eval | `F_local`: emit a fragment result over the party's own graph | trivial (local computation, no interaction) | outputs feeding stage 2/3 must be exactly what `F_local` defines — no side-channel from engine timing (out of model) |
| 2 Share | `F_share`: distribute a `t`-of-`n` sharing | Shamir is perfectly private for any `t` shares (Shamir'79); standalone-secure semi-honest | randomness must be a CSPRNG (`rng.rs`, `SecureRng::from_os`) — a deterministic PRNG breaks privacy; `insecure-test-rng` off by default |
| 3 Join | `F_join`: output the join, **leaking the match bit per pair** (L2) | semi-honest secure IF the opened `m` is simulatable from the match bit (§3) | **the leak MUST be in `F_join`** — the downstream aggregate composes against a functionality that *already leaked the match structure*; a `LeakageProfile` should surface it |
| 4 Aggregate | `F_sum`: threshold sum, open nothing | linear ops are local & perfectly private (no interaction) | none new (zero-round); but any *chained* multiplication (degree reduction) adds an open → re-enters §3's obligation |
| 5 Reconstruct | `F_open`: open the DISCLOSED result only | threshold open is the intended output | the reconstructed value must be a *function of the ideal outputs*, not of intermediate shares |
| 6 Proof | `F_prove`: prove correct evaluation, leak nothing about honest witnesses | **UNBUILT + coZK-gated (§4)** | **validate the extended witness before proving** (2025/1026) — the unfilled composition obligation |

The pattern: **every mid-protocol open transfers a leakage obligation to the ideal functionality that
the next stage composes against.** Composition is sound iff each downstream stage is proven secure
*relative to a functionality that already accounts for the upstream leak*. sparq's job is to keep those
functionalities honest — the danger is a stage silently composing against a leak-free `F` that the real
upstream protocol does not deliver.

---

## 3. Justifying the `secure_equal` mid-protocol open

The `HiddenValueJoin::secure_equal` operator (`join.rs`) is the one built operator that **opens a value
mid-computation**: to test `a == b` it computes `d = a − b`, draws a uniform nonzero mask `r` (one
Shamir multiplication), and **opens `m = d·r`**. Then `m == 0 ⇔ a == b`.

**Why naive sequential composition does not *automatically* justify it.** Modular composition requires
the opened `m` be simulatable from `F_join`'s output. If `F_join` were defined to output *only the join
result* (leaking nothing else), the simulator would have to produce `m` from nothing — impossible when
`m` is a real field element the environment observes. So a leak-free `F_join` is the **wrong**
functionality; composing against it is unsound.

**The justification (and its exact precondition).** Model `F_join` to leak **exactly the match bit**
per pair. Then:
- when `a ≠ b`: `d ≠ 0`, and `r` uniform nonzero ⇒ `m = d·r` is **uniform over the nonzero field**,
  independent of `d` — so a simulator holding the bit "unequal" samples a uniform nonzero element,
  a perfect simulation;
- when `a = b`: `d = 0` ⇒ `m = 0` deterministically — the simulator holding "equal" outputs `0`.

Hence the open reveals **only the match bit**, and `secure_equal` **standalone-realizes** the
bit-leaking `F_join` (semi-honest). The mid-protocol open is *justified* — **conditional on the
composition using the bit-leaking functionality downstream.** Two obligations fall out, both already
partly encoded:

1. **The leak must be carried, not hidden.** `secure_equal_leaks_full_bipartite_match_graph`
   (`join.rs`) pins that `join` opens the WHOLE bipartite match graph over `O(|L|·|R|)` pairs — i.e.
   the join-key fan-out/multiplicity (leak L2). Any downstream stage (aggregate, proof) must compose
   against a functionality that *already* leaked this. The bit-vector variant `secure_equal_to_bit`
   (`compare.rs`) keeps the per-pair bit **secret-shared, never opened** — that is the composition-safe
   primitive to prefer when the bit itself must not surface (e.g. inside a larger circuit).
2. **The open must be consistency-checked against a malicious opener.** `m = d·r` is a degree-`2t`
   Reed–Solomon codeword over `n` party points; at the minimal honest-majority `n = 2t+1` it has **zero
   RS redundancy** (`shamir.rs reconstruct_degree`, `sq-7q9i`/WI-2 consistency-checked open), so a
   forged product share can silently flip a verdict and is information-theoretically undetectable at
   minimal `n`. Under semi-honest this is out of model; the **composition-with-malicious-security**
   story needs the IT-MAC upgrade ([`mpc-malicious-security-design.md`](./mpc-malicious-security-design.md),
   §8 step 5) so the open MAC-checks before revealing — the SPDZ discipline of *check-then-open* is
   itself a composition-safety mechanism.

**Bottom line for §3:** the open is composition-justified **as scoped** (semi-honest, bit-leaking
`F_join`, checked open), and this is honest to state; it is **not** justified against a malicious opener
at minimal `n` without the authenticated-open upgrade. Operators that open NO value mid-chain (e.g. the
bounded-path design's no-mid-chain-open property,
[`mpc-bounded-property-path-design.md`](./mpc-bounded-property-path-design.md)) sidestep this obligation
entirely and are the composition-cleanest building blocks.

---

## 4. The collaborative-proof layer: a composition-security failure the UC lens catches

The proof stage (stage 6) is where composition security bites hardest, and it is why this record and
the coZK re-audit are twinned. Per **eprint 2025/1026** (Garg–Goel–Jain–Roberts–Sekar, CRYPTO'25,
*Malicious Security in Collaborative zk-SNARKs*), a collaborative prover that proves on an
**inconsistent / maliciously-extended witness** can **LEAK the honest parties' inputs**. In the
composition framing this is precisely a **composition-security failure**: the proof sub-protocol, when
composed after the MPC evaluation, does NOT realize a leak-free `F_prove` — a corrupted prover uses the
proving step as a channel to exfiltrate honest witness data. A stand-alone "the SNARK is
zero-knowledge" claim is exactly the kind of local reasoning a UC/composition treatment **refutes**: ZK
of the proof object does not imply zero leakage *of the composed protocol* when the witness can be
adversarially extended between the MPC output and the prover input.

**The composition obligation this imposes:** the extended witness must be **validated (consistency-checked)
before proving** — the "fix" 2025/1026 names. sparq has **named but not encoded** this obligation
([`mpc-cozk-reaudit.md`](./mpc-cozk-reaudit.md) §3, RE-OPEN), and every collaborative-proof method is a
fail-closed `MpcError::NotYetImplemented` stub. That fail-closed state is the **correct** composition
posture today: since the validate-before-prove obligation is not enforceable, the only honest move is to
NOT compose the proof stage at all. The `secure_equal` mid-pipeline open (§3) is the *live in-crate
instance* of the same family — a mid-computation reveal whose downstream composition must account for
what it exposed (coZK re-audit Lens 1).

**No production/attestation claim over the composed proof path may be made** until (a) the path is
built, (b) validate-before-prove is encoded and enforced, and (c) the **external** cryptographer audit
**`sq-qhy4`** covers the *multi-prover* construction (not only the single-prover verifier). This is the
LIVE privacy-claims gate; any ZK/MPC soundness statement stays research-grade / not-externally-audited
(`sq-qhy4`), MPC semi-honest-only.

---

## 5. Which results apply — the summary map

| Question | Result / posture | Applies to sparq how |
|----------|------------------|----------------------|
| Does the straight-line pipeline compose? | **Sequential modular composition** (Canetti J.Crypto'00; Goldreich) | YES for the in-process single-session sim, **iff** each open is in its stage's `F` (§2, §3) |
| Why is honest-majority the default? | **UC WITHOUT setup for honest majority** (Canetti FOCS'01; CLOS STOC'02) | the composition argument FOR HM — composes without a CRS/PKI trust root; DM would need setup (`sq-j5ok` refused) |
| Is the `secure_equal` open OK? | **Yes as scoped** — opens only the match bit; standalone-realizes the bit-leaking `F_join` (§3) | conditional on carrying leak L2 downstream + a checked open; NOT malicious-safe at minimal `n` without IT-MACs |
| Concurrent / networked composition? | **UC (FOCS'01)** is the target framework | ASPIRATIONAL — no networked concurrent protocol exists yet; do not claim UC on today's sim |
| Collaborative proof composition? | **coZK 2025/1026** — validate extended witness before proving | OBLIGATION named, NOT encoded; stage stays `NotYetImplemented`; gated by `sq-qhy4` |
| Malicious-with-abort composition | check-then-open (SPDZ/IT-MAC) is a composition-safety mechanism | the [`mpc-malicious-security-design.md`](./mpc-malicious-security-design.md) upgrade closes the §3 checked-open hole |

---

## 6. Non-goals, honesty caveats, and follow-ups

- **This is a posture record, not a proof.** No formal UC simulator is constructed here; the record
  states *which* theorems apply and *what obligations* each imposes. A machine-checkable or
  paper-grade composition proof is a heavier, separate, audit-gated deliverable — and any UC/composition
  soundness statement inherits the `sq-qhy4` external-audit gate.
- **In-process sim ≠ networked protocol.** Everything about concurrent/UC composition is aspirational
  until a real networked, multi-session protocol exists. Claiming UC on the current single-process sim
  would be overclaim.
- **Semi-honest default.** The §3 justification is semi-honest; the malicious-opener hole at minimal
  `n = 2t+1` is real and is closed only by the IT-MAC upgrade, not by this record.
- **Sibling bead `sq-aaop`.** Listed with `sq-wj4k` in the roadmap as "the composition / UC posture
  design records." This record covers the composition-obligations + which-results-apply framing for the
  MPC pipeline; if `sq-aaop` is retained as a distinct slice it should cover the *formal simulator
  sketch / paper-grade write-up* rather than re-state this posture (avoid duplication — one source of
  truth).
- **Surface a `LeakageProfile`.** §2/§3's "carry the leak downstream" obligation wants a machine-readable
  per-operator leakage descriptor so a federation can reason about residual composed leakage; that is an
  implementation follow-up, tracked separately (out of scope for this doc-only record).

---

## References

Canetti, *Security and Composition of Multiparty Cryptographic Protocols*, J. Cryptology 2000 (sequential
modular composition). Canetti, *Universally Composable Security: A New Paradigm for Cryptographic
Protocols*, FOCS'01 (UC; honest-majority UC-without-setup). Canetti–Lindell–Ostrovsky–Sahai, *Universally
Composable Two-Party and Multi-Party Secure Computation*, STOC'02. Goldreich, *Foundations of Cryptography
Vol. 2*, §7 (modular composition). Shamir, *How to Share a Secret*, CACM 1979. Garg–Goel–Jain–Roberts–Sekar,
*Malicious Security in Collaborative zk-SNARKs: More than Meets the Eye*, eprint 2025/1026 (CRYPTO'25).
In-crate: `crates/sparq-mpc/src/{pipeline,join,compare,shamir,proof,rng}.rs`; companions
[`mpc-cozk-reaudit.md`](./mpc-cozk-reaudit.md), [`mpc-security-models-and-benchmarks.md`](./mpc-security-models-and-benchmarks.md),
[`mpc-sparql-capability-matrix.md`](./mpc-sparql-capability-matrix.md),
[`mpc-malicious-security-design.md`](./mpc-malicious-security-design.md),
[`mpc-distributed-randomness-design.md`](./mpc-distributed-randomness-design.md),
[`mpc-zkp-federated-sparql-design.md`](./mpc-zkp-federated-sparql-design.md). Gates: `sq-qhy4` (external
audit), collaborative proof `NotYetImplemented` fail-closed.
