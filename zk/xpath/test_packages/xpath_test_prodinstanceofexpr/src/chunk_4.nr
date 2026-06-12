//! Test chunk 4 for prod:InstanceofExpr
//! Contains 50 tests

use dep::xpath::stub_prodinstanceofexpr;

#[test]
fn k2_seqexprinstanceof_26() {
    //  Check the return type of xs:unsignedByte in a simple way.
    // XPath: xs:unsignedByte(0) instance of xs:unsignedByte
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_27() {
    //  Check the return type of xs:positiveInteger in a simple way.
    // XPath: xs:positiveInteger(1) instance of xs:positiveInteger
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_28() {
    //  Check the return type of xs:normalizedString in a simple way.
    // XPath: xs:normalizedString(\"NCName\") instance of xs:normalizedStr
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_29() {
    //  Check the return type of xs:token in a simple way.
    // XPath: xs:token(\"NCName\") instance of xs:token
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_30() {
    //  Check the return type of xs:language in a simple way.
    // XPath: xs:language(\"NCName\") instance of xs:language
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_31() {
    //  Check the return type of xs:NMTOKEN in a simple way.
    // XPath: xs:NMTOKEN(\"NCName\") instance of xs:NMTOKEN
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_32() {
    //  Check the return type of xs:Name in a simple way.
    // XPath: xs:Name(\"NCName\") instance of xs:Name
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_33() {
    //  Check the return type of xs:NCName in a simple way.
    // XPath: xs:NCName(\"NCName\") instance of xs:NCName
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_34() {
    //  Check the return type of xs:ID in a simple way.
    // XPath: xs:ID(\"NCName\") instance of xs:ID
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_35() {
    //  Check the return type of xs:IDREF in a simple way.
    // XPath: xs:IDREF(\"NCName\") instance of xs:IDREF
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_36() {
    //  Check the return type of xs:ENTITY in a simple way.
    // XPath: xs:ENTITY(\"NCName\") instance of xs:ENTITY
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_37() {
    //  Check that xs:normalizedString has the correct parent type.
    // XPath: xs:normalizedString(\"ncname\") instance of xs:string
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_38() {
    //  Check that xs:token has the correct parent type.
    // XPath: xs:token(\"ncname\") instance of xs:normalizedString
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_39() {
    //  Check that xs:language has the correct parent type.
    // XPath: xs:language(\"ncname\") instance of xs:token
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_40() {
    //  Check that xs:NMTOKEN has the correct parent type.
    // XPath: xs:NMTOKEN(\"ncname\") instance of xs:token
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_41() {
    //  Check that xs:Name has the correct parent type.
    // XPath: xs:Name(\"ncname\") instance of xs:token
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_42() {
    //  Check that xs:NCName has the correct parent type.
    // XPath: xs:NCName(\"ncname\") instance of xs:Name
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_43() {
    //  Check that xs:ID has the correct parent type.
    // XPath: xs:ID(\"ncname\") instance of xs:NCName
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_44() {
    //  Check that xs:IDREF has the correct parent type.
    // XPath: xs:IDREF(\"ncname\") instance of xs:NCName
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_45() {
    //  Check that xs:ENTITY has the correct parent type.
    // XPath: xs:ENTITY(\"ncname\") instance of xs:NCName
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_46() {
    //  Check that xs:nonPositiveInteger has the correct parent type.
    // XPath: xs:nonPositiveInteger(\"0\") instance of xs:integer
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_47() {
    //  Check that xs:negativeInteger has the correct parent type.
    // XPath: xs:negativeInteger(\"-1\") instance of xs:nonPositiveInteger
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_48() {
    //  Check that xs:long has the correct parent type.
    // XPath: xs:long(\"0\") instance of xs:integer
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_49() {
    //  Check that xs:int has the correct parent type.
    // XPath: xs:int(\"0\") instance of xs:long
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_50() {
    //  Check that xs:short has the correct parent type.
    // XPath: xs:short(\"0\") instance of xs:int
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_51() {
    //  Check that xs:byte has the correct parent type.
    // XPath: xs:byte(\"0\") instance of xs:short
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_52() {
    //  Check that xs:nonNegativeInteger has the correct parent type.
    // XPath: xs:nonNegativeInteger(\"0\") instance of xs:integer
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_53() {
    //  Check that xs:unsignedLong has the correct parent type.
    // XPath: xs:unsignedLong(\"0\") instance of xs:nonNegativeInteger
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_54() {
    //  Check that xs:unsignedInt has the correct parent type.
    // XPath: xs:unsignedInt(\"0\") instance of xs:unsignedLong
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_55() {
    //  Check that xs:unsignedShort has the correct parent type.
    // XPath: xs:unsignedShort(\"0\") instance of xs:unsignedInt
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_56() {
    //  Check that xs:unsignedByte has the correct parent type.
    // XPath: xs:unsignedByte(\"0\") instance of xs:unsignedShort
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_57() {
    //  Check that xs:positiveInteger has the correct parent type.
    // XPath: xs:positiveInteger(\"1\") instance of xs:nonNegativeInteger
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_58() {
    //  Check that xs:nonNegativeInteger is not a child of nonPositiveInteger.
    // XPath: xs:nonNegativeInteger(\"1\") instance of xs:nonPositiveInteg
    // Expected: false
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_59() {
    //  Check that xs:nonNegativeInteger is not a child of negativeInteger.
    // XPath: xs:nonNegativeInteger(\"1\") instance of xs:negativeInteger
    // Expected: false
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_60() {
    //  Check that xs:positiveInteger is not a child of negativeInteger.
    // XPath: xs:positiveInteger(\"1\") instance of xs:negativeInteger
    // Expected: false
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_61() {
    //  Check that xs:positiveInteger is not a child of nonPositiveInteger.
    // XPath: xs:positiveInteger(\"1\") instance of xs:nonPositiveInteger
    // Expected: false
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_62() {
    //  Check that xs:long is not a child of negativeInteger.
    // XPath: xs:long(\"1\") instance of xs:negativeInteger
    // Expected: false
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_63() {
    //  Check that xs:long is not a child of nonPositiveInteger.
    // XPath: xs:long(\"1\") instance of xs:nonPositiveInteger
    // Expected: false
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_64() {
    //  Check that xs:long is not a child of nonNegativeInteger.
    // XPath: xs:long(\"1\") instance of xs:nonNegativeInteger
    // Expected: false
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_65() {
    //  Check that xs:long is not a child of positiveInteger.
    // XPath: xs:long(\"1\") instance of xs:positiveInteger
    // Expected: false
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_66() {
    //  Check that xs:token is not a child of xs:NCName.
    // XPath: xs:token(\"ncname\") instance of xs:NCName
    // Expected: false
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_67() {
    //  Check that xs:ID is not a child of xs:NCName.
    // XPath: xs:ID(\"ncname\") instance of xs:IDREF
    // Expected: false
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_68() {
    //  Check that xs:ENTITY is not a child of xs:NCName.
    // XPath: xs:ENTITY(\"ncname\") instance of xs:IDREF
    // Expected: false
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_69() {
    //  Check that xs:language is not a child of xs:NMTOKEN.
    // XPath: xs:language(\"ncname\") instance of xs:NMTOKEN
    // Expected: false
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_70() {
    //  Check that xs:Name is not a child of xs:language.
    // XPath: xs:Name(\"ncname\") instance of xs:language
    // Expected: false
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_71() {
    //  Check that xs:normalizedString is not a child of xs:token.
    // XPath: xs:normalizedString(\"ncname\") instance of xs:token
    // Expected: false
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_72() {
    //  Ensure a name test with the attribute axis gets the correct type.
    // XPath: <e a=\"\"/>/attribute::a instance of attribute(a)
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_73() {
    //  Ensure a name test with the abbreviated attribute axis gets the correct type.
    // XPath: <e a=\"\"/>/@a instance of attribute(a)
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_74() {
    //  Ensure that a cast to xs:byte is of the right type, when casting from xs:double
    // XPath: xs:byte(xs:double(\"1\")) instance of xs:byte
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k2_seqexprinstanceof_75() {
    //  Ensure that a cast to xs:nonPositiveInteger is of the right type, when casting
    // XPath: xs:nonPositiveInteger(xs:double(\"0\")) instance of xs:nonPo
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}
