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
//!       public-input field vector from the DECLARED [`ProofInputs`] (in `main`
//!       declaration order) using the verifier's own challenge, serialize to
//!       bb's byte layout (32-byte BE field elements, no header — see
//!       [`reconstruct_public_inputs`]), and assert byte-equality with the
//!       prover's `public_inputs` blob. This binds the JSON statement to the
//!       proof; without it stages 1-2 (JSON) and the proof (a detached crypto
//!       object) describe potentially different statements.
//!    b. **Canonical vk (audit #2).** Recompute the vk verifier-side from the
//!       compiled member named by the re-derived [`CircuitId`] (never the
//!       prover's `art.vk`), pinning the vk to the FULL CircuitId (subsumes the
//!       #11 n/d/r relabel).
//!    c. `bb verify` over (prover proof, reconstructed public inputs, canonical
//!       vk).
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

use crate::build::{derive_filter_int_id, derive_scan_id};
use crate::driver::{CircuitProver, DriverError};
use crate::manifest::{
    BindingMode, CircuitId, FieldHex, ProofInputs, ProofManifest,
};
use sparq_zk::encode::encode_term;
use sparq_zk::field::{field_from_hex_str, field_to_be_bytes_32, field_to_hex, Fr};
// [OPUS-4.8] codex 2221 HIGH: only the SALT-BOUND `commitment_message_with_salt`
// is used on the scan-verify path; the bare salt-less `commitment_message` is no
// longer reachable here (a scan-covering attestation must be salt-bound).
use sparq_zk::sig::{
    commitment_message_with_salt, public_key_from_hex, signature_from_hex, verify as sig_verify,
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
}

impl KeySet {
    /// An empty trust anchor: trusts NO issuer. Any scan carrying commitments is
    /// then rejected (fail closed). Useful as the explicit "no source is
    /// authoritative" policy and as a test default.
    pub fn empty() -> Self {
        KeySet { keys: BTreeSet::new() }
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
        KeySet { keys }
    }

    /// Whether `pk_hex` (any case, optional `0x`) is a member of the trusted set.
    fn contains_hex(&self, pk_hex: &str) -> bool {
        self.keys.contains(&normalize_hex(pk_hex))
    }

    /// The trusted set is empty (trusts no issuer).
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
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
/// # Persistence
/// The in-memory [`InMemorySeenNonces`] is process-local: it enforces single-use
/// within one verifier process/session. A multi-process / restart-surviving
/// relying party MUST back this trait with durable storage (a database row with a
/// UNIQUE constraint on the nonce, a KV store with compare-and-set, etc.); that
/// is a documented future impl. The trait boundary is exactly so the persistence
/// choice is pluggable without touching the verifier.
// [OPUS-4.8] audit #4: single-use nonce store.
pub trait SeenNonces {
    /// Record `nonce` as used and return `true` iff it was FRESH (not previously
    /// recorded). Returns `false` if the nonce was already seen — the verifier
    /// then rejects the manifest as a replay. Implementations MUST be atomic
    /// (check-and-insert) so concurrent verifiers cannot both observe the same
    /// nonce as fresh.
    fn record_fresh(&self, nonce: &VerifierNonce) -> bool;
}

/// Process-local, thread-safe [`SeenNonces`] (audit #4). Enforces single-use
/// within one verifier process. NOT durable across restarts — see the
/// [`SeenNonces`] persistence note; a production relying party backs the trait
/// with a database/KV store with a uniqueness constraint.
// [OPUS-4.8] audit #4: in-memory single-use store (persistence is a future impl).
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
pub fn prefilter_manifest_structure(
    manifest: &ProofManifest,
    trusted_key_set: &KeySet,
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
        let operand = match &to.inputs {
            ProofInputs::FilterInt { operand_enc, .. } => operand_enc,
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
    // attested commitment to the proved statement.
    bind_issuer_attestations(manifest, trusted_key_set)?;

    Ok(required)
}

/// Stage 2d: bind every scan commitment to an issuer signature whose key is in
/// the EXTERNAL trusted key-set `K` (audit #3, soundness fix for codex #1). For
/// each `commitments[g]` of each scan sub-proof:
/// - there MUST be a `commitment_attestations` entry over that commitment value,
/// - its signature MUST verify under its declared `issuer_public_key`,
/// - that key MUST be a member of the EXTERNAL `trusted_key_set` (the relying
///   party's argument — NEVER `manifest.key_set`),
/// - and, when the prover DECLARED a narrowed `manifest.key_set` (non-empty),
///   that key MUST ALSO be a member of it (codex 2216 LOW): a declared narrowing
///   must be internally consistent with the attestations actually proven. The
///   accept decision stays anchored on the external K (this consistency rule is
///   ADDED to, never substituted for, the external-K check).
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
            let message = commitment_message_with_salt(&commitment_fr, &salt_fr);
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

/// Full verification: structure (stage 1+2) then the cryptographic gate
/// (stage 3). `prover` points at the `zk/compose/` workspace; `work_dir` is
/// scratch for bb artifacts; `trusted_key_set` is the relying party's EXTERNAL
/// issuer trust anchor `K` (audit #3 codex #1 — never the prover's
/// `manifest.key_set`).
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
pub fn verify_manifest(
    manifest: &ProofManifest,
    prover: &CircuitProver,
    work_dir: &Path,
    trusted_key_set: &KeySet,
    nonce: &VerifierNonce,
    seen: &dyn SeenNonces,
) -> Result<(), CheckError> {
    prefilter_manifest_structure(manifest, trusted_key_set)?;

    // --- Audit #4: single-use (fail-closed, BEFORE the crypto gate). ---
    // Record the verifier's nonce as used; reject if it was already seen. Doing
    // this first means a replayed (nonce, manifest) pair is rejected without
    // even running bb. The store is consulted unconditionally — there is no
    // opt-out path that could bypass single-use (the parameter is mandatory).
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
    if bytes.len() % 2 != 0 {
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
