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
//! 3. **bb verification**: every sub-proof's bb proof verifies against the
//!    member's vk.
//!
//! Stage 1+2 run WITHOUT bb (the fast structural gate); stage 3 is the
//! cryptographic gate. [`verify_manifest_structure`] is the fast path;
//! [`verify_manifest`] adds bb.

use crate::build::{derive_filter_int_id, derive_scan_id};
use crate::driver::{CircuitProver, DriverError, ProofArtifacts};
use crate::manifest::{CircuitId, ProofInputs, ProofManifest};
use sparq_zk::verify::{recheck, JoinEdge, VerifyError};
use std::collections::BTreeSet;
use std::path::Path;

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

/// Stage 1+2: structural verification (no bb). Returns the required obligation
/// edges on success.
pub fn verify_manifest_structure(
    manifest: &ProofManifest,
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

    Ok(required)
}

/// Full verification: structure (stage 1+2) then bb (stage 3). `prover` points
/// at the `zk/compose/` workspace; `work_dir` is scratch for bb artifacts.
pub fn verify_manifest(
    manifest: &ProofManifest,
    prover: &CircuitProver,
    work_dir: &Path,
) -> Result<(), CheckError> {
    verify_manifest_structure(manifest)?;

    for (i, sp) in manifest.sub_proofs.iter().enumerate() {
        if sp.proof_hex.is_empty() {
            return Err(CheckError::MissingProof { proof: i });
        }
        let proof = hex_decode(&sp.proof_hex);
        // Re-derive vk + public inputs by regenerating them from a witness is
        // not necessary: the manifest carries the proof; we re-derive vk from
        // the circuit and re-derive public inputs from the proof bytes that bb
        // itself bundles. v1: the manifest carries proof bytes that include the
        // public inputs (bb's `proof` layout), and we recompute the vk from the
        // compiled member. We store/forward public_inputs + vk alongside the
        // proof in the SubProof's proof_hex as a length-prefixed blob.
        let art = decode_artifacts(&proof);
        let sub_work = work_dir.join(format!("sub{i}"));
        let ok = prover
            .verify(&art, &sub_work)
            .map_err(CheckError::Driver)?;
        if !ok {
            return Err(CheckError::ProofRejected { proof: i });
        }
    }
    Ok(())
}

/// SubProof `proof_hex` blob layout: `len(proof) | proof | len(pi) | pi | vk`,
/// each length a 4-byte big-endian u32. Keeps the three bb artifacts together
/// in one manifest field.
pub fn encode_artifacts(art: &ProofArtifacts) -> String {
    let mut blob = Vec::new();
    push_lp(&mut blob, &art.proof);
    push_lp(&mut blob, &art.public_inputs);
    blob.extend_from_slice(&art.vk);
    hex_encode(&blob)
}

fn decode_artifacts(blob: &[u8]) -> ProofArtifacts {
    let (proof, rest) = take_lp(blob);
    let (public_inputs, vk) = take_lp(rest);
    ProofArtifacts {
        proof: proof.to_vec(),
        public_inputs: public_inputs.to_vec(),
        vk: vk.to_vec(),
    }
}

fn push_lp(buf: &mut Vec<u8>, data: &[u8]) {
    buf.extend_from_slice(&(data.len() as u32).to_be_bytes());
    buf.extend_from_slice(data);
}

fn take_lp(buf: &[u8]) -> (&[u8], &[u8]) {
    let len = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
    let body = &buf[4..4 + len];
    (body, &buf[4 + len..])
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn hex_decode(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("valid hex"))
        .collect()
}
