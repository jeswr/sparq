//! Test chunk 0 for fn:prefix-from-QName
//! Contains 19 tests

use dep::xpath::stub_fnprefix_from_qname;

#[test]
fn fn_prefix_from_qname_3() {
    //  Evaluation of fn-prefix-from-QName function as argument to fn:string function a
    // XPath: fn:count(fn:prefix-from-QName(xs:QName(\"name\")))
    // Expected: 0
    // Calls stub function - will fail until fn:prefix-from-QName is implemented
    let _ = stub_fnprefix_from_qname();
}

#[test]
fn fn_prefix_from_qname_4() {
    //  Evaluation of fn-prefix-from-QName function as argument to xs:string function a
    // XPath: fn:string-length(xs:string(fn:prefix-from-QName(xs:QName(\"n
    // Expected: 0
    // Calls stub function - will fail until fn:prefix-from-QName is implemented
    let _ = stub_fnprefix_from_qname();
}

#[test]
fn fn_prefix_from_qname_5() {
    //  Evaluation of fn-prefix-from-QName function as argument to xs:string function a
    // XPath: xs:string(fn:prefix-from-QName(xs:QName(\"foo:name\")))
    // Expected: foo
    // Calls stub function - will fail until fn:prefix-from-QName is implemented
    let _ = stub_fnprefix_from_qname();
}

#[test]
fn fn_prefix_from_qname_6() {
    //  Evaluation of fn-prefix-from-QName function as argument to fn:string function a
    // XPath: fn:string(fn:prefix-from-QName(xs:QName(\"foo:name\")))
    // Expected: foo
    // Calls stub function - will fail until fn:prefix-from-QName is implemented
    let _ = stub_fnprefix_from_qname();
}

#[test]
fn fn_prefix_from_qname_7() {
    //  Evaluation of fn-prefix-from-QName function with argument set to empty sequence
    // XPath: fn:count(fn:prefix-from-QName(()))
    // Expected: 0
    // Calls stub function - will fail until fn:prefix-from-QName is implemented
    let _ = stub_fnprefix_from_qname();
}

#[test]
fn fn_prefix_from_qname_9() {
    //  Evaluation of fn-prefix-from-QName function as argument to fn:upper-case.
    // XPath: fn:upper-case(fn:prefix-from-QName(xs:QName(\"foo:bar\")))
    // Expected: FOO
    // Calls stub function - will fail until fn:prefix-from-QName is implemented
    let _ = stub_fnprefix_from_qname();
}

#[test]
fn fn_prefix_from_qname_10() {
    //  Evaluation of fn-prefix-from-QName function as argument to fn:lower-case.
    // XPath: fn:lower-case(fn:prefix-from-QName(xs:QName(\"FOO:bar\")))
    // Expected: foo
    // Calls stub function - will fail until fn:prefix-from-QName is implemented
    let _ = stub_fnprefix_from_qname();
}

#[test]
fn fn_prefix_from_qname_11() {
    //  Evaluation of fn-prefix-from-QName function as argument to fn:lower-case.
    // XPath: fn:concat(fn:prefix-from-QName(xs:QName(\"foo:bar\")),\":bar
    // Expected: foo:bar
    // Calls stub function - will fail until fn:prefix-from-QName is implemented
    let _ = stub_fnprefix_from_qname();
}

#[test]
fn fn_prefix_from_qname_12() {
    //  Evaluation of fn-prefix-from-QName function as argument to fn:substring-before.
    // XPath: fn:substring-before(fn:prefix-from-QName(xs:QName(\"foo:bar\
    // Expected: f
    // Calls stub function - will fail until fn:prefix-from-QName is implemented
    let _ = stub_fnprefix_from_qname();
}

#[test]
fn fn_prefix_from_qname_13() {
    //  Evaluation of fn-prefix-from-QName function as argument to fn:substring-after.
    // XPath: fn:substring-after(fn:prefix-from-QName(xs:QName(\"foo:bar\"
    // Expected: oo
    // Calls stub function - will fail until fn:prefix-from-QName is implemented
    let _ = stub_fnprefix_from_qname();
}

#[test]
fn fn_prefix_from_qname_14() {
    //  Evaluation of fn-prefix-from-QName function as argument to fn:string-length.
    // XPath: fn:string-length(fn:prefix-from-QName(xs:QName(\"foo:bar\"))
    // Expected: 3
    // Calls stub function - will fail until fn:prefix-from-QName is implemented
    let _ = stub_fnprefix_from_qname();
}

#[test]
fn fn_prefix_from_qname_15() {
    //  Evaluation of fn-prefix-from-QName function as argument to fn:substring.
    // XPath: fn:substring(fn:prefix-from-QName(xs:QName(\"foo:bar\")),2)
    // Expected: oo
    // Calls stub function - will fail until fn:prefix-from-QName is implemented
    let _ = stub_fnprefix_from_qname();
}

#[test]
fn fn_prefix_from_qname_16() {
    //  Evaluation of fn-prefix-from-QName function as argument to fn:string-join.
    // XPath: fn:string-join((fn:prefix-from-QName(xs:QName(\"foo:bar\")),
    // Expected: foo:bar
    // Calls stub function - will fail until fn:prefix-from-QName is implemented
    let _ = stub_fnprefix_from_qname();
}

#[test]
fn fn_prefix_from_qname_17() {
    //  Evaluation of fn-prefix-from-QName function as argument to fn:starts-with.
    // XPath: fn:starts-with(fn:prefix-from-QName(xs:QName(\"foo:bar\")),\
    // Expected: true
    // Calls stub function - will fail until fn:prefix-from-QName is implemented
    let _ = stub_fnprefix_from_qname();
}

#[test]
fn fn_prefix_from_qname_18() {
    //  Evaluation of fn-prefix-from-QName function as argument to fn:ends-with.
    // XPath: fn:ends-with(fn:prefix-from-QName(xs:QName(\"foo:bar\")),\"f
    // Expected: false
    // Calls stub function - will fail until fn:prefix-from-QName is implemented
    let _ = stub_fnprefix_from_qname();
}

#[test]
fn fn_prefix_from_qname_19() {
    //  Evaluation of fn-prefix-from-QName function as argument to fn:contains.
    // XPath: fn:contains(fn:prefix-from-QName(xs:QName(\"foo:bar\")),\"f\
    // Expected: true
    // Calls stub function - will fail until fn:prefix-from-QName is implemented
    let _ = stub_fnprefix_from_qname();
}

#[test]
fn fn_prefix_from_qname_20() {
    //  Evaluation of fn-prefix-from-QName function with two namespaces declared with s
    // XPath: fn:prefix-from-QName(xs:QName(\"foo:bar\"))
    // Expected: foo
    // Calls stub function - will fail until fn:prefix-from-QName is implemented
    let _ = stub_fnprefix_from_qname();
}

#[test]
fn k_prefixfromqname_3() {
    //  A test whose essence is: `empty(prefix-from-QName( () ))`.
    // XPath: empty(prefix-from-QName( () ))
    // Expected: true
    // Calls stub function - will fail until fn:prefix-from-QName is implemented
    let _ = stub_fnprefix_from_qname();
}

#[test]
fn k_prefixfromqname_4() {
    //  A test whose essence is: `prefix-from-QName( QName('example.com/', 'pre:lname')
    // XPath: prefix-from-QName( QName(\"example.com/\", \"pre:lname\")) e
    // Expected: true
    // Calls stub function - will fail until fn:prefix-from-QName is implemented
    let _ = stub_fnprefix_from_qname();
}
