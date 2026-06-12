// ============================================================================
// IEEE 754 Float64 Operations Module
// ============================================================================

pub(crate) mod helpers;
pub(crate) mod add;
pub(crate) mod mul;
pub(crate) mod div;
pub(crate) mod cmp;
pub(crate) mod sqrt;

// Re-export all public functions
pub use helpers::{
    float64_from_bits, float64_infinity, float64_is_denormal, float64_is_infinity, float64_is_nan,
    float64_is_zero, float64_nan, float64_to_bits, float64_zero,
};

pub use add::{add_float64, add_float64_with_rounding};
pub use cmp::{
    float64_compare, float64_eq, float64_ge, float64_gt, float64_le, float64_lt, float64_ne,
    float64_unordered,
};
pub use div::{div_float64, div_float64_with_rounding};
pub use mul::{mul_float64, mul_float64_with_rounding};
pub use sqrt::{sqrt_float64, sqrt_float64_with_rounding};

// Subtraction is implemented as a + (-b)
use crate::types::{IEEE754Float64, ROUNDING_MODE_NEAREST_EVEN};

// IEEE 754 compliant subtraction for 64-bit floats with rounding mode
// Implemented as a + (-b)
pub fn sub_float64_with_rounding(
    a: IEEE754Float64,
    b: IEEE754Float64,
    rounding_mode: u8,
) -> IEEE754Float64 {
    // Negate b's sign and add
    let neg_b = IEEE754Float64 { sign: 1 - b.sign, exponent: b.exponent, mantissa: b.mantissa };
    add_float64_with_rounding(a, neg_b, rounding_mode)
}

// IEEE 754 compliant subtraction for 64-bit floats (default rounding mode)
// Backward-compatible wrapper that uses round-to-nearest-ties-to-even
pub fn sub_float64(a: IEEE754Float64, b: IEEE754Float64) -> IEEE754Float64 {
    sub_float64_with_rounding(a, b, ROUNDING_MODE_NEAREST_EVEN)
}

// IEEE 754 compliant absolute value for 64-bit floats
// Returns the magnitude of the float (sign bit set to 0)
// Note: abs(NaN) returns NaN (with sign bit cleared)
pub fn abs_float64(a: IEEE754Float64) -> IEEE754Float64 {
    IEEE754Float64 { sign: 0, exponent: a.exponent, mantissa: a.mantissa }
}
