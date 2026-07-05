//! XPath `fn:` namespace functions
//!
//! This module provides a deterministic mapping from XPath function names in the `fn:` namespace
//! to their Noir implementations. The naming follows the pattern:
//!
//! - XPath `fn:function-name` -> `xpath_fn::function_name` (hyphens become underscores)
//!
//! This enables code generators to compose function names programmatically by:
//! 1. Taking the XPath function name (e.g., `fn:year-from-dateTime`)
//! 2. Replacing `fn:` with `xpath_fn::` and `-` with `_`
//! 3. Accessing the function directly (e.g., `xpath_fn::year_from_dateTime`)
//!
//! ## Usage
//!
//! ```noir
//! use xpath::xpath_fn;
//!
//! // fn:abs
//! let result = xpath_fn::abs(-5);
//!
//! // fn:year-from-dateTime
//! let year = xpath_fn::year_from_dateTime(dt);
//! ```

// Re-export numeric functions
// fn:abs
pub use crate::numeric::abs_int as abs;

// fn:ceiling
pub use crate::numeric::ceil_int as ceiling;

// fn:floor
pub use crate::numeric::floor_int as floor;

// fn:round
pub use crate::numeric::round_int as round;

// fn:not
pub use crate::boolean::fn_not as not;

// fn:true - using fn_true since 'true' is a reserved keyword in Noir
pub use crate::boolean::fn_true;

// fn:false - using fn_false since 'false' is a reserved keyword in Noir
pub use crate::boolean::fn_false;

// Re-export datetime component extraction functions
// fn:year-from-dateTime
pub use crate::datetime::year_from_datetime as year_from_dateTime;

// fn:month-from-dateTime
pub use crate::datetime::month_from_datetime as month_from_dateTime;

// fn:day-from-dateTime
pub use crate::datetime::day_from_datetime as day_from_dateTime;

// fn:hours-from-dateTime
pub use crate::datetime::hours_from_datetime as hours_from_dateTime;

// fn:minutes-from-dateTime
pub use crate::datetime::minutes_from_datetime as minutes_from_dateTime;

// fn:seconds-from-dateTime
pub use crate::datetime::seconds_from_datetime as seconds_from_dateTime;

// fn:timezone-from-dateTime
pub use crate::datetime::timezone_from_datetime as timezone_from_dateTime;

// Re-export duration component extraction functions
// fn:days-from-duration
pub use crate::duration::days_from_duration as days_from_duration;

// fn:hours-from-duration
pub use crate::duration::hours_from_duration as hours_from_duration;

// fn:minutes-from-duration
pub use crate::duration::minutes_from_duration as minutes_from_duration;

// fn:seconds-from-duration
pub use crate::duration::seconds_from_duration as seconds_from_duration;

// Re-export date component extraction functions
// fn:year-from-date
pub use crate::date::year_from_date as year_from_date;

// fn:month-from-date
pub use crate::date::month_from_date as month_from_date;

// fn:day-from-date
pub use crate::date::day_from_date as day_from_date;

// fn:timezone-from-date
pub use crate::date::timezone_from_date as timezone_from_date;

// Re-export time component extraction functions
// fn:hours-from-time
pub use crate::time::hours_from_time as hours_from_time;

// fn:minutes-from-time
pub use crate::time::minutes_from_time as minutes_from_time;

// fn:seconds-from-time
pub use crate::time::seconds_from_time as seconds_from_time;

// fn:timezone-from-time
pub use crate::time::timezone_from_time as timezone_from_time;

// Re-export sequence functions
// fn:empty
pub use crate::sequence::is_empty as empty;

// fn:exists
pub use crate::sequence::exists;

// fn:count
pub use crate::sequence::count;

// fn:sum (integer version)
pub use crate::sequence::sum_int as sum;

// fn:avg (integer version)
pub use crate::sequence::avg_int as avg;

// fn:min (integer sequence version) - returns (value, present);
// present == false is the empty sequence (fn:min(()) = () per F&O)
pub use crate::sequence::min_int_seq as min;

// fn:max (integer sequence version) - returns (value, present);
// present == false is the empty sequence (fn:max(()) = () per F&O)
pub use crate::sequence::max_int_seq as max;

// fn:head
pub use crate::sequence::head;

// fn:tail
pub use crate::sequence::tail;

// fn:reverse
pub use crate::sequence::reverse;

// fn:index-of
pub use crate::sequence::index_of;

// fn:distinct-values
pub use crate::sequence::distinct_values;

// fn:subsequence
pub use crate::sequence::subsequence;

// fn:zero-or-one
pub use crate::sequence::zero_or_one;

// fn:one-or-more
pub use crate::sequence::one_or_more;

// fn:exactly-one
pub use crate::sequence::exactly_one;

// Re-export string functions
// fn:string-length
pub use crate::string::string_length as string_length;

// fn:starts-with
pub use crate::string::starts_with as starts_with;

// fn:ends-with
pub use crate::string::ends_with as ends_with;

// fn:contains
pub use crate::string::contains;

// fn:compare
pub use crate::string::compare;

// fn:codepoint-equal
pub use crate::string::codepoint_equal;

// fn:string-to-codepoints
pub use crate::string::string_to_codepoints;

// ============================================================================
// [OPUS-4.8] sq-y73: regex (bounded subset) + hash (circuit-native).
//
// NOTE: `fn:matches` / `fn:replace` proper (full XML-Schema regex) remain
// STUBS (see stubs.nr) because full PCRE is not circuit-feasible. The bounded
// circuit-friendly primitives are exposed under explicit `matches_*` /
// `replace_literal` names so a code generator picks the bounded semantics
// deliberately, never mistaking them for full regex. Similarly the SPARQL
// MD5/SHA* hash builtins are NOT provided (no stdlib digest at this toolchain
// pin); `string_pedersen_hash` is the circuit-native content hash. See the
// regex.nr / hash.nr module headers for the full scope rationale.
// ============================================================================

// fn:matches (bounded: literal / char-class / anchored subset)
pub use crate::regex::matches_anchored;
pub use crate::regex::matches_char_class_any;
pub use crate::regex::matches_char_class_plus;
pub use crate::regex::matches_char_class_star;
pub use crate::regex::matches_literal;
pub use crate::regex::matches_prefix;

// fn:replace (bounded: literal pattern + literal replacement)
pub use crate::regex::replace_literal;

// hash: circuit-native content hash (NOT SPARQL MD5/SHA*)
pub use crate::hash::bytes_to_lower_hex;
pub use crate::hash::string_pedersen_hash;

// ============================================================================
// Float/Double variants
// ============================================================================

// fn:abs (float)
pub use crate::numeric_types::abs_float;

// fn:abs (double)
pub use crate::numeric_types::abs_double;

// fn:round (float)
pub use crate::numeric_types::round_float;

// fn:round (double)
pub use crate::numeric_types::round_double;

// fn:ceiling (float)
pub use crate::numeric_types::ceil_float as ceiling_float;

// fn:ceiling (double)
pub use crate::numeric_types::ceil_double as ceiling_double;

// fn:floor (float)
pub use crate::numeric_types::floor_float;

// fn:floor (double)
pub use crate::numeric_types::floor_double;

// ============================================================================
// Additional Sequence Functions
// ============================================================================

// fn:unordered
pub use crate::sequence::unordered;

// fn:remove
pub use crate::sequence::remove;

// fn:insert-before
pub use crate::sequence::insert_before;

// fn:sort (integer version)
pub use crate::sequence::sort_int as sort;

// fn:deep-equal (integer version)
pub use crate::sequence::deep_equal_int as deep_equal;

// ============================================================================
// DateTime Construction
// ============================================================================

// fn:dateTime
pub use crate::date::fn_dateTime as dateTime;

// ============================================================================
// Timezone Adjustment Functions
// ============================================================================

// fn:adjust-dateTime-to-timezone
pub use crate::datetime::adjust_datetime_to_timezone as adjust_dateTime_to_timezone;

// fn:adjust-date-to-timezone
pub use crate::date::adjust_date_to_timezone as adjust_date_to_timezone;

// fn:adjust-time-to-timezone
pub use crate::time::adjust_time_to_timezone as adjust_time_to_timezone;

// ============================================================================
// Rounding Functions
// ============================================================================

// fn:round-half-to-even
pub use crate::numeric::round_half_to_even_int as round_half_to_even;

// ============================================================================
// Duration Component Extraction (sq-u91ki) [SONNET-4.6]
// Generic dispatch on the duration subtype via the YearMonthComponents trait:
// xs:dayTimeDuration -> 0 (F&O sec. 8.4.3 / sec. 8.4.4),
// xs:yearMonthDuration -> real component values.
// ============================================================================

// fn:years-from-duration (dayTimeDuration -> 0, yearMonthDuration -> real years)
pub use crate::duration::years_from_duration as years_from_duration;

// fn:months-from-duration (dayTimeDuration -> 0, yearMonthDuration -> real months)
pub use crate::duration::months_from_duration as months_from_duration;
