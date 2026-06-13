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
//! Stage 1+2 run WITHOUT bb (the fast structural gate); stage 3 is the
//! cryptographic gate. [`verify_manifest_structure`] is the fast path;
//! [`verify_manifest`] adds bb.

use crate::build::{derive_filter_int_id, derive_scan_id};
use crate::driver::{CircuitProver, DriverError};
use crate::manifest::{
    BindingEdge, BindingMode, CircuitId, FieldHex, ProofInputs, ProofManifest,
};
use sparq_zk::encode::encode_term;
use sparq_zk::field::{field_to_be_bytes_32, field_to_hex, Fr};
use sparq_zk::verify::{
    fragment_filters, fragment_pattern_consts, fragment_patterns, recheck, variable_slots,
    FilterCmp, JoinEdge, QueryFilter, VerifyError,
};
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

    // --- Stage 2b + 2c: query-correctness binding (audit #5/#6/#7/#10). ---
    // The bb public-input vector already cryptographically binds the scan
    // pattern constants and the FILTER op/bound/expected/operand_enc (audit
    // #1). This stage is the VERIFIER-SIDE check that those bound values match
    // the query the relying party reads in `manifest.query` — without it a
    // proof of one statement is presented under a different query.
    bind_query_correctness(manifest)?;

    Ok(required)
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

    // (2c) every query FILTER has a matching, slot-bound, true-verdict proof.
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
        let satisfied = manifest.binding_edges.iter().any(|edge| {
            filter_edge_satisfies(manifest, edge, &consts, &positions, *op, *bound)
        });
        if !satisfied {
            return Err(CheckError::UnboundFilter { variable: variable.clone() });
        }
    }
    Ok(())
}

/// Whether one binding edge witnesses a query FILTER `?v op bound` where `?v`
/// binds at `positions` (the `(pattern, slot)` pairs from `variable_slots`).
/// See [`bind_query_correctness`] for the four conditions.
fn filter_edge_satisfies(
    manifest: &ProofManifest,
    edge: &BindingEdge,
    consts: &[[Option<oxrdf::Term>; 3]],
    positions: &[(usize, usize)],
    op: FilterCmp,
    bound: u64,
) -> bool {
    let Some(scan) = manifest.sub_proofs.get(edge.from_proof) else { return false };
    let Some(filt) = manifest.sub_proofs.get(edge.to_proof) else { return false };
    // (3)+(4): the filter's bound op/bound match the query, verdict is true.
    match &filt.inputs {
        ProofInputs::FilterInt { op: f_op, bound: f_bound, expected, .. } => {
            if f_op.code() != op.code() || *f_bound != bound || !*expected {
                return false;
            }
        }
        _ => return false,
    }
    // (1)+(2): the edge's scan answers a query pattern where ?v binds, and the
    // edge's `from_slot` is exactly ?v's slot in that pattern.
    positions.iter().any(|(pi, si)| {
        consts
            .get(*pi)
            .is_some_and(|c| scan_matches_pattern(&scan.inputs, c))
            && edge.from_slot == *si
    })
}

/// Full verification: structure (stage 1+2) then the cryptographic gate
/// (stage 3). `prover` points at the `zk/compose/` workspace; `work_dir` is
/// scratch for bb artifacts.
///
/// Stage 3, per sub-proof, binds the declared statement to the proof (audit
/// #1/#2): (a) reconstruct the public-input byte vector from the DECLARED
/// `ProofInputs` using the verifier's challenge and assert byte-equality with
/// the proof's `public_inputs`; (b) recompute the CANONICAL member vk
/// verifier-side; (c) `bb verify` over (prover proof, reconstructed public
/// inputs, canonical vk). The prover-supplied vk and public-input bytes from
/// the blob are NEVER trusted.
pub fn verify_manifest(
    manifest: &ProofManifest,
    prover: &CircuitProver,
    work_dir: &Path,
) -> Result<(), CheckError> {
    verify_manifest_structure(manifest)?;

    // The challenge that MUST appear as public-input field 0 of every member.
    // It comes from the manifest's binding (a later agent binds this to a
    // verifier-issued fresh nonce + single-use store, audit #4 — the byte
    // binding into the reconstructed vector is done here).
    let challenge = match &manifest.binding {
        BindingMode::Challenge { challenge } => challenge,
        BindingMode::HolderPop { challenge, .. } => challenge,
    };

    for (i, sp) in manifest.sub_proofs.iter().enumerate() {
        if sp.proof_hex.is_empty() {
            return Err(CheckError::MissingProof { proof: i });
        }
        // Hardening: prover-controlled bytes are rejected, never panicked on.
        let blob = hex_decode(&sp.proof_hex)
            .ok_or(CheckError::MalformedProof { proof: i })?;
        let art = decode_artifacts(&blob).ok_or(CheckError::MalformedProof { proof: i })?;

        // (a) Reconstruct public inputs from the DECLARED statement (audit #1)
        // and assert byte-equality with the proof's public_inputs. This is the
        // single load-bearing binding: stages 1-2 check JSON, the proof is a
        // detached crypto object, and THIS ties them to the same statement.
        let reconstructed = reconstruct_public_inputs(&sp.inputs, challenge, i)?;
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
///   pattern_const_enc[3], rows[r][3] (row-major), row_count.
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

    /// `scan_k1_n16_r4` over the probe values. 21 fields * 32 = 672 bytes; the
    /// single active row plus 3 zero-padded rows exercise the row-major
    /// flattening and the pad-to-`r` path.
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
        };
        let got = reconstruct_public_inputs(&inputs, &fh("0x2a"), 0).unwrap();
        assert_eq!(got.len(), 672);
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
