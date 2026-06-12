//! Test chunk 0 for fn:sort
//! Contains 23 tests

use dep::xpath::stub_fnsort;

#[test]
fn fn_sort_1() {
    // Empty sequence as input gives empty sequence as output
    // XPath: fn:empty(fn:sort( () ))
    // Expected: true
    // Calls stub function - will fail until fn:sort is implemented
    let _ = stub_fnsort();
}

#[test]
fn fn_sort_2() {
    // Simple sequence of integers
    // XPath: let $r := fn:sort( (1, 2, 3) ) return (count($r) eq 3 and $r
    // Expected: true
    // Calls stub function - will fail until fn:sort is implemented
    let _ = stub_fnsort();
}

#[test]
fn fn_sort_3() {
    // Simple sequence of integers, descending
    // XPath: let $r := fn:sort( (3, 2, 1) ) return (count($r) eq 3 and $r
    // Expected: true
    // Calls stub function - will fail until fn:sort is implemented
    let _ = stub_fnsort();
}

#[test]
fn fn_sort_4() {
    // Singleton integer
    // XPath: let $r := fn:sort( (1) ) return (count($r) eq 1 and $r[1] eq
    // Expected: true
    // Calls stub function - will fail until fn:sort is implemented
    let _ = stub_fnsort();
}

#[test]
fn fn_sort_5() {
    // Singleton integer not expressed as sequence
    // XPath: fn:sort(42) = 42
    // Expected: true
    // Calls stub function - will fail until fn:sort is implemented
    let _ = stub_fnsort();
}

#[test]
fn fn_sort_6() {
    // Length 1
    // XPath: let $r := fn:sort( (1) ) return (count($r) eq 1 and $r[1] eq
    // Expected: true
    // Calls stub function - will fail until fn:sort is implemented
    let _ = stub_fnsort();
}

#[test]
fn fn_sort_7() {
    // length 2
    // XPath: let $r := fn:sort( (1, 17) ) return (count($r) eq 2 and $r[1
    // Expected: true
    // Calls stub function - will fail until fn:sort is implemented
    let _ = stub_fnsort();
}

#[test]
fn fn_sort_8() {
    // length 2, disordered
    // XPath: let $r := fn:sort( (17, 1) ) return (count($r) eq 2 and $r[1
    // Expected: true
    // Calls stub function - will fail until fn:sort is implemented
    let _ = stub_fnsort();
}

#[test]
fn fn_sort_9() {
    // length 2, duplicates
    // XPath: let $r := fn:sort( (17, 17) ) return (count($r) eq 2 and $r[
    // Expected: true
    // Calls stub function - will fail until fn:sort is implemented
    let _ = stub_fnsort();
}

#[test]
fn fn_sort_10() {
    // length 3, disordered
    // XPath: let $r := fn:sort( (17, 5, 14) ) return (count($r) eq 3 and
    // Expected: true
    // Calls stub function - will fail until fn:sort is implemented
    let _ = stub_fnsort();
}

#[test]
fn fn_sort_11() {
    // length 3, duplicates
    // XPath: let $r := fn:sort( (4, 4, 4) ) return (count($r) eq 3 and $r
    // Expected: true
    // Calls stub function - will fail until fn:sort is implemented
    let _ = stub_fnsort();
}

#[test]
fn fn_sort_12() {
    // length 3, descending
    // XPath: let $r := fn:sort( (17, 14, 5) ) return (count($r) eq 3 and
    // Expected: true
    // Calls stub function - will fail until fn:sort is implemented
    let _ = stub_fnsort();
}

#[test]
fn fn_sort_13() {
    // length 6
    // XPath: let $r := fn:sort( (-42, 14, 5, 6, 14, 0) ) return (count($r
    // Expected: true
    // Calls stub function - will fail until fn:sort is implemented
    let _ = stub_fnsort();
}

#[test]
fn fn_sort_15() {
    // One-argument sort where input is not simple
    // XPath: deep-equal(             sort(               (
    // Expected: true
    // Calls stub function - will fail until fn:sort is implemented
    let _ = stub_fnsort();
}

#[test]
fn fn_sort_16() {
    // One-argument sort where input is a mix of simple and complex
    // XPath: deep-equal(          sort((\"b\", \"d\", <e>a</e>, \"c\", <f
    // Expected: true
    // Calls stub function - will fail until fn:sort is implemented
    let _ = stub_fnsort();
}

#[test]
fn fn_sort_frac_1() {
    // numeric but not all integers
    // XPath: let $r := fn:sort( (-4.2, 14.25, 5.99, 6, 14.24, 0) ) return
    // Expected: true
    // Calls stub function - will fail until fn:sort is implemented
    let _ = stub_fnsort();
}

#[test]
fn fn_sort_str_1() {
    // Simple sequence of strings
    // XPath: let $r := fn:sort( (\"boy\",\"for\",\"new\",\"chosen\",\"bla
    // Expected: true
    // Calls stub function - will fail until fn:sort is implemented
    let _ = stub_fnsort();
}

#[test]
fn fn_sort2_str_1() {
    // Simple sequence of strings with fn
    // XPath: let $r := fn:sort( (\"boy\",\"for\",\"new\",\"chosen\",\"bla
    // Expected: true
    // Calls stub function - will fail until fn:sort is implemented
    let _ = stub_fnsort();
}

#[test]
fn fn_sort2_str_2() {
    // Simple sequence of strings with user-defined fn
    // XPath: let $r := fn:sort( (\"boy\",\"for\",\"new\",\"chosen\",\"bla
    // Expected: true
    // Calls stub function - will fail until fn:sort is implemented
    let _ = stub_fnsort();
}

#[test]
fn fn_sort_spec_1() {
    // spec example 1
    // XPath: let $r := fn:sort((1, 4, 6, 5, 3), (), fn:data#1 ) return (c
    // Expected: true
    // Calls stub function - will fail until fn:sort is implemented
    let _ = stub_fnsort();
}

#[test]
fn fn_sort_spec_1b() {
    // spec example 1 but check numeric, not lexical
    // XPath: let $r := fn:sort((10, 4, 6, 5, 3), (), fn:data#1 ) return (
    // Expected: true
    // Calls stub function - will fail until fn:sort is implemented
    let _ = stub_fnsort();
}

#[test]
fn fn_sort_spec_2() {
    // spec example 2
    // XPath: let $r := fn:sort((1, -2, 5, 10, -10, 10, 8), (), fn:abs#1)
    // Expected: true
    // Calls stub function - will fail until fn:sort is implemented
    let _ = stub_fnsort();
}

#[test]
fn fn_sort_spec_4() {
    // spec example 4
    // XPath: let    $employees := (      <emp><name><first>Reginald</firs
    // Expected: true
    // Calls stub function - will fail until fn:sort is implemented
    let _ = stub_fnsort();
}
