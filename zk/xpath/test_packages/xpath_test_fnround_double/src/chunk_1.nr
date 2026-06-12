//! Test chunk 1 for fn:round-double
//! Contains 37 tests

use dep::xpath::{round_double, XsdDouble};

#[test]
fn k2_roundfunc_118() {
    //  Invoke on -0.50, type xs:double.
    let val = XsdDouble::from_bits(13826050856027422720);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_120() {
    //  Invoke on 0.61, type xs:double.
    let val = XsdDouble::from_bits(4603669611090668421);
    assert(round_double(val).to_bits() == 4607182418800017408);
}

#[test]
fn k2_roundfunc_122() {
    //  Invoke on -0.61, type xs:double.
    let val = XsdDouble::from_bits(13827041647945444229);
    assert(round_double(val).to_bits() == 13830554455654793216);
}

#[test]
fn k2_roundfunc_124() {
    //  Invoke on 0.64, type xs:double.
    let val = XsdDouble::from_bits(4603939827068310651);
    assert(round_double(val).to_bits() == 4607182418800017408);
}

#[test]
fn k2_roundfunc_126() {
    //  Invoke on -0.64, type xs:double.
    let val = XsdDouble::from_bits(13827311863923086459);
    assert(round_double(val).to_bits() == 13830554455654793216);
}

#[test]
fn k2_roundfunc_128() {
    //  Invoke on 0.65, type xs:double.
    let val = XsdDouble::from_bits(4604029899060858061);
    assert(round_double(val).to_bits() == 4607182418800017408);
}

#[test]
fn k2_roundfunc_130() {
    //  Invoke on -0.65, type xs:double.
    let val = XsdDouble::from_bits(13827401935915633869);
    assert(round_double(val).to_bits() == 13830554455654793216);
}

#[test]
fn k2_roundfunc_132() {
    //  Invoke on 0.66, type xs:double.
    let val = XsdDouble::from_bits(4604119971053405471);
    assert(round_double(val).to_bits() == 4607182418800017408);
}

#[test]
fn k2_roundfunc_134() {
    //  Invoke on -0.66, type xs:double.
    let val = XsdDouble::from_bits(13827492007908181279);
    assert(round_double(val).to_bits() == 13830554455654793216);
}

#[test]
fn k2_roundfunc_136() {
    //  Invoke on 0.69, type xs:double.
    let val = XsdDouble::from_bits(4604390187031047700);
    assert(round_double(val).to_bits() == 4607182418800017408);
}

#[test]
fn k2_roundfunc_138() {
    //  Invoke on -0.69, type xs:double.
    let val = XsdDouble::from_bits(13827762223885823508);
    assert(round_double(val).to_bits() == 13830554455654793216);
}

#[test]
fn k2_roundfunc_140() {
    //  Invoke on 0.60, type xs:double.
    let val = XsdDouble::from_bits(4603579539098121011);
    assert(round_double(val).to_bits() == 4607182418800017408);
}

#[test]
fn k2_roundfunc_142() {
    //  Invoke on -0.60, type xs:double.
    let val = XsdDouble::from_bits(13826951575952896819);
    assert(round_double(val).to_bits() == 13830554455654793216);
}

#[test]
fn k2_roundfunc_144() {
    //  Invoke on 0.91, type xs:double.
    let val = XsdDouble::from_bits(4606371770867090719);
    assert(round_double(val).to_bits() == 4607182418800017408);
}

#[test]
fn k2_roundfunc_146() {
    //  Invoke on -0.91, type xs:double.
    let val = XsdDouble::from_bits(13829743807721866527);
    assert(round_double(val).to_bits() == 13830554455654793216);
}

#[test]
fn k2_roundfunc_148() {
    //  Invoke on 0.94, type xs:double.
    let val = XsdDouble::from_bits(4606641986844732948);
    assert(round_double(val).to_bits() == 4607182418800017408);
}

#[test]
fn k2_roundfunc_150() {
    //  Invoke on -0.94, type xs:double.
    let val = XsdDouble::from_bits(13830014023699508756);
    assert(round_double(val).to_bits() == 13830554455654793216);
}

#[test]
fn k2_roundfunc_152() {
    //  Invoke on 0.95, type xs:double.
    let val = XsdDouble::from_bits(4606732058837280358);
    assert(round_double(val).to_bits() == 4607182418800017408);
}

#[test]
fn k2_roundfunc_154() {
    //  Invoke on -0.95, type xs:double.
    let val = XsdDouble::from_bits(13830104095692056166);
    assert(round_double(val).to_bits() == 13830554455654793216);
}

#[test]
fn k2_roundfunc_156() {
    //  Invoke on 0.96, type xs:double.
    let val = XsdDouble::from_bits(4606822130829827768);
    assert(round_double(val).to_bits() == 4607182418800017408);
}

#[test]
fn k2_roundfunc_158() {
    //  Invoke on -0.96, type xs:double.
    let val = XsdDouble::from_bits(13830194167684603576);
    assert(round_double(val).to_bits() == 13830554455654793216);
}

#[test]
fn k2_roundfunc_160() {
    //  Invoke on 0.99, type xs:double.
    let val = XsdDouble::from_bits(4607092346807469998);
    assert(round_double(val).to_bits() == 4607182418800017408);
}

#[test]
fn k2_roundfunc_162() {
    //  Invoke on -0.99, type xs:double.
    let val = XsdDouble::from_bits(13830464383662245806);
    assert(round_double(val).to_bits() == 13830554455654793216);
}

#[test]
fn k2_roundfunc_164() {
    //  Invoke on 0.90, type xs:double.
    let val = XsdDouble::from_bits(4606281698874543309);
    assert(round_double(val).to_bits() == 4607182418800017408);
}

#[test]
fn k2_roundfunc_166() {
    //  Invoke on -0.90, type xs:double.
    let val = XsdDouble::from_bits(13829653735729319117);
    assert(round_double(val).to_bits() == 13830554455654793216);
}

#[test]
fn k2_roundfunc_168() {
    //  Invoke on 0.101, type xs:double.
    let val = XsdDouble::from_bits(4591942237660995650);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_170() {
    //  Invoke on -0.101, type xs:double.
    let val = XsdDouble::from_bits(13815314274515771458);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_172() {
    //  Invoke on 0.104, type xs:double.
    let val = XsdDouble::from_bits(4592158410443109433);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_174() {
    //  Invoke on -0.104, type xs:double.
    let val = XsdDouble::from_bits(13815530447297885241);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_176() {
    //  Invoke on 0.105, type xs:double.
    let val = XsdDouble::from_bits(4592230468037147361);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_178() {
    //  Invoke on -0.105, type xs:double.
    let val = XsdDouble::from_bits(13815602504891923169);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_180() {
    //  Invoke on 0.106, type xs:double.
    let val = XsdDouble::from_bits(4592302525631185289);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_182() {
    //  Invoke on -0.106, type xs:double.
    let val = XsdDouble::from_bits(13815674562485961097);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_184() {
    //  Invoke on 0.109, type xs:double.
    let val = XsdDouble::from_bits(4592518698413299073);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_186() {
    //  Invoke on -0.109, type xs:double.
    let val = XsdDouble::from_bits(13815890735268074881);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_188() {
    //  Invoke on 0.100, type xs:double.
    let val = XsdDouble::from_bits(4591870180066957722);
    assert(round_double(val) == XsdDouble::zero());
}

#[test]
fn k2_roundfunc_190() {
    //  Invoke on -0.100, type xs:double.
    let val = XsdDouble::from_bits(13815242216921733530);
    assert(round_double(val) == XsdDouble::zero());
}
