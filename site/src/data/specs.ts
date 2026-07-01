// [OPUS-4.8] sq-rvgr2.1 — the spec-factory registry, the /specs analogue of src/data/papers.ts.
// ONE source of truth for the /specs index, the per-spec static routes
// (generateStaticParams), the Cmd-K palette entry, and the build step
// (scripts/build-specs.mjs reads slug/source/title/shortName/status/date/editors from this
// file via parallel-array regexes — keep every field a SINGLE-LINE string literal).
//
// To register a new draft: add an entry here + a site/specs/<source>.typ authored with the
// helpers in site/specs/_lib/spec.typ. Everything else (index card, route, ToC, PDF + HTML
// build) is data-driven off this list.
//
// HONESTY: every draft in this series is a W3C-style *Unofficial Proposal Draft* — it has NO
// W3C standing and is NOT a standard (the Status-of-This-Document box on each page states this
// plainly, and the `status` label below never claims otherwise).

export type SpecStatus = "unofficial" | "cg-draft" | "draft";

export interface Spec {
  /** URL slug AND the basename of the built PDF/HTML artifacts. */
  slug: string;
  /** The .typ source under site/specs/ (relative path). */
  source: string;
  /** Document title — also injected into the doc header so it can't drift from this card. */
  title: string;
  /** ReSpec-style short name (a stable identifier for the draft). */
  shortName: string;
  /** Readiness/standing. NONE of these is a W3C standard. */
  status: SpecStatus;
  /** ISO date the draft was last built/edited. */
  date: string;
  /** Editors line, rendered in the doc header. */
  editors: string;
  /** One-line summary for the index card + page subtitle (not parsed by the build step). */
  blurb: string;
}

export const STATUS_LABEL: Record<SpecStatus, string> = {
  unofficial: "Unofficial Proposal Draft",
  "cg-draft": "Draft Community Group Report",
  draft: "Editor's Draft",
};

export const STATUS_VARIANT: Record<
  SpecStatus,
  "success" | "warning" | "muted" | "default"
> = {
  unofficial: "muted",
  "cg-draft": "warning",
  draft: "default",
};

export const SPECS: Spec[] = [
  {
    slug: "mpc-sparql",
    source: "mpc-sparql.typ",
    title:
      "MPC-SPARQL: Secure Multi-Party Federated SPARQL — Requirements and Reference Architecture",
    shortName: "mpc-sparql",
    status: "unofficial",
    date: "2026-07-01",
    editors: "Jesse Wright · the sparq project",
    blurb:
      "A requirements and reference-architecture draft (not yet a protocol specification — the interoperable byte formats are explicitly open) for evaluating federated SPARQL across mutually distrusting data sources under secure multi-party computation: consolidated adversary model, secret-share and join-key encodings with a 61-bit-field statistical-security account, per-operator disclosure routing, verifier-side binding rules, related-work positioning, and testable-now vs future-implementation conformance categories separating what is built (M0–M3) from what is designed-not-built (the collaborative proof, distributed attestation, deployed transport). Nothing is production-claimable today: the planned external audit clears the single-prover ZK layer only, and the MPC layer needs its own.",
  },
  {
    slug: "proposed-specifications-template",
    source: "proposed-specifications-template.typ",
    title: "SPARQ Proposed Specifications — Template",
    shortName: "sparq-spec-template",
    status: "unofficial",
    date: "2026-07-01",
    editors: "Jesse Wright · the sparq project",
    blurb:
      "The template and worked example every sparq Unofficial Proposal Draft follows — status notice, numbered sections, an RFC 2119 conformance section, a worked example, and references. Proves the single-source PDF + in-site render pipeline.",
  },
];

export function specBySlug(slug: string): Spec | undefined {
  return SPECS.find((s) => s.slug === slug);
}
