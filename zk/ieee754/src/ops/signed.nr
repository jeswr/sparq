fn signed_magnitude_from_bits<let SIGN_SCALE: u128>(sign: bool, bits: u128) -> (bool, u128) {
    let magnitude = if sign { (SIGN_SCALE * 2) - bits } else { bits };

    (sign, magnitude)
}

pub(crate) fn signed_magnitude_i8(value: i8) -> (bool, u128) {
    let unsigned = value as u8;

    signed_magnitude_from_bits::<0x80_u128>(unsigned >= 0x80, unsigned as u128)
}

pub(crate) fn signed_magnitude_i16(value: i16) -> (bool, u128) {
    let unsigned = value as u16;

    signed_magnitude_from_bits::<0x8000_u128>(unsigned >= 0x8000, unsigned as u128)
}

pub(crate) fn signed_magnitude_i32(value: i32) -> (bool, u128) {
    let unsigned = value as u32;

    signed_magnitude_from_bits::<0x80000000_u128>(unsigned >= 0x80000000, unsigned as u128)
}

pub(crate) fn signed_magnitude_i64(value: i64) -> (bool, u128) {
    let unsigned = value as u64;

    signed_magnitude_from_bits::<0x8000000000000000_u128>(unsigned >= 0x8000000000000000, unsigned as u128)
}
