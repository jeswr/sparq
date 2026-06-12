//! Test chunk 0 for prod:StringConstructor
//! Contains 8 tests

use dep::xpath::stub_prodstringconstructor;

#[test]
fn string_constructor_021() {
    // Use in regex
    // XPath: matches(\"\"\"\", ``[[\"']]``)
    // Expected: true
    // Calls stub function - will fail until prod:StringConstructor is implemented
    let _ = stub_prodstringconstructor();
}

#[test]
fn string_constructor_022() {
    // Empty string constructor
    // XPath: ``[]``
    // Expected: ""
    // Calls stub function - will fail until prod:StringConstructor is implemented
    let _ = stub_prodstringconstructor();
}

#[test]
fn string_constructor_023() {
    // Effect of a misplaced space
    // XPath: ``[` {$n}`]``
    // Expected: "` {$n}`"
    // Calls stub function - will fail until prod:StringConstructor is implemented
    let _ = stub_prodstringconstructor();
}

#[test]
fn string_constructor_024() {
    // Absent enclosed expression
    // XPath: ``[` *`{}`* `]``
    // Expected: "` ** `"
    // Calls stub function - will fail until prod:StringConstructor is implemented
    let _ = stub_prodstringconstructor();
}

#[test]
fn string_constructor_025() {
    // Absent enclosed expression
    // XPath: ``[` *`{(:Nothing here:)}`* `]``
    // Expected: "` ** `"
    // Calls stub function - will fail until prod:StringConstructor is implemented
    let _ = stub_prodstringconstructor();
}

#[test]
fn string_constructor_026() {
    // Date-valued enclosed expression
    // XPath: ``[ *`{xs:date('2012-05-05')}`* ]``
    // Expected: " *2012-05-05* "
    // Calls stub function - will fail until prod:StringConstructor is implemented
    let _ = stub_prodstringconstructor();
}

#[test]
fn string_constructor_027() {
    // Confusing enclosed expression in element constructor
    // XPath: <a>Today is `{xs:date('2012-05-05')}`</a>
    // Expected: "Today is `2012-05-05`"
    // Calls stub function - will fail until prod:StringConstructor is implemented
    let _ = stub_prodstringconstructor();
}

#[test]
fn string_constructor_912() {
    // String constructor containing sequence
    // XPath: declare variable $n as xs:integer external := 10;          `
    // Expected: "There were 1 2 3 4 5 6 7 8 9 10 green b
    // Calls stub function - will fail until prod:StringConstructor is implemented
    let _ = stub_prodstringconstructor();
}
