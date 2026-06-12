//! Test chunk 0 for fn:namespace-uri-for-prefix
//! Contains 25 tests

use dep::xpath::stub_fnnamespace_uri_for_prefix;

#[test]
fn fn_namespace_uri_for_prefix_1() {
    //  Evaluation of fn:namespace-uri-for-prefix function, for which the element is de
    // XPath: let $var := <anElement xmlns:p1 = \"http://www.example.com/e
    // Expected: http://www.example.com/examples
    // Calls stub function - will fail until fn:namespace-uri-for-prefix is implemented
    let _ = stub_fnnamespace_uri_for_prefix();
}

#[test]
fn fn_namespace_uri_for_prefix_2() {
    //  Evaluation of fn:namespace-uri-for-prefix function, for which the element is
    // XPath: let $var := <anElement xmlns:p1 = \"http://www.example.com/e
    // Expected: 0
    // Calls stub function - will fail until fn:namespace-uri-for-prefix is implemented
    let _ = stub_fnnamespace_uri_for_prefix();
}

#[test]
fn fn_namespace_uri_for_prefix_3() {
    //  Evaluation of fn:namespace-uri-for-prefix function,with the first argument (pre
    // XPath: declare default element namespace \"http://www.example.com/d
    // Expected: http://www.example.com/defaultspace
    // Calls stub function - will fail until fn:namespace-uri-for-prefix is implemented
    let _ = stub_fnnamespace_uri_for_prefix();
}

#[test]
fn fn_namespace_uri_for_prefix_4() {
    //  Evaluation of fn:namespace-uri-for-prefix function,with the first argument (pre
    // XPath: declare default element namespace \"http://www.example.com/d
    // Expected: http://www.example.com/defaultspace
    // Calls stub function - will fail until fn:namespace-uri-for-prefix is implemented
    let _ = stub_fnnamespace_uri_for_prefix();
}

#[test]
fn fn_namespace_uri_for_prefix_5() {
    //  Evaluation of fn:namespace-uri-for-prefix function,with the first argument (pre
    // XPath: fn:string(fn:namespace-uri-for-prefix(\"p1\",<anElement xmln
    // Expected: http://www.example.com/examples
    // Calls stub function - will fail until fn:namespace-uri-for-prefix is implemented
    let _ = stub_fnnamespace_uri_for_prefix();
}

#[test]
fn fn_namespace_uri_for_prefix_6() {
    //  Evaluation of fn:namespace-uri-for-prefix function,with the first argument (pre
    // XPath: fn:count(fn:namespace-uri-for-prefix(\"p2\",<anElement xmlns
    // Expected: 0
    // Calls stub function - will fail until fn:namespace-uri-for-prefix is implemented
    let _ = stub_fnnamespace_uri_for_prefix();
}

#[test]
fn fn_namespace_uri_for_prefix_7() {
    //  Evaluation of fn:namespace-uri-for-prefix function,with the first argument (pre
    // XPath: fn:string(fn:namespace-uri-for-prefix(xs:string(\"p1\"),<anE
    // Expected: http://www.example.com/examples
    // Calls stub function - will fail until fn:namespace-uri-for-prefix is implemented
    let _ = stub_fnnamespace_uri_for_prefix();
}

#[test]
fn fn_namespace_uri_for_prefix_8() {
    //  Evaluation of fn:namespace-uri-for-prefix function,with the first argument (pre
    // XPath: fn:string(fn:namespace-uri-for-prefix(fn:string(\"p1\"),<anE
    // Expected: http://www.example.com/examples
    // Calls stub function - will fail until fn:namespace-uri-for-prefix is implemented
    let _ = stub_fnnamespace_uri_for_prefix();
}

#[test]
fn fn_namespace_uri_for_prefix_9() {
    //  Evaluation of fn:namespace-uri-for-prefix function,with the first argument (pre
    // XPath: fn:string(fn:namespace-uri-for-prefix(fn:upper-case(\"p1\"),
    // Expected: http://www.example.com/examples
    // Calls stub function - will fail until fn:namespace-uri-for-prefix is implemented
    let _ = stub_fnnamespace_uri_for_prefix();
}

#[test]
fn fn_namespace_uri_for_prefix_10() {
    //  Evaluation of fn:namespace-uri-for-prefix function,with the first argument (pre
    // XPath: fn:string(fn:namespace-uri-for-prefix(fn:lower-case(\"P1\"),
    // Expected: http://www.example.com/examples
    // Calls stub function - will fail until fn:namespace-uri-for-prefix is implemented
    let _ = stub_fnnamespace_uri_for_prefix();
}

#[test]
fn fn_namespace_uri_for_prefix_11() {
    //  Evaluation of fn:namespace-uri-for-prefix function,with the first argument (pre
    // XPath: fn:string(fn:namespace-uri-for-prefix(fn:substring(\"abcp1\"
    // Expected: http://www.example.com/examples
    // Calls stub function - will fail until fn:namespace-uri-for-prefix is implemented
    let _ = stub_fnnamespace_uri_for_prefix();
}

#[test]
fn fn_namespace_uri_for_prefix_12() {
    //  Evaluation of fn:namespace-uri-for-prefix function,with the first argument (pre
    // XPath: fn:string(fn:namespace-uri-for-prefix(fn:concat(\"p\",\"1\")
    // Expected: http://www.example.com/examples
    // Calls stub function - will fail until fn:namespace-uri-for-prefix is implemented
    let _ = stub_fnnamespace_uri_for_prefix();
}

#[test]
fn fn_namespace_uri_for_prefix_13() {
    //  Evaluation of fn:namespace-uri-for-prefix function,with the first argument (pre
    // XPath: fn:string(fn:namespace-uri-for-prefix(fn:string-join((\"p\",
    // Expected: http://www.example.com/examples
    // Calls stub function - will fail until fn:namespace-uri-for-prefix is implemented
    let _ = stub_fnnamespace_uri_for_prefix();
}

#[test]
fn fn_namespace_uri_for_prefix_14() {
    //  Evaluation of fn:namespace-uri-for-prefix function,with the first argument (pre
    // XPath: fn:string(fn:namespace-uri-for-prefix(fn:substring-before(\"
    // Expected: http://www.example.com/examples
    // Calls stub function - will fail until fn:namespace-uri-for-prefix is implemented
    let _ = stub_fnnamespace_uri_for_prefix();
}

#[test]
fn fn_namespace_uri_for_prefix_15() {
    //  Evaluation of fn:namespace-uri-for-prefix function,with the first argument (pre
    // XPath: fn:string(fn:namespace-uri-for-prefix(fn:substring-after(\"a
    // Expected: http://www.example.com/examples
    // Calls stub function - will fail until fn:namespace-uri-for-prefix is implemented
    let _ = stub_fnnamespace_uri_for_prefix();
}

#[test]
fn fn_namespace_uri_for_prefix_16() {
    //  Evaluation of fn:namespace-uri-for-prefix function for which the element
    // XPath: declare default element namespace \"http://www.example.com/d
    // Expected: 0
    // Calls stub function - will fail until fn:namespace-uri-for-prefix is implemented
    let _ = stub_fnnamespace_uri_for_prefix();
}

#[test]
fn fn_namespace_uri_for_prefix_17() {
    //  Evaluation of fn:namespace-uri-for-prefix function for which the element is com
    // XPath: declare default element namespace \"http://www.example.com/d
    // Expected: 0
    // Calls stub function - will fail until fn:namespace-uri-for-prefix is implemented
    let _ = stub_fnnamespace_uri_for_prefix();
}

#[test]
fn fn_namespace_uri_for_prefix_18() {
    //  Evaluation of fn:namespace-uri-for-prefix function for which the element is com
    // XPath: declare default element namespace \"http://www.example.com/d
    // Expected: 0
    // Calls stub function - will fail until fn:namespace-uri-for-prefix is implemented
    let _ = stub_fnnamespace_uri_for_prefix();
}

#[test]
fn fn_namespace_uri_for_prefix_19() {
    //  Evaluation of fn:namespace-uri-for-prefix function for which the element is com
    // XPath: declare default element namespace \"http://www.example.com/d
    // Expected: 0
    // Calls stub function - will fail until fn:namespace-uri-for-prefix is implemented
    let _ = stub_fnnamespace_uri_for_prefix();
}

#[test]
fn fn_namespace_uri_for_prefix_20() {
    //  Evaluation of fn:namespace-uri-for-prefix function for which the element is com
    // XPath: declare default element namespace \"http://www.example.com/d
    // Expected: 0
    // Calls stub function - will fail until fn:namespace-uri-for-prefix is implemented
    let _ = stub_fnnamespace_uri_for_prefix();
}

#[test]
fn fn_namespace_uri_for_prefix_21() {
    //  Evaluation of fn:namespace-uri-for-prefix function for which the element is dir
    // XPath: fn:string(fn:namespace-uri-for-prefix(\"p1\",
    // Expected: http://www.example.com/examples
    // Calls stub function - will fail until fn:namespace-uri-for-prefix is implemented
    let _ = stub_fnnamespace_uri_for_prefix();
}

#[test]
fn k2_namespaceuriforprefixfunc_1() {
    //  Test that the correct default namespace is picked up.
    // XPath: declare namespace p = \"http://www.example.com/B\"; namespac
    // Expected: http://www.example.com/B
    // Calls stub function - will fail until fn:namespace-uri-for-prefix is implemented
    let _ = stub_fnnamespace_uri_for_prefix();
}

#[test]
fn k2_namespaceuriforprefixfunc_2() {
    //  Test that the correct default namespace is picked up(#2).
    // XPath: namespace-uri-for-prefix(\"\", exactly-one(
    // Expected:
    // Calls stub function - will fail until fn:namespace-uri-for-prefix is implemented
    let _ = stub_fnnamespace_uri_for_prefix();
}

#[test]
fn k2_namespaceuriforprefixfunc_3() {
    //  Test that the correct default namespace is picked up(#3).
    // XPath: declare namespace p = \"http://www.example.com/A\"; namespac
    // Expected: http://www.example.com/A
    // Calls stub function - will fail until fn:namespace-uri-for-prefix is implemented
    let _ = stub_fnnamespace_uri_for_prefix();
}

#[test]
fn cbcl_namespace_uri_for_prefix_001() {
    //  Test EBV of fn:namespace-uri-for-prefix
    // XPath: fn:boolean(fn:namespace-uri-for-prefix('', <a />))
    // Expected: false
    // Calls stub function - will fail until fn:namespace-uri-for-prefix is implemented
    let _ = stub_fnnamespace_uri_for_prefix();
}
