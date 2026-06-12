# Vendored: noir_IEEE754 (ieee754 package)

- **Upstream:** https://github.com/jeswr/noir_IEEE754, subdirectory `ieee754/`
- **Upstream ref:** tag `v0.3.1` (commit `8275f3164aed60977661b00ceb36d5a8fc9b7cf2`)
  — the exact version `xpath` was pinned to upstream.
- **Vendored on:** 2026-06-12. LICENSE copied from the upstream repo root.

## Why vendored instead of pinned

The installed Noir toolchain (`nargo 1.0.0-beta.21`) removed the `u1` type.
Every upstream ref of noir_IEEE754 (tags v0.1.0–v0.9.0, `main`, and all
`ci/noir-*` branches as of 2026-06-12) still uses `u1` in the old
free-function API, so no tag/branch pin can compile. nargo git dependencies
cannot be pinned to commit SHAs, only tags/branches. Vendoring v0.3.1 with a
minimal compile fix was the last resort.

## Local changes vs upstream v0.3.1

Single mechanical substitution: `u1` -> `u8` (17 sites across `src/types.nr`,
`src/utils.nr`, `src/float32/{add,div,mul,helpers}.nr`,
`src/float64/{add,div,mul,helpers}.nr`). All `u1` values were sign/sticky bits
constrained to {0,1} (masked with `& 1` or assigned 0/1), used only with
integer ops (`^`, `==`, shifts, widening casts) — no wrapping, bitwise-not, or
ordering idioms — so widening to `u8` preserves semantics exactly. No other
changes.

## Planned follow-up

This vendored copy exists only to keep the old free-function float API
working. A later deliverable migrates `xpath` to the vendored `sparq_ieee754`
library at `../../../ieee754` (struct Float API; reference half-migration:
`jeswr/zkp-sparql-workspace:circuits/noir_XPath@refactor/new-ieee754-api`),
after which this directory should be deleted.
