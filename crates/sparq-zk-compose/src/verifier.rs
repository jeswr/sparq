// [OPUS-4.8] written while Fable 5 unavailable — re-review when Fable returns.
//! Manifest verifier (plan §S4.E module iii, layer 3).
//!
//! Verification stages, all of which must pass:
//! 1. **Manifest re-checks** (cheap, no proving): re-parse the query and run
//!    `sparq_zk::verify::recheck` — the bnode cross-graph join guard (Q6) plus
//!    attribution arity. Re-derive each sub-proof's circuit id from its public
//!    inputs and confirm it equals the declared id (a proof cannot be replayed
//!    against a different family member).
//! 2. **Binding-consistency edges**: each declared edge's scan-proof row/slot
//!    encoding must equal the consuming filter proof's `operand_enc` (a plain
//!    field equality over public inputs — the modular composition seam).
//!
//! 2d. **Issuer-signature / key-set binding (audit #3, codex #1).** Every scan
//!    sub-proof's `commitments[g]` must carry an issuer attestation
//!    ([`crate::manifest::CommitmentAttestation`]) whose Schnorr signature
//!    verifies under its declared issuer key and whose key is a member of the
//!    EXTERNAL trusted key-set `K` — the relying party's [`KeySet`] argument,
//!    NOT the prover-supplied `manifest.key_set` (trusting the latter as the
//!    anchor was the codex #1 soundness hole: a prover signs a forgery with its
//!    own key and self-lists it). `manifest.key_set` is only accepted as a
//!    SUBSET of the external `K`. `commitments[g]` is byte-bound into the bb
//!    public inputs by stage 3a, so this ties the attested commitment to the
//!    proved statement: an unsigned/prover-invented commitment, a
//!    drop-a-triple-and-recommit suppression (the truncated `C(G')` has no valid
//!    attestation), a key-not-in-external-`K` signature, and a prover key-set
//!    that widens `K` all REJECT. Interim privacy note: this reveals WHICH
//!    issuer signed; the in-circuit undisclosed-key upgrade (see
//!    `sparq_zk::sig`) removes that.
//! 3. **bb verification (the cryptographic gate)**, for each sub-proof, all of:
//!    a. **Public-input reconstruction (audit #1).** Independently rebuild the
//!   public-input field vector from the DECLARED [`ProofInputs`] (in `main`
//!   declaration order) using the verifier's own challenge, serialize to
//!   bb's byte layout (32-byte BE field elements, no header — see
//!   [`reconstruct_public_inputs`]), and assert byte-equality with the
//!   prover's `public_inputs` blob. This binds the JSON statement to the
//!   proof; without it stages 1-2 (JSON) and the proof (a detached crypto
//!   object) describe potentially different statements.
//!    b. **Canonical vk (audit #2).** Recompute the vk verifier-side from the
//!   compiled member named by the re-derived [`CircuitId`] (never the
//!   prover's `art.vk`), pinning the vk to the FULL CircuitId (subsumes the
//!   #11 n/d/r relabel).
//!    c. `bb verify` over (prover proof, reconstructed public inputs, canonical
//!   vk).
//!
//! 2f. **Revocation / freshness (audit #12, + re-audit Option B).** Leverages #3:
//!    the issuer signature ALSO binds the credential's STATUS-LIST REFERENCE
//!    (`status_ref_digest(H(list IRI), index, version)`), so a scan-covering
//!    attestation MUST carry an issuer-bound status reference (mandatory /
//!    fail-closed — a `status: None` attestation is rejected, and an omitted
//!    `manifest.revocation` leaves the status-bound signature uncheckable and is
//!    rejected). The verifier then recomputes the digest from the disclosed
//!    `manifest.revocation`, requires it to match the issuer-signed
//!    `AttestedStatusRef`, and — THE RE-AUDIT FIX — reads the credential's status
//!    BIT from the relying party's OWN AUTHORITATIVE [`crate::manifest::StatusListSnapshot`]
//!    for `(list, version)`, carried EXTERNALLY in [`RevocationPolicy`] (exactly
//!    as the trusted key-set `K` is external), NOT from the prover's
//!    `manifest.status_snapshots`. The issuer signature binds the REFERENCE but
//!    NOT the bit VALUES, so trusting the prover's bitstring let a genuine
//!    reference + a forged all-zero snapshot reverify a REVOKED credential; the
//!    bit is now sourced off prover-controlled bytes. It asserts (i) the
//!    reference version is within the relying party's freshness window, (ii) the
//!    AUTHORITATIVE snapshot's bit at `index` is UNSET, and (iii) any prover
//!    snapshot for the same `(list, version)` byte-equals the authoritative one
//!    (a tamper tripwire). A REVOKED bit, a STALE reference, a missing
//!    AUTHORITATIVE snapshot, a disagreeing prover snapshot, or an
//!    omitted/forged/mismatched reference all REJECT. [OPUS-4.8] sq-ayv: a
//!    credential may instead use the COMMITTED-index path — the issuer signs
//!    `status_ref_commit_digest(H(list), index_commitment, version)` (a hiding
//!    commitment to the index), the clear index is WITHHELD, and liveness is
//!    checked by the hidden-index proof cross-bound to that commitment (so neither
//!    the index nor the bit is disclosed). The clear-index path above is unchanged
//!    for clear references. See [`bind_revocation`] / [`bind_hidden_revocation`].
//!
//! 4. **Freshness / single-use (audit #4).** [`verify_manifest`] takes a
//!    VERIFIER-ISSUED fresh [`VerifierNonce`] (minted out of band, handed to the
//!    prover before proving) and a single-use [`SeenNonces`] store. It (i)
//!    records the nonce single-use BEFORE the crypto gate (replay of the same
//!    nonce => [`CheckError::NonceReplay`]); (ii) requires `manifest.binding`'s
//!    declared challenge to equal the nonce (fail-closed); and (iii) feeds the
//!    nonce — NOT `manifest.binding` — as field 0 of the stage-3a reconstruction,
//!    so a proof committed under any OTHER challenge fails the byte-compare. A
//!    captured manifest re-presented under a fresh nonce is rejected by the
//!    byte-compare; the same manifest re-presented under its original nonce is
//!    rejected by the store. (Field 0 stays an unconstrained in-circuit tag — the
//!    binding is wholly verifier-side via the audit-#1 byte-compare, so no circuit
//!    change is needed.)
//!
//! Stage 1+2 run WITHOUT bb (the fast structural gate); stage 3 is the
//! cryptographic gate; stage 4 (freshness) is enforced by `verify_manifest`
//! around stage 3. [`prefilter_manifest_structure`] runs stages 1+2 ONLY and
//! is NOT a sound verifier (it binds nothing to a proof and enforces NO
//! freshness — see its docs); [`verify_manifest`] is the sound public entry
//! point: it runs the pre-filter then the freshness + bb verify +
//! public-input reconstruction that binds the JSON statement (incl. attribution
//! bits) and the verifier's nonce to the proofs.

use crate::build::{derive_filter_f64_id, derive_filter_int_id, derive_scan_id};
use crate::driver::{CircuitProver, DriverError};
use crate::manifest::{
    BindingMode, CircuitId, EntailmentRegime, FieldHex, ProofInputs, ProofManifest,
    StatusListSnapshot,
};
// [OPUS-4.8] sq-314: derivation-step re-check for entailment-regime enforcement.
use crate::derivation::regime_admits;
// [OPUS-4.8] sq-3e5 + sq-h2v: hidden-index revocation root derivation.
use crate::revocation::merkle_root;
use sparq_zk::encode::encode_term;
use sparq_zk::field::{field_from_hex_str, field_to_be_bytes_32, field_to_hex, Fr};
// [OPUS-4.8] codex 2221 HIGH: only the SALT-BOUND `commitment_message_with_salt`
// is used on the scan-verify path; the bare salt-less `commitment_message` is no
// longer reachable here (a scan-covering attestation must be salt-bound).
// [OPUS-4.8] audit #12: the STATUS-BOUND `commitment_message_with_status` is the
// scan-verify path's signed message — a scan-covering attestation must bind the
// status reference (status_ref_digest over the disclosed list/index/version).
use sparq_zk::sig::{
    commitment_message_with_status, holder_pop_message, public_key_from_hex, signature_from_hex,
    status_list_id_to_field, status_ref_commit_digest, status_ref_digest, verify as sig_verify,
    SignatureScheme,
};
use sparq_zk::verify::{
    fragment_filters, fragment_pattern_consts, fragment_patterns, recheck, variable_slots,
    FilterCmp, JoinEdge, QueryFilter, VerifyError,
};
use std::collections::BTreeSet;
use std::path::Path;

/// The relying party's EXTERNALLY-anchored trusted issuer key-set `K` — the
/// soundness fix for audit #3 codex finding #1.
///
/// # Why this is a verifier input, not a manifest field
/// The manifest carries a `key_set` field, but it is PROVER-SUPPLIED: a
/// malicious prover signs a forged commitment with its OWN key and lists that
/// key in `manifest.key_set`, so a verifier that trusts `manifest.key_set` as
/// the anchor gives NO "authoritative source" guarantee — `#3` would be
/// vacuous. The trust anchor MUST come from outside the proof. A relying party
/// constructs a [`KeySet`] from the issuer keys IT decides to trust (its policy
/// / an issuer-key registry it resolves out of band) and passes it into
/// [`verify_manifest`] (the sound entry point) / [`prefilter_manifest_structure`]
/// (the structural pre-filter). The accept decision then
/// depends only on this external set; `manifest.key_set`, if present at all, is
/// only accepted when it is a SUBSET of this external set (checked, never
/// trusted as the anchor) — see [`bind_issuer_attestations`].
///
/// Keys are stored in normalized hex form (no `0x`, lowercase) and validated as
/// parseable, non-identity Baby-JubJub points at construction (an unparseable or
/// identity key can never be a real issuer key, so it is dropped fail-closed).
// [OPUS-4.8] audit #3 codex #1: external trust anchor.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KeySet {
    keys: BTreeSet<String>,
    /// OPTIONAL Merkle-tree depth for the HIDDEN-ISSUER attestation proof
    /// (sq-z9l). When set, the verifier derives the authoritative key-set Merkle
    /// root from THIS set (canonical order) at this depth and accepts a
    /// `manifest.hidden_issuer_attestations` proof whose PUBLIC root byte-equals
    /// it — proving "signed by SOME key in K" without disclosing which. `None` =>
    /// the hidden-issuer path is disabled (the verifier only runs the clear-key
    /// `bind_issuer_attestations` check). MUST equal the `hidden_issuer_d{depth}`
    /// member the prover used.
    // [OPUS-4.8] sq-z9l: opt-in hidden-issuer verification depth.
    hidden_issuer_depth: Option<u32>,
}

impl KeySet {
    /// An empty trust anchor: trusts NO issuer. Any scan carrying commitments is
    /// then rejected (fail closed). Useful as the explicit "no source is
    /// authoritative" policy and as a test default.
    pub fn empty() -> Self {
        KeySet { keys: BTreeSet::new(), hidden_issuer_depth: None }
    }

    /// Build a trust anchor from the relying party's trusted issuer public keys
    /// (hex). Each key is validated as a parseable, non-identity Baby-JubJub
    /// point (`public_key_from_hex` already rejects the identity — codex #3) and
    /// stored in normalized hex; unparseable/identity entries are dropped (they
    /// can never match a real attestation key, so dropping them is fail-closed,
    /// not a silent widening of trust).
    pub fn from_hex_keys<I, S>(keys: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let keys = keys
            .into_iter()
            .filter_map(|h| public_key_from_hex(h.as_ref()).map(|_| normalize_hex(h.as_ref())))
            .collect();
        KeySet { keys, hidden_issuer_depth: None }
    }

    /// Enable the HIDDEN-ISSUER attestation path (sq-z9l) at Merkle depth `depth`
    /// (builder style). The verifier will then derive the authoritative key-set
    /// Merkle root from THIS set (canonical order) at `depth` and accept a
    /// `manifest.hidden_issuer_attestations` proof whose PUBLIC root matches —
    /// proving "signed by SOME key in K" without disclosing which. `depth` MUST
    /// equal the `hidden_issuer_d{depth}` member the prover used. The clear-key
    /// `bind_issuer_attestations` path is unaffected.
    // [OPUS-4.8] sq-z9l: opt-in hidden-issuer verification depth.
    pub fn with_hidden_issuer_depth(mut self, depth: u32) -> Self {
        self.hidden_issuer_depth = Some(depth);
        self
    }

    /// The hidden-issuer Merkle depth, if the relying party enabled the path.
    /// `None` => disabled (a `manifest.hidden_issuer_attestations` is not accepted).
    // [OPUS-4.8] sq-z9l.
    fn hidden_issuer_depth(&self) -> Option<u32> {
        self.hidden_issuer_depth
    }

    /// Whether `pk_hex` (any case, optional `0x`) is a member of the trusted set.
    fn contains_hex(&self, pk_hex: &str) -> bool {
        self.keys.contains(&normalize_hex(pk_hex))
    }

    /// The trusted set is empty (trusts no issuer).
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// The trusted keys in the canonical CANONICAL leaf order for the hidden-issuer
    /// key-set Merkle tree (sq-z9l): the normalized-hex set's sorted order
    /// (`BTreeSet` iteration). Both the relying party (deriving the authoritative
    /// `key_set_root`) and the prover (building its membership path) commit K in
    /// THIS order, so the roots agree. Parsed back to [`PublicKey`]s (every stored
    /// key is already validated parseable at construction, so this never drops a
    /// member).
    // [OPUS-4.8] sq-z9l: canonical leaf order for the key-set commitment.
    fn ordered_keys(&self) -> Vec<sparq_zk::sig::PublicKey> {
        self.keys
            .iter()
            .filter_map(|h| public_key_from_hex(h))
            .collect()
    }

    /// The authoritative hidden-issuer key-set Merkle root (sq-z9l) over the
    /// trusted keys in canonical order, at depth `depth`. This is the TRUST ANCHOR
    /// the relying party derives from its OWN [`KeySet`] (exactly as
    /// `RevocationPolicy` derives the status root from its own snapshot), and which
    /// a `manifest.hidden_issuer_attestations` proof's PUBLIC `key_set_root` must
    /// byte-equal. `None` if the set overflows the tree or `depth` is implausible.
    // [OPUS-4.8] sq-z9l.
    pub fn hidden_issuer_root(&self, depth: u32) -> Option<Fr> {
        crate::issuer::key_set_root(&self.ordered_keys(), depth)
    }

    /// The 0-based index of `pk_hex` in the canonical leaf order, if it is a
    /// member — the slot the prover proves membership at. (Prover-side convenience;
    /// the verifier never needs the index, which stays private.)
    // [OPUS-4.8] sq-z9l.
    pub fn member_index(&self, pk_hex: &str) -> Option<usize> {
        let target = normalize_hex(pk_hex);
        self.keys.iter().position(|k| *k == target)
    }
}

/// The relying party's EXTERNALLY-anchored set of trusted HOLDER keys (sq-cwq) —
/// the trust anchor for the `HolderPop` binding's proof-of-possession.
///
/// # Why this is a verifier input, not a manifest field
/// Exactly the audit-#3 external-`K` precedent: the holder key the PoP is checked
/// against MUST come from outside the proof, or any party could mint a key, sign
/// the challenge with it, list it in the manifest, and pass — a PoP anchored on a
/// prover-chosen key proves nothing. A relying party constructs a `HolderRegistry`
/// from the holder keys IT authorises (its policy / a holder-key registry it
/// resolves out of band) and passes it into [`verify_manifest`]. The accept
/// decision for a `HolderPop` binding then depends only on this external set.
///
/// # Fail-closed
/// An EMPTY registry trusts no holder: a `HolderPop` binding presented against an
/// empty registry is REJECTED ([`CheckError::HolderRegistryEmpty`]). There is NO
/// path on which a `HolderPop` binding is accepted as a bare challenge (the
/// previous placeholder behaviour, which silently waived the holder check). A
/// relying party that does not use holder binding simply uses
/// [`BindingMode::Challenge`]; one that DOES must supply a non-empty registry.
///
/// # Scope (honest deferral)
/// Membership here means "this holder key is authorised to present" — it does NOT
/// bind the key to a SPECIFIC credential. Issuer-attested credential↔holder
/// binding (the issuer signing the holder key into the credential) is deferred;
/// see [`bind_holder_pop`]. Keys are stored normalized (no `0x`, lowercase) and
/// validated as parseable, non-identity Baby-JubJub points at construction
/// (unparseable/identity entries dropped fail-closed).
// [OPUS-4.8] sq-cwq: external holder trust anchor (mirrors KeySet).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HolderRegistry {
    holders: BTreeSet<String>,
}

impl HolderRegistry {
    /// An empty registry: trusts NO holder. A `HolderPop` binding is then rejected
    /// (`HolderRegistryEmpty`) — the explicit "holder binding not in use" anchor
    /// and the test default. (A `Challenge` binding is unaffected.)
    pub fn empty() -> Self {
        HolderRegistry { holders: BTreeSet::new() }
    }

    /// Build a registry from the relying party's authorised holder public keys
    /// (hex). Each is validated as a parseable, non-identity Baby-JubJub point and
    /// stored normalized; unparseable/identity entries are dropped (fail-closed —
    /// they can never match a real PoP key).
    pub fn from_hex_keys<I, S>(keys: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let holders = keys
            .into_iter()
            .filter_map(|h| public_key_from_hex(h.as_ref()).map(|_| normalize_hex(h.as_ref())))
            .collect();
        HolderRegistry { holders }
    }

    /// The registry trusts no holder.
    pub fn is_empty(&self) -> bool {
        self.holders.is_empty()
    }

    /// Whether `holder_hex` (any case, optional `0x`) is an authorised holder.
    fn contains_hex(&self, holder_hex: &str) -> bool {
        self.holders.contains(&normalize_hex(holder_hex))
    }
}

/// The relying party's ENTAILMENT-REGIME policy (sq-314): which entailment regimes
/// it will accept a manifest under. The verifier enforces this fail-closed
/// ([`bind_entailment`]) so `manifest.entailment_regime` is no longer free
/// metadata.
///
/// # Default = `Simple`-only (fail-closed)
/// [`EntailmentPolicy::default`] / [`EntailmentPolicy::simple_only`] accepts ONLY
/// `Simple` (no inference) — the conservative anchor, matching what v1 actually
/// proves cryptographically. A relying party that wants to accept inference opts
/// in explicitly with [`EntailmentPolicy::with_rdfs`] / [`EntailmentPolicy::with_owl`];
/// when it does, the verifier additionally requires the manifest's
/// `derivation_steps` to STRUCTURALLY ground every derived triple (see
/// [`bind_entailment`] + the `derivation` module). A regime the policy does not
/// accept REJECTS.
///
/// # Honest scope
/// Accepting `Rdfs`/`Owl` here means "I accept a derivation re-checked against the
/// disclosed base"; it does NOT (yet) mean an in-circuit closure proof (deferred —
/// see the `derivation` module docs). A relying party that requires
/// cryptographic-strength inference keeps the `Simple`-only default.
// [OPUS-4.8] sq-314: entailment-regime policy (external, fail-closed).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntailmentPolicy {
    accept_rdfs: bool,
    accept_owl: bool,
}

impl Default for EntailmentPolicy {
    fn default() -> Self {
        EntailmentPolicy::simple_only()
    }
}

impl EntailmentPolicy {
    /// Accept ONLY `Simple` (no inference) — the fail-closed default.
    pub fn simple_only() -> Self {
        EntailmentPolicy { accept_rdfs: false, accept_owl: false }
    }

    /// Additionally accept `Rdfs` manifests (with grounded derivation steps).
    pub fn with_rdfs(mut self) -> Self {
        self.accept_rdfs = true;
        self
    }

    /// Additionally accept `Owl` (RDFS-RL/OWL-RL) manifests. `Owl` subsumes RDFS,
    /// so this also accepts `Rdfs`.
    pub fn with_owl(mut self) -> Self {
        self.accept_owl = true;
        self.accept_rdfs = true;
        self
    }

    /// Whether this policy accepts `regime`.
    fn accepts(&self, regime: EntailmentRegime) -> bool {
        match regime {
            EntailmentRegime::Simple => true,
            EntailmentRegime::Rdfs => self.accept_rdfs,
            EntailmentRegime::Owl => self.accept_owl,
        }
    }
}

/// The relying party's revocation/freshness policy (audit #12). The verifier
/// requires every scan-covering credential to carry an issuer-bound status-list
/// reference, that the AUTHORITATIVE status snapshot (resolved by the relying
/// party, NOT taken from the prover — see below) show the credential's status
/// bit UNSET, and that the snapshot's version be within `[min_version, now]`
/// where `now` is the current/latest version the relying party accepts and
/// `now - min_version <= freshness_window` (a snapshot older than the window is
/// STALE and rejected).
///
/// # The authenticated-bits fix (audit #12 re-audit — the load-bearing change)
/// The issuer signature binds only the status-list REFERENCE
/// (`status_ref_digest(H(list IRI), index, version)`), NOT the bit VALUES. So the
/// status BITSTRING is unauthenticated: a prover that presents a genuine
/// issuer-signed reference can attach a FORGED all-zero
/// `manifest.status_snapshots` entry, and reading `snapshot.bit(index)` from THAT
/// would let a REVOKED credential verify — the liveness decision would rest on
/// prover-controlled bytes (the recurring "decision on an unauthenticated
/// prover-supplied input" pattern that bit #3/#8/#9/#4/#12-ref).
///
/// The fix (Option B — mirrors the audit-#3 external-trust-anchor `K` precedent):
/// the AUTHORITATIVE status-list snapshot is an EXTERNAL relying-party input
/// carried HERE, in the policy, exactly as the trusted key-set is external. The
/// verifier reads ITS OWN authoritative snapshot's `bit[index]` for the liveness
/// decision; the prover's `manifest.status_snapshots` is NEVER trusted for the
/// bit. The credential's issuer-signed `(list IRI, index, version)` ties it to a
/// specific list/slot/version; the verifier resolves the snapshot for THAT
/// `(list, version)` from its own store and reads the bit. A relying party
/// populates this store from the status-list credential(s) it fetches + verifies
/// out of band (the real W3C status-list object IS a separately-signed,
/// periodically-updated artifact; resolving it is the relying party's job, just
/// like resolving issuer keys for `K`).
///
/// If the prover ALSO discloses a snapshot for the referenced `(list, version)`,
/// it MUST byte-equal the authoritative one (else REJECT — a disagreeing
/// prover snapshot is a tamper signal); but the bit decision always uses the
/// authoritative bytes regardless. A reference whose `(list, version)` the
/// relying party has NO authoritative snapshot for is REJECTED fail-closed
/// (`StatusSnapshotMissing`): the verifier will not vouch for a liveness view it
/// cannot itself authenticate.
///
/// # Freshness model (version-as-counter)
/// `version` is a monotone counter the issuer increments each time it republishes
/// the status list (a publication sequence number / `validFrom` epoch). The
/// relying party tracks the latest version it has observed (`now`) and how far
/// back it will trust a snapshot (`freshness_window`). A snapshot at version `v`
/// is FRESH iff `min_version <= v <= now`, where `min_version = now.saturating_sub(window)`.
/// This is deliberately simple and transport-agnostic (no wall-clock in the
/// verifier core); a production relying party maps its real status-list refresh
/// cadence onto `(now, freshness_window)`.
///
/// # Fail-closed
/// There is no "no policy" opt-out: [`verify_manifest`] takes a `&RevocationPolicy`
/// mandatorily, and the status check runs unconditionally. A relying party that
/// does not (yet) track versions can use [`RevocationPolicy::accept_version`] to
/// pin the exact version it trusts (window 0), or [`RevocationPolicy::up_to`] for
/// a window. A policy with NO authoritative snapshot for a referenced
/// `(list, version)` rejects (fail-closed). There is no constructor that disables
/// the bit-unset check.
// [OPUS-4.8] audit #12: revocation / freshness policy.
// [OPUS-4.8] audit #12 re-audit (Option B): authoritative snapshot is EXTERNAL —
// the bit decision rests on relying-party-resolved bytes, never the prover's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevocationPolicy {
    /// The latest status-list version the relying party accepts as current.
    now: u64,
    /// How many versions back a snapshot may be and still be fresh.
    freshness_window: u64,
    /// The relying party's AUTHORITATIVE status-list snapshots, keyed by
    /// `(status_list IRI, version)`. The liveness bit decision reads from HERE,
    /// never from the prover's `manifest.status_snapshots`. Populated from the
    /// status-list credential(s) the relying party fetched + authenticated out of
    /// band (the external trust anchor for the bitstring, mirroring `K`).
    // [OPUS-4.8] audit #12 re-audit: external authoritative bitstrings.
    authoritative: std::collections::BTreeMap<(String, u64), StatusListSnapshot>,
    /// OPTIONAL Merkle-tree depth for the HIDDEN-INDEX revocation proof (sq-3e5 /
    /// sq-h2v). When set, the verifier derives the authoritative status-list
    /// Merkle root from its own [`StatusListSnapshot`] at this depth and checks a
    /// `manifest.hidden_revocation` proof's PUBLIC root byte-equals it (so the
    /// holder's index is never disclosed). `None` => no hidden-index proof is
    /// accepted (the relying party only runs the clear-index path). MUST equal the
    /// depth the prover used (the `revoke_unset_d{depth}` member).
    // [OPUS-4.8] sq-3e5 + sq-h2v: authoritative-root derivation depth.
    hidden_index_depth: Option<u32>,
}

impl RevocationPolicy {
    /// Accept snapshots in `[now - window, now]` (a freshness window of `window`
    /// versions ending at `now`). Carries NO authoritative snapshots yet — attach
    /// them with [`Self::with_snapshot`] / [`Self::with_snapshots`]; a referenced
    /// `(list, version)` with no attached authoritative snapshot is rejected
    /// fail-closed.
    pub fn up_to(now: u64, freshness_window: u64) -> Self {
        RevocationPolicy {
            now,
            freshness_window,
            authoritative: std::collections::BTreeMap::new(),
            hidden_index_depth: None,
        }
    }

    /// Accept EXACTLY version `v` (window 0) — the relying party pins the single
    /// status-list version it has resolved and trusts. Attach the authoritative
    /// snapshot(s) with [`Self::with_snapshot`] / [`Self::with_snapshots`].
    pub fn accept_version(v: u64) -> Self {
        RevocationPolicy {
            now: v,
            freshness_window: 0,
            authoritative: std::collections::BTreeMap::new(),
            hidden_index_depth: None,
        }
    }

    /// Enable the HIDDEN-INDEX revocation path (sq-3e5 / sq-h2v) at Merkle depth
    /// `depth` (builder style). The verifier will then derive the authoritative
    /// status-list Merkle root from its own snapshot at this depth and accept a
    /// `manifest.hidden_revocation` proof whose PUBLIC root matches — without the
    /// holder disclosing its index. `depth` MUST equal the `revoke_unset_d{depth}`
    /// member the prover used. The clear-index path is unaffected.
    // [OPUS-4.8] sq-3e5 + sq-h2v: opt-in hidden-index verification depth.
    pub fn with_hidden_index_depth(mut self, depth: u32) -> Self {
        self.hidden_index_depth = Some(depth);
        self
    }

    /// Attach one AUTHORITATIVE status-list snapshot (builder style). The relying
    /// party resolves + authenticates this out of band (a verified W3C status-list
    /// credential); the verifier reads its `bit[index]` for the liveness decision,
    /// NEVER the prover's snapshot. Keyed by `(status_list, version)`; a later
    /// snapshot for the same key replaces an earlier one.
    // [OPUS-4.8] audit #12 re-audit: relying-party authoritative bitstring.
    pub fn with_snapshot(mut self, snapshot: StatusListSnapshot) -> Self {
        self.authoritative
            .insert((snapshot.status_list.clone(), snapshot.version), snapshot);
        self
    }

    /// Attach several authoritative snapshots at once (builder style).
    // [OPUS-4.8] audit #12 re-audit.
    pub fn with_snapshots<I>(mut self, snapshots: I) -> Self
    where
        I: IntoIterator<Item = StatusListSnapshot>,
    {
        for s in snapshots {
            self.authoritative.insert((s.status_list.clone(), s.version), s);
        }
        self
    }

    /// The relying party's authoritative snapshot for `(list, version)`, if it has
    /// resolved one. `None` => the verifier has no authenticated liveness view for
    /// this reference and rejects fail-closed.
    // [OPUS-4.8] audit #12 re-audit: authoritative lookup (never the prover's).
    fn authoritative_snapshot(&self, list: &str, version: u64) -> Option<&StatusListSnapshot> {
        self.authoritative.get(&(list.to_string(), version))
    }

    /// The hidden-index Merkle depth, if the relying party enabled the
    /// hidden-index revocation path. `None` => the path is disabled (a
    /// `manifest.hidden_revocation` is then not accepted as the liveness check).
    // [OPUS-4.8] sq-3e5 + sq-h2v.
    fn hidden_index_depth(&self) -> Option<u32> {
        self.hidden_index_depth
    }

    /// The oldest version still considered fresh.
    fn min_version(&self) -> u64 {
        self.now.saturating_sub(self.freshness_window)
    }

    /// Whether `v` is within the freshness window `[min_version, now]`.
    fn is_fresh(&self, v: u64) -> bool {
        v >= self.min_version() && v <= self.now
    }
}

/// A verifier-issued freshness nonce (audit #4) — a fresh BN254 field element the
/// relying party mints out of band and hands to the prover BEFORE proving. The
/// prover MUST incorporate it as the circuit's `challenge` public input (field 0)
/// at prove time; [`verify_manifest`] then reconstructs the public-input vector
/// using THIS nonce (never `manifest.binding`), so the existing audit-#1
/// byte-compare rejects any proof whose committed challenge ≠ the verifier's
/// nonce. A captured manifest re-presented under a fresh nonce therefore fails
/// the byte-compare (replay defence), and the [`SeenNonces`] store below rejects
/// a nonce presented twice (single-use defence).
///
/// Stored as a normalized field element so `0x`-padding differences cannot make
/// two presentations of the same nonce look distinct to the single-use store.
// [OPUS-4.8] audit #4: verifier-issued freshness nonce.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifierNonce(Fr);

impl VerifierNonce {
    /// Adopt a caller-chosen field element as the nonce (e.g. from an external
    /// CSPRNG mapped into the field, or a fresh per-session value). The relying
    /// party is responsible for unpredictability/uniqueness; this type only
    /// guarantees a canonical field representation for the binding + single-use
    /// machinery. `None` if `hex` is not a parseable field element.
    pub fn from_hex(hex: &str) -> Option<Self> {
        field_from_hex_str(hex).map(VerifierNonce)
    }

    /// Adopt a field element directly (e.g. one drawn by the relying party's RNG
    /// and reduced into the field).
    pub fn from_field(f: Fr) -> Self {
        VerifierNonce(f)
    }

    /// The nonce as the `FieldHex` the circuit challenge / reconstruction uses.
    pub fn as_field_hex(&self) -> FieldHex {
        FieldHex(field_to_hex(&self.0))
    }

    /// Canonical hex key for the single-use store (representation-insensitive).
    fn canonical_key(&self) -> String {
        field_to_hex(&self.0)
    }
}

/// Single-use nonce store (audit #4): records every verifier nonce a manifest has
/// already been accepted (or attempted) under, so a captured (nonce, manifest)
/// pair cannot be replayed. [`verify_manifest`] calls [`SeenNonces::record_fresh`]
/// BEFORE the cryptographic gate; a nonce already present REJECTS
/// ([`CheckError::NonceReplay`]).
///
/// # Fail-closed contract
/// `record_fresh` must (a) return `false` (already-seen) if the nonce was ever
/// previously recorded, and (b) atomically mark it recorded otherwise. The store
/// is consulted on EVERY `verify_manifest` call with NO opt-out — there is no
/// "skip when the store is absent" path (the parameter is mandatory), so the
/// single-use property cannot be bypassed by omitting a field.
///
/// # Persistence — pick the right impl, durability is NOT optional in production
/// This crate ships TWO implementations and the choice is load-bearing for the
/// audit-#4 single-use guarantee:
///
/// - [`FileSeenNonces`] (audit #4 durability fix, sq-aih) — DURABLE. Backed by an
///   append-only file with an `fsync` + an advisory `flock(LOCK_EX)` around every
///   check-and-append, so a recorded nonce SURVIVES A RESTART and the
///   check-and-insert is atomic across concurrent processes sharing the path. THIS
///   is what a relying party that can be restarted (i.e. every real deployment)
///   MUST use. See its docs for the exact durability/atomicity contract and the
///   honest "reference impl vs. a DB UNIQUE constraint" caveats.
/// - [`InMemorySeenNonces`] — NON-DURABLE, TEST / single-session ONLY. Process-local:
///   it enforces single-use only WITHIN one verifier process. A replayed proof is
///   accepted again after a restart because the seen-set is lost. It is deliberately
///   labelled test-only and MUST NOT be used by a relying party that can restart.
///
/// The trait boundary exists so the persistence choice is pluggable; a relying
/// party with an existing datastore can also back it with a database row carrying a
/// UNIQUE constraint on the nonce or a KV store with compare-and-set — either gives
/// the same restart-surviving, atomic semantics as [`FileSeenNonces`].
// [OPUS-4.8] audit #4: single-use nonce store. sq-aih: durability is mandatory —
// FileSeenNonces is the durable impl; InMemory is test-only.
pub trait SeenNonces {
    /// Record `nonce` as used and return `true` iff it was FRESH (not previously
    /// recorded). Returns `false` if the nonce was already seen — the verifier
    /// then rejects the manifest as a replay. Implementations MUST be atomic
    /// (check-and-insert) so concurrent verifiers cannot both observe the same
    /// nonce as fresh.
    fn record_fresh(&self, nonce: &VerifierNonce) -> bool;
}

/// Process-local, thread-safe [`SeenNonces`] (audit #4) — **NON-DURABLE, TEST /
/// single-session ONLY.**
///
/// # ⚠️ NOT durable across restarts — DO NOT use in a restartable relying party.
/// This enforces single-use only WITHIN one verifier process: the seen-set lives
/// in memory and is LOST on restart, so a captured (nonce, manifest) pair that was
/// already rejected once is ACCEPTED AGAIN after the process restarts. That defeats
/// the audit-#4 single-use guarantee for any deployment that can be restarted (i.e.
/// every real one). It exists for unit tests and ephemeral single-session tooling
/// only. A relying party MUST use [`FileSeenNonces`] (durable: survives restart, is
/// the sq-aih fix) or another durable backing (a DB row with a UNIQUE constraint /
/// a KV store with compare-and-set).
// [OPUS-4.8] audit #4 / sq-aih: in-memory store is NON-DURABLE, test-only. The
// durable impl is FileSeenNonces — single-use survives a restart there.
#[derive(Debug, Default)]
pub struct InMemorySeenNonces {
    seen: std::sync::Mutex<BTreeSet<String>>,
}

impl InMemorySeenNonces {
    pub fn new() -> Self {
        InMemorySeenNonces { seen: std::sync::Mutex::new(BTreeSet::new()) }
    }
}

impl SeenNonces for InMemorySeenNonces {
    fn record_fresh(&self, nonce: &VerifierNonce) -> bool {
        // `.insert` returns true iff the value was NOT already present — exactly
        // the "was fresh" semantics. A poisoned lock fails closed (treat as
        // already-seen / reject) rather than panicking on prover-triggerable
        // input: a poisoned mutex means another verify panicked mid-record, and
        // we must not optimistically accept a possibly-replayed nonce.
        match self.seen.lock() {
            Ok(mut set) => set.insert(nonce.canonical_key()),
            Err(_) => false,
        }
    }
}

/// DURABLE, restart-surviving, cross-process single-use [`SeenNonces`] — the
/// audit-#4 durability fix (sq-aih). Backed by an APPEND-ONLY file of canonical
/// nonce keys (one per line). Single-use SURVIVES A RESTART: the recorded set is
/// the file's contents, so reopening the same path re-loads every previously-seen
/// nonce and a replayed proof is still rejected.
///
/// # Atomicity / concurrency contract
/// Every [`record_fresh`](SeenNonces::record_fresh) does, under an advisory
/// exclusive `flock(LOCK_EX)` held on the file fd for the whole operation:
/// 1. seek to start, read the WHOLE file, parse it into the seen-set (the file —
///    not an in-RAM cache — is the source of truth, so a concurrent process that
///    appended between calls is observed);
/// 2. if the nonce is already present, release the lock and return `false`
///    (replay);
/// 3. otherwise append `key\n`, `flush` + `fsync` the file (durably on disk before
///    we report fresh), release the lock, return `true`.
///
/// `flock(LOCK_EX)` serialises step 1–3 across ALL processes that opened the SAME
/// path on the SAME machine, so two concurrent verifiers cannot both observe one
/// nonce as fresh (the check-and-append is atomic). An in-process [`Mutex`] also
/// guards the fd so threads in one process can't interleave the seek/read/append.
///
/// # Honest scope — reference impl, not a clustered DB
/// This is a single-file, single-host reference implementation chosen to give a
/// real restart-surviving guarantee with NO heavy dependency (just `libc::flock`):
/// - `flock` is ADVISORY and per-host. It does NOT coordinate across machines /
///   NFS mounts that don't honour `flock` / networked filesystems. A multi-HOST
///   relying party MUST use a shared transactional store instead (a DB row with a
///   `UNIQUE` constraint on the nonce, or a KV store with compare-and-set) —
///   exactly what the [`SeenNonces`] trait boundary is for.
/// - The file GROWS unbounded (one line per nonce ever seen). For a high-volume
///   deployment, rotate/compact behind the same path or use a DB. The set is also
///   re-read from disk on every call (O(file) per verify) — fine for a reference /
///   moderate-volume relying party, replace with an indexed store at scale.
/// - A poisoned in-process mutex, an I/O error, a lock failure, or a malformed line
///   all FAIL CLOSED (return `false` = treat as already-seen / reject), never
///   accept-on-error and never panic on prover-triggerable input.
// [OPUS-4.8] sq-aih (audit #4 durability): durable append-only + flock single-use
// store. Single-use survives a restart. Reference impl — see scope caveats above.
#[derive(Debug)]
pub struct FileSeenNonces {
    /// The append-only nonce log. Held open for the lifetime of the store so the
    /// advisory `flock` and the appends target a stable fd. Guarded by a process-
    /// local mutex so threads within one process serialise too.
    file: std::sync::Mutex<std::fs::File>,
}

impl FileSeenNonces {
    /// Open (creating if absent) the durable nonce log at `path`. Any nonces
    /// already recorded there are immediately in force — reopening the same path
    /// after a restart re-loads them, so single-use survives the restart.
    ///
    /// Returns an `io::Error` if the file cannot be opened/created; a relying party
    /// MUST treat that as fatal (it has no durable single-use store and must not
    /// fall back to the non-durable in-memory one silently).
    pub fn open<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(path.as_ref())?;
        Ok(FileSeenNonces { file: std::sync::Mutex::new(file) })
    }

    /// Whether `key` is already present in the (locked) file. Reads the WHOLE file
    /// from the start so a concurrent appender is observed. `Err` on an I/O failure
    /// (the caller fails closed). The fd MUST already hold the advisory lock.
    fn contains_key_locked(file: &mut std::fs::File, key: &str) -> std::io::Result<bool> {
        use std::io::{Read, Seek, SeekFrom};
        file.seek(SeekFrom::Start(0))?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        // Each non-empty line is a canonical nonce key. Trailing/embedded blank
        // lines (e.g. a partial write) are ignored — they can never equal a
        // canonical key, so ignoring them cannot mask a real replay.
        Ok(buf.lines().any(|line| line == key))
    }
}

impl SeenNonces for FileSeenNonces {
    fn record_fresh(&self, nonce: &VerifierNonce) -> bool {
        use std::io::Write;
        use std::os::unix::io::AsRawFd;

        // Serialise threads in THIS process; cross-process serialisation is the
        // `flock` below. A poisoned mutex fails closed (another verify panicked
        // mid-record — do not optimistically accept a possibly-replayed nonce).
        let mut file = match self.file.lock() {
            Ok(f) => f,
            Err(_) => return false,
        };

        let fd = file.as_raw_fd();
        // Advisory EXCLUSIVE lock for the whole check-and-append: this is what
        // makes the operation atomic ACROSS PROCESSES sharing the path. Blocks
        // until acquired; an error fails closed.
        // SAFETY: `fd` is a valid open file descriptor owned by `file` for the
        // duration of this call (the MutexGuard keeps `file` alive).
        if unsafe { libc::flock(fd, libc::LOCK_EX) } != 0 {
            return false;
        }

        // Unlock helper — runs on every return path below so we never leak the
        // advisory lock to the next caller (a leaked lock would deadlock).
        let unlock = |fd: i32| {
            // SAFETY: same valid, locked fd; LOCK_UN releases our advisory lock.
            let _ = unsafe { libc::flock(fd, libc::LOCK_UN) };
        };

        let key = nonce.canonical_key();

        // Step 1: is it already recorded (in the durable file)? Fail closed on I/O
        // error — we must not report a nonce fresh if we couldn't read the log.
        match Self::contains_key_locked(&mut file, &key) {
            Ok(true) => {
                unlock(fd);
                return false; // replay
            }
            Ok(false) => {}
            Err(_) => {
                unlock(fd);
                return false; // fail closed
            }
        }

        // Step 2: durably append the new key, then fsync BEFORE reporting fresh —
        // so a crash after we return `true` cannot lose the record (which would
        // re-open the replay window). The file was opened in append mode, so the
        // write lands at EOF regardless of the seek done by the read above.
        let mut line = key;
        line.push('\n');
        if file.write_all(line.as_bytes()).is_err()
            || file.flush().is_err()
            || file.sync_all().is_err()
        {
            // The append may be partially written; a partial (non-newline-
            // terminated) line is ignored on the next read, and we report
            // not-fresh so this presentation is rejected fail-closed. The nonce is
            // NOT durably recorded, but rejecting here is the safe (non-accepting)
            // direction.
            unlock(fd);
            return false;
        }

        unlock(fd);
        true
    }
}

/// Why a manifest was rejected.
#[derive(Debug)]
pub enum CheckError {
    /// The sparq-zk layer-3 re-check failed (parse / arity / bnode obligation).
    Sparqzk(VerifyError),
    /// A sub-proof's declared circuit id does not match the id re-derived from
    /// its public inputs.
    CircuitIdMismatch { proof: usize, declared: CircuitId, derived: Option<CircuitId> },
    /// A binding edge references a non-existent proof / row / slot.
    DanglingEdge { edge: usize },
    /// A binding edge connects proofs whose kinds cannot be bound (e.g. the
    /// source is not a scan or the target is not a filter).
    EdgeKindMismatch { edge: usize },
    /// A binding edge's encodings disagree: the scanned column does not equal
    /// the filter's operand (the join the prover claimed is a lie).
    BindingInconsistent { edge: usize },
    /// bb rejected a sub-proof.
    ProofRejected { proof: usize },
    /// A sub-proof carried no bb proof bytes (structure-only manifest passed to
    /// the full verifier).
    MissingProof { proof: usize },
    /// A sub-proof's `proof_hex` blob is malformed (non-hex, odd length, a
    /// truncated/oversized length prefix) — rejected before any bb call rather
    /// than panicking (audit hardening: route prover-controlled bytes through
    /// the REJECT channel).
    MalformedProof { proof: usize },
    /// A declared `ProofInputs` field is not a parseable BN254 field element, so
    /// the public-input vector cannot be reconstructed (audit #1).
    MalformedField { proof: usize, what: &'static str },
    /// The public-input vector reconstructed from the declared `ProofInputs`
    /// (audit #1) does not byte-match the prover's `public_inputs` blob: the
    /// JSON statement and the cryptographic proof describe different statements.
    PublicInputMismatch { proof: usize },
    /// A query BGP pattern's constant slots have no scan sub-proof whose bound
    /// `pattern_is_const`/`pattern_const_enc` match them (audit #10): the
    /// disclosed scan does not actually answer the query's pattern (e.g. an age
    /// scan presented under a salary query — constant-swap).
    UnboundPattern { pattern: usize },
    /// A query FILTER has no bound `filter_int` sub-proof matching its operator
    /// and constant, reachable via a binding edge from the scan slot the FILTER
    /// variable binds to, with a true verdict (audit #5/#6/#10): the FILTER is
    /// unproven (FILTER-add, comparison-substitution, wrong-operand-slot, or an
    /// `expected==false` row presented as passing).
    UnboundFilter { variable: String },
    /// A query FILTER constrains a variable that does not bind to any scanned
    /// column of a BGP pattern (cannot be mapped to a `filter_int` operand).
    UnmappableFilterVar { variable: String },
    /// A scan sub-proof's commitment has no issuer attestation in the manifest
    /// (audit #3): `commitments[g]` is unsigned / prover-invented, so the
    /// "credential issued by X" claim has no cryptographic backing. Closes the
    /// unsigned-commitment forgery and the drop-a-triple-and-recommit
    /// suppression (a truncated-leaf recommit yields a different `C(G)` with no
    /// valid attestation).
    // [OPUS-4.8] audit #3.
    UnattestedCommitment { proof: usize, commitment: String },
    /// An attestation's signature did not verify under its declared issuer key
    /// (audit #3): forged/tampered signature, or the signature is over a
    /// different commitment than declared.
    // [OPUS-4.8] audit #3.
    InvalidIssuerSignature { commitment: String },
    /// An attestation's issuer key is not a member of the EXTERNAL trusted
    /// key-set `K` (audit #3, codex #1): a signature by a key outside the
    /// relying-party-supplied trusted set is rejected. The trust anchor is the
    /// verifier's [`KeySet`] argument, NOT the prover's `manifest.key_set`.
    // [OPUS-4.8] audit #3 / codex #1.
    IssuerKeyNotInKeySet { commitment: String },
    /// The prover's `manifest.key_set` lists a key that is NOT in the external
    /// trusted key-set `K` (audit #3 codex #1): the prover tried to widen the
    /// trust anchor with a key the relying party never trusted. The manifest's
    /// declared key-set must be a SUBSET of the external `K`; a superset is
    /// rejected so the accept decision can never depend on a prover-chosen key.
    // [OPUS-4.8] audit #3 / codex #1.
    UntrustedDeclaredKey { key: String },
    /// The prover DECLARED a narrowed `manifest.key_set` (non-empty) but an
    /// accepted attestation's issuer key is not a member of it (audit #3 codex
    /// 2216 LOW): the declared narrowed set is inconsistent with the
    /// attestations actually proven. The accept decision stays anchored on the
    /// external trusted `K` (an attestation key is ALWAYS required to be in `K`);
    /// this additionally enforces internal consistency of a declared narrowing,
    /// so a prover cannot advertise a tighter issuer set than it actually used.
    // [OPUS-4.8] codex 2216 LOW.
    AttestationKeyNotInDeclaredSet { commitment: String, key: String },
    /// A query BGP pattern's declared `manifest.attributions[pattern]` does not
    /// cover the PROOF-BOUND set of graphs the answering scan sub-proof's
    /// in-circuit attribution shows it actually drew matched triples from (audit
    /// #8): the prover under-declared a contributing graph to collapse a genuine
    /// cross-graph join and drop its non-bnode obligation (the `[[0],[0]]`
    /// forge). `proof_graph` is a graph index the scan proved a contribution
    /// from that `manifest.attributions[pattern]` omits.
    // [OPUS-4.8] audit #8.
    AttributionUnderDeclared { pattern: usize, proof_graph: usize },
    /// A query BGP pattern matched no scan sub-proof carrying a proof-bound
    /// attribution to cross-check (audit #8): every BGP pattern must be answered
    /// by a scan (already enforced by stage 2b `UnboundPattern`), so this is the
    /// belt-and-braces fail-closed if attribution binding cannot find the
    /// answering scan.
    // [OPUS-4.8] audit #8.
    AttributionUnbound { pattern: usize },
    /// Two DISTINCT committed graphs disclosed the SAME per-graph bnode salt
    /// (audit #9): a reused salt makes a same-label canonical bnode encode
    /// identically across both graphs — the Q6 cross-graph correlation handle.
    /// Each graph must be committed under a globally-unique issuer-attested salt.
    // [OPUS-4.8] audit #9.
    SaltReused { salt: String },
    /// The attestation covering a scan sub-proof's commitment carries NO salt
    /// (`salt: None`) — fail-closed for audit #9 / codex 2221 HIGH. A salt-less
    /// (legacy) attestation does NOT bind the per-graph RDFC10 salt into the
    /// issuer signature and does NOT participate in the salt-uniqueness check, so
    /// accepting one for a scan-covering commitment would silently bypass the #9
    /// salt-separation guarantee (a salt-reusing ingester just omits the salt
    /// field). Every attestation that covers a verified scan commitment MUST carry
    /// a salt and verify via the salt-bound `commitment_message_with_salt` path.
    // [OPUS-4.8] codex 2221 HIGH: salt-bound attestation is mandatory for scans.
    ScanCommitmentSaltMissing { proof: usize, commitment: String },
    /// A scan sub-proof's `attribution` vector is absent/empty or the wrong
    /// length (codex 2221 MEDIUM, fail-closed for audit #8). The per-graph source
    /// attribution is a security-relevant, proof-bound quantity (`scan.nr` step 4)
    /// that the cross-graph obligation gate (stage 2e) cross-checks against
    /// `manifest.attributions`. `#[serde(default)]` lets a prover OMIT it (empty
    /// vec) or under-length it, and `bind_attributions` only checks the bits
    /// present — so an omitted attribution makes the #8 cross-check vacuous,
    /// resurrecting the `[[0],[0]]` collapse forge. It MUST be present and EXACTLY
    /// `CircuitId.k` bits (no default/pad-to-false). `expected` = k.
    // [OPUS-4.8] codex 2221 MEDIUM: attribution must be present + exactly k bits.
    AttributionMalformed { proof: usize, expected: usize, got: usize },
    /// The verifier-issued nonce has already been seen by the single-use store
    /// (audit #4): a captured (nonce, manifest) pair re-presented to the SAME
    /// verifier session. Rejected before the cryptographic gate so a bearer proof
    /// cannot be replayed. The honest flow uses each verifier nonce exactly once.
    // [OPUS-4.8] audit #4: single-use.
    NonceReplay,
    /// The manifest's declared `binding` challenge does not equal the
    /// verifier-issued nonce (audit #4). The proof's committed challenge (field 0)
    /// is byte-bound to the nonce by the audit-#1 reconstruction; this additional
    /// check fails closed when the JSON `binding` advertises a DIFFERENT challenge
    /// than the verifier issued (a manifest minted for a different session/nonce).
    /// A consistent honest manifest sets `binding.challenge == nonce`.
    // [OPUS-4.8] audit #4: nonce/binding consistency, fail-closed.
    NonceBindingMismatch,
    /// A scan-covering attestation carries NO issuer-bound status reference
    /// (`status: None`) — fail-closed for audit #12. A status-unbound attestation
    /// does NOT bind the credential's revocation reference into the issuer
    /// signature, so accepting one for a scan-covering commitment would let a
    /// prover OMIT the status reference and present a revoked credential
    /// unchecked. Every attestation covering a verified scan commitment MUST bind
    /// a status reference and verify via the status-bound message.
    // [OPUS-4.8] audit #12: status-bound attestation is mandatory for scans.
    ScanCommitmentStatusMissing { proof: usize, commitment: String },
    /// The manifest carries an issuer-bound status reference for a scan
    /// commitment but NO `manifest.revocation` to recompute the signed digest
    /// from (audit #12): the prover dropped the disclosed reference. The
    /// status-bound issuer signature cannot be checked without it, so this is
    /// rejected (fail-closed — the omit-the-field bypass).
    // [OPUS-4.8] audit #12.
    RevocationReferenceMissing { proof: usize },
    /// The disclosed `manifest.revocation` (list id / index / version) does not
    /// match the issuer-signed status reference for a scan commitment (audit
    /// #12): the prover disclosed a different reference than the issuer signed
    /// (e.g. pointing at another list/index/version whose bit is unset). Detected
    /// because the recomputed status digest then fails the signature check, OR
    /// because the disclosed index/version differs from the attestation's signed
    /// `AttestedStatusRef`.
    // [OPUS-4.8] audit #12.
    RevocationReferenceMismatch { commitment: String },
    /// [OPUS-4.8] sq-ayv: the status reference's index-disclosure MODE is malformed
    /// — neither a clear `index` nor an `index_commitment` is present, or BOTH are
    /// (the attestation's signed reference and the disclosed reference must each be
    /// EXACTLY one of clear-index or committed-index, and the two must agree on the
    /// mode). Rejected fail-closed: an ambiguous reference mode could let a prover
    /// recompute the signed digest one way while the liveness check reads the other.
    RevocationReferenceModeInvalid { commitment: String },
    /// [OPUS-4.8] sq-ayv: a COMMITTED-index revocation reference (clear index
    /// withheld) was disclosed but NO `manifest.hidden_revocation` proof is present
    /// to check liveness against the authoritative root. The committed-index path
    /// MOVES the liveness decision to the hidden-index proof; without it the
    /// credential's liveness is unchecked, so this is rejected fail-closed
    /// (revocation is NEVER skipped — either the clear-index bit check or the
    /// hidden-index proof must run).
    HiddenRevocationRequired { proof: usize },
    /// [OPUS-4.8] sq-ayv: a `manifest.hidden_revocation` proof's PUBLIC index
    /// commitment does NOT equal the ISSUER-SIGNED index commitment in
    /// `manifest.revocation.index_commitment` (the cross-binding): the index proven
    /// unset is not provably the index the issuer committed to. Rejected — a holder
    /// could otherwise sign over a commitment to its REVOKED index and prove
    /// bit-unset for a different (active) index.
    HiddenRevocationIndexCommitmentMismatch,
    /// The relying party has NO AUTHORITATIVE status-list snapshot for the
    /// credential's (issuer-bound) revocation reference `(status_list, version)`
    /// (audit #12 / re-audit Option B): the verifier cannot AUTHENTICATE the
    /// credential's liveness view, so it rejects (fail-closed). The bit decision
    /// is sourced from the relying party's [`RevocationPolicy`] authoritative
    /// store — NOT the prover's `manifest.status_snapshots` — so an absent
    /// authoritative snapshot is the verifier's own missing trust input, never a
    /// prover-skippable field.
    // [OPUS-4.8] audit #12 / re-audit: missing AUTHORITATIVE (relying-party) snapshot.
    StatusSnapshotMissing { status_list: String, version: u64 },
    /// The prover disclosed a status-list snapshot for the referenced
    /// `(status_list, version)` whose bits DISAGREE with the relying party's
    /// AUTHORITATIVE snapshot (audit #12 re-audit, Option B): a tamper signal. The
    /// liveness bit decision does NOT depend on the prover's snapshot (it always
    /// reads the authoritative bytes), but a disagreeing prover snapshot — e.g. a
    /// FORGED all-zero bitstring presented alongside a genuine reference for a
    /// REVOKED credential — is surfaced explicitly as a forgery attempt rather
    /// than silently ignored.
    // [OPUS-4.8] audit #12 re-audit: prover snapshot ≠ authoritative snapshot.
    StatusSnapshotTampered { status_list: String, version: u64 },
    /// The credential's status bit is SET in the disclosed snapshot (audit #12):
    /// the credential is REVOKED / SUSPENDED. Rejected.
    // [OPUS-4.8] audit #12.
    CredentialRevoked { status_list: String, index: u64 },
    /// The disclosed status-list snapshot's version is OUTSIDE the verifier's
    /// freshness window (audit #12): the snapshot is STALE (or implausibly
    /// future-dated). A relying party will not trust a revocation view it cannot
    /// vouch is current, so a stale snapshot is rejected (a revoked-since-snapshot
    /// credential must not slip through on an old "active" view).
    // [OPUS-4.8] audit #12.
    StatusListStale { status_list: String, version: u64 },
    /// A `manifest.hidden_revocation` proof was presented but the relying party
    /// has NOT enabled the hidden-index path (no `hidden_index_depth` in the
    /// policy) (sq-3e5 / sq-h2v): the verifier cannot derive an authoritative
    /// Merkle root to bind the proof to, so it rejects fail-closed rather than
    /// accepting a root it did not itself compute.
    // [OPUS-4.8] sq-3e5 + sq-h2v.
    HiddenRevocationNotEnabled,
    /// The `manifest.hidden_revocation` declared a Merkle depth that does NOT
    /// match the relying party's policy depth (sq-3e5 / sq-h2v): the trees (and
    /// roots) would be over different leaf layouts, so the proof cannot be bound
    /// to the verifier's authoritative root. Rejected.
    // [OPUS-4.8] sq-3e5 + sq-h2v.
    HiddenRevocationDepthMismatch { declared: u32, policy: u32 },
    /// The relying party could not derive an authoritative Merkle root for the
    /// hidden-index revocation proof (sq-3e5 / sq-h2v): there is no authoritative
    /// snapshot for the referenced `(list, version)`, or the depth is implausible.
    /// Fail-closed — the verifier will not vouch for a liveness view it cannot
    /// itself anchor.
    // [OPUS-4.8] sq-3e5 + sq-h2v.
    HiddenRevocationRootUnavailable { status_list: String, version: u64 },
    /// The `manifest.hidden_revocation` proof's PUBLIC Merkle root does NOT equal
    /// the root the relying party derived from its OWN authoritative snapshot
    /// (sq-3e5 / sq-h2v): the proof was produced against a different (e.g.
    /// prover-forged all-zero) status list. Rejected — the liveness fact is not
    /// bound to the relying party's authenticated status data.
    // [OPUS-4.8] sq-3e5 + sq-h2v.
    HiddenRevocationRootMismatch,
    /// bb rejected the hidden-index revocation proof (sq-3e5 / sq-h2v): the
    /// zero-knowledge bit-unset/inclusion statement did not verify against the
    /// canonical `revoke_unset_d{depth}` vk and the reconstructed public inputs.
    // [OPUS-4.8] sq-3e5 + sq-h2v.
    HiddenRevocationProofRejected,
    /// The `manifest.hidden_revocation` proof blob is malformed (non-hex /
    /// truncated length prefix) (sq-3e5 / sq-h2v) — rejected before any bb call
    /// (audit hardening; prover-controlled bytes never panic).
    // [OPUS-4.8] sq-3e5 + sq-h2v.
    HiddenRevocationMalformedProof,
    /// A `manifest.hidden_issuer_attestations` entry was presented but the relying
    /// party has NOT enabled the hidden-issuer path (no `hidden_issuer_depth` in
    /// the KeySet) (sq-z9l): the verifier cannot derive an authoritative key-set
    /// Merkle root to bind the proof to, so it rejects fail-closed.
    // [OPUS-4.8] sq-z9l.
    HiddenIssuerNotEnabled,
    /// A `manifest.hidden_issuer_attestations` entry declared a Merkle depth that
    /// does NOT match the KeySet policy depth (sq-z9l): the trees (and roots) would
    /// be over different layouts, so the proof cannot be bound to the verifier's
    /// authoritative root. Rejected.
    // [OPUS-4.8] sq-z9l.
    HiddenIssuerDepthMismatch { declared: u32, policy: u32 },
    /// The relying party could not derive an authoritative key-set Merkle root for
    /// the hidden-issuer proof (sq-z9l): the trusted set overflows the tree at this
    /// depth, or the depth is implausible. Fail-closed.
    // [OPUS-4.8] sq-z9l.
    HiddenIssuerRootUnavailable,
    /// The `manifest.hidden_issuer_attestations` proof's PUBLIC key-set Merkle root
    /// does NOT equal the root the relying party derived from its OWN authoritative
    /// KeySet (sq-z9l): the proof was produced against a different (e.g.
    /// prover-chosen) key set. Rejected — the "in K" fact is not anchored on the
    /// relying party's trust.
    // [OPUS-4.8] sq-z9l.
    HiddenIssuerRootMismatch,
    /// The `manifest.hidden_issuer_attestations` proof's PUBLIC message `m` does
    /// NOT equal the issuer-signed message the verifier recomputed from the
    /// disclosed commitment + salt + status reference (sq-z9l): the hidden-issuer
    /// proof is not bound to this committed graph. Rejected.
    // [OPUS-4.8] sq-z9l.
    HiddenIssuerMessageMismatch { commitment: String },
    /// A `manifest.hidden_issuer_attestations` entry covers a commitment that no
    /// verified scan sub-proof references (sq-z9l): a dangling hidden attestation.
    /// Rejected (every hidden attestation must cover a scan-referenced commitment,
    /// mirroring the clear-key path's UnattestedCommitment discipline in reverse).
    // [OPUS-4.8] sq-z9l.
    HiddenIssuerUnreferencedCommitment { commitment: String },
    /// `bb verify` REJECTED the hidden-issuer proof against the canonical
    /// `hidden_issuer_d{depth}` vk + reconstructed public inputs (sq-z9l): the
    /// signature is invalid, the key is not in K, or the public inputs were
    /// tampered. Rejected.
    // [OPUS-4.8] sq-z9l.
    HiddenIssuerProofRejected,
    /// The `manifest.hidden_issuer_attestations` proof blob is malformed (non-hex /
    /// truncated length prefix) (sq-z9l) — rejected before any bb call.
    // [OPUS-4.8] sq-z9l.
    HiddenIssuerMalformedProof,
    /// The manifest's binding is `HolderPop` but the relying party supplied NO
    /// holder registry (an empty [`HolderRegistry`]) (sq-cwq): the verifier has no
    /// trust anchor to check the holder key against, so it cannot accept a holder
    /// PoP. Rejected fail-closed — a HolderPop binding without a registry is NEVER
    /// silently accepted as a bare challenge (the previous placeholder behaviour).
    // [OPUS-4.8] sq-cwq.
    HolderRegistryEmpty,
    /// The `HolderPop` binding's `holder` key is not a member of the relying
    /// party's external [`HolderRegistry`] (sq-cwq): the presenter is not an
    /// authorised holder. Rejected.
    // [OPUS-4.8] sq-cwq.
    HolderNotTrusted { holder: String },
    /// The `HolderPop` binding's `cryptosuite` is unknown/unsupported, or its
    /// `holder` key / `pop` signature did not parse (sq-cwq): the PoP is
    /// unverifiable. Rejected fail-closed (prover-controlled bytes never panic).
    // [OPUS-4.8] sq-cwq.
    HolderPopMalformed,
    /// The `HolderPop` binding's `pop` signature did not verify under the `holder`
    /// key over the challenge-bound PoP message (sq-cwq): the presenter did NOT
    /// prove possession of the holder secret (a forged/replayed/absent PoP).
    /// Rejected.
    // [OPUS-4.8] sq-cwq.
    HolderPopInvalid { holder: String },
    /// The manifest's `entailment_regime` is NOT accepted by the relying party's
    /// [`EntailmentPolicy`] (sq-314): e.g. an `Rdfs`/`Owl` manifest under a
    /// `Simple`-only policy. Rejected fail-closed — the regime is enforced, not
    /// free metadata (a relying party opts into inference explicitly).
    // [OPUS-4.8] sq-314.
    EntailmentRegimeNotAccepted { regime: &'static str },
    /// A `Simple` manifest carried derivation steps (sq-314): `Simple` means NO
    /// inference, so any `derivation_steps` are inconsistent with the declared
    /// regime. Rejected.
    // [OPUS-4.8] sq-314.
    UnexpectedDerivationSteps,
    /// A non-`Simple` manifest carried NO derivation steps (sq-314): an inference
    /// regime with nothing to justify the inference is a vacuous claim. Rejected
    /// fail-closed (a relying party that accepts `Rdfs`/`Owl` requires the
    /// derivation to be recorded + re-checkable).
    // [OPUS-4.8] sq-314.
    MissingDerivationSteps { regime: &'static str },
    /// A derivation step is not a well-formed instance of its rule, or its rule is
    /// not admitted by the declared regime (sq-314): the recorded inference does
    /// not match the rule shape / regime. Rejected.
    // [OPUS-4.8] sq-314.
    MalformedDerivationStep { step: usize },
    /// A derivation step's antecedent is UNGROUNDED (sq-314): it is neither an
    /// earlier step's derived triple nor a triple disclosed by a scan sub-proof
    /// (the asserted base). A derived triple cannot rest on an antecedent the proof
    /// does not establish, so this is rejected fail-closed. (The in-circuit closure
    /// proof that would ground antecedents not disclosed is deferred — see the
    /// `derivation` module; until then only the disclosed base grounds a step.)
    // [OPUS-4.8] sq-314.
    UngroundedDerivationAntecedent { step: usize, antecedent: usize },
    /// Subprocess / io failure (not a verification verdict).
    Driver(DriverError),
}

impl std::fmt::Display for CheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckError::Sparqzk(e) => write!(f, "sparq-zk re-check failed: {e}"),
            CheckError::CircuitIdMismatch { proof, declared, derived } => write!(
                f,
                "sub-proof {proof}: declared circuit id {declared:?} but inputs derive {derived:?}"
            ),
            CheckError::DanglingEdge { edge } => {
                write!(f, "binding edge {edge} references a missing proof/row/slot")
            }
            CheckError::EdgeKindMismatch { edge } => {
                write!(f, "binding edge {edge} connects incompatible proof kinds")
            }
            CheckError::BindingInconsistent { edge } => write!(
                f,
                "binding edge {edge}: scanned column does not equal the filter operand"
            ),
            CheckError::ProofRejected { proof } => write!(f, "bb rejected sub-proof {proof}"),
            CheckError::MissingProof { proof } => {
                write!(f, "sub-proof {proof} has no proof bytes")
            }
            CheckError::MalformedProof { proof } => {
                write!(f, "sub-proof {proof} proof_hex blob is malformed")
            }
            CheckError::MalformedField { proof, what } => {
                write!(f, "sub-proof {proof}: declared field `{what}` is not a valid field element")
            }
            CheckError::PublicInputMismatch { proof } => write!(
                f,
                "sub-proof {proof}: reconstructed public inputs do not match the proof's public inputs (declared statement is not the proved statement)"
            ),
            CheckError::UnboundPattern { pattern } => write!(
                f,
                "query BGP pattern {pattern} has no scan sub-proof binding its constant slots (constant-swap / unproven pattern)"
            ),
            CheckError::UnboundFilter { variable } => write!(
                f,
                "query FILTER on ?{variable} has no matching slot-bound, true-verdict filter_int sub-proof (FILTER-add / comparison- or operand-substitution / false-verdict row)"
            ),
            CheckError::UnmappableFilterVar { variable } => write!(
                f,
                "query FILTER on ?{variable} does not bind to any scanned column"
            ),
            CheckError::UnattestedCommitment { proof, commitment } => write!(
                f,
                "scan sub-proof {proof}: commitment {commitment} has no issuer attestation (unsigned / prover-invented commitment — no credential provenance)"
            ),
            CheckError::InvalidIssuerSignature { commitment } => write!(
                f,
                "commitment {commitment}: issuer signature does not verify under its declared key (forged/tampered signature or wrong commitment)"
            ),
            CheckError::IssuerKeyNotInKeySet { commitment } => write!(
                f,
                "commitment {commitment}: attestation key is not a member of the external trusted key-set K (untrusted issuer)"
            ),
            CheckError::UntrustedDeclaredKey { key } => write!(
                f,
                "manifest key_set declares key {key} which is not in the external trusted key-set K (prover may not widen the trust anchor)"
            ),
            CheckError::AttestationKeyNotInDeclaredSet { commitment, key } => write!(
                f,
                "commitment {commitment}: attestation key {key} is not in the prover's declared (non-empty) manifest key_set (declared narrowing is inconsistent with the proven attestations)"
            ),
            CheckError::AttributionUnderDeclared { pattern, proof_graph } => write!(
                f,
                "query BGP pattern {pattern}: manifest.attributions omits graph {proof_graph}, which the scan proof's in-circuit attribution shows it drew a matched triple from (under-declared attribution — the [[0],[0]] collapse-two-graphs forge that would drop a cross-graph bnode obligation)"
            ),
            CheckError::AttributionUnbound { pattern } => write!(
                f,
                "query BGP pattern {pattern} has no scan sub-proof carrying a proof-bound attribution to cross-check"
            ),
            CheckError::SaltReused { salt } => write!(
                f,
                "salt {salt} is reused across two distinct committed graphs (audit #9: cross-graph bnode-correlation channel — each graph needs a globally-unique issuer-attested salt)"
            ),
            CheckError::ScanCommitmentSaltMissing { proof, commitment } => write!(
                f,
                "scan sub-proof {proof}: the attestation covering commitment {commitment} carries no salt (audit #9 / codex 2221 HIGH: a scan-covering attestation MUST be salt-bound — a salt-less legacy attestation bypasses salt-separation)"
            ),
            CheckError::AttributionMalformed { proof, expected, got } => write!(
                f,
                "scan sub-proof {proof}: attribution must be present and exactly {expected} bits (CircuitId.k), got {got} (audit #8 / codex 2221 MEDIUM: an omitted/short attribution makes the cross-graph under-declaration check vacuous)"
            ),
            CheckError::NonceReplay => write!(
                f,
                "verifier nonce already seen (audit #4: single-use — a captured (nonce, manifest) pair may not be replayed)"
            ),
            CheckError::NonceBindingMismatch => write!(
                f,
                "manifest binding challenge does not equal the verifier-issued nonce (audit #4: the manifest was minted for a different nonce/session)"
            ),
            CheckError::ScanCommitmentStatusMissing { proof, commitment } => write!(
                f,
                "scan sub-proof {proof}: the attestation covering commitment {commitment} carries no issuer-bound status reference (audit #12: a scan-covering attestation MUST bind a status-list reference — a status-unbound attestation lets a revoked credential be presented unchecked)"
            ),
            CheckError::RevocationReferenceMissing { proof } => write!(
                f,
                "scan sub-proof {proof}: an issuer-bound status reference is present but manifest.revocation is absent (audit #12: the prover dropped the disclosed revocation reference needed to check the issuer-signed status digest)"
            ),
            CheckError::RevocationReferenceMismatch { commitment } => write!(
                f,
                "commitment {commitment}: the disclosed manifest.revocation does not match the issuer-signed status reference (audit #12: index/version/list mismatch — the prover disclosed a different reference than the issuer signed)"
            ),
            CheckError::RevocationReferenceModeInvalid { commitment } => write!(
                f,
                "commitment {commitment}: the status reference's index-disclosure mode is malformed (sq-ayv: must be EXACTLY one of clear index or index_commitment, and the disclosed reference must agree with the issuer-signed one — both-set, neither-set, or a clear/committed mode mismatch is rejected fail-closed)"
            ),
            CheckError::HiddenRevocationRequired { proof } => write!(
                f,
                "scan sub-proof {proof}: a committed-index revocation reference (clear index withheld) requires a manifest.hidden_revocation proof to check liveness against the authoritative root (sq-ayv: the committed-index path moves the liveness decision to the hidden-index proof; revocation is never skipped)"
            ),
            CheckError::HiddenRevocationIndexCommitmentMismatch => write!(
                f,
                "the hidden-revocation proof's public index commitment does not equal the issuer-signed index commitment (sq-ayv cross-binding: the index proven unset must be the index the issuer committed to)"
            ),
            CheckError::StatusSnapshotMissing { status_list, version } => write!(
                f,
                "the relying party has no AUTHORITATIVE status-list snapshot for the credential's reference (list {status_list}, version {version}) (audit #12 re-audit: the liveness bit is read from the relying party's own resolved snapshot, not the prover's — an unresolved reference fails closed)"
            ),
            CheckError::StatusSnapshotTampered { status_list, version } => write!(
                f,
                "the prover-disclosed status-list snapshot for (list {status_list}, version {version}) disagrees with the relying party's AUTHORITATIVE snapshot (audit #12 re-audit: forged/tampered bitstring — the bit decision uses the authoritative bytes, and a disagreeing prover snapshot is rejected)"
            ),
            CheckError::CredentialRevoked { status_list, index } => write!(
                f,
                "credential is REVOKED/SUSPENDED: status bit {index} is SET in list {status_list} (audit #12)"
            ),
            CheckError::StatusListStale { status_list, version } => write!(
                f,
                "status-list snapshot for {status_list} at version {version} is outside the verifier freshness window (audit #12: stale revocation view)"
            ),
            CheckError::HiddenRevocationNotEnabled => write!(
                f,
                "manifest carries a hidden-index revocation proof but the relying party's policy has not enabled the hidden-index path (no Merkle depth set) (sq-3e5/sq-h2v: the verifier cannot derive an authoritative root to bind the proof to)"
            ),
            CheckError::HiddenRevocationDepthMismatch { declared, policy } => write!(
                f,
                "hidden-index revocation proof declares Merkle depth {declared} but the relying party's policy depth is {policy} (sq-3e5/sq-h2v: trees over different leaf layouts cannot share a root)"
            ),
            CheckError::HiddenRevocationRootUnavailable { status_list, version } => write!(
                f,
                "the relying party could not derive an authoritative Merkle root for the hidden-index revocation proof (list {status_list}, version {version}) (sq-3e5/sq-h2v: no authoritative snapshot, or implausible depth -- fail-closed)"
            ),
            CheckError::HiddenRevocationRootMismatch => write!(
                f,
                "hidden-index revocation proof's public Merkle root does not equal the relying party's authoritative root (sq-3e5/sq-h2v: the proof was produced against a different/forged status list)"
            ),
            CheckError::HiddenRevocationProofRejected => write!(
                f,
                "bb rejected the hidden-index revocation proof (sq-3e5/sq-h2v: the zero-knowledge bit-unset/inclusion statement did not verify)"
            ),
            CheckError::HiddenRevocationMalformedProof => write!(
                f,
                "hidden-index revocation proof blob is malformed (sq-3e5/sq-h2v)"
            ),
            CheckError::HiddenIssuerNotEnabled => write!(
                f,
                "hidden-issuer attestation present but the relying party has not enabled the hidden-issuer path (no KeySet::with_hidden_issuer_depth) (sq-z9l)"
            ),
            CheckError::HiddenIssuerDepthMismatch { declared, policy } => write!(
                f,
                "hidden-issuer attestation depth {declared} does not match the KeySet policy depth {policy} (sq-z9l)"
            ),
            CheckError::HiddenIssuerRootUnavailable => write!(
                f,
                "relying party could not derive an authoritative key-set Merkle root for the hidden-issuer proof (set overflows the tree or implausible depth) (sq-z9l)"
            ),
            CheckError::HiddenIssuerRootMismatch => write!(
                f,
                "hidden-issuer proof's public key-set root does not equal the relying party's authoritative key-set root (sq-z9l: proved against a different key set)"
            ),
            CheckError::HiddenIssuerMessageMismatch { commitment } => write!(
                f,
                "hidden-issuer proof's public message does not equal the issuer-signed message recomputed for commitment {commitment} (sq-z9l: proof not bound to this committed graph)"
            ),
            CheckError::HiddenIssuerUnreferencedCommitment { commitment } => write!(
                f,
                "hidden-issuer attestation covers commitment {commitment} which no verified scan sub-proof references (sq-z9l: dangling attestation)"
            ),
            CheckError::HiddenIssuerProofRejected => write!(
                f,
                "bb rejected the hidden-issuer attestation proof (sq-z9l: the zero-knowledge signature-validity + key-set-membership statement did not verify)"
            ),
            CheckError::HiddenIssuerMalformedProof => write!(
                f,
                "hidden-issuer attestation proof blob is malformed (sq-z9l)"
            ),
            CheckError::HolderRegistryEmpty => write!(
                f,
                "manifest binding is HolderPop but the relying party supplied no holder registry (sq-cwq: no trust anchor to check the holder key against — a holder PoP cannot be accepted, fail-closed)"
            ),
            CheckError::HolderNotTrusted { holder } => write!(
                f,
                "HolderPop binding holder key {holder} is not a member of the relying party's holder registry (sq-cwq: unauthorised holder)"
            ),
            CheckError::HolderPopMalformed => write!(
                f,
                "HolderPop binding is unverifiable: unknown cryptosuite, or the holder key / pop signature did not parse (sq-cwq: fail-closed, no silent accept)"
            ),
            CheckError::HolderPopInvalid { holder } => write!(
                f,
                "HolderPop binding pop signature does not verify under holder key {holder} over the challenge-bound message (sq-cwq: the presenter did not prove possession of the holder secret)"
            ),
            CheckError::EntailmentRegimeNotAccepted { regime } => write!(
                f,
                "manifest entailment regime `{regime}` is not accepted by the relying party's EntailmentPolicy (sq-314: the regime is enforced, not free metadata — a relying party must opt into inference)"
            ),
            CheckError::UnexpectedDerivationSteps => write!(
                f,
                "a Simple-regime manifest carried derivation steps (sq-314: Simple means no inference — steps are inconsistent with the regime)"
            ),
            CheckError::MissingDerivationSteps { regime } => write!(
                f,
                "a non-Simple regime `{regime}` carried no derivation steps (sq-314: an inference regime must record the derivation it claims — fail-closed)"
            ),
            CheckError::MalformedDerivationStep { step } => write!(
                f,
                "derivation step {step} is not a well-formed instance of its rule, or its rule is not admitted by the declared regime (sq-314)"
            ),
            CheckError::UngroundedDerivationAntecedent { step, antecedent } => write!(
                f,
                "derivation step {step} antecedent {antecedent} is ungrounded (sq-314: it is neither an earlier step's derived triple nor a disclosed scan row — a derived triple cannot rest on an antecedent the proof does not establish)"
            ),
            CheckError::Driver(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for CheckError {}

impl From<VerifyError> for CheckError {
    fn from(e: VerifyError) -> Self {
        CheckError::Sparqzk(e)
    }
}

impl From<DriverError> for CheckError {
    fn from(e: DriverError) -> Self {
        CheckError::Driver(e)
    }
}

/// Re-derive a sub-proof's circuit id from its public inputs alone (the
/// verifier never trusts the declared `id`).
fn derive_id(inputs: &ProofInputs) -> Option<CircuitId> {
    match inputs {
        ProofInputs::Scan { commitments, rows, row_count, .. } => {
            let k = commitments.len() as u32;
            // The verifier knows r (= declared rows length) and row_count, but
            // not the private graph sizes; n is part of the declared id and
            // re-derived only for r/k. We recompute the id from k + rows.len()
            // (r bucket) + the declared n carried in the inputs' id.
            let r_bucket = rows.len() as u32;
            // n cannot be derived without the witness; trust the declared n in
            // the id but bind k and r.
            let declared_n = match inputs.circuit_id() {
                CircuitId::Scan { n, .. } => *n,
                _ => return None,
            };
            let _ = row_count;
            derive_scan_id(k, declared_n, r_bucket.max(*row_count))
        }
        ProofInputs::FilterInt { bound, .. } => {
            // digit count of the *bound* is not the operand's; the operand's
            // digit count is private. The declared d is re-checked against the
            // member family only (it must be a compiled d).
            let _ = bound;
            let d = match inputs.circuit_id() {
                CircuitId::FilterInt { d } => *d,
                _ => return None,
            };
            derive_filter_int_id(d)
        }
        // [OPUS-4.8] sq-q7e + sq-tat: composable xsd:double FILTER. As with
        // filter_int, the operand's digit count is PRIVATE; the declared `d` is
        // re-checked against the compiled f64 family only (it must be a compiled
        // d). The full CircuitId (incl. `d`) is what the audit-#1 public-input
        // reconstruction + canonical-vk recompute pin, so a wrong `d` cannot
        // byte-match a real member's proof.
        ProofInputs::FilterF64 { .. } => {
            let d = match inputs.circuit_id() {
                CircuitId::FilterF64 { d } => *d,
                _ => return None,
            };
            derive_filter_f64_id(d)
        }
    }
}

/// Stage 1+2: the structural PRE-FILTER (no bb). Returns the required obligation
/// edges on success.
///
/// # ⚠️ THIS IS NOT A VERIFIER — it provides NO cryptographic or attribution soundness.
///
/// [OPUS-4.8] codex 2223 MEDIUM. This runs only the JSON/structural stages
/// (circuit-id re-derivation, binding-edge consistency, query-correctness
/// binding, the cross-graph attribution length+superset gate, and the
/// issuer-signature/salt checks). It does **NOT** run the bb proof verification
/// and does **NOT** run the public-input reconstruction byte-compare. The
/// attribution / pattern / filter / commitment bits it inspects are therefore
/// **NOT cryptographically bound to any proof here** — a manifest can pass this
/// pre-filter while its declared `ProofInputs` differ from what the bb proofs
/// actually attest. The only thing that binds the JSON statement to the proof is
/// [`verify_manifest`], whose stage 3 reconstructs the public-input vector from
/// the DECLARED inputs and byte-compares it against each proof's `public_inputs`
/// (and then runs `bb verify`). **A relying party MUST call [`verify_manifest`];
/// this function is an internal fast pre-filter / test seam only and accepting a
/// manifest on its result alone is unsound.**
///
/// The name was deliberately chosen so it cannot be mistaken for a verifier (it
/// was `verify_manifest_structure`, which read as a verification entry point).
///
/// `trusted_key_set` is the relying party's EXTERNALLY-anchored issuer trust set
/// `K` (audit #3 codex #1) — NOT read from the manifest. Every committed graph's
/// attestation key must be a member of THIS set, and the prover's
/// `manifest.key_set` (if any) must be a subset of it. See [`KeySet`].
///
/// `revocation_policy` is the relying party's freshness/revocation policy (audit
/// #12) — also external, NOT read from the manifest. The status check is
/// mandatory: a scan-covering credential MUST carry an issuer-bound status
/// reference, show a status bit UNSET in a disclosed snapshot, and that snapshot
/// MUST be within the policy's freshness window. See [`RevocationPolicy`].
pub fn prefilter_manifest_structure(
    manifest: &ProofManifest,
    trusted_key_set: &KeySet,
    revocation_policy: &RevocationPolicy,
) -> Result<Vec<JoinEdge>, CheckError> {
    // --- Stage 1a: sparq-zk layer-3 bnode / arity re-check. ---
    let attributions: Vec<BTreeSet<usize>> = manifest
        .attributions
        .iter()
        .map(|s| s.iter().copied().collect())
        .collect();
    let declared: Vec<JoinEdge> = manifest
        .join_obligations
        .iter()
        .map(|(variable, i, j)| JoinEdge {
            variable: variable.clone(),
            patterns: (*i, *j),
        })
        .collect();
    let required = recheck(&manifest.query, &attributions, &declared)?;

    // --- Stage 1b: re-derive each circuit id from public inputs. ---
    for (i, sp) in manifest.sub_proofs.iter().enumerate() {
        let declared = sp.inputs.circuit_id().clone();
        let derived = derive_id(&sp.inputs);
        if derived.as_ref() != Some(&declared) {
            return Err(CheckError::CircuitIdMismatch {
                proof: i,
                declared,
                derived,
            });
        }
        // [OPUS-4.8] codex 2221 MEDIUM (fail-closed for audit #8): a scan's
        // per-graph source attribution is a security-relevant, proof-bound
        // quantity (`scan.nr` step 4) the cross-graph obligation gate (stage 2e)
        // cross-checks against `manifest.attributions`. `#[serde(default)]` lets a
        // prover OMIT it (empty vec) or under-length it; `bind_attributions` then
        // only inspects the bits PRESENT, so an omitted attribution makes the
        // under-declaration cross-check vacuous and resurrects the `[[0],[0]]`
        // collapse forge. Require it PRESENT and EXACTLY `CircuitId.k` bits — the
        // same k the (already-verified) circuit id declares and the audit #1
        // reconstruction byte-binds — with NO default and NO pad-to-false. A
        // missing/empty/short/long attribution is rejected here, before any gate
        // can silently skip the omitted graphs.
        if let ProofInputs::Scan { attribution, .. } = &sp.inputs {
            let k = match &declared {
                CircuitId::Scan { k, .. } => *k as usize,
                _ => unreachable!("Scan inputs always carry a Scan circuit id"),
            };
            if attribution.len() != k {
                return Err(CheckError::AttributionMalformed {
                    proof: i,
                    expected: k,
                    got: attribution.len(),
                });
            }
        }
    }

    // --- Stage 2: binding-consistency edges. ---
    for (e, edge) in manifest.binding_edges.iter().enumerate() {
        let from = manifest
            .sub_proofs
            .get(edge.from_proof)
            .ok_or(CheckError::DanglingEdge { edge: e })?;
        let to = manifest
            .sub_proofs
            .get(edge.to_proof)
            .ok_or(CheckError::DanglingEdge { edge: e })?;
        let scanned = match &from.inputs {
            ProofInputs::Scan { rows, .. } => rows
                .get(edge.from_row)
                .and_then(|r| r.get(edge.from_slot))
                .ok_or(CheckError::DanglingEdge { edge: e })?,
            _ => return Err(CheckError::EdgeKindMismatch { edge: e }),
        };
        // [OPUS-4.8] sq-q7e + sq-tat: a binding edge may consume the scanned
        // column into EITHER an xsd:integer FILTER (filter_int) or a composable
        // xsd:double FILTER (filter_f64) — both carry `operand_enc` as the
        // scan-proof anchor, bound to the committed literal in-circuit.
        let operand = match &to.inputs {
            ProofInputs::FilterInt { operand_enc, .. } => operand_enc,
            ProofInputs::FilterF64 { operand_enc, .. } => operand_enc,
            _ => return Err(CheckError::EdgeKindMismatch { edge: e }),
        };
        if scanned != operand {
            return Err(CheckError::BindingInconsistent { edge: e });
        }
    }

    // --- Stage 2b + 2c: query-correctness binding (audit #5/#6/#7/#10). ---
    // The bb public-input vector already cryptographically binds the scan
    // pattern constants and the FILTER op/bound/expected/operand_enc (audit
    // #1). This stage is the VERIFIER-SIDE check that those bound values match
    // the query the relying party reads in `manifest.query` — without it a
    // proof of one statement is presented under a different query.
    bind_query_correctness(manifest)?;

    // --- Stage 2e: cross-graph attribution binding (audit #8). ---
    // Bind manifest.attributions (the JSON sets fed to the Q6 obligation gate in
    // stage 1a) to the PROOF-BOUND per-graph attribution each scan sub-proof
    // carries (scan.nr step 4, byte-bound by the audit #1 reconstruction). A
    // prover whose pattern genuinely matches in two graphs can no longer declare
    // a collapsed `[[0],[0]]` to drop the cross-graph non-bnode obligation: the
    // declared attribution must be a SUPERSET of the proof-bound matched-graph
    // set, so under-declaring a contributing graph is rejected here.
    bind_attributions(manifest)?;

    // --- Stage 2d: issuer-signature / key-set binding (audit #3 / codex #1). ---
    // Every scan sub-proof's commitments[g] must carry a valid issuer signature
    // whose key ∈ the EXTERNAL trusted K (the verifier's argument, NOT
    // manifest.key_set). commitments[g] is byte-bound into the bb public inputs
    // by the audit #1 reconstruction, so this verifier-side check ties the
    // attested commitment to the proved statement. The issuer signature ALSO
    // binds the credential's status-list reference (audit #12), so a scan-covering
    // attestation that omits/forges the reference is rejected here.
    //
    // [OPUS-4.8] sq-xxg: a scan commitment may be covered EITHER by a clear
    // attestation OR by a hidden-issuer proof (when the relying party enabled the
    // hidden path). Compute the STRUCTURALLY hidden-covered commitments so the
    // clear attestation is treated as NOT-REQUIRED for them; the fail-closed
    // either-clear-or-hidden / never-neither rule is enforced inside
    // `bind_issuer_attestations`. The hidden proof itself is cryptographically
    // verified by `bind_hidden_issuer_attestations` in the bb stage of
    // `verify_manifest` (which binds its public `key_set_root` to the relying
    // party's authoritative KeySet and its `m` to the issuer-signed message), so a
    // hidden-only commitment is no less attested than a clear one — only WHICH
    // issuer signed is hidden.
    let hidden_covered = hidden_issuer_covered_commitments(manifest, trusted_key_set);
    bind_issuer_attestations(manifest, trusted_key_set, &hidden_covered)?;

    // --- Stage 2f: revocation / freshness (audit #12). ---
    // The status reference is now issuer-bound (stage 2d); check the credential's
    // status bit is UNSET in the disclosed snapshot and the snapshot version is
    // within the relying party's freshness window. A revoked bit, a stale
    // snapshot, or a missing snapshot all REJECT (fail-closed).
    bind_revocation(manifest, revocation_policy)?;

    Ok(required)
}

/// Stage 2d: bind every scan commitment to an issuer signature whose key is in
/// the EXTERNAL trusted key-set `K` (audit #3, soundness fix for codex #1). For
/// each `commitments[g]` of each scan sub-proof:
/// - there MUST be a `commitment_attestations` entry over that commitment value
///   OR a hidden-issuer proof covering it (sq-xxg, see below),
/// - its signature MUST verify under its declared `issuer_public_key`,
/// - that key MUST be a member of the EXTERNAL `trusted_key_set` (the relying
///   party's argument — NEVER `manifest.key_set`),
/// - and, when the prover DECLARED a narrowed `manifest.key_set` (non-empty),
///   that key MUST ALSO be a member of it (codex 2216 LOW): a declared narrowing
///   must be internally consistent with the attestations actually proven. The
///   accept decision stays anchored on the external K (this consistency rule is
///   ADDED to, never substituted for, the external-K check).
///
/// # sq-xxg: clear attestation is OPTIONAL when a hidden-issuer proof covers it
/// `hidden_covered` is the set of commitment-hex keys for which the manifest
/// carries a `hidden_issuer_attestations` entry AND the relying party enabled the
/// hidden-issuer path. For a commitment in this set, the clear-key
/// `commitment_attestations` entry is NOT required: the hidden-issuer proof (whose
/// public `key_set_root` is bound to the relying party's authoritative KeySet, and
/// whose `m` is the issuer-signed message, both checked cryptographically by
/// [`bind_hidden_issuer_attestations`] in the bb stage) is the attestation. The
/// privacy gain is that WHICH trusted issuer signed is hidden; the soundness
/// guarantee ("signed by SOME key in the relying party's K over this committed
/// graph's status-bound message") is unchanged.
///
/// FAIL-CLOSED (either-clear-or-hidden, NEVER neither): a commitment with NEITHER
/// a clear attestation NOR a hidden-issuer entry is rejected
/// ([`CheckError::UnattestedCommitment`]). A commitment WITH a clear attestation
/// is still fully checked here (key ∈ K, signature, salt, status) regardless of
/// any hidden entry — the hidden path only RELAXES the *requirement* for a clear
/// entry, it never relaxes the checks applied to a clear entry that IS present.
/// So no commitment can go unattested, and a hidden-only commitment is gated by
/// the (mandatory, fail-closed) bb hidden-issuer verification.
///
/// # The codex #1 soundness fix
/// Previously the trust anchor was `manifest.key_set` — PROVER-supplied. A
/// malicious prover could sign a forged commitment with its own key, list that
/// key in `manifest.key_set`, and pass. That made `#3` vacuous. Now the anchor
/// is `trusted_key_set`, an external relying-party input. `manifest.key_set` is
/// only allowed as a SUBSET of the external `K` (a prover may narrow but never
/// widen the trust set); a `manifest.key_set` key NOT in the external `K` is
/// rejected ([`CheckError::UntrustedDeclaredKey`]) so the accept decision can
/// never depend on a prover-chosen key.
///
/// Closes: (a) an unsigned/prover-invented commitment (no matching attestation),
/// (b) drop-a-triple-and-recommit suppression (the truncated graph's `C(G')`
/// differs from the signed `C(G)`, so no valid attestation exists),
/// (c) a signature by a key not in the external K (incl. the prover's own key —
/// the forge the codex #1 test exercises). The commitment is no longer an
/// unsigned prover-chosen public input under a prover-chosen trust anchor.
fn bind_issuer_attestations(
    manifest: &ProofManifest,
    trusted_key_set: &KeySet,
    hidden_covered: &BTreeSet<String>,
) -> Result<(), CheckError> {
    // The prover's declared key_set may NARROW but never WIDEN the external trust
    // anchor: every key it lists must already be in the external K. (A
    // declared-but-untrusted key is the codex #1 forge: prover lists its own key.)
    for declared in &manifest.key_set {
        if !trusted_key_set.contains_hex(declared) {
            return Err(CheckError::UntrustedDeclaredKey {
                key: declared.clone(),
            });
        }
    }

    // [OPUS-4.8] codex 2223 LOW: the verified per-graph salt for every commitment
    // ACTUALLY REFERENCED by a verified scan sub-proof. The salt-uniqueness check
    // (step 3) runs ONLY over this referenced set, not over every declared
    // attestation: the #9 security property only concerns committed graphs a
    // verified scan drew triples from, so an unrelated extra attestation reusing a
    // salt must NOT false-reject a valid proof. Keyed by canonical commitment hex
    // (so the same graph referenced by several scans records once); the value is
    // the verified salt hex. Populated only after the attestation over `c` has
    // fully verified (key ∈ K, signature valid, salt present + salt-bound), so a
    // recorded salt is always issuer-attested.
    let mut referenced_salt: std::collections::BTreeMap<String, String> =
        std::collections::BTreeMap::new();

    for (pi, sp) in manifest.sub_proofs.iter().enumerate() {
        let ProofInputs::Scan { commitments, .. } = &sp.inputs else {
            continue;
        };
        for c in commitments {
            // Find an attestation declared over this exact commitment value
            // (compare as field elements, so 0x-padding differences don't slip
            // an unattested commitment past).
            let c_field = c.to_field();
            let att = manifest.commitment_attestations.iter().find(|a| {
                a.commitment.to_field().is_some() && a.commitment.to_field() == c_field
            });
            let Some(att) = att else {
                // [OPUS-4.8] sq-xxg: no CLEAR attestation. This is acceptable ONLY
                // if a hidden-issuer proof covers this commitment (and the relying
                // party enabled the hidden path) — the fail-closed
                // either-clear-or-hidden / never-neither rule. The hidden proof is
                // cryptographically verified by `bind_hidden_issuer_attestations`
                // (bb stage), so a hidden-covered commitment is fully attested
                // there; here we only RELAX the clear-entry requirement. A
                // commitment covered by NEITHER is rejected as unattested.
                if let Some(c_fr) = c_field {
                    if hidden_covered.contains(&field_to_hex(&c_fr)) {
                        // The hidden-only commitment still participates in the
                        // audit-#9 salt-uniqueness check (the Q6 cross-graph
                        // bnode-correlation channel applies to ANY committed graph
                        // a verified scan drew from, hidden-attested or clear). Its
                        // salt is the one the verifier uses to recompute the
                        // issuer-signed `m` (resolve_commitment_salt). If no salt is
                        // disclosed for a hidden-only commitment, the message cannot
                        // be recomputed and the bb hidden-issuer gate rejects it as
                        // unreferenced (fail-closed) — so we record only a present,
                        // parseable salt here.
                        if let Some(salt_fr) = resolve_commitment_salt(manifest, &c_fr) {
                            referenced_salt
                                .insert(field_to_hex(&c_fr), field_to_hex(&salt_fr));
                        }
                        continue;
                    }
                }
                return Err(CheckError::UnattestedCommitment {
                    proof: pi,
                    commitment: c.0.clone(),
                });
            };
            // (1) The attestation key must be in the EXTERNAL trusted set K.
            // (Check membership BEFORE the signature so an untrusted issuer is
            // the reported reason even if its signature is internally valid.)
            // This is the codex #1 fix: K is the verifier's argument, never the
            // prover-supplied manifest.key_set.
            if !trusted_key_set.contains_hex(&att.issuer_public_key) {
                return Err(CheckError::IssuerKeyNotInKeySet {
                    commitment: c.0.clone(),
                });
            }
            // (1b) [OPUS-4.8] codex 2216 LOW: declared-key_set consistency. The
            // accept decision is ALREADY anchored on the external K above (never
            // weakened here). But if the prover DECLARED a narrowed key_set
            // (non-empty), every accepted attestation key must also be a member
            // of it — otherwise the advertised narrowing is inconsistent with the
            // attestations actually proven (the prover claims a tighter issuer
            // set than it used). An empty declared key_set means "no narrowing
            // declared", so this is skipped (external K alone governs).
            if !manifest.key_set.is_empty()
                && !manifest
                    .key_set
                    .iter()
                    .any(|k| normalize_hex(k) == normalize_hex(&att.issuer_public_key))
            {
                return Err(CheckError::AttestationKeyNotInDeclaredSet {
                    commitment: c.0.clone(),
                    key: att.issuer_public_key.clone(),
                });
            }
            // (2) The cryptosuite must be a known/verifiable scheme, the key +
            // signature must parse, and the signature must verify over the
            // SALT-BOUND domain-separated commitment message. Any failure =>
            // reject (fail closed; prover-controlled bytes never panic).
            let Some(commitment_fr) = c_field else {
                return Err(CheckError::InvalidIssuerSignature {
                    commitment: c.0.clone(),
                });
            };
            // [OPUS-4.8] codex 2221 HIGH (fail-closed for audit #9): an
            // attestation that COVERS a verified scan commitment MUST carry a salt
            // and be verified via the salt-bound `commitment_message_with_salt`
            // path. The salt-less (legacy) `commitment_message` path does NOT bind
            // the per-graph RDFC10 salt and does NOT participate in the
            // salt-uniqueness check below, so accepting it here would let a
            // salt-reusing ingester bypass the whole #9 salt-separation guarantee
            // simply by OMITTING the salt field. There is therefore NO `salt:
            // None` branch on the scan-covering path — `None` is rejected. (The
            // bare `commitment_message` remains a primitive used elsewhere; it is
            // not reachable from the default scan-verify path.) A salt that is
            // present but unparseable also fails closed.
            let salt_fr = match &att.salt {
                Some(salt_hex) => match salt_hex.to_field() {
                    Some(salt_fr) => salt_fr,
                    None => {
                        return Err(CheckError::InvalidIssuerSignature {
                            commitment: c.0.clone(),
                        })
                    }
                },
                None => {
                    return Err(CheckError::ScanCommitmentSaltMissing {
                        proof: pi,
                        commitment: c.0.clone(),
                    })
                }
            };
            // [OPUS-4.8] audit #12 (fail-closed, leverages #3): a scan-covering
            // attestation MUST bind the credential's STATUS-LIST REFERENCE. The
            // issuer signs `commitment_message_with_status(C(G), salt,
            // status_ref_digest)`, where `status_ref_digest` folds the disclosed
            // `manifest.revocation` (H(list IRI), index, version). So the signed
            // message can only be reconstructed from a disclosed reference that
            // MATCHES what the issuer signed — an omitted/forged/swapped reference
            // yields a different message and so no valid signature (the
            // omit-the-field bypass that bit #3/#8/#9/#4 is closed here). There is
            // NO `status: None` branch on the scan-covering path — `None` is
            // rejected (mirrors the salt-mandatory precedent above).
            let Some(att_status) = &att.status else {
                return Err(CheckError::ScanCommitmentStatusMissing {
                    proof: pi,
                    commitment: c.0.clone(),
                });
            };
            // The disclosed revocation reference is required to recompute the
            // signed digest (the prover cannot drop it to skip the check).
            let Some(rev) = &manifest.revocation else {
                return Err(CheckError::RevocationReferenceMissing { proof: pi });
            };
            // The disclosed reference MUST equal what the issuer signed (the
            // attestation's `AttestedStatusRef`), in the SAME disclosure mode. A
            // mismatch means the prover disclosed a different reference than was
            // signed (e.g. an index/commitment whose bit is unset); reject
            // explicitly. [OPUS-4.8] sq-ayv: `resolve_status_ref` handles BOTH the
            // clear-index (audit #12) and committed-index paths and recomputes the
            // issuer-signed `status_ref` over the disclosed value the issuer signed
            // (clear index OR index commitment).
            let list_id_fr = status_list_id_to_field(&rev.status_list);
            let (status_ref, _mode) =
                resolve_status_ref(rev, att_status, &list_id_fr, &c.0)?;
            let message =
                commitment_message_with_status(&commitment_fr, &salt_fr, &status_ref);
            let ok = SignatureScheme::from_cryptosuite_iri(&att.cryptosuite).is_some()
                && match (
                    public_key_from_hex(&att.issuer_public_key),
                    signature_from_hex(&att.signature),
                ) {
                    (Some(pk), Some(sig)) => sig_verify(&pk, &message, &sig),
                    _ => false,
                };
            if !ok {
                return Err(CheckError::InvalidIssuerSignature {
                    commitment: c.0.clone(),
                });
            }
            // [OPUS-4.8] codex 2223 LOW: record the now-verified salt for this
            // SCAN-REFERENCED commitment. Keyed by canonical commitment hex so the
            // same graph referenced by multiple scans records a single entry; the
            // salt-uniqueness gate (step 3) iterates only this set, so an unrelated
            // extra attestation never participates.
            referenced_salt.insert(field_to_hex(&commitment_fr), field_to_hex(&salt_fr));
        }
    }

    // (3) Salt uniqueness (audit #9): no two DISTINCT committed graphs USED BY A
    // VERIFIED SCAN may share a salt. A reused salt is the Q6 cross-graph
    // bnode-correlation channel — a same-label canonical bnode then encodes
    // identically across both graphs. [OPUS-4.8] codex 2223 LOW: this check is
    // scoped to `referenced_salt` (commitments an actually-verified scan drew from)
    // rather than every `manifest.commitment_attestations` entry. The #9 property
    // only concerns committed graphs a verified scan used, so an UNRELATED extra
    // attestation that happens to reuse a salt must NOT false-reject an otherwise
    // valid proof. Each recorded salt is already issuer-attested (recorded only
    // after the signature verified above). A salt reused across two distinct
    // SCAN-referenced commitments still REJECTS. (Two attestations over the SAME
    // commitment collapse to one entry by the commitment-keyed map and so are
    // fine: the same graph attested twice.)
    let mut salt_to_commitment: std::collections::BTreeMap<String, String> =
        std::collections::BTreeMap::new();
    for (commitment_key, salt_key) in &referenced_salt {
        match salt_to_commitment.get(salt_key) {
            Some(prev) if prev != commitment_key => {
                return Err(CheckError::SaltReused {
                    salt: salt_key.clone(),
                });
            }
            _ => {
                salt_to_commitment.insert(salt_key.clone(), commitment_key.clone());
            }
        }
    }
    Ok(())
}

/// The set of commitment-hex keys (canonical `field_to_hex`) for which a CLEAR
/// issuer attestation may be withheld because a HIDDEN-ISSUER proof covers them
/// (sq-xxg). A commitment is hidden-covered iff (a) the relying party ENABLED the
/// hidden-issuer path (`KeySet::with_hidden_issuer_depth`) AND (b) the manifest
/// carries a `hidden_issuer_attestations` entry for it.
///
/// This is a STRUCTURAL coverage set only — it does NOT verify the hidden proof
/// (that is the bb-stage [`bind_hidden_issuer_attestations`], which binds the
/// proof's public `key_set_root` to the authoritative KeySet and its `m` to the
/// issuer-signed message, and rejects a dangling/forged/unreferenced entry). So a
/// commitment in this set is treated as "clear attestation not required" by the
/// structural [`bind_issuer_attestations`], but is still gated by the mandatory,
/// fail-closed bb hidden-issuer verification before `verify_manifest` accepts.
///
/// When the relying party did NOT enable the hidden path, this returns the empty
/// set, so every commitment then requires a clear attestation exactly as before
/// (a manifest carrying hidden entries against a non-enabled policy is separately
/// rejected by `bind_hidden_issuer_attestations` with `HiddenIssuerNotEnabled`).
// [OPUS-4.8] sq-xxg: structural hidden-issuer coverage set (clear-attestation
// optionality). Soundness rests on the bb gate; this only relaxes the structural
// clear-entry requirement and never lets a commitment go uncovered (either-clear-
// or-hidden, never-neither — enforced in `bind_issuer_attestations`).
fn hidden_issuer_covered_commitments(
    manifest: &ProofManifest,
    trusted_key_set: &KeySet,
) -> BTreeSet<String> {
    if trusted_key_set.hidden_issuer_depth().is_none() {
        // The hidden path is disabled; no commitment may withhold its clear
        // attestation. (Any hidden entry present is rejected fail-closed by the
        // bb-stage gate with HiddenIssuerNotEnabled.)
        return BTreeSet::new();
    }
    manifest
        .hidden_issuer_attestations
        .iter()
        .filter_map(|hi| hi.commitment.to_field().map(|f| field_to_hex(&f)))
        .collect()
}

/// Whether a status reference uses the sq-ayv COMMITTED-index path (the clear
/// index is withheld and a hiding `index_commitment` is bound) vs the audit-#12
/// CLEAR-index path. Determined by the ATTESTATION's signed reference
/// (`AttestedStatusRef`), which the issuer controls — never a prover claim alone.
// [OPUS-4.8] sq-ayv.
#[derive(Clone, Copy, PartialEq, Eq)]
enum StatusRefMode {
    /// Clear-index (audit #12): the issuer signed `status_ref_digest(list, index,
    /// version)`; liveness is the clear-index bit check.
    Clear,
    /// Committed-index (sq-ayv): the issuer signed
    /// `status_ref_commit_digest(list, index_commitment, version)`; liveness is the
    /// hidden-index proof cross-bound to that commitment.
    Committed,
}

/// Resolve the issuer-signed `status_ref` field element AND the disclosure mode
/// from the attestation's signed reference (`att_status`) cross-checked against
/// the disclosed `rev` reference (sq-ayv). Fail-closed:
/// - the ATTESTED reference must be EXACTLY one of clear `index` / committed
///   `index_commitment` (both-set or neither-set => `RevocationReferenceModeInvalid`);
/// - the DISCLOSED `rev` must agree on the same mode and value (a mismatch or a
///   mode disagreement => the named error). The clear-index path stays byte-for-byte
///   what audit #12 did; the committed path recomputes the digest over the disclosed
///   `index_commitment` (which the issuer signed, never the clear index).
///
/// Returns `(status_ref_fr, mode)`. `list_id_fr` is the caller-hashed list IRI.
// [OPUS-4.8] sq-ayv: mode-aware status-reference resolution.
fn resolve_status_ref(
    rev: &crate::manifest::RevocationStatus,
    att_status: &crate::manifest::AttestedStatusRef,
    list_id_fr: &Fr,
    commitment_hex: &str,
) -> Result<(Fr, StatusRefMode), CheckError> {
    let att_committed = att_status.index_commitment.is_some();
    let att_clear = att_status.index.is_some();
    // The ATTESTED reference must be EXACTLY one mode.
    if att_committed == att_clear {
        return Err(CheckError::RevocationReferenceModeInvalid {
            commitment: commitment_hex.to_string(),
        });
    }
    if att_committed {
        // Committed-index (sq-ayv). The disclosed reference must withhold the clear
        // index AND disclose a matching commitment + version.
        let att_ic_hex = att_status
            .index_commitment
            .as_ref()
            .expect("att_committed");
        let Some(rev_ic) = &rev.index_commitment else {
            return Err(CheckError::RevocationReferenceModeInvalid {
                commitment: commitment_hex.to_string(),
            });
        };
        // A disclosed clear index alongside a committed reference is a mode
        // disagreement (the index must be withheld on the committed path).
        if rev.index.is_some() {
            return Err(CheckError::RevocationReferenceModeInvalid {
                commitment: commitment_hex.to_string(),
            });
        }
        // The disclosed commitment must equal the issuer-signed one (compare as
        // field elements so 0x-padding cannot slip a different value past), and
        // the version must match.
        let (Some(att_ic_fr), Some(rev_ic_fr)) = (att_ic_hex.to_field(), rev_ic.to_field())
        else {
            return Err(CheckError::RevocationReferenceModeInvalid {
                commitment: commitment_hex.to_string(),
            });
        };
        if att_ic_fr != rev_ic_fr || rev.version != att_status.version {
            return Err(CheckError::RevocationReferenceMismatch {
                commitment: commitment_hex.to_string(),
            });
        }
        let status_ref = status_ref_commit_digest(list_id_fr, &att_ic_fr, att_status.version);
        Ok((status_ref, StatusRefMode::Committed))
    } else {
        // Clear-index (audit #12), unchanged. The disclosed reference must carry the
        // same clear index + version and NOT carry a commitment.
        let att_index = att_status.index.expect("att_clear");
        let Some(rev_index) = rev.index else {
            return Err(CheckError::RevocationReferenceMismatch {
                commitment: commitment_hex.to_string(),
            });
        };
        if rev.index_commitment.is_some() {
            return Err(CheckError::RevocationReferenceModeInvalid {
                commitment: commitment_hex.to_string(),
            });
        }
        if rev_index != att_index || rev.version != att_status.version {
            return Err(CheckError::RevocationReferenceMismatch {
                commitment: commitment_hex.to_string(),
            });
        }
        let status_ref = status_ref_digest(list_id_fr, rev_index, att_status.version);
        Ok((status_ref, StatusRefMode::Clear))
    }
}

/// Stage 2f: revocation / freshness check (audit #12). PRECONDITION:
/// [`bind_issuer_attestations`] has already run and accepted the manifest, so —
/// for every scan-covering commitment — the disclosed `manifest.revocation`
/// reference has been cross-checked against, and bound under, the issuer
/// signature (its index/version equal the signed [`AttestedStatusRef`] and the
/// `status_ref_digest` over `(H(list IRI), index, version)` is the one the
/// issuer signed). So here `manifest.revocation` is the ISSUER'S reference, not
/// a prover claim.
///
/// This stage then validates the credential's LIVENESS against the AUTHORITATIVE
/// status-list snapshot (resolved by the relying party into [`RevocationPolicy`],
/// NOT taken from the prover) and the relying party's freshness policy:
/// - resolve the AUTHORITATIVE [`StatusListSnapshot`] matching the reference's
///   `(status_list, version)` from the policy — NO authoritative snapshot REJECTS
///   ([`CheckError::StatusSnapshotMissing`]);
/// - the version MUST be within the policy's freshness window — a stale
///   (or future-dated) reference REJECTS ([`CheckError::StatusListStale`]);
/// - if the prover ALSO disclosed a snapshot for the SAME `(list, version)` it
///   MUST byte-equal the authoritative one — a disagreeing prover snapshot is a
///   tamper signal and REJECTS ([`CheckError::StatusSnapshotTampered`]);
/// - the credential's status bit at `index` IN THE AUTHORITATIVE SNAPSHOT MUST be
///   UNSET — a set bit means the credential is REVOKED/SUSPENDED and REJECTS
///   ([`CheckError::CredentialRevoked`]).
///
/// # The authenticated-bits fix (Option B) — load-bearing
/// The bit decision reads `authoritative.bit(index)`, where `authoritative` is
/// the relying party's OWN snapshot for the reference's `(list, version)` — NEVER
/// `manifest.status_snapshots`. The issuer signature binds the reference (which
/// list / slot / version) but NOT the bit values; if the verifier read the
/// prover's snapshot bytes for the bit, a prover holding a genuine reference
/// could attach a forged all-zero snapshot and a REVOKED credential would verify
/// (the re-audit break). Sourcing the bitstring externally — exactly as the
/// trusted key-set `K` is external (audit #3) — moves the liveness decision off
/// prover-controlled bytes. The prover's snapshot, if present for the referenced
/// key, is only checked for byte-equality with the authoritative one (a tamper
/// tripwire); it is otherwise ignored.
///
/// # Mandatory / fail-closed
/// This runs whenever the manifest carries a `revocation` reference, which —
/// because a scan-covering attestation MUST bind one (audit #12 in
/// `bind_issuer_attestations`) — is EVERY manifest carrying a scan. A
/// status-bound credential cannot reach acceptance without passing this. (A
/// manifest with no scans and no revocation has nothing to revoke; the check is
/// vacuous, not skipped — there is no scan-covering commitment whose liveness is
/// in question.) A relying party that has resolved NO authoritative snapshot for
/// the referenced list/version rejects (it cannot authenticate the liveness view).
///
/// # Privacy — the hidden-index upgrade (sq-3e5 / sq-h2v) and its residual gap
/// `index` is matched against the authoritative bitstring in the clear HERE, so
/// this clear-index path reveals WHICH list slot the credential occupies (a
/// linkability handle). The IN-CIRCUIT hidden-index inclusion + bit-unset proof is
/// now IMPLEMENTED ([`bind_hidden_revocation`] + the `revoke_unset_d{depth}`
/// circuit): it proves "the bit at my HIDDEN index in the tree rooted at the
/// relying party's authoritative root is unset" in zero knowledge, disclosing
/// neither the index nor the other bits, and is bound to the verifier's OWN
/// authoritative Merkle root (so the trust anchor is preserved). When a manifest
/// carries `hidden_revocation` and the policy enables it, that proof is the
/// index-hiding liveness evidence.
///
/// # sq-ayv: the residual index leak is now CLOSED (committed-index path)
/// The index-leak gap is resolved. A credential may use the COMMITTED-index path:
/// the issuer signs `status_ref_commit_digest(H(list), index_commitment, version)`
/// — a HIDING commitment to the index, not the clear index — so the clear index is
/// withheld from every signed object AND disclosed field (`RevocationStatus.index`
/// is `None`). In that mode this function does NOT run the clear bit check (there
/// is no clear index); instead it REQUIRES a hidden-index proof
/// ([`CheckError::HiddenRevocationRequired`]) whose in-circuit cross-binding ties
/// the proven-unset index to the issuer-signed commitment ([`bind_hidden_revocation`]).
/// So a hidden-revocation presentation discloses neither the index nor the liveness
/// bit, while revocation is still checked against the authoritative root (never
/// skipped). The CLEAR-index path below remains unchanged for clear references (the
/// always-on soundness floor); a relying party that does not need index-hiding can
/// keep using it.
///
/// HONEST REMAINING DISCLOSURE: the status-list IRI and the `version` are still
/// disclosed in the clear (both issuer-bound); only the index + liveness bit are
/// hidden on the committed path. Closing the IRI/version leak is sq-6qe (a new
/// committed-reference circuit + sig path; see `research/zk-statuslist-hide-iri-version.md`).
/// [OPUS-4.8] sq-hwe: the host Merkle builder is now SPARSE
/// ([`crate::revocation::merkle_root`]) — `O(set-bits * depth)`, independent of
/// `2^depth` — so it no longer bounds the list size; the remaining size bound is
/// the single COMPILED `revoke_unset_d10` circuit member (compiling a deeper member
/// is mechanical, the relation is depth-generic — tracked as follow-up).
// [OPUS-4.8] audit #12: verifier-side revocation / freshness check.
// [OPUS-4.8] audit #12 re-audit (Option B): the bit decision reads the
// AUTHORITATIVE (relying-party-resolved) snapshot, never the prover's bytes.
fn bind_revocation(
    manifest: &ProofManifest,
    policy: &RevocationPolicy,
) -> Result<(), CheckError> {
    let Some(rev) = &manifest.revocation else {
        // No revocation reference. `bind_issuer_attestations` guarantees that a
        // scan-covering commitment forces one to be present (and bound), so
        // reaching here means there is no scan-covering credential whose liveness
        // is in question — the check is vacuously satisfied.
        return Ok(());
    };
    // Freshness FIRST, on the (issuer-bound) reference's version: a stale (or
    // future-dated) reference is rejected so a revoked-since-snapshot credential
    // cannot slip through on an old "active" view, regardless of whether the
    // relying party still holds that old version's snapshot.
    if !policy.is_fresh(rev.version) {
        return Err(CheckError::StatusListStale {
            status_list: rev.status_list.clone(),
            version: rev.version,
        });
    }
    // Resolve the AUTHORITATIVE snapshot from the relying party's policy (NOT the
    // prover). No authoritative snapshot for the referenced (list, version) =>
    // the verifier cannot authenticate the liveness view => REJECT fail-closed.
    let Some(authoritative) = policy.authoritative_snapshot(&rev.status_list, rev.version) else {
        return Err(CheckError::StatusSnapshotMissing {
            status_list: rev.status_list.clone(),
            version: rev.version,
        });
    };
    // [OPUS-4.8] sq-ayv: the index-disclosure mode. A COMMITTED-index reference
    // (clear index withheld, `index_commitment` present) MOVES the liveness
    // decision to the hidden-index proof (`bind_hidden_revocation`, bb stage),
    // which proves bit-unset against THIS authoritative root and cross-binds the
    // index commitment. The clear bit-read below cannot run (there is no clear
    // index), so we REQUIRE a hidden-revocation proof here (fail-closed —
    // revocation is never skipped) and defer the bit decision to it.
    match (rev.index, &rev.index_commitment) {
        (None, Some(_)) => {
            // Committed-index path: a hidden-revocation proof MUST be present (its
            // cryptographic bit-unset + cross-binding verification is the liveness
            // gate, run in the bb stage). Without it, liveness is unchecked.
            if manifest.hidden_revocation.is_none() {
                return Err(CheckError::HiddenRevocationRequired { proof: 0 });
            }
            // The authoritative snapshot's existence + freshness are already
            // checked above; the bit-unset decision happens in bind_hidden_revocation
            // against the root derived from this same authoritative snapshot.
        }
        (Some(index), None) => {
            // Clear-index path (audit #12), unchanged. Liveness (THE security
            // decision): the credential's status bit must be UNSET IN THE
            // AUTHORITATIVE snapshot (out-of-range reads as SET — fail closed). This
            // reads the relying party's OWN authenticated bytes, so the verdict is
            // identical regardless of what snapshot the prover attached.
            if authoritative.bit(index) {
                return Err(CheckError::CredentialRevoked {
                    status_list: rev.status_list.clone(),
                    index,
                });
            }
        }
        _ => {
            // Neither or both disclosed: a malformed reference mode. (The
            // attestation-side check `resolve_status_ref` already enforces the mode
            // for status-bound scans; this guards a `revocation` with no scan
            // covering it, fail-closed.)
            return Err(CheckError::RevocationReferenceModeInvalid {
                commitment: rev.status_list.clone(),
            });
        }
    }
    // Tamper tripwire: the authoritative bit is UNSET, but if the prover ALSO
    // disclosed a snapshot for this exact (list, version) it MUST byte-equal the
    // authoritative one. The liveness verdict above did not depend on it; this
    // only surfaces a disagreeing prover snapshot as an explicit forgery signal
    // (e.g. the prover lied the OTHER way — claimed REVOKED — or doctored
    // unrelated bits) rather than silently accepting it.
    // [OPUS-4.8] audit #12 re-audit: prover snapshot is a tamper tripwire only.
    // roborev #2263: check EVERY snapshot matching (list, version), not just the first.
    // A prover can attach duplicate entries — a benign one that byte-equals the
    // authoritative snapshot followed by a forged one — and a `.find()` that stops at the
    // first match would never inspect the forgery. `any()` over all matches trips on ANY
    // disagreeing snapshot.
    if manifest
        .status_snapshots
        .iter()
        .any(|s| s.status_list == rev.status_list && s.version == rev.version && s.bits != authoritative.bits)
    {
        return Err(CheckError::StatusSnapshotTampered {
            status_list: rev.status_list.clone(),
            version: rev.version,
        });
    }
    Ok(())
}

/// The HIDDEN-INDEX revocation check (sq-3e5 / sq-h2v) — the privacy upgrade
/// over [`bind_revocation`]'s clear-index path. Runs only the cryptographic gate
/// for `manifest.hidden_revocation`; the structural pre-conditions (the
/// issuer-bound reference, freshness) are still enforced by
/// [`bind_issuer_attestations`] + [`bind_revocation`] in the structural prefilter.
///
/// # What it proves and what stays hidden
/// The prover supplies a `revoke_unset_d{depth}` bb proof whose PUBLIC inputs are
/// the verifier's `challenge` and a status-list Merkle `root`; the holder's
/// INDEX, the leaf bit, and the authentication path are PRIVATE. The circuit
/// proves "the bit at my hidden index in the tree rooted at `root` is UNSET" — so
/// a relying party learns the credential is live WITHOUT learning which list slot
/// it occupies (closing the clear-index linkability channel).
///
/// # Trust anchor (preserves the audit-#12 re-audit fix — load-bearing)
/// `root` is a prover-committed public input, NOT trusted as a prover claim. The
/// verifier derives the AUTHORITATIVE root from its OWN [`StatusListSnapshot`] for
/// the credential's (issuer-bound, freshness-checked) `(list, version)` — exactly
/// the snapshot [`bind_revocation`] uses, resolved out of band like the trusted
/// key-set `K` — at the policy's `hidden_index_depth`, and REQUIRES the proof's
/// public root to byte-equal it. A prover that proves bit-unset against a FORGED
/// all-zero tree (its own root) fails this equality: the liveness fact is bound to
/// the relying party's authenticated status bytes, never the prover's. (A
/// genuinely REVOKED credential cannot even produce the proof — the in-circuit
/// `bit == 0` assertion is unsatisfiable for a set bit — and additionally the
/// authoritative root over the relying party's snapshot, whose bit IS set, would
/// differ from any all-zero forgery.)
///
/// # Fail-closed
/// - No `manifest.hidden_revocation` => nothing to check here (the clear-index
///   [`bind_revocation`] remains the liveness gate); returns `Ok`.
/// - A proof present but the policy has NOT enabled the hidden-index path
///   (`hidden_index_depth == None`) => REJECT ([`CheckError::HiddenRevocationNotEnabled`]):
///   the verifier will not accept a root it cannot itself derive.
/// - A `depth` mismatch, an unresolvable authoritative root, a root mismatch, a
///   malformed blob, or a bb rejection all REJECT.
///
/// PRECONDITION: `bind_revocation` has already validated the issuer-bound
/// reference + freshness for `manifest.revocation`, so the `(list, version)` used
/// to resolve the authoritative snapshot here is the issuer's, not a prover claim.
///
/// # Scope (honest, see [`crate::revocation`])
/// [OPUS-4.8] sq-hwe: the authoritative root is now derived with the SPARSE
/// [`merkle_root`] builder (`O(set-bits * depth)`, independent of `2^depth`, with
/// roots BIT-IDENTICAL to the old dense fold), so the host side scales to
/// production list sizes (2^17+ slots) WITHOUT materialising `2^depth` leaves. The
/// remaining size bound is that only the `revoke_unset_d10` member (depth 10, up to
/// 1024 indices) is COMPILED — the circuit relation is depth-generic, so compiling
/// a deeper member is mechanical (tracked as follow-up). The soundness of the
/// binding (root equality + bb verify + in-circuit bit-unset) holds at any supported
/// depth. The status-list IRI + version are still disclosed in the clear (sq-6qe).
// [OPUS-4.8] sq-3e5 + sq-h2v: hidden-index revocation cryptographic gate.
fn bind_hidden_revocation(
    manifest: &ProofManifest,
    policy: &RevocationPolicy,
    prover: &CircuitProver,
    work_dir: &Path,
    challenge: &FieldHex,
) -> Result<(), CheckError> {
    let Some(hidden) = &manifest.hidden_revocation else {
        // No hidden-index proof; the clear-index path (bind_revocation) is the
        // liveness gate. Nothing to verify here.
        return Ok(());
    };
    // The relying party must have OPTED IN to the hidden-index path; otherwise it
    // has no authoritative root to bind the proof to and rejects fail-closed.
    let Some(depth) = policy.hidden_index_depth() else {
        return Err(CheckError::HiddenRevocationNotEnabled);
    };
    if hidden.depth != depth {
        return Err(CheckError::HiddenRevocationDepthMismatch {
            declared: hidden.depth,
            policy: depth,
        });
    }
    // The issuer-bound reference (validated by bind_revocation upstream) names the
    // (list, version) whose AUTHORITATIVE snapshot we derive the root from. Without
    // a reference there is no credential whose liveness is in question.
    let Some(rev) = &manifest.revocation else {
        return Ok(());
    };
    let Some(authoritative) = policy.authoritative_snapshot(&rev.status_list, rev.version) else {
        return Err(CheckError::HiddenRevocationRootUnavailable {
            status_list: rev.status_list.clone(),
            version: rev.version,
        });
    };
    // Derive the AUTHORITATIVE root from the relying party's OWN snapshot (the
    // trust anchor). This is what the proof's public root must equal.
    let Some(auth_root) = merkle_root(authoritative, depth) else {
        return Err(CheckError::HiddenRevocationRootUnavailable {
            status_list: rev.status_list.clone(),
            version: rev.version,
        });
    };
    // The proof's declared public root must byte-equal the authoritative root. A
    // malformed declared root fails closed.
    let Some(declared_root) = hidden.root.to_field() else {
        return Err(CheckError::HiddenRevocationRootMismatch);
    };
    if declared_root != auth_root {
        return Err(CheckError::HiddenRevocationRootMismatch);
    }

    // [OPUS-4.8] sq-ayv: the index commitment the proof cross-binds. The TRUST
    // ANCHOR is the ISSUER-SIGNED commitment in `manifest.revocation.index_commitment`
    // (validated under the issuer signature in `bind_issuer_attestations` /
    // `resolve_status_ref`), NOT the prover's declared `hidden.index_commitment`.
    // The `revoke_unset_d{depth}` member ALWAYS exposes an index_commitment public
    // input now (sq-ayv), so a hidden-revocation proof REQUIRES an issuer-signed
    // index commitment to bind it to — a hidden proof without one is rejected
    // fail-closed (it would be a free-floating bit-unset over an unbound index).
    let Some(rev_ic_hex) = &rev.index_commitment else {
        return Err(CheckError::HiddenRevocationIndexCommitmentMismatch);
    };
    let Some(auth_index_commitment) = rev_ic_hex.to_field() else {
        return Err(CheckError::HiddenRevocationIndexCommitmentMismatch);
    };
    // The proof's DECLARED public index commitment must byte-equal the issuer-signed
    // one (the prover cannot prove against a commitment the issuer did not sign).
    let Some(declared_ic) = hidden
        .index_commitment
        .as_ref()
        .and_then(|h| h.to_field())
    else {
        return Err(CheckError::HiddenRevocationIndexCommitmentMismatch);
    };
    if declared_ic != auth_index_commitment {
        return Err(CheckError::HiddenRevocationIndexCommitmentMismatch);
    }

    // Cryptographic gate: reconstruct the public inputs (challenge, root,
    // index_commitment) from the AUTHORITATIVE root + the ISSUER-SIGNED index
    // commitment (NOT the prover's declared bytes — byte-equal above, but we feed
    // our own to be end-to-end authentic), recompute the canonical vk, and bb
    // verify. The prover's bundled vk / public_inputs are never trusted (audit #2
    // discipline, mirrored here). The in-circuit cross-binding asserts the proof's
    // index_commitment is a commitment to the SAME index proven bit-unset, so the
    // index the issuer committed to is the one shown active.
    let blob = hex_decode(&hidden.proof_hex).ok_or(CheckError::HiddenRevocationMalformedProof)?;
    let art = decode_artifacts(&blob).ok_or(CheckError::HiddenRevocationMalformedProof)?;

    // Public-input layout for revoke_unset_d{depth} main: challenge, root,
    // index_commitment (three 32-byte BE field words, declaration order).
    let mut reconstructed: Vec<u8> = Vec::with_capacity(96);
    let challenge_fr = challenge
        .to_field()
        .ok_or(CheckError::HiddenRevocationMalformedProof)?;
    reconstructed.extend_from_slice(&field_to_be_bytes_32(&challenge_fr));
    reconstructed.extend_from_slice(&field_to_be_bytes_32(&auth_root));
    reconstructed.extend_from_slice(&field_to_be_bytes_32(&auth_index_commitment));
    if reconstructed != art.public_inputs {
        // The proof's committed public inputs (challenge + root + index_commitment)
        // do not match the verifier's nonce + authoritative root + issuer-signed
        // commitment. Diagnose the index-commitment word distinctly.
        let pi = &art.public_inputs;
        if pi.len() == 96 && pi[64..96] != field_to_be_bytes_32(&auth_index_commitment) {
            return Err(CheckError::HiddenRevocationIndexCommitmentMismatch);
        }
        return Err(CheckError::HiddenRevocationRootMismatch);
    }

    let id = CircuitId::RevokeUnset { depth };
    let sub_work = work_dir.join("hidden_revocation");
    let canonical_vk = prover
        .canonical_vk(&id, &sub_work.join("vk"))
        .map_err(CheckError::Driver)?;
    let ok = prover
        .verify_with(&art.proof, &reconstructed, &canonical_vk, &sub_work.join("verify"))
        .map_err(CheckError::Driver)?;
    if !ok {
        return Err(CheckError::HiddenRevocationProofRejected);
    }
    Ok(())
}

/// Stage 3b (sq-z9l): the HIDDEN-ISSUER attestation cryptographic gate — the
/// privacy upgrade over [`bind_issuer_attestations`]'s clear-key check. Runs only
/// the cryptographic gate for `manifest.hidden_issuer_attestations`; the
/// structural pre-conditions (the issuer-bound reference + salt + clear-key
/// attestation) are still enforced by the prefilter.
///
/// # What it proves and what stays hidden
/// Each entry supplies a `hidden_issuer_d{depth}` bb proof whose PUBLIC inputs are
/// the verifier's `challenge`, the commitment message `m`, and the key-set Merkle
/// `key_set_root`; the issuer key, the signature `(R, s)`, the challenge-reduction
/// witness, and the membership index/path are PRIVATE. The circuit proves "this
/// `m` was signed by SOME issuer whose key is in the tree rooted at `key_set_root`"
/// — so the relying party learns a TRUSTED authority vouched for the commitment
/// WITHOUT learning WHICH (closing the clear-key deanonymisation channel).
///
/// # Trust anchor (preserves the audit #3 external-K anchor — load-bearing)
/// `key_set_root` is a prover-committed public input, NOT trusted as a claim. The
/// verifier derives the AUTHORITATIVE root from its OWN [`KeySet`] (canonical
/// order, the same external anchor `bind_issuer_attestations` checks membership
/// against) at the policy's `hidden_issuer_depth`, and REQUIRES the proof's public
/// root to byte-equal it. A prover that proves membership in its OWN (forged) key
/// set fails this equality. And `m` is recomputed from the disclosed
/// commitment + salt + status reference (the SAME issuer-signed message the clear
/// path binds), so the proof is tied to a specific committed graph the relying
/// party can name — not a free-floating "some signature exists".
///
/// # Fail-closed
/// - No entries => nothing to check (the clear-key [`bind_issuer_attestations`]
///   remains the attestation gate); returns `Ok`.
/// - An entry present but the KeySet has NOT enabled the hidden-issuer path
///   (`hidden_issuer_depth == None`) => REJECT ([`CheckError::HiddenIssuerNotEnabled`]).
/// - A depth mismatch, an unresolvable root, a root mismatch, a message mismatch,
///   an unreferenced commitment, a malformed blob, or a bb rejection all REJECT.
///
/// PRECONDITION: `bind_issuer_attestations` + `bind_revocation` have already run
/// in the prefilter, so `manifest.revocation` is the ISSUER's (bound) reference
/// and the per-commitment salt is the issuer-attested one — the message recomputed
/// here is therefore the genuine issuer-signed message.
// [OPUS-4.8] sq-z9l: hidden-issuer attestation cryptographic gate.
fn bind_hidden_issuer_attestations(
    manifest: &ProofManifest,
    trusted_key_set: &KeySet,
    prover: &CircuitProver,
    work_dir: &Path,
    challenge: &FieldHex,
) -> Result<(), CheckError> {
    if manifest.hidden_issuer_attestations.is_empty() {
        // No hidden-issuer proofs; the clear-key path is the attestation gate.
        return Ok(());
    }
    // The relying party must have OPTED IN; otherwise it has no authoritative
    // key-set root to bind the proof to and rejects fail-closed.
    let Some(depth) = trusted_key_set.hidden_issuer_depth() else {
        return Err(CheckError::HiddenIssuerNotEnabled);
    };
    // Derive the AUTHORITATIVE key-set root from the relying party's OWN KeySet
    // (canonical order) — the trust anchor every entry's public root must equal.
    let auth_root = trusted_key_set
        .hidden_issuer_root(depth)
        .ok_or(CheckError::HiddenIssuerRootUnavailable)?;

    let challenge_fr = challenge
        .to_field()
        .ok_or(CheckError::HiddenIssuerMalformedProof)?;

    // The set of commitments a VERIFIED scan sub-proof references, with the
    // issuer-signed message recomputed for each (commitment hex -> message Fr).
    // Mirrors bind_issuer_attestations' referenced-commitment discipline: the
    // message is `commitment_message_with_status(C(G), salt, status_ref)`, exactly
    // the clear path's signed message, recomputed from the disclosed (and, by the
    // prefilter, issuer-bound) reference + salt.
    let referenced = scan_referenced_messages(manifest)?;

    for (i, hi) in manifest.hidden_issuer_attestations.iter().enumerate() {
        if hi.depth != depth {
            return Err(CheckError::HiddenIssuerDepthMismatch {
                declared: hi.depth,
                policy: depth,
            });
        }
        // The covered commitment must be referenced by a verified scan, and we use
        // OUR recomputed message (never the prover's declared `hi.message`).
        let Some(c_fr) = hi.commitment.to_field() else {
            return Err(CheckError::HiddenIssuerUnreferencedCommitment {
                commitment: hi.commitment.0.clone(),
            });
        };
        let c_key = field_to_hex(&c_fr);
        let Some(expected_m) = referenced.get(&c_key) else {
            return Err(CheckError::HiddenIssuerUnreferencedCommitment {
                commitment: hi.commitment.0.clone(),
            });
        };

        let blob = hex_decode(&hi.proof_hex).ok_or(CheckError::HiddenIssuerMalformedProof)?;
        let art = decode_artifacts(&blob).ok_or(CheckError::HiddenIssuerMalformedProof)?;

        // Public-input layout for hidden_issuer_d{depth} main: challenge, m,
        // key_set_root (three 32-byte BE field words, declaration order). We feed
        // OUR own values (nonce, recomputed message, authoritative root) — never
        // the prover's declared bytes — so a different challenge, a message not
        // bound to a referenced commitment, or a non-authoritative key set all fail
        // the byte-compare.
        let mut reconstructed: Vec<u8> = Vec::with_capacity(96);
        reconstructed.extend_from_slice(&field_to_be_bytes_32(&challenge_fr));
        reconstructed.extend_from_slice(&field_to_be_bytes_32(expected_m));
        reconstructed.extend_from_slice(&field_to_be_bytes_32(&auth_root));
        if reconstructed != art.public_inputs {
            // Diagnose WHICH public input diverged for a precise reason. The
            // proof's first word is the challenge (already bound elsewhere); the
            // second is the message, the third the key-set root.
            let pi = &art.public_inputs;
            if pi.len() == 96 && pi[64..96] != field_to_be_bytes_32(&auth_root) {
                return Err(CheckError::HiddenIssuerRootMismatch);
            }
            if pi.len() == 96 && pi[32..64] != field_to_be_bytes_32(expected_m) {
                return Err(CheckError::HiddenIssuerMessageMismatch {
                    commitment: hi.commitment.0.clone(),
                });
            }
            // Otherwise the challenge word (or the blob length) diverged.
            return Err(CheckError::HiddenIssuerProofRejected);
        }

        let id = CircuitId::HiddenIssuer { depth };
        let sub_work = work_dir.join(format!("hidden_issuer_{i}"));
        let canonical_vk = prover
            .canonical_vk(&id, &sub_work.join("vk"))
            .map_err(CheckError::Driver)?;
        let ok = prover
            .verify_with(&art.proof, &reconstructed, &canonical_vk, &sub_work.join("verify"))
            .map_err(CheckError::Driver)?;
        if !ok {
            return Err(CheckError::HiddenIssuerProofRejected);
        }
    }
    Ok(())
}

/// The per-graph salt the verifier uses to recompute a scan commitment's
/// issuer-signed message, resolved from EITHER a clear [`CommitmentAttestation`]
/// over `c_fr` OR — for a HIDDEN-ONLY commitment (sq-xxg) — the
/// [`HiddenIssuerAttestation`]'s own `salt`. The clear attestation's salt is
/// preferred when present (the additive mode); the hidden entry's salt is the
/// fallback so a commitment with no clear attestation can still have its `m`
/// recomputed. `None` if neither source supplies a parseable salt.
// [OPUS-4.8] sq-xxg: salt source for hidden-only message reconstruction.
fn resolve_commitment_salt(manifest: &ProofManifest, c_fr: &Fr) -> Option<Fr> {
    // Prefer the clear attestation's salt (the original sq-z9l additive path).
    if let Some(att) = manifest.commitment_attestations.iter().find(|a| {
        a.commitment.to_field().is_some() && a.commitment.to_field() == Some(*c_fr)
    }) {
        if let Some(salt_hex) = &att.salt {
            if let Some(salt_fr) = salt_hex.to_field() {
                return Some(salt_fr);
            }
        }
    }
    // Fall back to a hidden-issuer entry's salt (the hidden-only case).
    manifest
        .hidden_issuer_attestations
        .iter()
        .find(|hi| hi.commitment.to_field() == Some(*c_fr))
        .and_then(|hi| hi.salt.as_ref())
        .and_then(|salt_hex| salt_hex.to_field())
}

/// The issuer-signed message for every commitment a VERIFIED scan sub-proof
/// references, keyed by canonical commitment hex (sq-z9l). The message is
/// `commitment_message_with_status(C(G), salt, status_ref_digest(H(list), index,
/// version))` — the SAME message [`bind_issuer_attestations`] verifies the clear
/// signature over, recomputed from the disclosed (prefilter-validated, so
/// issuer-bound) salt + revocation reference. The salt is resolved via
/// [`resolve_commitment_salt`], so a HIDDEN-ONLY commitment (no clear attestation,
/// salt carried on the hidden entry — sq-xxg) is also covered. Used by the
/// hidden-issuer gate to bind each proof's PUBLIC `m` to a specific committed
/// graph.
fn scan_referenced_messages(
    manifest: &ProofManifest,
) -> Result<std::collections::BTreeMap<String, Fr>, CheckError> {
    let mut out: std::collections::BTreeMap<String, Fr> = std::collections::BTreeMap::new();
    let Some(rev) = &manifest.revocation else {
        // No reference => no status-bound message can be formed. The hidden-issuer
        // path requires the same issuer-bound reference the clear path does.
        return Ok(out);
    };
    let list_id_fr = status_list_id_to_field(&rev.status_list);
    // [OPUS-4.8] sq-ayv: derive the status reference in the mode the credential
    // uses (clear index OR committed index), matching the message
    // `bind_issuer_attestations` verified the issuer signature over.
    let Some(status_ref) = status_ref_from_revocation(rev, &list_id_fr) else {
        return Ok(out);
    };
    for sp in &manifest.sub_proofs {
        let ProofInputs::Scan { commitments, .. } = &sp.inputs else {
            continue;
        };
        for c in commitments {
            let Some(c_fr) = c.to_field() else { continue };
            let Some(salt_fr) = resolve_commitment_salt(manifest, &c_fr) else { continue };
            let m = commitment_message_with_status(&c_fr, &salt_fr, &status_ref);
            out.insert(field_to_hex(&c_fr), m);
        }
    }
    Ok(out)
}

/// Derive the issuer-signed `status_ref` field element from the DISCLOSED
/// revocation reference's index-disclosure mode (sq-ayv): clear index =>
/// [`status_ref_digest`]; committed index => [`status_ref_commit_digest`]. `None`
/// if the reference's mode is malformed (neither/both set, or an unparseable
/// commitment). The disclosed reference has ALREADY been cross-checked against the
/// issuer signature by `bind_issuer_attestations`, so deriving from it here yields
/// the genuine issuer-signed message.
// [OPUS-4.8] sq-ayv.
fn status_ref_from_revocation(
    rev: &crate::manifest::RevocationStatus,
    list_id_fr: &Fr,
) -> Option<Fr> {
    match (rev.index, &rev.index_commitment) {
        (Some(index), None) => Some(status_ref_digest(list_id_fr, index, rev.version)),
        (None, Some(ic_hex)) => ic_hex
            .to_field()
            .map(|ic| status_ref_commit_digest(list_id_fr, &ic, rev.version)),
        _ => None,
    }
}

/// Normalize a hex key for set membership: strip an optional `0x` prefix and
/// lowercase, so K-membership is representation-insensitive.
fn normalize_hex(h: &str) -> String {
    h.strip_prefix("0x").unwrap_or(h).to_ascii_lowercase()
}

/// Encode a query pattern's constant term to its `pattern_const_enc` field-hex
/// (salt-independent for IRIs/literals — only bnodes use the salt, and query
/// constants are never bnodes; matches `build::encode_slot` / the in-circuit
/// `pattern_const_enc`). Variable slots encode as `0x0`.
fn encode_pattern_slot(c: &Option<oxrdf::Term>) -> Option<FieldHex> {
    match c {
        Some(t) => encode_term(t, &Fr::from(0u64)).map(|f| FieldHex(field_to_hex(&f))),
        None => Some(FieldHex("0x0".to_string())),
    }
}

/// Whether a scan sub-proof's bound pattern constancy/encoding matches a query
/// pattern's constant slots. This is the verifier-side equality over the
/// bb-bound `pattern_is_const`/`pattern_const_enc` (audit #10): a scan over
/// `<hasAge>` cannot answer a query pattern over `<hasSalary>` (constant-swap),
/// and a variable slot must be a variable on both sides.
fn scan_matches_pattern(inputs: &ProofInputs, consts: &[Option<oxrdf::Term>; 3]) -> bool {
    let (is_const, const_enc) = match inputs {
        ProofInputs::Scan { pattern_is_const, pattern_const_enc, .. } => {
            (pattern_is_const, pattern_const_enc)
        }
        _ => return false,
    };
    for slot in 0..3 {
        let q_is_const = consts[slot].is_some();
        if q_is_const != is_const[slot] {
            return false;
        }
        // For a constant slot, the bound encoding must equal the query
        // constant's encoding; for a variable slot both carry the `0x0` filler
        // (still checked, so a non-`0x0` variable slot mismatches).
        match encode_pattern_slot(&consts[slot]) {
            Some(enc) if enc == const_enc[slot] => {}
            _ => return false,
        }
    }
    true
}

/// Stage 2b/2c: bind every query BGP pattern's constants to a scan sub-proof
/// (audit #10) and every query FILTER to a slot-bound, true-verdict
/// `filter_int` sub-proof reached via a binding edge (audit #5/#6/#7).
///
/// FILTER var→slot mapping: a query FILTER `?v op c` is satisfied iff there is
/// a binding edge `(from_proof=scan, from_row, from_slot, to_proof=filter)`
/// such that (1) the scan answers the query pattern in which `?v` binds (its
/// constants match, `scan_matches_pattern`); (2) `from_slot` is exactly the
/// slot `?v` occupies in that pattern (audit #6 — closes "point the operand at
/// the salary slot for an age filter"); (3) the filter's bound `(op, bound)`
/// EQUAL the query's `(op, c)` (audit #5 — closes the 17-vs-`>=18`
/// comparison-substitution); and (4) the filter's `expected == true` (audit
/// #5/#6 — the verdict gates row inclusion; an `expected==false` row may not be
/// presented as passing).
///
/// A FILTER with no such edge ⇒ REJECT (audit #10 FILTER-add / a `filter_int`
/// over the wrong operand). Stage 2 already enforced the edge's scanned-slot
/// encoding equals the filter `operand_enc` (so #7 operand substitution is
/// closed by that equality over the now-bb-bound values + this slot check).
fn bind_query_correctness(manifest: &ProofManifest) -> Result<(), CheckError> {
    let patterns = fragment_patterns(&manifest.query)?;
    let consts = fragment_pattern_consts(&patterns);
    let filters = fragment_filters(&manifest.query)?;
    let var_slots = variable_slots(&patterns);

    // (2b) every query BGP pattern's constants are bound by SOME scan sub-proof.
    for (pi, c) in consts.iter().enumerate() {
        let bound = manifest
            .sub_proofs
            .iter()
            .any(|sp| scan_matches_pattern(&sp.inputs, c));
        if !bound {
            return Err(CheckError::UnboundPattern { pattern: pi });
        }
    }

    // (2c) every query FILTER has a matching, slot-bound, true-verdict proof for
    // EVERY active disclosed row of EVERY scan that answers the FILTER's pattern.
    // Per-row gating is the load-bearing part of #5/#6: the disclosed result is
    // the scans' rows, so a FILTER row whose verdict is false must be EXCLUDED —
    // i.e. every active row of a filtered pattern must carry a true-verdict
    // filter_int sub-proof over that row's operand slot. A single missing/false
    // row makes the FILTER unproven for the disclosed set => REJECT.
    for QueryFilter { variable, op, bound } in &filters {
        // The (pattern, slot) positions ?variable binds to. A FILTER over a
        // variable that never binds to a scanned column is unmappable.
        let positions: Vec<(usize, usize)> = var_slots
            .iter()
            .filter(|(v, _, _)| v == variable)
            .map(|(_, p, s)| (*p, *s))
            .collect();
        if positions.is_empty() {
            return Err(CheckError::UnmappableFilterVar { variable: variable.clone() });
        }
        // The FILTER must constrain at least one disclosed row (else it is
        // vacuously "satisfied" by an empty result while the query carries a
        // FILTER the prover never proved). Track that some scan answered the
        // pattern AND every one of its active rows is gated true.
        let mut any_scan_answered = false;
        for (spi, sp) in manifest.sub_proofs.iter().enumerate() {
            let (rows, row_count) = match &sp.inputs {
                ProofInputs::Scan { rows, row_count, .. } => (rows, *row_count as usize),
                _ => continue,
            };
            // Is this scan the one that answers a pattern ?v binds in, and at
            // which slot does ?v sit there?
            let slot = positions.iter().find_map(|(pi, si)| {
                consts
                    .get(*pi)
                    .filter(|c| scan_matches_pattern(&sp.inputs, c))
                    .map(|_| *si)
            });
            let Some(slot) = slot else { continue };
            any_scan_answered = true;
            // Every ACTIVE disclosed row must have a true-verdict filter_int
            // edge at this slot with matching (op, bound).
            for row in 0..row_count.min(rows.len()) {
                let gated = manifest.binding_edges.iter().any(|edge| {
                    edge.from_proof == spi
                        && edge.from_row == row
                        && edge.from_slot == slot
                        && filter_edge_true(manifest, edge.to_proof, *op, *bound)
                });
                if !gated {
                    return Err(CheckError::UnboundFilter { variable: variable.clone() });
                }
            }
        }
        if !any_scan_answered {
            // A FILTER whose variable binds in a pattern, but no scan answers
            // that pattern: the FILTER cannot be discharged (FILTER-add on a
            // manifest missing the filtered pattern's scan).
            return Err(CheckError::UnboundFilter { variable: variable.clone() });
        }
    }
    Ok(())
}

/// Stage 2e: bind the prover's `manifest.attributions` (which drives the Q6
/// cross-graph-bnode-join obligation gate in stage 1a) to the PROOF-BOUND
/// per-graph attribution each scan sub-proof carries (audit #8).
///
/// For each query BGP pattern `pi`, find the scan sub-proof that answers it
/// (constants match, `scan_matches_pattern`) and require
/// `manifest.attributions[pi]` to be a SUPERSET of that scan's proof-bound
/// matched-graph set (`attribution[g] == true`). Soundness:
/// - **Under-declaring** (the `[[0],[0]]` forge): a graph the scan proved a
///   contribution from but `manifest.attributions[pi]` omits is rejected. This
///   is the load-bearing #8 fix — the prover can no longer shrink the
///   attribution set below the truth to drop a cross-graph obligation.
/// - **Over-declaring** is conservative-safe: extra graphs in
///   `manifest.attributions[pi]` only widen `|A_i ∪ A_j|`, demanding MORE
///   non-bnode obligations (the coarser-is-safe direction, per `verify.rs`
///   module docs). So a superset relation, not equality, is the correct gate —
///   it preserves the legitimate "this pattern MAY draw from these graphs"
///   semantics while forbidding the dishonest narrowing.
///
/// The proof-bound attribution is byte-checked against the bb public inputs in
/// stage 3 (audit #1 reconstruction). Here in the structural stage we cross-check
/// the SAME declared `attribution` field that the reconstruction will bind, so a
/// structure-only verify already rejects the forge and the full verify rejects it
/// twice (structurally + cryptographically).
// [OPUS-4.8] audit #8.
fn bind_attributions(manifest: &ProofManifest) -> Result<(), CheckError> {
    let patterns = fragment_patterns(&manifest.query)?;
    let consts = fragment_pattern_consts(&patterns);

    for (pi, c) in consts.iter().enumerate() {
        let declared: BTreeSet<usize> = manifest
            .attributions
            .get(pi)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default();

        // Find a scan sub-proof that answers this pattern and read its
        // proof-bound attribution. (Stage 2b already guarantees one exists.)
        let mut matched_a_scan = false;
        for sp in &manifest.sub_proofs {
            let ProofInputs::Scan { attribution, .. } = &sp.inputs else {
                continue;
            };
            if !scan_matches_pattern(&sp.inputs, c) {
                continue;
            }
            matched_a_scan = true;
            // Every graph the scan PROVED a contribution from must be declared.
            for (g, &bit) in attribution.iter().enumerate() {
                if bit && !declared.contains(&g) {
                    return Err(CheckError::AttributionUnderDeclared {
                        pattern: pi,
                        proof_graph: g,
                    });
                }
            }
        }
        if !matched_a_scan {
            return Err(CheckError::AttributionUnbound { pattern: pi });
        }
    }
    Ok(())
}

/// Whether sub-proof `to_proof` is a `filter_int` whose bound `(op, bound)`
/// match the query FILTER and whose disclosed verdict is TRUE (audit #5/#6:
/// op/bound substitution + verdict gating). The operand-slot equality (the edge
/// references the FILTER variable's scanned column) is enforced by the caller;
/// the edge's scanned-slot encoding == the filter `operand_enc` is enforced by
/// stage 2 over the now-bb-bound values (audit #7).
fn filter_edge_true(
    manifest: &ProofManifest,
    to_proof: usize,
    op: FilterCmp,
    bound: u64,
) -> bool {
    match manifest.sub_proofs.get(to_proof).map(|sp| &sp.inputs) {
        Some(ProofInputs::FilterInt { op: f_op, bound: f_bound, expected, .. }) => {
            f_op.code() == op.code() && *f_bound == bound && *expected
        }
        _ => false,
    }
}

/// sq-cwq: verify the holder proof-of-possession when the binding is `HolderPop`.
/// Runs ONLY for a `HolderPop` binding (a `Challenge` binding returns `Ok(())`
/// immediately — no PoP is required). `challenge` is the VERIFIER'S nonce (the
/// same field element bound as public-input field 0 of every sub-proof), so the
/// PoP is fresh: a captured manifest re-presented under a new nonce cannot reuse
/// an old PoP, and the holder must sign THIS verifier's challenge.
///
/// Fail-closed contract (the sq-cwq fix — no silent accept of an absent PoP):
/// 1. an EMPTY `holder_registry` => `HolderRegistryEmpty` (no trust anchor — a
///    holder PoP cannot be accepted; the relying party must supply authorised
///    holder keys to use holder binding);
/// 2. `holder` not a member of the registry => `HolderNotTrusted`;
/// 3. an unknown `cryptosuite`, or a `holder`/`pop` that does not parse =>
///    `HolderPopMalformed` (prover-controlled bytes never panic);
/// 4. `pop` not a valid signature under `holder` over
///    `holder_pop_message(challenge)` => `HolderPopInvalid`.
///
/// The previous placeholder accepted a `HolderPop` binding by simply reading its
/// `challenge` (exactly like a bare `Challenge`), silently waiving (1)-(4) — an
/// absent/forged PoP passed. That silent-accept path is removed.
///
/// # Scope (honest deferral)
/// This proves the presenter possesses a holder key the relying party trusts AND
/// signed the verifier's fresh nonce with it. It does NOT bind that key to the
/// SPECIFIC credential the scan/filter sub-proofs attest — an issuer-attested
/// holder binding (the issuer signing the holder key into the credential, e.g. a
/// `holderBinding` field folded into the commitment message) is DEFERRED. Without
/// it, a trusted holder A could present a trusted holder B's credential. The
/// holder registry narrows "who may present at all" to authorised holders, which
/// is the meaningful interim guarantee; the per-credential binding is the
/// documented next step. This is NOT a silent gap: the registry is fail-closed and
/// the deferral is recorded here and in the manifest/README docs.
// [OPUS-4.8] sq-cwq: holder PoP implemented (challenge-bound Schnorr) + fail-closed.
fn bind_holder_pop(
    manifest: &ProofManifest,
    holder_registry: &HolderRegistry,
    challenge: &FieldHex,
) -> Result<(), CheckError> {
    let (holder, pop, cryptosuite) = match &manifest.binding {
        // A plain challenge binding requires no holder PoP.
        BindingMode::Challenge { .. } => return Ok(()),
        BindingMode::HolderPop { holder, pop, cryptosuite, .. } => (holder, pop, cryptosuite),
    };

    // (1) No trust anchor => cannot accept a holder PoP (fail-closed). This is the
    // load-bearing replacement for the old silent-accept: a HolderPop binding is
    // NEVER waived just because no registry was supplied.
    if holder_registry.is_empty() {
        return Err(CheckError::HolderRegistryEmpty);
    }
    // (2) The holder key must be authorised by the relying party's external set.
    if !holder_registry.contains_hex(holder) {
        return Err(CheckError::HolderNotTrusted { holder: holder.clone() });
    }
    // (3) Known cryptosuite + parseable key/signature (fail-closed on bad bytes).
    if SignatureScheme::from_cryptosuite_iri(cryptosuite).is_none() {
        return Err(CheckError::HolderPopMalformed);
    }
    let (Some(pk), Some(sig)) = (public_key_from_hex(holder), signature_from_hex(pop)) else {
        return Err(CheckError::HolderPopMalformed);
    };
    // The challenge must parse to a field element (it equals the verifier nonce,
    // already validated in verify_manifest, but check here for a standalone caller).
    let Some(challenge_fr) = challenge.to_field() else {
        return Err(CheckError::HolderPopMalformed);
    };
    // (4) The PoP signature must verify under the holder key over the
    // challenge-bound, domain-separated PoP message — proving possession of the
    // holder secret, freshly over the verifier's nonce.
    let message = holder_pop_message(&challenge_fr);
    if !sig_verify(&pk, &message, &sig) {
        return Err(CheckError::HolderPopInvalid { holder: holder.clone() });
    }
    Ok(())
}

/// Encode an IRI to its salt-independent term encoding `FieldHex` (the same
/// encoding scans disclose), `None` if it does not encode. Used by
/// [`bind_entailment`] to compute the RDFS schema-vocabulary encodings.
// [OPUS-4.8] sq-314.
fn encode_iri_hex(iri: &str) -> Option<FieldHex> {
    let nn = oxrdf::NamedNode::new(iri).ok()?;
    let enc = encode_term(&oxrdf::Term::NamedNode(nn), &Fr::from(0u64))?;
    Some(FieldHex(field_to_hex(&enc)))
}

/// sq-314: enforce the manifest's entailment regime end-to-end. This makes
/// `entailment_regime` a CHECKED claim rather than free metadata.
///
/// Fail-closed contract:
/// 1. the regime MUST be accepted by the relying party's [`EntailmentPolicy`]
///    (`Simple` always; `Rdfs`/`Owl` only on explicit opt-in) — else
///    `EntailmentRegimeNotAccepted`;
/// 2. a `Simple` manifest MUST carry NO derivation steps (no inference) — else
///    `UnexpectedDerivationSteps`;
/// 3. a non-`Simple` manifest MUST carry derivation steps — else
///    `MissingDerivationSteps`;
/// 4. every derivation step MUST be a well-formed, regime-admitted rule instance
///    (`MalformedDerivationStep`), and every antecedent MUST be GROUNDED: equal to
///    an EARLIER step's derived triple or to a triple disclosed by a scan
///    sub-proof (the asserted base) — else `UngroundedDerivationAntecedent`.
///
/// Grounding is by TERM-ENCODING equality (the disclosed scan rows and the step
/// triples are both `FieldHex` encodings), so an antecedent that equals a
/// disclosed asserted triple is soundly grounded. Step ordering is significant:
/// each step may only ground on STRICTLY EARLIER steps (a forward-chained
/// derivation), so there are no cyclic self-justifications.
///
/// # Honest scope (what is NOT proved here — deferred)
/// Grounding ties antecedents to the DISCLOSED base; it does NOT (yet) prove in
/// zero-knowledge that an undisclosed antecedent is in the committed graph's
/// closure. An antecedent that is neither disclosed nor chained to a disclosed
/// triple is REJECTED (not assumed). The full in-circuit RDFS/OWL-RL closure proof
/// is the inference-circuit deliverable; this stage makes the regime claim
/// non-vacuous and auditable over the disclosed-base fragment, fail-closed.
// [OPUS-4.8] sq-314: entailment regime + derivation steps, end-to-end.
fn bind_entailment(
    manifest: &ProofManifest,
    policy: &EntailmentPolicy,
) -> Result<(), CheckError> {
    let regime = manifest.entailment_regime;
    let regime_name = match regime {
        EntailmentRegime::Simple => "simple",
        EntailmentRegime::Rdfs => "rdfs",
        EntailmentRegime::Owl => "owl",
    };
    // (1) The regime must be accepted by the relying party.
    if !policy.accepts(regime) {
        return Err(CheckError::EntailmentRegimeNotAccepted { regime: regime_name });
    }
    let steps = &manifest.derivation_steps;
    // (2) Simple => no inference steps.
    if regime == EntailmentRegime::Simple {
        if !steps.is_empty() {
            return Err(CheckError::UnexpectedDerivationSteps);
        }
        return Ok(());
    }
    // (3) Non-Simple => steps required.
    if steps.is_empty() {
        return Err(CheckError::MissingDerivationSteps { regime: regime_name });
    }

    // The asserted base: every triple disclosed by a scan sub-proof (active rows),
    // as [s, p, o] encodings — the ground set a step may chain from.
    let mut disclosed: BTreeSet<[String; 3]> = BTreeSet::new();
    for sp in &manifest.sub_proofs {
        if let ProofInputs::Scan { rows, row_count, .. } = &sp.inputs {
            for row in rows.iter().take(*row_count as usize) {
                disclosed.insert([
                    normalize_hex(&row[0].0),
                    normalize_hex(&row[1].0),
                    normalize_hex(&row[2].0),
                ]);
            }
        }
    }

    // RDFS schema-vocabulary encodings (salt-independent IRIs). If any fails to
    // encode the whole check fails closed (it cannot happen for these constants).
    let (Some(rdf_type), Some(rdfs_subclassof), Some(rdfs_subpropertyof)) = (
        encode_iri_hex("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"),
        encode_iri_hex("http://www.w3.org/2000/01/rdf-schema#subClassOf"),
        encode_iri_hex("http://www.w3.org/2000/01/rdf-schema#subPropertyOf"),
    ) else {
        return Err(CheckError::MalformedDerivationStep { step: 0 });
    };

    // (4) Re-check each step. `derived_so_far` accumulates the derived triples of
    // EARLIER steps (forward chaining only — no cyclic self-grounding).
    let mut derived_so_far: BTreeSet<[String; 3]> = BTreeSet::new();
    for (si, step) in steps.iter().enumerate() {
        // 4a. Well-formed rule instance AND the regime admits the rule.
        if !regime_admits(regime, step.rule)
            || !step.is_well_formed(&rdf_type, &rdfs_subclassof, &rdfs_subpropertyof)
        {
            return Err(CheckError::MalformedDerivationStep { step: si });
        }
        // 4b. Every antecedent grounded (disclosed base OR an earlier derived).
        for (ai, ant) in step.antecedents.iter().enumerate() {
            let key = [
                normalize_hex(&ant[0].0),
                normalize_hex(&ant[1].0),
                normalize_hex(&ant[2].0),
            ];
            if !disclosed.contains(&key) && !derived_so_far.contains(&key) {
                return Err(CheckError::UngroundedDerivationAntecedent {
                    step: si,
                    antecedent: ai,
                });
            }
        }
        // This step's derived triple is now available to ground later steps.
        derived_so_far.insert([
            normalize_hex(&step.derived[0].0),
            normalize_hex(&step.derived[1].0),
            normalize_hex(&step.derived[2].0),
        ]);
    }
    Ok(())
}

/// Full verification: structure (stage 1+2) then the cryptographic gate
/// (stage 3). `prover` points at the `zk/compose/` workspace; `work_dir` is
/// scratch for bb artifacts; `trusted_key_set` is the relying party's EXTERNAL
/// issuer trust anchor `K` (audit #3 codex #1 — never the prover's
/// `manifest.key_set`); `revocation_policy` is the relying party's EXTERNAL
/// freshness/revocation policy (audit #12 — the status check is mandatory: a
/// revoked credential, a stale status snapshot, or an omitted/forged
/// issuer-bound status reference all REJECT); `holder_registry` is the relying
/// party's EXTERNAL set of authorised holder keys (sq-cwq) — consulted ONLY when
/// `manifest.binding` is `HolderPop`, in which case the holder MUST prove
/// possession of a registry-member key by signing the verifier nonce (an empty
/// registry, an untrusted holder, or a malformed/invalid PoP all REJECT; a
/// `Challenge` binding ignores the registry). Pass `&HolderRegistry::empty()`
/// when holder binding is not in use. `entailment_policy` is the relying party's
/// EXTERNAL entailment-regime policy (sq-314): which regimes it accepts. The
/// regime is enforced fail-closed — a regime the policy rejects, or a non-`Simple`
/// regime whose `derivation_steps` do not structurally ground every derived triple
/// to the disclosed base, REJECTS. Pass `&EntailmentPolicy::simple_only()` (the
/// default) to accept only non-inference proofs.
///
/// # Freshness / single-use (audit #4) — the challenge-response flow
/// `nonce` is the relying party's OWN fresh value (a [`VerifierNonce`]), minted
/// out of band and handed to the prover BEFORE proving. The honest flow is a
/// three-step challenge-response:
/// 1. **Verifier → prover:** the relying party mints a fresh `nonce` and sends
///    it. (Unpredictability/uniqueness is the relying party's responsibility; a
///    CSPRNG value reduced into the field is the expected source.)
/// 2. **Prover:** proves with `nonce` as the circuit `challenge` public input
///    (field 0) — the prove path threads it through `toml.rs` / `build.rs`, so
///    NO circuit change is needed (field 0 is an unconstrained in-circuit tag;
///    the binding is verifier-side).
/// 3. **Verifier:** this function (a) records the nonce single-use via `seen`
///    BEFORE the crypto gate (a second presentation of the same nonce =>
///    [`CheckError::NonceReplay`]); (b) asserts `manifest.binding`'s declared
///    challenge equals `nonce` (fail-closed JSON consistency); and (c)
///    reconstructs the public-input vector using `nonce` (NOT `manifest.binding`)
///    as field 0, so the audit-#1 byte-compare rejects any proof whose committed
///    challenge ≠ the verifier's nonce. A CAPTURED manifest re-presented under a
///    NEW verifier nonce therefore fails the byte-compare (its proof committed
///    the OLD nonce); the same manifest re-presented under its ORIGINAL nonce
///    fails the single-use store. Both replay vectors are closed fail-closed —
///    there is no "skip when a binding field is absent" path.
///
/// Stage 3, per sub-proof, binds the declared statement to the proof (audit
/// #1/#2): (a) reconstruct the public-input byte vector from the DECLARED
/// `ProofInputs` using the VERIFIER'S NONCE as field 0 and assert byte-equality
/// with the proof's `public_inputs`; (b) recompute the CANONICAL member vk
/// verifier-side; (c) `bb verify` over (prover proof, reconstructed public
/// inputs, canonical vk). The prover-supplied vk and public-input bytes from
/// the blob are NEVER trusted.
// [OPUS-4.8] audit #4: verifier-issued nonce + single-use.
// [OPUS-4.8] sq-cwq: + holder_registry (external holder trust anchor). Each
// argument is a DISTINCT external trust input the relying party supplies (issuer
// key-set K, revocation policy, holder registry, fresh nonce, single-use store) —
// deliberately separate, not bundled, to keep each anchor explicit at the call
// site; the count exceeds clippy's default heuristic but every arg is load-bearing.
#[allow(clippy::too_many_arguments)]
pub fn verify_manifest(
    manifest: &ProofManifest,
    prover: &CircuitProver,
    work_dir: &Path,
    trusted_key_set: &KeySet,
    revocation_policy: &RevocationPolicy,
    holder_registry: &HolderRegistry,
    entailment_policy: &EntailmentPolicy,
    nonce: &VerifierNonce,
    seen: &dyn SeenNonces,
) -> Result<(), CheckError> {
    prefilter_manifest_structure(manifest, trusted_key_set, revocation_policy)?;

    // --- sq-314: entailment regime + derivation steps (fail-closed). ---
    // Enforce `manifest.entailment_regime` against the relying party's policy so
    // it is a CHECKED claim, not free metadata: a regime the policy rejects, a
    // Simple manifest carrying inference steps, a non-Simple manifest with no
    // steps, or a derivation step that is malformed / ungrounded all REJECT. This
    // is a pure-JSON structural check (no bb); placed before the nonce burn since
    // it is independent of the crypto gate.
    bind_entailment(manifest, entailment_policy)?;

    // --- Audit #4: single-use (fail-closed, BEFORE the crypto gate). ---
    // Record the verifier's nonce as used; reject if it was already seen. Doing
    // this first means a replayed (nonce, manifest) pair is rejected without
    // even running bb. The store is consulted unconditionally — there is no
    // opt-out path that could bypass single-use (the parameter is mandatory).
    //
    // [OPUS-4.8] sq-3v2 — BURN-ON-MISMATCH is intentional. Because this records
    // FIRST, the verifier nonce is CONSUMED even when the manifest is subsequently
    // rejected (NonceBindingMismatch, MalformedProof, a bb failure, …). That is the
    // chosen freshness/replay policy: a verifier-issued nonce is spent the moment it
    // is PRESENTED, so a rejection is never a free retry — an attacker who captured
    // a nonce cannot use ANY rejection as an oracle to probe-and-retry the SAME
    // nonce (a re-presentation is a flat NonceReplay). It cannot harm an honest
    // prover (whose binding == nonce, so it never takes the mismatch path; and a
    // fresh session always mints a new nonce). The cost — a transient/buggy
    // submission spends the nonce — is accepted in exchange for the strict
    // no-retry-after-rejection property; the relying party simply issues a new
    // nonce. The `nonce_binding_mismatch_rejected` e2e test asserts this exact
    // policy (a re-presentation under the burnt nonce returns NonceReplay).
    if !seen.record_fresh(nonce) {
        return Err(CheckError::NonceReplay);
    }

    // --- Audit #4: nonce/binding consistency (fail-closed). ---
    // The challenge that MUST appear as public-input field 0 of every member is
    // the VERIFIER'S nonce, NOT the prover-written `manifest.binding`. We still
    // require the declared `binding` challenge to EQUAL the nonce so an honest
    // manifest is internally consistent and a manifest minted for a different
    // nonce/session is rejected explicitly (rather than only failing the
    // byte-compare further down). The load-bearing freshness anchor, though, is
    // the nonce fed into `reconstruct_public_inputs` below — the JSON `binding`
    // is no longer trusted as the challenge source (closing the audit-#4
    // single-JSON-substitution rebind).
    let challenge = nonce.as_field_hex();
    let declared_binding_challenge = match &manifest.binding {
        BindingMode::Challenge { challenge } => challenge,
        BindingMode::HolderPop { challenge, .. } => challenge,
    };
    // Compare as field elements so 0x-padding differences don't spuriously
    // diverge; a malformed declared binding challenge fails closed.
    let declared_fr = declared_binding_challenge.to_field();
    if declared_fr.is_none() || declared_fr != challenge.to_field() {
        return Err(CheckError::NonceBindingMismatch);
    }

    // --- sq-cwq: holder proof-of-possession (fail-closed for HolderPop). ---
    // When the binding is `HolderPop`, the holder MUST prove possession of a
    // relying-party-trusted holder key by signing the VERIFIER'S nonce (above);
    // an absent registry, an untrusted holder, or a malformed/invalid PoP all
    // REJECT here — there is no silent-accept of a HolderPop as a bare challenge.
    // A `Challenge` binding requires no PoP (this returns Ok immediately). The
    // nonce is recorded single-use BEFORE this, so a rejected PoP still burns the
    // nonce (consistent with the burn-on-mismatch policy above).
    bind_holder_pop(manifest, holder_registry, &challenge)?;

    for (i, sp) in manifest.sub_proofs.iter().enumerate() {
        if sp.proof_hex.is_empty() {
            return Err(CheckError::MissingProof { proof: i });
        }
        // Hardening: prover-controlled bytes are rejected, never panicked on.
        let blob = hex_decode(&sp.proof_hex)
            .ok_or(CheckError::MalformedProof { proof: i })?;
        let art = decode_artifacts(&blob).ok_or(CheckError::MalformedProof { proof: i })?;

        // (a) Reconstruct public inputs from the DECLARED statement (audit #1)
        // using the VERIFIER'S NONCE as field 0 (audit #4) and assert
        // byte-equality with the proof's public_inputs. This is the single
        // load-bearing binding: stages 1-2 check JSON, the proof is a detached
        // crypto object, and THIS ties them to the same statement AND to the
        // verifier's fresh nonce (a proof committed under a different challenge
        // cannot byte-match — closing replay).
        let reconstructed = reconstruct_public_inputs(&sp.inputs, &challenge, i)?;
        if reconstructed != art.public_inputs {
            return Err(CheckError::PublicInputMismatch { proof: i });
        }

        // (b) Recompute the canonical vk verifier-side from the member named by
        // the re-derived circuit id — NEVER the prover's art.vk (audit #2).
        // derive_id already passed in stage 1b, so the declared id is sound to
        // select the member by; recompute against it (full CircuitId pins the
        // n/d/r bucket, subsuming audit #11).
        let id = sp.inputs.circuit_id();
        let sub_work = work_dir.join(format!("sub{i}"));
        let canonical_vk = prover
            .canonical_vk(id, &sub_work.join("vk"))
            .map_err(CheckError::Driver)?;

        // (c) bb verify over (prover proof, OUR reconstructed public inputs, OUR
        // canonical vk). We pass the reconstructed public inputs (byte-equal to
        // the proof's, asserted above) so a single authentic vector is used end
        // to end.
        let ok = prover
            .verify_with(&art.proof, &reconstructed, &canonical_vk, &sub_work.join("verify"))
            .map_err(CheckError::Driver)?;
        if !ok {
            return Err(CheckError::ProofRejected { proof: i });
        }
    }

    // --- sq-3e5 / sq-h2v: hidden-index revocation cryptographic gate. ---
    // If the manifest carries a hidden-index revocation proof, verify it against
    // the relying party's OWN authoritative Merkle root (derived from its
    // authoritative snapshot) and the verifier's nonce. The clear-index liveness
    // gate (bind_revocation, in the prefilter above) is UNCHANGED and still runs;
    // this is the additive privacy upgrade that lets the holder NOT disclose its
    // index. The challenge fed here is the verifier's nonce (audit #4), identical
    // to the sub-proof loop's binding.
    bind_hidden_revocation(manifest, revocation_policy, prover, work_dir, &challenge)?;

    // --- sq-z9l: hidden-issuer attestation cryptographic gate. ---
    // If the manifest carries hidden-issuer attestation proofs, verify each
    // against the relying party's OWN authoritative key-set Merkle root (derived
    // from its trusted KeySet) and the verifier's nonce. The clear-key
    // attestation gate (bind_issuer_attestations, in the prefilter above) is
    // UNCHANGED and still runs; this is the additive privacy upgrade that lets the
    // holder NOT disclose WHICH issuer signed. The challenge fed here is the
    // verifier's nonce (audit #4), identical to the sub-proof loop's binding.
    bind_hidden_issuer_attestations(manifest, trusted_key_set, prover, work_dir, &challenge)?;

    Ok(())
}

/// Reconstruct the bb `public_inputs` byte vector from the DECLARED
/// `ProofInputs` (audit #1), in each member's `main` declaration order, using
/// the verifier's `challenge` as public-input field 0. bb's layout (determined
/// empirically against bb 5.0.0-nightly): each public input is one 32-byte
/// big-endian field element, structs/arrays flattened in index order
/// (row-major), `bool` -> {0,1}, `u32`/`u64` -> the integer value; no header,
/// no length prefix. The result is byte-compared against the proof's
/// `public_inputs`.
///
/// Declaration order — the single source of truth is each
/// `zk/compose/<member>/src/main.nr` (mirrored 1:1 by `toml.rs`):
/// - scan_k{k}_n{n}_r{r}: challenge, commitments[k], pattern_is_const[3],
///   pattern_const_enc[3], rows[r][3] (row-major), row_count, attribution[k]
///   (audit #8 — `bool` -> {0,1}, one word per committed graph).
/// - filter_int_d{d}: challenge, operand_enc, op, bound, expected.
///
/// Crucially the layout is sized by the DECLARED `CircuitId`'s `r`/`k` (rows
/// padded to `r`, commitments are exactly `k`), so a relabel that disagrees
/// with the actual member also fails (#11): a wrong `r` produces a vector of
/// the wrong length, which cannot byte-match the proof of the real member.
// [OPUS-4.8] public-input reconstruction (audit #1, subsumes #7/#11 with vk pin).
fn reconstruct_public_inputs(
    inputs: &ProofInputs,
    challenge: &FieldHex,
    proof: usize,
) -> Result<Vec<u8>, CheckError> {
    let mut out: Vec<u8> = Vec::new();
    // Append a field element parsed from a FieldHex, rejecting malformed hex.
    fn push_field(
        out: &mut Vec<u8>,
        h: &FieldHex,
        proof: usize,
        what: &'static str,
    ) -> Result<(), CheckError> {
        let f = h
            .to_field()
            .ok_or(CheckError::MalformedField { proof, what })?;
        out.extend_from_slice(&field_to_be_bytes_32(&f));
        Ok(())
    }
    // Append a small integer (u32/u64/bool/op-code) as a 32-byte BE word.
    fn push_uint(out: &mut Vec<u8>, v: u64) {
        let mut word = [0u8; 32];
        word[24..].copy_from_slice(&v.to_be_bytes());
        out.extend_from_slice(&word);
    }

    // Field 0 is always the verifier's challenge (every member's first `pub`).
    push_field(&mut out, challenge, proof, "challenge")?;

    match inputs {
        ProofInputs::Scan {
            commitments,
            pattern_is_const,
            pattern_const_enc,
            rows,
            row_count,
            attribution,
            ..
        } => {
            // commitments[k] — exactly k words (k is the declared CircuitId.k,
            // re-derived from commitments.len() in stage 1b).
            for c in commitments {
                push_field(&mut out, c, proof, "commitments")?;
            }
            // pattern_is_const[3] (bool -> {0,1}).
            for b in pattern_is_const {
                push_uint(&mut out, u64::from(*b));
            }
            // pattern_const_enc[3] (variable slots carry 0x0).
            for e in pattern_const_enc {
                push_field(&mut out, e, proof, "pattern_const_enc")?;
            }
            // rows[r][3], row-major, PADDED to the declared CircuitId.r with
            // zero rows (matching the prover's pad_rows / the circuit's R).
            let r = match inputs.circuit_id() {
                CircuitId::Scan { r, .. } => *r as usize,
                _ => return Err(CheckError::MalformedField { proof, what: "scan id" }),
            };
            for j in 0..r {
                match rows.get(j) {
                    Some(row) => {
                        for slot in row {
                            push_field(&mut out, slot, proof, "rows")?;
                        }
                    }
                    // Padding row: three zero words.
                    None => out.extend_from_slice(&[0u8; 96]),
                }
            }
            // row_count: u32.
            push_uint(&mut out, u64::from(*row_count));
            // attribution[k]: bool -> {0,1}, one word per committed graph (audit
            // #8). Bound to the proof so the verifier-side cross-check against
            // manifest.attributions operates over a PROOF-BOUND quantity. The
            // declared CircuitId.k is the authority for the length (re-derived
            // from commitments.len() in stage 1b); a wrong-length attribution
            // therefore yields a wrong-length vector that cannot byte-match.
            //
            // [OPUS-4.8] codex 2221 MEDIUM: `verify_manifest` runs
            // `prefilter_manifest_structure` (stage 1b) FIRST, which already rejects
            // any scan whose `attribution.len() != k` (`AttributionMalformed`), so
            // on this path `attribution` is exactly k bits. The `unwrap_or(false)`
            // below is therefore belt-and-braces only (never the pad-to-false
            // bypass — that is rejected upstream); it cannot widen the accepted
            // shape because the structural gate runs unconditionally before here.
            let k = match inputs.circuit_id() {
                CircuitId::Scan { k, .. } => *k as usize,
                _ => return Err(CheckError::MalformedField { proof, what: "scan id" }),
            };
            for g in 0..k {
                let bit = attribution.get(g).copied().unwrap_or(false);
                push_uint(&mut out, u64::from(bit));
            }
        }
        ProofInputs::FilterInt { operand_enc, op, bound, expected, .. } => {
            push_field(&mut out, operand_enc, proof, "operand_enc")?;
            push_uint(&mut out, u64::from(op.code()));
            push_uint(&mut out, *bound);
            push_uint(&mut out, u64::from(*expected));
        }
        // [OPUS-4.8] sq-q7e + sq-tat: filter_f64_d{d} public inputs, in
        // `main` declaration order: challenge (pushed above), operand_enc, op,
        // b_bits (the constant double's IEEE-754 bit pattern as a u64 word),
        // expected.
        ProofInputs::FilterF64 { operand_enc, op, b_bits, expected, .. } => {
            push_field(&mut out, operand_enc, proof, "operand_enc")?;
            push_uint(&mut out, u64::from(op.code()));
            push_uint(&mut out, *b_bits);
            push_uint(&mut out, u64::from(*expected));
        }
    }
    Ok(out)
}

/// SubProof `proof_hex` blob layout: `len(proof) | proof | len(pi) | pi | vk`,
/// each length a 4-byte big-endian u32. Keeps the three bb artifacts together
/// in one manifest field.
pub fn encode_artifacts(art: &crate::driver::ProofArtifacts) -> String {
    let mut blob = Vec::new();
    push_lp(&mut blob, &art.proof);
    push_lp(&mut blob, &art.public_inputs);
    blob.extend_from_slice(&art.vk);
    hex_encode(&blob)
}

/// Decoded bb artifacts split out of the `proof_hex` blob. `vk` is the
/// PROVER-supplied vk and is NOT trusted by `verify_manifest` (audit #2); it is
/// retained only for tooling / round-trip tests.
struct DecodedArtifacts {
    proof: Vec<u8>,
    public_inputs: Vec<u8>,
    #[allow(dead_code)]
    vk: Vec<u8>,
}

/// Decode the `proof_hex` blob. Returns `None` on any malformed length prefix /
/// truncation (the caller rejects via `CheckError::MalformedProof` rather than
/// panicking — audit hardening; prover-controlled bytes reach here before bb).
fn decode_artifacts(blob: &[u8]) -> Option<DecodedArtifacts> {
    let (proof, rest) = take_lp(blob)?;
    let (public_inputs, vk) = take_lp(rest)?;
    Some(DecodedArtifacts {
        proof: proof.to_vec(),
        public_inputs: public_inputs.to_vec(),
        vk: vk.to_vec(),
    })
}

fn push_lp(buf: &mut Vec<u8>, data: &[u8]) {
    buf.extend_from_slice(&(data.len() as u32).to_be_bytes());
    buf.extend_from_slice(data);
}

/// Read a 4-byte-big-endian-length-prefixed segment, returning `(body, rest)`.
/// `None` if the buffer is shorter than the prefix or the prefix overruns it
/// (no panic on prover-controlled bytes — audit hardening).
fn take_lp(buf: &[u8]) -> Option<(&[u8], &[u8])> {
    if buf.len() < 4 {
        return None;
    }
    let len = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
    let end = 4usize.checked_add(len)?;
    if end > buf.len() {
        return None;
    }
    Some((&buf[4..end], &buf[end..]))
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Decode a hex string to bytes. `None` on odd length or any non-hex nibble —
/// the caller rejects via `CheckError::MalformedProof` rather than panicking
/// (audit hardening: `proof_hex` is prover-controlled and reaches the verifier
/// before any bb call; the old `.expect("valid hex")` + OOB slice aborted the
/// process under the release `panic="abort"` profile).
fn hex_decode(s: &str) -> Option<Vec<u8>> {
    let bytes = s.as_bytes();
    // [OPUS-4.8] sq-hbg7: stable-1.96 clippy `manual_is_multiple_of`.
    if !bytes.len().is_multiple_of(2) {
        return None;
    }
    let mut out = Vec::with_capacity(bytes.len() / 2);
    let mut i = 0;
    while i < bytes.len() {
        let hi = (bytes[i] as char).to_digit(16)?;
        let lo = (bytes[i + 1] as char).to_digit(16)?;
        out.push((hi * 16 + lo) as u8);
        i += 2;
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::FilterOp;

    fn fh(s: &str) -> FieldHex {
        FieldHex(s.to_string())
    }

    // [OPUS-4.8] EMPIRICAL anchor for audit #1: the reconstruction must equal
    // the REAL bb `public_inputs` blobs captured by `bb prove --write_vk -t
    // noir-recursive` on the compiled members (bb 5.0.0-nightly.20260324). If a
    // toolchain bump changes the serialization these byte-vectors break loudly
    // — exactly the gate the binding layer needs. The hex below is the verbatim
    // probe output (see STATUS.md DESIGN); no nargo/bb needed to run this test.

    /// `filter_int_d1` over: challenge=0x2a, operand_enc=0x0831…943b, op=Lt(0),
    /// bound=10, expected=true. 5 fields * 32 = 160 bytes.
    #[test]
    fn reconstruct_filter_int_matches_real_bb_public_inputs() {
        let bb = hex_decode(concat!(
            "000000000000000000000000000000000000000000000000000000000000002a",
            "0831327030a1bd8f46862134e0d6273c75d1d33f4b3334fca37d9f7ed1a7943b",
            "0000000000000000000000000000000000000000000000000000000000000000",
            "000000000000000000000000000000000000000000000000000000000000000a",
            "0000000000000000000000000000000000000000000000000000000000000001",
        ))
        .unwrap();
        let inputs = ProofInputs::FilterInt {
            id: CircuitId::FilterInt { d: 1 },
            operand_enc: fh("0x0831327030a1bd8f46862134e0d6273c75d1d33f4b3334fca37d9f7ed1a7943b"),
            op: FilterOp::Lt,
            bound: 10,
            expected: true,
        };
        let got = reconstruct_public_inputs(&inputs, &fh("0x2a"), 0).unwrap();
        assert_eq!(got.len(), 160);
        assert_eq!(got, bb, "filter_int reconstruction must byte-match bb");
    }

    /// `scan_k1_n16_r4` over the probe values. 22 fields * 32 = 704 bytes; the
    /// single active row plus 3 zero-padded rows exercise the row-major
    /// flattening and the pad-to-`r` path, and the trailing `attribution[1]`
    /// word (audit #8) exercises the per-graph source-attribution binding.
    ///
    /// [OPUS-4.8] audit #8: the trailing `...01` word is the in-circuit
    /// attribution for the single committed graph (the pattern matches in it).
    /// Regenerated by the `probe_scan_public_inputs_hex` test (e2e.rs, ignored)
    /// against a real `bb prove` of the new scan_k1_n16_r4 — the empirical anchor.
    #[test]
    fn reconstruct_scan_matches_real_bb_public_inputs() {
        let bb = hex_decode(concat!(
            // challenge
            "000000000000000000000000000000000000000000000000000000000000002a",
            // commitments[0]
            "000c6024571cb9fc261106500dbacab6fbd7fdafd50566f881c68b9fde9cffe1",
            // pattern_is_const [false,true,false]
            "0000000000000000000000000000000000000000000000000000000000000000",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000000",
            // pattern_const_enc [0, 0x0579…5b61, 0]
            "0000000000000000000000000000000000000000000000000000000000000000",
            "057914fc592ade7b970f92e4992958ea8a6a265caeb012b5b63903257eef5b61",
            "0000000000000000000000000000000000000000000000000000000000000000",
            // rows[0] = active match
            "067d8d75b405117a4cce58d59db1cbe420dabf0ff8d4c0fa50d80ccf0ed4a713",
            "057914fc592ade7b970f92e4992958ea8a6a265caeb012b5b63903257eef5b61",
            "2b5caeb2bbd290ab32434a9109030784c7faebadee7a9908d24dccb847910d1d",
            // rows[1..4] = zero rows (9 zero words)
            "0000000000000000000000000000000000000000000000000000000000000000",
            "0000000000000000000000000000000000000000000000000000000000000000",
            "0000000000000000000000000000000000000000000000000000000000000000",
            "0000000000000000000000000000000000000000000000000000000000000000",
            "0000000000000000000000000000000000000000000000000000000000000000",
            "0000000000000000000000000000000000000000000000000000000000000000",
            "0000000000000000000000000000000000000000000000000000000000000000",
            "0000000000000000000000000000000000000000000000000000000000000000",
            "0000000000000000000000000000000000000000000000000000000000000000",
            // row_count = 1
            "0000000000000000000000000000000000000000000000000000000000000001",
            // attribution[0] = true (the single graph matches the pattern)
            "0000000000000000000000000000000000000000000000000000000000000001",
        ))
        .unwrap();
        let z = || fh("0x0");
        let inputs = ProofInputs::Scan {
            id: CircuitId::Scan { k: 1, n: 16, r: 4 },
            commitments: vec![fh(
                "0x000c6024571cb9fc261106500dbacab6fbd7fdafd50566f881c68b9fde9cffe1",
            )],
            pattern_is_const: [false, true, false],
            pattern_const_enc: [
                z(),
                fh("0x057914fc592ade7b970f92e4992958ea8a6a265caeb012b5b63903257eef5b61"),
                z(),
            ],
            // Only the active row is declared; reconstruction pads to r=4.
            rows: vec![[
                fh("0x067d8d75b405117a4cce58d59db1cbe420dabf0ff8d4c0fa50d80ccf0ed4a713"),
                fh("0x057914fc592ade7b970f92e4992958ea8a6a265caeb012b5b63903257eef5b61"),
                fh("0x2b5caeb2bbd290ab32434a9109030784c7faebadee7a9908d24dccb847910d1d"),
            ]],
            row_count: 1,
            // The single graph contributes => attribution[0] = true (audit #8).
            attribution: vec![true],
        };
        let got = reconstruct_public_inputs(&inputs, &fh("0x2a"), 0).unwrap();
        assert_eq!(got.len(), 704);
        assert_eq!(got, bb, "scan reconstruction must byte-match bb");
    }

    /// A different declared statement (a single mutated field) must produce a
    /// different vector — the property the byte-compare relies on (audit #1).
    #[test]
    fn reconstruct_is_statement_sensitive() {
        let base = ProofInputs::FilterInt {
            id: CircuitId::FilterInt { d: 1 },
            operand_enc: fh("0x05"),
            op: FilterOp::Ge,
            bound: 18,
            expected: true,
        };
        let v0 = reconstruct_public_inputs(&base, &fh("0x2a"), 0).unwrap();
        // Flip the bound 18 -> 17.
        let mut m = base.clone();
        if let ProofInputs::FilterInt { bound, .. } = &mut m {
            *bound = 17;
        }
        assert_ne!(v0, reconstruct_public_inputs(&m, &fh("0x2a"), 0).unwrap());
        // Flip the challenge.
        assert_ne!(v0, reconstruct_public_inputs(&base, &fh("0x2b"), 0).unwrap());
    }

    /// Malformed `proof_hex` is rejected, never panics (audit hardening).
    #[test]
    fn hex_decode_and_take_lp_reject_bad_bytes() {
        assert!(hex_decode("zz").is_none()); // non-hex
        assert!(hex_decode("abc").is_none()); // odd length
        assert!(hex_decode("").unwrap().is_empty());
        assert!(take_lp(&[0, 0, 0]).is_none()); // < 4-byte prefix
        assert!(take_lp(&[0, 0, 0, 255, 1, 2]).is_none()); // oversized length
        assert!(decode_artifacts(&[0, 0, 0, 1, 9]).is_none()); // pi prefix missing
    }

    /// Audit #4: the in-memory single-use store records a nonce on first sight
    /// (fresh => true) and rejects it on every subsequent sight (=> false).
    #[test]
    fn in_memory_seen_nonces_is_single_use() {
        let store = InMemorySeenNonces::new();
        let n = VerifierNonce::from_hex("0x2a").unwrap();
        // First presentation is fresh; second (and third) are not.
        assert!(store.record_fresh(&n), "first sight must be fresh");
        assert!(!store.record_fresh(&n), "replay must be rejected");
        assert!(!store.record_fresh(&n), "replay stays rejected");
        // A DIFFERENT nonce is independently fresh.
        let n2 = VerifierNonce::from_hex("0x2b").unwrap();
        assert!(store.record_fresh(&n2));
    }

    /// A unique temp path for a durable-store test (cleaned by the test).
    fn tmp_nonce_log(tag: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "sparq_zk_compose_seen_nonces_{tag}_{}_{:?}.log",
            std::process::id(),
            std::thread::current().id()
        ))
    }

    /// sq-aih (audit #4 durability): the DURABLE store records a nonce, and a
    /// FRESH store REOPENED FROM THE SAME PATH (modelling a verifier restart —
    /// the in-memory state is gone) STILL REJECTS the reused nonce. This is the
    /// load-bearing property the in-memory store lacks.
    #[test]
    fn file_seen_nonces_single_use_survives_restart() {
        let path = tmp_nonce_log("restart");
        let _ = std::fs::remove_file(&path);
        let n = VerifierNonce::from_hex("0x2a").unwrap();

        // Session 1: first sight is fresh, replay rejected. Then DROP the store
        // (its in-memory fd/state is gone — a restart).
        {
            let store = FileSeenNonces::open(&path).expect("open durable store");
            assert!(store.record_fresh(&n), "first sight must be fresh");
            assert!(!store.record_fresh(&n), "replay rejected within the session");
        }

        // Session 2: a COMPLETELY FRESH store reopened from the SAME path. The
        // nonce must still be seen — single-use survived the restart.
        {
            let store = FileSeenNonces::open(&path).expect("reopen durable store");
            assert!(
                !store.record_fresh(&n),
                "a nonce recorded before restart must STILL be rejected after reopening the store"
            );
            // A different nonce is independently fresh in the reopened store.
            let n2 = VerifierNonce::from_hex("0x2b").unwrap();
            assert!(store.record_fresh(&n2), "an unseen nonce is fresh after restart");
        }

        // Session 3: confirm the second nonce is now ALSO durable.
        {
            let store = FileSeenNonces::open(&path).expect("reopen durable store");
            let n2 = VerifierNonce::from_hex("0x2b").unwrap();
            assert!(!store.record_fresh(&n2), "the post-restart nonce is durable too");
        }

        let _ = std::fs::remove_file(&path);
    }

    /// sq-aih: the durable store keys by CANONICAL field value too — a re-padded
    /// hex spelling of an already-recorded nonce is still rejected after a restart
    /// (mirrors the in-memory `seen_nonces_key_is_representation_insensitive`).
    #[test]
    fn file_seen_nonces_key_is_representation_insensitive_across_restart() {
        let path = tmp_nonce_log("repr");
        let _ = std::fs::remove_file(&path);
        let padded = VerifierNonce::from_hex(
            "0x000000000000000000000000000000000000000000000000000000000000002a",
        )
        .unwrap();
        let bare = VerifierNonce::from_hex("0x2a").unwrap();
        {
            let store = FileSeenNonces::open(&path).expect("open");
            assert!(store.record_fresh(&padded), "first sight fresh");
        }
        {
            // Restart, then present the SAME field under a different spelling.
            let store = FileSeenNonces::open(&path).expect("reopen");
            assert!(
                !store.record_fresh(&bare),
                "same field, different spelling => replay even across a restart"
            );
        }
        let _ = std::fs::remove_file(&path);
    }

    /// Audit #4: a nonce is keyed by its CANONICAL field value, so two hex
    /// spellings of the same field element collapse to one single-use entry (a
    /// prover cannot re-present by re-padding the hex).
    #[test]
    fn seen_nonces_key_is_representation_insensitive() {
        let store = InMemorySeenNonces::new();
        let padded = VerifierNonce::from_hex(
            "0x000000000000000000000000000000000000000000000000000000000000002a",
        )
        .unwrap();
        let bare = VerifierNonce::from_hex("0x2a").unwrap();
        assert!(store.record_fresh(&padded), "first sight fresh");
        assert!(!store.record_fresh(&bare), "same field, different spelling => replay");
    }

    /// Audit #4: the nonce round-trips to the FieldHex the reconstruction binds
    /// as field 0 (canonical 0x-prefixed 64-nibble hex).
    #[test]
    fn verifier_nonce_round_trips_to_field_hex() {
        let n = VerifierNonce::from_hex("0x2a").unwrap();
        assert_eq!(
            n.as_field_hex(),
            FieldHex(
                "0x000000000000000000000000000000000000000000000000000000000000002a".to_string()
            )
        );
        // A reconstruction with the nonce as challenge equals one with the
        // equivalent FieldHex challenge (the nonce IS the challenge anchor).
        let inputs = ProofInputs::FilterInt {
            id: CircuitId::FilterInt { d: 1 },
            operand_enc: fh("0x05"),
            op: FilterOp::Ge,
            bound: 18,
            expected: true,
        };
        let via_nonce =
            reconstruct_public_inputs(&inputs, &n.as_field_hex(), 0).unwrap();
        let via_hex = reconstruct_public_inputs(&inputs, &fh("0x2a"), 0).unwrap();
        assert_eq!(via_nonce, via_hex);
    }

    /// Audit #4: a non-field nonce hex is rejected at construction (fail-closed —
    /// a relying party cannot mint an unparseable nonce).
    #[test]
    fn verifier_nonce_rejects_malformed_hex() {
        assert!(VerifierNonce::from_hex("0xzz").is_none());
        assert!(VerifierNonce::from_hex("").is_none());
    }

    /// A non-field hex string in a declared slot is rejected (no panic).
    #[test]
    fn reconstruct_rejects_malformed_field() {
        let inputs = ProofInputs::FilterInt {
            id: CircuitId::FilterInt { d: 1 },
            operand_enc: fh("0xnot-a-field"),
            op: FilterOp::Ge,
            bound: 18,
            expected: true,
        };
        assert!(matches!(
            reconstruct_public_inputs(&inputs, &fh("0x2a"), 0),
            Err(CheckError::MalformedField { proof: 0, what: "operand_enc" })
        ));
    }
}
