//! Test chunk 13 for prod:CastableExpr
//! Contains 50 tests

use dep::xpath::stub_prodcastableexpr;

#[test]
fn k_seqexprcastable_20() {
    //  Casting an empty sequence to xs:QName does not work.
    // XPath: not(() castable as xs:QName)
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn k_seqexprcastable_21() {
    //  Casting an empty sequence to xs:QName? works.
    // XPath: () castable as xs:QName?
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn k_seqexprcastable_23() {
    //  A test whose essence is: `not(('one', 'two') castable as xs:string?)`.
    // XPath: not((\"one\", \"two\") castable as xs:string?)
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn k_seqexprcastable_24() {
    //  A test whose essence is: `not('string' castable as xs:boolean)`.
    // XPath: not('string' castable as xs:boolean)
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn k_seqexprcastable_25() {
    //  A test whose essence is: `'true' castable as xs:boolean`.
    // XPath: 'true' castable as xs:boolean
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn k_seqexprcastable_26() {
    //  A test whose essence is: `xs:float(1) castable as xs:float`.
    // XPath: xs:float(1) castable as xs:float
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn k_seqexprcastable_27() {
    //  A test whose essence is: `xs:float(1) castable as xs:untypedAtomic`.
    // XPath: xs:float(1) castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn k_seqexprcastable_28() {
    //  A test whose essence is: `xs:float(1) castable as xs:string`.
    // XPath: xs:float(1) castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn k_seqexprcastable_29() {
    //  A test whose essence is: `not(xs:anyURI('example.com/') castable as xs:integer)
    // XPath: not(xs:anyURI(\"example.com/\") castable as xs:integer)
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn k_seqexprcastable_30() {
    //  A test whose essence is: `not('three' castable as xs:float)`.
    // XPath: not(\"three\" castable as xs:float)
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn k_seqexprcastable_31() {
    //  A test whose essence is: `not('three' castable as xs:integer)`.
    // XPath: not(\"three\" castable as xs:integer)
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn k_seqexprcastable_32() {
    //  A test whose essence is: `not('three' castable as xs:decimal)`.
    // XPath: not(\"three\" castable as xs:decimal)
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn k_seqexprcastable_33() {
    //  A test whose essence is: `not('three' castable as xs:double)`.
    // XPath: not(\"three\" castable as xs:double)
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn k_seqexprcastable_34() {
    //  A test whose essence is: `not((1, 2, 3) castable as xs:integer)`.
    // XPath: not((1, 2, 3) castable as xs:integer)
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn k_seqexprcastable_35() {
    //  A test whose essence is: `not(() castable as xs:integer)`.
    // XPath: not(() castable as xs:integer)
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn k_seqexprcastable_36() {
    //  A test whose essence is: `() castable as xs:integer?`.
    // XPath: () castable as xs:integer?
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn k_seqexprcastable_37() {
    //  A test whose essence is: `(1) castable as xs:integer?`.
    // XPath: (1) castable as xs:integer?
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn k_seqexprcastable_38() {
    //  A test whose essence is: `not(('one', 'two') castable as xs:string?)`.
    // XPath: not((\"one\", \"two\") castable as xs:string?)
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn k2_seqexprcastable_1() {
    //  Testing castability to xs:QName where the cardinality is wrong.
    // XPath: (QName(\"http://example.com/ANamespace\", \"ncname\"), QName
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn k2_seqexprcastable_2() {
    //  Testing castability to xs:QName where the cardinality is wrong(#2).
    // XPath: (QName(\"http://example.com/ANamespace\", \"ncname\"), QName
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn k2_seqexprcastable_5() {
    //  Check casting an xs:positiveInteger value to xs:double.
    // XPath: xs:positiveInteger(\"52\") castable as xs:double
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn k2_seqexprcastable_6() {
    //  Check that an invalid xs:anyURI gets flagged as not being castable.
    // XPath: \"%\" castable as xs:anyURI
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn k2_seqexprcastable_7() {
    //  Check that an invalid xs:anyURI gets flagged as not being castable.
    // XPath: xs:untypedAtomic(\"%\") castable as xs:anyURI
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_byte_001() {
    //  test castable to xs:byte of out-of-range value
    // XPath: 128 castable as xs:byte
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_byte_002() {
    //  test castable to xs:byte of out-of-range value
    // XPath: -129 castable as xs:byte
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_byte_003() {
    //  test castable to xs:byte of out-of-range value
    // XPath: \"128\" castable as xs:byte
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_byte_004() {
    //  test castable to xs:byte of out-of-range value
    // XPath: \"-129\" castable as xs:byte
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_date_001() {
    //  test castable to xs:date with overflow
    // XPath: \"-25252734927766555-06-06\" castable as xs:date
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_date_002() {
    //  test castable to xs:date with overflow
    // XPath: \"25252734927766555-07-29\" castable as xs:date
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_datetime_001() {
    //  test castable to xs:dateTime with overflow
    // XPath: \"-25252734927766555-06-06T00:00:00Z\" castable as xs:dateTi
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_datetime_002() {
    //  test castable to xs:dateTime with overflow
    // XPath: \"25252734927766555-07-29T00:00:00Z\" castable as xs:dateTim
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_decimal_001() {
    //  test castability of xs:float('NaN') as xs:decimal
    // XPath: xs:float('NaN') castable as xs:decimal
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_decimal_002() {
    //  test castability of xs:float('-INF') as xs:decimal
    // XPath: xs:float('-INF') castable as xs:decimal
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_decimal_003() {
    //  test castability of xs:float('INF') as xs:decimal
    // XPath: xs:float('INF') castable as xs:decimal
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_decimal_004() {
    //  test castability of xs:double('NaN') as xs:double
    // XPath: xs:double('NaN') castable as xs:decimal
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_decimal_005() {
    //  test castability of xs:double('-INF') as xs:decimal
    // XPath: xs:double('-INF') castable as xs:decimal
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_decimal_006() {
    //  test castability of xs:double('INF') as xs:decimal
    // XPath: xs:double('INF') castable as xs:decimal
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_duration_001() {
    //  test castable to xs:duration with overflow
    // XPath: \"-P768614336404564651Y\" castable as xs:duration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_duration_002() {
    //  test castable to xs:duration with overflow
    // XPath: \"P768614336404564651Y\" castable as xs:duration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_int_001() {
    //  test castable to xs:int of out-of-range value
    // XPath: 2147483648 castable as xs:int
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_int_002() {
    //  test castable to xs:int of out-of-range value
    // XPath: -2147483649 castable as xs:int
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_int_003() {
    //  test castable to xs:int of out-of-range value
    // XPath: \"2147483648\" castable as xs:int
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_int_004() {
    //  test castable to xs:int of out-of-range value
    // XPath: \"-2147483649\" castable as xs:int
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_language_001() {
    //  test castable to xs:language
    // XPath: xs:language('en-gb') castable as xs:language
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_language_002() {
    //  test castable to xs:language from a type which can only fail
    // XPath: 1.0 castable as xs:language
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_language_003() {
    //  test castable to xs:language
    // XPath: 'en-gb' castable as xs:language
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_language_004() {
    //  test castable to xs:language
    // XPath: \"gobbledygook\" castable as xs:language
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_long_003() {
    //  test castable to xs:long of out-of-range value
    // XPath: \"9223372036854775808\" castable as xs:long
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_long_004() {
    //  test castable to xs:long of out-of-range value
    // XPath: \"-9223372036854775809\" castable as xs:long
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn cbcl_castable_name_001() {
    //  test castable as xs:Name from a subtype of xs:Name
    // XPath: xs:NCName('NCName') castable as xs:Name
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}
