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