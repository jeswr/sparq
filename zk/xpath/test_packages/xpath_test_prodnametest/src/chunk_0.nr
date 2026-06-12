//! Test chunk 0 for prod:NameTest
//! Contains 17 tests

use dep::xpath::stub_prodnametest;

#[test]
fn nodetest003() {
    //  Simple test for * node test
    // XPath: /*/*[1]/name()
    // Expected: Customers
    // Calls stub function - will fail until prod:NameTest is implemented
    let _ = stub_prodnametest();
}

#[test]
fn nametest_8() {
    //  Name test that examines 'b' for a newly constructed element and used as argumen
    // XPath: let $var := <a attr1 = \"abc1\"><b attr2 = \"abc2\">context2
    // Expected: b
    // Calls stub function - will fail until prod:NameTest is implemented
    let _ = stub_prodnametest();
}

#[test]
fn nametest_9() {
    //  Name test that attempts to select non-existent nodes from a newly constructed.
    // XPath: let $var := <a attr1 = \"abc1\"><b attr2 = \"abc2\">context2
    // Expected: 0
    // Calls stub function - will fail until prod:NameTest is implemented
    let _ = stub_prodnametest();
}

#[test]
fn nametest_10() {
    //  Name test that selects an 'child::b' of a newly created element node and whose
    // XPath: declare default element namespace \"http://www.example.org/e
    // Expected: http://www.example.org/examples
    // Calls stub function - will fail until prod:NameTest is implemented
    let _ = stub_prodnametest();
}

#[test]
fn nametest_11() {
    //  Name test that selects an 'child::b' of a newly created element node and whose
    // XPath: let $var := <a attr1 = \"abc1\"><b attr2 = \"abc2\">context2
    // Expected: 1
    // Calls stub function - will fail until prod:NameTest is implemented
    let _ = stub_prodnametest();
}

#[test]
fn nametest_14() {
    //  Name test that selects an 'child::*' and 'child::day' (same nodes) of an elemen
    // XPath: let $var := /works[1]/child::employee[12]/overtime return fn
    // Expected: 0
    // Calls stub function - will fail until prod:NameTest is implemented
    let _ = stub_prodnametest();
}

#[test]
fn nametest_15() {
    //  Name test that selects all the children of the context node with 'child::*'.
    // XPath: let $var := /works return fn:count($var/child::*)
    // Expected: 13
    // Calls stub function - will fail until prod:NameTest is implemented
    let _ = stub_prodnametest();
}

#[test]
fn nametest_16() {
    //  Name test that selects all the children of the context node with 'child::employ
    // XPath: let $var := /works return fn:count($var/child::employee)
    // Expected: 13
    // Calls stub function - will fail until prod:NameTest is implemented
    let _ = stub_prodnametest();
}

#[test]
fn nametest_17() {
    //  Name test that selects all the children of the context node with 'child::*:empl
    // XPath: let $var := /works return fn:count($var/child::*:employee)
    // Expected: 13
    // Calls stub function - will fail until prod:NameTest is implemented
    let _ = stub_prodnametest();
}

#[test]
fn nametest_19() {
    //  Nametest using Q{uri}local syntax
    // XPath: let $e := <a attr1 = \"abc1\" xmlns=\"http://example.org/nam
    // Expected: true
    // Calls stub function - will fail until prod:NameTest is implemented
    let _ = stub_prodnametest();
}

#[test]
fn nametest_20() {
    //  Nametest using Q{}local syntax
    // XPath: let $e := <a attr1 = \"abc1\"/>        return exists($e/self
    // Expected: true
    // Calls stub function - will fail until prod:NameTest is implemented
    let _ = stub_prodnametest();
}

#[test]
fn nametest_21() {
    //  Nametest using Q{uri}* syntax
    // XPath: let $e := <a attr1 = \"abc1\" xmlns=\"http://example.org/nam
    // Expected: true
    // Calls stub function - will fail until prod:NameTest is implemented
    let _ = stub_prodnametest();
}

#[test]
fn nametest_22() {
    //  Nametest using Q{}* syntax
    // XPath: let $e := <a attr1 = \"abc1\"/>        return exists($e/self
    // Expected: true
    // Calls stub function - will fail until prod:NameTest is implemented
    let _ = stub_prodnametest();
}

#[test]
fn k2_nametest_12() {
    //  A nametest being 'xml:space'.
    // XPath: declare variable $var := <elem xml:space=\"default\"/>; $var
    // Expected: true
    // Calls stub function - will fail until prod:NameTest is implemented
    let _ = stub_prodnametest();
}

#[test]
fn k2_nametest_13() {
    //  A nametest being 'xml:*'.
    // XPath: declare variable $var := <elem xml:space=\"preserve\"/>; str
    // Expected: preserve
    // Calls stub function - will fail until prod:NameTest is implemented
    let _ = stub_prodnametest();
}

#[test]
fn k2_nametest_22() {
    //  A processing-instruction() with an invalid NCName in a string literal.
    // XPath: empty(let $e := <a b =\"content\"><?b asd?><b/></a> return $
    // Expected: false
    // Calls stub function - will fail until prod:NameTest is implemented
    let _ = stub_prodnametest();
}

#[test]
fn k2_nametest_48() {
    //  Use 'xmlns' as an attribute name test.
    // XPath: declare variable $i := <e xmlns=\"http://example.com/\"/>; e
    // Expected: true
    // Calls stub function - will fail until prod:NameTest is implemented
    let _ = stub_prodnametest();
}
