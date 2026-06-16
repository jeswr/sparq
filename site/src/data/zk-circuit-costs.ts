// [OPUS-4.8] sq-1s2 family — typed accessor over the per-member ZK circuit cost +
// semantics snapshot (src/data/zk-circuit-costs.json). The JSON is the SINGLE SOURCE
// OF TRUTH for every gate count, timing and "what this proves" string the site
// renders; nothing here hard-codes a number or a semantics claim. Gate counts are the
// DETERMINISTIC `bb gates -s ultra_honk` snapshot; prove_ms_* are INDICATIVE /
// NON-CANONICAL and are null where unmeasured (rendered as "—", never guessed).
//
// PRIVACY-CLAIMS GATE (sq-qhy4 / sq-9hrn / sq-1s2): the composition verifier is
// NOT-yet-sound, the estate is research-grade and NOT externally audited, the MPC
// layer is semi-honest. Every consumer surfaces the standing caveats verbatim from
// the JSON — a passing/fast proof is not a soundness or privacy guarantee.

import raw from "@/data/zk-circuit-costs.json";

/** One circuit-family member, exactly as captured in the snapshot JSON. */
export interface ZkCircuitMember {
  member: string;
  /** Plain-language statement the circuit proves (as designed / as landed). */
  proves: string;
  /** The cryptographic primitive the member is built on. */
  primitive: string;
  /** True only for the one statement proven live in-browser today (filter_int_d2). */
  exercised_live: boolean;
  /** Deterministic `bb gates -s ultra_honk` circuit_size. */
  gate_count: number;
  /** Indicative single-thread prove ms (non-canonical); null = unmeasured. */
  prove_ms_t1: number | null;
  /** Indicative multi-thread prove ms (non-canonical); null = unmeasured. */
  prove_ms_tN: number | null;
  source: string;
}

export interface ConstantFamilyCosts {
  proof_bytes_noir_recursive: number;
  proof_bytes_evm_keccak: number;
  vk_bytes: number;
  verify_ms_aarch64: number;
  note: string;
}

interface ZkCircuitCostsFile {
  snapshot_source: string;
  gate_tool: string;
  bb_version: string;
  nargo_version: string;
  timing_caveat: string;
  soundness_caveat: string;
  constant_family_costs: ConstantFamilyCosts;
  members: ZkCircuitMember[];
}

const data = raw as ZkCircuitCostsFile;

export const ZK_MEMBERS: readonly ZkCircuitMember[] = data.members;
export const ZK_CONSTANT_COSTS: ConstantFamilyCosts = data.constant_family_costs;
export const ZK_GATE_TOOL = data.gate_tool;
export const ZK_BB_VERSION = data.bb_version;
export const ZK_NARGO_VERSION = data.nargo_version;
export const ZK_TIMING_CAVEAT = data.timing_caveat;
export const ZK_SOUNDNESS_CAVEAT = data.soundness_caveat;

/** Members sorted by gate count, heaviest first — the cost-breakdown ordering. */
export const ZK_MEMBERS_BY_GATES: readonly ZkCircuitMember[] = [...data.members].sort(
  (a, b) => b.gate_count - a.gate_count,
);

/** The single live-in-browser member (filter_int_d2), if present. */
export const ZK_LIVE_MEMBER: ZkCircuitMember | undefined = data.members.find(
  (m) => m.exercised_live,
);

/** Coarse operator family inferred from the member id (filter / scan / join / …). */
export type ZkFamilyKey =
  | "filter"
  | "scan"
  | "join"
  | "issuer"
  | "holder"
  | "revoke";

export function zkFamilyOf(member: string): ZkFamilyKey {
  if (member.startsWith("filter")) return "filter";
  if (member.startsWith("scan")) return "scan";
  if (member.startsWith("join")) return "join";
  if (member.startsWith("hidden_issuer")) return "issuer";
  if (member.startsWith("holder")) return "holder";
  return "revoke";
}

/** Hotspot extremes, derived (not hard-coded) for the cost-breakdown call-outs. */
export const ZK_HEAVIEST: ZkCircuitMember = ZK_MEMBERS_BY_GATES[0];
export const ZK_CHEAPEST: ZkCircuitMember =
  ZK_MEMBERS_BY_GATES[ZK_MEMBERS_BY_GATES.length - 1];
export const ZK_ISSUER_MEMBER: ZkCircuitMember | undefined = data.members.find(
  (m) => zkFamilyOf(m.member) === "issuer",
);
