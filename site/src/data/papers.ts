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
      "Deterministic conformance ratchet floors: OGC GeoSPARQL topology (119) as one row of a 7-suite cross-family scoreboard totalling 3442, each floor CI-enforced, monotone, and guarded against drift. No latency claim; spatial algorithms are standard prior art.",
  },
  {
    slug: "solid-acl-conformance",
    source: "solid-acl-conformance.typ",
    title:
      "Library-Level Solid Access-Control Conformance: WAC and ACP Decision-Parity Ratchets Joining a Cross-Family Scoreboard",
    blurb:
      "A library-level conformance signal for both Solid access-control models: a per-construct, fail-closed-checked decision-parity corpus for WAC and for ACP, each ratcheted to a monotone floor and registered as a row of the single cross-family scoreboard with a drift guard. The honest evidence is conformance, not speed, and no security property is claimed; HTTP / CTH wire conformance is explicitly out of scope.",
    authors: "Jesse Wright · the sparq project",
    venue: "ISWC resources / in-use track (or demo)",
    status: "publishable-now",
    family: "B",
    evidence:
      "Deterministic conformance ratchet floors: Solid WAC decision parity (12) and Solid ACP decision parity (12), as the two newest rows of a 7-suite cross-family scoreboard totalling 3442, each floor CI-enforced, monotone, and guarded against drift. No latency claim and no security/soundness claim; library-level decision parity only, not HTTP/CTH wire conformance.",
  },
  {
    slug: "unsafe-attestation",
    source: "unsafe-attestation.typ",
    title:
      "Auditing the unsafe: A Confined, Registered, CI-Ratcheted unsafe-Rust Surface as a Machine-Checkable Memory-Safety Attestation for an RDF Engine",
    blurb:
      "How an RDF engine makes its unsafe-Rust surface auditable rather than trusted: confined to 5 of 35 crates by compile-time forbiddance, counted at 59 sites behind a required CI ratchet that gates any growth, justified per-site in a lint-pinned register, and bounded by a layered Miri / corruption-oracle / fuzz / sanitizer coverage matrix. The honest evidence is coverage and discipline, not a proof of soundness: no claim that the engine is free of undefined behaviour, and the open soundness gaps on the untrusted-input mmap boundary are named, not hidden.",
    authors: "Jesse Wright · the sparq project",
    venue: "Security-engineering / resources / in-use track (or workshop)",
    status: "publishable-now",
    family: "A",
    evidence:
      "Deterministic, committed, CI-enforced integers: an unsafe-site count ratchet (59 sites, ceiling-gated), a crate confinement partition (30 forbid + 5 unsafe-bearing = 35), and a 100%-coverage per-site register, plus a Miri/fuzz/ASan coverage matrix. No latency claim and explicitly NOT a proof of memory safety; open soundness gaps on the untrusted-input boundary are stated.",
  },
  {
    slug: "cozk-witness-validation",
    source: "cozk-witness-validation.typ",
    title:
      "Cannot Certify, So We Encode the Obligation: A Collaborative-zk-SNARK Re-Audit as a Witness-Validation Negative Result for Federated SPARQL",
    blurb:
      "An adversarial re-audit of an engine's intended collaborative (multi-prover) zk-SNARK path against the CRYPTO'25 (eprint 2025/1026) failure modes. The honest disposition is a negative result: the path is unbuilt, so soundness cannot be certified and every lens is RE-OPEN. The durable output is R-WV, a witness-validation-before-proving test obligation encoded as a build-time gate. NO security, privacy, or attestation property is claimed.",
    authors: "Jesse Wright · the sparq project",
    venue: "PoPETs / a security workshop (negative-result · security-engineering lessons)",
    status: "wip-arxiv",
    family: "C",
    evidence:
      "Negative result — asserts NO proven security/privacy/soundness/attestation property (the collaborative path is unbuilt: 6 proof/attestation entry points fail closed with NotYetImplemented). Committed structural counts only: 4 re-audit lenses, all RE-OPEN; a 5-clause R-WV witness-validation obligation encoded as a build-time gate; 12 prior single-prover findings under the open external-audit gate. Estate is research-grade and not externally audited; cites the gates sq-qhy4 (external single-prover audit, open) + sq-9hrn (coZK re-audit). No performance claim.",
  },
];

export function paperBySlug(slug: string): Paper | undefined {
  return PAPERS.find((p) => p.slug === slug);
}
