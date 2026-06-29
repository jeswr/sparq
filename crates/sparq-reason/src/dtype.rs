//! D-entailment (datatype / value-space) materialization — RDF 1.1 Semantics §7-8.
//!
//! [OPUS-4.8] sq-e5atd (epic sq-pbz04). OPT-IN, behind the `d-entail` cargo
//! feature — the production materializer never carries it, so the lean default
//! build (and the wasm bundle, which never depends on this code) is byte-identical.
//!
//! ## What D-entailment adds beyond simple entailment
//!
//! A *recognized datatype map* D fixes, for each datatype IRI it covers, a lexical
//! space, a value space, and the lexical-to-value mapping (RDF 1.1 Semantics §7).
//! The D-entailment lemma then sanctions, on top of simple entailment, the
//! **datatype typing rule rdfD1**: a well-formed literal `"l"^^d` of a recognized
//! datatype `d` is an instance of `d`, i.e. it entails `"l"^^d rdf:type d`. (In the
//! abstract semantics the literal is replaced by a *surrogate* allocated for its
//! value; the materializer keeps the literal in subject position — a GENERALIZED
//! triple — and lets the downstream regime restriction decide whether a surrogate
//! may surface in a query answer. This mirrors the conformance harness's
//! `harness_rules` rdfD1 arm.)
//!
//! The load-bearing invariant is **value-space equality**: two literals whose
//! values coincide are interchangeable under D — e.g. `"1"^^xsd:integer` and
//! `"1.0"^^xsd:decimal` denote the SAME value (the integers are a subset of the
//! decimals), so they must compare equal. [`d_value_key`] is the canonical
//! comparison: equal keys mean equal D-values, across lexical forms AND across the
//! integer/decimal value spaces that coincide.
//!
//! ## Why NOT an f64 fast path
//!
//! Collapsing the numeric value to `f64` is UNSOUND for D-equality: `f64` cannot
//! represent every `xsd:integer` (anything past 2^53 rounds) nor every
//! `xsd:decimal` exactly, so distinct values would alias and equal values past the
//! mantissa would diverge — silent precision loss in a SEMANTIC equality. Instead
//! the integer/decimal value space is compared as a CANONICAL DECIMAL STRING
//! (sign + minimal integer digits + minimal fraction digits, parsed exactly): this
//! is the correct typed comparison, exact at any magnitude.
//!
//! ## sparq-core substrate
//!
//! Datatype recognition (`is_integer_datatype`) and temporal value parsing
//! (`temporal::Temporal`) come from `sparq-core` — sparq-reason depends ONLY on
//! sparq-core, never sparq-engine. NOTE: the shared zero-overhead eval substrate's
//! typed `Num` compare (the substrate move-chain, research/ sq-6tykl) is not yet
//! wired; once `sparq-substrate::compare` lands, `d_value_key` migrates onto it
//! (one canonical typed-value comparator shared by the engine FILTER path and this
//! materializer), and this module's local canonicalization is deleted.

use crate::Vocab;
use rustc_hash::FxHashSet;
use sparq_core::dict::{self, Dict, Id, TermParts};
use sparq_core::{is_integer_datatype, temporal};

const XSD: &str = "http://www.w3.org/2001/XMLSchema#";
const XSD_INTEGER: &str = "http://www.w3.org/2001/XMLSchema#integer";
const RDF_LANG_STRING: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString";

/// The recognized-datatype set D for a materialization. `xsd:string` and
/// `rdf:langString` are always recognized in RDF 1.1; callers add the others a
/// test or dataset declares (the `sd:recognizedDatatypes` / OWL 2 datatype map).
/// The empty constructor (only the always-recognized pair) is the conservative
/// default — under it the D closure adds NOTHING beyond what is asserted, so it is
/// safe to materialize over an arbitrary graph.
#[derive(Clone, Debug)]
pub struct Recognized {
    iris: FxHashSet<String>,
}

impl Default for Recognized {
    fn default() -> Self {
        let mut iris = FxHashSet::default();
        iris.insert(format!("{XSD}string"));
        iris.insert(RDF_LANG_STRING.to_string());
        Recognized { iris }
    }
}

impl Recognized {
    /// Recognize exactly `iris` (plus the always-recognized `xsd:string` /
    /// `rdf:langString`).
    pub fn new<I: IntoIterator<Item = String>>(iris: I) -> Recognized {
        let mut r = Recognized::default();
        r.iris.extend(iris);
        r
    }

    /// The standard datatype map — every XSD datatype this module has a value
    /// mapping for (the OWL 2 datatype map's numeric/boolean/string/temporal core).
    pub fn standard() -> Recognized {
        let locals = [
            "string",
            "normalizedString",
            "token",
            "boolean",
            "integer",
            "long",
            "int",
            "short",
            "byte",
            "nonNegativeInteger",
            "positiveInteger",
            "nonPositiveInteger",
            "negativeInteger",
            "unsignedLong",
            "unsignedInt",
            "unsignedShort",
            "unsignedByte",
            "decimal",
            "double",
            "float",
            "dateTime",
            "dateTimeStamp",
            "date",
        ];
        Recognized::new(locals.iter().map(|l| format!("{XSD}{l}")))
    }

    /// Is `dt` in the recognized set?
    pub fn contains(&self, dt: &str) -> bool {
        self.iris.contains(dt)
    }
}

/// Expand `triples` in place with the D-entailment closure for the recognized
/// datatype map `d`: for every well-formed literal `"l"^^t` whose datatype `t` is
/// recognized AND has a value mapping here, add the rdfD1 typing triple
/// `("l"^^t rdf:type t)`. Returns the number of NEW triples added. Idempotent: a
/// second call adds nothing (the typing triples are already present).
///
/// The triple is GENERALIZED — its subject is a literal id — which the regular
/// triple store cannot index; callers that feed the result to a SPARQL query must
/// drop literal-subject rows (they can never be a query answer), exactly the
/// existing entailment-harness contract. The materializer's job is the SOUND
/// closure; answer-shaping is the regime restriction's.
pub fn materialize_d(d: &Recognized, dict: &mut Dict, triples: &mut Vec<[Id; 3]>) -> usize {
    let v = Vocab::intern(dict);
    let asserted: FxHashSet<[Id; 3]> = triples.iter().copied().collect();

    // Phase 1 (immutable borrow of `dict`): find every literal id of a recognized,
    // well-formed, value-mapped datatype, recording the literal id + its datatype
    // IRI as an owned String. We do NOT call `intern_iri` here — that needs a
    // mutable borrow while a `TermParts` borrow is live — so the datatype IRI is
    // copied out and interned in phase 2.
    let mut to_type: Vec<(Id, String)> = Vec::new();
    let mut literal_ids: FxHashSet<Id> = FxHashSet::default();
    for &[s, _, o] in triples.iter() {
        for lit_id in [s, o] {
            if !literal_ids.insert(lit_id) {
                continue; // a literal seen once needs typing once
            }
            // Inline ids encode a canonical NON-NEGATIVE small `xsd:integer` (value
            // = id - INLINE_BASE) with NO dictionary record, so `term_parts` cannot
            // resolve them — handle them directly. They are always well-formed and,
            // when xsd:integer is recognized, typed by rdfD1.
            if dict::is_inline(lit_id) {
                if d.contains(XSD_INTEGER) {
                    to_type.push((lit_id, XSD_INTEGER.to_string()));
                }
                continue;
            }
            if let TermParts::Lit { value, datatype, lang } = dict.term_parts(lit_id) {
                // A language-tagged literal's datatype is rdf:langString; rdfD1 adds
                // no useful value-space typing for it, so skip.
                if lang.is_some() {
                    continue;
                }
                if !d.contains(datatype) || !has_value_mapping(datatype) {
                    continue;
                }
                // Ill-formed for its (recognized) datatype: rdfD1 does NOT type it —
                // that case is a D-CLASH (the inconsistency checker's concern), not
                // a typing produced by this monotone closure.
                if d_value_key(value, datatype).is_none() {
                    continue;
                }
                to_type.push((lit_id, datatype.to_string()));
            }
        }
    }

    // Phase 2 (mutable borrow): intern each datatype IRI (idempotent — returns the
    // id the data already uses) and emit the rdfD1 typing triple, deduplicated
    // against what is already asserted. Sort the new rows for deterministic output.
    let mut new_rows: FxHashSet<[Id; 3]> = FxHashSet::default();
    for (lit_id, dt_iri) in to_type {
        let dt_id = dict.intern_iri(&dt_iri);
        let row = [lit_id, v.ty, dt_id];
        if !asserted.contains(&row) {
            new_rows.insert(row);
        }
    }
    let added = new_rows.len();
    let mut new_sorted: Vec<[Id; 3]> = new_rows.into_iter().collect();
    new_sorted.sort_unstable();
    triples.extend(new_sorted);
    added
}

/// Datatypes this module has a value mapping for (so rdfD1 / value comparison can
/// judge well-formedness). The numeric/boolean/string core + the temporal types
/// sparq-core parses. An UNRECOGNIZED-shaped IRI returns false (we cannot judge it,
/// so we never type or clash on it — conservative).
pub fn has_value_mapping(dt: &str) -> bool {
    if dt == RDF_LANG_STRING || dt == format!("{XSD}string") {
        return true;
    }
    if is_integer_datatype(dt) {
        return true;
    }
    matches!(
        dt.strip_prefix(XSD),
        Some(
            "normalizedString"
                | "token"
                | "boolean"
                | "decimal"
                | "double"
                | "float"
                | "dateTime"
                | "dateTimeStamp"
                | "date"
        )
    )
}

/// A CANONICAL, value-space comparison key for a typed literal: two literals denote
/// the same D-value iff their keys are equal. `None` = ill-formed for its datatype
/// (no value), or a datatype with no value mapping here.
///
/// Numbers in the integer/decimal value space share ONE key form
/// ([`DValue::Decimal`], a canonical decimal STRING — never an `f64`), so
/// `"1"^^xsd:integer`, `"01"^^xsd:integer`, `"1.0"^^xsd:decimal` and
/// `"+1.00"^^xsd:decimal` all key-equal. `xsd:float` / `xsd:double` are a DISTINCT
/// value space (IEEE-754) and key by the rounded bit pattern. Temporal types key by
/// the sparq-core `Temporal` instant.
///
/// [OPUS-4.8] migrates to `sparq-substrate::compare` once the substrate move lands
/// (sq-6tykl); the canonical-decimal logic here is the interim, sparq-core-only
/// implementation (the typed Num compare is not yet wired into the substrate).
pub fn d_value_key(lex: &str, dt: &str) -> Option<DValue> {
    if dt == RDF_LANG_STRING {
        return None; // language-tagged: keyed by (lex, lang) elsewhere, not here
    }
    if dt == format!("{XSD}string")
        || dt == format!("{XSD}normalizedString")
        || dt == format!("{XSD}token")
    {
        return Some(DValue::Str(lex.to_string()));
    }
    if dt == format!("{XSD}boolean") {
        return match lex {
            "true" | "1" => Some(DValue::Bool(true)),
            "false" | "0" => Some(DValue::Bool(false)),
            _ => None,
        };
    }
    if is_integer_datatype(dt) {
        let v: i128 = lex.parse().ok()?;
        // Integer-subtype range facets (the value must be IN the datatype's space).
        if !integer_subtype_ok(dt, v) {
            return None;
        }
        return Some(DValue::Decimal(canon_decimal(&v.to_string())?));
    }
    if dt == format!("{XSD}decimal") {
        return Some(DValue::Decimal(canon_decimal(lex)?));
    }
    if dt == format!("{XSD}double") {
        let f = parse_xsd_double(lex)?;
        return Some(DValue::F64(if f.is_nan() {
            f64::NAN.to_bits()
        } else {
            f.to_bits()
        }));
    }
    if dt == format!("{XSD}float") {
        let f = parse_xsd_double(lex)? as f32;
        return Some(DValue::F32(if f.is_nan() {
            f32::NAN.to_bits()
        } else {
            f.to_bits()
        }));
    }
    if dt == format!("{XSD}dateTime")
        || dt == format!("{XSD}dateTimeStamp")
        || dt == format!("{XSD}date")
    {
        let t = temporal::Temporal::of_lit(lex, dt)?;
        // Key by the comparable instant (sparq-core's correct temporal value),
        // tagged with the temporal family and tz-presence so a `date` and a
        // `dateTime` at the same instant (DISJOINT value spaces) never key-equal,
        // and a floating vs zoned same-instant pair stays distinguishable.
        let kind = matches!(t.kind, temporal::TemporalKind::Date);
        return Some(DValue::Temporal(t.instant.to_bits(), t.has_tz, kind));
    }
    None
}

/// True D-value equality for two typed literals under recognized-map semantics.
/// Equal iff both have a value mapping AND their canonical value keys coincide.
pub fn d_value_eq(lex_a: &str, dt_a: &str, lex_b: &str, dt_b: &str) -> bool {
    match (d_value_key(lex_a, dt_a), d_value_key(lex_b, dt_b)) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

/// A canonical D-value, comparable for equality across lexical forms (and across
/// the integer/decimal value spaces, which coincide). NOT an `f64`-collapsed view
/// for the exact value spaces — see the module doc on why the f64 fast path is
/// unsound for semantic equality.
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum DValue {
    /// Canonical decimal string (sign + minimal int digits + minimal frac digits)
    /// — the shared key for `xsd:integer` (+ subtypes) and `xsd:decimal`.
    Decimal(String),
    /// `xsd:double` value as its IEEE-754 bit pattern (NaN canonicalized).
    F64(u64),
    /// `xsd:float` value as its IEEE-754 bit pattern (NaN canonicalized).
    F32(u32),
    Bool(bool),
    Str(String),
    /// A temporal value: (instant f64 bits, tz-present, is-date-family). The
    /// instant is sparq-core's `Temporal::instant` (which orders the value space);
    /// the family flag keeps `xsd:date` and `xsd:dateTime` — DISJOINT value spaces
    /// — from key-aliasing at a shared instant.
    Temporal(u64, bool, bool),
}

/// xsd float/double lexical syntax is a subset of Rust's `f64::parse`; reject the
/// forms Rust accepts but XSD does not (hex, trailing `f`/`d`, the `inf` spelling).
fn parse_xsd_double(lex: &str) -> Option<f64> {
    match lex {
        "INF" | "+INF" => Some(f64::INFINITY),
        "-INF" => Some(f64::NEG_INFINITY),
        "NaN" => Some(f64::NAN),
        _ => {
            if lex.contains(['x', 'X']) || lex.ends_with(['f', 'F', 'd', 'D']) || lex.contains("inf")
            {
                return None;
            }
            lex.parse::<f64>().ok()
        }
    }
}

/// The integer-subtype range facet check: the value must be inside the bounded
/// derived type's value space (e.g. `xsd:byte` is [-128, 127]). `xsd:long`/`int`/
/// `short`/`byte` past i128 already failed to parse; the bounded ones below add the
/// sign facets the XSD spec fixes for the un/signed derived integer types.
fn integer_subtype_ok(dt: &str, v: i128) -> bool {
    let Some(local) = dt.strip_prefix(XSD) else {
        return true;
    };
    match local {
        "nonNegativeInteger" | "unsignedLong" | "unsignedInt" | "unsignedShort"
        | "unsignedByte" => v >= 0,
        "positiveInteger" => v > 0,
        "nonPositiveInteger" => v <= 0,
        "negativeInteger" => v < 0,
        _ => true,
    }
}

/// Canonicalize a decimal lexical form to (sign)(minimal-int).(minimal-frac);
/// `None` when ill-formed. `"-0.0"` canonicalizes to `"0"`; `"1.0"` to `"1"`; so
/// the integer 1 and the decimal 1.0 share the key `"1"`.
fn canon_decimal(lex: &str) -> Option<String> {
    let (sign, rest) = match lex.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, lex.strip_prefix('+').unwrap_or(lex)),
    };
    let (int_part, frac_part) = match rest.split_once('.') {
        Some((i, f)) => (i, f),
        None => (rest, ""),
    };
    if (int_part.is_empty() && frac_part.is_empty())
        || !int_part.bytes().all(|b| b.is_ascii_digit())
        || !frac_part.bytes().all(|b| b.is_ascii_digit())
    {
        return None;
    }
    let int_trim = int_part.trim_start_matches('0');
    let frac_trim = frac_part.trim_end_matches('0');
    let int_c = if int_trim.is_empty() { "0" } else { int_trim };
    let neg = sign && !(int_c == "0" && frac_trim.is_empty());
    Some(format!(
        "{}{}{}{}",
        if neg { "-" } else { "" },
        int_c,
        if frac_trim.is_empty() { "" } else { "." },
        frac_trim
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxrdf::vocab::rdf;
    use oxrdf::{Literal, NamedNode, Term};

    fn lit(value: &str, local: &str) -> Term {
        Term::Literal(Literal::new_typed_literal(
            value,
            NamedNode::new_unchecked(format!("{XSD}{local}")),
        ))
    }

    /// The load-bearing invariant: integer/decimal value-space equality across
    /// lexical forms, using the CORRECT TYPED (canonical-decimal) comparison — NOT
    /// an f64 fast path.
    #[test]
    fn integer_decimal_value_space_coincides() {
        // "1"^^xsd:integer is equivalent to "1.0"^^xsd:decimal — same value.
        assert!(d_value_eq("1", &format!("{XSD}integer"), "1.0", &format!("{XSD}decimal")));
        // leading zeros / signs / trailing fraction zeros are all the same value.
        assert!(d_value_eq("01", &format!("{XSD}integer"), "+1", &format!("{XSD}integer")));
        assert!(d_value_eq("1", &format!("{XSD}integer"), "1.00", &format!("{XSD}decimal")));
        assert!(d_value_eq("-0", &format!("{XSD}integer"), "0", &format!("{XSD}decimal")));
        // distinct values are NOT equal.
        assert!(!d_value_eq("1", &format!("{XSD}integer"), "2", &format!("{XSD}decimal")));
        assert!(!d_value_eq("1.5", &format!("{XSD}decimal"), "1", &format!("{XSD}integer")));
    }

    /// f64 would silently alias these past the 53-bit mantissa; the canonical
    /// decimal comparison keeps them DISTINCT (the unsound-f64 guard).
    #[test]
    fn large_integers_are_not_aliased_by_f64() {
        let big_a = "9007199254740993"; // 2^53 + 1, not representable in f64
        let big_b = "9007199254740992"; // 2^53
        assert!(!d_value_eq(big_a, &format!("{XSD}integer"), big_b, &format!("{XSD}integer")));
        // ...but each equals its own decimal spelling exactly.
        assert!(d_value_eq(
            big_a,
            &format!("{XSD}integer"),
            "9007199254740993.0",
            &format!("{XSD}decimal")
        ));
    }

    /// float and double are a SEPARATE value space from integer/decimal.
    #[test]
    fn float_double_distinct_from_decimal() {
        assert!(!d_value_eq("1", &format!("{XSD}integer"), "1.0", &format!("{XSD}double")));
        assert!(!d_value_eq("1.0", &format!("{XSD}decimal"), "1.0", &format!("{XSD}float")));
        // but two double lexical forms of the same value are equal.
        assert!(d_value_eq("1.0E0", &format!("{XSD}double"), "1.0", &format!("{XSD}double")));
    }

    #[test]
    fn integer_subtype_range_facets() {
        // 200 is out of xsd:byte [-128,127] in the XSD value space → ill-formed (no value).
        assert!(d_value_key("-1", &format!("{XSD}nonNegativeInteger")).is_none());
        assert!(d_value_key("0", &format!("{XSD}positiveInteger")).is_none());
        assert!(d_value_key("1", &format!("{XSD}nonPositiveInteger")).is_none());
        // in range parses.
        assert!(d_value_key("100", &format!("{XSD}nonNegativeInteger")).is_some());
    }

    #[test]
    fn ill_formed_literal_has_no_value() {
        assert!(d_value_key("abc", &format!("{XSD}integer")).is_none());
        assert!(d_value_key("not-a-date", &format!("{XSD}dateTime")).is_none());
    }

    /// rdfD1: a recognized, well-formed literal is typed by its datatype; the
    /// closure is idempotent and adds nothing for an unrecognized datatype.
    #[test]
    fn materialize_d_types_recognized_literals() {
        let mut dict = Dict::new();
        let s = dict.intern(&Term::NamedNode(NamedNode::new_unchecked("http://e/s")));
        let p = dict.intern(&Term::NamedNode(NamedNode::new_unchecked("http://e/p")));
        let one = dict.intern(&lit("1", "integer"));
        let ty = dict.intern(&Term::NamedNode(NamedNode::new_unchecked(rdf::TYPE.as_str())));
        let xsd_integer =
            dict.intern(&Term::NamedNode(NamedNode::new_unchecked(format!("{XSD}integer"))));
        let mut triples = vec![[s, p, one]];

        // Recognized: the literal is typed xsd:integer (rdfD1).
        let d = Recognized::new([format!("{XSD}integer")]);
        let added = materialize_d(&d, &mut dict, &mut triples);
        assert_eq!(added, 1, "rdfD1 types the recognized integer literal");
        assert!(
            triples.contains(&[one, ty, xsd_integer]),
            "(\"1\"^^xsd:integer rdf:type xsd:integer)"
        );

        // Idempotent.
        let added2 = materialize_d(&d, &mut dict, &mut triples);
        assert_eq!(added2, 0, "second materialization adds nothing");
    }

    #[test]
    fn materialize_d_skips_unrecognized_datatype() {
        let mut dict = Dict::new();
        let s = dict.intern(&Term::NamedNode(NamedNode::new_unchecked("http://e/s")));
        let p = dict.intern(&Term::NamedNode(NamedNode::new_unchecked("http://e/p")));
        let one = dict.intern(&lit("1", "integer"));
        let mut triples = vec![[s, p, one]];
        // integer NOT in D (only the always-recognized string/langString).
        let d = Recognized::default();
        let added = materialize_d(&d, &mut dict, &mut triples);
        assert_eq!(added, 0, "an unrecognized datatype yields no typing");
    }

    #[test]
    fn materialize_d_skips_ill_typed_literal() {
        let mut dict = Dict::new();
        let s = dict.intern(&Term::NamedNode(NamedNode::new_unchecked("http://e/s")));
        let p = dict.intern(&Term::NamedNode(NamedNode::new_unchecked("http://e/p")));
        let bad = dict.intern(&lit("abc", "integer"));
        let mut triples = vec![[s, p, bad]];
        let d = Recognized::new([format!("{XSD}integer")]);
        let added = materialize_d(&d, &mut dict, &mut triples);
        assert_eq!(
            added, 0,
            "ill-typed literal is not typed by rdfD1 (it is a clash, not a typing)"
        );
    }
}
