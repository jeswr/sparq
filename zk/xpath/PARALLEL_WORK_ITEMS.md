# Parallel Work Items for noir_XPath

This document describes independent work streams that can be executed in parallel to expand the noir_XPath library's functionality. Each work stream is self-contained and can be assigned to different contributors.

## Overview

| Stream | Description | Complexity | New Code? | Dependencies |
|--------|-------------|------------|-----------|--------------|
| **A** | Duration Function Tests | Low | No | None |
| **B** | DateTime-Duration Arithmetic Tests | Low | No | None |
| **C** | Date Type & Functions | Medium | Yes | None |
| **D** | Time Type & Functions | Medium | Yes | None |
| **E** | Numeric Unary Operator Tests | Low | No | None |
| **F** | Boolean Comparison Tests | Low | No | None |
| **G** | Float/Double Rounding | Medium | Yes | IEEE754 lib |
| **H** | Timezone Adjustment Functions | Medium | Yes | None |

---

## Stream A: Duration Function Test Generation

**Status:** Code exists, needs qt3tests integration  
**Estimated Effort:** 2-4 hours  
**Parallelizable:** Yes (independent)

### Objective
Add qt3tests coverage for duration component extraction functions that are already implemented in `xpath/src/duration.nr`.

### Tasks

1. **Update `scripts/generate_tests.py`**
   - Add to `FUNCTION_TEST_FILES`:
     ```python
     "fn:days-from-duration": "fn/days-from-duration.xml",
     "fn:hours-from-duration": "fn/hours-from-duration.xml",
     "fn:minutes-from-duration": "fn/minutes-from-duration.xml",
     "fn:seconds-from-duration": "fn/seconds-from-duration.xml",
     ```
   - Add to `FUNCTION_MAP`:
     ```python
     "fn:days-from-duration": "days_from_duration",
     "fn:hours-from-duration": "hours_from_duration",
     "fn:minutes-from-duration": "minutes_from_duration",
     "fn:seconds-from-duration": "seconds_from_duration",
     ```

2. **Add Duration Parsing Support**
   - Create `parse_daytime_duration()` function similar to `parse_datetime()`
   - Handle `xs:dayTimeDuration("P3DT10H30M15S")` format
   - Convert to microseconds for Noir

3. **Add Duration Expression Conversion**
   - Update `convert_xpath_expr()` to handle duration extraction functions
   - Pattern: `fn:days-from-duration(xs:dayTimeDuration("..."))` → `days_from_duration(dur)`

4. **Generate Test Packages**
   ```bash
   python generate_tests.py --functions "fn:days-from-duration,fn:hours-from-duration,fn:minutes-from-duration,fn:seconds-from-duration"
   ```

5. **Update Workspace**
   - Add new test packages to `Nargo.toml`
   - Verify tests pass with `nargo test`

### Files to Modify
- `scripts/generate_tests.py`
- `Nargo.toml` (add workspace members)

### Files to Create
- `test_packages/xpath_test_fndays_from_duration/`
- `test_packages/xpath_test_fnhours_from_duration/`
- `test_packages/xpath_test_fnminutes_from_duration/`
- `test_packages/xpath_test_fnseconds_from_duration/`

### Acceptance Criteria
- [ ] All 4 test packages generated
- [ ] `nargo test --package xpath_test_fndays_from_duration` passes
- [ ] `nargo test --package xpath_test_fnhours_from_duration` passes
- [ ] `nargo test --package xpath_test_fnminutes_from_duration` passes
- [ ] `nargo test --package xpath_test_fnseconds_from_duration` passes

---

## Stream B: DateTime-Duration Arithmetic Test Generation

**Status:** Code exists, needs qt3tests integration  
**Estimated Effort:** 4-6 hours  
**Parallelizable:** Yes (independent)

### Objective
Add qt3tests coverage for DateTime-Duration arithmetic operations already implemented.

### Tasks

1. **Update `scripts/generate_tests.py`**
   - Add to `FUNCTION_TEST_FILES`:
     ```python
     "op:add-dayTimeDuration-to-dateTime": "op/add-dayTimeDuration-to-dateTime.xml",
     "op:subtract-dayTimeDuration-from-dateTime": "op/subtract-dayTimeDuration-from-dateTime.xml",
     "op:subtract-dateTimes": "op/subtract-dateTimes.xml",
     "op:add-dayTimeDurations": "op/add-dayTimeDurations.xml",
     "op:subtract-dayTimeDurations": "op/subtract-dayTimeDurations.xml",
     "op:dayTimeDuration-equal": "op/dayTimeDuration-equal.xml",
     "op:dayTimeDuration-less-than": "op/dayTimeDuration-less-than.xml",
     "op:dayTimeDuration-greater-than": "op/dayTimeDuration-greater-than.xml",
     ```
   - Add corresponding entries to `FUNCTION_MAP`

2. **Add Duration Comparison Expression Handling**
   - Handle `xs:dayTimeDuration("...") eq xs:dayTimeDuration("...")`
   - Handle `xs:dayTimeDuration("...") lt xs:dayTimeDuration("...")`
   - Handle `xs:dayTimeDuration("...") gt xs:dayTimeDuration("...")`

3. **Add DateTime-Duration Arithmetic Expression Handling**
   - Handle `xs:dateTime("...") + xs:dayTimeDuration("...")`
   - Handle `xs:dateTime("...") - xs:dayTimeDuration("...")`
   - Handle `xs:dateTime("...") - xs:dateTime("...")`
   - Handle `xs:dayTimeDuration("...") + xs:dayTimeDuration("...")`

4. **Handle String Output Assertions**
   - Many duration tests use `assert-string-value` (e.g., "P15DT11H59M59S")
   - Option A: Parse expected duration strings and compare microseconds
   - Option B: Skip string-value assertions, only test `assert-true`/`assert-false`

5. **Generate Test Packages**

### Files to Modify
- `scripts/generate_tests.py`
- `Nargo.toml`

### Files to Create
- `test_packages/xpath_test_opdaytime_duration_equal/`
- `test_packages/xpath_test_opdaytime_duration_less_than/`
- `test_packages/xpath_test_opdaytime_duration_greater_than/`
- `test_packages/xpath_test_opadd_daytime_duration_to_datetime/`
- `test_packages/xpath_test_opsubtract_daytime_duration_from_datetime/`
- `test_packages/xpath_test_opsubtract_datetimes/`
- `test_packages/xpath_test_opadd_daytime_durations/`
- `test_packages/xpath_test_opsubtract_daytime_durations/`

### Acceptance Criteria
- [ ] Duration comparison test packages generated and passing
- [ ] DateTime-duration arithmetic test packages generated and passing
- [ ] Duration arithmetic test packages generated and passing

---

## Stream C: Date Type & Functions

**Status:** New code required  
**Estimated Effort:** 6-8 hours  
**Parallelizable:** Yes (independent of Stream D)

### Objective
Implement `xs:date` type and related functions, mirroring the existing `XsdDateTime` implementation.

### Tasks

1. **Add XsdDate Type** (`xpath/src/types.nr`)
   ```noir
   struct XsdDate {
       epoch_days: Field,      // Days since Unix epoch
       tz_offset_minutes: i16, // Timezone offset
   }
   ```

2. **Create Date Module** (`xpath/src/date.nr`)
   - `date_from_components(year, month, day, tz_offset) -> XsdDate`
   - `date_from_epoch_days(days, tz_offset) -> XsdDate`
   - `year_from_date(date) -> i32`
   - `month_from_date(date) -> u8`
   - `day_from_date(date) -> u8`
   - `timezone_from_date(date) -> XsdDayTimeDuration`
   - `date_equal(d1, d2) -> bool`
   - `date_less_than(d1, d2) -> bool`
   - `date_greater_than(d1, d2) -> bool`

3. **Update Library Exports** (`xpath/src/lib.nr`)
   - Add `mod date;`
   - Export all public functions

4. **Add Unit Tests** (`xpath_unit_tests/src/date_tests.nr`)
   - Test date construction
   - Test component extraction
   - Test comparisons with different timezones

5. **Update Test Generator**
   - Add date parsing support
   - Add mappings for date functions
   - Generate test packages

### Files to Create
- `xpath/src/date.nr`
- `xpath_unit_tests/src/date_tests.nr`
- `test_packages/xpath_test_fnyear_from_date/`
- `test_packages/xpath_test_fnmonth_from_date/`
- `test_packages/xpath_test_fnday_from_date/`
- `test_packages/xpath_test_opdate_equal/`
- `test_packages/xpath_test_opdate_less_than/`
- `test_packages/xpath_test_opdate_greater_than/`

### Files to Modify
- `xpath/src/types.nr`
- `xpath/src/lib.nr`
- `xpath_unit_tests/src/lib.nr`
- `scripts/generate_tests.py`
- `Nargo.toml`

### Acceptance Criteria
- [ ] `XsdDate` type implemented
- [ ] All date component extraction functions working
- [ ] Date comparison functions working
- [ ] Unit tests passing
- [ ] qt3tests packages generated and passing

---

## Stream D: Time Type & Functions

**Status:** New code required  
**Estimated Effort:** 4-6 hours  
**Parallelizable:** Yes (independent of Stream C)

### Objective
Implement `xs:time` type and related functions.

### Tasks

1. **Add XsdTime Type** (`xpath/src/types.nr`)
   ```noir
   struct XsdTime {
       microseconds_of_day: u64, // Microseconds since midnight (0-86399999999)
       tz_offset_minutes: i16,   // Timezone offset
   }
   ```

2. **Create Time Module** (`xpath/src/time.nr`)
   - `time_from_components(hour, minute, second, microsecond, tz_offset) -> XsdTime`
   - `hours_from_time(time) -> u8`
   - `minutes_from_time(time) -> u8`
   - `seconds_from_time(time) -> u8`
   - `timezone_from_time(time) -> XsdDayTimeDuration`
   - `time_equal(t1, t2) -> bool`
   - `time_less_than(t1, t2) -> bool`
   - `time_greater_than(t1, t2) -> bool`

3. **Update Library Exports**

4. **Add Unit Tests**

5. **Update Test Generator**

### Files to Create
- `xpath/src/time.nr`
- `xpath_unit_tests/src/time_tests.nr`
- `test_packages/xpath_test_fnhours_from_time/`
- `test_packages/xpath_test_fnminutes_from_time/`
- `test_packages/xpath_test_fnseconds_from_time/`
- `test_packages/xpath_test_optime_equal/`
- `test_packages/xpath_test_optime_less_than/`
- `test_packages/xpath_test_optime_greater_than/`

### Files to Modify
- `xpath/src/types.nr`
- `xpath/src/lib.nr`
- `xpath_unit_tests/src/lib.nr`
- `scripts/generate_tests.py`
- `Nargo.toml`

### Acceptance Criteria
- [ ] `XsdTime` type implemented
- [ ] All time component extraction functions working
- [ ] Time comparison functions working
- [ ] Unit tests passing
- [ ] qt3tests packages generated and passing

---

## Stream E: Numeric Unary Operator Tests

**Status:** Code exists, needs qt3tests integration  
**Estimated Effort:** 1-2 hours  
**Parallelizable:** Yes (independent)

### Objective
Add qt3tests coverage for unary numeric operators.

### Tasks

1. **Update `scripts/generate_tests.py`**
   - Add to `FUNCTION_TEST_FILES`:
     ```python
     "op:numeric-unary-plus": "op/numeric-unary-plus.xml",
     "op:numeric-unary-minus": "op/numeric-unary-minus.xml",
     ```
   - Add to `FUNCTION_MAP`:
     ```python
     "op:numeric-unary-plus": "numeric_unary_plus_int",
     "op:numeric-unary-minus": "numeric_unary_minus_int",
     ```

2. **Add Unary Expression Handling**
   - Handle `+$value` → `numeric_unary_plus_int(value)`
   - Handle `-$value` → `numeric_unary_minus_int(value)`

3. **Generate Test Packages**

### Files to Modify
- `scripts/generate_tests.py`
- `Nargo.toml`

### Files to Create
- `test_packages/xpath_test_opnumeric_unary_plus/`
- `test_packages/xpath_test_opnumeric_unary_minus/`

### Acceptance Criteria
- [ ] Both test packages generated
- [ ] All tests passing

---

## Stream F: Boolean Comparison Tests

**Status:** Code exists, needs qt3tests integration  
**Estimated Effort:** 1-2 hours  
**Parallelizable:** Yes (independent)

### Objective
Add qt3tests coverage for boolean comparison operators.

### Tasks

1. **Update `scripts/generate_tests.py`**
   - Add to `FUNCTION_TEST_FILES`:
     ```python
     "op:boolean-less-than": "op/boolean-less-than.xml",
     "op:boolean-greater-than": "op/boolean-greater-than.xml",
     ```
   - Add to `FUNCTION_MAP`:
     ```python
     "op:boolean-less-than": "boolean_less_than",
     "op:boolean-greater-than": "boolean_greater_than",
     ```

2. **Add Boolean Comparison Expression Handling**
   - Handle `true() lt false()` patterns
   - Handle `xs:boolean("true") gt xs:boolean("false")` patterns

3. **Generate Test Packages**

### Files to Modify
- `scripts/generate_tests.py`
- `Nargo.toml`

### Files to Create
- `test_packages/xpath_test_opboolean_less_than/`
- `test_packages/xpath_test_opboolean_greater_than/`

### Acceptance Criteria
- [ ] Both test packages generated
- [ ] All tests passing

---

## Stream G: Float/Double Rounding Functions

**Status:** New code required (depends on IEEE754 capabilities)  
**Estimated Effort:** 8-12 hours  
**Parallelizable:** Yes, but has external dependency

### Objective
Implement rounding functions for float and double types using the `noir_IEEE754` library.

### Prerequisites
- Verify `noir_IEEE754` library supports or can support rounding operations
- If not available, may need to implement using bit manipulation

### Tasks

1. **Research IEEE754 Library Capabilities**
   - Check if `round`, `ceil`, `floor` exist
   - If not, determine feasibility of implementation

2. **Add Float Rounding Functions** (`xpath/src/numeric_types.nr`)
   ```noir
   pub fn round_float(x: XsdFloat) -> XsdFloat
   pub fn ceil_float(x: XsdFloat) -> XsdFloat
   pub fn floor_float(x: XsdFloat) -> XsdFloat
   pub fn round_half_to_even_float(x: XsdFloat, precision: i32) -> XsdFloat
   ```

3. **Add Double Rounding Functions**
   ```noir
   pub fn round_double(x: XsdDouble) -> XsdDouble
   pub fn ceil_double(x: XsdDouble) -> XsdDouble
   pub fn floor_double(x: XsdDouble) -> XsdDouble
   pub fn round_half_to_even_double(x: XsdDouble, precision: i32) -> XsdDouble
   ```

4. **Update Library Exports**

5. **Add Unit Tests**

6. **Update Test Generator for Float Tests**
   - Enable float/double test generation for rounding functions
   - Handle special values (NaN, Infinity)

### Files to Modify
- `xpath/src/numeric_types.nr`
- `xpath/src/lib.nr`
- `scripts/generate_tests.py`

### Files to Create
- `test_packages/xpath_test_fnround_float/`
- `test_packages/xpath_test_fnround_double/`
- `test_packages/xpath_test_fnceiling_float/`
- `test_packages/xpath_test_fnceiling_double/`
- `test_packages/xpath_test_fnfloor_float/`
- `test_packages/xpath_test_fnfloor_double/`

### Acceptance Criteria
- [ ] Float rounding functions implemented
- [ ] Double rounding functions implemented
- [ ] Special value handling (NaN, Infinity) correct
- [ ] Unit tests passing
- [ ] qt3tests packages generated and passing

---

## Stream H: Timezone Adjustment Functions

**Status:** New code required  
**Estimated Effort:** 6-8 hours  
**Parallelizable:** Yes (after Streams C and D complete for full coverage)

### Objective
Implement timezone adjustment functions for dateTime, date, and time types.

### Tasks

1. **Implement adjust-dateTime-to-timezone** (`xpath/src/datetime.nr`)
   ```noir
   /// Adjusts a dateTime to a new timezone
   /// If new_tz is None, removes timezone (returns localized time)
   pub fn adjust_datetime_to_timezone(
       dt: XsdDateTime, 
       new_tz_minutes: Option<i16>
   ) -> XsdDateTime
   ```

2. **Implement adjust-date-to-timezone** (`xpath/src/date.nr`)
   - Similar logic for date type
   - Handle date boundary crossings when adjusting

3. **Implement adjust-time-to-timezone** (`xpath/src/time.nr`)
   - Similar logic for time type
   - Handle day wraparound

4. **Update Library Exports**

5. **Add Unit Tests**
   - Test timezone conversion accuracy
   - Test boundary conditions (midnight crossings, etc.)
   - Test removal of timezone

6. **Update Test Generator**

### Files to Modify
- `xpath/src/datetime.nr`
- `xpath/src/date.nr` (from Stream C)
- `xpath/src/time.nr` (from Stream D)
- `xpath/src/lib.nr`
- `scripts/generate_tests.py`

### Files to Create
- `test_packages/xpath_test_fnadjust_datetime_to_timezone/`
- `test_packages/xpath_test_fnadjust_date_to_timezone/`
- `test_packages/xpath_test_fnadjust_time_to_timezone/`

### Acceptance Criteria
- [ ] All three adjustment functions implemented
- [ ] Timezone conversion mathematically correct
- [ ] Boundary conditions handled
- [ ] Unit tests passing
- [ ] qt3tests packages generated and passing

---

## Dependency Graph

```
     ┌─────┐     ┌─────┐     ┌─────┐     ┌─────┐
     │  A  │     │  B  │     │  E  │     │  F  │
     │Dur. │     │DT+D │     │Unary│     │Bool │
     │Tests│     │Arith│     │ Op  │     │Comp │
     └─────┘     └─────┘     └─────┘     └─────┘
        │           │           │           │
        └───────────┴───────────┴───────────┘
                        │
                  All Independent
                        │
     ┌─────┐     ┌─────┐           ┌─────┐
     │  C  │     │  D  │           │  G  │
     │Date │     │Time │           │Float│
     │Type │     │Type │           │Round│
     └──┬──┘     └──┬──┘           └──┬──┘
        │           │                 │
        └─────┬─────┘                 │
              │                       │
              ▼                       │
         ┌─────┐                      │
         │  H  │                      │
         │ TZ  │◄─────────────────────┘
         │Adj. │   (Optional: can test
         └─────┘    with just DateTime)
```

---

## Recommended Execution Order

### Phase 1: Quick Wins (Can start immediately, in parallel)
- **Stream A**: Duration Function Tests
- **Stream E**: Numeric Unary Operator Tests  
- **Stream F**: Boolean Comparison Tests

*Estimated time: 1-2 days with 1 contributor, or same day with 3 contributors*

### Phase 2: Duration Arithmetic (Can start immediately)
- **Stream B**: DateTime-Duration Arithmetic Tests

*Estimated time: 1-2 days*

### Phase 3: New Types (Can start immediately, in parallel)
- **Stream C**: Date Type & Functions
- **Stream D**: Time Type & Functions

*Estimated time: 2-3 days with 1 contributor, or 2 days with 2 contributors*

### Phase 4: Advanced Features (After Phase 3)
- **Stream G**: Float/Double Rounding (can start earlier if IEEE754 research done)
- **Stream H**: Timezone Adjustment Functions

*Estimated time: 3-4 days*

---

## Contributor Guidelines

### Before Starting
1. Claim your stream in the issue tracker
2. Create a feature branch: `feature/stream-X-description`
3. Review existing code patterns in similar modules

### During Development
1. Follow existing code style (run `nargo fmt`)
2. Add inline documentation for all public functions
3. Write unit tests before/alongside implementation
4. Test locally with `nargo test --package <package_name>`

### Completing Work
1. Run full test suite: `nargo test`
2. Update `TESTING.md` with new test coverage
3. Update `SPARQL_COVERAGE.md` if applicable
4. Create PR with description of changes
5. Request review from maintainer

---

## Success Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Functions with qt3tests | ~25 | ~45 |
| XPath types supported | 3 (DateTime, Duration, numeric) | 5 (+Date, Time) |
| Duration functions tested | 0 | 4 |
| DateTime-Duration ops tested | 0 | 8 |
| Date functions tested | 0 | 6 |
| Time functions tested | 0 | 6 |

---

## Questions / Blockers

Document any blockers or questions that arise during implementation:

1. **IEEE754 Rounding**: Does the `noir_IEEE754` library support rounding operations, or do we need to implement them?

2. **Duration String Parsing**: Many qt3tests expect string output (e.g., "P15DT11H59M59S"). Should we:
   - Skip these tests?
   - Parse expected strings and compare numerically?
   - Implement duration-to-string conversion?

3. **Negative Durations**: How should negative durations be represented in test assertions?

4. **Date Boundary Handling**: When adjusting timezone for dates near midnight, how do we handle day changes?
