<!-- internal-stub -->

# sparq-wrapper-shacl

The SHACL seam for `sparq-wrapper`: turns a shapes graph into Rust `struct`s.
Shapes come in as a `sparq_shacl::ShapesModel` — this crate parses no Turtle and
re-implements no SHACL. `lower` produces a comparable IR, `emit` renders it as a
`std`-only module, `generate` runs both. Ill-formed or contradictory shapes are
typed `LoweringError`s, never a guess. See the crate rustdoc for the mapping
table and `skills/rdf-wrapper/SKILL.md` for how it fits the wrapper surface.
<!-- [FABLE-5] sq-1rg2q.12 -->
