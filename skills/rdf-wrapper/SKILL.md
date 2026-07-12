---
name: rdf-wrapper
description: "Traverse sparq RDF graphs as native Rust objects with the opt-in sparq-wrapper crate: bind a focus Term to an owned or borrowed Store, follow outgoing/incoming NamedNode predicates with iterators, unwrap values, convert typed literals to str/i64/bool, mutate owned stores, and optionally use the unlanded distinct-result, typed-cardinality, and typed-focus-kinds proposals. Use when Rust code should work with focus objects instead of raw triples or dictionary IDs; SHACL-to-Rust code generation is a later surface."
---

# Use sparq-wrapper

Add the opt-in crate explicitly:

```toml
[dependencies]
sparq-core = "0.1"
sparq-wrapper = "0.1"
oxrdf = "0.3"
```

Load a graph, borrow it, and traverse with typed predicates:

```rust
use oxrdf::NamedNode;
use sparq_core::Graph;
use sparq_wrapper::Store;

let graph = Graph::load_str(
    "@prefix ex: <http://example.org/> . ex:alice ex:knows ex:bob . ex:bob ex:age 42 .",
    "turtle",
)?;
let store = Store::borrowed(&graph);
let alice = NamedNode::new("http://example.org/alice")?;
let knows = NamedNode::new("http://example.org/knows")?;
let age = NamedNode::new("http://example.org/age")?;

let bob = store.node(alice).out(&knows).next().expect("friend");
assert_eq!(bob.out(&age).next().expect("age").as_i64()?, 42);
# Ok::<(), Box<dyn std::error::Error>>(())
```

`.out()` and `.r#in()` return `NodeSet`, an `ExactSizeIterator<Item = Node>`;
the raw identifier is Rust's required spelling for a method named `in`.
Call `.values()` on a traversal to yield owned `oxrdf::Term`s. An absent focus
or predicate is valid and yields an empty iterator. `Node::dataset()` exposes a
borrowed dataset wrapper; `.graph()` is the raw `sparq_core::Graph` escape hatch.

Choose ownership deliberately:

- `Store::borrowed(&graph)` is read-only and tied to the graph's lifetime.
- `Store::owned(graph)` and `Store::new()` own the graph and allow
  `insert`/`remove`. Nodes borrow the store, so stop using them before a write
  and reacquire them afterwards.
- Traversal addresses the default graph in M1. Reach named graphs through the
  raw graph until a scoped-dataset surface lands.

Typed accessors are strict:

- `as_str()` accepts `xsd:string` and `rdf:langString`.
- `as_i64()` accepts the XML Schema integer family, enforces every derived
  datatype's exact bounds (`byte` through `unsignedLong`), then checks that the
  value is representable as `i64`.
- `as_bool()` accepts only `xsd:boolean`, including `true/false/1/0`.
- `as_typed_literal()` returns lexical form, datatype, and language.

All return `Result<_, AccessError>`; do not silently coerce a mismatched RDF
datatype.

Three explicitly experimental features implement proposals that remain unlanded
in rdfjs/wrapper:

```toml
sparq-wrapper = { version = "0.1", features = [
  "proposed-distinct",
  "proposed-cardinality",
  "proposed-focus-kinds",
] }
```

`proposed-distinct` adds `Dataset::subjects_of` / `objects_of` and yields each
term once ([issue #25](https://github.com/rdfjs/wrapper/issues/25),
[draft PR #88](https://github.com/rdfjs/wrapper/pull/88)).
`proposed-cardinality` adds `Node::required_out` / `optional_out` and typed
`CardinalityError` data ([draft PR #89](https://github.com/rdfjs/wrapper/pull/89)).
`proposed-focus-kinds` adds typed focus kinds based on rdfjs/wrapper draft PRs
#83-#87: `SubjectNode`, `PredicateNode`, and `ObjectNode` carry their RDF
positional legality in the type; a sealed `IntoSubject` trait ensures that a
`Literal` cannot be passed to `BoundFactory::subject` at compile time.

```rust
use sparq_wrapper::proposed::typed_focus::BoundFactory;

// factory borrows the store once; nodes share the borrow with zero cloning.
let factory = BoundFactory::from_store(&store);
let alice = factory.subject(NamedNode::new("http://example.org/alice")?);
let bob   = factory.subject(NamedNode::new("http://example.org/bob")?);
let knows = NamedNode::new("http://example.org/knows")?;
for friend in alice.out(&knows) { println!("{}", friend.focus()); }
```

SHACL-to-Rust struct generation is not part of M1. Reuse `sparq-shacl`'s
`ShapesModel` for that work; do not invent a second SHACL parser.
