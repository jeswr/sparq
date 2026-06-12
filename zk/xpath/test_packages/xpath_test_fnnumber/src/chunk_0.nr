//! Test chunk 0 for fn:number
//! Contains 50 tests

use dep::xpath::stub_fnnumber;

#[test]
fn fn_numberint1args_1() {
    //  Evaluates The 'number' function with the arguments set as follows: $arg = xs:in
    // XPath: fn:number(xs:int(\"-2147483648\"))
    // Expected: -2.147483648E9
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn fn_numberint1args_2() {
    //  Evaluates The 'number' function with the arguments set as follows: $arg = xs:in
    // XPath: fn:number(xs:int(\"-1873914410\"))
    // Expected: -1.87391441E9
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn fn_numberint1args_3() {
    //  Evaluates The 'number' function with the arguments set as follows: $arg = xs:in
    // XPath: fn:number(xs:int(\"2147483647\"))
    // Expected: 2.147483647E9
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn fn_numberintg1args_1() {
    //  Evaluates The 'number' function with the arguments set as follows: $arg = xs:in
    // XPath: fn:number(xs:integer(\"-999999999999999999\"))
    // Expected: -1.0E18
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn fn_numberintg1args_2() {
    //  Evaluates The 'number' function with the arguments set as follows: $arg = xs:in
    // XPath: fn:number(xs:integer(\"830993497117024304\")) eq 83099349711
    // Expected: true
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn fn_numberintg1args_3() {
    //  Evaluates The 'number' function with the arguments set as follows: $arg = xs:in
    // XPath: fn:number(xs:integer(\"999999999999999999\"))
    // Expected: 1.0E18
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn fn_numberdec1args_1() {
    //  Evaluates The 'number' function with the arguments set as follows: $arg = xs:de
    // XPath: fn:number(xs:decimal(\"-999999999999999999\"))
    // Expected: -1.0E18
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn fn_numberdec1args_2() {
    //  Evaluates The 'number' function with the arguments set as follows: $arg = xs:de
    // XPath: fn:number(xs:decimal(\"617375191608514839\")) eq 61737519160
    // Expected: true
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn fn_numberdec1args_3() {
    //  Evaluates The 'number' function with the arguments set as follows: $arg = xs:de
    // XPath: fn:number(xs:decimal(\"999999999999999999\"))
    // Expected: 1.0E18
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn fn_numberdbl1args_1() {
    //  Evaluates The 'number' function with the arguments set as follows: $arg = xs:do
    // XPath: fn:number(xs:double(\"-1.7976931348623157E308\"))
    // Expected: -1.7976931348623157E308
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn fn_numberdbl1args_2() {
    //  Evaluates The 'number' function with the arguments set as follows: $arg = xs:do
    // XPath: fn:number(xs:double(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn fn_numberdbl1args_3() {
    //  Evaluates The 'number' function with the arguments set as follows: $arg = xs:do
    // XPath: fn:number(xs:double(\"1.7976931348623157E308\"))
    // Expected: 1.7976931348623157E308
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn fn_numberflt1args_1() {
    //  Evaluates The 'number' function with the arguments set as follows: $arg = xs:fl
    // XPath: fn:number(xs:float(\"-3.4028235E38\")) eq -3.402823466385288
    // Expected: true
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn fn_numberflt1args_2() {
    //  Evaluates The 'number' function with the arguments set as follows: $arg = xs:fl
    // XPath: fn:number(xs:float(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn fn_numberlng1args_1() {
    //  Evaluates The 'number' function with the arguments set as follows: $arg = xs:lo
    // XPath: fn:number(xs:long(\"-92233720368547758\"))
    // Expected: -9.223372036854776E16
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn fn_numberlng1args_2() {
    //  Evaluates The 'number' function with the arguments set as follows: $arg = xs:lo
    // XPath: fn:number(xs:long(\"-47175562203048468\")) eq -4717556220304
    // Expected: true
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn fn_numberlng1args_3() {
    //  Evaluates The 'number' function with the arguments set as follows: $arg = xs:lo
    // XPath: fn:number(xs:long(\"92233720368547758\"))
    // Expected: 9.223372036854776E16
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn fn_numberusht1args_1() {
    //  Evaluates The 'number' function with the arguments set as follows: $arg = xs:un
    // XPath: fn:number(xs:unsignedShort(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn fn_numberusht1args_2() {
    //  Evaluates The 'number' function with the arguments set as follows: $arg = xs:un
    // XPath: fn:number(xs:unsignedShort(\"44633\"))
    // Expected: 44633
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn fn_numberusht1args_3() {
    //  Evaluates The 'number' function with the arguments set as follows: $arg = xs:un
    // XPath: fn:number(xs:unsignedShort(\"65535\"))
    // Expected: 65535
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn fn_numbernint1args_1() {
    //  Evaluates The 'number' function with the arguments set as follows: $arg = xs:ne
    // XPath: fn:number(xs:negativeInteger(\"-999999999999999999\"))
    // Expected: -1.0E18
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn fn_numbernint1args_2() {
    //  Evaluates The 'number' function with the arguments set as follows: $arg = xs:ne
    // XPath: fn:number(xs:negativeInteger(\"-297014075999096793\")) eq -2
    // Expected: true
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn fn_numbernint1args_3() {
    //  Evaluates The 'number' function with the arguments set as follows: $arg = xs:ne
    // XPath: fn:number(xs:negativeInteger(\"-1\"))
    // Expected: -1
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn fn_numberpint1args_1() {
    //  Evaluates The 'number' function with the arguments set as follows: $arg = xs:po
    // XPath: fn:number(xs:positiveInteger(\"1\"))
    // Expected: 1
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn fn_numberpint1args_2() {
    //  Evaluates The 'number' function with the arguments set as follows: $arg = xs:po
    // XPath: fn:number(xs:positiveInteger(\"52704602390610033\")) eq 5270
    // Expected: true
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn fn_numberpint1args_3() {
    //  Evaluates The 'number' function with the arguments set as follows: $arg = xs:po
    // XPath: fn:number(xs:positiveInteger(\"999999999999999999\"))
    // Expected: 1.0E18
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn fn_numberulng1args_1() {
    //  Evaluates The 'number' function with the arguments set as follows: $arg = xs:un
    // XPath: fn:number(xs:unsignedLong(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn fn_numberulng1args_3() {
    //  Evaluates The 'number' function with the arguments set as follows: $arg = xs:un
    // XPath: fn:number(xs:unsignedLong(\"184467440737095516\")) eq 1.8446
    // Expected: true
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn fn_numbernpi1args_1() {
    //  Evaluates The 'number' function with the arguments set as follows: $arg = xs:no
    // XPath: fn:number(xs:nonPositiveInteger(\"-999999999999999999\"))
    // Expected: -1.0E18
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn fn_numbernpi1args_3() {
    //  Evaluates The 'number' function with the arguments set as follows: $arg = xs:no
    // XPath: fn:number(xs:nonPositiveInteger(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn fn_numbernni1args_1() {
    //  Evaluates The 'number' function with the arguments set as follows: $arg = xs:no
    // XPath: fn:number(xs:nonNegativeInteger(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn fn_numbernni1args_2() {
    //  Evaluates The 'number' function with the arguments set as follows: $arg = xs:no
    // XPath: fn:number(xs:nonNegativeInteger(\"303884545991464527\")) eq
    // Expected: true
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn fn_numbernni1args_3() {
    //  Evaluates The 'number' function with the arguments set as follows: $arg = xs:no
    // XPath: fn:number(xs:nonNegativeInteger(\"999999999999999999\"))
    // Expected: 1.0E18
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn fn_numbersht1args_1() {
    //  Evaluates The 'number' function with the arguments set as follows: $arg = xs:sh
    // XPath: fn:number(xs:short(\"-32768\"))
    // Expected: -32768
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn fn_numbersht1args_2() {
    //  Evaluates The 'number' function with the arguments set as follows: $arg = xs:sh
    // XPath: fn:number(xs:short(\"-5324\"))
    // Expected: -5324
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn fn_numbersht1args_3() {
    //  Evaluates The 'number' function with the arguments set as follows: $arg = xs:sh
    // XPath: fn:number(xs:short(\"32767\"))
    // Expected: 32767
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn fn_number_1() {
    //  Evaluation of the fn:number function with empty sequence as an argument.
    // XPath: fn:number(())
    // Expected: NaN
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn fn_number_2() {
    //  Evaluation of the fn:number function with a string set as an argument.
    // XPath: fn:number(\"A String\")
    // Expected: NaN
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn k_nodenumberfunc_2() {
    //  A test whose essence is: `fn:number(1) eq 1`.
    // XPath: fn:number(1) eq 1
    // Expected: true
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn k_nodenumberfunc_3() {
    //  A test whose essence is: `not(fn:number('results in NaN'))`.
    // XPath: not(fn:number(\"results in NaN\"))
    // Expected: true
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn k_nodenumberfunc_4() {
    //  A test whose essence is: `fn:number('results in NaN') instance of xs:double`.
    // XPath: fn:number(\"results in NaN\") instance of xs:double
    // Expected: true
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn k_nodenumberfunc_5() {
    //  A test whose essence is: `fn:number(1) instance of xs:double`.
    // XPath: fn:number(1) instance of xs:double
    // Expected: true
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn k_nodenumberfunc_6() {
    //  A test whose essence is: `fn:number(1.1) instance of xs:double`.
    // XPath: fn:number(1.1) instance of xs:double
    // Expected: true
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn k_nodenumberfunc_7() {
    //  A test whose essence is: `fn:number(xs:float(3)) instance of xs:double`.
    // XPath: fn:number(xs:float(3)) instance of xs:double
    // Expected: true
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn k_nodenumberfunc_8() {
    //  A test whose essence is: `fn:number(xs:double(3)) instance of xs:double`.
    // XPath: fn:number(xs:double(3)) instance of xs:double
    // Expected: true
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn k_nodenumberfunc_9() {
    //  A test whose essence is: `fn:number('NaN') instance of xs:double`.
    // XPath: fn:number(\"NaN\") instance of xs:double
    // Expected: true
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn k_nodenumberfunc_10() {
    //  A test whose essence is: `fn:number(()) instance of xs:double`.
    // XPath: fn:number(()) instance of xs:double
    // Expected: true
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn k_nodenumberfunc_11() {
    //  A test whose essence is: `not(fn:number(()))`.
    // XPath: not(fn:number(()))
    // Expected: true
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn k_nodenumberfunc_12() {
    //  fn:number() applied to a type which a cast regardless of source value never wou
    // XPath: string(number(xs:anyURI(\"example.com/\"))) eq \"NaN\"
    // Expected: true
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}

#[test]
fn k_nodenumberfunc_13() {
    //  A test whose essence is: `string(number(xs:anyURI('1'))) eq 'NaN'`.
    // XPath: string(number(xs:anyURI(\"1\"))) eq \"NaN\"
    // Expected: true
    // Calls stub function - will fail until fn:number is implemented
    let _ = stub_fnnumber();
}
