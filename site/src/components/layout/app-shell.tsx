"use client";

import * as React from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { Github, Menu } from "lucide-react";

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
import { SidebarNav } from "@/components/layout/sidebar-nav";

const REPO_URL = "https://github.com/jeswr/sparq";

function TopTab({ href, children }: { href: string; children: React.ReactNode }) {
  const pathname = usePathname();
  const active = href === "/" ? pathname === "/" : pathname.startsWith(href);
  return (
    <Link
      href={href}
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

  return (
    <div className="flex min-h-svh flex-col">
      <div className="flex flex-1">
        {/* Persistent desktop sidebar (w-64) */}
        <aside className="hidden w-64 shrink-0 border-r bg-sidebar lg:flex lg:flex-col">
          <div className="flex h-16 items-center border-b px-4">
            <Link href="/" aria-label="sparq home" className="flex items-center">
              <Logo className="h-7 w-auto" />
            </Link>
          </div>
          <div className="flex-1 overflow-y-auto p-3">
            <SidebarNav />
          </div>
        </aside>

        <div className="flex min-w-0 flex-1 flex-col">
          {/* Sticky header (h-16, backdrop blur) */}
          <header className="sticky top-0 z-30 flex h-16 items-center gap-2 border-b bg-background/90 px-4 backdrop-blur">
            {/* Mobile hamburger → left Sheet drawer */}
            <Sheet open={mobileOpen} onOpenChange={setMobileOpen}>
              <SheetTrigger asChild>
                <Button
                  variant="ghost"
                  size="icon"
                  className="lg:hidden"
                  aria-label="Open navigation"
                >
                  <Menu className="size-5" />
                </Button>
              </SheetTrigger>
              <SheetContent side="left" className="overflow-y-auto">
                <SheetTitle className="sr-only">Navigation</SheetTitle>
                <Link
                  href="/"
                  className="mb-2 flex items-center"
                  aria-label="sparq home"
                  onClick={() => setMobileOpen(false)}
                >
                  <Logo className="h-7 w-auto" />
                </Link>
                <SidebarNav onNavigate={() => setMobileOpen(false)} />
              </SheetContent>
            </Sheet>

            <Link href="/" className="flex items-center lg:hidden" aria-label="sparq home">
              <Logo className="h-6 w-auto" />
            </Link>

            {/* Top-bar tabs (desktop) — quick jumps alongside the persistent sidebar. */}
            <nav className="hidden items-center gap-1 lg:flex" aria-label="Primary">
              <TopTab href="/">Showcase</TopTab>
              <TopTab href="/benchmarks">Benchmarks</TopTab>
              {/* [OPUS-4.8] sq-gl3cf — discoverable /download entry (the GUI/CLI download page). */}
              <TopTab href="/download">Download</TopTab>
              <TopTab href="/about">About</TopTab>
            </nav>

            <div className="ml-auto flex items-center gap-1">
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
      </div>
    </div>
  );
}
