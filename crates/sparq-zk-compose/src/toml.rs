// [OPUS-4.8] written while Fable 5 unavailable — re-review when Fable returns.
//! Prover.toml emission for the circuit family.
//!
//! Each circuit `main`'s public + private inputs are written in declaration
//! order. We render field elements as decimal-free `0x` hex (nargo's toml
//! reader accepts `0x`-prefixed Field literals) and arrays as inline tables.
//! Private witnesses (graph encodings, filter digits) are supplied by the
//! prover driver, never present in the manifest.

use crate::manifest::{CircuitId, FieldHex, ProofInputs};

/// Error returned by [`prover_toml_for`] when a `Prover.toml` cannot be emitted
/// for the given inputs.
///
/// [OPUS-4.8] sq-fi03 / PR #178: `prover_toml_for` is a public fn, so a premature
/// call on a not-yet-wired arm must surface a recoverable error rather than panic
/// (a `unimplemented!` in a public fn is a downstream-crash footgun).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProverTomlError {
    /// The `join_eq` Prover.toml emitter is deferred to sq-sfsi (step 4); sq-fi03
    /// adds the manifest schema only. The join member's private witnesses
    /// (`enc_a`/`counts_a`/`enc_b`/`counts_b`/`row_a`/`row_b`/`blinding`) are not
    /// yet plumbed through this function's parameters, so it cannot emit a witness.
    JoinEqUnsupported,
}

impl std::fmt::Display for ProverTomlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProverTomlError::JoinEqUnsupported => write!(
                f,
                "join_eq Prover.toml generation is not yet implemented (deferred to \
                 sq-sfsi, step 4); sq-fi03 adds the manifest schema only. The join \
                 member's private witnesses are not plumbed through prover_toml_for."
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
/// Returns [`ProverTomlError::JoinEqUnsupported`] for [`ProofInputs::JoinEq`],
/// whose prover path is deferred to sq-sfsi (step 4). This is a recoverable
/// error so a downstream caller (or a CLI loading a manifest containing
/// `join_eq`) gets a structured failure rather than a panic. [OPUS-4.8]
pub fn prover_toml_for(
    inputs: &ProofInputs,
    challenge: &FieldHex,
    // scan witnesses (ignored for filter): per-graph active triple-counts and
    // per-graph per-slot encodings.
    scan_counts: &[u32],
    scan_enc: &[Vec<[FieldHex; 3]>],
    // filter witness (ignored for scan): canonical decimal digits.
    filter_digits: &[u8],
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
        // [OPUS-4.8] sq-bwwl / sq-fi03 (step 3): hidden cross-credential JOIN.
        // SCHEMA ONLY — this bead adds the manifest types, NOT the prover path.
        // Generating a `join_eq` Prover.toml needs the join member's PRIVATE
        // witnesses (`enc_a`/`counts_a`/`enc_b`/`counts_b`/`row_a`/`row_b`/
        // `blinding`), none of which this function's `scan_*`/`filter_digits`
        // parameters can carry — so the prover wiring is deferred to step 4
        // (sq-sfsi), which extends the signature. Until then this arm returns a
        // recoverable `Err` (NOT a panic): a public fn must not crash a downstream
        // caller that loads a manifest containing `join_eq`. It also never emits a
        // bogus witness; sq-sfsi replaces it with the real join TOML emitter.
        ProofInputs::JoinEq { .. } => return Err(ProverTomlError::JoinEqUnsupported),
    };
    Ok(out)
}
