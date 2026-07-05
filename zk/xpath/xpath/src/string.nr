//! XPath 2.0 String Functions
//!
//! This module implements XPath 2.0 string functions that work correctly in Noir.
//! These functions are required by SPARQL 1.1 Query Language.
//!
//! ## Working Functions
//!
//! The following functions work correctly and are exported:
//! - `string_length` - Returns the length in Unicode codepoints as a number
//! - `starts_with` - Returns boolean
//! - `ends_with` - Returns boolean
//! - `contains` - Returns boolean
//!
//! ## Conventions
//!
//! A `str<N>` in this crate is a fixed-CAPACITY buffer: the logical string is
//! the bytes up to the first NUL terminator (or all N bytes if none), matching
//! `hash.nr`. Comparison and equality functions operate on that logical
//! content, so the same string stored in `str<10>` and `str<3>` compares
//! equal. NUL is a terminator, never U+0000 content.
//!
//! ## Limitations
//!
//! Functions that need to create new strings (substring, upper-case,
//! lower-case, concat, etc.) cannot return `str<_>` values because Noir
//! doesn't provide a way to convert byte arrays back to strings at runtime.
//! They instead return a `([u8; _], u32)` (bytes, logical length) pair.

/// Logical length of a NUL-padded byte buffer: bytes up to the first NUL
/// terminator (the capacity convention used across this crate; mirrors the
/// private helper in hash.nr). [FABLE-5]
fn logical_len_bytes<let N: u32>(bytes: [u8; N]) -> u32 {
    let mut len: u32 = 0;
    let mut ended = false;
    for i in 0..N {
        let is_zero = bytes[i] == 0;
        if (!ended) & (!is_zero) {
            len += 1;
        }
        ended = ended | is_zero;
    }
    len
}

/// fn:string-length - Returns the length of a string in Unicode CODEPOINTS
/// XPath: fn:string-length($arg as xs:string?) as xs:integer
/// SPARQL: STRLEN(?str)
///
/// Counts UTF-8 lead bytes (bytes with (b & 0xC0) != 0x80) in the logical
/// content (up to the first NUL / capacity). For well-formed UTF-8 this is
/// exactly the codepoint count that fn:string-length requires; for pure-ASCII
/// content it equals the byte count. Counts codepoints, not grapheme
/// clusters, per the F&O definition. Ill-formed UTF-8 is not detected: every
/// non-continuation byte counts as one codepoint. [FABLE-5]
pub fn string_length<let N: u32>(s: str<N>) -> u64 {
    let s_bytes = s.as_bytes();
    let mut len: u64 = 0;
    let mut ended = false;

    for i in 0..N {
        let byte = s_bytes[i];
        let is_zero = byte == 0;
        let is_continuation = (byte & 0xC0) == 0x80;
        let increment = (!ended) & (!is_zero) & (!is_continuation);
        len = len + (increment as u64);
        ended = ended | is_zero;
    }

    len
}

/// fn:starts-with - Returns true if a string starts with a given prefix
/// XPath: fn:starts-with($arg1 as xs:string?, $arg2 as xs:string?) as xs:boolean
/// SPARQL: STRSTARTS(?str, ?prefix)
///
/// Matches on the LOGICAL content of the prefix (bytes before the first NUL),
/// not the buffer capacity, so a prefix stored in a wider NUL-padded buffer
/// compares only its meaningful bytes against the haystack. Per F&O sec.7.5.1:
/// fn:starts-with($arg1, "") is true for any $arg1 (empty prefix always
/// matches). [SONNET-4.6] sq-0ylr9
pub fn starts_with<let N: u32, let M: u32>(s: str<N>, prefix: str<M>) -> bool {
    let s_bytes = s.as_bytes();
    let prefix_bytes = prefix.as_bytes();
    let s_len = logical_len_bytes(s_bytes);
    let prefix_len = logical_len_bytes(prefix_bytes);

    // F&O sec.7.5.1: fn:starts-with($arg1, "") = true for any $arg1.
    // A logically-empty prefix always matches; a prefix whose logical length
    // exceeds the logical haystack never matches.
    let fits = prefix_len <= s_len;
    let mut matches = fits;

    // Compare only the logical bytes of the prefix; NUL padding in the
    // prefix buffer must not require NULs in the haystack.
    for i in 0..M {
        let in_range = ((i as u32) < prefix_len) & fits;
        // Clamp for the (always-executed, post-flatten) static read; when
        // in_range is true, i < prefix_len <= s_len <= N, so safe_s_idx == i.
        let safe_s_idx = if i < N { i } else { 0 };
        if in_range & (s_bytes[safe_s_idx] != prefix_bytes[i]) {
            matches = false;
        }
    }

    matches
}

/// fn:ends-with - Returns true if a string ends with a given suffix
/// XPath: fn:ends-with($arg1 as xs:string?, $arg2 as xs:string?) as xs:boolean
/// SPARQL: STRENDS(?str, ?suffix)
///
/// Anchored at the LOGICAL end of the string (bytes before the first NUL),
/// not the buffer capacity, so STRENDS("hello" in str<10>, "llo") is true.
/// An empty suffix always matches. [FABLE-5] sq-3x7dl.6
pub fn ends_with<let N: u32, let M: u32>(s: str<N>, suffix: str<M>) -> bool {
    let s_bytes = s.as_bytes();
    let suffix_bytes = suffix.as_bytes();
    let s_len = logical_len_bytes(s_bytes);
    let suffix_len = logical_len_bytes(suffix_bytes);

    // A suffix longer than the logical content never matches; the same
    // flag doubles as the underflow guard for the offset below.
    let fits = suffix_len <= s_len;
    let mut matches = fits;
    let offset: u32 = if fits { s_len - suffix_len } else { 0 };

    // Comptime guard: for N == 0 the whole loop folds away, so the size-0
    // byte array is never indexed.
    if N > 0 {
        for i in 0..M {
            if i < suffix_len {
                let s_idx = offset + i;
                // Clamp for the (always executed, post-flatten) dynamic
                // read; when the clamp fires, fits is false and matches is
                // already pinned false, so the spurious compare is inert.
                let safe_idx = if s_idx < N { s_idx } else { 0 };
                if s_bytes[safe_idx] != suffix_bytes[i] {
                    matches = false;
                }
            }
        }
    }

    matches
}

/// Helper function to find the byte-index of the first occurrence of
/// `substring` within `s`, operating on LOGICAL lengths (bytes before the
/// first NUL) for both arguments.
///
/// Returns the 0-based start index of the first match, or -1 if not found.
/// Returns 0 for an empty needle (F&O sec.7.5.3: contains(X, "") = true).
///
/// **Performance Note**: O(N*M) time; due to Noir's lack of early-exit the
/// scan continues past the first match. For large strings this increases
/// proof-generation cost. [SONNET-4.6] sq-0ylr9
fn index_of<let N: u32, let M: u32>(s: str<N>, substring: str<M>) -> i64 {
    let s_bytes = s.as_bytes();
    let sub_bytes = substring.as_bytes();
    let s_len = logical_len_bytes(s_bytes);
    let sub_len = logical_len_bytes(sub_bytes);

    let mut result: i64 = -1;

    // F&O sec.7.5.3: fn:contains($arg1, "") = true; by convention index_of
    // returns 0 for an empty needle (found at position 0 of any string).
    let empty_needle = sub_len == 0;

    if empty_needle {
        result = 0;
    } else if sub_len <= s_len {
        let mut found_index: i64 = -1;

        // Iterate all possible start positions within the logical haystack.
        // Comptime loop bound is N (buffer capacity); the runtime guard
        // `(i as u32) + sub_len <= s_len` restricts to valid positions.
        // Can't use break, so the scan continues past the first match.
        for i in 0..N {
            let start_valid = ((i as u32) + sub_len) <= s_len;
            if start_valid {
                let mut matches = true;
                for j in 0..M {
                    // Compare only the logical bytes of the needle; NUL
                    // padding in the needle buffer must not require NULs in
                    // the haystack.
                    if (j as u32) < sub_len {
                        let s_idx = i + j;
                        // Clamp for the (always-executed, post-flatten) read;
                        // within start_valid & j < sub_len, s_idx < s_len <=
                        // N, so the clamp fires only on inert dead paths.
                        let safe_s_idx = if s_idx < N { s_idx } else { 0 };
                        if s_bytes[safe_s_idx] != sub_bytes[j] {
                            matches = false;
                        }
                    }
                }
                if matches & (found_index == -1) {
                    found_index = i as i64;
                }
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
///
/// Operates on the LOGICAL content (bytes before the first NUL); trailing
/// capacity NULs are terminators, not U+0000 content, so equal content in
/// str<10> and str<3> compares equal. Byte-wise comparison of well-formed
/// UTF-8 coincides with Unicode codepoint order, matching the codepoint
/// collation. [FABLE-5] sq-3x7dl.6
pub fn compare<let N: u32, let M: u32>(s1: str<N>, s2: str<M>) -> i64 {
    let s1_bytes = s1.as_bytes();
    let s2_bytes = s2.as_bytes();
    let l1 = logical_len_bytes(s1_bytes);
    let l2 = logical_len_bytes(s2_bytes);

    // Comptime bound; the logical lengths never exceed the capacities.
    let min_cap = if N < M { N } else { M };

    let mut result: i64 = 0;
    let mut determined = false;

    // Compare byte by byte within the shared logical prefix
    for i in 0..min_cap {
        let in_content = (i < l1) & (i < l2);
        if (!determined) & in_content {
            if s1_bytes[i] < s2_bytes[i] {
                result = -1;
                determined = true;
            } else if s1_bytes[i] > s2_bytes[i] {
                result = 1;
                determined = true;
            }
        }
    }

    // If the shared prefix is equal, the shorter LOGICAL string comes first
    if !determined {
        if l1 < l2 {
            result = -1;
        } else if l1 > l2 {
            result = 1;
        }
        // else result stays 0 (strings are equal)
    }

    result
}

/// fn:codepoint-equal - Returns true if two strings are equal codepoint by codepoint
/// XPath: fn:codepoint-equal($comparand1 as xs:string?, $comparand2 as xs:string?) as xs:boolean?
///
/// Gated on the LOGICAL lengths (bytes before the first NUL), not the buffer
/// capacities, so equal content in str<10> and str<3> is codepoint-equal.
/// Byte equality of the logical content is exactly codepoint equality for
/// (well-formed) UTF-8. [FABLE-5] sq-3x7dl.6
pub fn codepoint_equal<let N: u32, let M: u32>(s1: str<N>, s2: str<M>) -> bool {
    let s1_bytes = s1.as_bytes();
    let s2_bytes = s2.as_bytes();
    let l1 = logical_len_bytes(s1_bytes);
    let l2 = logical_len_bytes(s2_bytes);

    let mut equal = l1 == l2;

    // Comptime bound; when l1 == l2 every content index is below it.
    let min_cap = if N < M { N } else { M };
    for i in 0..min_cap {
        if (i < l1) & (i < l2) {
            if s1_bytes[i] != s2_bytes[i] {
                equal = false;
            }
        }
    }
    equal
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
                // Check if there's a corresponding character in trans_string.
                // [OPUS-4.8] sq-y73 fix: Noir's `&` does NOT short-circuit, so
                // `trans_bytes[trans_idx]` was indexed even when trans_idx >= K,
                // panicking with "Index out of bounds" whenever transString is
                // shorter than mapString (the delete case). Clamp the index used
                // for the read and gate the write on the real bound.
                let trans_idx = found_idx as u32;
                let in_bounds = trans_idx < K;
                // Clamp the index to a statically-valid slot for the (always
                // executed, post-flatten) read; `K > 0` is comptime so the
                // whole read folds away for an empty transString.
                if K > 0 {
                    let safe_idx = if in_bounds { trans_idx } else { 0 };
                    let trans_byte = trans_bytes[safe_idx];
                    if in_bounds & (trans_byte != 0) & (result_len < N) {
                        result[result_len] = trans_byte;
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
///
/// F&O 5.4.3: the result is the characters at 1-based positions p with
/// round(start) <= p < round(start) + round(length). The window is NOT
/// shifted when start < 1 -- positions below 1 simply fall outside the
/// string, so a start below 1 consumes part of the length budget:
///   substring("12345",  0, 3) == "12"  (window [0,3)  -> positions 1,2)
///   substring("12345", -3, 5) == "1"   (window [-3,2) -> position 1)
///
/// This signature takes the already-rounded integer window: callers wiring
/// the xs:double form must apply the F&O round() first and map a NaN start
/// or length to an empty result (round() of NaN has no integer image); a
/// rounded length <= 0 maps to the empty result (carried by the u32 type).
///
/// POSITION UNIT CAVEAT: start/length index BYTE positions in the logical
/// content -- exact vs F&O only for single-byte (ASCII) content. Since
/// string_length() counts CODEPOINTS, composing them (e.g.
/// substring(s, i, string_length(s))) is wrong for multi-byte UTF-8;
/// codepoint-positional substring is not implemented (bead sq-hjvte).
/// [FABLE-5] sq-3x7dl.6
pub fn substring<let N: u32, let R: u32>(s: str<N>, start_pos: i64, length: u32) -> ([u8; R], u32) {
    let s_bytes = s.as_bytes();
    let mut result: [u8; R] = [0; R];
    let mut result_len: u32 = 0;

    let s_cap: i64 = N as i64;
    // Clamp the raw start into [-(2^32), N + 1] so the window arithmetic
    // below cannot overflow i64. Both clamps preserve the result: a start
    // beyond N + 1 is past any content (empty result), and a start at or
    // below -(2^32) keeps start + length (length < 2^32) at or below 0 < 1
    // (empty result) on both sides of the clamp.
    let lo: i64 = -4294967296;
    let clamped_start: i64 = if start_pos > s_cap + 1 {
        s_cap + 1
    } else if start_pos < lo {
        lo
    } else {
        start_pos
    };

    // Window end (exclusive) and effective 1-based start position.
    let end_pos: i64 = clamped_start + (length as i64);
    let eff_start: i64 = if clamped_start < 1 { 1 } else { clamped_start };

    // Number of positions in [eff_start, end_pos), clamped to the capacity.
    let window: i64 = end_pos - eff_start;
    let count: u32 = if window <= 0 {
        0
    } else if window > s_cap {
        N
    } else {
        window as u32
    };

    // eff_start is in [1, N + 1], so this cast is exact.
    let start_idx: u32 = (eff_start - 1) as u32;

    for i in 0..N {
        if (i >= start_idx) & (result_len < count) & (result_len < R) {
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
        // Use the LOGICAL length of the needle (bytes before the first NUL),
        // not the buffer capacity M.  With NUL-padded needles M > sub_len, so
        // using M caused over-skipping (sq-0ylr9). [SONNET-4.6]
        let sub_len = logical_len_bytes(substr.as_bytes());
        let start_idx = (idx as u32) + sub_len;
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

// ============================================================================
// [OPUS-4.8] sq-y73: coverage for previously-untested string transforms.
// VENDOR.md flagged substring/substring_before/after, upper/lower_case,
// concat_bytes, string_join_two, encode_for_uri, compare/string_equal/
// less/greater, translate, normalize_space as implemented-but-untested.
// These inline tests close that gap. Flag for re-review when Fable returns.
// ============================================================================

#[test]
fn test_substring_with_length() {
    // fn:substring("hello", 2, 3) -> "ell"
    let s: str<5> = "hello";
    let (out, len) = substring::<5, 5>(s, 2, 3);
    assert(len == 3);
    let expected = "ell".as_bytes();
    for i in 0..3 {
        assert(out[i] == expected[i]);
    }
}

// [FABLE-5] sq-3x7dl.6: the old test_substring_start_clamped enshrined the
// wrong answer (asserted substring("hello", -2, 4) == "hell"). Per F&O 5.4.3
// the window [-2, 2) intersected with the string positions [1, 5] is just
// position 1, so the result is "h". Oracle (Python, F&O window definition):
// substring('hello',-2,4) = 'h'.
#[test]
fn test_substring_start_below_one_consumes_length() {
    // fn:substring("hello", -2, 4) -> "h" (window [-2,2) -> position 1 only)
    let s: str<5> = "hello";
    let (out, len) = substring::<5, 5>(s, -2, 4);
    assert(len == 1);
    let expected = "h".as_bytes();
    assert(out[0] == expected[0]);
}

#[test]
fn test_substring_start_zero() {
    // fn:substring("12345", 0, 3) -> "12" (window [0,3) -> positions 1,2)
    let s: str<5> = "12345";
    let (out, len) = substring::<5, 5>(s, 0, 3);
    assert(len == 2);
    let expected = "12".as_bytes();
    for i in 0..2 {
        assert(out[i] == expected[i]);
    }
}

#[test]
fn test_substring_start_negative() {
    // fn:substring("12345", -3, 5) -> "1" (window [-3,2) -> position 1 only)
    let s: str<5> = "12345";
    let (out, len) = substring::<5, 5>(s, -3, 5);
    assert(len == 1);
    let expected = "1".as_bytes();
    assert(out[0] == expected[0]);
}

#[test]
fn test_substring_window_entirely_before_string() {
    // fn:substring("hello", -100, 3) -> "" (window ends before position 1)
    let s: str<5> = "hello";
    let (_out, len) = substring::<5, 5>(s, -100, 3);
    assert(len == 0);
}

#[test]
fn test_substring_start_past_end() {
    // fn:substring("hello", 7, 3) -> ""
    let s: str<5> = "hello";
    let (_out, len) = substring::<5, 5>(s, 7, 3);
    assert(len == 0);
}

#[test]
fn test_substring_zero_length() {
    // fn:substring("hello", 3, 0) -> ""
    let s: str<5> = "hello";
    let (_out, len) = substring::<5, 5>(s, 3, 0);
    assert(len == 0);
}

#[test]
fn test_substring_length_exceeds_string() {
    // fn:substring("hello", 1, 10) -> "hello" (window clamped to content)
    let s: str<5> = "hello";
    let (out, len) = substring::<5, 5>(s, 1, 10);
    assert(len == 5);
    let expected = "hello".as_bytes();
    for i in 0..5 {
        assert(out[i] == expected[i]);
    }
}

#[test]
fn test_substring_extreme_start_no_overflow() {
    // Far-out-of-range starts must yield empty results, not overflow.
    let s: str<5> = "hello";
    let (_out1, len1) = substring::<5, 5>(s, 0x7FFFFFFFFFFFFFFF, 3);
    assert(len1 == 0);
    let (_out2, len2) = substring::<5, 5>(s, -0x7FFFFFFFFFFFFFFF, 0);
    assert(len2 == 0);
}

// ============================================================================
// [FABLE-5] sq-3x7dl.6: logical-length (NUL-padding) semantics + codepoint
// STRLEN. Expected values from an independent Python oracle over the F&O
// definitions (see the bead / PR).
// ============================================================================

#[test]
fn test_ends_with_nul_padded() {
    // STRENDS("hello" stored in str<10>, "llo") must be true: the anchor is
    // the logical end (before the NUL padding), not the buffer capacity.
    let s: str<10> = "hello\0\0\0\0\0";
    let suffix: str<3> = "llo";
    assert(ends_with::<10, 3>(s, suffix));
}

#[test]
fn test_ends_with_nul_padded_suffix() {
    // Suffix itself NUL-padded: logical "llo" in str<6>.
    let s: str<5> = "hello";
    let suffix: str<6> = "llo\0\0\0";
    assert(ends_with::<5, 6>(s, suffix));
}

#[test]
fn test_ends_with_nul_padded_negative() {
    // "hell" is a prefix, not a suffix, of the logical content.
    let s: str<10> = "hello\0\0\0\0\0";
    let suffix: str<4> = "hell";
    assert(!ends_with::<10, 4>(s, suffix));
}

#[test]
fn test_ends_with_empty_suffix() {
    // fn:ends-with(x, "") is always true.
    let s: str<5> = "hello";
    let suffix: str<0> = "";
    assert(ends_with::<5, 0>(s, suffix));
}

#[test]
fn test_ends_with_suffix_longer_than_content() {
    // Logical suffix longer than the logical string never matches, even
    // when the string BUFFER is wider than the suffix buffer.
    let s: str<10> = "lo\0\0\0\0\0\0\0\0";
    let suffix: str<5> = "hello";
    assert(!ends_with::<10, 5>(s, suffix));
}

#[test]
fn test_compare_ignores_nul_padding() {
    // Equal logical content in different capacities compares equal.
    let a: str<10> = "ab\0\0\0\0\0\0\0\0";
    let b: str<2> = "ab";
    assert(compare::<10, 2>(a, b) == 0);
    assert(string_equal::<10, 2>(a, b));
    // Logical prefix still sorts first regardless of capacities.
    let c: str<3> = "abc";
    assert(compare::<10, 3>(a, c) == -1);
    assert(compare::<3, 10>(c, a) == 1);
}

#[test]
fn test_codepoint_equal_ignores_nul_padding() {
    // Equal logical content in different capacities is codepoint-equal.
    let a: str<10> = "abc\0\0\0\0\0\0\0";
    let b: str<3> = "abc";
    assert(codepoint_equal::<10, 3>(a, b));
    // Different logical content is not, even at equal capacity.
    let c: str<10> = "abd\0\0\0\0\0\0\0";
    assert(!codepoint_equal::<10, 10>(a, c));
    // Same bytes, different logical length is not equal.
    let d: str<10> = "ab\0\0\0\0\0\0\0\0";
    assert(!codepoint_equal::<10, 10>(a, d));
}

#[test]
fn test_string_length_counts_codepoints() {
    // STRLEN("e-acute") == 1: U+00E9 is two UTF-8 bytes (0xC3 0xA9) but one
    // codepoint. Oracle: len('\u{e9}') == 1, UTF-8 bytes [195, 169].
    let s: str<2> = "é";
    assert(s.as_bytes()[0] == 195);
    assert(s.as_bytes()[1] == 169);
    assert(string_length::<2>(s) == 1);
}

#[test]
fn test_string_length_mixed_ascii_multibyte() {
    // STRLEN("a" + e-acute) == 2 (3 UTF-8 bytes). Oracle: bytes [97,195,169].
    let s: str<3> = "aé";
    assert(string_length::<3>(s) == 2);
}

#[test]
fn test_string_length_nul_padded() {
    // Logical length stops at the first NUL terminator.
    let s: str<10> = "hello\0\0\0\0\0";
    assert(string_length::<10>(s) == 5);
}

#[test]
fn test_substring_before() {
    // fn:substring-before("1999/04/01", "/") -> "1999"
    let s: str<10> = "1999/04/01";
    let sep: str<1> = "/";
    let (out, len) = substring_before::<10, 1>(s, sep);
    assert(len == 4);
    let expected = "1999".as_bytes();
    for i in 0..4 {
        assert(out[i] == expected[i]);
    }
}

#[test]
fn test_substring_after() {
    // fn:substring-after("1999/04/01", "/") -> "04/01"
    let s: str<10> = "1999/04/01";
    let sep: str<1> = "/";
    let (out, len) = substring_after::<10, 1>(s, sep);
    assert(len == 5);
    let expected = "04/01".as_bytes();
    for i in 0..5 {
        assert(out[i] == expected[i]);
    }
}

// ============================================================================
// [SONNET-4.6] sq-0ylr9: substring_after / substring_before with NUL-padded
// needles: regression tests added after fixing the M-vs-sub_len over-skip bug.
// Expected values from Python:
//   >>> import re
//   >>> def substring_after(s, needle): i=s.find(needle); return s[i+len(needle):] if i>=0 else ""
//   >>> def substring_before(s, needle): i=s.find(needle); return s[:i] if i>=0 else ""
// ============================================================================

#[test]
fn test_substring_after_padded_needle() {
    // Python oracle: substring_after("abcdefgh", "cd") -> "efgh"
    // "cd" stored in str<4> as "cd\0\0"; M=4 but logical needle length=2.
    // Pre-fix this returned "gh" (over-skipped by 2 bytes). (sq-0ylr9)
    let s: str<8> = "abcdefgh";
    let needle: str<4> = "cd\0\0";
    let (out, len) = substring_after::<8, 4>(s, needle);
    assert(len == 4);
    let expected = "efgh".as_bytes();
    for i in 0..4 {
        assert(out[i] == expected[i]);
    }
}

#[test]
fn test_substring_after_empty_needle_in_wider_buf() {
    // F&O 7.5.5: fn:substring-after($arg1, "") = $arg1 for any $arg1.
    // Python oracle: substring_after("hello", "") -> "hello"
    // Logically-empty needle in a wider buffer (str<3> = "\0\0\0") must not
    // drop M bytes; the whole string must be returned.
    let s: str<5> = "hello";
    let needle: str<3> = "\0\0\0";
    let (out, len) = substring_after::<5, 3>(s, needle);
    assert(len == 5);
    let expected = "hello".as_bytes();
    for i in 0..5 {
        assert(out[i] == expected[i]);
    }
}

#[test]
fn test_substring_before_padded_needle() {
    // Python oracle: substring_before("abcdefgh", "cd") -> "ab"
    // "cd" stored in str<4> as "cd\0\0"; logical needle length=2.
    // Locks the behavior: substring_before already used index_of which now
    // computes the match position logically, so the result must stay "ab".
    let s: str<8> = "abcdefgh";
    let needle: str<4> = "cd\0\0";
    let (out, len) = substring_before::<8, 4>(s, needle);
    assert(len == 2);
    let expected = "ab".as_bytes();
    for i in 0..2 {
        assert(out[i] == expected[i]);
    }
}

#[test]
fn test_upper_case() {
    let s: str<5> = "aBc12";
    let (out, len) = upper_case::<5>(s);
    assert(len == 5);
    let expected = "ABC12".as_bytes();
    for i in 0..5 {
        assert(out[i] == expected[i]);
    }
}

#[test]
fn test_lower_case() {
    let s: str<5> = "AbC12";
    let (out, len) = lower_case::<5>(s);
    assert(len == 5);
    let expected = "abc12".as_bytes();
    for i in 0..5 {
        assert(out[i] == expected[i]);
    }
}

#[test]
fn test_concat_bytes() {
    // concat("foo", "bar") -> "foobar"
    let a = "foo".as_bytes();
    let b = "bar".as_bytes();
    let (out, len) = concat_bytes::<3, 3, 6>(a, 3, b, 3);
    assert(len == 6);
    let expected = "foobar".as_bytes();
    for i in 0..6 {
        assert(out[i] == expected[i]);
    }
}

#[test]
fn test_string_join_two() {
    // string-join(("a","b"), "-") -> "a-b"
    let a = "a".as_bytes();
    let b = "b".as_bytes();
    let sep = "-".as_bytes();
    let (out, len) = string_join_two::<1, 1, 1, 3>(a, 1, b, 1, sep, 1);
    assert(len == 3);
    let expected = "a-b".as_bytes();
    for i in 0..3 {
        assert(out[i] == expected[i]);
    }
}

#[test]
fn test_encode_for_uri() {
    // encode-for-uri("a b") -> "a%20b"
    let s: str<3> = "a b";
    let (out, len) = encode_for_uri::<3, 9>(s);
    assert(len == 5);
    let expected = "a%20b".as_bytes();
    for i in 0..5 {
        assert(out[i] == expected[i]);
    }
}

#[test]
fn test_compare_and_relops() {
    let a: str<3> = "abc";
    let b: str<3> = "abd";
    let c: str<3> = "abc";
    assert(compare::<3, 3>(a, b) == -1);
    assert(compare::<3, 3>(b, a) == 1);
    assert(compare::<3, 3>(a, c) == 0);
    assert(string_equal::<3, 3>(a, c));
    assert(string_less_than::<3, 3>(a, b));
    assert(string_greater_than::<3, 3>(b, a));
    assert(string_le::<3, 3>(a, c));
    assert(string_ge::<3, 3>(a, c));
}

#[test]
fn test_compare_prefix_shorter_first() {
    // "ab" < "abc" (shorter string with equal prefix sorts first)
    let a: str<2> = "ab";
    let b: str<3> = "abc";
    assert(compare::<2, 3>(a, b) == -1);
}

#[test]
fn test_translate() {
    // translate("bar","abc","ABC") -> "BAr"
    let s: str<3> = "bar";
    let from: str<3> = "abc";
    let to: str<3> = "ABC";
    let (out, len) = translate::<3, 3, 3>(s, from, to);
    assert(len == 3);
    let expected = "BAr".as_bytes();
    for i in 0..3 {
        assert(out[i] == expected[i]);
    }
}

#[test]
fn test_translate_delete() {
    // translate("abcabc","abc","AB") -> "ABAB" (c has no mapping -> deleted)
    let s: str<6> = "abcabc";
    let from: str<3> = "abc";
    let to: str<2> = "AB";
    let (out, len) = translate::<6, 3, 2>(s, from, to);
    assert(len == 4);
    let expected = "ABAB".as_bytes();
    for i in 0..4 {
        assert(out[i] == expected[i]);
    }
}

#[test]
fn test_normalize_space() {
    // normalize-space("  a  b  ") -> "a b"
    let s: str<8> = "  a  b  ";
    let (out, len) = normalize_space::<8>(s);
    assert(len == 3);
    let expected = "a b".as_bytes();
    for i in 0..3 {
        assert(out[i] == expected[i]);
    }
}

#[test]
fn test_codepoint_equal() {
    let a: str<3> = "abc";
    let b: str<3> = "abc";
    let c: str<3> = "abd";
    assert(codepoint_equal::<3, 3>(a, b));
    assert(!codepoint_equal::<3, 3>(a, c));
}

// ============================================================================
// [SONNET-4.6] sq-0ylr9: logical-length (NUL-padded NEEDLE) handling for
// starts_with, contains, and the private index_of helper.
// Bug class: a needle stored in a wider NUL-padded buffer compared all M
// buffer bytes (including NULs) against the haystack, yielding false negatives.
// Fix: derive needle logical length with logical_len_bytes; compare only the
// meaningful bytes; empty needle = true per F&O sec.7.5.1/sec.7.5.3.
// Expected values from an independent Python oracle over the F&O definitions.
// ============================================================================

// --- starts_with: NUL-padded prefix ---

#[test]
fn test_starts_with_padded_prefix_at_start() {
    // Python oracle: "hello".startswith("he") == True
    // prefix "he" stored in str<6>; NUL padding must not require NULs in s.
    let s: str<5> = "hello";
    let prefix: str<6> = "he\0\0\0\0";
    assert(starts_with::<5, 6>(s, prefix));
}

#[test]
fn test_starts_with_padded_prefix_full_content() {
    // Python oracle: "hello".startswith("hello") == True
    // prefix logical length == haystack logical length; both stored at exact
    // and widened capacities.
    let s: str<5> = "hello";
    let prefix: str<8> = "hello\0\0\0";
    assert(starts_with::<5, 8>(s, prefix));
}

#[test]
fn test_starts_with_empty_needle() {
    // F&O sec.7.5.1: fn:starts-with($arg1, "") = true for any $arg1.
    // Python oracle: "hello".startswith("") == True
    let s: str<5> = "hello";
    let prefix: str<0> = "";
    assert(starts_with::<5, 0>(s, prefix));
}

#[test]
fn test_starts_with_padded_prefix_empty_needle_in_wider_buf() {
    // A logically-empty prefix in a non-zero-capacity buffer also matches.
    // Python oracle: "hello".startswith("") == True (NUL prefix = empty)
    let s: str<5> = "hello";
    let prefix: str<4> = "\0\0\0\0";
    assert(starts_with::<5, 4>(s, prefix));
}

#[test]
fn test_starts_with_padded_prefix_needle_longer_than_haystack() {
    // Python oracle: "hi".startswith("hello") == False
    // Logical needle (5) > logical haystack (2): never matches.
    let s: str<2> = "hi";
    let prefix: str<6> = "hello\0";
    assert(!starts_with::<2, 6>(s, prefix));
}

#[test]
fn test_starts_with_padded_prefix_mismatch() {
    // Python oracle: "hello".startswith("world") == False
    let s: str<5> = "hello";
    let prefix: str<6> = "world\0";
    assert(!starts_with::<5, 6>(s, prefix));
}

// --- contains / index_of: NUL-padded needle ---

#[test]
fn test_contains_padded_needle_at_end() {
    // Python oracle: "llo" in "hello" == True
    // needle "llo" stored in str<6>; NUL padding must not require NULs in s.
    let s: str<5> = "hello";
    let needle: str<6> = "llo\0\0\0";
    assert(contains::<5, 6>(s, needle));
}

#[test]
fn test_contains_padded_needle_in_middle() {
    // Python oracle: "ell" in "hello" == True
    let s: str<5> = "hello";
    let needle: str<5> = "ell\0\0";
    assert(contains::<5, 5>(s, needle));
}

#[test]
fn test_contains_padded_needle_at_start() {
    // Python oracle: "hel" in "hello" == True
    let s: str<5> = "hello";
    let needle: str<5> = "hel\0\0";
    assert(contains::<5, 5>(s, needle));
}

#[test]
fn test_contains_padded_needle_full_content() {
    // Python oracle: "hello" in "hello" == True
    // needle logical length == haystack logical length in a wider buffer.
    let s: str<5> = "hello";
    let needle: str<7> = "hello\0\0";
    assert(contains::<5, 7>(s, needle));
}

#[test]
fn test_contains_empty_needle() {
    // F&O sec.7.5.3: fn:contains($arg1, "") = true for any $arg1.
    // Python oracle: "" in "hello" == True (Python / XPath agree here)
    let s: str<5> = "hello";
    let needle: str<0> = "";
    assert(contains::<5, 0>(s, needle));
}

#[test]
fn test_contains_empty_needle_in_wider_buf() {
    // Logically-empty needle in a non-zero-capacity buffer also matches.
    let s: str<5> = "hello";
    let needle: str<3> = "\0\0\0";
    assert(contains::<5, 3>(s, needle));
}

#[test]
fn test_contains_padded_needle_longer_than_haystack() {
    // Python oracle: "hello" in "hi" == False
    // Logical needle (5) > logical haystack (2): never matches.
    let s: str<2> = "hi";
    let needle: str<6> = "hello\0";
    assert(!contains::<2, 6>(s, needle));
}

#[test]
fn test_contains_padded_needle_no_match() {
    // Python oracle: "xyz" in "hello" == False
    let s: str<5> = "hello";
    let needle: str<5> = "xyz\0\0";
    assert(!contains::<5, 5>(s, needle));
}
