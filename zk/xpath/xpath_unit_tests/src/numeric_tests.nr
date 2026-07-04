//! Additional tests for numeric operations

use dep::xpath::{
    abs_int, cast_double_to_float, cast_double_to_integer, cast_float_to_integer,
    cast_integer_to_double, cast_integer_to_float, ceil_int, compare_double_float_eq,
    compare_double_float_ge, compare_double_float_gt, compare_double_float_le,
    compare_double_float_lt, compare_double_int_eq, compare_double_int_ge, compare_double_int_gt,
    compare_double_int_le, compare_double_int_lt, compare_float_double_eq, compare_float_double_ge,
    compare_float_double_gt, compare_float_double_le, compare_float_double_lt, compare_float_int_eq,
    compare_float_int_ge, compare_float_int_gt, compare_float_int_le, compare_float_int_lt,
    compare_int_double_eq, compare_int_double_ge, compare_int_double_gt, compare_int_double_le,
    compare_int_double_lt, compare_int_float_eq, compare_int_float_ge, compare_int_float_gt,
    compare_int_float_le, compare_int_float_lt, floor_int, fn_number_from_string, get_common_type,
    max_int, min_int,
    numeric_add_double, numeric_add_float, numeric_add_int, numeric_divide_int,
    numeric_divide_int_as_double,
    numeric_equal_double, numeric_equal_float, numeric_equal_int, numeric_ge_int,
    numeric_greater_than_double, numeric_greater_than_float, numeric_greater_than_int,
    numeric_le_int, numeric_less_than_double, numeric_less_than_float, numeric_less_than_int,
    numeric_mod_int, numeric_multiply_int, numeric_subtract_int, numeric_unary_minus_int,
    numeric_unary_plus_int, NumericType, round_int, XsdDouble, XsdFloat,
};

// ============================================================================
// Tests moved from xpath/src/numeric.nr
// ============================================================================

#[test]
fn test_numeric_add_int() {
    assert(numeric_add_int(2, 3) == 5);
    assert(numeric_add_int(-2, 3) == 1);
    assert(numeric_add_int(-2, -3) == -5);
    assert(numeric_add_int(0, 0) == 0);
}

#[test]
fn test_numeric_subtract_int() {
    assert(numeric_subtract_int(5, 3) == 2);
    assert(numeric_subtract_int(3, 5) == -2);
    assert(numeric_subtract_int(-2, -3) == 1);
}

#[test]
fn test_numeric_multiply_int() {
    assert(numeric_multiply_int(2, 3) == 6);
    assert(numeric_multiply_int(-2, 3) == -6);
    assert(numeric_multiply_int(-2, -3) == 6);
    assert(numeric_multiply_int(0, 100) == 0);
}

#[test]
fn test_numeric_divide_int() {
    // op:numeric-integer-divide (idiv) -- truncates TOWARD ZERO.
    // op:numeric-divide over integers is numeric_divide_int_as_double below.
    assert(numeric_divide_int(6, 3) == 2);
    assert(numeric_divide_int(7, 3) == 2); // idiv truncates
    assert(numeric_divide_int(-6, 3) == -2);
    assert(numeric_divide_int(7, 2) == 3); // 7 idiv 2 = 3 (7 div 2 = 3.5)
    assert(numeric_divide_int(-7, 2) == -3); // toward zero, not floor (-4)
}

// ============================================================================
// op:numeric-divide over xs:integer operands (decimal semantics, never
// truncating). Expected bit patterns are from an independent Python/IEEE 754
// hardware oracle: struct.unpack('<Q', struct.pack('<d', a/b)). [FABLE-5]
// ============================================================================

#[test]
fn test_numeric_divide_int_as_double_non_exact() {
    // 7/2 = 3.5 -- THE case the qt3 corpus never exercises (exact quotients only)
    assert(numeric_divide_int_as_double(7, 2) == XsdDouble::from_bits(0x400C000000000000));
    // -7/2 = -3.5
    assert(numeric_divide_int_as_double(-7, 2) == XsdDouble::from_bits(0xC00C000000000000));
    // 10/4 = 2.5
    assert(numeric_divide_int_as_double(10, 4) == XsdDouble::from_bits(0x4004000000000000));
    // 1/2 = 0.5
    assert(numeric_divide_int_as_double(1, 2) == XsdDouble::from_bits(0x3FE0000000000000));
}

#[test]
fn test_numeric_divide_int_as_double_repeating() {
    // Non-terminating decimal quotients: correctly-rounded doubles.
    // 1/3 = 0.3333333333333333 (0x3FD5555555555555)
    assert(numeric_divide_int_as_double(1, 3) == XsdDouble::from_bits(0x3FD5555555555555));
    // 2/3 = 0.6666666666666666 (0x3FE5555555555555)
    assert(numeric_divide_int_as_double(2, 3) == XsdDouble::from_bits(0x3FE5555555555555));
    // 10/3 = 3.3333333333333335 (0x400AAAAAAAAAAAAB)
    assert(numeric_divide_int_as_double(10, 3) == XsdDouble::from_bits(0x400AAAAAAAAAAAAB));
    // -10/3 = -3.3333333333333335 (0xC00AAAAAAAAAAAAB)
    assert(numeric_divide_int_as_double(-10, 3) == XsdDouble::from_bits(0xC00AAAAAAAAAAAAB));
    // 22/7 = 3.142857142857143 (0x4009249249249249)
    assert(numeric_divide_int_as_double(22, 7) == XsdDouble::from_bits(0x4009249249249249));
}

#[test]
fn test_numeric_divide_int_as_double_exact() {
    // Exact quotients agree with idiv promoted to double
    assert(numeric_divide_int_as_double(6, 3) == XsdDouble::from_bits(0x4000000000000000)); // 2.0
    assert(numeric_divide_int_as_double(0, 5) == XsdDouble::from_bits(0x0000000000000000)); // 0.0
}

#[test]
fn test_numeric_divide_int_as_double_beyond_2_53() {
    // Operand above 2^53: the i64->f64 conversion itself rounds (documented
    // approximation). Oracle: float(999999999999999999)/float(3)
    // = 3.333333333333333e+17 (0x439280F39A348555).
    assert(
        numeric_divide_int_as_double(999999999999999999, 3)
            == XsdDouble::from_bits(0x439280F39A348555),
    );
}

#[test]
fn test_divide_and_integer_divide_are_distinct() {
    // The two XPath operators must give DIFFERENT answers on 7/2:
    // op:numeric-integer-divide -> 3, op:numeric-divide -> 3.5.
    assert(numeric_divide_int(7, 2) == 3);
    let q = numeric_divide_int_as_double(7, 2);
    assert(q == XsdDouble::from_bits(0x400C000000000000)); // 3.5
    // and 3.5 != 3.0 (0x4008000000000000)
    assert(!(q == XsdDouble::from_bits(0x4008000000000000)));
}

#[test(should_fail_with = "Division by zero")]
fn test_numeric_divide_int_as_double_by_zero_fails() {
    // xs:decimal division by zero is err:FOAR0001 -- fail-closed, no Infinity
    let _ = numeric_divide_int_as_double(1, 0);
}

#[test]
fn test_numeric_unary_plus_int() {
    assert(numeric_unary_plus_int(5) == 5);
    assert(numeric_unary_plus_int(-5) == -5);
}

#[test]
fn test_numeric_unary_minus_int() {
    assert(numeric_unary_minus_int(5) == -5);
    assert(numeric_unary_minus_int(-5) == 5);
}

#[test]
fn test_numeric_equal_int() {
    assert(numeric_equal_int(5, 5) == true);
    assert(numeric_equal_int(5, 3) == false);
    assert(numeric_equal_int(-5, -5) == true);
}

#[test]
fn test_numeric_less_than_int() {
    assert(numeric_less_than_int(3, 5) == true);
    assert(numeric_less_than_int(5, 3) == false);
    assert(numeric_less_than_int(5, 5) == false);
    assert(numeric_less_than_int(-5, 3) == true);
}

#[test]
fn test_numeric_greater_than_int() {
    assert(numeric_greater_than_int(5, 3) == true);
    assert(numeric_greater_than_int(3, 5) == false);
    assert(numeric_greater_than_int(5, 5) == false);
    assert(numeric_greater_than_int(3, -5) == true);
}

#[test]
fn test_abs_int() {
    assert(abs_int(5) == 5);
    assert(abs_int(-5) == 5);
    assert(abs_int(0) == 0);
}

#[test]
fn test_numeric_mod_int() {
    assert(numeric_mod_int(7, 3) == 1);
    assert(numeric_mod_int(6, 3) == 0);
    assert(numeric_mod_int(-7, 3) == -1);
    assert(numeric_mod_int(7, -3) == 1);
}

#[test]
fn test_rounding_functions_int() {
    // For integers, these are identity operations
    assert(round_int(5) == 5);
    assert(round_int(-5) == -5);
    assert(ceil_int(5) == 5);
    assert(floor_int(5) == 5);
}

#[test]
fn test_min_max_int() {
    assert(min_int(3, 5) == 3);
    assert(min_int(5, 3) == 3);
    assert(min_int(-3, 5) == -3);
    assert(max_int(3, 5) == 5);
    assert(max_int(5, 3) == 5);
    assert(max_int(-3, 5) == 5);
}

// ============================================================================
// Tests moved from xpath/src/numeric_types.nr
// ============================================================================

#[test]
fn test_numeric_type_constructors() {
    assert(NumericType::integer().is_integer());
    assert(NumericType::decimal().is_decimal());
    assert(NumericType::float().is_float());
    assert(NumericType::double().is_double());
}

#[test]
fn test_get_common_type() {
    // Integer + Integer = Integer
    assert(get_common_type(NumericType::integer(), NumericType::integer()).is_integer());

    // Integer + Decimal = Decimal
    assert(get_common_type(NumericType::integer(), NumericType::decimal()).is_decimal());

    // Float + Integer = Float (XPath 2.0 F&O Section 17.1.4: xs:decimal/xs:integer promote to xs:float,
    // not xs:double; only xs:double triggers double promotion - Bug B fix [SONNET-4.6])
    assert(get_common_type(NumericType::float(), NumericType::integer()).is_float());

    // Float + Decimal = Float (same reasoning)
    assert(get_common_type(NumericType::float(), NumericType::decimal()).is_float());

    // Double + anything = Double
    assert(get_common_type(NumericType::double(), NumericType::integer()).is_double());
    assert(get_common_type(NumericType::double(), NumericType::float()).is_double());
    assert(get_common_type(NumericType::double(), NumericType::decimal()).is_double());
}

#[test]
fn test_float_zero() {
    let zero = XsdFloat::zero();
    let neg_zero = XsdFloat::neg_zero();

    // +0 == -0 per IEEE 754
    assert(numeric_equal_float(zero, neg_zero));
}

#[test]
fn test_float_nan() {
    let nan = XsdFloat::nan();

    // NaN != NaN per IEEE 754
    assert(!numeric_equal_float(nan, nan));
    assert(!numeric_less_than_float(nan, nan));
    assert(!numeric_greater_than_float(nan, nan));
}

#[test]
fn test_float_infinity() {
    let inf = XsdFloat::infinity();
    let neg_inf = XsdFloat::neg_infinity();
    let one = XsdFloat::from_bits(0x3F800000); // 1.0f

    assert(numeric_less_than_float(neg_inf, one));
    assert(numeric_less_than_float(one, inf));
    assert(numeric_greater_than_float(inf, neg_inf));
}

#[test]
fn test_double_zero() {
    let zero = XsdDouble::zero();
    let neg_zero = XsdDouble::neg_zero();

    // +0 == -0 per IEEE 754
    assert(numeric_equal_double(zero, neg_zero));
}

#[test]
fn test_double_nan() {
    let nan = XsdDouble::nan();

    // NaN != NaN per IEEE 754
    assert(!numeric_equal_double(nan, nan));
    assert(!numeric_less_than_double(nan, nan));
    assert(!numeric_greater_than_double(nan, nan));
}

#[test]
fn test_float_arithmetic() {
    // Use known bit patterns for 2.0 and 3.0
    let two = XsdFloat::from_bits(0x40000000); // 2.0f
    let three = XsdFloat::from_bits(0x40400000); // 3.0f
    let five = XsdFloat::from_bits(0x40A00000); // 5.0f

    let sum = numeric_add_float(two, three);
    assert(numeric_equal_float(sum, five));
}

#[test]
fn test_double_arithmetic() {
    // Use known bit patterns for 2.0 and 3.0
    let two = XsdDouble::from_bits(0x4000000000000000); // 2.0
    let three = XsdDouble::from_bits(0x4008000000000000); // 3.0
    let five = XsdDouble::from_bits(0x4014000000000000); // 5.0

    let sum = numeric_add_double(two, three);
    assert(numeric_equal_double(sum, five));
}

#[test]
fn test_float_to_double_promotion() {
    // Promote 1.0f to double
    let one_float = XsdFloat::from_bits(0x3F800000); // 1.0f
    let one_double = XsdDouble::from_float(one_float);
    let expected = XsdDouble::from_bits(0x3FF0000000000000); // 1.0

    assert(numeric_equal_double(one_double, expected));
}

#[test]
fn test_small_int_to_float() {
    let one = XsdFloat::from_small_int(1);
    let expected = XsdFloat::from_bits(0x3F800000); // 1.0f
    assert(numeric_equal_float(one, expected));
}

#[test]
fn test_small_int_to_double() {
    let one = XsdDouble::from_small_int(1);
    let expected = XsdDouble::from_bits(0x3FF0000000000000); // 1.0
    assert(numeric_equal_double(one, expected));
}

// ============================================================================
// Type Casting Tests
// ============================================================================

#[test]
fn test_cast_integer_to_float_zero() {
    let result = cast_integer_to_float(0);
    assert(numeric_equal_float(result, XsdFloat::zero()));
}

#[test]
fn test_cast_integer_to_float_one() {
    let result = cast_integer_to_float(1);
    let expected = XsdFloat::from_bits(0x3F800000); // 1.0f
    assert(numeric_equal_float(result, expected));
}

#[test]
fn test_cast_integer_to_float_negative() {
    let result = cast_integer_to_float(-1);
    let expected = XsdFloat::from_bits(0xBF800000); // -1.0f
    assert(numeric_equal_float(result, expected));
}

#[test]
fn test_cast_integer_to_float_ten() {
    let result = cast_integer_to_float(10);
    let expected = XsdFloat::from_bits(0x41200000); // 10.0f
    assert(numeric_equal_float(result, expected));
}

#[test]
fn test_cast_integer_to_double_zero() {
    let result = cast_integer_to_double(0);
    assert(numeric_equal_double(result, XsdDouble::zero()));
}

#[test]
fn test_cast_integer_to_double_one() {
    let result = cast_integer_to_double(1);
    let expected = XsdDouble::from_bits(0x3FF0000000000000); // 1.0
    assert(numeric_equal_double(result, expected));
}

#[test]
fn test_cast_integer_to_double_negative() {
    let result = cast_integer_to_double(-1);
    let expected = XsdDouble::from_bits(0xBFF0000000000000); // -1.0
    assert(numeric_equal_double(result, expected));
}

#[test]
fn test_cast_integer_to_double_hundred() {
    // Test with 100, which is within i8 range
    let result = cast_integer_to_double(100);
    let expected = XsdDouble::from_bits(0x4059000000000000); // 100.0
    assert(numeric_equal_double(result, expected));
}

#[test]
fn test_cast_float_to_integer_zero() {
    let f = XsdFloat::zero();
    let result = cast_float_to_integer(f);
    assert(result.is_some());
    assert(result.unwrap() == 0);
}

#[test]
fn test_cast_float_to_integer_one() {
    let f = XsdFloat::from_bits(0x3F800000); // 1.0f
    let result = cast_float_to_integer(f);
    assert(result.is_some());
    assert(result.unwrap() == 1);
}

#[test]
fn test_cast_float_to_integer_negative() {
    let f = XsdFloat::from_bits(0xC0A00000); // -5.0f
    let result = cast_float_to_integer(f);
    assert(result.is_some());
    assert(result.unwrap() == -5);
}

#[test]
fn test_cast_float_to_integer_truncates() {
    let f = XsdFloat::from_bits(0x40133333); // 2.3f
    let result = cast_float_to_integer(f);
    assert(result.is_some());
    assert(result.unwrap() == 2);
}

#[test]
fn test_cast_float_to_integer_nan() {
    let f = XsdFloat::nan();
    let result = cast_float_to_integer(f);
    assert(result.is_none());
}

#[test]
fn test_cast_float_to_integer_infinity() {
    let f = XsdFloat::infinity();
    let result = cast_float_to_integer(f);
    assert(result.is_none());
}

#[test]
fn test_cast_double_to_integer_zero() {
    let d = XsdDouble::zero();
    let result = cast_double_to_integer(d);
    assert(result.is_some());
    assert(result.unwrap() == 0);
}

#[test]
fn test_cast_double_to_integer_one() {
    let d = XsdDouble::from_bits(0x3FF0000000000000); // 1.0
    let result = cast_double_to_integer(d);
    assert(result.is_some());
    assert(result.unwrap() == 1);
}

#[test]
fn test_cast_double_to_integer_negative() {
    let d = XsdDouble::from_bits(0xC014000000000000); // -5.0
    let result = cast_double_to_integer(d);
    assert(result.is_some());
    assert(result.unwrap() == -5);
}

#[test]
fn test_cast_double_to_integer_truncates() {
    let d = XsdDouble::from_bits(0x4002666666666666); // 2.3
    let result = cast_double_to_integer(d);
    assert(result.is_some());
    assert(result.unwrap() == 2);
}

#[test]
fn test_cast_double_to_integer_nan() {
    let d = XsdDouble::nan();
    let result = cast_double_to_integer(d);
    assert(result.is_none());
}

#[test]
fn test_cast_double_to_integer_infinity() {
    let d = XsdDouble::infinity();
    let result = cast_double_to_integer(d);
    assert(result.is_none());
}

#[test]
fn test_cast_double_to_float_zero() {
    let d = XsdDouble::zero();
    let result = cast_double_to_float(d);
    assert(numeric_equal_float(result, XsdFloat::zero()));
}

#[test]
fn test_cast_double_to_float_one() {
    let d = XsdDouble::from_bits(0x3FF0000000000000); // 1.0
    let result = cast_double_to_float(d);
    let expected = XsdFloat::from_bits(0x3F800000); // 1.0f
    assert(numeric_equal_float(result, expected));
}

#[test]
fn test_cast_double_to_float_nan() {
    let d = XsdDouble::nan();
    let result = cast_double_to_float(d);
    assert(result.is_nan());
}

#[test]
fn test_cast_double_to_float_infinity() {
    let d = XsdDouble::infinity();
    let result = cast_double_to_float(d);
    assert(numeric_equal_float(result, XsdFloat::infinity()));
}

#[test]
fn test_cast_double_to_float_neg_infinity() {
    let d = XsdDouble::neg_infinity();
    let result = cast_double_to_float(d);
    assert(numeric_equal_float(result, XsdFloat::neg_infinity()));
}

// ============================================================================
// Additional tests
// ============================================================================

#[test]
fn test_integer_edge_cases() {
    // Test with larger numbers
    assert(numeric_add_int(1000000, 2000000) == 3000000);
    assert(numeric_multiply_int(10000, 10000) == 100000000);

    // Test division truncation
    assert(numeric_divide_int(10, 3) == 3);
    assert(numeric_divide_int(10, 4) == 2);
    assert(numeric_divide_int(-10, 3) == -3);
}

#[test]
fn test_integer_comparison_edge_cases() {
    // Test with negative numbers
    assert(numeric_less_than_int(-100, -50) == true);
    assert(numeric_less_than_int(-50, -100) == false);
    assert(numeric_greater_than_int(-50, -100) == true);

    // Test zero
    assert(numeric_less_than_int(-1, 0) == true);
    assert(numeric_less_than_int(0, 1) == true);
    assert(numeric_equal_int(0, 0) == true);
}

#[test]
fn test_abs_edge_cases() {
    // Large values
    assert(abs_int(1000000000) == 1000000000);
    assert(abs_int(-1000000000) == 1000000000);
}

// ============================================================================
// Tests for new comparison operators (<=, >=)
// ============================================================================

#[test]
fn test_numeric_le_int() {
    // Less than
    assert(numeric_le_int(3, 5) == true);
    // Equal
    assert(numeric_le_int(5, 5) == true);
    // Greater than
    assert(numeric_le_int(5, 3) == false);
    // Negative numbers
    assert(numeric_le_int(-10, -5) == true);
    assert(numeric_le_int(-5, -5) == true);
    assert(numeric_le_int(-5, -10) == false);
    // Zero
    assert(numeric_le_int(0, 0) == true);
    assert(numeric_le_int(-1, 0) == true);
    assert(numeric_le_int(0, 1) == true);
}

#[test]
fn test_numeric_ge_int() {
    // Greater than
    assert(numeric_ge_int(5, 3) == true);
    // Equal
    assert(numeric_ge_int(5, 5) == true);
    // Less than
    assert(numeric_ge_int(3, 5) == false);
    // Negative numbers
    assert(numeric_ge_int(-5, -10) == true);
    assert(numeric_ge_int(-5, -5) == true);
    assert(numeric_ge_int(-10, -5) == false);
    // Zero
    assert(numeric_ge_int(0, 0) == true);
    assert(numeric_ge_int(0, -1) == true);
    assert(numeric_ge_int(1, 0) == true);
}

// ============================================================================
// Tests for mixed-type comparison operators
// ============================================================================

#[test]
fn test_compare_int_double_le() {
    let int_val = 5;
    let double_val = XsdDouble::from_small_int(10);
    assert(compare_int_double_le(int_val, double_val) == true);

    let int_val2 = 10;
    let double_val2 = XsdDouble::from_small_int(10);
    assert(compare_int_double_le(int_val2, double_val2) == true);

    let int_val3 = 15;
    let double_val3 = XsdDouble::from_small_int(10);
    assert(compare_int_double_le(int_val3, double_val3) == false);
}

#[test]
fn test_compare_int_double_ge() {
    let int_val = 15;
    let double_val = XsdDouble::from_small_int(10);
    assert(compare_int_double_ge(int_val, double_val) == true);

    let int_val2 = 10;
    let double_val2 = XsdDouble::from_small_int(10);
    assert(compare_int_double_ge(int_val2, double_val2) == true);

    let int_val3 = 5;
    let double_val3 = XsdDouble::from_small_int(10);
    assert(compare_int_double_ge(int_val3, double_val3) == false);
}

#[test]
fn test_compare_double_int_le() {
    let double_val = XsdDouble::from_small_int(5);
    let int_val = 10;
    assert(compare_double_int_le(double_val, int_val) == true);

    let double_val2 = XsdDouble::from_small_int(10);
    let int_val2 = 10;
    assert(compare_double_int_le(double_val2, int_val2) == true);

    let double_val3 = XsdDouble::from_small_int(15);
    let int_val3 = 10;
    assert(compare_double_int_le(double_val3, int_val3) == false);
}

#[test]
fn test_compare_double_int_ge() {
    let double_val = XsdDouble::from_small_int(15);
    let int_val = 10;
    assert(compare_double_int_ge(double_val, int_val) == true);

    let double_val2 = XsdDouble::from_small_int(10);
    let int_val2 = 10;
    assert(compare_double_int_ge(double_val2, int_val2) == true);

    let double_val3 = XsdDouble::from_small_int(5);
    let int_val3 = 10;
    assert(compare_double_int_ge(double_val3, int_val3) == false);
}

#[test]
fn test_compare_float_double_le() {
    let float_val = XsdFloat::from_small_int(5);
    let double_val = XsdDouble::from_small_int(10);
    assert(compare_float_double_le(float_val, double_val) == true);

    let float_val2 = XsdFloat::from_small_int(10);
    let double_val2 = XsdDouble::from_small_int(10);
    assert(compare_float_double_le(float_val2, double_val2) == true);

    let float_val3 = XsdFloat::from_small_int(15);
    let double_val3 = XsdDouble::from_small_int(10);
    assert(compare_float_double_le(float_val3, double_val3) == false);
}

#[test]
fn test_compare_float_double_ge() {
    let float_val = XsdFloat::from_small_int(15);
    let double_val = XsdDouble::from_small_int(10);
    assert(compare_float_double_ge(float_val, double_val) == true);

    let float_val2 = XsdFloat::from_small_int(10);
    let double_val2 = XsdDouble::from_small_int(10);
    assert(compare_float_double_ge(float_val2, double_val2) == true);

    let float_val3 = XsdFloat::from_small_int(5);
    let double_val3 = XsdDouble::from_small_int(10);
    assert(compare_float_double_ge(float_val3, double_val3) == false);
}

#[test]
fn test_compare_double_float_le() {
    let double_val = XsdDouble::from_small_int(5);
    let float_val = XsdFloat::from_small_int(10);
    assert(compare_double_float_le(double_val, float_val) == true);

    let double_val2 = XsdDouble::from_small_int(10);
    let float_val2 = XsdFloat::from_small_int(10);
    assert(compare_double_float_le(double_val2, float_val2) == true);
}

#[test]
fn test_compare_double_float_ge() {
    let double_val = XsdDouble::from_small_int(15);
    let float_val = XsdFloat::from_small_int(10);
    assert(compare_double_float_ge(double_val, float_val) == true);

    let double_val2 = XsdDouble::from_small_int(10);
    let float_val2 = XsdFloat::from_small_int(10);
    assert(compare_double_float_ge(double_val2, float_val2) == true);
}

#[test]
fn test_compare_int_float_le() {
    let int_val = 5;
    let float_val = XsdFloat::from_small_int(10);
    assert(compare_int_float_le(int_val, float_val) == true);

    let int_val2 = 10;
    let float_val2 = XsdFloat::from_small_int(10);
    assert(compare_int_float_le(int_val2, float_val2) == true);
}

#[test]
fn test_compare_int_float_ge() {
    let int_val = 15;
    let float_val = XsdFloat::from_small_int(10);
    assert(compare_int_float_ge(int_val, float_val) == true);

    let int_val2 = 10;
    let float_val2 = XsdFloat::from_small_int(10);
    assert(compare_int_float_ge(int_val2, float_val2) == true);
}

#[test]
fn test_compare_float_int_le() {
    let float_val = XsdFloat::from_small_int(5);
    let int_val = 10;
    assert(compare_float_int_le(float_val, int_val) == true);

    let float_val2 = XsdFloat::from_small_int(10);
    let int_val2 = 10;
    assert(compare_float_int_le(float_val2, int_val2) == true);
}

#[test]
fn test_compare_float_int_ge() {
    let float_val = XsdFloat::from_small_int(15);
    let int_val = 10;
    assert(compare_float_int_ge(float_val, int_val) == true);

    let float_val2 = XsdFloat::from_small_int(10);
    let int_val2 = 10;
    assert(compare_float_int_ge(float_val2, int_val2) == true);
}

// ============================================================================
// Bug A fix (sq-3x7dl.5): soundness tests for exact i64->f64/f32 conversion.
// These cases FAIL under the old `a as i8` truncation and PASS with from_i64.
// [SONNET-4.6]
// ============================================================================

/// 256 as i8 truncates to 0, so the old code reported 256 == 0.0 as TRUE.
/// With exact i64->f64: 256.0 != 0.0, result must be FALSE.
#[test]
fn test_int_double_eq_256_vs_zero_is_false() {
    let d = XsdDouble::zero(); // 0.0
    assert(!compare_int_double_eq(256, d));
}

/// 1000 as i8 truncates to -24 (1000 mod 256 = 232, interpreted as signed = -24),
/// so the old code reported 1000 == 1000.0 as FALSE.
/// With exact i64->f64: 1000 == 1000.0 must be TRUE.
#[test]
fn test_int_double_eq_1000_is_true() {
    // 1000.0 in f64: 0x408F400000000000
    let d = XsdDouble::from_bits(0x408F400000000000);
    assert(compare_int_double_eq(1000, d));
}

/// 65536 (2^16) is outside i8 range; 65536 as i8 = 0 (truncation).
/// compare_int_double_gt(65536, 0.0) must be TRUE.
#[test]
fn test_int_double_gt_large_positive() {
    let d = XsdDouble::zero();
    assert(compare_int_double_gt(65536, d));
}

/// -256 as i8 = 0 (truncation: -256 mod 256 interpreted as 0),
/// so old code reported -256 == 0.0 as TRUE. Must be FALSE.
#[test]
fn test_int_double_eq_neg256_vs_zero_is_false() {
    let d = XsdDouble::zero();
    assert(!compare_int_double_eq(-256, d));
}

/// 1000000 (10^6) vs its exact f64 representation must be equal.
/// 10^6 as i8 = -24, so old code would report NOT equal. Must be TRUE.
#[test]
fn test_int_double_eq_million_is_true() {
    // 1000000.0 in f64: 0x412E848000000000
    let d = XsdDouble::from_bits(0x412E848000000000);
    assert(compare_int_double_eq(1000000, d));
}

/// compare_int_double_lt(-300, 0.0): -300 < 0 must be TRUE.
/// Old code: -300 as i8 = -44, which is still < 0, but this verifies
/// the sign is preserved correctly for out-of-i8-range negatives.
#[test]
fn test_int_double_lt_neg300_vs_zero() {
    let d = XsdDouble::zero();
    assert(compare_int_double_lt(-300, d));
}

/// compare_int_double_ge(256, 256.0) must be TRUE (equal case).
#[test]
fn test_int_double_ge_256_is_true() {
    // 256.0 in f64: 0x4070000000000000
    let d = XsdDouble::from_bits(0x4070000000000000);
    assert(compare_int_double_ge(256, d));
}

/// compare_int_double_le(256, 257.0) must be TRUE.
#[test]
fn test_int_double_le_256_lt_257() {
    // 257.0 in f64: 0x4070100000000000
    let d = XsdDouble::from_bits(0x4070100000000000);
    assert(compare_int_double_le(256, d));
}

/// double to int direction: compare_double_int_eq(1000.0, 1000) must be TRUE.
/// Old code: 1000 as i8 = -24, so the double side gets -24.0 and 1000.0 != -24.0.
#[test]
fn test_double_int_eq_1000_is_true() {
    // 1000.0 in f64
    let d = XsdDouble::from_bits(0x408F400000000000);
    assert(compare_double_int_eq(d, 1000));
}

/// compare_double_int_lt(0.0, 256) must be TRUE (0.0 < 256).
/// Old code: 256 as i8 = 0, so comparing 0.0 < 0.0 = FALSE (wrong).
#[test]
fn test_double_int_lt_zero_lt_256() {
    let d = XsdDouble::zero();
    assert(compare_double_int_lt(d, 256));
}

/// compare_double_int_gt(256.0, 255) must be TRUE.
/// 255 as i8 = -1, so old code compared 256.0 > -1.0 (accidentally TRUE for this case,
/// but the semantics are wrong). We keep this as a regression guard.
#[test]
fn test_double_int_gt_256_gt_255() {
    // 256.0 in f64: 0x4070000000000000
    let d = XsdDouble::from_bits(0x4070000000000000);
    assert(compare_double_int_gt(d, 255));
}

// ----------------------------------------------------------------------------
// Float (f32) direction - same class of bug
// ----------------------------------------------------------------------------

/// compare_int_float_eq(256, 256.0f) must be TRUE.
/// 256 as i8 = 0, so old code compared 0.0f == 256.0f = FALSE (wrong).
#[test]
fn test_int_float_eq_256_is_true() {
    // 256.0f in f32: 0x43800000
    let f = XsdFloat::from_bits(0x43800000);
    assert(compare_int_float_eq(256, f));
}

/// compare_int_float_eq(1000, 0.0f) must be FALSE.
/// 1000 as i8 = -24, so old code compared -24.0f == 0.0f = FALSE (accidentally correct)
/// but compare_int_float_eq(256, 0.0f) = (0.0f == 0.0f) = TRUE (wrong). Explicit:
#[test]
fn test_int_float_eq_256_vs_zero_is_false() {
    let f = XsdFloat::zero();
    assert(!compare_int_float_eq(256, f));
}

/// compare_int_float_gt(300, 0.0f) must be TRUE.
/// 300 as i8 = 44, so old code gave 44.0f > 0.0f = TRUE (correct by coincidence),
/// but this is a direct correctness guard for the exact conversion.
#[test]
fn test_int_float_gt_300_vs_zero() {
    let f = XsdFloat::zero();
    assert(compare_int_float_gt(300, f));
}

/// compare_float_int_eq(1000.0f, 1000) must be TRUE.
/// 1000 as i8 = -24, so old code compared 1000.0f == -24.0f = FALSE (wrong).
#[test]
fn test_float_int_eq_1000_is_true() {
    // 1000.0f in f32: 0x447A0000
    let f = XsdFloat::from_bits(0x447A0000);
    assert(compare_float_int_eq(f, 1000));
}

/// compare_float_int_lt(0.0f, 256) must be TRUE.
/// 256 as i8 = 0, so old code compared 0.0f < 0.0f = FALSE (wrong).
#[test]
fn test_float_int_lt_zero_lt_256() {
    let f = XsdFloat::zero();
    assert(compare_float_int_lt(f, 256));
}

// ----------------------------------------------------------------------------
// IEEE preservation: NaN and signed-zero semantics must be unaffected
// ----------------------------------------------------------------------------

/// NaN vs integer: all comparisons must return false (NaN is unordered).
#[test]
fn test_nan_vs_integer_all_false() {
    let nan = XsdDouble::nan();
    assert(!compare_double_int_eq(nan, 0));
    assert(!compare_double_int_lt(nan, 0));
    assert(!compare_double_int_gt(nan, 0) );
    assert(!compare_double_int_le(nan, 0));
    assert(!compare_double_int_ge(nan, 0));
}

/// +0.0 == integer 0 must be TRUE.
#[test]
fn test_pos_zero_eq_integer_zero() {
    assert(compare_double_int_eq(XsdDouble::zero(), 0));
}

/// -0.0 == integer 0 must also be TRUE (IEEE: -0.0 == +0.0).
#[test]
fn test_neg_zero_eq_integer_zero() {
    assert(compare_double_int_eq(XsdDouble::neg_zero(), 0));
}

// ----------------------------------------------------------------------------
// Bug C fix (sq-3x7dl.5): denormal f32->f64 promotion must be exact [SONNET-4.6]
// ----------------------------------------------------------------------------

/// The smallest denormal f32 (bits 0x00000001 = 2^-149) promoted to f64
/// must equal 2^-149 in f64 (bits 0x36A0000000000000).
/// Old code mapped all denormals to signed zero - this would give 0.0 != 2^-149.
#[test]
fn test_from_float_smallest_denorm_is_exact() {
    let f32_denorm = XsdFloat::from_bits(0x00000001); // 2^-149
    let promoted = XsdDouble::from_float(f32_denorm);
    // 2^-149 in f64
    let expected = XsdDouble::from_bits(0x36A0000000000000);
    assert(numeric_equal_double(promoted, expected));
}

/// A larger denormal f32 (bits 0x00000003 = 3 * 2^-149 = 1.5 * 2^-148).
/// f64 bits: biased exp = 1 + 874 = 875 = 0x36B; mantissa = 1 << 51 = 0x0008000000000000
///           combined: 0x36B8000000000000
#[test]
fn test_from_float_denorm_with_lower_bits_is_exact() {
    let f32_denorm = XsdFloat::from_bits(0x00000003); // 3 * 2^-149
    let promoted = XsdDouble::from_float(f32_denorm);
    let expected = XsdDouble::from_bits(0x36B8000000000000);
    assert(numeric_equal_double(promoted, expected));
}

/// Negative denormal f32 (bits 0x80000001 = -2^-149).
#[test]
fn test_from_float_negative_denorm_is_exact() {
    let f32_denorm = XsdFloat::from_bits(0x80000001); // -2^-149
    let promoted = XsdDouble::from_float(f32_denorm);
    // -2^-149 in f64: sign bit set
    let expected = XsdDouble::from_bits(0xB6A0000000000000);
    assert(numeric_equal_double(promoted, expected));
}

/// Denorm with multi-bit mantissa: f32 bits 0x003FFFFF
/// mant32 = 0x3FFFFF (bits 21..0 all set), exp32 = 0, sign = 0.
/// Derivation:
///   k = 21  (highest set bit of 0x3FFFFF is bit 21 = 0x200000)
///   lower = 0x3FFFFF ^ 0x200000 = 0x1FFFFF
///   f64 biased exp = 21 + 874 = 895 = 0x37F
///   f64 mant = 0x1FFFFF as u64 << 31 = 0x000F_FFFF_8000_0000
///   f64 bits = 0x37F0_0000_0000_0000 | 0x000F_FFFF_8000_0000
///            = 0x37FF_FFFF_8000_0000
#[test]
fn test_from_float_large_denorm_mantissa() {
    let f32_denorm = XsdFloat::from_bits(0x003FFFFF);
    let promoted = XsdDouble::from_float(f32_denorm);
    // 0x37FF_FFFF_8000_0000 = (895 << 52) | (0x1FFFFF_u64 << 31)
    let expected = XsdDouble::from_bits(0x37FFFFFF80000000);
    assert(numeric_equal_double(promoted, expected));
}

/// Largest f32 denormal: bits 0x007FFFFF, all 23 mantissa bits set.
/// This exercises k=22 (the first branch of f32_denorm_leading_bit).
/// Derivation:
///   mant32 = 0x7FFFFF, bit 22 (0x400000) is set -> k = 22
///   lower  = 0x7FFFFF ^ 0x400000 = 0x3FFFFF
///   f64 biased exp = 22 + 874 = 896 = 0x380
///   f64 mant = 0x3FFFFF << 30 = 0x000FFFFFC0000000
///   f64 bits = 0x3800000000000000 | 0x000FFFFFC0000000
///            = 0x380FFFFFC0000000
/// Mutation guard: changing the k=22 branch to return 21 gives wrong exp+mant,
/// causing this assertion to fail. [SONNET-4.6]
#[test]
fn test_from_float_largest_denorm_k22() {
    let f32_denorm = XsdFloat::from_bits(0x007FFFFF); // largest f32 denormal
    let promoted = XsdDouble::from_float(f32_denorm);
    let expected = XsdDouble::from_bits(0x380FFFFFC0000000);
    assert(numeric_equal_double(promoted, expected));
}

/// Denormal f32 with mantissa 0x100001 (bits 20 and 0 set).
/// This exercises k=20 (bit 20 is the highest set bit).
/// Derivation:
///   mant32 = 0x100001, bits 22 and 21 are clear -> k = 20
///   lower  = 0x100001 ^ 0x100000 = 0x000001
///   f64 biased exp = 20 + 874 = 894 = 0x37E
///   f64 mant = 0x000001 << 32 = 0x0000000100000000
///   f64 bits = 0x37E0000000000000 | 0x0000000100000000
///            = 0x37E0000100000000
/// Value check: (2^20 + 1) * 2^-149 = 2^-129 + 2^-149. [SONNET-4.6]
#[test]
fn test_from_float_denorm_k20() {
    let f32_denorm = XsdFloat::from_bits(0x00100001); // mant = 0x100001, k=20
    let promoted = XsdDouble::from_float(f32_denorm);
    let expected = XsdDouble::from_bits(0x37E0000100000000);
    assert(numeric_equal_double(promoted, expected));
}

// ============================================================================
// fn:number string parsing -- whitespace handling (xsd whiteSpace="collapse":
// leading/trailing #x20/#x9/#xA/#xD stripped; INTERIOR whitespace is not in
// the xs:double lexical space, so it must yield the NaN indicator). [FABLE-5]
// ============================================================================

#[test]
fn test_fn_number_from_string_basic() {
    let (v, ok) = fn_number_from_string("42");
    assert(ok);
    assert(v == 42);
    let (v2, ok2) = fn_number_from_string("-13");
    assert(ok2);
    assert(v2 == -13);
    let (v3, ok3) = fn_number_from_string("+7");
    assert(ok3);
    assert(v3 == 7);
}

#[test]
fn test_fn_number_from_string_leading_trailing_whitespace() {
    // Leading + trailing whitespace is stripped (collapse facet)
    let (v, ok) = fn_number_from_string(" 42 ");
    assert(ok);
    assert(v == 42);
    let (v2, ok2) = fn_number_from_string("\t-13\n");
    assert(ok2);
    assert(v2 == -13);
    let (v3, ok3) = fn_number_from_string("  99  ");
    assert(ok3);
    assert(v3 == 99);
}

#[test]
fn test_fn_number_from_string_interior_whitespace_is_nan() {
    // fn:number('1 2') must be NaN -- NOT 12 (the pre-fix bug folded digits
    // across interior whitespace)
    let (_, ok) = fn_number_from_string("1 2");
    assert(!ok);
    let (_, ok2) = fn_number_from_string("4 2 ");
    assert(!ok2);
    // whitespace between sign and digits is interior too
    let (_, ok3) = fn_number_from_string("- 5");
    assert(!ok3);
}

#[test]
fn test_fn_number_from_string_junk_after_trailing_whitespace_is_nan() {
    // Once the trailing-whitespace run starts, any non-whitespace is invalid
    let (_, ok) = fn_number_from_string("7 x");
    assert(!ok);
    let (_, ok2) = fn_number_from_string("12 .");
    assert(!ok2);
}

#[test]
fn test_fn_number_from_string_malformed_is_nan() {
    let (_, ok) = fn_number_from_string("12x");
    assert(!ok);
    let (_, ok2) = fn_number_from_string("x12");
    assert(!ok2);
    let (_, ok3) = fn_number_from_string("   ");
    assert(!ok3);
    let (_, ok4) = fn_number_from_string("-");
    assert(!ok4);
}
