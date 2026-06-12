//! Unit tests for XPath string functions
//!
//! NOTE: Due to Noir's limitations, only functions that return boolean or numeric
//! values can be tested. Functions that create new strings (substring, upper_case,
//! lower_case, concat, etc.) cannot work correctly in Noir as there's no runtime
//! byte-to-string conversion.

use dep::xpath::{
    contains, contains_token, ends_with, fn_string_from_boolean, fn_string_from_integer,
    starts_with, string_length, tokenize_whitespace,
};

// Tests for string_length (fn:string-length / STRLEN)
// This works because it returns a number, not a string
#[test]
fn test_string_length_empty() {
    let s: str<0> = "";
    assert(string_length::<0>(s) == 0);
}

#[test]
fn test_string_length_word() {
    let s: str<5> = "hello";
    assert(string_length::<5>(s) == 5);
}

#[test]
fn test_string_length_phrase() {
    let s: str<11> = "Hello World";
    assert(string_length::<11>(s) == 11);
}

// Tests for starts_with (fn:starts-with / STRSTARTS)
// This works because it returns a boolean
#[test]
fn test_starts_with_true() {
    let s: str<11> = "Hello World";
    let prefix: str<5> = "Hello";
    assert(starts_with::<11, 5>(s, prefix));
}

#[test]
fn test_starts_with_false() {
    let s: str<11> = "Hello World";
    let prefix: str<5> = "World";
    assert(!starts_with::<11, 5>(s, prefix));
}

#[test]
fn test_starts_with_partial() {
    let s: str<7> = "testing";
    let prefix: str<4> = "test";
    assert(starts_with::<7, 4>(s, prefix));
}

// Tests for ends_with (fn:ends-with / STRENDS)
// This works because it returns a boolean
#[test]
fn test_ends_with_true() {
    let s: str<11> = "Hello World";
    let suffix: str<5> = "World";
    assert(ends_with::<11, 5>(s, suffix));
}

#[test]
fn test_ends_with_false() {
    let s: str<11> = "Hello World";
    let suffix: str<5> = "Hello";
    assert(!ends_with::<11, 5>(s, suffix));
}

#[test]
fn test_ends_with_partial() {
    let s: str<7> = "testing";
    let suffix: str<3> = "ing";
    assert(ends_with::<7, 3>(s, suffix));
}

// Tests for contains (fn:contains / CONTAINS)
// This works because it returns a boolean
#[test]
fn test_contains_middle() {
    let s: str<11> = "Hello World";
    let sub: str<5> = "lo Wo";
    assert(contains::<11, 5>(s, sub));
}

#[test]
fn test_contains_beginning() {
    let s: str<11> = "Hello World";
    let sub: str<5> = "Hello";
    assert(contains::<11, 5>(s, sub));
}

#[test]
fn test_contains_end() {
    let s: str<11> = "Hello World";
    let sub: str<5> = "World";
    assert(contains::<11, 5>(s, sub));
}

#[test]
fn test_contains_not_found() {
    let s: str<11> = "Hello World";
    let sub: str<3> = "xyz";
    assert(!contains::<11, 3>(s, sub));
}

// ============================================================================
// Tests for contains_token (fn:contains-token)
// ============================================================================

#[test]
fn test_contains_token_single_word() {
    let s: str<5> = "hello";
    let token: str<5> = "hello";
    assert(contains_token(s, token));
}

#[test]
fn test_contains_token_first_word() {
    let s: str<11> = "hello world";
    let token: str<5> = "hello";
    assert(contains_token(s, token));
}

#[test]
fn test_contains_token_last_word() {
    let s: str<11> = "hello world";
    let token: str<5> = "world";
    assert(contains_token(s, token));
}

#[test]
fn test_contains_token_middle_word() {
    let s: str<15> = "one two three  ";
    let token: str<3> = "two";
    assert(contains_token(s, token));
}

#[test]
fn test_contains_token_not_found() {
    let s: str<11> = "hello world";
    let token: str<3> = "foo";
    assert(!contains_token(s, token));
}

#[test]
fn test_contains_token_partial_no_match() {
    // "hell" is not a token, "hello" is
    let s: str<11> = "hello world";
    let token: str<4> = "hell";
    assert(!contains_token(s, token));
}

#[test]
fn test_contains_token_empty_token() {
    let s: str<11> = "hello world";
    let token: str<0> = "";
    assert(!contains_token(s, token)); // Empty token never matches
}

// ============================================================================
// Tests for tokenize_whitespace (fn:tokenize)
// ============================================================================

#[test]
fn test_tokenize_whitespace_simple() {
    let s: str<11> = "hello world";
    let (tokens, lens, count) = tokenize_whitespace::<11, 5, 10>(s);

    assert(count == 2);
    assert(lens[0] == 5); // "hello"
    assert(lens[1] == 5); // "world"

    // Check first token bytes: "hello"
    assert(tokens[0][0] == 104); // 'h'
    assert(tokens[0][1] == 101); // 'e'
    assert(tokens[0][2] == 108); // 'l'
    assert(tokens[0][3] == 108); // 'l'
    assert(tokens[0][4] == 111); // 'o'
}

#[test]
fn test_tokenize_whitespace_multiple_spaces() {
    let s: str<14> = "one  two three";
    let (tokens, lens, count) = tokenize_whitespace::<14, 5, 10>(s);

    assert(count == 3);
    assert(lens[0] == 3); // "one"
    assert(lens[1] == 3); // "two"
    assert(lens[2] == 5); // "three"
}

#[test]
fn test_tokenize_whitespace_single_word() {
    let s: str<5> = "hello";
    let (tokens, lens, count) = tokenize_whitespace::<5, 5, 10>(s);

    assert(count == 1);
    assert(lens[0] == 5);
}

#[test]
fn test_tokenize_whitespace_empty_string() {
    let s: str<1> = " ";
    let (_tokens, _lens, count) = tokenize_whitespace::<1, 5, 10>(s);

    assert(count == 0);
}

// ============================================================================
// Tests for fn_string_from_integer (fn:string)
// ============================================================================

#[test]
fn test_fn_string_from_integer_zero() {
    let (result, len) = fn_string_from_integer::<5>(0);
    assert(len == 1);
    assert(result[0] == 48); // '0'
}

#[test]
fn test_fn_string_from_integer_positive() {
    let (result, len) = fn_string_from_integer::<10>(12345);
    assert(len == 5);
    assert(result[0] == 49); // '1'
    assert(result[1] == 50); // '2'
    assert(result[2] == 51); // '3'
    assert(result[3] == 52); // '4'
    assert(result[4] == 53); // '5'
}

#[test]
fn test_fn_string_from_integer_negative() {
    let (result, len) = fn_string_from_integer::<10>(-42);
    assert(len == 3);
    assert(result[0] == 45); // '-'
    assert(result[1] == 52); // '4'
    assert(result[2] == 50); // '2'
}

#[test]
fn test_fn_string_from_integer_single_digit() {
    let (result, len) = fn_string_from_integer::<5>(7);
    assert(len == 1);
    assert(result[0] == 55); // '7'
}

// ============================================================================
// Tests for fn_string_from_boolean (fn:string)
// ============================================================================

#[test]
fn test_fn_string_from_boolean_true() {
    let (result, len) = fn_string_from_boolean::<10>(true);
    assert(len == 4);
    assert(result[0] == 116); // 't'
    assert(result[1] == 114); // 'r'
    assert(result[2] == 117); // 'u'
    assert(result[3] == 101); // 'e'
}

#[test]
fn test_fn_string_from_boolean_false() {
    let (result, len) = fn_string_from_boolean::<10>(false);
    assert(len == 5);
    assert(result[0] == 102); // 'f'
    assert(result[1] == 97); // 'a'
    assert(result[2] == 108); // 'l'
    assert(result[3] == 115); // 's'
    assert(result[4] == 101); // 'e'
}

// NOTE: Tests for substring, upper_case, lower_case, concat, substring_before,
// substring_after are not included because these functions cannot work correctly
// in Noir due to the inability to convert byte arrays back to strings at runtime.
