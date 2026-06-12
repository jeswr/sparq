//! Test chunk 0 for fn:ceiling
//! Contains 33 tests

use dep::xpath::ceil_int;

#[test]
fn fn_ceilingint1args_1() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    assert(ceil_int(-2147483648) == -2147483648);
}

#[test]
fn fn_ceilingint1args_2() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    assert(ceil_int(-1873914410) == -1873914410);
}

#[test]
fn fn_ceilingint1args_3() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    assert(ceil_int(2147483647) == 2147483647);
}

#[test]
fn fn_ceilingintg1args_1() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    assert(ceil_int(-999999999999999999) == -999999999999999999);
}

#[test]
fn fn_ceilingintg1args_2() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    assert(ceil_int(830993497117024304) == 830993497117024304);
}

#[test]
fn fn_ceilingintg1args_3() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    assert(ceil_int(999999999999999999) == 999999999999999999);
}

#[test]
fn fn_ceilingdec1args_1() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    assert(ceil_int(-999999999999999999) == -999999999999999999);
}

#[test]
fn fn_ceilingdec1args_2() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    assert(ceil_int(617375191608514839) == 617375191608514839);
}

#[test]
fn fn_ceilingdec1args_3() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    assert(ceil_int(999999999999999999) == 999999999999999999);
}

#[test]
fn fn_ceilinglng1args_1() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    assert(ceil_int(-92233720368547758) == -92233720368547758);
}

#[test]
fn fn_ceilinglng1args_2() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    assert(ceil_int(-47175562203048468) == -47175562203048468);
}

#[test]
fn fn_ceilinglng1args_3() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    assert(ceil_int(92233720368547758) == 92233720368547758);
}

#[test]
fn fn_ceilingusht1args_1() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    assert(ceil_int(0) == 0);
}

#[test]
fn fn_ceilingusht1args_2() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    assert(ceil_int(44633) == 44633);
}

#[test]
fn fn_ceilingusht1args_3() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    assert(ceil_int(65535) == 65535);
}

#[test]
fn fn_ceilingnint1args_1() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    assert(ceil_int(-999999999999999999) == -999999999999999999);
}

#[test]
fn fn_ceilingnint1args_2() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    assert(ceil_int(-297014075999096793) == -297014075999096793);
}

#[test]
fn fn_ceilingnint1args_3() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    assert(ceil_int(-1) == -1);
}

#[test]
fn fn_ceilingpint1args_1() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    assert(ceil_int(1) == 1);
}

#[test]
fn fn_ceilingpint1args_2() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    assert(ceil_int(52704602390610033) == 52704602390610033);
}

#[test]
fn fn_ceilingpint1args_3() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    assert(ceil_int(999999999999999999) == 999999999999999999);
}

#[test]
fn fn_ceilingulng1args_1() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    assert(ceil_int(0) == 0);
}

#[test]
fn fn_ceilingulng1args_2() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    assert(ceil_int(130747108607674654) == 130747108607674654);
}

#[test]
fn fn_ceilingulng1args_3() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    assert(ceil_int(184467440737095516) == 184467440737095516);
}

#[test]
fn fn_ceilingnpi1args_1() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    assert(ceil_int(-999999999999999999) == -999999999999999999);
}

#[test]
fn fn_ceilingnpi1args_2() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    assert(ceil_int(-475688437271870490) == -475688437271870490);
}

#[test]
fn fn_ceilingnpi1args_3() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    assert(ceil_int(0) == 0);
}

#[test]
fn fn_ceilingnni1args_1() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    assert(ceil_int(0) == 0);
}

#[test]
fn fn_ceilingnni1args_2() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    assert(ceil_int(303884545991464527) == 303884545991464527);
}

#[test]
fn fn_ceilingnni1args_3() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    assert(ceil_int(999999999999999999) == 999999999999999999);
}

#[test]
fn fn_ceilingsht1args_1() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    assert(ceil_int(-32768) == -32768);
}

#[test]
fn fn_ceilingsht1args_2() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    assert(ceil_int(-5324) == -5324);
}

#[test]
fn fn_ceilingsht1args_3() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    assert(ceil_int(32767) == 32767);
}
