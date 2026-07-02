"use client";

// [OPUS-4.8] sq-vw3ax.7 — the COLLAPSED navigation.
//
// Before: a persistent w-64 sidebar rendering the full 6-group/16-surface tree (sidebar-nav.tsx)
// AND a duplicate top-tab bar — the same surfaces in two places, the literal "too much" the
// maintainer flagged (research/website-redesign.md §2, §7). After: ONE slim top bar of content
// destinations + a utility cluster. The sidebar is gone; the 0-click fast path to every surface
// is the Cmd-K palette (sq-vw3ax.1), which is WHY removing the sidebar is now safe — Cmd-K
// shipped first. A mobile Sheet drawer mirrors the same slim links (no full tree).
//
// OPTION-B (the maintainer's decision after #1004 opened, sq-rclb8 / sq-vnd0i). Two DISTINCT
// destinations, not one "Try the GUI": "Try" → /try is the lightweight in-browser SPARQL REPL
// playground (kept unchanged); "App" → /app is the LIVE operational GUI. That GUI is a SEPARATE
// Next.js app (gui/app), overlaid at /app/ by the Pages deploy (pages.yml) — it is NOT a
// route of this site. The old single "Try the GUI" → /gui slot is dropped (/gui now redirects to
// /app).
//
// [OPUS-4.8] sq-vw3ax.11 — the "App" slot is therefore a HARD (full-page) link, not a next/link
// soft navigation: soft-navigating across two distinct Next builds fetches the WRONG app's RSC
// Flight payload (/app/index.txt) and lands on a raw .txt instead of the GUI. See NavLink.
//
// Destinations (research §2 + the maintainer's discoverability gaps), slim at 6:
//   Home · Capabilities · Try · App · Benchmarks · Download
//   utility cluster: { Cmd-K · GitHub · theme }
// Papers stays a real route but lives in Cmd-K (overflow) to keep the bar slim, not bloated.
// (/examples and the deep-page rebuild are sequenced in sq-vw3ax.6 / .4 — not this PR.)

import * as React from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { Github, Menu } from "lucide-react";

import { cn } from "@/lib/utils";
import { withBasePath } from "@/lib/base-path";
import { Button } from "@/components/ui/button";
import {
  Sheet,
  SheetContent,
  SheetTitle,
  SheetTrigger,
} from "@/components/ui/sheet";
import { Logo } from "@/components/logo";
import { ThemeToggle } from "@/components/theme-toggle";
import {
  CommandPalette,
  CommandPaletteTrigger,
} from "@/components/command-palette";
// [OPUS-4.8] sq-ixc3.10 — the operational-command registry. Mounted once inside the palette so
// the workbench (the REPL) can contribute live run / EXPLAIN / connect / export / import /
// switch-workspace / named-graph / recent-query commands to the keyboard-first spine.
import { PaletteCommandsProvider } from "@/components/palette-commands";

const REPO_URL = "https://github.com/jeswr/sparq";

// The slim top bar's content destinations. Capabilities is the single gallery that replaces
// the collapsed /surface/* tree; Try is the lightweight in-browser SPARQL REPL playground; App
// is the live operational GUI destination; Download surfaces the desktop GUI + CLI binaries (the
// maintainer's discoverability ask). Each is a real, built route. Papers is intentionally NOT in
// the bar (it stays a route, reachable via Cmd-K) so the bar stays slim at 6, not bloated.
// `external: true` marks a destination that is served by a SEPARATE deployed app at the same
// origin (here: /app = the gui/app workbench overlaid at /app/). Such a slot must be a hard,
// full-page navigation — see NavLink — not a next/link soft nav (sq-vw3ax.11).
const NAV_ITEMS: { href: string; label: string; external?: boolean }[] = [
  { href: "/", label: "Home" },
  { href: "/capabilities", label: "Capabilities" },
  { href: "/try", label: "Try" },
  { href: "/app", label: "App", external: true },
  { href: "/benchmarks", label: "Benchmarks" },
  { href: "/download", label: "Download" },
];

function useIsActive() {
  const pathname = usePathname();
  return (href: string) =>
    href === "/" ? pathname === "/" : pathname.startsWith(href);
}

function NavLink({
  href,
  children,
  active,
  external,
  onNavigate,
}: {
  href: string;
  children: React.ReactNode;
  active: boolean;
  external?: boolean;
  onNavigate?: () => void;
}) {
  const className = cn(
    "rounded-md px-3 py-1.5 text-sm transition-colors",
    active
      ? "bg-sidebar-accent text-sidebar-accent-foreground font-medium"
      : "text-foreground/70 hover:bg-muted hover:text-foreground",
  );

  // [OPUS-4.8] sq-vw3ax.11 — an `external` destination (e.g. /app) is served in production by a
  // DIFFERENT Next.js app overlaid at /app/ (gui/app, sq-vnd0i), so it must be a hard,
  // full-page navigation. A next/link soft nav across two distinct Next builds would fetch the
  // foreign RSC Flight payload (/app/index.txt) and render a raw .txt instead of the GUI —
  // exactly the bug this fixes. withBasePath prefixes the hand-written absolute href for the
  // /sparq (Pages) vs "" (Tauri) hosts — Next does NOT auto-prefix a plain <a>. The trailing
  // slash matches `trailingSlash: true` so the static export's directory index resolves.
  if (external) {
    return (
      <a
        href={withBasePath(href.endsWith("/") ? href : `${href}/`)}
        onClick={onNavigate}
        aria-current={active ? "page" : undefined}
        className={className}
      >
        {children}
      </a>
    );
  }

  return (
    <Link
      href={href}
      onClick={onNavigate}
      aria-current={active ? "page" : undefined}
      className={className}
    >
      {children}
    </Link>
  );
}

export function AppShell({ children }: { children: React.ReactNode }) {
  const [mobileOpen, setMobileOpen] = React.useState(false);
  const isActive = useIsActive();

  return (
    // The whole shell is wrapped in <CommandPalette> so the global ⌘K / Ctrl-K binding and the
    // header trigger share one palette instance, mounted once. With the sidebar removed, this
    // palette is the discoverability backstop for every surface (sq-vw3ax.1 → .7).
    // [OPUS-4.8] sq-ixc3.10 — <PaletteCommandsProvider> nests inside so the palette (read side)
    // and the workbench (register side) share one operational-command registry.
    <CommandPalette>
      <PaletteCommandsProvider>
      <div className="flex min-h-svh flex-col">
        {/* Sticky slim top bar (h-16, backdrop blur) — the ONE navigation. */}
        <header className="sticky top-0 z-30 flex h-16 items-center gap-2 border-b bg-background/90 px-4 backdrop-blur md:px-6">
          {/* Mobile hamburger → left Sheet drawer with the SAME slim links (no full tree). */}
          <Sheet open={mobileOpen} onOpenChange={setMobileOpen}>
            <SheetTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                className="md:hidden"
                aria-label="Open navigation"
              >
                <Menu className="size-5" />
              </Button>
            </SheetTrigger>
            <SheetContent side="left" className="overflow-y-auto">
              <SheetTitle className="sr-only">Navigation</SheetTitle>
              <Link
                href="/"
                className="mb-4 flex items-center"
                aria-label="sparq home"
                onClick={() => setMobileOpen(false)}
              >
                <Logo className="h-7 w-auto" />
              </Link>
              <nav className="flex flex-col gap-1" aria-label="Primary">
                {NAV_ITEMS.map((item) => (
                  <NavLink
                    key={item.href}
                    href={item.href}
                    active={isActive(item.href)}
                    external={item.external}
                    onNavigate={() => setMobileOpen(false)}
                  >
                    {item.label}
                  </NavLink>
                ))}
              </nav>
            </SheetContent>
          </Sheet>

          <Link href="/" className="flex items-center" aria-label="sparq home">
            <Logo className="h-7 w-auto" />
          </Link>

          {/* The slim top-bar destinations (desktop). */}
          <nav
            className="ml-3 hidden items-center gap-1 md:flex"
            aria-label="Primary"
          >
            {NAV_ITEMS.map((item) => (
              <NavLink
                key={item.href}
                href={item.href}
                active={isActive(item.href)}
                external={item.external}
              >
                {item.label}
              </NavLink>
            ))}
          </nav>

          <div className="ml-auto flex items-center gap-1">
            {/* Try (/try, the REPL) and App (/app, the live GUI) are both first-class slim-bar
                destinations now (Option-B), so the utility cluster is just Cmd-K · GitHub · theme. */}
            {/* The Cmd-K affordance — the fast path to every surface now the sidebar is gone. */}
            <CommandPaletteTrigger className="mr-1 hidden lg:inline-flex" />
            <Button variant="ghost" size="icon" asChild>
              <a
                href={REPO_URL}
                target="_blank"
                rel="noopener noreferrer"
                aria-label="sparq on GitHub"
              >
                <Github className="size-4" />
              </a>
            </Button>
            <ThemeToggle />
          </div>
        </header>

        <main className="flex-1">
          <div className="mx-auto w-full max-w-6xl px-4 py-8 md:px-6 md:py-10">
            {children}
          </div>
        </main>
      </div>
      </PaletteCommandsProvider>
    </CommandPalette>
  );
}
