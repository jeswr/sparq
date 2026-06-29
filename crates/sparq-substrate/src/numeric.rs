//! The XSD numeric value tower — [`Num`] and [`as_numeric`].
//!
//! This is the value-space machinery a D-entailment / RIF-builtin reasoner needs and that
//! the engine uses for FILTER / BIND arithmetic (`research/shared-eval-substrate.md` §2.1).
//! It classifies a numeric literal into the XPath promotion tower — `xsd:integer` →
//! `xsd:decimal` → `xsd:float` → `xsd:double` — keeping the *exact* representation
//! (`i64` integers and a fixed-point [`Dec`]) so a high-precision decimal threshold is not
//! silently flattened to `f64` (the soundness note in research record §5).
//!
//! # Scaffold status (read before extending)
//!
//! SCAFFOLD: this module establishes the value-tower TYPE shape and the literal →
//! `Num` classification, and the drift-proof accessors ([`Num::rank`], [`Num::f64`]). It
//! does NOT yet host the engine's `compare_values` total order or the `Num` arithmetic
//! (`binop` / `neg` / `abs` / rounding) — those move from `sparq-engine::exec` in epic
//! sq-qonbz Phase 2 (research record §7.2), so the engine and the substrate stay the single
//! source of that logic and it is validated perf-neutral by the W3C conformance floor + the
//! ORDER BY / FILTER micro-benches on that PR. Re-implementing the arithmetic here now would
//! create a second copy that could drift, so this scaffold deliberately stops at the type +
//! classification layer.
//!
//! The representation mirrors the engine's private `exec::Num` / `exec::Dec` exactly (the
//! same four variants, the same `mant * 10^-scale` decimal), so the eventual move is a pure
//! relabelling, not a re-design.

/// An EXACT fixed-point decimal: `mant * 10^-scale`. Carries an `xsd:decimal` (or a large
/// `xsd:integer` that overflows `i64`) without `f64` rounding, so a value-space rule
/// threshold keeps full precision. Within `i128` range; a value outside it is not
/// representable as a [`Dec`] (the engine's arithmetic path, arriving in Phase 2, falls
/// back to `f64` on overflow).
///
/// Mirrors the engine's private `exec::Dec` field-for-field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Dec {
    /// The scaled integer mantissa.
    pub mant: i128,
    /// The number of fractional decimal digits: the value is `mant * 10^-scale`.
    pub scale: u32,
}

impl Dec {
    /// Parses an integer / decimal lexical (`[+-]?digits(.digits)?`); `None` otherwise
    /// (including `i128` mantissa overflow). This is the lexical → exact-value step the
    /// numeric tower shares; it does NOT validate XSD canonical form (the caller decides,
    /// via the datatype, whether a fractional part is allowed — see [`as_numeric`]).
    pub fn parse(s: &str) -> Option<Dec> {
        let (neg, int, frac) = split_decimal(s)?;
        let scale = frac.len() as u32;
        let mut mag: i128 = 0;
        for &ch in int.as_bytes().iter().chain(frac.as_bytes()) {
            mag = mag.checked_mul(10)?.checked_add((ch - b'0') as i128)?;
        }
        Some(Dec { mant: if neg { -mag } else { mag }, scale })
    }

    /// The value as an `f64` (lossy for mantissas beyond `f64`'s exact integer range — the
    /// exact value lives in `mant`/`scale`). Used by the LENIENT order / arithmetic fallback
    /// the engine owns; kept here as the drift-proof accessor the type needs.
    pub fn f64(self) -> f64 {
        (self.mant as f64) / 10f64.powi(self.scale as i32)
    }
}

/// Splits a decimal lexical into `(is_negative, integer_digits, fractional_digits)` with
/// the insignificant zeros stripped (leading zeros off the integer part, trailing zeros off
/// the fraction), or `None` if it is not a well-formed `[+-]?digits(.digits)?`.
///
/// Mirrors the engine's private `exec::split_decimal` exactly, so a value classified here
/// gets the SAME mantissa/scale the engine computes — the scaffold must not diverge from the
/// logic it will later host.
fn split_decimal(s: &str) -> Option<(bool, &str, &str)> {
    let s = s.trim();
    let (neg, s) = match s.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, s.strip_prefix('+').unwrap_or(s)),
    };
    let (int, frac) = s.split_once('.').unwrap_or((s, ""));
    if (int.is_empty() && frac.is_empty()) || !int.bytes().chain(frac.bytes()).all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some((neg, int.trim_start_matches('0'), frac.trim_end_matches('0')))
}

/// A numeric VALUE typed by the XPath / XSD numeric tower. Comparison and arithmetic are
/// done at the highest rank of the two operands (promotion); exact tiers (`Int` / `Dec`)
/// avoid `f64` rounding. Mirrors the engine's private `exec::Num` variant-for-variant.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Num {
    /// `xsd:integer` (and its sub-types) within `i64`.
    Int(i64),
    /// `xsd:decimal`, or an `xsd:integer` beyond `i64`, as an exact fixed-point [`Dec`].
    Dec(Dec),
    /// `xsd:float` (single precision).
    Float(f32),
    /// `xsd:double`.
    Double(f64),
}

impl Num {
    /// Promotion rank in the XPath numeric tower (`Int` < `Dec` < `Float` < `Double`). A
    /// binary numeric op promotes both operands to the higher rank.
    pub fn rank(self) -> u8 {
        match self {
            Num::Int(_) => 0,
            Num::Dec(_) => 1,
            Num::Float(_) => 2,
            Num::Double(_) => 3,
        }
    }

    /// The value as an `f64`. Lossy for the exact tiers' values beyond `f64`'s exact range
    /// (the exact value lives in the `Int` / `Dec` payload). The drift-proof accessor the
    /// type needs; the engine's lenient order / arithmetic fallback uses it.
    pub fn f64(self) -> f64 {
        match self {
            Num::Int(i) => i as f64,
            Num::Dec(d) => d.f64(),
            Num::Float(f) => f as f64,
            Num::Double(d) => d,
        }
    }
}

/// The typed numeric value of a literal, or `None` if it is not a well-formed numeric (an
/// ill-formed numeric operand is a SPARQL type error). A language-tagged literal is never
/// numeric.
///
/// SCAFFOLD note: this is the literal → tower CLASSIFICATION the substrate owns. It uses the
/// SAME datatype predicate the engine uses (`sparq_core::is_integer_datatype`), so the
/// integer-sub-type set is consistent. The engine's `Num::of_literal` will collapse onto this
/// in Phase 2; until then the engine keeps its own copy (no behaviour change anywhere).
pub fn as_numeric(l: &oxrdf::Literal) -> Option<Num> {
    use oxrdf::vocab::xsd;
    if l.language().is_some() {
        return None;
    }
    let dt = l.datatype();
    let v = l.value().trim();
    if sparq_core::is_integer_datatype(dt.as_str()) {
        if let Ok(i) = v.parse::<i64>() {
            return Some(Num::Int(i));
        }
        // Integer beyond i64: exact i128 mantissa if it fits AND it is an integer lexical
        // (scale 0). "1.5"^^xsd:integer is ill-formed; a too-large magnitude is unrepresentable.
        return match Dec::parse(v) {
            Some(d) if d.scale == 0 => Some(Num::Dec(d)),
            _ => None,
        };
    }
    if dt == xsd::DECIMAL {
        return Dec::parse(v).map(Num::Dec);
    }
    if dt == xsd::FLOAT {
        return v.parse::<f32>().ok().map(Num::Float);
    }
    if dt == xsd::DOUBLE {
        return v.parse::<f64>().ok().map(Num::Double);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxrdf::vocab::xsd;
    use oxrdf::Literal;

    fn typed(value: &str, dt: oxrdf::NamedNodeRef<'_>) -> Literal {
        Literal::new_typed_literal(value, dt)
    }

    #[test]
    fn integer_literal_classifies_to_int() {
        let n = as_numeric(&typed("42", xsd::INTEGER)).expect("42 is a well-formed integer");
        assert_eq!(n, Num::Int(42));
        assert_eq!(n.rank(), 0);
        assert_eq!(n.f64(), 42.0);
    }

    #[test]
    fn big_integer_beyond_i64_classifies_to_exact_dec() {
        // 2^63 overflows i64 but fits an i128 mantissa with scale 0 — kept EXACT, not f64.
        let big = "9223372036854775808"; // i64::MAX + 1
        let n = as_numeric(&typed(big, xsd::INTEGER)).expect("a large integer is exact in Dec");
        assert_eq!(n, Num::Dec(Dec { mant: 9_223_372_036_854_775_808i128, scale: 0 }));
        assert_eq!(n.rank(), 1);
    }

    #[test]
    fn fractional_integer_lexical_is_ill_formed() {
        // "1.5"^^xsd:integer is NOT a well-formed integer -> type error -> None.
        assert!(as_numeric(&typed("1.5", xsd::INTEGER)).is_none());
    }

    #[test]
    fn decimal_keeps_exact_fixed_point() {
        // 0.1 as an EXACT decimal (mant=1, scale=1) — the f64 path would round it.
        let n = as_numeric(&typed("0.1", xsd::DECIMAL)).expect("0.1 is a well-formed decimal");
        assert_eq!(n, Num::Dec(Dec { mant: 1, scale: 1 }));
        assert_eq!(n.rank(), 1);
    }

    #[test]
    fn float_and_double_classify_by_datatype() {
        let f = as_numeric(&typed("1.5", xsd::FLOAT)).expect("float");
        assert_eq!(f, Num::Float(1.5));
        assert_eq!(f.rank(), 2);
        let d = as_numeric(&typed("1.5", xsd::DOUBLE)).expect("double");
        assert_eq!(d, Num::Double(1.5));
        assert_eq!(d.rank(), 3);
    }

    #[test]
    fn language_tagged_and_non_numeric_are_not_numeric() {
        assert!(as_numeric(&Literal::new_language_tagged_literal("12", "en").unwrap()).is_none());
        assert!(as_numeric(&typed("hello", xsd::STRING)).is_none());
        assert!(as_numeric(&typed("not-a-number", xsd::DECIMAL)).is_none());
    }

    #[test]
    fn dec_parse_rejects_malformed() {
        assert!(Dec::parse("").is_none());
        assert!(Dec::parse("+").is_none());
        assert!(Dec::parse(".").is_none());
        assert!(Dec::parse("1.2.3").is_none());
        assert!(Dec::parse("1e5").is_none()); // exponential is not a decimal lexical
        assert_eq!(Dec::parse("0.25"), Some(Dec { mant: 25, scale: 2 }));
    }
}
