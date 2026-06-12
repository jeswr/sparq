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
fn integer_conversions_are_public_api() {
    let negative_round_down: i32 = -16_777_217;

    assert_eq(f16::from(2051 as u16).bits(), 0x6802 as u16);
    assert_eq(f32::from(negative_round_down).bits(), 0xcb800000 as u32);
    assert_eq(f64::from(9_007_199_254_740_993 as u64).bits(), 0x4340000000000000 as u64);
    assert_eq(f128::from(340282366920938463463374607431768211455 as u128).bits(), 0x407f0000000000000000000000000000 as u128);
}