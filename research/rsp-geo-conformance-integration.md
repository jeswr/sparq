# Design — RSP (RDF Stream Processing) + GeoSPARQL conformance integration

<!-- [OPUS-4.8] Design-for-review for epics sq-2n1q3 (RSP) and sq-lk3aw (GeoSPARQL + full-text),
under umbrella sq-6tykl. NO production code in this PR. 🤖 SPARQ agent. -->

> 🤖 **SPARQ agent** — design record for @jeswr's review. DESIGN-FOR-REVIEW only.

**Status:** DESIGN / design-for-review. **Epics:** sq-2n1q3 (RSP), sq-lk3aw (GeoSPARQL +
full-text). Both are independent of the substrate and federation — they parallelise.

**Recommendation in one line:** GeoSPARQL already has a hand-curated OGC ratchet (119, in the
scoreboard); **graduate the geo query-rewrite gap and add the in-scope OGC requirement coverage**,
and for RSP/text **frame the ratchets honestly** — there is **no W3C/CSPARQL RSP conformance suite
and no full-text RDF standard**, so wire RSP as an *expressivity/correctness* ratchet (against the
SRBench oracle) and text as a *BM25 differential-oracle* ratchet, **not** as "conformance to a
standard."

---

## 0. Premise check (honesty first — this is where the brief most needs discipline)

Verified against the code and the standards landscape:

- **GeoSPARQL: real OGC ratchet, already in the scoreboard.**
  `crates/sparq-geo/tests/ogc_compliance_ratchet.rs` pins `OGC_RATCHET_FLOOR = 119` and is
  registered as `Runner::CrateTest { krate: "sparq-geo", target: "ogc_compliance_ratchet" }`
  (`scoreboard.rs:190`). Covers R1–R30 topology (sf/eh/rcc8). Confirmed.
- **GeoSPARQL gaps are documented, not hidden:** the query-rewrite *property* form
  (`geo:sfWithin` as a property path, not a `geof:` FILTER function) is **not implemented**
  (sq-5ts8); the official OGC TEAM Engine / CITE suite is Java/HTTP and **not vendorable** to an
  MIT repo; metric distance for extended geometries uses a **local equirectangular approximation**
  (point-to-point is exact Haversine). All three are real and must stay documented.
- **RSP: there is NO published executable W3C/CSPARQL conformance suite.** Confirmed against the
  landscape — C-SPARQL / CQELS / RSP4J are *runtime* engines with wall-clock windows; SRBench is a
  *correctness benchmark*, not a conformance ETS. sparq-rsp is **clock-free / deterministic**. So
  **"RSP conformance" is not a thing that exists to conform to.** The brief is explicit about this
  and it is the single most important honesty point in this epic.
- **Full-text: no standard at all.** `text:` predicates are a sparq extension
  (`http://sparq.dev/text#`, BM25 + UAX-#29 tokenisation + phrase/proximity). There is no W3C/ISO
  full-text-RDF standard. "Text conformance" would be a category error.
- **RSP and text are ABSENT from the scoreboard** (grep count 0). Confirmed.

**Correction to the brief's framing:** the epic title (sq-lk3aw) says "W3C conformance" for
GeoSPARQL+text. GeoSPARQL conformance is to **OGC** (not W3C), via a *hand-curated* probe (the
official suite is not vendorable). Full-text has **no** conformance target. The honest deliverable
is: graduate/extend the *existing* OGC ratchet for geo, and add *correctness/expressivity*
ratchets — clearly labelled as such — for RSP and text. Do **not** add a scoreboard row whose
`family` claims a standard that does not exist.

---

## 1. GeoSPARQL (sq-lk3aw) — extend the existing OGC ratchet

The infrastructure exists; the work is coverage + one capability gap:

- **Graduate the query-rewrite property form** (sq-5ts8): implement `geo:sfWithin`-style topology
  *property* rewriting (opt-in `geosparql_rewrite`, already partially present) and add it to the
  ratchet. Honest: keep it behind the existing opt-in entry point so default W3C SPARQL behaviour
  is unaffected.
- **Raise the OGC requirement coverage** where the hand-curated probe under-covers (metadata
  properties `geo:dimension`/`isEmpty`/`isSimple` already partly covered via
  `tests/ogc_geosparql_requirements.rs`; extend toward the full R1–R30 + metadata set).
- **Document the distance approximation** prominently in the geometry README + the scoreboard note
  (extended-geometry distance is approximate; point-to-point is exact). A conformance suite that
  exercised continent-spanning distance would surface this — naming it is the honest move.

### Full-text (under sq-lk3aw) — differential oracle, not conformance

Wire `sparq-text`'s existing differential BM25 oracle (`tests/oracle.rs`) as a scoreboard ratchet
labelled **"text-search differential oracle"** (family: sparq extension), NOT "conformance." The
floor is the oracle pass count. This is honest and still valuable — it catches index/scoring
regressions — without claiming a nonexistent standard.

## 2. RSP (sq-2n1q3) — expressivity/correctness ratchet against SRBench

- Wire the existing `bench/rsp` SRBench correctness oracle (~22 deterministic per-window
  assertions across tumbling/sliding/count windows × RSTREAM/ISTREAM/DSTREAM × multi-window joins
  × the 3–4 EvalModes) into the scoreboard as a `Runner::CrateTest` with a pinned
  `RSP_EXPRESSIVITY_FLOOR`.
- **Label it "RSP expressivity / SRBench correctness," with a scoreboard note stating no formal
  W3C/CSPARQL conformance suite exists.** This is the load-bearing honesty requirement: any claim
  of "RSP conformance" is challengeable and must not appear.
- The deterministic per-window row-count assertions are the gate; window *throughput* stays a
  trend-only advisory metric (not gated), and the work-box timing is non-canonical.

## 3. Soundness / scope notes

- All three crates (`sparq-rsp`, `sparq-geo`, `sparq-text`) are **opt-in, wasm-isolated** (tier-b
  bundles `sparq-rsp-wasm`, `sparq-text-wasm`), with **zero core-engine dependency**, so wiring
  their ratchets does **not** touch the lean wasm bundle floor (1686907) or the byte ratchets.
- The scoreboard crate must stay **free of depending on** `sparq-geo`/`sparq-rsp`/`sparq-text`
  (it already deliberately avoids pulling them in); the `Runner::CrateTest` indirection (run the
  crate's own `cargo test` target) is exactly how this is kept acyclic. New rows follow that
  pattern — no new dependency edge into the scoreboard crate.
- **No privacy/ZK/MPC surface** in any of these.

## 4. Phased plan (each phase = a future bead)

Under **sq-lk3aw** (geo + text):

1. **Geo: graduate query-rewrite property form** (sq-5ts8) into the OGC ratchet (opt-in). *Acc:*
   `OGC_RATCHET_FLOOR` rises; default SPARQL behaviour unchanged; scoreboard note updated.
2. **Geo: extend OGC requirement coverage** (metadata/metric where in-scope) + document the
   distance approximation. *Acc:* ratchet rises or holds; README + scoreboard note state the
   approximation honestly.
3. **Text: wire BM25 differential-oracle ratchet** into the scoreboard, labelled as an extension
   oracle (not conformance). *Acc:* `TEXT_ORACLE_FLOOR` const + scoreboard row + guard entry;
   note states no standard exists.

Under **sq-2n1q3** (RSP):

4. **RSP: wire SRBench expressivity/correctness ratchet** into the scoreboard, labelled honestly.
   *Acc:* `RSP_EXPRESSIVITY_FLOOR` const + scoreboard row + guard entry; note states no W3C/CSPARQL
   conformance suite exists; deterministic per-window assertions gate, throughput trend-only.

## 5. Open questions for the maintainer

1. **Labelling:** confirm you are comfortable with RSP/text scoreboard rows labelled
   "expressivity / differential oracle" rather than "conformance" — this is the honest framing and
   I will not ship a row claiming a nonexistent standard without your steer.
2. **Geo distance:** is the equirectangular approximation acceptable to keep (documented), or do
   you want exact geodesic distance for extended geometries as a follow-up bead?
3. **OGC CITE:** the official suite is not MIT-vendorable; confirm the hand-curated probe remains
   the sanctioned conformance evidence for GeoSPARQL.
