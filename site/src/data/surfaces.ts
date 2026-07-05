// [OPUS-4.8] sq-8thu — the canonical site IA, derived from the feature-showcase
// design doc (research/feature-showcase-site-design.md §2). One source for the
// sidebar nav AND the landing surface grid. `tier` drives the honesty badge.
//
// [OPUS-4.8] sq-vw3ax.2 — re-grouped ONCE here into the 5 capability THEMES from the
// website-redesign record (research/website-redesign.md §2). Every consumer — the
// sidebar nav, the Home theme grid, the future /capabilities gallery, and the Cmd-K
// command palette — derives from this single GROUPS source, so one edit restructures
// all of them. This commit is a STRUCTURAL regroup only: no surface, blurb, tier, or
// route was added, removed, or reworded — only their grouping (and per-theme metadata)
// changed.

import type { LucideIcon } from "lucide-react";
import {
  Database,
  FileCode2,
  Boxes,
  Brain,
  ShieldCheck,
  Search,
  Sparkles,
  Binary,
  MapPin,
  Radio,
  Lock,
  Network,
  Server,
  Terminal,
  Code2,
  Info,
} from "lucide-react";

/** Live-execution tier — drives the honesty badge colour + label. */
export type Tier =
  | "live" // (a) shipped wasm, in your tab today
  | "live-new-wasm" // (b) a new wasm bundle (portability spike first)
  | "live-bbjs" // (c) 3rd-party WASM (bb.js UltraHonk proving)
  | "live-sim" // (d) faithful in-tab JS simulation
  | "hosted" // (e) hosted sparq-server
  | "walkthrough" // (e) captured-I/O replay (different host / native-only)
  | "soon"; // not yet built — honest placeholder

export interface Surface {
  slug: string;
  href: string;
  title: string;
  blurb: string;
  tier: Tier;
  icon: LucideIcon;
  built?: boolean; // does a real page exist yet?
}

export interface SurfaceGroup {
  /** Stable anchor id — `/capabilities#<id>` + the Home theme grid + Cmd-K group key. */
  id: string;
  label: string;
  /** One-line theme description (the capability the surfaces under it deliver). */
  description: string;
  surfaces: Surface[];
}

export const TIER_LABEL: Record<Tier, string> = {
  live: "Live in your tab",
  "live-new-wasm": "Live (new wasm)",
  "live-bbjs": "Live via bb.js",
  "live-sim": "Live simulation",
  hosted: "Hosted",
  walkthrough: "Walkthrough",
  soon: "Coming soon",
};

export const TIER_VARIANT: Record<
  Tier,
  "success" | "warning" | "muted" | "default"
> = {
  live: "success",
  "live-new-wasm": "success",
  "live-bbjs": "success",
  "live-sim": "default",
  hosted: "warning",
  walkthrough: "muted",
  soon: "muted",
};

export const FLAGSHIPS: Surface[] = [
  {
    slug: "zk-car-hire",
    href: "/showcase/zk-car-hire",
    title: "ZK cross-credential car-hire",
    blurb:
      "Prove you may hire a car — age ≥ 25, valid non-revoked licence, same holder across two credentials — without revealing your documents.",
    tier: "live-bbjs",
    icon: ShieldCheck,
    built: true,
  },
  {
    slug: "mpc-100k",
    href: "/showcase/mpc-100k",
    title: "MPC £100k secure threshold",
    blurb:
      "Four flatmates learn only whether their combined income clears a £100k threshold — no salary, and not even the exact total, is revealed.",
    tier: "live-sim",
    icon: Lock,
    built: true,
  },
  {
    slug: "solid-pairs",
    href: "/showcase/solid-pairs",
    title: "Solid (user, app)-pair result sets",
    blurb:
      "One Pod, one query — different result sets per (agent, client) pair, enforced by the engine's FROM NAMED dataset restriction, live in your tab.",
    tier: "live",
    icon: Network,
    built: true,
  },
];

// [OPUS-4.8] sq-vw3ax.2 — the 5 capability THEMES (research/website-redesign.md §2).
// Surfaces are mapped to themes by what they DO, keeping every existing surface, route,
// blurb and tier verbatim:
//   1. Query & data      — SPARQL + the data-format/JS-WASM/geo query surfaces
//   2. Reason & validate — inference + SHACL
//   3. Search & GenAI    — full-text BM25, vector, GenAI/NLQ
//   4. Privacy (ZK / MPC)— ZK query proofs + MPC federation (the Solid flagship sits in
//      /showcase; it is surfaced via FLAGSHIPS, not duplicated as a /surface row here)
//   5. Serve & embed     — HTTP server, CLI, Python, streaming RSP-QL
// The redesign's aspirational extra rows (structural-similarity, a federation surface
// page) are NOT invented here — there is no such route today, and this regroup adds no
// content. They become beads, not fabricated nav entries.
export const GROUPS: SurfaceGroup[] = [
  {
    id: "query-data",
    label: "Query & data",
    // [OPUS-4.8] sq-4hiqe — the in-tab workbench now lives at /app (the /try REPL was removed);
    // this description is user-facing copy, so it stays clean (no route/bead cross-reference).
    description:
      "Run SPARQL 1.1/1.2 over RDF — the query engine and the formats it ingests.",
    surfaces: [
      {
        slug: "sparql",
        href: "/surface/sparql",
        title: "SPARQL 1.1 / 1.2",
        blurb: "SELECT / ASK / CONSTRUCT / UPDATE, property paths, RDF 1.2 triple terms.",
        tier: "live",
        icon: FileCode2,
        built: true,
      },
      {
        slug: "data-formats",
        href: "/surface/data-formats",
        title: "Data formats",
        blurb: "Turtle / N-Triples / N-Quads / TriG + compressed ingest.",
        tier: "live",
        icon: Database,
        built: true,
      },
      {
        slug: "javascript-wasm",
        href: "/surface/javascript-wasm",
        title: "JavaScript / WASM",
        blurb: "The @jeswr/sparq browser & Node API — streaming cursors, match, applyDelta.",
        tier: "live",
        icon: Boxes,
        built: true,
      },
      {
        slug: "geosparql",
        href: "/surface/geosparql",
        title: "GeoSPARQL",
        blurb: "geof: functions + sf*/eh*/rcc8* + R-tree GeoIndex with a map overlay.",
        // [OPUS-4.8] sq-ndaz: tier-e. sparq-geo is an opt-in native crate (the core engine
        // + lean wasm bundle carry zero geometry code; the server exposes geof: only behind
        // its non-default `geo` feature), and the static Pages site has no backend — so the
        // honest surface is a captured-output walkthrough (the real London–Paris distance
        // < 400 km query, within-polygon spatial join, topology-property rewrite, and R-tree
        // GeoIndex metres, all answer-exact), not a live hosted endpoint.
        tier: "walkthrough",
        icon: MapPin,
        built: true,
      },
    ],
  },
  {
    id: "reason-validate",
    label: "Reason & validate",
    description:
      "Derive new triples and check graphs against shapes — RDFS / OWL 2 RL / N3 closure and SHACL.",
    surfaces: [
      {
        slug: "inference",
        href: "/surface/inference",
        title: "Inference",
        blurb: "RDFS / OWL 2 RL / N3 closure + proof trees.",
        tier: "live-new-wasm",
        icon: Brain,
        built: true,
      },
      {
        slug: "shacl",
        href: "/surface/shacl",
        title: "SHACL",
        blurb: "SHACL Core + SHACL-SPARQL → W3C validation report.",
        tier: "live",
        icon: ShieldCheck,
        built: true,
      },
    ],
  },
  {
    id: "search-genai",
    label: "Search & GenAI",
    description:
      "Find and generate over RDF — BM25 full-text, vector k-NN, and a natural-language → SPARQL loop.",
    surfaces: [
      {
        slug: "full-text",
        href: "/surface/full-text",
        title: "Full-text",
        blurb: "BM25 text search via magic predicates.",
        tier: "live-new-wasm",
        icon: Search,
        built: true,
      },
      {
        slug: "vector",
        href: "/surface/vector",
        title: "Vector",
        blurb: "Embedding store + cosine top-k (HNSW / DiskANN) + k-NN inside SPARQL.",
        // [OPUS-4.8] sq-dwdm: tier-e. sparq-vectors is an opt-in native crate (nothing in
        // the workspace or the lean wasm bundle depends on it, and the vec: magic predicate
        // sits behind the non-default vec-predicate feature), and the static Pages site has
        // no backend — so the honest surface is a captured-output walkthrough (the real
        // Usain Bolt label-embedding run + real vec:nearest / vec:search result tables,
        // answer-exact backend), not a live hosted endpoint.
        tier: "walkthrough",
        icon: Binary,
        built: true,
      },
      {
        slug: "genai",
        href: "/surface/genai",
        title: "GenAI / NLQ",
        blurb: "Schema-card / VoID introspection + natural-language → SPARQL loop.",
        // [OPUS-4.8] sq-3was: tier-e. sparq-nlq is an opt-in native crate (it can pull a
        // model behind a trait), not in the lean wasm bundle, and the static Pages site
        // has no backend — so the honest surface is a captured-output walkthrough (real
        // schema card + real executed result tables, scripted-fixture model step), not a
        // live hosted endpoint.
        tier: "walkthrough",
        icon: Sparkles,
        built: true,
      },
    ],
  },
  {
    id: "privacy",
    label: "Privacy (ZK / MPC)",
    description:
      "Answer queries without revealing the data — zero-knowledge query proofs and threshold MPC federation. Research-grade: the v1 verifier is not externally audited.",
    surfaces: [
      {
        slug: "zk",
        href: "/surface/zk",
        title: "ZK query proofs",
        blurb: "Commitments, BGP + FILTER, issuer attestation, revocation.",
        tier: "live-bbjs",
        icon: Lock,
        built: true,
      },
      {
        slug: "mpc",
        href: "/surface/mpc",
        title: "MPC federation",
        blurb: "Federated SPARQL across distrusting holders (Shamir, threshold).",
        tier: "live-sim",
        icon: Network,
        built: true,
      },
    ],
  },
  {
    id: "serve-embed",
    label: "Serve & embed",
    description:
      "Run sparq as a service or embed it — the HTTP endpoint, the CLI, Python bindings, and streaming RSP-QL.",
    surfaces: [
      {
        slug: "http-server",
        href: "/surface/http-server",
        title: "HTTP server",
        blurb: "SPARQL 1.1 Protocol endpoint, GSP, /metrics, WS/SSE subscriptions.",
        // [OPUS-4.8] sq-rnwc: tier-e. No backend behind the static Pages site, so the
        // honest surface is a captured curl + SSE-frame walkthrough (incl. a live
        // subscription firing on a committed UPDATE), not a live hosted endpoint.
        tier: "walkthrough",
        icon: Server,
        built: true,
      },
      {
        slug: "streaming-rsp",
        href: "/surface/streaming-rsp",
        title: "Streaming RSP",
        blurb: "RSP-QL windows (sliding / tumbling, R/I/DSTREAM).",
        tier: "live-new-wasm",
        icon: Radio,
        built: true,
      },
      {
        slug: "cli",
        href: "/surface/cli",
        title: "CLI",
        blurb: "sparq-cli — query / reason / build / query-mmap.",
        tier: "walkthrough",
        icon: Terminal,
      },
      {
        slug: "python",
        href: "/surface/python",
        title: "Python",
        blurb: "sparq pyo3 bindings.",
        tier: "walkthrough",
        icon: Code2,
      },
    ],
  },
];

// [OPUS-4.8] sq-vw3ax.2 — /about is a UTILITY destination (the "what runs where" matrix),
// not a capability theme, so it is kept OUT of GROUPS (which is now strictly the 5 themes)
// and exported separately. Consumers that previously walked GROUPS for the About entry
// (the sidebar, the About-page table) read this instead, so no route is lost.
export const ABOUT_SURFACE: Surface = {
  slug: "about",
  href: "/about",
  title: "About",
  blurb: 'Architecture and the honest "what runs where" matrix.',
  tier: "live",
  icon: Info,
  built: true,
};

/** Every capability surface across the 5 themes (excludes the /about utility page). */
export const ALL_SURFACES: Surface[] = GROUPS.flatMap((g) => g.surfaces);

/** Capability surfaces + the /about utility page — the complete navigable surface set. */
export const ALL_SURFACES_WITH_ABOUT: Surface[] = [...ALL_SURFACES, ABOUT_SURFACE];
