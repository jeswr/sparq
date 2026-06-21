import type { Metadata } from "next";

// [OPUS-4.8] sq-vw3ax.3 — the /surface/full-text walkthrough page was folded into the single
// /capabilities gallery (its demo lazily mounts in-place there). Static export cannot 301
// (research §7), so this is a CLIENT REDIRECT STUB that sends inbound links to the surface's
// new home under the "Search & GenAI" theme.
import { RedirectStub } from "@/components/redirect-stub";

export const metadata: Metadata = {
  title: "Moved to Capabilities",
  description:
    "This surface now lives in the Capabilities gallery under Search & GenAI.",
};

export default function full_textRedirectPage() {
  return (
    <RedirectStub to="/capabilities#search-genai" label="Capabilities · Search & GenAI" />
  );
}
