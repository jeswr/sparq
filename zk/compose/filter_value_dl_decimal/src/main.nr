// Circuit-family member: DUAL-LEAF value-lane FILTER over a committed
// xsd:decimal at a FIXED canonical fraction-digit scale (sq-2ezsx, extends the
// integer member sq-xojl). Like filter_value_dl_int it binds the operand to the
// dual-leaf commitment via TWO Poseidon2 permutations over the witnessed
// VALUE_HOOK (here the SIGNED scaled magnitude) and carries `lexical_component`
// as a FREE witness -- NO in-circuit blake3.
//
// B4 -- the canonical-scale bind: xsd:decimal is many-to-one on the term at a
// fixed scale ("5.0"=="5.00"), so the value handle is the magnitude at exactly FD
// fraction places and the member's FD is folded into the public datatype_const
// (the host derives it from BOTH the xsd:decimal IRI AND FD, so a value at one
// scale can never collide a value at another). Within a member the scaled value is
// canonical; "5.0"/"5.00" are disambiguated by lexical_component for identity ops
// (reject-list (v), enforced by the dispatch matrix sq-cfmv). The member is
// FD-AGNOSTIC: the scale lives in datatype_const (a public input), so ONE compiled
// member serves every decimal scale -- unlike the blake3 filter_decimal_i{id}_f{fd}
// family, this is DIGIT-COUNT-FREE (no member-selection leak of ceil(log10) or fd).
//
// Carries the DOCUMENTED INV-VL downgrade: value<->lexical agreement is
// TRUSTED-ISSUER-HONESTY, not machine-enforced (#769 accepted, CR-G8 / sq-qhy4).
// The ZK estate is NOT externally audited; no soundness/privacy claim.
// Relation: sparq_zk_compose_core::filter_value::filter_value_dl_decimal.
use sparq_zk_compose_core::filter_value::filter_value_dl_decimal;

fn main(
    // -- public --
    challenge: pub Field,      // verifier nonce (replay binding S2.5)
    operand_enc: pub Field,    // the hidden column's dual-leaf decimal encoding (scan anchor)
    op: pub u32,               // 0=lt 1=le 2=gt 3=ge 4=eq 5=ne
    bound_neg: pub bool,       // sign of the FILTER's constant operand
    bound_scaled: pub u64,     // round(|bound| * 10^FD) -- host-prescaled to FD places
    datatype_const: pub Field, // blake3(xsd:decimal IRI || "@scale=FD") as a field
    expected: pub bool,        // the disclosed verdict for this operand
    // -- private --
    value_neg: bool,           // sign of the operand value
    value_hook_scaled: Field,  // round(|v| * 10^FD) -- the value handle at scale FD
    lexical_component: Field,  // OFF-circuit blake3 lexical hash, carried as a witness
) {
    let _ = challenge;
    filter_value_dl_decimal(
        operand_enc,
        op,
        bound_neg,
        bound_scaled,
        expected,
        value_neg,
        value_hook_scaled,
        lexical_component,
        datatype_const,
    );
}
