//! Test chunk 1 for op:concat
//! Contains 40 tests

use dep::xpath::stub_opconcat;

#[test]
fn op_concatnpi2args_1() {
    //  ******************************************************* Test: concatnpi2args-1
    // XPath: (xs:nonPositiveInteger(\"-999999999999999999\") || xs:nonPos
    // Expected: -999999999999999999-999999999999999999
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatnpi2args_2() {
    //  ******************************************************* Test: concatnpi2args-2
    // XPath: (xs:nonPositiveInteger(\"-475688437271870490\") || xs:nonPos
    // Expected: -475688437271870490-999999999999999999
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatnpi2args_3() {
    //  ******************************************************* Test: concatnpi2args-3
    // XPath: (xs:nonPositiveInteger(\"0\") || xs:nonPositiveInteger(\"-99
    // Expected: 0-999999999999999999
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatnpi2args_4() {
    //  ******************************************************* Test: concatnpi2args-4
    // XPath: (xs:nonPositiveInteger(\"-999999999999999999\") || xs:nonPos
    // Expected: -999999999999999999-475688437271870490
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatnpi2args_5() {
    //  ******************************************************* Test: concatnpi2args-5
    // XPath: (xs:nonPositiveInteger(\"-999999999999999999\") || xs:nonPos
    // Expected: -9999999999999999990
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatnni2args_1() {
    //  ******************************************************* Test: concatnni2args-1
    // XPath: (xs:nonNegativeInteger(\"0\") || xs:nonNegativeInteger(\"0\"
    // Expected: 00
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatnni2args_2() {
    //  ******************************************************* Test: concatnni2args-2
    // XPath: (xs:nonNegativeInteger(\"303884545991464527\") || xs:nonNega
    // Expected: 3038845459914645270
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatnni2args_3() {
    //  ******************************************************* Test: concatnni2args-3
    // XPath: (xs:nonNegativeInteger(\"999999999999999999\") || xs:nonNega
    // Expected: 9999999999999999990
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatnni2args_4() {
    //  ******************************************************* Test: concatnni2args-4
    // XPath: (xs:nonNegativeInteger(\"0\") || xs:nonNegativeInteger(\"303
    // Expected: 0303884545991464527
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatnni2args_5() {
    //  ******************************************************* Test: concatnni2args-5
    // XPath: (xs:nonNegativeInteger(\"0\") || xs:nonNegativeInteger(\"999
    // Expected: 0999999999999999999
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatsht2args_1() {
    //  ******************************************************* Test: concatsht2args-1
    // XPath: (xs:short(\"-32768\") || xs:short(\"-32768\"))
    // Expected: -32768-32768
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatsht2args_2() {
    //  ******************************************************* Test: concatsht2args-2
    // XPath: (xs:short(\"-5324\") || xs:short(\"-32768\"))
    // Expected: -5324-32768
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatsht2args_3() {
    //  ******************************************************* Test: concatsht2args-3
    // XPath: (xs:short(\"32767\") || xs:short(\"-32768\"))
    // Expected: 32767-32768
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatsht2args_4() {
    //  ******************************************************* Test: concatsht2args-4
    // XPath: (xs:short(\"-32768\") || xs:short(\"-5324\"))
    // Expected: -32768-5324
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatsht2args_5() {
    //  ******************************************************* Test: concatsht2args-5
    // XPath: (xs:short(\"-32768\") || xs:short(\"32767\"))
    // Expected: -3276832767
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concat_1() {
    // Evaluation of concat function as per example 1 (for this function)   from the F&
    // XPath: ('un' ||  'grateful')
    // Expected: ungrateful
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concat_2() {
    // Evaluation of concat function as per example 2 (for this function)   from the F&
    // XPath: ('Thy ' ||  () ||  'old ' ||  \"groans\" ||  \"\" ||  ' ring
    // Expected: Thy old groans ring yet in my ancient ea
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concat_3() {
    // Evaluation of concat function as per example 3 (for this function)   from the F&
    // XPath: ('Ciao!' || ())
    // Expected: Ciao!
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concat_4() {
    // Evaluation of concat function as per example 4 (for this function)   from the F&
    // XPath: ('Ingratitude, ' ||  'thou ' ||  'marble-hearted' ||  ' fien
    // Expected: Ingratitude, thou marble-hearted fiend!
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concat_5() {
    // Evaluation of concat function that uses only upper case letters as part of argum
    // XPath: (\"AB\" || \"CD\")
    // Expected: ABCD
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concat_6() {
    // Evaluation of concat function that uses only lower case letters as part of argum
    // XPath: (\"abc\" || \"de\")
    // Expected: abcde
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concat_7() {
    // Evaluation of concat function that uses both upper and lower case letters as par
    // XPath: (\"ABCDE\" || \"abcde\")
    // Expected: ABCDEabcde
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concat_8() {
    // Evaluation of concat function that uses the empty string as part of argument  Us
    // XPath: fn:count((\"\" || \"\"))
    // Expected: 1
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concat_9() {
    // Evaluation of concat function that uses the 'upper-case' function as part of arg
    // XPath: (fn:upper-case(\"Abc\") || fn:upper-case(\"DH\"))
    // Expected: ABCDH
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concat_10() {
    // Evaluation of concat function with argument set to '*****'
    // XPath: (\"**\" || \"***\")
    // Expected: *****
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concat_11() {
    // Evaluation of concat function with argument that uses another concat function
    // XPath: ((\"zzz\" || \"zz\") || \"123\")
    // Expected: zzzzz123
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concat_12() {
    // Evaluation of concat function as an argument to the 'fn:boolean' function
    // XPath: fn:boolean((\"ab\" || \"cde\"))
    // Expected: true
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concat_13() {
    // Evaluation of concat function as an argument to the 'fn:string' function
    // XPath: fn:string((\"abc\" || \"de\"))
    // Expected: abcde
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concat_14() {
    // Evaluation of concat function as an argument to the 'fn:not' function
    // XPath: fn:not((\"ab\" || \"cde\"))
    // Expected: false
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concat_15() {
    // Evaluation of concat function with argument set to '%$' || #@!'
    // XPath: (\"%$\" || \"#@!\")
    // Expected: %$#@!
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concat_16() {
    // Evaluation of concat function with argument set to 'concat' || 'concat'
    // XPath: (\"concat\" || \"concat\")
    // Expected: concatconcat
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concat_17() {
    // Evaluation of concat function as part of a boolean expression
    // XPath: (\"abc\" || \"abc\") and (\"abc\" || \"abc\")
    // Expected: true
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concat_20() {
    // Concat operator has higher precedence than eq
    // XPath: \"1234\" eq 12 || 34
    // Expected: true
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn k_concatop_3() {
    //   Purpose: A test whose essence is: `('ab' ||  'c') eq 'abc'`.
    // XPath: (\"ab\" ||  \"c\") eq \"abc\"
    // Expected: true
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn k_concatop_4() {
    //   Purpose: A test whose essence is: `concat('ab' ||  'c') instance of xs:string`
    // XPath: (\"ab\" ||  \"c\") instance of xs:string
    // Expected: true
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn k_concatop_5() {
    //   Purpose: A test whose essence is: `concat(() ||  ()) instance of xs:string`.
    // XPath: (() ||  ()) instance of xs:string
    // Expected: true
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn k_concatop_6() {
    //   Purpose: A test whose essence is: `concat(() ||  ()) eq ''`.
    // XPath: (() ||  ()) eq \"\"
    // Expected: true
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn k_concatop_7() {
    //   Purpose: A test whose essence is: `('a' ||  'b' ||  'c' ||  () ||  'd' ||  'e'
    // XPath: ('a' ||  'b' ||  'c' ||  () ||  'd' ||  'e' ||  'f' ||  'g'
    // Expected: true
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn k_concatop_8() {
    //   Purpose: A test whose essence is: `(1 ||  2 ||  3) eq '123'`.
    // XPath: (1 ||  2 ||  3) eq \"123\"
    // Expected: true
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn k_concatop_9() {
    //   Purpose: A test whose essence is: `(1 ||  '2' ||  3) eq '123'`.
    // XPath: (1 ||  \"2\" ||  3) eq \"123\"
    // Expected: true
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}
