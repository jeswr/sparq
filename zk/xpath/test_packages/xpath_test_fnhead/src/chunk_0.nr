//! Test chunk 0 for fn:head
//! Contains 6 tests

use dep::xpath::stub_fnhead;

#[test]
fn head_001() {
    //  head() of a simple sequence
    // XPath: head(3 to 10)
    // Expected: 3
    // Calls stub function - will fail until fn:head is implemented
    let _ = stub_fnhead();
}

#[test]
fn head_002() {
    //  head() of a simple sequence
    // XPath: head((\"a\", \"b\", current-dateTime()))
    // Expected: "a"
    // Calls stub function - will fail until fn:head is implemented
    let _ = stub_fnhead();
}

#[test]
fn head_003() {
    //  head() of a node sequence
    // XPath: let $a := /works return head($a/*)/string(@name)
    // Expected: Jane Doe 1
    // Calls stub function - will fail until fn:head is implemented
    let _ = stub_fnhead();
}

#[test]
fn head_004() {
    //  head() of a singleton sequence
    // XPath: let $a := /works return name(head($a))
    // Expected: works
    // Calls stub function - will fail until fn:head is implemented
    let _ = stub_fnhead();
}

#[test]
fn head_006() {
    //  head() preserves identity
    // XPath: let $a := /works/* return (head($a) is $a[1])
    // Expected: true
    // Calls stub function - will fail until fn:head is implemented
    let _ = stub_fnhead();
}

#[test]
fn head_007() {
    //  dynamic call on head()
    // XPath: let $f := if (current-date() gt xs:date('1900-01-01')) then
    // Expected: 5
    // Calls stub function - will fail until fn:head is implemented
    let _ = stub_fnhead();
}
