// ============================================================================
// IEEE 754 Float32 Division
// ============================================================================

use crate::float32::helpers::{
    float32_infinity, float32_is_denormal, float32_is_infinity, float32_is_nan, float32_is_zero,
    float32_nan, float32_zero,
};
use crate::types::{
    FLOAT32_EXPONENT_MAX, FLOAT32_IMPLICIT_BIT, IEEE754Float32, ROUNDING_MODE_NEAREST_EVEN,
};
use crate::utils::{shift_right_sticky_u64, should_round_up_4bit};

// IEEE 754 compliant division for 32-bit floats with rounding mode
pub fn div_float32_with_rounding(
    a: IEEE754Float32,
    b: IEEE754Float32,
    rounding_mode: u8,
) -> IEEE754Float32 {
    // Determine special cases
    let a_is_nan = float32_is_nan(a);
    let b_is_nan = float32_is_nan(b);
    let a_is_inf = float32_is_infinity(a);
    let b_is_inf = float32_is_infinity(b);
    let a_is_zero = float32_is_zero(a);
    let b_is_zero = float32_is_zero(b);
    let a_is_denormal = float32_is_denormal(a);
    let b_is_denormal = float32_is_denormal(b);

    // Result sign is XOR of input signs
    let result_sign: u8 = a.sign ^ b.sign;

    // Get mantissas and normalize denormals
    // For denormals, we need to normalize them to have the implicit bit at position 23
    // and adjust the exponent accordingly

    // Count leading zeros for denormal normalization (23-bit mantissa field)
    // Goal: find shift amount to put highest set bit at position 23 (implicit bit position)
    // This equals (23 - highest_bit_position) = leading_zeros_from_bit22 + 1
    let a_mant_raw: u64 = a.mantissa as u64;
    let mut a_lz: i64 = 0;
    if a_is_denormal & (a_mant_raw != 0) {
        let mut v = a_mant_raw;
        // Binary search for leading zeros relative to bit 22
        if (v & 0x7FF800) == 0 {
            a_lz = a_lz + 12;
            v = v << 12;
        } // bits 11-22
        if (v & 0x7E0000) == 0 {
            a_lz = a_lz + 6;
            v = v << 6;
        } // bits 17-22
        if (v & 0x700000) == 0 {
            a_lz = a_lz + 3;
            v = v << 3;
        } // bits 20-22
        if (v & 0x600000) == 0 {
            a_lz = a_lz + 2;
            v = v << 2;
        } // bits 21-22
        if (v & 0x400000) == 0 {
            a_lz = a_lz + 1;
        } // bit 22
        // Add 1 to get shift to position 23
        a_lz = a_lz + 1;
    }

    let b_mant_raw: u64 = b.mantissa as u64;
    let mut b_lz: i64 = 0;
    if b_is_denormal & (b_mant_raw != 0) {
        let mut v = b_mant_raw;
        if (v & 0x7FF800) == 0 {
            b_lz = b_lz + 12;
            v = v << 12;
        }
        if (v & 0x7E0000) == 0 {
            b_lz = b_lz + 6;
            v = v << 6;
        }
        if (v & 0x700000) == 0 {
            b_lz = b_lz + 3;
            v = v << 3;
        }
        if (v & 0x600000) == 0 {
            b_lz = b_lz + 2;
            v = v << 2;
        }
        if (v & 0x400000) == 0 {
            b_lz = b_lz + 1;
        }
        b_lz = b_lz + 1;
    }

    // Get effective mantissas with implicit bit (24 bits)
    // For denormals, shift to normalize (put highest bit at position 23)
    let mant_a: u64 = if a_is_denormal {
        a_mant_raw << (a_lz as u64)
    } else if a_is_zero {
        0
    } else {
        (a.mantissa | FLOAT32_IMPLICIT_BIT) as u64
    };

    let mant_b: u64 = if b_is_denormal {
        b_mant_raw << (b_lz as u64)
    } else if b_is_zero {
        0
    } else {
        (b.mantissa | FLOAT32_IMPLICIT_BIT) as u64
    };

    // Get effective exponents (unbiased), accounting for denormal normalization
    // For denormal: base exp is -126, but we shifted by lz bits, so subtract lz
    let exp_a_eff: i64 = if a_is_denormal {
        1 - 127 - a_lz
    } else if a_is_zero {
        0
    } else {
        (a.exponent as i64) - 127
    };

    let exp_b_eff: i64 = if b_is_denormal {
        1 - 127 - b_lz
    } else if b_is_zero {
        0
    } else {
        (b.exponent as i64) - 127
    };

    // For division of normalized numbers:
    // mant_a / mant_b where both have implicit bit at position 23
    // Both mantissas represent values in [1, 2) with implicit bit
    // Their quotient is in range [0.5, 2)
    // After shifting dividend left by 27, quotient represents (mant_a/mant_b) * 2^27

    // Shift dividend left by 27 bits to get 27 quotient bits (24 + 3 guard)
    let shifted_dividend: u64 = mant_a << 27;

    // Perform division
    let quotient: u64 = if mant_b != 0 {
        shifted_dividend / mant_b
    } else {
        0
    };

    let remainder: u64 = if mant_b != 0 {
        shifted_dividend % mant_b
    } else {
        0
    };

    // Result exponent
    let mut result_exp: i64 = exp_a_eff - exp_b_eff + 127;

    // The quotient structure:
    // - If mant_a >= mant_b (quotient >= 1.0): quotient in [2^27, 2^28), bit 27 is implicit bit
    // - If mant_a < mant_b (quotient < 1.0): quotient in [2^26, 2^27), bit 26 is highest

    let mut result_mant: u64 = quotient;

    // Check if quotient >= 2^28 (result >= 2.0), need to shift right
    let bit_28_set = (quotient >> 28) & 1 == 1;
    if bit_28_set {
        result_mant = shift_right_sticky_u64(quotient, 1);
        result_exp = result_exp + 1;
    }

    // Check if quotient < 2^27 (result < 1.0), need to shift left and decrease exponent
    // This happens when mant_a < mant_b
    let bit_27_clear = (quotient >> 27) & 1 == 0;
    if bit_27_clear & !bit_28_set & (quotient != 0) {
        result_mant = quotient << 1;
        result_exp = result_exp - 1;
    }

    // Add sticky bit from remainder
    if remainder != 0 {
        result_mant = result_mant | 1;
    }

    // Now result_mant should have implicit bit at position 27
    // We need to shift down to position 23, keeping 3 guard bits at positions 2-0
    // Final position: implicit at 23, mantissa at 22-0, so we need to shift from 27 to 26 (leaving 3 guard bits)
    // Wait, if implicit is at 27, shifting right by 4 gives implicit at 23
    // But we want 3 guard bits, so shift by (27 - 23 - 3) = 1? No...

    // Let's think again:
    // result_mant has implicit bit at position 27
    // We want final mantissa to have implicit bit at position 23
    // We need 3 guard bits (positions 2, 1, 0 of result_mant before final shift)
    // So shift from position 27 to position 26 (shift right by 1) then the guard bits are at 3, 2, 1, 0
    // Then shift right by 3 more to remove guard bits
    // Total shift: 27 - 26 = 1, but we already have it at 27

    // Actually, let's be clear:
    // result_mant format: implicit bit at 27, mantissa bits at 26-4, guard bits at 3-0
    // We want: implicit bit at 23, mantissa bits at 22-0
    // Shift right by 4 (27-23), then guard bits are the lowest 4 bits
    // But IEEE 754 uses 3 guard bits, so let's use shift of 27-26=1 first for alignment
    // No wait, the standard is guard (G), round (R), sticky (S) = 3 bits

    // Simpler approach: shift right by (27 - 23) = 4, use lowest 4 bits as guard
    // Or shift right by (27 - 26) = 1, then lowest 3 bits as guard, then shift by 3 more

    // Current: implicit at 27
    // Shift right by 1: implicit at 26, lowest bit is guard1
    // Shift right by 3 more: implicit at 23, lowest 3 bits were guard bits
    // Total shift: 4
    // Guard bits: original bits 3, 2, 1, 0 after first shift... this is getting confusing.

    // Let me use a clean approach:
    // result_mant has implicit bit at position 27
    // We want implicit bit at position 23 with 4 guard bits
    // But first handle denormalization BEFORE rounding

    // Handle underflow to denormal BEFORE rounding
    // Currently implicit bit is at position 27, we want to keep 4 guard bits
    // So target implicit position is 27 (with guard bits at 3-0)
    let mut is_denormal_result = false;
    if result_exp <= 0 {
        let denorm_shift = 1 - result_exp;
        // Need to shift more to account for denormalization
        // The result_mant currently has implicit at position 27
        // We need to shift by (denorm_shift + 4) to put implicit at position (23 - denorm_shift)
        // But we keep 4 guard bits, so effectively we shift by denorm_shift here
        // and then the normal shift of 4 later
        if denorm_shift < 28 {
            result_mant = shift_right_sticky_u64(result_mant, denorm_shift as u64);
        } else {
            result_mant = 0;
        }
        result_exp = 0;
        is_denormal_result = true;
    }

    // Now shift to remove guard bits and round
    let guard_bits = result_mant & 0xF; // 4 bits
    result_mant = result_mant >> 4;

    // Round to nearest, ties to even (using 4-bit guard, threshold is 8)
    let should_round = if rounding_mode == 0 {
        // NEAREST_EVEN (optimized inline)
        (guard_bits > 8) | ((guard_bits == 8) & ((result_mant & 1) == 1))
    } else {
        should_round_up_4bit(guard_bits, result_mant, result_sign, rounding_mode)
    };
    if should_round {
        result_mant = result_mant + 1;
        // Check for overflow from rounding
        if !is_denormal_result & (result_mant >= ((FLOAT32_IMPLICIT_BIT as u64) << 1)) {
            result_mant = result_mant >> 1;
            result_exp = result_exp + 1;
        }
        // For denormals, if rounding causes overflow into normal range, adjust
        if is_denormal_result & (result_mant >= (FLOAT32_IMPLICIT_BIT as u64)) {
            result_exp = 1;
            is_denormal_result = false;
        }
    }

    // Check for overflow to infinity
    let overflows_to_inf = result_exp >= (FLOAT32_EXPONENT_MAX as i64);

    // Remove implicit bit for final mantissa
    let final_mantissa = if is_denormal_result {
        (result_mant & 0x7FFFFF) as u32
    } else {
        (result_mant & ((FLOAT32_IMPLICIT_BIT - 1) as u64)) as u32
    };

    let normal_result =
        IEEE754Float32 { sign: result_sign, exponent: result_exp as u8, mantissa: final_mantissa };

    // Handle special cases
    let mut result = normal_result;

    // Zero dividend
    if a_is_zero {
        if b_is_zero | b_is_nan {
            result = float32_nan();
        } else {
            result = float32_zero(result_sign);
        }
    }

    // Zero divisor (division by zero)
    if b_is_zero & !a_is_zero & !a_is_nan {
        result = float32_infinity(result_sign);
    }

    // Overflow to infinity
    if overflows_to_inf & !a_is_zero & !b_is_inf {
        result = float32_infinity(result_sign);
    }

    // Handle infinity dividend
    if a_is_inf & !b_is_nan {
        if b_is_inf {
            result = float32_nan(); // inf / inf = NaN
        } else {
            result = float32_infinity(result_sign);
        }
    }

    // Handle infinity divisor
    if b_is_inf & !a_is_inf & !a_is_nan {
        result = float32_zero(result_sign);
    }

    // Handle NaN inputs (highest priority)
    if a_is_nan | b_is_nan {
        result = float32_nan();
    }

    result
}

// IEEE 754 compliant division for 32-bit floats (default rounding mode)
// Backward-compatible wrapper that uses round-to-nearest-ties-to-even
pub fn div_float32(a: IEEE754Float32, b: IEEE754Float32) -> IEEE754Float32 {
    div_float32_with_rounding(a, b, ROUNDING_MODE_NEAREST_EVEN)
}
