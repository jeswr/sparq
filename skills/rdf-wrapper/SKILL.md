---
name: rdf-wrapper
description: "Traverse sparq RDF graphs as native Rust objects with the opt-in sparq-wrapper crate: bind a focus Term to an owned or borrowed Store, follow outgoing/incoming NamedNode predicates with iterators, unwrap values, convert typed literals to str/i64/bool, mutate owned stores, and optionally use the unlanded distinct-result, typed-cardinality, and typed-focus proposals. Use when Rust code should work with focus objects instead of raw triples or dictionary IDs; SHACL-to-Rust code generation is a later surface."
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
  "proposed-typed-focus",
] }
```

`proposed-distinct` adds `Dataset::subjects_of` / `objects_of` and yields each
term once ([issue #25](https://github.com/rdfjs/wrapper/issues/25),
[draft PR #88](https://github.com/rdfjs/wrapper/pull/88)).
`proposed-cardinality` adds `Node::required_out` / `optional_out` and typed
`CardinalityError` data ([draft PR #89](https://github.com/rdfjs/wrapper/pull/89)).

`proposed-typed-focus` adds the `sparq_wrapper::proposed::typed_focus` module.
Its `NodeFactory` binds one borrowed graph, store, or dataset view and can wrap
many terms without cloning the graph. Kind-specific constructors return a
`TypedNode` whose available traversals reflect the term's legal positions;
`NodeFactory::term` instead returns `AnyNode`, whose enum variant preserves the
concrete focus kind at run time. <!-- [GPT-5.6] sq-1rg2q.2 -->

```rust
use oxrdf::{Literal, NamedNode, Term};
use sparq_core::Graph;
use sparq_wrapper::proposed::typed_focus::{AnyNode, NodeFactory};

let mut graph = Graph::new();
let alice = NamedNode::new("http://example.org/alice")?;
let name = NamedNode::new("http://example.org/name")?;
graph.insert_triple(
    alice.clone(),
    name.clone(),
    Literal::new_simple_literal("Alice"),
)?;

let factory = NodeFactory::new(&graph);
let subject = factory.iri(alice);
assert_eq!(subject.out(&name).len(), 1);

match factory.term(Term::Literal(Literal::new_simple_literal("Alice"))) {
    AnyNode::Literal(value) => assert_eq!(value.r#in(&name).len(), 1),
    _ => unreachable!("the factory preserves the concrete term kind"),
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

Every typed focus supports incoming traversal because every RDF term may be an
object. Outgoing traversal is available only for `SubjectFocus` kinds, so code
such as `factory.literal(value).out(&predicate)` fails to compile. The
predicate-wide `subjects()` / `objects()` helpers are available only on the IRI
focus returned by `NodeFactory::iri`. Match an `AnyNode` variant to recover
those kind-specific methods, or call `into_node()` to erase the focus kind and
return to the untyped wrapper.

SHACL-to-Rust struct generation is not part of M1. Reuse `sparq-shacl`'s
`ShapesModel` for that work; do not invent a second SHACL parser.
