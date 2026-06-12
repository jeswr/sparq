//! Test chunk 0 for fn:floor
//! Contains 33 tests

use dep::xpath::floor_int;

#[test]
fn fn_floorint1args_1() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    assert(floor_int(-2147483648) == -2147483648);
}

#[test]
fn fn_floorint1args_2() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    assert(floor_int(-1873914410) == -1873914410);
}

#[test]
fn fn_floorint1args_3() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    assert(floor_int(2147483647) == 2147483647);
}

#[test]
fn fn_floorintg1args_1() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    assert(floor_int(-999999999999999999) == -999999999999999999);
}

#[test]
fn fn_floorintg1args_2() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    assert(floor_int(830993497117024304) == 830993497117024304);
}

#[test]
fn fn_floorintg1args_3() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    assert(floor_int(999999999999999999) == 999999999999999999);
}

#[test]
fn fn_floordec1args_1() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    assert(floor_int(-999999999999999999) == -999999999999999999);
}

#[test]
fn fn_floordec1args_2() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    assert(floor_int(617375191608514839) == 617375191608514839);
}

#[test]
fn fn_floordec1args_3() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    assert(floor_int(999999999999999999) == 999999999999999999);
}

#[test]
fn fn_floorlng1args_1() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    assert(floor_int(-92233720368547758) == -92233720368547758);
}

#[test]
fn fn_floorlng1args_2() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    assert(floor_int(-47175562203048468) == -47175562203048468);
}

#[test]
fn fn_floorlng1args_3() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    assert(floor_int(92233720368547758) == 92233720368547758);
}

#[test]
fn fn_floorusht1args_1() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    assert(floor_int(0) == 0);
}

#[test]
fn fn_floorusht1args_2() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    assert(floor_int(44633) == 44633);
}

#[test]
fn fn_floorusht1args_3() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    assert(floor_int(65535) == 65535);
}

#[test]
fn fn_floornint1args_1() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    assert(floor_int(-999999999999999999) == -999999999999999999);
}

#[test]
fn fn_floornint1args_2() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    assert(floor_int(-297014075999096793) == -297014075999096793);
}

#[test]
fn fn_floornint1args_3() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    assert(floor_int(-1) == -1);
}

#[test]
fn fn_floorpint1args_1() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    assert(floor_int(1) == 1);
}

#[test]
fn fn_floorpint1args_2() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    assert(floor_int(52704602390610033) == 52704602390610033);
}

#[test]
fn fn_floorpint1args_3() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    assert(floor_int(999999999999999999) == 999999999999999999);
}

#[test]
fn fn_floorulng1args_1() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    assert(floor_int(0) == 0);
}

#[test]
fn fn_floorulng1args_2() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    assert(floor_int(130747108607674654) == 130747108607674654);
}

#[test]
fn fn_floorulng1args_3() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    assert(floor_int(184467440737095516) == 184467440737095516);
}

#[test]
fn fn_floornpi1args_1() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    assert(floor_int(-999999999999999999) == -999999999999999999);
}

#[test]
fn fn_floornpi1args_2() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    assert(floor_int(-475688437271870490) == -475688437271870490);
}

#[test]
fn fn_floornpi1args_3() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    assert(floor_int(0) == 0);
}

#[test]
fn fn_floornni1args_1() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    assert(floor_int(0) == 0);
}

#[test]
fn fn_floornni1args_2() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    assert(floor_int(303884545991464527) == 303884545991464527);
}

#[test]
fn fn_floornni1args_3() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    assert(floor_int(999999999999999999) == 999999999999999999);
}

#[test]
fn fn_floorsht1args_1() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    assert(floor_int(-32768) == -32768);
}

#[test]
fn fn_floorsht1args_2() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    assert(floor_int(-5324) == -5324);
}

#[test]
fn fn_floorsht1args_3() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    assert(floor_int(32767) == 32767);
}
