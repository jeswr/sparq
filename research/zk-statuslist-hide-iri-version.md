# Hiding the status-list IRI + version on the committed-index revocation path (sq-6qe)

Status: **IMPLEMENTED end-to-end (sq-kndw, #2992) — the leak this record describes
is CLOSED on the fully-hidden path.** The cryptographic primitives
(`sparq-zk::sig`), the host-side accepted-set anchor, the compiled Noir member
`revoke_hidden_ref_d10_a4`, the manifest mode, and the verifier gate
`bind_fully_hidden_revocation` all exist (see §6 / §8). Two DELIBERATE DEVIATIONS
from the design below were made during implementation and are recorded inline:
`ref_commitment` is a PUBLIC input (§3, soundness), and the §5 cost prior was
WRONG (§5, measured). The clear-index and committed-index paths are unchanged.
NOT externally audited (sq-qhy4); no soundness / privacy property is asserted as
achieved. Author: SPARQ agent (Claude Opus 4.8 — Fable unavailable, tag
for re-review; implemented + corrected by Opus 5 under sq-kndw). Inputs: the existing committed-index revocation estate
(`zk/compose/compose_core/src/revoke.nr`, `…/issuer.nr`,
`crates/sparq-zk-compose/src/{revocation.rs,verifier.rs,manifest.rs}`,
`crates/sparq-zk/src/sig.rs`), the internal skills `noir-optimisation` /
`noir-circuit-patterns`, and a grounded cost/feasibility review (June 2026).
Companion: `research/zk-soundness-audit.md`, `research/zkp-query-proofs-plan.md`.

[OPUS-5] sq-kndw: the circuit HAS landed, so this record is now a DESIGN +
IMPLEMENTATION record rather than a proposal; per the AGENTS.md "research records
become architecture docs" rule it should be folded into the crate README / SKILL
surface on the next docs pass. §8 records what shipped and where.

## 1. The gap

sq-ayv closed the index + liveness-bit leak on the committed-index revocation
path: the issuer signs `status_ref_commit_digest(H(list), index_commitment,
version)` — a HIDING commitment to the index, not the clear index — and the
hidden-index proof (`revoke_unset_check_committed`) cross-binds the proven-unset
index to that commitment. A hidden-revocation presentation therefore discloses
neither the index nor the liveness bit.

But the **status-list IRI** (`H(list)`) and the **version** are still folded into
the signed digest IN THE CLEAR, and `RevocationStatus.{status_list, version}` are
disclosed in the manifest. This leaves a coarser linkability / correlation
channel: a relying party learns WHICH status list and WHICH publication epoch a
presentation pertains to. Two presentations of credentials from the same issuer's
list (or two presentations citing the same epoch) are correlatable on that axis.
sq-6qe closes it: bind/commit the list IRI + version too, and prove freshness
in-zk against a committed version, so a hidden-revocation presentation discloses
**neither index, bit, IRI, nor version**.

This is the verifier-side note flagged in `verifier::bind_revocation`'s "HONEST
REMAINING DISCLOSURE" doc.

## 2. The design tension

The verifier, to run the bit-unset Merkle inclusion check, needs the
**status-list root** for the credential's `(list, version)`. Today it resolves
that root by looking the snapshot up by the CLEAR `(list, version)` in
`RevocationPolicy.authoritative` and folding it (`merkle_root`). If `(list,
version)` is hidden, the verifier can no longer name the snapshot — so the root
must be resolved WITHOUT the verifier learning which list/version.

## 3. Chosen design — sub-option A: bind the root inside an accepted-set leaf

The issuer signs over a HIDING reference commitment instead of clear
`(list, version)`:

```text
ref_commitment = Poseidon2([DOMAIN_RC, list_id, version, ref_blinding], 4)
signed digest  = Poseidon2([DOMAIN_FC, ref_commitment, index_commitment], 3)
```

(`DOMAIN_RC` / `DOMAIN_FC` are distinct tags from the clear-index and
committed-index-clear-list digests, so no disclosure-mode substitution is
possible.) The holder withholds `list_id`, `version`, `ref_blinding`; the manifest
carries only `ref_commitment` (and `index_commitment`, as today).

The relying party publishes a single **accepted-set Merkle root** over leaves

```text
accepted_leaf = Poseidon2([DOMAIN_AL, list_id, version, status_list_root], 4)
```

for every `(list, version)` it currently trusts (built from its own authoritative,
freshness-curated snapshots — the audit-#12 trust anchor, moved behind a
commitment; no new trust assumption). A NEW circuit member
(`revoke_hidden_ref_d{D}_a{A}`, depth `D` for the status-list tree, depth `A` for
the accepted set) proves, in zero knowledge:

- **(a) ref open**: `ref_commitment` recomputes from the private `(list_id,
  version, ref_blinding)` — ties the proof to the issuer-signed reference.
- **(b) accepted-set membership**: `accepted_leaf(list_id, version,
  status_list_root)` is a member of the PUBLIC accepted-set root, at a private
  index/path. This is `issuer.nr::key_set_membership` generalised from a 2-input
  `key_leaf` to a 4-input leaf — and, crucially, it PRIVATELY BINDS the
  `status_list_root` for the hidden `(list, version)`.
- **(c) freshness**: `version >= min_version` for a PUBLIC `min_version` floor —
  a typed `u64` comparison (cheap; see §5). `version` has no witness freedom (it
  is bound through both (a) and (b)), so a holder cannot prove an OLD version is
  fresh.
- **(d) bit-unset inclusion**: the existing `revoke_unset_check_committed` body,
  but folding against the PRIVATE `status_list_root` from (b) instead of a public
  `root`, and cross-binding `index_commitment` to the proven-unset index as today.

Public inputs: `challenge`, **`ref_commitment`**, `index_commitment`,
`accepted_set_root`, `min_version`. Everything else is private.

> **[OPUS-5] sq-kndw CORRECTION — `ref_commitment` must be PUBLIC.** The original
> list above omitted it, which is UNSOUND: with `ref_commitment` private, relation
> (a) recomputes a value that is never compared to anything the verifier knows, so
> it constrains nothing and a holder could prove liveness for ANY accepted
> `(list, version)` — including one whose reference the issuer never signed for
> this credential. Publishing it restores exactly the sq-ayv cross-binding
> discipline (the verifier byte-matches it against the ISSUER-SIGNED
> `AttestedStatusRef::ref_commitment`). It is a hiding commitment, so publishing it
> discloses neither the IRI nor the version — but it IS a stable per-credential
> handle, which is precisely why §4's re-blinding requirement is load-bearing.

### Why sub-option A over a public candidate-root set (B)

(B) — the verifier publishes a SET of candidate roots and the proof shows the fold
holds against one (an OR / disjunction) — leaks strictly more (the candidate-root
list and its cardinality) and costs N folds. **A** leaks only the accepted-set
root, costs ONE membership fold, and binds the root atomically inside the leaf so
a prover cannot pair list₁'s identity with list₂'s root. Sub-option A is the
clear choice.

## 4. Disclosure floor after sq-6qe (honest)

A hidden-revocation presentation then discloses **no holder-identifying
attribute**: not the index, bit, list IRI, nor version. The statement reduces to
"some accepted `(list, version)` in the relying party's committed set, with
`version >= min_version`, has my hidden index unset."

Residual disclosures — all policy-side / structural, NOT holder-identifying:

- **accepted-set root** = the RP's policy fingerprint (stable across presentations
  to that RP; same character as the hidden-issuer `key_set_root` already accepted).
- **public `min_version`** = the coarse epoch FLOOR of the RP's policy, not the
  credential's epoch.
- **circuit member depth(s)** `D` / `A` (via the vk / member name) = a cardinality
  bound (≤ 2^D leaves, ≤ 2^A accepted entries). Inherent to fixed-depth Merkle.

**Conditional dependency — call out loudly:** the whole privacy guarantee is
undone if `index_commitment` (or `ref_commitment`) is REUSED across presentations
— a static commitment is itself a cross-presentation linkage handle. The
holder/issuer flow MUST re-blind per presentation (fresh `blinding` and
`ref_blinding`), or use a re-randomisable commitment. This is the single most
important operational requirement of the design.

## 5. Cost — MEASURED (`bb gates -s ultra_honk`)

> **[OPUS-5] sq-kndw: the priors below were WRONG and are superseded by the
> measurement.** Kept visible because the way they were wrong is the useful part.

**Measured**: `revoke_hidden_ref_d10_a4` = **4308** `circuit_size`, against
`revoke_unset_d10` = 899 — i.e. `+3409`, ~4.8x the base circuit, not the "one
permutation + one fold + an essentially-free compare" the priors predicted. Taken
on linux x86_64 with the pinned toolchain (`nargo 1.0.0-beta.21` /
`bb 5.0.0-nightly.20260324`); every PRE-EXISTING member re-measured
byte-identical to its checked-in baseline on the same box, so the number is
comparable rather than platform-shifted. Baselined in
`crates/sparq-zk-compose/tests/gate_count_snapshot.json` +
`bench/zk-compose/gate_counts_latest.json` (NON-CANONICAL work-box numbers, as
always).

**Attribution** (probe circuits compiled and measured one relation at a time —
not inferred):

| component | gates |
|---|---|
| `revoke_unset_check_committed` D=10 base | 899 |
| FIRST integer-typed input (range/lookup-table setup) | **+2785** |
| the `version >= min_version` compare itself | +~10 |
| two `h4` permutations (ref open + accepted-set leaf) | +~150 |
| the A=4 accepted-set Merkle fold (4x `h2` + `to_le_bits`) | +~463 |
| **total** | **4308** |

**Where the prior went wrong.** §3(c) predicted the `u64` freshness compare would
be "essentially free — plookup absorbs the range check". The PER-CALL figure was
right (~10 gates, matching `noir-optimisation`'s ~13 gates/call for `u64 <`), but
the ONE-TIME cost of introducing the first integer-typed value into a circuit that
previously had none was unaccounted: an **UNUSED** `u64` input measures the same
`+2785`, so it is lookup-table setup, not the comparison. Generalisable lesson for
this estate: in a small Poseidon-only circuit, the first `uN` you introduce costs
roughly 3x the whole circuit; in a circuit that already pays for the table (the
`filter_*` family) it is genuinely near-free. Always measure the DELTA in the
target circuit, never carry a per-call figure across circuits.

**Is the compare worth 2795 gates?** It is defence-in-depth: §6b's freshness
CURATION already excludes stale/future versions from the accepted set, so relation
(c) is not the only freshness check (see §6b). It was kept because the curation is
a host-side policy discipline while (c) is a circuit-enforced invariant, and 4308
gates is still far below `hidden_issuer_d4` (24452) — there is no scalar mul on
this path. A future member that drops (c) to save the table is a legitimate
trade-off but would rest the whole freshness guarantee on curation alone.

Gate-counting mechanism (unchanged): `nargo compile --package <member>`,
`bb gates -s ultra_honk -b target/<member>.json` -> `functions[0].circuit_size`;
re-baseline both JSONs together via `bench/zk-compose/scripts/gate_counts.sh`, and
regenerate `bench/zk-compose/bb_gates_matrix.json` (its coverage gate requires a
row per member). Beware the per-circuit padding floor (`noir-optimisation` §2.4):
measure deltas against the existing revocation member, not an isolated small
circuit.

## 6. What is implemented now (the tractable increments)

### 6a. The `sparq-zk::sig` primitives

The cryptographic PRIMITIVES the deferred circuit + verifier will consume are
landed in `crates/sparq-zk/src/sig.rs`, domain-separated and unit-tested
(hiding / binding / domain-separation / atomic-triple-binding), mirroring how the
sq-ayv `status_index_commitment` + `status_ref_commit_digest` primitives landed
before that path's circuit:

- `sig::status_ref_commitment(list_id, version, ref_blinding) -> Fr` — the hiding
  `(list, version)` commitment of §3, `Poseidon2([DOMAIN_RC, list_id, version,
  ref_blinding], 4)`.
- `sig::status_ref_fully_committed_digest(ref_commitment, index_commitment) -> Fr`
  — the issuer-signed digest of §3, `Poseidon2([DOMAIN_FC, ref_commitment,
  index_commitment], 3)`, distinct-domain from both other modes.
- `sig::accepted_status_leaf(list_id, version, status_list_root) -> Fr` — the
  accepted-set Merkle leaf of §3 (sub-option A), `Poseidon2([DOMAIN_AL, list_id,
  version, status_list_root], 4)`.

These are off-circuit Rust whose field outputs are exactly what the Noir member
recomputes in-circuit (single source of truth), so the in-circuit
opening/membership byte-matches the host without drift — the same discipline
the existing `status_index_commitment` cross-vector test pins. [OPUS-5] sq-kndw:
the member's `SIG_DOMAIN_STATUS_REF_COMMITMENT` / `SIG_DOMAIN_ACCEPTED_STATUS_LEAF`
globals carry the SAME `ZKSIG_RC` / `ZKSIG_AL` tags, and the agreement is pinned
EMPIRICALLY by a real `bb prove`/`verify` round trip
(`full_manifest_fully_hidden_revocation`) rather than only by matching constants.

### 6b. [OPUS-5] The host-side ACCEPTED-SET anchor (`sparq-zk-compose`)

The §3 sub-option-A trust anchor and the prover's membership path are landed in
`crates/sparq-zk-compose/src/revocation.rs` + `verifier.rs`, unit-tested:

- `revocation::{AcceptedStatusEntry, accepted_set_leaf, accepted_set_root,
  accepted_set_witness}` — the accepted-set Merkle tree over
  `sig::accepted_status_leaf(list_id, version, status_list_root)` leaves, built on
  the SHARED sparse fold (`issuer::sparse_root_from_leaves` /
  `sparse_witness_from_leaves`, `Fr::from(0)` padding), so it is bit-identical to
  the tree the generalised `key_set_membership` relation folds and costs
  `O(n·set_depth)` rather than `O(2^set_depth)`.
- `RevocationPolicy::{with_accepted_set_depth, accepted_entries, accepted_set_root,
  accepted_member_index, min_version}` — the relying party derives the anchor from
  its OWN authoritative snapshots, FRESHNESS-CURATED (only versions inside
  `[min_version, now]` become leaves) and in the canonical sorted
  `(status_list, version)` order both sides must commit. `min_version` is now
  public: it is the designed public epoch-FLOOR input of §3(c).

Because membership is restricted to the curated window, the audit-#12 freshness
gate SURVIVES the move behind the commitment: a stale or future-dated version is
not a member, so no proof can be built against it. The in-circuit
`version >= min_version` of §3(c) is then defence-in-depth, not the only check.

**[OPUS-5] sq-kndw — superseded.** When §6b landed there was no compiled member,
no fully-hidden manifest mode, and no verifier gate, so the disclosure gap was
still open. All three now exist (§7), and `verifier::bind_revocation` routes a
fully-hidden reference to `bind_fully_hidden_revocation` instead of reading
`rev.status_list` / `rev.version`. It still reads both in the clear on the
CLEAR-INDEX and COMMITTED-INDEX paths, which are deliberately unchanged. Nothing
here is externally audited (sq-qhy4).

## 7. What shipped (sq-kndw, #2992) — the deferred list, closed

Every item of the original deferred list landed. Mapping, for review:

1. **Noir member** — `zk/compose/compose_core/src/revoke.nr::revoke_hidden_ref_check<D, A>`
   (relations (a)-(d) of §3, with `ref_commitment` public per the §3 correction)
   plus the compiled bin `zk/compose/revoke_hidden_ref_d10_a4`. Baselined at 4308
   (§5). `hashes.nr` gained `h4` for the 4-input leaves. The host<->circuit
   agreement is pinned EMPIRICALLY by a real `bb prove`/`verify` round trip
   (`full_manifest_fully_hidden_revocation`), which is also the anchor for the
   public-input byte layout the verifier reconstructs by hand.
2. **`CircuitId` + derive + renderer + witness builder** —
   `CircuitId::RevokeHiddenRef { depth, set_depth }` (package
   `revoke_hidden_ref_d{depth}_a{set_depth}`), `build::derive_revoke_hidden_ref_id`
   (EXACT-match against `REVOKE_HIDDEN_REF_MEMBERS`, the single source of the
   compiled family list), `revocation::revoke_hidden_ref_prover_toml`, and
   `revocation::{HiddenRefWitness, hidden_ref_witness}` (reusing the sparse
   `merkle_witness` + `accepted_set_witness`, and fail-closed when the prover's
   bitstring disagrees with the accepted entry's bound root).
3. **Manifest mode** — `RevocationStatus` gained `ref_commitment` and made
   `status_list` / `version` optional, giving the THIRD disclosure mode
   (`status_list`/`index`/`version` all `None`). `AttestedStatusRef` mirrors it.
   Constructors (`::clear` / `::committed` / `::fully_hidden`) make the illegal
   combinations hard to build. The proof rides in a SEPARATE
   `ProofManifest::fully_hidden_revocation: Option<FullyHiddenRevocation>` rather
   than more `Option`s inside `HiddenIndexRevocation` — the two modes have disjoint
   public-input vectors and disjoint trust anchors, so keeping them apart makes the
   mixed state unrepresentable and leaves the audited committed-index gate's code
   path untouched.
4. **Verifier gate + issuer attestation** — `verifier::bind_fully_hidden_revocation`
   (anchors from the relying party's OWN policy, public inputs rebuilt from them,
   canonical vk, `bb verify`), and the third arm of the `resolve_status_ref`
   chokepoint resolving `status_ref_fully_committed_digest`. No new issuer SIGNING
   API was needed: `SecretKey::sign_commitment_with_status` is digest-agnostic.
5. **Re-blinding** — enforced, not just documented: the gate records a
   domain-separated linkage tag `Poseidon2([ZKLINK_1, ref_commitment,
   index_commitment])` in the SAME durable `SeenNonces` store the audit-#4 nonce
   defence uses, and rejects a repeat
   (`FullyHiddenRevocationLinkageReplay`). See the honest limit in §4a below.
6. **SKILL.md** — `skills/zk-query-proofs/SKILL.md` updated to describe the mode as
   usable, with the disclosure floor and the re-blinding requirement stated.

### 4a. The honest limit of the re-blinding enforcement

Single-use of the `(ref_commitment, index_commitment)` pair only helps against an
HONEST relying party: a malicious one simply does not run the check, and by the
time it could it has already observed the pair. The enforcement is worth having
because it makes the requirement OPERATIONAL rather than advisory and turns silent
linkability into a visible rejection — but the real fix is upstream. Re-blinding
requires the ISSUER to mint a fresh `(ref_blinding, blinding)` pair and RE-SIGN per
presentation, because the digest the issuer signs folds both commitments. A
re-randomisable commitment + signature scheme would remove that round trip; sparq
does not implement one. **This is the residual operational gap of the design.**

## 8. Remaining follow-ups

- Only the `(D=10, A=4)` bucket is compiled — up to 1024 status indices and 16
  accepted `(list, version)` pairs. Wider buckets are mechanical (both relations
  are depth-generic) but each needs its own measured baseline.
- The issuer-side re-signing flow of §4a is not automated anywhere; a holder using
  this mode today must arrange fresh per-presentation issuance itself.
- Externally-accredited cryptographer review remains PENDING (sq-qhy4). Nothing
  here is externally audited and no soundness / privacy property is asserted as
  achieved.

## Appendix: the original deferred list (superseded by §7)

## 7. Deferred (the follow-up beads)

1. The Noir member `revoke_hidden_ref_d{D}_a{A}` (relations (a)–(d) of §3),
   cross-vector-tested against the §6a host primitives + the §6b accepted-set
   leaf/fold (poseidon2_noir_cross style), plus a `gate_count_snapshot.json`
   baseline (§5).
2. A new `CircuitId` variant + `derive_*` id and a `Prover.toml` renderer. The
   host witness builders are DONE (§6b `accepted_set_witness` for the accepted-set
   tree, `merkle_witness` for the status-list tree); only the renderer, which needs
   the member's input names, is outstanding.
3. Manifest: a fully-hidden `RevocationStatus` mode (`status_list` / `version` /
   `index` all `None`, carrying `ref_commitment` + `index_commitment`) and a
   `HiddenIndexRevocation` carrying `accepted_set_root` + `min_version`.
4. Verifier: a `bind_fully_hidden_revocation` gate that supplies
   `RevocationPolicy::accepted_set_root()` + `min_version()` (§6b) as public inputs
   and `bb verify`s — never trusting the prover's declared root. Plus the
   issuer-side `status_ref_fully_committed_digest` attestation path in
   `bind_issuer_attestations` / `resolve_status_ref`.
5. The re-blinding requirement (§4) enforced/documented in the holder flow — the
   single most important operational requirement, and NOT yet addressed anywhere.
6. SKILL.md (`skills/zk-query-proofs`): document the fully-hidden MODE once it is
   end-to-end usable. (The §6b host anchor is already listed there, explicitly
   flagged as anchor-only with the leak still open.)
