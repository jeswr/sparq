// [OPUS-4.8] written while Fable 5 unavailable — re-review when Fable returns.
//! Prover.toml emission for the circuit family.
//!
//! Each circuit `main`'s public + private inputs are written in declaration
//! order. We render field elements as decimal-free `0x` hex (nargo's toml
//! reader accepts `0x`-prefixed Field literals) and arrays as inline tables.
//! Private witnesses (graph encodings, filter digits) are supplied by the
//! prover driver, never present in the manifest.

use crate::build::{FilterSignedWitness, JoinWitness};
use crate::manifest::{CircuitId, FieldHex, ProofInputs};

/// Error returned by [`prover_toml_for`] when a `Prover.toml` cannot be emitted
/// for the given inputs.
///
/// [OPUS-4.8] sq-fi03 / PR #178: `prover_toml_for` is a public fn, so a premature
/// call on a not-yet-wired arm must surface a recoverable error rather than panic
/// (a `unimplemented!` in a public fn is a downstream-crash footgun).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProverTomlError {
    /// A [`ProofInputs::JoinEq`] was passed without its private [`JoinWitness`]:
    /// the `join_eq` member's private inputs
    /// (`enc_a`/`counts_a`/`enc_b`/`counts_b`/`row_a`/`row_b`/`blinding`) live in
    /// the witness, not the manifest, so a join Prover.toml cannot be emitted
    /// without it. The caller obtains the witness from [`crate::build::build_join`]
    /// and threads it through `prover_toml_for`'s `join_witness` parameter.
    // [OPUS-4.8] sq-r2s8: the join proving path is now implemented; this error is
    // the "witness omitted for a JoinEq input" recoverable failure (no panic in a
    // public fn).
    JoinEqMissingWitness,
    /// A [`ProofInputs::FilterSignedInt`] or [`ProofInputs::FilterDecimal`] was
    /// passed without its private [`crate::build::FilterSignedWitness`]: the
    /// operand's SIGN flag and canonical digits live in the witness, not the
    /// manifest, so the Prover.toml cannot be emitted without it. The caller obtains
    /// the witness from [`crate::build::build_filter_signed_int`] /
    /// [`crate::build::build_filter_decimal`] and threads it through
    /// `prover_toml_for`'s `filter_signed_witness` parameter.
    // [OPUS-4.8] sq-7lrq: signed/decimal proving path; witness-omitted recoverable
    // failure (no panic in a public fn).
    FilterSignedMissingWitness,
}

impl std::fmt::Display for ProverTomlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProverTomlError::JoinEqMissingWitness => write!(
                f,
                "join_eq Prover.toml generation requires the private JoinWitness \
                 (enc_a/counts_a/enc_b/counts_b/row_a/row_b/blinding) — obtain it \
                 from build_join and pass it via prover_toml_for's join_witness arg"
            ),
            ProverTomlError::FilterSignedMissingWitness => write!(
                f,
                "signed-int / decimal FILTER Prover.toml generation requires the \
                 private FilterSignedWitness (sign flag + canonical digits) — obtain \
                 it from build_filter_signed_int / build_filter_decimal and pass it \
                 via prover_toml_for's filter_signed_witness arg"
            ),
        }
    }
}

impl std::error::Error for ProverTomlError {}

/// Render the `Prover.toml` body for a scan proof.
///
/// Order MUST match `scan_k{k}_n{n}_r{r}/src/main.nr`:
/// challenge, commitments, pattern_is_const, pattern_const_enc, rows,
/// row_count, attribution, counts, enc.
#[allow(clippy::too_many_arguments)]
pub fn scan_prover_toml(
    challenge: &FieldHex,
    commitments: &[FieldHex],
    pattern_is_const: &[bool; 3],
    pattern_const_enc: &[FieldHex; 3],
    rows: &[[FieldHex; 3]],
    row_count: u32,
    attribution: &[bool],
    counts: &[u32],
    enc: &[Vec<[FieldHex; 3]>],
) -> String {
    let mut s = String::new();
    s.push_str(&format!("challenge = \"{}\"\n", challenge.0));
    s.push_str(&format!("commitments = {}\n", hex_array(commitments)));
    s.push_str(&format!(
        "pattern_is_const = [{}]\n",
        pattern_is_const
            .iter()
            .map(|b| b.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    ));
    s.push_str(&format!(
        "pattern_const_enc = {}\n",
        hex_array(pattern_const_enc)
    ));
    s.push_str(&format!("rows = {}\n", rows_array(rows)));
    s.push_str(&format!("row_count = \"{row_count}\"\n"));
    // attribution: [bool; K] (audit #8) -- declared right after row_count.
    s.push_str(&format!(
        "attribution = [{}]\n",
        attribution
            .iter()
            .map(|b| b.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    ));
    s.push_str(&format!(
        "counts = [{}]\n",
        counts
            .iter()
            .map(|c| format!("\"{c}\""))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    // enc: [[[Field;3];N];K]
    s.push_str("enc = [");
    for (gi, graph) in enc.iter().enumerate() {
        if gi > 0 {
            s.push_str(", ");
        }
        s.push_str(&rows_array(graph));
    }
    s.push_str("]\n");
    s
}

/// Render the `Prover.toml` body for a filter_int proof. Order MUST match
/// `filter_int_d{d}/src/main.nr`: challenge, operand_enc, op, bound, expected,
/// digits.
pub fn filter_int_prover_toml(
    challenge: &FieldHex,
    operand_enc: &FieldHex,
    op: u32,
    bound: u64,
    expected: bool,
    digits: &[u8],
) -> String {
    let mut s = String::new();
    s.push_str(&format!("challenge = \"{}\"\n", challenge.0));
    s.push_str(&format!("operand_enc = \"{}\"\n", operand_enc.0));
    s.push_str(&format!("op = \"{op}\"\n"));
    s.push_str(&format!("bound = \"{bound}\"\n"));
    s.push_str(&format!("expected = {expected}\n"));
    s.push_str(&format!(
        "digits = [{}]\n",
        digits
            .iter()
            .map(|d| format!("\"{d}\""))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    s
}

/// Render the `Prover.toml` body for a MANIFEST-COMPOSABLE filter_f64 proof
/// ([OPUS-4.8] sq-q7e / sq-tat). Order MUST match `filter_f64_d{d}/src/main.nr`:
/// challenge, operand_enc, op, b_bits, expected, digits.
pub fn filter_f64_prover_toml(
    challenge: &FieldHex,
    operand_enc: &FieldHex,
    op: u32,
    b_bits: u64,
    expected: bool,
    digits: &[u8],
) -> String {
    let mut s = String::new();
    s.push_str(&format!("challenge = \"{}\"\n", challenge.0));
    s.push_str(&format!("operand_enc = \"{}\"\n", operand_enc.0));
    s.push_str(&format!("op = \"{op}\"\n"));
    s.push_str(&format!("b_bits = \"{b_bits}\"\n"));
    s.push_str(&format!("expected = {expected}\n"));
    s.push_str(&format!(
        "digits = [{}]\n",
        digits
            .iter()
            .map(|d| format!("\"{d}\""))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    s
}

/// Render a `[ "d0", "d1", … ]` inline array of decimal digit bytes (each byte
/// rendered as its ASCII codepoint string, matching the circuit's `[u8; N]`).
fn digits_array(digits: &[u8]) -> String {
    format!(
        "[{}]",
        digits
            .iter()
            .map(|d| format!("\"{d}\""))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

/// Render the `Prover.toml` body for a MANIFEST-COMPOSABLE `filter_signed_int`
/// proof ([OPUS-4.8] sq-7lrq). Order MUST match
/// `filter_signed_int_d{md}/src/main.nr`: PUBLIC `challenge, operand_enc, op,
/// bound_neg, bound, expected` then PRIVATE `neg, mag_digits`. `neg` is the hidden
/// operand's sign flag; `mag_digits` are its canonical MAGNITUDE digits (length
/// MD).
#[allow(clippy::too_many_arguments)]
pub fn filter_signed_int_prover_toml(
    challenge: &FieldHex,
    operand_enc: &FieldHex,
    op: u32,
    bound_neg: bool,
    bound: u64,
    expected: bool,
    neg: bool,
    mag_digits: &[u8],
) -> String {
    let mut s = String::new();
    s.push_str(&format!("challenge = \"{}\"\n", challenge.0));
    s.push_str(&format!("operand_enc = \"{}\"\n", operand_enc.0));
    s.push_str(&format!("op = \"{op}\"\n"));
    s.push_str(&format!("bound_neg = {bound_neg}\n"));
    s.push_str(&format!("bound = \"{bound}\"\n"));
    s.push_str(&format!("expected = {expected}\n"));
    s.push_str(&format!("neg = {neg}\n"));
    s.push_str(&format!("mag_digits = {}\n", digits_array(mag_digits)));
    s
}

/// Render the `Prover.toml` body for a MANIFEST-COMPOSABLE `filter_decimal` proof
/// ([OPUS-4.8] sq-7lrq). Order MUST match `filter_decimal_i{id}_f{fd}/src/main.nr`:
/// PUBLIC `challenge, operand_enc, op, bound_neg, bound_scaled, expected` then
/// PRIVATE `neg, int_digits, frac_digits`. `neg` is the hidden operand's sign flag;
/// `int_digits` (length ID) / `frac_digits` (length FD) are its canonical
/// integer-part / fraction digits.
#[allow(clippy::too_many_arguments)]
pub fn filter_decimal_prover_toml(
    challenge: &FieldHex,
    operand_enc: &FieldHex,
    op: u32,
    bound_neg: bool,
    bound_scaled: u64,
    expected: bool,
    neg: bool,
    int_digits: &[u8],
    frac_digits: &[u8],
) -> String {
    let mut s = String::new();
    s.push_str(&format!("challenge = \"{}\"\n", challenge.0));
    s.push_str(&format!("operand_enc = \"{}\"\n", operand_enc.0));
    s.push_str(&format!("op = \"{op}\"\n"));
    s.push_str(&format!("bound_neg = {bound_neg}\n"));
    s.push_str(&format!("bound_scaled = \"{bound_scaled}\"\n"));
    s.push_str(&format!("expected = {expected}\n"));
    s.push_str(&format!("neg = {neg}\n"));
    s.push_str(&format!("int_digits = {}\n", digits_array(int_digits)));
    s.push_str(&format!("frac_digits = {}\n", digits_array(frac_digits)));
    s
}

/// Render the `Prover.toml` body for a hidden cross-credential `join_eq` proof
/// ([OPUS-4.8] sq-r2s8). Order MUST match `join_eq_na{n_a}_nb{n_b}/src/main.nr`:
/// PUBLIC `challenge, commit_a, commit_b, join_commitment, slot_a, slot_b` then
/// PRIVATE `enc_a, counts_a, enc_b, counts_b, row_a, row_b, blinding`. `enc_a`/
/// `enc_b` are padded to `n_a`/`n_b` slots by the caller (mirrors the scan arm).
#[allow(clippy::too_many_arguments)]
pub fn join_prover_toml(
    challenge: &FieldHex,
    commit_a: &FieldHex,
    commit_b: &FieldHex,
    join_commitment: &FieldHex,
    slot_a: u32,
    slot_b: u32,
    enc_a: &[[FieldHex; 3]],
    counts_a: u32,
    enc_b: &[[FieldHex; 3]],
    counts_b: u32,
    row_a: &[FieldHex; 3],
    row_b: &[FieldHex; 3],
    blinding: &FieldHex,
) -> String {
    let mut s = String::new();
    s.push_str(&format!("challenge = \"{}\"\n", challenge.0));
    s.push_str(&format!("commit_a = \"{}\"\n", commit_a.0));
    s.push_str(&format!("commit_b = \"{}\"\n", commit_b.0));
    s.push_str(&format!("join_commitment = \"{}\"\n", join_commitment.0));
    s.push_str(&format!("slot_a = \"{slot_a}\"\n"));
    s.push_str(&format!("slot_b = \"{slot_b}\"\n"));
    s.push_str(&format!("enc_a = {}\n", rows_array(enc_a)));
    s.push_str(&format!("counts_a = \"{counts_a}\"\n"));
    s.push_str(&format!("enc_b = {}\n", rows_array(enc_b)));
    s.push_str(&format!("counts_b = \"{counts_b}\"\n"));
    s.push_str(&format!(
        "row_a = [\"{}\", \"{}\", \"{}\"]\n",
        row_a[0].0, row_a[1].0, row_a[2].0
    ));
    s.push_str(&format!(
        "row_b = [\"{}\", \"{}\", \"{}\"]\n",
        row_b[0].0, row_b[1].0, row_b[2].0
    ));
    s.push_str(&format!("blinding = \"{}\"\n", blinding.0));
    s
}

fn hex_array(items: &[FieldHex]) -> String {
    format!(
        "[{}]",
        items
            .iter()
            .map(|h| format!("\"{}\"", h.0))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn rows_array(rows: &[[FieldHex; 3]]) -> String {
    format!(
        "[{}]",
        rows.iter()
            .map(|r| format!(
                "[\"{}\", \"{}\", \"{}\"]",
                r[0].0, r[1].0, r[2].0
            ))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

/// Pad a hex-encoding list to `len` with zero field elements (the circuit's
/// inactive padding slots).
pub fn pad_hex(mut v: Vec<FieldHex>, len: usize) -> Vec<FieldHex> {
    while v.len() < len {
        v.push(FieldHex("0x0".to_string()));
    }
    v
}

/// Pad a list of rows to `len` with zero rows.
pub fn pad_rows(mut v: Vec<[FieldHex; 3]>, len: usize) -> Vec<[FieldHex; 3]> {
    let zero = || FieldHex("0x0".to_string());
    while v.len() < len {
        v.push([zero(), zero(), zero()]);
    }
    v
}

/// Sanity helper: digit bytes of a non-negative integer's canonical decimal.
pub fn canonical_digits(value: u64) -> Vec<u8> {
    let s = value.to_string();
    s.bytes().collect()
}

/// Render the witness-bearing `Prover.toml` for any `ProofInputs`, given the
/// private witnesses the manifest does not carry. Returns the package id too.
///
/// For [`ProofInputs::JoinEq`] the private [`JoinWitness`] MUST be supplied via
/// `join_witness` (obtained from [`crate::build::build_join`]); omitting it returns
/// [`ProverTomlError::JoinEqMissingWitness`] — a recoverable error, never a panic
/// (a public fn must not crash a downstream caller). The `join_witness` argument is
/// ignored for every non-join input, exactly as `scan_*` is ignored for filters.
/// [OPUS-4.8] sq-r2s8: the join_eq proving path is now implemented.
///
/// For [`ProofInputs::FilterSignedInt`] / [`ProofInputs::FilterDecimal`] the private
/// [`FilterSignedWitness`] (operand sign + canonical digits) MUST be supplied via
/// `filter_signed_witness`; omitting it returns
/// [`ProverTomlError::FilterSignedMissingWitness`] (recoverable, never a panic — the
/// `filter_digits` arg does NOT carry the sign these members need). The
/// `filter_signed_witness` argument is ignored for every other input.
/// [OPUS-4.8] sq-7lrq: the signed-int / decimal proving path is now implemented.
#[allow(clippy::too_many_arguments)]
pub fn prover_toml_for(
    inputs: &ProofInputs,
    challenge: &FieldHex,
    // scan witnesses (ignored for filter): per-graph active triple-counts and
    // per-graph per-slot encodings.
    scan_counts: &[u32],
    scan_enc: &[Vec<[FieldHex; 3]>],
    // filter witness (ignored for scan): canonical decimal digits (filter_int /
    // filter_f64). Signed-int / decimal carry their digits in `filter_signed_witness`.
    filter_digits: &[u8],
    // join witness (ignored for scan/filter): the join_eq member's private inputs
    // (enc_a/counts_a/enc_b/counts_b/row_a/row_b/blinding). [OPUS-4.8] sq-r2s8.
    join_witness: Option<&JoinWitness>,
    // signed-int / decimal witness (ignored for every other input): the operand's
    // PRIVATE sign flag + canonical digits. [OPUS-4.8] sq-7lrq.
    filter_signed_witness: Option<&FilterSignedWitness>,
) -> Result<(CircuitId, String), ProverTomlError> {
    let out = match inputs {
        ProofInputs::Scan {
            id,
            commitments,
            pattern_is_const,
            pattern_const_enc,
            rows,
            row_count,
            attribution,
        } => {
            let CircuitId::Scan { n, r, .. } = id else {
                unreachable!("scan inputs carry a scan id")
            };
            let rows = pad_rows(rows.clone(), *r as usize);
            // Pad each graph's enc to N slots.
            let enc: Vec<Vec<[FieldHex; 3]>> = scan_enc
                .iter()
                .map(|g| pad_rows(g.clone(), *n as usize))
                .collect();
            let toml = scan_prover_toml(
                challenge,
                commitments,
                pattern_is_const,
                pattern_const_enc,
                &rows,
                *row_count,
                attribution,
                scan_counts,
                &enc,
            );
            (id.clone(), toml)
        }
        ProofInputs::FilterInt {
            id,
            operand_enc,
            op,
            bound,
            expected,
        } => {
            let toml = filter_int_prover_toml(
                challenge,
                operand_enc,
                op.code(),
                *bound,
                *expected,
                filter_digits,
            );
            (id.clone(), toml)
        }
        // [OPUS-4.8] sq-q7e + sq-tat: composable xsd:double FILTER. `filter_digits`
        // carries the integer-valued double's canonical decimal digits (same role
        // as the filter_int digit witness); `b_bits` is the constant double's IEEE
        // bit pattern.
        ProofInputs::FilterF64 {
            id,
            operand_enc,
            op,
            b_bits,
            expected,
        } => {
            let toml = filter_f64_prover_toml(
                challenge,
                operand_enc,
                op.code(),
                *b_bits,
                *expected,
                filter_digits,
            );
            (id.clone(), toml)
        }
        // [OPUS-4.8] sq-7lrq: composable SIGNED xsd:integer FILTER. The operand's
        // PRIVATE sign flag + magnitude digits come from `filter_signed_witness`
        // (built by `build_filter_signed_int`); omitting it is a recoverable `Err`
        // (no panic in a public fn). `frac_digits` is empty for signed-int.
        ProofInputs::FilterSignedInt {
            id,
            operand_enc,
            op,
            bound_neg,
            bound,
            expected,
        } => {
            let w = filter_signed_witness.ok_or(ProverTomlError::FilterSignedMissingWitness)?;
            let toml = filter_signed_int_prover_toml(
                challenge,
                operand_enc,
                op.code(),
                *bound_neg,
                *bound,
                *expected,
                w.neg,
                &w.int_digits,
            );
            (id.clone(), toml)
        }
        // [OPUS-4.8] sq-7lrq: composable xsd:decimal FILTER. The operand's PRIVATE
        // sign flag + integer-part / fraction digits come from `filter_signed_witness`
        // (built by `build_filter_decimal`); `bound_scaled` is the host-prescaled
        // constant magnitude. Omitting the witness is a recoverable `Err`.
        ProofInputs::FilterDecimal {
            id,
            operand_enc,
            op,
            bound_neg,
            bound_scaled,
            expected,
        } => {
            let w = filter_signed_witness.ok_or(ProverTomlError::FilterSignedMissingWitness)?;
            let toml = filter_decimal_prover_toml(
                challenge,
                operand_enc,
                op.code(),
                *bound_neg,
                *bound_scaled,
                *expected,
                w.neg,
                &w.int_digits,
                &w.frac_digits,
            );
            (id.clone(), toml)
        }
        // [OPUS-4.8] sq-bwwl / sq-r2s8 (step 4 proving path): hidden cross-credential
        // JOIN. The public inputs (commit_a/commit_b/join_commitment/slot_a/slot_b)
        // come from `inputs`; the PRIVATE witnesses
        // (enc_a/counts_a/enc_b/counts_b/row_a/row_b/blinding) come from the
        // `join_witness` the caller built with `build_join`. Omitting it is a
        // recoverable `Err` (no panic in a public fn). `enc_a`/`enc_b` are padded to
        // the member's `n_a`/`n_b` buckets, exactly as the scan arm pads `enc`.
        ProofInputs::JoinEq {
            id,
            commit_a,
            commit_b,
            join_commitment,
            slot_a,
            slot_b,
        } => {
            let CircuitId::JoinEq { n_a, n_b } = id else {
                unreachable!("join_eq inputs carry a join_eq id")
            };
            let w = join_witness.ok_or(ProverTomlError::JoinEqMissingWitness)?;
            let enc_a = pad_rows(w.enc_a.clone(), *n_a as usize);
            let enc_b = pad_rows(w.enc_b.clone(), *n_b as usize);
            let toml = join_prover_toml(
                challenge,
                commit_a,
                commit_b,
                join_commitment,
                *slot_a,
                *slot_b,
                &enc_a,
                w.counts_a,
                &enc_b,
                w.counts_b,
                &w.row_a,
                &w.row_b,
                &w.blinding,
            );
            (id.clone(), toml)
        }
    };
    Ok(out)
}
