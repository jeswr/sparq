// [OPUS-4.8] sq-vw3ax.16 — the honest competitive feature matrix, data layer.
//
// This is a FAITHFUL transcription of section 1 of
// research/competitive-feature-analysis-2026-07.md — the five-way competitor
// research fan-out (RDFox, Stardog, GraphDB, TopBraid EDG, and the enterprise
// cluster Neptune / AllegroGraph / Virtuoso / Neo4j-n10s / MarkLogic) scored
// against the sparq self-inventory.
//
// HONESTY CONTRACT (the sq-vw3ax.16 invariant, mirrors the design record):
//   * NO performance numbers live here — measured, same-box comparative evidence is
//     itself open work (beads sq-hmd7l / sq-vw3ax.12); the page LINKS the benchmark
//     dashboard and never inlines a figure.
//   * Competitor marks are their PUBLISHED capabilities/claims as surveyed in the
//     design record, not measured by us.
//   * A PARTIAL / GAP row that names TRACKED future work carries its governing bead id
//     so the row links to its tracker issue (build-in-public); a few explicitly-deferred
//     items are not yet beaded and say so in the note ("deferred" / "not beaded"). The
//     data test test/competitive-matrix.test.mjs enforces this — every such row either
//     carries a bead or openly flags itself untracked — so the copy and the data cannot
//     silently drift apart.
//   * The ZK/MPC row stays caveated: the estate is research-grade, the v1 verifier is
//     pending external audit (sq-qhy4) and sparq-mpc is honest-majority semi-honest
//     only — no settled privacy/soundness property is claimed.
//
// The page (app/compare/page.tsx) is a pure renderer over this data.

/** A competitor's support for a feature, as surveyed in the design record. */
export type Mark = "yes" | "partial" | "no";

/**
 * sparq's honest standing on a feature, bucketed for colour. The precise, nuanced
 * wording from the design record (e.g. "LEAD on breadth", "PARITY-PLUS") lives in
 * `tierLabel`; the note carries the full honest framing.
 */
export type Tier = "LEAD" | "PARITY" | "PARTIAL" | "GAP" | "SKIP";

export interface MatrixRow {
  feature: string;
  /** Competitor marks, in the column order [RDFox, Stardog, GraphDB, TopBraid, Ent.]. */
  rdfox: Mark;
  stardog: Mark;
  graphdb: Mark;
  topbraid: Mark;
  ent: Mark;
  /** Colour bucket for the sparq-status badge. */
  tier: Tier;
  /** Optional precise wording shown on the badge (defaults to `tier`). */
  tierLabel?: string;
  /** The honest one-line framing of sparq's standing. */
  note: string;
  /**
   * Governing tracker beads — rendered as build-in-public links. Optional: a PARTIAL/GAP
   * row that names future work carries its bead(s) here when the work is TRACKED; a few
   * explicitly-deferred items are not yet beaded and instead flag that in `note`.
   */
  beads?: string[];
}

export interface MatrixSection {
  id: string;
  title: string;
  rows: MatrixRow[];
}

export const COMPETITORS = [
  { key: "rdfox", label: "RDFox" },
  { key: "stardog", label: "Stardog" },
  { key: "graphdb", label: "GraphDB" },
  { key: "topbraid", label: "TopBraid" },
  { key: "ent", label: "Ent." },
] as const;

export const MATRIX: MatrixSection[] = [
  {
    id: "reasoning",
    title: "Reasoning",
    rows: [
      {
        feature: "OWL profile coverage (RL / EL / QL / …)",
        rdfox: "partial", stardog: "yes", graphdb: "partial", topbraid: "partial", ent: "no",
        tier: "LEAD", tierLabel: "LEAD · breadth",
        note: "RL forward-chaining + a complete EL classifier + a QL CQ-gate, with a Direct-Semantics checker in progress. RDFox has no EL/QL classification.",
        beads: ["sq-6tykl", "sq-pbz04"],
      },
      {
        feature: "Materialization with incremental insert",
        rdfox: "yes", stardog: "no", graphdb: "yes", topbraid: "no", ent: "no",
        tier: "PARITY",
        note: "sparq-reason has incremental maintenance; per-dimension perf vs RDFox's published claims is an open, same-box bench axis (linked, never inlined).",
        beads: ["sq-hmd7l"],
      },
      {
        feature: "Incremental deletion at FBF grade",
        rdfox: "yes", stardog: "no", graphdb: "yes", topbraid: "no", ent: "no",
        tier: "PARTIAL",
        note: "Incremental deletion exists, but deletion-heavy efficiency is unmeasured versus RDFox's FBF / GraphDB's smooth-delete. Bead: a deletion-workload bench + retraction optimization.",
        beads: ["sq-6tykl.4", "sq-hmd7l"],
      },
      {
        feature: "Stratified negation + aggregation in recursive rules",
        rdfox: "yes", stardog: "partial", graphdb: "partial", topbraid: "no", ent: "no",
        tier: "GAP",
        note: "N3/RIF support does not yet cover stratified AGGREGATE()/negation-as-failure in recursive rules with incremental maintenance — the single biggest engine gap versus RDFox (XL, design-record-first).",
        beads: ["sq-6tykl.3"],
      },
      {
        feature: "owl:sameAs canonical-representative rewriting",
        rdfox: "yes", stardog: "no", graphdb: "yes", topbraid: "no", ent: "no",
        tier: "PARTIAL",
        note: "Union-Find sameAs inside the OWL-RL fixpoint exists; store-level representative rewriting is absent and deferred (even RDFox cannot compose it with negation/aggregation).",
        beads: ["sq-6w7x6"],
      },
      {
        feature: "Explanation / proof trees (engine)",
        rdfox: "yes", stardog: "yes", graphdb: "partial", topbraid: "partial", ent: "no",
        tier: "PARITY",
        note: "The explain feature and why(triple) proof trees exist in the engine; wiring them into the workbench UI is the remaining gap (see Workbench).",
        beads: ["sq-ixc3.20"],
      },
      {
        feature: "Query-time rewriting mode + multiple named schemas",
        rdfox: "no", stardog: "yes", graphdb: "no", topbraid: "no", ent: "no",
        tier: "PARTIAL",
        note: "QL is rewriting-shaped, but a first-class “reason under schema X per query” toggle is absent. Deferred and noted for the reasoner program.",
      },
      {
        feature: "SWRL ingestion",
        rdfox: "yes", stardog: "yes", graphdb: "no", topbraid: "no", ent: "no",
        tier: "GAP",
        note: "Only a migration-story parser is envisaged. Small; deferred until migration demand appears (not beaded).",
      },
      {
        feature: "Reasoning profiler (per-rule cost)",
        rdfox: "yes", stardog: "partial", graphdb: "no", topbraid: "no", ent: "no",
        tier: "GAP",
        note: "What makes a rule engine debuggable; sparq-introspect and the slow-query ring are seeds. Small bead.",
        beads: ["sq-6tykl.5"],
      },
      {
        feature: "SHACL-AF rules + sh:values node expressions",
        rdfox: "no", stardog: "no", graphdb: "no", topbraid: "yes", ent: "no",
        tier: "LEAD", tierLabel: "LEAD · among engines",
        note: "A full node-expression algebra + sh:values + a gated W3C SHACL-AF harness landed. TopBraid is the only peer elsewhere, and it is not an engine.",
      },
    ],
  },
  {
    id: "engine",
    title: "Engine / query capabilities",
    rows: [
      {
        feature: "SPARQL 1.1 conformance",
        rdfox: "partial", stardog: "yes", graphdb: "yes", topbraid: "yes", ent: "partial",
        tier: "LEAD",
        note: "A full W3C SPARQL conformance ratchet plus SPARQL 1.2 triple terms; RDFox's SPARQL surface trails its reasoner. Conformance evidence lives on the assurance page, not inlined here.",
      },
      {
        feature: "RDF-star / RDF 1.2",
        rdfox: "partial", stardog: "partial", graphdb: "yes", topbraid: "partial", ent: "partial",
        tier: "LEAD", tierLabel: "LEAD-candidate",
        note: "Native triple terms in the engine and parsers; Stardog's own docs flag known perf problems here, making it a bench wedge once same-box numbers exist.",
        beads: ["sq-hmd7l"],
      },
      {
        feature: "PATHS full-path queries (endpoints + intermediate nodes, VIA)",
        rdfox: "no", stardog: "yes", graphdb: "partial", topbraid: "no", ent: "partial",
        tier: "GAP",
        note: "W3C property paths return endpoints only. Full-path queries are a de-facto-standard extension and pure engine work on home turf (L).",
        beads: ["sq-lsp7k.3"],
      },
      {
        feature: "Standing queries with incremental answer deltas",
        rdfox: "yes", stardog: "no", graphdb: "no", topbraid: "no", ent: "no",
        tier: "PARTIAL",
        note: "/subscriptions streams result deltas over WS/SSE today; the RDFox differentiator is true incremental view maintenance (no re-eval) + retrievable change history (L/XL).",
        beads: ["sq-lsp7k.6"],
      },
      {
        feature: "Anytime queries (partial results under budget + resumption)",
        rdfox: "no", stardog: "no", graphdb: "no", topbraid: "no", ent: "yes",
        tier: "GAP",
        note: "Budgets and timeouts exist but return errors, not partial results plus a continuation token. Matters the day anyone hosts a public sparq endpoint (M).",
        beads: ["sq-lsp7k.4"],
      },
      {
        feature: "In-query graph analytics (PageRank etc. composable in queries)",
        rdfox: "no", stardog: "no", graphdb: "no", topbraid: "no", ent: "yes",
        tier: "PARTIAL",
        note: "sparq-algos has the algorithms; the SPARQL-callable surface is designed and was blocked on sign-off. Unblocking the existing bead beats new work.",
        beads: ["sq-mg5hk"],
      },
      {
        feature: "Facet-count fast path (grouped class/predicate/value distributions)",
        rdfox: "no", stardog: "no", graphdb: "no", topbraid: "no", ent: "yes",
        tier: "GAP",
        note: "An engine-side aggregate fast path + API; the columnar layout is well-suited, and it feeds the faceted-browse UI (M).",
        beads: ["sq-lsp7k.5"],
      },
      {
        feature: "Point-in-time query (arbitrary timestamp)",
        rdfox: "no", stardog: "partial", graphdb: "yes", topbraid: "partial", ent: "partial",
        tier: "PARTIAL",
        note: "time-travel pins queries to recent generations (the ring); a CDC stream and PITR backup deltas exist. The gap is a persistent temporal index for arbitrary timestamps (L).",
        beads: ["sq-lsp7k.7"],
      },
      {
        feature: "ACID + MVCC multi-user serving",
        rdfox: "yes", stardog: "yes", graphdb: "yes", topbraid: "yes", ent: "yes",
        tier: "PARTIAL",
        note: "Update-in-place, copy-on-write snapshots, and CDC exist; the held serving PRs (ArcSwap swap-in, plan cache, replanner) are the path. Already maintainer-flagged; no new bead.",
        beads: ["sq-0g6g"],
      },
      {
        feature: "HA / replication",
        rdfox: "partial", stardog: "yes", graphdb: "yes", topbraid: "no", ent: "yes",
        tier: "SKIP",
        note: "A deliberate non-goal for now: single-node dominance first, with the durable CDC stream as the future seed. RDFox's own HA is architecturally modest, so no moat is lost.",
      },
      {
        feature: "Persistence / snapshot-restore / encryption at rest",
        rdfox: "yes", stardog: "yes", graphdb: "yes", topbraid: "no", ent: "yes",
        tier: "PARTIAL",
        note: "An on-disk format, mmap, and online backup + delta PITR ship; at-rest encryption is researched but not shipped. A checkbox, deferred.",
        beads: ["sq-o5bi", "sq-bu1a"],
      },
    ],
  },
  {
    id: "integration",
    title: "Integration / interop",
    rows: [
      {
        feature: "SPARQL SERVICE federation",
        rdfox: "partial", stardog: "yes", graphdb: "yes", topbraid: "no", ent: "partial",
        tier: "LEAD",
        note: "A full streaming federation client: service-description discovery, TPF/brTPF, pushdown, bind-vs-hash planning, adaptive re-planning, and fail-closed egress. RDFox gained federation only recently.",
      },
      {
        feature: "MCP server",
        rdfox: "no", stardog: "no", graphdb: "yes", topbraid: "yes", ent: "no",
        tier: "PARITY", tierLabel: "PARITY-PLUS",
        note: "sparq-mcp shipped. The extension bead adds shape-aware editing tools, FTS/vector/facet tools, and stored templates.",
        beads: ["sq-lsp7k.10"],
      },
      {
        feature: "Tabular→RDF ingest (CSV / R2RML materializing import)",
        rdfox: "yes", stardog: "yes", graphdb: "yes", topbraid: "partial", ent: "yes",
        tier: "GAP",
        note: "The #1 enterprise adoption funnel (“my data is in CSV/Postgres”). A CLI-first slice: CSV direct mapping + R2RML materializing import on the sparq-arrow seed (M).",
        beads: ["sq-lsp7k.8"],
      },
      {
        feature: "Virtual graphs / SQL pushdown (query external DBs in place)",
        rdfox: "yes", stardog: "yes", graphdb: "yes", topbraid: "no", ent: "yes",
        tier: "GAP", tierLabel: "GAP · scoped out",
        note: "Full per-dialect virtualization is a multi-year platform play. sparq's counter-story is materializing import + reload speed; only a one-dialect bridge if demanded. Not beaded.",
      },
      {
        feature: "BI/SQL wire-protocol facade (Tableau/Power BI connect directly)",
        rdfox: "no", stardog: "yes", graphdb: "no", topbraid: "no", ent: "no",
        tier: "GAP",
        note: "A big rock (XL). Almost no OSS RDF store does this, and Rust has mature pgwire crates. Parked at P3 pending maintainer appetite.",
        beads: ["sq-lsp7k.12"],
      },
      {
        feature: "GraphQL over RDF / shapes",
        rdfox: "no", stardog: "yes", graphdb: "yes", topbraid: "yes", ent: "no",
        tier: "GAP",
        note: "A bounded opt-in crate (shapes→schema, read-only) once forms land; low evidence of demand versus SPARQL/BI. Deferred at P3.",
        beads: ["sq-lsp7k.13"],
      },
      {
        feature: "Kafka / CDC change feed out",
        rdfox: "no", stardog: "no", graphdb: "yes", topbraid: "no", ent: "no",
        tier: "PARITY",
        note: "A durable, replayable CDC stream landed on the substance; Kafka packaging is niche and skipped.",
      },
      {
        feature: "Notebook UX (%%magic + hosted notebooks)",
        rdfox: "no", stardog: "no", graphdb: "no", topbraid: "no", ent: "yes",
        tier: "GAP",
        note: "A %%sparq magic in sparq-py plus a JupyterLite/WASM zero-install demo; graph-notebook already speaks generic SPARQL, so server compat may be nearly free (S/M).",
        beads: ["sq-lsp7k.11"],
      },
      {
        feature: "Third-party explorer compat (graph-explorer etc.)",
        rdfox: "no", stardog: "no", graphdb: "no", topbraid: "no", ent: "yes",
        tier: "GAP",
        note: "Verify and document sparq-server as a graph-explorer SPARQL target for a free visualization story. Trivial; folded into the notebook bead.",
        beads: ["sq-lsp7k.11"],
      },
      {
        feature: "JSON-LD 1.1 full pipeline",
        rdfox: "no", stardog: "partial", graphdb: "partial", topbraid: "partial", ent: "partial",
        tier: "LEAD",
        note: "Native, dependency-free expansion / flattening / compaction / framing, default-on.",
      },
      {
        feature: "HDT read + write",
        rdfox: "no", stardog: "no", graphdb: "no", topbraid: "no", ent: "no",
        tier: "LEAD",
        note: "HDT write support is rare anywhere; sparq reads and writes the archive format.",
      },
      {
        feature: "Arrow columnar results",
        rdfox: "no", stardog: "no", graphdb: "no", topbraid: "no", ent: "no",
        tier: "LEAD",
        note: "sparq-arrow exposes results in the Arrow columnar format.",
      },
      {
        feature: "Python / JS bindings",
        rdfox: "yes", stardog: "yes", graphdb: "yes", topbraid: "yes", ent: "yes",
        tier: "PARITY",
        note: "sparq-py plus a WASM binding; the RDF/JS conformance program is open.",
        beads: ["sq-iwhl8", "sq-xqchl"],
      },
    ],
  },
  {
    id: "ml",
    title: "ML / vector / NLQ",
    rows: [
      {
        feature: "Vector index architecture",
        rdfox: "partial", stardog: "partial", graphdb: "partial", topbraid: "no", ent: "yes",
        tier: "LEAD", tierLabel: "LEAD · architecture",
        note: "Native in-engine HNSW + DiskANN + quantization, and the only in-browser graph+vector story. Honest caveat: raw ANN perf versus purpose-built vector systems is being closed under a matched-recall Pareto gate.",
        beads: ["sq-lhcot"],
      },
      {
        feature: "NL→SPARQL grounded loop",
        rdfox: "no", stardog: "yes", graphdb: "yes", topbraid: "yes", ent: "yes",
        tier: "PARITY", tierLabel: "PARITY-PLUS",
        note: "The core ground→generate→validate→execute→repair loop plus a measured cheap-model NL tool. Wiring it into the GUI is an existing bead; the agent-platform zoo is skipped.",
        beads: ["sq-96o1"],
      },
      {
        feature: "LLM-in-SPARQL magic predicates",
        rdfox: "no", stardog: "no", graphdb: "yes", topbraid: "no", ent: "yes",
        tier: "SKIP",
        note: "A deliberate non-goal: nondeterminism inside evaluation muddies semantics and caching. MCP (the LLM calls the database) is the right direction, and sparq has it.",
      },
      {
        feature: "GNN pipelines",
        rdfox: "no", stardog: "no", graphdb: "no", topbraid: "no", ent: "yes",
        tier: "SKIP",
        note: "Interop instead: .spqv embedding import/export already exists.",
      },
      {
        feature: "Full-text search in SPARQL",
        rdfox: "partial", stardog: "yes", graphdb: "yes", topbraid: "no", ent: "yes",
        tier: "PARITY", tierLabel: "PARITY-PLUS",
        note: "Native BM25 text: magic predicates, in-process, in the wasm build; a bench axis versus jena-text is in flight (linked, never inlined).",
        beads: ["sq-hmd7l"],
      },
    ],
  },
  {
    id: "shacl",
    title: "SHACL & validation",
    rows: [
      {
        feature: "SHACL Core validation",
        rdfox: "yes", stardog: "yes", graphdb: "yes", topbraid: "yes", ent: "partial",
        tier: "PARITY", tierLabel: "PARITY-PLUS",
        note: "Full Core + SHACL-SPARQL + custom components + the W3C 1.2 suite, plus SHACL-AF that no engine peer offers.",
      },
      {
        feature: "Validation callable in-query (violations as rows)",
        rdfox: "yes", stardog: "yes", graphdb: "no", topbraid: "no", ent: "no",
        tier: "GAP",
        note: "Small deltas on existing crates: expose validate as a function/SERVICE returning rows, with an asserted-vs-inferred fact-domain switch once reasoner closure is queryable (M).",
        beads: ["sq-lsp7k.2"],
      },
      {
        feature: "Guard mode (transactions rejected on violation)",
        rdfox: "no", stardog: "yes", graphdb: "yes", topbraid: "yes", ent: "no",
        tier: "GAP",
        note: "Commit-time enforcement in sparq-server, folded into the validate-in-query bead; also feeds the forms commit path.",
        beads: ["sq-lsp7k.2"],
      },
      {
        feature: "SHACL-driven forms (authoring UX)",
        rdfox: "no", stardog: "no", graphdb: "no", topbraid: "yes", ent: "no",
        tier: "GAP", tierLabel: "GAP · white space",
        note: "No engine vendor contests it — everyone stops at validation. The headless core, live validation, SPARQL UPDATE commit, and native GUI renderer already land; the MCP agent surface and shape editor are planned. See the SHACL-forms capability page.",
        beads: ["sq-lsp7k.1"],
      },
      {
        feature: "DASH suggestions (one-click constraint repair)",
        rdfox: "no", stardog: "no", graphdb: "no", topbraid: "yes", ent: "no",
        tier: "GAP",
        note: "Parameterized SPARQL-UPDATE repair actions; small once forms exist, and agent-friendly (M).",
        beads: ["sq-lsp7k.1"],
      },
    ],
  },
  {
    id: "workbench",
    title: "Workbench UI",
    rows: [
      {
        feature: "SPARQL editor + results + saved state",
        rdfox: "yes", stardog: "yes", graphdb: "yes", topbraid: "yes", ent: "yes",
        tier: "PARITY",
        note: "An editor with highlighting/completion, multi-view results, workspaces, and a command palette.",
        beads: ["sq-ixc3"],
      },
      {
        feature: "Visual query-plan explorer",
        rdfox: "partial", stardog: "yes", graphdb: "no", topbraid: "no", ent: "no",
        tier: "GAP", tierLabel: "GAP · cheap",
        note: "The engine already emits a typed plan tree, per-operator q-error, and EXPLAIN ANALYZE; the GUI panel is rendering work that turns the perf mandate into a demo (M).",
        beads: ["sq-ixc3.19"],
      },
      {
        feature: "Click-to-explain inferred facts (proof-tree UI)",
        rdfox: "yes", stardog: "yes", graphdb: "no", topbraid: "partial", ent: "no",
        tier: "GAP", tierLabel: "GAP · cheap",
        note: "The engine why() already exists; wire it into the Inference tool and results/graph views. No OSS engine offers this (M).",
        beads: ["sq-ixc3.20"],
      },
      {
        feature: "Dataset summary views (class bubble/chord, domain-range)",
        rdfox: "no", stardog: "no", graphdb: "yes", topbraid: "no", ent: "no",
        tier: "GAP", tierLabel: "GAP · cheap",
        note: "Two or three aggregate queries plus the sparq-introspect schema card / VoID that is already computed (S/M).",
        beads: ["sq-ixc3.21"],
      },
      {
        feature: "Graph viz of results + expansion",
        rdfox: "yes", stardog: "yes", graphdb: "yes", topbraid: "partial", ent: "yes",
        tier: "PARITY",
        note: "Parity on the basics. Gap slices: SPARQL-configurable expansion lenses and RDF-star annotation rendering on edges (almost no viz shows the latter).",
        beads: ["sq-ixc3.22"],
      },
      {
        feature: "Faceted browse (search→expand→filter with counts)",
        rdfox: "no", stardog: "partial", graphdb: "no", topbraid: "partial", ent: "yes",
        tier: "GAP",
        note: "The UI plus the engine facet API, including an auto map view on geo literals (GeoSPARQL is already in). Depends on the facet fast path (M/L).",
        beads: ["sq-ixc3.23", "sq-lsp7k.5"],
      },
      {
        feature: "Visual query builder (diagram→SPARQL)",
        rdfox: "no", stardog: "yes", graphdb: "no", topbraid: "no", ent: "yes",
        tier: "GAP",
        note: "Emit honest SPARQL into the editor, with shape-aware suggestions once forms land (L).",
        beads: ["sq-ixc3.24"],
      },
      {
        feature: "Autocomplete index + rank-ordered defaults",
        rdfox: "no", stardog: "yes", graphdb: "yes", topbraid: "yes", ent: "partial",
        tier: "GAP",
        note: "A dedicated IRI/label autocomplete index that stays snappy at large scale, used workbench-wide, with PageRank-ordered defaults (sparq-algos has PageRank) (M, engine + GUI).",
        beads: ["sq-lsp7k.9"],
      },
      {
        feature: "Instance editing / authoring",
        rdfox: "no", stardog: "no", graphdb: "partial", topbraid: "yes", ent: "partial",
        tier: "GAP",
        note: "The forms program — see the SHACL-driven forms row above.",
        beads: ["sq-lsp7k.1"],
      },
      {
        feature: "Ontology / shape editor (meta-circular)",
        rdfox: "no", stardog: "yes", graphdb: "no", topbraid: "yes", ent: "no",
        tier: "GAP",
        note: "A follow-on after forms: the forms engine pointed at shape resources IS the shape editor. Not beaded yet; noted as forms phase 2.",
      },
      {
        feature: "Server-side parameterized query/update templates",
        rdfox: "no", stardog: "partial", graphdb: "yes", topbraid: "no", ent: "no",
        tier: "GAP",
        note: "Prepared/parameterized queries exist in-engine; add named server-side templates over REST + MCP + GUI as a safe update surface for apps and agents (S/M).",
        beads: ["sq-lsp7k.10"],
      },
      {
        feature: "NL chat in workbench",
        rdfox: "no", stardog: "yes", graphdb: "yes", topbraid: "yes", ent: "no",
        tier: "PARTIAL",
        note: "The NL→SPARQL loop exists (see ML/vector/NLQ); wiring it into the workbench chat is an existing bead.",
        beads: ["sq-96o1"],
      },
      {
        feature: "Admin / monitoring / kill-query",
        rdfox: "yes", stardog: "yes", graphdb: "yes", topbraid: "yes", ent: "yes",
        tier: "PARTIAL", tierLabel: "PARITY-PARTIAL",
        note: "A health panel, Prometheus metrics, and subscriptions shipped; the kill-query UI is folded into the plan-explorer bead's acceptance.",
        beads: ["sq-ixc3.19"],
      },
    ],
  },
  {
    id: "governance",
    title: "Governance / trust / deployment",
    rows: [
      {
        feature: "Access-control expressiveness",
        rdfox: "yes", stardog: "yes", graphdb: "yes", topbraid: "yes", ent: "yes",
        tier: "LEAD", tierLabel: "LEAD · language",
        note: "WAC + ACP + ODRL fail-closed evaluation (nobody else has ODRL). The gap is plumbing: OIDC/JWT/API-keys/RBAC is already-flagged open work.",
      },
      {
        feature: "ZK / MPC / verifiable queries / canonicalization / VC",
        rdfox: "no", stardog: "no", graphdb: "no", topbraid: "no", ent: "no",
        tier: "LEAD", tierLabel: "LEAD · research-grade",
        note: "No competitor plays here at all. Honest status: research-grade and unaudited — the v1 verifier is remediated but external accredited-cryptographer sign-off is pending (sq-qhy4), and sparq-mpc is honest-majority semi-honest only. No settled guarantee is claimed.",
        beads: ["sq-qhy4"],
      },
      {
        feature: "Provenance / lineage",
        rdfox: "yes", stardog: "partial", graphdb: "yes", topbraid: "yes", ent: "partial",
        tier: "PARITY", tierLabel: "PARITY-PLUS",
        note: "PROV-O lineage for derived graphs plus the attestation estate.",
      },
      {
        feature: "Versioning / audit",
        rdfox: "no", stardog: "partial", graphdb: "yes", topbraid: "yes", ent: "partial",
        tier: "PARTIAL",
        note: "See the point-in-time row: the gap is a persistent temporal index (L).",
        beads: ["sq-lsp7k.7"],
      },
      {
        feature: "Edge / embedded / browser",
        rdfox: "yes", stardog: "no", graphdb: "no", topbraid: "no", ent: "no",
        tier: "LEAD",
        note: "WASM in-page, with reasoning / SHACL / FTS / streaming included, runs where RDFox cannot; a Rust FFI covers on-device. WASM builds stay first-class.",
      },
    ],
  },
];
