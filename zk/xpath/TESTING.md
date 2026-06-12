# Testing Guide

This document describes the testing strategy and how to run tests for noir_XPath.

## Test Structure

The project uses a multi-layered testing approach:

### 1. Inline Module Tests
Basic unit tests embedded directly in each module:
- `xpath/src/boolean.nr` - Boolean operation tests
- `xpath/src/numeric.nr` - Numeric operation tests
- `xpath/src/datetime.nr` - DateTime component extraction tests
- `xpath/src/duration.nr` - Duration operation tests
- `xpath/src/sequence.nr` - Sequence/aggregate function tests
- `xpath/src/comparison.nr` - Comparison utility tests

### 2. Unit Test Package
Additional comprehensive tests in `xpath_unit_tests/`:
- `datetime_tests.nr` - Extended datetime tests including timezone
- `boolean_tests.nr` - Extended boolean tests
- `numeric_tests.nr` - Extended numeric tests

### 3. Generated qt3tests Packages
Auto-generated test packages from the W3C qt3tests suite in `test_packages/`:
- `xpath_test_fnabs/` - Tests for fn:abs from qt3tests
- `xpath_test_fnday_from_datetime/` - Tests for fn:day-from-dateTime
- `xpath_test_fnhours_from_datetime/` - Tests for fn:hours-from-dateTime
- `xpath_test_fnminutes_from_datetime/` - Tests for fn:minutes-from-dateTime
- `xpath_test_fnmonth_from_datetime/` - Tests for fn:month-from-dateTime
- `xpath_test_fnnot/` - Tests for fn:not
- `xpath_test_fnseconds_from_datetime/` - Tests for fn:seconds-from-dateTime
- `xpath_test_fnyear_from_datetime/` - Tests for fn:year-from-dateTime
- `xpath_test_opdatetime_equal/` - Tests for op:dateTime-equal
- `xpath_test_opdatetime_greater_than/` - Tests for op:dateTime-greater-than
- `xpath_test_opdatetime_less_than/` - Tests for op:dateTime-less-than
- `xpath_test_opnumeric_add/` - Tests for op:numeric-add (integer)
- `xpath_test_opnumeric_add_float/` - Tests for op:numeric-add (float)
- `xpath_test_opnumeric_add_double/` - Tests for op:numeric-add (double)
- `xpath_test_opnumeric_divide/` - Tests for op:numeric-divide (integer)
- `xpath_test_opnumeric_divide_float/` - Tests for op:numeric-divide (float)
- `xpath_test_opnumeric_divide_double/` - Tests for op:numeric-divide (double)
- `xpath_test_opnumeric_integer_divide/` - Tests for op:numeric-integer-divide
- `xpath_test_opnumeric_mod/` - Tests for op:numeric-mod
- `xpath_test_opnumeric_multiply/` - Tests for op:numeric-multiply (integer)
- `xpath_test_opnumeric_multiply_float/` - Tests for op:numeric-multiply (float)
- `xpath_test_opnumeric_multiply_double/` - Tests for op:numeric-multiply (double)
- `xpath_test_opnumeric_subtract/` - Tests for op:numeric-subtract (integer)
- `xpath_test_opnumeric_subtract_float/` - Tests for op:numeric-subtract (float)
- `xpath_test_opnumeric_subtract_double/` - Tests for op:numeric-subtract (double)

### 4. Type Casting Test Packages
Manual test packages for XPath type casting functions:
- `xpath_test_xsfloat_from_int/` - Tests for xs:float($arg as xs:integer)
- `xpath_test_xsdouble_from_int/` - Tests for xs:double($arg as xs:integer)
- `xpath_test_xsinteger_from_float/` - Tests for xs:integer($arg as xs:float)
- `xpath_test_xsinteger_from_double/` - Tests for xs:integer($arg as xs:double)
- `xpath_test_xsfloat_from_double/` - Tests for xs:float($arg as xs:double)

## Running Tests

### Run All Tests
```bash
nargo test
```

This runs tests across all workspace members.

### Run Tests for Main Library Only
```bash
nargo test --package xpath
```

### Run Unit Tests Only
```bash
nargo test --package xpath_unit_tests
```

### Run Specific Generated Test Package
```bash
nargo test --package xpath_test_fnabs
```

## Test Coverage by Function

### ✅ Fully Tested (with qt3tests)

**Numeric Functions (Integer):**
- fn:abs - `xpath_test_fnabs/`
- op:numeric-add - `xpath_test_opnumeric_add/`
- op:numeric-subtract - `xpath_test_opnumeric_subtract/`
- op:numeric-multiply - `xpath_test_opnumeric_multiply/`
- op:numeric-divide - `xpath_test_opnumeric_divide/`
- op:numeric-integer-divide - `xpath_test_opnumeric_integer_divide/`
- op:numeric-mod - `xpath_test_opnumeric_mod/`

**DateTime Functions:**
- fn:year-from-dateTime - `xpath_test_fnyear_from_datetime/`
- fn:month-from-dateTime - `xpath_test_fnmonth_from_datetime/`
- fn:day-from-dateTime - `xpath_test_fnday_from_datetime/`
- fn:hours-from-dateTime - `xpath_test_fnhours_from_datetime/`
- fn:minutes-from-dateTime - `xpath_test_fnminutes_from_datetime/`
- fn:seconds-from-dateTime - `xpath_test_fnseconds_from_datetime/`
- op:dateTime-equal - `xpath_test_opdatetime_equal/`
- op:dateTime-less-than - `xpath_test_opdatetime_less_than/`
- op:dateTime-greater-than - `xpath_test_opdatetime_greater_than/`

**Boolean Functions:**
- fn:not - `xpath_test_fnnot/`

### ✅ Tested (Unit Tests Only)

Functions with comprehensive unit tests but no qt3tests coverage yet:

**Type Casting:**
- xs:float (from integer) - `xpath_test_xsfloat_from_int/` (manual tests)
- xs:double (from integer) - `xpath_test_xsdouble_from_int/` (manual tests)
- xs:integer (from float) - `xpath_test_xsinteger_from_float/` (manual tests)
- xs:integer (from double) - `xpath_test_xsinteger_from_double/` (manual tests)
- xs:float (from double) - `xpath_test_xsfloat_from_double/` (manual tests)

**Numeric:**
- fn:round, fn:ceiling, fn:floor (identity for integers)
- op:numeric-unary-plus, op:numeric-unary-minus
- op:numeric-equal, op:numeric-less-than, op:numeric-greater-than
- min, max (pairwise and sequence)

**Boolean:**
- logical-and, logical-or
- op:boolean-equal, op:boolean-less-than, op:boolean-greater-than

**DateTime:**
- fn:timezone-from-dateTime
- datetime construction from components
- datetime construction from epoch
- microseconds extraction

**Duration:**
- All duration operations (construction, extraction, arithmetic, comparison)
- DateTime-duration arithmetic

**Sequence/Aggregates:**
- is_empty, exists, count
- sum_int, avg_int, min_int_seq, max_int_seq
- Partial array operations
- Boolean aggregates (all_true, any_true, count_true)

**Comparison:**
- Generic value_equal, value_less_than, value_greater_than

## Generating New Tests

To generate tests from qt3tests for a specific function:

```bash
cd scripts
python generate_tests.py --functions "fn:timezone-from-dateTime"
```

To regenerate all tests:

```bash
cd scripts
python generate_tests.py
```

See [scripts/README.md](./scripts/README.md) for detailed documentation on test generation.

## Test Limitations

### Integer-Only Coverage
Current qt3tests are generated only for integer values. Float and double test generation requires:
1. noir_IEEE754 dependency integration
2. Updated test generator to handle float literals
3. Float-specific test packages

### Skipped Test Cases
The test generator skips certain qt3tests cases:
- Tests with float/double values (until float support is added)
- Tests with string operations
- Tests with complex expressions (multiple function calls)
- Tests with external variable references
- Tests requiring schema validation
- Tests with dates before Unix epoch (negative timestamps)

### Manual Test Coverage
Some functionality requires manual testing:
- Edge cases not covered by qt3tests
- Noir-specific constraints (Field size limits, etc.)
- Integration scenarios
- Error handling (division by zero, invalid inputs)

## Adding New Tests

### For New Functions
1. Add implementation in appropriate module (numeric.nr, datetime.nr, etc.)
2. Export from lib.nr
3. Add inline tests in the module
4. Add comprehensive tests to xpath_unit_tests/
5. Map to qt3tests in scripts/generate_tests.py (if applicable)
6. Generate qt3tests package
7. Add package to workspace Nargo.toml

### For Edge Cases
Add tests to `xpath_unit_tests/src/` in the appropriate test file.

## CI/CD

Tests are run automatically on:
- Every push to main branch
- Every pull request
- Nightly builds

Test packages are designed to be independent and can run in parallel for faster CI execution.

## Test Quality Metrics

Target metrics for the library:
- ✅ All implemented XPath functions have qt3tests coverage
- ✅ All XPath operators have qt3tests coverage
- ✅ Edge cases covered in unit tests
- ✅ Timezone handling tested
- ✅ Negative values tested
- ✅ Boundary conditions tested

## Future Test Additions

When new features are implemented:

**Float Support (noir_IEEE754):**
- Generate float/double test packages from qt3tests
- Test NaN, Infinity, -Infinity handling
- Test float precision edge cases

**String Support:**
- Generate string function test packages
- Test UTF-8 encoding edge cases
- Test empty strings, very long strings

**Hash Functions:**
- Test against known hash vectors
- Test empty input
- Test long input
