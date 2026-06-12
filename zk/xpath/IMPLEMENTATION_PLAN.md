# noir_XPath Implementation Plan

Phased implementation approach for the noir_XPath library.

## Phase 0: Project Setup (Week 1)

### Tasks:
1. **Restructure Repository**
   - [x] Create ARCHITECTURE.md
   - [x] Create IMPLEMENTATION_PLAN.md
   - [ ] Convert to workspace structure
   - [ ] Create `xpath/` main package
   - [ ] Create `xpath_unit_tests/` package
   - [ ] Create `test_packages/` directory
   - [ ] Create `scripts/` directory

2. **Configure Dependencies**
   - [ ] Add ieee754 dependency to xpath package
   - [ ] Configure workspace members in root Nargo.toml

3. **Setup CI/CD**
   - [ ] Create GitHub Actions workflow for testing
   - [ ] Configure test chunking for parallel CI

### Deliverables:
- Working workspace structure
- Empty module files with function stubs
- CI pipeline running

---

## Phase 1: Core Types & Boolean Operations (Week 2)

### Module: `types.nr`

Define core data type structures with minimal complexity:

```noir
// DateTime representation - single Field for circuit efficiency
// Stores microseconds since Unix epoch (1970-01-01T00:00:00Z) as UTC
struct XsdDateTime {
    epoch_microseconds: Field,
}

// Duration representation (for intervals)
struct XsdDayTimeDuration {
    microseconds: Field,
    negative: bool,
}
```

Implement type constructors and validation.

> **🔮 Future**: `XsdDecimal` deferred due to complexity of fixed-point arithmetic in ZK circuits.

### Module: `boolean.nr`

| Function | XPath | Priority | Complexity |
|----------|-------|----------|------------|
| `fn_not` | `fn:not` | P0 | Low |
| `logical_and` | `op:and` | P0 | Low |
| `logical_or` | `op:or` | P0 | Low |
| `boolean_equal` | `op:boolean-equal` | P0 | Low |
| `boolean_less_than` | `op:boolean-less-than` | P1 | Low |
| `boolean_greater_than` | `op:boolean-greater-than` | P1 | Low |

All functions are straightforward with Noir's native bool type.

### Deliverables:
- Complete `types.nr` with all type definitions
- Complete `boolean.nr` with all functions
- Unit tests for all types and boolean functions

---

## Phase 2: Numeric Operations (Weeks 3-4)

### Module: `numeric.nr`

#### Integer Operations (Week 3)

| Function | XPath | Priority | Notes |
|----------|-------|----------|-------|
| `numeric_add_int` | `op:numeric-add` | P0 | Use i128 |
| `numeric_subtract_int` | `op:numeric-subtract` | P0 | |
| `numeric_multiply_int` | `op:numeric-multiply` | P0 | |
| `numeric_divide_int` | `op:numeric-integer-divide` | P0 | |
| `numeric_mod_int` | `op:numeric-mod` | P1 | |
| `numeric_unary_plus` | `op:numeric-unary-plus` | P0 | |
| `numeric_unary_minus` | `op:numeric-unary-minus` | P0 | |
| `numeric_equal_int` | `op:numeric-equal` | P0 | |
| `numeric_less_than_int` | `op:numeric-less-than` | P0 | |
| `numeric_greater_than_int` | `op:numeric-greater-than` | P0 | |
| `abs_int` | `fn:abs` | P0 | |
| `round_int` | `fn:round` | P0 | Identity for integers |
| `ceil_int` | `fn:ceiling` | P0 | Identity for integers |
| `floor_int` | `fn:floor` | P0 | Identity for integers |

#### Float Operations (Week 4)

Integrate with noir_IEEE754:

| Function | XPath | Priority | Notes |
|----------|-------|----------|-------|
| `numeric_add_float` | `op:numeric-add` | P0 | ieee754::add_float32 |
| `numeric_add_double` | `op:numeric-add` | P0 | ieee754::add_float64 |
| `numeric_subtract_float` | `op:numeric-subtract` | P0 | |
| `numeric_multiply_float` | `op:numeric-multiply` | P0 | |
| `numeric_divide_float` | `op:numeric-divide` | P0 | |
| `numeric_equal_float` | `op:numeric-equal` | P0 | |
| `numeric_less_than_float` | `op:numeric-less-than` | P0 | |
| `abs_float` | `fn:abs` | P0 | |
| `round_float` | `fn:round` | P1 | |
| `ceil_float` | `fn:ceiling` | P1 | |
| `floor_float` | `fn:floor` | P1 | |

> **🔮 Future**: Decimal operations deferred. Will require careful scale handling when implemented.

### Deliverables:
- Complete integer arithmetic
- Complete float arithmetic (via ieee754)
- Basic decimal arithmetic
- Comparison operators for all numeric types
- qt3tests integration for `fn-abs`, `op-numeric-*`

---

## Phase 3: String Functions Part 1 — 🔮 Future

> **Status**: Deferred. String operations in ZK circuits are complex due to:
> - Variable-length data handling
> - UTF-8 encoding complexity
> - High constraint counts for string manipulation
>
> Will be implemented in a future version after core numeric/datetime functionality is stable.

### Module: `string.nr` (Future)

#### Basic String Operations

| Function | SPARQL | XPath | Status |
|----------|--------|-------|--------|
| `string_length` | STRLEN | `fn:string-length` | 🔮 Future |
| `substring` | SUBSTR | `fn:substring` | 🔮 Future |
| `concat` | CONCAT | `fn:concat` | 🔮 Future |
| `upper_case` | UCASE | `fn:upper-case` | 🔮 Future |
| `lower_case` | LCASE | `fn:lower-case` | 🔮 Future |

#### String Matching

| Function | SPARQL | XPath | Status |
|----------|--------|-------|--------|
| `starts_with` | STRSTARTS | `fn:starts-with` | 🔮 Future |
| `ends_with` | STRENDS | `fn:ends-with` | 🔮 Future |
| `contains` | CONTAINS | `fn:contains` | 🔮 Future |
| `substring_before` | STRBEFORE | `fn:substring-before` | 🔮 Future |
| `substring_after` | STRAFTER | `fn:substring-after` | 🔮 Future |

---

## Phase 4: String Functions Part 2 - Regex — 🔮 Future

> **Status**: Deferred. Regex is particularly complex in ZK circuits.

| Function | SPARQL | XPath | Status |
|----------|--------|-------|--------|
| `matches` | REGEX | `fn:matches` | 🔮 Future |
| `replace` | REPLACE | `fn:replace` | 🔮 Future |
| `compare` | - | `fn:compare` | 🔮 Future |
| `encode_for_uri` | ENCODE_FOR_URI | `fn:encode-for-uri` | 🔮 Future |

---

## Phase 5: DateTime Functions (Weeks 9-10)

### Module: `datetime.nr`

#### Component Extraction (Week 9)

| Function | SPARQL | XPath | Priority |
|----------|--------|-------|----------|
| `year_from_datetime` | YEAR | `fn:year-from-dateTime` | P0 |
| `month_from_datetime` | MONTH | `fn:month-from-dateTime` | P0 |
| `day_from_datetime` | DAY | `fn:day-from-dateTime` | P0 |
| `hours_from_datetime` | HOURS | `fn:hours-from-dateTime` | P0 |
| `minutes_from_datetime` | MINUTES | `fn:minutes-from-dateTime` | P0 |
| `seconds_from_datetime` | SECONDS | `fn:seconds-from-dateTime` | P0 |
| `timezone_from_datetime` | TIMEZONE | `fn:timezone-from-dateTime` | P1 |

#### DateTime Comparison (Week 10)

| Function | XPath | Priority |
|----------|-------|----------|
| `datetime_equal` | `op:dateTime-equal` | P0 |
| `datetime_less_than` | `op:dateTime-less-than` | P0 |
| `datetime_greater_than` | `op:dateTime-greater-than` | P0 |

### Implementation Notes:

DateTime comparison must handle timezones:
1. Normalize to UTC for comparison
2. Handle "no timezone" values according to XPath spec

```noir
fn datetime_to_utc_seconds(dt: XsdDateTime) -> i64 {
    // Convert to seconds since epoch in UTC
    let base_seconds = /* calendar calculation */;
    base_seconds - (dt.tz_offset_minutes as i64 * 60)
}

fn datetime_less_than(a: XsdDateTime, b: XsdDateTime) -> bool {
    datetime_to_utc_seconds(a) < datetime_to_utc_seconds(b)
}
```

### Deliverables:
- Complete datetime component extraction
- DateTime comparison with timezone handling
- DateTime parsing from ISO 8601 strings
- qt3tests for datetime functions

---

## Phase 4: Hash Functions — 🔮 Future

> **Status**: Deferred. Hash functions depend on string handling for hex output formatting.

| Function | SPARQL | Status |
|----------|--------|--------|
| `md5` | MD5 | 🔮 Future |
| `sha1` | SHA1 | 🔮 Future |
| `sha256` | SHA256 | 🔮 Future |
| `sha384` | SHA384 | 🔮 Future |
| `sha512` | SHA512 | 🔮 Future |

Will leverage Noir's stdlib hash primitives when string support is available.

---

## Phase 5: Test Generation & Integration (Weeks 7-8)

### Test Generation Script

Create `scripts/generate_tests.py`:

```python
#!/usr/bin/env python3
"""
Generate Noir test code from W3C qt3tests test suite.
"""

import xml.etree.ElementTree as ET
from pathlib import Path
import os

QT3_NAMESPACE = "{http://www.w3.org/2010/09/qt-fots-catalog}"

# Map XPath functions to Noir implementations
FUNCTION_MAP = {
    "fn:abs": ("xpath::numeric::abs", "numeric"),
    "fn:string-length": ("xpath::string::string_length", "string"),
    # ... etc
}

# Functions to include (current scope)
SPARQL_FUNCTIONS = [
    # Numeric
    "fn:abs", "fn:round", "fn:ceiling", "fn:floor",
    "op:numeric-add", "op:numeric-subtract",
    "op:numeric-multiply", "op:numeric-divide",
    "op:numeric-equal", "op:numeric-less-than", "op:numeric-greater-than",
    # DateTime
    "fn:year-from-dateTime", "fn:month-from-dateTime", "fn:day-from-dateTime",
    "fn:hours-from-dateTime", "fn:minutes-from-dateTime", "fn:seconds-from-dateTime",
    "op:dateTime-equal", "op:dateTime-less-than", "op:dateTime-greater-than",
    # Boolean
    "fn:not", "op:boolean-equal",
]

def parse_test_set(xml_path: Path) -> list:
    """Parse a qt3tests test set XML file."""
    tree = ET.parse(xml_path)
    root = tree.getroot()
    
    tests = []
    for test_case in root.findall(f".//{QT3_NAMESPACE}test-case"):
        test = {
            "name": test_case.get("name"),
            "test": test_case.find(f"{QT3_NAMESPACE}test").text,
            "result": parse_result(test_case.find(f"{QT3_NAMESPACE}result")),
        }
        tests.append(test)
    return tests

def generate_noir_test(test: dict, func_name: str) -> str:
    """Generate Noir test function from qt3tests test case."""
    # ... implementation
    pass

def main():
    # Clone/update qt3tests repo
    # Parse catalog.xml
    # For each function in SPARQL_FUNCTIONS:
    #   Parse relevant test files
    #   Generate Noir test package
    #   Split into chunks of ~100 tests
    pass
```

### Test Package Structure

Each generated package:
- Maximum 100 tests per chunk
- Separate package per function to enable parallel CI
- Clear test naming for traceability

### Deliverables:
- `generate_tests.py` script
- Generated test packages for all implemented functions
- CI workflow running generated tests

---

## Phase 6: Documentation & Polish (Week 9)

### Documentation

1. **README.md**
   - Installation instructions
   - Quick start guide
   - Function reference with examples

2. **API Documentation**
   - Inline documentation for all public functions
   - Type documentation

3. **CONTRIBUTING.md**
   - Development setup
   - Testing guidelines
   - PR process

### Polish

1. Error messages and assertions
2. Performance optimization where needed
3. Edge case handling

### Deliverables:
- Complete documentation
- Polished API
- Release v1.0.0

---

## Priority Summary

### P0 (Must Have - Current Scope)
- All numeric operations (integers + floats via ieee754)
- DateTime construction and component extraction
- DateTime comparison (single-field efficient)
- Boolean operations

### P1 (Should Have - Current Scope)
- Float rounding functions (round, ceil, floor)
- Duration type support

### 🔮 Future (Deferred)
- **String functions**: All string operations deferred due to ZK complexity
- **Regex**: Deferred (depends on strings)
- **Hash functions**: Deferred (depends on strings for hex output)
- **Decimal type**: Deferred due to fixed-point arithmetic complexity
- **Timezone handling**: All storage is UTC; timezone conversion deferred

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| ieee754 API changes | Pin version, add integration tests |
| DateTime epoch overflow | Use Field which supports large values |
| Calendar arithmetic complexity | Use proven algorithms (Howard Hinnant) |
| Test coverage gaps | Manual test addition for edge cases |
| CI timeout | Aggressive test chunking |

---

## Success Metrics

1. **Correctness**: >95% qt3tests pass rate for implemented functions (numeric, datetime, boolean)
2. **Coverage**: All P0 functions implemented
3. **Performance**: Reasonable constraint counts for typical inputs
4. **Documentation**: All public APIs documented with examples
5. **Efficiency**: DateTime comparisons use single-field operations
