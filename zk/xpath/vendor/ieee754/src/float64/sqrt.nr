// ============================================================================
// IEEE 754 Float64 Square Root
// ============================================================================
//
// IEEE 754 square root rules:
// - sqrt(+0) = +0
// - sqrt(-0) = -0
// - sqrt(+Inf) = +Inf
// - sqrt(negative) = NaN (except -0)
// - sqrt(NaN) = NaN
// ============================================================================

use crate::float64::helpers::{
    float64_infinity, float64_is_denormal, float64_is_infinity, float64_is_nan, float64_is_zero,
    float64_nan, float64_zero,
};
use crate::types::{
    FLOAT64_EXPONENT_MAX, FLOAT64_IMPLICIT_BIT, IEEE754Float64, ROUNDING_MODE_NEAREST_EVEN,
};
use crate::utils::{shift_right_sticky_u64, should_round_up, U128};

// IEEE 754 compliant square root for 64-bit floats with rounding mode
pub fn sqrt_float64_with_rounding(a: IEEE754Float64, rounding_mode: u8) -> IEEE754Float64 {
    // Handle special cases
    let a_is_nan = float64_is_nan(a);
    let a_is_inf = float64_is_infinity(a);
    let a_is_zero = float64_is_zero(a);
    let a_is_denormal = float64_is_denormal(a);
    let a_is_negative = a.sign == 1;

    // Normalize denormals and get effective exponent
    let a_mant_raw: u64 = a.mantissa;
    let mut a_lz: i64 = 0;
    if a_is_denormal & (a_mant_raw != 0) {
        let mut v = a_mant_raw;
        // Binary search for leading zeros relative to bit 51
        if (v & 0xFFFFFFF000000) == 0 {
            a_lz = a_lz + 28;
            v = v << 28;
        }
        if (v & 0xFFFC000000000) == 0 {
            a_lz = a_lz + 14;
            v = v << 14;
        }
        if (v & 0xFE00000000000) == 0 {
            a_lz = a_lz + 7;
            v = v << 7;
        }
        if (v & 0xF000000000000) == 0 {
            a_lz = a_lz + 4;
            v = v << 4;
        }
        if (v & 0xC000000000000) == 0 {
            a_lz = a_lz + 2;
            v = v << 2;
        }
        if (v & 0x8000000000000) == 0 {
            a_lz = a_lz + 1;
        }
        a_lz = a_lz + 1;
    }

    // Get mantissa with implicit bit (53 bits, implicit at position 52)
    let mant_a: u64 = if a_is_denormal {
        a_mant_raw << (a_lz as u64)
    } else {
        a.mantissa | FLOAT64_IMPLICIT_BIT
    };

    // Get effective exponent (unbiased)
    let exp_a_eff: i64 = if a_is_denormal {
        1 - 1023 - a_lz
    } else {
        (a.exponent as i64) - 1023
    };

    // For sqrt: result_exp = floor(exp_a_eff / 2)
    // If exp_a_eff is odd, we need to adjust the mantissa
    let exp_is_odd = (exp_a_eff & 1) != 0;
    let mut result_exp: i64 = if exp_is_odd {
        (exp_a_eff - 1) / 2 + 1023
    } else {
        exp_a_eff / 2 + 1023
    };

    // Adjust mantissa for odd exponent (multiply by 2)
    let adjusted_mant: u64 = if exp_is_odd { mant_a << 1 } else { mant_a };

    // Square root algorithm scaling:
    // - For result with implicit at position 55 (52 mantissa + 3 guard)
    // - result / 2^55 = sqrt(adjusted_mant / 2^52) (with binary point after bit 52)
    // - result = sqrt(adjusted_mant / 2^52) * 2^55 = sqrt(adjusted_mant) * 2^(55-26) = sqrt(adjusted_mant) * 2^29
    // - result = sqrt(adjusted_mant * 2^58)
    //
    // This works for both even and odd exp (shift is 58 in both cases).
    // For odd exp, adjusted_mant represents value in [2,4), sqrt in [sqrt(2), 2).
    // For even exp, adjusted_mant represents value in [1,2), sqrt in [1, sqrt(2)).

    let radicand_shift: u64 = 58;

    // Create 128-bit radicand
    let radicand_high: u64 = adjusted_mant >> (64 - radicand_shift);
    let radicand_low: u64 = adjusted_mant << radicand_shift;

    // Digit-by-digit square root algorithm (binary restoring method) for up to 112-bit input
    // Computes floor(sqrt(radicand)) by processing 2 bits at a time
    //
    // We use 56 iterations to handle radicand up to 2^111

    let mut sqrt_result: u64 = 0;
    let mut remainder = U128 { high: 0, low: 0 };

    // Process up to 112 bits of input, 2 at a time (56 iterations)
    for i in 0..56 {
        // Shift remainder left by 2
        remainder =
            U128 { high: (remainder.high << 2) | (remainder.low >> 62), low: remainder.low << 2 };

        // Add next two bits from radicand
        // bit position: 110 - 2*i for the pair (starts at bits 110-111, ends at 0-1)
        let shift_pos: i64 = 110 - 2 * (i as i64);
        let two_bits: u64 = if shift_pos >= 64 {
            (radicand_high >> ((shift_pos - 64) as u64)) & 3
        } else if shift_pos >= 0 {
            (radicand_low >> (shift_pos as u64)) & 3
        } else {
            0
        };
        remainder.low = remainder.low | two_bits;

        // Candidate: (result << 2) | 1 = 4*result + 1
        // This fits in 64 bits since result < 2^57
        let cand: u64 = (sqrt_result << 2) | 1;

        // Compare remainder >= cand
        let remainder_ge_cand =
            (remainder.high > 0) | ((remainder.high == 0) & (remainder.low >= cand));

        if remainder_ge_cand {
            // remainder = remainder - cand
            if remainder.low >= cand {
                remainder.low = remainder.low - cand;
            } else {
                remainder.high = remainder.high - 1;
                remainder.low = remainder.low + (0 - cand); // wrapping subtraction
            }
            sqrt_result = (sqrt_result << 1) | 1;
        } else {
            sqrt_result = sqrt_result << 1;
        }
    }

    // sqrt_result has ~56 bits with implicit bit at position 55
    // We need implicit bit at position 52, so we have 3 guard bits (positions 2,1,0)
    let mut result_mant = sqrt_result;

    // Add sticky bit if remainder is non-zero
    if (remainder.high != 0) | (remainder.low != 0) {
        result_mant = result_mant | 1;
    }

    // result_mant format: implicit bit at position 55, we want at position 52
    // So we have 3 guard bits at positions 2, 1, 0

    // Handle underflow to denormal
    let mut is_denormal_result = false;

    if result_exp <= 0 {
        let denorm_shift = 1 - result_exp;
        if denorm_shift < 57 {
            result_mant = shift_right_sticky_u64(result_mant, denorm_shift as u64);
        } else {
            result_mant = 0;
        }
        result_exp = 0;
        is_denormal_result = true;
    }

    // Shift to remove guard bits (shift by 3)
    let guard_bits = result_mant & 0x7;
    result_mant = result_mant >> 3;

    // Round to nearest, ties to even
    let should_round = if rounding_mode == 0 {
        // NEAREST_EVEN (optimized inline)
        (guard_bits > 4) | ((guard_bits == 4) & ((result_mant & 1) == 1))
    } else {
        should_round_up(guard_bits, result_mant, 0, rounding_mode)
    }; // sqrt result is always positive
    if should_round {
        result_mant = result_mant + 1;
        // Check for overflow from rounding
        if !is_denormal_result & (result_mant >= (FLOAT64_IMPLICIT_BIT << 1)) {
            result_mant = result_mant >> 1;
            result_exp = result_exp + 1;
        }
        // For denormals, if rounding causes overflow into normal range
        if is_denormal_result & (result_mant >= FLOAT64_IMPLICIT_BIT) {
            result_exp = 1;
            is_denormal_result = false;
        }
    }

    // Check for overflow to infinity (shouldn't happen for sqrt)
    let overflows_to_inf = result_exp >= (FLOAT64_EXPONENT_MAX as i64);

    // Remove implicit bit for final mantissa
    let final_mantissa = if is_denormal_result {
        result_mant & 0xFFFFFFFFFFFFF
    } else {
        result_mant & (FLOAT64_IMPLICIT_BIT - 1)
    };

    let normal_result =
        IEEE754Float64 { sign: 0, exponent: result_exp as u16, mantissa: final_mantissa };

    // Handle special cases (highest priority last)
    let mut result = normal_result;

    // Overflow to infinity
    if overflows_to_inf {
        result = float64_infinity(0);
    }

    // sqrt(+0) = +0, sqrt(-0) = -0
    if a_is_zero {
        result = float64_zero(a.sign);
    }

    // sqrt(+Inf) = +Inf
    if a_is_inf & !a_is_negative {
        result = float64_infinity(0);
    }

    // sqrt of negative number (except -0) is NaN
    if a_is_negative & !a_is_zero {
        result = float64_nan();
    }

    // NaN input produces NaN
    if a_is_nan {
        result = float64_nan();
    }

    result
}

// IEEE 754 compliant square root for 64-bit floats (default rounding mode)
// Backward-compatible wrapper that uses round-to-nearest-ties-to-even
pub fn sqrt_float64(a: IEEE754Float64) -> IEEE754Float64 {
    sqrt_float64_with_rounding(a, ROUNDING_MODE_NEAREST_EVEN)
}
