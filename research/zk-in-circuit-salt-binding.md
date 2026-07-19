# In-circuit salt binding for per-graph bnode encodings (sq-hyhj, audit-#9 fix (b))

**Status:** DESIGN ONLY — not implemented. The implementation is **toolchain-gated**
(requires `nargo`/`bb` to recompile the scan member and re-anchor its canonical VK +
`gate_count_snapshot.json`), which is absent in the default checkout. This record
de-risks the work so the toolchain-available implementation is mechanical.

**Priority:** P3. This is **defense-in-depth**, not a soundness fix. Audit-#9 "fix (a)"
(bind the salt into the issuer signature + enforce global salt-uniqueness at ingest and
in the verifier) is **already done on `main`** and is the load-bearing control; a
revoked/forged credential still fails and a salt-reusing ingester is already rejected
(`SaltReused`). This record covers the *in-circuit* alternative "fix (b)" only.

## 1. Scope boundary — what sq-hyhj owns vs. what it does NOT

The bead title reads "in-circuit salt binding **+ remaining revocation-disclosure
privacy residuals**." The revocation-disclosure residuals are already **owned by
separate open beads** and MUST NOT be duplicated under sq-hyhj:

- **`sq-6qe` (P2)** — hide status-list IRI + version on the committed-index revocation
  path. Has its own design record (`research/zk-statuslist-hide-iri-version.md`) and a
  dependent bead. **Owns the revocation IRI/version residual.**
- **`sq-93h` (P3)** — per-graph salt disclosed on the hidden-only attestation path.
  **Owns the hidden-path salt-disclosure residual.**

So the genuinely-unique, still-open core of sq-hyhj is **in-circuit salt binding** for
the scan commitment (this document). The revocation half is delegated to sq-6qe/sq-93h.

## 2. Current state (verified against code)

The per-graph bnode salt is applied **off-circuit** today:

- `crates/sparq-zk/src/encode.rs::encode_term` computes, for a blank node,
  `Enc_t(bnode) = h2(BLANK_NODE, h2(salt, blake3(canonical_label)))`
  (`encode.rs:56-59`). IRIs and literals are salt-independent.
- The scan circuit `zk/compose/compose_core/src/scan.nr` consumes the already-salted
  per-slot encodings `enc[g][i]` as witnesses and only re-hashes them
  (`leaves[i] = h3(enc[g][i][0..3])`, then `commit_fold`) up to the public
  `commitments[g]` (`scan.nr:96-104`). **The salt never enters any circuit**, and the
  raw canonical label never enters any circuit — the prover supplies `enc` directly.

Consequence (audit #9, HIGH privacy/merge-distinctness, NOT a forgery): the Q6 "bnodes
from different graphs are distinct by construction" guarantee rests on distinct
per-graph salts, but nothing in-circuit forces `enc` for a blank node to actually be
`h2(BLANK_NODE, h2(salt_g, blake3(label)))` for the attested `salt_g`. Off-circuit the
honest host does this; in-circuit it is unconstrained. Fix (a) closes the *soundness*
exposure by binding `salt_g` into the issuer signature and enforcing global uniqueness,
so a salt-reusing ingester is rejected verifier-side. Fix (b) below removes the residual
by making the salt provably load-bearing inside the proof itself.

## 3. Proposed in-circuit design (fix (b))

Constrain each blank-node slot's encoding to be derived from a per-graph salt and a raw
label digest, both witnessed, with the salt bound as a public input the verifier
reconstructs from the issuer-attested value.

### 3.1 New witnesses (private) per slot
- `is_bnode[g][i] : bool` — slot term is a blank node. Constrained consistent with the
  existing type discipline (the encoding's `TYPE_CODE_BLANK_NODE` layer).
- `label_digest[g][i] : Field` — `blake3(canonical_label)` truncated to 248 bits
  (`field_from_hash_bytes`), the same `h_s` output the module already treats as a
  witness for every term type (`encode.rs:16-24`: "the circuits must recompute `h_2`
  layers only; `h_s` outputs are witnesses").

### 3.2 New public input per graph
- `salt[g] : Field` — the per-graph RDFC10 salt. Public so the verifier byte-binds it
  (same discipline as `commitments[g]`/`attribution[g]`).

### 3.3 New in-circuit constraint (only for bnode slots)
For every slot `i` in graph `g` with `is_bnode[g][i]`:
```
inner   = h2(salt[g], label_digest[g][i])
enc_bn  = h2(BLANK_NODE, inner)
assert enc[g][i][pos] == enc_bn        // pos = the term position that is the bnode
```
Non-bnode positions keep the existing witness-encoding treatment (IRIs/literals are
salt-independent, so no change). `h2`/`h3` are the existing Poseidon2 gadgets
(`crate::hashes`), already cross-tested bit-identical to the Rust `poseidon2::hash`
(`crates/sparq-zk/tests/poseidon2_noir_cross.rs`), so no new hash surface is introduced.

### 3.4 Public-input layout + verifier reconstruction
- Append `salt[g]` (K field words, declaration order) to the scan member's public-input
  vector, AFTER the existing `commitments`/`attribution` words. Update
  `reconstruct_public_inputs` in `crates/sparq-zk-compose/src/verifier.rs` to append the
  **issuer-attested** salt (`resolve_commitment_salt`, already available — it is the
  salt the verifier uses to recompute the issuer-signed `m`), NOT the prover's declared
  bytes. This ties the in-circuit salt to the same value fix (a) already binds under the
  issuer signature, so the two controls agree by construction.
- The salt-uniqueness gate (`SaltReused`) and the salt-bound issuer-signature check are
  unchanged and remain the outer fail-closed layer.

## 4. Verification obligations (the toolchain-gated part)

These are the reason this cannot land without `nargo`/`bb`:

1. **Recompile** the affected scan member(s) (`scan_k*_n*_r*`) and **re-anchor the
   canonical VK** — the public-input arity changes, so the VK changes. Every anchored
   VK / `derive_id` circuit-id fixture for those members must be regenerated.
2. **Re-anchor `crates/sparq-zk-compose/tests/gate_count_snapshot.json`** — the added
   Poseidon2 layers raise the gate count; capture the new `bb gates` figure (do not
   hand-edit; run the tool). Expect a modest per-bnode-slot increase (two `h2` calls per
   constrained slot); size it with `bb gates` before claiming any number.
3. **Differential test (result-equivalence):** for random graphs with blank nodes,
   assert the in-circuit `enc_bn` equals the off-circuit `encode_term` output for the
   same `(salt, label)` — the circuit must reproduce `encode.rs:56-59` exactly.
4. **Forge test (goes RED on the attack):** a prover that supplies a bnode `enc` NOT
   equal to `h2(BLANK_NODE, h2(salt_g, label_digest))` — i.e. a cross-graph
   correlation handle built by reusing another graph's salted leaf — must now FAIL the
   in-circuit assertion (today it passes the circuit and is only caught, if at all, by
   the outer uniqueness gate). Add to the audit-#9 row of the forge-and-verify MAP
   (sq-1gir) alongside the existing `SaltReused` reject.
5. **Real-bb e2e** in the toolchain-gated CI lane (`.github/workflows/zk-toolchain.yml`)
   proving a non-revoked credential still VERIFIES with the new public input present.

## 5. Honesty / privacy-claims-gate note

Until §4 lands and is green in the real-bb lane, **no code or doc may claim the salt is
bound in-circuit**. The current honest statement is exactly what `ingest.rs:37-38`,
`encode.rs:5-14`, and the audit dossier CR-G6 already say: salt uniqueness is enforced
at ingest and bound into the issuer signature (fix (a), done); the in-circuit binding
(fix (b)) is deferred. This record does not change that posture — it specifies the
deferred work.

## 6. Cross-references
- `research/zk-soundness-audit.md` #9 (fix (a)/(b) statement, `encode.rs`/`scan.nr`/
  `registry.rs` locations).
- `research/zk-audit-readiness-dossier.md` CR-G6 (residual privacy deferrals register;
  `sq-hyhj` = in-circuit salt binding, `sq-93h` = hidden-path salt disclosure).
- `research/zk-verifier-reaudit.md` "Recommended beads" #3 (NEW-2b).
- `research/zk-statuslist-hide-iri-version.md` (`sq-6qe`, the revocation IRI/version
  half — NOT this bead).
