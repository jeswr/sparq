import type { Metadata } from "next";
import { notFound } from "next/navigation";

import { SurfacePlaceholder } from "@/components/surface-placeholder";
import { FLAGSHIPS } from "@/data/surfaces";

// The MPC (/showcase/mpc-100k) and Solid (/showcase/solid-pairs) flagships have
// their own real pages; the remaining flagships fall back to the honest
// placeholder until their pages are built.
const HAS_OWN_PAGE = new Set(["mpc-100k", "solid-pairs"]);

export function generateStaticParams() {
  return FLAGSHIPS.filter((s) => !HAS_OWN_PAGE.has(s.slug)).map((s) => ({
    slug: s.slug,
  }));
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
