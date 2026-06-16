# sparq-reason-wasm

**The tier-b "W-reason" WebAssembly bundle** ([OPUS-4.8] sq-6qw3) for
[`sparq-reason`](../sparq-reason/README.md) — RDFS / OWL 2 RL / Notation3 forward-chaining
**inference, live in the browser tab**.

This is a SEPARATE, lazy-loaded bundle from the lean [`sparq-wasm`](../sparq-wasm/README.md)
triplestore bundle. The lean bundle deliberately carries no reasoning code; the showcase
site's `/surface/inference` page loads this bundle on demand (`next/dynamic`, client-only) so
the landing page stays light. It mirrors the per-bundle-crate pattern of `sparq-wasm`.

## What it exposes

A single `Reasoner` with stateless one-shot entry points:

```js
import init, { Reasoner } from "./sparq_reason_wasm.js";
await init();

// Full RDFS closure (asserted base + every entailed triple) as N-Triples:
const closure = Reasoner.materialize(turtle, "turtle", "rdfs");

// Only the NEWLY-entailed triples (the "what reasoning added" delta):
const added = Reasoner.entailed(turtle, "turtle", "owl-rl");

// Base vs entailed counts, without serialising the closure:
const { baseTriples, closureTriples, entailed } =
  JSON.parse(Reasoner.materializeStats(turtle, "turtle", "rdfs"));

// Notation3 rule reasoning ({ … } => { … }) — the Socrates example:
const derived = Reasoner.reasonN3(n3);
```

- `profile` is `"rdfs"` or `"owl-rl"` (OWL 2 RL includes RDFS).
- Document syntaxes are those `sparq_core::Graph::load_str` accepts: `"turtle"`,
  `"ntriples"`, `"nquads"`, `"trig"` (named graphs are folded into the default graph).
- Output is canonical N-Triples (a valid Turtle subset), serialised through the engine's
  tested CONSTRUCT path — this bundle owns no term-serialisation code of its own.

### `why()` proof trees — opt-in `explain` feature

Built with `--features explain`, the bundle also exposes the derivation **proof tree** for a
single entailed triple:

```js
// Why is `Socrates a Mortal` in the closure?
const proof = JSON.parse(Reasoner.why(
  turtle, "turtle", "rdfs",
  "<http://ex/Socrates>",
  "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>",
  "<http://ex/Mortal>",
));
// proof === { root, nodes: [{ id, conclusion: [s,p,o], rule, premises }, …] }
// or `null` if the triple is not entailed.
```

The JSON is `sparq-reason`'s flat, premises-before-conclusion `ProofTree::to_json` shape:
leaves are `"asserted"` base facts; every internal node names the inference rule that fired
and its premise node indices. `why()` returns **one** derivation (the first in the
deterministic search order), not an enumeration of all of them.

`explain` is OFF by default, so the standard reason bundle carries zero proof-tree code; CI
builds and tests the bundle in BOTH feature states.

## Building the bundle

```sh
# Lean reason bundle (materialize / entailed / stats / reasonN3):
wasm-pack build crates/sparq-reason-wasm --target web --release
# With the why() proof tree:
wasm-pack build crates/sparq-reason-wasm --target web --release -- --features explain
```

The build is single-threaded (no rayon) and pure-Rust. The reasoner's `regex` dependency
(the N3 `string:matches` builtin) compiles to `wasm32-unknown-unknown` — the regex automata
are the bundle-size consideration the design flags, which is why this bundle is lazy-loaded
on its page only and built `-Oz`.

## Status / what remains

This crate delivers the wasm-compatibility changes, the `Reasoner` entry points, the `why()`
proof-tree binding (opt-in), and a headless `wasm-pack test --node` smoke suite. The npm
wrapper packaging and the GitHub Pages deploy wiring for this bundle are tracked separately
(the inference page bead sq-0po6 and the Pages workflow); see the PR description.

## License

[MIT](../../LICENSE).
