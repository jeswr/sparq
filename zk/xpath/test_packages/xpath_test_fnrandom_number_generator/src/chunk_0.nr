//! Test chunk 0 for fn:random-number-generator
//! Contains 18 tests

use dep::xpath::stub_fnrandom_number_generator;

#[test]
fn fn_random_number_generator_3() {
    // Test randomly generated number is greater than 0
    // XPath: random-number-generator()?number ge 0
    // Expected: true
    // Calls stub function - will fail until fn:random-number-generator is implemented
    let _ = stub_fnrandom_number_generator();
}

#[test]
fn fn_random_number_generator_4() {
    // Test randomly generated number is in correct range; variable reference
    // XPath: let $r := random-number-generator()?number return ($r ge 0 a
    // Expected: true
    // Calls stub function - will fail until fn:random-number-generator is implemented
    let _ = stub_fnrandom_number_generator();
}

#[test]
fn fn_random_number_generator_5() {
    // Test randomly generated number is in correct range; variable reference
    // XPath: let $r := random-number-generator()?number return ($r lt 0 o
    // Expected: false
    // Calls stub function - will fail until fn:random-number-generator is implemented
    let _ = stub_fnrandom_number_generator();
}

#[test]
fn fn_random_number_generator_8() {
    // 'next' function and 'number' generator; variable reference
    // XPath: let $r := fn:random-number-generator()?next()?number return
    // Expected: true
    // Calls stub function - will fail until fn:random-number-generator is implemented
    let _ = stub_fnrandom_number_generator();
}

#[test]
fn fn_random_number_generator_9() {
    // 'next' function and 'number' generator; variable reference
    // XPath: let $r := fn:random-number-generator()?next()?number return
    // Expected: false
    // Calls stub function - will fail until fn:random-number-generator is implemented
    let _ = stub_fnrandom_number_generator();
}

#[test]
fn fn_random_number_generator_12() {
    // Random number generator with seed, 'number' generator
    // XPath: random-number-generator(3147)?number ge 0
    // Expected: true
    // Calls stub function - will fail until fn:random-number-generator is implemented
    let _ = stub_fnrandom_number_generator();
}

#[test]
fn fn_random_number_generator_13() {
    // Random number generator with seed, check number is in correct range
    // XPath: let $r := random-number-generator(\"random\")?number return
    // Expected: true
    // Calls stub function - will fail until fn:random-number-generator is implemented
    let _ = stub_fnrandom_number_generator();
}

#[test]
fn fn_random_number_generator_16() {
    // Random number generator with seed, 'next' function and 'number' generator
    // XPath: let $r := fn:random-number-generator()?next()?number return
    // Expected: true
    // Calls stub function - will fail until fn:random-number-generator is implemented
    let _ = stub_fnrandom_number_generator();
}

#[test]
fn fn_random_number_generator_22() {
    // Detect randomness
    // XPath: declare function local:random-sequence($length as xs:integer
    // Expected: true
    // Calls stub function - will fail until fn:random-number-generator is implemented
    let _ = stub_fnrandom_number_generator();
}

#[test]
fn fn_random_number_generator_23() {
    // Highly unlikely that two random number generators produce the same result
    // XPath: let $G := fn:random-number-generator(fn:current-dateTime())
    // Expected: false
    // Calls stub function - will fail until fn:random-number-generator is implemented
    let _ = stub_fnrandom_number_generator();
}

#[test]
fn fn_random_number_generator_24() {
    // The arity-1 function is deterministic
    // XPath: deep-equal(fn:random-number-generator(fn:current-dateTime())
    // Expected: true
    // Calls stub function - will fail until fn:random-number-generator is implemented
    let _ = stub_fnrandom_number_generator();
}

#[test]
fn fn_random_number_generator_25() {
    // The arity-0 function is deterministic
    // XPath: deep-equal(fn:random-number-generator()?permute(1 to 100), f
    // Expected: true
    // Calls stub function - will fail until fn:random-number-generator is implemented
    let _ = stub_fnrandom_number_generator();
}

#[test]
fn fn_random_number_generator_26() {
    // The choice of seed makes a difference
    // XPath: deep-equal(fn:random-number-generator(0)?permute(1 to 100),
    // Expected: false
    // Calls stub function - will fail until fn:random-number-generator is implemented
    let _ = stub_fnrandom_number_generator();
}

#[test]
fn fn_random_number_generator_27() {
    // NaN is a valid seed
    // XPath: fn:random-number-generator(xs:double('NaN'))?permute(1)
    // Expected: 1
    // Calls stub function - will fail until fn:random-number-generator is implemented
    let _ = stub_fnrandom_number_generator();
}

#[test]
fn fn_random_number_generator_31() {
    // Use the map as function item
    // XPath: declare function local:test($f as function(*)) as xs:integer
    // Expected: 0
    // Calls stub function - will fail until fn:random-number-generator is implemented
    let _ = stub_fnrandom_number_generator();
}

#[test]
fn fn_random_number_generator_34() {
    // Test that random-number-generator with empty sequence is equivalent to unary fun
    // XPath: random-number-generator()?number eq               random-num
    // Expected: true
    // Calls stub function - will fail until fn:random-number-generator is implemented
    let _ = stub_fnrandom_number_generator();
}

#[test]
fn fn_random_number_generator_39() {
    // Test that random-number-generator with empty sequence is equivalent to unary fun
    // XPath: fn:random-number-generator()?next()?number eq
    // Expected: true
    // Calls stub function - will fail until fn:random-number-generator is implemented
    let _ = stub_fnrandom_number_generator();
}

#[test]
fn fn_random_number_generator_43() {
    // Permute sequence of integers
    // XPath: deep-equal(fn:random-number-generator()?permute(1 to 100),
    // Expected: true
    // Calls stub function - will fail until fn:random-number-generator is implemented
    let _ = stub_fnrandom_number_generator();
}
