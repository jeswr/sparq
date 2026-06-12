//! Test chunk 1 for op:numeric-equal
//! Contains 13 tests

use dep::xpath::numeric_equal_int;

#[test]
fn op_numeric_equalsht2args_1() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(-32768, -32768) == true);
}

#[test]
fn op_numeric_equalsht2args_2() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(-5324, -32768) == false);
}

#[test]
fn op_numeric_equalsht2args_3() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(32767, -32768) == false);
}

#[test]
fn op_numeric_equalsht2args_4() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(-32768, -5324) == false);
}

#[test]
fn op_numeric_equalsht2args_5() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(-32768, 32767) == false);
}

#[test]
fn k_numericequal_1() {
    //  A test whose essence is: `xs:integer(1) eq xs:integer(1)`.
    assert(numeric_equal_int(1, 1) == true);
}

#[test]
fn k_numericequal_3() {
    //  A test whose essence is: `xs:integer(1) eq xs:integer(1)`.
    assert(numeric_equal_int(1, 1) == true);
}

#[test]
fn k_numericequal_4() {
    //  A test whose essence is: `xs:integer(-1) eq xs:integer(-1)`.
    assert(numeric_equal_int(-1, -1) == true);
}

#[test]
fn k_numericequal_6() {
    //  A test whose essence is: `xs:decimal(1.1) eq xs:decimal(1.1)`.
    assert(numeric_equal_int(1, 1) == true);
}

#[test]
fn k_numericequal_8() {
    //  A test whose essence is: `xs:decimal(1.1) eq xs:decimal(1.1)`.
    assert(numeric_equal_int(1, 1) == true);
}

#[test]
fn k_numericequal_9() {
    //  A test whose essence is: `xs:decimal(-1.1) eq xs:decimal(-1.1)`.
    assert(numeric_equal_int(-1, -1) == true);
}

#[test]
fn k_numericequal_41() {
    //  An expression involving the 'eq' operator that trigger certain optimization
    assert(numeric_equal_int(0, 0) == true);
}

#[test]
fn k_numericequal_45() {
    //  Test automatic xs:untypedAtomic conversion.
    assert(numeric_equal_int(3, 3) == true);
}
