//! Test chunk 1 for prod:ForClause
//! Contains 36 tests

use dep::xpath::stub_prodforclause;

#[test]
fn k2_forexprwithout_26() {
    //  Use a name test named 'let' inside a for loop, inside a user function.
    // XPath: declare function local:func($arg as element()* ) as element(
    // Expected: 1
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k2_forexprwithout_27() {
    //  Use a name test named 'declare' inside a for loop, inside a user function.
    // XPath: declare function local:func($arg as element()* ) as element(
    // Expected: 1
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k2_forexprwithout_28() {
    //  Use a name test named 'some' inside a for loop, inside a user function.
    // XPath: declare function local:func($arg as element()* ) as element(
    // Expected: 1
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k2_forexprwithout_29() {
    //  Use a name test named 'child' inside a for loop, inside a user function.
    // XPath: declare function local:func($arg as element()* ) as element(
    // Expected: 1
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k2_forexprwithout_30() {
    //  Use a name test named 'ordered' inside a for loop, inside a user function.
    // XPath: declare function local:func($arg as element()* ) as element(
    // Expected: 1
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k2_forexprwithout_31() {
    //  Use a name test named 'unordered' inside a for loop, inside a user function.
    // XPath: declare function local:func($arg as element()* ) as element(
    // Expected: 1
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k2_forexprwithout_32() {
    //  Use a name test named 'schema-attribute' inside a for loop, inside a user funct
    // XPath: declare function local:func($arg as element()* ) as element(
    // Expected: 1
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k2_forexprwithout_33() {
    //  Use a name test named 'schema-attribute' inside a for loop, inside a user funct
    // XPath: declare function local:func($arg as element()* ) as element(
    // Expected: 1
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k2_forexprwithout_34() {
    //  Use a name test named 'item' inside a for loop, inside a user function.
    // XPath: declare function local:func($arg as element()* ) as element(
    // Expected: 1
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k2_forexprwithout_35() {
    //  Use a name test named 'following-sibling' inside a for loop, inside a user func
    // XPath: declare function local:func($arg as element()* ) as element(
    // Expected: 1
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k2_forexprwithout_36() {
    //  Use a name test named 'validate' inside a for loop, inside a user function.
    // XPath: declare function local:func($arg as element()* ) as element(
    // Expected: 1
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k2_forexprwithout_37() {
    //  Use a name test named 'instance' inside a for loop, inside a user function.
    // XPath: declare function local:func($arg as element()* ) as element(
    // Expected: 1
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k2_forexprwithout_38() {
    //  Use a name test named 'castable' inside a for loop, inside a user function.
    // XPath: declare function local:func($arg as element()* ) as element(
    // Expected: 1
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k2_forexprwithout_39() {
    //  Use a name test named 'import' inside a for loop, inside a user function.
    // XPath: declare function local:func($arg as element()* ) as element(
    // Expected: 1
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k2_forexprwithout_42() {
    //  Use a computed attribute constructor with name return, inside a for loop.
    // XPath: for $n in attribute return {()} return 1
    // Expected: 1
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k2_forexprwithout_43() {
    //  Use a computed element constructor with name return, inside a for loop.
    // XPath: for $n in element return {()} return 1
    // Expected: 1
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k2_forexprwithout_44() {
    //  Use a computed processing-instruction constructor with name return, inside a fo
    // XPath: for $n in processing-instruction return {()} return 1
    // Expected: 1
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn forexprtype002() {
    //  FLWOR expressions with type declaration (attribute type)
    // XPath: for $fileName as attribute(name,xs:untypedAtomic) in (//Fold
    // Expected: File00000000000 File00000000001 File0000
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn forexprtype005() {
    //  FLWOR expressions with type declaration (simple type)
    // XPath: for $int as xs:int in (xs:int(1), xs:int(2)) return $int
    // Expected: 1 2
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn forexprtype015() {
    //  Type declaration is a super type
    // XPath: for $num as xs:decimal in (xs:integer(1), xs:integer(2), xs:
    // Expected: 1 2 3
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn forexprtype016() {
    //  Type declaration is a super type of the union type bound to the variable
    // XPath: for $num as xs:decimal in (xs:integer(1), xs:decimal(2), xs:
    // Expected: 1 2 3
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn forexprtype019() {
    //  Match a text node type
    // XPath: for $fileName as text() in (//Folder)[1]/File/FileName/text(
    // Expected: File00000000000 File00000000001 File0000
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn forexprtype020() {
    //  Match a 'item' type
    // XPath: for $fileName as item() in data( (//Folder)[1]/File/FileName
    // Expected: File00000000000 File00000000001 File0000
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn forexprtype021() {
    //  Match a 'document' type
    // XPath: for $doc as document-node() in (/) return count( $doc )
    // Expected: 1
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn forexprtype022() {
    //  FLWOR expressions with type declaration (attribute type). Simple attribute wild
    // XPath: declare default element namespace \"http://www.example.com/t
    // Expected: 10 20
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn forexprtype023() {
    //  FLWOR expressions with type declaration (attribute type). Simple attribute name
    // XPath: declare default element namespace \"http://www.example.com/t
    // Expected: 10
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn forexprtype024() {
    //  FLWOR expressions with type declaration (attribute type). anySimpleType attribu
    // XPath: declare default element namespace \"http://www.example.com/t
    // Expected: 10
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn forexprtype028() {
    //  FLWOR expressions with type declaration. Value subtype promotion: Numeric based
    // XPath: for $test as xs:integer in ( xs:byte(\"1\"), xs:long(\"10000
    // Expected: 1 10000 -10 100 100 100
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn forexprtype029() {
    //  FLWOR expressions with type declaration. Value subtype promotion: Numeric based
    // XPath: for $test as xs:decimal in ( xs:integer(\"100\"), xs:short(\
    // Expected: 100 1 10000 -10 100 100 100
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn forexprtype030() {
    //  FLWOR expressions with type declaration. Value subtype promotion: String based
    // XPath: for $test as xs:normalizedString in ( xs:language(\"en-US\")
    // Expected: en-US foobar token1 entity1
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn forexprtype031() {
    //  FLWOR expressions with type declaration. Value subtype promotion: Time based ty
    // XPath: for $test as xs:anyAtomicType in ( xs:duration(\"P0Y1347M0D\
    // Expected: P112Y3M 1999-05-31T13:20:00-05:00 13:00:
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn forexprtype032() {
    //  FLWOR expressions with type declaration. Value based subtype promotion: 2nd lev
    // XPath: for $test as xs:anyAtomicType in ( xs:boolean(\"true\"), xs:
    // Expected: true abcd 1234 5.7 7.5 http://foo
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn forexprtype033() {
    //  FLWOR expressions with type declaration. Value based subtype promotion: Types f
    // XPath: for $test as xs:anyAtomicType in ( xs:unsignedByte(\"10\"),
    // Expected: 10 20 30 -40 5.5 entity1 foobar en-US fo
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn forexprtype034() {
    //  FLWOR expressions with type declaration. Value based subtype promotion: numeric
    // XPath: for $test as xs:decimal in ( 1, 0.3 ) return $test
    // Expected: 1 0.3
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn forexprtype035() {
    //  FLWOR expressions with type declaration. Value based subtype promotion: string
    // XPath: for $test as xs:anyAtomicType in ( 1, \"string\", 1e2, 0.3 )
    // Expected: 1 string 100 0.3
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn forexprtype054() {
    //  FLWOR expressions with type declaration. Type declaration matching item() value
    // XPath: declare namespace fs=\"http://www.example.com/filesystem\";
    // Expected: File00000000000 File00000000001 File0000
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}
