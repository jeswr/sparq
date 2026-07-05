//! Test chunk 0 for op:multiply-yearMonthDuration
//! Contains 21 tests

use dep::xpath::{
    fn_string_from_ym_duration, XsdDouble, XsdYearMonthDuration, ym_duration_divide_by_duration,
    ym_duration_equal, ym_duration_ge, ym_duration_le, ym_duration_multiply, ym_duration_subtract,
};

#[test]
fn op_multiply_yearmonthduration2args_2() {
    //  ******************************************************* Test:
    // XPath: xs:yearMonthDuration("P1000Y6M") * xs:double("-0")
    // Expected: assert-eq P0M
    let dur = XsdYearMonthDuration::new(12006);
    assert(ym_duration_equal(ym_duration_multiply(dur, 0), XsdYearMonthDuration::new(0)));
}

#[test]
fn op_multiply_yearmonthduration2args_3() {
    //  ******************************************************* Test:
    // XPath: xs:yearMonthDuration("P2030Y12M") * xs:double("-0")
    // Expected: assert-eq P0M
    let dur = XsdYearMonthDuration::new(24372);
    assert(ym_duration_equal(ym_duration_multiply(dur, 0), XsdYearMonthDuration::new(0)));
}

#[test]
fn op_multiply_yearmonthduration2args_4() {
    //  ******************************************************* Test:
    // XPath: xs:yearMonthDuration("P0Y0M") * xs:double("0")
    // Expected: assert-eq P0M
    let dur = XsdYearMonthDuration::new(0);
    assert(ym_duration_equal(ym_duration_multiply(dur, 0), XsdYearMonthDuration::new(0)));
}

#[test]
fn op_multiply_yearmonthduration_2() {
    //  ******************************************************* Test:
    // XPath: fn:string((xs:yearMonthDuration("P10Y11M")) * 2.0) and fn:false()
    // Expected: assert-false false
    let dur_332 = ym_duration_multiply(XsdYearMonthDuration::new(131), 2);
    let (_str_bytes_332, str_len_332): ([u8; 32], u32) = fn_string_from_ym_duration(dur_332);
    assert((str_len_332 > 0) & (false) == false);
}

#[test]
fn op_multiply_yearmonthduration_3() {
    //  ******************************************************* Test:
    // XPath: fn:string((xs:yearMonthDuration("P20Y10M") * 2.0)) or fn:false()
    // Expected: assert-true true
    let dur_6699 = ym_duration_multiply(XsdYearMonthDuration::new(250), 2);
    let (_str_bytes_6699, str_len_6699): ([u8; 32], u32) = fn_string_from_ym_duration(dur_6699);
    assert((str_len_6699 > 0) | (false) == true);
}

#[test]
fn op_multiply_yearmonthduration_4() {
    //  ******************************************************* Test:
    // XPath: fn:not(fn:string(xs:yearMonthDuration("P11Y04M") * 2.0))
    // Expected: assert-false false
    let dur_3394 = ym_duration_multiply(XsdYearMonthDuration::new(136), 2);
    let (_str_bytes_3394, str_len_3394): ([u8; 32], u32) = fn_string_from_ym_duration(dur_3394);
    assert(!(str_len_3394 > 0) == false);
}

#[test]
fn op_multiply_yearmonthduration_5() {
    //  ******************************************************* Test:
    // XPath: fn:boolean(fn:string(xs:yearMonthDuration("P05Y08M") * 2.0))
    // Expected: assert-true true
    let dur_1611 = ym_duration_multiply(XsdYearMonthDuration::new(68), 2);
    let (_str_bytes_1611, str_len_1611): ([u8; 32], u32) = fn_string_from_ym_duration(dur_1611);
    assert((str_len_1611 > 0) == true);
}

#[test]
fn op_multiply_yearmonthduration_7() {
    //  ******************************************************* Test:
    // XPath: fn:string(xs:yearMonthDuration("P03Y08M") * 2.0)
    // Expected: assert-eq P7Y4M
    let dur_result = ym_duration_multiply(XsdYearMonthDuration::new(44), 2);
    let (str_bytes, str_len): ([u8; 5], u32) = fn_string_from_ym_duration(dur_result);
    assert(str_len == 5);
    assert(str_bytes[0] == 80); // 'P'
    assert(str_bytes[1] == 55); // '7'
    assert(str_bytes[2] == 89); // 'Y'
    assert(str_bytes[3] == 52); // '4'
    assert(str_bytes[4] == 77); // 'M'
}

#[test]
fn op_multiply_yearmonthduration_8() {
    //  ******************************************************* Test:
    // XPath: (xs:yearMonthDuration("P10Y01M") * -2.0)
    // Expected: assert-eq -P20Y2M
    let dur = XsdYearMonthDuration::new(121);
    assert(
        ym_duration_equal(
            ym_duration_multiply(dur, -2),
            XsdYearMonthDuration::new(-242),
        ),
    );
}

#[test]
fn op_multiply_yearmonthduration_9() {
    //  ******************************************************* Test:
    // XPath: fn:string((xs:yearMonthDuration("P01Y01M") * 2.0)) and fn:string((xs:yearMonthDuration("P02Y03M") * 2.0))
    // Expected: assert-true true
    let dur_3741 = ym_duration_multiply(XsdYearMonthDuration::new(13), 2);
    let (_str_bytes_3741, str_len_3741): ([u8; 32], u32) = fn_string_from_ym_duration(dur_3741);
    let dur_3525 = ym_duration_multiply(XsdYearMonthDuration::new(27), 2);
    let (_str_bytes_3525, str_len_3525): ([u8; 32], u32) = fn_string_from_ym_duration(dur_3525);
    assert((str_len_3741 > 0) & (str_len_3525 > 0) == true);
}

#[test]
fn op_multiply_yearmonthduration_10() {
    //  ******************************************************* Test:
    // XPath: fn:string((xs:yearMonthDuration("P05Y02M") * 2.0)) or fn:string((xs:yearMonthDuration("P05Y03M") * 2.0))
    // Expected: assert-true true
    let dur_7933 = ym_duration_multiply(XsdYearMonthDuration::new(62), 2);
    let (_str_bytes_7933, str_len_7933): ([u8; 32], u32) = fn_string_from_ym_duration(dur_7933);
    let dur_7952 = ym_duration_multiply(XsdYearMonthDuration::new(63), 2);
    let (_str_bytes_7952, str_len_7952): ([u8; 32], u32) = fn_string_from_ym_duration(dur_7952);
    assert((str_len_7933 > 0) | (str_len_7952 > 0) == true);
}

#[test]
fn op_multiply_yearmonthduration_11() {
    //  ******************************************************* Test:
    // XPath: (xs:yearMonthDuration("P20Y11M") * 2.0) div (xs:yearMonthDuration("P20Y11M") * 2.0)
    // Expected: assert-eq 1
    // Oracle: x div x = 1.0 exactly (0x3FF0000000000000)
    assert(
        ym_duration_divide_by_duration(
            ym_duration_multiply(XsdYearMonthDuration::new(251), 2),
            ym_duration_multiply(XsdYearMonthDuration::new(251), 2),
        )
            == XsdDouble::from_bits(0x3FF0000000000000),
    );
}

#[test]
fn op_multiply_yearmonthduration_12() {
    //  ******************************************************* Test:
    // XPath: fn:string((xs:yearMonthDuration("P10Y11M") * 2.0)) and (fn:true())
    // Expected: assert-true true
    let dur_332 = ym_duration_multiply(XsdYearMonthDuration::new(131), 2);
    let (_str_bytes_332, str_len_332): ([u8; 32], u32) = fn_string_from_ym_duration(dur_332);
    assert((str_len_332 > 0) & (true) == true);
}

#[test]
fn op_multiply_yearmonthduration_13() {
    //  ******************************************************* Test:
    // XPath: (xs:yearMonthDuration("P23Y11M") * 2.0) eq xs:yearMonthDuration("P23Y11M")
    // Expected: assert-false false
    assert(
        ym_duration_equal(
            ym_duration_multiply(XsdYearMonthDuration::new(287), 2),
            XsdYearMonthDuration::new(287),
        )
            == false,
    );
}

#[test]
fn op_multiply_yearmonthduration_14() {
    //  ******************************************************* Test:
    // XPath: (xs:yearMonthDuration("P21Y12M") * 2.0) ne xs:yearMonthDuration("P08Y05M")
    // Expected: assert-true true
    assert(
        !ym_duration_equal(
            ym_duration_multiply(XsdYearMonthDuration::new(264), 2),
            XsdYearMonthDuration::new(101),
        )
            == true,
    );
}

#[test]
fn op_multiply_yearmonthduration_15() {
    //  ******************************************************* Test:
    // XPath: (xs:yearMonthDuration("P10Y01M") * 2.0) le xs:yearMonthDuration("P17Y02M")
    // Expected: assert-false false
    assert(
        ym_duration_le(
            ym_duration_multiply(XsdYearMonthDuration::new(121), 2),
            XsdYearMonthDuration::new(206),
        )
            == false,
    );
}

#[test]
fn op_multiply_yearmonthduration_16() {
    //  ******************************************************* Test:
    // XPath: (xs:yearMonthDuration("P13Y09M") - xs:yearMonthDuration("P18Y02M")) ge xs:yearMonthDuration("P18Y02M")
    // Expected: assert-false false
    assert(
        ym_duration_ge(
            ym_duration_subtract(
                XsdYearMonthDuration::new(165),
                XsdYearMonthDuration::new(218),
            ),
            XsdYearMonthDuration::new(218),
        )
            == false,
    );
}

#[test]
fn k_yearmonthdurationmultiply_1() {
    //  *******************************************************  Test:
    // XPath: xs:yearMonthDuration("P3Y36M") * 3 eq xs:yearMonthDuration("P18Y")
    // Expected: assert-true true
    assert(
        ym_duration_equal(
            ym_duration_multiply(XsdYearMonthDuration::new(72), 3),
            XsdYearMonthDuration::new(216),
        )
            == true,
    );
}

#[test]
fn k_yearmonthdurationmultiply_2() {
    //  *******************************************************  Test:
    // XPath: 3 * xs:yearMonthDuration("P3Y36M") eq xs:yearMonthDuration("P18Y")
    // Expected: assert-true true
    assert(
        ym_duration_equal(
            ym_duration_multiply(XsdYearMonthDuration::new(72), 3),
            XsdYearMonthDuration::new(216),
        )
            == true,
    );
}

#[test]
fn k_yearmonthdurationmultiply_3() {
    //  *******************************************************  Test:
    // XPath: 0 * xs:yearMonthDuration("P3Y36M") eq xs:yearMonthDuration("P0M")
    // Expected: assert-true true
    assert(
        ym_duration_equal(
            ym_duration_multiply(XsdYearMonthDuration::new(72), 0),
            XsdYearMonthDuration::new(0),
        )
            == true,
    );
}

#[test]
fn k_yearmonthdurationmultiply_4() {
    //  *******************************************************  Test:
    // XPath: xs:yearMonthDuration("P3Y36M") * 0 eq xs:yearMonthDuration("P0M")
    // Expected: assert-true true
    assert(
        ym_duration_equal(
            ym_duration_multiply(XsdYearMonthDuration::new(72), 0),
            XsdYearMonthDuration::new(0),
        )
            == true,
    );
}
