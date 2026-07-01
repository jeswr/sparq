//! The XSD numeric value tower — [`Num`], [`Dec`], and [`as_numeric`].
//!
//! This is the value-space machinery a D-entailment / RIF-builtin reasoner needs and that
//! the SPARQL engine uses for FILTER / BIND arithmetic (`research/shared-eval-substrate.md`
//! §2.1). It classifies a numeric literal into the XPath promotion tower — `xsd:integer` →
//! `xsd:decimal` → `xsd:float` → `xsd:double` — keeping the *exact* representation (`i64`
//! integers and a fixed-point [`Dec`]) so a high-precision decimal threshold is not silently
//! flattened to `f64` (the soundness note in research record §5).
//!
//! # Provenance (sq-ev41x, epic sq-qonbz)
//!
//! This module is the engine's id-level numeric value tower, **moved here verbatim** from
//! `sparq-engine::exec` (the private `Num` / `Dec` / `ArithOp` / `RoundMode` and their
//! helpers `split_decimal` / `parse_xsd_f64` / `parse_xsd_f32` / `apply_f64` /
//! `fmt_xsd_double`). The move is **behaviour-neutral**: the engine now calls
//! `sparq_substrate::numeric::{Num, Dec, as_numeric, …}` and the W3C SPARQL conformance
//! floor + the ORDER BY / FILTER / numeric tests are bit-identical to pre-move.
//!
//! The earlier scaffold (sq-fmprw, #1290) carried a *placeholder* `as_numeric` that used
//! `Dec::parse` / `v.parse::<f32/f64>()`; that PLACEHOLDER is now replaced by the engine's
//! EXACT lexical path — `Dec::parse_lexical` (scale-preserving) for `xsd:decimal` and
//! `parse_xsd_f32` / `parse_xsd_f64` (which handle the XSD `INF` / `-INF` / `NaN` /
//! exponent spellings) for `xsd:float` / `xsd:double` — so classification is bit-identical
//! to what the engine did before the move.
//!
//! # Zero-overhead intent
//!
//! Every item here is monomorphic over the concrete numeric tiers (`i64` / `i128` /
//! `f32` / `f64`); there is NO `Box<dyn>` / `&dyn` / vtable anywhere. The accessors and
//! arithmetic carry `#[inline]` so cross-crate inlining (with the workspace LTO profile)
//! keeps the engine's FILTER / BIND / ORDER BY hot loops identical to the pre-move codegen.

use std::cmp::Ordering;

/// An EXACT fixed-point decimal: `mant * 10^-scale`. Used to evaluate `+ - *` on
/// integer / `xsd:decimal` operands without f64 rounding (`0.1 + 0.2` is exactly `0.3`),
/// which the f64 arithmetic path gets wrong. Within `i128` range; overflow → `None` →
/// the caller falls back to f64. Division and `xsd:double`/`float` stay f64.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Dec {
    /// The scaled integer mantissa.
    pub mant: i128,
    /// The number of fractional decimal digits: the value is `mant * 10^-scale`.
    pub scale: u32,
}

impl Dec {
    /// Parses an integer / decimal lexical (`[+-]?digits(.digits)?`), `None` otherwise
    /// (including `i128` mantissa overflow). Normalises insignificant zeros away (see
    /// [`split_decimal`]); use [`Dec::parse_lexical`] when the written scale must survive.
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
    fn align(self, o: Dec) -> Option<(i128, i128)> {
        let scale = self.scale.max(o.scale);
        let a = self.mant.checked_mul(10i128.checked_pow(scale - self.scale)?)?;
        let b = o.mant.checked_mul(10i128.checked_pow(scale - o.scale)?)?;
        Some((a, b))
    }

    /// EXACT addition, or `None` on overflow (the caller falls back to f64).
    #[inline]
    pub fn checked_add(self, o: Dec) -> Option<Dec> {
        let (a, b) = self.align(o)?;
        Some(Dec { mant: a.checked_add(b)?, scale: self.scale.max(o.scale) })
    }
    /// EXACT subtraction, or `None` on overflow (the caller falls back to f64).
    #[inline]
    pub fn checked_sub(self, o: Dec) -> Option<Dec> {
        let (a, b) = self.align(o)?;
        Some(Dec { mant: a.checked_sub(b)?, scale: self.scale.max(o.scale) })
    }
    /// EXACT multiplication, or `None` on overflow (the caller falls back to f64).
    #[inline]
    pub fn checked_mul(self, o: Dec) -> Option<Dec> {
        Some(Dec { mant: self.mant.checked_mul(o.mant)?, scale: self.scale.checked_add(o.scale)? })
    }
    /// EXACT ordering of two decimals, or `None` on a scale-alignment overflow.
    // Deliberately NOT `Ord::cmp`: this is the FALLIBLE total order (it returns
    // `Option<Ordering>` so a scale-alignment overflow can fall back to f64), so it cannot
    // implement the infallible `Ord`/`PartialOrd` trait method. Kept as `cmp` so the engine
    // call sites are byte-identical to pre-move (this is a verbatim code-move). [OPUS-4.8]
    #[allow(clippy::should_implement_trait)]
    #[inline]
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
    /// exact value lives in `mant`/`scale`). Used by the LENIENT order / arithmetic fallback.
    #[inline]
    pub fn f64(self) -> f64 {
        self.mant as f64 / 10f64.powi(self.scale as i32)
    }

    /// The plain (never exponent) decimal lexical at this value's scale: scale 0 prints
    /// as an integer ("3"); otherwise exactly `scale` fraction digits ("3.0", "0.05").
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

/// Splits a decimal lexical into (negative, integer-digits, fraction-digits), normalised
/// (no leading zeros on the integer part, no trailing zeros on the fraction). `None` if
/// the lexical is not digits with at most one `.`. Shared with the engine's exact
/// decimal-string comparison and significant-digit count.
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

/// The arithmetic operator for [`Num::binop`]: `+ - * /` under XPath operand promotion.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ArithOp {
    /// `+`
    Add,
    /// `-`
    Sub,
    /// `*`
    Mul,
    /// `/` (integer / integer is DECIMAL division per SPARQL).
    Div,
}

/// The rounding direction for [`Dec::round_to_int`] / [`Num::ceil`] / [`Num::floor`] /
/// [`Num::round`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RoundMode {
    /// Round towards positive infinity.
    Ceil,
    /// Round towards negative infinity.
    Floor,
    /// Round half towards positive infinity (XPath `fn:round`).
    HalfUp,
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

impl Num {
    /// Promotion rank in the XPath numeric tower (`Int` < `Dec` < `Float` < `Double`). A
    /// binary numeric op promotes both operands to the higher rank.
    #[inline]
    pub fn rank(self) -> u8 {
        match self {
            Num::Int(_) => 0,
            Num::Dec(_) => 1,
            Num::Float(_) => 2,
            Num::Double(_) => 3,
        }
    }

    /// The value as an `f64` (lossy for the exact tiers' values beyond `f64`'s exact
    /// range — the exact value lives in the `Int` / `Dec` payload).
    #[inline]
    pub fn f64(self) -> f64 {
        match self {
            Num::Int(i) => i as f64,
            Num::Dec(d) => d.f64(),
            Num::Float(f) => f as f64,
            Num::Double(d) => d,
        }
    }

    /// As an exact [`Dec`] for the exact tiers (`Int` / `Dec`); `None` for `Float` /
    /// `Double` (which are not exactly representable as a fixed-point decimal).
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

    /// Unary negation, preserving the datatype (overflow falls back to double).
    // Deliberately NOT `Neg::neg`: SPARQL unary minus promotes to `Double` on `i64`/`Dec`
    // overflow (it never panics/wraps), so the typed result differs from a plain `Neg`. Kept
    // as `neg` so the engine call site is byte-identical to pre-move (verbatim move). [OPUS-4.8]
    #[allow(clippy::should_implement_trait)]
    #[inline]
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
    ///
    /// The float tiers use `round_half_to_pos_inf`, NOT the naive `(x + 0.5).floor()`:
    /// the `x + 0.5` addition double-rounds, so a value just below half such as
    /// `0.49999999999999994` would wrongly round up to `1` instead of `0`. [OPUS-4.8]
    #[inline]
    pub fn round(self) -> Num {
        match self {
            Num::Int(_) => self,
            Num::Dec(d) => Num::Dec(d.round_to_int(RoundMode::HalfUp)),
            // f32 promotes to f64 losslessly and the integral result is exact in f32,
            // so the shared f64 helper yields the correct f32 round. [OPUS-4.8]
            Num::Float(f) => Num::Float(round_half_to_pos_inf(f as f64) as f32),
            Num::Double(d) => Num::Double(round_half_to_pos_inf(d)),
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

    /// `true` if this is a floating tier holding NaN.
    #[inline]
    pub fn is_nan(self) -> bool {
        match self {
            Num::Float(f) => f.is_nan(),
            Num::Double(d) => d.is_nan(),
            _ => false,
        }
    }

    /// `true` if the value is exactly zero (any tier).
    #[inline]
    pub fn is_zero(self) -> bool {
        match self {
            Num::Int(i) => i == 0,
            Num::Dec(d) => d.mant == 0,
            Num::Float(f) => f == 0.0,
            Num::Double(d) => d == 0.0,
        }
    }

    /// The typed numeric value of a literal, or `None` if the literal is not a
    /// well-formed numeric (an ill-formed numeric operand is a SPARQL type error).
    /// This is the EXACT engine lexical path: `parse_xsd_f32`/`parse_xsd_f64` (with the
    /// XSD `INF`/`-INF`/`NaN`/exponent spellings) for float/double, and the
    /// scale-preserving [`Dec::parse_lexical`] for decimal.
    #[inline]
    pub fn of_literal(l: &oxrdf::Literal) -> Option<Num> {
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
}

/// Free-function alias of [`Num::of_literal`]: the typed numeric value of a literal, or
/// `None` if it is not a well-formed numeric (an ill-formed numeric operand is a SPARQL
/// type error; a language-tagged literal is never numeric). This is the substrate's
/// public literal → numeric-tower classifier the engine and a value-space reasoner share.
#[inline]
pub fn as_numeric(l: &oxrdf::Literal) -> Option<Num> {
    Num::of_literal(l)
}

#[inline]
fn apply_f64(a: f64, b: f64, op: ArithOp) -> f64 {
    match op {
        ArithOp::Add => a + b,
        ArithOp::Sub => a - b,
        ArithOp::Mul => a * b,
        ArithOp::Div => a / b,
    }
}

/// Round `x` half towards POSITIVE INFINITY (XPath `fn:round`) WITHOUT the classic
/// double-rounding defect of `(x + 0.5).floor()`.
///
/// `x + 0.5` is a rounded addition, so for a value just below half — e.g.
/// `0.49999999999999994` (the f64 predecessor of `0.5`) — the sum rounds UP to `1.0`
/// and `.floor()` then yields `1`, when the mathematically-nearest integer is `0`.
///
/// The fractional part `x - x.floor()` is instead computed EXACTLY (Sterbenz: the
/// difference between a float and its floor is representable), so the half-comparison
/// is exact. Ties (`x - floor(x) == 0.5`) go to the larger integer, i.e. towards
/// `+INF`, matching `round(-2.5) = -2`. NaN / ±INF / ±0.0 pass through unchanged. [OPUS-4.8]
#[inline]
fn round_half_to_pos_inf(x: f64) -> f64 {
    let fl = x.floor();
    if x - fl >= 0.5 {
        fl + 1.0
    } else {
        fl
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

/// Parse an xsd:float lexical (the [`parse_xsd_f64`] spellings, narrowed to `f32`).
#[inline]
pub fn parse_xsd_f32(v: &str) -> Option<f32> {
    parse_xsd_f64(v).map(|d| d as f32)
}

/// Float/double serialisation: an INTEGRAL value prints as a plain integer ("6",
/// "1050" — matching the dominant convention across the W3C expected results, which
/// mix plain and scientific forms); anything else uses the XSD canonical
/// mantissa-E-exponent form with a mandatory fractional digit ("2.0E-1", "1.5E1").
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
        assert_eq!(n.to_dec(), Some(Dec { mant: 9_223_372_036_854_775_808i128, scale: 0 }));
        assert_eq!(n.rank(), 1);
    }

    #[test]
    fn fractional_integer_lexical_is_ill_formed() {
        // "1.5"^^xsd:integer is NOT a well-formed integer -> type error -> None.
        assert!(as_numeric(&typed("1.5", xsd::INTEGER)).is_none());
    }

    #[test]
    fn decimal_keeps_exact_fixed_point_and_written_scale() {
        // 0.10 as an EXACT decimal — parse_lexical PRESERVES the written scale (2), unlike
        // Dec::parse which would normalise the trailing zero away. This is the engine's
        // real path (the scaffold placeholder used Dec::parse and would have lost the scale).
        let n = as_numeric(&typed("0.10", xsd::DECIMAL)).expect("0.10 is a well-formed decimal");
        assert_eq!(n.to_dec(), Some(Dec { mant: 10, scale: 2 }));
        assert_eq!(n.rank(), 1);
        // And it round-trips to its written lexical.
        assert_eq!(n.lexical(), "0.10");
    }

    #[test]
    fn float_double_xsd_specials_parse() {
        // The XSD special spellings (INF / -INF / NaN) — the scaffold placeholder used
        // a bare v.parse::<f64>() which would REJECT "INF" and accept Rust's "inf".
        assert!(matches!(as_numeric(&typed("INF", xsd::DOUBLE)), Some(Num::Double(d)) if d == f64::INFINITY));
        assert!(matches!(as_numeric(&typed("-INF", xsd::DOUBLE)), Some(Num::Double(d)) if d == f64::NEG_INFINITY));
        assert!(matches!(as_numeric(&typed("NaN", xsd::DOUBLE)), Some(Num::Double(d)) if d.is_nan()));
        assert!(matches!(as_numeric(&typed("INF", xsd::FLOAT)), Some(Num::Float(f)) if f == f32::INFINITY));
        // Rust-only spellings XSD forbids are rejected.
        assert!(as_numeric(&typed("inf", xsd::DOUBLE)).is_none());
        assert!(as_numeric(&typed("infinity", xsd::DOUBLE)).is_none());
        assert!(as_numeric(&typed("nan", xsd::DOUBLE)).is_none());
    }

    #[test]
    fn float_double_exponent_forms_parse() {
        assert!(matches!(as_numeric(&typed("1.5E2", xsd::DOUBLE)), Some(Num::Double(d)) if d == 150.0));
        assert!(matches!(as_numeric(&typed("1.5", xsd::FLOAT)), Some(Num::Float(f)) if f == 1.5));
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

    #[test]
    fn exact_decimal_arithmetic_is_not_f64_rounded() {
        // 0.1 + 0.2 is EXACTLY 0.3 (the f64 path gets 0.30000000000000004).
        let a = as_numeric(&typed("0.1", xsd::DECIMAL)).unwrap();
        let b = as_numeric(&typed("0.2", xsd::DECIMAL)).unwrap();
        let s = a.binop(b, ArithOp::Add).unwrap();
        assert_eq!(s.lexical(), "0.3");
    }

    #[test]
    fn integer_division_is_decimal_and_zero_divisor_is_type_error() {
        let one = Num::Int(1);
        let three = Num::Int(3);
        // 1/3 -> decimal (rounded at scale 18), datatype xsd:decimal.
        let q = one.binop(three, ArithOp::Div).unwrap();
        assert_eq!(q.datatype(), xsd::DECIMAL);
        // x / 0 is a type error, not INF.
        assert!(one.binop(Num::Int(0), ArithOp::Div).is_none());
    }

    #[test]
    fn round_ceil_floor_preserve_datatype() {
        let d = as_numeric(&typed("2.5", xsd::DECIMAL)).unwrap();
        assert_eq!(d.ceil().lexical(), "3");
        assert_eq!(d.floor().lexical(), "2");
        assert_eq!(d.round().lexical(), "3");
        let neg = as_numeric(&typed("-2.5", xsd::DECIMAL)).unwrap();
        // fn:round is half-up towards +INF: round(-2.5) = -2.
        assert_eq!(neg.round().lexical(), "-2");
    }

    #[test]
    fn round_float_tier_no_double_rounding() {
        // Regression for sq-l11x2: the naive `(x + 0.5).floor()` double-rounds because
        // the `x + 0.5` addition is itself rounded, so a value just below one-half wrongly
        // rounds UP. `0.49999999999999994` (the f64 predecessor of 0.5) must round to 0. [OPUS-4.8]
        let just_below_half = 0.49999999999999994_f64;
        assert!(just_below_half < 0.5);
        // Pin the exact defect the OLD formula exhibited so any regression is loud:
        assert_eq!((just_below_half + 0.5).floor(), 1.0, "the naive formula rounds up");
        // The corrected helper and the xsd:double path both round DOWN to 0.
        assert_eq!(round_half_to_pos_inf(just_below_half), 0.0);
        let rd = Num::Double(just_below_half).round();
        assert_eq!(rd.f64(), 0.0);
        assert_eq!(rd.datatype(), xsd::DOUBLE);

        // The same double-rounding defect at the xsd:float tier: predecessor of 0.5_f32.
        let jbh_f32 = f32::from_bits(0x3EFF_FFFF);
        assert!(jbh_f32 < 0.5);
        let rf = Num::Float(jbh_f32).round();
        assert_eq!(rf.f64(), 0.0);
        assert_eq!(rf.datatype(), xsd::FLOAT);
    }

    #[test]
    fn round_float_tier_half_up_towards_pos_inf() {
        // fn:round is round-half-towards-+INF at the float tiers, without double rounding.
        assert_eq!(round_half_to_pos_inf(0.5), 1.0);
        assert_eq!(round_half_to_pos_inf(1.5), 2.0);
        assert_eq!(round_half_to_pos_inf(2.5), 3.0);
        assert_eq!(round_half_to_pos_inf(-0.5), 0.0); // towards +INF, not -1
        assert_eq!(round_half_to_pos_inf(-1.5), -1.0);
        assert_eq!(round_half_to_pos_inf(-2.5), -2.0);
        assert_eq!(round_half_to_pos_inf(2.4), 2.0);
        assert_eq!(round_half_to_pos_inf(2.6), 3.0);
        assert_eq!(round_half_to_pos_inf(-2.6), -3.0);
        // And through the datatype-preserving Num::round entry points.
        assert_eq!(Num::Double(-2.5).round().f64(), -2.0);
        assert_eq!(Num::Double(2.5).round().f64(), 3.0);
        assert_eq!(Num::Float(-2.5).round().f64(), -2.0);
        assert_eq!(Num::Float(2.5).round().f64(), 3.0);
    }

    #[test]
    fn round_float_tier_specials_pass_through() {
        // NaN / ±INF flow through the fractional-part computation unchanged.
        assert!(round_half_to_pos_inf(f64::NAN).is_nan());
        assert!(Num::Double(f64::NAN).round().f64().is_nan());
        assert_eq!(Num::Double(f64::INFINITY).round().f64(), f64::INFINITY);
        assert_eq!(Num::Double(f64::NEG_INFINITY).round().f64(), f64::NEG_INFINITY);
        assert!(Num::Float(f32::NAN).round().f64().is_nan());
    }

    #[test]
    fn ceil_floor_float_tier_are_single_rounded() {
        // ceil/floor already use the single correctly-rounded f64/f32 ops (no double
        // rounding); pin the boundary value alongside round to keep the trio in scope. [OPUS-4.8]
        let jbh = 0.49999999999999994_f64;
        assert_eq!(Num::Double(jbh).ceil().f64(), 1.0);
        assert_eq!(Num::Double(jbh).floor().f64(), 0.0);
        let jbh_f32 = f32::from_bits(0x3EFF_FFFF);
        assert_eq!(Num::Float(jbh_f32).ceil().f64(), 1.0);
        assert_eq!(Num::Float(jbh_f32).floor().f64(), 0.0);
    }

    #[test]
    fn split_decimal_normalises_zeros() {
        assert_eq!(split_decimal("007.50"), Some((false, "7", "5")));
        assert_eq!(split_decimal("-0.0"), Some((true, "", "")));
        assert!(split_decimal("1e5").is_none());
    }

    #[test]
    fn fmt_xsd_double_integral_and_scientific() {
        assert_eq!(fmt_xsd_double(6.0), "6");
        assert_eq!(fmt_xsd_double(0.2), "2.0E-1");
        assert_eq!(fmt_xsd_double(f64::INFINITY), "INF");
        assert_eq!(fmt_xsd_double(f64::NAN), "NaN");
    }

    #[test]
    fn canonical_lexical_is_always_scientific_for_floats() {
        let two = Num::Double(2.0);
        assert_eq!(two.canonical_lexical(), "2.0E0");
        // plain lexical keeps the integral-plain convention.
        assert_eq!(two.lexical(), "2");
    }
}
