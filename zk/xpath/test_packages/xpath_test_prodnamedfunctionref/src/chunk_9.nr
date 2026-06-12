//! Test chunk 9 for prod:NamedFunctionRef
//! Contains 34 tests

use dep::xpath::stub_prodnamedfunctionref;

#[test]
fn function_literal_501() {
    // Attempts to look up function xs:Name.
    // XPath: exists(Q{http://www.w3.org/2001/XMLSchema}Name#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_502() {
    // Attempts to invoke function xs:Name.
    // XPath: Q{http://www.w3.org/2001/XMLSchema}Name#1('Name')
    // Expected: Name
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_503() {
    // Attempts to look up function xs:NCName.
    // XPath: exists(Q{http://www.w3.org/2001/XMLSchema}NCName#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_504() {
    // Attempts to invoke function xs:NCName.
    // XPath: Q{http://www.w3.org/2001/XMLSchema}NCName#1('NCName')
    // Expected: NCName
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_505() {
    // Attempts to look up function xs:ID.
    // XPath: exists(Q{http://www.w3.org/2001/XMLSchema}ID#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_506() {
    // Attempts to invoke function xs:ID.
    // XPath: Q{http://www.w3.org/2001/XMLSchema}ID#1('ID')
    // Expected: ID
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_507() {
    // Attempts to look up function xs:IDREF.
    // XPath: exists(Q{http://www.w3.org/2001/XMLSchema}IDREF#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_508() {
    // Attempts to invoke function xs:IDREF.
    // XPath: Q{http://www.w3.org/2001/XMLSchema}IDREF#1('IDREF')
    // Expected: IDREF
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_509() {
    // Attempts to look up function xs:ENTITY.
    // XPath: exists(Q{http://www.w3.org/2001/XMLSchema}ENTITY#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_510() {
    // Attempts to invoke function xs:ENTITY.
    // XPath: Q{http://www.w3.org/2001/XMLSchema}ENTITY#1('ENTITY')
    // Expected: ENTITY
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_511() {
    // Attempts to look up function xs:boolean.
    // XPath: exists(Q{http://www.w3.org/2001/XMLSchema}boolean#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_512() {
    // Attempts to invoke function xs:boolean.
    // XPath: Q{http://www.w3.org/2001/XMLSchema}boolean#1('1')
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_513() {
    // Attempts to look up function xs:base64Binary.
    // XPath: exists(Q{http://www.w3.org/2001/XMLSchema}base64Binary#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_514() {
    // Attempts to invoke function xs:base64Binary.
    // XPath: Q{http://www.w3.org/2001/XMLSchema}base64Binary#1('D74D35D35
    // Expected: D74D35D35D35
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_515() {
    // Attempts to look up function xs:hexBinary.
    // XPath: exists(Q{http://www.w3.org/2001/XMLSchema}hexBinary#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_516() {
    // Attempts to invoke function xs:hexBinary.
    // XPath: Q{http://www.w3.org/2001/XMLSchema}hexBinary#1('0fb7')
    // Expected: 0FB7
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_517() {
    // Attempts to look up function xs:anyURI.
    // XPath: exists(Q{http://www.w3.org/2001/XMLSchema}anyURI#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_518() {
    // Attempts to invoke function xs:anyURI.
    // XPath: Q{http://www.w3.org/2001/XMLSchema}anyURI#1('http://www.exam
    // Expected: http://www.example.org/
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_519() {
    // Attempts to look up function xs:QName.
    // XPath: exists(Q{http://www.w3.org/2001/XMLSchema}QName#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_520() {
    // Attempts to invoke function xs:QName.
    // XPath: Q{http://www.w3.org/2001/XMLSchema}QName#1('fn:QName')
    // Expected: fn:QName
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_523() {
    // Attempts to look up function xs:IDREFS.
    // XPath: exists(Q{http://www.w3.org/2001/XMLSchema}IDREFS#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_524() {
    // Attempts to invoke function xs:IDREFS.
    // XPath: Q{http://www.w3.org/2001/XMLSchema}IDREFS#1('ID1 ID2 ID3')
    // Expected: ID1 ID2 ID3
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_525() {
    // Attempts to look up function xs:NMTOKENS.
    // XPath: exists(Q{http://www.w3.org/2001/XMLSchema}NMTOKENS#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_526() {
    // Attempts to invoke function xs:NMTOKENS.
    // XPath: Q{http://www.w3.org/2001/XMLSchema}NMTOKENS#1('NMTOKEN1 NMTO
    // Expected: NMTOKEN1 NMTOKEN2 NMTOKEN3
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_527() {
    // Attempts to look up function xs:ENTITES.
    // XPath: exists(Q{http://www.w3.org/2001/XMLSchema}ENTITIES#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_528() {
    // Attempts to invoke function xs:ENTITIES.
    // XPath: Q{http://www.w3.org/2001/XMLSchema}ENTITIES#1('ENTITY1 ENTIT
    // Expected: ENTITY1 ENTITY2 ENTITY3
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_701() {
    // Attempts to look up user-defined function.
    // XPath: declare function local:plus($x, $y) {           $x + $y
    // Expected: QName("http://www.w3.org/2005/xquery-loc
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_702() {
    // Attempts to look up user-defined function.
    // XPath: declare function local:plus($x, $y) {           $x + $y
    // Expected: 2
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_703() {
    // Attempts to invoke user-defined function.
    // XPath: declare function local:plus($x, $y) {           $x + $y
    // Expected: 12
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_704() {
    // Attempts to look up user-defined function, forwards-reference.
    // XPath: declare variable $v := function-name(local:plus#2);
    // Expected: QName("http://www.w3.org/2005/xquery-loc
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_705() {
    // Attempts to look up user-defined function, forwards reference.
    // XPath: declare variable $v := function-arity(local:plus#2);
    // Expected: 2
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_706() {
    // Attempts to invoke user-defined function, forwards reference.
    // XPath: declare variable $f := (if (current-date() gt xs:date('1900-
    // Expected: 12
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_707() {
    // Convoluted variable initialization.
    // XPath: declare variable $f := (if (current-date() gt xs:date('1900-
    // Expected: 9
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_708() {
    // Attempts to invoke user-defined function, default namespace.
    // XPath: declare default function namespace \"http://example.com/func
    // Expected: 12
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}
