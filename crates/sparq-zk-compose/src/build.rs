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
use sparq_zk::sig::join_value_commitment;
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
/// Compiled MAGNITUDE-digit counts (`md`) of the MANIFEST-COMPOSABLE
/// `filter_signed_int` family ([OPUS-4.8] sq-7lrq, the sq-1q9h members). Same
/// EXACT-match discipline as `filter_int` / `filter_f64`: the
/// `filter_signed_int_d{MD}` circuit's `mag_digits: [u8; MD]` witness pins the
/// operand's magnitude-digit count to `MD` (`filter_signed_int_check::<MD>` rebuilds
/// the canonical `"[-]?<digits>"^^<…#integer>` token from exactly `MD` digits), so a
/// member is provable for an operand iff its magnitude-digit count == `MD`. v1
/// compiles `{2, 4}` (the two members sq-1q9h landed: 2-digit negative coordinates
/// and 4-digit signed amounts). An out-of-family magnitude-digit count returns
/// `None` from [`derive_filter_signed_int_id`] (clean error, never a wrong-MD
/// silently-unprovable member — sq-wto). `MD <= 19` is the `u64`-magnitude ceiling.
pub const FILTER_SIGNED_INT_MD_VALUES: &[u32] = &[2, 4];
/// Compiled `(id, fd)` integer-/fraction-digit counts of the MANIFEST-COMPOSABLE
/// `filter_decimal` family ([OPUS-4.8] sq-7lrq, the sq-1q9h member). The
/// `filter_decimal_i{ID}_f{FD}` circuit pins the operand's integer-digit count to
/// `ID` AND its fraction-digit count to `FD` (the `int_digits: [u8; ID]` /
/// `frac_digits: [u8; FD]` witnesses), so a `(id, fd)`-shaped operand is provable
/// ONLY by the `(ID, FD) == (id, fd)` member. v1 compiles `(3, 2)` (the
/// `"123.45"`-shape member sq-1q9h landed); an out-of-family `(id, fd)` returns
/// `None` from [`derive_filter_decimal_id`] (clean error, never a wrong-shape member).
pub const FILTER_DECIMAL_ID_FD_VALUES: &[(u32, u32)] = &[(3, 2)];
/// Compiled graph-size buckets (`n_a`/`n_b`) of the hidden-join `join_eq` family,
/// ascending (sq-bwwl / sq-fi03). The `join_eq_na{N_A}_nb{N_B}` member re-commits
/// each witnessed graph with `[[Field; 3]; N]` slots (the `join_eq_check<N_A,
/// N_B>` const generics), so a graph holding `<= N` triples is provable by the
/// smallest compiled bucket `>= N` — exactly the scan `n`-bucket discipline. The
/// [`SCAN_N_BUCKETS`]-aligned `{16, 64}` set is compiled, in all four `(n_a, n_b)`
/// combinations (`na16_nb16`, `na16_nb64`, `na64_nb16`, `na64_nb64`), so a join
/// can compose with a scan over either scan-`n` bucket; an out-of-family size
/// (`> 64`) derives `None` (a clean error, never a wrong-N member). The verifier
/// gate that consumes the derived id is `bind_joins` (step 4, sq-sfsi).
// [OPUS-4.8] sq-bwwl / sq-fi03 (step 3): compiled join_eq graph-size buckets.
// [OPUS-4.8] sq-pzet (sq-8dx2): add the `64` bucket — the remaining (na, nb)
// members so a hidden join composes with the n=64 scan bucket. NOT a soundness
// change (the verifier remains NOT-yet-sound, sq-qhy4 / sq-9hrn / sq-1s2): the
// in-circuit relation is `join_eq_check` verbatim, only the bucket grows.
pub const JOIN_EQ_N_BUCKETS: &[u32] = &[16, 64];

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

/// Derive the MANIFEST-COMPOSABLE `filter_signed_int` circuit id for a
/// `mag_digits`-magnitude-digit SIGNED xsd:integer operand ([OPUS-4.8] sq-7lrq).
/// EXACT-match discipline, identical to [`derive_filter_int_id`]: the
/// `filter_signed_int_d{MD}` circuit pins the operand's MAGNITUDE-digit count to
/// `MD` (`mag_digits: [u8; MD]`; the optional leading `-` is bound into the token
/// but not counted in `MD`), so only the `MD == mag_digits` member is provable.
/// Out-of-family counts (no compiled `MD == mag_digits`) return `None` — a clean
/// error, never a wrong-MD silently-unprovable member (sq-wto).
// [OPUS-4.8] sq-7lrq: exact magnitude-digit match for the composable signed-int family.
pub fn derive_filter_signed_int_id(mag_digits: u32) -> Option<CircuitId> {
    let md = mag_digits.max(1);
    if FILTER_SIGNED_INT_MD_VALUES.contains(&md) {
        Some(CircuitId::FilterSignedInt { md })
    } else {
        None
    }
}

/// Derive the MANIFEST-COMPOSABLE `filter_decimal` circuit id for an operand with
/// `int_digits` integer-part digits and `frac_digits` fraction digits ([OPUS-4.8]
/// sq-7lrq). EXACT-match discipline over BOTH counts: the `filter_decimal_i{ID}_f{FD}`
/// circuit pins the operand's integer-digit count to `ID` AND its fraction-digit
/// count to `FD` (`int_digits: [u8; ID]` / `frac_digits: [u8; FD]`), so only the
/// `(ID, FD) == (int_digits, frac_digits)` member is provable. The integer part is
/// clamped to at least one digit (canonical `0.xx` has the single integer digit
/// `"0"`); `frac_digits` is taken verbatim (a decimal's fraction count is fixed by
/// its lexical form, including trailing zeros, which `oxrdf` preserves). An
/// out-of-family `(id, fd)` returns `None` — a clean error, never a wrong-shape
/// member.
// [OPUS-4.8] sq-7lrq: exact (id, fd) digit-shape match for the composable decimal family.
pub fn derive_filter_decimal_id(int_digits: u32, frac_digits: u32) -> Option<CircuitId> {
    let id = int_digits.max(1);
    let fd = frac_digits;
    if FILTER_DECIMAL_ID_FD_VALUES.contains(&(id, fd)) {
        Some(CircuitId::FilterDecimal { id, fd })
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

/// XSD integer / decimal datatype IRIs (the signed-int / decimal filter
/// fragments' literal types). [OPUS-4.8] sq-1q9h.
const XSD_INTEGER: &str = "http://www.w3.org/2001/XMLSchema#integer";
const XSD_DECIMAL: &str = "http://www.w3.org/2001/XMLSchema#decimal";

/// Build a MANIFEST-COMPOSABLE `filter_signed_int` proof for a hidden SIGNED
/// xsd:integer operand `value` under operator `op` against the constant `bound`
/// (also signed `i64`), with the disclosed `expected` verdict ([OPUS-4.8] sq-7lrq).
/// `operand_enc` must be the SAME term encoding the scan proof disclosed for the
/// operand column (binding consistency, exactly like [`build_filter_int`]).
///
/// Returns `(inputs, witness)` — the [`FilterSignedWitness`] carries the operand's
/// PRIVATE sign flag + canonical MAGNITUDE digits the manifest never holds — or
/// `None` if `value`'s MAGNITUDE digit count has no compiled
/// `filter_signed_int_d{MD}` member ([`derive_filter_signed_int_id`] => `None`, a
/// clean error, never a silently-unprovable wrong-MD — sq-wto). The witnessed
/// `mag_digits` are the canonical magnitude (`|value|.to_string()`, no sign); the
/// circuit re-derives the sign from the witnessed `neg` flag and the token. The
/// FILTER's constant `bound` is carried sign-split as `(bound_neg, |bound|)` in the
/// PUBLIC inputs. Thread the witness through [`crate::toml::prover_toml_for`]'s
/// `filter_signed_witness` arg (mirrors `build_join` -> the `join_witness` arg).
// [OPUS-4.8] sq-7lrq: composable signed xsd:integer FILTER builder.
pub fn build_filter_signed_int(
    operand_enc: FieldHex,
    value: i64,
    op: FilterOp,
    bound: i64,
    expected: bool,
) -> Option<(ProofInputs, FilterSignedWitness)> {
    let mag = value.unsigned_abs();
    let digits = mag.to_string();
    let id = derive_filter_signed_int_id(digits.len() as u32)?;
    let mag_digit_bytes: Vec<u8> = digits.bytes().collect();
    Some((
        ProofInputs::FilterSignedInt {
            id,
            operand_enc,
            op,
            bound_neg: bound < 0,
            bound: bound.unsigned_abs(),
            expected,
        },
        FilterSignedWitness {
            neg: value < 0,
            int_digits: mag_digit_bytes,
            frac_digits: Vec::new(),
        },
    ))
}

/// Build a MANIFEST-COMPOSABLE `filter_decimal` proof for a hidden xsd:decimal
/// operand given its `neg` sign, canonical integer-part digit string `int_part`, and
/// EXACTLY `frac` fraction-digit string (lexical form `[-]?<int>.<frac>`, e.g.
/// `(false, "123", "45")` => `"123.45"`), under operator `op` against a HOST-PRESCALED
/// constant `(bound_neg, bound_scaled)` where `bound_scaled = round(|bound| * 10^fd)`
/// and `fd == frac.len()` ([OPUS-4.8] sq-7lrq), with the disclosed `expected` verdict.
/// `operand_enc` must be the SAME term encoding the scan proof disclosed for the
/// operand column (binding consistency, exactly like [`build_filter_int`]).
///
/// The caller supplies the operand digits as STRINGS (not a parsed number) because a
/// decimal's lexical form fixes BOTH digit counts — including a leading integer `0`
/// (`"0.50"`) or trailing fraction zeros (`"1.50"`) — which the committed token, and
/// hence the member shape, must match byte-for-byte. The caller is responsible for
/// prescaling `bound` to `bound_scaled` at `fd = frac.len()` decimal places.
///
/// Returns `(inputs, witness)` — the [`FilterSignedWitness`] carries the operand's
/// PRIVATE sign flag + canonical integer-part and fraction digits — or `None` if the
/// operand's `(int_part.len(), frac.len())` shape has no compiled
/// `filter_decimal_i{ID}_f{FD}` member ([`derive_filter_decimal_id`] => `None`, a
/// clean error, never a silently-unprovable wrong shape — sq-wto), or if either
/// string contains a non-ASCII-digit byte. Thread the witness through
/// [`crate::toml::prover_toml_for`]'s `filter_signed_witness` arg.
// [OPUS-4.8] sq-7lrq: composable xsd:decimal FILTER builder.
#[allow(clippy::too_many_arguments)]
pub fn build_filter_decimal(
    operand_enc: FieldHex,
    neg: bool,
    int_part: &str,
    frac: &str,
    op: FilterOp,
    bound_neg: bool,
    bound_scaled: u64,
    expected: bool,
) -> Option<(ProofInputs, FilterSignedWitness)> {
    if !int_part.bytes().all(|b| b.is_ascii_digit())
        || !frac.bytes().all(|b| b.is_ascii_digit())
    {
        return None;
    }
    let id = derive_filter_decimal_id(int_part.len() as u32, frac.len() as u32)?;
    let int_digit_bytes: Vec<u8> = int_part.bytes().collect();
    let frac_digit_bytes: Vec<u8> = frac.bytes().collect();
    Some((
        ProofInputs::FilterDecimal {
            id,
            operand_enc,
            op,
            bound_neg,
            bound_scaled,
            expected,
        },
        FilterSignedWitness {
            neg,
            int_digits: int_digit_bytes,
            frac_digits: frac_digit_bytes,
        },
    ))
}

/// The PRIVATE witnesses a composable signed-int / decimal FILTER proof needs but
/// the manifest never carries — the operand's sign flag + canonical digits. For
/// [`ProofInputs::FilterSignedInt`] the digits live in [`Self::int_digits`] (the
/// magnitude digits) and [`Self::frac_digits`] is empty; for
/// [`ProofInputs::FilterDecimal`] both arrays carry the integer-part and fraction
/// digits. Mirrors [`JoinWitness`]: a non-manifest witness threaded through
/// [`crate::toml::prover_toml_for`]'s `filter_signed_witness` arg. [OPUS-4.8] sq-7lrq.
#[derive(Debug, Clone)]
pub struct FilterSignedWitness {
    /// The hidden operand's sign flag (`true` = negative). For signed-int this is
    /// bound into the canonical token via the leading `-`; for decimal likewise.
    pub neg: bool,
    /// Canonical digits of the operand's integer part (signed-int: the magnitude
    /// digits; decimal: the integer-part digits). Length is the member's `MD` / `ID`.
    pub int_digits: Vec<u8>,
    /// Canonical fraction digits (decimal only; EMPTY for signed-int). Length is the
    /// member's `FD`.
    pub frac_digits: Vec<u8>,
}

/// Encode a SIGNED `xsd:integer` literal `value` (canonical lexical form
/// `value.to_string()`, e.g. `-42` => `"-42"^^xsd:integer`) to its term
/// encoding — the host convenience wiring a `filter_signed_int` proof to a known
/// constant operand. The lexical form (with an optional leading `-`) matches the
/// `filter_signed_int_check` token rebuild byte-for-byte (oxrdf passes the
/// lexical form through verbatim). (Salt-independent for literals.)
/// [OPUS-4.8] sq-1q9h.
pub fn encode_signed_int_literal(value: i64) -> FieldHex {
    let lit = oxrdf::Literal::new_typed_literal(
        value.to_string(),
        NamedNode::new(XSD_INTEGER).unwrap(),
    );
    let enc = encode_term(&Term::Literal(lit), &Fr::from(0u64))
        .expect("integer literal is committable");
    hexf(&enc)
}

/// Encode an `xsd:decimal` literal from its sign, integer-part digits, and
/// EXACTLY `fd` fraction digits (lexical form `[-]?<int>.<frac>`, e.g.
/// `(false, 123, "45")` => `"123.45"^^xsd:decimal`) to its term encoding — the
/// host convenience wiring a `filter_decimal` proof to a known constant operand.
/// The lexical form matches the `filter_decimal_check` token rebuild
/// byte-for-byte. `frac` must be exactly the member's `FD` digits (zero-padded
/// by the caller, e.g. `"50"` for `0.50`). (Salt-independent for literals.)
/// [OPUS-4.8] sq-1q9h.
pub fn encode_decimal_literal(neg: bool, int_part: u64, frac: &str) -> FieldHex {
    let sign = if neg { "-" } else { "" };
    let lexical = format!("{sign}{int_part}.{frac}");
    let lit = oxrdf::Literal::new_typed_literal(lexical, NamedNode::new(XSD_DECIMAL).unwrap());
    let enc = encode_term(&Term::Literal(lit), &Fr::from(0u64))
        .expect("decimal literal is committable");
    hexf(&enc)
}

fn subj_term(t: &oxrdf::Triple) -> Term {
    match &t.subject {
        oxrdf::NamedOrBlankNode::NamedNode(n) => Term::NamedNode(n.clone()),
        oxrdf::NamedOrBlankNode::BlankNode(b) => Term::BlankNode(b.clone()),
    }
}

// === Hidden cross-credential JOIN (sq-bwwl / sq-r2s8, step 4 proving path) ====
// [OPUS-4.8] sq-r2s8: assemble the `join_eq` witness for proving — the host
// analogue of `build_scan`. Mirrors the scan build_* pattern: re-encode each
// witnessed graph's canonical triples (same salt convention the commitment used,
// so the in-circuit `commit_fold` recompute reproduces `C(G)`), locate the two
// rows whose chosen slots carry the SAME join value, and bind that value under a
// per-presentation blinder into the public hiding `join_commitment`. The join
// VALUE and both rows are PRIVATE witnesses; only the two commitments, the hiding
// commitment, and the two query-bound slots are public.

/// The private witnesses a `join_eq` proof needs but the manifest never carries —
/// exactly the `main` PRIVATE parameters of `join_eq_na{n_a}_nb{n_b}` (in
/// declaration order): the two graphs' per-slot encodings + active counts, the two
/// joined rows, and the per-presentation blinder. `build` does NOT pad to the
/// circuit's `N` slots; [`crate::toml::join_prover_toml`] pads `enc_a`/`enc_b` to
/// the member's bucket exactly as `prover_toml_for`'s scan arm pads `enc`.
// [OPUS-4.8] sq-r2s8: hidden cross-credential JOIN witness (mirrors ScanWitness).
#[derive(Debug, Clone)]
pub struct JoinWitness {
    /// Graph-A per-slot term encodings of every canonical triple (active leaves).
    pub enc_a: Vec<[FieldHex; 3]>,
    /// Active triple count of graph A (`<= n_a`).
    pub counts_a: u32,
    /// Graph-B per-slot term encodings.
    pub enc_b: Vec<[FieldHex; 3]>,
    /// Active triple count of graph B (`<= n_b`).
    pub counts_b: u32,
    /// The joined row of graph A (the row whose `slot_a` carries the join value).
    pub row_a: [FieldHex; 3],
    /// The joined row of graph B (the row whose `slot_b` carries the join value).
    pub row_b: [FieldHex; 3],
    /// The per-presentation blinder folded into the public `join_commitment`.
    pub blinding: FieldHex,
}

/// A built hidden-join proof: the public inputs ([`ProofInputs::JoinEq`]) + the
/// private witness ([`JoinWitness`]). Mirrors [`BuiltScan`].
// [OPUS-4.8] sq-r2s8.
#[derive(Debug, Clone)]
pub struct BuiltJoin {
    pub inputs: ProofInputs,
    pub witness: JoinWitness,
}

/// One graph's canonical triples re-encoded both as field elements (to locate the
/// joined rows) and as hex (the witness the toml emits). [OPUS-4.8] sq-r2s8.
type EncodedGraph = (Vec<[Fr; 3]>, Vec<[FieldHex; 3]>);

/// Re-encode every canonical triple of `c` under its recorded salt — the same
/// re-derivation [`build_scan`] performs. `None` if a term is not committable.
// [OPUS-4.8] sq-r2s8.
fn encode_join_graph(c: &GraphCommitment) -> Option<EncodedGraph> {
    let salt = c.salt;
    let mut fr = Vec::with_capacity(c.canonical.triples.len());
    let mut hex = Vec::with_capacity(c.canonical.triples.len());
    for t in &c.canonical.triples {
        let s = encode_term(&subj_term(t), &salt)?;
        let p = encode_term(&Term::NamedNode(t.predicate.clone()), &salt)?;
        let o = encode_term(&t.object, &salt)?;
        fr.push([s, p, o]);
        hex.push([hexf(&s), hexf(&p), hexf(&o)]);
    }
    Some((fr, hex))
}

/// Build a hidden cross-credential JOIN proof's inputs + witness from the two
/// graph commitments and the two query-bound join slots, blinding the joined
/// value with `blinding` (sq-bwwl / sq-r2s8, design §2.2/§3.2).
///
/// `slot_a`/`slot_b` (in `{0,1,2}` for s/p/o) are the query-derived positions the
/// shared variable occupies in each pattern — the SAME slots the verifier's
/// `bind_joins` gate pins (design §4.4). This builder finds the first `(row_a,
/// row_b)` whose `row_a[slot_a] == row_b[slot_b]` (the join value), so an honest
/// caller need only supply the two committed graphs and the slots. Both graphs
/// must use the salt convention recorded in their [`GraphCommitment`] (the
/// in-circuit `commit_fold` recomputes `C(G)` from the re-encoded leaves — a wrong
/// encoding would fail that recompute, so this re-derivation is sound by
/// construction, exactly as in [`build_scan`]).
///
/// Returns `None` if: either slot is out of `{0,1,2}`; no row pair shares a value
/// at the chosen slots (no honest join — the prover has no satisfying witness); or
/// either graph's size has no compiled [`JOIN_EQ_N_BUCKETS`] member
/// ([`derive_join_eq_id`] => `None`, a clean error, never a wrong-N member).
///
/// `blinding` is a per-presentation random field element; two presentations of the
/// same join value under different blinders produce UNLINKABLE `join_commitment`s
/// (design §2.4 R4). The caller draws it (e.g. from its CSPRNG mapped into the
/// field), exactly as the ingest salt is drawn.
// [OPUS-4.8] sq-r2s8: hidden cross-credential JOIN builder.
pub fn build_join(
    commit_a: &GraphCommitment,
    slot_a: u32,
    commit_b: &GraphCommitment,
    slot_b: u32,
    blinding: Fr,
) -> Option<BuiltJoin> {
    if slot_a > 2 || slot_b > 2 {
        return None;
    }

    // Re-encode each graph's canonical triples under its recorded salt — the same
    // re-derivation `build_scan` does (the leaf is `h3(enc)`, so a wrong encoding
    // fails the in-circuit commitment recompute). The field form locates the joined
    // rows; the hex form is the witness the toml emits.
    let (fr_a, enc_a) = encode_join_graph(commit_a)?;
    let (fr_b, enc_b) = encode_join_graph(commit_b)?;
    let counts_a = fr_a.len() as u32;
    let counts_b = fr_b.len() as u32;

    let id = derive_join_eq_id(counts_a, counts_b)?;

    // Locate the join: the first row pair whose chosen slots carry the same value.
    // `slot_a`/`slot_b` are bounds-checked above, so the indexing is in range.
    let (sa, sb) = (slot_a as usize, slot_b as usize);
    let mut joined: Option<(usize, usize, Fr)> = None;
    'outer: for (ia, ra) in fr_a.iter().enumerate() {
        for (ib, rb) in fr_b.iter().enumerate() {
            if ra[sa] == rb[sb] {
                joined = Some((ia, ib, ra[sa]));
                break 'outer;
            }
        }
    }
    let (ia, ib, value) = joined?;

    let row_a = enc_a[ia].clone();
    let row_b = enc_b[ib].clone();

    // Bind the join value under the per-presentation blinder — the in-circuit step
    // 5 recomputes this identically (single source of truth: the Rust + Noir
    // `join_value_commitment` agree byte-for-byte), so the public `join_commitment`
    // matches what the proof binds.
    let join_commitment = hexf(&join_value_commitment(&value, &blinding));

    Some(BuiltJoin {
        inputs: ProofInputs::JoinEq {
            id,
            commit_a: hexf(&commit_a.commitment),
            commit_b: hexf(&commit_b.commitment),
            join_commitment,
            slot_a,
            slot_b,
        },
        witness: JoinWitness {
            enc_a,
            counts_a,
            enc_b,
            counts_b,
            row_a,
            row_b,
            blinding: hexf(&blinding),
        },
    })
}
