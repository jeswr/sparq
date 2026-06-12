//! Test chunk 0 for prod:SequenceType
//! Contains 21 tests

use dep::xpath::stub_prodsequencetype;

#[test]
fn sequence_type_1() {
    //  Evaluate simple sequence type using instance of and integers.
    // XPath: (1,2,3) instance of xs:integer*
    // Expected: true
    // Calls stub function - will fail until prod:SequenceType is implemented
    let _ = stub_prodsequencetype();
}

#[test]
fn sequence_type_2() {
    //  Evaluate simple sequence type using instance of and doubles.
    // XPath: (1E3,2E3,3E3) instance of xs:double*
    // Expected: true
    // Calls stub function - will fail until prod:SequenceType is implemented
    let _ = stub_prodsequencetype();
}

#[test]
fn sequence_type_3() {
    //  Evaluate simple sequence type using instance of and decimal.
    // XPath: (1.1,3.2,4.5) instance of xs:decimal*
    // Expected: true
    // Calls stub function - will fail until prod:SequenceType is implemented
    let _ = stub_prodsequencetype();
}

#[test]
fn sequence_type_4() {
    //  Evaluate simple sequence type using instance of and string.
    // XPath: (\"ab\",\"cde\",\"fgh\") instance of xs:string*
    // Expected: true
    // Calls stub function - will fail until prod:SequenceType is implemented
    let _ = stub_prodsequencetype();
}

#[test]
fn sequence_type_5() {
    //  Evaluate simple sequence type using instance of and strings. uses 'integer*' on
    // XPath: (\"ab\",\"cde\",\"fgh\") instance of xs:integer*
    // Expected: false
    // Calls stub function - will fail until prod:SequenceType is implemented
    let _ = stub_prodsequencetype();
}

#[test]
fn sequence_type_6() {
    //  Evaluate simple sequence type using instance of and string. uses 'integer?'.
    // XPath: (1) instance of xs:integer?
    // Expected: true
    // Calls stub function - will fail until prod:SequenceType is implemented
    let _ = stub_prodsequencetype();
}

#[test]
fn sequence_type_7() {
    //  Evaluate simple sequence type using instance of and integer. uses 'integer?'.
    // XPath: (1,2) instance of xs:integer?
    // Expected: false
    // Calls stub function - will fail until prod:SequenceType is implemented
    let _ = stub_prodsequencetype();
}

#[test]
fn sequence_type_8() {
    //  Evaluate simple sequence type using instance of and integer. uses 'integer+'.
    // XPath: (1,2) instance of xs:integer+
    // Expected: true
    // Calls stub function - will fail until prod:SequenceType is implemented
    let _ = stub_prodsequencetype();
}

#[test]
fn sequence_type_9() {
    //  Evaluate simple sequence type using castable and integer. uses 'integer?'.
    // XPath: (2) castable as xs:integer?
    // Expected: true
    // Calls stub function - will fail until prod:SequenceType is implemented
    let _ = stub_prodsequencetype();
}

#[test]
fn sequence_type_10() {
    //  Evaluate simple sequence type using castable and integer. uses 'integer?' on se
    // XPath: (1,2) castable as xs:integer?
    // Expected: false
    // Calls stub function - will fail until prod:SequenceType is implemented
    let _ = stub_prodsequencetype();
}

#[test]
fn sequence_type_11() {
    //  Evaluate simple sequence type using castable and double. uses 'double?' on sequ
    // XPath: (1E3) castable as xs:double?
    // Expected: true
    // Calls stub function - will fail until prod:SequenceType is implemented
    let _ = stub_prodsequencetype();
}

#[test]
fn sequence_type_12() {
    //  Evaluate simple sequence type using castable and double. uses 'double?' on sequ
    // XPath: (1E3,2E3) castable as xs:double?
    // Expected: false
    // Calls stub function - will fail until prod:SequenceType is implemented
    let _ = stub_prodsequencetype();
}

#[test]
fn sequence_type_13() {
    //  Evaluate simple sequence type using castable and decimal. uses 'decimal?' on se
    // XPath: (1.2) castable as xs:decimal?
    // Expected: true
    // Calls stub function - will fail until prod:SequenceType is implemented
    let _ = stub_prodsequencetype();
}

#[test]
fn sequence_type_14() {
    //  Evaluate simple sequence type using castable and decimal. uses 'decimal?' on se
    // XPath: (1.2,2.1) castable as xs:decimal?
    // Expected: false
    // Calls stub function - will fail until prod:SequenceType is implemented
    let _ = stub_prodsequencetype();
}

#[test]
fn sequence_type_15() {
    //  Evaluate simple sequence type using castable and string. uses 'string?' on sequ
    // XPath: (\"abc\") castable as xs:string?
    // Expected: true
    // Calls stub function - will fail until prod:SequenceType is implemented
    let _ = stub_prodsequencetype();
}

#[test]
fn sequence_type_16() {
    //  Evaluate simple sequence type using castable and string. uses 'string?' on sequ
    // XPath: (\"abc\",\"def\") castable as xs:string?
    // Expected: false
    // Calls stub function - will fail until prod:SequenceType is implemented
    let _ = stub_prodsequencetype();
}

#[test]
fn sequence_type_17() {
    //  Evaluate simple sequence type using castable and integer. uses 'integer?' on se
    // XPath: (\"abc\") castable as xs:integer?
    // Expected: false
    // Calls stub function - will fail until prod:SequenceType is implemented
    let _ = stub_prodsequencetype();
}

#[test]
fn sequence_type_18() {
    //  Evaluate simple sequence type using castable and integer. uses 'integer?' on se
    // XPath: (\"abc\",\"def\") castable as xs:integer?
    // Expected: false
    // Calls stub function - will fail until prod:SequenceType is implemented
    let _ = stub_prodsequencetype();
}

#[test]
fn sequence_type_19() {
    //  Evaluate simple sequence type using instance and boolean. uses 'boolean*' on se
    // XPath: (fn:true(),fn:false()) instance of xs:boolean*
    // Expected: true
    // Calls stub function - will fail until prod:SequenceType is implemented
    let _ = stub_prodsequencetype();
}

#[test]
fn sequence_type_20() {
    //  Evaluate simple sequence type using instance and boolean. uses 'boolean?' on se
    // XPath: (fn:true(),fn:false()) instance of xs:boolean?
    // Expected: false
    // Calls stub function - will fail until prod:SequenceType is implemented
    let _ = stub_prodsequencetype();
}

#[test]
fn sequence_type_21() {
    //  Evaluate simple sequence type using instance and boolean. uses 'boolean+' on se
    // XPath: (fn:true(),fn:false()) instance of xs:boolean+
    // Expected: true
    // Calls stub function - will fail until prod:SequenceType is implemented
    let _ = stub_prodsequencetype();
}
