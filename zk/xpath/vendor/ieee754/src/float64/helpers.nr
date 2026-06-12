// ============================================================================
// IEEE 754 Float64 Helper Functions
// ============================================================================

use crate::types::{FLOAT64_EXPONENT_MAX, IEEE754Float64};

// Check if Float64 is NaN (exponent = 2047, mantissa != 0)
pub fn float64_is_nan(x: IEEE754Float64) -> bool {
    (x.exponent == FLOAT64_EXPONENT_MAX) & (x.mantissa != 0)
}

// Check if Float64 is Infinity (exponent = 2047, mantissa = 0)
pub fn float64_is_infinity(x: IEEE754Float64) -> bool {
    (x.exponent == FLOAT64_EXPONENT_MAX) & (x.mantissa == 0)
}

// Check if Float64 is Zero (exponent = 0, mantissa = 0)
pub fn float64_is_zero(x: IEEE754Float64) -> bool {
    (x.exponent == 0) & (x.mantissa == 0)
}

// Check if Float64 is Denormalized (exponent = 0, mantissa != 0)
pub fn float64_is_denormal(x: IEEE754Float64) -> bool {
    (x.exponent == 0) & (x.mantissa != 0)
}

// Create Float64 NaN
pub fn float64_nan() -> IEEE754Float64 {
    IEEE754Float64 { sign: 0, exponent: FLOAT64_EXPONENT_MAX, mantissa: 0x8000000000000 }
}

// Create Float64 Infinity
pub fn float64_infinity(sign: u8) -> IEEE754Float64 {
    IEEE754Float64 { sign, exponent: FLOAT64_EXPONENT_MAX, mantissa: 0 }
}

// Create Float64 Zero
pub fn float64_zero(sign: u8) -> IEEE754Float64 {
    IEEE754Float64 { sign, exponent: 0, mantissa: 0 }
}

// Convert from u64 bits to IEEE754Float64
pub fn float64_from_bits(bits: u64) -> IEEE754Float64 {
    let sign = ((bits >> 63) & 1) as u8;
    let exponent = ((bits >> 52) & 0x7FF) as u16;
    let mantissa = bits & 0xFFFFFFFFFFFFF;
    IEEE754Float64 { sign, exponent, mantissa }
}

// Convert IEEE754Float64 to u64 bits
pub fn float64_to_bits(f: IEEE754Float64) -> u64 {
    ((f.sign as u64) << 63) | ((f.exponent as u64) << 52) | f.mantissa
}
