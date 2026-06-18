# Feature research: Olaf Hartig's body of work as a sparq seeding point  `[OPUS-4.8]`

> Deep-research record under epic **sq-3183** (federation + broad-SPARQL research themes).
> Purpose: survey the corpus of **Olaf Hartig** (Linköping University; co-chair of the W3C
> RDF-star / RDF 1.2 work; "father" of link-traversal querying and Linked Data Fragments)
> and map each research strand to a concrete sparq candidate feature with a FIT
> classification, impact (1–5), and effort (S/M/L). The orchestrator synthesises all theme
> docs; this one guarantees first-class coverage of Hartig's cross-cutting contributions.

Hartig's corpus is unusually well-aligned with sparq's two open research fronts:
**federation** (SERVICE, source selection, heterogeneous interfaces) and
**broad-SPARQL** (RDF-star/RDF 1.2 completeness, OPTIONAL/well-designed semantics,
provenance, property paths on the Web). He is a primary author on the standards
(RDF-star CG report, RDF 1.2, SPARQL 1.2) and on the foundational theory
(link traversal, Linked Data Fragments, FedQPL, cost models).

FIT vocabulary (per epic): `clear-fit:<component>` | `new-component-but-fits` |
`ambiguous-ask-user`.

---

## 0. What sparq has today (grounding, vs Hartig's themes)

Inventory from the codebase (paths are repo-relative):

| Hartig theme | sparq status today |
|---|---|
| **SPARQL SERVICE / federation** | **Present.** `crates/sparq-engine/src/service.rs` + `eval_service` in `crates/sparq-engine/src/exec.rs`. Blocking `ureq` HTTP SPARQL client (`HttpTransport`, service.rs:443) behind a `Transport` trait; SPARQL-Results-JSON parser (`parse_srj`, service.rs:81) incl. RDF 1.2 `triple` bindings + base-direction. `SERVICE SILENT` supported. Strong default-deny SSRF egress filter. **Gaps:** the inner pattern is forwarded **verbatim** as `SELECT * WHERE {…}`, materialised, then joined locally — **no bindings pushdown, no source selection, no cost model, no `SERVICE ?var`** (explicitly rejected, exec.rs:1959). A local `bind_join` (exec.rs:4464) exists but is an intra-engine index-nested-loop join, *not* federation-level bound joins. |
| **RDF-star / SPARQL-star / RDF 1.2 triple terms** | **Present, RDF 1.2 flavour.** Triple terms are first-class RDF terms via `oxrdf::Term::Triple` (`rdf-12` feature). Custom N-Triples/N-Quads parser handles `<<( s p o )>>` triple terms (object position only — `crates/sparq-core/src/nt.rs:314`, subject-position rejected nt.rs:534); Turtle/TriG via `oxttl`. All SPARQL-star functions present (`TRIPLE`, `isTRIPLE`, `SUBJECT`, `PREDICATE`, `OBJECT` — exec.rs:7066–7099) plus `rdf:reifies` reification (exec.rs:8921). **Gaps:** subject-position triple terms in the custom NT parser; sharded parallel-dict merge falls back when triple terms present (`crates/sparq-core/src/lib.rs:3645`, the `has_triple_terms` serial-path guard); no annotation-syntax sugar (`{\| … \|}`). |
| **Triple Pattern Fragments / LDF / brTPF** | **Absent.** Zero matches for `TPF`/`brTPF`/`hydra`/`linked data fragment` in `crates/*/src`. |
| **Link traversal / follow-your-nose** | **Absent.** `traversal`/`reachability` only refer to in-engine property-path closure (`eval_path`, exec.rs:2568); `dereference` only in `FROM`/`LOAD`/`owl:imports`. No web link-traversal query mode. |
| **Provenance (annotated RDF / why-where)** | **Absent as a data model.** Only: LLM-transcript "provenance" in `sparq-nlq`, reasoner first-derivation provenance (`crates/sparq-reason/src/incremental_explain.rs`), HDT VoID header metadata. Named graphs (`GRAPH`) exist but are not wired as a provenance model. |
| **SPARQL optimiser / semantics** | **Present.** Greedy Operator Ordering (`goo_seed`/`goo_pick`, exec.rs:3476/3605) on cardinality + ndv estimates; characteristic-set table (`cs.rs`) for star patterns; WCOJ Leapfrog Triejoin for cyclic BGPs vs binary sort-merge for acyclic; filter pushdown; bag (multiset) semantics throughout. **Gap:** no explicit **well-designed-pattern / OPT+** OPTIONAL rewriting. |

So Hartig's work touches **two present-but-shallow** areas (SERVICE federation, RDF-star)
and **three entirely missing** areas (TPF/brTPF, link traversal, provenance) plus an
**optimiser refinement** (well-designed OPTIONAL).

---

## 1. Linked Data Fragments (LDF) + Triple Pattern Fragments (TPF)

**Summary.** TPF is a deliberately *low-cost* server interface: the server answers only
single-triple-pattern requests (paged), pushing join/query effort to the client. This
shifts the bandwidth/server-load trade-off — cheap, cacheable, highly available servers at
the cost of more client round-trips. The LDF framework is the conceptual lattice that
positions interfaces (data dump ↔ TPF ↔ full SPARQL endpoint) by what each can evaluate.

**Key papers/specs.**
- Verborgh, Vander Sande, **Hartig** et al., *Triple Pattern Fragments: a Low-cost
  Knowledge Graph Interface for the Web*, J. Web Semantics 2016 —
  <http://olafhartig.de/files/VerborghEtAl_JWS2016.pdf>
- **Hartig**, Letter, Pérez, *A Formal Framework for Comparing Linked Data Fragments*,
  ISWC 2017 (Best Paper) — <http://olafhartig.de/files/HartigEtAl_ISWC2017_Preprint.pdf>
- <https://linkeddatafragments.org/in-depth/>

**sparq candidate features.**
- **(1a) Expose a TPF server endpoint** over a sparq Graph (paged single-pattern responses
  + Hydra controls + metadata count). sparq's permutation indexes make single-pattern paging
  trivially cheap; this turns any sparq instance into a low-cost, cacheable, federatable
  data source. → **FIT: `new-component-but-fits`** (a thin `sparq-ldf`/`sparq-server` route
  over existing indexes). **Impact 4. Effort M.**
- **(1b) TPF client** — evaluate SPARQL by issuing paged single-pattern requests to a remote
  TPF server and joining locally (an alternative `SERVICE`/source kind alongside the HTTP
  SPARQL client). → **FIT: `clear-fit:sparq-engine/service`** (new `Transport` impl + a
  client join loop). **Impact 3. Effort M.**

---

## 2. brTPF (bindings-restricted Triple Pattern Fragments)

**Summary.** brTPF extends TPF so the client may attach a set of *intermediate bindings* to
a triple-pattern request; the server returns only triples that join with at least one
binding. This is the bound-join (a.k.a. bind-join / VALUES-pushdown) idea applied to the
LDF interface: far fewer requests and far less wasted bandwidth than vanilla TPF, without a
full SPARQL endpoint's cost.

**Key paper.** **Hartig**, Buil-Aranda, *Bindings-Restricted Triple Pattern Fragments*,
ODBASE 2016 — <https://arxiv.org/abs/1608.08148>

**sparq candidate feature.**
- **(2) Bindings pushdown for federation** — attach a batch of already-computed bindings
  (as `VALUES`) to remote `SERVICE` sub-queries (and to the TPF client of §1b as a brTPF
  request), instead of forwarding the inner pattern verbatim and joining everything locally.
  This directly fixes the documented "correct, if not maximally-selective" gap in
  service.rs:9–14. The intra-engine `bind_join` already proves sparq knows the bound-join
  shape; this lifts it across the network boundary. → **FIT: `clear-fit:sparq-engine/service`.**
  **Impact 5. Effort M.** *(Highest-leverage federation win Hartig's work recommends.)*

---

## 3. Link-traversal-based query execution (LTBQE) / Querying the Web of Linked Data

**Summary.** Hartig's PhD: execute SPARQL by *following URIs at query time* — dereference
IRIs encountered during evaluation, discover new sources on the fly, and intertwine link
traversal with result construction (iterator model). Because the source set is not fixed in
advance, completeness is defined relative to a **reachability semantics** (e.g.
c_Match / c_All / c_None reachability criteria) rather than the whole Web.

**Key works.**
- **Hartig**, Bizer, Freytag, *Executing SPARQL Queries over the Web of Linked Data*,
  ISWC 2009 (SWSA 10-Year Award) — <https://squin.sourceforge.net/> (SQUIN system)
- **Hartig**, *SPARQL for a Web of Linked Data: Semantics and Computability*, ESWC 2012
- **Hartig**, *Querying a Web of Linked Data: Foundations and Query Execution*,
  IOS Press 2013 (SWSA Distinguished Dissertation) —
  <https://swsa.semanticweb.org/sites/default/files/201507/DissertationOlafHartig_0.pdf>
- **Hartig**, Pérez, *LDQL: A Query Language for the Web of Linked Data*, JWS 2016
  (explicit reachability/navigation specification)

**sparq candidate feature.**
- **(3) Link-traversal query mode** — a federation-client mode that, given seed IRIs and a
  reachability criterion, dereferences IRIs at query time (parse the returned RDF with
  sparq's existing fast parsers), accumulates into a transient Graph, and evaluates the
  query against the growing dataset. Bounded by the reachability semantics + sparq's budget
  framework. Reuses the SSRF egress filter from `service.rs` for safety. → **FIT:
  `new-component-but-fits`** (a `sparq-ltbqe` client orchestrating dereference → parse →
  incremental eval; engine + parsers already exist). **Impact 3. Effort L.**
  *(Powerful, distinctive, but the heaviest lift and least aligned with sparq's
  embedded/single-store core — flag for user prioritisation vs §1/§2.)*

---

## 4. RDF-star / SPARQL-star → RDF 1.2 triple terms + SPARQL 1.2

**Summary.** Hartig co-originated RDF*/SPARQL* (statement-level metadata by embedding a
triple as the subject/object of another triple) and co-chaired its standardisation into
**RDF 1.2 triple terms** + **SPARQL 1.2**. This is the standards-track successor to RDF
reification, with the annotation syntax `{| … |}` and `rdf:reifies`.

**Key works.**
- **Hartig**, Thompson, *Foundations of an Alternative Approach to Reification in RDF*,
  arXiv 2014 — <https://arxiv.org/abs/1406.3399>
- **Hartig**, *Foundations of RDF\* and SPARQL\**, AMW 2017 —
  <https://ceur-ws.org/Vol-1912/paper12.pdf>
- W3C *RDF 1.2 Concepts* (CR, Apr 2026) and *SPARQL 1.2 Query Language* (WD) — Hartig editor.

**sparq status & candidate features.** sparq is **already strong here** (first-class triple
terms, all SPARQL-star functions, `rdf:reifies`). The candidate features are **completeness
gaps**, not greenfield:
- **(4a) Subject-position triple terms in the custom N-Triples/N-Quads parser**
  (currently object-only, nt.rs:534) → **FIT: `clear-fit:sparq-core/nt`. Impact 2. Effort S.**
- **(4b) RDF 1.2 reifying/annotation syntax sugar `<< … >> {| … |}`** in Turtle/TriG and
  the matching SPARQL annotation syntax, tracking the moving SPARQL 1.2 WD → **FIT:
  `clear-fit:sparq-engine` (+ parser). Impact 3. Effort M.**
- **(4c) Triple-term-aware sharded dictionary merge** (remove the `has_triple_terms`
  fallback at `crates/sparq-core/src/lib.rs:3645` so RDF-star bulk loads keep full parallelism) → **FIT:
  `clear-fit:sparq-core`. Impact 2. Effort M.**
- **(4d) Conformance-track SPARQL 1.2 as the spec finalises** (Hartig is editor; sparq
  should follow the rec) → **FIT: `clear-fit:sparq-engine`. Impact 3. Effort M (ongoing).**

---

## 5. Foundations of SPARQL — semantics, OPTIONAL / well-designed patterns, optimisation

**Summary.** Hartig contributed to the semantics line (multiset/bag semantics, OPTIONAL
behaviour, property paths on the Web) and proposed **OPT+**, a *monotonic* alternative to
OPTIONAL that avoids the non-monotonic anomalies of `LeftJoin` and connects to the
well-designed-pattern theory (Pérez–Arenas–Gutiérrez) that makes OPTIONAL nesting tractable
and safely reorderable.

**Key works.**
- **Hartig**, Cheng, *OPT+: A Monotonic Alternative to OPTIONAL in SPARQL*, JWE 2019
- **Hartig**, Pirró, *SPARQL with Property Paths on the Web*, SWJ 2017
- (foundational context) Pérez, Arenas, Gutiérrez, *Semantics and Complexity of SPARQL*,
  well-designed patterns.

**sparq status & candidate feature.** sparq already has bag semantics and a real optimiser
(GOO + characteristic sets + WCOJ), but **no explicit well-designed-OPTIONAL handling** — a
grep for any well-designed-pattern / OPTIONAL-reordering stage in `crates/sparq-engine/src/`
finds none (the existing `LeftJoin` handling is plain evaluation + a count-pushdown branch in
`try_count`, not a well-designed rewrite).
- **(5) Well-designed-pattern OPTIONAL optimisation** — detect well-designed
  `OPTIONAL` nesting and reorder/merge `LeftJoin`s safely (the move that makes OPTIONAL-heavy
  queries tractable), feeding the existing GOO planner. → **FIT:
  `clear-fit:sparq-engine` (planner). Impact 3. Effort M.**

---

## 6. Provenance for SPARQL / annotated RDF

**Summary.** Hartig's early line treats provenance as a first-class, queryable citizen:
a provenance model distinguishing data-creation vs data-access provenance, publishing/
consuming provenance metadata on the Web, and **tSPARQL** (trust-weighted SPARQL). The
RDF-star work later gives the *mechanism* (statement-level annotation) for carrying
provenance/trust/temporal annotations inline.

**Key works.**
- **Hartig**, *Provenance Information in the Web of Data*, LDOW 2009 —
  <https://ceur-ws.org/Vol-538/ldow2009_paper18.pdf>
- **Hartig**, *Querying Trust in RDF Data with tSPARQL*, ESWC 2009 (Best Paper) —
  <https://link.springer.com/content/pdf/10.1007/978-3-642-02121-3_5.pdf>
- **Hartig**, Zhao, *Publishing and Consuming Provenance Metadata on the Web of Linked
  Data*, IPAW 2010
- *PROV-AQ: Provenance Access and Query*, W3C WG Note 2013 (Hartig co-author).

**sparq candidate feature.**
- **(6) Provenance-carrying query answers** — track which source graph / named graph / triple
  term each binding derived from, and optionally surface it on results (a where-provenance
  annotation). This composes naturally with (i) federation §1–3 (per-source attribution),
  (ii) RDF-star §4 (annotations as the carrier), and (iii) sparq's **GenAI grounding** story
  (citable provenance for LLM-grounded answers — a strong differentiator). A lightweight
  first cut: propagate the contributing `GRAPH` IRI(s) through the binding. → **FIT:
  `ambiguous-ask-user`** — the *scope* (named-graph attribution vs full why/where-provenance
  semiring annotation à la annotated RDF) is a product decision. **Impact 4. Effort M–L.**

---

## 7. Recent work — heterogeneous federation, cost models, RDF datatypes

**Summary.** Hartig's current flagship is **HeFQUIN**, a federation engine for
*heterogeneous* federations (SPARQL endpoints + TPF/brTPF servers + JSON Web APIs), built on
**FedQPL** (a formal logical-query-plan language for source selection + plan representation)
and a **cost model** for heterogeneous federations. Also: vocabulary-mapping-aware plans,
the **FedShop** federation-scalability benchmark, and RDF 1.2 literal datatypes for lists/maps.

**Key works.**
- **Hartig** et al., *HeFQUIN* engine — <https://github.com/LiUSemWeb/HeFQUIN>;
  Cheng & Hartig, *FedQPL*, iiWAS 2020 — <https://arxiv.org/abs/2010.01190>
- Cheng & Hartig, *A Cost Model to Optimize Queries over Heterogeneous Federations of RDF
  Data Sources*, DMKG 2023
- Cheng, Ferrada, Hartig, *Considering Vocabulary Mappings in Query Plans …*, CoopIS 2023
  (Best Paper)
- Saleem, **Hartig** et al., *CostFed*, SEMANTiCS 2018 (Best Paper)
- Dang, …, **Hartig**, *FedShop*, ISWC 2023 (federation-scalability benchmark)

**sparq candidate features.**
- **(7a) Federation source-selection + cost model** — pick relevant sources per
  triple-pattern (FedQPL-style) and cost alternative plans; the natural home for the bindings
  pushdown of §2 and the heterogeneous TPF/brTPF clients of §1. → **FIT:
  `clear-fit:sparq-engine/service` (new planner stage). Impact 4. Effort L.**
- **(7b) Heterogeneous federation interfaces** (treat SPARQL endpoint, TPF, brTPF, and
  potentially JSON APIs as interchangeable `Transport`s with capability metadata) — sparq's
  `Transport` trait is already the right seam. → **FIT: `clear-fit:sparq-engine/service`.
  Impact 3. Effort M.**
- **(7c) FedShop-style federation benchmark** in sparq's bench harness, to *measure* §2/§7a
  before claiming wins (empirical-honesty mandate). → **FIT: `clear-fit:bench`. Impact 3.
  Effort M.**

---

## 8. Where Hartig's work most strongly recommends a sparq feature (prioritisation)

Ranked by (impact × alignment with sparq's existing seams ÷ effort), and by how directly the
literature ties the feature to a measurable win:

1. **Bindings pushdown for federation `SERVICE` (§2, brTPF idea).** *Impact 5, Effort M,
   clear-fit.* Directly fixes the documented verbatim-forward gap in `service.rs`; reuses the
   existing `bind_join` shape across the network; the single highest-leverage federation
   improvement, with a strong literature basis (brTPF measured large request/bandwidth cuts).
2. **Expose a TPF server + TPF/brTPF client (§1a/§1b).** *Impact 4/3, Effort M,
   new-component-but-fits / clear-fit.* sparq's permutation indexes make a low-cost,
   cacheable, federatable TPF source nearly free; the client makes sparq a first-class LDF
   consumer. Together with §2 these turn sparq into a heterogeneous federation peer.
3. **RDF-star / RDF 1.2 + SPARQL 1.2 completeness (§4a–4d).** *Impact 2–3, Effort S–M,
   clear-fit.* sparq is already strong; these are cheap, standards-tracking completeness wins
   on a spec Hartig edits — subject-position triple terms, annotation syntax, triple-term
   sharded merge, SPARQL 1.2 conformance.
4. **Federation source-selection + cost model + heterogeneous interfaces (§7a/§7b).**
   *Impact 4/3, Effort L/M.* The HeFQUIN/FedQPL/CostFed line; the planner home for §1–2, but a
   larger build — sequence it after §2 lands and is measured (§7c FedShop bench).
5. **Provenance-carrying answers (§6).** *Impact 4, Effort M–L, ambiguous-ask-user.* High
   value via the GenAI-grounding differentiator, but scope (named-graph attribution vs full
   annotated-RDF semiring) needs a user decision — flag to orchestrator/user.
6. **Well-designed-OPTIONAL optimisation (§5).** *Impact 3, Effort M, clear-fit.* A focused
   planner refinement (OPT+/well-designed-pattern theory) for OPTIONAL-heavy workloads.
7. **Link-traversal query mode (§3).** *Impact 3, Effort L, new-component-but-fits.* Hartig's
   signature contribution and distinctive, but the heaviest lift and least aligned with
   sparq's embedded single-store core — lowest priority of the set; revisit if a Web-of-Data
   client becomes a product goal.

**Net:** Hartig's corpus most strongly pushes sparq toward becoming a **proper federation
peer** — both *producer* (TPF source, §1a) and *smart consumer* (brTPF bindings pushdown
§2, source selection + cost model §7) — while offering cheap, standards-aligned completeness
wins on the **RDF-star/RDF 1.2/SPARQL 1.2** front he co-authors, and a **provenance** story
that dovetails with sparq's GenAI grounding differentiator.

---

### Sources

- Triple Pattern Fragments (JWS 2016): <http://olafhartig.de/files/VerborghEtAl_JWS2016.pdf>
- A Formal Framework for Comparing LDFs (ISWC 2017): <http://olafhartig.de/files/HartigEtAl_ISWC2017_Preprint.pdf>
- brTPF (ODBASE 2016 / arXiv 1608.08148): <https://arxiv.org/abs/1608.08148>
- Executing SPARQL over the Web of Linked Data / SQUIN: <https://squin.sourceforge.net/>
- Dissertation, *Querying a Web of Linked Data* (2013/2014): <https://swsa.semanticweb.org/sites/default/files/201507/DissertationOlafHartig_0.pdf>
- Foundations of an Alternative Approach to Reification (arXiv 1406.3399): <https://arxiv.org/abs/1406.3399>
- Foundations of RDF\* and SPARQL\* (AMW 2017): <https://ceur-ws.org/Vol-1912/paper12.pdf>
- Provenance Information in the Web of Data (LDOW 2009): <https://ceur-ws.org/Vol-538/ldow2009_paper18.pdf>
- Querying Trust in RDF Data with tSPARQL (ESWC 2009): <https://link.springer.com/content/pdf/10.1007/978-3-642-02121-3_5.pdf>
- FedQPL (arXiv 2010.01190): <https://arxiv.org/abs/2010.01190>
- HeFQUIN engine: <https://github.com/LiUSemWeb/HeFQUIN>
- Olaf Hartig publications index: <https://olafhartig.de/publications.html>
- dblp: <https://dblp.org/pid/29/3132.html>
