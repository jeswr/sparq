// ============================================================================
// IEEE 754 Float32 Comparison Operations
// ============================================================================
//
// IEEE 754 comparison rules:
// - NaN is unordered with everything (including itself)
// - +0 == -0
// - -Inf < all finite numbers < +Inf
// - Comparisons are signed (negative numbers < positive numbers)
// ============================================================================

use crate::float32::helpers::{float32_is_nan, float32_is_zero};
use crate::types::IEEE754Float32;

// IEEE 754 equality comparison
// Returns true if a == b
// NaN != NaN (and NaN != anything)
// +0 == -0
pub fn float32_eq(a: IEEE754Float32, b: IEEE754Float32) -> bool {
    // NaN is not equal to anything, including itself
    let a_is_nan = float32_is_nan(a);
    let b_is_nan = float32_is_nan(b);

    let mut result = false;

    if !a_is_nan & !b_is_nan {
        // Both zeros are equal regardless of sign
        let a_is_zero = float32_is_zero(a);
        let b_is_zero = float32_is_zero(b);

        if a_is_zero & b_is_zero {
            result = true;
        } else {
            // Normal comparison: sign, exponent, and mantissa must all match
            result = (a.sign == b.sign) & (a.exponent == b.exponent) & (a.mantissa == b.mantissa);
        }
    }

    result
}

// IEEE 754 not-equal comparison
// Returns true if a != b
pub fn float32_ne(a: IEEE754Float32, b: IEEE754Float32) -> bool {
    !float32_eq(a, b)
}

// IEEE 754 less-than comparison
// Returns true if a < b
// If either operand is NaN, returns false (unordered)
pub fn float32_lt(a: IEEE754Float32, b: IEEE754Float32) -> bool {
    // NaN comparisons are always false
    let a_is_nan = float32_is_nan(a);
    let b_is_nan = float32_is_nan(b);

    let mut result = false;

    if !a_is_nan & !b_is_nan {
        let a_is_zero = float32_is_zero(a);
        let b_is_zero = float32_is_zero(b);

        // +0 and -0 are equal, so neither is less than the other
        if a_is_zero & b_is_zero {
            result = false;
        } else if a.sign != b.sign {
            // Different signs: negative < positive
            // a < b when a is negative (sign=1) and b is positive (sign=0)
            result = a.sign == 1;
        } else if a.sign == 0 {
            // Both positive: smaller exponent or same exponent with smaller mantissa
            if a.exponent != b.exponent {
                result = a.exponent < b.exponent;
            } else {
                result = a.mantissa < b.mantissa;
            }
        } else {
            // Both negative: larger magnitude means smaller value
            if a.exponent != b.exponent {
                result = a.exponent > b.exponent;
            } else {
                result = a.mantissa > b.mantissa;
            }
        }
    }

    result
}

// IEEE 754 less-than-or-equal comparison
// Returns true if a <= b
// If either operand is NaN, returns false (unordered)
pub fn float32_le(a: IEEE754Float32, b: IEEE754Float32) -> bool {
    // NaN comparisons are always false
    let a_is_nan = float32_is_nan(a);
    let b_is_nan = float32_is_nan(b);

    let mut result = false;

    if !a_is_nan & !b_is_nan {
        result = float32_lt(a, b) | float32_eq(a, b);
    }

    result
}

// IEEE 754 greater-than comparison
// Returns true if a > b
// If either operand is NaN, returns false (unordered)
pub fn float32_gt(a: IEEE754Float32, b: IEEE754Float32) -> bool {
    // a > b is equivalent to b < a
    float32_lt(b, a)
}

// IEEE 754 greater-than-or-equal comparison
// Returns true if a >= b
// If either operand is NaN, returns false (unordered)
pub fn float32_ge(a: IEEE754Float32, b: IEEE754Float32) -> bool {
    // a >= b is equivalent to b <= a
    float32_le(b, a)
}

// IEEE 754 unordered comparison
// Returns true if either operand is NaN
pub fn float32_unordered(a: IEEE754Float32, b: IEEE754Float32) -> bool {
    float32_is_nan(a) | float32_is_nan(b)
}

// IEEE 754 total ordering
// Returns -1 if a < b, 0 if a == b, 1 if a > b
// For NaN, this follows IEEE 754-2008 totalOrder: -NaN < -Inf < ... < -0 < +0 < ... < +Inf < +NaN
// Note: This function provides a total ordering even for NaN values
pub fn float32_compare(a: IEEE754Float32, b: IEEE754Float32) -> i8 {
    let a_is_nan = float32_is_nan(a);
    let b_is_nan = float32_is_nan(b);

    let mut result: i8 = 0;

    if a_is_nan & b_is_nan {
        // Both NaN: compare by sign (negative NaN < positive NaN)
        if a.sign != b.sign {
            result = if a.sign == 1 { -1 } else { 1 };
        } else {
            // Same sign NaNs: compare payloads for total ordering
            // Negative NaNs: larger payload means smaller (more negative)
            // Positive NaNs: larger payload means larger
            if a.mantissa < b.mantissa {
                result = if a.sign == 1 { 1 } else { -1 };
            } else if a.mantissa > b.mantissa {
                result = if a.sign == 1 { -1 } else { 1 };
            }
        }
    } else if a_is_nan {
        // a is NaN, b is not: -NaN < everything < +NaN
        result = if a.sign == 1 { -1 } else { 1 };
    } else if b_is_nan {
        // b is NaN, a is not: -NaN < everything < +NaN
        result = if b.sign == 1 { 1 } else { -1 };
    } else {
        // Neither is NaN: standard comparison
        if float32_lt(a, b) {
            result = -1;
        } else if float32_gt(a, b) {
            result = 1;
        }
        // else result stays 0 (equal)
    }

    result
}
