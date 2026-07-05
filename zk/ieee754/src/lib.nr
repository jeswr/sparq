pub(crate) mod codegen;
pub(crate) mod ops;
pub(crate) mod parts;
pub(crate) mod sizing;

comptime fn generate_float_type(item: TypeDefinition, bits: u8) -> Quoted {
	codegen::generate_float_type(item, bits)
}

#[generate_float_type(16)]
#[allow(dead_code)]
struct GenerateF16 {}

#[generate_float_type(32)]
#[allow(dead_code)]
struct GenerateF32 {}

#[generate_float_type(64)]
#[allow(dead_code)]
struct GenerateF64 {}

#[generate_float_type(128)]
#[allow(dead_code)]
struct GenerateF128 {}

#[test]
fn generated_float_types_are_available_in_global_namespace() {
	let half: f16 = f16::new(6143 as u16);
	let single: f32 = f32::new(1_073_741_823 as u32);

	assert_eq(half.sign, false);
	assert_eq(half.exponent, 5 as u8);
	assert_eq(half.mantissa, 1023 as u16);
	assert_eq(half.bits(), 6143 as u16);
	assert_eq(single.sign, false);
	assert_eq(single.exponent, 127 as u8);
	assert_eq(single.mantissa, 8_388_607 as u32);
	assert_eq(single.bits(), 1_073_741_823 as u32);

	let double: f64 = f64::new(0 as u64);
	let quad: f128 = f128::new(0 as u128);

	assert_eq(double.bits(), 0 as u64);
	assert_eq(quad.bits(), 0 as u128);
}

#[test]
fn generated_float_types_round_trip_through_parts() {
	let parts: parts::FloatParts<u8, u16> = parts::FloatParts::new(true, 5 as u8, 1023 as u16);
	let half: f16 = f16::from_parts(parts);
	let round_trip: parts::FloatParts<u8, u16> = half.to_parts();

	assert_eq(round_trip.sign, true);
	assert_eq(round_trip.exponent, 5 as u8);
	assert_eq(round_trip.mantissa, 1023 as u16);
}

#[test]
fn arithmetic_traits_are_wired_to_ops() {
	let zero = f16::new(0x0000 as u16);
	let one = f16::new(0x3c00 as u16);
	let two = f16::new(0x4000 as u16);
	let three = f16::new(0x4200 as u16);
	let one_point_five = f16::new(0x3e00 as u16);

	assert_eq((one + two).bits(), 0x4200 as u16);
	assert_eq((two - one).bits(), 0x3c00 as u16);
	assert_eq((one - two).bits(), 0xbc00 as u16);
	assert_eq((one + f16::new(0x1000 as u16)).bits(), 0x3c00 as u16);
	assert_eq((one + f16::new(0x1600 as u16)).bits(), 0x3c02 as u16);
	assert_eq((two * two).bits(), 0x4400 as u16);
	assert_eq((one_point_five * one_point_five).bits(), 0x4080 as u16);
	assert_eq((two / two).bits(), 0x3c00 as u16);
	assert_eq((one / three).bits(), 0x3555 as u16);
	assert_eq((zero + two).bits(), 0x4000 as u16);
	assert_eq((two * zero).bits(), 0x0000 as u16);
}

#[test]
fn arithmetic_handles_subnormals_and_special_values() {
	let positive_zero = f16::new(0x0000 as u16);
	let negative_zero = f16::new(0x8000 as u16);
	let smallest_subnormal = f16::new(0x0001 as u16);
	let largest_subnormal = f16::new(0x03ff as u16);
	let one = f16::new(0x3c00 as u16);
	let positive_infinity = f16::new(0x7c00 as u16);
	let negative_infinity = f16::new(0xfc00 as u16);

	assert_eq((smallest_subnormal + smallest_subnormal).bits(), 0x0002 as u16);
	assert_eq((largest_subnormal + smallest_subnormal).bits(), 0x0400 as u16);
	assert_eq((negative_zero + negative_zero).bits(), 0x8000 as u16);
	assert_eq((positive_zero + negative_zero).bits(), 0x0000 as u16);
	assert_eq((one / positive_zero).bits(), 0x7c00 as u16);
	assert_eq((positive_infinity + negative_infinity).bits(), 0x7e00 as u16);
	assert_eq((positive_infinity * positive_zero).bits(), 0x7e00 as u16);
}

#[test]
fn arithmetic_scales_across_generated_float_widths() {
	let single_one = f32::new(0x3f800000 as u32);
	let single_two = f32::new(0x40000000 as u32);
	let single_three = f32::new(0x40400000 as u32);
	let single_one_point_five = f32::new(0x3fc00000 as u32);

	assert_eq((single_one + single_two).bits(), 0x40400000 as u32);
	assert_eq((single_one / single_three).bits(), 0x3eaaaaab as u32);
	assert_eq((single_one_point_five * single_one_point_five).bits(), 0x40100000 as u32);

	let double_one = f64::new(0x3ff0000000000000 as u64);
	let double_two = f64::new(0x4000000000000000 as u64);
	let double_three = f64::new(0x4008000000000000 as u64);

	assert_eq((double_one + double_two).bits(), 0x4008000000000000 as u64);
	assert_eq((double_one / double_three).bits(), 0x3fd5555555555555 as u64);

	let quad_one = f128::new(0x3fff0000000000000000000000000000 as u128);
	let quad_two = f128::new(0x40000000000000000000000000000000 as u128);

	assert_eq((quad_one + quad_two).bits(), 0x40008000000000000000000000000000 as u128);
	assert_eq((quad_two / quad_two).bits(), 0x3fff0000000000000000000000000000 as u128);
}

// Regression: binary128 mul/div on deep-underflow inputs must return a
// correctly-signed zero per IEEE 754-2019 section 7.5, NOT abort the circuit on
// an unsigned exponent underflow (sq-25mgo). Oracle: exact-rational binary128
// soft-float (round-nearest-even), cross-checked with an exact-fraction script -
// a power-of-two / clearly-below-half-ULP product rounds to +/-0 with the XOR sign.
//   smallest_subnormal = 2^-16494  (0x...0001)
//   max_finite         = (2 - 2^-112) * 2^16383  (exponent 0x7ffe, mantissa all-ones)
#[test]
fn f128_mul_deep_underflow_returns_signed_zero() {
	let smallest_subnormal = f128::new(0x00000000000000000000000000000001 as u128);
	let neg_smallest_subnormal = f128::new(0x80000000000000000000000000000001 as u128);

	// smallest_subnormal^2 = 2^-32988 << half-ULP(2^-16494) -> +0 (repro case 1).
	assert_eq((smallest_subnormal * smallest_subnormal).bits(), 0x00000000000000000000000000000000 as u128);
	// Sign is the XOR of operand signs even when the magnitude underflows to zero.
	assert_eq((smallest_subnormal * neg_smallest_subnormal).bits(), 0x80000000000000000000000000000000 as u128);
	assert_eq((neg_smallest_subnormal * smallest_subnormal).bits(), 0x80000000000000000000000000000000 as u128);
	assert_eq((neg_smallest_subnormal * neg_smallest_subnormal).bits(), 0x00000000000000000000000000000000 as u128);
}

#[test]
fn f128_div_deep_underflow_returns_signed_zero() {
	let min_sub = f128::new(0x00000000000000000000000000000001 as u128);
	let neg_min_sub = f128::new(0x80000000000000000000000000000001 as u128);
	let max_finite = f128::new(0x7ffeffffffffffffffffffffffffffff as u128);
	let neg_max_finite = f128::new(0xfffeffffffffffffffffffffffffffff as u128);

	// min_sub / max_finite = 2^-16494 / ~2^16384 = 2^-32878 << half-ULP -> +0 (repro case 2).
	assert_eq((min_sub / max_finite).bits(), 0x00000000000000000000000000000000 as u128);
	assert_eq((neg_min_sub / max_finite).bits(), 0x80000000000000000000000000000000 as u128);
	assert_eq((min_sub / neg_max_finite).bits(), 0x80000000000000000000000000000000 as u128);
	assert_eq((neg_min_sub / neg_max_finite).bits(), 0x00000000000000000000000000000000 as u128);
}

// Anti-over-saturation guard: results that are genuine subnormals must stay
// subnormal, not be flushed to zero by the underflow-safe exponent clamp.
// Oracle (exact rational): 2^-16382 * 2^-112 = 2^-16494 = smallest subnormal;
// 2^-16382 / 2 = 2^-16383 = subnormal with mantissa 2^111.
#[test]
fn f128_arithmetic_preserves_gradual_underflow_subnormals() {
	let smallest_normal = f128::new(0x00010000000000000000000000000000 as u128); // 2^-16382
	let pow2_neg_112 = f128::new(0x3f8f0000000000000000000000000000 as u128);     // 2^-112
	let two = f128::new(0x40000000000000000000000000000000 as u128);
	let half = f128::new(0x3ffe0000000000000000000000000000 as u128);            // 2^-1

	// Multiplying down to the exact smallest subnormal (one ULP above zero).
	assert_eq((smallest_normal * pow2_neg_112).bits(), 0x00000000000000000000000000000001 as u128);
	// Dividing into the subnormal range (2^-16383, mantissa bit 2^111 set).
	assert_eq((smallest_normal / two).bits(), 0x00008000000000000000000000000000 as u128);
	assert_eq((smallest_normal * half).bits(), 0x00008000000000000000000000000000 as u128);
}

#[test]
fn unsigned_integer_conversions_follow_ieee_rounding() {
	assert_eq(f16::from(0 as u8).bits(), 0x0000 as u16);
	assert_eq(f16::from(1 as u8).bits(), 0x3c00 as u16);
	assert_eq(f16::from(2049 as u16).bits(), 0x6800 as u16);
	assert_eq(f16::from(2051 as u16).bits(), 0x6802 as u16);
	assert_eq(f16::from(65519 as u16).bits(), 0x7bff as u16);
	assert_eq(f16::from(65520 as u16).bits(), 0x7c00 as u16);

	assert_eq(f32::from(16_777_217 as u32).bits(), 0x4b800000 as u32);
	assert_eq(f32::from(16_777_219 as u32).bits(), 0x4b800002 as u32);
	assert_eq(f32::from(4_294_967_295 as u32).bits(), 0x4f800000 as u32);

	assert_eq(f64::from(9_007_199_254_740_993 as u64).bits(), 0x4340000000000000 as u64);
	assert_eq(f64::from(9_007_199_254_740_995 as u64).bits(), 0x4340000000000002 as u64);

	assert_eq(f128::from(10384593717069655257060992658440193 as u128).bits(), 0x40700000000000000000000000000000 as u128);
	assert_eq(f128::from(10384593717069655257060992658440195 as u128).bits(), 0x40700000000000000000000000000002 as u128);
	assert_eq(f128::from(340282366920938463463374607431768211455 as u128).bits(), 0x407f0000000000000000000000000000 as u128);
}

#[test]
fn signed_integer_conversions_preserve_sign_and_rounding() {
	let minus_one: i8 = -1;
	let min_i8: i8 = -128;
	let negative_round_down: i32 = -16_777_217;
	let negative_u64_round_down: i64 = -9_007_199_254_740_993;
	let almost_min_i64: i64 = -9_223_372_036_854_775_807;
	let min_i64 = almost_min_i64 - 1;

	assert_eq(f16::from(minus_one).bits(), 0xbc00 as u16);
	assert_eq(f16::from(min_i8).bits(), 0xd800 as u16);
	assert_eq(f32::from(negative_round_down).bits(), 0xcb800000 as u32);
	assert_eq(f64::from(negative_u64_round_down).bits(), 0xc340000000000000 as u64);
	assert_eq(f64::from(min_i64).bits(), 0xc3e0000000000000 as u64);
}

#[test]
fn comparison_predicates_follow_ieee_semantics() {
	let one = f16::new(0x3c00 as u16);
	let two = f16::new(0x4000 as u16);
	let neg_one = f16::new(0xbc00 as u16);
	let neg_two = f16::new(0xc000 as u16);
	let pos_zero = f16::new(0x0000 as u16);
	let neg_zero = f16::new(0x8000 as u16);
	let nan = f16::new(0x7e00 as u16);
	let pos_inf = f16::new(0x7c00 as u16);
	let neg_inf = f16::new(0xfc00 as u16);
	let min_sub = f16::new(0x0001 as u16);

	// Ordinary total ordering on non-NaN values.
	assert(one.lt(two));
	assert(!two.lt(one));
	assert(one.le(two));
	assert(one.le(one));
	assert(two.gt(one));
	assert(two.ge(two));
	assert(one.eq(one));
	assert(!one.eq(two));
	assert(one.ne(two));
	assert(!one.ne(one));

	// Sign handling, including negative magnitude reversal.
	assert(neg_two.lt(neg_one));
	assert(!neg_one.lt(neg_two));
	assert(neg_one.lt(one));
	assert(neg_zero.lt(min_sub));
	assert(neg_inf.lt(neg_two));
	assert(two.lt(pos_inf));
	assert(pos_inf.eq(pos_inf));
	assert(neg_inf.lt(pos_inf));

	// -0 == +0 in every predicate.
	assert(pos_zero.eq(neg_zero));
	assert(neg_zero.eq(pos_zero));
	assert(!pos_zero.lt(neg_zero));
	assert(!neg_zero.lt(pos_zero));
	assert(pos_zero.le(neg_zero));
	assert(neg_zero.ge(pos_zero));
	assert(!pos_zero.ne(neg_zero));

	// NaN is unordered against everything, including itself.
	assert(!nan.eq(nan));
	assert(nan.ne(nan));
	assert(!nan.lt(one));
	assert(!one.lt(nan));
	assert(!nan.le(nan));
	assert(!nan.gt(one));
	assert(!nan.ge(one));
	assert(!pos_inf.lt(nan));
	assert(nan.ne(one));
}

#[test]
fn comparison_predicates_scale_across_widths() {
	let single_one = f32::new(0x3f800000 as u32);
	let single_one_next = f32::new(0x3f800001 as u32);
	let single_nan = f32::new(0x7fc00000 as u32);

	assert(single_one.lt(single_one_next));
	assert(single_one_next.gt(single_one));
	assert(!single_nan.eq(single_nan));

	let double_neg_zero = f64::new(0x8000000000000000 as u64);
	let double_pos_zero = f64::new(0x0000000000000000 as u64);
	let double_min_sub = f64::new(0x0000000000000001 as u64);

	assert(double_neg_zero.eq(double_pos_zero));
	assert(double_neg_zero.lt(double_min_sub));
	assert(double_min_sub.gt(double_pos_zero));

	let quad_one = f128::new(0x3fff0000000000000000000000000000 as u128);
	let quad_one_next = f128::new(0x3fff0000000000000000000000000001 as u128);
	let quad_neg_one = f128::new(0xbfff0000000000000000000000000000 as u128);
	let quad_nan = f128::new(0x7fff8000000000000000000000000000 as u128);

	assert(quad_one.lt(quad_one_next));
	assert(quad_neg_one.lt(quad_one));
	assert(quad_one.ge(quad_one));
	assert(!quad_nan.le(quad_one));
	assert(quad_nan.ne(quad_nan));
}

#[test]
fn round_to_integral_handles_modes_and_signs() {
	let one_point_five = f16::new(0x3e00 as u16);
	let neg_one_point_five = f16::new(0xbe00 as u16);
	let two_point_five = f16::new(0x4100 as u16);
	let half = f16::new(0x3800 as u16);
	let neg_half = f16::new(0xb800 as u16);
	let quarter = f16::new(0x3400 as u16);
	let one_next = f16::new(0x3c01 as u16);

	// floor / ceil / trunc / ties-even on 1.5 and -1.5.
	assert_eq(one_point_five.floor().bits(), 0x3c00 as u16);
	assert_eq(one_point_five.ceil().bits(), 0x4000 as u16);
	assert_eq(one_point_five.trunc().bits(), 0x3c00 as u16);
	assert_eq(one_point_five.round_ties_even().bits(), 0x4000 as u16);
	assert_eq(neg_one_point_five.floor().bits(), 0xc000 as u16);
	assert_eq(neg_one_point_five.ceil().bits(), 0xbc00 as u16);
	assert_eq(neg_one_point_five.trunc().bits(), 0xbc00 as u16);
	assert_eq(neg_one_point_five.round_ties_even().bits(), 0xc000 as u16);

	// Ties to even: 2.5 -> 2, 0.5 -> 0, -0.5 -> -0.
	assert_eq(two_point_five.round_ties_even().bits(), 0x4000 as u16);
	assert_eq(half.round_ties_even().bits(), 0x0000 as u16);
	assert_eq(neg_half.round_ties_even().bits(), 0x8000 as u16);
	assert_eq(half.ceil().bits(), 0x3c00 as u16);
	assert_eq(half.floor().bits(), 0x0000 as u16);
	assert_eq(neg_half.floor().bits(), 0xbc00 as u16);
	assert_eq(neg_half.ceil().bits(), 0x8000 as u16);
	assert_eq(neg_half.trunc().bits(), 0x8000 as u16);

	// |x| < 0.5 and a value just above one.
	assert_eq(quarter.round_ties_even().bits(), 0x0000 as u16);
	assert_eq(one_next.floor().bits(), 0x3c00 as u16);
	assert_eq(one_next.ceil().bits(), 0x4000 as u16);
	assert_eq(one_next.round_ties_even().bits(), 0x3c00 as u16);

	// 1023.5 sits exactly between 1023 (odd) and 1024 (even).
	let near_max_half = f16::new(0x63ff as u16);

	assert_eq(near_max_half.round_ties_even().bits(), 0x6400 as u16);
	assert_eq(near_max_half.floor().bits(), 0x63fe as u16);
	assert_eq(near_max_half.ceil().bits(), 0x6400 as u16);
}

#[test]
fn round_to_integral_handles_specials_and_subnormals() {
	let nan = f16::new(0x7e01 as u16);
	let pos_inf = f16::new(0x7c00 as u16);
	let neg_inf = f16::new(0xfc00 as u16);
	let pos_zero = f16::new(0x0000 as u16);
	let neg_zero = f16::new(0x8000 as u16);
	let min_sub = f16::new(0x0001 as u16);
	let neg_max_sub = f16::new(0x83ff as u16);
	let big_integral = f16::new(0x6800 as u16);

	assert_eq(nan.floor().bits(), 0x7e00 as u16);
	assert_eq(nan.round_ties_even().bits(), 0x7e00 as u16);
	assert_eq(pos_inf.ceil().bits(), 0x7c00 as u16);
	assert_eq(neg_inf.floor().bits(), 0xfc00 as u16);
	assert_eq(pos_zero.floor().bits(), 0x0000 as u16);
	assert_eq(neg_zero.ceil().bits(), 0x8000 as u16);
	assert_eq(min_sub.floor().bits(), 0x0000 as u16);
	assert_eq(min_sub.ceil().bits(), 0x3c00 as u16);
	assert_eq(min_sub.trunc().bits(), 0x0000 as u16);
	assert_eq(min_sub.round_ties_even().bits(), 0x0000 as u16);
	assert_eq(neg_max_sub.floor().bits(), 0xbc00 as u16);
	assert_eq(neg_max_sub.ceil().bits(), 0x8000 as u16);
	assert_eq(big_integral.floor().bits(), 0x6800 as u16);
	assert_eq(big_integral.round_ties_even().bits(), 0x6800 as u16);
}

#[test]
fn round_to_integral_scales_across_widths() {
	let single_two_point_five = f32::new(0x40200000 as u32);
	let single_neg_small = f32::new(0xbe99999a as u32);

	assert_eq(single_two_point_five.round_ties_even().bits(), 0x40000000 as u32);
	assert_eq(single_two_point_five.ceil().bits(), 0x40400000 as u32);
	assert_eq(single_neg_small.ceil().bits(), 0x80000000 as u32);
	assert_eq(single_neg_small.floor().bits(), 0xbf800000 as u32);

	let double_one_point_five = f64::new(0x3ff8000000000000 as u64);
	let double_neg_two_point_five = f64::new(0xc004000000000000 as u64);
	let double_max_half = f64::new(0x432fffffffffffff as u64);

	assert_eq(double_one_point_five.floor().bits(), 0x3ff0000000000000 as u64);
	assert_eq(double_one_point_five.round_ties_even().bits(), 0x4000000000000000 as u64);
	assert_eq(double_neg_two_point_five.round_ties_even().bits(), 0xc000000000000000 as u64);
	assert_eq(double_neg_two_point_five.floor().bits(), 0xc008000000000000 as u64);
	assert_eq(double_neg_two_point_five.ceil().bits(), 0xc000000000000000 as u64);
	// 2^52 - 0.5 ties to 2^52.
	assert_eq(double_max_half.round_ties_even().bits(), 0x4330000000000000 as u64);
	assert_eq(double_max_half.trunc().bits(), 0x432ffffffffffffe as u64);

	let quad_one_point_five = f128::new(0x3fff8000000000000000000000000000 as u128);
	let quad_neg_half = f128::new(0xbffe0000000000000000000000000000 as u128);

	assert_eq(quad_one_point_five.floor().bits(), 0x3fff0000000000000000000000000000 as u128);
	assert_eq(quad_one_point_five.round_ties_even().bits(), 0x40000000000000000000000000000000 as u128);
	assert_eq(quad_neg_half.round_ties_even().bits(), 0x80000000000000000000000000000000 as u128);
	assert_eq(quad_neg_half.floor().bits(), 0xbfff0000000000000000000000000000 as u128);
}

#[test]
fn float_to_int_casts_truncate_toward_zero() {
	let double_three_point_seven_five = f64::new(0x400e000000000000 as u64);
	let double_neg_three_point_seven_five = f64::new(0xc00e000000000000 as u64);
	let double_neg_half = f64::new(0xbfe0000000000000 as u64);
	let double_neg_zero = f64::new(0x8000000000000000 as u64);
	let double_two_pow_63 = f64::new(0x43e0000000000000 as u64);
	let double_neg_two_pow_63 = f64::new(0xc3e0000000000000 as u64);
	let double_near_two_pow_64 = f64::new(0x43efffffffffffff as u64);

	assert_eq(double_three_point_seven_five.to_u64(), 3 as u64);
	assert_eq(double_three_point_seven_five.to_i64(), 3);
	assert_eq(double_neg_three_point_seven_five.to_i64(), -3);
	assert_eq(double_neg_half.to_u64(), 0 as u64);
	assert_eq(double_neg_half.to_i64(), 0);
	assert_eq(double_neg_zero.to_u64(), 0 as u64);
	assert_eq(double_two_pow_63.to_u64(), 0x8000000000000000 as u64);
	assert_eq(double_neg_two_pow_63.to_i64(), -9223372036854775807 - 1);
	assert_eq(double_near_two_pow_64.to_u64(), 0xfffffffffffff800 as u64);

	let half_max = f16::new(0x7bff as u16);
	let half_neg_one = f16::new(0xbc00 as u16);

	assert_eq(half_max.to_u64(), 65504 as u64);
	assert_eq(half_max.to_i64(), 65504);
	assert_eq(half_neg_one.to_i64(), -1);

	let single_five_point_five = f32::new(0x40b00000 as u32);
	let single_two_pow_63 = f32::new(0x5f000000 as u32);

	assert_eq(single_five_point_five.to_u64(), 5 as u64);
	assert_eq(single_two_pow_63.to_u64(), 0x8000000000000000 as u64);

	let quad_one_point_five = f128::new(0x3fff8000000000000000000000000000 as u128);
	let quad_two_pow_63 = f128::new(0x403e0000000000000000000000000000 as u128);
	let quad_neg_two_point_five = f128::new(0xc0004000000000000000000000000000 as u128);

	assert_eq(quad_one_point_five.to_u64(), 1 as u64);
	assert_eq(quad_two_pow_63.to_u64(), 0x8000000000000000 as u64);
	assert_eq(quad_neg_two_point_five.to_i64(), -2);
}

#[test]
fn sqrt_handles_exact_and_rounded_roots() {
	let four = f16::new(0x4400 as u16);
	let nine = f16::new(0x4880 as u16);
	let one = f16::new(0x3c00 as u16);
	let two = f16::new(0x4000 as u16);
	let quarter = f16::new(0x3400 as u16);
	let half = f16::new(0x3800 as u16);

	assert_eq(four.sqrt().bits(), 0x4000 as u16);
	assert_eq(nine.sqrt().bits(), 0x4200 as u16);
	assert_eq(one.sqrt().bits(), 0x3c00 as u16);
	// sqrt(2) and sqrt(0.5) exercise both exponent parities.
	assert_eq(two.sqrt().bits(), 0x3da8 as u16);
	assert_eq(half.sqrt().bits(), 0x39a8 as u16);
	assert_eq(quarter.sqrt().bits(), 0x3800 as u16);

	// Subnormal input: sqrt(2^-24) = 2^-12 is normal.
	assert_eq(f16::new(0x0001 as u16).sqrt().bits(), 0x0c00 as u16);

	let single_two = f32::new(0x40000000 as u32);
	let double_two = f64::new(0x4000000000000000 as u64);

	assert_eq(single_two.sqrt().bits(), 0x3fb504f3 as u32);
	assert_eq(double_two.sqrt().bits(), 0x3ff6a09e667f3bcd as u64);

	let quad_four = f128::new(0x40010000000000000000000000000000 as u128);
	let quad_two_point_two_five = f128::new(0x40002000000000000000000000000000 as u128);

	assert_eq(quad_four.sqrt().bits(), 0x40000000000000000000000000000000 as u128);
	assert_eq(quad_two_point_two_five.sqrt().bits(), 0x3fff8000000000000000000000000000 as u128);
}

#[test]
fn sqrt_handles_special_values() {
	let pos_zero = f16::new(0x0000 as u16);
	let neg_zero = f16::new(0x8000 as u16);
	let pos_inf = f16::new(0x7c00 as u16);
	let neg_inf = f16::new(0xfc00 as u16);
	let neg_one = f16::new(0xbc00 as u16);
	let nan = f16::new(0x7e01 as u16);

	assert_eq(pos_zero.sqrt().bits(), 0x0000 as u16);
	assert_eq(neg_zero.sqrt().bits(), 0x8000 as u16);
	assert_eq(pos_inf.sqrt().bits(), 0x7c00 as u16);
	assert_eq(neg_inf.sqrt().bits(), 0x7e00 as u16);
	assert_eq(neg_one.sqrt().bits(), 0x7e00 as u16);
	assert_eq(nan.sqrt().bits(), 0x7e00 as u16);

	let double_neg = f64::new(0xbff0000000000000 as u64);
	let quad_neg_inf = f128::new(0xffff0000000000000000000000000000 as u128);

	assert_eq(double_neg.sqrt().bits(), 0x7ff8000000000000 as u64);
	assert_eq(quad_neg_inf.sqrt().bits(), 0x7fff8000000000000000000000000000 as u128);
}

#[test(should_fail)]
fn float_to_u64_rejects_nan() {
	let _ = f64::new(0x7ff8000000000000 as u64).to_u64();
}

#[test(should_fail)]
fn float_to_i64_rejects_infinity() {
	let _ = f64::new(0x7ff0000000000000 as u64).to_i64();
}

#[test(should_fail)]
fn float_to_u64_rejects_negative_values() {
	let _ = f64::new(0xc00e000000000000 as u64).to_u64();
}

#[test(should_fail)]
fn float_to_u64_rejects_two_pow_64() {
	let _ = f64::new(0x43f0000000000000 as u64).to_u64();
}

#[test(should_fail)]
fn float_to_i64_rejects_two_pow_63() {
	let _ = f64::new(0x43e0000000000000 as u64).to_i64();
}

#[test(should_fail)]
fn float_to_i64_rejects_below_min() {
	// -(2^63 + 2^11) truncates below i64::MIN.
	let _ = f64::new(0xc3e0000000000001 as u64).to_i64();
}

// -- sq-3x7dl.1 soundness: negative / forge tests ----------------------------
//
// Each test constructs the NON-CANONICAL witness that the audit showed re-encodes
// to the same bit pattern as the canonical decode, then calls assert_canonical_fields
// on the malicious struct to verify the range check REJECTS it.
//
// Pattern for each width: {exp-1, mantissa+2^mant_size} gives the same bits()
// as canonical {exp, mantissa} because:
//   (exp-1)*2^mant_size + (mantissa+2^mant_size) = exp*2^mant_size + mantissa
//
// `should_fail` means the test PASSES only when the circuit panics -- i.e.
// only when the range check correctly rejects the over-wide witness.
//
// The tests call assert_canonical_fields (not assert_max_bit_size inline) so that
// deleting the assertions from the helper body turns these tests RED -- pinning the
// helper's logic against regression.  True circuit-level pinning of the new()
// call-site (a malicious witness fed through the prover) is a forge/proving-level
// obligation tracked under sq-l9ulg / sq-qhy4.
// [SONNET-4.6] sq-3x7dl.1

// -- f32: primary counterexample from the audit -------------------------------
// bits=0x3F800001 (1.0 + 2^-23). Canonical: {sign:0, exp:127, mant:1}.
// Malicious:       {sign:0, exp:126, mant:0x800001=8388609}.
// 126*2^23 + 8388609 = 127*2^23 + 1 = 0x3F800001.
// 8388609 > 2^23-1=8388607 so assert_canonical_fields rejects it.
#[test(should_fail)]
fn f32_new_rejects_malicious_mantissa_overlap() {
	let malicious = f32 { sign: false, exponent: 126 as u8, mantissa: 0x800001 as u32 };
	assert_eq(malicious.bits(), 0x3F800001 as u32);
	f32::assert_canonical_fields(malicious.exponent, malicious.mantissa);
}

// -- f16: mantissa-overlap attack ---------------------------------------------
// bits=0x1400. Canonical: {exp:5, mant:0}.
// Malicious {exp:4, mant:1024=2^10}: 4*2^10+1024=5*2^10=0x1400. 1024>2^10-1=1023.
#[test(should_fail)]
fn f16_new_rejects_malicious_mantissa_overlap() {
	let malicious = f16 { sign: false, exponent: 4 as u8, mantissa: 1024 as u16 };
	assert_eq(malicious.bits(), 0x1400 as u16);
	f16::assert_canonical_fields(malicious.exponent, malicious.mantissa);
}

// -- f16: exponent-overflow attack --------------------------------------------
// bits=0x8000 (-0). Canonical: {sign:1, exp:0, mant:0}.
// Malicious {sign:0, exp:32, mant:0}: 32*2^10=0x8000 (same bits).
// exp=32 overflows the 5-bit IEEE exponent field (max 31).
#[test(should_fail)]
fn f16_new_rejects_malicious_exponent_overflow() {
	let malicious = f16 { sign: false, exponent: 32 as u8, mantissa: 0 as u16 };
	assert_eq(malicious.bits(), 0x8000 as u16);
	f16::assert_canonical_fields(malicious.exponent, malicious.mantissa);
}

// -- f64: mantissa-overlap attack ---------------------------------------------
// bits=0x3FF0000000000001 (1.0 + ulp). Canonical: {exp:1023, mant:1}.
// Malicious {exp:1022, mant:2^52+1=4503599627370497}:
//   1022*2^52 + (2^52+1) = 1023*2^52 + 1 = 0x3FF0000000000001.
// 4503599627370497 > 2^52-1=4503599627370495.
#[test(should_fail)]
fn f64_new_rejects_malicious_mantissa_overlap() {
	let two_pow_52_plus_1: u64 = (1 as u64 << 52) + 1;
	let malicious = f64 { sign: false, exponent: 1022 as u16, mantissa: two_pow_52_plus_1 };
	assert_eq(malicious.bits(), 0x3FF0000000000001 as u64);
	f64::assert_canonical_fields(malicious.exponent, malicious.mantissa);
}

// -- f128: mantissa-overlap attack --------------------------------------------
// bits=0x3FFF0000000000000000000000000001 (canonical: {exp:16383, mant:1}).
// Malicious {exp:16382, mant:2^112+1}:
//   16382*2^112 + (2^112+1) = 16383*2^112 + 1 = bits.
// 2^112+1 > 2^112-1.
#[test(should_fail)]
fn f128_new_rejects_malicious_mantissa_overlap() {
	let two_pow_112: u128 = 1 as u128 << 112;
	let malicious = f128 {
		sign: false,
		exponent: 16382 as u16,
		mantissa: two_pow_112 + 1,
	};
	assert_eq(malicious.bits(), 0x3FFF0000000000000000000000000001 as u128);
	f128::assert_canonical_fields(malicious.exponent, malicious.mantissa);
}

// -- positive: canonical boundary values still accepted -----------------------
// Confirms the fix does NOT break canonical decodes at the max-field boundary.
#[test]
fn new_accepts_canonical_boundary_values() {
	// f16 max mantissa (2^10-1=1023) and max exponent (31=0x1F).
	let half_max_sub = f16::new(0x03FF as u16); // mant=1023, exp=0
	assert_eq(half_max_sub.mantissa, 1023 as u16);
	let half_inf = f16::new(0x7C00 as u16); // exp=31, mant=0
	assert_eq(half_inf.exponent, 31 as u8);

	// f32 max mantissa (2^23-1=8388607) and max exponent (255).
	let single_nan_payload = f32::new(0x7FFFFFFF as u32);
	assert_eq(single_nan_payload.exponent, 255 as u8);
	assert_eq(single_nan_payload.mantissa, 8388607 as u32);

	// f64 max mantissa (2^52-1=4503599627370495) and max exponent (2047).
	let double_nan = f64::new(0x7FFFFFFFFFFFFFFF as u64);
	assert_eq(double_nan.exponent, 2047 as u16);
	assert_eq(double_nan.mantissa, 4503599627370495 as u64);

	// f128 max mantissa (2^112-1) and max exponent (32767).
	let quad_nan = f128::new(0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF as u128);
	assert_eq(quad_nan.exponent, 32767 as u16);
	assert_eq(quad_nan.mantissa, 5192296858534827628530496329220095 as u128);
}

#[test]
fn uint_types_match_float_component_sizes() {
	comptime {
		assert_eq(sizing::uint_type(5), quote { u8 }.as_type());
		assert_eq(sizing::uint_type(10), quote { u16 }.as_type());
		assert_eq(sizing::uint_type(23), quote { u32 }.as_type());
	}
}

#[test]
fn ieee_component_sizes_are_available_at_comptime() {
	comptime {
		assert_eq(sizing::exponent_size(16), 5);
		assert_eq(sizing::mantissa_size(16), 10);
		assert_eq(sizing::exponent_size(32), 8);
		assert_eq(sizing::mantissa_size(32), 23);
		assert_eq(sizing::exponent_size(64), 11);
		assert_eq(sizing::mantissa_size(64), 52);
		assert_eq(sizing::exponent_size(128), 15);
		assert_eq(sizing::mantissa_size(128), 112);
		assert_eq(sizing::significand_size(128), 113);
	}
}