# sparq-wac-oracle

Server-independent **WAC/ACP decision test-vectors** — embedded fixture pods
(N-Quads) plus spec-declared `(session, resource, mode) → {allow, granted_modes,
scope, status}` rows — and a runner asserting `sparq-solid`'s per-resource decision
surface (`PodStore::decide` / `decide_batch`) matches every row (bead `sq-t999y`).

**Internal tooling — not published** (`publish = false`): nothing in the shipping
graph depends on it; sparq-core/sparq-engine are untouched. **EXPERIMENTAL** — rows
and fixtures may grow; the row schema is stable-intent. This is the executable
oracle the PSS-parity suite and the future `solid-server` landing (epic `sq-gg0qq`)
assert against.

## 🚀 Quickstart

```rust
// Every embedded fixture row must hold on the real decision engine.
for fixture in sparq_wac_oracle::fixtures() {
    let report = sparq_wac_oracle::run_fixture(fixture)?;
    assert!(report.passed(), "{report}");
}
# Ok::<(), String>(())
```

Or drive a store you built yourself (any pod dataset + materializer):

```rust
use sparq_wac_oracle::{build_store, fixtures, run_vectors};

let fixture = &fixtures()[0]; // "wac"
let store = build_store(fixture)?;
let report = run_vectors(&store, fixture.vectors);
assert!(report.passed(), "{report}");
# Ok::<(), String>(())
```

## ✨ Features

- **Protocol-agnostic corpus** — the vectors are plain data (`Vector` /
  `Expected`), so a Solid HTTP server implementation can assert the same rows
  against its own request path, independent of sparq.
- **Two embedded pods** — a WAC `.acl` fixture (nearest-ACL shadowing, `accessTo`
  vs `default` scope, agent / group / public / authenticated / origin-pair
  subjects) and an ACP `.acr` fixture (cumulative inheritance, `allOf`
  agent+client pairs, deny-overrides, `noneOf` conditional grants), both derived
  from the `sparq_solid::fixture` seed's policy-shape subtrees.
- **Fail-closed rows included** — no-ACL (`NoAcl`) and malformed-IRI (`Transient`)
  requests are part of the corpus, so a consumer cannot pass while failing open.
- **Batch parity checked** — `decide_batch` must return element-for-element the
  same decisions as singleton `decide` calls; divergence is a reported failure.
- **Structured reports, no panics** — `run_vectors` returns an `OracleReport`
  with a field-level expected-vs-got diff per failing row.

No cargo features; the crate is opt-in by being a leaf `publish = false`
workspace member that nothing depends on.

## 📚 Learn more

- API docs: `cargo doc -p sparq-wac-oracle --open` (module docs state the
  result-equivalence invariant).
- The decision surface under test: `crates/sparq-solid` (`PodStore::decide`,
  `WacDecision`, `AclScope`, `AclStatus`) and `skills/access-control/SKILL.md`.
- The seed fixtures + semantics tables: `crates/sparq-solid/src/fixture.rs`,
  `crates/sparq-solid/tests/{wac,acp,decide}.rs`.
- Related by-construction generator (no engine link): `crates/sparq-acbench`.

## License

MIT — see the workspace root `LICENSE`.
