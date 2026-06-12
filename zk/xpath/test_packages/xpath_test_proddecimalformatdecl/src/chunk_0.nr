//! Test chunk 0 for prod:DecimalFormatDecl
//! Contains 21 tests

use dep::xpath::stub_proddecimalformatdecl;

#[test]
fn decimal_format_01() {
    //          Purpose: Test of format-number with 2 arguments, showing zeroes.
    // XPath: declare default decimal-format zero-digit=\"0\" grouping-sep
    // Expected: 087,504.481200
    // Calls stub function - will fail until prod:DecimalFormatDecl is implemented
    let _ = stub_proddecimalformatdecl();
}

#[test]
fn decimal_format_02() {
    //          Purpose: Test of format-number and # and 0 in format string.
    // XPath: declare default decimal-format digit=\"#\" grouping-separato
    // Expected: 1,235,464.8812
    // Calls stub function - will fail until prod:DecimalFormatDecl is implemented
    let _ = stub_proddecimalformatdecl();
}

#[test]
fn decimal_format_03() {
    //          Purpose: Test of format-number on a negative number.
    // XPath: declare default decimal-format minus-sign=\"-\" grouping-sep
    // Expected: -102,136.4812
    // Calls stub function - will fail until prod:DecimalFormatDecl is implemented
    let _ = stub_proddecimalformatdecl();
}

#[test]
fn decimal_format_04() {
    //          Purpose: Test of format-number on a negative number; should choose seco
    // XPath: declare default decimal-format minus-sign=\"-\" pattern-sepa
    // Expected: 87,504.4812
    // Calls stub function - will fail until prod:DecimalFormatDecl is implemented
    let _ = stub_proddecimalformatdecl();
}

#[test]
fn decimal_format_05() {
    //          Purpose: Test of format-number percentage format.
    // XPath: declare decimal-format local:df minus-sign=\"-\" percent=\"%
    // Expected: 48.57%
    // Calls stub function - will fail until prod:DecimalFormatDecl is implemented
    let _ = stub_proddecimalformatdecl();
}

#[test]
fn decimal_format_06() {
    //          Purpose: Test of format-number per-mille format.
    // XPath: declare decimal-format local:df minus-sign=\"-\" per-mille=\
    // Expected: 485.7?
    // Calls stub function - will fail until prod:DecimalFormatDecl is implemented
    let _ = stub_proddecimalformatdecl();
}

#[test]
fn decimal_format_09() {
    //          Purpose: Test non-default decimal-format on separator characters, chang
    // XPath: declare default decimal-format decimal-separator=\"|\" group
    // Expected: 000.931|486
    // Calls stub function - will fail until prod:DecimalFormatDecl is implemented
    let _ = stub_proddecimalformatdecl();
}

#[test]
fn decimal_format_11() {
    //          Purpose: Test default decimal-format on pattern-only characters, positi
    // XPath: declare default decimal-format digit=\"!\" pattern-separator
    // Expected: +26,931.4
    // Calls stub function - will fail until prod:DecimalFormatDecl is implemented
    let _ = stub_proddecimalformatdecl();
}

#[test]
fn decimal_format_12() {
    //          Purpose: Test default decimal-format on pattern-only characters, negati
    // XPath: declare default decimal-format digit=\"!\" pattern-separator
    // Expected: -26,931.4
    // Calls stub function - will fail until prod:DecimalFormatDecl is implemented
    let _ = stub_proddecimalformatdecl();
}

#[test]
fn decimal_format_13() {
    //          Purpose: Test default decimal-format on pattern-only characters, negati
    // XPath: declare default decimal-format digit=\"!\" pattern-separator
    // Expected: -26,931.4
    // Calls stub function - will fail until prod:DecimalFormatDecl is implemented
    let _ = stub_proddecimalformatdecl();
}

#[test]
fn decimal_format_14() {
    //          Purpose: Test specified result pattern for infinity.
    // XPath: declare decimal-format local:df2 infinity=\"off-the-scale\";
    // Expected: off-the-scale
    // Calls stub function - will fail until prod:DecimalFormatDecl is implemented
    let _ = stub_proddecimalformatdecl();
}

#[test]
fn decimal_format_15() {
    //          Purpose: Test specified result pattern for not-a-number.
    // XPath: declare decimal-format local:df2 NaN=\"non-numeric\";
    // Expected: non-numeric
    // Calls stub function - will fail until prod:DecimalFormatDecl is implemented
    let _ = stub_proddecimalformatdecl();
}

#[test]
fn decimal_format_16() {
    //          Purpose: Test of decimal-format per-mille format with character being c
    // XPath: declare default decimal-format  per-mille=\"m\";         for
    // Expected: 485.7m
    // Calls stub function - will fail until prod:DecimalFormatDecl is implemented
    let _ = stub_proddecimalformatdecl();
}

#[test]
fn decimal_format_17() {
    //          Purpose: Test decimal-format output character for negative, 2 patterns.
    // XPath: declare default decimal-format  minus-sign=\"_\";         fo
    // Expected: -26,931.4
    // Calls stub function - will fail until prod:DecimalFormatDecl is implemented
    let _ = stub_proddecimalformatdecl();
}

#[test]
fn decimal_format_18() {
    //          Purpose: Test decimal-format output character for negative, one pattern
    // XPath: declare default decimal-format minus-sign=\"_\";         for
    // Expected: _26,931.4
    // Calls stub function - will fail until prod:DecimalFormatDecl is implemented
    let _ = stub_proddecimalformatdecl();
}

#[test]
fn decimal_format_19() {
    //          Purpose: Test decimal-format declaration with an unprefixed name.
    // XPath: declare decimal-format myminus minus-sign=\"_\";          co
    // Expected: _26,931.4/-42,857.1
    // Calls stub function - will fail until prod:DecimalFormatDecl is implemented
    let _ = stub_proddecimalformatdecl();
}

#[test]
fn decimal_format_20() {
    //          Purpose: Test of decimal-format with qualified name. Unqualified name p
    // XPath: declare namespace foo=\"http://foo.ns\";         declare dec
    // Expected: 1*234!567
    // Calls stub function - will fail until prod:DecimalFormatDecl is implemented
    let _ = stub_proddecimalformatdecl();
}

#[test]
fn decimal_format_21() {
    //          Purpose: Decimal formats apply only within their own module.
    // XPath: import module namespace m=\"http://www.w3.org/TestModules/df
    // Expected: 123!456.789-123'456.789
    // Calls stub function - will fail until prod:DecimalFormatDecl is implemented
    let _ = stub_proddecimalformatdecl();
}

#[test]
fn decimal_format_33() {
    //          Purpose: Test of decimal-format declaration with two different named de
    // XPath: declare decimal-format fmt1 decimal-separator = \"^\";
    // Expected: 0123^450 and 0123~450
    // Calls stub function - will fail until prod:DecimalFormatDecl is implemented
    let _ = stub_proddecimalformatdecl();
}

#[test]
fn decimal_format_39() {
    // Purpose: Use escape sequence within decimal format value
    // XPath: declare default decimal-format minus-sign=\"-\";         for
    // Expected: -26,931.4
    // Calls stub function - will fail until prod:DecimalFormatDecl is implemented
    let _ = stub_proddecimalformatdecl();
}

#[test]
fn decimal_format_40() {
    // Purpose: It's OK to use '0' to represent NaN
    // XPath: declare default decimal-format NaN = \"0\";       format-num
    // Expected: 0
    // Calls stub function - will fail until prod:DecimalFormatDecl is implemented
    let _ = stub_proddecimalformatdecl();
}
