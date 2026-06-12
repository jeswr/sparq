//! Test chunk 0 for fn:QName
//! Contains 3 tests

use dep::xpath::stub_fnqname;

#[test]
fn k_expandedqnameconstructfunc_4() {
    //  A test whose essence is: `QName('http://www.w3.org/2005/xpath-functions', 'pref
    // XPath: QName(\"http://www.w3.org/2005/xpath-functions\", \"prefix:l
    // Expected: true
    // Calls stub function - will fail until fn:QName is implemented
    let _ = stub_fnqname();
}

#[test]
fn k_expandedqnameconstructfunc_9() {
    //  A test whose essence is: `QName('', 'local') eq xs:QName('local')`.
    // XPath: QName(\"\", \"local\") eq xs:QName(\"local\")
    // Expected: true
    // Calls stub function - will fail until fn:QName is implemented
    let _ = stub_fnqname();
}

#[test]
fn k_expandedqnameconstructfunc_10() {
    //  A test whose essence is: `QName((), 'local') eq xs:QName('local')`.
    // XPath: QName((), \"local\") eq xs:QName(\"local\")
    // Expected: true
    // Calls stub function - will fail until fn:QName is implemented
    let _ = stub_fnqname();
}
