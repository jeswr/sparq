// ============================================================================
// IEEE 754 Compliant Floating Point Types
// ============================================================================

// IEEE 754 Rounding Modes
// As defined in IEEE 754-2019 Section 4.3
// Using u8 constants instead of enums for better stability
pub global ROUNDING_MODE_NEAREST_EVEN: u8 = 0; // Round to nearest, ties to even (default) - =0 in fptest
pub global ROUNDING_MODE_NEAREST_AWAY: u8 = 1; // Round to nearest, ties away from zero - =^ in fptest
pub global ROUNDING_MODE_TOWARD_POSITIVE: u8 = 2; // Round toward +Infinity (ceiling) - > in fptest
pub global ROUNDING_MODE_TOWARD_NEGATIVE: u8 = 3; // Round toward -Infinity (floor) - < in fptest
pub global ROUNDING_MODE_TOWARD_ZERO: u8 = 4; // Round toward 0 (truncate) - 0 in fptest

// IEEE 754 Single-Precision (32-bit) Float
// - 1 bit sign
// - 8 bits exponent (bias 127)
// - 23 bits mantissa (with implicit leading 1 for normalized numbers)
pub struct IEEE754Float32 {
    pub sign: u8, // 0 = positive, 1 = negative
    pub exponent: u8, // Biased exponent (0-255)
    pub mantissa: u32, // 23-bit mantissa stored in lower bits
}

// IEEE 754 Double-Precision (64-bit) Float
// - 1 bit sign
// - 11 bits exponent (bias 1023)
// - 52 bits mantissa (with implicit leading 1 for normalized numbers)
pub struct IEEE754Float64 {
    pub sign: u8, // 0 = positive, 1 = negative
    pub exponent: u16, // Biased exponent (0-2047), using u16 to hold 11 bits
    pub mantissa: u64, // 52-bit mantissa stored in lower bits
}

// Constants for IEEE 754 Float32
pub global FLOAT32_EXPONENT_MAX: u8 = 255;
pub global FLOAT32_IMPLICIT_BIT: u32 = 0x800000; // 1 << 23

// Float32 special value constants (bit representations)
pub global FLOAT32_ZERO: u32 = 0x00000000; // +0.0
pub global FLOAT32_NEG_ZERO: u32 = 0x80000000; // -0.0
pub global FLOAT32_ONE: u32 = 0x3F800000; // 1.0
pub global FLOAT32_NEG_ONE: u32 = 0xBF800000; // -1.0
pub global FLOAT32_INFINITY: u32 = 0x7F800000; // +Infinity
pub global FLOAT32_NEG_INFINITY: u32 = 0xFF800000; // -Infinity
pub global FLOAT32_NAN: u32 = 0x7FC00000; // Quiet NaN
pub global FLOAT32_SIGNALING_NAN: u32 = 0x7FA00000; // Signaling NaN

// Constants for IEEE 754 Float64
pub global FLOAT64_EXPONENT_MAX: u16 = 2047;
pub global FLOAT64_IMPLICIT_BIT: u64 = 0x10000000000000; // 1 << 52

// Float64 special value constants (bit representations)
pub global FLOAT64_ZERO: u64 = 0x0000000000000000; // +0.0
pub global FLOAT64_NEG_ZERO: u64 = 0x8000000000000000; // -0.0
pub global FLOAT64_ONE: u64 = 0x3FF0000000000000; // 1.0
pub global FLOAT64_NEG_ONE: u64 = 0xBFF0000000000000; // -1.0
pub global FLOAT64_INFINITY: u64 = 0x7FF0000000000000; // +Infinity
pub global FLOAT64_NEG_INFINITY: u64 = 0xFFF0000000000000; // -Infinity
pub global FLOAT64_NAN: u64 = 0x7FF8000000000000; // Quiet NaN
pub global FLOAT64_SIGNALING_NAN: u64 = 0x7FF4000000000000; // Signaling NaN
