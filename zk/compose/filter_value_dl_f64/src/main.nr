// Circuit-family member: DUAL-LEAF value-lane FILTER over a committed
// xsd:double (sq-2ezsx, extends the integer member sq-xojl). Like
// filter_value_dl_int it binds the operand to the dual-leaf commitment via TWO
// Poseidon2 permutations over the witnessed VALUE_HOOK (here the IEEE-754 bit
// pattern) and carries `lexical_component` as a FREE witness -- NO in-circuit
// blake3 (the gate win) AND no in-circuit decimal->IEEE parse (the unbudgeted
// filter_float deferral the value lane sidesteps by committing the bits).
//
// B4 -- the IEEE-bit canonical bind: the member canonicalises the witnessed bits
// (-0.0 -> +0.0; any NaN -> the canonical qNaN) BEFORE forming the value
// component, so two bit-distinct but SPARQL-numerically-equal terms collapse to
// ONE value handle -- the many-to-one-on-the-term property the lexical_component
// disambiguates for identity ops (reject-list (v), enforced by the dispatch
// matrix sq-cfmv). Carries the DOCUMENTED INV-VL downgrade: value<->lexical
// agreement is TRUSTED-ISSUER-HONESTY, not machine-enforced (#769 accepted,
// CR-G8 / sq-qhy4). The ZK estate is NOT externally audited; no soundness/privacy
// claim. Relation: sparq_zk_compose_core::filter_value::filter_value_dl_f64.
use sparq_zk_compose_core::filter_value::filter_value_dl_f64;

fn main(
    // -- public --
    challenge: pub Field,      // verifier nonce (replay binding S2.5)
    operand_enc: pub Field,    // the hidden column's dual-leaf double encoding (scan anchor)
    op: pub u32,               // 0=lt 1=le 2=gt 3=ge 4=eq 5=ne
    b_bits: pub u64,           // the FILTER's constant double as an IEEE-754 bit pattern
    datatype_const: pub Field, // blake3(datatype IRI) as a field (xsd:double)
    expected: pub bool,        // the disclosed verdict for this operand
    // -- private --
    value_hook: Field,         // the IEEE-754 double bit pattern (the value handle)
    lexical_component: Field,  // OFF-circuit blake3 lexical hash, carried as a witness
) {
    let _ = challenge;
    filter_value_dl_f64(
        operand_enc,
        op,
        b_bits,
        expected,
        value_hook,
        lexical_component,
        datatype_const,
    );
}
