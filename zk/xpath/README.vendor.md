# Vendored: noir_XPath

XPath 2.0 functions & operators in Noir (SPARQL FILTER semantics), vendored into
`sparq` as a subfolder for now (per Jesse: work with IEEE754 and XPath as
subfolders in this repository).

## Provenance

- **Upstream:** https://github.com/jeswr/noir_XPath, branch `main`
- **Upstream commit:** `fe88a5d1dec1d6400e9e6c7dc37876753441d85a` (2026-01-29)
- **Vendored on:** 2026-06-12 (tracked files only; `result.txt` test-log artifact dropped)
- **Layout:** `xpath` (lib, ~10.9k lines), `xpath_unit_tests` (bin), 241
  `test_packages/*` workspace members (FPgen/MPFR-oracle style, auto-generated
  by `scripts/generate_tests.py`). CI upstream runs `nargo test --workspace`.

## Local changes vs upstream (toolchain drift fixes only)

Upstream pins Noir `1.0.0-beta.16` (`.github/noir-versions.json`); this copy is
made to compile and test green on the installed `nargo 1.0.0-beta.21`, which
removed the `u1` type and broke both upstream git dependencies. Changes:

1. `xpath/Nargo.toml` — both git deps replaced with path deps on `vendor/`:
   - `ieee754` was `jeswr/noir_IEEE754@v0.3.1` (old free-function API:
     `add_float32`, `float64_lt`, `IEEE754Float32`, `ROUNDING_MODE_*`, ...).
     No upstream tag or branch of noir_IEEE754 compiles on beta.21 (all refs
     still use `u1`), and nargo cannot pin git deps to commit SHAs, so the
     exact v0.3.1 tree is vendored at `vendor/ieee754/` with a minimal
     mechanical `u1` -> `u8` substitution (17 sites). See
     `vendor/ieee754/VENDOR-PROVENANCE.md`.
   - `json_parser` was `noir-lang/noir_json_parser@main` — a FLOATING tag
     (drift hazard: the nargo cache holds a stale, beta.21-broken snapshot of
     `main`, and the released tags v0.1.0–v0.4.0 are far too old for beta.21).
     Vendored at `vendor/json_parser/` from upstream `main` commit
     `695b25add4a3229a5808ec0a0d40089c6cecfa60` (2026-05-27), unmodified. See
     `vendor/json_parser/VENDOR-PROVENANCE.md`.
2. No `.nr` source changes in `xpath/`, `xpath_unit_tests/`, or
   `test_packages/` — upstream sources compile on beta.21 as-is (warnings
   only).

## Planned follow-up (out of scope here)

- **Float API migration:** a later deliverable will migrate this copy off the
  old free-function IEEE754 API onto the vendored `sparq_ieee754` library at
  `../ieee754` (the `zk-ieee754` branch vendoring). A half-done reference
  migration exists at
  `jeswr/zkp-sparql-workspace:circuits/noir_XPath` branch
  `refactor/new-ieee754-api` (local checkout:
  `/Users/jesght/Documents/GitHub/jeswr/zkp-sparql-workspace/circuits/noir_XPath`)
  — use it as a reference, do not copy it wholesale.
- **Upstreaming:** the beta.21 drift fixes (and the dep-pinning story) should
  eventually be upstreamed to jeswr/noir_XPath.
