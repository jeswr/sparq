//! Test chunk 1 for fn:format-number
//! Contains 50 tests

use dep::xpath::stub_fnformat_number;

#[test]
fn numberformat60p() {
    //          Test format-number() applied to large numbers (test how good the roundi
    // XPath: format-number(1E100,'#####################')
    // Expected: 1000000000000000000000000000000000000000
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat61() {
    //          format-number() applied to an empty sequence
    // XPath: format-number((),'###.###')
    // Expected: NaN
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat65() {
    //          Purpose: Test that a trailing decimalpoint is removed.
    // XPath: concat(format-number(1234e0, '0000.####'), '|',
    // Expected: 1234|1234
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat70() {
    //          Decimal separator and grouping separator in the astral planes
    // XPath: format-number(1234567890.123456,'?000?000')
    // Expected: 1?234?567?890?123
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat71() {
    //          Use Osmanya digits (non-BMP) in formatted output
    // XPath: format-number(1234567890.123456,'##########?.?#####')
    // Expected: ??????????.??????
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat72() {
    //          Check that overflow isn't an error
    // XPath: format-number(1234567890.123456,'000.000')
    // Expected: 1234567890.123
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat80() {
    //          PURPOSE: test format-number() with two arguments
    // XPath: format-number(12.34, '##.##')
    // Expected: 12.34
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat81() {
    //          PURPOSE: test format-number() with three arguments; also tests use of w
    // XPath: format-number(12.34, '0.000,00', ' b:test ')
    // Expected: 0.012,34
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat82() {
    //          PURPOSE: test format-number() where third argument is not known statica
    // XPath: format-number(12.34, '0.000,00', if (current-date() gt xs:da
    // Expected: 0.012,34
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat83() {
    //          PURPOSE: test format-number() where third argument is not known statica
    // XPath: format-number(12.34, '0.000,00', concat(if (current-date() l
    // Expected: 0.012,34
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat84() {
    //          PURPOSE: test format-number() with grouping separator
    // XPath: format-number(123456789.34, '#,###.##')
    // Expected: 123,456,789.34
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat85() {
    //          PURPOSE: test special case of format-number() imposing minWholePartSize
    // XPath: string-join((                 format-number(0, '#'),
    // Expected: 0|0|0|0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat86() {
    // In 3.0, () allowed as third argument (bug 14931)
    // XPath: format-number(0.4857,'###.###%', ())
    // Expected: 48.57%
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat87() {
    //          PURPOSE: test format-number() with EQName for third argument, known sta
    // XPath: format-number(12.34, '0.000,00', 'Q{http://a.ns/}test ')
    // Expected: 0.012,34
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat88() {
    //          PURPOSE: test format-number() with EQName for third argument, not known
    // XPath: format-number(12.34, '0.000,00', if (current-date() lt xs:da
    // Expected: 0.012,34
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat89() {
    //        PURPOSE: test format-number() with using '9' rather than '0' in picture
    // XPath: format-number(12.34, '9,999.99')
    // Expected: 0,012.34
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat90() {
    //        PURPOSE: test format-number() with using '9' rather than '0' in picture,
    // XPath: format-number(12.34, '#,##9.99')
    // Expected: 12.34
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat91() {
    //        PURPOSE: test format-number() with using '001' rather than '000' in pictu
    // XPath: format-number(42, '001')
    // Expected: 042
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat92() {
    //        PURPOSE: test format-number() with pattern in which fractional part has s
    // XPath: fn:format-number(12345.678, '9.')
    // Expected: 12346
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat93() {
    //        PURPOSE: test format-number() with pattern in which fractional part has s
    // XPath: fn:format-number(12345.000, '99999.')
    // Expected: 12345
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat94() {
    //        PURPOSE: test format-number() with pattern in which min integer part size
    // XPath: fn:format-number(12345.678, '.9')
    // Expected: 12345.7
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat95() {
    //        PURPOSE: test format-number() with pattern in which min integer part size
    // XPath: fn:format-number(0.2, '.#')
    // Expected: .2
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat101() {
    //        PURPOSE: test format-number() using default decimal-format declaration wi
    // XPath: format-number(12345.678, '9.9999e999')
    // Expected: 1.2346e004
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat103() {
    //        PURPOSE: test format-number() changing default decimal-format declaration
    // XPath: declare default decimal-format exponent-separator=\"E\";
    // Expected: 1.2346E004
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat105() {
    //        PURPOSE: test format-number() with named decimal-format declaration using
    // XPath: declare decimal-format exp-E exponent-separator=\"E\";
    // Expected: 1.2346E004
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat109() {
    //        PURPOSE: test format-number() with named format and bizarre exponent sepa
    // XPath: declare decimal-format exp-bizarre exponent-separator=\"?\";
    // Expected: 1.2346?004
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat112() {
    //        PURPOSE: test format-number() with named format in which exponent-separat
    // XPath: declare decimal-format exp-p percent=\"!\" exponent-separato
    // Expected: 1.2346%004
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat113() {
    //        PURPOSE: test format-number() with pattern in which exponent-separator is
    // XPath: let $x := fn:format-number(12345.678, '9.9999eDog')       re
    // Expected: 12345.6780eDog
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat115() {
    //        PURPOSE: test format-number() with an integer instead of a decimal
    // XPath: let $x := fn:format-number(12345, '9.9999e99')       return
    // Expected: 1.2345e04
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat116() {
    //        PURPOSE: test format-number() with a smaller integer instead of a decimal
    // XPath: let $x := fn:format-number(5, '9.9999e99')       return $x
    // Expected: 5.0000e00
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat117() {
    //        PURPOSE: test format-number() with a fractional decimal       (bug 28568
    // XPath: let $x := fn:format-number(.05, '9.9999e99')       return $x
    // Expected: 5.0000e-02
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat118() {
    //        PURPOSE: test format-number() with a real number
    // XPath: let $x := fn:format-number(123.45E7, '9.9999e99')       retu
    // Expected: 1.2345e09
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat119() {
    //        PURPOSE: test format-number() with a decimal having higher precision than
    // XPath: fn:format-number(0.900001000020000345, '9.99999999999999999e
    // Expected: 9.00001000020000345e-01
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat120() {
    //        PURPOSE: test format-number() with an integer having higher precision tha
    // XPath: fn:format-number(123456789012345678, '9.99999999999999999e99
    // Expected: 1.23456789012345678e17
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat123() {
    //          Use Osmanya digits (non-BMP) in scientific format output
    // XPath: format-number(1.0,'?.?e?')
    // Expected: ?.?e?
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat129() {
    //        PURPOSE: test format-number() with pattern in which integer part of manti
    // XPath: fn:format-number(12345.678, '999.99e99')
    // Expected: 123.46e02
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat130() {
    //        PURPOSE: test format-number() with pattern in which integer part of manti
    // XPath: fn:format-number(12345.678, '#99.99e99')
    // Expected: 12.35e03
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat131() {
    //        PURPOSE: test format-number() with pattern in which integer part of manti
    // XPath: fn:format-number(12345.678, '#.99e99')
    // Expected: 0.12e05
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat132() {
    //        PURPOSE: test format-number() with pattern in which min integer part size
    // XPath: fn:format-number(0045.234, '#.#e99')
    // Expected: 0.5e02
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat133() {
    //        PURPOSE: test format-number() with pattern in which min integer part size
    // XPath: fn:format-number(0.2, '.#e9')
    // Expected: .2e0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat134() {
    //        PURPOSE: test format-number() with pattern in which min integer part size
    // XPath: fn:format-number(0.2, '.999e9')
    // Expected: .200e0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat135() {
    //        PURPOSE: test format-number() with pattern in which fractional part has s
    // XPath: fn:format-number(0.2, '#.e9')
    // Expected: 0.2e0
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat136() {
    //        PURPOSE: test format-number() with pattern in which fractional part has s
    // XPath: fn:format-number(0.2, '9e9')
    // Expected: 2e-1
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat137() {
    //        PURPOSE: test format-number() with pattern in which fractional part has s
    // XPath: fn:format-number(12345.678, '999.e99')
    // Expected: 123e02
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat138() {
    //        PURPOSE: test format-number() with pattern in which fractional part has s
    // XPath: fn:format-number(12345.678, '999e9')
    // Expected: 123e2
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat140() {
    //        PURPOSE: test format-number() - negative number into scientific format
    // XPath: fn:format-number(-12345.678, '999.99e99')
    // Expected: -123.46e02
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat141() {
    //        PURPOSE: test format-number() - exponent of result is negative
    // XPath: fn:format-number(0.00012345678, '9.99e99')
    // Expected: 1.23e-04
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat142() {
    //        PURPOSE: test format-number() - negative number, and exponent of result i
    // XPath: fn:format-number(-0.00012345678, '9.99e99')
    // Expected: -1.23e-04
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat143() {
    //        PURPOSE: test format-number() with pattern in which 2nd exponent-separato
    // XPath: fn:format-number(12345.678, '9.9999e99end')
    // Expected: 1.2346e04end
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}

#[test]
fn numberformat144() {
    //        PURPOSE: test format-number() with pattern in which 2nd exponent-separato
    // XPath: fn:format-number(12345.678, '9.9999e99e')
    // Expected: 1.2346e04e
    // Calls stub function - will fail until fn:format-number is implemented
    let _ = stub_fnformat_number();
}
