// [OPUS-4.8] sq-vjn4 — the /benchmarks section shell: a left sidebar with one entry per
// benchmark TYPE (capability family) beside the section content. The site's outer
// AppShell already provides the global nav + header; this adds the per-section type nav.
import { FAMILIES, familyMetricCount } from "@/data/benchmarks";
import { TypeSidebar } from "@/components/benchmarks/type-sidebar";

export default function BenchmarksLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const items = FAMILIES.map((f) => ({
    key: f.key,
    title: f.title,
    count: familyMetricCount(f.key),
  }));

  return (
    <div className="flex flex-col gap-8 lg:flex-row lg:gap-10">
      <aside className="lg:w-56 lg:shrink-0">
        <div className="lg:sticky lg:top-24">
          <h2 className="px-2 pb-1 text-xs font-medium uppercase tracking-wide text-muted-foreground/80">
            Benchmark types
          </h2>
          <TypeSidebar items={items} />
        </div>
      </aside>
      <div className="min-w-0 flex-1">{children}</div>
    </div>
  );
}
