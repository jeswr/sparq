//! Test chunk 0 for fn:local-name-from-QName
//! Contains 11 tests

use dep::xpath::stub_fnlocal_name_from_qname;

#[test]
fn localnamefromqnamefunc001() {
    //  Test function fn:local-name-from-QName. Using QName element as input
    // XPath: fn:local-name-from-QName(fn:QName(\"http://www.example.com/Q
    // Expected: bar
    // Calls stub function - will fail until fn:local-name-from-QName is implemented
    let _ = stub_fnlocal_name_from_qname();
}

#[test]
fn localnamefromqnamefunc002() {
    //  Test function fn:local-name-from-QName. Using QName attribute as input
    // XPath: fn:local-name-from-QName(fn:QName(\"http://www.example.com/Q
    // Expected: bar
    // Calls stub function - will fail until fn:local-name-from-QName is implemented
    let _ = stub_fnlocal_name_from_qname();
}

#[test]
fn localnamefromqnamefunc003() {
    //  Test function fn:local-name-from-QName. Using element derived from QName as inp
    // XPath: fn:local-name-from-QName(fn:QName(\"http://www.example.com/Q
    // Expected: barDeriv
    // Calls stub function - will fail until fn:local-name-from-QName is implemented
    let _ = stub_fnlocal_name_from_qname();
}

#[test]
fn localnamefromqnamefunc004() {
    //  Test function fn:local-name-from-QName. Using attribute derived from QName as i
    // XPath: fn:local-name-from-QName(fn:QName(\"http://www.example.com/Q
    // Expected: barDeriv
    // Calls stub function - will fail until fn:local-name-from-QName is implemented
    let _ = stub_fnlocal_name_from_qname();
}

#[test]
fn localnamefromqnamefunc005() {
    //  Test function fn:local-name-from-QName. Check return type of function is xs:NCN
    // XPath: fn:local-name-from-QName(fn:QName(\"http://www.example.com/Q
    // Expected: true
    // Calls stub function - will fail until fn:local-name-from-QName is implemented
    let _ = stub_fnlocal_name_from_qname();
}

#[test]
fn localnamefromqnamefunc018() {
    //  Test function fn:local-name-from-QName. Explicit call to data to extract simple
    // XPath: fn:local-name-from-QName(fn:QName(\"http://www.example.com/Q
    // Expected: bar
    // Calls stub function - will fail until fn:local-name-from-QName is implemented
    let _ = stub_fnlocal_name_from_qname();
}

#[test]
fn localnamefromqnamefunc019() {
    //  Test function fn:local-name-from-QName. Conditional expression as parameter whe
    // XPath: fn:local-name-from-QName( if( true() ) then fn:QName(\"http:
    // Expected: bar
    // Calls stub function - will fail until fn:local-name-from-QName is implemented
    let _ = stub_fnlocal_name_from_qname();
}

#[test]
fn localnamefromqnamefunc020() {
    //  Test function fn:local-name-from-QName. Conditional expression as parameter whe
    // XPath: fn:local-name-from-QName( if( false() ) then fn:QName(\"http
    // Expected: bar
    // Calls stub function - will fail until fn:local-name-from-QName is implemented
    let _ = stub_fnlocal_name_from_qname();
}

#[test]
fn fn_local_name_from_qname_024() {
    //  Evaluation of fn-local-name-from-QName applied to node-name().
    // XPath: fn:local-name-from-QName(node-name(/*))
    // Expected: works
    // Calls stub function - will fail until fn:local-name-from-QName is implemented
    let _ = stub_fnlocal_name_from_qname();
}

#[test]
fn k_localnamefromqnamefunc_3() {
    //  A test whose essence is: `empty(local-name-from-QName( () ))`.
    // XPath: empty(local-name-from-QName( () ))
    // Expected: true
    // Calls stub function - will fail until fn:local-name-from-QName is implemented
    let _ = stub_fnlocal_name_from_qname();
}

#[test]
fn k_localnamefromqnamefunc_4() {
    //  A test whose essence is: `local-name-from-QName( QName('example.com/', 'pre:lna
    // XPath: local-name-from-QName( QName(\"example.com/\", \"pre:lname\"
    // Expected: true
    // Calls stub function - will fail until fn:local-name-from-QName is implemented
    let _ = stub_fnlocal_name_from_qname();
}
