//! Test chunk 0 for op:duration-equal
//! Contains 36 tests

use dep::xpath::{
    duration_equal,
    XsdYearMonthDuration,
    ym_duration_equal,
    ym_duration_le,
    ym_duration_ge,
    ym_duration_from_string,
    duration_from_microseconds,
};

#[test]
fn op_yearmonthduration_equal2args_1() {
    // Evaluates The 'op:yearMonthDuration-equal' operator with the arguments set as follows: $arg1 = xs:yearMonthDuration(lower bound) $arg2 = xs:yearMonthDuration(lower bound)
    // XPath: xs:yearMonthDuration("P0Y0M") eq xs:yearMonthDuration("P0Y0M")
    // Expected: assert-true true
    assert(ym_duration_equal(XsdYearMonthDuration::new(0), XsdYearMonthDuration::new(0)) == true);
    // Additional assertions: verify string parsing produces expected durations
    let (parsed_ymd_0, parse_ok_0) = ym_duration_from_string("P0Y0M".as_bytes(), 5);
    assert(parse_ok_0);
    assert(ym_duration_equal(parsed_ymd_0, XsdYearMonthDuration::new(0)));
    let (parsed_ymd_1, parse_ok_1) = ym_duration_from_string("P0Y0M".as_bytes(), 5);
    assert(parse_ok_1);
    assert(ym_duration_equal(parsed_ymd_1, XsdYearMonthDuration::new(0)));
}

#[test]
fn op_yearmonthduration_equal2args_2() {
    // Evaluates The 'op:yearMonthDuration-equal' operator with the arguments set as follows: $arg1 = xs:yearMonthDuration(mid range) $arg2 = xs:yearMonthDuration(lower bound)
    // XPath: xs:yearMonthDuration("P1000Y6M") eq xs:yearMonthDuration("P0Y0M")
    // Expected: assert-false false
    assert(ym_duration_equal(XsdYearMonthDuration::new(12006), XsdYearMonthDuration::new(0)) == false);
    // Additional assertions: verify string parsing produces expected durations
    let (parsed_ymd_0, parse_ok_0) = ym_duration_from_string("P1000Y6M".as_bytes(), 8);
    assert(parse_ok_0);
    assert(ym_duration_equal(parsed_ymd_0, XsdYearMonthDuration::new(12006)));
    let (parsed_ymd_1, parse_ok_1) = ym_duration_from_string("P0Y0M".as_bytes(), 5);
    assert(parse_ok_1);
    assert(ym_duration_equal(parsed_ymd_1, XsdYearMonthDuration::new(0)));
}

#[test]
fn op_yearmonthduration_equal2args_3() {
    // Evaluates The 'op:yearMonthDuration-equal' operator with the arguments set as follows: $arg1 = xs:yearMonthDuration(upper bound) $arg2 = xs:yearMonthDuration(lower bound)
    // XPath: xs:yearMonthDuration("P2030Y12M") eq xs:yearMonthDuration("P0Y0M")
    // Expected: assert-false false
    assert(ym_duration_equal(XsdYearMonthDuration::new(24372), XsdYearMonthDuration::new(0)) == false);
    // Additional assertions: verify string parsing produces expected durations
    let (parsed_ymd_0, parse_ok_0) = ym_duration_from_string("P2030Y12M".as_bytes(), 9);
    assert(parse_ok_0);
    assert(ym_duration_equal(parsed_ymd_0, XsdYearMonthDuration::new(24372)));
    let (parsed_ymd_1, parse_ok_1) = ym_duration_from_string("P0Y0M".as_bytes(), 5);
    assert(parse_ok_1);
    assert(ym_duration_equal(parsed_ymd_1, XsdYearMonthDuration::new(0)));
}

#[test]
fn op_yearmonthduration_equal2args_4() {
    // Evaluates The 'op:yearMonthDuration-equal' operator with the arguments set as follows: $arg1 = xs:yearMonthDuration(lower bound) $arg2 = xs:yearMonthDuration(mid range)
    // XPath: xs:yearMonthDuration("P0Y0M") eq xs:yearMonthDuration("P1000Y6M")
    // Expected: assert-false false
    assert(ym_duration_equal(XsdYearMonthDuration::new(0), XsdYearMonthDuration::new(12006)) == false);
    // Additional assertions: verify string parsing produces expected durations
    let (parsed_ymd_0, parse_ok_0) = ym_duration_from_string("P0Y0M".as_bytes(), 5);
    assert(parse_ok_0);
    assert(ym_duration_equal(parsed_ymd_0, XsdYearMonthDuration::new(0)));
    let (parsed_ymd_1, parse_ok_1) = ym_duration_from_string("P1000Y6M".as_bytes(), 8);
    assert(parse_ok_1);
    assert(ym_duration_equal(parsed_ymd_1, XsdYearMonthDuration::new(12006)));
}

#[test]
fn op_yearmonthduration_equal2args_5() {
    // Evaluates The 'op:yearMonthDuration-equal' operator with the arguments set as follows: $arg1 = xs:yearMonthDuration(lower bound) $arg2 = xs:yearMonthDuration(upper bound)
    // XPath: xs:yearMonthDuration("P0Y0M") eq xs:yearMonthDuration("P2030Y12M")
    // Expected: assert-false false
    assert(ym_duration_equal(XsdYearMonthDuration::new(0), XsdYearMonthDuration::new(24372)) == false);
    // Additional assertions: verify string parsing produces expected durations
    let (parsed_ymd_0, parse_ok_0) = ym_duration_from_string("P0Y0M".as_bytes(), 5);
    assert(parse_ok_0);
    assert(ym_duration_equal(parsed_ymd_0, XsdYearMonthDuration::new(0)));
    let (parsed_ymd_1, parse_ok_1) = ym_duration_from_string("P2030Y12M".as_bytes(), 9);
    assert(parse_ok_1);
    assert(ym_duration_equal(parsed_ymd_1, XsdYearMonthDuration::new(24372)));
}

#[test]
fn op_yearmonthduration_equal2args_6() {
    // Evaluates The 'op:yearMonthDuration-equal' operator with the arguments set as follows: $arg1 = xs:yearMonthDuration(lower bound) $arg2 = xs:yearMonthDuration(lower bound)
    // XPath: xs:yearMonthDuration("P0Y0M") ne xs:yearMonthDuration("P0Y0M")
    // Expected: assert-false false
    assert(!ym_duration_equal(XsdYearMonthDuration::new(0), XsdYearMonthDuration::new(0)) == false);
    // Additional assertions: verify string parsing produces expected durations
    let (parsed_ymd_0, parse_ok_0) = ym_duration_from_string("P0Y0M".as_bytes(), 5);
    assert(parse_ok_0);
    assert(ym_duration_equal(parsed_ymd_0, XsdYearMonthDuration::new(0)));
    let (parsed_ymd_1, parse_ok_1) = ym_duration_from_string("P0Y0M".as_bytes(), 5);
    assert(parse_ok_1);
    assert(ym_duration_equal(parsed_ymd_1, XsdYearMonthDuration::new(0)));
}

#[test]
fn op_yearmonthduration_equal2args_7() {
    // Evaluates The 'op:yearMonthDuration-equal' operator with the arguments set as follows: $arg1 = xs:yearMonthDuration(mid range) $arg2 = xs:yearMonthDuration(lower bound)
    // XPath: xs:yearMonthDuration("P1000Y6M") ne xs:yearMonthDuration("P0Y0M")
    // Expected: assert-true true
    assert(!ym_duration_equal(XsdYearMonthDuration::new(12006), XsdYearMonthDuration::new(0)) == true);
    // Additional assertions: verify string parsing produces expected durations
    let (parsed_ymd_0, parse_ok_0) = ym_duration_from_string("P1000Y6M".as_bytes(), 8);
    assert(parse_ok_0);
    assert(ym_duration_equal(parsed_ymd_0, XsdYearMonthDuration::new(12006)));
    let (parsed_ymd_1, parse_ok_1) = ym_duration_from_string("P0Y0M".as_bytes(), 5);
    assert(parse_ok_1);
    assert(ym_duration_equal(parsed_ymd_1, XsdYearMonthDuration::new(0)));
}

#[test]
fn op_yearmonthduration_equal2args_8() {
    // Evaluates The 'op:yearMonthDuration-equal' operator with the arguments set as follows: $arg1 = xs:yearMonthDuration(upper bound) $arg2 = xs:yearMonthDuration(lower bound)
    // XPath: xs:yearMonthDuration("P2030Y12M") ne xs:yearMonthDuration("P0Y0M")
    // Expected: assert-true true
    assert(!ym_duration_equal(XsdYearMonthDuration::new(24372), XsdYearMonthDuration::new(0)) == true);
    // Additional assertions: verify string parsing produces expected durations
    let (parsed_ymd_0, parse_ok_0) = ym_duration_from_string("P2030Y12M".as_bytes(), 9);
    assert(parse_ok_0);
    assert(ym_duration_equal(parsed_ymd_0, XsdYearMonthDuration::new(24372)));
    let (parsed_ymd_1, parse_ok_1) = ym_duration_from_string("P0Y0M".as_bytes(), 5);
    assert(parse_ok_1);
    assert(ym_duration_equal(parsed_ymd_1, XsdYearMonthDuration::new(0)));
}

#[test]
fn op_yearmonthduration_equal2args_9() {
    // Evaluates The 'op:yearMonthDuration-equal' operator with the arguments set as follows: $arg1 = xs:yearMonthDuration(lower bound) $arg2 = xs:yearMonthDuration(mid range)
    // XPath: xs:yearMonthDuration("P0Y0M") ne xs:yearMonthDuration("P1000Y6M")
    // Expected: assert-true true
    assert(!ym_duration_equal(XsdYearMonthDuration::new(0), XsdYearMonthDuration::new(12006)) == true);
    // Additional assertions: verify string parsing produces expected durations
    let (parsed_ymd_0, parse_ok_0) = ym_duration_from_string("P0Y0M".as_bytes(), 5);
    assert(parse_ok_0);
    assert(ym_duration_equal(parsed_ymd_0, XsdYearMonthDuration::new(0)));
    let (parsed_ymd_1, parse_ok_1) = ym_duration_from_string("P1000Y6M".as_bytes(), 8);
    assert(parse_ok_1);
    assert(ym_duration_equal(parsed_ymd_1, XsdYearMonthDuration::new(12006)));
}

#[test]
fn op_yearmonthduration_equal2args_10() {
    // Evaluates The 'op:yearMonthDuration-equal' operator with the arguments set as follows: $arg1 = xs:yearMonthDuration(lower bound) $arg2 = xs:yearMonthDuration(upper bound)
    // XPath: xs:yearMonthDuration("P0Y0M") ne xs:yearMonthDuration("P2030Y12M")
    // Expected: assert-true true
    assert(!ym_duration_equal(XsdYearMonthDuration::new(0), XsdYearMonthDuration::new(24372)) == true);
    // Additional assertions: verify string parsing produces expected durations
    let (parsed_ymd_0, parse_ok_0) = ym_duration_from_string("P0Y0M".as_bytes(), 5);
    assert(parse_ok_0);
    assert(ym_duration_equal(parsed_ymd_0, XsdYearMonthDuration::new(0)));
    let (parsed_ymd_1, parse_ok_1) = ym_duration_from_string("P2030Y12M".as_bytes(), 9);
    assert(parse_ok_1);
    assert(ym_duration_equal(parsed_ymd_1, XsdYearMonthDuration::new(24372)));
}

#[test]
fn op_yearmonthduration_equal2args_11() {
    // Evaluates The 'op:yearMonthDuration-equal' operator (le) with the arguments set as follows: $arg1 = xs:yearMonthDuration(lower bound) $arg2 = xs:yearMonthDuration(lower bound)
    // XPath: xs:yearMonthDuration("P0Y0M") le xs:yearMonthDuration("P0Y0M")
    // Expected: assert-true true
    assert(ym_duration_le(XsdYearMonthDuration::new(0), XsdYearMonthDuration::new(0)) == true);
    // Additional assertions: verify string parsing produces expected durations
    let (parsed_ymd_0, parse_ok_0) = ym_duration_from_string("P0Y0M".as_bytes(), 5);
    assert(parse_ok_0);
    assert(ym_duration_equal(parsed_ymd_0, XsdYearMonthDuration::new(0)));
    let (parsed_ymd_1, parse_ok_1) = ym_duration_from_string("P0Y0M".as_bytes(), 5);
    assert(parse_ok_1);
    assert(ym_duration_equal(parsed_ymd_1, XsdYearMonthDuration::new(0)));
}

#[test]
fn op_yearmonthduration_equal2args_12() {
    // Evaluates The 'op:yearMonthDuration-equal' operator (le) with the arguments set as follows: $arg1 = xs:yearMonthDuration(mid range) $arg2 = xs:yearMonthDuration(lower bound)
    // XPath: xs:yearMonthDuration("P1000Y6M") le xs:yearMonthDuration("P0Y0M")
    // Expected: assert-false false
    assert(ym_duration_le(XsdYearMonthDuration::new(12006), XsdYearMonthDuration::new(0)) == false);
    // Additional assertions: verify string parsing produces expected durations
    let (parsed_ymd_0, parse_ok_0) = ym_duration_from_string("P1000Y6M".as_bytes(), 8);
    assert(parse_ok_0);
    assert(ym_duration_equal(parsed_ymd_0, XsdYearMonthDuration::new(12006)));
    let (parsed_ymd_1, parse_ok_1) = ym_duration_from_string("P0Y0M".as_bytes(), 5);
    assert(parse_ok_1);
    assert(ym_duration_equal(parsed_ymd_1, XsdYearMonthDuration::new(0)));
}

#[test]
fn op_yearmonthduration_equal2args_13() {
    // Evaluates The 'op:yearMonthDuration-equal' operator (le) with the arguments set as follows: $arg1 = xs:yearMonthDuration(upper bound) $arg2 = xs:yearMonthDuration(lower bound)
    // XPath: xs:yearMonthDuration("P2030Y12M") le xs:yearMonthDuration("P0Y0M")
    // Expected: assert-false false
    assert(ym_duration_le(XsdYearMonthDuration::new(24372), XsdYearMonthDuration::new(0)) == false);
    // Additional assertions: verify string parsing produces expected durations
    let (parsed_ymd_0, parse_ok_0) = ym_duration_from_string("P2030Y12M".as_bytes(), 9);
    assert(parse_ok_0);
    assert(ym_duration_equal(parsed_ymd_0, XsdYearMonthDuration::new(24372)));
    let (parsed_ymd_1, parse_ok_1) = ym_duration_from_string("P0Y0M".as_bytes(), 5);
    assert(parse_ok_1);
    assert(ym_duration_equal(parsed_ymd_1, XsdYearMonthDuration::new(0)));
}

#[test]
fn op_yearmonthduration_equal2args_14() {
    // Evaluates The 'op:yearMonthDuration-equal' operator (le) with the arguments set as follows: $arg1 = xs:yearMonthDuration(lower bound) $arg2 = xs:yearMonthDuration(mid range)
    // XPath: xs:yearMonthDuration("P0Y0M") le xs:yearMonthDuration("P1000Y6M")
    // Expected: assert-true true
    assert(ym_duration_le(XsdYearMonthDuration::new(0), XsdYearMonthDuration::new(12006)) == true);
    // Additional assertions: verify string parsing produces expected durations
    let (parsed_ymd_0, parse_ok_0) = ym_duration_from_string("P0Y0M".as_bytes(), 5);
    assert(parse_ok_0);
    assert(ym_duration_equal(parsed_ymd_0, XsdYearMonthDuration::new(0)));
    let (parsed_ymd_1, parse_ok_1) = ym_duration_from_string("P1000Y6M".as_bytes(), 8);
    assert(parse_ok_1);
    assert(ym_duration_equal(parsed_ymd_1, XsdYearMonthDuration::new(12006)));
}

#[test]
fn op_yearmonthduration_equal2args_15() {
    // Evaluates The 'op:yearMonthDuration-equal' operator (le) with the arguments set as follows: $arg1 = xs:yearMonthDuration(lower bound) $arg2 = xs:yearMonthDuration(upper bound)
    // XPath: xs:yearMonthDuration("P0Y0M") le xs:yearMonthDuration("P2030Y12M")
    // Expected: assert-true true
    assert(ym_duration_le(XsdYearMonthDuration::new(0), XsdYearMonthDuration::new(24372)) == true);
    // Additional assertions: verify string parsing produces expected durations
    let (parsed_ymd_0, parse_ok_0) = ym_duration_from_string("P0Y0M".as_bytes(), 5);
    assert(parse_ok_0);
    assert(ym_duration_equal(parsed_ymd_0, XsdYearMonthDuration::new(0)));
    let (parsed_ymd_1, parse_ok_1) = ym_duration_from_string("P2030Y12M".as_bytes(), 9);
    assert(parse_ok_1);
    assert(ym_duration_equal(parsed_ymd_1, XsdYearMonthDuration::new(24372)));
}

#[test]
fn op_yearmonthduration_equal2args_16() {
    // Evaluates The 'op:yearMonthDuration-equal' operator (ge) with the arguments set as follows: $arg1 = xs:yearMonthDuration(lower bound) $arg2 = xs:yearMonthDuration(lower bound)
    // XPath: xs:yearMonthDuration("P0Y0M") ge xs:yearMonthDuration("P0Y0M")
    // Expected: assert-true true
    assert(ym_duration_ge(XsdYearMonthDuration::new(0), XsdYearMonthDuration::new(0)) == true);
    // Additional assertions: verify string parsing produces expected durations
    let (parsed_ymd_0, parse_ok_0) = ym_duration_from_string("P0Y0M".as_bytes(), 5);
    assert(parse_ok_0);
    assert(ym_duration_equal(parsed_ymd_0, XsdYearMonthDuration::new(0)));
    let (parsed_ymd_1, parse_ok_1) = ym_duration_from_string("P0Y0M".as_bytes(), 5);
    assert(parse_ok_1);
    assert(ym_duration_equal(parsed_ymd_1, XsdYearMonthDuration::new(0)));
}

#[test]
fn op_yearmonthduration_equal2args_17() {
    // Evaluates The 'op:yearMonthDuration-equal' operator (ge) with the arguments set as follows: $arg1 = xs:yearMonthDuration(mid range) $arg2 = xs:yearMonthDuration(lower bound)
    // XPath: xs:yearMonthDuration("P1000Y6M") ge xs:yearMonthDuration("P0Y0M")
    // Expected: assert-true true
    assert(ym_duration_ge(XsdYearMonthDuration::new(12006), XsdYearMonthDuration::new(0)) == true);
    // Additional assertions: verify string parsing produces expected durations
    let (parsed_ymd_0, parse_ok_0) = ym_duration_from_string("P1000Y6M".as_bytes(), 8);
    assert(parse_ok_0);
    assert(ym_duration_equal(parsed_ymd_0, XsdYearMonthDuration::new(12006)));
    let (parsed_ymd_1, parse_ok_1) = ym_duration_from_string("P0Y0M".as_bytes(), 5);
    assert(parse_ok_1);
    assert(ym_duration_equal(parsed_ymd_1, XsdYearMonthDuration::new(0)));
}

#[test]
fn op_yearmonthduration_equal2args_18() {
    // Evaluates The 'op:yearMonthDuration-equal' operator (ge) with the arguments set as follows: $arg1 = xs:yearMonthDuration(upper bound) $arg2 = xs:yearMonthDuration(lower bound)
    // XPath: xs:yearMonthDuration("P2030Y12M") ge xs:yearMonthDuration("P0Y0M")
    // Expected: assert-true true
    assert(ym_duration_ge(XsdYearMonthDuration::new(24372), XsdYearMonthDuration::new(0)) == true);
    // Additional assertions: verify string parsing produces expected durations
    let (parsed_ymd_0, parse_ok_0) = ym_duration_from_string("P2030Y12M".as_bytes(), 9);
    assert(parse_ok_0);
    assert(ym_duration_equal(parsed_ymd_0, XsdYearMonthDuration::new(24372)));
    let (parsed_ymd_1, parse_ok_1) = ym_duration_from_string("P0Y0M".as_bytes(), 5);
    assert(parse_ok_1);
    assert(ym_duration_equal(parsed_ymd_1, XsdYearMonthDuration::new(0)));
}

#[test]
fn op_yearmonthduration_equal2args_19() {
    // Evaluates The 'op:yearMonthDuration-equal' operator (ge) with the arguments set as follows: $arg1 = xs:yearMonthDuration(lower bound) $arg2 = xs:yearMonthDuration(mid range)
    // XPath: xs:yearMonthDuration("P0Y0M") ge xs:yearMonthDuration("P1000Y6M")
    // Expected: assert-false false
    assert(ym_duration_ge(XsdYearMonthDuration::new(0), XsdYearMonthDuration::new(12006)) == false);
    // Additional assertions: verify string parsing produces expected durations
    let (parsed_ymd_0, parse_ok_0) = ym_duration_from_string("P0Y0M".as_bytes(), 5);
    assert(parse_ok_0);
    assert(ym_duration_equal(parsed_ymd_0, XsdYearMonthDuration::new(0)));
    let (parsed_ymd_1, parse_ok_1) = ym_duration_from_string("P1000Y6M".as_bytes(), 8);
    assert(parse_ok_1);
    assert(ym_duration_equal(parsed_ymd_1, XsdYearMonthDuration::new(12006)));
}

#[test]
fn op_yearmonthduration_equal2args_20() {
    // Evaluates The 'op:yearMonthDuration-equal' operator (ge) with the arguments set as follows: $arg1 = xs:yearMonthDuration(lower bound) $arg2 = xs:yearMonthDuration(upper bound)
    // XPath: xs:yearMonthDuration("P0Y0M") ge xs:yearMonthDuration("P2030Y12M")
    // Expected: assert-false false
    assert(ym_duration_ge(XsdYearMonthDuration::new(0), XsdYearMonthDuration::new(24372)) == false);
    // Additional assertions: verify string parsing produces expected durations
    let (parsed_ymd_0, parse_ok_0) = ym_duration_from_string("P0Y0M".as_bytes(), 5);
    assert(parse_ok_0);
    assert(ym_duration_equal(parsed_ymd_0, XsdYearMonthDuration::new(0)));
    let (parsed_ymd_1, parse_ok_1) = ym_duration_from_string("P2030Y12M".as_bytes(), 9);
    assert(parse_ok_1);
    assert(ym_duration_equal(parsed_ymd_1, XsdYearMonthDuration::new(24372)));
}

#[test]
fn op_daytimeduration_equal2args_1() {
    // Evaluates The 'op:dayTimeDuration-equal' operator with the arguments set as follows: $arg1 = xs:dayTimeDuration(lower bound) $arg2 = xs:dayTimeDuration(lower bound)
    // XPath: xs:dayTimeDuration("P0DT0H0M0S") eq xs:dayTimeDuration("P0DT0H0M0S")
    // Expected: assert-true true
    let dur1 = duration_from_microseconds(0);
        let dur2 = duration_from_microseconds(0);
    assert(duration_equal(dur1, dur2) == true);
}

#[test]
fn op_daytimeduration_equal2args_2() {
    // Evaluates The 'op:dayTimeDuration-equal' operator with the arguments set as follows: $arg1 = xs:dayTimeDuration(mid range) $arg2 = xs:dayTimeDuration(lower bound)
    // XPath: xs:dayTimeDuration("P15DT11H59M59S") eq xs:dayTimeDuration("P0DT0H0M0S")
    // Expected: assert-false false
    let dur1 = duration_from_microseconds(1339199000000);
        let dur2 = duration_from_microseconds(0);
    assert(duration_equal(dur1, dur2) == false);
}

#[test]
fn op_daytimeduration_equal2args_3() {
    // Evaluates The 'op:dayTimeDuration-equal' operator with the arguments set as follows: $arg1 = xs:dayTimeDuration(upper bound) $arg2 = xs:dayTimeDuration(lower bound)
    // XPath: xs:dayTimeDuration("P31DT23H59M59S") eq xs:dayTimeDuration("P0DT0H0M0S")
    // Expected: assert-false false
    let dur1 = duration_from_microseconds(2764799000000);
        let dur2 = duration_from_microseconds(0);
    assert(duration_equal(dur1, dur2) == false);
}

#[test]
fn op_daytimeduration_equal2args_4() {
    // Evaluates The 'op:dayTimeDuration-equal' operator with the arguments set as follows: $arg1 = xs:dayTimeDuration(lower bound) $arg2 = xs:dayTimeDuration(mid range)
    // XPath: xs:dayTimeDuration("P0DT0H0M0S") eq xs:dayTimeDuration("P15DT11H59M59S")
    // Expected: assert-false false
    let dur1 = duration_from_microseconds(0);
        let dur2 = duration_from_microseconds(1339199000000);
    assert(duration_equal(dur1, dur2) == false);
}

#[test]
fn op_daytimeduration_equal2args_5() {
    // Evaluates The 'op:dayTimeDuration-equal' operator with the arguments set as follows: $arg1 = xs:dayTimeDuration(lower bound) $arg2 = xs:dayTimeDuration(upper bound)
    // XPath: xs:dayTimeDuration("P0DT0H0M0S") eq xs:dayTimeDuration("P31DT23H59M59S")
    // Expected: assert-false false
    let dur1 = duration_from_microseconds(0);
        let dur2 = duration_from_microseconds(2764799000000);
    assert(duration_equal(dur1, dur2) == false);
}

#[test]
fn k_daytimedurationeq_1() {
    // Simple test of 'eq' for xs:dayTimeDuration, returning positive.
    // XPath: xs:dayTimeDuration("P3DT08H34M12.143S") eq xs:dayTimeDuration("P3DT08H34M12.143S")
    // Expected: assert-true true
    let dur1 = duration_from_microseconds(290052143000);
        let dur2 = duration_from_microseconds(290052143000);
    assert(duration_equal(dur1, dur2) == true);
}

#[test]
fn k_daytimedurationeq_2() {
    // Testing 'eq' involving xs:dayTimeDuration with two zeroed values.
    // XPath: xs:dayTimeDuration("PT0S") eq xs:dayTimeDuration("PT0S")
    // Expected: assert-true true
    let dur1 = duration_from_microseconds(0);
        let dur2 = duration_from_microseconds(0);
    assert(duration_equal(dur1, dur2) == true);
}

#[test]
fn k_daytimedurationeq_6() {
    // The xs:dayTimeDuration values -PT0S and PT0S are equal.
    // XPath: xs:dayTimeDuration("-PT0S") eq xs:dayTimeDuration("PT0S")
    // Expected: assert-true true
    let dur1 = duration_from_microseconds(0);
        let dur2 = duration_from_microseconds(0);
    assert(duration_equal(dur1, dur2) == true);
}

#[test]
fn k_daytimedurationeq_8() {
    // Test eq operator for xs:dayTimeDuration values with large milli second component.
    // XPath: xs:dayTimeDuration("P6DT16H34M59.613999S") eq xs:dayTimeDuration("P6DT16H34M59.613999S")
    // Expected: assert-true true
    let dur1 = duration_from_microseconds(578099613999);
        let dur2 = duration_from_microseconds(578099613999);
    assert(duration_equal(dur1, dur2) == true);
}

#[test]
fn k_yearmonthdurationeq_1() {
    // Simple test of 'eq' for xs:yearMonthDuration, returning positive.
    // XPath: xs:yearMonthDuration("P1999Y10M") eq xs:yearMonthDuration("P1999Y10M")
    // Expected: assert-true true
    assert(ym_duration_equal(XsdYearMonthDuration::new(23998), XsdYearMonthDuration::new(23998)) == true);
    // Additional assertions: verify string parsing produces expected durations
    let (parsed_ymd_0, parse_ok_0) = ym_duration_from_string("P1999Y10M".as_bytes(), 9);
    assert(parse_ok_0);
    assert(ym_duration_equal(parsed_ymd_0, XsdYearMonthDuration::new(23998)));
    let (parsed_ymd_1, parse_ok_1) = ym_duration_from_string("P1999Y10M".as_bytes(), 9);
    assert(parse_ok_1);
    assert(ym_duration_equal(parsed_ymd_1, XsdYearMonthDuration::new(23998)));
}

#[test]
fn k_yearmonthdurationeq_2() {
    // Testing 'eq' involving xs:yearMonthDuration with two zeroed values.
    // XPath: xs:yearMonthDuration("P0M") eq xs:yearMonthDuration("P0M")
    // Expected: assert-true true
    assert(ym_duration_equal(XsdYearMonthDuration::new(0), XsdYearMonthDuration::new(0)) == true);
    // Additional assertions: verify string parsing produces expected durations
    let (parsed_ymd_0, parse_ok_0) = ym_duration_from_string("P0M".as_bytes(), 3);
    assert(parse_ok_0);
    assert(ym_duration_equal(parsed_ymd_0, XsdYearMonthDuration::new(0)));
    let (parsed_ymd_1, parse_ok_1) = ym_duration_from_string("P0M".as_bytes(), 3);
    assert(parse_ok_1);
    assert(ym_duration_equal(parsed_ymd_1, XsdYearMonthDuration::new(0)));
}

#[test]
fn k_yearmonthdurationeq_3() {
    // Simple test of 'eq' for xs:yearMonthDuration.
    // XPath: not(xs:yearMonthDuration("P1999Y") eq xs:yearMonthDuration("P1999Y10M"))
    // Expected: assert-true true
    assert(!(ym_duration_equal(XsdYearMonthDuration::new(23988), XsdYearMonthDuration::new(23998))) == true);
    // Additional assertions: verify string parsing produces expected durations
    let (parsed_ymd_0, parse_ok_0) = ym_duration_from_string("P1999Y".as_bytes(), 6);
    assert(parse_ok_0);
    assert(ym_duration_equal(parsed_ymd_0, XsdYearMonthDuration::new(23988)));
    let (parsed_ymd_1, parse_ok_1) = ym_duration_from_string("P1999Y10M".as_bytes(), 9);
    assert(parse_ok_1);
    assert(ym_duration_equal(parsed_ymd_1, XsdYearMonthDuration::new(23998)));
}

#[test]
fn k_yearmonthdurationeq_4() {
    // Simple test of 'ne' for xs:yearMonthDuration.
    // XPath: xs:yearMonthDuration("P1999Y01M") ne xs:yearMonthDuration("P1999Y10M")
    // Expected: assert-true true
    assert(!ym_duration_equal(XsdYearMonthDuration::new(23989), XsdYearMonthDuration::new(23998)) == true);
    // Additional assertions: verify string parsing produces expected durations
    let (parsed_ymd_0, parse_ok_0) = ym_duration_from_string("P1999Y01M".as_bytes(), 9);
    assert(parse_ok_0);
    assert(ym_duration_equal(parsed_ymd_0, XsdYearMonthDuration::new(23989)));
    let (parsed_ymd_1, parse_ok_1) = ym_duration_from_string("P1999Y10M".as_bytes(), 9);
    assert(parse_ok_1);
    assert(ym_duration_equal(parsed_ymd_1, XsdYearMonthDuration::new(23998)));
}

#[test]
fn k_yearmonthdurationeq_5() {
    // Simple test of 'ne' for xs:yearMonthDuration.
    // XPath: not(xs:yearMonthDuration("P1999Y10M") ne xs:yearMonthDuration("P1999Y10M"))
    // Expected: assert-true true
    assert(!(!ym_duration_equal(XsdYearMonthDuration::new(23998), XsdYearMonthDuration::new(23998))) == true);
    // Additional assertions: verify string parsing produces expected durations
    let (parsed_ymd_0, parse_ok_0) = ym_duration_from_string("P1999Y10M".as_bytes(), 9);
    assert(parse_ok_0);
    assert(ym_duration_equal(parsed_ymd_0, XsdYearMonthDuration::new(23998)));
    let (parsed_ymd_1, parse_ok_1) = ym_duration_from_string("P1999Y10M".as_bytes(), 9);
    assert(parse_ok_1);
    assert(ym_duration_equal(parsed_ymd_1, XsdYearMonthDuration::new(23998)));
}

#[test]
fn k_yearmonthdurationeq_6() {
    // The xs:yearMonthDuration values -P0M and P0M are equal.
    // XPath: xs:yearMonthDuration("-P3Y8M") ne xs:yearMonthDuration("P3Y8M")
    // Expected: assert-true true
    assert(!ym_duration_equal(XsdYearMonthDuration::new(-44), XsdYearMonthDuration::new(44)) == true);
    // Additional assertions: verify string parsing produces expected durations
    let (parsed_ymd_0, parse_ok_0) = ym_duration_from_string("-P3Y8M".as_bytes(), 6);
    assert(parse_ok_0);
    assert(ym_duration_equal(parsed_ymd_0, XsdYearMonthDuration::new(-44)));
    let (parsed_ymd_1, parse_ok_1) = ym_duration_from_string("P3Y8M".as_bytes(), 5);
    assert(parse_ok_1);
    assert(ym_duration_equal(parsed_ymd_1, XsdYearMonthDuration::new(44)));
}

#[test]
fn k_yearmonthdurationeq_7() {
    // The xs:yearMonthDuration values -P3Y8M and +P3Y8M are not equal.
    // XPath: xs:yearMonthDuration("-P3Y8M") ne xs:yearMonthDuration("P3Y8M")
    // Expected: assert-true true
    assert(!ym_duration_equal(XsdYearMonthDuration::new(-44), XsdYearMonthDuration::new(44)) == true);
    // Additional assertions: verify string parsing produces expected durations
    let (parsed_ymd_0, parse_ok_0) = ym_duration_from_string("-P3Y8M".as_bytes(), 6);
    assert(parse_ok_0);
    assert(ym_duration_equal(parsed_ymd_0, XsdYearMonthDuration::new(-44)));
    let (parsed_ymd_1, parse_ok_1) = ym_duration_from_string("P3Y8M".as_bytes(), 5);
    assert(parse_ok_1);
    assert(ym_duration_equal(parsed_ymd_1, XsdYearMonthDuration::new(44)));
}
