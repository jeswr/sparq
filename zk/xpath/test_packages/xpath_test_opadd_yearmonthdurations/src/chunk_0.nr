//! Test chunk 0 for op:add-yearMonthDurations
//! Contains 23 tests

use dep::xpath::{
    fn_string_from_ym_duration, XsdYearMonthDuration, ym_duration_add,
    ym_duration_divide_by_duration, ym_duration_equal, ym_duration_ge, ym_duration_le,
};

#[test]
fn op_add_yearmonthdurations2args_1() {
    //  ******************************************************* Test:
    // XPath: xs:yearMonthDuration("P0Y0M") + xs:yearMonthDuration("P0Y0M")
    // Expected: assert-eq P0M
    let d1 = XsdYearMonthDuration::new(0);
    let d2 = XsdYearMonthDuration::new(0);
    assert(ym_duration_equal(ym_duration_add(d1, d2), XsdYearMonthDuration::new(0)));
}

#[test]
fn op_add_yearmonthdurations2args_2() {
    //  ******************************************************* Test:
    // XPath: xs:yearMonthDuration("P1000Y6M") + xs:yearMonthDuration("P0Y0M")
    // Expected: assert-eq P1000Y6M
    let d1 = XsdYearMonthDuration::new(12006);
    let d2 = XsdYearMonthDuration::new(0);
    assert(ym_duration_equal(ym_duration_add(d1, d2), XsdYearMonthDuration::new(12006)));
}

#[test]
fn op_add_yearmonthdurations2args_3() {
    //  ******************************************************* Test:
    // XPath: xs:yearMonthDuration("P2030Y12M") + xs:yearMonthDuration("P0Y0M")
    // Expected: assert-eq P2031Y
    let d1 = XsdYearMonthDuration::new(24372);
    let d2 = XsdYearMonthDuration::new(0);
    assert(ym_duration_equal(ym_duration_add(d1, d2), XsdYearMonthDuration::new(24372)));
}

#[test]
fn op_add_yearmonthdurations2args_4() {
    //  ******************************************************* Test:
    // XPath: xs:yearMonthDuration("P0Y0M") + xs:yearMonthDuration("P1000Y6M")
    // Expected: assert-eq P1000Y6M
    let d1 = XsdYearMonthDuration::new(0);
    let d2 = XsdYearMonthDuration::new(12006);
    assert(ym_duration_equal(ym_duration_add(d1, d2), XsdYearMonthDuration::new(12006)));
}

#[test]
fn op_add_yearmonthdurations2args_5() {
    //  ******************************************************* Test:
    // XPath: xs:yearMonthDuration("P0Y0M") + xs:yearMonthDuration("P2030Y12M")
    // Expected: assert-eq P2031Y
    let d1 = XsdYearMonthDuration::new(0);
    let d2 = XsdYearMonthDuration::new(24372);
    assert(ym_duration_equal(ym_duration_add(d1, d2), XsdYearMonthDuration::new(24372)));
}

#[test]
fn op_add_yearmonthdurations_1() {
    //  ******************************************************* Test:
    // XPath: xs:yearMonthDuration("P2Y11M") + xs:yearMonthDuration("P3Y3M")
    // Expected: assert-eq P6Y2M
    let d1 = XsdYearMonthDuration::new(35);
    let d2 = XsdYearMonthDuration::new(39);
    assert(ym_duration_equal(ym_duration_add(d1, d2), XsdYearMonthDuration::new(74)));
}

#[test]
fn op_add_yearmonthdurations_2() {
    //  ******************************************************* Test:
    // XPath: fn:string(xs:yearMonthDuration("P10Y11M") + xs:yearMonthDuration("P12Y07M")) and fn:false()
    // Expected: assert-false false
    let dur_426 = ym_duration_add(
        XsdYearMonthDuration::new(131),
        XsdYearMonthDuration::new(151),
    );
    let (_str_bytes_426, str_len_426): ([u8; 32], u32) = fn_string_from_ym_duration(dur_426);
    assert((str_len_426 > 0) & (false) == false);
}

#[test]
fn op_add_yearmonthdurations_3() {
    //  ******************************************************* Test:
    // XPath: fn:string(xs:yearMonthDuration("P20Y10M") + xs:yearMonthDuration("P19Y10M")) or fn:false()
    // Expected: assert-true true
    let dur_9426 = ym_duration_add(
        XsdYearMonthDuration::new(250),
        XsdYearMonthDuration::new(238),
    );
    let (_str_bytes_9426, str_len_9426): ([u8; 32], u32) = fn_string_from_ym_duration(dur_9426);
    assert((str_len_9426 > 0) | (false) == true);
}

#[test]
fn op_add_yearmonthdurations_4() {
    //  ******************************************************* Test:
    // XPath: fn:not(fn:string(xs:yearMonthDuration("P11Y04M") + xs:yearMonthDuration("P02Y11M")))
    // Expected: assert-false false
    let dur_9624 = ym_duration_add(
        XsdYearMonthDuration::new(136),
        XsdYearMonthDuration::new(35),
    );
    let (_str_bytes_9624, str_len_9624): ([u8; 32], u32) = fn_string_from_ym_duration(dur_9624);
    assert(!(str_len_9624 > 0) == false);
}

#[test]
fn op_add_yearmonthdurations_5() {
    //  ******************************************************* Test:
    // XPath: fn:boolean(fn:string(xs:yearMonthDuration("P05Y08M") + xs:yearMonthDuration("P03Y06M")))
    // Expected: assert-true true
    let dur_5265 = ym_duration_add(XsdYearMonthDuration::new(68), XsdYearMonthDuration::new(42));
    let (_str_bytes_5265, str_len_5265): ([u8; 32], u32) = fn_string_from_ym_duration(dur_5265);
    assert((str_len_5265 > 0) == true);
}

#[test]
fn op_add_yearmonthdurations_7() {
    //  ******************************************************* Test:
    // XPath: fn:string(xs:yearMonthDuration("P03Y08M") + xs:yearMonthDuration("P01Y02M"))
    // Expected: assert-eq P4Y10M
    let dur_result = ym_duration_add(XsdYearMonthDuration::new(44), XsdYearMonthDuration::new(14));
    let (str_bytes, str_len): ([u8; 6], u32) = fn_string_from_ym_duration(dur_result);
    assert(str_len == 6);
    assert(str_bytes[0] == 80); // 'P'
    assert(str_bytes[1] == 52); // '4'
    assert(str_bytes[2] == 89); // 'Y'
    assert(str_bytes[3] == 49); // '1'
    assert(str_bytes[4] == 48); // '0'
    assert(str_bytes[5] == 77); // 'M'
}

#[test]
fn op_add_yearmonthdurations_8() {
    //  ******************************************************* Test:
    // XPath: (xs:yearMonthDuration("P10Y01M") + xs:yearMonthDuration("-P11Y02M"))
    // Expected: assert-eq -P1Y1M
    let d1 = XsdYearMonthDuration::new(121);
    let d2 = XsdYearMonthDuration::new(-134);
    assert(ym_duration_equal(ym_duration_add(d1, d2), XsdYearMonthDuration::new(-13)));
}

#[test]
fn op_add_yearmonthdurations_9() {
    //  ******************************************************* Test:
    // XPath: fn:string((xs:yearMonthDuration("P01Y01M") + xs:yearMonthDuration("P02Y02M"))) and fn:string((xs:yearMonthDuration("P02Y03M") + xs:yearMonthDuration("P04Y04M")))
    // Expected: assert-true true
    let dur_3070 = ym_duration_add(XsdYearMonthDuration::new(13), XsdYearMonthDuration::new(26));
    let (_str_bytes_3070, str_len_3070): ([u8; 32], u32) = fn_string_from_ym_duration(dur_3070);
    let dur_5646 = ym_duration_add(XsdYearMonthDuration::new(27), XsdYearMonthDuration::new(52));
    let (_str_bytes_5646, str_len_5646): ([u8; 32], u32) = fn_string_from_ym_duration(dur_5646);
    assert((str_len_3070 > 0) & (str_len_5646 > 0) == true);
}

#[test]
fn op_add_yearmonthdurations_10() {
    //  ******************************************************* Test:
    // XPath: fn:string((xs:yearMonthDuration("P05Y02M") + xs:yearMonthDuration("P03Y04M"))) or fn:string((xs:yearMonthDuration("P05Y03M") + xs:yearMonthDuration("P01Y03M")))
    // Expected: assert-true true
    let dur_2003 = ym_duration_add(XsdYearMonthDuration::new(62), XsdYearMonthDuration::new(40));
    let (_str_bytes_2003, str_len_2003): ([u8; 32], u32) = fn_string_from_ym_duration(dur_2003);
    let dur_4004 = ym_duration_add(XsdYearMonthDuration::new(63), XsdYearMonthDuration::new(15));
    let (_str_bytes_4004, str_len_4004): ([u8; 32], u32) = fn_string_from_ym_duration(dur_4004);
    assert((str_len_2003 > 0) | (str_len_4004 > 0) == true);
}

#[test]
fn op_add_yearmonthdurations_11() {
    //  ******************************************************* Test:
    // XPath: (xs:yearMonthDuration("P42Y10M") + xs:yearMonthDuration("P28Y10M")) div (xs:yearMonthDuration("P10Y10M") + xs:yearMonthDuration("P60Y10M"))
    // Expected: assert-eq 1
    assert(
        ym_duration_divide_by_duration(
            ym_duration_add(
                XsdYearMonthDuration::new(514),
                XsdYearMonthDuration::new(346),
            ),
            ym_duration_add(
                XsdYearMonthDuration::new(130),
                XsdYearMonthDuration::new(730),
            ),
        )
            == 1,
    );
}

#[test]
fn op_add_yearmonthdurations_12() {
    //  ******************************************************* Test:
    // XPath: fn:string((xs:yearMonthDuration("P10Y11M") + xs:yearMonthDuration("P05Y07M"))) and (fn:true())
    // Expected: assert-true true
    let dur_3442 = ym_duration_add(
        XsdYearMonthDuration::new(131),
        XsdYearMonthDuration::new(67),
    );
    let (_str_bytes_3442, str_len_3442): ([u8; 32], u32) = fn_string_from_ym_duration(dur_3442);
    assert((str_len_3442 > 0) & (true) == true);
}

#[test]
fn op_add_yearmonthdurations_13() {
    //  ******************************************************* Test:
    // XPath: (xs:yearMonthDuration("P23Y11M") + xs:yearMonthDuration("P23Y11M")) eq xs:yearMonthDuration("P23Y11M")
    // Expected: assert-false false
    assert(
        ym_duration_equal(
            ym_duration_add(
                XsdYearMonthDuration::new(287),
                XsdYearMonthDuration::new(287),
            ),
            XsdYearMonthDuration::new(287),
        )
            == false,
    );
}

#[test]
fn op_add_yearmonthdurations_14() {
    //  ******************************************************* Test:
    // XPath: (xs:yearMonthDuration("P21Y12M") + xs:yearMonthDuration("P08Y05M")) ne xs:yearMonthDuration("P08Y05M")
    // Expected: assert-true true
    assert(
        !ym_duration_equal(
            ym_duration_add(
                XsdYearMonthDuration::new(264),
                XsdYearMonthDuration::new(101),
            ),
            XsdYearMonthDuration::new(101),
        )
            == true,
    );
}

#[test]
fn op_add_yearmonthdurations_15() {
    //  ******************************************************* Test:
    // XPath: (xs:yearMonthDuration("P10Y01M") + xs:yearMonthDuration("P17Y02M")) le xs:yearMonthDuration("P17Y02M")
    // Expected: assert-false false
    assert(
        ym_duration_le(
            ym_duration_add(
                XsdYearMonthDuration::new(121),
                XsdYearMonthDuration::new(206),
            ),
            XsdYearMonthDuration::new(206),
        )
            == false,
    );
}

#[test]
fn op_add_yearmonthdurations_16() {
    //  ******************************************************* Test:
    // XPath: (xs:yearMonthDuration("P13Y09M") + xs:yearMonthDuration("P18Y02M")) ge xs:yearMonthDuration("P18Y02M")
    // Expected: assert-true true
    assert(
        ym_duration_ge(
            ym_duration_add(
                XsdYearMonthDuration::new(165),
                XsdYearMonthDuration::new(218),
            ),
            XsdYearMonthDuration::new(218),
        )
            == true,
    );
}

#[test]
fn k_yearmonthdurationadd_1() {
    //  *******************************************************  Test:
    // XPath: xs:yearMonthDuration("P3Y3M") + xs:yearMonthDuration("P0M") eq xs:yearMonthDuration("P3Y3M")
    // Expected: assert-true true
    assert(
        ym_duration_equal(
            ym_duration_add(XsdYearMonthDuration::new(39), XsdYearMonthDuration::new(0)),
            XsdYearMonthDuration::new(39),
        )
            == true,
    );
}

#[test]
fn k_yearmonthdurationadd_2() {
    //  *******************************************************  Test:
    // XPath: xs:yearMonthDuration("P0M") + xs:yearMonthDuration("P3Y3M") eq xs:yearMonthDuration("P3Y3M")
    // Expected: assert-true true
    assert(
        ym_duration_equal(
            ym_duration_add(XsdYearMonthDuration::new(0), XsdYearMonthDuration::new(39)),
            XsdYearMonthDuration::new(39),
        )
            == true,
    );
}

#[test]
fn k_yearmonthdurationadd_3() {
    //  *******************************************************  Test:
    // XPath: xs:yearMonthDuration("P32Y43M") + xs:yearMonthDuration("P12Y13M") eq xs:yearMonthDuration("P48Y8M")
    // Expected: assert-true true
    assert(
        ym_duration_equal(
            ym_duration_add(
                XsdYearMonthDuration::new(427),
                XsdYearMonthDuration::new(157),
            ),
            XsdYearMonthDuration::new(584),
        )
            == true,
    );
}
