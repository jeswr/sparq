# Hiding the status-list IRI + version on the committed-index revocation path (sq-6qe)

Status: **design for review — the cryptographic PRIMITIVES (`sparq-zk::sig`) and
the HOST-SIDE accepted-set anchor (`sparq-zk-compose::revocation` +
`RevocationPolicy`) are implemented (see §6); the CIRCUIT + manifest mode +
verifier gate are still DEFERRED, so the IRI/version leak this record describes is
STILL OPEN.** Author: SPARQ agent (Claude Opus 4.8 — Fable unavailable, tag
for re-review). Inputs: the existing committed-index revocation estate
(`zk/compose/compose_core/src/revoke.nr`, `…/issuer.nr`,
`crates/sparq-zk-compose/src/{revocation.rs,verifier.rs,manifest.rs}`,
`crates/sparq-zk/src/sig.rs`), the internal skills `noir-optimisation` /
`noir-circuit-patterns`, and a grounded cost/feasibility review (June 2026).
Companion: `research/zk-soundness-audit.md`, `research/zkp-query-proofs-plan.md`.

[OPUS-4.8] This record describes work that is PROVISIONAL — graduate it into an
architecture note (or fold into the crate README / SKILL) once the circuit lands,
per the AGENTS.md "research records become architecture docs" rule. It must not be
read as describing shipped code: only §6's landed increments exist today (the
`sparq-zk::sig` primitives and the host-side accepted-set anchor). [OPUS-5]

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

Public inputs: `challenge`, `index_commitment`, `accepted_set_root`,
`min_version`. Everything else is private.

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

## 5. Cost (PRIORS — must be measured with `bb gates`)

This box runs `nargo 1.0.0-beta.21` / `bb 5.0.0-nightly.20260324`, NEWER than the
toolchain the `noir-optimisation` skill's gate figures were calibrated on, so the
numbers below are PRIORS and **must be re-measured with `bb gates -s ultra_honk -b
target/<member>.json` before any number is committed** (the skill's first rule:
"Always run `bb gates` before claiming a saving").

- **(c) freshness `version >= min_version`**: keep `version` a typed `u64` and use
  `>=` — the cheapest comparison primitive (plookup absorbs the range check; the
  skill puts `u64 <` at ~13 gates/call vs `Field::lt` ~14× more). Likely reuses an
  existing range table the circuit already pays for. Essentially free. Do NOT cast
  `version` to `Field` for the compare.
- **(a) ref open**: one extra Poseidon2 permutation (~74 gates by the carried
  estimate), a 4-input `Poseidon2::hash([...], 4)`.
- **(b) accepted-set fold**: `A` `h2` permutations (~74 each) + one `to_le_bits`
  decomposition — structurally identical to `key_set_membership`. Roughly DOUBLES
  the revocation circuit's Merkle work (two folds), but the fold is the cheap part.

Crucially this member has **no scalar mul** (unlike the Schnorr / hidden-issuer
members, whose two ~251-bit double-and-adds dominate the estate), so even with two
folds + a permutation + a `u64` compare it stays in the "tiny next to the scan /
Schnorr family" regime. Feasible and cheap — but **measure before claiming**.

Gate-counting mechanism (already in the repo): add the member under `zk/compose/`,
`nargo compile --package <member>`, `bb gates -s ultra_honk -b
target/<member>.json` → `functions[0].circuit_size`; the snapshot gate is
`crates/sparq-zk-compose/tests/gate_count_snapshot.json` (3% tolerance), re-baselined
by `bench/zk-compose/scripts/gate_counts.sh`. A new member MUST get a baseline or
`snapshot_covers_every_member` fails. Beware the per-circuit padding floor
(`noir-optimisation` §2.4): measure deltas against the existing revocation member,
not the raw floor of an isolated small circuit.

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

These are off-circuit Rust whose field outputs are exactly what the future Noir
member will recompute in-circuit (single source of truth), so the in-circuit
opening/membership will byte-match the host without drift — the same discipline
the existing `status_index_commitment` cross-vector test pins.

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

**What this does NOT do (honest):** there is no compiled `revoke_hidden_ref_*`
member, no fully-hidden `RevocationStatus` mode, and no verifier gate that
consumes the anchor. `verifier::bind_revocation` still reads
`rev.status_list` / `rev.version` in the clear on every path, so the sq-6qe
disclosure gap is UNCHANGED and the bead stays OPEN. Nothing here is externally
audited (sq-qhy4).

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
