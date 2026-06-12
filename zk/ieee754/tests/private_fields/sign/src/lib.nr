use sparq_ieee754::f16;

#[test]
fn sign_field_is_not_public_api() {
    let half: f16 = f16::new(0);
    let _sign = half.sign;
}