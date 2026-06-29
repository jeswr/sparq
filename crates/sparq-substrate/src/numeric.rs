//! The XSD numeric value tower — [`Num`], [`Dec`], the arithmetic / rounding ops, and the
//! literal → value classifier [`as_numeric`].
//!
//! This is the value-space machinery a D-entailment / RIF-builtin reasoner needs and that
//! the engine uses for FILTER / BIND arithmetic (`research/shared-eval-substrate.md` §2.1).
//! It classifies a numeric literal into the XPath promotion tower — `xsd:integer` →
//! `xsd:decimal` → `xsd:float` → `xsd:double` — keeping the *exact* representation
//! (`i64` integers and a fixed-point [`Dec`]) so a high-precision decimal threshold is not
//! silently flattened to `f64` (the soundness note in research record §5).
//!
//! # Provenance (sq-ev41x, epic sq-qonbz, Phase 2)
//!
//! This module hosts the engine's numeric tower MOVED VERBATIM from
//! `sparq-engine::exec` (the `Num` / `Dec` / `ArithOp` / `RoundMode` types and their methods,
//! `as_numeric` = the engine's old `Num::of_literal`, `num_compare`, `apply_f64`,
//! `fmt_xsd_double`, `parse_xsd_f32` / `parse_xsd_f64`, `split_decimal`, `sig_digits`). It is a
//! BEHAVIOUR-NEUTRAL code-move: the logic — including the EXACT XSD lexical parsers
//! (`parse_xsd_*` accept the XSD `INF` / `-INF` / `NaN` spellings and scientific notation,
//! `Dec::parse_lexical` preserves the written scale) — is bit-identical to the pre-move engine,
//! validated by the W3C SPARQL conformance floor and the ORDER BY / numeric / relop tests
//! staying unchanged. The engine now `use`s these as free functions / shared types.
//!
//! # Zero-overhead
//!
//! Every item is monomorphic over the concrete numeric types and `#[inline]`, so a caller in
//! `sparq-engine` (built with the workspace `lto = "fat"` profile) gets the SAME codegen it had
//! when the tower lived in-crate — no `Box<dyn>`, no vtable on the FILTER / BIND / ORDER BY
//! hot path. The single engine-private piece that did NOT move is `Num::canonical_term`, which
//! returns the engine's private `Value`; the engine keeps it as a thin free helper over the
//! shared [`Num::canonical_lexical`] + [`Num::datatype`].

use std::cmp::Ordering;

/// An EXACT fixed-point decimal: `mant * 10^-scale`. Used to evaluate `+ - *` on
/// integer / `xsd:decimal` operands without f64 rounding (`0.1 + 0.2` is exactly `0.3`),
/// which the f64 arithmetic path gets wrong. Within `i128` range; overflow → `None` →
/// the caller falls back to f64. Division and `xsd:double`/`float` stay f64.
#[derive(Clone, Copy, Debug)]
pub struct Dec {
    /// The scaled integer mantissa.
    pub mant: i128,
    /// The number of fractional decimal digits: the value is `mant * 10^-scale`.
    pub scale: u32,
}

impl Dec {
    /// Parses an integer / decimal lexical (`[+-]?digits(.digits)?`), `None` otherwise.
    #[inline]
    pub fn parse(s: &str) -> Option<Dec> {
        let (neg, int, frac) = split_decimal(s)?;
        let scale = frac.len() as u32;
        let mut mag: i128 = 0;
        for &ch in int.as_bytes().iter().chain(frac.as_bytes()) {
            mag = mag.checked_mul(10)?.checked_add((ch - b'0') as i128)?;
        }
        Some(Dec { mant: if neg { -mag } else { mag }, scale })
    }

    /// Both mantissas scaled to the common (max) scale, or `None` on overflow.
    #[inline]
    pub fn align(self, o: Dec) -> Option<(i128, i128)> {
        let scale = self.scale.max(o.scale);
        let a = self.mant.checked_mul(10i128.checked_pow(scale - self.scale)?)?;
        let b = o.mant.checked_mul(10i128.checked_pow(scale - o.scale)?)?;
        Some((a, b))
    }

    /// Exact addition; `None` on `i128` overflow (caller falls back to f64).
    #[inline]
    pub fn checked_add(self, o: Dec) -> Option<Dec> {
        let (a, b) = self.align(o)?;
        Some(Dec { mant: a.checked_add(b)?, scale: self.scale.max(o.scale) })
    }
    /// Exact subtraction; `None` on `i128` overflow (caller falls back to f64).
    #[inline]
    pub fn checked_sub(self, o: Dec) -> Option<Dec> {
        let (a, b) = self.align(o)?;
        Some(Dec { mant: a.checked_sub(b)?, scale: self.scale.max(o.scale) })
    }
    /// Exact multiplication; `None` on `i128` overflow (caller falls back to f64).
    #[inline]
    pub fn checked_mul(self, o: Dec) -> Option<Dec> {
        Some(Dec { mant: self.mant.checked_mul(o.mant)?, scale: self.scale.checked_add(o.scale)? })
    }
    /// Total order on the exact values, or `None` on a scale-alignment overflow (the
    /// caller then falls back to the `f64` comparison). Named `cmp` (not the `Ord` trait
    /// method) because it is FALLIBLE — two decimals can be incomparable when scale
    /// alignment overflows — so it cannot be `Ord::cmp` (which is total).
    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn cmp(self, o: Dec) -> Option<Ordering> {
        let (a, b) = self.align(o)?;
        Some(a.cmp(&b))
    }

    /// Parses an integer / decimal lexical PRESERVING the written scale ("1.0" keeps
    /// scale 1), unlike [`Dec::parse`] which normalises trailing fraction zeros away.
    /// The scale is what XSD-canonical serialisation of decimal arithmetic preserves
    /// (`1.0 + 2` is `"3.0"`), so typed values must carry it.
    #[inline]
    pub fn parse_lexical(s: &str) -> Option<Dec> {
        let s = s.trim();
        let (neg, s) = match s.strip_prefix('-') {
            Some(r) => (true, r),
            None => (false, s.strip_prefix('+').unwrap_or(s)),
        };
        let (int, frac) = s.split_once('.').unwrap_or((s, ""));
        if (int.is_empty() && frac.is_empty()) || !int.bytes().chain(frac.bytes()).all(|c| c.is_ascii_digit()) {
            return None;
        }
        let mut mag: i128 = 0;
        for &ch in int.as_bytes().iter().chain(frac.as_bytes()) {
            mag = mag.checked_mul(10)?.checked_add((ch - b'0') as i128)?;
        }
        Some(Dec { mant: if neg { -mag } else { mag }, scale: frac.len() as u32 })
    }

    /// EXACT decimal division. The result's scale is the SMALLEST `s >= 1` at which the
    /// quotient terminates (`0 / 2 = "0.0"`, `11.1 / 5 = "2.22"`); a non-terminating
    /// quotient is rounded half-up at scale 18. `None` on overflow (caller falls back
    /// to double); the caller must reject a zero divisor first (type error).
    #[inline]
    pub fn checked_div(self, o: Dec) -> Option<Dec> {
        debug_assert!(o.mant != 0);
        let neg = (self.mant < 0) != (o.mant < 0);
        let n0 = self.mant.unsigned_abs();
        let d = o.mant.unsigned_abs();
        // mant(s) = n0 * 10^(s + o.scale - self.scale) / d
        let num_den = |s: u32| -> Option<(u128, u128)> {
            let e = s as i32 + o.scale as i32 - self.scale as i32;
            if e >= 0 {
                Some((n0.checked_mul(10u128.checked_pow(e as u32)?)?, d))
            } else {
                Some((n0, d.checked_mul(10u128.checked_pow((-e) as u32)?)?))
            }
        };
        const MAX_SCALE: u32 = 18;
        for s in 1..=MAX_SCALE {
            let (num, den) = num_den(s)?;
            if num % den == 0 {
                let mant = i128::try_from(num / den).ok()?;
                return Some(Dec { mant: if neg { -mant } else { mant }, scale: s });
            }
        }
        // Non-terminating: round half-up at the max scale.
        let (num, den) = num_den(MAX_SCALE)?;
        let q = num / den + u128::from(num % den * 2 >= den);
        let mant = i128::try_from(q).ok()?;
        Some(Dec { mant: if neg { -mant } else { mant }, scale: MAX_SCALE })
    }

    /// Rounds to an integer-valued decimal (scale 0), preserving the decimal TYPE
    /// (`CEIL("2.5"^^xsd:decimal)` is `"3"^^xsd:decimal`).
    #[inline]
    pub fn round_to_int(self, mode: RoundMode) -> Dec {
        if self.scale == 0 || self.mant == 0 {
            return Dec { mant: self.mant, scale: 0 };
        }
        // [OPUS-4.8] `10i128.pow(self.scale)` overflows (debug panic / release wrap) for any
        // valid decimal whose scale is >= 39 (10^39 > i128::MAX). When the power exceeds i128
        // the integer part is necessarily 0 (|mant| < i128::MAX < 10^scale), so |value| < 1 and
        // also < 0.5 (2*i128::MAX < 10^39 <= 10^scale), making the rounded result obvious from
        // the sign alone — derive it directly instead of constructing the overflowing power.
        let mant = match 10i128.checked_pow(self.scale) {
            Some(p) => {
                let q = self.mant.div_euclid(p);
                let r = self.mant.rem_euclid(p); // 0..p
                match mode {
                    RoundMode::Floor => q,
                    RoundMode::Ceil => q + i128::from(r > 0),
                    RoundMode::HalfUp => q + i128::from(r * 2 >= p),
                }
            }
            None => match mode {
                // |value| < 1: floor of a tiny positive is 0, of a tiny negative is -1.
                RoundMode::Floor => -i128::from(self.mant < 0),
                // ceil of a tiny positive is 1, of a tiny negative is 0.
                RoundMode::Ceil => i128::from(self.mant > 0),
                // |value| < 0.5 always at this scale, so half-up rounds to 0.
                RoundMode::HalfUp => 0,
            },
        };
        Dec { mant, scale: 0 }
    }

    /// The value as an `f64` (lossy for mantissas beyond `f64`'s exact integer range — the
    /// exact value lives in `mant`/`scale`). Used by the lenient order / arithmetic fallback.
    #[inline]
    pub fn f64(self) -> f64 {
        self.mant as f64 / 10f64.powi(self.scale as i32)
    }

    /// The plain (never exponent) decimal lexical at this value's scale: scale 0 prints
    /// as an integer ("3"); otherwise exactly `scale` fraction digits ("3.0", "0.05").
    #[inline]
    pub fn lexical(self) -> String {
        let mag = self.mant.unsigned_abs().to_string();
        let s = self.scale as usize;
        let mut out = String::with_capacity(mag.len() + s + 2);
        if self.mant < 0 {
            out.push('-');
        }
        if s == 0 {
            out.push_str(&mag);
            return out;
        }
        if mag.len() > s {
            out.push_str(&mag[..mag.len() - s]);
        } else {
            out.push('0');
        }
        out.push('.');
        for _ in mag.len()..s {
            out.push('0');
        }
        if mag.len() > s {
            out.push_str(&mag[mag.len() - s..]);
        } else {
            out.push_str(&mag);
        }
        out
    }
}

/// The rounding mode for [`Dec::round_to_int`] — drives `xsd:decimal` CEIL / FLOOR / ROUND.
pub enum RoundMode {
    /// Towards positive infinity (CEIL).
    Ceil,
    /// Towards negative infinity (FLOOR).
    Floor,
    /// Round half towards positive infinity (XPath `fn:round`).
    HalfUp,
}

/// Splits a decimal lexical into (negative, integer-digits, fraction-digits), normalised
/// (no leading zeros on the integer part, no trailing zeros on the fraction). `None` if
/// the lexical is not digits with at most one `.`.
#[inline]
pub fn split_decimal(s: &str) -> Option<(bool, &str, &str)> {
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

/// Significant decimal digits in a numeric lexical — used to decide whether the f64
/// sargable path is precision-safe (<= 15 digits round-trips through f64 unambiguously).
#[inline]
pub fn sig_digits(s: &str) -> usize {
    let (_, int, frac) = match split_decimal(s) {
        Some(p) => p,
        None => return usize::MAX,
    };
    if int.is_empty() {
        // 0.00123 -> significant digits start at the first non-zero fraction digit.
        frac.trim_start_matches('0').len()
    } else {
        int.len() + frac.len()
    }
}

/// A COMPUTED numeric value carrying its XSD type, implementing the SPARQL/XPath
/// operand-type-promotion tower: integer < decimal < float < double. Arithmetic
/// promotes both operands to the greater type; integer and decimal arithmetic is
/// EXACT (i64 / fixed-point [`Dec`]), falling back to double only on overflow.
/// Serialisation (see [`Num::lexical`]) is the XSD canonical form of the type.
#[derive(Clone, Copy, Debug)]
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

/// The four arithmetic operators under XPath operand promotion.
#[derive(Clone, Copy, PartialEq)]
pub enum ArithOp {
    /// `+`
    Add,
    /// `-`
    Sub,
    /// `*`
    Mul,
    /// `/`
    Div,
}

impl Num {
    /// Promotion rank in the XPath numeric tower.
    #[inline]
    pub fn rank(self) -> u8 {
        match self {
            Num::Int(_) => 0,
            Num::Dec(_) => 1,
            Num::Float(_) => 2,
            Num::Double(_) => 3,
        }
    }

    /// The value as an `f64` (lossy for the exact tiers beyond `f64`'s exact range — the
    /// exact value lives in the `Int` / `Dec` payload). The lenient order / arithmetic
    /// fallback uses it.
    #[inline]
    pub fn f64(self) -> f64 {
        match self {
            Num::Int(i) => i as f64,
            Num::Dec(d) => d.f64(),
            Num::Float(f) => f as f64,
            Num::Double(d) => d,
        }
    }

    /// The value as an exact [`Dec`] for the exact tiers (`Int` / `Dec`); `None` for
    /// `Float` / `Double` (which have no exact fixed-point form).
    #[inline]
    pub fn to_dec(self) -> Option<Dec> {
        match self {
            Num::Int(i) => Some(Dec { mant: i as i128, scale: 0 }),
            Num::Dec(d) => Some(d),
            _ => None,
        }
    }

    /// `op` under XPath operand promotion. `None` is a SPARQL type error (exact-type
    /// division by zero); exact-arithmetic overflow falls back to double, mirroring
    /// the engine's previous f64 behaviour.
    #[inline]
    pub fn binop(self, o: Num, op: ArithOp) -> Option<Num> {
        let rank = self.rank().max(o.rank());
        if rank == 3 {
            return Some(Num::Double(apply_f64(self.f64(), o.f64(), op)));
        }
        if rank == 2 {
            let (a, b) = (self.f64() as f32, o.f64() as f32);
            return Some(Num::Float(apply_f64(a as f64, b as f64, op) as f32));
        }
        // Exact tier: integer / decimal.
        let (a, b) = (self.to_dec()?, o.to_dec()?);
        if op == ArithOp::Div {
            // xsd:integer / xsd:integer is DECIMAL division per SPARQL; exact-type
            // division by zero is a type error (not INF/NaN).
            if b.mant == 0 {
                return None;
            }
            return match a.checked_div(b) {
                Some(d) => Some(Num::Dec(d)),
                None => Some(Num::Double(self.f64() / o.f64())),
            };
        }
        if rank == 0 {
            // integer op integer -> integer (i64; on overflow fall back to double).
            let (x, y) = (match self { Num::Int(i) => i, _ => unreachable!() }, match o { Num::Int(i) => i, _ => unreachable!() });
            let r = match op {
                ArithOp::Add => x.checked_add(y),
                ArithOp::Sub => x.checked_sub(y),
                ArithOp::Mul => x.checked_mul(y),
                ArithOp::Div => unreachable!(),
            };
            return Some(match r {
                Some(i) => Num::Int(i),
                None => Num::Double(apply_f64(x as f64, y as f64, op)),
            });
        }
        let r = match op {
            ArithOp::Add => a.checked_add(b),
            ArithOp::Sub => a.checked_sub(b),
            ArithOp::Mul => a.checked_mul(b),
            ArithOp::Div => unreachable!(),
        };
        Some(match r {
            Some(d) => Num::Dec(d),
            None => Num::Double(apply_f64(self.f64(), o.f64(), op)),
        })
    }

    /// Unary negation, preserving the datatype (overflow falls back to double). Named `neg`
    /// to match the SPARQL/XPath operation (not `std::ops::Neg`, which cannot carry the
    /// datatype-promotion/overflow-fallback semantics this needs).
    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn neg(self) -> Num {
        match self {
            Num::Int(i) => i.checked_neg().map(Num::Int).unwrap_or(Num::Double(-(i as f64))),
            Num::Dec(d) => d.mant.checked_neg().map(|m| Num::Dec(Dec { mant: m, scale: d.scale })).unwrap_or(Num::Double(-d.f64())),
            Num::Float(f) => Num::Float(-f),
            Num::Double(d) => Num::Double(-d),
        }
    }

    /// Absolute value, preserving the datatype (overflow falls back to double).
    #[inline]
    pub fn abs(self) -> Num {
        match self {
            Num::Int(i) => i.checked_abs().map(Num::Int).unwrap_or(Num::Double((i as f64).abs())),
            Num::Dec(d) => d.mant.checked_abs().map(|m| Num::Dec(Dec { mant: m, scale: d.scale })).unwrap_or(Num::Double(d.f64().abs())),
            Num::Float(f) => Num::Float(f.abs()),
            Num::Double(d) => Num::Double(d.abs()),
        }
    }

    /// XPath `fn:ceiling`, preserving the datatype.
    #[inline]
    pub fn ceil(self) -> Num {
        match self {
            Num::Int(_) => self,
            Num::Dec(d) => Num::Dec(d.round_to_int(RoundMode::Ceil)),
            Num::Float(f) => Num::Float(f.ceil()),
            Num::Double(d) => Num::Double(d.ceil()),
        }
    }

    /// XPath `fn:floor`, preserving the datatype.
    #[inline]
    pub fn floor(self) -> Num {
        match self {
            Num::Int(_) => self,
            Num::Dec(d) => Num::Dec(d.round_to_int(RoundMode::Floor)),
            Num::Float(f) => Num::Float(f.floor()),
            Num::Double(d) => Num::Double(d.floor()),
        }
    }

    /// XPath fn:round — round half towards POSITIVE INFINITY (so round(-2.5) = -2),
    /// preserving the argument's datatype.
    #[inline]
    pub fn round(self) -> Num {
        match self {
            Num::Int(_) => self,
            Num::Dec(d) => Num::Dec(d.round_to_int(RoundMode::HalfUp)),
            Num::Float(f) => Num::Float((f + 0.5).floor()),
            Num::Double(d) => Num::Double((d + 0.5).floor()),
        }
    }

    /// The XSD datatype IRI of this value's tier.
    #[inline]
    pub fn datatype(self) -> oxrdf::NamedNodeRef<'static> {
        use oxrdf::vocab::xsd;
        match self {
            Num::Int(_) => xsd::INTEGER,
            Num::Dec(_) => xsd::DECIMAL,
            Num::Float(_) => xsd::FLOAT,
            Num::Double(_) => xsd::DOUBLE,
        }
    }

    /// XSD CANONICAL lexical form of the value: integers as plain digits; decimals
    /// preserving the arithmetic scale ("3.0" for 1.0+2, "3" for CEIL(2.5)); float /
    /// double in mantissa-E-exponent form with a mandatory fractional digit ("3.21E4",
    /// "2.0E-1") and NaN / INF / -INF spelled per XSD.
    #[inline]
    pub fn lexical(self) -> String {
        match self {
            Num::Int(i) => i.to_string(),
            Num::Dec(d) => d.lexical(),
            // f32 must be formatted as f32 (shortest round-trip); via f64 it would
            // grow spurious digits ("2.0000000298023224E-1" for 0.2f32).
            Num::Float(f) => {
                if f.is_nan() {
                    "NaN".to_string()
                } else if f == f32::INFINITY {
                    "INF".to_string()
                } else if f == f32::NEG_INFINITY {
                    "-INF".to_string()
                } else if f.fract() == 0.0 && f.abs() < 1e15 {
                    format!("{}", f as i64)
                } else {
                    let s = format!("{f:E}");
                    match s.split_once('E') {
                        Some((m, e)) if !m.contains('.') => format!("{m}.0E{e}"),
                        _ => s,
                    }
                }
            }
            Num::Double(d) => fmt_xsd_double(d),
        }
    }

    /// STRICT XSD-canonical lexical: float/double ALWAYS in mantissa-E-exponent form
    /// ("3.21E4", "1.0E2"), never plain. The W3C aggregate expected results use this
    /// for MIN/MAX/SUM, while arithmetic results use the plain-integral convention of
    /// [`Num::lexical`] — the suites were generated by different engines.
    #[inline]
    pub fn canonical_lexical(self) -> String {
        match self {
            Num::Int(_) | Num::Dec(_) => self.lexical(),
            Num::Float(f) => {
                if f.is_nan() || f.is_infinite() {
                    self.lexical()
                } else {
                    let s = format!("{f:E}");
                    match s.split_once('E') {
                        Some((m, e)) if !m.contains('.') => format!("{m}.0E{e}"),
                        _ => s,
                    }
                }
            }
            Num::Double(d) => {
                if d.is_nan() || d.is_infinite() {
                    self.lexical()
                } else {
                    let s = format!("{d:E}");
                    match s.split_once('E') {
                        Some((m, e)) if !m.contains('.') => format!("{m}.0E{e}"),
                        _ => s,
                    }
                }
            }
        }
    }

    /// `true` if this value is a floating NaN (used by EBV and equality).
    #[inline]
    pub fn is_nan(self) -> bool {
        match self {
            Num::Float(f) => f.is_nan(),
            Num::Double(d) => d.is_nan(),
            _ => false,
        }
    }

    /// `true` if this value is zero in its tier (used by EBV and exact-division guards).
    #[inline]
    pub fn is_zero(self) -> bool {
        match self {
            Num::Int(i) => i == 0,
            Num::Dec(d) => d.mant == 0,
            Num::Float(f) => f == 0.0,
            Num::Double(d) => d == 0.0,
        }
    }
}

/// Applies `op` to two `f64` operands (the inexact-tier and overflow fallback path).
#[inline]
pub fn apply_f64(a: f64, b: f64, op: ArithOp) -> f64 {
    match op {
        ArithOp::Add => a + b,
        ArithOp::Sub => a - b,
        ArithOp::Mul => a * b,
        ArithOp::Div => a / b,
    }
}

/// Parse an xsd:float/xsd:double lexical: the XSD spellings of the specials, plus the
/// ordinary scientific notation Rust's parser shares with XSD. `None` = ill-formed.
#[inline]
pub fn parse_xsd_f64(v: &str) -> Option<f64> {
    match v {
        "NaN" => Some(f64::NAN),
        "INF" | "+INF" => Some(f64::INFINITY),
        "-INF" => Some(f64::NEG_INFINITY),
        // Rust accepts "inf"/"infinity"/"nan" spellings XSD does not; exclude them.
        _ if v.bytes().all(|c| c.is_ascii_digit() || matches!(c, b'+' | b'-' | b'.' | b'e' | b'E')) => v.parse::<f64>().ok(),
        _ => None,
    }
}

/// Parse an xsd:float lexical (the `f64` parse, narrowed to `f32`). `None` = ill-formed.
#[inline]
pub fn parse_xsd_f32(v: &str) -> Option<f32> {
    parse_xsd_f64(v).map(|d| d as f32)
}

/// Float/double serialisation: an INTEGRAL value prints as a plain integer ("6",
/// "1050" — matching the dominant convention across the W3C expected results, which
/// mix plain and scientific forms); anything else uses the XSD canonical
/// mantissa-E-exponent form with a mandatory fractional digit ("2.0E-1", "1.5E1").
#[inline]
pub fn fmt_xsd_double(v: f64) -> String {
    if v.is_nan() {
        return "NaN".to_string();
    }
    if v == f64::INFINITY {
        return "INF".to_string();
    }
    if v == f64::NEG_INFINITY {
        return "-INF".to_string();
    }
    if v.fract() == 0.0 && v.abs() < 1e15 {
        return format!("{}", v as i64);
    }
    let s = format!("{v:E}"); // shortest round-trip mantissa, e.g. "2E-1"
    match s.split_once('E') {
        Some((m, e)) if !m.contains('.') => format!("{m}.0E{e}"),
        _ => s,
    }
}

/// Numeric value comparison over the promotion tower: the two exact tiers (`Int` / `Dec`)
/// compare EXACTLY via [`Dec::cmp`]; anything inexact (or an exact-scale overflow) compares
/// by `f64` (NaN-aware via `partial_cmp`, so `None` propagates correctly).
#[inline]
pub fn num_compare(a: Num, c: Num) -> Option<Ordering> {
    if let (Some(x), Some(y)) = (a.to_dec(), c.to_dec()) {
        if let Some(o) = x.cmp(y) {
            return Some(o);
        }
    }
    a.f64().partial_cmp(&c.f64())
}

/// The typed numeric value of a literal, or `None` if it is not a well-formed numeric (an
/// ill-formed numeric operand is a SPARQL type error). A language-tagged literal is never
/// numeric.
///
/// This is the engine's old `Num::of_literal` MOVED HERE VERBATIM (sq-ev41x): it uses the
/// SAME datatype predicate the engine uses (`sparq_core::is_integer_datatype`) and the SAME
/// EXACT lexical parsers — `Dec::parse_lexical` (scale-preserving), `parse_xsd_f32` /
/// `parse_xsd_f64` (XSD `INF` / `-INF` / `NaN` / scientific forms) — so the classification is
/// bit-identical to pre-move. (It supersedes the Phase-1 scaffold placeholder that used
/// `Dec::parse` / `str::parse` and so mishandled the XSD special spellings.)
#[inline]
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
        // Integer beyond i64: exact i128 mantissa if it fits (scale 0 = integer
        // lexical), else not representable -> double.
        return match Dec::parse(v) {
            Some(d) if d.scale == 0 => Some(Num::Dec(d)),
            Some(_) => None, // "1.5"^^xsd:integer is ill-formed
            None => None,
        };
    }
    if dt == xsd::DECIMAL {
        return Dec::parse_lexical(v).map(Num::Dec);
    }
    if dt == xsd::FLOAT {
        return parse_xsd_f32(v).map(Num::Float);
    }
    if dt == xsd::DOUBLE {
        return parse_xsd_f64(v).map(Num::Double);
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
        assert!(matches!(n, Num::Int(42)));
        assert_eq!(n.rank(), 0);
        assert_eq!(n.f64(), 42.0);
    }

    #[test]
    fn big_integer_beyond_i64_classifies_to_exact_dec() {
        // 2^63 overflows i64 but fits an i128 mantissa with scale 0 — kept EXACT, not f64.
        let big = "9223372036854775808"; // i64::MAX + 1
        let n = as_numeric(&typed(big, xsd::INTEGER)).expect("a large integer is exact in Dec");
        assert!(matches!(n, Num::Dec(Dec { mant: 9_223_372_036_854_775_808i128, scale: 0 })));
        assert_eq!(n.rank(), 1);
    }

    #[test]
    fn fractional_integer_lexical_is_ill_formed() {
        // "1.5"^^xsd:integer is NOT a well-formed integer -> type error -> None.
        assert!(as_numeric(&typed("1.5", xsd::INTEGER)).is_none());
    }

    #[test]
    fn decimal_keeps_exact_fixed_point_and_written_scale() {
        // 0.1 as an EXACT decimal (mant=1, scale=1) — the f64 path would round it.
        let n = as_numeric(&typed("0.1", xsd::DECIMAL)).expect("0.1 is a well-formed decimal");
        assert!(matches!(n, Num::Dec(Dec { mant: 1, scale: 1 })));
        // parse_lexical PRESERVES the written scale: "1.50" keeps scale 2 (Dec::parse would
        // normalise the trailing zero away). This is the engine semantics, not the scaffold's.
        let n = as_numeric(&typed("1.50", xsd::DECIMAL)).expect("1.50 is a well-formed decimal");
        assert!(matches!(n, Num::Dec(Dec { mant: 150, scale: 2 })));
    }

    #[test]
    fn float_and_double_classify_by_datatype() {
        let f = as_numeric(&typed("1.5", xsd::FLOAT)).expect("float");
        assert!(matches!(f, Num::Float(x) if x == 1.5));
        assert_eq!(f.rank(), 2);
        let d = as_numeric(&typed("1.5", xsd::DOUBLE)).expect("double");
        assert!(matches!(d, Num::Double(x) if x == 1.5));
        assert_eq!(d.rank(), 3);
    }

    #[test]
    fn xsd_special_float_double_spellings() {
        // The REAL engine parsers accept the XSD specials and scientific notation — the
        // bit that the Phase-1 scaffold placeholder (str::parse) got wrong.
        assert!(as_numeric(&typed("INF", xsd::DOUBLE)).expect("INF double").f64().is_infinite());
        assert!(as_numeric(&typed("-INF", xsd::DOUBLE)).expect("-INF double").f64() == f64::NEG_INFINITY);
        assert!(as_numeric(&typed("NaN", xsd::FLOAT)).expect("NaN float").is_nan());
        assert!(matches!(as_numeric(&typed("1.5E2", xsd::DOUBLE)), Some(Num::Double(x)) if x == 150.0));
        // Rust-only spellings XSD forbids are rejected.
        assert!(as_numeric(&typed("infinity", xsd::DOUBLE)).is_none());
        assert!(as_numeric(&typed("nan", xsd::DOUBLE)).is_none());
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
        assert!(matches!(Dec::parse("0.25"), Some(Dec { mant: 25, scale: 2 })));
    }

    #[test]
    fn exact_arithmetic_is_not_f64_rounded() {
        // 0.1 + 0.2 is EXACTLY 0.3 in the decimal tier (the f64 path gives 0.30000000000000004).
        let a = as_numeric(&typed("0.1", xsd::DECIMAL)).unwrap();
        let b = as_numeric(&typed("0.2", xsd::DECIMAL)).unwrap();
        let s = a.binop(b, ArithOp::Add).unwrap();
        assert_eq!(s.lexical(), "0.3");
        // integer / integer is DECIMAL division per SPARQL.
        let p = Num::Int(1).binop(Num::Int(2), ArithOp::Div).unwrap();
        assert_eq!(p.lexical(), "0.5");
        // exact-type division by zero is a type error (None), not INF.
        assert!(Num::Int(1).binop(Num::Int(0), ArithOp::Div).is_none());
    }

    #[test]
    fn promotion_and_canonical_lexical() {
        // int + double -> double (rank 3); double canonical lexical is mantissa-E-exponent.
        let r = Num::Int(1).binop(Num::Double(0.5), ArithOp::Add).unwrap();
        assert!(matches!(r, Num::Double(x) if x == 1.5));
        assert_eq!(r.canonical_lexical(), "1.5E0");
        // an integral double prints plain via lexical(), E-form via canonical_lexical().
        assert_eq!(Num::Double(6.0).lexical(), "6");
        assert_eq!(Num::Double(6.0).canonical_lexical(), "6.0E0");
    }

    #[test]
    fn rounding_preserves_type_and_half_up_to_positive_infinity() {
        let d = as_numeric(&typed("2.5", xsd::DECIMAL)).unwrap();
        assert_eq!(d.ceil().lexical(), "3");
        assert_eq!(d.floor().lexical(), "2");
        assert_eq!(d.round().lexical(), "3");
        let nd = as_numeric(&typed("-2.5", xsd::DECIMAL)).unwrap();
        assert_eq!(nd.round().lexical(), "-2"); // half towards +inf
    }

    #[test]
    fn num_compare_uses_exact_tier() {
        let a = as_numeric(&typed("0.1", xsd::DECIMAL)).unwrap();
        let b = as_numeric(&typed("0.10", xsd::DECIMAL)).unwrap();
        assert_eq!(num_compare(a, b), Some(Ordering::Equal));
        assert_eq!(num_compare(Num::Int(2), Num::Double(2.5)), Some(Ordering::Less));
        // NaN is unordered: partial_cmp yields None.
        assert_eq!(num_compare(Num::Double(f64::NAN), Num::Int(1)), None);
    }
}
