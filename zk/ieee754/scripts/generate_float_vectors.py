#!/usr/bin/env python3
"""Generate public-consumer Noir tests for the generated float types.

The reference model is deliberately self-contained: it uses exact rational
arithmetic and round-to-nearest-even packing so it can produce vectors for
f16, f32, f64, and f128 without NumPy or MPFR. If an IBM FPgen cache is
available, a bounded subset of its public .fptest arithmetic cases is folded
into the generated Noir tests as an additional corpus source.
"""

from __future__ import annotations

import argparse
import math
import random
import re
import urllib.error
import urllib.request
from dataclasses import dataclass
from fractions import Fraction
from pathlib import Path


FPGEN_BASE_URL = "https://raw.githubusercontent.com/sergev/ieee754-test-suite/master"
DEFAULT_FPGEN_FILES = [
    "Add-Cancellation-And-Subnorm-Result.fptest",
    "Add-Cancellation.fptest",
    "Add-Shift.fptest",
    "Corner-Rounding.fptest",
    "Divide-Divide-By-Zero-Exception.fptest",
    "Divide-Trailing-Zeros.fptest",
    "Overflow.fptest",
    "Rounding.fptest",
    "Sticky-Bit-Calculation.fptest",
    "Underflow.fptest",
    "Vicinity-Of-Rounding-Boundaries.fptest",
]


@dataclass(frozen=True)
class FloatFormat:
    name: str
    total_bits: int
    exponent_bits: int
    mantissa_bits: int
    noir_uint: str

    @property
    def bias(self) -> int:
        return (1 << (self.exponent_bits - 1)) - 1

    @property
    def max_exponent(self) -> int:
        return (1 << self.exponent_bits) - 1

    @property
    def sign_mask(self) -> int:
        return 1 << (self.total_bits - 1)

    @property
    def exponent_mask(self) -> int:
        return self.max_exponent << self.mantissa_bits

    @property
    def mantissa_mask(self) -> int:
        return (1 << self.mantissa_bits) - 1

    @property
    def hidden_bit(self) -> int:
        return 1 << self.mantissa_bits


FORMATS = [
    FloatFormat("f16", 16, 5, 10, "u16"),
    FloatFormat("f32", 32, 8, 23, "u32"),
    FloatFormat("f64", 64, 11, 52, "u64"),
    FloatFormat("f128", 128, 15, 112, "u128"),
]

FORMAT_BY_BITS = {fmt.total_bits: fmt for fmt in FORMATS}
FORMAT_BY_NAME = {fmt.name: fmt for fmt in FORMATS}
OP_SYMBOLS = {"add": "+", "sub": "-", "mul": "*", "div": "/"}
FPGEN_OPS = {"+": "add", "-": "sub", "*": "mul", "/": "div"}


@dataclass(frozen=True)
class Vector:
    fmt: FloatFormat
    op: str
    left: int
    right: int
    expected: int
    source: str

    def key(self) -> tuple[str, str, int, int, int]:
        return (self.fmt.name, self.op, self.left, self.right, self.expected)


def pow2(exponent: int) -> Fraction:
    if exponent >= 0:
        return Fraction(1 << exponent, 1)
    return Fraction(1, 1 << -exponent)


def scale_by_power_of_two(value: Fraction, exponent: int) -> Fraction:
    if exponent >= 0:
        return Fraction(value.numerator << exponent, value.denominator)
    return Fraction(value.numerator, value.denominator << -exponent)


def round_nearest_even(value: Fraction) -> int:
    quotient, remainder = divmod(value.numerator, value.denominator)
    twice_remainder = remainder * 2

    if twice_remainder > value.denominator:
        return quotient + 1
    if twice_remainder < value.denominator:
        return quotient
    if quotient & 1:
        return quotient + 1
    return quotient


def floor_log2(value: Fraction) -> int:
    exponent = value.numerator.bit_length() - value.denominator.bit_length()

    while value < pow2(exponent):
        exponent -= 1
    while value >= pow2(exponent + 1):
        exponent += 1

    return exponent


def canonical_nan(fmt: FloatFormat) -> int:
    quiet_bit = 1 << (fmt.mantissa_bits - 1)
    return fmt.exponent_mask | quiet_bit


def infinity(fmt: FloatFormat, sign: bool) -> int:
    return (fmt.sign_mask if sign else 0) | fmt.exponent_mask


def zero(fmt: FloatFormat, sign: bool) -> int:
    return fmt.sign_mask if sign else 0


def split_bits(fmt: FloatFormat, bits: int) -> tuple[bool, int, int]:
    bits &= (1 << fmt.total_bits) - 1
    sign = (bits & fmt.sign_mask) != 0
    exponent = (bits >> fmt.mantissa_bits) & fmt.max_exponent
    mantissa = bits & fmt.mantissa_mask
    return sign, exponent, mantissa


def is_nan(fmt: FloatFormat, bits: int) -> bool:
    _sign, exponent, mantissa = split_bits(fmt, bits)
    return exponent == fmt.max_exponent and mantissa != 0


def is_infinite(fmt: FloatFormat, bits: int) -> bool:
    _sign, exponent, mantissa = split_bits(fmt, bits)
    return exponent == fmt.max_exponent and mantissa == 0


def is_zero(fmt: FloatFormat, bits: int) -> bool:
    _sign, exponent, mantissa = split_bits(fmt, bits)
    return exponent == 0 and mantissa == 0


def finite_fraction(fmt: FloatFormat, bits: int) -> tuple[bool, Fraction]:
    sign, exponent, mantissa = split_bits(fmt, bits)

    if exponent == 0:
        significand = mantissa
        value_exponent = 1 - fmt.bias - fmt.mantissa_bits
    else:
        significand = fmt.hidden_bit + mantissa
        value_exponent = exponent - fmt.bias - fmt.mantissa_bits

    return sign, scale_by_power_of_two(Fraction(significand, 1), value_exponent)


def pack_finite(fmt: FloatFormat, sign: bool, magnitude: Fraction) -> int:
    if magnitude == 0:
        return zero(fmt, sign)

    min_normal = pow2(1 - fmt.bias)

    if magnitude < min_normal:
        scaled = scale_by_power_of_two(magnitude, fmt.bias + fmt.mantissa_bits - 1)
        mantissa = round_nearest_even(scaled)

        if mantissa == 0:
            return zero(fmt, sign)
        if mantissa >= fmt.hidden_bit:
            return (fmt.sign_mask if sign else 0) | (1 << fmt.mantissa_bits)
        return (fmt.sign_mask if sign else 0) | mantissa

    exponent = floor_log2(magnitude)
    scaled = scale_by_power_of_two(magnitude, fmt.mantissa_bits - exponent)
    significand = round_nearest_even(scaled)

    if significand >= (fmt.hidden_bit << 1):
        significand >>= 1
        exponent += 1

    exponent_field = exponent + fmt.bias
    if exponent_field >= fmt.max_exponent:
        return infinity(fmt, sign)
    if exponent_field <= 0:
        scaled = scale_by_power_of_two(magnitude, fmt.bias + fmt.mantissa_bits - 1)
        mantissa = round_nearest_even(scaled)
        if mantissa == 0:
            return zero(fmt, sign)
        if mantissa >= fmt.hidden_bit:
            return (fmt.sign_mask if sign else 0) | (1 << fmt.mantissa_bits)
        return (fmt.sign_mask if sign else 0) | mantissa

    return (fmt.sign_mask if sign else 0) | (exponent_field << fmt.mantissa_bits) | (significand - fmt.hidden_bit)


def reference_op(fmt: FloatFormat, op: str, left: int, right: int) -> int:
    left_sign, _left_exp, _left_mant = split_bits(fmt, left)
    right_sign, _right_exp, _right_mant = split_bits(fmt, right)
    result_sign = left_sign ^ right_sign

    if op == "sub":
        return reference_op(fmt, "add", left, right ^ fmt.sign_mask)

    if is_nan(fmt, left) or is_nan(fmt, right):
        return canonical_nan(fmt)

    if op == "add":
        if is_infinite(fmt, left) and is_infinite(fmt, right) and left_sign != right_sign:
            return canonical_nan(fmt)
        if is_infinite(fmt, left):
            return infinity(fmt, left_sign)
        if is_infinite(fmt, right):
            return infinity(fmt, right_sign)
        if is_zero(fmt, left) and is_zero(fmt, right):
            return zero(fmt, left_sign and right_sign)

        left_value_sign, left_value = finite_fraction(fmt, left)
        right_value_sign, right_value = finite_fraction(fmt, right)
        exact = (-left_value if left_value_sign else left_value) + (-right_value if right_value_sign else right_value)

        if exact < 0:
            return pack_finite(fmt, True, -exact)
        return pack_finite(fmt, False, exact)

    if op == "mul":
        if (is_infinite(fmt, left) and is_zero(fmt, right)) or (is_infinite(fmt, right) and is_zero(fmt, left)):
            return canonical_nan(fmt)
        if is_infinite(fmt, left) or is_infinite(fmt, right):
            return infinity(fmt, result_sign)
        if is_zero(fmt, left) or is_zero(fmt, right):
            return zero(fmt, result_sign)

        _left_value_sign, left_value = finite_fraction(fmt, left)
        _right_value_sign, right_value = finite_fraction(fmt, right)
        return pack_finite(fmt, result_sign, left_value * right_value)

    if op == "div":
        if (is_infinite(fmt, left) and is_infinite(fmt, right)) or (is_zero(fmt, left) and is_zero(fmt, right)):
            return canonical_nan(fmt)
        if is_infinite(fmt, left):
            return infinity(fmt, result_sign)
        if is_infinite(fmt, right):
            return zero(fmt, result_sign)
        if is_zero(fmt, left):
            return zero(fmt, result_sign)
        if is_zero(fmt, right):
            return infinity(fmt, result_sign)

        _left_value_sign, left_value = finite_fraction(fmt, left)
        _right_value_sign, right_value = finite_fraction(fmt, right)
        return pack_finite(fmt, result_sign, left_value / right_value)

    raise ValueError(f"unsupported operation: {op}")


def bits_for_fraction(fmt: FloatFormat, value: Fraction) -> int:
    if value < 0:
        return pack_finite(fmt, True, -value)
    return pack_finite(fmt, False, value)


COMPARE_OPS = ["eq", "ne", "lt", "le", "gt", "ge"]
ROUND_OPS = ["floor", "ceil", "trunc", "round_ties_even"]
CAST_OPS = ["to_u64", "to_i64"]


def signed_extended_value(fmt: FloatFormat, bits: int):
    """Numeric value of a non-NaN float as Fraction or +/-inf float."""
    sign, exponent, _mantissa = split_bits(fmt, bits)

    if is_infinite(fmt, bits):
        return float("-inf") if sign else float("inf")

    value_sign, magnitude = finite_fraction(fmt, bits)
    return -magnitude if value_sign else magnitude


def reference_compare(fmt: FloatFormat, op: str, left: int, right: int) -> bool:
    if is_nan(fmt, left) or is_nan(fmt, right):
        return op == "ne"

    a = signed_extended_value(fmt, left)
    b = signed_extended_value(fmt, right)

    if op == "eq":
        return a == b
    if op == "ne":
        return a != b
    if op == "lt":
        return a < b
    if op == "le":
        return a <= b
    if op == "gt":
        return a > b
    if op == "ge":
        return a >= b
    raise ValueError(f"unsupported comparison: {op}")


def reference_round_integral(fmt: FloatFormat, op: str, bits: int) -> int:
    if is_nan(fmt, bits):
        return canonical_nan(fmt)
    if is_infinite(fmt, bits) or is_zero(fmt, bits):
        return bits

    sign, magnitude = finite_fraction(fmt, bits)
    value = -magnitude if sign else magnitude

    if op == "floor":
        result = math.floor(value)
    elif op == "ceil":
        result = math.ceil(value)
    elif op == "trunc":
        result = int(value)
    elif op == "round_ties_even":
        lower = math.floor(value)
        fraction = value - lower
        if fraction > Fraction(1, 2):
            result = lower + 1
        elif fraction < Fraction(1, 2):
            result = lower
        elif lower % 2 == 0:
            result = lower
        else:
            result = lower + 1
    else:
        raise ValueError(f"unsupported rounding op: {op}")

    if result == 0:
        return zero(fmt, sign)
    return pack_finite(fmt, result < 0, Fraction(abs(result)))


def reference_sqrt(fmt: FloatFormat, bits: int) -> int:
    if is_nan(fmt, bits):
        return canonical_nan(fmt)
    if is_zero(fmt, bits):
        return bits

    sign, _exponent, _mantissa = split_bits(fmt, bits)
    if sign:
        return canonical_nan(fmt)
    if is_infinite(fmt, bits):
        return bits

    _value_sign, magnitude = finite_fraction(fmt, bits)
    exponent = floor_log2(magnitude) // 2  # sqrt(magnitude) in [2^e, 2^(e+1))
    scaled = scale_by_power_of_two(magnitude, 2 * (fmt.mantissa_bits - exponent))
    numerator, denominator = scaled.numerator, scaled.denominator

    # floor(sqrt(scaled)) with exact integer arithmetic.
    root = math.isqrt(numerator // denominator)
    while (root + 1) * (root + 1) * denominator <= numerator:
        root += 1
    while root * root * denominator > numerator:
        root -= 1

    # Round to nearest, ties to even: compare scaled against (root + 1/2)^2.
    tie_compare = 4 * numerator - ((2 * root + 1) ** 2) * denominator
    if tie_compare > 0:
        rounded = root + 1
    elif tie_compare < 0:
        rounded = root
    elif root % 2 == 0:
        rounded = root
    else:
        rounded = root + 1

    if rounded == fmt.hidden_bit << 1:
        rounded = fmt.hidden_bit
        exponent += 1

    exponent_field = exponent + fmt.bias
    assert 0 < exponent_field < fmt.max_exponent, "sqrt cannot overflow or go subnormal"
    return (exponent_field << fmt.mantissa_bits) | (rounded - fmt.hidden_bit)


def reference_to_int(fmt: FloatFormat, op: str, bits: int) -> tuple[bool, int]:
    """XPath F&O cast semantics: trunc toward zero, then range check."""
    if is_nan(fmt, bits) or is_infinite(fmt, bits):
        return (False, 0)

    sign, magnitude = finite_fraction(fmt, bits)
    value = -magnitude if sign else magnitude
    truncated = int(value)

    if op == "to_u64":
        return (0 <= truncated <= (1 << 64) - 1, truncated)
    if op == "to_i64":
        return (-(1 << 63) <= truncated <= (1 << 63) - 1, truncated)
    raise ValueError(f"unsupported cast: {op}")


@dataclass(frozen=True)
class CompareVector:
    fmt: FloatFormat
    op: str
    left: int
    right: int
    expected: bool
    source: str

    def key(self):
        return ("cmp", self.fmt.name, self.op, self.left, self.right)


@dataclass(frozen=True)
class UnaryVector:
    fmt: FloatFormat
    op: str
    input: int
    expected: int
    source: str

    def key(self):
        return ("unary", self.fmt.name, self.op, self.input)


@dataclass(frozen=True)
class CastVector:
    fmt: FloatFormat
    op: str
    input: int
    valid: bool
    expected: int
    source: str

    def key(self):
        return ("cast", self.fmt.name, self.op, self.input)


def compare_pairs(fmt: FloatFormat) -> list[tuple[int, int]]:
    v = interesting_values(fmt)
    neg = lambda bits: bits ^ fmt.sign_mask

    return [
        (v["one"], v["two"]),
        (v["two"], v["one"]),
        (v["one"], v["one"]),
        (v["one"], v["one_next"]),
        (v["neg_one"], v["one"]),
        (v["one"], v["neg_one"]),
        (v["neg_one"], neg(v["two"])),
        (neg(v["two"]), v["neg_one"]),
        (v["pos_zero"], v["neg_zero"]),
        (v["neg_zero"], v["pos_zero"]),
        (v["pos_zero"], v["pos_zero"]),
        (v["neg_zero"], v["min_sub"]),
        (v["min_sub"], v["two_min_sub"]),
        (v["max_sub"], v["min_norm"]),
        (v["max_finite"], v["pos_inf"]),
        (v["pos_inf"], v["pos_inf"]),
        (v["neg_inf"], v["pos_inf"]),
        (v["neg_inf"], v["neg_inf"]),
        (v["nan"], v["nan"]),
        (v["nan"], v["one"]),
        (v["one"], v["nan"]),
        (v["nan"], v["pos_inf"]),
        (v["neg_zero"], v["nan"]),
    ]


def round_inputs(fmt: FloatFormat) -> list[int]:
    v = interesting_values(fmt)
    neg = lambda bits: bits ^ fmt.sign_mask
    half_ulp_below_max_int = bits_for_fraction(fmt, Fraction((fmt.hidden_bit << 1) - 1, 2))

    return [
        v["pos_zero"],
        v["neg_zero"],
        v["one"],
        v["neg_one"],
        v["one_next"],
        neg(v["one_next"]),
        v["half"],
        neg(v["half"]),
        v["one_point_five"],
        neg(v["one_point_five"]),
        bits_for_fraction(fmt, Fraction(5, 2)),
        bits_for_fraction(fmt, Fraction(-5, 2)),
        bits_for_fraction(fmt, Fraction(7, 2)),
        bits_for_fraction(fmt, Fraction(1, 4)),
        bits_for_fraction(fmt, Fraction(-1, 4)),
        v["min_sub"],
        neg(v["min_sub"]),
        v["max_sub"],
        neg(v["max_sub"]),
        v["min_norm"],
        half_ulp_below_max_int,
        neg(half_ulp_below_max_int),
        bits_for_fraction(fmt, Fraction(fmt.hidden_bit)),
        v["max_finite"],
        v["pos_inf"],
        v["neg_inf"],
        v["nan"],
    ]


def sqrt_inputs(fmt: FloatFormat) -> list[int]:
    v = interesting_values(fmt)

    return [
        v["pos_zero"],
        v["neg_zero"],
        v["one"],
        v["two"],
        v["three"],
        bits_for_fraction(fmt, Fraction(4)),
        bits_for_fraction(fmt, Fraction(9)),
        v["half"],
        bits_for_fraction(fmt, Fraction(1, 4)),
        v["one_point_five"],
        v["one_next"],
        v["min_sub"],
        v["two_min_sub"],
        v["max_sub"],
        v["min_norm"],
        v["max_finite"],
        v["pos_inf"],
        v["neg_inf"],
        v["neg_one"],
        v["nan"],
    ]


def cast_inputs(fmt: FloatFormat) -> list[int]:
    v = interesting_values(fmt)
    neg = lambda bits: bits ^ fmt.sign_mask
    candidates = [
        v["pos_zero"],
        v["neg_zero"],
        v["one"],
        v["neg_one"],
        v["half"],
        neg(v["half"]),
        v["one_point_five"],
        neg(v["one_point_five"]),
        v["min_sub"],
        neg(v["min_sub"]),
        v["max_finite"],
        v["pos_inf"],
        v["neg_inf"],
        v["nan"],
        bits_for_fraction(fmt, Fraction(1 << 63)),
        bits_for_fraction(fmt, Fraction(-(1 << 63))),
        bits_for_fraction(fmt, Fraction((1 << 64))),
        bits_for_fraction(fmt, Fraction((1 << 63) - 1)),
        bits_for_fraction(fmt, Fraction(-(1 << 63) - 1)),
        bits_for_fraction(fmt, Fraction((1 << 64) - 1)),
    ]

    return candidates


def new_op_vectors(fmt: FloatFormat, per_op: int, seed: int):
    rng = random.Random(seed + fmt.total_bits + 7541)
    compares: list[CompareVector] = []
    unaries: list[UnaryVector] = []
    casts: list[CastVector] = []

    pairs = compare_pairs(fmt)
    random_pairs = [
        (random_finite_bits(fmt, rng), random_finite_bits(fmt, rng)) for _ in range(per_op)
    ]
    for op in COMPARE_OPS:
        for index, (left, right) in enumerate(pairs):
            compares.append(CompareVector(fmt, op, left, right, reference_compare(fmt, op, left, right), f"curated:{index}"))
        for index, (left, right) in enumerate(random_pairs):
            compares.append(CompareVector(fmt, op, left, right, reference_compare(fmt, op, left, right), f"random:{seed}:{index}"))

    round_random = [random_finite_bits(fmt, rng) for _ in range(per_op)]
    for op in ROUND_OPS:
        for index, bits in enumerate(round_inputs(fmt)):
            unaries.append(UnaryVector(fmt, op, bits, reference_round_integral(fmt, op, bits), f"curated:{index}"))
        for index, bits in enumerate(round_random):
            unaries.append(UnaryVector(fmt, op, bits, reference_round_integral(fmt, op, bits), f"random:{seed}:{index}"))

    for index, bits in enumerate(sqrt_inputs(fmt)):
        unaries.append(UnaryVector(fmt, "sqrt", bits, reference_sqrt(fmt, bits), f"curated:{index}"))
    for index in range(per_op):
        bits = random_finite_bits(fmt, rng)
        unaries.append(UnaryVector(fmt, "sqrt", bits, reference_sqrt(fmt, bits), f"random:{seed}:{index}"))

    for op in CAST_OPS:
        for index, bits in enumerate(cast_inputs(fmt)):
            valid, expected = reference_to_int(fmt, op, bits)
            casts.append(CastVector(fmt, op, bits, valid, expected, f"curated:{index}"))
        emitted = 0
        attempt = 0
        while emitted < per_op and attempt < per_op * 20:
            attempt += 1
            bits = random_finite_bits(fmt, rng)
            valid, expected = reference_to_int(fmt, op, bits)
            if not valid:
                continue
            casts.append(CastVector(fmt, op, bits, valid, expected, f"random:{seed}:{emitted}"))
            emitted += 1

    return compares, unaries, casts


def dedupe_keyed(vectors):
    seen = set()
    unique = []
    for vector in vectors:
        key = vector.key()
        if key not in seen:
            unique.append(vector)
            seen.add(key)
    return unique


def render_i64_literal(value: int) -> str:
    if value == -(1 << 63):
        return "-9223372036854775807 - 1"
    return str(value)


def render_new_op_tests(lines: list[str], compares, unaries, casts) -> None:
    compare_groups: dict[tuple[str, str], list[CompareVector]] = {}
    for vector in compares:
        compare_groups.setdefault((vector.fmt.name, vector.op), []).append(vector)

    for (fmt_name, op), group in compare_groups.items():
        fmt = FORMAT_BY_NAME[fmt_name]
        lines.append("#[test]")
        lines.append(f"fn generated_{fmt_name}_{op}_vectors_match_reference() {{")
        for vector in group:
            lines.append(f"    // {vector.source}")
            lines.append(
                f"    assert_eq({fmt_name}::new({literal(fmt, vector.left)}).{op}("
                f"{fmt_name}::new({literal(fmt, vector.right)})), {str(vector.expected).lower()});"
            )
        lines.append("}")
        lines.append("")

    unary_groups: dict[tuple[str, str], list[UnaryVector]] = {}
    for vector in unaries:
        unary_groups.setdefault((vector.fmt.name, vector.op), []).append(vector)

    for (fmt_name, op), group in unary_groups.items():
        fmt = FORMAT_BY_NAME[fmt_name]
        lines.append("#[test]")
        lines.append(f"fn generated_{fmt_name}_{op}_vectors_match_reference() {{")
        for vector in group:
            lines.append(f"    // {vector.source}")
            lines.append(
                f"    assert_eq({fmt_name}::new({literal(fmt, vector.input)}).{op}().bits(), "
                f"{literal(fmt, vector.expected)});"
            )
        lines.append("}")
        lines.append("")

    cast_groups: dict[tuple[str, str], list[CastVector]] = {}
    for vector in casts:
        cast_groups.setdefault((vector.fmt.name, vector.op), []).append(vector)

    for (fmt_name, op), group in cast_groups.items():
        fmt = FORMAT_BY_NAME[fmt_name]
        valid_vectors = [vector for vector in group if vector.valid]
        invalid_vectors = [vector for vector in group if not vector.valid]

        if valid_vectors:
            lines.append("#[test]")
            lines.append(f"fn generated_{fmt_name}_{op}_vectors_match_reference() {{")
            for vector in valid_vectors:
                if op == "to_u64":
                    expected_literal = f"{vector.expected} as u64"
                else:
                    expected_literal = render_i64_literal(vector.expected)
                lines.append(f"    // {vector.source}")
                lines.append(
                    f"    assert_eq({fmt_name}::new({literal(fmt, vector.input)}).{op}(), {expected_literal});"
                )
            lines.append("}")
            lines.append("")

        for index, vector in enumerate(invalid_vectors):
            lines.append("// out-of-range or non-numeric cast must fail to prove")
            lines.append(f"// {vector.source}")
            lines.append("#[test(should_fail)]")
            lines.append(f"fn generated_{fmt_name}_{op}_invalid_{index}() {{")
            lines.append(f"    let _ = {fmt_name}::new({literal(fmt, vector.input)}).{op}();")
            lines.append("}")
            lines.append("")


def interesting_values(fmt: FloatFormat) -> dict[str, int]:
    one_exp = fmt.bias << fmt.mantissa_bits
    two_exp = (fmt.bias + 1) << fmt.mantissa_bits
    half_exp = (fmt.bias - 1) << fmt.mantissa_bits
    max_finite = ((fmt.max_exponent - 1) << fmt.mantissa_bits) | fmt.mantissa_mask

    return {
        "pos_zero": zero(fmt, False),
        "neg_zero": zero(fmt, True),
        "min_sub": 1,
        "two_min_sub": 2,
        "max_sub": fmt.mantissa_mask,
        "min_norm": 1 << fmt.mantissa_bits,
        "one": one_exp,
        "neg_one": fmt.sign_mask | one_exp,
        "one_next": one_exp + 1,
        "half": half_exp,
        "one_point_five": one_exp | (fmt.hidden_bit >> 1),
        "two": two_exp,
        "three": bits_for_fraction(fmt, Fraction(3, 1)),
        "max_finite": max_finite,
        "pos_inf": infinity(fmt, False),
        "neg_inf": infinity(fmt, True),
        "nan": canonical_nan(fmt),
    }


def curated_vectors(fmt: FloatFormat) -> list[Vector]:
    v = interesting_values(fmt)
    pairs_by_op = {
        "add": [
            (v["one"], v["two"]),
            (v["one"], v["half"]),
            (v["one"], v["min_sub"]),
            (v["min_sub"], v["min_sub"]),
            (v["max_sub"], v["min_sub"]),
            (v["pos_zero"], v["neg_zero"]),
            (v["neg_zero"], v["neg_zero"]),
            (v["pos_inf"], v["neg_inf"]),
            (v["nan"], v["one"]),
        ],
        "sub": [
            (v["two"], v["one"]),
            (v["one"], v["two"]),
            (v["one"], v["one"]),
            (v["min_norm"], v["min_sub"]),
            (v["pos_inf"], v["pos_inf"]),
            (v["nan"], v["one"]),
        ],
        "mul": [
            (v["two"], v["two"]),
            (v["one_point_five"], v["one_point_five"]),
            (v["neg_one"], v["two"]),
            (v["min_sub"], v["two"]),
            (v["max_finite"], v["two"]),
            (v["pos_inf"], v["pos_zero"]),
            (v["nan"], v["one"]),
        ],
        "div": [
            (v["two"], v["two"]),
            (v["one"], v["three"]),
            (v["one"], v["two"]),
            (v["min_norm"], v["two"]),
            (v["one"], v["pos_zero"]),
            (v["pos_zero"], v["pos_zero"]),
            (v["pos_inf"], v["pos_inf"]),
            (v["nan"], v["one"]),
        ],
    }

    vectors: list[Vector] = []
    for op, pairs in pairs_by_op.items():
        for index, (left, right) in enumerate(pairs):
            vectors.append(Vector(fmt, op, left, right, reference_op(fmt, op, left, right), f"curated:{index}"))
    return vectors


def random_finite_bits(fmt: FloatFormat, rng: random.Random) -> int:
    sign = fmt.sign_mask if rng.randrange(2) else 0
    category = rng.randrange(10)

    if category == 0:
        return sign
    if category in (1, 2):
        return sign | rng.randrange(1, fmt.hidden_bit)

    exponent = rng.randrange(1, fmt.max_exponent)
    mantissa = rng.getrandbits(fmt.mantissa_bits)
    return sign | (exponent << fmt.mantissa_bits) | mantissa


def random_vectors(fmt: FloatFormat, per_op: int, seed: int) -> list[Vector]:
    rng = random.Random(seed + fmt.total_bits)
    vectors: list[Vector] = []

    for op in OP_SYMBOLS:
        for index in range(per_op):
            left = random_finite_bits(fmt, rng)
            right = random_finite_bits(fmt, rng)
            if op == "div" and is_zero(fmt, right) and index % 3 != 0:
                right = interesting_values(fmt)["one"]
            expected = reference_op(fmt, op, left, right)
            vectors.append(Vector(fmt, op, left, right, expected, f"random:{seed}:{index}"))

    return vectors


HEX_FLOAT_RE = re.compile(r"^([+-])([0-9A-F]+)\.([0-9A-F]+)P([+-]?\d+)$", re.IGNORECASE)
FPGEN_LINE_RE = re.compile(r"^b(32|64)([+\-*/])\s+")


def parse_fpgen_value(fmt: FloatFormat, token: str) -> int:
    token = token.strip()

    if token == "+Zero":
        return zero(fmt, False)
    if token == "-Zero":
        return zero(fmt, True)
    if token == "+Inf":
        return infinity(fmt, False)
    if token == "-Inf":
        return infinity(fmt, True)
    if token in {"Q", "S", "+Q", "-Q", "+S", "-S"}:
        sign = token.startswith("-")
        return (fmt.sign_mask if sign else 0) | canonical_nan(fmt)

    match = HEX_FLOAT_RE.match(token)
    if match is None:
        raise ValueError(f"unsupported FPgen value token: {token}")

    sign_text, integer_hex, fraction_hex, exponent_text = match.groups()
    significand = int(integer_hex + fraction_hex, 16)
    exponent = int(exponent_text) - (4 * len(fraction_hex))
    magnitude = scale_by_power_of_two(Fraction(significand, 1), exponent)
    return pack_finite(fmt, sign_text == "-", magnitude)


def download_fpgen_file(cache_dir: Path, filename: str) -> Path | None:
    cache_dir.mkdir(parents=True, exist_ok=True)
    target = cache_dir / filename
    if target.exists():
        return target

    try:
        with urllib.request.urlopen(f"{FPGEN_BASE_URL}/{filename}", timeout=20) as response:
            target.write_bytes(response.read())
        return target
    except (urllib.error.URLError, TimeoutError):
        return None


def fpgen_vectors(cache_dir: Path, files: list[str], per_op_limit: int, download: bool) -> list[Vector]:
    counts: dict[tuple[str, str], int] = {}
    vectors: list[Vector] = []

    for filename in files:
        path = cache_dir / filename
        if not path.exists() and download:
            downloaded = download_fpgen_file(cache_dir, filename)
            if downloaded is not None:
                path = downloaded
        if not path.exists():
            continue

        for line_number, line in enumerate(path.read_text().splitlines(), start=1):
            if "->" not in line or FPGEN_LINE_RE.match(line) is None:
                continue

            before, after = line.split("->", 1)
            left_tokens = before.split()
            head = left_tokens[0]
            precision = int(head[1:-1])
            operation = FPGEN_OPS[head[-1]]
            rounding = left_tokens[1]
            if rounding != "=0" or len(left_tokens) != 4:
                continue

            fmt = FORMAT_BY_BITS[precision]
            key = (fmt.name, operation)
            if counts.get(key, 0) >= per_op_limit:
                continue

            try:
                left = parse_fpgen_value(fmt, left_tokens[-2])
                right = parse_fpgen_value(fmt, left_tokens[-1])
                expected = parse_fpgen_value(fmt, after.split()[0])
            except ValueError:
                continue

            vectors.append(Vector(fmt, operation, left, right, expected, f"fpgen:{filename}:{line_number}"))
            counts[key] = counts.get(key, 0) + 1

    return vectors


def dedupe(vectors: list[Vector]) -> list[Vector]:
    seen: set[tuple[str, str, int, int, int]] = set()
    unique: list[Vector] = []

    for vector in vectors:
        key = vector.key()
        if key not in seen:
            unique.append(vector)
            seen.add(key)

    return unique


def literal(fmt: FloatFormat, value: int) -> str:
    width = fmt.total_bits // 4
    return f"0x{value:0{width}x} as {fmt.noir_uint}"


def test_name(fmt: FloatFormat, op: str) -> str:
    return f"generated_{fmt.name}_{op}_vectors_match_reference"


def render_noir(vectors: list[Vector], compares, unaries, casts) -> str:
    lines = [
        "// Generated by scripts/generate_float_vectors.py. Do not edit by hand.",
        "use sparq_ieee754::{f128, f16, f32, f64};",
        "",
    ]

    grouped: dict[tuple[str, str], list[Vector]] = {}
    for vector in vectors:
        grouped.setdefault((vector.fmt.name, vector.op), []).append(vector)

    for fmt in FORMATS:
        for op in OP_SYMBOLS:
            group = grouped.get((fmt.name, op), [])
            if not group:
                continue

            lines.append("#[test]")
            lines.append(f"fn {test_name(fmt, op)}() {{")
            for index, vector in enumerate(group):
                lines.append(f"    // {vector.source}")
                lines.append(
                    f"    assert_eq(("
                    f"{fmt.name}::new({literal(fmt, vector.left)}) {OP_SYMBOLS[op]} "
                    f"{fmt.name}::new({literal(fmt, vector.right)})).bits(), "
                    f"{literal(fmt, vector.expected)});"
                )
                if index + 1 != len(group):
                    lines.append("")
            lines.append("}")
            lines.append("")

    render_new_op_tests(lines, compares, unaries, casts)

    return "\n".join(lines).rstrip() + "\n"


def build_vectors(args: argparse.Namespace) -> list[Vector]:
    vectors: list[Vector] = []
    selected_formats = [FORMAT_BY_NAME[name] for name in args.formats]

    for fmt in selected_formats:
        vectors.extend(curated_vectors(fmt))
        vectors.extend(random_vectors(fmt, args.random_per_op, args.seed))

    if args.include_fpgen:
        vectors.extend(fpgen_vectors(args.fpgen_cache, args.fpgen_files, args.fpgen_per_op, args.download_fpgen))

    return dedupe(vectors)


def main() -> None:
    repo_root = Path(__file__).resolve().parent.parent
    parser = argparse.ArgumentParser(description="Generate Noir float arithmetic vector tests")
    parser.add_argument("--output", type=Path, default=repo_root / "tests" / "generated_arithmetic" / "src" / "lib.nr")
    parser.add_argument("--formats", nargs="+", choices=FORMAT_BY_NAME.keys(), default=[fmt.name for fmt in FORMATS])
    parser.add_argument("--random-per-op", type=int, default=8)
    parser.add_argument("--seed", type=int, default=754)
    parser.add_argument("--include-fpgen", action="store_true")
    parser.add_argument("--download-fpgen", action="store_true")
    parser.add_argument("--fpgen-cache", type=Path, default=repo_root / ".ieee754_test_cache")
    parser.add_argument("--fpgen-files", nargs="+", default=DEFAULT_FPGEN_FILES)
    parser.add_argument("--fpgen-per-op", type=int, default=8)
    args = parser.parse_args()

    vectors = build_vectors(args)
    selected_formats = [FORMAT_BY_NAME[name] for name in args.formats]
    all_compares: list[CompareVector] = []
    all_unaries: list[UnaryVector] = []
    all_casts: list[CastVector] = []

    for fmt in selected_formats:
        compares, unaries, casts = new_op_vectors(fmt, args.random_per_op, args.seed)
        all_compares.extend(compares)
        all_unaries.extend(unaries)
        all_casts.extend(casts)

    all_compares = dedupe_keyed(all_compares)
    all_unaries = dedupe_keyed(all_unaries)
    all_casts = dedupe_keyed(all_casts)

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(render_noir(vectors, all_compares, all_unaries, all_casts))

    counts: dict[tuple[str, str], int] = {}
    for vector in vectors:
        key = (vector.fmt.name, vector.op)
        counts[key] = counts.get(key, 0) + 1
    for vector in all_compares + all_unaries + all_casts:
        key = (vector.fmt.name, vector.op)
        counts[key] = counts.get(key, 0) + 1

    total = len(vectors) + len(all_compares) + len(all_unaries) + len(all_casts)
    print(f"wrote {total} vectors to {args.output}")
    ops_order = list(OP_SYMBOLS) + COMPARE_OPS + ROUND_OPS + ["sqrt"] + CAST_OPS
    for fmt in FORMATS:
        for op in ops_order:
            count = counts.get((fmt.name, op), 0)
            if count:
                print(f"  {fmt.name} {op}: {count}")


if __name__ == "__main__":
    main()