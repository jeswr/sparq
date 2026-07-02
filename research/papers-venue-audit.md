# Papers Venue-Bar Audit

> Papers venue-bar audit (bead sq-gum8.2). Verdicts are non-sycophantic; KILL means kill.

## How the Papers Pipeline Works

Single-source Typst → gated compile → static-injected fragment + PDF.

**REGISTRY**: `site/src/data/papers.ts` is the one source of truth (`PAPERS[]` with
slug/source/title/venue/status/family). It drives the index card grid
(`site/src/app/papers/page.tsx:68`), the per-paper static routes via `generateStaticParams`
(`site/src/app/papers/[slug]/page.tsx:31`), and the build step.

**BUILD**: `site/scripts/build-papers.mjs` runs on prebuild. It reads slug+source out of
`papers.ts` with a regex (`readRegistry`, line 68), runs a data-layer honesty gate over
`site/src/data/paper-evidence.json` (`runHonestyGate`, line 85 — checks every record has
`environment in {canonical,indicative}+source+value`), then a build-BOUNDARY scan
(`runBuildBoundaryHonestyScan`, line 137) that re-invokes the two shared CI gates
`scripts/check-no-perf-numbers.py --enforce` and `scripts/check-privacy-claims.sh` over the
exact `.typ` sources + evidence JSON (fail-closed). Then `compilePaper` (line 178) shells
`typst compile <src>.typ <slug>.pdf --input data=<paper-evidence.json>` for the PDF and again
with `--format html --features html` for the in-site render, and writes the `<body>` inner HTML
(with a GENERATED provenance comment) to `site/src/generated/papers/<slug>.html`. If typst is
absent it emits placeholder fragments (line 229).

**EVIDENCE BINDING**: every number in a `.typ` comes only through
`site/papers/_lib/bench.typ` — `headline(key)` (line 31) PANICS the compile if the record's
`environment != "canonical"`; `ev(key)` (line 28) is the ungated accessor for indicative
callouts. Same JSON feeds PDF and HTML so they cannot disagree.

**RENDER**: `site/src/app/papers/[slug]/page.tsx` `readPaperHtml(slug)` reads the generated
fragment at build time and injects it via `<PaperHtml>`; `provenanceFor` (line 61) stamps
canonical/indicative counts + commit. `site/src/generated/` is git-ignored (regenerated every
build).

NOTE: `paper-evidence.json` is distinct from `benchmarks.generated.json` (the latter is all
`environment=indicative` work-box timing and may never back a headline).

---

## Per-Paper Verdict Table

| slug | topic | novelty verdict | nearest prior work | target venue | verdict |
|---|---|---|---|---|---|
| `filtered-ann` | RDF-native filtered-ANN: BGP-to-IdMask pre-filter over shared dict-id space, transitive connected-component pushdown, answer-safety invariant | MODEST — a real integration kernel; but ACORN (SIGMOD'24) already accepts arbitrary predicate bitsets; the genuine delta is the shared-dictionary-id space (no metadata mirroring) and connected-component pushdown | ACORN (Patel et al., SIGMOD 2024); NaviX (Sehgal & Salihoglu, arXiv:2506.23397); PathFinder (arXiv:2511.00995); EMA (2606.00734); E2E adaptive termination (2602.06721); unified filtered-ANN benchmark (2509.07789) | ISWC/ESWC research or resources track, or EDBT short. NOT SIGMOD/VLDB (no new algorithm, no evaluation) | **REWRITE** |
| `honest-benchmarking` | Benchmarking-methodology / reproducibility note: gate timing on correctness, label every number's environment, no-hard-coded-numbers, negative-results ledger | NONE — correctness-before-timing, full hardware labelling, and a negative-results ledger are all established good practice (SIGMOD reproducibility norms); paper itself admits it "reports no performance number as evidence" | General RDF/DB benchmarking-reproducibility discourse (WatDiv, WDBench, BSBM; ACM/SIGMOD artifact review) | None as a standalone paper. At most SIGMOD Record / reproducibility-track experience note, or website content | **KILL** |
| `geosparql-optin-crate` | Faithful GeoSPARQL 1.0/1.1 subset as an opt-in crate, OGC topology conformance ratchet floor (119) | ZERO — faithful GeoSPARQL exists in Jena, RDF4J, Virtuoso, Stardog, GraphDB, Oxigraph; opt-in-crate decoupling is routine Rust engineering; the "conformance ratchet" is a CI mechanism, not a result | Apache Jena GeoSPARQL / jena-spatial, RDF4J GeoSPARQL, Virtuoso/Stardog/GraphDB spatial; the OGC GeoSPARQL 1.0/1.1 standard itself | ISWC poster/demo at most. Not a research or resources full paper | **KILL** |
| `solid-acl-conformance` | Hand-curated per-construct decision-parity test corpus for Solid WAC (12) and ACP (12) at the library level, ratcheted in the cross-family scoreboard | ZERO — 24-scenario unit-test suite plus a CI ratchet, honestly scoped to "decision parity, not wire conformance, no security property"; no research question, no technique | Solid Protocol / WAC / ACP specifications; Solid Conformance Test Harness (CTH) and Community Solid Server conformance suites | None / demo. Could at most be an artifact appendix | **MERGE** (into geosparql paper) |
| `odrl-policy-bridge` | Single-node ODRL→access-control view bridge: deny-overrides, asymmetric fail-closed deny retraction, re-checked conditional grants, atomic count enforcement | THIN — a Rust re-instantiation of OAC/Pandit-class ODRL→access-control mappings; the genuinely novel half (federated ODRL→MPC disclosure, ODRL-Duty→ZK obligation) is deferred/unbuilt; fail-closed deny-overrides is the ODRL Formal-Semantics default | OAC - ODRL Profile for Access Control (Esteves, Pandit et al., ESWC 2022); ODRE enforcement framework (arXiv:2409.17602); Slabbinck et al. on ODRL+WAC/ACP integration; UMA usage-control (arXiv:2601.18761) | ISWC/ESWC in-use track or SEMANTiCS/policy workshop — IF it adds an evaluation. Not top-tier as-is | **REWRITE** |
| `unsafe-attestation` | N=1 engineering-discipline write-up: unsafe confined to 5/35 crates by `forbid(unsafe_code)`, 59 sites behind a CI count ratchet, per-site `// SAFETY` register, Miri/oracle/fuzz/ASan coverage matrix | LOW — paper states "we claim no novelty in any one of them"; the claimed contribution is "the composition for one RDF engine"; empirical unsafe-Rust studies already exist | Evans et al. 'Is Rust Used Safely?' (ICSE 2020); Astrauskas et al. 'How Do Programmers Use Unsafe Rust?' (OOPSLA 2020); RustBelt; cargo-geiger / RustSec; Miri | Security-engineering / experience workshop or a resources track at most. NOT a top-tier security venue | **KILL** |
| `cozk-witness-validation` | Adversarial 're-audit' of the engine's INTENDED collaborative zk-SNARK path against CRYPTO'25 eprint 2025/1026 failure modes; every lens RE-OPEN because the path is unbuilt (6 `NotYetImplemented` stubs) | NONE — the audited path is 6 `NotYetImplemented` stubs; there is nothing built to audit, forge against, or measure; the paper applies (does not extend) 2025/1026's results and explicitly "cannot certify soundness" | eprint 2025/1026 'Malicious Security in Collaborative zk-SNARKs' (Garg, Goel, Jain, Roberts, Sekar, CRYPTO'25); Ozdemir & Boneh collaborative zk-SNARKs (USENIX Security 2022); TACEO co-snarks; eprint 2024/143, 2024/940 | None. No PETS/security venue accepts an audit of unbuilt stubs with no construction, no proof, and no evaluation | **NEW_TOPIC_INSTEAD** |

---

## Writing Gaps per Paper

### `filtered-ann` (REWRITE)

- ZERO performance evaluation — explicitly "no wall-clock claim"; a filtered-ANN paper with no
  latency/throughput/recall-vs-latency vs ACORN/NaviX is desk-reject at a DB venue and thin
  even at ISWC.
- Recall floors (0.95/0.90) are a correctness sanity-check on unmodified prior-art HNSW,
  presented as if they were the evaluation.
- Related work is one paragraph and predates the 2025–26 filtered-ANN wave (PathFinder/EMA/
  E2E/unified benchmark).
- No scalability study, no selectivity sweep, no figures — only two 3-column tables.
- Answer-safety is stated as the headline theorem but is near-tautological for an exact mask;
  needs to either prove something harder or be demoted.

### `honest-benchmarking` (KILL)

- It is an opinion/experience essay with no evaluation that the methodology changes any outcome.
- Makes no falsifiable claim by its own admission.
- The "contribution" is a CI gate in this one repo — not generalisable science.
- The whole content belongs as the Evaluation-Methodology subsection of a real systems paper.

### `geosparql-optin-crate` (KILL)

- No evaluation and no comparison to any existing GeoSPARQL engine (coverage or speed).
- The single-row "119 passing assertions" table is a CI floor, not a scientific result.
- The cross-family scoreboard is padded across two papers (also in `solid-acl`) to manufacture
  substance.
- Nothing here generalises beyond this repo's CI config.

### `solid-acl-conformance` (MERGE)

- 12+12 scenarios is a tiny hand-authored corpus presented as a conformance result.
- Explicitly asserts no security/soundness/completeness — so the only claim is "our test count
  is ≥ N".
- No comparison to the actual Solid CTH.
- The cross-family scoreboard section is near-identical to the geosparql paper (self-plagiarised
  padding).

### `odrl-policy-bridge` (REWRITE)

- No comparison or evaluation vs ODRE / the OAC editor+enforcer — the direct competitors.
- Novelty leans on a deferred, unbuilt crypto half that is explicitly out of scope, leaving only
  the non-novel single-node mapping.
- "Answer-safety invariants" are four unit-test booleans, not an evaluation on real policies.
- Related-work concedes the contribution is an integration but does not differentiate from
  ODRE/OAC feature-by-feature.

### `unsafe-attestation` (KILL)

- N=1 case study with no evaluation that the discipline ever caught a real UB (did the
  corruption oracle/fuzzer find anything?).
- The counts (59 sites / 5 crates) are project-internal bookkeeping, uninteresting to an
  outside reader.
- Honestly concedes it is "coverage, not soundness" — so there is no verifiable safety claim,
  only a process description.
- No comparison to how other Rust systems (e.g., other engines/databases) manage their unsafe
  surface.

### `cozk-witness-validation` (NEW_TOPIC_INSTEAD)

- There is no system — the "evidence" is a count of `NotYetImplemented` stubs (6) and RE-OPEN
  verdicts (4).
- A negative result must overturn something that was believed true of a REAL construction; here
  nothing was ever built or claimed sound.
- The honest disposition ("cannot certify anything") is itself the admission that there is no
  reportable finding.
- The durable output (R-WV) is a project CI gate, not a transferable scientific artifact.

---

## Missing Topics That Deserve a Paper

1. **FO-KM empirical finding (STRONGEST missing paper)**: schema.org-as-top beats gUFO,
   DOLCE-DUL and the no-FO incumbent for LLM-agent KM over a Project-Knowledge-Graph
   (`bench/fo-km/RESULTS.md`; measured: schema.org 0.84 vs gUFO 0.58/0.54 and no-FO 0.64
   answer accuracy; the win is driven by LLM training-data fluency, NOT formal richness). This
   is a genuinely novel, counterintuitive, MEASURED result with no direct prior work, and NO
   paper exists yet. Venue: ISWC/ESWC empirical, K-CAP, or a NeSy/LLM+KG workshop. Caveat
   before top-tier: N=16 single counterbalanced run with heuristic grading is too thin — needs
   a larger pre-registered multi-run study.

2. **Out-of-core SPARQL engine systems result (highest-impact thread, NO paper)**: matching/
   beating QLever on compute across few→100M triples while committing single-digit-MB out-of-
   core (6 permutations, lazy/streaming counts, inline tagged ValueIds, bind joins), correctness
   canonically gated by differential fuzz vs Oxigraph. This is the repo's flagship engineering
   and the natural VLDB/CIDR/EDBT paper — yet it has no paper while 3 CI-ratchet artifacts got
   one. Blocker: all timings are NON-canonical work-box/Docker numbers; needs the canonical
   bare-metal runner + native (non-Docker) QLever baseline before any speed/memory headline.

3. **Honest verifiable-federated-SPARQL DESIGN / feasibility-envelope SoK** (should REPLACE
   the `cozk-witness-validation` paper): the composition of ZK query-proofs + MPC + attested
   inputs + full SPARQL + GLOBAL-IRI federation, with the disqualifying-feature argument vs
   node-local-id graph-MPC (GOOSE/SMPG/GORAM) and the 3-axis (adversary × output-guarantee ×
   threshold) capability matrix incl. proven negative results (unbounded property paths
   OUT-OF-REACH). Framed strictly as design/vision (NOT-YET-SOUND, cite the open external-
   audit gate sq-qhy4), this is a real PoPETs/SoK-style contribution, unlike auditing unbuilt
   stubs.

   NOTE — two candidates tested that do NOT deserve their own paper:
   - The zero-overhead shared eval substrate (`crates/sparq-substrate`, wired into
     `sparq-engine`+`sparq-reason`) is a perf-neutral code-move extraction — good engineering,
     but sharing a join/numeric core between an engine and a reasoner is routine (cf
     RDFox/VLog) and not a research novelty.
   - The honest-conformance ratchet methodology is a CI-discipline artifact and is already
     OVER-REPRESENTED (it pads 3 of the 7 existing papers) — it belongs in a reproducibility
     appendix, not a standalone paper.

---

> **Empirical-honesty reminder**: ZK and MPC estates are NOT production-sound until the
> external cryptographer audit sq-qhy4 completes. All work-box benchmarks are non-canonical;
> do not hard-code them in documentation or tests.

---

*Recon captured by Sonnet 4.6 under the Fable program; [SONNET-4.6]*
