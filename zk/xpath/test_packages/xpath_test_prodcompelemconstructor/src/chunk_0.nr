//! Test chunk 0 for prod:CompElemConstructor
//! Contains 20 tests

use dep::xpath::stub_prodcompelemconstructor;

#[test]
fn constr_compelem_adjtext_1() {
    //  merge adjacent atomic values to text node
    // XPath: count((element elem {1, 'string', 1,2e3})/text())
    // Expected: 1
    // Calls stub function - will fail until prod:CompElemConstructor is implemented
    let _ = stub_prodcompelemconstructor();
}

#[test]
fn constr_compelem_adjtext_2() {
    //  merge adjacent atomic values and text nodes to text node
    // XPath: count((element elem {1, //text(), 'string'})/text())
    // Expected: 1
    // Calls stub function - will fail until prod:CompElemConstructor is implemented
    let _ = stub_prodcompelemconstructor();
}

#[test]
fn constr_compelem_adjtext_3() {
    //  merge adjacent atomic values between other nodes to text node
    // XPath: count((element elem {1, 2, <a/>, 3, 4, <b/>, 5, 6})/text())
    // Expected: 3
    // Calls stub function - will fail until prod:CompElemConstructor is implemented
    let _ = stub_prodcompelemconstructor();
}

#[test]
fn constr_compelem_nodeid_1() {
    //  Copied element node has new node identity
    // XPath: for $x in <a/>, $y in element elem {$x} return exactly-one($
    // Expected: false
    // Calls stub function - will fail until prod:CompElemConstructor is implemented
    let _ = stub_prodcompelemconstructor();
}

#[test]
fn constr_compelem_nodeid_2() {
    //  Copied attribute node has new node identity
    // XPath: for $x in <a b=\"b\"/>, $y in element elem {$x/@b} return $y
    // Expected: false
    // Calls stub function - will fail until prod:CompElemConstructor is implemented
    let _ = stub_prodcompelemconstructor();
}

#[test]
fn constr_compelem_nodeid_3() {
    //  Copied comment node has new node identity
    // XPath: for $x in <!--comment-->, $y in element elem {$x} return exa
    // Expected: false
    // Calls stub function - will fail until prod:CompElemConstructor is implemented
    let _ = stub_prodcompelemconstructor();
}

#[test]
fn constr_compelem_nodeid_4() {
    //  Copied pi node has new node identity
    // XPath: for $x in <?pi content?>, $y in element elem {$x} return exa
    // Expected: false
    // Calls stub function - will fail until prod:CompElemConstructor is implemented
    let _ = stub_prodcompelemconstructor();
}

#[test]
fn constr_compelem_nodeid_5() {
    //  Copied text node has new node identity
    // XPath: for $x in <a>text</a>, $y in element elem {$x/text()} return
    // Expected: false
    // Calls stub function - will fail until prod:CompElemConstructor is implemented
    let _ = stub_prodcompelemconstructor();
}

#[test]
fn constr_compelem_baseuri_1() {
    //  base-uri through xml:base attribute
    // XPath: fn:base-uri(element elem {attribute xml:base {\"http://www.e
    // Expected: http://www.example.com
    // Calls stub function - will fail until prod:CompElemConstructor is implemented
    let _ = stub_prodcompelemconstructor();
}

#[test]
fn constr_compelem_baseuri_2() {
    //  base-uri through parent
    // XPath: fn:base-uri(exactly-one((<elem xml:base=\"http://www.example
    // Expected: http://www.example.com
    // Calls stub function - will fail until prod:CompElemConstructor is implemented
    let _ = stub_prodcompelemconstructor();
}

#[test]
fn constr_compelem_baseuri_3() {
    //  base-uri through declaration
    // XPath: declare base-uri \"http://www.example.com\"; fn:base-uri(ele
    // Expected: http://www.example.com
    // Calls stub function - will fail until prod:CompElemConstructor is implemented
    let _ = stub_prodcompelemconstructor();
}

#[test]
fn constr_compelem_parent_1() {
    //  empty parent
    // XPath: count((element elem {})/..)
    // Expected: 0
    // Calls stub function - will fail until prod:CompElemConstructor is implemented
    let _ = stub_prodcompelemconstructor();
}

#[test]
fn constr_compelem_string_1() {
    //  string value of element
    // XPath: fn:string(element elem {'a', element a {}, 'b'})
    // Expected: ab
    // Calls stub function - will fail until prod:CompElemConstructor is implemented
    let _ = stub_prodcompelemconstructor();
}

#[test]
fn constr_compelem_data_1() {
    //  typed value of element
    // XPath: fn:data(element elem {'a', element a {}, 'b'})
    // Expected: ab
    // Calls stub function - will fail until prod:CompElemConstructor is implemented
    let _ = stub_prodcompelemconstructor();
}

#[test]
fn k2_computeconelem_6() {
    //  Test the type of the typed value of a computationally constructed element node.
    // XPath: data(element foo {\"dsa\"}) instance of xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CompElemConstructor is implemented
    let _ = stub_prodcompelemconstructor();
}

#[test]
fn k2_computeconelem_7() {
    //  Test the data of the typed value of a computationally constructed element node.
    // XPath: data(element foo {\"dsa\"})
    // Expected: dsa
    // Calls stub function - will fail until prod:CompElemConstructor is implemented
    let _ = stub_prodcompelemconstructor();
}

#[test]
fn k2_computeconelem_8() {
    //  Check that a directly constructed element gets the right type.
    // XPath: element e {\"content\"} instance of element(*, xs:anyType)
    // Expected: true
    // Calls stub function - will fail until prod:CompElemConstructor is implemented
    let _ = stub_prodcompelemconstructor();
}

#[test]
fn k2_computeconelem_10() {
    //  Check that a directly constructed element gets the right type(#3).
    // XPath: declare construction strip; element e {\"content\"} instance
    // Expected: true
    // Calls stub function - will fail until prod:CompElemConstructor is implemented
    let _ = stub_prodcompelemconstructor();
}

#[test]
fn k2_computeconelem_11() {
    //  Check that a directly constructed element gets the right type, and that the nam
    // XPath: element e {\"content\"} instance of element(a, xs:anyType)
    // Expected: false
    // Calls stub function - will fail until prod:CompElemConstructor is implemented
    let _ = stub_prodcompelemconstructor();
}

#[test]
fn k2_computeconelem_12() {
    //  Check that a directly constructed element gets the right type, and that the nam
    // XPath: declare construction strip; element e {\"content\"} instance
    // Expected: false
    // Calls stub function - will fail until prod:CompElemConstructor is implemented
    let _ = stub_prodcompelemconstructor();
}
