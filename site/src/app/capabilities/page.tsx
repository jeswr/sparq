import type { Metadata } from "next";
import Link from "next/link";
import { PlayCircle } from "lucide-react";

// [OPUS-4.8] sq-vw3ax.3 — /capabilities: ONE compact gallery that replaces the 14 dense
// /surface/* pages. Five theme sections, each a list of ~64px rows derived from the single
// GROUPS source (data/surfaces.ts). Each row is one of:
//   * "Open →"  — a retained live deep page (sparql/data-formats/javascript-wasm/shacl/inference)
//   * "Demo ▸"  — a removed walkthrough whose existing component LAZILY mounts in-place; the
//                 demo chunk is fetched only on first expand (components/capabilities/lazy-demo.tsx),
//                 so this gallery is NOT heavier than the 14 pages it replaces — the #1 review risk.
//   * "Source"  — a not-yet-built surface (cli/python).
// The removed /surface/* routes ship client redirect stubs to /capabilities#<theme>.
import { capabilityThemes } from "@/data/capabilities";
import { CapabilityRowItem } from "@/components/capabilities/capability-row";

export const metadata: Metadata = {
  title: "Capabilities",
  description:
    "Every sparq capability in one compact gallery — grouped into five themes. Open a live deep page or expand a demo in place. Each surface is honestly tier-labelled by how it runs.",
};

export default function CapabilitiesPage() {
  const themes = capabilityThemes();
  return (
    <div className="space-y-10">
      <header className="space-y-3">
        <h1 className="text-3xl font-semibold tracking-tight">Capabilities</h1>
        <p className="measure text-muted-foreground">
          The full sparq feature set, grouped into five themes. Open a live deep
          page, or expand a demo in place — each one runs the real engine and is
          labelled by how it runs. For depth, every demo links its crate README and{" "}
          <code className="font-mono">SKILL.md</code>.
        </p>
      </header>

      {themes.map(({ group, rows }) => (
        <section
          key={group.id}
          id={group.id}
          // scroll-mt clears the sticky h-16 header when an in-page #anchor is targeted.
          className="scroll-mt-20 space-y-3"
        >
          <div className="space-y-1">
            <h2 className="text-xl font-semibold">{group.label}</h2>
            <p className="measure text-sm text-muted-foreground">
              {group.description}
            </p>
          </div>

          <div className="divide-y rounded-2xl border bg-card">
            {rows.map(({ surface, kind }) => (
              <CapabilityRowItem
                key={surface.slug}
                slug={surface.slug}
                kind={kind}
              />
            ))}
          </div>

          {group.id === "query-data" && (
            <p className="text-sm">
              <Link
                href="/try"
                className="inline-flex items-center gap-1.5 font-medium text-primary underline-offset-4 hover:underline"
              >
                <PlayCircle className="size-4" aria-hidden />
                Open the full live SPARQL REPL
              </Link>
            </p>
          )}
        </section>
      ))}
    </div>
  );
}
