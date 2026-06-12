// ============================================================================
// IEEE 754 Float64 Multiplication
// ============================================================================

use crate::float64::helpers::{
    float64_infinity, float64_is_denormal, float64_is_infinity, float64_is_nan, float64_is_zero,
    float64_nan, float64_zero,
};
use crate::types::{
    FLOAT64_EXPONENT_MAX, FLOAT64_IMPLICIT_BIT, IEEE754Float64, ROUNDING_MODE_NEAREST_EVEN,
};
use crate::utils::{
    mul_u64_to_u128, shift_right_sticky_u128, shift_right_sticky_u64, should_round_up,
};

// IEEE 754 compliant multiplication for 64-bit floats with rounding mode
pub fn mul_float64_with_rounding(
    a: IEEE754Float64,
    b: IEEE754Float64,
    rounding_mode: u8,
) -> IEEE754Float64 {
    // Determine special cases
    let a_is_nan = float64_is_nan(a);
    let b_is_nan = float64_is_nan(b);
    let a_is_inf = float64_is_infinity(a);
    let b_is_inf = float64_is_infinity(b);
    let a_is_zero = float64_is_zero(a);
    let b_is_zero = float64_is_zero(b);
    let a_is_denormal = float64_is_denormal(a);
    let b_is_denormal = float64_is_denormal(b);

    // Result sign is XOR of input signs
    let result_sign: u8 = a.sign ^ b.sign;

    // Get mantissas and normalize denormals
    // For denormals, we need to normalize them to have the implicit bit at position 52
    // and adjust the exponent accordingly

    // Count leading zeros for denormal normalization (52-bit mantissa field)
    // Goal: find shift amount to put highest set bit at position 52 (implicit bit position)
    let a_mant_raw: u64 = a.mantissa;
    let mut a_lz: i64 = 0;
    if a_is_denormal & (a_mant_raw != 0) {
        let mut v = a_mant_raw;
        // Binary search for leading zeros relative to bit 51
        if (v & 0xFFFFFFF000000) == 0 {
            a_lz = a_lz + 28;
            v = v << 28;
        } // bits 24-51
        if (v & 0xFFFC000000000) == 0 {
            a_lz = a_lz + 14;
            v = v << 14;
        } // bits 38-51
        if (v & 0xFE00000000000) == 0 {
            a_lz = a_lz + 7;
            v = v << 7;
        } // bits 45-51
        if (v & 0xF000000000000) == 0 {
            a_lz = a_lz + 4;
            v = v << 4;
        } // bits 48-51
        if (v & 0xC000000000000) == 0 {
            a_lz = a_lz + 2;
            v = v << 2;
        } // bits 50-51
        if (v & 0x8000000000000) == 0 {
            a_lz = a_lz + 1;
        } // bit 51
        // Add 1 to get shift to position 52
        a_lz = a_lz + 1;
    }

    let b_mant_raw: u64 = b.mantissa;
    let mut b_lz: i64 = 0;
    if b_is_denormal & (b_mant_raw != 0) {
        let mut v = b_mant_raw;
        if (v & 0xFFFFFFF000000) == 0 {
            b_lz = b_lz + 28;
            v = v << 28;
        }
        if (v & 0xFFFC000000000) == 0 {
            b_lz = b_lz + 14;
            v = v << 14;
        }
        if (v & 0xFE00000000000) == 0 {
            b_lz = b_lz + 7;
            v = v << 7;
        }
        if (v & 0xF000000000000) == 0 {
            b_lz = b_lz + 4;
            v = v << 4;
        }
        if (v & 0xC000000000000) == 0 {
            b_lz = b_lz + 2;
            v = v << 2;
        }
        if (v & 0x8000000000000) == 0 {
            b_lz = b_lz + 1;
        }
        b_lz = b_lz + 1;
    }

    // Get effective mantissas with implicit bit (53 bits)
    // For denormals, shift to normalize (put highest bit at position 52)
    let mant_a: u64 = if a_is_denormal {
        a_mant_raw << (a_lz as u64)
    } else if a_is_zero {
        0
    } else {
        a.mantissa | FLOAT64_IMPLICIT_BIT
    };

    let mant_b: u64 = if b_is_denormal {
        b_mant_raw << (b_lz as u64)
    } else if b_is_zero {
        0
    } else {
        b.mantissa | FLOAT64_IMPLICIT_BIT
    };

    // Get effective exponents (unbiased), accounting for denormal normalization
    let exp_a_eff: i64 = if a_is_denormal {
        1 - 1023 - a_lz
    } else if a_is_zero {
        0
    } else {
        (a.exponent as i64) - 1023
    };

    let exp_b_eff: i64 = if b_is_denormal {
        1 - 1023 - b_lz
    } else if b_is_zero {
        0
    } else {
        (b.exponent as i64) - 1023
    };

    // Multiply mantissas (53-bit * 53-bit = 106-bit max)
    // For two normalized numbers: 1.xxx * 1.yyy gives result in range [1, 4)
    // The product will have the implicit bit at position 104 or 105
    let product = mul_u64_to_u128(mant_a, mant_b);

    // Check if product is zero
    let result_is_zero = (product.high == 0) & (product.low == 0);

    // Result exponent
    let mut result_exp: i64 = exp_a_eff + exp_b_eff + 1023;

    // For normalized * normalized:
    // mant_a has implicit bit at position 52
    // mant_b has implicit bit at position 52
    // product has implicit bit at position 104 or 105 (105 if >= 2.0)

    // Check if bit 105 is set (product >= 2.0)
    // bit 105 in 128-bit = bit 41 of high part (105 - 64 = 41)
    let bit_105_set = (product.high >> 41) & 1 == 1;

    let mut result_mant: u64 = 0;
    if !result_is_zero {
        if bit_105_set {
            // Product >= 2.0, need to shift extra and increase exponent
            result_exp = result_exp + 1;
            // Shift from position 105 to position 55 (52 + 3 guard bits)
            // That's a shift of 105 - 55 = 50
            result_mant = shift_right_sticky_u128(product, 50);
        } else {
            // Product in [1, 2) for normalized, or lower for denormal inputs
            // First check if leading bit is at position 104
            let bit_104_set = (product.high >> 40) & 1 == 1;

            if bit_104_set {
                // Normal case: implicit bit at position 104
                // Shift from position 104 to position 55
                // That's a shift of 104 - 55 = 49
                result_mant = shift_right_sticky_u128(product, 49);
            } else {
                // Denormal case: leading bit is below position 104
                // Need to find leading bit and normalize
                // First, shift to get the value into a u64 for easier manipulation
                // We know the leading bit is below 104, so it's in bits 0-103 of the 128-bit value

                // Find leading bit position using binary search
                let mut leading_pos: i64 = 0;
                let mut found = false;

                // Check high part first (bits 64-127 of product)
                if product.high != 0 {
                    let mut pos: i64 = 63;
                    let mut v = product.high;
                    if v & 0xFFFFFFFF00000000 != 0 {
                        pos = pos;
                    } else {
                        pos = pos - 32;
                        v <<= 32;
                    }
                    if v & 0xFFFF000000000000 != 0 {
                        pos = pos;
                    } else {
                        pos = pos - 16;
                        v <<= 16;
                    }
                    if v & 0xFF00000000000000 != 0 {
                        pos = pos;
                    } else {
                        pos = pos - 8;
                        v <<= 8;
                    }
                    if v & 0xF000000000000000 != 0 {
                        pos = pos;
                    } else {
                        pos = pos - 4;
                        v <<= 4;
                    }
                    if v & 0xC000000000000000 != 0 {
                        pos = pos;
                    } else {
                        pos = pos - 2;
                        v <<= 2;
                    }
                    if v & 0x8000000000000000 != 0 {
                        pos = pos;
                    } else {
                        pos = pos - 1;
                    }
                    leading_pos = pos + 64; // Position in 128-bit value
                    found = true;
                }

                if !found & (product.low != 0) {
                    let mut pos: i64 = 63;
                    let mut v = product.low;
                    if v & 0xFFFFFFFF00000000 != 0 {
                        pos = pos;
                    } else {
                        pos = pos - 32;
                        v <<= 32;
                    }
                    if v & 0xFFFF000000000000 != 0 {
                        pos = pos;
                    } else {
                        pos = pos - 16;
                        v <<= 16;
                    }
                    if v & 0xFF00000000000000 != 0 {
                        pos = pos;
                    } else {
                        pos = pos - 8;
                        v <<= 8;
                    }
                    if v & 0xF000000000000000 != 0 {
                        pos = pos;
                    } else {
                        pos = pos - 4;
                        v <<= 4;
                    }
                    if v & 0xC000000000000000 != 0 {
                        pos = pos;
                    } else {
                        pos = pos - 2;
                        v <<= 2;
                    }
                    if v & 0x8000000000000000 != 0 {
                        pos = pos;
                    } else {
                        pos = pos - 1;
                    }
                    leading_pos = pos; // Position in 128-bit value (low part)
                    found = true;
                }

                // Adjust exponent based on how far below 104 the leading bit is
                let exp_adjust = 104 - leading_pos;
                result_exp = result_exp - exp_adjust;

                // Shift to position 55 (target position with guard bits)
                // If leading bit is at leading_pos, we want it at position 55
                // So we shift by (leading_pos - 55) if leading_pos > 55, otherwise we'd need to shift left
                let target_pos: i64 = 55;
                if leading_pos >= target_pos {
                    let shift_amount = (leading_pos - target_pos) as u64;
                    result_mant = shift_right_sticky_u128(product, shift_amount);
                } else {
                    // Need to shift left - this means the product is very small
                    // Shift from 128-bit to get all significant bits into result_mant
                    // then shift left
                    let shift_left = (target_pos - leading_pos) as u64;
                    // Get bits from low part and shift left
                    result_mant = product.low << shift_left;
                    // Add bits from high if any overflow into the range
                    if (product.high != 0) & (shift_left < 64) {
                        result_mant = result_mant | (product.high << (shift_left + 64));
                    }
                }
            }
        }
    }

    // Handle underflow to denormal BEFORE rounding
    // The mantissa currently has implicit bit at position 55 (52 mantissa + 3 guard)
    let mut is_denormal_result = false;
    if result_exp <= 0 {
        let denorm_shift = 1 - result_exp;
        if denorm_shift < 56 {
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
        if !is_denormal_result & (result_mant >= (FLOAT64_IMPLICIT_BIT << 1)) {
            result_mant = result_mant >> 1;
            result_exp = result_exp + 1;
        }
        // For denormals, if rounding causes overflow into normal range, adjust
        if is_denormal_result & (result_mant >= FLOAT64_IMPLICIT_BIT) {
            // Becomes a normal number with exponent 1
            result_exp = 1;
            is_denormal_result = false;
        }
    }

    // Check for overflow to infinity
    let overflows_to_inf = result_exp >= (FLOAT64_EXPONENT_MAX as i64);

    // Remove implicit bit for final mantissa
    let final_mantissa = if is_denormal_result {
        result_mant & 0xFFFFFFFFFFFFF
    } else {
        result_mant & (FLOAT64_IMPLICIT_BIT - 1)
    };

    let normal_result =
        IEEE754Float64 { sign: result_sign, exponent: result_exp as u16, mantissa: final_mantissa };

    // Handle special cases
    let mut result = normal_result;

    // Zero result
    if result_is_zero | (result_mant == 0) {
        result = float64_zero(result_sign);
    }

    // Overflow to infinity
    if overflows_to_inf {
        result = float64_infinity(result_sign);
    }

    // Handle zero inputs
    if a_is_zero | b_is_zero {
        if a_is_inf | b_is_inf {
            result = float64_nan();
        } else {
            result = float64_zero(result_sign);
        }
    }

    // Handle infinity inputs
    if (a_is_inf | b_is_inf) & !a_is_zero & !b_is_zero {
        if a_is_nan | b_is_nan {
            result = float64_nan();
        } else {
            result = float64_infinity(result_sign);
        }
    }

    // Handle NaN inputs (highest priority)
    if a_is_nan | b_is_nan {
        result = float64_nan();
    }

    result
}

// IEEE 754 compliant multiplication for 64-bit floats (default rounding mode)
// Backward-compatible wrapper that uses round-to-nearest-ties-to-even
pub fn mul_float64(a: IEEE754Float64, b: IEEE754Float64) -> IEEE754Float64 {
    mul_float64_with_rounding(a, b, ROUNDING_MODE_NEAREST_EVEN)
}
