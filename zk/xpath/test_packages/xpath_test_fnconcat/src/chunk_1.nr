//! Test chunk 1 for fn:concat
//! Contains 40 tests

use dep::xpath::stub_fnconcat;

#[test]
fn fn_concatnpi2args_1() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:n
    // XPath: fn:concat(xs:nonPositiveInteger(\"-999999999999999999\"),xs:
    // Expected: -999999999999999999-999999999999999999
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatnpi2args_2() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:n
    // XPath: fn:concat(xs:nonPositiveInteger(\"-475688437271870490\"),xs:
    // Expected: -475688437271870490-999999999999999999
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatnpi2args_3() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:n
    // XPath: fn:concat(xs:nonPositiveInteger(\"0\"),xs:nonPositiveInteger
    // Expected: 0-999999999999999999
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatnpi2args_4() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:n
    // XPath: fn:concat(xs:nonPositiveInteger(\"-999999999999999999\"),xs:
    // Expected: -999999999999999999-475688437271870490
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatnpi2args_5() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:n
    // XPath: fn:concat(xs:nonPositiveInteger(\"-999999999999999999\"),xs:
    // Expected: -9999999999999999990
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatnni2args_1() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:n
    // XPath: fn:concat(xs:nonNegativeInteger(\"0\"),xs:nonNegativeInteger
    // Expected: 00
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatnni2args_2() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:n
    // XPath: fn:concat(xs:nonNegativeInteger(\"303884545991464527\"),xs:n
    // Expected: 3038845459914645270
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatnni2args_3() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:n
    // XPath: fn:concat(xs:nonNegativeInteger(\"999999999999999999\"),xs:n
    // Expected: 9999999999999999990
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatnni2args_4() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:n
    // XPath: fn:concat(xs:nonNegativeInteger(\"0\"),xs:nonNegativeInteger
    // Expected: 0303884545991464527
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatnni2args_5() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:n
    // XPath: fn:concat(xs:nonNegativeInteger(\"0\"),xs:nonNegativeInteger
    // Expected: 0999999999999999999
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatsht2args_1() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:s
    // XPath: fn:concat(xs:short(\"-32768\"),xs:short(\"-32768\"))
    // Expected: -32768-32768
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatsht2args_2() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:s
    // XPath: fn:concat(xs:short(\"-5324\"),xs:short(\"-32768\"))
    // Expected: -5324-32768
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatsht2args_3() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:s
    // XPath: fn:concat(xs:short(\"32767\"),xs:short(\"-32768\"))
    // Expected: 32767-32768
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatsht2args_4() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:s
    // XPath: fn:concat(xs:short(\"-32768\"),xs:short(\"-5324\"))
    // Expected: -32768-5324
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatsht2args_5() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:s
    // XPath: fn:concat(xs:short(\"-32768\"),xs:short(\"32767\"))
    // Expected: -3276832767
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concat_1() {
    // Evaluation of concat function as per example 1 (for this function) from the F&O
    // XPath: fn:concat('un', 'grateful')
    // Expected: ungrateful
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concat_2() {
    // Evaluation of concat function as per example 2 (for this function) from the F&O
    // XPath: fn:concat('Thy ', (), 'old ', \"groans\", \"\", ' ring', ' y
    // Expected: Thy old groans ring yet in my ancient ea
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concat_3() {
    // Evaluation of concat function as per example 3 (for this function) from the F&O
    // XPath: fn:concat('Ciao!',())
    // Expected: Ciao!
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concat_4() {
    // Evaluation of concat function as per example 4 (for this function) from the F&O
    // XPath: fn:concat('Ingratitude, ', 'thou ', 'marble-hearted', ' fien
    // Expected: Ingratitude, thou marble-hearted fiend!
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concat_5() {
    // Evaluation of concat function that uses only upper case letters as part of argum
    // XPath: fn:concat(\"AB\",\"CD\")
    // Expected: ABCD
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concat_6() {
    // Evaluation of concat function that uses only lower case letters as part of argum
    // XPath: fn:concat(\"abc\",\"de\")
    // Expected: abcde
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concat_7() {
    // Evaluation of concat function that uses both upper and lower case letters as par
    // XPath: fn:concat(\"ABCDE\",\"abcde\")
    // Expected: ABCDEabcde
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concat_8() {
    // Evaluation of concat function that uses the empty string as part of argument Use
    // XPath: fn:count(fn:concat(\"\",\"\"))
    // Expected: 1
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concat_9() {
    // Evaluation of concat function that uses the 'upper-case' function as part of arg
    // XPath: fn:concat(fn:upper-case(\"Abc\"),fn:upper-case(\"DH\"))
    // Expected: ABCDH
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concat_10() {
    // Evaluation of concat function with argument set to '*****'
    // XPath: fn:concat(\"**\",\"***\")
    // Expected: *****
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concat_11() {
    // Evaluation of concat function with argument that uses another concat function
    // XPath: fn:concat(fn:concat(\"zzz\",\"zz\"),\"123\")
    // Expected: zzzzz123
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concat_12() {
    // Evaluation of concat function as an argument to the 'fn:boolean' function
    // XPath: fn:boolean(fn:concat(\"ab\",\"cde\"))
    // Expected: true
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concat_13() {
    // Evaluation of concat function as an argument to the 'fn:string' function
    // XPath: fn:string(fn:concat(\"abc\",\"de\"))
    // Expected: abcde
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concat_14() {
    // Evaluation of concat function as an argument to the 'fn:not' function
    // XPath: fn:not(fn:concat(\"ab\",\"cde\"))
    // Expected: false
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concat_15() {
    // Evaluation of concat function with argument set to '%$',#@!'
    // XPath: fn:concat(\"%$\",\"#@!\")
    // Expected: %$#@!
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concat_16() {
    // Evaluation of concat function with argument set to 'concat','concat'
    // XPath: fn:concat(\"concat\",\"concat\")
    // Expected: concatconcat
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concat_17() {
    // Evaluation of concat function as part of a boolean expression
    // XPath: fn:concat(\"abc\",\"abc\") and fn:concat(\"abc\",\"abc\")
    // Expected: true
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn k_concatfunc_3() {
    //  A test whose essence is: `concat('ab', 'c') eq 'abc'`.
    // XPath: concat(\"ab\", \"c\") eq \"abc\"
    // Expected: true
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn k_concatfunc_4() {
    //  A test whose essence is: `concat('ab', 'c') instance of xs:string`.
    // XPath: concat(\"ab\", \"c\") instance of xs:string
    // Expected: true
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn k_concatfunc_5() {
    //  A test whose essence is: `concat((), ()) instance of xs:string`.
    // XPath: concat((), ()) instance of xs:string
    // Expected: true
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn k_concatfunc_6() {
    //  A test whose essence is: `concat((), ()) eq ''`.
    // XPath: concat((), ()) eq \"\"
    // Expected: true
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn k_concatfunc_7() {
    //  A test whose essence is: `concat('a', 'b', 'c', (), 'd', 'e', 'f', 'g', 'h', '
    // XPath: concat('a', 'b', 'c', (), 'd', 'e', 'f', 'g', 'h', ' ', 'i',
    // Expected: true
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn k_concatfunc_8() {
    //  A test whose essence is: `concat(1, 2, 3) eq '123'`.
    // XPath: concat(1, 2, 3) eq \"123\"
    // Expected: true
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn k_concatfunc_9() {
    //  A test whose essence is: `concat(1, '2', 3) eq '123'`.
    // XPath: concat(1, \"2\", 3) eq \"123\"
    // Expected: true
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn cbcl_concat_001() {
    //  test fn:boolean(fn:concat()) where concat returns empty string
    // XPath: fn:boolean(fn:concat('', ''))
    // Expected: false
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}
