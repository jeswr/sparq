//! Test chunk 0 for fn:years-from-duration
//! Contains 1 tests

use dep::xpath::{duration_from_microseconds, years_from_duration};

#[test]
fn cbcl_years_from_duration_001() {
    //  test fn:years-from-duration on xs:dayTimeDuration
    let dur = duration_from_microseconds(86400000000);
    assert(years_from_duration(dur) == 0);
}
