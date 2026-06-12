//! Test chunk 0 for op:add-dayTimeDurations
//! Contains 6 tests

use dep::xpath::{duration_add, duration_equal, duration_from_microseconds};

#[test]
fn op_add_daytimedurations2args_1() {
    //  ******************************************************* Test:
    let dur1 = duration_from_microseconds(0);
    let dur2 = duration_from_microseconds(0);
    assert(duration_equal(duration_add(dur1, dur2), duration_from_microseconds(0)));
}

#[test]
fn op_add_daytimedurations2args_2() {
    //  ******************************************************* Test:
    let dur1 = duration_from_microseconds(1339199000000);
    let dur2 = duration_from_microseconds(0);
    assert(
        duration_equal(
            duration_add(dur1, dur2),
            duration_from_microseconds(1339199000000),
        ),
    );
}

#[test]
fn op_add_daytimedurations2args_3() {
    //  ******************************************************* Test:
    let dur1 = duration_from_microseconds(2764799000000);
    let dur2 = duration_from_microseconds(0);
    assert(
        duration_equal(
            duration_add(dur1, dur2),
            duration_from_microseconds(2764799000000),
        ),
    );
}

#[test]
fn op_add_daytimedurations2args_4() {
    //  ******************************************************* Test:
    let dur1 = duration_from_microseconds(0);
    let dur2 = duration_from_microseconds(1339199000000);
    assert(
        duration_equal(
            duration_add(dur1, dur2),
            duration_from_microseconds(1339199000000),
        ),
    );
}

#[test]
fn op_add_daytimedurations2args_5() {
    //  ******************************************************* Test:
    let dur1 = duration_from_microseconds(0);
    let dur2 = duration_from_microseconds(2764799000000);
    assert(
        duration_equal(
            duration_add(dur1, dur2),
            duration_from_microseconds(2764799000000),
        ),
    );
}

#[test]
fn op_add_daytimedurations_1() {
    //  ******************************************************* Test:
    let dur1 = duration_from_microseconds(216300000000);
    let dur2 = duration_from_microseconds(475200000000);
    assert(
        duration_equal(
            duration_add(dur1, dur2),
            duration_from_microseconds(691500000000),
        ),
    );
}
