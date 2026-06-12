//! Test chunk 1 for op:numeric-add
//! Contains 4 tests

use dep::xpath::numeric_add_int;

#[test]
fn op_numeric_add_derived_2() {
    //  Evaluates The 'op:numeric-add' operator with the operators set as follows
    assert(numeric_add_int(10, -15) == -5);
}

#[test]
fn op_numeric_add_derived_3() {
    //  Evaluates The 'op:numeric-add' operator with the operators set as follows
    assert(numeric_add_int(10, 15) == 25);
}

#[test]
fn op_numeric_add_derived_4() {
    //  Evaluates The 'op:numeric-add' operator with the operators set as follows
    assert(numeric_add_int(10, 145) == 155);
}

#[test]
fn op_numeric_add_derived_5() {
    //  Evaluates The 'op:numeric-add' operator with the operators set as follows
    assert(numeric_add_int(10, -5) == 5);
}
