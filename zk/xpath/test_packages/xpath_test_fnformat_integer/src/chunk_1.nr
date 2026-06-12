//! Test chunk 1 for fn:format-integer
//! Contains 8 tests

use dep::xpath::stub_fnformat_integer;

#[test]
fn format_integer_066() {
    // format-integer - Ordinal number output request - Italian - using CLDR
    // XPath: format-integer(5, 'Ww;o(%spellout-ordinal-masculine)', 'it')
    // Expected: 'Quinto;Quinta'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_068() {
    // format-integer - zeroth
    // XPath: format-integer(0, 'w;o')
    // Expected: 'zeroth'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_069() {
    // format-integer - hundredth
    // XPath: format-integer(100, 'w;o')
    // Expected: 'one hundredth'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_070() {
    // format-integer - no pattern
    // XPath: format-integer(123456789, '0,0,00,0')
    // Expected: '12345,6,78,9'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_071() {
    // format-integer - pattern
    // XPath: format-integer(123456789, '00,00,00')
    // Expected: '1,23,45,67,89'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_072() {
    // format-integer - no pattern
    // XPath: format-integer(123456789, '000,00,00')
    // Expected: '12345,67,89'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_073() {
    // format-integer - picture grouping, non-ASCII grouping separator
    // XPath: format-integer(150000, '##?000')
    // Expected: '150?000'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_074() {
    // format-integer - decimal-digit-pattern with grouping separators, negative ordina
    // XPath: format-integer(-8500, '#,##1;o')
    // Expected: '-8,500th'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}
