// [OPUS-4.8] sq-rvgr2.1 — the /specs section shell, mirroring papers/layout.tsx. The outer
// AppShell already provides the global nav + header; this just constrains the reading width
// for specification content.
export default function SpecsLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return <div className="mx-auto w-full max-w-3xl">{children}</div>;
}
