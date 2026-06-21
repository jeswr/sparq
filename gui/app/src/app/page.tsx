import { Workbench } from "@/components/workbench/workbench";

// [OPUS-4.8] sq-ixc3.9 — the single route: the operational workbench shell. The GUI has NO
// route tree (no /about, /benchmarks, /showcase) — it is a workbench, not a site. Tools open
// as TABS inside the shell, not as pages.
export default function Page() {
  return <Workbench />;
}
