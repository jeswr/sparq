//! Test chunk 0 for fn:round
//! Contains 31 tests

use dep::xpath::round_int;

#[test]
fn fn_roundint1args_2() {
    //  Evaluates The 'round' function with the arguments set as follows: $arg =
    assert(round_int(-1873914410) == -1873914410);
}

#[test]
fn fn_roundint1args_3() {
    //  Evaluates The 'round' function with the arguments set as follows: $arg =
    assert(round_int(2147483647) == 2147483647);
}

#[test]
fn fn_roundintg1args_1() {
    //  Evaluates The 'round' function with the arguments set as follows: $arg =
    assert(round_int(-999999999999999999) == -999999999999999999);
}

#[test]
fn fn_roundintg1args_2() {
    //  Evaluates The 'round' function with the arguments set as follows: $arg =
    assert(round_int(830993497117024304) == 830993497117024304);
}

#[test]
fn fn_roundintg1args_3() {
    //  Evaluates The 'round' function with the arguments set as follows: $arg =
    assert(round_int(999999999999999999) == 999999999999999999);
}

#[test]
fn fn_rounddec1args_1() {
    //  Evaluates The 'round' function with the arguments set as follows: $arg =
    assert(round_int(-999999999999999999) == -999999999999999999);
}

#[test]
fn fn_rounddec1args_3() {
    //  Evaluates The 'round' function with the arguments set as follows: $arg =
    assert(round_int(999999999999999999) == 999999999999999999);
}

#[test]
fn fn_roundlng1args_1() {
    //  Evaluates The 'round' function with the arguments set as follows: $arg =
    assert(round_int(-92233720368547758) == -92233720368547758);
}

#[test]
fn fn_roundlng1args_2() {
    //  Evaluates The 'round' function with the arguments set as follows: $arg =
    assert(round_int(-47175562203048468) == -47175562203048468);
}

#[test]
fn fn_roundlng1args_3() {
    //  Evaluates The 'round' function with the arguments set as follows: $arg =
    assert(round_int(92233720368547758) == 92233720368547758);
}

#[test]
fn fn_roundusht1args_1() {
    //  Evaluates The 'round' function with the arguments set as follows: $arg =
    assert(round_int(0) == 0);
}

#[test]
fn fn_roundusht1args_2() {
    //  Evaluates The 'round' function with the arguments set as follows: $arg =
    assert(round_int(44633) == 44633);
}

#[test]
fn fn_roundusht1args_3() {
    //  Evaluates The 'round' function with the arguments set as follows: $arg =
    assert(round_int(65535) == 65535);
}

#[test]
fn fn_roundnint1args_1() {
    //  Evaluates The 'round' function with the arguments set as follows: $arg =
    assert(round_int(-999999999999999999) == -999999999999999999);
}

#[test]
fn fn_roundnint1args_2() {
    //  Evaluates The 'round' function with the arguments set as follows: $arg =
    assert(round_int(-297014075999096793) == -297014075999096793);
}

#[test]
fn fn_roundnint1args_3() {
    //  Evaluates The 'round' function with the arguments set as follows: $arg =
    assert(round_int(-1) == -1);
}

#[test]
fn fn_roundpint1args_1() {
    //  Evaluates The 'round' function with the arguments set as follows: $arg =
    assert(round_int(1) == 1);
}

#[test]
fn fn_roundpint1args_2() {
    //  Evaluates The 'round' function with the arguments set as follows: $arg =
    assert(round_int(52704602390610033) == 52704602390610033);
}

#[test]
fn fn_roundpint1args_3() {
    //  Evaluates The 'round' function with the arguments set as follows: $arg =
    assert(round_int(999999999999999999) == 999999999999999999);
}

#[test]
fn fn_roundulng1args_1() {
    //  Evaluates The 'round' function with the arguments set as follows: $arg =
    assert(round_int(0) == 0);
}

#[test]
fn fn_roundulng1args_2() {
    //  Evaluates The 'round' function with the arguments set as follows: $arg =
    assert(round_int(130747108607674654) == 130747108607674654);
}

#[test]
fn fn_roundulng1args_3() {
    //  Evaluates The 'round' function with the arguments set as follows: $arg =
    assert(round_int(184467440737095516) == 184467440737095516);
}

#[test]
fn fn_roundnpi1args_1() {
    //  Evaluates The 'round' function with the arguments set as follows: $arg =
    assert(round_int(-999999999999999999) == -999999999999999999);
}

#[test]
fn fn_roundnpi1args_2() {
    //  Evaluates The 'round' function with the arguments set as follows: $arg =
    assert(round_int(-475688437271870490) == -475688437271870490);
}

#[test]
fn fn_roundnpi1args_3() {
    //  Evaluates The 'round' function with the arguments set as follows: $arg =
    assert(round_int(0) == 0);
}

#[test]
fn fn_roundnni1args_1() {
    //  Evaluates The 'round' function with the arguments set as follows: $arg =
    assert(round_int(0) == 0);
}

#[test]
fn fn_roundnni1args_2() {
    //  Evaluates The 'round' function with the arguments set as follows: $arg =
    assert(round_int(303884545991464527) == 303884545991464527);
}

#[test]
fn fn_roundnni1args_3() {
    //  Evaluates The 'round' function with the arguments set as follows: $arg =
    assert(round_int(999999999999999999) == 999999999999999999);
}

#[test]
fn fn_roundsht1args_1() {
    //  Evaluates The 'round' function with the arguments set as follows: $arg =
    assert(round_int(-32768) == -32768);
}

#[test]
fn fn_roundsht1args_2() {
    //  Evaluates The 'round' function with the arguments set as follows: $arg =
    assert(round_int(-5324) == -5324);
}

#[test]
fn fn_roundsht1args_3() {
    //  Evaluates The 'round' function with the arguments set as follows: $arg =
    assert(round_int(32767) == 32767);
}
