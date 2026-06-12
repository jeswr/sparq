use sparq_ieee754::{f128, f16, f32, f64};

#[test]
fn generated_float_types_are_public_api() {
    let half: f16 = f16::new(6143);
    let single: f32 = f32::new(1_073_741_823);
    let double: f64 = f64::new(0);
    let quad: f128 = f128::new(0);

    assert_eq(half.bits(), 6143);
    assert_eq(single.bits(), 1_073_741_823);
    assert_eq(double.bits(), 0);
    assert_eq(quad.bits(), 0);
}

#[test]
fn comparison_predicates_are_public_api() {
    let half_one = f16::new(0x3c00 as u16);
    let half_two = f16::new(0x4000 as u16);
    let single_nan = f32::new(0x7fc00000 as u32);
    let double_neg_zero = f64::new(0x8000000000000000 as u64);
    let double_pos_zero = f64::new(0 as u64);
    let quad_one = f128::new(0x3fff0000000000000000000000000000 as u128);

    assert(half_one.lt(half_two));
    assert(half_two.gt(half_one));
    assert(half_one.le(half_one));
    assert(half_one.ge(half_one));
    assert(half_one.ne(half_two));
    assert(single_nan.ne(single_nan));
    assert(double_neg_zero.eq(double_pos_zero));
    assert(quad_one.eq(quad_one));
}

#[test]
fn round_to_integral_methods_are_public_api() {
    let half_one_point_five = f16::new(0x3e00 as u16);
    let single_two_point_five = f32::new(0x40200000 as u32);
    let double_neg_half = f64::new(0xbfe0000000000000 as u64);
    let quad_one_point_five = f128::new(0x3fff8000000000000000000000000000 as u128);

    assert_eq(half_one_point_five.floor().bits(), 0x3c00 as u16);
    assert_eq(half_one_point_five.ceil().bits(), 0x4000 as u16);
    assert_eq(single_two_point_five.round_ties_even().bits(), 0x40000000 as u32);
    assert_eq(double_neg_half.trunc().bits(), 0x8000000000000000 as u64);
    assert_eq(quad_one_point_five.floor().bits(), 0x3fff0000000000000000000000000000 as u128);
}

#[test]
fn float_to_int_casts_are_public_api() {
    let half_max = f16::new(0x7bff as u16);
    let single_five_point_five = f32::new(0x40b00000 as u32);
    let double_neg_three_point_seven_five = f64::new(0xc00e000000000000 as u64);
    let quad_two_pow_63 = f128::new(0x403e0000000000000000000000000000 as u128);

    assert_eq(half_max.to_u64(), 65504 as u64);
    assert_eq(single_five_point_five.to_i64(), 5);
    assert_eq(double_neg_three_point_seven_five.to_i64(), -3);
    assert_eq(quad_two_pow_63.to_u64(), 0x8000000000000000 as u64);
}

#[test]
fn sqrt_is_public_api() {
    let half_four = f16::new(0x4400 as u16);
    let single_two = f32::new(0x40000000 as u32);
    let double_two = f64::new(0x4000000000000000 as u64);
    let quad_four = f128::new(0x40010000000000000000000000000000 as u128);

    assert_eq(half_four.sqrt().bits(), 0x4000 as u16);
    assert_eq(single_two.sqrt().bits(), 0x3fb504f3 as u32);
    assert_eq(double_two.sqrt().bits(), 0x3ff6a09e667f3bcd as u64);
    assert_eq(quad_four.sqrt().bits(), 0x40000000000000000000000000000000 as u128);
}

#[test]
fn integer_conversions_are_public_api() {
    let negative_round_down: i32 = -16_777_217;

    assert_eq(f16::from(2051 as u16).bits(), 0x6802 as u16);
    assert_eq(f32::from(negative_round_down).bits(), 0xcb800000 as u32);
    assert_eq(f64::from(9_007_199_254_740_993 as u64).bits(), 0x4340000000000000 as u64);
    assert_eq(f128::from(340282366920938463463374607431768211455 as u128).bits(), 0x407f0000000000000000000000000000 as u128);
}