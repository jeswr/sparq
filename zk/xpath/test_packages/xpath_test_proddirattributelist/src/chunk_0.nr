//! Test chunk 0 for prod:DirAttributeList
//! Contains 20 tests

use dep::xpath::stub_proddirattributelist;

#[test]
fn constr_attr_parent_1() {
    //  Attribute parent element
    // XPath: for $x in <elem attr=\"value\"/> return $x is $x/@attr/..
    // Expected: true
    // Calls stub function - will fail until prod:DirAttributeList is implemented
    let _ = stub_proddirattributelist();
}

#[test]
fn constr_attr_nsdecl_1() {
    //  namespace declaration does not count as attribute
    // XPath: fn:count((<elem xmlns:foo=\"http://ns.example.com/uri\"/>)/@
    // Expected: 0
    // Calls stub function - will fail until prod:DirAttributeList is implemented
    let _ = stub_proddirattributelist();
}

#[test]
fn constr_attr_nsdecl_2() {
    //  namespace default declaration does not count as attribute
    // XPath: fn:count((<elem xmlns=\"http://ns.example.com/uri\"/>)/@*)
    // Expected: 0
    // Calls stub function - will fail until prod:DirAttributeList is implemented
    let _ = stub_proddirattributelist();
}

#[test]
fn constr_attr_entref_2() {
    //  entity references in attribute content
    // XPath: fn:string-length(string((<elem attr=\"&amp;&lt;&gt;\"/>)/@at
    // Expected: 3
    // Calls stub function - will fail until prod:DirAttributeList is implemented
    let _ = stub_proddirattributelist();
}

#[test]
fn k2_directconelemattr_5() {
    //  Whitespace is allowed in the end tag after the QName.
    // XPath: <ncname>content</ncname > = 'content'
    // Expected: true
    // Calls stub function - will fail until prod:DirAttributeList is implemented
    let _ = stub_proddirattributelist();
}

#[test]
fn k2_directconelemattr_29() {
    //  Attribute containing two quotes.
    // XPath: string(<foo attr=\"\"\"\"/>/@attr)
    // Expected: "
    // Calls stub function - will fail until prod:DirAttributeList is implemented
    let _ = stub_proddirattributelist();
}

#[test]
fn k2_directconelemattr_30() {
    //  Attribute containing two quotes(#2).
    // XPath: string(<foo attr='\"\"'/>/@attr)
    // Expected: ""
    // Calls stub function - will fail until prod:DirAttributeList is implemented
    let _ = stub_proddirattributelist();
}

#[test]
fn k2_directconelemattr_31() {
    //  Attribute containing two apostrophes.
    // XPath: string(<foo attr=\"''\"/>/@attr)
    // Expected: ''
    // Calls stub function - will fail until prod:DirAttributeList is implemented
    let _ = stub_proddirattributelist();
}

#[test]
fn k2_directconelemattr_32() {
    //  Attribute containing two apostrophes(#2).
    // XPath: string(<foo attr=''''/>/@attr)
    // Expected: '
    // Calls stub function - will fail until prod:DirAttributeList is implemented
    let _ = stub_proddirattributelist();
}

#[test]
fn k2_directconelemattr_34() {
    //  Extract the string value from a directly constructed attribute whose content co
    // XPath: string(<elem attr=\"{comment {\" content \"}}\"/>/@attr)
    // Expected: content
    // Calls stub function - will fail until prod:DirAttributeList is implemented
    let _ = stub_proddirattributelist();
}

#[test]
fn k2_directconelemattr_35() {
    //  Extract the string value from a directly constructed attribute whose content co
    // XPath: local-name(<elem attr=\"{comment {\" content \"}}\"/>/@attr)
    // Expected: attr
    // Calls stub function - will fail until prod:DirAttributeList is implemented
    let _ = stub_proddirattributelist();
}

#[test]
fn k2_directconelemattr_36() {
    //  Extract the string value from a directly constructed attribute whose content co
    // XPath: string(<elem attr=\"{processing-instruction name {\" content
    // Expected: content
    // Calls stub function - will fail until prod:DirAttributeList is implemented
    let _ = stub_proddirattributelist();
}

#[test]
fn k2_directconelemattr_37() {
    //  Extract the local-name from a directly constructed attribute whose content cons
    // XPath: local-name(<elem attr=\"{processing-instruction name {\" con
    // Expected: attr
    // Calls stub function - will fail until prod:DirAttributeList is implemented
    let _ = stub_proddirattributelist();
}

#[test]
fn k2_directconelemattr_38() {
    //  Extract the string value from a directly constructed attribute whose content co
    // XPath: local-name(<elem attr=\"{attribute name {\" content \"}}\"/>
    // Expected: attr
    // Calls stub function - will fail until prod:DirAttributeList is implemented
    let _ = stub_proddirattributelist();
}

#[test]
fn k2_directconelemattr_39() {
    //  Extract the string value from a directly constructed attribute whose content co
    // XPath: string(<elem attr=\"{attribute name {\" content \"}}\"/>/@at
    // Expected: content
    // Calls stub function - will fail until prod:DirAttributeList is implemented
    let _ = stub_proddirattributelist();
}

#[test]
fn k2_directconelemattr_47() {
    //  Ensure the typed value of directly constructed attributes has the correct type.
    // XPath: data(<e foo=\"content\"/>/@*) instance of xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:DirAttributeList is implemented
    let _ = stub_proddirattributelist();
}

#[test]
fn k2_directconelemattr_70() {
    //  Ensure that attributes constructed with direct constructors doesn't pick up the
    // XPath: declare default element namespace \"http://typedecl\"; names
    // Expected: true
    // Calls stub function - will fail until prod:DirAttributeList is implemented
    let _ = stub_proddirattributelist();
}

#[test]
fn k2_directconelemattr_71() {
    //  Ensure that attributes constructed with computed constructors doesn't pick up t
    // XPath: declare default element namespace \"http://typedecl\"; names
    // Expected: true
    // Calls stub function - will fail until prod:DirAttributeList is implemented
    let _ = stub_proddirattributelist();
}

#[test]
fn k2_directconelemattr_72() {
    //  Ensure that attributes constructed with computed constructors doesn't pick up t
    // XPath: namespace-uri-from-QName(node-name(exactly-one(<e xmlns=\"ht
    // Expected: true
    // Calls stub function - will fail until prod:DirAttributeList is implemented
    let _ = stub_proddirattributelist();
}

#[test]
fn k2_directconelemattr_73() {
    //  Ensure that attributes constructed with computed constructors doesn't pick up t
    // XPath: declare default element namespace \"http://example.com/\"; n
    // Expected: true
    // Calls stub function - will fail until prod:DirAttributeList is implemented
    let _ = stub_proddirattributelist();
}
