//! Test chunk 1 for prod:CastExpr.derived
//! Contains 34 tests

use dep::xpath::stub_prodcastexpr_derived;

#[test]
fn cast_within_9() {
    //  Casting from integer to unsignedInt.
    // XPath: let $value := xs:integer(10.0) return $value cast as xs:unsi
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_within_10() {
    //  Casting from integer to byte.
    // XPath: let $value := xs:integer(10.0) return $value cast as xs:byte
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_within_11() {
    //  Casting from integer to unsignedShort.
    // XPath: let $value := xs:integer(10.0) return $value cast as xs:unsi
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_within_12() {
    //  Casting from integer to an unsignedByte.
    // XPath: let $value := xs:integer(10.0) return $value cast as xs:unsi
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_within_13() {
    //  Casting from a long to an integer.
    // XPath: let $value := xs:long(10.0) return $value cast as xs:integer
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_within_14() {
    //  Casting from long to a nonNegativeInteger.
    // XPath: let $value := xs:long(10.0) return $value cast as xs:nonNega
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_within_15() {
    //  Casting from long to a negativeInteger.
    // XPath: let $value := xs:long(-10) return $value cast as xs:negative
    // Expected: -10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_within_16() {
    //  Casting from long to an int.
    // XPath: let $value := xs:long(10) return $value cast as xs:int
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_within_17() {
    //  Casting from long to an unsignedLong.
    // XPath: let $value := xs:long(10) return $value cast as xs:unsignedL
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_within_18() {
    //  Casting from long to a positiveInteger.
    // XPath: let $value := xs:long(10) return $value cast as xs:positiveI
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_within_19() {
    //  Casting from long to a short.
    // XPath: let $value := xs:long(10) return $value cast as xs:short
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_within_20() {
    //  Casting from long to unsignedInt.
    // XPath: let $value := xs:long(10) return $value cast as xs:unsignedI
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_within_21() {
    //  Casting from long to a byte.
    // XPath: let $value := xs:long(10) return $value cast as xs:byte
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cbcl_cast_entity_001() {
    //  test cast to xs:ENTITY from a subtype of xs:NCName
    // XPath: xs:NCName('entity') cast as xs:ENTITY
    // Expected: entity
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cbcl_cast_id_001() {
    //  test cast to xs:ID from a subtype of xs:NCName
    // XPath: xs:NCName('id') cast as xs:ID
    // Expected: id
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cbcl_cast_idref_001() {
    //  test cast to xs:IDREF from a subtype of xs:NCName
    // XPath: xs:NCName('idref') cast as xs:IDREF
    // Expected: idref
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cbcl_cast_language_001() {
    //  test cast to xs:language
    // XPath: xs:language('en-gb') cast as xs:language
    // Expected: en-gb
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cbcl_cast_name_001() {
    //  test cast to xs:Name from a subtype of xs:Name
    // XPath: xs:NCName('NCName') cast as xs:Name
    // Expected: NCName
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cbcl_cast_ncname_001() {
    //  test cast to xs:NCName from a subtype of xs:NCName
    // XPath: xs:ID('id') cast as xs:NCName
    // Expected: id
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cbcl_cast_nmtoken_001() {
    //  test cast to xs:NMTOKEN
    // XPath: let $string := (\"&#xD;&#xA;&#x9; foobar &#xA;&#xD;&#x9;\" c
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cbcl_cast_nmtoken_001b() {
    //  test cast to xs:NMTOKEN
    // XPath: for $string in (\"    foobar    \" cast as xs:NMTOKEN) retur
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cbcl_cast_nmtoken_002() {
    //  test cast to xs:NMTOKEN from a subtype of xs:NMTOKEN
    // XPath: xs:NMTOKEN('NMTOKEN') cast as xs:NMTOKEN
    // Expected: NMTOKEN
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cbcl_cast_normalizedstring_001() {
    //  test cast to xs:normalizedString
    // XPath: for $string in (\"&#xD; foo &#x9; bar &#xA;\" cast as xs:nor
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cbcl_cast_normalizedstring_001b() {
    //  test cast to xs:normalizedString
    // XPath: for $string in (\"  foo   bar  \" cast as xs:normalizedStrin
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cbcl_cast_token_001() {
    //  test cast to xs:token
    // XPath: for $string in (\"&#xD; foo &#x9; bar &#xA;\" cast as xs:tok
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cbcl_cast_token_001b() {
    //  test cast to xs:token
    // XPath: for $string in (\"  foo   bar  \" cast as xs:token)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cbcl_cast_nonnegativeinteger_003() {
    //  Ensure that '-00' is a valid lexical value for xs:nonNegativeInteger
    // XPath: xs:nonNegativeInteger(\"-00\")
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cbcl_cast_nonpositiveinteger_003() {
    //  Ensure that '+00' is a valid lexical value for xs:nonPositiveInteger
    // XPath: xs:nonPositiveInteger(\"+00\")
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cbcl_cast_unsignedbyte_005() {
    //  Ensure that '-00' is a valid lexical value for xs:unsignedByte
    // XPath: xs:unsignedByte(\"-00\")
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cbcl_cast_unsignedint_005() {
    //  Ensure that '-00' is a valid lexical value for xs:unsignedInt
    // XPath: xs:unsignedInt(\"-00\")
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cbcl_cast_unsignedlong_005() {
    //  Ensure that '-00' is a valid lexical value for xs:unsignedLong
    // XPath: xs:unsignedLong(\"-00\")
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cbcl_cast_unsignedshort_005() {
    //  Ensure that '-00' is a valid lexical value for xs:unsignedShort
    // XPath: xs:unsignedShort(\"-00\")
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cbcl_cast_gyearmonth_004() {
    //  test cast year 0000 xs:gYearMonth
    // XPath: \"-0000-05\" cast as xs:gYearMonth
    // Expected: 0000-05
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cbcl_case_anyuri_001() {
    //  Test a uri with an empty path, but with a valid query
    // XPath: \"http://example.com?query=\" cast as xs:anyURI
    // Expected: http://example.com?query=
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}
