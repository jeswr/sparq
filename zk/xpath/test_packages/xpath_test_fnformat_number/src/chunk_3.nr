//! Test chunk 3 for fn:format-number
//! Contains 50 tests

use dep::xpath::stub_fnformat_number;

#[test]
fn numberformat245() {
    //        format-number() with pattern in which min integer part size is zero,
    // XPath: fn:format-number(0.2, '#.0e0')
    // Expected: 0.2e0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat246() {
    //        test format-number() with pattern in which min integer part size is zero,
    // XPath: fn:format-number(1.2, '#.0e0')
    // Expected: 0.1e1
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat247() {
    //        format-number() with pattern in which integer part contains no decimal-di
    // XPath: fn:format-number(0.2, '#.e0')
    // Expected: 0.2e0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat248() {
    //        test format-number() with pattern in which min integer part size is zero,
    // XPath: fn:format-number(1.2, '#.e0')
    // Expected: 0.1e1
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat249() {
    //        test format-number() with pattern in which min integer part size is one,
    // XPath: fn:format-number(0.2, '0.e0')
    // Expected: 2e-1
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat250() {
    //        format-number() with pattern in which min integer part size is one,
    // XPath: fn:format-number(1.2, '0.e0')
    // Expected: 1e0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat251() {
    //        test format-number() with pattern in which min integer part size is zero
    // XPath: fn:format-number(0.2, 'this.#e0')
    // Expected: this.2e0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat252() {
    //        format-number() with pattern in which min integer part size is zero
    // XPath: fn:format-number(1.2, 'this.#e0')
    // Expected: this.1e1
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat253() {
    //        test format-number() with pattern in which min integer part size is zero
    // XPath: fn:format-number(0.2, 'this.0e0')
    // Expected: this.2e0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat254() {
    //        test format-number() with pattern in which min integer part size is zero
    // XPath: fn:format-number(1.2, 'this.0e0')
    // Expected: this.1e1
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat255() {
    //        test format-number() with pattern in which min integer part size is three
    // XPath: fn:format-number(0.2, '000.0e0')
    // Expected: 200.0e-3
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat261() {
    //        test format-number() with pattern in which min integer part size is zero,
    // XPath: fn:format-number(0.002, '.0e0')
    // Expected: .2e-2
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat262() {
    //        test format-number() with pattern in which min integer part size is zero,
    // XPath: fn:format-number(0.002, '.000e0')
    // Expected: .200e-2
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat263() {
    //        test format-number() with pattern in which min integer part size is zero,
    // XPath: fn:format-number(0.002, '#.###e0')
    // Expected: 0.2e-2
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat301() {
    //        exponential notation: unexpected number of digits in integer part after r
    // XPath: fn:format-number(0.99999999, '.#e0')
    // Expected: 1.0e0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat302() {
    //        exponential notation: unexpected number of digits in integer part after r
    // XPath: fn:format-number(0.99999999, '.0e0')
    // Expected: 1.0e0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat303() {
    //        exponential notation: unexpected number of digits in integer part after r
    // XPath: fn:format-number(0.99999999, '#.#e0')
    // Expected: 1e0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat304() {
    //        exponential notation: unexpected number of digits in integer part after r
    // XPath: fn:format-number(0.99999999, '0.0e0')
    // Expected: 10.0e-1
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat310() {
    // Single grouping separator in pattern: regular or not?
    // XPath: format-number(642120, '####,##')
    // Expected: 6421,20
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat311() {
    // Single grouping separator in pattern: regular or not?
    // XPath: format-number(642120, '###,##')
    // Expected: 6421,20
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat312() {
    // Single grouping separator in pattern: regular or not?
    // XPath: format-number(642120, '##,##')
    // Expected: 64,21,20
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat313() {
    // Single grouping separator in pattern: regular or not?
    // XPath: format-number(642120, '#,##')
    // Expected: 64,21,20
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat314() {
    // Single grouping separator in pattern: regular or not?
    // XPath: format-number(642120, '0000,00')
    // Expected: 6421,20
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat315() {
    // Single grouping separator in pattern: regular or not?
    // XPath: format-number(642120, '000,00')
    // Expected: 6421,20
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat316() {
    // Single grouping separator in pattern: regular or not?
    // XPath: format-number(642120, '00,00')
    // Expected: 64,21,20
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat317() {
    // Single grouping separator in pattern: regular or not?
    // XPath: format-number(642120, '0,00')
    // Expected: 64,21,20
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat318() {
    // Two grouping separators in pattern: regular or not?
    // XPath: format-number(642120, '#,#,#')
    // Expected: 6,4,2,1,2,0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat319() {
    // Two grouping separators in pattern: regular or not?
    // XPath: format-number(642120, '##,#,#')
    // Expected: 6421,2,0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat320() {
    // Grouping separator before first digit symbol
    // XPath: format-number(2001, ',##0') || ' ~ ' || format-number(897,',
    // Expected: 2,001 ~ 897 ~ 3,000,000
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat321() {
    // Exponential representation of zero
    // XPath: format-number(0.0e0, '0.0e0')
    // Expected: 0.0e0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat322() {
    // Exponential representation of zero
    // XPath: format-number(-0.0e0, '0.0e0')
    // Expected: -0.0e0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat323() {
    // Exponential representation of zero
    // XPath: format-number(0.0e0, '0.0e01')
    // Expected: 0.0e00
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat324() {
    // Exponential representation of zero
    // XPath: format-number(0, '0.0e01')
    // Expected: 0.0e00
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat325() {
    // Exponential representation of zero
    // XPath: format-number(0, '0e01')
    // Expected: 0e00
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat326() {
    // Exponential representation of zero
    // XPath: format-number(0, '#.#e9')
    // Expected: 0e0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat327() {
    // Exponential representation of zero
    // XPath: format-number(0.00000000, '#.0e9')
    // Expected: 0.0e0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformatnan() {
    //          Creator: Zhen Hua Liu         Purpose: NaN input from number('abc').
    // XPath: format-number(number(\"abc\"),'#############')
    // Expected: NaN
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_001() {
    // If the input number is NaN (not a number), the result is the specified NaN-symbo
    // XPath: format-number(number(\"abc\"),'PREFIX#SUFFIX')
    // Expected: NaN
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_002() {
    // Negative zero is taken as negative, positive zero as positive..
    // XPath: format-number(0 div xs:double(\"INF\"), '#')
    // Expected: 0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_003() {
    // Negative zero is taken as negative, positive zero as positive..
    // XPath: format-number(0 div xs:double(\"-INF\"), '#')
    // Expected: -0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_004() {
    // If the input number is positive or negative infinity, the result is the concaten
    // XPath: format-number(1 div 0e0,'PREFIX#SUFFIX')
    // Expected: PREFIXINFSUFFIX
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_005() {
    // If the input number is positive or negative infinity, the result is the concaten
    // XPath: format-number(-1 div 0e0,'PREFIX#SUFFIX')
    // Expected: -PREFIXINFSUFFIX
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_006() {
    // If the input number is positive or negative infinity, the result is the concaten
    // XPath: format-number(-1 div 0e0,'PREFIX#SUFFIX;prefix#suffix')
    // Expected: prefixINFsuffix
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_007() {
    // If the input number is positive or negative infinity, the result is the concaten
    // XPath: format-number(-1 div 0e0,'PREFIX#SUFFIX;prefix#suffix')
    // Expected: prefixINFsuffix
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_008() {
    // Check that the result doesn't start with a grouping separator.
    // XPath: format-number(1e2,'#,000')
    // Expected: 100
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_009() {
    // Check that the result doesn't start with a grouping separator.
    // XPath: format-number(100,'#,000')
    // Expected: 100
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_010() {
    // Check rounding in format-number.
    // XPath: format-number(xs:double('0.9952832031249997'), '###,##0.00')
    // Expected: 1.00
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_011() {
    // Check grouping formating in format-number.
    // XPath: format-number(xs:decimal('123456789'), '#,#')
    // Expected: 1,2,3,4,5,6,7,8,9
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_012() {
    // Check grouping formating in format-number.
    // XPath: format-number(xs:decimal('123456789'), '#,##,00')
    // Expected: 1,23,45,67,89
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn cbcl_fn_format_number_013() {
    // Check grouping formating in format-number.
    // XPath: format-number(xs:decimal('123456789'), '##,##,00')
    // Expected: 1,23,45,67,89
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}
