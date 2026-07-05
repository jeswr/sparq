# noir_XPath Architecture

A Noir library implementing XPath 2.0 functions and operators required by SPARQL 1.1.

## Overview

This library provides Noir implementations of XPath/XQuery functions as defined in [XQuery 1.0 and XPath 2.0 Functions and Operators](https://www.w3.org/TR/xpath-functions/) that are required by [SPARQL 1.1 Query Language](https://www.w3.org/TR/sparql11-query/).

## Project Structure

Following the pattern established by [noir_IEEE754](https://github.com/jeswr/noir_IEEE754), the project uses a workspace with multiple packages to manage the codebase and tests efficiently:

```text
noir_XPath/
├── Nargo.toml                          # Workspace configuration
├── ARCHITECTURE.md                     # This file
├── SPARQL_COVERAGE.md                  # Per-function implementation status
├── README.md                           # User documentation
│
├── xpath/                              # Main library package
│   ├── Nargo.toml
│   └── src/
│       ├── lib.nr                      # Module exports
│       ├── numeric.nr                  # Numeric functions & operators
│       ├── numeric_types.nr            # Float/double (via noir_IEEE754)
│       ├── boolean.nr                  # Boolean functions & operators
│       ├── datetime.nr                 # DateTime functions
│       ├── date.nr / time.nr           # Date / Time types & functions
│       ├── duration.nr                 # Duration functions
│       ├── string.nr                   # String functions (byte-array tuples)
│       ├── regex.nr / hash.nr          # Bounded regex subset / circuit-native hash
│       ├── comparison.nr               # Comparison operators
│       └── types.nr                    # Type definitions & conversions
│
├── xpath_unit_tests/                   # Manual unit tests
│   ├── Nargo.toml
│   └── src/
│       ├── main.nr
│       ├── numeric_tests.nr
│       ├── datetime_tests.nr
│       └── boolean_tests.nr
│
├── test_packages/                      # Auto-generated test packages
│   ├── xpath_test_fn_abs/             # Tests for fn:abs
│   │   ├── Nargo.toml
│   │   └── src/
│   │       ├── main.nr
│   │       └── chunk_*.nr
│   ├── xpath_test_op_numeric_add/     # Tests for op:numeric-add
│   └── ...
│
└── scripts/
    ├── generate_tests.py               # Test generator from qt3tests
    ├── run_tests.py                    # Local test runner
    └── regenerate_tests.sh             # Regenerate all test packages
```

## Dependencies

### External Libraries

- **ieee754** - [noir_IEEE754](https://github.com/jeswr/noir_IEEE754) for IEEE 754 floating-point arithmetic
  - Used for: `xsd:float` and `xsd:double` operations
  - Provides: `add_float32`, `sub_float32`, `mul_float32`, `div_float32`, etc.

## Data Types

### Supported XML Schema Datatypes (as used in SPARQL)

| XSD Type | Noir Representation | Status | Notes |
|----------|---------------------|--------|-------|
| `xsd:integer` | `i128` or `Field` | ✅ Supported | Arbitrary precision integers |
| `xsd:decimal` | - | 🔮 Future | Deferred for complexity |
| `xsd:float` | `u32` (IEEE 754 bits) | ✅ Supported | Via noir_IEEE754 |
| `xsd:double` | `u64` (IEEE 754 bits) | ✅ Supported | Via noir_IEEE754 |
| `xsd:string` | `([u8; N], u32)` byte-array tuple | ✅ Supported | See `string.nr`; codepoint-vs-byte substring caveat tracked in sq-hjvte |
| `xsd:boolean` | `bool` | ✅ Supported | Native Noir bool |
| `xsd:dateTime` | `Field` (epoch microseconds) | ✅ Supported | Single Field for efficiency |

### Type Structs

```noir
// DateTime representation - single Field for circuit efficiency
// Stores microseconds since Unix epoch (1970-01-01T00:00:00Z) as UTC
// This minimizes constraints while allowing all component extraction
struct XsdDateTime {
    /// Microseconds since Unix epoch (UTC), as a SIGNED i64 stored in its
    /// two's-complement 64-bit pattern (encode `(e as u64) as Field`, decode
    /// `f as i64`); pre-1970 instants are negative (sq-3x7dl.7)
    /// Range: supports dates from ~292,000 BCE to ~292,000 CE with microsecond precision
    epoch_microseconds: Field,
}

// Duration representation (for timezone offsets and intervals)
struct XsdDayTimeDuration {
    /// Total microseconds (signed via separate flag)
    microseconds: Field,
    /// Whether duration is negative
    negative: bool,
}
```

### Design Rationale: Single-Field DateTime

Noir is a proving language where constraint count directly impacts performance. Using a single `Field` for DateTime:

1. **Minimal storage**: One field element vs. 8+ fields for component struct
2. **Efficient comparison**: Single field comparison vs. cascading component comparisons
3. **Derived properties**: Year, month, day, etc. are computed on-demand from epoch
4. **Timezone handling**: All storage is UTC; timezone conversion at input/output boundaries

The tradeoff is that component extraction requires division/modulo operations, but these are computed once when needed rather than stored redundantly.

> **🔮 Future Work**: `XsdDecimal` for fixed-point decimal arithmetic

## Module Organization

### 1. Numeric Module (`numeric.nr`)

Implements numeric operations required by SPARQL operator mapping:

```noir
// Unary operators
fn numeric_unary_plus<T>(a: T) -> T
fn numeric_unary_minus<T>(a: T) -> T

// Binary arithmetic (using ieee754 for floats)
fn numeric_add<T>(a: T, b: T) -> T
fn numeric_subtract<T>(a: T, b: T) -> T
fn numeric_multiply<T>(a: T, b: T) -> T
fn numeric_divide<T>(a: T, b: T) -> T

// Comparison
fn numeric_equal<T>(a: T, b: T) -> bool
fn numeric_less_than<T>(a: T, b: T) -> bool
fn numeric_greater_than<T>(a: T, b: T) -> bool

// Functions
fn abs<T>(value: T) -> T
fn round<T>(value: T) -> T
fn ceil<T>(value: T) -> T
fn floor<T>(value: T) -> T
fn rand() -> u64  // Returns bits of xsd:double in [0, 1)
```

### 2. String Module (`string.nr`) — ✅ Implemented

Strings are represented as `([u8; N], u32)` byte-array tuples (buffer + logical
length) rather than Noir's `str<N>`, because Noir cannot reconstruct a `str` from
bytes at runtime. Functions that produce new strings return this tuple; predicate
functions return `bool`/numeric values.

Implemented (see [SPARQL_COVERAGE.md](./SPARQL_COVERAGE.md) for the authoritative
per-function status):
- `fn:string-length`, `fn:substring`, `fn:concat`
- `fn:upper-case`, `fn:lower-case`
- `fn:starts-with`, `fn:ends-with`, `fn:contains`
- `fn:substring-before`, `fn:substring-after`
- `fn:normalize-space`, `fn:translate`, `fn:encode-for-uri`

> **Caveat**: `fn:substring` windows BYTE positions in the logical content — exact
> parity with the F&O spec only for ASCII. A codepoint-positional variant for
> multi-byte UTF-8 is tracked in sq-hjvte. A bounded, circuit-friendly `fn:matches`
> / `fn:replace` subset lives in `regex.nr` (sq-y73); full XPath regex is deferred.

### 3. Boolean Module (`boolean.nr`)

```noir
// Logical operators
fn fn_not(a: bool) -> bool
fn logical_and(a: bool, b: bool) -> bool
fn logical_or(a: bool, b: bool) -> bool

// Comparison
fn boolean_equal(a: bool, b: bool) -> bool
fn boolean_less_than(a: bool, b: bool) -> bool    // false < true
fn boolean_greater_than(a: bool, b: bool) -> bool
```

### 4. DateTime Module (`datetime.nr`)

All DateTime values stored as UTC microseconds since epoch in a single `Field`.
Component extraction computes values on-demand via division/modulo.

```noir
// Constants for time calculations
global MICROSECONDS_PER_SECOND: Field = 1_000_000;
global MICROSECONDS_PER_MINUTE: Field = 60_000_000;
global MICROSECONDS_PER_HOUR: Field = 3_600_000_000;
global MICROSECONDS_PER_DAY: Field = 86_400_000_000;

// Construction
fn datetime_from_components(
    year: i32, month: u8, day: u8,
    hour: u8, minute: u8, second: u8, microsecond: u32
) -> XsdDateTime

fn datetime_from_epoch_microseconds(micros: Field) -> XsdDateTime

// Component extraction (computed from epoch_microseconds)
fn year_from_datetime(dt: XsdDateTime) -> i32
fn month_from_datetime(dt: XsdDateTime) -> u8
fn day_from_datetime(dt: XsdDateTime) -> u8
fn hours_from_datetime(dt: XsdDateTime) -> u8
fn minutes_from_datetime(dt: XsdDateTime) -> u8
fn seconds_from_datetime(dt: XsdDateTime) -> u8  // Integer seconds
fn microseconds_from_datetime(dt: XsdDateTime) -> u32

// Comparison (efficient single-field comparisons)
fn datetime_equal(a: XsdDateTime, b: XsdDateTime) -> bool {
    a.epoch_microseconds == b.epoch_microseconds
}

fn datetime_less_than(a: XsdDateTime, b: XsdDateTime) -> bool {
    a.epoch_microseconds as u64 < b.epoch_microseconds as u64
}

fn datetime_greater_than(a: XsdDateTime, b: XsdDateTime) -> bool {
    a.epoch_microseconds as u64 > b.epoch_microseconds as u64
}
```

> **Note**: The XPath `fn:seconds-from-dateTime` returns a decimal including fractional seconds. Since decimals are deferred, we provide integer seconds + separate microseconds accessor.

### 5. Hash Module (`hash.nr`) — partial

`hash.nr` provides a circuit-native content hash (`string_pedersen_hash`) plus a
`bytes_to_lower_hex` hex formatter (sq-y73). The SPARQL-named cryptographic digest
functions (`MD5`, `SHA1`, `SHA256`, `SHA384`, `SHA512`) with their canonical
hex-string output are **deferred** — see SPARQL_COVERAGE.md. When added they will
leverage Noir's stdlib hash primitives over the byte-array string representation.

### 6. Comparison Module (`comparison.nr`)

Generic comparison utilities:

```noir
// Value comparison for numeric types
fn value_equal<T>(a: T, b: T) -> bool where T: Eq
fn value_less_than<T>(a: T, b: T) -> bool where T: Ord
fn value_greater_than<T>(a: T, b: T) -> bool where T: Ord
```

> **🔮 Future**: String collation comparisons (depends on string module)

## Test Infrastructure

### Test Generation from qt3tests

The `scripts/generate_tests.py` script:

1. **Clones/updates** the [w3c/qt3tests](https://github.com/w3c/qt3tests) repository
2. **Parses** XML test files from relevant test sets (fn/, op/, etc.)
3. **Filters** tests to those applicable to SPARQL (XPath 2.0 subset)
4. **Generates** Noir test code with:
   - Input values parsed from test expressions
   - Expected outputs from result assertions
   - Test functions grouped by function/operator
5. **Organizes** tests into chunked packages to enable parallel compilation

### Test File Format (qt3tests)

Test files use the namespace `http://www.w3.org/2010/09/qt-fots-catalog`:

```xml
<test-set name="fn-abs">
  <description>Tests for the fn:abs function</description>
  <test-case name="fn-abs-1">
    <description>Test abs with positive integer</description>
    <test>abs(10)</test>
    <result>
      <assert-eq>10</assert-eq>
    </result>
  </test-case>
  <test-case name="fn-abs-2">
    <test>abs(-10)</test>
    <result>
      <assert-eq>10</assert-eq>
    </result>
  </test-case>
</test-set>
```

### Generated Noir Test Example

```noir
// Auto-generated from qt3tests fn-abs
// Test suite source: https://github.com/w3c/qt3tests

use xpath::{abs};

#[test]
fn test_fn_abs_1() {
    // abs(10) = 10
    let result = abs(10);
    assert_eq(result, 10);
}

#[test]
fn test_fn_abs_2() {
    // abs(-10) = 10
    let result = abs(-10);
    assert_eq(result, 10);
}
```

## Implementation Notes

### UTF-8 String Handling

Since Noir operates on bytes, UTF-8 strings require careful handling:

1. **Character counting**: `string_length` must count Unicode codepoints, not bytes
2. **Substring**: Must respect codepoint boundaries
3. **Case conversion**: Only ASCII case conversion is straightforward; full Unicode case folding is complex

Initial implementation may support ASCII-only for simplicity, with Unicode support added incrementally.

### Floating-Point Operations

All float operations delegate to `noir_IEEE754`:

```noir
use ieee754::{float64_from_bits, float64_to_bits, add_float64, ...};

fn numeric_add_double(a: u64, b: u64) -> u64 {
    let fa = float64_from_bits(a);
    let fb = float64_from_bits(b);
    float64_to_bits(add_float64(fa, fb))
}
```

### DateTime Epoch Calculations

Component extraction from epoch microseconds uses calendar arithmetic:

```noir
// Simplified algorithm for year/month/day extraction
// Based on Howard Hinnant's date algorithms
fn days_from_epoch(epoch_micros: Field) -> i64 {
    (epoch_micros / MICROSECONDS_PER_DAY) as i64
}

fn year_from_days(days: i64) -> i32 {
    // Civil calendar algorithm
    // Accounts for leap years in 400-year cycles
}
```

### Error Handling

XPath functions can raise errors (e.g., division by zero). In Noir:

1. **Assertions**: For clearly invalid inputs
2. **Option types**: For functions that may not return a value
3. **Result types**: For operations that may fail

```noir
// Division may fail on zero divisor
fn numeric_divide_safe(a: i128, b: i128) -> Option<i128> {
    if b == 0 {
        Option::none()
    } else {
        Option::some(a / b)
    }
}
```

## References

- [SPARQL 1.1 Query Language](https://www.w3.org/TR/sparql11-query/)
- [XQuery 1.0 and XPath 2.0 Functions and Operators](https://www.w3.org/TR/xpath-functions/)
- [W3C QT3 Test Suite](https://github.com/w3c/qt3tests)
- [noir_IEEE754](https://github.com/jeswr/noir_IEEE754)
- [Noir Language Documentation](https://noir-lang.org/docs)
