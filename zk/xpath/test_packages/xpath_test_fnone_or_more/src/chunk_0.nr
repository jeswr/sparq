//! Test chunk 0 for fn:one-or-more
//! Contains 47 tests

use dep::xpath::stub_fnone_or_more;

#[test]
fn fn_one_or_moreint1args_1() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:int(\"-2147483648\"))
    // Expected: -2147483648
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_moreint1args_2() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:int(\"-1873914410\"))
    // Expected: -1873914410
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_moreint1args_3() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:int(\"2147483647\"))
    // Expected: 2147483647
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_moreintg1args_1() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:integer(\"-999999999999999999\"))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_moreintg1args_2() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:integer(\"830993497117024304\"))
    // Expected: 830993497117024304
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_moreintg1args_3() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:integer(\"999999999999999999\"))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_moredec1args_1() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:decimal(\"-999999999999999999\"))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_moredec1args_2() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:decimal(\"617375191608514839\"))
    // Expected: 617375191608514839
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_moredec1args_3() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:decimal(\"999999999999999999\"))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_moredbl1args_1() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:double(\"-1.7976931348623157E308\"))
    // Expected: -1.7976931348623157E308
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_moredbl1args_2() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:double(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_moredbl1args_3() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:double(\"1.7976931348623157E308\"))
    // Expected: 1.7976931348623157E308
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_moreflt1args_1() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:float(\"-3.4028235E38\"))
    // Expected: -3.4028235E38
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_moreflt1args_2() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:float(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_moreflt1args_3() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:float(\"3.4028235E38\"))
    // Expected: 3.4028235E38
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_morelng1args_1() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:long(\"-92233720368547758\"))
    // Expected: -92233720368547758
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_morelng1args_2() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:long(\"-47175562203048468\"))
    // Expected: -47175562203048468
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_morelng1args_3() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:long(\"92233720368547758\"))
    // Expected: 92233720368547758
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_moreusht1args_1() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:unsignedShort(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_moreusht1args_2() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:unsignedShort(\"44633\"))
    // Expected: 44633
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_moreusht1args_3() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:unsignedShort(\"65535\"))
    // Expected: 65535
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_morenint1args_1() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:negativeInteger(\"-999999999999999999\"))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_morenint1args_2() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:negativeInteger(\"-297014075999096793\"))
    // Expected: -297014075999096793
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_morenint1args_3() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:negativeInteger(\"-1\"))
    // Expected: -1
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_morepint1args_1() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:positiveInteger(\"1\"))
    // Expected: 1
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_morepint1args_2() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:positiveInteger(\"52704602390610033\"))
    // Expected: 52704602390610033
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_morepint1args_3() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:positiveInteger(\"999999999999999999\"))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_moreulng1args_1() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:unsignedLong(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_moreulng1args_2() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:unsignedLong(\"130747108607674654\"))
    // Expected: 130747108607674654
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_moreulng1args_3() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:unsignedLong(\"184467440737095516\"))
    // Expected: 184467440737095516
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_morenpi1args_1() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:nonPositiveInteger(\"-999999999999999999\"
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_morenpi1args_2() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:nonPositiveInteger(\"-475688437271870490\"
    // Expected: -475688437271870490
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_morenpi1args_3() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:nonPositiveInteger(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_morenni1args_1() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:nonNegativeInteger(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_morenni1args_2() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:nonNegativeInteger(\"303884545991464527\")
    // Expected: 303884545991464527
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_morenni1args_3() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:nonNegativeInteger(\"999999999999999999\")
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_moresht1args_1() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:short(\"-32768\"))
    // Expected: -32768
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_moresht1args_2() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:short(\"-5324\"))
    // Expected: -5324
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn fn_one_or_moresht1args_3() {
    //  Evaluates The 'one-or-more' function with the arguments set as follows: $arg =
    // XPath: fn:one-or-more(xs:short(\"32767\"))
    // Expected: 32767
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn k_seqoneormorefunc_3() {
    //  A test whose essence is: `one-or-more('one')`.
    // XPath: one-or-more(\"one\")
    // Expected: one
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn k_seqoneormorefunc_4() {
    //  A test whose essence is: `exists(one-or-more(('one', 2)))`.
    // XPath: exists(one-or-more((\"one\", 2)))
    // Expected: true
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn k_seqoneormorefunc_5() {
    //  A test whose essence is: `count(one-or-more( 'one' )) eq 1`.
    // XPath: count(one-or-more( \"one\" )) eq 1
    // Expected: true
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn k_seqoneormorefunc_6() {
    //  A test whose essence is: `count(one-or-more( (1, 2, 3, 'four') )) eq 4`.
    // XPath: count(one-or-more( (1, 2, 3, \"four\") )) eq 4
    // Expected: true
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn k2_seqoneormorefunc_1() {
    //  Have fn:one-or-more() as an operand to an expression that requires zero or more
    // XPath: codepoints-to-string(one-or-more(string-to-codepoints(\"foo\
    // Expected: foo
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn cbcl_one_or_more_001() {
    //  test fn:one-or-more-one on a count-preserving function
    // XPath: declare function local:generate($arg as xs:integer?) { if ($
    // Expected: 3 2 1
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn cbcl_one_or_more_003() {
    //  test fn:boolean on fn:one-or-more-one on a sequence of nodes
    // XPath: boolean(one-or-more(for $x in 1 to month-from-date(current-d
    // Expected: true
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}

#[test]
fn cbcl_one_or_more_007() {
    //  Tests one or more on a single boolean value
    // XPath: boolean(one-or-more(for $x in (1 to 10)[. div 2 = 1] return
    // Expected: true
    // Calls stub function - will fail until fn:one-or-more is implemented
    let _ = stub_fnone_or_more();
}
