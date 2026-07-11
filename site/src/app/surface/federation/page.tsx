import type { Metadata } from "next";

// [FABLE-5] sq-vw3ax.13 — the federation surface lives in the /capabilities gallery
// (its captured-output walkthrough lazily mounts in-place there, under "Serve & embed").
// Static export cannot 301 (research §7), so — like every other demo surface — the
// /surface/federation route is a CLIENT REDIRECT STUB, giving inbound links a real page.
import { RedirectStub } from "@/components/redirect-stub";
import { capabilityAnchor } from "@/data/capabilities";

export const metadata: Metadata = {
  title: "Moved to Capabilities",
  description:
    "This surface now lives in the Capabilities gallery under Serve & embed.",
};

export default function federationRedirectPage() {
  return (
    <RedirectStub to={capabilityAnchor("serve-embed")} label="Capabilities · Serve & embed" />
  );
}
