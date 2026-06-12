//! Test chunk 3 for fn:deep-equal
//! Contains 50 tests

use dep::xpath::stub_fndeep_equal;

#[test]
fn k_seqdeepequalfunc_17() {
    //  A test whose essence is: `not(deep-equal(QName('example.com', 'ncname'), 3e2))`
    // XPath: not(deep-equal(QName(\"example.com\", \"ncname\"), 3e2))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_18() {
    //  A test whose essence is: `not(deep-equal((1, 2, xs:anyURI('example.com')), (1,
    // XPath: not(deep-equal((1, 2, xs:anyURI(\"example.com\")), (1, 2, 3)
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_19() {
    //  A test whose essence is: `not(deep-equal((1, xs:decimal('2.2'), 3), (1, 2, 3)))
    // XPath: not(deep-equal((1, xs:decimal(\"2.2\"), 3), (1, 2, 3)))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_20() {
    //  A test whose essence is: `not(deep-equal((true(), 2, 3), (1, 2, 3)))`.
    // XPath: not(deep-equal((true(), 2, 3), (1, 2, 3)))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_21() {
    //  A test whose essence is: `not(deep-equal((1, 2, 3), (1, 2, QName('example.com',
    // XPath: not(deep-equal((1, 2, 3), (1, 2, QName(\"example.com\", \"nc
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_22() {
    //  A test whose essence is: `not(deep-equal((1, 2, 3), (1, xs:hexBinary('FF'), 3))
    // XPath: not(deep-equal((1, 2, 3), (1, xs:hexBinary(\"FF\"), 3)))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_23() {
    //  A test whose essence is: `not(deep-equal((1, 2, 3), (xs:base64Binary('FFFF'), 2
    // XPath: not(deep-equal((1, 2, 3), (xs:base64Binary(\"FFFF\"), 2, 3))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_24() {
    //  A test whose essence is: `deep-equal((1, 2, xs:base64Binary('FFFF')), (1, 2, xs
    // XPath: deep-equal((1, 2, xs:base64Binary(\"FFFF\")), (1, 2, xs:base
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_25() {
    //  A test whose essence is: `deep-equal((1, 1, 1), (1, 1, 1))`.
    // XPath: deep-equal((1, 1, 1), (1, 1, 1))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_26() {
    //  A test whose essence is: `not(deep-equal((1, 1, 3), (1, 1, 1)))`.
    // XPath: not(deep-equal((1, 1, 3), (1, 1, 1)))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_27() {
    //  A test whose essence is: `not(deep-equal((1, 3, 1), (1, 1, 1)))`.
    // XPath: not(deep-equal((1, 3, 1), (1, 1, 1)))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_28() {
    //  A test whose essence is: `not(deep-equal((3, 1, 1), (1, 1, 1)))`.
    // XPath: not(deep-equal((3, 1, 1), (1, 1, 1)))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_29() {
    //  A test whose essence is: `not(deep-equal((1, 1, 1), (3, 1, 1)))`.
    // XPath: not(deep-equal((1, 1, 1), (3, 1, 1)))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_30() {
    //  A test whose essence is: `not(deep-equal((1, 1, 1), (1, 3, 1)))`.
    // XPath: not(deep-equal((1, 1, 1), (1, 3, 1)))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_31() {
    //  A test whose essence is: `not(deep-equal((1, 1, 1), (1, 1, 3)))`.
    // XPath: not(deep-equal((1, 1, 1), (1, 1, 3)))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_32() {
    //  A test whose essence is: `deep-equal(index-of(20, 20), (1))`.
    // XPath: deep-equal(index-of(20, 20), (1))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_33() {
    //  A test whose essence is: `deep-equal(index-of((20, 40), 20), (1))`.
    // XPath: deep-equal(index-of((20, 40), 20), (1))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_34() {
    //  A test whose essence is: `deep-equal(index-of((20, 20), 20), (1, 2))`.
    // XPath: deep-equal(index-of((20, 20), 20), (1, 2))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_35() {
    //  A test whose essence is: `deep-equal(index-of((20, 40, 20), 20), (1, 3))`.
    // XPath: deep-equal(index-of((20, 40, 20), 20), (1, 3))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_36() {
    //  A test whose essence is: `deep-equal((1, 1, 'str'), (1, 1, 'str'))`.
    // XPath: deep-equal((1, 1, \"str\"), (1, 1, \"str\"))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_37() {
    //  A test whose essence is: `deep-equal((1, 'str', 1), (1, 'str', 1))`.
    // XPath: deep-equal((1, \"str\", 1), (1, \"str\", 1))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_38() {
    //  A test whose essence is: `deep-equal(('str', 1, 1), ('str', 1, 1))`.
    // XPath: deep-equal((\"str\", 1, 1), (\"str\", 1, 1))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_39() {
    //  A test whose essence is: `deep-equal((1, 1, ()), (1, 1, ()))`.
    // XPath: deep-equal((1, 1, ()), (1, 1, ()))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_40() {
    //  A test whose essence is: `deep-equal((1, (), 1), (1, (), 1))`.
    // XPath: deep-equal((1, (), 1), (1, (), 1))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_41() {
    //  A test whose essence is: `deep-equal(((), 1, 1), ((), 1, 1))`.
    // XPath: deep-equal(((), 1, 1), ((), 1, 1))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_42() {
    //  A test whose essence is: `not(deep-equal((1, 1, ()), (1, 1, 1)))`.
    // XPath: not(deep-equal((1, 1, ()), (1, 1, 1)))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_43() {
    //  A test whose essence is: `not(deep-equal((1, (), 1), (1, 1, 1)))`.
    // XPath: not(deep-equal((1, (), 1), (1, 1, 1)))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_44() {
    //  A test whose essence is: `not(deep-equal(((), 1, 1), (1, 1, 1)))`.
    // XPath: not(deep-equal(((), 1, 1), (1, 1, 1)))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_45() {
    //  A test whose essence is: `not(deep-equal((1, 1, 1), (1, 1, ())))`.
    // XPath: not(deep-equal((1, 1, 1), (1, 1, ())))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_46() {
    //  A test whose essence is: `not(deep-equal((1, 1, 1), (1, (), 1)))`.
    // XPath: not(deep-equal((1, 1, 1), (1, (), 1)))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_47() {
    //  A test whose essence is: `not(deep-equal((1, 1, 1), ((), 1, 1)))`.
    // XPath: not(deep-equal((1, 1, 1), ((), 1, 1)))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_48() {
    //  A test whose essence is: `not(deep-equal((1, 1), (1, 1, 1)))`.
    // XPath: not(deep-equal((1, 1), (1, 1, 1)))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_49() {
    //  A test whose essence is: `not(deep-equal((1, 1, 1), (1, 1)))`.
    // XPath: not(deep-equal((1, 1, 1), (1, 1)))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_50() {
    //  A test whose essence is: `not(deep-equal((), (1, 1, 1)))`.
    // XPath: not(deep-equal((), (1, 1, 1)))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_51() {
    //  A test whose essence is: `not(deep-equal((1, 1, 1), ()))`.
    // XPath: not(deep-equal((1, 1, 1), ()))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_52() {
    //  A test whose essence is: `deep-equal((), reverse(0 to -5))`.
    // XPath: deep-equal((), reverse(0 to -5))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_53() {
    //  A test whose essence is: `deep-equal((), 0 to -5)`.
    // XPath: deep-equal((), 0 to -5)
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_54() {
    //  A test whose essence is: `deep-equal(reverse(0 to -5), ())`.
    // XPath: deep-equal(reverse(0 to -5), ())
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_55() {
    //  A test whose essence is: `deep-equal(0 to -5, ())`.
    // XPath: deep-equal(0 to -5, ())
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_56() {
    // Use case-blind collation, if supported
    // XPath: deep-equal((\"a\", \"A\"), (\"A\", \"a\"), \"http://www.w3.o
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_57() {
    // Use case-blind collation, if supported
    // XPath: deep-equal((\"a\", \"A\"), (\"A\", \"b\"), \"http://www.w3.o
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_58() {
    // Use case-blind collation, if supported, within attribute nodes
    // XPath: deep-equal(<a><b x=\"abc\"/></a>, <a><b x=\"ABC\"/></a>,
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_59() {
    // Use case-blind collation, if supported, within element nodes
    // XPath: deep-equal(<a><b>abc</b></a>, <a><b>ABC</b></a>,
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_60() {
    // Use case-blind collation, if supported, doesn't apply to names
    // XPath: deep-equal(<a><b>abc</b></a>, <a><B>abc</B></a>,
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_61() {
    // Use case-blind collation, if supported, doesn't apply to attribute names
    // XPath: deep-equal(<a><b x=\"abc\"/></a>, <a><b X=\"abc\"/></a>,
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_62() {
    // Use case-blind collation, if supported, partially applied
    // XPath: let $deep := deep-equal(?, ?, \"http://www.w3.org/2010/09/qt
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_63() {
    // Use case-blind collation, if supported, in dynamic call
    // XPath: let $deep := deep-equal#3           return $deep(<a><b>abc</
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_64() {
    // Use case-blind collation, if supported
    // XPath: deep-equal((\"a\", \"A\"), (\"A\", \"a\"), \"http://www.w3.o
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_65() {
    // Use case-blind collation, if supported
    // XPath: deep-equal((\"a\", \"A\"), (\"A\", \"b\"), \"http://www.w3.o
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn k_seqdeepequalfunc_66() {
    // Use case-blind collation, if supported, within attribute nodes
    // XPath: deep-equal(<a><b x=\"abc\"/></a>, <a><b x=\"ABC\"/></a>,
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}
