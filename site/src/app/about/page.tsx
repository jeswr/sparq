import type { Metadata } from "next";

// [OPUS-4.8] sq-vw3ax.7 — /about is demoted: its "what runs where" tier story folds into the
// Home #how-it-runs strip + the "How tiers work" <details> (research/website-redesign.md §2).
// Static export cannot 301 (research §7), so this is a CLIENT REDIRECT STUB that sends inbound
// links to that Home section.
import { RedirectStub } from "@/components/redirect-stub";

export const metadata: Metadata = {
  title: "About — moved to Home",
  description:
    'The "what runs where" tier story now lives in the Home page\'s "How it runs" section.',
};

export default function AboutRedirectPage() {
  return <RedirectStub to="/#how-it-runs" label="How it runs (on the Home page)" />;
}
