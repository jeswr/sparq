//! Numeric type definitions and type promotion for XPath/SPARQL
//!
//! Implements the xs:numeric union type and type promotion rules as defined in
//! XPath 2.0 Functions and Operators Section 4.1.
//!
//! Type hierarchy:
//! - xs:integer (derived from xs:decimal)
//! - xs:decimal
//! - xs:float (IEEE 754 single precision)
//! - xs:double (IEEE 754 double precision)
//!
//! Type promotion rules:
//! - xs:integer can substitute for xs:decimal (subtype substitution)
//! - xs:decimal can be promoted to xs:float or xs:double
//! - xs:float can be promoted to xs:double
//!
//! For operations with mixed types, both operands are promoted to the same type.

use sparq_ieee754::{f32, f64};

// ============================================================================
// Bit-level IEEE 754 predicates
// ============================================================================
//
// sparq_ieee754 intentionally keeps struct fields and classification helpers
// private; the wrapper-level predicates below operate on the canonical bit
// encoding instead (cheap mask/compare constraints, no decode round-trip).

/// NaN: exponent all-ones and non-zero mantissa (float32 layout)
fn f32_bits_is_nan(bits: u32) -> bool {
    (((bits >> 23) & 0xFF) == 0xFF) & ((bits & 0x7FFFFF) != 0)
}

/// NaN | Infinity | zero (float32 layout); denormals are NOT special
fn f32_bits_is_special(bits: u32) -> bool {
    (((bits >> 23) & 0xFF) == 0xFF) | ((bits & 0x7FFFFFFF) == 0)
}

/// NaN: exponent all-ones and non-zero mantissa (float64 layout)
fn f64_bits_is_nan(bits: u64) -> bool {
    (((bits >> 52) & 0x7FF) == 0x7FF) & ((bits & 0xFFFFFFFFFFFFF) != 0)
}

/// NaN | Infinity | zero (float64 layout); denormals are NOT special
fn f64_bits_is_special(bits: u64) -> bool {
    (((bits >> 52) & 0x7FF) == 0x7FF) | ((bits & 0x7FFFFFFFFFFFFFFF) == 0)
}

// ============================================================================
// Numeric Type Enum
// ============================================================================

/// XPath numeric type discriminator
/// Used to track the original type of a numeric value
pub struct NumericType {
    /// 0 = integer, 1 = decimal, 2 = float, 3 = double
    pub type_id: u8,
}

impl NumericType {
    pub fn integer() -> Self {
        NumericType { type_id: 0 }
    }

    pub fn decimal() -> Self {
        NumericType { type_id: 1 }
    }

    pub fn float() -> Self {
        NumericType { type_id: 2 }
    }

    pub fn double() -> Self {
        NumericType { type_id: 3 }
    }

    pub fn is_integer(self) -> bool {
        self.type_id == 0
    }

    pub fn is_decimal(self) -> bool {
        self.type_id == 1
    }

    pub fn is_float(self) -> bool {
        self.type_id == 2
    }

    pub fn is_double(self) -> bool {
        self.type_id == 3
    }
}

impl Eq for NumericType {
    fn eq(self, other: Self) -> bool {
        self.type_id == other.type_id
    }
}

// ============================================================================
// Float32 Wrapper (xs:float)
// ============================================================================

/// IEEE 754 float32 wrapped as XPath xs:float
pub struct XsdFloat {
    /// The underlying IEEE 754 float32 value (sparq_ieee754 generated type)
    pub value: f32,
}

impl XsdFloat {
    /// Create from IEEE 754 bit representation
    pub fn from_bits(bits: u32) -> Self {
        XsdFloat { value: f32::new(bits) }
    }

    /// Convert to IEEE 754 bit representation
    pub fn to_bits(self) -> u32 {
        self.value.bits()
    }

    /// Check if this value is NaN
    pub fn is_nan(self) -> bool {
        f32_bits_is_nan(self.value.bits())
    }

    /// Create positive zero
    pub fn zero() -> Self {
        XsdFloat::from_bits(0x00000000)
    }

    /// Create negative zero
    pub fn neg_zero() -> Self {
        XsdFloat::from_bits(0x80000000)
    }

    /// Create positive infinity
    pub fn infinity() -> Self {
        XsdFloat::from_bits(0x7F800000)
    }

    /// Create negative infinity
    pub fn neg_infinity() -> Self {
        XsdFloat::from_bits(0xFF800000)
    }

    /// Create NaN
    pub fn nan() -> Self {
        XsdFloat::from_bits(0x7FC00000) // Quiet NaN
    }

    /// Create from a small integer (-128 to 127)
    /// For zero-knowledge compatible conversion (exact for the full i8 range)
    pub fn from_small_int(n: i8) -> Self {
        XsdFloat { value: f32::from(n) }
    }

    /// Create from an i64 integer with exact IEEE 754 single-precision conversion.
    /// Uses the full i64 range; exact for |n| <= 2^24, IEEE round-to-nearest-even beyond.
    /// This is the correct conversion for XPath xs:integer -> xs:float promotion.
    /// Use this instead of `from_small_int(n as i8)` which silently truncates to [-128,127].
    /// [SONNET-4.6]
    pub fn from_i64(n: i64) -> Self {
        XsdFloat { value: f32::from(n) }
    }
}

impl Eq for XsdFloat {
    fn eq(self, other: Self) -> bool {
        self.value.eq(other.value)
    }
}

// ============================================================================
// Float64 Wrapper (xs:double)
// ============================================================================

/// IEEE 754 float64 wrapped as XPath xs:double
pub struct XsdDouble {
    /// The underlying IEEE 754 float64 value (sparq_ieee754 generated type)
    pub value: f64,
}

impl XsdDouble {
    /// Create from IEEE 754 bit representation
    pub fn from_bits(bits: u64) -> Self {
        XsdDouble { value: f64::new(bits) }
    }

    /// Convert to IEEE 754 bit representation
    pub fn to_bits(self) -> u64 {
        self.value.bits()
    }

    /// Check if this value is NaN
    pub fn is_nan(self) -> bool {
        f64_bits_is_nan(self.value.bits())
    }

    /// Create positive zero
    pub fn zero() -> Self {
        XsdDouble::from_bits(0x0000000000000000)
    }

    /// Create negative zero
    pub fn neg_zero() -> Self {
        XsdDouble::from_bits(0x8000000000000000)
    }

    /// Create positive infinity
    pub fn infinity() -> Self {
        XsdDouble::from_bits(0x7FF0000000000000)
    }

    /// Create negative infinity
    pub fn neg_infinity() -> Self {
        XsdDouble::from_bits(0xFFF0000000000000)
    }

    /// Create NaN
    pub fn nan() -> Self {
        XsdDouble::from_bits(0x7FF8000000000000) // Quiet NaN
    }

    /// Create from a small integer (-128 to 127)
    /// For zero-knowledge compatible conversion (exact for the full i8 range)
    pub fn from_small_int(n: i8) -> Self {
        XsdDouble { value: f64::from(n) }
    }

    /// Create from an i64 integer with exact IEEE 754 double-precision conversion.
    /// Uses the full i64 range; exact for |n| <= 2^53, IEEE round-to-nearest-even beyond.
    /// This is the correct conversion for XPath xs:integer -> xs:double promotion.
    /// Use this instead of `from_small_int(n as i8)` which silently truncates to [-128,127].
    /// [SONNET-4.6]
    pub fn from_i64(n: i64) -> Self {
        XsdDouble { value: f64::from(n) }
    }

    /// Promote from float32 to float64
    /// This is exact for all finite float32 values
    pub fn from_float(f: XsdFloat) -> Self {
        let bits32 = f.to_bits();
        let sign: u64 = ((bits32 >> 31) & 1) as u64;
        let exp32: u32 = (bits32 >> 23) & 0xFF;
        let mant32: u32 = bits32 & 0x7FFFFF;

        // Handle zero
        let is_zero = (exp32 == 0) & (mant32 == 0);
        // Handle infinity/NaN
        let is_special = exp32 == 0xFF;
        // Handle denormalized
        let is_denorm = (exp32 == 0) & (mant32 != 0);

        // For each case, compute the bits (only one will be valid)
        // Zero case
        let zero_bits: u64 = sign << 63;

        // Special (infinity/NaN) case
        let special_exp: u64 = 0x7FF;
        let special_mant: u64 = (mant32 as u64) << 29;
        let special_bits: u64 = (sign << 63) | (special_exp << 52) | special_mant;

        // Normal case
        let normal_exp: u64 = ((exp32 as i64 - 127 + 1023) as u64) & 0x7FF;
        let normal_mant: u64 = (mant32 as u64) << 29;
        let normal_bits: u64 = (sign << 63) | (normal_exp << 52) | normal_mant;

        // Denormalized f32: value = sign * 2^(-126) * (0.mant32)
        //                        = sign * mant32 * 2^(-149)
        // Let k = position of the highest set bit in mant32 (0-indexed from LSB).
        // Then: mant32 = (1 << k) + lower_bits  (lower_bits < 2^k)
        //        value = sign * (1 + lower_bits/2^k) * 2^(k - 149)
        // f64 biased exponent = (k - 149) + 1023 = k + 874
        // f64 mantissa (52 bits) = lower_bits << (52 - k)
        // This is an exact widening: every finite f32 value is representable in f64. [SONNET-4.6]
        let k = f32_denorm_leading_bit(mant32);
        let lower_mant: u64 = (mant32 as u64) ^ (1_u64 << k);
        let denorm_f64_exp: u64 = k + 874;
        let denorm_f64_mant: u64 = lower_mant << (52 - k);
        let denorm_bits: u64 = (sign << 63) | (denorm_f64_exp << 52) | denorm_f64_mant;

        // Select the appropriate result
        let result_bits = if is_zero {
            zero_bits
        } else if is_special {
            special_bits
        } else if is_denorm {
            denorm_bits
        } else {
            normal_bits
        };

        XsdDouble::from_bits(result_bits)
    }
}

impl Eq for XsdDouble {
    fn eq(self, other: Self) -> bool {
        self.value.eq(other.value)
    }
}

// ============================================================================
// Denormal f32 helper
// ============================================================================

/// Returns the 0-indexed position of the highest set bit in a non-zero 23-bit
/// f32 mantissa value (bits 22..0). Used exclusively by `XsdDouble::from_float`
/// to normalise denormal f32 values when widening to f64.
/// The caller must guarantee `v != 0`. [SONNET-4.6]
fn f32_denorm_leading_bit(v: u32) -> u64 {
    if (v & 0x400000) != 0 {
        22
    } else if (v & 0x200000) != 0 {
        21
    } else if (v & 0x100000) != 0 {
        20
    } else if (v & 0x080000) != 0 {
        19
    } else if (v & 0x040000) != 0 {
        18
    } else if (v & 0x020000) != 0 {
        17
    } else if (v & 0x010000) != 0 {
        16
    } else if (v & 0x008000) != 0 {
        15
    } else if (v & 0x004000) != 0 {
        14
    } else if (v & 0x002000) != 0 {
        13
    } else if (v & 0x001000) != 0 {
        12
    } else if (v & 0x000800) != 0 {
        11
    } else if (v & 0x000400) != 0 {
        10
    } else if (v & 0x000200) != 0 {
        9
    } else if (v & 0x000100) != 0 {
        8
    } else if (v & 0x000080) != 0 {
        7
    } else if (v & 0x000040) != 0 {
        6
    } else if (v & 0x000020) != 0 {
        5
    } else if (v & 0x000010) != 0 {
        4
    } else if (v & 0x000008) != 0 {
        3
    } else if (v & 0x000004) != 0 {
        2
    } else if (v & 0x000002) != 0 {
        1
    } else {
        0 // bit 0 is set (guaranteed by caller: v != 0)
    }
}

// ============================================================================
// Type Promotion Functions
// ============================================================================

/// Get the common type for two numeric types according to XPath promotion rules
/// Returns the type that both operands should be promoted to
pub fn get_common_type(t1: NumericType, t2: NumericType) -> NumericType {
    // XPath 2.0 F&O Section 6.2 / Appendix B.1 numeric type promotion hierarchy:
    //   xs:integer <: xs:decimal -> xs:float -> xs:double
    // When both operands are the same type, that type is returned.
    // When mixing: promote to the least-general common type.
    //   double + anything          -> double
    //   float  + integer/decimal   -> float  (not double -- Bug B fix [SONNET-4.6])
    //   decimal + integer          -> decimal
    //   integer + integer          -> integer
    if t1.is_double() | t2.is_double() {
        NumericType::double()
    } else if t1.is_float() | t2.is_float() {
        NumericType::float()
    } else if t1.is_decimal() | t2.is_decimal() {
        NumericType::decimal()
    } else {
        NumericType::integer()
    }
}

// ============================================================================
// Float Comparison Functions
// ============================================================================

/// op:numeric-equal for xs:float
/// NaN does not equal itself per IEEE 754 / XPath spec
pub fn numeric_equal_float(a: XsdFloat, b: XsdFloat) -> bool {
    a.value.eq(b.value)
}

/// op:numeric-less-than for xs:float
/// Returns false if either operand is NaN
pub fn numeric_less_than_float(a: XsdFloat, b: XsdFloat) -> bool {
    a.value.lt(b.value)
}

/// op:numeric-greater-than for xs:float
/// Returns false if either operand is NaN
pub fn numeric_greater_than_float(a: XsdFloat, b: XsdFloat) -> bool {
    a.value.gt(b.value)
}

/// op:numeric-less-than-or-equal for xs:float
pub fn numeric_le_float(a: XsdFloat, b: XsdFloat) -> bool {
    a.value.le(b.value)
}

/// op:numeric-greater-than-or-equal for xs:float
pub fn numeric_ge_float(a: XsdFloat, b: XsdFloat) -> bool {
    a.value.ge(b.value)
}

// ============================================================================
// Double Comparison Functions
// ============================================================================

/// op:numeric-equal for xs:double
/// NaN does not equal itself per IEEE 754 / XPath spec
pub fn numeric_equal_double(a: XsdDouble, b: XsdDouble) -> bool {
    a.value.eq(b.value)
}

/// op:numeric-less-than for xs:double
/// Returns false if either operand is NaN
pub fn numeric_less_than_double(a: XsdDouble, b: XsdDouble) -> bool {
    a.value.lt(b.value)
}

/// op:numeric-greater-than for xs:double
/// Returns false if either operand is NaN
pub fn numeric_greater_than_double(a: XsdDouble, b: XsdDouble) -> bool {
    a.value.gt(b.value)
}

/// op:numeric-less-than-or-equal for xs:double
pub fn numeric_le_double(a: XsdDouble, b: XsdDouble) -> bool {
    a.value.le(b.value)
}

/// op:numeric-greater-than-or-equal for xs:double
pub fn numeric_ge_double(a: XsdDouble, b: XsdDouble) -> bool {
    a.value.ge(b.value)
}

// ============================================================================
// Float Arithmetic Functions
// ============================================================================

/// op:numeric-add for xs:float
pub fn numeric_add_float(a: XsdFloat, b: XsdFloat) -> XsdFloat {
    XsdFloat { value: a.value + b.value }
}

/// op:numeric-subtract for xs:float
pub fn numeric_subtract_float(a: XsdFloat, b: XsdFloat) -> XsdFloat {
    XsdFloat { value: a.value - b.value }
}

/// op:numeric-multiply for xs:float
pub fn numeric_multiply_float(a: XsdFloat, b: XsdFloat) -> XsdFloat {
    XsdFloat { value: a.value * b.value }
}

/// op:numeric-divide for xs:float
pub fn numeric_divide_float(a: XsdFloat, b: XsdFloat) -> XsdFloat {
    XsdFloat { value: a.value / b.value }
}

/// fn:abs for xs:float
/// IEEE 754 abs clears the sign bit (NaN payloads preserved)
pub fn abs_float(a: XsdFloat) -> XsdFloat {
    XsdFloat::from_bits(a.to_bits() & 0x7FFFFFFF)
}

// ============================================================================
// Double Arithmetic Functions
// ============================================================================

/// op:numeric-add for xs:double
pub fn numeric_add_double(a: XsdDouble, b: XsdDouble) -> XsdDouble {
    XsdDouble { value: a.value + b.value }
}

/// op:numeric-subtract for xs:double
pub fn numeric_subtract_double(a: XsdDouble, b: XsdDouble) -> XsdDouble {
    XsdDouble { value: a.value - b.value }
}

/// op:numeric-multiply for xs:double
pub fn numeric_multiply_double(a: XsdDouble, b: XsdDouble) -> XsdDouble {
    XsdDouble { value: a.value * b.value }
}

/// op:numeric-divide for xs:double
pub fn numeric_divide_double(a: XsdDouble, b: XsdDouble) -> XsdDouble {
    XsdDouble { value: a.value / b.value }
}

/// fn:abs for xs:double
/// IEEE 754 abs clears the sign bit (NaN payloads preserved)
pub fn abs_double(a: XsdDouble) -> XsdDouble {
    XsdDouble::from_bits(a.to_bits() & 0x7FFFFFFFFFFFFFFF)
}

// ============================================================================
// Integer Division with Decimal Semantics (op:numeric-divide on xs:integer)
// ============================================================================

/// op:numeric-divide for two xs:integer operands.
///
/// XPath F&O 4.2.6: when both operands are xs:integer (a subtype of
/// xs:decimal), `op:numeric-divide` returns their xs:decimal quotient and
/// NEVER truncates: `7 div 2 = 3.5`. Truncating division is the DISTINCT
/// operator `op:numeric-integer-divide` (`idiv`) -- `numeric_divide_int` in
/// `numeric.nr`. SPARQL 1.1 section 17.4 maps `/` to op:numeric-divide, so a
/// truncating `/` silently corrupts query answers (`?x / 2` off by up to 0.5).
///
/// DOCUMENTED APPROXIMATION (not silent truncation): this module has no
/// arbitrary-precision xs:decimal type -- decimals are modelled as IEEE 754
/// doubles throughout (see the module doc). The quotient is therefore
/// computed in double precision: each operand is converted with
/// `XsdDouble::from_i64` (exact for |n| <= 2^53, round-to-nearest-ties-even
/// beyond) and divided with `numeric_divide_double` (correctly rounded per
/// IEEE 754). For operands within +/-2^53 the result is the correctly-rounded
/// double of the true rational quotient; beyond that the operand conversion
/// itself may round first (at most two roundings total).
///
/// Division by zero: xs:decimal division raises err:FOAR0001 in XPath (the
/// Infinity/NaN escape hatch exists only for float/double operands), so this
/// asserts `b != 0` -- fail-closed, matching `numeric_divide_int`. [FABLE-5]
pub fn numeric_divide_int_as_double(a: i64, b: i64) -> XsdDouble {
    assert(b != 0, "Division by zero");
    numeric_divide_double(XsdDouble::from_i64(a), XsdDouble::from_i64(b))
}

// ============================================================================
// Mixed Type Comparison Functions
// ============================================================================

/// Compare integer to double for equality.
/// Promotes integer to double using IEEE 754 i64->f64 conversion: exact for |n| <= 2^53,
/// correctly-rounded (round-to-nearest-ties-even) beyond. [SONNET-4.6]
pub fn compare_int_double_eq(a: i64, b: XsdDouble) -> bool {
    let a_double = XsdDouble::from_i64(a);
    numeric_equal_double(a_double, b)
}

/// Compare integer to double for less-than. [SONNET-4.6]
pub fn compare_int_double_lt(a: i64, b: XsdDouble) -> bool {
    let a_double = XsdDouble::from_i64(a);
    numeric_less_than_double(a_double, b)
}

/// Compare integer to double for greater-than. [SONNET-4.6]
pub fn compare_int_double_gt(a: i64, b: XsdDouble) -> bool {
    let a_double = XsdDouble::from_i64(a);
    numeric_greater_than_double(a_double, b)
}

/// Compare double to integer for equality. [SONNET-4.6]
pub fn compare_double_int_eq(a: XsdDouble, b: i64) -> bool {
    let b_double = XsdDouble::from_i64(b);
    numeric_equal_double(a, b_double)
}

/// Compare double to integer for less-than. [SONNET-4.6]
pub fn compare_double_int_lt(a: XsdDouble, b: i64) -> bool {
    let b_double = XsdDouble::from_i64(b);
    numeric_less_than_double(a, b_double)
}

/// Compare double to integer for greater-than. [SONNET-4.6]
pub fn compare_double_int_gt(a: XsdDouble, b: i64) -> bool {
    let b_double = XsdDouble::from_i64(b);
    numeric_greater_than_double(a, b_double)
}

/// Compare float to double for equality (promotes float to double)
pub fn compare_float_double_eq(a: XsdFloat, b: XsdDouble) -> bool {
    let a_double = XsdDouble::from_float(a);
    numeric_equal_double(a_double, b)
}

/// Compare float to double for less-than
pub fn compare_float_double_lt(a: XsdFloat, b: XsdDouble) -> bool {
    let a_double = XsdDouble::from_float(a);
    numeric_less_than_double(a_double, b)
}

/// Compare float to double for greater-than
pub fn compare_float_double_gt(a: XsdFloat, b: XsdDouble) -> bool {
    let a_double = XsdDouble::from_float(a);
    numeric_greater_than_double(a_double, b)
}

/// Compare integer to double for less-than-or-equal. [SONNET-4.6]
pub fn compare_int_double_le(a: i64, b: XsdDouble) -> bool {
    let a_double = XsdDouble::from_i64(a);
    numeric_le_double(a_double, b)
}

/// Compare integer to double for greater-than-or-equal. [SONNET-4.6]
pub fn compare_int_double_ge(a: i64, b: XsdDouble) -> bool {
    let a_double = XsdDouble::from_i64(a);
    numeric_ge_double(a_double, b)
}

/// Compare double to integer for less-than-or-equal. [SONNET-4.6]
pub fn compare_double_int_le(a: XsdDouble, b: i64) -> bool {
    let b_double = XsdDouble::from_i64(b);
    numeric_le_double(a, b_double)
}

/// Compare double to integer for greater-than-or-equal. [SONNET-4.6]
pub fn compare_double_int_ge(a: XsdDouble, b: i64) -> bool {
    let b_double = XsdDouble::from_i64(b);
    numeric_ge_double(a, b_double)
}

/// Compare float to double for less-than-or-equal
pub fn compare_float_double_le(a: XsdFloat, b: XsdDouble) -> bool {
    let a_double = XsdDouble::from_float(a);
    numeric_le_double(a_double, b)
}

/// Compare float to double for greater-than-or-equal
pub fn compare_float_double_ge(a: XsdFloat, b: XsdDouble) -> bool {
    let a_double = XsdDouble::from_float(a);
    numeric_ge_double(a_double, b)
}

/// Compare double to float for equality
pub fn compare_double_float_eq(a: XsdDouble, b: XsdFloat) -> bool {
    let b_double = XsdDouble::from_float(b);
    numeric_equal_double(a, b_double)
}

/// Compare double to float for less-than
pub fn compare_double_float_lt(a: XsdDouble, b: XsdFloat) -> bool {
    let b_double = XsdDouble::from_float(b);
    numeric_less_than_double(a, b_double)
}

/// Compare double to float for greater-than
pub fn compare_double_float_gt(a: XsdDouble, b: XsdFloat) -> bool {
    let b_double = XsdDouble::from_float(b);
    numeric_greater_than_double(a, b_double)
}

/// Compare double to float for less-than-or-equal
pub fn compare_double_float_le(a: XsdDouble, b: XsdFloat) -> bool {
    let b_double = XsdDouble::from_float(b);
    numeric_le_double(a, b_double)
}

/// Compare double to float for greater-than-or-equal
pub fn compare_double_float_ge(a: XsdDouble, b: XsdFloat) -> bool {
    let b_double = XsdDouble::from_float(b);
    numeric_ge_double(a, b_double)
}

/// Compare integer to float for equality.
/// Promotes integer to float using IEEE 754 i64->f32 conversion: exact for |n| <= 2^24,
/// correctly-rounded (round-to-nearest-ties-even) beyond. [SONNET-4.6]
pub fn compare_int_float_eq(a: i64, b: XsdFloat) -> bool {
    let a_float = XsdFloat::from_i64(a);
    numeric_equal_float(a_float, b)
}

/// Compare integer to float for less-than. [SONNET-4.6]
pub fn compare_int_float_lt(a: i64, b: XsdFloat) -> bool {
    let a_float = XsdFloat::from_i64(a);
    numeric_less_than_float(a_float, b)
}

/// Compare integer to float for greater-than. [SONNET-4.6]
pub fn compare_int_float_gt(a: i64, b: XsdFloat) -> bool {
    let a_float = XsdFloat::from_i64(a);
    numeric_greater_than_float(a_float, b)
}

/// Compare integer to float for less-than-or-equal. [SONNET-4.6]
pub fn compare_int_float_le(a: i64, b: XsdFloat) -> bool {
    let a_float = XsdFloat::from_i64(a);
    numeric_le_float(a_float, b)
}

/// Compare integer to float for greater-than-or-equal. [SONNET-4.6]
pub fn compare_int_float_ge(a: i64, b: XsdFloat) -> bool {
    let a_float = XsdFloat::from_i64(a);
    numeric_ge_float(a_float, b)
}

/// Compare float to integer for equality. [SONNET-4.6]
pub fn compare_float_int_eq(a: XsdFloat, b: i64) -> bool {
    let b_float = XsdFloat::from_i64(b);
    numeric_equal_float(a, b_float)
}

/// Compare float to integer for less-than. [SONNET-4.6]
pub fn compare_float_int_lt(a: XsdFloat, b: i64) -> bool {
    let b_float = XsdFloat::from_i64(b);
    numeric_less_than_float(a, b_float)
}

/// Compare float to integer for greater-than. [SONNET-4.6]
pub fn compare_float_int_gt(a: XsdFloat, b: i64) -> bool {
    let b_float = XsdFloat::from_i64(b);
    numeric_greater_than_float(a, b_float)
}

/// Compare float to integer for less-than-or-equal. [SONNET-4.6]
pub fn compare_float_int_le(a: XsdFloat, b: i64) -> bool {
    let b_float = XsdFloat::from_i64(b);
    numeric_le_float(a, b_float)
}

/// Compare float to integer for greater-than-or-equal. [SONNET-4.6]
pub fn compare_float_int_ge(a: XsdFloat, b: i64) -> bool {
    let b_float = XsdFloat::from_i64(b);
    numeric_ge_float(a, b_float)
}

// ============================================================================
// Type Casting Functions (xs:float, xs:double, xs:integer constructors)
// ============================================================================

/// Cast a small integer (-127 to 127) to xs:float
/// xs:float($arg as xs:integer) -> xs:float
///
/// Note: For Noir compatibility, this only handles small integers.
/// Use XsdFloat::from_small_int for the implementation.
pub fn cast_integer_to_float(n: i8) -> XsdFloat {
    XsdFloat::from_small_int(n)
}

/// Cast a small integer (-127 to 127) to xs:double
/// xs:double($arg as xs:integer) -> xs:double
///
/// Note: For Noir compatibility, this only handles small integers.
/// Use XsdDouble::from_small_int for the implementation.
pub fn cast_integer_to_double(n: i8) -> XsdDouble {
    XsdDouble::from_small_int(n)
}

/// Cast xs:float to xs:integer (truncation toward zero)
/// xs:integer($arg as xs:float) -> xs:integer
///
/// Returns None if the value is NaN, Infinity, or out of range.
/// This simplified version handles common cases for small values.
pub fn cast_float_to_integer(f: XsdFloat) -> Option<i64> {
    let bits = f.to_bits();
    let sign = (bits >> 31) & 1;
    let exp = ((bits >> 23) & 0xFF) as u32;
    let mant = bits & 0x7FFFFF;

    // Check for NaN or infinity (exp == 255)
    let is_special = exp == 255;

    // Check for zero or denormal (exp == 0)
    let is_zero_or_denorm = exp == 0;

    // Unbias the exponent (bias is 127)
    // For valid floating point numbers, exp is 1-254, so unbiased is -126 to 127
    let unbiased_exp: i32 = (exp as i32) - 127;

    // If exponent is negative, value is < 1, truncates to 0
    let truncates_to_zero = unbiased_exp < 0;

    // If exponent is too large (> 62), value exceeds i64 range
    let out_of_range = unbiased_exp > 62;

    // Determine if we need to compute a shift at all
    let needs_computation = !is_special & !is_zero_or_denorm & !truncates_to_zero & !out_of_range;

    // Add implicit leading 1 to mantissa (24 bits total)
    let full_mantissa: u64 = (mant as u64) | 0x800000;

    // Calculate safe shift amounts - clamp to valid range
    let clamped_exp: i32 = if unbiased_exp < 0 {
        0
    } else if unbiased_exp > 62 {
        62
    } else {
        unbiased_exp
    };

    let shift_left_amount: u64 = if clamped_exp >= 23 {
        (clamped_exp - 23) as u64
    } else {
        0
    };
    let shift_right_amount: u64 = if clamped_exp < 23 {
        (23 - clamped_exp) as u64
    } else {
        0
    };

    // Only compute shifted value when needed (using clamped safe values)
    let shifted_value: u64 = if needs_computation {
        if clamped_exp >= 23 {
            full_mantissa << shift_left_amount
        } else {
            full_mantissa >> shift_right_amount
        }
    } else {
        0
    };

    // Apply sign
    let signed_result: i64 = if sign == 1 {
        -(shifted_value as i64)
    } else {
        shifted_value as i64
    };

    // Return based on conditions
    if is_special {
        Option::none()
    } else if is_zero_or_denorm {
        Option::some(0)
    } else if truncates_to_zero {
        Option::some(0)
    } else if out_of_range {
        Option::none()
    } else {
        Option::some(signed_result)
    }
}

/// Cast xs:double to xs:integer (truncation toward zero)
/// xs:integer($arg as xs:double) -> xs:integer
///
/// Returns None if the value is NaN, Infinity, or out of range.
pub fn cast_double_to_integer(d: XsdDouble) -> Option<i64> {
    let bits = d.to_bits();
    let sign = (bits >> 63) & 1;
    let exp = ((bits >> 52) & 0x7FF) as u32;
    let mant = bits & 0xFFFFFFFFFFFFF;

    // Check for NaN or infinity (exp == 2047)
    let is_special = exp == 2047;

    // Check for zero or denormal (exp == 0)
    let is_zero_or_denorm = exp == 0;

    // Unbias the exponent (bias is 1023), but clamp for safety
    // For exp=2047 (special), this gives 1024, but we won't use it
    let unbiased_exp: i32 = (exp as i32) - 1023;

    // If exponent is negative, value is < 1, truncates to 0
    let truncates_to_zero = unbiased_exp < 0;

    // If exponent is too large (> 62), value exceeds i64 range
    let out_of_range = unbiased_exp > 62;

    // Determine if we need to compute a shift at all
    // Only compute shift for normal values in range
    let needs_computation = !is_special & !is_zero_or_denorm & !truncates_to_zero & !out_of_range;

    // Add implicit leading 1 to mantissa (53 bits total)
    let full_mantissa: u64 = mant | 0x10000000000000;

    // Calculate safe shift amounts - clamp to valid range [0, 63]
    // When needs_computation is false, these values don't matter
    let clamped_exp: i32 = if unbiased_exp < 0 {
        0
    } else if unbiased_exp > 62 {
        62
    } else {
        unbiased_exp
    };

    let shift_left_amount: u64 = if clamped_exp >= 52 {
        (clamped_exp - 52) as u64
    } else {
        0
    };
    let shift_right_amount: u64 = if clamped_exp < 52 {
        (52 - clamped_exp) as u64
    } else {
        0
    };

    // Only compute shifted value when needed (using clamped safe values)
    let shifted_value: u64 = if needs_computation {
        if clamped_exp >= 52 {
            full_mantissa << shift_left_amount
        } else {
            full_mantissa >> shift_right_amount
        }
    } else {
        0
    };

    // Apply sign
    let signed_result: i64 = if sign == 1 {
        -(shifted_value as i64)
    } else {
        shifted_value as i64
    };

    // Return based on conditions
    if is_special {
        Option::none()
    } else if is_zero_or_denorm {
        Option::some(0)
    } else if truncates_to_zero {
        Option::some(0)
    } else if out_of_range {
        Option::none()
    } else {
        Option::some(signed_result)
    }
}

/// Cast xs:double to xs:float (narrowing conversion)
/// xs:float($arg as xs:double) -> xs:float
///
/// Converts double to float with potential precision loss and overflow/underflow.
/// Uses XsdDouble::from_float in reverse via bit manipulation.
/// Implements proper IEEE 754 rounding (round to nearest, ties to even).
pub fn cast_double_to_float(d: XsdDouble) -> XsdFloat {
    let bits64 = d.to_bits();
    let sign: u32 = ((bits64 >> 63) & 1) as u32;
    let exp64 = ((bits64 >> 52) & 0x7FF) as u32;
    let mant64: u64 = bits64 & 0xFFFFFFFFFFFFF;

    // Handle zero
    let is_zero = (exp64 == 0) & (mant64 == 0);

    // Handle infinity or NaN (exp == 2047)
    let is_infinity = (exp64 == 2047) & (mant64 == 0);
    let is_nan = (exp64 == 2047) & (mant64 != 0);

    // Handle denormal double (exp == 0, mant != 0)
    let is_denorm = (exp64 == 0) & (mant64 != 0);

    // Convert exponent: unbias from 1023, rebias to 127
    let unbiased_exp: i32 = (exp64 as i32) - 1023;
    let new_exp: i32 = unbiased_exp + 127;

    // Check for overflow (becomes infinity in float)
    let is_overflow = new_exp >= 255;

    // Check for underflow (becomes zero in float)
    let is_underflow = new_exp <= 0;

    // Truncate mantissa from 52 bits to 23 bits with proper rounding
    // IEEE 754 round-to-nearest-even: check bit 28 (round bit) and bits 0-27 (sticky bits)
    let mant32_truncated: u32 = ((mant64 >> 29) & 0x7FFFFF) as u32;
    let round_bit: u64 = (mant64 >> 28) & 1;
    let sticky_bits: u64 = mant64 & 0xFFFFFFF; // Lower 28 bits

    // Round up if: round_bit=1 AND (sticky_bits != 0 OR lsb of result is 1)
    // This implements round-to-nearest-even
    let lsb: u32 = mant32_truncated & 1;
    let round_up: bool = (round_bit == 1) & ((sticky_bits != 0) | (lsb == 1));

    // Apply rounding
    let mant32: u32 = if round_up {
        mant32_truncated + 1
    } else {
        mant32_truncated
    };

    // Handle mantissa overflow from rounding (rare case: 0x7FFFFF + 1 = 0x800000)
    let mant_overflow: bool = mant32 > 0x7FFFFF;
    let final_mant32: u32 = if mant_overflow { 0 } else { mant32 };
    let final_exp: i32 = if mant_overflow { new_exp + 1 } else { new_exp };

    // Re-check overflow after potential exp increment
    let final_overflow: bool = is_overflow | (final_exp >= 255);

    // Build result bits
    let zero_bits: u32 = sign << 31;
    let inf_bits: u32 = (sign << 31) | 0x7F800000;
    let nan_bits: u32 = 0x7FC00000; // Quiet NaN
    let normal_bits: u32 = (sign << 31) | ((final_exp as u32) << 23) | final_mant32;

    let result_bits = if is_zero {
        zero_bits
    } else if is_nan {
        nan_bits
    } else if is_infinity {
        inf_bits
    } else if is_denorm {
        zero_bits // Denormal doubles become zero in float
    } else if final_overflow {
        inf_bits
    } else if is_underflow {
        zero_bits
    } else {
        normal_bits
    };

    XsdFloat::from_bits(result_bits)
}

// ============================================================================
// Rounding Functions
// ============================================================================

// Float32 constants
global FLOAT32_SIGN_MASK: u32 = 0x80000000;
global FLOAT32_EXP_MASK: u32 = 0x7F800000;
global FLOAT32_MANTISSA_MASK: u32 = 0x007FFFFF;
global FLOAT32_EXP_BIAS: u32 = 127;
global FLOAT32_MANTISSA_BITS: u32 = 23;

// Float64 constants
global FLOAT64_SIGN_MASK: u64 = 0x8000000000000000;
global FLOAT64_EXP_MASK: u64 = 0x7FF0000000000000;
global FLOAT64_MANTISSA_MASK: u64 = 0x000FFFFFFFFFFFFF;
global FLOAT64_EXP_BIAS: u64 = 1023;
global FLOAT64_MANTISSA_BITS: u64 = 52;

/// fn:round for xs:float
/// Round to nearest integer, ties toward positive infinity (XPath spec)
/// XPath fn:round: 0.5 -> 1, -0.5 -> -0, 1.5 -> 2, -1.5 -> -1
pub fn round_float(a: XsdFloat) -> XsdFloat {
    let bits = a.to_bits();
    let is_special = f32_bits_is_special(bits);

    let sign = bits & FLOAT32_SIGN_MASK;
    let is_negative = sign != 0;
    let exp_bits = (bits & FLOAT32_EXP_MASK) >> FLOAT32_MANTISSA_BITS;
    let mantissa = bits & FLOAT32_MANTISSA_MASK;
    let exp = exp_bits as i32 - FLOAT32_EXP_BIAS as i32;

    // Case: already integer (exp >= 23)
    let is_integer = exp >= FLOAT32_MANTISSA_BITS as i32;

    // Case: |value| < 0.5 (exp < -1)
    let is_small = exp < -1;

    // Case: 0.5 <= |value| < 1 (exp == -1)
    let is_half_to_one = exp == -1;

    // For exp == -1, the value is sign * 1.mantissa * 2^-1 = sign * (1 + mantissa/2^23) / 2
    // mantissa == 0 means |value| = 0.5
    // mantissa > 0 means |value| > 0.5

    // Result for is_half_to_one case:
    // Positive: always rounds to 1.0 (0.5 -> 1, 0.6 -> 1)
    // Negative with mantissa == 0 (exactly -0.5): rounds to -0 (toward +infinity)
    // Negative with mantissa > 0 (e.g., -0.6): rounds to -1.0
    let half_to_one_result = if is_negative {
        if mantissa == 0 {
            FLOAT32_SIGN_MASK // -0.0
        } else {
            0xBF800000 // -1.0
        }
    } else {
        0x3F800000 // 1.0
    };

    // Normal case: exp in [0, 22]
    let is_normal_case = (exp >= 0) & (exp < FLOAT32_MANTISSA_BITS as i32);

    // Compute for normal case
    let frac_bits = if is_normal_case {
        (FLOAT32_MANTISSA_BITS as i32 - exp) as u32
    } else {
        1
    };
    let frac_mask = (1u32 << frac_bits) - 1;
    let int_mask = !frac_mask;
    let frac = mantissa & frac_mask;
    let half = 1u32 << (frac_bits - 1);

    // For ties toward +infinity:
    // Positive: round up if frac >= half
    // Negative: round up (toward -infinity) only if frac > half (not >= half)
    let round_up = if is_negative {
        frac > half
    } else {
        frac >= half
    };

    let int_mantissa = mantissa & int_mask;

    // Compute rounded result for normal case
    let increment = 1u32 << frac_bits;
    let new_mantissa = int_mantissa + increment;
    let overflow = new_mantissa >= (1u32 << FLOAT32_MANTISSA_BITS);

    let normal_result = if round_up {
        if overflow {
            sign | ((exp_bits + 1) << FLOAT32_MANTISSA_BITS)
        } else {
            sign | (exp_bits << FLOAT32_MANTISSA_BITS) | new_mantissa
        }
    } else {
        sign | (exp_bits << FLOAT32_MANTISSA_BITS) | int_mantissa
    };

    // Select result based on case
    let result_bits = if is_special | is_integer {
        bits
    } else if is_small {
        sign // +0 or -0
    } else if is_half_to_one {
        half_to_one_result
    } else {
        normal_result
    };

    XsdFloat::from_bits(result_bits)
}

/// fn:round for xs:double
/// Round to nearest integer, ties toward positive infinity (XPath spec)
/// XPath fn:round: 0.5 -> 1, -0.5 -> -0, 1.5 -> 2, -1.5 -> -1
pub fn round_double(a: XsdDouble) -> XsdDouble {
    let bits = a.to_bits();
    let is_special = f64_bits_is_special(bits);

    let sign = bits & FLOAT64_SIGN_MASK;
    let is_negative = sign != 0;
    let exp_bits = (bits & FLOAT64_EXP_MASK) >> FLOAT64_MANTISSA_BITS;
    let mantissa = bits & FLOAT64_MANTISSA_MASK;
    let exp = exp_bits as i64 - FLOAT64_EXP_BIAS as i64;

    // Case: already integer (exp >= 52)
    let is_integer = exp >= FLOAT64_MANTISSA_BITS as i64;

    // Case: |value| < 0.5 (exp < -1)
    let is_small = exp < -1;

    // Case: 0.5 <= |value| < 1 (exp == -1)
    let is_half_to_one = exp == -1;

    // Result for is_half_to_one case:
    // Positive: always rounds to 1.0 (0.5 -> 1, 0.6 -> 1)
    // Negative with mantissa == 0 (exactly -0.5): rounds to -0 (toward +infinity)
    // Negative with mantissa > 0 (e.g., -0.6): rounds to -1.0
    let half_to_one_result = if is_negative {
        if mantissa == 0 {
            FLOAT64_SIGN_MASK // -0.0
        } else {
            0xBFF0000000000000 // -1.0
        }
    } else {
        0x3FF0000000000000 // 1.0
    };

    // Normal case: exp in [0, 51]
    let is_normal_case = (exp >= 0) & (exp < FLOAT64_MANTISSA_BITS as i64);

    // Compute for normal case
    let frac_bits = if is_normal_case {
        (FLOAT64_MANTISSA_BITS as i64 - exp) as u64
    } else {
        1
    };
    let frac_mask = (1u64 << frac_bits) - 1;
    let int_mask = !frac_mask;
    let frac = mantissa & frac_mask;
    let half = 1u64 << (frac_bits - 1);

    // For ties toward +infinity:
    // Positive: round up if frac >= half
    // Negative: round up (toward -infinity) only if frac > half (not >= half)
    let round_up = if is_negative {
        frac > half
    } else {
        frac >= half
    };

    let int_mantissa = mantissa & int_mask;

    // Compute rounded result for normal case
    let increment = 1u64 << frac_bits;
    let new_mantissa = int_mantissa + increment;
    let overflow = new_mantissa >= (1u64 << FLOAT64_MANTISSA_BITS);

    let normal_result = if round_up {
        if overflow {
            sign | ((exp_bits + 1) << FLOAT64_MANTISSA_BITS)
        } else {
            sign | (exp_bits << FLOAT64_MANTISSA_BITS) | new_mantissa
        }
    } else {
        sign | (exp_bits << FLOAT64_MANTISSA_BITS) | int_mantissa
    };

    // Select result based on case
    let result_bits = if is_special | is_integer {
        bits
    } else if is_small {
        sign // +0 or -0
    } else if is_half_to_one {
        half_to_one_result
    } else {
        normal_result
    };

    XsdDouble::from_bits(result_bits)
}

/// fn:ceiling for xs:float
/// Round toward positive infinity
/// Bit-twiddling implementation: measured cheaper than the sparq_ieee754
/// f32 ceil kernel at this call shape (185.0 lib-kernel-regressed ~325 ->
/// 130.4 gates/call; see VENDOR.md migration bench table).
// [OPUS-4.8] written while Fable 5 unavailable - re-review when Fable returns
pub fn ceil_float(a: XsdFloat) -> XsdFloat {
    let bits = a.to_bits();
    let is_special = f32_bits_is_special(bits);

    let sign = bits & FLOAT32_SIGN_MASK;
    let is_negative = sign != 0;
    let exp_bits = (bits & FLOAT32_EXP_MASK) >> FLOAT32_MANTISSA_BITS;
    let mantissa = bits & FLOAT32_MANTISSA_MASK;
    let exp = exp_bits as i32 - FLOAT32_EXP_BIAS as i32;

    // Case: already integer (exp >= 23)
    let is_integer = exp >= FLOAT32_MANTISSA_BITS as i32;

    // Case: |value| < 1 (exp < 0)
    let is_less_than_one = exp < 0;

    // Normal case: exp in [0, 22]
    let is_normal_case = (exp >= 0) & (exp < FLOAT32_MANTISSA_BITS as i32);

    // Compute for normal case
    let frac_bits = if is_normal_case {
        (FLOAT32_MANTISSA_BITS as i32 - exp) as u32
    } else {
        1
    };
    let frac_mask = (1u32 << frac_bits) - 1;
    let int_mask = !frac_mask;
    let frac = mantissa & frac_mask;
    let int_mantissa = mantissa & int_mask;
    let round_up = !is_negative & (frac != 0);

    // Compute rounded result for normal case
    let increment = 1u32 << frac_bits;
    let new_mantissa = int_mantissa + increment;
    let overflow = new_mantissa >= (1u32 << FLOAT32_MANTISSA_BITS);

    let normal_result = if round_up {
        if overflow {
            sign | ((exp_bits + 1) << FLOAT32_MANTISSA_BITS)
        } else {
            sign | (exp_bits << FLOAT32_MANTISSA_BITS) | new_mantissa
        }
    } else {
        sign | (exp_bits << FLOAT32_MANTISSA_BITS) | int_mantissa
    };

    // Result for |value| < 1 case
    let less_than_one_result = if is_negative {
        FLOAT32_SIGN_MASK // -0
    } else {
        0x3F800000 // 1.0
    };

    // Select result based on case
    let result_bits = if is_special | is_integer {
        bits
    } else if is_less_than_one {
        less_than_one_result
    } else {
        normal_result
    };

    XsdFloat::from_bits(result_bits)
}

/// fn:ceiling for xs:double
/// Round toward positive infinity (sparq_ieee754 roundToIntegralTowardPositive kernel)
pub fn ceil_double(a: XsdDouble) -> XsdDouble {
    XsdDouble { value: a.value.ceil() }
}

/// fn:floor for xs:float
/// Round toward negative infinity
/// Bit-twiddling implementation: measured cheaper than the sparq_ieee754
/// f32 floor kernel at this call shape (185.0 lib-kernel-regressed ~325 ->
/// 130.4 gates/call; see VENDOR.md migration bench table).
// [OPUS-4.8] written while Fable 5 unavailable - re-review when Fable returns
pub fn floor_float(a: XsdFloat) -> XsdFloat {
    let bits = a.to_bits();
    let is_special = f32_bits_is_special(bits);

    let sign = bits & FLOAT32_SIGN_MASK;
    let is_negative = sign != 0;
    let exp_bits = (bits & FLOAT32_EXP_MASK) >> FLOAT32_MANTISSA_BITS;
    let mantissa = bits & FLOAT32_MANTISSA_MASK;
    let exp = exp_bits as i32 - FLOAT32_EXP_BIAS as i32;

    // Case: already integer (exp >= 23)
    let is_integer = exp >= FLOAT32_MANTISSA_BITS as i32;

    // Case: |value| < 1 (exp < 0)
    let is_less_than_one = exp < 0;

    // Normal case: exp in [0, 22]
    let is_normal_case = (exp >= 0) & (exp < FLOAT32_MANTISSA_BITS as i32);

    // Compute for normal case
    let frac_bits = if is_normal_case {
        (FLOAT32_MANTISSA_BITS as i32 - exp) as u32
    } else {
        1
    };
    let frac_mask = (1u32 << frac_bits) - 1;
    let int_mask = !frac_mask;
    let frac = mantissa & frac_mask;
    let int_mantissa = mantissa & int_mask;
    let round_down = is_negative & (frac != 0);

    // Compute rounded result for normal case
    let increment = 1u32 << frac_bits;
    let new_mantissa = int_mantissa + increment;
    let overflow = new_mantissa >= (1u32 << FLOAT32_MANTISSA_BITS);

    let normal_result = if round_down {
        if overflow {
            sign | ((exp_bits + 1) << FLOAT32_MANTISSA_BITS)
        } else {
            sign | (exp_bits << FLOAT32_MANTISSA_BITS) | new_mantissa
        }
    } else {
        sign | (exp_bits << FLOAT32_MANTISSA_BITS) | int_mantissa
    };

    // Result for |value| < 1 case
    let less_than_one_result = if is_negative {
        0xBF800000 // -1.0
    } else {
        0 // 0.0
    };

    // Select result based on case
    let result_bits = if is_special | is_integer {
        bits
    } else if is_less_than_one {
        less_than_one_result
    } else {
        normal_result
    };

    XsdFloat::from_bits(result_bits)
}

/// fn:floor for xs:double
/// Round toward negative infinity (sparq_ieee754 roundToIntegralTowardNegative kernel)
pub fn floor_double(a: XsdDouble) -> XsdDouble {
    XsdDouble { value: a.value.floor() }
}
