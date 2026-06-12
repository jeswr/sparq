//! Test chunk 0 for op:numeric-add
//! Contains 50 tests

use dep::xpath::numeric_add_int;

#[test]
fn op_numeric_addint2args_1() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(-1873914410, -273569238) == -2147483648);
}

#[test]
fn op_numeric_addint2args_2() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(2147483647, -2147483648) == -1);
}

#[test]
fn op_numeric_addint2args_3() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(-273569238, -1873914410) == -2147483648);
}

#[test]
fn op_numeric_addint2args_4() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(-2147483648, 2147483647) == -1);
}

#[test]
fn op_numeric_addintg2args_1() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(830993497117024304, -999999999999999999) == -169006502882975695);
}

#[test]
fn op_numeric_addintg2args_2() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(999999999999999999, -999999999999999999) == 0);
}

#[test]
fn op_numeric_addintg2args_3() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(-999999999999999999, 830993497117024304) == -169006502882975695);
}

#[test]
fn op_numeric_addintg2args_4() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(-999999999999999999, 999999999999999999) == 0);
}

#[test]
fn op_numeric_adddec2args_1() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(617375191608514839, -999999999999999999) == -382624808391485160);
}

#[test]
fn op_numeric_adddec2args_2() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(999999999999999999, -999999999999999999) == 0);
}

#[test]
fn op_numeric_adddec2args_3() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(-999999999999999999, 617375191608514839) == -382624808391485160);
}

#[test]
fn op_numeric_adddec2args_4() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(-999999999999999999, 999999999999999999) == 0);
}

#[test]
fn op_numeric_addlng2args_1() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(-47175562203048468, -45058158165499290) == -92233720368547758);
}

#[test]
fn op_numeric_addlng2args_2() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(92233720368547758, -92233720368547758) == 0);
}

#[test]
fn op_numeric_addlng2args_3() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(-45058158165499290, -47175562203048468) == -92233720368547758);
}

#[test]
fn op_numeric_addlng2args_4() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(-92233720368547758, 92233720368547758) == 0);
}

#[test]
fn op_numeric_addusht2args_1() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(0, 0) == 0);
}

#[test]
fn op_numeric_addusht2args_2() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(44633, 0) == 44633);
}

#[test]
fn op_numeric_addusht2args_3() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(65535, 0) == 65535);
}

#[test]
fn op_numeric_addusht2args_4() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(0, 44633) == 44633);
}

#[test]
fn op_numeric_addusht2args_5() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(0, 65535) == 65535);
}

#[test]
fn op_numeric_addnint2args_1() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(-297014075999096793, -702985924000903206) == -999999999999999999);
}

#[test]
fn op_numeric_addnint2args_2() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(-1, -999999999999999999) == -1000000000000000000);
}

#[test]
fn op_numeric_addnint2args_3() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(-702985924000903206, -297014075999096793) == -999999999999999999);
}

#[test]
fn op_numeric_addnint2args_4() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(-999999999999999999, -1) == -1000000000000000000);
}

#[test]
fn op_numeric_addpint2args_1() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(1, 1) == 2);
}

#[test]
fn op_numeric_addpint2args_2() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(52704602390610033, 1) == 52704602390610034);
}

#[test]
fn op_numeric_addpint2args_3() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(999999999999999998, 1) == 999999999999999999);
}

#[test]
fn op_numeric_addpint2args_4() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(1, 52704602390610033) == 52704602390610034);
}

#[test]
fn op_numeric_addpint2args_5() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(1, 999999999999999998) == 999999999999999999);
}

#[test]
fn op_numeric_addulng2args_1() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(0, 0) == 0);
}

#[test]
fn op_numeric_addulng2args_2() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(130747108607674654, 0) == 130747108607674654);
}

#[test]
fn op_numeric_addulng2args_3() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(184467440737095516, 0) == 184467440737095516);
}

#[test]
fn op_numeric_addulng2args_4() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(0, 130747108607674654) == 130747108607674654);
}

#[test]
fn op_numeric_addulng2args_5() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(0, 184467440737095516) == 184467440737095516);
}

#[test]
fn op_numeric_addnpi2args_1() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(-475688437271870490, -524311562728129509) == -999999999999999999);
}

#[test]
fn op_numeric_addnpi2args_2() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(0, -999999999999999999) == -999999999999999999);
}

#[test]
fn op_numeric_addnpi2args_3() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(-524311562728129509, -475688437271870490) == -999999999999999999);
}

#[test]
fn op_numeric_addnpi2args_4() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(-999999999999999999, 0) == -999999999999999999);
}

#[test]
fn op_numeric_addnni2args_1() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(0, 0) == 0);
}

#[test]
fn op_numeric_addnni2args_2() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(303884545991464527, 0) == 303884545991464527);
}

#[test]
fn op_numeric_addnni2args_3() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(999999999999999999, 0) == 999999999999999999);
}

#[test]
fn op_numeric_addnni2args_4() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(0, 303884545991464527) == 303884545991464527);
}

#[test]
fn op_numeric_addnni2args_5() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(0, 999999999999999999) == 999999999999999999);
}

#[test]
fn op_numeric_addsht2args_1() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(-5324, -27444) == -32768);
}

#[test]
fn op_numeric_addsht2args_2() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(32767, -32768) == -1);
}

#[test]
fn op_numeric_addsht2args_3() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(-27444, -5324) == -32768);
}

#[test]
fn op_numeric_addsht2args_4() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    assert(numeric_add_int(-32768, 32767) == -1);
}

#[test]
fn op_numeric_addmix2args_3() {
    //  Simple addition test, second operator cast string to integer
    assert(numeric_add_int(1, 1) == 2);
}

#[test]
fn op_numeric_add_derived_1() {
    //  Evaluates The 'op:numeric-add' operator with the operators set as follows
    assert(numeric_add_int(10, 35) == 45);
}
