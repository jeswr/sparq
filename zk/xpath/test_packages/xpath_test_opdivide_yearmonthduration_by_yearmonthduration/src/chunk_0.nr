//! Test chunk 0 for op:divide-yearMonthDuration-by-yearMonthDuration
//! Contains 7 tests

use dep::xpath::{
    ym_duration_divide_by_duration,
    XsdYearMonthDuration,
    ym_duration_equal,
    ym_duration_from_string,
    XsdDouble,
};

#[test]
fn op_divide_yearmonthduration_by_yearmonthduration2args_1() {
    // Test: op-divide-yearMonthDuration-by-yearMonthDuration2args-1
    // Written By: Carmelo Montanez
    // Date: Tue Apr 12 16:29:08 GMT-05:00 2005
    // Purpose: Evaluates The 'op:divide-yearMonthDuration-by-yearMonthDuration' operator
    // with the arguments set as follows:
    // $arg1 = xs:yearMonthDuration(lower bound)
    // $arg2 = xs:yearMonthDuration(lower bound)
    // XPath: xs:yearMonthDuration("P0Y0M") div xs:yearMonthDuration("P0Y1M")
    // Expected: assert-eq 0
    assert(ym_duration_divide_by_duration(XsdYearMonthDuration::new(0), XsdYearMonthDuration::new(1)) == XsdDouble::zero());
    // Additional assertions: verify string parsing produces expected durations
    let (parsed_ymd_0, parse_ok_0) = ym_duration_from_string("P0Y0M".as_bytes(), 5);
    assert(parse_ok_0);
    assert(ym_duration_equal(parsed_ymd_0, XsdYearMonthDuration::new(0)));
    let (parsed_ymd_1, parse_ok_1) = ym_duration_from_string("P0Y1M".as_bytes(), 5);
    assert(parse_ok_1);
    assert(ym_duration_equal(parsed_ymd_1, XsdYearMonthDuration::new(1)));
}

#[test]
fn op_divide_yearmonthduration_by_yearmonthduration2args_2() {
    // Test: op-divide-yearMonthDuration-by-yearMonthDuration2args-2
    // Written By: Carmelo Montanez
    // Date: Tue Apr 12 16:29:08 GMT-05:00 2005
    // Purpose: Evaluates The 'op:divide-yearMonthDuration-by-yearMonthDuration' operator
    // with the arguments set as follows:
    // $arg1 = xs:yearMonthDuration(mid range)
    // $arg2 = xs:yearMonthDuration(lower bound)
    // XPath: xs:yearMonthDuration("P1000Y6M") div xs:yearMonthDuration("P0Y1M")
    // Expected: assert-eq 12006
    assert(ym_duration_divide_by_duration(XsdYearMonthDuration::new(12006), XsdYearMonthDuration::new(1)).to_bits() == 4667825982630002688);
    // Additional assertions: verify string parsing produces expected durations
    let (parsed_ymd_0, parse_ok_0) = ym_duration_from_string("P1000Y6M".as_bytes(), 8);
    assert(parse_ok_0);
    assert(ym_duration_equal(parsed_ymd_0, XsdYearMonthDuration::new(12006)));
    let (parsed_ymd_1, parse_ok_1) = ym_duration_from_string("P0Y1M".as_bytes(), 5);
    assert(parse_ok_1);
    assert(ym_duration_equal(parsed_ymd_1, XsdYearMonthDuration::new(1)));
}

#[test]
fn op_divide_yearmonthduration_by_yearmonthduration2args_3() {
    // Test: op-divide-yearMonthDuration-by-yearMonthDuration2args-3
    // Written By: Carmelo Montanez
    // Date: Tue Apr 12 16:29:08 GMT-05:00 2005
    // Purpose: Evaluates The 'op:divide-yearMonthDuration-by-yearMonthDuration' operator
    // with the arguments set as follows:
    // $arg1 = xs:yearMonthDuration(upper bound)
    // $arg2 = xs:yearMonthDuration(lower bound)
    // XPath: xs:yearMonthDuration("P2030Y12M") div xs:yearMonthDuration("P0Y1M")
    // Expected: assert-eq 24372
    assert(ym_duration_divide_by_duration(XsdYearMonthDuration::new(24372), XsdYearMonthDuration::new(1)).to_bits() == 4672428538303873024);
    // Additional assertions: verify string parsing produces expected durations
    let (parsed_ymd_0, parse_ok_0) = ym_duration_from_string("P2030Y12M".as_bytes(), 9);
    assert(parse_ok_0);
    assert(ym_duration_equal(parsed_ymd_0, XsdYearMonthDuration::new(24372)));
    let (parsed_ymd_1, parse_ok_1) = ym_duration_from_string("P0Y1M".as_bytes(), 5);
    assert(parse_ok_1);
    assert(ym_duration_equal(parsed_ymd_1, XsdYearMonthDuration::new(1)));
}

#[test]
fn op_divide_yearmonthduration_by_yearmonthduration2args_4() {
    // Test: op-divide-yearMonthDuration-by-yearMonthDuration2args-4
    // Written By: Carmelo Montanez
    // Date: Tue Apr 12 16:29:08 GMT-05:00 2005
    // Purpose: Evaluates The 'op:divide-yearMonthDuration-by-yearMonthDuration' operator
    // with the arguments set as follows:
    // $arg1 = xs:yearMonthDuration(lower bound)
    // $arg2 = xs:yearMonthDuration(mid range)
    // XPath: xs:yearMonthDuration("P0Y0M") div xs:yearMonthDuration("P1000Y6M")
    // Expected: assert-eq 0
    assert(ym_duration_divide_by_duration(XsdYearMonthDuration::new(0), XsdYearMonthDuration::new(12006)) == XsdDouble::zero());
    // Additional assertions: verify string parsing produces expected durations
    let (parsed_ymd_0, parse_ok_0) = ym_duration_from_string("P0Y0M".as_bytes(), 5);
    assert(parse_ok_0);
    assert(ym_duration_equal(parsed_ymd_0, XsdYearMonthDuration::new(0)));
    let (parsed_ymd_1, parse_ok_1) = ym_duration_from_string("P1000Y6M".as_bytes(), 8);
    assert(parse_ok_1);
    assert(ym_duration_equal(parsed_ymd_1, XsdYearMonthDuration::new(12006)));
}

#[test]
fn op_divide_yearmonthduration_by_yearmonthduration2args_5() {
    // Test: op-divide-yearMonthDuration-by-yearMonthDuration2args-5
    // Written By: Carmelo Montanez
    // Date: Tue Apr 12 16:29:08 GMT-05:00 2005
    // Purpose: Evaluates The 'op:divide-yearMonthDuration-by-yearMonthDuration' operator
    // with the arguments set as follows:
    // $arg1 = xs:yearMonthDuration(lower bound)
    // $arg2 = xs:yearMonthDuration(upper bound)
    // XPath: xs:yearMonthDuration("P0Y0M") div xs:yearMonthDuration("P2030Y12M")
    // Expected: assert-eq 0
    assert(ym_duration_divide_by_duration(XsdYearMonthDuration::new(0), XsdYearMonthDuration::new(24372)) == XsdDouble::zero());
    // Additional assertions: verify string parsing produces expected durations
    let (parsed_ymd_0, parse_ok_0) = ym_duration_from_string("P0Y0M".as_bytes(), 5);
    assert(parse_ok_0);
    assert(ym_duration_equal(parsed_ymd_0, XsdYearMonthDuration::new(0)));
    let (parsed_ymd_1, parse_ok_1) = ym_duration_from_string("P2030Y12M".as_bytes(), 9);
    assert(parse_ok_1);
    assert(ym_duration_equal(parsed_ymd_1, XsdYearMonthDuration::new(24372)));
}

#[test]
fn op_divide_yearmonthduration_by_ymd_1() {
    // Test: op-divide-yearMonthDuration-by-YMD-1
    // Written By: Carmelo Montanez
    // Date: June 30, 2005
    // Purpose: Evaluates The 'divide-yearMonthDuration-by-YMD' operator
    // As per example 1 (for this function)of the F&O specs.
    // XPath: xs:yearMonthDuration("P3Y4M") div xs:yearMonthDuration("-P1Y4M")
    // Expected: assert-eq -2.5
    assert(ym_duration_divide_by_duration(XsdYearMonthDuration::new(40), XsdYearMonthDuration::new(-16)).to_bits() == 13836183955189006336);
    // Additional assertions: verify string parsing produces expected durations
    let (parsed_ymd_0, parse_ok_0) = ym_duration_from_string("P3Y4M".as_bytes(), 5);
    assert(parse_ok_0);
    assert(ym_duration_equal(parsed_ymd_0, XsdYearMonthDuration::new(40)));
    let (parsed_ymd_1, parse_ok_1) = ym_duration_from_string("-P1Y4M".as_bytes(), 6);
    assert(parse_ok_1);
    assert(ym_duration_equal(parsed_ymd_1, XsdYearMonthDuration::new(-16)));
}

#[test]
fn op_divide_yearmonthduration_by_ymd_8() {
    // Test: op-divide-yearMonthDuration-by-YMD-8
    // Written By: Carmelo Montanez
    // Date: June 30, 2005
    // Purpose: Evaluates The 'divide-yearMonthDuration-by-YMD' operator that
    // returns a negative value.
    // XPath: (xs:yearMonthDuration("P10Y01M") div xs:yearMonthDuration("-P10Y01M"))
    // Expected: assert-eq -1
    assert(ym_duration_divide_by_duration(XsdYearMonthDuration::new(121), XsdYearMonthDuration::new(-121)).to_bits() == 13830554455654793216);
    // Additional assertions: verify string parsing produces expected durations
    let (parsed_ymd_0, parse_ok_0) = ym_duration_from_string("P10Y01M".as_bytes(), 7);
    assert(parse_ok_0);
    assert(ym_duration_equal(parsed_ymd_0, XsdYearMonthDuration::new(121)));
    let (parsed_ymd_1, parse_ok_1) = ym_duration_from_string("-P10Y01M".as_bytes(), 8);
    assert(parse_ok_1);
    assert(ym_duration_equal(parsed_ymd_1, XsdYearMonthDuration::new(-121)));
}
