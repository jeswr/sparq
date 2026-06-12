use sparq_ieee754::f16;

#[test]
fn parts_conversion_methods_are_not_public_api() {
    let half: f16 = f16::new(0);
    let _parts = half.to_parts();
}