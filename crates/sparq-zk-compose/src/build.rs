// [OPUS-4.8] written while Fable 5 unavailable — re-review when Fable returns.
//! Builds circuit inputs + witnesses from sparq-zk commitments.
//!
//! The (k, n, r) circuit-family id is DERIVED from the data: `k` = number of
//! committed graphs, `n` = smallest compiled slot-bucket >= the largest
//! graph, `r` = smallest compiled row-bucket >= the disclosed match count.
//! Prover and verifier both call [`derive_scan_id`] so a proof can only fit
//! its member (brief: "derive the circuit-family id from the proof manifest").

use crate::manifest::{CircuitId, FieldHex, FilterOp, ProofInputs};
use sparq_zk::commit::GraphCommitment;
use sparq_zk::encode::encode_term;
use sparq_zk::field::{field_to_be_bytes_32, field_to_hex, Fr};
use oxrdf::{NamedNode, Term};

/// Compiled slot buckets (`n`) of the scan family, ascending.
pub const SCAN_N_BUCKETS: &[u32] = &[16, 64];
/// Compiled row buckets (`r`) of the scan family, ascending.
pub const SCAN_R_BUCKETS: &[u32] = &[4, 8];
/// Compiled `k` values of the scan family.
pub const SCAN_K_VALUES: &[u32] = &[1, 2];
/// Compiled digit counts (`d`) of the filter_int family. The `filter_int_d{D}`
/// circuit's `digits: [u8; D]` witness requires the operand's decimal digit
/// count to EXACTLY equal `D` (see `compose_core::filter_int::filter_int_check`),
/// so a member is provable for an operand iff its digit count == `D`. v1 compiles
/// the contiguous range 1..=4 plus the historical gap-fill at 3 (sq-wto).
// [OPUS-4.8] sq-wto: 3 added so the {1,2,4} family is contiguous 1..=4 and no
// in-range digit count derives an unprovable wrong-D member.
pub const FILTER_INT_D_VALUES: &[u32] = &[1, 2, 3, 4];
/// Compiled digit counts (`d`) of the MANIFEST-COMPOSABLE filter_f64 family
/// ([OPUS-4.8] sq-q7e / sq-tat). Same EXACT-match discipline as `filter_int`:
/// `filter_f64_d{D}`'s `digits: [u8; D]` witness pins the operand's digit count
/// to `D`, so a member is provable iff the integer-valued double's decimal digit
/// count equals `D`. v1 compiles the contiguous 1..=4; `D <= 15` is the soundness
/// ceiling (value `< 2^53`, exact `f64::from`), so larger counts have no member
/// and `derive_filter_f64_id` returns `None` (clean error, never a wrong-D).
pub const FILTER_F64_D_VALUES: &[u32] = &[1, 2, 3, 4];
/// Compiled graph-size buckets (`n_a`/`n_b`) of the hidden-join `join_eq` family,
/// ascending (sq-bwwl / sq-fi03). The `join_eq_na{N_A}_nb{N_B}` member re-commits
/// each witnessed graph with `[[Field; 3]; N]` slots (the `join_eq_check<N_A,
/// N_B>` const generics), so a graph holding `<= N` triples is provable by the
/// smallest compiled bucket `>= N` — exactly the scan `n`-bucket discipline. v1
/// compiles the single symmetric `join_eq_na16_nb16` member, so only the `16`
/// bucket exists; an out-of-family size derives `None` (clean error). The
/// verifier gate that consumes the derived id is step 4 (sq-sfsi).
// [OPUS-4.8] sq-bwwl / sq-fi03 (step 3): compiled join_eq graph-size buckets.
pub const JOIN_EQ_N_BUCKETS: &[u32] = &[16];

fn smallest_bucket(buckets: &[u32], need: u32) -> Option<u32> {
    buckets.iter().copied().find(|&b| b >= need)
}

/// Derive the scan circuit id for `k` graphs whose largest holds `max_graph`
/// triples, disclosing `match_rows` rows. `None` if no compiled member fits.
pub fn derive_scan_id(k: u32, max_graph: u32, match_rows: u32) -> Option<CircuitId> {
    if !SCAN_K_VALUES.contains(&k) {
        return None;
    }
    let n = smallest_bucket(SCAN_N_BUCKETS, max_graph.max(1))?;
    let r = smallest_bucket(SCAN_R_BUCKETS, match_rows.max(1))?;
    Some(CircuitId::Scan { k, n, r })
}

/// Derive the filter_int circuit id for a `digits`-digit value.
///
/// # sq-wto correctness (EXACT match, not smallest-bucket-≥)
/// The `filter_int_d{D}` circuit pins the operand's digit count to `D` exactly:
/// its witness is `digits: [u8; D]` and `filter_int_check::<D>` rebuilds the
/// canonical token from exactly `D` digit bytes, so a `d`-digit operand is
/// provable ONLY by the `D == d` member. The old `smallest_bucket(.., digits)`
/// (smallest compiled `D >= digits`) was a COMPLETENESS BUG (sq-61g differential
/// fuzzer / sq-wto): a 3-digit operand derived `d = 4`, but no 3-digit value can
/// fill a `[u8; 4]` witness, so `nargo execute` produced no witness — an honest
/// FILTER that SILENTLY yields an unprovable statement. We now require an EXACT
/// compiled member and return `None` for any digit count with no `D == digits`
/// member (out-of-family => a CLEAN error at the call site, never a
/// silently-unprovable d). With the d3 member the provable range is the
/// contiguous 1..=4; 0 maps to the 1-digit canonical form ("0"); 5..=19 (and
/// >19, which overflows u64) have no member and return `None`.
// [OPUS-4.8] sq-wto: exact digit-count match; out-of-bucket => None (clean
// error), never a wrong-D silently-unprovable member.
pub fn derive_filter_int_id(digits: u32) -> Option<CircuitId> {
    let d = digits.max(1);
    if FILTER_INT_D_VALUES.contains(&d) {
        Some(CircuitId::FilterInt { d })
    } else {
        None
    }
}

/// Derive the MANIFEST-COMPOSABLE filter_f64 circuit id for a `digits`-digit
/// integer-valued double ([OPUS-4.8] sq-q7e / sq-tat). EXACT-match discipline,
/// identical to [`derive_filter_int_id`]: the `filter_f64_d{D}` circuit pins the
/// operand's digit count to `D` (`digits: [u8; D]`), so only the `D == digits`
/// member is provable. Out-of-family counts (no compiled `D == digits`, incl. the
/// `>15` soundness ceiling) return `None` — a clean error, never a wrong-D
/// silently-unprovable member.
// [OPUS-4.8] sq-q7e + sq-tat: exact digit-count match for the composable f64 family.
pub fn derive_filter_f64_id(digits: u32) -> Option<CircuitId> {
    let d = digits.max(1);
    if FILTER_F64_D_VALUES.contains(&d) {
        Some(CircuitId::FilterF64 { d })
    } else {
        None
    }
}

/// Derive the hidden-join `join_eq` circuit id for two graphs holding `n_a` /
/// `n_b` triples (sq-bwwl / sq-fi03). Mirrors [`derive_scan_id`]'s `n`-bucket
/// discipline: each side maps to the smallest compiled [`JOIN_EQ_N_BUCKETS`]
/// bucket `>= size` (the member re-commits `[[Field; 3]; N]` slots per graph, so
/// any graph with `<= N` triples fits the `N`-bucket member). `None` if either
/// side exceeds every compiled bucket — a clean error, never a wrong-N member.
/// Prover and verifier both derive the id this way so a `join_eq` proof can only
/// verify against the member its witnesses fit (audit-#2 canonical-vk discipline).
// [OPUS-4.8] sq-bwwl / sq-fi03 (step 3): derive the join_eq member id.
pub fn derive_join_eq_id(n_a: u32, n_b: u32) -> Option<CircuitId> {
    let n_a = smallest_bucket(JOIN_EQ_N_BUCKETS, n_a.max(1))?;
    let n_b = smallest_bucket(JOIN_EQ_N_BUCKETS, n_b.max(1))?;
    Some(CircuitId::JoinEq { n_a, n_b })
}

/// A constant or variable slot of a BGP triple pattern.
#[derive(Debug, Clone)]
pub enum Slot {
    Const(Term),
    Var,
}

/// A single BGP triple pattern (subject, predicate, object slots).
#[derive(Debug, Clone)]
pub struct Pattern {
    pub s: Slot,
    pub p: Slot,
    pub o: Slot,
}

/// The witnesses a scan proof needs but the manifest never carries.
#[derive(Debug, Clone)]
pub struct ScanWitness {
    /// Per-graph active triple counts.
    pub counts: Vec<u32>,
    /// Per-graph per-slot term encodings (active leaves; build pads to N).
    pub enc: Vec<Vec<[FieldHex; 3]>>,
}

/// Built scan proof: the public inputs + the private witness.
#[derive(Debug, Clone)]
pub struct BuiltScan {
    pub inputs: ProofInputs,
    pub witness: ScanWitness,
}

fn hexf(f: &Fr) -> FieldHex {
    FieldHex(field_to_hex(f))
}

/// Encode a BGP slot to the (is_const, const_enc) pair the circuit takes, and
/// also return the encoded constant (for row matching).
fn encode_slot(slot: &Slot, salt: &Fr) -> (bool, FieldHex, Option<Fr>) {
    match slot {
        Slot::Const(t) => {
            let enc = encode_term(t, salt).expect("committable constant term");
            (true, hexf(&enc), Some(enc))
        }
        Slot::Var => (false, FieldHex("0x0".to_string()), None),
    }
}

/// Build a scan proof's inputs + witnesses from per-graph commitments.
///
/// All graphs must share the salt convention used to commit them; the encoded
/// leaves come straight from each [`GraphCommitment`]'s recorded leaves' source
/// terms. We re-encode each canonical triple's terms (the circuit checks
/// `h3(enc) == leaf`), so this re-derivation is sound by construction: a wrong
/// encoding would fail the in-circuit commitment recompute.
///
/// # Strict-ordering canonicalisation (plan S2.5, [OPUS-4.8])
/// `scan_check` enforces `commitments[0] < commitments[1] < ...` (strict, on the
/// canonical field representative) to force the committed graphs pairwise
/// distinct and close the duplicate-inclusion / COUNT-forgery class. So this
/// builder emits the commitments -- and, in lock-step, every per-graph witness
/// (`counts`, `enc`) and the public `attribution` vector -- sorted ASCENDING by
/// the canonical big-endian commitment bytes (the exact order the in-circuit
/// `Field::lt` / the bb public-input word use). Sorting here (rather than
/// trusting caller order) means an honest k>=2 scan always satisfies the gate
/// regardless of the order the caller supplied its graphs in. If the caller
/// supplies two graphs with the SAME commitment (a genuine duplicate), the sort
/// places them adjacent and the in-circuit `<` then rejects -- the gate's intent.
pub fn build_scan(
    commitments: &[GraphCommitment],
    pattern: &Pattern,
) -> Option<BuiltScan> {
    let k = commitments.len() as u32;
    // S2.5: canonicalise graph order ascending by the commitment's field
    // representative so the strict-ordering gate (`scan_check` step 1b) is
    // satisfied for any honest input order. Sort the SAME canonical big-endian
    // bytes the circuit compares (`field_to_be_bytes_32`), so the host order
    // and the in-circuit `Field::lt` order agree exactly.
    let mut order: Vec<usize> = (0..commitments.len()).collect();
    order.sort_by(|&a, &b| {
        field_to_be_bytes_32(&commitments[a].commitment)
            .cmp(&field_to_be_bytes_32(&commitments[b].commitment))
    });
    let commitments: Vec<&GraphCommitment> = order.iter().map(|&i| &commitments[i]).collect();
    // Per-graph term encodings of every canonical triple.
    let mut enc: Vec<Vec<[FieldHex; 3]>> = Vec::with_capacity(commitments.len());
    let mut enc_fr: Vec<Vec<[Fr; 3]>> = Vec::with_capacity(commitments.len());
    let mut counts = Vec::with_capacity(commitments.len());
    for c in &commitments {
        let salt = c.salt;
        let mut graph_hex = Vec::new();
        let mut graph_fr = Vec::new();
        for t in &c.canonical.triples {
            let s = encode_term(&subj_term(t), &salt)?;
            let p = encode_term(&Term::NamedNode(t.predicate.clone()), &salt)?;
            let o = encode_term(&t.object, &salt)?;
            graph_hex.push([hexf(&s), hexf(&p), hexf(&o)]);
            graph_fr.push([s, p, o]);
        }
        counts.push(graph_hex.len() as u32);
        enc.push(graph_hex);
        enc_fr.push(graph_fr);
    }

    // Pattern encoding.
    // Each graph shares no single salt for IRIs/literals (those are
    // salt-independent), so constant encodings are stable across graphs; use
    // graph 0's salt as the reference (bnodes are never query constants).
    let ref_salt = commitments.first().map(|c| c.salt).unwrap_or(Fr::from(0u64));
    let (sc, se, sf) = encode_slot(&pattern.s, &ref_salt);
    let (pc, pe, pf) = encode_slot(&pattern.p, &ref_salt);
    let (oc, oe, of) = encode_slot(&pattern.o, &ref_salt);
    let pattern_is_const = [sc, pc, oc];
    let pattern_const_enc = [se, pe, oe];
    let const_fr = [sf, pf, of];

    // Disclosed rows: every active slot matching the pattern's constants. While
    // sweeping, record per-graph source attribution (audit #8): `attribution[g]`
    // is true iff graph `g` contributes at least one matched triple — the exact
    // bit `scan.nr` step 4 constrains. This is the proof-bound provenance the
    // verifier cross-checks against `manifest.attributions`.
    // [OPUS-4.8] audit #8.
    let mut rows: Vec<[FieldHex; 3]> = Vec::new();
    let mut attribution: Vec<bool> = Vec::with_capacity(enc_fr.len());
    for graph in &enc_fr {
        let mut graph_matches = false;
        for triple in graph {
            let matches = const_fr.iter().enumerate().all(|(i, c)| match c {
                Some(cf) => triple[i] == *cf,
                None => true,
            });
            if matches {
                graph_matches = true;
                rows.push([hexf(&triple[0]), hexf(&triple[1]), hexf(&triple[2])]);
            }
        }
        attribution.push(graph_matches);
    }
    let row_count = rows.len() as u32;
    let max_graph = counts.iter().copied().max().unwrap_or(0);
    let id = derive_scan_id(k, max_graph, row_count)?;

    let commitments_hex: Vec<FieldHex> =
        commitments.iter().map(|c| hexf(&c.commitment)).collect();

    Some(BuiltScan {
        inputs: ProofInputs::Scan {
            id,
            commitments: commitments_hex,
            pattern_is_const,
            pattern_const_enc,
            rows,
            row_count,
            attribution,
        },
        witness: ScanWitness { counts, enc },
    })
}

/// Build a filter_int proof for a hidden xsd:integer operand `value` under
/// operator `op` against `bound`, with the disclosed `expected` verdict.
/// `operand_enc` must be the same term encoding the scan proof disclosed for
/// the operand column (binding consistency).
pub fn build_filter_int(
    operand_enc: FieldHex,
    value: u64,
    op: FilterOp,
    bound: u64,
    expected: bool,
) -> Option<(ProofInputs, Vec<u8>)> {
    let digits = value.to_string();
    let id = derive_filter_int_id(digits.len() as u32)?;
    let digit_bytes: Vec<u8> = digits.bytes().collect();
    Some((
        ProofInputs::FilterInt { id, operand_enc, op, bound, expected },
        digit_bytes,
    ))
}

/// Encode an xsd:integer literal `value` to its term encoding under `salt`
/// (salt-independent for literals; convenience for callers wiring a filter to
/// a known constant).
pub fn encode_int_literal(value: u64) -> FieldHex {
    let lit = oxrdf::Literal::new_typed_literal(
        value.to_string(),
        NamedNode::new("http://www.w3.org/2001/XMLSchema#integer").unwrap(),
    );
    let enc = encode_term(&Term::Literal(lit), &Fr::from(0u64))
        .expect("integer literal is committable");
    hexf(&enc)
}

/// XSD double datatype IRI (the composable filter_f64 fragment's literal type).
const XSD_DOUBLE: &str = "http://www.w3.org/2001/XMLSchema#double";

/// Build a MANIFEST-COMPOSABLE filter_f64 proof for a hidden INTEGER-VALUED
/// xsd:double operand `value` (written in plain canonical decimal-integer lexical
/// form, e.g. `42` => `"42"^^xsd:double`) under operator `op` against the constant
/// double `bound`, with the disclosed `expected` verdict ([OPUS-4.8] sq-q7e /
/// sq-tat). `operand_enc` must be the SAME term encoding the scan proof disclosed
/// for the operand column (binding consistency, exactly like `build_filter_int`).
///
/// Returns `(inputs, digit_bytes)` or `None` if `value`'s digit count has no
/// compiled `filter_f64_d{D}` member (out-of-family => clean error, never a
/// silently-unprovable wrong-D — sq-wto discipline). `bound` is the FILTER's
/// constant double; its IEEE-754 bit pattern is carried as `b_bits` and the
/// verdict follows IEEE semantics (NaN-correct via `sparq_ieee754`).
///
/// # Fragment (honest scope)
/// The hidden operand must be an integer-valued double in plain decimal form: the
/// circuit derives the IEEE bits as `f64::from(value)` (exact for the <= 2^53
/// fragment) and binds `operand_enc` to the `"<digits>"^^xsd:double` token. A
/// fractional/scientific operand is outside this member (no in-circuit
/// decimal→IEEE parser — deferred). `value` itself is a `u64` here, so the host
/// caller wires an integer-valued operand by construction.
pub fn build_filter_f64(
    operand_enc: FieldHex,
    value: u64,
    op: FilterOp,
    bound: f64,
    expected: bool,
) -> Option<(ProofInputs, Vec<u8>)> {
    let digits = value.to_string();
    let id = derive_filter_f64_id(digits.len() as u32)?;
    let digit_bytes: Vec<u8> = digits.bytes().collect();
    Some((
        ProofInputs::FilterF64 { id, operand_enc, op, b_bits: bound.to_bits(), expected },
        digit_bytes,
    ))
}

/// Encode an INTEGER-VALUED xsd:double literal `value` (plain decimal-integer
/// lexical form `"<value>"^^xsd:double`) to its term encoding — the convenience
/// wiring a composable float filter to a known constant operand. The lexical form
/// is `value.to_string()` (no `.0`/exponent), matching the
/// `filter_f64_composable_check` token rebuild. (Salt-independent for literals.)
// [OPUS-4.8] sq-q7e + sq-tat.
pub fn encode_double_literal(value: u64) -> FieldHex {
    let lit = oxrdf::Literal::new_typed_literal(
        value.to_string(),
        NamedNode::new(XSD_DOUBLE).unwrap(),
    );
    let enc = encode_term(&Term::Literal(lit), &Fr::from(0u64))
        .expect("double literal is committable");
    hexf(&enc)
}

fn subj_term(t: &oxrdf::Triple) -> Term {
    match &t.subject {
        oxrdf::NamedOrBlankNode::NamedNode(n) => Term::NamedNode(n.clone()),
        oxrdf::NamedOrBlankNode::BlankNode(b) => Term::BlankNode(b.clone()),
    }
}
