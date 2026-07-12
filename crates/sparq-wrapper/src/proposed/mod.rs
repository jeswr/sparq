// [SONNET-4.6] sq-1rg2q.2: proposed extensions submodule.
//
// Each submodule in this tree corresponds to one still-unlanded rdfjs/wrapper
// proposal and is gated on its own cargo feature so callers opt in precisely.

/// Typed focus kinds and bound node factories (rdfjs/wrapper PRs #83-#87).
///
/// Enable with `features = ["proposed-focus-kinds"]`.
pub mod typed_focus;
