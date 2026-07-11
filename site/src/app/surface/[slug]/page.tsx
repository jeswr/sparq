import type { Metadata } from "next";
import { notFound } from "next/navigation";

// [OPUS-4.8] sq-vw3ax.3 — the /surface/[slug] catch-all now serves CLIENT REDIRECT STUBS for
// the removed walkthrough routes (the redesign collapsed the 14 /surface/* pages into the
// single /capabilities gallery). Static export cannot 301 (research §7 must_fix), so each old
// path prerenders a tiny stub that client-redirects to /capabilities#<theme>.
//
// The 5 retained live deep pages (sparql / data-formats / javascript-wasm / shacl / inference)
// keep their hand-built /surface/<slug>/page.tsx, which take precedence over this catch-all (no
// generateStaticParams collision). The other surfaces that had their OWN hand-built folder
// (geosparql, full-text, vector, genai, http-server, streaming-rsp, zk, mpc) ship their own
// per-folder stub; this catch-all covers the remaining removed slugs that never had a folder
// (cli, python — the not-yet-built placeholders), so every inbound /surface/* link resolves.
import { RedirectStub } from "@/components/redirect-stub";
import { GROUPS } from "@/data/surfaces";
import { DEEP_PAGE_SLUGS, capabilityAnchor } from "@/data/capabilities";

// Slugs that already own a folder under src/app/surface/<slug> (a deep page OR a per-folder
// redirect stub) MUST be excluded here, or generateStaticParams collides with that static
// route. This catch-all only handles the surfaces that have NO folder of their own.
const FOLDER_SLUGS = new Set([
  ...DEEP_PAGE_SLUGS,
  "geosparql",
  "full-text",
  "vector",
  "genai",
  "http-server",
  "streaming-rsp",
  "zk",
  "mpc",
  // [FABLE-5] sq-vw3ax.13 — federation ships its own per-folder redirect stub.
  "federation",
]);

interface Stub {
  slug: string;
  anchor: string;
  themeLabel: string;
}

const STUBS: Stub[] = GROUPS.flatMap((group) =>
  group.surfaces
    .filter(
      (s) =>
        s.href.startsWith("/surface/") &&
        s.slug !== "try" &&
        !FOLDER_SLUGS.has(s.slug),
    )
    .map((s) => ({
      slug: s.slug,
      anchor: capabilityAnchor(group.id),
      themeLabel: group.label,
    })),
);

export function generateStaticParams() {
  return STUBS.map((s) => ({ slug: s.slug }));
}

export const dynamicParams = false;

export async function generateMetadata({
  params,
}: {
  params: Promise<{ slug: string }>;
}): Promise<Metadata> {
  const { slug } = await params;
  const stub = STUBS.find((s) => s.slug === slug);
  if (!stub) return {};
  return {
    title: "Moved to Capabilities",
    description: `This surface now lives in the Capabilities gallery under ${stub.themeLabel}.`,
  };
}

export default async function SurfaceRedirectPage({
  params,
}: {
  params: Promise<{ slug: string }>;
}) {
  const { slug } = await params;
  const stub = STUBS.find((s) => s.slug === slug);
  if (!stub) notFound();
  return (
    <RedirectStub
      to={stub.anchor}
      label={`Capabilities · ${stub.themeLabel}`}
    />
  );
}
