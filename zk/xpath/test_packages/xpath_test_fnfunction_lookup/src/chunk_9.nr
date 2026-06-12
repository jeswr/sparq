//! Test chunk 9 for fn:function-lookup
//! Contains 50 tests

use dep::xpath::stub_fnfunction_lookup;

#[test]
fn fn_function_lookup_505() {
    // Attempts to look up function xs:ID.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2001/XMLS
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_506() {
    // Attempts to invoke function xs:ID.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2001/XMLSchema',
    // Expected: ID
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_507() {
    // Attempts to look up function xs:IDREF.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2001/XMLS
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_508() {
    // Attempts to invoke function xs:IDREF.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2001/XMLSchema',
    // Expected: IDREF
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_509() {
    // Attempts to look up function xs:ENTITY.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2001/XMLS
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_510() {
    // Attempts to invoke function xs:ENTITY.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2001/XMLSchema',
    // Expected: ENTITY
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_511() {
    // Attempts to look up function xs:boolean.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2001/XMLS
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_512() {
    // Attempts to invoke function xs:boolean.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2001/XMLSchema',
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_513() {
    // Attempts to look up function xs:base64Binary.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2001/XMLS
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_514() {
    // Attempts to invoke function xs:base64Binary.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2001/XMLSchema',
    // Expected: D74D35D35D35
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_515() {
    // Attempts to look up function xs:hexBinary.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2001/XMLS
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_516() {
    // Attempts to invoke function xs:hexBinary.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2001/XMLSchema',
    // Expected: 0FB7
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_517() {
    // Attempts to look up function xs:anyURI.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2001/XMLS
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_518() {
    // Attempts to invoke function xs:anyURI.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2001/XMLSchema',
    // Expected: http://www.example.org/
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_519() {
    // Attempts to look up function xs:QName.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2001/XMLS
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_520() {
    // Attempts to invoke function xs:QName.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2001/XMLSchema',
    // Expected: fn:QName
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_522() {
    // Attempts to invoke function xs:dateTimeStamp.
    // XPath: let $f := function-lookup(fn:QName('http://www.w3.org/2001/X
    // Expected: 2012
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_523() {
    // Attempts to look up function xs:IDREFS.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2001/XMLS
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_524() {
    // Attempts to invoke function xs:IDREFS.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2001/XMLSchema',
    // Expected: ID1 ID2 ID3
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_525() {
    // Attempts to look up function xs:NMTOKENS.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2001/XMLS
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_526() {
    // Attempts to invoke function xs:NMTOKENS.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2001/XMLSchema',
    // Expected: NMTOKEN1 NMTOKEN2 NMTOKEN3
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_527() {
    // Attempts to look up function xs:ENTITES.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2001/XMLS
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_528() {
    // Attempts to invoke function xs:ENTITIES.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2001/XMLSchema',
    // Expected: ENTITY1 ENTITY2 ENTITY3
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_529() {
    // Attempts to look up non-existant function local:missing.
    // XPath: empty(function-lookup(fn:QName('http://www.w3.org/2005/xquer
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_601() {
    // Applied to a built-in function
    // XPath: function-lookup(QName(\"http://www.w3.org/2005/xpath-functio
    // Expected: 3
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_602() {
    // Applied to a built-in function with variable arity
    // XPath: function-lookup(QName(\"http://www.w3.org/2005/xpath-functio
    // Expected: abc
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_603() {
    // Applied to a built-in function with zero arity
    // XPath: function-lookup(QName(\"http://www.w3.org/2005/xpath-functio
    // Expected: 3
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_604() {
    // Applied to a constructor function
    // XPath: function-lookup(QName(\"http://www.w3.org/2001/XMLSchema\",
    // Expected: xs:time("12:30:00Z")
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_605() {
    // Applied to a user-defined function
    // XPath: xquery version \"3.0\";        declare function local:square
    // Expected: 169
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_606() {
    // Applied to a private user-defined function in the same module
    // XPath: xquery version \"3.0\";        declare %private function loc
    // Expected: 169
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_609() {
    // Get the name and arity of the result
    // XPath: function-lookup(QName(\"http://www.w3.org/2005/xpath-functio
    // Expected: http://www.w3.org/2005/xpath-functions r
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_612() {
    // Curry the resulting function
    // XPath: function-lookup(QName(\"http://www.w3.org/2005/xpath-functio
    // Expected: 1.235
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_720() {
    // Attempts to look up function fn:parse-ietf-date.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_721() {
    // Attempts to invoke function fn:parse-ietf-date.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: xs:dateTime('2014-08-20T19:36:01Z')
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_724() {
    // Attempts to look up function fn:apply#2.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_725() {
    // Attempts to invoke function fn:apply#2.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: 2
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_726() {
    // Attempts to look up function fn:sort#1.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_728() {
    // Attempts to look up function fn:sort#2.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_730() {
    // Attempts to look up function fn:sort#3.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_732() {
    // Attempts to look up function fn:contains-token.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_733() {
    // Attempts to invoke function fn:contains-token.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_734() {
    // Attempts to look up function fn:default-language#0.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_736() {
    // Attempts to look up function fn:json-doc#1.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_738() {
    // Attempts to look up function fn:json-doc#2.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_740() {
    // Attempts to look up function fn:json-to-xml#1.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_742() {
    // Attempts to look up function fn:json-to-xml#2.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_744() {
    // Attempts to look up function fn:parse-json#1.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_745() {
    // Attempts to invoke function fn:parse-json#1.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_746() {
    // Attempts to look up function fn:parse-json#2.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_747() {
    // Attempts to invoke function fn:json-doc#2.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}
