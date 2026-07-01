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
      "Filter-as-Query: Filtered Approximate Nearest-Neighbour Search over SPARQL, where the Filter is an Exact BGP over the Engine's Own Dictionary Ids",
    blurb:
      "An RDF-native filtered-ANN integration: the filter on a vector-neighbour variable is the join-connected sub-BGP of the query itself, evaluated exactly by the engine and materialised as an id-mask over the shared dictionary-id space (no metadata mirroring, no boundary id translation), with the pre≡post answer contract enforced as machine-checked invariants — exact path by construction, approximate path preconditioned and verified on broad-mask fixtures. Related work covers the 2023–26 filtered-ANN wave incl. the engine-integrated systems (VBASE/NaviX) it is closest to; the performance evaluation is pre-registered (baselines, workloads, falsification criteria) pending the canonical runner.",
    authors: "Jesse Wright · the sparq project",
    venue:
      "EDBT short / demo or ESWC in-use/resources — results-free systems description; research-track submission deferred until the pre-registered evaluation is executed",
    status: "draft",
    family: "A",
    evidence:
      "Deterministic only: recall floors (0.95 unfiltered, 0.90 filtered — sanity checks, labelled as such) + asserted pre-filter ≡ post-filter equivalence (single / transitive / cyclic; exact path unconditional, approximate path broad-mask) + the cost-model crossover constant. No latency/throughput claim; the performance evaluation is pre-registered but unexecuted (blocked on the canonical runner).",
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
    // [FABLE-5] sq-gum8.3-odrl REVISION 2 (PR #1330 review response): ONE track picked (ESWC
    // research track — the "in-use" framing is dropped: library-level, single-node, no
    // deployment/users); the WAC/ACP floors reframed as CONTEXT for the target layer, not
    // bridge evidence (the only direct bridge evidence is the four invariants); related work
    // extended to the decision-caching / authorization-recycling / materialised-view
    // neighborhood for C3; the conflict default corrected against ODRL IM 2.2 (`invalid` is
    // the spec default — the bridge hard-wires `prohibit` and cannot honour `perm`/`invalid`,
    // a first-class limitation); "pre-registered" corrected to "specified" (no registry
    // deposit exists); worked end-to-end example + artifact statement added. Status stays
    // `draft` until the §5.3 comparative study runs — the explicit bar for submission.
    slug: "odrl-policy-bridge",
    source: "odrl-policy-bridge.typ",
    title:
      "An ODRL Policy Bridge for SPARQL Access Control: Fail-Closed Compilation of Usage Policies into a Queryable Solid Access-Control View",
    blurb:
      "Compile, don't co-evaluate: a matched ODRL Permission/Prohibition compiles into the same triples the engine's existing, queryable WAC/ACP view already understands — no second enforcement engine, and every compiled decision is auditable, provenance-tagged RDF (the paper walks one policy end-to-end: Turtle policy → compiled auth/provenance triples → SPARQL audit query and its result). The contribution is the fail-closed lifecycle discipline compilation demands: ODRL's prohibit conflict strategy realised structurally by allow-minus-deny set subtraction (the perm and spec-default invalid strategies are not representable — a disclosed, first-class limitation), asymmetric three-valued deny retraction on policy refresh (positioned against decision caching, authorization recycling, Zanzibar-style consistent authorization, and materialised-view maintenance — not only ODRL enforcers), per-session re-checked conditional grants with safe one-shot fallback, and atomic count budgets. Evidence, honestly: four machine-checked invariants are the only direct bridge evidence; the WAC/ACP decision-parity floors are context for the pre-existing target layer, not bridge evaluation; the specified comparative decision-agreement study vs ODRE/OAC has not run (hence draft — not submittable until it has). The federated ODRL→MPC / ODRL-Duty→ZK composition is deferred, unbuilt, and claims nothing (sq-qhy4).",
    authors: "Jesse Wright · the sparq project",
    venue:
      "ESWC research track (policy) — draft; not submittable until the §5.3 comparative study runs",
    status: "draft",
    family: "B",
    evidence:
      "Honestly partitioned. Direct evidence (all of it, today): deterministic, test-proven answer-safety invariants of the bridge (prohibit-strategy set subtraction correct through unchanged enforcement; asymmetric fail-closed deny retraction; recipient constraints persist as re-checked conditional grants with one-shot fallback; atomic stateful count enforcement). Context — NOT bridge evidence: the CI-enforced, drift-guarded Solid WAC (12) and ACP (12) decision-parity ratchet floors of the pre-existing target layer. PENDING — the blocking gap to submission: a fully specified (not pre-registered: no registry deposit exists) decision-agreement study vs the ODRE enforcement engines and an OAC-style matcher over the cited systems' own policy corpora. No latency claim; no novel-semantics claim; no in-use claim; the federated/ZK disclosure half is explicitly deferred (research-grade crypto, not externally audited; sq-qhy4).",
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
