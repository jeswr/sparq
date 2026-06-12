//! Test chunk 6 for prod:OrExpr
//! Contains 50 tests

use dep::xpath::stub_prodorexpr;

#[test]
fn op_logical_or_146() {
    //  Logical 'or' using byte values
    // XPath: xs:byte(-1) or xs:byte(0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_147() {
    //  Logical 'or' using byte values
    // XPath: xs:byte(1) or xs:byte(-1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_148() {
    //  Logical 'or' using byte values
    // XPath: xs:byte(127) or xs:byte(0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_149() {
    //  Logical 'or' using byte values
    // XPath: xs:byte(127) or xs:byte(-1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_150() {
    //  Logical 'or' using byte values
    // XPath: xs:byte(-128) or xs:byte(0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_151() {
    //  Logical 'or' using byte values
    // XPath: xs:byte(-128) or xs:byte(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_152() {
    //  Logical 'or' using byte values
    // XPath: xs:byte(127) or xs:byte(-128)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_153() {
    //  Logical 'or' using unsignedByte values
    // XPath: xs:unsignedByte(0) or xs:unsignedByte(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_154() {
    //  Logical 'or' using unsignedByte values
    // XPath: xs:unsignedByte(0) or xs:unsignedByte(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_155() {
    //  Logical 'or' using unsignedByte values
    // XPath: xs:unsignedByte(1) or xs:unsignedByte(0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_156() {
    //  Logical 'or' using unsignedByte values
    // XPath: xs:unsignedByte(1) or xs:unsignedByte(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_157() {
    //  Logical 'or' using unsignedByte values
    // XPath: xs:unsignedByte(255) or xs:unsignedByte(0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_158() {
    //  Logical 'or' using unsignedByte values
    // XPath: xs:unsignedByte(255) or xs:unsignedByte(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_159() {
    //  Logical 'or' using unsignedByte values
    // XPath: xs:unsignedByte(255) or xs:unsignedByte(255)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn k_logicexpr_1() {
    //  A test whose essence is: `not('' or 0)`.
    // XPath: not(\"\" or 0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn k_logicexpr_2() {
    //  A test whose essence is: `not(0 or 0)`.
    // XPath: not(0 or 0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn k_logicexpr_3() {
    //  A test whose essence is: `not(0 or 0)`.
    // XPath: not(0 or 0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn k_logicexpr_4() {
    //  A test whose essence is: `not('' or 0)`.
    // XPath: not(\"\" or 0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn k_logicexpr_5() {
    //  A test whose essence is: `1 or 1`.
    // XPath: 1 or 1
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn k_logicexpr_6() {
    //  A test whose essence is: `1 or 0`.
    // XPath: 1 or 0
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn k_logicexpr_7() {
    //  A test whose essence is: `1 or 5`.
    // XPath: 1 or 5
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn k_logicexpr_8() {
    //  Non-empty xs:anyURIs in the left branch of an or-expression has an EBV value of
    // XPath: xs:anyURI(\"example.com/\") or 0
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn k_logicexpr_9() {
    //  Non-empty xs:anyURIs in the right branch of an or-expression has an EBV value o
    // XPath: 0 or xs:anyURI(\"example.com/\")
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn k_logicexpr_10() {
    //  Non-empty xs:anyURIs in the left branch of an or-expression has an EBV value of
    // XPath: not(xs:anyURI(\"\") or 0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn k_logicexpr_11() {
    //  Non-empty xs:anyURIs in the right branch of an or-expression has an EBV value o
    // XPath: not(0 or xs:anyURI(\"\"))
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn k_logicexpr_12() {
    //  Non-empty xs:untypedAtomics in the left branch of an or-expression has an EBV v
    // XPath: xs:untypedAtomic(\"a string\") or 0
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn k_logicexpr_13() {
    //  Non-empty xs:untypedAtomics in the right branch of an or-expression has an EBV
    // XPath: 0 or xs:untypedAtomic(\"a string\")
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn k_logicexpr_14() {
    //  A test whose essence is: `true() or (0, current-time())[1] treat as xs:integer`
    // XPath: true() or (0, current-time())[1] treat as xs:integer
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn k_logicexpr_15() {
    //  A test whose essence is: `(0, current-time())[1] treat as xs:integer or true()`
    // XPath: (0, current-time())[1] treat as xs:integer or true()
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn k_logicexpr_22() {
    //  An and-test applied on fn:count().
    // XPath: count((1, 2, 3, timezone-from-time(current-time()), 4)) or f
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn k_logicexpr_23() {
    //  An or-test applied on fn:count().
    // XPath: false() or count((1, 2, 3, timezone-from-time(current-time()
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn k_logicexpr_24() {
    //  An or-test applied on fn:count().
    // XPath: count((1, 2, 3, timezone-from-time(current-time()), 4)) or c
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn k_logicexpr_25() {
    //  or expression combined with fn:boolean().
    // XPath: boolean((1, 2, 3, current-time())[1] treat as xs:integer) or
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn k_logicexpr_26() {
    //  and expression combined or fn:boolean().
    // XPath: boolean((1, 2, 3, current-time())[1] treat as xs:integer) an
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn k_logicexpr_27() {
    //  or expression combined with fn:boolean().
    // XPath: true() or boolean((1, 2, 3, current-time())[1] treat as xs:i
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn k_logicexpr_28() {
    //  A test whose essence is: `not('' and 0)`.
    // XPath: not(\"\" and 0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn k_logicexpr_29() {
    //  A test whose essence is: `1 and 1`.
    // XPath: 1 and 1
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn k_logicexpr_30() {
    //  A test whose essence is: `10.3 and 'non-zero length string'`.
    // XPath: 10.3 and \"non-zero length string\"
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn k_logicexpr_31() {
    //  A test whose essence is: `1 and 5`.
    // XPath: 1 and 5
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn k_logicexpr_32() {
    //  A test whose essence is: `not((1, current-time())[1] treat as xs:integer and fa
    // XPath: not((1, current-time())[1] treat as xs:integer and false())
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn k_logicexpr_33() {
    //  A test whose essence is: `not(false() and (1, current-time())[1] treat as xs:in
    // XPath: not(false() and (1, current-time())[1] treat as xs:integer)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn k_logicexpr_34() {
    //  Non-empty xs:anyURIs have an EBV value of true.
    // XPath: xs:anyURI(\"example.com/\") and xs:anyURI(\"example.com/\")
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn k_logicexpr_35() {
    //  Empty xs:anyURIs have an EBV value of false.
    // XPath: not(xs:anyURI(\"example.com/\") and xs:anyURI(\"\"))
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn k_logicexpr_36() {
    //  Empty xs:anyURIs have an EBV value of false.
    // XPath: not(xs:anyURI(\"\") and xs:anyURI(\"example.com/\"))
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn k_logicexpr_37() {
    //  Non-empty xs:untypedAtomics have an EBV value of true.
    // XPath: xs:untypedAtomic(\"a string\") and xs:untypedAtomic(\"a stri
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn k_logicexpr_38() {
    //  Empty xs:untypedAtomics have an EBV value of false.
    // XPath: not(xs:untypedAtomic(\"a string\") and xs:untypedAtomic(\"\"
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn k_logicexpr_39() {
    //  An and-test applied on fn:count().
    // XPath: count((1, 2, 3, timezone-from-time(current-time()), 4)) and
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn k_logicexpr_40() {
    //  An and-test applied on fn:count().
    // XPath: true() and count((1, 2, 3, timezone-from-time(current-time()
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn k_logicexpr_41() {
    //  An and-test applied on fn:count().
    // XPath: count((1, 2, 3, timezone-from-time(current-time()), 4)) and
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn k_logicexpr_42() {
    //  and-expression combined with fn:boolean().
    // XPath: boolean((1, 2, 3, current-time())[1] treat as xs:integer) an
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}
