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
// Destinations (research §2 + the maintainer's discoverability gap):
//   Home · Capabilities · Benchmarks · Papers · Download   +   Try the GUI
//   utility cluster: { Cmd-K · GitHub · theme }
// (/examples and the deep-page rebuild are sequenced in sq-vw3ax.6 / .4 — not this PR.)

import * as React from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { Github, Menu, MonitorPlay } from "lucide-react";

import { cn } from "@/lib/utils";
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

const REPO_URL = "https://github.com/jeswr/sparq";

// The slim top bar's content destinations. Capabilities is the single gallery that replaces
// the collapsed /surface/* tree; Download surfaces the desktop GUI + CLI binaries (the
// maintainer's discoverability ask). Each is a real, built route.
const NAV_ITEMS: { href: string; label: string }[] = [
  { href: "/", label: "Home" },
  { href: "/capabilities", label: "Capabilities" },
  { href: "/benchmarks", label: "Benchmarks" },
  { href: "/papers", label: "Papers" },
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
  onNavigate,
}: {
  href: string;
  children: React.ReactNode;
  active: boolean;
  onNavigate?: () => void;
}) {
  return (
    <Link
      href={href}
      onClick={onNavigate}
      aria-current={active ? "page" : undefined}
      className={cn(
        "rounded-md px-3 py-1.5 text-sm transition-colors",
        active
          ? "bg-sidebar-accent text-sidebar-accent-foreground font-medium"
          : "text-foreground/70 hover:bg-muted hover:text-foreground",
      )}
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
    <CommandPalette>
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
                    onNavigate={() => setMobileOpen(false)}
                  >
                    {item.label}
                  </NavLink>
                ))}
                <NavLink
                  href="/gui"
                  active={isActive("/gui")}
                  onNavigate={() => setMobileOpen(false)}
                >
                  Try the GUI
                </NavLink>
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
              <NavLink key={item.href} href={item.href} active={isActive(item.href)}>
                {item.label}
              </NavLink>
            ))}
          </nav>

          <div className="ml-auto flex items-center gap-1">
            {/* "Try the GUI" — the hosted web GUI try-live destination (a placeholder page
                today; the GUI track fills /gui). Prominent because the maintainer flagged GUI
                discoverability as a gap. */}
            <Button
              asChild
              variant="outline"
              size="sm"
              className="mr-1 hidden sm:inline-flex"
            >
              <Link href="/gui" aria-current={isActive("/gui") ? "page" : undefined}>
                <MonitorPlay className="size-4" aria-hidden />
                Try the GUI
              </Link>
            </Button>
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
    </CommandPalette>
  );
}
