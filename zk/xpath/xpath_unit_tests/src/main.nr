//! Unit tests for xpath library

fn main() {
    // This is a test binary - tests are run via `nargo test`
    println("xpath_unit_tests - run with `nargo test`");
}

mod numeric_tests;
mod datetime_tests;
mod date_tests;
mod boolean_tests;
mod string_tests;
// [OPUS-4.8] sq-y73: bounded regex subset + circuit-native hash coverage.
mod regex_tests;
mod hash_tests;
mod qname_tests;
mod sequence_tests;
mod time_tests;
mod year_month_duration_tests;
mod binary_tests;
mod gregorian_tests;
mod comparison_tests;
