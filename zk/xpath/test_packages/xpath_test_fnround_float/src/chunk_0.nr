//! Test chunk 0 for fn:round-float
//! Contains 50 tests

use dep::xpath::{round_float, XsdFloat};

#[test]
fn fn_roundflt1args_1() {
    //  Evaluates The 'round' function with the arguments set as follows: $arg =
    let val = XsdFloat::from_bits(4286578687);
    assert(round_float(val).to_bits() == 4286578687);
}

#[test]
fn fn_roundflt1args_3() {
    //  Evaluates The 'round' function with the arguments set as follows: $arg =
    let val = XsdFloat::from_bits(2139095039);
    assert(round_float(val).to_bits() == 2139095039);
}

#[test]
fn k2_roundfunc_19() {
    //  Invoke on xs:float 0.
    let val = XsdFloat::from_bits(0);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_20() {
    //  Invoke on xs:float -0.
    let val = XsdFloat::from_bits(2147483648);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_23() {
    //  Invoke on 0.01, type xs:float.
    let val = XsdFloat::from_bits(1008981770);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_25() {
    //  Invoke on -0.01, type xs:float.
    let val = XsdFloat::from_bits(3156465418);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_27() {
    //  Invoke on 0.04, type xs:float.
    let val = XsdFloat::from_bits(1025758986);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_29() {
    //  Invoke on -0.04, type xs:float.
    let val = XsdFloat::from_bits(3173242634);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_31() {
    //  Invoke on 0.05, type xs:float.
    let val = XsdFloat::from_bits(1028443341);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_33() {
    //  Invoke on -0.05, type xs:float.
    let val = XsdFloat::from_bits(3175926989);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_35() {
    //  Invoke on 0.06, type xs:float.
    let val = XsdFloat::from_bits(1031127695);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_37() {
    //  Invoke on -0.06, type xs:float.
    let val = XsdFloat::from_bits(3178611343);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_39() {
    //  Invoke on 0.09, type xs:float.
    let val = XsdFloat::from_bits(1035489772);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_41() {
    //  Invoke on -0.09, type xs:float.
    let val = XsdFloat::from_bits(3182973420);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_43() {
    //  Invoke on 0.00, type xs:float.
    let val = XsdFloat::from_bits(0);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_45() {
    //  Invoke on -0.00, type xs:float.
    let val = XsdFloat::from_bits(2147483648);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_47() {
    //  Invoke on 0.11, type xs:float.
    let val = XsdFloat::from_bits(1038174126);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_49() {
    //  Invoke on -0.11, type xs:float.
    let val = XsdFloat::from_bits(3185657774);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_51() {
    //  Invoke on 0.14, type xs:float.
    let val = XsdFloat::from_bits(1041194025);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_53() {
    //  Invoke on -0.14, type xs:float.
    let val = XsdFloat::from_bits(3188677673);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_55() {
    //  Invoke on 0.15, type xs:float.
    let val = XsdFloat::from_bits(1041865114);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_57() {
    //  Invoke on -0.15, type xs:float.
    let val = XsdFloat::from_bits(3189348762);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_59() {
    //  Invoke on 0.16, type xs:float.
    let val = XsdFloat::from_bits(1042536202);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_61() {
    //  Invoke on -0.16, type xs:float.
    let val = XsdFloat::from_bits(3190019850);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_63() {
    //  Invoke on 0.19, type xs:float.
    let val = XsdFloat::from_bits(1044549468);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_65() {
    //  Invoke on -0.19, type xs:float.
    let val = XsdFloat::from_bits(3192033116);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_67() {
    //  Invoke on 0.10, type xs:float.
    let val = XsdFloat::from_bits(1036831949);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_69() {
    //  Invoke on -0.10, type xs:float.
    let val = XsdFloat::from_bits(3184315597);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_71() {
    //  Invoke on 0.41, type xs:float.
    let val = XsdFloat::from_bits(1053944709);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_73() {
    //  Invoke on -0.41, type xs:float.
    let val = XsdFloat::from_bits(3201428357);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_75() {
    //  Invoke on 0.44, type xs:float.
    let val = XsdFloat::from_bits(1054951342);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_77() {
    //  Invoke on -0.44, type xs:float.
    let val = XsdFloat::from_bits(3202434990);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_79() {
    //  Invoke on 0.45, type xs:float.
    let val = XsdFloat::from_bits(1055286886);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_81() {
    //  Invoke on -0.45, type xs:float.
    let val = XsdFloat::from_bits(3202770534);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_83() {
    //  Invoke on 0.46, type xs:float.
    let val = XsdFloat::from_bits(1055622431);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_85() {
    //  Invoke on -0.46, type xs:float.
    let val = XsdFloat::from_bits(3203106079);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_87() {
    //  Invoke on 0.49, type xs:float.
    let val = XsdFloat::from_bits(1056629064);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_89() {
    //  Invoke on -0.49, type xs:float.
    let val = XsdFloat::from_bits(3204112712);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_91() {
    //  Invoke on 0.40, type xs:float.
    let val = XsdFloat::from_bits(1053609165);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_93() {
    //  Invoke on -0.40, type xs:float.
    let val = XsdFloat::from_bits(3201092813);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_95() {
    //  Invoke on 0.51, type xs:float.
    let val = XsdFloat::from_bits(1057132380);
    assert(round_float(val).to_bits() == 1065353216);
}

#[test]
fn k2_roundfunc_97() {
    //  Invoke on -0.51, type xs:float.
    let val = XsdFloat::from_bits(3204616028);
    assert(round_float(val).to_bits() == 3212836864);
}

#[test]
fn k2_roundfunc_99() {
    //  Invoke on 0.54, type xs:float.
    let val = XsdFloat::from_bits(1057635697);
    assert(round_float(val).to_bits() == 1065353216);
}

#[test]
fn k2_roundfunc_101() {
    //  Invoke on -0.54, type xs:float.
    let val = XsdFloat::from_bits(3205119345);
    assert(round_float(val).to_bits() == 3212836864);
}

#[test]
fn k2_roundfunc_103() {
    //  Invoke on 0.55, type xs:float.
    let val = XsdFloat::from_bits(1057803469);
    assert(round_float(val).to_bits() == 1065353216);
}

#[test]
fn k2_roundfunc_105() {
    //  Invoke on -0.55, type xs:float.
    let val = XsdFloat::from_bits(3205287117);
    assert(round_float(val).to_bits() == 3212836864);
}

#[test]
fn k2_roundfunc_107() {
    //  Invoke on 0.56, type xs:float.
    let val = XsdFloat::from_bits(1057971241);
    assert(round_float(val).to_bits() == 1065353216);
}

#[test]
fn k2_roundfunc_109() {
    //  Invoke on -0.56, type xs:float.
    let val = XsdFloat::from_bits(3205454889);
    assert(round_float(val).to_bits() == 3212836864);
}

#[test]
fn k2_roundfunc_111() {
    //  Invoke on 0.59, type xs:float.
    let val = XsdFloat::from_bits(1058474557);
    assert(round_float(val).to_bits() == 1065353216);
}

#[test]
fn k2_roundfunc_113() {
    //  Invoke on -0.59, type xs:float.
    let val = XsdFloat::from_bits(3205958205);
    assert(round_float(val).to_bits() == 3212836864);
}
