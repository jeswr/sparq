//! Test chunk 1 for fn:round-float
//! Contains 38 tests

use dep::xpath::{round_float, XsdFloat};

#[test]
fn k2_roundfunc_115() {
    //  Invoke on 0.50, type xs:float.
    let val = XsdFloat::from_bits(1056964608);
    assert(round_float(val).to_bits() == 1065353216);
}

#[test]
fn k2_roundfunc_117() {
    //  Invoke on -0.50, type xs:float.
    let val = XsdFloat::from_bits(3204448256);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_119() {
    //  Invoke on 0.61, type xs:float.
    let val = XsdFloat::from_bits(1058810102);
    assert(round_float(val).to_bits() == 1065353216);
}

#[test]
fn k2_roundfunc_121() {
    //  Invoke on -0.61, type xs:float.
    let val = XsdFloat::from_bits(3206293750);
    assert(round_float(val).to_bits() == 3212836864);
}

#[test]
fn k2_roundfunc_123() {
    //  Invoke on 0.64, type xs:float.
    let val = XsdFloat::from_bits(1059313418);
    assert(round_float(val).to_bits() == 1065353216);
}

#[test]
fn k2_roundfunc_125() {
    //  Invoke on -0.64, type xs:float.
    let val = XsdFloat::from_bits(3206797066);
    assert(round_float(val).to_bits() == 3212836864);
}

#[test]
fn k2_roundfunc_127() {
    //  Invoke on 0.65, type xs:float.
    let val = XsdFloat::from_bits(1059481190);
    assert(round_float(val).to_bits() == 1065353216);
}

#[test]
fn k2_roundfunc_129() {
    //  Invoke on -0.65, type xs:float.
    let val = XsdFloat::from_bits(3206964838);
    assert(round_float(val).to_bits() == 3212836864);
}

#[test]
fn k2_roundfunc_131() {
    //  Invoke on 0.66, type xs:float.
    let val = XsdFloat::from_bits(1059648963);
    assert(round_float(val).to_bits() == 1065353216);
}

#[test]
fn k2_roundfunc_133() {
    //  Invoke on -0.66, type xs:float.
    let val = XsdFloat::from_bits(3207132611);
    assert(round_float(val).to_bits() == 3212836864);
}

#[test]
fn k2_roundfunc_135() {
    //  Invoke on 0.69, type xs:float.
    let val = XsdFloat::from_bits(1060152279);
    assert(round_float(val).to_bits() == 1065353216);
}

#[test]
fn k2_roundfunc_137() {
    //  Invoke on -0.69, type xs:float.
    let val = XsdFloat::from_bits(3207635927);
    assert(round_float(val).to_bits() == 3212836864);
}

#[test]
fn k2_roundfunc_139() {
    //  Invoke on 0.60, type xs:float.
    let val = XsdFloat::from_bits(1058642330);
    assert(round_float(val).to_bits() == 1065353216);
}

#[test]
fn k2_roundfunc_141() {
    //  Invoke on -0.60, type xs:float.
    let val = XsdFloat::from_bits(3206125978);
    assert(round_float(val).to_bits() == 3212836864);
}

#[test]
fn k2_roundfunc_143() {
    //  Invoke on 0.91, type xs:float.
    let val = XsdFloat::from_bits(1063843267);
    assert(round_float(val).to_bits() == 1065353216);
}

#[test]
fn k2_roundfunc_145() {
    //  Invoke on -0.91, type xs:float.
    let val = XsdFloat::from_bits(3211326915);
    assert(round_float(val).to_bits() == 3212836864);
}

#[test]
fn k2_roundfunc_147() {
    //  Invoke on 0.94, type xs:float.
    let val = XsdFloat::from_bits(1064346583);
    assert(round_float(val).to_bits() == 1065353216);
}

#[test]
fn k2_roundfunc_149() {
    //  Invoke on -0.94, type xs:float.
    let val = XsdFloat::from_bits(3211830231);
    assert(round_float(val).to_bits() == 3212836864);
}

#[test]
fn k2_roundfunc_151() {
    //  Invoke on 0.95, type xs:float.
    let val = XsdFloat::from_bits(1064514355);
    assert(round_float(val).to_bits() == 1065353216);
}

#[test]
fn k2_roundfunc_153() {
    //  Invoke on -0.95, type xs:float.
    let val = XsdFloat::from_bits(3211998003);
    assert(round_float(val).to_bits() == 3212836864);
}

#[test]
fn k2_roundfunc_155() {
    //  Invoke on 0.96, type xs:float.
    let val = XsdFloat::from_bits(1064682127);
    assert(round_float(val).to_bits() == 1065353216);
}

#[test]
fn k2_roundfunc_157() {
    //  Invoke on -0.96, type xs:float.
    let val = XsdFloat::from_bits(3212165775);
    assert(round_float(val).to_bits() == 3212836864);
}

#[test]
fn k2_roundfunc_159() {
    //  Invoke on 0.99, type xs:float.
    let val = XsdFloat::from_bits(1065185444);
    assert(round_float(val).to_bits() == 1065353216);
}

#[test]
fn k2_roundfunc_161() {
    //  Invoke on -0.99, type xs:float.
    let val = XsdFloat::from_bits(3212669092);
    assert(round_float(val).to_bits() == 3212836864);
}

#[test]
fn k2_roundfunc_163() {
    //  Invoke on 0.90, type xs:float.
    let val = XsdFloat::from_bits(1063675494);
    assert(round_float(val).to_bits() == 1065353216);
}

#[test]
fn k2_roundfunc_165() {
    //  Invoke on -0.90, type xs:float.
    let val = XsdFloat::from_bits(3211159142);
    assert(round_float(val).to_bits() == 3212836864);
}

#[test]
fn k2_roundfunc_167() {
    //  Invoke on 0.101, type xs:float.
    let val = XsdFloat::from_bits(1036966167);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_169() {
    //  Invoke on -0.101, type xs:float.
    let val = XsdFloat::from_bits(3184449815);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_171() {
    //  Invoke on 0.104, type xs:float.
    let val = XsdFloat::from_bits(1037368820);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_173() {
    //  Invoke on -0.104, type xs:float.
    let val = XsdFloat::from_bits(3184852468);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_175() {
    //  Invoke on 0.105, type xs:float.
    let val = XsdFloat::from_bits(1037503037);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_177() {
    //  Invoke on -0.105, type xs:float.
    let val = XsdFloat::from_bits(3184986685);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_179() {
    //  Invoke on 0.106, type xs:float.
    let val = XsdFloat::from_bits(1037637255);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_181() {
    //  Invoke on -0.106, type xs:float.
    let val = XsdFloat::from_bits(3185120903);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_183() {
    //  Invoke on 0.109, type xs:float.
    let val = XsdFloat::from_bits(1038039908);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_185() {
    //  Invoke on -0.109, type xs:float.
    let val = XsdFloat::from_bits(3185523556);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_187() {
    //  Invoke on 0.100, type xs:float.
    let val = XsdFloat::from_bits(1036831949);
    assert(round_float(val) == XsdFloat::zero());
}

#[test]
fn k2_roundfunc_189() {
    //  Invoke on -0.100, type xs:float.
    let val = XsdFloat::from_bits(3184315597);
    assert(round_float(val) == XsdFloat::zero());
}
