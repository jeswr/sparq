//! placeholder — wired in a later commit.

use super::report::TestResult;
use std::path::Path;

pub fn notes() -> Vec<String> {
    Vec::new()
}

pub fn run_suite(_root: &Path, _out: &mut Vec<TestResult>) -> Result<(), String> {
    Err("not wired yet".into())
}
