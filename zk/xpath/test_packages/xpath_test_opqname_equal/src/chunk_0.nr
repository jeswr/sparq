//! Test chunk 0 for op:QName-equal
//! Contains 32 tests

use dep::xpath::stub_opqname_equal;

#[test]
fn op_qname_equal_1() {
    //   Name: op-qName-equal-1   Description: Evaluation of op-QName-equal operator wi
    // XPath: fn:QName(\"http://www.example.com/example\", \"person\") eq
    // Expected: true
    // Calls stub function - will fail until op:QName-equal is implemented
    let _ = stub_opqname_equal();
}

#[test]
fn op_qname_equal_2() {
    //   Name: op-qName-equal-2   Description: Evaluation of op-QName-equal operator wi
    // XPath: fn:QName(\"http://www.example.com/example\", \"person\") ne
    // Expected: false
    // Calls stub function - will fail until op:QName-equal is implemented
    let _ = stub_opqname_equal();
}

#[test]
fn op_qname_equal_3() {
    //   Name: op-qName-equal-3   Description: Evaluation of op-QName-equal operator wi
    // XPath: fn:QName(\"http://www.example.com/example1\", \"person\") eq
    // Expected: false
    // Calls stub function - will fail until op:QName-equal is implemented
    let _ = stub_opqname_equal();
}

#[test]
fn op_qname_equal_4() {
    //   Name: op-qName-equal-4   Description: Evaluation of op-QName-equal operator wi
    // XPath: fn:QName(\"http://www.example.com/example1\", \"person\") ne
    // Expected: true
    // Calls stub function - will fail until op:QName-equal is implemented
    let _ = stub_opqname_equal();
}

#[test]
fn op_qname_equal_5() {
    //   Name: op-qName-equal-5   Description: Evaluation of op-QName-equal operator wi
    // XPath: fn:QName(\"http://www.example.com/example\", \"person1\") eq
    // Expected: false
    // Calls stub function - will fail until op:QName-equal is implemented
    let _ = stub_opqname_equal();
}

#[test]
fn op_qname_equal_6() {
    //   Name: op-qName-equal-6   Description: Evaluation of op-QName-equal operator wi
    // XPath: fn:QName(\"http://www.example.com/example\", \"person1\") ne
    // Expected: true
    // Calls stub function - will fail until op:QName-equal is implemented
    let _ = stub_opqname_equal();
}

#[test]
fn op_qname_equal_7() {
    //   Name: op-qName-equal-7   Description: Evaluation of op-QName-equal operator wi
    // XPath: fn:QName(\"http://www.example.com/example\", \"px1:person\")
    // Expected: true
    // Calls stub function - will fail until op:QName-equal is implemented
    let _ = stub_opqname_equal();
}

#[test]
fn op_qname_equal_8() {
    //   Name: op-qName-equal-8   Description: Evaluation of op-QName-equal operator wi
    // XPath: fn:QName(\"http://www.example.com/example\", \"px1:person\")
    // Expected: false
    // Calls stub function - will fail until op:QName-equal is implemented
    let _ = stub_opqname_equal();
}

#[test]
fn op_qname_equal_9() {
    //   Name: op-qName-equal-9   Description: Evaluation of op-QName-equal operator wi
    // XPath: fn:QName(\"http://www.example.com/example\", \"px1:person\")
    // Expected: true
    // Calls stub function - will fail until op:QName-equal is implemented
    let _ = stub_opqname_equal();
}

#[test]
fn op_qname_equal_10() {
    //   Name: op-qName-equal-10   Description: Evaluation of op-QName-equal operator w
    // XPath: fn:QName(\"http://www.example.com/example\", \"px1:person\")
    // Expected: false
    // Calls stub function - will fail until op:QName-equal is implemented
    let _ = stub_opqname_equal();
}

#[test]
fn op_qname_equal_11() {
    //   Name: op-qName-equal-11   Description: Evaluation of op-QName-equal operator w
    // XPath: fn:QName(\"http://www.example.com/example\", \"px1:person1\"
    // Expected: false
    // Calls stub function - will fail until op:QName-equal is implemented
    let _ = stub_opqname_equal();
}

#[test]
fn op_qname_equal_12() {
    //   Name: op-qName-equal-12   Description: Evaluation of op-QName-equal operator w
    // XPath: fn:QName(\"http://www.example.com/example\", \"px1:person1\"
    // Expected: true
    // Calls stub function - will fail until op:QName-equal is implemented
    let _ = stub_opqname_equal();
}

#[test]
fn op_qname_equal_13() {
    //   Name: op-qName-equal-13   Description: Evaluation of op-QName-equal operator w
    // XPath: fn:QName(\"http://www.example.com/example1\", \"px1:person1\
    // Expected: false
    // Calls stub function - will fail until op:QName-equal is implemented
    let _ = stub_opqname_equal();
}

#[test]
fn op_qname_equal_14() {
    //   Name: op-qName-equal-14   Description: Evaluation of op-QName-equal operator w
    // XPath: fn:QName(\"http://www.example.com/example1\", \"px1:person1\
    // Expected: true
    // Calls stub function - will fail until op:QName-equal is implemented
    let _ = stub_opqname_equal();
}

#[test]
fn op_qname_equal_15() {
    //   Name: op-qName-equal-15   Description: Evaluation of op-QName-equal operator w
    // XPath: fn:QName(\"\", \"person\") eq fn:QName(\"http://www.example.
    // Expected: false
    // Calls stub function - will fail until op:QName-equal is implemented
    let _ = stub_opqname_equal();
}

#[test]
fn op_qname_equal_16() {
    //   Name: op-qName-equal-16   Description: Evaluation of op-QName-equal operator w
    // XPath: fn:QName(\"\", \"person\") ne fn:QName(\"http://www.example.
    // Expected: true
    // Calls stub function - will fail until op:QName-equal is implemented
    let _ = stub_opqname_equal();
}

#[test]
fn op_qname_equal_17() {
    //   Name: op-qName-equal-17   Description: Evaluation of op-QName-equal operator w
    // XPath: fn:QName(\"http://www.example.com/example\", \"person\") eq
    // Expected: false
    // Calls stub function - will fail until op:QName-equal is implemented
    let _ = stub_opqname_equal();
}

#[test]
fn op_qname_equal_18() {
    //   Name: op-qName-equal-18   Description: Evaluation of op-QName-equal operator w
    // XPath: fn:QName(\"http://www.example.com/example\", \"person\") ne
    // Expected: true
    // Calls stub function - will fail until op:QName-equal is implemented
    let _ = stub_opqname_equal();
}

#[test]
fn op_qname_equal_19() {
    //   Name: op-qName-equal-19   Description: Evaluation of op-QName-equal operator w
    // XPath: fn:QName(\"\", \"person\") eq fn:QName(\"\",\"person\")
    // Expected: true
    // Calls stub function - will fail until op:QName-equal is implemented
    let _ = stub_opqname_equal();
}

#[test]
fn op_qname_equal_20() {
    //   Name: op-qName-equal-20   Description: Evaluation of op-QName-equal operator w
    // XPath: fn:QName(\"\", \"person\") ne fn:QName(\"\",\"person\")
    // Expected: false
    // Calls stub function - will fail until op:QName-equal is implemented
    let _ = stub_opqname_equal();
}

#[test]
fn op_qname_equal_21() {
    //   Name: op-qName-equal-21   Description: Evaluation of op-QName-equal operator a
    // XPath: fn:not(fn:QName(\"http://www.example.com/example\", \"px1:pe
    // Expected: true
    // Calls stub function - will fail until op:QName-equal is implemented
    let _ = stub_opqname_equal();
}

#[test]
fn op_qname_equal_22() {
    //   Name: op-qName-equal-22   Description: Evaluation of op-QName-equal operator a
    // XPath: fn:not(fn:QName(\"http://www.example.com/example\", \"px1:pe
    // Expected: false
    // Calls stub function - will fail until op:QName-equal is implemented
    let _ = stub_opqname_equal();
}

#[test]
fn op_qname_equal_23() {
    //   Name: op-qName-equal-23   Description: Evaluation of op-QName-equal operator a
    // XPath: (fn:QName(\"http://www.example.com/example\", \"px:person\")
    // Expected: true
    // Calls stub function - will fail until op:QName-equal is implemented
    let _ = stub_opqname_equal();
}

#[test]
fn op_qname_equal_24() {
    //   Name: op-qName-equal-24   Description: Evaluation of op-QName-equal operator a
    // XPath: (fn:QName(\"http://www.example.com/example\", \"px:person\")
    // Expected: false
    // Calls stub function - will fail until op:QName-equal is implemented
    let _ = stub_opqname_equal();
}

#[test]
fn value_comparison_1() {
    //   name : value-comparison-1   description : Evaluation of xs:string compare to x
    // XPath: xs:string(\"example.org/\") eq xs:anyURI(\"example.org/\")
    // Expected: true
    // Calls stub function - will fail until op:QName-equal is implemented
    let _ = stub_opqname_equal();
}

#[test]
fn value_comparison_2() {
    //   name : value-comparison-2   description : Evaluation of xs:string compare to x
    // XPath: xs:string(\"example.org/\") ne xs:anyURI(\"example.org/\")
    // Expected: false
    // Calls stub function - will fail until op:QName-equal is implemented
    let _ = stub_opqname_equal();
}

#[test]
fn k_qnameeq_1() {
    //  *******************************************************  Test: K-QNameEQ-1
    // XPath: QName(\"example.com/\", \"p:ncname\") eq QName(\"example.com
    // Expected: true
    // Calls stub function - will fail until op:QName-equal is implemented
    let _ = stub_opqname_equal();
}

#[test]
fn k_qnameeq_2() {
    //  *******************************************************  Test: K-QNameEQ-2
    // XPath: QName(\"example.com/\", \"p:ncname\") eq QName(\"example.com
    // Expected: true
    // Calls stub function - will fail until op:QName-equal is implemented
    let _ = stub_opqname_equal();
}

#[test]
fn k_qnameeq_3() {
    //  *******************************************************  Test: K-QNameEQ-3
    // XPath: QName(\"example.com/\", \"p:ncname\") ne QName(\"example.com
    // Expected: true
    // Calls stub function - will fail until op:QName-equal is implemented
    let _ = stub_opqname_equal();
}

#[test]
fn k_qnameeq_4() {
    //  *******************************************************  Test: K-QNameEQ-4
    // XPath: QName(\"example.com/\", \"p:ncname\") ne QName(\"example.com
    // Expected: true
    // Calls stub function - will fail until op:QName-equal is implemented
    let _ = stub_opqname_equal();
}

#[test]
fn cbcl_qname_equal_001() {
    //  test equality of xs:QName
    // XPath: declare function local:QName-value($arg as xs:boolean) as xs
    // Expected: false
    // Calls stub function - will fail until op:QName-equal is implemented
    let _ = stub_opqname_equal();
}

#[test]
fn cbcl_qname_equal_002() {
    //  test equality of xs:QName
    // XPath: declare function local:QName-value($arg as xs:boolean) as xs
    // Expected: true
    // Calls stub function - will fail until op:QName-equal is implemented
    let _ = stub_opqname_equal();
}
