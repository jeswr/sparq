//! Sequence and aggregate functions for XPath/SPARQL
//!
//! Implements functions that operate on sequences of values.
//! In SPARQL context, these are typically used with GROUP BY.
//!
//! XPath functions implemented:
//! - fn:empty, fn:exists (sequence tests)
//! - fn:count (cardinality)
//! - fn:sum, fn:avg, fn:min, fn:max (aggregates for integers)
//!
//! Note: These work on fixed-size arrays in Noir. For variable-length
//! sequences, the caller must track the actual length.

// ============================================================================
// Sequence Test Functions
// ============================================================================

/// fn:empty for arrays
/// Returns true if the sequence is empty (length is 0)
pub fn is_empty<T, let N: u32>(arr: [T; N]) -> bool {
    N == 0
}

/// fn:exists for arrays
/// Returns true if the sequence is not empty (has at least one item)
pub fn exists<T, let N: u32>(arr: [T; N]) -> bool {
    N > 0
}

/// fn:count for arrays
/// Returns the number of items in the sequence
pub fn count<T, let N: u32>(_arr: [T; N]) -> u32 {
    N
}

// ============================================================================
// Integer Aggregate Functions
// ============================================================================

/// fn:sum for integer arrays
/// Returns the sum of all values
pub fn sum_int<let N: u32>(arr: [i64; N]) -> i64 {
    let mut total: i64 = 0;
    for i in 0..N {
        total += arr[i];
    }
    total
}

/// fn:avg for integer arrays (truncated to integer)
/// Returns the average value (integer division)
/// Panics if array is empty
pub fn avg_int<let N: u32>(arr: [i64; N]) -> i64 {
    assert(N > 0, "Cannot compute average of empty sequence");
    sum_int(arr) / (N as i64)
}

/// fn:min for integer arrays
/// Returns (value, present). XPath F&O: fn:min(()) is the EMPTY SEQUENCE,
/// not an error, so an empty input must stay provable: present == false
/// models the empty result (value is 0 in that case) and the caller
/// branches on it. An assert here would make any circuit containing an
/// empty-sequence min unsatisfiable. The (value, bool) tuple follows this
/// crate's emptiness convention (cf. string_to_integer, fn_number_from_string).
pub fn min_int_seq<let N: u32>(arr: [i64; N]) -> (i64, bool) {
    let mut result: i64 = 0;
    let mut present: bool = false;
    for i in 0..N {
        if (!present) | (arr[i] < result) {
            result = arr[i];
        }
        present = true;
    }
    (result, present)
}

/// fn:max for integer arrays
/// Returns (value, present). XPath F&O: fn:max(()) is the EMPTY SEQUENCE,
/// not an error; present == false models the empty result (value is 0).
/// See min_int_seq for the rationale.
pub fn max_int_seq<let N: u32>(arr: [i64; N]) -> (i64, bool) {
    let mut result: i64 = 0;
    let mut present: bool = false;
    for i in 0..N {
        if (!present) | (arr[i] > result) {
            result = arr[i];
        }
        present = true;
    }
    (result, present)
}

// ============================================================================
// Partial Sequence Functions (with explicit length)
// ============================================================================

/// Sum for arrays with explicit length
/// Only sums the first `len` elements
pub fn sum_int_partial<let N: u32>(arr: [i64; N], len: u32) -> i64 {
    assert(len <= N, "Length exceeds array size");
    let mut total: i64 = 0;
    for i in 0..N {
        if i < len {
            total += arr[i];
        }
    }
    total
}

/// Average for arrays with explicit length
pub fn avg_int_partial<let N: u32>(arr: [i64; N], len: u32) -> i64 {
    assert(len > 0, "Cannot compute average of empty sequence");
    assert(len <= N, "Length exceeds array size");
    sum_int_partial(arr, len) / (len as i64)
}

/// Minimum for arrays with explicit length
/// Returns (value, present): fn:min(()) is the empty sequence per XPath F&O,
/// so len == 0 yields present == false (value 0) instead of an unsatisfiable
/// assert. len is a runtime witness here, so this is the load-bearing case:
/// an assert(len > 0) would make provability data-dependent.
/// The len <= N assert stays: exceeding the array is a caller-contract
/// violation (a genuine error), not an empty sequence.
pub fn min_int_partial<let N: u32>(arr: [i64; N], len: u32) -> (i64, bool) {
    assert(len <= N, "Length exceeds array size");
    let mut result: i64 = 0;
    let mut present: bool = false;
    for i in 0..N {
        if i < len {
            if (!present) | (arr[i] < result) {
                result = arr[i];
            }
            present = true;
        }
    }
    (result, present)
}

/// Maximum for arrays with explicit length
/// Returns (value, present); len == 0 yields present == false (value 0).
/// See min_int_partial for the rationale.
pub fn max_int_partial<let N: u32>(arr: [i64; N], len: u32) -> (i64, bool) {
    assert(len <= N, "Length exceeds array size");
    let mut result: i64 = 0;
    let mut present: bool = false;
    for i in 0..N {
        if i < len {
            if (!present) | (arr[i] > result) {
                result = arr[i];
            }
            present = true;
        }
    }
    (result, present)
}

// ============================================================================
// Boolean Aggregate Functions
// ============================================================================

/// Returns true if all values are true (universal quantification)
pub fn all_true<let N: u32>(arr: [bool; N]) -> bool {
    let mut result = true;
    for i in 0..N {
        result &= arr[i];
    }
    result
}

/// Returns true if any value is true (existential quantification)
pub fn any_true<let N: u32>(arr: [bool; N]) -> bool {
    let mut result = false;
    for i in 0..N {
        result |= arr[i];
    }
    result
}

/// Count of true values in array
pub fn count_true<let N: u32>(arr: [bool; N]) -> u32 {
    let mut count: u32 = 0;
    for i in 0..N {
        if arr[i] {
            count += 1;
        }
    }
    count
}

// ============================================================================
// Sequence Access Functions
// ============================================================================

/// fn:head - Returns the first item in a sequence
/// Panics if the sequence is empty
pub fn head<T, let N: u32>(arr: [T; N]) -> T {
    assert(N > 0, "Cannot get head of empty sequence");
    arr[0]
}

/// fn:tail - Returns all items except the first
/// Returns an array of size N-1
pub fn tail<T, let N: u32, let M: u32>(arr: [T; N]) -> [T; M]
where
    T: Default,
{
    // M should equal N - 1, but we can't express that constraint directly
    assert(N > 0, "Cannot get tail of empty sequence");
    assert(M == N - 1, "Output array size must be N-1");
    let mut result: [T; M] = [T::default(); M];
    for i in 0..M {
        result[i] = arr[i + 1];
    }
    result
}

/// fn:reverse - Returns the sequence in reverse order
pub fn reverse<T, let N: u32>(arr: [T; N]) -> [T; N]
where
    T: Default,
{
    let mut result: [T; N] = [T::default(); N];
    for i in 0..N {
        result[i] = arr[N - 1 - i];
    }
    result
}

/// fn:index-of - Returns positions where a value occurs (1-indexed as per XPath)
/// Returns an array with found positions; unused positions are 0
pub fn index_of<T, let N: u32, let M: u32>(arr: [T; N], value: T) -> ([u32; M], u32)
where
    T: Eq,
{
    let mut result: [u32; M] = [0; M];
    let mut found_count: u32 = 0;
    for i in 0..N {
        if (arr[i] == value) & (found_count < M) {
            result[found_count] = (i + 1) as u32; // XPath uses 1-based indexing
            found_count += 1;
        }
    }
    (result, found_count)
}

/// fn:distinct-values - Returns unique values from a sequence
/// Returns (result_array, count_of_distinct_values)
pub fn distinct_values<T, let N: u32>(arr: [T; N]) -> ([T; N], u32)
where
    T: Eq + Default,
{
    let mut result: [T; N] = [T::default(); N];
    let mut count: u32 = 0;

    for i in 0..N {
        let mut is_duplicate = false;
        for j in 0..N {
            if (j < count) & (arr[i] == result[j]) {
                is_duplicate = true;
            }
        }
        if !is_duplicate {
            result[count] = arr[i];
            count += 1;
        }
    }
    (result, count)
}

/// fn:subsequence - Returns a subsequence starting at position (1-indexed)
/// start_pos: 1-indexed position to start from
/// length: number of items to return
pub fn subsequence<T, let N: u32, let M: u32>(arr: [T; N], start_pos: u32, length: u32) -> [T; M]
where
    T: Default,
{
    let mut result: [T; M] = [T::default(); M];
    // XPath uses 1-based indexing, convert to 0-based
    let start_idx = if start_pos > 0 { start_pos - 1 } else { 0 };

    for i in 0..M {
        let src_idx = start_idx + i;
        if (i < length) & (src_idx < N) {
            result[i] = arr[src_idx];
        }
    }
    result
}

/// fn:zero-or-one - Returns the argument if it has zero or one items, otherwise errors
/// For fixed-size arrays in Noir, this just checks the constraint
pub fn zero_or_one<T, let N: u32>(arr: [T; N]) -> [T; N] {
    assert(N <= 1, "Sequence contains more than one item");
    arr
}

/// fn:one-or-more - Returns the argument if it has one or more items, otherwise errors
pub fn one_or_more<T, let N: u32>(arr: [T; N]) -> [T; N] {
    assert(N >= 1, "Sequence is empty");
    arr
}

/// fn:exactly-one - Returns the argument if it has exactly one item, otherwise errors
pub fn exactly_one<T, let N: u32>(arr: [T; N]) -> [T; N] {
    assert(N == 1, "Sequence does not contain exactly one item");
    arr
}

/// fn:unordered - Returns the sequence in implementation-defined order
/// Since we have no ordering requirements in ZK, this is an identity function
pub fn unordered<T, let N: u32>(arr: [T; N]) -> [T; N] {
    arr
}

/// fn:remove - Removes the item at the given position from the sequence
/// position is 1-based (XPath convention), returns (result_array, new_length)
/// If position is out of range, returns the original sequence unchanged
pub fn remove<T, let N: u32>(arr: [T; N], position: u32, len: u32) -> ([T; N], u32)
where
    T: std::default::Default,
{
    let mut result: [T; N] = arr;
    let mut new_len = len;

    // Position is 1-based, convert to 0-based index
    let pos_valid = (position >= 1) & (position <= len);

    if pos_valid {
        let idx = position - 1;
        // Shift elements left
        for i in 0..N {
            if (i as u32 >= idx) & ((i as u32) < len - 1) {
                result[i] = arr[i + 1];
            }
        }
        // Clear last element
        if len > 0 {
            let last_idx: u32 = len - 1;
            result[last_idx] = T::default();
        }
        new_len = len - 1;
    }

    (result, new_len)
}

/// fn:insert-before - Inserts an item at the given position in the sequence
/// position is 1-based (XPath convention), returns (result_array, new_length)
/// If position < 1, inserts at the beginning
/// If position > length, appends at the end
pub fn insert_before<T, let N: u32>(arr: [T; N], position: u32, len: u32, item: T) -> ([T; N], u32)
where
    T: std::default::Default,
{
    assert(len < N, "Cannot insert: array is full");

    let mut result: [T; N] = [T::default(); N];

    // Normalize position: XPath uses 1-based indexing
    let insert_idx: u32 = if position < 1 {
        0
    } else if position > len {
        len
    } else {
        position - 1
    };

    // Copy elements before insertion point
    for i in 0..N {
        if (i as u32) < insert_idx {
            result[i] = arr[i];
        }
    }

    // Insert the new item
    result[insert_idx] = item;

    // Copy elements after insertion point (shifted by 1)
    for i in 0..N {
        if (i as u32 >= insert_idx) & ((i as u32) < len) {
            result[i + 1] = arr[i];
        }
    }

    (result, len + 1)
}

/// fn:sort - Sorts a sequence of integers in ascending order (bubble sort)
/// Returns (sorted_array, length)
pub fn sort_int<let N: u32>(arr: [i64; N], len: u32) -> [i64; N] {
    let mut result = arr;

    // Bubble sort - simple but O(n^2)
    for _pass in 0..N {
        for i in 0..N {
            if (i as u32) + 1 < len {
                let j = i + 1;
                if result[i] > result[j] {
                    let temp = result[i];
                    result[i] = result[j];
                    result[j] = temp;
                }
            }
        }
    }

    result
}

/// fn:deep-equal for integer arrays - Compares two sequences for deep equality
pub fn deep_equal_int<let N: u32, let M: u32>(
    arr1: [i64; N],
    len1: u32,
    arr2: [i64; M],
    len2: u32,
) -> bool {
    if len1 != len2 {
        false
    } else {
        let mut equal = true;
        for i in 0..N {
            if (i as u32) < len1 {
                // Need to check bounds for arr2 as well
                if (i as u32) < M {
                    if arr1[i] != arr2[i] {
                        equal = false;
                    }
                } else {
                    equal = false;
                }
            }
        }
        equal
    }
}

// ============================================================================
// Set Operations (op:union, op:intersect, op:except)
// ============================================================================

/// op:union - Returns the union of two sequences (distinct values from both)
/// Result contains all distinct values from both sequences in order of first occurrence
pub fn union_int<let N: u32, let M: u32, let R: u32>(
    arr1: [i64; N],
    len1: u32,
    arr2: [i64; M],
    len2: u32,
) -> ([i64; R], u32)
where
    [i64; R]: std::default::Default,
{
    let mut result: [i64; R] = [0; R];
    let mut result_len: u32 = 0;

    // Add all distinct values from arr1
    for i in 0..N {
        if (i as u32) < len1 {
            let val = arr1[i];
            // Check if already in result
            let mut found = false;
            for j in 0..R {
                if (j as u32) < result_len {
                    if result[j] == val {
                        found = true;
                    }
                }
            }
            if !found & (result_len < R) {
                result[result_len] = val;
                result_len += 1;
            }
        }
    }

    // Add distinct values from arr2 not already in result
    for i in 0..M {
        if (i as u32) < len2 {
            let val = arr2[i];
            // Check if already in result
            let mut found = false;
            for j in 0..R {
                if (j as u32) < result_len {
                    if result[j] == val {
                        found = true;
                    }
                }
            }
            if !found & (result_len < R) {
                result[result_len] = val;
                result_len += 1;
            }
        }
    }

    (result, result_len)
}

/// op:intersect - Returns the intersection of two sequences
/// Result contains values that appear in both sequences (distinct)
pub fn intersect_int<let N: u32, let M: u32, let R: u32>(
    arr1: [i64; N],
    len1: u32,
    arr2: [i64; M],
    len2: u32,
) -> ([i64; R], u32)
where
    [i64; R]: std::default::Default,
{
    let mut result: [i64; R] = [0; R];
    let mut result_len: u32 = 0;

    // For each value in arr1, check if it's in arr2 and not already in result
    for i in 0..N {
        if (i as u32) < len1 {
            let val = arr1[i];

            // Check if val is in arr2
            let mut in_arr2 = false;
            for j in 0..M {
                if (j as u32) < len2 {
                    if arr2[j] == val {
                        in_arr2 = true;
                    }
                }
            }

            // Check if already in result
            let mut in_result = false;
            for k in 0..R {
                if (k as u32) < result_len {
                    if result[k] == val {
                        in_result = true;
                    }
                }
            }

            if in_arr2 & !in_result & (result_len < R) {
                result[result_len] = val;
                result_len += 1;
            }
        }
    }

    (result, result_len)
}

/// op:except - Returns the difference of two sequences (values in first but not in second)
/// Result contains distinct values from arr1 that are not in arr2
pub fn except_int<let N: u32, let M: u32, let R: u32>(
    arr1: [i64; N],
    len1: u32,
    arr2: [i64; M],
    len2: u32,
) -> ([i64; R], u32)
where
    [i64; R]: std::default::Default,
{
    let mut result: [i64; R] = [0; R];
    let mut result_len: u32 = 0;

    // For each value in arr1, check if it's NOT in arr2 and not already in result
    for i in 0..N {
        if (i as u32) < len1 {
            let val = arr1[i];

            // Check if val is in arr2
            let mut in_arr2 = false;
            for j in 0..M {
                if (j as u32) < len2 {
                    if arr2[j] == val {
                        in_arr2 = true;
                    }
                }
            }

            // Check if already in result
            let mut in_result = false;
            for k in 0..R {
                if (k as u32) < result_len {
                    if result[k] == val {
                        in_result = true;
                    }
                }
            }

            if !in_arr2 & !in_result & (result_len < R) {
                result[result_len] = val;
                result_len += 1;
            }
        }
    }

    (result, result_len)
}

// ============================================================================
// Range Operator (op:to)
// ============================================================================

/// op:to - Generates a sequence of integers from start to end (inclusive)
/// Returns (result_array, length)
/// If start > end, returns an empty sequence
pub fn range_to<let N: u32>(start: i64, end: i64) -> ([i64; N], u32)
where
    [i64; N]: std::default::Default,
{
    let mut result: [i64; N] = [0; N];
    let mut result_len: u32 = 0;

    if start <= end {
        let count = (end - start + 1) as u32;
        let actual_len = if count > N { N } else { count };

        for i in 0..N {
            if (i as u32) < actual_len {
                result[i] = start + (i as i64);
            }
        }
        result_len = actual_len;
    }

    (result, result_len)
}

// ============================================================================
// Sequence Concatenation (op:concatenate)
// ============================================================================

/// op:concatenate - Concatenates two sequences
/// Returns (result_array, length)
pub fn concatenate<T, let N: u32, let M: u32, let R: u32>(
    arr1: [T; N],
    len1: u32,
    arr2: [T; M],
    len2: u32,
) -> ([T; R], u32)
where
    T: std::default::Default,
{
    let mut result: [T; R] = [T::default(); R];
    let mut result_len: u32 = 0;

    // Copy first array
    for i in 0..N {
        if ((i as u32) < len1) & (result_len < R) {
            result[result_len] = arr1[i];
            result_len += 1;
        }
    }

    // Copy second array
    for i in 0..M {
        if ((i as u32) < len2) & (result_len < R) {
            result[result_len] = arr2[i];
            result_len += 1;
        }
    }

    (result, result_len)
}

// ============================================================================
// Context Functions
// ============================================================================

/// fn:last - Returns the context size (passed as parameter since context isn't tracked)
/// XPath: fn:last() as xs:integer
/// In ZK context, this must be passed as input since there's no dynamic context
pub fn last(context_size: u32) -> u32 {
    context_size
}

/// fn:position - Returns the context position (passed as parameter since context isn't tracked)
/// XPath: fn:position() as xs:integer
/// In ZK context, this must be passed as input since there's no dynamic context
pub fn position(context_position: u32) -> u32 {
    context_position
}

// ============================================================================
// Context and Collation Functions
// ============================================================================

/// fn:implicit-timezone - Returns the implicit timezone
/// XPath: fn:implicit-timezone() as xs:dayTimeDuration
/// In ZK context, this must be passed as input since there's no dynamic context.
/// Returns timezone offset in minutes from UTC.
/// Common values: 0 (UTC), -300 (EST), -480 (PST), 60 (CET)
pub fn implicit_timezone(tz_offset_minutes: i32) -> i32 {
    tz_offset_minutes
}

/// fn:default-collation - Returns the default collation URI
/// XPath: fn:default-collation() as xs:string
/// Returns the Unicode Codepoint Collation URI as bytes.
/// In practice, most implementations use codepoint ordering by default.
pub fn default_collation<let N: u32>() -> ([u8; N], u32) {
    // Unicode Codepoint Collation URI
    // "http://www.w3.org/2005/xpath-functions/collation/codepoint"
    let uri: [u8; 58] = [
        104, 116, 116, 112, 58, 47, 47, 119, 119, 119, 46, 119, 51, 46, 111, 114, 103, 47, 50, 48,
        48, 53, 47, 120, 112, 97, 116, 104, 45, 102, 117, 110, 99, 116, 105, 111, 110, 115, 47, 99,
        111, 108, 108, 97, 116, 105, 111, 110, 47, 99, 111, 100, 101, 112, 111, 105, 110, 116,
    ];

    let mut result: [u8; N] = [0; N];
    let len: u32 = if 58 > N { N } else { 58 };

    for i in 0..N {
        if (i as u32) < len {
            result[i] = uri[i];
        }
    }

    (result, len)
}

/// fn:default-language - Returns the default language
/// XPath: fn:default-language() as xs:language
/// Returns "en" (English) as the default language tag.
/// In a full implementation, this could be passed as a parameter.
pub fn default_language<let N: u32>() -> ([u8; N], u32) {
    // "en" as default language
    let mut result: [u8; N] = [0; N];
    let len: u32 = if 2 > N { N } else { 2 };

    if N >= 1 {
        result[0] = 101; // 'e'
    }
    if N >= 2 {
        result[1] = 110; // 'n'
    }

    (result, len)
}

/// fn:data - Atomization: extracts the typed value from a node or atomic value
/// XPath: fn:data($arg as item()*) as xs:anyAtomicType*
/// For atomic values (which is all we have in ZK context), this is an identity function.
/// The value is already atomic, so we just return it.
/// This is a generic marker - actual implementations would be type-specific.
pub fn fn_data_identity<T>(value: T) -> T {
    value
}

// ============================================================================
// Tests
// ============================================================================

#[test]
fn test_sequence_empty_exists() {
    let empty: [i64; 0] = [];
    let non_empty: [i64; 3] = [1, 2, 3];

    assert(is_empty(empty) == true);
    assert(is_empty(non_empty) == false);

    assert(exists(empty) == false);
    assert(exists(non_empty) == true);
}

#[test]
fn test_count() {
    let arr: [i64; 5] = [1, 2, 3, 4, 5];
    assert(count(arr) == 5);

    let empty: [i64; 0] = [];
    assert(count(empty) == 0);
}

#[test]
fn test_sum_int() {
    let arr: [i64; 5] = [1, 2, 3, 4, 5];
    assert(sum_int(arr) == 15);

    let negative: [i64; 3] = [-1, -2, 3];
    assert(sum_int(negative) == 0);
}

#[test]
fn test_avg_int() {
    let arr: [i64; 5] = [10, 20, 30, 40, 50];
    assert(avg_int(arr) == 30);

    // Integer division truncates
    let arr2: [i64; 3] = [1, 2, 3];
    assert(avg_int(arr2) == 2); // (1+2+3)/3 = 2
}

#[test]
fn test_min_max_int_seq() {
    let arr: [i64; 5] = [3, 1, 4, 1, 5];
    assert(min_int_seq(arr) == (1, true));
    assert(max_int_seq(arr) == (5, true));

    let negative: [i64; 3] = [-5, 0, 5];
    assert(min_int_seq(negative) == (-5, true));
    assert(max_int_seq(negative) == (5, true));

    // Single-element sequence: the element itself, present
    let single: [i64; 1] = [42];
    assert(min_int_seq(single) == (42, true));
    assert(max_int_seq(single) == (42, true));
}

#[test]
fn test_min_max_int_seq_empty() {
    // fn:min(()) / fn:max(()) = () per XPath F&O: this test EXECUTES
    // min/max on an empty sequence, so the circuit stays satisfiable
    // (with the old assert this test would abort, i.e. be unprovable).
    let empty: [i64; 0] = [];
    let (min_val, min_present) = min_int_seq(empty);
    assert(!min_present);
    assert(min_val == 0);
    let (max_val, max_present) = max_int_seq(empty);
    assert(!max_present);
    assert(max_val == 0);
}

#[test]
fn test_min_max_int_partial_empty() {
    // len == 0 (a runtime witness, the load-bearing circuit case):
    // empty in -> present == false out, no unsatisfiable assert.
    let arr: [i64; 4] = [7, 8, 9, 10];
    assert(min_int_partial(arr, 0) == (0, false));
    assert(max_int_partial(arr, 0) == (0, false));
}

#[test(should_fail_with = "Cannot compute average of empty sequence")]
fn test_avg_int_partial_empty_still_errors() {
    // fn:avg over an empty group deliberately stays an error in this
    // library (see avg_int_partial); the empty-sequence fix is scoped
    // to min/max only.
    let arr: [i64; 3] = [1, 2, 3];
    let _ = avg_int_partial(arr, 0);
}

#[test]
fn test_partial_aggregates() {
    let arr: [i64; 5] = [10, 20, 30, 40, 50];

    // Sum first 3 elements
    assert(sum_int_partial(arr, 3) == 60);

    // Average of first 2
    assert(avg_int_partial(arr, 2) == 15);

    // Min/max of first 3
    assert(min_int_partial(arr, 3) == (10, true));
    assert(max_int_partial(arr, 3) == (30, true));
}

#[test]
fn test_boolean_aggregates() {
    let all_t: [bool; 3] = [true, true, true];
    let some_t: [bool; 3] = [true, false, true];
    let all_f: [bool; 3] = [false, false, false];

    assert(all_true(all_t) == true);
    assert(all_true(some_t) == false);
    assert(all_true(all_f) == false);

    assert(any_true(all_t) == true);
    assert(any_true(some_t) == true);
    assert(any_true(all_f) == false);

    assert(count_true(all_t) == 3);
    assert(count_true(some_t) == 2);
    assert(count_true(all_f) == 0);
}

#[test]
fn test_head() {
    let arr: [i64; 3] = [10, 20, 30];
    assert(head(arr) == 10);
}

#[test]
fn test_reverse() {
    let arr: [i64; 4] = [1, 2, 3, 4];
    let reversed = reverse(arr);
    assert(reversed[0] == 4);
    assert(reversed[1] == 3);
    assert(reversed[2] == 2);
    assert(reversed[3] == 1);
}

#[test]
fn test_distinct_values() {
    let arr: [i64; 5] = [1, 2, 1, 3, 2];
    let (result, count) = distinct_values(arr);
    assert(count == 3);
    assert(result[0] == 1);
    assert(result[1] == 2);
    assert(result[2] == 3);
}
