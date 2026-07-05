//! Real test vectors for op:NOTATION-equal (F&O sec. 9.1)
//! op:NOTATION-equal($arg1 as xs:NOTATION, $arg2 as xs:NOTATION) as xs:boolean
//! xs:NOTATION equality is by expanded name (namespace URI + local name),
//! identical to xs:QName equality (F&O sec. 9.1). Oracle: XPath F&O 3.2 sec. 9.1.
//! Implementation delegates to qname_equal (same type, same semantics).
//! [SONNET-4.6] (sq-rcpdz)

use dep::xpath::{notation_equal, XsdQName};

#[test]
fn opnotation_equal_same_ns_same_local() {
    // Same namespace URI and same local name -> equal.
    // Oracle F&O sec.9.1: identity of expanded names.
    let ns = "http://example.com".as_bytes();
    let loc = "foo".as_bytes();
    let a: XsdQName<18, 3> = XsdQName::new(ns, 18, loc, 3);
    let b: XsdQName<18, 3> = XsdQName::new(ns, 18, loc, 3);
    assert(notation_equal::<18, 3, 18, 3>(a, b));
}

#[test]
fn opnotation_equal_same_ns_different_local() {
    // Same NS but different local names -> not equal.
    // Oracle F&O sec.9.1: local name must match.
    let ns = "http://example.com".as_bytes();
    let loc_a = "foo".as_bytes();
    let loc_b = "bar".as_bytes();
    let a: XsdQName<18, 3> = XsdQName::new(ns, 18, loc_a, 3);
    let b: XsdQName<18, 3> = XsdQName::new(ns, 18, loc_b, 3);
    assert(!notation_equal::<18, 3, 18, 3>(a, b));
}

#[test]
fn opnotation_equal_different_ns_same_local() {
    // Different NS, same local name -> not equal.
    // Oracle F&O sec.9.1: namespace URI must match.
    let ns_a = "http://example.com".as_bytes();
    let ns_b = "http://other.org  ".as_bytes(); // same length, different content
    let loc = "foo".as_bytes();
    let a: XsdQName<18, 3> = XsdQName::new(ns_a, 18, loc, 3);
    let b: XsdQName<18, 3> = XsdQName::new(ns_b, 18, loc, 3);
    assert(!notation_equal::<18, 3, 18, 3>(a, b));
}

#[test]
fn opnotation_equal_no_namespace_same_local() {
    // No-namespace NOTATION values: equality by local name only.
    // Oracle F&O sec.9.1: empty namespace URI is a valid expanded name.
    let loc = "mynotation".as_bytes();
    let a: XsdQName<0, 10> = XsdQName::from_local_name(loc, 10);
    let b: XsdQName<0, 10> = XsdQName::from_local_name(loc, 10);
    assert(notation_equal::<0, 10, 0, 10>(a, b));
}

#[test]
fn opnotation_equal_no_namespace_different_local() {
    // No-namespace NOTATIONs with different local names -> not equal.
    let loc_a = "nota1     ".as_bytes();
    let loc_b = "nota2     ".as_bytes();
    let a: XsdQName<0, 10> = XsdQName::from_local_name(loc_a, 5);
    let b: XsdQName<0, 10> = XsdQName::from_local_name(loc_b, 5);
    assert(!notation_equal::<0, 10, 0, 10>(a, b));
}

#[test]
fn opnotation_equal_reflexivity() {
    // Every NOTATION must equal itself (reflexivity).
    let ns = "http://www.w3.org/TR/".as_bytes();
    let loc = "html".as_bytes();
    let a: XsdQName<21, 4> = XsdQName::new(ns, 21, loc, 4);
    assert(notation_equal::<21, 4, 21, 4>(a, a));
}
