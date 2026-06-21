import type { Metadata } from "next";

// [OPUS-4.8] sq-vw3ax.3 — the /surface/http-server walkthrough page was folded into the single
// /capabilities gallery (its demo lazily mounts in-place there). Static export cannot 301
// (research §7), so this is a CLIENT REDIRECT STUB that sends inbound links to the surface's
// new home under the "Serve & embed" theme.
import { RedirectStub } from "@/components/redirect-stub";
import { capabilityAnchor } from "@/data/capabilities";

export const metadata: Metadata = {
  title: "Moved to Capabilities",
  description:
    "This surface now lives in the Capabilities gallery under Serve & embed.",
};

export default function http_serverRedirectPage() {
  return (
    <RedirectStub to={capabilityAnchor("serve-embed")} label="Capabilities · Serve & embed" />
  );
}
