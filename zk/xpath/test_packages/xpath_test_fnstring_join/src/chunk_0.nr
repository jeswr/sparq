//! Test chunk 0 for fn:string-join
//! Contains 41 tests

use dep::xpath::stub_fnstring_join;

#[test]
fn fn_string_join2args_1() {
    //  Evaluates The 'string-join' function with the arguments set as follows: $arg1 =
    // XPath: fn:string-join(xs:string(\"This is a characte\"),xs:string(\
    // Expected: This is a characte
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn fn_string_join2args_2() {
    //  Evaluates The 'string-join' function with the arguments set as follows: $arg1 =
    // XPath: fn:string-join(xs:string(\"This is a characte\"),xs:string(\
    // Expected: This is a characte
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn fn_string_join2args_3() {
    //  Evaluates The 'string-join' function with the arguments set as follows: $arg1 =
    // XPath: fn:string-join(xs:string(\"This is a characte\"),xs:string(\
    // Expected: This is a characte
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn fn_string_join2args_4() {
    //  Evaluates The 'string-join' function with the arguments set as follows: $arg1 =
    // XPath: fn:string-join(xs:string(\"This is a characte\"),xs:string(\
    // Expected: This is a characte
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn fn_string_join2args_5() {
    //  Evaluates The 'string-join' function with the arguments set as follows: $arg1 =
    // XPath: fn:string-join(xs:string(\"This is a characte\"),xs:string(\
    // Expected: This is a characte
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn fn_string_join_1() {
    //  Evaluates The 'string-join' function as per example 1 for this frunction in F&O
    // XPath: fn:string-join(('Now', 'is', 'the', 'time', '...'), ' ')
    // Expected: Now is the time ...
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn fn_string_join_2() {
    //  Evaluates The 'string-join' function as per example 2 for this function in the
    // XPath: fn:string-join(('Blow, ', 'blow, ', 'thou ', 'winter ', 'win
    // Expected: Blow, blow, thou winter wind!
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn fn_string_join_3() {
    //  Evaluates The 'string-join' function as per example 3 for this function in the
    // XPath: fn:count(fn:string-join((), 'separator'))
    // Expected: 1
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn fn_string_join_4() {
    //  Evaluates The 'string-join' function with the arguments set as follows: $arg1 =
    // XPath: fn:count(fn:string-join((),\"\"))
    // Expected: 1
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn fn_string_join_5() {
    //  Evaluates The 'string-join' function with the arguments set as follows: $arg1 =
    // XPath: fn:count(fn:string-join(\"\",\"\"))
    // Expected: 1
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn fn_string_join_6() {
    //  Evaluates The 'string-join' function with the arguments set as follows: $arg1 =
    // XPath: fn:count(fn:string-join(\"\",\"A Character String\"))
    // Expected: 1
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn fn_string_join_7() {
    //  Evaluates The 'string-join' function with the arguments set as follows: $arg1 =
    // XPath: fn:count(fn:string-join((),\"A Character String\"))
    // Expected: 1
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn fn_string_join_8() {
    //  Evaluates The 'string-join' function with the arguments set as follows: $arg1 =
    // XPath: fn:string-join((\"AAAAABBBBBCCCCC\"),\"BBBBB\")
    // Expected: AAAAABBBBBCCCCC
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn fn_string_join_9() {
    //  Evaluates The 'string-join' function with the arguments set as follows: $arg1 =
    // XPath: fn:string-join((\"AAAAABBBBB\"),\" \")
    // Expected: AAAAABBBBB
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn fn_string_join_10() {
    //  Evaluates The 'string-join' function with the arguments set as follows: $arg1 =
    // XPath: fn:count(fn:string-join((\" \"),\"AAAAABBBBB\"))
    // Expected: 1
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn fn_string_join_11() {
    //  Evaluates The 'string-join' function using it as a argument of a fn:not - retur
    // XPath: fn:not(fn:string-join((),\"A\"))
    // Expected: true
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn fn_string_join_12() {
    //  Evaluates The 'string-join' function using it as a argument of a fn:not - retur
    // XPath: fn:not(fn:string-join((\"A\"),\"B\"))
    // Expected: false
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn fn_string_join_13() {
    //  Evaluates The 'string-join' function with the arguments set as follows: $arg1 =
    // XPath: fn:string-join((xs:string(\"A\")),\"A\")
    // Expected: A
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn fn_string_join_14() {
    //  Evaluates The 'string-join' function with the arguments set as follows: $arg1 =
    // XPath: fn:string-join((\"A\"),xs:string(\"A\"))
    // Expected: A
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn fn_string_join_15() {
    //  Evaluates The 'string-join' function with the arguments set as follows: $arg1 =
    // XPath: fn:string-join((\"A\"),\"a\")
    // Expected: A
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn fn_string_join_16() {
    //  Evaluates The 'string-join' function with the arguments set as follows: $arg1 =
    // XPath: fn:string-join((\"a\"),\"A\")
    // Expected: a
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn fn_string_join_17() {
    //  Evaluates The 'string-join' function with the arguments set as follows: $arg1 =
    // XPath: fn:string-join(\"string-join\",\"string-join\")
    // Expected: string-join
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn fn_string_join_18() {
    //  Evaluates The 'string-join' function with the arguments set as follows: $arg1 =
    // XPath: fn:string-join((\"string-joinstring-join\"),\"string-join\")
    // Expected: string-joinstring-join
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn fn_string_join_19() {
    //  Evaluates The 'string-join' function with the arguments set as follows: $arg1 =
    // XPath: fn:string-join(\"****\",\"***\")
    // Expected: ****
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn fn_string_join_20() {
    //  Evaluates The 'string-join' function with the arguments set as follows: $arg1 =
    // XPath: fn:string-join(\"12345\",\"1234\")
    // Expected: 12345
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn fn_string_join_21() {
    //  Evaluates The 'string-join' function with the arguments set as follows: $arg1 =
    // XPath: fn:string-join(\"string-join\",\"nioj-gnirts\")
    // Expected: string-join
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn fn_string_join_22() {
    //  Default second argument is zero-length string in 3.0
    // XPath: fn:string-join((\"1\", \"2\", \"3\"))
    // Expected: 123
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn fn_string_join_23() {
    //  Default second argument is zero-length string in 3.0
    // XPath: let $e := <e><a>1</a><b>2</b><c>3</c></e>          return fn
    // Expected: 123
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn fn_string_join_24() {
    //  Default second argument is zero-length string in 3.0
    // XPath: let $e := <e><a>1</a><b></b><c>3</c></e>          return fn:
    // Expected: 13
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn fn_string_join_25() {
    //  Default second argument is zero-length string in 3.0
    // XPath: let $e := <e><a>1</a><b></b><c>3</c></e>          return fn:
    // Expected:
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn fn_string_join_26() {
    //  Default second argument is zero-length string in 3.0
    // XPath: fn:string-join((1 to 9)!string())
    // Expected: 123456789
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn fn_string_join_28() {
    //  First arg is xs:anyAtomicType* in 3.1 (bug 29184)
    // XPath: fn:string-join(1 to 5, \"\")
    // Expected: 12345
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn fn_string_join_30() {
    //  First arg is xs:anyAtomicType* in 3.1 (bug 29184)
    // XPath: fn:string-join((xs:string#1, xs:gYear#1, xs:float#1, xs:doub
    // Expected: 1900,1900,1900,1900,1900
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn fn_string_join_31() {
    //  First arg is xs:anyAtomicType* in 3.1 (bug 29184)
    // XPath: fn:string-join(string-to-codepoints('abc'), '-')
    // Expected: 97-98-99
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn fn_string_join_32() {
    //  First arg is xs:anyAtomicType* in 3.1 (bug 29184)
    // XPath: let $x := <a xmlns=\"http://sj31/\"><b/><c/><d/></a> return
    // Expected: b-c-d
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn k_stringjoinfunc_1a() {
    //  A test whose essence is: `string-join('a string')`. Allowed in 3.0
    // XPath: string-join(\"a string\")
    // Expected: a string
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn k_stringjoinfunc_3() {
    //  A test whose essence is: `string-join(('Now', 'is', 'the', 'time', '...'), ' ')
    // XPath: string-join(('Now', 'is', 'the', 'time', '...'), ' ') eq \"N
    // Expected: true
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn k_stringjoinfunc_4() {
    //  A test whose essence is: `string-join(('abc', 'def'), '') eq 'abcdef'`.
    // XPath: string-join((\"abc\", \"def\"), \"\") eq \"abcdef\"
    // Expected: true
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn k_stringjoinfunc_5() {
    //  A test whose essence is: `string-join(('Blow, ', 'blow, ', 'thou ', 'winter ',
    // XPath: string-join(('Blow, ', 'blow, ', 'thou ', 'winter ', 'wind!'
    // Expected: true
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn k_stringjoinfunc_6() {
    //  A test whose essence is: `string-join((), 'separator') eq ''`.
    // XPath: string-join((), 'separator') eq \"\"
    // Expected: true
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}

#[test]
fn cbcl_fn_string_join_001() {
    //  Test boolean fn:string-join on various cases
    // XPath: declare function local:repeat($count as xs:integer, $arg as
    // Expected: true
    // Calls stub function - will fail until fn:string-join is implemented
    let _ = stub_fnstring_join();
}
