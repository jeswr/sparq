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
fn integer_conversions_are_public_api() {
    let negative_round_down: i32 = -16_777_217;

    assert_eq(f16::from(2051 as u16).bits(), 0x6802 as u16);
    assert_eq(f32::from(negative_round_down).bits(), 0xcb800000 as u32);
    assert_eq(f64::from(9_007_199_254_740_993 as u64).bits(), 0x4340000000000000 as u64);
    assert_eq(f128::from(340282366920938463463374607431768211455 as u128).bits(), 0x407f0000000000000000000000000000 as u128);
}