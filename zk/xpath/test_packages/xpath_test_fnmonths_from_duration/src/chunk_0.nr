//! Test chunk 0 for fn:months-from-duration
//! Contains 1 tests

use dep::xpath::{duration_from_microseconds, months_from_duration};

#[test]
fn cbcl_months_from_duration_001() {
    //  test fn:months-from-duration on xs:dayTimeDuration
    let dur = duration_from_microseconds(86400000000);
    assert(months_from_duration(dur) == 0);
}
