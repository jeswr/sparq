//! Test chunk 0 for fn:format-number
//! Contains 50 tests

use dep::xpath::stub_fnformat_number;

#[test]
fn numberformat01() {
    //          Creator: David Marston          Purpose: Test of format-number with 2 a
    // XPath: format-number(2392.14*36.58,'000,000.000000')
    // Expected: 087,504.481200
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat02() {
    //          Creator: David Marston          Purpose: Test of format-number and # an
    // XPath: format-number(12792.14*96.58,'##,###,000.000###')
    // Expected: 1,235,464.8812
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat03() {
    //          Creator: David Marston          Purpose: Test of format-number on a neg
    // XPath: format-number(2792.14*(-36.58),'000,000.000###')
    // Expected: -102,136.4812
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat04() {
    //          Creator: David Marston          Purpose: Test of format-number on a neg
    // XPath: format-number(2392.14*(-36.58),'000,000.000###;###,###.000##
    // Expected: 87,504.4812
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat05() {
    //          Creator: David Marston          Purpose: Test of format-number percenta
    // XPath: format-number(0.4857,'###.###%')
    // Expected: 48.57%
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat06() {
    //          Creator: David Marston          Purpose: Test of format-number per-mill
    // XPath: format-number(0.4857,'###.###?')
    // Expected: 485.7?
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat07() {
    //          Creator: David Marston          Purpose: Test of format-number currency
    // XPath: format-number(95.4857,'?###.####')
    // Expected: ?95.4857
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat08() {
    //          Creator: David Marston          Purpose: Test of format-number with pre
    // XPath: format-number(2.14*86.58,'PREFIX##00.000###SUFFIX')
    // Expected: PREFIX185.2812SUFFIX
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat09() {
    //          Creator: David Marston          Purpose: Test default decimal-format on
    // XPath: format-number(931.4857,'000.000|###')
    // Expected: 000.931|486
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat11() {
    //          Creator: David Marston          Purpose: Test default decimal-format on
    // XPath: format-number(26931.4,'+!!!,!!!.!!!\-!!,!!!.!!!')
    // Expected: +26,931.4
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat12() {
    //          Creator: David Marston          Purpose: Test default decimal-format on
    // XPath: format-number(-26931.4,'+!!,!!!.!!!\-!!!,!!!.!!!')
    // Expected: -26,931.4
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat13() {
    //          Creator: David Marston          Purpose: Test default decimal-format on
    // XPath: format-number(-26931.4,'!!!,!!!.!!!')
    // Expected: -26,931.4
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat14() {
    //          Creator: David Marston          Purpose: Test specified result pattern
    // XPath: format-number(1 div 0e0,'###############################')
    // Expected: off-the-scale
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat15() {
    //          Creator: David Marston          Purpose: Test specified result pattern
    // XPath: format-number('foo','#############')
    // Expected: non-numeric
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat16() {
    //          Creator: David Marston          Purpose: Test of decimal-format per-mil
    // XPath: format-number(0.4857,'###.###m')
    // Expected: 485.7m
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat17() {
    //          Creator: David Marston          Purpose: Test decimal-format output cha
    // XPath: format-number(-26931.4,'+###,###.###;-###,###.###')
    // Expected: -26,931.4
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat18() {
    //          Creator: David Marston          Purpose: Test decimal-format output cha
    // XPath: format-number(-26931.4,'###,###.###')
    // Expected: _26,931.4
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat19() {
    //          Creator: David Marston          Purpose: Test decimal-format declaratio
    // XPath: concat(format-number(-26931.4,'###,###.###','myminus'), '/',
    // Expected: _26,931.4/-42,857.1
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat20() {
    //          Creator: David Marston          Purpose: Test of decimal-format with qu
    // XPath: format-number(1234.567,'#*###*###!###','foo:decimal1')
    // Expected: 1*234!567
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat26() {
    //          Creator: David Marston          Purpose: Designate a space as the group
    // XPath: format-number(7654321.4857,'### ### ###,#####')
    // Expected: 7 654 321,4857
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat27() {
    //          Creator: David Marston          Purpose: Test of format-number minus-si
    // XPath: string-join((format-number(2392.14*36.58,'000,000.000000','m
    // Expected: 087,504.481200 087,504.481200 087,504.48
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat28() {
    //          Creator: David Marston          Purpose: Test what happens to minus sig
    // XPath: format-number(2392.14*(-36.58),'000,000.000###;-###,###.000#
    // Expected: -87,504.4812
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat29() {
    //          Creator: David Marston          Purpose: Test decimal-format output cha
    // XPath: format-number(-26931.4,'+###,###.###;_###,###.###')
    // Expected: _26,931.4
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat30() {
    //          Creator: David Marston          Purpose: Test effects of minus-sign in
    // XPath: string-join((                 format-number(-26931.4,'-###,#
    // Expected: --26,931.4 _zzz-26,931.4 __26,931.4
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat31() {
    //          Creator: David Marston          Purpose: Test output of altered minus,
    // XPath: format-number(-26931.4,'###,###.###;###,###.###')
    // Expected: 26,931.4
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat32() {
    //          Creator: David Marston          Purpose: Test of format-number percent
    // XPath: format-number(0.4857,'###.###c')
    // Expected: 48.57c
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat34() {
    //          Creator: David Marston (modified by MHK because there are now rules on
    // XPath: format-number(4030201.0506,'#!!!,!!!,???.??????0')
    // Expected: #?,???,???.??????0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat35() {
    //          Creator: David Marston          Purpose: Test of unequal spacing of gro
    // XPath: format-number(987654321,'###,##0,00.00')
    // Expected: 9876,543,21.00
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat36() {
    //          Creator: David Marston          Purpose: Test what happens when we over
    // XPath: format-number(239236.588,'00000.00')
    // Expected: 239236.59
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat37() {
    //          Creator: David Marston          Purpose: Test result pattern for infini
    // XPath: format-number(1 div 0e0,'###############################')
    // Expected: Infinity
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat38() {
    //          Creator: David Marston          Purpose: Test result pattern for not-a-
    // XPath: format-number('foo','#############')
    // Expected: NaN
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat39() {
    //          Creator: David Marston          Purpose: Test result pattern for negati
    // XPath: format-number(-1 div 0e0,'###############################')
    // Expected: -Infinity
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat40() {
    //          Creator: David Marston          Purpose: Test specification of result p
    // XPath: format-number(-1 div 0e0,'###############################')
    // Expected: -huge
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat41() {
    //          Creator: David Bertoni          Purpose: Test of multiple decimal-forma
    // XPath: string-join((                 format-number('NaN','###','foo
    // Expected: not a number|-13.2|not a number|-13.2
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat42() {
    //          Creator: David Bertoni          Purpose: Test of multiple decimal-forma
    // XPath: concat(format-number('NaN','###','decimal2'), '|',
    // Expected: not a number|3.2
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat60a() {
    //          Test format-number() applied to large numbers (test how good the roundi
    // XPath: format-number(1E25,'#,######')
    // Expected: 10,000000,000000,000000,000000
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat60b() {
    //          Test format-number() applied to large numbers (test how good the roundi
    // XPath: format-number(1E10,'#####################')
    // Expected: 10000000000
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat60c() {
    //          Test format-number() applied to large numbers (test how good the roundi
    // XPath: format-number(1E11,'#####################')
    // Expected: 100000000000
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat60d() {
    //          Test format-number() applied to large numbers (test how good the roundi
    // XPath: format-number(1E12,'#####################')
    // Expected: 1000000000000
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat60e() {
    //          Test format-number() applied to large numbers (test how good the roundi
    // XPath: format-number(1E13,'#####################')
    // Expected: 10000000000000
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat60f() {
    //          Test format-number() applied to large numbers (test how good the roundi
    // XPath: format-number(1E14,'#####################')
    // Expected: 100000000000000
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat60g() {
    //          Test format-number() applied to large numbers (test how good the roundi
    // XPath: format-number(1E15,'#####################')
    // Expected: 1000000000000000
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat60h() {
    //          Test format-number() applied to large numbers (test how good the roundi
    // XPath: format-number(1E16,'#####################')
    // Expected: 10000000000000000
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat60i() {
    //          Test format-number() applied to large numbers (test how good the roundi
    // XPath: format-number(1E17,'#####################')
    // Expected: 100000000000000000
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat60j() {
    //          Test format-number() applied to large numbers (test how good the roundi
    // XPath: format-number(1E18,'#####################')
    // Expected: 1000000000000000000
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat60k() {
    //          Test format-number() applied to large numbers (test how good the roundi
    // XPath: format-number(1E19,'#####################')
    // Expected: 10000000000000000000
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat60l() {
    //          Test format-number() applied to large numbers (test how good the roundi
    // XPath: format-number(1E20,'#####################')
    // Expected: 100000000000000000000
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat60m() {
    //          Test format-number() applied to large numbers (test how good the roundi
    // XPath: format-number(1E25,'#####################')
    // Expected: 10000000000000000000000000
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat60n() {
    //          Test format-number() applied to large numbers (test how good the roundi
    // XPath: format-number(1E30,'#####################')
    // Expected: 1000000000000000000000000000000
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat60o() {
    //          Test format-number() applied to large numbers (test how good the roundi
    // XPath: format-number(1E35,'#####################')
    // Expected: 100000000000000000000000000000000000
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}
