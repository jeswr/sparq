<!-- [SONNET-4.6] sq-gg0qq.6 — vendored jeswr/lws-spec conformance corpus. -->
# Vendored `jeswr/lws-spec` conformance corpus

The Linked Web Storage (LWS) contract this crate is measured against is
[`jeswr/lws-spec`](https://github.com/jeswr/lws-spec), not this repository.
**Where the crate and the spec disagree, the SPEC WINS** — the fix belongs in
`crates/sparq-lws-core`, or, when the spec itself looks wrong, in a change
proposal raised on `lws-spec` for review there. Never "fix" a vector locally to
make a test pass; a vector edit is a spec change and does not belong in this
repo.

## Provenance (pinned)

| Field | Value |
| --- | --- |
| Upstream | `https://github.com/jeswr/lws-spec` |
| Pinned commit | `ffaea0497de41cd709a742e0c4a90831a500fd97` |
| Commit date | 2026-07-12 |
| `manifest.json` `specSource` | `lws-spec@59da847` (the spec revision the generator ran against) |
| Cases vendored | 157 across 10 suites (asserted by `tests/lws_spec_vectors.rs`) |

Vendored subtrees, copied verbatim with no local edits:

- `test-vectors/manifest.json` + `test-vectors/vectors/**` — the suite index and
  the 157 language-neutral cases.
- `semantics/access-decision.n3` + `semantics/access-decision.query.n3` — the
  normative, executable access-decision rule set for the strict ODRL access
  profile `https://w3id.org/jeswr/lws/access-profile/odrl-1`. This file **is**
  the definition of the `evaluate-access` decision function that
  `src/authz/access_profile.rs` ports to Rust.

Upstream's `test-vectors/README.md`, `test-vectors/GAPS.md`, `shapes/`,
`test-suite/`, and the vector generator under `test-vectors/tools/` are **not**
vendored — read them upstream at the pinned commit. `tools/` in particular
carries TEST-ONLY private JWKs that this repo has no use for.

> The `test-vectors/vectors/*/keyring/` fixtures that ARE vendored (public JWKS,
> pre-signed JWTs, and a symmetric session record) are TEST-ONLY conformance
> fixture material published upstream. They are inputs to the suite and must
> never be used as key material anywhere else.

## Decision: vendored fixtures, not a git submodule

Vendored. The workspace uses no git submodules anywhere else, and the vector
corpus is a small set of static JSON. Vendoring buys:

- **Hermetic CI.** `cargo test -p sparq-lws-core --features access-profile-odrl1`
  needs no network, no `git submodule update`, and no second checkout step in
  every workflow that touches this crate.
- **A reviewable pin bump.** Refreshing the corpus shows up as a normal diff, so
  a vector whose expected verdict changed is visible in review rather than
  hidden behind a moved submodule pointer.
- **No build-time dependency on an external host.** A `lws-spec` outage cannot
  turn this crate's gate red.

The cost is that the pin must be advanced deliberately — see below.

## Refreshing the pin

```sh
crates/sparq-lws-core/lws-spec/vendor.sh <upstream-commit-sha>
```

The script re-clones upstream at the given commit and replaces the vendored
subtrees wholesale, then prints the new pin so this README's table can be
updated in the same commit. Expect `tests/lws_spec_vectors.rs` to fail after a
refresh that changes the corpus: the suite asserts the case count and the
per-operation coverage ledger in `coverage-baseline.json`, so any added,
removed, or re-classified vector must be acknowledged explicitly.

## Coverage today

`tests/lws_spec_vectors.rs` walks every vendored case and dispatches on the
case's `operation`. Only `evaluate-access` (19 cases in the `access-grants`
suite) is reproduced in Rust today, by
[`authz::access_profile`](../src/authz/access_profile.rs); the other 138 cases
are enumerated as *pending* in `coverage-baseline.json` rather than silently
skipped, and the suite fails if that ledger drifts in either direction. The
74 `http-exchange` cases need a live server harness and are the natural next
slice.

## The N3 oracle lane (opt-in, not wired into CI)

The upstream oracle executes `semantics/access-decision.n3` under an N3
reasoner (EYE / `eyereasoner`) over the same vectors:

```sh
git clone https://github.com/jeswr/lws-spec && cd lws-spec
git checkout ffaea0497de41cd709a742e0c4a90831a500fd97
npm ci --prefix test-suite
node test-suite/tools/oracle-access.mjs
```

That needs Node plus the `eyereasoner` package, so it is **not** part of this
crate's `cargo test` gate. The Rust port is checked against the *vectors*, which
upstream's oracle in turn checks against the *rule set* — so the rule set stays
the authority without this repo taking on a Node/EYE build dependency. Wiring
the re-derivation as its own opt-in workflow lane is tracked separately.
