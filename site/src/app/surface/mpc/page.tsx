import type { Metadata } from "next";

// [OPUS-4.8] sq-vw3ax.3 — the /surface/mpc walkthrough page was folded into the single
// /capabilities gallery (its demo lazily mounts in-place there). Static export cannot 301
// (research §7), so this is a CLIENT REDIRECT STUB that sends inbound links to the surface's
// new home under the "Privacy (ZK / MPC)" theme.
import { RedirectStub } from "@/components/redirect-stub";
import { capabilityAnchor } from "@/data/capabilities";

export const metadata: Metadata = {
  title: "Moved to Capabilities",
  description:
    "This surface now lives in the Capabilities gallery under Privacy (ZK / MPC).",
};

export default function mpcRedirectPage() {
  return (
    <RedirectStub to={capabilityAnchor("privacy")} label="Capabilities · Privacy (ZK / MPC)" />
  );
}
