pub struct FloatParts<E, M> {
    pub(crate) sign: bool,
    pub(crate) exponent: E,
    pub(crate) mantissa: M,
}

impl<E, M> FloatParts<E, M> {
    pub(crate) fn new(sign: bool, exponent: E, mantissa: M) -> Self {
        FloatParts { sign, exponent, mantissa }
    }
}