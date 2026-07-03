use crate::sizing::{exponent_size, mantissa_size, uint_type};

pub comptime fn float_type_from_params(bits: u8) -> Quoted {
    let exp_size: u8 = exponent_size(bits);
    let mant_size: u8 = mantissa_size(bits);
    // [SONNET-4.6] sq-3x7dl.1: u32 versions for assert_max_bit_size::<N: u32>().
    let exp_size_u32: u32 = exp_size as u32;
    let mant_size_u32: u32 = mant_size as u32;

    let exp_type = uint_type(exp_size);
    let mant_type = uint_type(mant_size);
    let full_type = uint_type(bits);
    let type_name = f"f{bits}".quoted_contents();
    let add_call = if bits == 128 {
        quote { Self::from_parts($crate::ops::add_wide(self.to_parts(), other.to_parts())) }
    } else {
        quote { Self::from_parts($crate::ops::add::<$exp_type, $mant_type, $exp_size, $mant_size>(self.to_parts(), other.to_parts())) }
    };
    let sub_call = if bits == 128 {
        quote { Self::from_parts($crate::ops::sub_wide(self.to_parts(), other.to_parts())) }
    } else {
        quote { Self::from_parts($crate::ops::sub::<$exp_type, $mant_type, $exp_size, $mant_size>(self.to_parts(), other.to_parts())) }
    };
    let mul_call = if bits == 128 {
        quote { Self::from_parts($crate::ops::mul_wide(self.to_parts(), other.to_parts())) }
    } else {
        quote { Self::from_parts($crate::ops::mul::<$exp_type, $mant_type, $exp_size, $mant_size>(self.to_parts(), other.to_parts())) }
    };
    let div_call = if bits == 128 {
        quote { Self::from_parts($crate::ops::div_wide(self.to_parts(), other.to_parts())) }
    } else {
        quote { Self::from_parts($crate::ops::div::<$exp_type, $mant_type, $exp_size, $mant_size>(self.to_parts(), other.to_parts())) }
    };
    let from_magnitude_parts = if bits == 128 {
        quote { $crate::ops::uint_to_float_parts_wide::<INPUT_BITS>(sign, magnitude) }
    } else if bits == 64 {
        quote { $crate::ops::uint_to_float_parts_u64::<$exp_size, $mant_size, INPUT_BITS>(sign, magnitude) }
    } else {
        quote { $crate::ops::from_u64_work_parts::<$exp_type, $mant_type>($crate::ops::uint_to_float_parts_u64::<$exp_size, $mant_size, INPUT_BITS>(sign, magnitude)) }
    };
    let compare_call = quote {
        $crate::ops::compare_parts::<$exp_type, $mant_type, $exp_size>(self.to_parts(), other.to_parts())
    };
    let floor_call = if bits == 128 {
        quote { Self::from_parts($crate::ops::round_to_integral_wide::<0_u8>(self.to_parts())) }
    } else {
        quote { Self::from_parts($crate::ops::round_to_integral::<$exp_type, $mant_type, $exp_size, $mant_size, 0_u8>(self.to_parts())) }
    };
    let ceil_call = if bits == 128 {
        quote { Self::from_parts($crate::ops::round_to_integral_wide::<1_u8>(self.to_parts())) }
    } else {
        quote { Self::from_parts($crate::ops::round_to_integral::<$exp_type, $mant_type, $exp_size, $mant_size, 1_u8>(self.to_parts())) }
    };
    let trunc_call = if bits == 128 {
        quote { Self::from_parts($crate::ops::round_to_integral_wide::<2_u8>(self.to_parts())) }
    } else {
        quote { Self::from_parts($crate::ops::round_to_integral::<$exp_type, $mant_type, $exp_size, $mant_size, 2_u8>(self.to_parts())) }
    };
    let round_ties_even_call = if bits == 128 {
        quote { Self::from_parts($crate::ops::round_to_integral_wide::<3_u8>(self.to_parts())) }
    } else {
        quote { Self::from_parts($crate::ops::round_to_integral::<$exp_type, $mant_type, $exp_size, $mant_size, 3_u8>(self.to_parts())) }
    };
    let to_u64_call = if bits == 128 {
        quote { $crate::ops::float_to_u64_wide(self.to_parts()) }
    } else {
        quote { $crate::ops::float_to_u64::<$exp_type, $mant_type, $exp_size, $mant_size>(self.to_parts()) }
    };
    let sqrt_call = if bits == 128 {
        quote { Self::from_parts($crate::ops::sqrt_wide(self.to_parts())) }
    } else {
        quote { Self::from_parts($crate::ops::sqrt_op::<$exp_type, $mant_type, $exp_size, $mant_size>(self.to_parts())) }
    };
    let to_i64_call = if bits == 128 {
        quote { $crate::ops::float_to_i64_wide(self.to_parts()) }
    } else {
        quote { $crate::ops::float_to_i64::<$exp_type, $mant_type, $exp_size, $mant_size>(self.to_parts()) }
    };
    let sign_scale = (1 as u128) << ((bits - 1) as u128);
    let exponent_scale = (1 as u128) << (mant_size as u128);
    let exponent_mask = ((1 as u128) << (exp_size as u128)) - 1;
    let mantissa_mask = ((1 as u128) << (mant_size as u128)) - 1;
    let exponent_pack = if bits == 128 {
        quote { ((self.exponent as $full_type) << ($mant_size as $full_type)) }
    } else {
        quote { ((self.exponent as $full_type) * ($exponent_scale as $full_type)) }
    };

    quote {
        pub struct $type_name {
            sign: bool,
            exponent: $exp_type,
            mantissa: $mant_type,
        }

        impl $type_name {
            // Private helper: constrain exponent and mantissa to their real IEEE field
            // widths.  new() calls this after decode; the forge tests call it directly
            // on malicious field values so that removing the body makes those tests RED
            // -- they pin the helper's logic, not a disconnected inline assert.
            // [SONNET-4.6] sq-3x7dl.1
            fn assert_canonical_fields(exponent: $exp_type, mantissa: $mant_type) {
                (exponent as Field).assert_max_bit_size::<$exp_size_u32>();
                (mantissa as Field).assert_max_bit_size::<$mant_size_u32>();
            }

            pub fn new(bits: $full_type) -> Self {
                // Safety: the decoded witness is constrained by re-encoding it with bits()
                // and asserting that it exactly matches the input bit pattern.
                let decoded = unsafe { Self::decode_unconstrained(bits) };
                assert_eq(decoded.bits(), bits);
                // [SONNET-4.6] sq-3x7dl.1 SOUNDNESS FIX: assert_eq above is NECESSARY
                // but NOT SUFFICIENT -- because bits() = sign*2^(W-1) + exp*2^mant_size
                // + mantissa is non-injective when field types exceed IEEE field widths, a
                // malicious prover can substitute {exp-1, mantissa+2^mant_size} for the
                // canonical {exp, mantissa} and produce the same bits() while denoting a
                // DIFFERENT real number.  assert_canonical_fields makes bits() injective
                // and pins the unique canonical IEEE decomposition for every generated width.
                // NOTE: circuit-level pinning of the new() call-site (a malicious witness
                // fed through the prover) is a forge/proving-level obligation tracked under
                // sq-l9ulg / sq-qhy4 -- the functional tests pin the helper's logic; the
                // forge suite + audit pin the call-site.
                Self::assert_canonical_fields(decoded.exponent, decoded.mantissa);

                decoded
            }

            unconstrained fn decode_unconstrained(bits: $full_type) -> Self {
                let sign = (bits & ($sign_scale as $full_type)) == ($sign_scale as $full_type);
                let exponent = ((bits >> ($mant_size as $full_type)) & ($exponent_mask as $full_type)) as $exp_type;
                let mantissa = (bits & ($mantissa_mask as $full_type)) as $mant_type;

                Self { sign, exponent, mantissa }
            }

            pub fn bits(self) -> $full_type {
                (if self.sign { $sign_scale as $full_type } else { 0 as $full_type })
                    + $exponent_pack
                    + (self.mantissa as $full_type)
            }

            fn from_parts(parts: $crate::parts::FloatParts<$exp_type, $mant_type>) -> Self {
                Self {
                    sign: parts.sign,
                    exponent: parts.exponent,
                    mantissa: parts.mantissa,
                }
            }

            fn to_parts(self) -> $crate::parts::FloatParts<$exp_type, $mant_type> {
                $crate::parts::FloatParts::new(self.sign, self.exponent, self.mantissa)
            }

            fn from_magnitude<let INPUT_BITS: u32>(sign: bool, magnitude: u128) -> Self {
                Self::from_parts($from_magnitude_parts)
            }

            // IEEE 754 comparison predicates. These are deliberately inherent
            // methods rather than std::cmp::Eq / std::cmp::Ord impls: those
            // traits assume a reflexive equality and a total order, and IEEE
            // floats have neither (NaN is unordered against everything,
            // including itself, while -0 == +0 makes bit-distinct values
            // equal). All six predicates evaluate both operands
            // unconditionally, so the gate count does not depend on the
            // secret values.

            /// IEEE numeric equality: false when either side is NaN; -0 == +0.
            pub fn eq(self, other: Self) -> bool {
                let (_unordered, _less, equal) = $compare_call;

                equal
            }

            /// IEEE numeric inequality: true when either side is NaN.
            /// `x.ne(x)` is the idiomatic NaN test.
            pub fn ne(self, other: Self) -> bool {
                let (_unordered, _less, equal) = $compare_call;

                !equal
            }

            /// IEEE less-than: false when either side is NaN.
            pub fn lt(self, other: Self) -> bool {
                let (_unordered, less, _equal) = $compare_call;

                less
            }

            /// IEEE less-than-or-equal: false when either side is NaN.
            pub fn le(self, other: Self) -> bool {
                let (_unordered, less, equal) = $compare_call;

                less | equal
            }

            /// IEEE greater-than: false when either side is NaN.
            pub fn gt(self, other: Self) -> bool {
                let (unordered, less, equal) = $compare_call;

                !unordered & !less & !equal
            }

            /// IEEE greater-than-or-equal: false when either side is NaN.
            pub fn ge(self, other: Self) -> bool {
                let (unordered, less, _equal) = $compare_call;

                !unordered & !less
            }

            // Round-to-integral family (IEEE 754 roundToIntegral*). All four
            // return floats, propagate NaN (canonicalised), pass infinities
            // through unchanged, and preserve the sign of zero results
            // (e.g. (-0.3).ceil() == -0).

            /// Round toward negative infinity (roundToIntegralTowardNegative).
            pub fn floor(self) -> Self {
                $floor_call
            }

            /// Round toward positive infinity (roundToIntegralTowardPositive).
            pub fn ceil(self) -> Self {
                $ceil_call
            }

            /// Round toward zero (roundToIntegralTowardZero).
            pub fn trunc(self) -> Self {
                $trunc_call
            }

            /// Round to nearest integer with ties to even
            /// (roundToIntegralTiesToEven).
            pub fn round_ties_even(self) -> Self {
                $round_ties_even_call
            }

            /// IEEE 754 squareRoot with round-to-nearest-even. NaN inputs and
            /// negative non-zero inputs (including -inf) return the canonical
            /// NaN; sqrt(+-0) preserves the zero's sign; sqrt(+inf) is +inf.
            pub fn sqrt(self) -> Self {
                $sqrt_call
            }

            // Float-to-integer casts with XPath 2.0 F&O semantics (the
            // SPARQL/XPath consumer): truncate toward zero, then range-check.
            // NaN and infinities assert (err:FOCA0002), as do results outside
            // the target range (err:FOCA0003). Negative inputs above -1
            // truncate to 0 and are therefore valid for to_u64. Pre-screen
            // with the comparison predicates (`x.ne(x)` is the NaN test) when
            // failure must remain provable.

            /// Truncating cast to u64; asserts on NaN, infinity, values at or
            /// above 2^64, and values at or below -1.
            pub fn to_u64(self) -> u64 {
                $to_u64_call
            }

            /// Truncating cast to i64; asserts on NaN, infinity, and values
            /// outside [-2^63, 2^63 - 1] after truncation.
            pub fn to_i64(self) -> i64 {
                $to_i64_call
            }
        }

        impl std::ops::Add for $type_name {
            fn add(self, other: Self) -> Self {
                $add_call
            }
        }

        impl std::ops::Sub for $type_name {
            fn sub(self, other: Self) -> Self {
                $sub_call
            }
        }

        impl std::ops::Mul for $type_name {
            fn mul(self, other: Self) -> Self {
                $mul_call
            }
        }

        impl std::ops::Div for $type_name {
            fn div(self, other: Self) -> Self {
                $div_call
            }
        }

        impl std::convert::From<u8> for $type_name {
            fn from(value: u8) -> Self {
                Self::from_magnitude::<8>(false, value as u128)
            }
        }

        impl std::convert::From<u16> for $type_name {
            fn from(value: u16) -> Self {
                Self::from_magnitude::<16>(false, value as u128)
            }
        }

        impl std::convert::From<u32> for $type_name {
            fn from(value: u32) -> Self {
                Self::from_magnitude::<32>(false, value as u128)
            }
        }

        impl std::convert::From<u64> for $type_name {
            fn from(value: u64) -> Self {
                Self::from_magnitude::<64>(false, value as u128)
            }
        }

        impl std::convert::From<u128> for $type_name {
            fn from(value: u128) -> Self {
                Self::from_magnitude::<128>(false, value)
            }
        }

        impl std::convert::From<i8> for $type_name {
            fn from(value: i8) -> Self {
                let (sign, magnitude) = $crate::ops::signed_magnitude_i8(value);

                Self::from_magnitude::<8>(sign, magnitude)
            }
        }

        impl std::convert::From<i16> for $type_name {
            fn from(value: i16) -> Self {
                let (sign, magnitude) = $crate::ops::signed_magnitude_i16(value);

                Self::from_magnitude::<16>(sign, magnitude)
            }
        }

        impl std::convert::From<i32> for $type_name {
            fn from(value: i32) -> Self {
                let (sign, magnitude) = $crate::ops::signed_magnitude_i32(value);

                Self::from_magnitude::<32>(sign, magnitude)
            }
        }

        impl std::convert::From<i64> for $type_name {
            fn from(value: i64) -> Self {
                let (sign, magnitude) = $crate::ops::signed_magnitude_i64(value);

                Self::from_magnitude::<64>(sign, magnitude)
            }
        }
    }
}

pub comptime fn generate_float_type(_item: TypeDefinition, bits: u8) -> Quoted {
    float_type_from_params(bits)
}