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

/// Revocation status (plan §S2.5 revocation = hidden-index status-list). v1
/// ships the placeholder shape only: a reference to a status list and the
/// (hidden, in the full design) index. The verifier records it but does not
/// yet check liveness in-circuit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevocationStatus {
    /// IRI of the status-list credential.
    pub status_list: String,
    /// Index into the list. In the full design this is hidden and proved
    /// in-range-and-unset; v1 carries it in the clear (documented deferral).
    pub index: u64,
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
    /// distinct issuers; informational in v1 — signature check deferred).
    #[serde(default)]
    pub issuers: Vec<String>,
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
    /// Optional revocation placeholder.
    #[serde(default)]
    pub revocation: Option<RevocationStatus>,
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
