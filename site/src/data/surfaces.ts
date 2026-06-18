// [OPUS-4.8] sq-8thu — the canonical site IA, derived from the feature-showcase
// design doc (research/feature-showcase-site-design.md §2). One source for the
// sidebar nav AND the landing surface grid. `tier` drives the honesty badge.

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
  PlayCircle,
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
  label: string;
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

export const GROUPS: SurfaceGroup[] = [
  {
    label: "Core engine",
    surfaces: [
      {
        slug: "try",
        href: "/try",
        title: "Live SPARQL REPL",
        blurb: "Run real SPARQL against a sample graph — the engine compiled to wasm, in your tab.",
        tier: "live",
        icon: PlayCircle,
        built: true,
      },
      {
        slug: "sparql",
        href: "/surface/sparql",
        title: "SPARQL 1.1 / 1.2",
        blurb: "SELECT / ASK / CONSTRUCT / UPDATE, property paths, RDF-star.",
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
    ],
  },
  {
    label: "Reasoning & validation",
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
    label: "Search & retrieval",
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
        blurb: "Embedding store + cosine top-k (HNSW / DiskANN) + hybrid fusion.",
        tier: "hosted",
        icon: Binary,
      },
      {
        slug: "genai",
        href: "/surface/genai",
        title: "GenAI / NLQ",
        blurb: "Schema-card / VoID introspection + natural-language → SPARQL loop.",
        tier: "hosted",
        icon: Sparkles,
      },
    ],
  },
  {
    label: "Spatial & streaming",
    surfaces: [
      {
        slug: "geosparql",
        href: "/surface/geosparql",
        title: "GeoSPARQL",
        blurb: "geof: functions + R-tree GeoIndex with a map overlay.",
        tier: "hosted",
        icon: MapPin,
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
    ],
  },
  {
    label: "Privacy (ZK + MPC)",
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
    label: "Serving & hosts",
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
  {
    label: "About",
    surfaces: [
      {
        slug: "about",
        href: "/about",
        title: "About",
        blurb: "Architecture and the honest \"what runs where\" matrix.",
        tier: "live",
        icon: Info,
        built: true,
      },
    ],
  },
];

export const ALL_SURFACES: Surface[] = GROUPS.flatMap((g) => g.surfaces);
