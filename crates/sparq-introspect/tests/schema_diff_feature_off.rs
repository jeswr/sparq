#![cfg(not(feature = "schema-diff"))]

#[test]
fn schema_diff_module_and_reexports_stay_feature_gated() {
    // The feature-off build cannot refer to gated types to prove their absence at
    // runtime. Pin the public module graph instead: removing either cfg makes this
    // guard fail before a default build can leak the schema-diff API or code.
    let lines: Vec<&str> = include_str!("../src/lib.rs").lines().collect();
    for declaration in ["mod diff;", "pub use diff::{DiffEntry, SchemaDiff};"] {
        let index = lines.iter().position(|line| *line == declaration).unwrap();
        assert_eq!(lines[index - 1], "#[cfg(feature = \"schema-diff\")]");
    }
}
