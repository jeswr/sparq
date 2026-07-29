<!-- internal-stub -->

# sparq-wrapper-shacl

Internal SHACL integration seam for `sparq-wrapper`. With the opt-in `oo-models`
feature it lowers a `sparq_shacl::ShapesModel` to a sorted object-model IR
(`ModelSchema`) and emits it as std-only Rust: `sh:minCount`/`sh:maxCount` become
`Option<T>` / `T` / `Vec<T>`, `sh:datatype` a checked scalar, `sh:class` a typed
reference, `sh:node` a nested struct, and `sh:closed` a predicate whitelist the
generated loader enforces. Both stages are deterministic; a shapes graph that
cannot be modelled faithfully is a typed `SchemaError`. The SHACL parser is
reused from `sparq-shacl`, never re-implemented. With the feature off the crate
exposes no API and pulls in no dependencies.

Licensed under the workspace licence. <!-- [FABLE-5] sq-1rg2q.12 -->
