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


def render_noir(vectors: list[Vector]) -> str:
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
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(render_noir(vectors))

    counts: dict[tuple[str, str], int] = {}
    for vector in vectors:
        key = (vector.fmt.name, vector.op)
        counts[key] = counts.get(key, 0) + 1

    print(f"wrote {len(vectors)} vectors to {args.output}")
    for fmt in FORMATS:
        for op in OP_SYMBOLS:
            count = counts.get((fmt.name, op), 0)
            if count:
                print(f"  {fmt.name} {op}: {count}")


if __name__ == "__main__":
    main()