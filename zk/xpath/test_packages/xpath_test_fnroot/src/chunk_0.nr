//! Test chunk 0 for fn:root
//! Contains 12 tests

use dep::xpath::stub_fnroot;

#[test]
fn fn_root_16() {
    //  Evaluation of the fn:root function with argument set to computed text node by s
    // XPath: fn:root(text {\"A text Node\"})
    // Expected: A text Node
    // Calls stub function - will fail until fn:root is implemented
    let _ = stub_fnroot();
}

#[test]
fn fn_root_17() {
    //  Evaluation of the fn:root function with argument set to computed text node by s
    // XPath: let $var := text {\"a text Node\"} return fn:root($var)
    // Expected: a text Node
    // Calls stub function - will fail until fn:root is implemented
    let _ = stub_fnroot();
}

#[test]
fn fn_root_18() {
    //  Evaluation of the fn:root function used in conjunction with 'is' operator (retu
    // XPath: let $var := element anElement {\"Element Content\"} return f
    // Expected: true
    // Calls stub function - will fail until fn:root is implemented
    let _ = stub_fnroot();
}

#[test]
fn fn_root_19() {
    //  Evaluation of the fn:root function used in conjunction with 'is' operator (retu
    // XPath: let $var := element anElement {\"Element Content\"} return f
    // Expected: true
    // Calls stub function - will fail until fn:root is implemented
    let _ = stub_fnroot();
}

#[test]
fn fn_root_20() {
    //  Evaluation of the fn:root function used as argument to namespace-uri function (
    // XPath: let $var := element anElement {\"Element Content\"} return f
    // Expected: 1
    // Calls stub function - will fail until fn:root is implemented
    let _ = stub_fnroot();
}

#[test]
fn fn_root_21() {
    //  Evaluation of the fn:root function used as argument to namespace-uri function (
    // XPath: let $var := <!-- An Element Node --> return fn:count(fn:name
    // Expected: 1
    // Calls stub function - will fail until fn:root is implemented
    let _ = stub_fnroot();
}

#[test]
fn fn_root_22() {
    //  Evaluation of the fn:root function used as argument to namespace-uri function (
    // XPath: let $var := <?format role=\"output\" ?> return fn:count(fn:n
    // Expected: 1
    // Calls stub function - will fail until fn:root is implemented
    let _ = stub_fnroot();
}

#[test]
fn fn_root_23() {
    //  Evaluation of the fn:root function used as argument to namespace-uri function (
    // XPath: let $var := text {\"A text node\"} return fn:count(fn:namesp
    // Expected: 1
    // Calls stub function - will fail until fn:root is implemented
    let _ = stub_fnroot();
}

#[test]
fn k2_noderootfunc_2() {
    //  Apply fn:root() to an empty sequence coming from a parent axis.
    // XPath: empty(fn:root(<e/>/..))
    // Expected: true
    // Calls stub function - will fail until fn:root is implemented
    let _ = stub_fnroot();
}

#[test]
fn k2_noderootfunc_6() {
    //  Invoke on a processing instruction node.
    // XPath: fn:root(text{\"text node\"})
    // Expected: text node
    // Calls stub function - will fail until fn:root is implemented
    let _ = stub_fnroot();
}

#[test]
fn k2_noderootfunc_7() {
    //  Invoke on a processing instruction node.
    // XPath: fn:root(text{\"text node\"})
    // Expected: text node
    // Calls stub function - will fail until fn:root is implemented
    let _ = stub_fnroot();
}

#[test]
fn k2_noderootfunc_8() {
    //  Invoke on a single document node.
    // XPath: root(document {()}) instance of document-node()
    // Expected: true
    // Calls stub function - will fail until fn:root is implemented
    let _ = stub_fnroot();
}
