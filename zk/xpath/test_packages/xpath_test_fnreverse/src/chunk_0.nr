//! Test chunk 0 for fn:reverse
//! Contains 50 tests

use dep::xpath::stub_fnreverse;

#[test]
fn fn_reverseint1args_1() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:i
    // XPath: fn:reverse((xs:int(\"-2147483648\")))
    // Expected: -2147483648
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reverseint1args_2() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:i
    // XPath: fn:reverse((xs:int(\"-1873914410\")))
    // Expected: -1873914410
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reverseint1args_3() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:i
    // XPath: fn:reverse((xs:int(\"2147483647\")))
    // Expected: 2147483647
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reverseintg1args_1() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:i
    // XPath: fn:reverse((xs:integer(\"-999999999999999999\")))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reverseintg1args_2() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:i
    // XPath: fn:reverse((xs:integer(\"830993497117024304\")))
    // Expected: 830993497117024304
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reverseintg1args_3() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:i
    // XPath: fn:reverse((xs:integer(\"999999999999999999\")))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reversedec1args_1() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:d
    // XPath: fn:reverse((xs:decimal(\"-999999999999999999\")))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reversedec1args_2() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:d
    // XPath: fn:reverse((xs:decimal(\"617375191608514839\")))
    // Expected: 617375191608514839
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reversedec1args_3() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:d
    // XPath: fn:reverse((xs:decimal(\"999999999999999999\")))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reversedbl1args_1() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:d
    // XPath: fn:reverse((xs:double(\"-1.7976931348623157E308\")))
    // Expected: -1.7976931348623157E308
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reversedbl1args_2() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:d
    // XPath: fn:reverse((xs:double(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reversedbl1args_3() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:d
    // XPath: fn:reverse((xs:double(\"1.7976931348623157E308\")))
    // Expected: 1.7976931348623157E308
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reverseflt1args_1() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:f
    // XPath: fn:reverse((xs:float(\"-3.4028235E38\")))
    // Expected: xs:float("-3.4028235E38")
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reverseflt1args_2() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:f
    // XPath: fn:reverse((xs:float(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reverseflt1args_3() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:f
    // XPath: fn:reverse((xs:float(\"3.4028235E38\")))
    // Expected: xs:float("3.4028235E38")
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reverselng1args_1() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:l
    // XPath: fn:reverse((xs:long(\"-92233720368547758\")))
    // Expected: -92233720368547758
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reverselng1args_2() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:l
    // XPath: fn:reverse((xs:long(\"-47175562203048468\")))
    // Expected: -47175562203048468
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reverselng1args_3() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:l
    // XPath: fn:reverse((xs:long(\"92233720368547758\")))
    // Expected: 92233720368547758
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reverseusht1args_1() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:u
    // XPath: fn:reverse((xs:unsignedShort(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reverseusht1args_2() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:u
    // XPath: fn:reverse((xs:unsignedShort(\"44633\")))
    // Expected: 44633
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reverseusht1args_3() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:u
    // XPath: fn:reverse((xs:unsignedShort(\"65535\")))
    // Expected: 65535
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reversenint1args_1() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:n
    // XPath: fn:reverse((xs:negativeInteger(\"-999999999999999999\")))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reversenint1args_2() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:n
    // XPath: fn:reverse((xs:negativeInteger(\"-297014075999096793\")))
    // Expected: -297014075999096793
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reversenint1args_3() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:n
    // XPath: fn:reverse((xs:negativeInteger(\"-1\")))
    // Expected: -1
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reversepint1args_1() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:p
    // XPath: fn:reverse((xs:positiveInteger(\"1\")))
    // Expected: 1
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reversepint1args_2() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:p
    // XPath: fn:reverse((xs:positiveInteger(\"52704602390610033\")))
    // Expected: 52704602390610033
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reversepint1args_3() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:p
    // XPath: fn:reverse((xs:positiveInteger(\"999999999999999999\")))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reverseulng1args_1() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:u
    // XPath: fn:reverse((xs:unsignedLong(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reverseulng1args_2() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:u
    // XPath: fn:reverse((xs:unsignedLong(\"130747108607674654\")))
    // Expected: 130747108607674654
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reverseulng1args_3() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:u
    // XPath: fn:reverse((xs:unsignedLong(\"184467440737095516\")))
    // Expected: 184467440737095516
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reversenpi1args_1() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:n
    // XPath: fn:reverse((xs:nonPositiveInteger(\"-999999999999999999\")))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reversenpi1args_2() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:n
    // XPath: fn:reverse((xs:nonPositiveInteger(\"-475688437271870490\")))
    // Expected: -475688437271870490
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reversenpi1args_3() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:n
    // XPath: fn:reverse((xs:nonPositiveInteger(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reversenni1args_1() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:n
    // XPath: fn:reverse((xs:nonNegativeInteger(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reversenni1args_2() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:n
    // XPath: fn:reverse((xs:nonNegativeInteger(\"303884545991464527\")))
    // Expected: 303884545991464527
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reversenni1args_3() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:n
    // XPath: fn:reverse((xs:nonNegativeInteger(\"999999999999999999\")))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reversesht1args_1() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:s
    // XPath: fn:reverse((xs:short(\"-32768\")))
    // Expected: -32768
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reversesht1args_2() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:s
    // XPath: fn:reverse((xs:short(\"-5324\")))
    // Expected: -5324
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn fn_reversesht1args_3() {
    //  Evaluates The 'reverse' function with the arguments set as follows: $arg = xs:s
    // XPath: fn:reverse((xs:short(\"32767\")))
    // Expected: 32767
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn reverse_003() {
    // Reverse sequence of nodes on the ancestor axis
    // XPath: string-join(reverse(((//*:Open)[1])/ancestor-or-self::*/loca
    // Expected: "Open~Schedule~Auction~AuctionWatchList"
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn k_seqreversefunc_4() {
    //  A test whose essence is: `not(empty(reverse((1))))`.
    // XPath: not(empty(reverse((1))))
    // Expected: true
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn k_seqreversefunc_7() {
    //  A test whose essence is: `exists(reverse((1)))`.
    // XPath: exists(reverse((1)))
    // Expected: true
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn k_seqreversefunc_8() {
    //  A test whose essence is: `reverse((1, current-time())[1])`.
    // XPath: reverse((1, current-time())[1])
    // Expected: 1
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn k_seqreversefunc_9() {
    //  A test whose essence is: `exists(reverse((1, 2, 3)))`.
    // XPath: exists(reverse((1, 2, 3)))
    // Expected: true
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn k_seqreversefunc_10() {
    //  A test whose essence is: `count(reverse((1, 2, 3)))`.
    // XPath: count(reverse((1, 2, 3)))
    // Expected: 3
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn k_seqreversefunc_11() {
    //  A test whose essence is: `count(reverse((1, 2, 3)))`.
    // XPath: count(reverse((1, 2, 3)))
    // Expected: 3
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn k_seqreversefunc_12() {
    //  A test whose essence is: `deep-equal((3, 2, 1), reverse(1 to 3))`.
    // XPath: deep-equal((3, 2, 1), reverse(1 to 3))
    // Expected: true
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn k_seqreversefunc_13() {
    //  A test whose essence is: `deep-equal((3, 2, 1), reverse((1, 2, 3)))`.
    // XPath: deep-equal((3, 2, 1), reverse((1, 2, 3)))
    // Expected: true
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn k_seqreversefunc_14() {
    //  A test whose essence is: `deep-equal((11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1), rever
    // XPath: deep-equal((11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1), reverse(((),
    // Expected: true
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn k_seqreversefunc_16() {
    //  Apply a predicate to the result of fn:reverse().
    // XPath: reverse((1, 2, 3))[last()] eq 1
    // Expected: true
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}
