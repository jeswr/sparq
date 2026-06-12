# Copilot Instructions for xpath

XPath expression evaluation library for Noir (ZK proof DSL).

## Project Structure

This is a Noir workspace with:
- **`xpath/`**: Main library package with XPath implementation
- **`test_packages/`**: Auto-generated test packages for XPath functions - **never edit manually**
- **`scripts/`**: Test generation and benchmarking tools

## Architecture Overview

- **`xpath/src/`**: Core XPath implementation with function evaluations, operators, and datetime handling
- **`scripts/generate_tests.py`**: Generates test packages from qt3tests XML + Noir export inspection

## Test Generation (How It Works Now)

- **Source of truth for “available functions”**: qt3tests XML files under `scripts/qt3tests/{fn,op}/*.xml`.
- **Source of truth for “implemented”**: whether the corresponding Noir symbol is **exported** from `xpath/src/lib.nr`.
- **No `FUNCTION_MAP`**: the generator uses a deterministic resolver (`resolve_noir_symbol`) that derives candidate Noir symbol names from the qt3tests name and validates them against exports.

### Important behaviors

- **Subset generation safety**: when running with `--functions ...`, the generator does **not** delete other existing packages.
- **Stubs**: unimplemented functions generate tests that call stub functions; stubs live in `xpath/src/stubs.nr` and are re-exported from `xpath/src/lib.nr`.

## Noir Language Constraints

- **No early returns**: Use conditional assignment patterns; all paths must reach function end
- **Fixed-width integers**: `u1`, `u8`, `u16`, `u32`, `u64` only
- **Shift operand type matching**: Both operands in `value >> shift` must have same width
- **No floating-point primitives**: All FP arithmetic uses integer math
- **`pub` required**: Functions used across modules need `pub` keyword

## Key Implementation Patterns

### XPath Operations
The library implements various XPath functions and operators including:
- Numeric operations (add, subtract, multiply, divide, integer-divide, mod, equal, less-than, greater-than, less-than-or-equal, greater-than-or-equal)
- Datetime operations (equal, less-than, greater-than, less-than-or-equal, greater-than-or-equal)
- Datetime extraction functions (year, month, day, hours, minutes, seconds, timezone)
- Duration operations (add, subtract, multiply, divide, equal, less-than, greater-than, less-than-or-equal, greater-than-or-equal)
- Boolean operations (not, and, or, equal, less-than, greater-than, less-than-or-equal, greater-than-or-equal)
- Math functions (abs, round, ceiling, floor)
- Comparison utilities (equal, less-than, greater-than, less-than-or-equal, greater-than-or-equal)

## Common Bug Patterns

- **Type constraints**: Ensure proper type conversions for numeric operations
- **Boundary conditions**: Handle edge cases for datetime operations (leap years, month boundaries)
- **Overflow handling**: Be careful with integer arithmetic overflow in operations

## Developer Commands

```bash
# Run test packages locally
python3 scripts/generate_tests.py  # Generate test packages

# Useful generator flags
python3 scripts/generate_tests.py --list-functions
python3 scripts/generate_tests.py --list-all
python3 scripts/generate_tests.py --functions op:time-equal --skip-fmt

# Run manual unit tests only (from project root)
nargo test --package xpath_unit_tests

# Run specific test packages
nargo test --package xpath_test_fnabs
nargo test --package xpath_test_opnumeric_add
```

## Supported Operations

| Category | Operations | Test Generation |
|----------|-----------|-----------------|
| Numeric | add, subtract, multiply, divide, integer-divide, mod, equal, less-than, greater-than, less-than-or-equal, greater-than-or-equal | ✅ |
| Datetime | equal, less-than, greater-than, less-than-or-equal, greater-than-or-equal | ✅ |
| Datetime Extraction | year, month, day, hours, minutes, seconds, timezone | ✅ |
| Duration | add, subtract, multiply, divide, equal, less-than, greater-than, less-than-or-equal, greater-than-or-equal | ✅ |
| Boolean | not, and, or, equal, less-than, greater-than, less-than-or-equal, greater-than-or-equal | ✅ |
| Math | abs, round, ceiling, floor | ✅ |
| Comparison | equal, less-than, greater-than, less-than-or-equal, greater-than-or-equal | ✅ |

Notes:
- Whether a function/operator is treated as implemented is determined by `xpath/src/lib.nr` exports.
- Some qt3tests expectations may not match Noir semantics yet (e.g., timezone-presence nuances), so a generated package compiling does not guarantee all tests pass.

## Future Extensions

1. **String operations**: String functions from XPath spec
2. **Additional datetime functions**: timezone-from-datetime, etc.
3. **Node operations**: Node set operations from XPath

## Critical Files

| File | Purpose |
|------|---------|
| `xpath/src/` | All XPath function implementations |
| `scripts/generate_tests.py` | Test generation from test data |
| `test_packages/` | Generated test packages (one per test suite) |

## Code Quality

**Always run `nargo fmt` at the end of each run** to automatically format all Noir code and resolve linting errors. This ensures code style consistency across the project.

```bash
nargo fmt
```
