//! Test chunk 0 for prod:CastExpr.schema
//! Contains 50 tests

use dep::xpath::stub_prodcastexpr_schema;

#[test]
fn qname_cast_1() {
    //  Evaluates casting an xs:QName type to another xs:QName type.
    // XPath: xs:QName(\"value1\") cast as xs:QName
    // Expected: value1
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn qname_cast_2() {
    //  Evaluates casting a type derived from an xs:QName to an xs:QName type.
    // XPath: import schema namespace myType=\"http://www.w3.org/XQueryTes
    // Expected: value1
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn qname_cast_3() {
    //  Evaluates casting a type derived from an xs:QName to a type derived from xs:QNa
    // XPath: import schema namespace myType=\"http://www.w3.org/XQueryTes
    // Expected: value1
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn qname_cast_4() {
    //  Evaluates casting a xs:QName type to a type derived from xs:QName.
    // XPath: import schema namespace myType=\"http://www.w3.org/XQueryTes
    // Expected: value1
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn notation_cast_3() {
    //  Evaluates casting a type derived from an xs:NOTATION to a type derived from xs:
    // XPath: import schema namespace myType = \"http://www.w3.org/XQueryT
    // Expected: myType:value1
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds1() {
    //  Simple test for evaluation of a simple expression casted as xs:integer - XML Da
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: 12678967543233
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds2() {
    //  Simple test for evaluation of a simple expression casted as xs:float - xml data
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds3() {
    //  Simple test for evaluation of a simple expression casted as xs:boolean - XML Da
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds4() {
    //  Simple test for evaluation of a simple expression casted as xs:double - XML Dat
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: 1.2678967543233E13
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds5() {
    //  Simple test for evaluation of a simple expression casted as xs:decimal - XML Da
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: 12678967543233
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds6() {
    //  Simple test for evaluation of a simple expression casted as xs:string - XML Dat
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: 12678967543233
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds7() {
    //  Simple test for evaluation a string casted as xs:string - XML Data Source
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: A String Function
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds8() {
    //  Simple test for casting a float as a float - XML Data Source
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds10() {
    //  Simple test for casting a float as a double
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds11() {
    //  Simple test for casting a float as a boolean
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds12() {
    //  Simple test for casting a double as a double - XML Data Source
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: 1.26743233E15
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds13() {
    //  Simple test for casting a double as a string - XML Data Source
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: false
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds14() {
    //  Simple test for casting a double as a float - XML Data Source
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds15() {
    //  Simple test for casting a double as a boolean - XML Data Source
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds16() {
    //  Simple test for casting a decimal as a string - XML Data Source
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: 12678967.543233
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds17() {
    //  Simple test for casting a decimal as a float - XML Data Source
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: 1.2678968E7
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds18() {
    //  Simple test for casting a decimal as a double - XML Data Source
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: 1.2678967543233E7
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds19() {
    //  Simple test for casting a decimal as a decimal - XML Data Source
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: 12678967.543233
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds21() {
    //  Simple test for casting a decimal as a boolean - XML Data Source
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds22() {
    //  Simple test for casting an integer as a string - XML Data Source
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: 12678967543233
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds23() {
    //  Simple test for casting an integer as a float - XML Data Source
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds24() {
    //  Simple test for casting an integer as a double - XML Data Source
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: 1.2678967543233E13
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds25() {
    //  Simple test for casting an integer as a decimal - XML Data Source
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: 12678967543233
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds26() {
    //  Simple test for casting an integer as an integer - XML Data Source
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: 12678967543233
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds27() {
    //  Simple test for casting an integer as a boolean - XML Data Source
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds28() {
    //  Simple test for casting an dateTime as a string - XML Data Source
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: 2002-04-02T12:00:00Z
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds29() {
    //  Simple test for casting an dateTime as a dateTime - XML Data Source
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: 2002-04-02T12:00:00Z
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds30() {
    //  Simple test for casting an dateTime as a time - XML Data Source
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: 12:00:00Z
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds31() {
    //  Simple test for casting an dateTime as a date - XML Data Source
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: 2002-04-02Z
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds32() {
    //  Simple test for casting an date as a string - XML Data Source
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: 2000-01-01+05:00
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds33() {
    //  Simple test for casting an date as a dateTime - XML Data Source
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: 2000-01-01T00:00:00+05:00
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds34() {
    //  Simple test for casting an date as a date - XML Data Source
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: 2000-01-01+05:00
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds35() {
    //  Simple test for casting an xs:time as a string - XML Data Source
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: 13:20:10.5Z
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds36() {
    //  Simple test for casting an xs:time as an xs:time - XML Data Source
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: 13:20:10.5Z
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds37() {
    //  Simple test for casting a boolean as string - XML Data Source
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds38() {
    //  Simple test for casting a boolean as float - XML Data Source
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: 1
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds39() {
    //  Simple test for casting a boolean as double - XML Data Source
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: 1
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds40() {
    //  Simple test for casting a boolean as decimal - XML Data Source
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: 1
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds41() {
    //  Simple test for casting a boolean as integer - XML Data Source
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: 1
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn casthcds42() {
    //  Simple test for casting a boolean as boolean - XML Data Source
    // XPath: import schema namespace atomic=\"http://www.w3.org/XQueryTes
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn user_defined_1() {
    //  Evaluation of simple user defined data type restricted from xs:integer.
    // XPath: import schema namespace myType=\"http://www.w3.org/XQueryTes
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn user_defined_3() {
    //  Evaluation of constructor function used as part of addition        operation us
    // XPath: import schema namespace myType=\"http://www.w3.org/XQueryTes
    // Expected: 3
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn user_defined_4() {
    //  Evaluation of simple user defined data type restricted from xs:string.
    // XPath: import schema namespace myType=\"http://www.w3.org/XQueryTes
    // Expected: valid value 4
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn user_defined_5() {
    //  Evaluation of constructor function operation on simple user        defined data
    // XPath: import schema namespace myType=\"http://www.w3.org/XQueryTes
    // Expected: valid value 4valid value 1
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn user_defined_6() {
    //  use constructor function for user defined type and cast value        to base ty
    // XPath: import schema namespace myType=\"http://www.w3.org/XQueryTes
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}
