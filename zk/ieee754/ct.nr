pub comptime fn uint_type(bits: u8) -> Type {
    // TODO Get field size from noir runtime rather than assuming we are using BN254
    std::static_assert(bits <= 254, "Bit width must fit within the Field size");
    std::static_assert(bits != 0, "Bit width must be greater than 0");

    let mut type_name: Type = quote { Field }.as_type();

    for i in [128, 64, 32, 16, 8] {
        if bits <= i {
            type_name = quote { f"u{i}".quoted_contents() }.as_type();
        }
    }

    if bits == 1 {
        type_name = quote { bool }.as_type();
    }

    type_name
}

pub comptime fn exponent_size(bits: u8) -> u8 {
    let mut e: u8 = 1;

    if bits == 16 {
        e = 5;
    } else if bits == 32 {
        e = 8;
    } else {
        // TODO: Double check this formula
        // For N >= 64, e = 4 log2(N) - 13
        let lhs: u64 = (e as u64 + 13) << 2;
        let rhs: u64 = (bits as u64 * bits as u64 * bits as u64 * bits as u64);

        while lhs < rhs {
            e = e + 1;
        }

        std::static_assert(lhs == rhs, "Exponent size calculation is incorrect");
    }

    e
}

comptime fn uint_type_from_float_params(bits: u8) -> Quoted {
    let exp_size: u8 = exponent_size(bits);
    let mant_size: u8 = bits - exp_size - 1;

    let exp_type = uint_type(exp_size);
    let mant_type = uint_type(mant_size);

    let tup = (exp_type, mant_type);

    quote {
        pub type f"c{bits}".quoted_contents() = tup;
    }
}
