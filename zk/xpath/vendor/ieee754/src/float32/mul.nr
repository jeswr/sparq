// ============================================================================
// IEEE 754 Float32 Multiplication
// ============================================================================

use crate::float32::helpers::{
    float32_infinity, float32_is_denormal, float32_is_infinity, float32_is_nan, float32_is_zero,
    float32_nan, float32_zero,
};
use crate::types::{
    FLOAT32_EXPONENT_MAX, FLOAT32_IMPLICIT_BIT, IEEE754Float32, ROUNDING_MODE_NEAREST_EVEN,
};
use crate::utils::{shift_right_sticky_u64, should_round_up};

// IEEE 754 compliant multiplication for 32-bit floats with rounding mode
pub fn mul_float32_with_rounding(
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

    // Multiply mantissas (24-bit * 24-bit = 48-bit max)
    // For two normalized numbers: 1.xxx * 1.yyy gives result in range [1, 4)
    // The product will have the implicit bit at position 46 or 47
    let product: u64 = mant_a * mant_b;

    // Special case: result is zero
    let result_is_zero = product == 0;

    // For multiplication of normalized numbers:
    // mant_a = 1.xxx (implicit bit at position 23)
    // mant_b = 1.yyy (implicit bit at position 23)
    // product = result in [2^46, 2^48) for normalized * normalized
    // If bit 47 is set, result >= 2.0, need to shift right by 1
    // If bit 46 is set but not 47, result in [1, 2), normal case

    // Result exponent starts at exp_a + exp_b
    let mut result_exp: i64 = exp_a_eff + exp_b_eff + 127;
    let mut result_mant: u64 = product;

    // Normalize the product: find leading bit and adjust
    // For normalized * normalized, leading bit is at 46 or 47
    // For denormal * normal, it can be lower
    // We need the leading bit at position 46 before the guard shift

    // Check if bit 47 is set (overflow case)
    let bit_47_set = (product >> 47) & 1 == 1;
    if bit_47_set {
        result_exp = result_exp + 1;
        // Shift right by 1, keeping sticky bit
        result_mant = shift_right_sticky_u64(product, 1);
    }

    // Now normalize if the leading bit is below position 46
    // Use binary search to count leading zeros
    if (result_mant != 0) & ((result_mant >> 46) == 0) {
        let target_bit: u64 = 46;
        let mut leading_zeros: u64 = 0;
        let mut v = result_mant;
        if v & 0xFFFFFFFF00000000 == 0 {
            leading_zeros += 32;
            v <<= 32;
        }
        if v & 0xFFFF000000000000 == 0 {
            leading_zeros += 16;
            v <<= 16;
        }
        if v & 0xFF00000000000000 == 0 {
            leading_zeros += 8;
            v <<= 8;
        }
        if v & 0xF000000000000000 == 0 {
            leading_zeros += 4;
            v <<= 4;
        }
        if v & 0xC000000000000000 == 0 {
            leading_zeros += 2;
            v <<= 2;
        }
        if v & 0x8000000000000000 == 0 {
            leading_zeros += 1;
        }

        // shift_needed = leading_zeros - (64 - target_bit - 1) = leading_zeros - 17
        let shift_needed = leading_zeros - (64 - target_bit - 1);
        result_mant = result_mant << shift_needed;
        result_exp = result_exp - (shift_needed as i64);
    }

    // Now the implicit bit should be at position 46
    // We need to shift down to position 23, but keep 3 guard bits
    // So shift from position 46 to position 26 (46 - 26 = 20)
    let guard_shift: u64 = 46 - 26;
    result_mant = shift_right_sticky_u64(result_mant, guard_shift);

    // Handle underflow to denormal BEFORE rounding
    // We need to shift the mantissa (which still has 3 guard bits at positions 0-2)
    // into position first, then round
    let mut is_denormal_result = false;
    if result_exp <= 0 {
        // Shift mantissa right to create denormal
        // denorm_shift is how many extra positions we need to shift
        // The mantissa currently has implicit bit at position 26 (23 mantissa + 3 guard)
        let denorm_shift = 1 - result_exp;
        if denorm_shift < 27 {
            // Shift right, preserving sticky bits
            result_mant = shift_right_sticky_u64(result_mant, denorm_shift as u64);
        } else {
            result_mant = 0;
        }
        result_exp = 0;
        is_denormal_result = true;
    }

    // Round to nearest, ties to even
    // Guard bits are in positions 0-2 (guard at bit 2, round at bit 1, sticky at bit 0)
    let guard_bits = result_mant & 0x7;
    result_mant = result_mant >> 3;

    let should_round = if rounding_mode == 0 {
        // NEAREST_EVEN (optimized inline)
        (guard_bits > 4) | ((guard_bits == 4) & ((result_mant & 1) == 1))
    } else {
        should_round_up(guard_bits, result_mant, result_sign, rounding_mode)
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
            // Becomes a normal number with exponent 1
            result_exp = 1;
            is_denormal_result = false;
        }
    }

    // Check for overflow to infinity
    let overflows_to_inf = result_exp >= (FLOAT32_EXPONENT_MAX as i64);

    // Remove implicit bit for normalized results
    let final_mantissa = if is_denormal_result {
        (result_mant & 0x7FFFFF) as u32
    } else {
        (result_mant & ((FLOAT32_IMPLICIT_BIT - 1) as u64)) as u32
    };

    let normal_result =
        IEEE754Float32 { sign: result_sign, exponent: result_exp as u8, mantissa: final_mantissa };

    // Handle special cases
    let mut result = normal_result;

    // Zero result
    if result_is_zero | (result_mant == 0) {
        result = float32_zero(result_sign);
    }

    // Overflow to infinity
    if overflows_to_inf {
        result = float32_infinity(result_sign);
    }

    // Handle zero inputs
    if a_is_zero | b_is_zero {
        // 0 * inf = NaN, 0 * finite = 0
        if a_is_inf | b_is_inf {
            result = float32_nan();
        } else {
            result = float32_zero(result_sign);
        }
    }

    // Handle infinity inputs
    if (a_is_inf | b_is_inf) & !a_is_zero & !b_is_zero {
        if a_is_nan | b_is_nan {
            result = float32_nan();
        } else {
            result = float32_infinity(result_sign);
        }
    }

    // Handle NaN inputs (highest priority)
    if a_is_nan | b_is_nan {
        result = float32_nan();
    }

    result
}

// IEEE 754 compliant multiplication for 32-bit floats (default rounding mode)
// Backward-compatible wrapper that uses round-to-nearest-ties-to-even
pub fn mul_float32(a: IEEE754Float32, b: IEEE754Float32) -> IEEE754Float32 {
    mul_float32_with_rounding(a, b, ROUNDING_MODE_NEAREST_EVEN)
}
