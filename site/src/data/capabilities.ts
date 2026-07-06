// [OPUS-4.8] sq-vw3ax.3 / .7 — the /capabilities + redirect-stub routing table, derived
// from the single GROUPS source (data/surfaces.ts). This is the ONE place that says, per
// surface, whether on /capabilities it is:
//
//   * a DEEP-PAGE row ("Open →") — one of the 5 retained live deep pages that still live at
//     their own /surface/<slug> route (sparql, data-formats, javascript-wasm, shacl,
//     inference). These are NOT removed and keep their hand-built deep page.
//   * a DEMO row ("Demo ▸") — a surface whose /surface/<slug> walkthrough page is REMOVED and
//     whose existing interactive component is now lazily mounted in-place on /capabilities
//     (see components/capabilities/lazy-demo.tsx). Its old route ships a client redirect stub
//     to /capabilities#<theme> (static export cannot 301).
//   * a SOON row — a surface with no built page yet (cli, python); a plain "coming soon" row
//     that links out to GitHub. Its old /surface/<slug> route is a placeholder today and is
//     redirected too.
//
// Keeping this derived from GROUPS means a regroup in surfaces.ts restructures the gallery,
// the sidebar history, the Home theme grid, and Cmd-K in one edit.

import { ALL_SURFACES, GROUPS, type Surface, type SurfaceGroup } from "@/data/surfaces";

/** Resolve a surface (incl. its non-serializable icon) by slug — client-side lookup so a
 *  server component can hand a client row just the serializable slug. */
export function surfaceBySlug(slug: string): Surface | undefined {
  return ALL_SURFACES.find((s) => s.slug === slug);
}

/** The 5 surfaces that KEEP a hand-built deep page at /surface/<slug> (research §2). */
export const DEEP_PAGE_SLUGS = new Set([
  "sparql",
  "data-formats",
  "javascript-wasm",
  "shacl",
  "inference",
]);

export type CapabilityKind = "deep" | "demo" | "soon";

export interface CapabilityRow {
  surface: Surface;
  /** Stable theme anchor — the surface's row lives under /capabilities#<themeId>. */
  themeId: string;
  kind: CapabilityKind;
}

/** Classify one surface for the gallery. `built === false` (cli/python) → "soon". */
export function classify(surface: Surface): CapabilityKind {
  if (DEEP_PAGE_SLUGS.has(surface.slug)) return "deep";
  if (surface.built === false || surface.built === undefined) {
    // cli / python have no `built: true` flag and no page → honest "coming soon" row.
    return surface.slug === "cli" || surface.slug === "python" ? "soon" : "demo";
  }
  return "demo";
}

/** The /capabilities anchor for a surface (e.g. "/capabilities/#privacy").
 *
 * [OPUS-4.8] sq-bpoey — the trailing slash BEFORE the fragment is load-bearing. With
 * next.config.ts `trailingSlash:true` the page is emitted to out/capabilities/index.html;
 * a slashless `/capabilities#id` link is remapped by scripts/check-site-links.sh to
 * file://<out>/capabilities (a bare path with no index.html), so lychee --include-fragments
 * cannot find the `#id` heading and the pages.yml link-check fails. The `/capabilities/#id`
 * form remaps to file://<out>/capabilities/ -> out/capabilities/index.html where the
 * `<section id="id">` lives, matching every other internal link's basePath+trailingSlash shape.
 * Browsers resolve `/capabilities/#id` to the same page+anchor, so the client redirect is
 * unaffected. */
export function capabilityAnchor(themeId: string): string {
  return `/capabilities/#${themeId}`;
}

/**
 * Every surface that previously had a /surface/<slug> route which is now REMOVED (a demo or
 * soon row) — used to generate the client-side redirect stubs. The 5 deep pages are excluded
 * (they keep their own route). The map value is the /capabilities anchor the stub redirects to.
 */
export function removedSurfaceRoutes(): { slug: string; anchor: string }[] {
  const out: { slug: string; anchor: string }[] = [];
  for (const group of GROUPS) {
    for (const s of group.surfaces) {
      if (DEEP_PAGE_SLUGS.has(s.slug)) continue; // deep pages keep their own route
      if (!s.href.startsWith("/surface/")) continue; // only /surface/* routes are removed
      out.push({ slug: s.slug, anchor: capabilityAnchor(group.id) });
    }
  }
  return out;
}

/** The themed gallery model: each theme + its classified rows. */
export interface CapabilityTheme {
  group: SurfaceGroup;
  rows: CapabilityRow[];
}

export function capabilityThemes(): CapabilityTheme[] {
  return GROUPS.map((group) => ({
    group,
    rows: group.surfaces.map((surface) => ({
      surface,
      themeId: group.id,
      kind: classify(surface),
    })),
  }));
}
