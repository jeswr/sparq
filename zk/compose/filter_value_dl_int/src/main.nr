// Circuit-family member: DUAL-LEAF value-lane FILTER over a committed
// NON-NEGATIVE xsd:integer (sq-xojl). Unlike `filter_int` (which re-hashes the
// canonical token with the in-circuit blake3 BLACKBOX -- ~14k gates), this member
// binds the operand to the dual-leaf commitment via TWO Poseidon2 permutations
// over the witnessed VALUE_HOOK and carries `lexical_component` as a FREE witness
// -- NO in-circuit blake3 (the gate win). It carries the DOCUMENTED INV-VL
// downgrade: value<->lexical agreement on the value-FILTER lane is
// TRUSTED-ISSUER-HONESTY, not machine-enforced (#769 accepted, CR-G8 / sq-qhy4).
// The whole ZK estate is NOT externally audited; no soundness/privacy claim.
// Relation: sparq_zk_compose_core::filter_value::filter_value_dl_int.
//
// Unlike the blake3 family, this member is DIGIT-COUNT-FREE: there is no `[u8; D]`
// witness, so the per-D family collapses to ONE member per datatype class and the
// member selection no longer leaks ceil(log10(value)) (the design S3.2 win).
use sparq_zk_compose_core::filter_value::filter_value_dl_int;

fn main(
    // -- public --
    challenge: pub Field,    // verifier nonce (replay binding S2.5)
    operand_enc: pub Field,  // the hidden column's dual-leaf term encoding (scan anchor)
    op: pub u32,             // 0=lt 1=le 2=gt 3=ge 4=eq 5=ne
    bound: pub u64,          // the FILTER's constant operand
    datatype_const: pub Field, // blake3(datatype IRI) as a field (xsd:integer)
    expected: pub bool,      // the disclosed verdict for this operand
    // -- private --
    value_hook: Field,        // the numeric value handle (the dual-leaf value component)
    lexical_component: Field,  // OFF-circuit blake3 lexical hash, carried as a witness
) {
    let _ = challenge;
    filter_value_dl_int(
        operand_enc,
        op,
        bound,
        expected,
        value_hook,
        lexical_component,
        datatype_const,
    );
}
