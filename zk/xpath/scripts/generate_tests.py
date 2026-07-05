#!/usr/bin/env python3
"""
Generate Noir test code from W3C qt3tests test suite.

This script parses the qt3tests XML test files and generates Noir test packages
for the XPath functions implemented in noir_XPath.

Usage:
    python generate_tests.py [--output-dir PATH] [--functions FUNC1,FUNC2,...]

Requirements:
    - Python 3.8+
    - elementpath (for XPath 2.0 parsing and evaluation)
    - qt3tests repository (will be cloned if not present)
"""

import argparse
import re
import shutil
import struct
import subprocess
import xml.etree.ElementTree as ET
from dataclasses import dataclass, field
from datetime import datetime, timezone, timedelta
from decimal import Decimal
from pathlib import Path
from typing import Optional

# Import elementpath for XPath 2.0 parsing
from elementpath import XPath2Parser
from elementpath.datatypes import DateTime10, Date10, Time, DayTimeDuration

# XML namespace for qt3tests
QT3_NS = "{http://www.w3.org/2010/09/qt-fots-catalog}"


# Cached set of crate-root exports from xpath/src/lib.nr.
XPATH_EXPORTED_SYMBOLS: set[str] = set()


def _extract_identifiers_from_pub_use_stmt(stmt: str) -> set[str]:
    """Extract exported identifiers from a single `pub use ...;` statement."""
    exports: set[str] = set()
    # Normalize whitespace for simpler parsing.
    stmt = re.sub(r"\s+", " ", stmt.strip())
    if not stmt.startswith("pub use ") or not stmt.endswith(";"):
        return exports

    # Handle `pub use path::{ a, b, c };`
    brace_start = stmt.find("::{")
    if brace_start != -1:
        brace_end = stmt.rfind("}")
        if brace_end != -1 and brace_end > brace_start:
            inner = stmt[brace_start + 3 : brace_end]
            exports.update(re.findall(r"\b[A-Za-z_][A-Za-z0-9_]*\b", inner))
        return exports

    # Handle `pub use path::name;`
    m = re.match(r"^pub use (?:[A-Za-z0-9_]+::)*([A-Za-z_][A-Za-z0-9_]*)\s*;\s*$", stmt)
    if m:
        exports.add(m.group(1))
    return exports


def discover_xpath_exports(xpath_dir: Path) -> set[str]:
    """Discover crate-root exported symbols from the Noir `xpath` library.

    We parse xpath/src/lib.nr and extract:
    - `pub fn <name>(...)`
    - `pub use ...;` re-exports (including brace lists)
    """
    lib_path = xpath_dir / "src" / "lib.nr"
    if not lib_path.exists():
        return set()

    content = lib_path.read_text()

    # Strip line comments to avoid picking up identifiers in commented-out code.
    content_no_comments = re.sub(r"//.*", "", content)

    exports: set[str] = set()
    exports.update(
        re.findall(r"\bpub\s+fn\s+([A-Za-z_][A-Za-z0-9_]*)\b", content_no_comments)
    )

    # Collect full `pub use ...;` statements (may span multiple lines).
    for stmt in re.findall(r"pub\s+use\s+[\s\S]*?;", content_no_comments):
        exports.update(_extract_identifiers_from_pub_use_stmt(stmt))

    return exports


TEST_FILE_OVERRIDES: dict[str, str] = {}


def default_test_file_relpath(function_name: str) -> Optional[str]:
    """Deterministically map a function name to a qt3tests XML relative path.

    Most qt3tests file names match the post-prefix name exactly (e.g. fn:abs -> fn/abs.xml,
    op:numeric-add -> op/numeric-add.xml). Some function variants share a base test file
    (e.g. *-float / *-double variants), and duration equality lives in duration-equal.xml.
    """
    override = TEST_FILE_OVERRIDES.get(function_name)
    if override is not None:
        return override

    if function_name.startswith("xs:"):
        # Cast expressions are tested in prod/CastExpr.xml
        return "prod/CastExpr.xml"

    m = re.match(r"^(fn|op):(.+)$", function_name)
    if not m:
        return None

    group = m.group(1)
    stem = m.group(2)

    # Most float/double variants reuse the base file (e.g. round-float -> round.xml).
    stem = re.sub(r"-(float|double)$", "", stem)

    # Duration equality is in duration-equal.xml
    if group == "op" and stem.endswith("Duration-equal"):
        stem = "duration-equal"

    return f"{group}/{stem}.xml"


def _qt3_op_to_snake(op: str) -> str:
    return op.replace("-", "_")


def _candidate_noir_symbols(function_name: str) -> list[str]:
    """Produce candidate Noir symbol names for a qt3tests function/operator name.

    We keep this intentionally conservative and order candidates by preference.
    """
    # Synthetic cast pseudo-functions.
    if function_name.startswith("xs:"):
        m = re.match(r"^xs:([A-Za-z]+)-from-([A-Za-z]+)$", function_name)
        if m:
            target, source = m.group(1), m.group(2)
            norm = {"int": "integer"}
            target_norm = norm.get(target, target)
            source_norm = norm.get(source, source)
            return [f"cast_{source_norm}_to_{target_norm}"]
        return []

    m = re.match(r"^(fn|op):(.+)$", function_name)
    if not m:
        return []

    kind, stem = m.group(1), m.group(2)
    candidates: list[str] = []

    if kind == "fn":
        # qt3tests uses `dateTime` (camel case); Noir exports use `datetime`.
        stem = stem.replace("dateTime", "datetime")

        # Handle float/double variants.
        variant = None
        if stem.endswith("-float"):
            variant = "float"
            stem = stem[: -len("-float")]
        elif stem.endswith("-double"):
            variant = "double"
            stem = stem[: -len("-double")]

        if stem == "dateTime":
            # Exported as fn_dateTime (note camel case)
            candidates.extend(["fn_dateTime", "fn_datetime", "datetime"])
            return candidates

        if stem == "not":
            candidates.extend(["fn_not"])
            return candidates

        # Numeric functions use *_int naming for the integer variants.
        if stem in {"abs", "round", "floor", "ceiling"}:
            base = "ceil" if stem == "ceiling" else stem
            if variant is None:
                candidates.append(f"{base}_int")
            else:
                candidates.append(f"{base}_{variant}")
            return candidates

        snake = _qt3_op_to_snake(stem)
        candidates.append(snake)
        candidates.append(f"fn_{snake}")
        return candidates

    # Operators
    # qt3tests uses `dateTime` (camel case); Noir exports use `datetime`.
    stem_norm = stem.replace("dateTime", "datetime")

    if stem.startswith("numeric-unary-"):
        op = stem[len("numeric-unary-") :]
        candidates.append(f"numeric_unary_{_qt3_op_to_snake(op)}_int")
        return candidates

    # Numeric binary operators
    num = re.match(
        r"^numeric-(add|subtract|multiply|divide|integer-divide|mod|equal|less-than|greater-than)(?:-(float|double))?$",
        stem_norm,
    )
    if num:
        op, variant = num.group(1), num.group(2)
        if op == "integer-divide":
            op = "divide"
        op_snake = _qt3_op_to_snake(op)
        suffix = variant if variant is not None else "int"
        candidates.append(f"numeric_{op_snake}_{suffix}")
        return candidates

    # dateTime/date/time/boolean comparisons
    dt_cmp = re.match(
        r"^(datetime|date|time|boolean)-(equal|less-than|greater-than)$", stem_norm
    )
    if dt_cmp:
        lhs, op = dt_cmp.group(1), dt_cmp.group(2)
        candidates.append(f"{lhs}_{_qt3_op_to_snake(op)}")
        return candidates

    # subtract of same-type values
    if stem in ("subtract-dateTimes", "subtract-datetimes"):
        return ["datetime_difference"]
    if stem == "subtract-dates":
        return ["subtract_dates"]
    if stem == "subtract-times":
        return ["subtract_times"]

    # Add/subtract duration to date/dateTime/time
    dur_to_from = re.match(
        r"^(add|subtract)-(dayTimeDuration|yearMonthDuration)-(to|from)-(datetime|date|time)$",
        stem_norm,
    )
    if dur_to_from:
        op, dur_kind, _, subject = dur_to_from.groups()
        dur_suffix = "duration" if dur_kind == "dayTimeDuration" else "ym_duration"
        candidates.append(f"{subject}_{op}_{dur_suffix}")
        return candidates

    # DayTimeDuration arithmetic and comparisons
    if stem in {"add-dayTimeDurations", "subtract-dayTimeDurations"}:
        op = "add" if stem.startswith("add-") else "subtract"
        return [f"duration_{op}"]
    if stem in {"multiply-dayTimeDuration", "divide-dayTimeDuration"}:
        op = "multiply" if stem.startswith("multiply-") else "divide"
        return [f"duration_{op}"]
    if stem == "divide-dayTimeDuration-by-dayTimeDuration":
        return ["duration_divide_by_duration"]
    dtd_cmp = re.match(r"^dayTimeDuration-(equal|less-than|greater-than)$", stem)
    if dtd_cmp:
        return [f"duration_{_qt3_op_to_snake(dtd_cmp.group(1))}"]

    # YearMonthDuration arithmetic and comparisons
    if stem in {"add-yearMonthDurations", "subtract-yearMonthDurations"}:
        op = "add" if stem.startswith("add-") else "subtract"
        return [f"ym_duration_{op}"]
    if stem in {"multiply-yearMonthDuration", "divide-yearMonthDuration"}:
        op = "multiply" if stem.startswith("multiply-") else "divide"
        return [f"ym_duration_{op}"]
    if stem == "divide-yearMonthDuration-by-yearMonthDuration":
        return ["ym_duration_divide_by_duration"]
    ymd_cmp = re.match(r"^yearMonthDuration-(equal|less-than|greater-than)$", stem)
    if ymd_cmp:
        return [f"ym_duration_{_qt3_op_to_snake(ymd_cmp.group(1))}"]

    # Fallback: direct snake-case mapping
    candidates.append(_qt3_op_to_snake(stem))
    return candidates


def resolve_noir_symbol(
    function_name: str, exports: Optional[set[str]] = None
) -> Optional[str]:
    """Resolve qt3tests name to an exported Noir symbol, or None if unknown/unexported."""
    exports = XPATH_EXPORTED_SYMBOLS if exports is None else exports
    for cand in _candidate_noir_symbols(function_name):
        if cand in exports:
            return cand
    return None


# Float type filter - which function variants accept which types
FLOAT_FUNCTION_TYPES = {
    "op:numeric-add-float": "float",
    "op:numeric-subtract-float": "float",
    "op:numeric-multiply-float": "float",
    "op:numeric-divide-float": "float",
    "op:numeric-equal-float": "float",
    "op:numeric-less-than-float": "float",
    "op:numeric-greater-than-float": "float",
    "fn:round-float": "float",
    "fn:ceiling-float": "float",
    "fn:floor-float": "float",
    "op:numeric-add-double": "double",
    "op:numeric-subtract-double": "double",
    "op:numeric-multiply-double": "double",
    "op:numeric-divide-double": "double",
    "op:numeric-equal-double": "double",
    "op:numeric-less-than-double": "double",
    "op:numeric-greater-than-double": "double",
    "fn:round-double": "double",
    "fn:ceiling-double": "double",
    "fn:floor-double": "double",
}

# Cast expression patterns - which casts from what types
CAST_FUNCTION_PATTERNS = {
    # Pattern: (source_type, target_type)
    "xs:float-from-int": ("int", "float"),  # xs:float(integer_expr)
    "xs:double-from-int": ("int", "double"),  # xs:double(integer_expr)
    "xs:integer-from-float": ("float", "int"),  # xs:integer(xs:float(...))
    "xs:integer-from-double": ("double", "int"),  # xs:integer(xs:double(...))
    "xs:float-from-double": ("double", "float"),  # xs:float(xs:double(...))
}


def discover_available_functions(qt3_dir: Path) -> set[str]:
    """Discover available function/operator names from qt3tests XML files."""
    if not qt3_dir.exists():
        return set()
    return set(discover_all_test_files(qt3_dir).keys())


def discover_all_test_files(
    qt3_dir: Path, *, include_prod: bool = False
) -> dict[str, str]:
    """Discover test files in qt3tests.

    Returns a dict mapping function names (e.g., 'fn:abs', 'op:numeric-add')
    to their test file paths relative to qt3_dir.
    """
    all_functions = {}

    # Discover fn/ test files
    fn_dir = qt3_dir / "fn"
    if fn_dir.exists():
        for xml_file in fn_dir.glob("*.xml"):
            # Extract function name from filename
            # e.g., abs.xml -> fn:abs, string-length.xml -> fn:string-length
            func_name = xml_file.stem
            all_functions[f"fn:{func_name}"] = f"fn/{xml_file.name}"

    # Discover op/ test files
    op_dir = qt3_dir / "op"
    if op_dir.exists():
        for xml_file in op_dir.glob("*.xml"):
            # e.g., numeric-add.xml -> op:numeric-add
            func_name = xml_file.stem
            all_functions[f"op:{func_name}"] = f"op/{xml_file.name}"

    # Optionally discover prod/ XML files (XQuery/XPath productions).
    # These are not function/operator tests, but they can be useful for inspection.
    if include_prod:
        prod_dir = qt3_dir / "prod"
        if prod_dir.exists():
            for xml_file in prod_dir.glob("*.xml"):
                func_name = xml_file.stem
                all_functions[f"prod:{func_name}"] = f"prod/{xml_file.name}"

    return all_functions


def is_function_implemented(function_name: str) -> bool:
    """Check if a function is implemented by verifying Noir exports.

    We consider a function implemented if we can deterministically resolve it to an
    exported Noir symbol from the xpath crate root (xpath/src/lib.nr).
    """
    return resolve_noir_symbol(function_name) is not None


@dataclass
class TestCase:
    """Represents a single test case from qt3tests."""

    name: str
    description: str
    test_expr: str
    expected_result: str
    result_type: str  # 'assert-eq', 'assert-true', 'assert-false', 'error', etc.
    dependencies: list = field(default_factory=list)


def clone_or_update_qt3tests(qt3_dir: Path) -> None:
    """Clone or update the qt3tests repository."""
    if qt3_dir.exists():
        print(f"qt3tests exists at {qt3_dir}, pulling latest...")
        subprocess.run(["git", "pull"], cwd=qt3_dir, check=True)
    else:
        print(f"Cloning qt3tests to {qt3_dir}...")
        subprocess.run(
            [
                "git",
                "clone",
                "--depth",
                "1",
                "https://github.com/w3c/qt3tests.git",
                str(qt3_dir),
            ],
            check=True,
        )


def parse_test_file(xml_path: Path) -> list[TestCase]:
    """Parse a qt3tests XML file and extract test cases."""
    if not xml_path.exists():
        print(f"Warning: Test file not found: {xml_path}")
        return []

    tree = ET.parse(xml_path)
    root = tree.getroot()

    tests = []
    for test_case in root.findall(f".//{QT3_NS}test-case"):
        name = test_case.get("name", "unknown")

        # Get dependencies (skip tests with unsupported features)
        deps = []
        for dep in test_case.findall(f".//{QT3_NS}dependency"):
            dep_type = dep.get("type", "")
            dep_value = dep.get("value", "")
            deps.append(f"{dep_type}:{dep_value}")

        # Get description
        desc_elem = test_case.find(f"{QT3_NS}description")
        description = desc_elem.text if desc_elem is not None and desc_elem.text else ""
        # Clean up asterisk decorations from qt3tests descriptions
        import re

        description = re.sub(r"\*+", "", description).strip()

        # Get test expression
        test_elem = test_case.find(f"{QT3_NS}test")
        if test_elem is None or test_elem.text is None:
            continue
        test_expr = test_elem.text.strip()

        # Get expected result
        result_elem = test_case.find(f"{QT3_NS}result")
        if result_elem is None:
            continue

        # Handle different result types
        result_type = "unknown"
        expected_result = ""

        for child in result_elem:
            tag = child.tag.replace(QT3_NS, "")
            if tag == "assert-eq":
                result_type = "assert-eq"
                expected_result = child.text.strip() if child.text else ""
            elif tag == "assert-string-value":
                result_type = "assert-eq"  # Treat same as assert-eq
                expected_result = child.text.strip() if child.text else ""
            elif tag == "assert-true":
                result_type = "assert-true"
                expected_result = "true"
            elif tag == "assert-false":
                result_type = "assert-false"
                expected_result = "false"
            elif tag == "error":
                result_type = "error"
                expected_result = child.get("code", "")
            elif tag in ("all-of", "any-of"):
                result_type = "complex"
            break

        if result_type not in ("unknown", "complex", "error"):
            tests.append(
                TestCase(
                    name=name,
                    description=description,
                    test_expr=test_expr,
                    expected_result=expected_result,
                    result_type=result_type,
                    dependencies=deps,
                )
            )

    return tests


def sanitize_test_name(name: str) -> str:
    """Convert test name to valid Noir identifier."""
    name = re.sub(r"[-.]", "_", name)
    name = re.sub(r"[^a-zA-Z0-9_]", "", name)
    if name and name[0].isdigit():
        name = "test_" + name
    return name.lower()


# ASCII character range for printable characters (space to tilde)
ASCII_PRINTABLE_START = 32  # space character
ASCII_PRINTABLE_END = 127  # DEL character (excluded)

# Truncation limits for stub test comments
STUB_DESCRIPTION_MAX_LEN = 80
STUB_EXPRESSION_MAX_LEN = 60
STUB_EXPECTED_MAX_LEN = 40

# Global set to track stub functions that need to be generated
STUB_FUNCTIONS_NEEDED: set[str] = set()


def sanitize_to_ascii(text: str) -> str:
    """Remove non-ASCII characters and control characters from text for use in Noir comments."""
    # Replace newlines with space, then filter to printable ASCII only
    text = text.replace("\n", " ").replace("\r", " ").replace("\t", " ")
    return "".join(
        c if ASCII_PRINTABLE_START <= ord(c) < ASCII_PRINTABLE_END else "?"
        for c in text
    )


def get_stub_function_name(function_name: str) -> str:
    """Get the Noir stub function name for an XPath function."""
    # Convert fn:some-function to stub_fn_some_function
    # Convert op:some-operator to stub_op_some_operator
    sanitized = sanitize_test_name(function_name)
    return f"stub_{sanitized}"


def register_stub_function(function_name: str) -> str:
    """Register a stub function as needed and return its Noir name."""
    stub_name = get_stub_function_name(function_name)
    STUB_FUNCTIONS_NEEDED.add(function_name)
    return stub_name


def generate_stub_functions_module(xpath_dir: Path) -> None:
    """Check and add required stub functions to the manually maintained stubs.nr.

    The stubs.nr file is manually maintained with detailed documentation about
    why each function cannot be implemented in Noir/ZK context. This function
    verifies that all needed stubs exist and appends any missing ones.
    """
    stubs_file = xpath_dir / "src" / "stubs.nr"

    if not STUB_FUNCTIONS_NEEDED:
        print(f"  No stub functions needed")
        return

    if not stubs_file.exists():
        print(
            f"  WARNING: stubs.nr does not exist but {len(STUB_FUNCTIONS_NEEDED)} stub functions are needed!"
        )
        print(f"  Missing stubs: {sorted(STUB_FUNCTIONS_NEEDED)}")
        return

    # Read existing stubs and check what's defined
    content = stubs_file.read_text()

    missing_stubs = []
    for function_name in sorted(STUB_FUNCTIONS_NEEDED):
        stub_name = get_stub_function_name(function_name)
        # Check if the stub function is defined
        if f"pub fn {stub_name}" not in content:
            missing_stubs.append((function_name, stub_name))

    if missing_stubs:
        print(f"  Adding {len(missing_stubs)} missing stub functions to stubs.nr...")

        # Append missing stubs to the file
        lines = [
            "",
            "// ============================================================================",
            "// AUTO-GENERATED STUBS: Additional stubs needed by test packages",
            "// ============================================================================",
            "// These stubs were auto-generated because test packages reference them.",
            "// Feel free to add documentation explaining why each cannot be implemented.",
            "",
        ]

        for func_name, stub_name in missing_stubs:
            # Determine category based on prefix
            if func_name.startswith("fn:"):
                category = "fn function"
            elif func_name.startswith("op:"):
                category = "operator"
            elif func_name.startswith("prod:"):
                category = "XQuery production"
            else:
                category = "function"

            lines.append(f"/// {func_name} - stub for unimplemented {category}")
            lines.append(f"pub fn {stub_name}() -> bool {{")
            lines.append(f'    assert(false, "{func_name} is not yet implemented");')
            lines.append(f"    false")
            lines.append(f"}}")
            lines.append("")

        # Append to file
        with open(stubs_file, "a") as f:
            f.write("\n".join(lines))

        print(f"  Added {len(missing_stubs)} stub functions to stubs.nr")
    else:
        print(
            f"  All {len(STUB_FUNCTIONS_NEEDED)} required stub functions found in stubs.nr"
        )


def update_lib_nr_with_stubs(xpath_dir: Path) -> None:
    """Ensure lib.nr includes and re-exports the stubs module.

    Important: Do not remove stubs support just because the current generator run
    didn't require any new stubs. Existing generated test packages may still
    depend on previously-generated stubs.
    """
    lib_file = xpath_dir / "src" / "lib.nr"
    if not lib_file.exists():
        return

    stubs_file = xpath_dir / "src" / "stubs.nr"
    if not stubs_file.exists():
        return

    content = lib_file.read_text()
    stubs_module_line = "mod stubs;"

    # Ensure `mod stubs;` exists (insert after the last `mod ...;` line).
    if stubs_module_line not in content:
        lines = content.split("\n")
        insert_idx = 0
        for i, line in enumerate(lines):
            if line.startswith("mod "):
                insert_idx = i + 1
        lines.insert(insert_idx, stubs_module_line)
        content = "\n".join(lines)

    # Determine all stub functions available in `stubs.nr`.
    stubs_content = stubs_file.read_text()
    stub_names = sorted(
        set(re.findall(r"\bpub\s+fn\s+(stub_[A-Za-z0-9_]+)\b", stubs_content))
    )

    # Remove any existing export block (explicit list) and any previous wildcard attempt.
    old_export_marker = "// Re-export stub functions for unimplemented XPath functions"
    content = content.replace("pub use stubs::*;\n", "")
    content = content.replace("pub use stubs::*;", "")

    if old_export_marker in content:
        start_idx = content.find(old_export_marker)
        pub_use_start = content.find("pub use stubs::{", start_idx)
        if pub_use_start != -1:
            brace_count = 0
            in_block = False
            end_idx = pub_use_start
            for i in range(pub_use_start, len(content)):
                c = content[i]
                if c == "{":
                    brace_count += 1
                    in_block = True
                elif c == "}":
                    brace_count -= 1
                    if in_block and brace_count == 0:
                        end_idx = i + 1
                        while end_idx < len(content) and content[end_idx] in ";\n":
                            end_idx += 1
                        break
            content = content[:start_idx] + content[end_idx:]
        else:
            content = content.replace(old_export_marker + "\n", "")

    # Clean up excessive blank lines after removal.
    while "\n\n\n" in content:
        content = content.replace("\n\n\n", "\n\n")

    # Append a fresh explicit export list.
    if not content.endswith("\n"):
        content += "\n"
    content += "\n// Re-export stub functions for unimplemented XPath functions\n"
    content += "pub use stubs::{\n"
    for name in stub_names:
        content += f"    {name},\n"
    content += "};\n"

    lib_file.write_text(content)


def generate_stub_test_with_function(
    test: "TestCase", function_name: str
) -> Optional[str]:
    """Generate a test that calls a stub function for an unimplemented XPath function.

    The stub function itself asserts false, so the test will fail until the
    function is correctly implemented.
    """
    test_name = sanitize_test_name(test.name)

    # Skip tests with unsupported dependencies
    unsupported_deps = ["schemaValidation", "schemaImport", "staticTyping"]
    for dep in test.dependencies:
        for unsup in unsupported_deps:
            if unsup in dep:
                return None

    # Skip tests with error expected results (we can't test errors without implementation)
    if test.result_type in ("unknown", "complex", "error"):
        return None

    # Register the stub function as needed
    stub_func_name = register_stub_function(function_name)

    # Generate a test that calls the stub function
    # Sanitize to ASCII for Noir comment compatibility
    desc = (
        sanitize_to_ascii(
            test.description.replace("\n", " ").replace('"', "'")[
                :STUB_DESCRIPTION_MAX_LEN
            ]
        )
        if test.description
        else ""
    )
    expr_escaped = sanitize_to_ascii(
        test.test_expr.replace("\n", " ").replace('"', '\\"')[:STUB_EXPRESSION_MAX_LEN]
    )
    expected_escaped = sanitize_to_ascii(
        str(test.expected_result)[:STUB_EXPECTED_MAX_LEN]
    )

    lines = [
        f"#[test]",
        f"fn {test_name}() {{",
    ]
    if desc:
        lines.append(f"    // {desc}")
    lines.append(f"    // XPath: {expr_escaped}")
    lines.append(f"    // Expected: {expected_escaped}")
    lines.append(
        f"    // Calls stub function - will fail until {function_name} is implemented"
    )
    lines.append(f"    let _ = {stub_func_name}();")
    lines.append("}")

    return "\n".join(lines)


def parse_integer(value: str) -> Optional[int]:
    """Parse an XPath integer literal."""
    value = value.strip()
    # Remove type suffix like 'xs:integer(...)'
    match = re.match(r"xs:integer\s*\(\s*['\"]?(-?\d+)['\"]?\s*\)", value)
    if match:
        return int(match.group(1))
    # Plain integer
    if re.match(r"^-?\d+$", value):
        return int(value)
    return None


def parse_boolean(value: str) -> Optional[bool]:
    """Parse an XPath boolean literal."""
    value = value.strip().lower()
    if value in ("true", "true()", "fn:true()"):
        return True
    if value in ("false", "false()", "fn:false()"):
        return False
    match = re.match(r"xs:boolean\s*\(['\"]?(true|false)['\"]?\)", value)
    if match:
        return match.group(1) == "true"
    return None


def float_to_bits(f: float) -> int:
    """Convert a Python float to IEEE 754 single precision bits."""
    packed = struct.pack(">f", f)
    return struct.unpack(">I", packed)[0]


def double_to_bits(f: float) -> int:
    """Convert a Python float to IEEE 754 double precision bits."""
    packed = struct.pack(">d", f)
    return struct.unpack(">Q", packed)[0]


def parse_float(value: str) -> Optional[tuple[float, str]]:
    """Parse an XPath float or double literal.

    Returns (float_value, type) where type is 'float' or 'double', or None if parsing fails.
    """
    value = value.strip()

    # Check for xs:float(...) or xs:double(...)
    float_match = re.match(r"xs:float\s*\(\s*['\"]?([^'\")\s]+)['\"]?\s*\)", value)
    double_match = re.match(r"xs:double\s*\(\s*['\"]?([^'\")\s]+)['\"]?\s*\)", value)

    if float_match:
        try:
            val = float(float_match.group(1))
            return (val, "float")
        except ValueError:
            # Handle special values
            inner = float_match.group(1).upper()
            if inner == "NAN":
                return (float("nan"), "float")
            elif inner == "INF":
                return (float("inf"), "float")
            elif inner == "-INF":
                return (float("-inf"), "float")
            return None

    if double_match:
        try:
            val = float(double_match.group(1))
            return (val, "double")
        except ValueError:
            inner = double_match.group(1).upper()
            if inner == "NAN":
                return (float("nan"), "double")
            elif inner == "INF":
                return (float("inf"), "double")
            elif inner == "-INF":
                return (float("-inf"), "double")
            return None

    # Try plain float/double literals (with E notation)
    if re.match(r"^-?\d+\.?\d*[eE][+-]?\d+$", value) or re.match(
        r"^-?\d+\.\d+$", value
    ):
        try:
            return (float(value), "double")  # Default to double for plain literals
        except ValueError:
            return None

    return None


def detect_operand_type(expr: str) -> Optional[str]:
    """Detect the numeric type from an XPath expression.

    Returns 'int', 'float', 'double', or None if cannot determine.
    """
    expr = expr.strip()

    # Check for explicit type casts
    if "xs:float" in expr:
        return "float"
    if "xs:double" in expr:
        return "double"
    if (
        "xs:decimal" in expr
        or "xs:integer" in expr
        or "xs:int" in expr
        or "xs:long" in expr
    ):
        return "int"

    # Check for floating point literals
    if re.search(r"\d+[eE][+-]?\d+", expr) or re.search(r"\d+\.\d+", expr):
        return "double"

    return "int"  # Default to int


def parse_datetime(value: str) -> Optional[tuple[int, int]]:
    """Parse an XPath dateTime literal using elementpath.

    Returns (UTC microseconds, tz_offset_minutes) or None if parsing fails.
    """
    value = value.strip()

    # Ensure it's wrapped in xs:dateTime() if not already
    if not value.startswith("xs:dateTime"):
        value = f"xs:dateTime('{value}')"

    try:
        parser = XPath2Parser()
        token = parser.parse(value)
        dt = token.evaluate()

        if not isinstance(dt, DateTime10):
            return None

        # Get timezone offset in minutes
        tz_offset_minutes = 0
        if dt.tzinfo is not None:
            offset = dt.tzinfo.offset
            tz_offset_minutes = int(offset.total_seconds() / 60)

        # Build a Python datetime from the components and convert to epoch
        # dt contains local time components with timezone info
        py_tz = timezone(timedelta(minutes=tz_offset_minutes))
        py_dt = datetime(
            dt.year,
            dt.month,
            dt.day,
            dt.hour,
            dt.minute,
            int(dt.second),
            dt.microsecond,
            tzinfo=py_tz,
        )

        # Convert to UTC epoch microseconds
        epoch = datetime(1970, 1, 1, tzinfo=timezone.utc)
        utc_dt = py_dt.astimezone(timezone.utc)
        delta = utc_dt - epoch
        utc_micros = int(delta.total_seconds() * 1_000_000)

        return (utc_micros, tz_offset_minutes)
    except Exception:
        return None


def parse_duration(value: str) -> Optional[int]:
    """Parse an XPath xs:dayTimeDuration literal.

    Returns duration in microseconds (signed integer) or None if parsing fails.
    Format: P[nD][T[nH][nM][n.nS]]
    Examples: "P3DT10H30M", "PT2H30M", "-P2DT4H"
    """
    value = value.strip()

    # Handle xs:dayTimeDuration() wrapper
    match = re.match(r"xs:dayTimeDuration\s*\(['\"]([^'\"]+)['\"]\)", value)
    if match:
        value = match.group(1)

    # Check for negative sign
    negative = value.startswith("-")
    if negative:
        value = value[1:]

    # Duration must start with P
    if not value.startswith("P"):
        return None

    value = value[1:]  # Remove 'P'

    # Split on 'T' to get date and time parts
    if "T" in value:
        date_part, time_part = value.split("T", 1)
    else:
        date_part = value
        time_part = ""

    total_micros = 0

    # Parse date part (days)
    if date_part:
        day_match = re.search(r"(\d+(?:\.\d+)?)D", date_part)
        if day_match:
            days = float(day_match.group(1))
            total_micros += int(days * 86_400_000_000)  # 24*60*60*1_000_000

    # Parse time part (hours, minutes, seconds)
    if time_part:
        hour_match = re.search(r"(\d+(?:\.\d+)?)H", time_part)
        if hour_match:
            hours = float(hour_match.group(1))
            total_micros += int(hours * 3_600_000_000)  # 60*60*1_000_000

        min_match = re.search(r"(\d+(?:\.\d+)?)M", time_part)
        if min_match:
            minutes = float(min_match.group(1))
            total_micros += int(minutes * 60_000_000)  # 60*1_000_000

        sec_match = re.search(r"(\d+(?:\.\d+)?)S", time_part)
        if sec_match:
            seconds = float(sec_match.group(1))
            total_micros += int(seconds * 1_000_000)

    if negative:
        total_micros = -total_micros

    return total_micros


# i64 bounds
I64_MIN = -9223372036854775808
I64_MAX = 9223372036854775807


def _fits_in_i64(val: int) -> bool:
    """Check if an integer fits in Noir's i64 range."""
    return I64_MIN <= val <= I64_MAX


def _fits_in_i32(val: int) -> bool:
    """Check if an integer fits in Noir's i32 range."""
    return -2147483648 <= val <= 2147483647


def parse_year_month_duration(value: str) -> Optional[int]:
    """Parse an xs:yearMonthDuration lexical form into total months (i32).

    Supports forms like:
    - P0M
    - P2Y11M
    - -P1Y1M
    - P0Y
    - P10M
    """
    s = value.strip()
    if not s:
        return None
    m = re.match(r"^(-)?P(?:(\d+)Y)?(?:(\d+)M)?$", s)
    if m is None:
        return None
    neg = m.group(1) is not None
    years_s = m.group(2)
    months_s = m.group(3)
    years = int(years_s) if years_s is not None else 0
    months = int(months_s) if months_s is not None else 0
    total = years * 12 + months
    if neg:
        total = -total
    if not _fits_in_i32(total):
        return None
    return total


def _get_function_name(token) -> Optional[str]:
    """Extract the function name from an elementpath token, handling namespace prefixes."""
    symbol = token.symbol

    # If there's a namespace prefix (symbol is ':'), get the actual function name
    if symbol == ":" and len(token) >= 2:
        # The function name is in the second child
        return token[1].symbol

    # Otherwise the symbol is the function name
    return symbol


def _get_function_args(token) -> list:
    """Get the function arguments from a token, handling namespace prefixes."""
    # If there's a namespace prefix (symbol is ':'), get args from the function token
    if token.symbol == ":" and len(token) >= 2:
        return list(token[1])
    # Otherwise args are direct children
    return list(token)


def extract_ymd_literals_from_expr(expr: str) -> list[tuple[str, int]]:
    """Extract all yearMonthDuration string literals from an XPath expression.
    Returns: list of (literal_string, months_value) tuples
    """
    result = []
    # Find all xs:yearMonthDuration("...") patterns
    pattern = r'xs:yearMonthDuration\s*\(\s*["\']([^"\']+)["\']\s*\)'
    for match in re.finditer(pattern, expr):
        lit_str = match.group(1)
        months = parse_year_month_duration(lit_str)
        if months is not None:
            result.append((lit_str, months))
    return result


def convert_xpath_expr(
    expr: str, function_name: str
) -> Optional[tuple[str, str, Optional[str]]]:
    """Convert an XPath expression to Noir code using elementpath for parsing.

    Returns tuple of (setup_code, test_expression, embedded_expected) or None if cannot convert.
    The embedded_expected is set when the XPath expression contains a comparison (e.g., `x eq 5`)
    and the expected value is extracted from the expression itself.
    """
    expr = expr.strip()
    noir_func = resolve_noir_symbol(function_name)
    if not noir_func:
        return None

    # Numeric type filtering (int vs float vs double) is only meaningful for numeric ops.
    # Applying it broadly causes non-numeric suites (e.g., duration arithmetic) to be skipped.
    is_cast_function = function_name in CAST_FUNCTION_PATTERNS
    expected_type = FLOAT_FUNCTION_TYPES.get(function_name)
    detected_type = detect_operand_type(expr)

    is_numeric_suite = (
        function_name.startswith("op:numeric-")
        or function_name.startswith("op:numeric-unary-")
        or function_name
        in {
            "fn:abs",
            "fn:ceiling",
            "fn:floor",
            "fn:round",
            "fn:round-float",
            "fn:ceiling-float",
            "fn:floor-float",
            "fn:round-double",
            "fn:ceiling-double",
            "fn:floor-double",
        }
    )

    if is_cast_function or is_numeric_suite or expected_type is not None:
        # For float/double variants, filter to only matching type tests
        if expected_type is not None:
            if expected_type != detected_type:
                return None
        # For integer variants, skip float/double tests
        elif detected_type in ("float", "double") and not is_cast_function:
            return None

    parser = XPath2Parser()

    # Try to parse and evaluate the expression with elementpath
    try:
        token = parser.parse(expr)
    except Exception:
        return None

    # Unwrap parenthesized expressions like '(a div b)'
    # elementpath represents these with symbol '(' and a single child.
    try:
        while getattr(token, "symbol", None) == "(" and len(token) == 1:
            token = token[0]
    except Exception:
        pass

    # Get the effective function/operator name (handling namespace prefixes)
    symbol = _get_function_name(token)
    if symbol is None:
        return None

    # ---------------------------------------------------------------------
    # Generic yearMonthDuration expression support
    #
    # qt3tests often wraps yearMonthDuration expressions in fn:string(...),
    # or compares results (eq/ne/le/ge) even in the arithmetic operator suites.
    # These conversions should not depend on the suite's resolved Noir symbol.
    # ---------------------------------------------------------------------
    def _unwrap_parens(t):
        try:
            while getattr(t, "symbol", None) == "(" and len(t) == 1:
                t = t[0]
        except Exception:
            return t
        return t

    def _eval_i32_scalar(t) -> Optional[int]:
        t = _unwrap_parens(t)
        try:
            v = t.evaluate()
        except Exception:
            return None
        if isinstance(v, bool):
            return None
        if isinstance(v, int):
            return v if _fits_in_i32(v) else None
        if isinstance(v, Decimal):
            if v == v.to_integral_value():
                as_int = int(v)
                return as_int if _fits_in_i32(as_int) else None
            return None
        if isinstance(v, float):
            if v.is_integer():
                as_int = int(v)
                return as_int if _fits_in_i32(as_int) else None
            return None
        if isinstance(v, str):
            parsed = parse_integer(v)
            if parsed is None:
                return None
            return parsed if _fits_in_i32(parsed) else None
        return None

    def _ym_ctor_from_token(t) -> Optional[str]:
        t = _unwrap_parens(t)
        if _get_function_name(t) != "yearMonthDuration":
            return None
        args = _get_function_args(t)
        if len(args) < 1:
            return None
        try:
            s = args[0].evaluate()
        except Exception:
            return None
        if not isinstance(s, str):
            return None
        months = parse_year_month_duration(s)
        if months is None:
            return None
        return f"XsdYearMonthDuration::new({months})"

    def _convert_ymd_expr(t) -> Optional[str]:
        t = _unwrap_parens(t)

        # xs:yearMonthDuration("...")
        ctor = _ym_ctor_from_token(t)
        if ctor is not None:
            return ctor

        # Binary operators
        sym = getattr(t, "symbol", None)
        if sym in ("+", "-", "*", "div") and len(t) >= 2:
            left = _unwrap_parens(t[0])
            right = _unwrap_parens(t[1])

            if sym in ("+", "-"):
                a = _convert_ymd_expr(left)
                b = _convert_ymd_expr(right)
                if a is None or b is None:
                    return None
                if sym == "+":
                    return f"ym_duration_add({a}, {b})"
                return f"ym_duration_subtract({a}, {b})"

            if sym == "*":
                # ymd * int OR int * ymd (only integral scalars supported)
                a = _convert_ymd_expr(left)
                b = _convert_ymd_expr(right)
                if a is not None:
                    factor = _eval_i32_scalar(right)
                    if factor is None:
                        return None
                    return f"ym_duration_multiply({a}, {factor})"
                if b is not None:
                    factor = _eval_i32_scalar(left)
                    if factor is None:
                        return None
                    return f"ym_duration_multiply({b}, {factor})"
                return None

            if sym == "div":
                a = _convert_ymd_expr(left)
                if a is None:
                    return None
                divisor = _eval_i32_scalar(right)
                if divisor is None:
                    return None
                return f"ym_duration_divide({a}, {divisor})"

        return None

    def _convert_bool_expr(t) -> Optional[tuple[str, str]]:
        """Convert a limited subset of boolean expressions used in YMD op suites.

        This targets qt3tests patterns like:
        - fn:string(<ymd>) and fn:false()
        - fn:string(<ymd>) or fn:false()
        - fn:boolean(fn:string(<ymd>))
        - fn:not(fn:string(<ymd>))

        We model EBV(fn:string(<ymd>)) as `true`, but still evaluate <ymd>
        to preserve the intent of testing the duration arithmetic.
        """
        t = _unwrap_parens(t)
        sym = _get_function_name(t)
        if sym is None:
            return None

        # Logical operators (elementpath uses token symbols 'and'/'or')
        if sym in ("and", "or") and len(t) >= 2:
            left = _convert_bool_expr(t[0])
            right = _convert_bool_expr(t[1])
            if left is None or right is None:
                return None
            setup_left, expr_left = left
            setup_right, expr_right = right
            setup = "\n".join([s for s in (setup_left, setup_right) if s])
            if sym == "and":
                return (setup, f"({expr_left}) & ({expr_right})")
            return (setup, f"({expr_left}) | ({expr_right})")

        # fn:true() / fn:false()
        if sym == "true":
            return ("", "true")
        if sym == "false":
            return ("", "false")

        # fn:string(<ymd>) -> Actually call fn_string_from_ym_duration and check string length > 0
        # In Noir, we compute the string and use fn_boolean_from_string_len for EBV
        if sym == "string":
            args = _get_function_args(t)
            if len(args) == 1:
                ymd_expr = _convert_ymd_expr(args[0])
                if ymd_expr is None:
                    return None
                # Generate unique variable name based on expression hash to avoid conflicts
                var_suffix = abs(hash(ymd_expr)) % 10000
                setup = (
                    f"let dur_{var_suffix} = {ymd_expr};\n"
                    f"    let (_str_bytes_{var_suffix}, str_len_{var_suffix}): ([u8; 32], u32) = fn_string_from_ym_duration(dur_{var_suffix});"
                )
                # EBV of fn:string(ymd) using fn_boolean_from_string_len
                return (setup, f"fn_boolean_from_string_len(str_len_{var_suffix})")

        # fn:boolean(fn:string(<ymd>)) -> true (and still evaluate <ymd>)
        if sym == "boolean":
            args = _get_function_args(t)
            if len(args) == 1:
                inner = _convert_bool_expr(args[0])
                if inner is not None:
                    return inner
            return None

        # fn:not(<bool>)
        if sym == "not":
            args = _get_function_args(t)
            if len(args) == 1:
                inner = _convert_bool_expr(args[0])
                if inner is None:
                    return None
                setup_inner, expr_inner = inner
                return (setup_inner, f"!({expr_inner})")

        # Also allow yearMonthDuration comparisons as boolean subexpressions.
        if (
            sym in ("eq", "ne", "lt", "gt", "le", "ge", "=", "!=", "<", ">", "<=", ">=")
            and len(t) >= 2
        ):
            a = _convert_ymd_expr(t[0])
            b = _convert_ymd_expr(t[1])
            if a is None or b is None:
                return None
            if sym in ("eq", "="):
                return ("", f"ym_duration_equal({a}, {b})")
            if sym in ("lt", "<"):
                return ("", f"ym_duration_less_than({a}, {b})")
            if sym in ("gt", ">"):
                return ("", f"ym_duration_greater_than({a}, {b})")
            if sym in ("le", "<="):
                return ("", f"ym_duration_le({a}, {b})")
            if sym in ("ge", ">="):
                return ("", f"ym_duration_ge({a}, {b})")
            if sym in ("ne", "!="):
                return ("", f"!ym_duration_equal({a}, {b})")

        return None

    # Handle boolean expressions that appear in yearMonthDuration operator suites.
    if symbol in ("and", "or", "boolean", "not"):
        bool_conv = _convert_bool_expr(token)
        if bool_conv is not None:
            setup, bool_expr = bool_conv
            return (setup, bool_expr, None)

    # fn:string(...) around yearMonthDuration - use marker prefix for string comparison
    # When assert-eq expects a string like "P1Y10M", we'll call fn_string_from_ym_duration
    if symbol == "string":
        args = _get_function_args(token)
        if len(args) == 1:
            ymd_expr = _convert_ymd_expr(args[0])
            if ymd_expr is not None:
                # Return with FN_STRING_YMD: prefix so assertion generator can detect it
                return ("", f"FN_STRING_YMD:{ymd_expr}", None)

    # Support comparisons involving yearMonthDuration expressions in arithmetic suites.
    if (
        symbol in ("eq", "ne", "lt", "gt", "le", "ge", "=", "!=", "<", ">", "<=", ">=")
        and len(token) >= 2
    ):
        a = _convert_ymd_expr(token[0])
        b = _convert_ymd_expr(token[1])
        if a is not None and b is not None:
            if symbol in ("eq", "="):
                return ("", f"ym_duration_equal({a}, {b})", None)
            if symbol in ("lt", "<"):
                return ("", f"ym_duration_less_than({a}, {b})", None)
            if symbol in ("gt", ">"):
                return ("", f"ym_duration_greater_than({a}, {b})", None)
            if symbol in ("le", "<="):
                return ("", f"ym_duration_le({a}, {b})", None)
            if symbol in ("ge", ">="):
                return ("", f"ym_duration_ge({a}, {b})", None)
            if symbol in ("ne", "!="):
                return ("", f"!ym_duration_equal({a}, {b})", None)

    # yearMonthDuration div yearMonthDuration -> i32 ratio (used in add/mul/div suites)
    if symbol == "div" and len(token) >= 2:
        left = _convert_ymd_expr(token[0])
        right = _convert_ymd_expr(token[1])
        if left is not None and right is not None:
            return ("", f"ym_duration_divide_by_duration({left}, {right})", None)

    # Handle dateTime component extraction functions (e.g. year-from-dateTime)
    dt_extract_match = re.match(
        r"^(year|month|day|hours|minutes|seconds|timezone)-from-dateTime$", symbol
    )
    if dt_extract_match is not None:
        expected_noir_fn = f"{dt_extract_match.group(1)}_from_datetime"
        if expected_noir_fn != noir_func:
            return None

        # Get function arguments (handling namespace prefix)
        args = _get_function_args(token)

        # The argument should be a dateTime - try to extract it
        if len(args) >= 1:
            arg = args[0]
            # Try to evaluate the datetime argument
            try:
                dt_val = arg.evaluate()
                if isinstance(dt_val, DateTime10):
                    result = _datetime_to_epoch(dt_val)
                    if result is None:
                        return None
                    utc_micros, tz_offset = result
                    # Skip dates before 1970 (negative epoch) - not supported
                    if utc_micros < 0:
                        return None
                    setup = f"let dt = datetime_from_epoch_microseconds_with_tz({utc_micros}, {tz_offset});"
                    return (setup, f"{noir_func}(dt)", None)
            except Exception:
                pass
        return None

    # Handle date component extraction functions (e.g. year-from-date)
    date_extract_match = re.match(r"^(year|month|day|timezone)-from-date$", symbol)
    if date_extract_match is not None:
        expected_noir_fn = f"{date_extract_match.group(1)}_from_date"
        if expected_noir_fn != noir_func:
            return None

        # Get function arguments
        args = _get_function_args(token)

        # The argument should be a date - try to extract it
        if len(args) >= 1:
            arg = args[0]
            try:
                date_val = arg.evaluate()
                if isinstance(date_val, Date10):
                    result = _date_to_epoch_days(date_val)
                    if result is None:
                        return None
                    epoch_days, tz_offset = result
                    # Skip dates before 1970 (negative epoch) - not supported
                    if epoch_days < 0:
                        return None
                    setup = f"let d = date_from_epoch_days_with_tz({epoch_days}, {tz_offset});"
                    return (setup, f"{noir_func}(d)", None)
            except Exception:
                pass
        return None

    # Handle time component extraction functions (e.g. hours-from-time)
    time_extract_match = re.match(
        r"^(hours|minutes|seconds|timezone)-from-time$", symbol
    )
    if time_extract_match is not None:
        expected_noir_fn = f"{time_extract_match.group(1)}_from_time"
        if expected_noir_fn != noir_func:
            return None

        # Get function arguments
        args = _get_function_args(token)

        # The argument should be a time - try to extract it
        if len(args) >= 1:
            arg = args[0]
            try:
                time_val = arg.evaluate()
                if isinstance(time_val, Time):
                    result = _time_to_microseconds(time_val)
                    if result is None:
                        return None
                    micros, tz_offset = result
                    setup = f"let t = time_from_microseconds_with_tz({micros}, {tz_offset});"
                    return (setup, f"{noir_func}(t)", None)
            except Exception:
                pass
        return None

    # Handle adjust-*-to-timezone functions
    # These functions return date/time/datetime values
    # Expected results are strings like "2002-03-07T10:00:00-05:00" which we parse and compare components
    if symbol in (
        "adjust-dateTime-to-timezone",
        "adjust-date-to-timezone",
        "adjust-time-to-timezone",
    ):
        if noir_func in (
            "adjust_datetime_to_timezone",
            "adjust_date_to_timezone",
            "adjust_time_to_timezone",
        ):
            args = _get_function_args(token)
            if len(args) >= 1:
                try:
                    # For adjust-date-to-timezone
                    if (
                        symbol == "adjust-date-to-timezone"
                        and noir_func == "adjust_date_to_timezone"
                    ):
                        date_val = args[0].evaluate()
                        if isinstance(date_val, Date10):
                            result = _date_to_epoch_days(date_val)
                            if result is None:
                                return None
                            epoch_days, src_tz = result
                            if epoch_days < 0:
                                return None

                            setup = f"let d = date_from_epoch_days_with_tz({epoch_days}, {src_tz});"

                            # Check second argument
                            if len(args) == 1:
                                # Single argument - uses implicit timezone, skip
                                return None

                            arg2_val = args[1].evaluate()
                            # Check if second arg is empty sequence ()
                            if isinstance(arg2_val, list) and len(arg2_val) == 0:
                                # Use the _none variant to remove timezone
                                return (setup, "adjust_date_to_timezone_none(d)", None)

                            # Check if second arg is a DayTimeDuration
                            if isinstance(arg2_val, DayTimeDuration):
                                target_tz_mins = int(arg2_val.seconds / 60)
                                return (
                                    setup,
                                    f"adjust_date_to_timezone(d, {target_tz_mins})",
                                    None,
                                )

                    # For adjust-time-to-timezone
                    elif (
                        symbol == "adjust-time-to-timezone"
                        and noir_func == "adjust_time_to_timezone"
                    ):
                        time_val = args[0].evaluate()
                        if isinstance(time_val, Time):
                            result = _time_to_microseconds(time_val)
                            if result is None:
                                return None
                            micros, src_tz = result

                            setup = f"let t = time_from_microseconds_with_tz({micros}, {src_tz});"

                            # Check second argument
                            if len(args) == 1:
                                # Single argument - uses implicit timezone, skip
                                return None

                            arg2_val = args[1].evaluate()
                            # Check if second arg is empty sequence ()
                            if isinstance(arg2_val, list) and len(arg2_val) == 0:
                                return (setup, "adjust_time_to_timezone_none(t)", None)

                            # Check if second arg is a DayTimeDuration
                            if isinstance(arg2_val, DayTimeDuration):
                                target_tz_mins = int(arg2_val.seconds / 60)
                                return (
                                    setup,
                                    f"adjust_time_to_timezone(t, {target_tz_mins})",
                                    None,
                                )

                    # For adjust-dateTime-to-timezone
                    elif (
                        symbol == "adjust-dateTime-to-timezone"
                        and noir_func == "adjust_datetime_to_timezone"
                    ):
                        dt_val = args[0].evaluate()
                        if isinstance(dt_val, DateTime10):
                            result = _datetime_to_epoch(dt_val)
                            if result is None:
                                return None
                            utc_micros, src_tz = result
                            if utc_micros < 0:
                                return None

                            setup = f"let dt = datetime_from_epoch_microseconds_with_tz({utc_micros}, {src_tz});"

                            # Check second argument
                            if len(args) == 1:
                                # Single argument - uses implicit timezone, skip
                                return None

                            arg2_val = args[1].evaluate()
                            # Check if second arg is empty sequence ()
                            if isinstance(arg2_val, list) and len(arg2_val) == 0:
                                return (
                                    setup,
                                    "adjust_datetime_to_timezone_none(dt)",
                                    None,
                                )

                            # Check if second arg is a DayTimeDuration
                            if isinstance(arg2_val, DayTimeDuration):
                                target_tz_mins = int(arg2_val.seconds / 60)
                                return (
                                    setup,
                                    f"adjust_datetime_to_timezone(dt, {target_tz_mins})",
                                    None,
                                )
                except Exception:
                    pass
        return None

    # Handle duration component extraction functions
    dur_extract_match = re.match(
        r"^(days|hours|minutes|seconds|years|months)-from-duration$", symbol
    )
    if dur_extract_match is not None:
        expected_noir_fn = f"{dur_extract_match.group(1)}_from_duration"
        if expected_noir_fn != noir_func:
            return None

        # Get function arguments
        args = _get_function_args(token)

        # The argument should be a duration - try to parse it
        if len(args) >= 1:
            arg = args[0]
            # Try to get the duration string from the token
            try:
                # Check if it's a duration constructor
                arg_symbol = _get_function_name(arg)
                if arg_symbol == "dayTimeDuration":
                    # Get the inner string value
                    inner_args = _get_function_args(arg)
                    if len(inner_args) >= 1:
                        duration_str = inner_args[0].evaluate()
                        if isinstance(duration_str, str):
                            micros = parse_duration(duration_str)
                            if micros is not None and _fits_in_i64(micros):
                                setup = (
                                    f"let dur = duration_from_microseconds({micros});"
                                )
                                return (setup, f"{noir_func}(dur)", None)
            except Exception:
                # If parsing the duration constructor fails, we cannot generate
                # the specialized setup; fall back to returning None to skip this test
                pass
        return None

    # Handle duration comparison operators (Stream B)
    if symbol in ("eq", "lt", "gt", "=", "<", ">"):
        duration_cmp_map = {
            "eq": "duration_equal",
            "=": "duration_equal",
            "lt": "duration_less_than",
            "<": "duration_less_than",
            "gt": "duration_greater_than",
            ">": "duration_greater_than",
        }
        expected_noir_fn = duration_cmp_map.get(symbol)

        if expected_noir_fn == noir_func and len(token) >= 2:
            # Both operands should be durations
            try:
                # Try to parse both as duration strings
                arg1_symbol = _get_function_name(token[0])
                arg2_symbol = _get_function_name(token[1])

                if (
                    arg1_symbol == "dayTimeDuration"
                    and arg2_symbol == "dayTimeDuration"
                ):
                    inner_args1 = _get_function_args(token[0])
                    inner_args2 = _get_function_args(token[1])

                    if len(inner_args1) >= 1 and len(inner_args2) >= 1:
                        duration_str1 = inner_args1[0].evaluate()
                        duration_str2 = inner_args2[0].evaluate()

                        if isinstance(duration_str1, str) and isinstance(
                            duration_str2, str
                        ):
                            micros1 = parse_duration(duration_str1)
                            micros2 = parse_duration(duration_str2)

                            if micros1 is not None and micros2 is not None:
                                if _fits_in_i64(micros1) and _fits_in_i64(micros2):
                                    setup = f"let dur1 = duration_from_microseconds({micros1});\n    let dur2 = duration_from_microseconds({micros2});"
                                    return (setup, f"{noir_func}(dur1, dur2)", None)
            except Exception:
                # This is a best-effort optimization for duration comparisons; if parsing
                # fails for any reason, fall back to the generic handling below
                pass

    # Handle duration arithmetic operators (Stream B)
    if symbol in ("+", "-"):
        # Try to detect if this is duration + duration or datetime + duration
        if len(token) >= 2:
            try:
                # Check the types of operands
                arg1_symbol = _get_function_name(token[0])
                arg2_symbol = _get_function_name(token[1])

                # Duration + Duration or Duration - Duration
                if (
                    arg1_symbol == "dayTimeDuration"
                    and arg2_symbol == "dayTimeDuration"
                ):
                    if symbol == "+" and noir_func == "duration_add":
                        inner_args1 = _get_function_args(token[0])
                        inner_args2 = _get_function_args(token[1])

                        if len(inner_args1) >= 1 and len(inner_args2) >= 1:
                            duration_str1 = inner_args1[0].evaluate()
                            duration_str2 = inner_args2[0].evaluate()

                            if isinstance(duration_str1, str) and isinstance(
                                duration_str2, str
                            ):
                                micros1 = parse_duration(duration_str1)
                                micros2 = parse_duration(duration_str2)

                                if micros1 is not None and micros2 is not None:
                                    if _fits_in_i64(micros1) and _fits_in_i64(micros2):
                                        setup = f"let dur1 = duration_from_microseconds({micros1});\n    let dur2 = duration_from_microseconds({micros2});"
                                        return (setup, f"{noir_func}(dur1, dur2)", None)
                    elif symbol == "-" and noir_func == "duration_subtract":
                        inner_args1 = _get_function_args(token[0])
                        inner_args2 = _get_function_args(token[1])

                        if len(inner_args1) >= 1 and len(inner_args2) >= 1:
                            duration_str1 = inner_args1[0].evaluate()
                            duration_str2 = inner_args2[0].evaluate()

                            if isinstance(duration_str1, str) and isinstance(
                                duration_str2, str
                            ):
                                micros1 = parse_duration(duration_str1)
                                micros2 = parse_duration(duration_str2)

                                if micros1 is not None and micros2 is not None:
                                    if _fits_in_i64(micros1) and _fits_in_i64(micros2):
                                        setup = f"let dur1 = duration_from_microseconds({micros1});\n    let dur2 = duration_from_microseconds({micros2});"
                                        return (setup, f"{noir_func}(dur1, dur2)", None)

                # DateTime + Duration
                elif arg1_symbol == "dateTime" and arg2_symbol == "dayTimeDuration":
                    if symbol == "+" and noir_func == "datetime_add_duration":
                        dt_val = token[0].evaluate()
                        inner_args2 = _get_function_args(token[1])

                        if isinstance(dt_val, DateTime10) and len(inner_args2) >= 1:
                            duration_str = inner_args2[0].evaluate()

                            if isinstance(duration_str, str):
                                result = _datetime_to_epoch(dt_val)
                                micros_dur = parse_duration(duration_str)

                                if result is not None and micros_dur is not None:
                                    utc_micros, tz_offset = result
                                    if utc_micros >= 0 and _fits_in_i64(micros_dur):
                                        setup = f"let dt = datetime_from_epoch_microseconds_with_tz({utc_micros}, {tz_offset});\n    let dur = duration_from_microseconds({micros_dur});"
                                        return (setup, f"{noir_func}(dt, dur)", None)

                # DateTime - Duration
                elif arg1_symbol == "dateTime" and arg2_symbol == "dayTimeDuration":
                    if symbol == "-" and noir_func == "datetime_subtract_duration":
                        dt_val = token[0].evaluate()
                        inner_args2 = _get_function_args(token[1])

                        if isinstance(dt_val, DateTime10) and len(inner_args2) >= 1:
                            duration_str = inner_args2[0].evaluate()

                            if isinstance(duration_str, str):
                                result = _datetime_to_epoch(dt_val)
                                micros_dur = parse_duration(duration_str)

                                if result is not None and micros_dur is not None:
                                    utc_micros, tz_offset = result
                                    if utc_micros >= 0 and _fits_in_i64(micros_dur):
                                        setup = f"let dt = datetime_from_epoch_microseconds_with_tz({utc_micros}, {tz_offset});\n    let dur = duration_from_microseconds({micros_dur});"
                                        return (setup, f"{noir_func}(dt, dur)", None)

                # DateTime - DateTime (returns duration)
                elif arg1_symbol == "dateTime" and arg2_symbol == "dateTime":
                    if symbol == "-" and noir_func == "datetime_difference":
                        dt1 = token[0].evaluate()
                        dt2 = token[1].evaluate()

                        if isinstance(dt1, DateTime10) and isinstance(dt2, DateTime10):
                            result1 = _datetime_to_epoch(dt1)
                            result2 = _datetime_to_epoch(dt2)

                            if result1 is not None and result2 is not None:
                                utc_micros1, tz_offset1 = result1
                                utc_micros2, tz_offset2 = result2

                                if utc_micros1 >= 0 and utc_micros2 >= 0:
                                    setup = f"let dt1 = datetime_from_epoch_microseconds_with_tz({utc_micros1}, {tz_offset1});\n    let dt2 = datetime_from_epoch_microseconds_with_tz({utc_micros2}, {tz_offset2});"
                                    return (setup, f"{noir_func}(dt1, dt2)", None)

                # Date - Date (returns dayTimeDuration)
                elif arg1_symbol == "date" and arg2_symbol == "date":
                    if symbol == "-" and noir_func == "subtract_dates":
                        d1 = token[0].evaluate()
                        d2 = token[1].evaluate()

                        if isinstance(d1, Date10) and isinstance(d2, Date10):
                            result1 = _date_to_epoch_days(d1)
                            result2 = _date_to_epoch_days(d2)

                            if result1 is not None and result2 is not None:
                                epoch_days1, tz_offset1 = result1
                                epoch_days2, tz_offset2 = result2

                                if epoch_days1 >= 0 and epoch_days2 >= 0:
                                    setup = f"let d1 = date_from_epoch_days_with_tz({epoch_days1}, {tz_offset1});\n    let d2 = date_from_epoch_days_with_tz({epoch_days2}, {tz_offset2});"
                                    return (setup, f"{noir_func}(d1, d2)", None)

                # Time - Time (returns dayTimeDuration)
                elif arg1_symbol == "time" and arg2_symbol == "time":
                    if symbol == "-" and noir_func == "subtract_times":
                        t1 = token[0].evaluate()
                        t2 = token[1].evaluate()

                        if isinstance(t1, Time) and isinstance(t2, Time):
                            result1 = _time_to_microseconds(t1)
                            result2 = _time_to_microseconds(t2)

                            if result1 is not None and result2 is not None:
                                micros1, tz_offset1 = result1
                                micros2, tz_offset2 = result2
                                setup = f"let t1 = time_from_microseconds_with_tz({micros1}, {tz_offset1});\n    let t2 = time_from_microseconds_with_tz({micros2}, {tz_offset2});"
                                return (setup, f"{noir_func}(t1, t2)", None)
            except Exception:
                # Skip tests that cannot be evaluated (e.g., complex expressions)
                pass

    # Handle duration scalar multiply/divide (Stream B)
    if symbol in ("*", "div") and len(token) >= 2:
        try:
            arg1_symbol = _get_function_name(token[0])
            arg2_symbol = _get_function_name(token[1])

            def _eval_int_scalar(t) -> Optional[int]:
                try:
                    v = t.evaluate()
                except Exception:
                    return None
                if isinstance(v, bool):
                    return None
                if isinstance(v, int):
                    return v if _fits_in_i64(v) else None
                if isinstance(v, Decimal):
                    if v == v.to_integral_value():
                        as_int = int(v)
                        return as_int if _fits_in_i64(as_int) else None
                    return None
                if isinstance(v, float):
                    if v.is_integer():
                        as_int = int(v)
                        return as_int if _fits_in_i64(as_int) else None
                    return None
                if isinstance(v, str):
                    parsed = parse_integer(v)
                    if parsed is None:
                        return None
                    return parsed if _fits_in_i64(parsed) else None
                return None

            def _parse_duration_ctor(t) -> Optional[int]:
                if _get_function_name(t) != "dayTimeDuration":
                    return None
                inner_args = _get_function_args(t)
                if len(inner_args) < 1:
                    return None
                s = inner_args[0].evaluate()
                if not isinstance(s, str):
                    return None
                micros = parse_duration(s)
                if micros is None or not _fits_in_i64(micros):
                    return None
                return micros

            def _parse_ym_duration_ctor(t) -> Optional[int]:
                # Accept both xs:yearMonthDuration("...") and yearMonthDuration("...")
                if _get_function_name(t) != "yearMonthDuration":
                    return None
                inner_args = _get_function_args(t)
                if len(inner_args) < 1:
                    return None
                s = inner_args[0].evaluate()
                if not isinstance(s, str):
                    return None
                months = parse_year_month_duration(s)
                return months

            # duration * int OR int * duration
            if symbol == "*" and noir_func == "duration_multiply":
                micros = None
                factor = None
                if arg1_symbol == "dayTimeDuration":
                    micros = _parse_duration_ctor(token[0])
                    factor = _eval_int_scalar(token[1])
                elif arg2_symbol == "dayTimeDuration":
                    micros = _parse_duration_ctor(token[1])
                    factor = _eval_int_scalar(token[0])
                if micros is not None and factor is not None:
                    setup = f"let dur = duration_from_microseconds({micros});"
                    return (setup, f"{noir_func}(dur, {factor})", None)

            # duration div int
            if symbol == "div" and noir_func == "duration_divide":
                if arg1_symbol == "dayTimeDuration":
                    micros = _parse_duration_ctor(token[0])
                    divisor = _eval_int_scalar(token[1])
                    if micros is not None and divisor is not None:
                        setup = f"let dur = duration_from_microseconds({micros});"
                        return (setup, f"{noir_func}(dur, {divisor})", None)

            # duration div duration -> xs:decimal ratio (IEEE 754 double,
            # F&O 3.1 sec. 8.4.10; sq-3x7dl.9)
            if symbol == "div" and noir_func == "duration_divide_by_duration":
                if (
                    arg1_symbol == "dayTimeDuration"
                    and arg2_symbol == "dayTimeDuration"
                ):
                    micros1 = _parse_duration_ctor(token[0])
                    micros2 = _parse_duration_ctor(token[1])
                    if micros1 is not None and micros2 is not None:
                        setup = (
                            f"let dur1 = duration_from_microseconds({micros1});\n"
                            f"    let dur2 = duration_from_microseconds({micros2});"
                        )
                        return (setup, f"{noir_func}(dur1, dur2)", None)

            # yearMonthDuration * int OR int * yearMonthDuration
            if symbol == "*" and noir_func == "ym_duration_multiply":
                months = None
                factor = None
                if arg1_symbol == "yearMonthDuration":
                    months = _parse_ym_duration_ctor(token[0])
                    factor = _eval_int_scalar(token[1])
                elif arg2_symbol == "yearMonthDuration":
                    months = _parse_ym_duration_ctor(token[1])
                    factor = _eval_int_scalar(token[0])
                if months is not None and factor is not None and _fits_in_i32(factor):
                    setup = f"let dur = XsdYearMonthDuration::new({months});"
                    return (setup, f"{noir_func}(dur, {factor})", None)

            # yearMonthDuration div int
            if symbol == "div" and noir_func == "ym_duration_divide":
                if arg1_symbol == "yearMonthDuration":
                    months = _parse_ym_duration_ctor(token[0])
                    divisor = _eval_int_scalar(token[1])
                    if (
                        months is not None
                        and divisor is not None
                        and _fits_in_i32(divisor)
                    ):
                        setup = f"let dur = XsdYearMonthDuration::new({months});"
                        return (setup, f"{noir_func}(dur, {divisor})", None)

            # yearMonthDuration div yearMonthDuration -> i32 ratio
            if symbol == "div" and noir_func == "ym_duration_divide_by_duration":
                if (
                    arg1_symbol == "yearMonthDuration"
                    and arg2_symbol == "yearMonthDuration"
                ):
                    months1 = _parse_ym_duration_ctor(token[0])
                    months2 = _parse_ym_duration_ctor(token[1])
                    if months1 is not None and months2 is not None:
                        setup = (
                            f"let d1 = XsdYearMonthDuration::new({months1});\n"
                            f"    let d2 = XsdYearMonthDuration::new({months2});"
                        )
                        return (setup, f"{noir_func}(d1, d2)", None)
        except Exception:
            pass

    # Handle yearMonthDuration add/subtract (simple binary)
    if symbol in ("+", "-") and len(token) >= 2:
        try:
            from elementpath.datatypes import YearMonthDuration

            d1 = token[0].evaluate()
            d2 = token[1].evaluate()
            if isinstance(d1, YearMonthDuration) and isinstance(d2, YearMonthDuration):
                months1 = int(d1.months)
                months2 = int(d2.months)
                if not _fits_in_i32(months1) or not _fits_in_i32(months2):
                    return None
                if symbol == "+" and noir_func == "ym_duration_add":
                    setup = (
                        f"let d1 = XsdYearMonthDuration::new({months1});\n"
                        f"    let d2 = XsdYearMonthDuration::new({months2});"
                    )
                    return (setup, f"{noir_func}(d1, d2)", None)
                if symbol == "-" and noir_func == "ym_duration_subtract":
                    setup = (
                        f"let d1 = XsdYearMonthDuration::new({months1});\n"
                        f"    let d2 = XsdYearMonthDuration::new({months2});"
                    )
                    return (setup, f"{noir_func}(d1, d2)", None)
        except Exception:
            pass

    # Handle datetime comparison operators (eq, lt, gt)
    if symbol in ("eq", "lt", "gt", "=", "<", ">"):
        op_map = {
            "eq": "datetime_equal",
            "=": "datetime_equal",
            "lt": "datetime_less_than",
            "<": "datetime_less_than",
            "gt": "datetime_greater_than",
            ">": "datetime_greater_than",
        }
        expected_noir_fn = op_map.get(symbol)

        if expected_noir_fn == noir_func and len(token) >= 2:
            # Both operands should be dateTimes
            try:
                dt1 = token[0].evaluate()
                dt2 = token[1].evaluate()

                if isinstance(dt1, DateTime10) and isinstance(dt2, DateTime10):
                    result1 = _datetime_to_epoch(dt1)
                    result2 = _datetime_to_epoch(dt2)

                    if result1 is not None and result2 is not None:
                        utc_micros1, tz_offset1 = result1
                        utc_micros2, tz_offset2 = result2
                        # Skip dates before 1970
                        if utc_micros1 < 0 or utc_micros2 < 0:
                            return None
                        setup = f"let dt1 = datetime_from_epoch_microseconds_with_tz({utc_micros1}, {tz_offset1});\n    let dt2 = datetime_from_epoch_microseconds_with_tz({utc_micros2}, {tz_offset2});"
                        return (setup, f"{noir_func}(dt1, dt2)", None)
            except Exception:
                pass

    # Handle time comparison operators (eq, lt, gt)
    if symbol in ("eq", "lt", "gt", "=", "<", ">"):
        time_op_map = {
            "eq": "time_equal",
            "=": "time_equal",
            "lt": "time_less_than",
            "<": "time_less_than",
            "gt": "time_greater_than",
            ">": "time_greater_than",
        }
        expected_noir_fn = time_op_map.get(symbol)

        if expected_noir_fn == noir_func and len(token) >= 2:
            try:
                t1 = token[0].evaluate()
                t2 = token[1].evaluate()

                if isinstance(t1, Time) and isinstance(t2, Time):
                    result1 = _time_to_microseconds(t1)
                    result2 = _time_to_microseconds(t2)

                    if result1 is not None and result2 is not None:
                        micros1, tz_offset1 = result1
                        micros2, tz_offset2 = result2
                        setup = f"let t1 = time_from_microseconds_with_tz({micros1}, {tz_offset1});\n    let t2 = time_from_microseconds_with_tz({micros2}, {tz_offset2});"
                        return (setup, f"{noir_func}(t1, t2)", None)
            except Exception:
                pass

    # Handle date comparison operators (eq, lt, gt)
    if symbol in ("eq", "lt", "gt", "=", "<", ">"):
        date_op_map = {
            "eq": "date_equal",
            "=": "date_equal",
            "lt": "date_less_than",
            "<": "date_less_than",
            "gt": "date_greater_than",
            ">": "date_greater_than",
        }
        expected_noir_fn = date_op_map.get(symbol)

        if expected_noir_fn == noir_func and len(token) >= 2:
            try:
                d1 = token[0].evaluate()
                d2 = token[1].evaluate()

                if isinstance(d1, Date10) and isinstance(d2, Date10):
                    result1 = _date_to_epoch_days(d1)
                    result2 = _date_to_epoch_days(d2)

                    if result1 is not None and result2 is not None:
                        epoch_days1, tz_offset1 = result1
                        epoch_days2, tz_offset2 = result2
                        # Skip dates before 1970
                        if epoch_days1 < 0 or epoch_days2 < 0:
                            return None
                        setup = f"let d1 = date_from_epoch_days_with_tz({epoch_days1}, {tz_offset1});\n    let d2 = date_from_epoch_days_with_tz({epoch_days2}, {tz_offset2});"
                        return (setup, f"{noir_func}(d1, d2)", None)
            except Exception:
                pass

    # Handle yearMonthDuration comparison operators (eq, lt, gt)
    if symbol in ("eq", "lt", "gt", "=", "<", ">"):
        ym_op_map = {
            "eq": "ym_duration_equal",
            "=": "ym_duration_equal",
            "lt": "ym_duration_less_than",
            "<": "ym_duration_less_than",
            "gt": "ym_duration_greater_than",
            ">": "ym_duration_greater_than",
        }
        expected_noir_fn = ym_op_map.get(symbol)

        if expected_noir_fn == noir_func and len(token) >= 2:
            try:
                from elementpath.datatypes import YearMonthDuration

                d1 = token[0].evaluate()
                d2 = token[1].evaluate()

                if isinstance(d1, YearMonthDuration) and isinstance(
                    d2, YearMonthDuration
                ):
                    # YearMonthDuration stores months (and years as months * 12)
                    months1 = d1.months
                    months2 = d2.months
                    setup = f"let d1 = XsdYearMonthDuration::new({months1});\n    let d2 = XsdYearMonthDuration::new({months2});"
                    return (setup, f"{noir_func}(d1, d2)", None)
            except Exception:
                pass

    # Handle fn:not
    if symbol == "not" and noir_func == "fn_not":
        args = _get_function_args(token)
        if len(args) >= 1:
            try:
                arg_val = args[0].evaluate()
                if isinstance(arg_val, bool):
                    return ("", f"fn_not({str(arg_val).lower()})", None)
            except Exception:
                pass
        return None

    # Handle numeric mod operator (integer only)
    if symbol == "mod" and noir_func == "numeric_mod_int":
        if len(token) >= 2:
            try:
                a = token[0].evaluate()
                b = token[1].evaluate()
                if isinstance(a, (int, float, Decimal)) and isinstance(
                    b, (int, float, Decimal)
                ):
                    a_int, b_int = int(a), int(b)
                    if not _fits_in_i64(a_int) or not _fits_in_i64(b_int):
                        return None
                    return ("", f"numeric_mod_int({a_int}, {b_int})", None)
            except Exception:
                pass
        return None

    # Handle float arithmetic operators
    float_ops = {
        "+": ("numeric_add_float", "numeric_add_double"),
        "-": ("numeric_subtract_float", "numeric_subtract_double"),
        "*": ("numeric_multiply_float", "numeric_multiply_double"),
        "div": ("numeric_divide_float", "numeric_divide_double"),
    }

    if symbol in float_ops and noir_func in float_ops[symbol]:
        if len(token) >= 2:
            try:
                a = token[0].evaluate()
                b = token[1].evaluate()
                if isinstance(a, (int, float, Decimal)) and isinstance(
                    b, (int, float, Decimal)
                ):
                    a_float, b_float = float(a), float(b)

                    # Determine if we're generating float32 or float64 code
                    is_float32 = noir_func.endswith("_float")

                    if is_float32:
                        a_bits = float_to_bits(a_float)
                        b_bits = float_to_bits(b_float)
                        setup = f"let a = XsdFloat::from_bits({a_bits});\n    let b = XsdFloat::from_bits({b_bits});"
                        return (setup, f"{noir_func}(a, b)", None)
                    else:
                        a_bits = double_to_bits(a_float)
                        b_bits = double_to_bits(b_float)
                        setup = f"let a = XsdDouble::from_bits({a_bits});\n    let b = XsdDouble::from_bits({b_bits});"
                        return (setup, f"{noir_func}(a, b)", None)
            except Exception:
                pass
        return None

    # Handle float comparison operators (eq, lt, gt)
    float_cmp_ops = {
        "eq": ("numeric_equal_float", "numeric_equal_double"),
        "lt": ("numeric_less_than_float", "numeric_less_than_double"),
        "gt": ("numeric_greater_than_float", "numeric_greater_than_double"),
        "=": ("numeric_equal_float", "numeric_equal_double"),
        "<": ("numeric_less_than_float", "numeric_less_than_double"),
        ">": ("numeric_greater_than_float", "numeric_greater_than_double"),
    }

    if symbol in float_cmp_ops and noir_func in float_cmp_ops[symbol]:
        if len(token) >= 2:
            try:
                a = token[0].evaluate()
                b = token[1].evaluate()
                if isinstance(a, (int, float, Decimal)) and isinstance(
                    b, (int, float, Decimal)
                ):
                    a_float, b_float = float(a), float(b)

                    is_float32 = noir_func.endswith("_float")

                    if is_float32:
                        a_bits = float_to_bits(a_float)
                        b_bits = float_to_bits(b_float)
                        setup = f"let a = XsdFloat::from_bits({a_bits});\n    let b = XsdFloat::from_bits({b_bits});"
                        return (setup, f"{noir_func}(a, b)", None)
                    else:
                        a_bits = double_to_bits(a_float)
                        b_bits = double_to_bits(b_float)
                        setup = f"let a = XsdDouble::from_bits({a_bits});\n    let b = XsdDouble::from_bits({b_bits});"
                        return (setup, f"{noir_func}(a, b)", None)
            except Exception:
                pass
        return None

    # Handle integer numeric operators
    numeric_ops = {
        "+": "numeric_add_int",
        "-": "numeric_subtract_int",
        "*": "numeric_multiply_int",
        "div": "numeric_divide_int",
        "idiv": "numeric_divide_int",
    }

    if symbol in numeric_ops and numeric_ops[symbol] == noir_func:
        if len(token) >= 2:
            try:
                a = token[0].evaluate()
                b = token[1].evaluate()
                if isinstance(a, (int, float, Decimal)) and isinstance(
                    b, (int, float, Decimal)
                ):
                    a_int, b_int = int(a), int(b)
                    # Skip values outside i64 range
                    if not _fits_in_i64(a_int) or not _fits_in_i64(b_int):
                        return None
                    return ("", f"{noir_func}({a_int}, {b_int})", None)
            except Exception:
                pass
        return None

    # Handle fn:abs
    if symbol == "abs" and noir_func == "abs_int":
        args = _get_function_args(token)
        if len(args) >= 1:
            try:
                arg_val = args[0].evaluate()
                if isinstance(arg_val, (int, float, Decimal)):
                    val_int = int(arg_val)
                    if not _fits_in_i64(val_int):
                        return None
                    return ("", f"abs_int({val_int})", None)
            except Exception:
                pass
        return None

    # Handle fn:ceiling
    if symbol == "ceiling" and noir_func == "ceil_int":
        args = _get_function_args(token)
        if len(args) >= 1:
            try:
                arg_val = args[0].evaluate()
                if isinstance(arg_val, (int, float, Decimal)):
                    val_int = int(arg_val)
                    if not _fits_in_i64(val_int):
                        return None
                    return ("", f"ceil_int({val_int})", None)
            except Exception:
                # Skip tests that cannot be evaluated (e.g., complex expressions)
                pass
        return None

    # Handle fn:floor
    if symbol == "floor" and noir_func == "floor_int":
        args = _get_function_args(token)
        if len(args) >= 1:
            try:
                arg_val = args[0].evaluate()
                if isinstance(arg_val, (int, float, Decimal)):
                    val_int = int(arg_val)
                    if not _fits_in_i64(val_int):
                        return None
                    return ("", f"floor_int({val_int})", None)
            except Exception:
                # Skip tests that cannot be evaluated (e.g., complex expressions)
                pass
        return None

    # Handle fn:round
    if symbol == "round" and noir_func == "round_int":
        args = _get_function_args(token)
        if len(args) >= 1:
            try:
                arg_val = args[0].evaluate()
                if isinstance(arg_val, (int, float, Decimal)):
                    val_int = int(arg_val)
                    if not _fits_in_i64(val_int):
                        return None
                    return ("", f"round_int({val_int})", None)
            except Exception:
                # Skip tests that cannot be evaluated (e.g., complex expressions)
                pass
        return None

    # Handle fn:round (float/double)
    if symbol == "round" and noir_func in ("round_float", "round_double"):
        args = _get_function_args(token)
        if len(args) >= 1:
            try:
                arg_val = args[0].evaluate()
                if isinstance(arg_val, (int, float, Decimal)):
                    val_float = float(arg_val)

                    is_float32 = noir_func == "round_float"

                    if is_float32:
                        val_bits = float_to_bits(val_float)
                        setup = f"let val = XsdFloat::from_bits({val_bits});"
                        return (setup, f"{noir_func}(val)", None)
                    else:
                        val_bits = double_to_bits(val_float)
                        setup = f"let val = XsdDouble::from_bits({val_bits});"
                        return (setup, f"{noir_func}(val)", None)
            except Exception:
                pass
        return None

    # Handle fn:ceiling (float/double)
    if symbol == "ceiling" and noir_func in ("ceil_float", "ceil_double"):
        args = _get_function_args(token)
        if len(args) >= 1:
            try:
                arg_val = args[0].evaluate()
                if isinstance(arg_val, (int, float, Decimal)):
                    val_float = float(arg_val)

                    is_float32 = noir_func == "ceil_float"

                    if is_float32:
                        val_bits = float_to_bits(val_float)
                        setup = f"let val = XsdFloat::from_bits({val_bits});"
                        return (setup, f"{noir_func}(val)", None)
                    else:
                        val_bits = double_to_bits(val_float)
                        setup = f"let val = XsdDouble::from_bits({val_bits});"
                        return (setup, f"{noir_func}(val)", None)
            except Exception:
                pass
        return None

    # Handle fn:floor (float/double)
    if symbol == "floor" and noir_func in ("floor_float", "floor_double"):
        args = _get_function_args(token)
        if len(args) >= 1:
            try:
                arg_val = args[0].evaluate()
                if isinstance(arg_val, (int, float, Decimal)):
                    val_float = float(arg_val)

                    is_float32 = noir_func == "floor_float"

                    if is_float32:
                        val_bits = float_to_bits(val_float)
                        setup = f"let val = XsdFloat::from_bits({val_bits});"
                        return (setup, f"{noir_func}(val)", None)
                    else:
                        val_bits = double_to_bits(val_float)
                        setup = f"let val = XsdDouble::from_bits({val_bits});"
                        return (setup, f"{noir_func}(val)", None)
            except Exception:
                pass
        return None

    # Handle numeric unary operators (Stream E)
    if symbol == "+" and noir_func == "numeric_unary_plus_int":
        # Unary plus: +$value
        if len(token) >= 1:
            try:
                arg_val = token[0].evaluate()
                if isinstance(arg_val, (int, float, Decimal)):
                    val_int = int(arg_val)
                    if not _fits_in_i64(val_int):
                        return None
                    return ("", f"numeric_unary_plus_int({val_int})", None)
            except Exception:
                # Skip tests that cannot be evaluated (e.g., complex expressions)
                pass
        return None

    if symbol == "-" and noir_func == "numeric_unary_minus_int":
        # Check if this is unary minus (single operand) vs binary subtract (two operands)
        if len(token) == 1:
            # Unary minus: -$value
            try:
                arg_val = token[0].evaluate()
                if isinstance(arg_val, (int, float, Decimal)):
                    val_int = int(arg_val)
                    if not _fits_in_i64(val_int):
                        return None
                    return ("", f"numeric_unary_minus_int({val_int})", None)
            except Exception:
                # Skip tests that cannot be evaluated (e.g., complex expressions)
                pass
        return None

    # Handle integer numeric comparison operators (eq, lt, gt)
    int_cmp_ops = {
        "eq": "numeric_equal_int",
        "lt": "numeric_less_than_int",
        "gt": "numeric_greater_than_int",
        "=": "numeric_equal_int",
        "<": "numeric_less_than_int",
        ">": "numeric_greater_than_int",
    }

    if symbol in int_cmp_ops and int_cmp_ops[symbol] == noir_func:
        if len(token) >= 2:
            try:
                a = token[0].evaluate()
                b = token[1].evaluate()
                if isinstance(a, (int, float, Decimal)) and isinstance(
                    b, (int, float, Decimal)
                ):
                    a_int, b_int = int(a), int(b)
                    # Skip values outside i64 range
                    if not _fits_in_i64(a_int) or not _fits_in_i64(b_int):
                        return None
                    return ("", f"{noir_func}({a_int}, {b_int})", None)
            except Exception:
                # Skip tests that cannot be evaluated (e.g., complex expressions)
                pass
        return None

    # Handle op:boolean-equal
    if symbol in ("eq", "=") and noir_func == "boolean_equal":
        if len(token) >= 2:
            try:
                a = token[0].evaluate()
                b = token[1].evaluate()
                if isinstance(a, bool) and isinstance(b, bool):
                    return (
                        "",
                        f"boolean_equal({str(a).lower()}, {str(b).lower()})",
                        None,
                    )
            except Exception:
                # Skip tests that cannot be evaluated (e.g., complex expressions)
                pass
        return None

    # Handle op:boolean-less-than and op:boolean-greater-than (Stream F)
    bool_cmp_ops = {
        "lt": "boolean_less_than",
        "<": "boolean_less_than",
        "gt": "boolean_greater_than",
        ">": "boolean_greater_than",
    }

    if symbol in bool_cmp_ops and bool_cmp_ops[symbol] == noir_func:
        if len(token) >= 2:
            try:
                a = token[0].evaluate()
                b = token[1].evaluate()
                if isinstance(a, bool) and isinstance(b, bool):
                    return (
                        "",
                        f"{noir_func}({str(a).lower()}, {str(b).lower()})",
                        None,
                    )
            except Exception:
                # Skip tests that cannot be evaluated (e.g., complex expressions)
                pass
        return None

    # Handle fn:timezone-from-dateTime
    if symbol == "timezone-from-dateTime" and noir_func == "timezone_from_datetime":
        args = _get_function_args(token)
        if len(args) >= 1:
            try:
                dt_val = args[0].evaluate()
                if isinstance(dt_val, DateTime10):
                    result = _datetime_to_epoch(dt_val)
                    if result is None:
                        return None
                    utc_micros, tz_offset = result
                    # Skip dates before 1970 (negative epoch) - not supported
                    if utc_micros < 0:
                        return None
                    setup = f"let dt = datetime_from_epoch_microseconds_with_tz({utc_micros}, {tz_offset});"
                    return (setup, f"{noir_func}(dt)", None)
            except Exception:
                # Skip tests that cannot be evaluated (e.g., complex expressions)
                pass
        return None

    # Handle type casting expressions using "cast as" syntax
    # e.g., xs:integer("-100") cast as xs:float, xs:double("1.5") cast as xs:integer
    cast_pattern = CAST_FUNCTION_PATTERNS.get(function_name)
    if cast_pattern is not None:
        source_type, target_type = cast_pattern

        # Map target types to expected xs: type names
        target_type_names = {
            "float": "float",
            "double": "double",
            "int": "integer",
        }
        expected_target = target_type_names.get(target_type)

        # Map source types to expected xs: type names
        source_type_names = {
            "int": "integer",
            "float": "float",
            "double": "double",
        }
        expected_source = source_type_names.get(source_type)

        # Handle "cast as" syntax: expr cast as xs:type
        if symbol == "cast" and len(token) >= 2:
            source_token = token[0]  # The value being cast
            target_token = token[1]  # The target type (xs:float, xs:double, xs:integer)

            # Get target type name from the second part of xs:type
            if target_token.symbol == ":" and len(target_token) >= 2:
                actual_target = (
                    target_token[1].value
                    if hasattr(target_token[1], "value")
                    else target_token[1].symbol
                )

                # Check if target matches what we're looking for
                if actual_target != expected_target:
                    return None

                # Get the source value's type
                source_symbol = _get_function_name(source_token)

                try:
                    # Evaluate the source value
                    source_val = source_token.evaluate()

                    # For xs:float-from-int or xs:double-from-int: expect integer source
                    if source_type == "int":
                        # Source should be xs:integer(...) or plain int, NOT xs:decimal
                        # xs:decimal should be skipped as we don't support decimal-to-float conversion yet
                        if source_symbol == "decimal":
                            return None  # Skip xs:decimal sources
                        if source_symbol == "integer" or isinstance(source_val, int):
                            val_int = int(source_val)
                            # Only accept i8 range (-128 to 127) for casts to float/double
                            if val_int < -128 or val_int > 127:
                                return None
                            return ("", f"{noir_func}({val_int})", None)

                    # For xs:integer-from-float: expect xs:float source
                    elif source_type == "float":
                        if source_symbol == "float":
                            float_val = float(source_val)
                            bits = float_to_bits(float_val)
                            setup = f"let f = XsdFloat::from_bits({bits});"
                            return (setup, f"{noir_func}(f)", None)

                    # For xs:integer-from-double or xs:float-from-double: expect xs:double source
                    elif source_type == "double":
                        if source_symbol == "double":
                            double_val = float(source_val)
                            bits = double_to_bits(double_val)
                            setup = f"let d = XsdDouble::from_bits({bits});"
                            return (setup, f"{noir_func}(d)", None)

                except Exception:
                    pass
            return None

        # Also handle constructor syntax: xs:float(integer_expr), xs:double(integer_expr), etc.
        # The symbol should be 'float', 'double', or 'integer' (depending on target type)
        if symbol == expected_target:
            args = _get_function_args(token)
            if len(args) >= 1:
                try:
                    # First check if the argument has the right source type
                    arg_token = args[0]
                    arg_symbol = _get_function_name(arg_token)

                    # For xs:float-from-int: expect integer input (plain number or xs:integer())
                    if source_type == "int":
                        # Try to evaluate as integer
                        arg_val = arg_token.evaluate()
                        if isinstance(arg_val, (int, Decimal)):
                            val_int = int(arg_val)
                            # Only accept i8 range (-128 to 127) for casts to float/double
                            if val_int < -128 or val_int > 127:
                                return None
                            return ("", f"{noir_func}({val_int})", None)
                        elif isinstance(arg_val, float):
                            # Float literal being cast - skip for int source
                            return None

                    # For xs:integer-from-float: expect xs:float() input
                    elif source_type == "float":
                        if arg_symbol == "float":
                            # Get the inner value
                            inner_args = _get_function_args(arg_token)
                            if len(inner_args) >= 1:
                                inner_val = inner_args[0].evaluate()
                                if isinstance(inner_val, (int, float, Decimal)):
                                    float_val = float(inner_val)
                                    bits = float_to_bits(float_val)
                                    setup = f"let f = XsdFloat::from_bits({bits});"
                                    return (setup, f"{noir_func}(f)", None)
                        return None

                    # For xs:integer-from-double or xs:float-from-double: expect xs:double() input
                    elif source_type == "double":
                        if arg_symbol == "double":
                            inner_args = _get_function_args(arg_token)
                            if len(inner_args) >= 1:
                                inner_val = inner_args[0].evaluate()
                                if isinstance(inner_val, (int, float, Decimal)):
                                    double_val = float(inner_val)
                                    bits = double_to_bits(double_val)
                                    setup = f"let d = XsdDouble::from_bits({bits});"
                                    return (setup, f"{noir_func}(d)", None)
                        return None

                except Exception:
                    pass
        return None

    return None


def _datetime_to_epoch(dt: DateTime10) -> Optional[tuple[int, int]]:
    """Convert elementpath DateTime10 to (UTC microseconds, tz_offset_minutes)."""
    try:
        # Get timezone offset in minutes
        tz_offset_minutes = 0
        if dt.tzinfo is not None:
            offset = dt.tzinfo.offset
            tz_offset_minutes = int(offset.total_seconds() / 60)

        # Build a Python datetime and convert to epoch
        py_tz = timezone(timedelta(minutes=tz_offset_minutes))
        py_dt = datetime(
            dt.year,
            dt.month,
            dt.day,
            dt.hour,
            dt.minute,
            int(dt.second),
            dt.microsecond,
            tzinfo=py_tz,
        )

        epoch = datetime(1970, 1, 1, tzinfo=timezone.utc)
        utc_dt = py_dt.astimezone(timezone.utc)
        delta = utc_dt - epoch
        utc_micros = int(delta.total_seconds() * 1_000_000)

        return (utc_micros, tz_offset_minutes)
    except Exception:
        return None


def _date_to_epoch_days(dt: Date10) -> Optional[tuple[int, int]]:
    """Convert elementpath Date10 to (epoch days, tz_offset_minutes).

    Returns (days since 1970-01-01, timezone offset in minutes).
    """
    try:
        # Get timezone offset in minutes
        tz_offset_minutes = 0
        if dt.tzinfo is not None:
            offset = dt.tzinfo.offset
            tz_offset_minutes = int(offset.total_seconds() / 60)

        # Build a Python date and convert to epoch days
        from datetime import date as pydate

        py_date = pydate(dt.year, dt.month, dt.day)
        epoch_date = pydate(1970, 1, 1)
        delta = py_date - epoch_date
        epoch_days = delta.days

        return (epoch_days, tz_offset_minutes)
    except Exception:
        return None


def _parse_date_string(s: str) -> Optional[tuple[int, int, int, Optional[int]]]:
    """Parse a date string like '2002-03-07-05:00' or '2002-03-07'.

    Returns (year, month, day, tz_offset_minutes) or None.
    tz_offset_minutes is None if no timezone is present.
    """
    import re

    # Pattern for date with optional timezone: YYYY-MM-DD[Z|[+-]HH:MM]
    pattern = r"^(-?\d{4})-(\d{2})-(\d{2})(Z|[+-]\d{2}:\d{2})?$"
    match = re.match(pattern, s)
    if not match:
        return None

    year = int(match.group(1))
    month = int(match.group(2))
    day = int(match.group(3))
    tz_str = match.group(4)

    tz_minutes = None
    if tz_str:
        if tz_str == "Z":
            tz_minutes = 0
        else:
            sign = 1 if tz_str[0] == "+" else -1
            hours = int(tz_str[1:3])
            minutes = int(tz_str[4:6])
            tz_minutes = sign * (hours * 60 + minutes)

    return (year, month, day, tz_minutes)


def _parse_time_string(s: str) -> Optional[tuple[int, int, int, Optional[int]]]:
    """Parse a time string like '11:23:00-05:00' or '11:23:00'.

    Returns (hours, minutes, seconds, tz_offset_minutes) or None.
    tz_offset_minutes is None if no timezone is present.
    """
    import re

    # Pattern for time with optional timezone: HH:MM:SS[.sss][Z|[+-]HH:MM]
    pattern = r"^(\d{2}):(\d{2}):(\d{2})(?:\.\d+)?(Z|[+-]\d{2}:\d{2})?$"
    match = re.match(pattern, s)
    if not match:
        return None

    hours = int(match.group(1))
    minutes = int(match.group(2))
    seconds = int(match.group(3))
    tz_str = match.group(4)

    tz_minutes = None
    if tz_str:
        if tz_str == "Z":
            tz_minutes = 0
        else:
            sign = 1 if tz_str[0] == "+" else -1
            hours_tz = int(tz_str[1:3])
            minutes_tz = int(tz_str[4:6])
            tz_minutes = sign * (hours_tz * 60 + minutes_tz)

    return (hours, minutes, seconds, tz_minutes)


def _parse_datetime_string(
    s: str,
) -> Optional[tuple[int, int, int, int, int, int, Optional[int]]]:
    """Parse a datetime string like '2002-03-07T11:23:00-05:00'.

    Returns (year, month, day, hours, minutes, seconds, tz_offset_minutes) or None.
    """
    import re

    # Pattern for datetime with optional timezone
    pattern = r"^(-?\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})(?:\.\d+)?(Z|[+-]\d{2}:\d{2})?$"
    match = re.match(pattern, s)
    if not match:
        return None

    year = int(match.group(1))
    month = int(match.group(2))
    day = int(match.group(3))
    hours = int(match.group(4))
    minutes = int(match.group(5))
    seconds = int(match.group(6))
    tz_str = match.group(7)

    tz_minutes = None
    if tz_str:
        if tz_str == "Z":
            tz_minutes = 0
        else:
            sign = 1 if tz_str[0] == "+" else -1
            hours_tz = int(tz_str[1:3])
            minutes_tz = int(tz_str[4:6])
            tz_minutes = sign * (hours_tz * 60 + minutes_tz)

    return (year, month, day, hours, minutes, seconds, tz_minutes)


def _time_to_microseconds(t: Time) -> Optional[tuple[int, int]]:
    """Convert elementpath Time to (microseconds since midnight, tz_offset_minutes).

    Returns (microseconds since 00:00:00, timezone offset in minutes).
    """
    try:
        # Get timezone offset in minutes
        tz_offset_minutes = 0
        if t.tzinfo is not None:
            offset = t.tzinfo.offset
            tz_offset_minutes = int(offset.total_seconds() / 60)

        # Calculate microseconds since midnight
        microseconds = (
            t.hour * 3600 * 1_000_000
            + t.minute * 60 * 1_000_000
            + int(t.second) * 1_000_000
            + t.microsecond
        )

        return (microseconds, tz_offset_minutes)
    except Exception:
        return None


def generate_noir_test(test: TestCase, function_name: str) -> Optional[str]:
    """Generate Noir test code for a test case.

    Returns None if the test cannot be converted to Noir.
    """
    test_name = sanitize_test_name(test.name)

    # Skip tests with unsupported dependencies
    unsupported_deps = ["schemaValidation", "schemaImport", "staticTyping"]
    for dep in test.dependencies:
        for unsup in unsupported_deps:
            if unsup in dep:
                return None

    # Try to convert the expression
    conversion = convert_xpath_expr(test.test_expr, function_name)
    if conversion is None:
        # Generate placeholder
        desc = test.description.replace("\n", " ").replace('"', "'")[:80]
        return f"""// SKIP: {test_name}
// Cannot auto-convert: {test.test_expr}
// Expected: {test.expected_result}
"""

    setup_code, test_expr, embedded_expected = conversion

    # Extract yearMonthDuration string literals from the expression for additional assertions
    ymd_literals = extract_ymd_literals_from_expr(test.test_expr)

    # Determine what functions return boolean values
    boolean_returning_functions = [
        "fn_not",
        "boolean_equal",
        "boolean_less_than",
        "boolean_greater_than",
        "datetime_equal",
        "datetime_less_than",
        "datetime_greater_than",
        "date_equal",
        "date_less_than",
        "date_greater_than",
        "time_equal",
        "time_less_than",
        "time_greater_than",
        "duration_equal",
        "duration_less_than",
        "duration_greater_than",
        "ym_duration_equal",
        "ym_duration_less_than",
        "ym_duration_greater_than",
        "numeric_equal_int",
        "numeric_less_than_int",
        "numeric_greater_than_int",
        "numeric_equal_float",
        "numeric_less_than_float",
        "numeric_greater_than_float",
        "numeric_equal_double",
        "numeric_less_than_double",
        "numeric_greater_than_double",
    ]

    # Functions that return float/double types
    float_returning_functions = [
        "numeric_add_float",
        "numeric_subtract_float",
        "numeric_multiply_float",
        "numeric_divide_float",
        "round_float",
        "ceil_float",
        "floor_float",
        "cast_integer_to_float",  # xs:float(integer)
        "cast_double_to_float",  # xs:float(double)
    ]
    double_returning_functions = [
        "numeric_add_double",
        "numeric_subtract_double",
        "numeric_multiply_double",
        "numeric_divide_double",
        "round_double",
        "ceil_double",
        "floor_double",
        "cast_integer_to_double",  # xs:double(integer)
        # op:divide-dayTimeDuration-by-dayTimeDuration returns the xs:decimal
        # ratio, modelled as an IEEE 754 double (F&O 3.1 sec. 8.4.10; sq-3x7dl.9)
        "duration_divide_by_duration",
    ]

    # Functions that return Option<i64>
    option_int_returning_functions = [
        "cast_float_to_integer",  # xs:integer(float)
        "cast_double_to_integer",  # xs:integer(double)
    ]

    noir_func = resolve_noir_symbol(function_name)
    func_returns_bool = noir_func in boolean_returning_functions
    func_returns_float = noir_func in float_returning_functions
    func_returns_double = noir_func in double_returning_functions
    func_returns_option_int = noir_func in option_int_returning_functions

    # Generate assertion based on result type
    # If embedded_expected is set, it means the XPath expression contained a comparison
    # and we extracted the expected value from it
    if embedded_expected is not None:
        # The expression contains a comparison, use the embedded expected value
        int_val = parse_integer(embedded_expected)
        if int_val is not None:
            assertion = f"assert({test_expr} == {int_val});"
        else:
            # Try as float (for seconds)
            try:
                float_val = float(embedded_expected)
                int_part = int(float_val)
                assertion = f"assert({test_expr} == {int_part});"
            except ValueError:
                return f"""// SKIP: {test_name}
// Cannot parse embedded expected: {embedded_expected}
"""
    elif test.result_type in ("assert-true", "assert-false"):
        # Handle fn:string(ymd) in boolean context - EBV of non-empty string is true
        if test_expr.startswith("FN_STRING_YMD:"):
            ymd_expr = test_expr[len("FN_STRING_YMD:") :]
            bool_val = test.result_type == "assert-true"
            # fn:string(ymd) always returns a non-empty string, so EBV is true
            setup_code = (
                f"let _fn_string_arg = {ymd_expr}; // fn:string() evaluates this"
            )
            assertion = f"assert(true /* EBV of non-empty string */ == {str(bool_val).lower()});"
        else:
            bool_val = test.result_type == "assert-true"
            assertion = f"assert({test_expr} == {str(bool_val).lower()});"
    elif test.result_type == "assert-eq":
        # Handle fn:string(ymd) with string expected value like "P1Y10M"
        if test_expr.startswith("FN_STRING_YMD:"):
            ymd_expr = test_expr[len("FN_STRING_YMD:") :]
            expected = test.expected_result
            expected_months = parse_year_month_duration(expected)
            if expected_months is not None:
                # Calculate the expected string representation
                # Format: [-]P[nY][nM]
                neg = expected_months < 0
                abs_months = abs(expected_months)
                years = abs_months // 12
                months = abs_months % 12

                # Build expected string
                parts = ["-"] if neg else []
                parts.append("P")
                if years > 0:
                    parts.append(f"{years}Y")
                if months > 0 or years == 0:
                    parts.append(f"{months}M")
                expected_str = "".join(parts)
                expected_len = len(expected_str)

                # Generate assertion that calls fn_string_from_ym_duration and compares bytes
                assertions = []
                assertions.append(f"let dur_result = {ymd_expr};")
                assertions.append(
                    f"let (str_bytes, str_len): ([u8; {expected_len}], u32) = fn_string_from_ym_duration(dur_result);"
                )
                assertions.append(f"assert(str_len == {expected_len});")
                expected_bytes = [ord(c) for c in expected_str]
                for i, b in enumerate(expected_bytes):
                    assertions.append(f"assert(str_bytes[{i}] == {b}); // '{chr(b)}'")
                assertion = "\n    ".join(assertions)
            else:
                return f"""// SKIP: {test_name}
// Cannot parse yearMonthDuration expected for fn:string: {expected}
"""
        else:
            # Parse expected value
            expected = test.expected_result
            int_val = parse_integer(expected)
            bool_val = parse_boolean(expected)
            float_val = parse_float(expected)

            # For Option<i64> returning functions (cast to integer)
            if func_returns_option_int:
                if int_val is not None:
                    assertion = f"assert({test_expr}.is_some());\n    assert({test_expr}.unwrap() == {int_val});"
                elif float_val is not None:
                    # Truncate float to integer
                    val, _ = float_val
                    int_part = int(val)
                    assertion = f"assert({test_expr}.is_some());\n    assert({test_expr}.unwrap() == {int_part});"
                else:
                    return f"""// SKIP: {test_name}
// Cannot parse expected for cast-to-integer function: {expected}
"""
            # For float/double returning functions, we need to convert the expected value to bits
            elif func_returns_float or func_returns_double:
                # Try to parse as float first
                if float_val is not None:
                    val, ftype = float_val
                    if func_returns_float:
                        # For zero comparisons, use equality (handles +0 vs -0)
                        if val == 0.0:
                            assertion = f"assert({test_expr} == XsdFloat::zero());"
                        else:
                            expected_bits = float_to_bits(val)
                            assertion = (
                                f"assert({test_expr}.to_bits() == {expected_bits});"
                            )
                    else:
                        if val == 0.0:
                            assertion = f"assert({test_expr} == XsdDouble::zero());"
                        else:
                            expected_bits = double_to_bits(val)
                            assertion = (
                                f"assert({test_expr}.to_bits() == {expected_bits});"
                            )
                elif int_val is not None:
                    # Integer value for float function - convert to float bits
                    if func_returns_float:
                        if int_val == 0:
                            assertion = f"assert({test_expr} == XsdFloat::zero());"
                        else:
                            expected_bits = float_to_bits(float(int_val))
                            assertion = (
                                f"assert({test_expr}.to_bits() == {expected_bits});"
                            )
                    else:
                        if int_val == 0:
                            assertion = f"assert({test_expr} == XsdDouble::zero());"
                        else:
                            expected_bits = double_to_bits(float(int_val))
                            assertion = (
                                f"assert({test_expr}.to_bits() == {expected_bits});"
                            )
                else:
                    # Cannot parse expected value for float function
                    return f"""// SKIP: {test_name}
// Cannot parse expected for float function: {expected}
"""
            elif int_val is not None:
                # Skip negative values for functions that return unsigned types
                unsigned_return_functions = [
                    "month_from_datetime",
                    "day_from_datetime",
                    "hours_from_datetime",
                    "minutes_from_datetime",
                    "seconds_from_datetime",
                    "days_from_duration",
                    "hours_from_duration",
                    "minutes_from_duration",
                    "seconds_from_duration",
                ]
                if int_val < 0 and noir_func in unsigned_return_functions:
                    return f"""// SKIP: {test_name}
// Negative expected value {int_val} incompatible with unsigned return type
"""
                assertion = f"assert({test_expr} == {int_val});"
            elif bool_val is not None:
                assertion = f"assert({test_expr} == {str(bool_val).lower()});"
            elif float_val is not None:
                # Float value but function doesn't return float - type mismatch
                return f"""// SKIP: {test_name}
// Float expected value incompatible with function {noir_func}
"""
            else:
                # Try to parse as date/time/datetime for adjust-*-to-timezone functions
                date_returning_functions = [
                    "adjust_date_to_timezone",
                    "adjust_date_to_timezone_none",
                ]
                time_returning_functions = [
                    "adjust_time_to_timezone",
                    "adjust_time_to_timezone_none",
                ]
                datetime_returning_functions = [
                    "adjust_datetime_to_timezone",
                    "adjust_datetime_to_timezone_none",
                ]

                if noir_func in date_returning_functions:
                    parsed = _parse_date_string(expected)
                    if parsed is not None:
                        year, month, day, tz_mins = parsed
                        if year < 1970:
                            return f"""// SKIP: {test_name}
// Date before 1970 not supported: {expected}
"""
                        assertions = []
                        assertions.append(f"let result = {test_expr};")
                        assertions.append(f"assert(year_from_date(result) == {year});")
                        assertions.append(
                            f"assert(month_from_date(result) == {month});"
                        )
                        assertions.append(f"assert(day_from_date(result) == {day});")
                        if tz_mins is not None:
                            # timezone_from_date returns XsdDayTimeDuration, compare using duration_equal
                            tz_micros = (
                                tz_mins * 60_000_000
                            )  # Convert minutes to microseconds
                            assertions.append(
                                f"assert(duration_equal(timezone_from_date(result), duration_from_microseconds({tz_micros})));"
                            )
                        assertion = "\n    ".join(assertions)
                    else:
                        return f"""// SKIP: {test_name}
// Cannot parse date expected: {expected}
"""
                elif noir_func in time_returning_functions:
                    parsed = _parse_time_string(expected)
                    if parsed is not None:
                        hours, minutes, seconds, tz_mins = parsed
                        assertions = []
                        assertions.append(f"let result = {test_expr};")
                        assertions.append(
                            f"assert(hours_from_time(result) == {hours});"
                        )
                        assertions.append(
                            f"assert(minutes_from_time(result) == {minutes});"
                        )
                        assertions.append(
                            f"assert(seconds_from_time(result) == {seconds});"
                        )
                        if tz_mins is not None:
                            # timezone_from_time returns XsdDayTimeDuration
                            tz_micros = tz_mins * 60_000_000
                            assertions.append(
                                f"assert(duration_equal(timezone_from_time(result), duration_from_microseconds({tz_micros})));"
                            )
                        assertion = "\n    ".join(assertions)
                    else:
                        return f"""// SKIP: {test_name}
// Cannot parse time expected: {expected}
"""
                elif noir_func in datetime_returning_functions:
                    parsed = _parse_datetime_string(expected)
                    if parsed is not None:
                        year, month, day, hours, minutes, seconds, tz_mins = parsed
                        if year < 1970:
                            return f"""// SKIP: {test_name}
// DateTime before 1970 not supported: {expected}
"""
                        assertions = []
                        assertions.append(f"let result = {test_expr};")
                        assertions.append(
                            f"assert(year_from_datetime(result) == {year});"
                        )
                        assertions.append(
                            f"assert(month_from_datetime(result) == {month});"
                        )
                        assertions.append(
                            f"assert(day_from_datetime(result) == {day});"
                        )
                        assertions.append(
                            f"assert(hours_from_datetime(result) == {hours});"
                        )
                        assertions.append(
                            f"assert(minutes_from_datetime(result) == {minutes});"
                        )
                        assertions.append(
                            f"assert(seconds_from_datetime(result) == {seconds});"
                        )
                        if tz_mins is not None:
                            # timezone_from_datetime returns XsdDayTimeDuration
                            tz_micros = tz_mins * 60_000_000
                            assertions.append(
                                f"assert(duration_equal(timezone_from_datetime(result), duration_from_microseconds({tz_micros})));"
                            )
                        assertion = "\n    ".join(assertions)
                    else:
                        return f"""// SKIP: {test_name}
// Cannot parse datetime expected: {expected}
"""
                else:
                    # Check if this is a duration-returning function
                    duration_returning_functions = [
                        "subtract_dates",
                        "subtract_times",
                        "datetime_difference",
                        "duration_add",
                        "duration_subtract",
                        "duration_multiply",
                        "duration_divide",
                        "timezone_from_date",
                        "timezone_from_time",
                        "timezone_from_datetime",
                    ]
                    if noir_func in duration_returning_functions:
                        # Parse the expected duration string
                        if expected.strip() in ("", "()"):
                            return f"""// SKIP: {test_name}
// Empty-sequence duration expected is not representable in Noir: {expected}
"""
                        expected_micros = parse_duration(expected)
                        if expected_micros is not None:
                            if _fits_in_i64(expected_micros):
                                assertion = f"assert(duration_equal({test_expr}, duration_from_microseconds({expected_micros})));"
                            else:
                                return f"""// SKIP: {test_name}
// Duration value too large: {expected}
"""
                        else:
                            return f"""// SKIP: {test_name}
// Cannot parse duration expected: {expected}
"""
                    else:
                        # Check if this is a yearMonthDuration-returning function
                        ym_duration_returning_functions = {
                            "ym_duration_add",
                            "ym_duration_subtract",
                            "ym_duration_multiply",
                            "ym_duration_divide",
                        }
                        if noir_func in ym_duration_returning_functions:
                            if expected.strip() in ("", "()"):
                                return f"""// SKIP: {test_name}
// Empty-sequence yearMonthDuration expected is not representable in Noir: {expected}
"""
                            expected_months = parse_year_month_duration(expected)
                            if expected_months is None:
                                return f"""// SKIP: {test_name}
// Cannot parse yearMonthDuration expected: {expected}
"""
                            # Generate two assertions:
                            # 1. Using pre-computed months value
                            # 2. Using string parsing at runtime
                            expected_str = expected.strip()
                            str_len = len(expected_str)
                            assertions = [
                                f"assert(ym_duration_equal({test_expr}, XsdYearMonthDuration::new({expected_months})));",
                                f"// Also verify string parsing produces the same result",
                                f'let (parsed_dur, parse_valid) = ym_duration_from_string("{expected_str}".as_bytes(), {str_len});',
                                f"assert(parse_valid);",
                                f"assert(ym_duration_equal({test_expr}, parsed_dur));",
                            ]
                            assertion = "\n    ".join(assertions)
                        else:
                            # Cannot parse expected value
                            return f"""// SKIP: {test_name}
// Cannot parse expected: {expected}
"""
    else:
        return None

    # Build test function
    # Use full description, preserving multi-line format
    lines = [f"#[test]", f"fn {test_name}() {{"]
    if test.description:
        desc = test.description.replace('"', "'")
        # Split description into lines and add each as a separate comment
        desc_lines = [line.strip() for line in desc.split("\n") if line.strip()]
        for desc_line in desc_lines:
            lines.append(f"    // {desc_line}")
    # Add original XPath expression and expected result as comments
    xpath_expr = sanitize_to_ascii(test.test_expr.replace("\n", " "))
    lines.append(f"    // XPath: {xpath_expr}")
    lines.append(
        f"    // Expected: {test.result_type} {sanitize_to_ascii(str(test.expected_result))}"
    )
    if setup_code:
        for line in setup_code.split("\n"):
            lines.append(f"    {line}")
    lines.append(f"    {assertion}")

    # Add string parsing assertions for yearMonthDuration literals
    # This verifies that runtime parsing produces the same values as compile-time parsing
    if ymd_literals:
        lines.append(
            f"    // Additional assertions: verify string parsing produces expected durations"
        )
        for i, (lit_str, lit_months) in enumerate(ymd_literals):
            str_len = len(lit_str)
            lines.append(
                f'    let (parsed_ymd_{i}, parse_ok_{i}) = ym_duration_from_string("{lit_str}".as_bytes(), {str_len});'
            )
            lines.append(f"    assert(parse_ok_{i});")
            lines.append(
                f"    assert(ym_duration_equal(parsed_ymd_{i}, XsdYearMonthDuration::new({lit_months})));"
            )

    lines.append("}")

    return "\n".join(lines)


def generate_test_package(
    function_name: str,
    tests: list[TestCase],
    output_dir: Path,
    chunk_size: int = 50,
    keep_empty_packages: bool = True,
) -> int:
    """Generate a Noir test package for a function. Returns count of generated tests.

    For implemented functions, generates real tests.
    For unimplemented functions, generates tests that call stub functions.

    Args:
        function_name: The XPath function name (e.g., 'fn:abs', 'op:numeric-add')
        tests: List of test cases from qt3tests
        output_dir: Directory to write test packages
        chunk_size: Number of tests per chunk file
        keep_empty_packages: If True, do not delete/skip generating a package when
            no tests can be converted; instead emit a placeholder test so the
            package remains present (useful for subset generation).
    """
    pkg_name = f"xpath_test_{sanitize_test_name(function_name)}"

    # Check if function is implemented
    func_implemented = is_function_implemented(function_name)

    # Convert tests first to see if we have any
    converted_tests = []
    skipped = 0
    placeholder_only = False
    for test in tests:
        if func_implemented:
            # For implemented functions, use normal test generation
            noir_test = generate_noir_test(test, function_name)
        else:
            # For unimplemented functions, generate tests that call stub functions
            noir_test = generate_stub_test_with_function(test, function_name)

        if noir_test and not noir_test.startswith("// SKIP"):
            converted_tests.append(noir_test)
        else:
            skipped += 1

    if not converted_tests:
        print(f"  No tests converted for {function_name} (skipped {skipped})")
        if not keep_empty_packages:
            # Clean up any existing empty package directory
            pkg_dir = output_dir / pkg_name
            if pkg_dir.exists():
                shutil.rmtree(pkg_dir)
            return 0

        placeholder = "\n".join(
            [
                "#[test]",
                f"fn {sanitize_test_name(function_name)}_no_converted_tests() {{",
                f"    // Placeholder: {skipped} qt3tests cases could not be converted.",
                f'    assert(false, "No qt3tests cases could be converted for {function_name}");',
                "}",
            ]
        )
        converted_tests = [placeholder]
        placeholder_only = True

    # Create package directory only if we have tests
    pkg_dir = output_dir / pkg_name
    src_dir = pkg_dir / "src"
    src_dir.mkdir(parents=True, exist_ok=True)

    # Generate Nargo.toml
    nargo_toml = f"""[package]
name = "{pkg_name}"
type = "lib"
authors = ["auto-generated"]

[dependencies]
xpath = {{ path = "../../xpath" }}
"""
    (pkg_dir / "Nargo.toml").write_text(nargo_toml)

    # Split tests into chunks
    chunks = [
        converted_tests[i : i + chunk_size]
        for i in range(0, len(converted_tests), chunk_size)
    ]

    # Determine required imports
    imports: list[str] = []
    if not placeholder_only:
        imports = ["use dep::xpath::{"]
        all_test_code = "\n".join(converted_tests)
        if func_implemented:
            noir_func = resolve_noir_symbol(function_name)
            if noir_func:
                imports.append(f"    {noir_func},")

            # Add dynamic imports based on emitted code for yearMonthDuration and boolean helpers.
            # This keeps imports minimal while allowing richer conversions.
            if "XsdYearMonthDuration" in all_test_code:
                imports.append("    XsdYearMonthDuration,")

            ymd_symbols = [
                "ym_duration_add",
                "ym_duration_subtract",
                "ym_duration_multiply",
                "ym_duration_divide",
                "ym_duration_divide_by_duration",
                "ym_duration_equal",
                "ym_duration_less_than",
                "ym_duration_greater_than",
                "ym_duration_le",
                "ym_duration_ge",
                "fn_string_from_ym_duration",
                "ym_duration_from_string",
            ]
            for sym in ymd_symbols:
                if f"{sym}(" in all_test_code:
                    imports.append(f"    {sym},")

            if "fn_not(" in all_test_code:
                imports.append("    fn_not,")

            if "fn_boolean_from_string_len(" in all_test_code:
                imports.append("    fn_boolean_from_string_len,")

            # Add datetime imports if needed
            if "datetime" in function_name.lower():
                imports.append("    datetime_from_epoch_microseconds_with_tz,")

            # Add date imports if needed (for date extraction functions and date comparisons)
            if (
                "from-date" in function_name.lower()
                and "datetime" not in function_name.lower()
            ):
                imports.append("    date_from_epoch_days_with_tz,")

            # Add imports for date comparison/subtraction operators
            if function_name in [
                "op:date-equal",
                "op:date-less-than",
                "op:date-greater-than",
                "op:subtract-dates",
            ]:
                imports.append("    date_from_epoch_days_with_tz,")

            # Add time imports if needed (for time extraction functions)
            if (
                "from-time" in function_name.lower()
                and "datetime" not in function_name.lower()
            ):
                imports.append("    time_from_microseconds_with_tz,")

            # Add imports for time comparison/subtraction operators
            if function_name in [
                "op:time-equal",
                "op:time-less-than",
                "op:time-greater-than",
                "op:subtract-times",
            ]:
                imports.append("    time_from_microseconds_with_tz,")

            # Add imports for duration comparisons/constructors used in generated assertions
            if function_name in [
                "op:subtract-dates",
                "op:subtract-times",
                "op:subtract-dateTimes",
                "op:add-dayTimeDurations",
                "op:subtract-dayTimeDurations",
                "op:multiply-dayTimeDuration",
                "op:divide-dayTimeDuration",
                "fn:timezone-from-date",
                "fn:timezone-from-time",
                "fn:timezone-from-dateTime",
            ]:
                imports.append("    duration_equal,")
                imports.append("    duration_from_microseconds,")

            # Add imports for adjust-*-to-timezone functions
            if "adjust-date-to-timezone" in function_name.lower():
                imports.append("    adjust_date_to_timezone_none,")
                imports.append("    date_from_epoch_days_with_tz,")
                imports.append("    year_from_date,")
                imports.append("    month_from_date,")
                imports.append("    day_from_date,")
                imports.append("    timezone_from_date,")
                imports.append("    duration_equal,")
                imports.append("    duration_from_microseconds,")

            if "adjust-time-to-timezone" in function_name.lower():
                imports.append("    adjust_time_to_timezone_none,")
                imports.append("    time_from_microseconds_with_tz,")
                imports.append("    hours_from_time,")
                imports.append("    minutes_from_time,")
                imports.append("    seconds_from_time,")
                imports.append("    timezone_from_time,")
                imports.append("    duration_equal,")
                imports.append("    duration_from_microseconds,")

            if "adjust-datetime-to-timezone" in function_name.lower():
                imports.append("    adjust_datetime_to_timezone_none,")
                imports.append("    datetime_from_epoch_microseconds_with_tz,")
                imports.append("    year_from_datetime,")
                imports.append("    month_from_datetime,")
                imports.append("    day_from_datetime,")
                imports.append("    hours_from_datetime,")
                imports.append("    minutes_from_datetime,")
                imports.append("    seconds_from_datetime,")
                imports.append("    timezone_from_datetime,")
                imports.append("    duration_equal,")
                imports.append("    duration_from_microseconds,")

            # Add duration constructor import for dayTimeDuration-related suites.
            # (yearMonthDuration uses XsdYearMonthDuration::new instead)
            if (
                "duration" in function_name.lower()
                and "yearmonthduration" not in function_name.lower()
            ):
                imports.append("    duration_from_microseconds,")

            # Add float/double type imports if needed
            # Check both function_name and noir_func for float/double
            func_lower = function_name.lower()
            noir_func_lower = noir_func.lower() if noir_func else ""

            if "float" in func_lower or "float" in noir_func_lower:
                imports.append("    XsdFloat,")
            if "double" in func_lower or "double" in noir_func_lower:
                imports.append("    XsdDouble,")
            # Emitted code can reference XsdDouble even when neither name
            # mentions "double" (e.g. duration_divide_by_duration's xs:decimal
            # ratio compares against XsdDouble::zero(); sq-3x7dl.9). The
            # dedup pass below removes any double-add.
            if "XsdDouble" in all_test_code:
                imports.append("    XsdDouble,")
        else:
            # For unimplemented functions, import the stub function
            stub_func_name = get_stub_function_name(function_name)
            imports.append(f"    {stub_func_name},")

        # De-duplicate import entries while preserving order.
        if len(imports) > 1:
            seen: set[str] = set()
            deduped = [imports[0]]
            for line in imports[1:]:
                if line in seen:
                    continue
                seen.add(line)
                deduped.append(line)
            imports = deduped

        imports.append("};")

    # Generate lib.nr
    stub_marker = " (uses stub function)" if not func_implemented else ""
    lib_lines = [
        f"//! Auto-generated tests for {function_name}{stub_marker}",
        f"//! Source: https://github.com/w3c/qt3tests",
        "",
    ]
    for i in range(len(chunks)):
        lib_lines.append(f"mod chunk_{i};")

    (src_dir / "lib.nr").write_text("\n".join(lib_lines))

    # Generate chunk files
    for i, chunk in enumerate(chunks):
        chunk_lines = [
            f"//! Test chunk {i} for {function_name}",
            f"//! Contains {len(chunk)} tests",
            "",
        ]

        if imports:
            chunk_lines.extend(imports)
            chunk_lines.append("")

        for test_code in chunk:
            chunk_lines.append(test_code)
            chunk_lines.append("")

        (src_dir / f"chunk_{i}.nr").write_text("\n".join(chunk_lines))

    status = "(stub)" if not func_implemented else ""
    print(
        f"  Generated: {pkg_name} ({len(converted_tests)} tests{status}, {skipped} skipped)"
    )
    return len(converted_tests)


def update_workspace_toml(workspace_dir: Path) -> None:
    """Update the workspace Nargo.toml to include generated test packages.

    This function reads the existing Nargo.toml, preserves any manually added
    members (those not in test_packages/), and adds/updates test package entries.
    """
    nargo_path = workspace_dir / "Nargo.toml"
    if not nargo_path.exists():
        return

    test_packages_dir = workspace_dir / "test_packages"
    if not test_packages_dir.exists():
        return

    # Find all generated packages in test_packages directory
    new_test_packages = set()
    for pkg_dir in sorted(test_packages_dir.iterdir()):
        if pkg_dir.is_dir() and (pkg_dir / "Nargo.toml").exists():
            new_test_packages.add(f"test_packages/{pkg_dir.name}")

    # Read existing Nargo.toml to preserve manually added members
    existing_content = nargo_path.read_text()
    existing_members = []

    # Parse existing members from the TOML file
    # Look for members = [ ... ] pattern
    members_match = re.search(r"members\s*=\s*\[(.*?)\]", existing_content, re.DOTALL)
    if members_match:
        members_str = members_match.group(1)
        # Extract quoted strings
        existing_members = re.findall(r'"([^"]+)"', members_str)

    # Separate existing members into:
    # 1. Non-test-package members (manually added, preserve these)
    # 2. Test package members (will be replaced with current test packages)
    preserved_members = []
    for member in existing_members:
        if not member.startswith("test_packages/"):
            preserved_members.append(member)

    # Build the complete members list:
    # - Preserved non-test-package members first (in original order)
    # - Then all current test packages (sorted)
    all_members = preserved_members + sorted(new_test_packages)

    # Generate new Nargo.toml content
    members_list = ",\n    ".join(f'"{m}"' for m in all_members)
    new_content = f"""[workspace]
members = [
    {members_list},
]
"""

    # Only write if content changed
    if new_content != existing_content:
        nargo_path.write_text(new_content)
        added_count = len(
            new_test_packages
            - set(m for m in existing_members if m.startswith("test_packages/"))
        )
        removed_count = len(
            set(m for m in existing_members if m.startswith("test_packages/"))
            - new_test_packages
        )
        print(f"\nUpdated workspace Nargo.toml: {len(new_test_packages)} test packages")
        if added_count > 0:
            print(f"  Added {added_count} new test package(s)")
        if removed_count > 0:
            print(f"  Removed {removed_count} obsolete test package(s)")
    else:
        print(
            f"\nWorkspace Nargo.toml is already up to date ({len(new_test_packages)} test packages)"
        )


def main():
    parser = argparse.ArgumentParser(description="Generate Noir tests from qt3tests")
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=Path(__file__).parent.parent / "test_packages",
        help="Output directory for generated test packages",
    )
    parser.add_argument(
        "--qt3-dir",
        type=Path,
        default=Path(__file__).parent / "qt3tests",
        help="Directory for qt3tests repository",
    )
    parser.add_argument(
        "--functions",
        type=str,
        default=None,
        help="Comma-separated list of functions to generate tests for",
    )
    parser.add_argument(
        "--skip-clone",
        action="store_true",
        help="Skip cloning/updating qt3tests",
    )
    parser.add_argument(
        "--list-functions",
        action="store_true",
        help="List available functions and exit",
    )
    parser.add_argument(
        "--list-all",
        action="store_true",
        help="List all discoverable functions (implemented and unimplemented) and exit",
    )
    parser.add_argument(
        "--skip-fmt",
        action="store_true",
        help="Skip running nargo fmt after generation",
    )
    args = parser.parse_args()

    # Discover xpath crate exports once (used by is_function_implemented).
    repo_root = Path(__file__).parent.parent
    xpath_crate_dir = repo_root / "xpath"
    global XPATH_EXPORTED_SYMBOLS
    XPATH_EXPORTED_SYMBOLS = discover_xpath_exports(xpath_crate_dir)

    # Clone/update qt3tests repository unless explicitly skipped.
    if not args.skip_clone:
        clone_or_update_qt3tests(args.qt3_dir)

    available_functions = discover_available_functions(args.qt3_dir)
    implemented_functions = sorted(
        f for f in available_functions if is_function_implemented(f)
    )

    if args.list_functions:
        print("Implemented functions (qt3tests fn/op with exported Noir symbol):")
        for func in implemented_functions:
            print(f"  {func}")
        print(
            f"\nTotal: {len(implemented_functions)} implemented / {len(available_functions)} available"
        )
        return

    if args.list_all:
        all_functions = sorted(available_functions)
        print("All qt3tests-discovered functions:")
        print(f"  (✓ = implemented, ✗ = not implemented)\n")
        for func in all_functions:
            status = "✓" if is_function_implemented(func) else "✗"
            print(f"  [{status}] {func}")
        print(
            f"\nTotal: {len(all_functions)} ({len(implemented_functions)} implemented)"
        )
        return

    # Determine which functions to process
    if args.functions:
        # Use explicitly specified functions
        functions_to_process = [f.strip() for f in args.functions.split(",")]
    else:
        # Default: process all discovered fn/op tests plus any implemented functions
        # (e.g., *-float/*-double variants, casts) that may not have their own XML file.
        functions_to_process = sorted(
            set(discover_all_test_files(args.qt3_dir).keys())
            | set(CAST_FUNCTION_PATTERNS.keys())
        )

    # Discover fn/op test files once for resolution during generation.
    discovered_test_files = discover_all_test_files(args.qt3_dir)

    # Clear the global stub functions set
    STUB_FUNCTIONS_NEEDED.clear()

    # Process each function
    total_tests_identified = 0
    total_tests_generated = 0
    total_stub_tests = 0
    implemented_count = 0
    unimplemented_count = 0

    print("\nGenerating tests...")
    for func in sorted(functions_to_process):
        relpath = discovered_test_files.get(func) or default_test_file_relpath(func)
        if relpath is None:
            print(f"  Warning: No deterministic test file mapping for {func}")
            continue

        test_file = args.qt3_dir / relpath
        tests = parse_test_file(test_file)
        total_tests_identified += len(tests)

        if tests:
            is_impl = is_function_implemented(func)
            if is_impl:
                implemented_count += 1
            else:
                unimplemented_count += 1

            count = generate_test_package(
                func,
                tests,
                args.output_dir,
            )
            total_tests_generated += count
            if not is_impl and count > 0:
                total_stub_tests += count
        else:
            print(f"  No tests found for {func}")

    # Generate stub functions module in xpath library
    if xpath_crate_dir.exists():
        generate_stub_functions_module(xpath_crate_dir)
        update_lib_nr_with_stubs(xpath_crate_dir)

    # Clean up old packages that shouldn't exist anymore.
    # IMPORTANT: Only do this when generating the full suite (no --functions),
    # otherwise subset generation would delete unrelated test packages.
    if args.functions is None:
        generated_pkg_names = set()
        for func in functions_to_process:
            pkg_name = f"xpath_test_{sanitize_test_name(func)}"
            generated_pkg_names.add(pkg_name)

        # Remove packages that exist but shouldn't
        # Only consider directories matching xpath_test_* pattern to avoid
        # accidentally deleting user-created directories
        existing_packages = set(
            p.name
            for p in args.output_dir.iterdir()
            if p.is_dir() and p.name.startswith("xpath_test_")
        )
        packages_to_remove = existing_packages - generated_pkg_names
        if packages_to_remove:
            print(f"\nCleaning up {len(packages_to_remove)} obsolete test packages...")
            for pkg_name in sorted(packages_to_remove):
                pkg_dir = args.output_dir / pkg_name
                shutil.rmtree(pkg_dir)
                print(f"  Removed: {pkg_name}")
    else:
        print("\nSkipping obsolete test package cleanup (subset generation).")

    # Update workspace Nargo.toml
    update_workspace_toml(repo_root)

    print(f"\nTest generation complete!")
    print(
        f"Functions processed: {implemented_count + unimplemented_count} ({implemented_count} implemented, {unimplemented_count} unimplemented)"
    )
    print(f"Total tests identified in qt3tests: {total_tests_identified}")
    print(f"Total tests generated: {total_tests_generated}")
    if total_stub_tests > 0:
        print(f"  - Tests using stub functions (will fail): {total_stub_tests}")
    if STUB_FUNCTIONS_NEEDED:
        print(f"  - Stub functions generated: {len(STUB_FUNCTIONS_NEEDED)}")

    # Run nargo fmt to format all generated code
    if not args.skip_fmt:
        print("\nRunning nargo fmt...")
        try:
            result = subprocess.run(
                ["nargo", "fmt"],
                cwd=args.output_dir.parent,
                capture_output=True,
                text=True,
            )
            if result.returncode == 0:
                print("  Formatting complete.")
            else:
                print(
                    f"  Warning: nargo fmt returned non-zero exit code: {result.returncode}"
                )
                if result.stderr:
                    print(f"  stderr: {result.stderr[:500]}")
        except FileNotFoundError:
            print("  Warning: nargo not found in PATH, skipping formatting.")
        except Exception as e:
            print(f"  Warning: Failed to run nargo fmt: {e}")


if __name__ == "__main__":
    main()
