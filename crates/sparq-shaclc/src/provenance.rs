//! GENERATED provenance pins — written by `scripts/regen-shuttle-parsers.sh`.
//! DO NOT EDIT by hand: regenerate alongside the artifacts in `src/raw/`.

/// The `jeswr/rdf-shuttle` commit whose `gen-rs` backend generated the
/// artifacts in `src/raw/` (and whose grammar is vendored in `grammar/`).
pub const RDF_SHUTTLE_COMMIT: &str = "20062110dc80752e306724be7d7a984b7702e7a9";

/// SHA-256 of the vendored `grammar/shaclc12ext.shuttle` at generation time.
/// `tests/provenance_drift.rs` recomputes this from the vendored file, so a
/// grammar edit without regeneration turns the suite red.
pub const GRAMMAR_SHA256: &str = "3e6bceb2210fd3d98a03e240a37b84740bc66a1b264660dc1840b9f7d15afff0";

/// The gen-rs CLI invocations that produced the two artifacts.
pub const REGEN_COMMANDS: &[&str] = &[
    "shuttle-gen-rs grammars/shaclc12ext.shuttle -o shaclc12.rs --profile rdf12",
    "shuttle-gen-rs grammars/shaclc12ext.shuttle -o shaclc12ext.rs --profile rdf12,ext",
];
