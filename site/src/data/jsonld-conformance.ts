// [GPT-5.6] sq-ztdez — public, hand-authored snapshot of the six ratcheted
// W3C JSON-LD 1.1 lanes. These are measured conformance floors, not targets or
// performance results. Keep the denominators paired with their pinned suites.
// TODO(sq-oy1f follow-up): wire generated feed

export interface JsonLdConformanceLane {
  id: "toRdf" | "expand" | "flatten" | "compact" | "frame" | "fromRdf";
  label: string;
  floor: number;
  total: number;
}

export const jsonLdConformanceLanes = [
  { id: "toRdf", label: "JSON-LD to RDF", floor: 413, total: 467 },
  { id: "expand", label: "Expansion", floor: 276, total: 385 },
  { id: "flatten", label: "Flattening", floor: 53, total: 58 },
  { id: "compact", label: "Compaction", floor: 228, total: 246 },
  { id: "frame", label: "Framing", floor: 92, total: 92 },
  { id: "fromRdf", label: "RDF to JSON-LD", floor: 52, total: 53 },
] as const satisfies readonly JsonLdConformanceLane[];
