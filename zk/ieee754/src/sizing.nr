pub comptime fn uint_type(bits: u8) -> Type {
    std::static_assert(bits <= 254, "Bit width must fit within the Field size");
    std::static_assert(bits != 0, "Bit width must be greater than 0");

    let mut type_name: Type = quote { Field }.as_type();

    for i in [128, 64, 32, 16, 8] {
        if bits <= i {
            type_name = f"u{i}".quoted_contents().as_type();
        }
    }

    if bits == 1 {
        type_name = quote { bool }.as_type();
    }

    type_name
}

pub comptime fn exponent_size(bits: u8) -> u8 {
    let mut exponent_bits: u8 = 1;

    if bits == 16 {
        exponent_bits = 5;
    } else if bits == 32 {
        exponent_bits = 8;
    } else if bits >= 64 {
        let mut power: u64 = 1;
        let mut log_bits: u8 = 0;

        while power < (bits as u64) {
            power = power << 1;
            log_bits = log_bits + 1;
        }

        std::static_assert(
            power == (bits as u64),
            "Exponent size calculation is only defined for powers of two",
        );
        exponent_bits = (4 as u8) * log_bits - 13;
    } else {
        std::static_assert(
            false,
            "Exponent size calculation is only defined for bit widths of 16, 32, or >= 64",
        );
    }

    exponent_bits
}

pub comptime fn mantissa_size(bits: u8) -> u8 {
    bits - exponent_size(bits) - 1
}

pub comptime fn significand_size(bits: u8) -> u8 {
    mantissa_size(bits) + 1
}
