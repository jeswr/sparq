<!-- internal-stub -->
# sparq-conformance-floors

Shared, zero-dependency ratchet-floor constants (bead `sq-z1xv8`).

A conformance/extension floor enforced in one crate's test runner and reported in
`sparq-conformance`'s central `scoreboard::SUITES` used to be spelled twice and kept
in step by reading the sibling crate's test *source* at a runtime-built workspace
path. That was both a drift risk and CI test-selection "residual 3"
(`scripts/ci_audit_inputs.py`; design `research/change-based-test-selection.md`
§4.2): a statically unresolvable out-of-crate read that no dependency closure or
`ci/path-ownership.toml` `readers` entry could attribute.

Each floor now lives here once. Runners take a `dev-dependency` (shipping graph
untouched); `sparq-conformance` takes a plain dependency. Both read the same
`const`, and the cargo edges put every enforcing crate in this crate's
reverse-dependency closure, so the selector cannot skip their lanes.

Floors may only **RISE**; measurement narrative stays with the runner. See
`src/lib.rs` and `sparq-conformance/tests/scoreboard_floors.rs`.

**Internal tooling — not published** (`publish = false`).

License: MIT
