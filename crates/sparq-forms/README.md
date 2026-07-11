# sparq-forms

Headless **SHACL-to-form derivation** for sparq: a pure function from
(data graph, shapes graph, focus node, mode) to a serde-JSON **`FormDescription`**
— DASH-compatible auto-generated data-entry/edit forms, with **no GUI
dependencies** (wasm-able by construction).

> Model: Claude Fable 5 [FABLE-5] (sq-lsp7k.1.1). Design record
> `research/competitive-feature-analysis-2026-07.md` §3.

## 🚀 Quickstart

```rust
use sparq_core::Graph;
use sparq_forms::{derive_form, FormOptions};
use oxrdf::{NamedNode, Term};

let shapes = Graph::load_str(r#"
  @prefix sh: <http://www.w3.org/ns/shacl#> .
  @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
  @prefix ex: <http://example.org/> .
  ex:PersonShape a sh:NodeShape ; sh:targetClass ex:Person ;
    sh:property [ sh:path ex:name ; sh:name "Name" ; sh:order 1 ;
                  sh:datatype xsd:string ; sh:minCount 1 ; sh:maxCount 1 ] .
"#, "turtle").unwrap();
let data = Graph::load_str(r#"
  @prefix ex: <http://example.org/> .
  ex:alice a ex:Person ; ex:name "Alice" .
"#, "turtle").unwrap();

let focus = Term::from(NamedNode::new("http://example.org/alice").unwrap());
let form = derive_form(&data, &shapes, &focus, &FormOptions::default());
let field = &form.groups[0].fields[0];
assert_eq!(field.label, "Name");
assert!(field.required && !field.multi);
assert_eq!(field.widget.editor.as_deref(),
           Some("http://datashapes.org/dash#TextFieldEditor"));
```

## ✨ Features

- **Shape selection** — `sh:targetNode`, `sh:targetClass` / implicit class
  targets (matched against the focus node's `rdf:type` with `rdfs:subClassOf`
  closure), `dash:applicableToClass`, plus an explicit override; every
  applicable shape lands in the description's shape-switcher list.
- **Field enumeration & layout** — one field per property shape: `sh:path`
  (including `sh:inversePath` incoming-reference fields), `sh:name` /
  `sh:description` with `rdfs:label`/`rdfs:comment` fallbacks, fractional
  `sh:order`, `sh:group` → `sh:PropertyGroup` sections, `sh:deactivated`
  suppression, and an implicit read-only **"Other properties"** group for
  off-shape triples.
- **DASH widget scoring registry** — the documented 0–100 suitability table
  (`sparq_forms::widgets` rustdoc) over the DASH editors/viewers (TextField /
  TextFieldWithLang / TextArea / BooleanSelect / Date+DateTimePicker /
  EnumSelect on `sh:in` / AutoComplete + InstancesSelect on `sh:class` /
  URIEditor / RichText on `rdf:HTML` / SubClassEditor / nested DetailsEditor /
  BlankNode fallback; Label/Image/Hyperlink/HTML/ValueTable viewers), with
  explicit `dash:editor` / `dash:viewer` overrides and runner-up alternatives.
- **Nested sub-forms** — `sh:node` values recurse into a nested
  `FormDescription` (`dash:DetailsEditor`), depth-limited and cycle-safe.
- **Constraints carried per field** — counts, datatypes, classes, node kinds,
  `sh:in` enumerations, pattern/length/range bounds, `sh:or` unions — the
  renderer's live-validation hints (F3) and the widget-scoring inputs.
- **Headless & opt-in** — consumes `sparq-shacl`'s shapes model; no GUI deps;
  builds for `wasm32-unknown-unknown`; nothing in the default workspace
  depends on it, so the engine core stays lean.

## 📚 Learn more

- [DASH form generation](https://datashapes.org/forms.html) — the widget
  vocabulary and scoring contract this crate implements (documented
  deviations are marked *(sparq)* in the `widgets` module docs).
- `sparq-shacl` — the shapes model + validation engine this crate consumes.
- Roadmap: F2 GUI renderer (sq-lsp7k.1.2), F3 live validation + DASH
  suggestions (sq-lsp7k.1.3), F5 RDF 1.2 / computed fields (sq-lsp7k.1.5),
  F6 sparq-mcp agent tools (sq-lsp7k.1.6).

## License

MIT — see the workspace root `LICENSE`.
