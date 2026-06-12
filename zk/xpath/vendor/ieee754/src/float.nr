// ============================================================================
// IEEE 754 Compliant Floating Point Library
// ============================================================================
//
// This module re-exports all IEEE 754 floating point types and operations
// from their respective submodules for backwards compatibility.
//
// Module structure:
// - types.nr: Core types (IEEE754Float32, IEEE754Float64) and constants
// - utils.nr: Shared utility functions (shift operations, U128 arithmetic)
// - float32/: Float32 operations (helpers, add, mul, div)
// - float64/: Float64 operations (helpers, add, mul, div)
// ============================================================================

// Re-export types and constants
pub use crate::types::{
    FLOAT32_EXPONENT_MAX, FLOAT32_IMPLICIT_BIT, FLOAT64_EXPONENT_MAX, FLOAT64_IMPLICIT_BIT,
    IEEE754Float32, IEEE754Float64,
    // Rounding mode constants
    ROUNDING_MODE_NEAREST_EVEN, ROUNDING_MODE_NEAREST_AWAY, ROUNDING_MODE_TOWARD_POSITIVE,
    ROUNDING_MODE_TOWARD_NEGATIVE, ROUNDING_MODE_TOWARD_ZERO,
    // Float32 special value constants
    FLOAT32_ZERO, FLOAT32_NEG_ZERO, FLOAT32_ONE, FLOAT32_NEG_ONE, FLOAT32_INFINITY,
    FLOAT32_NEG_INFINITY, FLOAT32_NAN, FLOAT32_SIGNALING_NAN,
    // Float64 special value constants
    FLOAT64_ZERO, FLOAT64_NEG_ZERO, FLOAT64_ONE, FLOAT64_NEG_ONE, FLOAT64_INFINITY,
    FLOAT64_NEG_INFINITY, FLOAT64_NAN, FLOAT64_SIGNALING_NAN,
};

// Re-export Float32 operations
pub use crate::float32::{
    add_float32, add_float32_with_rounding, div_float32, div_float32_with_rounding,
    float32_from_bits, float32_infinity, float32_is_denormal, float32_is_infinity, float32_is_nan,
    float32_is_zero, float32_nan, float32_to_bits, float32_zero, mul_float32,
    mul_float32_with_rounding, sub_float32, sub_float32_with_rounding, abs_float32,
    // Comparison operations
    float32_eq, float32_ne, float32_lt, float32_le, float32_gt, float32_ge, float32_unordered,
    float32_compare,
    // Square root
    sqrt_float32, sqrt_float32_with_rounding,
};

// Re-export Float64 operations
pub use crate::float64::{
    add_float64, add_float64_with_rounding, div_float64, div_float64_with_rounding,
    float64_from_bits, float64_infinity, float64_is_denormal, float64_is_infinity, float64_is_nan,
    float64_is_zero, float64_nan, float64_to_bits, float64_zero, mul_float64,
    mul_float64_with_rounding, sub_float64, sub_float64_with_rounding, abs_float64,
    // Comparison operations
    float64_eq, float64_ne, float64_lt, float64_le, float64_gt, float64_ge, float64_unordered,
    float64_compare,
    // Square root
    sqrt_float64, sqrt_float64_with_rounding,
};
