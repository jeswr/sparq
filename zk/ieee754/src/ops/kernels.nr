use crate::parts::FloatParts;
use std::convert::AsPrimitive;

fn zero_parts(sign: bool) -> FloatParts<u16, u128> {
    FloatParts::new(sign, 0 as u16, 0 as u128)
}

fn exponent_bias(exponent_bits: u8) -> u16 {
    ((1 as u16) << ((exponent_bits - 1) as u16)) - 1
}

fn exponent_offset() -> u16 {
    32768_u16
}

fn hidden_bit() -> u128 {
    hidden_bit_typed::<u128>(112_u8)
}

fn max_exponent_typed<T>(exponent_bits: u8) -> T
where
    T: From<u8> + std::ops::Shl + std::ops::Sub,
{
    (T::from(1_u8) << T::from(exponent_bits)) - T::from(1_u8)
}

fn hidden_bit_typed<T>(mantissa_bits: u8) -> T
where
    T: From<u8> + std::ops::Shl,
{
    T::from(1_u8) << T::from(mantissa_bits)
}

fn recompose_bits_to_field<let BITS: u32>(bits: [bool; BITS]) -> Field {
    let mut result = 0 as Field;
    let mut weight = 1 as Field;

    for index in 0..BITS {
        result = result + (bits[index] as Field) * weight;
        weight = weight * 2;
    }

    result
}

fn pow2_from_bits<let BITS: u32>(bits: [bool; BITS]) -> Field {
    let mut result = 1 as Field;
    let mut factor = 2 as Field;

    for index in 0..BITS {
        result = result * (1 + (bits[index] as Field) * (factor - 1));
        factor = factor * factor;
    }

    result
}

fn const_pow2_field<let SHIFT: u8>() -> Field {
    let mut result = 1 as Field;

    for _step in 0..SHIFT {
        result = result * 2;
    }

    result
}

fn const_pow2_field_u32<let SHIFT: u32>() -> Field {
    let mut result = 1 as Field;

    for _step in 0..SHIFT {
        result = result * 2;
    }

    result
}

unconstrained fn pow2_hint_bits<let BITS: u32>(shift: u16) -> (Field, [bool; BITS]) {
    let mut pow2 = 1 as Field;

    for _step in 0..shift {
        pow2 = pow2 * 2;
    }

    let mut bits = [false; BITS];
    let mut remaining = shift;

    for index in 0..BITS {
        bits[index] = (remaining & 1) == 1;
        remaining = remaining >> 1;
    }

    (pow2, bits)
}

unconstrained fn pow2_pair_hint_bits<let BITS: u32>(shift: u16, total_bits: u16) -> (Field, Field, [bool; BITS]) {
    let (pow2, bits) = pow2_hint_bits::<BITS>(shift);
    let mut complement = 1 as Field;

    for _step in 0..(total_bits - shift) {
        complement = complement * 2;
    }

    (pow2, complement, bits)
}

fn verified_pow2_bits_bounded<let BITS: u32>(shift: u16) -> Field {
    // Safety: the bit recomposition and product-chain checks pin this to 2^shift.
    let (pow2, bits): (Field, [bool; BITS]) = unsafe { pow2_hint_bits::<BITS>(shift) };

    assert(recompose_bits_to_field::<BITS>(bits) == shift as Field);
    assert(pow2 == pow2_from_bits::<BITS>(bits));
    pow2
}

fn verified_pow2_u64_shift_bounded<let POW_BITS: u32>(shift: u16) -> Field {
    if POW_BITS <= 14 {
        verified_pow2_bits_bounded::<4>(shift)
    } else if POW_BITS <= 27 {
        verified_pow2_bits_bounded::<5>(shift)
    } else {
        verified_pow2_bits_bounded::<6>(shift)
    }
}

fn canonical_nan_typed<E, M, let EXPONENT_BITS: u8, let MANTISSA_BITS: u8>() -> FloatParts<E, M>
where
    E: From<u8> + std::ops::Shl + std::ops::Sub,
    M: From<u8> + std::ops::Shl,
{
    FloatParts::new(false, max_exponent_typed::<E>(EXPONENT_BITS), M::from(1_u8) << M::from(MANTISSA_BITS - 1))
}

fn canonical_nan() -> FloatParts<u16, u128> {
    canonical_nan_typed::<u16, u128, 15_u8, 112_u8>()
}

fn infinity_parts_typed<E, M>(sign: bool, exponent_bits: u8) -> FloatParts<E, M>
where
    E: From<u8> + std::ops::Shl + std::ops::Sub,
    M: From<u8>,
{
    FloatParts::new(sign, max_exponent_typed::<E>(exponent_bits), M::from(0_u8))
}

fn infinity_parts(sign: bool) -> FloatParts<u16, u128> {
    infinity_parts_typed::<u16, u128>(sign, 15_u8)
}

fn classify_typed<E, M>(parts: FloatParts<E, M>, exponent_bits: u8) -> (bool, bool, bool)
where
    E: From<u8> + std::ops::Shl + std::ops::Sub + Eq,
    M: From<u8> + Eq,
{
    let exponent_zero = parts.exponent == E::from(0_u8);
    let mantissa_zero = parts.mantissa == M::from(0_u8);
    let special = parts.exponent == max_exponent_typed::<E>(exponent_bits);
    let zero = exponent_zero & mantissa_zero;
    let nan = special & !mantissa_zero;
    let infinite = special & mantissa_zero;

    (nan, infinite, zero)
}

fn classify(parts: FloatParts<u16, u128>) -> (bool, bool, bool) {
    classify_typed::<u16, u128>(parts, 15_u8)
}

fn significand(parts: FloatParts<u16, u128>) -> u128 {
    hidden_bit() * ((parts.exponent != 0) as u128) + parts.mantissa
}

fn work_exponent(parts: FloatParts<u16, u128>) -> u16 {
    let bias = exponent_bias(15_u8);
    let offset = exponent_offset();

    offset + parts.exponent + ((parts.exponent == 0) as u16) - bias
}

fn normalized_finite(parts: FloatParts<u16, u128>) -> (u128, u16) {
    let hidden = hidden_bit();
    let mut result_significand = significand(parts);
    let mut result_exponent = work_exponent(parts);

    if !is_at_least_threshold_u128_bounded::<112>(result_significand, hidden) {
        let (shift, shifted) = left_shift_nonzero_to_threshold_bounded(result_significand, hidden, 112_u16);

        result_significand = shifted;
        result_exponent = result_exponent - shift;
    }

    (result_significand, result_exponent)
}

unconstrained fn shift_right_jam_unconstrained(value: u128, shift: u16) -> (u128, u128) {
    let pow2 = 1 << (shift as u128);
    (value / pow2, value % pow2)
}

fn jam_low_bit(value: u128, sticky: bool) -> u128 {
    value + ((sticky & ((value % 2) == 0)) as u128)
}

fn shift_right_one_jam(value: u128) -> u128 {
    (value / 4) * 2 + ((value % 4) != 0) as u128
}

fn shift_right_jam_nonzero_bounded(value: u128, shift: u16) -> u128 {
    if shift >= 116_u16 {
        1 as u128
    } else {
        // Safety: the reconstruction checks below pin the hinted split.
        let (quotient, remainder): (u128, u128) = unsafe { shift_right_jam_unconstrained(value, shift) };

        // Safety: the decomposition checks pin pow2 = 2^shift and complement = 2^(116-shift).
        let (pow2, pow2_complement, bits): (Field, Field, [bool; 7]) = unsafe { pow2_pair_hint_bits::<7>(shift, 116_u16) };

        assert(recompose_bits_to_field::<7>(bits) == shift as Field);
        assert(pow2 == pow2_from_bits::<7>(bits));
        assert(pow2 * pow2_complement == 0x100000000000000000000000000000 as Field);
        assert(value as Field == (quotient as Field) * pow2 + (remainder as Field));
        ((remainder as Field) * pow2_complement).assert_max_bit_size::<116>();

        jam_low_bit(quotient, remainder != 0)
    }
}

unconstrained fn highest_set_bit_u128_unconstrained<let INPUT_BITS: u32>(value: u128) -> (bool, u16, u128) {
    let mut nonzero = false;
    let mut highest = 0 as u16;
    let mut highest_pow2 = 1 as u128;

    for index in 0..INPUT_BITS {
        let bit_set = ((value >> (index as u128)) & 1) == 1;
        if bit_set {
            nonzero = true;
            highest = index as u16;
            highest_pow2 = (1 as u128) << (index as u128);
        }
    }

    let lower = if nonzero { value - highest_pow2 } else { 0 as u128 };

    (nonzero, highest, lower)
}

fn assert_highest_set_bit_split<let POW_BITS: u32, let INPUT_BITS: u32>(
    value: u128,
    highest: u16,
    lower: u128,
) -> (Field, Field) {
    // Safety: the decomposition checks pin pow2 = 2^highest and complement = 2^(INPUT_BITS-highest).
    let (pow2, pow2_complement, bits): (Field, Field, [bool; POW_BITS]) = unsafe { pow2_pair_hint_bits::<POW_BITS>(highest, INPUT_BITS as u16) };

    assert(recompose_bits_to_field::<POW_BITS>(bits) == highest as Field);
    assert(pow2 == pow2_from_bits::<POW_BITS>(bits));
    assert(pow2 * pow2_complement == const_pow2_field_u32::<INPUT_BITS>());
    assert(value as Field == pow2 + (lower as Field));
    ((lower as Field) * pow2_complement).assert_max_bit_size::<INPUT_BITS>();

    (pow2, pow2_complement)
}

fn highest_set_bit_u128<let INPUT_BITS: u32, let CHECK_RANGE: u8>(value: u128) -> (bool, u16, Field, Field) {
    std::static_assert(
        (INPUT_BITS == 8) | (INPUT_BITS == 16) | (INPUT_BITS == 32) | (INPUT_BITS == 64) | (INPUT_BITS == 128),
        "Unsupported integer conversion input width",
    );
    std::static_assert((CHECK_RANGE == 0) | (CHECK_RANGE == 1), "Unsupported highest-bit range-check flag");

    // Safety: the split proof pins the hinted highest set bit when value is nonzero.
    let (nonzero, highest, lower): (bool, u16, u128) = unsafe { highest_set_bit_u128_unconstrained::<INPUT_BITS>(value) };

    let (highest_pow2, highest_pow2_complement) = if nonzero {
        if CHECK_RANGE == 1 {
            assert(highest < (INPUT_BITS as u16));
        }

        if INPUT_BITS <= 16 {
            assert_highest_set_bit_split::<4, INPUT_BITS>(value, highest, lower)
        } else if INPUT_BITS <= 32 {
            assert_highest_set_bit_split::<5, INPUT_BITS>(value, highest, lower)
        } else if INPUT_BITS <= 64 {
            assert_highest_set_bit_split::<6, INPUT_BITS>(value, highest, lower)
        } else {
            assert_highest_set_bit_split::<7, INPUT_BITS>(value, highest, lower)
        }
    } else {
        assert(value == 0 as u128);

        (1 as Field, const_pow2_field_u32::<INPUT_BITS>())
    };

    (nonzero, highest, highest_pow2, highest_pow2_complement)
}

unconstrained fn shifted_numerator_bit_unconstrained(value: u128, bit_index: u128) -> bool {
    let mut result = false;

    if bit_index >= 115 {
        let shifted_index = bit_index - 115;

        for candidate in 0..128 {
            if shifted_index == (candidate as u128) {
                result = (value & ((1 as u128) << (candidate as u128))) != 0;
            }
        }
    }

    result
}

unconstrained fn left_shift_to_threshold_unconstrained(
    value: u128,
    threshold: u128,
    max_shift: u16,
) -> (u16, u128) {
    let mut shift = 0 as u16;
    let mut shifted = value;

    for _step in 0..max_shift {
        if (shifted != 0) & (shifted < threshold) {
            shifted = shifted * 2;
            shift = shift + 1;
        }
    }

    (shift, shifted)
}

unconstrained fn left_shift_unconstrained(value: u128, shift: u16) -> u128 {
    value << (shift as u128)
}

unconstrained fn shift_to_target_divmod_unconstrained(value: u128, highest: u16, target: u16) -> (u128, u128) {
    if highest > target {
        shift_right_jam_unconstrained(value, highest - target)
    } else {
        (value << (target - highest) as u128, 0)
    }
}

fn shift_to_target_jam_verified<let TARGET: u8, let INPUT_BITS: u32>(
    value: u128,
    highest: u16,
    highest_pow2: Field,
    highest_pow2_complement: Field,
) -> u128 {
    // Safety: the Field split and remainder bound pin the rounded normalization shift.
    let (quotient, remainder): (u128, u128) = unsafe {
        shift_to_target_divmod_unconstrained(value, highest, TARGET as u16)
    };

    let target_pow2 = const_pow2_field::<TARGET>();

    assert((value as Field) * target_pow2 == (quotient as Field) * highest_pow2 + (remainder as Field) * target_pow2);
    ((remainder as Field) * target_pow2 * highest_pow2_complement).assert_max_bit_size::<INPUT_BITS>();

    jam_low_bit(quotient, remainder != 0)
}

fn left_shift_nonzero_to_threshold_bounded(value: u128, threshold: u128, max_shift: u16) -> (u16, u128) {
    // Safety: the pow2 reconstruction and threshold bounds pin the minimal useful shift.
    let (shift, shifted): (u16, u128) = unsafe { left_shift_to_threshold_unconstrained(value, threshold, max_shift) };
    let pow2 = verified_pow2_bits_bounded::<7>(shift);

    assert(shift <= max_shift);
    assert(shifted as Field == (value as Field) * pow2);
    assert(shifted < (threshold << 1));

    if shift < max_shift {
        assert(shifted >= threshold);
    }

    (shift, shifted)
}

fn round_nearest_even(result_significand: u128, hidden: u128) -> (u128, bool) {
    let retained = result_significand / 8;
    let round_bits = result_significand % 8;
    let increment = (round_bits + (retained % 2)) > 4;
    let mut rounded = retained + (increment as u128);
    let rounded_overflow = rounded == (hidden * 2);

    if rounded_overflow {
        rounded = hidden;
    }

    (rounded, rounded_overflow)
}

fn pack_finite_rounded<M>(
    sign: bool,
    result_exponent: u16,
    rounded: M,
    bias: u16,
    offset: u16,
    hidden: M,
    rounded_is_subnormal: bool,
) -> FloatParts<u16, M>
where
    M: From<u8> + std::ops::Sub,
{
    if rounded_is_subnormal {
        FloatParts::new(sign, 0 as u16, rounded)
    } else {
        FloatParts::new(sign, result_exponent + bias - offset, rounded - hidden)
    }
}

fn pack_rounded(
    sign: bool,
    result_exponent: u16,
    rounded: u128,
    maximum_finite_exponent: u16,
    bias: u16,
    offset: u16,
    hidden: u128,
) -> FloatParts<u16, u128> {
    if result_exponent > maximum_finite_exponent {
        infinity_parts(sign)
    } else {
        let rounded_is_subnormal = !is_at_least_threshold_u128_bounded::<112>(rounded, hidden);

        pack_finite_rounded(sign, result_exponent, rounded, bias, offset, hidden, rounded_is_subnormal)
    }
}

fn round_pack_normalized(
    sign: bool,
    exponent: u16,
    raw_significand: u128,
) -> FloatParts<u16, u128> {
    let bias = exponent_bias(15_u8);
    let offset = exponent_offset();
    let minimum_exponent = offset + 1 - bias;
    let maximum_finite_exponent = offset + bias;
    let hidden = hidden_bit();
    let mut result_exponent = exponent;
    let mut result_significand = raw_significand;

    if result_exponent < minimum_exponent {
        result_significand = shift_right_jam_nonzero_bounded(result_significand, minimum_exponent - result_exponent);
        result_exponent = minimum_exponent;
    }

    let (rounded, rounded_overflow) = round_nearest_even(result_significand, hidden);
    result_exponent = result_exponent + (rounded_overflow as u16);

    pack_rounded(sign, result_exponent, rounded, maximum_finite_exponent, bias, offset, hidden)
}

pub(crate) fn uint_to_float_parts_wide<let INPUT_BITS: u32>(sign: bool, magnitude: u128) -> FloatParts<u16, u128> {
    let (nonzero, highest, highest_pow2, highest_pow2_complement) = if INPUT_BITS <= 113 {
        highest_set_bit_u128::<INPUT_BITS, 1_u8>(magnitude)
    } else {
        highest_set_bit_u128::<INPUT_BITS, 0_u8>(magnitude)
    };

    if !nonzero {
        zero_parts(sign)
    } else {
        if INPUT_BITS <= 113 {
            let shift = 112_u16 - highest;
            // Safety: the pow2 reconstruction and product equation pin this exact shift.
            let normalized_significand: u128 = unsafe { left_shift_unconstrained(magnitude, shift) };
            let pow2 = verified_pow2_bits_bounded::<7>(shift);

            assert(normalized_significand as Field == (magnitude as Field) * pow2);

            FloatParts::new(sign, exponent_bias(15_u8) + highest, normalized_significand - hidden_bit())
        } else {
            let raw_significand = shift_to_target_jam_verified::<115_u8, INPUT_BITS>(
                magnitude,
                highest,
                highest_pow2,
                highest_pow2_complement,
            );
            let bias = exponent_bias(15_u8);
            let hidden = hidden_bit();
            let (rounded, rounded_overflow) = round_nearest_even(raw_significand, hidden);
            let result_exponent = highest + bias + (rounded_overflow as u16);

            FloatParts::new(sign, result_exponent, rounded - hidden)
        }
    }
}

unconstrained fn scaled_product_divmod_by_terms(
    left_significand: u128,
    right_significand: u128,
) -> (u128, u128) {
    let limb_mask = 0x00ffffffffffffff as u128;
    let scale = (1 as u128) << 109;
    let left_low = left_significand & limb_mask;
    let left_mid = (left_significand >> 56) & limb_mask;
    let left_high = left_significand >> 112;
    let right_low = right_significand & limb_mask;
    let right_mid = (right_significand >> 56) & limb_mask;
    let right_high = right_significand >> 112;
    let low_product = left_low * right_low;
    let left_low_right_mid = left_low * right_mid;
    let left_mid_right_low = left_mid * right_low;
    let quotient_low = low_product >> 109;
    let quotient_left_low_right_mid = left_low_right_mid >> 53;
    let quotient_left_mid_right_low = left_mid_right_low >> 53;
    let mut result = quotient_low + quotient_left_low_right_mid + quotient_left_mid_right_low;
    let mut remainder = low_product - (quotient_low << 109);

    remainder = remainder + ((left_low_right_mid - (quotient_left_low_right_mid << 53)) << 56);
    if remainder >= scale {
        result = result + 1;
        remainder = remainder - scale;
    }

    remainder = remainder + ((left_mid_right_low - (quotient_left_mid_right_low << 53)) << 56);
    if remainder >= scale {
        result = result + 1;
        remainder = remainder - scale;
    }

    result = result
        + ((left_low * right_high + left_mid * right_mid + left_high * right_low) << 3)
        + ((left_mid * right_high + left_high * right_mid) << 59)
        + (left_high * right_high << 115);

    (result, remainder)
}

unconstrained fn at_least_threshold_u128_hint(value: u128, threshold: u128) -> bool {
    value >= threshold
}

fn is_at_least_threshold_u128_bounded<let LOW_BITS: u32>(value: u128, threshold: u128) -> bool {
    // Safety: the low-part bound pins the hinted threshold side because value < 2 * threshold.
    let at_least: bool = unsafe { at_least_threshold_u128_hint(value, threshold) };
    let low = (value as Field) - (at_least as Field) * (threshold as Field);

    low.assert_max_bit_size::<LOW_BITS>();

    at_least
}

unconstrained fn less_than_u128_hint(left: u128, right: u128) -> bool {
    left < right
}

fn less_than_u128_bounded<let DIFF_BITS: u32>(left: u128, right: u128) -> bool {
    // Safety: the bounded difference pins the hinted order for values below the caller's bit bound.
    let left_less: bool = unsafe { less_than_u128_hint(left, right) };
    let diff = if left_less {
        (right as Field) - (left as Field) - 1
    } else {
        (left as Field) - (right as Field)
    };

    diff.assert_max_bit_size::<DIFF_BITS>();

    left_less
}

unconstrained fn remainder_complement_unconstrained(remainder: u128, divisor: u128) -> u128 {
    if (divisor != 0) & (remainder < divisor) {
        divisor - 1 - remainder
    } else {
        0 as u128
    }
}

unconstrained fn scaled_quotient_long_unconstrained(
    numerator: u128,
    denominator: u128,
) -> (u128, u128) {
    let mut quotient = 0 as u128;
    let mut remainder = 0 as u128;

    if denominator != 0 {
        for high_step in 0..128 {
            let bit_index = (255 - high_step) as u128;
            remainder = remainder << 1;

            if shifted_numerator_bit_unconstrained(numerator, bit_index) {
                remainder = remainder + 1;
            }

            if remainder >= denominator {
                remainder = remainder - denominator;
            }
        }

        for low_step in 0..128 {
            let bit_index = (127 - low_step) as u128;
            remainder = remainder << 1;

            if shifted_numerator_bit_unconstrained(numerator, bit_index) {
                remainder = remainder + 1;
            }

            if remainder >= denominator {
                remainder = remainder - denominator;
                quotient = quotient + ((1 as u128) << bit_index);
            }
        }
    }

    (quotient, remainder)
}

pub(crate) fn add_wide(
    left: FloatParts<u16, u128>,
    right: FloatParts<u16, u128>,
) -> FloatParts<u16, u128> {
    let (left_nan, left_infinite, left_zero) = classify(left);
    let (right_nan, right_infinite, right_zero) = classify(right);

    if left_nan | right_nan | (left_infinite & right_infinite & (left.sign != right.sign)) {
        canonical_nan()
    } else if left_zero & right_zero {
        zero_parts(left.sign & right.sign)
    } else if left_infinite {
        left
    } else if right_infinite {
        right
    } else {
        let left_exponent = work_exponent(left);
        let right_exponent = work_exponent(right);
        let left_significand = significand(left) * 8;
        let right_significand = significand(right) * 8;
        let left_larger = (left_exponent > right_exponent)
            | ((left_exponent == right_exponent) & !less_than_u128_bounded::<116>(left_significand, right_significand));
        let large_significand = if left_larger { left_significand } else { right_significand };
        let small_significand = if left_larger { right_significand } else { left_significand };
        let large_exponent = if left_larger { left_exponent } else { right_exponent };
        let small_exponent = if left_larger { right_exponent } else { left_exponent };
        let result_sign = if left_larger { left.sign } else { right.sign };
        let signs_match = left.sign == right.sign;
        let exponent_delta = large_exponent - small_exponent;
        let aligned_small = if exponent_delta == 0 {
            small_significand
        } else {
            shift_right_jam_nonzero_bounded(small_significand, exponent_delta)
        };
        let result_significand = large_significand + 2 * (signs_match as u128) * aligned_small - aligned_small;

        if result_significand == 0 {
            zero_parts(false)
        } else {
            let bias = exponent_bias(15_u8);
            let offset = exponent_offset();
            let minimum_exponent = offset + 1 - bias;
            let maximum_finite_exponent = offset + bias;
            let hidden = hidden_bit();
            let hidden_with_round_bits = hidden * 8;
            let mut result_exponent = large_exponent;
            let mut rounded_significand = result_significand;

            if is_at_least_threshold_u128_bounded::<116>(rounded_significand, hidden_with_round_bits * 2) {
                rounded_significand = shift_right_one_jam(rounded_significand);
                result_exponent = result_exponent + 1;
            }

            if (!is_at_least_threshold_u128_bounded::<115>(rounded_significand, hidden_with_round_bits)) & (result_exponent > minimum_exponent)
            {
                let exponent_room = result_exponent - minimum_exponent;
                let (shift, shifted) = left_shift_nonzero_to_threshold_bounded(rounded_significand, hidden_with_round_bits, exponent_room);

                rounded_significand = shifted;
                result_exponent = result_exponent - shift;
            }

            if result_exponent < minimum_exponent {
                rounded_significand = shift_right_jam_nonzero_bounded(rounded_significand, minimum_exponent - result_exponent);
                result_exponent = minimum_exponent;
            }

            let (rounded, rounded_overflow) = round_nearest_even(rounded_significand, hidden);
            result_exponent = result_exponent + (rounded_overflow as u16);

            pack_rounded(result_sign, result_exponent, rounded, maximum_finite_exponent, bias, offset, hidden)
        }
    }
}

pub(crate) fn sub_wide(
    left: FloatParts<u16, u128>,
    right: FloatParts<u16, u128>,
) -> FloatParts<u16, u128> {
    let negated_right = FloatParts::new(!right.sign, right.exponent, right.mantissa);

    add_wide(left, negated_right)
}

pub(crate) fn mul_wide(
    left: FloatParts<u16, u128>,
    right: FloatParts<u16, u128>,
) -> FloatParts<u16, u128>
{
    let result_sign = left.sign != right.sign;
    let (left_nan, left_infinite, left_zero) = classify(left);
    let (right_nan, right_infinite, right_zero) = classify(right);
    let invalid = left_nan | right_nan | (left_infinite & right_zero) | (right_infinite & left_zero);

    if invalid {
        canonical_nan()
    } else if left_infinite | right_infinite {
        infinity_parts(result_sign)
    } else if left_zero | right_zero {
        zero_parts(result_sign)
    } else {
        let (left_significand, left_exponent) = normalized_finite(left);
        let (right_significand, right_exponent) = normalized_finite(right);
        let mut result_exponent = ((left_exponent as u32) + (right_exponent as u32) - (exponent_offset() as u32)) as u16;
        let divisor = (1 as u128) << 109;
        let product = (left_significand as Field) * (right_significand as Field);
        // Safety: the product reconstruction and strict-remainder complement pin the split.
        let (quotient, remainder): (u128, u128) = unsafe {
            scaled_product_divmod_by_terms(left_significand, right_significand)
        };

        assert(product == (quotient as Field) * (divisor as Field) + (remainder as Field));
        (remainder as Field).assert_max_bit_size::<109>();

        let mut result_significand = jam_low_bit(quotient, remainder != 0);
        let hidden_with_round_bits = hidden_bit() * 8;

        if is_at_least_threshold_u128_bounded::<116>(result_significand, hidden_with_round_bits * 2) {
            result_significand = shift_right_one_jam(result_significand);
            result_exponent = result_exponent + 1;
        }

        round_pack_normalized(result_sign, result_exponent, result_significand)
    }
}

pub(crate) fn div_wide(
    left: FloatParts<u16, u128>,
    right: FloatParts<u16, u128>,
) -> FloatParts<u16, u128> {
    let result_sign = left.sign != right.sign;
    let (left_nan, left_infinite, left_zero) = classify(left);
    let (right_nan, right_infinite, right_zero) = classify(right);
    let invalid = left_nan | right_nan | (left_infinite & right_infinite) | (left_zero & right_zero);

    if invalid {
        canonical_nan()
    } else if left_infinite | right_zero {
        infinity_parts(result_sign)
    } else if right_infinite | left_zero {
        zero_parts(result_sign)
    } else {
        let (mut left_significand, mut result_exponent) = normalized_finite(left);
        let (right_significand, right_exponent) = normalized_finite(right);
        result_exponent = ((result_exponent as u32) + (exponent_offset() as u32) - (right_exponent as u32)) as u16;

        if less_than_u128_bounded::<112>(left_significand, right_significand) {
            left_significand = left_significand * 2;
            result_exponent = result_exponent - 1;
        }

        // Safety: reconstruction and the strict-remainder complement pin the unique split.
        let (quotient, remainder): (u128, u128) = unsafe {
            scaled_quotient_long_unconstrained(left_significand, right_significand)
        };
        let pow2 = const_pow2_field::<115_u8>();

        assert((left_significand as Field) * pow2 == (quotient as Field) * (right_significand as Field) + (remainder as Field));

        // Safety: a valid u128 complement exists exactly when remainder < divisor.
        let complement = unsafe { remainder_complement_unconstrained(remainder, right_significand) };

        assert((remainder as Field) + (complement as Field) + 1 == right_significand as Field);

        let result_significand = jam_low_bit(quotient, remainder != 0);

        round_pack_normalized(result_sign, result_exponent, result_significand)
    }
}

fn hidden_bit_u64(mantissa_bits: u8) -> u64 {
    hidden_bit_typed::<u64>(mantissa_bits)
}

fn zero_parts_u64(sign: bool) -> FloatParts<u16, u64> {
    FloatParts::new(sign, 0 as u16, 0 as u64)
}

fn canonical_nan_u64<let EXPONENT_BITS: u8, let MANTISSA_BITS: u8>() -> FloatParts<u16, u64> {
    canonical_nan_typed::<u16, u64, EXPONENT_BITS, MANTISSA_BITS>()
}

fn infinity_parts_u64<let EXPONENT_BITS: u8>(sign: bool) -> FloatParts<u16, u64> {
    infinity_parts_typed::<u16, u64>(sign, EXPONENT_BITS)
}

fn classify_u64<let EXPONENT_BITS: u8>(parts: FloatParts<u16, u64>) -> (bool, bool, bool) {
    classify_typed::<u16, u64>(parts, EXPONENT_BITS)
}

fn significand_u64<let MANTISSA_BITS: u8>(parts: FloatParts<u16, u64>) -> u64 {
    if MANTISSA_BITS == 52 {
        hidden_bit_u64(MANTISSA_BITS) * ((parts.exponent != 0) as u64) + parts.mantissa
    } else if parts.exponent == 0 {
        parts.mantissa
    } else {
        hidden_bit_u64(MANTISSA_BITS) + parts.mantissa
    }
}

fn work_exponent_u64<let EXPONENT_BITS: u8>(parts: FloatParts<u16, u64>) -> u16 {
    let bias = exponent_bias(EXPONENT_BITS);
    let offset = exponent_offset();

    if EXPONENT_BITS == 11 {
        offset + parts.exponent + ((parts.exponent == 0) as u16) - bias
    } else if parts.exponent == 0 {
        offset + 1 - bias
    } else {
        offset + parts.exponent - bias
    }
}

unconstrained fn shift_right_jam_u64_unconstrained(value: u64, shift: u16) -> (u64, u64) {
    let mut pow2 = 1 as u64;

    for _step in 0..shift {
        pow2 = pow2 * 2;
    }

    let quotient = value / pow2;
    let remainder = value - quotient * pow2;

    (quotient, remainder)
}

fn jam_low_bit_u64(value: u64, sticky: bool) -> u64 {
    value + ((sticky & ((value % 2) == 0)) as u64)
}

unconstrained fn at_least_threshold_u64_hint(value: u64, threshold: u64) -> bool {
    value >= threshold
}

fn is_at_least_threshold_u64_bounded<let LOW_BITS: u32>(value: u64, threshold: u64) -> bool {
    // Safety: the low-part bound pins the hinted threshold side because value < 2 * threshold.
    let at_least: bool = unsafe { at_least_threshold_u64_hint(value, threshold) };
    let low = (value as Field) - (at_least as Field) * (threshold as Field);

    low.assert_max_bit_size::<LOW_BITS>();

    at_least
}

fn is_at_least_hidden_threshold_u64_f32(value: u64, threshold: u64) -> bool {
    is_at_least_threshold_u64_bounded::<23>(value, threshold)
}

fn assert_shift_right_jam_u64_split<let MAX_VALUE_BITS: u32>(
    value: u64,
    quotient: u64,
    remainder: u64,
    pow2: Field,
    pow2_complement: Field,
) {
    std::static_assert(
        (MAX_VALUE_BITS == 14) | (MAX_VALUE_BITS == 27) | (MAX_VALUE_BITS == 56),
        "Unsupported bounded u64 shift width",
    );
    let pow2_limit = if MAX_VALUE_BITS == 14 {
        0x4000 as Field
    } else if MAX_VALUE_BITS == 27 {
        0x8000000 as Field
    } else {
        0x100000000000000 as Field
    };

    assert(pow2 * pow2_complement == pow2_limit);
    assert(value as Field == (quotient as Field) * pow2 + (remainder as Field));
    ((remainder as Field) * pow2_complement).assert_max_bit_size::<MAX_VALUE_BITS>();
}

fn shift_right_jam_u64_nonzero_bounded<let MAX_VALUE_BITS: u32>(value: u64, shift: u16) -> u64 {
    if shift >= (MAX_VALUE_BITS as u16) {
        1 as u64
    } else {
        // Safety: the reconstruction checks below pin the hinted split.
        let (quotient, remainder): (u64, u64) = unsafe { shift_right_jam_u64_unconstrained(value, shift) };

        if MAX_VALUE_BITS <= 14 {
            // Safety: the decomposition checks pin pow2 = 2^shift and complement = 2^(MAX_VALUE_BITS-shift).
            let (pow2, pow2_complement, bits): (Field, Field, [bool; 4]) = unsafe { pow2_pair_hint_bits::<4>(shift, MAX_VALUE_BITS as u16) };

            assert(recompose_bits_to_field::<4>(bits) == shift as Field);
            assert(pow2 == pow2_from_bits::<4>(bits));
            assert_shift_right_jam_u64_split::<MAX_VALUE_BITS>(value, quotient, remainder, pow2, pow2_complement);
        } else if MAX_VALUE_BITS <= 27 {
            // Safety: the decomposition checks pin pow2 = 2^shift and complement = 2^(MAX_VALUE_BITS-shift).
            let (pow2, pow2_complement, bits): (Field, Field, [bool; 5]) = unsafe { pow2_pair_hint_bits::<5>(shift, MAX_VALUE_BITS as u16) };

            assert(recompose_bits_to_field::<5>(bits) == shift as Field);
            assert(pow2 == pow2_from_bits::<5>(bits));
            assert_shift_right_jam_u64_split::<MAX_VALUE_BITS>(value, quotient, remainder, pow2, pow2_complement);
        } else {
            // Safety: the decomposition checks pin pow2 = 2^shift and complement = 2^(MAX_VALUE_BITS-shift).
            let (pow2, pow2_complement, bits): (Field, Field, [bool; 6]) = unsafe { pow2_pair_hint_bits::<6>(shift, MAX_VALUE_BITS as u16) };

            assert(recompose_bits_to_field::<6>(bits) == shift as Field);
            assert(pow2 == pow2_from_bits::<6>(bits));
            assert_shift_right_jam_u64_split::<MAX_VALUE_BITS>(value, quotient, remainder, pow2, pow2_complement);
        }

        if MAX_VALUE_BITS == 27 {
            jam_low_bit_u64(quotient, remainder != 0)
        } else {
            quotient | ((remainder != 0) as u64)
        }
    }
}

fn shift_right_jam_u64_alignment<let MANTISSA_BITS: u8>(value: u64, shift: u16) -> u64 {
    if MANTISSA_BITS == 10 {
        shift_right_jam_u64_nonzero_bounded::<14>(value, shift)
    } else if MANTISSA_BITS == 23 {
        shift_right_jam_u64_nonzero_bounded::<27>(value, shift)
    } else {
        shift_right_jam_u64_nonzero_bounded::<56>(value, shift)
    }
}

unconstrained fn left_shift_to_threshold_u64_unconstrained(value: u64, threshold: u64, max_shift: u16) -> (u16, u64) {
    let mut shift = 0 as u16;
    let mut shifted = value;

    for _step in 0..max_shift {
        if (shifted != 0) & (shifted < threshold) {
            shifted = shifted * 2;
            shift = shift + 1;
        }
    }

    (shift, shifted)
}

unconstrained fn left_shift_u64_unconstrained(value: u64, shift: u16) -> u64 {
    let mut shifted = value;

    for _step in 0..shift {
        shifted = shifted * 2;
    }

    shifted
}

fn left_shift_exact_u64_bounded<let POW_BITS: u32>(value: u64, shift: u16) -> u64 {
    // Safety: the pow2 reconstruction and product equation pin this exact shift.
    let shifted: u64 = unsafe { left_shift_u64_unconstrained(value, shift) };
    let pow2 = verified_pow2_u64_shift_bounded::<POW_BITS>(shift);

    assert(shifted as Field == (value as Field) * pow2);

    shifted
}

fn left_shift_nonzero_to_threshold_u64_bounded<let POW_BITS: u32>(value: u64, threshold: u64, max_shift: u16) -> (u16, u64) {
    // Safety: the pow2 reconstruction and threshold bounds pin the minimal useful shift.
    let (shift, shifted): (u16, u64) = unsafe { left_shift_to_threshold_u64_unconstrained(value, threshold, max_shift) };
    let pow2 = verified_pow2_u64_shift_bounded::<POW_BITS>(shift);

    assert(shift <= max_shift);
    assert(shifted as Field == (value as Field) * pow2);
    assert(shifted < (threshold << 1));

    if shift < max_shift {
        assert(shifted >= threshold);
    }

    (shift, shifted)
}

fn normalized_finite_u64<let EXPONENT_BITS: u8, let MANTISSA_BITS: u8>(parts: FloatParts<u16, u64>) -> (u64, u16) {
    let hidden = hidden_bit_u64(MANTISSA_BITS);
    let mut result_significand = significand_u64::<MANTISSA_BITS>(parts);
    let mut result_exponent = work_exponent_u64::<EXPONENT_BITS>(parts);

    let needs_hidden_shift = if MANTISSA_BITS == 23 {
        !is_at_least_hidden_threshold_u64_f32(result_significand, hidden)
    } else {
        result_significand < hidden
    };

    if needs_hidden_shift {
        let (shift, shifted) = if MANTISSA_BITS == 10 {
            left_shift_nonzero_to_threshold_u64_bounded::<11>(result_significand, hidden, MANTISSA_BITS as u16)
        } else if MANTISSA_BITS == 23 {
            left_shift_nonzero_to_threshold_u64_bounded::<24>(result_significand, hidden, MANTISSA_BITS as u16)
        } else {
            left_shift_nonzero_to_threshold_u64_bounded::<53>(result_significand, hidden, MANTISSA_BITS as u16)
        };

        result_significand = shifted;
        result_exponent = result_exponent - shift;
    }

    (result_significand, result_exponent)
}

fn round_nearest_even_u64(result_significand: u64, hidden: u64) -> (u64, bool) {
    let retained = result_significand / 8;
    let round_bits = result_significand % 8;
    let increment = (round_bits + (retained % 2)) > 4;
    let mut rounded = retained + (increment as u64);
    let rounded_overflow = rounded == (hidden * 2);

    if rounded_overflow {
        rounded = hidden;
    }

    (rounded, rounded_overflow)
}

fn pack_rounded_u64<let EXPONENT_BITS: u8, let MANTISSA_BITS: u8>(
    sign: bool,
    result_exponent: u16,
    rounded: u64,
    maximum_finite_exponent: u16,
    bias: u16,
    offset: u16,
    hidden: u64,
) -> FloatParts<u16, u64> {
    if result_exponent > maximum_finite_exponent {
        infinity_parts_u64::<EXPONENT_BITS>(sign)
    } else {
        let rounded_is_subnormal = if MANTISSA_BITS == 23 {
            !is_at_least_hidden_threshold_u64_f32(rounded, hidden)
        } else {
            rounded < hidden
        };

        pack_finite_rounded(sign, result_exponent, rounded, bias, offset, hidden, rounded_is_subnormal)
    }
}

fn round_pack_normalized_u64<let EXPONENT_BITS: u8, let MANTISSA_BITS: u8>(
    sign: bool,
    exponent: u16,
    raw_significand: u64,
) -> FloatParts<u16, u64> {
    let bias = exponent_bias(EXPONENT_BITS);
    let offset = exponent_offset();
    let minimum_exponent = offset + 1 - bias;
    let maximum_finite_exponent = offset + bias;
    let hidden = hidden_bit_u64(MANTISSA_BITS);
    let mut result_exponent = exponent;
    let mut result_significand = raw_significand;

    if result_exponent < minimum_exponent {
        result_significand = shift_right_jam_u64_alignment::<MANTISSA_BITS>(result_significand, minimum_exponent - result_exponent);
        result_exponent = minimum_exponent;
    }

    let (rounded, rounded_overflow) = round_nearest_even_u64(result_significand, hidden);
    result_exponent = result_exponent + (rounded_overflow as u16);

    pack_rounded_u64::<EXPONENT_BITS, MANTISSA_BITS>(sign, result_exponent, rounded, maximum_finite_exponent, bias, offset, hidden)
}

pub(crate) fn uint_to_float_parts_u64<let EXPONENT_BITS: u8, let MANTISSA_BITS: u8, let INPUT_BITS: u32>(
    sign: bool,
    magnitude: u128,
) -> FloatParts<u16, u64> {
    let (nonzero, highest, highest_pow2, highest_pow2_complement) = if INPUT_BITS <= ((MANTISSA_BITS as u32) + 1) {
        highest_set_bit_u128::<INPUT_BITS, 1_u8>(magnitude)
    } else {
        highest_set_bit_u128::<INPUT_BITS, 0_u8>(magnitude)
    };

    if !nonzero {
        zero_parts_u64(sign)
    } else {
        if INPUT_BITS <= ((MANTISSA_BITS as u32) + 1) {
            let shift = (MANTISSA_BITS as u16) - highest;
            let normalized_significand = if MANTISSA_BITS == 10 {
                left_shift_exact_u64_bounded::<14>(magnitude as u64, shift)
            } else if MANTISSA_BITS == 23 {
                left_shift_exact_u64_bounded::<27>(magnitude as u64, shift)
            } else {
                left_shift_exact_u64_bounded::<56>(magnitude as u64, shift)
            };

            FloatParts::new(sign, exponent_bias(EXPONENT_BITS) + highest, normalized_significand - hidden_bit_u64(MANTISSA_BITS))
        } else {
            let raw_significand = if MANTISSA_BITS == 10 {
                shift_to_target_jam_verified::<13_u8, INPUT_BITS>(
                    magnitude,
                    highest,
                    highest_pow2,
                    highest_pow2_complement,
                ) as u64
            } else if MANTISSA_BITS == 23 {
                shift_to_target_jam_verified::<26_u8, INPUT_BITS>(
                    magnitude,
                    highest,
                    highest_pow2,
                    highest_pow2_complement,
                ) as u64
            } else {
                shift_to_target_jam_verified::<55_u8, INPUT_BITS>(
                    magnitude,
                    highest,
                    highest_pow2,
                    highest_pow2_complement,
                ) as u64
            };
            let bias = exponent_bias(EXPONENT_BITS);
            let hidden = hidden_bit_u64(MANTISSA_BITS);
            let retained = raw_significand / 8;
            let round_bits = raw_significand % 8;
            let increment = (round_bits + (retained % 2)) > 4;
            let rounded = retained + (increment as u64);
            let rounded_overflow = rounded == (hidden * 2);
            let mantissa = if rounded_overflow { 0 as u64 } else { rounded - hidden };
            let result_exponent = highest + bias + (rounded_overflow as u16);

            if ((EXPONENT_BITS == 5) | ((EXPONENT_BITS == 8) & (INPUT_BITS == 128))) & (result_exponent > (bias * 2)) {
                infinity_parts_u64::<EXPONENT_BITS>(sign)
            } else {
                FloatParts::new(sign, result_exponent, mantissa)
            }
        }
    }
}

unconstrained fn product_overflow_hint(product: Field, threshold: Field) -> bool {
    !product.lt(threshold)
}

unconstrained fn field_divmod_u64_unconstrained(dividend: Field, divisor: u64) -> (u64, u64) {
    let dividend_u128 = dividend as u128;
    let divisor_u128 = divisor as u128;

    if divisor == 0 {
        (0 as u64, 0 as u64)
    } else {
        let quotient = dividend_u128 / divisor_u128;
        let remainder = dividend_u128 - quotient * divisor_u128;

        (quotient as u64, remainder as u64)
    }
}

unconstrained fn scaled_quotient_u64_unconstrained(
    numerator: u64,
    denominator: u64,
    shift: u8,
) -> (u64, u64) {
    if denominator == 0 {
        (0 as u64, 0 as u64)
    } else {
        let mut pow2 = 1 as u128;

        for _step in 0..shift {
            pow2 = pow2 * 2;
        }

        let dividend = (numerator as u128) * pow2;
        let quotient = dividend / (denominator as u128);
        let remainder = dividend - quotient * (denominator as u128);

        (quotient as u64, remainder as u64)
    }
}

pub(crate) fn add<E, M, let EXPONENT_BITS: u8, let MANTISSA_BITS: u8>(
    left: FloatParts<E, M>,
    right: FloatParts<E, M>,
) -> FloatParts<E, M>
where
    E: Into<u16>,
    M: Into<u64>,
    u16: AsPrimitive<E>,
    u64: AsPrimitive<M>,
{
    from_u64_work_parts(add_u64_work::<EXPONENT_BITS, MANTISSA_BITS>(to_u64_work_parts(left), to_u64_work_parts(right)))
}

fn add_u64_work<let EXPONENT_BITS: u8, let MANTISSA_BITS: u8>(
    left: FloatParts<u16, u64>,
    right: FloatParts<u16, u64>,
) -> FloatParts<u16, u64> {
    let (left_nan, left_infinite, left_zero) = classify_u64::<EXPONENT_BITS>(left);
    let (right_nan, right_infinite, right_zero) = classify_u64::<EXPONENT_BITS>(right);

    if left_nan | right_nan {
        canonical_nan_u64::<EXPONENT_BITS, MANTISSA_BITS>()
    } else if left_infinite & right_infinite & (left.sign != right.sign) {
        canonical_nan_u64::<EXPONENT_BITS, MANTISSA_BITS>()
    } else if left_zero & right_zero {
        zero_parts_u64(left.sign & right.sign)
    } else if left_infinite {
        left
    } else if right_infinite {
        right
    } else {
        let left_exponent = work_exponent_u64::<EXPONENT_BITS>(left);
        let right_exponent = work_exponent_u64::<EXPONENT_BITS>(right);
        let left_significand = significand_u64::<MANTISSA_BITS>(left) * 8;
        let right_significand = significand_u64::<MANTISSA_BITS>(right) * 8;
        let left_larger = (left_exponent > right_exponent)
            | ((left_exponent == right_exponent) & (left_significand >= right_significand));
        let large_significand = if left_larger { left_significand } else { right_significand };
        let small_significand = if left_larger { right_significand } else { left_significand };
        let large_exponent = if left_larger { left_exponent } else { right_exponent };
        let small_exponent = if left_larger { right_exponent } else { left_exponent };
        let result_sign = if left_larger { left.sign } else { right.sign };
        let signs_match = left.sign == right.sign;
        let exponent_delta = large_exponent - small_exponent;
        let aligned_small = if exponent_delta == 0 {
            small_significand
        } else {
            shift_right_jam_u64_alignment::<MANTISSA_BITS>(small_significand, exponent_delta)
        };
        let result_significand = large_significand + 2 * (signs_match as u64) * aligned_small - aligned_small;

        if result_significand == 0 {
            zero_parts_u64(false)
        } else {
            let bias = exponent_bias(EXPONENT_BITS);
            let offset = exponent_offset();
            let minimum_exponent = offset + 1 - bias;
            let maximum_finite_exponent = offset + bias;
            let hidden = hidden_bit_u64(MANTISSA_BITS);
            let hidden_with_round_bits = hidden * 8;
            let mut result_exponent = large_exponent;
            let mut rounded_significand = result_significand;

            let needs_shift = if MANTISSA_BITS == 52 {
                rounded_significand >= (hidden_with_round_bits * 2)
            } else {
                if MANTISSA_BITS == 10 {
                    is_at_least_threshold_u64_bounded::<14>(rounded_significand, hidden_with_round_bits * 2)
                } else if MANTISSA_BITS == 23 {
                    is_at_least_threshold_u64_bounded::<27>(rounded_significand, hidden_with_round_bits * 2)
                } else {
                    is_at_least_threshold_u64_bounded::<56>(rounded_significand, hidden_with_round_bits * 2)
                }
            };

            if needs_shift {
                rounded_significand = (rounded_significand / 4) * 2 + ((rounded_significand % 4) != 0) as u64;
                result_exponent = result_exponent + 1;
            }

            let needs_left_shift = if MANTISSA_BITS == 23 {
                !is_at_least_threshold_u64_bounded::<26>(rounded_significand, hidden_with_round_bits)
            } else {
                rounded_significand < hidden_with_round_bits
            };

            if needs_left_shift & (result_exponent > minimum_exponent)
            {
                let exponent_room = result_exponent - minimum_exponent;
                let (shift, shifted) = if MANTISSA_BITS == 10 {
                    left_shift_nonzero_to_threshold_u64_bounded::<14>(rounded_significand, hidden_with_round_bits, exponent_room)
                } else if MANTISSA_BITS == 23 {
                    left_shift_nonzero_to_threshold_u64_bounded::<27>(rounded_significand, hidden_with_round_bits, exponent_room)
                } else {
                    left_shift_nonzero_to_threshold_u64_bounded::<56>(rounded_significand, hidden_with_round_bits, exponent_room)
                };

                rounded_significand = shifted;
                result_exponent = result_exponent - shift;
            }

            if result_exponent < minimum_exponent {
                rounded_significand = shift_right_jam_u64_alignment::<MANTISSA_BITS>(rounded_significand, minimum_exponent - result_exponent);
                result_exponent = minimum_exponent;
            }

            let (rounded, rounded_overflow) = round_nearest_even_u64(rounded_significand, hidden);
            result_exponent = result_exponent + (rounded_overflow as u16);

            pack_rounded_u64::<EXPONENT_BITS, MANTISSA_BITS>(result_sign, result_exponent, rounded, maximum_finite_exponent, bias, offset, hidden)
        }
    }
}

pub(crate) fn sub<E, M, let EXPONENT_BITS: u8, let MANTISSA_BITS: u8>(
    left: FloatParts<E, M>,
    right: FloatParts<E, M>,
) -> FloatParts<E, M>
where
    E: Into<u16>,
    M: Into<u64>,
    u16: AsPrimitive<E>,
    u64: AsPrimitive<M>,
{
    let left_work = to_u64_work_parts(left);
    let right_work = to_u64_work_parts(right);
    let negated_right = FloatParts::new(!right_work.sign, right_work.exponent, right_work.mantissa);

    from_u64_work_parts(add_u64_work::<EXPONENT_BITS, MANTISSA_BITS>(left_work, negated_right))
}

pub(crate) fn mul<E, M, let EXPONENT_BITS: u8, let MANTISSA_BITS: u8>(
    left: FloatParts<E, M>,
    right: FloatParts<E, M>,
) -> FloatParts<E, M>
where
    E: Into<u16>,
    M: Into<u64>,
    u16: AsPrimitive<E>,
    u64: AsPrimitive<M>,
{
    let left_work = to_u64_work_parts(left);
    let right_work = to_u64_work_parts(right);
    let result_sign = left_work.sign != right_work.sign;
    let (left_nan, left_infinite, left_zero) = classify_u64::<EXPONENT_BITS>(left_work);
    let (right_nan, right_infinite, right_zero) = classify_u64::<EXPONENT_BITS>(right_work);
    let invalid = left_nan | right_nan | (left_infinite & right_zero) | (right_infinite & left_zero);

    let result = if invalid {
        canonical_nan_u64::<EXPONENT_BITS, MANTISSA_BITS>()
    } else if left_infinite | right_infinite {
        infinity_parts_u64::<EXPONENT_BITS>(result_sign)
    } else if left_zero | right_zero {
        zero_parts_u64(result_sign)
    } else {
        let (left_significand, left_exponent) = normalized_finite_u64::<EXPONENT_BITS, MANTISSA_BITS>(left_work);
        let (right_significand, right_exponent) = normalized_finite_u64::<EXPONENT_BITS, MANTISSA_BITS>(right_work);
        let mut result_exponent = ((left_exponent as u32) + (right_exponent as u32) - (exponent_offset() as u32)) as u16;
        let hidden = hidden_bit_u64(MANTISSA_BITS);
        let product = (left_significand as Field) * (right_significand as Field);
        let overflow_threshold = (hidden as Field) * (hidden as Field) * 2;
        // Safety: the low bound below pins this to product >= overflow_threshold.
        let product_overflow: bool = unsafe { product_overflow_hint(product, overflow_threshold) };
        let product_overflow_field = product_overflow as Field;
        let low = product - product_overflow_field * overflow_threshold;

        if MANTISSA_BITS == 10 {
            low.assert_max_bit_size::<21>();
        } else if MANTISSA_BITS == 23 {
            low.assert_max_bit_size::<47>();
        } else {
            low.assert_max_bit_size::<105>();
        }

        let scaled = product * (8 - 4 * product_overflow_field);
        let divisor = hidden + hidden;
        // Safety: reconstruction and the power-of-two remainder bound pin the unique split.
        let (quotient_high, combined_remainder): (u64, u64) = unsafe { field_divmod_u64_unconstrained(scaled, divisor) };

        assert(scaled == (quotient_high as Field) * (divisor as Field) + (combined_remainder as Field));
        if MANTISSA_BITS == 10 {
            (combined_remainder as Field).assert_max_bit_size::<11>();
        } else if MANTISSA_BITS == 23 {
            (combined_remainder as Field).assert_max_bit_size::<24>();
        } else {
            (combined_remainder as Field).assert_max_bit_size::<53>();
        }

        let result_significand = quotient_high * 2 + ((combined_remainder != 0) as u64);

        result_exponent = result_exponent + product_overflow as u16;

        round_pack_normalized_u64::<EXPONENT_BITS, MANTISSA_BITS>(result_sign, result_exponent, result_significand)
    };

    from_u64_work_parts(result)
}

pub(crate) fn div<E, M, let EXPONENT_BITS: u8, let MANTISSA_BITS: u8>(
    left: FloatParts<E, M>,
    right: FloatParts<E, M>,
) -> FloatParts<E, M>
where
    E: Into<u16>,
    M: Into<u64>,
    u16: AsPrimitive<E>,
    u64: AsPrimitive<M>,
{
    let left_work = to_u64_work_parts(left);
    let right_work = to_u64_work_parts(right);
    let result_sign = left_work.sign != right_work.sign;
    let (left_nan, left_infinite, left_zero) = classify_u64::<EXPONENT_BITS>(left_work);
    let (right_nan, right_infinite, right_zero) = classify_u64::<EXPONENT_BITS>(right_work);
    let invalid = left_nan | right_nan | (left_infinite & right_infinite) | (left_zero & right_zero);

    let result = if invalid {
        canonical_nan_u64::<EXPONENT_BITS, MANTISSA_BITS>()
    } else if left_infinite | right_zero {
        infinity_parts_u64::<EXPONENT_BITS>(result_sign)
    } else if right_infinite | left_zero {
        zero_parts_u64(result_sign)
    } else {
        let (mut left_significand, mut result_exponent) = normalized_finite_u64::<EXPONENT_BITS, MANTISSA_BITS>(left_work);
        let (right_significand, right_exponent) = normalized_finite_u64::<EXPONENT_BITS, MANTISSA_BITS>(right_work);
        result_exponent = ((result_exponent as u32) + (exponent_offset() as u32) - (right_exponent as u32)) as u16;

        let needs_shift = left_significand < right_significand;
        left_significand = left_significand * (1 + (needs_shift as u64));
        result_exponent = result_exponent - (needs_shift as u16);

        // Safety: reconstruction and typed remainder comparison pin the unique split.
        let (quotient, remainder): (u64, u64) = unsafe {
            scaled_quotient_u64_unconstrained(left_significand, right_significand, MANTISSA_BITS + 3)
        };
        let pow2 = if MANTISSA_BITS == 10 {
            const_pow2_field::<13_u8>()
        } else if MANTISSA_BITS == 23 {
            const_pow2_field::<26_u8>()
        } else {
            const_pow2_field::<55_u8>()
        };

        assert((left_significand as Field) * pow2 == (quotient as Field) * (right_significand as Field) + (remainder as Field));
        assert(remainder < right_significand);

        let result_significand = if MANTISSA_BITS == 23 {
            jam_low_bit_u64(quotient, remainder != 0)
        } else {
            quotient | ((remainder != 0) as u64)
        };

        round_pack_normalized_u64::<EXPONENT_BITS, MANTISSA_BITS>(result_sign, result_exponent, result_significand)
    };

    from_u64_work_parts(result)
}

fn to_u64_work_parts<E, M>(parts: FloatParts<E, M>) -> FloatParts<u16, u64>
where
    E: Into<u16>,
    M: Into<u64>,
{
    FloatParts::new(parts.sign, parts.exponent.into(), parts.mantissa.into())
}

pub(crate) fn from_u64_work_parts<E, M>(parts: FloatParts<u16, u64>) -> FloatParts<E, M>
where
    u16: AsPrimitive<E>,
    u64: AsPrimitive<M>,
{
    FloatParts::new(parts.sign, parts.exponent.as_(), parts.mantissa.as_())
}
