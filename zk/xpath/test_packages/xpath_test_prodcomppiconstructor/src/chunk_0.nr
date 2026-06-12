//! Test chunk 0 for prod:CompPIConstructor
//! Contains 11 tests

use dep::xpath::stub_prodcomppiconstructor;

#[test]
fn constr_comppi_parent_1() {
    //  empty parent
    // XPath: count((processing-instruction pi {()})/..)
    // Expected: 0
    // Calls stub function - will fail until prod:CompPIConstructor is implemented
    let _ = stub_prodcomppiconstructor();
}

#[test]
fn constr_comppi_string_1() {
    //  string value of PI
    // XPath: fn:string(processing-instruction pi {'a', element a {}, 'b'}
    // Expected: a  b
    // Calls stub function - will fail until prod:CompPIConstructor is implemented
    let _ = stub_prodcomppiconstructor();
}

#[test]
fn constr_comppi_data_1() {
    //  typed value of PI
    // XPath: fn:data(processing-instruction pi {'a', element a {}, 'b'})
    // Expected: a  b
    // Calls stub function - will fail until prod:CompPIConstructor is implemented
    let _ = stub_prodcomppiconstructor();
}

#[test]
fn constr_comppi_space_3() {
    //  leading whitespace in PI content
    // XPath: string-to-codepoints(processing-instruction pi {' text'})
    // Expected: 116 101 120 116
    // Calls stub function - will fail until prod:CompPIConstructor is implemented
    let _ = stub_prodcomppiconstructor();
}

#[test]
fn constr_comppi_space_4() {
    //  leading whitespace in PI content
    // XPath: string-to-codepoints(processing-instruction pi {'&#x20;&#x0A
    // Expected: 116 101 120 116
    // Calls stub function - will fail until prod:CompPIConstructor is implemented
    let _ = stub_prodcomppiconstructor();
}

#[test]
fn k2_computeconpi_4() {
    //  Extract the name from a computed PI.
    // XPath: local-name(processing-instruction {\" name \"} {\"content\"}
    // Expected: name
    // Calls stub function - will fail until prod:CompPIConstructor is implemented
    let _ = stub_prodcomppiconstructor();
}

#[test]
fn k2_computeconpi_5() {
    //  Extract the name from a computed PI.
    // XPath: local-name(processing-instruction {\" XmLnaMe \"} {\"content
    // Expected: XmLnaMe
    // Calls stub function - will fail until prod:CompPIConstructor is implemented
    let _ = stub_prodcomppiconstructor();
}

#[test]
fn k2_computeconpi_8() {
    //  Extract the data from a computed PI.
    // XPath: string(processing-instruction {\"thename\"} {\"asdas? >\"})
    // Expected: asdas? >
    // Calls stub function - will fail until prod:CompPIConstructor is implemented
    let _ = stub_prodcomppiconstructor();
}

#[test]
fn k2_computeconpi_9() {
    //  Extract (possible parser-confusing) PI data.
    // XPath: string(processing-instruction {\"thename\"} {\"content {1+ }
    // Expected: content {1+ } {
    // Calls stub function - will fail until prod:CompPIConstructor is implemented
    let _ = stub_prodcomppiconstructor();
}

#[test]
fn k2_computeconpi_10() {
    //  Data that only is whitespace.
    // XPath: processing-instruction name {\" \"} eq \"\"
    // Expected: true
    // Calls stub function - will fail until prod:CompPIConstructor is implemented
    let _ = stub_prodcomppiconstructor();
}

#[test]
fn k2_computeconpi_11() {
    //  Test the atomized type.
    // XPath: data(processing-instruction name {\"content\"}) instance of
    // Expected: true
    // Calls stub function - will fail until prod:CompPIConstructor is implemented
    let _ = stub_prodcomppiconstructor();
}
