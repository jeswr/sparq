//! XPath `op:` namespace operators
//!
//! This module provides a deterministic mapping from XPath operator names in the `op:` namespace
//! to their Noir implementations. The naming follows the pattern:
//!
//! - XPath `op:operator-name` -> `xpath_op::operator_name` (hyphens become underscores)
//!
//! This enables code generators to compose function names programmatically by:
//! 1. Taking the XPath operator name (e.g., `op:numeric-add`)
//! 2. Replacing `op:` with `xpath_op::` and `-` with `_`
//! 3. Accessing the function directly (e.g., `xpath_op::numeric_add`)
//!
//! ## Usage
//!
//! ```noir
//! use xpath::xpath_op;
//!
//! // op:numeric-add
//! let sum = xpath_op::numeric_add(5, 3);
//!
//! // op:dateTime-equal
//! let equal = xpath_op::dateTime_equal(dt1, dt2);
//! ```

// ============================================================================
// Numeric operators (integer)
// ============================================================================

// op:numeric-add
pub use crate::numeric::numeric_add_int as numeric_add;

// op:numeric-subtract
pub use crate::numeric::numeric_subtract_int as numeric_subtract;

// op:numeric-multiply
pub use crate::numeric::numeric_multiply_int as numeric_multiply;

// op:numeric-divide -- two xs:integer operands yield the xs:decimal quotient
// (7 div 2 = 3.5), computed in IEEE 754 double precision (see the doc on
// numeric_divide_int_as_double). NOT an alias of op:numeric-integer-divide.
pub use crate::numeric_types::numeric_divide_int_as_double as numeric_divide;

// op:numeric-integer-divide (idiv) -- truncating division toward zero.
// Deliberately a DISTINCT code path from op:numeric-divide above.
pub use crate::numeric::numeric_divide_int as numeric_integer_divide;

// op:numeric-mod
pub use crate::numeric::numeric_mod_int as numeric_mod;

// op:numeric-equal
pub use crate::numeric::numeric_equal_int as numeric_equal;

// op:numeric-less-than
pub use crate::numeric::numeric_less_than_int as numeric_less_than;

// op:numeric-greater-than
pub use crate::numeric::numeric_greater_than_int as numeric_greater_than;

// op:numeric-unary-plus
pub use crate::numeric::numeric_unary_plus_int as numeric_unary_plus;

// op:numeric-unary-minus
pub use crate::numeric::numeric_unary_minus_int as numeric_unary_minus;

// ============================================================================
// Numeric operators (float)
// ============================================================================

// op:numeric-add (float)
pub use crate::numeric_types::numeric_add_float;

// op:numeric-subtract (float)
pub use crate::numeric_types::numeric_subtract_float;

// op:numeric-multiply (float)
pub use crate::numeric_types::numeric_multiply_float;

// op:numeric-divide (float)
pub use crate::numeric_types::numeric_divide_float;

// op:numeric-equal (float)
pub use crate::numeric_types::numeric_equal_float;

// op:numeric-less-than (float)
pub use crate::numeric_types::numeric_less_than_float;

// op:numeric-greater-than (float)
pub use crate::numeric_types::numeric_greater_than_float;

// op:numeric-less-than-or-equal (float)
pub use crate::numeric_types::numeric_le_float;

// op:numeric-greater-than-or-equal (float)
pub use crate::numeric_types::numeric_ge_float;

// ============================================================================
// Numeric operators (double)
// ============================================================================

// op:numeric-add (double)
pub use crate::numeric_types::numeric_add_double;

// op:numeric-subtract (double)
pub use crate::numeric_types::numeric_subtract_double;

// op:numeric-multiply (double)
pub use crate::numeric_types::numeric_multiply_double;

// op:numeric-divide (double)
pub use crate::numeric_types::numeric_divide_double;

// op:numeric-equal (double)
pub use crate::numeric_types::numeric_equal_double;

// op:numeric-less-than (double)
pub use crate::numeric_types::numeric_less_than_double;

// op:numeric-greater-than (double)
pub use crate::numeric_types::numeric_greater_than_double;

// op:numeric-less-than-or-equal (double)
pub use crate::numeric_types::numeric_le_double;

// op:numeric-greater-than-or-equal (double)
pub use crate::numeric_types::numeric_ge_double;

// ============================================================================
// Boolean operators
// ============================================================================

// op:boolean-equal
pub use crate::boolean::boolean_equal;

// op:boolean-less-than
pub use crate::boolean::boolean_less_than;

// op:boolean-greater-than
pub use crate::boolean::boolean_greater_than;

// op:boolean-less-than-or-equal
pub use crate::boolean::boolean_le as boolean_less_than_or_equal;

// op:boolean-greater-than-or-equal
pub use crate::boolean::boolean_ge as boolean_greater_than_or_equal;

// ============================================================================
// DateTime operators
// ============================================================================

// op:dateTime-equal
pub use crate::datetime::datetime_equal as dateTime_equal;

// op:dateTime-less-than
pub use crate::datetime::datetime_less_than as dateTime_less_than;

// op:dateTime-greater-than
pub use crate::datetime::datetime_greater_than as dateTime_greater_than;

// op:dateTime-less-than-or-equal
pub use crate::datetime::datetime_le as dateTime_less_than_or_equal;

// op:dateTime-greater-than-or-equal
pub use crate::datetime::datetime_ge as dateTime_greater_than_or_equal;

// ============================================================================
// Date operators
// ============================================================================

// op:date-equal
pub use crate::date::date_equal;

// op:date-less-than
pub use crate::date::date_less_than;

// op:date-greater-than
pub use crate::date::date_greater_than;

// op:date-less-than-or-equal
pub use crate::date::date_le as date_less_than_or_equal;

// op:date-greater-than-or-equal
pub use crate::date::date_ge as date_greater_than_or_equal;

// ============================================================================
// Time operators
// ============================================================================

// op:time-equal
pub use crate::time::time_equal;

// op:time-less-than
pub use crate::time::time_less_than;

// op:time-greater-than
pub use crate::time::time_greater_than;

// op:time-less-than-or-equal
pub use crate::time::time_le as time_less_than_or_equal;

// op:time-greater-than-or-equal
pub use crate::time::time_ge as time_greater_than_or_equal;

// op:subtract-times
pub use crate::time::subtract_times;

// op:add-dayTimeDuration-to-time
pub use crate::time::time_add_duration as add_dayTimeDuration_to_time;

// op:subtract-dayTimeDuration-from-time
pub use crate::time::time_subtract_duration as subtract_dayTimeDuration_from_time;

// ============================================================================
// Duration operators
// ============================================================================

// op:add-dayTimeDurations
pub use crate::duration::duration_add as add_dayTimeDurations;

// op:subtract-dayTimeDurations
pub use crate::duration::duration_subtract as subtract_dayTimeDurations;

// op:multiply-dayTimeDuration
pub use crate::duration::duration_multiply as multiply_dayTimeDuration;

// op:divide-dayTimeDuration
pub use crate::duration::duration_divide as divide_dayTimeDuration;

// op:divide-dayTimeDuration-by-dayTimeDuration
pub use crate::duration::duration_divide_by_duration as divide_dayTimeDuration_by_dayTimeDuration;

// op:dayTimeDuration-equal
pub use crate::duration::duration_equal as dayTimeDuration_equal;

// op:dayTimeDuration-less-than
pub use crate::duration::duration_less_than as dayTimeDuration_less_than;

// op:dayTimeDuration-greater-than
pub use crate::duration::duration_greater_than as dayTimeDuration_greater_than;

// op:dayTimeDuration-less-than-or-equal
pub use crate::duration::duration_le as dayTimeDuration_less_than_or_equal;

// op:dayTimeDuration-greater-than-or-equal
pub use crate::duration::duration_ge as dayTimeDuration_greater_than_or_equal;

// ============================================================================
// DateTime + Duration operators
// ============================================================================

// op:add-dayTimeDuration-to-dateTime
pub use crate::duration::datetime_add_duration as add_dayTimeDuration_to_dateTime;

// op:subtract-dayTimeDuration-from-dateTime
pub use crate::duration::datetime_subtract_duration as subtract_dayTimeDuration_from_dateTime;

// op:subtract-dateTimes
pub use crate::duration::datetime_difference as subtract_dateTimes;

// ============================================================================
// String operators
// ============================================================================

// op:string-equal
pub use crate::string::string_equal;

// op:string-less-than
pub use crate::string::string_less_than;

// op:string-greater-than
pub use crate::string::string_greater_than;

// ============================================================================
// anyURI operators (treat as strings)
// ============================================================================

// op:anyURI-equal
pub use crate::string::anyuri_equal as anyURI_equal;

// op:anyURI-less-than
pub use crate::string::anyuri_less_than as anyURI_less_than;

// op:anyURI-greater-than
pub use crate::string::anyuri_greater_than as anyURI_greater_than;

// ============================================================================
// Date + Duration operators
// ============================================================================

// op:add-dayTimeDuration-to-date
pub use crate::date::date_add_duration as add_dayTimeDuration_to_date;

// op:subtract-dayTimeDuration-from-date
pub use crate::date::date_subtract_duration as subtract_dayTimeDuration_from_date;

// op:subtract-dates
pub use crate::date::subtract_dates;

// ============================================================================
// Binary comparison operators
// ============================================================================

// op:hexBinary-equal
pub use crate::binary::hexbinary_equal as hexBinary_equal;

// op:hexBinary-less-than
pub use crate::binary::hexbinary_less_than as hexBinary_less_than;

// op:hexBinary-greater-than
pub use crate::binary::hexbinary_greater_than as hexBinary_greater_than;

// op:base64Binary-equal
pub use crate::binary::base64binary_equal as base64Binary_equal;

// op:base64Binary-less-than
pub use crate::binary::base64binary_less_than as base64Binary_less_than;

// op:base64Binary-greater-than
pub use crate::binary::base64binary_greater_than as base64Binary_greater_than;

// ============================================================================
// Gregorian comparison operators
// ============================================================================

// op:gYear-equal
pub use crate::gregorian::gyear_equal as gYear_equal;

// op:gMonth-equal
pub use crate::gregorian::gmonth_equal as gMonth_equal;

// op:gDay-equal
pub use crate::gregorian::gday_equal as gDay_equal;

// op:gYearMonth-equal
pub use crate::gregorian::gyearmonth_equal as gYearMonth_equal;

// op:gMonthDay-equal
pub use crate::gregorian::gmonthday_equal as gMonthDay_equal;

// ============================================================================
// YearMonthDuration operators
// ============================================================================

// op:add-yearMonthDurations
pub use crate::year_month_duration::ym_duration_add as add_yearMonthDurations;

// op:subtract-yearMonthDurations
pub use crate::year_month_duration::ym_duration_subtract as subtract_yearMonthDurations;

// op:multiply-yearMonthDuration
pub use crate::year_month_duration::ym_duration_multiply as multiply_yearMonthDuration;

// op:divide-yearMonthDuration
pub use crate::year_month_duration::ym_duration_divide as divide_yearMonthDuration;

// op:divide-yearMonthDuration-by-yearMonthDuration
pub use crate::year_month_duration::ym_duration_divide_by_duration as divide_yearMonthDuration_by_yearMonthDuration;

// op:yearMonthDuration-equal
pub use crate::year_month_duration::ym_duration_equal as yearMonthDuration_equal;

// op:yearMonthDuration-less-than
pub use crate::year_month_duration::ym_duration_less_than as yearMonthDuration_less_than;

// op:yearMonthDuration-greater-than
pub use crate::year_month_duration::ym_duration_greater_than as yearMonthDuration_greater_than;

// op:yearMonthDuration-less-than-or-equal
pub use crate::year_month_duration::ym_duration_le as yearMonthDuration_less_than_or_equal;

// op:yearMonthDuration-greater-than-or-equal
pub use crate::year_month_duration::ym_duration_ge as yearMonthDuration_greater_than_or_equal;

// op:add-yearMonthDuration-to-dateTime
pub use crate::year_month_duration::datetime_add_ym_duration as add_yearMonthDuration_to_dateTime;

// op:subtract-yearMonthDuration-from-dateTime
pub use crate::year_month_duration::datetime_subtract_ym_duration as subtract_yearMonthDuration_from_dateTime;

// op:add-yearMonthDuration-to-date
pub use crate::year_month_duration::date_add_ym_duration as add_yearMonthDuration_to_date;

// op:subtract-yearMonthDuration-from-date
pub use crate::year_month_duration::date_subtract_ym_duration as subtract_yearMonthDuration_from_date;

// ============================================================================
// Sequence set operators
// ============================================================================

// op:union
pub use crate::sequence::union_int as union;

// op:intersect
pub use crate::sequence::intersect_int as intersect;

// op:except
pub use crate::sequence::except_int as except;

// op:to (range operator)
pub use crate::sequence::range_to as to;

// ============================================================================
// QName operators
// ============================================================================

// op:QName-equal
pub use crate::qname::qname_equal as QName_equal;

// op:NOTATION-equal (F&O sec. 9.1: equality by expanded name, same as QName)
pub use crate::qname::notation_equal as NOTATION_equal;
