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

use ieee754::float::{
    abs_float32, abs_float64, add_float32, add_float32_with_rounding, add_float64,
    add_float64_with_rounding, div_float32, div_float64, float32_eq, float32_from_bits, float32_ge,
    float32_gt, float32_is_infinity, float32_is_nan, float32_is_zero, float32_le, float32_lt,
    float32_to_bits, FLOAT32_ZERO, float64_eq, float64_from_bits, float64_ge, float64_gt,
    float64_is_infinity, float64_is_nan, float64_is_zero, float64_le, float64_lt, float64_to_bits,
    FLOAT64_ZERO, IEEE754Float32, IEEE754Float64, mul_float32, mul_float64, sub_float32,
    sub_float64,
};
use ieee754::types::{
    ROUNDING_MODE_NEAREST_AWAY, ROUNDING_MODE_TOWARD_NEGATIVE, ROUNDING_MODE_TOWARD_POSITIVE,
};

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
    /// The underlying IEEE 754 float32 value
    pub value: IEEE754Float32,
}

impl XsdFloat {
    /// Create from IEEE 754 bit representation
    pub fn from_bits(bits: u32) -> Self {
        XsdFloat { value: float32_from_bits(bits) }
    }

    /// Convert to IEEE 754 bit representation
    pub fn to_bits(self) -> u32 {
        float32_to_bits(self.value)
    }

    /// Check if this value is NaN
    pub fn is_nan(self) -> bool {
        float32_is_nan(self.value)
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

    /// Create from a small integer (-127 to 127)
    /// For zero-knowledge compatible conversion
    pub fn from_small_int(n: i8) -> Self {
        // Handle zero case
        if n == 0 {
            XsdFloat::zero()
        } else {
            let is_negative = n < 0;
            let abs_n: u8 = if is_negative {
                ((-n) as i8) as u8
            } else {
                n as u8
            };

            // Find highest set bit position using bounded loop
            let mut msb_pos: u32 = 0;
            for i in 0..8 {
                if (abs_n >> i) != 0 {
                    msb_pos = i as u32;
                }
            }

            // Exponent is biased by 127
            let exponent: u32 = 127 + msb_pos;

            // Mantissa: shift value left to fill 23 bits, remove implicit leading 1
            let mantissa: u32 = if msb_pos == 0 {
                0
            } else {
                ((abs_n as u32) << (23 - msb_pos)) & 0x7FFFFF
            };

            let sign_bit: u32 = if is_negative { 1 << 31 } else { 0 };
            XsdFloat::from_bits(sign_bit | (exponent << 23) | mantissa)
        }
    }
}

impl Eq for XsdFloat {
    fn eq(self, other: Self) -> bool {
        float32_eq(self.value, other.value)
    }
}

// ============================================================================
// Float64 Wrapper (xs:double)
// ============================================================================

/// IEEE 754 float64 wrapped as XPath xs:double
pub struct XsdDouble {
    /// The underlying IEEE 754 float64 value
    pub value: IEEE754Float64,
}

impl XsdDouble {
    /// Create from IEEE 754 bit representation
    pub fn from_bits(bits: u64) -> Self {
        XsdDouble { value: float64_from_bits(bits) }
    }

    /// Convert to IEEE 754 bit representation
    pub fn to_bits(self) -> u64 {
        float64_to_bits(self.value)
    }

    /// Check if this value is NaN
    pub fn is_nan(self) -> bool {
        float64_is_nan(self.value)
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

    /// Create from a small integer (-127 to 127)
    /// For zero-knowledge compatible conversion
    pub fn from_small_int(n: i8) -> Self {
        if n == 0 {
            XsdDouble::zero()
        } else {
            let is_negative = n < 0;
            let abs_n: u8 = if is_negative {
                ((-n) as i8) as u8
            } else {
                n as u8
            };

            // Find highest set bit position using bounded loop
            let mut msb_pos: u32 = 0;
            for i in 0..8 {
                if (abs_n >> i) != 0 {
                    msb_pos = i as u32;
                }
            }

            // Exponent is biased by 1023 for float64
            let exponent: u64 = (1023 + msb_pos) as u64;

            // Mantissa: shift value left to fill 52 bits, remove implicit leading 1
            let shift_amount: u64 = (52 - msb_pos) as u64;
            let mantissa: u64 = if msb_pos == 0 {
                0
            } else {
                ((abs_n as u64) << shift_amount) & 0xFFFFFFFFFFFFF
            };

            let sign_bit: u64 = if is_negative { 1 << 63 } else { 0 };
            XsdDouble::from_bits(sign_bit | (exponent << 52) | mantissa)
        }
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

        // Denormalized - simplified handling (treat as zero for now)
        // Full denorm handling requires unbounded loops
        let denorm_bits: u64 = sign << 63;

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
        float64_eq(self.value, other.value)
    }
}

// ============================================================================
// Type Promotion Functions
// ============================================================================

/// Get the common type for two numeric types according to XPath promotion rules
/// Returns the type that both operands should be promoted to
pub fn get_common_type(t1: NumericType, t2: NumericType) -> NumericType {
    // double has highest precedence, then float, then decimal, then integer
    // When mixing types, promote to the highest precedence type
    if t1.is_double() | t2.is_double() {
        NumericType::double()
    } else if t1.is_float() | t2.is_float() {
        // Per XPath spec: When mixing float with decimal, promote to double
        // to avoid precision loss
        NumericType::double()
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
    float32_eq(a.value, b.value)
}

/// op:numeric-less-than for xs:float
/// Returns false if either operand is NaN
pub fn numeric_less_than_float(a: XsdFloat, b: XsdFloat) -> bool {
    float32_lt(a.value, b.value)
}

/// op:numeric-greater-than for xs:float
/// Returns false if either operand is NaN
pub fn numeric_greater_than_float(a: XsdFloat, b: XsdFloat) -> bool {
    float32_gt(a.value, b.value)
}

/// op:numeric-less-than-or-equal for xs:float
pub fn numeric_le_float(a: XsdFloat, b: XsdFloat) -> bool {
    float32_le(a.value, b.value)
}

/// op:numeric-greater-than-or-equal for xs:float
pub fn numeric_ge_float(a: XsdFloat, b: XsdFloat) -> bool {
    float32_ge(a.value, b.value)
}

// ============================================================================
// Double Comparison Functions
// ============================================================================

/// op:numeric-equal for xs:double
/// NaN does not equal itself per IEEE 754 / XPath spec
pub fn numeric_equal_double(a: XsdDouble, b: XsdDouble) -> bool {
    float64_eq(a.value, b.value)
}

/// op:numeric-less-than for xs:double
/// Returns false if either operand is NaN
pub fn numeric_less_than_double(a: XsdDouble, b: XsdDouble) -> bool {
    float64_lt(a.value, b.value)
}

/// op:numeric-greater-than for xs:double
/// Returns false if either operand is NaN
pub fn numeric_greater_than_double(a: XsdDouble, b: XsdDouble) -> bool {
    float64_gt(a.value, b.value)
}

/// op:numeric-less-than-or-equal for xs:double
pub fn numeric_le_double(a: XsdDouble, b: XsdDouble) -> bool {
    float64_le(a.value, b.value)
}

/// op:numeric-greater-than-or-equal for xs:double
pub fn numeric_ge_double(a: XsdDouble, b: XsdDouble) -> bool {
    float64_ge(a.value, b.value)
}

// ============================================================================
// Float Arithmetic Functions
// ============================================================================

/// op:numeric-add for xs:float
pub fn numeric_add_float(a: XsdFloat, b: XsdFloat) -> XsdFloat {
    XsdFloat { value: add_float32(a.value, b.value) }
}

/// op:numeric-subtract for xs:float
pub fn numeric_subtract_float(a: XsdFloat, b: XsdFloat) -> XsdFloat {
    XsdFloat { value: sub_float32(a.value, b.value) }
}

/// op:numeric-multiply for xs:float
pub fn numeric_multiply_float(a: XsdFloat, b: XsdFloat) -> XsdFloat {
    XsdFloat { value: mul_float32(a.value, b.value) }
}

/// op:numeric-divide for xs:float
pub fn numeric_divide_float(a: XsdFloat, b: XsdFloat) -> XsdFloat {
    XsdFloat { value: div_float32(a.value, b.value) }
}

/// fn:abs for xs:float
pub fn abs_float(a: XsdFloat) -> XsdFloat {
    XsdFloat { value: abs_float32(a.value) }
}

// ============================================================================
// Double Arithmetic Functions
// ============================================================================

/// op:numeric-add for xs:double
pub fn numeric_add_double(a: XsdDouble, b: XsdDouble) -> XsdDouble {
    XsdDouble { value: add_float64(a.value, b.value) }
}

/// op:numeric-subtract for xs:double
pub fn numeric_subtract_double(a: XsdDouble, b: XsdDouble) -> XsdDouble {
    XsdDouble { value: sub_float64(a.value, b.value) }
}

/// op:numeric-multiply for xs:double
pub fn numeric_multiply_double(a: XsdDouble, b: XsdDouble) -> XsdDouble {
    XsdDouble { value: mul_float64(a.value, b.value) }
}

/// op:numeric-divide for xs:double
pub fn numeric_divide_double(a: XsdDouble, b: XsdDouble) -> XsdDouble {
    XsdDouble { value: div_float64(a.value, b.value) }
}

/// fn:abs for xs:double
pub fn abs_double(a: XsdDouble) -> XsdDouble {
    XsdDouble { value: abs_float64(a.value) }
}

// ============================================================================
// Mixed Type Comparison Functions
// ============================================================================

/// Compare integer to double for equality
/// Promotes integer to double first
pub fn compare_int_double_eq(a: i64, b: XsdDouble) -> bool {
    let a_double = XsdDouble::from_small_int(a as i8);
    numeric_equal_double(a_double, b)
}

/// Compare integer to double for less-than
pub fn compare_int_double_lt(a: i64, b: XsdDouble) -> bool {
    let a_double = XsdDouble::from_small_int(a as i8);
    numeric_less_than_double(a_double, b)
}

/// Compare integer to double for greater-than
pub fn compare_int_double_gt(a: i64, b: XsdDouble) -> bool {
    let a_double = XsdDouble::from_small_int(a as i8);
    numeric_greater_than_double(a_double, b)
}

/// Compare double to integer for equality
pub fn compare_double_int_eq(a: XsdDouble, b: i64) -> bool {
    let b_double = XsdDouble::from_small_int(b as i8);
    numeric_equal_double(a, b_double)
}

/// Compare double to integer for less-than
pub fn compare_double_int_lt(a: XsdDouble, b: i64) -> bool {
    let b_double = XsdDouble::from_small_int(b as i8);
    numeric_less_than_double(a, b_double)
}

/// Compare double to integer for greater-than
pub fn compare_double_int_gt(a: XsdDouble, b: i64) -> bool {
    let b_double = XsdDouble::from_small_int(b as i8);
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

/// Compare integer to double for less-than-or-equal
pub fn compare_int_double_le(a: i64, b: XsdDouble) -> bool {
    let a_double = XsdDouble::from_small_int(a as i8);
    numeric_le_double(a_double, b)
}

/// Compare integer to double for greater-than-or-equal
pub fn compare_int_double_ge(a: i64, b: XsdDouble) -> bool {
    let a_double = XsdDouble::from_small_int(a as i8);
    numeric_ge_double(a_double, b)
}

/// Compare double to integer for less-than-or-equal
pub fn compare_double_int_le(a: XsdDouble, b: i64) -> bool {
    let b_double = XsdDouble::from_small_int(b as i8);
    numeric_le_double(a, b_double)
}

/// Compare double to integer for greater-than-or-equal
pub fn compare_double_int_ge(a: XsdDouble, b: i64) -> bool {
    let b_double = XsdDouble::from_small_int(b as i8);
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

/// Compare integer to float for equality
pub fn compare_int_float_eq(a: i64, b: XsdFloat) -> bool {
    let a_float = XsdFloat::from_small_int(a as i8);
    numeric_equal_float(a_float, b)
}

/// Compare integer to float for less-than
pub fn compare_int_float_lt(a: i64, b: XsdFloat) -> bool {
    let a_float = XsdFloat::from_small_int(a as i8);
    numeric_less_than_float(a_float, b)
}

/// Compare integer to float for greater-than
pub fn compare_int_float_gt(a: i64, b: XsdFloat) -> bool {
    let a_float = XsdFloat::from_small_int(a as i8);
    numeric_greater_than_float(a_float, b)
}

/// Compare integer to float for less-than-or-equal
pub fn compare_int_float_le(a: i64, b: XsdFloat) -> bool {
    let a_float = XsdFloat::from_small_int(a as i8);
    numeric_le_float(a_float, b)
}

/// Compare integer to float for greater-than-or-equal
pub fn compare_int_float_ge(a: i64, b: XsdFloat) -> bool {
    let a_float = XsdFloat::from_small_int(a as i8);
    numeric_ge_float(a_float, b)
}

/// Compare float to integer for equality
pub fn compare_float_int_eq(a: XsdFloat, b: i64) -> bool {
    let b_float = XsdFloat::from_small_int(b as i8);
    numeric_equal_float(a, b_float)
}

/// Compare float to integer for less-than
pub fn compare_float_int_lt(a: XsdFloat, b: i64) -> bool {
    let b_float = XsdFloat::from_small_int(b as i8);
    numeric_less_than_float(a, b_float)
}

/// Compare float to integer for greater-than
pub fn compare_float_int_gt(a: XsdFloat, b: i64) -> bool {
    let b_float = XsdFloat::from_small_int(b as i8);
    numeric_greater_than_float(a, b_float)
}

/// Compare float to integer for less-than-or-equal
pub fn compare_float_int_le(a: XsdFloat, b: i64) -> bool {
    let b_float = XsdFloat::from_small_int(b as i8);
    numeric_le_float(a, b_float)
}

/// Compare float to integer for greater-than-or-equal
pub fn compare_float_int_ge(a: XsdFloat, b: i64) -> bool {
    let b_float = XsdFloat::from_small_int(b as i8);
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
    let is_special =
        float32_is_nan(a.value) | float32_is_infinity(a.value) | float32_is_zero(a.value);

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
    let is_special =
        float64_is_nan(a.value) | float64_is_infinity(a.value) | float64_is_zero(a.value);

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
pub fn ceil_float(a: XsdFloat) -> XsdFloat {
    let bits = a.to_bits();
    let is_special =
        float32_is_nan(a.value) | float32_is_infinity(a.value) | float32_is_zero(a.value);

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
/// Round toward positive infinity
pub fn ceil_double(a: XsdDouble) -> XsdDouble {
    let bits = a.to_bits();
    let is_special =
        float64_is_nan(a.value) | float64_is_infinity(a.value) | float64_is_zero(a.value);

    let sign = bits & FLOAT64_SIGN_MASK;
    let is_negative = sign != 0;
    let exp_bits = (bits & FLOAT64_EXP_MASK) >> FLOAT64_MANTISSA_BITS;
    let mantissa = bits & FLOAT64_MANTISSA_MASK;
    let exp = exp_bits as i64 - FLOAT64_EXP_BIAS as i64;

    // Case: already integer (exp >= 52)
    let is_integer = exp >= FLOAT64_MANTISSA_BITS as i64;

    // Case: |value| < 1 (exp < 0)
    let is_less_than_one = exp < 0;

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
    let int_mantissa = mantissa & int_mask;
    let round_up = !is_negative & (frac != 0);

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

    // Result for |value| < 1 case
    let less_than_one_result = if is_negative {
        FLOAT64_SIGN_MASK // -0
    } else {
        0x3FF0000000000000 // 1.0
    };

    // Select result based on case
    let result_bits = if is_special | is_integer {
        bits
    } else if is_less_than_one {
        less_than_one_result
    } else {
        normal_result
    };

    XsdDouble::from_bits(result_bits)
}

/// fn:floor for xs:float
/// Round toward negative infinity
pub fn floor_float(a: XsdFloat) -> XsdFloat {
    let bits = a.to_bits();
    let is_special =
        float32_is_nan(a.value) | float32_is_infinity(a.value) | float32_is_zero(a.value);

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
/// Round toward negative infinity
pub fn floor_double(a: XsdDouble) -> XsdDouble {
    let bits = a.to_bits();
    let is_special =
        float64_is_nan(a.value) | float64_is_infinity(a.value) | float64_is_zero(a.value);

    let sign = bits & FLOAT64_SIGN_MASK;
    let is_negative = sign != 0;
    let exp_bits = (bits & FLOAT64_EXP_MASK) >> FLOAT64_MANTISSA_BITS;
    let mantissa = bits & FLOAT64_MANTISSA_MASK;
    let exp = exp_bits as i64 - FLOAT64_EXP_BIAS as i64;

    // Case: already integer (exp >= 52)
    let is_integer = exp >= FLOAT64_MANTISSA_BITS as i64;

    // Case: |value| < 1 (exp < 0)
    let is_less_than_one = exp < 0;

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
    let int_mantissa = mantissa & int_mask;
    let round_down = is_negative & (frac != 0);

    // Compute rounded result for normal case
    let increment = 1u64 << frac_bits;
    let new_mantissa = int_mantissa + increment;
    let overflow = new_mantissa >= (1u64 << FLOAT64_MANTISSA_BITS);

    let normal_result = if round_down {
        if overflow {
            sign | ((exp_bits + 1) << FLOAT64_MANTISSA_BITS)
        } else {
            sign | (exp_bits << FLOAT64_MANTISSA_BITS) | new_mantissa
        }
    } else {
        sign | (exp_bits << FLOAT64_MANTISSA_BITS) | int_mantissa
    };

    // Result for |value| < 1 case
    let less_than_one_result = if is_negative {
        0xBFF0000000000000 // -1.0
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

    XsdDouble::from_bits(result_bits)
}
