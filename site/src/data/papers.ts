// [OPUS-4.8] sq-gum8 — the paper-factory registry. ONE source of truth for the /papers
// index, the per-paper static routes (generateStaticParams), the sidebar nav, and the PDF
// build step (scripts/build-papers.mjs reads `slug` + `source` from this file).
//
// To register a new paper: add an entry here + a site/papers/<source>.typ. Everything else
// (index card, route, nav link, PDF + HTML build) is data-driven off this list.
//
// HONESTY: `status` is the paper-factory readiness verdict from
// research/paper-contributions-inventory.md. A C-family (ZK/MPC) paper would be `wip-arxiv`
// and must cite sq-qhy4; neither pilot is C-family. Numbers inside each paper are gated to
// `environment: canonical` by papers/_lib/bench.typ + scripts/build-papers.mjs.

export type PaperStatus = "publishable-now" | "wip-arxiv" | "draft";
export type PaperFamily = "A" | "B" | "C";

export interface Paper {
  /** URL slug AND the basename of the built PDF/HTML artifacts. */
  slug: string;
  /** The .typ source under site/papers/ (relative path). */
  source: string;
  title: string;
  /** One-line summary for the index card + page subtitle. */
  blurb: string;
  /** Authors; rendered as "anonymized" by the .typ when an anon build is requested. */
  authors: string;
  /** Target venue / track (honest target, not an acceptance). */
  venue: string;
  status: PaperStatus;
  family: PaperFamily;
  /** What the headline evidence is — surfaced on the card so the framing is honest up front. */
  evidence: string;
}

export const STATUS_LABEL: Record<PaperStatus, string> = {
  "publishable-now": "Publishable now",
  "wip-arxiv": "WIP · arXiv only",
  draft: "Draft",
};

export const STATUS_VARIANT: Record<
  PaperStatus,
  "success" | "warning" | "muted" | "default"
> = {
  "publishable-now": "success",
  "wip-arxiv": "warning",
  draft: "muted",
};

export const FAMILY_LABEL: Record<PaperFamily, string> = {
  A: "Systems / DB",
  B: "Semantic Web",
  C: "Crypto (WIP)",
};

export const PAPERS: Paper[] = [
  {
    slug: "filtered-ann",
    source: "filtered-ann.typ",
    title:
      "Filter-as-Query: Predicate-Constrained Vector Search where the Filter is an Exact RDF BGP over the Engine's Own Dictionary Ids",
    blurb:
      "An RDF-native filtered-ANN: the filter compiles to an exact, transitively-pushed-down BGP mask over the shared dictionary-id space, making pre-filtering answer-safe. A systems-integration contribution.",
    authors: "Jesse Wright · the sparq project",
    venue: "ISWC / ESWC research track (systems-integration)",
    status: "publishable-now",
    family: "A",
    evidence:
      "Deterministic: recall floors (0.95 unfiltered, 0.90 filtered) + proven pre-filter ≡ post-filter equivalence (single / transitive / cyclic). No latency claim.",
  },
  {
    slug: "honest-benchmarking",
    source: "honest-benchmarking.typ",
    title:
      "Honest Same-Box Benchmarking for RDF Engines: Differential-Correctness-Gated, Hardware-Labelled, Negative-Results-Inclusive",
    blurb:
      "A benchmarking methodology: correctness gates timing, every number is environment-labelled, canonical evidence is separated from indicative work-box measurement by a build-time gate, and negative results are first-class.",
    authors: "Jesse Wright · the sparq project",
    venue: "Reproducibility / E&A track (methods note)",
    status: "publishable-now",
    family: "A",
    evidence:
      "Methodology contribution — reports no performance number as evidence; demonstrates the canonical/indicative honesty gate this factory enforces.",
  },
  {
    slug: "geosparql-optin-crate",
    source: "geosparql-optin-crate.typ",
    title:
      "A Conformant, Opt-In GeoSPARQL Layer for a Dictionary-Id RDF Engine, Backed by a Cross-Family Conformance Ratchet",
    blurb:
      "An opt-in GeoSPARQL 1.0/1.1 layer that imposes no cost on a core or wasm build that never uses it, reusing the host engine's extension-function registry and generic entailment. The honest evidence is conformance, not speed: an OGC topology floor that is one row of a single cross-family, CI-enforced, drift-guarded ratchet.",
    authors: "Jesse Wright · the sparq project",
    venue: "ISWC resources / in-use track (or demo)",
    status: "publishable-now",
    family: "B",
    evidence:
      "Deterministic conformance ratchet floors: OGC GeoSPARQL topology (119) as one row of a 5-suite cross-family scoreboard totalling 3418, each floor CI-enforced, monotone, and guarded against drift. No latency claim; spatial algorithms are standard prior art.",
  },
];

export function paperBySlug(slug: string): Paper | undefined {
  return PAPERS.find((p) => p.slug === slug);
}
