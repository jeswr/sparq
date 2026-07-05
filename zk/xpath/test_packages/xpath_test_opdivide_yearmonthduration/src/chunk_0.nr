//! Test chunk 0 for op:divide-yearMonthDuration
//! Contains 15 tests

use dep::xpath::{
    fn_string_from_ym_duration, XsdDouble, XsdYearMonthDuration, ym_duration_divide,
    ym_duration_divide_by_duration, ym_duration_equal, ym_duration_ge, ym_duration_le,
};

#[test]
fn op_divide_yearmonthduration_2() {
    //  ******************************************************* Test:
    // XPath: fn:string((xs:yearMonthDuration("P10Y11M") div 2.0)) and fn:false()
    // Expected: assert-false false
    let dur_883 = ym_duration_divide(XsdYearMonthDuration::new(131), 2);
    let (_str_bytes_883, str_len_883): ([u8; 32], u32) = fn_string_from_ym_duration(dur_883);
    assert((str_len_883 > 0) & (false) == false);
}

#[test]
fn op_divide_yearmonthduration_3() {
    //  ******************************************************* Test:
    // XPath: fn:string((xs:yearMonthDuration("P20Y10M") div 2.0)) or fn:false()
    // Expected: assert-true true
    let dur_3988 = ym_duration_divide(XsdYearMonthDuration::new(250), 2);
    let (_str_bytes_3988, str_len_3988): ([u8; 32], u32) = fn_string_from_ym_duration(dur_3988);
    assert((str_len_3988 > 0) | (false) == true);
}

#[test]
fn op_divide_yearmonthduration_4() {
    //  ******************************************************* Test:
    // XPath: fn:not(fn:string(xs:yearMonthDuration("P11Y04M") div 2.0))
    // Expected: assert-false false
    let dur_4655 = ym_duration_divide(XsdYearMonthDuration::new(136), 2);
    let (_str_bytes_4655, str_len_4655): ([u8; 32], u32) = fn_string_from_ym_duration(dur_4655);
    assert(!(str_len_4655 > 0) == false);
}

#[test]
fn op_divide_yearmonthduration_5() {
    //  ******************************************************* Test:
    // XPath: fn:boolean(fn:string(xs:yearMonthDuration("P05Y08M") div 2.0))
    // Expected: assert-true true
    let dur_4473 = ym_duration_divide(XsdYearMonthDuration::new(68), 2);
    let (_str_bytes_4473, str_len_4473): ([u8; 32], u32) = fn_string_from_ym_duration(dur_4473);
    assert((str_len_4473 > 0) == true);
}

#[test]
fn op_divide_yearmonthduration_7() {
    //  ******************************************************* Test:
    // XPath: fn:string(xs:yearMonthDuration("P03Y08M") div 2.0)
    // Expected: assert-eq P1Y10M
    let dur_result = ym_duration_divide(XsdYearMonthDuration::new(44), 2);
    let (str_bytes, str_len): ([u8; 6], u32) = fn_string_from_ym_duration(dur_result);
    assert(str_len == 6);
    assert(str_bytes[0] == 80); // 'P'
    assert(str_bytes[1] == 49); // '1'
    assert(str_bytes[2] == 89); // 'Y'
    assert(str_bytes[3] == 49); // '1'
    assert(str_bytes[4] == 48); // '0'
    assert(str_bytes[5] == 77); // 'M'
}

#[test]
fn op_divide_yearmonthduration_8() {
    //  ******************************************************* Test:
    // XPath: (xs:yearMonthDuration("P10Y01M") div -2.0)
    // Expected: assert-eq -P5Y
    let dur = XsdYearMonthDuration::new(121);
    assert(ym_duration_equal(ym_duration_divide(dur, -2), XsdYearMonthDuration::new(-60)));
}

#[test]
fn op_divide_yearmonthduration_9() {
    //  ******************************************************* Test:
    // XPath: fn:string((xs:yearMonthDuration("P01Y01M") div 2.0)) and fn:string((xs:yearMonthDuration("P02Y03M") div 2.0))
    // Expected: assert-true true
    let dur_5987 = ym_duration_divide(XsdYearMonthDuration::new(13), 2);
    let (_str_bytes_5987, str_len_5987): ([u8; 32], u32) = fn_string_from_ym_duration(dur_5987);
    let dur_3808 = ym_duration_divide(XsdYearMonthDuration::new(27), 2);
    let (_str_bytes_3808, str_len_3808): ([u8; 32], u32) = fn_string_from_ym_duration(dur_3808);
    assert((str_len_5987 > 0) & (str_len_3808 > 0) == true);
}

#[test]
fn op_divide_yearmonthduration_10() {
    //  ******************************************************* Test:
    // XPath: fn:string((xs:yearMonthDuration("P05Y02M") div 2.0)) or fn:string((xs:yearMonthDuration("P05Y03M") div 2.0))
    // Expected: assert-true true
    let dur_6298 = ym_duration_divide(XsdYearMonthDuration::new(62), 2);
    let (_str_bytes_6298, str_len_6298): ([u8; 32], u32) = fn_string_from_ym_duration(dur_6298);
    let dur_6424 = ym_duration_divide(XsdYearMonthDuration::new(63), 2);
    let (_str_bytes_6424, str_len_6424): ([u8; 32], u32) = fn_string_from_ym_duration(dur_6424);
    assert((str_len_6298 > 0) | (str_len_6424 > 0) == true);
}

#[test]
fn op_divide_yearmonthduration_11() {
    //  ******************************************************* Test:
    // XPath: (xs:yearMonthDuration("P20Y11M") div 2.0) div (xs:yearMonthDuration("P20Y11M") div 2.0)
    // Expected: assert-eq 1
    // Oracle: x div x = 1.0 exactly (0x3FF0000000000000)
    assert(
        ym_duration_divide_by_duration(
            ym_duration_divide(XsdYearMonthDuration::new(251), 2),
            ym_duration_divide(XsdYearMonthDuration::new(251), 2),
        )
            == XsdDouble::from_bits(0x3FF0000000000000),
    );
}

#[test]
fn op_divide_yearmonthduration_12() {
    //  ******************************************************* Test:
    // XPath: fn:string((xs:yearMonthDuration("P10Y11M") div 2.0)) and (fn:true())
    // Expected: assert-true true
    let dur_883 = ym_duration_divide(XsdYearMonthDuration::new(131), 2);
    let (_str_bytes_883, str_len_883): ([u8; 32], u32) = fn_string_from_ym_duration(dur_883);
    assert((str_len_883 > 0) & (true) == true);
}

#[test]
fn op_divide_yearmonthduration_13() {
    //  ******************************************************* Test:
    // XPath: (xs:yearMonthDuration("P23Y11M") div 2.0) eq xs:yearMonthDuration("P23Y11M")
    // Expected: assert-false false
    assert(
        ym_duration_equal(
            ym_duration_divide(XsdYearMonthDuration::new(287), 2),
            XsdYearMonthDuration::new(287),
        )
            == false,
    );
}

#[test]
fn op_divide_yearmonthduration_14() {
    //  ******************************************************* Test:
    // XPath: (xs:yearMonthDuration("P21Y12M") div 2.0) ne xs:yearMonthDuration("P08Y05M")
    // Expected: assert-true true
    assert(
        !ym_duration_equal(
            ym_duration_divide(XsdYearMonthDuration::new(264), 2),
            XsdYearMonthDuration::new(101),
        )
            == true,
    );
}

#[test]
fn op_divide_yearmonthduration_15() {
    //  ******************************************************* Test:
    // XPath: (xs:yearMonthDuration("P10Y01M") div 2.0) le xs:yearMonthDuration("P17Y02M")
    // Expected: assert-true true
    assert(
        ym_duration_le(
            ym_duration_divide(XsdYearMonthDuration::new(121), 2),
            XsdYearMonthDuration::new(206),
        )
            == true,
    );
}

#[test]
fn op_divide_yearmonthduration_16() {
    //  ******************************************************* Test:
    // XPath: (xs:yearMonthDuration("P13Y09M") div 2.0) ge xs:yearMonthDuration("P18Y02M")
    // Expected: assert-false false
    assert(
        ym_duration_ge(
            ym_duration_divide(XsdYearMonthDuration::new(165), 2),
            XsdYearMonthDuration::new(218),
        )
            == false,
    );
}

#[test]
fn k_yearmonthdurationdivide_1() {
    //  *******************************************************  Test:
    // XPath: xs:yearMonthDuration("P3Y36M") div 3 eq xs:yearMonthDuration("P2Y")
    // Expected: assert-true true
    assert(
        ym_duration_equal(
            ym_duration_divide(XsdYearMonthDuration::new(72), 3),
            XsdYearMonthDuration::new(24),
        )
            == true,
    );
}
