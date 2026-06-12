//! Test chunk 4 for fn:deep-equal
//! Contains 49 tests

use dep::xpath::stub_fndeep_equal;

#[test]
fn k_seqdeepequalfunc_67() {
    // Use case-blind collation, if supported, within element nodes
    // XPath: deep-equal(<a><b>abc</b></a>, <a><b>ABC</b></a>,
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_68() {
    // Use case-blind collation, if supported, doesn't apply to names
    // XPath: deep-equal(<a><b>abc</b></a>, <a><B>abc</B></a>,
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_69() {
    // Use case-blind collation, if supported, doesn't apply to attribute names
    // XPath: deep-equal(<a><b x=\"abc\"/></a>, <a><b X=\"abc\"/></a>,
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_70() {
    // Use case-blind collation, if supported, partially applied
    // XPath: let $deep := deep-equal(?, ?, \"http://www.w3.org/2005/xpath
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_71() {
    // Use case-blind collation, if supported, in dynamic call
    // XPath: let $deep := deep-equal#3           return $deep(<a><b>abc</
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k2_seqdeepequalfunc_1() {
    //  Two date/time values that never will compare equal.
    // XPath: not(deep-equal(current-time(), current-date()))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k2_seqdeepequalfunc_2() {
    //  A date and node value that never will compare equal.
    // XPath: deep-equal(current-time(), <e/>)
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k2_seqdeepequalfunc_3() {
    //  A date and node value that never will compare equal(#2).
    // XPath: deep-equal(<e/>, current-time())
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k2_seqdeepequalfunc_4() {
    //  Time values that never will compare equal.
    // XPath: deep-equal((current-time(), current-time(), current-time()),
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k2_seqdeepequalfunc_5() {
    //  Comparing mixed content.
    // XPath: deep-equal((1, <e/>, 2), (1, <e/>, 2))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k2_seqdeepequalfunc_6() {
    //  Comparing mixed content(#2).
    // XPath: deep-equal((1, <a/>, 2), (1, <b/>, 2))
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k2_seqdeepequalfunc_7() {
    //  Comparing mixed content(#3).
    // XPath: deep-equal((<a/>, <b/>, 4), (<a/>, <b/>, 2))
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k2_seqdeepequalfunc_10() {
    //  fn:deep-equal() with two sequence of nodes.
    // XPath: deep-equal((<a/>, <b/>, <c/>), (<a/>, <b/>, <c/>))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k2_seqdeepequalfunc_11() {
    //  fn:deep-equal() with two sequence of nodes(#2).
    // XPath: deep-equal((<a/>, <b/>, <c/>), (<a/>, <b/>, <a/>))
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k2_seqdeepequalfunc_12() {
    //  fn:deep-equal() with two sequence of nodes(#3).
    // XPath: deep-equal((<a/>, <b/>), (<a/>, <b/>, <c/>))
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k2_seqdeepequalfunc_13() {
    //  fn:deep-equal() with two sequence of nodes(#4).
    // XPath: deep-equal((<a/>, <b/>, <c/>), (<a/>, <b/>))
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k2_seqdeepequalfunc_14() {
    //  Ensure processing instructions are ignored if children of a document node.
    // XPath: declare variable $d1 := document { <?target data?>, text{\"s
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k2_seqdeepequalfunc_15() {
    //  Ensure processing instructions are ignored if children of a document node(#2).
    // XPath: declare variable $d1 := document {()}; declare variable $d2
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k2_seqdeepequalfunc_16() {
    //  Ensure processing instructions are ignored if children of a document node.
    // XPath: declare variable $d1 := document { <?target data?>, text{\"s
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k2_seqdeepequalfunc_17() {
    //  Ensure processing instructions are ignored if children of a document node(#2).
    // XPath: declare variable $d1 := document {()}; declare variable $d2
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k2_seqdeepequalfunc_18() {
    //  fn:deep-equal() with mixed content.
    // XPath: deep-equal(<e>1</e>, 1)
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k2_seqdeepequalfunc_19() {
    //  fn:deep-equal() with mixed content(#2).
    // XPath: deep-equal(1, <e>1</e>)
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k2_seqdeepequalfunc_20() {
    //  One of the operands has two text nodes, and hence it evaluate to false.
    // XPath: declare variable $d1 := <e a=\"1\" b=\"2\">te<?target data?>
    // Expected: false false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k2_seqdeepequalfunc_21() {
    //  Processing instructions inside elements are ignored.
    // XPath: declare variable $d1 := <e a=\"1\" b=\"2\"><?target data?>te
    // Expected: true true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k2_seqdeepequalfunc_22() {
    //  One of the operands has two text nodes besides the comment, and hence it evalua
    // XPath: declare variable $d1 := <e a=\"1\" b=\"2\">te<!-- content --
    // Expected: false false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k2_seqdeepequalfunc_23() {
    //  Comments inside elements are ignored.
    // XPath: declare variable $d1 := <e a=\"1\" b=\"2\"><!-- content -->t
    // Expected: true true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k2_seqdeepequalfunc_24() {
    //  Compare attribute nodes.
    // XPath: deep-equal(attribute name {\"content\"}, attribute name {\"c
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k2_seqdeepequalfunc_25() {
    //  Compare attribute nodes in a weird order.
    // XPath: deep-equal((attribute name2 {\"content\"}, attribute name {\
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k2_seqdeepequalfunc_26() {
    //  Compare operands with attributes nodes of different size.
    // XPath: deep-equal((attribute name {\"content\"}, attribute name {\"
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k2_seqdeepequalfunc_27() {
    //  Compare operands with attributes nodes of different size(#2).
    // XPath: deep-equal((attribute name {\"content\"}, attribute name {\"
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k2_seqdeepequalfunc_28() {
    //  Test deep-equalness of two attribute nodes.
    // XPath: deep-equal(attribute name {}, attribute name {})
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k2_seqdeepequalfunc_29() {
    //  Test deep-equalness of two attribute nodes.
    // XPath: deep-equal(attribute name {\"content\"}, attribute name {\"c
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k2_seqdeepequalfunc_30() {
    //  Test deep-equalness of two attribute nodes that has the same name, but differen
    // XPath: deep-equal(attribute name {\"content\"}, attribute name {})
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k2_seqdeepequalfunc_31() {
    //  Test deep-equalness of two attribute nodes that has same text content but diffe
    // XPath: deep-equal(attribute name {}, attribute name2 {})
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k2_seqdeepequalfunc_32() {
    //  Test deep-equalness of two attribute nodes that has identical text content but
    // XPath: deep-equal(attribute name {\"content \"}, attribute name2 {\
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k2_seqdeepequalfunc_33() {
    //  Namespace prefixes and namespace nodes are ignored
    // XPath: deep-equal(<e xmlns=\"http://www.example.com/ns\"/>, <p:e xm
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k2_seqdeepequalfunc_34() {
    //  Parent node is ignored
    // XPath: deep-equal(<e><a>12</a></e>/*, <f><a>12</a></f>/*)
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k2_seqdeepequalfunc_35() {
    //  Type annotation is ignored for atomic values
    // XPath: deep-equal(\"a\", xs:NCName(\"a\"))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k2_seqdeepequalfunc_40() {
    //  Handle dependency on implicit timezone
    // XPath: deep-equal(xs:dateTime('2012-05-30T12:00:00'), xs:dateTime('
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn cbcl_deep_equal_001() {
    //  Test deep-equal ignores processing instructions and comments in elements.
    // XPath: let $doc1 := <doc><?processing-instruction content ?></doc>
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn cbcl_deep_equal_002() {
    //  Test deep-equal on processing instructions with different values.
    // XPath: let $doc1 := <?cheese brie?> let $doc2 := <?cheese stilton?>
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn cbcl_deep_equal_003() {
    //  Test deep-equal on processing instructions with different names.
    // XPath: let $doc1 := <?foo test?> let $doc2 := <?bar test?> return d
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn cbcl_deep_equal_004() {
    //  Test deep-equal on equal processing instructions
    // XPath: let $doc1 := <?foo bar?> let $doc2 := <?foo bar?> return dee
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn cbcl_deep_equal_005() {
    //  Test deep-equal simplifying to fn:empty
    // XPath: declare function local:f($x as xs:integer)as xs:integer* { 1
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn cbcl_deep_equal_006() {
    //  Test deep-equal simplifying to fn:empty
    // XPath: declare function local:f($x as xs:integer) as xs:integer* {
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn cbcl_deep_equal_007() {
    //  Test deep-equal on comparison requiring dynamic context
    // XPath: deep-equal(xs:gMonth(\"--12+03:00\"), xs:gMonth(\"--12\")) e
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn cbcl_deep_equal_008() {
    //  Test that deep equal compares untypedAtomic values correctly
    // XPath: deep-equal((xs:untypedAtomic(\"a\"), xs:untypedAtomic(\"P1Y\
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn cbcl_deep_equal_009() {
    //  test fn:deep equal (was duplicate cbcl-deep-equal-001)
    // XPath: fn:deep-equal(<a b=\"b\" c=\"c\" />, <a b=\"b\" />)
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn cbcl_deep_equal_010() {
    //  Test that deep equal compares untypedAtomic values correctly
    // XPath: deep-equal((<a />, xs:untypedAtomic(\"a\"), xs:untypedAtomic
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}
