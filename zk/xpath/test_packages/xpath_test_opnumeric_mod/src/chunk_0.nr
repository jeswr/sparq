//! Test chunk 0 for op:numeric-mod
//! Contains 20 tests

use dep::xpath::numeric_mod_int;

#[test]
fn op_numeric_modint2args_1() {
    //  Evaluates The 'op:numeric-mod' operator with the arguments set as follows:
    assert(numeric_mod_int(-2147483648, 2147483647) == -1);
}

#[test]
fn op_numeric_modintg2args_1() {
    //  Evaluates The 'op:numeric-mod' operator with the arguments set as follows:
    assert(numeric_mod_int(-999999999999999999, 830993497117024304) == -169006502882975695);
}

#[test]
fn op_numeric_modintg2args_2() {
    //  Evaluates The 'op:numeric-mod' operator with the arguments set as follows:
    assert(numeric_mod_int(-999999999999999999, 999999999999999999) == 0);
}

#[test]
fn op_numeric_moddec2args_1() {
    //  Evaluates The 'op:numeric-mod' operator with the arguments set as follows:
    assert(numeric_mod_int(-999999999999999999, 617375191608514839) == -382624808391485160);
}

#[test]
fn op_numeric_moddec2args_2() {
    //  Evaluates The 'op:numeric-mod' operator with the arguments set as follows:
    assert(numeric_mod_int(-999999999999999999, 999999999999999999) == 0);
}

#[test]
fn op_numeric_modlng2args_1() {
    //  Evaluates The 'op:numeric-mod' operator with the arguments set as follows:
    assert(numeric_mod_int(-92233720368547758, 92233720368547758) == 0);
}

#[test]
fn op_numeric_modusht2args_1() {
    //  Evaluates The 'op:numeric-mod' operator with the arguments set as follows:
    assert(numeric_mod_int(0, 44633) == 0);
}

#[test]
fn op_numeric_modusht2args_2() {
    //  Evaluates The 'op:numeric-mod' operator with the arguments set as follows:
    assert(numeric_mod_int(0, 65535) == 0);
}

#[test]
fn op_numeric_modpint2args_1() {
    //  Evaluates The 'op:numeric-mod' operator with the arguments set as follows:
    assert(numeric_mod_int(1, 1) == 0);
}

#[test]
fn op_numeric_modpint2args_2() {
    //  Evaluates The 'op:numeric-mod' operator with the arguments set as follows:
    assert(numeric_mod_int(52704602390610033, 1) == 0);
}

#[test]
fn op_numeric_modpint2args_3() {
    //  Evaluates The 'op:numeric-mod' operator with the arguments set as follows:
    assert(numeric_mod_int(999999999999999999, 1) == 0);
}

#[test]
fn op_numeric_modpint2args_4() {
    //  Evaluates The 'op:numeric-mod' operator with the arguments set as follows:
    assert(numeric_mod_int(1, 52704602390610033) == 1);
}

#[test]
fn op_numeric_modpint2args_5() {
    //  Evaluates The 'op:numeric-mod' operator with the arguments set as follows:
    assert(numeric_mod_int(1, 999999999999999999) == 1);
}

#[test]
fn op_numeric_modulng2args_1() {
    //  Evaluates The 'op:numeric-mod' operator with the arguments set as follows:
    assert(numeric_mod_int(0, 130747108607674654) == 0);
}

#[test]
fn op_numeric_modulng2args_2() {
    //  Evaluates The 'op:numeric-mod' operator with the arguments set as follows:
    assert(numeric_mod_int(0, 184467440737095516) == 0);
}

#[test]
fn op_numeric_modnni2args_1() {
    //  Evaluates The 'op:numeric-mod' operator with the arguments set as follows:
    assert(numeric_mod_int(0, 303884545991464527) == 0);
}

#[test]
fn op_numeric_modnni2args_2() {
    //  Evaluates The 'op:numeric-mod' operator with the arguments set as follows:
    assert(numeric_mod_int(0, 999999999999999999) == 0);
}

#[test]
fn op_numeric_modsht2args_1() {
    //  Evaluates The 'op:numeric-mod' operator with the arguments set as follows:
    assert(numeric_mod_int(-32768, 32767) == -1);
}

#[test]
fn op_numeric_modmix2args_3() {
    //  Simple mod test, second operator cast string to integer
    assert(numeric_mod_int(1, 1) == 0);
}

#[test]
fn k2_numericmod_1() {
    //  Divisor is small negative xs:integer.
    assert(numeric_mod_int(-1, -1) == 0);
}
