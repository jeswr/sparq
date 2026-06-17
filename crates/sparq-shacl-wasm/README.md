# sparq-shacl-wasm

**The tier-b "W-shacl" WebAssembly bundle** ([OPUS-4.8] sq-lfmf) for
[`sparq-shacl`](../sparq-shacl/README.md) — SHACL Core + SHACL-SPARQL (`sh:sparql`, W3C
SHACL §5.2) **validation, live in the browser tab**.

This is a SEPARATE, lazy-loaded bundle from the lean [`sparq-wasm`](../sparq-wasm/README.md)
triplestore bundle. The lean bundle deliberately carries no SHACL on the landing page; the
showcase site's `/surface/shacl` page loads this bundle on demand (`next/dynamic`,
client-only). It mirrors the per-bundle-crate pattern of
[`sparq-reason-wasm`](../sparq-reason-wasm/README.md).

> The lean `sparq-wasm` bundle also exposes a minimal `Store.validate(...)` behind its
> non-default `shacl` feature (sq-yqi1, #162) for the PSS ADR-0014 `ShaclValidator` seam.
> This standalone bundle is the **showcase** artifact: a stateless `Validator` with the
> FULL report surface — JSON, W3C-vocabulary report Turtle, plain text, and a
> severity-filtered conformance flag — so the page can render the per-violation report
> (e.g. the `ex:age "thirty"` datatype-violation example from
> [`skills/shacl-validation/SKILL.md`](../../skills/shacl-validation/SKILL.md)) without
> putting SHACL code on the landing page.

## What it exposes

A single `Validator` with stateless one-shot entry points. `data` and `shapes` are RDF
documents in the syntaxes `sparq_core::Graph::load_str` accepts: `"turtle"`, `"ntriples"`,
`"nquads"`, `"trig"` (named graphs are folded into the default graph). Every method errors
(a thrown `Error`) only if a document fails to parse — malformed shapes are skipped by the
engine, never surfaced as an error.

```js
import init, { Validator } from "./sparq_shacl_wasm.js";
await init();

// The W3C validation report as JSON ({ conforms, results: [...] }):
const report = JSON.parse(Validator.validate(dataTurtle, shapesTurtle, "turtle"));
// report.conforms === false
// report.results[0] === { focusNode, path, value, sourceShape,
//                         sourceConstraintComponent, severity, message }

// The same report as report-RDF Turtle (the sh:ValidationReport vocabulary):
const ttl = Validator.validateTurtle(dataTurtle, shapesTurtle, "turtle");

// A human-readable rendering (one line per result):
const text = Validator.validateText(dataTurtle, shapesTurtle, "turtle");

// Just the conformance flag — pass `true` for a violations-only (ignore Warning/Info) gate:
const ok = Validator.conforms(dataTurtle, shapesTurtle, "turtle", /* violationsOnly */ false);
```

- In each JSON result, `path` and `value` are `null` when the result carries none;
  `focusNode`/`value`/`sourceShape` are N-Triples term strings (the lexical form `Term`'s
  `Display` produces), `path` is a SHACL Turtle path expression, and `message` is the plain
  text of the shape's first `sh:message` (or the generated default).
- `conforms` (and `report.conforms`) counts EVERY result regardless of severity — the
  W3C-suite notion of `sh:conforms`. Pass `violationsOnly: true` (or filter `results` by
  `severity`) for a CI-style gate that ignores `sh:Warning` / `sh:Info`.
- Validation correctness is the native `sparq-shacl` engine's; this bundle only loads the
  two graphs and hand-serialises the report (no serde, mirroring the lean bundle's JSON
  path).

### SHACL Advanced Features (`sh:rule`) — opt-in `shacl-af` feature

Built with `--features shacl-af`, the bundle also engages `sparq-shacl`'s SHACL-AF rule
validation (`sh:rule`). It is OFF by default so the standard SHACL bundle carries zero rule
code; CI builds and tests the bundle in BOTH feature states.

## Building the bundle

```sh
# SHACL Core + SHACL-SPARQL:
wasm-pack build crates/sparq-shacl-wasm --target web --release
# Add SHACL Advanced Features (sh:rule):
wasm-pack build crates/sparq-shacl-wasm --target web --release -- --features shacl-af
```

The build is single-threaded (no rayon) and pure-Rust. `sparq-shacl`'s `regex` dependency
(the `sh:pattern` constraint) compiles to `wasm32-unknown-unknown` — the regex automata are
the bundle-size consideration, which is why this bundle is lazy-loaded on its page only and
built `-Oz`.

### Measuring the bundle size

We deliberately do **not** quote a hard-coded byte/MB figure here — bundle size drifts with
the toolchain (`rustc`, `wasm-bindgen`, `wasm-opt`) and dependency versions, so any number
would silently rot. To get a reproducible figure for your toolchain, build and measure the
emitted `.wasm` directly:

```sh
wasm-pack build crates/sparq-shacl-wasm --target web --release   # add `-- --features shacl-af` for the rules variant
f=crates/sparq-shacl-wasm/pkg/sparq_shacl_wasm_bg.wasm
echo "pre-gzip: $(stat -c%s "$f") bytes   gzip -9: $(gzip -9 -c "$f" | wc -c) bytes (the over-the-wire transfer size)"
```

The gzip figure — not the pre-gzip one — is what end users actually download, since the
showcase site serves the `.wasm` gzip-compressed.

## Status / what remains

This crate delivers the wasm portability (the spike confirmed `sparq-shacl` + `sparq-engine`
build for `wasm32-unknown-unknown` with no `std::fs`/threads, carrying the `sh:pattern`
REGEX path), the `Validator` entry points, the `shacl-af` opt-in, and a headless
`wasm-pack test --node` smoke suite. The npm wrapper packaging and the GitHub Pages deploy
wiring for this bundle are tracked separately (the `/surface/shacl` page bead sq-egy6 and
the Pages workflow); see the PR description.

## License

[MIT](../../LICENSE).
