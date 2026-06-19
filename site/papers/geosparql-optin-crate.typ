// [OPUS-4.8] sq-gum8 — Paper A3: a conformant, opt-in GeoSPARQL layer + the cross-family
// conformance ratchet that backs it.
// Single-source Typst. Numbers come ONLY from #headline(...) / #ev(...) (paper-evidence.json),
// never hard-coded. Compiles to BOTH a PDF (the download) and semantic HTML (the in-site page).
// Framed HONESTLY as a RESOURCES / IN-USE contribution: the evidence is the set of
// DETERMINISTIC, CI-enforced conformance ratchet floors (machine-independent integers), NOT a
// performance result and NOT a research-novelty claim about spatial algorithms.

#import "_lib/bench.typ": headline, ev, provenance, authors, anon

#set document(title: "A Conformant, Opt-In GeoSPARQL Layer")
#set text(size: 11pt)
#set par(justify: true)
#set heading(numbering: "1.")

#align(center)[
  #text(size: 17pt, weight: "bold")[
    A Conformant, Opt-In GeoSPARQL Layer for a Dictionary-Id RDF Engine,
    Backed by a Cross-Family Conformance Ratchet
  ]
]
#authors()

#align(center)[#text(style: "italic", size: 0.9em)[
  A resources / in-use contribution. The evidence is a set of deterministic, CI-enforced
  conformance ratchet floors (machine-independent integers that may only rise); no wall-clock
  performance is claimed, and the spatial algorithms are standard pure-Rust prior art, not a
  novelty.
]]

== Abstract

Adding GeoSPARQL to an RDF engine usually means coupling a spatial stack into the core, which
burdens every build — including a WebAssembly target — whether or not spatial queries are ever
issued. We describe an _opt-in_ GeoSPARQL 1.0/1.1 layer that ships as a separate crate engaged
only by declaring it as a dependency: no core or wasm build links the spatial stack
unconditionally. The layer reuses the host engine's generic SPARQL machinery — its
extension-function registry runs `geof:` functions inside real `FILTER`/`BIND`, and its generic
RDFS/OWL-RL closure satisfies the GeoSPARQL entailment requirements with no spatial-specific
reasoner. We make no performance claim. Instead the contribution is _conformance discipline_:
the spatial layer is one suite in a single cross-family conformance ratchet — a registry of
every suite the project gates, each with a monotone floor that CI enforces and that a guard
test proves stays in lock-step with the runner it mirrors. This is a resources / in-use
contribution; its value is a faithful, decoupled spatial layer with a machine-checkable
conformance floor, not an algorithmic result.

== Contributions

This paper substantiates the following claims; each is refutable and forward-references its
evidence. None is a performance or novelty claim about spatial algorithms.

- *An opt-in spatial layer with no core/wasm coupling* (§3) — GeoSPARQL support is a
  separate crate; nothing in the core engine or the wasm build links the spatial stack unless
  the crate is added as a dependency. Spatial capability is purely additive.
- *The host engine's generic machinery is reused, not duplicated* (§3) — `geof:` functions
  run as host extension functions inside ordinary SPARQL expressions, and the GeoSPARQL
  RDFS/OWL-RL entailment requirements are met by the engine's _generic_ closure over the
  GeoSPARQL ontology axioms, with no geometry-specific reasoner.
- *The spatial layer carries a deterministic, CI-enforced conformance floor* (§4) — OGC
  GeoSPARQL topology compliance is ratcheted at a floor of
  #headline("conformance.geosparql_topology_floor") passing assertions that may only rise.
- *That floor is one row of a single cross-family ratchet* (§5) — the project registers
  #headline("conformance.suite_count") conformance suites across crates in one central
  scoreboard with a combined floor of #headline("conformance.cross_family_total"), and a guard
  test proves each declared floor equals the floor the runner actually enforces.

== The opt-in layer and its reuse of the host engine <layer>

GeoSPARQL is a spatial extension to RDF/SPARQL: WKT and GML geometry literals
(`geo:wktLiteral`, `geo:gmlLiteral`), the `geof:` spatial functions (distance, the
simple-features / Egenhofer / RCC8 topological relations via the DE-9IM matrix, set operations,
constructors), and the class/property entailment requirements. Implementing it faithfully is
table-stakes parity with established engines; it is _not_ a research novelty, and we do not
claim one. The geometry parsing and algorithms wrap the standard pure-Rust spatial stack, and
the spatial index wraps a standard R-tree.

The systems angle is purely _architectural_: the layer is a separate crate, so the spatial
stack is linked only when the crate is declared as a dependency. No other crate — and in
particular not the WebAssembly build — depends on it unconditionally. A deployment that never
issues a spatial query pays nothing for spatial support: no binary-size cost, no extra
dependency surface, no wasm bloat. Spatial capability is _additive_, engaged by opting in.

Decoupling is not the same as isolation: the layer _reuses_ the host engine rather than
re-implementing query infrastructure. The `geof:` functions register with the engine's
extension-function registry, so they evaluate inside real SPARQL `FILTER` and `BIND`
expressions exactly like any built-in. The GeoSPARQL RDFS/OWL-RL entailment requirements (the
`geo:Feature` / `geo:Geometry` / `geo:SpatialObject` class hierarchy and the
`geo:hasGeometry` / `geo:hasDefaultGeometry` property axioms) are satisfied by running the
engine's _generic_ RDFS / OWL-RL closure over the GeoSPARQL ontology axioms — there is no
geometry-specific reasoner. The result is a faithful spatial layer that stays decoupled from
core while inheriting the engine's full query and reasoning machinery.

== Conformance, not performance, is the evidence <conformance>

We make _no_ wall-clock claim. The spatial algorithms are standard, and the canonical
performance runner this project gates timings on is not yet operational; an honest spatial
benchmark would need it. What _is_ canonical today is _correctness against the OGC suite_,
expressed as a ratchet floor: a monotone lower bound on the number of conformance assertions
that pass, which CI enforces and which may only rise.

#figure(
  table(
    columns: 3,
    align: (left, right, left),
    table.header[Suite][Ratchet floor][Basis],
    [OGC GeoSPARQL topology compliance],
    [#headline("conformance.geosparql_topology_floor")],
    [hand-curated sf / eh / rcc8 topology + WKT/GML equivalence],
  ),
  caption: [
    The spatial layer's deterministic conformance floor: passing OGC GeoSPARQL topology
    assertions. A monotone lower bound a CI test enforces — machine-independent (the value of a
    `const` an assertion checks), not a timing. The floor may only rise.
  ],
)

#provenance("conformance.geosparql_topology_floor")

This floor is deterministic and machine-independent: it is the value of a constant the OGC
compliance test asserts, identical on any host, with no dependence on wall-clock timing. That
property is exactly why it can back a headline here while a latency number could not.

== One cross-family ratchet, guarded against drift <ratchet>

The spatial floor does not stand alone. The project maintains a _single central scoreboard_:
a registry of every conformance suite it gates — across crates — each row carrying the spec
family, the runner that executes it, the CI job that enforces it, and its ratchet floor. The
scoreboard registers #headline("conformance.suite_count") suites with a combined floor of
#headline("conformance.cross_family_total") passing assertions:

#figure(
  table(
    columns: 3,
    align: (left, left, right),
    table.header[Suite][Family][Floor],
    [W3C SPARQL (query + update + syntax)],
    [W3C SPARQL],
    [#headline("conformance.sparql_floor")],
    [Inference (rdf-mt / OWL 2 RL / N3 / entailment)],
    [W3C RDF Semantics + OWL + N3],
    [#headline("conformance.inference_floor")],
    [W3C SHACL core],
    [W3C SHACL],
    [#headline("conformance.shacl_core_floor")],
    [W3C SHACL-SPARQL],
    [W3C SHACL],
    [#headline("conformance.shacl_sparql_floor")],
    [OGC GeoSPARQL topology],
    [OGC GeoSPARQL],
    [#headline("conformance.geosparql_topology_floor")],
    table.cell(colspan: 2)[*Total (#headline("conformance.suite_count") suites)*],
    [*#headline("conformance.cross_family_total")*],
  ),
  caption: [
    The cross-family conformance ratchet: every suite the project gates, in one registry, each
    with a monotone floor. The spatial layer (last row) is one suite among five spanning four
    spec families. Each floor is a deterministic integer CI enforces and that may only rise.
  ],
)

The scoreboard is a _reporting_ registry, not a merged runner: each suite's runner stays in the
crate where its dependencies live (so the spatial and SHACL ratchets are not pulled into the
SPARQL/inference harness, and that harness does not depend on the spatial or SHACL crates).
Consolidation happens at the reporting layer. To keep the central declaration honest, a guard
test reads each crate-local runner's floor constant _textually_ and asserts it equals the value
copied into the scoreboard — so the two can never silently diverge: if a runner raises its
floor, the guard fails until the registry is updated to match. The cross-family total is
therefore the sum of independently CI-enforced, drift-guarded floors, not a hand-maintained
number.

== Related work and honest positioning

Faithful GeoSPARQL implementations exist in established stacks (RDF triple stores and spatial
SPARQL engines); we do not claim a spatial-algorithm advance over them, and the geometry
algorithms here are standard pure-Rust libraries used as-is. The only mild systems angle is the
clean opt-in-crate architecture — spatial support that imposes no cost on a core or wasm build
that never uses it — together with the engine-reuse (extension-function registry, generic
entailment closure) that avoids duplicating query infrastructure. The conformance ratchet
methodology is engineering discipline rather than a research result; it is presented here as
the machine-checkable _evidence_ for the in-use claim, and as the mechanism that lets a spatial
suite join a cross-family floor without a bespoke harness.

== Conclusion

A spatial layer need not burden the engine that hosts it: shipped as an opt-in crate, GeoSPARQL
support costs nothing for a build that never issues a spatial query, while still inheriting the
host engine's SPARQL evaluation and generic entailment. The honest evidence is conformance, not
speed — an OGC topology floor of #headline("conformance.geosparql_topology_floor") that is one
row of a single cross-family ratchet totalling #headline("conformance.cross_family_total")
across #headline("conformance.suite_count") suites, every floor deterministic, CI-enforced,
monotone, and guarded against drift. We make no wall-clock claim; a spatial performance
evaluation is deferred to the canonical runner.

#if not anon [
  #line(length: 100%)
  #text(size: 0.8em, fill: gray)[
    sparq project · evidence traces to `crates/sparq-geo/tests`, the cross-family scoreboard in
    `crates/sparq-conformance/src/scoreboard.rs`, and its drift guard
    `tests/scoreboard_floors.rs`. Numbers in this document are injected at build time from the
    paper-bound evidence file; see the provenance stamp on the published page.
  ]
]
