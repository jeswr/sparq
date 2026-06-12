//! XPath 2.0 String Functions
//!
//! This module implements XPath 2.0 string functions that work correctly in Noir.
//! These functions are required by SPARQL 1.1 Query Language.
//!
//! ## Working Functions
//!
//! The following functions work correctly and are exported:
//! - `string_length` - Returns the length as a number
//! - `starts_with` - Returns boolean
//! - `ends_with` - Returns boolean
//! - `contains` - Returns boolean
//!
//! ## Limitations
//!
//! **IMPORTANT**: Functions that need to create new strings from byte arrays
//! (substring, upper-case, lower-case, concat, etc.) cannot be implemented in Noir
//! because Noir doesn't provide a way to convert byte arrays back to strings at runtime.
//! These functions have been removed from this implementation.

/// fn:string-length - Returns the length of a string
/// XPath: fn:string-length($arg as xs:string?) as xs:integer
/// SPARQL: STRLEN(?str)
///
/// For fixed-size strings in Noir, this returns the actual length by counting
/// non-null bytes. If your strings always fill their capacity (no trailing nulls),
/// this will equal N.
pub fn string_length<let N: u32>(s: str<N>) -> u64 {
    let s_bytes = s.as_bytes();
    let mut len: u64 = 0;
    let mut ended = false;

    for i in 0..N {
        let byte = s_bytes[i];
        let is_zero = byte == 0;
        let increment = (!ended) & (!is_zero);
        len = len + (increment as u64);
        ended = ended | is_zero;
    }

    len
}

/// fn:starts-with - Returns true if a string starts with a given prefix
/// XPath: fn:starts-with($arg1 as xs:string?, $arg2 as xs:string?) as xs:boolean
/// SPARQL: STRSTARTS(?str, ?prefix)
pub fn starts_with<let N: u32, let M: u32>(s: str<N>, prefix: str<M>) -> bool {
    if M > N {
        false
    } else {
        let s_bytes = s.as_bytes();
        let prefix_bytes = prefix.as_bytes();
        let mut matches = true;

        for i in 0..M {
            if s_bytes[i] != prefix_bytes[i] {
                matches = false;
            }
        }

        matches
    }
}

/// fn:ends-with - Returns true if a string ends with a given suffix
/// XPath: fn:ends-with($arg1 as xs:string?, $arg2 as xs:string?) as xs:boolean
/// SPARQL: STRENDS(?str, ?suffix)
pub fn ends_with<let N: u32, let M: u32>(s: str<N>, suffix: str<M>) -> bool {
    if M > N {
        false
    } else {
        let s_bytes = s.as_bytes();
        let suffix_bytes = suffix.as_bytes();
        let start_pos = N - M;
        let mut matches = true;

        for i in 0..M {
            if s_bytes[start_pos + i] != suffix_bytes[i] {
                matches = false;
            }
        }

        matches
    }
}

/// Helper function to find index of substring
///
/// **Performance Note**: This function has O(N*M) time complexity and, due to Noir's
/// lack of early return/break, continues checking all positions even after finding a
/// match. For large strings, this could significantly impact proof generation time.
fn index_of<let N: u32, let M: u32>(s: str<N>, substring: str<M>) -> i64 {
    let mut result: i64 = -1;

    // Handle edge cases without early return
    let size_ok = M <= N;
    let empty_substring = M == 0;

    if empty_substring {
        result = 0;
    } else if size_ok {
        let s_bytes = s.as_bytes();
        let sub_bytes = substring.as_bytes();
        let mut found_index: i64 = -1;

        // Can't use break, so we iterate through all positions
        for i in 0..(N - M + 1) {
            let mut matches = true;
            for j in 0..M {
                if s_bytes[i + j] != sub_bytes[j] {
                    matches = false;
                }
            }
            // Only update if this is the first match
            if matches & (found_index == -1) {
                found_index = i as i64;
            }
        }
        result = found_index;
    }

    result
}

/// fn:contains - Returns true if a string contains a given substring
/// XPath: fn:contains($arg1 as xs:string?, $arg2 as xs:string?) as xs:boolean
/// SPARQL: CONTAINS(?str, ?substring)
pub fn contains<let N: u32, let M: u32>(s: str<N>, substring: str<M>) -> bool {
    index_of(s, substring) != -1
}

// ============================================================================
// String Comparison Functions
// ============================================================================

/// fn:compare - Compares two strings lexicographically
/// Returns -1 if s1 < s2, 0 if s1 == s2, 1 if s1 > s2
/// XPath: fn:compare($comparand1 as xs:string?, $comparand2 as xs:string?) as xs:integer?
pub fn compare<let N: u32, let M: u32>(s1: str<N>, s2: str<M>) -> i64 {
    let s1_bytes = s1.as_bytes();
    let s2_bytes = s2.as_bytes();

    // Determine the minimum length for comparison
    let min_len = if N < M { N } else { M };

    let mut result: i64 = 0;
    let mut determined = false;

    // Compare byte by byte
    for i in 0..min_len {
        if !determined {
            if s1_bytes[i] < s2_bytes[i] {
                result = -1;
                determined = true;
            } else if s1_bytes[i] > s2_bytes[i] {
                result = 1;
                determined = true;
            }
        }
    }

    // If all compared bytes are equal, shorter string comes first
    if !determined {
        if N < M {
            result = -1;
        } else if N > M {
            result = 1;
        }
        // else result stays 0 (strings are equal)
    }

    result
}

/// fn:codepoint-equal - Returns true if two strings are equal codepoint by codepoint
/// XPath: fn:codepoint-equal($comparand1 as xs:string?, $comparand2 as xs:string?) as xs:boolean?
/// This is equivalent to string equality for ASCII strings
pub fn codepoint_equal<let N: u32, let M: u32>(s1: str<N>, s2: str<M>) -> bool {
    if N != M {
        false
    } else {
        let s1_bytes = s1.as_bytes();
        let s2_bytes = s2.as_bytes();
        let mut equal = true;
        for i in 0..N {
            if s1_bytes[i] != s2_bytes[i] {
                equal = false;
            }
        }
        equal
    }
}

/// op:string-equal - Returns true if both strings are equal
/// XPath: op:string-equal($arg1 as xs:string, $arg2 as xs:string) as xs:boolean
pub fn string_equal<let N: u32, let M: u32>(s1: str<N>, s2: str<M>) -> bool {
    compare(s1, s2) == 0
}

/// op:string-less-than - Returns true if s1 is lexicographically less than s2
/// XPath: op:string-less-than($arg1 as xs:string, $arg2 as xs:string) as xs:boolean
pub fn string_less_than<let N: u32, let M: u32>(s1: str<N>, s2: str<M>) -> bool {
    compare(s1, s2) < 0
}

/// op:string-greater-than - Returns true if s1 is lexicographically greater than s2
/// XPath: op:string-greater-than($arg1 as xs:string, $arg2 as xs:string) as xs:boolean
pub fn string_greater_than<let N: u32, let M: u32>(s1: str<N>, s2: str<M>) -> bool {
    compare(s1, s2) > 0
}

/// String less than or equal
pub fn string_le<let N: u32, let M: u32>(s1: str<N>, s2: str<M>) -> bool {
    compare(s1, s2) <= 0
}

/// String greater than or equal
pub fn string_ge<let N: u32, let M: u32>(s1: str<N>, s2: str<M>) -> bool {
    compare(s1, s2) >= 0
}

// ============================================================================
// Codepoint Functions
// ============================================================================

/// fn:string-to-codepoints - Converts a string to a sequence of Unicode codepoints
/// XPath: fn:string-to-codepoints($arg as xs:string?) as xs:integer*
/// Note: For ASCII strings, each byte is a codepoint. Returns (array, length).
pub fn string_to_codepoints<let N: u32>(s: str<N>) -> ([u32; N], u32) {
    let s_bytes = s.as_bytes();
    let mut result: [u32; N] = [0; N];
    let mut len: u32 = 0;

    for i in 0..N {
        let byte = s_bytes[i];
        if byte != 0 {
            result[i] = byte as u32;
            len = (i + 1) as u32;
        }
    }

    (result, len)
}

// ============================================================================
// URI Comparison Functions (URIs are treated as strings)
// ============================================================================

/// op:anyURI-equal - Compare two URIs for equality
pub fn anyuri_equal<let N: u32, let M: u32>(uri1: str<N>, uri2: str<M>) -> bool {
    string_equal(uri1, uri2)
}

/// op:anyURI-less-than - Compare two URIs lexicographically
pub fn anyuri_less_than<let N: u32, let M: u32>(uri1: str<N>, uri2: str<M>) -> bool {
    string_less_than(uri1, uri2)
}

/// op:anyURI-greater-than - Compare two URIs lexicographically
pub fn anyuri_greater_than<let N: u32, let M: u32>(uri1: str<N>, uri2: str<M>) -> bool {
    string_greater_than(uri1, uri2)
}

// ============================================================================
// String Transformation Functions (return byte arrays)
// ============================================================================

/// fn:upper-case - Converts ASCII lowercase letters to uppercase
/// XPath: fn:upper-case($arg as xs:string?) as xs:string
/// Returns (byte_array, length)
pub fn upper_case<let N: u32>(s: str<N>) -> ([u8; N], u32) {
    let s_bytes = s.as_bytes();
    let mut result: [u8; N] = [0; N];
    let mut len: u32 = 0;

    for i in 0..N {
        let byte = s_bytes[i];
        if byte != 0 {
            // Convert a-z (97-122) to A-Z (65-90)
            if (byte >= 97) & (byte <= 122) {
                result[i] = byte - 32;
            } else {
                result[i] = byte;
            }
            len = (i + 1) as u32;
        }
    }

    (result, len)
}

/// fn:lower-case - Converts ASCII uppercase letters to lowercase
/// XPath: fn:lower-case($arg as xs:string?) as xs:string
/// Returns (byte_array, length)
pub fn lower_case<let N: u32>(s: str<N>) -> ([u8; N], u32) {
    let s_bytes = s.as_bytes();
    let mut result: [u8; N] = [0; N];
    let mut len: u32 = 0;

    for i in 0..N {
        let byte = s_bytes[i];
        if byte != 0 {
            // Convert A-Z (65-90) to a-z (97-122)
            if (byte >= 65) & (byte <= 90) {
                result[i] = byte + 32;
            } else {
                result[i] = byte;
            }
            len = (i + 1) as u32;
        }
    }

    (result, len)
}

/// fn:normalize-space - Strips leading/trailing whitespace and collapses internal whitespace
/// XPath: fn:normalize-space($arg as xs:string?) as xs:string
/// Returns (byte_array, length)
pub fn normalize_space<let N: u32>(s: str<N>) -> ([u8; N], u32) {
    let s_bytes = s.as_bytes();
    let mut result: [u8; N] = [0; N];
    let mut result_len: u32 = 0;
    let mut prev_was_space = true; // Start true to skip leading whitespace

    for i in 0..N {
        let byte = s_bytes[i];
        if byte != 0 {
            // Check if whitespace (space=32, tab=9, newline=10, carriage return=13)
            let is_whitespace = (byte == 32) | (byte == 9) | (byte == 10) | (byte == 13);

            if is_whitespace {
                // Only add space if previous wasn't space and we have content
                if !prev_was_space & (result_len > 0) {
                    if result_len < N {
                        result[result_len] = 32; // space
                        result_len += 1;
                    }
                }
                prev_was_space = true;
            } else {
                if result_len < N {
                    result[result_len] = byte;
                    result_len += 1;
                }
                prev_was_space = false;
            }
        }
    }

    // Remove trailing space if present
    if (result_len > 0) & (result[result_len - 1] == 32) {
        result[result_len - 1] = 0;
        result_len -= 1;
    }

    (result, result_len)
}

/// fn:translate - Replaces characters in a string based on a mapping
/// XPath: fn:translate($arg as xs:string?, $mapString as xs:string, $transString as xs:string) as xs:string
/// Each character in mapString is replaced with the corresponding character in transString
/// If transString is shorter, characters without a mapping are deleted
pub fn translate<let N: u32, let M: u32, let K: u32>(
    s: str<N>,
    map_string: str<M>,
    trans_string: str<K>,
) -> ([u8; N], u32) {
    let s_bytes = s.as_bytes();
    let map_bytes = map_string.as_bytes();
    let trans_bytes = trans_string.as_bytes();
    let mut result: [u8; N] = [0; N];
    let mut result_len: u32 = 0;

    for i in 0..N {
        let byte = s_bytes[i];
        if byte != 0 {
            // Find if this character is in map_string
            let mut found_idx: i64 = -1;
            for j in 0..M {
                if (map_bytes[j] != 0) & (map_bytes[j] == byte) & (found_idx == -1) {
                    found_idx = j as i64;
                }
            }

            if found_idx == -1 {
                // Not in map, keep original
                if result_len < N {
                    result[result_len] = byte;
                    result_len += 1;
                }
            } else {
                // Check if there's a corresponding character in trans_string
                let trans_idx = found_idx as u32;
                if (trans_idx < K) & (trans_bytes[trans_idx] != 0) {
                    if result_len < N {
                        result[result_len] = trans_bytes[trans_idx];
                        result_len += 1;
                    }
                }
                // If no corresponding char in trans_string, character is deleted
            }
        }
    }

    (result, result_len)
}

/// fn:concat - Concatenates two byte arrays
/// XPath: fn:concat($arg1 as xs:anyAtomicType?, $arg2 as xs:anyAtomicType?, ...) as xs:string
/// Returns (byte_array, length)
pub fn concat_bytes<let N: u32, let M: u32, let R: u32>(
    a: [u8; N],
    a_len: u32,
    b: [u8; M],
    b_len: u32,
) -> ([u8; R], u32) {
    let mut result: [u8; R] = [0; R];
    let mut result_len: u32 = 0;

    // Copy first array
    for i in 0..N {
        if ((i as u32) < a_len) & (result_len < R) {
            result[result_len] = a[i];
            result_len += 1;
        }
    }

    // Copy second array
    for i in 0..M {
        if ((i as u32) < b_len) & (result_len < R) {
            result[result_len] = b[i];
            result_len += 1;
        }
    }

    (result, result_len)
}

/// fn:string-join - Joins a sequence of strings with a separator
/// XPath: fn:string-join($arg1 as xs:anyAtomicType*, $arg2 as xs:string) as xs:string
/// This version joins two strings with a separator
pub fn string_join_two<let N: u32, let M: u32, let S: u32, let R: u32>(
    a: [u8; N],
    a_len: u32,
    b: [u8; M],
    b_len: u32,
    sep: [u8; S],
    sep_len: u32,
) -> ([u8; R], u32) {
    let mut result: [u8; R] = [0; R];
    let mut result_len: u32 = 0;

    // Copy first string
    for i in 0..N {
        if ((i as u32) < a_len) & (result_len < R) {
            result[result_len] = a[i];
            result_len += 1;
        }
    }

    // Copy separator (only if both strings are non-empty)
    if (a_len > 0) & (b_len > 0) {
        for i in 0..S {
            if ((i as u32) < sep_len) & (result_len < R) {
                result[result_len] = sep[i];
                result_len += 1;
            }
        }
    }

    // Copy second string
    for i in 0..M {
        if ((i as u32) < b_len) & (result_len < R) {
            result[result_len] = b[i];
            result_len += 1;
        }
    }

    (result, result_len)
}

/// fn:substring - Returns a substring starting at position (1-indexed)
/// XPath: fn:substring($sourceString as xs:string?, $start as xs:double) as xs:string
/// XPath: fn:substring($sourceString as xs:string?, $start as xs:double, $length as xs:double) as xs:string
pub fn substring<let N: u32, let R: u32>(s: str<N>, start_pos: i64, length: u32) -> ([u8; R], u32) {
    let s_bytes = s.as_bytes();
    let mut result: [u8; R] = [0; R];
    let mut result_len: u32 = 0;

    // XPath uses 1-based indexing
    let start_idx: u32 = if start_pos < 1 {
        0
    } else {
        (start_pos - 1) as u32
    };

    for i in 0..N {
        let idx = i as u32;
        if (idx >= start_idx) & (result_len < length) & (result_len < R) {
            let byte = s_bytes[i];
            if byte != 0 {
                result[result_len] = byte;
                result_len += 1;
            }
        }
    }

    (result, result_len)
}

/// fn:substring-before - Returns the part of string before the first occurrence of substring
/// XPath: fn:substring-before($arg1 as xs:string?, $arg2 as xs:string?) as xs:string
pub fn substring_before<let N: u32, let M: u32>(s: str<N>, substr: str<M>) -> ([u8; N], u32) {
    let s_bytes = s.as_bytes();
    let mut result: [u8; N] = [0; N];
    let idx = index_of(s, substr);

    let result_len: u32 = if idx < 0 { 0 } else { idx as u32 };

    for i in 0..N {
        if (i as u32) < result_len {
            result[i] = s_bytes[i];
        }
    }

    (result, result_len)
}

/// fn:substring-after - Returns the part of string after the first occurrence of substring
/// XPath: fn:substring-after($arg1 as xs:string?, $arg2 as xs:string?) as xs:string
pub fn substring_after<let N: u32, let M: u32>(s: str<N>, substr: str<M>) -> ([u8; N], u32) {
    let s_bytes = s.as_bytes();
    let mut result: [u8; N] = [0; N];
    let idx = index_of(s, substr);

    if idx < 0 {
        // Substring not found, return empty
        (result, 0)
    } else {
        let start_idx = (idx as u32) + (M as u32);
        let mut result_len: u32 = 0;

        for i in 0..N {
            let src_idx = start_idx + (i as u32);
            if src_idx < N {
                let byte = s_bytes[src_idx];
                if byte != 0 {
                    result[i] = byte;
                    result_len = (i + 1) as u32;
                }
            }
        }

        (result, result_len)
    }
}

/// fn:codepoints-to-string - Converts a sequence of codepoints to a string (byte array)
/// XPath: fn:codepoints-to-string($arg as xs:integer*) as xs:string
pub fn codepoints_to_string<let N: u32>(codepoints: [u32; N], len: u32) -> ([u8; N], u32) {
    let mut result: [u8; N] = [0; N];

    for i in 0..N {
        if (i as u32) < len {
            // Truncate to u8 (ASCII range)
            result[i] = (codepoints[i] & 0xFF) as u8;
        }
    }

    (result, len)
}

// ============================================================================
// URI Encoding Functions
// ============================================================================

/// Check if a character is unreserved (doesn't need encoding)
/// Unreserved = ALPHA / DIGIT / "-" / "." / "_" / "~"
fn is_uri_unreserved(byte: u8) -> bool {
    let is_alpha = ((byte >= 65) & (byte <= 90)) | ((byte >= 97) & (byte <= 122));
    let is_digit = (byte >= 48) & (byte <= 57);
    let is_special = (byte == 45) | (byte == 46) | (byte == 95) | (byte == 126);
    is_alpha | is_digit | is_special
}

/// Convert a nibble (0-15) to hex character
fn nibble_to_hex(n: u8) -> u8 {
    if n < 10 {
        48 + n // '0' + n
    } else {
        65 + n - 10 // 'A' + n - 10
    }
}

/// fn:encode-for-uri - Encodes special characters for use in URI
/// XPath: fn:encode-for-uri($uri-part as xs:string?) as xs:string
/// Returns (byte_array, length) - result may be up to 3x input length
pub fn encode_for_uri<let N: u32, let R: u32>(s: str<N>) -> ([u8; R], u32) {
    let s_bytes = s.as_bytes();
    let mut result: [u8; R] = [0; R];
    let mut result_len: u32 = 0;

    for i in 0..N {
        let byte = s_bytes[i];
        if byte != 0 {
            if is_uri_unreserved(byte) {
                if result_len < R {
                    result[result_len] = byte;
                    result_len += 1;
                }
            } else {
                // Percent-encode: %XX
                if result_len + 3 <= R {
                    result[result_len] = 37; // '%'
                    result[result_len + 1] = nibble_to_hex((byte >> 4) & 0x0F);
                    result[result_len + 2] = nibble_to_hex(byte & 0x0F);
                    result_len += 3;
                }
            }
        }
    }

    (result, result_len)
}

/// fn:iri-to-uri - Converts an IRI to a URI by encoding non-ASCII characters
/// XPath: fn:iri-to-uri($iri as xs:string?) as xs:string
/// This is a simplified version that percent-encodes non-ASCII bytes
pub fn iri_to_uri<let N: u32, let R: u32>(s: str<N>) -> ([u8; R], u32) {
    let s_bytes = s.as_bytes();
    let mut result: [u8; R] = [0; R];
    let mut result_len: u32 = 0;

    for i in 0..N {
        let byte = s_bytes[i];
        if byte != 0 {
            // Keep ASCII printable characters (32-126) as-is, encode others
            if (byte >= 32) & (byte <= 126) {
                if result_len < R {
                    result[result_len] = byte;
                    result_len += 1;
                }
            } else {
                // Percent-encode
                if result_len + 3 <= R {
                    result[result_len] = 37; // '%'
                    result[result_len + 1] = nibble_to_hex((byte >> 4) & 0x0F);
                    result[result_len + 2] = nibble_to_hex(byte & 0x0F);
                    result_len += 3;
                }
            }
        }
    }

    (result, result_len)
}

/// fn:escape-html-uri - Escapes a URI for HTML
/// XPath: fn:escape-html-uri($uri as xs:string?) as xs:string
/// Escapes non-printable ASCII and non-ASCII characters
pub fn escape_html_uri<let N: u32, let R: u32>(s: str<N>) -> ([u8; R], u32) {
    // Same as iri_to_uri for our purposes
    iri_to_uri(s)
}

// ============================================================================
// Token Functions
// ============================================================================

/// Helper function to check if a byte is whitespace
fn is_whitespace(b: u8) -> bool {
    (b == 0x20) | (b == 0x09) | (b == 0x0A) | (b == 0x0D) // space, tab, newline, carriage return
}

/// fn:contains-token - Check if string contains a specific token
/// XPath: fn:contains-token($input as xs:string*, $token as xs:string) as xs:boolean
/// A token is a word separated by whitespace. Uses default collation (codepoint comparison).
/// Returns true if any whitespace-separated token equals the search token.
pub fn contains_token<let N: u32, let M: u32>(s: str<N>, token: str<M>) -> bool {
    let s_bytes = s.as_bytes();
    let token_bytes = token.as_bytes();

    // Get actual token length (excluding null terminators)
    let mut token_len: u32 = 0;
    for i in 0..M {
        if (token_bytes[i] != 0) & (token_len == (i as u32)) {
            token_len += 1;
        }
    }

    // Empty token never matches - use conditional result instead of early return
    let mut found = false;
    let mut token_start: u32 = 0;
    let mut in_token = false;

    // Only process if token is non-empty
    if token_len > 0 {
        for i in 0..N {
            let byte = s_bytes[i];
            let is_ws = is_whitespace(byte) | (byte == 0);

            if !is_ws & !in_token {
                // Start of a new token
                token_start = i as u32;
                in_token = true;
            } else if is_ws & in_token {
                // End of current token - check if it matches
                let current_token_len = (i as u32) - token_start;
                if current_token_len == token_len {
                    let mut matches = true;
                    for j in 0..M {
                        if (j as u32) < token_len {
                            let s_idx = token_start + (j as u32);
                            if s_idx < N {
                                if s_bytes[s_idx] != token_bytes[j] {
                                    matches = false;
                                }
                            }
                        }
                    }
                    if matches {
                        found = true;
                    }
                }
                in_token = false;
            }
        }

        // Check final token if string doesn't end with whitespace
        if in_token & !found {
            let current_token_len = N - token_start;
            if current_token_len == token_len {
                let mut matches = true;
                for j in 0..M {
                    if (j as u32) < token_len {
                        let s_idx = token_start + (j as u32);
                        if s_idx < N {
                            if s_bytes[s_idx] != token_bytes[j] {
                                matches = false;
                            }
                        }
                    }
                }
                if matches {
                    found = true;
                }
            }
        }
    }

    found
}

/// fn:tokenize (whitespace only) - Splits a string by whitespace
/// XPath: fn:tokenize($input as xs:string?, $pattern as xs:string) as xs:string*
/// This is the simplified version that splits by whitespace only (no regex).
/// Returns array of token byte arrays and their lengths.
/// MAX_TOKENS is the maximum number of tokens, TOKEN_LEN is max length per token.
pub fn tokenize_whitespace<let N: u32, let MAX_TOKENS: u32, let TOKEN_LEN: u32>(
    s: str<N>,
) -> ([[u8; TOKEN_LEN]; MAX_TOKENS], [u32; MAX_TOKENS], u32) {
    let s_bytes = s.as_bytes();
    let mut tokens: [[u8; TOKEN_LEN]; MAX_TOKENS] = [[0; TOKEN_LEN]; MAX_TOKENS];
    let mut token_lens: [u32; MAX_TOKENS] = [0; MAX_TOKENS];
    let mut num_tokens: u32 = 0;

    let mut token_start: u32 = 0;
    let mut in_token = false;

    for i in 0..N {
        let byte = s_bytes[i];
        let is_ws = is_whitespace(byte) | (byte == 0);

        if !is_ws & !in_token {
            // Start of a new token
            token_start = i as u32;
            in_token = true;
        } else if is_ws & in_token {
            // End of current token - save it if we have room
            if num_tokens < MAX_TOKENS {
                let token_len = (i as u32) - token_start;
                let actual_len = if token_len > TOKEN_LEN {
                    TOKEN_LEN
                } else {
                    token_len
                };
                for j in 0..TOKEN_LEN {
                    if (j as u32) < actual_len {
                        let s_idx = token_start + (j as u32);
                        if s_idx < N {
                            tokens[num_tokens][j] = s_bytes[s_idx];
                        }
                    }
                }
                token_lens[num_tokens] = actual_len;
                num_tokens += 1;
            }
            in_token = false;
        }
    }

    // Handle final token if string doesn't end with whitespace
    if in_token & (num_tokens < MAX_TOKENS) {
        let token_len = N - token_start;
        let actual_len = if token_len > TOKEN_LEN {
            TOKEN_LEN
        } else {
            token_len
        };
        for j in 0..TOKEN_LEN {
            if (j as u32) < actual_len {
                let s_idx = token_start + (j as u32);
                if s_idx < N {
                    tokens[num_tokens][j] = s_bytes[s_idx];
                }
            }
        }
        token_lens[num_tokens] = actual_len;
        num_tokens += 1;
    }

    (tokens, token_lens, num_tokens)
}

// ============================================================================
// String Conversion Functions
// ============================================================================

/// fn:string for integers - Converts an integer to its string representation
/// XPath: fn:string($arg as item()?) as xs:string
/// Returns (bytes, length)
pub fn fn_string_from_integer<let N: u32>(value: i64) -> ([u8; N], u32) {
    let mut result: [u8; N] = [0; N];
    let mut result_len: u32 = 0;

    if value == 0 {
        if N > 0 {
            result[0] = 0x30; // '0'
            result_len = 1;
        }
    } else {
        let mut temp: [u8; 20] = [0; 20]; // Max 20 digits for i64
        let mut temp_len: u32 = 0;
        let negative = value < 0;
        let mut abs_val = if negative { -value } else { value };

        // Extract digits in reverse order
        for _ in 0..20 {
            if abs_val > 0 {
                let digit = (abs_val % 10) as u8;
                temp[temp_len] = 0x30 + digit; // '0' + digit
                temp_len += 1;
                abs_val = abs_val / 10;
            }
        }

        // Add sign if negative
        if negative & (result_len < N) {
            result[result_len] = 0x2D; // '-'
            result_len += 1;
        }

        // Reverse digits into result
        for i in 0..20 {
            if ((i as u32) < temp_len) & (result_len < N) {
                let reverse_idx = temp_len - 1 - (i as u32);
                result[result_len] = temp[reverse_idx];
                result_len += 1;
            }
        }
    }

    (result, result_len)
}

/// fn:string for booleans - Converts a boolean to "true" or "false"
/// XPath: fn:string($arg as item()?) as xs:string
pub fn fn_string_from_boolean<let N: u32>(value: bool) -> ([u8; N], u32) {
    let mut result: [u8; N] = [0; N];
    let mut result_len: u32 = 0;

    if value {
        // "true"
        if N >= 4 {
            result[0] = 0x74; // 't'
            result[1] = 0x72; // 'r'
            result[2] = 0x75; // 'u'
            result[3] = 0x65; // 'e'
            result_len = 4;
        }
    } else {
        // "false"
        if N >= 5 {
            result[0] = 0x66; // 'f'
            result[1] = 0x61; // 'a'
            result[2] = 0x6C; // 'l'
            result[3] = 0x73; // 's'
            result[4] = 0x65; // 'e'
            result_len = 5;
        }
    }

    (result, result_len)
}
