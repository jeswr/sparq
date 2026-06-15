import type { Metadata } from "next";
import { notFound } from "next/navigation";

import { SurfacePlaceholder } from "@/components/surface-placeholder";
import { ALL_SURFACES } from "@/data/surfaces";

// Surfaces routed here are the ones whose interactive page is not built yet
// (the REPL has its own /try page). Static export needs every slug enumerated.
const PLACEHOLDER_SURFACES = ALL_SURFACES.filter(
  (s) => s.slug !== "try" && s.slug !== "about" && s.href.startsWith("/surface/"),
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
