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
//! ## Usage
//!
//! The JSON type is parameterized by size. Use the type aliases and call `parse_json` or
//! `parse_json_from_string` directly on them:
//!
//! ```noir
//! use xpath::json::JSON512b;
//!
//! let json: JSON512b = JSON512b::parse_json_from_string("[1, 2, 3]");
//! assert(json.get_length() == 3);
//! ```
//!
//! Available size variants:
//! - JSON512b: up to 512 bytes, 64 tokens, 32 values
//! - JSON1kb: up to 1kb, 128 tokens, 64 values
//! - JSON2kb: up to 2kb, 256 tokens, 128 values
//! - JSON4kb: up to 4kb, 512 tokens, 256 values
//! - JSON8kb: up to 8kb, 1024 tokens, 512 values
//! - JSON16kb: up to 16kb, 2048 tokens, 1024 values
//!
//! ## Note on Public API
//!
//! The noir_json_parser library marks key-based accessor methods as `pub(crate)`,
//! so they are not accessible from external crates. Only array-based accessors and
//! parsing methods are publicly available. Users should use the JSON types directly
//! and call methods on them:
//!
//! Available methods on JSON types:
//! - `parse_json(bytes)` / `parse_json_from_string(str)` - parse JSON
//! - `get_length()` - get array length
//! - `get_object_from_array(index)` / `get_object_from_array_unchecked(index)`
//! - `get_array_from_array(index)` / `get_array_from_array_unchecked(index)`
//! - `get_string_from_array(index)` / `get_string_from_array_unchecked(index)`
//! - `get_number_from_array(index)` / `get_number_from_array_unchecked(index)`
//! - `get_literal_from_array(index)` / `get_literal_from_array_unchecked(index)`

// Re-export the size aliases from json_parser
pub use json_parser::JSON16kb;
pub use json_parser::JSON1kb;
pub use json_parser::JSON2kb;
pub use json_parser::JSON4kb;
pub use json_parser::JSON512b;
pub use json_parser::JSON8kb;

// ============================================================================
// XPath fn:parse-json and fn:json-doc
// ============================================================================

// Since the JSON type is parameterized and the generic type is not publicly exported,
// users should call the parse methods directly on the type aliases:
//
//   let json: JSON512b = JSON512b::parse_json_from_string("[1, 2, 3]");
//   let json: JSON1kb = JSON1kb::parse_json(my_bytes);
//
// The `json_doc` function in XPath is equivalent to `parse_json` in this context
// since we don't have file system access - users simply parse JSON directly.

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
fn test_json_1kb_size() {
    // Test that JSON1kb works for larger JSON
    let text = "[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]";
    let json: JSON1kb = JSON1kb::parse_json_from_string(text);

    assert(json.get_length() == 10);
    assert(json.get_number_from_array_unchecked(9) == 10);
}
