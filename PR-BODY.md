> 🤖 Authored by an agent from the **Kern / Kernel-of-Truth** research project (working in the sparq estate). Opening as a **draft — not yet ready for maintainer review.**

**Why.** Kern stores axiom sidecars and world-layer records as RDF 1.2 reified triples (reifiers + triple terms) over a reasoned base graph, so it is load-bearing for us that reification is semantically OPAQUE in sparq's reasoners: a triple term must never be treated as asserted, and reifying a triple must never perturb the closure of the graph it refers to. RDF 1.2 (Concepts "Reifying triples"; Semantics, triple terms) makes this normative — a reified triple mints a TERM, not an assertion — but sparq only pinned the *parser* side (desugaring, structural interning). This PR pins the *reasoner* side as a conformance ratchet, so a future rule that unwraps triple terms or special-cases `rdf:reifies` goes red instead of silently changing entailments. Genuine upstream hardening — not gated on any Kern experiment.

**What.** A conformance lane under `crates/sparq-conformance/tests/`:

- `quoted_triple_opacity.rs` — the runner (UNGATED: plain `sparq_reason::materialize`, default features, no fetched data), per profile (RDFS + OWL 2 RL):
  1. **Quoting never asserts** — a reified triple `<< s p o >>` in all three surface forms (`rdf:reifies` + triple term, reified-triple term as subject with `~ :r`, as object with a fresh reifier) entails neither `s p o` nor any consequence (domain/range typing, sub-property fan-out, subclass hop). The RDF 1.2 annotation form `s p o {| … |}` — which DOES assert — is the in-fixture positive control proving the guarded rules genuinely fire on asserted triples.
  2. **Closure non-interference** — `closure(base ∪ overlay) == closure(base) ∪ overlay`, byte-identical, where the overlay quotes an asserted, a false, an entailed-but-unasserted, and a reversed base triple; `closure(base)` is additionally byte-pinned against committed expected-answer files (`quoted_opacity/expected/*.nt`, sorted N-Triples, hand-verified line-by-line).
  3. **Reifier annotations reason normally** — `rdfs:domain` typing + a subclass hop fire ON the reifier node (`:r5 a :Claim, :Statement`) without leaking the quoted content.
- An **EL arm** behind the existing opt-in `el-suite` feature: a quoted `rdfs:subClassOf` axiom never enters the `sparq-reason-el` TBox (lattice byte-identical with/without the overlay). Excluded from the floor so the measured count is feature-independent.
- `quoted_opacity/` — small explicit Turtle 1.2 fixtures + expected-answer files + README.
- A scoreboard row (`scoreboard::SUITES`) + textual floor-sync guard (`QUOTED_OPACITY_FLOOR = 84`, the measured assertion count), following the UFO-SN3/extension-row pattern — HONESTLY tallied as a sparq-extension ratchet (self-authored fixtures pinning normative RDF 1.2 semantics), never folded into the conformance total.

Test-only; no library/src behavior changes (one registry row + one test-count assertion updated in `scoreboard_floors.rs`); no new deps.

**Run:** `cargo test -p sparq-conformance --test quoted_triple_opacity` (and `--features el-suite` for the EL arm); `cargo test -p sparq-conformance --test scoreboard_floors` (passing locally).

**Honest caveats.** The fixtures are self-authored: the W3C rdf-tests 1.2 entailment corpus does not yet cover reasoner-side opacity, so this is ratcheted as a sparq EXTENSION row, not a W3C pass count. OWL 2 QL is not covered — `sparq-reason-ql` is a query rewriter (no materialized closure to compare); its certain-answer oracles are already ratcheted elsewhere. N3-rule-level quoted-triple matching remains the tracked gap noted in the UFO-SN3 suite (sparq-org/sparq#2012) and is out of scope here.

@jeswr for review when it's ready.
