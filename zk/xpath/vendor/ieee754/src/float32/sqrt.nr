// ============================================================================
// IEEE 754 Float32 Square Root
// ============================================================================
//
// IEEE 754 square root rules:
// - sqrt(+0) = +0
// - sqrt(-0) = -0
// - sqrt(+Inf) = +Inf
// - sqrt(negative) = NaN (except -0)
// - sqrt(NaN) = NaN
// ============================================================================

use crate::float32::helpers::{
    float32_infinity, float32_is_denormal, float32_is_infinity, float32_is_nan, float32_is_zero,
    float32_nan, float32_zero,
};
use crate::types::{
    FLOAT32_EXPONENT_MAX, FLOAT32_IMPLICIT_BIT, IEEE754Float32, ROUNDING_MODE_NEAREST_EVEN,
};
use crate::utils::{shift_right_sticky_u64, should_round_up};

// IEEE 754 compliant square root for 32-bit floats with rounding mode
pub fn sqrt_float32_with_rounding(a: IEEE754Float32, rounding_mode: u8) -> IEEE754Float32 {
    // Handle special cases
    let a_is_nan = float32_is_nan(a);
    let a_is_inf = float32_is_infinity(a);
    let a_is_zero = float32_is_zero(a);
    let a_is_denormal = float32_is_denormal(a);
    let a_is_negative = a.sign == 1;

    // Normalize denormals and get effective exponent
    let a_mant_raw: u64 = a.mantissa as u64;
    let mut a_lz: i64 = 0;
    if a_is_denormal & (a_mant_raw != 0) {
        let mut v = a_mant_raw;
        // Binary search for leading zeros relative to bit 22
        if (v & 0x7FF800) == 0 {
            a_lz = a_lz + 12;
            v = v << 12;
        }
        if (v & 0x7E0000) == 0 {
            a_lz = a_lz + 6;
            v = v << 6;
        }
        if (v & 0x700000) == 0 {
            a_lz = a_lz + 3;
            v = v << 3;
        }
        if (v & 0x600000) == 0 {
            a_lz = a_lz + 2;
            v = v << 2;
        }
        if (v & 0x400000) == 0 {
            a_lz = a_lz + 1;
        }
        a_lz = a_lz + 1;
    }

    // Get mantissa with implicit bit (24 bits, implicit at position 23)
    let mant_a: u64 = if a_is_denormal {
        a_mant_raw << (a_lz as u64)
    } else {
        (a.mantissa | FLOAT32_IMPLICIT_BIT) as u64
    };

    // Get effective exponent (unbiased)
    let exp_a_eff: i64 = if a_is_denormal {
        1 - 127 - a_lz
    } else {
        (a.exponent as i64) - 127
    };

    // For sqrt: result_exp = floor(exp_a_eff / 2)
    // If exp_a_eff is odd, we need to adjust the mantissa
    // sqrt(m * 2^e) = sqrt(m) * 2^(e/2) if e is even
    // sqrt(m * 2^e) = sqrt(2*m) * 2^((e-1)/2) if e is odd

    let exp_is_odd = (exp_a_eff & 1) != 0;
    let mut result_exp: i64 = if exp_is_odd {
        (exp_a_eff - 1) / 2 + 127
    } else {
        exp_a_eff / 2 + 127
    };

    // Adjust mantissa for odd exponent (multiply by 2, i.e., shift left by 1)
    // The mantissa is in [1, 2) with implicit bit at position 23
    // After adjustment for odd exp, it's in [1, 4) with value at positions 23-24
    let adjusted_mant: u64 = if exp_is_odd { mant_a << 1 } else { mant_a };

    // Square root algorithm:
    // We want to compute sqrt of the significand value (in [1,2) or [1,4)) and scale result.
    //
    // The significand = adjusted_mant / 2^23 (or 2^24 for odd exp)
    // We want result significand also with implicit bit at position 23.
    //
    // For integer sqrt: sqrt(X) gives us floor(sqrt(X))
    // We want: result_mant / 2^26 = sqrt(adjusted_mant / 2^k) where k = 23 or 24
    // => result_mant = sqrt(adjusted_mant) * 2^(26 - k/2)
    //                = sqrt(adjusted_mant * 2^(52 - k))
    //
    // For even exp: k=23, so radicand = adjusted_mant << 29 = adjusted_mant * 2^29 = mant_a * 2^29
    // For odd exp:  k=24, so radicand = adjusted_mant << 28 = adjusted_mant * 2^28 = mant_a * 2^29
    //
    // So shift amount is 29 in both cases!

    let radicand: u64 = adjusted_mant << 29;

    // Digit-by-digit square root algorithm (binary restoring method)
    // Computes floor(sqrt(radicand)) by processing 2 bits at a time
    //
    // Invariant: at step i, remainder = radicand[0:2i] - result[0:i]^2
    // We try setting the next bit: (result*2 + 1)^2 = 4*result^2 + 4*result + 1
    // So we check if 4*remainder + next_2_bits >= 4*result + 1
    //
    // For radicand ~ 2^52, result is ~ 2^26

    let mut sqrt_result: u64 = 0;
    let mut remainder: u64 = 0;

    // Process radicand from MSB, 2 bits at a time (27 iterations for 54-bit input)
    for i in 0..27 {
        // Shift remainder left by 2 and add next two bits from radicand
        let shift_pos = 52 - 2 * i; // Starts at 52, goes to 0
        let two_bits = if shift_pos >= 0 {
            (radicand >> (shift_pos as u64)) & 3
        } else {
            0
        };
        remainder = (remainder << 2) | two_bits;

        // Candidate for subtraction: (result << 2) | 1 = 4*result + 1
        let cand = (sqrt_result << 2) | 1;

        if remainder >= cand {
            remainder = remainder - cand;
            sqrt_result = (sqrt_result << 1) | 1;
        } else {
            sqrt_result = sqrt_result << 1;
        }
    }

    // Now sqrt_result has ~27 bits with implicit bit at position 26
    // We need implicit bit at position 23, so shift right by 3 (keeping guard bits)

    // Add sticky bit if remainder is non-zero
    if remainder != 0 {
        sqrt_result = sqrt_result | 1;
    }

    // sqrt_result format: implicit bit at position 26, we want at position 23
    // So we have 3 extra bits (guard bits) at positions 2, 1, 0

    // Handle underflow to denormal
    let mut is_denormal_result = false;
    let mut result_mant = sqrt_result;

    if result_exp <= 0 {
        let denorm_shift = 1 - result_exp;
        if denorm_shift < 27 {
            result_mant = shift_right_sticky_u64(result_mant, denorm_shift as u64);
        } else {
            result_mant = 0;
        }
        result_exp = 0;
        is_denormal_result = true;
    }

    // Shift to remove guard bits (shift by 3)
    let guard_bits = result_mant & 0x7; // 3 guard bits
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
        if !is_denormal_result & (result_mant >= ((FLOAT32_IMPLICIT_BIT as u64) << 1)) {
            result_mant = result_mant >> 1;
            result_exp = result_exp + 1;
        }
        // For denormals, if rounding causes overflow into normal range
        if is_denormal_result & (result_mant >= (FLOAT32_IMPLICIT_BIT as u64)) {
            result_exp = 1;
            is_denormal_result = false;
        }
    }

    // Check for overflow to infinity (shouldn't happen for sqrt, but be safe)
    let overflows_to_inf = result_exp >= (FLOAT32_EXPONENT_MAX as i64);

    // Remove implicit bit for final mantissa
    let final_mantissa = if is_denormal_result {
        (result_mant & 0x7FFFFF) as u32
    } else {
        (result_mant & ((FLOAT32_IMPLICIT_BIT - 1) as u64)) as u32
    };

    let normal_result =
        IEEE754Float32 { sign: 0, exponent: result_exp as u8, mantissa: final_mantissa };

    // Handle special cases (highest priority last)
    let mut result = normal_result;

    // Overflow to infinity
    if overflows_to_inf {
        result = float32_infinity(0);
    }

    // sqrt(+0) = +0, sqrt(-0) = -0
    if a_is_zero {
        result = float32_zero(a.sign);
    }

    // sqrt(+Inf) = +Inf
    if a_is_inf & !a_is_negative {
        result = float32_infinity(0);
    }

    // sqrt of negative number (except -0) is NaN
    if a_is_negative & !a_is_zero {
        result = float32_nan();
    }

    // NaN input produces NaN
    if a_is_nan {
        result = float32_nan();
    }

    result
}

// IEEE 754 compliant square root for 32-bit floats (default rounding mode)
// Backward-compatible wrapper that uses round-to-nearest-ties-to-even
pub fn sqrt_float32(a: IEEE754Float32) -> IEEE754Float32 {
    sqrt_float32_with_rounding(a, ROUNDING_MODE_NEAREST_EVEN)
}
