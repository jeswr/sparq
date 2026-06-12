// ============================================================================
// Shared Utility Functions for IEEE 754 Operations
// ============================================================================

use crate::types::{
    ROUNDING_MODE_NEAREST_AWAY, ROUNDING_MODE_NEAREST_EVEN, ROUNDING_MODE_TOWARD_NEGATIVE,
    ROUNDING_MODE_TOWARD_POSITIVE, ROUNDING_MODE_TOWARD_ZERO,
};
use std::ops::WrappingAdd;

// Helper to shift right with sticky bit preservation for u64
pub fn shift_right_sticky_u64(value: u64, shift: u64) -> u64 {
    if shift == 0 {
        value
    } else if shift >= 64 {
        if value != 0 {
            1
        } else {
            0
        }
    } else {
        let mask = (1 << shift) - 1;
        let shifted_out = value & mask;
        let result = value >> shift;
        if shifted_out != 0 {
            result | 1
        } else {
            result
        }
    }
}

// Helper for 128-bit multiplication result
pub struct U128 {
    pub high: u64,
    pub low: u64,
}

// Multiply two u64 values and return 128-bit result
pub fn mul_u64_to_u128(a: u64, b: u64) -> U128 {
    // Split into 32-bit parts
    let a_lo = a & 0xFFFFFFFF;
    let a_hi = a >> 32;
    let b_lo = b & 0xFFFFFFFF;
    let b_hi = b >> 32;

    // Partial products (all fit in 64 bits since 32-bit * 32-bit = 64-bit max)
    let p0 = a_lo * b_lo; // a_lo * b_lo
    let p1 = a_lo * b_hi; // a_lo * b_hi
    let p2 = a_hi * b_lo; // a_hi * b_lo
    let p3 = a_hi * b_hi; // a_hi * b_hi

    // The result is: p0 + (p1 << 32) + (p2 << 32) + (p3 << 64)
    // = p0 + ((p1 + p2) << 32) + (p3 << 64)
    //
    // Split into low 64 bits and high 64 bits:
    // low  = (p0) + ((p1 + p2) << 32) mod 2^64
    // high = p3 + ((p1 + p2) >> 32) + carry_from_low

    // Sum p1 + p2 (can be up to 65 bits, so we need to track carry)
    let mid_sum = p1 + p2;
    let mid_carry: u64 = if mid_sum < p1 { 1 } else { 0 }; // This carry goes to bit 96

    // Now compute low = p0 + (mid_sum << 32)
    // This can overflow, so we do it in parts:
    // low = p0[31:0] + p0[63:32]*2^32 + mid_sum[31:0]*2^32 + mid_sum[63:32]*2^64
    // The mid_sum[63:32]*2^64 part goes to high

    // low[31:0] = p0[31:0] (no overflow possible in low 32 bits from mid)
    // low[63:32] = p0[63:32] + mid_sum[31:0] + possible carry from low[31:0]

    // Actually, simpler approach: use wrapping semantics by detecting overflow
    // low = p0 + (mid_sum & 0xFFFFFFFF) << 32
    let mid_lo_shifted = (mid_sum & 0xFFFFFFFF) << 32;

    // Add p0 + mid_lo_shifted using wrapping arithmetic
    // To detect carry: if result < p0 (or result < mid_lo_shifted), there was overflow
    let low = p0.wrapping_add(mid_lo_shifted);
    let low_carry: u64 = if low < p0 { 1 } else { 0 };

    // high = p3 + (mid_sum >> 32) + (mid_carry << 32) + low_carry
    let high = p3 + (mid_sum >> 32) + (mid_carry << 32) + low_carry;

    U128 { high, low }
}

// Shift right a 128-bit value with sticky bit preservation
pub fn shift_right_sticky_u128(val: U128, shift: u64) -> u64 {
    if shift == 0 {
        val.low
    } else if shift >= 128 {
        if (val.high != 0) | (val.low != 0) {
            1
        } else {
            0
        }
    } else if shift >= 64 {
        let effective_shift = shift - 64;
        let sticky = if val.low != 0 { 1 } else { 0 };
        if effective_shift >= 64 {
            if (val.high != 0) | (sticky != 0) {
                1
            } else {
                0
            }
        } else {
            let mask = (1 << effective_shift) - 1;
            let shifted_out = val.high & mask;
            let result = val.high >> effective_shift;
            if (shifted_out != 0) | (sticky != 0) {
                result | 1
            } else {
                result
            }
        }
    } else {
        // shift < 64
        let mask = (1 << shift) - 1;
        let shifted_out = val.low & mask;
        let result_low = (val.low >> shift) | (val.high << (64 - shift));
        if shifted_out != 0 {
            result_low | 1
        } else {
            result_low
        }
    }
}

// Helper for 128-bit division: divides a 128-bit dividend by a 64-bit divisor
// Returns quotient (fits in 64 bits for our use case) and remainder
pub fn div_u128_by_u64(dividend: U128, divisor: u64) -> (u64, u64) {
    // For simplicity, we'll do this with a long division approach
    // This works because our dividend is at most 2^109 and divisor is at least 2^52
    // so the quotient fits in 57 bits (< 64 bits)

    if divisor == 0 {
        (0, 0)
    } else if dividend.high == 0 {
        (dividend.low / divisor, dividend.low % divisor)
    } else {
        // Long division bit by bit
        // Process 128 bits from most significant to least significant
        let mut quotient: u64 = 0;
        let mut remainder: u64 = 0;

        // Process high bits first (64 bits)
        for i in 0..64 {
            let bit_pos = 63 - i;
            remainder = remainder << 1;
            remainder = remainder | ((dividend.high >> (bit_pos as u64)) & 1);

            if remainder >= divisor {
                remainder = remainder - divisor;
                // This bit position in quotient would be 64 + bit_pos, which is >= 64
                // For our use case, we know the quotient fits in 64 bits, so these
                // high bits should all be 0. We can skip adding to quotient here.
            }
        }

        // Process low bits (64 bits)
        for i in 0..64 {
            let bit_pos = 63 - i;
            remainder = remainder << 1;
            remainder = remainder | ((dividend.low >> (bit_pos as u64)) & 1);

            quotient = quotient << 1;
            if remainder >= divisor {
                remainder = remainder - divisor;
                quotient = quotient | 1;
            }
        }

        (quotient, remainder)
    }
}

// ============================================================================
// Rounding Utilities
// ============================================================================

/// Determine if a value should round up based on guard bits and rounding mode
///
/// # Arguments
/// * `guard_bits` - The guard, round, and sticky bits (typically 3 bits: GRS)
/// * `result_mant` - The mantissa after shifting out guard bits
/// * `result_sign` - The sign of the result (0 = positive, 1 = negative)
/// * `mode` - The rounding mode to use (use ROUNDING_MODE_* constants)
///
/// # Returns
/// * `true` if the mantissa should be incremented, `false` otherwise
pub fn should_round_up(guard_bits: u64, result_mant: u64, result_sign: u8, mode: u8) -> bool {
    // Guard bit position varies by operation, but for standard 3-bit guard:
    // Bits 2-0: Guard(2), Round(1), Sticky(0)
    // guard_bits = GRS as 0bGRS (value 0-7)

    let is_positive = result_sign == 0;
    let is_negative = result_sign == 1;

    // Check if we're exactly at the midpoint: guard_bits = 0b100 (G=1, R=0, S=0)
    let at_midpoint = guard_bits == 4;

    // Check if we're above the midpoint (values strictly greater than 0b100)
    let above_midpoint = guard_bits > 4;

    if mode == ROUNDING_MODE_NEAREST_EVEN {
        // Round to nearest, ties to even
        // Round up if: above midpoint OR (at midpoint AND result is odd)
        above_midpoint | (at_midpoint & ((result_mant & 1) == 1))
    } else if mode == ROUNDING_MODE_NEAREST_AWAY {
        // Round to nearest, ties away from zero
        // Round up if: above midpoint OR at midpoint
        above_midpoint | at_midpoint
    } else if mode == ROUNDING_MODE_TOWARD_POSITIVE {
        // Round toward +Infinity (ceiling)
        // Round up if positive and any guard bits are set
        is_positive & (guard_bits != 0)
    } else if mode == ROUNDING_MODE_TOWARD_NEGATIVE {
        // Round toward -Infinity (floor)
        // Round up if negative and any guard bits are set
        // (rounding "up" for negative numbers means toward more negative)
        is_negative & (guard_bits != 0)
    } else {
        // ROUNDING_MODE_TOWARD_ZERO or any other value
        // Round toward zero (truncate)
        // Never round up (always truncate)
        false
    }
}

/// Determine if a value should round up based on guard bits and rounding mode (4-bit version)
/// This version is for operations that use 4 guard bits (like division)
///
/// # Arguments
/// * `guard_bits` - The guard bits (4 bits representing a value 0-15, with midpoint at 8 for rounding)
/// * `result_mant` - The mantissa after shifting out guard bits
/// * `result_sign` - The sign of the result (0 = positive, 1 = negative)
/// * `mode` - The rounding mode to use (use ROUNDING_MODE_* constants)
///
/// # Returns
/// * `true` if the mantissa should be incremented, `false` otherwise
pub fn should_round_up_4bit(guard_bits: u64, result_mant: u64, result_sign: u8, mode: u8) -> bool {
    // For 4-bit guard: Bits 3-0: G(3), R(2), S(1), S(0)
    // guard_bits = GRSS (value 0-15)
    // Midpoint is at 0b1000 = 8

    let is_positive = result_sign == 0;
    let is_negative = result_sign == 1;

    // Check if we're exactly at the midpoint (guard_bits == 8, which is 0b1000)
    let at_midpoint = guard_bits == 8;

    // Check if we're above the midpoint
    let above_midpoint = guard_bits > 8;

    if mode == ROUNDING_MODE_NEAREST_EVEN {
        // Round to nearest, ties to even
        // Round up if: above midpoint OR (at midpoint AND result is odd)
        above_midpoint | (at_midpoint & ((result_mant & 1) == 1))
    } else if mode == ROUNDING_MODE_NEAREST_AWAY {
        // Round to nearest, ties away from zero
        // Round up if: above midpoint OR at midpoint
        above_midpoint | at_midpoint
    } else if mode == ROUNDING_MODE_TOWARD_POSITIVE {
        // Round toward +Infinity (ceiling)
        // Round up if positive and any guard bits are set
        is_positive & (guard_bits != 0)
    } else if mode == ROUNDING_MODE_TOWARD_NEGATIVE {
        // Round toward -Infinity (floor)
        // Round up if negative and any guard bits are set
        is_negative & (guard_bits != 0)
    } else {
        // ROUNDING_MODE_TOWARD_ZERO
        // Never round up (always truncate)
        false
    }
}

/// Determine if a value should round up based on guard bits and rounding mode (8-bit version)
/// This version is for operations that use 8 guard bits (like float64 addition)
///
/// # Arguments
/// * `guard_bits` - The guard bits (8 bits)
/// * `result_mant` - The mantissa after shifting out guard bits
/// * `result_sign` - The sign of the result (0 = positive, 1 = negative)
/// * `mode` - The rounding mode to use (use ROUNDING_MODE_* constants)
///
/// # Returns
/// * `true` if the mantissa should be incremented, `false` otherwise
pub fn should_round_up_8bit(guard_bits: u64, result_mant: u64, result_sign: u8, mode: u8) -> bool {
    // For 8-bit guard: midpoint is at 0x80 = 128

    let is_positive = result_sign == 0;
    let is_negative = result_sign == 1;

    // Check if we're exactly at the midpoint
    let at_midpoint = guard_bits == 0x80;

    // Check if we're above the midpoint
    let above_midpoint = guard_bits > 0x80;

    if mode == ROUNDING_MODE_NEAREST_EVEN {
        // Round to nearest, ties to even
        above_midpoint | (at_midpoint & ((result_mant & 1) == 1))
    } else if mode == ROUNDING_MODE_NEAREST_AWAY {
        // Round to nearest, ties away from zero
        above_midpoint | at_midpoint
    } else if mode == ROUNDING_MODE_TOWARD_POSITIVE {
        // Round toward +Infinity (ceiling)
        is_positive & (guard_bits != 0)
    } else if mode == ROUNDING_MODE_TOWARD_NEGATIVE {
        // Round toward -Infinity (floor)
        is_negative & (guard_bits != 0)
    } else {
        // ROUNDING_MODE_TOWARD_ZERO
        false
    }
}
