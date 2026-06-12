//! Test chunk 0 for fn:round-double
//! Contains 50 tests

use dep::xpath::{round_double, XsdDouble};

#[test]
fn fn_rounddbl1args_1() {
    //  Evaluates The 'round' function with the arguments set as follows: $arg =
    let val = XsdDouble::from_bits(18442240474082181119);
    assert(round_double(val).to_bits() == 18442240474082181119);
}

#[test]
fn fn_rounddbl1args_3() {
    //  Evaluates The 'round' function with the arguments set as follows: $arg =
    let val = XsdDouble::from_bits(9218868437227405311);
    assert(round_double(val).to_bits() == 9218868437227405311);
}

#[test]
fn k2_roundfunc_15() {
    //  Invoke on xs:double 0.
    let val = XsdDouble::from_bits(0);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_16() {
    //  Invoke on xs:double -0.
    let val = XsdDouble::from_bits(9223372036854775808);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_24() {
    //  Invoke on 0.01, type xs:double.
    let val = XsdDouble::from_bits(4576918229304087675);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_26() {
    //  Invoke on -0.01, type xs:double.
    let val = XsdDouble::from_bits(13800290266158863483);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_28() {
    //  Invoke on 0.04, type xs:double.
    let val = XsdDouble::from_bits(4585925428558828667);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_30() {
    //  Invoke on -0.04, type xs:double.
    let val = XsdDouble::from_bits(13809297465413604475);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_32() {
    //  Invoke on 0.05, type xs:double.
    let val = XsdDouble::from_bits(4587366580439587226);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_34() {
    //  Invoke on -0.05, type xs:double.
    let val = XsdDouble::from_bits(13810738617294363034);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_36() {
    //  Invoke on 0.06, type xs:double.
    let val = XsdDouble::from_bits(4588807732320345784);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_38() {
    //  Invoke on -0.06, type xs:double.
    let val = XsdDouble::from_bits(13812179769175121592);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_40() {
    //  Invoke on 0.09, type xs:double.
    let val = XsdDouble::from_bits(4591149604126578442);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_42() {
    //  Invoke on -0.09, type xs:double.
    let val = XsdDouble::from_bits(13814521640981354250);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_44() {
    //  Invoke on 0.00, type xs:double.
    let val = XsdDouble::from_bits(0);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_46() {
    //  Invoke on -0.00, type xs:double.
    let val = XsdDouble::from_bits(9223372036854775808);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_48() {
    //  Invoke on 0.11, type xs:double.
    let val = XsdDouble::from_bits(4592590756007337001);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_50() {
    //  Invoke on -0.11, type xs:double.
    let val = XsdDouble::from_bits(13815962792862112809);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_52() {
    //  Invoke on 0.14, type xs:double.
    let val = XsdDouble::from_bits(4594212051873190380);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_56() {
    //  Invoke on 0.15, type xs:double.
    let val = XsdDouble::from_bits(4594572339843380019);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_58() {
    //  Invoke on -0.15, type xs:double.
    let val = XsdDouble::from_bits(13817944376698155827);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_60() {
    //  Invoke on 0.16, type xs:double.
    let val = XsdDouble::from_bits(4594932627813569659);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_62() {
    //  Invoke on -0.16, type xs:double.
    let val = XsdDouble::from_bits(13818304664668345467);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_64() {
    //  Invoke on 0.19, type xs:double.
    let val = XsdDouble::from_bits(4596013491724138578);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_66() {
    //  Invoke on -0.19, type xs:double.
    let val = XsdDouble::from_bits(13819385528578914386);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_68() {
    //  Invoke on 0.10, type xs:double.
    let val = XsdDouble::from_bits(4591870180066957722);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_70() {
    //  Invoke on -0.10, type xs:double.
    let val = XsdDouble::from_bits(13815242216921733530);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_72() {
    //  Invoke on 0.41, type xs:double.
    let val = XsdDouble::from_bits(4601057523306793533);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_74() {
    //  Invoke on -0.41, type xs:double.
    let val = XsdDouble::from_bits(13824429560161569341);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_76() {
    //  Invoke on 0.44, type xs:double.
    let val = XsdDouble::from_bits(4601597955262077993);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_78() {
    //  Invoke on -0.44, type xs:double.
    let val = XsdDouble::from_bits(13824969992116853801);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_80() {
    //  Invoke on 0.45, type xs:double.
    let val = XsdDouble::from_bits(4601778099247172813);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_82() {
    //  Invoke on -0.45, type xs:double.
    let val = XsdDouble::from_bits(13825150136101948621);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_84() {
    //  Invoke on 0.46, type xs:double.
    let val = XsdDouble::from_bits(4601958243232267633);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_86() {
    //  Invoke on -0.46, type xs:double.
    let val = XsdDouble::from_bits(13825330280087043441);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_88() {
    //  Invoke on 0.49, type xs:double.
    let val = XsdDouble::from_bits(4602498675187552092);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_90() {
    //  Invoke on -0.49, type xs:double.
    let val = XsdDouble::from_bits(13825870712042327900);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_92() {
    //  Invoke on 0.40, type xs:double.
    let val = XsdDouble::from_bits(4600877379321698714);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_94() {
    //  Invoke on -0.40, type xs:double.
    let val = XsdDouble::from_bits(13824249416176474522);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_96() {
    //  Invoke on 0.51, type xs:double.
    let val = XsdDouble::from_bits(4602768891165194322);
    assert(round_double(val).to_bits() == 4607182418800017408);
}

#[test]
fn k2_roundfunc_98() {
    //  Invoke on -0.51, type xs:double.
    let val = XsdDouble::from_bits(13826140928019970130);
    assert(round_double(val).to_bits() == 13830554455654793216);
}

#[test]
fn k2_roundfunc_100() {
    //  Invoke on 0.54, type xs:double.
    let val = XsdDouble::from_bits(4603039107142836552);
    assert(round_double(val).to_bits() == 4607182418800017408);
}

#[test]
fn k2_roundfunc_102() {
    //  Invoke on -0.54, type xs:double.
    let val = XsdDouble::from_bits(13826411143997612360);
    assert(round_double(val).to_bits() == 13830554455654793216);
}

#[test]
fn k2_roundfunc_104() {
    //  Invoke on 0.55, type xs:double.
    let val = XsdDouble::from_bits(4603129179135383962);
    assert(round_double(val).to_bits() == 4607182418800017408);
}

#[test]
fn k2_roundfunc_106() {
    //  Invoke on -0.55, type xs:double.
    let val = XsdDouble::from_bits(13826501215990159770);
    assert(round_double(val).to_bits() == 13830554455654793216);
}

#[test]
fn k2_roundfunc_108() {
    //  Invoke on 0.56, type xs:double.
    let val = XsdDouble::from_bits(4603219251127931372);
    assert(round_double(val).to_bits() == 4607182418800017408);
}

#[test]
fn k2_roundfunc_110() {
    //  Invoke on -0.56, type xs:double.
    let val = XsdDouble::from_bits(13826591287982707180);
    assert(round_double(val).to_bits() == 13830554455654793216);
}

#[test]
fn k2_roundfunc_112() {
    //  Invoke on 0.59, type xs:double.
    let val = XsdDouble::from_bits(4603489467105573601);
    assert(round_double(val).to_bits() == 4607182418800017408);
}

#[test]
fn k2_roundfunc_114() {
    //  Invoke on -0.59, type xs:double.
    let val = XsdDouble::from_bits(13826861503960349409);
    assert(round_double(val).to_bits() == 13830554455654793216);
}

#[test]
fn k2_roundfunc_116() {
    //  Invoke on 0.50, type xs:double.
    let val = XsdDouble::from_bits(4602678819172646912);
    assert(round_double(val).to_bits() == 4607182418800017408);
}
