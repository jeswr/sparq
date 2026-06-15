<!-- [OPUS-4.8] Design-for-review authored by Opus 4.8 (Fable 5 unavailable) — re-review when Fable returns. -->
# Issuer-attested, credential-bound Holder Proof-of-Possession (HolderPoP)

Design record for bead **sq-c2ql** (`area:sparq-zk-compose`, `zk`): close the
**trusted-holder-A-presents-B's-credential** gap that `sq-cwq` deliberately left
open. Scope is the credential↔holder *binding* — i.e. proving the presenter is the
subject the issuer bound the credential to — **without the issuer being online at
presentation time** ("issuer-attested"). NO production code here; this is a
design-for-review record matching the rigour of `research/zk-soundness-audit.md`
and `research/zkp-query-proofs-plan.md`.

Parent context: `sq-gbp4` NEW-2a (re-audit follow-up), epic `sq-1s2`
("ZK query-proof build-out + in-circuit privacy upgrades"). Siblings this design
is sized to fit: `sq-bwwl` (in-circuit hidden cross-credential JOIN),
`sq-kndw` (fully-hidden revocation), `sq-z9l` (in-circuit issuer-signature
gadget — the **direct reuse precedent**, already landed).

---

## 0. Where the trusted-holder assumption lives today (cite file:line)

`sq-cwq` ("ZK: implement HolderPoP proof-of-possession or gate it unimplemented")
shipped a **challenge-bound Schnorr HolderPoP** that proves the presenter
possesses a holder key the relying party *already trusts*. The assumption this
design must remove is stated, in the code, in three places:

1. **`crates/sparq-zk-compose/src/verifier.rs:2965-2975`** — the
   `bind_holder_pop` "honest deferral" doc block, verbatim:

   > *"It does NOT bind that key to the SPECIFIC credential the scan/filter
   > sub-proofs attest … Without it, a trusted holder A could present a trusted
   > holder B's credential. The holder registry narrows 'who may present at all'
   > to authorised holders, which is the meaningful interim guarantee; the
   > per-credential binding is the documented next step."*

   The function (`crates/sparq-zk-compose/src/verifier.rs:2977-3018`) checks
   exactly four things: registry non-empty, `holder ∈ HolderRegistry`, known
   cryptosuite + parseable bytes, and `sig_verify(holder_pk, holder_pop_message(challenge), pop)`.
   **None of the four reads any commitment, attestation, or issuer signature.**
   The PoP message is `holder_pop_message(challenge) = Poseidon2([ZKSIG_HP, challenge])`
   (`crates/sparq-zk/src/sig.rs:532-535`) — it binds the *nonce*, not the *credential*.

2. **`crates/sparq-zk-compose/src/manifest.rs:102-108`** — the `BindingMode::HolderPop`
   "Scope (honest deferral)" doc: *"It does NOT yet bind that key to a SPECIFIC
   credential — an issuer-attested holder binding (the issuer signing the holder
   key into the credential) is deferred."* The `HolderPop` variant
   (`manifest.rs:111-124`) carries only `{ challenge, holder, pop, cryptosuite }`.
   There is **no field linking `holder` to any `CommitmentAttestation`.**

3. **`crates/sparq-zk-compose/src/verifier.rs:264-290`** — `HolderRegistry`'s doc:
   *"Membership here means 'this holder key is authorised to present' — it does
   NOT bind the key to a SPECIFIC credential."* The registry is the *interim*
   trust anchor; it is a coarse allow-list, not a per-credential subject binding.

**The gap, precisely.** The issuer signs `commitment_message_with_status(C(G), salt, status_ref)`
(`crates/sparq-zk/src/sig.rs:505-512`); the signed object contains the graph
commitment, the salt, and the status reference — **but not the holder's key**.
So there is no cryptographic fact tying *this credential* to *that holder secret*.
A holder A who is a member of the relying party's `HolderRegistry` can take holder
B's manifest+proof (B's issuer-signed `C(G)`, B's scan/filter sub-proofs), replace
the `binding` with `HolderPop { holder: A_pk, pop: sign(A_sk, challenge) }`, and
pass `bind_holder_pop`: A is in the registry, A's PoP over the challenge verifies,
and nothing checks that the issuer ever bound A (or B) to `C(G)`. **A presents B's
credential.** This is the trusted-holder gap.

> Note the audit-#3/#9/#12 issuer-signature gates *are* sound and *are* checked
> (`bind_issuer_attestations`, `crates/sparq-zk-compose/src/verifier.rs:1700`),
> so A genuinely cannot forge `C(G)` or the issuer signature. The gap is narrowly
> the **subject binding**: who the credential was issued *to*. Closing it converts
> the holder check from "is the presenter on my allow-list" to "is the presenter
> the subject *the issuer* named in *this* credential".

---

## 1. Threat model

### 1.1 Setting and parties

Three parties, exactly as `research/zkp-query-proofs-plan.md` §2.5 frames them:

- **Issuer** — signs credentials offline; trusted to bind a subject. Present at
  *issuance*, **offline at presentation** (the issuer-attested requirement).
- **Holder (subject)** — the legitimate subject the issuer bound. Holds a
  long-lived **holder secret** `hsk` with public key `hpk = hsk·G`. Proves a
  SPARQL query over its credential store + presents.
- **Verifier (relying party)** — anchors trusted issuer keys (`KeySet K`) and
  issues a fresh `challenge` per presentation. Runs `verify_manifest`.

### 1.2 Adversary

A **malicious holder A** who:

- legitimately holds its own `(hsk_A, hpk_A)` and (today) may be in the relying
  party's `HolderRegistry`;
- has obtained a complete, valid presentation for **a different subject B** —
  B's issuer-signed `C(G)`, B's bb scan/filter sub-proofs, B's manifest — by any
  means (B's device was seized/leaked, B colluded once, the proof was captured on
  the wire, B's derived credential was re-shared);
- controls its own side fully (the standard ZK-prover threat model: it writes the
  manifest JSON, chooses the binding, and runs the prover).

A's **goal**: get `verify_manifest` to return `Ok(())` for a presentation that the
relying party attributes to **A as the subject of B's credential** — impersonating
B, or laundering B's credential as A's own.

The issuer is honest; the verifier is honest and enforces its policy fail-closed;
the in-circuit relations and the audit-#1/#2 cryptographic gate (public-input
reconstruction + canonical-vk pinning) and audit-#3/#9/#12 issuer gates are in
force (post-`sq-1s2` remediation). The only residual hole is the subject binding.

### 1.3 What "issuer-attested, holder-bound" MUST guarantee

The verifier, with the issuer **offline**, gains assurance that:

- **(G1) Subject binding.** The presenter possesses a holder secret `hsk` whose
  public key `hpk` was **bound by the issuer into the very credential the
  scan/filter sub-proofs attest** — not merely a key on an allow-list, and not a
  key bound to *some other* credential. Concretely: the issuer signature that
  covers `C(G)` (the audit-#3 attestation) also covers `hpk`, and the presenter
  proves knowledge of `hsk`.
- **(G2) Freshness / replay resistance.** The PoP is bound to the verifier's
  fresh `challenge`, so a captured presentation cannot be replayed by a party that
  does not hold `hsk` (this is already delivered by `sq-cwq`; the design must
  *preserve* it, composed with G1).
- **(G3) Offline issuer.** No issuer interaction at presentation. The issuer's
  one-time signature over `(C(G), …, hpk)` at issuance is the only issuer act; the
  verifier checks it (or a PoK of it) against its own trusted `KeySet K`.

### 1.4 What it explicitly does NOT guarantee (honest scope)

Stated plainly, because over-claiming here is exactly the failure mode the
soundness audit punishes:

- **It does not stop a holder sharing its OWN secret.** If the legitimate subject
  B *voluntarily hands `hsk_B` to A*, A becomes computationally indistinguishable
  from B and can present. HolderPoP binds the credential to a *key*, and possession
  of the key *is* the subject; key-sharing is a key-management/incentive problem
  (mitigations: non-extractable hardware keys, all-or-nothing non-transferability
  by binding `hsk` to high-value secrets, accountability — all **out of scope** for
  a circuit-level PoP). The guarantee is **impersonation of a *different* subject
  who did *not* share their key**, not non-transferability of one's own.
- **It does not prove the holder is a *human* / a particular real-world identity.**
  It proves possession of the issuer-bound key. Linking that key to a legal person
  is the issuer's KYC at issuance, outside the proof.
- **It does not, by itself, hide WHICH holder** (the clear-`hpk` variant is a
  linkability channel). A hidden-holder upgrade is a separate privacy step
  (§2.4 fallback / §6 plan), mirroring the clear-key → hidden-key issuer upgrade
  (`sq-z9l`).
- **It does not retroactively fix issuance.** If the issuer signed a credential
  with **no** `hpk` (a bearer credential), there is nothing to bind to; such
  credentials remain bearer (and the verifier must, fail-closed, refuse to treat
  a `holder-pop`-mode presentation as holder-bound over a credential whose
  attestation carries no `holderBinding`).

---

## 2. Cryptographic construction

We survey the standard approaches against the **constraints that actually bind
this repo**: BN254 is the proving field; the in-tree signature primitive is
**Schnorr over Baby-JubJub with a Poseidon2 challenge** (`crates/sparq-zk/src/sig.rs`),
and there is already a **bit-for-bit in-circuit verifier for it** (`schnorr_verify`,
`zk/compose/compose_core/src/issuer.nr:187`) plus a **Poseidon2-Merkle set-membership
gadget** (`key_set_membership`, `issuer.nr:221`). Pairings (BBS+) are *not* in-tree
and are expensive on BN254/Grumpkin (verifiable-credentials-zk skill).

For every approach: **what the issuer commits to at issuance**, **what the holder
proves at presentation**, **how the verifier checks (issuer offline)**, and the
**trust assumptions**.

### 2.A Link-secret committed at issuance (Idemix / Anoncreds style)

- **Issuance:** holder draws a secret *link secret* `ls`; the issuer signs a
  credential over (attributes ∥ a commitment `Cm = Commit(ls; r)` to `ls`). The
  issuer never learns `ls`.
- **Presentation:** holder proves, in ZK, knowledge of `ls`, `r` opening `Cm`,
  *and* that the same `ls` opens the commitment inside the issuer-signed object.
- **Verifier:** checks the issuer signature (or PoK of it) + the opening relation.
- **Trust:** binding rests on the hiding/binding of `Cm` and signature
  unforgeability; the *same* `ls` can be re-used to bind several credentials
  together (the Anoncreds multi-credential link).
- **Fit/cost:** clean, but the "same `ls` across credentials" linking is a feature
  this design does **not** need (cross-credential join is `sq-bwwl`'s separate
  concern, and Anoncreds-style linking is a *privacy* property, not a subject
  binding). It also requires a commitment-opening gadget *in addition* to a key
  PoK. **More machinery than needed for G1.**

### 2.B Key-bound credential + presentation-time PoK of the bound key (`cnf`/holder-binding) — RECOMMENDED

This is the construction the plan already names (`zkp-query-proofs-plan.md`
§2.5: *"the credential carries the holder's public key in the VC `cnf`
(confirmation) claim, and the circuit proves knowledge of the matching secret …
a Schnorr-style PoK over the embedded curve, the same cost class as one
issuer-signature check"*). It is the SD-JWT-VC / OpenID4VP key-binding model
(KB-JWT `cnf`) and the W3C VC `cnf`/`confirmationMethod` model, instantiated on
this repo's native curve.

- **Issuance (issuer commits to `hpk`).** The issuer folds the holder's public key
  `hpk` into the **same signed object that already binds `C(G)`**. Concretely,
  extend the audit-#12 message family with a holder-bound variant:

  ```
  commitment_message_with_holder(C(G), salt, status_ref, holder_pk_digest)
      = Poseidon2([ZKSIG_C4, C(G), salt, status_ref, holder_pk_digest])
  holder_pk_digest = Poseidon2([ZKSIG_HK, hpk.x, hpk.y])   // domain-separated key digest
  ```

  (a **new** domain tag `ZKSIG_C4`, distinct from `ZKSIG_C1/C2/C3` at
  `crates/sparq-zk/src/sig.rs:57,283,504`, so a holder-bound attestation can never
  be cross-substituted for a non-holder-bound one). The issuer signs this with its
  Schnorr key; the signature is verified against the relying party's `KeySet K`,
  exactly like every other attestation. **The issuer is online only here, once.**

- **Presentation (holder proves knowledge of `hsk` matching the bound `hpk`).**
  Two faithful instantiations, in increasing strength/cost:

  - **B1 — verifier-side PoP + issuer-attested `hpk` (minimal, clear-key).** The
    manifest's `HolderPop` carries the *issuer-attested* `holder_pk` (now bound
    into the issuer signature, not a free allow-list entry) and a Schnorr PoP over
    `holder_pop_message(challenge)`. The verifier (i) recomputes
    `commitment_message_with_holder(C(G), salt, status_ref, Poseidon2([ZKSIG_HK, hpk]))`
    and checks the **issuer** signature over it (so `hpk` is issuer-attested *for
    this credential*), then (ii) checks the **holder** PoP over the fresh
    challenge under that same `hpk`. G1 now holds: the key A must possess is the
    one the issuer bound into B's credential — A cannot substitute its own key
    without invalidating the issuer signature. **No new circuit; reuses the
    existing host-side Schnorr `sig_verify` and the audit-#12 attestation
    plumbing.** Cost ≈ one extra issuer-signature recompute + the existing PoP
    check. *Limitation:* `hpk` is disclosed (linkability) — acceptable as the
    clear-key tier (mirrors clear-key issuer attestations), upgraded by B2.

  - **B2 — in-circuit PoK of `hsk` bound to the proof (hidden-key, strong).** A
    new circuit member proves, in zero knowledge over the verifier's `challenge`:
    *"I know `hsk` such that `hpk = hsk·G`, and `Poseidon2([ZKSIG_HK, hpk.x, hpk.y])`
    equals the PUBLIC `holder_pk_digest` that the verifier folds into the
    issuer-signed message."* This is **one Baby-JubJub scalar-mul + a Poseidon2
    digest** — strictly *cheaper* than `schnorr_verify` (which does *two*
    scalar-muls), and it reuses `scalar_mul`/`point_add`/`assert_on_curve`/
    `assert_lt_l` from `zk/compose/compose_core/src/issuer.nr:97-203` verbatim.
    The holder key never leaves the circuit if `holder_pk_digest` is the only
    public artefact (hidden-holder); or `hpk` is public for the clear tier. G2 is
    preserved by binding the same `challenge` as public-input field 0 (the family
    convention). This is the recommended *strong* tier and the natural home of the
    binding.

- **Trust assumptions:** Schnorr-over-Baby-JubJub EUF-CMA (already assumed for
  issuer + holder signatures), Poseidon2 collision/hiding (already assumed for
  commitments and the audit-#12 digests), discrete-log hardness on Baby-JubJub
  for the PoK soundness. **No new assumption beyond the ones the estate already
  relies on.**

### 2.C Signature-PoK / BBS+-style holder binding (fallback)

- The issuer issues a BBS+ credential whose messages include `hpk` (or a
  Pedersen commitment to `ls`); at presentation the holder proves knowledge of a
  BBS+ signature over the (selectively disclosed) messages including the key.
- **Strength:** native selective disclosure + unlinkable presentations.
- **Cost/fit:** pairings are not in-tree and are heavy on BN254 (skill table);
  adopting BBS+ is a *signature-scheme* migration of the whole estate, not a
  HolderPoP increment. **Rejected as primary** for this bead; recorded as the
  fallback if/when the estate moves to BBS+ for selective disclosure (the skill's
  "BBS+ becomes attractive once the pipeline is demonstrated end-to-end").

### 2.D Recommendation

**Adopt 2.B.** Ship **B1 (verifier-side, issuer-attested clear-key holder
binding)** first — it closes G1 with *no new circuit* by extending the audit-#12
signed-message family — then **B2 (in-circuit PoK, hidden-key)** as the privacy
tier, mirroring the clear-key→hidden-key issuer trajectory (`bind_issuer_attestations`
→ `bind_hidden_issuer_attestations`/`sq-z9l`). Justification: B2's in-circuit
relation is *cheaper than the already-landed `schnorr_verify`* (one scalar-mul vs
two) and reuses its gadgets verbatim; the curve, hash, domain-separation
discipline, and Merkle membership are all in place; and the construction is the
exact one the plan (§2.5) and the SD-JWT-VC/`cnf` ecosystem already specify.
2.A and 2.C add machinery (commitment-opening; pairings) for properties this bead
does not require.

---

## 3. Fit to sparq

### 3.1 Issuance/attestation layer (`crates/sparq-zk/src/sig.rs`)

Add, alongside the existing message builders (`commitment_message` :259,
`commitment_message_with_salt` :284, `commitment_message_with_status` :505):

```rust
const SIG_DOMAIN_HOLDER_KEY:        u64 = /* "ZKSIG_HK" */;
const SIG_DOMAIN_COMMITMENT_HOLDER: u64 = /* "ZKSIG_C4" */;  // distinct from C1/C2/C3

pub fn holder_key_digest(hpk: &PublicKey) -> Fr;            // Poseidon2([ZKSIG_HK, x, y])
pub fn commitment_message_with_holder(                      // issuer-signed, holder-bound
    commitment: &Fr, salt: &Fr, status_ref: &Fr, holder_pk_digest: &Fr) -> Fr;
```

This mirrors the **exact** pattern `commitment_message_with_status` used to close
audit #12 (`sig.rs:486-512`): a new domain tag, fold one more field into the same
Schnorr-signed object, single source of truth for issuer/verifier/circuit. The
existing `holder_pop_message` (:532) and `sign_holder_pop` (:544) stay as-is for
the freshness PoP (G2).

### 3.2 Manifest layer (`crates/sparq-zk-compose/src/manifest.rs`)

- Extend `CommitmentAttestation` (`manifest.rs:158-200`) with an **optional**
  `holder: Option<AttestedHolderBinding>` whose presence means the issuer signed
  the holder-bound message variant (`commitment_message_with_holder`), carrying
  `holder_pk_digest: FieldHex` (and, for the clear tier, the `hpk` hex). This is
  the strict analogue of the `status: Option<AttestedStatusRef>` field
  (`manifest.rs:184-199`, :214-234) that closed audit #12 — exactly one signed-message
  shape per attestation, cross-checked by the verifier.
- Extend `BindingMode::HolderPop` (`manifest.rs:111-124`) so the disclosed
  `holder` key is the one cross-checked against the attestation's
  `holder_pk_digest` (rather than only against the registry). The B2 tier adds a
  `holder_binding` sub-proof entry to `sub_proofs` (a new `ProofInputs::HolderPok`)
  carrying the public `holder_pk_digest` and `challenge`.

### 3.3 Verifier layer (`crates/sparq-zk-compose/src/verifier.rs`)

- **B1:** upgrade `bind_holder_pop` (`verifier.rs:2977-3018`) so it, **in addition
  to** the existing registry+PoP checks, requires that the credential's
  `CommitmentAttestation` carries a `holder` binding whose `holder_pk_digest`
  equals `holder_key_digest(disclosed hpk)`, **and** that the issuer signature on
  that attestation verified over `commitment_message_with_holder(...)` (this last
  check folds into `bind_issuer_attestations`, `verifier.rs:1763`, which already
  selects the signed-message variant from which optional fields are present —
  e.g. the salt/status selection at :1733-1745, :96 of the audit doc). New
  `CheckError` variants: `HolderBindingMissing` (a `holder-pop` presentation over
  an attestation with no `holder` binding — fail-closed, no silent bearer
  fallback) and `HolderKeyMismatch` (disclosed `hpk` ≠ issuer-attested digest).
- **B2:** add `bind_holder_pok`, the analogue of `bind_hidden_issuer_attestations`
  (`verifier.rs:2528`): bind the public `holder_pk_digest` to the attestation, bind
  the verifier's fresh `challenge` (the audit-#4 nonce, already reconstructed into
  public-input field 0), select the canonical vk for the `holder_pok` member by
  re-derived `CircuitId` (audit-#2 discipline), reconstruct the public-input
  vector and byte-equal it (audit-#1 discipline), and `bb verify`. The hidden-key
  digest is the only public holder artefact (no clear `hpk`).
- The whole thing sits **inside the existing four-stage `verify_manifest`** between
  the issuer-attestation gate and the final accept, gated by a relying-party policy
  object (a `HolderBindingPolicy`, mirroring `RevocationPolicy`/`EntailmentPolicy`)
  so a deployment that does not use holder binding stays on the `Challenge` path
  fail-closed.

### 3.4 Circuit layer (`zk/compose/compose_core/src/`)

- **New module `holder.nr`** (sibling of `issuer.nr`). The relation:

  ```
  pub fn holder_pok(challenge: Field, holder_pk_digest: Field,   // public
                    hsk: Field, hpk: Point, e_k_unused: ()) {    // private (hsk, hpk)
      assert_on_curve(hpk);
      assert(!((hpk.x == 0) & (hpk.y == 1)), "identity holder key rejected");
      assert_lt_l(hsk);
      let g = Point { x: BJJ_GX, y: BJJ_GY };
      let derived = scalar_mul(hsk, g);                          // hpk == hsk·G
      assert((derived.x == hpk.x) & (derived.y == hpk.y), "holder key mismatch");
      assert(h2(hpk.x, hpk.y) == holder_pk_digest, "holder digest mismatch");
      let _ = challenge;                                          // bound by public inputs
  }
  ```

  Every callee — `scalar_mul`, `point_add`, `assert_on_curve`, `assert_lt_l`,
  `h2`, the `BJJ_*` constants — is **reused verbatim** from
  `zk/compose/compose_core/src/issuer.nr:62-203`. New circuit-family member
  `holder_pok` (a single depth-free member; no Merkle parameterisation needed for
  the clear-digest tier). A future hidden-holder-**set** tier (analogue of
  `key_set_membership`) would add a depth `D` exactly as `hidden_issuer_d{D}`
  does.

- **New main** `zk/compose/holder_pok/src/main.nr`, modelled on
  `zk/compose/hidden_issuer_d4/src/main.nr`: public `challenge`,
  `holder_pk_digest`; private `hsk`, `hpk_x`, `hpk_y`. Parameter order is the
  public-input layout the verifier reconstructs (audit-#1).

### 3.5 What is reused vs new (summary)

| Piece | Reused | New |
| --- | --- | --- |
| Schnorr/Baby-JubJub curve + Poseidon2 challenge | ✅ `sig.rs`, `issuer.nr` | — |
| In-circuit scalar-mul / point-add / on-curve / `< L` | ✅ `issuer.nr:97-203` | — |
| Poseidon2 digest `h2` | ✅ `hashes.nr` | — |
| Freshness PoP (G2) | ✅ `holder_pop_message`/`sign_holder_pop` | — |
| Issuer attestation plumbing + key-set `K` | ✅ `bind_issuer_attestations` | — |
| Public-input reconstruction + canonical-vk gate | ✅ audit-#1/#2 path | — |
| Holder-bound signed message | — | `commitment_message_with_holder`, `holder_key_digest` |
| Attestation/binding manifest fields | — | `AttestedHolderBinding`, `HolderPok` inputs |
| Verifier gates | — | `HolderBindingMissing`/`HolderKeyMismatch`, `bind_holder_pok`, `HolderBindingPolicy` |
| In-circuit holder PoK | — | `holder.nr` + `holder_pok` member |

---

## 4. Soundness argument

Against the §1.2 adversary (malicious holder A holding B's full presentation), and
tied to the discipline in `research/zk-soundness-audit.md`.

### 4.1 Why A cannot forge a PoP for B's credential (G1)

- A's only path to acceptance is to present a credential whose
  `CommitmentAttestation` carries a `holder` binding (else `HolderBindingMissing`,
  fail-closed — no bearer fallback). For B's credential, that binding's
  `holder_pk_digest = holder_key_digest(hpk_B)`, and it is **inside the issuer's
  Schnorr signature** (`commitment_message_with_holder`), which A cannot alter
  without invalidating it (EUF-CMA; the issuer key is in the verifier's `K`, never
  prover-supplied — the audit-#3 anchor).
- **B1:** the verifier requires the disclosed `hpk` to satisfy
  `holder_key_digest(hpk) == holder_pk_digest`, i.e. `hpk = hpk_B` (Poseidon2
  collision resistance), and then requires a valid PoP under `hpk_B` over the
  fresh challenge. A does not hold `hsk_B`, so A cannot produce that PoP
  (Schnorr EUF-CMA). If A instead substitutes its own `hpk_A`, the digest equality
  fails (`HolderKeyMismatch`). Either way: **reject**.
- **B2:** the circuit constrains `hpk = hsk·G` for a *witnessed* `hsk` and
  `h2(hpk) = holder_pk_digest` for the **public** digest the verifier bound from
  B's issuer-attested binding. Soundness of the proof system + DL-hardness on
  Baby-JubJub ⇒ a valid proof exists only if the prover knows `hsk` with
  `hsk·G = hpk_B`. A does not, so A cannot produce the proof. The identity-key
  guard (`hpk ≠ (0,1)`) and `assert_lt_l(hsk)` rule out the degenerate/non-canonical
  forgeries that `schnorr_verify` already rules out (`issuer.nr:187-203`).

The binding therefore upgrades the holder check from *"is the presenter on the
allow-list"* to *"is the presenter the subject the issuer bound into THIS
credential"* — exactly G1.

### 4.2 Replay resistance (G2) — preserved under composition

The PoP (B1) and PoK (B2) are both over the verifier's fresh `challenge`, which is
**reconstructed into public-input field 0 and byte-equalled** by the audit-#1
gate and enforced single-use by the audit-#4 nonce store
(`verify_manifest`). A captured B-presentation re-presented to a new verifier
fails the new nonce. The PoP/PoK are *additionally* bound to the issuer-attested
`holder_pk_digest`, so even a same-nonce replay within a window is rejected unless
the presenter holds `hsk_B`. G1 and G2 compose (the plan's *"a holder-pop derived
credential is still challenge-bound at each presentation"*, §2.5).

### 4.3 New soundness obligations on the verifier (tie-back to the audit discipline)

The audit's load-bearing lesson is: **every prover-supplied JSON field must be
reconstructed into / byte-equalled against the bb public inputs, and every trust
anchor must be the verifier's, never the prover's.** This design inherits and
extends those obligations:

1. **Anchor `holder_pk_digest` in the issuer signature, never the manifest alone.**
   The digest the verifier binds (B1 cross-check / B2 public input) MUST be the one
   recovered from the issuer-attested `commitment_message_with_holder`, exactly as
   audit-#12 anchors the status reference. A digest read only from a prover JSON
   field is the audit-#1 hole reopened.
2. **Fold `holder_pk_digest` and `challenge` into the reconstructed public-input
   vector for the `holder_pok` member and byte-equal them (audit-#1); select the
   `holder_pok` vk by re-derived `CircuitId` from the canonical compiled member
   (audit-#2).** A `holder_pok` proof whose public inputs disagree with the bound
   digest/challenge MUST reject.
3. **Fail-closed, no silent bearer fallback.** A `holder-pop` presentation over an
   attestation lacking a `holder` binding ⇒ `HolderBindingMissing` (the
   audit-#12 `status:None ⇒ reject` precedent). Distinct domain tag `ZKSIG_C4`
   prevents cross-substituting a non-holder-bound attestation for a holder-bound
   one (the audit's domain-separation discipline).
4. **The new gate must carry a standing forge-and-verify regression test**, one
   per failure mode (key-substitution → `HolderKeyMismatch`; absent binding →
   `HolderBindingMissing`; B's-proof-with-A's-key → PoP/PoK reject; cross-credential
   digest swap → reject; replay under fresh nonce → reject), in the spirit of
   `sq-1gir`/`sq-ajl` (the audit-CRITICAL regression map) and the
   `crates/sparq-zk-compose/tests/forge_*` suite.

If any of (1)–(3) is implemented as a JSON-only check, the trusted-holder gap
**silently reopens** — this is the precise failure class `sq-gbp4` was created to
catch, so the design records it explicitly.

---

## 5. Performance / feasibility

Qualitative only (no fabricated numbers; per repo policy, any cost claim must be
measured with `bb gates -s ultra_honk` before it is asserted — see
`research/zk-soundness-audit.md` and the `noir-optimisation` discipline).

- **B1 (verifier-side, clear-key):** **no circuit cost.** One extra host-side
  Poseidon2 digest + one extra Schnorr `sig_verify` per presentation (the issuer
  holder-bound message recompute) plus the already-existing PoP check. Negligible
  next to `bb verify`.
- **B2 (in-circuit PoK):** the dominant cost is **one ~251-bit Baby-JubJub
  double-and-add scalar-mul** (`hsk·G`) plus a Poseidon2 digest. This is
  **roughly half** the heavy gadget of `schnorr_verify`, which does *two*
  scalar-muls (`s·G` and `e·pk`) — see the cost note at `issuer.nr:267-273`. So
  `holder_pok` lands **strictly inside** the existing ZK envelope: it is cheaper
  than a single hidden-issuer attestation, which the estate already proves
  (`hidden_issuer_d4`). No pairings, no foreign-field emulation (the whole point
  of the Baby-JubJub-on-BN254 choice, `issuer.nr:29-43`).
- **Feasibility verdict:** **high.** Every primitive is in-tree and already
  compiled/benchmarked for the harder `schnorr_verify` case. B1 is a small
  verifier+message change; B2 is a new member built entirely from reused gadgets.
  The actual `bb gates` figure for `holder_pok` must be measured and regression-gated
  (`sq-c5f` precedent) before any number is published.

---

## 6. Implementation plan (sequenced)

Tier 1 closes G1 with no circuit (ship first); Tier 2 adds the privacy/strong
in-circuit binding. Each step is a context-independent deliverable with its own
forge-and-verify tests.

1. **[host crypto] Holder-bound signed-message family + key digest.** Add
   `SIG_DOMAIN_HOLDER_KEY`/`SIG_DOMAIN_COMMITMENT_HOLDER`, `holder_key_digest`,
   `commitment_message_with_holder`, and a `SecretKey::sign_commitment_with_holder`
   issuance helper in `crates/sparq-zk/src/sig.rs`, with cross-vector tests
   (host digest ↔ a Noir `Poseidon2` reference, mirroring the poseidon2 cross
   tests). **(Tier 1 foundation; no deps.)**
2. **[manifest] Attestation + binding schema.** Add
   `AttestedHolderBinding { holder_pk_digest, hpk? }` to `CommitmentAttestation`
   and wire `BindingMode::HolderPop` to the attested digest, in
   `crates/sparq-zk-compose/src/manifest.rs`. (Deps: 1.)
3. **[verifier B1] Issuer-attested clear-key holder binding.** Upgrade
   `bind_holder_pop` + `bind_issuer_attestations` to select the holder-bound
   message variant, cross-check disclosed `hpk` ↔ attested digest, and require it
   fail-closed; add `HolderBindingMissing`/`HolderKeyMismatch` + a
   `HolderBindingPolicy`. Closes G1 for the clear tier. (Deps: 1, 2.)
4. **[tests B1] Forge-and-verify regression suite.** One test per §4.3(4) failure
   mode in `crates/sparq-zk-compose/tests/` (extend `forge_*`/`e2e`/`verifier_errors`).
   Asserts A-presents-B's-credential is REJECTED. (Deps: 3.)
5. **[circuit B2] `holder.nr` + `holder_pok` member.** Implement the PoK relation
   reusing `issuer.nr`'s gadgets; add `zk/compose/holder_pok/src/main.nr`; register
   the circuit-family member + `CircuitId` derive + `Prover.toml` renderer.
   Cross-vector test the in-circuit digest/scalar-mul vs the host. (Deps: 1.)
6. **[verifier B2] `bind_holder_pok` (hidden-key).** Reconstruct+byte-equal public
   inputs, canonical-vk by `CircuitId`, `bb verify`; bind `holder_pk_digest` to the
   issuer attestation; gate by policy. Hidden-holder privacy tier. (Deps: 5, 3.)
7. **[gates] Regression-gate `holder_pok` gate count** with `bb gates -s ultra_honk`
   (extend `crates/sparq-zk-compose/tests/gate_count.rs`); publish the measured
   number only after it lands. (Deps: 5.)
8. **[docs] SKILL.md + README** once end-to-end usable: update
   `crates/sparq-zk-compose/README.md` holder-binding section (remove the
   "documented deferral" caveat for the clear tier), and the
   `verifiable-credentials-zk` skill's `cnf`/key-binding note. (Deps: 6.)

---

## 7. Open questions / honest limitations recap

- **Key-sharing is not non-transferability** (§1.4). If desired later, bind `hsk`
  to a high-value secret (an "all-or-nothing" disincentive) or require
  hardware-attested non-extractable keys — both **outside** this circuit-level
  design.
- **Hidden-holder unlinkability** beyond B2's single-key hiding (e.g. proving
  membership in a holder *set* without revealing which holder, the analogue of
  `key_set_membership`) is a further privacy tier, reusing the same Merkle gadget;
  it composes with, but is not required by, G1. Filed as a follow-up if the
  use-case demands holder-set anonymity.
- **Issuer must actually bind `hpk` at issuance.** Credentials minted before this
  lands carry no `holderBinding`; the verifier treats them, fail-closed, as
  non-holder-bound (bearer) and refuses to honour a `holder-pop` claim over them.
  Re-issuance is the migration path.
