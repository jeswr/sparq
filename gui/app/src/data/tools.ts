// [OPUS-4.8] sq-ixc3.9 — the operational TOOLS taxonomy for the left-rail TOOLS list.
//
// Per research/gui-design.md §A.2: each tool is a VERB the user opens as a tab to RUN over the
// live store — NOT a marketing page that describes the feature. Each carries an honesty-TIER
// dot inherited from the engine's existing tier taxonomy (site/src/data/surfaces.ts) so a
// `walkthrough`/`soon` capability is never silently dressed up as `live`.
//
// The foundation shell (this PR) ships the Query tool as a working tab and registers the rest
// as honest STUBS the later phases fill (sq-ixc3.11 turns surfaces into live tools;
// sq-ixc3.12 the query workbench; sq-ixc3.13 the import drawer). A stub tab states plainly
// what it will do and its current tier — it does not fabricate a result.

import type { LucideIcon } from "lucide-react";
import {
  Database,
  Network as GraphIcon,
  ShieldCheck,
  Brain,
  Search,
  Binary,
  MapPin,
  Radio,
  Lock,
  Users,
  Server,
} from "lucide-react";

/**
 * Live-execution tier — the honesty dot. Mirrors the engine's existing tier taxonomy
 * (site/src/data/surfaces.ts) so the GUI inherits the same honest "what runs where" framing.
 *   live           — shipped wasm, in-tab today
 *   live-new-wasm  — a new wasm bundle (portability spike first)
 *   live-bbjs      — 3rd-party WASM (bb.js UltraHonk proving); NOT externally audited
 *   live-sim       — faithful in-tab JS simulation; NOT the native protocol
 *   walkthrough    — captured-I/O replay (native-only / different host)
 *   soon           — not yet built; honest placeholder
 */
export type Tier =
  | "live"
  | "live-new-wasm"
  | "live-bbjs"
  | "live-sim"
  | "walkthrough"
  | "soon";

export interface ToolDef {
  /** Stable id — the tab key + the Cmd-K (later) index key. */
  id: string;
  /** The VERB shown in the rail + the tab label. */
  label: string;
  /** One-line operational description (what running this tool DOES over the live store). */
  blurb: string;
  tier: Tier;
  icon: LucideIcon;
  /** Does a working tab exist yet, or is this an honest stub the later phases fill? */
  built: boolean;
}

/** The TOOLS list, in the rail order from research/gui-design.md §A.2. */
export const TOOLS: ToolDef[] = [
  {
    id: "query",
    label: "Query",
    blurb: "Run SPARQL 1.1/1.2 over the live store — SELECT/ASK/CONSTRUCT/DESCRIBE/UPDATE.",
    tier: "live",
    icon: Database,
    built: true,
  },
  {
    id: "graph-view",
    label: "Graph view",
    blurb: "Node-link visualisation of CONSTRUCT/DESCRIBE results over the live store.",
    tier: "soon",
    icon: GraphIcon,
    built: false,
  },
  {
    id: "shacl",
    label: "SHACL",
    blurb: "Validate the live store against SHACL shapes; W3C report with sh:detail.",
    tier: "live",
    icon: ShieldCheck,
    built: false,
  },
  {
    id: "inference",
    label: "Inference",
    blurb: "Materialise RDFS / OWL-RL / N3 closure over the live store + proof trees.",
    tier: "live-new-wasm",
    icon: Brain,
    built: false,
  },
  {
    id: "full-text",
    label: "Full-text",
    blurb: "BM25 full-text search via magic predicates over the live store.",
    tier: "live-new-wasm",
    icon: Search,
    built: false,
  },
  {
    id: "vector",
    label: "Vector",
    blurb: "kNN over embeddings (vec:nearest); native-only, not yet in the wasm bundle.",
    tier: "walkthrough",
    icon: Binary,
    built: false,
  },
  {
    id: "geosparql",
    label: "GeoSPARQL",
    blurb: "Spatial joins + topology (geof:); native-only, not yet in the wasm bundle.",
    tier: "walkthrough",
    icon: MapPin,
    built: false,
  },
  {
    id: "streaming",
    label: "Streaming",
    blurb: "RSP-QL windowed stream processing with R/I/DSTREAM tick view.",
    tier: "live-new-wasm",
    icon: Radio,
    built: false,
  },
  {
    id: "zk",
    label: "ZK proofs",
    blurb:
      "Commit → prove → verify a query result in-tab (UltraHonk). Research-grade; NOT externally audited.",
    tier: "live-bbjs",
    icon: Lock,
    built: false,
  },
  {
    id: "mpc",
    label: "MPC",
    blurb:
      "Multi-party threshold over additive shares. In-tab JS illustration; NOT the native protocol, NOT live MPC.",
    tier: "live-sim",
    icon: Users,
    built: false,
  },
  {
    id: "server",
    label: "Server",
    blurb:
      "Connect to a running sparq-server (SPARQL 1.1 Protocol) — endpoint mode, subscriptions, health.",
    tier: "walkthrough",
    icon: Server,
    built: false,
  },
];

/** Tier → honesty-dot colour class + short label, for the rail dots and tab headers. */
export const TIER_META: Record<Tier, { dot: string; label: string }> = {
  live: { dot: "bg-[var(--success)]", label: "Live in-tab" },
  "live-new-wasm": { dot: "bg-[var(--success)]", label: "Live (new wasm)" },
  "live-bbjs": { dot: "bg-primary", label: "Live (bb.js); not audited" },
  "live-sim": { dot: "bg-[var(--warning)]", label: "In-tab simulation" },
  walkthrough: { dot: "bg-muted-foreground", label: "Walkthrough (native-only)" },
  soon: { dot: "bg-muted-foreground", label: "Not yet built" },
};

export function toolById(id: string): ToolDef | undefined {
  return TOOLS.find((t) => t.id === id);
}
