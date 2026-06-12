// ============================================================================
// IEEE 754 Float64 Addition
// ============================================================================

use crate::float64::helpers::{
    float64_infinity, float64_is_denormal, float64_is_infinity, float64_is_nan, float64_is_zero,
    float64_nan, float64_zero,
};
use crate::types::{
    FLOAT64_EXPONENT_MAX, FLOAT64_IMPLICIT_BIT, IEEE754Float64, ROUNDING_MODE_NEAREST_EVEN,
};
use crate::utils::should_round_up_8bit;

// IEEE 754 compliant addition for 64-bit doubles with rounding mode
pub fn add_float64_with_rounding(
    a: IEEE754Float64,
    b: IEEE754Float64,
    rounding_mode: u8,
) -> IEEE754Float64 {
    let a_is_nan = float64_is_nan(a);
    let b_is_nan = float64_is_nan(b);
    let a_is_inf = float64_is_infinity(a);
    let b_is_inf = float64_is_infinity(b);
    let a_is_zero = float64_is_zero(a);
    let b_is_zero = float64_is_zero(b);
    let a_is_denormal = float64_is_denormal(a);
    let b_is_denormal = float64_is_denormal(b);

    let mant_a_raw: u64 = if a_is_denormal {
        a.mantissa
    } else {
        a.mantissa | FLOAT64_IMPLICIT_BIT
    };
    let mant_b_raw: u64 = if b_is_denormal {
        b.mantissa
    } else {
        b.mantissa | FLOAT64_IMPLICIT_BIT
    };
    let exp_a_eff: u64 = if a_is_denormal | a_is_zero {
        1
    } else {
        a.exponent as u64
    };
    let exp_b_eff: u64 = if b_is_denormal | b_is_zero {
        1
    } else {
        b.exponent as u64
    };

    let mut mant_a: u64 = mant_a_raw << 8;
    let mut mant_b: u64 = mant_b_raw << 8;
    let mut sticky_a: u64 = 0;
    let mut sticky_b: u64 = 0;
    let mut result_exp: u64 = exp_a_eff;

    if exp_a_eff > exp_b_eff {
        let shift_amount = exp_a_eff - exp_b_eff;
        result_exp = exp_a_eff;
        if shift_amount >= 64 {
            sticky_b = if mant_b != 0 { 1 } else { 0 };
            mant_b = 0;
        } else {
            let mask = (1 << shift_amount) - 1;
            let shifted_out = mant_b & mask;
            mant_b = mant_b >> shift_amount;
            if shifted_out != 0 {
                sticky_b = 1;
            }
        }
    } else if exp_b_eff > exp_a_eff {
        let shift_amount = exp_b_eff - exp_a_eff;
        result_exp = exp_b_eff;
        if shift_amount >= 64 {
            sticky_a = if mant_a != 0 { 1 } else { 0 };
            mant_a = 0;
        } else {
            let mask = (1 << shift_amount) - 1;
            let shifted_out = mant_a & mask;
            mant_a = mant_a >> shift_amount;
            if shifted_out != 0 {
                sticky_a = 1;
            }
        }
    }

    let mut result_mant: u64 = 0;
    let mut result_sticky: u64 = 0;
    let mut result_sign: u8 = 0;

    if a.sign == b.sign {
        result_mant = mant_a + mant_b;
        result_sticky = sticky_a | sticky_b;
        result_sign = a.sign;
        if result_mant < mant_a {
            result_sticky = result_sticky | (result_mant & 1);
            result_mant = (result_mant >> 1) | 0x8000000000000000;
            result_exp = result_exp + 1;
        }
    } else {
        if mant_a > mant_b {
            result_mant = mant_a - mant_b;
            if sticky_b != 0 {
                result_mant = result_mant - 1;
                result_sticky = 1;
            }
            result_sign = a.sign;
        } else if mant_b > mant_a {
            result_mant = mant_b - mant_a;
            if sticky_a != 0 {
                result_mant = result_mant - 1;
                result_sticky = 1;
            }
            result_sign = b.sign;
        } else {
            if sticky_a > sticky_b {
                result_mant = 0;
                result_sticky = 1;
                result_sign = a.sign;
            } else if sticky_b > sticky_a {
                result_mant = 0;
                result_sticky = 1;
                result_sign = b.sign;
            } else {
                result_mant = 0;
                result_sticky = 0;
            }
        }
    }

    // Overflow normalization
    let overflow_threshold: u64 = FLOAT64_IMPLICIT_BIT << 9;
    if result_mant >= overflow_threshold {
        result_sticky = result_sticky | (result_mant & 1);
        result_mant = result_mant >> 1;
        result_exp = result_exp + 1;
    }

    // Underflow normalization using binary search for leading zeros
    let normal_position: u64 = FLOAT64_IMPLICIT_BIT << 8;
    if (result_mant != 0) & (result_mant < normal_position) {
        let target_bit: u64 = 60; // Position of implicit bit after shift by 8
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

        let shift_needed = leading_zeros - (64 - target_bit - 1);
        let max_shift = if result_exp > 1 { result_exp - 1 } else { 0 };
        let actual_shift = if shift_needed <= max_shift {
            shift_needed
        } else {
            max_shift
        };

        result_mant = result_mant << actual_shift;
        result_exp = result_exp - actual_shift;
    }

    // Denormal result
    let mut is_denormal_result = false;
    if (result_exp == 0) | ((result_exp == 1) & (result_mant < normal_position)) {
        if result_exp == 1 {
            result_mant = result_mant >> 1;
        }
        result_exp = 0;
        is_denormal_result = true;
    }

    // Rounding
    let guard_bits = result_mant & 0xFF;
    result_mant = result_mant >> 8;
    let round_bits = guard_bits | result_sticky;
    let should_round = if rounding_mode == 0 {
        // NEAREST_EVEN (optimized inline)
        let halfway: u64 = 0x80;
        (round_bits > halfway) | ((round_bits == halfway) & ((result_mant & 1) == 1))
    } else {
        should_round_up_8bit(round_bits, result_mant, result_sign, rounding_mode)
    };
    if should_round {
        result_mant = result_mant + 1;
        if result_mant >= (FLOAT64_IMPLICIT_BIT << 1) {
            result_mant = result_mant >> 1;
            result_exp = result_exp + 1;
        } else if is_denormal_result & (result_mant >= FLOAT64_IMPLICIT_BIT) {
            result_exp = 1;
        }
    }

    let overflows_to_inf = result_exp >= (FLOAT64_EXPONENT_MAX as u64);
    let final_mantissa = if is_denormal_result & (result_exp == 0) {
        result_mant & 0xFFFFFFFFFFFFF
    } else {
        result_mant & (FLOAT64_IMPLICIT_BIT - 1)
    };
    let normal_result =
        IEEE754Float64 { sign: result_sign, exponent: result_exp as u16, mantissa: final_mantissa };

    let mut result = normal_result;
    if (result_mant == 0) & (result_sticky == 0) {
        result = float64_zero(0);
    }
    if overflows_to_inf {
        result = float64_infinity(result_sign);
    }
    if a_is_zero & b_is_zero {
        result = float64_zero(a.sign & b.sign);
    } else if a_is_zero {
        result = b;
    } else if b_is_zero {
        result = a;
    }
    if a_is_inf & !b_is_inf {
        result = a;
    } else if b_is_inf & !a_is_inf {
        result = b;
    } else if a_is_inf & b_is_inf {
        if a.sign == b.sign {
            result = a;
        } else {
            result = float64_nan();
        }
    }
    if a_is_nan | b_is_nan {
        result = float64_nan();
    }

    result
}

// IEEE 754 compliant addition for 64-bit doubles (default rounding mode)
// Backward-compatible wrapper that uses round-to-nearest-ties-to-even
pub fn add_float64(a: IEEE754Float64, b: IEEE754Float64) -> IEEE754Float64 {
    add_float64_with_rounding(a, b, ROUNDING_MODE_NEAREST_EVEN)
}
