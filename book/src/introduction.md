<!-- [OPUS-4.8] sq-h0tr — scaffold page. Hand-written prose distilled from
README.md + AGENTS.md (no lorem, no hard-coded performance numbers). The
include-wiring bead (sq-im8u) will replace this prose with a single-source
{{#include}} from the canonical file once anchors are added there. -->

# Introduction

**sparq** is a lightning-fast [RDF](https://www.w3.org/TR/rdf12-concepts/) triplestore and
[SPARQL 1.1](https://www.w3.org/TR/sparql11-query/) / [1.2](https://www.w3.org/TR/sparql12-query/)
engine, written in Rust — usable as a library, a CLI, an HTTP server, and from Python and
JavaScript/WASM.

It is a from-scratch engine: dictionary-encoded terms, six sorted permutation indexes, parallel
and streaming execution, RDFS / OWL-RL / N3 inference, an out-of-core (mmap) mode with a
compressed on-disk format, a WebAssembly build, and a W3C-conformant HTTP server.

> **Status: experimental research engine.** The API is unstable and pre-1.0. Conformance against
> the W3C SPARQL, SHACL, and inference suites is tracked by CI ratchets that only ever go up.
> SERVICE federation is not yet fully implemented (see
> [`research/roadmap.md`](https://github.com/jeswr/sparq/blob/main/research/roadmap.md)).

## How it is published

The engine core is always built; every other capability is an opt-in crate that the core does not
depend on, so it stays lean (enforced in CI). The same query surface is mirrored across:

- **Rust crates** (crates.io): `sparq-core`, `sparq-engine`, `sparq-cli`, `sparq-server`, plus the
  opt-in capability crates.
- **npm**: `@jeswr/sparq` — an RDF/JS-typed API over the wasm build, with zero runtime deps.
- **PyPI**: `sparq` — pyo3 / maturin bindings (`import sparq`).

## Where to go next

- [Install & build from source](./getting-started/install.md) — get a working build.
- [Capabilities at a glance](./getting-started/capabilities.md) — what each opt-in surface does.

Per-surface how-to guides live in the
[usage skills](https://github.com/jeswr/sparq/blob/main/skills/SKILL.md) router, and the full crate
map is in [`AGENTS.md`](https://github.com/jeswr/sparq/blob/main/AGENTS.md). Live per-commit
performance metrics are on the
[benchmarks dashboard](https://jeswr.github.io/sparq/dev/bench) — numbers are deliberately **not**
baked into these docs, because they drift.
