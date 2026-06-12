//! Test chunk 2 for fn:format-number
//! Contains 50 tests

use dep::xpath::stub_fnformat_number;

#[test]
fn numberformat145() {
    //        PURPOSE: test format-number() with pattern with exponent-separator and su
    // XPath: fn:format-number(12345.678, '9.9999e99dog')
    // Expected: 1.2346e04dog
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat146() {
    //        PURPOSE: test format-number() with pattern with exponent-separator, prefi
    // XPath: fn:format-number(12345.678, 'end9.9999e99end')
    // Expected: end1.2346e04end
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat154() {
    //        PURPOSE: test format-number() setting zero-digit
    // XPath: declare default decimal-format zero-digit = \"?\";       for
    // Expected: ?
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat155() {
    //        PURPOSE: test format-number() setting zero-digit
    // XPath: declare default decimal-format zero-digit = \"?\";       for
    // Expected: ?
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat157() {
    //        PURPOSE: test format-number() with pattern with grouping-separators in fr
    // XPath: fn:format-number(12345.6789012345, '#.#,##,#')
    // Expected: 12345.6,78,9
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat158() {
    //        PURPOSE: test format-number() with pattern with grouping-separators in fr
    // XPath: fn:format-number(12345.6789012345, '#.##,##,##')
    // Expected: 12345.67,89,01
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat201() {
    //        format-number() with pattern in which min integer part size is zero, frac
    // XPath: fn:format-number(0.2, '#')
    // Expected: 0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat202() {
    //        format-number() with pattern in which min integer part size is zero, frac
    // XPath: fn:format-number(1.2, '#')
    // Expected: 1
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat203() {
    //        format-number() with pattern in which min integer part size is one, frac
    // XPath: fn:format-number(0.2, '0')
    // Expected: 0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat204() {
    //        format-number() with pattern in which min integer part size is one, frac
    // XPath: fn:format-number(1.2, '0')
    // Expected: 1
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat205() {
    //        test format-number() with pattern in which min integer part size is zero,
    // XPath: fn:format-number(0.2, '.#')
    // Expected: .2
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat206() {
    //        format-number() with pattern in which min integer part size is zero, frac
    // XPath: fn:format-number(1.2, '.#')
    // Expected: 1.2
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat207() {
    //        test format-number() with pattern in which min integer part size is zero,
    // XPath: fn:format-number(0.2, '.0')
    // Expected: .2
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat208() {
    //        test format-number() with pattern in which min integer part size is zero,
    // XPath: fn:format-number(1.2, '.0')
    // Expected: 1.2
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat209() {
    //        test format-number() with pattern in which min integer part size is zero,
    // XPath: fn:format-number(0.2, '#.#')
    // Expected: .2
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat210() {
    //        format-number() with pattern in which min integer part size is zero, frac
    // XPath: fn:format-number(1.2, '#.#')
    // Expected: 1.2
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat211() {
    //        test format-number() with pattern in which min integer part size is one,
    // XPath: fn:format-number(0.2, '0.0')
    // Expected: 0.2
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat212() {
    //        test format-number() with pattern in which min integer part size is one,
    // XPath: fn:format-number(1.2, '0.0')
    // Expected: 1.2
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat213() {
    //        format-number() with pattern in which min integer part size is one, frac
    // XPath: fn:format-number(0.2, '0.#')
    // Expected: 0.2
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat214() {
    //        format-number() with pattern in which min integer part size is one, frac
    // XPath: fn:format-number(1.2, '0.#')
    // Expected: 1.2
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat215() {
    //        format-number() with pattern in which min integer part size is zero, frac
    // XPath: fn:format-number(0.2, '#.0')
    // Expected: .2
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat216() {
    //        test format-number() with pattern in which min integer part size is zero,
    // XPath: fn:format-number(1.2, '#.0')
    // Expected: 1.2
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat217() {
    //        format-number() with pattern in which integer part contains no decimal-di
    // XPath: fn:format-number(0.2, '#.')
    // Expected: 0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat218() {
    //        test format-number() with pattern in which min integer part size is zero,
    // XPath: fn:format-number(1.2, '#.')
    // Expected: 1
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat219() {
    //        test format-number() with pattern in which min integer part size is one,
    // XPath: fn:format-number(0.2, '0.')
    // Expected: 0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat220() {
    //        format-number() with pattern in which min integer part size is one, frac
    // XPath: fn:format-number(1.2, '0.')
    // Expected: 1
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat221() {
    //        test format-number() with pattern in which min integer part size is zero
    // XPath: fn:format-number(0.2, 'this.#')
    // Expected: this.2
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat222() {
    //        format-number() with pattern in which min integer part size is zero
    // XPath: fn:format-number(1.2, 'this.#')
    // Expected: this1.2
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat223() {
    //        test format-number() with pattern in which min integer part size is zero
    // XPath: fn:format-number(0.2, 'this.0')
    // Expected: this.2
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat224() {
    //        test format-number() with pattern in which min integer part size is zero
    // XPath: fn:format-number(1.2, 'this.0')
    // Expected: this1.2
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat225() {
    //        test format-number() with pattern in which min integer part size is zero,
    // XPath: fn:format-number(0.2, '#.suffix')
    // Expected: 0suffix
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat226() {
    //        test format-number() with pattern in which min integer part size is zero,
    // XPath: fn:format-number(1.2, '#.suffix')
    // Expected: 1suffix
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat227() {
    //        test format-number() with pattern in which min integer part size is three
    // XPath: fn:format-number(0.2, '000.0')
    // Expected: 000.2
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat228() {
    //        format-number() with pattern in which min integer part size is zero, frac
    // XPath: fn:format-number(0, '#.#')
    // Expected: .0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat229() {
    //        format-number() with pattern in which min integer part size is zero, frac
    // XPath: fn:format-number(0, '.#')
    // Expected: .0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat230() {
    //        format-number() with pattern in which min integer part size is zero, frac
    // XPath: fn:format-number(xs:integer(0), '###.###')
    // Expected: .0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat231() {
    //        format-number() with pattern in which min integer part size is zero,
    // XPath: fn:format-number(0.2, '#e0')
    // Expected: 0.2e0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat232() {
    //        format-number() with pattern in which min integer part size is zero,
    // XPath: fn:format-number(1.2, '#e0')
    // Expected: 0.1e1
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat233() {
    //        format-number() with pattern in which min integer part size is one,
    // XPath: fn:format-number(0.2, '0e0')
    // Expected: 2e-1
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat234() {
    //        format-number() with pattern in which min integer part size is one,
    // XPath: fn:format-number(1.2, '0e0')
    // Expected: 1e0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat235() {
    //        test format-number() with pattern in which min integer part size is zero,
    // XPath: fn:format-number(0.2, '.#e0')
    // Expected: .2e0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat236() {
    //        format-number() with pattern in which min integer part size is zero,
    // XPath: fn:format-number(1.2, '.#e0')
    // Expected: .1e1
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat237() {
    //        test format-number() with pattern in which min integer part size is zero,
    // XPath: fn:format-number(0.2, '.0e0')
    // Expected: .2e0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat238() {
    //        test format-number() with pattern in which min integer part size is zero,
    // XPath: fn:format-number(1.2, '.0e0')
    // Expected: .1e1
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat239() {
    //        test format-number() with pattern in which min integer part size is zero,
    // XPath: fn:format-number(0.2, '#.#e0')
    // Expected: 0.2e0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat240() {
    //        format-number() with pattern in which min integer part size is zero,
    // XPath: fn:format-number(1.2, '#.#e0')
    // Expected: 0.1e1
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat241() {
    //        test format-number() with pattern in which min integer part size is one,
    // XPath: fn:format-number(0.2, '0.0e0')
    // Expected: 2.0e-1
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat242() {
    //        test format-number() with pattern in which min integer part size is one,
    // XPath: fn:format-number(1.2, '0.0e0')
    // Expected: 1.2e0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat243() {
    //        format-number() with pattern in which min integer part size is one,
    // XPath: fn:format-number(0.2, '0.#e0')
    // Expected: 2e-1
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat244() {
    //        format-number() with pattern in which min integer part size is one,
    // XPath: fn:format-number(1.2, '0.#e0')
    // Expected: 1.2e0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}
