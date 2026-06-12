//! Test chunk 0 for fn:string-to-codepoints
//! Contains 28 tests

use dep::xpath::stub_fnstring_to_codepoints;

#[test]
fn fn_string_to_codepoints_1() {
    //   Description: Evaluation of an 'fn:string-to-codepoints' with argument set to a
    // XPath: fn:string-to-codepoints('1')
    // Expected: 49
    // Calls stub function - will fail until fn:string-to-codepoints is implemented
    let _ = stub_fnstring_to_codepoints();
}

#[test]
fn fn_string_to_codepoints_2() {
    //   Description: Evaluation of an 'fn:string-to-codepoints' with argument set to a
    // XPath: fn:string-to-codepoints('a')
    // Expected: 97
    // Calls stub function - will fail until fn:string-to-codepoints is implemented
    let _ = stub_fnstring_to_codepoints();
}

#[test]
fn fn_string_to_codepoints_9() {
    //   Description: Evaluation of an 'fn:string-to-codepoints' that is used as argume
    // XPath: fn:count(fn:string-to-codepoints(\"A String\"))
    // Expected: 8
    // Calls stub function - will fail until fn:string-to-codepoints is implemented
    let _ = stub_fnstring_to_codepoints();
}

#[test]
fn fn_string_to_codepoints_10() {
    //   Description: Evaluation of an 'fn:string-to-codepoints' that is used as argume
    // XPath: fn:avg(fn:string-to-codepoints(\"A String\"))
    // Expected: 91
    // Calls stub function - will fail until fn:string-to-codepoints is implemented
    let _ = stub_fnstring_to_codepoints();
}

#[test]
fn fn_string_to_codepoints_11() {
    //   Description: Evaluation of an 'fn:string-to-codepoints' that is used as argume
    // XPath: fn:empty(fn:string-to-codepoints(\"A String\"))
    // Expected: false
    // Calls stub function - will fail until fn:string-to-codepoints is implemented
    let _ = stub_fnstring_to_codepoints();
}

#[test]
fn fn_string_to_codepoints_12() {
    //   Description: Evaluation of an 'fn:string-to-codepoints' that is used as argume
    // XPath: fn:empty(fn:string-to-codepoints(()))
    // Expected: true
    // Calls stub function - will fail until fn:string-to-codepoints is implemented
    let _ = stub_fnstring_to_codepoints();
}

#[test]
fn fn_string_to_codepoints_13() {
    //   Description: Evaluation of an 'fn:string-to-codepoints' that is used as argume
    // XPath: fn:exists(fn:string-to-codepoints(()))
    // Expected: false
    // Calls stub function - will fail until fn:string-to-codepoints is implemented
    let _ = stub_fnstring_to_codepoints();
}

#[test]
fn fn_string_to_codepoints_14() {
    //   Description: Evaluation of an 'fn:string-to-codepoints' that is used as argume
    // XPath: fn:exists(fn:string-to-codepoints(\"A String\"))
    // Expected: true
    // Calls stub function - will fail until fn:string-to-codepoints is implemented
    let _ = stub_fnstring_to_codepoints();
}

#[test]
fn fn_string_to_codepoints_15() {
    //  Evaluation of  'fn:string-to-codepoints' with non-BMP characters
    // XPath: fn:deep-equal(fn:string-to-codepoints(\"??\"), (65537, 65538
    // Expected: true
    // Calls stub function - will fail until fn:string-to-codepoints is implemented
    let _ = stub_fnstring_to_codepoints();
}

#[test]
fn k_stringtocodepointfunc_3() {
    //   Purpose: A test whose essence is: `empty(string-to-codepoints(()))`.
    // XPath: empty(string-to-codepoints(()))
    // Expected: true
    // Calls stub function - will fail until fn:string-to-codepoints is implemented
    let _ = stub_fnstring_to_codepoints();
}

#[test]
fn k_stringtocodepointfunc_4() {
    //   Purpose: A test whose essence is: `empty(string-to-codepoints(''))`.
    // XPath: empty(string-to-codepoints(\"\"))
    // Expected: true
    // Calls stub function - will fail until fn:string-to-codepoints is implemented
    let _ = stub_fnstring_to_codepoints();
}

#[test]
fn k_stringtocodepointfunc_5() {
    //   Purpose: A test whose essence is: `count(string-to-codepoints('123')) eq 3`.
    // XPath: count(string-to-codepoints(\"123\")) eq 3
    // Expected: true
    // Calls stub function - will fail until fn:string-to-codepoints is implemented
    let _ = stub_fnstring_to_codepoints();
}

#[test]
fn k_stringtocodepointfunc_6() {
    //   Purpose: A test whose essence is: `count(string-to-codepoints('')) eq 0`.
    // XPath: count(string-to-codepoints(\"\")) eq 0
    // Expected: true
    // Calls stub function - will fail until fn:string-to-codepoints is implemented
    let _ = stub_fnstring_to_codepoints();
}

#[test]
fn k_stringtocodepointfunc_7() {
    //   Purpose: A test whose essence is: `empty(string-to-codepoints(''))`.
    // XPath: empty(string-to-codepoints(\"\"))
    // Expected: true
    // Calls stub function - will fail until fn:string-to-codepoints is implemented
    let _ = stub_fnstring_to_codepoints();
}

#[test]
fn k_stringtocodepointfunc_8() {
    //   Purpose: A test whose essence is: `string-to-codepoints('e')`.
    // XPath: string-to-codepoints(\"e\")
    // Expected: 101
    // Calls stub function - will fail until fn:string-to-codepoints is implemented
    let _ = stub_fnstring_to_codepoints();
}

#[test]
fn k_stringtocodepointfunc_9() {
    //   Purpose: A test whose essence is: `deep-equal(string-to-codepoints('ee'), (101
    // XPath: deep-equal(string-to-codepoints(\"ee\"), (101, 101))
    // Expected: true
    // Calls stub function - will fail until fn:string-to-codepoints is implemented
    let _ = stub_fnstring_to_codepoints();
}

#[test]
fn k_stringtocodepointfunc_10() {
    //   Purpose: A test whose essence is: `deep-equal(string-to-codepoints('eee'), (10
    // XPath: deep-equal(string-to-codepoints(\"eee\"), (101, 101, 101))
    // Expected: true
    // Calls stub function - will fail until fn:string-to-codepoints is implemented
    let _ = stub_fnstring_to_codepoints();
}

#[test]
fn k_stringtocodepointfunc_11() {
    //   Purpose: A test whose essence is: `string-join(for $code in string-to-codepoin
    // XPath: string-join(for $code in string-to-codepoints(\"example.com/
    // Expected: true
    // Calls stub function - will fail until fn:string-to-codepoints is implemented
    let _ = stub_fnstring_to_codepoints();
}

#[test]
fn k_stringtocodepointfunc_12() {
    //   Purpose: Combine fn:deep-equal and string-to-codepoints().
    // XPath: deep-equal(string-to-codepoints(\"Th?r?se\"), (84, 104, 233,
    // Expected: true
    // Calls stub function - will fail until fn:string-to-codepoints is implemented
    let _ = stub_fnstring_to_codepoints();
}

#[test]
fn k_stringtocodepointfunc_13() {
    //   Purpose: A test whose essence is: `codepoints-to-string((87, 36, 56, 87, 102,
    // XPath: codepoints-to-string((87, 36, 56, 87, 102, 96)) eq \"W$8Wf`\
    // Expected: true
    // Calls stub function - will fail until fn:string-to-codepoints is implemented
    let _ = stub_fnstring_to_codepoints();
}

#[test]
fn k_stringtocodepointfunc_14() {
    //   Purpose: Combine string-to-codepoints() and a predicate.
    // XPath: string-to-codepoints(\"Th?r?se\")[last()]
    // Expected: 101
    // Calls stub function - will fail until fn:string-to-codepoints is implemented
    let _ = stub_fnstring_to_codepoints();
}

#[test]
fn k_stringtocodepointfunc_15() {
    //   Purpose: Combine string-to-codepoints() and a predicate.
    // XPath: string-to-codepoints(\"Th?r?se\")[0 + last()]
    // Expected: 101
    // Calls stub function - will fail until fn:string-to-codepoints is implemented
    let _ = stub_fnstring_to_codepoints();
}

#[test]
fn k_stringtocodepointfunc_16() {
    //   Purpose: Combine string-to-codepoints() and a predicate.
    // XPath: string-to-codepoints(\"Th?r?se\")[last() - 1]
    // Expected: 115
    // Calls stub function - will fail until fn:string-to-codepoints is implemented
    let _ = stub_fnstring_to_codepoints();
}

#[test]
fn k_stringtocodepointfunc_17() {
    //   Purpose: Combine string-to-codepoints() and a predicate.
    // XPath: string-to-codepoints(\"Th?r?se\")[last() - 0]
    // Expected: 101
    // Calls stub function - will fail until fn:string-to-codepoints is implemented
    let _ = stub_fnstring_to_codepoints();
}

#[test]
fn k_stringtocodepointfunc_18() {
    //   Purpose: Combine string-to-codepoints() and a predicate.
    // XPath: string-to-codepoints(\"Th?r?se\")[0 + last()]
    // Expected: 101
    // Calls stub function - will fail until fn:string-to-codepoints is implemented
    let _ = stub_fnstring_to_codepoints();
}

#[test]
fn k_stringtocodepointfunc_19() {
    //   Purpose: Combine string-to-codepoints() and a predicate.
    // XPath: string-to-codepoints(\"Th?r?se\")[last() - 2]
    // Expected: 232
    // Calls stub function - will fail until fn:string-to-codepoints is implemented
    let _ = stub_fnstring_to_codepoints();
}

#[test]
fn k_stringtocodepointfunc_20() {
    //   Purpose: Combine string-to-codepoints() and a predicate.
    // XPath: empty(string-to-codepoints(\"Th?r?se\")[last() - 7])
    // Expected: true
    // Calls stub function - will fail until fn:string-to-codepoints is implemented
    let _ = stub_fnstring_to_codepoints();
}

#[test]
fn k_stringtocodepointfunc_21() {
    //   Purpose: Combine string-to-codepoints() and a predicate.
    // XPath: string-to-codepoints(\"Th?r?se\")[last() - 6]
    // Expected: 84
    // Calls stub function - will fail until fn:string-to-codepoints is implemented
    let _ = stub_fnstring_to_codepoints();
}
