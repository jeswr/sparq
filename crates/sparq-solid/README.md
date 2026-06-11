# sparq-solid

Solid Pod access control over the sparq engine — pods stored as **named graph per
document**, WAC/ACP access-control documents stored **as plain triples** (named graphs
too), their semantics encoded as **N3 rules** (`rules/*.n3`, run by `sparq-reason`)
that materialize a queryable authorization view in `<urn:sparq:auth>`, and queries
filtered per (WebID, client) session to the authorized graph set.

Design + measured v1 baseline: `research/solid-access-control-design.md`.

```rust
let graph = sparq_core::Graph::load_dataset(&nquads, "nquads")?;
let mut store = sparq_solid::PodStore::new(graph);
store.materialize_wac()?;                       // or materialize_acp()
let session = sparq_solid::Session { agent: Some("https://alice.ex/card#me"), client: None };
let result = store.query_as(&session, sparq_solid::Mode::Read,
    "SELECT ?title WHERE { ?s <https://ex.dev/ns#title> ?title }")?;
```

Benchmark: `cargo run -p sparq-solid --example bench --release`.
This crate is a dependency of nothing in the workspace; the engine carries zero
Solid-specific code (the generic zero-copy dataset view is a specified follow-up —
design doc §5).
