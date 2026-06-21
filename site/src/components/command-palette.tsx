"use client";

// [OPUS-4.8] sq-vw3ax.1 — the Cmd-K command palette.
//
// WHY IT IS LOAD-BEARING (research/website-redesign.md §2, §7). The redesign collapses a
// tripled navigation (full-tree sidebar + top-tab bar + 14-card landing grid) into one slim
// top bar. That is only safe once there is a fast path to every surface that loses a visible
// nav entry — a fuzzy command palette that jumps to any surface by name in 0 clicks. So this
// ships FIRST (this bead), BEFORE the sidebar is removed (sq-vw3ax.7).
//
// SINGLE SOURCE. Everything it indexes derives from the one canonical IA source
// `@/data/surfaces` — the 5 capability THEMES (`GROUPS`) and the flagship showcases
// (`FLAGSHIPS`) — plus the fixed top-level pages and a small set of actions. Re-grouping in
// surfaces.ts restructures this palette in one edit, exactly like the Home theme grid.
//
// POST-COLLAPSE (sq-vw3ax.7). With the sidebar removed, this palette is the 0-click fast path
// to every surface, so it routes to each surface's POST-redesign home (surfaceHref): the 5
// retained deep pages keep /surface/<slug>, every other surface jumps to /capabilities#<theme>.
//
// MOUNTED ONCE. It is a global overlay mounted once in the
// AppShell so the ⌘K / Ctrl-K binding is global.

import * as React from "react";
import { useRouter } from "next/navigation";
import { Command } from "cmdk";
import { Dialog as DialogPrimitive } from "radix-ui";
import {
  Search,
  Github,
  Sun,
  Moon,
  CornerDownLeft,
  type LucideIcon,
} from "lucide-react";
import { useTheme } from "next-themes";

import { cn } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";
import {
  FLAGSHIPS,
  GROUPS,
  TIER_LABEL,
  TIER_VARIANT,
  type Surface,
} from "@/data/surfaces";
import {
  DEEP_PAGE_SLUGS,
  capabilityAnchor,
} from "@/data/capabilities";

const REPO_URL = "https://github.com/jeswr/sparq";

// [OPUS-4.8] sq-vw3ax.3 — the palette is the 0-click fast path now the sidebar is gone, so it
// must jump to each surface's NEW home, not a removed /surface/* route that only redirects.
// The 5 retained deep pages + /try keep their own route; every other surface lives as a row
// under /capabilities#<theme>. Resolving here (not at the data source) keeps surfaces.ts a pure
// structural source while the palette routes to the post-redesign IA.
function surfaceHref(surface: Surface, groupId: string): string {
  if (surface.slug === "try") return surface.href; // the live REPL keeps its own /try route
  if (DEEP_PAGE_SLUGS.has(surface.slug)) return surface.href; // retained deep page
  return capabilityAnchor(groupId); // a demo/soon row lives in the gallery
}

// A tiny context so a header trigger button (or anything else in the shell) can open the
// palette without prop-drilling — the palette owns the state, exposes setOpen via context.
const CommandPaletteContext = React.createContext<{
  open: boolean;
  setOpen: (open: boolean) => void;
} | null>(null);

/** Open / toggle the global command palette from anywhere inside <CommandPalette>. */
export function useCommandPalette() {
  const ctx = React.useContext(CommandPaletteContext);
  if (!ctx) {
    throw new Error("useCommandPalette must be used within <CommandPalette>");
  }
  return ctx;
}

// The fixed top-level destinations of the slim top bar that are NOT capability surfaces (so
// they are not in GROUPS). Option-B (sq-rclb8): "Try" (the live REPL) lives in the surface data
// (the first Query & data surface), so it is not repeated here; "App" is the SEPARATE live
// operational GUI destination (a placeholder today; the GUI track fills it) — it replaces the old
// "/gui · Try the GUI" entry, which now client-redirects to /app. /about is folded into Home
// #how-it-runs (it redirects), so the palette points there. /capabilities is the new gallery;
// /download + /app are the discovery gaps the maintainer flagged.
const TOP_PAGES: { href: string; title: string; blurb: string }[] = [
  { href: "/", title: "Home", blurb: "Overview — what sparq is, the live REPL, the flagships." },
  { href: "/capabilities", title: "Capabilities", blurb: "Every surface in one gallery, by theme." },
  { href: "/app", title: "App", blurb: "The live operational GUI (hosted web app coming soon)." },
  { href: "/benchmarks", title: "Benchmarks", blurb: "Per-commit, same-box benchmark dashboard." },
  { href: "/papers", title: "Papers", blurb: "The research papers behind sparq." },
  { href: "/download", title: "Download", blurb: "Desktop GUI + CLI/server binaries (latest release)." },
  { href: "/#how-it-runs", title: "How it runs", blurb: "The honest \"what runs where\" tier model." },
];

/** A tier badge a surface carries (flagship / surface rows render this in the right gutter). */
function TierBadge({ surface }: { surface: Surface }) {
  return (
    <Badge
      variant={TIER_VARIANT[surface.tier]}
      className="h-4 shrink-0 px-1.5 text-[10px]"
      title={TIER_LABEL[surface.tier]}
    >
      {TIER_LABEL[surface.tier]}
    </Badge>
  );
}

// [OPUS-4.8] sq-vw3ax.1 — a clean SUBSTRING/word scorer, used instead of cmdk's default
// fuzzy-subsequence one. The default scores almost every row > 0 for a short query (because
// each row's keyword soup contains the query's letters as a loose subsequence), and then DOM
// order breaks the near-ties — so "shacl" would surface a flagship rather than the SHACL row.
// This scorer only returns > 0 on a real substring hit, and weights a TITLE match far above a
// keyword/blurb match, so the obvious row wins. Case-insensitive; whitespace-normalised.
function scoreItem(value: string, search: string, keywords?: string[]): number {
  const q = search.trim().toLowerCase();
  if (!q) return 1; // empty query → show everything (cmdk renders all, DOM order)
  const title = value.toLowerCase();
  // Exact title, then title-prefix, then title-substring — the strongest signals.
  if (title === q) return 1;
  if (title.startsWith(q)) return 0.9;
  if (title.includes(q)) return 0.7;
  // Otherwise a keyword/blurb substring hit (theme label, blurb, action verbs).
  const hay = (keywords ?? []).join(" ").toLowerCase();
  if (hay.includes(q)) return 0.4;
  return 0; // no substring match → hidden
}

/** A single selectable row. `value` is the title; cmdk scores it via our custom filter. */
function PaletteItem({
  icon: Icon,
  title,
  blurb,
  keywords,
  onSelect,
  trailing,
}: {
  icon: LucideIcon;
  title: string;
  blurb?: string;
  // Secondary match terms (theme label, blurb, action verbs). cmdk scores the `value`
  // (the title) FIRST, then falls back to keywords — so "shacl" ranks the SHACL surface
  // above an unrelated row whose blurb happens to fuzzy-match, rather than diluting the
  // score by concatenating everything into one `value` string.
  keywords?: string[];
  onSelect: () => void;
  trailing?: React.ReactNode;
}) {
  return (
    <Command.Item
      value={title}
      keywords={keywords}
      onSelect={onSelect}
      className={cn(
        "flex cursor-pointer items-center gap-3 rounded-md px-3 py-2 text-sm",
        "text-foreground/90 data-[selected=true]:bg-sidebar-accent data-[selected=true]:text-sidebar-accent-foreground",
      )}
    >
      <span className="flex size-7 shrink-0 items-center justify-center rounded-md bg-primary/10 text-primary">
        <Icon className="size-4" />
      </span>
      <span className="flex min-w-0 flex-1 flex-col">
        <span className="truncate font-medium">{title}</span>
        {blurb && (
          <span className="truncate text-xs text-muted-foreground">{blurb}</span>
        )}
      </span>
      {trailing}
    </Command.Item>
  );
}

export function CommandPalette({ children }: { children: React.ReactNode }) {
  const [open, setOpen] = React.useState(false);
  const router = useRouter();
  const { theme, setTheme, resolvedTheme } = useTheme();

  const ctx = React.useMemo(() => ({ open, setOpen }), [open]);

  // Global ⌘K (macOS) / Ctrl-K (Windows/Linux) toggles the palette. We also let the platform
  // browser-search shortcut stay free: only K with the platform modifier is intercepted.
  React.useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key.toLowerCase() === "k" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        setOpen((v) => !v);
      }
    };
    document.addEventListener("keydown", onKeyDown);
    return () => document.removeEventListener("keydown", onKeyDown);
  }, []);

  // Navigate (or run an action), then close. next/navigation's router.push respects the
  // configured basePath, so "/surface/zk" resolves under /sparq on Pages and at root under
  // Tauri — no manual basePath juggling here.
  const go = React.useCallback(
    (href: string) => {
      setOpen(false);
      router.push(href);
    },
    [router],
  );

  const runAction = React.useCallback((fn: () => void) => {
    setOpen(false);
    fn();
  }, []);

  const isDark = (resolvedTheme ?? theme) === "dark";

  return (
    <CommandPaletteContext.Provider value={ctx}>
      {children}
      <DialogPrimitive.Root open={open} onOpenChange={setOpen}>
        <DialogPrimitive.Portal>
        <DialogPrimitive.Overlay
          className={cn(
            "fixed inset-0 z-50 bg-black/40 backdrop-blur-sm",
            "data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0",
          )}
        />
        <DialogPrimitive.Content
          className={cn(
            "bg-card text-card-foreground fixed left-1/2 top-[12vh] z-50 w-[min(38rem,calc(100vw-2rem))] -translate-x-1/2",
            "overflow-hidden rounded-xl p-0 shadow-2xl ring-1 ring-foreground/10",
            "data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95",
          )}
        >
          {/* The Title is the dialog's accessible name (radix links aria-labelledby to it). */}
          <DialogPrimitive.Title className="sr-only">
            Command palette
          </DialogPrimitive.Title>
          <DialogPrimitive.Description className="sr-only">
            Type to fuzzy-search every sparq surface, page, and quick action, then press Enter
            to jump to it.
          </DialogPrimitive.Description>

          <Command
            label="Search surfaces, pages and actions"
            className="flex flex-col"
            filter={scoreItem}
          >
            <div className="flex items-center gap-2 border-b px-3">
              <Search className="size-4 shrink-0 text-muted-foreground" aria-hidden />
              <Command.Input
                autoFocus
                placeholder="Search surfaces, pages, actions…"
                className="h-12 flex-1 bg-transparent text-sm outline-none placeholder:text-muted-foreground"
              />
              <kbd className="hidden rounded border px-1.5 py-0.5 text-[10px] font-medium text-muted-foreground sm:inline-block">
                ESC
              </kbd>
            </div>

            <Command.List className="max-h-[min(60vh,28rem)] overflow-y-auto p-2">
              <Command.Empty className="px-3 py-6 text-center text-sm text-muted-foreground">
                No matches. Try a surface name, &ldquo;benchmarks&rdquo;, or &ldquo;theme&rdquo;.
              </Command.Empty>

              {/* Flagship showcases first — the strongest proof artifacts. */}
              <Command.Group
                heading="Flagship showcases"
                className="px-1 pb-1 text-xs font-medium text-muted-foreground [&_[cmdk-group-heading]]:px-2 [&_[cmdk-group-heading]]:py-1.5"
              >
                {FLAGSHIPS.map((f) => (
                  <PaletteItem
                    key={f.href}
                    icon={f.icon}
                    title={f.title}
                    blurb={f.blurb}
                    keywords={["showcase", "flagship", f.blurb]}
                    onSelect={() => go(f.href)}
                    trailing={<TierBadge surface={f} />}
                  />
                ))}
              </Command.Group>

              {/* The 5 capability THEMES — every surface, derived from the single GROUPS source. */}
              {GROUPS.map((group) => (
                <Command.Group
                  key={group.id}
                  heading={group.label}
                  className="px-1 pb-1 text-xs font-medium text-muted-foreground [&_[cmdk-group-heading]]:px-2 [&_[cmdk-group-heading]]:py-1.5"
                >
                  {group.surfaces.map((s) => (
                    <PaletteItem
                      key={s.href}
                      icon={s.icon}
                      title={s.title}
                      blurb={s.blurb}
                      // The theme label + blurb are keywords so a theme search surfaces its
                      // members, without diluting the title-driven primary score.
                      keywords={[group.label, s.blurb]}
                      // Jump to the surface's POST-redesign home: a retained deep page keeps
                      // /surface/<slug>; every other surface lives at /capabilities#<theme>.
                      onSelect={() => go(surfaceHref(s, group.id))}
                      trailing={<TierBadge surface={s} />}
                    />
                  ))}
                </Command.Group>
              ))}

              {/* Fixed top-level pages (the slim top bar). */}
              <Command.Group
                heading="Pages"
                className="px-1 pb-1 text-xs font-medium text-muted-foreground [&_[cmdk-group-heading]]:px-2 [&_[cmdk-group-heading]]:py-1.5"
              >
                {TOP_PAGES.map((p) => (
                  <PaletteItem
                    key={p.href}
                    icon={Search}
                    title={p.title}
                    blurb={p.blurb}
                    keywords={["page", p.blurb]}
                    onSelect={() => go(p.href)}
                  />
                ))}
              </Command.Group>

              {/* Quick actions. */}
              <Command.Group
                heading="Actions"
                className="px-1 pb-1 text-xs font-medium text-muted-foreground [&_[cmdk-group-heading]]:px-2 [&_[cmdk-group-heading]]:py-1.5"
              >
                <PaletteItem
                  icon={isDark ? Sun : Moon}
                  title={isDark ? "Switch to light theme" : "Switch to dark theme"}
                  keywords={["action", "toggle", "theme", "dark", "light", "mode"]}
                  onSelect={() => runAction(() => setTheme(isDark ? "light" : "dark"))}
                />
                <PaletteItem
                  icon={Github}
                  title="Open sparq on GitHub"
                  blurb={REPO_URL}
                  keywords={["action", "github", "source", "repository", "code"]}
                  onSelect={() =>
                    runAction(() =>
                      window.open(REPO_URL, "_blank", "noopener,noreferrer"),
                    )
                  }
                />
              </Command.Group>
            </Command.List>

            <div className="flex items-center justify-between gap-2 border-t px-3 py-2 text-[11px] text-muted-foreground">
              <span className="inline-flex items-center gap-1">
                <CornerDownLeft className="size-3" aria-hidden /> to open
              </span>
              <span>
                Open with{" "}
                <kbd className="rounded border px-1 py-0.5 font-medium">⌘K</kbd> /{" "}
                <kbd className="rounded border px-1 py-0.5 font-medium">Ctrl K</kbd>
              </span>
            </div>
          </Command>
        </DialogPrimitive.Content>
        </DialogPrimitive.Portal>
      </DialogPrimitive.Root>
    </CommandPaletteContext.Provider>
  );
}

/**
 * The header trigger — a slim "search" affordance showing the ⌘K hint, plus an icon-only
 * variant for the mobile bar. Clicking it opens the same palette the keyboard shortcut does.
 */
export function CommandPaletteTrigger({ className }: { className?: string }) {
  const { setOpen } = useCommandPalette();
  return (
    <button
      type="button"
      onClick={() => setOpen(true)}
      aria-label="Search (Command-K)"
      className={cn(
        "inline-flex h-9 items-center gap-2 rounded-md border bg-background px-3 text-sm text-muted-foreground transition-colors hover:bg-muted hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/50",
        className,
      )}
    >
      <Search className="size-4" aria-hidden />
      <span className="hidden sm:inline">Search…</span>
      <kbd className="ml-1 hidden rounded border px-1.5 py-0.5 text-[10px] font-medium sm:inline-block">
        ⌘K
      </kbd>
    </button>
  );
}
