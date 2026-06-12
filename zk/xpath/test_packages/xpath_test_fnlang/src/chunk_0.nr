//! Test chunk 0 for fn:lang
//! Contains 39 tests

use dep::xpath::stub_fnlang;

#[test]
fn fn_lang1args_1() {
    //  Evaluates The 'lang' function with the arguments set as follows: $testlang = 'e
    // XPath: fn:lang(xs:string(\"en\"),/root[1]/time[1])
    // Expected: false
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn fn_lang1args_2() {
    //  Evaluates The 'lang' function with the arguments set as follows: $testlang = 'E
    // XPath: fn:lang(xs:string(\"EN\"),./root[1]/time[1])
    // Expected: false
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn fn_lang1args_3() {
    //  Evaluates The 'lang' function with the arguments set as follows: $testlang = 'e
    // XPath: fn:lang(xs:string(\"eN\"),./root[1]/time[1])
    // Expected: false
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn fn_lang_2() {
    //  Evaluation of the fn:lang function with testlang set to empty sequence Uses fn:
    // XPath: fn:count(fn:lang((), ./langs[1]/para[1]))
    // Expected: 1
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn fn_lang_3() {
    //  Evaluation of the fn:lang function with testlang set to 'en' as per example 1 f
    // XPath: for $x in /langs/para[1] return $x/fn:lang(\"en\")
    // Expected: true
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn fn_lang_4() {
    //  Evaluation of the fn:lang function with testlang set to 'en' as per example 2 f
    // XPath: for $x in /langs/div[1]/para return $x/fn:lang(\"en\")
    // Expected: true
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn fn_lang_5() {
    //  Evaluation of the fn:lang function with testlang set to 'EN'. Context node is '
    // XPath: for $x in /langs/para[2] return $x/fn:lang(\"EN\")
    // Expected: true
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn fn_lang_6() {
    //  Evaluation of the fn:lang function with testlang set to 'En'. Context node is '
    // XPath: for $x in /langs/para[2] return $x/fn:lang(\"En\")
    // Expected: true
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn fn_lang_7() {
    //  Evaluation of the fn:lang function with testlang set to 'eN'. Context node is '
    // XPath: for $x in /langs/para[2] return $x/fn:lang(\"eN\")
    // Expected: true
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn fn_lang_8() {
    //  Evaluation of the fn:lang function with testlang set to 'en'. Context node is '
    // XPath: for $x in /langs/para[2] return $x/fn:lang(\"en\")
    // Expected: true
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn fn_lang_9() {
    //  Evaluation of the fn:lang function with testlang set to 'en'. Context node is '
    // XPath: for $x in /langs/para[3] return $x/fn:lang(\"en-us\")
    // Expected: true
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn fn_lang_10() {
    //  Evaluation of the fn:lang function with testlang set to 'EN'. Context node is '
    // XPath: for $x in /langs/para[3] return $x/fn:lang(\"EN\")
    // Expected: true
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn fn_lang_11() {
    //  Evaluation of the fn:lang function with testlang set to 'En'. Context node is '
    // XPath: for $x in /langs/para[3] return $x/fn:lang(\"En\")
    // Expected: true
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn fn_lang_12() {
    //  Evaluation of the fn:lang function with testlang set to 'eN'. Context node is '
    // XPath: for $x in /langs/para[3] return $x/fn:lang(\"eN\")
    // Expected: true
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn fn_lang_13() {
    //  Evaluation of the fn:lang function with testlang set to 'en'. Context node is '
    // XPath: for $x in /langs/para[3] return $x/fn:lang(\"en\")
    // Expected: true
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn fn_lang_14() {
    //  Evaluation of the fn:lang function with testlang set to 'en-us'. Context node i
    // XPath: for $x in /langs/para[3] return $x/fn:lang(\"en-us\")
    // Expected: true
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn fn_lang_16() {
    //  Evaluation of the fn:lang function with testlang set to 'us-en' and context nod
    // XPath: for $x in /langs/para[3] return $x/fn:lang(\"us-us\")
    // Expected: false
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn fn_lang_17() {
    //  Evaluation of the fn:lang function with testlang set to 'fr' and context node h
    // XPath: for $x in /langs/para[2] return $x/fn:lang(\"fr\")
    // Expected: false
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn fn_lang_18() {
    //  Evaluation of the fn:lang function with testlang set to 'en' and specified node
    // XPath: fn:lang(\"en\",fn:exactly-one(/langs/para[1]))
    // Expected: true
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn fn_lang_19() {
    //  Evaluation of the fn:lang function with testlang set to 'fr' and specified node
    // XPath: fn:lang(\"fr\",fn:exactly-one(/langs/para[1]))
    // Expected: false
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn fn_lang_20() {
    //  Evaluation of the fn:lang function with testlang set to 'en' and specified node
    // XPath: fn:lang(\"en\",fn:exactly-one(/langs/para[3]))
    // Expected: true
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn fn_lang_21() {
    //  Evaluation of the fn:lang function with testlang set to 'en' and specified node
    // XPath: fn:lang(\"en\",fn:exactly-one(/langs/para[2]))
    // Expected: true
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn fn_lang_23() {
    //  Evaluation of the fn:lang function using one part of a three-part tag
    // XPath: fn:lang(\"de\",fn:exactly-one(/langs/para[4]))
    // Expected: true
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn fn_lang_24() {
    //  Evaluation of the fn:lang function using two parts of a three-part tag
    // XPath: fn:lang(\"de-DE\",fn:exactly-one(/langs/para[4]))
    // Expected: true
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn fn_lang_25() {
    //  Evaluation of the fn:lang function using two parts of a three-part tag ignoring
    // XPath: fn:lang(\"DE-de\",fn:exactly-one(/langs/para[4]))
    // Expected: true
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn fn_lang_26() {
    //  Evaluation of the fn:lang function using three parts of a three-part tag
    // XPath: fn:lang(\"de-DE-1996\",fn:exactly-one(/langs/para[4]))
    // Expected: true
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn fn_lang_27() {
    //  Evaluation of the fn:lang function using three parts of a three-part tag, no ma
    // XPath: fn:lang(\"de-DE-1998\",fn:exactly-one(/langs/para[4]))
    // Expected: false
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn fn_lang_28() {
    //  Evaluation of the fn:lang function using leading substring of a three-part tag,
    // XPath: fn:lang(\"de-D\",fn:exactly-one(/langs/para[4]))
    // Expected: false
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn fn_lang_29() {
    //  Evaluation of the fn:lang function using leading substring of a three-part tag,
    // XPath: fn:lang(\"de-DE-\",fn:exactly-one(/langs/para[4]))
    // Expected: false
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn fn_lang_30() {
    //  Evaluation of the fn:lang function using four parts of a three-part tag, no mat
    // XPath: fn:lang(\"de-DE-1996-x\",fn:exactly-one(/langs/para[4]))
    // Expected: false
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn fn_lang_31() {
    // Dynamic call on fn:lang#1 using retained context item
    // XPath: let $l := /langs/para[4]!fn:lang#1 return /langs/para[1]!$l(
    // Expected: false
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn fn_lang_32() {
    // Dynamic call on fn:lang#1 using retained context item
    // XPath: let $l := /langs/para[4]!fn:lang#1 return /langs/para[1]!$l(
    // Expected: true
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn k2_nodelangfunc_1() {
    //  Invoke fn:lang from the top and bottom of a tree.
    // XPath: let $i := <e xml:lang=\"en\"> <b xml:lang=\"de\"/> </e> retu
    // Expected: true false
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn k2_nodelangfunc_2() {
    //  Start navigating from the actual attribute node.
    // XPath: let $i := <e xml:lang=\"en\"> <b xml:lang=\"de\"/> </e> retu
    // Expected: true
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn k2_nodelangfunc_3() {
    //  Navigate a single, empty attribute.
    // XPath: lang(\"de\", attribute xml:lang {()})
    // Expected: false
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn k2_nodelangfunc_4() {
    //  Sub-languages must be separated with dashes.
    // XPath: lang(\"en\", <e xml:lang=\"ene\"/>)
    // Expected: false
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn k2_nodelangfunc_5() {
    //  Use a single attribute node as input.
    // XPath: fn:lang(\"fr\", attribute xml:lang {\"fr\"})
    // Expected: false
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn k2_nodelangfunc_6() {
    //  Use a single attribute node as input.
    // XPath: fn:lang(\"fr\", attribute xml:lang {()})
    // Expected: false
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}

#[test]
fn cbcl_fn_lang_001() {
    //  Try to force optional item call on fn:lang
    // XPath: if(current-date() > xs:date(\"2000-01-01\")) then lang(\"en\
    // Expected: false
    // Calls stub function - will fail until fn:lang is implemented
    let _ = stub_fnlang();
}
