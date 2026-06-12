// ============================================================================
// IEEE 754 Float32 Helper Functions
// ============================================================================

use crate::types::{FLOAT32_EXPONENT_MAX, IEEE754Float32};

// Check if Float32 is NaN (exponent = 255, mantissa != 0)
pub fn float32_is_nan(x: IEEE754Float32) -> bool {
    (x.exponent == FLOAT32_EXPONENT_MAX) & (x.mantissa != 0)
}

// Check if Float32 is Infinity (exponent = 255, mantissa = 0)
pub fn float32_is_infinity(x: IEEE754Float32) -> bool {
    (x.exponent == FLOAT32_EXPONENT_MAX) & (x.mantissa == 0)
}

// Check if Float32 is Zero (exponent = 0, mantissa = 0)
pub fn float32_is_zero(x: IEEE754Float32) -> bool {
    (x.exponent == 0) & (x.mantissa == 0)
}

// Check if Float32 is Denormalized (exponent = 0, mantissa != 0)
pub fn float32_is_denormal(x: IEEE754Float32) -> bool {
    (x.exponent == 0) & (x.mantissa != 0)
}

// Create Float32 NaN
pub fn float32_nan() -> IEEE754Float32 {
    IEEE754Float32 { sign: 0, exponent: FLOAT32_EXPONENT_MAX, mantissa: 0x400000 }
}

// Create Float32 Infinity
pub fn float32_infinity(sign: u8) -> IEEE754Float32 {
    IEEE754Float32 { sign, exponent: FLOAT32_EXPONENT_MAX, mantissa: 0 }
}

// Create Float32 Zero
pub fn float32_zero(sign: u8) -> IEEE754Float32 {
    IEEE754Float32 { sign, exponent: 0, mantissa: 0 }
}

// Convert from u32 bits to IEEE754Float32
pub fn float32_from_bits(bits: u32) -> IEEE754Float32 {
    let sign = ((bits >> 31) & 1) as u8;
    let exponent = ((bits >> 23) & 0xFF) as u8;
    let mantissa = bits & 0x7FFFFF;
    IEEE754Float32 { sign, exponent, mantissa }
}

// Convert IEEE754Float32 to u32 bits
pub fn float32_to_bits(f: IEEE754Float32) -> u32 {
    ((f.sign as u32) << 31) | ((f.exponent as u32) << 23) | f.mantissa
}
