# Vendored: noir_json_parser

- **Upstream:** https://github.com/noir-lang/noir_json_parser
- **Upstream ref:** `main` @ commit `695b25add4a3229a5808ec0a0d40089c6cecfa60`
  (2026-05-27, "chore: Update noir_sort dependency to version 0.4.0 (#100)")
- **Vendored on:** 2026-06-12, **unmodified** (tracked files; `.github/` dropped).

## Why vendored instead of pinned

`xpath` upstream pinned this dep to the FLOATING `tag = "main"`:

- the latest released tag (v0.4.0) is years behind and fails on
  `nargo 1.0.0-beta.21` with hundreds of errors (Field indexing, `u1`,
  visibility);
- current `main` HEAD compiles cleanly on beta.21, but nargo cannot pin git
  deps to commit SHAs, and `tag = "main"` both drifts upstream and is cached
  forever by nargo (`~/nargo/.../noir_json_parser/main` holds a stale,
  beta.21-broken snapshot that nargo never refreshes).

Vendoring the known-good commit is the only way to get a true pin.

## Remaining git dependencies (fixed tags, resolve fine on beta.21)

- `noir_sort = noir-lang/noir_sort@v0.4.0`
- `poseidon = noir-lang/poseidon@v0.3.0`
