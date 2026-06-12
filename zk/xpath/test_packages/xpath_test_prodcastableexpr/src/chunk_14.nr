//! Test chunk 14 for prod:CastableExpr
//! Contains 48 tests

use dep::xpath::stub_prodcastableexpr;

#[test]
fn cbcl_castable_name_002() {
    //  test castable as xs:Name from a type which will always fail
    // XPath: fn:current-time() castable as xs:Name
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_name_003() {
    //  test castable as xs:Name
    // XPath: 'NCName' castable as xs:Name
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_name_004() {
    //  test castable as xs:Name
    // XPath: 'N A M E' castable as xs:Name
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_ncname_001() {
    //  test castable as xs:NCName from a subtype of xs:NCName
    // XPath: xs:ID('id') castable as xs:NCName
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_ncname_002() {
    //  test castable as xs:NCName from a type which will always fail
    // XPath: fn:current-time() castable as xs:NCName
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_ncname_003() {
    //  test castable as xs:NCName
    // XPath: 'NCName' castable as xs:NCName
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_ncname_004() {
    //  test castable as xs:NCName
    // XPath: 'NC:Name' castable as xs:NCName
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_negativeinteger_001() {
    //  test castable to xs:negativeInteger of out-of-range value
    // XPath: 0 castable as xs:negativeInteger
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_negativeinteger_002() {
    //  test castable to xs:negativeInteger of out-of-range value
    // XPath: \"0\" castable as xs:negativeInteger
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_nmtoken_001() {
    //  test castable as xs:NMTOKEN from a subtype of xs:NMTOKEN
    // XPath: xs:NMTOKEN('NMTOKEN') castable as xs:NMTOKEN
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_nmtoken_002() {
    //  test cast to xs:NMTOKEN
    // XPath: \"&#xD;&#xA;&#x9; foobar &#xA;&#xD;&#x9;\" castable as xs:NM
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_nmtoken_002b() {
    //  test cast to xs:NMTOKEN
    // XPath: \"    foobar    \" castable as xs:NMTOKEN
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_nonnegativeinteger_001() {
    //  test castable to xs:nonNegativeInteger of out-of-range value
    // XPath: -1 castable as xs:nonNegativeInteger
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_nonnegativeinteger_002() {
    //  test castable to xs:nonNegativeInteger of out-of-range value
    // XPath: \"-1\" castable as xs:nonNegativeInteger
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_nonpositiveinteger_001() {
    //  test castable to xs:nonPositiveInteger of out-of-range value
    // XPath: 1 castable as xs:nonPositiveInteger
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_nonpositiveinteger_002() {
    //  test castable to xs:nonPositiveInteger of out-of-range value
    // XPath: \"1\" castable as xs:nonPositiveInteger
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_positiveinteger_001() {
    //  test castable to xs:positiveInteger of out-of-range value
    // XPath: 0 castable as xs:positiveInteger
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_positiveinteger_002() {
    //  test castable to xs:positiveInteger of out-of-range value
    // XPath: \"0\" castable as xs:positiveInteger
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_short_001() {
    //  test castable to xs:short of out-of-range value
    // XPath: 32768 castable as xs:short
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_short_002() {
    //  test castable to xs:short of out-of-range value
    // XPath: -32769 castable as xs:short
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_short_003() {
    //  test castable to xs:short of out-of-range value
    // XPath: \"32769\" castable as xs:short
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_short_004() {
    //  test castable to xs:short of out-of-range value
    // XPath: \"-32769\" castable as xs:short
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_unsignedbyte_001() {
    //  test castable to xs:unsignedByte of out-of-range value
    // XPath: 256 castable as xs:unsignedByte
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_unsignedbyte_002() {
    //  test castable to xs:unsignedByte of out-of-range value
    // XPath: -1 castable as xs:unsignedByte
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_unsignedbyte_003() {
    //  test castable to xs:unsignedByte of out-of-range value
    // XPath: \"256\" castable as xs:unsignedByte
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_unsignedbyte_004() {
    //  test castable to xs:unsignedByte of out-of-range value
    // XPath: \"-1\" castable as xs:unsignedByte
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_unsignedint_001() {
    //  test castable to xs:unsignedInt of out-of-range value
    // XPath: 4294967296 castable as xs:unsignedInt
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_unsignedint_002() {
    //  test castable to xs:unsignedInt of out-of-range value
    // XPath: -1 castable as xs:unsignedInt
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_unsignedint_003() {
    //  test castable to xs:unsignedInt of out-of-range value
    // XPath: \"4294967296\" castable as xs:unsignedInt
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_unsignedint_004() {
    //  test castable to xs:unsignedInt of out-of-range value
    // XPath: \"-1\" castable as xs:unsignedInt
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_unsignedlong_002() {
    //  test castable to xs:unsignedLong of out-of-range value
    // XPath: -1 castable as xs:unsignedLong
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_unsignedlong_003() {
    //  test castable to xs:unsignedLong of out-of-range value
    // XPath: \"18446744073709551616\" castable as xs:unsignedLong
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_unsignedlong_004() {
    //  test castable to xs:unsignedLong of out-of-range value
    // XPath: \"-1\" castable as xs:unsignedLong
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_unsignedshort_001() {
    //  test castable to xs:unsignedShort of out-of-range value
    // XPath: 65536 castable as xs:unsignedShort
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_unsignedshort_002() {
    //  test castable to xs:unsignedShort of out-of-range value
    // XPath: -1 castable as xs:unsignedShort
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_unsignedshort_003() {
    //  test castable to xs:unsignedShort of out-of-range value
    // XPath: \"65536\" castable as xs:unsignedShort
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_unsignedshort_004() {
    //  test castable to xs:unsignedShort of out-of-range value
    // XPath: \"-1\" castable as xs:unsignedShort
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_yearmonthduration_001() {
    //  test castable to xs:yearMonthDuration with overflow
    // XPath: \"-P768614336404564651Y\" castable as xs:yearMonthDuration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_yearmonthduration_002() {
    //  test castable to xs:yearMonthDuration with overflow
    // XPath: \"P768614336404564651Y\" castable as xs:yearMonthDuration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_nonnegativeinteger_003() {
    //  Ensure that '-00' is a valid lexical value for xs:nonNegativeInteger
    // XPath: \"-00\" castable as xs:nonNegativeInteger
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_nonpositiveinteger_003() {
    //  Ensure that '+00' is a valid lexical value for xs:nonPositiveInteger
    // XPath: \"+00\" castable as xs:nonPositiveInteger
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_unsignedbyte_005() {
    //  Ensure that '-00' is a valid lexical value for xs:unsignedByte
    // XPath: \"-00\" castable as xs:unsignedByte
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_unsignedint_005() {
    //  Ensure that '-00' is a valid lexical value for xs:unsignedInt
    // XPath: \"-00\" castable as xs:unsignedInt
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_unsignedlong_005() {
    //  Ensure that '-00' is a valid lexical value for xs:unsignedLong
    // XPath: \"-00\" castable as xs:unsignedLong
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_unsignedshort_005() {
    //  Ensure that '-00' is a valid lexical value for xs:unsignedShort
    // XPath: \"-00\" castable as xs:unsignedShort
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_gyear_001() {
    //  test cast to xs:gYear with overflow
    // XPath: \"99999999999999999999999999999\" castable as xs:gYear
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_gyearmonth_001() {
    //  test cast to xs:gYearMonth with overflow
    // XPath: \"99999999999999999999999999999-01\" castable as xs:gYearMon
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_gyearmonth_002() {
    //  test cast to xs:gYearMonth with overflow
    // XPath: \"99999999999999999999999999999-XX\" castable as xs:gYearMon
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}
