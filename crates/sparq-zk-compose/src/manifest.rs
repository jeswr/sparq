// [OPUS-4.8] written while Fable 5 unavailable — re-review when Fable returns.
//! The proof manifest: the public, serializable description of a query-result
//! proof (plan v3 §S2.5 / §S4.E module (iii)).
//!
//! A manifest travels with the bb proof bytes and is everything a verifier
//! needs (besides the proof) to re-derive the circuit-family id, re-check the
//! sparq-zk obligations, and reconstruct the public-input vector the circuit
//! was proved against. It deliberately carries NO witnesses — only public
//! inputs and the metadata that binds them to issuers, an entailment regime,
//! and a freshness challenge.
//!
//! Credential model (named-graph): the proven content lives in a named graph;
//! the manifest is the proof's metadata graph (plan §S4.E). `did:key` issuer
//! refs and the status-list placeholder mirror the W3C VC data model so a
//! manifest can later be lifted into a `DataIntegrityProof`-shaped credential
//! without a schema change.

use serde::{Deserialize, Serialize};
use sparq_zk::field::{field_to_hex, field_from_hex_str, Fr};

/// Field element rendered as `0x`-prefixed 64-nibble hex — the same
/// representation `<urn:sparq:zk>` registry literals use, so commitments and
/// term encodings round-trip through the manifest verbatim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FieldHex(pub String);

impl FieldHex {
    pub fn from_field(f: &Fr) -> Self {
        FieldHex(field_to_hex(f))
    }
    /// Parse back to a field element. `None` if the hex is malformed.
    pub fn to_field(&self) -> Option<Fr> {
        field_from_hex_str(&self.0)
    }
}

/// SPARQL numeric comparison operator selector — mirrors the circuit globals
/// `OP_*` in `sparq_zk_compose_core::filter_int` (value = the `op` public
/// input).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterOp {
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
}

impl FilterOp {
    /// The `op` public-input value the circuit expects.
    pub fn code(self) -> u32 {
        match self {
            FilterOp::Lt => 0,
            FilterOp::Le => 1,
            FilterOp::Gt => 2,
            FilterOp::Ge => 3,
            FilterOp::Eq => 4,
            FilterOp::Ne => 5,
        }
    }
}

/// Entailment regime under which the proof was produced (plan §S2.5
/// with/without-inference). The verifier records which regime it is checking;
/// in v1 only `Simple` is proved (no in-circuit reasoning) — `Rdfs`/`Owl` are
/// placeholders so the field is stable across the inference deliverable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EntailmentRegime {
    /// No inference: the commitment holds exactly the asserted triples.
    Simple,
    /// RDFS entailment (deferred — placeholder).
    Rdfs,
    /// OWL RL entailment (deferred — placeholder).
    Owl,
}

/// How the proof is bound against replay / holder (plan §S2.5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "kebab-case")]
pub enum BindingMode {
    /// Verifier-supplied fresh nonce, carried as the circuit's `challenge`
    /// public input (the v1 default).
    Challenge { challenge: FieldHex },
    /// Holder proof-of-possession (deferred — placeholder; the field is
    /// reserved so the manifest schema is stable when PoP lands).
    HolderPop { challenge: FieldHex, holder: String },
}

impl BindingMode {
    pub fn challenge(&self) -> &FieldHex {
        match self {
            BindingMode::Challenge { challenge } => challenge,
            BindingMode::HolderPop { challenge, .. } => challenge,
        }
    }
}

/// An issuer attestation over one per-graph commitment `C(G)` (audit #3): the
/// commitment value, the issuer's public key, and the issuer's signature over
/// the domain-separated commitment message. The verifier checks (a) the
/// signature is valid under `issuer_public_key`, and (b) `issuer_public_key` is
/// a member of the manifest's disclosed `key_set` K — so `commitments[]` is no
/// longer an unsigned prover-chosen public input. Every scan sub-proof's
/// commitment must have a matching, in-`K` attestation, else the manifest is
/// rejected.
///
/// Privacy note (interim): this discloses WHICH issuer signed each graph (the
/// key is checked in the clear). The full-privacy upgrade verifies the same
/// signature IN-CIRCUIT with an undisclosed signing key + a set-membership
/// gadget over K, revealing only "signed by SOME key in K". See
/// `sparq_zk::sig` module docs.
// [OPUS-4.8] audit #3.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitmentAttestation {
    /// `C(G)` — must match a scan sub-proof's `commitments[g]` (and is itself
    /// byte-bound into the bb public inputs by the audit #1 reconstruction).
    pub commitment: FieldHex,
    /// The issuer verification key (compressed Baby-JubJub point, hex). Must be
    /// a member of `ProofManifest::key_set` (audit #3 key-set membership).
    pub issuer_public_key: String,
    /// The issuer's signature, hex. When `salt` is present (audit #9) the signed
    /// message is `commitment_message_with_salt(C(G), salt_G)`; otherwise it is
    /// the bare `commitment_message(C(G))` (audit #3, salt-unbound legacy).
    pub signature: String,
    /// The signature scheme's `zk:cryptosuite` IRI (`poseidon2-schnorr-v1` in
    /// v1). An unknown cryptosuite is unverifiable => the attestation is
    /// rejected (fail closed).
    pub cryptosuite: String,
    /// The per-graph RDFC10 bnode salt this commitment was committed under
    /// (audit #9), hex. When present, the issuer signature is verified over the
    /// SALT-BOUND message so the salt is issuer-attested (a salt-reusing
    /// ingester cannot present a graph under a salt the issuer did not sign), and
    /// the verifier additionally rejects two DISTINCT commitments sharing a salt
    /// (the Q6 cross-graph bnode-correlation channel). Absent => salt-unbound
    /// legacy attestation (audit #3 only); the salt-separation guarantee then
    /// rests on the honest-ingest convention, which the verifier cannot enforce.
    // [OPUS-4.8] audit #9: issuer-attested per-graph salt.
    #[serde(default)]
    pub salt: Option<FieldHex>,
    /// The credential's STATUS-LIST REFERENCE as the issuer signed it (audit
    /// #12): the index + version that, together with `H(status_list)` from the
    /// manifest's [`RevocationStatus`], form the issuer-bound
    /// [`sparq_zk::sig::status_ref_digest`]. When present the issuer signature is
    /// verified over the STATUS-BOUND message
    /// ([`sparq_zk::sig::commitment_message_with_status`]), so the reference is
    /// unforgeable and un-omittable: the verifier recomputes the digest from this
    /// field + the disclosed `RevocationStatus` and the bare signature check
    /// fails if they disagree. A scan-covering attestation MUST carry this
    /// (mandatory / fail-closed — mirrors the codex-2221 salt-mandatory
    /// precedent; an omitted status ref is rejected, never silently un-checked).
    /// Absent => status-unbound (audit #3/#9 only) attestation, NOT accepted on
    /// the scan-verify path.
    // [OPUS-4.8] audit #12: issuer-attested status-list reference.
    #[serde(default)]
    pub status: Option<AttestedStatusRef>,
}

/// The index + version of a credential's status-list reference as bound under
/// the issuer signature (audit #12). The list IRI is the manifest's
/// [`RevocationStatus::status_list`]; the verifier hashes it
/// ([`sparq_zk::sig::status_list_id_to_field`]) and folds it with these into
/// [`sparq_zk::sig::status_ref_digest`] to recompute the signed message. Carried
/// in the attestation (not just `RevocationStatus`) so the issuer-signed values
/// and the disclosed reference are cross-checked for equality — a prover that
/// disclosed a different index/version than the issuer signed is rejected.
// [OPUS-4.8] audit #12.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttestedStatusRef {
    /// The credential's index into the status list (as issuer-signed).
    pub index: u64,
    /// The status-list version (as issuer-signed).
    pub version: u64,
}

/// A credential's revocation reference (plan §S2.6, Bitstring/StatusList2021
/// shape): which status list tracks the credential's liveness, the credential's
/// index into that list, and the list VERSION the prover is asserting against.
///
/// # Issuer-bound (audit #12, leverages #3)
/// This reference is NOT trusted as a prover claim. The issuer signature (which
/// already binds `C(G)` + salt, audit #3/#9) ALSO binds
/// [`sparq_zk::sig::status_ref_digest`]`(H(status_list), index, version)`
/// ([`CommitmentAttestation::status`]). The verifier recomputes that digest from
/// THIS disclosed reference and requires it to match the issuer-signed value —
/// so a prover cannot omit, forge, or swap the reference (an omitted/forged
/// reference yields no valid issuer signature, fail-closed). The verifier then
/// checks the disclosed status-list snapshot's bit at `index` is UNSET and the
/// `version` is within its freshness window.
///
/// # Privacy (interim, documented deferral)
/// `index` is disclosed in the CLEAR here — a linkability channel (a relying
/// party can correlate two presentations of the same credential by its index).
/// The full-privacy upgrade is an IN-CIRCUIT hidden-index status-list inclusion
/// + bit-unset proof bound to a disclosed list version, revealing only "the
/// (hidden) index is in-range and unset in version V". See the verifier module
/// docs (audit #12 remaining-step note).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevocationStatus {
    /// IRI of the status-list credential (bound under the issuer signature via
    /// [`sparq_zk::sig::status_list_id_to_field`]).
    pub status_list: String,
    /// Index into the list. Disclosed in the clear in v1 (a documented
    /// linkability channel — see the type docs); the full design hides it.
    pub index: u64,
    /// The status-list version the credential asserts against (a monotone
    /// freshness counter — the issuer's status-list publication sequence /
    /// `validFrom` epoch). Bound under the issuer signature and freshness-window
    /// checked by the verifier (audit #12). `#[serde(default)]` keeps old
    /// version-less manifests parseable, but the verifier's status check is
    /// mandatory and a version-0 reference still must match the issuer-signed
    /// digest and a fresh snapshot, so the default does not bypass the gate.
    // [OPUS-4.8] audit #12: issuer-bound, freshness-checked version.
    #[serde(default)]
    pub version: u64,
}

/// A snapshot of a Bitstring/StatusList2021-style status list (audit #12): the
/// list IRI, its version, and its status bitstring. The credential's liveness is
/// `bit[index] == 0` (unset = active; set = revoked/suspended).
///
/// # Two roles — and which one is authoritative (re-audit Option B)
/// This type is used in TWO places:
/// 1. The relying party's AUTHORITATIVE snapshot, carried externally in
///    `verifier::RevocationPolicy` (resolved + authenticated by the relying party
///    out of band, like the trusted key-set `K`). The liveness BIT decision reads
///    from HERE.
/// 2. The prover's `ProofManifest::status_snapshots` — UNAUTHENTICATED
///    prover-supplied bytes. The issuer signature binds the status-list REFERENCE
///    (`status_ref_digest(H(list IRI), index, version)`) but NOT the bit VALUES,
///    so the prover's bitstring is NOT trusted for the bit decision. If a prover
///    snapshot is present for the referenced `(list, version)` the verifier only
///    requires it to byte-equal the authoritative one (a tamper tripwire —
///    `CheckError::StatusSnapshotTampered`); otherwise it is ignored. Reading the
///    bit from the prover's snapshot was the re-audit hole: a genuine reference +
///    a forged all-zero snapshot reverified a REVOKED credential.
///
/// The verifier checks, against the (issuer-bound) [`RevocationStatus`] and its
/// AUTHORITATIVE snapshot: (i) the reference `version` is within the verifier's
/// freshness window, (ii) the AUTHORITATIVE `bit[index]` is UNSET, and (iii) any
/// prover snapshot for `(list, version)` agrees byte-for-byte. A revoked
/// authoritative bit, a stale version, a missing authoritative snapshot, or a
/// disagreeing prover snapshot all REJECT (fail-closed).
///
/// `bits` is the raw status bitstring, LSB-first within each byte (bit `i` is
/// `bits[i / 8] >> (i % 8) & 1`) — the StatusList2021 convention.
// [OPUS-4.8] audit #12: status-list snapshot (authoritative copy lives in the policy).
// [OPUS-4.8] audit #12 re-audit: prover's manifest copy is NOT the bit-decision source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusListSnapshot {
    /// IRI of the status list this snapshot is for (must equal the credential's
    /// `RevocationStatus::status_list`).
    pub status_list: String,
    /// The snapshot's version (must equal the credential's
    /// `RevocationStatus::version` and be within the verifier freshness window).
    pub version: u64,
    /// The raw status bitstring, LSB-first within each byte.
    pub bits: Vec<u8>,
}

impl StatusListSnapshot {
    /// The status bit at `index` (LSB-first within each byte). An out-of-range
    /// index reads as SET (revoked) — fail-closed: a credential whose index
    /// falls outside the disclosed snapshot is treated as not-proven-live.
    // [OPUS-4.8] audit #12: out-of-range reads as revoked (fail closed).
    pub fn bit(&self, index: u64) -> bool {
        let byte = (index / 8) as usize;
        let off = (index % 8) as u32;
        match self.bits.get(byte) {
            Some(b) => (b >> off) & 1 == 1,
            None => true,
        }
    }
}

/// The circuit-family id: which compiled member of the `zk/compose/` family
/// the proof was produced by. Prover and verifier BOTH derive this from the
/// manifest shape (the (k, n) lattice id), so a proof can only verify against
/// the member its public inputs fit (plan Q5 / brief: "derive the circuit-
/// family id from the proof manifest").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CircuitId {
    /// `scan_k{k}_n{n}_r{r}` — a BGP triple-pattern scan over `k` graphs,
    /// `n` slots/graph, `r` disclosed rows.
    Scan { k: u32, n: u32, r: u32 },
    /// `filter_int_d{d}` — hidden xsd:integer FILTER with `d` decimal digits.
    FilterInt { d: u32 },
    /// `filter_f64` — xsd:double FILTER (v1 building block, not yet
    /// manifest-composable).
    FilterF64,
}

impl CircuitId {
    /// The on-disk package directory name under `zk/compose/`.
    pub fn package(&self) -> String {
        match self {
            CircuitId::Scan { k, n, r } => format!("scan_k{k}_n{n}_r{r}"),
            CircuitId::FilterInt { d } => format!("filter_int_d{d}"),
            CircuitId::FilterF64 => "filter_f64".to_string(),
        }
    }
}

/// One per-property proof's public inputs, by circuit kind. These are exactly
/// the `pub` parameters of the corresponding `main` (in declaration order) —
/// the prover serializes them into `Prover.toml`, the verifier re-derives the
/// public-input vector from them. `binding`'s challenge is prepended as the
/// first `challenge: pub Field` of every member.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "circuit")]
pub enum ProofInputs {
    /// scan_k{k}_n{n}_r{r}: commitment recompute + scan completeness.
    #[serde(rename = "scan")]
    Scan {
        id: CircuitId,
        /// Per-graph flat Poseidon2 commitments (length k).
        commitments: Vec<FieldHex>,
        /// BGP slot constancy (s, p, o).
        pattern_is_const: [bool; 3],
        /// Term encodings of constant slots (0 = variable).
        pattern_const_enc: [FieldHex; 3],
        /// Disclosed matched rows (length r, padded with zero rows).
        rows: Vec<[FieldHex; 3]>,
        /// Active row count (<= r).
        row_count: u32,
        /// Per-graph source attribution (length k): `attribution[g]` is true
        /// iff this pattern's match set draws a triple from committed graph `g`.
        /// Constrained in-circuit (`scan.nr` step 4, audit #8) and byte-bound
        /// into the bb public inputs by [`crate::verifier::reconstruct_public_inputs`],
        /// so it is PROOF-BOUND, not a prover-controlled claim. The verifier
        /// cross-checks it against `manifest.attributions` for the pattern this
        /// scan answers (closes the `[[0],[0]]` collapse-two-graphs forge).
        // [OPUS-4.8] audit #8: in-circuit per-graph source attribution.
        #[serde(default)]
        attribution: Vec<bool>,
    },
    /// filter_int_d{d}: hidden-operand numeric FILTER over an xsd:integer.
    #[serde(rename = "filter_int")]
    FilterInt {
        id: CircuitId,
        /// The hidden column's term encoding (the scan-proof anchor).
        operand_enc: FieldHex,
        op: FilterOp,
        /// The FILTER's constant operand.
        bound: u64,
        /// The disclosed verdict.
        expected: bool,
    },
}

impl ProofInputs {
    pub fn circuit_id(&self) -> &CircuitId {
        match self {
            ProofInputs::Scan { id, .. } => id,
            ProofInputs::FilterInt { id, .. } => id,
        }
    }
}

/// One composed sub-proof: the circuit member, its public inputs, and the
/// raw bb proof bytes (hex). Composition is the verifier checking each
/// sub-proof AND the binding-consistency edges between them (a shared
/// `operand_enc` appearing in both a scan proof's rows and a filter proof's
/// public inputs is a plain public-input equality — the "modular per-property
/// proof" pattern, sparql_noir reference architecture).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubProof {
    pub inputs: ProofInputs,
    /// bb proof bytes, hex-encoded (no `0x`). Empty in witness-only manifests.
    #[serde(default)]
    pub proof_hex: String,
}

/// A binding-consistency edge: the term encoding at `from_proof`'s row/slot
/// must equal the `operand_enc` of `to_proof` (a numeric FILTER applied to a
/// scanned column). Verifier checks this as a field equality over the
/// already-verified public inputs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BindingEdge {
    /// Index into `ProofManifest::sub_proofs` of the scan proof.
    pub from_proof: usize,
    /// Which disclosed row of the scan proof carries the operand.
    pub from_row: usize,
    /// Which slot (0=s,1=p,2=o) of that row is the operand column.
    pub from_slot: usize,
    /// Index into `sub_proofs` of the consuming filter proof.
    pub to_proof: usize,
}

/// The full query-result proof manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofManifest {
    /// Schema marker (URN registry convention, mirrors `<urn:sparq:zk>`).
    #[serde(default = "default_type")]
    pub r#type: String,
    /// The SPARQL query text the proof attests a result for. The verifier
    /// re-parses this (sparq-zk `verify::recheck`) — it is NOT trusted.
    pub query: String,
    /// did:key issuer references for the committed graphs (length = number of
    /// distinct issuers; informational provenance — the cryptographic check is
    /// `key_set` + `commitment_attestations` below).
    #[serde(default)]
    pub issuers: Vec<String>,
    /// The prover's DECLARED key-set (audit #3): the issuer public keys the
    /// prover claims its commitments draw on, as hex (compressed Baby-JubJub
    /// points). This is informational / a narrowing claim ONLY — it is NOT the
    /// trust anchor.
    ///
    /// # Codex #1 soundness fix
    /// The verifier's trust anchor `K` is an EXTERNAL relying-party input
    /// ([`crate::verifier::KeySet`]), passed into
    /// [`crate::verifier::verify_manifest`], NOT this field. Trusting this
    /// prover-supplied field as the anchor was a soundness hole: a prover signs a
    /// forged commitment with its own key and self-lists it here. The verifier
    /// now (a) checks every attestation key against the EXTERNAL `K`, and (b)
    /// requires this declared `key_set` to be a SUBSET of the external `K` (a
    /// prover may narrow but never widen the trust set). An empty external `K`
    /// trusts no issuer — any scan carrying commitments is then rejected.
    // [OPUS-4.8] audit #3 / codex #1: prover-declared, NOT the trust anchor.
    #[serde(default)]
    pub key_set: Vec<String>,
    /// Issuer attestations over the per-graph commitments (audit #3): one per
    /// distinct `commitments[g]` value that any scan sub-proof carries. The
    /// verifier requires every scan commitment to have a matching attestation
    /// whose signature is valid and whose key is in `key_set`.
    // [OPUS-4.8] audit #3.
    #[serde(default)]
    pub commitment_attestations: Vec<CommitmentAttestation>,
    /// Per-pattern graph attribution sets (which committed graph indices each
    /// BGP pattern may draw from) — fed to `verify::recheck` for the Q6
    /// cross-graph bnode-join guard.
    pub attributions: Vec<Vec<usize>>,
    /// Declared non-bnode join obligations (manifest side of the layer-3
    /// gate). `(variable, pattern_i, pattern_j)`.
    #[serde(default)]
    pub join_obligations: Vec<(String, usize, usize)>,
    pub entailment_regime: EntailmentRegime,
    pub binding: BindingMode,
    /// The credential's revocation reference (audit #12): which status list,
    /// index, and version. Issuer-bound (see [`RevocationStatus`]). When ANY
    /// scan-covering attestation carries an issuer-bound status reference
    /// ([`CommitmentAttestation::status`]) this MUST be present and match it —
    /// an omitted `revocation` for a status-bound credential is REJECTED
    /// (fail-closed; the prover cannot drop the reference to skip the check).
    #[serde(default)]
    pub revocation: Option<RevocationStatus>,
    /// Disclosed status-list snapshots (audit #12): the bitstrings the verifier
    /// checks the credential's status bit against. Keyed (by the verifier) on
    /// `(status_list, version)`. The snapshot matching the credential's
    /// (issuer-bound) `revocation` reference must show `bit[index] == 0` and a
    /// version within the verifier's freshness window. A missing matching
    /// snapshot REJECTS.
    // [OPUS-4.8] audit #12.
    #[serde(default)]
    pub status_snapshots: Vec<StatusListSnapshot>,
    /// The composed sub-proofs (per-property circuit instances).
    pub sub_proofs: Vec<SubProof>,
    /// Binding-consistency edges between sub-proofs.
    #[serde(default)]
    pub binding_edges: Vec<BindingEdge>,
}

fn default_type() -> String {
    "urn:sparq:zk:ProofManifest".to_string()
}

impl ProofManifest {
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("manifest is serializable")
    }

    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}
