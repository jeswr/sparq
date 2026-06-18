import type { Metadata } from "next";
import { notFound } from "next/navigation";

import { SurfacePlaceholder } from "@/components/surface-placeholder";
import { ALL_SURFACES } from "@/data/surfaces";

// Surfaces routed here are the ones whose interactive page is not built yet
// (the REPL has its own /try page). Surfaces with a real, hand-built content
// page live under src/app/surface/<slug>/page.tsx and are excluded here so the
// static route takes precedence (no generateStaticParams collision).
const BUILT_SURFACES = new Set([
  "sparql",
  "data-formats",
  "javascript-wasm",
  "shacl",
  "inference",
  "mpc",
  "zk",
  "streaming-rsp",
  "full-text",
  "http-server",
]);
const PLACEHOLDER_SURFACES = ALL_SURFACES.filter(
  (s) =>
    s.slug !== "try" &&
    s.slug !== "about" &&
    s.href.startsWith("/surface/") &&
    !BUILT_SURFACES.has(s.slug),
);

export function generateStaticParams() {
  return PLACEHOLDER_SURFACES.map((s) => ({ slug: s.slug }));
}

export const dynamicParams = false;

export async function generateMetadata({
  params,
}: {
  params: Promise<{ slug: string }>;
}): Promise<Metadata> {
  const { slug } = await params;
  const surface = PLACEHOLDER_SURFACES.find((s) => s.slug === slug);
  if (!surface) return {};
  return { title: surface.title, description: surface.blurb };
}

export default async function SurfacePage({
  params,
}: {
  params: Promise<{ slug: string }>;
}) {
  const { slug } = await params;
  const surface = PLACEHOLDER_SURFACES.find((s) => s.slug === slug);
  if (!surface) notFound();
  return <SurfacePlaceholder surface={surface} />;
}
