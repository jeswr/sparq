// [OPUS-4.8] sq-gum8 — the /papers section shell. The outer AppShell already provides the
// global nav + header; this just constrains the reading width for paper content.
export default function PapersLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return <div className="mx-auto w-full max-w-3xl">{children}</div>;
}
