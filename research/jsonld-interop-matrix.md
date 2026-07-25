<!-- [GPT-5.6] sq-ictdf — evidence-indexed JSON-LD surface inventory. -->
# JSON-LD interoperability matrix

This is the durable, surface-by-surface inventory for the JSON-LD 1.1 work in
epic `sq-oy1f`. It records shipped behaviour separately from planned work. A
ratchet count below is a measured minimum at the pinned test-suite revision,
not a claim that a surface or the engine is conformant.

## Status key

- **IV** — implemented and verified by the cited landed PR and bead.
- **B** — beaded or in flight; the cited open bead owns the missing surface.
- **FG** — implemented and verified, but only when the cited opt-in Cargo
  feature is enabled.
- **NG** — not a goal of that surface. This does not imply that the underlying
  Rust algorithm is absent.

`expanded-out`, `flattened-out`, and `compacted-out` mean RDF-to-JSON-LD
serialization in that document form. `framed-out` means caller-frame output.
The `remote-context` and `html-script` columns mean dereferencing a remote
context and extracting JSON-LD from HTML respectively, not merely accepting
an already-inline context or a JSON document.

## Surface matrix

| Surface | ingest (toRdf) | expanded-out | flattened-out | compacted-out | framed-out | remote-context | html-script |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Native CLI | **IV** `sq-oy1f.4`, [#1007](https://github.com/sparq-org/sparq/pull/1007) | **IV** [#240](https://github.com/sparq-org/sparq/pull/240), `sq-ixc3.3` [#875](https://github.com/sparq-org/sparq/pull/875) | **IV** [#240](https://github.com/sparq-org/sparq/pull/240), `sq-ixc3.3` [#875](https://github.com/sparq-org/sparq/pull/875) | **IV** `sq-oy1f.5`, [#957](https://github.com/sparq-org/sparq/pull/957) | **B** `sq-oy1f.42` | **B** `sq-oy1f.42` (explicit opt-in policy) | **B** `sq-oy1f.33` |
| Server | **IV** `sq-oy1f.1`, [#977](https://github.com/sparq-org/sparq/pull/977) | **B** `sq-oy1f.34` (profile negotiation) | **IV** `sq-oy1f.1`, [#977](https://github.com/sparq-org/sparq/pull/977) (no-profile response) | **B** `sq-oy1f.34` (profile plus context) | **B** `sq-oy1f.34` (profile plus frame) | **B** `sq-oy1f.32` and `sq-oy1f.34` (deny-by-default loader policy) | **B** `sq-oy1f.33` |
| Python | **IV** `sq-oy1f.20`, [#1021](https://github.com/sparq-org/sparq/pull/1021) | **B** `sq-oy1f.43` | **B** `sq-oy1f.43` | **B** `sq-oy1f.43` | **B** `sq-oy1f.43` | **B** `sq-oy1f.32` and `sq-oy1f.43` | **B** `sq-oy1f.33` |
| Wasm / npm | **FG** `jsonld`, `sq-dvyi` [#519](https://github.com/sparq-org/sparq/pull/519) | **FG** `serialize-rdf`, `sq-ixc3.5` [#900](https://github.com/sparq-org/sparq/pull/900) | **FG** `serialize-rdf`, `sq-ixc3.5` [#900](https://github.com/sparq-org/sparq/pull/900) | **FG** `serialize-rdf`, `sq-oy1f.5` [#957](https://github.com/sparq-org/sparq/pull/957) | **B** `sq-oy1f.44` | **NG** (no ambient fetch; a future loader is caller-supplied) | **B** `sq-oy1f.33` |
| GUI (current site) | **IV** `jsonld` site bundle, `sq-dvyi` [#519](https://github.com/sparq-org/sparq/pull/519) | **NG** (current data-formats demo is ingest-only) | **NG** (current data-formats demo is ingest-only) | **NG** (current data-formats demo is ingest-only) | **NG** (no current framing control) | **NG** (the in-tab engine deliberately performs no ambient fetch) | **NG** (no HTML-document import control) |

The native CLI, server, and Python binary/wheel builds enable their JSON-LD
ingest features by default (`sq-oy1f.4` and `sq-oy1f.20`). The Rust library and
lean Wasm build remain opt-in. For the server, only flattened output is a
verified current wire surface; the other document forms exist below the HTTP
layer but are not server capabilities until profile negotiation bead
`sq-oy1f.34` lands. Likewise, helpers left from the removed `/try` page do not
turn an unexposed operation into a current GUI capability.

## Ratcheted algorithm lanes

These are the six committed, rise-only measured floors used by the JSON-LD
suite harness. Denominators are the pinned lane totals, including honest
failures and skips; a full lane does not elevate the cross-surface matrix into
an unqualified conformance claim.

| Lane | Measured floor | Oracle and ownership |
| --- | ---: | --- |
| toRdf | **413 / 467** | oxjsonld RDF dataset comparison; native strict/differential path remains open as `sq-oy1f.30` |
| fromRdf | **52 / 53** | native document comparison plus lossless RDF round-trip and negative error checks; `sq-oy1f.28`, [#1923](https://github.com/sparq-org/sparq/pull/1923) |
| expand | **276 / 385** | native document-level comparison; `sq-oy1f.25`, [#1380](https://github.com/sparq-org/sparq/pull/1380), with later correctness ratchets |
| flatten | **53 / 58** | native document-level comparison; `sq-oy1f.26`, [#1811](https://github.com/sparq-org/sparq/pull/1811) |
| compact | **228 / 246** | native normative document comparison; `sq-oy1f.27`, [#1934](https://github.com/sparq-org/sparq/pull/1934) |
| frame | **92 / 92** | native normative document comparison, including negative cases; `sq-oy1f.29`, [#1995](https://github.com/sparq-org/sparq/pull/1995) |

The authoritative numbers live in
`crates/sparq-conformance/src/floors/{to_rdf,from_rdf,expand,flatten,compact,frame}.rs`.
They describe algorithm-lane measurements, not HTTP negotiation, CLI option
coverage, Python bindings, Wasm bundle contents, or GUI controls. Remote
documents (`sq-oy1f.32`) and HTML script extraction (`sq-oy1f.33`) remain
separate planned lanes and are not silently counted as implemented here.

## Reading this record

Use the matrix before assigning surface work: an **IV** or **FG** cell must keep
its cited regression tests green; a **B** cell belongs to its cited bead; and an
**NG** cell requires an explicit scope decision before implementation. The
older design and decomposition records remain useful for algorithm detail and
dependency order, but proposed or designed-only text there must not override
the landed-state classification above.
