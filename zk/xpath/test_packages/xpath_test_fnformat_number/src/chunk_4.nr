//! Test chunk 4 for fn:format-number
//! Contains 37 tests

use dep::xpath::stub_fnformat_number;

#[test]
fn cbcl_fn_format_number_014() {
    // Check grouping formating in format-number.
    // XPath: format-number(xs:decimal('123456789'), '#,##,##,00')
    // Expected: 1,23,45,67,89
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_015() {
    // Check grouping formating in format-number.
    // XPath: format-number(xs:decimal('123456789'), '###,##,00')
    // Expected: 12345,67,89
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_016() {
    // Check grouping formating in format-number.
    // XPath: format-number(xs:decimal('123456789'), '####,###,##,0')
    // Expected: 123,456,78,9
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_018() {
    // Check grouping formating in format-number.
    // XPath: format-number(xs:decimal('0'), '.#')
    // Expected: .0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_019() {
    // Check grouping formating in format-number.
    // XPath: format-number(xs:decimal('0'), '#.')
    // Expected: 0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_020() {
    // Check grouping formating in format-number.
    // XPath: format-number(xs:decimal('0'), '.0')
    // Expected: .0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_021() {
    // Check grouping formating in format-number.
    // XPath: format-number(xs:decimal('0'), '0.')
    // Expected: 0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_022() {
    // Check grouping formating in format-number.
    // XPath: format-number(xs:decimal('1.10203040506070809'),
    // Expected: 110.20304050607080900%
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_023() {
    // Check rounding in format-number.
    // XPath: format-number(xs:double('123456789'), '#,#')
    // Expected: 1,2,3,4,5,6,7,8,9
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_024() {
    // Check rounding in format-number.
    // XPath: format-number(xs:double('123456789'), '#,##,00')
    // Expected: 1,23,45,67,89
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_025() {
    // Check rounding in format-number.
    // XPath: format-number(xs:double('123456789'), '##,##,00')
    // Expected: 1,23,45,67,89
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_026() {
    // Check rounding in format-number.
    // XPath: format-number(xs:double('123456789'), '#,##,##,00')
    // Expected: 1,23,45,67,89
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_027() {
    // Check rounding in format-number.
    // XPath: format-number(xs:double('123456789'), '###,##,00')
    // Expected: 12345,67,89
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_028() {
    // Check rounding in format-number.
    // XPath: format-number(xs:double('123456789'), '####,###,##,0')
    // Expected: 123,456,78,9
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_029() {
    // Check rounding in format-number.
    // XPath: format-number(xs:double('123456789'), '####,###,##,0')
    // Expected: 123,456,78,9
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_030() {
    // Check grouping formating in format-number.
    // XPath: format-number(xs:double('0'), '.#')
    // Expected: .0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_031() {
    // Check grouping formating in format-number.
    // XPath: format-number(xs:double('0'), '#.')
    // Expected: 0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_032() {
    // Check grouping formating in format-number.
    // XPath: format-number(xs:double('0'), '.0')
    // Expected: .0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_033() {
    // Check grouping formating in format-number.
    // XPath: format-number(xs:double('0'), '0.')
    // Expected: 0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_034() {
    // Check edge case in format-number.
    // XPath: format-number(xs:double('1e308'), '0%')
    // Expected: Infinity%
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_035() {
    // Check per-mille multiplier in format-number.
    // XPath: substring-after(               format-number(xs:double('1.10
    // Expected: true
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_036() {
    // Check edge case in format-number.
    // XPath: format-number(xs:double('-1e308'), '0%')
    // Expected: -Infinity%
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_038() {
    // Check  with small doubles in format-number.
    // XPath: format-number(xs:double('0.1'), '0.00')
    // Expected: 0.10
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_039() {
    // Check with small doubles in format-number.
    // XPath: format-number(xs:double('0.01'), '0.00')
    // Expected: 0.01
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_040() {
    // Check with small doubles in format-number.
    // XPath: format-number(xs:double('0'), '#.00')
    // Expected: .00
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_041() {
    // Check with small doubles in format-number.
    // XPath: format-number(xs:double('0'), '#.##')
    // Expected: .0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_042() {
    // Check with small doubles in format-number.
    // XPath: format-number(xs:double('0'), '0.00')
    // Expected: 0.00
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_043() {
    // Check  with small decimals in format-number.
    // XPath: format-number(xs:double('0.1'), '0.00')
    // Expected: 0.10
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_044() {
    // Check with small decimals in format-number.
    // XPath: format-number(xs:double('0.01'), '0.00')
    // Expected: 0.01
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_045() {
    // Check with small decimals in format-number.
    // XPath: format-number(xs:double('0.01'), '0.00')
    // Expected: 0.01
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_046() {
    // Check with small decimals in format-number.
    // XPath: format-number(xs:double('0.01'), '0.00')
    // Expected: 0.01
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_047() {
    // Check with small decimals in format-number.
    // XPath: format-number(xs:double('0'), '#.00')
    // Expected: .00
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_048() {
    // Check with small decimals in format-number.
    // XPath: format-number(xs:decimal('0'), '#.00')
    // Expected: .00
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_049() {
    // Check with small decimals in format-number.
    // XPath: format-number(xs:decimal('0'), '###,###.##')
    // Expected: .0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_050() {
    // Check with small decimals in format-number.
    // XPath: format-number(xs:double('0'), '###,###.##')
    // Expected: .0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_051() {
    // Check with small decimals in format-number.
    // XPath: format-number(xs:decimal('0.1'), '###,###.##')
    // Expected: .1
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_052() {
    // Check with small decimals in format-number.
    // XPath: format-number(xs:double('0.1'), '###,###.##')
    // Expected: .1
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}
