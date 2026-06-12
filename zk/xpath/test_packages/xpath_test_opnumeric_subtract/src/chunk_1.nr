//! Test chunk 1 for op:numeric-subtract
//! Contains 1 tests

use dep::xpath::numeric_subtract_int;

#[test]
fn op_numeric_subtractmix2args_3() {
    //  Simple subtraction test, second operator cast string to integer
    assert(numeric_subtract_int(1, 1) == 0);
}
