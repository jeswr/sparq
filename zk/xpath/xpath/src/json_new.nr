//! JSON parsing functions for XPath/SPARQL
//!
//! This module wraps the noir_json_parser library to provide JSON parsing capabilities.
//!
//! ## XPath Functions
//!
//! This module implements:
//! - fn:parse-json - Parse a JSON string into a structured representation
//! - fn:json-doc - Parse JSON from a document (simplified to parse-json)
//!
//! The library provides different size variants: JSON512b, JSON1kb, JSON2kb, JSON4kb, JSON8kb.
//!
//! ## Note on Public API
//!
//! The noir_json_parser library marks key-based accessor methods as `pub(crate)`,
//! so they are not accessible from external crates. Only array-based accessors and
//! parsing methods are publicly available. Users should use the JSON types directly
//! and call methods on them.

// Re-export types from json_parser
pub use json_parser::JSON1kb;
pub use json_parser::JSON2kb;
pub use json_parser::JSON4kb;
pub use json_parser::JSON512b;
pub use json_parser::JSON8kb;
pub use json_parser::JSONLiteral;

// ============================================================================
// Parsing functions
// ============================================================================

/// Parse a JSON string into a JSON512b structure (up to 512 bytes)
pub fn parse_json_512b<let N: u32>(text: [u8; N]) -> JSON512b {
    JSON512b::parse_json(text)
}

/// Parse a JSON string from a string literal into a JSON512b structure
pub fn parse_json_512b_from_string<let N: u32>(text: str<N>) -> JSON512b {
    JSON512b::parse_json_from_string(text)
}

/// Parse a JSON string into a JSON1kb structure (up to 1kb)
pub fn parse_json_1kb<let N: u32>(text: [u8; N]) -> JSON1kb {
    JSON1kb::parse_json(text)
}

/// Parse a JSON string from a string literal into a JSON1kb structure
pub fn parse_json_1kb_from_string<let N: u32>(text: str<N>) -> JSON1kb {
    JSON1kb::parse_json_from_string(text)
}

/// Parse a JSON string into a JSON2kb structure (up to 2kb)
pub fn parse_json_2kb<let N: u32>(text: [u8; N]) -> JSON2kb {
    JSON2kb::parse_json(text)
}

/// Parse a JSON string from a string literal into a JSON2kb structure
pub fn parse_json_2kb_from_string<let N: u32>(text: str<N>) -> JSON2kb {
    JSON2kb::parse_json_from_string(text)
}

/// Parse a JSON string into a JSON4kb structure (up to 4kb)
pub fn parse_json_4kb<let N: u32>(text: [u8; N]) -> JSON4kb {
    JSON4kb::parse_json(text)
}

/// Parse a JSON string from a string literal into a JSON4kb structure
pub fn parse_json_4kb_from_string<let N: u32>(text: str<N>) -> JSON4kb {
    JSON4kb::parse_json_from_string(text)
}

/// Parse a JSON string into a JSON8kb structure (up to 8kb)
pub fn parse_json_8kb<let N: u32>(text: [u8; N]) -> JSON8kb {
    JSON8kb::parse_json(text)
}

/// Parse a JSON string from a string literal into a JSON8kb structure
pub fn parse_json_8kb_from_string<let N: u32>(text: str<N>) -> JSON8kb {
    JSON8kb::parse_json_from_string(text)
}

// ============================================================================
// Array element access functions (these are public in json_parser)
// ============================================================================

/// Get an object from a JSON512b array at a given index
pub fn json_get_object_from_array_512b(json: JSON512b, index: Field) -> Option<JSON512b> {
    json.get_object_from_array(index)
}

/// Get an object from a JSON1kb array at a given index
pub fn json_get_object_from_array_1kb(json: JSON1kb, index: Field) -> Option<JSON1kb> {
    json.get_object_from_array(index)
}

/// Get an object from a JSON512b array at a given index (unchecked)
pub fn json_get_object_from_array_unchecked_512b(json: JSON512b, index: Field) -> JSON512b {
    json.get_object_from_array_unchecked(index)
}

/// Get an object from a JSON1kb array at a given index (unchecked)
pub fn json_get_object_from_array_unchecked_1kb(json: JSON1kb, index: Field) -> JSON1kb {
    json.get_object_from_array_unchecked(index)
}

/// Get an array from a JSON512b array at a given index
pub fn json_get_array_from_array_512b(json: JSON512b, index: Field) -> Option<JSON512b> {
    json.get_array_from_array(index)
}

/// Get an array from a JSON1kb array at a given index
pub fn json_get_array_from_array_1kb(json: JSON1kb, index: Field) -> Option<JSON1kb> {
    json.get_array_from_array(index)
}

/// Get an array from a JSON512b array at a given index (unchecked)
pub fn json_get_array_from_array_unchecked_512b(json: JSON512b, index: Field) -> JSON512b {
    json.get_array_from_array_unchecked(index)
}

/// Get an array from a JSON1kb array at a given index (unchecked)
pub fn json_get_array_from_array_unchecked_1kb(json: JSON1kb, index: Field) -> JSON1kb {
    json.get_array_from_array_unchecked(index)
}

/// Get a string from a JSON512b array at a given index
pub fn json_get_string_from_array_512b<let MAX_LEN: u32>(
    json: JSON512b,
    index: Field,
) -> Option<BoundedVec<u8, MAX_LEN>> {
    json.get_string_from_array(index)
}

/// Get a string from a JSON1kb array at a given index
pub fn json_get_string_from_array_1kb<let MAX_LEN: u32>(
    json: JSON1kb,
    index: Field,
) -> Option<BoundedVec<u8, MAX_LEN>> {
    json.get_string_from_array(index)
}

/// Get a string from a JSON512b array at a given index (unchecked)
pub fn json_get_string_from_array_unchecked_512b<let MAX_LEN: u32>(
    json: JSON512b,
    index: Field,
) -> BoundedVec<u8, MAX_LEN> {
    json.get_string_from_array_unchecked(index)
}

/// Get a string from a JSON1kb array at a given index (unchecked)
pub fn json_get_string_from_array_unchecked_1kb<let MAX_LEN: u32>(
    json: JSON1kb,
    index: Field,
) -> BoundedVec<u8, MAX_LEN> {
    json.get_string_from_array_unchecked(index)
}

/// Get a number from a JSON512b array at a given index
pub fn json_get_number_from_array_512b(json: JSON512b, index: Field) -> Option<u64> {
    json.get_number_from_array(index)
}

/// Get a number from a JSON1kb array at a given index
pub fn json_get_number_from_array_1kb(json: JSON1kb, index: Field) -> Option<u64> {
    json.get_number_from_array(index)
}

/// Get a number from a JSON512b array at a given index (unchecked)
pub fn json_get_number_from_array_unchecked_512b(json: JSON512b, index: Field) -> u64 {
    json.get_number_from_array_unchecked(index)
}

/// Get a number from a JSON1kb array at a given index (unchecked)
pub fn json_get_number_from_array_unchecked_1kb(json: JSON1kb, index: Field) -> u64 {
    json.get_number_from_array_unchecked(index)
}

/// Get a literal from a JSON512b array at a given index
pub fn json_get_literal_from_array_512b(json: JSON512b, index: Field) -> Option<JSONLiteral> {
    json.get_literal_from_array(index)
}

/// Get a literal from a JSON1kb array at a given index
pub fn json_get_literal_from_array_1kb(json: JSON1kb, index: Field) -> Option<JSONLiteral> {
    json.get_literal_from_array(index)
}

/// Get a literal from a JSON512b array at a given index (unchecked)
pub fn json_get_literal_from_array_unchecked_512b(json: JSON512b, index: Field) -> JSONLiteral {
    json.get_literal_from_array_unchecked(index)
}

/// Get a literal from a JSON1kb array at a given index (unchecked)
pub fn json_get_literal_from_array_unchecked_1kb(json: JSON1kb, index: Field) -> JSONLiteral {
    json.get_literal_from_array_unchecked(index)
}

// ============================================================================
// Array length functions
// ============================================================================

/// Get the length of a JSON512b array
pub fn json_array_length_512b(json: JSON512b) -> u32 {
    json.get_length()
}

/// Get the length of a JSON1kb array
pub fn json_array_length_1kb(json: JSON1kb) -> u32 {
    json.get_length()
}

// ============================================================================
// Tests
// ============================================================================

#[test]
fn test_parse_json_array() {
    let text = "[1, 2, 3, 4, 5]";
    let json: JSON512b = JSON512b::parse_json_from_string(text);

    assert(json.get_length() == 5);
    assert(json.get_number_from_array_unchecked(0) == 1);
    assert(json.get_number_from_array_unchecked(1) == 2);
    assert(json.get_number_from_array_unchecked(2) == 3);
    assert(json.get_number_from_array_unchecked(3) == 4);
    assert(json.get_number_from_array_unchecked(4) == 5);
}

#[test]
fn test_parse_json_string_array() {
    let text = "[\"hello\", \"world\"]";
    let json: JSON512b = JSON512b::parse_json_from_string(text);

    assert(json.get_length() == 2);
    let s1: BoundedVec<u8, 10> = json.get_string_from_array_unchecked(0);
    let s2: BoundedVec<u8, 10> = json.get_string_from_array_unchecked(1);
    assert(s1.len() == 5);
    assert(s2.len() == 5);
}

#[test]
fn test_parse_json_nested_array() {
    let text = "[[1, 2], [3, 4]]";
    let json: JSON512b = JSON512b::parse_json_from_string(text);

    assert(json.get_length() == 2);

    let inner1 = json.get_array_from_array_unchecked(0);
    assert(inner1.get_length() == 2);
    assert(inner1.get_number_from_array_unchecked(0) == 1);
    assert(inner1.get_number_from_array_unchecked(1) == 2);

    let inner2 = json.get_array_from_array_unchecked(1);
    assert(inner2.get_length() == 2);
    assert(inner2.get_number_from_array_unchecked(0) == 3);
    assert(inner2.get_number_from_array_unchecked(1) == 4);
}

#[test]
fn test_parse_json_array_of_objects() {
    let text = "[{\"a\": 1}, {\"b\": 2}]";
    let json: JSON512b = JSON512b::parse_json_from_string(text);

    assert(json.get_length() == 2);

    let _obj1 = json.get_object_from_array_unchecked(0);
    let _obj2 = json.get_object_from_array_unchecked(1);
}

#[test]
fn test_parse_json_literal_array() {
    let text = "[true, false, null]";
    let json: JSON512b = JSON512b::parse_json_from_string(text);

    assert(json.get_length() == 3);

    let lit1 = json.get_literal_from_array_unchecked(0);
    let lit2 = json.get_literal_from_array_unchecked(1);
    let lit3 = json.get_literal_from_array_unchecked(2);

    assert(lit1.is_true());
    assert(lit2.is_false());
    assert(lit3.is_null());
}
