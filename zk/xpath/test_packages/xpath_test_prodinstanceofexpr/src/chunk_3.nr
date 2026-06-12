//! Test chunk 3 for prod:InstanceofExpr
//! Contains 50 tests

use dep::xpath::stub_prodinstanceofexpr;

#[test]
fn k_seqexprinstanceof_27() {
    //  A test whose essence is: `not(1 instance of text())`.
    // XPath: not(1 instance of text())
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_28() {
    //  A test whose essence is: `not(1 instance of node())`.
    // XPath: not(1 instance of node())
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_29() {
    //  A test whose essence is: `not(1 instance of element())`.
    // XPath: not(1 instance of element())
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_30() {
    //  A test whose essence is: `not(1 instance of processing-instruction())`.
    // XPath: not(1 instance of processing-instruction())
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_31() {
    //  A test whose essence is: `not(1 instance of comment())`.
    // XPath: not(1 instance of comment())
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_32() {
    //  A comment appearing inside the paranteses of the sequence type text(). comment
    // XPath: not(1 instance of text())
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_33() {
    //  A comment appearing inside the paranteses of the sequence type node(). comment
    // XPath: not(1 instance of node())
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_34() {
    //  A comment appearing inside the paranteses of the sequence type element(). comme
    // XPath: not(1 instance of element())
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_35() {
    //  A comment appearing inside the paranteses of the sequence type processing-instr
    // XPath: not(1 instance of processing-instruction())
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_36() {
    //  A comment appearing inside the paranteses of the sequence type comment(). comme
    // XPath: not(1 instance of comment())
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_37() {
    //  A test whose essence is: `not(1 instance of empty-sequence())`.
    // XPath: not(1 instance of empty-sequence())
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_38() {
    //  A test whose essence is: `not( (1, 2, 3) instance of empty-sequence())`.
    // XPath: not( (1, 2, 3) instance of empty-sequence())
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_39() {
    //  A test whose essence is: `() instance of empty-sequence()`.
    // XPath: () instance of empty-sequence()
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_40() {
    //  Comment appearing inside the paranteses of empty-sequence(). comment
    // XPath: () instance of empty-sequence()
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_41() {
    //  A test whose essence is: `(((()))) instance of empty-sequence()`.
    // XPath: (((()))) instance of empty-sequence()
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_42() {
    //  A test whose essence is: `((), (), ()) instance of empty-sequence()`.
    // XPath: ((), (), ()) instance of empty-sequence()
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_43() {
    //  A test whose essence is: `((), 'xs:string') instance of xs:string`.
    // XPath: ((), \"xs:string\") instance of xs:string
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_44() {
    //  A test whose essence is: `('xs:string', (), 'xs:string') instance of xs:string+
    // XPath: (\"xs:string\", (), \"xs:string\") instance of xs:string+
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_45() {
    //  A test whose essence is: `(1.1, (), 1) instance of xs:decimal+`.
    // XPath: (1.1, (), 1) instance of xs:decimal+
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_46() {
    //  A test whose essence is: `not('a string' instance of xs:NOTATION)`.
    // XPath: not(\"a string\" instance of xs:NOTATION)
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_47() {
    //  A test whose essence is: `not('a string' instance of xs:QName)`.
    // XPath: not(\"a string\" instance of xs:QName)
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_48() {
    //  A test whose essence is: `1.1 instance of xs:decimal and not(1.1 instance of xs
    // XPath: 1.1 instance of xs:decimal and not(1.1 instance of xs:intege
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_64() {
    //  A test whose essence is: `not(((10)div(3)) instance of xs:integer)`.
    // XPath: not(((10)div(3)) instance of xs:integer)
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_65() {
    //  A test whose essence is: `((10)idiv(3)) instance of xs:integer`.
    // XPath: ((10)idiv(3)) instance of xs:integer
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_66() {
    //  A test whose essence is: `((10)mod(3)) instance of xs:integer`.
    // XPath: ((10)mod(3)) instance of xs:integer
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_1() {
    //  Test an element node against type element().
    // XPath: <e/> instance of element()
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_2() {
    //  Test an element node against type element(*).
    // XPath: <e/> instance of element(*)
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_3() {
    //  Test an element node against type element(*).
    // XPath: <e/> instance of element(e)
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_4() {
    //  Test an element node against type element(name).
    // XPath: <e/> instance of element(name)
    // Expected: false
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_5() {
    //  Test an element node against type attribute().
    // XPath: <e/> instance of attribute()
    // Expected: false
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_6() {
    //  Test an element node against type attribute(*).
    // XPath: <e/> instance of attribute(*)
    // Expected: false
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_7() {
    //  Test an element node against type attribute(e).
    // XPath: <e/> instance of attribute(e)
    // Expected: false
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_8() {
    //  Test an attribute node against type element(name).
    // XPath: attribute e{\"content\"} instance of attribute(e)
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_9() {
    //  Test an attribute node against type attribute().
    // XPath: attribute e{\"content\"} instance of attribute()
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_10() {
    //  Test an attribute node against type attribute(*).
    // XPath: attribute e{\"content\"} instance of attribute(*)
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_11() {
    //  Test an attribute node against type element(e).
    // XPath: attribute e{\"content\"} instance of element(e)
    // Expected: false
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_12() {
    //  Test an attribute node against type element().
    // XPath: attribute e{\"content\"} instance of element()
    // Expected: false
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_13() {
    //  Test an attribute node against type element(*).
    // XPath: attribute e{\"content\"} instance of element(*)
    // Expected: false
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_14() {
    //  Test an attribute node against type element(name).
    // XPath: attribute e{\"content\"} instance of attribute(name)
    // Expected: false
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_15() {
    //  Check the return type of xs:nonPositiveInteger in a simple way.
    // XPath: xs:nonPositiveInteger(0) instance of xs:nonPositiveInteger
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_16() {
    //  Check the return type of xs:negativeInteger in a simple way.
    // XPath: xs:negativeInteger(-1) instance of xs:negativeInteger
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_17() {
    //  Check the return type of xs:long in a simple way.
    // XPath: xs:long(0) instance of xs:long
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_18() {
    //  Check the return type of xs:int in a simple way.
    // XPath: xs:int(0) instance of xs:int
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_19() {
    //  Check the return type of xs:short in a simple way.
    // XPath: xs:short(0) instance of xs:short
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_20() {
    //  Check the return type of xs:byte in a simple way.
    // XPath: xs:byte(0) instance of xs:byte
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_21() {
    //  Check the return type of xs:nonNegativeInteger in a simple way.
    // XPath: xs:nonNegativeInteger(0) instance of xs:nonNegativeInteger
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_22() {
    //  Check the return type of xs:nonNegativeInteger in a simple way.
    // XPath: xs:nonNegativeInteger(0) instance of xs:nonNegativeInteger
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_23() {
    //  Check the return type of xs:unsignedLong in a simple way.
    // XPath: xs:unsignedLong(0) instance of xs:unsignedLong
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_24() {
    //  Check the return type of xs:unsignedInt in a simple way.
    // XPath: xs:unsignedInt(0) instance of xs:unsignedInt
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_25() {
    //  Check the return type of xs:unsignedShort in a simple way.
    // XPath: xs:unsignedShort(0) instance of xs:unsignedShort
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}
