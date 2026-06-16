import type { Metadata } from "next";
import { notFound } from "next/navigation";

import { SurfacePlaceholder } from "@/components/surface-placeholder";
import { FLAGSHIPS } from "@/data/surfaces";

export function generateStaticParams() {
  // Flagships with a hand-built page (e.g. zk-car-hire) own their own route under
  // /showcase/<slug>/page.tsx, so they must NOT also be generated here — a static
  // route and a dynamic param for the same path collide at export.
  return FLAGSHIPS.filter((s) => !s.built).map((s) => ({ slug: s.slug }));
}

export const dynamicParams = false;

export async function generateMetadata({
  params,
}: {
  params: Promise<{ slug: string }>;
}): Promise<Metadata> {
  const { slug } = await params;
  const surface = FLAGSHIPS.find((s) => s.slug === slug);
  if (!surface) return {};
  return { title: surface.title, description: surface.blurb };
}

export default async function ShowcasePage({
  params,
}: {
  params: Promise<{ slug: string }>;
}) {
  const { slug } = await params;
  const surface = FLAGSHIPS.find((s) => s.slug === slug);
  if (!surface) notFound();
  return <SurfacePlaceholder surface={surface} flagship />;
}
